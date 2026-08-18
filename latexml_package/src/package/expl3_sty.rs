use crate::prelude::*;

/// Run a chunk of our own expl3-syntax TeX under the expl3 catcode regime.
///
/// **Every `raw_tex` chunk in this file that names an expl3 CS must go through
/// here.** Real expl3 code always lives between `\ExplSyntaxOn` and
/// `\ExplSyntaxOff`, which make `:` and `_` LETTER so a name like
/// `\c_sys_shell_escape_int` is a SINGLE control sequence. `raw_tex` tokenizes
/// with the **ambient** catcodes, and after the expl3 load the ambient
/// (document) regime has `_` = SUB (8) — so an unwrapped chunk mis-tokenizes:
/// the CS terminates at the first `_`, and `\edef\c_sys_shell_escape_int{0}`
/// becomes `\edef\c` with parameter text `_sys_shell_escape_int` and body `0`,
/// silently rebinding LaTeX's cedilla accent `\c` (`Fran\c cois` → "Fran0cois",
/// no error at all, where Perl renders "François"). That was issue 421 —
/// witness arXiv 2605.11579; see the NOTE at the deleted `\c_sys_*` block.
///
/// Catcode-INDEPENDENT alternatives, when they fit, are safer still: `T_CS!` /
/// `Let!` / `parse_prototype` (`def_macro_noop` &c.) build the CS name as a
/// string and never reach the tokenizer.
///
/// We cannot delegate to `\ExplSyntaxOn`/`\ExplSyntaxOff` here: our expl3
/// kernel's `\ExplSyntaxOff` is incomplete (the reason xparse_sty.rs and
/// siunitx_sty.rs hardcode a `~`/`:`/`_` restore after their raw loads —
/// docs/parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md). Instead this
/// saves the ambient catcodes of `:`/`_`, installs the expl3 LETTER regime for
/// the duration, and restores the saved values — including on the error path,
/// so a failing chunk cannot leak the LETTER regime into the document.
fn with_expl_catcodes<T>(body: impl FnOnce() -> Result<T>) -> Result<T> {
  // (char, catcode to fall back to if the ambient table has no entry — the
  // document/plain-TeX default for that char).
  const EXPL_LETTERS: [(char, Catcode); 2] = [(':', Catcode::OTHER), ('_', Catcode::SUB)];
  let saved = EXPL_LETTERS.map(|(c, dflt)| lookup_catcode(c).unwrap_or(dflt));
  for (c, _) in EXPL_LETTERS {
    assign_catcode(c, Catcode::LETTER, Some(Scope::Global));
  }
  let result = body();
  for ((c, _), cc) in EXPL_LETTERS.into_iter().zip(saved) {
    assign_catcode(c, cc, Some(Scope::Global));
  }
  result
}

#[rustfmt::skip]
LoadDefinitions!({
  // Strict-Perl translation of LaTeXML/lib/LaTeXML/Package/expl3.sty.ltxml:
  //   LoadPool('LaTeX');
  //   InputDefinitions('expl3', type => 'lua');
  //   InputDefinitions('expl3', type => 'sty', noltxml => 1);
  //
  // The raw expl3.sty file has a TeX-level guard
  //   \expandafter\ifx\csname tex_let:D\endcsname\relax
  //     \expandafter\@firstofone\else\expandafter\@gobble\fi
  //     {\input expl3-code.tex }%
  // which detects the dump-loaded `\tex_let:D` PA-alias and skips
  // re-loading expl3-code.tex. So this 3-line wrapper does the right
  // thing: load lua portion, then load .sty (which short-circuits).
  LoadPool!("LaTeX");
  InputDefinitions!("expl3", extension => Some(Cow::Borrowed("lua")), notex => true);

  // Mirror expl3.sty's TeX-level guard so we know whether the .sty load
  // about to run will actually `\input expl3-code.tex` (cascade-prone
  // raw load) or short-circuit it (dump path). The guard inside
  // expl3.sty is `\ifx\csname tex_let:D\endcsname\relax {\input ...}`,
  // so an undefined `\tex_let:D` here ⇒ raw load will fire.
  let raw_load_will_run = lookup_meaning(&T_CS!("\\tex_let:D")).is_none();

  // In degraded no-dump mode the raw expl3.sty load re-runs expl3-code.tex,
  // whose group-end codepoint block (L33074-33180) trips known raw-load-only
  // cascades that the dump avoids and that Perl's own raw-load does not produce
  // (Perl converts `\usepackage{fvextra}` with 0 errors). They leave expl3's
  // runtime usable — the document still converts correctly — but their benign
  // error/fatal records would otherwise count against the CONVERSION, so a
  // perfectly good no-dump run reports "Conversion failed: 1 fatal error" (issue
  // #651: a bare `\usepackage{fvextra}`, whose output is a healthy
  // `<p>text</p>`). Snapshot the report across this raw load and restore it —
  // and mute the spurious stderr lines — so only the DOCUMENT's own diagnostics
  // reach the conversion verdict. Dump mode short-circuits the re-load
  // (`raw_load_will_run == false`), so this is scoped strictly to the degraded
  // fallback; canvas/parity always run on the dump.
  use latexml_core::common::error::{REPORT, set_suppress_log_output};
  let report_snapshot = raw_load_will_run.then(|| REPORT.borrow().clone());
  let prev_suppress = raw_load_will_run.then(|| set_suppress_log_output(true));

  let _ = input_definitions("expl3", NewDefault!(InputDefinitionOptions,
    noltxml => true, extension => Some(Cow::Borrowed("sty"))));

  if let Some(snapshot) = report_snapshot {
    *REPORT.borrow_mut() = snapshot;
  }
  if let Some(prev) = prev_suppress {
    set_suppress_log_output(prev);
  }

  // Post-load fixup for `\__kernel_msg_info:nnxx`. xparse-2018-04-12.sty
  // (line 101, 112, 218, 222) calls `\__kernel_msg_info:nnxx { xparse }
  // { define-command }` etc. for every `\NewDocumentCommand`, but
  // expl3-code.tex defines only the `:nnee` variant — `:nnxx` is a
  // deprecated argument-spec letter (`x` = e-expanded, replaced by
  // `e` in modern expl3) that xparse-2018-04-12 expects but expl3
  // never auto-generates. Without this stub the CS is undefined →
  // generate_error_stub installs an `<ltx:ERROR>` Constructor and
  // EVERY `\NewDocumentCommand` invocation leaks the error element
  // plus the unused message-body args into document text.
  //
  // We define `\__kernel_msg_info:nnxx` as a 4-arg no-op, matching
  // Perl LaTeXML's effective end-state (`\msg_info:nnxx` is a
  // log-only path; we have no log channel so a no-op is the closest
  // equivalent).
  //
  // The historical "\cs_end: cascade" that this stub also masked was
  // root-caused and fixed in latexml_core/src/binding/content.rs:
  // \@pushfilename now runs BEFORE \@currname/\@currext are set,
  // matching latex.ltx:15518-15519. With that fix the prior need to
  // also stub `\g__file_record_seq` is gone.
  //
  // GATE: only install when the raw .sty actually re-loaded
  // expl3-code.tex. On the dump path the guard short-circuits the
  // re-load and the dump already provides the right state.
  if raw_load_will_run {
    with_expl_catcodes(|| raw_tex(r"\protected\gdef\__kernel_msg_info:nnxx#1#2#3#4{}"))?;
  }

  // expl3 case-folding override.
  //
  // The kernel `\__kernel_codepoint_case:nn` walks per-codepoint case maps
  // built from `c__codepoint_<case>_<cp>_tl` constants. Those are populated
  // by reading UnicodeData.txt / CaseFolding.txt / SpecialCasing.txt during
  // expl3-code.tex's group-end block at L33074-33180. Our raw expl3 load
  // currently fails to open those files (the `ior_open` chain trips on a
  // file_input dispatch issue tracked separately), leaving the codepoint
  // tables empty — so `\str_lowercase:n {Hello}` returns "Hello" unchanged.
  //
  // Override the kernel function with a Rust impl using `char::to_lowercase`
  // and `char::to_uppercase` from std. Returns a triple `{cp1}{cp2}{cp3}`
  // matching expl3's expected return contract — first slot is the primary
  // result codepoint, slots 2/3 hold combining chars for compound mappings
  // (e.g. "ß" → "SS" upper has slot1=S, slot2=S; we model only single-cp
  // mappings here, leaving slots 2/3 blank). For ASCII this is exact; for
  // non-Latin scripts that map to multi-char sequences (Latin extended,
  // Greek, etc.) Rust's std char::to_lowercase yields the right primary cp.
  DefMacro!(T_CS!("\\__kernel_codepoint_case:nn"), "{}{}", sub[(case_type, cp_str)] {
    let case = case_type.to_string().to_lowercase();
    let cp_text = cp_str.to_string();
    let cp_n: u32 = cp_text.trim().parse().unwrap_or(0);
    let result_cp = if cp_n == 0 {
      0u32
    } else if let Some(c) = char::from_u32(cp_n) {
      let folded: String = match case.as_str() {
        "lowercase" | "casefold" => c.to_lowercase().collect(),
        "uppercase" | "titlecase" => c.to_uppercase().collect(),
        _ => c.to_string(),
      };
      folded.chars().next().map(|fc| fc as u32).unwrap_or(cp_n)
    } else {
      cp_n
    };
    Ok(Tokenize!(TeXString::assembled(format!("{{{}}}{{}}{{}}", result_cp))))
  });

  // `\c_sys_jobname_str` is one of the system-info constants bound by
  // `\g__sys_everyjob_tl` at job start (via `\everyjob`), which our engine
  // never fires (matching Perl's gap). When packages like `duckuments.sty`
  // then do
  //   `\str_if_eq_p:Vn \c_sys_jobname_str { example-image-duck }`
  // the V-expansion triggers `\if_int_compare:w` cascades on Rust
  // (Perl emits one undefined error and recovers; Rust's recovery
  // re-fires per scan, surfacing 21+ relational-token cascades).
  //
  // A plain `\Let` alias to `\jobname`, rather than the full `\str_const:Ne`
  // machinery — those expl3 constructors themselves require a working
  // `\c_sys_jobname_str` at definition time. The sibling date/time int
  // constants need NO such patch-up: they are already defined, with live
  // values, by the time a package body runs (see the NOTE below).
  //
  // Driver: 2406.14142 (duckuments cascade, 21 errors → 4 expected
  // (matching Perl's residual undefined-CS count)).
  Let!("\\c_sys_jobname_str", "\\jobname");

  // expl3 historical-alias: `\hbox_unpack_clear:N` was deprecated
  // around 2018 in favor of `\hbox_unpack_drop:N` (both call `\unhbox`
  // — read out the box's contents AND clear/drop the box itself).
  // Modern l3kernel no longer ships the alias, so our dump doesn't
  // contain it, but third-party expl3 packages (mmacells.sty,
  // letgut-lstlang.sty) still call the deprecated name. Add the
  // alias post-dump so those packages load cleanly. Witness:
  // arXiv:2002.07146 (uses `\usepackage{mmacells}`).
  // (See also `\hbox_unpack:N` in dump = `\unhcopy` which is the
  // non-clearing version.)
  Let!("\\hbox_unpack_clear:N", "\\hbox_unpack_drop:N");

  // NOTE (issue 421): there used to be a `RawTeX!` block here `\edef`-ing the
  // seven `\c_sys_{minute,hour,day,month,year}_int` / `\c_sys_timestamp_str` /
  // `\c_sys_shell_escape_int` constants, on the belief that they stay undefined
  // because we never fire `\everyjob`. **Do not re-add it.** Both halves of that
  // belief were wrong, measured on the current engine:
  //
  //  * They ARE defined, at package-load time and with live values — probing
  //    `\number\csname c_sys_year_int\endcsname` right after `\usepackage{xparse}`
  //    gives the real year, and `c_sys_minute_int` advances between runs. Only
  //    `\c_sys_jobname_str` needed the `Let!` above. (That is the dump path. On
  //    the `LATEXML_NODUMP=1` raw-load branch they come back undefined — but the
  //    block did not define them there either, so nothing changed; that branch
  //    dies earlier anyway, on the expl3-code codepoint group.)
  //  * The block never actually ran. Written as raw TeX, it was tokenized with
  //    the AMBIENT catcodes, and after the expl3 load the document regime has
  //    `_` = SUB — so `\edef\c_sys_minute_int{0}` parsed as `\edef\c` with
  //    parameter text `_sys_minute_int`. It defined none of the constants and
  //    instead REBOUND LaTeX's cedilla accent `\c` (`\meaning\c` =
  //    `macro:_sys_shell_escape_int->0`), so every later `Fran\c cois` rendered
  //    "Fran0cois" — silently, 0 errors, where Perl renders "François".
  //    Witness arXiv 2605.11579; guard
  //    `expl3_load_does_not_clobber_cedilla_accent`.
  //
  // Fixing only the tokenization would have been worse than deleting: the
  // constants would then overwrite expl3's live clock values with frozen
  // dummies (and a hardcoded year). Perl's expl3.sty.ltxml has no such block
  // either — it is three lines. Any FUTURE expl3-syntax raw TeX added to this
  // file must go through `with_expl_catcodes`.

  // expl3 l3regex is handled NATIVELY by the real expl3 VM (loaded from
  // expl3-code.tex: \__regex_compile:n / \__regex_build:n / \__regex_match:n,
  // etc.), which now runs correctly under our gullet — so \regex_match,
  // \regex_count, \regex_replace_all, \regex_extract_once and their \seq_*
  // results all work faithfully. A former Rust-`regex`-crate SHIM bypassed the
  // VM (its \if_int_compare:w-driven expansion used to stall); it was REMOVED
  // 2026-06-20 after intervening gullet fixes made the real VM pass — verified
  // on its original cascade witness 2406.14142 (21 errors -> 0) and the
  // expl3/regex_match + expl3/regex_native test fixtures. (Shim in git history.)

  assign_catcode(':', Catcode::OTHER, Some(Scope::Global));
  assign_catcode('_', Catcode::SUB, Some(Scope::Global));
  // Also restore `~` to ACTIVE (13). `\usepackage{expl3}` in real LaTeX
  // leaves expl3 syntax OFF — `~` back to its document catcode (13/active),
  // not the expl3 `~`=10 (SPACE). Our raw expl3-code.tex load leaves `~` at
  // 10 because Rust's `\ExplSyntaxOff` is incomplete; the `:`/`_` restores
  // above already turn expl3-letters off, but `~` was missed. Without this, a
  // LATER `\usepackage[english]{babel}` runs `\initiate@active@char{~}` with
  // `~` at catcode 10 → an `expected:<relationaltoken>` cascade. Minimal
  // repro `\usepackage{expl3}\usepackage[english]{babel}` (2 errors → 0).
  // `~` is not an expl3 LETTER char (unlike `:`/`_`), so this is glossary-safe.
  // Complements the per-package restore in xparse_sty.rs / siunitx_sty.rs.
  assign_catcode('~', Catcode::ACTIVE, Some(Scope::Global));
});

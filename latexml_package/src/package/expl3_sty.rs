use crate::prelude::*;
use regex::Regex;

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

  let _ = input_definitions("expl3", NewDefault!(InputDefinitionOptions,
    noltxml => true, extension => Some(Cow::Borrowed("sty"))));

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
    state::assign_catcode(':', Catcode::LETTER, Some(Scope::Global));
    state::assign_catcode('_', Catcode::LETTER, Some(Scope::Global));
    raw_tex(r"\protected\gdef\__kernel_msg_info:nnxx#1#2#3#4{}")?;
    state::assign_catcode(':', Catcode::OTHER, Some(Scope::Global));
    state::assign_catcode('_', Catcode::SUB, Some(Scope::Global));
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
    Ok(Tokenize!(&format!("{{{}}}{{}}{{}}", result_cp)))
  });

  // expl3 system-info constants normally bound by `\g__sys_everyjob_tl`
  // expansion at job start (via `\everyjob`). Our engine never fires
  // `\everyjob` (matching Perl's gap), so the tl never runs and these
  // CSes stay undefined. When packages like `duckuments.sty` then do
  //   `\str_if_eq_p:Vn \c_sys_jobname_str { example-image-duck }`
  // the V-expansion triggers `\if_int_compare:w` cascades on Rust
  // (Perl emits one undefined error and recovers; Rust's recovery
  // re-fires per scan, surfacing 21+ relational-token cascades).
  //
  // Mirror the body of `\g__sys_everyjob_tl` for the constants
  // duckuments-class packages actually consume — `\c_sys_jobname_str`
  // (= jobname) plus the date/time int constants. Use plain `\Let`/
  // `\edef` aliases rather than the full `\str_const:Ne` machinery
  // because those expl3 constructors themselves require a working
  // `\c_sys_jobname_str` at definition time.
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

  RawTeX!(r"
    \edef\c_sys_minute_int{0}%
    \edef\c_sys_hour_int{0}%
    \edef\c_sys_day_int{1}%
    \edef\c_sys_month_int{1}%
    \edef\c_sys_year_int{2026}%
    \edef\c_sys_timestamp_str{}%
    \edef\c_sys_shell_escape_int{0}%
  ");

  // expl3 regex matching is implemented via a deeply intertwined chain of
  // `\__regex_compile:n`, `\__regex_match:n`, `\__regex_if_match:nn`, etc.
  // All of those routines drive `\if_int_compare:w` against
  // `\l__regex_*_int` expl3 registers in ways that exercise gullet-level
  // macro expansion timing very precisely. Our Rust port doesn't faithfully
  // reproduce the timing, so when packages like `duckuments.sty` invoke
  // `\regex_match:NnTF` against `\c_duckuments_example_regex` (a compiled
  // regex constant), the expansion stalls and emits a 21-error cascade of
  // `Error:expected:<relationaltoken>` + `Error:unexpected:fi:` at end of
  // document.
  //
  // Native Rust implementation of expl3 regex matching.
  //
  // Background: expl3's `\regex_match:NnTF` (and friends) is built atop a
  // VM-style regex engine (`\__regex_compile:n`, `\__regex_match:n`, etc.)
  // whose macros drive `\if_int_compare:w` against `\l__regex_*_int`
  // registers in a way that exercises gullet expansion timing very
  // precisely. Our Rust port doesn't faithfully reproduce that timing,
  // so when packages like duckuments.sty invoke `\regex_match:NnTF`
  // against a `\regex_const:Nn`-compiled regex, the expansion stalls and
  // emits a 21-error cascade at end-of-document. Driver: 2406.14142.
  //
  // We bypass the expl3 VM entirely: store the raw pattern string at
  // `\regex_const:Nn` time keyed by CS name, and compile + match using
  // Rust's `regex` crate at `\regex_match:NnTF` time. The expl3 regex
  // syntax is largely PCRE-compatible (`\d`, `[:class:]`, `(?:...)`,
  // alternation, quantifiers) — Rust's regex crate handles these without
  // translation. Patterns that use expl3-specific extensions (e.g.
  // `\c{cs_name}` for control-sequence literals) will fail to compile
  // and silently take the FALSE branch.
  //
  // Verified: 2406.14142: 21 errors → 0 (was last historical
  // REAL_REGRESSION). Min-repro `\regex_match:nnTF{\d+}{abc}{T}{F}`
  // now correctly returns F (matches Perl LaTeXML behaviour).
  state::assign_catcode(':', Catcode::LETTER, Some(Scope::Global));
  state::assign_catcode('_', Catcode::LETTER, Some(Scope::Global));

  // \regex_const:Nn \cs {pattern} — store pattern keyed by CS name.
  DefMacro!(T_CS!("\\regex_const:Nn"), "DefToken {}", sub[(cs, pattern)] {
    let cs_name = cs.to_string();
    let pattern_str = pattern.to_string();
    state::assign_value(&format!("regex_pattern:{}", cs_name),
                        Stored::String(arena::pin(&pattern_str)),
                        Some(Scope::Global));
  });

  // \regex_match:NnTF \cs {target} {T} {F}
  DefMacro!(T_CS!("\\regex_match:NnTF"), "DefToken {}{}{}",
    sub[(cs, target, t_toks, f_toks)] {
    let cs_name = cs.to_string();
    let target_str = target.to_string();
    let key = format!("regex_pattern:{}", cs_name);
    let pattern = state::with_value(&key, |v| match v {
      Some(Stored::String(s)) => arena::to_string(*s),
      _ => String::new(),
    });
    let matches = !pattern.is_empty() && match Regex::new(&expl3_to_rust_regex(&pattern)) {
      Ok(re) => re.is_match(&target_str),
      Err(_) => false,
    };
    if matches { t_toks.unlist() } else { f_toks.unlist() }
  });
  DefMacro!(T_CS!("\\regex_match:NnT"), "DefToken {}{}",
    sub[(cs, target, t_toks)] {
    let cs_name = cs.to_string();
    let target_str = target.to_string();
    let key = format!("regex_pattern:{}", cs_name);
    let pattern = state::with_value(&key, |v| match v {
      Some(Stored::String(s)) => arena::to_string(*s),
      _ => String::new(),
    });
    let matches = !pattern.is_empty() && match Regex::new(&expl3_to_rust_regex(&pattern)) {
      Ok(re) => re.is_match(&target_str),
      Err(_) => false,
    };
    if matches { t_toks.unlist() } else { Vec::new() }
  });
  DefMacro!(T_CS!("\\regex_match:NnF"), "DefToken {}{}",
    sub[(cs, target, f_toks)] {
    let cs_name = cs.to_string();
    let target_str = target.to_string();
    let key = format!("regex_pattern:{}", cs_name);
    let pattern = state::with_value(&key, |v| match v {
      Some(Stored::String(s)) => arena::to_string(*s),
      _ => String::new(),
    });
    let matches = !pattern.is_empty() && match Regex::new(&expl3_to_rust_regex(&pattern)) {
      Ok(re) => re.is_match(&target_str),
      Err(_) => false,
    };
    if matches { Vec::new() } else { f_toks.unlist() }
  });

  // \regex_match:nnTF {pattern} {target} {T} {F} — pattern inline.
  DefMacro!(T_CS!("\\regex_match:nnTF"), "{}{}{}{}",
    sub[(pattern, target, t_toks, f_toks)] {
    let pattern_str = pattern.to_string();
    let target_str = target.to_string();
    let matches = match Regex::new(&expl3_to_rust_regex(&pattern_str)) {
      Ok(re) => re.is_match(&target_str),
      Err(_) => false,
    };
    if matches { t_toks.unlist() } else { f_toks.unlist() }
  });
  DefMacro!(T_CS!("\\regex_match:nnT"), "{}{}{}",
    sub[(pattern, target, t_toks)] {
    let pattern_str = pattern.to_string();
    let target_str = target.to_string();
    let matches = match Regex::new(&expl3_to_rust_regex(&pattern_str)) {
      Ok(re) => re.is_match(&target_str),
      Err(_) => false,
    };
    if matches { t_toks.unlist() } else { Vec::new() }
  });
  DefMacro!(T_CS!("\\regex_match:nnF"), "{}{}{}",
    sub[(pattern, target, f_toks)] {
    let pattern_str = pattern.to_string();
    let target_str = target.to_string();
    let matches = match Regex::new(&expl3_to_rust_regex(&pattern_str)) {
      Ok(re) => re.is_match(&target_str),
      Err(_) => false,
    };
    if matches { Vec::new() } else { f_toks.unlist() }
  });

  state::assign_catcode(':', Catcode::OTHER, Some(Scope::Global));
  state::assign_catcode('_', Catcode::SUB, Some(Scope::Global));
});

/// Translate an expl3 regex pattern string into a Rust regex.
///
/// expl3 syntax is PCRE-style. Rust's `regex` crate handles most of
/// the common cases (`\d`, `\w`, `[abc]`, `(?:...)`, `|`, `*`, `+`,
/// `?`, `{n,m}`) directly. expl3 patterns spread across multiple
/// lines (as in `\regex_const:Nn` brace-delimited bodies) include
/// significant whitespace; expl3's lexer ignores horizontal whitespace
/// inside the pattern by default, so we mirror that by stripping
/// newlines + leading/trailing whitespace from each line.
///
/// Patterns using expl3-specific extensions (`\c{...}`, `\u{...}`)
/// pass through; the regex crate will likely return Err on those, and
/// the caller falls through to the FALSE branch.
fn expl3_to_rust_regex(pattern: &str) -> String {
  pattern
    .lines()
    .map(str::trim)
    .filter(|l| !l.is_empty())
    .collect::<Vec<&str>>()
    .join("")
}

use crate::prelude::*;

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
});

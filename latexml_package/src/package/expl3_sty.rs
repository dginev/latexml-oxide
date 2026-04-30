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
  let _ = input_definitions("expl3", NewDefault!(InputDefinitionOptions,
    noltxml => true, extension => Some(Cow::Borrowed("sty"))));

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

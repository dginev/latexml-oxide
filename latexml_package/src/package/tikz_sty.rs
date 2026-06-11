use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: tikz.sty.ltxml (57 lines)
  // TODO: Full port requires InputDefinitions("tikz", noltxml => true) which loads
  // the raw TeX tikz package. Also needs:
  // 1. \use@@tikzlibrary{} — DefPrimitive that loads tikzlibrary*.code.tex files
  // 2. \tikzcdset — redirect to pgfqkeys
  // 3. pgf infrastructure (pgfsys-latexml.def)
  //
  // Perl source: LaTeXML/lib/LaTeXML/Package/tikz.sty.ltxml
  // TikZ documents generate many warnings from unported pgf primitives.
  // Increase MAX_ERRORS to allow processing to complete.
  AssignValue!("MAX_ERRORS" => Stored::Int(1000));

  DefMacro!("\\pgfmathresult", "0.0");
  DefMacro!("\\tikz@align@temp", "\\pgfmathresult");
  InputDefinitions!("tikz", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // Perl L35-52: \use@@tikzlibrary — loads tikz/pgf library files
  // Makes sure libraries are loaded as definitions (InputDefinitions) rather than
  // content, with catcode management for | character.
  DefPrimitive!("\\use@@tikzlibrary{}", sub[(libs)] {
    let mut libs_str = do_expand(libs)?.to_string();
    // tikzlibrarycd.code.tex checks for `decorations.pathmorphing`
    // and `decorations.markings` @loaded flags at squiggly/markings
    // arrow expansion time and emits a vendor PackageError if they're
    // missing. arXiv's local pdflatex compiles these papers cleanly
    // because the libraries are autoloaded by pdftex's TikZ; mirror
    // by injecting both as deps whenever the cd library is requested.
    // Papers commonly use `\usetikzlibrary{cd}` directly (without
    // `\usepackage{tikz-cd}`), so the tikz_cd_sty binding never runs.
    // Witness 2306.03232 (squiggly), 2308.06778 (markings).
    if libs_str.split(',').any(|s| s.trim() == "cd") {
      libs_str = s!("decorations.pathmorphing,decorations.markings,{libs_str}");
    }
    for lib in libs_str.split(',') {
      let lib = lib.trim();
      if lib.is_empty() {
        continue;
      }
      let loaded_cs = T_CS!(s!("\\tikz@library@{lib}@loaded"));
      if lookup_definition(&loaded_cs)?.is_some() {
        continue; // already loaded
      }
      Info!("tikz", lib, &s!("TIKZ LIBRARY {lib}"));
      Let!(loaded_cs.clone(), T_CS!("\\pgfutil@empty"));
      // Save and set catcode for | to OTHER (Perl L42-50)
      let bar_cc = lookup_catcode('|');
      assign_catcode('|', Catcode::OTHER, None);
      // Try tikzlibrary*.code.tex first, then pgflibrary*.code.tex
      let tikz_name = s!("tikzlibrary{lib}.code");
      let pgf_name = s!("pgflibrary{lib}.code");
      let tikz_opts = NewDefault!(InputDefinitionOptions,
        noerror => true, extension => Some(Cow::Borrowed("tex")));
      let pgf_opts = NewDefault!(InputDefinitionOptions,
        noerror => true, extension => Some(Cow::Borrowed("tex")));
      if input_definitions(&tikz_name, tikz_opts).is_err()
        && input_definitions(&pgf_name, pgf_opts).is_err() {
          Warn!("missing_file", lib,
            &s!("Can't find tikz library '{}' (neither pgf nor tikz). Anticipate undefined macros.", lib));
        }
      // Restore catcode for |
      if let Some(cc) = bar_cc {
        assign_catcode('|', cc, None);
      }
    }
    Ok(Vec::new())
  });

  DefMacro!("\\tikzcdset", "\\pgfqkeys{/tikz/commutative diagrams}");
});

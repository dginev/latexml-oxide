use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "mathscinet.sty",
    "mathscinet.sty is only minimally stubbed and will not be interpreted raw."
  );

  // AMS MathSciNet's `mathscinet.sty` is shipped only with the MathSciNet
  // export tooling, not TeX Live. Common in math-paper bibliographies
  // because MathSciNet's BBL export uses these shortcuts. Bindings:
  //
  //   \scr  → \mathcal   (script font; called like `$\scr O$`)
  //   \sci  → italic (already)
  //   \roman handled by core LaTeX
  //
  // Two-grep audit: neither in Perl `*.ltxml` nor in TL. Witness:
  // arXiv:2508.10772 — `$\scr O$` in `main.bbl` → Rust 1 error → 0.
  Let!("\\scr",     "\\mathcal");
  Let!("\\msnscr",  "\\mathcal");
});

//! Shared post-raw-load patches for the KOMA-Script classes (scrartcl /
//! scrbook / scrreprt). The classes themselves are raw-interpreted (see
//! `scrartcl_cls.rs`); this module only re-targets the handful of commands
//! whose real definitions are print-only and would otherwise LOSE author
//! content or bury a heading in a presentational paragraph.
use latexml_package::prelude::*;

/// Applied after the raw `.cls` finished loading, so every definition here
/// deliberately overrides the class's own.
pub(crate) fn koma_post_load() -> Result<()> {
  // Title-page pieces. scrartcl.cls L2768-2803 store these into `\@extratitle`
  // … `\@dedication` for KOMA's own `\maketitle` (L2815) to typeset — but
  // `\maketitle` is a locked constructor here (the class's redefinition is
  // ignored, like `\@maketitle`), so the stored text would never reach the
  // XML. Re-target the setters at the frontmatter. `\subtitle` is a real
  // ltx:subtitle; the rest keep the `ltx:note[role]` shape the former stub
  // established (witness 2305.01582, ar5iv #498: a `\titlehead` banner with
  // the software name + `\giturl`). Neither Perl nor upstream LaTeXML binds
  // these (no scrartcl.cls.ltxml) — surpass-Perl content recovery.
  DefMacro!("\\subtitle{}", "\\@add@frontmatter{ltx:subtitle}{#1}");
  for role in [
    "titlehead",
    "subject",
    "publishers",
    "extratitle",
    "frontispiece",
    "uppertitleback",
    "lowertitleback",
    "dedication",
  ] {
    RawTeX!(&s!(
      r"\def\{role}#1{{\@add@frontmatter{{ltx:note}}[role={role}]{{#1}}}}"
    ));
  }
  // `\minisec{title}` (scrartcl.cls L5081-5100): an unnumbered, un-TOC'd
  // freestanding heading. The real body is `\usekomafont{minisec}{#1\par}`
  // inside a `\parbox`-free group — a bold sans paragraph in the XML, no
  // heading semantics. The starred paragraph heading is the closest
  // structural equivalent and keeps the title as
  // `<ltx:paragraph><ltx:title>` (TL doc corpus: 17 bundles).
  DefMacro!("\\minisec{}", "\\paragraph*{#1}");
  Ok(())
}

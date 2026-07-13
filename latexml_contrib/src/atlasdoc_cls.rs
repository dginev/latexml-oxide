//! atlasdoc.cls — the ATLAS Collaboration paper/note class (author-bundled, very
//! large; not raw-loaded). A full binding (hundreds of physics-notation macros,
//! dozens of bundled `atlas*.sty`) is out of scope. This binds only the
//! FRONT-MATTER macros that otherwise leak as literal text — the reported issue
//! (witness 2508.20929 → `\AtlasTitle \AtlasAbstract` shown raw, no title/abstract).
//!
//! The ATLAS author list is `\input` in the document BODY (a `flushleft`
//! `{\Large The ATLAS Collaboration}` + `\AtlasOrcid[orcid]{Name}` list), not in
//! the pre-`\maketitle` frontmatter, so those names render as the collaboration
//! block in the body rather than as title-page creators; bind `\AtlasOrcid` so
//! the names show cleanly instead of leaking the macro.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");

  // atlasdoc.cls L296: `\newcommand{\AtlasTitle}[1]{\title{#1}…}`.
  DefMacro!("\\AtlasTitle{}", "\\title{#1}");
  // atlasdoc.cls L360: `\AtlasAbstract{#1}` stashes the abstract; surface it as
  // the document abstract.
  DefMacro!("\\AtlasAbstract{}", "\\lx@add@abstract{#1}");
  // atlasdoc.cls L515: `\NewDocumentCommand \AtlasOrcid { o m }` — optional ORCID
  // + author name. Render the name (the trailing `$^{…}$` affiliation marker in
  // the source follows naturally); drop the ORCID decoration.
  DefMacro!("\\AtlasOrcid[]{}", "#2");
});

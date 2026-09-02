//! scrartcl.cls — the KOMA-Script article class, raw-interpreted through the
//! engine (memoir precedent, `memoir_cls.rs`).
//!
//! The former stub (`LoadClass!("OmniBus")` + ~30 no-ops, git history
//! 3c9baade57^) hid the real class: `\DeclareSectionCommand`-defined
//! headings, `\RedeclareSectionCommand`, `\usekomafont` (which must EXPAND
//! to the font switches the author put into `\setkomafont`/`\newkomafont`),
//! `\KOMAoptions`, `\deftocheading`, tocbasic's `\DeclareTOCStyleEntry`
//! family, `\Ifstr`/`\Ifthispageodd`, `\labeling`, `\dictum`, `\addmargin`,
//! `\captionabove`, `\captionof`, `\Ifpdfoutput` … — every one an
//! `undefined:` error or a silently swallowed argument under the stub, and
//! `\newkomafont{x}{…}` followed by a real `\usekomafont{x}` (loaded later
//! by a raw scrkbase) died with "font element x not defined" because the
//! stub's `\newkomafont` never registered anything (witness
//! contract-example-de/en). The real class raw-loads cleanly under the
//! engine and yields the correct `<section>`/`<subsection>` structure. Perl
//! LaTeXML ships no KOMA bindings; it raw-loads the class the same way.
//! Keeping this binding (rather than deleting the file) makes scrartcl
//! raw-load under BOTH `[rawclasses]` and the default arXiv configuration.
//!
//! Witnesses (perfect-kernel corpus): tikzlings-doc, tikzpingus-doc,
//! shadethm-doc, fnpara-doc, glossaries-user, LaTeX_RefSheet, easybook,
//! tutodoc, neoschool, contract-example-de/en, DEMO-TUDa*, tudaexercise
//! (`\DeclareNewSectionCommand{task}`), bohr/bohr_en (`\recalctypearea`),
//! arXiv 1802.07175 (`\ifpdf` via scrbase → iftex), 2305.01582
//! (`\titlehead`), 1702.04336 (scrartcl + tocloft `\sectfont`).
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("scrartcl", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  crate::koma_script::koma_post_load()?;
});

//! Stub for CEUR-WS ceurart.cls.
//!
//! ceurart.cls is built on top of scrartcl + expl3/xparse. The class
//! defines `\sep` via `\tex_def:D \sep{\unskip,}` inside an expl3 block,
//! which our raw-load can't reliably execute. Most user-facing
//! frontmatter macros (`\ead`, `\fnmark`, etc.) use `\NewDocumentCommand`
//! with expl3 bodies that don't fully unfurl either.
//!
//! Provide content-preserving stubs that route the author-supplied text
//! into either frontmatter notes (\tnotetext, \fntext, \cortext) or
//! contact entries (\ead, \orcidauthor), so no body material is dropped.
//!
//! Witness: 2501.13802, 2501.14238, 2501.16855, 2501.17381, 2502.01404,
//! 2502.02753, 2502.06743 — all `Error:undefined:\sep`.
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("hyperref");
  RequirePackage!("xcolor");
  RequirePackage!("graphicx");
  RequirePackage!("etoolbox");
  RequirePackage!("booktabs");
  RequirePackage!("makecell");
  RequirePackage!("multirow");
  RequirePackage!("array");
  RequirePackage!("xspace");
  RequirePackage!("calc");
  RequirePackage!("natbib");

  // The core separator — used in author/affiliation/keyword lists.
  DefMacro!("\\sep", ",");

  // Title notes / footnotes / corresp marks: route the user-supplied
  // text into ltx:note frontmatter so it's preserved in the XML output.
  // The optional [label] is ignored (LaTeXML auto-numbers notes).
  DefMacro!("\\tnotetext[]{}", "\\@add@frontmatter{ltx:note}[role=titlenote]{#2}");
  DefMacro!("\\fntext[]{}", "\\@add@frontmatter{ltx:note}[role=footnote]{#2}");
  DefMacro!("\\cortext[]{}", "\\@add@frontmatter{ltx:note}[role=corresp]{#2}");
  DefMacro!("\\nonumnote{}", "\\@add@frontmatter{ltx:note}{#1}");
  DefMacro!("\\nonumtnotetext{}", "\\@add@frontmatter{ltx:note}[role=titlenote]{#1}");

  // Mark macros: emit a footnote-like superscript with the optional
  // label preserved (defaults to empty if none provided).
  DefMacro!("\\tnotemark[]", "\\textsuperscript{#1}");
  DefMacro!("\\tnoteref[]{}", "\\textsuperscript{#2}");
  DefMacro!("\\fnmark[]", "\\textsuperscript{#1}");
  DefMacro!("\\fnref[]{}", "\\textsuperscript{#2}");
  DefMacro!("\\cormark[]", "\\textsuperscript{*#1}");
  DefMacro!("\\corref[]", "\\textsuperscript{*#1}");

  // Affiliation / address — route the user-visible text to a
  // ltx:note (allowed at document/frontmatter level; ltx:contact is
  // not). The semantic role is captured in the role attribute so a
  // downstream processor can still recognize an affiliation/address.
  DefMacro!("\\affiliation[]{}", "\\@add@frontmatter{ltx:note}[role=affiliation]{#2}");
  DefMacro!("\\address[]{}[]", "\\@add@frontmatter{ltx:note}[role=address]{#2}");

  // Email-address-of-author. Preserved as a ltx:note.
  DefMacro!("\\ead[]{}", "\\@add@frontmatter{ltx:note}[role=email]{#2}");
  def_macro_noop("\\eadsep")?;
  def_macro_noop("\\eadauthor")?;

  // ORCID/URL/email per-author; preserve user-visible value (#2) as
  // ltx:note. #1 is the author tag (used for cross-ref; ignored here).
  DefMacro!("\\orcidauthor{}{}", "\\@add@frontmatter{ltx:note}[role=orcid]{#2}");
  DefMacro!("\\urlauthor{}{}", "\\@add@frontmatter{ltx:note}[role=url]{#2}");
  DefMacro!("\\emailauthor{}{}", "\\@add@frontmatter{ltx:note}[role=email]{#2}");
  DefMacro!("\\creditauthor{}{}", "\\@add@frontmatter{ltx:note}[role=credit]{#2}");

  // "print*" commands typically emit a list of previously stashed
  // entries. Since our \ead/\orcidauthor/etc. already produce
  // frontmatter entries, these are now redundant — gobble cleanly.
  def_macro_noop("\\printcredits")?;
  def_macro_noop("\\printemails")?;
  def_macro_noop("\\printurls")?;
  def_macro_noop("\\printorcid")?;
  def_macro_noop("\\printtnotes")?;

  // Copyright year metadata. Author-supplied year goes to ltx:note.
  DefMacro!("\\copyrightyear{}",
    "\\@add@frontmatter{ltx:note}[role=copyrightyear]{#1}");

  // Subtitle — emit as a creator / extra-title fragment.
  DefMacro!("\\subtitle{}", "\\@add@frontmatter{ltx:subtitle}{#1}");

  // CEUR-WS conference metadata. These DO carry author content (event
  // name, date, location, etc.) — preserve as ltx:note frontmatter.
  DefMacro!("\\conference{}",
    "\\@add@frontmatter{ltx:note}[role=venue]{#1}");
  DefMacro!("\\copyrightclause{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}");
  DefMacro!("\\ceurConference[]{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=venue]{#2 #3 #4 #5}");
  DefMacro!("\\ceurEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}");
  DefMacro!("\\ceurAuthors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}");
  DefMacro!("\\ceurTitle{}",
    "\\@add@frontmatter{ltx:note}[role=ceur-title]{#1}");
  DefMacro!("\\ceurVolumeNr{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  DefMacro!("\\ceurLabel{}", "\\label{#1}");
  DefMacro!("\\ceurRef{}", "\\ref{#1}");
  DefMacro!("\\ceurpubyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\ceurwsurl{}",
    "\\@add@frontmatter{ltx:note}[role=url]{#1}");
  DefMacro!("\\ceurvolnr{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
});

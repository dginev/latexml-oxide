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
  DefMacro!(
    "\\tnotetext[]{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote]{#2}"
  );
  DefMacro!(
    "\\fntext[]{}",
    "\\@add@frontmatter{ltx:note}[role=footnote]{#2}"
  );
  DefMacro!(
    "\\cortext[]{}",
    "\\@add@frontmatter{ltx:note}[role=corresp]{#2}"
  );
  DefMacro!("\\nonumnote{}", "\\@add@frontmatter{ltx:note}{#1}");
  DefMacro!(
    "\\nonumtnotetext{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote]{#1}"
  );

  // Mark macros: emit a footnote-like superscript with the optional
  // label preserved (defaults to empty if none provided).
  DefMacro!("\\tnotemark[]", "\\textsuperscript{#1}");
  DefMacro!("\\tnoteref[]{}", "\\textsuperscript{#2}");
  DefMacro!("\\fnmark[]", "\\textsuperscript{#1}");
  DefMacro!("\\fnref[]{}", "\\textsuperscript{#2}");
  DefMacro!("\\cormark[]", "\\textsuperscript{*#1}");
  DefMacro!("\\corref[]", "\\textsuperscript{*#1}");

  // Affiliation / address: ceurart.cls:1477 `\address[label]{text}[keys]` is
  // the affiliation of the authors whose `\author[labels]` name that label.
  // Routed as Perl routes aas affiliations (aas_support.sty.ltxml:118
  // `\lx@add@affiliation`), with the label matched against the creators'
  // annotations (the elsarticle mechanism, elsart_support_core.sty.ltxml:38-42),
  // so each lands as a `<contact role="affiliation">` inside its `<creator>`
  // instead of a document-level `<note>` (witness arXiv 2511.11770). Guard:
  // `binding_singletons_56::ceurart_contacts_attach_to_creators`.
  DefMacro!("\\affiliation[]{}", "\\lx@add@affiliation[label={#1}]{#2}");
  DefMacro!("\\address[]{}[]", "\\lx@add@affiliation[label={#1}]{#2}");

  // ceurart.cls:640-643 `\ead[opt]{addr}`: an email of the preceding author,
  // or its URL when an option is given (`\@uad`). Perl
  // aas_support.sty.ltxml:122 `\lx@add@email` (witness arXiv 2511.11770).
  DefMacro!(
    "\\ead[] Semiverbatim",
    "\\if\\relax\\detokenize{#1}\\relax\\lx@add@email{#2}\\else\\lx@add@url{#2}\\fi"
  );
  def_macro_noop("\\eadsep")?;
  def_macro_noop("\\eadauthor")?;

  // ceurart.cls L1247 `\RenewDocumentCommand\author{ O{} m O{} }` —
  // `\author[affil]{name}[orcid=…,email=…,url=…,…]`. The trailing keyval bracket
  // is the modern per-author metadata carrier. Our binding didn't define
  // `\author`, so it fell to OmniBus's `\author[]{}`, leaving the `[keyval]`
  // bracket unconsumed — it LEAKED as raw `orcid=…, email=…` text into the body.
  // Define the 3-arg form: route the name to a structured creator and parse the
  // keyval (ceurart.cls L1033-1061) so the content-bearing keys survive as
  // frontmatter notes (matching `\orcidauthor`/`\ead` above), while the
  // formatting/flag keys are gobbled — so `\setkeys` never hits an undefined
  // key. arXiv/html_feedback#6650, witness 2511.11770 (both authors'
  // `orcid=…,email=…` leaked).
  RequirePackage!("keyval");
  RawTeX!(
    r"\define@key{lx@ceur@au}{orcid}{\lx@add@orcid{#1}}%
\define@key{lx@ceur@au}{email}{\lx@add@email{#1}}%
\define@key{lx@ceur@au}{url}{\lx@add@url{#1}}%
\define@key{lx@ceur@au}{twitter}{\@add@frontmatter{ltx:note}[role=twitter]{#1}}%
\define@key{lx@ceur@au}{facebook}{\@add@frontmatter{ltx:note}[role=facebook]{#1}}%
\define@key{lx@ceur@au}{linkedin}{\@add@frontmatter{ltx:note}[role=linkedin]{#1}}%
\define@key{lx@ceur@au}{plus}{\@add@frontmatter{ltx:note}[role=gplus]{#1}}%
\define@key{lx@ceur@au}{gplus}{\@add@frontmatter{ltx:note}[role=gplus]{#1}}%
\define@key{lx@ceur@au}{prefix}{}\define@key{lx@ceur@au}{suffix}{}%
\define@key{lx@ceur@au}{degree}{}\define@key{lx@ceur@au}{role}{}%
\define@key{lx@ceur@au}{auid}{}\define@key{lx@ceur@au}{bioid}{}%
\define@key{lx@ceur@au}{alt}{}\define@key{lx@ceur@au}{style}{}%
\define@key{lx@ceur@au}{collab}{}\define@key{lx@ceur@au}{anon}{}%
\define@key{lx@ceur@au}{deceased}{}\define@key{lx@ceur@au}{type}{}"
  );
  DefMacro!(
    "\\author []{} []",
    "\\lx@add@creator[annotations={#1}]{#2}\\setkeys{lx@ceur@au}{#3}"
  );

  // ORCID/URL/email per-author. #1 is the author tag (cross-ref; ignored here);
  // #2 is the ORCID id. Route ORCID through `\lx@add@orcid` → `ltx:contact
  // [role=orcid]` → clickable orcid.org link + logo icon (vs a dagger note).
  // The contact attaches to the most-recent creator. html_feedback#6571.
  DefMacro!("\\orcidauthor{}{}", "\\lx@add@orcid{#2}");
  // ceurart.cls:665/689 `\urlauthor{url}{name}` / `\emailauthor{email}{name}`
  // (written to the .aux by `\ead`): the address is #1 (witness arXiv
  // 2511.11770).
  DefMacro!("\\urlauthor{}{}", "\\lx@add@url{#1}");
  DefMacro!("\\emailauthor{}{}", "\\lx@add@email{#1}");
  DefMacro!(
    "\\creditauthor{}{}",
    "\\@add@frontmatter{ltx:note}[role=credit]{#2}"
  );

  // "print*" commands typically emit a list of previously stashed
  // entries. Since our \ead/\orcidauthor/etc. already produce
  // frontmatter entries, these are now redundant — gobble cleanly.
  def_macro_noop("\\printcredits")?;
  def_macro_noop("\\printemails")?;
  def_macro_noop("\\printurls")?;
  def_macro_noop("\\printorcid")?;
  def_macro_noop("\\printtnotes")?;

  // Copyright year metadata. Author-supplied year goes to ltx:note.
  DefMacro!(
    "\\copyrightyear{}",
    "\\@add@frontmatter{ltx:note}[role=copyrightyear]{#1}"
  );

  // Subtitle — emit as a creator / extra-title fragment.
  DefMacro!("\\subtitle{}", "\\@add@frontmatter{ltx:subtitle}{#1}");

  // CEUR-WS conference metadata. These DO carry author content (event
  // name, date, location, etc.) — preserve as ltx:note frontmatter.
  DefMacro!(
    "\\conference{}",
    "\\@add@frontmatter{ltx:note}[role=venue]{#1}"
  );
  DefMacro!(
    "\\copyrightclause{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}"
  );
  DefMacro!(
    "\\ceurConference[]{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=venue]{#2 #3 #4 #5}"
  );
  DefMacro!(
    "\\ceurEditors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}"
  );
  DefMacro!(
    "\\ceurAuthors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}"
  );
  DefMacro!(
    "\\ceurTitle{}",
    "\\@add@frontmatter{ltx:note}[role=ceur-title]{#1}"
  );
  DefMacro!(
    "\\ceurVolumeNr{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}"
  );
  DefMacro!("\\ceurLabel{}", "\\label{#1}");
  DefMacro!("\\ceurRef{}", "\\ref{#1}");
  DefMacro!(
    "\\ceurpubyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}"
  );
  DefMacro!(
    "\\ceurwsurl{}",
    "\\@add@frontmatter{ltx:note}[role=url]{#1}"
  );
  DefMacro!(
    "\\ceurvolnr{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}"
  );
});

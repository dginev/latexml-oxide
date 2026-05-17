//! Stub for daj.cls (Discrete Analysis journal class).
//!
//! daj.cls (2016, v1.03) extends tocbase to typeset "Discrete Analysis"
//! articles. The raw cls defines `\dajAUTHORdetails`, `\dajEDITORdetails`,
//! `\dajdetails`, and `{dajauthors}` env via `\setkeys` machinery. The raw
//! cls loads but its body fails to define them at runtime (likely a
//! `\setkeys`/key-handler interaction). Provide content-preserving stubs
//! so the substantive author/editor information is captured. Witness
//! 2305.10828, 2305.11062.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("article");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("amsfonts");
  RequirePackage!("hyperref");

  // \dajdetails / \dajAUTHORdetails / \dajEDITORdetails — argument
  // is a comma-separated key=val list (publication details, ORCID,
  // affiliation, etc.). Preserve as frontmatter note.
  DefMacro!("\\dajdetails{}",
    "\\@add@frontmatter{ltx:note}[role=daj-details]{#1}");
  DefMacro!("\\dajAUTHORdetails{}",
    "\\@add@frontmatter{ltx:note}[role=daj-author-details]{#1}");
  DefMacro!("\\dajEDITORdetails{}",
    "\\@add@frontmatter{ltx:note}[role=daj-editor-details]{#1}");

  // {dajauthors} environment wraps repeated `{authorinfo}[id]{body}` blocks.
  // The raw cls nests `\newenvironment{authorinfo}` inside `{dajauthors}`,
  // which fails to register at runtime. Define both so the substantive
  // author prose is preserved as semantically-named notes.
  DefEnvironment!("{dajauthors}",
    "<ltx:note role='daj-authors'>#body</ltx:note>",
    mode => "internal_vertical");
  DefEnvironment!("{authorinfo}[]",
    "<ltx:note role='daj-authorinfo'>#body</ltx:note>",
    mode => "internal_vertical");

  // daj.cls L54-55: `\imageat` and `\imagedot` are graphic stand-ins
  // for `@` and `.` in email addresses (anti-bot scraping). The raw
  // cls defines them as `\tocat`/`\tocdot` (image macros). Render as
  // plain characters — preserves email readability in HTML.
  DefMacro!("\\imageat", "@");
  DefMacro!("\\imagedot", ".");

  // {frontmatter}[options] — wraps title/authors/abstract block. The
  // paper at 2305.10828 uses `\begin{frontmatter}[classification=text]`
  // and closes with `\end{frontmatter}` around all the meta. Treat as
  // pass-through (children retain their own \title/\author handlers).
  DefEnvironment!("{frontmatter}[]", "#body",
    mode => "internal_vertical");
});

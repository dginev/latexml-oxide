//! Shared patches around the raw load of the KOMA-Script classes (scrartcl /
//! scrbook / scrreprt). The classes themselves are raw-interpreted (see
//! `scrartcl_cls.rs`); this module gives them the standard classes' semantic
//! layer (ids, `\appendix`, the kernel sectioning check: OXIDIZED_DESIGN_DIVERGENCES
//! #737) and re-targets the handful of commands whose real definitions are
//! print-only and would otherwise LOSE author content or bury a heading in a
//! presentational paragraph.
use latexml_core::binding::counter::dialect::{NewCounterOptions, define_counter_ids};
use latexml_package::prelude::*;

/// Which KOMA class is loaded: the standard class whose binding gives its semantic layer (ids, appendix level).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum KomaClass {
  /// scrartcl, as article.cls (article_cls.rs)
  Article,
  /// scrreprt and scrbook, as report.cls and book.cls (report_cls.rs, book_cls.rs)
  Chaptered,
}

/// Applied before the raw `.cls` is read. KOMA checks that the kernel's sectioning workers are still latex.ltx's
/// before it builds on them (scrartcl.cls:3292 + :3326, scrreprt/scrbook alike: its copy of the kernel
/// `\@startsection` against `\@startsection`; :3520 + :3552 its copy of `\@sect` against `\@sect`) and warns when
/// they are not ("`\@startsection' has been changed", "Unexpected definition of \@sect!"). Here the kernel's are
/// ours — the native `\@startsection` dispatcher (sect04.rs) and the `\@sect` router
/// (latex_constructs_rust_only.rs) — so under pdflatex neither test fails and neither Warning is written (minimal
/// `\documentclass{scrartcl}` log, TL 2025), while every KOMA paper here had both. KOMA's copies of the kernel
/// workers are therefore the kernel's own here: each is set to ours and kept against the class's definition of it,
/// and the class takes the branch it takes under pdflatex. Neither copy is called on that branch (the dispatcher
/// KOMA installs over `\@startsection` is refused by its lock, and `\scr@startsection` is ours, `koma_post_load`).
/// Witnesses: every scrartcl/scrreprt/scrbook paper (2509.11496, 2608.23214 via ceurart). Guard
/// `perfect_kernel_batch72a::koma_classes_take_the_kernel_branch_without_warnings`.
pub(crate) fn koma_pre_load() -> Result<()> {
  for (copy, kernel) in [
    ("\\scr@saved@startsection", "\\@startsection"),
    ("\\scr@latex@sect", "\\@sect"),
  ] {
    Let!(T_CS!(copy), T_CS!(kernel), Scope::Global);
    assign_value(&s!("{copy}:locked"), true, Some(Scope::Global));
  }
  Ok(())
}

/// Applied after the raw `.cls` finished loading, so every definition here
/// deliberately overrides the class's own.
pub(crate) fn koma_post_load(class: KomaClass) -> Result<()> {
  koma_counter_ids(class)?;
  koma_appendix(class)?;
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
    // Inside a maketitle deposit the setter fills KOMA's own field, as
    // scrartcl.cls's `\gdef\@<role>{#1}` does (uni-titlepage's `subject=`
    // key reads `\@subject`; sect05.rs `\lx@deposit@setters`). The field is
    // set globally and outlives the deposit, unlike `\@title`/`\@author`/`\@date`
    // that the kernel `\maketitle` empties: only a replayed class body reads it,
    // and the frontmatter never takes it from there.
    RawTeX!(&s!(
      r"\def\lx@deposit@{role}{{\lx@deposit@field\@{role}}}\g@addto@macro\lx@deposit@setters{{\let\{role}\lx@deposit@{role}}}"
    ));
  }
  RawTeX!(
    r"\def\lx@deposit@subtitle{\lx@deposit@field\@subtitle}\g@addto@macro\lx@deposit@setters{\let\subtitle\lx@deposit@subtitle}"
  );
  // `\minisec{title}` (scrartcl.cls L5081-5100): an unnumbered, un-TOC'd
  // freestanding heading. The real body is `\usekomafont{minisec}{#1\par}`
  // inside a `\parbox`-free group — a bold sans paragraph in the XML, no
  // heading semantics. The starred paragraph heading is the closest
  // structural equivalent and keeps the title as
  // `<ltx:paragraph><ltx:title>` (TL doc corpus: 17 bundles).
  DefMacro!("\\minisec{}", "\\paragraph*{#1}");
  // `\DeclareNewSectionCommand[style=section,level=N]{foo}` routes every KOMA
  // `style=section` heading — standard AND author-declared — through
  // `\scr@startsection` (scrartcl.cls:3316), a 6-arg positional drop-in for
  // `\@startsection` `{name}{level}{indent}{beforeskip}{afterskip}{font}`. Our
  // sectioning recognition lives only in the native (locked) `\@startsection`
  // (sect04.rs:89-156), which standard `\section`/`\subsection` still reach; but
  // a newly-declared command (`\task`) takes KOMA's route and never reaches it,
  // so the heading degrades to a bold para. Aliasing the single choke point maps
  // it back onto the SECTION_ELEMENT machinery (reads `level` as #2 → the right
  // element). RUST-ONLY surpass (no Perl koma binding). Do NOT alias
  // `\scr@startpart` — different signature.
  Let!("\\scr@startsection", "\\@startsection");
  koma_bibliography_preamble()?;
  Ok(())
}

/// `\setbibpreamble{<text>}` (scrartcl.cls:6752-6754, scrreprt/scrbook alike) sets `\bibpreamble`, which KOMA's
/// `thebibliography` typesets after the heading when it is neither `\relax` (its default) nor empty (:6762-6763,
/// `\noindent\bibpreamble\par\nobreak`). The bibliography here is the kernel's (the class's `thebibliography` gives way
/// to it), which typesets `\lx@bibliography@preamble` there (sect11.rs): it is KOMA's test and its text. Before, the text
/// was dropped without a diagnostic. natbib, loaded after the class, points the preamble at its own `\bibpreamble`
/// (natbib_sty.rs), as its `thebibliography` replaces KOMA's. Neither Perl (no KOMA binding) nor the former binding
/// printed it. Guard `perfect_kernel_batch77v::koma_setbibpreamble_is_printed`.
fn koma_bibliography_preamble() -> Result<()> {
  RawTeX!(
    r"\def\lx@bibliography@preamble{\ifx\bibpreamble\relax\else\ifx\bibpreamble\@empty\else\noindent\bibpreamble\par\fi\fi}"
  );
  Ok(())
}

/// scrbook's `\frontmatter`, `\mainmatter` and `\backmatter` (scrbook.cls:5272-5283) switch `\if@mainmatter`, by which
/// KOMA's chapter numbers (scrbook.cls:5573-5576 `\if@mainmatter \@tempswatrue`): a front- or back-matter chapter is
/// unnumbered and leaves the chapter counter alone. LaTeXML's sectioning numbers by `no_number_sections` instead
/// (sect04.rs `\@startsection`), which book.cls's binding sets in its `\frontmatter`/`\mainmatter` (book_cls.rs; Perl
/// book.cls.ltxml:112-118). Here the two also set it, before KOMA's own code (the page clearing, `\@mainmatterfalse`,
/// `\pagenumbering`), which stays as written. `\backmatter` is left to KOMA: the flag unnumbers every level, where KOMA
/// unnumbers only the chapter and keeps a section's number without the chapter's (scrbook.cls:6030-6036
/// `\if@mainmatter\thechapter.\fi`), and a back-matter appendix's sections are referenced by those numbers
/// (2508.01496: "Appendix" with sections 1-24; set at `\backmatter`, every reference to them lost its number). Book's
/// `no_number_chapters` (book_cls.rs, 78a) unnumbers the chapter alone but does not reset the counters within it, as
/// an unnumbered KOMA chapter does (scrbook.cls:5590-5600 from v3.15): 2508.01496's appendix figures would be off by
/// one, so its appendix chapter stays "Appendix A" (PDF "Appendix"). Witness 2506.17485: "Abstract" unnumbered, then "1 Introduction", as its PDF (before: "Chapter 1
/// Abstract", every later number one too high). Guard
/// `perfect_kernel_batchint23::scrbook_front_matter_chapters_are_unnumbered`.
pub(crate) fn koma_book_matter() -> Result<()> {
  DefPrimitive!("\\lx@koma@unnumbered", {
    AssignValue!("no_number_sections" => true);
  });
  DefPrimitive!("\\lx@koma@numbered", {
    AssignValue!("no_number_sections" => false);
  });
  RawTeX!(
    r"\let\lx@koma@frontmatter\frontmatter\def\frontmatter{\lx@koma@unnumbered\lx@koma@frontmatter}%
\let\lx@koma@mainmatter\mainmatter\def\mainmatter{\lx@koma@numbered\lx@koma@mainmatter}"
  );
  Ok(())
}

/// The ids of the sectioning and float counters, as the standard class's binding gives them (article_cls.rs:86-107,
/// report_cls.rs:94-130, book_cls.rs:100-136; Perl article.cls.ltxml:52-63): the raw class declares the counters with
/// `\newcounter` (scrartcl.cls:5045-5127, scrreprt.cls:5530-6051, tocbasic's `\DeclareNewTOC` for figure and table),
/// which names their ids after the counter (`section1.subsection1`, `equation1`), where an article paper's are
/// `S1.SS1` and `S1.E1`. Perl, which has no KOMA binding, loads OmniBus (article) for these classes and gives the
/// article's ids. Only the ids are set: the registers and reset lists stay the class's. A counter the class did not
/// declare is left alone. Witnesses: 124 of the 126 ceurart/atlasdoc papers of 2506+2509 and 97 of the KOMA 100 had
/// counter-named ids (2509.11496, 2506.00694, 2509.01219). Guard
/// `perfect_kernel_batch72a::koma_ids_and_appendix_follow_the_standard_classes`.
fn koma_counter_ids(class: KomaClass) -> Result<()> {
  let opts = |idprefix, idwithin, nested| NewCounterOptions { idprefix, idwithin, nested };
  let table: Vec<(&str, &str, NewCounterOptions)> = match class {
    KomaClass::Article => vec![
      ("part", "document", opts("Pt", "", vec!["section"])),
      ("section", "document", opts("S", "", vec!["subsection"])),
      (
        "subsection",
        "section",
        opts("SS", "", vec!["subsubsection"]),
      ),
      (
        "subsubsection",
        "subsection",
        opts("SSS", "", vec!["paragraph"]),
      ),
      (
        "paragraph",
        "subsubsection",
        opts("P", "", vec!["subparagraph"]),
      ),
      (
        "subparagraph",
        "paragraph",
        opts("SP", "", vec!["equation", "figure", "table"]),
      ),
      ("equation", "document", opts("E", "section", vec![])),
      ("figure", "document", opts("F", "section", vec![])),
      ("table", "document", opts("T", "section", vec![])),
    ],
    KomaClass::Chaptered => vec![
      ("part", "document", opts("Pt", "", vec!["chapter"])),
      ("chapter", "document", opts("Ch", "", vec!["section"])),
      ("section", "chapter", opts("S", "", vec!["subsection"])),
      (
        "subsection",
        "section",
        opts("SS", "", vec!["subsubsection"]),
      ),
      (
        "subsubsection",
        "subsection",
        opts("SSS", "", vec!["paragraph"]),
      ),
      (
        "paragraph",
        "subsubsection",
        opts("P", "", vec!["subparagraph"]),
      ),
      (
        "subparagraph",
        "paragraph",
        opts("SP", "", vec!["equation", "figure", "table"]),
      ),
      ("equation", "chapter", opts("E", "", vec![])),
      ("figure", "chapter", opts("F", "", vec![])),
      ("table", "chapter", opts("T", "", vec![])),
    ],
  };
  for (ctr, within, options) in table {
    if lookup_definition(&T_CS!(s!("\\c@{ctr}")))?.is_some() {
      define_counter_ids(ctr, within, Some(&options))?;
    }
  }
  Ok(())
}

/// `\appendix` starts the appendices, as the standard class's binding does (article_cls.rs:148-152,
/// report_cls.rs:178-182; Perl article.cls.ltxml `\@appendix` → `startAppendices('section')`), and then runs the
/// class's own (scrartcl.cls:4842-4867, scrreprt.cls:5305-5335, scrbook.cls:5284-5314): KOMA's counter resets, its
/// `\thesection`/`\thechapter` lettering, `\@chapapp` = `\appendixname`, its one-time hook and the author's
/// `\appendixmore` stay as written, after `start_appendices` has put the section (scrartcl) or chapter
/// (scrreprt/scrbook) level aside for the appendix. Before, a KOMA appendix was more sections (`ltx:section`, its
/// ids colliding with the body's: `section1a`), in 26 of 27 ceurart/atlasdoc papers with one (2509.11496,
/// 2506.00694). A second `\appendix` runs KOMA's alone (`\@appendix` is `\relax` after the first, as in the
/// standard classes).
fn koma_appendix(class: KomaClass) -> Result<()> {
  let level = match class {
    KomaClass::Article => "section",
    KomaClass::Chaptered => "chapter",
  };
  DefPrimitive!("\\@appendix", {
    start_appendices(level);
  });
  RawTeX!(r"\let\lx@koma@appendix\appendix\def\appendix{\@appendix\lx@koma@appendix}");
  Ok(())
}

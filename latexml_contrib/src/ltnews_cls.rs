//! ltnews.cls (the LaTeX News): the class is loaded raw, and its copyright line — `\@indicia` ("… brought to you by
//! the LaTeX Project Team; Copyright \@year, license LPPL", ltnews.cls:461-466), which the class prints only in the
//! title page's foot (`\ps@titlepage`, :483-488, selected by `\maketitle`'s `\thispagestyle{titlepage}`, :472-481), a
//! page style LaTeXML never typesets (`\thispagestyle` is a no-op, Perl latex_constructs.pool.ltxml:998, Rust
//! latex_constructs/sect05.rs) — becomes a frontmatter `note` of role `copyright` at the class's own print site,
//! `\maketitle`. The rest of the foot and head stays dropped as page furniture; a copyright notice is a semantic note
//! (user ruling 2026-10-01). SHARED: Perl with raw classes drops it alike. Witnesses base/ltnews* (42 issues).
//! Guard `perfect_kernel_batch61::class_copyright_lines_are_frontmatter_notes`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("ltnews", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  RawTeX!(
    r"\let\lx@ltnews@maketitle\maketitle
\def\maketitle{\lx@add@frontmatter{ltx:note}[role=copyright]{\@indicia}\lx@ltnews@maketitle}"
  );
});

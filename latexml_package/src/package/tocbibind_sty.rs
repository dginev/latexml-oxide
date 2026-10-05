use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: tocbibind.sty.ltxml defines nothing. tocbibind.sty's state is read by packages built around it — tocloft's
  // `\tableofcontents` tests `\if@dotoctoc` then `\if@bibchapter` (tocloft.sty:105-114) — so the conditionals are
  // tocbibind.sty's (:38-69): each `\if@doto…` true unless its `not…` option clears it, `\if@bibchapter` true when the
  // class has chapters. Without `\if@bibchapter` a skipped branch paired its `\else`/`\fi` with the outer conditional
  // (a stray `\fi`: SciPost.cls loads tocloft raw and this binding; 1811.09408, 2105.01655, 2203.11601). Witnesses
  // 2408.01486 (SciPost probing `\if@dotoctoc`), 2003.02382 (`\if@dotoclof`/`\if@dotoclot` under `[nottoc]`).
  Digest!(r"\newcommand{\@bibquit}{}\newif\if@bibchapter
\@ifundefined{chapter}{\@bibchapterfalse}{\@bibchaptertrue}
\newif\if@dotocbib\@dotocbibtrue
\newif\if@dotocind\@dotocindtrue
\newif\if@dotoctoc\@dotoctoctrue
\newif\if@dotoclot\@dotoclottrue
\newif\if@dotoclof\@dotocloftrue
\newif\if@donumbib\@donumbibfalse
\newif\if@donumindex\@donumindexfalse")?;
  DeclareOption!("section", { Digest!(r"\@bibchapterfalse")?; });
  DeclareOption!("notbib", { Digest!(r"\@dotocbibfalse")?; });
  DeclareOption!("notindex", { Digest!(r"\@dotocindfalse")?; });
  DeclareOption!("nottoc", { Digest!(r"\@dotoctocfalse")?; });
  DeclareOption!("notlot", { Digest!(r"\@dotoclotfalse")?; });
  DeclareOption!("notlof", { Digest!(r"\@dotocloffalse")?; });
  DeclareOption!("numbib", { Digest!(r"\@donumbibtrue")?; });
  DeclareOption!("numindex", { Digest!(r"\@donumindextrue")?; });
  // `chapter` only warns when the class has no chapters (tocbibind.sty:75-78); `other` and `none` (:80-90).
  DeclareOption!("chapter", None);
  DeclareOption!("other", { Digest!(r"\@bibchapterfalse")?; });
  DeclareOption!("none", {
    Digest!(r"\@dotocbibfalse\@dotocindfalse\@dotoctocfalse\@dotoclotfalse\@dotocloffalse\@donumbibfalse\@donumindexfalse")?;
  });

  ProcessOptions!();

  DefMacro!("\\@tocextra", "section");
  def_macro_noop("\\tocotherhead{}")?;
});

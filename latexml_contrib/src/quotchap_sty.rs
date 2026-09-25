//! quotchap.sty — quotations that open a chapter. The package typesets each
//! `savequote` into a box that its own `\chapter` prints at the next chapter
//! head (quotchap.sty:97-132). Our `\chapter` is locked, as Perl's is
//! (latex_constructs.pool.ltxml:557), so that `\chapter` never runs and the box
//! was never printed: the manual lost both quotations and their authors
//! (recall 44 %, sweep 120; Perl loses them alike). The quotation becomes an
//! epigraph (epigraph_sty.rs's structure) where it is written, just before the
//! chapter it opens (DIVERGENCES #287).
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("quotchap", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // quotchap.sty:97 `\begin{savequote}[<width>]`: the width is the box's.
  DefEnvironment!(
    "{savequote}[]",
    "<ltx:quote class='ltx_epigraph ltx_quotchap'>#body</ltx:quote>"
  );
  // quotchap.sty:106 `\qauthor{<source>}`, set flush right under the quotation.
  DefConstructor!(
    "\\qauthor{}",
    "<ltx:block class='ltx_epigraph_source'>#1</ltx:block>"
  );
});

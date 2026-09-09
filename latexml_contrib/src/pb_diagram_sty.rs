use std::borrow::Cow;

use latexml_package::prelude::*;

use crate::discard_env::discard_env_body;

#[rustfmt::skip]
LoadDefinitions!({
  // The real package allocates the registers and macros a document uses
  // outside its pictures (pb-diagram.sty:45 `\newskip\dgARROWLENGTH`,
  // `\dgARROWPARTS`, `\dgMAXSQUARE`, …; pb-manual.tex:374 `\divide
  // \dgARROWLENGTH by2`). The stub used to refuse the raw load, so every such
  // use was "undefined" + "expected a Variable" while Perl (no binding on the
  // default path) raw-loads it cleanly. Batch 56bf: raw-load, then keep the
  // ar5iv override for the picture environment below.
  InputDefinitions!("pb-diagram", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // Perl ar5iv-bindings/pb-diagram.sty.ltxml L22-41: \begin{diagram} emits
  // <ltx:ERROR> and swallows the body via discard_env_body (the commutative
  // diagram is drawn with `\put`-arithmetic pictures; an SVG rendering is
  // future work).
  DefConstructor!(
    T_CS!("\\begin{diagram}"), None,
    "<ltx:ERROR>{diagram}</ltx:ERROR>",
    bounded => true,
    mode    => "text",
    locked  => true,
    before_digest => { discard_env_body("diagram", "pb-diagram.sty.ltxml")?; }
  );
  DefMacro!("\\enddiagram", "\\relax");
});

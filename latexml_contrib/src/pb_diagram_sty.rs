use latexml_package::prelude::*;

use crate::discard_env::discard_env_body;

#[rustfmt::skip]
LoadDefinitions!({
  Warn!(
    "missing_file",
    "pb-diagram.sty",
    "pb-diagram.sty is not implemented and will not be interpreted raw."
  );
  // Perl ar5iv-bindings/pb-diagram.sty.ltxml L22-41: \begin{diagram} emits
  // <ltx:ERROR> and swallows the body via discard_env_body.
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

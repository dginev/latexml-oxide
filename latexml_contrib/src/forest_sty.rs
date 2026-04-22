use latexml_package::prelude::*;

use crate::discard_env::discard_env_body;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("tikz");
  RequirePackage!("etoolbox");
  Warn!(
    "missing_file",
    "forest.sty",
    "forest.sty is not implemented and will not be interpreted raw."
  );
  // Perl ar5iv-bindings/forest.sty.ltxml L46-50: \begin{forest} emits
  // <ltx:ERROR> and discards the body via discard_env_body.
  DefConstructor!(
    T_CS!("\\begin{forest}"), None,
    "<ltx:ERROR>{forest}</ltx:ERROR>",
    bounded => true,
    mode    => "text",
    locked  => true,
    before_digest => { discard_env_body("forest", "forest.sty.ltxml")?; }
  );
  DefMacro!("\\endforest", "\\relax");
  DefMacro!("\\forestset{}", "\\relax");
  DefMacro!("\\forestoption{}", "\\relax");
  DefMacro!("\\foresteoption{}", "\\relax");
  DefMacro!("\\forestregister{}", "\\relax");
  DefMacro!("\\foresteregister{}", "\\relax");
});

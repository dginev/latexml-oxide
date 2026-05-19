use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DeclareOption!(None, {
    Digest!("\\PassOptionsToPackage{\\CurrentOption}{xy}")?;
  });
  ProcessOptions!();
  // Mirror Perl's xypic.tex.ltxml: load xy.sty WITHOUT binding any options,
  // then explicitly invoke `\xyoption{v2}` to set up v2 compatibility.
  //
  // The earlier `RequirePackage!("xy", options=["v2"])` recorded xy.sty as
  // "loaded with options [v2]". A subsequent `\usepackage[all,tips]{xy}`
  // then tripped `option clash` and silently skipped option processing —
  // so `\xygraph` never got defined. Witness 2311.05789 (uses both
  // `\input xypic` AND `\usepackage[all,tips]{xy}`).
  RequirePackage!("xy");
  Digest!("\\xyoption{v2}")?;
});

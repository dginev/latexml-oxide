//! `\usepackage{diagrams}`: diagrams.sty is diagrams.tex (www.PaulTaylor.EU/diagrams ships the one
//! file under both names; its options, `PostScript=dvips`, `centredisplay`, …, set the drawing),
//! so the package is the diagrams.tex binding (diagrams_tex.rs). The option `amstex` enables
//! AmS-TeX's commutative-diagram commands (manual "Options", diagrams.tex `\cds@amstex`): the amscd
//! binding's `{CD}`, as `\diagramstyle[amstex]` gives it (math0104206, math0403203).
use latexml_package::prelude::*;

LoadDefinitions!({
  DeclareOption!("amstex", {
    RequirePackage!("amscd");
  });
  DeclareOption!(None, {});
  ProcessOptions!();
  InputDefinitions!("diagrams", extension => Some(Cow::Borrowed("tex")));
});

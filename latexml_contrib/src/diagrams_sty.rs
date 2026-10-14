//! `\usepackage{diagrams}`: diagrams.sty is diagrams.tex (www.PaulTaylor.EU/diagrams ships the one
//! file under both names; its options, `PostScript=dvips`, `centredisplay`, …, set the drawing),
//! so the package is the diagrams.tex binding (diagrams_tex.rs).
use latexml_package::prelude::*;

LoadDefinitions!({
  DeclareOption!(None, {});
  ProcessOptions!();
  InputDefinitions!("diagrams", extension => Some(Cow::Borrowed("tex")));
});

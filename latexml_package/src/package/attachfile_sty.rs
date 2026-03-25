use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("keyval");
  RequirePackage!("ifpdf");
  RequirePackage!("calc");
  RequirePackage!("color");
  DefMacro!("\\noattachfile []",       "");
  DefMacro!("\\notextattachfile []{}", "#2");
  DefMacro!("\\attachfile []{}",       "#2");
  DefMacro!("\\textattachfile []{}{}", "#3");
});

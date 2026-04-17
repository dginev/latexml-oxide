use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("keyval");
  RequirePackage!("ifpdf");
  RequirePackage!("calc");
  RequirePackage!("color");
  // Perl L24-27: \attachfilesetup accumulates global keyval options
  DefMacro!("\\lx@attachfile@options", None);
  DefPrimitive!("\\attachfilesetup {}", sub[(opts)] {
    let cs = T_CS!("\\lx@attachfile@options");
    AddToMacro!(cs, opts);
  });
  DefMacro!("\\noattachfile []",       "");
  DefMacro!("\\notextattachfile []{}", "#2");
  DefMacro!("\\attachfile []{}",       "#2");
  DefMacro!("\\textattachfile []{}{}", "#3");
});

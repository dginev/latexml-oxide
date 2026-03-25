//! standalone.sty — compile standalone sub-documents
//! Perl: standalone.sty.ltxml
//! NOTE: standalone.cls is handled separately; this is the .sty package.
//! The Perl implementation uses DefPrimitiveI/DefPrimitive with sub callbacks,
//! which are hard to translate faithfully. We provide stub macros that approximate
//! the behavior.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\@standalone@end@input", "\\egroup\\endinput");
  // \@standalone@start@input sets inPreamble to 0
  // Approximated as a no-op macro since we can't set state from a DefMacro
  DefMacro!("\\@standalone@start@input", "");
  // \@standalone@documentclass is a complex primitive in Perl;
  // simplified stub that loads packages from the argument
  DefMacro!("\\@standalone@documentclass[]{}", "");
});

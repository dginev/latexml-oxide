//! TeX Marks
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Marks Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  //  //======================================================================
  // Marks
  //----------------------------------------------------------------------
  // \mark             c  specifies text which should be marked.
  // \topmark          c  is the value of \botmark on the previous page.
  // \botmark          c  is the mark text most recently encountered on a page.
  // \firstmark        c  is the mark text first encountered on a page.

  // Perl TeX_Marks.pool.ltxml L30-34
  DefPrimitive!("\\mark{}", None);
  DefMacro!(T_CS!("\\topmark"), None, Tokens!());
  DefMacro!(T_CS!("\\botmark"), None, Tokens!());
  DefMacro!(T_CS!("\\firstmark"), None, Tokens!());
});

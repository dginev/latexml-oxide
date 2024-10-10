//! TeX Inserts
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Inserts Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // Inserting material
  //----------------------------------------------------------------------
  // \insert           c  places material into an insertions class.
  // \insert<8bit><filler>{<vertical mode material>}
  DefPrimitive!("\\insert Number", None);

  //======================================================================
  // Splitting a box
  //----------------------------------------------------------------------
  // \vsplit c removes a specified amount of material from a box register .
  // \splitbotmark c is the mark text of the last mark in the most recent \vsplit operation .
  // \splitfirstmark c is the mark text of the first mark in the most recent \vsplit operation .
  DefPrimitive!("\\vsplit Number Match:to Dimension", sub[(number,_to,_dimension)] {
    // analog to \box for now.
    let box_key   = s!("box{}", number.value_of());
    if let Some(Stored::Digested(stuff)) = lookup_value(&box_key) {
      adjust_box_color(&stuff)?;
      if stuff.is_empty()? { Digested::from(List::default()) } else { stuff }
    } else {
      Digested::from(List::default())
    }
  });
  DefMacro!(T_CS!("\\splitfirstmark"), None, Tokens!());
  DefMacro!(T_CS!("\\splitbotmark"), None, Tokens!());

  //======================================================================
  // Insertion parameters
  //----------------------------------------------------------------------
  // \insertpenalties  iq is a quantity used by TeX in two different ways.
  // \splitmaxdepth    pd is the maximum depth of boxes created by \vsplit.
  // \splittopskip     pg is special glue placed inside the box created by \vsplit.
  // \holdinginserts   pi is positive if insertions should remain dormant when \output is called.
  DefRegister!("\\insertpenalties", Number!(0));
  DefRegister!("\\splitmaxdepth", Dimension!("16383.99999pt"));
  DefRegister!("\\splittopskip", Glue!("10pt"));
  DefRegister!("\\holdinginserts", Number!(0));
});

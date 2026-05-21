//! TeX Penalties
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Penalties Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // Adding/removing penalties
  //----------------------------------------------------------------------
  // \penalty          c  adds a penalty to the current list.
  // \unpenalty        c  removes a penalty from the current list.
  // \lastpenalty      iq is 0 or the last penalty on the current list.
  def_primitive_noop("\\penalty Number")?;
  def_primitive_noop("\\unpenalty")?;
  DefRegister!("\\lastpenalty", Number::new(0), readonly => true);

  //======================================================================
  // values for various penalties
  //----------------------------------------------------------------------
  // \brokenpenalty    pi is the penalty added after a line ending with an hyphenated word.
  // \clubpenalty      pi is the penalty added after the first line in a paragraph.
  // \exhyphenpenalty  pi is the penalty for a line break after an explicit hyphen.
  // \floatingpenalty  pi is the penalty for insertions that are split between pages.
  // \hyphenpenalty    pi is the penalty for a line break after a discretionary hyphen.
  // \interlinepenalty pi is the penalty added between lines in a paragraph.
  // \linepenalty      pi is an amount added to the \badness calculated for every line in a
  // paragraph. \outputpenalty    pi holds the penalty from the current page break.
  // \widowpenalty     pi is the penalty added after the penultimate line in a paragraph.
  DefRegister!("\\brokenpenalty", Number!(100));
  DefRegister!("\\clubpenalty", Number!(150));
  DefRegister!("\\exhyphenpenalty", Number!(50));
  DefRegister!("\\floatingpenalty", Number!(0));
  DefRegister!("\\hyphenpenalty", Number!(50));
  DefRegister!("\\interlinepenalty", Number!(0));
  DefRegister!("\\linepenalty", Number!(10));
  DefRegister!("\\outputpenalty", Number!(0));
  DefRegister!("\\widowpenalty", Number!(150));
});

//! TeX Page
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Page Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // Parameters for page layout
  //----------------------------------------------------------------------
  // \hoffset          pd is a value added to the default 1-inch left margin.
  // \voffset          pd is a value added to the default 1-inch top margin.
  // \topskip          pg is special glue added before the first box on each page.
  // \pagedepth        iq is the actual depth of the last box on the main page.
  // \pagetotal        iq is the accumulated height of the current page.
  // \maxdepth         pd is the maximum depth of boxes on the main page.
  // \vsize            pd is the desired height of the current page.
  // \pagegoal         iq is the desired height of the current page.
  // \pageshrink       iq is the amount of finite shrinkability in the current page.
  // \pagestretch      iq is the amount of finite stretchability in the current page.
  // \pagefilllstretch iq is the amount of third-order infinite stretchability in the current page.
  // \pagefillstretch  iq is the amount of second-order infinite stretchability in the current page.
  // \pagefilstretch   iq is the amount of first-order infinite stretchability in the current page.

  DefRegister!("\\hoffset", Dimension!("0"));
  DefRegister!("\\voffset", Dimension!("0"));
  DefRegister!("\\topskip", Glue!("10pt"));
  DefRegister!("\\pagedepth", Dimension::new(0));
  DefRegister!("\\pagetotal", Dimension::new(0));
  DefRegister!("\\maxdepth", Dimension!("4pt"));
  DefRegister!("\\vsize", Dimension!("8.9in"));

  DefRegister!("\\pagegoal", Dimension::new(0));
  DefRegister!("\\pagestretch", Dimension::new(0));
  DefRegister!("\\pagefilstretch", Dimension::new(0));
  DefRegister!("\\pagefillstretch", Dimension::new(0));
  DefRegister!("\\pagefilllstretch", Dimension::new(0));
  DefRegister!("\\pageshrink", Dimension::new(0));

  //======================================================================
  // Usable for things line \clearpage, etc.
  DefConstructor!("\\lx@newpage", "^<ltx:pagination role='newpage'/>");
});

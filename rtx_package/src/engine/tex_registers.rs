use crate::prelude::*;

//======================================================================
// Registers & Parameters
// See Chapter 24, Summary of Vertical Mode
// Define a whole mess of useless registers here ...
// Values are from Appendix B, pp. 348-349 (for whatever its worth)
//======================================================================
#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // Integer registers; TeXBook p. 272-273

  DefRegister!("\\pretolerance", Number!(100));
  DefRegister!("\\tolerance", Number!(200));
  DefRegister!("\\linepenalty", Number!(10));
  DefRegister!("\\hyphenpenalty", Number!(50));
  DefRegister!("\\exhyphenpenalty", Number!(50));
  DefRegister!("\\binoppenalty", Number!(700));
  DefRegister!("\\relpenalty", Number!(500));
  DefRegister!("\\clubpenalty", Number!(150));
  DefRegister!("\\widowpenalty", Number!(150));
  DefRegister!("\\displaywidowpenalty", Number!(50));
  DefRegister!("\\brokenpenalty", Number!(100));
  DefRegister!("\\predisplaypenalty", Number!(10000));
  DefRegister!("\\postdisplaypenalty", Number!(0));
  DefRegister!("\\interlinepenalty", Number!(0));
  DefRegister!("\\floatingpenalty", Number!(0));
  DefRegister!("\\outputpenalty", Number!(0));
  DefRegister!("\\doublehyphendemerits", Number!(10000));
  DefRegister!("\\finalhyphendemerits", Number!(5000));
  DefRegister!("\\adjdemerits", Number!(10000));
  DefRegister!("\\looseness", Number!(0));
  
  
  DefRegister!("\\defaultskewchar", Number!(-1));
  DefRegister!("\\hangafter", Number!(0));
  DefRegister!("\\magnification", Number!(1000));
  DefRegister!("\\delimiterfactor", Number!(0));
  
  // Most of these are ignored, but...
  DefMacro!(
    "\\tracingall",
    "\\tracingonline=1 \\tracingcommands=2 \\tracingstats=2\\tracingpages=1 \\tracingoutput=1\\tracinglostchars=1\\tracingmacros=2 %\
\\tracingparagraphs=1 \\tracingrestores=1\\showboxbreadth=\\maxdimen \\showboxdepth=\\maxdimen \\errorstopmode"
  );
  DefMacro!("\\tracingnone", None);
  DefMacro!("\\hideoutput", None);



  // Read-only Integer registers
  DefRegister!("\\lastpenalty",Number!(0), readonly => true);


  // Special integer registers (?)
  // <special integer> = \spacefactor | \prevgraf | \deadcycles | \insertpenalties
  DefRegister!("\\spacefactor", Number!(0));
  DefRegister!("\\prevgraf", Number!(0));

  // ======================================================================
  // Dimen registers; TeXBook p. 274
  DefRegister!("\\emergencystretch", Dimension!("0"));
  DefRegister!("\\hsize", Dimension!("6.5in"));
  DefRegister!("\\vsize", Dimension!("8.9in"));
  DefRegister!("\\maxdepth", Dimension!("4pt"));

  DefRegister!("\\lineskiplimit", Dimension!("0"));
  DefRegister!("\\delimitershortfall", Dimension!("5pt"));
  DefRegister!("\\nulldelimiterspace", Dimension!("1.2pt"));
  DefRegister!("\\scriptspace", Dimension!("0.5pt"));
  DefRegister!("\\mathsurround", Dimension!("0"));
  DefRegister!("\\predisplaysize", Dimension!("0"));
  DefRegister!("\\displaywidth", Dimension!("0"));
  DefRegister!("\\displayindent", Dimension!("0"));
  DefRegister!("\\parindent", Dimension!("20pt"));
  DefRegister!("\\hangindent", Dimension!("0"));
  DefRegister!("\\hoffset", Dimension!("0"));
  DefRegister!("\\voffset", Dimension!("0"));

  // Special dimension registers (?)
  // <special dimen> = \prevdepth | \pagegoal | \pagetotal | \pagestretch | \pagefilstretch
  //    | \pagefillstretch | \pagefilllstretch | pageshrink | \pagedepth
  DefRegister!("\\pagegoal", Dimension::new(0));
  DefRegister!("\\pagetotal", Dimension::new(0));
  DefRegister!("\\pagestretch", Dimension::new(0));
  DefRegister!("\\pagefilstretch", Dimension::new(0));
  DefRegister!("\\pagefillstretch", Dimension::new(0));
  DefRegister!("\\pagefilllstretch", Dimension::new(0));
  DefRegister!("\\pageshrink", Dimension::new(0));
  DefRegister!("\\pagedepth", Dimension::new(0));

  // ======================================================================
  //  Glue registers; TeXBook p.274
  DefRegister!("\\baselineskip", Glue!("12pt"));
  DefRegister!("\\lineskip", Glue!("1pt"));
  DefRegister!("\\parskip", Glue!("0pt plus 1pt"));
  DefRegister!("\\abovedisplayskip", Glue!("12pt plus 3pt minus 9pt"));
  DefRegister!("\\abovedisplayshortskip", Glue!("0pt plus 3pt"));
  DefRegister!("\\belowdisplayskip", Glue!("12pt plus 3pt minus 9pt"));
  DefRegister!("\\belowdisplayshortskip", Glue!("0pt plus 3pt"));
  DefRegister!("\\leftskip", Glue!("0"));
  DefRegister!("\\rightskip", Glue!("0"));
  DefRegister!("\\topskip", Glue!("10pt"));
  DefRegister!("\\tabskip", Glue!("0"));
  DefRegister!("\\spaceskip", Glue!("0"));
  DefRegister!("\\xspaceskip", Glue!("0"));
  DefRegister!("\\parfillskip", Glue!("0pt plus 1fil"));

  //======================================================================
  // MuGlue registers; TeXBook p.274
  DefRegister!("\\thinmuskip", Glue!("3mu"));
  DefRegister!("\\medmuskip", Glue!("4mu plus 2mu minus 4mu"));
  DefRegister!("\\thickmuskip", Glue!("5mu plus 5mu"));
  //======================================================================
  // Token registers; TeXBook p.275
  DefRegister!("\\everypar", Tokens!());  
  DefRegister!("\\everycr", Tokens!());
});

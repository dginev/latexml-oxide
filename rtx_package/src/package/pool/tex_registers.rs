use crate::package::*;
use chrono::prelude::*;

//======================================================================
// Registers & Parameters
// See Chapter 24, Summary of Vertical Mode
// Define a whole mess of useless registers here ...
// Values are from Appendix B, pp. 348-349 (for whatever its worth)
//======================================================================
LoadDefinitions!(state, {
  //======================================================================
  // Integer registers; TeXBook p. 272-273
  DefRegister!("\\tracingmacros", Number!(0.0),
    getter => { LookupNumber!("TRACINGMACROS") },
    setter => sub[value, args, state] { AssignValue!("TRACINGMACROS" => value.value_of()); });
  DefRegister!("\\tracingcommands", Number!(0.0),
    getter => { LookupNumber!("TRACINGCOMMANDS") },
    setter => sub[value, args, state] { AssignValue!("TRACINGCOMMANDS" => value.value_of()); });

  for (key, val) in [
    ("pretolerance", 100),
    ("tolerance", 200),
    ("hbadness", 1000),
    ("vbadness", 1000),
    ("linepenalty", 10),
    ("hyphenpenalty", 50),
    ("exhyphenpenalty", 50),
    ("binoppenalty", 700),
    ("relpenalty", 500),
    ("clubpenalty", 150),
    ("widowpenalty", 150),
    ("displaywidowpenalty", 50),
    ("brokenpenalty", 100),
    ("predisplaypenalty", 10000),
    ("postdisplaypenalty", 0),
    ("interlinepenalty", 0),
    ("floatingpenalty", 0),
    ("outputpenalty", 0),
    ("doublehyphendemerits", 10000),
    ("finalhyphendemerits", 5000),
    ("adjdemerits", 10000),
    ("looseness", 0),
    ("pausing", 0),
    ("holdinginserts", 0),
    ("tracingonline", 0),
    ("tracingstats", 0),
    ("tracingparagraphs", 0),
    ("tracingpages", 0),
    ("tracingoutput", 0),
    ("tracinglostchars", 1),
    ("tracingrestores", 0),
    ("language", 0),
    ("uchyph", 1),
    ("lefthyphenmin", 0),
    ("righthyphenmin", 0),
    ("globaldefs", 0),
    ("defaulthyphenchar", 45),
    ("defaultskewchar", -1),
    ("escapechar", 92),
    ("endlinechar", 13),
    ("newlinechar", -1),
    ("maxdeadcycles", 0),
    ("hangafter", 0),
    ("fam", -1),
    ("mag", 1000),
    ("magnification", 1000),
    ("delimiterfactor", 0),
    ("time", 0),
    ("day", 0),
    ("month", 0),
    ("year", 0),
    ("showboxbreadth", 5),
    ("showboxdepth", 3),
    ("errorcontextlines", 5),
  ]
  .iter()
  {
    DefRegister!(&s!("\\{}", key), Number!(*val));
  }

  // Most of these are ignored, but...
  DefMacro!(
    "\\tracingall",
    "\\tracingonline=1 \\tracingcommands=2 \\tracingstats=2\
     \\tracingpages=1 \\tracingoutput=1 \\tracinglostchars=1\
     \\tracingmacros=2 \\tracingparagraphs=1 \\tracingrestores=1\
     \\showboxbreadth=\\maxdimen \\showboxdepth=\\maxdimen \\errorstopmode"
  );

  let dt = Local::now();

  AssignValue!("\\day", Number!(dt.day()), Scope::Global);
  AssignValue!("\\month", Number!(dt.month()), Scope::Global);
  AssignValue!("\\year", Number!(dt.year()), Scope::Global);
  AssignValue!("\\time", Number!(60 * dt.hour() + dt.minute()), Scope::Global);

  // Read-only Integer registers
  for name in &["lastpenalty", "inputlineno", "badness"] {
    DefRegister!(&s!("\\{}",name), Number!(0.0), readonly => true);
  }

  // Special integer registers (?)
  // <special integer> = \spacefactor | \prevgraf | \deadcycles | \insertpenalties
  for name in &["spacefactor", "prevgraf", "deadcycles", "insertpenalties"] {
    DefRegister!(&s!("\\{}", name), Number!(0.0));
  }

  // ======================================================================
  // Dimen registers; TeXBook p. 274
  let dparms = [
    ("hfuzz", "0.1pt"),
    ("vfuzz", "0.1pt"),
    ("overfullrule", "5pt"),
    ("emergencystretch", "0"),
    ("hsize", "6.5in"),
    ("vsize", "8.9in"),
    ("maxdepth", "4pt"),
    ("splitmaxdepth", "16383.99999pt"),
    ("boxmaxdepth", "16383.99999pt"),
    ("lineskiplimit", "0"),
    ("delimitershortfall", "5pt"),
    ("nulldelimiterspace", "1.2pt"),
    ("scriptspace", "0.5pt"),
    ("mathsurround", "0"),
    ("predisplaysize", "0"),
    ("displaywidth", "0"),
    ("displayindent", "0"),
    ("parindent", "20pt"),
    ("hangindent", "0"),
    ("hoffset", "0"),
    ("voffset", "0"),
  ];
  for (name, value) in &dparms {
    DefRegister!(&s!("\\{}", name), Dimension!(value));
  }

  // Special dimension registers (?)
  // <special dimen> = \prevdepth | \pagegoal | \pagetotal | \pagestretch | \pagefilstretch
  //    | \pagefillstretch | \pagefilllstretch | pageshrink | \pagedepth
  for name in &[
    "prevdepth",
    "pagegoal",
    "pagetotal",
    "pagestretch",
    "pagefilstretch",
    "pagefillstretch",
    "pagefilllstretch",
    "pageshrink",
    "pagedepth",
  ] {
    DefRegister!(&s!("\\{}", name), Dimension::new(0.0));
  }

  // ======================================================================
  //  Glue registers; TeXBook p.274
  let gparms = &[
    ("baselineskip", "12pt"),
    ("lineskip", "1pt"),
    ("parskip", "0pt plus 1pt"),
    ("abovedisplayskip", "12pt plus 3pt minus 9pt"),
    ("abovedisplayshortskip", "0pt plus 3pt"),
    ("belowdisplayskip", "12pt plus 3pt minus 9pt"),
    ("belowdisplayshortskip", "0pt plus 3pt"),
    ("leftskip", "0"),
    ("rightskip", "0"),
    ("topskip", "10pt"),
    ("splittopskip", "10pt"),
    ("tabskip", "0"),
    ("spaceskip", "0"),
    ("xspaceskip", "0"),
    ("parfillskip", "0pt plus 1fil"),
  ];
  for (name, value) in gparms {
    DefRegister!(&s!("\\{}", name), Glue!(value));
  }

  //======================================================================
  // MuGlue registers; TeXBook p.274

  let mparms = &[
    ("thinmuskip", "3mu"),
    ("medmuskip", "4mu plus 2mu minus 4mu"),
    ("thickmuskip", "5mu plus 5mu"),
  ];
  for (name, value) in mparms {
    DefRegister!(&s!("\\{}", name), Glue!(value));
  }

  //======================================================================
  // Token registers; TeXBook p.275

  let tparms = &[
    "output",
    "everypar",
    "everymath",
    "everydisplay",
    "everyhbox",
    "everyvbox",
    "everyjob",
    "everycr",
    "everyhelp",
  ];
  for name in tparms {
    DefRegister!(&s!("\\{}", name), Tokens!());
  }
});

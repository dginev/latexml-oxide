use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // stmaryrd.sty L40: \def\stmry@if#1#2{\let#2=\@undefined\iftrue#1#2}
  // The pattern in TL is `\stmry@if X Y \fi` where #1=X, #2=Y. The
  // default (option `only` not given) opens an \iftrue that absorbs
  // content until the trailing \fi. We bind stmaryrd symbols
  // directly (not via raw load), but old-arrows.sty raw-loads
  // stmaryrd and triggers \stmry@if at definition time. Provide a
  // \iftrue-emitting stub so the \fi matches.
  // Witness 2406.00395 (old-arrows), 2406.02375.
  RawTeX!(r"\def\stmry@if#1#2{\iftrue}");

  // Relational operators
  DefMath!("\\Yup",    None, "\u{2144}",                  role => "RELOP");
  DefMath!("\\Ydown",  None, "\\lx@nounicode{\\Ydown}",   role => "RELOP");
  DefMath!("\\Yleft",  None, "\\lx@nounicode{\\Yleft}",   role => "RELOP");
  DefMath!("\\Yright", None, "\\lx@nounicode{\\Yright}",  role => "RELOP");

  DefMath!("\\baro", None, "\u{233D}", role => "RELOP",
    meaning => "apl-reversal");
  DefMath!("\\bbslash", None, "\u{244A}", role => "RELOP");
  DefMath!("\\binampersand", None, "\u{0026}", role => "RELOP",
    meaning => "additive-conjunction");
  DefMath!("\\bindnasrepma", None, "\u{214B}", role => "RELOP",
    meaning => "multiplicative-disjunction");

  // Binary operators
  DefMath!("\\boxast",    None, "\u{29C6}", role => "BINOP");
  DefMath!("\\boxbar",    None, "\u{25EB}", role => "RELOP");
  DefMath!("\\boxbox",    None, "\u{29C8}", role => "BINOP");
  DefMath!("\\boxbslash", None, "\u{29C5}", role => "BINOP");
  DefMath!("\\boxcircle", None, "\u{29C7}", role => "BINOP");
  DefMath!("\\boxdot",    None, "\u{22A1}", role => "MULOP");
  DefMath!("\\boxempty",  None, "\u{25A1}", role => "RELOP");
  DefMath!("\\boxslash",  None, "\u{29C4}", role => "BINOP");

  // Arrows (no unicode)
  DefMath!("\\curlyveedownarrow",   None, "\\lx@nounicode{\\curlyveedownarrow}",   role => "ARROW");
  DefMath!("\\curlyveeuparrow",     None, "\\lx@nounicode{\\curlyveeuparrow}",     role => "ARROW");
  DefMath!("\\curlywedgedownarrow", None, "\\lx@nounicode{\\curlywedgedownarrow}", role => "ARROW");
  DefMath!("\\curlywedgeuparrow",   None, "\\lx@nounicode{\\curlywedgeuparrow}",   role => "ARROW");
  DefMath!("\\fatbslash",           None, "\\lx@nounicode{\\fatbslash}",           role => "ARROW");
  DefMath!("\\fatsemi",             None, "\u{2A1F}",                              role => "RELOP");
  DefMath!("\\fatslash",            None, "\\lx@nounicode{\\fatslash}",            role => "ARROW");
  DefMath!("\\interleave",          None, "\u{2AF4}",                              role => "RELOP");
  DefMath!("\\leftslice",           None, "\u{2AA6}",                              role => "RELOP");
  DefMath!("\\merge",               None, "\u{2A07}",                              role => "RELOP");
  DefMath!("\\minuso",              None, "\u{29B5}",                              role => "RELOP");
  DefMath!("\\moo",                 None, "\\lx@nounicode{\\moo}");
  DefMath!("\\obar",                None, "\u{29B6}",         role => "RELOP");
  DefMath!("\\oblong",              None, "\u{2395}",         role => "RELOP");
  DefMath!("\\obslash",             None, "\u{29B8}",         role => "RELOP");
  DefMath!("\\ogreaterthan",        None, "\u{29C1}",         role => "RELOP");
  DefMath!("\\olessthan",           None, "\u{29C0}",         role => "RELOP");
  DefMath!("\\ovee",                None, "\u{2228}\u{20DD}", role => "RELOP");
  DefMath!("\\owedge",              None, "\u{2227}\u{20DD}", role => "RELOP");
  DefMath!("\\rightslice",          None, "\u{2AA7}",         role => "RELOP");
  DefMath!("\\sslash",              None, "\u{2AFD}",         role => "RELOP");
  DefMath!("\\talloblong",          None, "\u{2AFF}",         role => "RELOP");
  DefMath!("\\varbigcirc",          None, "\u{25EF}",         role => "MULOP");
  DefMath!("\\varcurlyvee",         None, "\u{22CE}",         role => "RELOP");
  DefMath!("\\varcurlywedge",       None, "\u{22CF}",         role => "RELOP");
  DefMath!("\\varoast",             None, "\u{229B}",         role => "MULOP");
  DefMath!("\\varobar",             None, "\u{29B6}",         role => "RELOP");
  DefMath!("\\varobslash",          None, "\u{29B8}",         role => "MULOP");
  DefMath!("\\varocircle",          None, "\u{229A}",         role => "MULOP");
  DefMath!("\\varodot",             None, "\u{2299}",         role => "MULOP");
  DefMath!("\\varogreaterthan",     None, "\u{29C1}",         role => "RELOP");
  DefMath!("\\varolessthan",        None, "\u{29C0}",         role => "RELOP");
  DefMath!("\\varominus",           None, "\u{2296}",         role => "ADDOP");
  DefMath!("\\varoplus", None, "\u{2295}", role => "ADDOP",
    meaning => "additive-disjunction");
  DefMath!("\\varoslash",  None, "\u{2298}",         role => "RELOP");
  DefMath!("\\varotimes", None, "\u{2297}", role => "MULOP",
    meaning => "multiplicative-conjunction");
  DefMath!("\\varovee",   None, "\u{2228}\u{20DD}", role => "RELOP");
  DefMath!("\\varowedge", None, "\u{2227}\u{20DD}", role => "RELOP");
  DefMath!("\\vartimes",  None, "\u{00D7}",         role => "MULOP");

  // Big operators (SUMOP with dynamic mathstyle + scriptpos)
  // Perl: scriptpos => \&doScriptpos, mathstyle => \&doVariablesizeOp
  DefMath!("\\bigbox", None, "\u{25A1}",
    font => { scale => 1.6 },
    role => "SUMOP", dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigcurlywedge", None, "\u{22CF}",
    font => { scale => 1.6 },
    role => "SUMOP", dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigcurlyvee", None, "\u{22CE}",
    font => { scale => 1.6 },
    role => "SUMOP", dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\biginterleave", None, "\u{2AFC}",
    role => "SUMOP", dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigparallel", None, "\u{2016}",
    role => "SUMOP", dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigsqcap", None, "\u{2A05}",
    role => "SUMOP", dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigtriangledown", None, "\u{25BD}",
    font => { scale => 1.6 },
    role => "SUMOP", dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\bigtriangleup", None, "\u{25B3}",
    font => { scale => 1.6 },
    role => "SUMOP", dynamic_scriptpos => true, dynamic_mathstyle => true);

  // More relational operators
  DefMath!("\\inplus",                None, "\u{2A2D}", role => "RELOP");
  DefMath!("\\niplus",                None, "\u{2A2E}", role => "RELOP");
  DefMath!("\\ntrianglelefteqslant",  None, "\u{22EC}", role => "RELOP");
  DefMath!("\\ntrianglerighteqslant", None, "\u{22ED}", role => "RELOP");

  // Kludged subset/superset operators
  DefMath!("\\subsetplus", None,
    "\\lx@kludged{\\subset{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-0.8em,yoffset=0.3ex}{+}}}",
    role => "RELOP", meaning => "subset-plus");
  DefMath!("\\subsetpluseq", None,
    "\\lx@kludged{\\subseteq{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-0.8em,yoffset=0.5ex}{+}}}",
    role => "RELOP", meaning => "subset-equals-plus");
  DefMath!("\\supsetplus", None,
    "\\lx@kludged{\\supset{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-1em,yoffset=0.3ex}{+}}}",
    role => "RELOP", meaning => "superset-plus");
  DefMath!("\\supsetpluseq", None,
    "\\lx@kludged{\\supseteq{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-1em,yoffset=0.5ex}{+}}}",
    role => "RELOP", meaning => "superset-equals-plus");
  DefMath!("\\nplus", None,
    "\\lx@kludged{\\cap{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-0.75em,yoffset=0.2ex}{+}}}",
    role => "ADDOP", meaning => "intersection-plus");
  DefMath!("\\bignplus", None,
    "\\lx@kludged{\\bigcap\\mathchoice{\\lx@tweaked{width=0pt,xoffset=-1.6em,yoffset=0.2ex}{+}}{\\lx@tweaked{width=0pt,xoffset=-1.3em,yoffset=0.2ex}{+}}{\\lx@tweaked{width=0pt,xoffset=-1.3em,yoffset=0.2ex}{+}}{\\lx@tweaked{width=0pt,xoffset=-1.3em,yoffset=0.2ex}{+}}}",
    role => "ADDOP", meaning => "intersection-plus",
    dynamic_scriptpos => true, dynamic_mathstyle => true);

  DefMath!("\\trianglelefteqslant",  None, "\u{22B4}",                  role => "RELOP");
  DefMath!("\\trianglerighteqslant", None, "\u{22B5}",                  role => "RELOP");
  DefMath!("\\arrownot",             None, "\\lx@tweaked{width=0pt}{/}", role => "RELOP");
  DefMath!("\\longarrownot",         None, "\\lx@tweaked{width=0pt}{/}", role => "RELOP");
  DefMath!("\\Arrownot",             None, "\\lx@tweaked{width=0pt}{/}", role => "RELOP");
  DefMath!("\\Longarrownot",         None, "\\lx@tweaked{width=0pt}{/}", role => "RELOP");
  DefMath!("\\mapsfromchar",         None, "|",                          role => "RELOP");
  DefMath!("\\Mapsfromchar",         None, "|",                          role => "RELOP");
  DefMath!("\\Mapstochar",           None, "|",                          role => "RELOP");
  DefMath!("\\Longmapsfrom",         None, "\u{27FD}",                   role => "ARROW");
  DefMath!("\\Longmapsto",           None, "\u{27FE}",                   role => "ARROW");
  DefMath!("\\Mapsfrom",             None, "\u{2906}",                   role => "ARROW");
  DefMath!("\\Mapsto",               None, "\u{2907}",                   role => "ARROW");
  DefMath!("\\leftarrowtriangle",    None, "\u{21FD}",                   role => "ARROW");
  DefMath!("\\leftrightarroweq",     None, "\\stackrel{\\leftrightarrow}{-}", role => "ARROW");
  DefMath!("\\leftrightarrowtriangle", None, "\u{21FF}",                 role => "ARROW");
  DefMath!("\\lightning",              None, "\u{21AF}",                  role => "ARROW");
  DefMath!("\\longmapsfrom",          None, "\u{27FB}",                  role => "ARROW");
  DefMath!("\\mapsfrom",              None, "\u{21A4}",                  role => "ARROW");
  DefMath!("\\nnearrow",              None, "\\lx@nounicode{\\nnearrow}", role => "ARROW");
  DefMath!("\\nnwarrow",              None, "\\lx@nounicode{\\nnwarrow}", role => "ARROW");
  DefMath!("\\rightarrowtriangle",    None, "\u{21FE}",                  role => "ARROW");
  DefMath!("\\rrparenthesis",         None, "\u{2988}",                  role => "ARROW");
  DefMath!("\\shortdownarrow",        None, "\u{2193}",                  role => "ARROW");
  DefMath!("\\shortleftarrow",        None, "\u{2190}",                  role => "ARROW");
  DefMath!("\\shortrightarrow",       None, "\u{2192}",                  role => "ARROW");
  DefMath!("\\shortuparrow",          None, "\u{2191}",                  role => "ARROW");
  DefMath!("\\ssearrow",              None, "\\lx@nounicode{\\ssearrow}", role => "ARROW");
  DefMath!("\\sswarrow",              None, "\\lx@nounicode{\\sswarrow}", role => "ARROW");

  // Delimiters
  DefMath!("\\Lbag", None, "\u{27C5}", role => "OPEN");
  DefMath!("\\Rbag", None, "\u{27C6}", role => "CLOSE");
  DefMath!("\\lbag", None, "\u{27C5}", role => "OPEN");
  DefMath!("\\llbracket", None, "\u{27E6}", role => "OPEN");
  DefMath!("\\llceil", None,
    "\\lx@kludged{\\lx@tweaked{width=0pt,xoffset=0.3em}{\\lceil}\\lceil}",
    role => "OPEN");
  DefMath!("\\rrceil", None,
    "\\lx@kludged{\\rceil\\lx@tweaked{width=0pt,xoffset=-0.3em}{\\rceil}}",
    role => "CLOSE");
  DefMath!("\\llfloor", None,
    "\\lx@kludged{\\lx@tweaked{width=0pt,xoffset=0.3em}{\\lfloor}\\lfloor}",
    role => "OPEN");
  DefMath!("\\rrfloor", None,
    "\\lx@kludged{\\rfloor\\lx@tweaked{width=0pt,xoffset=-0.31em}{\\rfloor}}",
    role => "CLOSE");
  DefMath!("\\llparenthesis", None, "\u{2987}", role => "OPEN");
  DefMath!("\\rrparenthesis", None, "\u{2988}", role => "CLOSE");
  DefMath!("\\rbag",      None, "\u{27C6}", role => "CLOSE");
  DefMath!("\\rrbracket", None, "\u{27E7}", role => "CLOSE");

  // Text symbol
  // Perl: DefPrimitive('\varcopyright', UTF(0xA9));
  DefPrimitive!("\\varcopyright", "\u{00A9}");
});

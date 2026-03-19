use crate::prelude::*;

LoadDefinitions!({
  // Relational operators
  DefMath!("\\Yup", "\u{2144}", role => "RELOP");
  DefMath!("\\Ydown", "\\lx@nounicode{\\Ydown}", role => "RELOP");
  DefMath!("\\Yleft", "\\lx@nounicode{\\Yleft}", role => "RELOP");
  DefMath!("\\Yright", "\\lx@nounicode{\\Yright}", role => "RELOP");

  DefMath!("\\baro", "\u{233D}", role => "RELOP",
    meaning => "apl-reversal");
  DefMath!("\\bbslash", "\u{244A}", role => "RELOP");
  DefMath!("\\binampersand", "\u{0026}", role => "RELOP",
    meaning => "additive-conjunction");
  DefMath!("\\bindnasrepma", "\u{214B}", role => "RELOP",
    meaning => "multiplicative-disjunction");

  // Binary operators
  DefMath!("\\boxast", "\u{29C6}", role => "BINOP");
  DefMath!("\\boxbar", "\u{25EB}", role => "RELOP");
  DefMath!("\\boxbox", "\u{29C8}", role => "BINOP");
  DefMath!("\\boxbslash", "\u{29C5}", role => "BINOP");
  DefMath!("\\boxcircle", "\u{29C7}", role => "BINOP");
  DefMath!("\\boxdot", "\u{22A1}", role => "MULOP");
  DefMath!("\\boxempty", "\u{25A1}", role => "RELOP");
  DefMath!("\\boxslash", "\u{29C4}", role => "BINOP");

  // Arrows
  DefMath!("\\curlyveedownarrow", "\\lx@nounicode{\\curlyveedownarrow}", role => "ARROW");
  DefMath!("\\curlyveeuparrow", "\\lx@nounicode{\\curlyveeuparrow}", role => "ARROW");
  DefMath!("\\curlywedgedownarrow", "\\lx@nounicode{\\curlywedgedownarrow}", role => "ARROW");
  DefMath!("\\curlywedgeuparrow", "\\lx@nounicode{\\curlywedgeuparrow}", role => "ARROW");
  DefMath!("\\fatbslash", "\\lx@nounicode{\\fatbslash}", role => "ARROW");
  DefMath!("\\fatsemi", "\u{2A1F}", role => "RELOP");
  DefMath!("\\fatslash", "\\lx@nounicode{\\fatslash}", role => "ARROW");
  DefMath!("\\interleave", "\u{2AF4}", role => "RELOP");
  DefMath!("\\leftslice", "\u{2AA6}", role => "RELOP");
  DefMath!("\\merge", "\u{2A07}", role => "RELOP");
  DefMath!("\\minuso", "\u{29B5}", role => "RELOP");
  DefMath!("\\moo", "\\lx@nounicode{\\moo}");
  DefMath!("\\obar", "\u{29B6}", role => "RELOP");
  DefMath!("\\oblong", "\u{2395}", role => "RELOP");
  DefMath!("\\obslash", "\u{29B8}", role => "RELOP");
  DefMath!("\\ogreaterthan", "\u{29C1}", role => "RELOP");
  DefMath!("\\olessthan", "\u{29C0}", role => "RELOP");
  DefMath!("\\ovee", "\u{2228}\u{20DD}", role => "RELOP");
  DefMath!("\\owedge", "\u{2227}\u{20DD}", role => "RELOP");
  DefMath!("\\rightslice", "\u{2AA7}", role => "RELOP");
  DefMath!("\\sslash", "\u{2AFD}", role => "RELOP");
  DefMath!("\\talloblong", "\u{2AFF}", role => "RELOP");
  DefMath!("\\varbigcirc", "\u{25EF}", role => "MULOP");
  DefMath!("\\varcurlyvee", "\u{22CE}", role => "RELOP");
  DefMath!("\\varcurlywedge", "\u{22CF}", role => "RELOP");
  DefMath!("\\varoast", "\u{229B}", role => "MULOP");
  DefMath!("\\varobar", "\u{29B6}", role => "RELOP");
  DefMath!("\\varobslash", "\u{29B8}", role => "MULOP");
  DefMath!("\\varocircle", "\u{229A}", role => "MULOP");
  DefMath!("\\varodot", "\u{2299}", role => "MULOP");
  DefMath!("\\varogreaterthan", "\u{29C1}", role => "RELOP");
  DefMath!("\\varolessthan", "\u{29C0}", role => "RELOP");
  DefMath!("\\varominus", "\u{2296}", role => "ADDOP");
  DefMath!("\\varoplus", "\u{2295}", role => "ADDOP",
    meaning => "additive-disjunction");
  DefMath!("\\varoslash", "\u{2298}", role => "RELOP");
  DefMath!("\\varotimes", "\u{2297}", role => "MULOP",
    meaning => "multiplicative-conjunction");
  DefMath!("\\varovee", "\u{2228}\u{20DD}", role => "RELOP");
  DefMath!("\\varowedge", "\u{2227}\u{20DD}", role => "RELOP");
  DefMath!("\\vartimes", "\u{00D7}", role => "MULOP");

  // Big operators (SUMOP with dynamic mathstyle/scriptpos)
  // TODO: font => { size => 'Big' } not yet supported
  DefMath!("\\bigbox", "\u{25A1}",
    role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigcurlywedge", "\u{22CF}",
    role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigcurlyvee", "\u{22CE}",
    role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\biginterleave", "\u{2AFC}",
    role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigparallel", "\u{2016}",
    role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigsqcap", "\u{2A05}",
    role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigtriangledown", "\u{25BD}",
    role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigtriangleup", "\u{25B3}",
    role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);

  // More relational operators
  DefMath!("\\inplus", "\u{2A2D}", role => "RELOP");
  DefMath!("\\niplus", "\u{2A2E}", role => "RELOP");
  DefMath!("\\ntrianglelefteqslant", "\u{22EC}", role => "RELOP");
  DefMath!("\\ntrianglerighteqslant", "\u{22ED}", role => "RELOP");

  // Kludged subset/superset operators
  DefMath!("\\subsetplus",
    "\\lx@kludged{\\subset{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-0.8em,yoffset=0.3ex}{+}}}",
    role => "RELOP", meaning => "subset-plus");
  DefMath!("\\subsetpluseq",
    "\\lx@kludged{\\subseteq{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-0.8em,yoffset=0.5ex}{+}}}",
    role => "RELOP", meaning => "subset-equals-plus");
  DefMath!("\\supsetplus",
    "\\lx@kludged{\\supset{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-1em,yoffset=0.3ex}{+}}}",
    role => "RELOP", meaning => "superset-plus");
  DefMath!("\\supsetpluseq",
    "\\lx@kludged{\\supseteq{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-1em,yoffset=0.5ex}{+}}}",
    role => "RELOP", meaning => "superset-equals-plus");
  DefMath!("\\nplus",
    "\\lx@kludged{\\cap{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-0.75em,yoffset=0.2ex}{+}}}",
    role => "ADDOP", meaning => "intersection-plus");
  DefMath!("\\bignplus",
    "\\lx@kludged{\\bigcap\\mathchoice{\\lx@tweaked{width=0pt,xoffset=-1.6em,yoffset=0.2ex}{+}}{\\lx@tweaked{width=0pt,xoffset=-1.3em,yoffset=0.2ex}{+}}{\\lx@tweaked{width=0pt,xoffset=-1.3em,yoffset=0.2ex}{+}}{\\lx@tweaked{width=0pt,xoffset=-1.3em,yoffset=0.2ex}{+}}}",
    role => "ADDOP", meaning => "intersection-plus",
    dynamic_mathstyle => true);

  DefMath!("\\trianglelefteqslant", "\u{22B4}", role => "RELOP");
  DefMath!("\\trianglerighteqslant", "\u{22B5}", role => "RELOP");
  DefMath!("\\arrownot", "\\lx@tweaked{width=0pt}{/}", role => "RELOP");
  DefMath!("\\longarrownot", "\\lx@tweaked{width=0pt}{/}", role => "RELOP");
  DefMath!("\\Arrownot", "\\lx@tweaked{width=0pt}{/}", role => "RELOP");
  DefMath!("\\Longarrownot", "\\lx@tweaked{width=0pt}{/}", role => "RELOP");
  DefMath!("\\mapsfromchar", "|", role => "RELOP");
  DefMath!("\\Mapsfromchar", "|", role => "RELOP");
  DefMath!("\\Mapstochar", "|", role => "RELOP");
  DefMath!("\\Longmapsfrom", "\u{27FD}", role => "ARROW");
  DefMath!("\\Longmapsto", "\u{27FE}", role => "ARROW");
  DefMath!("\\Mapsfrom", "\u{2906}", role => "ARROW");
  DefMath!("\\Mapsto", "\u{2907}", role => "ARROW");
  DefMath!("\\leftarrowtriangle", "\u{21FD}", role => "ARROW");
  DefMath!("\\leftrightarroweq", "\\stackrel{\\leftrightarrow}{-}",
    role => "ARROW");
  DefMath!("\\leftrightarrowtriangle", "\u{21FF}", role => "ARROW");
  DefMath!("\\lightning", "\u{21AF}", role => "ARROW");
  DefMath!("\\longmapsfrom", "\u{27FB}", role => "ARROW");
  DefMath!("\\mapsfrom", "\u{21A4}", role => "ARROW");
  DefMath!("\\nnearrow", "\\lx@nounicode{\\nnearrow}", role => "ARROW");
  DefMath!("\\nnwarrow", "\\lx@nounicode{\\nnwarrow}", role => "ARROW");
  DefMath!("\\rightarrowtriangle", "\u{21FE}", role => "ARROW");
  DefMath!("\\rrparenthesis", "\u{2988}", role => "ARROW");
  DefMath!("\\shortdownarrow", "\u{2193}", role => "ARROW");
  DefMath!("\\shortleftarrow", "\u{2190}", role => "ARROW");
  DefMath!("\\shortrightarrow", "\u{2192}", role => "ARROW");
  DefMath!("\\shortuparrow", "\u{2191}", role => "ARROW");
  DefMath!("\\ssearrow", "\\lx@nounicode{\\ssearrow}", role => "ARROW");
  DefMath!("\\sswarrow", "\\lx@nounicode{\\sswarrow}", role => "ARROW");

  // Delimiters
  DefMath!("\\Lbag", "\u{27C5}", role => "OPEN");
  DefMath!("\\Rbag", "\u{27C6}", role => "CLOSE");
  DefMath!("\\lbag", "\u{27C5}", role => "OPEN");
  DefMath!("\\llbracket", "\u{27E6}", role => "OPEN");
  DefMath!("\\llceil",
    "\\lx@kludged{\\lx@tweaked{width=0pt,xoffset=0.3em}{\\lceil}\\lceil}",
    role => "OPEN");
  DefMath!("\\rrceil",
    "\\lx@kludged{\\rceil\\lx@tweaked{width=0pt,xoffset=-0.3em}{\\rceil}}",
    role => "CLOSE");
  DefMath!("\\llfloor",
    "\\lx@kludged{\\lx@tweaked{width=0pt,xoffset=0.3em}{\\lfloor}\\lfloor}",
    role => "OPEN");
  DefMath!("\\rrfloor",
    "\\lx@kludged{\\rfloor\\lx@tweaked{width=0pt,xoffset=-0.31em}{\\rfloor}}",
    role => "CLOSE");
  DefMath!("\\llparenthesis", "\u{2987}", role => "OPEN");
  DefMath!("\\rrparenthesis", "\u{2988}", role => "CLOSE");
  DefMath!("\\rbag", "\u{27C6}", role => "CLOSE");
  DefMath!("\\rrbracket", "\u{27E7}", role => "CLOSE");

  // Text symbol
  DefPrimitive!("\\varcopyright", "\u{00A9}");
});

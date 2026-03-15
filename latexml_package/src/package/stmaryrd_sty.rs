use crate::prelude::*;

LoadDefinitions!({
  // Relational operators
  DefMath!("\\Yup", "\u{2144}", operator_role => "RELOP");
  DefMath!("\\Ydown", "\\lx@nounicode{\\Ydown}", operator_role => "RELOP");
  DefMath!("\\Yleft", "\\lx@nounicode{\\Yleft}", operator_role => "RELOP");
  DefMath!("\\Yright", "\\lx@nounicode{\\Yright}", operator_role => "RELOP");

  DefMath!("\\baro", "\u{233D}", operator_role => "RELOP",
    meaning => "apl-reversal");
  DefMath!("\\bbslash", "\u{244A}", operator_role => "RELOP");
  DefMath!("\\binampersand", "\u{0026}", operator_role => "RELOP",
    meaning => "additive-conjunction");
  DefMath!("\\bindnasrepma", "\u{214B}", operator_role => "RELOP",
    meaning => "multiplicative-disjunction");

  // Binary operators
  DefMath!("\\boxast", "\u{29C6}", operator_role => "BINOP");
  DefMath!("\\boxbar", "\u{25EB}", operator_role => "RELOP");
  DefMath!("\\boxbox", "\u{29C8}", operator_role => "BINOP");
  DefMath!("\\boxbslash", "\u{29C5}", operator_role => "BINOP");
  DefMath!("\\boxcircle", "\u{29C7}", operator_role => "BINOP");
  DefMath!("\\boxdot", "\u{22A1}", operator_role => "MULOP");
  DefMath!("\\boxempty", "\u{25A1}", operator_role => "RELOP");
  DefMath!("\\boxslash", "\u{29C4}", operator_role => "BINOP");

  // Arrows
  DefMath!("\\curlyveedownarrow", "\\lx@nounicode{\\curlyveedownarrow}", operator_role => "ARROW");
  DefMath!("\\curlyveeuparrow", "\\lx@nounicode{\\curlyveeuparrow}", operator_role => "ARROW");
  DefMath!("\\curlywedgedownarrow", "\\lx@nounicode{\\curlywedgedownarrow}", operator_role => "ARROW");
  DefMath!("\\curlywedgeuparrow", "\\lx@nounicode{\\curlywedgeuparrow}", operator_role => "ARROW");
  DefMath!("\\fatbslash", "\\lx@nounicode{\\fatbslash}", operator_role => "ARROW");
  DefMath!("\\fatsemi", "\u{2A1F}", operator_role => "RELOP");
  DefMath!("\\fatslash", "\\lx@nounicode{\\fatslash}", operator_role => "ARROW");
  DefMath!("\\interleave", "\u{2AF4}", operator_role => "RELOP");
  DefMath!("\\leftslice", "\u{2AA6}", operator_role => "RELOP");
  DefMath!("\\merge", "\u{2A07}", operator_role => "RELOP");
  DefMath!("\\minuso", "\u{29B5}", operator_role => "RELOP");
  DefMath!("\\moo", "\\lx@nounicode{\\moo}");
  DefMath!("\\obar", "\u{29B6}", operator_role => "RELOP");
  DefMath!("\\oblong", "\u{2395}", operator_role => "RELOP");
  DefMath!("\\obslash", "\u{29B8}", operator_role => "RELOP");
  DefMath!("\\ogreaterthan", "\u{29C1}", operator_role => "RELOP");
  DefMath!("\\olessthan", "\u{29C0}", operator_role => "RELOP");
  DefMath!("\\ovee", "\u{2228}\u{20DD}", operator_role => "RELOP");
  DefMath!("\\owedge", "\u{2227}\u{20DD}", operator_role => "RELOP");
  DefMath!("\\rightslice", "\u{2AA7}", operator_role => "RELOP");
  DefMath!("\\sslash", "\u{2AFD}", operator_role => "RELOP");
  DefMath!("\\talloblong", "\u{2AFF}", operator_role => "RELOP");
  DefMath!("\\varbigcirc", "\u{25EF}", operator_role => "MULOP");
  DefMath!("\\varcurlyvee", "\u{22CE}", operator_role => "RELOP");
  DefMath!("\\varcurlywedge", "\u{22CF}", operator_role => "RELOP");
  DefMath!("\\varoast", "\u{229B}", operator_role => "MULOP");
  DefMath!("\\varobar", "\u{29B6}", operator_role => "RELOP");
  DefMath!("\\varobslash", "\u{29B8}", operator_role => "MULOP");
  DefMath!("\\varocircle", "\u{229A}", operator_role => "MULOP");
  DefMath!("\\varodot", "\u{2299}", operator_role => "MULOP");
  DefMath!("\\varogreaterthan", "\u{29C1}", operator_role => "RELOP");
  DefMath!("\\varolessthan", "\u{29C0}", operator_role => "RELOP");
  DefMath!("\\varominus", "\u{2296}", operator_role => "ADDOP");
  DefMath!("\\varoplus", "\u{2295}", operator_role => "ADDOP",
    meaning => "additive-disjunction");
  DefMath!("\\varoslash", "\u{2298}", operator_role => "RELOP");
  DefMath!("\\varotimes", "\u{2297}", operator_role => "MULOP",
    meaning => "multiplicative-conjunction");
  DefMath!("\\varovee", "\u{2228}\u{20DD}", operator_role => "RELOP");
  DefMath!("\\varowedge", "\u{2227}\u{20DD}", operator_role => "RELOP");
  DefMath!("\\vartimes", "\u{00D7}", operator_role => "MULOP");

  // Big operators (SUMOP with dynamic mathstyle/scriptpos)
  // TODO: font => { size => 'Big' } not yet supported
  DefMath!("\\bigbox", "\u{25A1}",
    operator_role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigcurlywedge", "\u{22CF}",
    operator_role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigcurlyvee", "\u{22CE}",
    operator_role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\biginterleave", "\u{2AFC}",
    operator_role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigparallel", "\u{2016}",
    operator_role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigsqcap", "\u{2A05}",
    operator_role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigtriangledown", "\u{25BD}",
    operator_role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);
  DefMath!("\\bigtriangleup", "\u{25B3}",
    operator_role => "SUMOP", dynamic_mathstyle => true, dynamic_scriptpos => true);

  // More relational operators
  DefMath!("\\inplus", "\u{2A2D}", operator_role => "RELOP");
  DefMath!("\\niplus", "\u{2A2E}", operator_role => "RELOP");
  DefMath!("\\ntrianglelefteqslant", "\u{22EC}", operator_role => "RELOP");
  DefMath!("\\ntrianglerighteqslant", "\u{22ED}", operator_role => "RELOP");

  // Kludged subset/superset operators
  DefMath!("\\subsetplus",
    "\\lx@kludged{\\subset{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-0.8em,yoffset=0.3ex}{+}}}",
    operator_role => "RELOP", meaning => "subset-plus");
  DefMath!("\\subsetpluseq",
    "\\lx@kludged{\\subseteq{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-0.8em,yoffset=0.5ex}{+}}}",
    operator_role => "RELOP", meaning => "subset-equals-plus");
  DefMath!("\\supsetplus",
    "\\lx@kludged{\\supset{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-1em,yoffset=0.3ex}{+}}}",
    operator_role => "RELOP", meaning => "superset-plus");
  DefMath!("\\supsetpluseq",
    "\\lx@kludged{\\supseteq{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-1em,yoffset=0.5ex}{+}}}",
    operator_role => "RELOP", meaning => "superset-equals-plus");
  DefMath!("\\nplus",
    "\\lx@kludged{\\cap{\\scriptscriptstyle\\lx@tweaked{width=0pt,xoffset=-0.75em,yoffset=0.2ex}{+}}}",
    operator_role => "ADDOP", meaning => "intersection-plus");
  DefMath!("\\bignplus",
    "\\lx@kludged{\\bigcap\\mathchoice{\\lx@tweaked{width=0pt,xoffset=-1.6em,yoffset=0.2ex}{+}}{\\lx@tweaked{width=0pt,xoffset=-1.3em,yoffset=0.2ex}{+}}{\\lx@tweaked{width=0pt,xoffset=-1.3em,yoffset=0.2ex}{+}}{\\lx@tweaked{width=0pt,xoffset=-1.3em,yoffset=0.2ex}{+}}}",
    operator_role => "ADDOP", meaning => "intersection-plus",
    dynamic_mathstyle => true);

  DefMath!("\\trianglelefteqslant", "\u{22B4}", operator_role => "RELOP");
  DefMath!("\\trianglerighteqslant", "\u{22B5}", operator_role => "RELOP");
  DefMath!("\\arrownot", "\\lx@tweaked{width=0pt}{/}", operator_role => "RELOP");
  DefMath!("\\longarrownot", "\\lx@tweaked{width=0pt}{/}", operator_role => "RELOP");
  DefMath!("\\Arrownot", "\\lx@tweaked{width=0pt}{/}", operator_role => "RELOP");
  DefMath!("\\Longarrownot", "\\lx@tweaked{width=0pt}{/}", operator_role => "RELOP");
  DefMath!("\\mapsfromchar", "|", operator_role => "RELOP");
  DefMath!("\\Mapsfromchar", "|", operator_role => "RELOP");
  DefMath!("\\Mapstochar", "|", operator_role => "RELOP");
  DefMath!("\\Longmapsfrom", "\u{27FD}", operator_role => "ARROW");
  DefMath!("\\Longmapsto", "\u{27FE}", operator_role => "ARROW");
  DefMath!("\\Mapsfrom", "\u{2906}", operator_role => "ARROW");
  DefMath!("\\Mapsto", "\u{2907}", operator_role => "ARROW");
  DefMath!("\\leftarrowtriangle", "\u{21FD}", operator_role => "ARROW");
  DefMath!("\\leftrightarroweq", "\\stackrel{\\leftrightarrow}{-}",
    operator_role => "ARROW");
  DefMath!("\\leftrightarrowtriangle", "\u{21FF}", operator_role => "ARROW");
  DefMath!("\\lightning", "\u{21AF}", operator_role => "ARROW");
  DefMath!("\\longmapsfrom", "\u{27FB}", operator_role => "ARROW");
  DefMath!("\\mapsfrom", "\u{21A4}", operator_role => "ARROW");
  DefMath!("\\nnearrow", "\\lx@nounicode{\\nnearrow}", operator_role => "ARROW");
  DefMath!("\\nnwarrow", "\\lx@nounicode{\\nnwarrow}", operator_role => "ARROW");
  DefMath!("\\rightarrowtriangle", "\u{21FE}", operator_role => "ARROW");
  DefMath!("\\rrparenthesis", "\u{2988}", operator_role => "ARROW");
  DefMath!("\\shortdownarrow", "\u{2193}", operator_role => "ARROW");
  DefMath!("\\shortleftarrow", "\u{2190}", operator_role => "ARROW");
  DefMath!("\\shortrightarrow", "\u{2192}", operator_role => "ARROW");
  DefMath!("\\shortuparrow", "\u{2191}", operator_role => "ARROW");
  DefMath!("\\ssearrow", "\\lx@nounicode{\\ssearrow}", operator_role => "ARROW");
  DefMath!("\\sswarrow", "\\lx@nounicode{\\sswarrow}", operator_role => "ARROW");

  // Delimiters
  DefMath!("\\Lbag", "\u{27C5}", operator_role => "OPEN");
  DefMath!("\\Rbag", "\u{27C6}", operator_role => "CLOSE");
  DefMath!("\\lbag", "\u{27C5}", operator_role => "OPEN");
  DefMath!("\\llbracket", "\u{27E6}", operator_role => "OPEN");
  DefMath!("\\llceil",
    "\\lx@kludged{\\lx@tweaked{width=0pt,xoffset=0.3em}{\\lceil}\\lceil}",
    operator_role => "OPEN");
  DefMath!("\\rrceil",
    "\\lx@kludged{\\rceil\\lx@tweaked{width=0pt,xoffset=-0.3em}{\\rceil}}",
    operator_role => "CLOSE");
  DefMath!("\\llfloor",
    "\\lx@kludged{\\lx@tweaked{width=0pt,xoffset=0.3em}{\\lfloor}\\lfloor}",
    operator_role => "OPEN");
  DefMath!("\\rrfloor",
    "\\lx@kludged{\\rfloor\\lx@tweaked{width=0pt,xoffset=-0.31em}{\\rfloor}}",
    operator_role => "CLOSE");
  DefMath!("\\llparenthesis", "\u{2987}", operator_role => "OPEN");
  DefMath!("\\rrparenthesis", "\u{2988}", operator_role => "CLOSE");
  DefMath!("\\rbag", "\u{27C6}", operator_role => "CLOSE");
  DefMath!("\\rrbracket", "\u{27E7}", operator_role => "CLOSE");

  // Text symbol
  DefPrimitive!("\\varcopyright", "\u{00A9}");
});

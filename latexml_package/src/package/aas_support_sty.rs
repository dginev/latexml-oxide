use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aas_support.sty.ltxml — support macros for AAS styles

  // Package dependencies
  // RequirePackage!("aas_macros"); // not yet ported — raw TeX defs
  RequirePackage!("url");
  RequirePackage!("longtable");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("array");
  // RequirePackage!("lineno"); // not yet ported
  RequirePackage!("amssymb");
  // RequirePackage!("epsf"); // not yet ported
  RequirePackage!("ulem");

  // 2.1.3 Editorial Information
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received,name=Received]{#1}");
  DefMacro!("\\revised{}", "\\@add@frontmatter{ltx:date}[role=revised,name=Revised]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted,name=Accepted]{#1}");
  DefMacro!("\\journalid{}{}", "");
  DefMacro!("\\articleid{}{}", "");
  DefMacro!("\\paperid{}", "");
  DefMacro!("\\msid{}", "");
  DefMacro!("\\added{}", "");
  DefMacro!("\\replaced{}", "");
  DefMacro!("\\deleted{}", "");
  DefMacro!("\\explain{}", "");
  DefMacro!("\\edit{}{}", "");
  DefMacro!("\\ccc{}", "");
  DefMacro!("\\cpright{}{}", "\\@add@frontmatter{ltx:note}[role=copyright]{\\copyright #2: #1}");
  DefMacro!("\\journal{}", "");
  DefMacro!("\\volume{}", "");
  DefMacro!("\\issue{}", "");
  DefMacro!("\\SGMLbi{}", "#1");
  DefMacro!("\\SGMLbsc{}", "#1");
  DefMacro!("\\SGMLclc{}", "#1");
  DefMacro!("\\SGMLentity{}", "#1");
  DefMacro!("\\SGML{}", "");

  // 2.1.4 Short Comment
  DefMacro!("\\slugcomment{}", "\\@add@frontmatter{ltx:note}[role=slugcomment]{#1}");

  // 2.1.5 Running Heads
  DefMacro!("\\shorttitle{}", "\\@add@frontmatter{ltx:toctitle}{#1}");
  DefMacro!("\\shortauthors{}", "");
  DefMacro!("\\correspondingauthor{}", "\\lx@contact{correspondent}{#1}");
  DefMacro!("\\lefthead{}", "");
  DefMacro!("\\righthead{}", "");

  // 2.3 Title and Author Information
  AssignMapping!("DOCUMENT_CLASSES", "ltx_authors_multiline" => true);

  DefConstructor!("\\@@personname[]{}", "<ltx:personname>#2</ltx:personname>",
    mode => "restricted_horizontal", enter_horizontal => true);

  DefMacro!("\\author[]{}", "\\@add@frontmatter{ltx:creator}[role=author]{\\@@personname[#1]{#2}}");

  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  DefMacro!("\\affil", "\\affiliation");
  DefConstructor!("\\@@@altaffil{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\altaffiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@altaffil{#1}}");
  DefConstructor!("\\@@@authoraddr{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\authoraddr{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@authoraddr{#1}}");

  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");

  DefPrimitive!("\\and", None);
  DefMacro!("\\authoremail", "\\email");

  DefMacro!("\\software{}", "\\@add@frontmatter{ltx:note}[role=software]{#1}");
  DefMacro!("\\submitjournal{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\nocollaboration{}", "");

  // 2.5 Keywords
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  Let!("\\subjectheadings", "\\keywords");

  // 2.6 Comments to Editors
  DefMacro!("\\notetoeditor{}", "");
  NewCounter!("editornote");
  DefMacro!("\\theeditornote", "E\\arabic{editornote}");

  // 2.8 Figure and Table Placement
  DefMacro!("\\placetable{}", "");
  DefMacro!("\\placefigure{}", "");
  DefMacro!("\\placeplate{}", "");
  NewCounter!("plate");
  DefMacro!("\\platename", "Plate");
  DefMacro!("\\platewidth{Dimension}", "");
  DefMacro!("\\platenum{}", "\\def\\theplate{#1}");
  DefMacro!("\\gridline{}", "");

  // 2.9 Acknowledgements
  Tag!("ltx:acknowledgements", auto_close => true);
  DefConstructor!("\\acknowledgements", "<ltx:acknowledgements>");
  Let!("\\acknowledgments", "\\acknowledgements");

  // 2.10 Facilities
  DefConstructor!("\\facility{}", "<ltx:text class='ltx_ast_facility'>#1</ltx:text>",
    enter_horizontal => true);
  DefMacro!("\\facilities{}", "\\@add@frontmatter{ltx:note}[role=facilities]{#1}");

  // 2.11 Appendices
  DefMacro!("\\appendix", "\\@appendix");

  // 2.12 Equations
  DefMacro!("\\mathletters", "\\lx@equationgroup@subnumbering@begin");
  DefMacro!("\\endmathletters", "\\lx@equationgroup@subnumbering@end");

  // 2.13 Citations
  DefMacro!("\\markcite{}", "");
  RequirePackage!("natbib");
  RequirePackage!("graphicx");

  // 2.14 Electronic Art
  DefMacro!("\\figurenum{}", "\\def\\thefigure{#1}");
  DefMacro!("\\epsscale{}", "");
  DefMacro!("\\plotone Semiverbatim", "\\includegraphics[width=\\textwidth]{#1}");
  DefMacro!("\\plottwo Semiverbatim Semiverbatim",
    "\\hbox{\\includegraphics[width=\\textwidth]{#1}\\includegraphics[width=\\textwidth]{#2}}");
  DefMacro!("\\plotfiddle Semiverbatim {}{}{}{}{}{}",
    "\\includegraphics[width=#4pt,height=#5pt]{#1}");

  // 2.14.2 Figure Captions
  DefMacro!("\\@figcaption {}", "\\begin{figure}#1\\end{figure}");

  // 2.15 Tables
  // RequirePackage!("deluxetable"); // not yet ported
  DefMacro!("\\phn", "\\phantom{0}");
  DefMacro!("\\phd", "\\phantom{.}");
  DefMacro!("\\phs", "\\phantom{+}");
  DefMacro!("\\phm{}", "\\phantom{string}");

  DefEnvironment!("{interactive}{}{}", "#body");
  DefEnvironment!("{longrotatetable}", "#body");

  // 2.17.1 Celestial Objects and Data Sets
  DefConstructor!("\\objectname OptionalSemiverbatim {}",
    "<ltx:text class='ltx_ast_objectname'>#2 (catalog #1)</ltx:text>",
    enter_horizontal => true);
  Let!("\\object", "\\objectname");
  DefConstructor!("\\dataset OptionalSemiverbatim {}",
    "<ltx:text class='ltx_ast_dataset'>#2 (catalog #1)</ltx:text>",
    enter_horizontal => true);

  // 2.17.2 Ionic Species
  DefMacro!("\\ion{}{}", "{#1~\\expandafter\\uppercase\\expandafter{\\romannumeral #2}}");

  DefPrimitive!("\\sbond", "\u{2212}");
  DefPrimitive!("\\dbond", "=");
  DefPrimitive!("\\tbond", "\u{2261}");

  // 2.17.3 Fractions
  DefMacro!("\\case{}{}", "\\ensuremath{\\frac{#1}{#2}}");
  Let!("\\slantfrac", "\\case");

  // 2.17.4 Astronomical Symbols
  DefPrimitive!("\\micron", "\u{00B5}m");
  DefMacro!("\\Sun", "\\sun");
  DefMacro!("\\Sol", "\\sun");
  DefPrimitive!("\\sun", "\u{2609}");
  DefPrimitive!("\\Mercury", "\u{263F}");
  DefPrimitive!("\\Venus", "\u{2640}");
  DefMacro!("\\Earth", "\\earth");
  DefMacro!("\\Terra", "\\earth");
  DefPrimitive!("\\earth", "\u{2295}");
  DefPrimitive!("\\Mars", "\u{2642}");
  DefPrimitive!("\\Jupiter", "\u{2643}");
  DefPrimitive!("\\Saturn", "\u{2644}");
  DefPrimitive!("\\Uranus", "\u{2645}");
  DefPrimitive!("\\Neptune", "\u{2646}");
  DefPrimitive!("\\Pluto", "\u{2647}");
  DefPrimitive!("\\Moon", "\u{263D}");
  DefMacro!("\\Luna", "\\Moon");
  DefPrimitive!("\\Aries", "\u{2648}");
  DefMacro!("\\VEq", "\\Aries");
  DefPrimitive!("\\Taurus", "\u{2649}");
  DefPrimitive!("\\Gemini", "\u{264A}");
  DefPrimitive!("\\Cancer", "\u{264B}");
  DefPrimitive!("\\Leo", "\u{264C}");
  DefPrimitive!("\\Virgo", "\u{264D}");
  DefPrimitive!("\\Libra", "\u{264E}");
  DefMacro!("\\AEq", "\\Libra");
  DefPrimitive!("\\Scorpius", "\u{264F}");
  DefPrimitive!("\\Sagittarius", "\u{2650}");
  DefPrimitive!("\\Capricornus", "\u{2651}");
  DefPrimitive!("\\Aquarius", "\u{2652}");
  DefPrimitive!("\\Pisces", "\u{2653}");

  DefPrimitive!("\\diameter", "\u{2300}");
  DefPrimitive!("\\sq", "\u{25A1}");

  DefPrimitive!("\\arcdeg", "\u{00B0}");
  Let!("\\degr", "\\arcdeg");
  DefPrimitive!("\\arcmin", "\u{2032}");
  DefPrimitive!("\\arcsec", "\u{2033}");
  DefMacro!("\\nodata", " ~$\\cdots$~ ");

  DefMacro!("\\fd", "\\ensuremath{\\@fd}");
  DefMacro!("\\fh", "\\ensuremath{\\@fh}");
  DefMacro!("\\fm", "\\ensuremath{\\@fm}");
  DefMacro!("\\fs", "\\ensuremath{\\@fs}");
  DefMacro!("\\fdg", "\\ensuremath{\\@fdg}");
  DefMacro!("\\farcm", "\\ensuremath{\\@farcm}");
  DefMacro!("\\farcs", "\\ensuremath{\\@farcs}");
  DefMacro!("\\fp", "\\ensuremath{\\@fp}");

  DefMacro!("\\onehalf", "\\ifmmode\\case{1}{2}\\else\\text@onehalf\\fi");
  DefPrimitive!("\\text@onehalf", "\u{00BD}");
  DefMacro!("\\onethird", "\\ifmmode\\case{1}{3}\\else\\text@onethird\\fi");
  DefPrimitive!("\\text@onethird", "\u{2153}");
  DefMacro!("\\twothirds", "\\ifmmode\\case{2}{3}\\else\\text@twothirds\\fi");
  DefPrimitive!("\\text@twothirds", "\u{2154}");
  DefMacro!("\\onequarter", "\\ifmmode\\case{1}{4}\\else\\text@onequarter\\fi");
  DefPrimitive!("\\text@onequarter", "\u{00BC}");
  DefMacro!("\\threequarters", "\\ifmmode\\case{3}{4}\\else\\text@threequarters\\fi");
  DefPrimitive!("\\text@threequarters", "\u{00BE}");

  // amssymb aliases
  // RequirePackage!("latexsym"); // not yet ported
  RequirePackage!("amssymb");

  Let!("\\la", "\\lesssim");
  Let!("\\ga", "\\gtrsim");

  // 2.17.5 Hypertext
  DefMacro!("\\anchor Semiverbatim Semiverbatim", "#2");

  // Misc
  DefMacro!("\\eqsecnum",
    "\\@addtoreset{equation}{section}\\def\\theequation{\\arabic{section}-\\arabic{equation}}");

  DefMacro!("\\singlespace", "");
  DefMacro!("\\doublespace", "");
  DefMacro!("\\tighten", "");
  DefMacro!("\\tightenlines", "");
  DefMacro!("\\nohyphenation", "");
  DefMacro!("\\offhyphenation", "");
  DefMacro!("\\ptlandscape", "");
  DefMacro!("\\refpar", "");
  DefMacro!("\\traceoutput", "");
  DefMacro!("\\tracingplain", "");

  DefMacro!("\\noprint {}", "");
  DefMacro!("\\figsetstart", "{\\bf Fig. Set}");
  DefMacro!("\\figsetend", "");
  DefMacro!("\\figsetgrpstart", "");
  DefMacro!("\\figsetgrpend", "");
  DefMacro!("\\figsetnum {}", "{\\bf #1.}");
  DefMacro!("\\figsettitle {}", "{\\bf #1}");
  DefMacro!("\\figsetgrpnum {}", "");
  DefMacro!("\\figsetgrptitle {}", "");
  DefMacro!("\\figsetplot {}", "");
  DefMacro!("\\figsetgrpnote {}", "");
});

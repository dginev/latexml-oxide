use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aas_support.sty.ltxml — support macros for AAS styles

  // Package dependencies — Perl L28-39
  RequirePackage!("aas_macros");
  RequirePackage!("url");
  RequirePackage!("longtable");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("array");
  RequirePackage!("lineno");
  RequirePackage!("amssymb");
  RequirePackage!("epsf");
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

  // Affiliation marks — Perl L126-132
  DefMacro!("\\altaffilmark{}", "\\@altaffilmark{#1}");
  DefConstructor!("\\@altaffilmark{}", "<ltx:note role='affiliationmark' mark='#1'/>",
    enter_horizontal => true);
  DefConstructor!("\\altaffiltext{}{}", "<ltx:note role='affiliationtext' mark='#1'>#2</ltx:note>");

  DefMacro!("\\software{}", "\\@add@frontmatter{ltx:note}[role=software]{#1}");
  DefMacro!("\\submitjournal{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");

  // DOI — Perl L137-138
  DefConstructor!("\\doi{}", "<ltx:ref href='https://doi.org/#1'>#1</ltx:ref>",
    enter_horizontal => true);

  // Collaboration — Perl L139-141
  DefConstructor!("\\@@@collaborator{}", "<ltx:note role='collaborator'>#1</ltx:note>");
  DefMacro!("\\collaboration{}{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@collaborator{#2}}");
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

  // Plate environments — Perl L179-201
  DefEnvironment!("{plate}[]",
    "<ltx:float xml:id='#id' class='ltx_float_plate'>#tags#body</ltx:float>",
    mode => "internal_vertical"
  );
  DefEnvironment!("{plate*}[]",
    "<ltx:float xml:id='#id' class='ltx_float_plate'>#tags#body</ltx:float>",
    mode => "internal_vertical"
  );

  // Fig macros — Perl L205-221
  DefMacro!("\\aas@fig Semiverbatim {Dimension}{}",
    "\\begin{figure}\\caption{#3}\\includegraphics[width=#2]{#1}\\end{figure}");
  DefMacro!("\\fig Semiverbatim", "\\aas@fig{#1}");
  Let!("\\leftfig", "\\fig");
  Let!("\\rightfig", "\\fig");
  Let!("\\boxedfig", "\\fig");
  DefMacro!("\\rotatefig{Number} Semiverbatim {Dimension}{}",
    "\\begin{figure}\\caption{#4}\\includegraphics[width=#3,angle=#1]{#2}\\end{figure}");

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

  // 2.12 Equations — Perl L261
  DefMacro!("\\eqnum{}", "");

  // 2.13 Citations — Perl L264-293
  DefMacro!("\\markcite{}", "");
  RequirePackage!("natbib");

  // References environment — Perl L283-293
  DefConstructor!("\\references",
    "<ltx:bibliography xml:id='#id'><ltx:biblist>");
  DefConstructor!("\\endreferences",
    "</ltx:biblist></ltx:bibliography>");
  Let!("\\reference", "\\bibitem");

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
  RequirePackage!("deluxetable");
  Let!("\\planotable", "\\deluxetable");
  Let!("\\endplanotable", "\\enddeluxetable");

  // Perl: aas_support.sty.ltxml L380-383
  Let!("\\splitdeluxetable", "\\deluxetable");
  Let!("\\endsplitdeluxetable", "\\enddeluxetable");
  state::let_i(&T_CS!("\\splitdeluxetable*"), &T_CS!("\\deluxetable*"), None);
  state::let_i(&T_CS!("\\endsplitdeluxetable*"), &T_CS!("\\enddeluxetable*"), None);

  // Decimal table conditionals — Perl L338-345
  DefConditional!("\\ifcolnumberson");
  DefConditional!("\\ifdeluxedecimals");
  DefMacro!("\\deluxedecimals", "\\global\\deluxedecimalstrue");
  RawTeX!("\\global\\deluxedecimalsfalse");
  Let!("\\decimals", "\\deluxedecimals");
  DefMacro!("\\colnumbers", "");
  DefMacro!("\\deluxedecimalcolnumbers", "\\deluxedecimalstrue\\colnumbersontrue");
  Let!("\\decimalcolnumbers", "\\deluxedecimalcolnumbers");

  // Hidden column environment — Perl L374
  DefEnvironment!("{eatone}", "");

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

  // Photometric bands — Perl L529-533
  DefPrimitive!("\\ubvr", "UBVR");
  DefPrimitive!("\\ub", "U\u{2000}B");
  DefPrimitive!("\\bv", "B\u{2000}V");
  DefPrimitive!("\\vr", "V\u{2000}R");
  DefPrimitive!("\\ur", "U\u{2000}R");

  // amssymb aliases
  RequirePackage!("latexsym");
  RequirePackage!("amssymb");

  Let!("\\la", "\\lesssim");
  Let!("\\ga", "\\gtrsim");

  // Nominal conversion constants — Perl L545-560
  DefMacro!("\\nomSolarEffTemp", "\\leavevmode\\hbox{\\boldmath$\\mathcal{T}^{\\rm N}_{\\mathrm{eff}\\odot}$}");
  DefMacro!("\\nomTerrEqRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{E\\mathrm e}$}");
  DefMacro!("\\nomTerrPolarRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{E\\mathrm p}$}");
  DefMacro!("\\nomJovianEqRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{J\\mathrm e}$}");
  DefMacro!("\\nomJovianPolarRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{J\\mathrm p}$}");
  DefMacro!("\\nomTerrMass", "\\leavevmode\\hbox{\\boldmath$(\\mathcal{GM})^{\\rm N}_{\\mathrm E}$}");
  DefMacro!("\\nomJovianMass", "\\leavevmode\\hbox{\\boldmath$(\\mathcal{GM})^{\\rm N}_{\\mathrm J}$}");
  DefMacro!("\\Qnom", "\\leavevmode\\hbox{\\boldmath$\\mathcal{Q}^{\\rm N}_{\\odot}$}");
  Let!("\\Qn", "\\Qnom");
  DefMacro!("\\nom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{\\odot}$}");
  DefMacro!("\\Eenom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Ee}$}");
  DefMacro!("\\Epnom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Ep}$}");
  DefMacro!("\\Jenom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Je}$}");
  DefMacro!("\\Jpnom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Jp}$}");

  // 2.17.5 Hypertext — Perl L563-577
  DefConstructor!("\\anchor Semiverbatim Semiverbatim", "<ltx:ref href='#1'>#2</ltx:ref>",
    enter_horizontal => true);
  DefConstructor!("\\@@email Semiverbatim", "<ltx:ref href='mailto:#1'>#1</ltx:ref>",
    enter_horizontal => true);

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

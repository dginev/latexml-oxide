//! aa_support.sty — Astronomy & Astrophysics journal support
//! Perl: aa_support.sty.ltxml — 469 lines
//! Shared by aa.cls and aa.sty
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Class options — Perl L26-45
  DeclareOption!("10pt", {});
  DeclareOption!("11pt", {});
  DeclareOption!("12pt", {});
  DeclareOption!("twoside", {});
  DeclareOption!("onecolumn", {});
  DeclareOption!("twocolumn", {});
  DeclareOption!("draft", {});
  DeclareOption!("final", {});
  DeclareOption!("referee", {});
  DeclareOption!("leqno", {});
  DeclareOption!("fleqn", {});
  DeclareOption!("longauth", {});
  DeclareOption!("rnote", {});
  DeclareOption!("runningheads", {});
  DeclareOption!("structabstract", {});
  DeclareOption!("traditabstract", {});
  DeclareOption!("letter", {});
  ProcessOptions!();

  // Dependencies — Perl L47-63
  RequirePackage!("inst_support");
  RequirePackage!("calc");
  RequirePackage!("etex");
  RequirePackage!("fontenc");
  RequirePackage!("geometry");
  RequirePackage!("setspace");
  RequirePackage!("fancyhdr");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("url");
  RequirePackage!("enumerate");
  RequirePackage!("longtable");
  RequirePackage!("xspace");
  RequirePackage!("babel");
  RequirePackage!("rotating");

  // Frontmatter — Perl L70-128
  DefMacro!("\\subtitle{}", "\\@add@frontmatter{ltx:subtitle}{#1}");
  // Structured abstract — Perl L75-86, simplified
  DefMacro!("\\abstract@old{}", "\\@add@frontmatter{ltx:abstract}{#1}");
  DefMacro!("\\abstract@new{}{}{}{}{}", "\\@add@frontmatter{ltx:abstract}{#2\\par#3\\par#4}");

  DefMacro!("\\keywordname", "\\sffamily\\bfseries Key Words.");
  DefRegister!("\\titlerunning" => Tokens!());
  DefRegister!("\\authorrunning" => Tokens!());
  DefMacro!("\\authrun", "");
  DefMacro!("\\titrun", "");
  DefMacro!("\\offprints{}", "\\@add@frontmatter{ltx:note}[role=offprints]{#1}");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\mail Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefMacro!("\\email Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");
  DefMacro!("\\journalname{}", "");
  DefMacro!("\\DOI{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\doi{}", "\\@add@frontmatter{ltx:classification}[scheme=doi]{#1}");
  DefMacro!("\\rnotename", "(Research Note)");
  DefMacro!("\\rnotname", "(RN)");
  DefMacro!("\\headnote{}", "\\@add@frontmatter{ltx:note}{#1}");
  DefMacro!("\\dedication{}", "\\@add@frontmatter{ltx:note}[role=dedicatory]{#1}");
  DefMacro!("\\mailname", "\\it Correspondence to \\/");
  DefMacro!("\\idline{}{}", "");
  DefMacro!("\\msnr{}", "");
  DefMacro!("\\institutename", "");
  DefMacro!("\\hugehead", "");
  DefMacro!("\\AALogo", "Astronomy and Astrophysics");

  // Acknowledgements — Perl L132-141
  DefConstructor!("\\acknowledgements", "<ltx:acknowledgements>");
  DefConstructor!("\\endacknowledgements", "</ltx:acknowledgements>");
  Let!("\\acknowledgement", "\\acknowledgements");
  Let!("\\endacknowledgement", "\\endacknowledgements");
  Tag!("ltx:acknowledgements", auto_close => true);
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  DefMacro!("\\ackname", "Acknowledgements");

  // Theorem environments — Perl L142-150
  RawTeX!("\\@ifundefined{corollary}{\\newtheorem{corollary}[theorem]{Corollary}}{}");
  RawTeX!("\\@ifundefined{definition}{\\newtheorem{definition}[theorem]{Definition}}{}");
  RawTeX!("\\@ifundefined{example}{\\newtheorem{example}[theorem]{Example}}{}");
  RawTeX!("\\@ifundefined{lemma}{\\newtheorem{lemma}[theorem]{Lemma}}{}");
  RawTeX!("\\@ifundefined{note}{\\newtheorem{note}[theorem]{Note}}{}");
  RawTeX!("\\@ifundefined{problem}{\\newtheorem{problem}[theorem]{Problem}}{}");

  // Keywords — Perl L137-160
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  Let!("\\endkeywords", "\\relax");

  // Section formatting — Perl L162-200
  DefMacro!("\\startsection", "");
  DefMacro!("\\acknowledgements", "\\section*{Acknowledgements}");
  Let!("\\acknowledgement", "\\acknowledgements");

  // Tables/figures — Perl L210-280
  DefMacro!("\\tablefont", "\\small");
  DefMacro!("\\tablenote{}{}", "\\footnote{#2}");
  DefMacro!("\\tablefoot{}", "#1");
  DefMacro!("\\tablecaption{}", "\\caption{#1}");
  DefEnvironment!("{longtab}{}", "#body");

  // Math/units — Perl L282-350
  DefMacro!("\\rg", "\\relax");
  DefMacro!("\\degr", "\u{00B0}");
  DefMacro!("\\arcmin", "\u{2032}");
  DefMacro!("\\arcsec", "\u{2033}");
  DefMacro!("\\la", "\\lesssim");
  DefMacro!("\\ga", "\\gtrsim");
  DefMacro!("\\getsto", "\\rightleftharpoons");
  DefMacro!("\\cor", "\\mathchoice{\\,\\raise.38ex\\hbox{$\\scriptstyle \\hat=$ }\\,}{\\hat=}{\\hat=}{\\hat=}");
  DefMacro!("\\sun", "\u{2609}");
  DefMacro!("\\diameter", "\u{2300}");
  DefMacro!("\\sq", "\u{25A1}");
  DefMacro!("\\fd", ".\\!^{\\mathrm{d}}");
  DefMacro!("\\fh", ".\\!^{\\mathrm{h}}");
  DefMacro!("\\fm", ".\\!^{\\mathrm{m}}");
  DefMacro!("\\fs", ".\\!^{\\mathrm{s}}");
  DefMacro!("\\fp", ".\\!^{\\mathrm{p}}");
  DefMacro!("\\udeg", "\\!^{\\circ}");
  DefMacro!("\\uarcmin", "\\!^{\\prime}");
  DefMacro!("\\uarcsec", "\\!^{\\prime\\prime}");
  DefMacro!("\\ion{}{}", "#1\\,{\\sc #2}");
  DefMacro!("\\element{}{}", "\\ensuremath{{}^{#2}\\mathrm{#1}}");
  DefMacro!("\\isotope{}{}", "\\ensuremath{{}^{#2}\\mathrm{#1}}");

  // Object names — Perl L352-395
  DefMacro!("\\object{}", "#1");
  DefMacro!("\\objectname{}", "#1");
  DefMacro!("\\citeyearpar{}", "");

  // Misc — Perl L400-469
  DefMacro!("\\tnote{}", "\\footnote{#1}");
  DefMacro!("\\fnmsep", ",\\,");
  DefMacro!("\\at", "@");
  DefMacro!("\\aap",    "A\\&A");
  DefMacro!("\\aapr",   "A\\&A~Rev.");
  DefMacro!("\\aaps",   "A\\&AS");
  DefMacro!("\\aj",     "AJ");
  DefMacro!("\\apj",    "ApJ");
  DefMacro!("\\apjl",   "ApJ");
  DefMacro!("\\apjs",   "ApJS");
  DefMacro!("\\apss",   "Ap\\&SS");
  DefMacro!("\\araa",   "ARA\\&A");
  DefMacro!("\\azh",    "AZh");
  DefMacro!("\\baas",   "BAAS");
  DefMacro!("\\bac",    "Bull. astr. Inst. Czechosl.");
  DefMacro!("\\caa",    "Chinese Astron. Astrophys.");
  DefMacro!("\\cjaa",   "Chinese J. Astron. Astrophys.");
  DefMacro!("\\gca",    "Geochim. Cosmochim. Acta");
  DefMacro!("\\grl",    "Geophys. Res. Lett.");
  DefMacro!("\\iaucirc", "IAU Circ.");
  DefMacro!("\\icarus", "Icarus");
  DefMacro!("\\jcap",   "J. Cosmology Astropart. Phys.");
  DefMacro!("\\jrasc",  "JRASC");
  DefMacro!("\\memras", "MmRAS");
  DefMacro!("\\mnras",  "MNRAS");
  DefMacro!("\\nat",    "Nature");
  DefMacro!("\\nphysa", "Nucl. Phys. A");
  DefMacro!("\\pasa",   "PASA");
  DefMacro!("\\pasp",   "PASP");
  DefMacro!("\\pasj",   "PASJ");
  DefMacro!("\\physrep", "Phys. Rep.");
  DefMacro!("\\physscr", "Phys. Scr.");
  DefMacro!("\\planss", "Planet. Space Sci.");
  DefMacro!("\\procspie", "Proc. SPIE");
  DefMacro!("\\rmxaa",  "Rev. Mexicana Astron. Astrofis.");
  DefMacro!("\\qjras", "QJRAS");
  DefMacro!("\\skytel", "S\\&T");
  DefMacro!("\\solphys", "Sol. Phys.");
  DefMacro!("\\sovast", "Soviet Ast.");
  DefMacro!("\\ssr",    "Space Sci. Rev.");
  DefMacro!("\\zap",    "ZAp");
  DefMacro!("\\prd",    "Phys. Rev. D");
  DefMacro!("\\prl",    "Phys. Rev. Lett.");
  DefMacro!("\\nar",    "New A Rev.");
  DefMacro!("\\na",     "New A");
  DefMacro!("\\lrr",    "Living Rev. Relativity");
});

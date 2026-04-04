//! iopart_support.sty — IOP Publishing journal support
//! Perl: iopart_support.sty.ltxml — 345 lines
//! Used by Journal of Physics, Classical and Quantum Gravity, etc.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Conditionals — Perl L22-26
  RawTeX!("\\newif\\ifletter\\letterfalse");
  RawTeX!("\\newif\\ifnumbysec\\numbysecfalse");
  RawTeX!("\\newif\\ifiopams\\iopamsfalse");

  // Equation numbering — Perl L28-29
  DefMacro!("\\eqnobysec", "\\numbysectrue\\@addtoreset{equation}{section}");

  // Frontmatter — Perl L33-90
  DefMacro!("\\title[]{}", "\\@add@frontmatter{ltx:title}{#2}");
  Let!("\\paper", "\\title");
  DefMacro!("\\article[]{}{}", "\\@add@frontmatter{ltx:title}{#3}");
  DefMacro!("\\letter{}", "\\article[Letter to the Editor]{Letter to the Editor}{#1}");
  DefMacro!("\\review[]{}", "\\article[#1]{Review Article}{#2}");
  DefMacro!("\\topical[]{}", "\\article[#1]{Topical Review}{#2}");
  DefMacro!("\\comment[]{}", "\\article[#1]{Comment}{#2}");
  DefMacro!("\\rapid[]{}", "\\article[#1]{Rapid Communication}{#2}");
  DefMacro!("\\note[]{}", "\\article[#1]{Note}{#2}");
  DefMacro!("\\prelim[]{}", "\\article[#1]{Preliminary Communication}{#2}");

  // Authors — Perl L55-80
  DefMacro!("\\author{}", "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#1}}");
  DefMacro!("\\address{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\ead Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");

  // Dates — Perl L82-86
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\revised{}", "\\@add@frontmatter{ltx:date}[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");
  DefMacro!("\\published{}", "\\@add@frontmatter{ltx:date}[role=published]{#1}");
  DefMacro!("\\online{}", "\\@add@frontmatter{ltx:date}[role=online]{#1}");

  // Abstract/Keywords — Perl L95-120
  DefMacro!("\\nosections", "");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  DefMacro!("\\submitto{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\pacs{}", "\\@add@frontmatter{ltx:classification}[scheme=pacs]{#1}");
  DefMacro!("\\ams{}", "\\@add@frontmatter{ltx:classification}[scheme=msc]{#1}");

  // Acknowledgements — Perl L122
  DefMacro!("\\ack", "\\section*{Acknowledgments}");
  Let!("\\ackn", "\\ack");

  // Misc — Perl L130-180
  DefMacro!("\\ft{}", "\\footnote{#1}");
  DefMacro!("\\query{}", "");
  DefMacro!("\\eqalign{}", "\\begin{aligned}#1\\end{aligned}");
  DefMacro!("\\eqalignno{}", "\\begin{aligned}#1\\end{aligned}");
  DefMacro!("\\cases{}", "\\begin{cases}#1\\end{cases}");
  DefMacro!("\\pmatrix{}", "\\begin{pmatrix}#1\\end{pmatrix}");
  DefMacro!("\\buildrel{} \\over{}", "\\mathrel{\\mathop{#3}\\limits^{#1}}");
  DefMacro!("\\overmark{}", "");
  DefMacro!("\\fl", "");
  DefMacro!("\\bi{}", "\\boldsymbol{#1}");
  DefMacro!("\\bbox{}", "\\mathbf{#1}");

  // Table/figure formatting — Perl L185-220
  DefMacro!("\\lineup", "");
  DefMacro!("\\0", "\\phantom{0}");
  DefMacro!("\\m", "\\phantom{-}");
  DefMacro!("\\centre{}{}", "\\multicolumn{#1}{c}{#2}");
  DefMacro!("\\crule{}", "\\cline{#1}");
  DefMacro!("\\ns", "");
  DefMacro!("\\ms", "\\noalign{\\vskip3pt}");
  DefMacro!("\\bs", "\\noalign{\\vskip6pt}");
  DefEnvironment!("{indented}", "#body");

  // Math symbols — Perl L225-280
  DefMacro!("\\la", "\\lesssim");
  DefMacro!("\\ga", "\\gtrsim");
  DefMacro!("\\sun", "\u{2609}");
  DefMacro!("\\degr", "\u{00B0}");
  DefMacro!("\\arcmin", "\u{2032}");
  DefMacro!("\\arcsec", "\u{2033}");

  // Math operators — Perl L110-134
  DefMath!("\\rmd", "\\mathrm{d}", role => "DIFFOP", meaning => "differential-d");
  DefMath!("\\rme", "\u{2147}", role => "ID", meaning => "exponential-e");
  DefMath!("\\rmi", "\u{2148}", role => "ID", meaning => "imaginary-i");
  Let!("\\e", "\\rme");
  DefMacro!("\\case{}{}", "{\\textstyle\\frac{#1}{#2}}");
  DefMath!("\\Tr", "\\mathrm{Tr}", role => "OPFUNCTION", meaning => "trace");
  DefMath!("\\tr", "\\mathrm{tr}", role => "OPFUNCTION", meaning => "trace");
  DefMath!("\\Or", "\\mathrm{O}", role => "OPFUNCTION", meaning => "Big-O");
  DefMacro!("\\dsty", "\\displaystyle");
  DefMacro!("\\tsty", "\\textstyle");
  DefMacro!("\\ssty", "\\scriptstyle");
  DefMacro!("\\sssty", "\\scriptscriptstyle");

  // Math symbols — Perl L145-158
  DefPrimitive!("\\opencircle", "\u{25CB}");
  DefPrimitive!("\\opensquare", "\u{25A1}");
  DefPrimitive!("\\opentriangle", "\u{25B3}");
  DefPrimitive!("\\opentriangledown", "\u{25BD}");
  DefPrimitive!("\\opendiamond", "\u{25C6}");
  DefPrimitive!("\\fullcircle", "\u{25CF}");
  DefPrimitive!("\\fullsquare", "\u{25A0}");

  // Equation numbering — Perl L160-162
  DefMacro!("\\numparts", "\\lx@equationgroup@subnumbering@begin");
  DefMacro!("\\endnumparts", "\\lx@equationgroup@subnumbering@end");
  Let!("\\pcal", "\\cal");
  Let!("\\pmit", "\\mathnormal");

  // Cross-referencing (with text prefixes) — Perl L185-192
  DefMacro!("\\eref{}", "(\\ref{#1})");
  DefMacro!("\\sref{}", "section~\\ref{#1}");
  DefMacro!("\\fref{}", "figure~\\ref{#1}");
  DefMacro!("\\tref{}", "table~\\ref{#1}");
  DefMacro!("\\Eref{}", "Equation (\\ref{#1})");
  DefMacro!("\\Sref{}", "Section~\\ref{#1}");
  DefMacro!("\\Fref{}", "Figure~\\ref{#1}");
  DefMacro!("\\Tref{}", "Table~\\ref{#1}");
  DefMacro!("\\aref{}", "\\ref{#1}");
  DefMacro!("\\Aref{}", "\\ref{#1}");

  // Tables — Perl L198-230
  DefMacro!("\\noappendix", "\\setcounter{figure}{0}\\setcounter{table}{0}\\def\\thetable{\\arabic{table}}\\def\\thefigure{\\arabic{figure}}");
  DefMacro!("\\Tables", "\\section*{Tables and table captions}\\noappendix");
  DefMacro!("\\Figures", "\\section*{Figure captions}\\noappendix");
  DefMacro!("\\Figure{}", "\\begin{figure}\\caption{#1}\\end{figure}");
  DefMacro!("\\lineup", "");
  DefMacro!("\\boldarrayrulewidth", "1pt");
  Let!("\\bhline", "\\hline");
  DefMacro!("\\br", "\\hline");
  DefMacro!("\\mr", "\\hline");

  // Bibliography — Perl L233-245
  DefMacro!("\\Bibliography{}", "\\section*{References}\\numrefs{#1}");
  DefMacro!("\\References", "\\section*{References}\\refs");
  DefMacro!("\\numrefs{}", "\\begin{thebibliography}{#1}");
  DefMacro!("\\endnumrefs", "\\end{thebibliography}");
  Let!("\\endbib", "\\endnumrefs");
  DefMacro!("\\thereferences", "\\begin{thebibliography}{}");
  DefMacro!("\\endthereferences", "\\end{thebibliography}");
  DefMacro!("\\harvard", "\\begin{thebibliography}{}");
  DefMacro!("\\endharvard", "\\end{thebibliography}");
  DefMacro!("\\refs", "\\begin{thebibliography}{}");
  DefMacro!("\\endrefs", "\\end{thebibliography}");

  // Acknowledgements — Perl L249-251
  DefConstructor!("\\ack", "<ltx:acknowledgements>");
  DefConstructor!("\\ackn", "<ltx:acknowledgements>");
  Tag!("ltx:acknowledgements", auto_close => true);

  // Journal abbreviations — Perl L258-343
  DefMacro!("\\etal", "\\textit{et al\\/}");
  DefMacro!("\\dash", "-----");
  DefMacro!("\\CQG",   "\\textit{Class. Quantum Grav.}");
  DefMacro!("\\EJP",   "\\textit{Eur. J. Phys.}");
  DefMacro!("\\IP",    "\\textit{Inverse Problems\\/}");
  DefMacro!("\\JO",    "\\textit{J. Opt.}");
  DefMacro!("\\JOA",   "\\textit{J. Opt. A: Pure Appl. Opt.}");
  DefMacro!("\\JOB",   "\\textit{J. Opt. B: Quantum Semiclass. Opt.}");
  DefMacro!("\\JPA",   "\\textit{J. Phys. A: Math. Gen.}");
  DefMacro!("\\JPB",   "\\textit{J. Phys. B: At. Mol. Phys.}");
  DefMacro!("\\jpb",   "\\textit{J. Phys. B: At. Mol. Opt. Phys.}");
  DefMacro!("\\JPC",   "\\textit{J. Phys. C: Solid State Phys.}");
  DefMacro!("\\JPCM",  "\\textit{J. Phys.: Condens. Matter\\/}");
  DefMacro!("\\JPD",   "\\textit{J. Phys. D: Appl. Phys.}");
  DefMacro!("\\JPG",   "\\textit{J. Phys. G: Nucl. Phys.}");
  DefMacro!("\\jpg",   "\\textit{J. Phys. G: Nucl. Part. Phys.}");
  DefMacro!("\\MST",   "\\textit{Meas. Sci. Technol.}");
  DefMacro!("\\NJP",   "\\textit{New J. Phys.}");
  DefMacro!("\\NL",    "\\textit{Nonlinearity\\/}");
  DefMacro!("\\NT",    "\\textit{Nanotechnology}");
  DefMacro!("\\PMB",   "\\textit{Phys. Med. Biol.}");
  DefMacro!("\\PPCF",  "\\textit{Plasma Phys. Control. Fusion\\/}");
  DefMacro!("\\RPP",   "\\textit{Rep. Prog. Phys.}");
  DefMacro!("\\SST",   "\\textit{Semicond. Sci. Technol.}");
  DefMacro!("\\SUST",  "\\textit{Supercond. Sci. Technol.}");
  DefMacro!("\\AP",    "\\textit{Ann. Phys., Lpz.}");
  DefMacro!("\\APNY",  "\\textit{Ann. Phys., NY\\/}");
  DefMacro!("\\JAP",   "\\textit{J. Appl. Phys.}");
  DefMacro!("\\JCP",   "\\textit{J. Chem. Phys.}");
  DefMacro!("\\JMP",   "\\textit{J. Math. Phys.}");
  DefMacro!("\\JOSA",  "\\textit{J. Opt. Soc. Am.}");
  DefMacro!("\\NP",    "\\textit{Nucl. Phys.}");
  DefMacro!("\\PL",    "\\textit{Phys. Lett.}");
  DefMacro!("\\PR",    "\\textit{Phys. Rev.}");
  DefMacro!("\\PRL",   "\\textit{Phys. Rev. Lett.}");
  DefMacro!("\\PRS",   "\\textit{Proc. R. Soc.}");
  DefMacro!("\\PS",    "\\textit{Phys. Scr.}");
  DefMacro!("\\RMP",   "\\textit{Rev. Mod. Phys.}");
  DefMacro!("\\RSI",   "\\textit{Rev. Sci. Instrum.}");
  DefMacro!("\\ZP",    "\\textit{ Z. Phys.}");
  DefMacro!("\\JNE",   "\\textit{J. Neural Eng.}");
  DefMacro!("\\SMS",   "\\textit{Smart Mater. Struct.}");

  // Mystery items — Perl L335-343
  DefMacro!("\\tqs", "\\hspace*{25pt}");
  DefMacro!("\\nosections", "");
  DefMacro!("\\indented", "\\itemize");
  DefMacro!("\\endindented", "\\enditemize");
  DefMacro!("\\varindent", "\\itemize");
  DefMacro!("\\endvarindent", "\\enditemize");
  DefMacro!("\\nonum", "\\par");
});

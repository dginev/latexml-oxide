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
  DefMacro!("\\title[]{}",
    "\\ifx.#1.\\else\\@add@frontmatter{ltx:toctitle}{#1}\\fi\\@add@frontmatter{ltx:title}{#2}");
  Let!("\\paper", "\\title");
  DefMacro!("\\@articletype", "");
  DefMacro!("\\article[]{}{}",
    "\\ifx.#1.\\else\\@add@frontmatter{ltx:toctitle}{#1}\\fi\\ifx.#2.\\else\\@add@frontmatter{ltx:classification}[scheme=type]{#2}\\fi\\@add@frontmatter{ltx:title}{#3}");
  DefMacro!("\\letter{}", "\\article[Letter to the Editor]{Letter to the Editor}{#1}\\lettertrue");
  DefMacro!("\\review[]{}", "\\article[#1]{Review Article}{#2}");
  DefMacro!("\\topical[]{}", "\\article[#1]{Topical Review}{#2}");
  DefMacro!("\\comment[]{}", "\\article[#1]{Comment}{#2}");
  DefMacro!("\\rapid[]{}", "\\article[#1]{Rapid Communication}{#2}");
  DefMacro!("\\note[]{}", "\\article[#1]{Note}{#2}");
  DefMacro!("\\prelim[]{}", "\\article[#1]{Preliminary Communication}{#2}");

  // Equation numbering — Perl L29
  DefMacro!("\\theequation", "\\ifnumbysec\\arabic{section}.\\arabic{equation}\\else\\arabic{equation}\\fi");

  // Authors — Perl L52-57
  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>", bounded => true);
  DefMacro!("\\address{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#1}}");
  DefMacro!("\\ead Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");

  // Dates — Perl L82-86
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\revised{}", "\\@add@frontmatter{ltx:date}[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");
  DefMacro!("\\published{}", "\\@add@frontmatter{ltx:date}[role=published]{#1}");
  DefMacro!("\\online{}", "\\@add@frontmatter{ltx:date}[role=online]{#1}");

  // Contact — Perl L57-59
  Let!("\\mailto", "\\ead");
  DefMacro!("\\eads{}", "#1");

  // Classification — Perl L61-63
  DefMacro!("\\pacno{}", "\\@add@frontmatter{ltx:classification}[scheme=pacs]{#1}");
  DefMacro!("\\pacs{}", "\\@add@frontmatter{ltx:classification}[scheme=pacs]{#1}");
  DefMacro!("\\ams{}", "\\@add@frontmatter{ltx:classification}[scheme=ams]{#1}");

  // Journal — Perl L99-104
  DefMacro!("\\journal", "Institute of Physics Publishing");
  DefMacro!("\\submitted", "\\submitto{\\journal}");
  DefMacro!("\\submitto{}", "\\def\\journal{#1}\\@add@to@frontmatter{ltx:note}[role=submitted]{#1}");

  // Abstract/Keywords — Perl L95-120
  DefMacro!("\\nosections", "");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // Acknowledgements — Perl L249-251 (DefConstructor, defined below)

  // Misc — Perl L130-180
  DefMacro!("\\ft{}", "\\footnote{#1}");
  DefMacro!("\\query{}", "");
  DefMacro!("\\eqalign{}", "\\begin{aligned}#1\\end{aligned}");
  DefMacro!("\\eqalignno{}", "\\begin{aligned}#1\\end{aligned}");
  DefMacro!("\\cases{}", "\\begin{cases}#1\\end{cases}");
  DefMacro!("\\pmatrix{}", "\\begin{pmatrix}#1\\end{pmatrix}");
  // Perl L136: \buildrel{} \over{} — uses literal \over delimiter between args
  // Use RawTeX \def because the Rust DefMacro parser doesn't support CS delimiters
  RawTeX!("\\def\\buildrel#1\\over#2{\\mathrel{\\mathop{#2}\\limits^{#1}}}");
  DefMacro!("\\overmark{}", "");
  DefMacro!("\\fl", "");
  DefMacro!("\\bi{}", "\\boldsymbol{#1}");
  DefMacro!("\\bbox{}", "\\mathbf{#1}");

  // Spacing — Perl L223-227
  Let!("\\ms", "\\,");
  Let!("\\bs", "\\:");
  Let!("\\ns", "\\!");
  Let!("\\es", "\\:");
  Let!("\\psemicolon", "\\;");

  // Perl L229
  DefMacro!("\\mat{}", "\\underline{\\underline{#1}}");

  // Table/figure formatting — Perl L195-220
  DefMacro!("\\lineup", "\\def\\0{\\hbox{\\phantom{\\footnotesize\\rm 0}}}\\def\\m{\\hbox{\\phantom{-}}}");
  DefMacro!("\\centre{}{}", "\\multispan{#1}{\\hfill #2\\hfill}");
  DefMacro!("\\crule{}", "\\multispan{#1}{\\hspace*{\\tabcolsep}\\hrulefill\\hspace*{\\tabcolsep}}");
  DefMacro!("\\fcrule{}", "\\multispan{#1}{\\hrulefill}");

  // Table/Figure environments — Perl L204-209
  DefMacro!("\\Table{}", "\\begin{table}\\caption{#1}\\begin{tabular}{@{}l*{15}{l}}");
  DefMacro!("\\endTable", "\\end{tabular}\\end{table}");
  Let!("\\endtab", "\\endTable");
  DefMacro!("\\fulltable{}", "\\begin{table}\\caption{#1}\\begin{tabular}{@{}l*{15}{l}}");
  DefMacro!("\\endfulltable", "\\end{tabular}\\end{table}");

  DefMacro!("\\boldarrayrulewidth", "1pt");
  Let!("\\bhline", "\\hline");
  DefMacro!("\\br", "\\hline");
  DefMacro!("\\mr", "\\hline");
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

  // Perl L136
  RawTeX!("\\def\\pt(#1){({\\it #1\\/})}");

  // Perl L139-143
  DefMacro!("\\lo{}", "\\llap{${}#1{}$}");
  DefMacro!("\\eql", "\\llap{${}={}$}");
  DefMacro!("\\lsim", "\\llap{${}\\sim{}$}");
  DefMacro!("\\lsimeq", "\\llap{${}\\simeq{}$}");
  DefMacro!("\\lequiv", "\\llap{${}\\equiv{}$}");

  // Perl L152-158
  DefMacro!("\\dotted", "\\ensuremath{{\\mathinner{\\cdotp\\cdotp\\cdotp\\cdotp\\cdotp\\cdotp}}}");
  DefMacro!("\\dashed", "{\\protect\\mbox{-\\; -\\; -\\; -}}");
  DefMacro!("\\broken", "{\\protect\\mbox{-- -- --}}");
  DefMacro!("\\longbroken", "{\\protect\\mbox{--- --- ---}}");
  DefMacro!("\\chain", "{\\protect\\mbox{--- $\\cdot$ ---}}");
  DefMacro!("\\dashddot", "{\\protect\\mbox{--- $\\cdot$ $\\cdot$ ---}}");
  DefMacro!("\\full", "{\\protect\\mbox{------}}");

  // Perl L170-181
  DefMacro!("\\eqnalign{}",
    "\\@eqnarray@bindings\\@@eqnarray\\@equationgroup@numbering{numbered=1,stepped=post,grouped=1,aligned=1}\\lx@begin@alignment#1\\lx@end@alignment\\end@eqnarray");
  DefMacro!("\\eqnalignno{}",
    "\\@eqnarray@bindings\\@@eqnarray\\@equationgroup@numbering{numbered=1,stepped=post,grouped=1,aligned=1}\\lx@begin@alignment#1\\lx@end@alignment\\end@eqnarray");

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

  // Tables — Perl L198-210
  DefMacro!("\\noappendix", "\\setcounter{figure}{0}\\setcounter{table}{0}\\def\\thetable{\\arabic{table}}\\def\\thefigure{\\arabic{figure}}");
  DefMacro!("\\Tables", "\\section*{Tables and table captions}\\noappendix");
  DefMacro!("\\Figures", "\\section*{Figure captions}\\noappendix");
  DefMacro!("\\Figure{}", "\\begin{figure}\\caption{#1}\\end{figure}");

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
  // Journal abbreviations — ALL from Perl L258-331
  DefMacro!("\\CQG",   "\\textit{Class. Quantum Grav.}");
  DefMacro!("\\CTM",   "\\textit{Combust. Theory Modelling\\/}");
  DefMacro!("\\DSE",   "\\textit{Distrib. Syst. Engng\\/}");
  DefMacro!("\\EJP",   "\\textit{Eur. J. Phys.}");
  DefMacro!("\\HPP",   "\\textit{High Perform. Polym.}");
  DefMacro!("\\IP",    "\\textit{Inverse Problems\\/}");
  DefMacro!("\\JHM",   "\\textit{J. Hard Mater.}");
  DefMacro!("\\JO",    "\\textit{J. Opt.}");
  DefMacro!("\\JOA",   "\\textit{J. Opt. A: Pure Appl. Opt.}");
  DefMacro!("\\JOB",   "\\textit{J. Opt. B: Quantum Semiclass. Opt.}");
  DefMacro!("\\JPA",   "\\textit{J. Phys. A: Math. Gen.}");
  DefMacro!("\\JPB",   "\\textit{J. Phys. B: At. Mol. Phys.}");
  DefMacro!("\\jpb",   "\\textit{J. Phys. B: At. Mol. Opt. Phys.}");
  DefMacro!("\\JPC",   "\\textit{J. Phys. C: Solid State Phys.}");
  DefMacro!("\\JPCM",  "\\textit{J. Phys.: Condens. Matter\\/}");
  DefMacro!("\\JPD",   "\\textit{J. Phys. D: Appl. Phys.}");
  DefMacro!("\\JPE",   "\\textit{J. Phys. E: Sci. Instrum.}");
  DefMacro!("\\JPF",   "\\textit{J. Phys. F: Met. Phys.}");
  DefMacro!("\\JPG",   "\\textit{J. Phys. G: Nucl. Phys.}");
  DefMacro!("\\jpg",   "\\textit{J. Phys. G: Nucl. Part. Phys.}");
  DefMacro!("\\MSMSE", "\\textit{Modelling Simulation Mater. Sci. Eng.}");
  DefMacro!("\\MST",   "\\textit{Meas. Sci. Technol.}");
  DefMacro!("\\NET",   "\\textit{Network: Comput. Neural Syst.}");
  DefMacro!("\\NJP",   "\\textit{New J. Phys.}");
  DefMacro!("\\NL",    "\\textit{Nonlinearity\\/}");
  DefMacro!("\\NT",    "\\textit{Nanotechnology}");
  DefMacro!("\\PAO",   "\\textit{Pure Appl. Optics\\/}");
  DefMacro!("\\PM",    "\\textit{Physiol. Meas.}");
  DefMacro!("\\PMB",   "\\textit{Phys. Med. Biol.}");
  DefMacro!("\\PPCF",  "\\textit{Plasma Phys. Control. Fusion\\/}");
  DefMacro!("\\PSST",  "\\textit{Plasma Sources Sci. Technol.}");
  DefMacro!("\\PUS",   "\\textit{Public Understand. Sci.}");
  DefMacro!("\\QO",    "\\textit{Quantum Opt.}");
  DefMacro!("\\QSO",   "\\textit{Quantum Semiclass. Opt.}");
  DefMacro!("\\RPP",   "\\textit{Rep. Prog. Phys.}");
  DefMacro!("\\SLC",   "\\textit{Sov. Lightwave Commun.}");
  DefMacro!("\\SST",   "\\textit{Semicond. Sci. Technol.}");
  DefMacro!("\\SUST",  "\\textit{Supercond. Sci. Technol.}");
  DefMacro!("\\WRM",   "\\textit{Waves Random Media\\/}");
  DefMacro!("\\JMM",   "\\textit{J. of Michromech. and Microeng.\\/}");
  DefMacro!("\\AC",    "\\textit{Acta Crystallogr.}");
  DefMacro!("\\AM",    "\\textit{Acta Metall.}");
  DefMacro!("\\AP",    "\\textit{Ann. Phys., Lpz.}");
  DefMacro!("\\APNY",  "\\textit{Ann. Phys., NY\\/}");
  DefMacro!("\\APP",   "\\textit{Ann. Phys., Paris\\/}");
  DefMacro!("\\CJP",   "\\textit{Can. J. Phys.}");
  DefMacro!("\\JAP",   "\\textit{J. Appl. Phys.}");
  DefMacro!("\\JCP",   "\\textit{J. Chem. Phys.}");
  DefMacro!("\\JJAP",  "\\textit{Japan. J. Appl. Phys.}");
  DefMacro!("\\JP",    "\\textit{J. Physique\\/}");
  DefMacro!("\\JPhCh", "\\textit{J. Phys. Chem.}");
  DefMacro!("\\JMMM",  "\\textit{J. Magn. Magn. Mater.}");
  DefMacro!("\\JMP",   "\\textit{J. Math. Phys.}");
  DefMacro!("\\JOSA",  "\\textit{J. Opt. Soc. Am.}");
  DefMacro!("\\JPSJ",  "\\textit{J. Phys. Soc. Japan\\/}");
  DefMacro!("\\JQSRT", "\\textit{J. Quant. Spectrosc. Radiat. Transfer\\/}");
  DefMacro!("\\NC",    "\\textit{Nuovo Cimento\\/}");
  DefMacro!("\\NIM",   "\\textit{Nucl. Instrum. Methods\\/}");
  DefMacro!("\\NP",    "\\textit{Nucl. Phys.}");
  DefMacro!("\\PL",    "\\textit{Phys. Lett.}");
  DefMacro!("\\PR",    "\\textit{Phys. Rev.}");
  DefMacro!("\\PRL",   "\\textit{Phys. Rev. Lett.}");
  DefMacro!("\\PRS",   "\\textit{Proc. R. Soc.}");
  DefMacro!("\\PS",    "\\textit{Phys. Scr.}");
  DefMacro!("\\PSS",   "\\textit{Phys. Status Solidi\\/}");
  DefMacro!("\\PTRS",  "\\textit{Phil. Trans. R. Soc.}");
  DefMacro!("\\RMP",   "\\textit{Rev. Mod. Phys.}");
  DefMacro!("\\RSI",   "\\textit{Rev. Sci. Instrum.}");
  DefMacro!("\\SSC",   "\\textit{Solid State Commun.}");
  DefMacro!("\\ZP",    "\\textit{ Z. Phys.}");
  DefMacro!("\\JNE",   "\\textit{J. Neural Eng.}");
  DefMacro!("\\PB",    "\\textit{Phys. Biol.}");
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

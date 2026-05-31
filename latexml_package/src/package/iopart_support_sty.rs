//! iopart_support.sty — IOP Publishing journal support
//! Perl: iopart_support.sty.ltxml — 345 lines
//! Used by Journal of Physics, Classical and Quantum Gravity, etc.
use crate::prelude::*;

/// DEP-NEW (2026-05-19): data-drive helper for the 74 IOP-physics
/// journal abbreviation macros that all expand to
/// `\textit{<name>}` (some with trailing italic-correction `\/`).
/// Replacing the per-call `DefMacro!` macro arms with this runtime
/// helper collapses the repeated macro expansion to one inlined fn.
/// Per [[wisdom_data_drive_min_call_sites]]: 74 ≫ 5 threshold.
fn def_iop_journal(cs: &str, name: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(cs, true)?;
  let body_toks = mouth::tokenize_internal(&format!("\\textit{{{name}}}"));
  def_macro(cs_tok, params, ExpansionBody::Tokens(body_toks), None)?;
  Ok(())
}

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
  def_macro_noop("\\@articletype")?;
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

  // NOTE: NO `\received`/`\revised`/`\accepted`/`\published`/`\online` date
  // macros here. The prior port added them citing "Perl L82-86", but those Perl
  // lines are the journal-NAME list, not date macros — Perl's
  // iopart_support.sty.ltxml defines none of these (and the real iopart.cls does
  // not either; `\received{…}` errors "undefined" in Perl). Defining them was a
  // non-faithful Rust-only addition that HIJACKED author redefinitions: a paper
  // that does `\newcommand{\revised}[1]{\textcolor{black}{#1}}` (a common
  // draft-markup macro) and then `\revised{…long body…}` had its `\newcommand`
  // silently fail against our pre-defined `\revised`, routing whole paragraphs
  // (and their display math) into `ltx:date` → "ltx:equation isn't allowed in
  // <ltx:date>". Leaving them undefined matches Perl: the `\newcommand` succeeds
  // and the body stays in the body. Witnesses 1608.01416, 1705.08023.

  // Contact — Perl L57-59
  Let!("\\mailto", "\\ead");
  DefMacro!("\\eads{}", "#1");

  // Classification — Perl L61-63
  DefMacro!("\\pacno{}", "\\@add@frontmatter{ltx:classification}[scheme=pacs]{#1}");
  DefMacro!("\\pacs{}", "\\@add@frontmatter{ltx:classification}[scheme=pacs]{#1}");
  DefMacro!("\\ams{}", "\\@add@frontmatter{ltx:classification}[scheme=ams]{#1}");

  // Journal — Perl L65-104
  static IOP_JOURNALS: &[&str] = &[
    "Institute of Physics Publishing",
    "J. Phys.\\ A: Math.\\ Gen.\\ ",
    "J. Phys.\\ B: At.\\ Mol.\\ Opt.\\ Phys.\\ ",
    "J. Phys.:\\ Condens. Matter\\ ",
    "J. Phys.\\ G: Nucl.\\ Part.\\ Phys.\\ ",
    "Inverse Problems\\ ",
    "Class. Quantum Grav.\\ ",
    "Network: Comput.\\ Neural Syst.\\ ",
    "Nonlinearity\\ ",
    "J. Opt. B: Quantum Semiclass. Opt.\\ ",
    "Waves Random Media\\ ",
    "J. Opt. A: Pure Appl. Opt.\\ ",
    "Phys. Med. Biol.\\ ",
    "Modelling Simul.\\ Mater.\\ Sci.\\ Eng.\\ ",
    "Plasma Phys. Control. Fusion\\ ",
    "Physiol. Meas.\\ ",
    "Combust. Theory Modelling\\ ",
    "High Perform.\\ Polym.\\ ",
    "Public Understand. Sci.\\ ",
    "Rep.\\ Prog.\\ Phys.\\ ",
    "J.\\ Phys.\\ D: Appl.\\ Phys.\\ ",
    "Supercond.\\ Sci.\\ Technol.\\ ",
    "Semicond.\\ Sci.\\ Technol.\\ ",
    "Nanotechnology\\ ",
    "Measur.\\ Sci.\\ Technol.\\ ",
    "Plasma.\\ Sources\\ Sci.\\ Technol.\\ ",
    "Smart\\ Mater.\\ Struct.\\ ",
    "J.\\ Micromech.\\ Microeng.\\ ",
    "Distrib.\\ Syst.\\ Engng\\ ",
    "Bioimaging\\ ",
    "J.\\ Radiol. Prot.\\ ",
    "Europ. J. Phys.\\ ",
    "J. Opt. A: Pure Appl. Opt.\\ ",
    "New. J. Phys.\\ ",
  ];
  DefMacro!("\\journal", "Institute of Physics Publishing");
  DefMacro!("\\submitted", "\\submitto{\\journal}");
  DefMacro!("\\submitto{}", "\\def\\journal{#1}\\@add@to@frontmatter{ltx:note}[role=submitted]{#1}");

  // Perl L102-104: \jl{n} — sets \journal to journals[n]
  DefPrimitive!("\\jl{}", sub[(n)] {
    let idx: usize = n.to_string().trim().parse().unwrap_or(0);
    if let Some(journal) = IOP_JOURNALS.get(idx) {
      def_macro(T_CS!("\\journal"), None, Tokenize!(journal), None)?;
    }
  });

  // Abstract/Keywords — Perl L95-120
  def_macro_noop("\\nosections")?;
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // Acknowledgements — Perl L249-251 (DefConstructor, defined below)

  // Misc — Perl L121,136
  // Note: \eqalign/\eqalignno/\cases/\pmatrix come from Plain TeX (plain_constructs);
  // \ft, \query, \bbox, \overmark are not in Perl iopart_support.sty.ltxml.
  // Perl L121: \fl expands to nothing
  def_macro_noop("\\fl")?;
  // Perl L136: \buildrel{} \over{} — uses literal \over delimiter between args
  // Use RawTeX \def because the Rust DefMacro parser doesn't support CS delimiters
  RawTeX!("\\def\\buildrel#1\\over#2{\\mathrel{\\mathop{#2}\\limits^{#1}}}");
  // Perl L122: \case (not \cases) — two-arg textstyle fraction
  DefMacro!("\\case{}{}", "{\\textstyle\\frac{#1}{#2}}");

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

  // NOTE: previously had speculative DefMacro!("\\la", "\\lesssim"),
  // \\ga, \\sun, \\degr, \\arcmin, \\arcsec — none of these exist in
  // Perl's iopart_support.sty.ltxml (verified). The `\la → \lesssim`
  // entry actively HARMED user macros: papers commonly do
  // `\newcommand\la{\langle}` (e.g. hep-ph0404036), but the prior
  // pre-binding made `\la` already-defined so `\newcommand` ignored
  // the redefinition, and the user's `\la n_G\ra` then expanded into
  // the undefined `\lesssim`. The whole block was Rust-only divergence
  // contradicting the "Perl is ground truth" rule.

  // Math operators — Perl L110-134
  DefMath!("\\rmd", "\u{2146}", role => "DIFFOP", meaning => "differential-d");
  DefMath!("\\rme", "\u{2147}", role => "ID", meaning => "exponential-e");
  DefMath!("\\rmi", "\u{2148}", role => "ID", meaning => "imaginary-i");
  Let!("\\e", "\\rme");
  DefMath!("\\Tr", "\\mathrm{Tr}", role => "OPFUNCTION", meaning => "trace");
  DefMath!("\\tr", "\\mathrm{tr}", role => "OPFUNCTION", meaning => "trace");
  DefMath!("\\Or", "\\mathrm{O}", role => "OPFUNCTION", meaning => "Big-O");
  // Perl L127-129: triple-dot overaccent + shade delimiters
  DefMath!("\\tdot {}", "\u{2026}", operator_role => "OVERACCENT");
  DefMath!("\\lshad", "\u{27E6}", role => "OPEN");
  DefMath!("\\rshad", "\u{27E7}", role => "CLOSE");
  // Perl L114-116: \bcal calligraphic bold primitive
  DefPrimitive!("\\bcal", "",
    font => { family => "caligraphic", series => "bold", shape => "upright", forcebold => true });
  // Perl iopart_support.sty.ltxml L117-119: \bi upright bold italic font
  // (math-mode analogue of \mathbf that keeps the italic family). arxiv
  // papers using iopart / iopart-num call this for vectors.
  DefPrimitive!("\\bi", "",
    font => { family => "italic", series => "bold", shape => "upright", forcebold => true });
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
  def_iop_journal("\\etal", "et al\\/")?;
  DefMacro!("\\dash", "-----");
  // Journal abbreviations — ALL from Perl L258-331
  def_iop_journal("\\CQG", "Class. Quantum Grav.")?;
  def_iop_journal("\\CTM", "Combust. Theory Modelling\\/")?;
  def_iop_journal("\\DSE", "Distrib. Syst. Engng\\/")?;
  def_iop_journal("\\EJP", "Eur. J. Phys.")?;
  def_iop_journal("\\HPP", "High Perform. Polym.")?;
  def_iop_journal("\\IP", "Inverse Problems\\/")?;
  def_iop_journal("\\JHM", "J. Hard Mater.")?;
  def_iop_journal("\\JO", "J. Opt.")?;
  def_iop_journal("\\JOA", "J. Opt. A: Pure Appl. Opt.")?;
  def_iop_journal("\\JOB", "J. Opt. B: Quantum Semiclass. Opt.")?;
  def_iop_journal("\\JPA", "J. Phys. A: Math. Gen.")?;
  def_iop_journal("\\JPB", "J. Phys. B: At. Mol. Phys.")?;
  def_iop_journal("\\jpb", "J. Phys. B: At. Mol. Opt. Phys.")?;
  def_iop_journal("\\JPC", "J. Phys. C: Solid State Phys.")?;
  def_iop_journal("\\JPCM", "J. Phys.: Condens. Matter\\/")?;
  def_iop_journal("\\JPD", "J. Phys. D: Appl. Phys.")?;
  def_iop_journal("\\JPE", "J. Phys. E: Sci. Instrum.")?;
  def_iop_journal("\\JPF", "J. Phys. F: Met. Phys.")?;
  def_iop_journal("\\JPG", "J. Phys. G: Nucl. Phys.")?;
  def_iop_journal("\\jpg", "J. Phys. G: Nucl. Part. Phys.")?;
  def_iop_journal("\\MSMSE", "Modelling Simulation Mater. Sci. Eng.")?;
  def_iop_journal("\\MST", "Meas. Sci. Technol.")?;
  def_iop_journal("\\NET", "Network: Comput. Neural Syst.")?;
  def_iop_journal("\\NJP", "New J. Phys.")?;
  def_iop_journal("\\NL", "Nonlinearity\\/")?;
  def_iop_journal("\\NT", "Nanotechnology")?;
  def_iop_journal("\\PAO", "Pure Appl. Optics\\/")?;
  def_iop_journal("\\PM", "Physiol. Meas.")?;
  def_iop_journal("\\PMB", "Phys. Med. Biol.")?;
  def_iop_journal("\\PPCF", "Plasma Phys. Control. Fusion\\/")?;
  def_iop_journal("\\PSST", "Plasma Sources Sci. Technol.")?;
  def_iop_journal("\\PUS", "Public Understand. Sci.")?;
  def_iop_journal("\\QO", "Quantum Opt.")?;
  def_iop_journal("\\QSO", "Quantum Semiclass. Opt.")?;
  def_iop_journal("\\RPP", "Rep. Prog. Phys.")?;
  def_iop_journal("\\SLC", "Sov. Lightwave Commun.")?;
  def_iop_journal("\\SST", "Semicond. Sci. Technol.")?;
  def_iop_journal("\\SUST", "Supercond. Sci. Technol.")?;
  def_iop_journal("\\WRM", "Waves Random Media\\/")?;
  def_iop_journal("\\JMM", "J. of Michromech. and Microeng.\\/")?;
  def_iop_journal("\\AC", "Acta Crystallogr.")?;
  def_iop_journal("\\AM", "Acta Metall.")?;
  def_iop_journal("\\AP", "Ann. Phys., Lpz.")?;
  def_iop_journal("\\APNY", "Ann. Phys., NY\\/")?;
  def_iop_journal("\\APP", "Ann. Phys., Paris\\/")?;
  def_iop_journal("\\CJP", "Can. J. Phys.")?;
  def_iop_journal("\\JAP", "J. Appl. Phys.")?;
  def_iop_journal("\\JCP", "J. Chem. Phys.")?;
  def_iop_journal("\\JJAP", "Japan. J. Appl. Phys.")?;
  def_iop_journal("\\JP", "J. Physique\\/")?;
  def_iop_journal("\\JPhCh", "J. Phys. Chem.")?;
  def_iop_journal("\\JMMM", "J. Magn. Magn. Mater.")?;
  def_iop_journal("\\JMP", "J. Math. Phys.")?;
  def_iop_journal("\\JOSA", "J. Opt. Soc. Am.")?;
  def_iop_journal("\\JPSJ", "J. Phys. Soc. Japan\\/")?;
  def_iop_journal("\\JQSRT", "J. Quant. Spectrosc. Radiat. Transfer\\/")?;
  def_iop_journal("\\NC", "Nuovo Cimento\\/")?;
  def_iop_journal("\\NIM", "Nucl. Instrum. Methods\\/")?;
  def_iop_journal("\\NP", "Nucl. Phys.")?;
  def_iop_journal("\\PL", "Phys. Lett.")?;
  def_iop_journal("\\PR", "Phys. Rev.")?;
  def_iop_journal("\\PRL", "Phys. Rev. Lett.")?;
  def_iop_journal("\\PRS", "Proc. R. Soc.")?;
  def_iop_journal("\\PS", "Phys. Scr.")?;
  def_iop_journal("\\PSS", "Phys. Status Solidi\\/")?;
  def_iop_journal("\\PTRS", "Phil. Trans. R. Soc.")?;
  def_iop_journal("\\RMP", "Rev. Mod. Phys.")?;
  def_iop_journal("\\RSI", "Rev. Sci. Instrum.")?;
  def_iop_journal("\\SSC", "Solid State Commun.")?;
  def_iop_journal("\\ZP", " Z. Phys.")?;
  def_iop_journal("\\JNE", "J. Neural Eng.")?;
  def_iop_journal("\\PB", "Phys. Biol.")?;
  def_iop_journal("\\SMS", "Smart Mater. Struct.")?;

  // Mystery items — Perl L335-343
  DefMacro!("\\tqs", "\\hspace*{25pt}");
  def_macro_noop("\\nosections")?;
  DefMacro!("\\indented", "\\itemize");
  DefMacro!("\\endindented", "\\enditemize");
  DefMacro!("\\varindent", "\\itemize");
  DefMacro!("\\endvarindent", "\\enditemize");
  DefMacro!("\\nonum", "\\par");
});

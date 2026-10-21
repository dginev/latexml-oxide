//! iopart_support.sty — IOP Publishing journal support
//! Perl: iopart_support.sty.ltxml — 345 lines
//! Used by Journal of Physics, Classical and Quantum Gravity, etc.
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};

use crate::prelude::*;

/// DEP-NEW (2026-05-19): data-drive helper for the 74 IOP-physics
/// journal abbreviation macros that all expand to
/// `\textit{<name>}` (some with trailing italic-correction `\/`).
/// Replacing the per-call `DefMacro!` macro arms with this runtime
/// helper collapses the repeated macro expansion to one inlined fn.
/// Per [[wisdom_data_drive_min_call_sites]]: 74 ≫ 5 threshold.
/// iopart.cls ends each with a space (`\newcommand{\jpa}{{\it J. Phys. A: Math. Theor.} }`, 2603.00348's
/// :972-1125), so `\jpa \textbf{40}` reads "J. Phys. A: Math. Theor. 40"; Perl's dropped it (KNOWN_PERL_ERRORS #634).
fn def_iop_journal(cs: &str, name: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(cs, true)?;
  let body_toks =
    mouth::tokenize_internal(TeXString::assembled(format!("\\textit{{{name}}}\\space")));
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
  // 68t: a call inside the document's `{titlepage}` also runs a refused printing `\title` there (sect05.rs
  // `titlepage_class_call`, as revtex4_support's: gr-qc9404034).
  // The class's refused `\title` runs first for its side effects, as under the kernel's (sect05.rs `\title`,
  // OXIDIZED_DESIGN #482: the cvpr template's `\thetitle`; no witness of this binding's own).
  DefMacro!("\\title[]{}",
    "\\lx@titlepage@class@call{title}{#1}{#2}{\\lx@replay@class@setter{title}{#1}{#2}\\gdef\\@shorttitle{#1}\\gdef\\@title{#2}\\ifx.#1.\\else\\lx@add@toctitle{#1}\\fi\\lx@add@title{#2}}");
  Let!("\\paper", "\\title");
  def_macro_noop("\\@articletype")?;
  DefMacro!("\\article[]{}{}",
    "\\ifx.#1.\\else\\lx@add@toctitle{#1}\\fi\\ifx.#2.\\else\\lx@add@pubnote[role=type]{#2}\\fi\\lx@add@title{#3}");
  DefMacro!("\\letter{}", "\\article[Letter to the Editor]{Letter to the Editor}{#1}\\lettertrue");
  DefMacro!("\\ftc{}", "\\article[Fast Track Communication]{Fast Track Communication}{#1}");
  DefMacro!("\\review[]{}", "\\article[#1]{Review Article}{#2}");
  DefMacro!("\\topical[]{}", "\\article[#1]{Topical Review}{#2}");
  DefMacro!("\\comment[]{}", "\\article[#1]{Comment}{#2}");
  DefMacro!("\\rapid[]{}", "\\article[#1]{Rapid Communication}{#2}");
  DefMacro!("\\note[]{}", "\\article[#1]{Note}{#2}");
  DefMacro!("\\prelim[]{}", "\\article[#1]{Preliminary Communication}{#2}");

  // Equation numbering — Perl L29
  DefMacro!("\\theequation", "\\ifnumbysec\\arabic{section}.\\arabic{equation}\\else\\arabic{equation}\\fi");
  // iopart.cls:771-775 (1907.10288's copy): `{equation}` is flush left, `\hbox to\linewidth\bgroup $ \displaystyle
  // \hskip\mathindent`, as fleqn.clo's (fleqn_sty.rs `math_displays_in_hbox`): an author's `$` in it is text in the math
  // (1905.11073; KNOWN_PERL_ERRORS #907). Guard `perfect_kernel_batch73b::dollar_in_an_iopart_equation_is_text`.
  fleqn_sty::math_displays_in_hbox()?;

  // Authors (Perl PR #2767)
  // iopart.cls:240-248: "Use once for each address" — each `\address` prints its own block, so each adds its
  // affiliation, linked by its superscript marks when the authors carry them. Perl's `\lx@add@affiliations`
  // (iopart_support.sty.ltxml:53) dequeues the affiliations before it, so only the last `\address` survived
  // (KNOWN_PERL_ERRORS #511; 21 of the 41 2609 iopart papers call it more than once, 2609.01831 lost three of four).
  // (its one leading mark is the mark of the `\ead` blocks printed right after it, `\lx@address@mark@set`)
  DefMacro!("\\address{}", "\\lx@address@mark@set{#1}\\lx@add@affiliation@marked{#1}");
  // iopart.cls:363 `\def\ead#1{\vspace*{5pt}\address{E-mail: \mailto{#1}}}`, `\mailto` = `{\tt #1}` (:364; 2301.12119,
  // 2501.12730): the address prints as an address block of its own right after the one before it, no name's. Perl's
  // `\lx@add@email` gives it to the last author (2311.12551's benedikt.maier@cern.ch on T. K. Årrestad, author 7 of 7;
  // in the 162 census iopart `\ead` papers A/B'd for batch 78f, int47a put 83 addresses, in 81 papers, on an author
  // another name spells). Rust-only (OXIDIZED_DESIGN_DIVERGENCES #806, #808, #809): placed as jheppub's `\emailAdd`
  // is — with the author it spells, else the n-th author's while the addresses are the authors' in order, else with
  // the one author requesting the mark of the `\address` it follows (`mark=address`: 2407.14305's `rockfish3141@` under
  // `$^1$ Waseda`, Hidetomo Hoshino's; 2311.07087 — in a document that writes an `\ead` between its addresses;
  // 1209.0173, 0910.0753, 1111.3344 write them all after), else the group's only author's (one `\author` per call:
  // 2311.17496), else — the one address left on the author block — the ONE author of its group (and of those requesting
  // its address's mark) marked as the corresponding author by a note naming them so or a mark whose legend says so
  // (`$^{1,*}$` under `\address{$^*$ Author to whom any correspondence should be addressed.}`, 2504.15015; two
  // legend-less `$^{*}$`, 2111.00753, mark no one), else the author block's (`\lx@add@contact@placed[owner=sequence]`).
  // An `\ead{a@x, b@y}` or `\ead{a@x and b@y}` list is an address each (2111.00753; 0710.2104, 1003.0073), and a
  // `\mailto` in it is the address font iopart.cls:364 gives it
  // (`\def\mailto#1{{\tt #1}}`: `\ead{\mailto{edvard@physto.se}, \mailto{chris.clarkson@uct.ac.za}}`, 0811.0981,
  // where the `\let\mailto\ead` below made each a contact nested in the first).
  DefMacro!("\\ead Semiverbatim", "\\lx@add@contact@list{\\lx@iop@ead}{#1}");
  DefMacro!("\\lx@iop@ead{}",
    "\\lx@add@contact@placed[role=email,owner=sequence,mark=address]{}{\\lx@address@urlfont{\\let\\mailto\\@firstofone#1}}");
  // iopart.cls:225-231: each `\author[short]{names}` typesets its names where it stands, as `\address` does, so each
  // call adds its authors (split as the article `\author` splits them). Perl has no iopart `\author`, and the kernel's
  // replaces the authors before it (latex_constructs.pool.ltxml:1076), keeping the last call's only
  // (KNOWN_PERL_ERRORS #550; 2311.17496 lost three of four authors). Locked as the kernel's is; a binding loaded later
  // (authblk's `\author[mark]`) still replaces it.
  // Each call starts an author group (`\lx@frontmatter@group@start`): an `\ead` placed by none of its names' spellings
  // or marks is the group's only author's (`\author{A. Bal and C. Cole}\address{KIT}\ead{…}\author{B. Maier}
  // \address{ICL}\ead{yy2@…}`: yy2@ B. Maier's), and the `\ead`s after it are under no address of before.
  DefMacro!("\\author[]{}",
    "\\lx@frontmatter@group@start\\lx@address@mark@set{}\\def\\@shortauthor{#1}\\def\\@author{#2}\\lx@add@authors@list{#2}",
    locked => true);

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

  // Classification
  DefMacro!("\\pacno{}", "\\lx@add@classification[scheme=pacs,name={PACS:~}]{#1}");
  DefMacro!("\\pacs{}", "\\lx@add@classification[scheme=pacs,name={PACS:~}]{#1}");
  DefMacro!("\\ams{}", "\\lx@add@classification[scheme=ams,name={AMS Classification:~}]{#1}");

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
  DefMacro!("\\submitto{}", "\\def\\journal{#1}\\lx@add@pubnote[role=journal]{#1}");

  // Perl L102-104: \jl{n} — sets \journal to journals[n]
  DefPrimitive!("\\jl{}", sub[(n)] {
    let idx: usize = n.to_string().trim().parse().unwrap_or(0);
    if let Some(journal) = IOP_JOURNALS.get(idx) {
      def_macro(T_CS!("\\journal"), None, Tokenize!(*journal), None)?;
    }
  });

  // Abstract/Keywords — Perl L95-120
  def_macro_noop("\\nosections")?;
  DefMacro!("\\keywords{}", "\\lx@add@frontmatter{ltx:keywords}{#1}");

  // Acknowledgements — Perl L249-251 (DefConstructor, defined below)

  // Misc — Perl L121,136
  // iopart.cls:807-809 `\def\eqalign#1{\null\vcenter{\def\\{\cr}…\ialign{…\crcr#1\crcr}}\,}`: plain's alignment
  // (plain_constructs), where `\\` ends the row (2402.10275).
  DefMacro!(
    "\\eqalign{}",
    r"\@@eqalign{\def\\{\cr}\lx@begin@alignment#1\lx@end@alignment}"
  );
  // Note: \eqalignno/\cases/\pmatrix come from Plain TeX (plain_constructs);
  // \ft, \query, \bbox, \overmark are not in Perl iopart_support.sty.ltxml.
  // Perl L121: \fl expands to nothing
  def_macro_noop("\\fl")?;
  // Perl L136: \buildrel{} \over{} — uses literal \over delimiter between args
  // Use RawTeX \def because the Rust DefMacro parser doesn't support CS delimiters
  RawTeX!("\\def\\buildrel#1\\over#2{\\mathrel{\\mathop{#2}\\limits^{#1}}}");
  // Perl L122: \case (not \cases) — two-arg textstyle fraction
  DefMacro!("\\case{}{}", "{\\textstyle\\frac{#1}{#2}}");

  // Spacing — Perl L223-227. iopart.cls :935-938 (1502.00309's copy): `\ms \bs \ns \es` are the extra spaces between
  // table and display rows, `\noalign{\vspace{…}}` (`\es` then `\displaystyle` for the next row), so `a & b \\\ns`
  // then `\mr` is two `\noalign`s at a row start. Perl's `\, \: \! \:` started a row there and misplaced the rule
  // (1502.00309; KNOWN_PERL_ERRORS #878). Guard: `perfect_kernel_batch68y::iopart_row_spacers_are_noalign`.
  DefMacro!("\\ms", r"\noalign{\vspace{3pt plus2pt minus1pt}}");
  DefMacro!("\\bs", r"\noalign{\vspace{6pt plus2pt minus2pt}}");
  DefMacro!("\\ns", r"\noalign{\vspace{-3pt plus-1pt minus-1pt}}");
  DefMacro!("\\es", r"\noalign{\vspace{6pt plus2pt minus2pt}}\displaystyle");
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
  DefPrimitive!("\\opendiamond", "\u{25C7}"); // WHITE DIAMOND (Perl #2845, was 25C6 black)
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
  // ltx:acknowledgements Tag (autoClose + inlist=toc) is global — set in
  // latex_constructs.rs (arXiv-fork 23771504 removed the binding-local copies).

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
  // The journals of the 2011-2014 iopart.cls (2603.00348's copy :1059-1140, "More journals added 8 Mar 2011" and
  // "6 Feb 2014"), which Perl's list predates: `\jpa` (J. Phys. A: Math. Theor.), `\NF`, `\JSTAT`, `\JHEP`, … and
  // `\MLST` (the later copies' form). Older copies lack them, so a paper's own definition replaces them
  // (`DefinitionOrigin::Fallback`). Run 340: `\jpa` 31 of 120 sampled iopart papers, ~200 first errors in all
  // (2603.00348).
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    def_iop_journal("\\GRG", "Gen. Rel. Grav.")?;
    def_iop_journal("\\PF", "Phys. Fluids\\/")?;
    def_iop_journal("\\SPJ", "Sov. Phys.--JETP\\/")?;
    def_iop_journal("\\jpa", "J. Phys. A: Math. Theor.")?;
    def_iop_journal("\\BF", "Biofabrication\\/")?;
    def_iop_journal("\\BB", "Bioinspir. Biomim.")?;
    def_iop_journal("\\BMM", "Biomed. Mater.")?;
    def_iop_journal("\\CSD", "Comput. Sci. Disc.")?;
    def_iop_journal("\\ERL", "Environ. Res. Lett.")?;
    def_iop_journal("\\JBR", "J. Breath Res.")?;
    def_iop_journal("\\JGE", "J. Geophys. Eng.")?;
    def_iop_journal("\\JOPT", "J. Opt.")?;
    def_iop_journal("\\JRP", "J. Radiol. Prot.")?;
    def_iop_journal("\\MET", "Metrologia\\/")?;
    def_iop_journal("\\NF", "Nucl. Fusion\\/")?;
    def_iop_journal("\\PED", "Phys. Educ.")?;
    def_iop_journal("\\TDM", "2D Mater.")?;
    def_iop_journal("\\MRE", "Mater. Res. Express\\/")?;
    def_iop_journal("\\MAF", "Methods Appl. Fluoresc.")?;
    def_iop_journal("\\TMR", "Transl. Mater. Res.")?;
    def_iop_journal("\\STMP", "Surf. Topogr.: Metrol. Prop.")?;
    def_iop_journal("\\AJ", "AJ\\/")?;
    def_iop_journal("\\APJ", "ApJ\\/")?;
    def_iop_journal("\\APJL", "ApJL\\/")?;
    def_iop_journal("\\APJS", "ApJS\\/")?;
    def_iop_journal("\\ANSN", "Adv. Nat. Sci: Nanosci. Nanotechnol.")?;
    def_iop_journal("\\CJCP", "Chin. J. Chem. Phys.")?;
    def_iop_journal("\\CPB", "Chinese Phys. B\\/")?;
    def_iop_journal("\\CPC", "Chinese Phys. C\\/")?;
    def_iop_journal("\\CPL", "Chinese Phys. Lett.")?;
    def_iop_journal("\\CTP", "Commun. Theor. Phys.")?;
    def_iop_journal("\\EPL", "EPL\\/")?;
    def_iop_journal("\\FDR", "Fluid Dyn. Res.")?;
    def_iop_journal("\\IZV", "Izv. Math.")?;
    def_iop_journal("\\JOS", "J. Semicond.")?;
    def_iop_journal("\\PHU", "Phys.-Usp.")?;
    def_iop_journal("\\PST", "Plasma Sci. Technol.")?;
    def_iop_journal("\\QEL", "Quantum Electron.")?;
    def_iop_journal("\\RAA", "Res. Astron. Astrophys.")?;
    def_iop_journal("\\RCR", "Russ. Chem. Rev.")?;
    def_iop_journal("\\RMS", "Russ. Math. Surv.")?;
    def_iop_journal("\\MSB", "Sb. Math.")?;
    def_iop_journal("\\SFC", "Science Foundation in China\\/")?;
    def_iop_journal("\\STAM", "Sci. Technol. Adv. Mater.")?;
    def_iop_journal("\\LP", "Laser Phys.")?;
    def_iop_journal("\\LPL", "Laser Phys. Lett.")?;
    def_iop_journal("\\APEX", "Appl. Phys. Express\\/")?;
    def_iop_journal("\\JCAP", "JCAP\\/")?;
    def_iop_journal("\\JHEP", "J. High Energy Phys.\\/")?;
    def_iop_journal("\\JSTAT", "J. Stat. Mech.")?;
    def_iop_journal("\\JINST", "JINST\\/")?;
    def_iop_journal("\\JPCS", "J. Phys.: Conf. Ser.")?;
    def_iop_journal("\\EES", "IOP Conf. Ser.: Earth Environ. Sci.")?;
    def_iop_journal("\\MSE", "IOP Conf. Ser.: Mater. Sci. Eng.")?;
    // `\MLST` as the later copies (2501.05515's :972, 1907.13401's) define it, with no trailing space.
    DefMacro!("\\MLST", "\\textit{Mach. Learn.: Sci. Technol.}");
  }

  // Mystery items — Perl L335-343
  DefMacro!("\\tqs", "\\hspace*{25pt}");
  def_macro_noop("\\nosections")?;
  DefMacro!("\\indented", "\\itemize");
  DefMacro!("\\endindented", "\\enditemize");
  DefMacro!("\\varindent", "\\itemize");
  DefMacro!("\\endvarindent", "\\enditemize");
  DefMacro!("\\nonum", "\\par");

  // iopart.cls:1095-1097 ("2012 new option for twocolumn output"): the two-column page geometry, then `\twocolumn`.
  // Witnesses 1004.1944, 1704.06471, 2012.05676, 2406.14105, 2212.07640.
  RawTeX!(r"\def\ioptwocol{\setlength\hoffset{-0.5in}\setlength\voffset{-0.5in}\setlength\textwidth{6.75in}
\setlength\columnsep{0.2in}\setlength\textheight{9.25in}\mathindent=0in\twocolumn}");
});

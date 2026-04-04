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

  // Cross-referencing — Perl L285-345
  DefMacro!("\\eref{}", "\\ref{#1}");
  DefMacro!("\\Eref{}", "\\ref{#1}");
  DefMacro!("\\fref{}", "\\ref{#1}");
  DefMacro!("\\Fref{}", "\\ref{#1}");
  DefMacro!("\\tref{}", "\\ref{#1}");
  DefMacro!("\\Tref{}", "\\ref{#1}");
  DefMacro!("\\sref{}", "\\ref{#1}");
  DefMacro!("\\Sref{}", "\\ref{#1}");
  DefMacro!("\\aref{}", "\\ref{#1}");
  DefMacro!("\\Aref{}", "\\ref{#1}");
});

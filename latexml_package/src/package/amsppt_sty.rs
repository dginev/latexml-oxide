//! amsppt.sty — AMSTeX plain TeX compatibility
//! Perl: amsppt.sty.ltxml — 500 lines
//! Document class for AMSTeX-style plain TeX documents.
//! Provides frontmatter, theorem environments, bibliography.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // amsppt loads the AmSTeX pool — Perl L27
  // AmSTeX pool is partially ported (~30%)

  // Frontmatter — Perl L32-80
  DefMacro!("\\makeheadline", "");
  DefMacro!("\\makefootline", "");
  DefMacro!("\\title", "\\@add@frontmatter{ltx:title}");
  DefMacro!("\\endtitle", "");
  DefMacro!("\\author", "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname}");
  DefMacro!("\\endauthor", "");

  // Affiliations and contacts — Perl L85-130
  DefConstructor!("\\@@@affil{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affil", "\\@add@to@frontmatter{ltx:creator}{\\@@@affil}");
  DefMacro!("\\endaffil", "");
  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\address", "\\@add@to@frontmatter{ltx:creator}{\\@@@address}");
  DefMacro!("\\endaddress", "");
  DefConstructor!("\\@@@curraddr{}", "^ <ltx:contact role='current_address'>#1</ltx:contact>");
  DefMacro!("\\curraddr", "\\@add@to@frontmatter{ltx:creator}{\\@@@curraddr}");
  DefMacro!("\\endcurraddr", "");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email", "\\@add@to@frontmatter{ltx:creator}{\\@@@email}");
  DefMacro!("\\endemail", "");
  DefConstructor!("\\@@@urladdr{}", "^ <ltx:contact role='url'>#1</ltx:contact>");
  DefMacro!("\\urladdr", "\\@add@to@frontmatter{ltx:creator}{\\@@@urladdr}");
  DefMacro!("\\endurladdr", "");

  // Abstract and classification — Perl L135-165
  DefMacro!("\\abstract", "\\@add@frontmatter{ltx:abstract}");
  DefMacro!("\\endabstract", "");
  DefMacro!("\\keywords", "\\@add@frontmatter{ltx:keywords}");
  DefMacro!("\\endkeywords", "");
  DefMacro!("\\subjclass", "\\@add@frontmatter{ltx:classification}[scheme=MSC]");
  DefMacro!("\\endsubjclass", "");

  // Section structure — Perl L170-200
  DefMacro!("\\heading", "\\section*");
  DefMacro!("\\endheading", "");
  DefMacro!("\\subheading", "\\subsection*");
  DefMacro!("\\endsubheading", "");
  DefMacro!("\\specialhead", "\\section*");
  DefMacro!("\\endspecialhead", "");

  // Theorem environments — Perl L200-260 (use DigestUntil)
  // Stubbed: DigestUntil is not fully ported
  DefMacro!("\\proclaim", "\\begin{theorem}");
  DefMacro!("\\endproclaim", "\\end{theorem}");
  DefMacro!("\\definition", "\\begin{definition}");
  DefMacro!("\\enddefinition", "\\end{definition}");
  DefMacro!("\\remark", "\\begin{remark}");
  DefMacro!("\\endremark", "\\end{remark}");
  DefMacro!("\\example", "\\begin{example}");
  DefMacro!("\\endexample", "\\end{example}");
  DefMacro!("\\demo", "\\begin{proof}");
  DefMacro!("\\enddemo", "\\end{proof}");

  // Lists — Perl L265-300
  DefMacro!("\\roster", "\\begin{enumerate}");
  DefMacro!("\\endroster", "\\end{enumerate}");

  // Perl amsppt.sty.ltxml L261-263: \block — simple block-quote container.
  // Previously unported. DigestUntil parameter type landed in 27cc66b60
  // makes this a direct translation.
  DefConstructor!(
    "\\block DigestUntil:\\endblock",
    "<ltx:quote>#1</ltx:quote>"
  );
  Let!(T_CS!("\\endblock"), T_CS!("\\relax"));

  // Footnotes — Perl L305-350
  DefMacro!("\\footnote", "\\lx@note{footnote}");

  // Bibliography — Perl L355-500
  // Complex Perl closure system for reference formatting
  DefMacro!("\\Refs", "\\begin{thebibliography}{}");
  DefMacro!("\\endRefs", "\\end{thebibliography}");
  DefMacro!("\\ref", "\\bibitem");
  DefMacro!("\\endref", "");
  DefMacro!("\\by", "");
  DefMacro!("\\bysame", "");
  DefMacro!("\\paper", "\\textit");
  DefMacro!("\\paperinfo{}", "#1");
  DefMacro!("\\jour{}", "\\textit{#1}");
  DefMacro!("\\vol{}", "{\\bf #1}");
  DefMacro!("\\yr{}", "(#1)");
  DefMacro!("\\pages{}", "#1");
  DefMacro!("\\page{}", "#1");
  DefMacro!("\\book{}", "\\textit{#1}");
  DefMacro!("\\bookinfo{}", "#1");
  DefMacro!("\\publ{}", "#1");
  DefMacro!("\\publaddr{}", "#1");
  DefMacro!("\\finalinfo{}", "#1");
  DefMacro!("\\eds{}", "(#1, eds.)");
  DefMacro!("\\ed{}", "(#1, ed.)");
  DefMacro!("\\moreref", "");
  DefMacro!("\\lang{}", "[#1]");
  DefMacro!("\\toappear", "(to appear)");
  DefMacro!("\\inpress", "(in press)");
  DefMacro!("\\issue{}", "no. #1");
  DefMacro!("\\miscnote{}", "#1");

  // Miscellaneous — Perl L480-500
  DefMacro!("\\nologo", "");
  DefMacro!("\\NoBlackBoxes", "");
  DefMacro!("\\redefine", "\\def");
  DefMacro!("\\define", "\\def");
});

//! amsppt.sty — AMSTeX plain TeX compatibility
//! Perl: amsppt.sty.ltxml — 500 lines
//! Document class for AMSTeX-style plain TeX documents.
//! Provides frontmatter, theorem environments, bibliography.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // amsppt loads the AmSTeX pool — Perl L27
  // AmSTeX pool is partially ported (~30%)

  // Frontmatter — Perl L32-80. Original (pre-LaTeX) AMSPPT syntax uses
  // `\title Foo \endtitle` (tokens terminated by `\endtitle`, not a
  // `{…}` group). Prior Rust stubbed these as naked DefMacro expanding
  // to `\@add@frontmatter{ltx:X}`, which only works when the user writes
  // `\title{Foo}` (LaTeX-ish form). Switch to the `Until:\endX` delimiter
  // form the Perl port uses, with the `\endX` `Let`-ed to `\relax`.
  DefMacro!("\\makeheadline", "");
  DefMacro!("\\makefootline", "");
  DefMacro!("\\title Until:\\endtitle", "\\@add@frontmatter{ltx:title}{#1}");
  Let!("\\endtitle", "\\relax");
  DefMacro!("\\author Until:\\endauthor",
    "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#1}}");
  Let!("\\endauthor", "\\relax");

  // Affiliations and contacts — Perl L85-130
  DefConstructor!("\\@@@affil{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affil Until:\\endaffil",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@affil{#1}}");
  Let!("\\endaffil", "\\relax");
  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\address Until:\\endaddress",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#1}}");
  Let!("\\endaddress", "\\relax");
  DefConstructor!("\\@@@curraddr{}", "^ <ltx:contact role='current_address'>#1</ltx:contact>");
  DefMacro!("\\curraddr Until:\\endcurraddr",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@curraddr{#1}}");
  Let!("\\endcurraddr", "\\relax");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email Until:\\endemail",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  Let!("\\endemail", "\\relax");
  DefConstructor!("\\@@@urladdr{}", "^ <ltx:contact role='url'>#1</ltx:contact>");
  DefMacro!("\\urladdr Until:\\endurladdr",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@urladdr{#1}}");
  Let!("\\endurladdr", "\\relax");

  // Perl amsppt.sty.ltxml L72-75: thanks/date/dedicatory/translator —
  // previously absent in Rust.
  DefMacro!("\\thanks Until:\\endthanks",
    "\\@add@frontmatter{ltx:note}[role=support]{#1}");
  Let!("\\endthanks", "\\relax");
  DefMacro!("\\date Until:\\enddate",
    "\\@add@frontmatter{ltx:date}[role=creation]{#1}");
  Let!("\\enddate", "\\relax");
  DefMacro!("\\dedicatory Until:\\enddedicatory",
    "\\@add@frontmatter{ltx:note}[role=dedicatory]{#1}");
  Let!("\\enddedicatory", "\\relax");
  DefMacro!("\\translator Until:\\endtranslator",
    "\\@add@frontmatter{ltx:creator}[role=translator]{\\@personname{#1}}");
  Let!("\\endtranslator", "\\relax");

  // Abstract and classification — Perl L76-79.
  DefMacro!("\\keywords Until:\\endkeywords",
    "\\@add@frontmatter{ltx:keywords}{#1}");
  Let!("\\endkeywords", "\\relax");
  DefMacro!("\\subjclass Until:\\endsubjclass",
    "\\@add@frontmatter{ltx:classification}[scheme=MSC]{#1}");
  Let!("\\endsubjclass", "\\relax");
  DefMacro!("\\abstract Until:\\endabstract",
    "\\@add@frontmatter{ltx:abstract}{#1}");
  Let!("\\endabstract", "\\relax");

  // Section structure — Perl L170-200
  DefMacro!("\\heading", "\\section*");
  DefMacro!("\\endheading", "");
  DefMacro!("\\subheading", "\\subsection*");
  DefMacro!("\\endsubheading", "");
  DefMacro!("\\specialhead", "\\section*");
  DefMacro!("\\endspecialhead", "");

  // Theorem environments — Perl L200-260 use
  //   DefConstructor('\<kind> Undigested DigestUntil:\end<kind>', …)
  // each with its own counter, title font, afterConstruct close, and
  // title-name Digest. DigestUntil is now fully ported (27cc66b60);
  // wiring these up to Perl-parity is still deferred because each
  // needs a NewCounter('<kind>') declaration plus the title-font
  // computation — risk of conflict with amsthm's theorem counter.
  // Current stubs forward to the corresponding `theorem`/`definition`/
  // etc. LaTeX environments, which produce valid ltx:theorem output
  // but with a different counter namespace than native amsppt would.
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

  // Identity / version metadata — Perl amsppt.sty.ltxml L29-35.
  DefMacro!("\\filename", "amsppt.sty");
  DefMacro!("\\fileversion", "2.1h");
  DefMacro!("\\filedate", "1997/02/02");
  DefMacro!("\\fileversiontest", "\\fileversion\\space(\\filedate)");
  DefMacro!("\\styname", "AMSPPT");
  DefMacro!("\\styversion", "\\fileversion");
  DefMacro!("\\plainend", "\\end");

  // Page-layout no-ops — Perl L40-52. Running-head tokens + page-contents
  // are TeX plain-format hooks with no LaTeXML analogue; swallow their
  // args.
  DefMacro!("\\leftheadline", "");
  DefMacro!("\\rightheadline", "");
  DefMacro!("\\leftheadtext{}", "");
  DefMacro!("\\rightheadtext{}", "");
  Let!("\\flheadline", "\\hfil");
  Let!("\\frheadline", "\\hfil");
  DefMacro!("\\headmark{}", "");
  DefMacro!("\\pagecontents", "");
  DefMacro!("\\cvolyear{}", "");
  DefMacro!("\\issueinfo{}{}{}{}", "");
  DefMacro!("\\NoRunningHeads", "");
  DefMacro!("\\Monograph", "");

  // Per-field "pre" hooks — Perl L90-95. No-ops; user can `\def` to override.
  Let!("\\pretitle", "\\relax");
  Let!("\\preauthor", "\\relax");
  Let!("\\preaffil", "\\relax");
  Let!("\\predate", "\\relax");
  Let!("\\preabstract", "\\relax");
  Let!("\\prepaper", "\\relax");

  // AMS Fonts + QED — Perl L320-330. `\rom` drops its arg into rm.
  // `\qed` emits `\ltx@qed`, which is isMath-aware (end-of-proof symbol).
  // `\tildechar` is the amsppt literal `~` in typewriter (bibliography
  // key separator).
  DefMacro!("\\rom{}", "{\\rm #1}");
  DefMacro!("\\PSAMSFonts", "");
  RawTeX!("\\newif\\ifPSAMSFonts\\PSAMSFontstrue");
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!(
    "\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    reversion => "\\qed"
  );
  DefMacro!("\\tildechar", "\\texttt{\\textasciitilde}");
  DefMacro!("\\breakcheck", "");
  DefMacro!("\\usualspace", " ");

  // References section — Perl L333, L361-365.
  DefMacro!("\\Refsname", "References");
  DefRegister!("\\refindentwd" => Dimension::new(0));
  DefMacro!("\\refstyle{}", "");
  DefMacro!("\\keyformat{}", "#1");
  DefMacro!("\\refbreaks", "");
  DefMacro!("\\defaultreftexts", "");
});

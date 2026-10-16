use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("elsart_support");
  RequirePackage!("amssymb");
  RequirePackage!("bm");
  RequirePackage!("keyval");
  RequirePackage!("hyperref");
  // The VTeX IMS markup arxbj.cls shares with arximspdf.cls: `{barticle}`/`\b*` (:2599-2897), `{pf}` with `\qed`
  // (:1428-1445), `\tablewidth` (:1684) and `\bolds` (1003.1189, 1203.0186).
  RequirePackage!("ims_support");
  def_macro_noop("\\pdftitle {}")?;
  def_macro_noop("\\pdfauthor {}")?;
  def_macro_noop("\\pdfsubject {}")?;
  def_macro_noop("\\pdfkeywords {}")?;
  def_macro_noop("\\printhistory")?;
  // Motivated by arXiv:1102.2078
  DefMacro!("\\tfrac{}{}", "{\\textstyle\\frac{#1}{#2}}");
  DefMacro!("\\dfrac{}{}", "{\\displaystyle\\frac{#1}{#2}}");
  DefMacro!("\\dvt", "\\colon\\ ");
  DefMacro!(
    "\\dvtx",
    "\\mathchoice{\\nobreak\\,\\colon\\relax}%\n{\\nobreak\\,\\colon\\relax}%\n{\\nobreak\\,\\colon\\;\\relax}%\n{\\nobreak\\,\\colon\\;\\relax}%"
  );
  // `{longlist}` is the class's own list (arxbj.cls:930-958, labels re-set at :1070): items counted `longlist`, labelled
  // "(i)", "(ii)", … with an optional widest-label argument. A bare `\list` (the ar5iv-bindings Perl `\let`) took the
  // first `\item` as its label argument, so every item's label was `\item` again: an endless recursion (1203.0186,
  // 1003.1189; Perl deep_recursion too).
  RawTeX!(
    r"\newcounter{longlist}
\def\thelonglist{\@roman\c@longlist}
\def\labellonglist{(\thelonglist)}
\def\longlist{\@ifnextchar[{\@longlist}{\@longlist[]}}
\def\@longlist[#1]{\list{\labellonglist}{\usecounter{longlist}}}
\let\endlonglist\endlist"
  );
  DefMacro!(
    "\\MR{}",
    "\\href{http://www.ams.org/mathscinet-getitem?mr=#1}{MR#1}"
  );
  // arxbj.cls (1106.2467's copy, line numbers of it) — the class's own commands the papers use, beside the URL and
  // attribute helpers it shares with arximspdf.cls (ims_support_sty.rs: `\arxivurl` 94 papers of the inventory,
  // `\doiurl` 43, `\surl` 17, `\setattribute`, `\tsub`; 1206.6586, 1106.2467, 1303.4890).
  // :2490-2541 `{keyword}[class=…]` with `\kwd[sep]{text}` (ims_support_sty.rs, the VTeX classes' shared markup):
  // arxbj's entries are joined by "; " (:2539 `\sep`), under its headings (:2495-2496).
  RawTeX!(
    r"\def\sep{\unskip; }
\setattribute{keyword}{KWD}{Keywords:}
\setattribute{keyword}{AMS}{AMS 2000 subject classifications:}"
  );
  // :2259-2260 `\def\docsubty#1{}`, the document subtype, which prints nothing (98 papers: 1106.2467, 0811.0643).
  def_macro_noop("\\docsubty{}")?;
  // :1273 `\eqntext{text}`, text set in an `{eqnarray}` row's number place (18 papers: 1002.0175), kept in the row.
  DefMacro!("\\eqntext{}", "#1");
  // :392-395 `\startlocaldefs`/`\endlocaldefs` around a paper's own definitions (:495-502 `\setvaluelist{list}{a,b,…}`
  // and `\getitemvalue{list}{n}`, the class's value lists, 3 papers: 0811.0643, are ims_support's); 0902.2924's copy:
  // `\vtexed` and `\CRC`, production flags; :1795 `\legend{text}`, a table note without a mark (1506.00847).
  RawTeX!(
    r"\def\startlocaldefs{\makeatletter}\def\endlocaldefs{\makeatother}
\def\legend#1{\tabnotetext[]{}{#1}}"
  );
  def_macro_noop("\\vtexed{}")?;
  def_macro_noop("\\CRC")?;
  // :2543-2598 `{supplement}[id=…]`: the supplementary material, under the heading "Supplementary Material" (once),
  // as `\sname`'s label, `\stitle`'s title, the `\slink[doi]`/`\slink[url]` link, `\sdatatype`, `\sdescription` print
  // at its end ("Supplement A: Title (DOI: …; .pdf). Description"; 35 papers: 1106.2467, 1302.0163); `\sfilename`
  // prints nothing (:2562), and `\thesuppdoi{id}` reprints the link.
  RawTeX!(
    r"\def\supplement@name{Supplementary Material}
\def\sname#1{\def\@sname{#1}\def\@currentlabel{#1}}
\def\stitle#1{\def\@stitle{#1}}
\def\sdatatype#1{\def\@sdatatype{#1}}
\def\sfilename#1{}
\define@key{slink}{text}{\def\sdoi@text{#1}}
\define@key{slink}{doi}[true]{\def\slink@type{doi}}
\define@key{slink}{url}[true]{\def\slink@type{url}}
\def\slink[#1]#2{\setkeys{slink}{#1}\@namedef{slink@\slink@type}{#2}}
\def\sdescription#1{\def\@sdescription{#1}}
\def\suppsection@fmt{\section*{\supplement@name}}
\def\lx@arxbj@doibase{http://dx.doi.org/}
\def\slink@doi@fmt{DOI: \href{\lx@arxbj@doibase\slink@doi}{\@ifundefined{sdoi@text}{\slink@doi}{\sdoi@text}}\@ifundefined{supp@label}{}{\expandafter\xdef\csname\supp@label @doi\endcsname{\slink@doi}}}
\def\thesuppdoi#1{\@ifundefined{#1@doi}{}{DOI: \href{\lx@arxbj@doibase\csname #1@doi\endcsname}{\csname #1@doi\endcsname}}}
\define@key{supplement}{id}{\def\supp@label{#1}}
\long\def\supplement{\@ifnextchar[{\@supplement}{\@supplement[]}}
\long\def\@supplement[#1]{\suppsection@fmt\global\let\suppsection@fmt\relax\setkeys{supplement}{#1}}
\def\endsupplement{\par\noindent\@ifundefined{@sname}{}{\@ifundefined{supp@label}{}{\label{\supp@label}}\textbf{\@sname: }}\@ifundefined{@stitle}{}{\textbf{\@stitle\ }}(\@ifundefined{slink@doi}{}{\slink@doi@fmt}\@ifundefined{slink@url}{}{\href{\slink@url}{\slink@url}}\@ifundefined{@sdatatype}{}{; \@sdatatype}). \@ifundefined{@sdescription}{}{\@sdescription}\par}"
  );
  // :1302-1345 (vtexthm.sty, which the class carries): the starred twin of every `\newtheorem` and `\newproclaim`
  // (ims_support_sty.rs), and arxbj's `\newremark` (:1479), `\newtheorem` in the remark style.
  RawTeX!(r"\def\newremark{\theoremstyle{remark}\lx@vtex@thm}");
  // :1412-1421: the class's own `{thm}` (elsart_support_sty.rs here) with its twin `{thm*}` (1309.7754).
  RawTeX!(r"\@ifundefined{thm*}{\lx@vtex@newtheorem*{thm*}{Theorem}}{}");
  RawTeX!(r"\expandafter\def\csname remark*\endcsname{\begin{remark}}");
  RawTeX!(r"\expandafter\def\csname endremark*\endcsname{\end{remark}}");
});

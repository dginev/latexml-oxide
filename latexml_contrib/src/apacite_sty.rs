//! apacite.sty (TL 2025 apacite.sty v6.03) — APA citations and reference lists.
//!
//! Two layers, as the package has them:
//! * the reference-list formatting layer — every macro apacite.bst writes into a `.bbl`
//!   (`\APACrefYearMonthDay`, `\APACjournalVolNumPages`, `\BPGS`, `\BCnt`, `{APACrefURL}`, …)
//!   and the strings they print — is the package's own TeX, run verbatim (`RawTeX!` blocks
//!   below, each citing its apacite.sty lines);
//! * the citation layer (`\cite<pre>[post]{keys}`, `\citeA`, `\citeNP`, …) is a semantic
//!   binding onto natbib's `\cite`: natbib_sty.rs `\lx@NAT@parselabel` splits the bibitem label
//!   `\citeauthoryear{full}{short}{year}` and digests its apacite macros (`\APACyear`,
//!   `\APACexlab{\BCnt{1}}`, `\BBA`, `\BOthers`, `\APACciteatitle`, …) into the `ltx:tags` the
//!   post-processor cites from.
//!
//! Perl ships no apacite binding: its default run leaves every macro undefined, and
//! `--includestyles` raw-loads apacite.sty, whose own `\cite`, `\@lbibitem` and
//! `\thebibliography` (apacite.sty:259-470, 1152-1242) leave every citation unresolved.
use latexml_package::prelude::*;

LoadDefinitions!({
  RequirePackage!("natbib");
  // apacite-generated .bbl entries routinely contain `\url{...}` even when the user's main
  // .tex doesn't load url. Witness 2205.09172 (cogsci article with apacite-formatted main.bbl).
  RequirePackage!("url");
  // `\doi`: apacite.sty:1812-1816 `\AtBeginDocument{\providecommand{\doi}{\begingroup
  // \urlstyle{APACsame}\Url}}` — the DOI set as a url in the surrounding font (`APACsame`,
  // :1803), with no link; `{APACrefDOI}` prints the `doi:` prefix (`\doiprefix`, :1817-1819).
  // `HyperVerbatim` reads it as `\Url` does (`_`, `~`, `%`, `<…>` literal), and the ASCII
  // encoding keeps OT1/T1 from turning them into accents, as `\UrlFont` does (url_sty.rs,
  // OXIDIZED_DESIGN #144). Witness 2205.09172.
  // A bare `\doi` (no brace) takes its token, as the url-less `\doi[1]` (:1814) does, rather than
  // letting `HyperVerbatim` scan ahead to the next `{`.
  DefMacro!(
    "\\lx@apac@doi",
    "\\@ifnextchar\\bgroup\\lx@apac@doi@verbatim\\lx@apac@doi@token"
  );
  DefMacro!(
    "\\lx@apac@doi@verbatim HyperVerbatim",
    "{\\fontencoding{ASCII}\\selectfont #1}"
  );
  DefMacro!("\\lx@apac@doi@token{}", "#1");

  // apacite.sty:133 `\newif\if@doi`, on by default (:181-182 `\ExecuteOptions{...,doi,...}`);
  // the citation-key suffix every flag lookup appends (:1905-1908, :1914, :1943); and
  // `\definemetaflag` (:1122-1136), which `\nocitemeta` (:1137-1149) writes to the `.aux` for
  // the next run — with no `.aux` round trip the flag is set at once, so `\APACinsertmetastar`
  // (:1257-1264) prints the meta-analysis asterisk when `\nocitemeta` precedes the bibliography
  // (after it, pdflatex's second run marks the entry and this does not). The note it defines
  // (`\APAC@metaprenote@…`) is typeset through natbib's `\bibpreamble` (:1835-1850, below).
  // OXIDIZED_DESIGN_DIVERGENCES #342.
  RawTeX!(
    r"%
\newif\if@doi
\@doitrue
\@ifundefined{@extra@b@citeb}{\def\@extra@b@citeb{}}{}
\gdef\@extra@binfo{}
\def\APAC@extra@b@citeb{\APAC@curr@aux\APAC@bu\@extra@b@citeb}%
\def\APAC@extra@binfo{\APAC@curr@aux\APAC@bu\@extra@binfo}%
\@ifundefined{APAC@bu}{\def\APAC@bu{}}{}
\@ifundefined{APAC@curr@aux}{\def\APAC@curr@aux{}}{}
\def\definemetaflag#1{%
  \@bsphack
  \expandafter\global\expandafter\def
    \csname APAC@metaprenote@\APAC@extra@b@citeb\endcsname{%
      \APACmetaprenote}%
  \@for\@citeb:=#1\do{%
    \edef\@citeb{\expandafter\@firstofone\@citeb}%
    \@ifundefined{flagmeta@\@citeb\APAC@extra@b@citeb}%
      {\global\expandafter
       \def\csname flagmeta@\@citeb\APAC@extra@b@citeb\endcsname{DUMMY}%
      }%
      {}%
  }%
  \@esphack
}
\def\nocitemeta#1{\nocite{#1}\definemetaflag{#1}}"
  );
  // apacite.sty:1835-1845 verbatim: natbib's `\bibpreamble` (typeset after the bibliography's
  // heading, the kernel's `\lx@bibliography@preamble`) gains `\bibliographyprenote` and the
  // meta-analysis note `\definemetaflag` defined; Perl has no apacite binding and natbib's never
  // typesets it (KPE #325). Not ported: the `\endthebibliography` redefinition beside it
  // (:1846-1848, a `\normalsize` after the list; the kernel constructor is locked). Guard
  // `perfect_kernel_gemini::bibpreamble_is_printed`.
  RawTeX!(
    r"%
\AtBeginDocument{%
  \@ifpackageloaded{natbib}{%
    \let\@oldbibpreamble\bibpreamble
    \def\bibpreamble{%
      \@oldbibpreamble%
      \bibliographytypesize%
      \bibliographyprenote%
      \@ifundefined{APAC@metaprenote@\APAC@extra@b@citeb}%
        {}% skip
        {\csname APAC@metaprenote@\APAC@extra@b@citeb\endcsname}%
    }%
  }{}%
}"
  );
  // Options (apacite.sty:128-180, defaults :181-182 `\ExecuteOptions{…,doi,…}` above): `nodoi`
  // (:134-135) drops the DOIs `{APACrefDOI}` holds; the others choose layout, the author index
  // and cite forms this binding does not build.
  DeclareOption!("doi", r"\@doitrue");
  DeclareOption!("nodoi", r"\@doifalse");
  DeclareOption!(None, {});
  ProcessOptions!();
  // The reference-list formatting layer, verbatim: apacite.sty:1243-1522 (`\@connect@with@commas`
  // through `\bibphant`) less `\let\Bem\emph` (:1519, below). Its skips re-allocate the class's
  // `\bibindent` (article.cls:563 `\newdimen`) as apacite's `-\bibleftmargin` (:1507-1516); a
  // `.tex` body's `\setlength{\bibleftmargin}{…}` reads them (witness 2205.09172).
  RawTeX!(
    r"%
\def\@connect@with@commas#1{%
  \def\@comma@space{\unskip, }%
  \let\@connect@string\relax
  \@for\@element@:=#1\do{%
     \ifx\@empty\@element@%
     \else
       \@connect@string\@element@%
       \let\@connect@string\@comma@space
     \fi
  }%
  \let\@connect@string\@undefined
  \let\@comma@space\@undefined
}
\newcommand{\APACmetastar}{\ensuremath{{}^\ast}}
\def\APACinsertmetastar#1{%
  \@for\@citeb:=#1\do{%
    \edef\@citeb{\expandafter\@firstofone\@citeb}%
    \@ifundefined{flagmeta@\@citeb\APAC@extra@b@citeb}%
      {}% skip
      {{\APACmetastar}}%
  }%
}
\newenvironment{APACrefauthors}{%
  \begingroup \APACrefauthstyle
}{\endgroup }
\newcommand{\APACrefYear}[1]{%
  {\BBOP}{#1}{\BBCP}%
}
\newcommand{\APACrefatitle}[2]{#2}
\newcommand{\APACrefbtitle}[2]{\Bem{#2}}
\newcommand{\APACrefaetitle}[2]{[#2]}
\newcommand{\APACrefbetitle}[2]{[#2]}
\newcommand{\APACjournalVolNumPages}[4]{%
  \Bem{#1}%             journal
  \ifx\@empty#2\@empty
  \else
    \unskip, \Bem{#2}%  volume
  \fi
  \ifx\@empty#3\@empty
  \else
    \unskip({#3})%      issue number
  \fi
  \ifx\@empty#4\@empty
  \else
    \unskip, {#4}%      pages
  \fi
}
\DeclareRobustCommand{\APACaddressPublisher}[2]{%
  \ifx\@empty#1\@empty
    \ifx\@empty#2\@empty
    \else
      {#2}%                 publisher
    \fi
  \else
    {#1}%                   address
    \ifx\@empty#2\@empty
    \else
      \unskip: {#2}%        publisher
    \fi
  \fi
}
\let\APACaddressInstitution\APACaddressPublisher
\DeclareRobustCommand{\APACaddressSchool}[2]{%
  \ifx\@empty#2\@empty
    \ifx\@empty#1\@empty
    \else
      {#1}%                 address
    \fi
  \else
    {#2}%                   school
    \ifx\@empty#1\@empty
    \else
      \unskip, {#1}%        address
    \fi
  \fi
}
\DeclareRobustCommand{\APACtypeAddressSchool}[3]{%
  \ifx\@empty#1\@empty
    \ifx\@empty#3\@empty
      \ifx\@empty#2\@empty
      \else
        ({#2})%               address
      \fi
    \else
      ({#3}%                  school
      \ifx\@empty#2\@empty
      \else
        \unskip, {#2}%        address
      \fi
      )%
    \fi
  \else
    ({#1}%                    type
    \ifx\@empty#3\@empty
      \ifx\@empty#2\@empty
      \else
        \unskip, {#2}%        address
      \fi
    \else
      \unskip, {#3}%          school
      \ifx\@empty#2\@empty
      \else
        \unskip, {#2}%        address
      \fi
    \fi
    )%
  \fi
}
\newcommand{\APACaddressPublisherEqAuth}[2]{%
  \ifx\@empty#1\@empty
    {\BAuthor{}}% Publisher formatted as ``Author''
  \else
    {#1\unskip: \BAuthor{}}% Address: Author
  \fi
}
\let\APACaddressInstitutionEqAuth\APACaddressPublisherEqAuth
\let\APAChowpublished\relax
\newenvironment{APACrefURL}[1][]{%
  \ifx\@empty#1\@empty
    \BRetrievedFrom % Retrieved from
  \else
    \BRetrieved{#1}%  Retrieved <date>, from
  \fi
}{}
\newenvironment{APACrefDOI}{%
  \global\let\old@doi\doi
  \if@doi
    \doiprefix
  \else
    \global\let\doi\@gobble
  \fi
  }{\global\let\doi\old@doi }
\newenvironment{APACrefURLmsg}{%
  \BMsgPostedTo
}{}
\newcommand{\APACorigED}[1]{%
  \ifx\@empty#1\@empty
  \else
    \Bby\ {#1}, \BED{}% ``by E. D. Itor (Ed.)''
  \fi
}
\newcommand{\APACorigEDS}[1]{%
  \ifx\@empty#1\@empty
  \else
    \Bby\ {#1}, \BEDS{}% ``by E. D. Itor \& A. N. Other (Eds.)''
  \fi
}
\newcommand{\APACbVolEdTR}[2]{%
  \ifx\@empty#1\@empty
    \ifx\@empty#2\@empty
    \else
      {(#2)}%            (Technical Report No.\ <no>)
    \fi
  \else
    ({#1}%               (2nd ed., Vol.~1
    \ifx\@empty#2\@empty
    \else
      \unskip; {#2}%     ; Technical Report No.\ <no>
    \fi
    )%                   Final parenthesis.
  \fi
}
\newcommand{\APACbVolEdTRpgs}[3]{%
  \ifx\@empty#1\@empty
    \ifx\@empty#2\@empty
      \ifx\@empty#3\@empty
      \else
        {(#3)}%% (pp. 10--30)
      \fi
    \else
      %% (Technical Report No.\ <no>, pp. 10--30)
      (\@connect@with@commas{{#2},{#3}})%
    \fi
  \else
    ({#1}%                  (2nd ed., Vol.~1
    \ifx\@empty#2\@empty
      \ifx\@empty#3\@empty
      \else
        \unskip; {#3}%      ; pp. 10--30
      \fi
    \else
      %%                    ; Technical Report No.\ <no>, pp. 10--30
      \unskip; \@connect@with@commas{{#2},{#3}}%
    \fi
    )%                      Final parenthesis.
  \fi
}
\newcommand{\APACrefnote}[1]{%
  \ifx\@empty#1\@empty
  \else
    ({#1})%
  \fi
}
\newcommand{\APACorigyearnote}[2]{%
  \ifx\@empty#1\@empty
    \APACrefnote{#2}%
  \else
    \ifx\bibnodate#1\@empty
      \APACrefnote{#2}%
    \else
      (\BOWP{} {#1}%
      \ifx\@empty#2\@empty
      \else
        \unskip; {#2}%
      \fi
      )%
    \fi
  \fi
}
\newcommand{\APACorigjournalnote}[6]{%
  (\BREPR{} %          ``(Reprinted from '' (note the space)
  \Bem{#2}%            Journal (should not be empty)
  \ifx\@empty#1\@empty
  \else
    \unskip, {#1}%     , year
  \fi
  \ifx\@empty#3\@empty
  \else
    \unskip, \Bem{#3}% , volume
  \fi
  \ifx\@empty#4\@empty
  \else
    \unskip{[#4]}%     [issue number]
  \fi
  \ifx\@empty#5\@empty
  \else
    \unskip, {#5}%     , pages
  \fi
  \ifx\@empty#6\@empty
  \else
    \unskip; {#6}%     ; note
  \fi
  )%                   Final parenthesis
}
\newcommand{\APACorigbooknote}[9]{%
    %% ``(Reprinted from '' (note the space)
  (\BREPR{} %
    %% Title, edition, volume, pages, editor, year, address: publisher
  \@connect@with@commas{%
    {#3},{#4},{#5},{#6},{#2},{#1},{\APACaddressPublisher{#7}{#8}}%
  }%%
  \ifx\@empty#9\@empty
  \else
    ; #9%%              ; note
  \fi
  )%                    Final parenthesis.
}
\newenvironment{APACrefannotation}{%
  \begin{quotation}\noindent\ignorespaces
}{\end{quotation}}
\newcommand{\BAstyle}{}%
\newcommand{\BAastyle}{}%
\newcommand{\APACrefauthstyle}{}%
\newcommand{\APACciteatitle}[1]{``#1''}
\newcommand{\APACcitebtitle}[1]{{\em #1\/}}
\newcommand{\APACyear}[1]{{#1}}%
\newcommand{\APACexlab}[1]{{#1}}%
\newcounter{BibCnt}
\renewcommand{\theBibCnt}{\alph{BibCnt}}
\DeclareRobustCommand{\BCnt}[1]{\setcounter{BibCnt}{#1}\theBibCnt}
\DeclareRobustCommand{\BCntIP}[1]{\setcounter{BibCnt}{#1}\mbox{-\theBibCnt}}
\DeclareRobustCommand{\BCntND}[1]{\setcounter{BibCnt}{#1}\mbox{-\theBibCnt}}
\let\bibliographytypesize\normalsize
\newcommand{\bibliographyprenote}{}
\newskip{\bibleftmargin}
\newskip{\bibindent}
\newskip{\bibparsep}
\newskip{\bibitemsep}
\newskip{\biblabelsep}
\setlength{\bibleftmargin}{2.5em}
\setlength{\bibindent}{-\bibleftmargin}
\setlength{\bibparsep}{0pt}
\setlength{\bibitemsep}{0pt plus .3pt}
\setlength{\biblabelsep}{0pt}
\let\bibcorporate\relax
\newcommand{\BBA}{\BBAA}% `\&'
\newcommand{\bibnotype}{}
\newcommand{\APACSortNoop}[1]{}
\let\bibphant\APACSortNoop"
  );
  // `\Bem`: apacite.sty:1519 `\let\Bem\emph`, the form every modern `.bbl` reaches through the
  // macros above (`\Bem{#2}`). A pre-2012 apacite.bst `.bbl` writes it as a declaration,
  // `{\Bem Biometrika}`, `{\Bem 63}` (theapa.sty:69 `\let\Bem\itshape`; witness 2304.11127,
  // html_feedback#6489, which ships its own theapa.sty — the guard is the old-`.bbl` fixture
  // `apacite_old_bbl.tex`), where `\emph` would italicize one letter. One binding serves both
  // generations: a brace makes it `\emph`, anything else the declaration `\em` — so an old
  // `{\Bem {IEEE} Transactions}` italicizes "IEEE" only, as today's apacite does.
  // OXIDIZED_DESIGN_DIVERGENCES #342.
  RawTeX!(r"\DeclareRobustCommand{\Bem}{\@ifnextchar\bgroup{\emph}{\em}}");
  RawTeX!(
    r"%
\AtBeginDocument{\providecommand{\doi}{\lx@apac@doi}}
\@ifundefined{doiprefix}{%
  \newcommand{\doiprefix}{doi:\penalty0{}}%
}{}%"
  );
  // The reference-list strings, `\APACmonth` and `\APACrefYearMonthDay`: apacite.sty:2068-2156,
  // less the `\hbox{}` after each abbreviation (`Vol.\hbox{}`, `pp.\hbox{}`, `et al.\hbox{}`):
  // a box only resets the space factor to 1000 (TeXbook ch. 12, texbook.tex:4464-4466; tex.web
  // :20905), so the period is not a sentence end — spacing the XML does not carry, while an
  // empty `\hbox{}` becomes an empty `<ltx:text xml:id>` under `ids` (ar5iv), as in Perl.
  // `\BBAA` is `\&`, not an alignment `&` (witness 2205.09172, 19 "Stray alignment" errors);
  // `\BOthersPeriod` (witness 2005.03899), `\BTR`/`\BNUM` (witness 2205.05718), `\BPG`
  // (witness 2205.09172), `\BEd` (witness 2106.02003).
  RawTeX!(
    r"%
\newcommand{\APACmetaprenote}{%
  References marked with an asterisk indicate studies included in
  the meta-analysis.}
\newcommand{\bibmessage}{Msg}% Message, for internet forums and the like
\newcommand{\bibcomputerprogram}{Computer program}
\newcommand{\bibcomputerprogrammanual}{Computer program manual}
\newcommand{\bibcomputerprogramandmanual}{Computer program and manual}
\newcommand{\bibcomputersoftware}{Computer software}
\newcommand{\bibcomputersoftwaremanual}{Computer software manual}
\newcommand{\bibcomputersoftwareandmanual}{Computer software and manual}
\newcommand{\bibprogramminglanguage}{Programming language}
\newcommand{\bibnodate}{n.d.}% no date
\newcommand{\BIP}{in press}         % in press
\newcommand{\BOthers}[1]{et al.}%       ``and others''
\newcommand{\BOthersPeriod}[1]{et al.}% ``and others.'', with a period
\newcommand{\BIn}{In}                 % for ``In '' editor...
\newcommand{\Bby}{by}                 % for ``by '' editor... (in reprints)
\newcommand{\BED}{Ed.}         % editor
\newcommand{\BEDS}{Eds.}       % editors
\newcommand{\BTRANS}{Trans.}   % translator
\newcommand{\BTRANSS}{Trans.}  % translators
\newcommand{\BTRANSL}{trans.}  % translation, for the year field
\newcommand{\BCHAIR}{Chair}           % chair of symposium
\newcommand{\BCHAIRS}{Chairs}         % chairs
\newcommand{\BVOL}{Vol.}       % volume (of a multi-volume book)
\newcommand{\BVOLS}{Vols.}     % volumes
\newcommand{\BNUM}{No.}        % number (of a technical report)
\newcommand{\BNUMS}{Nos.}      % numbers
\newcommand{\BEd}{ed.}         % edition
\newcommand{\BCHAP}{chap.}     % chapter (for electronic documents)
\newcommand{\BCHAPS}{chap.}    % chapters
\newcommand{\BPG}{p.}          % page
\newcommand{\BPGS}{pp.}        % pages
%% Default technical report type name.
\newcommand{\BTR}{Tech.\ Rep.}
%% Default PhD thesis type name.
\newcommand{\BPhD}{Doctoral dissertation}
%% Default unpublished PhD thesis type name.
\newcommand{\BUPhD}{Unpublished doctoral dissertation}
%% Default master's thesis type name.
\newcommand{\BMTh}{Master's thesis}
%% Default unpublished master's thesis type name.
\newcommand{\BUMTh}{Unpublished master's thesis}
\newcommand{\BAuthor}{Author}% ``Author'' if publisher = author
\newcommand{\BOWP}{Original work published}
\newcommand{\BREPR}{Reprinted from}
\newcommand{\BAvailFrom}{Available from\ }%          Websites; note the space.
%% The argument is the date on which it was last checked.
\newcommand{\BRetrieved}[1]{Retrieved {#1}, from\ }% Websites; note the space.
\newcommand{\BRetrievedFrom}{Retrieved from\ }%      Websites; note the space.
\newcommand{\BMsgPostedTo}{Message posted to\ }%     Messages; note the space.
\newcommand{\BBOP}{(}   % opening parenthesis
\newcommand{\BBCP}{)}   % closing parenthesis
\newcommand{\BBOQ}{}    % opening quote for article title
\newcommand{\BBCQ}{}    % closing quote for article title
\newcommand{\BBAA}{\&}  % between authors in parenthetical cites and ref. list
\newcommand{\BBAB}{and} % between authors in in-text citation
\newcommand{\BAnd}{\&}  % for ``Ed. \& Trans.'' in ref. list
\DeclareRobustCommand{\BPBI}{.~}% Period between initials
\DeclareRobustCommand{\BHBI}{.-}% Hyphen between initials
\newcommand{\BAP}{ }    % after prefix, before first citation
\newcommand{\BBAY}{, }  % between author(s) and year
\newcommand{\BBYY}{, }  % between years of multiple citations with same author
\newcommand{\BBC}{; }   % between cites
\newcommand{\BBN}{, }   % before note
\newcommand{\BCBT}{,}   % comma between authors in ref. list when no. of
                       %% authors = 2
\newcommand{\BCBL}{,}   % comma before last author when no. of authors > 2
\newcommand{\BDBL}{, \dots{} }% dots before last author when no. of authors > 7
\newcommand{\APACmonth}[1]{\ifcase #1\or January\or February\or March\or
    April\or May\or June\or July\or August\or September\or October\or
    November\or December\or Winter\or Spring\or Summer\or Fall\else
    {#1}\fi}
\newcommand{\APACrefYearMonthDay}[3]{%
  {\BBOP}{#1}%           year (+ addendum); should not be empty
  \ifx\@empty#2\@empty
    \ifx\@empty#3\@empty
    \else
      \unskip, {#3}%     day
    \fi
  \else
    \unskip, {#2}%       month
    \ifx\@empty#3\@empty
    \else
      \unskip~{#3}%      day
    \fi
  \fi
  {\BBCP}%               closing parenthesis
}"
  );
  // `\PrintOrdinal{N}` → "Nth": apacite.sty:2157-2196 (condensed). `.bbl` edition fields,
  // `\PrintOrdinal{3}\ \BEd` (witness 2106.02003).
  RawTeX!(
    r"%
\let\@xp\expandafter
\newcommand{\PrintOrdinal}[1]{%
    \afterassignment\print@ordinal
    \count@ 0#1\relax\@nil}
\def\print@ordinal#1#2\@nil{%
    \ifx\relax#1\relax
        \ifnum\count@>\z@ \CardinalNumeric\count@ \else ??th\fi
    \else
        \ifnum \count@>\z@ \number\count@ \fi #1#2\relax
    \fi}
\newcommand{\CardinalNumeric}[1]{%
    \number#1\relax
    \if \ifnum#1<14 \ifnum#1>\thr@@ T\else F\fi \else F\fi Tth%
    \else \@xp\keep@last@digit\@xp#1\number#1\relax
        \ifcase#1th\or st\or nd\or rd\else th\fi
    \fi}
\def\keep@last@digit#1#2{%
    \ifx\relax#2\@xp\@gobbletwo \else #1=#2\relax \fi \keep@last@digit#1}"
  );

  // Hooks with no reference-list text: backref's (apacite.sty:1875-1877, `\PrintBackRefs` is
  // `\@gobble` without backref, :1898), url's line-break tables (:1782), and the author-index
  // `theindex` layouts the `stdindex`/`tocindex`/`emindex`/`ltxemindex` options install
  // (:1586, :1649, :1714, :1741 — no arguments; the index is built by the post-processor).
  def_macro_noop("\\PrintBackRefs{}")?;
  def_macro_noop("\\CurrentBib")?;
  def_macro_noop("\\APACrestorebibitem")?;
  def_macro_noop("\\APACurlBreaks")?;
  def_macro_noop("\\APACstdindex")?;
  def_macro_noop("\\APACtocindex")?;
  def_macro_noop("\\APACemindex")?;
  def_macro_noop("\\APACltxemindex")?;
  // The author-index entries `apacitex.bst`/`apacannx.bst` write into a `.bbl` (`\AX{…}`,
  // `\corporateAX{…}`) gobble without the `index` option (apacite.sty:1583-1584).
  RawTeX!(r"\let\AX\@gobble\let\corporateAX\@gobble");
  // `\APACbibcite{key}{label}` (apacite.sty:1167-1172, the `.aux`'s `\bibcite`, :1173) caches `\b@key`
  // and `\Y@key` for apacite's own `\cite`; natbib's cite resolves from the bibitem tags instead.
  def_macro_noop("\\APACbibcite{}{}")?;
  DefMacro!("\\APAhyperref{}{}", "#2");
  def_macro_identity("\\APACstd{}")?;

  // apacite citation forms (apacite.sty L328+). Delegate to natbib's \cite
  // which we wrapped in natbib_sty.rs. Forms:
  //   \citeA<pre>[post]{key}      — author-only ("Smith")
  //   \citeNP<pre>[post]{key}     — citation without parens
  //   \citeyearNP<pre>[post]{key} — year-only without parens
  // Witness 2407.14158, 2407.18402, 2407.16770 (apacite-using papers).
  //
  // apacite's ANGLE-BRACKET pre-note. apacite.sty L259-311 gives every classic
  // cite form the dispatch `\@ifnextchar< {\@cite} {\@cite<>}`, and L313-327
  // `\def\@cite<#1>{... \@ifnextchar[ {\@@cite<#1>} {\@@cite<#1>[]}}` — so the
  // full apacite citation syntax is
  //   \cite<pre-note>[post-note]{key-list}
  // Without the `<...>` form the kernel/natbib `\cite` takes the single token
  // `<` as its key list: the citation renders as a dangling `[<]`, the REAL
  // keys are never cited (so they are silently absent from the References) and
  // `see>` leaks into the body text. Witness 2605.10951
  // (`\cite<see>{Gangopadhyay02,Ferris25}`, agujournal2019), 2606.16518,
  // 2606.19048, 2606.21531, 2606.24563.
  //
  // `OptionalAngled` (NOT `OptionalMatch:< OptionalUntil:>`): `Until` never
  // checks for the OPENING delimiter, so when no `<` is present it scans to the
  // next `>` anywhere downstream — `\citeA{Smith} and $a > b$` swallowed the
  // cite and the math and reported the key as `b`.
  // The signature `[pre1] OptionalAngled [post]` is UNAMBIGUOUS and so serves
  // both spellings with one prototype: natbib's `[pre][post]` can only ever
  // match `[pre1]…[post]` (the angled slot peeks, sees `[`, and yields nothing),
  // and apacite's `<pre>[post]` can only ever match `OptionalAngled [post]`
  // (the `[pre1]` slot peeks, sees `<`, and yields nothing). The two pre-note
  // slots are therefore mutually exclusive, so `[#1#2]` simply concatenates
  // whichever one fired — no conditional needed. `\citeA[a][b]{k}` /
  // `\citeA[a]{k}` keep emitting exactly what they did before this change.
  DefMacro!(
    "\\citeA [] OptionalAngled [] Semiverbatim",
    "\\citet[#1#2][#3]{#4}"
  );
  DefMacro!(
    "\\citeNP [] OptionalAngled [] Semiverbatim",
    "\\citealp[#1#2][#3]{#4}"
  );
  DefMacro!(
    "\\citeyearNP [] OptionalAngled [] Semiverbatim",
    "\\citeyear[#1#2][#3]{#4}"
  );

  // `\cite` keeps natbib's meaning on the no-`<` path (apacite's `\cite` is
  // parenthetical, but re-pointing it at `\citep` would change rendering for
  // every apacite paper — out of scope here); it only gains the pre-note form.
  // Re-emitting `[#1#2][#3]` is behaviour-preserving for the bracket spellings:
  // natbib's `swap_pre_post` keys off a NON-EMPTY post, so `\cite[a]{k}` →
  // `[a][]` still swaps to post=a exactly as the bare `\cite[a]{k}` does.
  Let!("\\lx@apac@core@cite", "\\cite");
  DefMacro!(
    "\\cite [] OptionalAngled [] Semiverbatim",
    "\\lx@apac@core@cite[#1#2][#3]{#4}"
  );
  // apacite "short" cite family (apacite.sty L277-401, the CLASSIC block —
  // distinct from the `\citet`/`\citep` defined only under the `natbibemu`
  // option at L587+). These are abbreviated-author variants of
  // \cite/\citeA/\citeNP/\citeauthor: apacite shortens long author lists to
  // "et al." sooner, but the reference resolves identically, so we delegate
  // to the matching natbib command (same approximation as `\citeNP` above).
  //   \shortcite       — parenthetical          → \citep
  //   \shortciteA      — textual                → \citet
  //   \shortciteNP     — no parentheses         → \citealp
  //   \shortciteauthor — author-only            → \citeauthor
  // Witness 1606.03620 (`\shortciteNP`, via apacdoc's `\DSMshortciteNP`).
  DefMacro!("\\shortcite[][] Semiverbatim", "\\citep[#1][#2]{#3}");
  DefMacro!("\\shortciteA[][] Semiverbatim", "\\citet[#1][#2]{#3}");
  DefMacro!("\\shortciteNP[][] Semiverbatim", "\\citealp[#1][#2]{#3}");
  DefMacro!(
    "\\shortciteauthor[][] Semiverbatim",
    "\\citeauthor[#1][#2]{#3}"
  );

  // Old-format compat macros (pre-2012 apacite.bst): the `\bibitem` label uses
  // `\BCAY{full}{short}{year}` and authors are separated with `\BBACOMMA`. Single binding, both
  // versions (LaTeXML design): the MODERN `.bbl` (`\citeauthoryear` + `{APACrefauthors}`) never
  // uses these, so the binding just defines the superset.
  //   * `\def\BCAY##1##2##3{\BCA{##1}{##2}}` (apacite.sty L260) and
  //     `\def\BCA##1##2{{\@BAstyle ##1}}` (L496/854) with `\@BAstyle` empty by
  //     default (L468/818) — i.e. the label is the full author.
  //   * `\BBACOMMA` (older versions) = comma before the `\BBA` ampersand
  //     ("Smith, J.\BBACOMMA\ \BBA\ Doe").
  // Perl ships no apacite binding; Rust surpasses. Related html_feedback#6489
  // (arXiv 2304.11127, multibib + `theapa` `.bbl`: 55×\BCAY / 66×\Bem flood).
  def_macro_noop("\\@BAstyle")?; // author text-style declaration (empty default)
  DefMacro!("\\BCA{}{}", "{\\@BAstyle #1}");
  DefMacro!("\\BCAY{}{}{}", "\\BCA{#1}{#2}");
  DefMacro!("\\BBACOMMA", ",");
});

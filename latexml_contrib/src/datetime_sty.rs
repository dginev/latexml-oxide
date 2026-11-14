use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "datetime.sty",
    "datetime.sty is only partially ported (date and time formats) and will not be interpreted raw."
  );

  // datetime.sty:51-110 and :149-187 verbatim (English, the monolingual branch; Perl has no datetime binding,
  // KNOWN_PERL_ERRORS #802): the three date counts `\@day`/`\@month`/`\@year` that `\dateformat{fmt}{d}{m}{y}`
  // (:564-565) sets before expanding `fmt`, read back as `\THEDAY`/`\THEMONTH`/`\THEYEAR` (:175-177); `\formatdate`,
  // which the format selectors redefine, `\longdate` the load-time default (with the day of the week while
  // `\ifshowdow`, :75 `\showdowtrue`/`\showdowfalse`, 2209.06700); `\newdateformat{name}{fmt}` (:181-187) defining
  // `\<name>` to make `\formatdate` `\dateformat{fmt}` — the same slot the selectors redefine, so whichever ran last
  // wins (chet chetdoc; 2506.21718, 2507.03037; batch 56aj: a private hook lost to the load-time `\longdate`; guard
  // `class_and_datetime_definitions_exist`); `\ordinaldate{n}` (:64-74, :63 of datetime-defaults.sty) is fmtcount's
  // `\ordinalnum` — "16th", the suffix raised by default (fmtcount.sty:34, fc-english.def:27-54; fmtcount has no
  // binding, so the English ordinal is defined here when it is undefined). `\textdate` keeps an approximation (it
  // spells the numbers with fmtcount's `\Ordinalstringnum`/`\Numberstringnum`).
  RawTeX!(
    r"\newif\if@dt@nodate\@dt@nodatefalse
\newcommand{\dateseparator}{/}
\@ifundefined{@orgargctr}{\newcount\@orgargctr}{}
\@ifundefined{@ordinalctr}{\newcount\@ordinalctr}{}
\providecommand*{\fmtord}[1]{\textsuperscript{#1}}
\providecommand*{\ordinalnum}[1]{\@orgargctr=#1\relax\@ordinalctr=#1\relax\@FCmodulo{\@ordinalctr}{100}%
  \ifnum\@ordinalctr>10 \ifnum\@ordinalctr<14 \def\lx@dt@ord{th}\else\lx@dt@ordsuffix\fi\else\lx@dt@ordsuffix\fi
  \number\@orgargctr\fmtord{\lx@dt@ord}}
\def\lx@dt@ordsuffix{\@FCmodulo{\@ordinalctr}{10}\edef\lx@dt@ord{\ifcase\@ordinalctr th\or st\or nd\or rd\else th\fi}}
\newcommand*{\ordinaldateenglish}[1]{\ordinalnum{#1}}
\DeclareRobustCommand*{\ordinaldate}[1]{\ordinalnum{#1}}
\newif\ifshowdow
\providecommand*{\formatdate}[3]{}
\newcount\@day \newcount\@month \newcount\@year
\DeclareRobustCommand*{\longdate}{\renewcommand*{\formatdate}[3]{%
  \ifshowdow\dayofweekname{##1}{##2}{##3} \fi
  \@day=##1\relax\@month=##2\relax\@year=##3\relax
  \ordinaldate{\the\@day}\ \monthname[\@month], \the\@year}}
\DeclareRobustCommand*{\shortdate}{\renewcommand*{\formatdate}[3]{%
  \ifshowdow\shortdayofweekname{##1}{##2}{##3} \fi
  \@day=##1\relax\@month=##2\relax\@year=##3\relax
  \ordinaldate{\the\@day}\ \shortmonthname[\@month], \the\@year}}
\newcount\@dtctr
\DeclareRobustCommand*{\yyyymmdddate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax
  \the\@year\dateseparator \twodigit\@month\dateseparator \twodigit\@day}}
\DeclareRobustCommand*{\ddmmyyyydate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax
  \twodigit\@day\dateseparator \twodigit\@month\dateseparator \the\@year}}
\DeclareRobustCommand*{\dmyyyydate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax
  \the\@day\dateseparator \the\@month\dateseparator \the\@year}}
\DeclareRobustCommand*{\ddmmyydate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax \@dtctr=\@year\@FCmodulo{\@dtctr}{100}%
  \twodigit\@day\dateseparator \twodigit\@month\dateseparator \twodigit\@dtctr}}
\DeclareRobustCommand*{\dmyydate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax \@dtctr=\@year\@FCmodulo{\@dtctr}{100}%
  \the\@day\dateseparator \the\@month\dateseparator \twodigit\@dtctr}}
\DeclareRobustCommand*{\textdate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax
  \the\@day\ \monthname[\@month] \the\@year}}
\DeclareRobustCommand*{\usdate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax
  \monthname[\@month]\ \the\@day, \the\@year}}
\DeclareRobustCommand*{\mmddyyyydate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax
  \twodigit\@month\dateseparator \twodigit\@day\dateseparator \the\@year}}
\DeclareRobustCommand*{\mdyyyydate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax
  \the\@month\dateseparator \the\@day\dateseparator \the\@year}}
\DeclareRobustCommand*{\mmddyydate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax \@dtctr=\@year\@FCmodulo{\@dtctr}{100}%
  \twodigit\@month\dateseparator \twodigit\@day\dateseparator \twodigit\@dtctr}}
\DeclareRobustCommand*{\mdyydate}{\renewcommand*{\formatdate}[3]{%
  \@day=##1\relax\@month=##2\relax\@year=##3\relax \@dtctr=\@year\@FCmodulo{\@dtctr}{100}%
  \the\@month\dateseparator \the\@day\dateseparator \twodigit\@dtctr}}
\providecommand*{\dateformat}[4]{\@day=#2\relax\@month=#3\relax\@year=#4\relax#1}
\providecommand{\newdateformat}[2]{\@ifundefined{#1}{%
  \expandafter\DeclareRobustCommand\csname#1\endcsname{\renewcommand{\formatdate}{\dateformat{#2}}}}{%
  \PackageError{datetime}{Can't create new date format, command \textbackslash#1 already defined}{}}}
\providecommand*\THEDAY{\the\@day}
\providecommand*\THEMONTH{\the\@month}
\providecommand*\THEYEAR{\the\@year}
\let\c@DAY=\@day \let\c@MONTH=\@month \let\c@YEAR=\@year"
  );
  // datetime.sty:188-258 verbatim: the time counters (`\THEHOUR`, `\THEMINUTE`, …),
  // `\currenttime` from `\time`, `\formattime`/`\settimeformat`, and
  // `\newtimeformat{name}{format}` defining `\<name>` and `\timeformat@<name>`
  // (huawei.cls:176-177 `\newtimeformat{daytime}{\twodigit{\THEHOUR}:…}`; class
  // census 2026-09-24). `\@FCmodulo` is fmtcount.sty:244's (datetime.sty:44-48 takes
  // it from there); the `\pdfcreationdate` branch is not taken here.
  RawTeX!(
    r"\let\twodigit\two@digits
\@ifundefined{@FCmodulo}{\newcount\@DT@modctr
  \def\@FCmodulo#1#2{\@DT@modctr=#1\relax\divide\@DT@modctr by #2\relax
    \multiply\@DT@modctr by #2\relax\advance #1 by -\@DT@modctr}}{}
\DeclareRobustCommand*{\currenttime}{\formattime{\currenthour}{\currentminute}{\currentsecond}}
\newcommand*{\formattime}[3]{\protect\@formattime{#1}{#2}{#3}}
\newcommand*{\@formattime}[3]{\csname timeformat@xxivtime\endcsname{#1}{#2}{#3}}
\newcommand*{\timeseparator}{:}
\providecommand*{\settimeformat}[1]{\@ifundefined{timeformat@#1}{\PackageError{datetime}{Unknown time format `#1'}{}}{\renewcommand*{\@formattime}[3]{\csname timeformat@#1\endcsname{##1}{##2}{##3}}}}
\newcount\c@HOUR \newcount\c@HOURXII \newcount\c@MINUTE \newcount\c@TOHOUR \newcount\c@TOMINUTE \newcount\c@SECOND
\def\THEHOUR{\the\c@HOUR}\def\THEHOURXII{\the\c@HOURXII}\def\THEMINUTE{\the\c@MINUTE}
\def\THETOHOUR{\the\c@TOHOUR}\def\THETOMINUTE{\the\c@TOMINUTE}\def\THESECOND{\the\c@SECOND}
\newcount\currenthour \newcount\currentminute \newcount\currentsecond
\currenthour=\time\relax \divide\currenthour by 60\relax
\currentminute=\time\relax \@FCmodulo{\currentminute}{60}\currentsecond=0\relax
\providecommand*{\newtimeformat}[2]{\@ifundefined{#1}{%
\expandafter\def\csname#1\endcsname{\csname timeformat@#1\endcsname{\currenthour}{\currentminute}{\currentsecond}}%
\expandafter\def\csname timeformat@#1\endcsname##1##2##3{\c@HOUR=##1\c@HOURXII=\c@HOUR
\ifnum\c@HOURXII>12 \advance\c@HOURXII by -12\relax\fi
\c@MINUTE=##2\c@TOHOUR=\c@HOURXII\advance\c@TOHOUR by 1\relax\@FCmodulo{\c@TOHOUR}{12}%
\c@TOMINUTE=\c@MINUTE\advance\c@TOMINUTE by -60\relax\multiply\c@TOMINUTE by -1\relax
\c@SECOND=##3\relax #2\relax}}{\PackageError{datetime}{Command \textbackslash#1 already defined}{}}}
\newtimeformat{xxivtime}{\twodigit\THEHOUR\timeseparator\twodigit\THEMINUTE}"
  );
  // datetime.sty:261-304: the other predefined time formats and their English
  // strings (arXiv 2605.13807, 2606.01587 `\settimeformat{hhmmsstime}`: "Unknown
  // time format" since the 56hz port kept only `xxivtime`). `oclock` (:271-290)
  // spells the time with fmtcount's `\Numberstring`, and fmtcount has no binding
  // (it loads only under `[rawstyles]`), so it is not declared and its option
  // stays a no-op.
  RequirePackage!("ifthen");
  RawTeX!(
    r"\newtimeformat{hhmmsstime}{\twodigit\THEHOUR\timeseparator\twodigit\THEMINUTE\timeseparator\twodigit\THESECOND}
\newtimeformat{ampmtime}{\ifthenelse{\value{HOUR}=0}{12}{\THEHOURXII}\timeseparator\twodigit\THEMINUTE
\ifthenelse{\value{HOUR}<12}{\amname}{\ifthenelse{\value{HOUR}=12}{ \noon}{\pmname}}}
\providecommand*{\amname}{am}\providecommand*{\pmname}{pm}
\providecommand*{\amorpmname}{\ifthenelse{\value{HOUR}>11}{\pmname}{\amname}}
\providecommand*{\amstring}{in the morning}\providecommand*{\pmstring}{in the afternoon}
\providecommand*{\amorpmstring}{\ifthenelse{\value{HOUR}>12}{\pmstring}{\amstring}}
\providecommand*{\halfpast}{Half past}\providecommand*{\quarterpast}{Quarter past}
\providecommand*{\quarterto}{Quarter to}\providecommand*{\noon}{Noon}
\providecommand*{\midnight}{Midnight}\providecommand*{\oclockstring}{O'Clock}"
  );
  // datetime.sty:571-611 `\monthname[num]`/`\shortmonthname[num]` (default `[\month]`), the English names of
  // datetime-defaults.sty:3-42 (ufrgscca manual); datetime.sty:613-731 the day of the week: `\testifleapyear`,
  // `\computedayofyear`/`\computedayofweek{d}{m}{y}` into the `\dayofyear`/`\dayofweek` counts, and
  // `\dayofweekname{d}{m}{y}` (26 papers, `\dayofweekname{\day}{\month}{\year}`: 1308.5288, 1409.5169, 2112.11386;
  // under ar5iv `\day`/`\month`/`\year` are the paper's source date), `\shortdayofweekname`, `\thisdayofweekname`, with
  // the English names of datetime-defaults.sty:44-62.
  RawTeX!(
    r"\providecommand*{\monthnameenglish}[1][\month]{\@orgargctr=#1\relax
  \ifcase\@orgargctr\PackageError{datetime}{Invalid Month number \the\@orgargctr}{}%
  \or January\or February\or March\or April\or May\or June\or July\or August\or September\or October\or November%
  \or December\else\PackageError{datetime}{Invalid Month number \the\@orgargctr}{}\fi}
\providecommand*{\shortmonthnameenglish}[1][\month]{\@orgargctr=#1\relax
  \ifcase\@orgargctr\PackageError{datetime}{Invalid Month number \the\@orgargctr}{}%
  \or Jan\or Feb\or Mar\or Apr\or May\or Jun\or Jul\or Aug\or Sept\or Oct\or Nov\or Dec%
  \else\PackageError{datetime}{Invalid Month number \the\@orgargctr}{}\fi}
\providecommand*{\dayofweeknameidenglish}[1]{\ifcase#1\relax
  \or Sunday\or Monday\or Tuesday\or Wednesday\or Thursday\or Friday\or Saturday\fi}
\providecommand*{\shortdayofweeknameidenglish}[1]{\ifcase#1\relax
  \or Sun\or Mon\or Tue\or Wed\or Thu\or Fri\or Sat\fi}
\newcommand*{\monthname}[1][\month]{\monthnameenglish[#1]}
\newcommand*{\shortmonthname}[1][\month]{\shortmonthnameenglish[#1]}
\newif\ifDTleapyear
\providecommand*{\testifleapyear}[1]{\DTleapyearfalse\@year=#1\relax
  \@dtctr=\@year\@FCmodulo{\@dtctr}{400}%
  \ifnum\@dtctr=0\relax\DTleapyeartrue
  \else\@dtctr=\@year\@FCmodulo{\@dtctr}{100}%
    \ifnum\@dtctr=0\relax\DTleapyearfalse
    \else\@dtctr=\@year\@FCmodulo{\@dtctr}{4}\ifnum\@dtctr=0\relax\DTleapyeartrue\fi
    \fi
  \fi}
\newcount\dayofyear
\providecommand*{\computedayofyear}[3]{\testifleapyear{#3}\dayofyear=0\relax
  \@day=#1\relax \@month=#2\relax \@year=#3\relax
  \ifcase\@month
  \or
  \or \advance\dayofyear by 31\relax
  \or \advance\dayofyear by 59\relax
  \or \advance\dayofyear by 90\relax
  \or \advance\dayofyear by 120\relax
  \or \advance\dayofyear by 151\relax
  \or \advance\dayofyear by 181\relax
  \or \advance\dayofyear by 212\relax
  \or \advance\dayofyear by 243\relax
  \or \advance\dayofyear by 273\relax
  \or \advance\dayofyear by 304\relax
  \or \advance\dayofyear by 334\relax
  \else\PackageError{datetime}{Invalid month number}{}%
  \fi
  \ifnum\@month>2\relax\ifDTleapyear\advance\dayofyear by 1\relax\fi\fi
  \advance\dayofyear by \@day\relax}
\newcount\dayofweek
\providecommand*{\computedayofweek}[3]{\computedayofyear{#1}{#2}{#3}%
  \@dtctr=#3\relax\advance\@dtctr by -1901\relax\@FCmodulo{\@dtctr}{28}%
  \dayofweek=\@dtctr\divide\dayofweek by 4\relax\advance\dayofweek by \@dtctr
  \advance\dayofweek by 2\relax\@FCmodulo{\dayofweek}{7}%
  \advance\dayofweek by \dayofyear\advance\dayofweek by -1\relax\@FCmodulo{\dayofweek}{7}%
  \advance\dayofweek by 1\relax}
\providecommand*{\dayofweeknameid}[1]{\dayofweeknameidenglish{#1}}
\providecommand*{\dayofweekname}[3]{\computedayofweek{#1}{#2}{#3}\dayofweeknameid{\dayofweek}}
\providecommand*{\thisdayofweekname}{\dayofweekname{\day}{\month}{\year}}
\providecommand*{\shortdayofweekname}[3]{\computedayofweek{#1}{#2}{#3}\shortdayofweeknameidenglish{\dayofweek}}
\providecommand*{\thisshortdayofweekname}{\dayofweekname{\day}{\month}{\year}}"
  );
  // datetime.sty:149-172 verbatim: `\newdate{name}{d}{m}{y}` stores the
  // three fields as `\date@<name>@d/@m/@y`, the `\getdate*` accessors read
  // them, and `\displaydate{name}` feeds them to `\formatdate` (batch 56aj —
  // the former no-ops dropped the displayed date).
  // datetime.sty:173-174 `\longdate` with the day of the week is the load-time default; :459 `\setdefaultdate` and the
  // :461-487 options select another (the `\dt@addtoextras` bookkeeping is babel's), `dayofweek`/`nodayofweek` the
  // day of the week, `level`/`raise` fmtcount's ordinal suffix (fmtcount.sty:230-233), `iso` the ISO date and time
  // (batch 56aj: the options were ignored). `nodate` (:478) sets its switch; the definitions it skips stay defined.
  RawTeX!(
    r"\newcommand*{\setdefaultdate}[1]{#1}\longdate\showdowtrue
\DeclareOption{long}{\setdefaultdate{\longdate}}
\DeclareOption{short}{\setdefaultdate{\shortdate}}
\DeclareOption{yyyymmdd}{\setdefaultdate{\yyyymmdddate}}
\DeclareOption{ddmmyyyy}{\setdefaultdate{\ddmmyyyydate}}
\DeclareOption{dmyyyy}{\setdefaultdate{\dmyyyydate}}
\DeclareOption{ddmmyy}{\setdefaultdate{\ddmmyydate}}
\DeclareOption{dmyy}{\setdefaultdate{\dmyydate}}
\DeclareOption{text}{\setdefaultdate{\textdate}}
\DeclareOption{us}{\setdefaultdate{\usdate}}
\DeclareOption{mmddyyyy}{\setdefaultdate{\mmddyyyydate}}
\DeclareOption{mdyyyy}{\setdefaultdate{\mdyyyydate}}
\DeclareOption{mmddyy}{\setdefaultdate{\mmddyydate}}
\DeclareOption{mdyy}{\setdefaultdate{\mdyydate}}
\DeclareOption{iso}{\setdefaultdate{\yyyymmdddate}\settimeformat{hhmmsstime}%
  \renewcommand*{\dateseparator}{-}\renewcommand*{\timeseparator}{:}}
\DeclareOption{level}{\def\fmtord##1{##1}}\DeclareOption{raise}{\def\fmtord##1{\textsuperscript{##1}}}
\DeclareOption{dayofweek}{\showdowtrue}\DeclareOption{nodayofweek}{\showdowfalse}
\DeclareOption{nodate}{\@dt@nodatetrue}\DeclareOption{hhmmss}{\settimeformat{hhmmsstime}}\DeclareOption{24hr}{\settimeformat{xxivtime}}\DeclareOption{12hr}{\settimeformat{ampmtime}}\DeclareOption{oclock}{}
\ProcessOptions\relax"
  );

  RawTeX!(
    r"\newcommand*{\newdate}[4]{\@ifundefined{date@#1@y}{\@namedef{date@#1@d}{#2}\@namedef{date@#1@m}{#3}\@namedef{date@#1@y}{#4}}{\PackageError{datetime}{Date `#1' already defined}{}}}
\newcommand*{\getdateyear}[1]{\@ifundefined{date@#1@y}{\PackageError{datetime}{Date `#1' not defined}{}}{\csname date@#1@y\endcsname}}
\newcommand*{\getdatemonth}[1]{\@ifundefined{date@#1@m}{\PackageError{datetime}{Date `#1' not defined}{}}{\csname date@#1@m\endcsname}}
\newcommand{\getdateday}[1]{\@ifundefined{date@#1@d}{\PackageError{datetime}{Date `#1' not defined}{}}{\csname date@#1@d\endcsname}}
\newcommand*{\displaydate}[1]{\@ifundefined{date@#1@y}{\PackageError{datetime}{Date `#1' not defined}{}}{\formatdate{\csname date@#1@d\endcsname}{\csname date@#1@m\endcsname}{\csname date@#1@y\endcsname}}}"
  );
  // datetime.sty:733-736 `\today` in the current date format, unless `nodate`; under ar5iv `\day`/`\month`/`\year` are
  // the paper's source date (the PDF's), and an undated source keeps ar5iv's `\relax` (ar5iv_sty.rs: the paper's date
  // is not invented). :789-829 `\pdfdate`, the load time as pdfTeX's `YYYYMMDDHHMMSS` with zero seconds (17 papers:
  // 1005.0302, 1203.1927, 1209.4035, `Compiled \pdfdate`).
  RawTeX!(
    r"\def\lx@dt@relax{\relax}
\if@dt@nodate\else\ifx\today\lx@dt@relax\else
  \DeclareRobustCommand*{\today}{\formatdate{\day}{\month}{\year}}%
\fi\fi
\newtoks\dt@a \newtoks\dt@b
\edef\pdfdate{\the\year}
\dt@b=\expandafter{\pdfdate}\dt@a=\expandafter{\the\month}
\ifnum\month<10\relax\edef\pdfdate{\the\dt@b0\the\dt@a}\else\edef\pdfdate{\the\dt@b\the\dt@a}\fi
\dt@b=\expandafter{\pdfdate}\dt@a=\expandafter{\the\day}
\ifnum\day<10\relax\edef\pdfdate{\the\dt@b0\the\dt@a}\else\edef\pdfdate{\the\dt@b\the\dt@a}\fi
\@dtctr=\time\divide\@dtctr by 60\relax
\dt@b=\expandafter{\pdfdate}\dt@a=\expandafter{\the\@dtctr}
\ifnum\@dtctr<10 \edef\pdfdate{\the\dt@b0\the\dt@a}\else\edef\pdfdate{\the\dt@b\the\dt@a}\fi
\@dtctr=\time\@FCmodulo{\@dtctr}{60}%
\dt@b=\expandafter{\pdfdate}\dt@a=\expandafter{\the\@dtctr}
\ifnum\@dtctr<10\relax\edef\pdfdate{\the\dt@b0\the\dt@a}\else\edef\pdfdate{\the\dt@b\the\dt@a}\fi
\dt@a={00}\dt@b=\expandafter{\pdfdate}\edef\pdfdate{\the\dt@b\the\dt@a}"
  );
});

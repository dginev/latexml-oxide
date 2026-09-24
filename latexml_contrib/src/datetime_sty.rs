use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "datetime.sty",
    "datetime.sty is only partially ported (date and time formats) and will not be interpreted raw."
  );

  // datetime.sty:181-188 `\newdateformat{name}{format}` DEFINES `\<name>`,
  // which installs `format` (written over `\THEDAY`/`\THEMONTH`/`\THEYEAR`,
  // datetime.sty:100-110) as the current date format; the no-op it replaced
  // left `\mydate` undefined (chet chetdoc; arXiv:2506.21718 / 2507.03037
  // only called the setter). `\formatdate{d}{m}{y}` binds the three fields
  // and expands the current format (default `d/m/y`, the earlier rendering).
  // datetime.sty:100-110 `\dateformat{fmt}{d}{m}{y}` binds the fields and
  // expands `fmt`; :181-188 `\newdateformat{name}{fmt}` makes `\<name>`
  // REDEFINE `\formatdate` as `\dateformat{fmt}` — the same slot the
  // built-in selectors (`\longdate` …) redefine, so whichever ran last wins
  // (batch 56aj: a private `\lx@dateformat` hook lost to the load-time
  // `\longdate`; guard `class_and_datetime_definitions_exist`).
  RawTeX!(
    r"\def\dateformat#1#2#3#4{\def\THEDAY{#2}\def\THEMONTH{#3}\def\THEYEAR{#4}#1}
\def\newdateformat#1#2{\@ifundefined{#1}{\expandafter\def\csname #1\endcsname{\def\formatdate{\dateformat{#2}}}}{}}
\def\formatdate{\dateformat{\THEDAY/\THEMONTH/\THEYEAR}}"
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
  // datetime.sty `\monthname[num]` / `\shortmonthname[num]` (default
  // `[\month]`) — were content-losing noops; emit the English month name
  // via \ifcase like the package's english definitions (datetime.sty /
  // datetime-defaults). Witness: ufrgscca manual (perfect-kernel corpus).
  RawTeX!(
    r"\newcommand*{\monthname}[1][\month]{%
  \ifcase#1\or January\or February\or March\or April\or May\or June\or
  July\or August\or September\or October\or November\or December\fi}
\newcommand*{\shortmonthname}[1][\month]{%
  \ifcase#1\or Jan\or Feb\or Mar\or Apr\or May\or Jun\or
  Jul\or Aug\or Sep\or Oct\or Nov\or Dec\fi}"
  );
  // datetime.sty L80-150 format selectors: each redefines \formatdate's
  // field order (faithful order, simplified separators). Witness assoccnt
  // manual (`\mmddyyyydate`; perfect-kernel 4-bundle cluster).
  RawTeX!(
    r"\def\mmddyyyydate{\def\formatdate##1##2##3{##2/##1/##3}}
\def\mdyyyydate{\def\formatdate##1##2##3{##2/##1/##3}}
\def\mmddyydate{\def\formatdate##1##2##3{##2/##1/##3}}
\def\mdyydate{\def\formatdate##1##2##3{##2/##1/##3}}
\def\ddmmyyyydate{\def\formatdate##1##2##3{##1/##2/##3}}
\def\dmyyyydate{\def\formatdate##1##2##3{##1/##2/##3}}
\def\ddmmyydate{\def\formatdate##1##2##3{##1/##2/##3}}
\def\dmyydate{\def\formatdate##1##2##3{##1/##2/##3}}
\def\yyyymmdddate{\def\formatdate##1##2##3{##3/##2/##1}}
\def\usdate{\def\formatdate##1##2##3{\monthname[##2] ##1, ##3}}
\def\textdate{\def\formatdate##1##2##3{##1 \monthname[##2] ##3}}
\def\longdate{\def\formatdate##1##2##3{\monthname[##2] ##1, ##3}}
\def\shortdate{\def\formatdate##1##2##3{\shortmonthname[##2] ##1, ##3}}"
  );

  // datetime.sty:149-172 verbatim: `\newdate{name}{d}{m}{y}` stores the
  // three fields as `\date@<name>@d/@m/@y`, the `\getdate*` accessors read
  // them, and `\displaydate{name}` feeds them to `\formatdate` (batch 56aj —
  // the former no-ops dropped the displayed date).
  // datetime.sty:173 `\longdate` is the load-time default (ours is the
  // `\monthname[m] d, y` approximation: no day-of-week/ordinal); :459 `\setdefaultdate`
  // and the :461-490 options select another (the `\dt@addtoextras` bookkeeping
  // is fmtcount presentation). Formerly the binding defaulted to `d/m/y` with
  // the options ignored (batch 56aj).
  RawTeX!(
    r"\newcommand*{\setdefaultdate}[1]{#1}\longdate
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
\DeclareOption{iso}{\setdefaultdate{\yyyymmdddate}}
\DeclareOption{level}{}\DeclareOption{raise}{}\DeclareOption{dayofweek}{}\DeclareOption{nodayofweek}{}
\DeclareOption{nodate}{}\DeclareOption{hhmmss}{}\DeclareOption{24hr}{}\DeclareOption{12hr}{}\DeclareOption{oclock}{}
\ProcessOptions\relax"
  );

  RawTeX!(
    r"\newcommand*{\newdate}[4]{\@ifundefined{date@#1@y}{\@namedef{date@#1@d}{#2}\@namedef{date@#1@m}{#3}\@namedef{date@#1@y}{#4}}{\PackageError{datetime}{Date `#1' already defined}{}}}
\newcommand*{\getdateyear}[1]{\@ifundefined{date@#1@y}{\PackageError{datetime}{Date `#1' not defined}{}}{\csname date@#1@y\endcsname}}
\newcommand*{\getdatemonth}[1]{\@ifundefined{date@#1@m}{\PackageError{datetime}{Date `#1' not defined}{}}{\csname date@#1@m\endcsname}}
\newcommand{\getdateday}[1]{\@ifundefined{date@#1@d}{\PackageError{datetime}{Date `#1' not defined}{}}{\csname date@#1@d\endcsname}}
\newcommand*{\displaydate}[1]{\@ifundefined{date@#1@y}{\PackageError{datetime}{Date `#1' not defined}{}}{\formatdate{\csname date@#1@d\endcsname}{\csname date@#1@m\endcsname}{\csname date@#1@y\endcsname}}}"
  );
});

use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "datetime.sty",
    "datetime.sty is only minimally stubbed and will not be interpreted raw."
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
  // Companion format setters as no-ops.
  def_macro_noop("\\settimeformat{}")?;
  DefMacro!("\\formattime{}{}{}", "#1:#2:#3");
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

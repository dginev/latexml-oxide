use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "datetime.sty",
    "datetime.sty is only minimally stubbed and will not be interpreted raw."
  );

  // datetime.sty L181-188 `\newdateformat{name}{def}` creates a date-
  // format command. Stub as no-op — we don't render datetime
  // distinctly so author's custom format is moot. Witness cluster:
  // arXiv:2506.21718 / 2507.03037 — Rust 4 → 0, beats Perl=0
  // (REAL REGRESSION → BOTH CLEAN).
  def_macro_noop("\\newdateformat{}{}")?;
  // Companion format setters as no-ops.
  def_macro_noop("\\settimeformat{}")?;
  // \formatdate{day}{month}{year} — emit as plain numeric date.
  // Round-34 surpass-Perl: was gobbled; preserve content inline.
  DefMacro!("\\formatdate{}{}{}", "#1/#2/#3");
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

  // datetime.sty L260+ `\newdate{name}{day}{month}{year}` declares a
  // named date that `\displaydate{name}` later prints. Real package
  // stores components in `\<name>@day`/`\<name>@month`/`\<name>@year`.
  // Stub each as no-op.
  def_macro_noop("\\newdate{}{}{}{}")?;
  def_macro_noop("\\displaydate{}")?;
});

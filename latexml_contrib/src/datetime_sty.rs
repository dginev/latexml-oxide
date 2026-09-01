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

  // datetime.sty L260+ `\newdate{name}{day}{month}{year}` declares a
  // named date that `\displaydate{name}` later prints. Real package
  // stores components in `\<name>@day`/`\<name>@month`/`\<name>@year`.
  // Stub each as no-op.
  def_macro_noop("\\newdate{}{}{}{}")?;
  def_macro_noop("\\displaydate{}")?;
});

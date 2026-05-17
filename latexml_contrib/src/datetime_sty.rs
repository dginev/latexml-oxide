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
  DefMacro!("\\newdateformat{}{}", "");
  // Companion format setters as no-ops.
  DefMacro!("\\settimeformat{}", "");
  // \formatdate{day}{month}{year} — emit as plain numeric date.
  // Round-34 surpass-Perl: was gobbled; preserve content inline.
  DefMacro!("\\formatdate{}{}{}", "#1/#2/#3");
  DefMacro!("\\formattime{}{}{}", "#1:#2:#3");
  // Date-component stubs (some packages call directly).
  DefMacro!("\\monthname[]", "");
  DefMacro!("\\shortmonthname[]", "");
});

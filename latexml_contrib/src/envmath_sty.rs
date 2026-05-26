//! envmath.sty (F. Bosisio) — extended math environments
//! (`Equation`, `MultiLine`, `System`, `EQNarray`, `EqSystem`,
//! `Cases`, ...).
//!
//! Raw-loading hits a `\catcode`\&=4` / `\OneShot@Amper`
//! pattern that interacts badly with our alignment-template
//! tracking — papers using envmath's auto-tab system run into
//! a PushbackLimit infinite loop (witness arXiv:1501.05259:
//! amsmath `cases` env + envmath redefining `&` → 650K-token
//! pushback runaway). Perl LaTeXML skips raw-load
//! (INCLUDE_STYLES=false), 10 warnings, conversion complete.
//!
//! Match Perl: stub the public API as no-ops. Map envmath's
//! envs to their amsmath equivalents:
//!   - `Equation` → `equation` (single-line equation)
//!   - `MultiLine` → `multline*` (multi-line equation)
//!   - `EQNarray` → `eqnarray` (alignment array)
//!   - `System` / `EqSystem` → `align*` (system of equations)
//!   - `Cases` → `cases` (case analysis)
//!
//! Lost fidelity: envmath's optional label arg and auto-numbering
//! quirks. Gained: error-free conversion.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "envmath.sty",
    "envmath.sty is minimally stubbed — extended math envs map to their amsmath equivalents."
  );
  // Top-level env aliases. Use Let! to bind both `\Foo` and
  // `\endFoo` indirectly via the macro-expansion text.
  DefMacro!("\\Equation", "\\begin{equation}");
  DefMacro!("\\endEquation", "\\end{equation}");
  DefMacro!("\\MultiLine", "\\begin{multline*}");
  DefMacro!("\\endMultiLine", "\\end{multline*}");
  DefMacro!("\\EQNarray", "\\begin{eqnarray}");
  DefMacro!("\\endEQNarray", "\\end{eqnarray}");
  DefMacro!("\\System", "\\begin{align*}");
  DefMacro!("\\endSystem", "\\end{align*}");
  DefMacro!("\\EqSystem []", "\\begin{align*}");
  DefMacro!("\\endEqSystem", "\\end{align*}");
  DefMacro!("\\Cases", "\\begin{cases}");
  DefMacro!("\\endCases", "\\end{cases}");
  // envmath's internal hooks — accept and ignore args.
  def_macro_noop("\\StartMath@Err{}")?;
  def_macro_noop("\\MakeAmper@Active{}")?;
  def_macro_noop("\\MakeAmper@Tab")?;
  def_macro_noop("\\OneShot@Amper{}{}")?;
});

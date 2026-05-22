//! trace.sty — debug-only macro-tracing wrapper.
//!
//! trace.sty (F. Mittelbach, 1999-2019) wraps `\frozen@everymath`,
//! `\frozen@everydisplay`, `\define@newfont`, `\calc@open`, and
//! `\maybe@ic@` with `\conditionally@traceoff`/`\conditionally@traceon`
//! pairs so that `\traceon`/`\traceoff` can selectively dump
//! `\tracingmacros`-style output during compilation. The package
//! has no semantic effect — its job is to write log lines.
//!
//! Raw-loading trace.sty trips our long-standing `\newtoks` slot-detach
//! bug (see `latexml_core/src/dump_writer.rs` L176-211 for the dump-time
//! workaround). trace.sty's reassignment of `\frozen@everymath = {...
//! \the\everymath}` lands on `\everymath`'s slot, making subsequent
//! math-mode entry expand `\the\everymath` recursively until the
//! conditional limit (8M) is exhausted.
//!
//! Stub the package as a no-op: define the user-visible knobs
//! (`\traceon`, `\traceoff`, `\tracingall`, `\conditionally@traceoff`,
//! `\conditionally@traceon`) as empty macros so user-defined wrappers
//! that call them don't error, then return. The package's diagnostic
//! tracing output is irrelevant for our XML/HTML rendering paradigm.
//!
//! Witnesses: canvas-3 stage-23 0812.0208 (`\documentclass{ptptex}`
//! → OmniBus + trace.sty raw-load → `\maketitle` triggers
//! `\frozen@everymath` loop on the `$^1$` in `\author{... \textsc{Y}$^1$
//! ...}`); minimal 7-line repro likewise fixed.
use crate::prelude::*;

LoadDefinitions!({
  def_macro_noop("\\traceon")?;
  def_macro_noop("\\traceoff")?;
  def_macro_noop("\\tracingall")?;
  def_macro_noop("\\conditionally@traceoff")?;
  def_macro_noop("\\conditionally@traceon")?;
  def_macro_noop("\\unconditionally@traceoff")?;
  def_macro_noop("\\tr@ce@n")?;
});

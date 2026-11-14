//! silence.sty — "Selective filtering of warnings and error messages"
//! (Paul Isambert, v1.5b 2012/07/02; TeX Live
//! `texmf-dist/tex/latex/silence/silence.sty`).
//!
//! No Perl LaTeXML binding exists — neither upstream
//! (`LaTeXML/lib/LaTeXML/Package/silence.sty.ltxml` is absent) nor in the
//! installed 0.8.8 tree — so same-host Perl reports the same
//! `Error:undefined:\WarningFilter` on the witnesses below. Perl only
//! survives when `--includestyles` raw-loads the real .sty; with the raw
//! load unavailable (bare mode, or a class whose `\RequirePackage{silence}`
//! never runs) every silence command is undefined.
//!
//! The package exists solely to filter LaTeX's *console* warnings and
//! errors. It contributes **no document content**, so every public command
//! is a no-op here. What matters is the ARITY: an undefined `\WarningFilter`
//! is recovered as a zero-argument `<ltx:ERROR/>`, which leaks its two
//! braced arguments into the document as visible text (witness 2605.05327
//! renders "Extended allocation already in use" in the body). Consuming the
//! arguments is the whole point of the binding.
//!
//! Signatures follow silence.sty verbatim:
//!   * `\SafeMode`, `\BoldMode`                         L71-72
//!   * `\WarningsOn[list]`, `\WarningsOff*[list]`       L90-114
//!   * `\WarningFilter*[family]{package}{message}`      L152-164
//!   * `\ActivateWarningFilters[list]`,
//!     `\DeactivateWarningFilters[list]`                L199-209
//!   * `\ActivateFilters[list]`, `\DeactivateFilters[list]` L231-247
//!   * `\ErrorsOn[list]`, `\ErrorsOff*[list]`           L495-519
//!   * `\ErrorFilter*[family]{package}{message}`        L521-530
//!   * `\ActivateErrorFilters[list]`,
//!     `\DeactivateErrorFilters[list]`                  L567-577
//!
//! The message argument is read `Semiverbatim`: silence itself reads it
//! under `\catcode`\\12` (L186, L554) precisely because filter patterns
//! quote LaTeX messages containing control sequences and `#`.
//!
//! silence also wraps `\PackageWarning` / `\ClassWarning` / `\PackageError`
//! / `\ClassError` / `\@latex@error` / `\GenericError` (L386-431, L433,
//! L581-599) to route them through its filter bank. Those are deliberately
//! NOT touched: LaTeXML's own definitions are what turn them into
//! `Error:`/`Warning:` diagnostics, and re-pointing them at a filter
//! silently downgrades real ones. That is not hypothetical — it is why this
//! binding is registered UNCONDITIONALLY, where the paper-bundled
//! `arxiv_sty.rs` sibling defers to the raw file under `INCLUDE_STYLES`.
//! Measured: `\usepackage{silence}\ErrorsOff` followed by a package that
//! raises `\PackageError` converts at **0 errors** under same-host Perl
//! 0.8.8 `--includestyles` (which raw-loads silence.sty) and at **1** with
//! this binding in front of it. Guard:
//! `107_silence_keeps_diagnostics::silence_errorsoff_does_not_swallow_a_package_error`.
//! The trade is that silence's own filter *state* is not modelled, which
//! costs nothing: LaTeXML has no message bank to filter.
//!
//! Witnesses: 2605.05327 (`\RequirePackage{silence}` before
//! `\documentclass`, aa.cls), 2605.06624 (`\usepackage{silence}` +
//! `\WarningFilter[captions]{caption}{...}` + `\ActivateWarningFilters`),
//! 2504.08779 / 2509.17283 / 2512.12232 (ascelike-new.cls, whose
//! `\RequirePackage{silence}` never runs).
use latexml_package::prelude::*;

LoadDefinitions!({
  // Options: debrief, immediate, safe, save, saveall, showwarnings,
  // showerrors (L38-48). All select filter *reporting* behaviour, which
  // has no analogue here.
  DeclareOption!(None, {});
  ProcessOptions!();

  // L71-72
  def_macro_noop("\\SafeMode")?;
  def_macro_noop("\\BoldMode")?;

  // L90-114. `\WarningsOff` takes an optional star OR an optional list;
  // `\WarningsOn` only the list.
  def_macro_noop("\\WarningsOn []")?;
  def_macro_noop("\\WarningsOff OptionalMatch:* []")?;

  // L152-164 / L521-530. `*` selects "safe" (`\makeatletter`) reading of
  // the message; both modes gobble it here.
  def_macro_noop("\\WarningFilter OptionalMatch:* [] {} Semiverbatim")?;
  def_macro_noop("\\ErrorFilter OptionalMatch:* [] {} Semiverbatim")?;

  // L199-209 / L231-247 / L567-577
  def_macro_noop("\\ActivateWarningFilters []")?;
  def_macro_noop("\\DeactivateWarningFilters []")?;
  def_macro_noop("\\ActivateErrorFilters []")?;
  def_macro_noop("\\DeactivateErrorFilters []")?;
  def_macro_noop("\\ActivateFilters []")?;
  def_macro_noop("\\DeactivateFilters []")?;

  // L495-519
  def_macro_noop("\\ErrorsOn []")?;
  def_macro_noop("\\ErrorsOff OptionalMatch:* []")?;

  // L362 `\def\sl@StoreMessage#1{…}` — the internal message-bank writer.
  // Third parties patch it (hep-font.sty L110 `\robustify\sl@StoreMessage`,
  // then `\pretocmd`), and with the internals absent etoolbox raises
  // `\sl@StoreMessage undefined` (perfect-kernel corpus: the hep-*
  // documentation family, 5+ TL doc bundles). Same one-arg noop as the
  // public surface — there is no message bank to store into.
  def_macro_noop("\\sl@StoreMessage {}")?;
});

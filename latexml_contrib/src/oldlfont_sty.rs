//! oldlfont.sty — old "logical-font" compatibility shim (LaTeX 2.09 era).
//!
//! oldlfont re-establishes pre-NFSS font commands like `\rm`, `\sf`,
//! `\bf`, `\it`, `\tt` as math-aware "robust" commands, and declares
//! symbol-font alphabets for `\mathbf`, `\mathsf`, `\mathtt`,
//! `\mathit`, `\mathsl`, `\mathsc`, `\mit`, `\cal`. The raw .sty does
//! `\let\mathit\undefined` then `\DeclareSymbolFontAlphabet\mathit{italic}`
//! to fully re-attach the alphabet binding.
//!
//! Our engine's `\DeclareSymbolFontAlphabet` support is incomplete:
//! the `\let\mathit\undefined` step succeeds but the re-binding
//! doesn't fully reinstate `\mathit` as a math-alphabet command —
//! subsequent `\mathit{...}` in math mode then errors as "undefined".
//!
//! Perl LaTeXML has no oldlfont binding either; with default
//! `INCLUDE_STYLES=false`, raw oldlfont.sty is not loaded — Perl's
//! existing kernel `\mathit`/`\mathbf`/`\mathsf`/`\mathtt`/`\mit`/
//! `\cal` definitions remain intact.
//!
//! Match Perl: stub as a no-op so the kernel definitions are
//! preserved. The old `\rm`/`\sf`/etc. font commands already exist
//! in latexml_engine's kernel; nothing to add.
//!
//! Witness: 1112.3561 (Colaiuda+Kokkotas neutron-star paper, mn2e
//! class, `\usepackage{oldlfont}` + `$\mathit{...}$` in math) —
//! previously fatal via cascading "\mathit undefined", now should
//! convert cleanly.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "oldlfont.sty",
    "oldlfont.sty is minimally stubbed — kernel \\mathit / \\mathbf / \\mathsf / \\mathtt / \\mit / \\cal definitions are preserved."
  );
  // No further definitions — kernel commands already cover the
  // semantic content oldlfont would re-bind via
  // `\DeclareSymbolFontAlphabet`.
});

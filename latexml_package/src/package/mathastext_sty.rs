//! mathastext.sty — math font substitution.
//!
//! Like fdsymbol.sty, mathastext L1075 runs `\let\lbrace\undefined`
//! before `\DeclareMathDelimiter{\lbrace}`. Our `:locked` guard
//! blocks the redef, leaving `\lbrace` (and `\rbrace`, etc.) UNDEFINED
//! and cascading 500+ math-mode errors per paper.
//!
//! Perl LaTeXML has no mathastext binding and skips it via
//! INCLUDE_STYLES=false. Match with a no-op binding — math-font
//! substitution is moot for XML/HTML output.
//!
//! Witness 2410.03274 (\lbrace cascade with 505 errors).
use crate::prelude::*;

LoadDefinitions!({
  // Intentionally empty: don't run mathastext's math-symbol slot
  // reassignments. The kernel \lbrace / \rbrace / \langle / etc.
  // remain authoritative.
});

//! fdsymbol.sty — alternative math font for primes/arrows/relations.
//!
//! Perl LaTeXML doesn't raw-load fdsymbol.sty: with INCLUDE_STYLES=false
//! (the Perl default), the package is silently skipped and the kernel
//! math chars remain authoritative. Our ar5iv profile raw-loads
//! everything it can find, so fdsymbol.sty runs and trips on:
//!
//!   L111: \if\relax\noexpand#1\let#1\undefined\fi
//!
//! `\if\relax\noexpand\X` is TRUE for any CS (TeX assigns charcode 256
//! to all CSes), so the `\let\X\undefined` fires for every symbol in
//! the package — including kernel `\prime`, `\downarrow`, etc. The
//! subsequent `\DeclareMathSymbol\X{...}` is then blocked by our
//! `:locked` guard (kernel math chars are explicitly locked at
//! `DefMath!` time). Net effect: `\prime`/`\downarrow`/... end up
//! UNDEFINED with the `:locked` flag still set, cascading 500+
//! errors per paper.
//!
//! Match Perl's behaviour: this binding is a no-op so the kernel math
//! chars remain authoritative. Font-substitution is moot for our
//! XML/HTML output. Witness 2406.19499, 2411.08746 — and ~8 cumulative
//! papers across stages 2-6 with hundreds of cascading errors each.
use crate::prelude::*;

LoadDefinitions!({
  // Intentionally empty: we don't apply fdsymbol's font swaps in XML output.
});

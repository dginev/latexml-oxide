//! curve2e.sty — extensions of the `picture` env (Bezier curves,
//! arrows, vector drawings) by Claudio Beccari.
//!
//! curve2e raw-loads into pictex-style territory: `\Dir@line`,
//! `\strokepath`, `\d@mX`, `\d@mY`, `\originalmoveto`,
//! `\pIIe@lineto` and a `\the\edef` pattern that our engine
//! rejects with "You can't use \edef after \the". Witness paper
//! arXiv:1408.2108 — amsart + curve2e (LOADED BUT UNUSED in the
//! body!) → 100+ errors + fatal. Perl converts the same input
//! with 26 warnings.
//!
//! Match Perl: stub the package as a no-op shell so the raw
//! .sty is never loaded. We provide the `picture` extensions as
//! pass-throughs (no fancy curves, but no crashes either). If a
//! paper uses `\Curve` / `\Arc` heavily the visual fidelity
//! suffers, but for the common case (loaded-but-unused) the
//! document converts cleanly.
use latexml_package::prelude::*;

LoadDefinitions!({
  // curve2e.sty:16-17 hard dependencies (graphicx, color, pict2e), then the
  // REAL package raw-loaded: with pict2e's driver-level path builders
  // (`\pIIe@moveto`… in pict2e_sty.rs) and cap/join declarations present,
  // curve2e's vector algebra (xfp `\fpeval`), `\Arc`/`\VectorArc`/`\VectorARC`,
  // `\polyline`/`\Vector`/`\segment`, `\Zbox`/`\Pbox`, `\xmultiput` and the
  // grid commands all run verbatim and render through `\lx@pic@polyline`
  // (curve2e-manual: the 9-stub binding left 32 undefined-command errors;
  // Perl's raw load of the same file, lacking the `\pIIe@*` builders, fails
  // with 102 errors + Fatal). Its `\@picture` redefinition (curve2e.sty:273)
  // is dead code here — the picture environment is a constructor that records
  // `\pict@dimen`/`\pict@offset` itself (sect13.rs, pict2e_sty.rs).
  RequirePackage!("graphicx");
  RequirePackage!("color");
  RequirePackage!("pict2e");
  InputDefinitions!("curve2e", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // curve2e.sty:92-96 snapshots pict2e's `\moveto`/`\lineto`/`\curveto` under
  // `\original*` before wrapping them for the macro-pair `(\P)` form; pin the
  // snapshots to the stable aliases so the wrappers can never resolve to
  // themselves (a `\moveto`→`\originalmoveto`→`\moveto` recursion was seen
  // with the body absorbed verbatim; the user-level primitives accept `(\P)`
  // directly anyway).
  RawTeX!(
    r"\let\originalmoveto\lx@pictii@moveto \let\originallineto\lx@pictii@lineto
\let\originalcurveto\lx@pictii@curveto"
  );
});

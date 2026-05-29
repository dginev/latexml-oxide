//! ctable.sty binding: deps + conditional raw-load.
//!
//! Perl ships no ctable binding and raw-loads the real ctable.sty (it's in TL),
//! so `\ctable[…]{…}{…}{…}` works. We do the same — BUT guarded on tikz being
//! absent, because of a known clash:
//!
//! ctable.sty (TL) does a guarded
//! `\@ifpackageloaded{tikz}{…define \transparent…}{\RequirePackage{transparent}}`
//! at load time, then `\@ifpackageloaded{tikz}{\@ifpackageloaded{transparent}
//! {\PackageError "You must load ctable after tikz"}{}}{}` inside
//! `\AtBeginDocument`. When tikz IS loaded, our raw-load path ends up with both
//! tikz and transparent in scope and fires that error. So for tikz papers we
//! keep the deps-only behavior (no `\ctable`, table dropped — same net effect
//! as Perl when its TEXINPUTS misses ctable). Witnesses: arXiv:1912.08312,
//! 1912.08818, 2001.00802, 2001.05616, 2001.09838, 2001.09978 (tikz+ctable).
//!
//! For the COMMON non-tikz case the earlier pure-deps stub was WRONG: it left
//! `\ctable` undefined, so papers that actually use `\ctable` errored where
//! Perl is clean. Witness arXiv:2011.04706 (`\usepackage{ctable}` +
//! `\ctable[caption=…]{lcccccr}{…}{…}`, no tikz): 3 err → 0. We now raw-load
//! ctable.sty there, defining `\ctable` exactly as Perl does.

use crate::prelude::*;

LoadDefinitions!({
  // Pull in ctable's real dependencies (`\RequirePackage{ifpdf,
  // etoolbox,xcolor,xkeyval,array,tabularx,booktabs,rotating}` —
  // ctable.sty L28). Papers that rely on ctable for its transitive
  // dependencies (the most common being booktabs for `\toprule`/
  // `\midrule`/`\bottomrule`) need this — without it our previous
  // pure-no-op stub silently dropped them. Witness 2002.05708 (loaded
  // ctable, used \toprule/\midrule/\bottomrule from booktabs without
  // a direct \usepackage{booktabs}).
  RequirePackage!("ifpdf");
  RequirePackage!("etoolbox");
  RequirePackage!("xcolor");
  RequirePackage!("xkeyval");
  RequirePackage!("array");
  RequirePackage!("tabularx");
  RequirePackage!("booktabs");
  RequirePackage!("rotating");
  // Raw-load the real ctable.sty so `\ctable[…]{…}{…}{…}` is actually defined
  // (Perl ships no ctable binding and raw-loads it → clean). The documented
  // "You must load ctable after tikz" AtBeginDocument error only fires when
  // tikz is ALSO loaded, so guard the raw-load on tikz being absent. tikz
  // papers keep the deps-only behavior (matching the prior stub).
  // Witness arXiv:2011.04706 (`\usepackage{ctable}` + `\ctable[caption=…]{…}`,
  // no tikz: Perl 0 err; the old deps-only stub left `\ctable` undefined → 3 err).
  if !lookup_bool("tikz.sty_loaded") {
    InputDefinitions!("ctable", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  }
});

//! french.sty (e-french) — Bernard Gaulle's French support, the
//! non-babel cousin of `babel + french.ldf`.
//!
//! Perl LaTeXML has no `french.sty.ltxml` binding, so its package
//! loader emits `Warning:missing_file:french` and never raw-loads
//! `french.sty` from TL. That's effectively a silent no-op —
//! e-french's user-facing impact in our XML-output paradigm is
//! limited to a handful of French captions and small text-mode
//! abbreviations.
//!
//! Rust used to fall through to raw-load `french.sty` from TL,
//! which immediately `\RequirePackage{msg}`. `msg.sty` defines
//! `\def\msgheader{\protect\msgheader}` (and `\msgtrailer`,
//! `\msgencoding`) — a TeX trick that only resolves under
//! `\protected@edef`. Our eager-expand model trips the
//! self-recursion guard and the conversion records multiple
//! `Error:recursion` for those CSes. Perl never gets there.
//!
//! Mirror Perl's effective outcome by intercepting `french.sty`
//! here: delegate to `french_ldf` for the user-facing macros and
//! caption strings, and skip the raw-load entirely. Witnesses:
//! `math9903002`, `gr-qc9511021`, `alg-geom9611022`, `math9807030`,
//! `math9807060`, `math9803069`, `math9810103`, `math9810088`.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Delegate to the babel-style french binding for captions and
  // ordinals (\up, \fup, etc.). This is what Perl would have done
  // for `\usepackage[french]{babel}`, and is acceptable here because
  // e-french and babel-french share the same user-visible vocabulary
  // — only the internal locale-switch / message-table plumbing
  // differs, and neither matters for HTML/XML output.
  RequirePackage!("french", extension => Some(Cow::Borrowed("ldf")));
});

//! No-op stub for autofe.sty (ucs's "Automatic switching of font
//! encodings").
//!
//! autofe.sty raw-loads `\RequirePackage[LGR]{fontenc}` (+ T2A + T1)
//! and rewrites `\DeclareTextCommand`/`\DeclareTextSymbol` to auto-
//! select a font encoding per character. Activating LGR (Greek font
//! encoding) leaves Latin letters transliterated to Greek, which then
//! leaks into CS-name building: e.g. `\thesection` becomes the
//! undefined `\theςεςτιον` ("section" letter-for-letter in Greek).
//!
//! Perl LaTeXML never observes this: `\usepackage[utf8x]{inputenc}`
//! pulls in ucs.sty/autofe.sty, but ucs is reported missing-file in
//! Perl's default TEXINPUTS so autofe never loads — utf8 decoding still
//! happens via inputenc's plain utf8 path. Verified 1701.05945: Perl
//! converts cleanly (`missing files[ucs.sty]`), our raw-load of autofe
//! produced `\theςεςτιον`/`\theςυβςεςτιον` undefined.
//!
//! Font-encoding auto-selection is a glyph-rendering concern with no
//! bearing on our XML/MathML output (we already decode utf8 input via
//! utf8.def), so a no-op stub is the faithful match to Perl's
//! effective behavior. Witness 1701.05945 (`[utf8x]{inputenc}`).

use latexml_package::prelude::*;

LoadDefinitions!({});

//! ifacconf.cls (IFAC conference proceedings) — intentionally has **no
//! binding**: we raw-load the real `ifacconf.cls` exactly like Perl.
//!
//! Perl LaTeXML ships no `ifacconf.cls.ltxml`. When a paper supplies its own
//! `ifacconf.cls` (the class is not in TeX Live, so papers must), Perl raw-loads
//! it under the OmniBus skeleton and scans its real dependencies
//! (`theorem, newlfont, natbib` — notably **not** hyperref). The class then
//! defines its own frontmatter (`\author`, `\address`, `\sep`, `\thanks`, …).
//! We now mirror that: this module is unregistered, so a real `ifacconf.cls`
//! falls through `find_class` → raw-load (OmniBus skeleton), matching Perl
//! byte-for-structure. Task #273 (shrink OmniBus stubs via raw interpretation).
//!
//! This replaces an earlier Elsevier-style stub (`LoadClass("OmniBus")` plus
//! eager `RequirePackage("amsmath"/"amssymb"/"amsthm"/"hyperref"/"graphicx"/
//! "natbib")` and hand-rolled `\sep`/`\ead`/`\fnref`/… helpers). The stub's
//! eager `\RequirePackage{hyperref}` was a Perl divergence: it bound `\url` to
//! hyperref's verbatim reader, so a `.bbl`'s `\providecommand{\url}[1]{...}`
//! (and a nested `\url{\url{...}}`) no longer collapsed to `\texttt`, producing
//! a group/mode-frame cascade (38 errors vs Perl 0). Witness arXiv:1611.06249.
//! The real class's own frontmatter is a strictly more faithful match than the
//! Elsevier guess, and raw-load converts 1611.06249 with **zero** errors.
//!
//! NOTE (eager-xcolor cluster — referenced from the other journal `*_cls.rs`):
//! do **not** eagerly `RequirePackage("xcolor")` from a class binding when Perl
//! ships none. Preloading xcolor means a later document-level
//! `\usepackage[table]{xcolor}` (or any `\usepackage[<opts>]{xcolor}`) sees
//! xcolor as already-loaded and silently drops its options — so the `table`
//! option never fires `\RequirePackage{colortbl}`, `array` is never pulled, and
//! `m{<frac>\textwidth}` / `b{}` column types become "Unrecognized tabular
//! template" → cascading "Extra alignment tab" errors. The document loads
//! xcolor itself when it needs it (with the right options). Witness
//! arXiv:2004.03970 (`\usepackage[table]{xcolor}` + `m{0.10\textwidth}` table →
//! 8 errors in Rust, 0 in Perl). The same lesson applies to any eager
//! `RequirePackage` of a package Perl leaves to the document.

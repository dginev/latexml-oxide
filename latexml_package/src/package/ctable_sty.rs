//! No-op stub for ctable.sty.
//!
//! Why a stub instead of raw-load: ctable.sty (TL) does a guarded
//! `\@ifpackageloaded{tikz}{...define \transparent...}{\RequirePackage{transparent}}`
//! at package-load time, then a second
//! `\@ifpackageloaded{tikz}{\@ifpackageloaded{transparent}{\PackageError...}{}}{}`
//! check inside `\AtBeginDocument`. Our raw-load path takes the
//! `\RequirePackage{transparent}` branch (the tikz-vs-non-tikz check
//! at *raw-load time* doesn't fire as expected for one of several
//! sub-state reasons), then the AtBeginDocument check sees BOTH tikz
//! and transparent loaded and fires `Package ctable Error: You must
//! load ctable after tikz`.
//!
//! Perl LaTeXML never observes this because its default TEXINPUTS
//! doesn't include `/usr/share/texlive`, so ctable.sty is reported as
//! missing-file and skipped. Verified on arXiv:1912.08312
//! (`\usepackage{tikz}\usepackage{ctable}` — same load-order as our
//! affected papers): Perl emits `Warning:missing_file:ctable Can't
//! find binding for package ctable` and completes the conversion
//! cleanly.
//!
//! Match Perl's *effective* behavior (no-op, file missing) by
//! registering a no-op binding. The 6 R-stage papers blocked on this
//! error were checked: NONE actually invoke `\ctable[...]{...}{...}{...}`
//! in their body, so the stub costs no document content. If a future
//! paper does use `\ctable`, the table would be lost the same way it
//! is in Perl LaTeXML today.
//!
//! Witness: arXiv:1912.08312, 1912.08818, 2001.00802, 2001.05616,
//! 2001.09838, 2001.09978 (all CONVERR_1 → OK).

use crate::prelude::*;

LoadDefinitions!({});

//! fancyvrb-ex.sty — the "example environments" extension to fancyvrb
//! (`Example`, `CenterExample`, `SideBySideExample` and the pstricks-aware
//! `P…` variants). The manual of every fancyvrb-using package — including
//! mhchem's own `mhchem.tex` — leans on these to show source-and-result.
//!
//! Perl LaTeXML ships no `fancyvrb-ex.sty.ltxml`, and at the default `notex`
//! setting it does not raw-load the real file either, so a document using
//! `\usepackage{fancyvrb-ex}` is parity (both engines leave the environments
//! undefined and error). This contrib binding goes one step *beyond* Perl by
//! raw-loading the genuine TL `fancyvrb-ex.sty`: it is a classic `\def`-based
//! package whose only hard dependencies (`fancyvrb`, `xcolor`) are themselves
//! bound, so the real example machinery (verbatim capture → `\jobname.tmp`
//! write → `\input` of the rendered result) runs unmodified. The `hcolor` /
//! `hbaw` / `pstricks` `\RequirePackage`s are guarded behind package
//! conditionals that are false by default, so they do not fire.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("fancyvrb-ex", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});

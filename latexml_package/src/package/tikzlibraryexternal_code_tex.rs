//! tikzlibraryexternal.code.tex — TikZ picture externalization.
//!
//! The real library is loaded; only its operating MODE is changed. With
//! `\tikzexternalize` in its default `convert with system call` mode every
//! `tikzpicture` is intercepted, the external file is absent, the shell
//! escape (`\write18`, inert here) yields nothing, and
//! tikzexternalshared.code.tex:1636 raises "the system call … did NOT
//! result in a usable output file" — then typesets the picture inline
//! anyway. TeX Live builds these manuals with `-shell-escape`; a plain run
//! and Perl LaTeXML raise the same error per picture (tikzviolinplots 591,
//! causets 106, tilings 80, tikz-feynhand 55, tikzscale 5). tikz's own
//! `mode=graphics if exists` (tikzexternalshared.code.tex:137) is the
//! faithful no-shell-escape behaviour: use the graphics file when present,
//! otherwise typeset inline with no system call. Guard:
//! `perfect_kernel_batch56::tikz_externalize_typesets_inline_without_a_system_call`.
use crate::prelude::*;

LoadDefinitions!({
  InputDefinitions!(
    "tikzlibraryexternal.code",
    extension => Some(Cow::Borrowed("tex")),
    noltxml => true
  );
  RawTeX!(r"\tikzset{/tikz/external/mode=graphics if exists}");
});

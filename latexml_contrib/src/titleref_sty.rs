//! titleref.sty — cross-reference titles (Donald Arseneau, 2001).
//!
//! titleref provides `\titleref{label}` to cross-reference the TITLE
//! of a section/caption (rather than its number, like `\ref{label}`).
//! Implementation details:
//!  * Redefines `\label` to wrap `\@currentlabel` in a
//!    `\TR@TitleReference{<number>}{<title>}` capture during `\edef`.
//!  * Redefines `\@caption`, `\@sect`, etc. to also stash the title.
//!
//! Raw-loading titleref.sty into our engine is fragile: titleref's
//! `\protected@edef\@currentlabel{\protect\TR@TitleReference
//! {\@currentlabel}{\TR@currentTitle}}` interacts badly with our
//! `\caption`/`\label` machinery — `\TR@currentTitle` may be unset and
//! `\@currentlabel` may contain `\the<counter>` tokens that the edef
//! processes incorrectly, surfacing "You can't use `}` after `\the`"
//! cascades that swamp `\caption` (driver: 1103.2227).
//!
//! Provide a minimal stub that:
//!  * Defines `\titleref` as a thin alias for `\ref` (the next-best
//!    behavior for HTML output — we lose the actual section-title text
//!    but produce a working cross-reference).
//!  * Stubs `\theTitleReference`, `\currenttitle`, and the package
//!    options as harmless no-ops.
//!  * Does NOT redefine `\label`, `\@caption`, or any kernel
//!    sectioning command.
//!
//! Matches Perl's effective behavior: Perl LaTeXML also has no
//! titleref binding; with the default `INCLUDE_STYLES=false` Perl
//! emits "missing binding" and the raw `.sty` is not loaded. Our stub
//! produces equivalent semantics (cross-refs work, title text is the
//! number).
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "titleref.sty",
    "titleref.sty is minimally stubbed — \\titleref behaves like \\ref (number, not title)."
  );
  // `\titleref{label}` — fall back to `\ref{label}` so cross-refs work.
  DefMacro!("\\titleref{}", "\\ref{#1}");
  // `\theTitleReference{num}{title}` — formatting helper; default shows title.
  // Kept as a 2-arg passthrough that returns the title (Perl's titleref
  // default).
  DefMacro!("\\theTitleReference{}{}", "#2");
  // `\currenttitle` — title of current section; we don't track titles,
  // so emit a noop placeholder.
  def_macro_noop("\\currenttitle")?;
  // No-op the option machinery. titleref accepts `[usetoc]` and
  // `[nostar]` options that toggle implementation strategies; both
  // become irrelevant with our stub.
});

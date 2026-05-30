//! icml2019.sty — paper-bundled ICML 2019 conference style.
//!
//! Perl ships **no** icml2019 binding: it raw-loads the paper's bundled
//! icml2019.sty, which defines the conference machinery AND a long tail of
//! per-paper author-status markers via `\newcommand` (`\icmlInternship`,
//! `\airesident`, `\icmlNiantic`, `\icmlOutsideContribution`, …) each carrying
//! the paper's *specific* institution/notice text, plus the
//! `\toptitlebar`/`\bottomtitlebar` title rules.
//!
//! The shared `icml_support` binding instead *intercepts* the bundled .sty, so
//! none of those paper-specific defs ever run; it then accreted a generic
//! per-marker fallback (`\icmlInternship → "…during an internship"`, etc.) for
//! every marker hit in the corpus — a stopgap its own comments flag, and one
//! that loses each paper's real text and re-breaks on the next unseen marker
//! (`\icmlNiantic`, witness 1906.04409).
//!
//! Mirror Perl instead: raw-load the bundled icml2019.sty (`noltxml`) so EVERY
//! paper-specific marker survives verbatim, THEN load `icml_support` to
//! re-apply our surpass-Perl frontmatter overrides (`\icmltitle → \title`,
//! `\icmlaffiliation → ltx:note`, `\icmltitlerunning → ltx:toctitle`, …) on top
//! of the raw layout macros. The raw .sty loads cleanly under our engine
//! (verified: 0 errors, `\icmlNiantic` renders), matching Perl. Witness
//! 1906.04409 (`\icmlNiantic`: RUST 1 → 0 = Perl).
use crate::prelude::*;

LoadDefinitions!({
  // Raw-load the paper-bundled icml2019.sty exactly as Perl does (noltxml
  // bypasses this very binding — no recursion — and reads the real file from
  // the document directory). This installs all per-paper `\newcommand` author
  // markers with their genuine institution/notice text.
  InputDefinitions!("icml2019", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // Re-apply our surpass-Perl frontmatter capture on top of the raw layout
  // macros. icml_support's `DefMacro`/`Let` override the raw `\icmltitle`,
  // `\icmlauthorlist`, `\icmladdress`, … with frontmatter-emitting versions;
  // the raw per-paper markers it does NOT name (e.g. `\icmlNiantic`) survive.
  RequirePackage!("icml_support");
});

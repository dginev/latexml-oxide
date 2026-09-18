//! directory.sty — address-book style directories (the bibliography end).
//!
//! `directory.sty:235` defines `\directory[ext]{bibs}` to write `\bibdata` to
//! the `.aux` and `\@input@{\jobname.ext}` — a `.bbl` only a bibtex run with
//! the package's `.bst` produces, which LaTeXML never has; the `.bib` list is
//! stringified into that `\write` and never re-read, so the directory
//! vanished at 0 errors (Perl raw-loads the same package and loses it too, at
//! 4 errors). A directory is by definition the complete address list — the
//! manual itself calls `\nodir{*}` (directory.sty:321, the package's
//! `\nocite`) — so the command is the kernel's `\lx@bibliography` over all
//! entries (`latex_constructs.pool.ltxml:3942`, `:4214` for `\nocite`), the
//! same interception Perl makes for bibunits (OXIDIZED_DESIGN_DIVERGENCES
//! #87). The extension argument selects the `.bbl` and has no LaTeXML role.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  let opts: Vec<String> = lookup_vecdeque("opt@directory.sty")
    .map(|v| v.iter().map(|o| o.to_string()).collect())
    .unwrap_or_default();
  InputDefinitions!("directory", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true, options => opts);
  DefMacro!("\\directory [Default:bbl] {}", "\\nocite{*}\\lx@bibliography{#2}");
});

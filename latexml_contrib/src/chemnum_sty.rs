//! chemnum.sty — comprehensive numbering of chemical compounds
//! (`\cmpd`, `\refcmpd`, `\initcmpd`, `\setchemnum`, ...).
//!
//! Heavy expl3/xparse-based code: 20+ `\NewDocumentCommand`s and
//! hundreds of `\cs_new_protected:Npn` declarations. Raw-loading
//! into our engine triggers cascading errors (101 errors + fatal
//! on the witness paper) because our expl3 support doesn't model
//! enough of the `\tl_*` / `\msg_*` / `\keys_*` plumbing.
//!
//! Witness arXiv:2103.03138 — loads `chemfig + chemnum` but
//! never invokes a single chemnum CS. Perl LaTeXML has no
//! chemnum binding (INCLUDE_STYLES=false skips), single
//! "missing binding" warning, conversion completes.
//!
//! Match Perl: stub the public API as no-ops so the raw .sty is
//! never loaded. We lose actual compound numbering; for papers
//! that USE `\cmpd{name}` the printed labels are gone, but the
//! document body still renders.
use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!(
    "missing_file",
    "chemnum.sty",
    "chemnum.sty is minimally stubbed — chemical-compound numbering API is no-op'd."
  );
  // Public chemnum API surface — consume args silently.
  DefMacro!("\\cmpd OptionalMatch:* [] {}", "");
  DefMacro!("\\refcmpd [] {}", "");
  DefMacro!("\\labelcmpd [] {}", "");
  DefMacro!("\\initcmpd [] {}", "");
  DefMacro!("\\resetcmpd []", "");
  DefMacro!("\\setchemnum {}", "");
  DefMacro!("\\setcmpdproperty {} {} {}", "");
  DefMacro!("\\setcmpdlabel {} {}", "");
  DefMacro!("\\newcmpdcounterformat {} {}", "");
  DefMacro!("\\replacecmpd OptionalMatch:* {}", "");
  DefMacro!("\\cmpdprintlabelid {}", "");
  DefMacro!("\\cmpdshowlabelmargin {}", "");
  DefMacro!("\\cmpdshowlabelinline {}", "");
  DefMacro!("\\chemnumshowdef {}", "");
  DefMacro!("\\chemnumshowref {}", "");
  DefMacro!("\\cmpdshowdef {}", "");
  DefMacro!("\\cmpdshowref {}", "");
  DefMacro!("\\subcmpdshowdef {} {}", "");
  DefMacro!("\\subcmpdshowref {} {}", "");
});

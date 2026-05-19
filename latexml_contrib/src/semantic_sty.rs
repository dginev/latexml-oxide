//! Stub for semantic.sty (proof tree / inference rules + ligatures).
//!
//! semantic.sty L?: \TestForConflict{\@ifnext, ...} checks if our LaTeX
//! kernel-defined `\@ifnext` is already defined, then errors via
//! \@ifdefinable with the "redefined" message. Our LaTeXML kernel
//! pre-binds \@ifnext (latex_constructs.rs: Let!("\\@ifnext",
//! "\\@ifnextchar")), so semantic.sty bails with
//! `Package Semantic Error: The 1 command(s) listed above have been
//! redefined.` Pre-undefine those CSes so semantic's TestForConflict
//! check passes silently. Witness 2403.04708.
use latexml_package::prelude::*;

LoadDefinitions!({
  // Pre-undefine our LaTeX-kernel \@ifnext etc so semantic.sty's
  // TestForConflict check in L?:
  //   \TestForConflict{\@dropnext,\@ifnext,\@ifn,\@ifNextMacro,\@ifnMacro}
  // passes silently. After raw-load, semantic.sty will define its own
  // \@ifnext (which is the same shape as \@ifnextchar anyway).
  state::assign_meaning(&T_CS!("\\@ifnext"), latexml_core::common::store::Stored::None, Some(Scope::Global));
  state::assign_meaning(&T_CS!("\\@dropnext"), latexml_core::common::store::Stored::None, Some(Scope::Global));
  state::assign_meaning(&T_CS!("\\@ifn"), latexml_core::common::store::Stored::None, Some(Scope::Global));
  state::assign_meaning(&T_CS!("\\@ifNextMacro"), latexml_core::common::store::Stored::None, Some(Scope::Global));
  state::assign_meaning(&T_CS!("\\@ifnMacro"), latexml_core::common::store::Stored::None, Some(Scope::Global));

  // Now let semantic.sty load raw.
  InputDefinitions!("semantic", extension => Some(Cow::Borrowed("sty")), noltxml => true);
});

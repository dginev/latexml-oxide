//! Stub for semantic.sty (proof tree / inference rules + ligatures).
//!
//! semantic.sty L?: \TestForConflict{\@ifnext, ...} checks if any of
//! those CSes are already defined, then errors via \@ifdefinable with
//! the "redefined" message: `Package Semantic Error: The N command(s)
//! listed above have been redefined.` Pre-undefine them so semantic's
//! TestForConflict check passes silently. Witness 2403.04708.
//! NOTE: upstream #2833 removed the kernel `Let('\@ifnext','\@ifnextchar')`
//! alias, so `\@ifnext` is no longer kernel-bound — its pre-undefine below
//! is now a defensive no-op (kept for robustness alongside the others).
use latexml_package::prelude::*;

LoadDefinitions!({
  // Pre-undefine our LaTeX-kernel \@ifnext etc so semantic.sty's
  // TestForConflict check in L?:
  //   \TestForConflict{\@dropnext,\@ifnext,\@ifn,\@ifNextMacro,\@ifnMacro}
  // passes silently. After raw-load, semantic.sty will define its own
  // \@ifnext (which is the same shape as \@ifnextchar anyway).
  assign_meaning(&T_CS!("\\@ifnext"), Stored::None, Some(Scope::Global));
  assign_meaning(&T_CS!("\\@dropnext"), Stored::None, Some(Scope::Global));
  assign_meaning(&T_CS!("\\@ifn"), Stored::None, Some(Scope::Global));
  assign_meaning(&T_CS!("\\@ifNextMacro"), Stored::None, Some(Scope::Global));
  assign_meaning(&T_CS!("\\@ifnMacro"), Stored::None, Some(Scope::Global));

  // Now let semantic.sty load raw.
  InputDefinitions!("semantic", extension => Some(Cow::Borrowed("sty")), noltxml => true);
});

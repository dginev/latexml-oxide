//! uspatent.cls — LaTeX class for United States patent applications.
//!
//! uspatent builds on memoir.cls and redefines `\maketitle` (uspatent.cls:188-191)
//! to run `\patentTitlePage` and `\patentStart`. `\patentStart` (:232-267) sets up
//! sectioning, page headers, and defines `\newcounter{parnum}` (used by `\patentParagraph`).
//! In latexml-oxide, kernel `\maketitle` is locked (latex_constructs/sect05.rs:944) so
//! raw redefinitions are ignored, leaving `\patentStart` uncalled and `parnum` undefined
//! (`undefined:\theparnum`, `undefined:counter:parnum` in PatentApplication.tex).
//!
//! Unlocking `\maketitle` allows uspatent's redefinition to take effect so that
//! `\patentStart` runs when `\maketitle` is called.

use latexml_package::prelude::*;

LoadDefinitions!({
  assign_value("\\maketitle:locked", false, Some(Scope::Global));
  InputDefinitions!("uspatent", noltxml => true, extension => Some(Cow::Borrowed("cls")));
});

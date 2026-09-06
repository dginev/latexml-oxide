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
//! `\patentStart` runs when `\maketitle` is called. This per-class opt-in is the
//! mechanism: a generic raw replay of any dropped `\maketitle` body (Gemini
//! round 7, reverted at merge) ran resphilosophica.cls:331's body against the
//! amsart BINDING, which lacks `\@setcopyright`/`\andify`/`\@maketitle@hook`.
//! A class owns `\maketitle` only when its binding says its body runs in our
//! model (KERNEL_CAPABILITIES, the deferred-frontmatter item).

use latexml_package::prelude::*;

LoadDefinitions!({
  assign_value("\\maketitle:locked", false, Some(Scope::Global));
  InputDefinitions!("uspatent", noltxml => true, extension => Some(Cow::Borrowed("cls")));
});

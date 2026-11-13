//! scrlfile.sty — KOMA-Script's file-hook layer (`\BeforePackage`,
//! `\AfterPackage`, `\AfterClass`, `\AfterAtEndOfPackage`, `\ReplaceInput`,
//! `\BeforeClosingMainAux`, …), since 2021 a thin wrapper over
//! `scrlfile-hook.sty` (scrlfile.sty:47-63), which maps everything onto the
//! kernel's `file/<name>/before|after` and `package/…` hooks
//! (scrlfile-hook.sty:85-230).
//!
//! Loaded RAW. The former approximation (`\AfterPackage` → an
//! `\AtBeginDocument{\@ifpackageloaded…}` conditional, `\BeforePackage`
//! absorbed) predates the load hooks firing around every package/class load
//! (content.rs `use_load_hooks`, batch 54c); with those in place the real
//! package works as written, and the approximation was wrong in both
//! directions: scrbook.cls:5466-5477 pairs `\BeforePackage{hyperref}`
//! (`\let\scr@orig@addchap\@addchap`) with `\AfterPackage{hyperref}`
//! (`\let\@addchap\scr@orig@addchap`) — the absorbed "before" left the
//! "after" restoring an UNDEFINED `\@addchap` (cleanthesis my-thesis,
//! bfh-ci DEMO-BFHThesis: `undefined:\@addchap`), and cnltx-doc.cls:728's
//! deprecated `\AfterPackage!{hyperref}{\RequirePackage{multicol,ragged2e}}`
//! ran at begin-document — real scrlfile honours `!` only under its
//! `withdeprecated` option (scrlfile.sty:64-92); without it xparse reads
//! `!` as the package name and the body runs at once as a plain group, and
//! pdflatex itself reports "Loading a class or package in a group" on
//! cnltx_en (probed 2026-09-02). The 26 cnltx-doc manuals therefore gain
//! their two `{multicols}`/`\RaggedRight` errors back — faithfully; none of
//! them is oracle-clean. Keeping a binding (rather than no
//! file) makes the raw load happen under the default arXiv configuration
//! too, where a bindingless package is skipped. Guard:
//! `perfect_kernel_batch54::scrlfile_before_and_after_package_hooks_fire`.

use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("scrlfile", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});

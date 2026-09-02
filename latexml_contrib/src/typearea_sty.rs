//! typearea.sty — KOMA-Script's page-layout package, raw-interpreted.
//!
//! The former no-op stub (git history 3c9baade57^) predates the engine's
//! ability to raw-load the KOMA chain; with the KOMA classes now raw-loaded
//! (`scrartcl_cls.rs`) the stub actively BREAKS them: scrartcl.cls
//! L2594-2628 skips `\if@areasetadvanced … \fi` and `\@lastdiv` inside
//! the false branch of `\if@scr@emulatestandardclasses`, and an undefined
//! `\if@areasetadvanced` is not a conditional to the skipper (tex.web
//! `pass_text` only counts `if_test` commands), so the inner `\else`
//! terminated the outer skip — "Too many }'s" / "Extra \fi" in pdfTeX
//! terms, `unexpected:}` + `unexpected:fi` here. The real package defines
//! the whole surface (`\typearea`, `\recalctypearea`, `\areaset`,
//! `\storeareas`, the `DIV=`/`BCOR=` keys) and only ever assigns the
//! standard `\textwidth`-family lengths, which the XML output does not
//! observe. Witnesses bohr/bohr_en (`\recalctypearea` via cnltx-doc.cls
//! L190), 1502.06768 (`\areaset`), the `Package scrbase Error: unknown
//! option` cluster (1504.00554, 1504.00666 — `DIV=11` now parses).
//! Registered as a binding so the raw load also happens under the default
//! (arXiv) configuration, where a bindingless `.sty` is only dependency-
//! scanned.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("typearea", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});

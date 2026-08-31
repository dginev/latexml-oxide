//! scrlfile.sty — KOMA-Script's file-hook layer (`\AfterPackage`,
//! `\AfterClass`, …), since 2021 a thin wrapper over `scrlfile-hook.sty`
//! (scrlfile.sty L47-63 just requires it).
//!
//! Real signatures (scrlfile-hook.sty): `\AfterPackage s m o +m` (L209-226) —
//! star = run NOW if the package is already loaded, else defer to its load
//! hook; no star = always defer to the load hook. `\AfterClass` (L189-208)
//! likewise with classes.
//!
//! Our approximation: the immediate star branch is exact
//! (`\@ifpackageloaded`); the defer branch becomes a begin-document
//! conditional — `\AtBeginDocument{\@ifpackageloaded{pkg}{code}{}}` — which
//! preserves the load-gated semantics (code never runs when the package never
//! loads) at the cost of running at begin-document rather than at load time.
//! `\BeforePackage`/`\BeforeClass` cannot be honored retroactively and
//! running their setup AFTER the load could double-patch, so they absorb
//! their arguments.
//!
//! Sweep-11 root cause (36-doc `undefined:\AfterPackage` cluster): with
//! `\AfterPackage` undefined, cnltx-doc.cls L728's `\AfterPackage!{hyperref}
//! {…\RequirePackage{multicol,ragged2e}…}` degraded into a plain TeX group —
//! the packages loaded INSIDE it, their definitions died at the closing
//! brace while the global `_loaded` flags survived, and the one undefined
//! hook avalanched into `{multicols}`/`\RaggedRight`/`\cnltx@tableofcontents`
//! undefineds downstream (witness bohr/bohr_en log L354-541).

use latexml_package::prelude::*;

LoadDefinitions!({
  // Star and no-star coincide for an ALREADY-loaded target: the no-star
  // form appends to the one-time `file/…/after` hook, and the kernel runs
  // code added to an already-fired one-time hook immediately — same outcome
  // as the star form's explicit `\@ifpackageloaded` NOW-branch. So one body
  // serves both: run now if loaded, else the begin-document conditional.
  // `!` and `+` are the DEPRECATED prefix forms (scrlfile.sty L65-92):
  // real scrlfile emulates both via `\AfterAtEndOfPackage*` — the same
  // run-when-loaded shape, so they fold into the one body here. cnltx's
  // `\AfterPackage!{hyperref}{…}` (cnltx-doc.cls L728) is the corpus driver.
  DefMacro!(
    "\\AfterPackage OptionalMatch:* OptionalMatch:! OptionalMatch:+ {} [] {}",
    "\\@ifpackageloaded{#4}{#6}{\\AtBeginDocument{\\@ifpackageloaded{#4}{#6}{}}}"
  );
  DefMacro!(
    "\\AfterClass OptionalMatch:* OptionalMatch:! OptionalMatch:+ {} [] {}",
    "\\@ifclassloaded{#4}{#6}{\\AtBeginDocument{\\@ifclassloaded{#4}{#6}{}}}"
  );
  DefMacro!(
    "\\AfterAtEndOfPackage {} [] {}",
    "\\AtBeginDocument{\\@ifpackageloaded{#1}{#3}{}}"
  );
  DefMacro!(
    "\\AfterAtEndOfClass {} [] {}",
    "\\AtBeginDocument{\\@ifclassloaded{#1}{#3}{}}"
  );
  def_macro_noop("\\BeforePackage{}[]{}")?;
  def_macro_noop("\\BeforeClass{}[]{}")?;
  // scrlfile-hook.sty L296/L309: `{o m}` — optional hook label + code
  // deferred to `enddocument/afterlastpage` / `enddocument/afteraux`. Both
  // corpus bodies are pure .aux/.toc write-back (tocbasic.sty L620 writes
  // the toc-file end marker; L2992 writes scr@dte@…maxnumwidth into
  // \@mainaux) — no document content survives into XML, so absorb (same
  // call the atveryend binding makes for \AfterLastShipout). Witness: the
  // 12 toptesi docs via toptesi.sty L50 \RequirePackage{scrextend} →
  // tocbasic.
  def_macro_noop("\\BeforeClosingMainAux[]{}")?;
  def_macro_noop("\\AfterReadingMainAux[]{}")?;
});

//! babel-language stubs for `.ldf` files that aren't installed in
//! minimal TeXLive environments.
//!
//! When babel-italian (or babel-spanish, babel-portuges, …) is
//! missing on disk, babel.sty L4175's `\InputIfFileExists{<lang>.ldf}`
//! fails and babel errors with "Unknown option '<lang>'. Either you
//! misspelled it or the language definition file <lang>.ldf was not
//! found".
//!
//! Each stub here:
//!   - registers via `lib.rs` as `<lang>.ldf` so find_file resolves
//!     to our binding rather than the missing file;
//!   - allocates `\l@<lang>` and defines empty `\captions<lang>` /
//!     `\extras<lang>` / `\noextras<lang>` / `\date<lang>` hooks so
//!     babel's `\selectlanguage` chain runs cleanly.
//!
//! The actual ISO-639 → xml:lang mapping happens at
//! `\selectlanguage` time via
//! `babel_support_sty::babel_language_to_iso`. We don't reproduce
//! the per-language typographic conventions (punctuation, dates,
//! etc.) — they're a typesetting-only concern, irrelevant for
//! XML/HTML output.
//!
//! Cluster (recent stages): italian 7, spanish 5, brazil 5, vietnamese
//! 3, portuguese 3, brazilian 2, polish 2, romanian 2, icelandic 2,
//! czech 2, turkish 1, slovene 1, portuges 1, farsi 1, dutch 1,
//! arabic 1 ≈ 38 papers.
use crate::prelude::*;

fn install_lang_stub(lang: &str) -> Result<()> {
  // `\newlanguage` is a TeX macro of the form
  // `\alloc@9\language\chardef\@cclvi`, which delegates to a 5-arg
  // `\alloc@{}{}{}{}{}` macro. So a *raw* `\newlanguage\csname
  // l@<lang>\endcsname` makes `\alloc@` grab `\csname` as the 5th
  // argument (parameter-text token-grabbing does NOT expand `\csname`)
  // and leaves `l@<lang>\endcsname` orphaned in the input stream — the
  // unmatched `\endcsname` then cascades through every following
  // package-load → 100 errors → fatal TooManyErrors abort.
  //
  // Force `\csname...\endcsname` to expand *first* with
  // `\expandafter\newlanguage\csname...`, so `\newlanguage` receives
  // the resolved `\l@<lang>` token directly.
  //
  // (The `\providecommand\captions{lang}{{}}` lines below are correct
  // — Rust `format!` substitutes `{lang}` inline, giving
  // `\providecommand\captionsbrazil{}` etc.)
  let body = format!(
    r"\expandafter\ifx\csname l@{lang}\endcsname\relax
      \expandafter\newlanguage\csname l@{lang}\endcsname
    \fi
    \providecommand\captions{lang}{{}}%
    \providecommand\extras{lang}{{}}%
    \providecommand\noextras{lang}{{}}%
    \providecommand\date{lang}{{}}",
    lang = lang
  );
  latexml_core::stomach::raw_tex(&body)?;
  Ok(())
}

pub fn load_italian() -> Result<()>    { install_lang_stub("italian") }
pub fn load_spanish() -> Result<()>    {
  install_lang_stub("spanish")?;
  // babel-spanish-specific `\decimalpoint` — switches decimal separator
  // from `,` (Spanish default) to `.`. We don't render locale-aware
  // numerics; HTML uses `.` by default. No-op preserves intent.
  // Driver 2511.19353 (`\usepackage[spanish]{babel}\decimalpoint`).
  // Also `\decimalcomma` for the reverse direction.
  //
  // Spanish math-operator aliases — historical babel-spanish
  // `\extrasspanish` hook adds the Spanish-language trig function
  // names. Cataluña/Spain convention uses `sen` (seno), `tg`
  // (tangente), `cotg` (cotangente), `cosec` (cosecante) etc. instead
  // of the English/AMS \sin, \tan, \cot, \csc. We install them
  // unconditionally rather than via the `\extras` hook — same
  // outcome for our XML output and avoids the hook-timing complexity.
  // Witness: arXiv:1909.12119 — `Error:undefined:\sen` /
  // `\cotg` / `\tg` / `\arcsen` cluster on `\usepackage[spanish]{babel}`.
  latexml_core::stomach::raw_tex(
    r"\providecommand\decimalpoint{}\providecommand\decimalcomma{}
    \providecommand\sen{\mathop{\mathrm{sen}}\nolimits}
    \providecommand\tg{\mathop{\mathrm{tg}}\nolimits}
    \providecommand\cotg{\mathop{\mathrm{cotg}}\nolimits}
    \providecommand\cosec{\mathop{\mathrm{cosec}}\nolimits}
    \providecommand\arcsen{\mathop{\mathrm{arc\,sen}}\nolimits}
    \providecommand\arctg{\mathop{\mathrm{arc\,tg}}\nolimits}
    \providecommand\arccotg{\mathop{\mathrm{arc\,cotg}}\nolimits}"
  )?;
  Ok(())
}
pub fn load_portuges() -> Result<()>   { install_lang_stub("portuges") }
pub fn load_portuguese() -> Result<()> { install_lang_stub("portuguese") }
pub fn load_brazil() -> Result<()>     { install_lang_stub("brazil") }
pub fn load_brazilian() -> Result<()>  { install_lang_stub("brazilian") }
pub fn load_czech() -> Result<()>      { install_lang_stub("czech") }
pub fn load_polish() -> Result<()>     { install_lang_stub("polish") }
pub fn load_romanian() -> Result<()>   { install_lang_stub("romanian") }
pub fn load_slovene() -> Result<()>    { install_lang_stub("slovene") }
pub fn load_turkish() -> Result<()>    { install_lang_stub("turkish") }
pub fn load_vietnamese() -> Result<()> { install_lang_stub("vietnamese") }
pub fn load_icelandic() -> Result<()>  { install_lang_stub("icelandic") }
pub fn load_arabic() -> Result<()>     { install_lang_stub("arabic") }
pub fn load_dutch() -> Result<()>      { install_lang_stub("dutch") }
pub fn load_farsi() -> Result<()>      { install_lang_stub("farsi") }
pub fn load_hindi() -> Result<()>      { install_lang_stub("hindi") }
pub fn load_latin() -> Result<()>      { install_lang_stub("latin") }
pub fn load_croatian() -> Result<()>   { install_lang_stub("croatian") }

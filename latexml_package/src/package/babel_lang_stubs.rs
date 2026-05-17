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
  let body = format!(
    r"\expandafter\ifx\csname l@{lang}\endcsname\relax
      \newlanguage\csname l@{lang}\endcsname
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
pub fn load_spanish() -> Result<()>    { install_lang_stub("spanish") }
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

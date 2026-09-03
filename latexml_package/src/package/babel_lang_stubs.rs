//! babel-language stubs for `.ldf` files that aren't installed in
//! minimal TeXLive environments.
//!
//! When babel-italian (or babel-spanish, babel-portuges, …) is
//! missing on disk, babel.sty L4175's `\InputIfFileExists{<lang>.ldf}`
//! fails and babel errors with "Unknown option '`<lang>`'. Either you
//! misspelled it or the language definition file `<lang>`.ldf was not
//! found".
//!
//! Each stub here:
//!   - registers via `lib.rs` as `<lang>.ldf` so find_file resolves to our binding rather than the
//!     missing file;
//!   - allocates `\l@<lang>` and defines empty `\captions<lang>` / `\extras<lang>` /
//!     `\noextras<lang>` / `\date<lang>` hooks so babel's `\selectlanguage` chain runs cleanly.
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

/// The stub is a FALLBACK, not a binding of the language: when the real
/// `<lang>.ldf` is installed (full TeX Live), raw-load it and return `true`
/// — the registry entry that routes `<lang>.ldf` here would otherwise
/// SHADOW the installed file, skipping its `\DeclareOption{<modifier>}`
/// (spanish.ldf:66-88 `mexico`, `es-noshorthands`) and
/// `\bbl@declare@ttribute` (czech.ldf:328 `split`), so babel's leftover
/// `\DeclareOption{<opt>}{unknown-package-option}` (babel.sty:4214) fired
/// at `\ProcessOptions` (:4300) and `\languageattribute` missed the
/// attribute (:1512-1538) — unamthesis, udepcolor-doc-ES, csbulletin; greek
/// (no stub) already raw-loads clean. Only when the file is absent does the
/// minimal hook set below stand in. Guard:
/// `perfect_kernel_batch54::installed_ldf_outranks_the_language_stub`.
fn install_lang_stub(lang: &str) -> Result<bool> {
  if find_file(
    lang,
    Some(FindFileOptions {
      ext_type: Some(Cow::Borrowed("ldf")),
      ..Default::default()
    }),
  )
  .is_some()
  {
    input_definitions(lang, InputDefinitionOptions {
      noltxml: true,
      extension: Some(Cow::Borrowed("ldf")),
      ..Default::default()
    })?;
    return Ok(true);
  }
  install_lang_stub_hooks(lang)?;
  Ok(false)
}

fn install_lang_stub_hooks(lang: &str) -> Result<()> {
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
  raw_tex(&body)?;
  Ok(())
}

pub fn load_italian() -> Result<()> {
  if install_lang_stub("italian")? {
    return Ok(());
  }
  // italian.ldf:154-171 `\setISOcompliance` and the begin-document `\unit`
  // (verifica.cls:66-70 turns compliance on; `$25\unit{m}$`), :179-180
  // `\IntelligentComma`/`\NoIntelligentComma` (the math-active `,` for
  // decimals is not modelled). Witness verifica example1-5.
  raw_tex(
    r"\newcount\it@ISOcompliance \it@ISOcompliance=\z@
    \providecommand\setISOcompliance{\it@ISOcompliance=\@ne}
    \providecommand\IntelligentComma{}\providecommand\NoIntelligentComma{}
    \AtBeginDocument{\unless\ifnum\it@ISOcompliance=\z@
      \DeclareRobustCommand*{\bbl@it@unit}[1]{\textormath{\,\textup{#1}}{\,\mathrm{#1}}}%
      \@ifpackageloaded{units}{}{\@ifpackageloaded{siunitx}{}{\@ifpackageloaded{SIunits}{}{\let\unit\bbl@it@unit}}}\fi}",
  )?;
  Ok(())
}
// English-family stubs. babel-english.ldf uses `\@namedef{captions
// \CurrentOption}` etc., so each variant gets its own
// `\captions<variant>` / `\date<variant>`. When babel dispatches a
// `\selectlanguage{american}` it expects `\captionsamerican` or the
// `\captionsenglish` fallback. With incomplete raw-load these aren't
// defined and the language-switch errors out (~17 papers in R-stages
// for `\dateUSenglish`, 13 for `\captionsenglish`).
//
// We register the captions/extras/date hooks for the canonical
// english variants (english, american, british, USenglish, UKenglish,
// canadian, australian, newzealand) as no-ops — the variant captions
// (chaptername etc.) just stay English in our HTML output, which is
// already the project's default. Witness:
// arXiv:1502.05791 (`\usepackage[british,american]{babel}`)
// CONVERR_2 → expected OK.
pub fn load_english() -> Result<()> { install_lang_stub("english").map(|_| ()) }
pub fn load_american() -> Result<()> {
  if install_lang_stub("american")? {
    return Ok(());
  }
  install_lang_stub_hooks("USenglish")?;
  install_lang_stub_hooks("english") // fallback chain
}
pub fn load_british() -> Result<()> {
  if install_lang_stub("british")? {
    return Ok(());
  }
  install_lang_stub_hooks("UKenglish")?;
  install_lang_stub_hooks("english")
}
pub fn load_usenglish() -> Result<()> {
  if install_lang_stub("USenglish")? {
    return Ok(());
  }
  install_lang_stub_hooks("english")
}
pub fn load_ukenglish() -> Result<()> {
  if install_lang_stub("UKenglish")? {
    return Ok(());
  }
  install_lang_stub_hooks("english")
}
pub fn load_canadian() -> Result<()> {
  if install_lang_stub("canadian")? {
    return Ok(());
  }
  install_lang_stub_hooks("english")
}
pub fn load_australian() -> Result<()> {
  if install_lang_stub("australian")? {
    return Ok(());
  }
  install_lang_stub_hooks("english")
}
pub fn load_newzealand() -> Result<()> {
  if install_lang_stub("newzealand")? {
    return Ok(());
  }
  install_lang_stub_hooks("english")
}

pub fn load_spanish() -> Result<()> {
  if install_lang_stub("spanish")? {
    return Ok(());
  }
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
  raw_tex(
    r"\providecommand\decimalpoint{}\providecommand\decimalcomma{}
    % spanish.ldf:680 \deactivatetilden (the ~n shorthand is not modelled; gaceta.cls:1612)
    \providecommand\deactivatetilden{}\providecommand\activatetilden{}
    \providecommand\sen{\mathop{\mathrm{sen}}\nolimits}
    \providecommand\tg{\mathop{\mathrm{tg}}\nolimits}
    \providecommand\cotg{\mathop{\mathrm{cotg}}\nolimits}
    \providecommand\cosec{\mathop{\mathrm{cosec}}\nolimits}
    \providecommand\arcsen{\mathop{\mathrm{arc\,sen}}\nolimits}
    \providecommand\arctg{\mathop{\mathrm{arc\,tg}}\nolimits}
    \providecommand\arccotg{\mathop{\mathrm{arc\,cotg}}\nolimits}",
  )?;
  Ok(())
}
pub fn load_portuges() -> Result<()> { install_lang_stub("portuges").map(|_| ()) }
pub fn load_portuguese() -> Result<()> { install_lang_stub("portuguese").map(|_| ()) }
pub fn load_brazil() -> Result<()> { install_lang_stub("brazil").map(|_| ()) }
pub fn load_brazilian() -> Result<()> { install_lang_stub("brazilian").map(|_| ()) }
pub fn load_czech() -> Result<()> { install_lang_stub("czech").map(|_| ()) }
pub fn load_polish() -> Result<()> { install_lang_stub("polish").map(|_| ()) }
pub fn load_romanian() -> Result<()> { install_lang_stub("romanian").map(|_| ()) }
pub fn load_slovene() -> Result<()> { install_lang_stub("slovene").map(|_| ()) }
pub fn load_turkish() -> Result<()> { install_lang_stub("turkish").map(|_| ()) }
pub fn load_vietnamese() -> Result<()> {
  if install_lang_stub("vietnamese")? {
    return Ok(());
  }
  // babel-vietnamese (vietnam.ldf) selects T5 font encoding and defines
  // the Vietnamese precomposed-character command set (`\ecircumflex`,
  // `\ocircumflex`, `\abreve`, `\ohorn`, `\uhorn`, hook-above `\h`, …).
  // vietnam.ldf is NOT installed in TeX Live's base tree, so without
  // this the commands stay undefined when a paper uses
  // `\usepackage[vietnamese]{babel}` with Vietnamese author names.
  // Surpass-Perl: Perl's babel can't find vietnam.ldf either; route
  // through our t5enc binding (mirrors Perl `t5enc.def.ltxml`) — the
  // same set vntex.sty pulls in. Witness 2003.07696
  // (`\usepackage[english,vietnamese]{babel}`, author
  // "Nguy\~\ecircumflex n Th\d{i} B\'ich Th\h{u}y").
  t5enc_def::load_definitions()?;
  Ok(())
}
pub fn load_icelandic() -> Result<()> { install_lang_stub("icelandic").map(|_| ()) }
pub fn load_arabic() -> Result<()> { install_lang_stub("arabic").map(|_| ()) }
pub fn load_dutch() -> Result<()> { install_lang_stub("dutch").map(|_| ()) }
pub fn load_farsi() -> Result<()> { install_lang_stub("farsi").map(|_| ()) }
pub fn load_hindi() -> Result<()> { install_lang_stub("hindi").map(|_| ()) }
pub fn load_latin() -> Result<()> { install_lang_stub("latin").map(|_| ()) }
pub fn load_croatian() -> Result<()> { install_lang_stub("croatian").map(|_| ()) }

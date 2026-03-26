//! babel.sty — multilingual support
//! Perl: babel.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Pre-define language registers (\l@german, \l@french, etc.) so babel's
  // \bbl@iflanguage check passes. Without these, \selectlanguage skips the
  // \captionsgerman/\bbl@switch call because the language isn't "recognized".
  // In Perl's precompiled kernel, these come from language.def hyphenation loading.
  RawTeX!(r"\expandafter\ifx\csname l@english\endcsname\relax\chardef\l@english=0\fi");
  RawTeX!(r"\expandafter\ifx\csname l@german\endcsname\relax\newlanguage\l@german\fi");
  RawTeX!(r"\expandafter\ifx\csname l@ngerman\endcsname\relax\newlanguage\l@ngerman\fi");
  RawTeX!(r"\expandafter\ifx\csname l@french\endcsname\relax\newlanguage\l@french\fi");
  RawTeX!(r"\expandafter\ifx\csname l@spanish\endcsname\relax\newlanguage\l@spanish\fi");
  RawTeX!(r"\expandafter\ifx\csname l@italian\endcsname\relax\newlanguage\l@italian\fi");
  RawTeX!(r"\expandafter\ifx\csname l@portuguese\endcsname\relax\newlanguage\l@portuguese\fi");
  RawTeX!(r"\expandafter\ifx\csname l@russian\endcsname\relax\newlanguage\l@russian\fi");
  RawTeX!(r"\expandafter\ifx\csname l@greek\endcsname\relax\newlanguage\l@greek\fi");
  RawTeX!(r"\expandafter\ifx\csname l@dutch\endcsname\relax\newlanguage\l@dutch\fi");
  RawTeX!(r"\expandafter\ifx\csname l@polish\endcsname\relax\newlanguage\l@polish\fi");
  RawTeX!(r"\expandafter\ifx\csname l@turkish\endcsname\relax\newlanguage\l@turkish\fi");
  RawTeX!(r"\expandafter\ifx\csname l@czech\endcsname\relax\newlanguage\l@czech\fi");

  // Pre-define \bbl@languages as empty. Babel's language.def loading uses \openin
  // to read hyphenation pattern files, which our system can't find (.ini search paths).
  // Without this, \bbl@languages stays undefined, and our error recovery defines it
  // as <ltx:ERROR/> which corrupts babel's list accumulation → infinite recursion → OOM.
  RawTeX!(r"\def\bbl@languages{}");

  InputDefinitions!("babel", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // After babel loads, clear \@fontenc@load@list to prevent stray commas
  // from babel's AtBeginDocument font encoding iteration.
  RawTeX!(r"\def\@fontenc@load@list{}");

  // Fix \bbl@main@language: babel's option processing sets it to "nil"
  // because the ini-based loading path doesn't call \main@language.
  // Read the last option from \@raw@classoptionslist or babel's internal
  // state and call \main@language explicitly.
  // Fix \bbl@main@language: use \bbl@loaded (set by babel option processing)
  // to call \main@language with the correct language name.
  DefPrimitive!("\\lx@babel@fix@mainlang", {
    let main = gullet::do_expand(T_CS!("\\bbl@main@language"))
      .map(|t| t.to_string()).unwrap_or_default();
    if main == "nil" || main.is_empty() {
      // \bbl@loaded contains the last loaded language
      let loaded = gullet::do_expand(T_CS!("\\bbl@loaded"))
        .map(|t| t.to_string()).unwrap_or_default();
      let last_lang = loaded.split(',').map(|s| s.trim())
        .filter(|s| !s.is_empty()).last().unwrap_or("").to_string();
      if !last_lang.is_empty() {
        gullet::unread(Tokenize!(&s!("\\main@language{{{}}}", last_lang)));
      }
    }
  });
  RawTeX!(r"\lx@babel@fix@mainlang");

  // Emulate Perl's precompiled kernel: pre-define \captions<lang> and \date<lang>
  // for common languages. In Perl, `make formats` precompiles the kernel so these
  // macros exist. Without them, babel's \bbl@provide@locale calls \babelprovide
  // which loads .ini files — a path that our engine can't handle (multiple undefined
  // macros hit error recovery → <ltx:ERROR/> corruption → OOM).
  // Pre-defining makes \bbl@provide@locale skip the heavy \babelprovide path.
  RawTeX!(r"\providecommand\captionsenglish{}\providecommand\dateenglish{}");
  RawTeX!(r"\providecommand\captionsfrench{}\providecommand\datefrench{}");
  // German captions (from germanb.ldf) — not empty, actual text.
  // Avoids OOM from \babelprovide AND provides correct localization.
  DefPrimitive!("\\lx@babel@setup@german", {
    RawTeX!(r"\providecommand\captionsgerman{%
      \def\prefacename{Vorwort}\def\refname{Literatur}%
      \def\abstractname{Zusammenfassung}\def\bibname{Literaturverzeichnis}%
      \def\chaptername{Kapitel}\def\appendixname{Anhang}%
      \def\contentsname{Inhaltsverzeichnis}%
      \def\listfigurename{Abbildungsverzeichnis}%
      \def\listtablename{Tabellenverzeichnis}%
      \def\indexname{Index}\def\figurename{Abbildung}%
      \def\tablename{Tabelle}\def\partname{Teil}%
      \def\pagename{Seite}\def\seename{siehe}%
      \def\alsoname{siehe auch}\def\proofname{Beweis}}");
    RawTeX!(r"\providecommand\dategerman{}");
    RawTeX!(r"\providecommand\captionsngerman{\captionsgerman}");
    RawTeX!(r"\providecommand\datengerman{\dategerman}");
  });
  RawTeX!(r"\lx@babel@setup@german");
  RawTeX!(r"\providecommand\captionsspanish{}\providecommand\datespanish{}");
  RawTeX!(r"\providecommand\captionsitalian{}\providecommand\dateitalian{}");
  RawTeX!(r"\providecommand\captionsportuges{}\providecommand\dateportuges{}");
  RawTeX!(r"\providecommand\captionsbrazilian{}\providecommand\datebrazilian{}");
  RawTeX!(r"\providecommand\captionsrussian{}\providecommand\daterussian{}");
  RawTeX!(r"\providecommand\captionspolish{}\providecommand\datepolish{}");
  RawTeX!(r"\providecommand\captionsdutch{}\providecommand\datedutch{}");
  RawTeX!(r"\providecommand\captionsczech{}\providecommand\dateczech{}");
  RawTeX!(r"\providecommand\captionsgreek{}\providecommand\dategreek{}");
  RawTeX!(r"\providecommand\captionsturkish{}\providecommand\dateturkish{}");
  RawTeX!(r"\providecommand\captionshungarian{}\providecommand\datehungarian{}");
  RawTeX!(r"\providecommand\captionsswedish{}\providecommand\dateswedish{}");
  RawTeX!(r"\providecommand\captionsdanish{}\providecommand\datedanish{}");
  RawTeX!(r"\providecommand\captionsfinnish{}\providecommand\datefinnish{}");
  RawTeX!(r"\providecommand\captionsnorsk{}\providecommand\datenorsk{}");
  RawTeX!(r"\providecommand\captionsromanian{}\providecommand\dateromanian{}");
  RawTeX!(r"\providecommand\captionscroatian{}\providecommand\datecroatian{}");
  RawTeX!(r"\providecommand\captionsbulgarian{}\providecommand\datebulgarian{}");
  RawTeX!(r"\providecommand\captionsamerican{}\providecommand\dateamerican{}");
  RawTeX!(r"\providecommand\captionsbritish{}\providecommand\datebritish{}");
  RawTeX!(r"\providecommand\captionsaustrian{}\providecommand\dateaustrian{}");
  RawTeX!(r"\providecommand\captionsnaustrian{}\providecommand\datenaustrian{}");
  RawTeX!(r"\providecommand\captionsfrancais{}\providecommand\datefrancais{}");
});

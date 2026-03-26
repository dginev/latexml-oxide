//! babel.sty — multilingual support
//! Perl: babel.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Pre-define \bbl@languages as empty. Babel's language.def loading uses \openin
  // to read hyphenation pattern files, which our system can't find (.ini search paths).
  // Without this, \bbl@languages stays undefined, and our error recovery defines it
  // as <ltx:ERROR/> which corrupts babel's list accumulation → infinite recursion → OOM.
  RawTeX!(r"\def\bbl@languages{}");

  InputDefinitions!("babel", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // After babel loads, clear \@fontenc@load@list to prevent stray commas
  // from babel's AtBeginDocument font encoding iteration.
  RawTeX!(r"\def\@fontenc@load@list{}");

  // Emulate Perl's precompiled kernel: pre-define \captions<lang> and \date<lang>
  // for common languages. In Perl, `make formats` precompiles the kernel so these
  // macros exist. Without them, babel's \bbl@provide@locale calls \babelprovide
  // which loads .ini files — a path that our engine can't handle (multiple undefined
  // macros hit error recovery → <ltx:ERROR/> corruption → OOM).
  // Pre-defining makes \bbl@provide@locale skip the heavy \babelprovide path.
  RawTeX!(r"\providecommand\captionsenglish{}\providecommand\dateenglish{}");
  RawTeX!(r"\providecommand\captionsfrench{}\providecommand\datefrench{}");
  RawTeX!(r"\providecommand\captionsgerman{}\providecommand\dategerman{}");
  RawTeX!(r"\providecommand\captionsngerman{}\providecommand\datengerman{}");
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

//! babel.sty — multilingual support
//! Perl: babel.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("babel", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // After babel loads, clear \@fontenc@load@list to prevent stray commas
  // from babel's AtBeginDocument font encoding iteration.
  RawTeX!(r"\def\@fontenc@load@list{}");

  // Prevent babel 3.x OOM: \selectlanguage triggers \bbl@provide@locale which
  // calls \babelprovide to load language .ini files. This ini-loading path causes
  // exponential memory growth (14-26GB). Perl avoids this via precompiled kernel
  // where languages are pre-loaded. We emulate this by pre-defining empty
  // \captions<lang> and \date<lang> macros for common languages.
  // \bbl@provide@locale checks \csname date<lang>\endcsname — if defined, skips loading.
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

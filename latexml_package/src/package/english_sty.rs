//! english.sty — legacy english language support, advises babel
//! Perl: english.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // english.sty advises to do \usepackage[english]{babel} instead
  // PassOptions not yet supported; just load babel directly
  RequirePackage!("babel");

  // Raw-load english.ldf to register its `ver@english.ldf` entry and
  // invoke babel's ini-based caption loading path for `en`. The actual
  // \\captionsenglish comes from babel-en.ini via babel's \\babelprovide
  // machinery during option processing (verified 2026-04-18: entries
  // include \\enclname/\\ccname/\\headtoname/\\glossaryname, all from
  // babel-en.ini — not from our previously-hardcoded providecommand
  // stub).
  InputDefinitions!("english", noltxml => true, extension => Some(Cow::Borrowed("ldf")));
});

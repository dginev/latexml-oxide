//! english.sty — legacy english language support, advises babel
//! Perl: english.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl english.sty.ltxml:
  //   PassOptions('babel', 'sty', 'english'); RequirePackage('babel');
  // Rust `\PassOptionsToPackage{english}{babel}` pushes onto the same
  // `opt@babel.sty` state queue that Perl's PassOptions helper uses
  // (verified latex_constructs.rs:3436). Faithful parity.
  raw_tex(r"\PassOptionsToPackage{english}{babel}")?;
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

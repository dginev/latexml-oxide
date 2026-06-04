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

  // babel-english.ldf builds `\date<CurrentOption>` via
  // `\@namedef{date\CurrentOption}` — so loading option `english` only
  // creates `\dateenglish`, NOT the canonical `\dateUSenglish` that
  // modern babel's babel-en.ini machinery then calls (english ≡
  // USenglish). Without it: `Error:undefined:\dateUSenglish` (witness
  // 1503.02002, 1608.02901, 1707.06505, 1808.10359). Bridge the
  // english↔variant aliasing babel expects: point each canonical
  // english-variant date hook at the real `\dateenglish` when undefined
  // (date format is typesetting-only — see babel_lang_stubs.rs — but
  // aliasing keeps `\today` faithful rather than no-op'ing it).
  raw_tex(r"\makeatletter
    \@ifundefined{dateenglish}{\@namedef{dateenglish}{}}{}%
    \@for\bbl@eng:={USenglish,UKenglish,american,british,canadian,australian,newzealand}\do{%
      \@ifundefined{date\bbl@eng}{%
        \expandafter\let\csname date\bbl@eng\endcsname\dateenglish}{}}%
    \makeatother")?;
});

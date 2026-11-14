use latexml_package::prelude::*;

LoadDefinitions!({
  // ifoddpage.sty — `\checkoddpage` + `\ifoddpage`/`\ifoddpageoroneside`.
  //
  // Pure kernel TeX (ifoddpage.sty L23-75: a `checkoddpage` counter, an aux
  // `\oddpage@label` round-trip and `\ifodd\c@page` — no engine primitives),
  // so the real file is interpreted as is in BOTH preload modes. Perl has no
  // binding; without this shim the default mode warned `missing_file` and
  // every `\checkoddpage`/`\ifoddpage` user errored `undefined` (hvfloat.sty
  // L55 requires it; witness hvfloat manual).
  InputDefinitions!("ifoddpage", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});

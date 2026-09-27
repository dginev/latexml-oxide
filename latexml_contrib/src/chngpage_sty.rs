use latexml_package::prelude::*;

LoadDefinitions!({
  // memoir marks chngpage loaded (memoir.cls:12218 `\EmulatedPackage{chngpage}`) without defining
  // its commands — memoir's own `\checkoddpage` stands in — so nothing is loaded under memoir; its
  // list `{adjustwidth}` is replaced below either way (changepage_sty.rs).
  if !(lookup_bool("memoir.cls_loaded") || lookup_bool("memoir.cls_raw_loaded")) {
    // chngpage is changepage's predecessor with its own macros, not an alias: `[strict]` (:21),
    // `\ifcpoddpage` (:32), the `cp@cnt` and `cp@tempcnt` counters (:33-34), `\cplabelprefix`
    // (:35), `\checkoddpage` (:48), `\cplabel{}` (:54).
    InputDefinitions!("chngpage", noltxml => true, extension => Some(Cow::Borrowed("sty")));
    RawTeX!(crate::changepage_sty::ONE_COLUMN_CH_NGETEXT);
  }
  // Its `{adjustwidth}[3][\@empty]` (:113) is a list, as changepage's (changepage_sty.rs): the
  // body in the flow, its paragraph closed before the end, the three arguments read, not typeset.
  DefEnvironment!("{adjustwidth} OptionalUndigested Undigested Undigested", "#body",
    mode => "internal_vertical", before_digest_end => { leave_horizontal()?; });
});

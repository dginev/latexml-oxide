use latexml_package::prelude::*;

LoadDefinitions!({
  // load raw for now.
  InputDefinitions!("tipa", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // tipa.sty internally calls `\RequirePackage[T3,\f@encoding]{fontenc}` to
  // pull in the T3 encoding definitions (`t3enc.def`), which is what
  // defines `\textrhookrevepsilon` / `\textbaru` / etc IPA symbols. Our
  // fontenc binding's `LoadDefinitions!` body only runs once per package,
  // so the already-loaded fontenc (with [T1] options) doesn't re-process
  // options when tipa's `\RequirePackage` re-arrives. Compensate by
  // directly reading `t3enc.def` here. Driver: arXiv:1802.05444.
  InputDefinitions!("t3enc", extension => Some(Cow::Borrowed("def")));
});

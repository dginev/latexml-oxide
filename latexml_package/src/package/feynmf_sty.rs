//! feynmf.sty — Feynman diagrams with MetaFont
//! Perl: feynmf.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("feynmf", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // feynmf {fmfgraph}/{fmfgraph*} environments: 2-arg-on-begin
  // `(width,height)` followed by Feynman diagram body. Real package
  // emits a Metafont diagram. For HTML rendering we drop the graphics
  // body (no Metafont in our pipeline) but preserve the env so the
  // surrounding equation/figure context still parses cleanly. Witness
  // 2309.07343 (15 errors all from {fmfgraph*} undefined).
  DefEnvironment!("{fmfgraph}{}{}",
    "<ltx:note role='feynman-diagram'>(Feynman diagram, #1x#2)</ltx:note>",
    mode => "internal_vertical");
  DefEnvironment!("{fmfgraph*}{}{}",
    "<ltx:note role='feynman-diagram'>(Feynman diagram, #1x#2)</ltx:note>",
    mode => "internal_vertical");
  // {fmffile}{name} - wraps a Feynman-diagram session. Render as no-op
  // env (the diagrams inside are rendered by {fmfgraph}/{fmfgraph*}).
  DefEnvironment!("{fmffile}{}", "#body", mode => "internal_vertical");
  // \fmf, \fmfv, \fmfset, \fmflabel — diagram-content macros used
  // inside {fmfgraph}. We don't render diagrams so absorb their args.
  DefMacro!("\\fmf{}{}", "");
  DefMacro!("\\fmfv{}{}", "");
  DefMacro!("\\fmfset{}{}", "");
  DefMacro!("\\fmflabel{}{}", "");
});

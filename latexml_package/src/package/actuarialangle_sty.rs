use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("actuarialangle", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefConstructor!("\\actuarialangle{}",
    "<ltx:XMApp><ltx:XMTok role='ENCLOSE' enclose='actuarial' meaning='actuarialangle' /><ltx:XMWrap>#1</ltx:XMWrap></ltx:XMApp>");
  Let!("\\lx@actuarialangle@angl", "\\angl");
  DefMath!("\\angl{}", "\\lx@actuarialangle@angl{#1}");
  DefMath!("\\angln", "\\angl{n}");
  DefMath!("\\anglr", "\\angl{r}");
  DefMath!("\\anglk", "\\angl{k}");
});

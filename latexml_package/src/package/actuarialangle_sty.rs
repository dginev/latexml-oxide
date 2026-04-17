use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("actuarialangle", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefConstructor!("\\actuarialangle{}",
    "<ltx:XMApp><ltx:XMTok role='ENCLOSE' enclose='actuarial' meaning='actuarialangle' /><ltx:XMWrap>#1</ltx:XMWrap></ltx:XMApp>");

  // Perl L26-30: \overanglebracket + \group unless nobracket option passed
  if if_condition(&T_CS!("\\ifacta@bracket"))?.unwrap_or(false) {
    DefMath!("\\overanglebracket{}", "\u{23E0}", operator_role => "OVERACCENT");
    DefMath!("\\group{}", "\\overanglebracket{#1}");
  }

  Let!("\\lx@actuarialangle@angl", "\\angl");
  DefMath!("\\angl{}", "\\lx@actuarialangle@angl{#1}");
  DefMath!("\\angln", "\\angl{n}");
  DefMath!("\\anglr", "\\angl{r}");
  DefMath!("\\anglk", "\\angl{k}");
});

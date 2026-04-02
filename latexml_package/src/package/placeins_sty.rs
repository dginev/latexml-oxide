use crate::prelude::*;
// placeins.sty — float barriers (no-op since floats stay where found)
LoadDefinitions!({
  DeclareOption!("section", None);
  DeclareOption!("above", None);
  DeclareOption!("below", None);
  DeclareOption!("verbose", None);
  DefMacro!("\\FloatBarrier", "");
});

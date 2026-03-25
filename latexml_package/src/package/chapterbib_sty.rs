use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DeclareOption!("rootbib", {
    state::assign_value("CITE_UNIT_GLOBAL", Stored::from(1), None);
  });
  DeclareOption!("sectionbib", {});
  DeclareOption!("gather",    {});
  DeclareOption!("duplicate", {});
  ProcessOptions!();
  DefMacro!("\\sectionbib{}{}", "");
});

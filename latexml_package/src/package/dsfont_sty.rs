use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: dsfont.sty.ltxml
  DefConstructor!("\\mathds{}", "#1",
    bounded => true, require_math => true,
    font => {family => "blackboard", series => "medium", shape => "upright"});
});

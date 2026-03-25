use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: bbm.sty.ltxml
  DefConstructor!("\\mathbbm{}", "#1",
    bounded => true, require_math => true,
    font => {family => "blackboard", series => "medium", shape => "upright"});

  // This should be both blackboard AND sansserif, but those are conflicting families!
  // Seemingly the blackboard is the most important?
  DefConstructor!("\\mathbbmss{}", "#1",
    bounded => true, require_math => true,
    font => {family => "blackboard", series => "medium", shape => "upright"});

  // Ditto blackboard and typewriter...
  DefConstructor!("\\mathbbmtt{}", "#1",
    bounded => true, require_math => true,
    font => {family => "blackboard", series => "medium", shape => "upright"});
});

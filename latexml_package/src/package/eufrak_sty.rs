use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: eufrak.sty.ltxml
  // Only defines \mathfrak to use the Euler Fraktur font
  DefConstructor!("\\EuFrak{}", "#1",
    bounded => true, require_math => true,
    font => {family => "fraktur", series => "medium", shape => "upright"});
  DefConstructor!("\\mathfrak{}", "#1",
    bounded => true, require_math => true,
    font => {family => "fraktur", series => "medium", shape => "upright"});
});

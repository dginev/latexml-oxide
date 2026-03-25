use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: tocbibind.sty.ltxml
  // I'm inclined to think there's nothing to do here!
  for option in ["notbib", "notindex", "nottoc", "notlof", "notlot"].iter() {
    DeclareOption!(*option, None);
  }

  ProcessOptions!();
});

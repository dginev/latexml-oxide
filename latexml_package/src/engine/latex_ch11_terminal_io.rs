use crate::prelude::*;

//======================================================================
// C.11.6 Terminal Input and Output
//======================================================================

LoadDefinitions!({
  DefPrimitive!("\\typeout{}", sub[(stuff)] {
    if lookup_int("VERBOSITY") > -1 {
      let content = Expand!(stuff);
      eprintln!("{content}\n");
    }
  });
  DefPrimitive!("\\typein[]{}", None);
});

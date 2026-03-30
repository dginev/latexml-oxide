use crate::prelude::*;

//======================================================================
// C.11.6 Terminal Input and Output
//======================================================================

LoadDefinitions!({
  DefPrimitive!("\\typeout{}", sub[(stuff)] {
    if state::current_verbosity() > -1 {
      let content = Expand!(stuff);
      Note!(s!("{content}"));
    }
  });
  DefPrimitive!("\\typein[]{}", None);
});

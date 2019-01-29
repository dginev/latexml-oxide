use crate::package::*;

LoadDefinitions!(state, {
  //======================================================================
  // C.6.2 List-Making environments
  //======================================================================
  Tag!("ltx:item",        auto_close => true, auto_open => true);
  Tag!("ltx:inline-item", auto_close => true, auto_open => true);
});

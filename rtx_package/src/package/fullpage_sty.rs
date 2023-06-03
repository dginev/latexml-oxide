use crate::package::*;
use rtx_core::state::State;

LoadDefinitions!(stomach, state, {

  // Ignore the options
  for option in ["in","cm","plain", "empty", "headings", "myheadings"] {
    DeclareOption!(option, None);
  }
  ProcessOptions!();

  // Nothing else to do....

});
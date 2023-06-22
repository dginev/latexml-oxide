use crate::package::*;
LoadDefinitions!( {
  // Ignore the options
  for option in ["in","cm","plain", "empty", "headings", "myheadings"] {
    DeclareOption!(option, None);
  }
  ProcessOptions!();
  // Nothing else to do....
});
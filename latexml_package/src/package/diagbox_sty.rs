use crate::prelude::*;

LoadDefinitions!({
  // Perl: diagbox.sty.ltxml — diagonal box for table headers
  // TODO: Full port (164 lines). Stub for now to prevent loading errors.
  // Provides \diagbox, \slashbox, \backslashbox

  // Minimal stubs to prevent undefined errors
  DefMacro!("\\diagbox[]{}{}",  "#2\\\\#3");
  DefMacro!("\\slashbox[]{}{}",    "#2\\\\#3");
  DefMacro!("\\backslashbox[]{}{}","#2\\\\#3");
});

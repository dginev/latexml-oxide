use rtx_package::prelude::*;

LoadDefinitions!({
  // Don't need to respect source newlines
  AssignValue!("INCLUDE_STYLES", true, Some(Scope::Global));
});

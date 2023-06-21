use rtx_package::package::*;

LoadDefinitions!({
  // Don't need to respect source newlines
  AssignValue!("INCLUDE_STYLES", true, Some(Scope::Global));
});

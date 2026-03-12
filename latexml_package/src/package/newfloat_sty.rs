use crate::prelude::*;

LoadDefinitions!({
  // Simplified stubs — full \DeclareFloatingEnvironment needs beforeFloat/afterFloat
  DefMacro!("\\SetupFloatingEnvironment OptionalKeyVals {}", "");
  DefMacro!("\\DeclareFloatingEnvironment OptionalKeyVals {}", "");
  DefMacro!("\\ForEachFloatingEnvironment{}", "");
  DefMacro!("\\PrepareListOf{}{}", "");
});

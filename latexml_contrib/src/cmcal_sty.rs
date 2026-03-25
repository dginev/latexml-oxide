use latexml_package::prelude::*;

LoadDefinitions!({
  // Fixes a typo in otherwise Fatal arxiv:math-ph/0002032
  DefMacro!("\\cmcal", "\\mathcal");
});

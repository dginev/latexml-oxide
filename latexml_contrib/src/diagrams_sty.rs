use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("diagrams", extension => Some(Cow::Borrowed("tex")));
});

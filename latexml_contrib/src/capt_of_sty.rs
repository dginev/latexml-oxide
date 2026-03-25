use latexml_package::prelude::*;

LoadDefinitions!({
  DefMacro!("\\captionof {}", "\\def\\@captype{#1}\\caption");
});

use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("hepparticles", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});

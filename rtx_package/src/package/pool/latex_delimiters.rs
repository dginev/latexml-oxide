use crate::package::*;

LoadDefinitions!(state, {
  LetI!(&T_CS!("\\vert"), T_OTHER!("|"));
  Let!("\\Vert", "\\|");
});

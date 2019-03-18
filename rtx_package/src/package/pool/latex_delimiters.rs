use crate::package::*;

LoadDefinitions!(state, {
  Let!(&T_CS!("\\vert"), T_OTHER!("|"));
  Let!("\\Vert", "\\|");
});

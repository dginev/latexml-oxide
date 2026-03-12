use crate::prelude::*;

LoadDefinitions!({
  // Redefine \documentclass to do nothing in subfiles
  DefMacro!("\\documentclass OptionalSemiverbatim SkipSpaces Semiverbatim []", "");
  // Define \subfile to be \input
  Let!("\\subfile", "\\input");
});

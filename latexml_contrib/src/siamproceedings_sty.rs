//! siamproceedings.sty: the paper's copy read raw, the marks its `\maketitle` sets declared — none under `titlepage`,
//! whose `\maketitle` sets no `\thefootnote` (2509.02371, 2509.13490; OD #1053).
use latexml_package::{
  prelude::*,
  shipped_class::{FNSYMBOL_TITLE_MARKS, input_shipped_package_title_marks},
};

LoadDefinitions!({
  // (siamproceedings.sty:187-189)
  input_shipped_package_title_marks(FNSYMBOL_TITLE_MARKS)?;
});

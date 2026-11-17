//! usenix.sty (usenix2019_v3, usenix-2020-09; the versioned fallback strips each to `usenix`): the paper's copy read
//! raw, the marks its `\maketitle` sets declared (2509.03117, 1110.1687, 2301.04841; OD #1053).
use latexml_package::{
  prelude::*,
  shipped_class::{FNSYMBOL_TITLE_MARKS, input_shipped_package_title_marks},
};

LoadDefinitions!({
  input_shipped_package_title_marks(&[FNSYMBOL_TITLE_MARKS])?;
});

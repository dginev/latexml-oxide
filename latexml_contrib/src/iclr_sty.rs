//! iclr.sty and its kin, the NIPS style's descendants (iclr, tmlr, corl, nips 2016-2018; the versioned fallback strips
//! each): the paper's copy read raw, the marks its `\maketitle` sets declared (2303.07937, 1706.03762; OD #1053).
use latexml_package::{
  prelude::*,
  shipped_class::{FNSYMBOL_TITLE_MARKS, input_shipped_package_title_marks},
};

LoadDefinitions!({
  input_shipped_package_title_marks(FNSYMBOL_TITLE_MARKS)?;
});

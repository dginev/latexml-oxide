//! acl.sty and its kin (acl20xx, naacl, emnlp, eacl, coling, tacl, conll; the versioned fallback strips each to its
//! family): the paper's copy read raw, the marks its `\maketitle` sets declared (2502.19103, 1905.10033; OD #1053).
use latexml_package::{
  prelude::*,
  shipped_class::{FNSYMBOL_TITLE_MARKS, input_shipped_package_title_marks},
};

LoadDefinitions!({
  input_shipped_package_title_marks(&[FNSYMBOL_TITLE_MARKS])?;
});

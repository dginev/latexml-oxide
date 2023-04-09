///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::tex_tests;
use std::sync::Arc;

tex_tests!(
  "tests/grouping",
  None,
  Some(Arc::new(rtx_contrib::dispatch))
);

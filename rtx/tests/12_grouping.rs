///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::tex_tests;
use std::rc::Rc;

tex_tests!(
  "tests/grouping",
  None,
  Some(Rc::new(rtx_contrib::dispatch))
);

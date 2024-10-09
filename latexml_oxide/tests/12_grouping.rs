///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use latexml::tex_tests;
use std::rc::Rc;

tex_tests!("tests/grouping", None, Some(Rc::new(latexml_contrib::dispatch)));

///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
use latexml::util::test::*;

#[test]
fn can_namespace() { latexml_tests("tests/structure", None, None); }

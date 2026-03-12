///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
use latexml::util::test::*;

#[test]
#[ignore] // Namespace tests need .latexml document-level bindings (custom DTDs)
fn can_namespace() { latexml_tests("tests/namespace", None, None); }

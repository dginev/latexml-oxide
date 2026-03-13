use latexml::util::test::*;

#[test]
#[ignore] // Note: defer math tests to the very end
fn can_mathl() { latexml_tests("tests/math", None, None); }

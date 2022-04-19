///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;

#[test]
fn can_digest() { rtx_tests("tests/digestion", None, None); }

#[test]
#[ignore]
fn can_digest_todo() { rtx_tests("tests/digestion_todo", None, None); }

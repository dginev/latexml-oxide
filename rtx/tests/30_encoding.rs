///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use rustc_hash::{FxHashMap as HashMap};

#[test]
#[ignore]
fn can_encode() {
  let mut requires = HashMap::default();
  requires.insert("ansinew", "ansinew.def");
  requires.insert("applemac", "applemac.def");
  requires.insert("cp437", "cp437.def");
  requires.insert("cp437de", "cp437de.def");
  requires.insert("cp850", "cp850.def");
  requires.insert("cp852", "cp852.def");
  requires.insert("cp858", "cp858.def");
  requires.insert("cp865", "cp865.def");
  requires.insert("cp1250", "cp1250.def");
  requires.insert("cp1252", "cp1252.def");
  requires.insert("decmulti", "decmulti.def");
  requires.insert("macce", "macce.def");
  requires.insert("next", "next.def");
  requires.insert("latin1", "latin1.def");
  requires.insert("latin2", "latin2.def");
  requires.insert("latin3", "latin3.def");
  requires.insert("latin4", "latin4.def");
  requires.insert("latin5", "latin5.def");
  requires.insert("latin9", "latin9.def");
  requires.insert("latin10", "latin10.def");
  requires.insert("ot1", "ot1enc.def");
  requires.insert("t1", "t1enc.def");
  requires.insert("t2a", "t2aenc.def");
  requires.insert("t2b", "t2benc.def");
  requires.insert("t2c", "t2cenc.def");
  requires.insert("ts1", "ts1enc.def");
  requires.insert("ly1", "ly1enc.def");

  rtx_tests("tests/encoding", Some(requires), None);
}

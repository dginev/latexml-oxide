use latexml::{
  tex_tests,
  util::test::{RequirementTable, Requirements},
};

// Perl t/30_encoding.t.
static REQUIRES: &RequirementTable = &[
  ("ansinew", Requirements::packages(&["ansinew.def"])),
  ("applemac", Requirements::packages(&["applemac.def"])),
  ("cp437", Requirements::packages(&["cp437.def"])),
  ("cp437de", Requirements::packages(&["cp437de.def"])),
  ("cp850", Requirements::packages(&["cp850.def"])),
  ("cp852", Requirements::packages(&["cp852.def"])),
  ("cp858", Requirements::packages(&["cp858.def"])),
  ("cp865", Requirements::packages(&["cp865.def"])),
  ("cp1250", Requirements::packages(&["cp1250.def"])),
  ("cp1252", Requirements::packages(&["cp1252.def"])),
  ("decmulti", Requirements::packages(&["decmulti.def"])),
  ("macce", Requirements::packages(&["macce.def"])),
  ("next", Requirements::packages(&["next.def"])),
  ("latin1", Requirements::packages(&["latin1.def"])),
  ("latin2", Requirements::packages(&["latin2.def"])),
  ("latin3", Requirements::packages(&["latin3.def"])),
  ("latin4", Requirements::packages(&["latin4.def"])),
  ("latin5", Requirements::packages(&["latin5.def"])),
  ("latin9", Requirements::packages(&["latin9.def"])),
  ("latin10", Requirements::packages(&["latin10.def"])),
  ("ot1", Requirements::packages(&["ot1enc.def"])),
  ("t1", Requirements::packages(&["t1enc.def"])),
  ("t2a", Requirements::packages(&["t2aenc.def"])),
  ("t2b", Requirements::packages(&["t2benc.def"])),
  ("t2c", Requirements::packages(&["t2cenc.def"])),
  ("ts1", Requirements::packages(&["ts1enc.def"])),
  ("ly1", Requirements::packages(&["ly1enc.def"])),
];

tex_tests!("tests/encoding", Some(REQUIRES), None);

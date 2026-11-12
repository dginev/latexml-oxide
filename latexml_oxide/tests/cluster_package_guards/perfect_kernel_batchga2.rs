//! Guards for batch GA2: "is this package loaded" has one answer, `state::file_loaded` (`\ver@<file>`, latex.ltx
//! `\@ifl@aded`), on every load path; its fence is `perfect_kernel_batche0::one_loaded_query`. A module of its own: the
//! loader guards (67y, 77c) test the loader's records, these the query beside them.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/loader/",
      $path
    ))
  };
}

/// Each row: a repro (with the class copy it ships), and its whole `<para>`: every `\@ifpackageloaded` /
/// `\@ifclassloaded` answer as pdflatex prints it, and the `\nonumber` test beside them.
#[test]
fn one_loaded_answer_on_every_load_path() {
  for (name, tex, files, para) in [
    // aa.cls's binding clears amsmath's load record (Perl aa.cls.ltxml:57) after the paper's aa.cls loads amsmath
    // (2501.07980, 1712.01010, 1812.04463): `\@ifpackageloaded` read that record ("not loaded") while `\nonumber`'s
    // test read `\ver@amsmath.sty` (the equation unnumbered); thmtools stands in for thm-patch (`state::mark_loaded`);
    // a document's `\@namedef{ver@foo.sty}` marks foo loaded.
    (
      "aa_copy",
      repro!("aa_copy/aa_copy_loads_amsmath_one_answer.tex"),
      &[("aa.cls", repro!("aa_copy/aa.cls"))][..],
      r#"<para xml:id="p1"> <p xml:id="p1.1">amsmath loaded.</p> <equation xml:id="S0.Ex1"> <Math mode="display" tex="a=b" text="a = b" xml:id="S0.Ex1.m1"> <XMath xml:id="S0.Ex1.m1.1"> <XMApp xml:id="S0.Ex1.m1.1.1"> <XMTok meaning="equals" role="RELOP">=</XMTok> <XMTok font="italic" role="UNKNOWN">a</XMTok> <XMTok font="italic" role="UNKNOWN">b</XMTok> </XMApp> </XMath> </Math> </equation> <p xml:id="p1.2">thm-patch loaded. foo loaded.</p> </para>"#,
    ),
    // revtex4-2.cls never loads hyperref: its binding's preload is unmarked, `\ver@hyperref.sty` too, until the
    // document's own load (2207.13707); fontenc.sty's tail unmarks itself; a class asked by `\@ifclassloaded`.
    (
      "revtex",
      repro!("loaded_query_follows_ver_marks.tex"),
      &[][..],
      r#"<para xml:id="p1"> <p xml:id="p1.1">hyperref not loaded. fontenc not loaded. revtex4-2 loaded. hyperref loaded.</p> </para>"#,
    ),
    // A document takes back the mark of a package it loaded (amsmath's `\nonumber` stays in force), and marks one inside
    // a group only.
    (
      "document",
      repro!("loaded_query_reads_document_marks.tex"),
      &[][..],
      r#"<para xml:id="p1"> <p xml:id="p1.1">amsmath not loaded. foo loaded. foo not loaded. amsmath not loaded.</p> <equation xml:id="S0.Ex1"> <Math mode="display" tex="a=b" text="a = b" xml:id="S0.Ex1.m1"> <XMath xml:id="S0.Ex1.m1.1"> <XMApp xml:id="S0.Ex1.m1.1.1"> <XMTok meaning="equals" role="RELOP">=</XMTok> <XMTok font="italic" role="UNKNOWN">a</XMTok> <XMTok font="italic" role="UNKNOWN">b</XMTok> </XMApp> </XMath> </Math> </equation> </para>"#,
    ),
  ] {
    let (log, xml) = convert_files_with(tex, files, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(xml.contains(para), "{name}: {xml}");
  }
}

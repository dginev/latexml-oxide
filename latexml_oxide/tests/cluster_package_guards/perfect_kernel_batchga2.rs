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
    // revGA2 B1: a query before the load (`\ifx\csname ver@X.sty\endcsname\relax`, `\@ifpackagelater`,
    // `\IfPackageAtLeastTF`, `\IfPackageLoadedTF`) leaves `\ver@X.sty` as `\relax`; the binding load (amsmath, natbib,
    // graphicx, hyperref, booktabs) and the raw load of a file declaring no version (bar.sty) mark it all the same.
    (
      "early_query",
      repro!("early_query/loaded_query_after_an_early_query.tex"),
      &[("bar.sty", repro!("early_query/bar.sty"))][..],
      r#"<para xml:id="p1"> <p xml:id="p1.1">hyperref loaded. amsmath loaded. natbib loaded. graphicx loaded. geometry loaded. booktabs loaded. bar loaded.</p> <equation xml:id="S0.Ex1"> <Math mode="display" tex="a=b" text="a = b" xml:id="S0.Ex1.m1"> <XMath xml:id="S0.Ex1.m1.1"> <XMApp xml:id="S0.Ex1.m1.1.1"> <XMTok meaning="equals" role="RELOP">=</XMTok> <XMTok font="italic" role="UNKNOWN">a</XMTok> <XMTok font="italic" role="UNKNOWN">b</XMTok> </XMApp> </XMath> </Math> </equation> <equation xml:id="S0.E1"> <tags> <tag>(1)</tag> <tag role="autoref">Equation 1<text xml:id="S0.E1.1"/></tag> <tag role="refnum">1</tag> </tags> <Math mode="display" tex="c=d" text="c = d" xml:id="S0.E1.m1"> <XMath xml:id="S0.E1.m1.1"> <XMApp xml:id="S0.E1.m1.1.1"> <XMTok meaning="equals" role="RELOP">=</XMTok> <XMTok font="italic" role="UNKNOWN">c</XMTok> <XMTok font="italic" role="UNKNOWN">d</XMTok> </XMApp> </XMath> </Math> </equation> </para>"#,
    ),
    // revGA2 N1: a repeat request after the document's unmark restores foo's own version, not `\fmtversion`.
    (
      "repeat_version",
      repro!("repeat_version/loaded_query_repeat_keeps_the_version.tex"),
      &[("foo.sty", repro!("repeat_version/foo.sty"))][..],
      r#"<para xml:id="p1"> <p xml:id="p1.1">foo loaded, earlier than 2010.</p> </para>"#,
    ),
    // revGA2 S1: cleveref's amsmath pretense keeps a recorded load the document unmarked (`\nonumber` stays amsmath's).
    (
      "pretense",
      repro!("loaded_query_pretense_keeps_the_record.tex"),
      &[][..],
      r#"<para xml:id="p1"> <p xml:id="p1.1">amsmath not loaded.</p> <equation xml:id="S0.Ex1"> <Math mode="display" tex="a=b" text="a = b" xml:id="S0.Ex1.m1"> <XMath xml:id="S0.Ex1.m1.1"> <XMApp xml:id="S0.Ex1.m1.1.1"> <XMTok meaning="equals" role="RELOP">=</XMTok> <XMTok font="italic" role="UNKNOWN">a</XMTok> <XMTok font="italic" role="UNKNOWN">b</XMTok> </XMApp> </XMath> </Math> </equation> <equation labels="LABEL:e" xml:id="S0.E1"> <tags> <tag>(1)</tag> <tag role="creflabel">(1)</tag> <tag role="creftype">eq.</tag> <tag role="creftypecap">Equation</tag> <tag role="creftypeplural">eqs.</tag> <tag role="creftypepluralcap">Equations</tag> <tag role="refnum">1</tag> </tags> <Math mode="display" tex="c=d" text="c = d" xml:id="S0.E1.m1"> <XMath xml:id="S0.E1.m1.1"> <XMApp xml:id="S0.E1.m1.1.1"> <XMTok meaning="equals" role="RELOP">=</XMTok> <XMTok font="italic" role="UNKNOWN">c</XMTok> <XMTok font="italic" role="UNKNOWN">d</XMTok> </XMApp> </XMath> </Math> </equation> <equationgroup class="ltx_eqn_align" xml:id="S0.EGx1"> <equation xml:id="S0.Ex2"> <MathFork> <Math tex="\displaystyle e=f" text="e = f" xml:id="S0.Ex2.m3"> <XMath xml:id="S0.Ex2.m3.1"> <XMApp xml:id="S0.Ex2.m3.1.1"> <XMTok meaning="equals" role="RELOP">=</XMTok> <XMTok font="italic" role="UNKNOWN">e</XMTok> <XMTok font="italic" role="UNKNOWN">f</XMTok> </XMApp> </XMath> </Math> <MathBranch> <td align="right" xml:id="S0.Ex2.1"><Math mode="inline" tex="\displaystyle e" text="e" xml:id="S0.Ex2.m1"> <XMath xml:id="S0.Ex2.m1.1"> <XMTok font="italic" role="UNKNOWN">e</XMTok> </XMath> </Math></td> <td align="left" xml:id="S0.Ex2.2"><Math mode="inline" tex="\displaystyle=f" text="absent = f" xml:id="S0.Ex2.m2"> <XMath xml:id="S0.Ex2.m2.1"> <XMApp xml:id="S0.Ex2.m2.1.1"> <XMTok meaning="equals" role="RELOP">=</XMTok> <XMTok meaning="absent"/> <XMTok font="italic" role="UNKNOWN">f</XMTok> </XMApp> </XMath> </Math></td> </MathBranch> </MathFork> </equation> <equation xml:id="S0.E2"> <tags> <tag>(2)</tag> <tag role="creflabel">(2)</tag> <tag role="creftype">eq.</tag> <tag role="creftypecap">Equation</tag> <tag role="creftypeplural">eqs.</tag> <tag role="creftypepluralcap">Equations</tag> <tag role="refnum">2</tag> </tags> <MathFork> <Math tex="\displaystyle g=h" text="g = h" xml:id="S0.E2.m3"> <XMath xml:id="S0.E2.m3.1"> <XMApp xml:id="S0.E2.m3.1.1"> <XMTok meaning="equals" role="RELOP">=</XMTok> <XMTok font="italic" role="UNKNOWN">g</XMTok> <XMTok font="italic" role="UNKNOWN">h</XMTok> </XMApp> </XMath> </Math> <MathBranch> <td align="right" xml:id="S0.E2.1"><Math mode="inline" tex="\displaystyle g" text="g" xml:id="S0.E2.m1"> <XMath xml:id="S0.E2.m1.1"> <XMTok font="italic" role="UNKNOWN">g</XMTok> </XMath> </Math></td> <td align="left" xml:id="S0.E2.2"><Math mode="inline" tex="\displaystyle=h" text="absent = h" xml:id="S0.E2.m2"> <XMath xml:id="S0.E2.m2.1"> <XMApp xml:id="S0.E2.m2.1.1"> <XMTok meaning="equals" role="RELOP">=</XMTok> <XMTok meaning="absent"/> <XMTok font="italic" role="UNKNOWN">h</XMTok> </XMApp> </XMath> </Math></td> </MathBranch> </MathFork> </equation> </equationgroup> <p xml:id="p1.2">See <ref labelref="LABEL:e" show="creftype~creflabel"/>.</p> </para>"#,
    ),
    // GA2c: latex.ltx:18840 marks `\ver@<file>` `\@empty` before the read, so a file asks for its loader while it loads
    // (1411.7188's l3names.sty for expl3): a raw package (int166a erred) and a binding reading the paper's .cfg.
    (
      "while_loading",
      repro!("while_loading/loaded_query_while_the_file_loads.tex"),
      &[
        ("outerpk.sty", repro!("while_loading/outerpk.sty")),
        ("innerpk.sty", repro!("while_loading/innerpk.sty")),
        ("subfigure.cfg", repro!("while_loading/subfigure.cfg")),
      ][..],
      r#"<para xml:id="p1"> <p xml:id="p1.1">outerpk loaded. subfigure loaded.</p> </para>"#,
    ),
    // GA2c: the class sibling, `\documentclass` and `\LoadClass` (outercl.cls loads basecl.cls, which requires innercl.sty).
    (
      "class_while_loading",
      repro!("while_loading/loaded_query_while_the_class_loads.tex"),
      &[
        ("outercl.cls", repro!("while_loading/outercl.cls")),
        ("basecl.cls", repro!("while_loading/basecl.cls")),
        ("innercl.sty", repro!("while_loading/innercl.sty")),
      ][..],
      r#"<para xml:id="p1"> <p xml:id="p1.1">outercl loaded. basecl loaded.</p> </para>"#,
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

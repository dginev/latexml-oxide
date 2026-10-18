//! Red/green guards for the int23 integration fix round (72a r1-r10 with 72b r1-r4): the 72a review r7's SHOULDs and
//! NIT 8. Every expectation is pdflatex's (the repro headers name the oracle run).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_files_with, error_count, warning_count};

/// The arXiv configuration's preload: a paper's own `.sty`/`.cls` is read raw.
const AR5IV: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// 0 errors, 0 warnings, 0 fatals.
fn assert_clean(log: &str) {
  assert_eq!(
    (
      error_count(log),
      warning_count(log),
      log.matches("Fatal:").count()
    ),
    (0, 0, 0),
    "{log}"
  );
}

/// 72a review r7 SHOULD 1 (OXIDIZED_DESIGN_DIVERGENCES #737 item 4): scrbook's `\frontmatter` unnumbers the chapters
/// and `\mainmatter` numbers them again, as KOMA's `\if@mainmatter` does under pdflatex (scrbook.cls:5272-5280,
/// :5573-5576): "Pre" unnumbered, "1 One", "1.1 S", "A App", "A.1 T". 72a r10: "Chapter 1 Pre", "Chapter 2 One",
/// "2.1 S". Witness 2506.17485.
#[test]
fn scrbook_front_matter_chapters_are_unnumbered() {
  let (log, xml) = convert_files_with(
    repro!("sectioning-frontmatter/scrbook_front_matter_chapters_are_unnumbered.tex"),
    &[],
    Some(AR5IV),
  );
  assert_clean(&log);
  assert_element(
    &xml,
    "chapter",
    &["xml:id=\"Chx1\""],
    "<chapter inlist=\"toc\" xml:id=\"Chx1\"><title>Pre</title><para xml:id=\"Chx1.p1\"><p \
     xml:id=\"Chx1.p1.1\">Pre.</p><pagination role=\"newpage\"/><pagination role=\"newpage\"/></para></chapter>",
  );
  assert_element(
    &xml,
    "section",
    &["xml:id=\"Ch1.S1\""],
    "<section inlist=\"toc\" xml:id=\"Ch1.S1\"><tags><tag>1.1</tag><tag role=\"refnum\">1.1</tag><tag \
     role=\"typerefnum\">§1.1</tag></tags><title><tag close=\" \">1.1</tag>S</title><para xml:id=\"Ch1.S1.p1\"><p \
     xml:id=\"Ch1.S1.p1.1\">Text.</p></para><figure inlist=\"lof\" xml:id=\"Ch1.F1\"><tags><tag>Figure\u{a0}1.1</tag><tag \
     role=\"refnum\">1.1</tag><tag role=\"typerefnum\">Figure 1.1</tag></tags><p \
     xml:id=\"Ch1.F1.1\">x</p><toccaption><tag close=\" \">1.1</tag>F</toccaption><caption><tag close=\": \">Figure\u{a0}1.1</tag>F</caption></figure></section>",
  );
  assert_element(
    &xml,
    "appendix",
    &["xml:id=\"A1\""],
    "<appendix inlist=\"toc\" xml:id=\"A1\"><tags><tag>Appendix A</tag><tag role=\"refnum\">A</tag><tag \
     role=\"typerefnum\">Appendix A</tag></tags><title><tag close=\" \">Appendix \
     A</tag>App</title><toctitle><tag close=\" \">A</tag>App</toctitle><section inlist=\"toc\" \
     xml:id=\"A1.S1\"><tags><tag>A.1</tag><tag role=\"refnum\">A.1</tag><tag \
     role=\"typerefnum\">§A.1</tag></tags><title><tag close=\" \">A.1</tag>T</title><para xml:id=\"A1.S1.p1\"><p \
     xml:id=\"A1.S1.p1.1\">More.</p></para></section></appendix>",
  );
}

/// 72a review r7 SHOULD 2: a ws-rv9x6.cls copy's own `\newcounter{chapter}` and `\newcounter{section}[chapter]`
/// (:397-399) keep the binding's chapter counter and its ids (ws_procs_cls.rs, 72b r3): `Ch1`, `Ch1.S1`,
/// `Ch1.S1.SS1`, where 72a r6-r10 named them after the counters (`chapter1`, `S1`). Witnesses 2301.03442, 1602.07982.
#[test]
fn ws_rv9x6_copy_counters_keep_the_binding_ids() {
  let (log, xml) = convert_files_with(
    repro!("stand-in-copies/counters/ws_rv9x6/ws_rv9x6_copy_counters_keep_the_binding_ids.tex"),
    &[(
      "ws-rv9x6.cls",
      repro!("stand-in-copies/counters/ws_rv9x6/ws-rv9x6.cls"),
    )],
    Some(AR5IV),
  );
  assert_clean(&log);
  assert_element(
    &xml,
    "chapter",
    &["xml:id=\"Ch1\""],
    "<chapter inlist=\"toc\" xml:id=\"Ch1\"><tags><tag>Chapter 1</tag><tag role=\"autoref\">chapter\u{a0}1<text \
     xml:id=\"Ch1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Chapter 1</tag></tags><title><tag \
     close=\" \">Chapter 1</tag>Intro</title><toctitle><tag close=\" \">1</tag>Intro</toctitle><section \
     inlist=\"toc\" xml:id=\"Ch1.S1\"><tags><tag>1</tag><tag role=\"autoref\">section\u{a0}1<text \
     xml:id=\"Ch1.S1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">§1</tag></tags><title><tag \
     close=\" \">1</tag>Sec</title><para xml:id=\"Ch1.S1.p1\"><p xml:id=\"Ch1.S1.p1.1\">Text.</p></para><subsection \
     inlist=\"toc\" xml:id=\"Ch1.S1.SS1\"><tags><tag>1.1</tag><tag role=\"autoref\">subsection\u{a0}1.1<text \
     xml:id=\"Ch1.S1.SS1.1\"/></tag><tag role=\"refnum\">1.1</tag><tag \
     role=\"typerefnum\">§1.1</tag></tags><title><tag close=\" \">1.1</tag>Sub</title><para \
     xml:id=\"Ch1.S1.SS1.p1\"><p xml:id=\"Ch1.S1.SS1.p1.1\">More.</p></para></subsection></section></chapter>",
  );
}

/// 72a review r7 NIT 8 (OXIDIZED_DESIGN_DIVERGENCES #736): a class a shipped intermediate class loads is the copy's
/// too. svmult.cls (the copy) `\LoadClass{mybase}` (shipped, read), whose `\LoadClass{article}` meets the class slot
/// with svmult's binding's book (`state::open_class_load_classes` reads the copy's first class load, not mybase.cls's
/// empty one): the slot's Info, article not loaded over book. pdflatex "1 Intro", "Text SV MB."; the "0.1" is book's,
/// the open svmult row (`copy_load_class_meets_the_class_slot`).
#[test]
fn a_shipped_intermediate_class_meets_the_class_slot() {
  let (log, xml) = convert_files_with(
    repro!(
      "stand-in-copies/class_slot/intermediate/a_shipped_intermediate_class_meets_the_class_slot.tex"
    ),
    &[
      (
        "svmult.cls",
        repro!("stand-in-copies/class_slot/intermediate/svmult.cls"),
      ),
      (
        "mybase.cls",
        repro!("stand-in-copies/class_slot/intermediate/mybase.cls"),
      ),
    ],
    Some(AR5IV),
  );
  assert_clean(&log);
  assert!(
    log.contains(
      "Info:provide:article.cls the binding's base class (book.cls) stands for the copy's \\LoadClass{article}: not \
       loaded"
    ),
    "{log}"
  );
  assert!(!log.contains("(Loading article_cls.rs"), "{log}");
  assert_element(
    &xml,
    "section",
    &["xml:id=\"Ch0.S1\""],
    "<section inlist=\"toc\" xml:id=\"Ch0.S1\"><tags><tag>0.1</tag><tag role=\"refnum\">0.1</tag><tag \
     role=\"typerefnum\">§0.1</tag></tags><title><tag close=\" \">0.1</tag>Intro</title><para xml:id=\"Ch0.S1.p1\"><p \
     xml:id=\"Ch0.S1.p1.1\">Text SV MB.</p></para></section>",
  );
}

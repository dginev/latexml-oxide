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

/// int23 fix round 2, BLOCKER 2 (main-session ruling 2026-10-18): a file a stand-in's copy pulls in by `\input`
/// (`\InputIfFileExists`: the `.clo` it ships) is part of the copy and follows its provides rule. svjour3.cls:86's
/// `\InputIfFileExists{sv\CurrentOption.clo}` reads svepjc3.clo, whose `\AtEndOfClass{\let\maketitle=\epj@maketitle}`
/// (svepj.clo:280-284) and re-read `\def\maketitle` (svepjc3.clo:57/:71) replaced the binding's title page: "Noname",
/// "Address(es) of author(s) should be given" (0901.1015, 1012.5412, 1111.3868, 1512.01957, 2411.18962, 2107.02119).
/// The option file's extra `\epjextra` stands. pdflatex: "Noname", "EPJ EXTRA" (the option file's own title page).
#[test]
fn a_copy_option_file_keeps_the_bindings_title() {
  let (log, xml) = convert_files_with(
    repro!("stand-in-copies/option_files/copy_option_file_keeps_the_bindings_title.tex"),
    &[
      (
        "svjour3.cls",
        repro!("stand-in-copies/option_files/svjour3.cls"),
      ),
      (
        "svepjc3.clo",
        repro!("stand-in-copies/option_files/svepjc3.clo"),
      ),
    ],
    Some(AR5IV),
  );
  assert_clean(&log);
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>A. Author</personname><contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Inst X, Town</contact></creator>",
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">EPJ EXTRA</p></para>",
  );
  assert!(!xml.contains("Noname"), "{xml}");
}

/// int23 fix round 2, SHOULD 3: elife.cls:392-416 `\figsupp` prints the supplement's heading in the figure and writes
/// its graphic and caption to the `suppinfo` stream, which :456-461's `\AtEndDocument` inputs; the copy's document-time
/// hook is not registered, so the elife binding owns `\figsupp` and prints the supplements at the end, numbered as the
/// stream numbers them (2311.00616's ~1,000 words of supplement captions). pdflatex: in the figure "Figure 1—figure
/// supplement 1. Short one.", "… 2. Long two."; at the end "GRAPHIC ONE Figure 1—figure supplement 1. Long one. Figure
/// 1—figure supplement 1—source data 1. Source data", "GRAPHIC TWO Figure 1—figure supplement 2. Long two.".
#[test]
fn elife_figure_supplements_are_printed() {
  let (log, xml) = convert_files_with(
    repro!("stand-in-copies/elife_figsupp/figure_supplements_are_printed.tex"),
    &[(
      "elife.cls",
      repro!("stand-in-copies/elife_figsupp/elife.cls"),
    )],
    Some(AR5IV),
  );
  assert_clean(&log);
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S0.F1.2\""],
    "<p class=\"ltx_figure_panel\" xml:id=\"S0.F1.2\"><text font=\"bold\" fontsize=\"90%\" xml:id=\"S0.F1.2.1\">Figure\u{a0}\
     1—figure supplement 1.<text font=\"medium\" xml:id=\"S0.F1.2.1.1\"> Short one.</text></text></p>",
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S0.F1.3\""],
    "<p class=\"ltx_figure_panel\" xml:id=\"S0.F1.3\"><text font=\"bold\" fontsize=\"90%\" xml:id=\"S0.F1.3.1\">Figure\u{a0}\
     1—figure supplement 2.<text font=\"medium\" xml:id=\"S0.F1.3.1.1\"> Long two.</text></text></p>",
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p2\""],
    "<para class=\"ltx_noindent\" xml:id=\"p2\"><p xml:id=\"p2.1\"><inline-block class=\"ltx_minipage\" \
     vattach=\"middle\" width=\"345.0pt\" xml:id=\"p2.1.1\"><p xml:id=\"p2.1.1.1\">GRAPHIC ONE</p><p \
     xml:id=\"p2.1.1.2\"><text font=\"bold\" xml:id=\"p2.1.1.2.1\">Figure\u{a0}1—figure supplement 1.</text> Long one.</p><p \
     xml:id=\"p2.1.1.3\"><text font=\"bold\" fontsize=\"90%\" xml:id=\"p2.1.1.3.1\">Figure\u{a0}1—figure supplement \
     1—source data\u{a0}1. <text font=\"medium\" xml:id=\"p2.1.1.3.1.1\"> Source data</text></text></p></inline-block></p></para>",
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p3\""],
    "<para class=\"ltx_noindent\" xml:id=\"p3\"><p xml:id=\"p3.1\"><inline-block class=\"ltx_minipage\" \
     vattach=\"middle\" width=\"345.0pt\" xml:id=\"p3.1.1\"><p xml:id=\"p3.1.1.1\">GRAPHIC TWO</p><p \
     xml:id=\"p3.1.1.2\"><text font=\"bold\" xml:id=\"p3.1.1.2.1\">Figure\u{a0}1—figure supplement 2.</text> Long \
     two.</p></inline-block></p></para>",
  );
}

/// int23 fix round 2, SHOULD 4: a neurips_2026.sty copy's own setters `\projectpage`/`\emailaddress` (2605.09681's
/// :410-411, printed by the end of its `{abstract}`, :445-458) are the author's contacts. pdflatex: "Project Page:
/// https://example.org/p", "Email Address: me@example.org".
#[test]
fn neurips_copy_setters_are_contacts() {
  let (log, xml) = convert_files_with(
    repro!("stand-in-copies/neurips_contact/copy_setters_are_contacts.tex"),
    &[(
      "neurips_2026.sty",
      repro!("stand-in-copies/neurips_contact/neurips_2026.sty"),
    )],
    Some(AR5IV),
  );
  assert_clean(&log);
  assert_element(
    &xml,
    "creator",
    &["role=\"author\""],
    "<creator role=\"author\"><personname>A. Author</personname><contact name=\"Project Page: \" \
     role=\"url\">https://example.org/p</contact><contact name=\"Email Address: \" \
     role=\"email\">me@example.org</contact></creator>",
  );
}

/// int23 fix round 2, SHOULD 5 (OXIDIZED_DESIGN_DIVERGENCES #731 rule 0): a copy's declaration is checked against the
/// binding that answered, not the name asked for. 1705.03989's aaart.cls (amsart renamed, `\ProvidesClass{aaart}`) is
/// answered by the `aa` binding through the version-name fallback; it shares too few of aa's names and is not read.
/// pdflatex "READ" (TeX reads the paper's file); the fallback's Warning is Perl's too.
#[test]
fn a_renamed_class_under_a_fallback_is_not_read() {
  let (log, xml) = convert_files_with(
    repro!("stand-in-copies/identity/renamed_class_under_a_fallback_is_not_read.tex"),
    &[("aaart.cls", repro!("stand-in-copies/identity/aaart.cls"))],
    Some(AR5IV),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("Can't find binding for class aaart (using aa)"),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"p1.1\""],
    "<p xml:id=\"p1.1\">NOT READ</p>",
  );
}

/// int23 fix round 2, SHOULD 6: the mdpi binding loads cleveref only where the paper's class does (the 2023+ copies:
/// 2410.21443's :445, 2401.15064, 2408.03136). The 2021 copies (2201.05363, 2111.02495) load none; cleveref under their
/// paracol `\globalcounter{figure}` and the paper's subcaption made every subfigure `\caption` err, as pdflatex errs
/// with cleveref added. pdflatex: "(a) Accuracy graph", "Figure 1. Main".
#[test]
fn mdpi_loads_cleveref_only_where_the_class_does() {
  let (log, xml) = convert_files_with(
    repro!("stand-in-copies/mdpi_cleveref/cleveref_only_where_the_class_loads_it.tex"),
    &[("mdpi.cls", repro!("stand-in-copies/mdpi_cleveref/mdpi.cls"))],
    Some(AR5IV),
  );
  assert_clean(&log);
  assert!(!log.contains("(Loading cleveref_sty.rs"), "{log}");
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1.sf1\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F1.sf1\"><tags><tag><text fontsize=\"90%\" \
     xml:id=\"S0.F1.sf1.1\">(a)</text></tag><tag role=\"autoref\">Figure\u{a0}1a<text xml:id=\"S0.F1.sf1.2\"/></tag><tag \
     role=\"refnum\">1a</tag></tags><toccaption><tag close=\" \">a</tag>Accuracy graph</toccaption><caption><tag \
     close=\" \"><text fontsize=\"90%\" xml:id=\"S0.F1.sf1.3\">(a)</text></tag><text fontsize=\"90%\" \
     xml:id=\"S0.F1.sf1.4\">Accuracy graph</text></caption></figure>",
  );
}

/// int23 fix round 2, SHOULD 7: the WileyNJD binding's apacite load stands for NJDapacite.sty (apacite.sty of 2013),
/// so the copy's own `\usepackage[apaciteclassic]{NJDapacite}` (WileyNJDv5.cls:6067) is a repeat. Read raw after the
/// binding, its skipped `\if@APAC@natbib@emu` branch met the natbib binding's `\ifciteindex` at its `\newif\ifciteindex`
/// (NJDapacite.sty:931; TeX, with no natbib, sees an undefined name) and the skip ran on through :1878-1882's
/// `\let\fi\fi`: three "Attempt to close a group" Errors (2606.10545, 2602.12288). pdflatex: "Text.".
#[test]
fn wileynjd_copy_apacite_load_is_a_repeat() {
  let (log, xml) = convert_files_with(
    repro!("stand-in-copies/njdapacite/copy_apacite_load_is_a_repeat.tex"),
    &[
      (
        "WileyNJDv5.cls",
        repro!("stand-in-copies/njdapacite/WileyNJDv5.cls"),
      ),
      (
        "NJDapacite.sty",
        repro!("stand-in-copies/njdapacite/NJDapacite.sty"),
      ),
    ],
    Some(AR5IV),
  );
  assert_clean(&log);
  assert!(!log.contains("NJDapacite.sty..."), "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">Text.</p></para>",
  );
}

/// int23 fix round 2, SHOULD 8: a typewriter `{algorithmic}` (WileyNJDv5.cls:6953 `\algrenewcommand
/// \ALG@beginalgorithmic{\ttfamily}`) keeps the space before its first `\item` out of the `ltx:listing`, whose model is
/// `listingline*`: Perl's whitespace rule where no auto-opened element reaches text (2608.15322, 2602.03247,
/// 2607.19926). pdflatex "1: x ← 1", "2: return x".
#[test]
fn a_typewriter_algorithmic_drops_the_leading_space() {
  let (log, xml) = convert_files_with(
    repro!("listings-algorithms/typewriter_algorithmic_drops_the_leading_space.tex"),
    &[],
    None,
  );
  assert_clean(&log);
  let listing = latexml::util::test::xml_element(&xml, "listing", &["framed=\"topbottom\""])
    .unwrap_or_else(|| panic!("no listing: {xml}"));
  assert!(
    listing.starts_with("<listing framed=\"topbottom\"")
      && listing.contains("\"><listingline xml:id=\"alg1.l1\">"),
    "{listing}"
  );
  if let Some(errors) = latexml::util::test::rng_error_count(&xml) {
    assert_eq!(errors, 0, "{xml}");
  }
}

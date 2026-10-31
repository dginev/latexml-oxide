//! Red/green guards for batch 95d: the last four lost-words papers of the frontmatter scorer's 3,000-paper sample,
//! three K11 store mechanisms (latexml_engine/src/frontmatter_stores.rs, OXIDIZED_DESIGN_DIVERGENCES #939-#941):
//! a raw class's token register printed only by the page style its `\maketitle` selects (`toks_store`, with
//! fancyhdr_sty.rs's faithful `\fancypagestyle`; 2607.06973, 2608.26053, 2604.13010, 2401.14196), a raw package's
//! table stores rerouted as a raw class's are (`reroute_raw_package_stores`; 2307.16262), and a table setter that
//! prints a mark before its pure store (`table_setter_stores`; 1202.2889). Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` from the binary, eager and `--streaming`, with no Error and no
//! Warning, and asserts the frontmatter it makes whole.
use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch89a::cli_conversion,
};

macro_rules! repro {
  ($name:literal) => {
    repro!("sectioning-frontmatter", $name)
  };
  ($dir:literal, $name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $name,
      ".tex"
    ))
  };
}

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Converts `tex` from the binary eager and `--streaming`, checks each has no Error and no Warning and both give the
/// same XML, and returns it squashed.
#[track_caller]
fn clean_both_ways(tex: &str) -> String {
  let (log, eager) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let (log, streamed) = cli_conversion(tex, true);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(eager, streamed);
  squash(&eager)
}

/// Converts `tex` eager, checks it has no Error and no Warning, and returns it squashed.
#[track_caller]
fn clean(tex: &str) -> String {
  let (log, xml) = cli_conversion(tex, false);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  squash(&xml)
}

/// The document from its `<title>` to its end: the frontmatter and the whole body.
fn front(xml: &str) -> &str {
  let start = xml.find("<title>").unwrap_or(0);
  let end = xml[start..]
    .find("</document>")
    .map_or(xml.len(), |end| start + end);
  xml[start..end].trim_end()
}

/// googlecloud.cls's corresponding-author token register, printed only in the foot of the fancyhdr page style its
/// `\maketitle` selects, is the frontmatter's corresponding note, led by the label the class prints before it; the
/// same register is no note when the page style is not selected, or when the foot only tests it (nvidiatechreport.cls's
/// copy in 2507.09850 prints "* Equal contribution." instead). Witnesses 2607.06973, 2608.26053, 2604.13010.
#[test]
fn toks_store_printed_in_a_selected_page_style_is_a_corresponding_note() {
  let tex = repro!("toks_store_printed_in_a_fancy_page_style_is_a_note");
  assert_eq!(
    front(&clean_both_ways(tex)),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <note role="corresponding" xml:id="id1">Corresponding author(s): ann@x.org</note> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
  let plain = r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  // (the page style no `\maketitle` selects prints nothing)
  let unselected = tex.replace(r"\par}\thispagestyle{firststyle}}", r"\par}}");
  assert_ne!(unselected, tex);
  assert_eq!(front(&clean(&unselected)), plain);
  // (a foot that only tests the register prints the class's own text, not the register)
  let tested = tex.replace(
    r"\else Corresponding author(s): \the\correspondingauthor\fi",
    r"\else Equal contribution.\fi",
  );
  assert_ne!(tested, tex);
  assert_eq!(front(&clean(&tested)), plain);
}

/// A raw package's pure date stores under elsarticle (medima.sty's "Article history") are the frontmatter's five dates,
/// in the roles fixed for them: received, final form `revised`, accepted, available online `published`, communicated —
/// the two of no default name both kept —, each named by the label the package's title page prints before it, in
/// medima's shape a head macro in the `\else` branch of the store's emptiness test, which leaves the title page with it;
/// a date whose label is not read has no name, not its role's default (review rev95d B2: "Revised " for "Received in
/// final form"). Under article, whose kernel `\maketitle` deposits the package's refused one, the same: each date printed
/// once, no lone label left in the body, the "Article history:" heading kept. Witnesses 2307.16262, 2104.10903.
#[test]
fn raw_package_date_stores_are_the_frontmatter_dates() {
  let tex = repro!("raw_package_history_dates_are_the_frontmatter_dates");
  let head =
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator>"#;
  let dates = r#"<date name="Received" role="received">1 May 2020</date> <date name="Received in final form" role="revised">9 May 2020</date> <date name="Accepted" role="accepted">2 June 2020</date> <date name="Available online" role="published">5 June 2020</date> <date name="Communicated by" role="communicated">B. Editor</date>"#;
  let text = r#"<para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  assert_eq!(
    front(&clean_both_ways(tex)),
    format!("{head} {dates} {text}")
  );
  // (a date printed with no label: unnamed)
  let bare = tex.replace(r"\receivedhead~\@received", r"\@received");
  assert_ne!(bare, tex);
  assert_eq!(
    front(&clean(&bare)),
    format!(
      "{head} {} {text}",
      dates.replace(
        r#"<date name="Received" role="received">"#,
        r#"<date role="received">"#
      )
    )
  );
  let article = tex
    .replace(r"\documentclass{elsarticle}", r"\documentclass{article}")
    .replace(r"\elsauthors", r"\@author")
    .replace(r"\begin{frontmatter}", "")
    .replace(r"\end{frontmatter}", r"\maketitle");
  assert_ne!(article, tex);
  assert_eq!(
    front(&clean(&article)),
    format!(
      r#"{head} {dates} <para xml:id="p1"> <p xml:id="p1.1"><text font="italic" xml:id="p1.1.1">Article history</text>:<break/></p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
    )
  );
}

/// A raw package's setter whose own store other code of the package prints — a macro the document calls
/// (`\printkeywords`), a `\printhistory` in its body — stays the package's: the text printed once, where it prints, no
/// frontmatter item (int124a's output); and one only uncalled code reads (rlj.sty:73's `\@keywords`, read only by the
/// `\makeCover` 2605.31044 comments out) is no item at all, as pdflatex prints it nowhere. Review rev95d B1, probes p5,
/// p7 (a raw class's, p9, stays RED: `raw_class_store_only_an_uncalled_macro_reads_is_no_item`).
#[test]
fn raw_package_store_other_code_prints_stays_where_it_prints() {
  let tex = repro!("raw_package_store_a_document_macro_prints_stays_in_the_body");
  let head = r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abs.</p> </abstract>"#;
  assert_eq!(
    front(&clean_both_ways(tex)),
    format!(
      r#"{head} <para class="ltx_noindent" xml:id="p1"> <p xml:id="p1.1"><text font="bold" xml:id="p1.1.1">Keywords:</text> alpha, beta</p> </para> <para xml:id="p2"> <p xml:id="p2.1">Text.</p> </para>"#
    )
  );
  // (the reader never called: no keywords anywhere)
  let uncalled = tex.replace("\\printkeywords\nText.", "Text.");
  assert_ne!(uncalled, tex);
  assert_eq!(
    front(&clean(&uncalled)),
    format!(r#"{head} <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#)
  );
  assert_eq!(
    front(&clean_both_ways(repro!(
      "raw_package_dates_printed_in_the_body_stay_there"
    ))),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para> <para class="ltx_noindent" xml:id="p2"> <p xml:id="p2.1">Received 1 May 2020; accepted 2 June 2020.</p> </para>"#
  );
}

/// astlb.cls's `\email`, a mark then a pure store that only the class's never-run `\endabstract` reads, prints its mark
/// at the name and makes the address that creator's e-mail. Witness 1202.2889.
#[test]
fn mark_then_store_email_is_its_creators_contact() {
  assert_eq!(
    front(&clean_both_ways(repro!(
      "mark_then_store_email_read_by_a_refused_abstract"
    ))),
    r#"<title>T</title> <creator role="author"> <personname>Ann Able<sup xml:id="id1">*</sup></personname> <contact name="Email: " role="email">ann@x.org</contact> </creator> <abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">Abs.</p> </abstract> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#
  );
}

/// A fancyhdr page style a shipped package gives a binding's class (popets.sty's `standardpagestyle` under the acmart
/// binding, its code testing acmart.cls's `\if@ACM@printfolios` and `\if@ACM@timestamp`) is stored to be read, and
/// selecting it runs nothing (fancyhdr_sty.rs, as Perl's no-op `\fancypagestyle`): no Error pdflatex does not raise, the
/// document as int124a converts it; a style using `\fancycenter` (fancyhdr.sty:378) is selected clean too. Witnesses
/// 2408.16304, 2602.20222, 2606.05273 (9 Errors each when the style ran); review rev95d S3, probe p3.
#[test]
fn a_fancy_page_style_a_package_gives_a_binding_class_selects_nothing() {
  let xml = clean_both_ways(repro!(
    "package-bindings",
    "fancy_page_style_a_package_gives_a_binding_class_selects_nothing"
  ));
  let doc = front(&xml);
  assert!(
    doc.starts_with(
      r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <date name="©" role="copyright">"#
    ),
    "{doc}"
  );
  assert!(
    doc.ends_with(r#"</pubnote> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#),
    "{doc}"
  );
  let fancycenter = "\\documentclass{article}\n\\usepackage{fancyhdr}\n\\fancypagestyle{mystyle}{\\fancyhf{}\\fancycenter{a}{b}{c}}\n\\pagestyle{mystyle}\n\\begin{document}\nText.\n\\end{document}\n";
  let xml = clean(fancycenter);
  assert!(
    xml.ends_with(r#"<para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para> </document>"#),
    "{xml}"
  );
}

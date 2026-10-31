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
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
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

/// The document's frontmatter and first paragraph: from `<title>` up to the end of the first `</para>`.
fn front(xml: &str) -> &str {
  let start = xml.find("<title>").unwrap_or(0);
  let end = xml[start..]
    .find("</para>")
    .map_or(xml.len(), |end| start + end + "</para>".len());
  &xml[start..end]
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
/// the two of no default name both kept —, each named by the label the package's title page prints before it, which
/// leaves the title page with it. Under article, whose kernel `\maketitle` deposits the package's refused one, the same:
/// each date printed once, no lone label left in the body. Witness 2307.16262.
#[test]
fn raw_package_date_stores_are_the_frontmatter_dates() {
  let tex = repro!("raw_package_history_dates_are_the_frontmatter_dates");
  let dates = r#"<title>T</title> <creator role="author"> <personname>Ann Able</personname> </creator> <date name="Received" role="received">1 May 2020</date> <date name="Received in final form" role="revised">9 May 2020</date> <date name="Accepted" role="accepted">2 June 2020</date> <date name="Available online" role="published">5 June 2020</date> <date name="Communicated by" role="communicated">B. Editor</date> <para xml:id="p1"> <p xml:id="p1.1">Text.</p> </para>"#;
  assert_eq!(front(&clean_both_ways(tex)), dates);
  let article = tex
    .replace(r"\documentclass{elsarticle}", r"\documentclass{article}")
    .replace(r"\elsauthors", r"\@author")
    .replace(r"\begin{frontmatter}", "")
    .replace(r"\end{frontmatter}", r"\maketitle");
  assert_ne!(article, tex);
  assert_eq!(front(&clean(&article)), dates);
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

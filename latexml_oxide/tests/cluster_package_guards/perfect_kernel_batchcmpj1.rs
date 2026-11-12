//! Guards for CMPJ1: cmpj's `\refaddr` address marks print as TeX's second run does (`cmpj_sty.rs`, one override of
//! `\@@refadx` over the real cmpj.sty, cmpj2.sty, cmpj3.sty; KNOWN_PERL_ERRORS #1233). Each row converts its repro and
//! asserts the exact Error and Warning counts, every name whole, and every creator's contacts in order. Red on int164a:
//! each name "?" (the first run's `\@ifundefined{r@…}` branch), the address list on a nameless creator.

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batchz4::run, perfect_kernel_batchz18o::assert_creators};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The count of an entry placed whole on names that request only some of its `\label`s (the resolver's
/// `warn_whole_entry_of_keyed_parts`): cmpj's address list, which no reading cuts per `\item` yet
/// (`cmpj_address_list_each_item_on_its_names_red`).
const WHOLE: &str = "The frontmatter reading places an entry holding 2 label targets (la, lb) whole: each name \
                     whose reference requests one of them holds them all";

const LIST: &str = "address=1 item 1 1 item 1 First Institute, Lviv, Ukraine 2 item 2 2 item 2 Second Institute, Kyiv, \
                    Ukraine";

/// A row: its name, its repro, every name whole, every creator's name and contacts, each Warning's message.
type Row<'a> = (
  &'a str,
  &'a str,
  Vec<String>,
  &'a [&'a [&'a str]],
  Vec<&'a str>,
);

/// Each binding's row: cmpj.sty (three names), cmpj2.sty with the document's options (`cm`, `eqsecnum`) and one
/// address, cmpj3.sty with a spaced key list (`{la, lb}`, the space dropped as cmpj3.sty:663's `\@firstofone` drops it);
/// and the count's safe sides: one address, and one name requesting both.
#[test]
fn cmpj_address_marks_print_the_second_run() {
  // a name of one mark, whole
  let one = |name: &str, id: u8, size: &str, key: &str| {
    format!(
      "<personname>{name}<sup xml:id=\"id{id}\"><ref font=\"sansserif\" fontsize=\"{size}\" \
       labelref=\"LABEL:{key}\"/></sup></personname>"
    )
  };
  let rows: [Row; 4] = [
    (
      "cmpj",
      repro!("cmpj_refaddr_marks_print_the_second_run"),
      vec![
        one("A. One", 1, "70%", "la"),
        one("B. Two", 3, "70%", "lb"),
        "<personname>C. Three<sup xml:id=\"id5\"><text font=\"sansserif\" fontsize=\"70%\" xml:id=\"id5.1\"><ref \
         labelref=\"LABEL:la\"/></text>,<text font=\"sansserif\" fontsize=\"70%\" xml:id=\"id5.2\"><ref \
         labelref=\"LABEL:lb\"/></text></sup></personname>"
          .to_string(),
      ],
      &[&["A. One", LIST], &["B. Two", LIST], &["C. Three ,", LIST]],
      vec![WHOLE],
    ),
    (
      "cmpj2",
      repro!("cmpj2_refaddr_one_address_with_options"),
      vec![
        one("A. One", 1, "80%", "la"),
        one("B. Two", 3, "80%", "la"),
      ],
      &[
        &["A. One", "address=1 item 1 1 item 1 First Institute, Lviv, Ukraine"],
        &["B. Two", "address=1 item 1 1 item 1 First Institute, Lviv, Ukraine"],
      ],
      vec![],
    ),
    (
      "cmpj3",
      repro!("cmpj3_refaddr_spaced_keys"),
      vec![
        one("A. One", 1, "80%", "la"),
        one("B. Two", 3, "80%", "lb"),
        "<personname>C. Three<sup xml:id=\"id5\"><text font=\"sansserif\" fontsize=\"80%\" xml:id=\"id5.1\"><ref \
         labelref=\"LABEL:la\"/></text><text font=\"sansserif\" fontsize=\"80%\" xml:id=\"id5.2\">,<text \
         xml:id=\"id5.2.1\"><ref labelref=\"LABEL:lb\"/></text></text></sup></personname>"
          .to_string(),
      ],
      &[&["A. One", LIST], &["B. Two", LIST], &["C. Three ,", LIST]],
      // (and Perl's own "misdefined" Warning on pgf's `\newif\pgf@lib@svg@relative`, Package.pm:1219, the svg.path
      // library cmpj3.sty:54 loads)
      vec![WHOLE, "The conditional \\pgf@lib@svg@relative is being defined but doesn't start with \\if"],
    ),
    // (one name requesting every label of the list holds it whole, as pdflatex prints it: no count)
    (
      "cmpj, one name",
      repro!("cmpj_refaddr_one_name_both_addresses"),
      vec![
        "<personname>A. One<sup xml:id=\"id1\"><text font=\"sansserif\" fontsize=\"70%\" xml:id=\"id1.1\"><ref \
         labelref=\"LABEL:la\"/></text>,<text font=\"sansserif\" fontsize=\"70%\" xml:id=\"id1.2\"><ref \
         labelref=\"LABEL:lb\"/></text></sup></personname>"
          .to_string(),
      ],
      &[&["A. One ,", LIST]],
      vec![],
    ),
  ];
  for (style, tex, names, expected, warnings) in rows {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings.len()),
      "{style}: {log}"
    );
    for warning in warnings {
      assert_eq!(log.matches(warning).count(), 1, "{style}: {warning}\n{log}");
    }
    let shown: Vec<&str> = xml
      .match_indices("<personname>")
      .map(|(at, _)| {
        &xml[at..at + xml[at..].find("</personname>").expect("closed") + "</personname>".len()]
      })
      .collect();
    assert_eq!(shown, names, "{style}: {xml}");
    assert!(
      !xml.contains("<personname>A. One<Math"),
      "{style}: a \"?\" mark: {xml}"
    );
    assert_creators(&xml, expected);
    if style == "cmpj2" {
      assert!(
        xml.contains("<tag role=\"refnum\">1.1</tag>"),
        "eqsecnum: {xml}"
      );
    }
  }
}

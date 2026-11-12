//! Guards for CMPJ1: cmpj's `\refaddr` address marks print as TeX's second run does (`cmpj_sty.rs`, one override of
//! `\@@refadx` over the real cmpj.sty, cmpj2.sty, cmpj3.sty; KNOWN_PERL_ERRORS #1233). Each row converts its repro and
//! asserts the exact Error and Warning counts, every name whole, and every creator's contacts in order. Red on int164a:
//! each name "?" (the first run's `\@ifundefined{r@…}` branch), the address list on a nameless creator. CMPJ2: the list
//! cut per `\item` (`events::item_led_pieces`), each item on the names its `\label` is requested by, its `labels=` kept;
//! red on CMPJ1: the whole list on every name, its items without `labels=`. CMPJ3: each item numbered with the value the
//! digestion stepped its counter to; red on CMPJ2: the items before it counted, (D)(B)(C) for pdflatex's (D)(E)(F).

use latexml::util::test::{error_count, warning_count};
use regex::Regex;

use super::{
  perfect_kernel_batch64::post_with, perfect_kernel_batchz4::run,
  perfect_kernel_batchz18o::assert_creators,
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

/// The count of an entry placed whole on names that request only some of its `\label`s (the resolver's
/// `warn_whole_entry_of_keyed_parts`): a list with a tagged item, which no reading cuts per `\item`
/// (`cmpj_address_list_tagged_item_kept_whole_red`).
const WHOLE: &str = "The frontmatter reading places an entry holding 2 label targets (la, lb) whole: each name \
                     whose reference requests one of them holds them all";

/// The list's items, each a contact of its own (CMPJ2, `events::item_led_pieces`).
const ONE: &str = "address=1 item 1 1 item 1 First Institute, Lviv, Ukraine";
const TWO: &str = "address=2 item 2 2 item 2 Second Institute, Kyiv, Ukraine";
const TAGGED: &str = "address=a item item a First Institute, Lviv, Ukraine 1 item 1 1 item 1 Second Institute, Kyiv, Ukraine";

/// A row: its name, its repro, every name whole, every creator's name and contacts, each Warning's message, the
/// `labels=` of the list items in document order, and the text's `\ref`s as the HTML prints them, if checked.
type Row<'a> = (
  &'a str,
  &'a str,
  Vec<String>,
  &'a [&'a [&'a str]],
  Vec<&'a str>,
  &'a [&'a str],
  Option<&'a str>,
);

/// Each binding's row: cmpj.sty (three names), cmpj2.sty with the document's options (`cm`, `eqsecnum`) and one
/// address, cmpj3.sty with a spaced key list (`{la, lb}`, the space dropped as cmpj3.sty:663's `\@firstofone` drops it);
/// one name requesting both; the address list cut per `\item` (CMPJ2: each item a contact keeping its number and
/// `labels=`): items by hand with a label after the text, a later flush, a list closed by a macro, amsart's address, its
/// counter set after the opening or stepped in an item (CMPJ3); and the counted sides, a group across items and a tagged
/// item.
#[test]
fn cmpj_address_marks_print_the_second_run() {
  // a name of one mark, whole
  let one = |name: &str, id: u8, size: &str, key: &str| {
    format!(
      "<personname>{name}<sup xml:id=\"id{id}\"><ref font=\"sansserif\" fontsize=\"{size}\" \
       labelref=\"LABEL:{key}\"/></sup></personname>"
    )
  };
  let rows: [Row; 13] = [
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
      &[&["A. One", ONE], &["B. Two", TWO], &["C. Three ,", ONE, TWO]],
      vec![],
      &["la", "lb", "la", "lb"],
      None,
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
      &["la", "la"],
      None,
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
      &[&["A. One", ONE], &["B. Two", TWO], &["C. Three ,", ONE, TWO]],
      // (Perl's own "misdefined" Warning on pgf's `\newif\pgf@lib@svg@relative`, Package.pm:1219, the svg.path
      // library cmpj3.sty:54 loads)
      vec!["The conditional \\pgf@lib@svg@relative is being defined but doesn't start with \\if"],
      &["la", "lb", "la", "lb"],
      None,
    ),
    // (one name requesting every label of the list holds each item)
    (
      "cmpj, one name",
      repro!("cmpj_refaddr_one_name_both_addresses"),
      vec![
        "<personname>A. One<sup xml:id=\"id1\"><text font=\"sansserif\" fontsize=\"70%\" xml:id=\"id1.1\"><ref \
         labelref=\"LABEL:la\"/></text>,<text font=\"sansserif\" fontsize=\"70%\" xml:id=\"id1.2\"><ref \
         labelref=\"LABEL:lb\"/></text></sup></personname>"
          .to_string(),
      ],
      &[&["A. One ,", ONE, TWO]],
      vec![],
      &["la", "lb"],
      None,
    ),
    (
      "cmpj2, the list cut per item",
      repro!("cmpj_address_list_each_item_on_its_names"),
      vec![
        one("A. One", 1, "80%", "la"),
        one("B. Two", 3, "80%", "lb"),
        "<personname>C. Three<sup xml:id=\"id5\"><text font=\"sansserif\" fontsize=\"80%\" xml:id=\"id5.1\"><ref \
         labelref=\"LABEL:la\"/></text><text font=\"sansserif\" fontsize=\"80%\" xml:id=\"id5.2\">,<text \
         xml:id=\"id5.2.1\"><ref labelref=\"LABEL:lb\"/></text></text></sup></personname>"
          .to_string(),
      ],
      &[&["A. One", ONE], &["B. Two", TWO], &["C. Three ,", ONE, TWO]],
      vec![],
      &["la", "lb", "la", "lb"],
      None,
    ),
    // (`\item`s by hand, a label after its item's text, a group in it: each item from its `\item`, its italic kept)
    (
      "cmpj2, items by hand",
      repro!("cmpj_address_list_items_by_hand"),
      vec![
        one("A. One", 1, "80%", "la"),
        one("B. Two", 3, "80%", "lb"),
      ],
      &[&["A. One", ONE], &["B. Two", TWO]],
      vec![],
      &["la", "lb"],
      None,
    ),
    // (cut at a later flush whose rehearsal reads `\addr` as a bare `\item`: the reading's cut, `split_pieces_as`; the
    // Ukrainian names' English items are RED `cmpj_ukrainianpart_addresses_of_the_second_title_red`)
    (
      "cmpj2, a later flush",
      repro!("cmpj_address_list_cut_at_a_later_flush"),
      vec![
        one("A. One", 1, "80%", "la"),
        one("B. Two", 3, "80%", "lb"),
        one("A. Odin", 5, "80%", "la"),
        one("B. Dva", 7, "80%", "lb"),
      ],
      &[
        &["A. One", ONE],
        &["B. Two", TWO],
        &["A. Odin", ONE],
        &["B. Dva", TWO],
      ],
      vec![],
      // (the second flush's items carry none: `\addr` is a bare `\item` there, as pdflatex reads it)
      &["la", "lb"],
      None,
    ),
    // (a list closed by a macro of the paper's: its close read from the digestion's frames)
    (
      "cmpj2, a list closed by a macro",
      repro!("cmpj_address_list_closed_by_a_macro"),
      vec![
        one("A. One", 1, "80%", "la"),
        one("B. Two", 3, "80%", "lb"),
      ],
      &[
        &["A. One", "address=1. item 1 1 item 1 First Institute, Lviv, Ukraine"],
        &["B. Two", "address=2. item 2 2 item 2 Second Institute, Kyiv, Ukraine"],
      ],
      vec![],
      &["la", "lb"],
      None,
    ),
    // (a group across two items: no item's tokens a balanced run, each item its text on its name, counted)
    (
      "cmpj2, a group across items",
      repro!("cmpj_address_list_group_across_items_red"),
      vec![
        one("A. One", 1, "80%", "la"),
        one("B. Two", 2, "80%", "lb"),
      ],
      &[
        &["A. One", "address=First Institute, Lviv, Ukraine"],
        &["B. Two", "address=Second Institute, Kyiv, Ukraine"],
      ],
      vec![
        "The frontmatter emitter writes \"First Institute, Lviv, Ukraine\" as its plain text, its markup lost: a list \
         item cut without its list, its number and label lost",
        "The frontmatter emitter writes \"Second Institute, Kyiv, Ukraine\" as its plain text, its markup lost: a list \
         item cut without its list, its number and label lost",
      ],
      &[],
      None,
    ),
    // (another class's address, its first `\item` right after the list's opening, whose box carries the opening's
    // position: each item from past the opening)
    (
      "amsart, an address list",
      repro!("amsart_address_list_cut_per_item"),
      vec!["<personname>A. One</personname>".to_string()],
      &[&[
        "A. One",
        "address=(1) 1 item 1 First Institute, Lviv",
        "address=(2) 2 item 2 Second Institute, Kyiv",
      ]],
      vec![],
      &["xa", "xb"],
      None,
    ),
    // (the counter set after the opening, `\theenumi` lettered, and stepped inside an item: each item numbered with
    // the value the digestion stepped its counter to, as pdflatex prints it — not the items before it, revCMPJ)
    (
      "amsart, the counter set after the opening",
      repro!("amsart_address_list_counter_set_after_the_opening"),
      vec!["<personname>A. One</personname>".to_string()],
      &[&[
        "A. One",
        "address=(D) D item D First Institute, Lviv",
        "address=(E) E item E Second Institute, Kyiv",
        "address=(F) F item F Third, Odesa",
      ]],
      vec![],
      &["xa", "xb", "xc"],
      Some("Text, see D, E, F."),
    ),
    (
      "amsart, the counter stepped in an item",
      repro!("amsart_address_list_counter_stepped_in_an_item"),
      vec!["<personname>A. One</personname>".to_string()],
      &[&[
        "A. One",
        "address=(1) 1 item 1 First Institute, Lviv",
        "address=(3) 3 item 3 Second Institute, Kyiv",
        "address=(4) 4 item 4 Third, Odesa",
      ]],
      vec![],
      &["xa", "xb", "xc"],
      Some("Text, see 1, 3, 4."),
    ),
    // (a tagged item steps no counter: no cut, the list whole on both names, counted)
    (
      "cmpj2, a tagged item",
      repro!("cmpj_address_list_tagged_item_kept_whole_red"),
      vec![
        one("A. One", 1, "80%", "la"),
        one("B. Two", 3, "80%", "lb"),
      ],
      &[&["A. One", TAGGED], &["B. Two", TAGGED]],
      vec![WHOLE],
      &["la", "lb", "la", "lb"],
      None,
    ),
  ];
  for (style, tex, names, expected, warnings, labels, refs) in rows {
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
    // (each item's label, in document order)
    let items: Vec<&str> = xml
      .match_indices("<item ")
      .filter_map(|(at, _)| {
        let tag = &xml[at..at + xml[at..].find('>').expect("closed")];
        let from = tag.find("labels=\"LABEL:")? + "labels=\"LABEL:".len();
        Some(&tag[from..from + tag[from..].find('"').expect("quoted")])
      })
      .collect();
    assert_eq!(items, labels, "{style}: {xml}");
    if let Some(refs) = refs {
      let html = post_with(&xml, "resources/XSLT/LaTeXML-html5.xsl");
      let at = html.find("Text, see").expect("the text");
      let text = Regex::new("<[^>]*>")
        .expect("regex")
        .replace_all(&html[at..at + html[at..].find("</p>").expect("closed")], "")
        .into_owned();
      assert_eq!(text, refs, "{style}: {html}");
    }
    if style == "cmpj2" {
      assert!(
        xml.contains("<tag role=\"refnum\">1.1</tag>"),
        "eqsecnum: {xml}"
      );
    }
  }
}

//! BMCART: bmcart.cls's article notes and correspondence as the class's thanksbox prints them (OXIDIZED_DESIGN_DIVERGENCES
//! #842): each note one shared note under the mark the class prints, the corresponding author's e-mail named
//! "Correspondence: ". Each guard converts its repro under `tools/perfect_kernel/repros/sectioning-frontmatter/
//! bmcart_marks_copy/` beside the unmodified class 2112.10068 ships, and asserts no Error or Warning and every creator
//! whole (one beside the 2017/07/31 class, `bmcart2017_copy/`); the mark and the name come from that copy alone (its `\bmcsymbol@item@<n>`, its `\corref@text`), so each
//! assertion also proves the copy is read. Witness 2112.10068 (DG's sandbox review).
use latexml::util::test::{convert_files_with, normalize_markup, xml_element};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/bmcart_marks_copy/",
      $path
    ))
  };
}

/// The class as 2112.10068 ships it (2014/01/24), unmodified.
const CLASS: &str = repro!("bmcart.cls");
/// The class as 2212.10682 ships it (2017/07/31), unmodified.
const CLASS_2017: &str = include_str!(
  "../../../tools/perfect_kernel/repros/sectioning-frontmatter/bmcart2017_copy/bmcart.cls"
);

/// `tex` converted beside the 2014 class, with no Error or Warning; its XML.
#[track_caller]
fn convert(tex: &str) -> String { convert_beside(tex, CLASS) }

/// `tex` converted beside `class`, with no Error or Warning; its XML.
#[track_caller]
fn convert_beside(tex: &str, class: &str) -> String {
  let (log, xml) = convert_files_with(tex, &[("bmcart.cls", class)], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// Every `<creator>` of `xml`, in document order, is EXACTLY the one `expected` lists at its place, and there are no
/// others.
#[track_caller]
fn assert_creators(xml: &str, expected: &[&str]) {
  let found: Vec<String> = xml
    .match_indices("<creator ")
    .filter_map(|(at, _)| xml_element(&xml[at..], "creator", &[]))
    .collect();
  let want: Vec<String> = expected.iter().map(|e| normalize_markup(e)).collect();
  assert_eq!(found, want, "{xml}");
}

/// 2112.10068's shape: two names cite `\note[id=n1]{Equal contributor}` (`noteref={n1}`), a third has `corref`. The
/// thanksbox (`\auto@set@thanksbox`, bmcart.cls:1435-1443) prints "*Correspondence: c@x.org" first, then "† Equal
/// contributor": the note is one shared note marked †, printed once with its mark on both names, and the corresponding
/// author's e-mail is named as the class prints it. pdflatex: "Ann Able1†, Bob Baker2† and Cy Cole3*".
#[test]
fn shared_note_is_one_marked_note() {
  let xml = convert(repro!("bmcart_shared_note_is_marked.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">a@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, US</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id1\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">b@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, IN</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id2\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Cole</personname>\
     <contact name=\"Correspondence: \" role=\"email\">c@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ C, FR</contact></creator>",
  ]);
}

/// A note's `symbol` key is its mark (`\thanksmark[$\star$]`, :1393-1398), which steps no counter: after the
/// correspondence's *, the next notes take † and ^ (`\^{}`, :1097). pdflatex: "⋆ Joint first authors† Equal
/// contributorˆ Deceased".
#[test]
fn note_symbol_is_its_mark_and_steps_nothing() {
  let xml = convert(repro!("bmcart_note_symbol_takes_no_step.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">a@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, US</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{22c6}\" \
     role=\"thanks\" xml:id=\"id1\">Joint first authors</note>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id2\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">b@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, IN</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{22c6}\" \
     role=\"thanks\" xml:id=\"id3\">Joint first authors</note>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id4\">Equal contributor</note>\
     <note class=\"ltx_note_frontmatter ltx_thanks_deceased\" mark=\"\u{2c6}\" role=\"thanks\" \
     xml:id=\"id5\">Deceased</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Cole</personname>\
     <contact name=\"Correspondence: \" role=\"email\">c@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ C, FR</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_deceased\" mark=\"\u{2c6}\" role=\"thanks\" \
     xml:id=\"id6\">Deceased</note>\
     </creator>",
  ]);
}

/// No `corref` key and no `email`: no correspondence prints, so the first note takes *. pdflatex: "Ann Able1* and Bob
/// Baker2*", "* Equal contributor".
#[test]
fn note_without_corresponding_author_takes_the_first_mark() {
  let xml = convert(repro!("bmcart_note_without_corresponding_author.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, US</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"*\" role=\"thanks\" \
     xml:id=\"id1\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, IN</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"*\" role=\"thanks\" \
     xml:id=\"id2\">Equal contributor</note>\
     </creator>",
  ]);
}

/// No `corref` key: from the second run the first author with an `email` corresponds (`\authorcorref@notset`, written
/// to the aux, :944, :1166, :1212-1217; 1711.01177, 1811.01467, 2105.06840), so the note takes †. pdflatex: "Ann
/// Able1†, Bob Baker2*† and Cy Cole3", "*Correspondence: b@x.org".
#[test]
fn first_emailed_author_corresponds_without_corref() {
  let xml = convert(repro!("bmcart_first_emailed_author_corresponds.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, US</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id1\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>\
     <contact name=\"Correspondence: \" role=\"email\">b@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, IN</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id2\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Cole</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">c@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ C, FR</contact></creator>",
  ]);
}

/// One address: the class makes `corref` a key that does nothing (:1066-1074), so the first author with an `email`
/// corresponds, not the `corref` one. pdflatex: "Ann Able*† and Bob Baker†", "*Correspondence: a@x.org".
#[test]
fn one_address_ignores_corref() {
  let xml = convert(repro!("bmcart_one_address_ignores_corref.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname>\
     <contact name=\"Correspondence: \" role=\"email\">a@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, US</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id1\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">b@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, US</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id2\">Equal contributor</note>\
     </creator>",
  ]);
}

/// Two `corref` authors: each sets `\corr@author@id` (:1218-1222), so the last one's correspondence prints (the first
/// name's mark is "??"). pdflatex: "Ann Able1??, Bob Baker2*† and Cy Cole3†", "*Correspondence: b@x.org".
#[test]
fn last_corref_author_corresponds() {
  let xml = convert(repro!("bmcart_last_corref_corresponds.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">a@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, US</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>\
     <contact name=\"Correspondence: \" role=\"email\">b@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, IN</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id1\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Cole</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">c@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ C, FR</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id2\">Equal contributor</note>\
     </creator>",
  ]);
}

/// A paper that empties `\corref@text` (1902.08707's `\setattribute{corref}{text}{}`) prints no "Correspondence:"
/// (:1458-1463): the corresponding author's e-mail keeps its own name, not none. pdflatex: "* c@x.org".
#[test]
fn empty_correspondence_text_names_nothing() {
  let xml = convert(repro!("bmcart_empty_correspondence_text_names_nothing.tex"));
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">a@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, US</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id1\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">b@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, IN</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id2\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Cole</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">c@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ C, FR</contact></creator>",
  ]);
}

/// The 2017/07/31 class makes every `corref` author a corresponding author (:1225; 2012.08156, 2101.05946): one mark,
/// "Correspondence: a@x.org; b@x.org", so both e-mails are named so and the note takes †. pdflatex: "Ann Able1*†, Bob
/// Baker2*† and Cy Cole3".
#[test]
fn every_corref_author_corresponds_in_the_2017_class() {
  let xml = convert_beside(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/bmcart2017_copy/\
       bmcart2017_every_corref_author_corresponds.tex"
    ),
    CLASS_2017,
  );
  assert_creators(&xml, &[
    "<creator role=\"author\"><personname>Ann Able</personname>\
     <contact name=\"Correspondence: \" role=\"email\">a@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A, US</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id1\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker</personname>\
     <contact name=\"Correspondence: \" role=\"email\">b@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B, IN</contact>\
     <note class=\"ltx_note_frontmatter ltx_thanks_contribution\" mark=\"\u{2020}\" \
     role=\"thanks\" xml:id=\"id2\">Equal contributor</note>\
     </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Cy Cole</personname>\
     <contact name=\"Email:\u{a0}\" role=\"email\">c@x.org</contact>\
     <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ C, FR</contact></creator>",
  ]);
}

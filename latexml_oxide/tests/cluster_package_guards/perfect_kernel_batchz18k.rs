//! Guards for Z18k: eptcs's author block and OmniBus's `\authors` list go whole to the reading (`\lx@add@author@split`,
//! OXIDIZED_DESIGN_DIVERGENCES #984, #991), off `author_block/`'s token split; and the reading's e-mail lists read as
//! they print. One guard per path: an e-mail list a control word parts, its addresses in the class's font (eptcs's
//! `\email`, 1101.4424; llncs's, 2310.01324, whose `\quad` led the address's printed run); an e-mail list of boxes and
//! skips (1210.2456); OmniBus's `\affil{1,2}` the request of the numbered affiliations (2512.09915). Each converts its
//! repro and asserts the exact Error and Warning counts and every creator's name and contacts in order. The eptcs
//! table and cell shapes are `perfect_kernel_batch84b`'s guards, which the reroute keeps green unchanged.

use latexml::util::test::{convert_files_with, error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// The eptcs repros' stand-in class, written from scratch (no code of eptcs.cls).
const EPTCS: &str = repro!("eptcs/eptcs.cls");

#[track_caller]
fn assert_creators(xml: &str, expected: &[&[&str]]) {
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(xml), want, "{xml}");
}

/// The text of each e-mail contact of `xml`, in order, and whether it is set in typewriter type.
fn email_fonts(xml: &str) -> Vec<bool> {
  xml
    .match_indices("role=\"email\"")
    .map(|(at, _)| {
      let end = at + xml[at..].find("</contact>").expect("closed contact");
      xml[at..end].contains("font=\"typewriter\"")
    })
    .collect()
}

/// eptcs's `Eve Eng \qquad Fay Fox` over `\email{e.eng@e.org \qquad fx1@e.org}` (1101.4424's `\email{g.stavrinos@…
/// \qquad\qquad\qquad tassiana98@gmail.com}`): one address each, in their order, and every address of the block in
/// typewriter type, as the class's `\email` sets the list. Red (Z18k's WIP): Fay Fox's address `\qquadfx1@e.org`
/// written as text in roman type, a note "qquad" on a nameless creator, the lists' addresses roman.
#[test]
fn eptcs_email_list_a_control_word_parts_keeps_the_font() {
  let tex = repro!("eptcs/eptcs_row_of_names.tex");
  let (log, xml) = convert_files_with(tex, &[("eptcs.cls", EPTCS)], Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &[
      "Carl Cole",
      "affiliation=Univ C",
      "email=cole@c.edu",
      "affiliation=Lab D",
    ],
    &[
      "Dan Dorn",
      "affiliation=Univ C",
      "email=dorn@c.edu",
      "affiliation=Lab D",
      "email=dorn@d.org",
    ],
    &["Eve Eng", "affiliation=Univ E", "email=e.eng@e.org"],
    &["Fay Fox", "affiliation=Univ E", "email=fx1@e.org"],
  ]);
  assert_eq!(email_fonts(&xml), [true; 5], "{xml}");
}

/// eptcs's names and e-mails each in a box, a skip between them (1210.2456's `\makebox[0cm][c]{…}\hspace{40mm}…`):
/// each address the name it spells. Red (int155a0): one creator "Ann Able Bob Baker" with both addresses; Z18k's WIP:
/// the two, and a nameless creator's note `\makebox[0.0pt][c]{ann@a.edu} …`.
#[test]
fn eptcs_email_row_of_boxes() {
  let tex = repro!("eptcs/eptcs_email_row_of_boxes.tex");
  let (log, xml) = convert_files_with(tex, &[("eptcs.cls", EPTCS)], Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &["Ann Able", "affiliation=Univ A", "email=ann@a.edu"],
    &["Bob Baker", "affiliation=Univ A", "email=bob@a.edu"],
  ]);
  assert_eq!(email_fonts(&xml), [true; 2], "{xml}");
}

/// llncs's `\email{able@x.org \quad baker@x.org \quad cole@y.org}` under the shared `\institute` (2310.01324): each
/// address its speller's, written as it prints without the `\quad` its run opens with, in typewriter type. Red
/// (int155a0): Bob Baker's and Cy Cole's addresses each twice, once `\quadbaker@x.org` written as text; 3 Warnings.
#[test]
fn llncs_email_list_quad_parted_keeps_the_font() {
  let (log, xml) = run(repro!("llncs_email_list_quad_parted_keeps_the_font.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_creators(&xml, &[
    &[
      "Ann Able",
      "affiliation=Univ X",
      "affiliation=Lab Y",
      "email=able@x.org",
    ],
    &["Bob Baker", "affiliation=Univ X", "email=baker@x.org"],
    &[
      "Cy Cole",
      "affiliation=Univ X",
      "affiliation=Lab Y",
      "email=cole@y.org",
    ],
  ]);
  assert_eq!(email_fonts(&xml), [true; 3], "{xml}");
}

/// OmniBus's `\authors{Ann Able\affil{1,2}, Bob Baker\affil{2}, and Cy Cole\affil{1}}` with `\affiliation{1}{…}`
/// `\affiliation{2}{…}` (agujournal2019; 2512.09915's `\affil{7,8}`, 1810.04324): each `\affil` of numbers requests
/// those affiliations. Red (int155a0): Ann Able's affiliation the text "1,2". The one Warning is the unknown class's
/// (`using OmniBus`); its stand-in `omnibus/aguauthors.cls` is for pdflatex only and is not given to the conversion.
#[test]
fn omnibus_authors_affil_number_lists() {
  let (log, xml) = run(repro!("omnibus/omnibus_authors_affil_number_lists.tex"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(log.contains("using OmniBus"), "{log}");
  assert_creators(&xml, &[
    &["Ann Able", "affiliation=Univ One", "affiliation=Lab Two"],
    &["Bob Baker", "affiliation=Lab Two"],
    &["Cy Cole", "affiliation=Univ One"],
  ]);
}

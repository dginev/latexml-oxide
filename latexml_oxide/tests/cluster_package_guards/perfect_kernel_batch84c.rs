//! Red/green guards for batch 84c: the lmcs.cls binding's title page (`latexml_contrib/src/lmcs_cls.rs`,
//! OXIDIZED_DESIGN_DIVERGENCES #899). `\author[short]{Name}[a,b]` requests the addresses its letters label, an address
//! is labelled by the letters written before it or by its place in the list, an e-mail goes to the author it spells
//! (else the one its address's letter names), `\lmcsorcid` in a name is the author's ORCID, and `\lmcsheading`'s
//! submission and publication dates are the paper's. Each guard converts its repro under
//! `tools/perfect_kernel/repros/class-bindings/` and asserts no Error or Warning and every creator's name and contacts
//! (role and text) in order. Witnesses 2306.08266 (eight authors, `[f,g]`, `\lmcsorcid`), 2402.09138 (one `\email` of
//! three addresses), 1103.0510, cs0701140, 1202.6352 (letters written in the names and addresses, `{\lsuper{b,c}}`),
//! 2001.03894, 2402.11987.
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/",
      $path
    ))
  };
}

/// Converts the repro, checks it converts clean and compares its creators; returns the XML.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&[&str]]) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(creators(&xml), want, "{xml}");
  xml
}

/// lmcs v0.92's `\author[short]{Name\lmcsorcid{…}}[a]`, `[b,a]`, none, `[c]` over three addresses lettered by their
/// place, an `\email` list under the first (2306.08266, 2402.09138): each author the addresses its letters name and the
/// e-mail it spells, Ann's ORCID a contact, and `\lmcsheading`'s two dates the paper's. Red (int111d): "ORCID:…" in the
/// names, every address on the last author, the letters a paragraph of their own.
#[test]
fn lmcs_lettered_addresses() {
  let xml = assert_creators(repro!("lmcs_lettered_addresses.tex"), &[
    &[
      "Ann Able",
      "orcid=",
      "address=Univ A",
      "email=ann.able@a.edu",
    ],
    &[
      "Bob Baker",
      "address=Univ A",
      "email=bob@a.edu",
      "address=Univ B",
    ],
    &["Carl Cole"],
    &["Dan Dorn", "address=Univ C", "email=zz9@c.org"],
  ]);
  let first = &xml[xml.find("<creator ").expect("a creator")..];
  let first = &first[..first.find("</creator>").expect("closed creator")];
  assert!(
    first.contains("href=\"https://orcid.org/0000-0002-1825-0097\""),
    "{first}"
  );
  // (each date's text, by its role)
  let date = |role: &str| -> String {
    let at = xml
      .match_indices("<date ")
      .map(|(at, _)| at)
      .find(|&at| {
        xml[at..]
          .split('>')
          .next()
          .unwrap_or("")
          .contains(&format!("role=\"{role}\""))
      })
      .unwrap_or_else(|| panic!("no {role} date: {xml}"));
    let open_end = at + xml[at..].find('>').expect("date tag");
    let close = open_end + xml[open_end..].find("</date>").expect("closed date");
    xml[open_end + 1..close].trim().to_string()
  };
  assert_eq!(date("submitted"), "Jun.\u{a0}16,\u{a0}2023");
  assert_eq!(date("published"), "Mar.\u{a0}20,\u{a0}2024");
}

/// Names with no letters, the addresses after them (2402.11987): amsart's reading, each address the run of authors' it
/// follows, each e-mail the author it spells.
#[test]
fn lmcs_unlettered_addresses() {
  assert_creators(repro!("lmcs_unlettered_addresses.tex"), &[
    &["Ann Able", "address=Univ A", "email=ann.able@a.edu"],
    &["Bob Baker", "address=Univ A", "email=bob.baker@a.edu"],
  ]);
}

/// The earlier classes' letters written by hand, `Name\rsuper b` and `\address{{\lsuper{b,c}}Univ BC}` (1202.6352,
/// 1103.0510, cs0701140): an address with two letters is each one's, and an address of spacing alone is none. Red
/// twice: int111d left the letters as math in the names and Carl without Univ BC; 84c r1 kept the two-letter address
/// at the block, off Bob and Carl.
#[test]
fn lmcs_written_letters() {
  assert_creators(repro!("lmcs_written_letters.tex"), &[
    &["Ann Able", "address=Univ A", "email=able@a.edu"],
    &["Bob Baker", "address=Univ BC", "email=baker@b.edu"],
    &["Carl Cole", "address=Univ BC"],
  ]);
}

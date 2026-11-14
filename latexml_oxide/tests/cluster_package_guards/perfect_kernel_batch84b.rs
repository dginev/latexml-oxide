//! Red/green guards for batch 84b: the eptcs.cls binding (`latexml_contrib/src/eptcs_cls.rs`, OXIDIZED_DESIGN_DIVERGENCES
//! #898). eptcs's `\institute{…}` and `\email{…}`, written inside `\author` after the names, are each a line under
//! them; read as the kernel's author line they stayed in the name. Each guard converts its repro under
//! `tools/perfect_kernel/repros/class-bindings/eptcs/` beside the repros' stand-in `eptcs.cls` and asserts no Error or
//! Warning, and every creator's name and contacts (role and text) in order. Witnesses 1408.5969, 1909.07676, 2406.16409
//! (an institute and an e-mail under each name), 2109.08295 (a row of names, an e-mail list under it), 1407.7928 (a
//! `tabular` of names), 2409.15014 (a paper's own `authors` environment).
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/class-bindings/eptcs/",
      $path
    ))
  };
}

/// The repros' stand-in class, written from scratch (no code of eptcs.cls).
const CLASS: &str = repro!("eptcs.cls");

/// The text of `fragment`, markup aside, its whitespace collapsed.
pub(super) fn text(fragment: &str) -> String {
  let mut out = String::new();
  let mut in_tag = false;
  for c in fragment.chars() {
    match c {
      '<' => in_tag = true,
      '>' => {
        in_tag = false;
        out.push(' ');
      },
      _ if !in_tag => out.push(c),
      _ => {},
    }
  }
  out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Each creator of `xml`, in order: its personname's text, then each contact as `role=text`.
pub(super) fn creators(xml: &str) -> Vec<Vec<String>> {
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let end = start + xml[start..].find("</creator>").expect("closed creator");
      let creator = &xml[start..end];
      let mut out = Vec::new();
      if let Some(at) = creator.find("<personname>") {
        let close = creator[at..]
          .find("</personname>")
          .expect("closed personname")
          + at;
        out.push(text(&creator[at + "<personname>".len()..close]));
      }
      for (at, _) in creator.match_indices("<contact ") {
        let open_end = at + creator[at..].find('>').expect("contact tag");
        let role = creator[at..open_end]
          .split("role=\"")
          .nth(1)
          .and_then(|r| r.split('"').next())
          .unwrap_or("");
        let close = open_end
          + creator[open_end..]
            .find("</contact>")
            .expect("closed contact");
        out.push(format!("{role}={}", text(&creator[open_end + 1..close])));
      }
      out
    })
    .collect()
}

/// Converts the repro beside the stand-in class, checks it converts clean, and compares its creators.
#[track_caller]
fn assert_creators(tex: &str, expected: &[&[&str]]) {
  let (log, xml) = convert_files_with(tex, &[("eptcs.cls", CLASS)], Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let found = creators(&xml);
  let want: Vec<Vec<String>> = expected
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  assert_eq!(found, want, "{xml}");
}

/// `Ann Able \institute{Univ A\\ Country A} \email{ann@a.edu} \and Bob Baker \institute{Univ B} \institute{Dept B}
/// \email{bob@b.edu}`: each name with its institutes and e-mail. Red (int111d): the institutes inside the names.
#[test]
fn eptcs_institute_email_cells() {
  assert_creators(repro!("eptcs_institute_email_cells.tex"), &[
    &[
      "Ann Able",
      "affiliation=Univ A Country A",
      "email=ann@a.edu",
    ],
    &[
      "Bob Baker",
      "affiliation=Univ B",
      "affiliation=Dept B",
      "email=bob@b.edu",
    ],
  ]);
}

/// A cell holding a row of names (`Carl Cole and Dan Dorn`, 2109.08295): each institute every name's, each address of
/// an e-mail list the name it spells; `Eve Eng \qquad Fay Fox` with one address each, in their order (1101.4424).
#[test]
fn eptcs_row_of_names() {
  assert_creators(repro!("eptcs_row_of_names.tex"), &[
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
}

/// A `tabular` of names with a row of e-mails under them and the institute lines under the table (1407.7928): each
/// column's name its e-mail, every line under the table each name's.
#[test]
fn eptcs_author_tabular() {
  assert_creators(repro!("eptcs_author_tabular.tex"), &[
    &[
      "Ann Able",
      "email=ann@a.edu",
      "affiliation=Dept of Tools",
      "affiliation=Univ of Here",
    ],
    &[
      "Bob Baker",
      "email=bob@b.edu",
      "affiliation=Dept of Tools",
      "affiliation=Univ of Here",
    ],
  ]);
}

/// A paper's own environment that opens a `tabular` of names (2409.15014's `{authors}{5}`), an `\institute` and an
/// `\email` line across its columns: each name the institute and the address it spells.
#[test]
fn eptcs_author_environment_columns() {
  assert_creators(repro!("eptcs_author_environment_columns.tex"), &[
    &["Ann Able", "affiliation=Univ X", "email=ann.able@x.org"],
    &["Bob Baker", "affiliation=Univ X", "email=bob.baker@x.org"],
    &["Carl Cole", "affiliation=Univ X", "email=carl.cole@x.org"],
  ]);
}

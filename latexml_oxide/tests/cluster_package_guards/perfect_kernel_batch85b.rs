//! Red/green guards for batch 85b: llncs's one `\institute` is every author's (OXIDIZED_DESIGN_DIVERGENCES #901,
//! `share_row_contacts`; llncs binding `latexml_package/src/package/llncs_cls.rs`). llncs.cls:868-886 prints a sole
//! institute (no `\and` in it) unnumbered under the whole author row; with no `\inst` mark picking it, each author is
//! its author, and its `\email` lines are lines under the row, each address the author it spells (FIDELITY_CHARTER §4).
//! It had been a nameless creator under the names, no author affiliated (20 of the frontmatter scorer's 56 llncs papers:
//! 1002.2793, 1307.8067, 1405.1167, 1509.00818, 1609.01220, 1809.11127, 1910.11599, 2011.00616, 2101.06781, 2106.04289,
//! 2205.06740, 2207.04574, 2207.11464, 2306.03360, 2405.11276, 2502.08785, 2511.12796, 2603.26486, 2604.20229; and
//! 2207.06953, several institutes, left the block's; 2608.21425 and 2407.13764 the negatives). Round 2 (the r1 review)
//! adds one guard per distinct path: each `\email` call decided on its own (2410.04091, 2206.10050, 2609.24668); an
//! e-mail in the institute's note never shared; only `\inst{n}` picks the institute (2607.02596, 1804.05870); an
//! inner `\institute` wins; an ambiguous spelling places nothing; a list of local parts keeps its braces
//! (1701.08125); the corresponding author's address, a template's no one's. Round 3 (the r2 re-review): a line holding
//! an address outside the `\email` calls is an e-mail line read by the address reader (1103.2027, 1307.8067's
//! `\urldef`), and a call's own label goes with it (2410.06985); an address in the institute's note is never shared;
//! the review's duplicate pairs merged. Near-duplicate probe shapes are repros without a guard. Each guard converts its repro under
//! `tools/perfect_kernel/repros/class-bindings/` and asserts no Error or Warning and every creator's name and contacts
//! (role and text) in order.
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

/// `\author{Ann Able \and Bob Baker \and Cat Cole}\institute{University of Somewhere, Town\\
/// \email{\{able,baker\}@x.edu}, \email{cole@y.edu}}` (1002.2793, 2106.04289, 2207.11464): each author the institute and
/// the address it spells, the brace list each address apart, under llncs's own e-mail name. Red (int114a): the
/// institute and the brace list on a nameless creator, no author affiliated.
#[test]
fn llncs_sole_institute_is_every_authors() {
  let xml = assert_creators(repro!("llncs_sole_institute_is_every_authors.tex"), &[
    &[
      "Ann Able",
      "affiliation=University of Somewhere, Town",
      "email=able@x.edu",
    ],
    &[
      "Bob Baker",
      "affiliation=University of Somewhere, Town",
      "email=baker@x.edu",
    ],
    &[
      "Cat Cole",
      "affiliation=University of Somewhere, Town",
      "email=cole@y.edu",
    ],
  ]);
  // (the institute once on each author, whole, under llncs's `\institutename`, its e-mails no part of it; each address
  // under llncs's `\emailname`)
  assert_eq!(xml.matches("role=\"affiliation\"").count(), 3, "{xml}");
  assert_eq!(
    xml
      .matches(
        "<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of Somewhere, Town</contact>"
      )
      .count(),
    3,
    "{xml}"
  );
  assert_eq!(
    xml
      .matches("<contact name=\"E-mail\u{a0}\" role=\"email\">")
      .count(),
    3,
    "{xml}"
  );
}

/// The institute written before the authors, `\inst{*}` and an empty `\inst{}` on the names (2207.04574, 2604.20229):
/// every author's, the star shown; `info@x.edu`, which no name spells, the author block's. Red (int114a): the institute
/// on a nameless creator.
#[test]
fn llncs_sole_institute_before_the_authors() {
  assert_creators(repro!("llncs_sole_institute_before_the_authors.tex"), &[
    &["Ann Able *", "affiliation=University of Somewhere, Town"],
    &["Bob Baker", "affiliation=University of Somewhere, Town"],
    &["email=info@x.edu"],
  ]);
}

/// `\author{Ann Able and Bob Baker \institute{University of Somewhere\\ Town}}` (1609.01220): the institute after the
/// names, both authors'. Red (int114a): on a nameless creator.
#[test]
fn llncs_institute_inside_author() {
  assert_creators(repro!("llncs_institute_inside_author.tex"), &[
    &["Ann Able", "affiliation=University of Somewhere Town"],
    &["Bob Baker", "affiliation=University of Somewhere Town"],
  ]);
}

/// Negative: `\inst{1}` on Ann Able alone picks the sole institute for her; the others stay unaffiliated, as the marks
/// say. Pinned so in round 2 under the rule that only the class's own `\inst{n}` picks: Bob Baker and Cat Cole, with
/// no `\inst`, would keep the institute only if no `\inst{n}` picked it for anyone — an `\inst{1}` is the class's
/// numbering, which the paper uses, so an author without one requests nothing. Its `\email` is a line under the row
/// (`able@` Ann Able's by spelling, as on the shared path). Green on int114a. A picked institute every line of which is
/// taken stays, answering its authors' marks (`\institute{\email{able@x.edu}, \email{baker@x.edu}}` under two
/// `\inst{1}`): each address its owner's, no mark shown. Red: 85b r2 and r3 panicked on the second call's separators
/// (the first unit had taken them), and r3 dropped the picked contact, showing the marks.
#[test]
fn llncs_inst_marks_pick_the_sole_institute() {
  let tex = repro!("llncs_inst_marks_pick_the_sole_institute.tex");
  assert_creators(tex, &[
    &[
      "Ann Able",
      "affiliation=University of Somewhere",
      "email=able@x.edu",
    ],
    &["Bob Baker"],
    &["Cat Cole"],
  ]);
  let calls = tex
    .replace(r"\and Bob Baker \and", r"\and Bob Baker\inst{1} \and")
    .replace(
      r"\institute{University of Somewhere \email{able@x.edu}}",
      r"\institute{\email{able@x.edu}, \email{baker@x.edu}}",
    );
  assert_ne!(calls, tex);
  assert_creators(&calls, &[
    &["Ann Able", "email=able@x.edu"],
    &["Bob Baker", "email=baker@x.edu"],
    &["Cat Cole"],
  ]);
}

/// Negatives, the class's dispatch: two institutes and three unmarked authors — the class numbers the institutes, pairs
/// them with no one (2207.06953) — stay on the block's nameless creator; an empty `\institute{}` is no institute, so the
/// `\inst` marks it would answer stay shown (2407.13764; an 85b draft made an empty shared contact that answered the
/// marks and hid them). Green on int114a.
#[test]
fn llncs_several_or_empty_institutes_are_not_shared() {
  assert_creators(
    repro!("llncs_several_institutes_without_marks_stay_the_blocks.tex"),
    &[&["Ann Able"], &["Bob Baker"], &["Cat Cole"], &[
      "affiliation=University A",
      "affiliation=University B",
    ]],
  );
  let xml = assert_creators(repro!("llncs_empty_institute_keeps_the_inst_marks.tex"), &[
    &["Ann Able 1"],
    &["Bob Baker 2"],
  ]);
  assert!(
    xml.contains("<personname>Ann Able<sup xml:id=\"id1\">1</sup></personname>"),
    "{xml}"
  );
}

/// Each `\email` call is decided on its own (`row_email_calls`): `\institute{Univ X\\ \email{able@x.edu},
/// \email{\{zz@x.edu\}}}` (the r1 review's q03) gives able@ to Ann Able by spelling and the braced single address to Bob
/// Baker by place. 2410.04091's three `\email{\{…\}}` are its three authors' in order. A call the reader cannot part
/// (`{zz,yy}@x.edu`, 2206.10050) and one it parts into pieces no whole address (`[zz@insa-, yy@insa-]rouen.fr`,
/// 2609.24668) are the block's as written; an `Email:` label goes with its call (2102.12877). Red (85b r1): able@ on
/// both authors, `{fatima.iribu@gmail.com}` on all three.
#[test]
fn llncs_each_email_call_is_decided_alone() {
  let tex = repro!("llncs_each_email_call_is_decided_alone.tex");
  assert_creators(tex, &[
    &["Ann Able", "affiliation=Univ X", "email=able@x.edu"],
    &["Bob Baker", "affiliation=Univ X", "email={zz@x.edu}"],
  ]);
  for (call, block) in [
    (r"{zz,yy}@x.edu", "email=zz,yy@x.edu"),
    (
      r"[zz@insa-, yy@insa-]rouen.fr",
      "email=[zz@insa-, yy@insa-]rouen.fr",
    ),
  ] {
    let variant = tex.replace(r"\email{\{zz@x.edu\}}}", &format!(r"\email{{{call}}}}}"));
    assert_ne!(variant, tex);
    assert_creators(&variant, &[
      &["Ann Able", "affiliation=Univ X", "email=able@x.edu"],
      &["Bob Baker", "affiliation=Univ X"],
      &[block],
    ]);
  }
  // (an institute of calls alone: each its owner's, no institute left — 85b r2 and r3 panicked on the second call's
  // separators, which the first unit, at the institute's start, had taken)
  let calls = tex.replace(r"\institute{Univ X\\ ", r"\institute{");
  assert_ne!(calls, tex);
  assert_creators(&calls, &[&["Ann Able", "email=able@x.edu"], &[
    "Bob Baker",
    "email={zz@x.edu}",
  ]]);
  let label = tex.replace(
    r"Univ X\\ \email{able@x.edu}, ",
    r"Univ X\\ Email: \email{able@x.edu}, ",
  );
  assert_ne!(label, tex);
  assert_creators(&label, &[
    &["Ann Able", "affiliation=Univ X", "email=able@x.edu"],
    &["Bob Baker", "affiliation=Univ X", "email={zz@x.edu}"],
  ]);
  let institute = "affiliation=Iran Broadcasting University (IRIBU), Tehran, Iran";
  assert_creators(
    repro!("llncs_single_address_braces_are_each_owners.tex"),
    &[
      &[
        "Fatemeh Allahdadi",
        institute,
        "email={fatima.iribu@gmail.com}",
      ],
      &[
        "Rahil Mahdian Toroghi",
        institute,
        "email={mahdiantr1974@ieee.org}",
      ],
      &["Hassan Zareian", institute, "email={zareian@iribu.ac.ir}"],
    ],
  );
}

/// An address in a note of the shared institute is never every author's (the r1 review's p19, the re-review's finding
/// 3): a raw `@` in `\thanks{Mail me at able@x.edu, \email{baker@x.edu}}` makes the institute's own line an address
/// line no reader parts, so the institute and its note are the block's, as written (int114a's placement); an `\email`
/// alone in the note (`\thanks{Contact \email{baker@x.edu}}`, s06's shape) keeps the institute shared and is Bob
/// Baker's by spelling. The note's own text is not asserted: an `\email` digested in a note becomes a contact and
/// leaves the note's sentence (a known residual, int114a too). Red: 85b r1 gave `baker@x.edu` to both authors; 85b r2
/// kept "Mail me at able@x.edu," in the note on every author's affiliation.
#[test]
fn llncs_address_in_institute_note_is_never_shared() {
  let tex = repro!("llncs_address_in_institute_note_is_never_shared.tex");
  // (since 92c r3 a `\thanks` inside the institute is no line of it — `shared_row/emails.rs` `note_argument` — so the
  // sole institute is every author's, as llncs prints it under the names, and `baker@x.edu` is Bob Baker's by
  // spelling; the note's sentence stays the note's)
  let xml = assert_creators(tex, &[&["Ann Able", "affiliation=Univ X"], &[
    "Bob Baker",
    "affiliation=Univ X",
    "email=baker@x.edu",
  ]]);
  assert!(xml.contains("Mail me at able@x.edu"), "{xml}");
  let call = tex.replace(
    r"\thanks{Mail me at able@x.edu, \email{baker@x.edu}}",
    r"\thanks{Contact \email{baker@x.edu}}",
  );
  assert_ne!(call, tex);
  assert_creators(&call, &[&["Ann Able", "affiliation=Univ X"], &[
    "Bob Baker",
    "affiliation=Univ X",
    "email=baker@x.edu",
  ]]);
}

/// Marks no one answers pick nothing (`marks_pick`; only the class's own `\inst{n}` does): 2607.02596's hand-typed
/// `$^{1}$` on every name and 1804.05870's `\printfnsymbol{1}` leave the one institute every author's, each brace-list
/// address its owner's; llncs's `\fnmsep\thanks{…}` in the institute is no mark of its own (the r1 review's r05). Red
/// (85b r1): the institute on a nameless creator in all three. And the negative on the same path (`raises_a_bare_mark`):
/// an institute holding a paper macro's marks is not shared (2608.21425).
#[test]
fn llncs_marks_that_pick_nothing() {
  let institute =
    "affiliation=College of Biomedical Engineering, Fudan University, Shanghai, China";
  assert_creators(repro!("llncs_hand_marks_pick_nothing.tex"), &[
    &["Xinyu Jia 1", institute],
    &["Weidong Guo 1", institute],
    &["Wangyuan Zhao 1", institute],
    &["Yi Guo 1", institute],
    &["Zeju Li 1,†", institute, "email=zejuli@fudan.edu.cn"],
    &["Yuanyuan Wang 1,†", institute, "email=yywang@fudan.edu.cn"],
  ]);
  assert_creators(repro!("llncs_paper_mark_macro_picks_nothing.tex"), &[
    &[
      "Rohit Pandey",
      "affiliation=Google Inc.",
      "email=rohitpandey@google.com",
    ],
    &[
      "Pavel Pidlypenskyi *",
      "affiliation=Google Inc.",
      "email=podlipensky@google.com",
    ],
    &[
      "Shuoran Yang",
      "affiliation=Google Inc.",
      "email=shuorany@google.com",
    ],
    &[
      "Christine Kaeser-Chen",
      "affiliation=Google Inc.",
      "email=christinech@google.com",
    ],
  ]);
  let xml = assert_creators(repro!("llncs_fnmsep_note_in_institute_is_shared.tex"), &[
    &["Ann Able", "affiliation=Univ X"],
    &["Bob Baker", "affiliation=Univ X"],
  ]);
  assert!(
    xml.contains("role=\"thanks\">Supported by Y.</pubnote>"),
    "{xml}"
  );
  // (negative: an institute holding the marks of a command of the paper's own, `\affmark{1} University A \quad
  // \affmark{2} University B`, `\affmark` a `\textsuperscript`, 2608.21425, is lines the names' marks pick, not one shared
  // institute: the block's, the marks shown; green on int114a, an 85b draft gave the whole line to every author)
  assert_creators(
    repro!("llncs_institute_marked_by_a_paper_macro_is_not_shared.tex"),
    &[&["Ann Able 1"], &["Bob Baker 2"], &[
      "affiliation=1 University A 2 University B",
    ]],
  );
}

/// An `\institute` inside `\author` wins over an outer one (the r1 review's r02; pdflatex "Univ X"); a later `\author`
/// without one gives the store back to the outer call (`\lx@llncs@inner@institute`). Red (85b r1): Univ Y on both.
#[test]
fn llncs_inner_institute_wins() {
  let tex = repro!("llncs_inner_institute_wins.tex");
  assert_creators(tex, &[&["Ann Able", "affiliation=Univ X"], &[
    "Bob Baker",
    "affiliation=Univ X",
  ]]);
  let later = tex.replace(
    "\n\\institute{Univ Y}\n",
    "\n\\author{Ann Able \\and Bob Baker}\n\\institute{Univ Y}\n",
  );
  assert_ne!(later, tex);
  assert_creators(&later, &[&["Ann Able", "affiliation=Univ Y"], &[
    "Bob Baker",
    "affiliation=Univ Y",
  ]]);
}

/// `lee@` spelling Ann Lee and Bob Lee at Cid Cole's place (the r1 review's r01) puts the list in no author order: the
/// three addresses the block's. Red (85b r1): lee@ on Cid Cole by place.
#[test]
fn llncs_ambiguous_spelling_is_no_place() {
  assert_creators(repro!("llncs_ambiguous_spelling_is_no_place.tex"), &[
    &["Ann Lee", "affiliation=Univ X"],
    &["Bob Lee", "affiliation=Univ X"],
    &["Cid Cole", "affiliation=Univ X"],
    &["email=x1@x.edu", "email=x2@x.edu", "email=lee@x.edu"],
  ]);
}

/// 1701.08125: the e-mail call split by owner sets `cms@` on Christian Müller-Schloer and keeps the printed braces of
/// the list of local parts no one owns, `{stomforde|bsick}@uni-kassel.de`, the block's. Red (85b r1): its braces lost.
#[test]
fn llncs_listed_local_parts_keep_their_braces() {
  let institute = "affiliation=Intelligent Embedded Systems Lab, University of Kassel, Germany Institute for Systems \
                   Engineering, Leibniz University of Hannover";
  assert_creators(repro!("llncs_listed_local_parts_keep_their_braces.tex"), &[
    &["Sven Tomforde", institute],
    &["Bernhard Sick", institute],
    &[
      "Christian Müller-Schloer",
      institute,
      "email=cms@sra.uni-hanover.de",
    ],
    &["email={stomforde|bsick}@uni-kassel.de"],
  ]);
}

/// The one address under the shared institute no name spells and no order places is the corresponding author's
/// (FIDELITY_CHARTER §4; the r1 review's p22). Red (85b r1): on the author block. Not when the corresponding author
/// owns another address of the line (`able@`): the one left is the block's (2510.12629's `kgwo0528@` beside Jinwoo
/// Kim's `jinwookim@`, under `\inst{1}` marks: a group written out by owner, decision 3 of 2026-10-22); nor when it is a
/// template's (`{firstname.lastname}@x.edu`, 2102.12877's shape).
#[test]
fn llncs_corresponding_author_takes_the_left_address() {
  let tex = repro!("llncs_corresponding_author_takes_the_left_address.tex");
  assert_creators(tex, &[
    &["Ann Able", "affiliation=Univ X", "email=zz@x.edu"],
    &["Bob Baker", "affiliation=Univ X"],
  ]);
  let owned = tex.replace(r"\email{zz@x.edu}", r"\email{zz@x.edu, able@x.edu}");
  assert_ne!(owned, tex);
  assert_creators(&owned, &[
    &["Ann Able", "affiliation=Univ X", "email=able@x.edu"],
    &["Bob Baker", "affiliation=Univ X"],
    &["email=zz@x.edu"],
  ]);
  // (the same under `\inst{1}` marks, 2510.12629's shape: a call parted by owner is written out there too, its
  // unspelled address the block's — decision 3 of 2026-10-22, the one exception to the picked contact keeping what has
  // no owner)
  let picked = owned.replace(
    r"\author{Ann Able\thanks{Corresponding author} \and Bob Baker}",
    r"\author{Ann Able\inst{1}\thanks{Corresponding author} \and Bob Baker\inst{1}}",
  );
  assert_ne!(picked, owned);
  assert_creators(&picked, &[
    &["Ann Able", "affiliation=Univ X", "email=able@x.edu"],
    &["Bob Baker", "affiliation=Univ X"],
    &["email=zz@x.edu"],
  ]);
  // (a template's address is no one's, `is_template_address`: the re-review's s08, s09; 85b r2 gave it to Ann Able)
  for (template, block) in [
    (
      r"\{firstname.lastname\}@x.edu",
      "email={firstname.lastname}@x.edu",
    ),
    ("first.last@x.edu", "email=first.last@x.edu"),
  ] {
    let variant = tex.replace(r"\email{zz@x.edu}", &format!(r"\email{{{template}}}"));
    assert_ne!(variant, tex);
    assert_creators(&variant, &[
      &["Ann Able", "affiliation=Univ X"],
      &["Bob Baker", "affiliation=Univ X"],
      &[block],
    ]);
  }
}

/// A line of the shared institute holding an `@` outside an `\email` call is an e-mail line of the row, read with the
/// address reader, each address its owner's (`holds_raw_address`, `raw_line_unit`; the re-review's finding 1, 19 census
/// papers): 1103.2027's `{\tt \{joxan,navas,andrews\}@…}` as `\texttt{…}` (s20) and `\url{{…}@…}` (1809.11127) alike,
/// an `Email:` label before it the contact's name (0712.2737), `\href{mailto:…}{…}` its spelled owner's (2408.03209,
/// s30); 1307.8067's `\urldef`'d `\mailsa` read as the text it prints. A line of the shared institute the reader cannot
/// read is the block's as written, an affiliation; the institute's own first line holding one keeps the whole
/// institute at the block (1706.02124; int114a's placement). Under `\inst{1}` marks the marks decide the institute's
/// authors: a line the reader cannot read, first or later, stays in it, as written, its picked authors' (int114a's
/// placement; the r3 re-review's n31-n33, the r4 re-review's q25). Red (85b r2): the address text in every author's
/// affiliation; (85b r3) the picked first line at the block; (85b r4) the picked later line at the block.
#[test]
fn llncs_address_lines_are_each_owners() {
  let tex = repro!("llncs_address_line_is_each_owners.tex");
  let institute = "affiliation=National University of Singapore";
  let each: [&[&str]; 3] = [
    &["Joxan Jaffar", institute, "email=joxan@comp.nus.edu.sg"],
    &["Jorge A. Navas", institute, "email=navas@comp.nus.edu.sg"],
    &[
      "Andrew E. Santosa",
      institute,
      "email=andrews@comp.nus.edu.sg",
    ],
  ];
  assert_creators(tex, &each);
  let line = r" {\tt \{joxan,navas,andrews\}@comp.nus.edu.sg} \\";
  for written in [
    r" \texttt{\{joxan,navas,andrews\}@comp.nus.edu.sg} \\",
    r" \url{{joxan,navas,andrews}@comp.nus.edu.sg} \\",
    r" Email: {\tt \{joxan,navas,andrews\}@comp.nus.edu.sg} \\",
  ] {
    let variant = tex.replace(line, written);
    assert_ne!(variant, tex);
    assert_creators(&variant, &each);
  }
  let link = tex.replace(
    line,
    r" \href{mailto:navas@comp.nus.edu.sg}{navas@comp.nus.edu.sg} \\",
  );
  assert_creators(&link, &[
    &["Joxan Jaffar", institute],
    &["Jorge A. Navas", institute, "email=navas@comp.nus.edu.sg"],
    &["Andrew E. Santosa", institute],
  ]);
  let unread = tex.replace(line, r" {\tt [joxan, navas]@comp.nus.edu.sg} \\");
  assert_creators(&unread, &[
    &["Joxan Jaffar", institute],
    &["Jorge A. Navas", institute],
    &["Andrew E. Santosa", institute],
    &["affiliation=[joxan, navas]@comp.nus.edu.sg"],
  ]);
  let first = tex.replace(
    &format!("National University of Singapore\\\\\n{line}"),
    r"National University of Singapore {\tt \{joxan,navas,andrews\}@comp.nus.edu.sg}",
  );
  assert_ne!(first, tex);
  assert_creators(&first, &[
    &["Joxan Jaffar"],
    &["Jorge A. Navas"],
    &["Andrew E. Santosa"],
    &["affiliation=National University of Singapore {joxan,navas,andrews}@comp.nus.edu.sg"],
  ]);
  // (the same first line under `\inst{1}` on every name: the picked institute stays its authors', as written, the marks
  // answered — int114a's placement; the r3 re-review's n31-n33, where 85b r3 sent it to the block and showed the marks)
  let picked = first.replace(
    r"\author{Joxan Jaffar \and Jorge A. Navas \and Andrew E. Santosa}",
    r"\author{Joxan Jaffar\inst{1} \and Jorge A. Navas\inst{1} \and Andrew E. Santosa\inst{1}}",
  );
  assert_ne!(picked, first);
  let line = "affiliation=National University of Singapore {joxan,navas,andrews}@comp.nus.edu.sg";
  assert_creators(&picked, &[
    &["Joxan Jaffar", line],
    &["Jorge A. Navas", line],
    &["Andrew E. Santosa", line],
  ]);
  // (a later line the reader does not read, under the same marks: it stays too, its picked authors' — the marks decide
  // the contact's authors, the block is the shared path's fail-safe only; the r4 re-review's q25, q01, q02, q19, where
  // 85b r4 sent the line to a nameless creator)
  let picked_later = unread.replace(
    r"\author{Joxan Jaffar \and Jorge A. Navas \and Andrew E. Santosa}",
    r"\author{Joxan Jaffar\inst{1} \and Jorge A. Navas\inst{1} \and Andrew E. Santosa}",
  );
  assert_ne!(picked_later, unread);
  let both = "affiliation=National University of Singapore [joxan, navas]@comp.nus.edu.sg";
  assert_creators(&picked_later, &[
    &["Joxan Jaffar", both],
    &["Jorge A. Navas", both],
    &["Andrew E. Santosa"],
  ]);
  // (a read unit no author owns: a template address, 2109.05583's shape — an entry's one-address `\email`, placed by
  // FIDELITY_CHARTER §4's chain since 92b's G3 (OD #917), the block's: two carriers, none spelled nor marked the
  // corresponding author; written so since llncs is written, 92c, OD #923 — 85b r5 had kept it on the picked authors;
  // and an address its owner takes off a line whose other text the reader does not read, the r4 re-review's q31 — the
  // rest of the line stays every picked author's; the r5 re-review's finding 2)
  let picked_template = r"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\inst{1} \and Bob Baker\inst{1}}
\institute{Univ X\\ \email{\{firstname.lastname\}@x.edu}}
\maketitle
Text.
\end{document}
";
  let template = "email={firstname.lastname}@x.edu";
  assert_creators(picked_template, &[
    &["Ann Able", "affiliation=Univ X"],
    &["Bob Baker", "affiliation=Univ X"],
    &[template],
  ]);
  let picked_owned = picked_template.replace(
    r"\email{\{firstname.lastname\}@x.edu}",
    r"\email{able@x.edu}, Twitter: @univx",
  );
  assert_ne!(picked_owned, picked_template);
  let rest = "affiliation=Univ X, Twitter: @univx";
  assert_creators(&picked_owned, &[
    &["Ann Able", rest, "email=able@x.edu"],
    &["Bob Baker", rest],
  ]);
  let institute =
    "affiliation=Old Dominion University, Department of Computer Science Norfolk VA, 23529, USA";
  assert_creators(repro!("llncs_urldef_address_line_is_each_owners.tex"), &[
    &["Mat Kelly", institute, "email=mkelly@cs.odu.edu"],
    &[
      "Justin F. Brunelle",
      institute,
      "email=jbrunelle@cs.odu.edu",
    ],
    &["Michele C. Weigle", institute, "email=mweigle@cs.odu.edu"],
    &["Michael L. Nelson", institute, "email=mln@cs.odu.edu"],
  ]);
}

/// A label of the call's own goes with the `\email` call (`call_label`), to the address's owner or with it to the
/// block, never every author's affiliation (the re-review's finding 1 siblings): 2410.06985's `Corresponding author:`
/// (2609.00838; 2509.05139's `Contact:`), the same after a comma on the institute's own line, 2608.24697's
/// `(Corresponding author),` after the call, s23's `Corresponding author email:` before an address no name spells (the
/// block's); a label read as printed (`\textit{Corresponding author:}`, `Corresponding~author:`, the r3 re-review's n15,
/// n16). Only the label forms `call_label` recognises go with the call; the rest of the line stays the institute's
/// (`… F-76000 Rouen, France. \email{…}`, 2609.24668; 1812.09276), text ending in a colon with no contact word too
/// (`Dept. of CS:`, the r3 re-review's n10; n09, n12, n35 alike), and so does a label of another form (a residual,
/// OXIDIZED_DESIGN_DIVERGENCES #901). Red (85b r2): "Corresponding author:" in every author's affiliation; (85b r3)
/// "of CS:" on the call's owner, "Corresponding author:" in a font's group on every author.
#[test]
fn llncs_email_label_goes_with_the_call() {
  let tex = repro!("llncs_email_label_goes_with_the_call.tex");
  let names = [
    "Shimon Vainer",
    "Konstantin Kutsy",
    "Dante De Nigris",
    "Ciara Rowles",
    "Slava Elizarov",
    "Simon Donné",
  ];
  let institute = "affiliation=Unity Technologies";
  // (every author with the institute, then the contacts given to that author, then the block's creator)
  let check = |tex: &str, given: &[(&str, &str)], block: Option<&str>| {
    let mut want: Vec<Vec<&str>> = names
      .iter()
      .map(|name| {
        let mut creator = vec![*name, institute];
        creator.extend(
          given
            .iter()
            .filter(|(owner, _)| owner == name)
            .map(|(_, c)| *c),
        );
        creator
      })
      .collect();
    want.extend(block.map(|c| vec![c]));
    let want: Vec<&[&str]> = want.iter().map(Vec::as_slice).collect();
    assert_creators(tex, &want);
  };
  let call = "email=Corresponding author: donnesimon@gmail.com";
  check(tex, &[("Simon Donné", call)], None);
  let comma = tex.replace(
    "Unity Technologies\\\\\nCorresponding author: ",
    "Unity Technologies, Corresponding author: ",
  );
  assert_ne!(comma, tex);
  check(&comma, &[("Simon Donné", call)], None);
  let original = r"Corresponding author: \email{donnesimon@gmail.com}";
  let after = tex.replace(
    original,
    r"\email{donnesimon@gmail.com} (Corresponding author),\\ \email{rowles@x.edu}",
  );
  check(
    &after,
    &[
      ("Ciara Rowles", "email=rowles@x.edu"),
      (
        "Simon Donné",
        "email=donnesimon@gmail.com (Corresponding author)",
      ),
    ],
    None,
  );
  let block = tex.replace(original, r"Corresponding author email: \email{zz@x.edu}");
  check(
    &block,
    &[],
    Some("email=Corresponding author email: zz@x.edu"),
  );
  for printed in [
    r"\textit{Corresponding author:} \email{donnesimon@gmail.com}",
    r"Corresponding~author: \email{donnesimon@gmail.com}",
  ] {
    let variant = tex.replace(original, printed);
    assert_ne!(variant, tex);
    check(&variant, &[("Simon Donné", call)], None);
  }
  // (text ending in a colon is no label without a contact word: it stays the institute's, whole)
  let names_no_contact = tex.replace(original, r"Dept. of CS: \email{donnesimon@gmail.com}");
  let mut want: Vec<Vec<&str>> = names
    .iter()
    .map(|name| vec![*name, "affiliation=Unity Technologies Dept. of CS:"])
    .collect();
  want[5].push("email=donnesimon@gmail.com");
  let want: Vec<&[&str]> = want.iter().map(Vec::as_slice).collect();
  assert_creators(&names_no_contact, &want);
}

//! Guards for batch TEAMAUTH (DG's rulings of 2026-11-27; OXIDIZED_DESIGN_DIVERGENCES #459): in an `\author` enumeration
//! a team or collaboration is an author wherever it stands, a "group" only after "and" or with its own mark, each group a
//! shared head noun lists by its own text; a remark's, a bracket's or an institution's reading stays as before
//! (`resolver::lines` `enumerated`, `resolver::blocks` `shared_column`). One table per rule, rows of synthetic shapes
//! (each pdflatex-clean) and witness-derived fixtures (`author_team_witness_*`), every creator whole ([`whole_creators`]).

use latexml::util::test::{error_count, warning_count};
use regex::Regex;

use super::{perfect_kernel_batch84b::text, perfect_kernel_batchz4::run};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

const ARTICLE: &str = r"\documentclass{article}";
const LLNCS: &str = r"\documentclass{llncs}";
/// The `before=` the author line's separator leaves on every creator after the first.
const SEP: &str = "before=\u{2003}\u{2003}";
/// amsart's: the " and " its `\author` list prints before its last name.
const AMS_SEP: &str = "before= and ";

/// A document of `class` whose title block is `author` (its `\author` and what follows it).
fn doc(class: &str, author: &str) -> String {
  format!(
    "{class}\n\\begin{{document}}\n\\title{{T}}\n{author}\n\\maketitle\nText.\n\\end{{document}}\n"
  )
}

/// Each creator of `xml`, whole: `role=`, `before=` where set, the personname, then every contact (`role=text`) and every
/// note (`note:role[mark]=text`) it holds, in order — the notes inside a contact are its text.
fn whole_creators(xml: &str) -> Vec<Vec<String>> {
  let creator = Regex::new(r"(?s)<creator\b([^>]*)>(.*?)</creator>").expect("regex");
  let attr = |tag: &str, name: &str| {
    Regex::new(&format!(r#"\b{name}="([^"]*)""#))
      .expect("regex")
      .captures(tag)
      .map(|c| c[1].to_string())
  };
  let item = Regex::new(r"<(contact|note)\b([^>]*?)(/?)>").expect("regex");
  creator
    .captures_iter(xml)
    .map(|c| {
      let (attrs, body) = (&c[1], &c[2]);
      let mut out = vec![format!("role={}", attr(attrs, "role").unwrap_or_default())];
      out.extend(attr(attrs, "before").map(|b| format!("before={b}")));
      let name = body.find("<personname>").map(|at| {
        &body[at + "<personname>".len()..at + body[at..].find("</personname>").expect("closed")]
      });
      out.push(name.map(text).unwrap_or_default());
      let mut from = 0;
      while let Some(found) = item.captures(&body[from..]) {
        let whole = found.get(0).expect("match");
        let (kind, tag) = (&found[1], &found[2]);
        let open_end = from + whole.end();
        let (content, next) = if &found[3] == "/" {
          ("", open_end)
        } else {
          let close = open_end
            + body[open_end..]
              .find(&format!("</{kind}>"))
              .expect("closed");
          (&body[open_end..close], close)
        };
        let role = attr(tag, "role").unwrap_or_default();
        out.push(if kind == "contact" {
          format!("{role}={}", text(content))
        } else {
          let mark = attr(tag, "mark")
            .map(|m| format!("[{m}]"))
            .unwrap_or_default();
          format!("note:{role}{mark}={}", text(content))
        });
        from = next;
      }
      out
    })
    .collect()
}

/// The text of the title block outside every creator and the title: none, every word a creator's.
fn outside_creators(xml: &str) -> String {
  let from = xml.find("</title>").expect("a title");
  let to = ["<para", "<section", "<abstract"]
    .iter()
    .filter_map(|tag| xml[from..].find(tag))
    .min()
    .map_or(xml.len(), |at| from + at);
  let creators = Regex::new(r"(?s)<creator\b.*?</creator>").expect("regex");
  text(&creators.replace_all(&xml[from + "</title>".len()..to], ""))
}

/// A row: its name, the document, its (Error, Warning) counts and its creators as [`whole_creators`] reads them.
type Row = (&'static str, String, (usize, usize), Vec<Vec<String>>);

fn row(name: &'static str, tex: String, counts: (usize, usize), creators: &[&[&str]]) -> Row {
  let creators = creators
    .iter()
    .map(|c| c.iter().map(|s| (*s).to_string()).collect())
    .collect();
  (name, tex, counts, creators)
}

/// Every row whose counts, creators or outside text differ, so one run shows each row's state.
fn mismatches(rows: &[Row]) -> Vec<String> {
  rows
    .iter()
    .filter_map(|(name, tex, counts, expected)| {
      let (log, xml) = run(tex);
      let got = (
        (error_count(&log), warning_count(&log)),
        whole_creators(&xml),
        outside_creators(&xml),
      );
      (got != (*counts, expected.clone(), String::new()))
        .then(|| format!("{name}: got {got:?}, expected {counts:?} {expected:?}"))
    })
    .collect()
}

#[track_caller]
fn check(rows: &[Row]) {
  let bad = mismatches(rows);
  assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// Rule 1 and 6: a team or collaboration enumerated by "and", "and the", `&` or `\and` (a bare comma only as rc6's
/// `the` collaboration after two names; an unframed one after it keeps rc6's reading, DG 2026-11-27) is its own author, first, middle or last, each marked one with its own address; a lone one is the creator. rc6: a
/// `collaboration` contact on each name (the `\and` column an affiliation); round 0 split two listed groups, the first a
/// contact and the second an author (review B1).
#[test]
fn an_enumerated_team_is_an_author() {
  check(&[
    row(
      "comma and (the R project's line)",
      doc(
        ARTICLE,
        r#"\author{Friedrich Leisch, Vincent Goulet and R Core Team}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Friedrich Leisch"],
        &["role=author", SEP, "Vincent Goulet"],
        &["role=author", SEP, "R Core Team"],
      ],
    ),
    row(
      "\\and columns",
      doc(
        ARTICLE,
        r#"\author{Friedrich Leisch \and Vincent Goulet \and R Core Team}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Friedrich Leisch"],
        &["role=author", SEP, "Vincent Goulet"],
        &["role=author", SEP, "R Core Team"],
      ],
    ),
    row(
      "and the",
      doc(ARTICLE, r#"\author{Ann Able, Bob Baker and the GAPS Team}"#),
      (0, 0),
      &[
        &["role=author", "Ann Able"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "the GAPS Team"],
      ],
    ),
    row(
      "a team before a name",
      doc(ARTICLE, r#"\author{Ann Able, R Core Team and Bob Baker}"#),
      (0, 0),
      // (rc6's reading: an unframed team after a bare comma (DG 2026-11-27, revisit in 0.8.1))
      &[&["role=author", "Ann Able", "affiliation=R Core Team"], &[
        "role=author",
        SEP,
        "Bob Baker",
      ]],
    ),
    row(
      "two teams, Oxford comma",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, R Core Team, and the GAPS Team}"#,
      ),
      (0, 0),
      // (rc6's reading: an unframed team after a bare comma (DG 2026-11-27))
      &[
        &["role=author", "Ann Able", "collaboration=the GAPS Team"],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=R Core Team",
          "collaboration=the GAPS Team",
        ],
      ],
    ),
    row(
      "two teams, no Oxford comma",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, R Core Team and the GAPS Team}"#,
      ),
      (0, 0),
      // (rc6's reading: an unframed team after a bare comma (DG 2026-11-27))
      &[
        &["role=author", "Ann Able", "collaboration=the GAPS Team"],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=R Core Team",
          "collaboration=the GAPS Team",
        ],
      ],
    ),
    row(
      "two collaborations after names (review B1)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, the ATLAS Collaboration and the CMS Collaboration}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "the ATLAS Collaboration"],
        &["role=author", SEP, "the CMS Collaboration"],
      ],
    ),
    row(
      "two collaborations, Oxford comma (review s1)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, the ATLAS Collaboration, and the CMS Collaboration}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "the ATLAS Collaboration"],
        &["role=author", SEP, "the CMS Collaboration"],
      ],
    ),
    row(
      "llncs inline \\and, two collaborations (review l2)",
      doc(
        LLNCS,
        r#"\author{Ann Able \and Bob Baker \and the ATLAS Collaboration \and the CMS Collaboration}\institute{Univ A}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "affiliation=Univ A"],
        &["role=author", SEP, "Bob Baker", "affiliation=Univ A"],
        &[
          "role=author",
          SEP,
          "the ATLAS Collaboration",
          "affiliation=Univ A",
        ],
        &[
          "role=author",
          SEP,
          "the CMS Collaboration",
          "affiliation=Univ A",
        ],
      ],
    ),
    row(
      "a list's commas, two collaborations",
      doc(
        ARTICLE,
        r#"\author{Ann Able, the ATLAS Collaboration, the CMS Collaboration}"#,
      ),
      (0, 0),
      // (rc6's reading: one name's comma lists no team (DG 2026-11-27, revisit in 0.8.1))
      &[&[
        "role=author",
        "Ann Able",
        "collaboration=the ATLAS Collaboration",
        "collaboration=the CMS Collaboration",
      ]],
    ),
    row(
      "two marked collaborations (review s5)",
      doc(
        ARTICLE,
        r#"\author{Ann Able$^1$, Bob Baker$^2$, the ATLAS Collaboration$^1$ and the CMS Collaboration$^2$\\ $^1$Univ A\\ $^2$Univ B}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "affiliation=Univ A"],
        &["role=author", SEP, "Bob Baker", "affiliation=Univ B"],
        &[
          "role=author",
          SEP,
          "the ATLAS Collaboration",
          "affiliation=Univ A",
        ],
        &[
          "role=author",
          SEP,
          "the CMS Collaboration",
          "affiliation=Univ B",
        ],
      ],
    ),
    row(
      "a team, then another team",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, R Core Team and Stan Development Team}"#,
      ),
      (0, 0),
      // (rc6's reading: an unframed team after a bare comma, nothing enumerated after the institution it is read as (DG 2026-11-27))
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=Stan Development Team",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=R Core Team",
          "collaboration=Stan Development Team",
        ],
      ],
    ),
    row(
      "two team \\and columns",
      doc(
        ARTICLE,
        r#"\author{Ann Able \and Bob Baker \and R Core Team \and the GAPS Team}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "R Core Team"],
        &["role=author", SEP, "the GAPS Team"],
      ],
    ),
    row(
      "an \\and column of a team",
      doc(
        ARTICLE,
        r#"\author{Ann Able \and Bob Baker \and Google Brain Team}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "Google Brain Team"],
      ],
    ),
    row(
      "the unknown-class route",
      doc(
        r#"\documentclass{nosuchclass}"#,
        r#"\author{Ann Able, Bob Baker and R Core Team}"#,
      ),
      (0, 1),
      &[
        &["role=author", "Ann Able"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "R Core Team"],
      ],
    ),
    // (one name's comma keeps the piece that name's where the list does not go on — rc6's reading; round 1 made the team
    // an author and the e-mail a nameless creator's, Gemini TEAMAUTH.9 b1; `Palo Alto` a creator is rc6's,
    // `author_team_comma_place_is_a_creator_red`)
    row(
      "one name's comma, a collaboration, an e-mail (Gemini TEAMAUTH.9 b1)",
      doc(
        ARTICLE,
        r#"\author{Chris Howard, SNO Collaboration, \texttt{c@sno.ca}}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Chris Howard",
        "email=c@sno.ca",
        "collaboration=SNO Collaboration",
      ]],
    ),
    row(
      "one name's comma, a team, an e-mail",
      doc(
        ARTICLE,
        r#"\author{Bob Baker, Google Brain Team, \texttt{bob@google.com}}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Bob Baker",
        "email=bob@google.com",
        "affiliation=Google Brain Team",
      ]],
    ),
    row(
      "one name's comma, a team, a place",
      doc(
        ARTICLE,
        r#"\author{Bob Baker, Google Brain Team, Palo Alto}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Bob Baker", "affiliation=Google Brain Team"],
        &["role=author", SEP, "Palo Alto"],
      ],
    ),
    row(
      "one name's comma, a team, a trailing comma",
      doc(ARTICLE, r#"\author{Bob Baker, Google Brain Team,}"#),
      (0, 0),
      &[&["role=author", "Bob Baker", "affiliation=Google Brain Team"]],
    ),
    row(
      "one name's comma, a team, a \\thanks after a comma",
      doc(
        ARTICLE,
        r#"\author{Bob Baker, Google Brain Team, \thanks{x}}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Bob Baker",
        "affiliation=Google Brain Team",
        "note:thanks[*]=x",
      ]],
    ),
    row(
      "a lone collaboration (rule 6)",
      doc(ARTICLE, r#"\author{The IceCube Collaboration}"#),
      (0, 0),
      &[&["role=author", "The IceCube Collaboration"]],
    ),
    row(
      "1708.04166",
      repro!("author_team_witness_1708_04166_pos_and_the_team").to_string(),
      (0, 0),
      &[
        &[
          "role=speaker",
          "Ann Able",
          "note:thanks=Funding note.",
          "affiliation=Univ A",
          "email=ann@a.org",
        ],
        &["role=author", "Bob Baker", "affiliation=Univ A"],
        &["role=author", SEP, "Cy Cole", "affiliation=Univ B"],
        &["role=author", SEP, "the GAPS Team"],
      ],
    ),
    row(
      "1609.09534",
      repro!("author_team_witness_1609_09534_iopart_comma_and_the_team").to_string(),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "email=ann@a.org",
          "affiliation=1 Univ A",
          "affiliation=2 Univ B",
        ],
        &["role=author", SEP, "Bob Baker", "affiliation=2 Univ B"],
        &["role=author", SEP, "Cy Cole", "affiliation=1 Univ A"],
        &["role=author", SEP, "the MAST Team"],
      ],
    ),
    row(
      "2111.00753",
      repro!("author_team_witness_2111_00753_iopart_and_marked_team").to_string(),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "affiliation=1 Univ A",
          "email=ann@a.org",
        ],
        &["role=author", SEP, "Bob Baker", "affiliation=2 Univ B"],
        &[
          "role=author",
          SEP,
          "Cy Cole",
          "note:footnotemark[∗]=",
          "affiliation=3 Institute of Plasma Physics, Hefei, China",
        ],
        &[
          "role=author",
          SEP,
          "Dee Dorn",
          "affiliation=3 Institute of Plasma Physics, Hefei, China",
        ],
        &[
          "role=author",
          SEP,
          "EAST Team",
          "affiliation=3 Institute of Plasma Physics, Hefei, China",
        ],
      ],
    ),
    row(
      "2509.25166",
      repro!("author_team_witness_2509_25166_mnras_newauthor_and_the_collaboration").to_string(),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "note:thanks[⋆]=E-mail: ann@a.org",
          "affiliation=Univ A",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=Univ A",
          "affiliation=Univ B",
        ],
        &["role=author", SEP, "Cy Cole", "affiliation=Univ B"],
        &["role=author", SEP, "Dee Dorn", "affiliation=Univ A"],
        &[
          "role=author",
          SEP,
          "the LSST Dark Energy Science Collaboration",
        ],
      ],
    ),
    row(
      "astro-ph0311261",
      repro!("author_team_witness_astro_ph0311261_and_the_collaboration_across_a_break")
        .to_string(),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "note:footnotemark[1]=",
          "affiliation=(1) Univ A, (2) Univ B, (3) Univ C",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "note:footnotemark[1]=",
          "affiliation=(1) Univ A, (2) Univ B, (3) Univ C",
        ],
        &[
          "role=author",
          SEP,
          "the ACBAR collaboration",
          "note:footnotemark[2,3]=",
          "affiliation=(1) Univ A, (2) Univ B, (3) Univ C",
        ],
      ],
    ),
    row(
      "astro-ph0509500",
      repro!("author_team_witness_astro_ph0509500_and_the_team_two_marks").to_string(),
      (0, 0),
      &[
        &["role=author", "Ann Able", "affiliation=Univ A"],
        &["role=author", SEP, "Bob Baker", "affiliation=Univ B"],
        &["role=author", SEP, "Cy Cole", "affiliation=Univ A"],
        &[
          "role=author",
          SEP,
          "the VVDS team",
          "affiliation=Univ A",
          "affiliation=Univ B",
        ],
      ],
    ),
    row(
      "2502.17397 (review B1)",
      repro!("author_team_witness_2502_17397_iopart_two_marked_teams").to_string(),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "note:footnotemark[†]=",
          "affiliation=1 Univ A",
          "affiliation=2 Univ B",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "note:footnotemark[†]=",
          "affiliation=1 Univ A",
        ],
        &["role=author", SEP, "Cy Cole", "affiliation=2 Univ B"],
        &[
          "role=author",
          SEP,
          "the TCV team",
          "affiliation=3 See author list of B. P. Duval et al. 2024",
        ],
        &[
          "role=author",
          SEP,
          "the WPTE team",
          "affiliation=4 See author list of E. Joffrin et al. 2024",
        ],
      ],
    ),
  ]);
}

/// Rule 2: a "group" is an author only after "and"/`&` or with its own mark, with no address under it; after a list's
/// comma, joined to an institution, in an `\and` column or before a place it is an affiliation, as rc6 reads it, and a
/// line that read an institution enumerates nothing after it (round 0 failed each, Gemini a3/a4, reviews S1, G1).
#[test]
fn a_group_is_an_author_after_and_or_its_mark() {
  check(&[
    row(
      "and, lowercase group",
      doc(ARTICLE, r#"\author{Ann Able and LHD experimental group}"#),
      (0, 0),
      &[&["role=author", "Ann Able"], &[
        "role=author",
        SEP,
        "LHD experimental group",
      ]],
    ),
    row(
      "a list's comma: an affiliation",
      doc(
        ARTICLE,
        r#"\author{Alice Smith, Bob Jones, Quantum Optics Group}"#,
      ),
      (0, 0),
      &[&["role=author", "Alice Smith"], &[
        "role=author",
        SEP,
        "Bob Jones",
        "affiliation=Quantum Optics Group",
      ]],
    ),
    row(
      "comma, the place on the next line (Gemini a3)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, Theory Group \\ CERN, Geneva}"#,
      ),
      (0, 0),
      &[&["role=author", "Ann Able", "affiliation=CERN, Geneva"], &[
        "role=author",
        SEP,
        "Bob Baker",
        "affiliation=Theory Group",
        "affiliation=CERN, Geneva",
      ]],
    ),
    row(
      "joined to an institution (Gemini a4a)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, Department of Physics and Quantum Optics Group}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=Quantum Optics Group",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=Department of Physics",
          "collaboration=Quantum Optics Group",
        ],
      ],
    ),
    row(
      "an \\and column (Gemini a4b)",
      doc(
        ARTICLE,
        r#"\author{Ann Able \and Bob Baker \and Machine Learning Group}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "affiliation=Machine Learning Group",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=Machine Learning Group",
        ],
      ],
    ),
    row(
      "llncs inline \\and before a place (review S1)",
      doc(
        LLNCS,
        r#"\author{Ann Able \and Bob Baker \and Speech Group, CERN}\institute{Univ A}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "affiliation=Speech Group",
          "affiliation=Univ A",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=CERN",
          "affiliation=Univ A",
        ],
      ],
    ),
    row(
      "and, the place on the next line",
      doc(
        ARTICLE,
        r#"\author{Ann Able and Bob Baker and Theory Group \\ CERN, Geneva}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=Theory Group",
          "affiliation=CERN, Geneva",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=Theory Group",
          "affiliation=CERN, Geneva",
        ],
      ],
    ),
    row(
      "its own mark after a comma",
      doc(
        ARTICLE,
        r#"\author{Ann Able$^1$, Bob Baker$^1$, LHD experimental group$^2$\\ $^1$Univ A\\ $^2$NIFS}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "affiliation=Univ A"],
        &["role=author", SEP, "Bob Baker", "affiliation=Univ A"],
        &[
          "role=author",
          SEP,
          "LHD experimental group",
          "affiliation=NIFS",
        ],
      ],
    ),
    // (a group with its own mark is an author with its own address, its marked line under the names no place for it —
    // rc6 and round 1: a contact on each name and a nameless creator holding NIFS, review S2 v7, m14b)
    row(
      "and, its own mark, marked lines under the names (review S2)",
      doc(
        ARTICLE,
        r#"\author{Ann Able$^1$, Bob Baker$^1$ and LHD experimental group$^2$\\ $^1$Univ A\\ $^2$NIFS}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "affiliation=Univ A"],
        &["role=author", SEP, "Bob Baker", "affiliation=Univ A"],
        &[
          "role=author",
          SEP,
          "LHD experimental group",
          "affiliation=NIFS",
        ],
      ],
    ),
    row(
      "and the, its own mark (review S2 m14b)",
      doc(
        ARTICLE,
        r#"\author{Ann Able$^1$, Bob Baker$^1$ and the XYZ Group$^{2}$ \\ $^1$Univ A \\ $^2$Univ B}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "affiliation=Univ A"],
        &["role=author", SEP, "Bob Baker", "affiliation=Univ A"],
        &["role=author", SEP, "the XYZ Group", "affiliation=Univ B"],
      ],
    ),
    row(
      "a company group after a comma (advisor 3)",
      doc(ARTICLE, r#"\author{Ann Able, Bob Baker, Alibaba Group}"#),
      (0, 0),
      &[&["role=author", "Ann Able"], &[
        "role=author",
        SEP,
        "Bob Baker",
        "affiliation=Alibaba Group",
      ]],
    ),
    row(
      "names then marked affiliations (advisor G1)",
      doc(
        ARTICLE,
        r#"\author{Ann Able$^1$, Bob Baker$^2$, $^1$Univ A, $^2$Robotics Group}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "note:footnotemark[1]=",
          "affiliation=Univ A",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "note:footnotemark[2,1]=",
          "affiliation=Robotics Group",
        ],
      ],
    ),
    row(
      "names then two institutions (advisor)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, University of X, Robotics Group}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "affiliation=University of X"],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=Robotics Group",
        ],
      ],
    ),
    row(
      "and the",
      doc(
        ARTICLE,
        r#"\author{Ann Able and Bob Baker and the Theory Group}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "the Theory Group"],
      ],
    ),
    row(
      "1711.07097",
      repro!("author_team_witness_1711_07097_revtex_and_marked_group").to_string(),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "note:footnotemark[∗]=",
          "affiliation=1 Univ A",
          "affiliation=2 National Institute for Fusion Science, Toki, Japan",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=2 National Institute for Fusion Science, Toki, Japan",
        ],
        &[
          "role=author",
          SEP,
          "Cy Cole",
          "affiliation=2 National Institute for Fusion Science, Toki, Japan",
        ],
        &[
          "role=author",
          SEP,
          "LHD experimental group",
          "affiliation=2 National Institute for Fusion Science, Toki, Japan",
        ],
      ],
    ),
  ]);
}

/// Rule 3: a shared head noun lists one author per group, each its own printed text, beside names printed before them
/// (1503.07589, 2408.02330, 2412.18491's shape in an `\author` list); rc6 made the last a contact. revtex's own
/// `\collaboration` list keeps rc6's reading (`revtex_collaboration_list_is_one_contact_red`), as a group-only list does
/// (`author_group_only_list_keeps_one_creator_red`).
#[test]
fn each_group_a_shared_head_lists_is_an_author() {
  check(&[
    row(
      "a list of groups under one plural",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, Fermilab Lattice, HPQCD, and MILC Collaborations}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "Fermilab Lattice"],
        &["role=author", SEP, "HPQCD"],
        &["role=author", SEP, "MILC Collaborations"],
      ],
    ),
    row(
      "and the, under one plural",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker and the ATLAS and CMS Collaborations}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "the ATLAS"],
        &["role=author", SEP, "CMS Collaborations"],
      ],
    ),
    row(
      "2408.02330",
      repro!("author_team_witness_2408_02330_and_column_shared_head_teams").to_string(),
      (0, 0),
      &[
        &["role=author", "Ann Able", "note:footnotemark[1]="],
        &["role=author", SEP, "Bob Baker", "note:footnotemark[2]="],
        &["role=author", SEP, "the Solar Orbiter EPD"],
        &["role=author", SEP, "Parker Solar Probe ISIS teams"],
      ],
    ),
    // (svjour3's `\institute` every listed group's, rule 1; the witness repro's input as rc6's guard bank had it)
    row(
      "1503.07589",
      repro!("author_collaboration_name_with_and_stays_one").to_string(),
      (0, 0),
      &[&["role=author", "The ATLAS", "affiliation=CERN"], &[
        "role=author",
        SEP,
        "CMS Collaborations",
        "affiliation=CERN",
      ]],
    ),
  ]);
}

/// Rules 4 and 5: an institution's `\and` column, a group before a place (an institution, paired as rc6 pairs them), a
/// name's `for`/`on behalf of`/`representing` remark, a bracketed line, a dash keep their reading; a bracket that closes
/// opens no remark (Gemini b1), so the team after it is an author. A remark's two Warnings are the emitter's markup
/// fallback (`author_names_inline_remark_markup_fallback_red`).
#[test]
fn a_framed_collaboration_and_an_institution_keep_their_reading() {
  check(&[
    row(
      "an institution's \\and column",
      doc(
        ARTICLE,
        r#"\author{Ann Able \and Bob Baker \and The R Foundation}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "affiliation=The R Foundation"],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=The R Foundation",
        ],
      ],
    ),
    // (the institution arm's split, as rc6 splits `and Univ A, CERN`: every name should hold both,
    // `author_group_before_place_every_name_red`)
    row(
      "a group before a place (review S4)",
      doc(
        ARTICLE,
        r#"\author{Ann Able and Bob Baker and Theory Group, CERN}"#,
      ),
      (0, 0),
      // (the guard returns rc6's reading: the group is no author before a place)
      &[
        &["role=author", "Ann Able", "collaboration=Theory Group"],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=Theory Group",
          "affiliation=CERN",
        ],
      ],
    ),
    // (an unframed team or collaboration before a place keeps rc6's contact on each name — round 1 made it the first
    // name's affiliation, review S1 m1, m3)
    row(
      "a collaboration before a place, a list's and (review S1)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker and XYZ Collaboration, CERN}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "collaboration=XYZ Collaboration"],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=XYZ Collaboration",
          "affiliation=CERN",
        ],
      ],
    ),
    row(
      "a team before a place, and (review S1 m3)",
      doc(
        ARTICLE,
        r#"\author{Ann Able and Bob Baker and XYZ Team, Univ A}"#,
      ),
      (0, 0),
      &[&["role=author", "Ann Able", "collaboration=XYZ Team"], &[
        "role=author",
        SEP,
        "Bob Baker",
        "collaboration=XYZ Team",
        "affiliation=Univ A",
      ]],
    ),
    row(
      "the team before a place",
      doc(
        ARTICLE,
        r#"\author{Ann Able and Bob Baker and the GAPS Team, Univ A}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "collaboration=the GAPS Team"],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=the GAPS Team",
          "affiliation=Univ A",
        ],
      ],
    ),
    row(
      "a name's for remark",
      doc(
        ARTICLE,
        r#"\author{Ann Able and Bob Baker for the ATLAS Collaboration}"#,
      ),
      (0, 2),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=for the ATLAS Collaboration",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=for the ATLAS Collaboration",
        ],
      ],
    ),
    row(
      "a bracketed line",
      doc(ARTICLE, r#"\author{Ann Able \\ (XYZ Collaboration)}"#),
      (0, 0),
      &[&[
        "role=author",
        "Ann Able",
        "collaboration=(XYZ Collaboration)",
      ]],
    ),
    row(
      "a bracket that is no remark (Gemini b1)",
      doc(
        ARTICLE,
        r#"\author{Ann Able (MIT), Bob Baker and R Core Team}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able (MIT)"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "R Core Team"],
      ],
    ),
    row(
      "(deceased) (Gemini b1)",
      doc(
        ARTICLE,
        r#"\author{Ann Able (deceased), Bob Baker and R Core Team}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able (deceased)"],
        &["role=author", SEP, "Bob Baker"],
        &["role=author", SEP, "R Core Team"],
      ],
    ),
    row(
      "a remark open on the line",
      doc(
        ARTICLE,
        r#"\author{Ann Able and Bob Baker, representing the X Team and the Y Team}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=representing the X Team",
          "collaboration=the Y Team",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=representing the X Team",
          "collaboration=the Y Team",
        ],
      ],
    ),
    row(
      "a bracket remark across commas",
      doc(
        ARTICLE,
        r#"\author{Ann Able (on behalf of the H1, ZEUS and HERA Collaborations)}"#,
      ),
      (0, 2),
      &[&[
        "role=author",
        "Ann Able",
        "collaboration=(on behalf of the H1, ZEUS and HERA Collaborations)",
      ]],
    ),
    row(
      "a team, then a name's remark (review s8)",
      doc(
        ARTICLE,
        r#"\author{Ann Able and the XYZ Team and Bob Baker for the ATLAS Collaboration}"#,
      ),
      (0, 2),
      // (the guard returns rc6's reading: the promotion would give Ann Bob's remark)
      &[
        &["role=author", "Ann Able", "collaboration=the XYZ Team"],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=for the ATLAS Collaboration",
        ],
      ],
    ),
    row(
      "hep-ex0106079",
      repro!("author_team_witness_hep_ex0106079_dash_collaboration_is_a_contact").to_string(),
      (0, 0),
      &[
        &["role=author", "A. ABLE", "collaboration=NA50 collaboration"],
        &["role=author", SEP, "B. Baker", "affiliation=Univ A."],
        &["role=author", SEP, "C. Cole", "affiliation=Univ B."],
        &["role=author", SEP, "D. Dorn", "affiliation=Univ A."],
      ],
    ),
    row(
      "1511.01102",
      repro!("author_team_witness_1511_01102_aastex_bracketed_line_is_a_contact").to_string(),
      (0, 0),
      &[
        &[
          "role=author",
          "A. Able",
          "collaboration=(The Nearby Supernova Factory)",
          "altaffiliation=Univ A",
          "altaffiliation=Univ B",
        ],
        &[
          "role=author",
          SEP,
          "B. Baker",
          "collaboration=(The Nearby Supernova Factory)",
          "altaffiliation=Univ A",
        ],
        &[
          "role=author",
          SEP,
          "C. Cole",
          "collaboration=(The Nearby Supernova Factory)",
          "altaffiliation=Univ B",
        ],
      ],
    ),
    row(
      "0712.1016, 1704.03168",
      repro!("author_team_witness_0712_1016_shared_column_is_every_names").to_string(),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "affiliation=School of Physics, Univ A",
          "affiliation=Main Road, City",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "affiliation=School of Physics, Univ A",
          "affiliation=Main Road, City",
        ],
        &[
          "role=author",
          SEP,
          "Cy Cole",
          "affiliation=School of Physics, Univ A",
          "affiliation=Main Road, City",
        ],
      ],
    ),
  ]);
}

/// Round 3 (DG's bar: rc6 where the rules are not reached): the look-ahead that makes a list go on. A run of groups
/// that ends in a place is a chain of the person's affiliations, and a framed remark after a team is no list's next
/// piece (`lines.rs` `place_after`; the comma route and its `list_goes_on` deleted in round 4, the guard
/// `resolver::enumeration`). f09ffb5d88e made the first group an author holding the place
/// (review B1, ADV_TEAMAUTH4 §4) or the team an author beside its remark (Gemini TEAMAUTH.10).
#[test]
fn a_group_run_before_a_place_or_a_remark_keeps_rc6() {
  let two_groups = |class: &str, sep: &'static str| {
    row(
      "two groups and a place",
      doc(
        class,
        r#"\author{Ann Able, Bob Baker, the XYZ Collaboration and the ABC Collaboration, CERN, Geneva}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=the XYZ Collaboration",
          "collaboration=the ABC Collaboration",
        ],
        &[
          "role=author",
          sep,
          "Bob Baker",
          "collaboration=the XYZ Collaboration",
          "collaboration=the ABC Collaboration",
          "affiliation=CERN, Geneva",
        ],
      ],
    )
  };
  let mut rows = vec![
    // (a framed remark ends a run of groups after one name's commas: rc6's affiliations, no invented co-authors —
    // round 3's blocker B1 r02, a harness pick)
    row(
      "a team run before a framed remark",
      doc(
        ARTICLE,
        r#"\author{Bob Baker, Google Brain Team, DeepMind Team, on behalf of the CMS Collaboration}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Bob Baker",
        "affiliation=Google Brain Team",
        "affiliation=DeepMind Team",
        "collaboration=on behalf of the CMS Collaboration",
      ]],
    ),
    row(
      "a team, a for remark (Gemini F1)",
      doc(
        ARTICLE,
        r#"\author{Bob Baker, ALICE Trigger Team, for the ALICE Collaboration}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Bob Baker",
        "affiliation=ALICE Trigger Team",
        "collaboration=for the ALICE Collaboration",
      ]],
    ),
    row(
      "a collaboration, an on behalf of remark",
      doc(
        ARTICLE,
        r#"\author{Chris Howard, SNO Collaboration, on behalf of the SNO Collaboration}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Chris Howard",
        "collaboration=SNO Collaboration",
        "collaboration=on behalf of the SNO Collaboration",
      ]],
    ),
    row(
      "a team, a representing remark",
      doc(
        ARTICLE,
        r#"\author{Bob Baker, Google Brain Team, representing the LHCb Collaboration}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Bob Baker",
        "affiliation=Google Brain Team",
        "collaboration=representing the LHCb Collaboration",
      ]],
    ),
    row(
      "a team, a bracketed collaboration",
      doc(
        ARTICLE,
        r#"\author{Bob Baker, Google Brain Team, (XYZ Collaboration)}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Bob Baker",
        "affiliation=Google Brain Team",
        "collaboration=(XYZ Collaboration)",
      ]],
    ),
    row(
      "two teams and a place (Gemini F10)",
      doc(
        ARTICLE,
        r#"\author{Bob Baker, Google Brain Team, DeepMind Team, London}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Bob Baker",
        "affiliation=Google Brain Team",
        "affiliation=DeepMind Team, London",
      ]],
    ),
    row(
      "a team, a collaboration, a place (review B1 q1)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, XYZ Team, ABC Collaboration, CERN}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Ann Able",
        "affiliation=XYZ Team",
        "affiliation=ABC Collaboration",
        "affiliation=CERN",
      ]],
    ),
    row(
      "two names, a team, a collaboration, a place (q2)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, XYZ Team, ABC Collaboration, CERN}"#,
      ),
      (0, 0),
      &[&["role=author", "Ann Able"], &[
        "role=author",
        SEP,
        "Bob Baker",
        "affiliation=XYZ Team",
        "affiliation=ABC Collaboration",
        "affiliation=CERN",
      ]],
    ),
    row(
      "the collaboration before a place with a comma (q3)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, XYZ Team, the ABC Collaboration, CERN, Geneva}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Ann Able",
        "affiliation=XYZ Team",
        "collaboration=the ABC Collaboration",
        "affiliation=CERN, Geneva",
      ]],
    ),
    row(
      "a team, a consortium, an institution, a name (q6)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, XYZ Team, ABC Consortium, Univ A, Bob Baker}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "affiliation=XYZ Team",
          "affiliation=ABC Consortium",
          "affiliation=Univ A",
        ],
        &["role=author", SEP, "Bob Baker"],
      ],
    ),
    row(
      "a list's and, a team, a collaboration, a place (q7)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker and XYZ Team, ABC Collaboration, CERN}"#,
      ),
      (0, 0),
      &[&["role=author", "Ann Able", "collaboration=XYZ Team"], &[
        "role=author",
        SEP,
        "Bob Baker",
        "collaboration=XYZ Team",
        "affiliation=ABC Collaboration",
        "affiliation=CERN",
      ]],
    ),
    row(
      "a team and a collaboration, a place (q8)",
      doc(
        ARTICLE,
        r#"\author{Ann Able, XYZ Team and ABC Collaboration, CERN}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Ann Able",
        "affiliation=XYZ Team",
        "collaboration=ABC Collaboration",
        "affiliation=CERN",
      ]],
    ),
    row(
      "two groups and a place, semicolons",
      doc(
        ARTICLE,
        r#"\author{Ann Able; Bob Baker; the XYZ Collaboration and the ABC Collaboration, CERN, Geneva}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=the XYZ Collaboration",
          "collaboration=the ABC Collaboration",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=the XYZ Collaboration",
          "collaboration=the ABC Collaboration",
          "affiliation=CERN, Geneva",
        ],
      ],
    ),
    row(
      "three groups and a place",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker, the XYZ Collaboration, the ABC Collaboration and the DEF Team, CERN, Geneva}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=the XYZ Collaboration",
          "collaboration=the ABC Collaboration",
          "collaboration=the DEF Team",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=the XYZ Collaboration",
          "collaboration=the ABC Collaboration",
          "collaboration=the DEF Team",
          "affiliation=CERN, Geneva",
        ],
      ],
    ),
    row(
      "one name, two groups and a place",
      doc(
        ARTICLE,
        r#"\author{Bob Baker, the XYZ Collaboration and the ABC Team, CERN}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Bob Baker",
        "collaboration=the XYZ Collaboration",
        "collaboration=the ABC Team",
        "affiliation=CERN",
      ]],
    ),
    // (rc6's: the places' marks gather on the last name, `2,1`)
    row(
      "two groups and marked places",
      doc(
        ARTICLE,
        r#"\author{Ann Able$^1$, Bob Baker$^2$, the XYZ Collaboration and the ABC Collaboration, $^1$CERN, $^2$DESY}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "note:footnotemark[1]=",
          "collaboration=the XYZ Collaboration",
          "collaboration=the ABC Collaboration",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "note:footnotemark[2,1]=",
          "collaboration=the XYZ Collaboration",
          "collaboration=the ABC Collaboration",
          "affiliation=CERN",
          "affiliation=DESY",
        ],
      ],
    ),
  ];
  rows.extend([
    two_groups(ARTICLE, SEP),
    two_groups(LLNCS, SEP),
    two_groups(r"\documentclass{amsart}", AMS_SEP),
    two_groups(r"\documentclass{revtex4-2}", SEP),
    two_groups(r"\documentclass{PoS}", SEP),
    two_groups(r"\documentclass{iopart}", SEP),
  ]);
  check(&rows);
}

/// Rule 1's owners: a group the enumeration made an author is no owner of what a class gives the call's last author —
/// the person keeps it, as rc6 gave it (`api.rs` `annotate:last`; ADV_TEAMAUTH3 G3: nucl-ex0510076 jpconf,
/// astro-ph0311630 kapproc) — and a sole author's block institute is every listed group's too, no nameless creator
/// (`post.rs sole_author_takes_the_block`; Gemini TEAMAUTH.7 a2, TEAMAUTH.9). Round 1: the address on the team, the
/// person none; svjour3's a nameless creator. jpconf's Warning is its unknown class.
#[test]
fn a_listed_group_owns_no_address_of_the_person() {
  const MITCHELL: &str =
    r#"\author{J.T. Mitchell and the PHENIX Collaboration}\address{BNL, Upton, NY}"#;
  let person = |role: &'static str, sep: &str| -> Vec<Vec<String>> {
    vec![
      vec![
        "role=author".into(),
        "J.T. Mitchell".into(),
        format!("{role}=BNL, Upton, NY"),
      ],
      vec![
        "role=author".into(),
        sep.into(),
        "the PHENIX Collaboration".into(),
      ],
    ]
  };
  let rows = [
    (
      "iopart",
      doc(r"\documentclass{iopart}", MITCHELL),
      (0, 0),
      person("affiliation", SEP),
    ),
    // (the Warning is the missing class's: jpconf is in no TeX Live)
    (
      "jpconf",
      doc(r"\documentclass{jpconf}", MITCHELL),
      (0, 1),
      person("address", SEP),
    ),
    (
      "ws-procs9x6",
      doc(r"\documentclass{ws-procs9x6}", MITCHELL),
      (0, 0),
      person("address", SEP),
    ),
    (
      "ws-ijmpa",
      doc(r"\documentclass{ws-ijmpa}", MITCHELL),
      (0, 0),
      person("address", SEP),
    ),
    (
      "amsart",
      doc(r"\documentclass{amsart}", MITCHELL),
      (0, 0),
      person("address", AMS_SEP),
    ),
    (
      "jheppub",
      doc(
        "\\documentclass{article}\\usepackage{jheppub}",
        r#"\author{J.T. Mitchell and the PHENIX Collaboration}\affiliation{BNL, Upton, NY}"#,
      ),
      (0, 0),
      person("affiliation", SEP),
    ),
  ];
  let mut rows: Vec<Row> = rows.into_iter().collect();
  rows.extend([
    row(
      "amsart \\email",
      doc(
        r"\documentclass{amsart}",
        r#"\author{Ann Able and the GAPS Team}\email{a@b.c}"#,
      ),
      (0, 0),
      &[&["role=author", "Ann Able", "email=a@b.c"], &[
        "role=author",
        AMS_SEP,
        "the GAPS Team",
      ]],
    ),
    // (a marked group the reading made a creator before is the holder still, and the team after it an author past the
    // line's closing `.` — round 1: the pile on the team, 2404.07458)
    row(
      "iopart, a marked group then a team, a closing period (2404.07458)",
      doc(
        r"\documentclass{iopart}",
        r#"\author{Ann Able$^{1}$, XYZ Working Group$^{a}$, and EAST Team$^{b}$.}
\address{$^{1}$Univ A}
\address{$a$ See X for the XYZ Working Group}
\address{$b$ See Y for the EAST Team}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "affiliation=1 Univ A"],
        &[
          "role=author",
          SEP,
          "XYZ Working Group",
          "note:footnotemark[a]=",
          "affiliation=a See X for the XYZ Working Group",
          "affiliation=b See Y for the EAST Team",
        ],
        &["role=author", SEP, "EAST Team", "note:footnotemark[b]="],
      ],
    ),
    // (a line's e-mail under a shared head's groups stays the head's, as rc6 gave it — round 1: a nameless creator's,
    // 1411.6086, 1212.5470, 1112.5642)
    row(
      "PoS, an e-mail line under a shared head (1411.6086)",
      doc(
        r"\documentclass{PoS}",
        r#"\author{Ann Able\\ Univ A\\ E-mail: \email{ann@a.edu}}
\author{Fermilab Lattice and MILC Collaborations\\ E-mail: \email{cc@ill.edu}}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "affiliation=Univ A",
          "email=ann@a.edu",
        ],
        &["role=author", SEP, "Fermilab Lattice", "email=cc@ill.edu"],
        &["role=author", SEP, "MILC Collaborations"],
      ],
    ),
    row(
      "svjour3, a person and a collaboration",
      doc(
        r"\documentclass{svjour3}",
        r#"\author{J.T. Mitchell and the PHENIX Collaboration}\institute{BNL, Upton, NY}"#,
      ),
      (0, 0),
      &[
        &["role=author", "J.T. Mitchell", "affiliation=BNL, Upton, NY"],
        &[
          "role=author",
          SEP,
          "the PHENIX Collaboration",
          "affiliation=BNL, Upton, NY",
        ],
      ],
    ),
    // (each of a sole author's institutes is the listed group's too — f09ffb5d88e gave it the first only, review S1)
    row(
      "svjour3, a team and two institutes (review S1)",
      doc(
        r"\documentclass{svjour3}",
        r#"\author{Ann Able and the GAPS Team}\institute{CERN \and Univ B}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "affiliation=CERN",
          "affiliation=Univ B",
        ],
        &[
          "role=author",
          SEP,
          "the GAPS Team",
          "affiliation=CERN",
          "affiliation=Univ B",
        ],
      ],
    ),
    row(
      "llncs, a team and two institutes (review S1)",
      doc(
        LLNCS,
        r#"\author{Ann Able and the GAPS Team}\institute{CERN \and Univ B}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "affiliation=CERN",
          "affiliation=Univ B",
        ],
        &[
          "role=author",
          SEP,
          "the GAPS Team",
          "affiliation=CERN",
          "affiliation=Univ B",
        ],
      ],
    ),
    row(
      "svjour3, a shared head, singular (Gemini a2)",
      repro!("author_shared_head_groups_svjour3_institute_is_every_groups").to_string(),
      (0, 0),
      &[&["role=author", "The ATLAS", "affiliation=CERN"], &[
        "role=author",
        SEP,
        "CMS Collaboration",
        "affiliation=CERN",
      ]],
    ),
    // (the plural with svjour3's `\institute`: `each_group_a_shared_head_lists_is_an_author`'s 1503.07589 row)
  ]);
  check(&rows);
}

/// rc6's reading whole where the enumeration's is worse in any check (`emit::Checks::worse`, OXIDIZED_DESIGN_DIVERGENCES
/// #459) — a team's mark whose labelled line under is an e-mail or shares a person's line, answered by no contact the team
/// holds (round 4b compared the word check alone: Gemini TEAMAUTH.17 B2a, C2h, B2c); names written from their text —, or
/// promotes only part of a run, or reads a revtex `\collaboration` (advisor TEAMAUTH6 A-C). Each pdflatex-clean.
#[test]
fn a_promotion_no_check_finds_worse_than_rc6() {
  let ann = |label: &str| {
    doc(
      ARTICLE,
      &format!(
        r#"\author{{Ann Able$^{{1}}$ and the GAPS Team$^{{{label}}}$ \\ $^{{1}}$Univ A \\ $^{{{label}}}$ann@x.org}}"#
      ),
    )
  };
  let rc6_ann: &[&[&str]] = &[&[
    "role=author",
    "Ann Able",
    "collaboration=the GAPS Team",
    "affiliation=Univ A",
    "email=ann@x.org",
  ]];
  check(&[
    row(
      "a team's e-mail line keeps rc6's reading",
      ann("2"),
      (0, 0),
      rc6_ann,
    ),
    row(
      "a team's symbol-labelled e-mail line keeps rc6's reading",
      ann(r"\dagger"),
      (0, 0),
      rc6_ann,
    ),
    row(
      "a team's label sharing a person's line keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Ann Able$^{1}$, Bob Baker$^{1}$ and the GAPS Team$^{2}$ \\ $^{1}$ $^{2}$Univ A}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=the GAPS Team",
          "affiliation=Univ A",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=the GAPS Team",
          "affiliation=Univ A",
        ],
      ],
    ),
    // (a run split over a line break promotes with its line or not at all: a line under the names that opens with a
    // list's conjunction keeps rc6's reading whole — pre-review of round 4c)
    row(
      "a run going on under the names keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker and the XYZ Team \\ and the ABC Team}"#,
      ),
      (0, 1),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=the XYZ Team",
          "affiliation=and the ABC Team",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=the XYZ Team",
          "affiliation=and the ABC Team",
        ],
      ],
    ),
    row(
      "a run going on under a spaced break keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker and the XYZ Team \\[2pt] and the ABC Team}"#,
      ),
      (0, 1),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=the XYZ Team",
          "affiliation=and the ABC Team",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=the XYZ Team",
          "affiliation=and the ABC Team",
        ],
      ],
    ),
    row(
      "a run going on with an ampersand keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker and the XYZ Team \\ \& the ABC Team}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=the XYZ Team",
          "affiliation=the ABC Team",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=the XYZ Team",
          "affiliation=the ABC Team",
        ],
      ],
    ),
    row(
      "a run going on after a comma keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker and the XYZ Team, \\ and the ABC Team}"#,
      ),
      (0, 1),
      &[
        &[
          "role=author",
          "Ann Able",
          "collaboration=the XYZ Team",
          "affiliation=and the ABC Team",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=the XYZ Team",
          "affiliation=and the ABC Team",
        ],
      ],
    ),
    row(
      "a run going on in an and column keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Cat Cole \and Ann Able, Bob Baker and the XYZ Team \\ and the ABC Team}"#,
      ),
      (0, 1),
      &[
        &["role=author", "Cat Cole"],
        &[
          "role=author",
          SEP,
          "Ann Able",
          "collaboration=the XYZ Team",
          "affiliation=and the ABC Team",
        ],
        &[
          "role=author",
          SEP,
          "Bob Baker",
          "collaboration=the XYZ Team",
          "affiliation=and the ABC Team",
        ],
      ],
    ),
    // (a name its events spell only through a macro's run: W 0 -> 2 promoted, advisor TEAMAUTH6 C)
    row(
      "names written from their text keep rc6's reading",
      doc(
        ARTICLE,
        r#"\author{\def\G{GAPS}Ann Able and the \G{} Team \\ CERN}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Ann Able",
        "collaboration=and the GAPS Team",
        "affiliation=CERN",
      ]],
    ),
    // (one run promotes all its teams or none: a dash frames a remark, a `;` lists none — advisor TEAMAUTH6 B)
    row(
      "a framed run keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Ann Able -- the XYZ Collaboration and the ABC Collaboration}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Ann Able",
        "collaboration=the XYZ Collaboration",
        "collaboration=the ABC Collaboration",
      ]],
    ),
    row(
      "a semicolon's run keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Ann Able; the XYZ Collaboration and the ABC Collaboration}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Ann Able",
        "collaboration=the XYZ Collaboration",
        "collaboration=the ABC Collaboration",
      ]],
    ),
    // (revtex's `\collaboration` is one group's name, no list: rc6's reading, 0705.3322, 2010.14527; the one-note
    // reading DG ruled is `revtex_collaboration_list_is_one_contact_red`'s — advisor TEAMAUTH6 A)
    row(
      "a revtex collaboration enumerates nothing",
      doc(
        r"\documentclass{revtex4-2}",
        r#"\author{Ann Able}\affiliation{CERN}\collaboration{XYZ Collaboration and ABC Collaboration}"#,
      ),
      (0, 0),
      &[&["role=author", "Ann Able", "affiliation=CERN"], &[
        "role=author",
        SEP,
        "XYZ Collaboration",
        "collaboration=ABC Collaboration",
      ]],
    ),
    // (the mark is the stomach's: a `\collaboration` expanded and never digested marks no later block — Gemini
    // TEAMAUTH.21 1)
    row(
      "an undigested revtex collaboration marks no later block",
      doc(
        r"\documentclass{revtex4-2}",
        concat!(
          r#"\author{Ann Able}\affiliation{CERN}"#,
          r#"\expandafter\def\expandafter\x\expandafter{\collaboration{Unused Collaboration}}"#,
          r#"\author{Bob Baker and the GAPS Team}\affiliation{DESY}"#,
          r#"\collaboration{XYZ Collaboration and ABC Collaboration}"#,
        ),
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able", "affiliation=CERN"],
        &["role=author", SEP, "Bob Baker", "affiliation=DESY"],
        &["role=author", SEP, "the GAPS Team", "affiliation=DESY"],
        &[
          "role=author",
          SEP,
          "XYZ Collaboration",
          "collaboration=ABC Collaboration",
        ],
      ],
    ),
    // (only lost ink moves to another creator: a picture's block keeps its promotion, W as rc6's — Gemini TEAMAUTH.21 2;
    // 06ec6712c31 gave it up for the unwritten picture)
    row(
      "an unwritten picture keeps the promotion",
      doc(
        "\\documentclass{article}\n\\usepackage{graphicx}",
        r#"\author{Ann Able and the GAPS Team \\ \includegraphics[height=1ex]{example-image} \texttt{ann@x.org}}"#,
      ),
      (0, 1),
      &[&["role=author", "Ann Able", "email=ann@x.org"], &[
        "role=author",
        SEP,
        "the GAPS Team",
      ]],
    ),
  ]);
}

/// Hand-picked lines of the generated harness (kit `teamauth_fuzz`), at most ten (DG), one behaviour each, every one red on
/// round 3 (3689214c83b) or round 4 (52eeaf68a5e) and green here (round 3's blocker B1 r02, a run of teams before a framed
/// remark, is a row of `a_group_run_before_a_place_or_a_remark_keeps_rc6`; a bare comma is "a company group after a comma
/// (advisor 3)"): the reading with the enumeration is rc6's plus its teams, else rc6's whole
/// (`resolver::enumeration`, OXIDIZED_DESIGN_DIVERGENCES #459). Each pdflatex-clean.
#[test]
fn harness_picks_keep_rc6_or_add_only_the_team() {
  check(&[
    // (the team an author, the place after it the last person's, as rc6's: a names line's institution never lands on a
    // listed group — ADV5 section 5)
    row(
      "a place after the listed team stays the person's",
      doc(
        ARTICLE,
        r#"\author{Ann Able, Bob Baker and the XYZ Collaboration, Theory Group}"#,
      ),
      (0, 0),
      &[
        &["role=author", "Ann Able"],
        &["role=author", SEP, "Bob Baker", "affiliation=Theory Group"],
        &["role=author", SEP, "the XYZ Collaboration"],
      ],
    ),
    // (a team's name holding a label before a colon is no author: rc6's reading, no safety-net Warning — harness-found)
    row(
      "a colon-labelled team falls back to rc6",
      doc(
        ARTICLE,
        r#"\author{Ann Able and R Core Team: KM3NeT Consortium}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Ann Able",
        "collaboration=R Core Team: KM3NeT Consortium",
      ]],
    ),
    // (an e-mail never moves to a team: the guard returns rc6's reading whole — 1611.07751, 1108.4847;
    // `author_team_thanks_email_is_the_persons_red` keeps the better reading's expectation)
    row(
      "an e-mail note on the team falls back to rc6",
      doc(
        ARTICLE,
        r#"\author{J.-F. Donati and the BCool Collaboration\thanks{E-mail: jf@a.edu}}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "J.-F. Donati",
        "collaboration=the BCool Collaboration",
        "note:thanks[*]=E-mail: jf@a.edu",
      ]],
    ),
    // (a team's own note is the one printed on its piece, marks and notes between at most; a later piece's or line's
    // stays the person's, so the reading is rc6's — harness F1, round 4's blocker class)
    row(
      "a team's own note moves with it",
      doc(
        ARTICLE,
        r#"\author{Ann Able \& the Speech Team\thanks{Note 1.} \\ CERN}"#,
      ),
      (0, 0),
      &[&["role=author", "Ann Able", "affiliation=CERN"], &[
        "role=author",
        SEP,
        "the Speech Team",
        "note:thanks[*]=Note 1.",
        "affiliation=CERN",
      ]],
    ),
    row(
      "a later piece's or line's note stays the person's",
      doc(
        ARTICLE,
        r#"\author{Cat Cole and the CMS Collaboration\thanks{Note 1.}, Alibaba Group\thanks{Note 2.} \\ CERN\thanks{Note 3.}}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Cat Cole",
        "collaboration=the CMS Collaboration",
        "note:thanks[*]=Note 1.",
        "affiliation=Alibaba Group",
        "note:thanks[\u{2020}]=Note 2.",
        "affiliation=CERN",
        "note:thanks[\u{2021}]=Note 3.",
      ]],
    ),
    // (a later collaboration contact never moves from the persons to the team — harness F3)
    row(
      "a later collaboration stays the person's",
      doc(
        ARTICLE,
        r#"\author{Ann Able -- the Speech Team \& LHD experimental group; the Machine Learning Group}"#,
      ),
      (0, 0),
      &[&[
        "role=author",
        "Ann Able",
        "collaboration=the Speech Team",
        "collaboration=LHD experimental group",
        "collaboration=the Machine Learning Group",
      ]],
    ),
    // (a `;` is no enumeration's comma: the charter's separators are `,`, "and", `&`, `\and` — harness F2)
    row(
      "a semicolon keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Fay Fox \& Dan Dore; the CMS Collaboration}"#,
      ),
      (0, 0),
      &[
        &[
          "role=author",
          "Fay Fox",
          "collaboration=the CMS Collaboration",
        ],
        &[
          "role=author",
          SEP,
          "Dan Dore",
          "collaboration=the CMS Collaboration",
        ],
      ],
    ),
    // (a team takes a copy of its column's line only where nothing else prints the line's words: the placed check would
    // count them more often than printed, a Warning rc6 has not — harness F4, round 4's L10212)
    row(
      "a line under that repeats a name keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Dan Dore \and the Solar Orbiter EPD and Parker Solar Probe ISIS teams \\ Dan Dore}"#,
      ),
      (0, 0),
      &[&["role=author", "Dan Dore"], &[
        "role=author",
        SEP,
        "the Solar Orbiter EPD",
        "collaboration=Parker Solar Probe ISIS teams",
        "affiliation=Dan Dore",
      ]],
    ),
    // (where the emitter's word check finds ink lost and not less than in rc6's, rc6's reading: the lost remark among
    // names (block written whole), the kept note on another creator — harness F5, F6)
    row(
      "a lost remark among the names keeps rc6's reading",
      doc(
        ARTICLE,
        r#"\author{Ann Able$^{1}$ \\ the ATLAS and CMS Collaborations \\ -- the VERITAS Collaboration -- \and the Speech Team$^{1}$}"#,
      ),
      (0, 1),
      &[&[
        "role=author",
        "Ann Able",
        "collaboration=the ATLAS and CMS Collaborations",
        "affiliation=the Speech Team",
        "note:note=the VERITAS Collaboration",
      ]],
    ),
    row(
      "a lost line stays the sole person's note",
      doc(
        ARTICLE,
        r#"\author{Dan Dore$^{1}$; the Machine Learning Group$^{1}$ \\ ATLAS Collaboration \\ Bob Baker}"#,
      ),
      (0, 1),
      &[&[
        "role=author",
        "Dan Dore",
        "note:footnotemark[1]=",
        "collaboration=the Machine Learning Group",
        "collaboration=ATLAS Collaboration",
        "note:note=Bob Baker",
      ]],
    ),
  ]);
}

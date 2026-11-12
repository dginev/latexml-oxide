//! Guards for batch C1: what the frontmatter queue's names ask of their marks is read once per entry and kept with the
//! queue (`frontmatter/queue_reading.rs`), where each of a block's contacts read every name again: the Cut
//! (`frontmatter_api::cut::cut_marked_contact`, 2103.04797's 975 authors; 2005.08745's 592 are labelled, so it leaves them at
//! once) and AASTeX's `\affiliation` (`relocate::queued_name_marks`, 1901.01540's 1,202). The output is the base's, byte
//! for byte (int142a); the count is the mechanism.
use std::process::Command;

use latexml::util::test::convert_files_with_setup_then;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::creators as text_creators,
  perfect_kernel_batchz4::run,
};

/// Converts `tex` under the arXiv preload with `--debug=frontmatter`, checks that it errs and warns nothing, and returns
/// its log and creators.
fn converted(tex: &str) -> (String, String) {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      "--preload=ar5iv.sty",
      "--debug=frontmatter",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).expect("the XML");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0));
  (log, creators(&xml))
}

/// Every creator of `xml`, whitespace squashed.
fn creators(xml: &str) -> String {
  let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
  let start = xml.find("<creator").expect("a creator");
  let end = xml.rfind("</creator>").expect("a closed creator") + "</creator>".len();
  xml[start..end].to_string()
}

/// A marked block under `\documentclass{revtex4-2}`: `names` names each with its own mark, half the marks answered by an
/// `\affiliation` of its own and half by one list. Returns the source and the creators the base (int142a) makes of it,
/// which are the document's: each name holds the entry its mark requests, the mark shown (P4c, P4S Q1: pdflatex prints
/// "1University 1, Town").
fn marked_block(names: usize) -> (String, String) {
  let name = |n: usize| format!("Ann{} Able{}", letters(n), letters(n + 1));
  let mut tex = String::from("\\documentclass{revtex4-2}\n\\begin{document}\n\\title{T}\n");
  let mut expected = Vec::new();
  for n in 1..=names {
    tex.push_str(&format!("\\author{{{}$^{{{n}}}$}}\n", name(n)));
    let before = if n == 1 { "" } else { r#" before=" ""# };
    expected.push(format!(
      r#"<creator{before} role="author"> <personname>{}<sup xml:id="id{}"><text font="italic" xml:id="id{}.1">{n}</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id{}"><text font="italic" xml:id="id{}.1">{n}</text></sup>University {n}, Town</contact> </creator>"#,
      name(n),
      2 * n - 1,
      2 * n - 1,
      2 * n,
      2 * n,
    ));
  }
  let half = names / 2;
  for n in 1..=half {
    tex.push_str(&format!(
      "\\affiliation{{$^{{{n}}}$University {n}, Town}}\n"
    ));
  }
  let list: Vec<String> = (half + 1..=names)
    .map(|n| format!("$^{{{n}}}$University {n}, Town"))
    .collect();
  tex.push_str(&format!("\\affiliation{{{}}}\n", list.join("\\\\\n")));
  tex.push_str("\\begin{abstract}A\\end{abstract}\n\\maketitle\nBody.\n\\end{document}\n");
  (tex, expected.join(" "))
}

/// The letters a name is told apart by (`aa`, `ba`, …): a digit in it would read as a mark's.
fn letters(n: usize) -> String {
  let digit = |n: usize| char::from(b'a' + u8::try_from(n % 26).expect("under 26"));
  format!("{}{}", digit(n), digit(n / 26))
}

/// The `--debug=frontmatter` log's counts: entries read and entries queued, each time the reading is forgotten.
fn reading_counts(log: &str) -> Vec<(usize, usize)> {
  log
    .lines()
    .filter_map(|line| {
      let rest = line.split("the queue's reading: ").nth(1)?;
      let (read, queued) = rest.split_once(" entries read for ")?;
      Some((
        read.parse().ok()?,
        queued.split_whitespace().next()?.parse().ok()?,
      ))
    })
    .collect()
}

/// A block of 300 names, each its own mark, 150 of the entries in `\affiliation`s of their own and 150 in one list. Each
/// name holds its entry, whole; and the flush's reading read each queued entry once (`entries read` equals `queued`),
/// though every contact asked (`--debug=frontmatter`; a reading given up between two askers counts its entries again).
#[test]
fn marked_block_reads_each_entry_once() {
  let (tex, expected) = marked_block(300);
  let (log, creators) = converted(&tex);
  assert_eq!(creators, expected);
  let counts = reading_counts(&log);
  let (read, queued) = *counts.last().expect("the flush's count");
  assert!(
    queued >= 450,
    "the flush's reading read the block: {counts:?}"
  );
  assert_eq!(read, queued, "each entry read once: {counts:?}");
}

/// A `\let` after the first names: `\up` means no mark when the first `\address` reads Ann's name and one after
/// `\let\up\textsuperscript`. Since F0 (Option C P4a) the reading keeps each name's marks as its first rehearsal read
/// them (the meaning tracking deleted, F0 report), so the later `\address` sees one request and the Cut leaves the list
/// whole: the flush's readers print the same creators (was one "cut address at its marks"). Witness of the `\let`
/// shape 2209.14855. Repro sectioning-frontmatter/queue_reading_alias_after_names.
#[test]
fn a_changed_meaning_reads_the_names_again() {
  let (log, creators) = converted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/queue_reading_alias_after_names.tex"
  ));
  assert_eq!(
    creators,
    r#"<creator role="author"> <personname>Ann Able<sup xml:id="id1">1</sup></personname> <contact name="Address: " role="address"><sup xml:id="id2"><text font="italic" xml:id="id2.1">1</text></sup>Univ Z</contact> <contact name="Address: " role="address"><sup xml:id="id3">1</sup>Univ A</contact> </creator> <creator before=" and " role="author"> <personname>Bob Baker<sup xml:id="id4">2</sup></personname> <contact name="Address: " role="address"><sup xml:id="id5">2</sup>Univ B</contact> </creator>"#
  );
  assert_eq!(log.matches("FRONT cut address at its marks").count(), 0);
}

/// The reading trusts the queue's length, so a removal forgets it: `\date{}` and `\keywords{}` dequeue two entries and
/// `\author{Bob}` and `\affiliation` queue two, back to 4. A reading kept across that hides Bob, no name requests mark 2,
/// and the Cut leaves the list whole. Repro sectioning-frontmatter/queue_reading_dequeue_same_length.
#[test]
fn a_removal_forgets_the_reading() {
  let (log, creators) = converted(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/queue_reading_dequeue_same_length.tex"
  ));
  assert_eq!(
    creators,
    r#"<creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2"><text font="italic" xml:id="id2.1">1</text></sup>Univ Z</contact> <contact name="Affiliation: " role="affiliation"><sup xml:id="id3"><text font="italic" xml:id="id3.1">1</text></sup>Univ A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id5"><text font="italic" xml:id="id5.1">2</text></sup>Univ B</contact> </creator>"#
  );
  assert_eq!(log.matches("FRONT cut affiliation at its marks").count(), 1);
}

/// A row: the repro (name, source), its Warning count, and its creators (name, then contacts).
type Row<'a> = ((&'a str, &'a str), usize, &'a [&'a [&'a str]]);

/// Option C F0/P4a: the queue's names request the labels their digested marks print (`queue_reading::author_marks`,
/// `argument_marks`), and the binding's gate reads the list as digested (`affiliation_is_marked`, P4c: each mark shown,
/// P4S Q1, where the binding's own cut dropped them). One row a
/// shape: a marked list, a macro printing the mark (its list keeps the printed marks, as int160c), a `\let` copy, the
/// unknown class's path, a mark inside a box; amsart's `\address` list repeated, its macro-printed marks requested by
/// printed text where the Cut's uncut copies give way (`place_uncut_copies`, review revF0 B1: three entries, two). Each:
/// no Error, its Warning count, every creator's name and contacts.
#[test]
fn queued_marks_read_as_digested() {
  macro_rules! repro {
    ($name:literal) => {
      (
        $name,
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
          $name,
          ".tex"
        )),
      )
    };
  }
  let rows: &[Row] = &[
    (repro!("queue_marks_revtex_marked_list"), 0, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 2", "affiliation=2 Univ B, Town B"],
    ]),
    (repro!("queue_marks_revtex_macro_held_marks"), 0, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 2", "affiliation=2 Univ B, Town B"],
    ]),
    (repro!("queue_marks_revtex_let_copy_marks"), 0, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 2", "affiliation=2 Univ B, Town B"],
    ]),
    (repro!("queue_marks_unknown_class_marked_list"), 1, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 2", "affiliation=2 Univ B, Town B"],
    ]),
    (repro!("queue_marks_revtex_boxed_marks"), 0, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 2", "affiliation=2 Univ B, Town B"],
    ]),
    (
      repro!("queue_marks_ams_macro_marks_three_entry_copies"),
      0,
      &[
        &["Ann Able 1", "address=1 Univ A, Town A"],
        &["Bob Baker 2", "address=2 Univ B, Town B"],
        &["Cy Cole 1", "address=1 Univ A, Town A"],
        &["address=3 Univ C, Town C"],
      ],
    ),
    (repro!("queue_marks_ams_macro_marks_repeated_list"), 0, &[
      &["Ann Able 1", "address=1 Univ A, Town A"],
      &["Bob Baker 2", "address=2 Univ B, Town B"],
    ]),
    // (P4b2: the Cut's decision reads the list's marks as digested, `requested_entry_labels` — marks a document macro
    // prints, a math-active `^` no superscript token spells, the list repeated, the unknown class; the per-line cutter
    // parts none of these lines, so each list stays whole and the observer places its entries)
    (repro!("api_cut_macro_marks_list"), 0, &[
      &["Ann Able 1", "address=1 Univ A, Town A"],
      &["Bob Baker 2", "address=2 Univ B, Town B"],
    ]),
    (repro!("api_cut_math_active_marks_list"), 0, &[
      &["Ann Able 1", "address=1 Univ A, Town A"],
      &["Bob Baker 2", "address=2 Univ B, Town B"],
    ]),
    (repro!("api_cut_macro_marks_repeated_list"), 0, &[
      &["Ann Able 1", "address=1 Univ A, Town A"],
      &["Bob Baker 2", "address=2 Univ B, Town B"],
    ]),
    (repro!("api_cut_unknown_class_macro_marks_list"), 1, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 2", "affiliation=2 Univ B, Town B"],
    ]),
  ];
  for ((name, tex), warnings, expected) in rows {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, *warnings),
      "{name}: {log}"
    );
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|c| c.iter().map(|s| s.to_string()).collect())
      .collect();
    assert_eq!(text_creators(&xml), expected, "{name}: {xml}");
  }
}

/// Option C P4d: `class_numbered_institutes` reads a numbered store by meaning — its entries at the kernel's `\and` a
/// document macro holds (`split_at_kernel_separators`), its own marks at its level (`argument_marks`, a boxed mark
/// none) — and the authors' requests by printed text (`numbered_marks_requested`, F0). One row a path: the store route
/// under llncs's `\institutename` (numinst.cls, preprint.cls 2405.16615) and under sab's `\refstepcounter` (2308.05831),
/// the displaced route (`displaced_entries`, astro-ph9805069), the store's boxed mark, `\inst` given by a macro. Each:
/// no Error, its Warning count, every creator's name and contacts.
#[test]
fn class_numbered_institutes_read_by_meaning() {
  macro_rules! repro {
    ($name:literal) => {
      (
        $name,
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
          $name,
          ".tex"
        )),
      )
    };
  }
  let numinst =
    include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/numinst.cls");
  let rows: &[Row] = &[
    (
      repro!("class_numbered_institutes_macro_held_separator"),
      0,
      &[&["Ann Able", "affiliation=Univ A, Town A"], &[
        "Bob Baker",
        "affiliation=Univ B, Town B",
      ]],
    ),
    // (a store mark a document macro prints, no name requesting it, is the entry's text: the numbers stand — revP4d S1,
    // red on P4d: both authors both entries)
    (
      repro!("class_numbered_institutes_macro_store_mark_gives_every_entry"),
      0,
      &[&["Ann Able", "affiliation=a Univ A, Town A"], &[
        "Bob Baker",
        "affiliation=Univ B, Town B",
      ]],
    ),
    (
      repro!("raw_class_refstepcounter_numbered_institutes_macro_held_separator"),
      0,
      &[
        &["Ann Able", "affiliation=Univ A, Town A"],
        &["Bob Baker", "affiliation=Univ B, Town B"],
        &["Cy Cole", "affiliation=Univ A, Town A"],
      ],
    ),
    (
      repro!("displaced_class_numbered_institutes_macro_held_separator"),
      0,
      &[
        &["Ann Able", "affiliation=1 Univ A, Town"],
        &["Bob Baker", "affiliation=2 Univ B, City"],
        &[
          "Cy Cole",
          "affiliation=1 Univ A, Town",
          "affiliation=2 Univ B, City",
        ],
      ],
    ),
    (repro!("class_numbered_institutes_boxed_store_mark"), 0, &[
      &["Ann Able", "affiliation=a Univ A, Town A"],
      &["Bob Baker", "affiliation=Univ B, Town B"],
    ]),
    (
      repro!("raw_class_refstepcounter_numbered_institutes_inst_by_macro"),
      0,
      &[&["Ann Able", "affiliation=Univ A, Town A"], &[
        "Bob Baker",
        "affiliation=Univ B, Town B",
      ]],
    ),
  ];
  for ((name, tex), warnings, expected) in rows {
    let (log, xml, ()) = convert_files_with_setup_then(
      tex,
      &[("numinst.cls", numinst)],
      Some("ar5iv.sty"),
      || {},
      |_| (),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, *warnings),
      "{name}: {log}"
    );
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|c| c.iter().map(|s| s.to_string()).collect())
      .collect();
    assert_eq!(text_creators(&xml), expected, "{name}: {xml}");
  }
}

/// Every note and pubnote of `xml` in document order, as `tag:role=text`.
fn notes(xml: &str) -> Vec<String> {
  let note =
    regex::Regex::new(r#"(?s)<(note|pubnote)\b[^>]*role="([^"]*)"[^>]*>(.*?)</(?:note|pubnote)>"#)
      .expect("regex");
  note
    .captures_iter(xml)
    .map(|c| {
      format!(
        "{}:{}={}",
        &c[1],
        &c[2],
        super::perfect_kernel_batch84b::text(&c[3])
      )
    })
    .collect()
}

/// Option C P4c: a binding's marked list (`\lx@add@affiliation@marked`, IEEEtran's `\IEEEauthorblockA`, iopart's
/// `\address`) is cut by the API's Cut as the binding declares it (`MARKED_LIST_CUT`, OD #1009): at every mark, an
/// address line entry text, legends and notes notes, no placement attribute; each mark shown (Q1). One row a witness's
/// shape (red on the P4c WIP): its Warning count, every creator's name and contacts, every note.
#[test]
fn marked_list_cut_as_its_binding_declares() {
  macro_rules! repro {
    ($name:literal) => {
      (
        $name,
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
          $name,
          ".tex"
        )),
      )
    };
  }
  type NotedRow<'a> = (
    (&'a str, &'a str),
    usize,
    &'a [&'a [&'a str]],
    &'a [&'a str],
  );
  let rows: &[NotedRow] = &[
    (
      repro!("ieee_marked_list_blocks_tail_email_line"),
      0,
      &[
        &[
          "Ann Able *",
          "affiliation=* Univ A, Town A",
          "affiliation=Email: * first.last@a.edu, † bob@b.edu",
        ],
        &["Bob Baker †", "affiliation=† Univ B, Town B"],
        &[
          "Cy Cole *",
          "affiliation=* Univ A, Town A",
          "affiliation=Email: * first.last@a.edu, † bob@b.edu",
        ],
      ],
      &[],
    ),
    (
      repro!("ieee_marked_list_unmarked_first_line"),
      2,
      &[
        &["Ann Able", "affiliation=Univ A, Town A", "email=ann@a.edu"],
        &["Bob Baker", "affiliation=Univ A, Town A", "email=bob@a.edu"],
        &[
          "Cy Cole 2",
          "affiliation=Univ A, Town A",
          "affiliation=† Centre B, Town B",
          "email=cy@b.ca",
        ],
      ],
      &[],
    ),
    (
      repro!("ieee_marked_list_notes_led_by_marks"),
      0,
      &[
        &[
          "Ann Able *",
          "affiliation=Univ A",
          "email=ann@a.edu",
          "affiliation=* A. Able is now with Lab C.",
        ],
        &["Bob Baker", "affiliation=Univ A", "email=bob@a.edu"],
      ],
      &["pubnote:thanks=† This version corrects typos."],
    ),
    (
      repro!("ieee_marked_list_single_entry_blocks_email_tail"),
      0,
      &[
        &[
          "Ann Able *",
          "affiliation=* Univ A, Town A",
          "email=ann@a.edu",
        ],
        &[
          "Bob Baker †",
          "affiliation=† Univ B, Town B",
          "email=bob@b.edu",
        ],
        &[
          "Cy Cole *",
          "affiliation=* Univ A, Town A",
          "email=cy@a.edu",
        ],
      ],
      &[],
    ),
    // (RED, the base's placement pinned with each mark shown — ieee_marked_list_email_block_last_address_nameless_red)
    (
      repro!("ieee_marked_list_unmarked_note_block"),
      0,
      &[
        &[
          "Ann Able *",
          "affiliation=* Univ A, Town A.",
          "affiliation=Bob Baker is the corresponding author.",
          "email=ann@a.edu",
        ],
        &[
          "Bob Baker * †",
          "affiliation=* Univ A, Town A.",
          "affiliation=† Centre B, Town B.",
          "affiliation=Bob Baker is the corresponding author.",
          "email=bob@b.edu",
        ],
        &[
          "Cy Cole ‡",
          "affiliation=‡ Univ C, Town C.",
          "affiliation=Bob Baker is the corresponding author.",
        ],
        &["email=cy@c.edu"],
      ],
      &[],
    ),
    // (RED, the base's placement pinned with each mark shown — ieee_marked_list_star_led_names_entries_nameless_red)
    (
      repro!("ieee_marked_list_star_legend_tail"),
      0,
      &[&["Ann Able"], &["Bob Baker"], &["Cy Cole"], &[
        "affiliation=† Univ A (A*STAR), Town A",
        "affiliation=‡ Univ B, Town B",
      ]],
      &[
        "note:thanks=*equally contributed",
        "note:thanks=*equally contributed",
        "note:thanks=*equally contributed",
      ],
    ),
    (
      repro!("ieee_marked_list_legend_line"),
      0,
      &[&["Ann Able"], &["Bob Baker"], &[
        "Cy Cole †",
        "affiliation=School A , Univ A , Town A",
      ]],
      &["note:thanks=† Corresponding Author. E-mail: cy@a.edu"],
    ),
    (
      repro!("ieee_marked_list_textsuperscript_note_tail"),
      0,
      &[
        &[
          "Ann Able * §",
          "affiliation=* Univ A, Town A",
          "affiliation=Accepted at a conference (DOI pending)",
        ],
        &[
          "Bob Baker *",
          "affiliation=* Univ A, Town A",
          "affiliation=Accepted at a conference (DOI pending)",
        ],
        &[
          "Cy Cole †",
          "affiliation=† Univ C, Town C",
          "affiliation=Accepted at a conference (DOI pending)",
          "email=cy@c.edu",
        ],
      ],
      &["note:thanks=ann@a.edu § Corresponding author"],
    ),
    // (names in a `\parbox`: their marks are the names' own, so the binding cuts the list, the link line after the
    // entries the last entry's — 2508.17623, red on P4c2: the url in no creator, 1 Warning)
    (
      repro!("ieee_marked_list_url_line_after_entries"),
      0,
      &[
        &["Ann Able § *", "affiliation=§ Univ A"],
        &["Bob Baker † *", "affiliation=† Univ B"],
        &[
          "Cy Cole ¶",
          "affiliation=¶ Univ D",
          "url=https://example.org/project/",
        ],
        &["Di Dow ‡", "affiliation=‡ Univ C"],
      ],
      &[],
    ),
    // (a note after the list printed once, no part of every entry — 2605.05837, red on P4c2: once per entry)
    (
      repro!("ieee_marked_list_note_after_the_list_once"),
      0,
      &[&["Ann Able *", "affiliation=* Univ A, Town A"], &[
        "Bob Baker †",
        "affiliation=† Univ B, Town B.",
      ]],
      &["pubnote:thanks=Supported by Grant 1."],
    ),
    // (RED, the base's: "Univ A, Town A" Cy Cole's alone — iopart_marked_list_unmarked_first_line_every_authors_red)
    (
      repro!("iopart_marked_list_unmarked_first_line"),
      0,
      &[
        &["Ann Able", "email=ann@a.edu"],
        &["Bob Baker 1", "affiliation=1 Univ B, Town B"],
        &["Cy Cole", "affiliation=Univ A, Town A"],
      ],
      &[],
    ),
    // (P4cR1, `at_own_level`: a mark in a box of its own counts — names and lists, IEEEtran, 2307.12121, red on P4b2b:
    // lj1994@ on a nameless creator; REVTeX's per-call cut, red: the marks lost, 2 Warnings — and a mark above words
    // each in a box, apa6, 2508.05776, red: the whole list in every note)
    (
      repro!("ieee_marked_list_marks_in_boxes_of_their_own"),
      0,
      &[
        &[
          "Ann Able 1",
          "affiliation=1 Univ A, Town",
          "email=ann@x.org",
        ],
        &[
          "Bob Baker 2,1",
          "affiliation=1 Univ A, Town",
          "affiliation=2 Univ B, City",
          "email=lj1994@y.org",
        ],
        &[
          "Cy Cole 1",
          "affiliation=1 Univ A, Town",
          "email=cole@z.org",
        ],
      ],
      &[],
    ),
    (
      repro!("revtex_marked_list_marks_in_boxes_of_their_own"),
      0,
      &[&["Ann Able 1", "affiliation=1 Univ A, Town A"], &[
        "Bob Baker 2",
        "affiliation=2 Univ B, Town B",
      ]],
      &[],
    ),
    (
      repro!("apa6_marked_list_entries_in_boxes"),
      0,
      &[
        &["Ann Able 1 ∗", "affiliation=Univ A, Town"],
        &["Bob Baker 2 ∗", "affiliation=Univ B, City"],
        &["Cy Cole 3 ∗", "affiliation=Univ C, Ville"],
      ],
      &[
        "note:thanks=∗ Equal contribution",
        "note:thanks=∗ Equal contribution",
        "note:thanks=∗ Equal contribution",
      ],
    ),
  ];
  let mut failures = Vec::new();
  for ((name, tex), warnings, expected, expected_notes) in rows {
    let (log, xml) = run(tex);
    let got = (
      (error_count(&log), warning_count(&log)),
      text_creators(&xml),
      notes(&xml),
    );
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|c| c.iter().map(|s| s.to_string()).collect())
      .collect();
    let expected_notes: Vec<String> = expected_notes.iter().map(|s| s.to_string()).collect();
    if got != ((0, *warnings), expected, expected_notes) {
      failures.push(format!("{name}: {got:?}"));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Option C P4b: the bindings' "a label leads this argument" read as digested (`leads_with_label`), a document macro's
/// or a class macro's mark leading as `$^{1}$` does. One row a site: jfm's `\affiliation` (red on P4bbase: the list a
/// note), REVTeX's per-call `\affiliation`, IEEEtran's stop after a grade, the ws_procl `\address`; and the API's Cut
/// over macro marks, pinned as today (its "marked" stays on tokens). Each: no Error, its Warning count, every creator.
#[test]
fn marks_lead_as_digested() {
  macro_rules! repro {
    ($name:literal) => {
      (
        $name,
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
          $name,
          ".tex"
        )),
      )
    };
  }
  let rows: &[Row] = &[
    (repro!("jfm_aff_macro_marked_list"), 0, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 2", "affiliation=2 Univ B, Town B"],
    ]),
    (repro!("revtex_macro_mark_affiliation_calls"), 0, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 2", "affiliation=2 Univ B, Town B"],
    ]),
    (repro!("ams_cut_macro_mark_lines"), 0, &[
      &["Ann Able 1", "address=1 Univ A, Town A"],
      &["Bob Baker 2", "address=2 Univ B, Town B"],
    ]),
    (repro!("ieee_names_macro_mark_note_label"), 0, &[
      &["Ann Able", "affiliation=Univ A, Town A."],
      &[
        "Bob Baker",
        "membership=Member, IEEE",
        "affiliation=Univ A, Town A.",
      ],
    ]),
    (repro!("ws_procl_macro_mark_address"), 0, &[
      &["A. ABLE a", "affiliation=a Univ A, Town A"],
      &["B. BAKER b", "affiliation=b Univ B, Town B"],
    ]),
  ];
  let mut failures = Vec::new();
  for ((name, tex), warnings, expected) in rows {
    let (log, xml) = run(tex);
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|c| c.iter().map(|s| s.to_string()).collect())
      .collect();
    let got = (
      (error_count(&log), warning_count(&log)),
      text_creators(&xml),
    );
    if got != ((0, *warnings), expected) {
      failures.push(format!("{name}: {got:?}"));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// P4b2b (2410.01932, Q1 a): a binding's list of names (`\lx@add@authors@list`: authblk, iopart, the unknown class) whose
/// affiliations are their own calls, each printing its mark, shows the marks each name prints, answered or not
/// (`per_author_marks`); authblk's labelled `\author[1]` keeps its own as before. Red on P4b2: every listed name bare.
#[test]
fn authblk_names_show_their_printed_marks() {
  macro_rules! repro {
    ($name:literal) => {
      (
        $name,
        include_str!(concat!(
          "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
          $name,
          ".tex"
        )),
      )
    };
  }
  let affiliations: &[&[&str]] = &[&["Ann Able 1", "affiliation=1 Univ A, Town A"], &[
    "Bob Baker 2",
    "affiliation=2 Univ B, Town B",
  ]];
  let rows: &[Row] = &[
    (repro!("authblk_hand_marks_answered_shown"), 0, affiliations),
    (
      repro!("authblk_hand_marks_one_call_answered_shown"),
      0,
      affiliations,
    ),
    (repro!("authblk_hand_marks_shared_answered_shown"), 0, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 1", "affiliation=1 Univ A, Town A"],
    ]),
    (repro!("authblk_hand_marks_unknown_class"), 1, affiliations),
    (repro!("iopart_hand_marks_answered_shown"), 0, affiliations),
    (repro!("jfm_authblk_aff_answered_name_marks"), 0, &[
      &["Ann Able 1", "affiliation=1 Univ A, Town A"],
      &["Bob Baker 1", "affiliation=1 Univ A, Town A"],
    ]),
    (repro!("authblk_labelled_author_hand_mark_kept"), 0, &[
      &["Ann Able *", "affiliation=Univ A, Town A"],
      &["Bob Baker", "affiliation=Univ B, Town B"],
    ]),
    // (a mark no entry answers and no request could show, printed: rev108a B2's shape, 1 Warning on P4b2)
    (repro!("authblk_unanswered_unshowable_mark_printed"), 0, &[
      &["Ann Able a-b"],
      &["Bob Baker"],
    ]),
  ];
  for ((name, tex), warnings, expected) in rows {
    let (log, xml) = run(tex);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, *warnings),
      "{name}: {log}"
    );
    let expected: Vec<Vec<String>> = expected
      .iter()
      .map(|c| c.iter().map(|s| s.to_string()).collect())
      .collect();
    assert_eq!(text_creators(&xml), expected, "{name}: {xml}");
    // (the mark the name prints, digested in its own superscript, as the affiliation's)
    if *name == "authblk_hand_marks_answered_shown" {
      assert_eq!(
        creators(&xml),
        r#"<creator role="author"> <personname>Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id2"><text font="italic" xml:id="id2.1">1</text></sup>Univ A, Town A</contact> </creator> <creator before=" " role="author"> <personname>Bob Baker<sup xml:id="id3"><text font="italic" xml:id="id3.1">2</text></sup></personname> <contact name="Affiliation: " role="affiliation"><sup xml:id="id4"><text font="italic" xml:id="id4.1">2</text></sup>Univ B, Town B</contact> </creator>"#,
        "{name}"
      );
    }
  }
}

/// revP4b2 S1: the API Cut's cost gate (`may_open_with_mark`) reads on past a macro printing nothing — an empty body
/// (`\newcommand\affilfont{}`), one of only braces, `\empty` two deep — to the list's first mark, so the list is cut once
/// (`--debug=frontmatter`'s cut line). Red on P4b2 for the braces' body: no cut, the observer reading the list.
#[test]
fn api_cut_gate_reads_past_empty_bodies() {
  let rows: [(&str, &str); 2] = [
    (
      "api_cut_gate_empty_macro_before_marks",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/api_cut_gate_empty_macro_before_marks.tex"
      ),
    ),
    (
      "api_cut_gate_nested_empty_macro_before_marks",
      include_str!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/api_cut_gate_nested_empty_macro_before_marks.tex"
      ),
    ),
  ];
  for (name, tex) in rows {
    let (log, xml, ()) = convert_files_with_setup_then(
      tex,
      &[],
      Some("ar5iv.sty"),
      || latexml_core::common::error::enable_debug_feature("frontmatter"),
      |_| (),
    );
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    assert_eq!(
      text_creators(&xml),
      [["Ann Able 1", "address=1 Univ A, Town A"], [
        "Bob Baker 2",
        "address=2 Univ B, Town B"
      ],],
      "{name}: {xml}"
    );
    assert_eq!(
      log.matches("FRONT cut address at its marks").count(),
      1,
      "{name}: {log}"
    );
  }
}

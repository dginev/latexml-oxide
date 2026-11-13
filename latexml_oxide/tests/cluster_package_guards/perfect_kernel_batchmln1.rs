//! Guards for batch MLN1: names led by their marks read as names, not affiliations. A line of an author block whose
//! first ink is a mark is a names line when its every entry reads as names, it lies above the block's first line setting
//! labels, and its marks are requests: answered by a later line, set by an affiliation call outside the block
//! (`\institute`'s n-th entry, `\affiliation[..]`, a mark-led `\address` line), or, no line setting any, the whole block
//! is such lines (`resolver::lines::names_led_by_marks`; OXIDIZED_DESIGN_DIVERGENCES #1019). Each row converts its repro
//! under `tools/perfect_kernel/repros/sectioning-frontmatter/` (pdflatex-checked, header oracle) and asserts its exact
//! Error and Warning counts and every creator whole: its name, each contact and note. Witnesses 2507.23021, 2605.06126,
//! 2505.14984, 1105.2204, 2205.03535 (PoS), 2606.07481 (oscmjournal, OmniBus), 2609.16600 (IEEEtran).
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::text,
};

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

/// Each creator of `xml`, in order: its personname's text, then each contact as `role=text` and each note as
/// `note:role=text`, in document order.
fn whole_creators(xml: &str) -> Vec<Vec<String>> {
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let creator = &xml[start..start + xml[start..].find("</creator>").expect("closed creator")];
      let mut out = Vec::new();
      if let Some(at) = creator.find("<personname>") {
        let close = at
          + creator[at..]
            .find("</personname>")
            .expect("closed personname");
        out.push(text(&creator[at + "<personname>".len()..close]));
      }
      let mut items: Vec<(usize, &str)> = creator
        .match_indices("<contact ")
        .chain(creator.match_indices("<note "))
        .collect();
      items.sort_unstable();
      for (at, open) in items {
        let kind = &open[1..open.len() - 1];
        let open_end = at + creator[at..].find('>').expect("an open tag");
        let role = creator[at..open_end]
          .split("role=\"")
          .nth(1)
          .and_then(|r| r.split('"').next())
          .unwrap_or("");
        let close = open_end
          + creator[open_end..]
            .find(&format!("</{kind}>"))
            .expect("a closed tag");
        let prefix = if kind == "note" { "note:" } else { "" };
        out.push(format!(
          "{prefix}{role}={}",
          text(&creator[open_end + 1..close])
        ));
      }
      out
    })
    .collect()
}

type Row = (
  (&'static str, &'static str),
  (usize, usize),
  &'static [&'static [&'static str]],
);

/// Every row's failures, all reported together.
fn check(rows: &[Row]) {
  let mut failures = Vec::new();
  for ((name, tex), counts, expected) in rows {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    let got = (
      (error_count(&log), warning_count(&log)),
      whole_creators(&xml),
    );
    let want: Vec<Vec<String>> = expected
      .iter()
      .map(|c| c.iter().map(|s| (*s).to_string()).collect())
      .collect();
    if got != (*counts, want) {
      failures.push(format!("{name}: {got:?}"));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Names their marks lead, each its own creator with what its marks request. Red on int169a (54ccc986ef): a nameless
/// creator holding the names as affiliations (silent), or the names after the first lost or another's affiliations. A
/// mark no contact answers keeps its text in the name, counted by the safety net's Warning (the post-merge mark-text
/// batch: `pos_leading_mark_kept_in_the_first_name_red`). Rows: the kernel's `\and` columns alone, over an affiliation
/// row, sharing a label, one label, a column grid; `\textsuperscript` marks; two comma rows sharing labels (2507.23021)
/// and a row of new labels (2605.06126); a symbol lead with trailing numbers (2505.14984); a `\thanks` (0710.4115); aa's and OmniBus's `\inst` answered by `\institute` (2606.07481), one institute; mnras's `\and`; PoS's
/// address lines (2205.03535).
#[test]
fn batchmln1_mark_led_names_read_as_names() {
  check(&[
    (repro!("aa_leading_inst_marks_lead_their_names"), (0, 0), &[
      &["Ann Able", "affiliation=Inst A"],
      &["Bob Baker", "affiliation=Inst B"],
      &["Cat Cole", "affiliation=Inst C"],
    ]),
    (repro!("aa_leading_inst_marks_one_institute"), (0, 0), &[
      &["Ann Able", "affiliation=Inst A"],
      &["Bob Baker", "affiliation=Inst A"],
    ]),
    (
      repro!("article_mark_led_names_and_columns_one_label"),
      (0, 1),
      &[&["Ann Able 1"], &["Bob Baker 1"]],
    ),
    (
      repro!("article_mark_led_names_and_columns_sharing_a_label"),
      (0, 0),
      &[
        &["Ann Able", "affiliation=Inst A"],
        &["Bob Baker", "affiliation=Inst A"],
        &["Cat Cole", "affiliation=Inst B"],
      ],
    ),
    (repro!("article_mark_led_names_column_grid"), (0, 0), &[
      &["Ann Able", "affiliation=Inst A"],
      &["Bob Baker", "affiliation=Inst B"],
    ]),
    (repro!("article_mark_led_names_in_and_columns"), (0, 1), &[
      &["Ann Able 1"],
      &["Bob Baker 2"],
    ]),
    (
      repro!("article_mark_led_names_over_an_affiliation_row"),
      (0, 0),
      &[&["Ann Able", "affiliation=Inst A"], &[
        "Bob Baker",
        "affiliation=Inst B",
      ]],
    ),
    (
      repro!("article_mark_led_names_second_row_new_labels"),
      (0, 0),
      &[
        &["Ann Able", "affiliation=Inst A"],
        &["Bob Baker", "affiliation=Inst B"],
        &["Cat Cole", "affiliation=Inst A"],
        &["Dan Dole", "affiliation=Inst C"],
        &["Eve Eads", "affiliation=Inst D"],
        &["Fay Ford", "affiliation=Inst A"],
      ],
    ),
    (
      repro!("article_mark_led_names_two_rows_sharing_labels"),
      (0, 0),
      &[
        &["Ann Able", "affiliation=Inst A, Italy"],
        &["Bob Baker", "affiliation=Inst A, Italy"],
        &["Cat Cole", "affiliation=Inst B, Italy"],
        &["Dan Dole", "affiliation=Inst A, Italy"],
        &["Eve Eads", "affiliation=Inst B, Italy"],
        &["Fay Ford", "affiliation=Inst A, Italy"],
      ],
    ),
    (repro!("article_mark_led_names_with_a_thanks"), (0, 0), &[
      &["Ann Able", "note:thanks=Funded by X.", "affiliation=Inst A"],
      &["Bob Baker", "affiliation=Inst B"],
    ]),
    (
      repro!("article_symbol_led_names_with_number_marks"),
      (0, 0),
      &[
        &[
          "Ann Able",
          "affiliation=Inst A",
          "note:thanks=† Equal contribution",
        ],
        &[
          "Bob Baker",
          "affiliation=Inst B",
          "note:thanks=† Equal contribution",
        ],
        &["Cat Cole", "affiliation=Inst B"],
      ],
    ),
    (
      repro!("article_textsuperscript_led_names_over_an_affiliation_row"),
      (0, 0),
      &[&["Ann Able", "affiliation=Inst A"], &[
        "Bob Baker",
        "affiliation=Inst B",
      ]],
    ),
    (repro!("mnras_mark_led_names_in_and_columns"), (0, 0), &[
      &["Ann Able", "affiliation=Inst A"],
      &["Bob Baker", "affiliation=Inst B"],
    ]),
    (
      repro!("omnibus_leading_inst_marks_lead_their_names"),
      (0, 1),
      &[
        &["Ann Able", "affiliation=Inst A"],
        &["Bob Baker", "affiliation=Inst B"],
        &["Cat Cole", "affiliation=Inst C"],
      ],
    ),
    (repro!("pos_mark_led_names_over_address_lines"), (0, 0), &[
      &[
        "(a)(b) Ann Able",
        "affiliation=Inst A",
        "affiliation=Inst B",
      ],
      &["Bob Baker", "affiliation=Inst A"],
      &["Cat Cole", "affiliation=Inst C"],
    ]),
  ]);
}

/// A mark-led line that answers the names' requests is affiliations, whatever its words: the risk rows, as on int169a
/// except where a name was lost there (the `\and` name-shaped row and the isotope row: Bob Baker a nameless creator's
/// affiliation). Rows: trailing marks over a name-shaped affiliation (`$^2$Zhipu AI`), mark-led names over one, a
/// `$^{*}$Corresponding author` legend line, an isotope line `$^{3}$He Facility`, trailing marks over an affiliation
/// row, a `\textsuperscript` comma row, mnras's comma row; and the residual a name led by an unanswered symbol leaves, a
/// nameless creator's affiliation counted by the reading's Warning (silent on int169a).
#[test]
fn batchmln1_marked_affiliations_stay_affiliations() {
  check(&[
    (
      repro!("article_mark_led_name_unanswered_is_counted"),
      (0, 1),
      &[&["affiliation=Ann Able", "affiliation=Inst A"]],
    ),
    (
      repro!("article_mark_led_names_over_a_name_shaped_affiliation"),
      (0, 0),
      &[&["Ann Able", "affiliation=Zhipu AI"], &[
        "Bob Baker",
        "affiliation=Deep Seek",
      ]],
    ),
    (
      repro!("article_mark_led_names_over_an_isotope_line"),
      (0, 0),
      &[
        &["Ann Able", "affiliation=Zhipu AI"],
        &["Bob Baker", "affiliation=Deep Seek"],
        &["affiliation=He Facility"],
      ],
    ),
    (
      repro!("article_symbol_led_legend_line_under_trailing_marks"),
      (0, 0),
      &[
        &[
          "Ann Able",
          "affiliation=Inst A",
          "note:thanks=∗ Corresponding author",
        ],
        &["Bob Baker", "affiliation=Inst A"],
      ],
    ),
    (
      repro!("article_textsuperscript_led_names_comma_row"),
      (0, 0),
      &[&["Ann Able", "affiliation=Inst A"], &[
        "Bob Baker",
        "affiliation=Inst B",
      ]],
    ),
    (
      repro!("article_trailing_marks_over_an_affiliation_row"),
      (0, 0),
      &[&["Ann Able", "affiliation=Inst A"], &[
        "Bob Baker",
        "affiliation=Inst B",
      ]],
    ),
    (
      repro!("article_trailing_marks_over_a_name_shaped_affiliation"),
      (0, 0),
      &[&["Ann Able", "affiliation=Max Planck Institut"], &[
        "Bob Baker",
        "affiliation=Zhipu AI",
      ]],
    ),
    (repro!("mnras_mark_led_names_comma_row"), (0, 0), &[
      &["Ann Able", "affiliation=Inst A"],
      &["Bob Baker", "affiliation=Inst B"],
    ]),
  ]);
}

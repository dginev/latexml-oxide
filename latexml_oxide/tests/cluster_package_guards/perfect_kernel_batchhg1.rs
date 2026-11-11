//! Guards for HG1: the guards the deletion pass (DEL1, `08d96cc760`) left hollow — tests that converted nothing,
//! discarded their conversion, never compared their expected values, or kept only the counts — asserting again. One row
//! per repro, each converted once in-process: its exact Error and Warning counts, the log lines its counts are, and
//! every creator whole (ids masked, whitespace between elements dropped, an ORCID logo's SVG body folded). A row may pin
//! fragments instead of its creators where part of its output is a RED of its own (the row says which). A hollow guard
//! whose output is wrong today is a `_red.tex` catalog repro instead (its header says why), not a row. A new module: the
//! rows come from ten batches (105a, 106s, 106t, 108a, 108b's review, 64zd, 77n, 78e, 97a, z14), none with an
//! in-process repro table that takes a class file beside its document.

use std::borrow::Cow;

use latexml::util::test::{convert_files_with, error_count, warning_count};
use regex::Regex;

use super::perfect_kernel_batchz4::creators_xml;

/// A repro of the catalog (`tools/perfect_kernel/repros/<path>.tex`), its name the row's.
macro_rules! repro {
  ($path:literal) => {
    (
      $path,
      Cow::Borrowed(include_str!(concat!(
        "../../../tools/perfect_kernel/repros/",
        $path,
        ".tex"
      ))),
    )
  };
}

/// A class file of the catalog, shipped beside the row's document under `name`.
macro_rules! file {
  ($name:literal, $path:literal) => {
    (
      $name,
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
    )
  };
}

/// What a row's creators must be: every creator whole, in order, or fragments some creator holds (where the rest of the
/// output is a RED of its own).
pub(super) enum Creators<'a> {
  Whole(&'a [&'a str]),
  Hold(&'a [&'a str]),
}

/// A row: the repro (name, source), the files beside it, its (Error, Warning) counts, the log lines it must hold, and
/// its creators.
pub(super) struct Row<'a> {
  pub(super) repro:    (&'a str, Cow<'a, str>),
  pub(super) files:    &'a [(&'a str, &'a str)],
  pub(super) counts:   (usize, usize),
  pub(super) lines:    &'a [&'a str],
  pub(super) creators: Creators<'a>,
}

/// The creators of `xml`, each whole: ids masked, whitespace between elements dropped, an SVG's body folded.
pub(super) fn whole_creators(xml: &str) -> Vec<String> {
  let between = Regex::new(r">\s+<").expect("regex");
  let svg = Regex::new(r"(?s)<svg:svg\b[^>]*>.*?</svg:svg>").expect("regex");
  creators_xml(xml)
    .iter()
    .map(|c| {
      svg
        .replace_all(&between.replace_all(c, "><"), "<svg:svg/>")
        .into_owned()
    })
    .collect()
}

/// Each row converted once: its counts, its log lines, its creators. Every row's failures collected, then one panic;
/// a row's found creators are printed as the literals the row would hold.
#[track_caller]
pub(super) fn assert_rows(rows: &[Row]) {
  let mut failures = Vec::new();
  for Row {
    repro: (name, tex),
    files,
    counts,
    lines,
    creators,
  } in rows
  {
    let (log, xml) = convert_files_with(tex, files, Some("ar5iv.sty"));
    let found_counts = (error_count(&log), warning_count(&log));
    if found_counts != *counts {
      failures.push(format!(
        "{name}: counts {found_counts:?}, want {counts:?}\n{log}"
      ));
    }
    for line in *lines {
      if !log.contains(line) {
        failures.push(format!("{name}: the log lacks {line:?}\n{log}"));
      }
    }
    let found = whole_creators(&xml);
    let wrong = match creators {
      Creators::Whole(expected) => found != *expected,
      Creators::Hold(fragments) => !fragments
        .iter()
        .all(|fragment| found.iter().any(|creator| creator.contains(fragment))),
    };
    if wrong {
      let literals: Vec<String> = found.iter().map(|c| format!("r#\"{c}\"#,")).collect();
      failures.push(format!("{name}: creators\n{}", literals.join("\n")));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// The restored rows, each as pdflatex prints it (a row's source and the guard it replaces in brackets):
/// - amsart's author reader keeps the lines a macro inside `\author` holds, the affiliation's `\footnotesize` lost and
///   counted (one Warning) — the content the 106s guard counted as lost is kept (2007.07526; 106s
///   `a_binding_losing_author_lines_is_counted`);
/// - xkeyval's own plumbing under a raw class's `\ead[label=e1,mark]{…}` is no dropped text: clean, the name whole (the
///   106s splitter residual gone; 2101.10506; 106t `a_bindings_own_plumbing_is_no_dropped_text`);
/// - a raw class's stores below the net's floor: the PACS (or MSC) code and the received date each a note line of the
///   author, clean (europhys `\Pacs` cond-mat9812194, ourlematema `\MSC` 1909.06690, tADR2e `\received` 2312.02510;
///   105a `a_code_or_date_line_below_the_floor_is_a_note_line`). Fragments only: the e-mail line is a note too, its own
///   RED (`net_shaped_store_email_line_is_a_contact_red`, OD #159);
/// - a mark holding an ORCID link stays on the name with its link, clean (2205.05709; 108a
///   `a_lost_marks_warning_parts_the_orcid`: no mark is lost, so no Warning names one);
/// - one name in a `\parbox` with its note, an affiliation line under it: one creator (77n
///   `parbox_single_author_stays_whole`);
/// - an article block of `\thanks`-noted name lines and affiliation lines: each line opens its author (64zd
///   `author_note_name_line_opens_next_author`). The list's separators (the trailing commas, "and") are not kept in
///   `before=` — the separator residual, RED in `author_marked_list_keeps_its_printed_separators_red`;
/// - revtex marked names with a lowercase particle (`B.~in der Baker`) and a collaboration line: a list, each name its
///   numbered address and the collaboration (97a `marked_list_with_an_unknown_name_shape_is_a_list`); amsart marked
///   names beside "the XYZ Team": a list and its collaboration (97a
///   `marked_names_beside_a_team_are_a_list_and_its_collaboration`);
/// - webofc's `\inst` form: each author its institute, the e-mail Ann Able's contact, no empty thanks note (2301.07438,
///   1411.7792, 2409.01693; 78e `webofc_authors_take_their_institutes`);
/// - amsart `\author{C. Vignat, Universit\'e d'Orsay}`: the name and its institution (1009.2396; z14
///   `an_elided_particle_is_a_surnames_only_after_an_initial`);
/// - a raw class's `\address` storing by a conditional into two slots (`\ifnum\addrn=1 \gdef\@addri{#1}\else…`), or into a
///   scratch name its title page prints (`\gdef\tmp{#1}`): every stored line Ann Able's address, nothing lost (rev108b2
///   findings 2 and 3, synthetic; the classes ST1 found never committed, now written by `filecontents*`).
#[test]
fn hollow_guards_restored_read_as_printed() {
  let net = repro!("sectioning-frontmatter/net_shaped_store_below_the_floor_is_kept");
  let msc = net.1.replace("\\Pacs{98.80.Cq}", "\\Pacs{05C12}");
  assert_ne!(msc, net.1);
  assert_rows(&[
    Row {
      repro:    repro!("sectioning-frontmatter/check_binding_author_lines_lost_are_counted"),
      files:    &[],
      counts:   (0, 1),
      lines:    &[
        "writes \"Lunar University Faculty of Craters\" as its plain text, its markup lost: a macro's span",
      ],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Lunar University Faculty of Craters</contact><contact name="email: " role="email"><text font="typewriter" fontsize="80%">ann@lunar.org</text></contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/check_binding_plumbing_is_no_dropped_text"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname></creator>"#,
      ]),
    },
    Row {
      repro:    net,
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"<note role="note">98.80.Cq</note><note role="note">3 March 2021</note></creator>"#,
      ]),
    },
    Row {
      repro:    (
        "net_shaped_store_below_the_floor_is_kept (an MSC code)",
        Cow::Owned(msc),
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"<note role="note">05C12</note><note role="note">3 March 2021</note></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_lost_mark_message_parts_the_orcid"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<Math mode="inline" tex="{}^{1\lx@orcidlink{0000-0001-8812-0565}{\lx@orcidlogo}}" text="^1 * []"><XMath><XMApp role="FLOATSUPERSCRIPT" scriptpos="1"><XMApp><XMTok meaning="times" role="MULOP">⁢</XMTok><XMTok fontsize="70%" meaning="1" role="NUMBER">1</XMTok><XMText><ref class="ltx_orcid" font="italic" fontsize="70%" href="https://orcid.org/0000-0001-8812-0565" title="ORCID 0000-0001-8812-0565"><svg:svg/></ref></XMText></XMApp></XMApp></XMath></Math></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_parbox_single_author_stays"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">ann@uni.edu</note><contact name="Affiliation: " role="affiliation">Univ A, City</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_note_name_line_opens_next_author"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">E-mail: ann@x.fi</note><contact name="Affiliation: " role="affiliation">Univ X, Helsinki, Finland</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">E-mail: bob@y.uk</note><contact name="Affiliation: " role="affiliation">Univ Y, Liverpool, UK</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Cole</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">E-mail: cy@x.fi</note><contact name="Affiliation: " role="affiliation">Univ X, Helsinki, Finland.</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_marked_list_lowercase_particle_merges"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>A. Able</personname><contact role="collaboration">(XYZ Collaboration)</contact><contact name="Affiliation: " role="affiliation"> Institute of One</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>B. in der Baker</personname><contact role="collaboration">(XYZ Collaboration)</contact><contact name="Affiliation: " role="affiliation"> University of Two</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>C. Cole</personname><contact role="collaboration">(XYZ Collaboration)</contact><contact name="Affiliation: " role="affiliation"> Institute of One</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "sectioning-frontmatter/author_marked_names_beside_a_team_are_a_reported_merge"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>A. Able<sup><text font="italic">1</text></sup></personname><contact role="collaboration">the XYZ Team</contact><contact name="Address: " role="address"><sup><text font="italic">1</text></sup> Institute of One</contact></creator>"#,
        r#"<creator before=" and " role="author"><personname>B. in der Baker<sup><text font="italic">2</text></sup></personname><contact role="collaboration">the XYZ Team</contact><contact name="Address: " role="address"><sup><text font="italic">2</text></sup> University of Two</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/webofc/webofc_authors_take_their_institutes"),
      files:    &[file!("webofc.cls", "class-bindings/webofc/webofc.cls")],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Email: " role="email">ann@x.org</contact><contact name="Affiliation: " role="affiliation">Univ A</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation">Univ B</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/z14r2_elided_particle_only_after_an_initial"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>C. Vignat</personname><contact name="Affiliation: " role="affiliation">Université d’Orsay</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/check_store_switch_slots_supersede"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Address: " role="address">Lunar Institute</contact><contact name="Address: " role="address">Gravel Road</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/check_scratch_name_is_the_store"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Address: " role="address">Lunar Institute</contact></creator>"#,
      ]),
    },
  ]);
}

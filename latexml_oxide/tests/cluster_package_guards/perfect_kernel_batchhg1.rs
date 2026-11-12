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

/// The restored rows, each as pdflatex prints it (a row's source and the guard it replaces in brackets); a part that is
/// not is pinned as today and names its RED:
/// - amsart's author reader keeps the lines a macro inside `\author` holds, the affiliation's `\footnotesize` and its line
///   break kept since AUT1 (the macro read in its place; one counted Warning before) — the content the 106s guard counted
///   as lost is kept (2007.07526; 106s `a_binding_losing_author_lines_is_counted`);
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
///   numbered address and the collaboration, the same with the list in one group (`{\bf …}`) (97a
///   `marked_list_with_an_unknown_name_shape_is_a_list`). Both pin before="  " where the list prints ", " — the
///   separator residual, RED in `author_marked_list_keeps_its_printed_separators_red`;
/// - amsart marked names (97a's amsart case): the list, each name its mark and address; C. Cole's before=" and " where
///   the line prints ", and" is pinned, RED in `author_marked_list_amsart_comma_and_keeps_its_comma_red`;
/// - amsart marked names beside "the XYZ Team": a list and its collaboration (97a
///   `marked_names_beside_a_team_are_a_list_and_its_collaboration`). B. in der Baker's before=" and " — the "and"
///   printed before the team moved between the names — is pinned, RED in
///   `author_marked_names_beside_a_team_move_their_and_red`;
/// - kernel `\and` names with symbol marks (`$^{*}$`, `$^{\dagger}$`, `\textsuperscript{*}`; html_feedback#6637
///   construct 2, OD #52; was `06_cluster_frontmatter::frontmatter_symbol_superscript_mark`): four creators, each mark
///   a visible `<sup>`, none an affiliation. Fiona Foxtrot's text asterisk written as the math `∗` (pdflatex: CMR8
///   "*") is pinned, RED in `author_textsuperscript_asterisk_stays_the_text_asterisk_red`;
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
  let particle = repro!("sectioning-frontmatter/author_marked_list_lowercase_particle_merges");
  let grouped = particle.1.replace(
    r"\author{A.~Able$^{1}$, B.~in der Baker$^{2}$, C.~Cole$^{1}$\\",
    r"\author{{\bf A.~Able$^{1}$, B.~in der Baker$^{2}$, C.~Cole$^{1}$}\\",
  );
  assert_ne!(grouped, particle.1);
  assert_rows(&[
    Row {
      repro:    repro!("sectioning-frontmatter/check_binding_author_lines_lost_are_counted"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation"><text fontsize="80%">Lunar University<break/>Faculty of Craters</text></contact><contact name="email: " role="email"><text font="typewriter" fontsize="80%">ann@lunar.org</text></contact></creator>"#,
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
      repro:    particle,
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
      repro:    (
        "author_marked_list_lowercase_particle_merges (in one group)",
        Cow::Owned(grouped),
      ),
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
        "sectioning-frontmatter/author_marked_list_amsart_comma_and_keeps_its_comma_red"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>A. Able<sup><text font="italic">1</text></sup></personname><contact name="Address: " role="address"><sup><text font="italic">1</text></sup> Institute of One</contact></creator>"#,
        r#"<creator before=", " role="author"><personname>B. in der Baker<sup><text font="italic">2</text></sup></personname><contact name="Address: " role="address"><sup><text font="italic">2</text></sup> University of Two</contact></creator>"#,
        r#"<creator before=" and " role="author"><personname>C. Cole<sup><text font="italic">1</text></sup></personname><contact name="Address: " role="address"><sup><text font="italic">1</text></sup> Institute of One</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "sectioning-frontmatter/author_marked_names_beside_a_team_move_their_and_red"
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
    Row {
      repro:    (
        "frontmatter_symbol_superscript_mark",
        Cow::Borrowed(include_str!(
          "../cluster_regressions/frontmatter_symbol_superscript_mark.tex"
        )),
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Dana Delta<sup>∗</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Evan Echo<sup>†</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Fiona Foxtrot<sup>∗</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Gina Golf</personname></creator>"#,
      ]),
    },
  ]);
}

/// Marked author blocks the counts-only guards DEL1 left (its audit's misses), each as pdflatex prints it; a part that
/// is not is pinned as today, its comment naming the RED.
#[test]
fn marked_blocks_restored_read_as_printed() {
  // (P4c: the lists' printed marks shown in the pins below, as pdflatex prints them — P4S Q1)
  assert_rows(&[
    Row {
      repro:    repro!("sectioning-frontmatter/ieee_shared_block/starred_names_one_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann (Al) Able*</personname><contact name="Affiliation: " role="affiliation">University of X, City</contact><contact name="Email: " role="email">ann@x.edu</contact><note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks">*The first two authors contributed equally to this work.</note></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker*</personname><contact name="Affiliation: " role="affiliation">University of X, City</contact><contact name="Email: " role="email">bob@x.edu</contact><note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks">*The first two authors contributed equally to this work.</note></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Cole</personname><contact name="Affiliation: " role="affiliation">University of X, City</contact><contact name="Email: " role="email">cy@x.edu</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_bf_rows_with_blank_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">Core.</note></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Cat Cole</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Dan Dee</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Eve Eck</personname><contact name="Affiliation: " role="affiliation"><text font="bold">Univ A</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Fay Fox</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_unmarked_affiliation_after_marked_names"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup>1</sup></personname><contact name="Affiliation: " role="affiliation"> Department of Physics, University of Somewhere</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation"> Department of Physics, University of Somewhere</contact><contact name="Affiliation: " role="affiliation">Univ B</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_declaration_vspace_led_affiliation_line"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann</personname><contact name="Affiliation: " role="affiliation"><text fontsize="90%">Univ A</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob</personname><contact name="Affiliation: " role="affiliation"><text fontsize="90%">Univ B</text></contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/ptephy_name_marks_link_affiliations"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Univ A, Tokyo</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup>∗</sup></personname><contact name="Affiliation: " role="affiliation">Univ A, Tokyo</contact><contact name="Affiliation: " role="affiliation">Lab B, Kyoto</contact><contact name="Email: " role="email">bob@example.org</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Coe</personname><contact name="Affiliation: " role="affiliation">Lab B, Kyoto</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/ptephy_address_marks_of_its_own_set_aside"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Univ A, Tokyo </contact><contact name="Email: " role="email">ann@example.org</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation">Lab B, Kyoto</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/ptephy_address_without_affil"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        "<creator role=\"author\"><personname>Ann Able∗</personname><contact name=\"Affiliation: \" role=\"affiliation\">Univ A, Tokyo\n</contact><contact name=\"Email: \" role=\"email\">ann@example.org</contact></creator>",
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/ptephy_v1_affil_single_argument"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation"><sup class="ltx_markedasmath">1</sup>Univ A, Tokyo</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation"><sup class="ltx_markedasmath">2</sup>Lab B, Kyoto</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/ptephy_name_thanks_and_address_text"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup>∗</sup></personname><contact name="Affiliation: " role="affiliation">Univ A, Tokyo</contact><contact name="Email: " role="email">ann@example.org</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup>†</sup></personname><note class="ltx_note_frontmatter ltx_thanks_address" role="thanks">Present address: Lab C</note><contact name="Affiliation: " role="affiliation">Lab B, Kyoto</contact><contact name="Email: " role="email">bob@example.org</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/ptephy_glued_marks"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Univ A</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup>∗</sup></personname><contact name="Affiliation: " role="affiliation">Univ A</contact><contact name="Email: " role="email">bob@example.org</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Coe<sup>†</sup></personname><contact name="Affiliation: " role="affiliation">Lab B</contact><contact name="Email: " role="email">cy@example.org</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Di Dee</personname><contact name="Affiliation: " role="affiliation">Lab B</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/ptephy_dag_affil_email"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup>∗</sup></personname><contact name="Affiliation: " role="affiliation">Graduate School, Osaka Univ</contact><contact name="Email: " role="email">ann@example.org</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker†</personname><contact name="Affiliation: " role="affiliation">Graduate School, Osaka Univ</contact><contact name="Email: " role="email">bob@example.org</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/ptephy_names_without_separator"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact role="collaboration">(The X Collaboration)</contact><contact name="Affiliation: " role="affiliation">Univ A</contact><contact name="Affiliation: " role="affiliation">Lab B</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup>∗</sup></personname><contact role="collaboration">(The X Collaboration)</contact><contact name="Affiliation: " role="affiliation">Univ A</contact><contact name="Email: " role="email">bob@example.org</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Coe</personname><contact role="collaboration">(The X Collaboration)</contact><contact name="Affiliation: " role="affiliation">Lab B</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Di Dee</personname><contact role="collaboration">(The X Collaboration)</contact><contact name="Affiliation: " role="affiliation">Univ A</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_ptephy_unlabelled_affil_after_labelled"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann</personname><contact name="Affiliation: " role="affiliation">Univ X</contact><contact name="Affiliation: " role="affiliation">Univ Z</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob</personname><contact name="Affiliation: " role="affiliation">Univ Y</contact><contact name="Affiliation: " role="affiliation">Univ Z</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/aa_thanks_email_leaves_no_empty_note"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Email: " role="email"><text font="typewriter">ann@x.org</text></contact><contact name="Affiliation: " role="affiliation">Univ A</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation">Univ B</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/revtex_sub_lettered_mark_every_line_answered"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>A. One</personname><contact role="collaboration">(X Collaboration)</contact><contact name="Affiliation: " role="affiliation"><text font="italic"> Alpha University, Beijing</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>B. Two</personname><contact role="collaboration">(X Collaboration)</contact><contact name="Affiliation: " role="affiliation"><text font="italic"> Beta Institute, Rome</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic"> Also at Delta College, Oslo</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>C. Three</personname><contact role="collaboration">(X Collaboration)</contact><contact name="Affiliation: " role="affiliation"><text font="italic"> Gamma Lab, Paris</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>D. Four<sup>23A</sup></personname><contact role="collaboration">(X Collaboration)</contact><contact name="Affiliation: " role="affiliation"><text font="italic"> (A)Zeta Lab, Turin; (B)Eta Lab, Milan</text></contact></creator>"#,
      ]),
    },
    // A. One's printed "1" lost (RED `revtex_comma_before_mark_keeps_its_mark_red`) and B. Two's ", and" lost
    // (RED `author_marked_list_keeps_its_printed_separators_red`); the lines on the names their marks name.
    Row {
      repro:    repro!("class-bindings/revtex_unsplit_labelled_address_block_lands_once"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>A. One</personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">1</text></sup>Alpha University, Beijing</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>B. Two<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">2</text></sup>Beta Institute, Rome</contact></creator>"#,
      ]),
    },
    // (98e's `\affiliation` sibling in a `\parbox`, whose guard ran only at =0 after DEL1: each name its line; Bob
    // Baker's printed "and" lost, RED `author_marked_list_keeps_its_printed_separators_red`.)
    Row {
      repro:    repro!("class-bindings/revtex_parbox_affiliation_lines_land_once"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">1</text></sup>Univ A, Town</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">2</text></sup>Univ B, City</contact></creator>"#,
      ]),
    },
    // (P4b4a2: the box's `[t]` sibling — a line set wholly in a box is cut in its body, the box dropped, each piece alike)
    Row {
      repro:    repro!("class-bindings/revtex_parbox_t_affiliation_lines_cut_alike"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">1</text></sup>Univ A, Town</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">2</text></sup>Univ B, City</contact></creator>"#,
      ]),
    },
    // (P4b4a3, revP4b4a B2: the declarations a cut box body opens with, on every piece, 2608.21425; and a line's, unboxed)
    Row {
      repro:    repro!("class-bindings/revtex_parbox_affiliation_lines_keep_declarations"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic" fontsize="70%">1</text></sup><text fontsize="70%">Univ A, Town</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic" fontsize="70%">2</text></sup><text fontsize="70%">Univ B, City</text></contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/llncs_institute_macro_marks_line_declarations"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup>1</sup></personname><contact name="Affiliation: " role="affiliation"><sup><text fontsize="90%">1</text></sup><text fontsize="90%"> Univ A</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup>2</sup></personname><contact name="Affiliation: " role="affiliation"><sup><text fontsize="90%">2</text></sup><text fontsize="90%"> Univ B</text></contact></creator>"#,
      ]),
    },
    // (P4b4a4, revP4b4a2 B1: a cut piece carries what sets the font, size or colour, by meaning; a macro printing text
    // or a glyph before the first mark prints once, on the first piece — `\mydept`, `\ldots`, in a box too)
    Row {
      repro:    repro!("class-bindings/llncs_institute_macro_lead_printed_once"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation">Dept of Physics, <sup><text font="italic">1</text></sup>Univ A</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">2</text></sup>Univ B</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/llncs_institute_ldots_lead_printed_once"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation">…<sup><text font="italic">1</text></sup>Univ A</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">2</text></sup>Univ B</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/llncs_institute_parbox_centering_macro_lead_printed_once"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation">Dept of Physics, <sup><text font="italic">1</text></sup>Univ A</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">2</text></sup>Univ B</contact></creator>"#,
      ]),
    },
    // (P4b4f, int166a's P4b4a regressions (CL166): a line of addresses read by its tokens, 1506.02349; a name's glued
    // mark requests each entry it spells, 2409.11440, 2511.20671; a start at the `\quad` before its line starts it,
    // 2601.03607; a cut piece carries `\tt` (LaTeX's whole declaration set), the llncs list and 2508.08560's title page,
    // whose hidden marks are pinned as int165a had them, RED `raw_maketitle_names_spaced_marks_hidden_red`)
    Row {
      repro:    repro!("class-bindings/ieeetran_authorblocka_marked_email_list_read_whole"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann One<sup>*</sup></personname><contact name="Affiliation: " role="affiliation"><sup>*</sup>Univ A</contact><contact name="Emails: " role="email">ann_one@ntu.edu.sg</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Two<sup>†</sup></personname><contact name="Affiliation: " role="affiliation"><sup>†</sup><sup>‡</sup>Univ B</contact><contact name="Emails: " role="email">bob_two@sutd.edu.sg</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Three<sup>‡</sup></personname><contact name="Affiliation: " role="affiliation"><sup>†</sup><sup>‡</sup>Univ B</contact><contact name="Emails: " role="email">cy_three@sutd.edu.sg</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/acmart_country_glued_mark_requests_each_entry"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann One<sup><text font="italic">1</text></sup><sup>∗</sup></personname><contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_country"><sup>1</sup>A Univ</text></contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks"><text class="ltx_affiliation_country"><sup>∗</sup>Equal contributions</text></note></creator>"#,
        r#"<creator before=", " role="author"><personname>Bob Two<sup><text font="italic">12</text></sup></personname><contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_country"><sup>1</sup>A Univ</text></contact><contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_country"><sup>2</sup>B Lab</text></contact></creator>"#,
        r#"<creator before=" and " role="author"><personname>Cy Three<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><text class="ltx_affiliation_country"><sup>1</sup>A Univ</text></contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/ieeeconf_authorblocka_quad_parted_marked_notes"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann One<sup><text font="bold">*</text></sup></personname><contact name="Affiliation: " role="affiliation">Stanford University</contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks"><sup>*</sup>Equal contribution</note></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Two<sup><text font="bold">*</text></sup></personname><contact name="Affiliation: " role="affiliation">Stanford University</contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks"><sup>*</sup>Equal contribution</note></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Three</personname><contact name="Affiliation: " role="affiliation">Stanford University</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Di Four<sup><text font="bold">†</text></sup></personname><contact name="Affiliation: " role="affiliation">Stanford University</contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks"><sup>†</sup>Equal advising</note></creator>"#,
        r#"<creator before="  " role="author"><personname>Ed Five<sup><text font="bold">†</text></sup></personname><contact name="Affiliation: " role="affiliation">Stanford University</contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks"><sup>†</sup>Equal advising</note></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/llncs_institute_marks_tt_lead_carried_to_every_entry"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">1</text></sup><text font="typewriter">Univ Alpha</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">2</text></sup><text font="typewriter">Univ Beta</text></contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "sectioning-frontmatter/raw_maketitle_affiliation_tt_lead_carried_to_every_email"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>A. One</personname><contact name="Affiliation: " role="affiliation"> Some Dept, Some Univ</contact><contact name="Email: " role="email"><text font="typewriter" fontsize="90%">one@a.org</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>B. Two</personname><contact name="Affiliation: " role="affiliation"> Some Dept, Some Univ</contact><contact name="Email: " role="email"><text font="typewriter" fontsize="90%">two@b.org</text></contact></creator>"#,
      ]),
    },
    // (revP4b4a2 B2: the box's alignment on no inline piece, the first included; its size on every one, 2608.21425)
    Row {
      repro:    repro!("class-bindings/llncs_institute_parbox_centering_marks_inline"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup>1</sup></personname><contact name="Affiliation: " role="affiliation"><sup><text fontsize="70%">1</text></sup><text fontsize="70%"> Univ A</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup>2</sup></personname><contact name="Affiliation: " role="affiliation"><text fontsize="70%"><sup>2</sup> Univ B</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Cole<sup>3</sup></personname><contact name="Affiliation: " role="affiliation"><sup><text fontsize="70%">3</text></sup><text fontsize="70%"> Univ C</text></contact></creator>"#,
      ]),
    },
    // (revP4b4a2 B3: a start inside an environment the list opens leaves the list to its tokens, the base's reading;
    // 2506.05954's `\begin{footnotesize}` lines)
    Row {
      repro:    repro!("class-bindings/llncs_institute_minipage_one_line_marks_whole_list"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup>1</sup></personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup>2</sup></personname></creator>"#,
        r#"<creator role="author"><contact name="Affiliation: " role="affiliation"><inline-block class="ltx_minipage" vattach="middle" width="345.0pt"><p><sup>1</sup> Univ A, <sup>2</sup> Univ B</p></inline-block></contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/revtex_affiliation_footnotesize_env_lines"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">a</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">a</text></sup>Univ A, Town</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">b</text></sup></personname><contact name="Affiliation: " role="affiliation"><sup><text font="italic">b</text></sup>Univ B, City</contact></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_names_line_team_piece_is_a_collaboration"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Ann Able<sup><text font="italic">1</text></sup></personname><contact role="collaboration">the GAPS Team</contact><contact name="Affiliation: " role="affiliation"><sup>1</sup>Univ A</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Baker<sup><text font="italic">2</text></sup></personname><contact role="collaboration">the GAPS Team</contact><contact name="Affiliation: " role="affiliation"><sup>2</sup>Univ B</contact></creator>"#,
      ]),
    },
    // The names whole, no "[ETHZ]" text; the addresses on a nameless creator and the printed "and" lost are pinned,
    // RED in `espcrc_addressmark_key_red` itself.
    Row {
      repro:    repro!("sectioning-frontmatter/espcrc_addressmark_key_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Whole(&[
        r#"<creator role="author"><personname>Slavo Kratochvila</personname></creator>"#,
        r#"<creator before="  " role="author"><personname>Philippe de Forcrand</personname></creator>"#,
        r#"<creator role="author"><contact name="Address: " role="address">ETH Zürich</contact><contact name="Address: " role="address">CERN</contact></creator>"#,
      ]),
    },
  ]);
}

/// The RED repros whose deleted guard asserted their counts (92e's `clean_both_ways`, 97b's and 97d's `both_ways`,
/// 59's, 64zd's, 65a's, 65e's, 84a's and 78s's clean conversions; review revHG1 S1): each row pins the counts and the
/// parts of its output that are right, the rest the RED its repro's header names. Row by row:
/// - kernel `\thanks` whose ink is a command, a math token, a rule or frame, a constructor, a colour box or an empty
///   superscript (`kernel_thanks_*_red`, 92e): 0 E / 0 W, every name; the authors whose `\thanks` prints nothing whole
///   (no note), Bob Baker's "B" note kept. The other notes are dropped silently: the REDs;
/// - an undefined command in `\thanks` (`kernel_thanks_undefined_command_is_ink_red`): its one Error; the note and its
///   error marker dropped: the RED;
/// - the orphan mark (`author_block_orphan_mark_is_kept_red`, 59): the name; its printed "1" lost: the RED;
/// - 64zd's asterisk-marked revtex names, noted affiliation and other note command
///   (`revtex_asterisk_marked_authors_split_red`, `author_noted_affiliation_stays_affiliation_red`,
///   `author_other_note_command_stays_affiliation_red`): the names and lines kept; the marks and the line read as a name
///   (`Ohio State`): the REDs;
/// - 65a's ASCE grade (`asce_membership_grade_qualifies_the_name_red`) and 65e's symbol mark shown by its e-mail
///   (`symbol_mark_shown_by_its_email_red`): each creator's contacts; the grade in the name and the lost `∗`: the REDs;
/// - 84a's prefix notes (`author_prefix_notes_then_space_separate_names_red`): the names, Ann's own note; the notes one
///   author off: the RED;
/// - 97b's separators and e-mail piece (`author_marked_list_keeps_its_printed_separators_red`,
///   `author_marked_email_piece_is_no_person_red`): the names, marks and affiliations; the lost separators, the missing
///   merge report: the REDs;
/// - 97d's collaboration lines (`author_names_for_two_collaborations_is_one_contact_red`,
///   `author_collaboration_heading_group_covers_names_after_red`, `author_collaboration_heading_colon_outside_groups_red`,
///   `author_collaboration_broken_after_and_is_one_contact_red`, `author_collaboration_after_and_heads_its_own_group_red`,
///   `author_trailing_collaboration_group_covers_every_name_red`): the names and their institutes; each collaboration
///   split, a name, or lost: the REDs.
#[test]
fn reds_keep_their_counts_and_right_parts() {
  assert_rows(&[
    Row {
      repro:    repro!("class-bindings/kernel_thanks_command_text_is_ink_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"<personname>Cat Cole</personname>"#,
        r#"<personname>Dan Doe</personname>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/kernel_thanks_math_token_is_ink_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"<personname>Cat Cole</personname>"#,
        r#"role="author"><personname>Dan Doe</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/kernel_thanks_rule_and_frame_are_ink_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"<personname>Cat Cole</personname>"#,
        r#"<personname>Dan Doe</personname>"#,
        r#"role="author"><personname>Eve Ell</personname></creator>"#,
        r#"role="author"><personname>Fay Fox</personname></creator>"#,
        r#"role="author"><personname>Gil Gray</personname></creator>"#,
        r#"role="author"><personname>Ida Ivy</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/kernel_thanks_constructor_text_is_ink_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"role="author"><personname>Cat Cole</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/kernel_thanks_colorbox_is_ink_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"<personname>Cat Cole</personname>"#,
        r#"<personname>Dan Doe</personname>"#,
        r#"<personname>Eve Ell</personname>"#,
        r#"role="author"><personname>Fay Fox</personname></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/kernel_thanks_empty_superscript_reads_alike_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"<personname>Bob Baker"#,
        r#"role="thanks">B</note></creator>"#,
      ]),
    },
    Row {
      repro:    repro!("class-bindings/kernel_thanks_undefined_command_is_ink_red"),
      files:    &[],
      counts:   (1, 0),
      lines:    &[r#"Error:undefined:\undefinedcs"#],
      creators: Creators::Hold(&[r#"<personname>Ann Able</personname>"#]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_block_orphan_mark_is_kept_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>A. Author"#,
        r#">Affiliation one</contact>"#,
        r#">Affiliation two</contact>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/revtex_asterisk_marked_authors_split_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>S. F. Pereira"#,
        r#"<personname>Z. Y. Ou"#,
        r#"<personname>H. J. Kimble</personname>"#,
        r#"role="address">Norman Bridge Laboratory, Pasadena</contact>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_noted_affiliation_stays_affiliation_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"role="affiliation">Department of Physics</contact>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_other_note_command_stays_affiliation_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">x</note>"#,
        r#"<personname>Bob Baker</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">y</note>"#,
        r#"role="affiliation">Dept</contact>"#,
        r#"role="affiliation">Dept Y</contact>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/asce_membership_grade_qualifies_the_name_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Matthew R. Kuhn"#,
        r#"role="affiliation">Univ. of Portland, Portland, OR 97203.</contact>"#,
        r#"<personname>Second Author</personname>"#,
        r#"role="affiliation">Affiliation of the second author.</contact>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/symbol_mark_shown_by_its_email_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Greg Hunt"#,
        r#"role="affiliation">Univ A</contact>"#,
        r#"gh@x.edu</contact>"#,
        r#"<personname>Cody Ground</personname>"#,
        r#"role="affiliation">Univ B</contact>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_prefix_notes_then_space_separate_names_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">Ann Able is with Univ A.</note>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"<personname>Cat Cole</personname>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "sectioning-frontmatter/author_marked_list_keeps_its_printed_separators_red"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>J. P. B. C. de Melo<sup><text font="italic">a</text></sup></personname>"#,
        r#"<personname>H. W. L. Naus<sup><text font="italic">b</text></sup></personname>"#,
        r#"<personname>T. Frederico<sup><text font="italic">c</text></sup></personname>"#,
        // (P4c: the list's printed marks shown, as pdflatex prints them — P4S Q1)
        r#"role="affiliation"><sup><text font="italic">a</text></sup>Univ A</contact>"#,
        r#"role="affiliation"><sup><text font="italic">b</text></sup>Univ B</contact>"#,
        r#"role="affiliation"><sup><text font="italic">c</text></sup>Univ C</contact>"#,
      ]),
    },
    Row {
      repro:    repro!("sectioning-frontmatter/author_marked_email_piece_is_no_person_red"),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        // (P4c: the list's printed mark shown, as pdflatex prints it — P4S Q1)
        r#"role="affiliation"><sup><text font="italic">1</text></sup>Institute of One</contact>"#,
        r#"b.baker@two.edu"#,
      ]),
    },
    Row {
      repro:    repro!(
        "sectioning-frontmatter/author_names_for_two_collaborations_is_one_contact_red"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"role="affiliation">Univ A</contact>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"role="affiliation">Univ B</contact>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "sectioning-frontmatter/author_collaboration_heading_group_covers_names_after_red"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"role="affiliation">Univ A</contact>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"role="affiliation">Univ B</contact>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "sectioning-frontmatter/author_collaboration_heading_colon_outside_groups_red"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"role="affiliation">Univ A</contact>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"role="affiliation">Univ B</contact>"#,
        r#"Corresponding authors: Ann Able</note>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "sectioning-frontmatter/author_collaboration_broken_after_and_is_one_contact_red"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname><contact role="collaboration">on behalf of the XYZ Collaboration</contact>"#,
        r#"role="affiliation">Univ A</contact>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "sectioning-frontmatter/author_collaboration_after_and_heads_its_own_group_red"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname><contact role="collaboration">(ATLAS Collaboration)</contact>"#,
        r#"role="affiliation">Univ A</contact>"#,
        r#"<personname>Bob Baker</personname><contact role="collaboration">(ATLAS Collaboration)</contact>"#,
        r#"role="affiliation">Univ B</contact>"#,
        r#"<personname>Cid Cole</personname>"#,
        r#"<personname>Dan Dee</personname>"#,
        r#"role="affiliation">Univ C</contact>"#,
      ]),
    },
    Row {
      repro:    repro!(
        "sectioning-frontmatter/author_trailing_collaboration_group_covers_every_name_red"
      ),
      files:    &[],
      counts:   (0, 0),
      lines:    &[],
      creators: Creators::Hold(&[
        r#"<personname>Ann Able</personname>"#,
        r#"role="affiliation">Univ X</contact>"#,
        r#"<personname>Bob Baker</personname>"#,
        r#"role="affiliation">Univ Y</contact>"#,
      ]),
    },
  ]);
}

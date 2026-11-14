//! Guards for ST1: RED catalog repros whose output already matched their `expect` (triage RT1, `2026-11-11_RT1_report.md`
//! §STALE) on int159a, renamed without `_red`. One row per repro, each converted once in-process: its exact Error and
//! Warning counts, the log lines its counts are (each Warning a counted loss: markup the emitter wrote as plain text, an
//! address the title page prints that the output does not hold), and every creator whole (ids masked, whitespace between
//! elements dropped). Two tables by concern: the author block's names, lines and addresses; and a marked run's cut with
//! the shapes whose diagnostics are counted. A new module: the rows span seven earlier batches (108a, 107b, 107e, 106s,
//! z3, z14, the e-mail lists), none with an in-process repro table to extend (108a's and 107e's guards spawn the CLI per
//! repro).

use latexml::util::test::{error_count, warning_count};
use regex::Regex;

use super::perfect_kernel_batchz4::{creators_xml, run};

/// A sectioning-frontmatter repro and its name, for a row's failure message.
macro_rules! row {
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

/// A row: the repro (name, source), its (Error, Warning) counts, the log lines it must hold, and its creators whole.
type Row<'a> = (
  (&'a str, &'a str),
  (usize, usize),
  &'a [&'a str],
  &'a [&'a str],
);

/// Each row converted: its counts, its log lines, its creators whole. Every row's failures collected, then one panic.
#[track_caller]
fn assert_rows(rows: &[Row]) {
  let between = Regex::new(r">\s+<").expect("regex");
  let mut failures = Vec::new();
  for ((name, tex), counts, lines, expected) in rows {
    let (log, xml) = run(tex);
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
    let found: Vec<String> = creators_xml(&xml)
      .iter()
      .map(|c| between.replace_all(c, "><").into_owned())
      .collect();
    if found != *expected {
      failures.push(format!("{name}: creators\n{found:#?}"));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// The author block's names, lines and addresses as pdflatex prints them, each row clean:
/// - a `\thanks` on an authblk name answers the line's `$^*$` (`addr_line_authblk_thanks_requested_mark`: the address
///   Carl Dee's, named "Email: "; synthetic, rev106a2 q5);
/// - a `\footnote` on a line under a sole authblk author is the creator's note contact (2601.20363, 2605.13069);
/// - a thin `\hspace` inside a name and a `\quad` after a title part nothing (rev108a S1, synthetic);
/// - an elsarticle block's sub-lettered marks (`B.~Baker$^{18a}$`) leave the names, each name its number's and its
///   letter's entries (1202.2048, 2007.02542); the unrequested entries stay on the block's creator;
/// - a one-line `{…}@domain` list whose entries spell no author: the legend each starred author's note, the unspelled
///   addresses on the block's creator (html_feedback #7233);
/// - an address glued to a name by a comma (`Ann Able,ann@a.org`, revZ14r3) and an affiliation line's `Email:` label
///   (2002.06945, revZ14r2) leave the name and the affiliation whole.
#[test]
fn author_blocks_read_as_printed() {
  assert_rows(&[
    (
      row!("addr_line_authblk_thanks_requested_mark"),
      (0, 0),
      &[],
      &[
        r#"<creator role="author"><personname>Ann Bee</personname><contact name="Affiliation: " role="affiliation">Univ X</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Carl Dee</personname><note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks">Corresponding author.</note><contact name="Affiliation: " role="affiliation">Univ X</contact><contact name="Email: " role="email">wr@x.org</contact></creator>"#,
      ],
    ),
    (
      row!("authblk_footnote_line_under_sole_author"),
      (0, 0),
      &[],
      &[
        r#"<creator role="author"><personname>Mariia Drozdova</personname><contact name="Affiliation: " role="affiliation">University of Geneva</contact><contact name="Email: " role="email"><text font="typewriter">mariia.drozdova@unige.ch</text></contact><contact name="Note: " role="note">Code available at somewhere</contact></creator>"#,
      ],
    ),
    (row!("authblk_small_hspace_in_name"), (0, 0), &[], &[
      // (the 1pt `\hspace` written as a hair space)
      "<creator role=\"author\"><personname>J.\u{200a}Smith</personname></creator>",
    ]),
    (row!("authblk_title_suffix_one_name"), (0, 0), &[], &[
      r#"<creator role="author"><personname>Dr. Ann Able</personname></creator>"#,
    ]),
    (
      row!("author_sub_lettered_mark_shown_on_name"),
      (0, 0),
      &[],
      &[
        r#"<creator role="author"><personname><text fontsize="90%">A. Able</text></personname><contact role="collaboration"><text fontsize="90%">(X Collaboration)</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N1, City</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname><text fontsize="90%">B. Baker</text></personname><contact role="collaboration"><text fontsize="90%">(X Collaboration)</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N18, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> also at Inst Alpha, Moscow</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname><text fontsize="90%">C. Cole</text></personname><contact role="collaboration"><text fontsize="90%">(X Collaboration)</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N10, City</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname><text fontsize="90%">D. Dee</text></personname><contact role="collaboration"><text fontsize="90%">(X Collaboration)</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N36, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> on leave from Inst Beta, Kiev</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname><text fontsize="90%">Lei Li</text></personname><contact role="collaboration"><text fontsize="90%">(X Collaboration)</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N1, City</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname><text fontsize="90%">N. B. Li</text></personname><contact role="collaboration"><text fontsize="90%">(X Collaboration)</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N22, City</text></contact></creator>"#,
        r#"<creator before="  " role="author"><personname><text fontsize="90%">Q. J. Li</text></personname><contact role="collaboration"><text fontsize="90%">(X Collaboration)</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N1, City</text></contact></creator>"#,
        r#"<creator role="author"><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N2, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N3, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N4, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N5, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N6, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N7, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N8, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N9, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N11, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N12, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N13, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N14, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N15, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N16, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N17, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N19, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N20, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N21, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N23, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N24, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N25, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N26, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N27, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N28, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N29, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N30, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N31, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N32, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N33, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N34, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N35, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N37, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N38, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N39, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N40, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N41, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N42, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N43, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> Univ N44, City</text></contact><contact name="Affiliation: " role="affiliation"><text font="italic" fontsize="90%"> now at Inst Gamma, Nagoya</text></contact></creator>"#,
      ],
    ),
    (
      row!("email_list_unspelled_entries_legend_becomes_creator"),
      (0, 0),
      &[],
      &[
        r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">University of Alpha</contact><contact name="Email: " role="email"><text font="typewriter">ann.able@alpha.edu</text></contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks"><sup>∗</sup>Contributed equally</note></creator>"#,
        r#"<creator before="  " role="author"><personname>Bo Baker</personname><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks"><sup>∗</sup>Contributed equally</note><contact name="Affiliation: " role="affiliation">University of Alpha</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Coe</personname><contact name="Affiliation: " role="affiliation">University of Alpha</contact><contact name="Email: " role="email"><text font="typewriter">cy.coe@alpha.edu</text></contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks"><sup>∗</sup>Contributed equally</note></creator>"#,
        r#"<creator role="author"><contact name="Email: " role="email"><text font="typewriter">xq.zz@alpha.edu</text></contact><contact name="Email: " role="email"><text font="typewriter">dd.dunn@alpha.edu</text></contact><contact name="Email: " role="email"><text font="typewriter">ee.egan@alpha.edu</text></contact></creator>"#,
      ],
    ),
    (row!("plain_address_glued_by_a_comma"), (0, 0), &[], &[
      r#"<creator role="author"><personname>Ann Able</personname><contact name="Email: " role="email">ann@a.org</contact></creator>"#,
    ]),
    (row!("z14_affiliation_label_word_left"), (0, 0), &[], &[
      r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Univ A</contact><contact name="Email: " role="email">ann@a.org</contact></creator>"#,
      r#"<creator before="  " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation">Univ A</contact><contact name="Email: " role="email">bob@b.org</contact></creator>"#,
    ]),
  ]);
}

/// A marked run's entries cut as printed, and the shapes whose diagnostics count real damage:
/// - an entry in a font group or a `\textit` wrapper followed by a comma-joined address (`{\small $^1$Univ A}, Rome`,
///   `\textit{$^1$Univ A}, Rome`; rev107e2 finding 1): each entry on its author, the wrapper's font lost and counted (one
///   Warning per entry: "its markup lost: span no balanced run");
/// - a legend line under a wrapped run whose last entry ends in a nested group, and nested wrappers per entry (rev107e2
///   finding 2): each entry on its carriers with its font, the legend each carrier's note;
/// - a class shipped in the paper's directory (`jcls.cls` by filecontents): its label word "corresponding" an Info
///   (`class_text`), the document's lost address words the Warning (rev106s2 s4b; cpc-hepnp.cls in 0802.2579, 0912.1248,
///   1806.07548);
/// - an Error the content of a cut entry raises (`ifthen`'s undefined boolean in revtex's `\affiliation`) said once, as
///   pdflatex and Perl say it (revZ3 S4).
#[test]
fn cut_runs_and_counts_read_as_printed() {
  assert_rows(&[
    (
      row!("cut_run_group_entry_then_comma_address"),
      (0, 2),
      &[
        "writes \"Univ A, Rome\" as its plain text, its markup lost: span no balanced run",
        "writes \"Inst B, Paris\" as its plain text, its markup lost: span no balanced run",
      ],
      &[
        r#"<creator role="author"><personname>Ann Lee</personname><contact name="Affiliation: " role="affiliation">Univ A, Rome</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Ray</personname><contact name="Affiliation: " role="affiliation">Inst B, Paris</contact></creator>"#,
      ],
    ),
    (
      row!("cut_run_wrapped_entry_then_comma_address"),
      (0, 2),
      &[
        "writes \"Univ A, Rome\" as its plain text, its markup lost: span no balanced run",
        "writes \"Inst B, Paris\" as its plain text, its markup lost: span no balanced run",
      ],
      &[
        r#"<creator role="author"><personname>Ann Lee</personname><contact name="Affiliation: " role="affiliation">Univ A, Rome</contact></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Ray</personname><contact name="Affiliation: " role="affiliation">Inst B, Paris</contact></creator>"#,
      ],
    ),
    (
      row!("cut_run_legend_after_wrapped_entry_ending_in_a_group"),
      (0, 0),
      &[],
      &[
        r#"<creator role="author"><personname>Ann Lee</personname><contact name="Affiliation: " role="affiliation"><text font="bold">Univ A</text></contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks"><sup>∗</sup> Equal Contribution</note></creator>"#,
        r#"<creator before="  " role="author"><personname>Bob Ray</personname><contact name="Affiliation: " role="affiliation"><text font="bold">Univ B, <emph font="italic">Rome</emph></text></contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks"><sup>∗</sup> Equal Contribution</note></creator>"#,
        r#"<creator before="  " role="author"><personname>Cy Do</personname><contact name="Affiliation: " role="affiliation"><text font="bold">Univ B, <emph font="italic">Rome</emph></text></contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks"><sup>†</sup> Project Leader</note></creator>"#,
      ],
    ),
    (row!("cut_run_nested_wrapper_per_entry"), (0, 0), &[], &[
      r#"<creator role="author"><personname>Ann Lee</personname><contact name="Affiliation: " role="affiliation"><text font="bold italic">Univ A</text></contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks"><sup>∗</sup> Equal Contribution</note></creator>"#,
      r#"<creator before="  " role="author"><personname>Bob Ray</personname><contact name="Affiliation: " role="affiliation"><text font="bold italic">Univ B</text></contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="∗" role="thanks"><sup>∗</sup> Equal Contribution</note></creator>"#,
      r#"<creator before="  " role="author"><personname>Cy Do</personname><contact name="Affiliation: " role="affiliation"><text font="bold italic">Univ B</text></contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="†" role="thanks"><sup>†</sup> Project Leader</note></creator>"#,
    ]),
    (
      row!("check_shipped_class_text_is_not_the_authors"),
      (0, 1),
      &[
        "Info:frontmatter:class_text The title page prints 1 word(s) of the class's own text (a label, a placeholder at its \
         default) that the output does not hold: corresponding",
        "Warning:frontmatter:unclaimed The title page prints 2 word(s) of the document's that its output does not hold: \
         lunar org",
      ],
      &[r#"<creator role="author"><personname>Ann Able</personname></creator>"#],
    ),
    // (A known residual pinned with it, not a ruling: the mark's `<sup><text font="italic">1</text></sup>` — TeX sets a
    // math digit upright (revST1 NIT).)
    (
      row!("writer_second_digestion_says_an_error_twice"),
      (1, 0),
      &["Package ifthen Error: Boolean ob undefined"],
      &[
        r#"<creator role="author"><personname>Ann Able</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">note</note><contact name="Affiliation: " role="affiliation"><sup><text font="italic">1</text></sup>Univ X a</contact><contact name="Affiliation: " role="affiliation"><sup><text font="italic">2</text></sup>Univ Y</contact></creator>"#,
      ],
    ),
  ]);
}

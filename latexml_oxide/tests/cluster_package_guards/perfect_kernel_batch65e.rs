//! Red/green guards for batch 65e, the regressions run 338 found against run 336: plain `\matrix` under the l-aa
//! binding, crckapb's `\keywords` in an abstract, a blank line between the `\bibitem`s of a bibliography inside a list
//! item, a collcell macro handed the template's `\ignorespaces`, the OmniBus theorem set replacing an author's macro, a
//! refused `\author` replayed half-way, ptephy's affiliation list copied into every author, and hyperref's
//! letltxmacro. Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv with `files` beside it, which must give no error and `warnings` warnings.
fn convert_files_warned(tex: &str, files: &[(&str, &str)], warnings: usize) -> String {
  let (log, xml) = latexml::util::test::convert_files_with(tex, files, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), warnings, "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  xml
}

fn convert_clean(tex: &str) -> String { convert_files_warned(tex, &[], 0) }

/// Every `<tag>` element of `xml` (`tag` without attributes, `<pubnote>`), whole, in document order.
fn bare_elements(xml: &str, tag: &str) -> Vec<String> {
  let open = format!("<{tag}>");
  let close = format!("</{tag}>");
  xml
    .split(&open)
    .skip(1)
    .map(|rest| {
      format!(
        "{open}{}",
        &rest[..rest.find(&close).expect("closed element") + close.len()]
      )
    })
    .collect()
}

/// Every `<creator>` element of `xml`, whole, with the indentation between its tags removed.
fn creators(xml: &str) -> Vec<String> {
  xml
    .split("<creator ")
    .skip(1)
    .map(|rest| {
      let element = &rest[..rest.find("</creator>").expect("closed creator") + "</creator>".len()];
      let lines: Vec<&str> = element.lines().map(str::trim).collect();
      format!("<creator {}", lines.concat())
    })
    .collect()
}

/// l-aa.sty loads no amsmath, so its papers write plain TeX's `\matrix{…\cr}` (plain.tex:1093); the binding loads
/// the aa class, whose amsmath made `\matrix` the environment-only form, an alignment nothing closed (astro-ph9609122,
/// astro-ph9809009). aa's `\@currenvir` dispatch, as for `\pmatrix` and `\cases`, reads it as plain TeX's. Repro
/// class-bindings/laa_plain_matrix_in_display.
#[test]
fn laa_plain_matrix_is_a_matrix() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/laa_plain_matrix_in_display.tex"
  ));
  assert_eq!(xml.matches("<XMArray ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "XMArray",
    &[r#"xml:id="S0.E1.m1.3""#],
    r#"<XMArray xml:id="S0.E1.m1.3">
      <XMRow xml:id="S0.E1.m1.3.1">
        <XMCell align="center" xml:id="S0.E1.m1.3.1.1"><XMTok font="italic" role="UNKNOWN">a</XMTok></XMCell>
        <XMCell align="center" xml:id="S0.E1.m1.3.1.2"><XMTok font="italic" role="UNKNOWN">b</XMTok></XMCell>
      </XMRow>
      <XMRow xml:id="S0.E1.m1.3.2">
        <XMCell align="center" xml:id="S0.E1.m1.3.2.1"><XMTok font="italic" role="UNKNOWN">c</XMTok></XMCell>
        <XMCell align="center" xml:id="S0.E1.m1.3.2.2"><XMTok font="italic" role="UNKNOWN">d</XMTok></XMCell>
      </XMRow>
    </XMArray>"#,
  );
  // the display ends with its equation: the section after it is a section, its math inline
  assert!(xml.contains(r#"<Math mode="inline" tex="a_{1}""#), "{xml}");
}

/// crckapb.sty:389-395's `\keywords#1\end@keywords` reads to the delimiter its `\abstract#1\end#2` appends; with that
/// `\abstract` refused by the lock it read to the end of the file (astro-ph0003044, astro-ph9702104). It ends at the
/// abstract's `\end`, as the class's does, labelled as `\make@abstract` prints it. Repro
/// class-bindings/crckapb_keywords_in_abstract.
#[test]
fn crckapb_keywords_end_with_the_abstract() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/crckapb_keywords_in_abstract.tex"
  ));
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">Text.</p></abstract>"#,
  );
  assert_eq!(xml.matches("<keywords ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Key words:\u{a0}\">Sun: irradiance</keywords>",
  );
}

/// A sectioning unit in a list item opens a paragraph-bridged `ltx:inline-sectional-block` (OXIDIZED_DESIGN #189); a
/// blank line between the `\bibitem`s of a bibliography in it closed that paragraph — and the block, the section and
/// the bibliography with it — so every later entry was misplaced (1805.06896, 2304.12197). A `\par` there ends no
/// paragraph, as inside a list. Repro list-structure/bib_blank_lines_in_closed_list (since 65i the witnesses' list,
/// never closed, gives way to the section, OXIDIZED_DESIGN #542: repro list-structure/bib_blank_lines_in_open_list,
/// `perfect_kernel_batch65i::open_list_gives_way_to_the_section`).
#[test]
fn par_in_item_bibliography_keeps_the_block() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/bib_blank_lines_in_closed_list.tex"
  ));
  assert_eq!(xml.matches("<biblist").count(), 1, "{xml}");
  assert_eq!(xml.matches("<inline-sectional-block").count(), 1, "{xml}");
  assert_element(
    &xml,
    "biblist",
    &[],
    r#"<biblist>
      <bibitem key="a" xml:id="bib.bib1"><tags><tag>[1]</tag><tag role="refnum">1</tag></tags><bibblock> A.
</bibblock></bibitem>
      <bibitem key="b" xml:id="bib.bib2"><tags><tag>[2]</tag><tag role="refnum">2</tag></tags><bibblock> B.
</bibblock></bibitem>
      <bibitem key="c" xml:id="bib.bib3"><tags><tag>[3]</tag><tag role="refnum">3</tag></tags><bibblock> C.
</bibblock></bibitem>
      <bibitem key="d" xml:id="bib.bib4"><tags><tag>[4]</tag><tag role="refnum">4</tag></tags><bibblock> D.
</bibblock></bibitem>
    </biblist>"#,
  );
  // the bibliography stays in the block, in the item's paragraph
  let block = &xml
    [xml.find("<inline-sectional-block").unwrap()..xml.find("</inline-sectional-block>").unwrap()];
  assert!(block.contains("</bibliography>"), "{xml}");
  // …while a blank line between two paragraphs of the section in the block still ends the first
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/list-structure/bib_blank_lines_in_closed_list.tex"
  )
  .replace("Thanks.\n", "Thanks.\n\nMore thanks.\n");
  let xml = convert_clean(&tex);
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="Sx1.p1""#],
    r#"<para xml:id="Sx1.p1"><p xml:id="Sx1.p1.1">Thanks.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="Sx1.p2""#],
    r#"<para xml:id="Sx1.p2"><p xml:id="Sx1.p2.1">More thanks.</p></para>"#,
  );
}

/// collcell.sty:76 `\collectcell#1#2\ignorespaces`: the cell starts after the template's `\ignorespaces` and ends
/// before its `\unskip` (here `\lx@column@trimright`), so a macro testing the cell gets the cell (1912.00582,
/// 2604.02352: `\ifdim\ignorespaces 0.2pt<…`). Repro alignment-bindings/collcell_cell_after_template_ignorespaces.
#[test]
fn collcell_cell_starts_after_the_template_ignorespaces() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/alignment-bindings/collcell_cell_after_template_ignorespaces.tex"
  ));
  assert_element(
    &xml,
    "tr",
    &[],
    r#"<tr xml:id="p1.1.1">
      <td align="center" xml:id="p1.1.1.1">low</td>
      <td align="center" xml:id="p1.1.1.2">high</td>
    </tr>"#,
  );
}

/// The OmniBus theorem set declares each environment only where its name is free, as LaTeX's `\newtheorem` refuses a
/// defined one (latex.ltx:17174-17195 `\@ifdefinable`): an author's `\conj` stays the author's (1308.6566, 0811.3048;
/// cs0511055 `\fact`). One warning: the unknown class. Repro class-bindings/omnibus_theorem_set_keeps_author_macro.
#[test]
fn omnibus_theorem_set_keeps_author_macro() {
  let xml = convert_files_warned(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/omnibus_theorem_set_keeps_author_macro.tex"
    ),
    &[],
    1,
  );
  assert_eq!(xml.matches("<theorem ").count(), 1, "{xml}");
  assert!(
    xml.contains(
      r#"<Math mode="display" tex="x=\overline{y_{1}}+z_{2}" text="x = overline@(y _ 1) + z _ 2""#
    ),
    "{xml}"
  );
}

/// An OmniBus theorem stub fires once and steps aside for an environment declared since it was installed (tlp.cls:494
/// `\newenvironment{proof}`, a raw class read over OmniBus): `\begin` finds the stub before `\proof`, and the set,
/// refusing the declared name, ran it again for ever (cs0511055). One warning: the unknown class. Repro
/// class-bindings/omnibus_stub_yields_to_declared_environment.
#[test]
fn omnibus_stub_yields_to_declared_environment() {
  let xml = convert_files_warned(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/omnibus_stub_yields_to_declared_environment.tex"
    ),
    &[],
    1,
  );
  assert_eq!(xml.matches("<proof").count(), 0, "{xml}");
  assert_eq!(xml.matches("<theorem ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1"><text font="italic" xml:id="p1.1.1">Pf.</text>First.</p>"#,
  );
}

/// siamltex.cls:459-476: the theorems share one counter, numbered within the section unless `onethmnum`, with
/// `{example}`, `{remark}` and `{defthm}` among them; the binding declared `{theorem}` unnumbered by section and no
/// `{remark}`, which the OmniBus theorem set re-declared within the section before it declared only free names
/// (2312.13144). Repro class-bindings/siamltex_theorems_numbered_within_section.
#[test]
fn siamltex_theorems_numbered_within_section() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/siamltex_theorems_numbered_within_section.tex"
  );
  let tags = |xml: &str| -> Vec<String> {
    xml
      .split("<tag>")
      .skip(1)
      .map(|rest| rest[..rest.find("</tag>").unwrap()].to_string())
      // (the headings' own tags, not their titles' styled copies)
      .filter(|tag| !tag.contains('<'))
      .collect()
  };
  let xml = convert_clean(tex);
  assert_eq!(tags(&xml), vec!["1", "Remark 1.1", "Theorem 1.2"], "{xml}");
  assert_eq!(xml.matches("<proof ").count(), 1, "{xml}");
  let xml = convert_clean(&tex.replace(
    r"\documentclass{siamltex}",
    r"\documentclass[onethmnum]{siamltex}",
  ));
  assert_eq!(tags(&xml), vec!["1", "Remark 1", "Theorem 2"], "{xml}");
}

/// ptephy: a superscript symbol mark no affiliation answers (`$^{\dag}$`) stays shown on its name
/// (`show_unanswered_marks`, base_utilities.rs), and the lines after the author list's first `\\` are its legends,
/// document notes with their marks, not one more creator (2203.01270). Repro
/// class-bindings/ptephy_symbol_marks_and_legend_lines.
#[test]
fn ptephy_symbol_marks_and_legend_lines() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ptephy_symbol_marks_and_legend_lines.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\"><personname>A.\u{a0}One<sup xml:id=\"id1\">\u{2020}</sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University One, City One</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>B.\u{a0}Two<sup xml:id=\"id2\">\u{2217}</sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University Two, City Two</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>C.\u{a0}Three</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University Two, City Two</contact></creator>",
  ]);
  assert_eq!(bare_elements(&xml, "pubnote"), vec![
    "<pubnote><sup xml:id=\"id3\"><text font=\"italic\" xml:id=\"id3.1\">\u{2217}</text></sup>Deceased, August 2020.</pubnote>",
    "<pubnote><sup xml:id=\"id4\"><text font=\"italic\" xml:id=\"id4.1\">\u{2020}</text></sup>Deceased, April 2021.</pubnote>",
  ]);
}

/// The OmniBus theorem set numbers a name within its owner's counter only when that counter exists: `{cor}`'s owner
/// `thm`, refused for an author's `\newcommand{\thm}`, leaves `{cor}` a counter of its own, not an undefined
/// `\thethm`. One warning: the unknown class. Repro class-bindings/omnibus_dependent_theorem_without_owner_counter.
#[test]
fn omnibus_dependent_theorem_without_owner_counter() {
  let xml = convert_files_warned(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/omnibus_dependent_theorem_without_owner_counter.tex"
    ),
    &[],
    1,
  );
  assert_eq!(xml.matches("<theorem ").count(), 1, "{xml}");
  assert_element(
    &xml,
    "tags",
    &[],
    r#"<tags>
      <tag>Corollary 1</tag>
      <tag role="refnum">1</tag>
      <tag role="typerefnum">Corollary 1</tag>
    </tags>"#,
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">See <text font="bold" xml:id="p1.1.1">Thm</text>.</p>"#,
  );
}

/// ptephy: a list of names that wraps with `\\` keeps every line names (only lines each led by a footnote symbol
/// are legends). Repro class-bindings/ptephy_name_lines_after_break_stay_names.
#[test]
fn ptephy_name_lines_after_break_stay_names() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ptephy_name_lines_after_break_stay_names.tex"
  ));
  let affiliation = |name: &str, univ: &str, first: bool| {
    format!(
      "<creator {}role=\"author\"><personname>{name}</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{univ}</contact></creator>",
      if first {
        ""
      } else {
        "before=\"\u{2003}\u{2003}\" "
      }
    )
  };
  assert_eq!(creators(&xml), vec![
    affiliation("A.\u{a0}One", "Univ A", true),
    affiliation("B.\u{a0}Two", "Univ B", false),
    affiliation("C.\u{a0}Three", "Univ A", false),
    affiliation("D.\u{a0}Four", "Univ B", false),
  ]);
  assert!(bare_elements(&xml, "pubnote").is_empty(), "{xml}");
}

/// A symbol mark nothing answers is shown on every author carrying it; a contact or note the author holds for another
/// reason answers nothing (`show_unanswered_marks`). Repro sectioning-frontmatter/symbol_mark_shown_on_each_author.
#[test]
fn symbol_mark_shown_on_each_author() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/symbol_mark_shown_on_each_author.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\"><personname>Ann Able<sup xml:id=\"id1\">\u{2020}</sup></personname><note class=\"ltx_note_frontmatter ltx_thanks_correspondence\" role=\"thanks\" xml:id=\"id2\">Corresponding author.</note></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>Bob Baker<sup xml:id=\"id3\">\u{2020}</sup></personname></creator>",
  ]);
}

/// ptephy: a legend line led by `\textsuperscript{*}` is a legend too (`leads_with_footnote_symbol` sees past the
/// command and its braces). Repro class-bindings/ptephy_textsuperscript_legend_line.
#[test]
fn ptephy_textsuperscript_legend_line() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ptephy_textsuperscript_legend_line.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\"><personname>A.\u{a0}One</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>B.\u{a0}Two<sup xml:id=\"id1\">\u{2217}</sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
  ]);
  assert_eq!(bare_elements(&xml, "pubnote"), vec![
    "<pubnote><sup xml:id=\"id2\">*</sup>Deceased.</pubnote>"
  ]);
}

/// A symbol mark the author's own email shows as well (`$^{*}$gh@x.edu`) stays on the name too, as the PDF prints it
/// on both ("GREG HUNT1,∗", "∗ GH@X.EDU"; 1910.00401): only a mark the placed contact drops is answered. Repro sectioning-frontmatter/symbol_mark_shown_by_its_email.
#[test]
fn symbol_mark_shown_by_its_email() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/symbol_mark_shown_by_its_email.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\"><personname>Greg Hunt<sup xml:id=\"id1\">\u{2217}</sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact><contact name=\"Email:\u{a0}\" role=\"email\"><sup xml:id=\"id2\">*</sup>gh@x.edu</contact></creator>",
    "<creator before=\" and \" role=\"author\"><personname>Cody Ground</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>",
  ]);
  // the glyph follows the mark's mode, as pdflatex prints it: `\textsuperscript{1,*}` a text `*`, `$^{1,*}$` a math `∗`
  // (65e review round 6: ts, tm)
  let mark_mode = |mark: &str| {
    format!(
      "\\documentclass{{amsart}}\n\\begin{{document}}\n\\title{{T}}\n\\author{{Greg Hunt{mark}, Cody Ground\\textsuperscript{{2}}\\\\ \\textsuperscript{{1}}Univ A\\\\ \\textsuperscript{{2}}Univ B}}\n\\maketitle\nText.\n\\end{{document}}\n"
    )
  };
  let cody = "<creator before=\" and \" role=\"author\"><personname>Cody Ground</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact></creator>";
  assert_eq!(
    creators(&convert_clean(&mark_mode("\\textsuperscript{1,*}"))),
    vec![
      "<creator role=\"author\"><personname>Greg Hunt<sup xml:id=\"id1\">*</sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
      cody,
    ]
  );
  assert_eq!(creators(&convert_clean(&mark_mode("$^{1,*}$"))), vec![
    "<creator role=\"author\"><personname>Greg Hunt<sup xml:id=\"id1\">\u{2217}</sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact></creator>",
    cody,
  ]);
}

/// OSA: a `\text…` command before an address without a braced argument (`\email{\textasteriskcentered able@x.edu}`)
/// is the address's text, not a mark inside a font command (whose slice ran backwards: a panic). The address's `*` is
/// Perl parity (textcomp.sty.ltxml:128 maps `\textasteriskcentered` to ASCII `*`; pdflatex prints `∗`). Repro
/// sectioning-frontmatter/osa_email_text_command_mark_without_brace.
#[test]
fn osa_email_text_command_mark_without_brace() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/osa_email_text_command_mark_without_brace.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\"><personname>A. Able<sup xml:id=\"id1\">\u{2217}</sup></personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact><contact name=\"Email:\u{a0}\" role=\"email\">*able@x.edu</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>B. Baker</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact></creator>",
  ]);
}

const REPLAY_UNBALANCED_CLASS: &str =
  include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/replayunbal.cls");

/// A replay that closes a group of the document cannot be rolled back (its frames changed): its assignments stay, and
/// so do its diagnostics, with an Info saying so. Three Errors, pinned: the class's own, the closed group, the left-over
/// `}`. Repro sectioning-frontmatter/replayed_setter_unbalanced_keeps_its_diagnostics.
#[test]
fn replayed_setter_unbalanced_keeps_its_diagnostics() {
  let (log, xml) = latexml::util::test::convert_files_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/replayed_setter_unbalanced_keeps_its_diagnostics.tex"
    ),
    &[("replayunbal.cls", REPLAY_UNBALANCED_CLASS)],
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&log), 3, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert!(
    log.contains("The refused \\author's replay raised 1 error(s) and could not be undone"),
    "{log}"
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Count: 1.</p>"#,
  );
}

const REPLAY_ERROR_CLASS: &str =
  include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/replayerr.cls");
const REPLAY_PACKAGE: &str =
  include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/replaypkg.sty");

/// The same class setter without the error, which the replay runs: its `\global\advance` is kept.
const REPLAY_CLEAN_CLASS: &str = r"\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{replayerr}[2026/10/12 the same setter without the error]
\LoadClass{article}
\newcounter{authcount}
\renewcommand\author[1]{\global\advance\c@authcount by 1\relax}
";

/// A refused class setter whose replay raises an error is undone whole, its assignments with its diagnostics
/// (`AssignmentJournalGuard::rollback`; jsty3-author.sty:542's `\author`, half-run, left an author list its printer
/// looped over: 2607.24171); the same setter without the error is replayed and kept. Repro
/// sectioning-frontmatter/replayed_setter_error_rolls_back.
#[test]
fn replayed_setter_error_rolls_back() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/replayed_setter_error_rolls_back.tex"
  );
  let alice = vec![r#"<creator role="author"><personname>Alice</personname></creator>"#];
  let xml = convert_files_warned(tex, &[("replayerr.cls", REPLAY_ERROR_CLASS)], 0);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Count: 0.</p>"#,
  );
  assert_eq!(creators(&xml), alice);
  let xml = convert_files_warned(tex, &[("replayerr.cls", REPLAY_CLEAN_CLASS)], 0);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Count: 1.</p>"#,
  );
  assert_eq!(creators(&xml), alice);
}

/// A package's refused setter is replayed too, as pdflatex runs it: the replay is not restricted to class setters
/// (an erring one is rolled back instead; 2607.24171). Repro sectioning-frontmatter/package_setter_is_replayed.
#[test]
fn package_setter_is_replayed() {
  let xml = convert_files_warned(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/package_setter_is_replayed.tex"
    ),
    &[("replaypkg.sty", REPLAY_PACKAGE)],
    0,
  );
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Count: 1.</p>"#,
  );
  assert_eq!(creators(&xml), vec![
    r#"<creator role="author"><personname>Alice</personname></creator>"#
  ]);
}

/// ptephy (authblk-TI's `\affil`): one `\affil{…}` of `$^{n}$`-led lines under names marked `$^{n}$` is the list of
/// the affiliations those marks label, each the names' requesting it; 64qa gave the whole list to every name
/// (2203.01270: 1669 names, 83 MB of XML, 6.9 GB). Repro class-bindings/ptephy_affil_list_to_marked_names.
#[test]
fn ptephy_affil_list_goes_to_marked_names() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/ptephy_affil_list_to_marked_names.tex"
  ));
  assert_eq!(creators(&xml), vec![
    "<creator role=\"author\"><personname>A.\u{a0}One</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University One, City One</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>B.\u{a0}Two</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University Two, City Two</contact><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University Three, City Three</contact></creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\"><personname>C.\u{a0}Three</personname><contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University Three, City Three</contact></creator>",
  ]);
}

/// hyperref loaded letltxmacro until v7.01h (2023-12-19), so `\LetLtxMacro` with hyperref alone is valid in its
/// years' papers (2106.06828, 1911.11856); the binding serves every version. Repro loader/hyperref_loads_letltxmacro.
#[test]
fn hyperref_loads_letltxmacro() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/loader/hyperref_loads_letltxmacro.tex"
  ));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1"><emph font="italic" xml:id="p1.1.1">old</emph> new</p>"#,
  );
}

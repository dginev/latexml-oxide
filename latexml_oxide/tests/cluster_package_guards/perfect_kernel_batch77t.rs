//! Red/green guards for batch 77t, quality batch 3 (frontmatter content and marks; the quality inventory's ranks 7, 15,
//! 17, 59, 92, 96, 98, 110; r2 the rev77t review's findings): a `\thanks` affiliation list keeps its text's
//! superscripts; marked names over an unmarked institution line give it to each name of the row above it (a text `\dag`
//! mark makes a block marked), a names row under it more authors, a role or note line no such line; several
//! `{IEEEkeywords}` blocks are all the keywords; vmsta's name parts as its title page prints them; entics/entcs
//! `\thanksref` names its `\thanks[key]` note; a parenthesized math lead is the mark of its line; an llncs-style
//! class's numbered `\institute` entries go by the authors' numbers; an llncs `\\` line reading as an affiliation is
//! the affiliation of the names since the last line under names, across the inline `\and`. Each guard asserts no Error
//! or Warning, every name once in its own `ltx:creator` and its contacts on it — whole elements.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch77d::{assert_no_merged_names, creator, creators, squash},
};

/// A repro under `tools/perfect_kernel/repros/`, converted with the ar5iv preload: its squashed XML, the conversion
/// free of errors and warnings.
macro_rules! repro {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// A repro beside the files it ships (`(name, path under repros/)`), as [`repro`].
macro_rules! repro_with {
  ($path:literal, $( ($name:literal, $file:literal) ),+ ) => {{
    let (log, xml) = convert_files_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      &[ $( ($name, include_str!(concat!("../../../tools/perfect_kernel/repros/", $file))) ),+ ],
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// An affiliation contact.
fn affiliation(text: &str) -> String {
  format!(r#"<contact name="Affiliation: " role="affiliation">{text}</contact>"#)
}

/// A name and its footnote-symbol mark.
fn marked(name: &str, id: &str, mark: &str) -> String {
  format!(r#"{name}<sup xml:id="{id}">{mark}</sup>"#)
}

/// Rank 7: the thanks-abuse idiom's `$^{n}$`-led affiliation list in a `\thanks` keeps an ordinary superscript of its
/// text — "10 m²" — where each lead labels its affiliation (2606.00313). GREEN on the 77t base already (62zq had
/// dropped the "2"); guarded here.
#[test]
fn thanks_affiliation_list_keeps_superscripts_in_its_text() {
  let xml =
    repro!("sectioning-frontmatter/thanks_affiliation_list_keeps_superscripts_in_its_text.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(
      false,
      "Bob Baker",
      &affiliation(
        r#"Univ B. Supported by 10 m<sup class="ltx_markedasmath" xml:id="id1">2</sup> labs."#
      )
    ),
  ]);
}

/// Rank 15: names marked `$^{\dag}$`/`$^{\ddag}$` (a text `\dag` an affiliation mark, so the block is a marked one)
/// over an unmarked institution line: the marks are shown, nothing answering them, and the line is each name's, as the
/// line prints under all of them (user ruling R12 2026-10-20). Was "Cat ColeUniversity of Somewhere" one creator.
#[test]
fn dag_marked_names_over_a_shared_institution() {
  let xml = repro!("sectioning-frontmatter/author_dag_marked_names_over_a_shared_institution.tex");
  assert_no_merged_names(&xml);
  let univ = affiliation("University of Somewhere");
  assert_eq!(creators(&xml), [
    creator(true, &marked("Ann Able", "id1", "†"), &univ),
    creator(false, &marked("Bob Baker", "id2", "‡"), &univ),
    creator(false, "Cat Cole", &univ),
  ]);
}

/// Rank 15, the `\and` column: an unmarked institution line under the second column's name is that column's, as the
/// column prints it (`Ann Able$^{1}$ \and Bob Baker\\ University of Somewhere`). Was "Bob BakerUniversity of
/// Somewhere".
#[test]
fn unmarked_institution_line_under_a_column_is_its_names() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
\title{T}\author{Ann Able$^{1}$ \and Bob Baker\\ University of Somewhere}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert_eq!(creators(&xml), [
    creator(true, &marked("Ann Able", "id1", "1"), ""),
    creator(false, "Bob Baker", &affiliation("University of Somewhere")),
  ]);
}

/// Rank 17: each `{IEEEkeywords}` block is printed where it stands (IEEEtran.cls:5286-5292), so a paper's several
/// blocks are all its keywords. Was only the last, "Zeta." (Perl's `\lx@begin@keywords` clears the keywords before it).
#[test]
fn ieeekeywords_blocks_are_all_kept() {
  let xml = repro!("sectioning-frontmatter/ieeekeywords_blocks_are_all_kept.tex");
  assert_eq!(xml.matches("<keywords ").count(), 2, "{xml}");
  assert!(
    xml.contains(r#"<keywords name="Index Terms: ">Eps. </keywords> <keywords name="Index Terms: "> Zeta. </keywords>"#),
    "{xml}"
  );
}

/// Rank 59: the vmsta binding (latexml_contrib vmsta_cls.rs) reads the class raw and gives the author content its title
/// page's name parts (vmsta.cls:278-291 `\name@m@front`: `\@fnm~\@snm`, the initials dropped; 1508.02842, 1411.4499,
/// 1601.01109). Was "Yu.YuliyaMishura".
#[test]
fn vmsta_title_page_name_parts() {
  let xml = repro_with!(
    "class-bindings/vmsta/vmsta_title_page_name_parts.tex",
    ("vmsta.cls", "class-bindings/vmsta/vmsta.cls")
  );
  // (the `~` between the parts an NBSP, which the squashed XML writes as a space)
  assert_eq!(creators(&xml), [
    creator(true, "Yuliya Mishura", ""),
    creator(false, "Ivan Voronov", ""),
  ]);
}

/// Rank 92: the entics binding (latexml_contrib entics_cls.rs) reads the class raw with elsart's frontmatter interface
/// (esl.dtx's): `\thanksref{ALL}` requests the `\thanks[ALL]{…}` note, the `\address` is the author's (2212.08177;
/// entcs.cls 1812.00040). Was the creator "Ann AbleALL" and the address a body paragraph.
#[test]
fn entics_thanksref_names_its_note() {
  let xml = repro_with!(
    "class-bindings/entics/entics_thanksref_names_its_note.tex",
    ("entics.cls", "class-bindings/entics/entics.cls")
  );
  assert_eq!(creators(&xml), [creator(
    true,
    "Ann Able",
    r#"<contact name="Address: " role="address">University of Somewhere</contact> <contact name="Thanks: " role="thanks">Email: ann@x.org</contact>"#
  )]);
}

/// Rank 96: a line led by a parenthesized mark in math, `$(a)$Univ A`, is labelled by that mark when an author requests
/// it as a superscript (1011.0516). Was every line welded into the last name.
#[test]
fn parenthesized_math_marks_label_affiliations() {
  let xml = repro!("sectioning-frontmatter/author_parenthesized_math_marks_label_affiliations.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
  ]);
}

/// Rank 98: a class carrying llncs's `\institutename` numbers the `\and` entries of its `\institute` store 1, 2, which
/// the authors' numeric marks pick (preprint.cls, 2405.16615). Was each author given both.
#[test]
fn class_numbered_institutes_go_by_the_authors_marks() {
  let xml = repro_with!(
    "sectioning-frontmatter/class_numbered_institutes_go_by_the_authors_marks.tex",
    ("numinst.cls", "sectioning-frontmatter/numinst.cls")
  );
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
  ]);
}

/// Rank 98, the `\inst` marks the class defines (preprint.cls's `\def\inst#1{\unskip$^{#1}$}`; 2512.08621's
/// `\inst{1,2}` and an unbraced `\inst2`): numbers too. Was each author given both institutes, and with the numbers
/// alone their text ("1Univ A", 77t r1's A/B).
#[test]
fn class_numbered_institutes_go_by_inst_marks() {
  let (log, xml) = convert_files_with(
    r"\documentclass{numinst}
\begin{document}
\title{T}
\author{Ann Able\inst{1,2} and Bob Baker\inst2}
\institute{Univ A \and Univ B}
\maketitle
Text.
\end{document}",
    &[(
      "numinst.cls",
      include_str!("../../../tools/perfect_kernel/repros/sectioning-frontmatter/numinst.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{} {}", affiliation("Univ A"), affiliation("Univ B"))
    ),
    creator(false, "Bob Baker", &affiliation("Univ B")),
  ]);
}

/// Rank 15, the negative: a line a footnote symbol leads (`\textsuperscript{*}Department …`, 2604.02578's `*`-led line
/// under names marked `*†`, `†‡`, `*†‡`) is no unmarked institution line: not given to every name of the block (77t
/// r1's A/B had given it to Robert L. Goldstone, who carries no `*`); it goes on as before (on the last name).
#[test]
fn symbol_led_line_under_marked_names_is_not_every_names() {
  let (log, xml) = convert_with(
    r"\documentclass{article}
\begin{document}
\title{T}\author{Ann Able\textsuperscript{*†}, Bob Baker\textsuperscript{†‡} \& Cat Cole\textsuperscript{*†‡}\\
\textsuperscript{*}Department of Computer Science, Univ A\\
\textsuperscript{†}Program in Cognition, Univ A}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  // (the `†`-led line is every name's that shows †, all three: since the label keys are compared by their printed
  // text (92b, OXIDIZED_DESIGN_DIVERGENCES #919), the names' `*†` runs request † as the line's `†` sets it — it was on
  // the block's shared creator, awaiting that ruling; pdflatex prints † on all three names. Each name shows the marks
  // no line answered, the `*` line staying on the last name as before)
  let program = affiliation("Program in Cognition, Univ A");
  assert_eq!(creators(&xml), [
    creator(true, r#"Ann Able<sup xml:id="id1">*</sup>"#, &program),
    creator(false, r#"Bob Baker<sup xml:id="id2">‡</sup>"#, &program),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id3">*,‡</sup>"#,
      &format!(
        "{} {program}",
        affiliation(r#"<sup xml:id="id4">*</sup>Department of Computer Science, Univ A"#)
      )
    ),
  ]);
}

/// Rank 110: in llncs a `\\` of `\author` is a line break of the centred author list (llncs.cls:1019-1036), so a line
/// under a name reading as an affiliation is that name's (review probe rv63k/llncs1). Was a creator "University of
/// Somewhere" holding the e-mail. The unmarked `\institute`, llncs's one institute, is every author's, after the lines
/// under the names (85b, OXIDIZED_DESIGN_DIVERGENCES #901; had been the block's shared creator).
#[test]
fn llncs_author_line_break_affiliation() {
  let xml = repro!("sectioning-frontmatter/llncs_author_line_break_affiliation.tex");
  assert_eq!(creators(&xml), [creator(
    true,
    "Ann Able",
    &format!(
      r#"{} <contact name="E-mail " role="email"><text font="typewriter" xml:id="id1">ann@x.org</text></contact> {}"#,
      affiliation("University of Somewhere"),
      affiliation("Inst A")
    )
  ),]);
}

/// A minimal article author block, converted with the ar5iv preload: its squashed XML, the conversion free of errors
/// and warnings.
fn article_author(author: &str) -> String {
  let (log, xml) = convert_with(
    &format!(
      "\\documentclass{{article}}\n\\begin{{document}}\n\\title{{T}}\\author{{{author}}}\n\\maketitle\nText.\n\\end{{document}}"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  squash(&xml)
}

/// Rank 15 (rev77t finding 1): an unmarked institution line is the names row's directly above it, not every name of
/// the `\and` group made before it (review probe r1: 77t r1 gave University B to Ann Able too, `annotate=2`).
#[test]
fn dag_names_rows_each_over_their_institution() {
  let xml = repro!("sectioning-frontmatter/author_dag_names_rows_each_over_their_institution.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &affiliation("University A")
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &affiliation("University B")
    ),
  ]);
}

/// Rank 15 (rev77t finding 1), the numbered marks nothing answers (review probe r2): as the `\dag` rows.
#[test]
fn numbered_names_rows_each_over_their_institution() {
  let xml = article_author(r"Ann Able$^{1}$\\ University A\\ Bob Baker$^{2}$\\ University B");
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "1"),
      &affiliation("University A")
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "2"),
      &affiliation("University B")
    ),
  ]);
}

/// Rank 15 (rev77t finding 2): a names row under an unmarked institution line is more authors, each holding the line
/// under their own row (review probe r7: 77t r1 appended the row to the shared line, two authors lost).
#[test]
fn second_names_row_under_a_shared_institution() {
  let xml = repro!("sectioning-frontmatter/author_second_names_row_under_a_shared_institution.tex");
  assert_no_merged_names(&xml);
  let somewhere = affiliation("University of Somewhere");
  let other = affiliation("Other Institute");
  assert_eq!(creators(&xml), [
    creator(true, &marked("Ann Able", "id1", "†"), &somewhere),
    creator(false, &marked("Bob Baker", "id2", "‡"), &somewhere),
    creator(false, "Cat Cole", &other),
    creator(false, "Dan Dee", &other),
  ]);
}

/// Rank 96 (rev77t finding 5): an author's mark written in parentheses, `$^{(a)}$`, requests the `$(a)$`-led line
/// (review probe A2: 77t r1 gave both lines to both authors).
#[test]
fn parenthesized_superscript_marks_label_affiliations() {
  let xml =
    repro!("sectioning-frontmatter/author_parenthesized_superscript_marks_label_affiliations.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
  ]);
}

/// Rank 15 (rev77t finding 5): a line that reads as an affiliation by a role or contact word only — "research",
/// "Collaboration", "Professor" —, or that names the corresponding author, is no unmarked institution line: not given
/// to every author (review probes r4, r6, r8). It stays where the 77t base put it, welded into the last name (the
/// Perl-identical continuation, KNOWN_PERL_ERRORS #446); each author's marked line is theirs.
#[test]
fn role_and_note_lines_are_no_unmarked_institution() {
  for line in [
    "This research was supported by the NSF",
    "for the ATLAS Collaboration",
    "Corresponding author: Professor Ann Able",
  ] {
    let xml = article_author(&format!(
      r"Ann Able$^{{1}}$, Bob Baker$^{{2}}$\\ {line}\\ $^{{1}}$Univ A\\ $^{{2}}$Univ B"
    ));
    assert_eq!(
      creators(&xml),
      [
        creator(true, "Ann Able", &affiliation("Univ A")),
        creator(
          false,
          &format!("Bob Baker<break/>{line}"),
          &affiliation("Univ B")
        ),
      ],
      "{line}"
    );
  }
}

/// Rank 110 (rev77t finding 3): an llncs `\\` affiliation line is the names' since the last affiliation line, a names
/// row after an affiliation starting a new row (review probe L4: 77t r1 gave Other Institute to Ann Able and Bob
/// Baker too). The unmarked `\institute` is every author's (85b, #901; had been the block's shared creator).
#[test]
fn llncs_author_line_break_affiliation_rows() {
  let xml = repro!("sectioning-frontmatter/llncs_author_line_break_affiliation_rows.tex");
  let somewhere = format!(
    "{} {}",
    affiliation("University of Somewhere"),
    affiliation("Inst A")
  );
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &somewhere),
    creator(false, "Bob Baker", &somewhere),
    creator(
      false,
      "Cat Cole",
      &format!(
        "{} {}",
        affiliation("Other Institute"),
        affiliation("Inst A")
      )
    ),
  ]);
}

/// Rank 110 (rev77t finding 3), a row per name (review probe L2): University of Y is Bob Baker's only; the one
/// `\institute` every author's (85b, #901).
#[test]
fn llncs_author_line_break_affiliation_per_name() {
  let (log, xml) = convert_with(
    r"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\\ University of X\\ Bob Baker\\ University of Y}
\institute{Inst A}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {}",
        affiliation("University of X"),
        affiliation("Inst A")
      )
    ),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{} {}",
        affiliation("University of Y"),
        affiliation("Inst A")
      )
    ),
  ]);
}

/// Rank 110 (rev77t finding 4): llncs's `\and` is inline (llncs.cls:1013-1018, `\unskip,` or `\lastand` in
/// `\@maketitle`), so the names it joins share the row and the line under it (2604.11038's Simon Fraser University;
/// review probe L1: 77t r1 gave it to Bob Baker only). The one `\institute` is every author's (85b, #901).
#[test]
fn llncs_author_and_is_inline() {
  let xml = repro!("sectioning-frontmatter/llncs_author_and_is_inline.tex");
  let somewhere = format!(
    "{} {}",
    affiliation("University of Somewhere"),
    affiliation("Inst A")
  );
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &somewhere),
    creator(false, "Bob Baker", &somewhere),
  ]);
}

/// Rank 59 (rev77t finding 7): a `\fnm` with no `\snm` after it is printed at the author content's end, as vmsta.cls's
/// `\name@m@front` prints its stored `\@fnm` after the content.
#[test]
fn vmsta_first_name_without_surname() {
  let xml = repro_with!(
    "class-bindings/vmsta/vmsta_first_name_without_surname.tex",
    ("vmsta.cls", "class-bindings/vmsta/vmsta.cls")
  );
  assert_eq!(creators(&xml), [
    creator(true, "Yuliya Mishura", ""),
    creator(false, "VoronovIvan", ""),
  ]);
}

/// Rank 110 (rev77t findings 3, 4), an e-mail line under a name (review probe L3): it ends that name's row as an
/// affiliation line does, so the next names start a new one — University of Y is Bob Baker's only (printed under
/// "ann@x.org and Bob Baker"); each e-mail its name's; the one `\institute` every author's (85b, #901).
#[test]
fn llncs_author_e_mail_line_ends_a_row() {
  let (log, xml) = convert_with(
    r"\documentclass{llncs}
\begin{document}
\title{T}
\author{Ann Able\\ \email{ann@x.org} \and Bob Baker\\ University of Y\\ \email{bob@y.org}}
\institute{Inst A}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  let email = |id: &str, address: &str| {
    format!(
      r#"<contact name="E-mail " role="email"><text font="typewriter" xml:id="{id}">{address}</text></contact>"#
    )
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{} {}", email("id1", "ann@x.org"), affiliation("Inst A"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{} {} {}",
        affiliation("University of Y"),
        email("id2", "bob@y.org"),
        affiliation("Inst A")
      )
    ),
  ]);
}

/// A place or a note under a shared institution line is the institution's next line, not a names row (the rev77t2
/// re-review's finding 1, probes s1, s4: 77t r2 made creators "Hong Kong SAR", "Equal Contribution"): a names row under
/// such a line is more authors only over an institution line of its own (`second_names_row_under_a_shared_institution`).
#[test]
fn place_or_note_line_under_a_shared_institution_is_no_names_row() {
  for (xml, institution) in [
    (
      repro!("sectioning-frontmatter/author_place_line_under_a_shared_institution.tex"),
      "The Hong Kong University of Science and Technology<break/>Hong Kong SAR",
    ),
    (
      repro!("sectioning-frontmatter/author_note_line_under_a_shared_institution.tex"),
      "University of Somewhere<break/>Equal Contribution",
    ),
  ] {
    assert_no_merged_names(&xml);
    let shared = affiliation(institution);
    assert_eq!(creators(&xml), [
      creator(true, &marked("Ann Able", "id1", "†"), &shared),
      creator(false, &marked("Bob Baker", "id2", "‡"), &shared),
    ]);
  }
}

/// llncs prints `\and` inline in the names row (llncs.cls:1013-1018), so a piece after `\and` is names, never an
/// affiliation line: `Ann Able \and Bob Baker \and the ATLAS Collaboration` is three creators (the rev77t2 re-review's
/// finding 2, probe L6: 77t r2 gave the collaboration to Ann Able and Bob Baker as an affiliation). The one
/// `\institute` is every author's (85b, #901).
#[test]
fn llncs_author_and_piece_is_names() {
  let xml = repro!("sectioning-frontmatter/llncs_author_and_piece_is_names.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Inst A")),
    creator(false, "Bob Baker", &affiliation("Inst A")),
    creator(false, "the ATLAS Collaboration", &affiliation("Inst A")),
  ]);
}

/// An llncs author list with `\inst` marks keeps the split, its `\institute` entries picked by the marks (the rev77t2
/// re-review's finding 3, probe L11: 77t r2 ran the row rule on it and left Inst A and Inst B on no one). The line under
/// the names, "University of X", stays a creator of its own as on the base (SYNC residual).
#[test]
fn llncs_author_inst_marks_keep_the_split() {
  let xml = repro!("sectioning-frontmatter/llncs_author_inst_marks_keep_the_split.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Inst A")),
    creator(false, "Bob Baker", &affiliation("Inst B")),
    creator(false, "University of X", ""),
  ]);
}

/// Parenthesized marks in a list, `$^{(a,b)}$` and `$^{(a),(b)}$`, request each of their marks (the rev77t2
/// re-review's finding 5, probes q1, q2): Univ A on all three names, Univ B on Bob Baker and Cat Cole. A regression
/// guard for the per-mark parenthesis strip (an int44 draft gave Univ B to Ann Able).
#[test]
fn parenthesized_mark_lists_label_affiliations() {
  let xml = repro!("sectioning-frontmatter/author_parenthesized_mark_lists_label_affiliations.tex");
  assert_no_merged_names(&xml);
  let both = format!("{} {}", affiliation("Univ A"), affiliation("Univ B"));
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &both),
    creator(false, "Cat Cole", &both),
  ]);
}

/// entics.cls's own `\qed` (entics.cls:449, `$\Box$`, an open box) stays the class's: the binding takes elsart's
/// frontmatter interface only (int44 A/B: elsart_support_core's `\ltx@qed` "∎" had replaced it in every proof of
/// 2204.12376, 2504.08349, 1905.09181). Red (int44a): "By induction.∎".
#[test]
fn entics_qed_is_the_class_box() {
  let xml = repro_with!(
    "class-bindings/entics/entics_qed_is_the_class_box.tex",
    ("entics.cls", "class-bindings/entics/entics.cls")
  );
  assert!(!xml.contains('∎'), "{xml}");
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m1\""],
    r#"<Math mode="inline" tex="\Box" text="Box" xml:id="p1.m1"> <XMath xml:id="p1.m1.1"> <XMTok name="Box" role="UNKNOWN">□</XMTok> </XMath> </Math>"#,
  );
}

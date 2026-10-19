//! Red/green guards for batch 77n: the author-name splitter reads a names line held inside a wrapper (S4) and the note
//! commands among names (S5) as LaTeX prints them (`names_in_wrapper`, `notes_to_the_name_before`, `is_name_annotation`,
//! base_utilities.rs; OXIDIZED_DESIGN_DIVERGENCES #765). A wrapper whose content holds the separators — an `\emph`, a
//! bare or declaring group, `\uppercase`, a `\parbox` or `\mbox` (laid out, so dropped) — splits inside, each name
//! keeping the wrapper's styling, its trailing spacing, paragraph breaks, marks and notes the last name's; a note among
//! the names (`\thanks`, `\footnotetext`, `\corref`, a `{footnote}` environment, a document's own note macro, ecai's
//! `\institute`, an IEEE grade) is the name's before it and neither separates nor merges. Perl merges each of these
//! into one author (SplitTokens passes over groups, Base_Utility.pool.ltxml:152-165); int31a merged them with
//! `Error:frontmatter:merged_creators`. r2 (the r1 review): ecai splits only an `\and` list or `\author[…]`, authblk's
//! lists add to the authors before them and its consecutive `\affil`s go to the same authors, a layout box's styling
//! reaches every name, "others"/"et al." name no one. r3 (the r2 review): a mark-led `\affil` goes by its mark only when
//! a queued author carries it and it is no footnote symbol, else to the authors since the last `\affil`; every marked
//! authblk `\author` is a list, so every mark is a request; a line-ending comma before a continuation line is kept;
//! a glued `$^{1*}$` and a superscript inside a word are read as printed. Each guard asserts the three measures: no Error or Warning, every
//! name once in its own `ltx:creator`, its notes, marks and affiliations on it.
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch77d::{assert_no_merged_names, creator, creators, squash},
};

/// A `sectioning-frontmatter` repro's squashed XML, its conversion free of errors and warnings.
macro_rules! convert_repro {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
        $path
      )),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// A `\thanks` note as the frontmatter writes it.
fn thanks(class: &str, id: &str, text: &str) -> String {
  format!(
    r#"<note class="ltx_note_frontmatter ltx_thanks_{class}" role="thanks" xml:id="{id}">{text}</note>"#
  )
}

/// An affiliation contact.
fn affiliation(text: &str) -> String {
  format!(r#"<contact name="Affiliation: " role="affiliation">{text}</contact>"#)
}

/// An IEEE membership grade contact.
fn membership(grade: &str) -> String { format!(r#"<contact role="membership">{grade}</contact>"#) }

/// IEEEtran `\emph{A, B, and C}` with spacing, a paragraph break and two notes after it (1108.3768): three authors in
/// italics, the notes the last's (was one creator + `Error:frontmatter:merged_creators`).
#[test]
fn ieee_wrapped_names_split_with_their_notes_after() {
  let xml = convert_repro!("author_ieee_wrapped_name_lists.tex");
  assert_no_merged_names(&xml);
  let notes = [
    thanks("note", "id4", "Ann Able and Bob Baker are with Univ A."),
    thanks("note", "id5", "A preliminary version appeared in X."),
  ]
  .join(" ");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"<emph font="italic" xml:id="id1">Ann Able</emph>"#,
      ""
    ),
    creator(
      false,
      r#"<emph font="italic" xml:id="id2">Bob Baker</emph>"#,
      ""
    ),
    creator(
      false,
      r#"<emph font="italic" xml:id="id3">Cat Cole</emph>"#,
      &notes
    ),
  ]);
}

/// IEEEtran names in an outer group, each with its grade, the group broken by `\\` (2112.09769): four authors, each
/// grade its name's membership (was one creator, every grade dropped).
#[test]
fn ieee_group_names_keep_their_memberships() {
  let xml = convert_repro!("author_ieee_group_names_with_memberships.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &membership("Senior Member, IEEE")),
    creator(false, "Bob Baker", &membership("Member, IEEE")),
    creator(false, "Cat Cole", ""),
    creator(
      false,
      "Dan Dorn",
      &format!(
        "{} {}",
        membership("Fellow, IEEE"),
        thanks("note", "id1", "Ann Able is with Univ A.")
      )
    ),
  ]);
}

/// IEEEtran bold names in an outer group with the grade at its end (2603.10769): three authors, the grade and notes
/// Cat Cole's.
#[test]
fn ieee_group_bold_names_split() {
  let xml = convert_repro!("author_ieee_group_bold_names_membership.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", ""),
    creator(false, "Bob Baker", ""),
    creator(
      false,
      "Cat Cole",
      &[
        membership("Member, IEEE"),
        thanks("note", "id1", "Ann Able is with Univ A."),
        thanks("funding", "id2", "Funded by X."),
      ]
      .join(" ")
    ),
  ]);
}

/// ieeeaccess names in one `\uppercase{…}` with grades among them (2209.12464): five authors in capitals, each with
/// its mark; the grades their names' memberships, in the case TeX prints them.
#[test]
fn ieee_uppercase_names_split_with_grades() {
  let xml = convert_repro!("author_ieee_uppercase_names_memberships.tex");
  assert_no_merged_names(&xml);
  let marked =
    |name: &str, id: &str, mark: &str| format!(r#"{name}<sup xml:id="{id}">{mark}</sup>"#);
  assert_eq!(creators(&xml), [
    creator(true, &marked("ANN ABLE", "id1", "1"), ""),
    creator(false, &marked("BOB BAKER", "id2", "2"), ""),
    creator(
      false,
      &marked("CAT COLE", "id3", "2"),
      &membership("SENIOR, IEEE")
    ),
    creator(
      false,
      &marked("DAN DORN", "id4", "3"),
      &membership("SENIOR, IEEE")
    ),
    creator(
      false,
      &marked("EVE ESS", "id5", "2"),
      &membership("Fellow, IEEE")
    ),
  ]);
}

/// An `\IEEEauthorblockN` names group with a mark after it, `^` math-active as breqn sets it (2504.02633): five
/// authors, each with its own mark.
#[test]
fn ieee_blockn_group_with_math_active_marks_splits() {
  let xml = convert_repro!("author_ieee_blockn_group_marks_math_active.tex");
  assert_no_merged_names(&xml);
  let found = creators(&xml);
  assert_eq!(found[..4], [
    creator(true, r#"Ann Able<sup xml:id="id1">⋆</sup>"#, ""),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2"><text font="italic" xml:id="id2.1">♭</text></sup>"#,
      ""
    ),
    creator(false, r#"Cat López<sup xml:id="id3">♯</sup>"#, ""),
    creator(
      false,
      r#"Dan Dorn<sup xml:id="id4"><text font="italic" xml:id="id4.1">♭</text></sup>"#,
      ""
    ),
  ]);
  assert_eq!(found.len(), 5, "{xml}");
  assert_eq!(
    found[4],
    creator(
      false,
      concat!(
        r#"Eve Ess<Math mode="inline" tex="{}^{\dagger\,\star}" text="^list@(dagger, star)" xml:id="m3"> "#,
        r#"<XMath xml:id="m3.3"> <XMApp role="FLOATSUPERSCRIPT" scriptpos="1" xml:id="m3.3.1"> <XMDual xml:id="m3.3.1.1"> "#,
        r#"<XMApp xml:id="m3.3.1.1.1"> <XMTok meaning="list"/> <XMRef idref="m3.1" xml:id="m3.3.1.1.1.2"/> "#,
        r#"<XMRef idref="m3.2" xml:id="m3.3.1.1.1.3"/> </XMApp> <XMWrap xml:id="m3.3.1.1.2"> "#,
        r#"<XMTok fontsize="70%" name="dagger" role="MULOP" rpadding="1.7pt" xml:id="m3.1">†</XMTok> "#,
        "<XMTok role=\"PUNCT\">\u{2063}</XMTok> ",
        r#"<XMTok fontsize="70%" name="star" role="MULOP" xml:id="m3.2">⋆</XMTok> </XMWrap> </XMDual> </XMApp> </XMath> </Math>"#
      ),
      ""
    )
  );
}

/// mn2e's names in a `\parbox` with two notes among them, affiliation lines below (1205.5573): two authors, no parbox
/// around either, the notes Ann Able's, each with the affiliation its mark names.
#[test]
fn parbox_names_split_and_drop_the_box() {
  let xml = convert_repro!("author_parbox_names_with_notes.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &[
        thanks("note", "id1", "E-mail: ann@uni.edu"),
        thanks("note", "id2", "Fellow"),
        affiliation(" Univ A, City"),
      ]
      .join(" ")
    ),
    creator(false, "Bob Baker", &affiliation(" Univ B, Town")),
  ]);
}

/// spconf's `{\parbox[c]{\textwidth}{\centering A$^{1}$, B$^{2}$, C$^{1}$}}` (2504.03146): three authors, the box and
/// its centring dropped, each with the affiliation its mark names.
#[test]
fn centered_parbox_names_in_a_group_split() {
  let xml = convert_repro!("author_parbox_centered_names_in_a_group.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A, City")),
    creator(false, "Bob Baker", &affiliation("Univ B, Town")),
    creator(false, "Cat Cole", &affiliation("Univ A, City")),
  ]);
}

/// A document's own note macro (`\blackthanks{…}` → `\thanks`) among `{\rm …}`-grouped names (2506.03876): five
/// authors, the notes Ann Able's and Dan Dorn's.
#[test]
fn note_macro_among_names_is_the_names_note() {
  let xml = convert_repro!("author_note_macro_among_names.tex");
  assert_no_merged_names(&xml);
  let mark = |n: &str| {
    format!(
      r#"<note mark="{n}" role="footnotemark" xml:id="footnotex{n}"><tags> <tag>{n}</tag> <tag role="refnum">{n}</tag> <tag role="typerefnum">footnote {n}</tag> </tags></note>"#
    )
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {}",
        thanks("note", "id1", "Co-first authors."),
        affiliation("Univ A")
      )
    ),
    creator(
      false,
      &format!("Bob Baker{}", mark("1")),
      &affiliation("Univ B")
    ),
    creator(false, "Cat Cole", &affiliation("Univ C")),
    creator(
      false,
      "Dan Dorn",
      &format!(
        "{} {}",
        thanks("correspondence", "id2", "Co-corresponding authors."),
        affiliation("Univ A")
      )
    ),
    creator(
      false,
      &format!("Eve Ess{}", mark("2")),
      &affiliation("Univ B")
    ),
  ]);
}

/// A row of names ending in a wrapped comma (`\textbf{B}$^{x}$\textrm{,} \\`, 2512.18601) is continued by the next
/// row: five authors, the legend Ann Able's and Bob Baker's.
#[test]
fn wrapped_comma_ends_a_row_of_names() {
  let xml = convert_repro!("author_wrapped_comma_ends_a_row.tex");
  assert_no_merged_names(&xml);
  let legend = |id: &str| {
    affiliation(&format!(
      r#"Core Contributor<break/><text font="typewriter" xml:id="{id}">{{first}}.{{last}}@kensho.com</text>"#
    ))
  };
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &legend("id1")),
    creator(false, "Bob Baker", &legend("id2")),
    creator(false, "Cat Cole", ""),
    creator(false, "Dan Dorn", ""),
    creator(false, "Eve Ess", ""),
  ]);
}

/// The separator "and" opening the last name's box (`\mbox{and {Cat~C.} Cole $^{2}$}`, 2108.09054): three authors,
/// each with the affiliations its marks name.
#[test]
fn and_opening_a_box_separates_names() {
  let xml = convert_repro!("author_and_opens_a_box.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation(" Univ A, City")),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{} {}",
        affiliation(" Univ A, City"),
        affiliation(" Univ B, Town")
      )
    ),
    creator(
      false,
      r#"<text xml:id="id1">Cat C. Cole </text>"#,
      &affiliation(" Univ B, Town")
    ),
  ]);
}

/// elsarticle's `\corref{cor1}` after two names (1201.6291): two authors, the correspondence Bob Baker's.
#[test]
fn elsarticle_corref_is_the_last_names() {
  let xml = convert_repro!("author_elsarticle_corref_after_names.tex");
  assert_no_merged_names(&xml);
  let address = r#"<contact name="Address: " role="address">Univ A, City</contact>"#;
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", address),
    creator(
      false,
      "Bob Baker",
      &format!(
        r#"{address} <contact name="Corresponding author: " role="correspondent">Corresponding author: bob@uni.edu</contact>"#
      )
    ),
  ]);
}

/// A `{footnote}` environment after each name, the comma between them inside the first (1703.02032): two authors,
/// each with its own note.
#[test]
fn footnote_environment_is_its_names_note() {
  let xml = convert_repro!("author_footnote_environment_after_names.tex");
  assert_no_merged_names(&xml);
  let note = |text: &str| format!(r#"<contact name="Note: " role="note">{text}</contact>"#);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &note("Univ A, City")),
    creator(false, "Bob Baker", &note("Univ B, Town")),
  ]);
}

/// A `\footnotetext` among revtex4-1 names (2201.11098): three authors, the note and the mark "1a" Bob Baker's.
#[test]
fn revtex_footnotetext_among_names_is_the_names() {
  let xml = convert_repro!("author_revtex_footnotetext_among_names.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">1</sup>"#,
      &affiliation("Univ A, City")
    ),
    creator(
      false,
      r#"Bob Baker <note mark="0" role="footnotetext" xml:id="footnotex1">On leave from Univ C, City</note> <sup xml:id="id2"><text font="italic" xml:id="id2.1">1a</text></sup>"#,
      ""
    ),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id3">2</sup>"#,
      &affiliation("Univ B, Town")
    ),
  ]);
}

/// amsppt's `\footnotemark \footnotetext{…}` after the last name (2209.04044): three authors, the note Cat Cole's.
#[test]
fn amsppt_footnotetext_after_names_is_the_last_names() {
  let xml = convert_repro!("author_amsppt_footnotetext_after_names.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    r#"<creator role="author"> <personname>Ann Able</personname> </creator>"#,
    r#"<creator before=", " role="author"> <personname>Bob Baker</personname> </creator>"#,
    r#"<creator before=" and " role="author"> <personname>Cat Cole<note mark="1" role="footnote" xml:id="id1">E-mail: cat@uni.edu <text class="ltx_phantom" xml:id="id1.1">*****</text></note></personname> </creator>"#,
  ]);
}

/// ecai.cls 2020's `\author{A\institute{…} \and B$^1$}` (2008.02735): two authors, the `\institute` Ann Able's
/// affiliation.
#[test]
fn ecai_institute_note_and_and_split() {
  let xml = convert_repro!("author_ecai_institute_notes_and.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &affiliation("Univ A, City, email: {ann, bob}@uni.edu")
    ),
    creator(false, r#"Bob Baker<sup xml:id="id1">1</sup>"#, ""),
  ]);
}

/// ecai.cls 2020's names parted only by `\ `, a `\\` line of more names and no `\and` (1606.03244): one author as
/// written, holding the five names, the `\institute` its affiliation — not split as a per-author class's block, which
/// read the second line as an affiliation.
#[test]
fn ecai_space_parted_names_stay_one_author() {
  let xml = convert_repro!("author_ecai_space_parted_names_stay_one.tex");
  assert_eq!(creators(&xml), [creator(
    true,
    "Ann Able Bob Baker Cat Cole <break/>Dan Dorn Eve Ess",
    &affiliation("Univ X, City")
  )]);
}

/// authblk's unlabelled `\author{A$^{1}$\orcidlink{…} and B$^2$\orcidlink{…}}` (2405.16615): two authors, each with
/// its own ORCID link and mark. The mark is shown after the link, where pdflatex prints it before (RED
/// sectioning-frontmatter/author_mark_before_orcid_link).
#[test]
fn authblk_and_between_names_splits() {
  let xml = convert_repro!("author_authblk_and_between_names.tex");
  assert_no_merged_names(&xml);
  let logo = orcid_logo(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &format!(
        r#"Ann Able<ref class="ltx_orcid" href="https://orcid.org/0000-0003-3690-1890" title="ORCID 0000-0003-3690-1890">{logo}</ref><sup xml:id="id1">1</sup>"#
      ),
      ""
    ),
    creator(
      false,
      &format!(
        r#"Bob Baker<ref class="ltx_orcid" href="https://orcid.org/0000-0002-9991-8393" title="ORCID 0000-0002-9991-8393">{logo}</ref><sup xml:id="id2">2</sup>"#
      ),
      ""
    ),
  ]);
}

/// The ORCID logo, whole, as the document's first ORCID link holds it (`\lx@orcidlogo`), checked for its shape.
fn orcid_logo(xml: &str) -> &str {
  let start = xml.find("<svg:svg ").expect("an ORCID logo");
  let end = start + xml[start..].find("</svg:svg>").expect("closed logo") + "</svg:svg>".len();
  let logo = &xml[start..end];
  assert!(
    logo.starts_with(
      r#"<svg:svg class="ltx_orcidlogo" height="1em" version="1.1" viewBox="0 0 72 72" width="1em">"#
    ),
    "{logo}"
  );
  logo
}

/// REVTeX 3 letter-marked names with `\vskip 0.3in` after them (hep-ph0107182): three authors, each with its mark and
/// the address block under them.
#[test]
fn revtex3_glue_after_names_is_no_name() {
  let xml = convert_repro!("author_revtex3_letter_marks_glue_after.tex");
  assert_no_merged_names(&xml);
  let sup = |id: usize, mark: &str| {
    format!(r#"<sup xml:id="id{id}"><text font="italic" xml:id="id{id}.1">{mark}</text></sup>"#)
  };
  // (the address block, its four lines each with its mark, ids from `first`)
  let addresses = |first: usize| {
    let lines: Vec<String> = ["a", "b", "c", "d"]
      .iter()
      .enumerate()
      .map(|(k, mark)| format!("{}Univ {}", sup(first + k, mark), mark.to_uppercase()))
      .collect();
    format!(
      r#"<contact name="Address: " role="address">{}</contact>"#,
      lines.join("<break/>")
    )
  };
  assert_eq!(creators(&xml), [
    creator(true, &format!("Ann Able{}", sup(1, "ab")), &addresses(2)),
    creator(false, &format!("Bob Baker{}", sup(6, "c")), &addresses(7)),
    creator(false, &format!("Cat Cole{}", sup(11, "d")), &addresses(12)),
  ]);
}

/// Two names in one link whose text opens with a graphic (`\href{…}{\includegraphics[…]{orcid.pdf}\hspace{1mm}A, B}`,
/// 2312.07592): two authors, each in the link, the graphic with the first, the note the last's.
#[test]
fn link_wrapper_with_a_graphic_splits() {
  let xml = convert_repro!("author_link_wrapper_with_graphic.tex");
  assert_no_merged_names(&xml);
  let found = creators(&xml);
  assert_eq!(found.len(), 2, "{xml}");
  // (the graphic's `candidates` path is the TeX tree's mwe image, relative to the working directory)
  let (head, rest) = found[0].split_once(r#"candidates=""#).expect("a graphic");
  let (candidates, tail) = rest.split_once('"').unwrap();
  assert!(
    candidates.ends_with("/mwe/example-image.png"),
    "{candidates}"
  );
  assert_eq!(
    format!(r#"{head}candidates="…"{tail}"#),
    creator(
      true,
      r#"<ref class="ltx_href" href="https://uni.edu"><graphics candidates="…" graphic="example-image" options="scale=0.06" xml:id="g1"/> Ann Able</ref>"#,
      ""
    )
  );
  assert_eq!(
    found[1],
    creator(
      false,
      r#"<ref class="ltx_href" href="https://uni.edu">Bob Baker</ref>"#,
      &thanks(
        "contribution",
        "id1",
        "These two authors contributed equally."
      )
    )
  );
}

/// Negative guard: one wrapped name with its note, "and" inside names in and beside boxes, an accented name before
/// "and" in a wrapper, a macro printing its first argument and noting its second: eight authors, none split inside.
#[test]
fn wrapper_split_negatives_stay() {
  let xml = convert_repro!("author_wrapper_split_negatives.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &thanks("note", "id1", "Univ A, City")),
    creator(false, r#"<text xml:id="id2">Rand Anders</text>"#, ""),
    creator(
      false,
      r#"<emph font="italic" xml:id="id3">Ferdinand Ott</emph>"#,
      ""
    ),
    creator(false, r#"<text xml:id="id4">Andrea Sand</text>"#, ""),
    creator(false, "Zoë Able", ""),
    creator(false, "Bob Baker", ""),
    creator(false, "Cat Cole", &thanks("note", "id5", "Univ C, Town")),
    creator(false, "Dan Dorn", &thanks("note", "id6", "Univ D, Village")),
  ]);
}

/// Negative guard: one name in a `\parbox` stays one author in its box.
#[test]
fn parbox_single_author_stays_whole() {
  let xml = convert_repro!("author_parbox_single_author_stays.tex");
  assert_eq!(creators(&xml), [creator(
    true,
    r#"<inline-block class="ltx_parbox" vattach="middle" width="345.0pt" xml:id="id1"> <p align="center" xml:id="id1.1">Ann Able</p> </inline-block>"#,
    &format!(
      "{} {}",
      thanks("note", "id2", "ann@uni.edu"),
      affiliation("Univ A, City")
    )
  )]);
}

/// authblk's unlabelled `\author`s, each a name with a `\thanks` holding a comma, each with its `\affil` (r1 review probe
/// a1): two authors, each with its note and affiliation (int31a and 77n r1 counted the note's comma as a list separator
/// and replaced Ann Able, 0 errors).
#[test]
fn authblk_note_comma_keeps_earlier_authors() {
  let xml = convert_repro!("author_authblk_note_comma_keeps_earlier_authors.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &[
        thanks("note", "id1", "Dept. X, Univ Y"),
        affiliation("Univ X")
      ]
      .join(" ")
    ),
    creator(
      false,
      "Bob Baker",
      &[
        thanks("note", "id2", "Dept. Z, Univ W"),
        affiliation("Univ Z")
      ]
      .join(" ")
    ),
  ]);
}

/// authblk's unlabelled `\author{Bob Baker, Cat Cole}` after `\author{Ann Able}\affil{Univ X}` (r1 review probe a2):
/// three authors, Ann Able's affiliation Univ X, the others' Univ Z (int31a and 77n r1 replaced Ann Able, her
/// affiliation left on a nameless creator).
#[test]
fn authblk_comma_list_adds_to_earlier_authors() {
  let xml = convert_repro!("author_authblk_comma_list_after_author.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ X")),
    creator(false, "Bob Baker", &affiliation("Univ Z")),
    creator(false, "Cat Cole", &affiliation("Univ Z")),
  ]);
}

/// authblk's consecutive unlabelled `\affil`s after `\author{Ann Able and Bob Baker}` (r1 review probe o7): both
/// affiliations are both authors', as authblk prints them under the one block (77n r1 gave Ann Able only the first).
#[test]
fn authblk_consecutive_affils_share_their_authors() {
  let xml = convert_repro!("author_authblk_consecutive_affils_share_authors.tex");
  let both = [affiliation("Univ X"), affiliation("Univ Y")].join(" ");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &both),
    creator(false, "Bob Baker", &both),
  ]);
}

/// `\parbox{\textwidth}{\centering\large Ann Able, Bob Baker}` (r1 review finding 6): two authors without the box and
/// its `\centering`, both large (77n r1 kept `\large` on the first name only).
#[test]
fn layout_box_declarations_style_every_name() {
  let xml = convert_repro!("author_layout_box_styles_every_name.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"<text fontsize="120%" xml:id="id1">Ann Able</text>"#,
      ""
    ),
    creator(
      false,
      r#"<text fontsize="120%" xml:id="id2">Bob Baker</text>"#,
      ""
    ),
  ]);
}

/// "others" and "et al." closing a names list (r1 review finding 8): `Ann Able \emph{and others} \and Bob Baker, Cat
/// Cole, et al.` is three authors — no creator "others" or "et al" — the "and" before "others" kept in its wrapper,
/// italic as printed (r2 review finding 4: r2 moved it out as the separator, upright).
#[test]
fn others_and_et_al_name_no_one() {
  let xml = convert_repro!("author_others_et_al_is_no_name.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able <emph font="italic" xml:id="id1">and others</emph>"#,
      ""
    ),
    creator(false, "Bob Baker", ""),
    creator(false, "Cat Cole, et al.", ""),
  ]);
}

/// A contact of the `note` role (a `\footnote` inside an author), as the frontmatter writes it.
fn note_contact(text: &str) -> String {
  format!(r#"<contact name="Note: " role="note">{text}</contact>"#)
}

/// authblk's unlabelled `\author`s each with a `\footnote` holding a comma, each with its `\affil` (AIAA new-aiaa.cls
/// pattern: 2610.21324, 2008.05535, which kept 1 of 3 authors on int31a): three authors in order, Ann Able's note kept.
#[test]
fn authblk_footnote_comma_keeps_all_three_authors() {
  let xml = convert_repro!("author_authblk_footnote_comma_keeps_three.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &[note_contact("Student, Dept. A."), affiliation("Univ A")].join(" ")
    ),
    creator(
      false,
      "Bob Baker",
      &[note_contact("Professor, Dept. B."), affiliation("Univ B")].join(" ")
    ),
    creator(false, "Cat Cole", &affiliation("Univ C")),
  ]);
}

/// authblk's unlabelled `\author{Bob Baker \and Cat Cole}` after `\author{Ann Able}\affil{Univ A}`: three authors, Ann
/// Able kept (int31a, 77n r1 and Perl replaced her: authblk.sty.ltxml:52 → `\lx@add@authors`).
#[test]
fn authblk_and_list_adds_to_earlier_authors() {
  let xml = convert_repro!("author_authblk_and_list_after_author.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
    creator(false, "Cat Cole", &affiliation("Univ B")),
  ]);
}

/// authblk's unlabelled `\affil{$^1$Univ X}\affil{$^2$Univ Y}` after `\author{Ann Able$^{1}$ and Bob Baker$^2$}` (r1
/// review probe o4): each affiliation goes to the author showing its mark, the marks answered.
#[test]
fn authblk_marked_affils_pick_their_authors() {
  let xml = convert_repro!("author_authblk_marked_affils_pick_their_authors.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ X")),
    creator(false, "Bob Baker", &affiliation("Univ Y")),
  ]);
}

/// An affiliation with a mark, as the frontmatter writes it when the mark is shown (`<sup>` of id `id`).
fn marked_affiliation(id: &str, mark: &str, text: &str) -> String {
  affiliation(&format!(r#"<sup xml:id="{id}">{mark}</sup>{text}"#))
}

/// authblk's `\affil{$^*$Equal contribution}` after mark-picked `\affil{$^1$Univ X}`, the authors `Ann$^{1*}$` and
/// `Bob$^{1*}$` (r2 review probe c1): a footnote symbol is no affiliation mark, so the line keeps authblk's run — both
/// authors', as printed under the block — and no creator without a name holds it (77n r2). The `1*` is affiliation 1's
/// mark and a star shown (`split_glued_symbol`).
#[test]
fn authblk_symbol_mark_affil_runs() {
  let xml = convert_repro!("author_authblk_symbol_mark_affil_runs.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann<sup xml:id="id1">*</sup>"#,
      &[
        marked_affiliation("id2", "∗", "Equal contribution"),
        affiliation("Univ X")
      ]
      .join(" ")
    ),
    creator(
      false,
      r#"Bob<sup xml:id="id3">*</sup>"#,
      &[
        marked_affiliation("id4", "∗", "Equal contribution"),
        affiliation("Univ X")
      ]
      .join(" ")
    ),
  ]);
}

/// authblk's `\author{Ann$^1$}\affil{$^1$X}\author{Bob}\affil{$^2$Y}` (r2 review probe b6): Y's mark no author carries,
/// so it is the run's — Bob's alone, the author since the last `\affil` (authblk.sty:115-116); 77n r2 gave Bob nothing
/// and Y a creator without a name.
#[test]
fn authblk_affil_after_unmarked_author() {
  let xml = convert_repro!("author_authblk_affil_after_unmarked_author.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann", &affiliation("X")),
    creator(false, "Bob", &marked_affiliation("id1", "2", "Y")),
  ]);
}

/// authblk's `\affil{$^3$Z}` whose mark no author carries (a typo; r2 review probe b5): Z is the run's, under both
/// authors as printed, and no creator without a name holds it. (Z precedes X on Ann: a mark-picked affiliation reaches
/// its authors after the run's.)
#[test]
fn authblk_typo_mark_affil_runs() {
  let xml = convert_repro!("author_authblk_typo_mark_affil_runs.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann",
      &[marked_affiliation("id1", "3", "Z"), affiliation("X")].join(" ")
    ),
    creator(
      false,
      r#"Bob<sup xml:id="id2">2</sup>"#,
      &marked_affiliation("id3", "3", "Z")
    ),
  ]);
}

/// authblk's `\author{Ann Able$^{1,2}$}\author{Bob Baker$^{2}$}` (r2 review probe a4): both marked names are author
/// lists, so every mark is a request on every path — Bob Baker gets Univ Y (77n r2: none, his mark requesting nothing).
#[test]
fn authblk_mark_list_then_single_mark() {
  let xml = convert_repro!("author_authblk_mark_list_then_single_mark.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &[affiliation("Univ X"), affiliation("Univ Y")].join(" ")
    ),
    creator(false, "Bob Baker", &affiliation("Univ Y")),
  ]);
}

/// authblk's `\author{Ann$^{1}$}\author{Bob$^{1,2}$}` (r2 review probe c3): Ann gets Univ X (77n r2: none).
#[test]
fn authblk_single_mark_then_mark_list() {
  let xml = convert_repro!("author_authblk_single_mark_then_mark_list.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann", &affiliation("Univ X")),
    creator(
      false,
      "Bob",
      &[affiliation("Univ X"), affiliation("Univ Y")].join(" ")
    ),
  ]);
}

/// authblk's `\author{Ann\textsuperscript{1}, Bob\textsuperscript{2}}\author{Cat\textsuperscript{2}}` (r2 review probe
/// c4): Cat gets Univ Y (77n r2: none).
#[test]
fn authblk_comma_names_then_marked_name() {
  let xml = convert_repro!("author_authblk_comma_names_then_marked_name.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann", &affiliation("Univ X")),
    creator(false, "Bob", &affiliation("Univ Y")),
    creator(false, "Cat", &affiliation("Univ Y")),
  ]);
}

/// authblk's `\author{Ann$^{1}$\thanks{Corresponding, ann@x.org}}\author{Bob$^{2}$}\author{Cat$^{1}$}` (r2 review probe
/// c5): Cat gets Univ X (77n r2: none), Ann keeps her note.
#[test]
fn authblk_note_comma_then_marked_name() {
  let xml = convert_repro!("author_authblk_note_comma_then_marked_name.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann",
      &[
        thanks("correspondence", "id1", "Corresponding, ann@x.org"),
        affiliation("Univ X")
      ]
      .join(" ")
    ),
    creator(false, "Bob", &affiliation("Univ Y")),
    creator(false, "Cat", &affiliation("Univ X")),
  ]);
}

/// A mark-led authblk `\affil{$^1$Univ X, \\ Dept Y}` and `\affil{$^2$Univ Z,}` (r2 review finding 3; 1507.08653's
/// "Université Laval,\\ pavillon …"): the comma before the line continuing it and the one closing the affiliation are
/// kept, as pdflatex prints them; only a comma before the next marked institution is a separator
/// (`06_cluster_frontmatter::frontmatter_superscript_affil_comma`).
#[test]
fn authblk_marked_affil_keeps_its_comma() {
  let xml = convert_repro!("author_authblk_marked_affil_keeps_its_comma.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann", &affiliation("Univ X,<break/>Dept Y")),
    creator(false, "Bob", &affiliation("Univ Z,")),
  ]);
}

/// A superscript inside a name, a letter on each side (`Craig M$^{\mathrm{c}}$Rae`, 2505.09041), beside a marked
/// authblk author: the name's spelling, shown in place, not an affiliation request. A name whose superscript text
/// follows is no list (`author_marks_end_name`, 77n r4), so it stays whole as int32a writes it; read as a mark on the
/// list path it left the name, "Craig MRae" (`rewrite_in_word_superscripts` keeps it there, 77n r3). The italic "c" is
/// int32a's font residual, not pdflatex's: `$^{\mathrm{c}}$` prints an upright c (77n r4 review).
#[test]
fn authblk_superscript_inside_a_name_stays() {
  let xml = convert_repro!("author_authblk_superscript_inside_a_name.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Craig M<sup xml:id="id1"><text font="italic" xml:id="id1.1">c</text></sup>Rae"#,
      &[
        thanks("note", "id2", "Electronic address: craig@x.org"),
        affiliation("Univ Y")
      ]
      .join(" ")
    ),
    creator(
      false,
      "Ann Able",
      &[affiliation("Univ Y"), affiliation("Univ X")].join(" ")
    ),
  ]);
}

/// A name's mark glued to the "and" before the next name (`Bipasha Kundu$^{1}$and Cristian Linte$^{1,2}$`, 2604.27101;
/// the r3 review): a mark, not the name's spelling — each author answered by the legends of its marks (77n r3 kept the
/// "1" in Ann Able's name, and her affiliation was lost).
#[test]
fn mark_glued_to_and_is_a_mark() {
  let xml = convert_repro!("author_mark_glued_to_and.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Ann Able is with Univ X.")),
    creator(
      false,
      "Bob Baker",
      &[
        affiliation("Ann Able is with Univ X."),
        affiliation("Bob Baker is with Univ Y.")
      ]
      .join(" ")
    ),
  ]);
}

/// A name's mark list glued to the "and" before the next name (`Hiroshi Ito$^{1,3}$and Tetsuya Ogata`, 2605.00471; the
/// r3 review): a list of marks is never spelling — Bob Baker keeps both legends (77n r3: "Bob Baker<sup>1,2</sup>", only
/// the second).
#[test]
fn mark_list_glued_to_and_is_marks() {
  let xml = convert_repro!("author_mark_list_glued_to_and.tex");
  let univ_x = affiliation("Ann Able, Bob Baker and Cat Cole are with Univ X.");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_x),
    creator(
      false,
      "Bob Baker",
      &[univ_x.clone(), affiliation("Bob Baker is with Univ Y.")].join(" ")
    ),
    creator(false, "Cat Cole", &univ_x),
  ]);
}

/// A digit superscript inside a word of an affiliation line (`$^1$NLP$^2$CT Lab`, 2603.20851, 2605.30274; `I$^2$R`,
/// 2603.04869): the r4 limits on `rewrite_in_word_superscripts` (no mark list, no "and" after it) leave it the word's.
#[test]
fn digit_superscript_inside_a_word_stays() {
  let xml = convert_repro!("author_superscript_inside_a_word_digit.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &affiliation(
        r#"NLP<sup xml:id="id1"><Math mode="inline" tex="2" text="2" xml:id="m4a"> <XMath xml:id="m4a.1"> <XMTok meaning="2" role="NUMBER">2</XMTok> </XMath> </Math></sup>CT Lab, Univ X"#
      )
    ),
    creator(false, "Bob Baker", &affiliation("Univ Y")),
  ]);
}

/// An unlabelled authblk `\affil` inside `\author` (r3 review probe k1, under titling): it expands at the flush, no
/// author queued, so its run is uncapped (`annotate=run`) — Ann Author keeps Some University (77n r3: `annotate=run:0`,
/// attached to no one). pdflatex prints no affiliation here (titling's `\theauthor` loses it); Perl's `annotate=new`
/// keeps it.
#[test]
fn authblk_affil_inside_author_reaches_it() {
  let xml = convert_repro!("author_authblk_affil_inside_author.tex");
  assert_eq!(creators(&xml), [creator(
    true,
    "Ann Author",
    &affiliation("Some University")
  )]);
}

/// authblk `\affil{1}`s inside a comma list of names, then a top-level `\affil{Lab X\\ Univ Y}` (2605.01689's DSSL
/// block; the r3 review): every affiliation reaches a creator, as int32a and Perl's `annotate=new` give them (77n r3:
/// none).
#[test]
fn authblk_affil_after_affils_in_author_reaches_them() {
  let xml = convert_repro!("author_authblk_affil_after_affils_in_author.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("1")),
    creator(
      false,
      "Bob Baker",
      &[affiliation("1"), affiliation("Lab X<break/>Univ Y")].join(" ")
    ),
  ]);
}

/// authblk `\affil{$^{1,2}$Univ X}` after `\author{Ann$^{1}$}\author{Bob$^{2}$}` (r3 review probes w1, v8): a combined
/// mark keeps authblk's run, so both authors get Univ X (77n r3: Ann alone, the marked path labelling it by its first
/// mark).
#[test]
fn authblk_combined_mark_affil_runs() {
  let xml = convert_repro!("author_authblk_combined_mark_affil.tex");
  let univ_x = |id: &str| {
    affiliation(&format!(
      r#"<sup xml:id="{id}"><text font="italic" xml:id="{id}.1">1,2</text></sup>Univ X"#
    ))
  };
  assert_eq!(creators(&xml), [
    creator(true, r#"Ann<sup xml:id="id1">1</sup>"#, &univ_x("id2")),
    creator(false, r#"Bob<sup xml:id="id3">2</sup>"#, &univ_x("id4")),
  ]);
}

/// authblk `\author`s whose marks a line or text follows (`Ann Able$^{1}$\\ ann@x.org`, `Bob Baker$^{2}$ (bob@x.org)`,
/// `Cat Cole$^{3}$ \orcidlink{…}`; r3 review probes m2, m7, m8, m9) stay single names (`author_marks_end_name`): each
/// mark where it is written (77n r3 read them as lists, dropping Ann Able's "1" before the `\\` and moving Bob Baker's
/// after his address, Cat Cole's after the ORCID link).
#[test]
fn authblk_mark_before_a_line_stays_in_place() {
  let xml = convert_repro!("author_authblk_mark_before_line_stays.tex");
  let logo = orcid_logo(&xml);
  let both = [affiliation("Univ X"), affiliation("Univ Y")].join(" ");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup><break/>ann@x.org"#,
      &both
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2"><text font="italic" xml:id="id2.1">2</text></sup> (bob@x.org)"#,
      &both
    ),
    creator(
      false,
      &format!(
        r#"Cat Cole<sup xml:id="id3"><text font="italic" xml:id="id3.1">3</text></sup> <ref class="ltx_orcid" href="https://orcid.org/0000-0002-9991-8393" title="ORCID 0000-0002-9991-8393">{logo}</ref>"#
      ),
      &both
    ),
  ]);
}

/// A raised text after an authblk name's mark (`Ann Able$^{1}$\textsuperscript{equal contrib.}`, r4 review probe d4; d1
/// `\textsuperscript{(*)}`, d3 `$^{\rm (a)}$`, c4 the same class): a superscript is a mark only when its operand reads
/// as labels (`reads_as_labels`), so the name is no list and shows its text as int32a and pdflatex do (77n r4: "Ann
/// Able", the words lost). Its "1" still requests Univ X on `\lx@add@creator` (`author_label_marks`), so the run's Univ
/// Y is Bob's alone (77n r5: Ann's as well, the residual of a mark requesting nothing there).
#[test]
fn authblk_textual_superscript_after_mark_is_shown() {
  let xml = convert_repro!("author_authblk_textual_superscript_after_mark.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup><sup xml:id="id2">equal contrib.</sup>"#,
      &affiliation("Univ X")
    ),
    creator(false, "Bob Baker", &affiliation("Univ Y")),
  ]);
}

/// A raised parenthesised text on an authblk name with no mark (`Ann Able\textsuperscript{ (equal)}`, r4 review probe
/// d6): shown as written, its parentheses and space kept (77n r4: `<sup>equal</sup>`); the "1" no author carries leaves
/// both affiliations to both, as authblk prints them under the block.
#[test]
fn authblk_textual_superscript_alone_is_shown() {
  let xml = convert_repro!("author_authblk_textual_superscript_alone.tex");
  let both = [
    affiliation(r#"<sup xml:id="id2">1</sup>Univ X"#),
    affiliation("Univ Y"),
  ]
  .join(" ");
  assert_eq!(creators(&xml), [
    creator(true, r#"Ann Able<sup xml:id="id1"> (equal)</sup>"#, &both),
    creator(false, "Bob Baker", &both.replace("id2", "id3")),
  ]);
}

/// Control for `reads_as_labels`: a `\textsuperscript{2}` after `$^{1}$` is a mark, so Ann Able's two marks are both
/// answered and shown by her affiliations.
#[test]
fn authblk_superscript_marks_stay_marks() {
  let xml = convert_repro!("author_authblk_superscript_marks_control.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &[affiliation("Univ X"), affiliation("Univ Y")].join(" ")
    ),
    creator(false, "Bob Baker", &affiliation("Univ Y")),
  ]);
}

/// An `\affil` inside a later authblk `\author` after a marked author an `\affil` answers (r4 review probe e1; e2, e3
/// the same class): the uncapped run's `new` reads Ann as answered by her deferred Univ X, so Univ Y is Bob's alone, as
/// int32a and Perl's `annotate=new` give it (77n r4: Ann with Univ Y as well). pdflatex prints neither Bob nor Univ Y.
#[test]
fn authblk_affil_in_later_author_skips_marked_one() {
  let xml = convert_repro!("author_authblk_affil_in_later_author_skips_marked.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann", &affiliation("Univ X")),
    creator(false, "Bob", &affiliation("Univ Y")),
  ]);
}

/// An `\affil` inside an authblk `\author` between marked authors (r4 review probe e4): Cat's `\affil` is digested with
/// her, before Bob's Univ Y, which the flush offers (`lx_frontmatter_offered_labels`) — so Univ Z is Cat's alone (77n
/// r4: all three; int32a: Bob with Univ X, Cat with Univ Y).
#[test]
fn authblk_affil_in_author_between_marked_ones() {
  let xml = convert_repro!("author_authblk_affil_in_author_between_marked.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann", &affiliation("Univ X")),
    creator(false, "Bob", &affiliation("Univ Y")),
    creator(false, "Cat", &affiliation("Univ Z")),
  ]);
}

/// An unlabelled authblk `\affil` after a labelled one (`\author[1]{Ann}\author{Bob}\affil[1]{Univ X}\affil{Univ Y}`,
/// r3/r4/r5 review probes q10, r07; 2605.25017, whose e-mail line is both authors'): a `label=` answers no one in the
/// run, so Univ Y is both authors', as authblk prints it under its note and Perl's `annotate=new` gives it (int32a; 77n
/// r5 gave it to Bob only). Univ X on Bob, which authblk's `\AB@thenote` prints too, is a SYNC row.
#[test]
fn authblk_labelled_author_shares_the_run() {
  let xml = convert_repro!("author_authblk_labelled_author_shares_run.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann",
      &[affiliation("Univ Y"), affiliation("Univ X")].join(" ")
    ),
    creator(false, "Bob", &affiliation("Univ Y")),
  ]);
}

/// A footnote-symbol legend after mark-picked `\affil`s (r3 review probe q04): a contact led by a mark of its own is no
/// answer and reads `new` by the contact only, so it is every author's of the run, the ones their marks answer included
/// — control for the run's `new`, which passes those over for an unmarked `\affil` (77n r4's output).
#[test]
fn authblk_symbol_legend_reaches_marked_authors() {
  let xml = convert_repro!("author_authblk_symbol_legend_reaches_marked.tex");
  let legend = |id: &str| {
    affiliation(&format!(
      r#"<sup xml:id="{id}">†</sup>Corresponding author"#
    ))
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann<sup xml:id="id1">†</sup>"#,
      &[legend("id2"), affiliation("Univ X")].join(" ")
    ),
    creator(
      false,
      "Bob",
      &[legend("id3"), affiliation("Univ Y")].join(" ")
    ),
  ]);
}

/// An unlabelled authblk `\affil` after a mark-picked one, the marked author before an unmarked one (r4 review probes
/// c10, c11, c5): Ann Able's mark answers her, so Univ Y is Bob Baker's (77n r4: Ann's as well; int32a: Univ X on Bob).
#[test]
fn authblk_plain_affil_after_marked_goes_to_new_author() {
  let xml = convert_repro!("author_authblk_marked_then_plain_affil.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ X")),
    creator(false, "Bob Baker", &affiliation("Univ Y")),
  ]);
}

/// Unmarked authblk `\affil`s after a mark-picked one, the marked author first (r5 review probe g01; g03, g05): Ann
/// Able's mark answers her, so the run's Univ Y and Univ W are Bob Baker's, and the contact grid redistributor
/// (`distribute_upfront_contacts`) leaves them there — Ann's Univ X is still pending when it runs (77n r5: Univ Y handed
/// to Ann, Bob with Univ W only).
#[test]
fn authblk_unmarked_affils_after_marked_author() {
  let xml = convert_repro!("author_authblk_unmarked_affils_after_marked_author.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ X")),
    creator(
      false,
      "Bob Baker",
      &[affiliation("Univ Y"), affiliation("Univ W")].join(" ")
    ),
  ]);
}

/// A marked authblk author with its `\affil`, then an unmarked author with two (r5 review probe g02): pdflatex prints
/// two blocks, Ann Able's and Bob Baker's — no grid (77n r4 and r5: Univ Y on Ann, Bob with Univ W only).
#[test]
fn authblk_unmarked_affils_after_marked_group() {
  let xml = convert_repro!("author_authblk_unmarked_affils_after_marked_group.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ X")),
    creator(
      false,
      "Bob Baker",
      &[affiliation("Univ Y"), affiliation("Univ W")].join(" ")
    ),
  ]);
}

/// Two authblk authors sharing a mark and an unmarked third, then a marked `\affil` and three unmarked ones (r5 review
/// probe g04): Univ X is the marked pair's, Univ Y, W and V Cat Cole's (77n r5 paired them one each, the grid).
#[test]
fn authblk_marked_pair_then_unmarked_author() {
  let xml = convert_repro!("author_authblk_marked_pair_then_unmarked_author.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ X")),
    creator(false, "Bob Baker", &affiliation("Univ X")),
    creator(
      false,
      "Cat Cole",
      &[
        affiliation("Univ Y"),
        affiliation("Univ W"),
        affiliation("Univ V")
      ]
      .join(" ")
    ),
  ]);
}

/// A symbol legend after an unmarked `\affil` the run gave the unmarked author only (r5 review probe m11; g05): Univ Y is
/// Bob Baker's, and the legend the whole run's (the extent Perl's `new` gives the contact before it), so Ann Able, who
/// shows its dagger, keeps it (77n r5: the grid gave Univ Y to Ann and left Bob the legend alone).
#[test]
fn authblk_legend_after_passed_over_run() {
  let xml = convert_repro!("author_authblk_legend_after_passed_over_run.tex");
  let legend = |id: &str| marked_affiliation(id, "†", "Corresponding author");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">†</sup>"#,
      &[legend("id2"), affiliation("Univ X")].join(" ")
    ),
    creator(
      false,
      "Bob Baker",
      &[affiliation("Univ Y"), legend("id3")].join(" ")
    ),
  ]);
}

/// A marked authblk name whose mark a raised text (r5 review probe k1) or an address (k2) follows stays on
/// `\lx@add@creator` as printed, and its mark still requests the affiliation it picks (`author_label_marks`): Bob Baker
/// keeps Univ X (77n r5: nothing; k2 since r3).
#[test]
fn authblk_mark_then_text_still_requests() {
  let xml = convert_repro!("author_authblk_mark_then_raised_text_requests.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ X")),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup><sup xml:id="id2">equal contrib.</sup>"#,
      &affiliation("Univ X")
    ),
  ]);
  let xml = convert_repro!("author_authblk_mark_then_address_requests.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ X")),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup> (bob@x.org)"#,
      &affiliation("Univ X")
    ),
  ]);
}

/// The same for a second affiliation's mark (r5 review probe k5): Bob Baker, "2" and "equal" shown, keeps Univ Y (77n
/// r4: the "2" lost; r5: Univ Y lost).
#[test]
fn authblk_second_mark_then_text_still_requests() {
  let xml = convert_repro!("author_authblk_second_mark_then_raised_text_requests.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &[affiliation("Univ X"), affiliation("Univ Y")].join(" ")
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id1"><text font="italic" xml:id="id1.1">2</text></sup><sup xml:id="id2">equal</sup>"#,
      &affiliation("Univ Y")
    ),
  ]);
}

/// ptephy's `\affil{}{Univ Z}` after `\affil{1}{Univ X}\affil{2}{Univ Y}` (r5 review probe t02; t04; witnesses
/// 1605.07339, 2311.07297, 1902.10888): every author of the run answered, so Univ Z is the whole run's, as the class
/// prints it under all of the names (77n r5: Bob's only).
#[test]
fn ptephy_unlabelled_affil_after_labelled_reaches_all() {
  let xml = convert_repro!("author_ptephy_unlabelled_affil_after_labelled.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann",
      &[affiliation("Univ Z"), affiliation("Univ X")].join(" ")
    ),
    creator(
      false,
      "Bob",
      &[affiliation("Univ Z"), affiliation("Univ Y")].join(" ")
    ),
  ]);
}

/// opex3's `\address{Univ Z}` after mark-picked addresses (r5 review probe o04): every author answered, Univ Z is both
/// authors' (int32a; 77n r5: Bob Baker's only).
#[test]
fn opex3_unmarked_address_after_marked_reaches_all() {
  let xml = convert_repro!("author_opex3_unmarked_address_after_marked.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &[affiliation("Univ Z"), affiliation("Univ X")].join(" ")
    ),
    creator(
      false,
      "Bob Baker",
      &[affiliation("Univ Z"), affiliation("Univ Y")].join(" ")
    ),
  ]);
}

/// authblk's unmarked `\affil{Univ Z}` after the mark-picked affiliations of every author (r5 review probe r01; g06):
/// both authors', printed under the block (77n r5: Bob's only; int32a: Univ X on Bob as well).
#[test]
fn authblk_unmarked_affil_after_all_marked_reaches_all() {
  let xml = convert_repro!("author_authblk_unmarked_affil_after_all_marked.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann",
      &[affiliation("Univ Z"), affiliation("Univ X")].join(" ")
    ),
    creator(
      false,
      "Bob",
      &[affiliation("Univ Z"), affiliation("Univ Y")].join(" ")
    ),
  ]);
}

/// A mark glued to a capitalised "And" before the next name (r4/r5 review probes a1, a6): only "and" itself is a
/// separator after a mark, so the line stays one creator and is reported (`Error:frontmatter:merged_creators`) — 77n r5
/// read "And" as one and merged the names silently, the marks gone. Bob Baker's visible "2" is lost (pdflatex prints
/// "Ann Able1 And Bob Baker2"): the content loss is the flagged Error's, not a silent one.
#[test]
fn mark_glued_to_capital_and_is_flagged() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_mark_glued_to_capital_and_is_flagged.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log
      .lines()
      .any(|l| l.starts_with("Error:frontmatter:merged_creators")),
    "{log}"
  );
  assert_eq!(creators(&squash(&xml)), [creator(
    true,
    r#"Ann Able<sup xml:id="id1"><Math mode="inline" tex="1" text="1" xml:id="m1"> <XMath xml:id="m1.1"> <XMTok meaning="1" role="NUMBER">1</XMTok> </XMath> </Math></sup>And Bob Baker"#,
    &[
      affiliation("Ann Able is with Univ X."),
      affiliation("Bob Baker is with Univ Y.")
    ]
    .join(" ")
  )]);
}

/// authblk `\affil{\vspace{-1.05em}$^2$\normalsize{…}}` lines (2605.11124): the spacing before the mark prints nothing,
/// so each line goes to the author its mark picks (`opens_with_a_mark` past `leading_unprinted_end`; 77n r5: every line
/// after the first to every author since the last `\affil`).
#[test]
fn authblk_marked_affil_after_vspace() {
  let xml = convert_repro!("author_authblk_marked_affil_after_vspace.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ X")),
    creator(false, "Bob Baker", &affiliation("Univ Y")),
    creator(false, "Cat Cole", &affiliation("Univ W")),
  ]);
}

/// An ORCID link glued to an address in a name line (r5 review probe n02; n04 `\cite{x}ann@x.org`): a name annotation
/// other than the address's markup keeps the run the name's, as printed (77n r5: the link moved into an e-mail
/// contact).
#[test]
fn orcid_glued_to_email_stays_in_the_name() {
  let xml = convert_repro!("author_orcid_glued_to_email_stays.tex");
  let logo = orcid_logo(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &format!(
        r#"Ann Able <ref class="ltx_orcid" href="https://orcid.org/0000-0002-1825-0097" title="ORCID 0000-0002-1825-0097">{logo}</ref>ann@x.org"#
      ),
      &affiliation("Univ X")
    ),
    creator(false, "Bob Baker", ""),
  ]);
}

/// A mark glued to a braced footnote symbol (`Cat Cole$^{1{\dagger}}$`, 2605.01693) under an `\affil{\small $^{1}$…}`:
/// the mark is 1 and the dagger shown (`split_glued_symbol`), so Cat Cole keeps Univ X now that the `\small` before the
/// line's mark no longer hides it (int32a and 77n r5: the dagger lost; the 77n r6 candidate: Univ X lost).
#[test]
fn authblk_glued_braced_symbol_mark() {
  let xml = convert_repro!("author_authblk_glued_braced_symbol_mark.tex");
  let univ_x = |id: &str| {
    affiliation(&format!(
      r#"<text fontsize="90%" xml:id="{id}">Univ X</text>"#
    ))
  };
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_x("id1")),
    creator(false, "Bob Baker", &univ_x("id2")),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id3">†</sup>"#,
      &univ_x("id4")
    ),
  ]);
}

/// E-mail lines piled on the last of two creators whose affiliations are mark-picked (2605.30274's ACL block): the
/// pending contacts are affiliations, no e-mail, so the contact grid redistributor still deals the e-mails out one per
/// creator, as int32a does (the 77n r6 candidate left both on the last). The names parted by spacing alone stay one
/// creator each: the SYNC row of spacing-parted names (batch 77p).
#[test]
fn email_pile_beside_marked_affiliations_is_dealt() {
  let xml = convert_repro!("author_email_pile_beside_marked_affiliations.tex");
  let email = |id: &str, address: &str| {
    format!(
      r#"<contact name="Email: " role="email"><text font="typewriter" xml:id="{id}">{address}</text></contact>"#
    )
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able Bob Baker",
      &[email("id1", "{ann,bob}@x.edu"), affiliation("Univ X")].join(" ")
    ),
    creator(
      false,
      r#"<text font="bold" xml:id="id2">Cat Cole</text> <text font="bold" xml:id="id3">Dan Dorn</text>"#,
      &[
        email("id4", "{cat}@y.com"),
        affiliation("Univ X"),
        affiliation("Univ Y")
      ]
      .join(" ")
    ),
  ]);
}

/// A footnote symbol on an authblk name an address (r6 review probe s3; s1, z1) or a raised text (s5) follows, kept on
/// `\lx@add@creator` as printed: the symbol is shown once, where the name prints it — its mark requests nothing
/// (`author_label_marks` requests labels only), so no unanswered mark is appended after the name (77n r6: "Bob Baker∗
/// (bob@x.org)∗", "Bob Baker∗equal∗"). Ann Able's "Corresponding author" is the run's extent (Perl's `annotate=new`),
/// not the page — pdflatex shows the ∗ on Bob only (77n r7 review finding 2; SYNC "A symbol legend reaches authors who
/// do not show its symbol").
#[test]
fn authblk_symbol_mark_then_text_shown_once() {
  let xml = convert_repro!("author_authblk_symbol_mark_then_address_shown_once.tex");
  let legend = |id: &str| marked_affiliation(id, "∗", "Corresponding author");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &[legend("id1"), affiliation("Univ X")].join(" ")
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2"><text font="italic" xml:id="id2.1">∗</text></sup> (bob@x.org)"#,
      &[affiliation("Univ Y"), legend("id3")].join(" ")
    ),
  ]);
  let xml = convert_repro!("author_authblk_symbol_mark_then_raised_text_shown_once.tex");
  assert_eq!(creators(&xml), [creator(
    true,
    r#"Bob Baker<sup xml:id="id1"><text font="italic" xml:id="id1.1">∗</text></sup><sup xml:id="id2">equal</sup>"#,
    &affiliation("Univ X")
  )]);
}

/// A name kept on `\lx@add@creator` whose mark is glued to a symbol (`Bob Baker$^{2*}$ (bob@y.org)`): the `\affil` its
/// label answers reads the mark as the request does (`label_pieces`), so `\affil{$^2$Univ Y}` is Bob's and not authblk's
/// run's (77n r7 review probe g8: Ann Able had Univ Y too). The `*` legend on Ann is the run's extent, not the page (SYNC
/// "A symbol legend reaches authors who do not show its symbol").
#[test]
fn authblk_glued_symbol_mark_answered_by_its_affil() {
  let xml = convert_repro!("author_authblk_glued_symbol_mark_answered_by_its_affil.tex");
  let legend = |id: &str| marked_affiliation(id, "∗", "Corresponding");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &[legend("id1"), affiliation("Univ X")].join(" ")
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2"><text font="italic" xml:id="id2.1">2∗</text></sup> (bob@y.org)"#,
      &[legend("id3"), affiliation("Univ Y")].join(" ")
    ),
  ]);
}

/// A whole author block in one authblk `\author`, its marks glued to symbols (`$^{1\ast}$`, `$^{1\dagger}$`) and its
/// legends inside it (2605.10201's shape): one creator, the block as printed — its glued marks request their labels
/// only, so nothing is appended to the name (77n r6: "∗,†" after the last legend).
#[test]
fn authblk_glued_symbols_in_one_author_block() {
  let xml = convert_repro!("author_authblk_glued_symbols_in_one_author_block.tex");
  let bold_marked = |id: &str, name: &str, mark: &str| {
    format!(
      r#"<text font="bold" xml:id="{id}">{name}<sup xml:id="{id}.1"><text font="italic" xml:id="{id}.1.1">{mark}</text></sup></text>"#
    )
  };
  assert_eq!(creators(&xml), [creator(
    true,
    &format!(
      r#"{} {} {} <break/><sup xml:id="id4">1</sup>Univ X <sup xml:id="id5">2</sup>Univ Y <sup xml:id="id6"><text font="italic" xml:id="id6.1">∗</text></sup> Equal contribution <sup xml:id="id7">†</sup> Corresponding author"#,
      bold_marked("id1", "Ann Able", "1∗"),
      bold_marked("id2", "Bob Baker", "2∗"),
      bold_marked("id3", "Cat Cole", "1†"),
    ),
    ""
  )]);
}

/// An `\author[1]` and another author's mark `$^{1}$` picking the same mark-led `\affil{$^1$Univ X}` (r6 review probe
/// f2): authblk prints the label as the superscript the mark is, so Bob Baker keeps Univ X beside the run's Univ Y
/// (int32a: both; 77n r3-r6: Univ Y only). Ann Able's Univ Y is the SYNC row of authblk's `\AB@thenote`.
#[test]
fn authblk_labelled_author_and_marked_affil() {
  let xml = convert_repro!("author_authblk_labelled_author_and_marked_affil.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ X")),
    creator(
      false,
      "Bob Baker",
      &[affiliation("Univ Y"), affiliation("Univ X")].join(" ")
    ),
  ]);
}

/// The mirror: `\author[1]{Ann Able}`, `\author{Bob Baker$^{1}$}` and `\affil[1]{Univ X}` (r6 review probe f7; f1):
/// Bob Baker's mark is the label, so both authors keep Univ X beside the run's Univ Y (int32a, 77n r3-r6: Bob Baker
/// without Univ X).
#[test]
fn authblk_marked_author_and_labelled_affil() {
  let xml = convert_repro!("author_authblk_marked_author_and_labelled_affil.tex");
  let both = [affiliation("Univ Y"), affiliation("Univ X")].join(" ");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &both),
    creator(false, "Bob Baker", &both),
  ]);
}

/// A symbol legend after two unmarked `\affil`s the run gave the unmarked author only (r6 review probe x1; x5): the run
/// keeps the extent Perl's `new` gave its first contact, so Ann Able, who shows the dagger, keeps the legend (int32a,
/// 77n r5, r6: Univ W recorded Bob Baker alone, and Ann lost it).
#[test]
fn authblk_legend_after_two_unmarked_affils() {
  let xml = convert_repro!("author_authblk_legend_after_two_unmarked_affils.tex");
  let legend = |id: &str| marked_affiliation(id, "†", "Corresponding author");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">†</sup>"#,
      &[legend("id2"), affiliation("Univ X")].join(" ")
    ),
    creator(
      false,
      "Bob Baker",
      &[affiliation("Univ Y"), affiliation("Univ W"), legend("id3")].join(" ")
    ),
  ]);
}

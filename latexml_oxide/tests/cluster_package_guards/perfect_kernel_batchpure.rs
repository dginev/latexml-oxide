//! Red/green guards for batch PURE: `ltx:personname` is the name alone (FIDELITY_CHARTER "a name's label is the name
//! alone"; DG 2026-11-16, purity and no lost mark one rule). `insert.rs`'s last step, `keep_personnames_pure`
//! (`frontmatter/pure_names.rs`), takes every mark printed on a name out of it: a mark a note of the creator answers is
//! that note's `mark`, a label of a contact the creator holds is consumed (ruling 3), any other a mark-only note after
//! the name (`<note role="footnotemark" mark="…"/>`). Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/pure_names/` and asserts no Error or Warning and every creator
//! whole (`xml:id`s aside, the space between tags collapsed). Witnesses 0707.3527, 0801.1438, 2406.11149, 2506.02863, 1904.03977,
//! 2007.16032, math0610203, 2211.11758.
use latexml::util::test::{convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/pure_names/",
      $path
    ))
  };
}

/// Each `ltx:creator` element of `xml` whole, in order: its `xml:id`s left out, the space between tags dropped, any other
/// run of space one.
fn creators(xml: &str) -> Vec<String> {
  xml
    .match_indices("<creator")
    .map(|(start, _)| {
      let end =
        start + xml[start..].find("</creator>").expect("closed creator") + "</creator>".len();
      let mut element = xml[start..end].to_string();
      while let Some(at) = element.find(" xml:id=\"") {
        let close = at
          + " xml:id=\"".len()
          + element[at + " xml:id=\"".len()..]
            .find('"')
            .expect("closed id");
        element.replace_range(at..=close, "");
      }
      let mut out = String::new();
      for (index, piece) in element.split('<').enumerate() {
        if index > 0 {
          out.push('<');
        }
        // (the space a tag ends before the next one opens: the indentation)
        let piece = match piece.split_once('>') {
          Some((tag, text)) if text.trim().is_empty() => format!("{tag}>"),
          _ => piece.to_string(),
        };
        out.push_str(&piece);
      }
      out.split_whitespace().collect::<Vec<_>>().join(" ")
    })
    .collect()
}

#[track_caller]
fn assert_creators((log, xml): (String, String), expected: &[&str]) {
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), expected, "{xml}");
}

/// 0707.3527's iaus `\author[]{M.Das$^1$, …}` over `\affiliation{$^1$Raman Research Institute…}`: the label of an
/// affiliation each creator holds is consumed (ruling 3), and a mark of the run no contact answers (`Cy Cole$^{1,*}$`'s
/// ∗) is a mark-only note. Red (REGFIX): `<personname>Ann Able<sup>1</sup></personname>`.
#[test]
fn a_label_of_a_held_affiliation_is_consumed() {
  let run = convert_files_with(
    repro!("pure_name_iaus_affiliation_label_consumed.tex"),
    &[("iaus.cls", repro!("iaus.cls"))],
    Some(PRELOAD),
  );
  assert_creators(run, &[
    r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation"><sup>1</sup>Univ A, Town A</contact></creator>"#,
    r#"<creator before=" " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation"><sup>2</sup>Univ B, Town B</contact></creator>"#,
    r#"<creator before=" " role="author"><personname>Cy Cole</personname><note mark="∗" role="footnotemark"/><contact name="Affiliation: " role="affiliation"><sup>1</sup>Univ A, Town A</contact></creator>"#,
  ]);
}

/// `Ann Able$^*$ and Bob Bee$^1$`, nothing answering either mark: each a mark-only note right after the name, never
/// dropped. Red (REGFIX): `<personname>Bob Bee<sup>1</sup></personname>`.
#[test]
fn an_unanswered_mark_is_a_mark_only_note() {
  assert_creators(
    convert_with(
      repro!("pure_name_unanswered_sup_mark_only.tex"),
      Some(PRELOAD),
    ),
    &[
      r#"<creator role="author"><personname>Ann Able</personname><note mark="∗" role="footnotemark"/></creator>"#,
      r#"<creator before=" " role="author"><personname>Bob Bee</personname><note mark="1" role="footnotemark"/></creator>"#,
    ],
  );
}

/// 0801.1438: `Kardo\v s$^{a }$\footnote{…}, …, Mi\v skuf\hskip 1pt$^{a }$\footnotemark[1]`, the affiliations set in a
/// `center` in the body: each letter a mark-only note right after its name, before the `\footnotemark`'s note, as
/// printed (`Cy Cole a∗`). Red (REGFIX): `<personname>Ann Able<sup>a</sup></personname>`.
#[test]
fn body_affiliation_labels_are_mark_only_notes() {
  assert_creators(
    convert_with(
      repro!("pure_name_body_affiliation_labels_mark_only.tex"),
      Some(PRELOAD),
    ),
    &[
      r#"<creator role="author"><personname>Ann Able</personname><note mark="a" role="footnotemark"/><contact name="Note: " role="note">Ann is supported by grant A.</contact></creator>"#,
      r#"<creator before=" " role="author"><personname>Bob Baker</personname><note mark="b" role="footnotemark"/><contact name="Note: " role="note">Bob is supported by grant B.</contact></creator>"#,
      r#"<creator before=" " role="author"><personname>Cy Cole</personname><note mark="a" role="footnotemark"/><note mark="*" role="footnotemark"><tags><tag>*</tag><tag role="refnum">*</tag><tag role="typerefnum">footnote *</tag></tags></note></creator>"#,
      r#"<creator before=" " role="author"><personname>Dan Dorn</personname><note mark="c" role="footnotemark"/><contact name="Note: " role="note">Dan is supported by grant D.</contact></creator>"#,
    ],
  );
}

/// 2406.11149's `Haoran Li$^{*}$\thanks{\quad Corresponding Author}` (and `Ann Able*\thanks{…}`; 2506.02863): a symbol
/// typed beside a `\thanks` is no mark of the `\thanks`, which prints its own (pdflatex "Haoran Li∗†"): a mark-only note
/// after the name, the `\thanks`'s note its own. Red (PURE round 1, revPURE B1): the typed ∗ the note's `mark`.
#[test]
fn a_typed_symbol_beside_a_thanks_is_a_mark_only_note() {
  assert_creators(
    convert_with(
      repro!("pure_name_typed_star_beside_thanks_mark_only.tex"),
      Some(PRELOAD),
    ),
    &[
      r#"<creator role="author"><personname>Ann Able</personname><note mark="*" role="footnotemark"/><note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks">Corresponding author.</note></creator>"#,
      r#"<creator before=" " role="author"><personname>Bob Baker</personname><note mark="∗" role="footnotemark"/><note class="ltx_note_frontmatter ltx_thanks_correspondence" role="thanks"> Corresponding author.</note></creator>"#,
      r#"<creator before=" " role="author"><personname>Cy Cole</personname></creator>"#,
    ],
  );
}

/// `Ann Able$^{a}$\thanks{a visitor at the Institute.}` and `Cy Cole$^{1}$\thanks{1 year at Univ C.}`: a letter or a
/// number opening a note's text is its word, never a copy of the name's mark (only a symbol is, or a superscript): each
/// mark a mark-only note, each note whole. Red (PURE round 1, revPURE B2): "visitor at the Institute." marked a.
#[test]
fn a_notes_opening_word_is_no_copy_of_the_mark() {
  assert_creators(
    convert_with(
      repro!("pure_name_letter_mark_beside_a_note_word.tex"),
      Some(PRELOAD),
    ),
    &[
      r#"<creator role="author"><personname>Ann Able</personname><note mark="a" role="footnotemark"/><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">a visitor at the Institute.</note></creator>"#,
      r#"<creator before=" " role="author"><personname>Bob Baker</personname><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks"><sup><text font="italic">b</text></sup>Visitor at the Lab.</note></creator>"#,
      r#"<creator before=" " role="author"><personname>Cy Cole</personname><note mark="1" role="footnotemark"/><note class="ltx_note_frontmatter ltx_thanks_note" role="thanks">1 year at Univ C.</note></creator>"#,
    ],
  );
}

/// 1904.03977's `\IEEEauthorblockN{Divyam Madaan$^{*}$\thanks{*Equal Contribution}}`: a note opening with the name's
/// mark takes it as its `mark`, its text without the copy, on each carrier. Red (REGFIX): `Ann Able<sup>*</sup>`, the
/// note "*Equal contribution." unmarked.
#[test]
fn a_legend_note_takes_the_names_mark() {
  assert_creators(
    convert_with(
      repro!("pure_name_legend_note_takes_the_mark.tex"),
      Some(PRELOAD),
    ),
    &[
      r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Univ A</contact><contact name="Email: " role="email">ann@a.edu</contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks">Equal contribution.</note></creator>"#,
      r#"<creator before=" " role="author"><personname>Bob Baker</personname><contact name="Affiliation: " role="affiliation">Univ B</contact><contact name="Email: " role="email">bob@b.edu</contact><note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks">Equal contribution.</note></creator>"#,
      r#"<creator before=" " role="author"><personname>Cy Cole</personname><contact name="Affiliation: " role="affiliation">Univ C</contact><contact name="Email: " role="email">cy@c.edu</contact></creator>"#,
    ],
  );
}

/// 2007.16032's svjour3 `Yuan Yuan*` (llncs here, in every TeX Live): a footnote symbol typed at the name's end, in its
/// text, is cut from it and a mark-only note. Red (REGFIX): `<personname>Bob Baker*</personname>`.
#[test]
fn a_hand_typed_symbol_leaves_the_name() {
  assert_creators(
    convert_with(
      repro!("pure_name_hand_typed_symbol_mark_only.tex"),
      Some(PRELOAD),
    ),
    &[
      r#"<creator role="author"><personname>Ann Able</personname><contact name="Affiliation: " role="affiliation">Univ A, Town A</contact></creator>"#,
      r#"<creator before=" " role="author"><personname>Bob Baker</personname><note mark="*" role="footnotemark"/><contact name="Affiliation: " role="affiliation">Univ A, Town A</contact></creator>"#,
    ],
  );
}

/// `James M\textsuperscript{c}Kernan` (math0610203) and IEEE's `1\textsuperscript{st} Bob Baker` (2211.11758): a
/// superscript that is the name's own ink stays, and no note is added. Green before and after.
#[test]
fn a_superscript_of_the_name_stays() {
  assert_creators(
    convert_with(
      repro!("pure_name_superscripts_of_the_name_kept.tex"),
      Some(PRELOAD),
    ),
    &[
      r#"<creator role="author"><personname>James M<sup>c</sup>Kernan</personname></creator>"#,
      r#"<creator before=" " role="author"><personname>1<sup>st</sup> Bob Baker</personname></creator>"#,
    ],
  );
}

/// 2208.09944: a name's mark-only note is no `\footnotemark` — a body `\footnotetext[2]` of the same mark is not its text
/// (`relocate_footnote` skips `_namemark`). Red (PURE round 1): each mark-only note took the body footnote's text.
#[test]
fn a_mark_only_note_takes_no_body_footnotetext() {
  let (log, xml) = convert_with(
    repro!("pure_name_mark_only_note_pairs_no_body_footnotetext.tex"),
    Some(PRELOAD),
  );
  assert_creators((log, xml.clone()), &[
    r#"<creator role="author"><personname>Ann Able</personname><note mark="1" role="footnotemark"/></creator>"#,
    r#"<creator before=" " role="author"><personname>Bob Baker</personname><note mark="2" role="footnotemark"/></creator>"#,
  ]);
  for text in ["Body note one.", "Body note two."] {
    assert_eq!(xml.matches(text).count(), 1, "{xml}");
  }
}

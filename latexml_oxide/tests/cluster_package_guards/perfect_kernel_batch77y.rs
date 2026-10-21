//! Red/green guards for batch 77y, the follow-up of batch 77p (the r8 re-review's findings, user rulings 2026-10-20
//! R10-R12 and the main-session decisions on the 77p r11 questions; `base_utilities.rs`, OXIDIZED_DESIGN_DIVERGENCES
//! #52(o)): a block merged into the lines above after a spaced break keeps the names below them; a link column printing
//! link words is a URL contact; an exact spelling of an address outranks a truncated one; a local part holding a list
//! spells no one; a marked affiliation's unmarked continuation lines stay welded to it in a symbol row; a marked address
//! no name spells goes to the names carrying its mark; a mark after a space labels the text before it; a symbol line set
//! in a font command is read through it; an icon a `\quad` parts from its link goes with the link; a bare domain's link
//! naming its domain is an institution. Round 2 (the 77y review's findings 1-8): a linked name whose surname is a link
//! word stays a name; a capitalized line alone under merged symbol lines stays the block's; a symbol row welds only
//! institutions and places, and nothing past its last labelled institution in a row with an unmarked name; a
//! homepage's handle link is a URL contact; a local part diverging from a longer name is the shorter name's. Round 3
//! (the rev77y2 re-review's findings 1-5): an unmarked line under a one-line last labelled institution is the row's; a
//! mark-led line under symbol lines is no names; link-word surnames stay names over any line; a link column over an
//! institution is the block's; the names split reads an e-mail and an institution's rest under it; a handle the name
//! spells is a URL contact. Round 4 (the rev77y3 re-review's findings 1-6, narrowing round 3): a name over an
//! institution of its own (an acronym, an opening no connective) stays a name; an organization's link the names run
//! through is their affiliation; a link column's lines are the block's by their kind; a plural or suffixed link-word
//! surname keeps a linked name. Each guard
//! asserts the three measures: no Error or Warning, every name once in its own `ltx:creator`, its contacts on it —
//! whole elements.
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

/// An affiliation contact.
fn affiliation(text: &str) -> String {
  format!(r#"<contact name="Affiliation: " role="affiliation">{text}</contact>"#)
}

/// An e-mail contact printed in typewriter type.
fn typewriter_email(address: &str, id: &str) -> String {
  format!(
    r#"<contact name="Email: " role="email"><text font="typewriter" xml:id="{id}">{address}</text></contact>"#
  )
}

/// A URL contact.
fn url(content: &str) -> String {
  format!(r#"<contact name="URL: " role="url">{content}</contact>"#)
}

/// An `\href` link as hyperref writes it, printing `text`.
fn href(target: &str, text: &str) -> String {
  format!(r#"<ref class="ltx_href" href="{target}">{text}</ref>"#)
}

/// The shared creator below the authors (OXIDIZED_DESIGN #159), holding `contacts`.
fn shared(contacts: &str) -> String { format!(r#"<creator role="author"> {contacts} </creator>"#) }

/// A name and its footnote-symbol mark.
fn marked(name: &str, id: &str, mark: &str) -> String {
  format!(r#"{name}<sup xml:id="{id}">{mark}</sup>"#)
}

/// A block merged into a row's symbol lines after a spaced break (`… \quad $^\ddagger$Univ B \\ Cat Cole \\ Univ C`)
/// takes only its leading labelled institutions there: the names line under them is the next author's, its line hers
/// (the r8 re-review's finding 1, probe w1). Was "Cat Cole" an affiliation of a nameless creator and Univ C on Ann Able
/// and Bob Baker (77p r11; int34a lost Cat Cole in a creator "‡Univ B").
#[test]
fn names_under_merged_symbol_lines_stay_names() {
  let xml = convert_repro!("author_symbol_lines_then_names_below.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &affiliation("Univ A")
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &affiliation("Univ B")
    ),
    creator(false, "Cat Cole", &affiliation("Univ C")),
  ]);
}

/// The same after `\\[1em]`, the names under it marked by a symbol of their own (the r8 re-review's finding 1, probe
/// w2): four authors, Univ C on the § names. Was "Cat Cole§" an affiliation of Bob Baker (77p r11).
#[test]
fn marked_names_under_merged_symbol_lines_stay_names() {
  let xml = convert_repro!("author_symbol_lines_then_marked_names_below.tex");
  assert_no_merged_names(&xml);
  let univ_c = affiliation("Univ C");
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &affiliation("Univ A")
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &affiliation("Univ B")
    ),
    creator(false, &marked("Cat Cole", "id3", "§"), &univ_c),
    creator(false, &marked("Dan Dorn", "id4", "§"), &univ_c),
  ]);
}

/// A column holding only a link that prints link words — "Model Card", "Paper Website", "Hugging Face Demo" — is a URL
/// contact of the block, no author (the r8 re-review's finding 2, probes u1, u3, u4). Was a creator named by each (77p
/// r11, int34a; 77p r10 as now).
#[test]
fn link_word_columns_are_urls() {
  let xml = convert_repro!("author_link_word_columns.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
    shared(&format!(
      "{} {} {}",
      url(&href("https://x.org/card", "Model Card")),
      url(&href("https://x.org/p", "Paper Website")),
      url(&href("https://huggingface.co/x", "Hugging Face Demo"))
    )),
  ]);
}

/// An address spelling one name exactly outranks a name whose longer spelling it holds only the start of (`wangyu@` is
/// Yu Wang's, not Yuxin Wang's "wangyuxin"; `liyang@` Yang Li's, not Yangfan Li's; the r8 re-review's finding 3, probes
/// s1, s2). Was each on the shared creator, the two names tied (77p r11).
#[test]
fn exact_spelling_outranks_a_truncated_one() {
  let xml = convert_repro!("author_email_exact_spelling_beats_truncated.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Yu Wang",
      &format!("{univ_a} {}", typewriter_email("wangyu@x.edu", "id1"))
    ),
    creator(false, "Yuxin Wang", &univ_a),
    creator(
      false,
      "Yang Li",
      &format!("{univ_a} {}", typewriter_email("liyang@x.edu", "id2"))
    ),
    creator(false, "Yangfan Li", &univ_a),
  ]);
}

/// A mailto link printing a list the address reader cannot part (`\{aable,bbaker,ccole\}it@y.edu`, 2605.27805's
/// `\{luoyanyan,…,fengjunlan\}it@chinamobile.com`; the r8 re-review's finding 4, probe t5) spells no one: one contact
/// of the block's shared creator (user ruling R10). Was the whole list the e-mail of Bob Baker (77p r11; int34a of Cat
/// Cole).
#[test]
fn unsplittable_address_list_spells_no_one() {
  let xml = convert_repro!("author_mailto_unsplittable_list.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    creator(false, "Cat Cole", &univ_a),
    shared(&format!(
      r#"<contact name="Email: " role="email">{}</contact>"#,
      href("mailto:x@y.edu", "{aable,bbaker,ccole}it@y.edu")
    )),
  ]);
}

/// A marked affiliation's unmarked continuation lines stay welded to it in a symbol row, as in a numbered block (x12;
/// main-session decision 3 on the 77p r10 questions, applied to symbol rows): `$^\dagger$Department of Physics \\
/// University A \\ Muncie, IN 47306 \\ $^\ddagger$School of Law \\ University B` (2602.05059; the r8 re-review's finding
/// 5, probe t7) — each line on the names carrying the mark above it. Was University A and University B on both names,
/// the place on the shared creator (77p r11; int34a every line on every name).
#[test]
fn symbol_row_continuation_lines_stay_welded() {
  let xml = convert_repro!("author_symbol_lines_continuation_welded.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &format!(
        "{} {} {}",
        affiliation("Department of Physics"),
        affiliation("University A"),
        affiliation("Muncie, IN 47306")
      )
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &format!(
        "{} {}",
        affiliation("School of Law"),
        affiliation("University B")
      )
    ),
  ]);
}

/// A marked address no name spells (`$^\dagger$zq77@x.edu`) goes to the names carrying its mark, without it (main-session
/// decision 3 on the 77p r11 questions; the r8 re-review's finding 6, probes s10, t4). Was "†zq77@x.edu" an e-mail of
/// the shared creator (77p r11).
#[test]
fn marked_unspelled_address_goes_by_its_mark() {
  let xml = convert_repro!("author_symbol_marked_unspelled_email.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &format!(
        "{} {}",
        affiliation("Univ A"),
        r#"<contact name="Email: " role="email">zq77@x.edu</contact>"#
      )
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &affiliation("Univ B")
    ),
  ]);
}

/// The same in a numbered block (`Ann Able$^{1,*}$ … \\ $^*$zq77@x.edu`; the r8 re-review's finding 6, probe t3): the
/// address goes to the author carrying `*`. Was "*zq77@x.edu" an e-mail of the shared creator (77p r11).
#[test]
fn marked_unspelled_address_goes_by_its_mark_numbered() {
  let xml = convert_repro!("author_numeric_marked_unspelled_email.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "∗"),
      &format!(
        "{univ_a} {}",
        r#"<contact name="Email: " role="email">zq77@x.edu</contact>"#
      )
    ),
    creator(false, "Bob Baker", &univ_a),
  ]);
}

/// A mark a space parts from the institution before it (`Univ A $^{\dagger}$`) labels that institution, no piece of its
/// own (the r8 re-review's finding 7, probe s3). Was Univ A and Univ B on every name and the marks "†", "‡" affiliations
/// of their own (77p r11).
#[test]
fn trailing_mark_after_a_space_labels_its_line() {
  let xml = convert_repro!("author_symbol_trailing_mark_after_space.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, &marked("Ann Able", "id1", "†"), &univ_a),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &affiliation("Univ B")
    ),
    creator(false, &marked("Cat Cole", "id3", "†"), &univ_a),
  ]);
}

/// A line of symbol-labelled institutions set in a one-argument font command (`\textit{$^{\dagger}$Univ A \qquad
/// $^{\ddagger}$Univ B}`) is read through it, as through a group (the r8 re-review's finding 8, probe s7): each
/// institution on the names carrying its symbol, its font aside as a wrapper group's is (2606.28166's `{\small …}`).
/// Was the whole line one affiliation of the shared creator (77p r11).
#[test]
fn symbol_line_in_a_font_command_goes_by_its_marks() {
  let xml = convert_repro!("author_symbol_line_in_font_command.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, &marked("Ann Able", "id1", "†"), &univ_a),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &affiliation("Univ B")
    ),
    creator(false, &marked("Cat Cole", "id3", "†"), &univ_a),
  ]);
}

/// An icon a `\quad` parts from the link after it (`\github \quad \url{…}`, 2601.17879's
/// `\def\github{\raisebox{-1.5pt}{\includegraphics…}}`; the r8 re-review's finding 9) goes with the link: one URL contact
/// of the block. Was the icon alone an affiliation of the last author, the URL on it too (77p r11).
#[test]
fn icon_parted_from_its_link_goes_with_it() {
  let xml = convert_repro!("author_icon_macro_quad_before_url.tex");
  assert_no_merged_names(&xml);
  // (the icon's file as the TeX tree holds it: where that tree is, aside)
  let texmf = regex::Regex::new(r#"candidates="[^"]*/(example-image\.[a-z]+)""#).unwrap();
  let xml = texmf.replace_all(&xml, r#"candidates="TEXMF/$1""#);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {}",
        affiliation("Univ A"),
        typewriter_email("aable@x.edu", "id1")
      )
    ),
    creator(false, "Bob Baker", &affiliation("Univ B")),
    shared(&url(concat!(
      r#"<text yoffset="-1.5pt" xml:id="id2"><graphics candidates="TEXMF/example-image.png" "#,
      r#"graphic="example-image" options="height=10.00002pt,keepaspectratio=true" xml:id="g1"/></text> "#,
      r#"<ref class="ltx_url" font="typewriter" href="https://github.com/x/y">https://github.com/x/y</ref>"#
    ))),
  ]);
}

/// A link to a bare domain printing the domain's name in lower case (`\href{https://www.deepseek.com}{deepseek-ai}`)
/// names that organization: an affiliation (the r8 re-review's finding 9, probe s14). Was a URL contact of Ann Able (77p
/// r11; r10 as now).
#[test]
fn bare_domain_link_naming_its_domain_is_an_institution() {
  let xml = convert_repro!("author_bare_domain_lowercase_organization.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &affiliation(&href("https://www.deepseek.com", "deepseek-ai"))
    ),
    creator(false, "Bob Baker", &affiliation("Univ B")),
  ]);
}

/// An e-mail label before a list the address reader parts, a semicolon before a brace list (`Email: aable@x.edu;
/// \{bbaker, ccole\}@y.edu`, 2604.10316; the r8 re-review's §B, probes t1, t2): each address the e-mail of the name it
/// spells. Was the whole line one affiliation of every name (77p r11; int34a the same).
#[test]
fn email_label_before_a_semicolon_brace_list() {
  let xml = convert_repro!("author_email_label_semicolon_brace_list.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", email("aable@x.edu"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", email("bbaker@y.edu"))
    ),
    creator(
      false,
      "Cat Cole",
      &format!("{univ_a} {}", email("ccole@y.edu"))
    ),
  ]);
}

/// Several mailto links after one label (`\textbf{Correspondence:} \href{mailto:…}{…}, \href{mailto:…}{…}`, 2604.25444;
/// the r8 re-review's §B, probe t6) are an e-mail line: each address the e-mail of the name it spells, named
/// "Correspondence:", a link to its mailbox. Was the whole line one affiliation of every name (77p r11; int34a the same).
#[test]
fn correspondence_label_before_two_mailto_links() {
  let xml = convert_repro!("author_correspondence_two_mailto_links.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  let email = |address: &str| {
    format!(
      r#"<contact name="Correspondence: " role="email">{}</contact>"#,
      href(&format!("mailto:{address}"), address)
    )
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", email("aable@x.edu"))
    ),
    creator(false, "Bob Baker", &univ_a),
    creator(
      false,
      "Cat Cole",
      &format!("{univ_a} {}", email("ccole@x.edu"))
    ),
  ]);
}

/// A linked name whose surname is a link word is a name: of two words with a hyphenated one (`Hans-Peter Link`, `Lucy
/// Page-Smith`), with an initial before it (`Lawrence E. Page`, the block's first column too), or over an institution
/// in its column (`Mary Ann Code \\ Univ D`) — the 77y review's finding 1, probes a1-a3, a7, a8. Was each column a
/// creator named by its institution, the name its URL contact (77y r1); int38b as now.
#[test]
fn link_word_surnames_stay_names() {
  let xml = convert_repro!("author_linked_names_link_word_surnames.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &href("https://x.org/lp", "Lawrence E. Page"),
      &affiliation("Univ A")
    ),
    creator(
      false,
      &href("https://x.org/hj", "Hans-Peter Link"),
      &affiliation("Univ B")
    ),
    creator(
      false,
      &href("https://x.org/ls", "Lucy Page-Smith"),
      &affiliation("Univ C")
    ),
    creator(
      false,
      &href("https://x.org/mc", "Mary Ann Code"),
      &affiliation("Univ D")
    ),
  ]);
}

/// A line of capitalized words alone under a row's symbol lines — an organization, a place, a legend — is no names
/// line: the block's, on its shared creator (the 77y review's finding 2, probes k1, k7, k9). Was a creator named by it,
/// no contact (77y r1); int38b as now.
#[test]
fn unnamed_line_under_symbol_lines_stays_the_blocks() {
  for (xml, line) in [
    (
      convert_repro!("author_symbol_lines_then_unnamed_line_google_deepmind.tex"),
      "Google DeepMind",
    ),
    (
      convert_repro!("author_symbol_lines_then_unnamed_line_hong_kong.tex"),
      "Hong Kong",
    ),
    (
      convert_repro!("author_symbol_lines_then_unnamed_line_equal_contribution.tex"),
      "Equal Contribution",
    ),
  ] {
    assert_no_merged_names(&xml);
    assert_eq!(
      creators(&xml),
      [
        creator(
          true,
          &marked("Ann Able", "id1", "†"),
          &affiliation("Univ A")
        ),
        creator(
          false,
          &marked("Bob Baker", "id2", "‡"),
          &affiliation("Univ B")
        ),
        shared(&affiliation(line)),
      ],
      "{line}"
    );
  }
}

/// A correspondence line under a symbol row's welded institutions (2604.02904's `Corresponding Author:
/// \texttt{aliwazirjam@gmail.com}`) is no continuation line: the block's (the 77y review's finding 3). Was an
/// affiliation of the names carrying the last institution's mark (77y r1); int38b as now.
#[test]
fn correspondence_line_under_welded_lines_is_the_blocks() {
  let xml = convert_repro!("author_symbol_lines_welded_then_correspondence.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &format!(
        "{} {}",
        affiliation("Department of Physics"),
        affiliation("University A")
      )
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &format!(
        "{} {}",
        affiliation("School of Law"),
        affiliation("University B")
      )
    ),
    shared(&affiliation(
      r#"Corresponding Author: <text font="typewriter" xml:id="id3">zq77@gmail.com</text>"#
    )),
  ]);
}

/// Notes under a symbol row's welded institutions (`Preprint. Under review.`, `Corresponding author: Ann Able`) are the
/// block's, no continuation lines (the 77y review's finding 3, probes c3, x4). Was each an affiliation of Bob Baker
/// (77y r1); int38b as now.
#[test]
fn notes_under_welded_lines_are_the_blocks() {
  let xml = convert_repro!("author_symbol_lines_welded_then_notes.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &format!(
        "{} {}",
        affiliation("Department of Physics,"),
        affiliation("University A")
      )
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &format!(
        "{} {}",
        affiliation("School of Law,"),
        affiliation("University B")
      )
    ),
    shared(&format!(
      "{} {}",
      affiliation("Preprint. Under review."),
      affiliation("Corresponding author: Ann Able")
    )),
  ]);
}

/// In a row with an unmarked name, an unmarked line under the last labelled institution is every name's of the row
/// (user ruling R12, literally; the 77y review's finding 3, probe c4; main-session decision on the 77y r2 question 2):
/// University B, right under `$^\ddagger$School of Law`, and `Department of Physics, University C` on all three. Was the
/// last line Bob Baker's alone, Cat Cole with none (77y r1); University B Bob Baker's alone (77y r2); int38b every line
/// on every name.
#[test]
fn line_past_the_welded_lines_of_a_row_with_an_unmarked_name_is_the_rows() {
  let xml = convert_repro!("author_symbol_lines_welded_unmarked_name.tex");
  assert_no_merged_names(&xml);
  let univ_b = affiliation("University B");
  let univ_c = affiliation("Department of Physics, University C");
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &format!(
        "{} {} {univ_b} {univ_c}",
        affiliation("Department of Physics"),
        affiliation("University A")
      )
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &format!("{} {univ_b} {univ_c}", affiliation("School of Law"))
    ),
    creator(false, "Cat Cole", &format!("{univ_b} {univ_c}")),
  ]);
}

/// The same row, its last labelled institution on one line (`$^\ddagger$School of Law, University B`): the line under
/// it, unmarked, is every name's of the row (R12; the rev77y2 re-review's finding 1, probe w1). Was it Bob Baker's
/// alone, Cat Cole with none (77y r1, r2: the weld carried the last labelled line on); int38b on all three.
#[test]
fn line_under_a_one_line_last_institution_is_the_rows() {
  let xml = convert_repro!("author_symbol_lines_welded_one_line_last_institution.tex");
  assert_no_merged_names(&xml);
  let univ_c = affiliation("Department of Physics, University C");
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &format!(
        "{} {} {univ_c}",
        affiliation("Department of Physics"),
        affiliation("University A")
      )
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &format!("{} {univ_c}", affiliation("School of Law, University B"))
    ),
    creator(false, "Cat Cole", &univ_c),
  ]);
}

/// A line a symbol mark leads under a row's symbol lines is a legend or a label, never names: `$^*$Project Lead` is the
/// legend of Ann Able's mark, on her; `$^\S$Google DeepMind`, a mark no name carries, the block's (the rev77y2
/// re-review's finding 2, probes m2, m4). Was each a creator named by it, no contact (77y r1, r2); int38b as now.
#[test]
fn mark_led_line_under_symbol_lines_is_no_names() {
  let xml = convert_repro!("author_symbol_lines_then_mark_led_legend.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†*"),
      &format!("{} {}", affiliation("Univ A"), affiliation("Project Lead"))
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &affiliation("Univ B")
    ),
  ]);
  let xml = convert_repro!("author_symbol_lines_then_mark_led_unlabelled.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &affiliation("Univ A")
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &affiliation("Univ B")
    ),
    shared(&affiliation(r#"<sup xml:id="id3">§</sup>Google DeepMind"#)),
  ]);
}

/// A linked name whose last word is a link word that is also a surname (`Mary Ann Code`, `Lawrence Edward Page`) is a
/// name whatever stands under it — an e-mail address, a place, nothing — and an earlier word is one whole (`Anne-Page
/// Smith`) (the rev77y2 re-review's finding 3, probes l4-l7). Was each column a creator named by the line under it, or
/// the name a URL contact of Ann Able (77y r2); int38b as now.
#[test]
fn link_word_surnames_stay_names_over_any_line() {
  let xml = convert_repro!("author_linked_names_link_word_surnames_any_line_under.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(
      false,
      &href("https://x.org/mc", "Mary Ann Code"),
      &typewriter_email("mcode@x.org", "id1")
    ),
    creator(
      false,
      &href("https://x.org/lp", "Lawrence Edward Page"),
      &affiliation("Stanford, CA")
    ),
    creator(
      false,
      &href("https://x.org/as", "Anne-Page Smith"),
      &affiliation("Univ B")
    ),
  ]);
  let xml = convert_repro!("author_linked_name_link_word_surname_alone.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, &href("https://x.org/mc", "Mary Ann Code"), ""),
  ]);
}

/// A link printing link words (`Hugging Face Demo`) over an institution in its `\and` column is no name: the link a URL
/// contact and the institution an affiliation of the block's shared creator (the rev77y2 re-review's finding 3, probes
/// l1, l3). Was a creator "Hugging Face Demo" holding Univ B (77y r2, int38b).
#[test]
fn link_column_over_an_institution_is_the_blocks() {
  let xml = convert_repro!("author_link_column_over_an_institution.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    shared(&format!(
      "{} {}",
      url(&href("https://x.org/hf", "Hugging Face Demo")),
      affiliation("Univ B")
    )),
  ]);
}

/// Under a row's symbol lines, a line of names over an e-mail address they spell is the next author's (`Cat Cole \\
/// \texttt{ccole@x.edu}`); a line over the rest of its institution is no names (`Shanghai Jiao Tong \\ University`, `Hong
/// Kong \\ The Hong Kong University of Science and Technology`): the block's, the institution every name's of the row
/// (R12) (the rev77y2 re-review's finding 4, probes k15, k12, k17). Was Cat Cole an affiliation of the shared creator
/// (77y r2, int38b), and creators "Shanghai Jiao Tong", "Hong Kong" (77y r1, r2). A line over an institution of its own
/// — an acronym (`CERN`, `RIKEN`), an opening no connective (`the University of Tokyo`) — is names, the institution
/// theirs (the rev77y3 re-review's finding 1, probes k20, k42, k31; 77y r3 and int38b had made them an affiliation of
/// the shared creator).
#[test]
fn names_split_under_symbol_lines_reads_the_line_under() {
  let xml = convert_repro!("author_symbol_lines_then_name_over_its_email.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      &marked("Ann Able", "id1", "†"),
      &affiliation("Univ A")
    ),
    creator(
      false,
      &marked("Bob Baker", "id2", "‡"),
      &affiliation("Univ B")
    ),
    creator(false, "Cat Cole", &typewriter_email("ccole@x.edu", "id3")),
  ]);
  for (xml, line, institution) in [
    (
      convert_repro!("author_symbol_lines_then_split_institution.tex"),
      "Shanghai Jiao Tong",
      "University",
    ),
    (
      convert_repro!("author_symbol_lines_then_place_over_its_institution.tex"),
      "Hong Kong",
      "The Hong Kong University of Science and Technology",
    ),
  ] {
    assert_no_merged_names(&xml);
    let institution = affiliation(institution);
    assert_eq!(
      creators(&xml),
      [
        creator(
          true,
          &marked("Ann Able", "id1", "†"),
          &format!("{} {institution}", affiliation("Univ A"))
        ),
        creator(
          false,
          &marked("Bob Baker", "id2", "‡"),
          &format!("{} {institution}", affiliation("Univ B"))
        ),
        shared(&affiliation(line)),
      ],
      "{line}"
    );
  }
  // (a line under the names that is an institution of its own — an acronym, an opening no connective — is theirs: the
  // rev77y3 re-review's finding 1, probes k20, k42, k31; 77y r3 made the names an affiliation of the shared creator)
  for (xml, names, institution) in [
    (
      convert_repro!("author_symbol_lines_then_name_over_an_acronym_institution.tex"),
      vec!["Cat Cole"],
      "CERN",
    ),
    (
      convert_repro!("author_symbol_lines_then_names_over_an_acronym_institution.tex"),
      vec!["Cat Cole", "Dan Dorn"],
      "RIKEN",
    ),
    (
      convert_repro!("author_symbol_lines_then_name_over_a_lowercase_institution.tex"),
      vec!["Cat Cole"],
      "the University of Tokyo",
    ),
  ] {
    assert_no_merged_names(&xml);
    let mut expected = vec![
      creator(
        true,
        &marked("Ann Able", "id1", "†"),
        &affiliation("Univ A"),
      ),
      creator(
        false,
        &marked("Bob Baker", "id2", "‡"),
        &affiliation("Univ B"),
      ),
    ];
    expected.extend(
      names
        .iter()
        .map(|name| creator(false, name, &affiliation(institution))),
    );
    assert_eq!(creators(&xml), expected, "{institution}");
  }
}

/// A homepage's link printing the name its domain spells (`\href{https://www.annable.com}{annable}`) is a URL contact
/// of its author, no organization (the 77y review's finding 4, probes d1, d2). Was each an affiliation (77y r1); int38b
/// as now.
#[test]
fn homepage_handle_link_is_a_url() {
  let xml = convert_repro!("author_homepage_handle_links.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {}",
        affiliation("Univ A"),
        url(&href("https://www.annable.com", "annable"))
      )
    ),
    creator(
      false,
      "Bob Baker",
      &url(&href("https://www.bobbaker.org", "bobbaker"))
    ),
  ]);
}

/// A local part going on past the start it shares with a longer name (`zhangweiwei@` beside "zhangweiwen") is no
/// truncation of it: the address is the shorter name's, whose spelling it opens with (the 77y review's finding 5, probe
/// n1). Was on Weiwen Zhang (77y r1); int38b as now.
#[test]
fn diverging_local_part_is_the_shorter_names() {
  let xml = convert_repro!("author_email_diverging_from_a_longer_name.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Wei Zhang",
      &format!("{univ_a} {}", typewriter_email("zhangweiwei@x.edu", "id1"))
    ),
    creator(false, "Weiwen Zhang", &univ_a),
  ]);
}

/// A homepage's link printing a handle its author's name spells (`{ann-able}`, `{bobbaker42}`) is a URL contact of its
/// author, though a mark or a digit sets it apart from a word (the rev77y2 re-review's finding 5, probes d5, d9). Was
/// each an affiliation (77y r2: an organization's handle, `deepseek-ai`); int38b as now.
#[test]
fn homepage_handle_spelling_the_name_is_a_url() {
  let xml = convert_repro!("author_homepage_handle_links_spelling_the_name.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {}",
        affiliation("Univ A"),
        url(&href("https://annable.com", "ann-able"))
      )
    ),
    creator(
      false,
      "Bob Baker",
      &url(&href("https://www.bobbaker42.com", "bobbaker42"))
    ),
  ]);
}

/// A link to an organization's bare domain printing its name (`\href{https://www.huawei.com}{Huawei}`) is the
/// affiliation of the names over it, though their words run inside it ("hua", "wei"): only a handle that is exactly a
/// spelling of one name is a homepage (the rev77y3 re-review's finding 2, probes h3, h9; h1-h8). Was a URL contact of
/// the shared creator and no affiliation on either name (77y r3); int38b and 77y r2 as now.
#[test]
fn organization_link_its_authors_names_run_through_is_their_affiliation() {
  let xml = convert_repro!("author_organization_link_its_authors_names_spell.tex");
  assert_no_merged_names(&xml);
  let huawei = affiliation(&href("https://www.huawei.com", "Huawei"));
  assert_eq!(creators(&xml), [
    creator(true, "Hua Wu", &huawei),
    creator(false, "Xiaoming Wei", &huawei),
  ]);
}

/// A column of a link over an institution over an e-mail address is the block's, each line by its kind: the link a URL
/// contact, the institution an affiliation, the address an e-mail (the rev77y3 re-review's finding 3, probe b9). Was the
/// address an affiliation (77y r3); a creator "Univ B" with the e-mail in int38b and 77y r2.
#[test]
fn link_column_lines_are_the_blocks_by_their_kind() {
  let xml = convert_repro!("author_link_column_over_an_institution_and_an_email.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    shared(&format!(
      "{} {} {}",
      url(&href("https://x.org/p", "Project Page")),
      affiliation("Univ B"),
      typewriter_email("contact@x.org", "id1")
    )),
  ]);
}

/// A linked name whose surname is a link word in the plural (`Mary Ann Pages`) or before a generational suffix
/// (`Lawrence Page Jr.`) stays a name over its institution (the rev77y3 re-review's findings 4 and 6, probes s9, s8).
/// Was the name a URL contact and the institution the shared creator's (77y r3; s8 a creator "Univ B" in 77y r2); int38b
/// as now.
#[test]
fn link_word_surnames_stay_names_plural_or_suffixed() {
  for (xml, name) in [
    (
      convert_repro!("author_linked_name_plural_link_word_surname.tex"),
      "Mary Ann Pages",
    ),
    (
      convert_repro!("author_linked_name_link_word_surname_with_suffix.tex"),
      "Lawrence Page Jr.",
    ),
  ] {
    assert_no_merged_names(&xml);
    assert_eq!(
      creators(&xml),
      [
        creator(true, "Ann Able", &affiliation("Univ A")),
        creator(
          false,
          &href("https://x.org/p", name),
          &affiliation("Univ B")
        ),
      ],
      "{name}"
    );
  }
}

//! Red/green guards for batch 77y, the follow-up of batch 77p (the r8 re-review's findings, user rulings 2026-10-20
//! R10-R12 and the main-session decisions on the 77p r11 questions; `base_utilities.rs`, OXIDIZED_DESIGN_DIVERGENCES
//! #52(o)): a block merged into the lines above after a spaced break keeps the names below them; a link column printing
//! link words is a URL contact; an exact spelling of an address outranks a truncated one; a local part holding a list
//! spells no one; a marked affiliation's unmarked continuation lines stay welded to it in a symbol row; a marked address
//! no name spells goes to the names carrying its mark; a mark after a space labels the text before it; a symbol line set
//! in a font command is read through it; an icon a `\quad` parts from its link goes with the link; a bare domain's link
//! naming its domain is an institution. Each guard asserts the three measures: no Error or Warning, every name once in
//! its own `ltx:creator`, its contacts on it — whole elements.
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
  let names = creators(&xml);
  assert_eq!(names.len(), 3, "{xml}");
  assert_eq!(
    names[0],
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {}",
        affiliation("Univ A"),
        typewriter_email("aable@x.edu", "id1")
      )
    )
  );
  assert_eq!(
    names[1],
    creator(false, "Bob Baker", &affiliation("Univ B"))
  );
  assert!(
    names[2].starts_with(r#"<creator role="author"> <contact name="URL: " role="url">"#)
      && names[2].contains("<graphics ")
      && names[2].contains(r#"href="https://github.com/x/y""#)
      && names[2].matches("<contact ").count() == 1,
    "{xml}"
  );
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

//! Red/green guards for batch 77p (arXiv html_feedback HF14, user rulings 2026-10-19): a run of two or more spacing
//! commands between two names at an author block's top level separates them, as `\hspace`/`\hfill` do
//! (`normalize_hspace_separators`, `spacing_run_separator`, base_utilities.rs; OXIDIZED_DESIGN_DIVERGENCES #52(o)); a
//! `\let` alias of a mark command is the command (`mark_command_meant`, TeX's `\ifx`); aero.cls's `\ADsetup` title
//! page reaches the frontmatter (aero_cls.rs; OXIDIZED_DESIGN_DIVERGENCES #775). Each guard asserts the three
//! measures: no Error or Warning, every name once in its own `ltx:creator`, its affiliations on it. Round 2: names parted
//! by horizontal space on one line share the address lines under them, as comma-parted names do (OXIDIZED_DESIGN
//! #52(o)). Round 3 (review findings): a run after an initial, off the names line, before what holds no name letter,
//! or after a byline's "By" parts nothing; a run of symbol commands in a mark is a label each, a font switch in one no
//! part of it, a blank line's `\par` in the names row nothing; a line's e-mail addresses go to the names they spell,
//! given names too, the rest by elimination; aero.cls hands over only the keys the paper set, and its `\cor` mark asks
//! for the e-mail. Round 4 (re-review): the word "and" in any case and one wrapper is the separator; a line after the
//! names line is a row of names only by its count or the line under it — so a row of institutions only where neither
//! tells it apart, another count or a legend line under it (probes v4, v2: the SYNC_STATUS address-row row) —; an
//! initial and a surname spell a name and elimination goes only where the order confirms it; a link line continues no
//! e-mail line, a legend line no name. Round 5 (re-review): a footnote-symbol-led line of a block with no marks parts nothing; over
//! an e-mail line a row is names only when an address spells one of its pieces or the addresses number the names of
//! both rows; on a marked e-mail line each settled address goes to its name, the rest stay one contact; the order path
//! respects the names spelled elsewhere; "and" in a run between two marked entries parts them. Round 6 (re-review): no
//! name goes by elimination while another e-mail line holds an address no name spells, and the run between two spelled
//! owners counts every author between them; a name scores the better of its spellings, not their sum; under an e-mail
//! line a line that is not addresses, and a link line anywhere, is its own entry; a command `\@fnsymbol` prints
//! (`\mathsection`) is a symbol mark, and a symbol-led affiliation line its own entry labelled by its symbol. Round 7
//! (user ruling 2026-10-20): a link line of an author block is a URL contact, the whole block's (the shared creator
//! below the authors) or its column's names'. Round 8 (user ruling 2026-10-20): a mailto line
//! (`\textbf{Correspondence:} \href{mailto:…}{…}`) is an e-mail line, its address the e-mail of the name it spells.
//! Round 9 (re-review): a colour command's colour is no letter of an address; a link printing an institution is an
//! affiliation, one to a bare address an e-mail line; a marked column's link line is its names', placed after their
//! marked institutions, and a link line opening a group no name; contacts in the order printed; the gap rule at a
//! list's end waits on the other e-mail lines; the lines under a row of names whose symbol marks label them go by those
//! marks; labelled lists of linked addresses, a symbol before an e-mail label, a sized group around one.
use latexml::util::test::{convert_files_with, convert_with};

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

/// `\textbf{Ann Able}\textsuperscript{1}\;\;\textbf{Bob Baker}\textsuperscript{2}` (2610.20409): two authors, each
/// with the affiliation its mark names (was one creator "Ann Able  Bob Baker" holding both).
#[test]
fn thin_space_run_separates_names() {
  let xml = convert_repro!("author_thin_space_run_separates_names.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A ")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
  ]);
}

/// `Ann Able$^{1}$ \ \ \ \ \ Bob Baker$^{2}$` (2310.10873), and a run ending its line, its last `\ ` a control return
/// (cs0207008): three authors.
#[test]
fn control_space_run_separates_names() {
  let xml = convert_repro!("author_control_space_run_separates_names.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
    creator(false, "Cat Cole", &affiliation("Univ A")),
  ]);
}

/// ACL's `Ann Able$^1$~~~~ Bob Baker$^2$~~~~ Cat Cole$^1$` (2603.20851, 2605.30274) over `$^1$Univ A~~~~$^2$Univ B`
/// (2310.15160): three authors, each with the affiliation its mark names (was one, with one affiliation "Univ A ²Univ B").
#[test]
fn tie_run_separates_names() {
  let xml = convert_repro!("author_tie_run_separates_names.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
    creator(false, "Cat Cole", &affiliation("Univ A")),
  ]);
}

/// The negatives: one tie or control space inside a name, a run beside "and", a run inside a group, a run before a
/// note or a mark — each stays where it was (the same creators as before the rule).
#[test]
fn single_spacing_stays_in_the_name() {
  let xml = convert_repro!("author_single_spacing_stays_in_the_name.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "J. Smith", ""),
    creator(false, "Ann Able", ""),
    creator(false, "Jan van der Berg", ""),
    creator(false, "Cat Cole", ""),
    creator(false, "Dan Dorn", ""),
    creator(false, "Eve Ell Fay Fox", ""),
    creator(
      false,
      "Gus Gray",
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note G.</note>"#
    ),
    creator(false, r#"Hal Hunt<sup xml:id="id2">1</sup>"#, ""),
  ]);
}

/// `\let\up\textsuperscript` (2209.14855): `\up{1}` marks as `\textsuperscript{1}` does — two authors with the
/// affiliations their marks name (was "Ann Able1", "Bob Baker1,2" and a creator "2Univ B").
#[test]
fn let_alias_mark_is_a_mark() {
  let xml = convert_repro!("author_let_alias_mark_is_a_mark.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(
      false,
      "Bob Baker",
      &format!("{} {}", affiliation("Univ A"), affiliation("Univ B"))
    ),
  ]);
}

/// aero.cls (2610.12827): the `\ADsetup` keys its `\maketitle` prints are the frontmatter — title, two authors with the
/// address, the e-mail on the author the class's `\cor` marks (no "(🖂)" in the name, 77p r3), the abstract, the
/// keywords joined by the class's ` · ` — and the class's `\maketitle` is not replayed into the body (was no title, no
/// creator, no abstract).
#[test]
fn aero_setup_is_the_frontmatter() {
  let (log, xml) = convert_files_with(
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/aero_setup_frontmatter.tex"),
    &[(
      "aero.cls",
      include_str!("../../../tools/perfect_kernel/repros/class-bindings/aero.cls"),
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(
      r#"class="ltx_authors_1line"> <title>Adaptive Nodes for Trajectories</title> <creator "#
    ),
    "{xml}"
  );
  let address = affiliation("Univ A, City, Country");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(r#"{address} <contact name="Email: " role="email">ann.able@univa.edu</contact>"#)
    ),
    creator(false, "Bob Baker", &address),
  ]);
  assert!(
    xml.contains(
      r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"> <p xml:id="abstract1.1">This is the abstract.</p> </abstract> <keywords name="Keywords: ">Alpha · Beta</keywords>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("This is the abstract.").count(), 1, "{xml}");
  assert_eq!(xml.matches("Univ A, City, Country").count(), 2, "{xml}");
}

/// An e-mail contact in the typewriter font, as `\texttt{…}` sets it.
fn typewriter_email(address: &str, id: &str) -> String {
  format!(
    r#"<contact name="Email: " role="email"><text font="typewriter" xml:id="{id}">{address}</text></contact>"#
  )
}

/// Names parted by horizontal space on one line — a spacing run, `\quad`, `\hspace`, `\hfill`, a `\quad` before the
/// line break that goes on with the next row of names — share the address lines under them, as comma-parted names do
/// (user ruling 2026-10-19, 77p r2; 1508.00093, 1009.3124, 1202.3179, 1503.08895, 2304.13998, 2507.06670): each address
/// on every name of its rows, the e-mails on the names they spell (was the lines on each row's last name only).
#[test]
fn space_parted_names_share_the_address() {
  let xml = convert_repro!("author_space_parted_names_share_the_address.tex");
  assert_no_merged_names(&xml);
  let (univ_a, univ_b, univ_c) = (
    affiliation("Univ A"),
    affiliation("Univ B"),
    affiliation("Univ C"),
  );
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    creator(
      false,
      "Cat Cole",
      &format!("{univ_b} {}", typewriter_email("cole@b.edu", "id1"))
    ),
    creator(false, "Dan Dorn", &univ_b),
    creator(
      false,
      "Eve Eads",
      &format!("{univ_b} {}", typewriter_email("eads@b.edu", "id2"))
    ),
    creator(false, "Fay Fox", &univ_c),
    creator(false, "Gil Gray", &univ_c),
    creator(false, "Hal Hunt", &univ_c),
    creator(false, "Ian Ives", &univ_c),
  ]);
}

/// The ruling's negatives keep their creators: a name with its own lines keeps them, names an `\and` parts share none,
/// one name keeps its lines, and lines labelled by the names' symbol marks go to the names carrying them, without the
/// mark (2505.23260, 2109.13296: were both on the last name; batch 77p r9, `apart_line_calls`). It also pins a
/// known-wrong residual, as it stands until its SYNC_STATUS row lands: a row of addresses parted by space too (set by
/// position) — Univ C on Dan Dorn, Cat Cole with none, "Univ D" an invented creator (row "an address row parted by
/// space, laid out by position").
#[test]
fn space_parted_names_negatives_pin_residuals() {
  let xml = convert_repro!("author_space_parted_names_negatives.tex");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
    creator(false, "Cat Cole", ""),
    creator(false, "Dan Dorn", &affiliation("Univ C")),
    creator(false, "Univ D", ""),
    creator(
      false,
      "Eve Eads",
      &format!(
        "{} {}",
        affiliation("Univ E"),
        typewriter_email("eads@e.edu", "id1")
      )
    ),
    creator(false, "Fay Fox", ""),
    creator(false, "Gil Gray", ""),
    creator(false, "Hal Hunt", &affiliation("Univ H")),
    creator(
      false,
      r#"Ivy Ives<sup xml:id="id2">†</sup>"#,
      &affiliation("Univ I")
    ),
    creator(
      false,
      r#"Jon Jay<sup xml:id="id3">‡</sup>"#,
      &affiliation("Univ J")
    ),
  ]);
}

/// The class-bindings repro's log and squashed XML for aero.cls's `\ADsetup` shapes, with the class excerpt `cls` beside
/// it, its conversion free of errors and warnings.
fn convert_aero(tex: &str, cls: &str) -> String {
  let (log, xml) = convert_files_with(tex, &[("aero.cls", cls)], Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  squash(&xml)
}

/// 2609.23496: the e-mail goes to the author the class's `\cor` marks, as elsarticle's `\corref` asks for its
/// `\cortext`, though it spells no surname; the mark prints no "(🖂)" inside the name (was a nameless creator holding the
/// e-mail and the glyph in the first personname).
#[test]
fn aero_corresponding_mark_owns_the_email() {
  let xml = convert_aero(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aero_corresponding_mark_owns_the_email.tex"
    ),
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/aero.cls"),
  );
  let address = affiliation(
    "1. University of the Azores, Ponta Delgada, Portugal <break/>2. Universidade de Coimbra, Coimbra, Portugal",
  );
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"A. K. de Almeida Jr<sup xml:id="id1">1,2</sup>"#,
      &format!(r#"{address} <contact name="Email: " role="email">allan.junior@uc.pt</contact>"#)
    ),
    creator(false, r#"T. Vaillant<sup xml:id="id2">2</sup>"#, &address),
  ]);
}

/// 2509.08607, 2511.17720: a copy of aero.cls whose `\maketitle` prints the kernel's `\@title`/`\@author`, under a paper
/// that never calls `\ADsetup`, keeps the paper's title and authors — the class's `initial:n` placeholders are no
/// frontmatter (was the title "Paper title: Preparation of papers for Astrodynamics", creators "First A. Author", …).
#[test]
fn aero_variant_keeps_the_kernel_title_and_authors() {
  let xml = convert_aero(
    include_str!(
      "../../../tools/perfect_kernel/repros/class-bindings/aero_variant/aero_title_author_without_setup.tex"
    ),
    include_str!("../../../tools/perfect_kernel/repros/class-bindings/aero_variant/aero.cls"),
  );
  assert!(xml.contains("<title>MasconCube Gravity</title>"), "{xml}");
  assert_eq!(creators(&xml), [
    creator(true, r#"Pietro Fanti<sup xml:id="id1">1</sup>"#, ""),
    creator(false, r#"Dario Izzo(🖂)<sup xml:id="id2">1</sup>"#, ""),
  ]);
  for placeholder in [
    "Paper title",
    "First A. Author",
    "Corresponding author",
    "Abstract content",
    "keywords content",
  ] {
    assert!(!xml.contains(placeholder), "{placeholder}: {xml}");
  }
}

/// 2603.17646 `R.~~Beyer`, 2604.25437 `L.~~Caballero-Ontanaya`, 2603.20914 `Rafa\l ~R.\ ~Suszek`: a run after an
/// initial is inside one person's name; a run after a whole name still parts (was "R." | "Beyer", …).
#[test]
fn spacing_run_after_an_initial_stays_in_the_name() {
  let xml = convert_repro!("author_spacing_run_after_an_initial.tex");
  assert_eq!(creators(&xml), [
    creator(true, "R. Beyer", ""),
    creator(false, "L. Caballero-Ontanaya", ""),
    creator(false, "Rafał R. Suszek", ""),
    creator(false, "Jo M. Gil", &affiliation("Univ A")),
    creator(false, "Ann Able", &affiliation("Univ A")),
  ]);
}

/// Off the names line a run parts names only (an address, a link, an e-mail row is no name: 2605.28131's `\url`s), and
/// anywhere only before a name letter at the top level or in a font group (not an icon's arguments: 2604.10005's
/// `\raisebox{…}{\includegraphics[…]{…}}\;\;Daedalus Research Team`; 2605.12398 `~~\JMorcid`; 2605.06250
/// `~~\mbox{\orcidlink{…}}`): five authors and no other creator (was "University of X", "Lab Y", "Eads Research Team",
/// "https://b.org" creators and an affiliation-less Eve Eads). The line of two links is a URL contact per link, its
/// column's author's (77p r11, the r7 re-review's finding 5; was one affiliation holding both).
#[test]
fn spacing_run_off_the_names_line_parts_nothing() {
  let xml = convert_repro!("author_spacing_run_off_the_names_line.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &affiliation("Department of Physics University of X")
    ),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{} {} {}",
        affiliation("Univ B"),
        affiliation("Dept. Lab Y"),
        typewriter_email("bob@y.edu", "id1")
      )
    ),
    creator(
      false,
      r#"Cat Cole <rule height="4.3pt" width="4.3pt"/>"#,
      r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id2">Note C.</note>"#
    ),
    creator(false, "Dan Dorn", ""),
    creator(
      false,
      "Eve Eads",
      &format!(
        "{} {} {}",
        affiliation(
          r#"<text yoffset="-1.3pt" xml:id="id3"><rule height="4.3pt" width="10.0pt" yoffset="0.4pt"/></text> Eads Research Team"#
        ),
        // (the two links each a URL contact of the column's author: 77p r11, the r7 re-review's finding 5)
        url(&link("", "https://a.org")),
        url(&link("", "https://b.org"))
      )
    ),
    creator(false, "Gil Gray", &affiliation("Univ G")),
    creator(false, "Hal Hunt", &affiliation("Univ G")),
    creator(false, "Ian Ives", &affiliation("Univ G")),
  ]);
}

/// Off the names line, marked entries that open with their size or font (`{\footnotesize $^{1}$Univ A} \ \ \ \small
/// $^{2}$Univ B`: 2603.27970, 2308.10525, 2605.24043, 2605.22036) part at a run, and an e-mail row parted by a run is
/// its addresses (`\texttt{a@x}~~~~\texttt{b@y}`: 2603.04592, 2603.12193, 2605.06264); a name in a group of its own
/// after a run that is already a separator still parts (2605.30545): three authors, each with its affiliation and its
/// e-mail (was "Bob Baker Cat Cole" one creator, "Univ A 2Univ B" one affiliation and the row one contact).
#[test]
fn spacing_run_marked_rows() {
  let xml = convert_repro!("author_spacing_run_marked_rows.tex");
  assert_no_merged_names(&xml);
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  let sized = |size: &str, id: &str, text: &str| {
    affiliation(&format!(
      r#"<text fontsize="{size}" xml:id="{id}">{text}</text>"#
    ))
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        r#"<note class="ltx_note_frontmatter ltx_thanks_note" role="thanks" xml:id="id1">Note A.</note> {} {}"#,
        sized("80%", "id2", "Univ A"),
        email("able@a.edu")
      )
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{} {}", sized("90%", "id3", "Univ B"), email("baker@b.edu"))
    ),
    creator(
      false,
      "Cat Cole",
      &format!("{} {}", sized("80%", "id4", "Univ A"), email("cole@a.edu"))
    ),
  ]);
}

/// A byline's "By" is no name, and the runs beside it and beside "and" part nothing themselves (2603.25922,
/// 2604.12864, 2603.27258, 2604.07186): four authors (was a creator "By", and Donald Robertson only the affiliation "and
/// Donald Robertson" of every other creator).
#[test]
fn byline_by_is_no_name() {
  let xml = convert_repro!("author_byline_by_before_spacing_runs.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"<text font="smallcaps" xml:id="id1">Bryna Kra</text>"#,
      ""
    ),
    creator(
      false,
      r#"<text font="smallcaps" xml:id="id2">Joel Moreira</text>"#,
      ""
    ),
    creator(
      false,
      r#"<text font="smallcaps" xml:id="id3">Florian K. Richter</text>"#,
      ""
    ),
    creator(
      false,
      r#"<text font="smallcaps" xml:id="id4">Donald Robertson</text>"#,
      ""
    ),
  ]);
}

/// A mark of two symbols is two labels (1811.10830 `$^{\spadesuit\heartsuit}$`), a font switch in a mark no part of its
/// label (2604.24532 `\textsuperscript{\normalfont{1},}`), and a blank line's `\par` between two names of the row prints
/// nothing (2009.12421): six authors, each with the affiliations its marks name (was a nameless creator holding Univ H,
/// Univ One and Univ Two, and "Eve Eads Fay Fox" one creator).
#[test]
fn mark_operand_labels_and_par_in_the_names_row() {
  let xml = convert_repro!("author_mark_operand_labels.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ S")),
    creator(
      false,
      "Bob Baker",
      &format!("{} {}", affiliation("Univ S"), affiliation("Univ H"))
    ),
    creator(false, "Cat Cole", &affiliation("Univ One")),
    creator(false, "Dan Dorn", &affiliation("Univ Two")),
    creator(false, "Eve Eads", &affiliation("Univ C")),
    creator(false, "Fay Fox", &affiliation("Univ C")),
  ]);
}

/// A line of as many addresses as names gives the ones no name spells to the names no address spells, in order
/// (1503.08895's `\{aszlam,jase,robfergus\}@fb.com`: "jase" to Jason Weston; was on Rob Fergus).
#[test]
fn email_owner_by_elimination() {
  let xml = convert_repro!("author_email_owner_by_elimination.tex");
  let (fair, univ_x) = (affiliation("FAIR"), affiliation("Univ X"));
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Arthur Szlam",
      &format!("{fair} {}", typewriter_email("aszlam@fb.com", "id1"))
    ),
    creator(
      false,
      "Jason Weston",
      &format!("{fair} {}", typewriter_email("jase@fb.com", "id2"))
    ),
    creator(
      false,
      "Rob Fergus",
      &format!("{fair} {}", typewriter_email("robfergus@fb.com", "id3"))
    ),
    creator(
      false,
      "Ann Able",
      &format!("{univ_x} {}", typewriter_email("able@x.edu", "id4"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_x} {}", typewriter_email("bobb@x.edu", "id5"))
    ),
    creator(
      false,
      "Cat Cole",
      &format!("{univ_x} {}", typewriter_email("cole@x.edu", "id6"))
    ),
  ]);
}

/// A marked block's e-mail line goes to the names its addresses spell, a given name too (2603.24730: yuqihu@, ahna@,
/// vasha@), and the rest by elimination (j8675309@ to Jen E. Corbett): was in order — ahna@ on Vasha DuTell — and
/// j8675309@ on a nameless creator.
#[test]
fn email_owner_by_given_name() {
  let xml = convert_repro!("author_email_owner_by_given_name.tex");
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  let (univ_b, mit) = (affiliation("Univ B"), affiliation("MIT "));
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Yuqi Hu",
      &format!("{univ_b} {}", email("yuqihu@berkeley.edu"))
    ),
    creator(
      false,
      "Vasha DuTell",
      &format!(
        "{univ_b} {mit} {} {}",
        email("vasha@berkeley.edu"),
        email("vasha@mit.edu")
      )
    ),
    creator(
      false,
      "Ahna R. Girshick",
      &format!("{univ_b} {}", email("ahna@berkeley.edu"))
    ),
    creator(
      false,
      "Jen E. Corbett",
      &format!("{mit} {}", email("j8675309@mit.edu"))
    ),
  ]);
}

/// Rows of names parted by runs, one address under them all (2603.09835): a run that parts two names on a line after a
/// row of names makes that line a row of names too, the break before it going on with the list — seven authors, each
/// with Microsoft (was "Dan B. Dorn" and "Fay Fox" read as address lines of Cat Cole and Eve Eads, Microsoft on Gil Gray
/// only).
#[test]
fn spacing_run_names_rows() {
  let xml = convert_repro!("author_spacing_run_names_rows.tex");
  assert_no_merged_names(&xml);
  let microsoft = affiliation("Microsoft");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id1">Equal contribution.</note> {microsoft}"#
      )
    ),
    creator(false, r#"Bob Baker<sup xml:id="id2">*</sup>"#, &microsoft),
    creator(false, "Cat Cole", &microsoft),
    creator(false, "Dan B. Dorn", &microsoft),
    creator(false, "Eve Eads", &microsoft),
    creator(false, "Fay Fox", &microsoft),
    creator(false, "Gil Gray", &microsoft),
  ]);
}

/// The word "and" between two names, in any case and in one wrapper, is their separator, never a name or an affiliation
/// (77p r4): `~~AND~~`, `~~And~~`, `~~{and}~~`, `~~\textrm{and}~~`, `\;\;\textnormal{AND}\;\;` (2606.27261), `\quad
/// \textnormal{and} \quad` (2608.24427), a byline's "BY" (was creators "AND", "And", "BY Max Moss", and "and" an
/// affiliation of the name before the wrapped word; 2607.03677 `~~ {and}~~`).
#[test]
fn and_word_in_any_case_or_wrapper_separates() {
  let xml = convert_repro!("author_and_word_in_any_case_or_wrapper.tex");
  assert_no_merged_names(&xml);
  let names = [
    "Ann Able",
    "Bob Baker",
    "Cat Cole",
    "Dan Dorn",
    "Eve Eads",
    "Fay Fox",
    "Gil Gray",
    "Hal Hunt",
    "Ivy Ives",
    "Jon Jay",
    "Kim Key",
    "Lou Lee",
    "Max Moss",
    "Ned Nash",
  ];
  assert_eq!(
    creators(&xml),
    names
      .iter()
      .enumerate()
      .map(|(at, name)| creator(at == 0, name, ""))
      .collect::<Vec<_>>()
  );
}

/// Off the names line of an unmarked block a spacing run parts names only on a row of names going on with them —
/// another number of names than the names line, or a line under it that is no row of as many entries (`row_of_names`)
/// —, not on the institutions
/// under the names, one under each (`Ann Able~~~~Bob Baker \\ Carnegie Mellon~~~~Johns Hopkins`; `\textbf` names over
/// `\textit` institutions; one name over two), nor on names under an affiliation line (probes p19, p27: a residual
/// pinned as int34a reads it, SYNC_STATUS row "names after an affiliation line"). Was "Carnegie Mellon", "Johns
/// Hopkins", "Google DeepMind", "Meta Platforms" creators (77p r3 re-review).
#[test]
fn institution_row_under_names_is_no_names() {
  let xml = convert_repro!("author_institution_row_under_names.tex");
  assert_no_merged_names(&xml);
  let row = affiliation("Carnegie Mellon Johns Hopkins");
  let italic_row = |first: &str, second: &str| {
    affiliation(&format!(
      r#"<text font="italic" xml:id="{first}">Google DeepMind</text> <text font="italic" xml:id="{second}">Meta Platforms</text>"#
    ))
  };
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &row),
    creator(false, "Bob Baker", &row),
    creator(false, "Cat Cole", &italic_row("id1", "id2")),
    creator(false, "Dan Dorn", &italic_row("id3", "id4")),
    creator(false, "Eve Eads", &row),
    creator(
      false,
      "Fay Fox",
      &format!(
        "{} {}",
        affiliation("Univ F"),
        affiliation("Gil Gray Hal Hunt")
      )
    ),
  ]);
}

/// Rows of names in another number than the names line, in another markup (2607.07318's `{\bf` row under a plain one;
/// 2603.16856, 2603.07624), or of as many names with a line under them that is no row of as many entries (2605.24414's
/// `\\[0.2em]` rows over `\textit{JIUTIAN Research}`, 2608.21409, 2607.09029: lost as affiliation lines in an r4 draft)
/// go on with the names: sixteen authors, each with the institution under its rows.
#[test]
fn names_rows_by_count_or_following_line() {
  let xml = convert_repro!("author_spacing_run_names_rows_by_count.tex");
  assert_no_merged_names(&xml);
  let personnames: Vec<String> = creators(&xml)
    .iter()
    .map(|c| {
      let start = c.find("<personname>").expect("a personname") + "<personname>".len();
      let end = c.find("</personname>").expect("a personname");
      c[start..end].to_string()
    })
    .collect();
  assert_eq!(personnames, [
    "Ann Able",
    "Bob Baker",
    "Cat Cole",
    "Dan Dorn",
    "Eve Eads",
    "Fay Fox",
    "Gil Gray",
    "Hal Hunt",
    "Ivy Ives",
    "Jon Jay",
    "Kim Key",
    "Lou Lee",
    "Max Moss",
    "Ned Nash",
    "Oli Ott",
    "Pat Poe",
  ]);
  for (at, c) in creators(&xml).iter().enumerate() {
    let institution = if at < 8 { "Tencent" } else { "LiAuto Inc." };
    assert!(
      c.ends_with(&format!("{} </creator>", affiliation(institution))),
      "{c}"
    );
    assert_eq!(c.matches("role=\"affiliation\"").count(), 1, "{c}");
  }
}

/// E-mail owners in an unmarked block (77p r4 decision: elimination only where the order is confirmed): the initials
/// spell a name (`bb@`, `aa@`: p9), a role mailbox is no person's (`info@`: p28, the block's), an
/// initial and a surname spell one (`lzhang@` Li Zhang, then `zhangw@` the one name left: p8), a name an address on
/// another e-mail line spells leaves the pool (`ccruz@` on the next line: `qq@` Bo Bell's; 2605.06223), and a line not in
/// name order (2509.10377, by institution) places no address by its place (`xx@`, `yy@` unplaced). Unplaced addresses
/// are the block's, on the shared creator (user ruling R10 2026-10-20; 77p r9 put them on the group's last name).
#[test]
fn email_owner_by_confirmed_order() {
  let xml = convert_repro!("author_email_owner_confirmed_order.tex");
  let tt = |address: &str, id: &str| typewriter_email(address, id);
  let (x, y, z, a, w) = (
    affiliation("Univ X"),
    affiliation("Univ Y"),
    affiliation("Univ Z"),
    affiliation("Univ A"),
    affiliation("Univ W"),
  );
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &format!("{x} {}", tt("aa@x.edu", "id1"))),
    creator(
      false,
      "Bob Baker",
      &format!("{x} {}", tt("bb@x.edu", "id2"))
    ),
    creator(false, "Cat Cole", &y),
    creator(
      false,
      "Dan Dorn",
      &format!("{y} {}", tt("ddorn@y.edu", "id3"))
    ),
    creator(
      false,
      "Wei Zhang",
      &format!("{z} {}", tt("zhangw@z.edu", "id4"))
    ),
    creator(
      false,
      "Li Zhang",
      &format!("{z} {}", tt("lzhang@z.edu", "id5"))
    ),
    creator(
      false,
      "Bob Smith",
      &format!("{z} {}", tt("smith@z.edu", "id6"))
    ),
    creator(
      false,
      "Al Ames",
      &format!("{a} {}", tt("aames@a.edu", "id7"))
    ),
    creator(false, "Bo Bell", &format!("{a} {}", tt("qq@a.edu", "id8"))),
    creator(
      false,
      "Cy Cruz",
      &format!("{a} {}", tt("ccruz@b.edu", "id9"))
    ),
    creator(
      false,
      "Eve Eads",
      &format!("{w} {}", tt("eeads@w.edu", "id10"))
    ),
    creator(
      false,
      "Fay Fox",
      &format!("{w} {}", tt("ffox@w.edu", "id11"))
    ),
    creator(false, "Gil Gray", &w),
    creator(false, "Hal Hunt", &w),
    shared(&format!(
      "{} {} {}",
      tt("info@y.edu", "id12"),
      tt("xx@w.edu", "id13"),
      tt("yy@w.edu", "id14")
    )),
  ]);
}

/// E-mail owners in a marked block: an initial and a surname spell a name (2608.10670's `psingh1_be22@` among three
/// Singhs, `kbatra_be22@`; 2606.23525's `tli104@`), an address between two spelled ones with one name between their
/// owners is that name's (2303.11225's `xuta@`, `musclewu@`), a link line under the e-mail lines is its own entry,
/// welded into neither the e-mail line (2403.08629) nor the affiliation before them (77p r6: it had continued "Univ M";
/// r6 put it on the last name, Runnan Li, as an affiliation; since r7 a URL contact of the whole block, the shared
/// creator below the authors, user ruling 2026-10-20), and a line of spacing alone under an e-mail line is no entry (2510.07961,
/// 2607.21993: an r4 draft panicked on it; since r6 dropped, `prints_nothing`, so the `\texttt` of the line above
/// stays on each address): every address on its author (was each line one contact on the last author, the link welded
/// into the last).
#[test]
fn email_owner_by_confirmed_order_marked() {
  let xml = convert_repro!("author_email_owner_confirmed_order_marked.tex");
  assert_no_merged_names(&xml);
  let (t, j, m) = (
    affiliation("Univ T"),
    affiliation("Univ J"),
    affiliation("Univ M"),
  );
  let mut ids = 0;
  let mut email = |address: &str| {
    ids += 1;
    typewriter_email(address, &format!("id{ids}"))
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Karamvir Singh Batra",
      &format!("{t} {}", email("kbatra_be22@t.edu"))
    ),
    creator(
      false,
      "Prathamjyot Singh",
      &format!("{t} {}", email("psingh1_be22@t.edu"))
    ),
    creator(
      false,
      "Jasmeet Singh",
      &format!("{t} {}", email("jasmeet.singh@t.edu"))
    ),
    creator(
      false,
      "Tianjian Li",
      &format!("{j} {}", email("tli104@j.edu"))
    ),
    creator(
      false,
      "Daniel Khashabi",
      &format!("{j} {}", email("danielk@j.edu"))
    ),
    creator(
      false,
      "Tianyu He",
      &format!("{m} {}", email("tianyuhe@m.com"))
    ),
    creator(false, "Xu Tan", &format!("{m} {}", email("xuta@m.com"))),
    creator(
      false,
      "Tadas Baltrusaitis",
      &format!("{m} {}", email("tabaltru@m.com"))
    ),
    creator(
      false,
      "HsiangTao Wu",
      &format!("{m} {}", email("musclewu@m.com"))
    ),
    creator(
      false,
      "Runnan Li",
      &format!("{m} {}", email("runnan.li@m.com"))
    ),
    shared_urls(&[
      r#"<ref class="ltx_href" href="https://a.github.io/">https://a.github.io/</ref>"#
    ]),
  ]);
}

/// A legend line after the names — a footnote symbol and its text, `\\ \textsuperscript{*}\textit{Equal
/// contribution.}` (2509.20748, aero.cls and the article `\author` alike) — is no part of the name before it (was "Bob
/// Baker*Equal contribution.1"). Its entry still lands on the last name, the symbol-mark SYNC_STATUS residual pinned
/// here; the `\dagger` line answers Bob Baker's mark.
#[test]
fn legend_line_is_no_name() {
  let xml = convert_repro!("author_legend_line_is_no_name.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">*</sup><sup xml:id="id2">1</sup>"#,
      ""
    ),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id3">*</sup><sup xml:id="id4">1</sup>"#,
      ""
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id5">1</sup>"#,
      &format!(
        "{} {}",
        affiliation(
          r#"<sup xml:id="id6">*</sup><text font="italic" xml:id="id7">Equal contribution.</text>"#
        ),
        affiliation(r#"<text font="italic" xml:id="id8">Chair of Things.</text>"#)
      )
    ),
  ]);
}

/// A line a footnote symbol leads, in a block marked only with footnote symbols (no mark there), is an affiliation line
/// as any other off the names line: a spacing run on it parts nothing (2609.35807's `$^{*}$\daplablogo\ Columbia
/// University\ \ \ $^{\dagger}$University at Buffalo`; probes h1, h3). Was a creator "†Univ B", "*Univ E",
/// "†University at Buffalo" each (77p r4 re-review). Each institution its symbol labels goes to the names of the group
/// carrying the symbol (77p r11, the r7 re-review's finding 1: was each line on every name of the group); a group whose
/// names carry no symbol of the line keeps the line whole (Cat Cole, Dan Dorn).
#[test]
fn symbol_led_affiliation_run_is_no_creator() {
  let xml = convert_repro!("author_symbol_led_affiliation_run.tex");
  assert_no_merged_names(&xml);
  let reversed = |ids: [u32; 2]| {
    format!(
      "{} {}",
      affiliation("Univ C"),
      affiliation(&format!(
        r#"<sup xml:id="id{}">†</sup>Univ D <sup xml:id="id{}">*</sup>Univ E"#,
        ids[0], ids[1]
      ))
    )
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">*</sup>"#,
      &affiliation("Univ A")
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">†</sup>"#,
      &affiliation("Univ B")
    ),
    creator(false, "Cat Cole", &reversed([3, 4])),
    creator(false, "Dan Dorn", &reversed([5, 6])),
    creator(
      false,
      r#"Eve Eads<sup xml:id="id7">*</sup>"#,
      &affiliation("Columbia University")
    ),
    creator(
      false,
      r#"Fay Fox<sup xml:id="id8">†</sup>"#,
      &affiliation("University at Buffalo")
    ),
  ]);
}

/// Over a line of e-mail addresses a row parted by spacing runs is names only when an address spells one of its pieces
/// or the addresses are as many as the names of both rows (`row_of_names`): institutions over the authors' addresses
/// stay the affiliation of the names above (probes v1, v3: were creators "Carnegie Mellon", "Johns Hopkins", "Google
/// DeepMind", 77p r4 re-review); `\{ffox,hhunt\}@x.edu` and four addresses for four names keep their rows names. The
/// trade: a real second row of names over an e-mail line with fewer addresses than names, none spelling a piece of the
/// row, reads as one affiliation too (the r5 re-review's probes n1-n4, n10; int34a the same) — the same structure as v3,
/// which no address rule tells apart (SYNC_STATUS address-row row). Two residuals are pinned here, not what the PDF
/// shows: "Carnegie Mellon Johns Hopkins" is one affiliation on both Ann Able and Bob Baker (the PDF prints one under
/// each). `p1@`-`p4@`, which no name spells, are the block's, on the shared creator (user ruling R10 2026-10-20; 77p r9
/// put them on the last name, Max Moss).
#[test]
fn institution_row_over_an_email_line_is_no_names() {
  let xml = convert_repro!("author_institution_row_over_email_line.tex");
  assert_no_merged_names(&xml);
  let two = affiliation("Carnegie Mellon Johns Hopkins");
  let three = affiliation("Carnegie Mellon Johns Hopkins Google DeepMind");
  let p_emails = (6..=9)
    .map(|n| typewriter_email(&format!("p{}@x.edu", n - 5), &format!("id{n}")))
    .collect::<Vec<_>>()
    .join(" ");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{two} {}", typewriter_email("aa@x.edu", "id1"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{two} {}", typewriter_email("bb@x.edu", "id2"))
    ),
    creator(
      false,
      "Cat Cole",
      &format!("{three} {}", typewriter_email("cat@x.edu", "id3"))
    ),
    creator(false, "Dan Dorn", &three),
    creator(false, "Eve Eads", &three),
    creator(false, "Fay Fox", &typewriter_email("ffox@x.edu", "id4")),
    creator(false, "Gil Gray", ""),
    creator(false, "Hal Hunt", &typewriter_email("hhunt@x.edu", "id5")),
    creator(false, "Ivy Ives", ""),
    creator(false, "Jon Jay", ""),
    creator(false, "Kim Kerr", ""),
    creator(false, "Lee Lowe", ""),
    creator(false, "Max Moss", ""),
    shared(&p_emails),
  ]);
}

/// On a marked e-mail line every address a name settles goes to that name, and only the addresses no name settles are
/// the block's, on the shared creator (user ruling R10 2026-10-20; 77p r5-r9 kept them one contact of the group's last
/// name; main-session decision 2026-10-20, 77p r5; 2609.15028's
/// `zihanxue@…`, `d09944015@…`, `serhii@…`, probe z2; 2609.05913, 2609.37829, 2609.38070). Was the whole line one
/// contact of Dan Dorn.
#[test]
fn settled_addresses_keep_their_owners() {
  let xml = convert_repro!("author_email_settled_addresses_keep_owners.tex");
  assert_no_merged_names(&xml);
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  let (a, b) = (affiliation("Univ A"), affiliation("Univ B"));
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &format!("{a} {}", email("aable@x.edu"))),
    creator(
      false,
      "Bob Baker",
      &format!("{b} {}", email("bbaker@x.edu"))
    ),
    creator(false, "Cat Cole", &format!("{a} {}", email("ccole@x.edu"))),
    creator(false, "Dan Dorn", &format!("{b} {}", email("ddorn@x.edu"))),
    shared(&email("qq@x.edu")),
  ]);
}

/// The order path of a marked e-mail line gives no address to an author another address of the block spells (2609.34459:
/// `mayhhu@…` is not Yuanyang Zhu's by order, his own address on the next line), nor one no name spells by its place
/// (user ruling R10 2026-10-20: `2796624556qw@…` first in its line, and so `mayhhu@…`, which elimination reached only
/// through it, are the block's, on the shared creator; 77p r9 put them on Yijie Sun and Yaohua Hu); probe y1's
/// `\{xx,yy\}@x.edu` over `ann.able@…`, spelling no one, the block's too (77p r9: Ann Able's and Bob Baker's by order).
#[test]
fn email_order_path_respects_spelled_owners() {
  let xml = convert_repro!("author_email_sequence_spelled_elsewhere.tex");
  assert_no_merged_names(&xml);
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  let (nanjing, hk, shenzhen) = (
    affiliation(" Nanjing University"),
    affiliation(" Hong Kong Polytechnic University"),
    affiliation(" Shenzhen University"),
  );
  assert_eq!(creators(&xml), [
    creator(true, "Yijie Sun", &nanjing),
    creator(
      false,
      "Sanquan Sun",
      &format!("{hk} {}", email("sanquan.sun@connect.polyu.hk"))
    ),
    creator(
      false,
      "Yanda Zhu",
      &format!("{nanjing} {}", email("yandazhu@smail.nju.edu.cn"))
    ),
    creator(
      false,
      "Yuanyang Zhu",
      &format!("{nanjing} {}", email("yuanyangzhu@nju.edu.cn"))
    ),
    creator(false, "Yaohua Hu", &shenzhen),
    creator(
      false,
      "Chunlin Chen",
      &format!("{nanjing} {}", email("clchen@nju.edu.cn"))
    ),
    shared(&format!(
      "{} {}",
      email("2796624556qw@gmail.com"),
      email("mayhhu@szu.edu.cn")
    )),
  ]);
  let xml = convert_repro!("author_email_sequence_more_addresses.tex");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {}",
        affiliation("Univ A"),
        typewriter_email("ann.able@gmail.com", "id1")
      )
    ),
    creator(false, "Bob Baker", &affiliation("Univ B")),
    shared(&format!(
      "{} {}",
      typewriter_email("xx@x.edu", "id2"),
      typewriter_email("yy@x.edu", "id3")
    )),
  ]);
}

/// The word "and" in a run between two marked affiliation entries parts them as the run does (probe w10: was "Univ A
/// and" Ann Able's affiliation).
#[test]
fn and_run_between_marked_entries() {
  let xml = convert_repro!("author_and_run_between_marked_entries.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
  ]);
}

/// An e-mail address goes to a name by elimination only while no other e-mail line of the block holds an address no
/// name spells: `q77@` is not Cat Cole's while `z99@` on the next line may be hers (the r5 re-review's probe q6). The
/// addresses no name settles are the block's, on the shared creator (user ruling R10 2026-10-20; 77p r9 put them on
/// the last name); every spelled one on its owner.
#[test]
fn email_elimination_waits_on_the_other_lines() {
  let xml = convert_repro!("author_email_elimination_across_lines.tex");
  assert_no_merged_names(&xml);
  let (a, b) = (affiliation("Univ A"), affiliation("Univ B"));
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{a} {}", typewriter_email("aable@x.edu", "id1"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{b} {}", typewriter_email("bbaker@y.edu", "id2"))
    ),
    creator(false, "Cat Cole", &b),
    creator(
      false,
      "Dan Dorn",
      &format!("{a} {}", typewriter_email("ddorn@x.edu", "id3"))
    ),
    shared(&format!(
      "{} {}",
      typewriter_email("q77@x.edu", "id4"),
      typewriter_email("z99@y.edu", "id5")
    )),
  ]);
}

/// The run of addresses between two spelled owners counts every author between them, spelled on another line or not
/// (2602.23777, the r5 re-review's probe q4): `nnwang@` between `xu_zhipeng@` and `dcheng@` has three authors between
/// their owners, so it is not Dongsheng Li's, nor `dongsli@` then Nannan Wang's by elimination; both are the block's,
/// on the shared creator (user ruling R10 2026-10-20; 77p r9 put them on the last name).
#[test]
fn email_gap_counts_every_author() {
  let xml = convert_repro!("author_email_gap_counts_every_author.tex");
  assert_no_merged_names(&xml);
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  let (xidian, msra) = (
    affiliation("Xidian University, "),
    affiliation("Microsoft Research Asia "),
  );
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Zhipeng Xu",
      &format!("{xidian} {}", email("xu_zhipeng@stu.xidian.edu.cn"))
    ),
    creator(
      false,
      "Zilong Wang",
      &format!("{msra} {}", email("wangzilong@microsoft.com"))
    ),
    creator(
      false,
      "Xinyang Jiang",
      &format!("{msra} {}", email("xinyangjiang@microsoft.com"))
    ),
    creator(false, "Dongsheng Li", &msra),
    creator(
      false,
      "De Cheng",
      &format!(
        "{xidian} {} {}",
        affiliation("NEL "),
        email("dcheng@xidian.edu.cn")
      )
    ),
    creator(false, "Nannan Wang", &xidian),
    shared(&format!(
      "{} {}",
      email("nnwang@xidian.edu.cn"),
      email("dongsli@microsoft.com")
    )),
  ]);
}

/// A name scores the better of its words spelled apart and a joined spelling, never their sum (2602.07835, the r5
/// re-review's probe k3): `khan.muhammad@` scores 12 for both names and goes to the surname's spelling, Khan Muhammad.
#[test]
fn email_score_is_the_best_spelling() {
  let xml = convert_repro!("author_email_score_max_not_sum.tex");
  assert_no_merged_names(&xml);
  let mbzuai = affiliation("MBZUAI");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Khan Muhammad",
      &format!(
        r#"{mbzuai} <contact name="Email: " role="email"><text font="typewriter" fontsize="90%" xml:id="id1">khan.muhammad@ieee.org</text></contact>"#
      )
    ),
    creator(false, "Muhammad Haris Khan", &mbzuai),
  ]);
}

/// Under an e-mail line, a line that is not addresses is its own entry, welded into neither the e-mail line nor the
/// affiliation above; a link line, braced and sized (`{\footnotesize \url{…}}`, 2602.22394, the r5 re-review's probes q1,
/// q3) or after a label (`Demo Code:~\url{…}`, 2602.09509), included. Was every address one contact of Sibei Yang. Each
/// link line is a URL contact of the whole block, the shared creator below the authors, its label its text (user ruling
/// 2026-10-20, 77p r7; 2605.06223, 2602.08439, 2604.12896); r6 made each an affiliation of Sibei Yang.
#[test]
fn link_line_under_an_email_line_is_its_own_entry() {
  let xml = convert_repro!("author_link_line_under_email_line.tex");
  assert_no_merged_names(&xml);
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Cheng Shi",
      &format!("{univ_a} {}", email("shicheng2025@x.hk"))
    ),
    creator(
      false,
      "Yizhou Yu",
      &format!("{univ_a} {}", email("yizhouy@acm.org"))
    ),
    creator(
      false,
      r#"Sibei Yang<sup xml:id="id1">†</sup>"#,
      &format!("{} {}", affiliation("Univ B"), email("yangsb3@y.cn"))
    ),
    shared_urls(&[
      &link(r#" fontsize="80%""#, "https://github.com/a/b"),
      &format!("Demo Code: {}", link("", "https://github.com/c/d")),
    ]),
  ]);
}

/// A URL contact (`role=url`).
fn url(content: &str) -> String {
  format!(r#"<contact name="URL: " role="url">{content}</contact>"#)
}

/// A `\url` link as the guards' conversions write it (url.sty, no hyperref), in `size` (an attribute or empty).
fn link(size: &str, url: &str) -> String {
  format!(r#"<ref class="ltx_nolink ltx_url" font="typewriter"{size} href="{url}">{url}</ref>"#)
}

/// The shared creator below the authors (OXIDIZED_DESIGN #159), holding the block's URL contacts.
fn shared_urls(contents: &[&str]) -> String {
  let contacts: Vec<String> = contents.iter().map(|content| url(content)).collect();
  format!(
    r#"<creator role="author"> {} </creator>"#,
    contacts.join(" ")
  )
}

/// A link line under an unmarked block's one group of names is a URL contact of the whole block, the shared creator
/// below the authors, its size kept; the institution on each name (user ruling 2026-10-20, 77p r7). Was the link an
/// affiliation of each name.
#[test]
fn link_line_of_an_unmarked_block_is_the_blocks_url() {
  let xml = convert_repro!("author_link_line_url_contact_unmarked.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    shared_urls(&[&link(r#" fontsize="90%""#, "https://project.example.org")]),
  ]);
}

/// In an unmarked block's `\and` columns, a link line is a URL contact of its column's names, as the column's
/// institution is: Ann Able's own; Bob Baker's and Cat Cole's both, "Code:" its text (user ruling 2026-10-20, 77p r7).
/// Was each an affiliation.
#[test]
fn link_line_of_an_and_column_is_its_names_url() {
  let xml = convert_repro!("author_link_line_url_contact_per_group.tex");
  assert_no_merged_names(&xml);
  let univ_b = affiliation("Univ B");
  let lab_b = url(&format!("Code: {}", link("", "https://lab-b.example.org")));
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {}",
        affiliation("Univ A"),
        url(&link("", "https://ann.example.org"))
      )
    ),
    creator(false, "Bob Baker", &format!("{univ_b} {lab_b}")),
    creator(false, "Cat Cole", &format!("{univ_b} {lab_b}")),
  ]);
}

/// In a marked block's `\and` columns of one author each, a link line is the URL contact of its column's author, who
/// the PDF prints it under, not of the whole block (user ruling 2026-10-20, 77p r7). Was an affiliation of Ann Able.
#[test]
fn link_line_of_a_marked_column_is_its_authors_url() {
  let xml = convert_repro!("author_link_line_url_contact_marked_column.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">*</sup>"#,
      &format!(
        "{} {}",
        affiliation("Univ A"),
        url(&link("", "https://ann.example.org"))
      )
    ),
    creator(false, "Bob Baker", &affiliation("Univ B")),
  ]);
}

/// A command LaTeX's `\@fnsymbol` prints is a footnote symbol as `\S` is (2601.02680's `$^\mathsection$`; the r5
/// re-review's probes r3, r5): shown on its name, and the run after it parts the names. Was "Cat Cole" and "Dan Dorn
/// Eve Eads†Univ A", the § dropped.
#[test]
fn fnsymbol_command_is_a_symbol_mark() {
  let xml = convert_repro!("author_symbol_command_marks.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, r#"Cat Cole<sup xml:id="id1">§</sup>"#, &univ_a),
    creator(false, r#"Dan Dorn<sup xml:id="id2">§</sup>"#, &univ_a),
    creator(false, r#"Eve Eads<sup xml:id="id3">†</sup>"#, &univ_a),
  ]);
}

/// A footnote symbol leading an affiliation line under another affiliation, the symbol an author carries, is its own
/// entry labelled by its symbol (2601.05633's ⋄ line under its ♡ line, the r5 re-review's probe d1): Univ C on Cat Cole,
/// who carries ⋄ though not last. Was "Univ B⋄ Univ C" on Ann Able.
#[test]
fn symbol_led_affiliation_is_labelled_by_its_symbol() {
  let xml = convert_repro!("author_symbol_led_affiliation_labelled.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation(" Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", affiliation(" Univ B"))
    ),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id1">⋄</sup>"#,
      &affiliation(" Univ C")
    ),
    creator(false, "Bob Baker", &univ_a),
  ]);
}

/// An e-mail contact named by its line's label (`Correspondence: `), the address kept the link `\href` makes of it.
fn mailto_email(label: &str, target: &str, address: &str) -> String {
  format!(
    r#"<contact name="{label}: " role="email"><ref class="ltx_href" href="mailto:{target}">{address}</ref></contact>"#
  )
}

/// A `\textbf{Correspondence:} \href{mailto:…}{…}` line under a marked block is an e-mail line: its address is an
/// e-mail contact of the name it spells, Bob Baker, named "Correspondence:" and kept a link to the target as written
/// (the ACL template's placeholder `mailto:email@domain`, 2601.07974); 2601.00240, 2604.23002, 2606.03014 (user ruling
/// 2026-10-20, 77p r8). Was an affiliation of the last author, Cat Cole.
#[test]
fn mailto_line_of_a_marked_block_is_the_spelled_names_email() {
  let xml = convert_repro!("author_mailto_line_email_marked.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{univ_a} {}",
        mailto_email("Correspondence", "email@domain", "bbaker@uni-b.edu")
      )
    ),
    creator(false, "Cat Cole", &affiliation("Univ B")),
  ]);
}

/// A `\textbf{Correspondence:} \href{mailto:…}{…}` line under an unmarked block's one group of names is an e-mail line:
/// its address goes to the name it spells alone, the institution to every name (2605.18563; user ruling 2026-10-20, 77p
/// r8). Was an affiliation of all three names. Bob Baker's contacts in the order printed, the institution first (77p r9,
/// the r5 re-review's finding 8).
#[test]
fn mailto_line_of_an_unmarked_block_leaves_the_other_names() {
  let xml = convert_repro!("author_mailto_line_email_unmarked.tex");
  assert_no_merged_names(&xml);
  let univ_b = affiliation("Univ B");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_b),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{univ_b} {}",
        mailto_email("Correspondence", "bbaker@uni-b.edu", "bbaker@uni-b.edu")
      )
    ),
    creator(false, "Cat Cole", &univ_b),
  ]);
}

/// A mailto line whose address no name spells is never guessed onto a name: one e-mail contact of the block, on the
/// shared creator (user ruling R10 2026-10-20; 77p r8 placed it on the group's last name). Was an affiliation of all
/// three names.
#[test]
fn mailto_line_spelling_no_name_is_one_contact() {
  let xml = convert_repro!("author_mailto_line_email_unspelled.tex");
  assert_no_merged_names(&xml);
  let univ_b = affiliation("Univ B");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_b),
    creator(false, "Bob Baker", &univ_b),
    creator(false, "Cat Cole", &univ_b),
    shared(&mailto_email(
      "Correspondence",
      "lab@uni-b.edu",
      "lab@uni-b.edu"
    )),
  ]);
}

/// A `\url` link as hyperref writes it.
fn hlink(url: &str) -> String {
  format!(r#"<ref class="ltx_url" font="typewriter" href="{url}">{url}</ref>"#)
}

/// A colour command's colour is no letter of an address (the r5 re-review's finding 1, probe c3; 2604.19440): each
/// address of `\texttt{\{\color{black}aable, bbaker\}@x.edu}` the e-mail of the name it spells. Was
/// "blackaable@x.edu" on Ann Able.
#[test]
fn colour_in_an_address_list_is_no_letter() {
  let xml = convert_repro!("author_email_colour_in_brace_list.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", typewriter_email("aable@x.edu", "id1"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", typewriter_email("bbaker@x.edu", "id2"))
    ),
  ]);
}

/// 2604.19440's address line, a mailto link around each local part set `\color{black}` (the r5 re-review's finding 1,
/// probes c5, c6): each address the e-mail of the name it spells, kept the link its line gives it; the project line a
/// URL contact of the block. Was "blackxinhao.zhang@…" and "blackmaxime.peyrard@…", the links lost. The empty
/// `\textsuperscript{}` before the institution is no mark (user ruling R11 2026-10-20): the block reads as unmarked, the
/// institution every name's (77p r9 pinned it on the last name only).
#[test]
fn colour_in_mailto_links_is_no_letter() {
  let xml = convert_repro!("author_email_colour_in_mailto_links.tex");
  assert_no_merged_names(&xml);
  let mailto = |target: &str, address: &str| {
    format!(
      r#"<contact name="Email: " role="email"><ref class="ltx_href" href="mailto:{target}">{address}</ref></contact>"#
    )
  };
  let grenoble =
    affiliation("Univ. Grenoble Alpes, CNRS, Grenoble INP, LIG, 38000 Grenoble, France");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Xinhao Zhang",
      &format!(
        "{grenoble} {}",
        mailto(
          "Xinhao.Zhang@univ-grenoble-alpes.fr",
          "xinhao.zhang@univ-grenoble-alpes.fr"
        )
      )
    ),
    creator(false, "Xi Chen", &grenoble),
    creator(false, "François Portet", &grenoble),
    creator(
      false,
      "Maxime Peyrard",
      &format!(
        "{grenoble} {}",
        mailto(
          "maxime.peyrard@univ-grenoble-alpes.fr",
          "maxime.peyrard@univ-grenoble-alpes.fr"
        )
      )
    ),
    shared_urls(&[
      r##"Project Website: <ref class="ltx_href" color="#0000FF" href="https://xinhao-zhang.github.io/traj_evo_search/">github.io/traj_evo_search</ref>"##
    ]),
  ]);
}

/// A link printing an institution is that institution linked, not a URL contact (2606.26479, the r5 re-review's
/// finding 2, probes u32, u35): "LaunchSafe Research" an affiliation of all four names, the link inside it. Was a URL
/// contact of the nameless shared creator.
#[test]
fn linked_institution_is_an_affiliation() {
  let xml = convert_repro!("author_linked_institution_is_an_affiliation.tex");
  assert_no_merged_names(&xml);
  let launchsafe = affiliation(
    r#"<ref class="ltx_href" font="bold" href="https://launchsafe.com">LaunchSafe Research</ref>"#,
  );
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Praneeth Narisetty",
      &format!("{launchsafe} {}", email("praneeth@launchsafe.com"))
    ),
    creator(
      false,
      "Shiva Kore",
      &format!("{launchsafe} {}", email("shiva@launchsafe.com"))
    ),
    creator(
      false,
      "Uday Kattamanchi",
      &format!("{launchsafe} {}", email("uday@launchsafe.com"))
    ),
    creator(
      false,
      "Jayaram Kumarapu",
      &format!("{launchsafe} {}", email("jayaram@launchsafe.com"))
    ),
  ]);
}

/// Each `\and` column's linked institution is its name's affiliation (the r5 re-review's probe u33). Was a URL contact
/// of each name.
#[test]
fn linked_institutions_per_column_are_affiliations() {
  let xml = convert_repro!("author_linked_institutions_per_column.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &affiliation(
        r#"<ref class="ltx_href" href="https://www.cmu.edu">Carnegie Mellon University</ref>"#
      )
    ),
    creator(
      false,
      "Bob Baker",
      &affiliation(r#"<ref class="ltx_href" href="https://www.mit.edu">MIT</ref>"#)
    ),
  ]);
}

/// The ACL template's `\textbf{Correspondence:} \href{x@y}{x@y}`, a link to a bare address without `mailto:`, is an
/// e-mail line (2604.04847, 2605.29064, 2605.29893, 2606.01600; the r5 re-review's finding 3): the address Bob Baker's,
/// the name it spells, the link as written. Was a URL contact of the shared creator.
#[test]
fn correspondence_link_to_a_bare_address_is_an_email() {
  let xml = convert_repro!("author_correspondence_bare_address_link.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!(
        r#"{univ_a} <contact name="Correspondence: " role="email"><ref class="ltx_href" href="bbaker@x.edu">bbaker@x.edu</ref></contact>"#
      )
    ),
    creator(false, "Cat Cole", &affiliation("Univ B")),
  ]);
}

/// A link line in a marked `\and` column of two names is both names' URL contact, after their institution, as the
/// column prints it (ruling 9, per-column link lines; the r5 re-review's finding 4, probe u6m). Was on the nameless
/// shared creator.
#[test]
fn link_line_of_a_marked_column_of_names_is_their_url() {
  let xml = convert_repro!("author_link_line_of_a_marked_column_of_names.tex");
  assert_no_merged_names(&xml);
  let ann_url = format!("{} {}", affiliation("Univ A"), url(&hlink("https://a.org")));
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &ann_url),
    creator(false, "Bob Baker", &ann_url),
    creator(false, "Cat Cole", &affiliation("Univ B")),
  ]);
}

/// A run of addresses no name spells at the end of a line's list is not read by the order while another e-mail line
/// of the block holds an address no name spells (the r5 re-review's finding 5, probe q7's end run): `q77@` is not Ann
/// Able's; it and `z99@` stay unsettled, the block's, on the shared creator (user ruling R10 2026-10-20: no position
/// placement at a list's end; 77p r9 put both on the group's last name). Was `q77@` on Ann Able.
#[test]
fn email_gap_at_the_list_end_waits_on_the_other_lines() {
  let xml = convert_repro!("author_email_gap_at_the_list_end_waits.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", typewriter_email("bbaker@x.edu", "id1"))
    ),
    creator(
      false,
      "Cat Cole",
      &format!("{univ_a} {}", typewriter_email("ccole@y.edu", "id2"))
    ),
    shared(&format!(
      "{} {}",
      typewriter_email("q77@x.edu", "id3"),
      typewriter_email("z99@y.edu", "id4")
    )),
  ]);
}

/// Names a spacing run parts on one row, marked by footnote symbols that label the lines under the row (2601.02680; the
/// r5 re-review's finding 6, probes r10, r11, r18): each institution goes to the names carrying its symbol, the address
/// to the name it spells, the repository link to the block, the empty `\url{}` beside it nothing. Was every line on the
/// last name, Dong Li.
#[test]
fn symbol_marked_row_lines_go_by_their_marks() {
  let xml = convert_repro!("author_symbol_marked_spacing_run_lines.tex");
  assert_no_merged_names(&xml);
  let (tsinghua, merced) = (
    affiliation("Tsinghua University"),
    affiliation("University of California, Merced"),
  );
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Dinghong Song<sup xml:id="id1">†</sup>"#,
      &format!(
        r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" role="thanks" xml:id="id2">Equal contribution</note> {merced} {}"#,
        typewriter_email("dinghongsong21@gmail.com", "id3")
      )
    ),
    creator(false, r#"Zhiwei Xu<sup xml:id="id4">§</sup>"#, &tsinghua),
    creator(false, r#"Hai Wan<sup xml:id="id5">§</sup>"#, &tsinghua),
    creator(false, r#"Xibin Zhao<sup xml:id="id6">§</sup>"#, &tsinghua),
    creator(false, r#"Pengfei Su<sup xml:id="id7">†</sup>"#, &merced),
    creator(false, r#"Dong Li<sup xml:id="id8">†</sup>"#, &merced),
    shared_urls(&[
      r##"<ref class="ltx_href" color="#FF00FF" font="typewriter" href="https://github.com/dinghongsong/ACL">https://github.com/dinghongsong/ACL</ref>"##
    ]),
  ]);
}

/// A link line opening an author block or an `\and` column is no name (the r5 re-review's finding 7, probes u4, u16):
/// both URLs contacts of the block. Was two creators named by their URLs.
#[test]
fn link_line_opening_a_group_is_no_name() {
  let xml = convert_repro!("author_link_line_opening_a_group.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", &affiliation("Univ B")),
    shared_urls(&[&hlink("https://x.org"), &hlink("https://y.org")]),
  ]);
}

/// A link line opening an unmarked block is no name (the r5 re-review's probe u17): Ann Able the author, the URL and her
/// institution her contacts, in the order printed. Was a creator named "https://x.org" holding "Ann Able".
#[test]
fn link_line_opening_an_unmarked_block_is_no_name() {
  let xml = convert_repro!("author_link_line_opening_an_unmarked_block.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [creator(
    true,
    "Ann Able",
    &format!("{} {}", url(&hlink("https://x.org")), affiliation("Univ A"))
  )]);
}

/// A marked column's contacts in the order the column prints them, its institution, then its URL (the r5 re-review's
/// finding 8, probe u14). Was the URL first.
#[test]
fn link_line_after_a_marked_affiliation_keeps_the_order() {
  let xml = convert_repro!("author_link_line_after_marked_affiliation_order.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{} {}", affiliation("Univ A"), url(&hlink("https://a.org")))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{} {}", affiliation("Univ B"), url(&hlink("https://b.org")))
    ),
  ]);
}

/// A labelled line of several linked addresses is an e-mail line: each address the name's it spells, under the line's
/// label, a link (the r5 re-review's residual 9, probe u22). Was welded into Univ A on both names.
#[test]
fn labelled_list_of_linked_addresses_is_an_email_line() {
  let xml = convert_repro!("author_email_row_labels_and_link_lists.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{univ_a} {}",
        mailto_email("Emails", "aable@x.edu", "aable@x.edu")
      )
    ),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{univ_a} {}",
        mailto_email("Emails", "bbaker@x.edu", "bbaker@x.edu")
      )
    ),
  ]);
}

/// A footnote symbol before an e-mail line's label is the line's mark (`$^*$Corresponding author: \href{mailto:…}`,
/// the r5 re-review's residual 9, probe u23): the address Ann Able's, the name it spells. Was welded into Univ A on both
/// names.
#[test]
fn symbol_led_email_label_is_an_email_line() {
  let xml = convert_repro!("author_email_row_symbol_led_label.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">∗</sup>"#,
      &format!(
        "{univ_a} {}",
        mailto_email("Corresponding author", "aable@x.edu", "aable@x.edu")
      )
    ),
    creator(false, "Bob Baker", &univ_a),
  ]);
}

/// An e-mail line set in a size and a group, `\tt\small {Email Address: …}` (2603.19228; the r5 re-review's residual 9):
/// the address Yuxin Song's, the name it spells, named "Email Address:"; the project page a URL contact of the block.
/// Was an affiliation of the last name, Jingdong Wang.
#[test]
fn email_line_in_a_sized_group_is_an_email_line() {
  let xml = convert_repro!("author_email_row_label_in_a_group.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Xinyao Zhang", &affiliation("Baidu")),
    creator(
      false,
      "Yuxin Song",
      &format!(
        r#"{} <contact name="Email Address: " role="email">songyuxinbb@outlook.com</contact>"#,
        affiliation("Baidu")
      )
    ),
    creator(false, "Jingdong Wang", &affiliation("Tsinghua University")),
    r#"<creator role="author"> <contact name="URL: " role="url"><text font="typewriter" fontsize="90%" xml:id="id1">Project Page: <ref class="ltx_url" href="https://example.github.io/SAMA">https://example.github.io/SAMA</ref></text></contact> </creator>"#.to_string(),
  ]);
}

/// The shared creator below the authors (OXIDIZED_DESIGN #159) holding `contacts`, as the squashed XML writes it.
fn shared(contacts: &str) -> String { format!(r#"<creator role="author"> {contacts} </creator>"#) }

/// An `\href` link as hyperref writes it, printing `text`.
fn href(target: &str, text: &str) -> String {
  format!(r#"<ref class="ltx_href" href="{target}">{text}</ref>"#)
}

/// An address line printed under a row of symbol-marked names goes by what it says, never by the line above it
/// (2607.16926; the r6 re-review's finding 1): each address of the `;`-separated brace list the e-mail of the name it
/// spells, Univ A on every † name, Lab B on the ∗ names. Was the whole address line an affiliation of the ∗ names, the
/// carriers of the institution above it.
#[test]
fn symbol_row_brace_list_goes_by_spelling() {
  let xml = convert_repro!("author_symbol_row_brace_list_under_the_row.tex");
  assert_no_merged_names(&xml);
  let (univ_a, lab_b) = (affiliation(" Univ A"), affiliation(" Lab B"));
  // (the address as the list spells it; the group's `\tt\small` around the line is no wrapper an address keeps)
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">†</sup><sup xml:id="id2">∗</sup>"#,
      &format!("{univ_a} {lab_b} {}", email("aable@x.edu"))
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id3">†</sup>"#,
      &format!("{univ_a} {}", email("bbaker@x.edu"))
    ),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id4">†</sup><sup xml:id="id5">∗</sup>"#,
      &format!("{univ_a} {lab_b} {}", email("ccole@x.edu"))
    ),
  ]);
}

/// A note under a row of symbol-marked names is the block's, not the carriers' of the institution above it (the r6
/// re-review's finding 1, probe y7). Was "Equal contribution" an affiliation of Bob Baker.
#[test]
fn symbol_row_note_line_is_the_blocks() {
  let xml = convert_repro!("author_symbol_row_note_line_is_the_blocks.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">†</sup>"#,
      &affiliation("Univ A")
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">‡</sup>"#,
      &affiliation("Univ B")
    ),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id3">†</sup>"#,
      &affiliation("Univ A")
    ),
    shared(&affiliation("Equal contribution")),
  ]);
}

/// An unmarked institution under a row of symbol-marked names is every name's of the row (user ruling R12 2026-10-20;
/// the r6 re-review's finding 1, probes y8, x7). Was "Department of Physics" an affiliation of Bob Baker alone.
#[test]
fn symbol_row_unmarked_institution_is_every_names() {
  let xml = convert_repro!("author_symbol_row_unmarked_institution_every_name.tex");
  assert_no_merged_names(&xml);
  let physics = affiliation("Department of Physics");
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">†</sup>"#,
      &format!("{} {physics}", affiliation("Univ A"))
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">‡</sup>"#,
      &format!("{} {physics}", affiliation("Univ B"))
    ),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id3">†</sup>"#,
      &format!("{} {physics}", affiliation("Univ A"))
    ),
  ]);
}

/// A link printing a link word ("Project Page") is a URL contact of the block, whatever the shape of its words
/// (2602.22227, 2604.24697; the r6 re-review's finding 2, probe w1). Was an affiliation of the shared creator.
#[test]
fn link_word_makes_a_url_line() {
  let xml = convert_repro!("author_link_word_is_a_url.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    shared(&url(&href("https://x.org/proj", "Project Page"))),
  ]);
}

/// A link to a bare domain printing a one-word institution is that institution linked, its column's name's affiliation
/// (the r6 re-review's finding 2, probes x10, w8). Was a URL contact of each name.
#[test]
fn linked_domain_root_is_an_institution() {
  let xml = convert_repro!("author_linked_domain_root_is_an_institution.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &affiliation(&href("https://www.google.com", "Google"))
    ),
    creator(
      false,
      "Bob Baker",
      &affiliation(&href("https://www.meta.com", "Meta"))
    ),
  ]);
}

/// An `\and` column that is only a "Project Page" link is no author: a URL contact of the block (the r6 re-review's
/// finding 2 nit, probe w22). Was a creator named "Project Page".
#[test]
fn link_column_is_no_creator() {
  let xml = convert_repro!("author_link_column_is_no_creator.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", ""),
    creator(false, "Bob Baker", ""),
    shared(&url(&href("https://x.org", "Project Page"))),
  ]);
}

/// The names carrying a symbol that labels a line under a row are the row's own, not those of another `\and` column
/// (the r6 re-review's finding 3, probes x6, w28). Was Univ A on Cat Cole too.
#[test]
fn symbol_carriers_stay_in_their_row() {
  let xml = convert_repro!("author_symbol_carriers_stay_in_their_row.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">†</sup>"#,
      &affiliation("Univ A")
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">‡</sup>"#,
      &affiliation("Univ B")
    ),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id3">†</sup>"#,
      &affiliation(r#"<sup xml:id="id4">†</sup> Univ C"#)
    ),
  ]);
}

/// A compound symbol mark (`$^{\dagger*}$`) is one label per symbol: Ann Able carries † and so has Univ A (the r6
/// re-review's finding 4, probes x1, w10, w11). Was Ann Able with no institution.
#[test]
fn compound_symbol_mark_is_a_label_per_symbol() {
  let xml = convert_repro!("author_compound_symbol_mark.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">†*</sup>"#,
      &affiliation("Univ A")
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">‡</sup>"#,
      &affiliation("Univ B")
    ),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id3">†</sup>"#,
      &affiliation("Univ A")
    ),
  ]);
}

/// A scale factor is no letter of an address (`\scalebox{0.9}{…}`; the r6 re-review's finding 5, probe w14). Was
/// "0.9aable@x.edu".
#[test]
fn scalebox_factor_is_no_address_letter() {
  let xml = convert_repro!("author_email_scalebox_argument.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  let plain =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", plain("aable@x.edu"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", plain("bbaker@x.edu"))
    ),
  ]);
}

/// A font family name is no letter of an address (`\fontfamily{pcr}\selectfont`; 2609.25669, 2609.35216; the r6
/// re-review's finding 5, probe w15). Was "pcraable@x.edu".
#[test]
fn font_family_is_no_address_letter() {
  let xml = convert_repro!("author_email_fontfamily_argument.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", typewriter_email("aable@x.edu", "id1"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", typewriter_email("bbaker@x.edu", "id2"))
    ),
  ]);
}

/// A link to a profile page printing a handle (`@aable`) is a URL line, no e-mail (the r6 re-review's finding 6, probe
/// w21). Was an e-mail contact "@aable".
#[test]
fn at_handle_link_is_a_url() {
  let xml = convert_repro!("author_at_handle_link_is_a_url.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    shared(&url(&format!(
      "Contact: {}",
      href("https://x.com/aable", "@aable")
    ))),
  ]);
}

/// A brace list of local parts set by `\url` is a list of addresses, each the e-mail of the name it spells (2609.00588;
/// the r6 re-review's finding 6). Was one e-mail contact on Bob Baker holding the whole list.
#[test]
fn url_brace_list_is_addresses_by_spelling() {
  let xml = convert_repro!("author_url_brace_list_of_addresses.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  // (each address in the line's `\texttt`, the list's one `\url` no link of any one of them)
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", typewriter_email("aable@x.edu", "id1"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", typewriter_email("bbaker@x.edu", "id2"))
    ),
    creator(
      false,
      "Cat Cole",
      &format!("{univ_a} {}", typewriter_email("ccole@x.edu", "id3"))
    ),
  ]);
}

/// Under a row whose only symbol line is a legend, the unmarked institution lines are every name's of the row (2609.24299,
/// 2603.06578; user ruling R12 2026-10-20; the r6 re-review's finding 7); the legend goes to the name carrying its
/// symbol, Bob Baker (its "Corresponding author:" label no e-mail label: the address stays in its text). Was every line on
/// Bob Baker alone.
#[test]
fn legend_row_shares_the_institution() {
  let xml = convert_repro!("author_legend_row_shares_the_institution.tex");
  assert_no_merged_names(&xml);
  let lines = |a: &str, b: &str| {
    format!(
      r#"<contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="{a}">Department of X,</text></contact> <contact name="Affiliation: " role="affiliation"><text fontsize="90%" xml:id="{b}">Univ A, City</text></contact>"#
    )
  };
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &lines("id1", "id2")),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id3">*</sup>"#,
      &format!(
        r#"{} {}"#,
        lines("id4", "id5"),
        affiliation(
          r#"<sup xml:id="id6"><text fontsize="90%" xml:id="id6.1">*</text></sup><text fontsize="90%" xml:id="id7">Corresponding author: bbaker@x.edu</text>"#
        )
      )
    ),
  ]);
}

/// An unmarked linked institution under a marked block is every author's (user ruling R12 2026-10-20; the r5
/// re-review's probe u38). Was an affiliation of the shared creator.
#[test]
fn marked_block_linked_institution_is_every_authors() {
  let xml = convert_repro!("author_marked_linked_institution_every_author.tex");
  assert_no_merged_names(&xml);
  let launchsafe = affiliation(
    r#"<ref class="ltx_href" font="bold" href="https://launchsafe.com">LaunchSafe Research</ref>"#,
  );
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{} {launchsafe}", affiliation("Univ A"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{} {launchsafe}", affiliation("Univ B"))
    ),
  ]);
}

/// An address no name spells is the block's, on the shared creator, in an unmarked block (user ruling R10 2026-10-20).
/// Was an e-mail of the last name, Bob Baker.
#[test]
fn unspelled_email_is_the_blocks_unmarked() {
  let xml = convert_repro!("author_unspelled_email_is_the_blocks_unmarked.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    shared(&typewriter_email("q77@x.edu", "id1")),
  ]);
}

/// An address no name spells is the block's in a marked block too, not placed by its position at the list's start
/// (user ruling R10 2026-10-20; 2509.10377). Was an e-mail of Ann Able.
#[test]
fn unspelled_email_is_the_blocks_marked() {
  let xml = convert_repro!("author_unspelled_email_is_the_blocks_marked.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(
      false,
      "Bob Baker",
      &format!("{univ_a} {}", typewriter_email("bbaker@x.edu", "id1"))
    ),
    creator(false, "Cat Cole", &affiliation("Univ B")),
    shared(&typewriter_email("q77@x.edu", "id2")),
  ]);
}

/// An empty mark (`\textsuperscript{}`, printing nothing) is no mark: the block reads as unmarked, its line every
/// name's (user ruling R11 2026-10-20; 2604.19440). Was the institution on Bob Baker alone.
#[test]
fn empty_mark_is_no_mark() {
  let xml = convert_repro!("author_empty_mark_is_no_mark.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
  ]);
}

/// A line led by a compound symbol mark (`$^{\dagger\ddagger}$`) names the names carrying all its symbols, a single
/// symbol every name carrying it (2602.19190; batch 77p r10 A/B): Univ A on both dagger names, "Project lead" Bob
/// Baker's alone. Was "Project lead" on Ann Able too (77p r9, r10 w2: the line read by its first symbol).
#[test]
fn compound_legend_mark_names_its_carrier() {
  let xml = convert_repro!("author_compound_legend_mark_names_its_carrier.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, r#"Ann Able<sup xml:id="id1">†</sup>"#, &univ_a),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">†‡</sup>"#,
      &format!("{univ_a} {}", affiliation("Project lead"))
    ),
  ]);
}

/// An address spells a name by its given name and surname whole (`diwu@` Di Wu's) or by the initials before the surname
/// (`kwchang@` Kai-Wei Chang's, not Ya-Liang Chang's) (2604.01354; batch 77p r10 A/B, where an address no name spells
/// is the block's). Was both on the shared creator.
#[test]
fn email_given_name_and_initials_spell() {
  let xml = convert_repro!("author_email_given_name_and_initials_spell.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Di Wu",
      &format!("{univ_a} {}", typewriter_email("diwu@cs.ucla.edu", "id1"))
    ),
    creator(false, "Siyue Liu", &univ_a),
    creator(false, "Ya-Liang Chang", &univ_a),
    creator(
      false,
      "Kai-Wei Chang",
      &format!(
        "{univ_a} {}",
        typewriter_email("kwchang@cs.ucla.edu", "id2")
      )
    ),
  ]);
}

/// An environment's `\begin{…}`/`\end{…}`, `\hfill` and `\strut` print nothing: IEEEtran's `\end{@IEEEauthorhalign}%
/// \hfill\strut` and `\begin{@IEEEauthorhalign}` lines are no entry (2309.15562; batch 77p r10 A/B). Was an empty
/// affiliation on Bob Baker (77p r9: two empty e-mail contacts on the shared creator, the environment's name read as an
/// address).
#[test]
fn ieee_halign_markers_print_nothing() {
  let xml = convert_repro!("author_ieee_halign_markers_print_nothing.tex");
  assert_no_merged_names(&xml);
  assert!(
    !xml.contains("role=\"affiliation\"/>") && !xml.contains("role=\"email\"/>"),
    "{xml}"
  );
  let univ_a = affiliation("Univ A");
  let tt = |address: &str, id: &str| {
    affiliation(&format!(
      r#"<text font="typewriter" xml:id="{id}">{address}</text>"#
    ))
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">1</text></sup>"#,
      &format!("{} {univ_a}", tt("aable@x.edu", "id2"))
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id3"><text font="italic" xml:id="id3.1">1</text></sup>"#,
      &format!("{} {univ_a}", tt("bbaker@x.edu", "id4"))
    ),
  ]);
}

/// An empty mark stays in a block other marks make a marked one, where each name it ends is an author's piece of its
/// own (2604.22129; batch 77p r10 A/B): three authors, Univ A Ann Able's. Was one creator "Ann Able Bob Baker Cat Cole"
/// (r10 w4, the empty marks dropped there too); 77p r9 the same as now.
#[test]
fn empty_mark_in_a_marked_block_stays() {
  let xml = convert_repro!("author_empty_mark_in_a_marked_block_stays.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &affiliation("Univ A")),
    creator(false, "Bob Baker", ""),
    creator(false, "Cat Cole", ""),
  ]);
}

/// A symbol-labelled line set in a group (`{\small $^{\dagger}$\,University of Toronto \qquad $^{\ddagger}$\,EPFL}`,
/// 2606.28166; the r7 re-review's finding 1, probe g1) is read through the group: each institution goes to the names
/// carrying its symbol, and the starred contact line under it, a legend, to the names carrying ∗. Was the whole line one
/// affiliation of every name of the row and the contact line the block's (77p r10; r9 both the last name's).
#[test]
fn symbol_row_group_wrapped_line_goes_by_its_marks() {
  let xml = convert_repro!("author_symbol_row_group_wrapped_line.tex");
  assert_no_merged_names(&xml);
  let (univ_a, univ_b) = (affiliation(" Univ A"), affiliation(" Univ B"));
  let contact = |id: usize| {
    affiliation(&format!(
      r#"<sup xml:id="id{id}"><text fontsize="90%" xml:id="id{id}.1">*</text></sup><text fontsize="90%" xml:id="id{}"> Contact: <text font="typewriter" xml:id="id{}.1">{{aable, ccole}}@x.edu</text></text>"#,
      id + 1,
      id + 1
    ))
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able <sup xml:id="id1">†*</sup>"#,
      &format!("{univ_a} {}", contact(2))
    ),
    creator(false, r#"Bob Baker <sup xml:id="id4">‡</sup>"#, &univ_b),
    creator(
      false,
      r#"Cat Cole <sup xml:id="id5">†*</sup>"#,
      &format!("{univ_a} {}", contact(6))
    ),
  ]);
}

/// Symbol-labelled lines each set in a group (`{\it \normalsize $^\sharp$Department of Biostatistics, …}`, 2602.16540;
/// the r7 re-review's finding 1, probe g3) go to the names carrying their symbols. Was each line an affiliation of every
/// name of the row (77p r10).
#[test]
fn symbol_row_group_wrapped_lines_go_by_their_marks() {
  let xml = convert_repro!("author_symbol_row_group_wrapped_lines.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A, Dublin");
  assert_eq!(creators(&xml), [
    creator(true, r#"Ann Able<sup xml:id="id1">†</sup>"#, &univ_a),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">‡</sup>"#,
      &affiliation("Department of Biostatistics, Univ B")
    ),
    creator(false, r#"Cat Cole<sup xml:id="id3">†</sup>"#, &univ_a),
  ]);
}

/// 2602.16540's block (the r7 re-review's finding 1): names marked ⋆ and ♯, a `\hspace` before the line break, each
/// institution on a line of its own in a group, led by its mark — two authors, each with the institution its mark
/// labels. Was a creator named "⋆School of Mathematics …" (int34a, 77p r9) and the ♯ line on all three (77p r10).
#[test]
fn symbol_lines_after_a_spaced_break_go_by_their_marks() {
  let xml = convert_repro!("author_symbol_lines_after_spaced_break.tex");
  assert_no_merged_names(&xml);
  // (each footnote an e-mail note of its name, before its institution; batch 77y: whole elements, the r8 re-review's
  // finding 12)
  let note = |address: &str, id: &str| {
    format!(
      r##"<contact name="Note: " role="note">E-mail: <text color="#008080" font="typewriter" xml:id="{id}">{address}</text></contact>"##
    )
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">⋆</sup>"#,
      &format!(
        "{} {}",
        note("aable@x.ie", "id2"),
        affiliation("School of Mathematics and Statistics, Univ A, Dublin 4, Republic of Ireland")
      )
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id3">♯</sup>"#,
      &format!(
        "{} {}",
        note("bbaker@y.hk", "id4"),
        affiliation("Department of Biostatistics, Univ B, Hong Kong")
      )
    ),
  ]);
}

/// A mark after its institution labels it (`Carnegie Mellon University, …$^{\dagger}$`, 2109.13296; the r7 re-review's
/// finding 1, probes p7, p8): Univ A on the † names, Univ B on the ‡ one, each without its mark. Was each line an
/// affiliation of every name of the row (77p r10).
#[test]
fn trailing_symbol_mark_labels_its_line() {
  let xml = convert_repro!("author_symbol_row_trailing_marks.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, r#"Ann Able<sup xml:id="id1">†</sup>"#, &univ_a),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">‡</sup>"#,
      &affiliation("Univ B")
    ),
    creator(false, r#"Cat Cole<sup xml:id="id3">†</sup>"#, &univ_a),
  ]);
}

/// 2109.13296's block (the r7 re-review's findings 1 and 9): the unmarked institution and its brace list under three
/// `\hspace`-parted names, then an institution and an address each marked † after the text — The Univ A on every name
/// (R12), each listed address on the name it spells, Univ B and `bakerbob@` on Bob Baker, the † name, alone. Was Univ B
/// on all three and `bakerbob@gmail.com†` on the shared creator (77p r10).
#[test]
fn trailing_marks_under_spaced_names_label_their_lines() {
  let xml = convert_repro!("author_trailing_mark_under_spaced_names.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("The Univ A, City, USA");
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  let tt = |address: &str, id: &str| typewriter_email(address, id);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", email("aable@a.edu"))
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id1">†</sup>"#,
      &format!(
        "{univ_a} {} {}",
        affiliation("Univ B, Pittsburgh, PA, USA"),
        tt("bakerbob@gmail.com", "id2")
      )
    ),
    creator(
      false,
      "Cat Cole",
      &format!("{univ_a} {}", email("ccole@a.edu"))
    ),
  ]);
}

/// A linked name with compact dotted initials is a name, not a web address (`\href{}{…Edward A.G. Schuur}`, 2601.06085;
/// the r7 re-review's finding 2): two authors, each with the institution under it. Was no "Edward A.G. Schuur" and six
/// creators named by the pieces of his institution, each with the link as a URL contact (77p r9, r10).
#[test]
fn dotted_initials_link_is_a_name() {
  let xml = convert_repro!("author_link_dotted_initials_are_a_name.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"<ref class="ltx_href" href="https://orcid.org/0000-0001"> Ann Able</ref>"#,
      &affiliation("Univ A")
    ),
    creator(
      false,
      r#"<ref class="ltx_href"> Edward A.G. Schuur</ref>"#,
      &affiliation(
        "Center for Ecosystem Science and Society, Northern Arizona University, Flagstaff, Arizona, USA"
      )
    ),
  ]);
}

/// authblk's `\author[1]{\href{https://orcid.org/…}{T.J.C. Bezerra}}` (2606.03745; the r7 re-review's finding 2) is an
/// author, Sussex his affiliation. Was a URL contact of the next author, the author lost (77p r9, r10).
#[test]
fn dotted_initials_link_is_an_authblk_name() {
  let xml = convert_repro!("author_link_dotted_initials_authblk.tex");
  assert_no_merged_names(&xml);
  let sussex = affiliation(
    "Department of Physics and Astronomy, University of Sussex, Brighton, United Kingdom",
  );
  assert_eq!(creators(&xml), [
    creator(
      true,
      &href("https://orcid.org/0000-0002-0424-7903", "T.J.C. Bezerra"),
      &sussex
    ),
    creator(
      false,
      &href("https://orcid.org/0000-0001-8035-7162", "L. Asquith"),
      &sussex
    ),
  ]);
}

/// A doubled mark (`$^{**}$`, `$^{\dagger\dagger}$`) labels its institution as a single one does (the r7 re-review's
/// finding 3, probes n1, n2): Univ B on Bob Baker, Univ C on Cat Cole. Was each an affiliation of the shared creator, its
/// mark kept (77p r10).
#[test]
fn doubled_symbol_marks_label_their_institutions() {
  let xml = convert_repro!("author_doubled_symbol_marks.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">*</sup>"#,
      &affiliation("Univ A")
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">**</sup>"#,
      &affiliation("Univ B")
    ),
    creator(
      false,
      r#"Cat Cole<sup xml:id="id3">††</sup>"#,
      &affiliation("Univ C")
    ),
  ]);
}

/// A linked name holding a link word ("Lucy Page", "Mark Link") is a name, not a link line (the r7 re-review's finding
/// 4, probes n3, n4): three authors. Was Lucy Page a URL contact of the block, Mark Link one of a creator named "Univ A"
/// (77p r10).
#[test]
fn link_words_in_a_linked_name_keep_the_name() {
  let xml = convert_repro!("author_linked_names_with_link_words.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(true, &href("https://x.org/lp", "Lucy Page"), ""),
    creator(
      false,
      &href("https://orcid.org/0000-0001", "Mark Link"),
      &affiliation("Univ A")
    ),
    creator(false, "Bob Baker", &affiliation("Univ B")),
  ]);
}

/// A linked institution holding a link word ("Data Science Institute") is that institution linked, an affiliation (the
/// r7 re-review's finding 4, probe n6). Was a URL contact of Ann Able (77p r10).
#[test]
fn link_words_in_a_linked_institution_keep_the_affiliation() {
  let xml = convert_repro!("author_linked_institution_with_link_words.tex");
  assert_no_merged_names(&xml);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &affiliation(&href(
        "https://datascience.columbia.edu",
        "Data Science Institute"
      ))
    ),
    creator(false, "Bob Baker", &affiliation("Univ B")),
  ]);
}

/// Under a row of symbol-marked names, a linked institution holding a link word ("Data Science Lab") is an unmarked
/// institution, every name's of the row (R12; the r7 re-review's finding 4, probe n7). Was a URL contact of the shared
/// creator (77p r10).
#[test]
fn link_words_in_a_linked_institution_under_a_marked_row() {
  let xml = convert_repro!("author_linked_institution_with_link_words_marked.tex");
  assert_no_merged_names(&xml);
  let lab = affiliation(&href("https://www.ds.org", "Data Science Lab"));
  assert_eq!(creators(&xml), [
    creator(
      true,
      r#"Ann Able<sup xml:id="id1">†</sup>"#,
      &format!("{} {lab}", affiliation("Univ A"))
    ),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">‡</sup>"#,
      &format!("{} {lab}", affiliation("Univ B"))
    ),
  ]);
}

/// An icon before a link (`\faGithub \, \href{…}{…}`, 2601.03661; the r7 re-review's finding 5) prints no word: the line
/// is a link line, a URL contact of the block. Was an affiliation of both names (77p r10).
#[test]
fn icon_led_link_line_is_a_url() {
  let xml = convert_repro!("author_icon_led_link_line.tex");
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
      &format!("{univ_a} {}", email("bbaker@x.edu"))
    ),
    shared(&url(concat!(
      r#"<inline-block aria:hidden="true" class="fas fa-github" xml:id="id1"/> "#,
      r#"<ref class="ltx_href" font="sansserif" fontsize="90%" href="https://github.com/aable/proj">"#,
      "https://github.com/aable/proj</ref>"
    ))),
  ]);
}

/// A line of several icon-led links (`\faGithub~\href{…}{Code}\hspace{1.5em} \yticon~\href{…}{Demo Video}`,
/// 2601.06979; the r7 re-review's finding 5), and one of links parted by `$\cdot$`, is a URL contact per link, the
/// block's. Was one affiliation "Code Demo Video" of Bob Baker, and "Dataset ⋅ Paper" another (77p r10).
#[test]
fn multi_link_line_is_a_url_per_link() {
  let xml = convert_repro!("author_multi_link_line.tex");
  assert_no_merged_names(&xml);
  let names = creators(&xml);
  assert_eq!(names.len(), 3, "{xml}");
  for name in &names[..2] {
    assert!(
      name.contains("Department of Computer Science, Univ A") && !name.contains("Code"),
      "{xml}"
    );
  }
  let urls: Vec<&str> = names[2]
    .split(r#"<contact name="URL: " role="url">"#)
    .skip(1)
    .collect();
  assert_eq!(urls.len(), 4, "{xml}");
  for (url, link) in urls.iter().zip([
    href("https://github.com/a/b", "Code"),
    href("https://www.youtube.com/watch?v=7", "Demo Video"),
    href("https://x.org/data", "Dataset"),
    href("https://x.org/paper", "Paper"),
  ]) {
    assert!(url.contains(&link) && !url.contains("⋅"), "{xml}");
  }
}

/// Lines that print nothing (`~~~~`) under rows of unmarked names are no entry, and an empty `\textsuperscript{}` before
/// the institution no mark (2602.23649; the r7 re-review's finding 6, probes e2, e3): Univ A on every name, the URL the
/// block's. Was two empty affiliations on every name (77p r10).
#[test]
fn empty_lines_under_unmarked_rows_print_nothing() {
  let xml = convert_repro!("author_empty_lines_unmarked_rows.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
    creator(false, "Cat Cole", &univ_a),
    creator(false, "Dan Dorn", &univ_a),
    creator(false, "Eve Eads", &univ_a),
    creator(false, "Fay Fox", &univ_a),
    shared(&url(&hlink("https://x.org/proj"))),
  ]);
}

/// A mailto link printing a brace list (`\href{mailto:julia.belikova@…}{\{julia.belikova, a.panchenko\}@skol.tech}`,
/// 2602.12235; the r7 re-review's finding 7) is an address per local part, each the e-mail of the name it spells. Was
/// the whole list one e-mail of Ann Able (77p r9, r10).
#[test]
fn mailto_brace_list_goes_by_spelling() {
  let xml = convert_repro!("author_mailto_brace_list.tex");
  assert_no_merged_names(&xml);
  // (the address the link reaches kept a link, the other as it prints)
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!(
        "{} {}",
        affiliation("Univ A"),
        mailto_email("Correspondence", "ann.able@x.edu", "ann.able@x.edu")
      )
    ),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{} {}",
        affiliation("Univ B"),
        r#"<contact name="Correspondence: " role="email">b.baker@x.edu</contact>"#
      )
    ),
  ]);
}

/// `\\ [0.5em]`: a line break skips the spaces before its optional argument, as latex.ltx's `\@xnewline` does
/// (2604.20841; the r7 re-review's finding 8) — the link under it a URL contact of the block, no "[0.5em]". Was an
/// affiliation "[0.5em] https://…" of Bob Baker (77p r10).
#[test]
fn line_break_optional_after_spaces_prints_nothing() {
  let xml = convert_repro!("author_line_break_spaced_optional.tex");
  assert_no_merged_names(&xml);
  assert!(!xml.contains("[0.5em]"), "{xml}");
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{univ_a} {}", typewriter_email("aable@x.edu", "id1"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!(
        "{univ_a} {} {}",
        affiliation("Univ B"),
        typewriter_email("bbaker@x.edu", "id2")
      )
    ),
    shared(&url(
      r#"<ref class="ltx_href" font="typewriter" fontsize="90%" href="https://x.org/p/">https://x.org/p/</ref>"#
    )),
  ]);
}

/// A truncated family-first address (`zhuzhenh22@`, `chenyuxi22@`; 2602.03328, 2609.11716; the r7 re-review's finding
/// 9) spells the name whose joined family-first spelling it opens with the first six letters of, as a surname's six
/// letters do. Was each on the shared creator, two names tied by the surname (77p r10).
#[test]
fn truncated_family_first_address_spells_its_name() {
  let xml = convert_repro!("author_email_truncated_family_first.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  let email =
    |address: &str| format!(r#"<contact name="Email: " role="email">{address}</contact>"#);
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Zhenhao Zhu",
      &format!("{univ_a} {}", email("zhuzhenh22@x.edu"))
    ),
    creator(
      false,
      "Yuxiang Chen",
      &format!("{univ_a} {}", email("chenyuxi22@x.edu"))
    ),
    creator(false, "Xinzhong Zhu", &univ_a),
    creator(false, "Jianfei Chen", &univ_a),
  ]);
}

/// A link to a bare domain printing a word that names no institution ("here") is a URL contact of the block (the r7
/// re-review's finding 10, probes n13, p9). Was an affiliation "here" of both names (77p r10).
#[test]
fn bare_domain_link_printing_here_is_a_url() {
  let xml = convert_repro!("author_bare_domain_link_word_here.tex");
  assert_no_merged_names(&xml);
  let names = creators(&xml);
  assert_eq!(names.len(), 3, "{xml}");
  assert!(
    !names[0].contains("here") && !names[1].contains("here"),
    "{xml}"
  );
  assert_eq!(
    names[2],
    shared(&url(&href("https://ann.org", "here"))),
    "{xml}"
  );
}

/// `\rule`, `\phantom`, `\raisebox`, `\kern` and `\hskip` print no letter of an address (the r7 re-review's finding 11,
/// probes p1, p2, p4, p5, p6): each address the e-mail of the name it spells. Was "0pt2exaable@", "xxccole@x.edu
/// ddorn@x.edu" one contact and "eeads@x.edu ffox@x.edu" an affiliation (77p r10).
#[test]
fn rules_phantoms_and_skips_print_no_letter() {
  let xml = convert_repro!("author_non_printing_rules_phantoms_skips.tex");
  assert_no_merged_names(&xml);
  let names = creators(&xml);
  assert_eq!(names.len(), 6, "{xml}");
  for (name, address) in names.iter().zip([
    "aable@x.edu",
    "bbaker@x.edu",
    "ccole@x.edu",
    "ddorn@x.edu",
    "eeads@x.edu",
    "ffox@x.edu",
  ]) {
    assert!(
      name.contains(&affiliation("Univ A"))
        && name.matches(r#"role="email""#).count() == 1
        && name.contains(&format!(">{address}<")),
      "{address}: {xml}"
    );
  }
  assert!(!xml.contains("0pt") && !xml.contains("xxccole"), "{xml}");
}

/// Symbol-labelled institutions on one line parted by `\qquad` under a row of marked names (the r7 re-review's g2, x13;
/// int34a the same): the block `\qquad` parts off after the line is the next institution of that line, no names —
/// Univ A on the † names, Univ B on the ‡ one. Was a creator named "‡Univ B" and "†Univ A" on Cat Cole alone.
#[test]
fn symbol_institution_row_parted_by_space() {
  let xml = convert_repro!("author_symbol_institution_row_spaced.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, r#"Ann Able<sup xml:id="id1">†</sup>"#, &univ_a),
    creator(
      false,
      r#"Bob Baker<sup xml:id="id2">‡</sup>"#,
      &affiliation("Univ B")
    ),
    creator(false, r#"Cat Cole<sup xml:id="id3">†</sup>"#, &univ_a),
  ]);
}

/// The negative of [`symbol_institution_row_parted_by_space`]: a name its mark leads (`$^\ddagger$Cat Cole`) after a
/// group's lines is no institution, an author.
#[test]
fn prefix_marked_name_after_lines_stays_a_name() {
  let xml = convert_repro!("author_prefix_marked_name_after_lines.tex");
  assert_no_merged_names(&xml);
  let names = creators(&xml);
  assert_eq!(names.len(), 3, "{xml}");
  assert!(
    names[2].contains("Cat Cole</personname>") && !names[1].contains("Cat Cole"),
    "{xml}"
  );
}

/// A mark that prints nothing but a `\relax` (`Ann Able\textsuperscript{\relax}, Bob Baker`, the r7 re-review's n10;
/// int34a the same) is no mark (user ruling R11): Univ A on both names. Was a creator "Bob BakerUniv A".
#[test]
fn relax_mark_is_no_mark() {
  let xml = convert_repro!("author_relax_mark_is_no_mark.tex");
  assert_no_merged_names(&xml);
  let univ_a = affiliation("Univ A");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &univ_a),
    creator(false, "Bob Baker", &univ_a),
  ]);
}

/// Comma-parted names whose own symbols label their lines (2603.07345; batch 77p r11 A/B): the unmarked
/// institution before the labelled one on its line is every name's (R12), the labelled one its carrier's, and the
/// e-mail line goes by the group's names as in any group — `bgh@` on Bob Baker by the order its neighbours confirm, the
/// †-marked address on Dan Dorn — in the order the lines print. Was "Uber Technologies †University of British Columbia"
/// one affiliation of every name (77p r10).
#[test]
fn comma_group_symbol_lines_go_by_their_marks() {
  let xml = convert_repro!("author_comma_group_symbol_lines.tex");
  assert_no_merged_names(&xml);
  let uber = affiliation("Uber Technologies");
  let it = |address: &str, id: &str| {
    format!(
      r#"<contact name="Email: " role="email"><text font="italic" xml:id="{id}">{address}</text></contact>"#
    )
  };
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Ann Able",
      &format!("{uber} {}", it("aable@uber.com", "id1"))
    ),
    creator(
      false,
      "Bob Baker",
      &format!("{uber} {}", it("bgh@uber.com", "id2"))
    ),
    creator(
      false,
      "Cat Cole",
      &format!("{uber} {}", it("ccole@uber.com", "id3"))
    ),
    creator(
      false,
      r#"Dan Dorn<sup xml:id="id4">†</sup>"#,
      &format!(
        "{uber} {} {}",
        affiliation("University of British Columbia"),
        it("dorn@cs.ubc.ca", "id5")
      )
    ),
  ]);
}

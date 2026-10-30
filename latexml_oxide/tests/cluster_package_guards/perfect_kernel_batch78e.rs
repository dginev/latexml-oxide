//! Red/green guards for batch 78e (quality batch 8): the webofc binding (its `\inst` authors and institutes, its
//! e-mail and `\fnsep`, its side caption and caption label, its `\author` lines), the jinstpub binding (jheppub's
//! frontmatter with JINST's e-mail placement, header line, equation numbering and caption), vmsta's e-mails and
//! addresses as their authors' contacts (the 2014-2018 and the 2018+ copies); round 2 (the rev78e review): a thanks
//! note all of whose content became contacts is none (webofc, aa), a split at `\\` drops the break's `*` and `[…]`
//! (webofc, llncs). Each guard converts its repro under `tools/perfect_kernel/repros/` beside the cut-down class or
//! package the papers ship, and asserts no Error or Warning and the authored content in whole elements, against
//! pdflatex. Witnesses 2610.23498, 2301.07438, 1411.7792 (webofc), 2006.05543, 2303.10224 (jinstpub), 1508.02842,
//! 1801.00977, 1411.4499, 1805.02511, 2001.02487, 1912.07948 (vmsta).
use latexml::util::test::assert_element;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch77d::{creator, creators, squash},
};

/// A repro beside the files it ships (`(name, path under repros/)`, none for a class in TeX Live), converted with the
/// ar5iv preload: its squashed XML, the conversion free of errors and warnings.
macro_rules! repro_with {
  ($path:literal $(, ($name:literal, $file:literal) )* $(,)? ) => {{
    let (log, xml) = latexml::util::test::convert_files_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      &[ $( ($name, include_str!(concat!("../../../tools/perfect_kernel/repros/", $file))) ),* ],
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// A contact of `role`, named as the frontmatter names it.
fn contact(role: &str, text: &str) -> String {
  let name = match role {
    "affiliation" => "Affiliation: ",
    "email" => "Email: ",
    "note" => "Note: ",
    _ => unreachable!(),
  };
  format!(r#"<contact name="{name}" role="{role}">{text}</contact>"#)
}

/// webofc.cls's title page is the `\inst` form: `\inst{n}` names the `n`-th `\institute` entry, the e-mail is its
/// author's contact and `\fnsep` before a `\thanks` sets nothing (webofc.cls:217, :434-440); the `\thanks` whose whole
/// content, the e-mail, became the contact is no note (ruling 2026-10-21). Red (int45b; the class read raw): the
/// personname "Ann Able<sup>,</sup><sup>1</sup>", both institutes on both authors, the e-mail "e-mail: …" in a thanks
/// note; 78e r1: an empty thanks note beside the e-mail contact. Witnesses 2301.07438, 1411.7792, 2409.01693. Repro
/// class-bindings/webofc/webofc_authors_take_their_institutes.
#[test]
fn webofc_authors_take_their_institutes() {
  let xml = repro_with!(
    "class-bindings/webofc/webofc_authors_take_their_institutes.tex",
    ("webofc.cls", "class-bindings/webofc/webofc.cls")
  );
  let ann = format!(
    "{} {}",
    contact("email", "ann@x.org"),
    contact("affiliation", "Univ A")
  );
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &ann),
    creator(false, "Bob Baker", &contact("affiliation", "Univ B")),
  ]);
}

/// webofc's `\sidecaption` sets the float's body beside its caption (webofc.cls:824-857): the caption stays the
/// figure's `<caption>`, which carries the label, and the class's `\@makecaption` (:788-812) closes the labels with ". "
/// ("**Figure 1.**", the bold under the class caption fonts knob alone, user ruling 2026-10-22:
/// `perfect_kernel_batch78y::webofc_with_class_caption_fonts`). Red (int45b): "Figure 1. Side caption text." typeset as
/// text in a `\parbox`, no `<caption>`, the label on the section; "Table 1:" upright; 78e r1: the period outside the
/// bold. Witness 2610.23498
/// (html_feedback). Repro class-bindings/webofc/webofc_sidecaption_keeps_its_caption.
#[test]
fn webofc_sidecaption_keeps_its_caption() {
  let xml = repro_with!(
    "class-bindings/webofc/webofc_sidecaption_keeps_its_caption.tex",
    ("webofc.cls", "class-bindings/webofc/webofc.cls")
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S1.F1\""],
    "<figure inlist=\"lof\" labels=\"LABEL:f1\" placement=\"t\" xml:id=\"S1.F1\"> <tags> <tag>Figure 1</tag> \
     <tag role=\"refnum\">1</tag> <tag role=\"typerefnum\">Figure 1</tag> </tags> \
     <rule height=\"28.5pt\" width=\"56.9pt\"/> <toccaption><tag close=\" \">1</tag>Side caption text.</toccaption> \
     <caption><tag close=\". \">Figure 1</tag>Side caption text.</caption> </figure>",
  );
  assert_element(
    &xml,
    "table",
    &["xml:id=\"S1.T1\""],
    "<table inlist=\"lot\" placement=\"b\" xml:id=\"S1.T1\"> <tags> <tag>Table 1</tag> \
     <tag role=\"refnum\">1</tag> <tag role=\"typerefnum\">Table 1</tag> </tags> \
     <toccaption><tag close=\" \">1</tag>Values.</toccaption> \
     <caption><tag close=\". \">Table 1</tag>Values.</caption> </table>",
  );
}

/// webofc sets the `\author` list as a paragraph (webofc.cls:497-552), its `\\` a line break whose `*` and `[…]` are
/// no text: the split at `\\` (`\lx@splitting`) starts the next creator without them. Red (78e r1): the creators
/// "*[2mm] Cat Cole" and "[1mm] The X Collaboration". The rev78e review's finding 6 (synthetic; 0 of 158 webofc
/// papers). Repro class-bindings/webofc/webofc_author_line_break_options.
#[test]
fn webofc_author_line_break_options() {
  let xml = repro_with!(
    "class-bindings/webofc/webofc_author_line_break_options.tex",
    ("webofc.cls", "class-bindings/webofc/webofc.cls")
  );
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &contact("affiliation", "Univ A")),
    creator(false, "Bob Baker", &contact("affiliation", "Univ B")),
    creator(false, "Cat Cole", &contact("affiliation", "Univ A")),
    creator(false, "The X Collaboration", ""),
  ]);
}

/// llncs's `\inst` author list splits at `\\` too (llncs_cls.rs `\lx@llncs@author@split`, the same `\lx@splitting`):
/// the break's `*` and `[…]` are no text. Red (78e r1): the creator "*[2mm] Cat Cole". The rev78e review's finding 6.
/// Repro sectioning-frontmatter/llncs_author_line_break_options_with_inst_marks.
#[test]
fn llncs_author_line_break_options_with_inst_marks() {
  let xml =
    repro_with!("sectioning-frontmatter/llncs_author_line_break_options_with_inst_marks.tex",);
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &contact("affiliation", "Inst A")),
    creator(false, "Bob Baker", &contact("affiliation", "Inst B")),
    creator(false, "Cat Cole", &contact("affiliation", "Inst A")),
  ]);
}

/// aa's template shape `\fnmsep\thanks{\email{…}}` (aa.cls v9.1 :761): the e-mail is the author's contact and the thanks
/// note it emptied is none (ruling 2026-10-21, `\lx@annotate@frontmatter@now`); an author's own empty `\thanks{}` is no
/// note either (ruling 2026-10-30, batch 92e, OD #916; Perl keeps it). Red (int45b): an empty thanks note on Ann Able.
/// The rev78e review's finding 2 (webofc's 2409.01693 the same shape). Repro
/// class-bindings/aa_thanks_email_leaves_no_empty_note.
#[test]
fn aa_thanks_email_leaves_no_empty_note() {
  let xml = repro_with!("class-bindings/aa_thanks_email_leaves_no_empty_note.tex",);
  let ann = format!(
    "{} {}",
    contact(
      "email",
      r#"<text font="typewriter" xml:id="id1">ann@x.org</text>"#
    ),
    contact("affiliation", "Univ A")
  );
  let bob = contact("affiliation", "Univ B");
  assert_eq!(creators(&xml), [
    creator(true, "Ann Able", &ann),
    creator(false, "Bob Baker", &bob),
  ]);
}

/// jinstpub.sty's frontmatter is jheppub's (jinstpub.sty:111-140): the labelled affiliations on their authors, the
/// e-mail and the `\note` the first author's contacts, the keywords; its equations are numbered within sections (:335)
/// and its caption label closed by a period (:362-378; the label's bold under the class caption fonts knob alone,
/// `perfect_kernel_batch78y::jinstpub_with_class_caption_fonts`). The copy's `\maketitle` ends with the table of contents (its
/// default, :241-249), which the family's rule drops, as jheppub's binding (Perl jheppub.sty.ltxml:97; a SYNC row
/// awaits a ruling); the stock header line is page furniture, no pubnote. Red (int45b; the package read raw): the
/// affiliations an itemize, the e-mail and the keywords paragraphs, "Corresponding author." inside the personname,
/// "(1)", "Figure 1:". Witnesses 2006.05543, 2303.10224. Repro
/// package-bindings/jinstpub/jinstpub_frontmatter_is_jheppub_s.
#[test]
fn jinstpub_frontmatter_is_jheppub_s() {
  let xml = repro_with!(
    "package-bindings/jinstpub/jinstpub_frontmatter_is_jheppub_s.tex",
    ("jinstpub.sty", "package-bindings/jinstpub/jinstpub.sty")
  );
  let fanelli = [
    contact("note", "Corresponding author."),
    contact("affiliation", "Univ A, Cambridge"),
    contact("affiliation", "Lab B, Newport News"),
    contact(
      "email",
      r#"<text font="typewriter" xml:id="id1">cfanelli@mit.edu</text>"#,
    ),
  ]
  .join(" ");
  assert_eq!(creators(&xml), [
    creator(true, "C. Fanelli", &fanelli),
    creator(
      false,
      "B. Baker",
      &contact("affiliation", "Lab B, Newport News")
    ),
  ]);
  assert_element(
    &xml,
    "keywords",
    &[],
    "<keywords name=\"Keywords\">Cherenkov detectors</keywords>",
  );
  assert_element(
    &xml,
    "equation",
    &["xml:id=\"S1.E1\""],
    "<equation labels=\"LABEL:e\" xml:id=\"S1.E1\"> <tags> <tag>(1.1)</tag> <tag role=\"autoref\">Equation 1.1\
     <text xml:id=\"S1.E1.1\"/></tag> <tag role=\"refnum\">1.1</tag> </tags> <Math mode=\"display\" tex=\"a=b\" \
     text=\"a = b\" xml:id=\"S1.E1.m1\"> <XMath xml:id=\"S1.E1.m1.1\"> <XMApp xml:id=\"S1.E1.m1.1.1\"> \
     <XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok> <XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok> \
     <XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok> </XMApp> </XMath> </Math> </equation>",
  );
  assert_element(
    &xml,
    "caption",
    &[],
    "<caption><tag close=\". \">Figure 1</tag>A box.</caption>",
  );
  assert!(!xml.contains("<TOC"), "{xml}");
  assert!(!xml.contains("<pubnote"), "{xml}");
}

/// jinstpub's `\emailAdd` lists the addresses for its title page's one "E-mail:" line (jinstpub.sty:133-138, :219-224):
/// an address is the contact of the author whose name it spells, else the author block's (`\lx@add@contact@placed`),
/// not jheppub's n-th address to the n-th author; an `\emailAdd{a, b}` list is an address each; the corresponding
/// author's `\note{…}` is no part of the name the address spells (`is_note_macro`); the one address left spelling no one
/// (info@) is the one author's that note marks as the corresponding author, B. Baker (user ruling 2026-10-21, batch 78f
/// r2; OXIDIZED_DESIGN_DIVERGENCES #809), no longer the block's — the note in his `\author`, its printed number written
/// on no other name (78f r3). Red (78e r1): bbaker@lab.gov on A. Able, "able@lab.gov,
/// info@lab.gov" one contact on B. Baker; before the note rule bbaker@ the block's. Witnesses
/// 2311.16063 (bross@fnal.gov is Bross's, not Bersani's), 2307.12991, 1910.06438, 2311.00870, 2605.30709; lists
/// 2211.05026, 2409.03065; notes 2406.08712, 2603.01867. Repro
/// package-bindings/jinstpub/jinstpub_emails_go_to_the_authors_they_spell.
#[test]
fn jinstpub_emails_go_to_the_authors_they_spell() {
  let xml = repro_with!(
    "package-bindings/jinstpub/jinstpub_emails_go_to_the_authors_they_spell.tex",
    ("jinstpub.sty", "package-bindings/jinstpub/jinstpub.sty")
  );
  let email = |id: &str, address: &str| {
    contact(
      "email",
      &format!(r#"<text font="typewriter" xml:id="{id}">{address}</text>"#),
    )
  };
  let able = format!(
    "{} {}",
    contact("affiliation", "Univ A"),
    email("id1", "able@lab.gov")
  );
  let baker = format!(
    "{} {} {} {}",
    contact("note", "Corresponding author."),
    contact("affiliation", "Univ A"),
    email("id2", "bbaker@lab.gov"),
    email("id3", "info@lab.gov")
  );
  assert_eq!(creators(&xml), [
    creator(true, "A. Able", &able),
    creator(false, "B. Baker", &baker),
  ]);
}

/// A jinstpub paper's own first-page header line (its copy's `\gdef\@fpheader{…}`, jinstpub.sty:22, printed first by
/// `\maketitle`, :180) is author content, a pubnote; the stock "Prepared for submission to JINST" none (above). Red (78e
/// r1): the header lost. Witnesses 2103.06391 (FERMILAB-PUB-21-079-ND-SCD), 2205.10526, 2311.06155. Repro
/// package-bindings/jinstpub/jinstpub_edited_header_is_a_preprint_note.
#[test]
fn jinstpub_edited_header_is_a_preprint_note() {
  let xml = repro_with!(
    "package-bindings/jinstpub/jinstpub_edited_header_is_a_preprint_note.tex",
    ("jinstpub.sty", "package-bindings/jinstpub/jinstpub.sty")
  );
  assert_element(
    &xml,
    "pubnote",
    &[],
    "<pubnote name=\"Preprint: \" role=\"preprint\">Prepared for submission to JINST \
     (FERMILAB-PUB-21-079-ND-SCD)</pubnote>",
  );
}

/// vmsta.cls (2014-2018 copies) writes each e-mail to the `.aux` for a later run's title page (:242-256 `\@email`) and
/// appends each address to its title page's list (:381-390): they are the authors' contacts, an `\address[a]` the
/// affiliation of the authors labelled `a`. Red (int45b): no e-mail, the addresses paragraphs after the title page.
/// Witnesses 1508.02842, 1801.00977, 1411.4499, 1506.02940, 1601.01109. Repro
/// class-bindings/vmsta/vmsta_emails_and_addresses_are_contacts.
#[test]
fn vmsta_emails_and_addresses_are_contacts() {
  let xml = repro_with!(
    "class-bindings/vmsta/vmsta_emails_and_addresses_are_contacts.tex",
    ("vmsta.cls", "class-bindings/vmsta/vmsta.cls")
  );
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Yuliya Mishura",
      &format!(
        "{} {}",
        contact("email", "myus@univ.kiev.ua"),
        contact("affiliation", "Univ A")
      )
    ),
    creator(
      false,
      "Ivan Voronov",
      &format!(
        "{} {}",
        contact("email", "ivan@univ.kiev.ua"),
        contact("affiliation", "Univ B")
      )
    ),
  ]);
  assert_element(
    &xml,
    "para",
    &[],
    "<para xml:id=\"p1\"> <p xml:id=\"p1.1\">Text.</p> </para>",
  );
}

/// The 2018+ vmsta.cls copies take the e-mail inside the author (`\ead`, :1263-1297) and keep the addresses in their
/// own list (`\address@optarg`, :871-920); the class's own `\author` still counts the e-mails for its check at
/// `\end{frontmatter}` (:550-565). Red (int45b): no e-mail, no affiliation, both printed by the deposited title page.
/// Witnesses 1805.02511, 1810.09681, 2001.02487, 1912.07948. Repro
/// class-bindings/vmsta_2018/vmsta_2018_emails_and_addresses_are_contacts.
#[test]
fn vmsta_2018_emails_and_addresses_are_contacts() {
  let xml = repro_with!(
    "class-bindings/vmsta_2018/vmsta_2018_emails_and_addresses_are_contacts.tex",
    ("vmsta.cls", "class-bindings/vmsta_2018/vmsta.cls")
  );
  assert_eq!(creators(&xml), [
    creator(
      true,
      "Yuliya Mishura",
      &format!(
        "{} {}",
        contact("email", "myus@univ.kiev.ua"),
        contact("affiliation", "Univ A")
      )
    ),
    creator(
      false,
      "Ivan Voronov",
      &format!(
        "{} {}",
        contact("email", "ivan@univ.kiev.ua"),
        contact("affiliation", "Univ B")
      )
    ),
  ]);
  assert_element(
    &xml,
    "para",
    &[],
    "<para xml:id=\"p1\"> <p xml:id=\"p1.1\">Text.</p> </para>",
  );
}

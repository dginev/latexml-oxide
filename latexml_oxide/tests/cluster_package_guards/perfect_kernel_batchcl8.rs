//! Guards for CL8 (`agent_reports/2026-11-09_brief_CL8.md`, `…_brief_CL8b.md`; classes C7, C8 and C9 of CL5's `=2`
//! content losses), each off and at `=2`, one per mechanism:
//! - C9: a used mark of digits and a sub-letter is written whole (`RCreator::whole_marks` → `_refmarkshown`): BESIII's
//!   "50A,50C" was "A,C" (1705.11109);
//! - C7: a wrapped `\thanks` whose span's plain argument prints less than the note is written by its text, never the id
//!   alone (`emit::note_argument`; SCITEPRESS's `\orcidAuthor`, 2608.08994, 2202.03759);
//! - C8: a class's own `\email` keeps the label it prints before the address as the contact's `name`
//!   (`rehearsal::own_email_properties`; jpp/jfm's "Email address for correspondence:", 2603.18265), in a `\thanks` and
//!   on a line;
//! - C8: a label the block prints before a binding's `\email` names the binding's own nested contact
//!   (`\lx@annotate@frontmatter@now`'s stub `_name`; JHEP3's `… T6G 2G7 E-mail: \email{…}`, 0812.0465).
//!
//! CL10 (`agent_reports/2026-11-09_brief_CL10.md`, review revX2): the C8 label is printed once — the content of a contact
//! whose call prints it drops it (`insert::drop_printed_name`); and a glued mark's letter is never another entry's label
//! (`post::sub_labels`; BESIII's `67a,67c` beside `$^{a}$`/`$^{c}$` "Also at …", 2201.02056).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// Convert `tex` under `emit`: the log and the XML.
fn run(tex: &str, emit: u8) -> (String, String) {
  let (log, xml, ()) = convert_files_with_setup_then(
    tex,
    &[],
    Some("ar5iv.sty"),
    move || {
      set_fm_shadow_override(None);
      set_fm_emit_override(Some(emit));
    },
    |_| (),
  );
  (log, xml)
}

/// Every `<creator>` element of `xml`, whole, its whitespace runs collapsed to one space.
fn creator_elements(xml: &str) -> Vec<String> {
  xml
    .match_indices("<creator ")
    .map(|(at, _)| {
      let end = at + xml[at..].find("</creator>").expect("a closed creator") + "</creator>".len();
      xml[at..end]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
    })
    .collect()
}

/// The sub-lettered marks `2A,2C` and `2B`, each parted into the used entry 2 and a letter no entry answers, show whole
/// on their names at `=2`, as off (int153z's `=2` showed "A,C" and "B").
#[test]
fn a_sublettered_used_mark_is_written_whole() {
  let turin = "<contact name=\"Affiliation: \" role=\"affiliation\"> (A)University of Turin, Turin; (B)University \
               of Piedmont, Alessandria; (C)INFN, Turin</contact>";
  let expected = [
    format!(
      "<creator role=\"author\"> <personname>A. Amoroso<sup xml:id=\"id1\">2A,2C</sup></personname> {turin} </creator>"
    ),
    "<creator before=\" \" role=\"author\"> <personname>B. Bee</personname> <contact name=\"Affiliation: \" \
     role=\"affiliation\"> Institute One, City One</contact> </creator>"
      .to_string(),
    format!(
      "<creator before=\" \" role=\"author\"> <personname>C. Cee<sup xml:id=\"id2\">2B</sup></personname> {turin} \
       </creator>"
    ),
  ];
  for emit in [0, 2] {
    let (log, xml) = run(repro!("cl8_sublettered_mark_written_whole"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(creator_elements(&xml), expected, "emit={emit}");
  }
}

/// `\orcidAuthor{id}` = `\thanks{ORCID~https://orcid.org/#1}`: each note is written whole at `=2` (by its text, the
/// "markup lost" Warning counted for each), never the id alone.
#[test]
fn a_wrapped_thanks_note_is_written_whole() {
  let expected = [
    "<creator role=\"author\"> <personname>Ann Able</personname> <note class=\"ltx_note_frontmatter \
     ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">ORCID https://orcid.org/0000-0001-8817-2744</note> </creator>",
    "<creator before=\" \" role=\"author\"> <personname>Bob Bee</personname> <note class=\"ltx_note_frontmatter \
     ltx_thanks_note\" role=\"thanks\" xml:id=\"id2\">ORCID https://orcid.org/0000-0002-6100-8255</note> </creator>",
  ];
  for (emit, warnings) in [(0, 0), (2, 2)] {
    let (log, xml) = run(repro!("cl8_wrapped_thanks_note_written_whole"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "emit={emit}: {log}"
    );
    if emit == 2 {
      assert_eq!(
        log.matches("its markup lost: note no argument").count(),
        2,
        "{log}"
      );
    }
    assert_eq!(creator_elements(&xml), expected, "emit={emit}");
  }
}

/// jpp's `\email` prints "Email address for correspondence: " before the address: at `=2` Ann's (in a `\thanks`) and
/// Bob's (on his line) contacts are named by it, not by the generic "Email: " (int153z: Ann's label dropped, an
/// "unclaimed" Warning), and the label is printed once: Bob's contact, whose content is his `\email` call, drops the
/// label it prints (CL8b wrote it in the content too, the label rendered twice: review revX2 2). Off keeps Ann's note
/// whole, Bob's contact named generically. A label in a font of its own is dropped with its emptied wrapper, the address
/// keeping its own (`cl10_class_email_font_label_printed_once`).
#[test]
fn a_class_email_keeps_its_printed_label_as_name() {
  let univ = "<contact name=\"Affiliation: \" role=\"affiliation\">Univ B</contact>";
  let label = "Email address for correspondence: ";
  let bob = |name: &str, printed: &str| {
    format!(
      "<creator before=\" \" role=\"author\"> <personname>Bob Bee</personname> {univ} <contact name=\"{name}\" \
       role=\"email\">{printed}bob@bee.org</contact> </creator>"
    )
  };
  let off = [
    format!(
      "<creator role=\"author\"> <personname>Ann Able</personname> <note class=\"ltx_note_frontmatter \
       ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">{label}ann@able.org</note> {univ} </creator>"
    ),
    bob("Email: ", label),
  ];
  let emitted = [
    format!(
      "<creator role=\"author\"> <personname>Ann Able</personname> <contact name=\"{label}\" \
       role=\"email\">ann@able.org</contact> {univ} </creator>"
    ),
    bob(label, ""),
  ];
  for (emit, expected) in [(0, off), (2, emitted)] {
    let (log, xml) = run(repro!("cl8b_class_email_label_kept_as_name"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(creator_elements(&xml), expected, "emit={emit}");
  }
  let ann = |name: &str, label: &str, id: u8| {
    vec![format!(
      "<creator role=\"author\"> <personname>Ann Able</personname> <contact name=\"Affiliation: \" \
       role=\"affiliation\">Univ A</contact> <contact name=\"{name}\" role=\"email\">{label}<text \
       font=\"typewriter\" xml:id=\"id{id}\">ann@able.org</text></contact> </creator>"
    )]
  };
  for (emit, expected) in [
    (
      0,
      ann(
        "Email: ",
        "<text font=\"italic\" xml:id=\"id1\">E-mail:</text> ",
        2,
      ),
    ),
    (2, ann("E-mail: ", "", 1)),
  ] {
    let (log, xml) = run(repro!("cl10_class_email_font_label_printed_once"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(creator_elements(&xml), expected, "emit={emit}");
  }
}

/// JHEP3's `Univ A, City A E-mail: \email{…}` on one line: at `=2` the binding's contact is named "E-mail: " (its call
/// the item's content, the nested `\lx@add@email` taking the item's name), the label out of the affiliation; int153z
/// named it "Email: " and counted "E mail" missing. Off leaves the label in the affiliation, as Perl.
#[test]
fn an_in_block_label_names_the_bindings_nested_contact() {
  let contact = |name: &str| {
    format!(
      "<contact name=\"{name}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann@able.org</text></contact>"
    )
  };
  let creator = |affiliation: &str, name: &str| {
    vec![format!(
      "<creator role=\"author\"> <personname>Ann Able</personname> <contact name=\"Affiliation: \" \
       role=\"affiliation\">{affiliation}</contact> {} </creator>",
      contact(name)
    )]
  };
  for (emit, expected) in [
    (0, creator("Univ A, City A E-mail: ", "Email: ")),
    (2, creator("Univ A, City A", "E-mail: ")),
  ] {
    let (log, xml) = run(repro!("cl8b_inblock_email_label_names_binding_call"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(creator_elements(&xml), expected, "emit={emit}");
  }
}

/// BESIII's `$^{67a,67c}$` under `$^{67}$ University of Turin and INFN, (A)…; (B)…; (C)…` beside `$^{a}$`/`$^{c}$` "Also
/// at …" (2201.02056): the letters are entry 2's sub-labels, never the a and c entries' — Amoroso holds entry 2 alone, the
/// mark shown whole (int153w, off and `=2`: the a and c entries too, `=2` dropping the mark). A comma mark `1,a` / `3,c`
/// still requests a / c; `2b`, its letter no entry's label, is as before. Off is the same (the shared reading).
#[test]
fn a_glued_marks_letter_is_never_another_entrys() {
  let affiliation =
    |text: &str| format!("<contact name=\"Affiliation: \" role=\"affiliation\">{text}</contact>");
  let turin = affiliation(
    " University of Turin and INFN, (A)University of Turin, Turin; (B)University of Piedmont, Alessandria; (C)INFN, \
     Turin",
  );
  let creator = |before: &str, name: &str, contacts: &[String]| {
    format!(
      "<creator {before}role=\"author\"> <personname>{name}</personname> {} </creator>",
      contacts.join(" ")
    )
  };
  let expected = vec![
    creator(
      "",
      "A. Amoroso<sup xml:id=\"id1\">2a,2c</sup>",
      std::slice::from_ref(&turin),
    ),
    creator("before=\" \" ", "B. Bee", &[
      affiliation(" Institute One, City One"),
      affiliation(" Also at the Moscow Institute, Moscow"),
    ]),
    creator("before=\" \" ", "C. Cee", &[
      affiliation(" Institute Three, City Three"),
      affiliation(" Also at the Kurchatov Institute, Gatchina"),
    ]),
    creator(
      "before=\" \" ",
      "D. Dee<sup xml:id=\"id2\">2b</sup>",
      std::slice::from_ref(&turin),
    ),
  ];
  for emit in [0, 2] {
    let (log, xml) = run(repro!("cl10_sublettered_mark_never_another_entrys"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(creator_elements(&xml), expected, "emit={emit}");
  }
}

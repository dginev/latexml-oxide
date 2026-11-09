//! Guards for IO1: a creator call's nested notes are its creator's — the reading attaches them (`Resolver::api`'s
//! `ltx:creator` branch), as Perl's `\lx@add@author[]{}` digests the whole call into its `ltx:creator`
//! (Base_Utility.pool.ltxml:588-589), so `=2` writes them with it (2505.09041's `\author{Craig
//! M$^{\mathrm{c}}$Rae\thanks{…}}`, whose thanks was lost at `=2`).

use latexml::util::test::{error_count, warning_count};

use super::perfect_kernel_batchz4::{creators_xml, run};

/// A `\thanks` inside an authblk `\author` call (a creator call read apart from the blocks): its note under the creator
/// on both switches, `=2` writing the creators off writes (its argument as written, `emit.rs` `call_note`, IO2).
#[test]
fn a_creator_calls_nested_thanks_is_its_creators() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_authblk_superscript_inside_a_name.tex"
  );
  let (log, emitted) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let creators = creators_xml(&emitted);
  assert_eq!(creators, vec![
    "<creator role=\"author\">\n    <personname>Craig M<sup><text font=\"italic\">c</text></sup>Rae</personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\">Electronic address: craig@x.org</note>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Y</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Y</contact>\n  </creator>"
  ]);
  assert_eq!(creators.len(), 2, "{emitted}");
  assert!(
    creators[0].contains("Rae</personname>")
      && creators[0].contains("role=\"thanks\"")
      && creators[0].contains(">Electronic address: craig@x.org</note>"),
    "{}",
    creators[0]
  );
}

/// IO2 (review revG5 3): the call's `\thanks` is written from its argument as written, not its text — the italic, the
/// formula and the link kept at `=2` as off keeps them (`emit.rs` `call_note`; distinct from the plain-text note above:
/// the markup only the tokens hold).
#[test]
fn a_creator_calls_thanks_keeps_its_markup() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_authblk_call_thanks_markup.tex"
  );
  let (log, out) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let creators = creators_xml(&out);
  assert_eq!(creators.len(), 2, "{out}");
  let craig = &creators[0];
  assert!(
    craig.contains("Rae</personname>")
      && craig.contains("role=\"thanks\"")
      && craig.contains("Note <text font=\"italic\"")
      && craig.contains("tex=\"x+y\"")
      && craig.contains("<ref class=\"ltx_href\" href=\"http://x.org\">the site</ref>.</note>"),
    "{craig}"
  );
}

/// IO2 (review revG5 2): an `\email` the document defines inside an authblk `\author` call is the name's text, as Perl's
/// `\lx@add@author` digests the whole call into the personname — never also a contact (the address written twice).
#[test]
fn a_creator_calls_email_stays_in_its_name() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_authblk_call_email_in_name.tex"
  );
  let (log, emitted) = run(tex);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let creators = creators_xml(&emitted);
  assert_eq!(creators, vec![
    "<creator role=\"author\">\n    <personname>Craig Rae <text font=\"typewriter\">craig@x.org</text></personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Y</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Ann Able</personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\">Note y.</note>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ X</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ Y</contact>\n  </creator>"
  ]);
  assert_eq!(creators.len(), 2, "{emitted}");
  assert!(
    creators[0].contains("craig@x.org</text></personname>")
      && creators[0].matches("craig@x.org").count() == 1,
    "{}",
    creators[0]
  );
}

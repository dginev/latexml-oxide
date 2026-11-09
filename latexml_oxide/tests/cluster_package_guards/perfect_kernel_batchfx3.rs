//! Guards for FX3: a `\thanks` note's link its owner or the author it spells takes (`ruling4:inner-contact`, `Resolver::note`, OD
//! #837's decision 2), is not written as a contact at `LATEXML_FM_EMIT=2` while the note written there still prints it —
//! one record per fact, the switch off's output (its writer declines the contact; `writer::emit::items_to_emit`'s
//! `in_a_written_note`). One test per way the note prints the address: a `mailto:` link (1410.4175, 2009.03378,
//! 2604.08687, 2410.14846) and the document's own `\email` printing its label (0807.0895); and the sibling it must keep, a
//! class binding's e-mail call whose note prints it nowhere (llncs, 2406.13066).

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// `tex`'s creators at `=2`: the switch-off ones (ids aside), no contact, `note` on the first creator holding `address`
/// once; the off log clean, the `=2` log's Warnings `warnings`.
fn held_once(tex: &str, note: &str, address: &str, warnings: usize) -> String {
  let (log, xml) = run(tex);
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  let want = vec![vec!["Ann Smith".to_string()], vec!["Bob Jones".to_string()]];
  assert_eq!(creators(&xml), want);
  let written = creators_xml(&xml).concat();
  assert!(written.contains(note), "{written}");
  assert_eq!(
    written.matches(&format!(">{address}<")).count(),
    1,
    "{written}"
  );
  log
}

/// `\thanks{Corresponding author: \href{mailto:ann@x.org}{ann@x.org}}`: the note with its link, no e-mail contact. Red
/// (FX3base `=2`): a contact `role=email` repeating the address, the content check's "more often" Warning.
#[test]
fn a_mailto_link_in_a_thanks_is_the_notes_alone() {
  held_once(
    repro!("emit_thanks_mailto_link_held_once"),
    r#"role="thanks">Corresponding author: <ref class="ltx_href" href="mailto:ann@x.org">ann@x.org</ref></note>"#,
    "ann@x.org",
    0,
  );
}

/// `\def\email#1{\textrm{e-mail:~#1}}` and `\thanks{Corresponding author: Ann Smith, Univ A (\email{\tt ann@x.org}).}`: the
/// note, label (its tie a no-break space) and brackets in place, no contact repeating "e-mail:" and the address. Red (FX3base `=2`): that contact.
/// No Warning: the content check reads the label the document's `\email` prints as the call's event's `name`
/// (`rehearsal::own_email_properties`, CL8b), printed beside the address, where it counted "e mail" more often than
/// printed (cw78's mechanism A, 1001.0337).
#[test]
fn an_own_email_call_in_a_thanks_is_the_notes_alone() {
  held_once(
    repro!("emit_thanks_own_email_call_held_once"),
    "role=\"thanks\">Corresponding author: Ann Smith, Univ A (e-mail:\u{a0}<text font=\"typewriter\">ann@x.org</text>).</note>",
    "ann@x.org",
    0,
  );
}

/// llncs's `\thanks{Corresponding author: \email{ann@x.org}}`: the class binding's call makes the contact and the note
/// written prints the address nowhere, so the contact stays, the address held once (2406.13066). Red (the FX3 draft, which
/// kept a binding's call only when `EmailMeaning::Binding`): the address lost, the content check's "missing" Warning.
#[test]
fn a_class_email_call_in_a_thanks_keeps_its_contact() {
  let (log, xml) = run(repro!("emit_thanks_llncs_email_call_keeps_its_contact"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let found = creators(&xml);
  assert_eq!(found.len(), 2, "{found:?}");
  assert_eq!(found[0][0], "Ann Smith");
  assert!(
    found[0].contains(&"email=ann@x.org".to_string()),
    "{found:?}"
  );
  assert_eq!(
    creators_xml(&xml).concat().matches(">ann@x.org<").count(),
    1,
    "{xml}"
  );
}

//! Guards for BC1, the one-pass fold's first step (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`; OXIDIZED_DESIGN_DIVERGENCES
//! #980): at `LATEXML_FM_EMIT=2` the emitter writes the author block's own creator (OXIDIZED_DESIGN #159), which relocate's
//! orphan-sharing branch made before.

use latexml::util::test::{error_count, warning_count};

use super::{
  perfect_kernel_batch84b::creators,
  perfect_kernel_batchz4::{creators_xml, run},
};

/// A block's e-mail line no name spells (`label=addresses:block`): the nameless creator below the names holds it, written by
/// the emitter (`writer::emit::block_creator`), into which relocate's orphan procedure shares where it would promote a stub,
/// uncounted among the names (no `_num`: Bob Baker's separator stays the last name's). Witnesses 1012.4847, 1106.3045 (byte
/// identical before and after on int149a `=2`). The creators' XML is the switch-off one.
#[test]
fn the_blocks_creator_is_the_emitters() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/emit_the_blocks_creator_is_the_emitters.tex"
  );
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let want = [
    vec!["Ann Able".to_string(), "affiliation=Univ A".to_string()],
    vec!["Bob Baker".to_string(), "affiliation=Univ A".to_string()],
    vec!["email=lab@a.org".to_string()],
  ];
  assert_eq!(creators(&xml), want, "{xml}");
  assert_eq!(creators_xml(&xml), creators_xml(&off));
  assert!(
    creators_xml(&xml)[2].starts_with(r#"<creator role="author">"#),
    "{xml}"
  );
}

/// At `=2` a block item keeps relocate's orphan rules (BC1r2; revBC1 finding 1): the emitter's creator holds only what
/// the procedure would share into a promoted stub. A sole author's line is the author's (aastex701, user ruling
/// 2026-11-05 "Sole author": Jane Doe's note, no nameless creator); a copy an author holds under another label is no
/// orphan (jheppub's `\emailAdd` list once, on its last author, no "more often" Warning). Creators and Warnings as
/// int149a's `=2`.
#[test]
fn a_block_item_keeps_the_orphan_rules() {
  let sole = "<creator role=\"author\">\n    <personname>Jane Doe</personname>\n    <rdf content=\"jane@x.org\" \
              property=\"schema:email\"/>\n    <contact name=\"Affiliation:\u{a0}\" \
              role=\"affiliation\">Univ A</contact>\n    <note role=\"note\">[note]</note>\n  </creator>";
  let jhep = [
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" \
     role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact \
     name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n    <contact name=\"Email:\u{a0}\" \
     role=\"email\"><text font=\"typewriter\">zq1@x.org, bbaker@x.org</text></contact>\n  </creator>",
  ]
  .map(str::to_string)
  .to_vec();
  for (name, warnings, want) in [
    ("email_call_with_trailing_text_on_author_line", 2, vec![
      sole.to_string(),
    ]),
    ("jheppub_emailadd_markup_list_kept_whole", 0, jhep),
  ] {
    let path = format!(
      "{}/../tools/perfect_kernel/repros/sectioning-frontmatter/{name}.tex",
      env!("CARGO_MANIFEST_DIR")
    );
    let tex = std::fs::read_to_string(&path).expect(&path);
    let (log, xml) = run(&tex, 2);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, warnings),
      "{name}: {log}"
    );
    assert_eq!(creators_xml(&xml), want, "{name}: {xml}");
  }
}

/// The block's own creator is no annotation's parent by place (BC1r2): an unlabelled annotation goes to the last named
/// creator before it, as before the emitter wrote one — llncs's restated `\institute`'s second `\email` stays Bo Li's,
/// beside the note, and the nameless creator holds the block's addresses once each. Witness 2009.10916 (byte-identical
/// to int149a `=2`).
#[test]
fn the_blocks_creator_is_no_parent_by_place() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/llncs_supplement_restated_email_not_copied.tex"
  );
  let (log, xml) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert_eq!(
    creators(&xml),
    [
      vec![
        "Lv Tang".to_string(),
        "affiliation=Univ X, Town".to_string()
      ],
      vec![
        "Bo Li".to_string(),
        "affiliation=Univ Y, City".to_string(),
        "email=libraboli@tencent.com".to_string(),
      ],
      vec![
        "email=luckybird1994@gmail.com".to_string(),
        "email=libraboli@tencent.com".to_string(),
      ],
    ],
    "{xml}"
  );
  let bo_li = &creators_xml(&xml)[1];
  assert!(
    bo_li.contains(r#"role="thanks">Correspondence should be addressed to Bo Li.</note>"#),
    "{bo_li}"
  );
}

//! Guards for CL7 (`agent_reports/2026-11-09_brief_CL7.md`): `LATEXML_FM_EMIT=2` never writes fewer of a block's printed
//! words than off. One guard per mechanism, each off and at `=2`: a block's own `annotations` holding a command are digested
//! on its creators as off digests them, the note the document's (authblk's `\author[1\thanks{…}]`; 2308.08363,
//! 2410.16774, 2207.11475, 2207.11842); a call whose printed words the reading writes none of is written as queued, as off
//! writes it (iopart's `\ead` with no address in its text; 2112.14174).

use latexml::util::test::{convert_files_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{set_fm_emit_override, set_fm_shadow_override};

use super::perfect_kernel_batch84b::creators;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

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

/// The elements `tag` of `xml`, whole, in order.
fn elements(xml: &str, tag: &str) -> Vec<String> {
  let (open, close) = (format!("<{tag}"), format!("</{tag}>"));
  xml
    .match_indices(&open)
    .filter_map(|(at, _)| {
      let end = xml[at..].find(&close)? + close.len();
      Some(xml[at..at + end].to_string())
    })
    .collect()
}

/// authblk's `\author[1\thanks{Equal contribution alpha.}]{Aviv Slobodkin}`: off digests the `annotations={1\thanks{…}}`
/// the binding gives the creator, the note the document's pubnote; `=2` wrote the reading's labels alone and lost it, with
/// no Warning. Now its creators carry the block's own annotations, as off's.
#[test]
fn a_note_in_an_authors_annotations_is_kept() {
  for emit in [0, 2] {
    let (log, xml) = run(repro!("cl7_authblk_thanks_in_annotations"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(
      creators(&xml),
      [
        row(&["Aviv Slobodkin", "affiliation=Bar-Ilan University"]),
        row(&["Niv Nachum", "affiliation=Amazon"])
      ],
      "emit={emit}"
    );
    assert_eq!(
      elements(&xml, "pubnote"),
      ["<pubnote>Equal contribution alpha.</pubnote>"],
      "emit={emit}: {xml}"
    );
  }
}

/// iopart's `\ead{jborkowska astro.uni.torun.pl}` prints no address: `=2`'s reading holds no place for its words and wrote
/// nothing of the call ("they are lost", and the content check counted 5 words). Now the call is written as queued, as off
/// writes it — the e-mail contact of the newest creator as off's queue digests it —, with its one counted Warning.
#[test]
fn a_call_the_reading_writes_nothing_of_is_written_as_queued() {
  for emit in [0, 2] {
    let (log, xml) = run(repro!("cl7_iopart_ead_without_address"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, usize::from(emit == 2)),
      "emit={emit}: {log}"
    );
    if emit == 2 {
      assert!(
        log.contains("the call is written as queued, as the switch off writes it"),
        "{log}"
      );
    }
    assert_eq!(
      creators(&xml),
      [
        row(&[
          "Justyna Borkowska",
          "affiliation=Institute of Astronomy, Torun, Poland",
          "email=jborkowska astro.uni.torun.pl"
        ]),
        row(&[
          "Boudewijn F. Roukema",
          "affiliation=Institute of Astronomy, Torun, Poland"
        ])
      ],
      "emit={emit}"
    );
  }
}

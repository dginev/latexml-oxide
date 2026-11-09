//! Guards for NE3: the last papers where `LATEXML_FM_EMIT=2` raised more Errors than off, the flip's no-harm gate. One guard
//! per mechanism, each off and at `=2`: a key value the reading wrote back is characters (2607.18332's legend mark `#`); two
//! names written from overlapping runs are their text (1602.03850's `\myauthors`); a run a macro of which reads past its end
//! is refused (2209.12410's `\authord`); a script character the rehearsal read outside math is written as its character
//! (2204.01080's `\textbf{Ningkai Mo} ^1`); an alignment tab refuses a run only where the block's digestion holds an
//! alignment (review revNE1 finding 1: a `\url`'s `&`).

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

/// The opening tags of `xml`'s `<note>` elements.
fn note_tags(xml: &str) -> Vec<&str> {
  xml
    .match_indices("<note ")
    .map(|(at, _)| &xml[at..at + xml[at..].find('>').expect("a note's tag")])
    .collect()
}

/// The legend `\textsuperscript{\#}Corresponding author.` gives its note `mark={#}`: `=2` wrote the key value through
/// `internal`, the `#` a parameter token, an Error ("should never reach Stomach"). The key list's values are characters
/// (`emit::key_tokens`), the mark "#" as off's.
#[test]
fn a_legends_mark_is_written_as_characters() {
  let expected = [
    row(&["Ann Able", "affiliation=Univ A, City A"]),
    row(&["Bob Baker", "affiliation=Univ B, City B"]),
  ];
  for emit in [0, 2] {
    let (log, xml) = run(repro!("ne3_mark_hash"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert_eq!(creators(&xml), expected, "emit={emit}");
    assert_eq!(
      note_tags(&xml),
      [
        r#"<note class="ltx_note_frontmatter ltx_thanks_contribution" mark="*" role="thanks" xml:id="id1""#,
        r##"<note class="ltx_note_frontmatter ltx_thanks_correspondence" mark="#" role="thanks" xml:id="id2""##,
      ],
      "emit={emit}: {xml}"
    );
  }
}

/// `\newcommand{\myauthors}{Xing Shi Cai, Luc Devroye}` under scrartcl: `=2` read two names and wrote each creator from a
/// run holding the macro, both names twice (2 x merged_creators). Runs that overlap print each other's ink: the block is
/// written whole, Perl's whole-block creator, one counted Warning (CL9 `emit::names_overlap`, review revX1 finding 1: each
/// name as its text dropped what else the macro prints; `perfect_kernel_batchcl9`). Off is the old reader's one merged
/// creator, unchanged.
#[test]
fn names_a_macro_prints_together_are_their_text() {
  let (log, xml) = run(repro!("ne3_macro_names"), 0);
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(log.contains("merged_creators"), "{log}");
  assert_eq!(creators(&xml).len(), 1, "{xml}");
  let (log, xml) = run(repro!("ne3_macro_names"), 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains("two names of its author block from one run of its tokens"),
    "{log}"
  );
  assert_eq!(creators(&xml), [row(&[
    "Xing Shi Cai, Luc Devroye Univ A, City A, xingshi.cai@mail.mcgill.ca lucdevroye@gmail.com"
  ])]);
}

/// `\def\authord#1#2{\indent\Mr{#1}\\ \textit{\indent#2}\vsm}`: `=2` wrote the name `\authord{Cat Cole}`, cut from its
/// second argument, which read the item's closing brace (a mode Error; 2209.12410's `\@undefined` too). A run a macro of
/// which reads past its end is refused: the name is its text, counted. Off is the old reader's one merged creator.
#[test]
fn a_macro_cut_from_its_arguments_is_its_text() {
  let (log, xml) = run(repro!("ne3_macro_args"), 0);
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert_eq!(creators(&xml).len(), 1, "{xml}");
  let (log, xml) = run(repro!("ne3_macro_args"), 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  let univ_ab = ["affiliation=Univ A, City A", "affiliation=Univ B, City B"];
  assert_eq!(creators(&xml), [
    row(&["ANN ABLE", univ_ab[0], univ_ab[1]]),
    row(&["Bob Baker", univ_ab[0], univ_ab[1]]),
    row(&["CAT COLE", "affiliation=Univ C, City C"]),
  ]);
}

/// `\textbf{Ann Able} ^1\footnotemark[1]`: a text-mode `^`, which TeX errs at (the repro is invalid on purpose, the paper's
/// own error) and the rehearsal's recovery prints. `=2` wrote it back as a script, an Error per `^` (2204.01080: 6). It is
/// written as the character it printed, `\lx@fm@script@char`, a Warning each, the name's footnote mark kept. Off reads the
/// marks, no Error.
#[test]
fn a_text_mode_script_is_written_as_its_character() {
  let (log, xml) = run(repro!("ne3_text_script"), 0);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml).len(), 2, "{xml}");
  let (log, xml) = run(repro!("ne3_text_script"), 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  assert_eq!(
    log
      .matches("Script ^ can only appear in math mode; the frontmatter writes it as its character")
      .count(),
    3,
    "{log}"
  );
  let names: Vec<String> = creators(&xml)
    .into_iter()
    .filter_map(|c| c.first().cloned())
    .collect();
  assert!(names[0].starts_with("Ann Able ^1"), "{names:?}");
  assert!(names[1].starts_with("Bob Baker ^1 ^2"), "{names:?}");
  assert!(xml.contains(r#"role="footnotemark""#), "{xml}");
}

/// `\author{Ann Able\thanks{Univ A} \and Bob Baker\\ Univ B, \url{http://y.org/?c=1&d=2}}`: NE1's backstop refused every run
/// holding an `&`, so `=2` wrote the address as text, its link lost (a Warning). A block whose digestion holds no alignment
/// has no tab of one: the link is kept, as off.
#[test]
fn a_urls_ampersand_in_a_block_of_no_alignment_keeps_its_link() {
  for emit in [0, 2] {
    let (log, xml) = run(repro!("ne3_url_amp"), emit);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "emit={emit}: {log}"
    );
    assert!(
      xml.contains(
        r#"<ref class="ltx_nolink ltx_url" font="typewriter" href="http://y.org/?c=1&amp;d=2">"#
      ),
      "emit={emit}: {xml}"
    );
    assert_eq!(creators(&xml).len(), 2, "emit={emit}: {xml}");
  }
}

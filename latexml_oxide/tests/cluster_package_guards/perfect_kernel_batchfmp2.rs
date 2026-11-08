//! Guards for batch P2 (`pk/P2`, 2026-11-08), three switch-off mechanisms ported to `LATEXML_FM_EMIT=2` (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! OXIDIZED_DESIGN_DIVERGENCES #980): cas's one unlabelled `\affiliation` is every author's
//! (`a_cas_shared_affiliation_is_every_authors`); a role note is the note the switch-off output writes — a labelled
//! legend (`a_printed_label_legend_is_its_carriers_note`), a `\thanks` no number leads
//! (`a_thanks_no_number_leads_is_a_note`), a numbered `\thanks` legend (`a_numbered_thanks_legend_is_a_note`) and a
//! `\footnote` (`a_numbered_footnote_legend_is_a_note`); a name's own tokens are cut out of a font wrapper holding its
//! mark and note, and a note its name's tokens print is written once (`a_name_and_its_note_are_written_once`).
//! Each is distinct: a separate code path (the resolver's shared run, the emitter's printed label, the resolver's note
//! kind, digestion's printed mark, the emitter's footnote, the name's run and the items its tokens print).

use latexml::util::test::{error_count, warning_count};

use super::perfect_kernel_batchz4::{creators_xml, run};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// `tex` converted off and at `=2`, both clean: their creators' XML.
fn both(tex: &str) -> (Vec<String>, Vec<String>) {
  let (off_log, off) = run(tex, 0);
  assert_eq!(
    (error_count(&off_log), warning_count(&off_log)),
    (0, 0),
    "{off_log}"
  );
  let (log, two) = run(tex, 2);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  (creators_xml(&off), creators_xml(&two))
}

/// cas prints its one unlabelled `\affiliation` under the whole row of names, so with the switch off it is every author's
/// (`shared_row::share_row_contacts`, `_share=authors,annotate=run`); at `=2` the reading gave it to the first author's run
/// only. Witnesses 2506.17280, 2106.02306, 2201.00009, 2207.13923, 2411.19633, 2606.24054.
#[test]
fn a_cas_shared_affiliation_is_every_authors() {
  let (off, two) = both(repro!(
    "emit_cas_unlabelled_affiliation_after_first_author_shared"
  ));
  assert_eq!(two.len(), 3);
  for (creator, name) in two.iter().zip(["Ann Able", "Bob Baker", "Cat Cole"]) {
    assert!(
      creator.contains(&format!("<personname>{name}</personname>")),
      "{creator}"
    );
    assert!(creator.contains(">Univ A, Xland</contact>"), "{creator}");
  }
  assert_eq!(two, off);
}

/// authblk's `\affil[*]{Corresponding author. E-mail: …}` and `\affil[$\dagger$]{These authors contributed equally …}`
/// are their carriers' notes by the Kind step for a legend, which reads the label the binding prints: at `=2` the emitter
/// writes it as `_labelprints`'s value. Witnesses 2605.31330, 2606.17567, 2201.11910, 2602.03863, 2605.11756.
#[test]
fn a_printed_label_legend_is_its_carriers_note() {
  let (off, two) = both(repro!("emit_authblk_label_legend_is_its_carriers_note"));
  let notes = |kind: &str| {
    two
      .iter()
      .filter(|c| c.contains(&format!("ltx_thanks_{kind}")))
      .count()
  };
  assert_eq!(
    (notes("correspondence"), notes("contribution")),
    (1, 2),
    "{two:#?}"
  );
  assert!(two[0].contains(">ann@a.org</contact>"), "{}", two[0]);
  assert!(
    two
      .iter()
      .all(|c| !c.contains("role=\"affiliation\">Corresponding")),
    "{two:#?}"
  );
  assert_eq!(two, off);
}

/// A `\thanks` opening with a mark that is no number in a superscript (`\IEEEauthorrefmark{2}`) is a note, as the
/// switch-off API keeps it (`\lx@add@thanks` re-routes only a numbered one): at `=2` the legend of the mark's carriers,
/// both names showing †, as the charter's symbol-line rule reads it (off: Bob's alone, the note's owner). Witnesses
/// 1107.5730, 2205.02029.
#[test]
fn a_thanks_no_number_leads_is_a_note() {
  let (off, two) = both(repro!("emit_thanks_symbol_led_is_its_carriers_note"));
  let note = "class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\"><sup>†</sup> Also with the School of \
              Physics, Univ B.</note>";
  let holders = |xml: &[String]| xml.iter().map(|c| c.contains(note)).collect::<Vec<_>>();
  assert_eq!(holders(&two), [true, true, false], "{two:#?}");
  assert_eq!(holders(&off), [false, true, false], "{off:#?}");
  assert!(
    two.iter().all(|c| !c.contains("role=\"affiliation\">Also")),
    "{two:#?}"
  );
}

/// A `\thanks{\textsuperscript{1}These authors contributed equally …}` is re-routed as an entry and is then its
/// carriers' note by the Kind step, its mark the one the entry printed (recorded at `=2` as `_labelprints`'s value);
/// Cat Cole's mixed `\thanks{$^{2}$Lab C, Univ C. All correspondence …}` stays her affiliation (2102.08072's shape).
#[test]
fn a_numbered_thanks_legend_is_a_note() {
  let (off, two) = both(repro!("emit_thanks_numbered_legend_is_a_note"));
  let legend = |c: &String| c.contains("ltx_thanks_contribution");
  assert_eq!(
    two.iter().map(legend).collect::<Vec<_>>(),
    [true, true, false],
    "{two:#?}"
  );
  assert!(
    two[2].contains("role=\"affiliation\">Lab C, Univ C."),
    "{}",
    two[2]
  );
  assert_eq!(two, off);
}

/// An author's `\footnote{\textsuperscript{1}These authors contributed equally …}` is a note (`\lx@add@note` never
/// re-routes it), its carriers' at `=2` as off; it was an affiliation of both. Witness 2110.12892.
#[test]
fn a_numbered_footnote_legend_is_a_note() {
  let (off, two) = both(repro!("emit_footnote_numbered_legend_is_a_note"));
  let noted = |xml: &[String]| {
    xml
      .iter()
      .map(|c| {
        c.contains(
          "role=\"note\"><sup>1</sup>These authors contributed equally to this work.</contact>",
        )
      })
      .collect::<Vec<_>>()
  };
  assert_eq!(noted(&two), [true, true, false], "{two:#?}");
  assert_eq!(noted(&off), noted(&two));
  assert!(
    two
      .iter()
      .all(|c| !c.contains("role=\"affiliation\"><sup>1</sup>These")),
    "{two:#?}"
  );
}

/// A name whose font wrapper holds its mark and `\thanks` (`\textbf{Cat Cole$^{1}$\thanks{…}}`) is written from its own
/// tokens, the answered mark and the note the reading's: the note once (its text as the PDF's footnote, not bold). A
/// name written from a macro that prints its note too (`\newcommand{\ann}{Ann Able\thanks{…}}`) digests the note with
/// the name, which is not written again (the same cause). Witnesses 2309.13682, 2212.02936, 2305.16982; review revLO4.
#[test]
fn a_name_and_its_note_are_written_once() {
  let (off, two) = both(repro!("emit_name_in_wrapper_mark_and_thanks_apart"));
  assert_eq!(two[..2], off[..2]);
  assert!(
    two[2].contains("<personname>Cat Cole</personname>"),
    "{}",
    two[2]
  );
  assert_eq!(
    two[2].matches("Corresponding author").count(),
    1,
    "{}",
    two[2]
  );
  assert!(
    two[2].contains("ltx_thanks_correspondence\" role=\"thanks\">Corresponding author</note>"),
    "{}",
    two[2]
  );
  let (off, two) = both(repro!("emit_revtex_macro_name_with_thanks_written_once"));
  assert_eq!(
    two[0].matches("Corresponding author.").count(),
    1,
    "{}",
    two[0]
  );
  assert_eq!(two, off);
}

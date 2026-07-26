//! End-to-end .bib smoke test for the Pre::BibTeX parser →
//! BIBENTRY registry → \ProcessBibTeXEntry → <ltx:bibentry> flow.
//!
//! Bound to `--bibtex` mode by setting `Config.mode =
//! Some(DigestionMode::BibTeX)`. The wrapper TeX produced by
//! `PreBibTeX::to_tex()` (a `\begin{bibtex@bibliography}...` block of
//! `\ProcessBibTeXEntry{<key>}` calls) is pushed back into the gullet
//! via `input_content("literal:...")` from
//! `core_interface::digest` — see `core_interface.rs:307-327`.

use latexml::converter::Converter;
use latexml_core::common::{Config, DigestionMode, OutputFormat};

#[test]
fn bibtex_mode_emits_bibentries() {
  assert!(latexml_core::util::logger::init(log::LevelFilter::Warn).is_ok());
  let bib_source = "tests/bibtex/sample.bib";
  let opts = Config {
    format: OutputFormat::XML,
    mode: Some(DigestionMode::BibTeX),
    ..Config::default()
  };
  let mut converter = Converter::from_config(opts);
  converter.initialize_session().expect("can initialize");
  let resp = converter.convert(bib_source.to_string());
  let Some(doc) = resp.result.as_ref() else {
    panic!(
      "BibTeX conversion produced no document. status={} ({})",
      resp.status_code, resp.status
    );
  };
  let s = doc.to_string();

  assert!(
    s.contains("Smith2020"),
    "expected Smith2020 in output, got:\n{}",
    s
  );
  assert!(
    s.contains("Doe1999"),
    "expected Doe1999 in output, got:\n{}",
    s
  );
  // The bibtex.rs orchestration tags each entry with its type.
  assert!(
    s.contains("type=\"article\""),
    "expected type=\"article\", got:\n{}",
    s
  );
  assert!(
    s.contains("type=\"book\""),
    "expected type=\"book\", got:\n{}",
    s
  );
  // The @string-macro expansion produced "Theoretical Computer Science"
  // in the journal field.
  assert!(
    s.contains("Theoretical Computer Science"),
    "expected @string macro `tcs` to expand to 'Theoretical Computer Science', got:\n{}",
    s
  );
  // Regression: an UNKNOWN field (no dedicated handler) routes to
  // `\bib@field@unknownasdata`, which must emit its value as the content of
  // `<ltx:bib-data role='zzcustomfield'>`. The old code set the value via a
  // `Stored::Tokens` property in `after_digest` — too late AND the wrong Stored
  // type for `#prop` content-insertion — so the element came out EMPTY and the
  // value was dropped. It must now appear.
  assert!(
    s.contains("unknown-field marker value"),
    "expected unknown bib field value to be emitted (not dropped), got:\n{}",
    s
  );

  // `\bib@entry@default@complete` runs `\bib@synthesize@mr\bib@synthesize@zbl`
  // (Perl `BibTeX.pool.ltxml:210-211`, `:803-845`), turning the non-standard
  // mrnumber/mrreviewer/zblno fields into MathReview/ZentralBlatt links. Both
  // synthesizers read `currentBibEntryField`, which in Perl returns the field's
  // STRING; the Rust `BibEntry::fields` (Tokens) store it consulted is only ever
  // populated by crossref-copy, so every lookup returned None and NEITHER
  // synthesizer could fire. Expected shapes verified against same-host Perl.
  for needle in [
    // mrnumber alone -> a plain identifier, no review.
    "<bib-identifier href=\"https://www.ams.org/mathscinet-getitem?mr=849427\" \
     id=\"849427\" scheme=\"mr\">MathReview Entry</bib-identifier>",
    // mrnumber + mrreviewer -> a review naming the reviewer.
    "<bib-review href=\"https://www.ams.org/mathscinet-getitem?mr=2124018\" \
     id=\"2124018\" scheme=\"mr\">MathReview (C. Three)</bib-review>",
    // `MR1380882 (96e:83024)` -> review, `MR` prefix and note stripped from the id.
    "<bib-review href=\"https://www.ams.org/mathscinet-getitem?mr=1380882\" \
     id=\"1380882\" scheme=\"mr\">MathReview</bib-review>",
    // zblno -> ZentralBlatt review.
    "<bib-review href=\"https://zbmath.org/0674.53077\" id=\"0674.53077\" \
     scheme=\"zbl\">ZentralBlatt</bib-review>",
  ] {
    assert!(
      s.contains(needle),
      "MR/Zbl synthesis missing:\n  expected: {needle}\ngot:\n{s}"
    );
  }

  // Same root, different symptom: the `date`-already-set guard in
  // `\bib@field@default@year` also read the dead `fields` store, so an entry
  // carrying BOTH emitted a SECOND <ltx:bib-date> from the year. Perl emits one.
  let dates = s.matches("<bib-date").count();
  assert_eq!(
    dates, 4,
    "expected exactly one <ltx:bib-date> per entry (4 entries, and DateAndYear1986 \
     must not emit a second one from its `year`), got {dates}:\n{s}"
  );
}

/// A runaway field must cost only its own entry — not the rest of the `.bib`.
///
/// A bare `%` in a BibTeX field is literal data (BibTeX has no comment syntax
/// inside an entry) but TeX source when `\ProcessBibTeXEntry` replays the entry
/// through a Mouth, so it comments out the closing brace and the argument read
/// runs away. Both engines mis-read the field — that is parity, and real
/// bibtex+pdflatex break on it too, so it is deliberately NOT corrected here.
///
/// What was Rust-only was the blast radius. `read_balanced` used to treat every
/// autoclose literal mouth as a token-level continuation of its parent
/// (`gullet.rs`, the xint `\scantokens` divergence), so the runaway crossed out
/// of the entry mouth and swallowed every following `\ProcessBibTeXEntry` AND
/// `\end{bibtex@bibliography}`: a 3-entry `.bib` produced ONE entry and a single
/// error, where same-host Perl produces all three. The entry mouth now declares
/// itself `BalancedBoundary::Opaque`, which is what Perl's readBalanced
/// (`Gullet.pm` L465-472, current mouth only) does for every mouth.
///
/// Ground truth for the assertions is same-host Perl on the same fixture: all
/// three keys present, and the runaway title truncated at the `%` to "Fifty".
#[test]
fn runaway_field_costs_only_its_own_entry() {
  // `init` is process-global and the sibling test in this target may have won
  // the race; either outcome is fine here.
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
  let opts = Config {
    format: OutputFormat::XML,
    mode: Some(DigestionMode::BibTeX),
    ..Config::default()
  };
  let mut converter = Converter::from_config(opts);
  converter.initialize_session().expect("can initialize");
  let resp = converter.convert("tests/bibtex/runaway_field.bib".to_string());
  let Some(doc) = resp.result.as_ref() else {
    panic!(
      "BibTeX conversion produced no document. status={} ({})",
      resp.status_code, resp.status
    );
  };
  let s = doc.to_string();

  for key in ["before", "runawaytitle", "after"] {
    assert!(
      s.contains(&format!("key=\"{key}\"")),
      "entry '{key}' was lost — a runaway field must not take other entries \
       with it (same-host Perl keeps all three). Got:\n{s}"
    );
  }
  // The entries that carry no runaway must be intact, not merely present.
  assert!(
    s.contains("Entry after the runaway"),
    "the entry FOLLOWING the runaway lost its title, so the runaway is still \
     consuming past its own mouth. Got:\n{s}"
  );
  // And the damaged entry is damaged exactly as far as Perl's is: the title is
  // truncated at the `%`, the rest of that line gone.
  assert!(
    s.contains("<bib-title>Fifty</bib-title>"),
    "expected the runaway title truncated at the `%` (same as Perl), got:\n{s}"
  );
}

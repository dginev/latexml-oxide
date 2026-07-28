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
  // NOT `assert!(… .is_ok())`: `init` is process-global, and libtest runs this
  // target's tests concurrently, so exactly one of them wins the race and the
  // other's `init` legitimately returns Err. Asserting success made whichever
  // test lost fail — latent while this target held a single test, and immediate
  // once `runaway_field_costs_only_its_own_entry` joined it.
  let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
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

/// A `%` in a field is data, and a real runaway costs only its own entry.
///
/// **The `%` half.** A bare `%` in a BibTeX field is literal data — BibTeX has
/// no comment syntax inside an entry — but it used to become TeX source when
/// `\ProcessBibTeXEntry` replays the entry through a Mouth, so catcode 14 ate
/// the closing brace and the argument read ran away. That is now corrected on
/// both seams (OXIDIZED_DESIGN #74): the entry Mouth reads `%` as OTHER, and so
/// does `\bib@@title`, which never touches that Mouth — it re-reads the RAW
/// field to re-case it and tokenizes the result itself. Hence the fixture's two
/// `%` entries, one per seam. Perl still truncates both (`Fifty`, `plain`), so
/// these assertions are deliberately BEYOND same-host Perl; see #74 for why
/// pdflatex, not Perl, is the ground truth for a directly-read `.bib`.
///
/// **The blast-radius half.** `runawaymath`'s unescaped `$` still runs away —
/// it opens math that never closes, so the `Digested` argument (`\bib@@field`,
/// BibTeX.pool L230) digests to EOF. `readDigested` then `pop`s the last box to
/// strip the closing brace, and `digest_next_body` must have pushed Perl's EOF
/// trailer box (`Stomach.pm` L130) or that `pop` eats real content — every
/// following entry. The gullet-level twin of that guarantee is the entry
/// mouth's `BalancedBoundary::Opaque` (Perl's readBalanced reads the current
/// mouth only, `Gullet.pm` L465-472); with `%` retired as a trigger, no `.bib`
/// input reachable through `Pre::BibTeX` produces a token-level unbalanced read
/// any more — the parser balances braces character-wise before we ever see the
/// value — so that boundary is now an invariant this fixture states rather than
/// one it can provoke.
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

  for key in [
    "before",
    "runawaytitle",
    "runawaynote",
    "runawaymath",
    "after",
  ] {
    assert!(
      s.contains(&format!("key=\"{key}\"")),
      "entry '{key}' was lost — a runaway field must not take other entries \
       with it. Got:\n{s}"
    );
  }
  // The entries that carry no runaway must be intact, not merely present.
  assert!(
    s.contains("Entry after the runaway"),
    "the entry FOLLOWING the runaway lost its title, so the runaway is still \
     consuming past its own mouth. Got:\n{s}"
  );
  // The `%` is data: both seams must keep the whole value, not stop at it.
  // `\bib@@title` re-tokenizes the raw field itself …
  assert!(
    s.contains("<bib-title>Fifty % of the time</bib-title>"),
    "the title was truncated at its `%` — a `%` in a .bib field is data, not a \
     comment (OXIDIZED_DESIGN #74). Got:\n{s}"
  );
  // … while the note comes straight off the entry Mouth.
  assert!(
    s.contains("plain % comment"),
    "the note was truncated at its `%` — the entry Mouth must read `%` as an \
     ordinary character (OXIDIZED_DESIGN #74). Got:\n{s}"
  );
}

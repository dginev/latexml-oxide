//! Guards for ERR2 (SYNC row 0b item 39, OXIDIZED_DESIGN_DIVERGENCES #983): an author block's undefined commands, whose
//! Errors the observer's rehearsal holds and discards with their stubs, are raised again — once, as Perl's one
//! digestion of the block raises them (State.pm:547-551, Base_Utility.pool.ltxml:691-693). In place where a piece the
//! emitter digests holds the command: right after a name (`content::Field::past_held_undefined`), past its marks and
//! notes (`emit::block_names`), and between two inks it glues, which no reading of addresses takes
//! (`events::Event::glued_undef`: an undefined command never changes the segmentation). Else at the flush's end
//! (`reading::HeldUndefined::raise_unraised`), for a piece written as text. Each guard asserts every creator's name and
//! contacts, in order, and the exact Error and Warning counts; witnesses 1101.4098, 1402.2906, 1808.09392, 2205.12847,
//! 2309.09045, 2405.06973, 2405.13308, 2506.10896, 2508.08170, 2508.21091, 2601.17318, 2602.18351, 2604.26688,
//! 2607.29185, 2609.00161, hep-th9306034.

use super::{
  perfect_kernel_batch46::{convert, error_count, warning_count},
  perfect_kernel_batch84b::creators,
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

fn row(items: &[&str]) -> Vec<String> { items.iter().map(ToString::to_string).collect() }

/// The undefined command names of the log's Errors, in order.
fn undefined_errors(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|line| line.strip_prefix("Error:undefined:"))
    .filter_map(|rest| rest.split(' ').next())
    .collect()
}

/// A piece written as text (a name its events don't spell, counted by its Warning) never digests its tokens: the Error
/// its rehearsal held is raised by the flush's net, once (`reading::HeldUndefined`; 2609.00161's `\aaai@corrmultitrue`).
/// Distinct: the one path where no piece raises it in place.
#[test]
fn an_undefined_command_in_a_text_written_name_errs_once_at_the_flush() {
  let (log, xml) = convert(
    repro!("emitter_undefined_command_in_a_name_written_as_text_errs_once"),
    false,
  );
  assert_eq!(undefined_errors(&log), ["\\undefinedprobecs"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 1), "{log}");
  assert!(
    log.contains("The frontmatter emitter writes \"Ann Able\" as its plain text"),
    "{log}"
  );
  assert_eq!(creators(&xml), [row(&["Ann Able"]), row(&["Bob Baker"])]);
}

/// The flip A/B's lost `Error:expected:\fi` chains (1603.08231: 18 off, 2112.06251: 12) are no Errors of the paper's: an
/// `\and` inside a conditional (sibarticle.sty's `\ifnum \c@numAuthors>1 \and …\fi`) is skipped with the name after it,
/// as TeX reads the block in one pass (pdflatex 0 errors, "Ann Able" alone); Perl's split at the `\and` tokens cuts each
/// conditional from its `\fi` (KNOWN_PERL_ERRORS #1218: 1 Error and a creator Bob Baker pdflatex never prints). Distinct:
/// the classification's evidence, a conditional across the block's separators.
#[test]
fn an_and_inside_a_conditional_is_read_as_tex_reads_it() {
  let (log, xml) = convert(
    repro!("author_and_inside_a_conditional_reads_as_tex"),
    false,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(creators(&xml), [row(&["Ann Able"])]);
}

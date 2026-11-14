//! Guard for RA0 (OXIDIZED_DESIGN_DIVERGENCES #983, revPC3 finding 2): a command the class's title code defines where
//! it reads `\@author` and that reads no argument is taken by the name it follows and prints there
//! (`title_code::reads_arguments`: `\newcommand`'s `[0]` is no parameter count). The predicate's reading of each
//! definition form is pinned by `title_code`'s unit test `reads_arguments_counts_parameters_not_brackets`; the shape
//! after an `\email{…}` is the catalog repro `title_code_command_with_no_parameters_after_an_email_red`.

use super::{
  perfect_kernel_batch46::{convert, error_count, warning_count},
  perfect_kernel_batcherr2::creator_elements,
};

/// A title-code `\newcommand{\address}[0]{\textsuperscript{x}}` after a name (synthetic, revPC3's `s1b` without its
/// `\email`): the name takes `\address` and its "x" prints, digested under the title code's definitions; every
/// creator element whole, no Error or Warning (int156a0: the "x" in no output, uncounted). Distinct: the
/// `\newcommand`/`\providecommand` count's branch, end to end.
#[test]
fn a_title_code_command_declared_with_no_parameters_is_taken() {
  let (log, xml) = convert(
    include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/title_code_command_declared_with_no_parameters_is_taken.tex"
    ),
    false,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains("<ERROR"), "{xml}");
  assert_eq!(creator_elements(&xml), [
    "<creator role=\"author\">\n    <personname>Ann Able<sup>x</sup></personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n  </creator>",
  ]);
}

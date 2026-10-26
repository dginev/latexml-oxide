//! Red/green guards for batch 81b (hyperref's autoref tag runs the kernel's `\null`, whatever the author made of it).
//! Each guard converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the
//! elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The autoref tag every numbered target builds (`\lx@autorefnum@@`, DIVERGENCES #418) ends with the token `\null`, the
/// delimiter of hyperref's `\def\equationautorefname~#1\null{…}` idiom, and runs it as the kernel's empty box: an
/// author's `\def\null0{…}` hiding a subsection is never run there, as pdflatex runs `\null` only where an `\autoref`
/// prints. Red (int110h): Fatal Stomach:Recursion, `\section` → its tag → the author's `\subsection`. Witness
/// 1501.00090. Repro cross-references/hyperref_autoref_tag_runs_the_kernel_null.
#[test]
fn an_authors_null_stays_out_of_the_autoref_tag() {
  let (log, xml) = convert_files_with(
    repro!("cross-references/hyperref_autoref_tag_runs_the_kernel_null.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("<tag role=\"autoref\">section\u{a0}2<"),
    "{xml}"
  );
  assert!(!xml.contains("Hidden"), "{xml}");
}

/// xargs.sty:205-212: a `\newcommandx` command with an optional argument is `\@protected@testopt(@xargs)\cs…`, which
/// stays unexpanded (`\@x@protect`) whenever `\protect` is not `\@typeset@protect` — inside ifthen's `\xdef` of a test
/// (ifthen.sty:92-117) an `\equal` argument holding such a command keeps it as a token. Red (int110m): `\sequence`
/// expanded there and ran its own `\ifthenelse`, Fatal "File ended while scanning use of a Until:\NOT argument"
/// (2002.09653). Repro package-bindings/xargs_optional_command_is_protected_in_ifthen_test.
#[test]
fn an_xargs_optional_command_is_protected_in_an_ifthen_test() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/xargs_optional_command_is_protected_in_ifthen_test.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"tex="\{a\mid(1)\}""#), "{xml}");
}

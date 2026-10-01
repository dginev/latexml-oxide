//! Red/green guards for perfect-kernel phase-59 batches: the TikZ performance levers (59a) and
//! the wheelchart pgfmath roots (59b).
use super::perfect_kernel_batch57::{RAW, assert_elements};

/// 59a: number scanning keeps the token after the signs in hand (tex.web §440), but an undefined
/// conditional met there is read again as its error stub (`\iffalse`, State.pm:537-545), as Perl's
/// chain of reads does: `\ifnum\ifdraft 1\else 0\fi=0` is true. Kept in hand, it fell to "Missing
/// number". Repro expansion-primitives/undefined_conditional_in_a_number_expands.
#[test]
fn undefined_conditional_in_a_number_expands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/undefined_conditional_in_a_number_expands.tex"
  );
  assert_elements(tex, RAW, (1, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>yes.</p></para>"##,
  )]);
}

/// 59a: the same in a dimension scan (`read_dimension`): the stub is read again before the
/// internal dimension and glue are tested, so `\ifcompact\columnwidth\else\textwidth\fi` is
/// `\textwidth` (kept in hand it was 0pt and the left-over `\textwidth` assigned 0 to itself).
/// Repro expansion-primitives/undefined_conditional_in_a_dimension_expands.
#[test]
fn undefined_conditional_in_a_dimension_expands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/undefined_conditional_in_a_dimension_expands.tex"
  );
  assert_elements(tex, RAW, (3, 0), &[
    (
      "para",
      "p1",
      r##"<para xml:id="p1"><p>A:[345.0pt]</p></para>"##,
    ),
    (
      "para",
      "p2",
      r##"<para xml:id="p2"><p>B:[7.0pt]</p></para>"##,
    ),
    ("para", "p3", r##"<para xml:id="p3"><p>C:yes.</p></para>"##),
  ]);
}

/// 59a: an undefined conditional that ends a factor's digits is read again as its stub before
/// the decimal point is tested (Perl `readFactor`, Gullet.pm:876-880): `1\ifFa\else.5\fi pt` is
/// 1.5pt, not 1pt and an "Illegal unit". Repro
/// expansion-primitives/undefined_conditional_after_digits_expands.
#[test]
fn undefined_conditional_after_digits_expands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/undefined_conditional_after_digits_expands.tex"
  );
  assert_elements(tex, RAW, (1, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>A:[1.5pt]</p></para>"##,
  )]);
}

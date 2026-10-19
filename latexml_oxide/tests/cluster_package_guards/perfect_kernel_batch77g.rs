//! Red/green guard for batch 77g: `\meaning` of the kernel's stand-ins for TeX primitives — the `\over` family,
//! `\overline`, `\underline` (tex_math.rs macros) — is the primitive's name, as tex.web §296 prints a primitive, while
//! the definition is LaTeXML's (tex_debugging.rs `KERNEL_PRIMITIVE_STANDINS_FOR_MEANING`; Perl
//! TeX_Debugging.pool.ltxml:125-127's exception list, extended). amsgen's `\@saveprimitive` check then saves them
//! (asl.cls:327 — 1005.4193, 1803.03483, 1111.1504; KNOWN_PERL_ERRORS #985). The guards assert the three measures: no
//! Error, the fraction kept as a division, the meanings TeX prints.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// amsgen's `\@saveprimitive` saves `\over` and `\overline`, and a fraction built on the saved `\@@over` divides.
#[test]
fn saveprimitive_accepts_kernel_fraction_primitives() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/expansion-primitives/saveprimitive_kernel_fractions.tex"
    ),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<Math mode="inline" tex="{a\over b}""#),
    "{xml}"
  );
  assert!(xml.contains(r#"meaning="divide""#), "{xml}");
  assert!(
    xml.contains(r#"<Math mode="inline" tex="\overline{x}""#),
    "{xml}"
  );
}

/// A `\let` copy of a stand-in reads as the primitive; a document's own `\def\over` is a macro again.
#[test]
fn kernel_primitive_standins_mean_their_primitive() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\begin{document}\n\\makeatletter\\let\\@@over\\over\n\\texttt{A:\\meaning\\@@over; \
     U:\\meaning\\underline; \\def\\over{x}B:\\meaning\\over}\n\\end{document}",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<text font="typewriter">A:\over; U:\underline; B:macro:-&gt;x"#),
    "{xml}"
  );
}

//! W3RD (PR #829 Rhai round D): diagnostics as Perl's (Common/Error.pm), the Package.pm options Rhai dropped, and the
//! thin wrappers over existing core calls. Each row is pinned against Perl `latexml` with the equivalent `.ltxml`
//! binding, or against TeX where Perl does not check.
#![cfg(feature = "runtime-bindings")]

use latexml::util::test::{assert_element, convert_files_with};

use crate::perfect_kernel_batch46::{error_count, warning_count};

const LXRD_ERR: &str = r#"DefPrimitive("\\badcodes", || { AssignCatcode("Q", 16); AssignDelcode("Q", -1); AssignDelcode("(", 164608); });
"#;

const LXRD_ERR_TEX: &str = r"\documentclass{article}
\usepackage{lxrderr}
\begin{document}
\badcodes\the\catcode`\Q
\end{document}
";

/// The Error side of the round's boundary. e1 (revW3e): an `Assign*code` value the table cannot hold is an Error
/// naming the call (TeX: "Invalid code (16), should be at most 15"; a delcode above "FFFF is the u16 table's limit,
/// `macro-state/delcode_holds_24_bits_red.tex`); red on f528beee1fb, which dropped all three silently (E=0).
#[test]
fn rhai_diagnostics_error_side() {
  let (log, xml) = convert_files_with(LXRD_ERR_TEX, &[("lxrderr.sty.rhai", LXRD_ERR)], None);
  assert_eq!(error_count(&log), 3, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for call in [
    "AssignCatcode(\"Q\", 16)",
    "AssignDelcode(\"Q\", -1)",
    "AssignDelcode(\"(\", 164608)",
  ] {
    assert!(
      log.contains(&format!(
        "Error:unexpected:{} {call}",
        call.split('(').next().unwrap()
      )),
      "{call}\n{log}"
    );
  }
  assert_element(&xml, "para", &[], r#"<para xml:id="p1"><p>11</p></para>"#);
}

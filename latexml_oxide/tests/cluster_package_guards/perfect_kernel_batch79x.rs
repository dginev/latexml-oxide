//! Red/green guards for batch 79x (complete breakage, continued: user 2026-10-24): `\@ifnextchar` and `\@ifstar` leave
//! the next token in `\@let@token`; a package's own `\zap@space` decides the option list stored in `\opt@<file>`. Each
//! guard converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// sect13.rs `let_peeked_token`: the native `\@ifnextchar` and `\@ifstar` let `\@let@token` to the token they peeked
/// (past spaces), as latex.ltx:1756-1771's `\futurelet\@let@token\@ifnch` does, so the branch taken can test it: each
/// peek names the marker, and a row scanner that asks `\ifx\@let@token\extracode` in `\@ifnextchar\end`'s false
/// branch (tikz-timing.sty:1156-1170 `\tikztimingtable@checkrow`) keeps its two rows, its extra code and the text after
/// the list. Red (int92c): "Next is something else" ×3, `\extracode` read as a row up to the end of the file ("File
/// ended while scanning use of a Until:\\ argument", Fatal), the rows and "After." lost. Witness 2409.18120. Repro
/// macro-state/ifnextchar_leaves_the_next_token_in_let_token.
#[test]
fn ifnextchar_leaves_the_next_token_in_let_token() {
  let (log, xml) = convert_files_with(
    repro!("macro-state/ifnextchar_leaves_the_next_token_in_let_token.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("Next is the marker.").count(), 2, "{xml}");
  assert!(xml.contains("Starless, next is the marker."), "{xml}");
  assert!(!xml.contains("something else"), "{xml}");
  for text in ["Row: first.", "Row: second.", "Extra code.", "After."] {
    assert!(xml.contains(text), "{text}: {xml}");
  }
}

/// content.rs `zapped_options`: `\opt@<file>` is latex.ltx:18514-18517's `\zap@space#2 \@empty` under whatever
/// `\zap@space` means at the load: a package's own (`#1 \@empty{#1}`, tikz-timing.sty:2331-2341 around its library
/// loads) keeps the list as given, "rising arrows,falling edge"; the kernel's drops every blank, "sharecounter,xy".
/// Red (int92c): "risingarrows,fallingedge", the key tikz-timing-clockarrows declares as `rising arrows` unknown.
/// Witness 2409.18120. Repro loader/package_zap_space_keeps_option_spaces.
#[test]
fn a_package_zap_space_keeps_the_option_spaces() {
  let (log, xml) = convert_files_with(
    repro!("loader/package_zap_space_keeps_option_spaces.tex"),
    &[
      ("spacedopts.sty", repro!("loader/spacedopts.sty")),
      ("zappedopts.sty", repro!("loader/zappedopts.sty")),
    ],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains("Spaced: [rising arrows,falling edge]."),
    "{xml}"
  );
  assert!(xml.contains("Kernel: [sharecounter,xy]."), "{xml}");
}

//! Red/green guards for batch 68v: undefined-CS binding gaps — package internals our bindings lacked (`\symAMSb`
//! through `\try@load@fontshape`, empheq's `overload` and `\empheqlbrace`, …) and class macros of shipped classes.
//! Every repro's pdflatex reference is in its header.
use latexml::util::test::{convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A repro under `tools/perfect_kernel/repros/`, converted with the ar5iv preload: its (errors, warnings) and the
/// squashed XML.
macro_rules! repro {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      Some("ar5iv.sty"),
    );
    ((error_count(&log), warning_count(&log)), log, squash(&xml))
  }};
}

/// A repro beside the files it ships (`(name, path under repros/)`), converted with the ar5iv preload.
#[allow(unused_macros)]
macro_rules! repro_with {
  ($path:literal, $( ($name:literal, $file:literal) ),+ ) => {{
    let (log, xml) = convert_files_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      &[ $( ($name, include_str!(concat!("../../../tools/perfect_kernel/repros/", $file))) ),+ ],
      Some("ar5iv.sty"),
    );
    ((error_count(&log), warning_count(&log)), log, squash(&xml))
  }};
}

fn assert_all(xml: &str, parts: &[&str]) {
  for part in parts {
    assert!(xml.contains(part), "{part}\n{xml}");
  }
}

/// fonts-nfss/amsfonts_try_load_fontshape: amsart's amsfonts test (amsart.cls:338-350, copied by nyjm.cls 2401.06615,
/// cedram.cls 2105.03188, myart.cls 1506.02470) finds umsa.fd through `\try@load@fontshape`, whose
/// `\DeclareFontShape` defines `\U/msa/m/n`; amsfonts loads, so `\symAMSb` is the AMSb symbol font.
#[test]
fn batch68v_try_load_fontshape_finds_amsfonts() {
  let (counts, log, xml) = repro!("fonts-nfss/amsfonts_try_load_fontshape.tex");
  assert_eq!(counts, (0, 0), "{log}");
  assert!(!log.contains("amsfonts NOT loaded"), "{log}");
  assert_all(&xml, &[
    r#"<XMTok name="varnothingx" role="ID">∅</XMTok>"#,
    r#"<Math mode="inline" tex="\mathbb{R}" text="R" xml:id="p1.m2">"#,
  ]);
}

/// package-bindings/empheq_overload_left: `\usepackage[overload]{empheq}` gives the AMS environments empheq's options
/// (empheq.sty:961-1066); `left` opens the first cell, `right` closes the last, `\empheqlbrace` is the brace grown to
/// the display (1410.0933, 1906.02657, 2210.09120, 2201.03303, 2207.01386). The option list is no longer typeset.
/// The 7 warnings are the math parser's on the empty fence `\left\{\right.` (math parsing is a separate stream).
#[test]
fn batch68v_empheq_overload_left_and_right() {
  let (counts, log, xml) = repro!("package-bindings/empheq_overload_left.tex");
  assert_eq!(counts, (0, 7), "{log}");
  assert!(!xml.contains("left="), "{xml}");
  assert_all(&xml, &[
    r#"tex="\displaystyle F(x)=\left\{\right.x""#,
    r#"tex="\displaystyle y=2""#,
    r#"tex="\displaystyle=3\left.\right\}""#,
    r#"tex="\displaystyle G=\left\{\right.a""#,
    r#"tex="\displaystyle b=2""#,
    r#"tex="\left\{\right.c=d""#,
    r#"<tags> <tag>(4)</tag> <tag role="refnum">4</tag> </tags>"#,
  ]);
}

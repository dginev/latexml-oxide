//! Red/green guards for batch 67l: which algorithm and float definitions a paper gets when a binding stands in for
//! the file that would load them (run 349 "undefined" clusters `\State` 560, `\For` 470, `{algorithm}` 544,
//! `\newfloat` 365).
use latexml::util::test::{convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

macro_rules! repro {
  ($path:literal) => {
    convert_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      Some("ar5iv.sty"),
    )
  };
}

/// The paper's icml2017.sty has `algorithmic` edited out (icml2017.sty:73 of 1706.03643's copy; icml2024.sty:94 "removed
/// due to conflict with algpseudocode", 2402.05403) and the paper loads algpseudocode: the copy decides, so algorithmicx
/// is not bailed and `\State`/`\For` are defined (1706.03643, 1706.04933, 1706.04964, 2002.06473, 2106.05074).
#[test]
fn icml_style_copy_decides_the_algorithm_packages() {
  let (log, xml) = repro!("package-bindings/icml_style_copy_without_algorithmic.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<float class="ltx_float_algorithm" framed="top" inlist="loa" xml:id="alg1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<caption><tag close=" "><text font="bold" xml:id="alg1.2">Algorithm 1</text></tag> Sum</caption>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<listingline ").count(), 4, "{xml}");
  assert!(
    xml.contains(
      r#"<listingline xml:id="alg1.l4"><tags> <tag><text fontsize="80%" xml:id="alg1.l4.1">4:</text></tag> <tag role="refnum">4</tag> </tags><text font="bold" xml:id="alg1.l4.2">end</text> <text font="bold" xml:id="alg1.l4.3">for</text></listingline>"#
    ),
    "{xml}"
  );
}

/// Without a copy of the style in the source the binding keeps the pair ICML distributes (icml2017.sty:72-73):
/// algorithmic's `\STATE` and `\FOR` are defined.
#[test]
fn icml_style_without_a_copy_keeps_the_distributed_pair() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{icml2017}\n\\begin{document}\n\\begin{algorithm}\n\\caption{Sum}\n\
     \\begin{algorithmic}\n\\STATE $x$\n\\FOR{$i$}\n\\STATE $y$\n\\ENDFOR\n\\end{algorithmic}\n\\end{algorithm}\n\
     \\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<listingline ").count(), 4, "{xml}");
}

/// revtex4-2.cls:4022-4042 (revtex4-1.cls:4001-4021) `\newfloat@ltx`, the class's repaired float.sty `\newfloat`, which a
/// preamble lets `\newfloat` to before `\usepackage{algorithm}` (2210.07980, 2210.08566, 2210.07097, 2402.00653).
#[test]
fn revtex_float_repair_defines_newfloat() {
  let (log, xml) = repro!("class-bindings/revtex_float_repair_before_document.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<float class="ltx_float_algorithm" framed="top" inlist="loa" xml:id="alg1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<caption><tag close=" "><text font="bold" xml:id="alg1.3">Algorithm 1</text></tag> Sum</caption>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<listingline ").count(), 2, "{xml}");
}

/// REVTeX 4.0 has no float repair (revtex4.cls): the copies stay undefined there, as in TeX.
#[test]
fn revtex4_0_has_no_float_repair() {
  let (log, xml) = convert_with(
    "\\documentclass{revtex4}\n\\begin{document}\n\\makeatletter\n\
     [\\ifdefined\\newfloat@ltx defined\\else undefined\\fi]\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("[undefined]"), "{xml}");
}

/// WileyNJD-v2.cls:3331 (WileyNJDv5.cls:6820) `\usepackage{algorithm, algorithmicx, algpseudocode}`: the class's
/// copy loads them for a paper that does not (1810.03033, 1810.12033, 2106.06046, 2402.13183).
#[test]
fn wileynjd_class_copy_loads_its_algorithm_packages() {
  let (log, xml) = repro!("class-bindings/wileynjd_class_copy_loads_algorithm_packages.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(
      r#"<caption><tag close=" "><text font="bold" xml:id="alg1.3">Algorithm 1</text></tag> Sum</caption>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<listingline ").count(), 4, "{xml}");
  assert!(
    xml.contains(
      r#"</tags><text font="bold" xml:id="alg1.l4.3">end</text> <text font="bold" xml:id="alg1.l4.4">for</text></listingline>"#
    ),
    "{xml}"
  );
}

/// A WileyNJD-v2 copy that loads algorithm2e instead (2002.02812's WileyNJD-v2.cls:3338): algorithm2e's syntax, and
/// no algpseudocode.
#[test]
fn wileynjd_class_copy_with_algorithm2e() {
  let (log, xml) = convert_files_with(
    "\\documentclass{WileyNJD-v2}\n\\begin{document}\n\\begin{algorithm}\n\\caption{Sum}\n\
     \\KwIn{$n$}\n$x\\leftarrow 0$\\;\n\\end{algorithm}\n\\end{document}\n",
    &[(
      "WileyNJD-v2.cls",
      "\\ProvidesClass{WileyNJD-v2}\n\\LoadClass{article}\n%\\usepackage{algorithm, algorithmicx, algpseudocode}\n\
       \\usepackage[ruled]{algorithm2e}\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(log.contains("algorithm2e"), "{log}");
  assert!(!log.contains("algpseudocode"), "{log}");
  assert!(
    xml.contains(r#"<float class="ltx_algorithm" framed="top" xml:id="algorithm1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<text font="bold" xml:id="algorithm1.4.1.1.1">Input:</text>"#),
    "{xml}"
  );
  assert_eq!(xml.matches("<listingline ").count(), 2, "{xml}");
}

/// bmvc2k.cls's copy loads `algorithm` and `algorithmic` (2210.01640's:162-163, 2002.09958's:132-133); 2210.06270's
/// loads `algorithm` and `algpseudocode`.
#[test]
fn bmvc2k_class_copy_loads_its_algorithm_packages() {
  let (log, xml) = repro!("class-bindings/bmvc2k_class_copy_loads_algorithmic.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(
      r#"<caption><tag close=" "><text font="bold" xml:id="alg1.3">Algorithm 1</text></tag> Sum</caption>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<listingline ").count(), 4, "{xml}");
}

/// autart.cls:568-577 defines its own `{algorithm}` float, captioned "Algorithm N." in bold, listed in `lof`
/// (2106.14831, 2402.17000).
#[test]
fn autart_defines_its_algorithm_float() {
  let (log, xml) = repro!("class-bindings/autart_algorithm_float.tex");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let xml = squash(&xml);
  assert!(
    xml.contains(r#"<float class="ltx_float_algorithm" inlist="lof" xml:id="alg1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(
      r#"<caption><tag close=". "><text font="bold" xml:id="alg1.2">Algorithm 1</text></tag>Sum</caption>"#
    ),
    "{xml}"
  );
  assert_eq!(xml.matches("<listingline ").count(), 2, "{xml}");
}

const ALGPSEUDOCODE_BODY: &str = "\\begin{document}\n\\begin{algorithm}\n\\caption{Sum}\n\\begin{algorithmic}\n\
                                  \\State $x$\n\\For{$i$}\n\\State $y$\n\\EndFor\n\\end{algorithmic}\n\
                                  \\end{algorithm}\n\\end{document}\n";

/// A versioned ICML style no binding is named for (`icml2026`, or `./icml2018` as 1706.04964 asks for it) falls back to
/// icml.sty's binding, which follows the requested copy as the named ones do.
#[test]
fn icml_versioned_fallback_follows_the_copy() {
  let (log, xml) = convert_files_with(
    &format!(
      "\\documentclass{{article}}\n\\usepackage{{icml2026}}\n\\usepackage{{algpseudocode}}\n{ALGPSEUDOCODE_BODY}"
    ),
    &[(
      "icml2026.sty",
      "\\ProvidesPackage{icml2026}\n\\RequirePackage{algorithm}\n% \\RequirePackage{algorithmic}\n\\endinput\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<listingline ").count(), 4, "{xml}");
}

/// A neurips style copy edited to load the algorithm packages (2402.18905's neurips_2023.sty:1-2) gives them to a paper
/// that relies on it; the distributed neurips styles load none.
#[test]
fn neurips_copy_loads_its_algorithm_packages() {
  let (log, xml) = convert_files_with(
    &format!("\\documentclass{{article}}\n\\usepackage{{neurips_2023}}\n{ALGPSEUDOCODE_BODY}"),
    &[(
      "neurips_2023.sty",
      "\\usepackage{algorithm}\n\\usepackage{algpseudocode}\n\\ProvidesPackage{neurips_2023}\n\\endinput\n",
    )],
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<listingline ").count(), 4, "{xml}");
}

/// AASTeX 6.x loads revtex4-1 (aastex62.cls:278-292) and so has its float repair (2002.07183).
#[test]
fn aastex6_float_repair_defines_newfloat() {
  let (log, xml) = convert_with(
    "\\documentclass{aastex62}\n\\usepackage{float}\n\\makeatletter\n\\let\\newfloat\\newfloat@ltx\n\\makeatother\n\
     \\usepackage{algorithm}\n\\usepackage{algorithmic}\n\\begin{document}\n\\begin{algorithm}\n\\caption{Sum}\n\
     \\begin{algorithmic}\n\\STATE $x$\n\\end{algorithmic}\n\\end{algorithm}\n\\end{document}\n",
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"<float class="ltx_float_algorithm""#),
    "{xml}"
  );
  assert_eq!(xml.matches("<listingline ").count(), 1, "{xml}");
}

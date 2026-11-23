//! Guards for REGTAILA (row R REGTAIL): papers clean on an old Rust build that now err. Each converts its repro and
//! asserts the exact Error and Warning counts and the whole structure the mechanism decides.

use latexml::util::test::convert_files_with;
use regex::Regex;

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The body's markup with ids, theorem tags and titles masked and the whitespace between elements dropped.
fn body_skeleton(xml: &str) -> String {
  let masks = [
    r#"\s(xml:id|inlist|class)="[^"]*""#,
    r"(?s)<tags>.*?</tags>",
    r"(?s)<title\b.*?</title>",
    r"(?s)<resource\b[^>]*/>",
  ];
  let mut out = xml[xml.find("<document").expect(xml)..].to_string();
  for mask in masks {
    out = Regex::new(mask).unwrap().replace_all(&out, "").into_owned();
  }
  Regex::new(r">\s+<")
    .unwrap()
    .replace_all(out.trim(), "><")
    .into_owned()
}

/// A theorem whose start a document wrapped (`\let\lutzremark=\remark \def\remark{\lutzremark\normalfont}`) and whose
/// end it redefined (`\def\endremark{\vspace{1ex}}`): `\begin` runs the wrapper, so the frame is the bare start's, and
/// `\end`'s `\endgroup` ends it with the binding's own end, whatever `\endremark` became. Nested, wrapped at both ends,
/// an end that is `\relax`, and the bare start closed by a brace group. Red (INT081G): 4 errors, "After." and the rest
/// of the document inside the first theorem (1907.13475). Repro macro-state/wrapped_newtheorem_start_with_redefined_end.
#[test]
fn wrapped_newtheorem_start_with_redefined_end() {
  let (log, xml) = convert_files_with(
    repro!("macro-state/wrapped_newtheorem_start_with_redefined_end.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let theorem = |text: &str| format!("<theorem><para><p>{text}</p></para></theorem>");
  let para = |text: &str| format!("<para><p>{text}</p></para>");
  let italic = |text: &str| format!(r#"<text font="italic">{text}</text>"#);
  let expected = [
    theorem("Body text."),
    para("After."),
    format!(
      "<theorem>{}<para><p>Outer.</p></para></theorem>",
      theorem("Inner.")
    ),
    para("Mid."),
    theorem(&italic("Lemma text.")),
    para("After lemma."),
    theorem(&italic("Claim text.")),
    para("After claim."),
    theorem("Bare text."),
    para("After bare."),
  ]
  .concat();
  assert_eq!(
    body_skeleton(&xml),
    format!(r#"<document xmlns="http://dlmf.nist.gov/LaTeXML">{expected}</document>"#),
    "{xml}"
  );
}

/// siunitx `S` cells led by `\centering`/`\raggedleft` (and a `>{\centering\arraybackslash}S` prefix): latex.ltx's
/// three are robust, so the cell's peek for `\omit` stops at `\protect` and siunitx reads the command whole; a leading
/// control sequence is peeled and the number after it parsed (Perl `CC_CS`). Red (INT081G): 8 errors, the cells split
/// at the `\\` the `\let` names (2110.03080, 2203.16677). Repro package-bindings/siunitx_s_cell_with_centering.
#[test]
fn siunitx_s_cell_with_centering() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/siunitx_s_cell_with_centering.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let math = Regex::new(r#"(?s)<Math [^>]*tex="([^"]*)".*?</Math>"#).unwrap();
  let flat = math.replace_all(&xml, "[$1]");
  let cell = Regex::new(r"(?s)<td\b[^>]*>(.*?)</td>").unwrap();
  let tables: Vec<Vec<String>> = Regex::new(r"(?s)<tabular\b.*?</tabular>")
    .unwrap()
    .find_iter(&flat)
    .map(|table| {
      cell
        .captures_iter(table.as_str())
        .map(|c| c[1].to_string())
        .collect()
    })
    .collect();
  let bold = |text: &str| format!(r#"<text font="bold">{text}</text>"#);
  assert_eq!(
    tables,
    [
      vec![
        bold("Data"),
        r#"<text align="center" font="bold">Total</text>"#.to_string(),
        bold("Min."),
        "A".to_string(),
        r"[83\,635]".to_string(),
        "[12]".to_string(),
      ],
      vec!["[1.5]".to_string(), "[12]".to_string()],
      vec![
        "x".to_string(),
        "[3.1]".to_string(),
        "y".to_string(),
        "Text".to_string(),
      ],
      vec!["Head".to_string(), "[2.5]".to_string()],
      vec!["[7.25]".to_string(), "[8]".to_string()],
      vec!["Total".to_string(), "[12]".to_string()],
      vec!["[12]".to_string()],
    ],
    "{xml}"
  );
  assert!(flat.contains("</tabular>\nAfter.</p>"), "{xml}");
}

/// `\mathchoice` has the size of the current style's branch only (tex.web §731): in text, display and script style,
/// four equal branches, and `\mathpalette`. Red (INT081G): every branch summed, so an xy node holding one was too wide
/// for its diagram (1504.04186, tensor.sty's `\mathpalette`; 2 errors). Repro
/// math-phantom/mathchoice_sizes_only_its_style_branch.
#[test]
fn mathchoice_sizes_only_its_style_branch() {
  let (log, xml) = convert_files_with(
    repro!("math-phantom/mathchoice_sizes_only_its_style_branch.tex"),
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let widths: Vec<&str> = Regex::new(r"[TDXSP]:[0-9.]+pt\.")
    .unwrap()
    .find_iter(&xml)
    .map(|m| m.as_str())
    .collect();
  assert_eq!(
    widths,
    [
      "T:0.0pt.",
      "D:22.86108pt.",
      "X:5.71527pt.",
      "S:24.35419pt.",
      "P:11.43054pt."
    ],
    "{xml}"
  );
  // the xy node of a wide display branch in text style (root-causer's repro): 0 errors
  let (log, _) = convert_files_with(
    r"\documentclass{article}
\usepackage[all]{xy}
\begin{document}
\[
\xymatrix{
\mathchoice{X}{XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX}{X}{X} \ar[d] \\ C
}
\]
\end{document}
",
    &[],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
}

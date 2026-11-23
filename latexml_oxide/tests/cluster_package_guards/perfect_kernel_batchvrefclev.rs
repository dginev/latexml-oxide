//! Red/green guards for batch VREFCLEV: after varioref, cleveref.sty:2378-2604 redefines varioref's six reference
//! commands on its raw `\@cref` and varioref's page internals, which neither binding provides (6 Errors and "??" a
//! call); the cleveref binding gives them its page-less `\cref` forms (OXIDIZED_DESIGN #1144, KNOWN_PERL_ERRORS #1318).
//! Witnesses 1602.00476, 2409.11413, 2505.14566. Repros under `tools/perfect_kernel/repros/package-bindings/`,
//! pdflatex-checked (two runs, 0 `!`, no Warning).
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/package-bindings/",
      $path
    ))
  };
}

/// The text of the first `<p class="ltx_p" …>` holding `needle`, tags stripped and white space collapsed.
fn paragraph_text(html: &str, needle: &str) -> String {
  let re = regex::Regex::new(r#"(?s)<p [^>]*class="ltx_p"[^>]*>(.*?)</p>"#).unwrap();
  let tag = regex::Regex::new(r"<[^>]+>").unwrap();
  re.captures_iter(html)
    .map(|c| {
      tag
        .replace_all(&c[1], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
    })
    .find(|text| text.contains(needle))
    .unwrap_or_else(|| panic!("no paragraph holding {needle:?}\n{html}"))
}

/// `\Vref`/`\vref` under hyperref (cleveref.sty:2572-2596, the starred form; red INT081C: 6 Errors, 6 Warnings, "??")
/// and all six commands without hyperref (:2597-2603), a label list included: each the binding's `\Cref`/`\cref`,
/// `\crefrange`/`\Crefrange`, page text dropped as the varioref binding drops it; `\vpageref` stays varioref's `\ref`.
/// The two branches of cleveref's varioref code are the two shapes of one mechanism, so one guard. (The ranges' `show`
/// and the pair conjunction hold a no-break space, cleveref's `~`.)
#[test]
fn cleveref_vref_with_varioref() {
  let (log, xml) = convert_with(repro!("cleveref_vref_with_varioref.tex"), None);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p><ref labelref="LABEL:s:one" show="creftypecap~creflabel"/> and <ref labelref="LABEL:s:one" show="creftype~creflabel"/>.</p>"#,
  );

  let (log, xml) = convert_with(repro!("cleveref_vref_family_with_varioref.tex"), None);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p><ref labelref="LABEL:s:one" show="creftypecap~creflabel"/>; <ref labelref="LABEL:s:one" show="creftype~creflabel"/>; <ref labelref="LABEL:f:a" show="creftypeplural~creflabel"/> and <ref labelref="LABEL:f:b" show="creflabel"/>; <ref labelref="LABEL:s:one" show="creftypeplural creflabel"/> to <ref labelref="LABEL:s:three" show="creflabel"/>; <ref labelref="LABEL:s:one" show="creftypepluralcap creflabel"/> to <ref labelref="LABEL:s:three" show="creflabel"/>; <ref labelref="LABEL:s:two" show="creftype~creflabel"/>; <ref labelref="LABEL:s:two" show="creftypecap~creflabel"/>; <ref labelref="LABEL:s:one"/>.</p>"#,
  );
  let html = super::svg_nested_picture::post_html(&xml);
  assert_eq!(
    paragraph_text(&html, "Section 1;"),
    "Section 1; section 1; figs. 1 and 2; sections 1 to 3; Sections 1 to 3; section 2; Section 2; 1."
  );
}

/// Controls: varioref alone keeps its page-less `\ref` for `\vref`/`\Vref` (pdflatex: "1 and 1", the page text empty on
/// the label's page), cleveref alone its `\Cref`/`\cref`; both unchanged by the batch (byte-identical to INT081C).
#[test]
fn varioref_alone_and_cleveref_alone_unchanged() {
  let tex = |pkgs: &str, refs: &str| {
    format!(
      "\\documentclass{{article}}\n{pkgs}\\begin{{document}}\n\\section{{One}}\\label{{s:one}}\n{refs}\n\\end{{document}}\n"
    )
  };
  let (log, xml) = convert_with(
    &tex(
      "\\usepackage{varioref}\n\\usepackage{hyperref}\n",
      "\\Vref{s:one} and \\vref{s:one}.",
    ),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p><ref labelref="LABEL:s:one"/> and <ref labelref="LABEL:s:one"/>.</p>"#,
  );
  let (log, xml) = convert_with(
    &tex(
      "\\usepackage{hyperref}\n\\usepackage[capitalise]{cleveref}\n",
      "\\Cref{s:one} and \\cref{s:one}.",
    ),
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[],
    r#"<p><ref labelref="LABEL:s:one" show="creftypecap~creflabel"/> and <ref labelref="LABEL:s:one" show="creftype~creflabel"/>.</p>"#,
  );
}

/// varioref's page-less range stubs (`\vpagerefrange`, `\reftextlabelrange`, `\reftextpagerange`) stay bare `\ref`s
/// under cleveref (pdflatex: "on this page" / "1 to 3"; no type names), as before the batch (INT081C byte-identical).
#[test]
fn varioref_range_stubs_stay_refs_under_cleveref() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\n\\usepackage{varioref}\n\\usepackage{cleveref}\n\\begin{document}\n\
     \\section{One}\\label{s:one}\n\\section{Two}\\label{s:two}\n\\section{Three}\\label{s:three}\n\
     A[\\vpagerefrange{s:one}{s:three}] B[\\reftextlabelrange{s:one}{s:three}] C[\\reftextpagerange{s:one}{s:three}]\n\
     \\end{document}\n",
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let range = r#"<ref labelref="LABEL:s:one"/>–<ref labelref="LABEL:s:three"/>"#;
  assert_element(
    &xml,
    "p",
    &[],
    &format!("<p>A[{range}] B[{range}] C[{range}]</p>"),
  );
}

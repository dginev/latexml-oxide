//! Red/green guards for batch 68h: caption's keys and `\DeclareCaptionSubType`, an ORCID given as its address,
//! cleveref's `\crefalias`, graphics' driver name.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// A clean conversion of a repro, squashed.
macro_rules! clean {
  ($path:literal) => {{
    let (log, xml) = convert_with(
      include_str!(concat!("../../../tools/perfect_kernel/repros/", $path)),
      Some("ar5iv.sty"),
    );
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    squash(&xml)
  }};
}

/// caption3.sty:718-858, :1347-1350, :1661-1715: the label format, separator, name, text format and fonts are the
/// caption's, as pdflatex sets them ("Figure 1. A figure", "TABLE 1: A table", "Fig. (2). Parens", "Figure 3").
#[test]
fn caption_keys_set_the_label_and_separator() {
  let xml = clean!("captions-floats/caption_keys_set_the_label_and_separator.tex");
  for caption in [
    r#"<caption><tag close=". "><text font="bold" xml:id="S0.F1.1">Figure 1</text></tag><text font="italic" xml:id="S0.F1.2">A figure</text></caption>"#,
    r#"<caption><tag close=": "><text font="bold" xml:id="S0.T1.1">TABLE 1</text></tag><text font="italic" xml:id="S0.T1.2">A table</text></caption>"#,
    r#"<caption><tag close=". "><text font="bold" xml:id="S0.F2.1">Fig. (2)</text></tag><text font="italic" xml:id="S0.F2.2">Parens</text></caption>"#,
    r#"<caption><tag><text font="bold" xml:id="S0.F3.1">Figure 3</text></tag></caption>"#,
    r#"<caption><tag close=" – "><text font="bold" xml:id="S0.F4.1">Fig. 4</text></tag><text font="italic" xml:id="S0.F4.2">Dash.</text></caption>"#,
    r#"<caption><tag><text font="bold" xml:id="S0.F5.1">Figure 5</text></tag><break/>Newline</caption>"#,
  ] {
    assert!(xml.contains(caption), "{caption}\n{xml}");
  }
}

/// caption3.sty:1774-1806: `\DeclareCaptionSubType*[arabic]{figure}` numbers the sub-figures 1.1, 1.2, and a starred
/// sub-type of algorithm gets its counter (1202.5818, 1604.05412, 2002.03129, 2106.01798, 1706.00437).
#[test]
fn declarecaptionsubtype_numbers_the_subtype() {
  let xml = clean!("captions-floats/declarecaptionsubtype_numbers_the_subtype.tex");
  assert!(
    xml.contains(
      r#"<caption><tag close=" "><text fontsize="90%" xml:id="S0.F1.sf1.3">1.1</text></tag><text fontsize="90%" xml:id="S0.F1.sf1.4">First</text></caption>"#
    ) && xml.contains(
      r#"<caption><tag close=" "><text fontsize="90%" xml:id="S0.F1.sf2.3">1.2</text></tag><text fontsize="90%" xml:id="S0.F1.sf2.4">Second</text></caption>"#
    ),
    "{xml}"
  );
  assert!(xml.contains(r#"<tag role="refnum">1.2</tag>"#), "{xml}");
  assert!(
    xml.contains(
      r#"<p xml:id="p1.1">See <ref labelref="LABEL:a"/>, <ref labelref="LABEL:b"/>, <ref labelref="LABEL:m"/> and 2.a.</p>"#
    ),
    "{xml}"
  );
}

/// An ORCID iD written as its orcid.org address is the iD (2104.12510, 2304.13458, 2407.18715).
#[test]
fn orcid_given_as_url_is_the_id() {
  let xml = clean!("sectioning-frontmatter/orcid_given_as_url_is_the_id.tex");
  for id in [
    "0000-0002-1825-0097",
    "0000-0001-5109-3700",
    "0000-0003-1415-9269",
  ] {
    let link =
      format!(r#"<ref class="ltx_orcid" href="https://orcid.org/{id}" title="ORCID {id}">"#);
    assert!(xml.contains(&link), "{link}\n{xml}");
  }
  assert!(!xml.contains("orcid.org/http"), "{xml}");
}

/// cleveref.sty:1277-1278 `\crefalias{<counter>}{<type>}`: "section 1, supplement A, step 1, item 1" (1810.01223,
/// 1706.03576, 2210.04778).
#[test]
fn cleveref_crefalias_names_the_type() {
  let xml = clean!("package-bindings/cleveref_crefalias_names_the_type.tex");
  let types: Vec<&str> = xml
    .split(r#"<tag role="creftype">"#)
    .skip(1)
    .filter_map(|rest| rest.split("</tag>").next())
    .collect();
  assert_eq!(types, ["section", "step", "item", "supplement"], "{xml}");
}

/// graphics.sty:66-107: `\Gin@driver` is pdftex.def for PDF output, so trimclip reads tc-pdftex.def and no pgf
/// (2202.10115, 2306.09847, 2507.09618).
#[test]
fn adjustbox_clipping_driver_is_pdftex() {
  let xml = clean!("package-bindings/adjustbox_clipping_driver_is_pdftex.tex");
  assert!(xml.contains("no pgf; driver pdftex.def.</p>"), "{xml}");
  assert!(
    xml.contains(
      r#"<p xml:id="p1.2"><text xml:id="p1.2.1"><text width="0.0pt" yoffset="-1.0pt" xml:id="p1.2.1.1">Box</text> </text></p>"#
    ),
    "{xml}"
  );
  assert!(!xml.contains("<picture"), "{xml}");
}

/// A document converted clean (0 errors, 0 warnings), squashed.
fn clean_doc(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  squash(&xml)
}

/// bmvc2k.cls:277-278 defines the colours papers name in caption's options (2210.10486).
#[test]
fn bmvc2k_caption_colour_is_defined() {
  let xml = clean_doc(
    "\\documentclass{bmvc2k}\n\\usepackage[labelfont={color=bmv@captioncolor,bf}]{caption}\n\\begin{document}\n\
     \\begin{figure}\\caption{Colored}\\end{figure}\n\\end{document}\n",
  );
  assert!(
    xml.contains(
      r##"<caption><tag close=": "><text color="#000066" font="bold" xml:id="S0.F1.2">Figure 1</text></tag>Colored</caption>"##
    ),
    "{xml}"
  );
}

/// cas-sc.cls:49, :67: `[review]` loads setspace, whose `\setstretch` a declared caption font calls (2512.10464).
#[test]
fn cas_review_option_loads_setspace() {
  let xml = clean_doc(
    "\\documentclass[review]{cas-sc}\n\\usepackage{caption}\n\\DeclareCaptionFont{mysingle}{\\setstretch{1}}\n\
     \\captionsetup[figure]{font={small,mysingle}}\n\\begin{document}\n\\begin{figure}\\caption{Single}\\end{figure}\n\
     \\end{document}\n",
  );
  assert!(
    xml.contains(
      r#"<caption><tag close=": "><text fontsize="90%" xml:id="S0.F1.2">Figure 1</text></tag><text fontsize="90%" xml:id="S0.F1.3">Single</text></caption>"#
    ),
    "{xml}"
  );
}

/// caption.sty:1167: longtable's caption is formatted by caption too (pdflatex "Table 1. Long").
#[test]
fn longtable_caption_takes_the_caption_separator() {
  let xml = clean_doc(
    "\\documentclass{article}\n\\usepackage{longtable}\n\\usepackage[labelsep=period]{caption}\n\\begin{document}\n\
     \\begin{longtable}{l}\\caption{Long}\\\\ a\\\\ \\end{longtable}\n\\end{document}\n",
  );
  assert!(
    xml.contains(r#"<caption><tag close=". ">Table 1</tag>Long</caption>"#),
    "{xml}"
  );
}

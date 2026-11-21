//! Red/green guards for batch REGFIXO: four v0.8.0 Fatals, each a binding that read its input differently from the
//! package. hyperref's PDF strings (`\protect` left as it is, hyperref.sty:405-413's specials, :555-565's gobbled
//! definitions; 0812.2921 1104.1704 1201.4321 1303.2471 1405.1922 1510.06427, 2001.10451); amsmath's `\start@align`
//! calling no environment back by name (2607.00952, 2610.38589); the extensible arrows robust in a `\protected@edef`
//! (2007.15542); thm-restate's body ending at the `\end` of the environment current at its start (2609.31596). Each
//! guard converts its repro under `tools/perfect_kernel/repros/` and asserts no Error or Warning and the authored
//! content, against pdflatex. Round 2: the PDF string's `\@protected@testopt`, `\@ifnextchar`, `\let` and arrows
//! (hyperref.sty:545-549, 563-564), and the package option's information strings.
use latexml::util::test::{assert_element, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch84b::text,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Every `<tag …>…</tag>` element of `xml` (no nesting of the same tag), in document order.
fn elements<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
  let (open, close) = (format!("<{tag} "), format!("</{tag}>"));
  xml
    .match_indices(&open)
    .map(|(i, _)| &xml[i..i + xml[i..].find(&close).map_or(0, |end| end + close.len())])
    .collect()
}

/// hyperref.sty:405-413: `\$ \{ \} \# \% \& \_` are their characters in a PDF string, and `\protect` stays what it is
/// (:386-570 never sets it), so a robust command is its inner body; the binding's empty `\protect` made the robust
/// `\$` re-expand itself without end (Fatal:Timeout:IfLimit). pdfinfo: Title "On the $q$-exponential {b} #1 50% R&D
/// x_y XY XY". Repro sectioning-frontmatter/hyperref_pdfinfo_specials_and_robust.
#[test]
fn hyperref_pdf_string_keeps_its_specials() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/hyperref_pdfinfo_specials_and_robust.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "rdf",
    &["property=\"dcterms:title\""],
    "<rdf about=\"\" content=\"On the $q$-exponential {b} #1 50% R&amp;D x_y XY XY\" property=\"dcterms:title\"/>",
  );
  assert!(text(&xml).contains("Text kept."), "{xml}");
}

/// hyperref.sty:555-565 `\HyPsd@DefCommand`: a `\def`, `\gdef`, `\edef`, `\xdef` met in a PDF string gobbles its
/// name, parameter text and body. jss.cls's `\Plaintitle{\Plaintitle{…}}` left a `\def\@Plaintitle` that re-expanded
/// `pdftitle={\@Plaintitle}` without end (Fatal:Timeout:PushbackLimit); a `\gdef` in the author kept its body.
/// pdfinfo: Title "", Author "Me". Repro sectioning-frontmatter/hyperref_pdfinfo_gobbles_a_def.
#[test]
fn hyperref_pdf_string_gobbles_a_definition() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/hyperref_pdfinfo_gobbles_a_def.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let rdf: Vec<&str> = xml
    .match_indices("<rdf ")
    .map(|(i, _)| &xml[i..i + xml[i..].find("/>").unwrap_or(0) + 2])
    .collect();
  assert_eq!(rdf, [
    "<rdf about=\"\" content=\"Me\" property=\"dcterms:creator\"/>",
    "<rdf about=\"\" property=\"dcterms:title\"/>",
  ]);
  assert!(text(&xml).contains("Text kept."), "{xml}");
}

/// hyperref.sty:545-549 and :563-564 in a PDF string: an optional-argument `\newcommand` is gobbled with its
/// `\@protected@testopt` arguments (`\opt[q]` is "[q]", `\opt{}` nothing; round 1's typesetting `\protect` took
/// latex.ltx's `\@testopt` branch, "Fq", "Fd"), `\@ifstar` its command and branches (`\st*x` "*x"), `\let` and
/// `\futurelet` their name (`\let\zzundef\relax` no undefined `\zzundef`); an extensible arrow keeps its labels
/// (`$a\xrightarrow{b}c$` "abc", Fatal:Timeout:PushbackLimit); the package option's `pdfauthor` is `\pdfstringdef`'d.
/// Repro sectioning-frontmatter/hyperref_pdfinfo_gobbles_testopt_and_let.
#[test]
fn hyperref_pdf_string_gobbles_testopt_and_let() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/hyperref_pdfinfo_gobbles_testopt_and_let.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let rdf: Vec<&str> = xml
    .match_indices("<rdf ")
    .map(|(i, _)| &xml[i..i + xml[i..].find("/>").unwrap_or(0) + 2])
    .collect();
  assert_eq!(rdf, [
    "<rdf about=\"\" content=\"On the $q$\" property=\"dcterms:creator\"/>",
    "<rdf about=\"\" content=\"An abc arrow [u]v\" property=\"dcterms:subject\"/>",
    "<rdf about=\"\" content=\"k [m]n X Y *x\" property=\"dcterms:subject\"/>",
    "<rdf about=\"\" content=\"X [q] #3\" property=\"dcterms:title\"/>",
  ]);
  assert!(text(&xml).contains("Text kept."), "{xml}");
}

/// amsmath.sty:1724 `\start@align` starts the alignment itself: the document's `{flalign}`, `{alignat}` (and starred)
/// renewed over it (linenoaa.sty:57,85, loaded by aa.cls:2077) are not called back by name, which looped
/// (Fatal:Timeout:Convert). Four displays, the unstarred two numbered (1), (2). Repro
/// alignment/start_align_reentry_by_name.
#[test]
fn start_align_does_not_reenter_by_name() {
  let (log, xml) = convert_with(
    repro!("alignment/start_align_reentry_by_name.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let groups = elements(&xml, "equationgroup");
  assert_eq!(groups.len(), 4, "{xml}");
  let tags: Vec<String> = groups
    .iter()
    .map(|group| {
      group
        .split_once("<tag>")
        .and_then(|(_, rest)| rest.split_once("</tag>"))
        .map_or_else(String::new, |(tag, _)| tag.to_string())
    })
    .collect();
  assert_eq!(tags, ["(1)", "(2)", "", ""], "{xml}");
  for group in &groups {
    assert_eq!(elements(group, "equation").len(), 1, "{group}");
    assert!(
      group.contains("tex=\"\\displaystyle a=b\"") && group.contains("tex=\"\\displaystyle c=d\""),
      "{group}"
    );
  }
  assert!(text(&xml).contains("Text kept."), "{xml}");
}

/// The extensible arrows are robust (amsmath.sty:1027 `\DeclareRobustCommand{\xrightarrow}[2][]`): a
/// `\protected@edef` keeps `\protect\xrightarrow`, where our body, naming the arrow, re-expanded without end
/// (natbib.sty:930 `\bibfield` after revtex4-1.cls:7046 `\let\NAT@merge\thr@@`; Fatal:Timeout:PushbackLimit). The
/// bibitem's formula is the arrow applied to its label; the edef'd `\xleftarrow[u]{v}` keeps both labels. Repro
/// package-bindings/amsmath_xarrow_in_protected_edef.
#[test]
fn xrightarrow_in_a_protected_edef() {
  let (log, xml) = convert_with(
    repro!("package-bindings/amsmath_xarrow_in_protected_edef.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let items = elements(&xml, "bibitem");
  assert_eq!(items.len(), 1, "{xml}");
  let item = items[0];
  assert!(
    text(item).contains("in the") && text(item).contains("transition"),
    "{item}"
  );
  assert!(item.contains("tex=\"2\\xrightarrow{H}2\""), "{item}");
  assert!(
    item.contains("<XMApp role=\"ARROW\"") && item.contains("role=\"OVERACCENT\">H</XMTok>"),
    "{item}"
  );
  let para = &xml[xml.find("Text kept").expect("the text")
    ..xml.find("<bibliography").expect("the bibliography")];
  assert_eq!(
    (
      para.matches("role=\"UNDERACCENT\"").count(),
      para.matches("role=\"OVERACCENT\"").count()
    ),
    (1, 2),
    "{para}"
  );
}

/// thm-restate.sty:34-47 `\thmt@collect@body` ends a `{restatable}`'s body at `\end{\@currenvir}`, the environment
/// current at its start: a document's environment calling `\restatable…\endrestatable` (and the starred one) is ended
/// by its own `\end`, where the literal `restatable` ran away (Fatal:Stomach:Recursion). Repro
/// package-bindings/thm_restate_restatable_in_a_wrapper_environment.
#[test]
fn restatable_in_a_wrapper_environment() {
  let (log, xml) = convert_with(
    repro!("package-bindings/thm_restate_restatable_in_a_wrapper_environment.tex"),
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let theorems: Vec<String> = elements(&xml, "theorem")
    .iter()
    .map(|theorem| {
      text(theorem)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
    })
    .collect();
  assert_eq!(theorems.len(), 4, "{xml}");
  assert!(
    theorems[0].starts_with("Theorem Xa") && theorems[0].ends_with("Body two."),
    "{theorems:?}"
  );
  assert!(theorems[1].ends_with("Body three."), "{theorems:?}");
  assert!(
    theorems[2].starts_with("Theorem Xa") && theorems[2].ends_with("Body two."),
    "{theorems:?}"
  );
  assert!(
    theorems[3].starts_with("Theorem 2") && theorems[3].ends_with("Body three."),
    "{theorems:?}"
  );
  // the `{restatable*}` is numbered as the later `\bar` (pdflatex "Theorem 2"): a reference to its label
  let elements = elements(&xml, "theorem");
  assert!(
    elements[1].contains("<tag role=\"refnum\"><ref labelref=\"LABEL:restatable:bar\"/></tag>"),
    "{}",
    elements[1]
  );
  assert!(
    elements[3].contains("labels=\"LABEL:restatable:bar\"")
      && elements[3].contains("<tag role=\"refnum\">2</tag>"),
    "{}",
    elements[3]
  );
  assert!(
    text(&xml).contains("Text kept.") && text(&xml).contains("Later."),
    "{xml}"
  );
}

//! Red/green guards for batch BACKMATTER (row R BACKMATTERPART, DG ruling 2026-11-23): `\backmatter` belongs to no
//! part, so it closes the open part and chapter where a new `\part` would (sect04.rs `\lx@begin@backmatter`,
//! OXIDIZED_DESIGN_DIVERGENCES #1147; Perl leaves them open, KNOWN_PERL_ERRORS #1320). Each guard converts its repro
//! under `tools/perfect_kernel/repros/sectioning-frontmatter/` and asserts the whole sectional skeleton. The split
//! pages' titles and links: `cluster_xslt_split::split_nav_relations::split_backmatter_pages_belong_to_no_part`.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The document's sectional units, nested: `name#id(children)`, siblings space-separated.
fn skeleton(xml: &str) -> String {
  const UNITS: &[&str] = &[
    "part",
    "chapter",
    "appendix",
    "section",
    "bibliography",
    "index",
  ];
  fn walk(node: roxmltree::Node, out: &mut Vec<String>) {
    for child in node.children().filter(|c| c.is_element()) {
      let name = child.tag_name().name();
      let mut inner = Vec::new();
      walk(child, &mut inner);
      if UNITS.contains(&name) {
        let id = child
          .attribute(("http://www.w3.org/XML/1998/namespace", "id"))
          .unwrap_or("");
        out.push(if inner.is_empty() {
          format!("{name}#{id}")
        } else {
          format!("{name}#{id}({})", inner.join(" "))
        });
      } else {
        out.extend(inner);
      }
    }
  }
  let doc = roxmltree::Document::parse(xml).expect("well-formed XML");
  let mut out = Vec::new();
  walk(doc.root_element(), &mut out);
  out.join(" ")
}

/// book.cls's `\backmatter` after `\part{B}`: the back-matter chapter, the bibliography and the index are the parts'
/// siblings, and the last part holds only its chapter. Red (INT081C, Perl the same): all three inside `Pt2`.
#[test]
fn book_backmatter_closes_the_last_part() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/book_backmatter_closes_the_last_part.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    skeleton(&xml),
    "part#Pt1(chapter#Ch1) part#Pt2(chapter#Ch2) chapter#Chx1 bibliography#bib index#idx"
  );
  assert_element(
    &xml,
    "chapter",
    &["xml:id=\"Chx1\""],
    "<chapter inlist=\"toc\" xml:id=\"Chx1\"><title>Afterword</title><para xml:id=\"Chx1.p1\"><p \
     xml:id=\"Chx1.p1.1\">W.</p></para></chapter>",
  );
}

/// amsbook's own `\backmatter` (amsbook.cls:946-948; its binding's, not book's) after an `\appendix` in a `\part`: the
/// open `ltx:appendix` and the part close, and the back matter's chapter (still appendix B, as pdflatex's "B
/// Afterword") and the bibliography follow the part. Red (INT081C, Perl the same): both inside `Pt1`.
#[test]
fn amsbook_backmatter_closes_the_appendix_and_part() {
  let (log, xml) = convert_with(
    repro!("sectioning-frontmatter/amsbook_backmatter_closes_the_appendix_and_part.tex"),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    skeleton(&xml),
    "part#Pt1(chapter#Ch1 appendix#A1) appendix#A2 bibliography#bib"
  );
}

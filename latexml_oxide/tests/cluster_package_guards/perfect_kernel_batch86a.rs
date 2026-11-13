//! Red/green guards for batch 86a: keywords and subject classifications the frontmatter bindings lost (silent). imsart's
//! `{keyword}[class=…]` blocks of `\kwd` entries are the VTeX classes' shared markup (ims_support_sty.rs
//! `vtex_keyword_markup`), locked against imsart.sty's own, and a `class=AMS|MSC|MSC2010|MSC2020` block is the paper's
//! classification; OmniBus's `\kwd` adds to the entries before it (KNOWN_PERL_ERRORS #1196); an amsart document's own
//! `\subjclass`/`\keywords` storing `\@subjclass`/`\@keywords` reaches the frontmatter at `\maketitle`
//! (OXIDIZED_DESIGN_DIVERGENCES #902). Witnesses 1410.4462, 2407.13970, 2605.27137, 1401.3408 (imsart and a copy),
//! 2004.06211, 1009.1929 (amsart). Each guard converts its repro under
//! `tools/perfect_kernel/repros/sectioning-frontmatter/` and asserts no Error or Warning and every keywords and
//! classification element of the document, in order: its start tag and its text. One guard per distinct path:
//! imsart.sty's two raw keyword paths (the title-footnote option's `\pseudo@keyword`, the default `\keyword@box`), a
//! renamed copy of the class read after the binding, OmniBus's `\kwd`, and the amsart hand-over. Round 2: a
//! classification's scheme is its printed heading, which the HTML shows (`<h6>`, LaTeXML-structure-xhtml.xsl:239-240);
//! the keyword markup is locked only while the class loads, so a document's own `\kwd` is its own.
use latexml::util::test::convert_with;

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch64::post_with,
  perfect_kernel_batch84b::text,
};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $path
    ))
  };
}

/// Each `<keywords>` and `<classification>` element of `xml`, in document order: its start tag and its text (markup
/// aside, whitespace collapsed; empty for a self-closed one). The name is followed by a space, `>` or `/`, so an
/// element without attributes counts too.
fn keyword_elements(xml: &str) -> Vec<(String, String)> {
  let mut found = Vec::new();
  for name in ["keywords", "classification"] {
    let open = format!("<{name}");
    let close = format!("</{name}>");
    for (start, _) in xml.match_indices(&open).filter(|(start, _)| {
      matches!(
        xml.as_bytes().get(start + open.len()),
        Some(b' ' | b'>' | b'/')
      )
    }) {
      let tag_end = start + xml[start..].find('>').expect("start tag closes");
      let tag = &xml[start..=tag_end];
      if tag.ends_with("/>") {
        found.push((start, tag.to_string(), String::new()));
      } else {
        let end = tag_end + xml[tag_end..].find(&close).expect("element closes");
        found.push((start, tag.to_string(), text(&xml[tag_end + 1..end])));
      }
    }
  }
  found.sort_by_key(|(start, ..)| *start);
  found
    .into_iter()
    .map(|(_, tag, body)| (tag, body))
    .collect()
}

/// Converts the repro, checks it converts clean, and compares its keywords and classification elements.
#[track_caller]
fn assert_keyword_elements(tex: &str, expected: &[(&str, &str)]) -> String {
  assert_keyword_elements_warned(tex, 0, expected)
}

/// [`assert_keyword_elements`] for a repro that warns `warnings` times on purpose.
#[track_caller]
fn assert_keyword_elements_warned(tex: &str, warnings: usize, expected: &[(&str, &str)]) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!(
    (error_count(&log), warning_count(&log)),
    (0, warnings),
    "{log}"
  );
  let found = keyword_elements(&xml);
  let found: Vec<(&str, &str)> = found
    .iter()
    .map(|(t, b)| (t.as_str(), b.as_str()))
    .collect();
  assert_eq!(found, expected, "{xml}");
  xml
}

/// imsart under a journal option that sets `keywordsasfootnote` (aap; 1410.4462): imsart.sty collects the blocks
/// through its `\pseudo@keyword`, which its own `\begin` reads first, for a title footnote; refused, every entry is kept
/// and the MSC block is the classification, its scheme the printed heading. Was `<keywords>perfect simulation,
/// </keywords>` alone (OmniBus's `\kwd`).
#[test]
fn imsart_keyword_blocks_footnote_option() {
  assert_keyword_elements(repro!("imsart_keyword_blocks_footnote_option.tex"), &[
    (
      "<keywords name=\"Keywords and phrases:\u{a0}\">",
      "coupling, conditional ergodicity, perfect simulation",
    ),
    (
      "<classification name=\"MSC 2010 subject classifications:\u{a0}\" scheme=\"MSC 2010 subject classifications\">",
      "Primary 60J10; Secondary 60G35",
    ),
  ]);
}

/// imsart by default (2407.13970, 2605.27137): imsart.sty sets the blocks in `\keyword@box`; refused, every entry is
/// kept, the math of one too, and a `class=MSC2020` block is the classification, its scheme the copy's `\setpkgattr`
/// heading (through its `\ims@href`) without its ":", which the HTML shows as the classification's heading (86a r1's
/// `scheme="MSC2020"` showed "MSC2020"). Was `<keywords>L2 rates, </keywords>` and two `<classification
/// scheme="keywords">` of the blocks' leftovers. Variants: arxbj's blocks, the same markup under arxbj's headings (a
/// `class=MSC` block, which arxbj gives no heading, keeps the key: 1106.2467's arxbj.cls); a document's own
/// `\renewcommand{\kwd}` after the class, the document's (86a review probe k1: was "Theif statement and the, while
/// loop.").
#[test]
fn imsart_keyword_blocks_in_the_box() {
  let xml = assert_keyword_elements(repro!("imsart_keyword_blocks_in_the_box.tex"), &[
    (
      "<keywords name=\"Keywords and phrases:\u{a0}\">",
      "Gaussian priors, credible sets, Bernstein–von Mises theorem, L 2 rates",
    ),
    (
      "<classification name=\"MSC2020 subject classifications:\u{a0}\" scheme=\"MSC2020 subject classifications\">",
      "Primary 62F15, 62G15; secondary 62G20",
    ),
  ]);
  assert!(
    xml.contains("theorem, <Math mode=\"inline\" tex=\"L^{2}\" text=\"L ^ 2\""),
    "{xml}"
  );
  let html = post_with(&xml, "resources/XSLT/LaTeXML-html5.xsl");
  assert!(
    html.contains(
      "<h6 class=\"ltx_title ltx_title_classification\">MSC2020 subject classifications</h6>"
    ),
    "{html}"
  );

  assert_keyword_elements(repro!("arxbj_keyword_classification_headings.tex"), &[
    (
      "<keywords name=\"Keywords:\u{a0}\">",
      "coupling; perfect simulation",
    ),
    (
      "<classification name=\"AMS 2000 subject classifications:\u{a0}\" scheme=\"AMS 2000 subject classifications\">",
      "Primary 60F05; secondary 62G20",
    ),
    ("<classification name=\"\u{a0}\" scheme=\"MSC\">", "60J10"),
  ]);

  let xml = assert_keyword_elements(repro!("imsart_document_kwd_is_the_documents.tex"), &[]);
  assert!(
    xml.contains(
      "The <text font=\"typewriter\" xml:id=\"p1.1.1\">if</text> statement and the <text font=\"typewriter\" \
       xml:id=\"p1.1.2\">while</text> loop.</p>"
    ),
    "{xml}"
  );
}

/// A renamed copy of imsart.cls that declares itself imsart (1401.3408's myimsart.cls and the imsart.sty copy it loads,
/// myimsart.sty), read after the imsart.cls binding: the keyword markup stays locked until the class's end, so the copy's
/// blocks are refused as imsart.sty's are. Distinct from the two guards above: the copy is read by the stand-in route
/// (content.rs `read_stand_in_copy`) after the binding's own raw read; a lock released after that read lost both
/// elements. The one Warning is the binding's request for an imsart.sty neither the paper nor TeX Live ships.
#[test]
fn imsart_renamed_copy_keyword_blocks() {
  assert_keyword_elements_warned(repro!("imsart_renamed_copy_keyword_blocks.tex"), 1, &[
    (
      "<keywords name=\"Keywords and phrases:\u{a0}\">",
      "Shewhart test, Change detection",
    ),
    (
      "<classification name=\"AMS 2000 subject classifications:\u{a0}\" scheme=\"AMS 2000 subject classifications\">",
      "Primary 62L10; secondary 62L15, 60G40",
    ),
  ]);
}

/// OmniBus's `\kwd` adds its entry to those before it (the optional separator ignored, as before); Perl's kept the last
/// one (KNOWN_PERL_ERRORS #1196). The `{keyword}` environment's own (empty) classification is Perl's too. Variant: the
/// list is the document's, never reset, and shows each entry once — two blocks, a second title and `\maketitle`, one
/// `<keywords>` in source order (the 86a review's suspected repeat).
#[test]
fn omnibus_kwd_entries_add_up() {
  let xml = assert_keyword_elements(repro!("omnibus_kwd_entries_add_up.tex"), &[
    (
      "<keywords name=\"Keywords:\u{a0}\">",
      "alpha, beta, L 2 spaces,",
    ),
    ("<classification scheme=\"keywords\"/>", ""),
  ]);
  assert!(
    xml.contains("alpha, beta, <Math mode=\"inline\" tex=\"L^{2}\" text=\"L ^ 2\""),
    "{xml}"
  );

  assert_keyword_elements(repro!("omnibus_kwd_blocks_each_entry_once.tex"), &[
    (
      "<keywords name=\"Keywords:\u{a0}\">",
      "alpha, beta, gamma, delta,",
    ),
    ("<classification scheme=\"keywords\"/>", ""),
    ("<classification scheme=\"keywords\"/>", ""),
  ]);
}

/// amsart: a document's own `\subjclass` and `\keywords` that store `\@subjclass` and `\@keywords`, as amsart.cls does
/// (2004.06211:12, 1009.1929), are the classification (its edition's name the scheme) and the keywords; both were lost.
#[test]
fn amsart_document_subjclass_store() {
  let xml = assert_keyword_elements(repro!("amsart_document_subjclass_store.tex"), &[
    (
      "<keywords name=\"Key words and phrases:\u{a0}\">",
      "Harmonic mappings, Schwarz lemma",
    ),
    (
      "<classification name=\"2000 Mathematics Subject Classification:\u{a0}\" scheme=\"2000 Mathematics Subject Classification\">",
      "Primary 31B05; Secondary 42B30",
    ),
  ]);
  assert_eq!(
    super::perfect_kernel_batch84b::creators(&xml),
    vec![vec!["David Kalaj".to_string()]],
    "{xml}"
  );
}

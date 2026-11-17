//! Red/green guards for batch INVDAG: the frontmatter never invents a glyph (DG 2026-11-16). A frontmatter note outside
//! the creators with no mark prints none — a block line at the document's level, inline in a field (`LaTeXML-meta-xhtml.xsl`,
//! OXIDIZED_DESIGN_DIVERGENCES #1063, #990's rule extended); icml's `\icmlsetsymbol` labels show their symbol on the
//! names. A label authblk prints on a name that no `\affil` explains shows on the name as its mark
//! (`Policy::labels_print`, OXIDIZED_DESIGN_DIVERGENCES #1064, #1059's rule). Witnesses 2602.04557, 2405.13379,
//! 2506.06941, 2603.01467, 2606.00446.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

use super::{
  perfect_kernel_batch64::post_with,
  perfect_kernel_batch84b::{creators, text},
  perfect_kernel_batchcr2::{convert_clean, elements},
};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

fn row(items: &[&str]) -> Vec<String> { items.iter().map(|s| (*s).to_string()).collect() }

/// The element opening with `open` in `html` (a page or the XML), whole: up to the close of its own tag name, nested ones of the same
/// name counted.
fn html_element<'a>(html: &'a str, open: &str) -> &'a str {
  let at = html
    .find(open)
    .unwrap_or_else(|| panic!("no {open} in {html}"));
  let name = open[1..].split([' ', '>']).next().expect("a tag name");
  let (start, close) = (format!("<{name}"), format!("</{name}>"));
  let mut depth = 0usize;
  let mut i = at;
  loop {
    let next_open = html[i + 1..].find(&start).map(|k| i + 1 + k);
    let next_close = html[i + 1..]
      .find(&close)
      .map(|k| i + 1 + k)
      .unwrap_or_else(|| panic!("{open} never closed in {html}"));
    match next_open {
      Some(o) if o < next_close => {
        depth += 1;
        i = o;
      },
      _ if depth == 0 => return &html[at..next_close + close.len()],
      _ => {
        depth -= 1;
        i = next_close;
      },
    }
  }
}

/// icml's notice (`\printAffiliationsAndNotice{\icmlEqualContribution}`), a note of the document's own level with no mark
/// (2602.04557; pdflatex prints it as a footnote with no mark, `\let\thefootnote\relax`, icml2026.sty:521): a block line,
/// `div.ltx_note_unmarked` after Perl's "affiliationnotice: " label, its `*` (icml2026.sty:502) kept, no `†`; the equal authors' names show the `*` their
/// `\icmlsetsymbol` label prints (icml2026.sty:446-458), a mark of the creator, not of its name, so Ann's e-mail is
/// still hers where Cy Able shares her surname (1904.07793: a `*` in the name lost Xiaosen Wang's), Cy none. The block
/// path of the unmarked-note template;
/// Interspeech's `\address`/`\email` notes (2405.13379, 2603.01467) take it too. Repro
/// sectioning-frontmatter/fm_unmarked_note_at_document_level_icml_notice.
#[test]
fn an_unmarked_document_level_note_is_a_block_line() {
  let xml = convert_clean(repro!("fm_unmarked_note_at_document_level_icml_notice"));
  assert_eq!(creators(&xml), [
    row(&[
      "Ann Able",
      "mark=*",
      "affiliation=Univ A",
      "email=ann@x.org"
    ]),
    row(&["Bob Baker", "mark=*", "affiliation=Univ A"]),
    row(&["Cy Able", "affiliation=Univ A"]),
  ]);
  assert_eq!(
    html_element(&xml, "<note role=\"affiliationnotice\""),
    r#"<note role="affiliationnotice" xml:id="id3"><sup xml:id="id3.1">*</sup>Equal contribution</note>"#
  );
  let html = post_with(&xml, "resources/XSLT/LaTeXML-html5.xsl");
  assert_eq!(
    html_element(&html, "<div id=\"id3\""),
    r#"<div id="id3" class="ltx_note ltx_role_affiliationnotice ltx_note_unmarked"><span class="ltx_note_type">affiliationnotice: </span><sup id="id3.1" class="ltx_sup">*</sup>Equal contribution</div>"#
  );
  // (the page's marks are the two names' `*`, no `†` anywhere and nothing hidden behind a mark)
  assert_eq!(html.matches("ltx_note_mark").count(), 2, "{html}");
  assert!(!html.contains('\u{2020}'), "{html}");
  assert!(!html.contains("ltx_note_content"), "{html}");
}

/// A `\footnotetext` in the abstract whose `\fnsymbol` mark is empty (the counter 0 after `\maketitle`; 2506.06941):
/// inline in the abstract's paragraph, `span.ltx_note_unmarked`, its content shown, no `†` and no label (a footnote's
/// text, as Perl labels no footnote); the body footnote after it
/// keeps its counter mark 1 and its pop-up (body notes keep Perl's rendering). The inline path of the unmarked-note
/// template. Repro sectioning-frontmatter/fm_unmarked_note_in_abstract_inline.
#[test]
fn an_unmarked_note_in_a_field_is_inline() {
  let xml = convert_clean(repro!("fm_unmarked_note_in_abstract_inline"));
  let html = post_with(&xml, "resources/XSLT/LaTeXML-html5.xsl");
  let abstract_p = html_element(html_element(&html, "<div id=\"abstract1\""), "<p");
  assert_eq!(
    abstract_p,
    r#"<p id="abstract1.1" class="ltx_p">An abstract.
<span id="footnotex1" class="ltx_note ltx_role_footnotetext ltx_note_unmarked">{ann, bob}@x.org</span></p>"#
  );
  // (the whitespace between its tags folded)
  let body_note = html_element(&html, "<span id=\"footnote1\"")
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
    .replace("> <", "><")
    .replace("> ", ">");
  assert_eq!(
    body_note,
    r#"<span id="footnote1" class="ltx_note ltx_role_footnote"><sup class="ltx_note_mark">1</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">1</sup><span class="ltx_tag ltx_tag_note">1</span>A body note.</span></span></span>"#
  );
  assert!(!html.contains('\u{2020}'), "{html}");
}

/// The siblings that make no unmarked note: a `\footnote` in `\date` (a publication note) and a `\thanks` in the
/// abstract after `\maketitle` (article.cls lets `\thanks` be `\relax` there, so its text prints in place, as pdflatex
/// prints "An abstract.Supported by the NSF."): no glyph invented on either. Repro
/// sectioning-frontmatter/fm_note_in_date_and_thanks_in_abstract_no_glyph.
#[test]
fn a_date_note_and_an_abstract_thanks_invent_no_glyph() {
  let xml = convert_clean(repro!("fm_note_in_date_and_thanks_in_abstract_no_glyph"));
  assert_eq!(
    xml.matches("<pubnote>Email: ann@x.org</pubnote>").count(),
    1,
    "{xml}"
  );
  assert_eq!(
    elements(&xml, "abstract")
      .iter()
      .map(|a| text(a).trim().to_string())
      .collect::<Vec<_>>(),
    ["An abstract.Supported by the NSF."]
  );
  let html = post_with(&xml, "resources/XSLT/LaTeXML-html5.xsl");
  assert!(!html.contains("ltx_note_mark"), "{html}");
  assert!(!html.contains('\u{2020}'), "{html}");
}

/// A label authblk prints on a name (`\AB@authnote`, authblk.sty:120-133) that no `\affil` explains — its legend in the
/// `\date`, as 2606.00446's `\author[1,*,§]{Jared Benson}` — shows on the name as a mark-only note, the mark it prints:
/// a literal `§`, a math `$\dagger$`, a document's macro printing `‡` (pdflatex "Ann Able1,§ , Bob Baker1,† , Cat
/// Cole1,‡ , and Dan Dee1"), no Warning; a piece printing two marks (`$^{2,3}$`, which the writer parts at its comma) is
/// placed nowhere and counted, one Warning; one a line's label explains shows none; one whose legend a line opens with,
/// placed apart from the name, shows linked to it (2606.17809). The binding reads both sides as printed
/// (`printed_label_pieces`) and declares each unexplained piece by its spelling, which the writer requests it by
/// (`declare_unexplained_name_labels`, `Policy::labels_print`). Repros
/// sectioning-frontmatter/authblk_label_explained_in_date_shows_on_name,
/// authblk_label_printing_two_marks_unexplained_is_counted, authblk_label_legend_line_marks_show.
#[test]
fn an_authblk_label_no_affil_explains_shows_on_its_name() {
  let xml = convert_clean(repro!("authblk_label_explained_in_date_shows_on_name"));
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "mark=§", "affiliation=Univ A"]),
    row(&["Bob Baker", "mark=†", "affiliation=Univ A"]),
    row(&["Cat Cole", "mark=‡", "affiliation=Univ A"]),
    row(&["Dan Dee", "affiliation=Univ A"]),
  ]);
  assert_eq!(xml.matches(r#"role="footnotemark""#).count(), 3, "{xml}");
  let (log, xml, ()) = convert_with_setup_then(
    repro!("authblk_label_printing_two_marks_unexplained_is_counted"),
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    log.contains(
      "Warning:frontmatter:unanswered_mark_lost authblk prints 2 label(s) of the names no \\affil explains that no \
       name can show: 2, 3"
    ),
    "{log}"
  );
  assert_eq!(creators(&xml), [
    row(&["Ann Able", "affiliation=Univ A"]),
    row(&["Bob Baker", "affiliation=Univ A"]),
  ]);
  // (a label a line explains shows no mark, though the reading places that line elsewhere: `\affil[$\dag$]`'s
  // affiliation the block's, as before — 1604.01848, 1406.4648; the binding's lines and names compared as printed)
  let (log, xml, ()) = convert_with_setup_then(
    r"\documentclass{article}
\usepackage{authblk}
\title{T}
\author[*]{Paul Beame}
\author[$\dag$]{Paraschos Koutris}
\affil[*]{University of Washington}
\affil[$\dag$]{University of Wisconsin-Madison}
\date{}
\begin{document}
\maketitle
Text.
\end{document}",
    Some("ar5iv.sty"),
    || {},
    |_| {},
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!xml.contains(r#"role="footnotemark""#), "{xml}");
  // (a legend an unlabelled line opens with, which the reading places apart from the names, the block's here: each
  // name's mark shown, linked to it — answered per creator, 2606.17809)
  let xml = convert_clean(repro!("authblk_label_legend_line_marks_show"));
  assert_eq!(creators(&xml), [
    row(&[
      "Ann Able",
      "mark=†",
      "affiliation=Univ A",
      "mark=*",
      "email=ann@x.org"
    ]),
    row(&["Bob Baker", "affiliation=Univ B"]),
    row(&["Cat Cole", "mark=†", "affiliation=Univ A"]),
    row(&["mark=†"]),
  ]);
}

//! Red/green guards for batches CR8, CR8b and CR8c: a picture in a frontmatter field is kept as markup, never its text
//! (`events::Lexer::walk_whatsit`'s picture Note, `events::place_pictures`, `emit::item_call`'s picture branch,
//! `emit::past_unprinted_tail`). Inside `\author` one on the names' line or ending a line goes out of the name, a plain
//! frontmatter note (`role="note"`) kept whole in source order (user ruling 2026-11-10); one inside a later line stays in
//! that line's entry; inside a `\thanks` it stays that note's. A pgf/TikZ picture and an `\includegraphics` alike.
//! OXIDIZED_DESIGN_DIVERGENCES #989.
use super::{
  perfect_kernel_batch64::post_with,
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

/// A text-bearing `tikzpicture` after a `\thanks` in `\author` (2404.13552's page-overlay copyright notice; int154t: a
/// second creator "1.00.00.01.0", the node's words lost, 1 Warning): one creator, its thanks note, then a plain note
/// (`role="note"`, no thanks class: CR8b, review revCR8 finding 2) holding the picture whole, the node's text in its `svg`. The picture-in-a-name path (the note written from the picture's
/// tokens). Repro sectioning-frontmatter/fm_picture_text_in_author_is_a_note.
#[test]
fn picture_in_author_is_a_note_kept_whole() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/fm_picture_text_in_author_is_a_note.tex"
  ));
  assert_eq!(elements(&xml, "creator"), CREATORS_A);
}

/// A drawing alone, `\tikz\fill (0,0) circle (2pt);`, beside the name (int154t: a nameless creator, an empty picture, 1
/// Error for the `;` the rewritten call lacked): one creator, then a plain note holding the drawing. Distinct from the guard
/// above: a picture printing no text (its item keyed by its place, `plan::note_key`) and the `;` its last box leaves
/// unread (`past_unprinted_tail`). Repro sectioning-frontmatter/fm_picture_textless_in_author_is_a_note.
#[test]
fn textless_picture_in_author_is_a_note() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/fm_picture_textless_in_author_is_a_note.tex"
  ));
  assert_eq!(elements(&xml, "creator"), CREATORS_B);
}

/// A drawing inside `\thanks{Zebra University \protect\tikz\draw (0,0)--(1,0);}` and an ORCID logo drawn in the link
/// beside the name (CR5; int154t: the note "Zebra University M 0 0 L 39.37 0fill:none", 1 Warning): the logo in its
/// link, the note holding its picture, no path data as text. Distinct: the picture inside a note, whose argument the
/// note's span reaches only grown over its unread `;}`. The repro is not pdflatex-clean (rc=1: TikZ inside `\thanks`, a
/// moving argument, errs even under `\protect`; review revCR8 finding 5): no pdflatex-clean input exercises the `;}` tail
/// inside a note. Repro sectioning-frontmatter/fm_picture_textless_in_thanks_stays_in_note.
#[test]
fn textless_picture_in_thanks_stays_in_the_note() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/fm_picture_textless_in_thanks_stays_in_note.tex"
  ));
  assert_eq!(elements(&xml, "creator"), CREATORS_C);
  assert!(!xml.contains("39.37 0fill"), "{xml}");
}

/// A picture glued to the separator after it: `\tikz\node{Zebraword};\and`, `\end{tikzpicture}, Cara Cole` (review
/// revCR8 finding 1; int154r: the note held the `\and` as the word "and", 1 Warning). Three creators, Ann's and Bob's
/// note holding its picture alone, the environment's `\end{tikzpicture}` read with it. Distinct: the tail
/// (`past_unprinted_tail`) stops at the first command or character it does not owe the picture. Repro
/// sectioning-frontmatter/fm_picture_glued_to_a_separator_ends_the_note.
#[test]
fn picture_glued_to_a_separator_ends_the_note() {
  let xml = convert_clean(repro!("fm_picture_glued_to_a_separator_ends_the_note"));
  assert_eq!(elements(&xml, "creator"), CREATORS_D);
}

/// Two pictures printing the same text beside one name (review revCR8 finding 3; int154r: one kept, "Zebraword"
/// missing, 1 Warning): two notes. Distinct: each picture's item keyed by its place (`plan::note_key`), and the
/// relocation's one copy per label (`relocate`) comparing no picture's note by its text. Repro
/// sectioning-frontmatter/fm_picture_same_text_twice_two_notes.
#[test]
fn two_same_text_pictures_are_two_notes() {
  let xml = convert_clean(repro!("fm_picture_same_text_twice_two_notes"));
  assert_eq!(elements(&xml, "creator"), CREATORS_E);
}

/// Pictures inside an affiliation line of `\author`, at its start and between its words (review revCR8 finding 4;
/// int154r: the affiliation plain, "its markup lost", the pictures notes after it): the affiliation holding both in
/// source order, no note. Distinct: `events::place_pictures` joining a picture to the line's runs. Repro
/// sectioning-frontmatter/fm_picture_inside_affiliation_line_stays.
#[test]
fn picture_inside_an_affiliation_line_stays_in_place() {
  let xml = convert_clean(repro!("fm_picture_inside_affiliation_line_stays"));
  assert_eq!(elements(&xml, "creator"), CREATORS_F);
}

/// `\includegraphics` in `\author`: beside a name, at an affiliation line's start, ending one (review revCR8 P1; int154r:
/// the three images dropped, 0 Warnings; Perl keeps the image in the personname). Ann's a note, Bob's inside his
/// affiliation, Cara's a note after hers. Distinct: an image's Layout event made a picture note, or joined to its line
/// (`events::place_pictures`). The `candidates` path, the TeX tree's, is not compared. Repro
/// sectioning-frontmatter/fm_includegraphics_in_author.
#[test]
fn includegraphics_in_author_is_kept() {
  let xml = convert_clean(repro!("fm_includegraphics_in_author"));
  let creators: Vec<String> = elements(&xml, "creator")
    .iter()
    .map(|c| without_candidates(c))
    .collect();
  assert_eq!(creators, CREATORS_G);
}

/// An image on the first marked affiliation line after a bold names line (review revCR8b finding 1; 2503.08161's
/// `polyu-logo.png`, `$^{1}$ \includegraphics[width=0.36cm]{polyu-logo.png} The Hong Kong …`, dropped silently on CR8b and
/// int154r while the second line's logo was kept): both affiliations hold their image before their words, 0 Warnings.
/// Distinct: the font switch back from the bold line, a spanless Layout event, stood between the image and its run,
/// which `events::place_pictures` now looks past. Repro sectioning-frontmatter/fm_image_first_marked_line_after_bold_names.
#[test]
fn image_on_the_first_marked_line_is_kept() {
  let xml = convert_clean(repro!("fm_image_first_marked_line_after_bold_names"));
  let creators: Vec<String> = elements(&xml, "creator")
    .iter()
    .map(|c| without_candidates(c))
    .collect();
  assert_eq!(
    creators.join("\n"),
    r##"<creator role="author">
    <personname>Ann Able</personname>
    <contact name="Affiliation: " role="affiliation"> <graphics graphic="example-image" options="width=10.243pt,keepaspectratio=true" xml:id="g1a"/> Zebra University</contact>
  </creator>
<creator before="  " role="author">
    <personname>Bob Baker</personname>
    <contact name="Affiliation: " role="affiliation"> <graphics graphic="example-image-a" options="width=10.243pt,keepaspectratio=true" xml:id="g2a"/> Quokka Institute</contact>
  </creator>"##
  );
}

/// A text node's picture then two `;` (review revCR8b finding 5): pdflatex prints the second, the note holds the
/// picture alone and the `;` parts the two names, 0 Warnings. Distinct: the box of a text node reaches past its path's
/// `;`, so `emit::past_unprinted_tail` takes none more (CR8b wrote the printed `;` inside the note). Repro
/// sectioning-frontmatter/fm_picture_text_node_then_two_semicolons.
#[test]
fn a_text_node_picture_takes_one_semicolon() {
  let xml = convert_clean(repro!("fm_picture_text_node_then_two_semicolons"));
  assert_eq!(
    elements(&xml, "creator").join("\n"),
    r##"<creator role="author">
    <personname>Ann Able</personname>
    <note role="note" xml:id="id1"><picture height="18.83" width="73.1" xml:id="pic1a">
        <svg:svg height="18.83" overflow="visible" version="1.1" viewBox="0 0 73.1 18.83" width="73.1">
          <svg:g fill="#000000" stroke="#000000" stroke-width="0.4pt" transform="translate(0,18.83) matrix(1 0 0 -1 0 0) translate(36.55,0) translate(0,9.42) matrix(1.0 0.0 0.0 1.0 -31.94 -4.8)">
            <svg:foreignObject height="9.61" overflow="visible" style="--ltx-fo-width:4.67em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;" transform="matrix(1 0 0 -1 0 9.61)" width="64.65"><text xml:id="pic1a.1">Zebraword</text></svg:foreignObject>
          </svg:g>
        </svg:svg>
      </picture></note>
  </creator>
<creator before="  " role="author">
    <personname>Bob Baker</personname>
  </creator>"##
  );
}

/// The HTML of an unmarked frontmatter note (user ruling 2026-11-10, "No dagger for unmarked notes"; OXIDIZED_DESIGN_DIVERGENCES
/// #990): the picture note of 2404.13552's shape renders inline, with no † (`ltx_note_mark`) and no hover-only
/// `ltx_note_content` (LaTeXML.css hides it), no "note:" label; the `\thanks` beside it keeps its mark (OD #156). Distinct:
/// the meta XSLT's `ltx:creator/ltx:note[not(@mark)]` template. Repro sectioning-frontmatter/fm_picture_text_in_author_is_a_note.
#[test]
fn an_unmarked_frontmatter_note_renders_inline_without_a_dagger() {
  let xml = convert_clean(repro!("fm_picture_text_in_author_is_a_note"));
  let html = post_with(&xml, "resources/XSLT/LaTeXML-html5.xsl");
  let note = html
    .find("<span id=\"id2\" class=\"ltx_note ltx_role_note\"")
    .map(|at| &html[at..])
    .and_then(|rest| rest.find("<svg").map(|end| &rest[..end]))
    .unwrap_or_else(|| panic!("no unmarked note in {html}"));
  assert_eq!(
    note,
    r##"<span id="id2" class="ltx_note ltx_role_note"><span class="ltx_inline-block">"##
  );
  let thanks = html
    .find("<span id=\"id1\" class=\"ltx_note ltx_note_frontmatter")
    .map(|at| &html[at..])
    .and_then(|rest| rest.find("</span></span></span>").map(|end| &rest[..end]))
    .unwrap_or_else(|| panic!("no thanks note in {html}"));
  assert_eq!(
    thanks,
    r##"<span id="id1" class="ltx_note ltx_note_frontmatter ltx_thanks_note ltx_role_thanks"><sup class="ltx_note_mark">†</sup><span class="ltx_note_outer"><span class="ltx_note_content"><sup class="ltx_note_mark">†</sup><span class="ltx_note_type">thanks: </span>Zebra University."##
  );
  // (the page's only marks and hidden content are the thanks': none after the picture either)
  assert_eq!(html.matches("ltx_note_mark").count(), 2, "{html}");
  assert_eq!(html.matches("ltx_note_content").count(), 1, "{html}");
}

/// `xml` without its `candidates="…"` attributes.
fn without_candidates(xml: &str) -> String {
  let mut out = String::new();
  let mut rest = xml;
  while let Some(at) = rest.find(" candidates=\"") {
    out.push_str(&rest[..at]);
    let after = &rest[at + " candidates=\"".len()..];
    rest = after.find('"').map_or("", |end| &after[end + 1..]);
  }
  out.push_str(rest);
  out
}

const CREATORS_A: [&str; 1] = [
  "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">Zebra University.</note>\n    <note role=\"note\" xml:id=\"id2\"><picture height=\"21.52\" width=\"175\" xml:id=\"pic1a\">\n        <svg:svg height=\"21.52\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 175 21.52\" width=\"175\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,21.52) matrix(1 0 0 -1 0 0) translate(87.5,0) translate(0,10.76) matrix(1.0 0.0 0.0 1.0 -82.89 -3.46)\">\n            <svg:foreignObject height=\"12.3\" overflow=\"visible\" style=\"--ltx-fo-width:12.09em;--ltx-fo-height:0.69em;--ltx-fo-depth:0.19em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 9.61)\" width=\"167.31\"><text xml:id=\"pic1a.1\">Zebraword copyright notice</text></svg:foreignObject>\n          </svg:g>\n        </svg:svg>\n      </picture></note>\n  </creator>",
];

const CREATORS_B: [&str; 1] = [
  "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note role=\"note\" xml:id=\"id1\"><picture height=\"5.53\" width=\"5.53\" xml:id=\"pic1a\">\n        <svg:svg height=\"5.53\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 5.53 5.53\" width=\"5.53\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,5.53) matrix(1 0 0 -1 0 0) translate(2.77,0) translate(0,2.77)\">\n            <svg:path d=\"M 0 0 M 2.77 0 C 2.77 1.53 1.53 2.77 0 2.77 C -1.53 2.77 -2.77 1.53 -2.77 0 C -2.77 -1.53 -1.53 -2.77 0 -2.77 C 1.53 -2.77 2.77 -1.53 2.77 0 Z M 0 0\" style=\"stroke:none\"/>\n          </svg:g>\n        </svg:svg>\n      </picture></note>\n  </creator>",
];

const CREATORS_C: [&str; 1] = [
  "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact role=\"orcid\"><ref class=\"ltx_orcid\" href=\"https://orcid.org/0000-0001\" title=\"ORCID 0000-0001\"><picture height=\"5.53\" width=\"5.53\" xml:id=\"pic1a\">\n          <svg:svg height=\"5.53\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 5.53 5.53\" width=\"5.53\">\n            <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,5.53) matrix(1 0 0 -1 0 0) translate(2.77,0) translate(0,2.77)\">\n              <svg:path d=\"M 0 0 M 2.77 0 C 2.77 1.53 1.53 2.77 0 2.77 C -1.53 2.77 -2.77 1.53 -2.77 0 C -2.77 -1.53 -1.53 -2.77 0 -2.77 C 1.53 -2.77 2.77 -1.53 2.77 0 Z M 0 0\" style=\"stroke:none\"/>\n            </svg:g>\n          </svg:svg>\n        </picture></ref></contact>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">Zebra University <picture height=\"1\" width=\"39.92\" xml:id=\"pic2a\">\n        <svg:svg height=\"1\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 39.92 1\" width=\"39.92\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,1) matrix(1 0 0 -1 0 0) translate(0.28,0) translate(0,0.28)\">\n            <svg:path d=\"M 0 0 L 39.37 0\" style=\"fill:none\"/>\n          </svg:g>\n        </svg:svg>\n      </picture></note>\n  </creator>",
];

const CREATORS_D: [&str; 3] = [
  "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note role=\"note\" xml:id=\"id1\"><picture height=\"18.83\" width=\"73.1\" xml:id=\"pic1a\">\n        <svg:svg height=\"18.83\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 73.1 18.83\" width=\"73.1\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,18.83) matrix(1 0 0 -1 0 0) translate(36.55,0) translate(0,9.42) matrix(1.0 0.0 0.0 1.0 -31.94 -4.8)\">\n            <svg:foreignObject height=\"9.61\" overflow=\"visible\" style=\"--ltx-fo-width:4.67em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 9.61)\" width=\"64.65\"><text xml:id=\"pic1a.1\">Zebraword</text></svg:foreignObject>\n          </svg:g>\n        </svg:svg>\n      </picture></note>\n  </creator>",
  "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <note role=\"note\" xml:id=\"id2\"><picture height=\"21.52\" width=\"84.6\" xml:id=\"pic2a\">\n        <svg:svg height=\"21.52\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 84.6 21.52\" width=\"84.6\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,21.52) matrix(1 0 0 -1 0 0) translate(42.3,0) translate(0,10.76) matrix(1.0 0.0 0.0 1.0 -37.69 -3.46)\">\n            <svg:foreignObject height=\"12.3\" overflow=\"visible\" style=\"--ltx-fo-width:5.56em;--ltx-fo-height:0.69em;--ltx-fo-depth:0.19em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 9.61)\" width=\"76.91\"><text xml:id=\"pic2a.1\">Quokkaword</text></svg:foreignObject>\n          </svg:g>\n        </svg:svg>\n      </picture></note>\n  </creator>",
  "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cara Cole</personname>\n  </creator>",
];

const CREATORS_E: [&str; 1] = [
  "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note role=\"note\" xml:id=\"id1\"><picture height=\"18.83\" width=\"73.1\" xml:id=\"pic1a\">\n        <svg:svg height=\"18.83\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 73.1 18.83\" width=\"73.1\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,18.83) matrix(1 0 0 -1 0 0) translate(36.55,0) translate(0,9.42) matrix(1.0 0.0 0.0 1.0 -31.94 -4.8)\">\n            <svg:foreignObject height=\"9.61\" overflow=\"visible\" style=\"--ltx-fo-width:4.67em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 9.61)\" width=\"64.65\"><text xml:id=\"pic1a.1\">Zebraword</text></svg:foreignObject>\n          </svg:g>\n        </svg:svg>\n      </picture></note>\n    <note role=\"note\" xml:id=\"id2\"><picture height=\"18.83\" width=\"73.1\" xml:id=\"pic2a\">\n        <svg:svg height=\"18.83\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 73.1 18.83\" width=\"73.1\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,18.83) matrix(1 0 0 -1 0 0) translate(36.55,0) translate(0,9.42) matrix(1.0 0.0 0.0 1.0 -31.94 -4.8)\">\n            <svg:foreignObject height=\"9.61\" overflow=\"visible\" style=\"--ltx-fo-width:4.67em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 9.61)\" width=\"64.65\"><text xml:id=\"pic2a.1\">Zebraword</text></svg:foreignObject>\n          </svg:g>\n        </svg:svg>\n      </picture></note>\n  </creator>",
];

const CREATORS_F: [&str; 1] = [
  "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><picture height=\"1\" width=\"39.92\" xml:id=\"pic1a\">\n        <svg:svg height=\"1\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 39.92 1\" width=\"39.92\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,1) matrix(1 0 0 -1 0 0) translate(0.28,0) translate(0,0.28)\">\n            <svg:path d=\"M 0 0 L 39.37 0\" style=\"fill:none\"/>\n          </svg:g>\n        </svg:svg>\n      </picture> Zebra <picture height=\"21.52\" width=\"55.35\" xml:id=\"pic2a\">\n        <svg:svg height=\"21.52\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 55.35 21.52\" width=\"55.35\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,21.52) matrix(1 0 0 -1 0 0) translate(27.67,0) translate(0,10.76) matrix(1.0 0.0 0.0 1.0 -23.06 -3.46)\">\n            <svg:foreignObject height=\"12.3\" overflow=\"visible\" style=\"--ltx-fo-width:3.39em;--ltx-fo-height:0.69em;--ltx-fo-depth:0.19em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 9.61)\" width=\"46.89\"><text xml:id=\"pic2a.1\">Quokka</text></svg:foreignObject>\n          </svg:g>\n        </svg:svg>\n      </picture> University</contact>\n  </creator>",
];

const CREATORS_G: [&str; 3] = [
  "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note role=\"note\" xml:id=\"id1\"><graphics graphic=\"example-image\" options=\"width=28.45274pt,keepaspectratio=true\" xml:id=\"g1a\"/></note>\n  </creator>",
  "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><graphics graphic=\"example-image-a\" options=\"width=28.45274pt,keepaspectratio=true\" xml:id=\"g2a\"/> Zebra University</contact>\n  </creator>",
  "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cara Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Quokka Institute</contact>\n    <note role=\"note\" xml:id=\"id2\"><graphics graphic=\"example-image-b\" options=\"width=28.45274pt,keepaspectratio=true\" xml:id=\"g3a\"/></note>\n  </creator>",
];

//! Red/green guards for batch CR8: a pgf/TikZ picture in a frontmatter field is kept as markup, never its text
//! (`events::Lexer::walk_whatsit`'s picture Note, `emit::item_call`'s picture branch, `emit::past_unprinted_tail`). Inside
//! `\author` it goes out of the name, a frontmatter note kept whole in source order (user ruling 2026-11-10); inside a
//! `\thanks` it stays that note's. OXIDIZED_DESIGN_DIVERGENCES #989.
use super::perfect_kernel_batchcr2::{convert_clean, elements};

/// A text-bearing `tikzpicture` after a `\thanks` in `\author` (2404.13552's page-overlay copyright notice; int154t: a
/// second creator "1.00.00.01.0", the node's words lost, 1 Warning): one creator, its thanks note, then a note holding
/// the picture whole, the node's text in its `svg`. The picture-in-a-name path (the note written from the picture's
/// tokens). Repro sectioning-frontmatter/fm_picture_text_in_author_is_a_note.
#[test]
fn picture_in_author_is_a_note_kept_whole() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/fm_picture_text_in_author_is_a_note.tex"
  ));
  assert_eq!(elements(&xml, "creator"), CREATORS_A);
}

/// A drawing alone, `\tikz\fill (0,0) circle (2pt);`, beside the name (int154t: a nameless creator, an empty picture, 1
/// Error for the `;` the rewritten call lacked): one creator, then a note holding the drawing. Distinct from the guard
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
/// note's span reaches only grown over its unread `;}`. Repro sectioning-frontmatter/fm_picture_textless_in_thanks_stays_in_note.
#[test]
fn textless_picture_in_thanks_stays_in_the_note() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/fm_picture_textless_in_thanks_stays_in_note.tex"
  ));
  assert_eq!(elements(&xml, "creator"), CREATORS_C);
  assert!(!xml.contains("39.37 0fill"), "{xml}");
}

const CREATORS_A: [&str; 1] = [
  "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">Zebra University.</note>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id2\"><picture height=\"21.52\" width=\"175\" xml:id=\"pic1a\">\n        <svg:svg height=\"21.52\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 175 21.52\" width=\"175\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,21.52) matrix(1 0 0 -1 0 0) translate(87.5,0) translate(0,10.76) matrix(1.0 0.0 0.0 1.0 -82.89 -3.46)\">\n            <svg:foreignObject height=\"12.3\" overflow=\"visible\" style=\"--ltx-fo-width:12.09em;--ltx-fo-height:0.69em;--ltx-fo-depth:0.19em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 9.61)\" width=\"167.31\"><text xml:id=\"pic1a.1\">Zebraword copyright notice</text></svg:foreignObject>\n          </svg:g>\n        </svg:svg>\n      </picture></note>\n  </creator>",
];

const CREATORS_B: [&str; 1] = [
  "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\"><picture height=\"5.53\" width=\"5.53\" xml:id=\"pic1a\">\n        <svg:svg height=\"5.53\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 5.53 5.53\" width=\"5.53\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,5.53) matrix(1 0 0 -1 0 0) translate(2.77,0) translate(0,2.77)\">\n            <svg:path d=\"M 0 0 M 2.77 0 C 2.77 1.53 1.53 2.77 0 2.77 C -1.53 2.77 -2.77 1.53 -2.77 0 C -2.77 -1.53 -1.53 -2.77 0 -2.77 C 1.53 -2.77 2.77 -1.53 2.77 0 Z M 0 0\" style=\"stroke:none\"/>\n          </svg:g>\n        </svg:svg>\n      </picture></note>\n  </creator>",
];

const CREATORS_C: [&str; 1] = [
  "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact role=\"orcid\"><ref class=\"ltx_orcid\" href=\"https://orcid.org/0000-0001\" title=\"ORCID 0000-0001\"><picture height=\"5.53\" width=\"5.53\" xml:id=\"pic1a\">\n          <svg:svg height=\"5.53\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 5.53 5.53\" width=\"5.53\">\n            <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,5.53) matrix(1 0 0 -1 0 0) translate(2.77,0) translate(0,2.77)\">\n              <svg:path d=\"M 0 0 M 2.77 0 C 2.77 1.53 1.53 2.77 0 2.77 C -1.53 2.77 -2.77 1.53 -2.77 0 C -2.77 -1.53 -1.53 -2.77 0 -2.77 C 1.53 -2.77 2.77 -1.53 2.77 0 Z M 0 0\" style=\"stroke:none\"/>\n            </svg:g>\n          </svg:svg>\n        </picture></ref></contact>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">Zebra University <picture height=\"1\" width=\"39.92\" xml:id=\"pic2a\">\n        <svg:svg height=\"1\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 39.92 1\" width=\"39.92\">\n          <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,1) matrix(1 0 0 -1 0 0) translate(0.28,0) translate(0,0.28)\">\n            <svg:path d=\"M 0 0 L 39.37 0\" style=\"fill:none\"/>\n          </svg:g>\n        </svg:svg>\n      </picture></note>\n  </creator>",
];

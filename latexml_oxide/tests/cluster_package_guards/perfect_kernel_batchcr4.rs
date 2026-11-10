//! Red/green guards for batch CR4: the frontmatter API's e-mail/URL reading prints a formula of glyphs as TeX prints it
//! (every binding), and imsart's address residuals — `\printead` in an address printed in place (at the end under
//! `\@addressatendtrue`), an unlabelled `\address` the run's, a date recovered from `\title`'s `\author` kept over a
//! later `\date`. OXIDIZED_DESIGN_DIVERGENCES #972, #985, #986.
use super::perfect_kernel_batchcr2::{convert_clean, elements};

/// CR4 item 1 (CR3b's RED; review revCR3b's answer to item 4): the API's `\lx@add@url`/`\lx@add@email` read the address
/// Semiverbatim, which made the shifts of `$\sim$` other, so the URL printed "http://www.zebra.edu/$∼$ann"; a formula
/// of glyphs is now its glyphs, one text with the address (1301.1466, math0608476). Distinct from batch107f's brace
/// sets (`math_braces_as_printed`): a math symbol's glyph, through a binding's call (imsart's `\ead`). Repro
/// sectioning-frontmatter/imsart_ead_url_reads_a_tex_argument.
#[test]
fn imsart_ead_url_prints_its_formula() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/imsart_ead_url_reads_a_tex_argument.tex"
  );
  assert_eq!(elements(&convert_clean(tex), "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"URL:\u{a0}\" role=\"url\"><text font=\"typewriter\" xml:id=\"id1\">http://www.zebra.edu/\u{223c}ann</text></contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">ann@zebra.edu</text></contact>\n  </creator>",
  ]);
}

/// CR4 item 3 (review revCR3b finding 3): an unlabelled `\address` inside `{aug}` after `\author{A}\and\author{B}` is
/// both authors' (imsart.sty:1021), not the last author's. Distinct: the in-frontmatter `\address`'s `annotate=run`.
/// Repro sectioning-frontmatter/imsart_unlabelled_address_is_the_runs.
#[test]
fn imsart_unlabelled_address_is_the_runs() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/imsart_unlabelled_address_is_the_runs.tex"
  );
  assert_eq!(elements(&convert_clean(tex), "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann@x.edu</text></contact>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ AB</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">bob@y.org</text></contact>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ AB</contact>\n  </creator>",
  ]);
}

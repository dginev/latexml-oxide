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

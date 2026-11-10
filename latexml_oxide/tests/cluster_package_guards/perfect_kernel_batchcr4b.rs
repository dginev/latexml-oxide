//! Red/green guards for batch CR4b, CR4's fix round: the math shifts of an e-mail pair as TeX pairs them, each imsart
//! address block goes to the authors whose `\ead` labels it cites, and a date a late block defers is placed at the end
//! of the outermost flush. OXIDIZED_DESIGN_DIVERGENCES #972, #985, #986.
use super::perfect_kernel_batchcr2::{convert_clean, elements};

/// CR4b items 1 and 2 (review revCR4 findings 1 and 4): two formulas in one e-mail argument, `$^a$ann@x.edu,
/// $^b$bob@y.edu`, keep both marks and the text between them (CR4 paired the first formula's closing shift with the
/// second's opening one: "$^aann@x.edu,^b$bob@y.edu", 2508.20055's `ej$_$…, s$_$…`); a mark before a glyph formula
/// keeps the mark and prints the glyph (`$^*$bob$\sim$b`); a character command in math prints its character
/// (`p$\_$r`; the brace list `$\{a,b\}$@w.edu` in the unit test `math_shifts_pair_left_to_right`). Distinct from cr4's `imsart_ead_url_prints_its_formula`
/// (one formula, a binding's call): the pairing across formulas and the character commands, through amsart's
/// `\email`. Repro sectioning-frontmatter/email_math_shifts_pair_as_tex_pairs_them.
#[test]
fn email_math_shifts_pair_as_tex_pairs_them() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/email_math_shifts_pair_as_tex_pairs_them.tex"
  );
  assert_eq!(elements(&convert_clean(tex), "creator"), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email address: \" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">$^a$ann@x.edu, $^b$bob@y.edu</text></contact>\n  </creator>",
    "<creator before=\", \" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Email address: \" role=\"email\"><text font=\"typewriter\" xml:id=\"id2\">$^*$bob\u{223c}b@z.edu</text></contact>\n  </creator>",
    "<creator before=\" and \" role=\"author\">\n    <personname>Carl Cole</personname>\n    <contact name=\"Email address: \" role=\"email\"><text font=\"typewriter\" xml:id=\"id3\">p_r@w.edu</text></contact>\n  </creator>",
  ]);
}

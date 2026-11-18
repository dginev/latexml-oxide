//! Guards for MARKSCALE (OXIDIZED_DESIGN_DIVERGENCES #1076): a quantity argument is no ink. graphics.sty's `\scalebox`
//! reads `{Float}[Float]{}` (Perl graphics.sty.ltxml:93), and the text readers (util_text.rs `argument_types`,
//! `shown_arguments`) take a braced or bracketed group of one typed parameter by that type, so a mark boxed by
//! `\scalebox{0.75}{\Letter}` spells "🖂", not "0.75🖂" (2606.29928, 2605.23320).

use latexml::util::test::{error_count, warning_count};

use super::{perfect_kernel_batch84b::creators, perfect_kernel_batchz4::run};

/// A mark macro whose glyph sits in `\scalebox{0.75}{…}` (and, a sibling on the same reader, in
/// `\raisebox{0.2ex}{\rotatebox{0}{\scalebox{0.75}[0.5]{…}}}`: a `Dimension`, a `{Float}`, a `[Float]`) is answered by
/// the `\footnotetext` that spells it, clean; the quantities printed no "0.75" (nor "0.00.750.5") into the mark, which
/// was lost with `Warning:frontmatter:unanswered_mark_lost`.
#[test]
fn scalebox_mark_spells_its_glyph_not_its_scale() {
  let repro = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/mark_scalebox_scale_is_no_ink.tex"
  );
  let sibling = repro.replace(
    r"\scalebox{0.75}{\Letter}",
    r"\raisebox{0.2ex}{\rotatebox{0}{\scalebox{0.75}[0.5]{\Letter}}}",
  );
  for tex in [repro, sibling.as_str()] {
    let (log, xml) = run(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    let mut found = creators(&xml);
    // (the footnotemark is written twice on Bob's name, with one id, as for a bare `\Letter` mark: a row R item)
    found[1].dedup();
    assert_eq!(found, [
      vec!["Ann Able", "affiliation=Foo University"],
      vec![
        "Bob Baker",
        "mark=🖂",
        "affiliation=Foo University",
        "email=bob@foo.edu"
      ],
    ]);
  }
}

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

/// What follows a scale factor in its braces is dropped, one Warning each, not read after the box: Dia's pstricks export
/// writes old pstricks' `\scalebox{1.000000 -1.000000}{label}` (cs0606057), whose "-1" printed beside every label once
/// the factor read as a `Float` (Perl drops it silently, Gullet.pm:57-69; pdflatex errs, "Illegal unit"). Invalid input
/// on purpose: the two Warnings are the count.
#[test]
fn a_scale_factors_tail_is_dropped_with_a_warning() {
  let (log, xml) = run(
    r"\documentclass{article}\usepackage{graphicx}\begin{document}A\scalebox{1 -1}{XY}B \scalebox{0.5}[1 -1]{Q}R\end{document}",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert_eq!(
    log
      .matches("argument of \\scalebox ('-1'); it is dropped")
      .count(),
    2,
    "{log}"
  );
  let para = &xml[xml.find("<para").expect("para")..xml.find("</para>").expect("para end")];
  assert_eq!(
    super::perfect_kernel_batch84b::text(para),
    "A XY B Q R",
    "{para}"
  );
}

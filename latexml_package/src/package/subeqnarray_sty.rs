use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: subeqnarray.sty.ltxml L21-22 — both \subeqnarray and
  // \endsubeqnarray carry `locked => 1`. The lock matters because the
  // immediately-following InputDefinitions pulls the raw subeqnarray.sty,
  // which redefines both names; without the lock the raw-TeX version
  // overwrites our subnumbering trampolines. See arXiv:hep-th/0002165.
  // subeqnarray.sty:33-41 is `\eqnarray` with per-row `\slabel` subnumbers,
  // so the environment is our eqnarray (sect07.rs `\eqnarray`) inside the
  // subnumbering group — Perl's binding (subeqnarray.sty.ltxml:21-22) opened
  // plain display math, where every `&` was a stray alignment tab
  // (subeqnarray-sample: Rust 1, Perl 6; pdflatex clean).
  // Guard: `perfect_kernel_batch56::subeqnarray_aligns_with_subnumbers`.
  DefMacro!("\\subeqnarray",
    "\\lx@equationgroup@subnumbering@begin\\@eqnarray@bindings\\@@eqnarray\
     \\@equationgroup@numbering{numbered=1,preset=1,deferretract=1,grouped=1,aligned=1}\
     \\lx@begin@alignment",
    locked => true);
  DefMacro!("\\endsubeqnarray",
    "\\cr\\lx@end@alignment\\end@eqnarray\\lx@equationgroup@subnumbering@end",
    locked => true);

  InputDefinitions!("subeqnarray", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});

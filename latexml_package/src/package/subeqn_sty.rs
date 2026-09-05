//! subeqn.sty — sub-equation numbering
//! Perl: subeqn.sty.ltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl subeqn.sty.ltxml L21-22 locks both `\subequations` and
  // `\endsubequations`. arXiv papers commonly load subeqn.sty alongside
  // amsmath (which also defines these names) — the lock keeps our
  // \lx@equationgroup@subnumbering@* trampolines from being shadowed.
  DefMacro!("\\subequations",    "\\lx@equationgroup@subnumbering@begin",
    locked => true);
  DefMacro!("\\endsubequations", "\\lx@equationgroup@subnumbering@end",
    locked => true);
  // subeqn.sty:51-57 `subeqnarray` = `\subequations` around `eqnarray`, with
  // an optional leading `\label{main}` taken by `\@lab@subeqnarray`. The
  // eqnarray half is the subeqnarray binding's (subeqnarray_sty.rs) — Perl
  // subeqn.sty.ltxml omits the environment (subeqn-sample; pdflatex clean).
  // Guard: `perfect_kernel_batch56::subeqn_subeqnarray_environment`.
  DefMacro!("\\subeqnarray",
    "\\lx@equationgroup@subnumbering@begin\\@ifnextchar\\label{\\lx@subeqn@lab}{\\lx@subeqn@eqnarray}",
    locked => true);
  DefMacro!("\\lx@subeqn@lab{}{}", "#1{#2}\\lx@subeqn@eqnarray");
  DefMacro!("\\lx@subeqn@eqnarray",
    "\\@eqnarray@bindings\\@@eqnarray\
     \\@equationgroup@numbering{numbered=1,preset=1,deferretract=1,grouped=1,aligned=1}\
     \\lx@begin@alignment");
  DefMacro!("\\endsubeqnarray",
    "\\cr\\lx@end@alignment\\end@eqnarray\\lx@equationgroup@subnumbering@end",
    locked => true);
});

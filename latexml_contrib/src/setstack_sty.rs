use latexml_package::{
  package::{
    amsmath_sty::{define_ams_matrix_base, define_ams_setstack_commands},
    amstext_sty::define_amstext_text,
  },
  prelude::*,
};

LoadDefinitions!({
  // setstack.sty (VIK 1998/12/15, IOP's author bundle, next to iopart.cls and iopams.sty) is neither in TeX Live nor in
  // most of the papers that load it (1008.1543, 1306.6377, 2206.02651; a copy ships with 0803.1550). It "reproduces
  // useful macros from amsmath.sty, thus avoiding the need to load the entire amsmath.sty": amstext's `\text`
  // (setstack.sty:10-19), `\overset`, `\underset`, `\sideset` (:34-46), `{subarray}` and `\substack` (:48-60),
  // `\dddot`/`\ddddot` (:64-67), `\leftroot`/`\uproot` (:75-76; the `\root` that reads them, :79-107, stays the
  // kernel's), `\boxed` (:110) and the optional-argument `\smash` (:113-116) — the amsmath bindings' own definitions
  // here, without amsmath itself (its equation environments, `\pmatrix` forms). Without them `\substack` was undefined
  // and the `\cr` of `\sum_{\substack{a\cr b}}` met the display's alignment (tex.web §1127). Guard
  // `perfect_kernel_batch77q::setstack_defines_the_amsmath_stacking_commands`.
  define_amstext_text()?;
  define_ams_matrix_base()?;
  define_ams_setstack_commands()?;
  // setstack.sty:120-134, its own arrows: a picture-mode `\vector` of `#2` mm, up/down or left/right by the sign `#1`.
  RawTeX!(
    r"\newcommand{\varrow}[2]{\unitlength=1mm\begin{picture}(0,6)\end{picture}\put(0,6){\vector(0,#1 3){#2}}}
\newcommand{\harrow}[2]{\unitlength=1mm\begin{picture}(8,1)(0,-1)\put(0,0){\vector(#1 2,0){#2}}\end{picture}}"
  );
});

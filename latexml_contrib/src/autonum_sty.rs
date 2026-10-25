//! autonum.sty binding (no Perl binding). OXIDIZED_DESIGN_DIVERGENCES #895.
//!
//! autonum numbers only the equations a reference names: its `\ref`/`\eqref`/`\cref` record the labels they name in
//! the aux file (autonum.sty:330-345), and on the next run its display environments, renewed at `\begin{document}`
//! (:127-140, :186-280), drop the numbers of the rest. A single pass has no earlier run to read. Raw-loaded, its
//! renewals also reach only the environments a binding does not lock: `{split}`'s ran inside our locked `{align}`, whose
//! own renewal — the one that restores `\label` at its end (`\autonum@restoreSubcommands`) — never did, so the display's
//! `\label` was left global and every later `\label` failed ("\lx@ams@display@outer@label undefined", the label lost:
//! 1905.00606). The binding loads autonum's packages and options and leaves the displays to amsmath's binding: every
//! equation keeps its number, as without autonum.
use latexml_package::prelude::*;

LoadDefinitions!({
  // autonum.sty:17-25.
  RawTeX!(r"\PassOptionsToPackage{hypertexnames=false}{hyperref}");
  RequirePackage!("etoolbox");
  RequirePackage!("etextools");
  RequirePackage!("amsmath");
  RequirePackage!("textpos", options => vec![String::from("absolute")]);
  RequirePackage!("letltxmacro");
  // autonum.sty:127-133, 176-183: each display environment it renews is kept as `<env>+`, the environment as amsmath
  // left it (`\newenvironment{#1+}[#2]{\csuse{autonum@#1Old}}{\csuse{autonum@end#1Old}}`): 1405.2249's `{align+}`.
  // A constructor (`\begin{equation}`) is aliased as one, a macro pair (`\align`/`\endalign`) as a pair.
  // autonum.sty:302-322: `\autonum@generatePatchedReference{cmd}` makes a reference command mark the labels it names,
  // for a document's own reference commands too (2211.09537's `\autonum@generatePatchedReference{autoref}`). Every
  // equation keeps its number here, so a reference marks nothing and the command stays as it is.
  DefMacro!("\\autonum@generatePatchedReference{}", "");
  DefMacro!("\\autonum@generatePatchedReferenceCSL{}", "");
  DefMacro!("\\autonum@generatePatchedReferenceGeneral{}{}", "");
  for env in [
    "equation", "gather", "multline", "flalign", "align", "alignat", "split",
  ] {
    let (constructor, end) = (s!("\\begin{{{env}}}"), s!("\\end{{{env}}}"));
    if is_defined(&constructor) {
      Let!(T_CS!(s!("\\begin{{{env}+}}")), T_CS!(constructor));
      Let!(T_CS!(s!("\\end{{{env}+}}")), T_CS!(end));
    } else if is_defined(&s!("\\{env}")) {
      Let!(T_CS!(s!("\\{env}+")), T_CS!(s!("\\{env}")));
      Let!(T_CS!(s!("\\end{env}+")), T_CS!(s!("\\end{env}")));
    }
  }
});

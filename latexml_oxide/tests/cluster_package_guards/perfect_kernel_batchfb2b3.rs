//! Guards for FB2b3, a names line's mark ported onto the one engine's writer (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! FB1's port P4), each fixture converted with the emitter's switch off and at `LATEXML_FM_EMIT=2`, the creators (name and
//! every contact, in order) the same both ways: a mark after a brace group of names is the last name's (`block_names`,
//! the old `math_active_scripts`' witness shape).

use super::perfect_kernel_batchnl1::both;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A mark after the close of a brace group of names is the last name's, written after its name as its own run
/// (`{Mohamed Benzaghta$^{\star}$, …, David López-Pérez}$^{\sharp}$`, 2504.02633), its `^` math-active (breqn's) or a
/// superscript; a group the name's run opens itself (`{\large Ann Able}$^{\star}$`, 1208.4825's shape) keeps the mark in
/// the one run. Red (FB2b2 `=2`): "Cat López" and "Cat Cole" written as plain text, the ♯ lost (a Warning).
#[test]
fn a_mark_after_a_group_of_names_is_the_last_names() {
  let (off, two) = both(repro!("author_ieee_blockn_group_marks_math_active"), (0, 0));
  let want = [
    vec!["Ann Able ⋆"],
    vec!["Bob Baker ♭"],
    vec!["Cat López ♯"],
    vec!["Dan Dorn ♭"],
    vec!["Eve Ess † \u{2063} ⋆"],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
  // (pdflatex: "Ann Able⋆ , Bob Baker♭ , Cat Cole♯ , Dan Dorn♭ , and Eve Ess†")
  let (off, two) = both(
    "\\documentclass[journal]{IEEEtran}\n\\begin{document}\n\\title{T}\n\\author{\\IEEEauthorblockN{{Ann \
     Able$^{\\star}$, Bob Baker$^{\\flat}$, Cat Cole}$^{\\sharp}$, Dan Dorn$^{\\flat}$, and Eve \
     Ess$^{\\dagger}$}}\n\\maketitle\nx\n\\end{document}\n",
    (0, 0),
  );
  let want = [
    vec!["Ann Able ⋆"],
    vec!["Bob Baker ♭"],
    vec!["Cat Cole ♯"],
    vec!["Dan Dorn ♭"],
    vec!["Eve Ess †"],
  ];
  assert_eq!(off, want);
  assert_eq!(two, want);
  // (pdflatex: "Ann Able⋆ , Bob Baker♭")
  let (off, two) = both(
    "\\documentclass[journal]{IEEEtran}\n\\begin{document}\n\\title{T}\n\\author{\\IEEEauthorblockN{{\\large Ann \
     Able}$^{\\star}$, Bob Baker$^{\\flat}$}}\n\\maketitle\nx\n\\end{document}\n",
    (0, 0),
  );
  let want = [vec!["Ann Able ⋆"], vec!["Bob Baker ♭"]];
  assert_eq!(off, want);
  assert_eq!(two, want);
}

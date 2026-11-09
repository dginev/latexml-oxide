//! Guards for FB2b3, a names line's mark ported onto the one engine's writer (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`;
//! FB1's port P4), each fixture converted with the emitter's switch off and at `LATEXML_FM_EMIT=2`, the creators (name and
//! every contact, in order) the same both ways: a mark after a brace group of names is the last name's (`block_names`;
//! the old reader's names split, `split_author_line`'s `names_in_wrapper`); FX2: a mark in the formula a name is in,
//! the run taking it whole, written once (review revH5 finding 2).

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
/// the one run, the mark a superscript of the name both ways. Red (FB2b2 `=2`): "Cat López" and "Cat Cole" written as
/// plain text, the ♯ lost (a Warning).
#[test]
fn a_mark_after_a_group_of_names_is_the_last_names() {
  let tex = repro!("author_ieee_blockn_group_marks_math_active");
  let xml = super::perfect_kernel_batchz4::run(tex).1;
  let at = xml
    .find("<personname>Cat López<sup")
    .expect("Cat López's mark a superscript");
  let end = at
    + xml[at..]
      .find("</personname>")
      .expect("a closed personname");
  assert!(xml[at..end].contains('♯'), "{}", &xml[at..end]);

  let two = both(tex, 0);
  let want = [
    vec!["Ann Able ⋆"],
    vec!["Bob Baker ♭"],
    vec!["Cat López ♯"],
    vec!["Dan Dorn ♭"],
    vec!["Eve Ess † \u{2063} ⋆"],
  ];
  assert_eq!(two, want);
  // (pdflatex: "Ann Able⋆ , Bob Baker♭ , Cat Cole♯ , Dan Dorn♭ , and Eve Ess†")
  let two = both(
    "\\documentclass[journal]{IEEEtran}\n\\begin{document}\n\\title{T}\n\\author{\\IEEEauthorblockN{{Ann \
     Able$^{\\star}$, Bob Baker$^{\\flat}$, Cat Cole}$^{\\sharp}$, Dan Dorn$^{\\flat}$, and Eve \
     Ess$^{\\dagger}$}}\n\\maketitle\nx\n\\end{document}\n",
    0,
  );
  let want = [
    vec!["Ann Able ⋆"],
    vec!["Bob Baker ♭"],
    vec!["Cat Cole ♯"],
    vec!["Dan Dorn ♭"],
    vec!["Eve Ess †"],
  ];
  assert_eq!(two, want);
  // (pdflatex: "Ann Able⋆ , Bob Baker♭")
  let two = both(
    "\\documentclass[journal]{IEEEtran}\n\\begin{document}\n\\title{T}\n\\author{\\IEEEauthorblockN{{\\large Ann \
     Able}$^{\\star}$, Bob Baker$^{\\flat}$}}\n\\maketitle\nx\n\\end{document}\n",
    0,
  );
  let want = [vec!["Ann Able ⋆"], vec!["Bob Baker ♭"]];
  assert_eq!(two, want);
  // (a name inside a formula, its mark there: the run takes the formula whole, the `}` of the `{\rm` the name opens in
  // no names group's close — at `=2` the mark and the name written twice, 4 Maths and W=2, review revH5 finding 2;
  // pdflatex: "Ann Able^a and Bob Baker^b")
  let tex = "\\documentclass[aps,prl]{revtex4-2}\n\\begin{document}\n\\title{T}\n\\author{${\\rm Ann~Able}^{a}$ \
             and ${\\rm Bob~Baker}^{b}$}\n\\affiliation{Univ A}\n\\begin{abstract}A.\\end{abstract}\n\\maketitle\n\
             x\n\\end{document}\n";
  let two = both(tex, 0);
  let want = [vec!["\u{2062} Ann Able a", "affiliation=Univ A"], vec![
    "\u{2062} Bob Baker b",
    "affiliation=Univ A",
  ]];
  assert_eq!(two, want);
  let xml = super::perfect_kernel_batchz4::run(tex).1;
  assert_eq!(xml.matches("<Math ").count(), 2, "{xml}");
}

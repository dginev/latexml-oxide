//! lni.cls (GI Lecture Notes in Informatics): the class is loaded raw, and its author block — which it builds by
//! copying authblk's code (lni.cls:360 `\RequirePackage{authblk}`, :384-435 `\renewcommand\author[4][]`, :444-464
//! `\renewcommand\affil[2][]`) into accumulators only its `\maketitle` reads — becomes the frontmatter as authblk's
//! binding makes it: `\author[marks]{name}{e-mail}{ORCID}` an author carrying its affiliation marks, its e-mail and
//! ORCID as contacts (the class prints them in the affiliation footnote, :425-432), `\affil[mark]{…}` authblk's.
//! Under the raw class (the rawclasses lever) the copy's `\affil` replaced the binding's and the kernel lock refused
//! its `\author`, so the affiliations, e-mails and ORCIDs were lost and the last two arguments were left as stray
//! preamble text (2608.21389). A binding, as the class family rule allows: one shape, TeX Live's lni.cls
//! (2024/02/25 v2.3); a paper's own older copy is read the same way.
use latexml_package::{package::authblk_sty::define_author_and_affil, prelude::*};

LoadDefinitions!({
  InputDefinitions!("lni", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  define_author_and_affil()?;
  Let!("\\lx@lni@author", "\\author");
  DefMacro!(
    "\\author[]{}{}{}",
    r"\if\relax\detokenize{#1}\relax\lx@lni@author{#2}\else\lx@lni@author[#1]{#2}\fi\if\relax\detokenize{#3}\relax\else\lx@add@email{#3}\fi\if\relax\detokenize{#4}\relax\else\lx@add@orcid{#4}\fi"
  );
});

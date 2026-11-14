use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: JHEP3.cls.ltxml
  // LoadClass("JHEP", withoptions => 1);
  LoadClass!("JHEP", withoptions => true);
  // JHEP3.cls v3.1.4 (2005/09/26; 60 of the 64 copies of it papers ship, 0709.3309's :1708): `\arXivid{0704.3719 [hep-th]}`, a
  // link to an arXiv identifier as the class writes the other e-print links (`\hepth`, :1687-1704) — 53 papers, in the
  // references (0709.3309, 0801.2642, 0710.5621) and the `\preprint` line (0704.3719, 0801.4910). And :748 `\newbox
  // \@firstaubox`, the speaker's name the running head reads, which a paper may set (hep-ph0112089; every
  // release from v3.0.2). Perl's binding lacks both (KNOWN_PERL_ERRORS #810).
  DefMacro!("\\arXivid{}", "\\href{http://arxiv.org/abs/#1}{\\tt arXiv:#1}");
  RawTeX!(r"\newbox\@firstaubox");
});

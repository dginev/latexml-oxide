use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: lastpage.sty.ltxml
  // Really nothing to do other than try(!) to arrange that lastpage labels
  // the last something?
  at_end_document(TokenizeInternal!(r"\label{lastpage}"))?;
  // lastpagemodern.sty:73 `\gdef\lastpage@lastpage{??}` (the value before the
  // second run) — lastpage-example prints it directly.
  RawTeX!(r"\gdef\lastpage@lastpage{??}\gdef\lastpage@lastpageHy{??}");
  // lastpagemodern.sty:84-89 (lastpageclassic.sty:92, at `\AtBeginDocument`): `\lastpage@putlabel` is `\relax` once
  // the document begins — the label is written at the end by the package itself (:219-226), so a class's own
  // `\AtEndDocument{\clearpage\lastpage@putlabel}` does nothing more (cta-author.cls:729-731 2209.11927, 1710.08633;
  // IET-Conf-Paper.cls:510-512 2609.02892; ECOC-author). Perl's binding lacks it (KNOWN_PERL_ERRORS #847).
  RawTeX!(r"\gdef\lastpage@putlabel{\relax}");
});

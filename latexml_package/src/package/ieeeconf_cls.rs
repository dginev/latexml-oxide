use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ieeeconf.cls.ltxml
  // LoadClass('IEEEtran');
  LoadClass!("IEEEtran");
  // ieeeconf.cls:4096-4118: the switch between "Appendix I" (the default) and "Appendix A" numbering, which
  // documents set (2609.16300), read by its own `\appendices` in place of IEEEtran's `romanappendices` option. Perl
  // only loads IEEEtran (KNOWN_PERL_ERRORS #491). Guard
  // `perfect_kernel_batch61::captions_and_2609_class_commands_keep_their_text`. The heading form `\thesectiondis`
  // follows (:4103-4117; 67x). It reaches the kernel's `\appendix` as IEEEtran's `\appendices` does: the class's
  // `\appendix` (:4065-4073) also numbers the subsections `\Alph`, which `\appendices` does not.
  RawTeX!(r"\newif\ifuseRomanappendices\useRomanappendicestrue");
  DefMacro!(
    "\\appendices",
    "\\@IEEEsaveoriginalsectiononce\\lx@IEEE@appendix\\ifuseRomanappendices\\gdef\\thesection{\\Roman{section}}\\gdef\\thesectiondis{\\Roman{section}}\\else\
     \\gdef\\thesectiondis{\\Alph{section}}\\fi"
  );
  // ieeeconf.cls (IEEEtran V1.6b; 67 papers ship it): its other names — `\ifcenterfigcaptions`, `\ifnocalcleftmargin`,
  // `\@specialpapernotice`, `\@IEEEauthorblockconfadjspace`, `\thesubsubsectiondis` (:1904, :1986-1992, :1532,
  // :4400, :3414) — are IEEEtran's V1.6 names, defined there (ieeetran_cls.rs; 67x). The copy with a first-page foot
  // line (TU Berlin RBO's, :4602-4630, over scrpage2): `\ieeefootline{<text>}`, the venue or copyright line set in the
  // first page's foot, a publication note here as `\IEEEpubid`'s (1803.09650, 1807.07870, 1810.07122, 1906.02017,
  // 1910.02445, 2110.11512, 2502.04167); a fallback a document's own definition replaces. Perl's binding only loads
  // IEEEtran (KNOWN_PERL_ERRORS #712).
  {
    use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
    let _fallback = OriginGuard::new(DefinitionOrigin::Fallback);
    DefMacro!("\\ieeefootline{}", "\\lx@add@pubnote[role=footline]{#1}");
  }
});

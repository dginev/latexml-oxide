use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ieeeconf.cls.ltxml
  // LoadClass('IEEEtran');
  LoadClass!("IEEEtran");
  // The page a float's panels are set in rows against (the panel grid rule, `declare_panel_page`), over IEEEtran's
  // 516pt (ieeetran_cls.rs): ieeeconf.cls:1163-1164 `\textwidth 7.0in`, `\columnsep 0.2in` — 505.89pt in two
  // 245.718pt columns as pdflatex gives it on letter and A4 paper — under a draft option the paper less 2in
  // (:1257-1259; 469.755pt on letter, 452.968pt on A4), one column under `onecolumn` (:749-751, the later declared of
  // the two). The class's `\ProcessOptions` runs the options in their declaration order.
  if is_document_class("ieeeconf") {
    let paper = match last_declared_class_option(&["letterpaper", "a4paper"]) {
      Some("a4paper") => 597.50787,
      _ => 614.295,
    };
    let draft = matches!(
      last_declared_class_option(&["draft", "draftcls", "draftclsnofoot", "final"]),
      Some("draft" | "draftcls" | "draftclsnofoot")
    );
    let text = if draft { paper - 144.54 } else { 505.89 };
    let twocolumn = last_declared_class_option(&["onecolumn", "twocolumn"]) != Some("onecolumn");
    declare_panel_page((text - 14.454) / 2.0, text, twocolumn);
  }
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

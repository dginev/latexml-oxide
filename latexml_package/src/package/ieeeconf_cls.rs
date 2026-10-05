use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: ieeeconf.cls.ltxml
  // LoadClass('IEEEtran');
  load_class("IEEEtran", Vec::new(), Tokens!())?;
  // ieeeconf.cls:4096-4118: the switch between "Appendix I" (the default) and "Appendix A" numbering, which
  // documents set (2609.16300), read by its own `\appendices` in place of IEEEtran's `romanappendices` option. Perl
  // only loads IEEEtran (KNOWN_PERL_ERRORS #491). Guard
  // `perfect_kernel_batch61::captions_and_2609_class_commands_keep_their_text`.
  RawTeX!(r"\newif\ifuseRomanappendices\useRomanappendicestrue");
  DefMacro!(
    "\\appendices",
    "\\appendix\\ifuseRomanappendices\\gdef\\thesection{\\Roman{section}}\\fi"
  );
});

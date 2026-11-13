use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: mathpazo.sty.ltxml
  // mathpazo.sty:35 `slantedGreek`; the binding reads no other option (:33-37).
  DeclareOption!("slantedGreek", { AssignValue!("pazo_slanted_greek" => true); });
  DeclareOption!(None, {});
  ProcessOptions!();
  // mathpazo.sty:109-133: the capital Greek letters from `letters` under `slantedGreek`, italic as a Latin letter
  // unless a math alphabet is in force, else from `upright` (Palatino's), over a class's italic ones
  // (llncs.cls:379-389; KNOWN_PERL_ERRORS #1147); :134-144 `\DeclareMathSymbol{\upGamma}{\mathord}{upright}{0}`…:
  // upright glyphs of their own whatever the option, `tex=` reading the source's `\upGamma` (review rev79e finding 2;
  // no census paper writes them in math: 2310.12571 ships the .sty).
  if lookup_bool("pazo_slanted_greek") {
    declare_capital_greek("", Some(LettersClass::Variable))?;
  } else {
    declare_capital_greek("", None)?;
  }
  declare_capital_greek_named("up", None, None)?;

  DefConstructor!("\\mathbb{}", "#1",
    bounded => true, require_math => true,
    font => {family => "blackboard", series => "medium", shape => "upright"});

  // mathpazo.sty:63/67 declares the `upright` symbol font (its upright Greek), so `\symupright` exists: flexisym's
  // mathpazo.sym:49 reads it (`\cs_gset:cpx{mg@Greek}{\hexnumber@\symupright}`) under `\usepackage[mathpazo]{flexisym}`
  // (breqn). Perl's binding declares none (2605.15826, 2605.30950: `\symupright` undefined).
  RawTeX!(r"\DeclareSymbolFont{upright}{OT1}{zplm}{m}{n}\SetSymbolFont{upright}{bold}{OT1}{zplm}{b}{n}");
  // mathpazo.sty:72 `\DeclareMathAlphabet{\mathbold}{OML}{zplm}{b}{it}`: bold italic math letters (72 papers;
  // 0807.0088, 0905.0925). The `\PazoBB` alphabet (:101-103, the blackboard letters `\mathbb` is made to call,
  // unless `noBBpl`) is the binding's `\mathbb`. Perl's binding defines neither (KNOWN_PERL_ERRORS #761).
  RawTeX!(r"\DeclareMathAlphabet{\mathbold}{OML}{zplm}{b}{it}");
  Let!("\\PazoBB", "\\mathbb");
});

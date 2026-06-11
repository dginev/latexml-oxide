use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // vntex.sty (Vietnamese TeX) sets up Vietnamese typesetting, whose
  // user-facing payload is the T5 font-encoding command set:
  // `\ecircumflex` (ê), `\ocircumflex` (ô), `\abreve` (ă), `\ohorn` (ơ),
  // `\uhorn` (ư), the hook-above accent `\h`, plus capitals. vntex.sty
  // is NOT installed in TeX Live's base tree (verified: no `vntex.sty`
  // anywhere on disk), so BOTH Perl LaTeXML and our raw-load report it
  // missing-file and skip it — leaving `\ecircumflex`/`\h`/etc.
  // undefined whenever a paper bundles Vietnamese author names.
  //
  // Surpass-Perl: faithfully reproduce vntex's actual effect by loading
  // the T5 encoding command set (our t5enc.def binding already mirrors
  // Perl `t5enc.def.ltxml`'s `\ecircumflex`/`\abreve`/`\ohorn`/`\uhorn`
  // + the `\h` hook-above accent). vntex is essentially
  // `\usepackage[T5]{fontenc}` plus Vietnamese helpers, so routing
  // through t5enc is the principled match.
  //
  // Witness arXiv:2003.12709 (`\usepackage{vntex}`, author name
  // "Nguy\~\ecircumflex n Th\d{i} B\'ich Th\h{u}y" → CONVERR_2 on
  // `\ecircumflex`/`\h`).
  t5enc_def::load_definitions()?;
});

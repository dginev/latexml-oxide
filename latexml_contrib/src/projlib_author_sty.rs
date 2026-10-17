//! projlib-author.sty (ProjLib, TeX Live 2025, 2025/10/21): the package is loaded raw, and the author-block commands it
//! provides — `\institute`, `\address`, `\curraddr`, `\email` (projlib-author.sty:121-139 `\ProvideDocumentCommand`,
//! each `\projlib_author_affil_generic:nn`, :98-119) — become the frontmatter: each line the package adds under the
//! author it follows (`\hook_gput_code:nnn { cmd/@author/after }`, run where `\@maketitle` prints `\@author`) is that
//! author's affiliation, address, current address or e-mail. Under the raw package the kernel's locked `\maketitle`
//! never ran the hook, so the lines were lost (2609.02711, colorart.cls: "Laboratoire de Mathématiques Jean Leray …").
//! Provided only as the package provides them: a command defined before the package loaded stays (an AMS class's
//! `\address`; the package also ends at once under one, :38 `\cs_if_exist:NT \subjclass { \endinput }`).
use latexml_package::prelude::*;

/// The commands projlib-author.sty provides and the frontmatter call each line is.
const PROVIDED: &[(&str, &str)] = &[
  ("institute", "\\lx@add@affiliation{#1}"),
  ("address", "\\lx@add@address{#1}"),
  ("curraddr", "\\lx@add@currentaddress{#1}"),
  ("email", "\\lx@add@email{#1}"),
];

LoadDefinitions!({
  let before: Vec<bool> = PROVIDED
    .iter()
    .map(|(name, _)| IsDefined!(&T_CS!(&s!("\\{name}"))))
    .collect();
  InputDefinitions!("projlib-author", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  if !IsDefined!(&T_CS!("\\projlib_author_affil_generic:nn")) {
    return Ok(());
  }
  for ((name, call), defined_before) in PROVIDED.iter().zip(before) {
    if !defined_before {
      RawTeX!(&s!(r"\long\def\{name}#1{{{call}}}"));
    }
  }
});

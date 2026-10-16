//! apa.cls (APA 5th edition manuscripts, TeX Live apa 2008/12/09 v1.3.4): the class is loaded raw, and its numbered
//! author forms (apa.cls:427-459, `\twoauthors` … `\sixaffiliations`), read only by its refused `\maketitle` (man
//! :901, jou `\put@one@authaffil`/`\put@two@authaffil` :1144-1189 at :1190, doc :1432), become the frontmatter
//! as apa6's do ([`crate::apa6_cls::apa_family_class`]): apa6.cls copies them line for line. apa.cls has
//! no `mask` option.
use latexml_package::prelude::*;

LoadDefinitions!({
  crate::apa6_cls::apa_family_class("apa", None)?;
});

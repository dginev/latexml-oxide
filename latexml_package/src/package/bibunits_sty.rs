use crate::prelude::*;

LoadDefinitions!({
  DefRegister!("\\@bibunitauxcnt", Number::new(0));
  DefMacro!("\\bu@unitname", None, "bu\\the\\@bibunitauxcnt");
  DefMacro!("\\bu@bibdata", "");
  DefMacro!("\\bu@bibstyle", "");

  DeclareOption!("globalcitecopy", {
    AssignValue!("CITE_UNIT_GLOBAL" => true);
  });
  DeclareOption!("labelstoglobalaux", { });
  DeclareOption!("sectionbib", {
    AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:section");
  });
  DeclareOption!("subsectionbib", {
    AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:subsection");
  });
  ProcessOptions!();

  Let!("\\std@cite", "\\cite");
  DefMacro!("\\cite", "\\@ifstar{\\lx@bibunits@setglobal\\std@cite}{\\lx@bibunits@resetglobal\\std@cite}");

  DefPrimitive!("\\lx@bibunits@setglobal", None);
  DefPrimitive!("\\lx@bibunits@resetglobal", None);

  DefMacro!("\\defaultbibliography Semiverbatim", "\\gdef\\bu@bibdata{#1}");
  DefMacro!("\\defaultbibliographystyle{}", "\\gdef\\bu@bibstyle{#1}");

  DefMacro!("\\bibliographyunit [DefToken]", "");
  DefMacro!("\\@bibunit", "\\lx@startbibunit\\old@bibunit");
  DefPrimitive!("\\lx@startbibunit", None);

  DefEnvironment!("{bibunit}[]", "#body");

  DefMacro!("\\putbib[]", "\\lx@bibliography[\\bu@unitname]{\\if.#1.\\bu@bibdata\\else#1\\fi}");
});

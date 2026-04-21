use crate::prelude::*;

LoadDefinitions!({
  DefRegister!("\\@bibunitauxcnt", Number::new(0));
  DefMacro!("\\bu@unitname", None, "bu\\the\\@bibunitauxcnt");
  DefMacro!("\\bu@bibdata", "");
  DefMacro!("\\bu@bibstyle", "");

  DeclareOption!("globalcitecopy", {
    AssignValue!("CITE_UNIT_GLOBAL" => true);
  });
  DeclareOption!("labelstoglobalaux", {});
  DeclareOption!("sectionbib", {
    AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:section");
  });
  DeclareOption!("subsectionbib", {
    AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:subsection");
  });
  ProcessOptions!();

  Let!("\\std@cite", "\\cite");
  DefMacro!(
    "\\cite",
    "\\@ifstar{\\lx@bibunits@setglobal\\std@cite}{\\lx@bibunits@resetglobal\\std@cite}"
  );

  // Perl: sets CITE_UNIT to "bibliography buN" if not global
  DefPrimitive!("\\lx@bibunits@setglobal", None,
  after_digest => {
    if !lookup_bool("CITE_UNIT_GLOBAL") {
      let unit = Digest!("\\bu@unitname")?.to_string();
      assign_value("CITE_UNIT", arena::pin(s!("bibliography {unit}")), None);
    }
  });
  // Perl: sets CITE_UNIT to "buN" if not global
  DefPrimitive!("\\lx@bibunits@resetglobal", None,
  after_digest => {
    if !lookup_bool("CITE_UNIT_GLOBAL") {
      let unit = Digest!("\\bu@unitname")?.to_string();
      assign_value("CITE_UNIT", arena::pin(&unit), None);
    }
  });

  DefMacro!(
    "\\defaultbibliography Semiverbatim",
    "\\gdef\\bu@bibdata{#1}"
  );
  DefMacro!("\\defaultbibliographystyle{}", "\\gdef\\bu@bibstyle{#1}");

  // Perl: \bibliographyunit[\section] — intercepts sectional command to start bib units
  DefPrimitive!("\\bibliographyunit [DefToken]", sub[args] {
    let unit_arg = args.into_iter().next().unwrap();
    if unit_arg.is_some() {
      let unit_tok = unit_arg.expected_token();
      let unit_cs = unit_tok.to_string();
      let unit_cs_tok = T_CS!(unit_cs.clone());
      Let!(T_CS!("\\old@bibunit"), unit_cs_tok.clone());
      Let!(unit_cs_tok, T_CS!("\\@bibunit"));
      // Map sectional unit to backmatter element
      let bme = match unit_cs.as_str() {
        "\\part" => Some("ltx:chapter"),
        "\\chapter" => Some("ltx:section"),
        "\\section" => Some("ltx:subsection"),
        "\\subsection" => Some("ltx:subsubsection"),
        "\\subsubsection" => Some("ltx:paragraph"),
        "\\paragraph" => Some("ltx:subparagraph"),
        _ => None,
      };
      if let Some(bme_val) = bme {
        AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => bme_val);
      }
    } else {
      assign_value("CITE_UNIT", Stored::None, None);
    }
  });
  DefMacro!("\\@bibunit", "\\lx@startbibunit\\old@bibunit");

  // Perl: startBibunit() — increment counter and set CITE_UNIT
  DefPrimitive!("\\lx@startbibunit", None,
  after_digest => {
    Digest!("\\global\\advance\\@bibunitauxcnt1")?;
    let unit = Digest!("\\bu@unitname")?.to_string();
    let cite_unit = if lookup_bool("CITE_UNIT_GLOBAL") {
      s!("bibliography {unit}")
    } else {
      unit
    };
    assign_value("CITE_UNIT", arena::pin(&cite_unit), None);
  });

  DefEnvironment!("{bibunit}[]", "#body");

  DefMacro!(
    "\\putbib[]",
    "\\lx@bibliography[\\bu@unitname]{\\if.#1.\\bu@bibdata\\else#1\\fi}"
  );

  // Perl: make \bibliography reset the backmatter element
  Let!("\\bu@orig@bibliography", "\\bibliography");
  // Store original backmatter element for bibliography
  if let Some(orig) = state::lookup_mapping("BACKMATTER_ELEMENT", "ltx:bibliography") {
    assign_value("ORIG_BIBUNIT", orig, None);
  }
  DefMacro!("\\bibliography", "\\lx@reset@bibunit\\bu@orig@bibliography");
  DefPrimitive!("\\lx@reset@bibunit", None,
  after_digest => {
    if let Some(orig) = lookup_value("ORIG_BIBUNIT") {
      state::assign_mapping("BACKMATTER_ELEMENT", "ltx:bibliography", Some(orig));
    }
  });
});

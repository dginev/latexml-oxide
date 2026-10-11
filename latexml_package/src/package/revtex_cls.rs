use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex.cls.ltxml
  // Ignorable options
  DeclareOption!("tighten", None);
  // Read by revtex3_support, as from `\documentstyle[...]{revtex}`: the preprint switch (revtex.sty v3.1:21-22)
  // and equations numbered within sections (revtex.sty v3.1:53, aps.sty:358-365).
  for option in ["manuscript", "preprint"] {
    DeclareOption!(option, {
      assign_value("revtex3@preprint", true, Some(Scope::Global));
    });
  }
  DeclareOption!("eqsecnum", {
    assign_value("revtex3@eqsecnum", true, Some(Scope::Global));
  });
  // Read by revtex3_support: REVTeX 3's `figure` is a float only under
  // `floats` (aps.sty:614-630).
  DeclareOption!("floats", {
    assign_value("revtex3@floats", true, Some(Scope::Global));
  });
  // Sub-styles; the society (revtex.sty v3.1:24-27) is read by revtex3_support.
  for society in ["aps", "osa", "seg"] {
    DeclareOption!(society, {
      assign_value("revtex3@society", society, Some(Scope::Global));
    });
  }
  for substyle in ["aip", "pra", "prb", "prc", "prd", "prl", "rmp"] {
    DeclareOption!(substyle, None);
  }
  // Perl revtex.cls.ltxml L30-35 + L45: `amsfonts`/`amssymb` options
  // add to a load list, `noamsfonts`/`noamssymb` remove. After
  // ProcessOptions, the list is RequirePackage'd. Prior Rust just
  // declared all 4 as no-ops, so `\documentclass[amsfonts]{revtex}`
  // silently dropped the load.
  DeclareOption!("amsfonts", {
    assign_value("revtex_load_amsfonts", true, Some(Scope::Global));
  });
  DeclareOption!("amssymb", {
    assign_value("revtex_load_amssymb", true, Some(Scope::Global));
  });
  DeclareOption!("noamsfonts", {
    assign_value("revtex_load_amsfonts", false, Some(Scope::Global));
  });
  DeclareOption!("noamssymb", {
    assign_value("revtex_load_amssymb", false, Some(Scope::Global));
  });
  // Pass other options to article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("natbib", options => vec![String::from("numbers")]);
  RequirePackage!("revtex3_support");
  // After ProcessOptions, load the tracked package list.
  if lookup_bool("revtex_load_amsfonts") {
    RequirePackage!("amsfonts");
  }
  if lookup_bool("revtex_load_amssymb") {
    RequirePackage!("amssymb");
  }
});

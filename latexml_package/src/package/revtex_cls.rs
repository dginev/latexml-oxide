use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex.cls.ltxml
  // Ignorable options
  for option in ["manuscript", "eqsecnum", "preprint", "tighten"].iter() {
    DeclareOption!(*option, None);
  }
  // Read by revtex3_support: REVTeX 3's `figure` is a float only under
  // `floats` (aps.sty:614-630).
  DeclareOption!("floats", {
    assign_value("revtex3@floats", true, Some(Scope::Global));
  });
  // Sub-styles
  for substyle in ["aps", "osa", "aip", "pra", "prb", "prc", "prd", "prl", "rmp", "seg"].iter() {
    DeclareOption!(*substyle, None);
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

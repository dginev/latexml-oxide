use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: icml.sty.ltxml
  // A versioned ICML style this binding answers (`icml2026`; `./icml2018`, 1706.04964) is a copy the paper ships,
  // which decides the algorithm packages (icml_support_sty.rs `load_icml_style`); `icml` itself loads Perl's pair.
  if lookup_string("fallback_target") == "icml" && !lookup_string("fallback_request").is_empty() {
    icml_support_sty::load_icml_style()?;
  } else {
    RequirePackage!("icml_support");
  }
});

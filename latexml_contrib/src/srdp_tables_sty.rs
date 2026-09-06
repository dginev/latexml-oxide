use latexml_package::prelude::*;

LoadDefinitions!({
  // srdp-tables.sty is a vendored verbatim copy of tabu.sty shipped with
  // the srdp-mathematik bundle. Route directly to the tabu binding.
  // Witness: srdp-mathematik (8 errors + fatal -> 0 errors).
  RequirePackage!("tabu");
});

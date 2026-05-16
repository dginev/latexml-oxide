//! Stub for ieeecolor.cls (IEEE colored journal class).
//!
//! ieeecolor.cls is a derivative of IEEEtran with color additions. Route
//! to the IEEEtran binding which supplies the substantive macros.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
  // The colored journal layout uses xcolor; ieeecolor.cls and its
  // generic.sty companion pre-define many colors before the user's
  // own \usepackage{xcolor} runs. Load with dvipsnames so the
  // extended palette (Cerulean, ForestGreen, PineGreen, ...) is
  // available — ieeecolor papers use these without an explicit
  // [dvipsnames] option themselves. If the user *does* pass
  // [dvipsnames] explicitly, our package-already-loaded check
  // short-circuits the second load without complaint.
  // Witness 2503.11795 (\\color{ForestGreen}).
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string()]);
  // \firstpagerule + \logowidth are lengths the IEEE header layout
  // uses; not visually relevant in XML output. Define as 0pt.
  DefRegister!("\\firstpagerule" => Dimension!("0pt"));
  DefRegister!("\\logowidth" => Dimension!("0pt"));
});

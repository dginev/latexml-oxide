//! Stub for IEEEAerospaceCLS (IEEE Aerospace conference template).
//!
//! IEEEAerospaceCLS is an IEEEtran-derived class for the IEEE Aerospace
//! Conference. Route to IEEEtran which supplies \appendices, \PARstart
//! and similar IEEE-template macros. Witness 2408.05924, 2408.06274.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("IEEEtran");
});

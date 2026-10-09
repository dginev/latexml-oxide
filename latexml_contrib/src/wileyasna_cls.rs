//! WileyASNA-v1.cls (Astronomische Nachrichten): the Wiley NJD binding, and the class's own bibliography package.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("WileyNJD-v1");
  // WileyASNA-v1.cls:2120 `\RequirePackage[natbibapa]{apacite}`: the apacite macros its `.bbl` entries use
  // (`\APACinsertmetastar`, `\APACrefYearMonthDay`, `{APACrefauthors}`, 2407.11648: 30 errors).
  RequirePackage!("apacite", options => vec![s!("natbibapa")]);
});

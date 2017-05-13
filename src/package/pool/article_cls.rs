use rtx_core::state::{State};
use package::*;
pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);
  LoadPool!("LaTeX");
  //**********************************************************************
  // Option handling
  for _option in ["10pt", "11pt", "12pt", "letterpaper", "legalpaper", "executivepaper", "a4paper",
    "a5paper", "b5paper", "landscape", "final", "draft", "oneside", "twoside", "openright", "openany",
    "onecolumn", "twocolumn", "notitlepage", "titlepage"]
    .into_iter().map(|s| s.to_string()) {
    // DeclareOption!(option, None);
  }




  // DeclareOption!("openbib",
  //     || { RequireResource!(None, type: "text/css", content: ".ltx_bibblock{display:block;}"); });
  // DeclareOption!("leqno", || { state.assign_mapping("DOCUMENT_CLASSES", "ltx_leqno": 1); });
  // DeclareOption!("fleqn", || { state.assign_mapping("DOCUMENT_CLASSES", "ltx_fleqn": 1); });

  // ProcessOptions!();

  //**********************************************************************
  // Document structure.
  RelaxNGSchema!("LaTeXML");
  RequireResource!("ltx-article.css");

  Ok(())
}
use latexml_package::prelude::*;

// Rust translation of labelled.latexml
// Sets up a LABEL_MAPPING_HOOK that maps labels to refnums/IDs,
// and redefines section title format macros.
LoadDefinitions!({
  // Set LABEL_MAPPING_HOOK: maps label to (refnum, id)
  // Perl: $label =~ /[\.^](\w+)$/; $last = $1 || $label;
  //       return ($last, $label);
  set_label_mapping_hook(Rc::new(|label: &str, _ctr: &str, _norefnum: bool| {
    if label.is_empty() {
      return (None, None);
    }
    // Extract the last component after '.' or '^'
    let last = label
      .rfind(['.', '^'])
      .map(|pos| &label[pos + 1..])
      .unwrap_or(label);
    (Some(last.to_string()), Some(label.to_string()))
  }));

  // A tiny aesthetic — redefine title format macros
  DefMacro!("\\format@title@section{}", "{\\thesection: #1}");
  DefMacro!("\\format@title@subsection{}", "{\\thesubsection: #1}");
  DefMacro!("\\format@toctitle@section{}", "{\\thesection: #1}");
  DefMacro!("\\format@toctitle@subsection{}", "{\\thesubsection: #1}");
});

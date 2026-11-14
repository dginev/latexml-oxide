use latexml_package::prelude::*;

// Answers an ordinary `\input glyphtounicode` (`\pdfglyphtounicode` is a no-op, so the raw read prints the same); a
// token-list read (`\everyeof` set, expl3's `\file_get:nnN`) reads the file (OXIDIZED_DESIGN_DIVERGENCES #1029).
LoadDefinitions!({
  // TODO: Can we leverage any of these declarations in HTML? Or are we good as-is.
  // Example: \pdfglyphtounicode{notcontains}{220C}
});

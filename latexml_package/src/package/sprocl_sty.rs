//! sprocl.sty — World Scientific Publishing proceedings class shim.
//!
//! No Perl binding exists; the raw .sty isn't on the texmf path either.
//! Provides minimal stubs for the title-page CSes that affected papers
//! call (\address, \abstracts) so they don't trip undefined-CS errors.
//! Reroute: `\address` → \nodisplay (silent), `\abstracts` → article-style
//! abstract environment passthrough.
//!
//! Cluster: ~21 papers in 2026-04-26b sandbox failing on \address +
//! \abstracts undefined.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Core sprocl title-block CSes — accept arg, render minimally
  DefMacro!("\\address{}", "");
  DefMacro!("\\abstracts{}", r"\par\noindent\textbf{Abstract:} #1\par");
  // Other sprocl-specific CSes that appear in sandbox papers as undefined:
  DefMacro!("\\institute{}", "");
  DefMacro!("\\email{}", "");
  // \maketitle is handled by article.cls; sprocl's override would set up
  // a custom layout but article's version is acceptable for our XML output.
});

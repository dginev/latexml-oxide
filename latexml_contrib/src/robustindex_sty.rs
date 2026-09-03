//! robustindex.sty — makeindex round-trip helpers: robust page references
//! written as `\indpageref{N}` labels, encap wrapping (`\wrappageref\textbf`)
//! and suppression (`\gobblepageref`), `\sindex`/`\setindex` for several
//! indexes. Loaded RAW; only the two page-reference hooks are re-bound.
//!
//! `\gobblepageref` = `\protect\gobbleindpageref` = `\wrappageref\@gobble`,
//! and `\wrapindpageref` is the delimited `\def\wrapindpageref#1, \indpageref#2`
//! (robustindex.sty:201-216): it consumes the `, \indpageref{N}` that
//! makeindex writes into every `.ind` line. LaTeXML builds the index from the
//! marks themselves — there is no `.ind` line and no page number — so
//! `\index{alpha!see also gamma\gobblepageref}` (robustsample.tex:82) had the
//! delimited scan run off the entry (`readBalanced ran out of input`;
//! robustsample, multisample, robustmanual; Perl shares the failure as a
//! missing-argument error). With no locator to wrap or drop, both hooks are
//! inert here: `\wrappageref` still swallows its wrapper token.
//! Guard: `perfect_kernel_batch54::robustindex_page_reference_hooks_are_inert`.

use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("robustindex", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefMacro!("\\gobblepageref", "");
  DefMacro!("\\wrappageref{}", "");
});

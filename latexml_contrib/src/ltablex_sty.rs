use latexml_package::prelude::*;

// ltablex.sty (v1.1): `tabularx` becomes a MULTI-PAGE table — the body runs
// through longtable (`\LT@` boxes, ltablex.sty:158-238 `\TX@endtabularx`), so
// `\endhead`/`\endfirsthead`/`\endfoot`/`\endlastfoot`/`\caption` are legal
// inside it — with tabularx's `X` columns (global `DefColumnType!("X")`,
// tabularx_sty.rs). `\keepXColumns`/`\convertXColumns` (:146-153) only toggle
// `\ifTX@convertX@`, the print-width decision of whether X columns are
// converted to `l` when the natural width fits; LaTeXML renders X columns the
// same either way. The former warn-only stub defined neither toggle
// (milsymb.tex `\keepXColumns` undefined, 44 errors; Perl raw-loads the file).
// Guard: `perfect_kernel_batch54::ltablex_tabularx_is_a_longtable_with_toggles`.
LoadDefinitions!({
  RequirePackage!("longtable");
  RequirePackage!("tabularx");
  RawTeX!(r"\newif\ifTX@convertX@ \TX@convertX@true");
  DefMacro!("\\keepXColumns", "\\TX@convertX@false");
  DefMacro!("\\convertXColumns", "\\TX@convertX@true");
  // `\begin{tabularx}{width}[pos]{cols}` → the longtable driver (the width is
  // the page width under ltablex, as for any longtable).
  DefMacro!(
    "\\tabularx{}[]{}",
    r"\lx@longtable@bindings{#3}\@@longtable[#2]{#3}\lx@begin@alignment"
  );
  DefMacro!("\\endtabularx", r"\lx@end@alignment\@end@tabular");
});

//! standalone.sty — compile standalone sub-documents
//! Perl: standalone.sty.ltxml (40 lines).
//! NOTE: standalone.cls is handled separately; this is the .sty package.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\@standalone@end@input", "\\egroup\\endinput");

  // Perl L21-23: DefPrimitiveI \@standalone@start@input — sets inPreamble = 0.
  DefPrimitive!("\\@standalone@start@input", {
    assign_value("inPreamble", false, None);
  });

  // Perl L24-33: DefPrimitive \@standalone@documentclass[]{} — open a
  // group, mark inPreamble = 1, RequirePackage each comma-separated entry
  // of the argument, and alias \begin{document}/\end{document} to the
  // start/end input primitives so the sub-document is injected as a
  // bounded scope inside the outer document.
  DefPrimitive!("\\@standalone@documentclass[]{}", sub[(_opts, packages_tks)] {
    bgroup();
    assign_value("inPreamble", true, None);
    let packages_str = packages_tks.to_string();
    for pkg in packages_str.split(',') {
      let pkg = pkg.trim();
      if !pkg.is_empty() {
        RequirePackage!(pkg);
      }
    }
    Let!(T_CS!("\\begin{document}"), T_CS!("\\@standalone@start@input"));
    Let!(T_CS!("\\end{document}"),   T_CS!("\\@standalone@end@input"));
  });

  // Perl L35-36: AtBeginDocument — swap \documentclass to the intercept.
  // Native push to @at@begin@document so the hook fires at the same
  // lifecycle point Perl uses.
  at_begin_document(TokenizeInternal!(r"\let\documentclass\@standalone@documentclass"))?;

  // standalone.sty L1014: \includestandalone[opts]{file}. Treat as
  // \includegraphics{file} so the figure surfaces in the XML output.
  // Witness 2406.02722.
  DefMacro!("\\includestandalone[]{}", "\\includegraphics{#2}");
});

use latexml_package::prelude::*;

//**********************************************************************
// LaTeXML Declaration for David Carlisle's xii.tex
// Maps custom song structure to standard LaTeXML elements:
//   song  -> ltx:section (auto-open as top container)
//   verse -> ltx:para    (created by \bigskip, auto-close)
//   line  -> ltx:p       (auto-open + auto-close for text)
//**********************************************************************
LoadDefinitions!({
  // Don't need to respect source newlines
  AssignValue!("PRESERVE_NEWLINES", 0);

  // Auto-open section to contain the song content (no \begin{document} in xii.tex)
  Tag!("ltx:section", auto_open => true);

  // Make \bigskip initiate a <ltx:para> (verse), closeable when needed.
  Tag!("ltx:para", auto_close => true);
  DefConstructor!("\\bigskip", "<ltx:para>");

  // David ends each line with \par; redefine \par to close an auto-opened <ltx:p>
  DefConstructor!("\\par", sub[doc,_args,_props] { doc.maybe_close_element("ltx:p")?; });

  Tag!("ltx:p", auto_close => true, auto_open => true);
});

use latexml_package::prelude::*;

//**********************************************************************
// LaTeXML Declaration for David Carlisle's xii.tex
//**********************************************************************
LoadDefinitions!({
  // Don't need to respect source newlines
  AssignValue!("PRESERVE_NEWLINES", 0);

  // We'll use a DTD for a (trivial) Song, containing verses with lines.
  DocType!("song", "-//NIST LaTeXML//LaTeXML Poem", "xii.dtd");

  // There's no explicit \begin{document}, so let the poem automatically open.
  Tag!("song", auto_open => true);

  // Make \bigskip initiate a <verse>, closeable when needed.
  Tag!("verse", auto_close => true);
  DefConstructor!("\\bigskip", "<verse>");

  // David ends each line with \par; redefine \par to close an auto-opened <line>
  DefConstructor!("\\par", sub[doc,_args,_props] { doc.maybe_close_element("line")?; });

  Tag!("line", auto_close => true, auto_open => true);
  // Ensure no namespaces
  model::register_document_namespace("", None);
  model::register_document_namespace("ltx", None);
});

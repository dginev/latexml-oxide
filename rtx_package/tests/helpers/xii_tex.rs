use rtx_package::package::*;

//**********************************************************************
// LaTeXML Declaration for David Carlisle's xii.tex
//**********************************************************************
LoadDefinitions!(state, {
  // Don't need to respect source newlines
  AssignValue!("PRESERVE_NEWLINES", false);

  // We'll use a DTD for a (trivial) Song, containing verses with lines.
  DocType!("song", "-//NIST LaTeXML//LaTeXML Poem", "xii.dtd");

  // There's no explicit \begin{document}, so let the poem automatically open.
  Tag!("song", auto_open => true);

  // Make \bigskip initiate a <verse>, closeable when needed.
  Tag!("verse", auto_close => true);
  DefConstructor!("\\bigskip", "<verse>");

  // David ends each line with \par; redefine \par to close an auto-opened <line>
  DefConstructor!("\\par", sub[doc,_args,_props,state] { doc.maybe_close_element("line", state)?; });

  Tag!("line", auto_close => true, auto_open => true);
  // Ensure no namespaces
  state.model.register_document_namespace("", None);
  state.model.register_document_namespace("ltx", None);
});

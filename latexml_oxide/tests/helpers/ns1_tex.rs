use latexml_package::prelude::*;

// Namespace test 1: No namespaces at all.
LoadDefinitions!({
  DocType!("song", "-//NIST LaTeXML//LaTeXML Poem", "ns1.dtd");

  Tag!("song", auto_open => true, auto_close => true);
  Tag!("verse", auto_open => true, auto_close => true);
  Tag!("line", auto_open => true, auto_close => true);
  DefConstructor!("\\bigskip", "<verse>");
  DefConstructor!("\\par", sub[doc,_args,_props] { doc.maybe_close_element("line")?; });

  // Remove default LaTeXML namespaces
  model::register_document_namespace("", None);
  model::register_document_namespace("ltx", None);
});

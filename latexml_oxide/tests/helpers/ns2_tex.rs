use latexml_package::prelude::*;

// Namespace test 2: Output in namespace http://example.com/ without prefixes.
// Code uses prefix "incode" for the namespace.
LoadDefinitions!({
  model::register_namespace("incode", Some("http://example.com/"));

  DocType!("incode:song", "-//NIST LaTeXML//LaTeXML Poem", "ns2.dtd");

  Tag!("incode:song", auto_open => true, auto_close => true);
  Tag!("incode:verse", auto_open => true, auto_close => true);
  Tag!("incode:line", auto_open => true, auto_close => true);
  DefConstructor!("\\bigskip", "<incode:verse>");
  DefConstructor!("\\par", sub[doc,_args,_props] { doc.maybe_close_element("incode:line")?; });

  // Remove default LaTeXML namespace
  model::register_document_namespace("", None);
  model::register_document_namespace("ltx", None);
});

use latexml_package::prelude::*;

// Namespace test 5: Two namespaces, but DTD only uses prefix "inner".
// Code uses "incode" for http://inner.com/ and "outcode" for http://outer.com/.
LoadDefinitions!({
  model::register_namespace("incode", Some("http://inner.com/"));
  model::register_namespace("outcode", Some("http://outer.com/"));

  DocType!("outcode:song", "-//NIST LaTeXML//LaTeXML Poem", "ns5.dtd");

  Tag!("outcode:song", auto_open => true, auto_close => true);
  Tag!("outcode:verse", auto_open => true, auto_close => true);
  Tag!("incode:line", auto_open => true, auto_close => true);
  DefConstructor!("\\bigskip", "<outcode:verse>");
  DefConstructor!("\\par", sub[doc,_args,_props] { doc.maybe_close_element("incode:line")?; });

  // Remove default LaTeXML namespace
  model::register_document_namespace("", None);
  model::register_document_namespace("ltx", None);
});

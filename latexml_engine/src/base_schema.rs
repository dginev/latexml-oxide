//! Base Schema                                                         | #
//!
//! Core TeX Implementation for LaTeXML                                 | #
use crate::prelude::*;

LoadDefinitions!({
  // NOTE that these define the namespaces we'll (probably) use
  // along with the prefixes to be used in "code"
  // The generated XML will use the prefixes defined by RegisterDocumentNamespace(...) (if ever)
  // or those prefixes defined by the Schema (typically RelaxNGSchema(..)
  RegisterNamespace!("ltx", "http://dlmf.nist.gov/LaTeXML");
  RegisterNamespace!("svg", "http://www.w3.org/2000/svg");
  // Needed for SVG
  RegisterNamespace!("xlink", "http://www.w3.org/1999/xlink");
  // Not directly used, but let's stake out the ground
  RegisterNamespace!("m", "http://www.w3.org/1998/Math/MathML");
  RegisterNamespace!("xhtml", "http://www.w3.org/1999/xhtml");
  // Namespace for arbitrary data attributes (mapped to data-xxx in html5)
  RegisterNamespace!("data" => "http://dlmf.nist.gov/LaTeXML/data");
  // Needed for ARIA accessibility attributes
  RegisterNamespace!("aria" => "http://www.w3.org/ns/wai-aria");

  // This is used for plain TeX, but needs to be undone for LaTeX (or...)!
  RelaxNGSchema!("LaTeXML");

  Tag!("ltx:section", auto_close => true);
  Tag!("ltx:document", auto_close => true, auto_open => true);
  Tag!("ltx:document", after_open => sub[document,root] {
    let mut bg_to_set = None;
    if let Some(bg) = document.get_node_font(root).get_background() {
      if *bg != latexml_core::common::color::WHITE {
        bg_to_set = Some(bg.to_attribute());
      }
    }
    if let Some(bg) = bg_to_set {
      document.set_attribute(root, "backgroundcolor", &bg)?;
    }
    // Apply font language as xml:lang on document root.
    // Also update the document element's font language to match, so the
    // font delta serializer doesn't override xml:lang with a stale value
    // (e.g., from class options processed before babel determines the main language).
    if let Some(lang) = lookup_value("DOCUMENT_LANGUAGE") {
      let lang_str = lang.to_string();
      if !lang_str.is_empty() {
        document.set_attribute(root, "xml:lang", &lang_str)?;
        let mut font = document.get_node_font(root).clone();
        font.language = Some(Cow::Owned(lang_str));
        document.set_node_font(root, &font)?;
      }
    }
  });

  //======================================================================
  // Core ID functionality.
  //======================================================================
  DefMacro!("\\lx@empty", None);

  // DOCUMENTID is the ID of the document
  // AND prefixes IDs on all other elements.
  let doc_id = state::lookup_string("DOCUMENTID");
  if !doc_id.is_empty() {
    // Wrap in T_OTHER so funny chars don't screw up (no space!)
    let doc_id_token = T_OTHER!(doc_id);
    DefMacro!(T_CS!("\\thedocument@ID"), None, doc_id_token);
  } else {
    Let!("\\thedocument@ID", "\\lx@empty");
  }

  NewCounter!("@lx@xmarg", "document", idprefix => "XM");

  //======================================================================
  Tag!("ltx:document",
  after_open => sub[document, _node] {
    document.process_pending_resources()?;
  });
  RequireResource!("LaTeXML.css");
  //======================================================================
  // The default "initial context" for XML+RDFa specifies some default
  // terms and prefixes, but no default vocabulary.
  // Ought to have a default for @vocab, but settable?
  // can we detect use of simple "term"s in attributes so we know whether we need @vocab?
  // Ought to have a default set of prefixes from RDFa Core,
  // but allow prefixes to be added.
  // Probably ought to scan rdf attributes for all uses of prefixes,
  // and include them in @prefix
  // The following prefixes are listed in http://www.w3.org/2011/rdfa-context/rdfa-1.1
  let rdf_prefixes = map!(
    "cc"      => "http://creativecommons.org/ns#",
    "ctag"    => "http://commontag.org/ns#",
    "dc"      => "http://purl.org/dc/terms/",
    "dcterms" => "http://purl.org/dc/terms/",
    "ical"    => "http://www.w3.org/2002/12/cal/icaltzd#",
    "foaf"    => "http://xmlns.com/foaf/0.1/",
    "gr"      => "http://purl.org/goodrelations/v1#",
    "grddl"   => "http://www.w3.org/2003/g/data-view#",
    "ma"      => "http://www.w3.org/ns/ma-ont#",
    "og"      => "http://ogp.me/ns#",
    "owl"     => "http://www.w3.org/2002/07/owl#",
    "rdf"     => "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    "rdfa"    => "http://www.w3.org/ns/rdfa#",
    "rdfs"    => "http://www.w3.org/2000/01/rdf-schema#",
    "rev"     => "http://purl.org/stuff/rev#",
    "rif"     => "http://www.w3.org/2007/rif#",
    "rr"      => "http://www.w3.org/ns/r2rml#",
    "schema"  => "http://schema.org/",
    "sioc"    => "http://rdfs.org/sioc/ns#",
    "skos"    => "http://www.w3.org/2004/02/skos/core#",
    "skosxl"  => "http://www.w3.org/2008/05/skos-xl#",
    "v"       => "http://rdf.data-vocabulary.org/#",
    "vcard"   => "http://www.w3.org/2006/vcard/ns#",
    "void"    => "http://rdfs.org/ns/void#",
    "xhv"     => "http://www.w3.org/1999/xhtml/vocab#",
    "xml"     => "http://www.w3.org/XML/1998/namespace",
    "xsd"     => "http://www.w3.org/2001/XMLSchema#",
    "wdr"     => "http://www.w3.org/2007/05/powder#",
    "wdrs"    => "http://www.w3.org/2007/05/powder-s#"
  );

  for (k, v) in rdf_prefixes.iter() {
    AssignMapping!("RDFa_prefixes", k => *v);
  }
});

//! Streaming iteration over an XML file, one top-level subtree at a time.
//!
//! Wraps rust-libxml's `TextReader` (our fork; `xmlTextReaderExpand`
//! underneath): the reader walks the file event-by-event, and
//! [`FragmentReader::next_fragment`] materializes ONLY the current top-level
//! element's subtree as an owned, mutable [`Document`], then skips past it —
//! the rest of the file is never parsed into memory at once. This is the
//! "partially warm DOM" substrate of pass 2: each fragment gets real XPath,
//! rewrites, math parsing and finalize, at a peak cost of one fragment.
//!
//! XPath over a fragment document must use a context built from THAT
//! document's nodes (`Context::from_node`) — evaluating a node against
//! another document's context is the cross-doc trap recorded in
//! `xpath-cross-doc-context-node`.

use libxml::{reader::TextReader, readonly::RoNode, tree::Document};

use crate::common::error::{Error, ErrorCategory, ErrorTarget, Result};

/// Iterates the top-level subtrees of an XML file (the children of its root
/// element), materializing one owned [`Document`] at a time.
pub struct FragmentReader {
  reader:  TextReader,
  /// Depth of the fragments to materialize: children of the document root sit
  /// at reader depth 1.
  entered: bool,
}

impl FragmentReader {
  /// Open `path` for streaming. The file's root element (e.g. the
  /// `_lxfragment` wrapper of a spilled segment, or `ltx:document` of a full
  /// core-XML file) is consumed as the container; fragments are its children.
  pub fn open(path: &std::path::Path) -> Result<Self> {
    let reader = TextReader::from_file(&path.to_string_lossy(), 0)
      .map_err(|()| reader_error(format!("cannot open {}", path.display())))?;
    Ok(FragmentReader { reader, entered: false })
  }

  /// Advance to the next top-level element and materialize its whole subtree
  /// as an owned, mutable [`Document`]. Returns `Ok(None)` at the end of the
  /// container. Non-element content between fragments (whitespace, comments,
  /// PIs) is skipped.
  pub fn next_fragment(&mut self) -> Result<Option<Document>> {
    loop {
      let advanced = if self.entered && self.at_fragment() {
        // We are positioned ON a fragment we already materialized: skip its
        // whole subtree without parsing it again.
        self.reader.read_next()
      } else {
        self.reader.read()
      }
      .map_err(|()| reader_error(String::from("parse error while streaming")))?;
      if !advanced {
        return Ok(None);
      }
      if !self.entered {
        // The first element event is the container root; descend into it.
        if self.reader.is_element() && self.reader.depth() == 0 {
          self.entered = true;
        }
        continue;
      }
      if self.at_fragment() {
        let doc = self
          .reader
          .expand_to_document()
          .ok_or_else(|| reader_error(String::from("expand_to_document failed on a fragment")))?;
        return Ok(Some(doc));
      }
      // depth 0 again = the container's end-element event; anything deeper
      // than 1 cannot happen here (read_next skips whole subtrees).
    }
  }

  /// Positioned on a top-level element (a fragment root)?
  fn at_fragment(&self) -> bool { self.reader.is_element() && self.reader.depth() == 1 }

  /// Borrow the current subtree read-only without copying (valid only until
  /// the next advance) — for peeking at a fragment (name, attributes) before
  /// deciding to materialize it.
  pub fn peek(&self) -> Option<RoNode> { self.reader.expand() }
}

fn reader_error(details: String) -> Error {
  Error {
    target:   ErrorTarget::Internal,
    category: ErrorCategory::Libxml,
    message:  format!("fragment-reader: {details}"),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn write_fixture(name: &str, content: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("lxsxml-reader-{}-{name}", std::process::id()));
    std::fs::write(&path, content).unwrap();
    path
  }

  #[test]
  fn iterates_fragments_one_at_a_time() {
    let path = write_fixture(
      "basic.xml",
      "<_lxfragment xmlns:ltx=\"http://dlmf.nist.gov/LaTeXML\">\
       <ltx:para xml:id=\"p1\"><ltx:p>first Gr\u{00fc}\u{00df}e</ltx:p></ltx:para>\
       <!-- a comment between fragments -->\
       <ltx:para xml:id=\"p2\"><ltx:p>second \u{6570}\u{5b66}</ltx:p></ltx:para>\
       <ltx:pagination role=\"newpage\"/>\
       </_lxfragment>",
    );
    let mut reader = FragmentReader::open(&path).expect("open");

    let mut seen = Vec::new();
    while let Some(doc) = reader.next_fragment().expect("stream") {
      let root = doc.get_root_element().expect("fragment root");
      // The fragment document is real and owned: namespace resolved, content
      // intact, XPath-able from its own context.
      assert_eq!(
        root.get_namespace().map(|ns| ns.get_href()),
        Some(String::from("http://dlmf.nist.gov/LaTeXML")),
        "fragment root keeps the wrapper-declared namespace"
      );
      seen.push((root.get_name(), root.get_content()));
    }
    let _ = std::fs::remove_file(&path);

    assert_eq!(seen.len(), 3, "three fragments, comment skipped: {seen:?}");
    assert_eq!(seen[0].0, "para");
    assert!(seen[0].1.contains("first Gr\u{00fc}\u{00df}e"));
    assert_eq!(seen[1].0, "para");
    assert!(seen[1].1.contains("second \u{6570}\u{5b66}"));
    assert_eq!(seen[2].0, "pagination", "empty-element fragment survives");
  }

  #[test]
  fn fragment_documents_are_independent_and_mutable() {
    let path = write_fixture(
      "mutable.xml",
      "<root><a n=\"1\"><child/></a><b n=\"2\"/></root>",
    );
    let mut reader = FragmentReader::open(&path).expect("open");

    let doc_a = reader.next_fragment().expect("stream").expect("fragment a");
    let mut root_a = doc_a.get_root_element().unwrap();
    // Mutating fragment A must not disturb streaming of fragment B.
    root_a.set_attribute("mutated", "yes").expect("mutable");

    let doc_b = reader.next_fragment().expect("stream").expect("fragment b");
    let root_b = doc_b.get_root_element().unwrap();
    assert_eq!(root_b.get_name(), "b");
    assert_eq!(root_b.get_attribute("n").as_deref(), Some("2"));
    assert_eq!(root_a.get_attribute("mutated").as_deref(), Some("yes"));

    assert!(
      reader.next_fragment().expect("stream").is_none(),
      "exhausted"
    );
    let _ = std::fs::remove_file(&path);
  }

  #[test]
  fn open_missing_file_is_an_error() {
    let missing = std::env::temp_dir().join("lxsxml-definitely-not-here.xml");
    assert!(FragmentReader::open(&missing).is_err());
  }

  #[test]
  fn segment_store_round_trip_streams_back() {
    // The integration the substrate exists for: spill via SegmentStore, read
    // back via FragmentReader.
    use crate::sxml::{SegmentMeta, SegmentStore};
    let tmp = std::env::temp_dir().join(format!("lxsxml-integ-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let mut store = SegmentStore::create(&tmp).expect("store");
    let meta = SegmentMeta {
      depth:      1,
      font:       None,
      namespaces: vec![(
        String::from("ltx"),
        String::from("http://dlmf.nist.gov/LaTeXML"),
      )],
    };
    let id = store
      .write_segment(
        "<ltx:section xml:id=\"S1\"><ltx:title>One</ltx:title></ltx:section>\
         <ltx:section xml:id=\"S2\"><ltx:title>Two</ltx:title></ltx:section>",
        meta,
      )
      .expect("write");

    let mut reader = FragmentReader::open(&store.segment_path(id)).expect("open segment");
    let mut titles = Vec::new();
    while let Some(doc) = reader.next_fragment().expect("stream") {
      let root = doc.get_root_element().unwrap();
      assert_eq!(root.get_name(), "section");
      titles.push(root.get_content());
    }
    assert_eq!(titles, vec![String::from("One"), String::from("Two")]);
    drop(store);
    let _ = std::fs::remove_dir_all(&tmp);
  }
}

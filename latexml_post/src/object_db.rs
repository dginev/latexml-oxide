//! Object database for cross-document data sharing.
//!
//! Port of `LaTeXML::Util::ObjectDB` + `ObjectDB::Entry`.
//! A key-value store used by Scan, CrossRef, MakeIndex, and MakeBibliography
//! to share structural information across documents and processing phases.
//!
//! Keys follow conventions:
//! - `ID:<xml:id>` — element data (type, parent, children, labels, location, etc.)
//! - `LABEL:<label>` — label → ID mapping
//! - `DOCUMENT:<path>` — document location → root ID mapping
//! - `SITE_ROOT` — root document of the site
//! - `BIBLABEL:<list>:<key>` — bibliography key → item ID
//! - `GLOSSARY:<list>:<key>` — glossary entries
//! - `INDEX:<phrase1>:<phrase2>:...` — index entries
//! - `DECLARATION:(global|local):<name>` — declared symbols
//! - `NOTATION:<name>` — notation entries

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

/// A single entry in the ObjectDB.
///
/// Port of `LaTeXML::Util::ObjectDB::Entry`.
#[derive(Debug, Clone)]
pub struct Entry {
  /// The key this entry is stored under.
  pub key: String,
  /// Attribute-value pairs.
  values:  HashMap<String, Value>,
}

/// A value stored in an Entry.
///
/// Values can be scalars, lists, nested hashes, or XML node references.
#[derive(Debug, Clone)]
pub enum Value {
  /// A simple string value.
  String(String),
  /// An integer value.
  Int(i64),
  /// A boolean value.
  Bool(bool),
  /// A list of values.
  List(Vec<Value>),
  /// A nested hash (for associations like referrers).
  Hash(HashMap<String, Value>),
  /// An XML node (cloned from the document).
  Xml(Node),
  /// Null/undefined.
  Null,
}

impl Value {
  /// Get as string, if possible.
  pub fn as_str(&self) -> Option<&str> {
    match self {
      Value::String(s) => Some(s),
      _ => None,
    }
  }

  /// Get as string, converting if needed.
  pub fn as_string(&self) -> String {
    match self {
      Value::String(s) => s.clone(),
      Value::Int(n) => n.to_string(),
      Value::Bool(b) => b.to_string(),
      Value::Xml(node) => node.get_content(),
      Value::Null => String::new(),
      _ => String::new(),
    }
  }

  /// Check if the value is truthy (non-null, non-empty).
  pub fn is_truthy(&self) -> bool {
    match self {
      Value::Null => false,
      Value::String(s) => !s.is_empty(),
      Value::Bool(b) => *b,
      Value::List(v) => !v.is_empty(),
      Value::Hash(h) => !h.is_empty(),
      _ => true,
    }
  }
}

impl From<&str> for Value {
  fn from(s: &str) -> Self { Value::String(s.to_string()) }
}

impl From<String> for Value {
  fn from(s: String) -> Self { Value::String(s) }
}

impl std::fmt::Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Value::String(s) => write!(f, "{}", s),
      Value::Int(n) => write!(f, "{}", n),
      Value::Bool(b) => write!(f, "{}", b),
      Value::Null => Ok(()),
      Value::Xml(node) => write!(f, "{}", node.get_content()),
      Value::List(_) | Value::Hash(_) => Ok(()),
    }
  }
}

// NOTE deliberately NO `impl From<Node> for Value`: a bare node handle points
// into whichever document it came from, and storing one couples the DB's
// validity to that document's lifetime (fatal under streaming, where page DOMs
// are freed as soon as they are processed). Nodes enter the DB only through
// `ObjectDB::adopt_xml`, which copies them into DB-owned storage.

impl From<bool> for Value {
  fn from(b: bool) -> Self { Value::Bool(b) }
}

impl From<Vec<String>> for Value {
  fn from(v: Vec<String>) -> Self { Value::List(v.into_iter().map(Value::String).collect()) }
}

impl Entry {
  /// Create a new entry with the given key.
  pub fn new(key: &str) -> Self {
    Entry {
      key:    key.to_string(),
      values: HashMap::default(),
    }
  }

  /// Get the entry's key.
  pub fn get_key(&self) -> &str { &self.key }

  /// Check if the entry has a value for the given attribute.
  pub fn has_value(&self, attr: &str) -> bool { self.values.contains_key(attr) }

  /// Get a value by attribute name.
  pub fn get_value(&self, attr: &str) -> Option<&Value> { self.values.get(attr) }

  /// Get a string value by attribute name.
  pub fn get_string(&self, attr: &str) -> Option<&str> {
    self.values.get(attr).and_then(|v| v.as_str())
  }

  /// Get an XML node value by attribute name.
  pub fn get_xml(&self, attr: &str) -> Option<&Node> {
    match self.values.get(attr) {
      Some(Value::Xml(n)) => Some(n),
      _ => None,
    }
  }

  /// Get a children list (as string IDs).
  pub fn get_children(&self) -> Vec<String> {
    match self.values.get("children") {
      Some(Value::List(items)) => items
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect(),
      _ => vec![],
    }
  }

  /// Set multiple attribute-value pairs.
  ///
  /// Port of `Entry::setValues`.
  pub fn set_values(&mut self, pairs: Vec<(&str, Value)>) {
    for (key, value) in pairs {
      match value {
        Value::Null => {
          self.values.remove(key);
        },
        _ => {
          self.values.insert(key.to_string(), value);
        },
      }
    }
  }

  /// Set a single value.
  pub fn set_value(&mut self, attr: &str, value: Value) {
    match value {
      Value::Null => {
        self.values.remove(attr);
      },
      _ => {
        self.values.insert(attr.to_string(), value);
      },
    }
  }

  /// Push values onto a list attribute.
  ///
  /// Port of `Entry::pushValues`.
  pub fn push_values(&mut self, attr: &str, values: Vec<Value>) {
    let list = self
      .values
      .entry(attr.to_string())
      .or_insert_with(|| Value::List(Vec::new()));
    if let Value::List(items) = list {
      for v in values {
        items.push(v);
      }
    }
  }

  /// Push values onto a list attribute, skipping duplicates.
  ///
  /// Port of `Entry::pushNew`.
  pub fn push_new(&mut self, attr: &str, values: Vec<Value>) {
    let list = self
      .values
      .entry(attr.to_string())
      .or_insert_with(|| Value::List(Vec::new()));
    if let Value::List(items) = list {
      for v in values {
        let s = v.to_string();
        if !items.iter().any(|existing| existing.to_string() == s) {
          items.push(v);
        }
      }
    }
  }

  /// Create nested hash association.
  ///
  /// Port of `Entry::noteAssociation`.
  /// `noteAssociation("referrers", "parent_id")` creates `{referrers => {parent_id => 1}}`
  pub fn note_association(&mut self, keys: &[&str]) {
    if keys.is_empty() {
      return;
    }
    if keys.len() == 1 {
      self.values.insert(keys[0].to_string(), Value::Bool(true));
      return;
    }

    // Navigate/create nested hash structure
    let first = keys[0];
    let rest = &keys[1..];

    let hash = self
      .values
      .entry(first.to_string())
      .or_insert_with(|| Value::Hash(HashMap::default()));

    if let Value::Hash(h) = hash {
      let mut current = h;
      for (i, &key) in rest.iter().enumerate() {
        if i == rest.len() - 1 {
          // Last key: set to true
          current.insert(key.to_string(), Value::Bool(true));
        } else {
          // Intermediate: navigate/create hash
          let entry = current
            .entry(key.to_string())
            .or_insert_with(|| Value::Hash(HashMap::default()));
          if let Value::Hash(inner) = entry {
            current = inner;
          } else {
            break;
          }
        }
      }
    }
  }
}

/// The Object Database.
///
/// Port of `LaTeXML::Util::ObjectDB`.
/// In-memory key-value store. For now, no external DB persistence
/// (the Perl version uses Berkeley DB via DB_File).
pub struct ObjectDB {
  /// In-memory entry storage.
  objects:  HashMap<String, Entry>,
  /// Owners of every [`Value::Xml`] node the DB stores. A stored node used to
  /// be a handle into the scanned page's own DOM, which made the DB's
  /// lifetime silently depend on every page document staying resident — a
  /// use-after-free the moment a streaming pipeline frees a processed page.
  /// [`ObjectDB::adopt_xml`] deep-copies into a fresh document owned HERE, so
  /// stored XML lives exactly as long as the DB, whatever happens to the
  /// source. (Titles/phrases are small; the arena cost is negligible.)
  xml_docs: Vec<libxml::tree::Document>,
}

impl ObjectDB {
  /// Create a new empty ObjectDB.
  pub fn new() -> Self {
    ObjectDB {
      objects:  HashMap::default(),
      xml_docs: Vec::new(),
    }
  }

  /// Adopt an XML node for storage: deep-copy it into a document the DB owns
  /// and return the [`Value::Xml`] wrapping the copy. This is the ONLY way a
  /// node enters the DB (`From<Node> for Value` was removed on purpose), so
  /// no stored value can dangle into a page document that was freed.
  ///
  /// Uses `dup_node_into_new_doc` — an orphan deep copy into a fresh
  /// document, sharing zero C-side state with the source, namespaces
  /// reconciled (the same primitive `Split` page extraction relies on).
  /// Returns `None` when the copy fails; callers should degrade to a string
  /// form rather than store nothing, and never crash.
  pub fn adopt_xml(&mut self, node: &Node) -> Option<Value> {
    let doc = libxml::tree::Document::dup_node_into_new_doc(node).ok()?;
    let copy = doc.get_root_element()?;
    self.xml_docs.push(doc);
    Some(Value::Xml(copy))
  }

  /// Look up an entry by key.
  ///
  /// Port of `ObjectDB::lookup`.
  pub fn lookup(&self, key: &str) -> Option<&Entry> { self.objects.get(key) }

  /// Look up an entry by key (mutable).
  pub fn lookup_mut(&mut self, key: &str) -> Option<&mut Entry> { self.objects.get_mut(key) }

  /// Register an entry: create if new, or return existing.
  /// Sets the given properties on the entry.
  ///
  /// Port of `ObjectDB::register`.
  pub fn register(&mut self, key: &str, props: Vec<(&str, Value)>) -> &mut Entry {
    let entry = self
      .objects
      .entry(key.to_string())
      .or_insert_with(|| Entry::new(key));
    if !props.is_empty() {
      entry.set_values(props);
    }
    self.objects.get_mut(key).unwrap()
  }

  /// Remove an entry.
  ///
  /// Port of `ObjectDB::unregister`.
  pub fn unregister(&mut self, key: &str) { self.objects.remove(key); }

  /// Get all keys, sorted.
  ///
  /// Port of `ObjectDB::getKeys`.
  pub fn get_keys(&self) -> Vec<&String> {
    let mut keys: Vec<_> = self.objects.keys().collect();
    keys.sort();
    keys
  }

  /// Number of registered entries. O(1); avoids `get_keys().len()`'s
  /// sort + allocation when only the count is needed.
  pub fn len(&self) -> usize { self.objects.len() }

  /// True when no entries are registered.
  pub fn is_empty(&self) -> bool { self.objects.is_empty() }

  /// Iterate keys in arbitrary order without allocating/sorting. Use when
  /// the traversal order does not matter (e.g. per-node fill-ins).
  pub fn keys_iter(&self) -> impl Iterator<Item = &String> { self.objects.keys() }

  /// Return a status string.
  ///
  /// Port of `ObjectDB::status`.
  pub fn status(&self) -> String { format!("{} objects", self.objects.len()) }
}

impl Default for ObjectDB {
  fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_entry_basic() {
    let mut entry = Entry::new("test:key");
    assert_eq!(entry.get_key(), "test:key");
    assert!(!entry.has_value("name"));

    entry.set_value("name", Value::from("Alice"));
    assert!(entry.has_value("name"));
    assert_eq!(entry.get_string("name"), Some("Alice"));
  }

  #[test]
  fn test_entry_push_new() {
    let mut entry = Entry::new("test");
    entry.push_new("children", vec![Value::from("a"), Value::from("b")]);
    entry.push_new("children", vec![Value::from("b"), Value::from("c")]);
    // "b" should not be duplicated
    let children = entry.get_children();
    assert_eq!(children, vec!["a", "b", "c"]);
  }

  #[test]
  fn test_entry_note_association() {
    let mut entry = Entry::new("test");
    entry.note_association(&["referrers", "doc1"]);
    entry.note_association(&["referrers", "doc2"]);

    assert!(entry.has_value("referrers"));
    if let Some(Value::Hash(refs)) = entry.get_value("referrers") {
      assert!(refs.contains_key("doc1"));
      assert!(refs.contains_key("doc2"));
    } else {
      panic!("Expected Hash value for referrers");
    }
  }

  #[test]
  fn test_db_register_lookup() {
    let mut db = ObjectDB::new();

    db.register("ID:doc1", vec![
      ("type", Value::from("ltx:document")),
      ("title", Value::from("Test Document")),
    ]);

    let entry = db.lookup("ID:doc1");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().get_string("type"), Some("ltx:document"));
    assert_eq!(entry.unwrap().get_string("title"), Some("Test Document"));

    // Lookup non-existent
    assert!(db.lookup("ID:missing").is_none());
  }

  #[test]
  fn test_db_register_updates() {
    let mut db = ObjectDB::new();
    db.register("ID:x", vec![("a", Value::from("1"))]);
    db.register("ID:x", vec![("b", Value::from("2"))]);

    let entry = db.lookup("ID:x").unwrap();
    assert_eq!(entry.get_string("a"), Some("1"));
    assert_eq!(entry.get_string("b"), Some("2"));
  }

  #[test]
  fn test_db_get_keys() {
    let mut db = ObjectDB::new();
    db.register("B", vec![]);
    db.register("A", vec![]);
    db.register("C", vec![]);

    let keys = db.get_keys();
    assert_eq!(keys, vec![
      &"A".to_string(),
      &"B".to_string(),
      &"C".to_string()
    ]);
  }

  #[test]
  fn test_db_status() {
    let mut db = ObjectDB::new();
    assert_eq!(db.status(), "0 objects");
    db.register("x", vec![]);
    assert_eq!(db.status(), "1 objects");
  }

  #[test]
  fn adopted_xml_survives_source_document_drop() {
    // The streaming contract: a stored XML value must stay valid after the
    // page document it was scanned from is freed. Under the old
    // `Value::Xml(source_node)` representation this was a use-after-free.
    let parser = libxml::parser::Parser::default();
    let source = parser
      .parse_string(
        "<title xmlns:m=\"http://www.w3.org/1998/Math/MathML\">\
         The <m:mi>\u{03b1}</m:mi> section</title>",
      )
      .expect("parse source");
    let title = source.get_root_element().expect("root");

    let mut db = ObjectDB::new();
    let adopted = db.adopt_xml(&title).expect("adopt");
    db.register("ID:S1", vec![("title", adopted)]);
    drop(source); // the page DOM goes away, as it will under streaming

    let entry = db.lookup("ID:S1").expect("entry");
    let node = entry.get_xml("title").expect("stored node");
    assert_eq!(node.get_name(), "title");
    assert_eq!(node.get_content(), "The \u{03b1} section");
    // Markup survives adoption, not just flattened text: the MathML child is
    // still an element in its namespace.
    let mi = node
      .get_child_nodes()
      .into_iter()
      .find(|c| c.get_name() == "mi")
      .expect("m:mi child survives");
    assert_eq!(
      mi.get_namespace().map(|ns| ns.get_href()),
      Some(String::from("http://www.w3.org/1998/Math/MathML"))
    );
  }

  #[test]
  fn test_value_truthy() {
    assert!(!Value::Null.is_truthy());
    assert!(!Value::String(String::new()).is_truthy());
    assert!(Value::String("hello".to_string()).is_truthy());
    assert!(Value::Bool(true).is_truthy());
    assert!(!Value::Bool(false).is_truthy());
    assert!(Value::Int(42).is_truthy());
    assert!(!Value::List(vec![]).is_truthy());
    assert!(Value::List(vec![Value::Int(1)]).is_truthy());
  }
}

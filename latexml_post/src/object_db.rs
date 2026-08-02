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
  objects:    HashMap<String, Entry>,
  /// The ONE document owning every [`Value::Xml`] node the DB stores. A
  /// stored node used to be a handle into the scanned page's own DOM, which
  /// made the DB's lifetime silently depend on every page document staying
  /// resident — a use-after-free the moment a streaming pipeline frees a
  /// processed page. [`ObjectDB::adopt_xml`] deep-copies into this document
  /// instead, so stored XML lives exactly as long as the DB, whatever happens
  /// to the source.
  ///
  /// **One document, not one per value.** The first cut retained a whole fresh
  /// `xmlDoc` per adopted node, and Scan adopts a `title` and `toctitle` for
  /// every object it registers: on a 614 MB core XML (200,403 objects) that is
  /// tens of thousands of documents — each with its own dictionary and
  /// structure — to hold a few MB of title markup. Measured, that regressed
  /// the split post-processing of that input from ~21 GB peak (completing over
  /// 40,201 pages) to **67 GB and a memory-ceiling kill with zero pages
  /// written**. Titles really are small; `xmlDoc`s are not.
  xml_holder: Option<libxml::tree::Document>,
  /// The attached external store, when this DB was opened via
  /// [`ObjectDB::attach`] (Perl `--dbfile`); `None` for purely in-memory use.
  external:   Option<ExternalDb>,
}

impl ObjectDB {
  /// Create a new empty ObjectDB.
  pub fn new() -> Self {
    ObjectDB {
      objects:    HashMap::default(),
      xml_holder: None,
      external:   None,
    }
  }

  /// The DB's holding document, created on first adoption (so a DB that
  /// stores no XML allocates none), with a root element to parent the copies
  /// under — an unparented node would not be reached by the holder's own
  /// `xmlFreeDoc` either.
  fn xml_holder_mut(&mut self) -> Option<&mut libxml::tree::Document> {
    if self.xml_holder.is_none() {
      let mut doc = libxml::tree::Document::new().ok()?;
      let root = Node::new("_objectdb_", None, &doc).ok()?;
      doc.set_root_element(&root);
      self.xml_holder = Some(doc);
    }
    self.xml_holder.as_mut()
  }

  /// Adopt an XML node for storage: deep-copy it into a document the DB owns
  /// and return the [`Value::Xml`] wrapping the copy. This is the ONLY way a
  /// node enters the DB (`From<Node> for Value` was removed on purpose), so
  /// no stored value can dangle into a page document that was freed.
  ///
  /// Two hops, because the fork's copy primitives pull in opposite
  /// directions: `dup_node_into_new_doc` copies from a LINKED source but only
  /// into a fresh document, while `import_node` copies into an EXISTING
  /// document but rejects a linked source. So: copy out to a scratch
  /// document, detach, copy into the holder, and free the scratch copy —
  /// an unlinked doc-owned node is freed by nobody (the rust-libxml `Linkage`
  /// rule behind `Node::free_subtree`), so dropping the scratch document
  /// alone would leak it. A one-hop version wants a fork method that copies
  /// from a linked source into an existing document; that is a publish + dep
  /// bump, and this is a regression fix.
  ///
  /// Returns `None` when the copy fails; callers should degrade to a string
  /// form rather than store nothing, and never crash.
  pub fn adopt_xml(&mut self, node: &Node) -> Option<Value> {
    let scratch = libxml::tree::Document::dup_node_into_new_doc(node).ok()?;
    let mut extracted = scratch.get_root_element()?;
    extracted.unlink_node();
    let holder = self.xml_holder_mut()?;
    let mut adopted = holder.import_node(&mut extracted).ok()?;
    let mut root = holder.get_root_element()?;
    root.add_child(&mut adopted).ok()?;
    extracted.free_subtree();
    Some(Value::Xml(adopted))
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
  pub fn unregister(&mut self, key: &str) {
    self.objects.remove(key);
    // Perl ObjectDB.pm:183: "Must remove external entry (if any) as well,
    // else it'll get pulled back in!" — without this the key resurrects on
    // the next attach (review 2026-08-03 finding #1).
    if let Some(external) = &mut self.external {
      external.baseline.remove(key);
      if !external.readonly {
        let _ = external
          .conn
          .execute("DELETE FROM entries WHERE key = ?1", rusqlite::params![key]);
      }
    }
  }

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

// ======================================================================
// Perl `--dbfile` parity: SQLite persistence (design 2026-08-02, docs/
// performance/STREAMING_POST_DESIGN_2026-07-06.md §6). Faithful to the
// OBSERVABLE contract of `LaTeXML::Util::ObjectDB` (ObjectDB.pm):
// `new(dbfile)` attaches a keyed store (creating it unless readonly),
// `lookup`/`getKeys` see the union of stored + registered entries, and
// `finish` writes back ONLY entries that differ from what is stored
// (ObjectDB.pm `sub finish`: `next if compare_hash($row, thaw($stored))`),
// then detaches. One deliberate internal divergence: Perl thaws entries
// lazily per `lookup` because Berkeley DB reads are cheap point-lookups;
// we load eagerly at attach — SQLite reads the whole table in one scan,
// every consumer (CrossRef's nav/TOC walks) touches most keys anyway, and
// eager load keeps `lookup(&self)` free of interior mutability.
//
// Storage: `entries(key TEXT PRIMARY KEY, props TEXT)` with the property
// map JSON-encoded (Perl uses Storable `nfreeze`; JSON keeps the artifact
// `sqlite3`-CLI-inspectable, which Storable never was), plus a `meta`
// table and `PRAGMA user_version` for staleness: a format-version mismatch
// refuses the file rather than limping (caller decides to rebuild).
// `Value::Xml` round-trips as serialized XML, re-adopted into the reader's
// own holder document via the same `adopt_xml` copy discipline Scan uses.

/// Bump when the on-disk encoding changes shape. Mismatched files are
/// refused at `attach` — never silently reinterpreted.
pub const DBFILE_FORMAT_VERSION: i64 = 1;

/// The attached external store: connection plus the as-stored JSON of every
/// loaded entry, so `finish` can implement Perl's changed-only write-back
/// by string comparison instead of re-reading the table.
struct ExternalDb {
  conn:     rusqlite::Connection,
  readonly: bool,
  baseline: HashMap<String, String>,
}

/// Options for [`ObjectDB::attach`], mirroring Perl `ObjectDB::new`'s
/// `dbfile`/`clean`/`readonly` knobs.
#[derive(Default)]
pub struct DbAttachOptions {
  /// Delete any existing file first (Perl `clean => 1`).
  pub clean:    bool,
  /// Open for reading only; `finish` will not write (Perl `readonly`).
  pub readonly: bool,
}

impl ObjectDB {
  /// Attach (and load) an external SQLite object store — Perl
  /// `ObjectDB->new(dbfile => …)`.
  pub fn attach(dbfile: &std::path::Path, options: DbAttachOptions) -> Result<Self, String> {
    if options.clean && options.readonly {
      return Err("dbfile: clean requires write access (Perl always opens O_RDWR)".to_string());
    }
    if options.clean && dbfile.exists() {
      latexml_core::common::error::emit_warn(
        "expected",
        "dbfile",
        &format!("Removing Object database file {}!", dbfile.display()),
      );
      std::fs::remove_file(dbfile)
        .map_err(|e| format!("cannot remove {}: {e}", dbfile.display()))?;
    }
    let conn = if options.readonly {
      rusqlite::Connection::open_with_flags(dbfile, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
    } else {
      rusqlite::Connection::open(dbfile)
    }
    .map_err(|e| format!("cannot attach DB {}: {e}", dbfile.display()))?;
    // Concurrent workers are this layer's purpose: WAL lets N readers overlap
    // one writer, and a busy timeout rides out a writer's commit instead of
    // failing mid-transaction with SQLITE_BUSY.
    if !options.readonly {
      let _ = conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()));
    }
    conn
      .busy_timeout(std::time::Duration::from_secs(10))
      .map_err(|e| format!("cannot set busy_timeout: {e}"))?;

    let version: i64 = conn
      .query_row("PRAGMA user_version", [], |r| r.get(0))
      .map_err(|e| format!("cannot read user_version: {e}"))?;
    let entries_table: bool = conn
      .query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries'",
        [],
        |r| r.get::<_, i64>(0),
      )
      .map(|n| n > 0)
      .unwrap_or(false);
    let any_table: bool = conn
      .query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
        [],
        |r| r.get::<_, i64>(0),
      )
      .map(|n| n > 0)
      .unwrap_or(false);
    if version == 0 && any_table && !entries_table {
      return Err(format!(
        "{} is a SQLite file but not an ObjectDB (no entries table) — refusing to adopt it",
        dbfile.display()
      ));
    }
    if version == 0 && !options.readonly {
      conn
        .execute_batch(&format!(
          "PRAGMA user_version = {DBFILE_FORMAT_VERSION};
           CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
           CREATE TABLE IF NOT EXISTS entries(key TEXT PRIMARY KEY, props TEXT NOT NULL);"
        ))
        .map_err(|e| format!("cannot initialize {}: {e}", dbfile.display()))?;
    } else if version != 0 && version != DBFILE_FORMAT_VERSION {
      return Err(format!(
        "object database {} has format version {version}, this binary reads {DBFILE_FORMAT_VERSION} — rebuild it (--dbfile with a clean run)",
        dbfile.display()
      ));
    }

    let mut db = ObjectDB::new();
    let mut baseline = HashMap::default();
    // A fresh readonly file has no tables; treat as empty rather than erroring.
    let table_exists: bool = conn
      .query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries'",
        [],
        |r| r.get::<_, i64>(0),
      )
      .map(|n| n > 0)
      .unwrap_or(false);
    if table_exists {
      let mut stmt = conn
        .prepare("SELECT key, props FROM entries")
        .map_err(|e| format!("cannot read entries: {e}"))?;
      let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| format!("cannot scan entries: {e}"))?;
      for row in rows {
        let (key, props) = row.map_err(|e| format!("bad row: {e}"))?;
        // One bad row must not poison the whole file (review finding #4):
        // warn with the key and skip; the on-disk row stays as it was.
        match db.decode_entry(&key, &props) {
          Ok(entry) => {
            db.objects.insert(key.clone(), entry);
            baseline.insert(key, props);
          },
          Err(e) => latexml_core::common::error::emit_warn("malformed", "dbfile_entry", &e),
        }
      }
      drop(stmt);
    }
    db.external = Some(ExternalDb {
      conn,
      readonly: options.readonly,
      baseline,
    });
    Ok(db)
  }

  /// Write back changed entries and detach — Perl `ObjectDB::finish`.
  /// Returns how many entries were stored. Idempotent: a second call (or a
  /// call on a never-attached DB) stores nothing.
  pub fn finish(&mut self) -> Result<usize, String> {
    let Some(external) = self.external.take() else {
      return Ok(0);
    };
    if external.readonly {
      return Ok(0);
    }
    // Stream: encode → compare → insert per entry inside one transaction, so
    // peak memory is ONE encoded entry, not the whole changed set (review
    // finding #5 — the baseline copy itself is dropped with `external` at
    // the end of this function).
    let mut stored = 0usize;
    let mut conn = external.conn;
    let tx = conn
      .transaction()
      .map_err(|e| format!("cannot begin dbfile transaction: {e}"))?;
    for (key, entry) in &self.objects {
      let props = {
        let map: serde_json::Map<String, serde_json::Value> = entry
          .values
          .iter()
          .map(|(k, v)| (k.clone(), value_to_json_with(self.xml_holder.as_ref(), v)))
          .collect();
        serde_json::Value::Object(map).to_string()
      };
      if external.baseline.get(key) == Some(&props) {
        continue;
      }
      tx.execute(
        "INSERT INTO entries(key, props) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET props = excluded.props",
        rusqlite::params![key, props],
      )
      .map_err(|e| format!("cannot store entry {key}: {e}"))?;
      stored += 1;
    }
    tx.execute(
      "INSERT INTO meta(key, value) VALUES ('latexml_version', ?1)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      rusqlite::params![env!("CARGO_PKG_VERSION")],
    )
    .map_err(|e| format!("cannot store meta: {e}"))?;
    tx.commit()
      .map_err(|e| format!("cannot commit dbfile: {e}"))?;
    Ok(stored)
  }

  /// Tagged, self-describing encoding — one-key objects so `Hash` values
  /// can never be confused with the wrapper: `{"s":…}` string, `{"i":…}`
  /// int, `{"b":…}` bool, `{"l":[…]}` list, `{"h":{…}}` hash, `{"x":"<…>"}`
  /// serialized XML, JSON `null` for `Null`.
  fn value_to_json(&self, value: &Value) -> serde_json::Value {
    value_to_json_with(self.xml_holder.as_ref(), value)
  }

  fn decode_entry(&mut self, key: &str, props: &str) -> Result<Entry, String> {
    let parsed: serde_json::Value =
      serde_json::from_str(props).map_err(|e| format!("entry {key} is not valid JSON: {e}"))?;
    let serde_json::Value::Object(map) = parsed else {
      return Err(format!("entry {key} is not a JSON object"));
    };
    let mut entry = Entry::new(key);
    for (attr, jv) in map {
      let value = self.json_to_value(&jv, key)?;
      entry.set_value(&attr, value);
    }
    Ok(entry)
  }

  fn json_to_value(&mut self, jv: &serde_json::Value, key: &str) -> Result<Value, String> {
    use serde_json::Value as J;
    let J::Object(o) = jv else {
      return match jv {
        J::Null => Ok(Value::Null),
        other => Err(format!("entry {key}: unexpected bare JSON value {other}")),
      };
    };
    if let Some((tag, inner)) = o.iter().next()
      && o.len() == 1
    {
      return match (tag.as_str(), inner) {
        ("s", J::String(s)) => Ok(Value::String(s.clone())),
        ("i", J::Number(n)) => n
          .as_i64()
          .map(Value::Int)
          .ok_or_else(|| format!("entry {key}: non-integer numeric value {n}")),
        ("b", J::Bool(b)) => Ok(Value::Bool(*b)),
        ("l", J::Array(items)) => Ok(Value::List(
          items
            .iter()
            .map(|v| self.json_to_value(v, key))
            .collect::<Result<Vec<_>, _>>()?,
        )),
        ("h", J::Object(map)) => {
          let mut out = HashMap::default();
          for (k, v) in map {
            out.insert(k.clone(), self.json_to_value(v, key)?);
          }
          Ok(Value::Hash(out))
        },
        ("x", J::String(xml)) => {
          let parsed = libxml::parser::Parser::default()
            .parse_string(xml)
            .map_err(|e| format!("entry {key}: stored XML does not parse: {e}"))?;
          let root = parsed
            .get_root_element()
            .ok_or_else(|| format!("entry {key}: stored XML is empty"))?;
          self
            .adopt_xml(&root)
            .ok_or_else(|| format!("entry {key}: could not adopt stored XML"))
        },
        (tag, _) => Err(format!("entry {key}: unknown value tag '{tag}'")),
      };
    }
    Err(format!("entry {key}: malformed value object"))
  }
}

/// Tagged, self-describing encoding (see [`ObjectDB::value_to_json`]); a free
/// function so `finish` can encode while iterating `self.objects`.
fn value_to_json_with(holder: Option<&libxml::tree::Document>, value: &Value) -> serde_json::Value {
  use serde_json::{Value as J, json};
  match value {
    Value::String(s) => json!({ "s": s }),
    Value::Int(i) => json!({ "i": i }),
    Value::Bool(b) => json!({ "b": b }),
    Value::Null => J::Null,
    Value::List(items) => {
      json!({ "l": items.iter().map(|v| value_to_json_with(holder, v)).collect::<Vec<_>>() })
    },
    Value::Hash(map) => {
      json!({ "h": map.iter().map(|(k, v)| (k.clone(), value_to_json_with(holder, v))).collect::<serde_json::Map<_, _>>() })
    },
    Value::Xml(node) => {
      let xml = holder
        .map(|doc| doc.node_to_string(node))
        .unwrap_or_default();
      json!({ "x": xml })
    },
  }
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
  fn many_adoptions_share_one_holding_document() {
    // The regression this guards: one `xmlDoc` per adopted value. Scan adopts
    // a title (and toctitle) per registered object, so a book-scale document
    // adopted tens of thousands of times — measured, that took split
    // post-processing of a 614 MB core XML from ~21 GB to 67 GB and a
    // memory-ceiling kill. Every copy must live in ONE document, and all of
    // them must still be readable after the sources are gone.
    let parser = libxml::parser::Parser::default();
    let mut db = ObjectDB::new();
    let mut sources = Vec::new();
    for i in 0..64 {
      let doc = parser
        .parse_string(format!("<title>Section <em>{i}</em></title>").as_bytes())
        .expect("parse source");
      let root = doc.get_root_element().expect("root");
      let adopted = db.adopt_xml(&root).expect("adopt");
      db.register(&format!("ID:S{i}"), vec![("title", adopted)]);
      sources.push(doc);
    }
    drop(sources); // every page DOM goes away
    for i in 0..64 {
      let entry = db.lookup(&format!("ID:S{i}")).expect("entry");
      let node = entry.get_xml("title").expect("stored node");
      assert_eq!(node.get_content(), format!("Section {i}"));
    }
    // All 64 copies are children of the single holder root, which is what
    // keeps the retained cost constant instead of linear in objects.
    let holder = db
      .xml_holder
      .as_ref()
      .expect("holder exists after adoption");
    let root = holder.get_root_element().expect("holder root");
    assert_eq!(root.get_child_elements().len(), 64);
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

  /// Perl `--dbfile` parity: a DB round-trips through its SQLite file —
  /// scalars, lists, nested hashes, and adopted XML (re-adopted into the
  /// READER's holder document, never a dangling node).
  #[test]
  fn dbfile_round_trips_all_value_shapes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dbfile = dir.path().join("site.db");

    let mut db = ObjectDB::attach(&dbfile, DbAttachOptions::default()).expect("attach fresh");
    let title_doc = libxml::parser::Parser::default()
      .parse_string("<ltx:text xmlns:ltx=\"http://dlmf.nist.gov/LaTeXML\">A <ltx:emph>title</ltx:emph></ltx:text>")
      .expect("title parses");
    let title = db
      .adopt_xml(&title_doc.get_root_element().expect("root"))
      .expect("adopted");
    db.register("doc1#s1", vec![
      ("type", Value::String("ltx:section".into())),
      ("pageid", Value::Int(3)),
      ("fresh", Value::Bool(true)),
      (
        "children",
        Value::List(vec![
          Value::String("doc1#s1.p1".into()),
          Value::String("doc1#s1.p2".into()),
        ]),
      ),
      (
        "referrers",
        Value::Hash({
          let mut h = HashMap::default();
          h.insert("doc2".to_string(), Value::String("ref".into()));
          h
        }),
      ),
      ("title", title),
      ("missing", Value::Null),
    ]);
    let stored = db.finish().expect("finish writes");
    assert_eq!(stored, 1, "one changed entry stored");

    let mut reloaded = ObjectDB::attach(&dbfile, DbAttachOptions {
      readonly: true,
      ..Default::default()
    })
    .expect("attach readonly");
    let entry = reloaded.lookup("doc1#s1").expect("entry survives");
    assert_eq!(entry.get_string("type"), Some("ltx:section"));
    assert!(matches!(entry.get_value("pageid"), Some(Value::Int(3))));
    assert!(matches!(entry.get_value("fresh"), Some(Value::Bool(true))));
    assert_eq!(entry.get_children(), vec!["doc1#s1.p1", "doc1#s1.p2"]);
    assert!(matches!(entry.get_value("referrers"), Some(Value::Hash(_))));
    let title = entry.get_xml("title").expect("XML value survives");
    assert!(
      title.get_content().contains("A title"),
      "adopted XML content round-trips"
    );
    assert_eq!(reloaded.finish().expect("readonly finish"), 0);

    // XML idempotence (review finding #3): re-attach READ-WRITE and finish —
    // if serialize→adopt→serialize is not byte-stable, the XML entry rewrites
    // on every finish and the changed-only contract silently degrades to
    // rewrite-everything.
    let mut again = ObjectDB::attach(&dbfile, DbAttachOptions::default()).expect("attach rw");
    assert_eq!(
      again.finish().expect("idempotent finish"),
      0,
      "an untouched XML-bearing entry must not re-store"
    );

    // unregister must delete the STORED row too (Perl ObjectDB.pm:183:
    // "else it'll get pulled back in!").
    let mut db = ObjectDB::attach(&dbfile, DbAttachOptions::default()).expect("attach 3");
    db.unregister("doc1#s1");
    db.finish().expect("finish after unregister");
    let db = ObjectDB::attach(&dbfile, DbAttachOptions::default()).expect("attach 4");
    assert!(
      db.lookup("doc1#s1").is_none(),
      "an unregistered key must not resurrect on re-attach"
    );
  }

  /// Perl `finish` stores only entries that DIFFER from what the file holds
  /// (ObjectDB.pm: `next if compare_hash(...)`) — re-finishing an unchanged
  /// DB writes nothing; touching one entry writes exactly one.
  #[test]
  fn dbfile_finish_writes_only_changes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dbfile = dir.path().join("site.db");
    let mut db = ObjectDB::attach(&dbfile, DbAttachOptions::default()).expect("attach");
    db.register("a", vec![("v", Value::Int(1))]);
    db.register("b", vec![("v", Value::Int(2))]);
    assert_eq!(db.finish().expect("first finish"), 2);

    let mut db = ObjectDB::attach(&dbfile, DbAttachOptions::default()).expect("re-attach");
    assert_eq!(
      db.finish().expect("no-op finish"),
      0,
      "unchanged DB stores nothing"
    );

    let mut db = ObjectDB::attach(&dbfile, DbAttachOptions::default()).expect("re-attach 2");
    db.register("b", vec![("v", Value::Int(99))]);
    db.register("c", vec![("v", Value::Int(3))]);
    assert_eq!(
      db.finish().expect("delta finish"),
      2,
      "one changed + one new"
    );
    let db = ObjectDB::attach(&dbfile, DbAttachOptions::default()).expect("re-attach 3");
    assert!(matches!(
      db.lookup("b").and_then(|e| e.get_value("v")),
      Some(Value::Int(99))
    ));
    assert_eq!(db.len(), 3);
  }

  /// The concurrency contract the layer exists for (parallel page-render
  /// workers): N readers attach the SAME file while a writer commits — WAL
  /// keeps readers unblocked on their snapshot — and a second writer rides
  /// out the first's transaction via busy_timeout instead of failing with
  /// SQLITE_BUSY. ObjectDB itself is !Send (libxml values), so each thread
  /// builds its OWN handle inside the thread — exactly the worker model.
  #[test]
  fn dbfile_concurrent_readers_and_writer_contention() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dbfile = dir.path().join("site.db");
    {
      let mut db = ObjectDB::attach(&dbfile, DbAttachOptions::default()).expect("seed");
      for i in 0..500 {
        db.register(&format!("k{i}"), vec![("v", Value::Int(i))]);
      }
      assert_eq!(db.finish().expect("seed finish"), 500);
    }

    // Writer-vs-writer: hold an IMMEDIATE transaction on a raw connection,
    // then finish() a second writer — busy_timeout must ride it out.
    let blocker = rusqlite::Connection::open(&dbfile).expect("blocker open");
    blocker
      .execute_batch("BEGIN IMMEDIATE; UPDATE entries SET props = props WHERE key = 'k0';")
      .expect("blocker tx");
    let release = std::thread::spawn(move || {
      std::thread::sleep(std::time::Duration::from_millis(300));
      blocker.execute_batch("COMMIT;").expect("blocker commit");
    });
    let mut writer = ObjectDB::attach(&dbfile, DbAttachOptions::default()).expect("writer");
    writer.register("k0", vec![("v", Value::Int(-1))]);
    let t0 = std::time::Instant::now();
    assert_eq!(
      writer.finish().expect("contended finish succeeds"),
      1,
      "the second writer stores its one change after the blocker commits"
    );
    assert!(
      t0.elapsed() < std::time::Duration::from_secs(9),
      "finish waited out the blocker, not the full timeout"
    );
    release.join().expect("blocker thread");

    // N concurrent readers, each with its own attach, racing a live writer.
    std::thread::scope(|scope| {
      let dbfile = &dbfile;
      let mut handles = Vec::new();
      for _ in 0..4 {
        handles.push(scope.spawn(move || {
          for _ in 0..10 {
            let db = ObjectDB::attach(dbfile, DbAttachOptions {
              readonly: true,
              ..Default::default()
            })
            .expect("reader attaches during writes");
            assert_eq!(db.len(), 500, "readers always see a full snapshot");
            assert!(db.lookup("k42").is_some());
          }
        }));
      }
      scope.spawn(move || {
        for round in 0..10 {
          let mut db =
            ObjectDB::attach(dbfile, DbAttachOptions::default()).expect("writer attaches");
          db.register("k1", vec![("v", Value::Int(1000 + round))]);
          db.finish().expect("interleaved writer finish");
        }
      });
      for h in handles {
        h.join().expect("reader thread");
      }
    });
  }

  /// Staleness contract: a format-version mismatch REFUSES the file with a
  /// named error — never a silent reinterpretation.
  #[test]
  fn dbfile_version_mismatch_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dbfile = dir.path().join("site.db");
    {
      let conn = rusqlite::Connection::open(&dbfile).expect("open");
      conn
        .execute_batch("PRAGMA user_version = 999;")
        .expect("stamp");
    }
    let err = ObjectDB::attach(&dbfile, DbAttachOptions::default())
      .err()
      .expect("mismatched version must refuse");
    assert!(
      err.contains("format version 999"),
      "names the found version: {err}"
    );
    // `clean` is the sanctioned rebuild path (Perl `clean => 1`).
    let db = ObjectDB::attach(&dbfile, DbAttachOptions {
      clean: true,
      ..Default::default()
    })
    .expect("clean rebuild attaches");
    assert!(db.is_empty());
  }
}

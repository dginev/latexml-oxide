//! Index generation processor.
//!
//! Port of `LaTeXML::Post::MakeIndex` (504 lines of Perl).
//! Collects INDEX:* entries from the ObjectDB, builds a tree of index entries
//! grouped by initial letter, and fills in `ltx:index` and `ltx:glossary` elements.
//! Supports permuted indexes, splitting by initial, see-also references,
//! range-style page references, and glossary entry formatting.

use libxml::tree::{Node, NodeType};
use rustc_hash::FxHashMap as HashMap;
use unicode_normalization::UnicodeNormalization;

use crate::document::{NodeData, PostDocument};
use crate::object_db::{ObjectDB, Value};
use crate::processor::{ProcessResult, Processor};

/// A see/see-also cross reference extracted from an `ltx:indexsee` node.
#[derive(Debug)]
struct SeeAlso {
  /// The cross-reference word from the `name` attribute ("see", "see also").
  name: Option<String>,
  /// Normalized text content, used to resolve the target entry.
  text: String,
  /// The original node, for re-rendering the phrase with its markup.
  node: Option<Node>,
}

/// One level of an index phrase, as collected by Scan: the sort/merge
/// key plus the original `ltx:indexphrase` node (when available) for
/// markup-preserving rendering.
#[derive(Debug, Clone)]
struct PhraseRef {
  key:  String,
  text: String,
  node: Option<Node>,
}

/// Index tree node.
#[derive(Debug)]
struct IndexTree {
  id:               String,
  key:              Option<String>,
  full_key:         Option<String>,
  phrase:           Option<String>,
  phrase_node:      Option<Node>,
  phrase_text:      Option<String>,
  full_phrase_text: Option<String>,
  subtrees:         HashMap<String, IndexTree>,
  referrers:        HashMap<String, HashMap<String, bool>>,
  see_also:         Vec<SeeAlso>,
}

impl IndexTree {
  fn new(id: &str) -> Self {
    IndexTree {
      id:               id.to_string(),
      key:              None,
      full_key:         None,
      phrase:           None,
      phrase_node:      None,
      phrase_text:      None,
      full_phrase_text: None,
      subtrees:         HashMap::default(),
      referrers:        HashMap::default(),
      see_also:         Vec::new(),
    }
  }
}

/// A glossary entry ready for rendering.
struct GlossaryEntry {
  initial:   String,
  sort_key:  String,
  formatted: NodeData,
}

/// MakeIndex post-processor.
///
/// Port of `LaTeXML::Post::MakeIndex`.
pub struct MakeIndex {
  name:     String,
  pub db:   ObjectDB,
  permuted: bool,
  split:    bool,
}

impl MakeIndex {
  pub fn new(db: ObjectDB, split: bool, permuted: bool) -> Self {
    MakeIndex {
      name: "MakeIndex".to_string(),
      db,
      split,
      permuted,
    }
  }

  /// Build the index tree from ObjectDB INDEX:* entries.
  ///
  /// Port of `MakeIndex::build_tree`.
  fn build_tree(&self, index_id: &str) -> Option<(IndexTree, HashMap<String, String>)> {
    let keys: Vec<String> = self
      .db
      .get_keys()
      .into_iter()
      .filter(|k| k.starts_with("INDEX:"))
      .cloned()
      .collect();
    if keys.is_empty() {
      return None;
    }
    Info!("make_index", "count", "MakeIndex: {} entries", keys.len());

    let mut all_phrases: HashMap<String, String> = HashMap::default();
    let mut tree = IndexTree::new(index_id);
    // Perl's final rescan runs every generated id through uniquifyID;
    // we dedup inline. Distinct phrases can strip to the same
    // alphanumeric key id (e.g. "probability density, ε-biased" with
    // and without the space → idx.probabilitydensityepsilonbiased),
    // which would otherwise emit a duplicate HTML id. Seeded with the
    // index root id so children can't collide with it.
    let mut used_ids: HashMap<String, u32> = HashMap::default();
    used_ids.insert(index_id.to_string(), 0);

    for key in &keys {
      if let Some(entry) = self.db.lookup(key) {
        // Perl drives the tree off the scanned ltx:indexphrase NODES
        // (`$entry->getValue('phrases')`): the `key` attribute is the
        // sort/merge key, the node itself re-renders the phrase with
        // its markup. Fall back to splitting the DB key for entries
        // scanned without nodes.
        let mut phrase_refs: Vec<PhraseRef> = Vec::new();
        if let Some(Value::List(phrase_nodes)) = entry.get_value("phrases") {
          for item in phrase_nodes {
            if let Value::Xml(node) = item {
              let nkey = node.get_attribute("key").unwrap_or_else(|| {
                get_index_content_key(&node.get_content())
              });
              phrase_refs.push(PhraseRef {
                key:  nkey,
                text: get_index_content_key(&node.get_content()),
                node: Some(node.clone()),
              });
            }
          }
        }
        if phrase_refs.is_empty() {
          phrase_refs = key
            .strip_prefix("INDEX:")
            .unwrap_or("")
            .split(':')
            .filter(|s| !s.is_empty())
            .map(|s| PhraseRef {
              key:  s.to_string(),
              text: s.to_string(),
              node: None,
            })
            .collect();
        }
        if phrase_refs.is_empty() {
          Warn!("expected", key, "Missing phrases in indexmark: '{}'", key);
          continue;
        }

        if self.permuted {
          // Cyclic permutations of phrase keys
          for perm in cyclic_permute(&phrase_refs) {
            if self.split {
              let init = initial_letter(&perm[0].key);
              let subtree = tree.subtrees.entry(init.clone()).or_insert_with(|| {
                let mut st = IndexTree::new(&tree.id);
                st.phrase = Some(init);
                st
              });
              add_tree_rec(subtree, &perm, &mut all_phrases, &mut used_ids, entry);
            } else {
              add_tree_rec(&mut tree, &perm, &mut all_phrases, &mut used_ids, entry);
            }
          }
        } else if self.split {
          let init = initial_letter(&phrase_refs[0].key);
          let subtree = tree.subtrees.entry(init.clone()).or_insert_with(|| {
            let mut st = IndexTree::new(&tree.id);
            st.phrase = Some(init);
            st
          });
          add_tree_rec(subtree, &phrase_refs, &mut all_phrases, &mut used_ids, entry);
        } else {
          add_tree_rec(&mut tree, &phrase_refs, &mut all_phrases, &mut used_ids, entry);
        }
      }
    }
    Some((tree, all_phrases))
  }

  /// Generate XML for an index list from a tree.
  ///
  /// `ancestor_phrases` carries the full_phrase_text of each enclosing
  /// entry (outermost first) — Perl reaches the same context by walking
  /// the tree's parent pointers in `lookupSeealsoPhrase`.
  fn make_index_list(
    &self,
    all_phrases: &HashMap<String, String>,
    tree: &IndexTree,
    ancestor_phrases: &[String],
  ) -> Option<NodeData> {
    if tree.subtrees.is_empty() {
      return None;
    }
    let mut sorted_keys: Vec<&String> = tree.subtrees.keys().collect();
    sorted_keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()).then(a.cmp(b)));

    let entries: Vec<NodeData> = sorted_keys
      .iter()
      .filter_map(|key| {
        tree
          .subtrees
          .get(*key)
          .map(|st| self.make_index_entry(all_phrases, st, ancestor_phrases))
      })
      .collect();
    if entries.is_empty() {
      return None;
    }
    Some(NodeData::Element {
      tag:        "ltx:indexlist".to_string(),
      attributes: None,
      children:   entries,
    })
  }

  /// Generate a single index entry with sub-entries, references, and see-also.
  ///
  /// Port of `MakeIndex::makeIndexEntry`.
  fn make_index_entry(
    &self,
    all_phrases: &HashMap<String, String>,
    tree: &IndexTree,
    ancestor_phrases: &[String],
  ) -> NodeData {
    let mut children = Vec::new();

    // Phrase: re-render the scanned ltx:indexphrase node when we have
    // one (math and styled text inside the phrase survive); plain text
    // otherwise. Perl: `$doc->trimChildNodes($$tree{phrase})`.
    if tree.phrase.is_some() || tree.phrase_node.is_some() {
      let phrase_children: Vec<NodeData> = match tree.phrase_node {
        Some(ref n) => trimmed_child_nodes(n),
        None => vec![NodeData::Text(tree.phrase.clone().unwrap_or_default())],
      };
      children.push(NodeData::Element {
        tag:        "ltx:indexphrase".to_string(),
        attributes: tree
          .key
          .as_ref()
          .map(|k| HashMap::from_iter([("key".to_string(), k.clone())])),
        children:   phrase_children,
      });
    }

    // Referrer links (combined with range handling). Perl prefixes
    // them with a one-space ltx:text separating phrase from refs.
    let mut links = Vec::new();
    if !tree.referrers.is_empty() {
      links.push(NodeData::Element {
        tag:        "ltx:text".to_string(),
        attributes: None,
        children:   vec![NodeData::Text(" ".to_string())],
      });
      links.extend(self.combine_index_entries(&tree.referrers));
    }

    // See/see-also cross references (Perl makeIndexEntry): each gets
    // a ", " separator, the italic name word ("see", "see also") when
    // present, and the phrase — resolved to a linked target entry via
    // seealsoSearch when possible, with its original markup.
    //
    // Lookup context, innermost first (Perl walks parent pointers):
    // this entry's full phrase, each ancestor's, then bare top-level.
    let mut see_context: Vec<String> = Vec::new();
    if let Some(ref fpt) = tree.full_phrase_text {
      see_context.push(fpt.clone());
    }
    see_context.extend(ancestor_phrases.iter().rev().cloned());
    see_context.push(String::new());

    for see in &tree.see_also {
      links.push(NodeData::Text(", ".to_string()));
      if let Some(ref name) = see.name {
        if !name.is_empty() {
          links.push(NodeData::Element {
            tag:        "ltx:text".to_string(),
            attributes: Some(HashMap::from_iter([(
              "font".to_string(),
              "italic".to_string(),
            )])),
            children:   vec![NodeData::Text(format!("{} ", name))],
          });
        }
      }
      let parts: Vec<SeeChunk> = match see.node {
        Some(ref n) => seealso_partition(n),
        None => vec![SeeChunk {
          key: see.text.clone(),
          xml: vec![NodeData::Text(see.text.clone())],
        }],
      };
      if let Some(see_links) = seealso_search_rec(&parts, all_phrases, &see_context) {
        links.extend(see_links);
      } else {
        // Perl warns unless the see phrase already contains refs of
        // its own, then falls back to the phrase's own markup.
        let already_linked = see
          .node
          .as_ref()
          .map(node_has_ref_descendant)
          .unwrap_or(false);
        if !already_linked {
          Warn!(
            "expected", &see.text,
            "Missing index see-also term '{}' (seen under {})",
            see.text,
            tree.full_key.as_deref().unwrap_or("?")
          );
        }
        let content: Vec<NodeData> = match see.node {
          Some(ref n) => n.get_child_nodes().into_iter().map(NodeData::XmlNode).collect(),
          None => vec![NodeData::Text(see.text.clone())],
        };
        links.push(NodeData::Element {
          tag:        "ltx:text".to_string(),
          attributes: None,
          children:   content,
        });
      }
    }

    if !links.is_empty() {
      children.push(NodeData::Element {
        tag:        "ltx:indexrefs".to_string(),
        attributes: None,
        children:   links,
      });
    }

    // Sub-entries: this entry's full phrase joins the ancestor context.
    let mut child_ancestors: Vec<String> = ancestor_phrases.to_vec();
    if let Some(ref fpt) = tree.full_phrase_text {
      child_ancestors.push(fpt.clone());
    }
    if let Some(sublist) = self.make_index_list(all_phrases, tree, &child_ancestors) {
      children.push(sublist);
    }

    NodeData::Element {
      tag: "ltx:indexentry".to_string(),
      attributes: if tree.id.is_empty() {
        None
      } else {
        // fragid mirrors xml:id so the XSLT add_id template emits an
        // HTML id for the <li> — same workaround as glossaryentry:
        // these nodes are created after Scan ran, so CrossRef's
        // fill_in_frags has no DB entry to source fragid from.
        Some(HashMap::from_iter([
          ("xml:id".to_string(), tree.id.clone()),
          ("fragid".to_string(), tree.id.clone()),
        ]))
      },
      children,
    }
  }

  /// Register an `ID:` entry for every index entry just created, so
  /// CrossRef can resolve the see/see-also `ltx:ref idref=idx.*` links
  /// to hrefs. Perl gets this for free from `$self->rescan($doc)` at
  /// the end of MakeIndex::process; we register the minimum CrossRef
  /// needs (location + fragid) directly.
  fn register_entry_ids(&mut self, tree: &IndexTree, location: &str) {
    for subtree in tree.subtrees.values() {
      if !subtree.id.is_empty() {
        let entry = self.db.register(&format!("ID:{}", subtree.id), vec![]);
        entry.set_value("location", Value::from(location));
        entry.set_value("fragid", Value::from(subtree.id.as_str()));
        entry.set_value("id", Value::from(subtree.id.as_str()));
        // Parent chain feeds CrossRef's generate_title context walk
        // (a see-ref's tooltip becomes the enclosing index's title,
        // as with Perl's rescan-built entries).
        if !tree.id.is_empty() {
          entry.set_value("parent", Value::from(tree.id.as_str()));
        }
      }
      self.register_entry_ids(subtree, location);
    }
  }

  /// Combine index entry referrers into a comma-separated list with range support.
  ///
  /// Port of `combineIndexEntries`.
  fn combine_index_entries(&self, refs: &HashMap<String, HashMap<String, bool>>) -> Vec<NodeData> {
    let mut ids: Vec<&String> = refs.keys().collect();
    ids.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()).then(a.cmp(b)));

    let mut links = Vec::new();
    let mut i = 0;
    while i < ids.len() {
      if !links.is_empty() {
        links.push(NodeData::Text(", ".to_string()));
      }
      let id = ids[i];
      let styles = refs.get(id).unwrap();

      // Check for range start. Perl pairs the start with the NEXT
      // range marker of either kind ($lvl-- for rangestart AND
      // rangeend): since ids are sorted alphabetically rather than
      // by document order, no real nesting is possible, and a stray
      // second start terminates the range rather than extending it.
      if styles.contains_key("rangestart") {
        let start_id = id;
        let mut end_id = id;
        let mut level = 1i32;
        i += 1;
        while i < ids.len() && level > 0 {
          end_id = ids[i];
          if let Some(s) = refs.get(end_id) {
            if s.contains_key("rangestart") {
              level -= 1;
            }
            if s.contains_key("rangeend") {
              level -= 1;
            }
          }
          i += 1;
        }
        // Range: start–end
        links.push(NodeData::Element {
          tag:        "ltx:text".to_string(),
          attributes: None,
          children:   vec![
            self.make_index_ref(start_id, styles),
            NodeData::Text("\u{2014}".to_string()), // em-dash
            self.make_index_ref(end_id, refs.get(end_id).unwrap_or(styles)),
          ],
        });
      } else {
        links.push(self.make_index_ref(id, styles));
        i += 1;
      }
    }
    links
  }

  /// Make a single index reference link.
  ///
  /// Port of `makeIndexRefs`.
  fn make_index_ref(&self, id: &str, styles: &HashMap<String, bool>) -> NodeData {
    // Perl: `sort keys %$entry` then take the first — "sorted styles
    // gives bold, italic, normal; let's just do the first". The sort
    // is what makes the pick deterministic (and prefers bold).
    let mut style: Vec<&String> = styles
      .keys()
      .filter(|s| *s != "rangestart" && *s != "rangeend")
      .collect();
    style.sort();
    let primary_style = style.first().map(|s| s.as_str()).unwrap_or("normal");

    let ref_node = NodeData::Element {
      tag:        "ltx:ref".to_string(),
      attributes: Some(HashMap::from_iter([
        ("idref".to_string(), id.to_string()),
        ("show".to_string(), "typerefnum".to_string()),
      ])),
      children:   vec![],
    };

    if primary_style != "normal" {
      NodeData::Element {
        tag:        "ltx:text".to_string(),
        attributes: Some(HashMap::from_iter([(
          "font".to_string(),
          primary_style.to_string(),
        )])),
        children:   vec![ref_node],
      }
    } else {
      ref_node
    }
  }

  /// Get glossary entries from the ObjectDB.
  ///
  /// Port of `MakeIndex::getGlossaryEntries`.
  fn get_glossary_entries(&mut self, lists: &str, glossary_id: &str) -> Vec<GlossaryEntry> {
    let list_set: rustc_hash::FxHashSet<&str> = lists.split(',').collect();
    let mut entries = Vec::new();

    // Clone the keys up-front so we can do mutable `lookup_mut` writes
    // inside the loop (registering `id` so CrossRef can resolve refs).
    let keys: Vec<String> = self.db.get_keys().into_iter().cloned().collect();
    for db_key in &keys {
      let db_key = db_key.as_str();
      if !db_key.starts_with("GLOSSARY:") {
        continue;
      }
      let parts: Vec<&str> = db_key.splitn(3, ':').collect();
      if parts.len() < 3 {
        continue;
      }
      let list = parts[1];
      let key = parts[2];
      if !list_set.contains(list) {
        continue;
      }

      if let Some(entry) = self.db.lookup(db_key) {
        // Check if it has referrers
        let has_refs = entry
          .get_value("referrers")
          .map(|v| v.is_truthy())
          .unwrap_or(false);
        if !has_refs {
          continue;
        }

        let sort_key = entry.get_string("phrase:sort").unwrap_or(key).to_string();
        let initial = sort_key
          .chars()
          .next()
          .filter(|c| c.is_ascii_alphabetic())
          .map(|c| c.to_uppercase().to_string())
          .unwrap_or_else(|| "*".to_string());
        let id = format!("{}.{}", glossary_id, key);
        let term = entry.get_string("phrase:name").unwrap_or(key).to_string();
        let desc = entry
          .get_string("phrase:description")
          .unwrap_or("")
          .to_string();

        entries.push(GlossaryEntry {
          initial,
          sort_key,
          formatted: NodeData::Element {
            tag:        "ltx:glossaryentry".to_string(),
            attributes: Some(HashMap::from_iter([
              ("lists".to_string(), lists.to_string()),
              ("xml:id".to_string(), id.clone()),
              // fragid mirrors xml:id so the XSLT `add_id` template emits
              // `<dt id="glo.main.cabbage">`. Without fragid, the dt
              // renders as `<dt class="..."/>` only. Normally CrossRef's
              // fill_in_frags populates fragid from the ObjectDB ID entry,
              // but our new glossaryentry nodes are created AFTER Scan
              // already ran, so the DB has no `ID:<id>` entry to source
              // from. Setting fragid eagerly here matches the end state.
              ("fragid".to_string(), id.clone()),
              ("key".to_string(), key.to_string()),
            ])),
            children:   vec![
              NodeData::Element {
                tag:        "ltx:glossaryphrase".to_string(),
                attributes: Some(HashMap::from_iter([
                  ("role".to_string(), "label".to_string()),
                  ("key".to_string(), key.to_string()),
                ])),
                children:   vec![NodeData::Text(term)],
              },
              NodeData::Element {
                tag:        "ltx:glossaryphrase".to_string(),
                attributes: Some(HashMap::from_iter([(
                  "role".to_string(),
                  "definition".to_string(),
                )])),
                children:   vec![NodeData::Text(desc)],
              },
            ],
          },
        });
        // Register the entry's id back in the DB so CrossRef's
        // fill_in_glossaryrefs can resolve `<glossaryref key="X">` to
        // the corresponding `<glossaryentry xml:id=…>`. Mirrors Perl
        // MakeIndex.pm where the entry construction sets `id`.
        if let Some(entry_mut) = self.db.lookup_mut(db_key) {
          entry_mut.set_value("id", Value::from(id.clone()));
        }
      }
    }
    // Perl `MakeIndex.pm` L487: `$doc->unisort(keys %hash)` — Unicode-
    // aware case-insensitive sort. For ASCII-Latin (which covers the
    // test fixture and the vast majority of glossary entries), lowercase
    // comparison matches the expected order: "Cabbage" sorts beside
    // "cabbage" rather than before all lowercase letters.
    entries.sort_by_key(|a| a.sort_key.to_lowercase());
    entries
  }

  /// Generate a glossary list.
  ///
  /// Port of `MakeIndex::makeGlossaryList`.
  fn make_glossary_list(&self, entries: &[GlossaryEntry]) -> NodeData {
    NodeData::Element {
      tag:        "ltx:glossarylist".to_string(),
      attributes: None,
      children:   entries.iter().map(|e| e.formatted.clone()).collect(),
    }
  }
}

impl Processor for MakeIndex {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    doc.findnodes("//ltx:index[not(ltx:indexlist)] | //ltx:glossary[not(ltx:glossarylist)]")
  }

  fn process(&mut self, mut doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    for node in &nodes {
      let tag = doc.get_qname(node).unwrap_or_default();
      // xml:id is namespace-bound: plain get_attribute misses it on
      // engine-constructed nodes (it only happens to work on nodes
      // round-tripped through serialization).
      let id = crate::document::get_xml_id(node).unwrap_or_default();

      if tag == "ltx:index" {
        if let Some((tree, all_phrases)) = self.build_tree(&id) {
          if let Some(index_list) = self.make_index_list(&all_phrases, &tree, &[]) {
            let mut node_mut = node.clone();
            doc.add_nodes(&mut node_mut, &[index_list]);
            let location = doc.site_relative_destination().unwrap_or_default();
            self.register_entry_ids(&tree, &location);
          }
        }
      } else if tag == "ltx:glossary" {
        let lists = node
          .get_attribute("lists")
          .unwrap_or_else(|| "glossary".to_string());
        let entries = self.get_glossary_entries(&lists, &id);
        if !entries.is_empty() {
          let glist = self.make_glossary_list(&entries);
          let mut node_mut = node.clone();
          doc.add_nodes(&mut node_mut, &[glist]);
        }
      }
    }
    Ok(vec![doc])
  }
}

// ======================================================================
// Helpers

fn add_tree_rec(
  tree: &mut IndexTree,
  phrases: &[PhraseRef],
  all_phrases: &mut HashMap<String, String>,
  used_ids: &mut HashMap<String, u32>,
  entry: &crate::object_db::Entry,
) {
  if phrases.is_empty() {
    // Leaf: record referrers and see_also.
    // Scan's note_association(["referrers", pid, style]) nests THREE
    // levels: referrers → {pid → {style → true}} (mirroring Perl's
    // $$entry{referrers}{$id}{$style}). The inner value is a Hash of
    // styles, not a scalar — reading it with to_string() used to
    // collapse every style to "" (Value's Display renders hashes as
    // the empty string), silently dropping bold/italic/etc. and
    // wrapping every index ref in an empty-font ltx:text.
    if let Some(Value::Hash(refs)) = entry.get_value("referrers") {
      for (k, v) in refs {
        let styles = tree.referrers.entry(k.clone()).or_default();
        if let Value::Hash(style_map) = v {
          for style in style_map.keys() {
            styles.insert(style.clone(), true);
          }
        }
      }
    }
    if let Some(Value::List(see_items)) = entry.get_value("see_also") {
      for item in see_items {
        // Stored as Value::Xml(ltx:indexsee) by Scan; keep the node
        // for markup-preserving rendering, plus its name attribute
        // ("see"/"see also") and normalized text for target lookup.
        if let Value::Xml(node) = item {
          tree.see_also.push(SeeAlso {
            name: node.get_attribute("name"),
            text: get_index_content_key(&node.get_content()),
            node: Some(node.clone()),
          });
        } else {
          tree.see_also.push(SeeAlso {
            name: None,
            text: get_index_content_key(&item.to_string()),
            node: None,
          });
        }
      }
    }
    return;
  }
  let phrase = &phrases[0];
  let rest = &phrases[1..];
  let key = &phrase.key;
  let key_id = get_index_key_id(key);
  let parent_key = tree.full_key.clone().unwrap_or_default();
  let full_key = if parent_key.is_empty() {
    key.to_string()
  } else {
    format!("{}.{}", parent_key, key)
  };
  // Perl: phrasetext (the normalized CONTENT, not the key) feeds the
  // see/see-also lookup table; multi-level phrases join with a space.
  let phrase_text = phrase.text.clone();
  let parent_phrase = tree.full_phrase_text.clone().unwrap_or_default();
  let full_phrase = if parent_phrase.is_empty() {
    phrase_text.clone()
  } else {
    format!("{} {}", parent_phrase, phrase_text)
  };

  let tree_id = tree.id.clone();
  // Allocate the id BEFORE or_insert_with so we can uniquify against
  // ids already used by sibling phrases that strip to the same
  // key_id. Only done when the subtree is new (an existing subtree
  // keeps its id). Perl: uniquifyID appends radix_alpha(clashes).
  let needs_id = !tree.subtrees.contains_key(key.as_str());
  let id = if needs_id {
    Some(uniquify_id(&format!("{}.{}", tree_id, key_id), used_ids))
  } else {
    None
  };
  let subtree = tree.subtrees.entry(key.to_string()).or_insert_with(|| {
    let id = id.unwrap();
    all_phrases.insert(full_key.clone(), id.clone());
    all_phrases.insert(full_key.to_lowercase(), id.clone());
    all_phrases.insert(full_phrase.clone(), id.clone());
    all_phrases.insert(full_phrase.to_lowercase(), id.clone());
    let mut st = IndexTree::new(&id);
    st.key = Some(key.to_string());
    st.full_key = Some(full_key.clone());
    st.phrase = Some(phrase_text.clone());
    st.phrase_node = phrase.node.clone();
    st.phrase_text = Some(phrase_text);
    st.full_phrase_text = Some(full_phrase);
    st
  });
  add_tree_rec(subtree, rest, all_phrases, used_ids, entry);
}

/// Perl Post::uniquifyID: return `base` if unused, else append
/// radix_alpha(n) (a, b, …) for the next free clash counter.
fn uniquify_id(base: &str, used: &mut HashMap<String, u32>) -> String {
  if !used.contains_key(base) {
    used.insert(base.to_string(), 0);
    return base.to_string();
  }
  loop {
    let n = used.get_mut(base).unwrap();
    *n += 1;
    let candidate = format!("{}{}", base, crate::radix::radix_alpha(*n));
    if !used.contains_key(&candidate) {
      used.insert(candidate.clone(), 0);
      return candidate;
    }
  }
}

fn initial_letter(key: &str) -> String {
  let decomposed: String = key.nfd().collect();
  match decomposed.trim().chars().next() {
    Some(c) if c.is_ascii_alphabetic() => c.to_uppercase().to_string(),
    _ => "*".to_string(),
  }
}

/// Perl getIndexKeyID: NFD-decompose, transliterate Greek letters to
/// their names (so math-bearing keys don't collapse to nothing), then
/// strip everything but ASCII alphanumerics.
fn get_index_key_id(key: &str) -> String {
  let decomposed: String = key.trim().nfd().collect();
  let mut out = String::with_capacity(decomposed.len());
  for c in decomposed.chars() {
    if let Some(name) = greek_ascii(c) {
      out.push_str(name);
    } else if c.is_ascii_alphanumeric() {
      out.push(c);
    }
  }
  out
}

/// Perl %GREEK_ASCII_MAP.
fn greek_ascii(c: char) -> Option<&'static str> {
  Some(match c {
    '\u{03B1}' => "alpha",
    '\u{03B2}' => "beta",
    '\u{03B3}' => "gamma",
    '\u{03B4}' => "delta",
    '\u{03F5}' => "epsilon",
    '\u{03B5}' => "varepsilon",
    '\u{03B6}' => "zeta",
    '\u{03B7}' => "eta",
    '\u{03B8}' => "theta",
    '\u{03D1}' => "vartheta",
    '\u{03B9}' => "iota",
    '\u{03BA}' => "kappa",
    '\u{03BB}' => "lambda",
    '\u{03BC}' => "mu",
    '\u{03BD}' => "nu",
    '\u{03BE}' => "xi",
    '\u{03C0}' => "pi",
    '\u{03D6}' => "varpi",
    '\u{03C1}' => "rho",
    '\u{03F1}' => "varrho",
    '\u{03C3}' => "sigma",
    '\u{03C2}' => "varsigma",
    '\u{03C4}' => "tau",
    '\u{03C5}' => "upsilon",
    '\u{03D5}' => "phi",
    '\u{03C6}' => "varphi",
    '\u{03C7}' => "chi",
    '\u{03C8}' => "psi",
    '\u{03C9}' => "omega",
    '\u{0393}' => "Gamma",
    '\u{0394}' => "Delta",
    '\u{0398}' => "Theta",
    '\u{039B}' => "Lambda",
    '\u{039E}' => "Xi",
    '\u{03A0}' => "Pi",
    '\u{03A3}' => "Sigma",
    '\u{03A5}' => "Upsilon",
    '\u{03A6}' => "Phi",
    '\u{03A8}' => "Psi",
    '\u{03A9}' => "Omega",
    _ => return None,
  })
}

/// Perl getIndexContentKey: trim, collapse internal whitespace,
/// strip trailing punctuation.
fn get_index_content_key(s: &str) -> String {
  let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
  collapsed
    .trim_end_matches(['.', ',', ';'])
    .trim_end()
    .to_string()
}

/// Perl `$doc->trimChildNodes`: the node's children minus pure-whitespace
/// text nodes at either end (whitespace INSIDE the sequence is kept).
fn trimmed_child_nodes(node: &Node) -> Vec<NodeData> {
  let children = node.get_child_nodes();
  let is_ws = |n: &Node| {
    n.get_type() == Some(NodeType::TextNode) && n.get_content().trim().is_empty()
  };
  let start = children.iter().position(|n| !is_ws(n));
  let end = children.iter().rposition(|n| !is_ws(n));
  match (start, end) {
    (Some(s), Some(e)) => children[s..=e].iter().cloned().map(NodeData::XmlNode).collect(),
    _ => vec![],
  }
}

/// Does the node contain an ltx:ref descendant? (Perl checks
/// `descendant-or-self::ltx:ref` before warning about an unresolved
/// see-also phrase.)
fn node_has_ref_descendant(node: &Node) -> bool {
  if node.get_type() == Some(NodeType::ElementNode) && node.get_name() == "ref" {
    return true;
  }
  node.get_child_nodes().iter().any(node_has_ref_descendant)
}

// ======================================================================
// See & see-also resolution (Perl MakeIndex.pm "A LOTTA work, for such
// a little thing!"). A see phrase is partitioned into alternating
// candidate-term and candidate-delimiter chunks; the search then tries
// interpreting each delimiter as part of a longer phrase or as a
// separator between independent targets.

/// One partition chunk: the normalized lookup key plus the XML pieces
/// it came from (used to fill the resulting ltx:ref).
#[derive(Debug, Clone)]
struct SeeChunk {
  key: String,
  xml: Vec<NodeData>,
}

/// Perl seealsoPartition: chunk the phrase, then (pass 1) combine
/// adjacent chunks that are both-or-neither delimiters, and (pass 2)
/// fold pure-space chunks into their neighbours.
fn seealso_partition(see: &Node) -> Vec<SeeChunk> {
  let parts = seealso_partition_aux(see);
  if parts.is_empty() {
    return parts;
  }
  // Pass 1: combine adjacent conjunction/punctuation chunks.
  let mut iter = parts.into_iter();
  let mut result: Vec<SeeChunk> = vec![iter.next().unwrap()];
  for next in iter {
    let prev_is = is_delimiter_key(&result.last().unwrap().key);
    let next_is = starts_with_delimiter(&next.key);
    if prev_is == next_is {
      let last = result.last_mut().unwrap();
      last.key.push_str(&next.key);
      last.xml.extend(next.xml);
    } else {
      result.push(next);
    }
  }
  // Pass 2: a pure-space chunk merges its neighbours into one phrase.
  let mut iter = result.into_iter();
  let mut merged: Vec<SeeChunk> = vec![iter.next().unwrap()];
  let mut rest: Vec<SeeChunk> = iter.collect();
  let mut i = 0;
  while i < rest.len() {
    let next = rest[i].clone();
    if !next.key.is_empty() && next.key.trim().is_empty() && i + 1 < rest.len() {
      let after = rest[i + 1].clone();
      let last = merged.last_mut().unwrap();
      last.key.push_str(&next.key);
      last.key.push_str(&after.key);
      last.xml.extend(next.xml);
      last.xml.extend(after.xml);
      i += 2;
    } else {
      merged.push(next);
      i += 1;
    }
  }
  merged
}

/// Perl seealsoPartition_aux: split into pure phrase/delimiter chunks.
fn seealso_partition_aux(node: &Node) -> Vec<SeeChunk> {
  let mut result = Vec::new();
  for ch in node.get_child_nodes() {
    match ch.get_type() {
      Some(NodeType::TextNode) => {
        let mut s = ch.get_content();
        while !s.is_empty() {
          if let Some((delim, rest)) = take_delimiter(&s) {
            result.push(SeeChunk {
              key: delim.clone(),
              xml: vec![NodeData::Text(delim)],
            });
            s = rest;
          } else {
            // ^([^,\.\s]+)
            let end = s
              .find(|c: char| c == ',' || c == '.' || c.is_whitespace())
              .unwrap_or(s.len());
            let (tok, rest) = s.split_at(end);
            result.push(SeeChunk {
              key: get_index_content_key(tok),
              xml: vec![NodeData::Text(tok.to_string())],
            });
            s = rest.to_string();
          }
        }
      },
      Some(NodeType::ElementNode) => {
        let name = ch.get_name();
        if name == "text" || name == "emph" {
          // Recurse, re-wrapping each sub-chunk in the styling element
          // so delimiters can split styled phrases (Perl does the same).
          let attrs: HashMap<String, String> = ch.get_properties().into_iter().collect();
          for sub in seealso_partition_aux(&ch) {
            result.push(SeeChunk {
              key: sub.key,
              xml: vec![NodeData::Element {
                tag:        format!("ltx:{}", name),
                attributes: if attrs.is_empty() { None } else { Some(attrs.clone()) },
                children:   sub.xml,
              }],
            });
          }
        } else {
          // Opaque element (math etc.): one phrase chunk.
          result.push(SeeChunk {
            key: get_index_content_key(&ch.get_content()),
            xml: vec![NodeData::XmlNode(ch.clone())],
          });
        }
      },
      _ => {},
    }
  }
  result
}

/// Leading delimiter per Perl: `^(,|\.|\s+|and\s+also\b|and\b|or\b)`.
fn take_delimiter(s: &str) -> Option<(String, String)> {
  if s.starts_with(',') || s.starts_with('.') {
    return Some((s[..1].to_string(), s[1..].to_string()));
  }
  let ws_len = s.len() - s.trim_start().len();
  if ws_len > 0 {
    return Some((s[..ws_len].to_string(), s[ws_len..].to_string()));
  }
  for kw in ["and also", "and", "or"] {
    if let Some(rest) = strip_keyword(s, kw) {
      return Some((s[..s.len() - rest.len()].to_string(), rest.to_string()));
    }
  }
  None
}

/// Match a keyword (with internal `\s+` flexibility for "and also")
/// at the start of `s`, requiring a word boundary after it.
fn strip_keyword<'a>(s: &'a str, kw: &str) -> Option<&'a str> {
  let mut rest = s;
  for (i, word) in kw.split(' ').enumerate() {
    if i > 0 {
      let trimmed = rest.trim_start();
      if trimmed.len() == rest.len() {
        return None; // needed \s+ between words
      }
      rest = trimmed;
    }
    rest = rest.strip_prefix(word)?;
  }
  // \b: next char must not be a word character
  match rest.chars().next() {
    Some(c) if c.is_alphanumeric() || c == '_' => None,
    _ => Some(rest),
  }
}

/// Is this chunk key delimiter-shaped?
/// Perl: `^,?\s*(?:,|\.|\s+|\band\s+also|\band|\bor)\s*$`.
fn is_delimiter_key(key: &str) -> bool {
  let k = key.strip_prefix(',').unwrap_or(key).trim();
  if k.is_empty() || k == "," || k == "." || k == "and" || k == "or" {
    return true;
  }
  let words: Vec<&str> = k.split_whitespace().collect();
  words == ["and", "also"]
}

/// Does this chunk key START with a delimiter?
/// Perl: `^(?:,|\.|\s+|and\b|or\b)`.
fn starts_with_delimiter(key: &str) -> bool {
  key.starts_with(',')
    || key.starts_with('.')
    || key.starts_with(|c: char| c.is_whitespace())
    || strip_keyword(key, "and").is_some()
    || strip_keyword(key, "or").is_some()
}

/// Perl seealsoJoin: reassemble consecutive chunks into one candidate.
fn seealso_join(parts: &[SeeChunk]) -> SeeChunk {
  let key = get_index_content_key(&parts.iter().map(|p| p.key.as_str()).collect::<String>());
  let xml = parts.iter().flat_map(|p| p.xml.clone()).collect();
  SeeChunk { key, xml }
}

/// Perl seealsoSearch_rec: parts alternate (potential) term and
/// (potential) delimiter. Try each delimiter first as phrase-internal
/// (joining its neighbours), then as a separator between targets.
fn seealso_search_rec(
  parts: &[SeeChunk],
  all_phrases: &HashMap<String, String>,
  context: &[String],
) -> Option<Vec<NodeData>> {
  if parts.is_empty() {
    return None;
  }
  if parts.len() < 3 {
    // Single term (with possible trailing punctuation): just look it up.
    let link = lookup_seealso_phrase(&parts[0], all_phrases, context)?;
    let mut out = vec![link];
    if parts.len() > 1 {
      out.extend(parts[1].xml.clone());
    }
    return Some(out);
  }
  // Try the first delimiter "literally" (term+delim+term as one phrase).
  let mut joined: Vec<SeeChunk> = vec![seealso_join(&parts[0..3])];
  joined.extend_from_slice(&parts[3..]);
  if let Some(links) = seealso_search_rec(&joined, all_phrases, context) {
    return Some(links);
  }
  // Try the delimiter as a separator between individual entries.
  if let Some(link) = lookup_seealso_phrase(&parts[0], all_phrases, context) {
    if let Some(rest) = seealso_search_rec(&parts[2..], all_phrases, context) {
      let mut out = vec![link];
      out.extend(parts[1].xml.clone());
      out.extend(rest);
      return Some(out);
    }
  }
  None
}

/// Perl lookupSeealsoPhrase: try the phrase as-is, ignoring commas,
/// plurals, case, or treating commas as level separators — first within
/// the current entry's context (innermost first), then at top level.
fn lookup_seealso_phrase(
  chunk: &SeeChunk,
  all_phrases: &HashMap<String, String>,
  context: &[String],
) -> Option<NodeData> {
  let phrase = chunk.key.trim().to_string();
  if phrase.is_empty() {
    return None;
  }
  let pnc = comma_to_space(&phrase);
  let ps = strip_plurals(&phrase);
  let psnc = comma_to_space(&ps);
  let pnlvl = comma_to(&phrase, ".");
  let mut trials: Vec<String> = Vec::new();
  for t in [&phrase, &pnc, &ps, &psnc, &pnlvl] {
    trials.push((*t).clone());
    trials.push(t.to_lowercase());
  }
  for trial in &trials {
    for prefix in context {
      let lookup_key = if prefix.is_empty() {
        trial.clone()
      } else {
        format!("{} {}", prefix, trial)
      };
      if let Some(id) = all_phrases.get(&lookup_key) {
        return Some(NodeData::Element {
          tag:        "ltx:ref".to_string(),
          attributes: Some(HashMap::from_iter([("idref".to_string(), id.clone())])),
          children:   chunk.xml.clone(),
        });
      }
    }
  }
  None
}

/// `s/,\s*/ /g`
fn comma_to_space(s: &str) -> String { comma_to(s, " ") }

fn comma_to(s: &str, repl: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == ',' {
      out.push_str(repl);
      while chars.peek().is_some_and(|c| c.is_whitespace()) {
        chars.next();
      }
    } else {
      out.push(c);
    }
  }
  out
}

/// `s/(\w+)s\b/$1/g` — strip a trailing "s" from every word.
fn strip_plurals(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let mut word = String::new();
  for c in s.chars() {
    if c.is_alphanumeric() || c == '_' {
      word.push(c);
    } else {
      push_depluraled(&mut out, &word);
      word.clear();
      out.push(c);
    }
  }
  push_depluraled(&mut out, &word);
  out
}

fn push_depluraled(out: &mut String, word: &str) {
  if word.len() > 1 && word.ends_with('s') {
    out.push_str(&word[..word.len() - 1]);
  } else {
    out.push_str(word);
  }
}

/// Generate cyclic permutations of a slice.
fn cyclic_permute<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
  if items.len() <= 1 {
    return vec![items.to_vec()];
  }
  (0..items.len())
    .map(|i| {
      let mut perm = items[i..].to_vec();
      perm.extend_from_slice(&items[..i]);
      perm
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn initial_letter_ascii_uppercases() {
    assert_eq!(initial_letter("alpha"), "A");
    assert_eq!(initial_letter("Zebra"), "Z");
  }

  #[test]
  fn initial_letter_skips_leading_whitespace() {
    assert_eq!(initial_letter("  beta"), "B");
  }

  #[test]
  fn initial_letter_nfd_decomposes_accents() {
    // NFD decomposes 'É' (U+00C9) into 'E' + combining accent — the first
    // char is then ASCII 'E'.
    assert_eq!(initial_letter("Éclair"), "E");
    assert_eq!(initial_letter("über"), "U");
  }

  #[test]
  fn initial_letter_non_alpha_is_star() {
    assert_eq!(initial_letter("123abc"), "*");
    assert_eq!(initial_letter("#hash"), "*");
    assert_eq!(initial_letter(""), "*");
    assert_eq!(initial_letter("   "), "*");
  }

  #[test]
  fn get_index_key_id_strips_non_alphanumeric() {
    assert_eq!(get_index_key_id("Foo Bar!"), "FooBar");
    assert_eq!(get_index_key_id("abc-123"), "abc123");
  }

  #[test]
  fn get_index_key_id_nfd_drops_combining_marks() {
    // 'é' → 'e' + combining accent; combining mark isn't ASCII alphanumeric,
    // so it's dropped, leaving just "e".
    assert_eq!(get_index_key_id("é"), "e");
    assert_eq!(get_index_key_id("Éclair"), "Eclair");
  }

  #[test]
  fn get_index_key_id_empty_input() {
    assert_eq!(get_index_key_id(""), "");
    assert_eq!(get_index_key_id("!!!"), "");
  }

  #[test]
  fn cyclic_permute_empty_returns_single_empty() {
    let empty: Vec<i32> = vec![];
    assert_eq!(cyclic_permute::<i32>(&empty), vec![Vec::<i32>::new()]);
  }

  #[test]
  fn cyclic_permute_single_element_returns_itself() {
    assert_eq!(cyclic_permute(&[42]), vec![vec![42]]);
  }

  #[test]
  fn cyclic_permute_three_element_rotations() {
    let result = cyclic_permute(&["a", "b", "c"]);
    assert_eq!(result, vec![
      vec!["a", "b", "c"],
      vec!["b", "c", "a"],
      vec!["c", "a", "b"],
    ]);
  }

  #[test]
  fn cyclic_permute_count_equals_input_len() {
    let result = cyclic_permute(&[1, 2, 3, 4, 5]);
    assert_eq!(result.len(), 5);
    for perm in &result {
      assert_eq!(perm.len(), 5);
    }
  }
}

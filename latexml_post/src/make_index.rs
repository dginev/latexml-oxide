//! Index generation processor.
//!
//! Port of `LaTeXML::Post::MakeIndex` (504 lines of Perl).
//! Collects INDEX:* entries from the ObjectDB, builds a tree of index entries
//! grouped by initial letter, and fills in `ltx:index` and `ltx:glossary` elements.
//! Supports permuted indexes, splitting by initial, see-also references,
//! range-style page references, and glossary entry formatting.

use libxml::tree::Node;
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;

use crate::document::{NodeData, PostDocument};
use crate::object_db::{ObjectDB, Value};
use crate::processor::{ProcessResult, Processor};

/// Index tree node.
#[derive(Debug)]
struct IndexTree {
  id:               String,
  key:              Option<String>,
  full_key:         Option<String>,
  phrase:           Option<String>,
  phrase_text:      Option<String>,
  full_phrase_text: Option<String>,
  subtrees:         HashMap<String, IndexTree>,
  referrers:        HashMap<String, HashMap<String, bool>>,
  see_also:         Vec<String>,
}

impl IndexTree {
  fn new(id: &str) -> Self {
    IndexTree {
      id:               id.to_string(),
      key:              None,
      full_key:         None,
      phrase:           None,
      phrase_text:      None,
      full_phrase_text: None,
      subtrees:         HashMap::new(),
      referrers:        HashMap::new(),
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
    log::info!("MakeIndex: {} entries", keys.len());

    let mut all_phrases: HashMap<String, String> = HashMap::new();
    let mut tree = IndexTree::new(index_id);

    for key in &keys {
      if let Some(entry) = self.db.lookup(key) {
        let phrase_keys: Vec<&str> = key
          .strip_prefix("INDEX:")
          .unwrap_or("")
          .split(':')
          .filter(|s| !s.is_empty())
          .collect();
        if phrase_keys.is_empty() {
          continue;
        }

        if self.permuted {
          // Cyclic permutations of phrase keys
          for perm in cyclic_permute(&phrase_keys) {
            if self.split {
              let init = initial_letter(perm[0]);
              let subtree = tree.subtrees.entry(init.clone()).or_insert_with(|| {
                let mut st = IndexTree::new(&tree.id);
                st.phrase = Some(init);
                st
              });
              add_tree_rec(subtree, &perm, &mut all_phrases, entry);
            } else {
              add_tree_rec(&mut tree, &perm, &mut all_phrases, entry);
            }
          }
        } else if self.split {
          let init = initial_letter(phrase_keys[0]);
          let subtree = tree.subtrees.entry(init.clone()).or_insert_with(|| {
            let mut st = IndexTree::new(&tree.id);
            st.phrase = Some(init);
            st
          });
          add_tree_rec(subtree, &phrase_keys, &mut all_phrases, entry);
        } else {
          add_tree_rec(&mut tree, &phrase_keys, &mut all_phrases, entry);
        }
      }
    }
    Some((tree, all_phrases))
  }

  /// Generate XML for an index list from a tree.
  fn make_index_list(
    &self,
    all_phrases: &HashMap<String, String>,
    tree: &IndexTree,
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
          .map(|st| self.make_index_entry(all_phrases, st))
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
  fn make_index_entry(&self, all_phrases: &HashMap<String, String>, tree: &IndexTree) -> NodeData {
    let mut children = Vec::new();

    // Phrase
    if let Some(ref phrase) = tree.phrase {
      children.push(NodeData::Element {
        tag:        "ltx:indexphrase".to_string(),
        attributes: tree
          .key
          .as_ref()
          .map(|k| HashMap::from([("key".to_string(), k.clone())])),
        children:   vec![NodeData::Text(phrase.clone())],
      });
    }

    // Referrer links (combined with range handling)
    let mut links = Vec::new();
    if !tree.referrers.is_empty() {
      links.extend(self.combine_index_entries(&tree.referrers));
    }

    // See-also links
    for see_text in &tree.see_also {
      if !links.is_empty() {
        links.push(NodeData::Text(", ".to_string()));
      }
      // Try to find the referenced phrase in allphrases
      if let Some(target_id) = all_phrases
        .get(see_text)
        .or_else(|| all_phrases.get(&see_text.to_lowercase()))
      {
        links.push(NodeData::Element {
          tag:        "ltx:ref".to_string(),
          attributes: Some(HashMap::from([("idref".to_string(), target_id.clone())])),
          children:   vec![NodeData::Text(see_text.clone())],
        });
      } else {
        links.push(NodeData::Element {
          tag:        "ltx:text".to_string(),
          attributes: Some(HashMap::from([("font".to_string(), "italic".to_string())])),
          children:   vec![NodeData::Text(see_text.clone())],
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

    // Sub-entries
    if let Some(sublist) = self.make_index_list(all_phrases, tree) {
      children.push(sublist);
    }

    NodeData::Element {
      tag: "ltx:indexentry".to_string(),
      attributes: if tree.id.is_empty() {
        None
      } else {
        Some(HashMap::from([("xml:id".to_string(), tree.id.clone())]))
      },
      children,
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

      // Check for range start
      if styles.contains_key("rangestart") {
        let start_id = id;
        let mut end_id = id;
        let mut level = 1i32;
        i += 1;
        while i < ids.len() && level > 0 {
          end_id = ids[i];
          if let Some(s) = refs.get(end_id) {
            if s.contains_key("rangestart") {
              level += 1;
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
    let style: Vec<&String> = styles
      .keys()
      .filter(|s| *s != "rangestart" && *s != "rangeend")
      .collect();
    let primary_style = style.first().map(|s| s.as_str()).unwrap_or("normal");

    let ref_node = NodeData::Element {
      tag:        "ltx:ref".to_string(),
      attributes: Some(HashMap::from([
        ("idref".to_string(), id.to_string()),
        ("show".to_string(), "typerefnum".to_string()),
      ])),
      children:   vec![],
    };

    if primary_style != "normal" {
      NodeData::Element {
        tag:        "ltx:text".to_string(),
        attributes: Some(HashMap::from([(
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
  fn get_glossary_entries(&self, lists: &str, glossary_id: &str) -> Vec<GlossaryEntry> {
    let list_set: std::collections::HashSet<&str> = lists.split(',').collect();
    let mut entries = Vec::new();

    for db_key in self.db.get_keys() {
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
            attributes: Some(HashMap::from([
              ("lists".to_string(), lists.to_string()),
              ("xml:id".to_string(), id),
              ("key".to_string(), key.to_string()),
            ])),
            children:   vec![
              NodeData::Element {
                tag:        "ltx:glossaryphrase".to_string(),
                attributes: Some(HashMap::from([
                  ("role".to_string(), "label".to_string()),
                  ("key".to_string(), key.to_string()),
                ])),
                children:   vec![NodeData::Text(term)],
              },
              NodeData::Element {
                tag:        "ltx:glossaryphrase".to_string(),
                attributes: Some(HashMap::from([(
                  "role".to_string(),
                  "definition".to_string(),
                )])),
                children:   vec![NodeData::Text(desc)],
              },
            ],
          },
        });
      }
    }
    entries.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
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
      let id = node.get_attribute("xml:id").unwrap_or_default();

      if tag == "ltx:index" {
        if let Some((tree, all_phrases)) = self.build_tree(&id) {
          if let Some(index_list) = self.make_index_list(&all_phrases, &tree) {
            let mut node_mut = node.clone();
            doc.add_nodes(&mut node_mut, &[index_list]);
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
  phrase_keys: &[&str],
  all_phrases: &mut HashMap<String, String>,
  entry: &crate::object_db::Entry,
) {
  if phrase_keys.is_empty() {
    // Leaf: record referrers and see_also
    if let Some(Value::Hash(refs)) = entry.get_value("referrers") {
      for (k, v) in refs {
        let styles = tree.referrers.entry(k.clone()).or_default();
        styles.insert(v.to_string(), true);
      }
    }
    if let Some(Value::List(see_items)) = entry.get_value("see_also") {
      for item in see_items {
        tree.see_also.push(item.to_string());
      }
    }
    return;
  }
  let key = phrase_keys[0];
  let rest = &phrase_keys[1..];
  let key_id = get_index_key_id(key);
  let parent_key = tree.full_key.clone().unwrap_or_default();
  let full_key = if parent_key.is_empty() {
    key.to_string()
  } else {
    format!("{}.{}", parent_key, key)
  };
  let parent_phrase = tree.full_phrase_text.clone().unwrap_or_default();
  let full_phrase = if parent_phrase.is_empty() {
    key.to_string()
  } else {
    format!("{} {}", parent_phrase, key)
  };

  let tree_id = tree.id.clone();
  let subtree = tree.subtrees.entry(key.to_string()).or_insert_with(|| {
    let id = format!("{}.{}", tree_id, key_id);
    all_phrases.insert(full_key.clone(), id.clone());
    all_phrases.insert(full_key.to_lowercase(), id.clone());
    all_phrases.insert(full_phrase.clone(), id.clone());
    all_phrases.insert(full_phrase.to_lowercase(), id.clone());
    let mut st = IndexTree::new(&id);
    st.key = Some(key.to_string());
    st.full_key = Some(full_key.clone());
    st.phrase = Some(key.to_string());
    st.phrase_text = Some(key.to_string());
    st.full_phrase_text = Some(full_phrase);
    st
  });
  add_tree_rec(subtree, rest, all_phrases, entry);
}

fn initial_letter(key: &str) -> String {
  let decomposed: String = key.nfd().collect();
  match decomposed.trim().chars().next() {
    Some(c) if c.is_ascii_alphabetic() => c.to_uppercase().to_string(),
    _ => "*".to_string(),
  }
}

fn get_index_key_id(key: &str) -> String {
  key
    .nfd()
    .collect::<String>()
    .chars()
    .filter(|c| c.is_ascii_alphanumeric())
    .collect()
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

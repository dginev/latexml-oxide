pub mod helpers;
pub mod resource;
pub mod tag;

use libxml::tree::Document as XmlDoc;
use libxml::tree::set_node_rc_guard;
use libxml::tree::{Namespace, Node, NodeType};
use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use std::backtrace::Backtrace;

use std::borrow::Cow;
use std::collections::{BTreeSet, VecDeque};
use std::fmt::Write as _;
use std::rc::Rc;

use crate::TexMode;
use crate::common::arena::{self, SymHashMap, SymStr};
use crate::common::error::*;
use crate::common::font::{FONT_TEXT_DEFAULT, Font};
use crate::common::locator::Locator;
use crate::common::model;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::common::xml::{self, XML_NS, XPath};
use crate::definition::FontDirective;
use crate::ligature::Ligature;
use crate::list::List;
use crate::pin;
use crate::state;
use crate::util::radix::radix_alpha;

use crate::Tbox;
use crate::document::resource::Resource;
use crate::document::tag::{TagConstructionClosure, TagOptionName};
use crate::{BoxOps, Digested, DigestedData};

static HAS_NONSPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\S").unwrap());
static ONLY_SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s+$").unwrap());
static DASHES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\-\-+").unwrap());

/// #217 diagnostic (TEMP): the source location of the most recent
/// `set_node` assignment to `self.node`, so a macOS run can report where
/// the current node was set when a later read finds it corrupt. Remove
/// together with the `[#217]` eprintln guards once the corruption source
/// is fixed.
#[thread_local]
static LAST_SET_NODE_LOC: std::cell::Cell<Option<&'static std::panic::Location<'static>>> =
  std::cell::Cell::new(None);

/// Node types that may legitimately be the document's current insertion
/// node (`self.node`). Anything else read there is a corrupt/freed node —
/// the macOS #217 residual. Used by the defensive guards below.
fn is_sane_current_node_type(t: &Option<NodeType>) -> bool {
  matches!(
    t,
    Some(
      NodeType::ElementNode
        | NodeType::TextNode
        | NodeType::DocumentNode
        | NodeType::DocumentFragNode
    )
  )
}
static NON_MERGEABLE_ATTRIBUTES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
  HashSet::from_iter([
    "about",
    "aboutlabelref",
    "aboutidref",
    "resource",
    "resourcelabelref",
    "resourceidref",
    "property",
    "rel",
    "rev",
    "tyupeof",
    "datatype",
    "content",
    "data",
    "datamimetype",
    "dataencoding",
  ])
});
// When merging attributes of two nodes, some attributes should be combined
// Merged space separated
static MERGE_ATTRIBUTE_SPACEJOIN: Lazy<HashSet<&'static str>> =
  Lazy::new(|| HashSet::from_iter(["class", "lists", "inlist", "labels"]));
// Merged ";" separated
static MERGE_ATTRIBUTE_SEMICOLONJOIN: Lazy<HashSet<&'static str>> =
  Lazy::new(|| HashSet::from_iter(["cssstyle"]));
// Summed lengths
static MERGE_ATTRIBUTE_SUMLENGTH: Lazy<HashSet<&'static str>> = Lazy::new(|| {
  HashSet::from_iter([
    "xoffset",
    "yoffset",
    "lpadding",
    "rpadding",
    "xtranslate",
    "ytranslate",
  ])
});

pub static FONT_ELEMENT_NAME: &str = "ltx:text";
pub static MATH_TOKEN_NAME: &str = "ltx:XMTok";
pub static MATH_HINT_NAME: &str = "ltx:XMHint";

pub struct Document {
  pub document:                XmlDoc,
  pub pending:                 Vec<Node>,
  context:                     Option<XPath>,
  pub node:                    Node,
  pub node_boxes:              HashMap<usize, Digested>, // used to be _box attribute
  pub node_fonts:              HashMap<u64, Font>,       // used to be _font attribute
  pub idstore:                 HashMap<String, Node>,
  // the rewrite labels used to be in each rewrite rule, but they make more sense in doc
  pub rewrite_labels:          HashMap<String, String>,
  // the following are internal "local"-based declarations in Perl
  localized_constructed_nodes: Vec<Vec<Node>>,
  constructed_nodes:           Vec<Node>,
  /// Free-list of emptied `Vec<Node>` buffers, reused across `init_constructed_nodes`
  /// / `close_constructed_nodes` cycles. The per-body constructed-nodes frame pushes
  /// here after its elements are drained so the next `init_constructed_nodes` can pop
  /// a buffer with pre-existing capacity instead of heap-allocating a fresh one.
  /// Cuts the `Vec<Node>::from_iter` hotspot that dominated absorb profiles.
  reusable_node_buffers:       Vec<Vec<Node>>,
  localized_boxes:             Vec<Option<Digested>>,
  box_to_absorb:               Option<Digested>, // local $LaTeXML::BOX;
  /// Source-map (`--source-map`) cache: the current `box_to_absorb`'s
  /// source range, captured as a plain `Copy` `Locator` at set time so
  /// stamping never re-borrows the box's `RefCell` mid-absorb (which
  /// panics for the mutably-borrowed `Alignment` path). Mirrors the
  /// `box_to_absorb` save stack; `None` when source-map is off.
  current_box_locator:         Option<Locator>,
  localized_box_locators:      Vec<Option<Locator>>,
  localized_fonts:             Vec<Rc<Font>>,
}
impl Default for Document {
  fn default() -> Self { Self::new() }
}
impl Object for Document {
  fn get_locator(&self) -> Option<Locator> {
    self.get_node_box(&self.node).and_then(|tbox| tbox.get_locator())
  }
}

/// Attachment policy for `Document::add_comment`. Kept module-local.
enum Placement_ {
  AppendChild,
  PrevSibling,
}

impl Document {
  pub fn new() -> Self {
    crate::ensure_libxml_init(); // Thread-safe libxml2 initialization
    // `NODE_RC_MAX_GUARD` is libxml's diagnostic threshold for mutable
    // access to Rc-shared nodes; the real aliasing guarantee is the
    // `weak_count == 0` check in `Node::mut_node`. For legitimate large
    // documents (e.g. arxiv 0805.2376 with deep dcpic commutative-diagram
    // sharing), natural ref counts exceed the crate's default of 2 by
    // several orders of magnitude. Raise to 8192 to accommodate.
    //
    // libxml-rs implements this as a `pub static mut NODE_RC_MAX_GUARD:
    // usize` with `unsafe { NODE_RC_MAX_GUARD = value; }` — concurrent
    // writes from N test threads each constructing their own Document
    // are a classic data race on a `static mut`. Gate the write through
    // a `std::sync::Once` so it happens exactly once per process; reads
    // from `Node::mut_node` then see a stable value.
    static GUARD_INIT: std::sync::Once = std::sync::Once::new();
    GUARD_INIT.call_once(|| set_node_rc_guard(8192));
    let doc_scaffold = XmlDoc::new().unwrap();
    let root = match doc_scaffold.get_root_element() {
      Some(root) => root,
      None => doc_scaffold.as_node(), // when empty, set the document node as a node.
    };
    Document {
      document:                    doc_scaffold,
      node:                        root,
      node_boxes:                  HashMap::default(),
      node_fonts:                  HashMap::default(),
      idstore:                     HashMap::default(),
      rewrite_labels:              HashMap::default(),
      pending:                     Vec::new(),
      localized_constructed_nodes: Vec::new(),
      constructed_nodes:           Vec::new(),
      reusable_node_buffers:       Vec::new(),
      box_to_absorb:               None,
      current_box_locator:         None,
      localized_box_locators:      Vec::new(),
      context:                     None,
      localized_boxes:             Vec::new(),
      localized_fonts:             Vec::new(),
    }
  }

  /// Get the element at (or containing) the current insertion point.
  pub fn get_element(&self) -> Option<Node> {
    let mut node = &self.node;
    let parent = node.get_parent();
    if node.get_type() == Some(NodeType::TextNode) {
      node = parent.as_ref().unwrap();
    }
    let final_type = node.get_type();
    if final_type.is_none() || final_type == Some(NodeType::DocumentNode) {
      None
    } else {
      Some(node.clone())
    }
  }

  /// Find the nodes according to the given `xpath` expression,
  /// the xpath is relative to $node (if given), otherwise to the document node.
  pub fn findnodes(&mut self, xpath: &str, node_opt: Option<&Node>) -> Vec<Node> {
    let node = match node_opt {
      Some(node) => Cow::Borrowed(node),
      None => match self.document.get_root_element() {
        Some(root) => Cow::Owned(root),
        None => return Vec::new(),
      },
    };
    self.get_xpath().findnodes(xpath, Some(&node))
  }

  /// Get an XPath context that knows about our namespace mappings.
  pub fn get_xpath(&mut self) -> &mut XPath {
    if let Some(ref mut ctxt) = self.context {
      ctxt
    } else {
      let mut context = XPath::new(&self.document, HashMap::default());
      model::with_code_namespaces(|code_ns| {
        for (prefix, ns) in code_ns {
          // TODO: Is this too slow? We may need to store an active context in the state as an
          // alternative
          arena::with2(*prefix, *ns, |p_str, ns_str| {
            context
              .register_namespace(p_str, ns_str)
              .expect("register_namespace has no reason to fail during get_xpath?");
          });
        }
      });
      self.context = Some(context);
      self.context.as_mut().unwrap()
    }
  }

  /// Like findnodes, but only returns the first matched node
  pub fn findnode(&mut self, xpath: &str, node: Option<&Node>) -> Option<Node> {
    let mut nodes = self.get_xpath().findnodes(xpath, node);
    if nodes.is_empty() {
      None
    } else {
      Some(nodes.remove(0))
    }
  }

  /// Like findnodes, but expects an xpath that evaluates to a literal value (e.g. for attributes)
  pub fn findvalues(&mut self, xpath: &str, node_opt: Option<&Node>) -> Vec<String> {
    match node_opt {
      Some(node) => self.get_xpath().findvalues(xpath, Some(node)),
      None => {
        if let Some(root) = self.document.get_root_element() {
          self.get_xpath().findvalues(xpath, Some(&root))
        } else {
          Vec::new()
        }
      },
    }
  }

  pub fn get_node(&self) -> &Node { &self.node }
  pub fn get_node_mut(&mut self) -> &mut Node { &mut self.node }

  pub fn get_document(&self) -> &XmlDoc { &self.document }
  pub fn get_document_mut(&mut self) -> &mut XmlDoc { &mut self.document }

  // **********************************************************************
  // This should be called before returning the final XML::LibXML::Document to the
  // outside world.  It resolves the fonts for each node relative to it's
  // ancestors. It removes the `helper' attributes that store fonts, source
  // box, etc.
  pub fn finalize(&mut self) -> Result<()> {
    // Belt-and-suspenders idstore rebuild before prune_xmduals.
    // Originally guarded a SIGSEGV on arxiv 1605.08055 where
    // `mark_xmnode_visibility` dereferenced dangling lookup_id
    // entries while recursing through XMRef nodes. Cycle 72 audited
    // the 5 hazard call sites the earlier comment listed (math-parser
    // `replace_tree` at parser.rs:456/690, `unbind_node` loops at
    // parser.rs:639/856 and rewrite.rs:522); all 5 now have proper
    // unrecord_node_ids / remove_node-cascade coverage. The rebuild
    // call is retained as a safety net pending empirical
    // 10k-sandbox verification on 1605.08055 — see SYNC_STATUS.md
    // D3b [~] entry. A fresh DOM walk drops any surviving dangling
    // entries; duplicates in DOM get modify_id via record_node_ids.
    self.rebuild_idstore_from_dom()?;
    // Sweep dangling XMRefs that were specifically created by
    // amsmath::rearrange_ams_split (tagged with `_split_ref="1"`).
    // The math parser later absorbs some XMArray cells (inserted
    // MULOPs, etc.) and the parallel XMWrap refs end up pointing
    // at vanished targets, cascading through Warn:expected:node
    // (here) and Error:expected:id (post-process) for ~1500 wp3
    // canvas papers. Restricting the sweep to `_split_ref` avoids
    // breaking declare_test's renamed-id case (XMRefs pointing to
    // `S1.Ex1.m1.1`-style ids that resolve through Perl-faithful
    // idstore staleness; those don't carry the marker).
    self.prune_dangling_split_xmrefs()?;
    self.prune_xmduals()?;
    if let Some(mut root) = self.document.get_root_element() {
      self.set_local_font(Rc::new(Font::text_default()));
      self.finalize_rec(&mut root)?;
      self.set_rdfa_prefixes();
      self.apply_document_namespace_declarations(&mut root);
      self.expire_local_font();
    }
    Ok(())
  }

  /// Apply registered document namespace declarations to the root element.
  /// Perl's RegisterDocumentNamespace stores prefix→URI mappings in the model.
  /// These must appear as xmlns:prefix="URI" on the root element during serialization.
  /// Only emit namespaces that are actually used in the document (Perl behavior).
  fn apply_document_namespace_declarations(&self, root: &mut Node) {
    let prefixes = model::get_document_namespace_prefixes();
    // Collect which prefixes are actually used in the document
    let nsnodes = root.get_namespace_declarations();
    let existing_prefixes: Vec<String> = nsnodes.iter().map(|ns| ns.get_prefix()).collect();
    for (prefix, ns_uri) in prefixes {
      // Skip internal/default namespaces
      if prefix.is_empty() || prefix == "ltx" || prefix == "xml" {
        continue;
      }
      // Skip namespaces containing "DEFAULT#" (internal model entries)
      if ns_uri.contains("DEFAULT#") {
        continue;
      }
      // Only add if this prefix appears as a namespace declaration on some child node
      // (meaning it's actually used in the document)
      if existing_prefixes.contains(&prefix) {
        continue; // already declared on root
      }
      // Check if any descendant element uses this namespace prefix
      // by looking for namespace declarations on descendant elements
      let has_usage = self.has_namespace_usage(root, &prefix);
      if has_usage {
        let attr_name = format!("xmlns:{prefix}");
        root.set_attribute(&attr_name, &ns_uri).ok();
      }
    }
  }

  /// Check if any descendant of node uses the given namespace prefix.
  fn has_namespace_usage(&self, node: &Node, prefix: &str) -> bool {
    // Check attributes of this node for prefix: usage. The
    // allocation-free check `starts_with(prefix) + byte-at-prefix.len()`
    // replaces `format!("{prefix}:")` which would heap-allocate per node
    // visited during the recursive descent.
    let plen = prefix.len();
    for (key, _) in node.get_attributes() {
      if key.len() > plen && key.starts_with(prefix) && key.as_bytes()[plen] == b':' {
        return true;
      }
    }
    // Check children recursively
    for child in node.get_child_nodes() {
      if child.get_type() == Some(NodeType::ElementNode) {
        // Check if element itself is in this namespace
        for ns in child.get_namespace_declarations() {
          if ns.get_prefix() == prefix {
            return true;
          }
        }
        if self.has_namespace_usage(&child, prefix) {
          return true;
        }
      }
    }
    false
  }

  /// Remove xml:ids from XMTok elements that aren't referenced by any idref.
  /// The Rust math parser generates xml:ids on XMTok nodes for internal XMRef linkage
  /// during parsing. After finalization (which includes prune_xmduals), some ids
  /// are no longer referenced. Perl's parser doesn't generate these ids.
  pub fn cleanup_unreferenced_xmtok_ids(&mut self) {
    use rustc_hash::FxHashSet as HashSet;
    let mut referenced_ids: HashSet<String> = HashSet::default();
    for node in self.findnodes("descendant-or-self::*[@idref]", None) {
      if let Some(idref) = node.get_attribute("idref") {
        referenced_ids.insert(idref);
      }
    }
    let xml_ns = "http://www.w3.org/XML/1998/namespace";
    let toks = self.findnodes("descendant-or-self::ltx:XMTok[@xml:id]", None);
    for mut tok in toks {
      if let Some(id) = tok.get_attribute_ns("id", xml_ns) {
        if !referenced_ids.contains(&id) {
          self.unrecord_id(&id);
          // Remove both the prefixed attribute and the ns attribute
          let _ = tok.remove_attribute("xml:id");
          let _ = tok.remove_attribute_ns("id", xml_ns);
        }
      }
    }
  }

  /// Iterative implementation of finalize_rec to avoid stack overflow.
  /// Uses an explicit heap-allocated work stack instead of call-stack recursion.
  /// Resolves fonts for each node relative to its ancestors, and removes
  /// helper attributes (_font, _standalone_font, etc).
  fn finalize_rec(&mut self, node: &mut Node) -> Result<()> {
    // Work items for the iterative traversal.
    // Enter: process font declarations and children (text children handled inline,
    //        element children deferred to the stack).
    // PostElement: after finalizing an element child, check for font wrapper collapse.
    // PostWork: after all children processed, remove bookkeeping attrs and expire font.
    #[allow(clippy::enum_variant_names)]
    enum Work {
      Enter(Node),
      PostElement {
        child:         Node,
        parent_qname:  SymStr,
        was_forcefont: bool,
      },
      PostWork {
        node:              Node,
        /// Bookkeeping attribute names captured during Enter, before tree modifications
        bookkeeping_attrs: Vec<String>,
      },
    }

    let mut stack: Vec<Work> = Vec::new();
    // Track nodes that have no visible attributes after bookkeeping removal.
    // Used in PostElement to decide collapse without calling get_attributes()
    // (which can crash on corrupted libxml2 attribute lists after replace_node).
    let mut empty_attr_nodes: HashSet<usize> = HashSet::default();
    // Defer collapse operations to avoid libxml2 memory corruption.
    // Collapsing nodes during tree traversal corrupts attribute linked lists
    // of ancestor nodes. By collecting collapses and running them after
    // the entire traversal completes, we avoid accessing corrupted state.
    let mut deferred_collapses: Vec<(Node, SymStr)> = Vec::new();
    stack.push(Work::Enter(node.clone()));

    while let Some(work) = stack.pop() {
      match work {
        Work::Enter(mut current) => {
          let qname = get_node_qname(&current);
          let local_font = self.get_local_font().unwrap();
          // _standalone_font is typically for metadata that gets extracted out of context
          let mut declared_font = if current.has_attribute("_standalone_font") {
            Cow::Borrowed(&*FONT_TEXT_DEFAULT)
          } else {
            Cow::Borrowed(&*local_font)
          };

          // TODO: _pre_comment / _comment insertion requires create_comment support in libxml
          // wrapper Perl: parent.insertBefore(XML::LibXML::Comment.new(comment), node)
          // Perl: parent.insertAfter(XML::LibXML::Comment.new(comment), node)

          // Use boxed HashMap to reduce work item size — Font is ~500 bytes per entry
          let mut pending_declaration: Box<HashMap<String, (String, Font)>> = Box::default();

          if self.has_node_font(&current) {
            let desired_font = self.get_node_font(&current);
            *pending_declaration = desired_font.relative_to(&declared_font);
            if (!current.get_child_nodes().is_empty() || current.has_attribute("_force_font"))
              && !pending_declaration.is_empty()
            {
              let mut keys_to_remove: Vec<SymStr> = Vec::new();
              let mut attrs_to_set: Vec<(SymStr, SymStr)> = Vec::new();
              for (key, (value, properties)) in pending_declaration.iter() {
                if model::can_have_attribute(qname, arena::pin(key)) {
                  let key_sym = arena::pin(key);
                  attrs_to_set.push((key_sym, arena::pin(value)));
                  // Merge to set the font currently in effect
                  declared_font = Cow::Owned(declared_font.merge_ref(properties));
                  keys_to_remove.push(key_sym);
                }
              }

              for (key, mut value) in attrs_to_set {
                if key == pin!("class") {
                  // Merge and sort class values alphabetically, matching Perl's behavior
                  if let Some(ovalue) = current.get_attribute("class") {
                    let new_s = arena::with(value, |s| s.to_string());
                    let mut classes: Vec<&str> = new_s
                      .split_whitespace()
                      .chain(ovalue.split_whitespace())
                      .collect();
                    classes.sort_unstable();
                    classes.dedup();
                    value = arena::pin(classes.join(" "));
                  }
                }
                // Resolve to owned Strings before calling set_attribute,
                // to avoid holding an arena borrow while set_attribute may need arena::pin
                // for schema-based attribute filtering (canHaveAttribute check).
                let key_s = arena::with(key, |s| s.to_string());
                let value_s = arena::with(value, |s| s.to_string());
                self.set_attribute(&mut current, &key_s, &value_s)?;
              }
              for key in keys_to_remove {
                arena::with(key, |key_str| pending_declaration.remove(key_str));
              }
            }
          }
          // Optionally add ids to all nodes (AFTER all parsing, rearrangement, etc)
          if qname != pin!("ltx:document")
            && state::lookup_bool("GENERATE_IDS")
            && !current.has_attribute("xml:id")
            && !current.has_attribute("id") // SVG elements with plain id don't need xml:id
            && arena::with(qname, |qname_str| can_have_attribute(qname_str, "xml:id"))
          {
            self.generate_id(&mut current, "")?;
          }
          self.set_local_font(Rc::new(declared_font.into_owned()));

          // Capture bookkeeping attribute names now, before tree modifications.
          // This avoids calling get_attributes() later when the attribute list
          // might be corrupted by replace_node operations.
          let bookkeeping_attrs: Vec<String> = current
            .get_attributes()
            .into_keys()
            .filter(|name| name.starts_with('_'))
            .collect();

          // Process children using the snapshot from get_child_nodes().
          // Element children are deferred to the stack; text children are handled inline.
          // PostWork is pushed first (runs after all children), then children in reverse.
          stack.push(Work::PostWork {
            node: current.clone(),
            bookkeeping_attrs,
          });

          let children = current.get_child_nodes();
          // Collect work items forward, then push in reverse for left-to-right processing
          let mut child_work: Vec<Work> = Vec::new();
          for child in &children {
            let child_type = child.get_type();
            if child_type == Some(NodeType::ElementNode) {
              let was_forcefont = child.has_attribute("_force_font");
              // Enter first, PostElement after (reversed when pushed to stack)
              child_work.push(Work::Enter(child.clone()));
              child_work.push(Work::PostElement {
                child: child.clone(),
                parent_qname: qname,
                was_forcefont,
              });
            } else if child_type == Some(NodeType::TextNode) {
              // Text node: wrap with font element if needed (handled inline)
              let mut text_keys_to_remove = Vec::new();
              for key in pending_declaration.keys() {
                if !can_have_attribute(FONT_ELEMENT_NAME, key) {
                  text_keys_to_remove.push(key.clone());
                }
              }
              for key in text_keys_to_remove {
                pending_declaration.remove(&key);
              }
              if can_contain(&current, FONT_ELEMENT_NAME) && !pending_declaration.is_empty() {
                if let Some(mut text) = self.wrap_nodes(FONT_ELEMENT_NAME, vec![child.clone()])? {
                  for (key, (value, _properties)) in pending_declaration.iter() {
                    self.set_attribute(&mut text, key, value)?;
                  }
                  // Text wrapper finalization is shallow (only text content), push to stack
                  child_work.push(Work::Enter(text));
                }
              }
            }
          }
          // Push in reverse so left-to-right processing order is maintained
          for work_item in child_work.into_iter().rev() {
            stack.push(work_item);
          }
        },
        Work::PostElement {
          child,
          parent_qname,
          was_forcefont,
        } => {
          // After finalizing a child element, check if it should be collapsed.
          // Use empty_attr_nodes set instead of child.get_attributes().is_empty()
          // to avoid traversing the attribute linked list, which can be corrupted
          // by prior replace_node operations in libxml2.
          // Defer the actual collapse to after the traversal completes, since
          // replace_node can corrupt ancestor attribute lists in libxml2.
          if (get_node_qname(&child) == pin!("ltx:text"))
            && !was_forcefont
            && empty_attr_nodes.contains(&child.to_hashable())
          {
            let grandchildren = child.get_child_nodes();
            if grandchildren
              .iter()
              .all(|gchild| can_contain_qsym(parent_qname, get_node_qname(gchild)))
            {
              deferred_collapses.push((child, parent_qname));
            }
          }
        },
        Work::PostWork {
          mut node,
          bookkeeping_attrs: _captured_at_enter,
        } => {
          // Mirrors Perl `Document.pm:452`: at finalize time, ANY attribute
          // whose name starts with `_` is internal bookkeeping and gets
          // stripped. Re-derive the set at PostWork time rather than relying
          // on the captured-at-Enter snapshot, because descendant
          // `generate_id` calls can write `_ID_counter_<prefix>_` attributes
          // ONTO the current node (their nearest-id-bearing ancestor) during
          // child traversal — those late-added attrs were missing from the
          // Enter snapshot and would otherwise leak into the output XML,
          // causing duplicate xml:id collisions in the post-processing
          // libxml2 validator (1312.5864 cluster: 70× `S8.T5.m2241 already
          // defined`, where the Math element carried `_ID_counter__="1"`
          // populated post-Enter).
          let attrs_now = node.get_attributes();
          let total_attrs = attrs_now.len();
          let bookkeeping_attrs: Vec<&String> = attrs_now
            .keys()
            .filter(|name| name.starts_with('_'))
            .collect();
          let bookkeeping_count = bookkeeping_attrs.len();
          for name in bookkeeping_attrs {
            let _ = node.remove_attribute(name);
          }
          // If all attributes were bookkeeping, the node now has empty attrs.
          if total_attrs <= bookkeeping_count {
            empty_attr_nodes.insert(node.to_hashable());
          }
          self.expire_local_font();
        },
      }
    }
    // Execute deferred font wrapper collapses now that the entire tree has been
    // finalized. Process from deepest (last) to shallowest (first) to avoid
    // corrupting ancestor attribute lists during the replacement operations.
    // This deferred approach prevents libxml2 memory corruption that occurs
    // when replace_node runs during tree traversal.
    for (child, _parent_qname) in deferred_collapses.into_iter().rev() {
      let grandchildren = child.get_child_nodes();
      if grandchildren.is_empty() {
        // Empty font wrapper — just remove it
        self.remove_node(child);
      } else {
        Debug!(
          "will replace {} grandchildren nodes in finalize_rec (deferred)",
          grandchildren.len()
        );
        self.replace_node(child, grandchildren)?;
      }
    }
    Ok(())
  }

  /// Document construction at the Current Insertion Point.
  ///
  /// absorb the given $box into the DOM (called from constructors).
  /// This will return a list of whatever nodes were created.
  /// Note that this may include nodes that are children of other nodes in the list
  /// or nodes that are no longer in the document.
  /// Also, note that when a text nodes is appended to, the complete text node is in the list,
  /// not just the portion that was added.
  /// [Note that recording the nodes being constructed isn't all that costly,
  /// but filtering them for parent/child relations IS, particularly since it usually isn't needed]
  ///
  /// A box that is a TBox, or List, or Whatsit, is responsible for carrying out
  /// its own insertion, but it should ultimately call methods of Document
  /// that will record the nodes that were created.
  /// $box can also be a plain string (Digested::Postponed)
  /// which will be inserted according to whatever
  /// font, mode, etc, are in %props.
  pub fn absorb(&mut self, object: &Digested, props_opt: Option<SymHashMap<Stored>>) -> Result<()> {
    use DigestedData::*;
    let props = props_opt.unwrap_or_default();
    let mut boxes = vec![Cow::Borrowed(object)];
    while let Some(front_box) = boxes.pop() {
      match front_box.data() {
        List(ref list) => {
          // Simply unwind Lists to avoid unneccessary recursion; This occurs quite frequently!
          for tbox in list.borrow().unlist().into_iter().rev() {
            boxes.push(Cow::Owned(tbox));
          }
        },
        // A Proper Box or Whatsit? Absorb it.
        TBox(ref digested) => {
          self.set_box_to_absorb(Some((*front_box).clone()));
          self.init_constructed_nodes();
          digested.borrow().be_absorbed(self)?;
          // record these for OUTER caller, but return only the most recent set
          self.close_constructed_nodes();
          self.expire_box_to_absorb();
        },
        Whatsit(ref digested) => {
          self.set_box_to_absorb(Some((*front_box).clone()));
          self.init_constructed_nodes();
          digested.borrow().be_absorbed(self)?;
          self.close_constructed_nodes();
          self.expire_box_to_absorb();
        },
        Alignment(ref alignment) => {
          self.set_box_to_absorb(Some((*front_box).clone()));
          self.init_constructed_nodes();
          alignment.borrow_mut().be_absorbed_mut(self)?;
          self.close_constructed_nodes();
          self.expire_box_to_absorb();
        },
        Comment(ref comment) => {
          comment.be_absorbed(self)?;
        },
        Postponed(ref tokens) => {
          let text_font_opt = if let Some(Stored::Font(ref prop_font)) = props.get("font") {
            Some(Rc::clone(prop_font))
          } else {
            match self.box_to_absorb {
              Some(ref thisbox) => thisbox
                .get_font()?
                .map(|thisfont| Rc::new(thisfont.into_owned())),
              None => None,
            }
          };
          let text_font = text_font_opt.unwrap_or_default();
          if let Some(new_text) = self.open_text(&tokens.to_string(), &text_font)? {
            self.record_constructed_node(&new_text);
          }
        },
        KeyVals(ref kv) => {
          // When KeyVals appear in body absorption (e.g. #1 for RequiredKeyVals),
          // convert them to text representation matching Perl's stringify behavior.
          let text = kv.to_string();
          if !text.is_empty() {
            let text_font = match self.box_to_absorb {
              Some(ref thisbox) => thisbox
                .get_font()?
                .map(|f| Rc::new(f.into_owned()))
                .unwrap_or_default(),
              None => Rc::default(),
            };
            if let Some(new_text) = self.open_text(&text, &text_font)? {
              self.record_constructed_node(&new_text);
            }
          }
        },
        RegisterValue(_) => {
          // RegisterValue should not normally appear in the absorption pipeline.
        },
      }
    }
    Ok(())
  }

  fn init_constructed_nodes(&mut self) {
    // Pop a buffer from the free-list (retaining its previously-grown capacity);
    // fall back to a fresh empty Vec if the pool is dry. Swap it in as the new
    // inner frame; the outgoing outer frame goes onto the save stack.
    let fresh = self.reusable_node_buffers.pop().unwrap_or_default();
    let prev = std::mem::replace(&mut self.constructed_nodes, fresh);
    self.localized_constructed_nodes.push(prev);
  }

  /// Close the current constructed-nodes frame, restoring the outer frame and
  /// re-recording the inner frame's nodes into it. The drained inner buffer
  /// is returned to `reusable_node_buffers` (empty but with capacity intact)
  /// so the next `init_constructed_nodes` can reuse it without allocating.
  fn close_constructed_nodes(&mut self) {
    let outer = self.localized_constructed_nodes.pop().unwrap_or_default();
    let mut inner = std::mem::replace(&mut self.constructed_nodes, outer);
    for n in inner.drain(..) {
      self.record_constructed_node(&n);
    }
    // `inner` is now empty; its capacity is preserved. Return it to the pool.
    self.reusable_node_buffers.push(inner);
  }
  pub fn get_constructed_nodes(&self) -> &[Node] { &self.constructed_nodes }

  /// This is a refactored `else` cases from the main absorb routine, to allow for better type
  /// hygiene
  pub fn absorb_string(
    &mut self,
    object: &str,
    props: &SymHashMap<Stored>,
  ) -> Result<Option<Node>> {
    // Else, plain string in text mode.
    let ismath: bool = match props.get("isMath") {
      Some(v) => v.into(),
      None => false,
    };
    if !ismath {
      // Perf: avoid cloning Rc<Font> into owned Font in the common case.
      // We pull out the Rc (shared reference is fine since open_text only
      // borrows the Font, not self) and go through Cow<Font>.
      let font_opt: Option<Rc<Font>> = match props.get("font") {
        Some(Stored::Font(fnt)) => Some(Rc::clone(fnt)),
        Some(Stored::FontDirective(FontDirective::Asset(fnt))) => Some(Rc::clone(fnt)),
        _ => None,
      };
      if let Some(fnt) = font_opt {
        return self.open_text(object, &fnt);
      }
      if let Some(Stored::FontDirective(FontDirective::Closure(code))) = props.get("font") {
        let fnt = code(None)?;
        return self.open_text(object, &fnt);
      }
      // Fallback to box_to_absorb font.
      let fnt = self
        .box_to_absorb
        .as_ref()
        .unwrap()
        .get_font()?
        .unwrap()
        .into_owned();
      self.open_text(object, &fnt)
    } else if get_node_qname(&self.node) == pin!("ltx:XMTok") {
      // Or plain string in math mode.
      // Note text nodes can ONLY appear in <XMTok> or <text>!!!
      // Have we already opened an XMTok? Then insert into it.
      Ok(Some(self.open_math_text_internal(object)?))
    // Else create the XMTok now.
    } else {
      // Odd case: constructors that work in math & text can insert raw strings in Math mode.
      let font_math_opt = match props.get("font") {
        Some(Stored::Font(fnt)) => Some(Cow::Borrowed(&**fnt)),
        Some(Stored::FontDirective(FontDirective::Asset(fnt))) => Some(Cow::Borrowed(&**fnt)),
        Some(Stored::FontDirective(FontDirective::Closure(code))) => Some(Cow::Owned(code(None)?)),
        _ => None,
      };
      if let Some(font_math) = font_math_opt {
        Ok(Some(self.insert_math_token(
          object,
          HashMap::default(),
          Some(&font_math),
        )?))
      } else {
        Ok(Some(self.insert_math_token(
          object,
          HashMap::default(),
          None,
        )?))
      }
    }
  }
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  /// Perl: insertElementBefore — insert a new element before a given point node.
  /// Creates a new element with the given qname and attributes, inserts it
  /// before `point` in the DOM tree, and returns the new node.
  pub fn insert_element_before(
    &self,
    point: &Node,
    qname: &str,
    attrib: Option<HashMap<String, String>>,
  ) -> Result<Node> {
    // Create element in LTX namespace (matching Perl's setNamespace($LTX_NS,'',1))
    let mut new_node = Node::new(qname, None, &self.document)?;
    if let Some(attrs) = attrib {
      for (key, value) in attrs {
        let _ = new_node.set_attribute(&key, &value);
      }
    }
    // insertBefore: add new_node before point
    let mut point_mut = point.clone();
    point_mut.add_prev_sibling(&mut new_node).ok();
    Ok(new_node)
  }

  /// Shorthand for open,absorb,close, but returns the new node.
  pub fn insert_element(
    &mut self,
    qname: &str,
    content: Vec<&Digested>,
    attrib: Option<HashMap<String, String>>,
  ) -> Result<Node> {
    // TODO: Quickly hacked together, needs a careful refactor with all .clone()
    // calls removed
    let node = self.open_element(qname, attrib, None)?;
    // Debug!("Inserting element {:?} with body: {:?}", qname, content);
    for digested in content {
      self.absorb(digested, None)?;
    }

    let self_node = self.node.get_parent().unwrap();
    let mut c = Some(self_node);
    while c.is_some()
      && c.as_ref() != Some(&node)
      && c.as_ref().unwrap().get_type() != Some(NodeType::DocumentNode)
    {
      let parent = c.unwrap().get_parent().unwrap();
      c = match parent.get_type() {
        Some(NodeType::DocumentNode) => None,
        None => None,
        Some(_) => Some(parent),
      };
    }

    // In obscure situations, `node` may have already gotten closed?
    // close it if it is still open.
    if (self.node == node) || (c.as_ref() == Some(&node)) {
      self.close_element(qname)?;
    }
    Ok(node)
  }

  /// Insert a ProcessingInstruction of the form <?op attr=value ...?>
  /// Does NOT move the current insertion point to the PI,
  /// but may move up past a text node.
  // Rust note: attrib would have been best as Vec<(String,String)> but
  // currently quote!() doesn't work out of the box on them
  pub fn insert_pi(
    &mut self,
    op: &str,
    attributes_opt: Option<HashMap<String, String>>,
  ) -> Result<()> {
    let mut attr_data = Vec::new();
    if let Some(attributes) = attributes_opt {
      let mut keys = vec!["class", "package", "options"];
      let other_keys = attributes
        .keys()
        .filter(|k| k.as_str() != "class" && k.as_str() != "package" && k.as_str() != "options")
        .map(String::as_str)
        .collect::<Vec<_>>();
      keys.extend(other_keys);
      for key in keys {
        if let Some(value) = attributes.get(key) {
          attr_data.push(s!("{}=\"{}\"", key, value));
        }
      }
    }
    // self.close_text_internal();  // Close any open text node
    let mut pi_node = self
      .document
      .create_processing_instruction(op, &attr_data.join(" "))
      .unwrap();
    if self.node.get_type() == Some(NodeType::DocumentNode) {
      self.pending.push(pi_node);
    } else {
      // Perl: insertPI always places PIs before the root element.
      // Find the document root and insert before it.
      let doc_node = self.document.clone();
      if let Some(mut root) = doc_node.get_root_element() {
        root.add_prev_sibling(&mut pi_node)?;
      } else {
        self.node.add_prev_sibling(&mut pi_node)?;
      }
    }
    Ok(())
  }

  pub fn open_element(
    &mut self,
    qname: &str,
    attributes: Option<HashMap<String, String>>,
    font_opt: Option<&Font>,
  ) -> Result<Node> {
    // NoteProgress('.') if (self.progress}++ % 25) == 0;
    // Debug!(
    //   s!("Open element {:?} at {:?}",
    //   qname,
    //   self.with_node_qname(&self.node))
    // );
    let mut point = self.find_insertion_point(qname, None)?;
    let newnode = self.open_element_at(&mut point, qname, attributes, font_opt.cloned())?;
    self.set_node(&newnode);
    // Underscore attributes such as _box and _font from LaTeXML-proper are now
    // bookkept in special substructs of Document Connected to the node hash.
    // Ideally should be as quick to recompute natively as it would be to set/get
    // attributes externally via libxml.
    //
    // TODO: also accept a _box argument eventually? Or store differently?
    // attributes.entry("_box").or_insert(state_mut!().locals.box);

    Ok(newnode)
  }

  /// Stamp a freshly-opened element with its source range as a
  /// `data-sourcepos` attribute, for the `--source-map` feature (issues
  /// #47/#92). The range comes from the construct currently being absorbed
  /// (`box_to_absorb`); the integer file `tag` is resolved through the
  /// document-level `sources` table (`state::source_tag`) so no path is
  /// inlined. See `docs/SOURCE_PROVENANCE.md` §0/§2.
  ///
  /// Math is kept **opaque** per the MVP scope: the `ltx:Math` wrapper is
  /// stamped, but its `ltx:XM*` MathML internals are skipped (the Marpa
  /// math parser has no locator awareness — §7 A.3). Only invoked when the
  /// source-map switch is on (the caller gates it).
  fn stamp_source_locator(&mut self, node: &Node, qname: &str) {
    // Math internals: stamp only the leaf token elements (`ltx:XMTok` — the
    // operators / identifiers / numbers) when token-locators gives them a real
    // located box locator (the math char's source origin). This is the per-token
    // in-equation provenance step (§7 A.3). The structural XM* (XMApp/XMDual/
    // XMArray) are rebuilt by the Marpa parser — created directly, not via
    // `open_element` — so they never reach here; the remaining digestion-built
    // wrappers (XMArg/XMHint/XMText/XMRef/XMWrap) stay opaque. The `data:sourcepos`
    // rides the XMTok element through the parser's restructuring (attribute on a
    // reparented node) and through the XMath→MathML XSLT.
    //
    // Gated at compile time: feature-OFF keeps math fully opaque (the MVP scope
    // and the golden's math-opacity assertion); only the token-locators build
    // exposes the located XMTok leaves.
    #[cfg(not(feature = "token-locators"))]
    if qname.starts_with("ltx:XM") {
      return;
    }
    #[cfg(feature = "token-locators")]
    if qname.starts_with("ltx:XM") && qname != "ltx:XMTok" {
      return;
    }
    // Read the pre-captured Copy locator — never re-borrow the box here.
    let Some(loc) = self.current_box_locator else {
      return;
    };
    // Skip locators with no real source position (default/synthetic).
    if loc.from_line == 0 {
      return;
    }
    // User-source only (§7.B): emit a navigable locator only into an editable
    // user document — `.tex`/`.ltx`, plus the bibliography sources `.bbl` (the
    // BibTeX-generated, but author-editable, list of `\bibitem`s) and `.bib`
    // (BibTeX database entries). All four are files the editor may legitimately
    // scroll into. This skips both synthetic default locators (whose source is
    // `…/locator.rs`, from `Locator::default()`'s `file!()`) and foreign
    // package/class files (`.sty`/`.cls`/…) — the editor must never scroll into
    // those. Foreign/unstamped elements inherit their nearest user-source
    // ancestor's range client-side (DOM walk-up). (MVP heuristic; a tracked
    // user-input set would be more precise.)
    let src = loc.get_source();
    let is_user_source = arena::with(src, |s| {
      let s = s.to_ascii_lowercase();
      s.ends_with(".tex") || s.ends_with(".ltx") || s.ends_with(".bbl") || s.ends_with(".bib")
    });
    if !is_user_source {
      return;
    }
    let tag = state::source_tag(src);
    // Emit in LaTeXML's `data:` namespace (`http://dlmf.nist.gov/LaTeXML/data`,
    // registered in `base_schema.rs:19`). The post XSLT's `copy_foreign_attributes`
    // path converts a `data:`-prefixed *foreign-namespaced* attribute to the HTML
    // `data-sourcepos` attribute (`LaTeXML-common.xsl`: `data:` prefix → `data-…`
    // when `USE_DATA_ATTRIBUTES` = true, i.e. HTML5). Faithful to Perl LaTeXML's
    // foreign-attribute convention; no XSLT change needed. The general namespaced-
    // attribute binding lives in `set_attribute` (shared with `aria:` etc.).
    let mut n = node.clone();
    let _ = self.set_attribute(&mut n, "data:sourcepos", &loc.to_sourcepos(tag));
  }

  /// Note: This closes the deepest open node of a given type.
  /// This can cause problems with auto-opened nodes, esp. ones for fontswitches!
  /// Since this is an "explicit request", we're currently skipping over those nodes,
  /// ie. we're automatically closing them, even if they're the same type as we're asking to
  /// close!!! This is kinda risky! Maybe we should try to request closing of specific nodes.
  pub fn close_element(&mut self, qname: &str) -> Result<Option<Node>> {
    Debug!(
      "document",
      "close_element",
      s!(
        "Close element {:?} at {:?}",
        qname,
        self.document.node_to_string(&self.node)
      )
    );
    let qsym = arena::pin(qname);
    self.close_text_internal()?;
    let mut node = self.node.clone();
    let mut cant_close = Vec::new();
    while node.get_type() != Some(NodeType::DocumentNode) {
      let t = get_node_qname(&node);
      // autoclose until node of same name BUT also close nodes opened' for font
      // switches!
      if t == qsym && !(t == pin!("ltx:text") && node.has_attribute("_fontswitch")) {
        break;
      }
      if !can_auto_close(&node) {
        cant_close.push(node.clone());
      }
      match node.get_parent() {
        Some(parent) => {
          node = parent;
        },
        None => break, // detached node — treat as not found
      }
    }

    if node.get_type() == Some(NodeType::DocumentNode) {
      // Didn't find $qname at all!!
      let qname_msg: String = match qname {
        "#PCDATA" => qname.to_owned(),
        _ => s!("</{qname}>"),
      };
      let message = s!(
        "Attempt to close {}, which isn't open. Currently in {}",
        qname_msg,
        self.get_insertion_context(None)?
      );
      Error!("malformed", qname, message);
      Ok(None)
    } else {
      // Found node.
      if !cant_close.is_empty() {
        // Intervening non-auto-closeable nodes!!
        let message = s!(
          "Closing tag {:?} whose open descendents do not auto-close. Descendants are {:?}",
          qname,
          cant_close
            .into_iter()
            .map(|n| n.get_name())
            .collect::<Vec<String>>()
            .join(",")
        );
        Error!("malformed", qname, message);
      }
      // So, now close up to the desired node.
      self.close_node_internal(&node)?;
      Ok(Some(node))
    }
  }

  // Check whether it is possible to open $qname at this point,
  // possibly by autoOpen'ing & autoClosing other tags.
  pub fn is_openable(&self, test_qname: &str) -> bool {
    let mut node_opt = Some(self.node.clone());
    let test_sym = arena::pin(test_qname);
    while let Some(node) = node_opt {
      let node_qname = get_node_qname(&node);
      if sym_can_contain_somehow(node_qname, test_sym).is_some() {
        return true;
      } else if !can_auto_close(&node) {
        return false; // could close, then check if parent can contain
      } else {
        node_opt = node.get_parent();
      }
    }
    false
  }

  /// Check whether it is possible to close each element in @tags,
  /// any intervening nodes must be autocloseable.
  /// returning the last Some(node) that would be closed if it is possible,
  /// otherwise None
  pub fn is_closeable<T: IntoVDQS>(&self, tags: T) -> Option<Node> {
    let mut tags: VecDeque<SymStr> = tags.into_vdqs();
    let mut node_opt = if self.node.get_type() == Some(NodeType::TextNode) {
      self.node.get_parent()
    } else {
      Some(self.node.clone())
    };
    while let Some(qname) = tags.pop_front() {
      'inner: loop {
        let node: &Node = match node_opt {
          None => break,
          Some(ref n) => n,
        };
        let node_type = node.get_type();
        if node_type == Some(NodeType::DocumentNode) || node_type.is_none() {
          return None;
        }
        let this_qname = get_node_qname(node);
        if this_qname == qname {
          break 'inner;
        }
        if !can_auto_close(node) {
          Debug!(
            "It was impossible to autoclose node: {:?}",
            self.document.node_to_string(node)
          );
          return None;
        }
        node_opt = node.get_parent();
      }
      if !tags.is_empty() {
        if let Some(node) = node_opt {
          node_opt = node.get_parent();
        }
      }
    }
    node_opt
  }

  // Close $qname, if it is closeable.
  pub fn maybe_close_element(&mut self, qname: &str) -> Result<Option<Node>> {
    if let Some(node) = self.is_closeable(qname) {
      self.close_node_internal(&node)?;
      Ok(Some(node))
    } else {
      Ok(None)
    }
  }

  /// Closes all nodes until $node becomes the current point.
  pub fn close_to_node(&mut self, node: &Node, ifopen: bool) -> Result<()> {
    let mut cant_close = Vec::new();
    let mut lastopen: Option<Node> = None;
    let mut n = self.node.clone();
    let mut n_type = n.get_type();
    // go up the tree from current node, till we find `node`
    while n_type != Some(NodeType::DocumentNode) && &n != node {
      if !can_auto_close(&n) {
        cant_close.push(n.clone());
      }
      lastopen = Some(n.clone());
      if let Some(p) = n.get_parent() {
        n = p;
        n_type = n.get_type();
      } else {
        break;
      }
    }
    if n_type == Some(NodeType::DocumentNode) {
      // Didn't find $node at all!!
      // Perl: suppress error when $ifopen is true
      if !ifopen {
        let message = s!("Attempt to close {:?}, which isn't open", node.get_name());
        arena::with(get_node_qname(node), |qname_str| {
          {
            Error!("malformed", qname_str, message)
          };
          Ok(())
        })?;
      }
    } else {
      // Found node.
      if !cant_close.is_empty() {
        // But found has intervening non-auto-closeable nodes!!
        let qname = get_node_qname(node);
        let message = s!(
          "Closing {:?} whose open descendents do not auto-close. Descendants are: {:?}",
          qname,
          cant_close
            .into_iter()
            .map(|n| n.get_name())
            .collect::<Vec<String>>()
            .join(",")
        );
        arena::with(qname, |qname_str| {
          {
            Error!("malformed", qname_str, message)
          };
          Ok(())
        })?;
      }
      if let Some(lastopen_node) = lastopen {
        self.close_node_internal(&lastopen_node)?;
      }
    }
    Ok(())
  }

  /// Closes all nodes until $node is closed.
  pub fn close_node(&mut self, node: &Node) -> Result<()> {
    self.close_node_with_strictness(true, node)
  }
  /// Only if needed/possible: closes all nodes until $node is closed
  pub fn maybe_close_node(&mut self, node: &Node) -> Result<()> {
    self.close_node_with_strictness(false, node)
  }

  pub fn close_node_with_strictness(&mut self, strict: bool, node: &Node) -> Result<()> {
    // Perl: my ($t, @cant_close) = (); ... while ((($t = $n->getType) != XML_DOCUMENT_NODE) ...
    let mut cant_close: Vec<Node> = Vec::new();
    let mut n = self.node.clone();
    let mut t = n.get_type(); // track walker node type, not target
    while t.is_some() && t != Some(NodeType::DocumentNode) && &n != node {
      if !can_auto_close(&n) {
        cant_close.push(n.clone());
      }
      match n.get_parent() {
        Some(parent) => {
          n = parent;
          t = n.get_type();
        },
        None => {
          t = None; // detached node — stop walking
        },
      }
    }

    if t == Some(NodeType::DocumentNode) || t.is_none() {
      // Didn't find $qname at all!!
      if strict {
        let qname = get_node_qname(node);
        arena::with(qname, |qname_str| {
          let message = s!(
            "Attempt to close {}, which isn't open. Currently in {:?}",
            qname_str,
            self.get_insertion_context(None)?
          );
          {
            Error!("malformed", qname_str, message)
          };
          Ok(())
        })?;
      }
    } else {
      // Found node.
      // Intervening non-auto-closeable nodes!!
      if !cant_close.is_empty() {
        model::with_node_qname(node, |qname| {
          let message = s!(
            "Closing {} whose open descendents do not auto-close. Descendents are {}",
            qname,
            cant_close
              .iter()
              .map(Node::get_name)
              .collect::<Vec<String>>()
              .join(", ")
          );
          if strict {
            Error!("malformed", qname, message);
          } else {
            Info!("malformed", qname, message);
          }
          Ok(())
        })?;
      }
      self.close_node_internal(node)?;
    }
    Ok(())
  }

  /// get the actions that should be performed on afterOpen or afterClose
  pub fn get_tag_action_list(
    &self,
    tag: SymStr,
    when: TagOptionName,
  ) -> Vec<TagConstructionClosure> {
    use self::tag::TagOptionName::*;
    // my ($p, $n) = (undef, $tag);
    // if ($tag =~ /^([^:]+):(.+)$/) {
    //   ($p, $n) = ($1, $2); }
    let mut when_early = None;
    let mut when_late = None;

    match when {
      AfterOpen => {
        when_early = Some(AfterOpenEarly);
        when_late = Some(AfterOpenLate);
      },
      AfterClose => {
        when_early = Some(AfterCloseEarly);
        when_late = Some(AfterCloseLate);
      },
      _ => {},
    };

    let tag_hash = state::get_tag_property(tag);
    let all_hash = state::get_tag_property(pin!("ltx:*"));

    let mut actions = Vec::new();
    // we have Rc<> around the closures, so cloning them is cheap - just another
    // pointer with a bumped up reference counter
    if let Some(when0) = when_early {
      actions.extend(tag_hash.get(&when0).cloned().unwrap_or_default());
      // ns_hash TODO
      actions.extend(all_hash.get(&when0).cloned().unwrap_or_default());
    }

    actions.extend(tag_hash.get(&when).cloned().unwrap_or_default());
    // ns_hash TODO
    actions.extend(all_hash.get(&when).cloned().unwrap_or_default());

    if let Some(when1) = when_late {
      actions.extend(tag_hash.get(&when1).cloned().unwrap_or_default());
      // ns_hash TODO
      actions.extend(all_hash.get(&when1).cloned().unwrap_or_default());
    }
    // return (
    //   (($v = $$taghash{$when0}) ? @$v : ()),
    //   (($v = $$nshash{$when0})  ? @$v : ()),
    //   (($v = $$allhash{$when0}) ? @$v : ()),
    //   (($v = $$taghash{$when})  ? @$v : ()),
    //   (($v = $$nshash{$when})   ? @$v : ()),
    //   (($v = $$allhash{$when})  ? @$v : ()),
    //   (($v = $$taghash{$when1}) ? @$v : ()),
    //   (($v = $$nshash{$when1})  ? @$v : ()),
    //   (($v = $$allhash{$when1}) ? @$v : ()),
    //   );
    actions
  }

  pub fn serialize_to_string(&self) -> String {
    // This line is to use libxml2's built-in serializer w/indentation heuristic.
    // Apparently, libxml2 is giving us "binary" or byte strings which we'd prefer
    // to have as text. return decode('UTF-8',
    // $self->getDocument->toString($format)); } This uses our own serializer
    // emulating libxml2's heuristic indentation.
    // This uses our own serializer with the correct schema-based indentation rules:
    // noindent_children=true when the element can contain #PCDATA per the schema.
    let result = self.serialize_aux(&self.document.as_node(), 0, false, false);
    // Trim trailing newline (the root element adds \n after </document>
    // but Perl doesn't include it)
    result.trim_end_matches('\n').to_string() + "\n"
  }

  /// We ought to try for something close to C14N (<http://www.w3.org/TR/xml-c14n>),
  /// but keep XML declaration, comments and don't convert empty elements.
  pub fn serialize_aux(
    &self,
    node: &Node,
    depth: usize,
    noindent: bool,
    heuristic: bool,
  ) -> String {
    let indent = "  ".repeat(depth);
    let mut serialized = String::new();

    match node.get_type() {
      Some(NodeType::DocumentNode) => {
        serialized.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        if let Some(child) = node.get_first_child() {
          let child_serialized = self.serialize_aux(&child, depth, noindent, heuristic);
          serialized.push_str(&child_serialized);
          let mut current_child = child;
          while let Some(sibling) = current_child.get_next_sibling() {
            let sibling_serialized = self.serialize_aux(&sibling, depth, noindent, heuristic);
            serialized.push_str(&sibling_serialized);
            current_child = sibling;
          }
        }
      },
      Some(NodeType::ElementNode) => {
        // Get the qualified name (prefix:localname) for namespace-prefixed elements
        let local_name = node.get_name();
        let tag = if let Some(ns) = node.get_namespace() {
          let prefix = ns.get_prefix();
          if prefix.is_empty() {
            local_name
          } else {
            s!("{}:{}", prefix, local_name)
          }
        } else {
          local_name
        };
        let children = node.get_child_nodes();
        let mut open_tag = s!("<{tag}");

        let nsnodes = node.get_namespace_declarations();
        for ns in nsnodes {
          let prefix = ns.get_prefix();
          let prefix_declaration = if prefix.is_empty() {
            s!("xmlns")
          } else {
            s!("xmlns:{}", prefix)
          };
          let href = ns.get_href();
          write!(open_tag, " {prefix_declaration}=\"{href}\"").ok();
        }

        let anodes = node.get_attributes();
        let mut anodes_keys: Vec<&String> = anodes.keys().collect();
        // Sort: xmlns:* declarations first (matching Perl's output order), then alphabetically
        anodes_keys.sort_by(|a, b| {
          let a_is_xmlns = a.starts_with("xmlns:");
          let b_is_xmlns = b.starts_with("xmlns:");
          match (a_is_xmlns, b_is_xmlns) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
          }
        });
        for key in anodes_keys {
          // Plain "id" is duplicated by libxml2 when xml:id is set.
          // Skip it for non-SVG elements (which use xml:id).
          // SVG elements use plain "id" (not xml:id).
          if key == "id" && !tag.starts_with("svg:") {
            continue;
          }
          // For SVG elements that have plain "id", skip the xml:id duplicate
          if key == "xml:id" && tag.starts_with("svg:") && node.get_attribute("id").is_some() {
            continue;
          }
          let key_sym = model::get_node_document_qname(&node.get_attribute_node(key).unwrap());
          let val_serialized = serialize_attr(&node.get_property(key).unwrap_or_default());
          arena::with(key_sym, |key_str| {
            write!(open_tag, " {key_str}=\"{val_serialized}\"")
          })
          .ok();
        }
        // HACK for xml:id for now, assuming last element.
        // SVG elements use plain "id" (not xml:id) — skip xml:id conversion for them.
        if anodes.contains_key("id") && !tag.starts_with("svg:") {
          let val_serialized = serialize_attr(&node.get_property("id").unwrap_or_default());
          write!(open_tag, " xml:id=\"{val_serialized}\"").ok();
        }

        let noindent_children: bool = if heuristic {
          // libxml2's heuristic: inline (noindent) if ANY direct child is a text node.
          // Crucially, this does NOT propagate the parent's noindent — each element
          // independently checks its own children for text nodes.
          children
            .iter()
            .any(|e| e.get_type() == Some(NodeType::TextNode))
        } else {
          // This is the "Correct" way to determine whether to add indentation
          let node_qname = get_node_qname(node);
          model::can_contain_sym(node_qname, pin!("#PCDATA"))
        };

        if !noindent {
          serialized.push_str(&indent)
        }
        serialized.push_str(&open_tag);
        // Perl serializes elements with children (including empty text nodes) as
        // <tag>...</tag>, and truly childless elements as <tag/>. Match this behavior.
        if !children.is_empty() {
          // with contents.
          serialized.push('>');
          if !noindent_children {
            serialized.push('\n');
          }
          for child in children {
            serialized.push_str(&self.serialize_aux(
              &child,
              depth + 1,
              noindent_children,
              heuristic,
            ));
          }
          if !noindent_children {
            serialized.push_str(&indent)
          }
          write!(serialized, "</{tag}>").ok();
        } else {
          // empty element.
          serialized.push_str("/>");
        }
        if !noindent {
          serialized.push('\n');
        }
      },
      Some(NodeType::TextNode) => {
        return serialize_string(&node.get_content());
      },
      Some(NodeType::PiNode) => {
        // should code this by hand, as well...
        if !noindent {
          serialized.push_str(&indent);
        }
        serialized.push_str(&self.document.node_to_string(node));
        if !noindent {
          serialized.push('\n');
        }
      },
      Some(NodeType::CommentNode) => {
        write!(
          serialized,
          "<!-- {}-->",
          serialize_string(&node.get_content())
        )
        .ok();
      },
      _ => {},
    }
    serialized
  }

  #[track_caller]
  pub fn set_node(&mut self, node: &Node) {
    // #217 diagnostic (TEMP): remember where the current node is being set,
    // so a later corrupt read can name this assignment site.
    LAST_SET_NODE_LOC.set(Some(std::panic::Location::caller()));
    // Perl Document.pm:setNode L74-87: if the candidate is a
    // DOCUMENT_FRAG_NODE, validate that it has exactly one child and
    // descend to that child. The original Rust port had this check
    // commented-out with a wrong-node-type marker (`DocumentNode`
    // instead of `DocumentFragNode`); revived with the correct enum.
    let mut chosen = node.clone();
    if chosen.get_type() == Some(NodeType::DocumentFragNode) {
      let children = chosen.get_child_nodes();
      // Wrap the Error!/note_status side-effects in an IIFE to swallow the
      // `Result<()>` the macro returns (it can early-return Err if the
      // MAX_ERRORS cap is hit). `set_node` returns `()` and is called from
      // 32 call-sites; threading `Result` through all of them is out of
      // scope for an audit-fix. The hot path stays the same; the rare
      // hit-the-cap case loses one escape attempt but the next Error!
      // anywhere else will trigger the cap regardless.
      let _ = (|| -> Result<()> {
        if children.len() > 1 {
          Error!(
            "unexpected",
            "multiple-nodes",
            "Cannot set insertion point to a DOCUMENT_FRAG_NODE"
          );
        } else if children.is_empty() {
          Error!(
            "unexpected",
            "empty-nodes",
            "Cannot set insertion point to an empty DOCUMENT_FRAG_NODE"
          );
        }
        Ok(())
      })();
      if let Some(first) = children.into_iter().next() {
        chosen = first;
      }
    }
    self.node = chosen;
    // #217 diagnostic (TEMP): catch a corrupt node being made current. If
    // this fires, the macOS corruption is upstream of THIS caller (it passed
    // a freed/garbage node); if it does NOT fire but a later read finds
    // `self.node` corrupt, the node was valid when set and freed afterwards.
    if !is_sane_current_node_type(&self.node.get_type()) {
      eprintln!(
        "[#217] set_node: self.node set to UNEXPECTED type {:?} by caller {}\n{}",
        self.node.get_type(),
        std::panic::Location::caller(),
        std::backtrace::Backtrace::force_capture()
      );
    }
  }

  // Internals
  /// Scan for RDFa attributes in the document and set the `prefix` attribute
  /// on the root element based on which RDFa prefixes are actually used.
  fn set_rdfa_prefixes(&mut self) {
    // Collect the RDFa prefix mapping from state
    let prefix_map: HashMap<String, String> = state::with_mapping_keys("RDFa_prefixes", |keys| {
      let mut map = HashMap::default();
      for key_sym in keys {
        let key_str = arena::to_string(key_sym);
        if let Some(stored) = state::lookup_mapping("RDFa_prefixes", &key_str) {
          map.insert(key_str, stored.to_string());
        }
      }
      map
    });
    if prefix_map.is_empty() {
      return;
    }

    let non_rdf_prefixes: HashSet<&str> = ["http", "https", "ftp"].iter().copied().collect();
    let rdf_term_attrs = [
      "about", "resource", "property", "typeof", "rel", "rev", "datatype",
    ];

    // Build XPath to find elements with any RDFa term attribute
    let xpath = format!(
      "descendant::*[{}]",
      rdf_term_attrs
        .iter()
        .map(|a| format!("@{a}"))
        .collect::<Vec<_>>()
        .join(" or ")
    );

    let mut used_prefixes: BTreeSet<String> = BTreeSet::new();

    let nodes = self.findnodes(&xpath, None);
    for node in &nodes {
      for attr_name in &rdf_term_attrs {
        if let Some(value) = node.get_attribute(attr_name) {
          for term in value.split_whitespace() {
            if let Some(colon_pos) = term.find(':') {
              let prefix = &term[..colon_pos];
              if !non_rdf_prefixes.contains(prefix) && prefix_map.contains_key(prefix) {
                used_prefixes.insert(prefix.to_string());
              }
            }
          }
        }
      }
    }

    if !used_prefixes.is_empty() {
      if let Some(mut root) = self.document.get_root_element() {
        let prefix_str = used_prefixes
          .iter()
          .map(|p| format!("{}: {}", p, prefix_map[p]))
          .collect::<Vec<_>>()
          .join(" ");
        let _ = root.set_attribute("prefix", &prefix_str);
      }
    }
  }

  pub fn insert_math_token(
    &mut self,
    text: &str,
    mut attributes: HashMap<String, String>,
    font_opt: Option<&Font>,
  ) -> Result<Node> {
    // Perf: avoid allocating the "role" String key unless the entry is missing.
    // HashMap::entry() takes an owned K, which forces allocation even on the
    // common path where "role" is already present.
    if !attributes.contains_key("role") {
      attributes.insert(String::from("role"), String::from("UNKNOWN"));
    }
    // Remove internal-only properties that should not become XML attributes.
    // In Perl, these are filtered by canHaveAttribute (model validation),
    // but we filter them explicitly here.
    attributes.remove("mode");
    attributes.remove("isMath");
    attributes.remove("cached_width");
    attributes.remove("cached_height");
    attributes.remove("cached_depth");
    // attributes.remove("stretchy");

    let is_space = attributes.contains_key("isSpace");
    let qname = if is_space {
      MATH_HINT_NAME
    } else {
      MATH_TOKEN_NAME
    };
    let cur_qname = get_node_qname(&self.node);
    let text = if is_space && !text.is_empty() && text.chars().all(|c| c.is_whitespace()) {
      "" // Make empty hint, of only spaces
    } else {
      text
    };
    if qname == MATH_TOKEN_NAME && cur_qname == pin!("ltx:XMTok") {
      // Already INSIDE a token!
      if !text.is_empty() {
        self.open_math_text_internal(text)?;
      }
    } else {
      let mut node = self.open_element(qname, Some(attributes), None)?;
      // let tbox  = $attributes{_box} || $LaTeXML::BOX;
      let font = match font_opt {
        Some(f) => f.clone(),
        None => match self.box_to_absorb {
          Some(ref tbox) => match tbox.get_font()? {
            Some(f) => f.into_owned(),
            None => Font::math_default(), // should never happen?
          },
          None => Font::math_default(), // should never happen?
        },
      };
      self.set_node_font(&mut node, &font)?;
      if let Some(ref digested) = self.box_to_absorb {
        // TODO: The Rc<Digested> node boxes still have some way to go until they are fully
        // ergonomic...
        self.set_node_box(&node, digested.clone());
      }
      if !text.is_empty() {
        self.open_math_text_internal(text)?;
      }
      self.close_node_internal(&node)?; // Should be safe.
    }
    Ok(self.node.clone())
  }

  /// Create a libxml2 comment node and attach it to `anchor` according to
  /// the supplied Placement. Called from `insert_comment`. Thin wrapper
  /// around the safe rust-libxml API (`Node::new_comment` +
  /// `add_child`/`add_prev_sibling`) — earlier versions of this method
  /// made direct FFI calls, which is now forbidden by the D3b policy.
  fn add_comment(document: &XmlDoc, anchor: &Node, comment_text: &str, placement: Placement_) {
    let Ok(mut comment) = Node::new_comment(comment_text, document) else {
      return;
    };
    let mut anchor = anchor.clone();
    match placement {
      Placement_::AppendChild => {
        let _ = anchor.add_child(&mut comment);
      },
      Placement_::PrevSibling => {
        let _ = anchor.add_prev_sibling(&mut comment);
      },
    }
  }

  /// Insert a new comment, or append to previous comment.
  /// Does NOT move the current insertion point to the Comment,
  /// but may move up past a text node.
  /// Perl: Document.pm lines 678-698
  pub fn insert_comment(&mut self, text: &str) -> Result<Node> {
    let trimmed = text.trim_end();
    let clean = DASHES_RE.replace_all(trimmed, "__");
    // Perl does NOT close the text node here — it uses getElement() to find
    // the nearest element, then inserts the comment relative to element children.
    // This preserves self.node as the current text node, so subsequent text
    // appends correctly and ligatures fire on the full text run.

    let comment_text = s!(" {} ", clean);

    if self.node.get_type() == Some(NodeType::DocumentNode) {
      Self::add_comment(
        &self.document,
        &self.node,
        &comment_text,
        Placement_::AppendChild,
      );
    } else if let Some(node) = self.get_element() {
      // Get the nearest element node (Perl: getElement)
      let prev = node.get_last_child();
      let prevtype = prev.as_ref().and_then(|n| n.get_type());

      if prevtype == Some(NodeType::CommentNode) {
        // Merge with previous comment
        if let Some(mut prev_comment) = prev {
          let existing = prev_comment.get_content();
          let merged = s!("{}\n     {} ", existing, clean);
          prev_comment.set_content(&merged).ok();
        }
      } else if prevtype == Some(NodeType::TextNode) {
        let prev_node = prev.unwrap();
        // If the node before the text is already a comment, just append new comment
        // Otherwise, insert before the text to avoid splitting text runs
        let before_text = prev_node.get_prev_sibling();
        let before_is_comment =
          before_text.as_ref().and_then(|n| n.get_type()) == Some(NodeType::CommentNode);

        if before_is_comment {
          Self::add_comment(
            &self.document,
            &node,
            &comment_text,
            Placement_::AppendChild,
          );
        } else {
          Self::add_comment(
            &self.document,
            &prev_node,
            &comment_text,
            Placement_::PrevSibling,
          );
        }
      } else {
        Self::add_comment(
          &self.document,
          &node,
          &comment_text,
          Placement_::AppendChild,
        );
      }
    }
    Ok(self.node.clone())
  }

  // **********************************************************************
  // Middle level, mostly public, API.
  // Handlers for various construction operations.
  // General naming: 'open' opens a node at current pos and sets it to current,
  // 'close' closes current node(s), inserts opens & closes, ie. w/o moving
  // current

  // Tricky: Insert some text in a particular font.
  // We need to find the current effective -- being the closest  _declared_ font,
  // (ie. it will appear in the elements attributes).  We may also want
  // to open/close some elements in such a way as to minimize the font switchiness.
  // I guess we should only open/close "text" elements, though.
  // [Actually, we'd like the user to _declare_ what element to use....
  //  I don't like having "text" built in here!
  //  AND, we've assumed that "font" names the relevant attribute!!!]

  pub fn open_text(&mut self, text: &str, font: &Font) -> Result<Option<Node>> {
    let node_type = self.node.get_type();
    // #217 robustness + diagnostic: if the current node is corrupt (a
    // freed/reused node read as an impossible libxml2 type — the macOS
    // residual), recover by skipping this insert instead of crashing
    // downstream in open_text_internal/can_contain. Report where the
    // corrupt node was last set so the source can be localized. On Linux
    // self.node is always a sane container here, so this never fires.
    if !is_sane_current_node_type(&node_type) {
      eprintln!(
        "[#217] open_text: self.node has UNEXPECTED type {:?} (corrupt current node); \
         last set_node at {:?}; skipping insert of {:?}.\n{}",
        node_type,
        LAST_SET_NODE_LOC.get(),
        text,
        std::backtrace::Backtrace::force_capture()
      );
      return Ok(None);
    }
    {
      // Ignore initial whitespace
      if (text.is_empty() || ONLY_SPACE_RE.is_match(text))
        && (node_type == Some(NodeType::DocumentNode)
          || (node_type == Some(NodeType::ElementNode) && !can_contain(&self.node, "#PCDATA")))
      {
        return Ok(None);
      }
    }
    if matches!(&font.family.as_deref(), Some("nullfont")) {
      return Ok(None);
    };
    Debug!(
      "document",
      "open_text",
      s!(
        "Insert text {:?} at {:?}",
        text,
        self.document.node_to_string(&self.node)
      )
    );

    // Get the desired font attributes, particularly the desired element
    // (usually ltx:text, but let Font override, eg for \emph)
    let declared_font = self.get_node_font(&self.node);
    let pending_declaration = font.relative_to(declared_font);
    let elementname = match pending_declaration.get("element") {
      Some((k, _v)) => k,
      None => FONT_ELEMENT_NAME,
    };
    let element_sym = arena::pin(elementname);
    // If not at document begin. And not appending text in same font.
    //
    // Defensive (issue #217): when `self.node` is a TextNode it should ALWAYS
    // have a parent element, so the original `self.node.get_parent().unwrap()`
    // was "safe" — but it was the macOS-only CI panic: under the macOS
    // allocator/TLS conditions `self.node` could be a DETACHED text node
    // (parent == None), a state never reached on Linux (this exact site is
    // hit ~3900×/sizes_test and always resolves to a live p/title/td/tag
    // parent). Match the None instead of unwrapping, and recover by skipping
    // this un-anchorable insert rather than crashing.
    let text_same_font = if node_type == Some(NodeType::TextNode) {
      match self.node.get_parent() {
        Some(parent) => font.distance(self.get_node_font(&parent)) == 0,
        None => {
          // TEMP diagnostic (#217): log + full backtrace so a macOS run
          // reveals which digestion op left `self.node` detached. Remove
          // once root-caused; the None guard itself stays.
          eprintln!(
            "[#217] open_text: current TextNode is DETACHED (get_parent()==None) — \
             content={:?} insert={:?}; skipping insert.\n{}",
            self.node.get_content(),
            text,
            std::backtrace::Backtrace::force_capture()
          );
          return Ok(None);
        },
      }
    } else {
      false
    };
    if node_type != Some(NodeType::DocumentNode) && !text_same_font
    {
      // then we'll need to do some open/close to get fonts matched.
      let node = self.close_text_internal()?; // Close text node, if any.
      let mut bestdiff = 99;
      let rc_node = Rc::new(node);
      let mut closeto: Rc<Node> = Rc::clone(&rc_node);
      let mut n: Rc<Node> = Rc::clone(&rc_node);
      while n.get_type() != Some(NodeType::DocumentNode) {
        let node_font = self.get_node_font(&n);
        let d = font.distance(node_font);
        if d < bestdiff {
          bestdiff = d;
          closeto = n.clone();
          if d == 0 {
            break;
          }
        }
        // Stop if not a font element, or if marked _noautoclose, or if
        // this is an explicit (non-fontswitch) text wrapper. A constructor-
        // opened `<ltx:text class='...'>` (e.g. `\uline{...}`) MUST NOT be
        // closed-out-of by a font-distance heuristic just because the parent
        // happens to score better — that produces an empty wrapper and
        // siblings the inner content (driver: 2402.16319 `\uline{\textbf{2}}`
        // inside `\sc` tabular). Only auto-opened fontswitch wrappers are
        // safe to walk past.
        if get_node_qname(&n) != element_sym
          || n.has_attribute("_noautoclose")
          || !n.has_attribute("_fontswitch")
        {
          break;
        }
        match n.get_parent() {
          Some(p) => n = Rc::new(p),
          None => break,
        }
      }

      // Move to best starting point for this text.
      if *closeto != *rc_node {
        self.close_to_node(&closeto, false)?;
      }
      if bestdiff > 0 {
        // Open if needed.
        self.open_element(
          elementname,
          Some(string_map!("_fontswitch" => "true", "_autoopened" => "true")),
          Some(font),
        )?;
      }
    }

    // Finally, insert the darned text.
    let outnode = self.open_text_internal(text)?;
    self.record_constructed_node(&outnode);
    Ok(Some(outnode))
  }

  pub fn close_text_internal(&mut self) -> Result<Node> {
    if self.node.get_type() == Some(NodeType::TextNode) {
      // Current node is text?
      let parent = self.node.get_parent().unwrap();
      let font = self.get_node_font(&parent);
      let ocontent = self.node.get_content();
      let mut content = Cow::Borrowed(&ocontent);
      state::with_value("TEXT_LIGATURES", |value_opt| {
        if let Some(Stored::VecDequeStored(ligatures)) = value_opt {
          for stored_ligature in ligatures.iter() {
            if let Stored::Ligature(ligature) = stored_ligature {
              if let Some(ref font_test) = ligature.font_test {
                if !(font_test)(font) {
                  continue; // if the font test fails, skip the ligature
                }
              }
              content = Cow::Owned((ligature.code.as_ref().unwrap())(&content));
            }
          }
        }
      });
      if *content != ocontent {
        self.node.set_content(&content)?;
      }
      self.node = parent.clone(); // Effectively closed (->setNode, but don't recurse)
      Ok(parent)
    } else {
      Ok(self.node.clone())
    }
  }

  /// Close `node`, and any current nodes below it.
  /// No checking! Use this when you've already verified that `node` can be closed.
  /// and, of course, `node` must be current or some ancestor of it!!!
  pub fn close_node_internal(&mut self, node: &Node) -> Result<()> {
    let closeto = match node.get_parent() {
      Some(p) => p,
      None => {
        // Node has been detached — nothing to close up to.
        return Ok(());
      },
    };
    let mut n = self.close_text_internal()?; // Close any open text node.
    while n.get_type() == Some(NodeType::ElementNode) {
      self.close_element_at(&mut n)?;
      self.auto_collapse_children(&mut n)?;
      if *node == n {
        break;
      }
      match n.get_parent() {
        Some(parent) => {
          n = parent;
        },
        None => {
          // Node was detached during close/collapse — bail out safely.
          break;
        },
      }
    }
    self.set_node(&closeto);
    Ok(())
  }

  /// Avoid redundant nesting of font switching elements:
  /// If we're closing a node that can take font switches and it contains
  /// a single FONT_ELEMENT_NAME node; pull it up.
  fn auto_collapse_children(&mut self, node: &mut Node) -> Result<()> {
    let qname = get_node_qname(node);
    if qname != pin!("ltx:_Capture_") {
      let mut c = node.get_child_nodes();
      // with single child, AND, $node can have all the attributes that the child has (but at least
      // "font") BUT, it isn"t being forced somehow
      if c.len() == 1
        && (get_node_qname(&c[0]) == pin!("ltx:text"))
        && model::can_have_attribute(qname, pin!("font"))
        && c[0]
          .get_attributes()
          .keys()
          .filter(|x| !x.starts_with('_'))
          .all(|v| {
            model::can_have_attribute(qname, arena::pin(v))
              && !(NON_MERGEABLE_ATTRIBUTES.contains(v.as_str()))
          })
        && !c[0].has_attribute("_force_font")
      {
        let c_first = c.pop().unwrap();
        let c_first_font = self.get_node_font(&c_first).clone();
        self.set_node_font(node, &c_first_font)?;
        for mut gc in c_first.get_child_nodes().into_iter() {
          gc.unlink();
          node.add_child(&mut gc)?;
          self.record_node_ids(node)?;
        }
        // Merge the attributes from the child onto $node
        self.merge_attributes(&c_first, node, None)?;
        self.remove_node(c_first);
      }
    }
    Ok(())
  }

  pub fn merge_attributes(
    &mut self,
    from: &Node,
    to: &mut Node,
    force: Option<&HashSet<&'static str>>,
  ) -> Result<()> {
    for (key, val) in from.get_attributes().iter() {
      // Skip internal attributes
      if key.starts_with('_') {
        continue;
      }
      // Normalize key: get_attributes() returns "id" for xml:id attributes.
      // Check both "xml:id" and bare "id" with XML namespace for the special case.
      let is_xml_id = key.as_str() == "xml:id"
        || (key.as_str() == "id"
          && from
            .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
            .is_some());
      let effective_key = if is_xml_id { "xml:id" } else { key.as_str() };
      let is_forced = force.is_some_and(|f| f.contains(effective_key));
      // Special case attributes
      if is_xml_id {
        // Use the replacement id. record_id_with_node returns a
        // deduplicated id when a DIFFERENT node already claims the
        // same one; must use the return value, not the original `val`.
        let to_has_id = to.has_attribute("xml:id")
          || to
            .get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace")
            .is_some();
        if !to_has_id || is_forced {
          self.unrecord_id(val);
          let deduped = self.record_id_with_node(val, to);
          to.set_attribute("xml:id", &deduped)?;
        }
      } else if MERGE_ATTRIBUTE_SPACEJOIN.contains(key.as_str()) {
        self.add_ss_values(to, key, val)?;
      } else if MERGE_ATTRIBUTE_SEMICOLONJOIN.contains(key.as_str()) {
        if let Some(existing) = to.get_attribute(key) {
          let merged = format!("{existing}; {val}");
          to.set_attribute(key, &merged)?;
        } else {
          to.set_attribute(key, val)?;
        }
      } else if MERGE_ATTRIBUTE_SUMLENGTH.contains(key.as_str()) {
        if let Some(val2) = to.get_attribute(key) {
          // Parse and sum pt values
          let v1 = val.trim_end_matches("pt").parse::<f64>().unwrap_or(0.0);
          let v2 = val2.trim_end_matches("pt").parse::<f64>().unwrap_or(0.0);
          to.set_attribute(key, &format!("{}pt", v1 + v2))?;
        } else {
          to.set_attribute(key, val)?;
        }
      } else if !to.has_attribute(key) || is_forced {
        // Else if attribute not present on $to, or if we specifically override it, just copy
        to.set_attribute(key, val)?;
      }
    }
    Ok(())
  }

  fn open_text_internal(&mut self, text: &str) -> Result<Node> {
    if text.is_empty() {
      return Ok(self.node.clone());
    }
    // Sibling guard to open_math_text_internal: libxml's append_text uses
    // CString and panics on embedded NULs (forbidden in XML text per spec).
    // Strip them up-front so all downstream append_text calls are safe.
    if text.contains('\0') {
      let cleaned: String = text.chars().filter(|c| *c != '\0').collect();
      return self.open_text_internal(&cleaned);
    }
    if self.node.get_type() == Some(NodeType::TextNode) {
      // current node already is a text node.
      Debug!(
        "document",
        "open_text_internal",
        s!(
          "Appending text {:?} to {:?}",
          text,
          self.document.node_to_string(&self.node)
        )
      );

      let parent = self.node.get_parent().unwrap();
      if self.box_to_absorb.is_some() && parent.get_attribute("_autoopened").is_some() {
        // Perl L1136-1137: appendNodeBox to accumulate boxes for autoopened elements
        let bta = self.box_to_absorb.clone().unwrap();
        self.append_node_box(&parent, &bta);
      }
      self.node.append_text(text)?;
    }
    // Perl lines 1139-1144: if lastChild is a comment node and its previous sibling is
    // a text node, swap them to avoid splitting text runs, then recurse.
    // This avoids libxml text-node merging which would bypass ligature processing.
    else if self.swap_comment_text_if_needed(text)? {
      // Handled by recursive call
    } else if HAS_NONSPACE_RE.is_match(text) || can_contain(&self.node, "#PCDATA") {
      // or text allowed here
      let mut point = self.find_insertion_point("#PCDATA", None)?;
      // Perl L1149-1150: appendNodeBox for autoopened insertion points
      if self.box_to_absorb.is_some() && point.get_attribute("_autoopened").is_some() {
        let bta = self.box_to_absorb.clone().unwrap();
        self.append_node_box(&point, &bta);
      }
      Debug!(
        "document",
        "open_text_internal",
        s!(
          "Inserting text node for {:?} into {:?}",
          text,
          self.document.node_to_string(&point)
        )
      );
      let mut node = Node::new_text(text, &self.document)?;
      point.add_child(&mut node)?;
      if node.get_type().is_none() {
        // Extremely important note! the Rust wrapper `add_child` follows `xmlAddChild` strictly,
        // so in a case where adjacent text nodes are added, libxml will **MERGE** them leading to
        // `node`'s underlying pointer disappearing.
        // Thus - we check if the node is gone - and use its parent instead if so.
        self.set_node(&point);
      } else {
        self.set_node(&node);
      }
    }
    Ok(self.node.clone())
  }

  /// Perl lines 1139-1144: Avoid splitting text runs across comments.
  /// If the current node's lastChild is a comment and the previous sibling is a text node,
  /// swap them so the text node is last, set it as current, and recurse to append new text.
  /// Returns true if the swap+append was performed, false otherwise.
  fn swap_comment_text_if_needed(&mut self, text: &str) -> Result<bool> {
    if self.node.get_type() != Some(NodeType::ElementNode) {
      return Ok(false);
    }
    if let Some(last_child) = self.node.get_last_child() {
      if last_child.get_type() == Some(NodeType::CommentNode) {
        if let Some(mut prev_text) = last_child.get_prev_sibling() {
          if prev_text.get_type() == Some(NodeType::TextNode) {
            // Swap: move text node after comment node
            let mut comment_node = last_child;
            comment_node.add_next_sibling(&mut prev_text)?;
            // Set current node to the text node and recurse
            self.set_node(&prev_text);
            self.open_text_internal(text)?;
            return Ok(true);
          }
        }
      }
    }
    Ok(false)
  }

  /// Perl: appendNodeBox — when material is added to an autoopened element,
  /// accumulate the record of boxes that created the node.
  /// Propagates up through autoopened ancestors.
  fn append_node_box(&mut self, node: &Node, thisbox: &Digested) {
    let mut node = node.clone();
    loop {
      let origbox = self.get_node_box(&node);
      if let Some(ref orig) = origbox {
        // Perl: ($box eq $origbox) || ($box eq ($origbox->unlist)[-1]) → skip (dedup)
        // Use pointer identity on the Rc-wrapped DigestedData
        let same_as_orig = std::ptr::eq(
          thisbox.data() as *const DigestedData,
          orig.data() as *const DigestedData,
        );
        let same_as_last = if !same_as_orig {
          if let DigestedData::List(ref list) = orig.data() {
            list
              .borrow()
              .boxes
              .last()
              .map(|b| {
                std::ptr::eq(
                  thisbox.data() as *const DigestedData,
                  b.data() as *const DigestedData,
                )
              })
              .unwrap_or(false)
          } else {
            false
          }
        } else {
          false
        };
        if !same_as_orig && !same_as_last {
          // Perl: List($origbox, $box, mode => $origbox->getProperty('mode'))
          let mode = orig.get_property("mode").and_then(|m| {
            if let Stored::String(s) = m.as_ref() {
              Some(*s)
            } else {
              None
            }
          });
          let mut new_list = List::new(vec![orig.clone(), thisbox.clone()]);
          if let Some(mode_str) = mode {
            new_list.properties.insert("mode", Stored::String(mode_str));
          }
          self.set_node_box(&node, new_list.into());
        }
      } else {
        self.set_node_box(&node, thisbox.clone());
      }
      // Propagate to autoopened ancestors
      match node.get_parent() {
        Some(parent)
          if parent.get_type() == Some(NodeType::ElementNode)
            && parent.get_attribute("_autoopened").is_some() =>
        {
          node = parent;
        },
        _ => break,
      }
    }
  }

  // Question: Why do I have math ligatures handled within openMathText_internal,
  // but text ligatures handled within closeText_internal ???

  /// Needed externally only for the binding generation
  fn open_math_text_internal(&mut self, text: &str) -> Result<Node> {
    // And if there's already text???
    let mut node = self.node.clone();
    // my $font = $self->getNodeFont($node);
    // libxml's append_text uses CString and panics on embedded NULs. NUL is
    // forbidden in XML text per spec anyway, so strip it. Witness:
    // astro-ph0202376 (a paper that produces math tokens with \char0 / NUL
    // bytes embedded in their content). Matches Perl's libxml behavior which
    // silently drops NULs in text content.
    if text.contains('\0') {
      let cleaned: String = text.chars().filter(|c| *c != '\0').collect();
      node.append_text(&cleaned)?;
    } else {
      node.append_text(text)?;
    }
    // print STDERR "Trying Math Ligatures at \"$string\"\n";
    if !state::get_nomathparse_flag() {
      self.apply_math_ligatures(&mut node)?;
    }
    Ok(node)
  }

  // New strategy (but inefficient): apply ligatures until one succeeds,
  // then remove it, and repeat until ALL (remaining) fail.
  fn apply_math_ligatures(&mut self, node: &mut Node) -> Result<()> {
    let checked_out_ligatures = state::checkout_value("MATH_LIGATURES");
    if let Some(Stored::VecDequeStored(ref stored_ligatures)) = checked_out_ligatures {
      let mut ligatures = stored_ligatures.iter().collect::<VecDeque<_>>();
      while !ligatures.is_empty() {
        let mut matched = false;
        let mut next_ligatures = VecDeque::new();
        while !ligatures.is_empty() {
          let ligature_stored = ligatures.pop_front().unwrap();
          if let Stored::Ligature(ligature) = ligature_stored {
            if self.apply_math_ligature(node, ligature)? {
              next_ligatures.extend(ligatures.drain(..));
              matched = true;
              break;
            }
          } else {
            next_ligatures.push_back(ligature_stored);
          }
        }
        ligatures = next_ligatures;
        if !matched {
          if let Some(value) = checked_out_ligatures {
            state::checkin_value("MATH_LIGATURES", value);
          }
          return Ok(());
        }
      }
    }
    if let Some(value) = checked_out_ligatures {
      state::checkin_value("MATH_LIGATURES", value);
    }
    Ok(())
  }

  /// Apply ligature operation to `node`, presumed the last insertion into it's parent(?)
  fn apply_math_ligature(&mut self, node: &mut Node, ligature: &Ligature) -> Result<bool> {
    if let Some((nmatched, newstring, attr)) = (ligature.matcher.as_ref().unwrap())(self, node)? {
      let mut boxes = VecDeque::new();
      boxes.push_front(self.get_node_box(node).unwrap());
      node.get_first_child().unwrap().set_content(&newstring)?;
      for _idx in 0..nmatched - 1 {
        let remove = node.get_prev_sibling().unwrap();
        boxes.push_front(self.get_node_box(&remove).unwrap());
        self.remove_node(remove);
      }
      // This fragment replaces the node's box by the composite boxes it replaces
      // HOWEVER, this gets things out of sync because parent lists of boxes still
      // have the old ones.  Unless we could recursively replace all of them, we'd better skip
      // it(??)
      if boxes.len() > 1 {
        // TODO: Cloning boxes is BAD. What is a better model?
        let mut list = List::new(boxes.into_iter().collect::<Vec<_>>());
        list.mode = Some(TexMode::Math);
        self.set_node_box(node, list.into());
      }
      for (key, value_opt) in attr.sorted_each() {
        if let Some(value) = value_opt {
          node.set_attribute(key, value)?;
        } else {
          node.remove_attribute(key)?;
        }
      }
      Ok(true)
    } else {
      Ok(false)
    }
  }

  /// Note that a box has been absorbed creating `node`;
  /// This does book keeping so that we can return the sequence of nodes
  /// that were added by absorbing material.
  pub fn record_constructed_node(&mut self, node: &Node) {
    // if ((defined $LaTeXML::RECORDING_CONSTRUCTION)    // If we're recording!
    let should_push = match self.constructed_nodes.last() {
      // and this node isn't already recorded
      None => true,
      Some(last_node) => last_node != node,
    };
    if should_push {
      self.constructed_nodes.push(node.clone());
    }
  }

  pub fn filter_deletions(&self, nodes: Vec<Node>) -> Vec<Node> {
    // This test seems to successfully determine inclusion,
    // without requiring the (dangerous? & dubious?) unbindNode to be used.
    if let Some(root) = self.document.get_root_element() {
      nodes
        .into_iter()
        .filter(|node| xml::is_descendant_or_self(node, &root))
        .collect()
    } else {
      Vec::new()
    }
  }

  /// Given a list of nodes such as from ->absorb,
  /// filter out all the nodes that are children of other nodes in the list.
  pub fn filter_children(&self, mut nodes: Vec<Node>) -> Vec<Node> {
    if nodes.is_empty() {
      Vec::new()
    } else {
      let mut new = vec![nodes.remove(0)];
      for node in nodes {
        if new
          .iter()
          .all(|other| !xml::is_descendant_or_self(&node, other))
        {
          new.push(node)
        }
      }
      new
    }
  }

  //**********************************************************************
  // Low level internal interface

  /// Return a string indicating the path to the current insertion point in the document.
  /// if $levels is defined, show only that many levels
  pub fn get_insertion_context(&self, levels_opt: Option<usize>) -> Result<String> {
    let mut levels = match levels_opt {
      None => {
        // Default depth is based on verbosity
        if state::current_verbosity() <= 1 {
          Some(5)
        } else {
          None
        }
      },
      Some(t) => Some(t),
    };
    let mut node = self.node.clone();
    let node_type = node.get_type();
    if node_type != Some(NodeType::TextNode)
      && node_type != Some(NodeType::ElementNode)
      && node_type != Some(NodeType::DocumentNode)
    {
      let message = s!(
        "Insertion point is not an element, document or text: {:?}",
        self.document.node_to_string(&node)
      );
      Error!("internal", "context", message);
      return Ok(String::new());
    }
    // Build a context path like "<ltx:document><ltx:section><ltx:p>" by walking
    // ancestors and prepending each element's qname. Mirrors Perl's
    // `Stringify($node)` chain with a depth cap from `levels_opt`.
    //
    // Cap each qname at 80 chars. Pathological inputs (e.g. xy-pic
    // emitting unparsed `\fontdimen 17 \cmr10 at NNsp` strings that
    // become element names through a sequence of recovery errors)
    // can produce multi-MB qnames; walking 5+ ancestors and
    // `format!`-ing each yields a 3.25 GB allocation request that
    // OOM-kills the worker. Truncating preserves the diagnostic
    // signal ("first 80 chars + …") without the unbounded growth.
    // Witness papers: math0203082, math0402448 (R35.B, sandbox).
    const QNAME_CAP: usize = 80;
    let truncate_qname = |qname: &str| -> String {
      if qname.len() <= QNAME_CAP {
        qname.to_string()
      } else {
        let mut s: String = qname.chars().take(QNAME_CAP).collect();
        s.push('…');
        s
      }
    };
    let qn_for = |n: &Node| -> String {
      match n.get_type() {
        Some(NodeType::ElementNode) => {
          with_node_qname(n, |qname| format!("<{}>", truncate_qname(qname)))
        },
        Some(NodeType::TextNode) => "#text".to_string(),
        Some(NodeType::DocumentNode) => "#document".to_string(),
        _ => "?".to_string(),
      }
    };
    let mut path = qn_for(&node);
    while let Some(parent_node) = node.get_parent() {
      node = parent_node;
      if let Some(levels_val) = levels {
        levels = Some(levels_val - 1);
        if levels_val <= 1 {
          path = format!("...{path}");
          break;
        }
      }
      path = format!("{}{}", qn_for(&node), path);
    }
    Ok(path)
  }

  /// Find the node where an element with qualified name `qname` can be inserted.
  /// This will move up the tree (closing auto-closable elements),
  /// or down (inserting auto-openable elements), as needed.
  pub fn find_insertion_point(
    &mut self,
    qname: &str,
    has_opened_opt: Option<SymStr>,
  ) -> Result<Node> {
    let qsym = arena::pin(qname);
    self.find_insertion_point_qsym(qsym, has_opened_opt)
  }

  pub fn find_insertion_point_qsym(
    &mut self,
    qsym: SymStr,
    has_opened_opt: Option<SymStr>,
  ) -> Result<Node> {
    self.close_text_internal()?; // Close any current text node.
    let cur_qname = get_node_qname(&self.node);
    // If `qname` is allowed at the current point, we're done.
    if can_contain_qsym(cur_qname, qsym) {
      return Ok(self.node.clone());
    // Else, if we can create an intermediate node that accepts $qname, we'll do
    // that.
    } else if let Some(inter) = can_contain_indirect(cur_qname, qsym) {
      if (inter != qsym) && (inter != cur_qname) {
        // TODO: can we avoid the clone here? there is a mutability conflict...
        let node_font = self.get_node_font(&self.node).clone();
        // TODO: avoid this clone?
        let inter_string = arena::to_string(inter);
        self.open_element(
          &inter_string,
          Some(string_map!("_autoopened" => "true")),
          Some(&node_font),
        )?;
        // And retry insertion (should work now).
        return self.find_insertion_point_qsym(qsym, Some(inter));
      }
    }
    if let Some(has_opened) = has_opened_opt {
      // out of options if already inside an auto-open chain
      let message: String =
        arena::with2(has_opened, cur_qname, |has_opened_str, cur_qname_str| {
          Ok::<String, crate::common::error::Error>(format!(
            "failed auto-open through <{}> at inadmissible <{}>. Currently in {}",
            has_opened_str,
            cur_qname_str,
            self.get_insertion_context(None)?
          ))
        })?;
      Error!("malformed", arena::to_string(qsym), message);
      Ok(self.node.clone()) // But we'll do it anyway, unless Error => Fatal.
    } else {
      // Now we're getting more desparate...
      // Check if we can auto close some nodes, and _then_ insert the `qname`.
      let mut node = self.node.clone();
      let mut close_to = None;
      while (node.get_type() != Some(NodeType::DocumentNode)) && can_auto_close(&node) {
        let parent_opt = node.get_parent();
        let parent_name = match parent_opt {
          None => pin!(""),
          Some(ref p) => get_node_qname(p),
        };
        if sym_can_contain_somehow(parent_name, qsym).is_some() {
          close_to = Some(node);
          break;
        }
        node = match parent_opt {
          Some(p) => p,
          None => break,
        };
      }
      if let Some(close_to_node) = close_to {
        self.close_node_internal(&close_to_node)?; // Close the auto closeable nodes.
        self.find_insertion_point_qsym(qsym, None) // Then retry, possibly w/auto open's
      } else {
        // Cascading-rejection suppression (2026-05-01): when a math leaf
        // element (`<ltx:XMTok>`) tries to insert into a text-mode
        // container (`<ltx:p>`/`<ltx:text>`), it's almost always a
        // cascade from a previously-rejected math wrapper (XMApp /
        // XMDual) — Perl emits the wrapper's rejection error but
        // doesn't continue to log per-child cascade errors. Mirror
        // that to drop the redundant noise. The Δ=2 witnesses are
        // hep-th0101146 (`$$ ... \end{equation}` mismatch) and
        // nlin0211024 (`${\mbox M}^{...}$$` inside `\begin{center}`).
        // We still return self.node.clone() so the caller proceeds
        // (the XMTok still gets inserted illegally, but the schema
        // validator will reject the whole math construct on
        // serialization anyway — the noise was purely diagnostic).
        let qsym_str = arena::to_string(qsym);
        let cur_str = arena::to_string(cur_qname);
        let is_math_leaf = qsym_str == "ltx:XMTok" || qsym_str == "ltx:XMArg";
        // `ltx:emph` added 2026-05-01 after math0010241 triage:
        // 13 XMTok-in-emph + 1 (Building line) cascade noise drops
        // Rust from 33 → 19, exact parity with Perl=19.
        let is_text_container =
          cur_str == "ltx:p" || cur_str == "ltx:text" || cur_str == "ltx:emph";
        if is_math_leaf && is_text_container {
          // Cascading rejection — skip the error log (Perl-faithful).
          return Ok(self.node.clone());
        }
        // Didn't find a legit place.
        let message = arena::with2(cur_qname, qsym, |cur_qname_str, qname| {
          s!(
            "{:?} isn't allowed in <{}>\n{}",
            qname,
            cur_qname_str,
            Backtrace::capture()
          )
        });
        //"Currently in " self.getInsertionContext());
        Error!("malformed", arena::to_string(qsym), message);

        // But we'll do it anyway, unless Error => Fatal.
        Ok(self.node.clone())
      }
    }
  }

  fn get_insertion_candidates(&self, node: &Node) -> Vec<Node> {
    let mut nodes: Vec<Node> = Vec::new();
    // Check the current element FIRST, then build list of candidates.
    let first = if node.get_type() == Some(NodeType::TextNode) {
      Cow::Owned(node.get_parent().unwrap())
    } else {
      Cow::Borrowed(node)
    };
    let is_capture = first.get_name() == "_Capture_";

    if first.get_type() != Some(NodeType::DocumentNode) && !is_capture {
      nodes.push(first.clone().into_owned());
    }

    // Collect previous siblings, if node is a text node.
    let mut element_node_opt: Option<Cow<Node>> = if node.get_type() == Some(NodeType::TextNode) {
      let mut current_opt = Some(Cow::Borrowed(node));
      while let Some(current) = current_opt {
        current_opt = current.get_prev_sibling().map(Cow::Owned);
        if current.get_name() == "_Capture_" {
          nodes.extend(xml::element_nodes(&current));
        } else {
          nodes.push(current.into_owned());
        }
      }
      node.get_parent().map(Cow::Owned)
    } else {
      Some(Cow::Borrowed(node))
    };
    // Now collect (element) node & ancestors
    while let Some(element_node) = element_node_opt {
      element_node_opt = element_node.get_parent().map(Cow::Owned);
      let node_type = element_node.get_type();
      if node_type.is_none() || node_type == Some(NodeType::DocumentNode) {
        break;
      }
      if element_node.get_name() == "_Capture_" {
        nodes.extend(xml::element_nodes(&element_node));
      } else {
        nodes.push(element_node.into_owned());
      }
    }
    if is_capture {
      nodes.push(first.into_owned());
    }

    nodes
  }

  pub fn node_set_attribute(&mut self, key: &str, value: &str) -> Result<()> {
    if value.is_empty() {
      return Ok(()); // skip if empty
    }
    if key == "xml:id" {
      // If it's an ID attribute
      let recorded = self.record_id(value); // Do id book keeping

      // TODO: Need to improve Namespace ergonomics, also in rust-libxml
      // let node_ns = self
      //   .document
      //   .get_root_element()
      //   .unwrap()
      //   .get_namespace_declarations()
      //   .into_iter()
      //   .find(|ns| ns.get_href() == XML_NS)
      //   .unwrap_or_else(|| {
      //     node
      //       .get_namespace_declarations()
      //       .into_iter()
      //       .find(|ns| ns.get_href() == XML_NS)
      //       .unwrap_or_else(|| {
      //         Namespace::new(
      //           "xml",
      //           &XML_NS.to_string().clone(),
      //           &mut self.document.get_root_element().unwrap(),
      //         ).unwrap_or_else(|_| {
      //           panic!(
      //             "Could not set NS for {:?}\n\n at \n\n {:?}",
      //             self.document.node_to_string(node),
      //             self.document.to_string(true)
      //           )
      //         })
      //       })
      //   });

      self.node.set_attribute("xml:id", &recorded)?; // and bypass all ns stuff
    } else if !key.contains(':') {
      // No colon; no namespace (the common case!)
      // Ignore attributes not allowed by the model,
      // but accept "internal" attributes.
      let qname = get_node_qname(&self.node);
      if key.starts_with('_') || model::can_have_attribute(qname, arena::pin(key)) {
        self.node.set_attribute(key, value)?
      };
    } else {
      // Namespaced attributes: set directly for now.
      // TODO: proper namespace prefix resolution via model->decodeQName
      self.node.set_attribute(key, value)?;
      //     else {
      //       node.setAttributeNS($ns, "$prefix:$name" => $value); } }
      //   else {
      //     node.setAttribute($name => $value); } }
    } // redundant case...
    Ok(())
  }
  pub fn node_get_attribute(&mut self, name: &str) -> Option<String> {
    self.node.get_attribute(name)
  }
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Document surgery (?)
  // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // The following carry out DOM modification but NOT relative to any current
  // insertion point (eg self.node), but rather relative to nodes specified
  // in the arguments.

  // Set any allowed attribute on a node, decoding the prefix, if any.
  // Also records, and checks, any id attributes.
  // [xml:id and namespaced attributes are always allowed]
  pub fn set_attribute(&mut self, node: &mut Node, key: &str, value: &str) -> Result<()> {
    if value.is_empty() {
      return Ok(()); // skip if empty
    }
    // Perl: setAttribute checks canHaveAttribute before setting.
    // Accept internal attributes (starting with _), namespaced (containing :), and model-allowed.
    if !key.starts_with('_') && !key.contains(':') && key != "id" {
      let qname = get_node_qname(node);
      if !model::can_have_attribute(qname, arena::pin(key)) {
        return Ok(()); // silently skip attributes not allowed by schema
      }
    }
    if key == "xml:id" || key == "id" {
      // Perl: only matches 'xml:id' literally, but our constructors use "id" for
      // xml:id. For SVG elements, keep plain "id" (SVG spec uses id, not xml:id).
      let node_qname = get_node_qname(node);
      let is_svg = arena::with(node_qname, |s| s.starts_with("svg:"));
      if is_svg && key == "id" {
        // SVG elements use plain id, not xml:id (matching Perl behavior)
        node.set_attribute(key, value)?;
      } else {
        // LaTeXML elements: always use xml:id.
        // record_id_with_node detects duplicates and returns a
        // deduplicated id when a DIFFERENT node already claims the
        // same id. Previously we discarded that return value and
        // wrote the original `value`, which meant libxml2 saw two
        // nodes with the same xml:id. The post-processing scan's
        // libxml2 idHash lookups then ran O(n²) on any document
        // with enough duplicates (see 1106.1389 / KNOWN_PERL_ERRORS
        // #13: 14 duplicate-id sites from \addtocounter{equation}{-1}
        // + \subequations inside a \newtheorem[equation] theorem).
        let deduped = self.record_id_with_node(value, node);
        node.set_attribute("xml:id", &deduped)?;
      }
    } else if !key.contains(':') {
      // No colon; no namespace (the common case!)
      // Note: Full model validation (can_have_attribute) is done in node_set_attribute.
      // Here we just set the attribute, since the caller is responsible for filtering.
      node.set_attribute(key, value)?;
    } else {
      // Namespaced attribute (`prefix:local`). Mirror Perl
      // `Core/Document.pm::setAttribute`, whose `getDocumentNamespacePrefix($ns, 1)`
      // *promotes* the prefix's namespace to a document namespace on first use.
      // That lets finalize's `apply_document_namespace_declarations` declare
      // `xmlns:prefix` on the root, so the prefixed attribute resolves into its
      // namespace on serialization and the post XSLT can copy it (e.g.
      // `data:sourcepos` → `data-sourcepos`). Without the promotion a code-only
      // namespace (like `data`, used by `--source-map`) is emitted unbound and
      // dropped. General over any registered prefix — implements the decodeQName
      // TODO. (`aria`/schema namespaces are already document namespaces, so the
      // re-registration is an idempotent no-op for them.)
      if let Ok((Some(ns_uri), _local)) = model::decode_qname(key) {
        let prefix = key.split(':').next().unwrap_or("");
        model::register_document_namespace(prefix, Some(&ns_uri));
      }
      node.set_attribute(key, value)?;
    }
    // ... TODO: continue (see Perl)
    Ok(())
  }

  pub fn add_ss_values(&mut self, node: &mut Node, key: &str, values_str: &str) -> Result<()> {
    // $values = $values->toAttribute if ref $values;
    if !values_str.is_empty() {
      // Skip if `empty'; but 0 is OK!
      let mut values: Vec<&str> = values_str.split_whitespace().collect();
      if let Some(oldvalues) = node.get_attribute(key) {
        // previous values?
        let mut old: Vec<&str> = oldvalues.split_whitespace().collect();
        for new in values {
          if old.iter().all(|v| *v != new) {
            old.push(new);
          }
        }
        old.sort_unstable();
        self.set_attribute(node, key, &old.join(" "))?;
      } else {
        values.sort_unstable();
        self.set_attribute(node, key, &values.join(" "))?;
      }
    }
    Ok(())
  }

  pub fn add_class(&mut self, node: &mut Node, class: &str) -> Result<()> {
    self.add_ss_values(node, "class", class)
  }

  /// Remove space-separated values from an attribute.
  /// Perl: sub removeSSValues (Document.pm lines 1423-1437)
  pub fn remove_ss_values(&mut self, node: &mut Node, key: &str, values: &str) {
    let to_remove: Vec<&str> = values.split_whitespace().collect();
    if to_remove.is_empty() {
      return;
    }
    if let Some(current) = node.get_attribute(key) {
      let updated: Vec<&str> = current
        .split_whitespace()
        .filter(|v| !to_remove.contains(v))
        .collect();
      if updated.is_empty() {
        let _ = node.remove_attribute(key);
      } else {
        let mut sorted = updated;
        sorted.sort_unstable();
        node
          .set_attribute(key, &sorted.join(" "))
          .unwrap_or_default();
      }
    }
  }

  /// Remove CSS class from element.
  /// Perl: sub removeClass (Document.pm lines 1439-1442)
  pub fn remove_class(&mut self, node: &mut Node, class: &str) {
    self.remove_ss_values(node, "class", class);
  }

  /// Float to a node that can accept the given attribute.
  /// Returns the previous node so it can be restored after setting the attribute.
  /// Perl: sub floatToAttribute (Document.pm lines 1080-1092)
  pub fn float_to_attribute(&mut self, key: &str) -> Option<Node> {
    let candidates = self.get_insertion_candidates(&self.node);
    for candidate in candidates {
      let qname_sym = get_node_qname(&candidate);
      if sym_can_have_attribute(qname_sym, arena::pin(key)) {
        let savenode = self.node.clone();
        self.set_node(&candidate);
        return Some(savenode);
      }
    }
    Warn!(
      "malformed",
      key,
      s!("No open node can get attribute '{}'", key)
    );
    None
  }

  /// Check if a node is currently open (i.e., is or contains the current node).
  /// Perl: sub isOpen (Document.pm lines 1998-2006)
  pub fn is_open(&self, node: &Node) -> bool {
    if *node == self.node {
      return true;
    }
    for child in node.get_child_nodes() {
      if self.is_open(&child) {
        return true;
      }
    }
    false
  }

  //**********************************************************************
  // Association of nodes and ids (xml:id)

  /// Records the association of the current Document `node` with the `id`,
  /// which should be the `xml:id` attribute of the `node`.
  /// Usually this association will be maintained by the methods
  /// that create nodes or set attributes.
  fn record_id(&mut self, id: &str) -> String {
    let needs_modify = if let Some(prev) = self.idstore.get(id) {
      // Whoops! Already assigned!!!
      // Can we recover?
      self.node != *prev
    } else {
      false
    };
    let final_id = if needs_modify {
      let badid = id.to_string();
      let new_id = self.modify_id(badid);
      Info!(
        "malformed",
        "id",
        s!("Duplicated attribute xml:id. Using id='{}'", new_id)
      );
      new_id
    } else {
      id.to_string()
    };
    self.idstore.insert(final_id.clone(), self.node.clone());
    final_id
  }

  /// Records the association of the given `node` with the `id`,
  /// which should be the `xml:id` attribute of the `node`.
  /// Usually this association will be maintained by the methods
  /// that create nodes or set attributes.
  fn record_id_with_node(&mut self, id: &str, node: &Node) -> String {
    let prev_opt = if let Some(prev) = self.idstore.get(id) {
      // Whoops! Already assigned!!!
      // Can we recover? Only conflict if a DIFFERENT node already has this id.
      if node != prev {
        Some(prev.clone())
      } else {
        None
      }
    } else {
      None
    };
    let final_id = if let Some(prev) = prev_opt {
      let badid = id;
      let new_id = self.modify_id(id.to_owned());
      // Concise node descriptions, mirroring Perl `Stringify($node)`
      // (Common/Object.pm L40-49: `<tag attrs…>` with no child
      // serialization). Rust previously dumped the FULL node via
      // `node_to_string`, which (a) diverged from Perl's concise form and
      // (b) spilled child TEXT into the log — e.g. a figure caption
      // beginning "Error bars are the standard deviations…" then appears as
      // a line starting "Error" and is mis-counted as an error by
      // text-grep error sweeps (false positive on 2009.01426, which has
      // ZERO real errors). Use just the qname (+ the relevant id), which is
      // what an id-dedup diagnostic actually needs.
      let message = s!(
        "Duplicated attribute xml:id. Using id='{}' on <{}> id='{}' already set on <{}>",
        new_id,
        arena::to_string(get_node_qname(node)),
        badid,
        arena::to_string(get_node_qname(&prev))
      );
      // Perl-faithful (Document.pm L1454): Info-level. The id-counter
      // collision is the dedup-recovery path (`modify_id` appends
      // suffix), not silent corruption. The earlier Error-level
      // promotion was motivated by 1410.8171 (Sárkány PRA), but root-
      // cause investigation showed the empty-S3+ rendering there was
      // the siunitx ExplodeText! tokenization bug (fixed by
      // fc2aae7266), not the dedup recovery. After that fix, the
      // residual dedup events on the canvas are SHARED with Perl —
      // the in-tree test fixture tests/math/declare.xml itself bakes
      // in the `xml:id="S1.Ex1.m1.2a"` dedup result, confirming
      // Perl produces the same behavior. Downstream broken XMRefs
      // (post-dedup) emit Warn:expected:node "No node found with
      // id=…" which already surfaces the consequence at the appro-
      // priate severity. Math-parser hygiene fix (preventing the
      // collision in the first place) is tracked separately.
      Info!("malformed", "id", message);
      new_id
    } else {
      id.to_string()
    };
    self.idstore.insert(final_id.clone(), node.clone());
    final_id
  }

  pub fn unrecord_id(&mut self, id: &str) { self.idstore.remove(id); }

  /// Guardian-safe unlink: walk `node`'s subtree invalidating every `xml:id`
  /// idstore entry, then detach it from its parent. Use this in preference
  /// to a raw `node.unlink()` anywhere the node *might* carry an `xml:id`
  /// (subtree reshuffles in math-parser, post-processing cleanup, etc.),
  /// to prevent the dangling-Node class of bug that produced the 1605.08055
  /// Finalizing-phase SIGSEGV (see SYNC_STATUS.md D3b).
  ///
  /// This is the unlink-only half of `remove_node` — it does **not** adjust
  /// `self.node` / the insertion point, which is correct for callers that
  /// intend to re-parent the unlinked subtree elsewhere (the common case).
  /// Callers that want the insertion-point bookkeeping should use
  /// `remove_node` instead.
  pub fn safe_unlink(&mut self, mut node: Node) {
    if node.get_type() == Some(NodeType::ElementNode) {
      if let Some(id) = node.get_attribute_ns("id", XML_NS) {
        self.unrecord_id(&id);
      }
      for child in node.get_child_nodes() {
        self.remove_node_aux(child);
      }
    }
    node.unlink();
  }

  /// These are used to record or unrecord, in bulk, all the ids within a node (tree).
  ///
  /// When `record_id_with_node` detects a duplicate it renames the id (e.g.
  /// `X.1.mf` → `X.1.mfa`). Any sibling `<ltx:XMRef idref="X.1.mf"/>` in the
  /// same subtree would otherwise become a dangling reference for the post-
  /// processor. After re-recording IDs, sweep the subtree once and update any
  /// XMRef whose `idref` matches an entry in the rename map.
  ///
  /// Perl `Core::Document::recordNodeIDs` (Document.pm L1466-1472) has the
  /// same latent bug — recording renames but XMRefs aren't touched. The bug
  /// surfaces in our port because the math-parser path
  /// (`parser.rs::install_replacements`-style `unrecord+record` round-trips)
  /// is denser than Perl's; both end up needing this remap to keep XMRef
  /// chains intact. Intentional surpass-Perl divergence; tracked in
  /// SYNC_STATUS Task #10.
  pub fn record_node_ids(&mut self, node: &Node) -> Result<()> {
    use rustc_hash::FxHashMap;
    let mut rename: FxHashMap<String, String> = FxHashMap::default();
    for mut idnode in self.findnodes("descendant-or-self::*[@xml:id]", Some(node)) {
      if let Some(id) = idnode.get_attribute_ns("id", XML_NS) {
        let newid = self.record_id_with_node(&id, &idnode);
        if newid != id {
          idnode.set_attribute("xml:id", &newid)?;
          rename.insert(id, newid);
        }
      }
    }
    if !rename.is_empty() {
      for mut xmref in self.findnodes("descendant-or-self::*[@idref]", Some(node)) {
        if let Some(idref) = xmref.get_attribute("idref") {
          if let Some(new) = rename.get(&idref) {
            xmref.set_attribute("idref", new)?;
          }
        }
      }
    }
    Ok(())
  }

  pub fn unrecord_node_ids(&mut self, node: &Node) {
    for idnode in self.findnodes("descendant-or-self::*[@xml:id]", Some(node)) {
      if let Some(id) = idnode.get_attribute_ns("id", XML_NS) {
        self.unrecord_id(&id);
      }
    }
  }

  /// Discard the in-memory `idstore` cache and rebuild it from the
  /// current DOM state. Historically guarded the 1605.08055 SIGSEGV
  /// where `mark_xmnode_visibility` dereferenced dangling lookup_id
  /// entries while recursing through XMRef targets.
  ///
  /// As of cycle 72, the 5 call sites that previously dropped nodes
  /// without unrecord_id — math-parser `replace_tree` at
  /// parser.rs:456/690 (cascades via remove_node) and `unbind_node`
  /// loops at parser.rs:639/856 + rewrite.rs:522 (all have
  /// preceding unrecord_node_ids guards) — are ID-safe. This rebuild
  /// is retained as a belt-and-suspenders probe until the
  /// 1605.08055 verification per SYNC_STATUS.md D3b lands.
  ///
  /// The rebuild is a DOM walk, so live id uniqueness is restored
  /// alongside — duplicates already in DOM are resolved with
  /// `modify_id`, matching `record_node_ids` semantics.
  pub fn rebuild_idstore_from_dom(&mut self) -> Result<()> {
    self.idstore.clear();
    if let Some(root) = self.document.get_root_element() {
      self.record_node_ids(&root)?;
    }
    Ok(())
  }

  /// Get a new, related, but unique id.
  /// Sneaky option: try "ID_SUFFIX" as a suffix for id, first.
  /// Perl: sub modifyID (Document.pm lines 1483-1494)
  pub fn modify_id(&mut self, id: String) -> String {
    if self.idstore.contains_key(&id) {
      // Whoops! Already assigned!!!
      // Can we recover?
      let badid = id;
      // First try ID_SUFFIX if set
      if let Some(Stored::String(suffix)) = state::lookup_value("ID_SUFFIX") {
        let suffixed = s!("{}{}", badid, arena::to_string(suffix));
        if !self.idstore.contains_key(&suffixed) {
          return suffixed;
        }
      }
      // Try radix_alpha(1) through radix_alpha(26^3)
      // Gotta give up, eventually; is 3 letters enough?
      for s1 in 1_i64..=(26 * 26 * 26) {
        let candidate = s!("{}{}", badid, radix_alpha(s1));
        if !self.idstore.contains_key(&candidate) {
          return candidate;
        }
      }
      log::error!(
        "malformed:id: Automatic incrementing of ID counters failed for '{}'",
        badid
      );
      badid
    } else {
      id
    }
  }

  pub fn lookup_id(&self, id: &str) -> Option<&Node> { self.idstore.get(id) }

  /// Clone the idstore for use in thread-local contexts (math parsing).
  pub fn get_idstore_clone(&self) -> HashMap<String, Node> { self.idstore.clone() }

  // ======================================================================
  //  Odd bit:
  //  In an XMDual, in each branch (content, presentation) there will be atoms
  //  that correspond to the input (one will be real, the other an XMRef to the first).
  //  But also there will be additional "decoration" (delimiters, punctuation, etc on the
  // presentation  side; other symbols, bindings, whatever, on the content side).
  //  These decorations should NOT be subject to rewrite rules,
  //  and in cross-linked parallel markup, they should be attributed to the
  //  upper containing object's ID, rather than left dangling.
  //
  //  To determine this, we mark all math nodes as to whether they are "visible" from
  //  presentation, content or both (the default top-level being both).
  //  Decorations are the nodes that are visible to only one mode.
  //  Note that nodes that are not visible at all CAN occur (& do currently when the parser
  //  creates XMDuals), pruneXMDuals (below) gets rid of them.

  // NOTE: This should ultimately be in a base Document class,
  // since it is also needed before conversion to parallel markup!
  pub fn mark_xmnode_visibility(&mut self) -> Result<()> {
    let xmath = self.findnodes("//ltx:XMath/*", None);
    for math in xmath.iter() {
      for mut node in self.findnodes("descendant-or-self::*[@_pvis or @_cvis]", Some(math)) {
        node.remove_attribute("_pvis")?;
        node.remove_attribute("_cvis")?;
      }
    }
    for math in xmath {
      self.mark_xmnode_visibility_aux(math, true, true)?;
    }
    Ok(())
  }

  fn mark_xmnode_visibility_aux(&self, node: Node, cvis: bool, pvis: bool) -> Result<()> {
    // Recurses to math-tree depth via XMDual/XMRef-following + element-
    // child fan-out. Deep grammar-ambiguous papers (sandbox 0711.4787
    // et al, #17) hit Rust's 8 MB main-thread stack here during the
    // `Finalizing...` phase (via prune_xmduals → mark_xmnode_visibility).
    // Grow the stack on demand instead of overflowing.
    stacker::maybe_grow(64 * 1024, 4 * 1024 * 1024, move || {
      self.mark_xmnode_visibility_aux_inner(node, cvis, pvis)
    })
  }

  fn mark_xmnode_visibility_aux_inner(
    &self,
    mut node: Node,
    cvis: bool,
    mut pvis: bool,
  ) -> Result<()> {
    if (!cvis || node.has_attribute("_cvis")) && (!pvis || node.has_attribute("_pvis")) {
      return Ok(());
    }
    let qname = get_node_qname(&node);
    // Special case: for XMArg used to wrap "formal" arguments on the content side,
    // mark them as visible as presentation as well.
    if cvis && (qname == pin!("ltx:XMArg")) {
      pvis = true;
    }
    if cvis {
      node.set_attribute("_cvis", "1")?;
    }
    if pvis {
      node.set_attribute("_pvis", "1")?;
    }
    if qname == pin!("ltx:XMDual") {
      let mut children = xml::element_nodes(&node);
      // XMDual should have exactly 2 element children (content + presentation),
      // but a malformed math parse (e.g. semantic action producing empty pair
      // during deep ambiguity collapse) can leave an empty XMDual. Skip
      // visibility-marking rather than panicking on `children.remove(0)` —
      // see wp5 sandbox 2110.10033 and 4 sibling papers.
      if children.len() >= 2 {
        let c = children.remove(0);
        let p = children.remove(0);
        if cvis {
          self.mark_xmnode_visibility_aux(c, true, false)?;
        }
        if pvis {
          self.mark_xmnode_visibility_aux(p, false, true)?;
        }
      }
    } else if qname == pin!("ltx:XMRef") {
      match node.get_attribute("idref") {
        None => {
          let key = node.get_attribute("_xmkey");
          Warn!(
            "expected",
            "id",
            "Missing idref on ltx:XMRef",
            s!("_xmkey is `{}`", key.unwrap_or_default())
          );
        },
        Some(id) => match self.lookup_id(&id) {
          None => {
            Warn!(
              "expected",
              "node",
              s!("No node found with id='{id}' (referred to from ltx:XMRef)")
            );
          },
          Some(reffed) => {
            self.mark_xmnode_visibility_aux(reffed.clone(), cvis, pvis)?;
          },
        },
      }
    } else {
      for child in xml::element_nodes(&node) {
        self.mark_xmnode_visibility_aux(child, cvis, pvis)?;
      }
    }
    Ok(())
  }

  /// Remove `ltx:XMRef[@_split_ref="1"]` whose `idref` no longer
  /// resolves. These are the XMRefs minted by
  /// `amsmath::rearrange_ams_split` to mirror the flattened cell
  /// sequence inside an `XMDual(XMWrap(refs), XMArray(cells))`. The
  /// math parser can later absorb some cells (typically inserted
  /// MULOP times-ops on `\mathcal{L}\rho` chains) into wrapping
  /// XMApps, dropping their xml:id from the live DOM and leaving
  /// the sibling XMRefs dangling. Left in place, each dangling
  /// XMRef trips three separate diagnostics later — math parser's
  /// read_xmref Warn, finalize's mark_xmnode_visibility Warn, and
  /// post-process's mark_xm_node_visibility Error.
  ///
  /// We restrict the sweep to the `_split_ref` marker so refs from
  /// other provenance (base_xmath `\lx@dual`, renamed-id cases like
  /// declare_test's `S1.Ex1.m1.1` → `.1a` rename) stay untouched.
  ///
  /// Content-preserving: XMRefs are structural cross-references,
  /// not author body, and the math parser has already absorbed the
  /// referenced cell into the visible XMArray branch — no glyph or
  /// formula material is lost.
  fn prune_dangling_split_xmrefs(&mut self) -> Result<()> {
    let xmrefs = self.findnodes("//ltx:XMRef[@_split_ref or @_mf_ref]", None);
    for xmref in xmrefs {
      let idref = match xmref.get_attribute("idref") {
        Some(id) => id,
        None => continue,
      };
      if self.lookup_id(&idref).is_none() && xmref.get_parent().is_some() {
        self.remove_node(xmref);
      }
    }
    // Broader sweep: any XMRef pointing to a canonical math node id
    // `S<N>.E<M>.m1.<K>...` that no longer resolves. These are minted
    // by base_xmath::add_column_to_math_fork during rearrange_ams_*
    // (align/gather/multline), but unlike `_split_ref` they aren't
    // marked. The math parser absorbs cells later, leaving the refs
    // dangling and triggering the `Error:expected:id` cascade in
    // post-processing.
    //
    // We restrict the regex to the equation-numbered form (E<digit>,
    // not Ex<digit>) so declare_test's renamed-id case
    // (`S1.Ex1.m1.1` → `.1a`) stays untouched. canvas papers using
    // `\begin{equation}` produce `E1`/`E2`/... ids.
    static RE_MATH_ID: once_cell::sync::Lazy<regex::Regex> =
      once_cell::sync::Lazy::new(|| regex::Regex::new(r"^S\d+\.E\d+\.m\d+\.").unwrap());
    let xmrefs2 = self.findnodes("//ltx:XMRef[@idref]", None);
    for xmref in xmrefs2 {
      // Skip if already pruned via _split_ref sweep above.
      if xmref.get_parent().is_none() {
        continue;
      }
      let idref = match xmref.get_attribute("idref") {
        Some(id) => id,
        None => continue,
      };
      if RE_MATH_ID.is_match(&idref) && self.lookup_id(&idref).is_none() {
        self.remove_node(xmref);
      }
    }
    Ok(())
  }

  /// Reduce any ltx:XMDual's to just the visible branch, if the other is not visible
  /// (according to markXMNodeVisibility)
  /// If we could be 100% sure that the marking had stayed consistent (after various doc surgery)
  /// we could avoid re-marking, but we'd better be sure before removing nodes!
  fn prune_xmduals(&mut self) -> Result<()> {
    // RE-mark visibility!
    self.mark_xmnode_visibility()?;
    // will reversing keep from problems removing nodes from trees that already have been removed?
    for dual in self
      .findnodes("descendant-or-self::ltx:XMDual", None)
      .into_iter()
      .rev()
    {
      self.document.node_to_string(&dual);
      let mut dual_children = xml::element_nodes(&dual);
      // Defensive: an XMDual should always have presentation +
      // content children, but a malformed math parse (post-ambiguity
      // collapse) can yield <2. Skip rather than panic.
      // Witness 2110.10033 (panicked at document.rs:3120, post-fix
      // continuation of earlier guard at document.rs:2993).
      let Some(presentation) = dual_children.pop() else { continue };
      let Some(content) = dual_children.pop() else { continue };
      if self
        .findnode("descendant-or-self::*[@_pvis or @_cvis]", Some(&content))
        .is_none()
      {
        // content never seen
        self.collapse_xmdual(dual, presentation)?;
      } else if self
        .findnode(
          "descendant-or-self::*[@_pvis or @_cvis]",
          Some(&presentation),
        )
        .is_none()
      {
        // pres.
        self.collapse_xmdual(dual, content)?;
      } else {
        // compact aligned structures, where possible
        self.compact_xmdual(dual, content, Some(presentation))?;
      }
    }
    Ok(())
  }

  fn compact_xmdual(
    &mut self,
    dual: Node,
    content: Node,
    presentation: Option<Node>,
  ) -> Result<()> {
    // Perl: our $content_transfer_overrides = { decl_id, meaning, name, omcd };
    // Perl: our $dual_transfer_overrides = { decl_id, meaning, name, omcd, xml:id, role };
    static CONTENT_TRANSFER: Lazy<HashSet<&'static str>> =
      Lazy::new(|| HashSet::from_iter(["decl_id", "meaning", "name", "omcd"]));
    static DUAL_TRANSFER: Lazy<HashSet<&'static str>> =
      Lazy::new(|| HashSet::from_iter(["decl_id", "meaning", "name", "omcd", "xml:id", "role"]));

    let presentation = match presentation {
      Some(p) => p,
      None => return Ok(()),
    };
    let c_name = with_node_qname(&content, |n| n.to_string());
    let p_name = with_node_qname(&presentation, |n| n.to_string());

    // Case 1: Quick fix — merge two tokens
    if c_name == "ltx:XMTok" && p_name == "ltx:XMTok" {
      let mut pres = presentation;
      let pres_had_id = pres.has_attribute("xml:id");
      self.merge_attributes(&content, &mut pres, Some(&CONTENT_TRANSFER))?;
      self.merge_attributes(&dual, &mut pres, Some(&DUAL_TRANSFER))?;
      // If presentation didn't originally have an xml:id and the dual doesn't either,
      // remove the content's xml:id that leaked through merge_attributes
      if !pres_had_id && !dual.has_attribute("xml:id") {
        if let Some(id) = pres.get_attribute("xml:id") {
          self.unrecord_id(&id);
          let _ = pres.remove_attribute("xml:id");
        }
      }
      // Unlink presentation from dual before replacing, since presentation is a child of dual
      pres.unlink();
      self.replace_node(dual, vec![pres])?;
      return Ok(());
    }

    // Case 2: Compact mirror XMApp nodes
    if c_name != "ltx:XMApp" || p_name != "ltx:XMApp" {
      return Ok(());
    }
    let content_args = xml::element_nodes(&content);
    let pres_args = xml::element_nodes(&presentation);
    if content_args.len() != pres_args.len() {
      return Ok(());
    }
    let n_args = content_args.len();

    // Walk the corresponding children, double-check they are referenced in the same order
    enum NewArg {
      Single(Node),
      Pair(Node, Node), // (content_arg, pres_arg) — to be merged
    }
    let mut new_args: Vec<NewArg> = Vec::with_capacity(n_args);
    for (c_arg, p_arg) in content_args.into_iter().zip(pres_args) {
      if let Some(c_idref) = c_arg.get_attribute("idref") {
        if c_idref == p_arg.get_attribute_ns("id", XML_NS).unwrap_or_default() {
          new_args.push(NewArg::Single(p_arg));
          continue;
        }
      }
      if let Some(p_idref) = p_arg.get_attribute("idref") {
        if p_idref == c_arg.get_attribute_ns("id", XML_NS).unwrap_or_default() {
          new_args.push(NewArg::Single(c_arg));
          continue;
        }
      }
      // We can handle content-side XMToks to any XM* presentation subtree
      let c_arg_name = with_node_qname(&c_arg, |n| n.to_string());
      if c_arg_name != "ltx:XMTok" {
        return Ok(()); // Can't compact this structure
      }
      new_args.push(NewArg::Pair(c_arg, p_arg));
    }

    // If we made it here, this dual has two mirrored applications — compact it.
    let mut parent = match dual.get_parent() {
      Some(p) => p,
      None => return Ok(()),
    };
    let mut compact_apply = self.open_element_at(&mut parent, "ltx:XMApp", None, None)?;
    for n_arg in new_args {
      let mut node = match n_arg {
        NewArg::Single(n) => n,
        NewArg::Pair(c_arg, mut p_arg) => {
          self.merge_attributes(&c_arg, &mut p_arg, Some(&CONTENT_TRANSFER))?;
          p_arg
        },
      };
      node.unlink();
      compact_apply.add_child(&mut node)?;
    }
    // Migrate dual attributes to the new XMApp
    self.merge_attributes(&dual, &mut compact_apply, Some(&DUAL_TRANSFER))?;
    // Direct DOM swap: replace dual with compact_apply without re-creating nodes.
    // Perl uses replaceChild which is a direct swap; replace_tree calls append_tree
    // which re-creates elements and fires afterOpen/afterClose hooks a second time.
    compact_apply.unlink();
    let mut dual_mut = dual;
    dual_mut.add_prev_sibling(&mut compact_apply).ok();
    self.remove_node(dual_mut);
    Ok(())
  }

  /// Replace an XMDual with one of its branches
  fn collapse_xmdual(&mut self, dual: Node, mut branch: Node) -> Result<()> {
    // The other branch is not visible, nor referenced,
    // but the dual may have an id and be referenced
    if let Some(dualid) = dual.get_attribute_ns("id", XML_NS) {
      self.unrecord_id(&dualid); // We'll move or remove the ID from the dual
      if let Some(branchid) = branch.get_attribute_ns("id", XML_NS) {
        // branch has id too!
        for mut tref in self.findnodes(&s!("//*[@idref='{}']", dualid), None) {
          tref.set_attribute("idref", &branchid)?;
        } // Change dualid refs to branchid
      } else {
        // Assign the dual's id to the branch. Record first so we
        // receive a deduplicated id if something else claimed it
        // between the `unrecord_id` above and now — write the
        // deduped value, not the original.
        let deduped = self.record_id_with_node(&dualid, &branch);
        branch.set_attribute("xml:id", &deduped)?;
      }
    }
    // Direct DOM swap: Perl uses replaceChild (no re-creation, no hooks fired twice)
    let mut dual_mut = dual;
    dual_mut.add_prev_sibling(&mut branch).ok();
    self.remove_node(dual_mut);
    Ok(())
  }

  //**********************************************************************
  /// Record the Box that created this node.
  pub fn set_node_box(&mut self, node: &Node, digested: Digested) {
    let nodeid = node.to_hashable();
    self.node_boxes.insert(nodeid, digested);
  }

  pub fn get_node_box(&self, node: &Node) -> Option<Digested> {
    if node.get_type() == Some(NodeType::ElementNode) {
      let nodeid = node.to_hashable();
      self.node_boxes.get(&nodeid).cloned()
    } else {
      None
    }
  }

  //**********************************************************************
  /// Record the Font of a node
  pub fn set_node_font(&mut self, node: &mut Node, font: &Font) -> Result<()> {
    let fontid = font.to_hashable();
    node.set_attribute("_font", &fontid.to_string())?;
    // try to avoid aggressive clones, when unnecessary
    match self.node_fonts.get(&fontid) {
      None => {
        self.node_fonts.insert(fontid, font.clone());
      },
      Some(v) => {
        if v != font {
          self.node_fonts.insert(fontid, font.clone());
        }
      },
    }
    Ok(())
  }

  pub fn copy_node_font(&mut self, from: &Node, to: &mut Node) -> Result<()> {
    if let Some(fontid) = from.get_attribute("_font") {
      to.set_attribute("_font", &fontid)?;
    }
    Ok(())
  }

  /// Possibly a sign of a design flaw; Set the node's font & all children that HAD the same font.
  pub fn merge_node_font_rec(&mut self, node: &Node, font: &Font) -> Result<()> {
    let oldfont = self.get_node_font(node);
    let props = oldfont.purestyle_changes(font);
    let mut nodes = VecDeque::new();
    nodes.push_front(node.clone());
    while let Some(mut n) = nodes.pop_front() {
      if n.get_type() == Some(NodeType::ElementNode) {
        let font = &self.get_node_font(&n).merge_ref(&props);
        self.set_node_font(&mut n, font)?;
        for child in n.get_child_nodes() {
          nodes.push_back(child);
        }
      }
    }
    Ok(())
  }

  pub fn set_box_font(&mut self, node: &mut Node) -> Result<()> {
    if let Some(ref thisbox) = self.box_to_absorb {
      if let Some(font) = thisbox.get_font()? {
        let todo_font_clone = font.into_owned();
        self.set_node_font(node, &todo_font_clone)?;
      }
    }
    Ok(())
  }

  pub fn get_node_font(&self, node: &Node) -> &Font {
    if let Some(element) = xml::closest_element(node) {
      // Use the closest element (for text nodes, this is the parent element)
      if let Some(fontid) = element.get_attribute("_font") {
        // Tolerate non-numeric `_font` attributes — they can occur when
        // a corrupted property propagates across reversion (driver:
        // 2304.07380 panicked at parse::<u64>().unwrap()). Fall through
        // to the default font instead of aborting the run.
        if let Ok(id) = fontid.parse::<u64>() {
          if let Some(fnt) = self.node_fonts.get(&id) {
            return fnt;
          }
        }
      }
    }
    &FONT_TEXT_DEFAULT
  }

  /// Decode a _font hash string to a Font object
  pub fn decode_font(&self, font_hash: &str) -> Option<&Font> {
    font_hash
      .parse::<u64>()
      .ok()
      .and_then(|id| self.node_fonts.get(&id))
  }

  pub fn has_node_font(&self, node: &Node) -> bool {
    if let Some(element) = xml::closest_element(node) {
      element.has_attribute("_font")
    } else {
      false
    }
  }

  pub fn get_node_language(&self, node: &Node) -> String {
    let mut node_ref = node;
    let mut current;
    loop {
      if node_ref.get_type() != Some(NodeType::ElementNode) {
        break;
      }
      if let Some(lang) = node_ref.get_attribute("xml:lang") {
        return lang;
      }
      if let Some(fontid) = node.get_attribute("_font") {
        if let Some(font) = self.node_fonts.get(&fontid.parse::<u64>().unwrap()) {
          if let Some(lang) = font.get_language() {
            return lang.to_string();
          }
        }
      }
      if let Some(parent) = node_ref.get_parent() {
        current = parent;
        node_ref = &current;
      } else {
        break;
      }
    }
    String::from("en")
  }

  // sub decodeFont {
  //   my ($self, $fontid) = @_;
  //   return $$self{node_fonts}{$fontid} || LaTeXML::Common::Font->textDefault(); }

  // Remove a node from the document (from it's parent)
  pub fn remove_node(&mut self, mut node: Node) {
    let mut chopped: bool = self.node == node; // Note if we're removing insertion point
    if node.get_type() == Some(NodeType::ElementNode) {
      // If an element, do ID bookkeeping.
      if let Some(id) = node.get_attribute_ns("id", xml::XML_NS) {
        self.unrecord_id(&id);
      }
      for child in node.get_child_nodes() {
        chopped = chopped || self.remove_node_aux(child);
      }
    }
    if let Some(parent) = node.get_parent() {
      if chopped {
        // Don't remove insertion point!
        self.set_node(&parent);
      }
      node.unlink();
    }
  }

  fn remove_node_aux(&mut self, node: Node) -> bool {
    let mut chopped = self.node == node;
    if node.get_type() == Some(NodeType::ElementNode) {
      // If an element, do ID bookkeeping.
      if let Some(id) = node.get_attribute_ns("id", xml::XML_NS) {
        self.unrecord_id(&id);
      }
      for child in node.get_child_nodes() {
        chopped = chopped || self.remove_node_aux(child);
      }
    }
    chopped
  }

  //**********************************************************************
  // Inserting new nodes at random points into the document,
  // typically, later in the process or during some kind of rearrangement.

  // This is a somewhat strange situation; There are commands and environments
  // that do some interesting thing to their contents. This include things like
  // center, flushleft, or rotate, or ...
  // Naively one is tempted to create a containing block with appropriate type &
  // attributes. However, since these things can be allowed in so many places
  // by LaTeX, that one has a difficult time creating a sensible document model.
  // The purpose of transformingBlock is to set the contents (possibly creating a
  // consistent <p> around them, if called for), and returning the list of newly
  // created nodes. These nodes can then have appropriate attributes added as
  // needed for each specific case.

  // Since this situation can occur in both LaTeX and AmSTeX type documents,
  // we'll put it in the TeX pool so it can be reused.

  // Tricky bit for creating nodes late in the game,

  ////// See createElementAt
  /// This opens a new element at the _specified_ point, rather than the current insertion point.
  /// This is useful during document rearrangement or augmentation that may be needed later
  /// in the process.
  pub fn open_element_at(
    &mut self,
    point: &mut Node,
    qname: &str,
    attributes: Option<HashMap<String, String>>,
    mut font_opt: Option<Font>,
  ) -> Result<Node> {
    // Font resolution priority (matching Perl's openElement/openElementAt):
    // 1. Explicit font_opt parameter
    // 2. _font attribute in attributes hash
    // 3. box_to_absorb.get_font() — the font of the current box being absorbed (Perl:
    //    $attributes{_box} = $LaTeXML::BOX; $font = $attributes{_box}->getFont)
    // 4. Insertion point's font (final fallback, handled later)
    if font_opt.is_none() {
      if let Some(ref attrs) = attributes {
        if let Some(fontid) = attrs.get("_font") {
          // Tolerate non-numeric `_font` attributes — see get_node_font
          // for the same defensive read; same panic site, different
          // call. Driver: 2406.14188.
          if let Ok(id) = fontid.parse::<u64>() {
            font_opt = self.node_fonts.get(&id).cloned();
          }
        }
      }
    }
    if font_opt.is_none() {
      if let Some(ref digested) = self.box_to_absorb {
        if let Ok(Some(font)) = digested.get_font() {
          font_opt = Some(font.into_owned());
        }
      }
    }
    let (decoded_ns, tag) = model::decode_qname(qname)?;
    let mut newnode;
    // box = self.node_boxes.get(box);    // may already be the string key
    // If this will be the document root node, things are slightly more involved.
    if point.get_type() == Some(NodeType::DocumentNode) {
      // First node! (?)
      Debug!("adding schema declaration, new node will be : {}", tag);
      model::add_schema_declaration(self);
      newnode = Node::new(&tag, None, &self.document).unwrap();
      self.record_constructed_node(&newnode);
      self.document.set_root_element(&newnode);
      for node in &mut self.pending {
        newnode.add_prev_sibling(node)?; // Add saved comments, PI's
      }

      if let Some(ns) = decoded_ns {
        // Here, we're creating the initial, document element, which will hold ALL of
        // the namespace declarations. If there is a default namespace (no
        // prefix), that will also be declared, and applied here. However, if
        // there is ALSO a prefix associated with that namespace, we have to declare it
        // FIRST due to the (apparently) buggy way that XML::LibXML works with
        // namespaces in setAttributeNS.
        let prefix_opt = model::get_document_namespace_prefix(&ns, false, false);
        let attprefix_opt = model::get_document_namespace_prefix(&ns, true, true);
        if prefix_opt.is_none() {
          if let Some(attprefix_sym) = attprefix_opt {
            let attr_ns_node = arena::with(attprefix_sym, |attprefix| {
              Namespace::new(attprefix, &ns, &mut newnode)
            })
            .unwrap();
            newnode.set_namespace(&attr_ns_node)?;
          }
        }
        // TODO: Figure out a better way to achieve the "activate" effect in
        // XML:LibXML::Element it seems just creating the namespace without
        // setting it is equivalent ??
        let ns_node = Namespace::new("", &ns, &mut newnode).unwrap();
        newnode.set_namespace(&ns_node)?;
      }
    } else {
      if font_opt.is_none() {
        font_opt = Some(self.get_node_font(point).clone());
      }
      newnode = self.open_element_internal(point, decoded_ns, &tag)?;
    }

    // Source-locator stamping (`--source-map`, issues #47/#92). `open_element_at`
    // is the shared element-creation primitive (plain `open_element`, math, and
    // alignment all route here), so stamping here covers them uniformly —
    // including the `ltx:Math` wrapper that bypasses `open_element`. Off by
    // default; the cheap gate keeps the normal path free.
    if state::source_map_enabled() {
      self.stamp_source_locator(&newnode, qname);
    }

    if let Some(attrs) = attributes {
      let mut sorted_keys = attrs.keys().map(String::as_str).collect::<Vec<_>>();
      sorted_keys.sort_unstable();
      for key in sorted_keys {
        if key == "font" || key == "locator" {
          continue;
        }
        self.set_attribute(&mut newnode, key, &attrs[key])?;
      }
    }
    if let Some(font) = font_opt {
      self.set_node_font(&mut newnode, &font)?;
    }

    // TODO [new]: Ever more certain there is a refactor waiting to happen with box_to_absorb
    //             holding a Rc<Digested> for easy cloning and management.
    //             Though the question remains how to maintain that, without cloning the box to
    // **make** the Rc<> Old note:
    // The .clone on boxes is potentially *VERY SLOW* and a code smell.
    // It can be eventually avoided by using a "memory arena" for all intermediate
    // objects - tokens, boxes, etc. and a well-designed referncing scheme into
    // the driver structs, such as Gullet, Stomach and Document
    if let Some(ref digested) = self.box_to_absorb {
      self.set_node_box(&newnode, digested.clone());
    }

    // Debug!(
    //   s!("Inserting {:?} into {:?}", get_node_qname(&newnode), get_node_qname(point))
    // );

    // Run afterOpen operations
    self.after_open(&mut newnode)?;

    Ok(newnode)
  }

  fn open_element_internal(
    &mut self,
    point: &mut Node,
    ns_opt: Option<String>,
    tag: &str,
  ) -> Result<Node> {
    // TODO:
    //
    // I am seriously irritated by the XML namespace and the confusion of the "default#"
    // tricks and libxml2's custom decisions about namespace interactions
    //
    // I have "hacked together" a working flow for now, but I expect to
    // encounter bugs related to the shortcuts taken here. I would welcome a
    // redesign       that simplifies the namespace logic dramatically.
    let new_ns = match ns_opt {
      Some(ns_uri) => {
        match point.lookup_namespace_prefix(&ns_uri) {
          // namespace not already declared?
          None => {
            if let Some(prefix) = model::get_document_namespace_prefix(&ns_uri, false, false) {
              if prefix != pin!("") {
                let mut root = self.document.get_root_element().unwrap();
                match arena::with(prefix, |prefix_str| {
                  Namespace::new(prefix_str, &ns_uri, &mut root)
                }) {
                  Ok(ns) => Some(ns),
                  Err(_) => {
                    // The namespace already exists on root (declared by an
                    // earlier element of the same namespace — e.g. a prior
                    // tikz/SVG picture) but `lookup_namespace_prefix` did not
                    // find it from this deeply-nested insertion point. Recover
                    // by reusing the root declaration (or creating it on the
                    // insertion point), exactly as the already-declared branch
                    // below — do NOT drop the namespace. Witness 1802.00756:
                    // a `tikzpicture` inside a nested `gather*`/`minipage`/
                    // `figure*` emitted 14× "failed to create namespace: svg"
                    // and the `<svg:svg>`/`<svg:g>` lost their namespace.
                    arena::with(prefix, |prefix_str| {
                      let found = root
                        .get_namespace_declarations()
                        .into_iter()
                        .find(|ns| ns.get_prefix() == prefix_str);
                      if found.is_none() {
                        Namespace::new(prefix_str, &ns_uri, point).ok()
                      } else {
                        found
                      }
                    })
                  },
                }
              } else {
                // default namespace?
                None
              }
            } else {
              // default namespace?
              None
            }
          },
          Some(prefix) => {
            if !prefix.is_empty() {
              let mut root = self.document.get_root_element().unwrap();
              match Namespace::new(&prefix, &ns_uri, &mut root) {
                Ok(ns) => Some(ns),
                Err(_) => {
                  // Namespace already exists on root — find and reuse it.
                  // We search declarations then fall back to creating on the
                  // insertion point (which inherits from root).
                  let found = root
                    .get_namespace_declarations()
                    .into_iter()
                    .find(|ns| ns.get_prefix() == prefix);
                  if found.is_none() {
                    // Try creating on the insertion point instead
                    Namespace::new(&prefix, &ns_uri, point).ok()
                  } else {
                    found
                  }
                },
              }
            } else {
              // default namespace?
              None
            }
          },
        }
      },
      None => None,
    };

    let no_ns = new_ns.is_none();
    let mut newnode = match Node::new(tag, new_ns.clone(), &self.document) {
      Ok(n) => n,
      Err(_) => {
        // libxml2 rejected the tag (e.g. NUL byte, malformed name).
        // Bail out of element creation rather than aborting; caller
        // can recover. Driver: 2304.07380 panic at Node::new unwrap.
        let message = s!("failed to create element {:?}", tag);
        Error!("document", "open_element_internal", message);
        return Err(message.into());
      },
    };
    point.add_child(&mut newnode)?;
    if no_ns {
      // When no explicit namespace was determined (default namespace element),
      // try to find the root's default namespace first. This prevents inheriting
      // the parent's namespace when inside a different namespace context (e.g.,
      // SVG elements getting svg: prefix on LaTeXML elements like Math/XMath).
      let root_ns = self
        .document
        .get_root_element()
        .and_then(|r| r.get_namespace());
      let parent_ns = point.get_namespace();
      if let Some(ref rns) = root_ns {
        // Use root's namespace for default-namespace elements
        let _ = newnode.set_namespace(rns);
      } else if let Some(ns) = parent_ns {
        // Fallback: inherit from parent (original behavior)
        newnode.set_namespace(&ns)?;
      }
    } else if let Some(ref ns) = new_ns {
      // For explicitly namespaced elements (e.g., svg:svg), ensure the namespace
      // is set after add_child — Node::new may not properly bind the namespace
      // when the Namespace was retrieved from get_namespace_declarations().
      let _ = newnode.set_namespace(ns);
    }

    self.record_constructed_node(&newnode);
    Ok(newnode)
  }

  /// Whenever a node has been created using openElementAt,
  /// closeElementAt ought to be used to close it, when you're finished inserting into $node.
  /// Basically, this just runs any afterClose operations.
  pub fn close_element_at(&mut self, node: &mut Node) -> Result<()> { self.after_close(node) }

  pub fn after_open(&mut self, node: &mut Node) -> Result<()> {
    // Set current point to this node, just in case the afterOpen's use it.
    let savenode = self.node.clone();
    self.set_node(node);
    let node_qname = get_node_qname(node);
    // Perl: my $box = getNodeBox($self, $node);
    let box_opt = self.get_node_box(node);
    for action in self.get_tag_action_list(node_qname, TagOptionName::AfterOpen) {
      action(self, node, box_opt.as_ref())?;
    }
    self.set_node(&savenode);
    Ok(())
  }

  pub fn after_close(&mut self, node: &mut Node) -> Result<()> {
    // Should we set point to this node? (or to last child, or something ??
    let savenode = self.node.clone();
    let node_qname = get_node_qname(node);
    // Perl: my $box = getNodeBox($self, $node);
    let box_opt = self.get_node_box(node);
    for action in self.get_tag_action_list(node_qname, TagOptionName::AfterClose) {
      action(self, node, box_opt.as_ref())?;
    }
    self.set_node(&savenode);
    Ok(())
  }

  //**********************************************************************
  // Appending clones of nodes

  // Inserting clones of nodes into the document.
  // Nodes that exist in some other part of the document (or some other document)
  // will need to be cloned so that they can be part of the new document;
  // otherwise, they would be removed from thier previous document.
  // Also, we want to have a clean namespace node structure
  // (otherwise, libxml2 has a tendency to introduce annoying "default" namespace prefix
  // declarations) And, finally, we need to modify any id's present in the old nodes,
  // since otherwise they may be duplicated.

  // # Should have variants here for prepend, insert before, insert after.... ???
  pub fn append_clone(&mut self, node: &mut Node, new_children: Vec<Node>) -> Result<()> {
    // Expand any document fragments
    let new_children = new_children
      .into_iter()
      .flat_map(|child| {
        if child.get_type() == Some(NodeType::DocumentFragNode) {
          child.get_child_nodes()
        } else {
          vec![child]
        }
      })
      .collect::<Vec<Node>>();
    // Now find all xml:id's in the new_children and record replacement id's for them
    let mut id_map = HashMap::default();
    // Find all id's defined in the copy and change the id.
    // Note: XPath ".//@xml:id" can fail to find namespace-qualified attributes.
    // Use DOM walking as fallback to ensure all ids are found.
    for child in new_children.iter() {
      let mut xpath_ids: Vec<String> = self.findvalues(".//@xml:id", Some(child));
      if xpath_ids.is_empty() {
        // Fallback: walk DOM to find xml:id attributes
        Self::collect_xml_ids_from(child, &mut xpath_ids);
      }
      for id in xpath_ids {
        id_map.insert(id.clone(), self.modify_id(id));
      }
    }
    // Now do the cloning (actually copying) and insertion.
    self.append_clone_aux(node, new_children, &mut id_map)
  }

  /// Walk DOM to collect xml:id attribute values (fallback when XPath fails).
  fn collect_xml_ids_from(node: &Node, ids: &mut Vec<String>) {
    if let Some(id) = node.get_attribute("xml:id") {
      ids.push(id);
    } else if let Some(id) = node.get_attribute_ns("id", XML_NS) {
      ids.push(id);
    }
    for child in node.get_child_nodes() {
      if child.get_type() == Some(NodeType::ElementNode) {
        Self::collect_xml_ids_from(&child, ids);
      }
    }
  }

  fn append_clone_aux(
    &mut self,
    node: &mut Node,
    new_children: Vec<Node>,
    id_map: &mut HashMap<String, String>,
  ) -> Result<()> {
    for child in new_children.into_iter() {
      match child.get_type() {
        Some(NodeType::ElementNode) => {
          let mut new = self.open_element_internal(
            node,
            child.get_namespace().map(|ns| ns.get_href()),
            &child.get_name(),
          )?;
          for (key, val) in child.get_attributes() {
            match key.as_str() {
              "xml:id" | "id" => {
                // Use the replacement id. The pre-walk
                // (findvalues/collect_xml_ids_from) normally populates
                // id_map with every `xml:id` it can see, but the
                // namespace/prefix of the attribute key returned by
                // libxml2's `get_attributes()` can differ from what the
                // XPath / DOM walk picked up (e.g. when the incoming
                // node's tree came through a cloneNode that stripped
                // the xml namespace). Fall back to minting a fresh
                // replacement id on-the-fly rather than panicking —
                // 1410.8508 hit this when the pre-walk found zero ids
                // but attributes on a child node did carry xml:id.
                let fresh;
                let mapped_id = match id_map.get(&val) {
                  Some(id) => id,
                  None => {
                    fresh = self.modify_id(val.clone());
                    id_map.insert(val.clone(), fresh.clone());
                    id_map.get(&val).unwrap()
                  },
                };
                let newid = self.record_id_with_node(mapped_id, &new);
                // Write the literal "xml:id" key (not the bare "id" local-
                // name returned by libxml's get_attributes). Otherwise the
                // cloned node only gets a plain `id` attribute, and the
                // subsequent `after_open` chain's `has_attribute_ns("id",
                // XML_NS)` check returns false, causing `generate_id` to
                // mint a fresh `.<parent>.N` xml:id that doesn't match the
                // sibling XMRef idrefs (which were rewritten from id_map).
                // Witness: arXiv:2509.07628 — MathFork mainfork emitted
                // 154 XMRefs with `.mf` idrefs while the cloned target
                // nodes received parent-scoped `.m2.N` xml:ids, leaving
                // every XMRef dangling and triggering 4
                // `Error:expected:id` per equation during post-processing
                // visibility marking. Same shape applies anywhere a
                // cloned subtree carries xml:id attributes — MathFork,
                // tabular-cell clone, _Capture_ flush.
                new.set_attribute("xml:id", &newid)?;
                // Update id_map so subsequent idref lookups use the ACTUAL recorded id.
                // record_id_with_node may change the id (e.g., if there are conflicts),
                // so the mapped_id and newid may differ.
                if *mapped_id != newid {
                  id_map.insert(val.clone(), newid);
                }
              },
              "idref" => {
                // Refer to the replacement id if it was replaced
                let id = id_map.get(&val).unwrap_or(&val);
                new.set_attribute(&key, id)?;
              },
              other_key =>
              // TODO: Are namespaced attributes successfully handled here? Check.
              {
                new.set_attribute(other_key, &val)?
              },
            };
          }
          self.after_open(&mut new)?;
          self.append_clone_aux(&mut new, child.get_child_nodes(), id_map)?;
          self.after_close(&mut new)?;
        },
        Some(NodeType::TextNode) => node.append_text(&child.get_content())?,
        Some(NodeType::CommentNode) => {
          // Skip XML comments during cloning (Perl also skips them in most contexts)
        },
        other => {
          log::warn!("append_clone_aux: skipping unsupported {other:?} node type");
        },
      };
    }
    Ok(())
  }

  //**********************************************************************
  // Wrapping & Unwrapping nodes by another element.

  // Wrap `nodes` with an element named `qname`, making the new element replace the first `node`,
  // and all `nodes` becomes the child of the new node.
  // [this makes most sense if `nodes` are a sequence of siblings]
  // Returns undef if $qname isn't allowed in the parent, or if `nodes` aren't allowed in `qname`,
  // otherwise, returns the newly created `qname`.
  pub fn wrap_nodes(&mut self, qname: &str, nodes: Vec<Node>) -> Result<Option<Node>> {
    if nodes.is_empty() {
      return Ok(None);
    }
    let first_node = &nodes[0];
    let mut parent = first_node.get_parent().unwrap();
    let (ns, tag) = model::decode_qname(qname)?;
    let mut new = self.open_element_internal(&mut parent, ns, &tag)?;
    self.after_open(&mut new)?;
    parent.replace_child_node(new.clone(), first_node.clone())?;

    self.copy_node_font(&parent, &mut new)?;

    if let Some(tbox) = self.get_node_box(&parent) {
      self.set_node_box(&new, tbox);
    }
    for mut node in nodes.into_iter() {
      node.unlink();
      new.add_child(&mut node)?;
    }
    self.after_close(&mut new)?;
    Ok(Some(new))
  }

  /// Unwrap the children of $node, by replacing $node by its children.
  pub fn unwrap_nodes(&mut self, node: Node) -> Result<()> {
    let children = node.get_child_nodes();
    self.replace_node(node, children)
  }

  /// Replace `node` by `nodes` (presumably descendants of some kind?)
  // DG: Don't return the replaced `node`, as it is groudns for memory management trouble
  //     with the low-level libxml layer. I've encountered segfaults here.
  pub fn replace_node(&mut self, mut node: Node, with: Vec<Node>) -> Result<()> {
    if let Some(_parent) = node.get_parent() {
      // libxml2's xmlAddNextSibling merges consecutive text nodes: when both
      // the reference sibling and the new node are TextNode, it appends the
      // new node's content to the reference node and frees the new node. The
      // Rust wrapper doesn't surface the merged result, so naively advancing
      // `c0_opt` to `with_node` would capture a pointer to freed memory,
      // producing silent data loss for the third+ insertion and eventually
      // a libxml2 SIGSEGV when the dangling pointer is re-traversed (e.g.
      // during a later XPath evaluation). Detect the text-text case and
      // coalesce in-place instead.
      let mut c0_opt: Option<Node> = None;
      for mut with_node in with.into_iter() {
        with_node.unlink();
        let is_text = with_node.get_type() == Some(NodeType::TextNode);
        if let Some(mut c0) = c0_opt {
          let c0_is_text = c0.get_type() == Some(NodeType::TextNode);
          if is_text && c0_is_text {
            let existing = c0.get_content();
            let added = with_node.get_content();
            c0.set_content(&format!("{existing}{added}"))?;
            // with_node is still a standalone (unlinked) text node; drop it.
            c0_opt = Some(c0);
            continue;
          }
          c0.add_next_sibling(&mut with_node)?;
        } else {
          // first node, swap in
          node.add_next_sibling(&mut with_node)?;
        }
        c0_opt = Some(with_node);
      }
      self.remove_node(node);
    }
    Ok(())
  }

  // initially since $node->setNodeName was broken in XML::LibXML 1.58
  // but this can provide for more options & correctness?
  pub fn rename_node(&mut self, node: Node, newname: &str, reinsert: bool) -> Result<Node> {
    let (ns, tag) = model::decode_qname(newname)?;
    let newsym = arena::pin(newname);
    self.rename_node_internal(node, newsym, ns, tag, reinsert)
  }
  pub fn rename_node_qsym(&mut self, node: Node, newsym: SymStr, reinsert: bool) -> Result<Node> {
    let (ns, tag) = model::decode_qname_sym(newsym)?;
    self.rename_node_internal(node, newsym, ns, tag, reinsert)
  }
  fn rename_node_internal(
    &mut self,
    mut node: Node,
    newname: SymStr,
    ns: Option<String>,
    tag: String,
    reinsert: bool,
  ) -> Result<Node> {
    let mut parent = node
      .get_parent()
      .expect("rename should never be called on an orphan or root node.");
    let mut new = self.open_element_internal(&mut parent, ns, &tag)?;
    // Move to the position AFTER node
    node.add_next_sibling(&mut new)?;
    // Copy ALL attributes from `node` to `newnode`
    let mut id = None;
    for (key, value) in node.get_attributes() {
      let can_have = model::can_have_attribute(newname, arena::pin(&key));
      if can_have {
        new.set_attribute(&key, &value)?;
      }
      if key == "xml:id" {
        id = Some(value); // Save to register after removal of old node.
      }
    }
    // AND move all content from `node` to `newnode`
    if !reinsert {
      for mut child in node.get_child_nodes() {
        child.unbind();
        new.add_child(&mut child)?;
      }
    } else {
      std::mem::swap(&mut self.node, &mut new);
      for mut child in node.get_child_nodes() {
        child.unbind();
        if child.get_type() == Some(NodeType::TextNode) {
          self.open_text_internal(&child.get_content())?;
          self.close_text_internal()?;
        } else {
          let child_qname = get_node_qname(&child);
          let mut point = self.find_insertion_point_qsym(child_qname, None)?;
          point.add_child(&mut child)?;
        }
      }
      std::mem::swap(&mut self.node, &mut new);
    }
    // THEN call afterOpen... ?
    //   It would normally be called before children added,
    //   but how can we know if we're duplicated auto-added stuff?
    self.after_open(&mut new)?;
    self.after_close(&mut new)?;
    // Finally, remove the old node
    self.remove_node(node);

    // and FINALLY, we can register the new node under the id.
    if let Some(id) = id {
      let newid = self.record_id_with_node(&id, &new);
      if newid != id {
        new.set_attribute("xml:id", &newid)?;
      }
    }

    Ok(new)
  }

  pub fn trim_node_whitespace(&mut self, node: &Node) -> Result<()> {
    trim_node_left_whitespace(node)?;
    trim_node_right_whitespace(node)?;
    Ok(())
  }

  pub fn add_resource(&mut self, resource: Resource) -> Result<()> {
    // let savenode_opt = self.float_to_element("ltx:resource", false);
    let savenode_opt = None;
    let mut attrib: HashMap<String, String> = HashMap::default();
    attrib.insert(s!("src"), resource.name);
    attrib.insert(s!("type"), resource.mimetype);
    attrib.insert(s!("media"), resource.media);
    let content_box = Digested::from(Tbox {
      text: arena::pin(resource.content),
      ..Tbox::default()
    });
    self.insert_element("ltx:resource", vec![&content_box], Some(attrib))?;
    if let Some(savenode) = savenode_opt {
      self.set_node(&savenode);
    }
    Ok(())
  }

  pub fn process_pending_resources(&mut self) -> Result<()> {
    let resources: Vec<Resource> = state::take_pending_resources();
    for resource in resources {
      self.add_resource(resource)?;
    }
    state::reset_pending_resources();
    Ok(())
  }

  pub fn make_error(&mut self, error_class: &str, content: &str) -> Result<()> {
    let savenode_opt = if !self.is_openable("ltx:ERROR") {
      self.float_to_element("ltx:ERROR", false)?
    } else {
      None
    };
    self.open_element("ltx:ERROR", Some(string_map!("class"=>error_class)), None)?;
    // Perl `Document.pm:makeError` L1346: `openText_internal($self,
    // ToString($content))`. Drops the failing token name (`\foo`,
    // `\bar`, …) as visible text inside the ERROR element so the
    // HTML5 `<span class="ltx_ERROR ...">` is not empty. Without
    // this, the user sees a zero-width invisible span where the
    // problem source should be.
    if !content.is_empty() {
      self.open_text_internal(content)?;
    }
    self.close_element("ltx:ERROR")?;
    if let Some(savenode) = savenode_opt {
      self.set_node(&savenode);
    }
    Ok(())
  }

  // The following "floatTo" operations find an appropriate point
  // within the document tree preceding the current insertion point.
  // They return undef (& issue a warning) if such a point cannot be found.
  // Otherwise, they move the current insertion point to the appropriate node,
  // and return the previous insertion point.
  // After you make whatever changes (insertions or whatever) to the tree,
  // you should do
  //   document.set_node(savenode)
  // to reset the insertion point to where it had been.

  /// Find a node in the document that can contain an element `qname`
  pub fn float_to_element(&mut self, qname: &str, closeifpossible: bool) -> Result<Option<Node>> {
    let mut candidates: VecDeque<Node> = VecDeque::from(self.get_insertion_candidates(&self.node));
    let mut closeable = true;
    // If the current node can contain already, we're fine right here - just return
    if !candidates.is_empty() && can_contain(&candidates[0], qname) {
      // Edge case: Don't resume at a text node, if it is current.
      // Don't append more to it after other insertions.
      if self.node.get_type() == Some(NodeType::TextNode) {
        self.set_node(&candidates[0]);
      }
      return Ok(candidates.pop_front());
    }
    while !candidates.is_empty() && !can_contain(&candidates[0], qname) {
      if closeable {
        closeable = can_auto_close(&candidates[0]);
      }
      candidates.pop_front();
    }
    if let Some(n) = candidates.pop_front() {
      if closeifpossible && closeable {
        self.close_to_node(&n, false)?;
      } else {
        let savenode = self.node.clone();
        self.set_node(&n);
        // Debug!("Floating from " . Stringify($savenode) . " to " . Stringify($n) . " for $qname")
        //   if ($$savenode ne $$n) && $LaTeXML::DEBUG{document};
        return Ok(Some(savenode));
      }
    } else if can_contain_node_somehow(&self.node, qname).is_none() {
      Warn!(
        "malformed",
        qname,
        s!("No open node can contain element '{}'", qname)
      );
      // self.get_insertion_context())
    }
    Ok(None)
  }

  // find a node that can accept a label.
  // A bit more than just whether the element can have the attribute, but
  // whether it has an id (and ideally either a refnum or title)
  pub fn float_to_label(&mut self) -> Option<Node> {
    let key = "labels";
    // Perl: start from lastChild of current node if it's an element
    let start = if self.node.get_type() == Some(NodeType::ElementNode) {
      self
        .node
        .get_last_child()
        .unwrap_or_else(|| self.node.clone())
    } else {
      self.node.clone()
    };
    let ancestors: Vec<Node> = self
      .get_insertion_candidates(&start)
      .into_iter()
      .filter(|node| node.get_type() == Some(NodeType::ElementNode))
      .collect();
    let mut candidates: VecDeque<&Node> = ancestors.iter().collect();
    // Should we only accept a node that already has an id, or should we create an id?
    let mut node_opt: Option<Cow<Node>> = None;
    while let Some(candidate) = candidates.pop_front() {
      if can_node_have_attribute(candidate, key) && candidate.has_attribute_ns("id", xml::XML_NS) {
        node_opt = Some(Cow::Borrowed(candidate));
        break;
      }
    }

    if node_opt.is_none() {
      // No appropriate ancestor?
      let sib: Option<Node> = match ancestors.first() {
        Some(n) => n.get_last_child(),
        None => None,
      };
      if let Some(sibling) = sib {
        if can_node_have_attribute(&sibling, key) && sibling.has_attribute_ns("id", xml::XML_NS) {
          node_opt = Some(Cow::Owned(sibling));
        } else if !ancestors.is_empty() {
          // just take root element?
          node_opt = Some(Cow::Borrowed(ancestors.last().as_ref().unwrap()));
        }
      } else if !ancestors.is_empty() {
        // just take root element?
        node_opt = Some(Cow::Borrowed(ancestors.last().as_ref().unwrap()));
      }
    }
    if let Some(node) = node_opt {
      let savenode = self.node.clone();
      self.set_node(&node);
      Some(savenode)
    } else {
      let message = s!("No open node with an xml:id can get attribute {:?}", key);
      Warn!("malformed", key, message);
      //  $self->getInsertionContext());
      None
    }
  }

  pub fn set_box_to_absorb(&mut self, arg: Option<Digested>) {
    self.localized_boxes.push(self.box_to_absorb.take());
    self.localized_box_locators.push(self.current_box_locator.take());
    self.box_to_absorb = arg;
    // Capture the locator now, while the box's RefCell is unborrowed — the
    // source-map stamping (`open_element`) reads this Copy value instead of
    // re-borrowing the box mid-`be_absorbed`. Gated so the normal path is free.
    self.current_box_locator = if state::source_map_enabled() {
      self.box_to_absorb.as_ref().and_then(|b| b.get_locator())
    } else {
      None
    };
  }
  pub fn expire_box_to_absorb(&mut self) {
    self.box_to_absorb = self.localized_boxes.pop().unwrap();
    self.current_box_locator = self.localized_box_locators.pop().unwrap_or(None);
  }

  /// token-locators: directly set the locator used to stamp the NEXT opened
  /// element, without touching the `box_to_absorb` stack. The alignment absorb
  /// uses this to give each `tabular`/`tr`/`td` its own (table/row/cell) span,
  /// since those elements are opened *before* their content's `box_to_absorb`
  /// is set. Transient: each cell overwrites it and the enclosing
  /// `expire_box_to_absorb` (the Alignment absorb frame) restores the prior
  /// value. See docs/SOURCE_PROVENANCE.md §3.1.3.
  #[cfg(feature = "token-locators")]
  pub fn set_current_box_locator(&mut self, loc: Option<Locator>) {
    self.current_box_locator = loc;
  }

  pub fn load_labels_for_rewrite(&mut self) -> Result<()> {
    for mut node in self.findnodes("//*[@labels]", None) {
      if let Some(labels) = node.get_attribute("labels") {
        // A labelled node MUST carry an xml:id so `\ref` can resolve to it.
        // Normally the `Tag('ltx:*', afterClose:late)` GenerateID hook
        // (latex_constructs.rs) stamps one, but it does not reach every node
        // — notably the <ltx:document> root, which receives a label when a
        // bare `\label{…}` appears with no enclosing id'd sectioning (e.g.
        // `\input{abs}` then `\label{sec:intro}` before any \section; witness
        // 1703.09326). Perl handles this by giving the root an xml:id — its
        // output is `<document … labels="LABEL:sec:intro" xml:id="id1">` —
        // NOT by erroring. Match Perl: generate an id here when one is
        // missing, exactly as Perl's GenerateID does (an id-less root yields
        // "id1" from `generate_id`'s empty-prefix→"id", no-ancestor path).
        let id = match node.get_attribute("id") {
          Some(id) => Some(id),
          None => {
            self.generate_id(&mut node, "")?;
            node.get_attribute_ns("id", XML_NS)
          },
        };
        if let Some(id) = id {
          for label in labels.split_whitespace() {
            self
              .rewrite_labels
              .insert(label.to_string(), id.clone());
          }
        }
        // If generate_id still couldn't assign one (a node the model forbids
        // an xml:id on), the label is simply unresolvable — drop it silently
        // (Perl does not error here either).
      }
    }
    Ok(())
  }

  fn set_local_font(&mut self, arg: Rc<Font>) { self.localized_fonts.push(arg); }
  fn get_local_font(&self) -> Option<Rc<Font>> { self.localized_fonts.last().cloned() }
  fn expire_local_font(&mut self) { self.localized_fonts.pop(); }

  //**********************************************************************
  /// This function computes an xml:id for a node, if it hasn't already got one.
  /// It is suitable for use in Tag afterOpen as
  ///  `Tag('ltx:para',afterOpen=>sub { GenerateID(@_,'p'); });`
  /// It generates an id of the form `<parentid>.<prefix><number>`
  /// The parent node (the one with `ID=<parentid>`) also maintains a counter
  /// stored in an attribute `_ID_counter_<prefix>` recording the last used
  /// `number` for `prefix` amongst its descendents.
  pub fn generate_id(&mut self, node: &mut Node, mut prefix: &str) -> Result<()> {
    // If node doesn't already have an id, and can
    // but isn't a _Capture_ node (which ultimately should disappear)
    let qname = get_node_qname(node);
    if !node.has_attribute_ns("id", XML_NS)
      && model::can_have_attribute(qname, pin!("xml:id"))
      && (qname != pin!("ltx:_Capture_"))
    {
      let mut ancestor = self
        .findnode("ancestor::*[@xml:id][1]", Some(node))
        .unwrap_or_else(|| self.get_document().get_root_element().unwrap());
      //// Old versions don't like ancestor.getAttribute('xml:id');
      let ancestor_id = ancestor.get_attribute_ns("id", XML_NS);
      // If we've got no ancestor_id, then we've got no ancestor (no document yet!),
      // or ancestor IS the root element (but without an id);
      // If we also have no prefix, we'll end up with an illegal id (just digits)!!!
      // We'll use "id" for an id prefix; this will work whether or not we have an ancestor.
      if prefix.is_empty() && ancestor_id.is_none() {
        prefix = "id";
      }

      // Perl `Package.pm:939` (`'_ID_counter_' . ($prefix ? $prefix . '_' : '')`)
      // — empty prefix uses `_ID_counter_` with a single trailing underscore,
      // not `_ID_counter__`. Matters for interop with code that reads the
      // attribute by exact name (e.g. `Base_XMath.pool.ltxml:940` reads
      // `_ID_counter_` for the empty-prefix counter).
      let ctrkey = if prefix.is_empty() {
        s!("_ID_counter_")
      } else {
        s!("_ID_counter_") + prefix + "_"
      };
      let a_ctr = ancestor.get_attribute(&ctrkey).unwrap_or_else(|| s!("0"));

      let ctr_int = 1 + a_ctr.parse::<u32>().unwrap_or(0);
      let ctr = ctr_int.to_string();

      let id = match ancestor_id {
        Some(aid) => aid + ".",
        None => String::new(),
      } + prefix
        + &ctr;

      ancestor.set_attribute(&ctrkey, &ctr)?;
      self.set_attribute(node, "xml:id", &id)?;
    }
    Ok(())
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Finally, another set of surgery methods
  // These take an array representation of the XML Tree to append
  //   [tagname,{attributes..}, children]
  // THESE SHOULD BE PART OF A COMMON BASE CLASS; DUPLICATED IN Post::Document
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  pub fn replace_tree(&mut self, new: Node, old: Node) -> Result<Option<Node>> {
    if let Some(mut parent) = old.get_parent() {
      let mut following = VecDeque::new(); // Collect the matching and following nodes
      while let Some(mut sib) = parent.get_last_child() {
        if sib == old {
          break;
        }
        // parent.remove_child(sib); // We're putting these back, in a moment!
        sib.unlink();
        following.push_front(sib);
      }
      // NOTE: remove_node calls old.unlink() which sets unlinked=true and detaches
      // the node from the DOM. The document's node cache holds another Rc reference,
      // so _Node::drop (and xmlFreeNode) doesn't run until the Document itself drops.
      // By that time, any children reused by `new` have been moved at the C level,
      // so xmlFreeNode on old only frees the shell.
      self.remove_node(old);
      self.append_tree(&mut parent, vec![new])?;
      let inserted = parent.get_last_child();
      for mut child in following {
        parent.add_child(&mut child)?; // No need for clone
      }
      Ok(inserted)
    } else {
      Ok(None)
    }
  }

  pub fn append_tree(&mut self, node: &mut Node, data: Vec<Node>) -> Result<()> {
    for child in data {
      match child.get_type() {
        Some(NodeType::ElementNode) => {
          let mut attributes: HashMap<String, String> =
            child.get_attributes().into_iter().collect();
          // Perl appendTree: REMOVE xml:id from source node before re-creation.
          // This prevents duplicate ID registration. The ID will be re-registered
          // by open_element_at when the new node is created with the same ID.
          if let Some(xmlid) = child.get_attribute_ns("id", XML_NS) {
            attributes
              .entry("xml:id".to_string())
              .or_insert_with(|| xmlid.clone());
            // Unrecord before re-creation (Perl: $child->removeAttribute('xml:id') + unRecordID)
            self.unrecord_id(&xmlid);
          }

          let tag_sym = get_node_qname(&child);
          let tag = arena::to_string(tag_sym);
          let mut new = self.open_element_at(node, &tag, Some(attributes), None)?;
          self.append_tree(&mut new, child.get_child_nodes())?;
          self.close_element_at(&mut new)?;
        },
        Some(NodeType::DocumentFragNode) => {
          self.append_tree(node, child.get_child_nodes())?;
        },
        Some(NodeType::TextNode) => {
          node.append_text(&child.get_content())?;
        },
        other => {
          log::debug!("append_tree: unhandled libxml NodeType {other:?}");
        },
      }
    }
    Ok(())
  }
}

// Auxiliary

fn serialize_string(string: &str) -> String {
  // Basic entities
  let mut serialized = string.replace('&', "&amp;");
  serialized = serialized.replace('>', "&gt;");
  serialized = serialized.replace('<', "&lt;");
  serialized
}

fn serialize_attr(string: &str) -> String {
  let mut serialized = serialize_string(string);
  // And escape any remaining special code points
  serialized = serialized.replace('\"', "&quot;");
  serialized = serialized.replace('\n', "&#10;");
  serialized = serialized.replace('\t', "&#9;");
  serialized
}

fn trim_node_left_whitespace(node: &Node) -> Result<()> {
  if let Some(mut first_child) = node.get_first_child() {
    match first_child.get_type() {
      Some(NodeType::TextNode) => {
        let content = first_child.get_content();
        // Perl: s/^ +// — only trim ASCII spaces, preserve unicode spaces (nbsp, em-space, etc.)
        let trimmed_content = content.trim_start_matches(' ');
        if !content.is_empty() && (trimmed_content != content) {
          first_child.set_content(trimmed_content)?;
        }
      },
      Some(NodeType::ElementNode) => trim_node_left_whitespace(&first_child)?,
      _ => {},
    };
  }
  Ok(())
}

fn trim_node_right_whitespace(node: &Node) -> Result<()> {
  // Skip trailing empty <text> font wrapper elements to find the real last content.
  // These are artifacts of font change tracking during alignment absorption.
  let mut candidate = node.get_last_child();
  while let Some(ref child) = candidate {
    if child.get_type() == Some(NodeType::ElementNode)
      && child.get_name() == "text"
      && child.get_child_nodes().is_empty()
      && child.has_attribute("_noautoclose")
    {
      candidate = child.get_prev_sibling();
    } else {
      break;
    }
  }
  if let Some(mut last_child) = candidate {
    match last_child.get_type() {
      Some(NodeType::TextNode) => {
        let content = last_child.get_content();
        // Perl: s/\s+$// — but we can't trim all Unicode whitespace because some
        // tests have significant thin spaces (U+2009) from DimensionToSpaces.
        // Trim: ASCII whitespace, nbsp (U+00A0), em-space (U+2003), en-space (U+2002).
        let trimmed_content = content.trim_end_matches(|c: char| {
          c.is_ascii_whitespace() || c == '\u{00A0}' || c == '\u{2003}' || c == '\u{2002}'
        });
        if !content.is_empty() && (trimmed_content != content) {
          if trimmed_content.is_empty() {
            // Remove the entirely-whitespace text node
            last_child.unlink();
          } else {
            last_child.set_content(trimmed_content)?;
          }
        }
      },
      Some(NodeType::ElementNode) => trim_node_right_whitespace(&last_child)?,
      _ => {},
    };
  }
  Ok(())
}

pub trait IntoVDQS {
  fn into_vdqs(self) -> VecDeque<SymStr>
  where Self: Sized;
}
impl IntoVDQS for SymStr {
  fn into_vdqs(self) -> VecDeque<SymStr> {
    let mut vdq = VecDeque::new();
    vdq.push_front(self);
    vdq
  }
}
impl IntoVDQS for &str {
  fn into_vdqs(self) -> VecDeque<SymStr> {
    let mut vdq = VecDeque::new();
    vdq.push_front(arena::pin(self));
    vdq
  }
}

impl IntoVDQS for VecDeque<SymStr> {
  fn into_vdqs(self) -> VecDeque<SymStr> { self }
}

// containment checks are package-level (and maybe can be moved in a new submodule?)

pub fn can_contain(node: &Node, child: &str) -> bool {
  let tag = model::get_node_qname(node);
  model::can_contain_sym(tag, arena::pin(child))
}

pub fn can_contain_node(node: &Node, child: &Node) -> bool {
  let tag = model::get_node_qname(node);
  let child_tag = model::get_node_qname(child);
  model::can_contain_sym(tag, child_tag)
}

pub fn can_contain_qname(tag: &str, child: &str) -> bool { model::can_contain(tag, child) }

pub fn node_can_contain_sym(node: &Node, child: SymStr) -> bool {
  let tag = model::get_node_qname(node);
  model::can_contain_sym(tag, child)
}
pub fn can_contain_qsym(tag: SymStr, child: SymStr) -> bool { model::can_contain_sym(tag, child) }

/// Can an element with (qualified name) `tag` contain a `childtag` element indirectly?
/// That is, by openning some number of autoOpen'able tags?
/// And if so, return the tag to open.
pub fn can_contain_indirect(tag: SymStr, child: SymStr) -> Option<SymStr> {
  // $tag = $model->getNodeQName($tag) if ref $tag;          // In case tag is a
  // node. $child = $model->getNodeQName($child) if ref $child;    // In case
  // child is a node.
  if !state::has_indirect_model() {
    let i_model = state::compute_indirect_model();
    state::set_indirect_model(i_model);
  }
  state::get_indirect_model_relationship(tag, child)
}

pub fn can_contain_node_somehow(node: &Node, child: &str) -> Option<Option<SymStr>> {
  let child_sym = arena::pin(child);
  sym_can_contain_somehow(model::get_node_qname(node), child_sym)
}

pub fn can_contain_somehow(tag: &str, child: &str) -> bool {
  let tag_sym = arena::pin(tag);
  let child_sym = arena::pin(child);
  sym_can_contain_somehow(tag_sym, child_sym).is_some()
}

/// The return type of this method is somewhat artisinal, as we have a three-way semantics:
/// - `None`: There is no known structure that allows `child` as a descendant of `tag`
/// - `Some(None)`: `child` is directly allowed inside `tag`
/// - `Some(Some(inter_tag))`: `child` is allowed inside `inter_tag`, which is allowed in `tag`
///
/// This could also (maybe more naturally?) be represented with a custom 3-valued enum.
/// That said, I think it may be wiser to refactor the method entirely, always requiring a `bool`
/// check, followed by an explicit request for the `inner_tag` name, which can be `Option<SymStr>`.
pub fn sym_can_contain_somehow(tag: SymStr, child: SymStr) -> Option<Option<SymStr>> {
  match model::can_contain_sym(tag, child) {
    true => Some(None),
    false => can_contain_indirect(tag, child).map(Some),
  }
}

pub fn can_node_have_attribute(node: &Node, attrib: &str) -> bool {
  let qname = model::get_node_qname(node);
  model::can_have_attribute(qname, arena::pin(attrib))
}
pub fn can_have_attribute(tag: &str, attrib: &str) -> bool {
  model::can_have_attribute(arena::pin(tag), arena::pin(attrib))
}
pub fn sym_can_have_attribute(tag: SymStr, attrib: SymStr) -> bool {
  model::can_have_attribute(tag, attrib)
}

// Dirty little secrets:
//  You can generically allow an element to autoClose using Tag.
// OR you can indicate a specific node can autoClose, or forbid it, using
// the _autoclose or _noautoclose attributes!
pub fn can_auto_close(node: &Node) -> bool {
  // text or comments auto close
  // otherwise must be element
  // without _noautoclose
  // and either with _autoclose
  // OR it has autoClose set on tag properties
  match node.get_type() {
    Some(NodeType::TextNode) | Some(NodeType::CommentNode) => true,
    Some(NodeType::ElementNode) if !node.has_attribute("_noautoclose") => {
      if node.has_attribute("_autoclose") {
        true
      } else {
        state::with_tag_property(get_node_qname(node), |props_opt| {
          if let Some(props) = props_opt {
            props.auto_close.unwrap_or(false)
          } else {
            false
          }
        })
      }
    },
    _ => false,
  }
}
/// Get the node's qualified name in standard form.
///
/// Ie. using the registered prefix for that namespace.
/// NOTE: Reconsider how _Capture_ & _WildCard_ should be integrated!?!
/// NOTE: Should Deprecate! (use model)
pub fn get_node_qname(node: &Node) -> SymStr { model::get_node_qname(node) }
pub fn with_node_qname<R, FnR>(node: &Node, caller: FnR) -> R
where FnR: FnOnce(&str) -> R {
  model::with_node_qname(node, caller)
}

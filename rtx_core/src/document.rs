pub mod helpers;
pub mod resource;
pub mod tag;

use lazy_static::lazy_static;
use libxml::tree::set_node_rc_guard;
use libxml::tree::Document as XmlDoc;
use libxml::tree::{Namespace, Node, NodeType};
use regex::Regex;

use std::borrow::Cow;
use std::collections::HashSet;
use std::collections::{VecDeque};
use rustc_hash::{FxHashMap as HashMap};
use std::fmt::Write as _;
use std::sync::Arc;

use crate::common::error::*;
use crate::common::font::{Font, FONT_TEXT_DEFAULT};
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::common::xml::{self, XML_NS};
use crate::definition::FontDirective;
use crate::ligature::Ligature;
use crate::list::List;
use crate::state::State;
use crate::TexMode;

use crate::document::resource::Resource;
use crate::document::tag::{TagConstructionClosure, TagOptionName, TagOptions};
use crate::Tbox;
use crate::{BoxOps, Digested, DigestedData};

lazy_static! {
  static ref HAS_NONSPACE_RE: Regex = Regex::new(r"\S").unwrap();
  static ref ONLY_SPACE_RE: Regex = Regex::new(r"^\s+$").unwrap();
  static ref DASHES_RE: Regex = Regex::new(r"\-\-+").unwrap();
  static ref NON_MERGEABLE_ATTRIBUTES : HashSet<&'static str> =
    HashSet::from(["about","aboutlabelref","aboutidref",
   "resource","resourcelabelref","resourceidref",
   "property","rel","rev","tyupeof","datatype",
   "content","data","datamimetype","dataencoding"]);
  // When merging attributes of two nodes, some attributes should be combined
  // Merged space separated
  static ref MERGE_ATTRIBUTE_SPACEJOIN : HashSet<&'static str> =
    HashSet::from(["class","lists","inlist","labels"]);
  // Merged ";" separated
  static ref MERGE_ATTRIBUTE_SEMICOLONJOIN : HashSet<&'static str> =
    HashSet::from(["cssstyle"]);
  // Summed lengths
  static ref MERGE_ATTRIBUTE_SUMLENGTH : HashSet<&'static str> =
    HashSet::from(["xoffset","yoffset","lpadding","rpadding","xtranslate",
    "ytranslate"]);
}

pub static FONT_ELEMENT_NAME: &str = "ltx:text";
pub static MATH_TOKEN_NAME: &str = "ltx:XMTok";
pub static MATH_HINT_NAME: &str = "ltx:XMHint";
pub struct Document {
  pub document: XmlDoc,
  pub pending: Vec<Node>,
  pub node: Node,
  pub node_boxes: HashMap<usize, Digested>, // used to be _box attribute
  pub node_fonts: HashMap<u64, Font>,            // used to be _font attribute
  pub idstore: HashMap<String, Node>,
  // the rewrite labels used to be in each rewrite rule, but they make more sense in doc
  pub rewrite_labels: HashMap<String, String>,
  // the following are internal "local"-based declarations in Perl
  localized_constructed_nodes: Vec<Vec<Node>>,
  constructed_nodes: Vec<Node>,
  localized_boxes: Vec<Option<Digested>>,
  box_to_absorb: Option<Digested>, // local $LaTeXML::BOX;
  localized_fonts: Vec<Arc<Font>>,
}
impl Default for Document {
  fn default() -> Self { Self::new() }
}
impl Object for Document {
  fn get_locator(&self) -> Option<Cow<Locator>> {
    if let Some(tbox) = self.get_node_box(&self.node) {
      tbox.get_locator().map(|l| Cow::Owned(l.into_owned()))
    } else {
      None
    }
  }
}
impl Document {
  pub fn new() -> Self {
    set_node_rc_guard(10); // We will need a high treshold for Node mutability
    let doc_scaffold = XmlDoc::new().unwrap();
    let root = match doc_scaffold.get_root_element() {
      Some(root) => root,
      None => doc_scaffold.as_node(), // when empty, set the document node as a node.
    };
    Document {
      document: doc_scaffold,
      node: root,
      node_boxes: HashMap::default(),
      node_fonts: HashMap::default(),
      idstore: HashMap::default(),
      rewrite_labels: HashMap::default(),
      pending: Vec::new(),
      localized_constructed_nodes: Vec::new(),
      constructed_nodes: Vec::new(),
      box_to_absorb: None,
      localized_boxes: Vec::new(),
      localized_fonts: Vec::new(),
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

  /// Find the nodes according to the given $xpath expression,
  /// the xpath is relative to $node (if given), otherwise to the document node.
  pub fn findnodes(&self, xpath: &str, node_opt: Option<&Node>, state: &mut State) -> Vec<Node> {
    if let Some(root) = self.document.get_root_element() {
      match node_opt {
        Some(node) => state.model.get_xpath(&self.document).findnodes(xpath, Some(node)),
        None => state.model.get_xpath(&self.document).findnodes(xpath, Some(&root)),
      }
    } else {
      vec![]
    }
  }

  /// Like findnodes, but only returns the first matched node
  pub fn findnode(&self, xpath: &str, node: Option<&Node>, state: &mut State) -> Option<Node> {
    let mut nodes = state.model.get_xpath(&self.document).findnodes(xpath, node);
    if nodes.is_empty() {
      None
    } else {
      Some(nodes.remove(0))
    }
  }

  /// Like findnodes, but expects an xpath that evaluates to a literal value (e.g. for attributes)
  pub fn findvalues(&self, xpath: &str, node_opt: Option<&Node>, state: &mut State) -> Vec<String> {
    match node_opt {
      Some(node) => state.model.get_xpath(&self.document).findvalues(xpath, Some(node)),
      None => {
        if let Some(root) = self.document.get_root_element() {
          state.model.get_xpath(&self.document).findvalues(xpath, Some(&root))
        } else {
          Vec::new()
        }
    }
    }
  }

  /// Get the node's qualified name in standard form
  /// Ie. using the registered prefix for that namespace.
  /// NOTE: Reconsider how _Capture_ & _WildCard_ should be integrated!?!
  /// NOTE: Should Deprecate! (use model)
  pub fn get_node_qname(&self, node: &Node, state: &State) -> String { state.model.get_node_qname(node) }

  pub fn get_node(&self) -> &Node { &self.node }
  pub fn get_node_mut(&mut self) -> &mut Node { &mut self.node }

  pub fn get_document(&self) -> &XmlDoc { &self.document }
  pub fn get_document_mut(&mut self) -> &mut XmlDoc { &mut self.document }

  // **********************************************************************
  // This should be called before returning the final XML::LibXML::Document to the
  // outside world.  It resolves the fonts for each node relative to it's
  // ancestors. It removes the `helper' attributes that store fonts, source
  // box, etc.
  pub fn finalize(&mut self, state: &mut State) -> Result<()> {
    self.prune_xmduals(state)?;
    if let Some(mut root) = self.document.get_root_element() {
      self.set_local_font(Arc::new(Font::text_default()));
      self.finalize_rec(&mut root, state)?;
      if let Some(Stored::String(prefixes)) = state.lookup_value("RDFa_prefixes") {
        self.set_rdfa_prefixes(Some(prefixes));
      }
      self.expire_local_font();
    }
    Ok(())
  }

  fn finalize_rec(&mut self, node: &mut Node, state: &mut State) -> Result<()> {
    let qname = state.model.get_node_qname(node);
    let local_font = self.get_local_font().unwrap();
    // _standalone_font is typically for metadata that gets extracted out of context
    let mut declared_font = if node.has_attribute("_standalone_font") {
      Cow::Borrowed(&*FONT_TEXT_DEFAULT)
    } else {
      Cow::Borrowed(&*local_font)
    };

    if let Some(_comment) = node.get_attribute("_pre_comment") {
      if let Some(_parent) = node.get_parent() {
        // parent.: Option<Node> insert_before(XML::LibXML::Comment.new(comment), node);
        unimplemented!();
      }
    }
    if let Some(_comment) = node.get_attribute("_comment") {
      if let Some(_parent) = node.get_parent() {
        // parent.add_next_sibling(XML::c0.:(omment.new(comment), );
        unimplemented!();
      }
    }

    let mut keys_to_remove: Vec<String> = Vec::new();
    let mut attrs_to_set: Vec<(String, String)> = Vec::new();
    let mut pending_declaration = HashMap::default();

    if self.has_node_font(node) {
      let desired_font = self.get_node_font(node);
      pending_declaration = desired_font.relative_to(&declared_font);
      if (!node.get_child_nodes().is_empty() || node.has_attribute("_force_font")) && !pending_declaration.is_empty() {
        for (key, (value, properties)) in &pending_declaration {
          if state.model.can_have_attribute(&qname, key) {
            attrs_to_set.push((key.to_string(), value.to_string()));
            // Merge to set the font currently in effect
            declared_font = Cow::Owned(declared_font.merge(properties.clone()));
            keys_to_remove.push(key.to_string());
          }
        }

        for (key, mut value) in attrs_to_set {
          if key == "class" { // Generalize?
            if let Some(ovalue) = node.get_attribute("class") {
              value.push(' ');
              value.push_str(&ovalue);
            }
          }
          self.set_attribute(node, &key, &value, state)?;
        }
        for key in keys_to_remove {
          pending_declaration.remove(&key);
        }
      }
    }
    // Optionally add ids to all nodes (AFTER all parsing, rearrangement, etc)
    if qname != "ltx:document"
      && state.lookup_bool("GENERATE_IDS")
      && !node.has_attribute("xml:id")
      && self.can_have_attribute(&qname, "xml:id", state)
    {
      self.generate_id(node, "", state)?;
    }
    self.set_local_font(Arc::new(declared_font.into_owned()));
    for mut child in node.get_child_nodes() {
      let child_type = child.get_type();
      if child_type == Some(NodeType::ElementNode) {
        let was_forcefont = child.has_attribute("_force_font");
        self.finalize_rec(&mut child, state)?;
        // Also check if child is  FONT_ELEMENT_NAME  AND has no attributes
        // AND providing node can contain that child's content, we'll collapse it.
        if (state.model.get_node_qname(&child) == FONT_ELEMENT_NAME) && !was_forcefont && child.get_attributes().is_empty() {
          let grandchildren = child.get_child_nodes();
          if grandchildren
            .iter()
            .all(|gchild| self.can_contain_qname(&qname, &state.model.get_node_qname(gchild), state))
          {
            Debug!("will replace {} grandchildren nodes in finalize_rec", grandchildren.len());
            self.replace_node(child, grandchildren)?;
          }
        }
      }
      // On the other hand, if the font declaration has NOT been effected,
      // We'll need to put an extra wrapper around the text!
      else if child_type == Some(NodeType::TextNode) {
        let mut keys_to_remove = Vec::new();
        // Remove any pending declarations that can't be on FONT_ELEMENT_NAME
        for key in pending_declaration.keys() {
          if !self.can_have_attribute(FONT_ELEMENT_NAME, key, state) {
            keys_to_remove.push(key.to_string());
          }
        }
        for key in keys_to_remove {
          pending_declaration.remove(&key);
        }
        if self.can_contain(node, FONT_ELEMENT_NAME, state) && !pending_declaration.is_empty() {
          // Too late to do wrapNodes?
          if let Some(mut text) = self.wrap_nodes(FONT_ELEMENT_NAME, vec![child], state)? {
            for (key, (value, _properties)) in &pending_declaration {
              self.set_attribute(&mut text, key, value, state)?;
            }
            self.finalize_rec(&mut text, state)?; // Now have to clean up the new node!
          }
        }
      }
    }

    // Attributes that begin with (the semi-legal) "_" are for Bookkeeping.
    // Remove them now.
    for (name, _) in node.get_attributes() {
      if name.starts_with('_') {
        node.remove_attribute(&name)?;
      }
    }
    self.expire_local_font();
    Ok(())
  }

  /// %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  /// Document construction at the Current Insertion Point.
  /// %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  ///
  /// **********************************************************************
  /// absorb the given $box into the DOM (called from constructors).
  /// This will return a list of whatever nodes were created.
  /// Note that this may include nodes that are children of other nodes in the list
  /// or nodes that are no longer in the document.
  /// Also, note that when a text nodes is appended to, the complete text node is in the list,
  /// not just the portion that was added.
  /// [Note that recording the nodes being constructed isn't all that costly,
  /// but filtering them for parent/child relations IS, particularly since it usually isn't needed]
  ///
  /// A $box that is a TBox, or List, or Whatsit, is responsible for carrying out
  /// its own insertion, but it should ultimately call methods of Document
  /// that will record the nodes that were created.
  /// $box can also be a plain string (Digested::Postponed)
  /// which will be inserted according to whatever
  /// font, mode, etc, are in %props.
  pub fn absorb(&mut self, object: &Digested, props_opt: Option<HashMap<String, Stored>>, state: &mut State) -> Result<()> {
    use DigestedData::*;
    let props = props_opt.unwrap_or_default();
    let mut boxes = vec![Cow::Borrowed(object)];
    while let Some(front_box) = boxes.pop() {
      match front_box.data() {
        List(ref list) => {
          // Simply unwind Lists to avoid unneccessary recursion; This occurs quite frequently!
          for tbox in list.unlist().into_iter().rev() {
            boxes.push(Cow::Owned(tbox));
          }
        },
        // A Proper Box or Whatsit? Absorb it.
        TBox(ref digested) => {
          self.set_box_to_absorb(Some((*front_box).clone()));
          self.init_constructed_nodes();
          digested.be_absorbed(self, state)?;
          // record these for OUTER caller!
          // but return only the most recent set
          {
            for n in self.drain_constructed_nodes() {
              self.record_constructed_node(&n);
            }
          }
          self.expire_box_to_absorb();
        },
        Whatsit(ref digested) => {
          self.set_box_to_absorb(Some((*front_box).clone()));
          self.init_constructed_nodes();
          digested.read().unwrap().be_absorbed(self, state)?;
          // record these for OUTER caller!
          // but return only the most recent set
          {
            for n in self.drain_constructed_nodes() {
              self.record_constructed_node(&n);
            }
          }
          self.expire_box_to_absorb();
        },
        Comment(ref comment) => {
          comment.be_absorbed(self, state)?;
        },
        Postponed(ref tokens) => {
          if !matches!(props.get("isMath"), Some(&Stored::Bool(true))) {
            let text_font_opt = if let Some(Stored::Font(ref prop_font)) = props.get("font") {
              Some(Arc::clone(prop_font))
            } else {
              match self.box_to_absorb {
                Some(ref thisbox) => thisbox.get_font(state)?.map(|thisfont| Arc::new(thisfont.into_owned())),
                None => None,
              }
            };
            // TODO: Sometimes we can't find a `font` here. Should `open_text` allow a None font arg?
            let text_font = text_font_opt.unwrap_or_default();
            if let Some(new_text) = self.open_text(&tokens.to_string(), &text_font, state)? {
              self.record_constructed_node(&new_text);
            }
          } else {
            unimplemented!();
          }
        },
        KeyVals(_) => unimplemented!(),
        RegisterValue(_) => unimplemented!(),
      }
    }
    Ok(())
  }

  fn init_constructed_nodes(&mut self) {
    self.localized_constructed_nodes.push(
      self.constructed_nodes.drain(..).collect()
    );
  }
  fn drain_constructed_nodes(&mut self) -> Vec<Node> {
    let drained = self.constructed_nodes.drain(..).collect();
    if let Some(saved) = self.localized_constructed_nodes.pop() {
      self.constructed_nodes = saved;
    }
    drained
  }
  pub fn get_constructed_nodes(&self) -> &[Node] {
    &self.constructed_nodes
  }

  /// This is a refactored `else` cases from the main absorb routine, to allow for better type
  /// hygiene
  pub fn absorb_string(&mut self, object: &str, props: &HashMap<String, Stored>, state: &mut State) -> Result<Option<Node>> {
    // Else, plain string in text mode.
    let ismath: bool = match props.get("isMath") {
      Some(v) => v.into(),
      None => false,
    };
    if !ismath {
      let font: Font = match props.get("font") {
        Some(Stored::Font(fnt)) => (**fnt).clone(),
        Some(Stored::FontDirective(FontDirective::Asset(fnt))) => (**fnt).clone(),
        Some(Stored::FontDirective(FontDirective::Closure(code))) => code(None, state)?,
        _ => self.box_to_absorb.as_ref().unwrap().get_font(state)?.unwrap().into_owned(),
      };
      self.open_text(object, &font, state)
    } else if self.get_node_qname(&self.node, state) == MATH_TOKEN_NAME {
      // Or plain string in math mode.
      // Note text nodes can ONLY appear in <XMTok> or <text>!!!
      // Have we already opened an XMTok? Then insert into it.
      Ok(Some(self.open_math_text_internal(object, state)?))
    // Else create the XMTok now.
    } else {
      // Odd case: constructors that work in math & text can insert raw strings in Math mode.
      let font_math_opt = match props.get("font") {
        Some(Stored::Font(fnt)) => Some(Cow::Borrowed(&**fnt)),
        Some(Stored::FontDirective(FontDirective::Asset(fnt))) => Some(Cow::Borrowed(&**fnt)),
        Some(Stored::FontDirective(FontDirective::Closure(code))) => Some(Cow::Owned(code(None, state)?)),
        _ => None,
      };
      if let Some(font_math) = font_math_opt {
        Ok(Some(
          self.insert_math_token(object, HashMap::default(), Some(&font_math), state)?))
      } else {
        Ok(Some(
        self.insert_math_token(object, HashMap::default(), None, state)?))
      }
    }
  }
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  /// Shorthand for open,absorb,close, but returns the new node.
  pub fn insert_element(&mut self, qname: &str, content: Vec<&Digested>, attrib: Option<HashMap<String, String>>, state: &mut State) -> Result<Node> {
    // TODO: Quickly hacked together, needs a careful refactor with all .clone()
    // calls removed
    let node = self.open_element(qname, attrib, None, state)?;
    Debug!("Inserting element {:?} with body: {:?}", qname, content);
    for digested in content {
      self.absorb(digested, None, state)?;
    }

    let self_node = self.node.get_parent().unwrap();
    let mut c = Some(self_node);
    while c.is_some() && c.as_ref() != Some(&node) && c.as_ref().unwrap().get_type() != Some(NodeType::DocumentNode) {
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
      self.close_element(qname, state)?;
    }
    Ok(node)
  }

  /// Insert a ProcessingInstruction of the form <?op attr=value ...?>
  /// Does NOT move the current insertion point to the PI,
  /// but may move up past a text node.
  // Rust note: attrib would have been best as Vec<(String,String)> but
  // currently quote!() doesn't work out of the box on them
  pub fn insert_pi(&mut self, op: &str, attributes_opt: Option<HashMap<String, String>>) -> Result<()> {
    let mut attr_data = Vec::new();
    if let Some(attributes) = attributes_opt {
      let mut keys = vec![String::from("class"), String::from("package"), String::from("options")];
      let other_keys = attributes
        .keys()
        .filter(|k| k.as_str() != "class" && k.as_str() != "package" && k.as_str() != "options")
        .map(ToString::to_string)
        .collect::<Vec<String>>();
      keys.extend(other_keys);
      for key in keys {
        if let Some(value) = attributes.get(&key) {
          attr_data.push(s!("{}=\"{}\"", key, value));
        }
      }
    }
    // self.close_text_internal();  // Close any open text node
    let mut pi_node = self.document.create_processing_instruction(op, &attr_data.join(" ")).unwrap();
    if self.node.get_type() == Some(NodeType::DocumentNode) {
      self.pending.push(pi_node);
    } else {
      self.node.add_prev_sibling(&mut pi_node)?;
    }
    Ok(())
  }

  pub fn open_element(
    &mut self,
    qname: &str,
    attributes: Option<HashMap<String, String>>,
    font_opt: Option<&Font>,
    state: &mut State,
  ) -> Result<Node> {
    // NoteProgress('.') if (self.progress}++ % 25) == 0;
    Debug!("Open element {:?} at {:?}", qname, self.get_node_qname(&self.node, state));
    let mut point = self.find_insertion_point(qname, None, state)?;

    let newnode = self.open_element_at(&mut point, qname, attributes, font_opt.cloned(), state)?;
    self.set_node(&newnode);
    // Underscore attributes such as _box and _font from LaTeXML-proper are now
    // bookkept in special substructs of Document Connected to the node hash.
    // Ideally should be as quick to recompute natively as it would be to set/get
    // attributes externally via libxml.
    //
    // TODO: also accept a _box argument eventually? Or store differently?
    // attributes.entry("_box").or_insert(state.locals.box);

    Ok(newnode)
  }

  /// Note: This closes the deepest open node of a given type.
  /// This can cause problems with auto-opened nodes, esp. ones for fontswitches!
  /// Since this is an "explicit request", we're currently skipping over those nodes,
  /// ie. we're automatically closing them, even if they're the same type as we're asking to
  /// close!!! This is kinda risky! Maybe we should try to request closing of specific nodes.
  pub fn close_element(&mut self, qname: &str, state: &mut State) -> Result<Option<Node>> {
    Debug!("Close element {:?} at {:?}", qname, self.document.node_to_string(&self.node));
    self.close_text_internal(state)?;
    let mut node = self.node.clone();
    let mut cant_close = Vec::new();
    while node.get_type() != Some(NodeType::DocumentNode) {
      let t = state.model.get_node_qname(&node);
      // autoclose until node of same name BUT also close nodes opened' for font
      // switches!
      if t == qname && !(t == FONT_ELEMENT_NAME && node.has_attribute("_fontswitch")) {
        break;
      }
      if !self.can_auto_close(&node, state) {
        cant_close.push(node.clone());
      }
      node = node.get_parent().unwrap();
    }

    if node.get_type() == Some(NodeType::DocumentNode) {
      // Didn't find $qname at all!!
      let qname_msg: String = match qname {
        "#PCDATA" => qname.to_owned(),
        _ => s!("</{}>", qname),
      };
      let message = s!(
        "Attempt to close {}, which isn't open. Currently in {}",
        qname_msg,
        self.get_insertion_context(None, state)
      );
      Error!("malformed", qname, self, state, message);
      Ok(None)
    } else {
      // Found node.
      if !cant_close.is_empty() {
        // Intervening non-auto-closeable nodes!!
        let message = s!(
          "Closing tag {:?} whose open descendents do not auto-close. Descendants are {:?}",
          qname,
          cant_close.into_iter().map(|n| n.get_name()).collect::<Vec<String>>().join(",")
        );
        Error!("malformed", qname, self, state, message);
      }
      // So, now close up to the desired node.
      self.close_node_internal(&node, state)?;
      Ok(Some(node))
    }
  }

  // Check whether it is possible to open $qname at this point,
  // possibly by autoOpen'ing & autoClosing other tags.
  pub fn is_openable(&self, qname: &str, state: &mut State) -> bool {
    let mut node_opt = Some(self.node.clone());
    while let Some(node) = node_opt {
      let node_qname = self.get_node_qname(&node, state);
      if self.can_contain_somehow(&node_qname, qname, state) {
        return true;
      } else if !self.can_auto_close(&node, state) {
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
  pub fn is_closeable<T: IntoVDQS>(&self, tags: T, state: &mut State) -> Option<Node> {
    let mut tags: VecDeque<String> = tags.into_vdqs();
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
        let this_qname = state.model.get_node_qname(node);
        if this_qname == qname {
          break 'inner;
        }
        if !self.can_auto_close(node, state) {
          Debug!("It was impossible to autoclose node: {:?}", self.document.node_to_string(node));
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
  pub fn maybe_close_element(&mut self, qname: &str, state: &mut State) -> Result<Option<Node>> {
    if let Some(node) = self.is_closeable(qname.to_string(), state) {
      self.close_node_internal(&node, state)?;
      Ok(Some(node))
    } else {
      Ok(None)
    }
  }

  /// Closes all nodes until $node becomes the current point.
  pub fn close_to_node(&mut self, node: &Node, _ifopen: bool, state: &mut State) -> Result<()> {
    let mut cant_close = Vec::new();
    let mut lastopen: Option<Node> = None;
    let mut n = self.node.clone();
    let mut n_type = n.get_type();
    // go up the tree from current node, till we find `node`
    while n_type != Some(NodeType::DocumentNode) && &n != node {
      if !self.can_auto_close(&n, state) {
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
      let message = s!("Attempt to close {:?}, which isn't open", node.get_name());
      Error!("malformed", state.model.get_node_qname(node), self, state, message);
    //     "Currently in " . $self->getInsertionContext()) unless $ifopen;
    } else {
      // Found node.
      if !cant_close.is_empty() {
        // But found has intervening non-auto-closeable nodes!!
        let qname = state.model.get_node_qname(node);
        let message = s!(
          "Closing {:?} whose open descendents do not auto-close. Descendants are: {:?}",
          qname,
          cant_close.into_iter().map(|n| n.get_name()).collect::<Vec<String>>().join(",")
        );
        Error!("malformed", qname, self, state, message);
      }
      if let Some(lastopen_node) = lastopen {
        self.close_node_internal(&lastopen_node, state)?;
      }
    }
    Ok(())
  }

  /// Closes all nodes until $node is closed.
  pub fn close_node(&mut self, node: &Node, state: &mut State) -> Result<()> { self.close_node_with_strictness(true, node, state) }
  /// Only if needed/possible: closes all nodes until $node is closed
  pub fn maybe_close_node(&mut self, node: &Node, state: &mut State) -> Result<()> { self.close_node_with_strictness(false, node, state) }

  pub fn close_node_with_strictness(&mut self, strict: bool, node: &Node, state: &mut State) -> Result<()> {
    // my ($t, @cant_close) = ();
    let mut cant_close: Vec<Node> = Vec::new();
    let mut n = self.node.clone();
    let mut t = node.get_type();
    while t.is_some() && t != Some(NodeType::DocumentNode) && &n != node {
      if !self.can_auto_close(&n, state) {
        cant_close.push(n.clone());
      }
      n = n.get_parent().unwrap();
      t = node.get_type();
    }

    if t == Some(NodeType::DocumentNode) {
      // Didn't find $qname at all!!
      if strict {
        let qname = state.model.get_node_qname(node);
        let message = s!(
          "Attempt to close {}, which isn't open. Currently in {:?}",
          qname,
          self.get_insertion_context(None, state)
        );
        Error!("malformed", qname, self, state, message);
      }
    } else {
      // Found node.
      // Intervening non-auto-closeable nodes!!
      if !cant_close.is_empty() {
        let qname = state.model.get_node_qname(node);
        let message = s!(
          "Closing {} whose open descendents do not auto-close. Descendents are {}",
          qname,
          cant_close.iter().map(Node::get_name).collect::<Vec<String>>().join(", ")
        );
        if strict {
          Error!("malformed", qname, self, state, message);
        } else {
          Info!("malformed", qname, self, state, message);
        }
      }
      self.close_node_internal(node, state)?;
    }
    Ok(())
  }

  // Dirty little secrets:
  //  You can generically allow an element to autoClose using Tag.
  // OR you can indicate a specific node can autoClose, or forbid it, using
  // the _autoclose or _noautoclose attributes!
  pub fn can_auto_close(&self, node: &Node, state: &State) -> bool {
    // text or comments auto close
    // otherwise must be element
    // without _noautoclose
    // and either with _autoclose
    // OR it has autoClose set on tag properties
    match node.get_type() {
      Some(NodeType::TextNode) | Some(NodeType::CommentNode) => true,
      Some(NodeType::ElementNode) => {
        if !node.has_attribute("_noautoclose") {
          if node.has_attribute("_autoclose") {
            true
          } else if let Some(props) = state.tag_properties.get(&self.get_node_qname(node, state)) {
            props.auto_close.unwrap_or(false)
          } else {
            false
          }
        } else {
          false
        }
      },
      _ => false,
    }
  }

  /// get the actions that should be performed on afterOpen or afterClose
  pub fn get_tag_action_list(&self, tag: &str, when: TagOptionName, state: &mut State) -> Vec<TagConstructionClosure> {
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

    let tag_hash = state.tag_properties.entry(tag.to_string()).or_insert_with(TagOptions::default).clone();
    // let ns_hash  = ((defined $p) && $STATE->lookupMapping('TAG_PROPERTIES', $p .
    // ':*')) || {};
    let all_hash = state.tag_properties.entry(s!("ltx:*")).or_insert_with(TagOptions::default).clone();

    let mut actions = Vec::new();
    // we have Rc<> around the closures, so cloning them is cheap - just another
    // pointer with a bumped up reference counter
    if let Some(when0) = when_early {
      actions.extend(tag_hash.get(&when0).clone().unwrap_or_default());
      // ns_hash TODO
      actions.extend(all_hash.get(&when0).clone().unwrap_or_default());
    }

    actions.extend(tag_hash.get(&when).clone().unwrap_or_default());
    // ns_hash TODO
    actions.extend(all_hash.get(&when).clone().unwrap_or_default());

    if let Some(when1) = when_late {
      actions.extend(tag_hash.get(&when1).clone().unwrap_or_default());
      // ns_hash TODO
      actions.extend(all_hash.get(&when1).clone().unwrap_or_default());
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

  pub fn serialize_to_string(&self, state: &mut State) -> String {
    // This line is to use libxml2's built-in serializer w/indentation heuristic.
    // Apparently, libxml2 is giving us "binary" or byte strings which we'd prefer
    // to have as text. return decode('UTF-8',
    // $self->getDocument->toString($format)); } This uses our own serializer
    // emulating libxml2's heuristic indentation.
    //  return $self->serialize_aux($self->getDocument, 0, 0, 1); }
    // This uses our own serializer w/ correct indentation rules.
    self.serialize_aux(&self.document.as_node(), 0, false, false, state)
  }

  /// We ought to try for something close to C14N (http://www.w3.org/TR/xml-c14n),
  /// but keep XML declaration, comments and don't convert empty elements.
  pub fn serialize_aux(&self, node: &Node, depth: usize, noindent: bool, heuristic: bool, state: &mut State) -> String {
    let indent = "  ".repeat(depth);
    let mut serialized = String::new();

    match node.get_type() {
      Some(NodeType::DocumentNode) => {
        serialized.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        if let Some(child) = node.get_first_child() {
          let child_serialized = self.serialize_aux(&child, depth, noindent, heuristic, state);
          serialized.push_str(&child_serialized);
          let mut current_child = child;
          while let Some(sibling) = current_child.get_next_sibling() {
            let sibling_serialized = self.serialize_aux(&sibling, depth, noindent, heuristic, state);
            serialized.push_str(&sibling_serialized);
            current_child = sibling;
          }
        }
      },
      Some(NodeType::ElementNode) => {
        // TODO: handle properly
        // let tag = state.model.get_node_document_qname(&node);
        let tag = node.get_name();
        let children = node.get_child_nodes();
        let mut open_tag = s!("<{tag}");

        let nsnodes = node.get_namespace_declarations();
        for ns in nsnodes {
          let prefix = ns.get_prefix();
          let prefix_declaration = if prefix.is_empty() { s!("xmlns") } else { s!("xmlns:{}", prefix) };
          let href = ns.get_href();
          write!(open_tag, " {prefix_declaration}=\"{href}\"").ok();
        }

        let anodes = node.get_attributes();
        let mut anodes_keys: Vec<&String> = anodes.keys().collect();
        anodes_keys.sort();
        for key in anodes_keys {
          if key == "id" {
            continue;
          } // HACK for xml:id
          let key_serialized = state.model.get_node_document_qname(&node.get_attribute_node(key).unwrap());
          let val_serialized = serialize_attr(&node.get_property(key).unwrap_or_default());
          write!(open_tag, " {key_serialized}=\"{val_serialized}\"").ok();
        }
        // HACK for xml:id for now, assuming last element
        if anodes.contains_key("id") {
          let val_serialized = serialize_attr(&node.get_property("id").unwrap_or_default());
          write!(open_tag, " xml:id=\"{val_serialized}\"").ok();
        }

        let noindent_children: bool = if heuristic {
          // This emulates libxml2"s heuristic
          noindent || children.iter().any(|e| e.get_type() == Some(NodeType::TextNode))
        } else {
          // This is the "Correct" way to determine whether to add indentation
          let node_qname = self.get_node_qname(node, state);
          state.model.can_contain(&node_qname, "#PCDATA")
        };

        if !noindent {
          serialized.push_str(&indent)
        }
        serialized.push_str(&open_tag);
        if !children.is_empty() {
          // with contents.
          serialized.push('>');
          if !noindent_children {
            serialized.push('\n');
          }
          for child in children {
            serialized.push_str(&self.serialize_aux(&child, depth + 1, noindent_children, heuristic, state));
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
        write!(serialized, "<!-- {}-->", serialize_string(&node.get_content())).ok();
      },
      _ => {},
    }
    serialized
  }

  pub fn set_node(&mut self, node: &Node) {
    // TODO: Does the frag_node check still make sense here?

    // if node.get_type() == Some(NodeType::DocumentNode) {  // Whoops
    //   if let Some(first_child) = node.get_first_child() {
    //     if let Some(_) = first_child.get_next_sibling() {
    //       error!("unexpected:multiple-nodes TODO");
    //       // Error('unexpected', 'multiple-nodes', $self,
    // //   "Cannot set insertion point to a DOCUMENT_FRAG_NODE",
    // Stringify($node)); }     } else {
    //       set_node = first_child;
    //     }
    //   } else {
    //       error!("unexpected:empty-nodes TODO");
    //       // Error('unexpected', 'empty-nodes', $self,
    //       //   "Cannot set insertion point to an empty DOCUMENT_FRAG_NODE"); }

    //   }
    self.node = node.clone();
  }

  // Internals
  fn set_rdfa_prefixes(&mut self, _prefixes: Option<&str>) {}

  pub fn insert_math_token(
    &mut self,
    text: &str,
    mut attributes: HashMap<String, String>,
    font_opt: Option<&Font>,
    state: &mut State,
  ) -> Result<Node> {
    attributes.entry(s!("role")).or_insert_with(|| s!("UNKNOWN"));
    // TODO: This seems ported out of order, where should these attributes be
    // getting removed?
    attributes.remove("mode");
    // attributes.remove("stretchy");

    let is_space = attributes.contains_key("isSpace");
    let qname = if is_space { MATH_HINT_NAME } else { MATH_TOKEN_NAME };
    let cur_qname = state.model.get_node_qname(&self.node);
    let text = if is_space && !text.is_empty() && text.chars().all(|c| c.is_whitespace()) {
      "" // Make empty hint, of only spaces
    } else {
      text
    };
    if qname == MATH_TOKEN_NAME && cur_qname == qname {
      // Already INSIDE a token!
      if !text.is_empty() {
        self.open_math_text_internal(text, state)?;
      }
    } else {
      let mut node = self.open_element(qname, Some(attributes), None, state)?;
      // let tbox  = $attributes{_box} || $LaTeXML::BOX;
      let font = match font_opt {
        Some(f) => f.clone(),
        None => match self.box_to_absorb {
          Some(ref tbox) => match tbox.get_font(state)? {
            Some(f) => f.into_owned(),
            None => Font::math_default(), // should never happen?
          },
          None => Font::math_default(), // should never happen?
        },
      };
      self.set_node_font(&mut node, &font)?;
      if let Some(ref digested) = self.box_to_absorb {
        // TODO: The Rc<Digested> node boxes still have some way to go until they are fully ergonomic...
        self.set_node_box(&node, digested.clone());
      }
      if !text.is_empty() {
        self.open_math_text_internal(text, state)?;
      }
      self.close_node_internal(&node, state)?; // Should be safe.
    }
    Ok(self.node.clone())
  }

  /// Insert a new comment, or append to previous comment.
  /// Does NOT move the current insertion point to the Comment,
  /// but may move up past a text node.
  pub fn insert_comment(&mut self, text: &str, state: &mut State) -> Result<Node> {
    // TODO:
    let trimmed = text.trim_end();
    let _clean = DASHES_RE.replace_all(trimmed,"__");
    self.close_text_internal(state)?;    // Close any open text node.
    if self.node.get_type() == Some(NodeType::DocumentNode) {
      // TODO: add "create_comment" (or equiv) to libxml wrapper
      // let comment = self.document.create_comment(s!(" {} ",clean));
      // self.pending.push(comment.clone());
      // Ok(comment)
    // } else {
    //   if let Some(last_child) = self.node.last_child() {
    //     if last_child.get_type() == NodeType::CommentNode {
    //       last_child.set_content(s!("{}\n     {} ",comment.get_content(), clean_text));
    //       return Ok(last_child);
    //     }
    //   }
    //   self.node.add_child(self.document.create_comment(s!(" {} ",clean_text));
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

  pub fn open_text(&mut self, text: &str, font: &Font, state: &mut State) -> Result<Option<Node>> {
    let node_type = self.node.get_type();
    {
      // Ignore initial whitespace
      if (text.is_empty() || ONLY_SPACE_RE.is_match(text))
        && (node_type == Some(NodeType::DocumentNode) || (node_type == Some(NodeType::ElementNode) && !self.can_contain(&self.node, "#PCDATA", state)))
      {
        return Ok(None);
      }
    }
    if matches!(&font.family.as_deref(), Some("nullfont")) {
      return Ok(None);
    };
    Debug!("Insert text {:?} at {:?}", text, self.document.node_to_string(&self.node));

    // Get the desired font attributes, particularly the desired element
    // (usually ltx:text, but let Font override, eg for \emph)
    let declared_font       = self.get_node_font(&self.node);
    let pending_declaration = font.relative_to(declared_font);
    let elementname         = match pending_declaration.get("element") {
      Some((k,_v)) => k,
      None => FONT_ELEMENT_NAME
    };
    // If not at document begin. And not appending text in same font.
    if node_type != Some(NodeType::DocumentNode)
      && !(node_type == Some(NodeType::TextNode) && (font.distance(self.get_node_font(&self.node.get_parent().unwrap())) == 0))
    {
      // then we'll need to do some open/close to get fonts matched.
      let node = self.close_text_internal(state)?; // Close text node, if any.
      let mut bestdiff = 99;
      let rc_node = Arc::new(node);
      let mut closeto: Arc<Node> = Arc::clone(&rc_node);
      let mut n: Arc<Node> = Arc::clone(&rc_node);
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
        if state.model.get_node_qname(&n) != elementname || n.has_attribute("_noautoclose") {
          break;
        }
        match n.get_parent() {
          Some(p) => n = Arc::new(p),
          None => break,
        }
      }

      // Move to best starting point for this text.
      if *closeto != *rc_node {
        self.close_to_node(&closeto, false, state)?;
      }
      if bestdiff > 0 {
        // Open if needed.
        self.open_element(elementname, Some(string_map!("_fontswitch" => "true", "_autoopened" => "true")), Some(font), state)?;
      }
    }

    // Finally, insert the darned text.
    let outnode = self.open_text_internal(text, state)?;
    self.record_constructed_node(&outnode);
    Ok(Some(outnode))
  }

  pub fn can_contain(&self, node: &Node, child: &str, state: &mut State) -> bool {
    let tag = state.model.get_node_qname(node);
    state.model.can_contain(&tag, child)
  }

  pub fn can_contain_qname(&self, tag: &str, child: &str, state: &mut State) -> bool { state.model.can_contain(tag, child) }

  /// Can an element with (qualified name) $tag contain a $childtag element indirectly?
  /// That is, by openning some number of autoOpen'able tags?
  /// And if so, return the tag to open.
  pub fn can_contain_indirect(&self, tag: &str, child: &str, state: &mut State) -> Option<String> {
    // $tag = $model->getNodeQName($tag) if ref $tag;          // In case tag is a
    // node. $child = $model->getNodeQName($child) if ref $child;    // In case
    // child is a node.

    if state.indirect_model.is_none() {
      let new_im = state.compute_indirect_model();
      state.indirect_model = Some(new_im);
    }

    let imodel = state.indirect_model.as_ref().unwrap();
    // returning inner_node
    match imodel.get(tag) {
      Some(sub_m) => sub_m.get(child).map(|node| node.to_string()),
      None => None,
    }
  }

  pub fn can_contain_node_somehow(&self, tag: &Node, child: &str, state: &mut State) -> bool {
    self.can_contain_somehow(&state.model.get_node_qname(tag), child, state)
  }

  pub fn can_contain_somehow(&self, tag: &str, child: &str, state: &mut State) -> bool {
    state.model.can_contain(tag, child) || self.can_contain_indirect(tag, child, state).is_some()
  }

  pub fn can_node_have_attribute(&mut self, node: &Node, attrib: &str, state: &mut State) -> bool {
    state.model.can_have_attribute(&state.model.get_node_qname(node), attrib)
  }
  pub fn can_have_attribute(&mut self, tag: &str, attrib: &str, state: &mut State) -> bool { state.model.can_have_attribute(tag, attrib) }

  pub fn close_text_internal(&mut self, state: &State) -> Result<Node> {
    if self.node.get_type() == Some(NodeType::TextNode) {
      // Current node is text?
      let parent = self.node.get_parent().unwrap();
      let font = self.get_node_font(&parent);
      let ocontent = self.node.get_content();
      let mut content = Cow::Borrowed(&ocontent);
      if let Some(Stored::VecDequeStored(ligatures)) = state.lookup_value("TEXT_LIGATURES") {
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
  pub fn close_node_internal(&mut self, node: &Node, state: &mut State) -> Result<()> {
    let closeto = node.get_parent().unwrap(); // Grab now in case afterClose screws the structure.
    let mut n = self.close_text_internal(state)?; // Close any open text node.
    while n.get_type() == Some(NodeType::ElementNode) {
      self.close_element_at(&mut n, state)?;
      self.auto_collapse_children(&mut n, state)?;
      if *node == n {
        break;
      }
      n = n.get_parent().unwrap();
    }
    self.set_node(&closeto);
    Ok(())
  }

  /// Avoid redundant nesting of font switching elements:
  /// If we're closing a node that can take font switches and it contains
  /// a single FONT_ELEMENT_NAME node; pull it up.
  fn auto_collapse_children(&mut self, node: &mut Node, state: &mut State) -> Result<()> {
    let qname = state.model.get_node_qname(node);
    if qname != "ltx:_Capture_" {
      let mut c = node.get_child_nodes();
      // with single child, AND, $node can have all the attributes that the child has (but at least "font")
      // BUT, it isn"t being forced somehow
      if c.len() == 1
        && (state.model.get_node_qname(&c[0]) == FONT_ELEMENT_NAME)
        && state.model.can_have_attribute(&qname, "font")
        && c[0]
          .get_attributes()
          .keys()
          .filter(|x| !x.starts_with('_'))
          .all(|v| state.model.can_have_attribute(&qname, v) && !(NON_MERGEABLE_ATTRIBUTES.contains(v.as_str())))
        && !c[0].has_attribute("_force_font")
      {
        let mut c_first = c.pop().unwrap();
        let c_first_font = self.get_node_font(&c_first).clone();
        self.set_node_font(node, &c_first_font)?;
        self.remove_node(&mut c_first);
        for mut gc in c_first.get_child_nodes().into_iter() {
          gc.unlink();
          node.add_child(&mut gc)?;
          // self.record_node_ids(&node); // TODO
        }
        // Merge the attributes from the child onto $node
        self.merge_attributes(c_first, node, None)?;
      }
    }
    Ok(())
  }

  pub fn merge_attributes(&mut self, from: Node, to: &mut Node, _force: Option<HashSet<&'static str>>) -> Result<()> {
    for (key, val) in from.get_attributes().iter() {
      // Special case attributes
      if key.as_str() == "xml:id" {
        // Use the replacement id
        if !to.has_attribute(key) {
          // val = self.record_id(val, node); // TODO
          to.set_attribute(key, val)?;
        }
      } else if MERGE_ATTRIBUTE_SPACEJOIN.contains(key.as_str())
        || MERGE_ATTRIBUTE_SEMICOLONJOIN.contains(key.as_str())
        || MERGE_ATTRIBUTE_SUMLENGTH.contains(key.as_str())
      {
        unimplemented!();
      } else if !to.has_attribute(key) {
        // || force...
        // Else if attribute not present on $to, or if we specificallly override it, just copy
        to.set_attribute(key, val)?;
      }
    }
    Ok(())
  }

  pub fn open_text_internal(&mut self, text: &str, state: &mut State) -> Result<Node> {
    if text.is_empty() {
      return Ok(self.node.clone())
    }
    if self.node.get_type() == Some(NodeType::TextNode) {
      // current node already is a text node.
      Debug!("Appending text {:?} to {:?}", text, self.document.node_to_string(&self.node));

      let parent = self.node.get_parent().unwrap();
      if self.box_to_absorb.is_some() && parent.get_attribute("_autoopened").is_some() {
        // TODO:
        // self.append_text_box(parent, self.box_to_absorb);
        // unimplemented!();
      }
      self.node.append_text(text)?;
    } else if HAS_NONSPACE_RE.is_match(text) || self.can_contain(&self.node, "#PCDATA", state) {
      // or text allowed here
      let mut point = self.find_insertion_point("#PCDATA", None, state)?;
      Debug!("Inserting text node for {:?} into {:?}", text, self.document.node_to_string(&point));
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

  // /// Since xml text nodes don't have attributes to record the origining box,
  // /// we need to manage the accumulation of autoOpen'ed boxes
  // /// Indeed, propogate it to ancestors if they were autoOpened for same cause (box)
  // fn append_text_box(&mut self, node: &Node, thisbox: &Digested, state: &State) {
  //   let origbox = self.get_node_box(node);
  //   if origbox.is_some() && thisbox != &**origbox.as_ref().unwrap() { // if not already the same box
  //     let newbox = List::new(vec![origbox, thisbox], state);
  //     self.set_node_box(node, newbox);
  //     let p = node;
  //     // AND, propogate change to autoOpen'd ancestors based on same initial box
  //     while (($p = $p->parentNode) && ($p->nodeType == XML_ELEMENT_NODE)
  //       && $p->getAttribute('_autoopened')
  //       && (($self->getNodeBox($p) || '') eq $origbox)) {
  //       $self->setNodeBox($p, $newbox); } }
  //   return; }

  // Question: Why do I have math ligatures handled within openMathText_internal,
  // but text ligatures handled within closeText_internal ???

  /// Needed externally only for the binding generation
  fn open_math_text_internal(&mut self, text: &str, state: &mut State) -> Result<Node> {
    // And if there's already text???
    let mut node = self.node.clone();
    // my $font = $self->getNodeFont($node);
    node.append_text(text)?;
    // print STDERR "Trying Math Ligatures at \"$string\"\n";
    if !state.nomathparse {
      self.apply_math_ligatures(&mut node, state)?;
    }
    Ok(node)
  }

  // New stategy (but inefficient): apply ligatures until one succeeds,
  // then remove it, and repeat until ALL (remaining) fail.
  fn apply_math_ligatures(&mut self, node: &mut Node, state: &mut State) -> Result<()> {
    let checked_out_ligatures = state.checkout_value("MATH_LIGATURES");
    if let Some(Stored::VecDequeStored(ref stored_ligatures)) = checked_out_ligatures {
      let mut ligatures = stored_ligatures.iter().collect::<VecDeque<_>>();
      while !ligatures.is_empty() {
        let mut matched = false;
        let mut next_ligatures = VecDeque::new();
        while !ligatures.is_empty() {
          let ligature_stored = ligatures.pop_front().unwrap();
          if let Stored::Ligature(ligature) = ligature_stored {
            if self.apply_math_ligature(node, ligature, state)? {
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
            state.checkin_value("MATH_LIGATURES", value);
          }
          return Ok(());
        }
      }
    }
    if let Some(value) = checked_out_ligatures {
      state.checkin_value("MATH_LIGATURES", value);
    }
    Ok(())
  }

  /// Apply ligature operation to `node`, presumed the last insertion into it's parent(?)
  fn apply_math_ligature(&mut self, node: &mut Node, ligature: &Ligature, state: &mut State) -> Result<bool> {
    if let Some((nmatched, newstring, attr)) = (ligature.matcher.as_ref().unwrap())(self, node, state)? {
      let mut boxes = VecDeque::new();
      boxes.push_front(self.get_node_box(node).unwrap());
      node.get_first_child().unwrap().set_content(&newstring)?;
      for _idx in 0..nmatched - 1 {
        let mut remove = node.get_prev_sibling().unwrap();
        boxes.push_front(self.get_node_box(&remove).unwrap());
        self.remove_node(&mut remove);
      }
      // This fragment replaces the node's box by the composite boxes it replaces
      // HOWEVER, this gets things out of sync because parent lists of boxes still
      // have the old ones.  Unless we could recursively replace all of them, we'd better skip it(??)
      if boxes.len() > 1 {
        // TODO: Cloning boxes is BAD. What is a better model?
        let mut list = List::new(boxes.into_iter().collect::<Vec<_>>(), state);
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
      nodes.into_iter().filter(|node| xml::is_descendant_or_self(node, &root)).collect()
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
        if new.iter().all(|other| !xml::is_descendant_or_self(&node, other)) {
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
  pub fn get_insertion_context(&self, levels_opt: Option<usize>, state: &mut State) -> String {
    let mut levels = match levels_opt {
      None => {
        // Default depth is based on verbosity
        if state.verbosity <= 1 {
          Some(5)
        } else {
          None
        }
      },
      Some(t) => Some(t),
    };
    let mut node = self.node.clone();
    let node_type = node.get_type();
    if node_type != Some(NodeType::TextNode) && node_type != Some(NodeType::ElementNode) && node_type != Some(NodeType::DocumentNode) {
      let message = s!(
        "Insertion point is not an element, document or text: {:?}",
        self.document.node_to_string(&node)
      );
      Error!("internal", "context", self, state, message);
      return String::new();
    }
    let mut path = s!("TODO"); //TODO: Stringify($node);
    while let Some(parent_node) = node.get_parent() {
      node = parent_node;
      if let Some(levels_val) = levels {
        levels = Some(levels_val - 1);
        if levels_val <= 1 {
          path = s!("...{}", path);
          break;
        }
      }
      // TODO: $path = Stringify($node) . $path; }
    }
    path
  }

  /// Find the node where an element with qualified name $qname can be inserted.
  /// This will move up the tree (closing auto-closable elements),
  /// or down (inserting auto-openable elements), as needed.
  pub fn find_insertion_point(&mut self, qname: &str, has_opened_opt: Option<String>, state: &mut State) -> Result<Node> {
    self.close_text_internal(state)?; // Close any current text node.
    let cur_qname = state.model.get_node_qname(&self.node);
    // If `qname` is allowed at the current point, we're done.
    if self.can_contain_qname(&cur_qname, qname, state) {
      return Ok(self.node.clone());
    // Else, if we can create an intermediate node that accepts $qname, we'll do
    // that.
    } else if let Some(inter) = self.can_contain_indirect(&cur_qname, qname, state) {
      if (inter != qname) && (inter != cur_qname) {
        // TODO: can we avoid the clone here? there is a mutability conflict...
        let node_font = self.get_node_font(&self.node).clone();
        self.open_element(&inter, None, Some(&node_font), state)?;

        return self.find_insertion_point(qname, Some(inter), state); // And retry insertion (should work now).
      }
    }

    if let Some(has_opened) = has_opened_opt {
      // out of options if already inside an auto-open chain
      let message = s!(
        "failed auto-open through <{}> at inadmissible <{}>. Currently in {}",
        has_opened,
        cur_qname,
        self.get_insertion_context(None, state)
      );
      Error!("malformed", qname, self, state, message);
      Ok(self.node.clone()) // But we'll do it anyway, unless Error => Fatal.
    } else {
      // Now we're getting more desparate...
      // Check if we can auto close some nodes, and _then_ insert the `qname`.
      let mut node = self.node.clone();
      let mut close_to = None;
      while (node.get_type() != Some(NodeType::DocumentNode)) && self.can_auto_close(&node, state) {
        let parent_opt = node.get_parent();
        let parent = match parent_opt {
          None => String::new(),
          Some(ref p) => state.model.get_node_qname(p),
        };
        if self.can_contain_somehow(&parent, qname, state) {
          close_to = Some(node);
          break;
        }
        node = match parent_opt {
          Some(p) => p,
          None => break,
        };
      }
      if let Some(close_to_node) = close_to {
        self.close_node_internal(&close_to_node, state)?; // Close the auto closeable nodes.
        self.find_insertion_point(qname, None, state) // Then retry, possibly w/auto open's
      } else {
        // Didn't find a legit place.
        let message = s!("{:?} isn't allowed in <{}>", qname, cur_qname);
        //"Currently in " self.getInsertionContext());
        Error!("malformed", qname, self, state, message);

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
      if node.get_name() == "_Capture_" {
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

  pub fn node_set_attribute(&mut self, key: &str, value: &str, state: &mut State) -> Result<()> {
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
      let qname = state.model.get_node_qname(&self.node);
      if key.starts_with('_') || state.model.can_have_attribute(&qname, key) {
        self.node.set_attribute(key, value)?;
      }
    } else {
      // Accept any namespaced attributes
      unimplemented!();
      //   my ($ns, $name) = state.model}->decodeQName($key);
      //   if ($ns) {             // If namespaced attribute (must have prefix!
      // let prefix = node.lookupNamespacePrefix($ns);    // namespace already
      // declared? if (!$prefix) {                                    // if
      // namespace not already declared $prefix =
      // state.model}->getDocumentNamespacePrefix($ns, 1);    // get the prefix to use
      // self.getDocument->documentElement->setNamespace($ns, $prefix, 0); }
      // // and declare it if ($prefix eq '//default') {    // Probably
      // shouldn't happen...?       node.setAttribute($name => $value); }
      //     else {
      //       node.setAttributeNS($ns, "$prefix:$name" => $value); } }
      //   else {
      //     node.setAttribute($name => $value); } }
    } // redundant case...
    Ok(())
  }
  pub fn node_get_attribute(&mut self, name: &str) -> Option<String> { self.node.get_attribute(name) }
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Document surgery (?)
  // %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // The following carry out DOM modification but NOT relative to any current
  // insertion point (eg self.node), but rather relative to nodes specified
  // in the arguments.

  // Set any allowed attribute on a node, decoding the prefix, if any.
  // Also records, and checks, any id attributes.
  // [xml:id and namespaced attributes are always allowed]
  pub fn set_attribute(&mut self, node: &mut Node, key: &str, value: &str, state: &State) -> Result<()> {
    if value.is_empty() {
      return Ok(()); // skip if empty
    }
    if key == "xml:id" || key == "id" {
      // If it's an ID attribute
      // Do id book keeping
      self.record_id_with_node(value, node, state);
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
      node.set_attribute("xml:id", value)?; // and bypass all ns stuff
    } else if !key.contains(':') {
      // No colon; no namespace (the common case!)
      // Ignore attributes not allowed by the model,
      // but accept "internal" attributes.
      // let model = state.model;
      // let qname = model.get_node_qname(node);
      // if key.starts_with("_") || model.can_have_attribute(qname, key) {
      node.set_attribute(key, value)?;
      // }
    } else { // Accept any namespaced attributes
      dbg!(key);
      unimplemented!();
    }
    //   my ($ns, $name) = state.model.decodeQName($key);
    //   if ($ns) {             // If namespaced attribute (must have prefix!
    // let prefix = node.lookupNamespacePrefix($ns);    // namespace already
    // declared? if (!$prefix) {                                    // if
    // namespace not already declared $prefix =
    // state.model.getDocumentNamespacePrefix($ns, 1);    // get the prefix to use
    // self.getDocument->documentElement->setNamespace($ns, $prefix, 0); }
    // // and declare it if ($prefix eq '//default') {    // Probably
    // shouldn't happen...?       node.setAttribute($name => $value); }
    //     else {
    //       node.setAttributeNS($ns, "$prefix:$name" => $value); } }
    //   else {
    //     node.setAttribute($name => $value); } } }    // redundant case...
    Ok(())
  }

  fn add_ss_values(&mut self, node: &mut Node, key: &str, values_str: &str, state: &State) -> Result<()> {
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
        self.set_attribute(node, key, &old.join(" "), state)?;
      } else {
        values.sort_unstable();
        self.set_attribute(node, key, &values.join(" "), state)?;
      }
    }
    Ok(())
  }

  pub fn add_class(&mut self, node: &mut Node, class: &str, state: &State) -> Result<()> { self.add_ss_values(node, "class", class, state) }

  //**********************************************************************
  // Association of nodes and ids (xml:id)

  /// Records the association of the current Document `node` with the `id`,
  /// which should be the `xml:id` attribute of the `node`.
  /// Usually this association will be maintained by the methods
  /// that create nodes or set attributes.
  fn record_id(&mut self, id: &str) -> String {
    if let Some(_prev) = self.idstore.get(id) {
      // Whoops! Already assigned!!!
      // Can we recover?
      unimplemented!();
      // if ! self.node.is_same_node(prev) {
      //   let badid = id.to_string();
      //   let id = self.modify_id(id);
      //   Info!("malformed", "id", node, "Duplicated attribute xml:id",
      //     "Using id='$id' on " . Stringify($node),
      //     "id='$badid' already set on " . Stringify($prev));
      // }
    }
    self.idstore.insert(id.to_string(), self.node.clone());
    id.to_string()
  }

  /// Records the association of the given `node` with the `id`,
  /// which should be the `xml:id` attribute of the `node`.
  /// Usually this association will be maintained by the methods
  /// that create nodes or set attributes.
  fn record_id_with_node(&mut self, id: &str, node: &Node, state: &State) -> String {
    let prev_opt = if let Some(prev) = self.idstore.get(id) {
      // Whoops! Already assigned!!!
      // Can we recover?
      if &self.node != prev {
        Some(prev.clone())
      } else { None }
    } else { None };
    if let Some(prev) = prev_opt {
      let badid = id;
      let id = self.modify_id(id.to_owned());
      let message = s!("Duplicated attribute xml:id. Using id='{}' on {} id='{}' already set on {}",
        id, self.document.node_to_string(node),badid, self.document.node_to_string(&prev));
      Info!("malformed", "id", self, state, message);
    }
    self.idstore.insert(id.to_string(), node.clone());
    id.to_string()
  }

  pub fn unrecord_id(&mut self, id: &str) { self.idstore.remove(id); }

  /// These are used to record or unrecord, in bulk, all the ids within a node (tree).
  pub fn record_node_ids(&mut self, node: &Node, state: &mut State) -> Result<()> {
    for mut idnode in self.findnodes("descendant-or-self::*[@xml:id]", Some(node), state) {
      if let Some(id) = idnode.get_attribute_ns("id", XML_NS) {
        let newid = self.record_id_with_node(&id, &idnode, state);
        if newid != id {
          idnode.set_attribute("xml:id", &newid)?;
        }
      }
    }
    Ok(())
  }

  pub fn unrecord_node_ids(&mut self, node: &Node, state: &mut State) {
    for idnode in self.findnodes("descendant-or-self::*[@xml:id]", Some(node), state) {
      if let Some(id) = idnode.get_attribute_ns("id", XML_NS) {
        self.unrecord_id(&id);
      }
    }
  }

  /// Get a new, related, but unique id
  /// Sneaky option: try "ID_SUFFIX" as a suffix for id, first.
  pub fn modify_id(&mut self, id: String ) -> String {
    if self.idstore.contains_key(&id) {
      // Whoops! Already assigned!!!
      // Can we recover?
      let badid = id;
      // if (!$LaTeXML::Core::Document::ID_SUFFIX
      // || $$self{idstore}{ $id = $badid . $LaTeXML::Core::Document::ID_SUFFIX }) {
      // foreach my $s1 (1 .. 26 * 26 * 26) {    # Gotta give up, eventually; is 3 letters enough?
      //   return $id unless $$self{idstore}{ $id = $badid . radix_alpha($s1) }; }
      // Error!("malformed", "id", "Automatic incrementing of ID counters failed", self, state, s!("Last alternative for '{}' is '{}'",id,badid)); } }
      // TODO
      s!("{}a",badid)
    } else {
      id
    }
  }

  pub fn lookup_id(&self, id: &str) -> Option<&Node> { self.idstore.get(id) }

  // ======================================================================
  //  Odd bit:
  //  In an XMDual, in each branch (content, presentation) there will be atoms
  //  that correspond to the input (one will be real, the other an XMRef to the first).
  //  But also there will be additional "decoration" (delimiters, punctuation, etc on the presentation
  //  side; other symbols, bindings, whatever, on the content side).
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
  pub fn mark_xmnode_visibility(&mut self, state: &mut State) -> Result<()> {
    let xmath = self.findnodes("//ltx:XMath/*", None, state);
    for math in xmath.iter() {
      for mut node in self.findnodes("descendant-or-self::*[@_pvis or @_cvis]", Some(math), state) {
        node.remove_attribute("_pvis")?;
        node.remove_attribute("_cvis")?;
      }
    }
    for math in xmath {
      self.mark_xmnode_visibility_aux(math, true, true, state)?;
    }
    Ok(())
  }

  fn mark_xmnode_visibility_aux(&self, mut node: Node, cvis: bool, mut pvis: bool, state: &mut State) -> Result<()> {
    let qname = self.get_node_qname(&node, state);
    if (!cvis || node.has_attribute("_cvis")) && (!pvis || node.has_attribute("_pvis")) {
      return Ok(());
    }
    // Special case: for XMArg used to wrap "formal" arguments on the content side,
    // mark them as visible as presentation as well.
    if cvis && (qname == "ltx:XMArg") {
      pvis = true;
    }
    if cvis {
      node.set_attribute("_cvis", "true")?;
    }
    if pvis {
      node.set_attribute("_pvis", "true")?;
    }
    if qname == "ltx:XMDual" {
      let mut children = xml::element_nodes(&node);
      let c = children.remove(0);
      let p = children.remove(0);
      if cvis {
        self.mark_xmnode_visibility_aux(c, true, false, state)?;
      }
      if pvis {
        self.mark_xmnode_visibility_aux(p, false, true, state)?;
      }
    } else if qname == "ltx:XMRef" {
      match node.get_attribute("idref") {
        None => {
          let key = node.get_attribute("_xmkey");
          Warn!("expected", "id", self, state, "Missing idref on ltx:XMRef",s!("_xmkey is `{}`",key.unwrap_or_default()));
        },
        Some(id) => {
          match self.lookup_id(&id) {
            None => {
              Warn!("expected", "node", self, state, s!("No node found with id='{id}' (referred to from ltx:XMRef)"));
            },
            Some(reffed) => {
              self.mark_xmnode_visibility_aux(reffed.clone(), cvis, pvis, state)?;
            }
          }
        }
      }
    } else {
      for child in xml::element_nodes(&node) {
        self.mark_xmnode_visibility_aux(child, cvis, pvis, state)?;
      }
    }
    Ok(())
  }

  /// Reduce any ltx:XMDual's to just the visible branch, if the other is not visible
  /// (according to markXMNodeVisibility)
  /// If we could be 100% sure that the marking had stayed consistent (after various doc surgery)
  /// we could avoid re-marking, but we'd better be sure before removing nodes!
  fn prune_xmduals(&mut self, state:&mut State) -> Result<()> {
    // RE-mark visibility!
    self.mark_xmnode_visibility(state)?;
    // will reversing keep from problems removing nodes from trees that already have been removed?
    for mut dual in self.findnodes("descendant-or-self::ltx:XMDual", None, state).into_iter().rev() {
      self.document.node_to_string(&dual);
      let mut dual_children = xml::element_nodes(&dual);
      let presentation = dual_children.pop().unwrap();
      let content = dual_children.pop().unwrap();
      if self.findnode("descendant-or-self::*[@_pvis or @_cvis]", Some(&content), state).is_none() { // content never seen
        self.collapse_xmdual(&mut dual, presentation, state)?;
      } else if self.findnode("descendant-or-self::*[@_pvis or @_cvis]", Some(&presentation), state).is_none() { // pres.
        self.collapse_xmdual(&mut dual, content, state)?;
      } else { // compact aligned structures, where possible
        self.compact_xmdual(dual, content, Some(presentation), state)?;
      }
    }
    Ok(())
  }

  // our $content_transfer_overrides = { map { ($_ => 1) } qw(decl_id meaning name omcd) };
  // our $dual_transfer_overrides    = { %$content_transfer_overrides,
  //   map { ($_ => 1) } qw(xml:id role) };

  fn compact_xmdual(&mut self, _dual: Node, _content: Node, _presentation: Option<Node>, _state: &mut State) -> Result<()> {
  //   my $c_name = $self->getNodeQName($content);
  //   my $p_name = $self->getNodeQName($presentation);
    // 1.Quick fix: merge two tokens
  //   if (($c_name eq 'ltx:XMTok') && ($p_name eq 'ltx:XMTok')) {
  //     $self->mergeAttributes($content, $presentation, $content_transfer_overrides);
  //     $self->mergeAttributes($dual,    $presentation, $dual_transfer_overrides);
  //     $self->replaceNode($dual, $presentation);

  //   # 2.For now, only main use case is compacting mirror XMApp nodes
  //   return if ($c_name ne 'ltx:XMApp') || ($p_name ne 'ltx:XMApp');
  //   my @content_args = element_nodes($content);
  //   my @pres_args    = element_nodes($presentation);
  //   return if scalar(@content_args) != scalar(@pres_args);

  //   my @new_args = ();
  //   # walk the corresponding children, and double-check they are referenced in the same order
  //   while ((my $c_arg = shift(@content_args)) and (my $p_arg = shift(@pres_args))) {
  //     my $c_idref = $c_arg->getAttribute('idref');
  //     if ($c_idref && ($c_idref eq ($p_arg->getAttribute('xml:id') || ''))) {
  //       push @new_args, $p_arg;
  //       next; }    # content-refs-pres, OK
  //     my $p_idref = $p_arg->getAttribute('idref');
  //     if ($p_idref && ($p_idref eq ($c_arg->getAttribute('xml:id') || ''))) {
  //       push @new_args, $c_arg;
  //       next; }    # pres-refs-content, OK

  //     # we can handle content-side XMToks, to any XM* presentation subtree differing for now.
  //     if ($self->getNodeQName($c_arg) ne 'ltx:XMTok') {
  //       return; }
  //     else { # otherwise we can compact this case. but delay actual libxml changes until we are *sure* the entire tree is compactable
  //       push(@new_args, [$c_arg, $p_arg]); } }

  // # If we made it here, this is a dual with two mirrored applications and a single XMTok difference, compact it.
  //   my $compact_apply = $self->openElementAt($dual->parentNode, 'ltx:XMApp');
  //   for my $n_arg (@new_args) {
  //     # one of the args has our dual node that needs compacting
  //     if (ref $n_arg eq 'ARRAY') {
  //       my ($c_arg, $p_arg) = @$n_arg;
  //       $self->mergeAttributes($c_arg, $p_arg, $content_transfer_overrides);
  //       $n_arg = $p_arg; }
  //     $n_arg->unbindNode;
  //     $compact_apply->appendChild($n_arg); }
  //   # if the dual has any attributes migrate them to the new XMApp
  //   $self->mergeAttributes($dual, $compact_apply, $dual_transfer_overrides);
  //   $self->replaceNode($dual, $compact_apply);
  //   $self->closeElementAt($compact_apply);
      Ok(())
    }

  /// Replace an XMDual with one of its branches
  fn collapse_xmdual(&mut self, dual: &mut Node, mut branch: Node, state:&mut State) -> Result<()> {
    // The other branch is not visible, nor referenced,
    // but the dual may have an id and be referenced
    if let Some(dualid) = dual.get_attribute_ns("id", XML_NS) {
      self.unrecord_id(&dualid); // We'll move or remove the ID from the dual
      if let Some(branchid) = branch.get_attribute_ns("id", XML_NS) { // branch has id too!
      for mut tref in self.findnodes(&s!("//*[@idref='{}']",dualid), None, state) {
        tref.set_attribute("idref", &branchid)?;
      } // Change dualid refs to branchid
      } else {
        branch.set_attribute("xml:id", &dualid)?; // Just use same ID on the branch
        self.record_id_with_node(&dualid, &branch, state);
      }
    }
    self.replace_tree(branch, dual, state)?;
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
        let font = &self.get_node_font(&n).merge(props.clone());
        self.set_node_font(&mut n, font)?;
        for child in n.get_child_nodes() {
          nodes.push_back(child);
        }
      }
    }
    Ok(())
  }

  pub fn set_box_font(&mut self, node: &mut Node, state: &mut State) -> Result<()> {
    if let Some(ref thisbox) = self.box_to_absorb {
      if let Some(font) = thisbox.get_font(state)? {
        let todo_font_clone = font.into_owned();
        self.set_node_font(node, &todo_font_clone)?;
      }
    }
    Ok(())
  }

  pub fn get_node_font(&self, node: &Node) -> &Font {
    if let Some(_element) = xml::closest_element(node) {
      if node.get_type() == Some(NodeType::ElementNode) {
        if let Some(fontid) = node.get_attribute("_font") {
          if let Some(fnt) = self.node_fonts.get(&fontid.parse::<u64>().unwrap()) {
            return fnt;
          }
        }
      }
    }
    &FONT_TEXT_DEFAULT
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
  pub fn remove_node(&mut self, node: &mut Node) {
    let mut chopped: bool = self.node == *node; // Note if we're removing insertion point
    if node.get_type() == Some(NodeType::ElementNode) {
      // If an element, do ID bookkeeping.
      if let Some(id) = node.get_attribute_ns("id", xml::XML_NS) {
        self.unrecord_id(&id);
      }
      for child in node.get_child_nodes() {
        chopped = chopped || self.remove_node_aux(child);
      }
    }
    let parent = node.get_parent().unwrap();
    if chopped {
      // Don't remove insertion point!
      self.set_node(&parent);
    }
    node.unlink(); // TODO: How is this different from parent.remove_child(node) ???
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
    state: &mut State,
  ) -> Result<Node> {
    // DG: This is a cursed way to manage fonts, how can we unwind it all back to a clear organization?
    if font_opt.is_none() {
      if let Some(ref attrs) = attributes {
        if let Some(fontid) = attrs.get("_font") {
          font_opt = self.node_fonts.get(&fontid.parse::<u64>().unwrap()).cloned()
        }
      }
    }
    let (decoded_ns, tag) = state.model.decode_qname(qname);
    let mut newnode;
    // box = self.node_boxes.get(box);    // may already be the string key
    // If this will be the document root node, things are slightly more involved.
    if point.get_type() == Some(NodeType::DocumentNode) {
      // First node! (?)
      Debug!("adding schema declaration, new node will be : {}", tag);
      state.model.add_schema_declaration(self);
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
        let prefix_opt = state.model.get_document_namespace_prefix(&ns, false, false);
        let attprefix_opt = state.model.get_document_namespace_prefix(&ns, true, true);
        if prefix_opt.is_none() {
          if let Some(ref attprefix) = attprefix_opt {
            let attr_ns_node = Namespace::new(attprefix, &ns, &mut newnode).unwrap();
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
      newnode = self.open_element_internal(point, decoded_ns, &tag, state)?;
    }

    if let Some(attrs) = attributes {
      let mut sorted_keys = attrs.keys().map(ToString::to_string).collect::<Vec<_>>();
      sorted_keys.sort();
      for key in &sorted_keys {
        if key == "font" || key == "locator" {
          continue;
        }
        self.set_attribute(&mut newnode, key, &attrs[key], state)?;
      }
    }
    if let Some(font) = font_opt {
      self.set_node_font(&mut newnode, &font)?;
    }

    // TODO [new]: Ever more certain there is a refactor waiting to happen with box_to_absorb
    //             holding a Rc<Digested> for easy cloning and management.
    //             Though the question remains how to maintain that, without cloning the box to **make** the Rc<>
    // Old note:
    // The .clone on boxes is potentially *VERY SLOW* and a code smell.
    // It can be eventually avoided by using a "memory arena" for all intermediate
    // objects - tokens, boxes, etc. and a well-designed referncing scheme into
    // the driver structs, such as Gullet, Stomach and Document
    if let Some(ref digested) = self.box_to_absorb {
      self.set_node_box(&newnode, digested.clone());
    }

    Debug!(
      "Inserting {:?} into {:?}",
      self.get_node_qname(&newnode, state),
      self.get_node_qname(point, state)
    );

    // Run afterOpen operations
    self.after_open(&mut newnode, state)?;

    Ok(newnode)
  }

  fn open_element_internal(&mut self, point: &mut Node, ns_opt: Option<String>, tag: &str, state: &mut State) -> Result<Node> {
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
            if let Some(prefix) = state.model.get_document_namespace_prefix(&ns_uri, false, false) {
              if !prefix.is_empty() {
                let mut root = self.document.get_root_element().unwrap();
                match Namespace::new(&prefix, &ns_uri, &mut root) {
                  Ok(ns) => Some(ns),
                  Err(_) => {
                    let message = s!("failed to create namespace: {:?}", prefix);
                    Error!("document", "open_element_internal", self, state, message);
                    None
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
                  let message = s!("failed to create namespace: {:?}", prefix);
                  Error!("document", "open_element_internal", self, state, message);
                  None
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
    let mut newnode = Node::new(tag, new_ns, &self.document).unwrap();
    point.add_child(&mut newnode)?;
    if no_ns {
      // without this explicit set call, an XPath for things such as "ltx:XMath"
      // fails ???
      if let Some(ns) = point.get_namespace() {
        newnode.set_namespace(&ns)?;
      }
    }

    self.record_constructed_node(&newnode);
    Ok(newnode)
  }

  /// Whenever a node has been created using openElementAt,
  /// closeElementAt ought to be used to close it, when you're finished inserting into $node.
  /// Basically, this just runs any afterClose operations.
  pub fn close_element_at(&mut self, node: &mut Node, state: &mut State) -> Result<()> { self.after_close(node, state) }

  pub fn after_open(&mut self, node: &mut Node, state: &mut State) -> Result<()> {
    // Set current point to this node, just in case the afterOpen's use it.
    let savenode = self.node.clone();
    self.set_node(node);
    let node_qname = self.get_node_qname(node, state);
    for action in self.get_tag_action_list(&node_qname, TagOptionName::AfterOpen, state) {
      action(self, node, state)?;
    }
    self.set_node(&savenode);
    Ok(())
  }

  pub fn after_close(&mut self, node: &mut Node, state: &mut State) -> Result<()> {
    // Should we set point to this node? (or to last child, or something ??
    let savenode = self.node.clone();
    let node_qname = self.get_node_qname(node, state);
    for action in self.get_tag_action_list(&node_qname, TagOptionName::AfterClose, state) {
      action(self, node, state)?;
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
  // (otherwise, libxml2 has a tendency to introduce annoying "default" namespace prefix declarations)
  // And, finally, we need to modify any id's present in the old nodes,
  // since otherwise they may be duplicated.

  // # Should have variants here for prepend, insert before, insert after.... ???
  pub fn append_clone(&mut self, node: &mut Node, new_children: Vec<Node>, state: &mut State) -> Result<()> {
    // Expand any document fragments
    let new_children = new_children.into_iter().flat_map(|child| if child.get_type() == Some(NodeType::DocumentFragNode) {  child.get_child_nodes() } else { vec![child] } ).collect::<Vec<Node>>();
    // Now find all xml:id's in the new_children and record replacement id's for them
    let mut id_map = HashMap::default();
    // Find all id's defined in the copy and change the id.
    for child in new_children.iter() {
      for id in self.findvalues(".//@xml:id", Some(child), state) {
        id_map.insert(id.to_string(), self.modify_id(id));
      }
    }
    // Now do the cloning (actually copying) and insertion.
    self.append_clone_aux(node, new_children, &mut id_map, state)
  }

  fn append_clone_aux(&mut self, node: &mut Node, new_children: Vec<Node>, id_map: &mut HashMap<String, String>, state: &mut State) -> Result<()> {
    for child in new_children.into_iter() {
      match child.get_type() {
      Some(NodeType::ElementNode) => {
        let mut new = self.open_element_internal(node, child.get_namespace().map(|ns| ns.get_href()), &child.get_name(), state)?;
        for (key,val) in child.get_attributes() {
          match key.as_str() {
            "xml:id" | "id" => { // Use the replacement id
              let mapped_id = id_map.get(&val).unwrap();
              let newid = self.record_id_with_node(mapped_id, &new, state);
              new.set_attribute(&key, &newid)?;
            }
            "idref" => { // Refer to the replacement id if it was replaced
              let id = id_map.get(&val).unwrap_or(&val);
              new.set_attribute(&key, id)?;
            }
            other_key => // TODO: Are namespaced attributes successfully handled here? Check.
              new.set_attribute(other_key, &val)?
         };
        }
        self.after_open(&mut new, state)?;
        self.append_clone_aux(&mut new, child.get_child_nodes(), id_map, state)?;
        self.after_close(&mut new, state)?;
      },
      Some(NodeType::TextNode) => node.append_text(&child.get_content())?,
      other => panic!("append_clone_aux called on {other:?} Node type.")
    };
  }
  Ok(()) }

  //**********************************************************************
  // Wrapping & Unwrapping nodes by another element.

  // Wrap `nodes` with an element named `qname`, making the new element replace the first `node`,
  // and all `nodes` becomes the child of the new node.
  // [this makes most sense if `nodes` are a sequence of siblings]
  // Returns undef if $qname isn't allowed in the parent, or if `nodes` aren't allowed in `qname`,
  // otherwise, returns the newly created `qname`.
  pub fn wrap_nodes(&mut self, qname: &str, mut nodes: Vec<Node>, state: &mut State) -> Result<Option<Node>> {
    if nodes.is_empty() {
      return Ok(None);
    }
    let first_node = nodes.remove(0);
    let mut parent = first_node.get_parent().unwrap();
    let (ns, tag) = state.model.decode_qname(qname);
    let mut new = self.open_element_internal(&mut parent, ns, &tag, state)?;
    self.after_open(&mut new, state)?;
    let mut old_node = parent.replace_child_node(new.clone(), first_node)?;

    self.copy_node_font(&parent, &mut new)?;

    if let Some(tbox) = self.get_node_box(&parent) {
      self.set_node_box(&new, tbox);
    }
    new.add_child(&mut old_node)?;
    for mut node in nodes.into_iter() {
      new.add_child(&mut node)?;
    }
    self.after_close(&mut new, state)?;
    Ok(Some(new))
  }

  /// Unwrap the children of $node, by replacing $node by its children.
  pub fn unwrap_nodes(&mut self, node: Node) -> Result<Node> {
    let children = node.get_child_nodes();
    self.replace_node(node, children)
  }

  // Replace $node by `nodes` (presumably descendants of some kind?)
  pub fn replace_node(&mut self, mut node: Node, with: Vec<Node>) -> Result<Node> {
    if let Some(_parent) = node.get_parent() {
      let mut c0_opt: Option<Node> = None;
      for mut with_node in with.into_iter() {
        with_node.unlink();
        if let Some(mut c0) = c0_opt {
          c0.add_next_sibling(&mut with_node)?;
        } else {
          // first node, swap in
          node.add_next_sibling(&mut with_node)?;
        }
        c0_opt = Some(with_node);
      }
      self.remove_node(&mut node);
    }
    Ok(node)
  }

  // initially since $node->setNodeName was broken in XML::LibXML 1.58
  // but this can provide for more options & correctness?
  pub fn rename_node(&mut self, _node: &mut Node, _newname: &str) -> Result<Node> {
    unimplemented!();
    // my ($self, $node, $newname) = @_;
    // my $model = $$self{model};
    // my ($ns, $tag) = $model->decodeQName($newname);
    // my $parent = $node->parentNode;
    // my $new = $self->openElement_internal($parent, $ns, $tag);
    // my $id;
    // # Move to the position AFTER $node
    // $parent->insertAfter($new, $node);
    // # Copy ALL attributes from $node to $newnode
    // foreach my $attr ($node->attributes) {
    //   my $key   = $attr->getName;
    //   my $value = $node->getAttribute($key);
    //   $id = $value if $key eq 'xml:id';    # Save to register after removal of old node.
    //   $new->setAttribute($key, $value); }
    // # AND move all content from $node to $newnode
    // foreach my $child ($node->childNodes) {
    //   $new->appendChild($child); }
    // ## THEN call afterOpen... ?
    // # It would normally be called before children added,
    // # but how can we know if we're duplicated auto-added stuff?
    // $self->afterOpen($new);
    // $self->afterClose($new);
    // # Finally, remove the old node
    // $self->removeNode($node);
    // # and FINALLY, we can register the new node under the id.
    // if ($id) {
    //   my $newid = $self->recordID($id, $new);
    //   $new->setAttribute('xml:id' => $newid) if $newid ne $id; }
    // return $new; }
  }

  pub fn trim_node_whitespace(&mut self, node: &mut Node) -> Result<()> {
    trim_node_left_whitespace(node)?;
    trim_node_right_whitespace(node)?;
    Ok(())
  }

  pub fn add_resource(&mut self, resource: Resource, state: &mut State) -> Result<()> {
    // let savenode_opt = self.float_to_element("ltx:resource", false);
    let savenode_opt = None;
    let mut attrib: HashMap<String, String> = HashMap::default();
    attrib.insert(s!("src"), resource.name);
    attrib.insert(s!("type"), resource.mimetype);
    attrib.insert(s!("media"), resource.media);
    let content_box = Digested::from(Tbox {
      text: resource.content,
      ..Tbox::default()
    });
    self.insert_element("ltx:resource", vec![&content_box], Some(attrib), state)?;
    if let Some(savenode) = savenode_opt {
      self.set_node(&savenode);
    }
    Ok(())
  }

  pub fn process_pending_resources(&mut self, state: &mut State) -> Result<()> {
    let resources: Vec<Resource> = state.pending_resources.drain(..).collect();
    for resource in resources {
      self.add_resource(resource, state)?;
    }
    state.pending_resources = Vec::new();
    Ok(())
  }

  pub fn make_error(&mut self, error_class: &str, _content: &str, state: &mut State) -> Result<()> {
    let savenode_opt = if !self.is_openable("ltx:ERROR", state) {
      self.float_to_element("ltx:ERROR", false)
    } else {
      None
    };
    self.open_element("ltx:ERROR", Some(string_map!("class"=>error_class)), None, state)?;
    self.close_element("ltx:ERROR", state)?;
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
  pub fn float_to_element(&mut self, _qname: &str, _closeifpossible: bool) -> Option<Node> {
    // TODO:

    // let candidates = get_insertion_candidates(self.node);
    // let mut closeable  = true;
    // // If the current node can contain already, we're fine right here - just return
    // if !candidates.is_empty() && self.can_contain(candidates[0], qname) {
    //   // Edge case: Don't resume at a text node, if it is current. Don't append more to it after other insertions.
    //   if self.node.get_type() == NodeType::TextNode {
    //     self.set_node(candidates[0]);
    //   }
    //   return Some(candidates[0]);
    // }
    // while !candidates.is_empty() && !self.can_contain(candidates[0], qname) {
    //   if closeable {
    //     closeable = self.can_auto_close(candidates[0]);
    //   }
    //   candidates.pop_front();
    // }
    // if let Some(n) = candidates.pop_front() {
    //   if closeifpossible && closeable {
    //     self.close_to_node(n);
    //   } else {
    //     let savenode = self.node;
    //     self.set_node(n);
    //     // Debug!("Floating from " . Stringify($savenode) . " to " . Stringify($n) . " for $qname")
    //     //   if ($$savenode ne $$n) && $LaTeXML::DEBUG{document};
    //     Some(savenode)
    //   }
    // } else {
    //   if !self.can_contain_somehow(self.node, qname) {
    //     Warn!("malformed", qname, self, "No open node can contain element '{}'", qname
    //       $self->getInsertionContext())
    //     }
    //   None
    // }
    None
  }

  // find a node that can accept a label.
  // A bit more than just whether the element can have the attribute, but
  // whether it has an id (and ideally either a refnum or title)
  pub fn float_to_label(&mut self, state: &mut State) -> Option<Node> {
    let key = "labels";
    let ancestors: Vec<Node> = self
      .get_insertion_candidates(&self.node)
      .into_iter()
      .filter(|node| node.get_type() == Some(NodeType::ElementNode))
      .collect();
    let mut candidates: VecDeque<&Node> = ancestors.iter().collect();
    // Should we only accept a node that already has an id, or should we create an id?
    let mut node_opt: Option<Cow<Node>> = None;
    while let Some(candidate) = candidates.pop_front() {
      if self.can_node_have_attribute(candidate, key, state) && candidate.has_attribute_ns("id", xml::XML_NS) {
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
        if self.can_node_have_attribute(&sibling, key, state) && sibling.has_attribute_ns("id", xml::XML_NS) {
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
      Warn!("malformed", key, self, state, message);
      //  $self->getInsertionContext());
      None
    }
  }

  pub fn set_box_to_absorb(&mut self, arg: Option<Digested>) {
    self.localized_boxes.push(self.box_to_absorb.take());
    self.box_to_absorb = arg;
  }
  pub fn expire_box_to_absorb(&mut self) { self.box_to_absorb = self.localized_boxes.pop().unwrap(); }

  pub fn load_labels_for_rewrite(&mut self, state: &mut State) {
    for node in self.findnodes("//*[@labels]", None, state) {
      if let Some(labels) = node.get_attribute("labels") {
        if let Some(id) = node.get_attribute("id") {
          for label in labels.split_whitespace() {
            self.rewrite_labels.insert(label.to_string(), id.to_string());
          }
        } else {
          Error!(
            "malformed",
            "label",
            None,
            None,
            format!("Node {} has labels but no xml:id", node.get_name())
          );
        }
      }
    }
  }

  fn set_local_font(&mut self, arg: Arc<Font>) {
    self.localized_fonts.push(arg);
  }
  fn get_local_font(&self) -> Option<Arc<Font>> {
    self.localized_fonts.last().cloned()
  }
  fn expire_local_font(&mut self) {
    self.localized_fonts.pop();
  }

  //**********************************************************************
  /// This function computes an xml:id for a node, if it hasn't already got one.
  /// It is suitable for use in Tag afterOpen as
  ///  `Tag('ltx:para',afterOpen=>sub { GenerateID(@_,'p'); });`
  /// It generates an id of the form <parentid>.<prefix><number>
  /// The parent node (the one with ID=<parentid>) also maintains a counter
  /// stored in an attribute `_ID_counter_<prefix>` recording the last used
  /// <number> for <prefix> amongst its descendents.
  pub fn generate_id(&mut self, node: &mut Node, mut prefix: &str, state: &mut State) -> Result<()> {
    // If node doesn't already have an id, and can
    let node_qname = self.get_node_qname(node, state);
    // but isn't a _Capture_ node (which ultimately should disappear)
    if !node.has_attribute_ns("id", XML_NS) && self.can_have_attribute(&node_qname, "xml:id", state) && (node_qname != "ltx:_Capture_") {
      let mut ancestor = self
        .findnode("ancestor::*[@xml:id][1]", Some(node), state)
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

      let ctrkey = s!("_ID_counter_") + prefix + "_";
      let a_ctr = ancestor.get_attribute(&ctrkey).unwrap_or_else(|| s!("0"));

      let ctr_int = 1 + a_ctr.parse::<u32>().unwrap_or(0);
      let ctr = ctr_int.to_string();

      let id = match ancestor_id {
        Some(aid) => aid + ".",
        None => String::new(),
      } + prefix
        + &ctr;

      ancestor.set_attribute(&ctrkey, &ctr)?;
      self.set_attribute(node, "xml:id", &id, state)?;
    }
    Ok(())
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Finally, another set of surgery methods
  // These take an array representation of the XML Tree to append
  //   [tagname,{attributes..}, children]
  // THESE SHOULD BE PART OF A COMMON BASE CLASS; DUPLICATED IN Post::Document
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  pub fn replace_tree(&mut self, new: Node, old: &mut Node, state: &mut State) -> Result<Option<Node>> {
    if let Some(mut parent) = old.get_parent() {
      let mut following = VecDeque::new(); // Collect the matching and following nodes
      while let Some(mut sib) = parent.get_last_child() {
        if sib == *old {
          break;
        }
        // parent.remove_child(sib); // We're putting these back, in a moment!
        sib.unlink();
        following.push_front(sib);
      }
      self.remove_node(old);
      self.append_tree(&mut parent, vec![new], state)?;
      let inserted = parent.get_last_child();
      for mut child in following {
        parent.add_child(&mut child)?; // No need for clone
      }
      Ok(inserted)
    } else {
      Ok(None)
    }
  }

  pub fn append_tree(&mut self, node: &mut Node, data: Vec<Node>, state: &mut State) -> Result<()> {
    for child in data {
      match child.get_type() {
        Some(NodeType::ElementNode) => {
          let tag = self.get_node_qname(&child, state);
          let attributes = child.get_attributes().into_iter().collect(); // map { $_->nodeType == XML_ATTRIBUTE_NODE ? ($self->getNodeQName($_) => $_->getValue) : () }
                                                   // TODO:
                                                   // DANGER: REMOVE the xml:id attribute from $child!!!!
                                                   // This protects against some versions of XML::LibXML that warn against duplicate id's
                                                   // Hopefully, you shouldn't be using the node any more
                                                   //         if (my $id = $attributes{'xml:id'}) {
                                                   //           $child->removeAttribute('xml:id');
                                                   //           $self->unRecordID($id); }

          let mut new = self.open_element_at( node, &tag, Some(attributes), None, state)?;
          self.append_tree(&mut new, child.get_child_nodes(), state)?;
          self.close_element_at(&mut new, state)?;
        },
        Some(NodeType::DocumentFragNode) => {
          self.append_tree(node, child.get_child_nodes(), state)?;
        },
        Some(NodeType::TextNode) => {
          node.append_text(&child.get_content())?;
        },
        other => {
          dbg!(other);
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
  // Remove dis-allowed code-points.
  // $string =~
  // s/(?:\x{00}-\x{08}|\x{0B}|\x{0C}|\x{0D}-\x{19}|\x{D800}-\x{DFFF}|\x{FFFE}-\x{FFFF})//g;
  //  Hmm... the upper ranges gives warning in some Perls...
  // TODO:
  // $string =~ s/(?:\x{00}-\x{08}|\x{0B}|\x{0C}|\x{0D}-\x{19})//g;
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

fn trim_node_left_whitespace(node: &mut Node) -> Result<()> {
  if let Some(mut first_child) = node.get_first_child() {
    match first_child.get_type() {
      Some(NodeType::TextNode) => {
        let content = first_child.get_content();
        let trimmed_content = content.trim_start();
        if !content.is_empty() && (trimmed_content != content) {
          first_child.set_content(trimmed_content)?;
        }
      },
      Some(NodeType::ElementNode) => trim_node_left_whitespace(&mut first_child)?,
      _ => {},
    };
  }
  Ok(())
}

fn trim_node_right_whitespace(node: &mut Node) -> Result<()> {
  if let Some(mut last_child) = node.get_last_child() {
    match last_child.get_type() {
      Some(NodeType::TextNode) => {
        let content = last_child.get_content();
        let trimmed_content = content.trim_end();
        if !content.is_empty() && (trimmed_content != content) {
          last_child.set_content(trimmed_content)?;
        }
      },
      Some(NodeType::ElementNode) => trim_node_right_whitespace(&mut last_child)?,
      _ => {},
    };
  }
  Ok(())
}

pub trait IntoVDQS {
  fn into_vdqs(self) -> VecDeque<String>
  where Self: Sized;
}
impl IntoVDQS for String {
  fn into_vdqs(self) -> VecDeque<String> {
    let mut vdq = VecDeque::new();
    vdq.push_front(self);
    vdq
  }
}
impl IntoVDQS for &str {
  fn into_vdqs(self) -> VecDeque<String> { self.to_string().into_vdqs() }
}
impl IntoVDQS for VecDeque<String> {
  fn into_vdqs(self) -> VecDeque<String> { self }
}

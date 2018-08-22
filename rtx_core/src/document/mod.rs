pub mod resource;
pub mod tag;

extern crate libxml;
extern crate regex;

use libxml::tree::set_node_rc_guard;
use libxml::tree::Document as XmlDoc;
use libxml::tree::{Namespace, Node, NodeType};
use regex::Regex;
use std::collections::{HashMap, VecDeque};
use std::iter;

use common::error::*;
use common::font::Font;
use common::store::Stored;
use state::State;

use document::resource::Resource;
use document::tag::{TagConstructionClosure, TagOptionName, TagOptions};
use Tbox;
use {BoxOps, Digested};

lazy_static! {
  static ref HAS_NONSPACE_RE: Regex = Regex::new(r"\S").unwrap();
  static ref ONLY_SPACE_RE: Regex = Regex::new(r"^\s+$").unwrap();
}

static FONT_ELEMENT_NAME: &'static str = "ltx:text";
static MATH_TOKEN_NAME: &'static str = "ltx:XMTok";

pub struct Document {
  pub document: XmlDoc,
  pub pending: Vec<Node>,
  pub node: Node,
  pub node_boxes: HashMap<usize, Digested>, // used to be _box attribute
  pub node_fonts: HashMap<usize, Font>,     // used to be _font attribute
  pub debug: bool,
  pub constructed_nodes: Vec<Node>,
  pub box_to_absorb: Option<Digested>,
}
impl Default for Document {
  fn default() -> Self { Self::new() }
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
      node_boxes: HashMap::new(),
      node_fonts: HashMap::new(),
      pending: Vec::new(),
      debug: true,
      constructed_nodes: Vec::new(),
      box_to_absorb: None,
    }
  }

  /// Get the element at (or containing) the current insertion point.
  pub fn get_element(&self) -> Option<Node> {
    match self.node.get_type() {
      Some(NodeType::TextNode) => self.node.get_parent(),
      Some(NodeType::DocumentNode) => None,
      _ => Some(self.node.clone()),
    }
  }

  /// Find the nodes according to the given $xpath expression,
  /// the xpath is relative to $node (if given), otherwise to the document node.
  pub fn findnodes(&self, xpath: &str, node_opt: Option<&Node>, state: &mut State) -> Vec<Node> {
    match node_opt {
      Some(node) => state
        .model
        .get_xpath(&self.document)
        .findnodes(xpath, Some(node)),
      None => state
        .model
        .get_xpath(&self.document)
        .findnodes(xpath, Some(&self.document.get_root_element().unwrap())),
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

  /// Get the node's qualified name in standard form
  /// Ie. using the registered prefix for that namespace.
  /// NOTE: Reconsider how _Capture_ & _WildCard_ should be integrated!?!
  /// NOTE: Should Deprecate! (use model)
  pub fn get_node_qname(&self, node: &Node, state: &mut State) -> String {
    state.model.get_node_qname(node)
  }

  pub fn get_node(&self) -> &Node { &self.node }
  pub fn get_document(&self) -> &XmlDoc { &self.document }

  // **********************************************************************
  // This should be called before returning the final XML::LibXML::Document to the
  // outside world.  It resolves the fonts for each node relative to it's
  // ancestors. It removes the `helper' attributes that store fonts, source
  // box, etc.
  pub fn finalize(&mut self, state: &mut State) -> Result<()> {
    self.prune_xmduals();
    let mut root = self.document.get_root_element().unwrap();
    let init_font = Font::text_default();
    self.finalize_rec(&mut root, &init_font, state)?;
    if let Some(&Stored::String(ref prefixes)) = state.lookup_value("RDFa_prefixes") {
      self.set_rdfa_prefixes(Some(prefixes.clone()));
    }
    Ok(())
  }

  fn finalize_rec(&mut self, node: &mut Node, init_font: &Font, state: &mut State) -> Result<()> {
    let qname = state.model.get_node_qname(node);
    let mut declared_font = init_font.clone();
    let desired_font;
    let mut pending_declaration = HashMap::new();
    if let Some(_comment) = node.get_attribute("_pre_comment") {
      if let Some(_parent) = node.get_parent() {
        // parent.insert_before(XML::LibXML::Comment.new(comment), node);
      }
    }
    if let Some(_comment) = node.get_attribute("_comment") {
      if let Some(_parent) = node.get_parent() {
        // parent.insert_after(XML::LibXML::Comment.new(comment), node);
      }
    }

    let mut keys_to_remove: Vec<String> = Vec::new();
    let mut attrs_to_set: Vec<(String, String)> = Vec::new();

    {
      if let Some(font) = self.get_node_font(node) {
        desired_font = font.clone();
        pending_declaration = desired_font.relative_to(&declared_font);
        if (!node.get_child_nodes().is_empty() || node.get_attribute("_force_font").is_some())
          && !pending_declaration.is_empty()
        {
          for (ref key, &(ref value, ref properties)) in &pending_declaration {
            if state.model.can_have_attribute(&qname, key) {
              attrs_to_set.push((key.to_string(), value.to_string()));
              // Merge to set the font currently in effect
              declared_font = declared_font.merge(properties.clone());
              keys_to_remove.push(key.to_string());
            }
          }
        }
      }
    }
    for (key, value) in attrs_to_set {
      self.set_attribute(node, &key, &value)?;
    }
    for key in keys_to_remove {
      pending_declaration.remove(&key);
    }

    let new_init_font = &declared_font;
    for mut child in node.get_child_nodes() {
      let child_type = child.get_type();
      if child_type == Some(NodeType::ElementNode) {
        let was_forcefont = child.get_attribute("_force_font").is_some();
        self.finalize_rec(&mut child, new_init_font, state)?;
        // Also check if child is  FONT_ELEMENT_NAME  AND has no attributes
        // AND providing node can contain that child's content, we'll collapse it.
        if (state.model.get_node_qname(&child) == FONT_ELEMENT_NAME)
          && !was_forcefont
          && child.get_attributes().is_empty()
        {
          let grandchildren = child.get_child_nodes();
          if grandchildren
            .iter()
            .filter(|gchild| !self.can_contain(node, &gchild.get_name(), state))
            .count()
            == 0
          {
            error!(target: "TODO", "replace_node");
            // self.replace_node(child, grandchildren);
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
          error!(target: "TODO", "too late to wrapNodes? {:?}", pending_declaration);
          //   my text = self.wrapNodes(FONT_ELEMENT_NAME, child);
          //   foreach my attr (keys %pending_declaration) {
          //     self.setAttribute(text, attr => pending_declaration{attr}{value}); }
          //   self.finalize_rec(text, state);    // Now have to clean up the new node!
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
  /// A $box that is a Box, or List, or Whatsit, is responsible for carrying out
  /// its own insertion, but it should ultimately call methods of Document
  /// that will record the nodes that were created.
  /// $box can also be a plain string which will be inserted according to whatever
  /// font, mode, etc, are in %props.
  pub fn absorb(&mut self, object: Digested, state: &mut State) -> Result<()> {
    // let mut results = Vec::new();
    let mut boxes = VecDeque::new();
    boxes.push_front(object);

    while let Some(front_box) = boxes.pop_front() {
      self.constructed_nodes = Vec::new();
      self.box_to_absorb = Some(front_box.clone());
      // info!(target: "document:absorb", "front box: {:?}", front_box);
      match front_box {
        // Simply unwind Lists to avoid unneccessary recursion; This occurs quite frequently!
        Digested::List(list) => for tbox in list.unlist().into_iter().rev() {
          boxes.push_front(tbox);
        },
        // A Proper Box or Whatsit? Absorb it.
        Digested::TBox(digested) => digested.be_absorbed(self, state)?,
        Digested::Whatsit(digested) => digested.be_absorbed(self, state)?,
        Digested::Postponed(ref _t) => unimplemented!(),
      };

      // TODO: Does the results extension make ANY sense???
      // These were created just now
      // let newly_created: Vec<Node> = self.constructed_nodes.drain(0..).collect();

      //for node in &newly_created {
      //   self.record_constructed_node(Some(&node)); // record these for OUTER caller!
      // }
      // results.extend(newly_created); // but return only the most recent set.

      // Else, plain string in text mode.
      // elsif (!$props{isMath}) {
      // push(@results, self.openText($box, $props{font} || ($LaTeXML::BOX &&
      // $LaTeXML::BOX->getFont))); } // Or plain string in math mode.
      // // Note text nodes can ONLY appear in <XMTok> or <text>!!!
      // // Have we already opened an XMTok? Then insert into it.
      // elsif (self.model}->getNodeQName(self.node}) eq $MATH_TOKEN_NAME) {
      //   push(@results, self.openMathText_internal($box)); }
      // // Else create the XMTok now.
      // else {
      // // Odd case: constructors that work in math & text can insert raw strings
      // in Math mode. push(@results, self.insertMathToken($box, font =>
      // $props{fontzz})); } }

      //   let mut box_node = self.node.add_child(None, "box").unwrap();
      //   box_node.set_content(&tbox.text);
    }
    // if self.debug {
    //   debug!("Document absorbed {:?} nodes", results.len());
    // }
    // Results never used, BUT leak Rc<Node> strong counts!!!
    // Ok(results)
    Ok(())
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  /// Shorthand for open,absorb,close, but returns the new node.
  pub fn insert_element(
    &mut self,
    qname: &str,
    content: Vec<Digested>,
    attrib: Option<HashMap<String, String>>,
    state: &mut State,
  ) -> Result<Node>
  {
    // TODO: Quickly hacked together, needs a careful refactor with all .clone()
    // calls removed
    let node = self.open_element(qname, attrib, None, state)?;
    if self.debug {
      debug!("Inserting element {:?} with body: {:?}", qname, content);
    }
    for digested in content {
      self.absorb(digested, state)?;
    }

    let mut needs_close = self.node == node;
    {
      // In obscure situations, `node` may have already gotten closed?
      // close it if it is still open.
      let self_node = self.node.get_parent().unwrap();
      let mut c = Some(self_node);

      while c.is_some()
        && c.as_ref().unwrap().get_type() != Some(NodeType::DocumentNode)
        && c.as_ref().unwrap() != &node
      {
        let parent = c.unwrap().get_parent().unwrap();
        if parent.get_type() != Some(NodeType::DocumentNode) {
          c = Some(parent);
        } else {
          c = None;
        }
      }
      if let Some(ref c_node) = c {
        if c_node == &node {
          needs_close = true;
        }
      }
    }
    if needs_close {
      self.close_element(qname, state)?;
    }
    Ok(node)
  }

  /// Insert a ProcessingInstruction of the form <?op attr=value ...?>
  /// Does NOT move the current insertion point to the PI,
  /// but may move up past a text node.
  // Rust note: attrib would have been best as Vec<(String,String)> but
  // currently quote!() doesn't work out of the box on them
  pub fn insert_pi(&mut self, op: &str, attributes: Option<HashMap<String, String>>) -> Result<()> {
    let mut attr_data = String::new();
    for (key, value) in &attributes.unwrap_or_default() {
      attr_data.push_str(key);
      attr_data.push_str("=\"");
      attr_data.push_str(value);
      attr_data.push('"');
    }
    // self.close_text_internal();  // Close any open text node
    let mut pi_node = self
      .document
      .create_processing_instruction(op, &attr_data)
      .unwrap();
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
  ) -> Result<Node>
  {
    // NoteProgress('.') if (self.progress}++ % 25) == 0;
    if self.debug {
      debug!("Open element {:?} at {:?}", qname, self.node.get_name());
    }
    let point = self.find_insertion_point(qname, state)?;
    let newnode = self.open_element_at(point, qname, attributes, font_opt, state)?;
    self.set_node(&newnode);
    // Underscore attributes such as _box and _font from LaTeXML-proper are now
    // bookkept in special substructs of Document Connected to the node hash.
    // Ideally should be as quick to recompute natively as it would be to set/get
    // attributes externally via libxml.
    //
    // TODO: also accept a _box argument eventually? Or store differently?
    // attributes.entry("_box").or_insert(state.locals.box);
    if let Some(box_font) = font_opt {
      self.set_node_font(&newnode, &box_font);
    } else {
      let mut box_opt = None;
      if let Some(ref tbox) = self.box_to_absorb {
        if let Some(box_font) = tbox.get_font() {
          box_opt = Some(box_font.clone());
        }
      }
      if let Some(box_font) = box_opt {
        self.set_node_font(&newnode, &box_font);
      }
    }

    Ok(newnode)
  }

  /// Note: This closes the deepest open node of a given type.
  /// This can cause problems with auto-opened nodes, esp. ones for fontswitches!
  /// Since this is an "explicit request", we're currently skipping over those nodes,
  /// ie. we're automatically closing them, even if they're the same type as we're asking to
  /// close!!! This is kinda risky! Maybe we should try to request closing of specific nodes.
  pub fn close_element(&mut self, qname: &str, state: &mut State) -> Result<Option<Node>> {
    if self.debug {
      debug!("Close element {:?} at {:?}", qname, self.node.get_name());
    }
    self.close_text_internal(state)?;
    let mut node = self.node.clone();
    let mut cant_close = Vec::new();
    while node.get_type() != Some(NodeType::DocumentNode) {
      let t = state.model.get_node_qname(&node);
      // autoclose until node of same name BUT also close nodes opened' for font
      // switches!
      if t == qname && !(t == FONT_ELEMENT_NAME && node.get_attribute("_fontswitch").is_some()) {
        break;
      }
      if !self.can_auto_close(&node) {
        cant_close.push(node.clone());
      }
      match node.get_parent() {
        Some(p) => node = p,
        None => break,
      };
    }

    if node.get_type() == Some(NodeType::DocumentNode) {
      // Didn't find $qname at all!!
      let qname_msg: String = match qname {
        "#PCDATA" => qname.to_owned(),
        _ => s!("</{}>", qname),
      };
      error!(
        target: &s!("malformed:{:?}", qname),
        "Attempt to close {}, which isn't open. Currently in {}",
        qname_msg,
        self.get_insertion_context(None, state)
      );
      Ok(None)
    } else {
      // Found node.
      if !cant_close.is_empty() {
        // Intervening non-auto-closeable nodes!!
        error!(
          target: &s!("malformed:{:?}", qname),
          "Closing tag whose open descendents do not auto-close"
        );
        // Error('malformed', $qname, $self,
        //   "Closing " . ($qname eq '#PCDATA' ? $qname : '</' . $qname . '>')
        //     . " whose open descendents do not auto-close",
        //   "Descendents are " . join(', ', map { Stringify($_) } @cant_close))
        //   if @cant_close;
      }
      // So, now close up to the desired node.
      self.close_node_internal(&node, state)?;
      Ok(Some(node))
    }
  }

  // Check whether it is possible to open $qname at this point,
  // possibly by autoOpen'ing & autoClosing other tags.
  pub fn is_openable(&mut self, qname: &str, state: &mut State) -> bool {
    let mut node_opt = Some(self.node.clone());
    while let Some(node) = node_opt {
      let node_qname = self.get_node_qname(&node, state);
      if self.can_contain_somehow(&node_qname, qname, state) {
        return true;
      } else if !self.can_auto_close(&node) {
        return false; // could close, then check if parent can contain
      } else {
        node_opt = node.get_parent();
      }
    }
    false
  }

  // Check whether it is possible to close each element in @tags,
  // any intervening nodes must be autocloseable.
  // returning the last node that would be closed if it is possible,
  // otherwise undef.
  pub fn is_closeable(&mut self, mut tags: VecDeque<String>, state: &mut State) -> Option<Node> {
    let mut node_opt = if self.node.get_type() == Some(NodeType::TextNode) {
      self.node.get_parent()
    } else {
      Some(self.node.clone())
    };
    while let Some(qname) = tags.pop_front() {
      loop {
        if node_opt.is_none() {
          break;
        }
        let node = node_opt.as_ref().unwrap().clone();

        if node.get_type() == Some(NodeType::DocumentNode) || node.get_type() == None {
          return None;
        }
        let this_qname = state.model.get_node_qname(&node);
        if this_qname == qname {
          break;
        }
        if !self.can_auto_close(&node) {
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
    let mut qname_vdq = VecDeque::new();
    qname_vdq.push_front(qname.to_string());
    if let Some(node) = self.is_closeable(qname_vdq, state) {
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
      if !self.can_auto_close(&n) {
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
      error!(
        target: &s!("malformed:{}", state.model.get_node_qname(node)),
        "Attempt to close {:?}, which isn't open",
        node.get_name()
      );
    //     "Currently in " . $self->getInsertionContext()) unless $ifopen;
    } else {
      // Found node.
      if !cant_close.is_empty() {
        // But found has intervening non-auto-closeable nodes!!
        error!(
          target: &s!("malformed:{}", state.model.get_node_qname(node)), //$self,
          "Closing {:?} whose open descendents do not auto-close",
          node.get_name()
        ); //,
           //     "Descendents are " . join(', ', map { Stringify($_) } @cant_close))
      }
      if let Some(lastopen_node) = lastopen {
        self.close_node_internal(&lastopen_node, state)?;
      }
    }
    Ok(())
  }

  // Closes all nodes until $node is closed.
  pub fn close_node(&mut self, _node: &Node, _state: &mut State) {
    // my $model = $$self{model};
    // my ($t, @cant_close) = ();
    // my $n = $$self{node};
    // while ((($t = $n->getType) != XML_DOCUMENT_NODE) && !$n->isSameNode($node)) {
    //   push(@cant_close, $n) unless $self->canAutoClose($n);
    //   $n = $n->parentNode; }
    // if ($t == XML_DOCUMENT_NODE) {    # Didn't find $qname at all!!
    //   Error('malformed', $model->getNodeQName($node), $self,
    //     "Attempt to close " . Stringify($node) . ", which isn't open",
    //     "Currently in " . $self->getInsertionContext()); }
    // else {                            # Found node.
    //                                   # Intervening non-auto-closeable nodes!!
    //   Error('malformed', $model->getNodeQName($node), $self,
    // "Closing " . Stringify($node) . " whose open descendents do not
    // auto-close", "Descendents are " . join(', ', map { Stringify($_) }
    // @cant_close))     if @cant_close;
    //   $self->closeNode_internal($node); }
    return;
  }

  pub fn can_auto_close(&self, _node: &Node) -> bool { true }

  /// get the actions that should be performed on afterOpen or afterClose
  pub fn get_tag_action_list(
    &self,
    tag: &str,
    when: TagOptionName,
    state: &mut State,
  ) -> Vec<TagConstructionClosure>
  {
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

    let tag_hash = state
      .tag_properties
      .entry(tag.to_string())
      .or_insert_with(TagOptions::default)
      .clone();
    // let ns_hash  = ((defined $p) && $STATE->lookupMapping('TAG_PROPERTIES', $p .
    // ':*')) || {};
    let all_hash = state
      .tag_properties
      .entry(s!("ltx:*"))
      .or_insert_with(TagOptions::default)
      .clone();

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

  pub fn to_string(&self, state: &mut State) -> String {
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
  pub fn serialize_aux(
    &self,
    node: &Node,
    depth: usize,
    noindent: bool,
    heuristic: bool,
    state: &mut State,
  ) -> String
  {
    let indent = iter::repeat("  ").take(depth).collect::<String>();
    let mut serialized = String::new();

    match node.get_type() {
      Some(NodeType::DocumentNode) => {
        serialized.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        if let Some(child) = node.get_first_child() {
          let child_serialized = self.serialize_aux(&child, depth, noindent, heuristic, state);
          serialized.push_str(&child_serialized);
          let mut current_child = child;
          while let Some(sibling) = current_child.get_next_sibling() {
            let sibling_serialized =
              self.serialize_aux(&sibling, depth, noindent, heuristic, state);
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
        let mut open_tag = s!("<{}", tag);

        let nsnodes = node.get_namespace_declarations();
        for ns in nsnodes {
          let prefix = ns.get_prefix();
          let prefix_declaration = if prefix.is_empty() {
            s!("xmlns")
          } else {
            s!("xmlns:{}", prefix)
          };
          let href = ns.get_href();
          open_tag.push_str(&s!(" {}=\"{}\"", prefix_declaration, href));
        }

        let anodes = node.get_attributes();
        let mut anodes_keys: Vec<&String> = anodes.keys().collect();
        anodes_keys.sort();
        for key in anodes_keys {
          if key == "id" {
            continue;
          } // HACK for xml:id
          let key_serialized = state
            .model
            .get_node_document_qname(&node.get_attribute_node(key).unwrap());
          let val_serialized = serialize_attr(&node.get_property(key).unwrap_or_default());
          open_tag.push_str(&s!(" {}=\"{}\"", key_serialized, val_serialized));
        }
        // HACK for xml:id for now, assuming last element
        if anodes.get("id").is_some() {
          let val_serialized = serialize_attr(&node.get_property("id").unwrap_or_default());
          open_tag.push_str(&s!(" {}=\"{}\"", "xml:id", val_serialized));
        }

        let noindent_children: bool = if heuristic {
          // This emulates libxml2"s heuristic
          noindent || !children
            .iter()
            .filter(|e| e.get_type() == Some(NodeType::TextNode))
            .collect::<Vec<&Node>>()
            .is_empty()
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
            serialized.push_str("\n");
          }
          for child in children {
            serialized.push_str(&self.serialize_aux(
              &child,
              depth + 1,
              noindent_children,
              heuristic,
              state,
            ));
          }
          if !noindent_children {
            serialized.push_str(&indent)
          }
          serialized.push_str(&s!("</{}>", tag));
          if !noindent {
            serialized.push_str("\n");
          }
        } else {
          // empty element.
          serialized.push_str("/>");
          if !noindent {
            serialized.push_str("\n");
          }
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
          serialized.push_str("\n");
        }
      },
      Some(NodeType::CommentNode) => {
        serialized.push_str(&s!("<!-- {}-->", serialize_string(&node.get_content())));
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
  fn set_rdfa_prefixes(&mut self, _prefixes: Option<String>) {}

  fn prune_xmduals(&self) {}

  pub fn insert_math_token(
    &mut self,
    text: &str,
    mut attributes: HashMap<String, String>,
    font_opt: Option<&Font>,
    state: &mut State,
  ) -> Result<Node>
  {
    // info!(target:"document:insert" ,"insert math token: {:?}", text);
    attributes
      .entry(s!("role"))
      .or_insert_with(|| s!("UNKNOWN"));

    let font = match font_opt {
      Some(f) => f.clone(),
      None => match &self.box_to_absorb {
        &Some(ref tbox) => match tbox.get_font() {
          Some(f) => f.clone(),
          None => Font::math_default(), // should never happen?
        },
        &None => Font::math_default(), // should never happen?
      },
    };
    attributes.remove("mode");
    attributes.remove("stretchy");
    let node = self.open_element(MATH_TOKEN_NAME, Some(attributes), None, state)?;

    // let tbox  = attributes.get("_box").or_insert( LateXML::Box ) // ???
    self.set_node_font(&node, &font);
    if let Some(digested) = self.box_to_absorb.clone() {
      self.set_node_box(&node, digested);
    }
    self.open_math_text_internal(text, state)?;
    self.close_node_internal(&node, state)?; // Should be safe.
    Ok(self.node.clone())
  }

  /// **********************************************************************
  /// Middle level, mostly public, API.
  /// Handlers for various construction operations.
  /// General naming: 'open' opens a node at current pos and sets it to current,
  /// 'close' closes current node(s), inserts opens & closes, ie. w/o moving
  /// current

  /// Tricky: Insert some text in a particular font.
  /// We need to find the current effective -- being the closest  _declared_ font,
  /// (ie. it will appear in the elements attributes).  We may also want
  /// to open/close some elements in such a way as to minimize the font switchiness.
  /// I guess we should only open/close "text" elements, though.
  /// [Actually, we'd like the user to _declare_ what element to use....
  ///  I don't like having "text" built in here!
  ///  AND, we've assumed that "font" names the relevant attribute!!!]

  pub fn open_text(&mut self, text: &str, font: &Font, state: &mut State) -> Result<Option<&Node>> {
    let node_type = self.node.get_type();
    {
      // Ignore initial whitespace
      if ONLY_SPACE_RE.is_match(text)
        && (node_type == Some(NodeType::DocumentNode)
          || (node_type == Some(NodeType::ElementNode)
            && !self.can_contain(&self.node, "#PCDATA", state)))
      {
        return Ok(None);
      }
    }
    if font.family == Some(s!("nullfont")) {
      return Ok(None);
    }
    if self.debug {
      debug!(
        "Insert text {:?} at {:?}",
        text,
        self.document.node_to_string(&self.node)
      );
    }

    if node_type != Some(NodeType::DocumentNode) // If not at document begin
      && !(node_type == Some(NodeType::TextNode) // And not appending text in same font.
      && (font.distance(self.get_node_font(&self.node.get_parent().unwrap())) == 0))
    {
      // then we'll need to do some open/close to get fonts matched.
      let node = self.close_text_internal(state)?; // Close text node, if any.
      let mut bestdiff = 99;
      let mut closeto: Node = node.clone();
      let mut n: Node = node.clone();
      while n.get_type() != Some(NodeType::DocumentNode) {
        let node_font = match self.get_node_font(&n) {
          Some(f) => f.clone(),
          None => Font::text_default(),
        };
        let d = font.distance(Some(&node_font));
        if d < bestdiff {
          bestdiff = d;
          closeto = n.clone();
          if d == 0
            || state.model.get_node_qname(&n) != FONT_ELEMENT_NAME
            || n.get_attribute("_noautoclose").is_some()
          {
            break;
          }
        }
        match n.get_parent() {
          Some(p) => n = p,
          None => break,
        }
      }

      // Move to best starting point for this text.
      if closeto != node {
        self.close_to_node(&closeto, false, state)?;
      }
      if bestdiff > 0 {
        self.open_element(
          FONT_ELEMENT_NAME,
          Some(string_map!("_fontswitch" => "true")),
          Some(font),
          state,
        )?; // Open if needed.
      }
    }

    // Finally, insert the darned text.
    self.open_text_internal(text, state)?;
    self.record_constructed_node(None);
    Ok(Some(&self.node))
  }

  pub fn can_contain(&self, node: &Node, child: &str, state: &mut State) -> bool {
    let tag = state.model.get_node_qname(node);
    state.model.can_contain(&tag, child)
  }

  pub fn can_contain_qname(&self, tag: &str, child: &str, state: &mut State) -> bool {
    // $tag   = $model->getNodeQName($tag)   if ref $tag;      # In case tag is a
    // node. $child = $model->getNodeQName($child) if ref $child;    # In case
    // child is a node.
    state.model.can_contain(tag, child)
  }

  /// Can an element with (qualified name) $tag contain a $childtag element indirectly?
  /// That is, by openning some number of autoOpen'able tags?
  /// And if so, return the tag to open.
  pub fn can_contain_indirect(
    &mut self,
    tag: &str,
    child: &str,
    state: &mut State,
  ) -> Option<String>
  {
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
      Some(sub_m) => match sub_m.get(child) {
        Some(node) => Some(node.to_string()),
        None => None,
      },
      None => None,
    }
  }

  pub fn can_contain_somehow(&mut self, tag: &str, child: &str, state: &mut State) -> bool {
    state.model.can_contain(tag, child) || self.can_contain_indirect(tag, child, state).is_some()
  }

  pub fn can_have_attribute(&mut self, tag: &str, attrib: &str, state: &mut State) -> bool {
    state.model.can_have_attribute(tag, attrib)
  }

  pub fn close_text_internal(&mut self, state: &State) -> Result<Node> {
    if self.node.get_type() == Some(NodeType::TextNode) {
      // Current node is text?
      let parent = self.node.get_parent().unwrap();
      let mut content = self.node.get_content();
      let ocontent = content.clone();
      // let fonttest;
      if let Some(Stored::VecDequeStored(ligatures)) = state.lookup_value("TEXT_LIGATURES") {
        let font_opt = self.get_node_font(&parent);
        for stored_ligature in ligatures.iter() {
          if let Stored::Ligature(ligature) = stored_ligature {
            if let Some(font) = font_opt {
              if let Some(ref font_test) = ligature.font_test {
                if !(font_test)(font) {
                  continue; // if the font test fails, skip the ligature
                }
              }
            }
            content = (ligature.code)(&content);
          }
        }
      }
      if content != ocontent {
        self.node.set_content(&content)?;
      }
      self.set_node(&parent); // Now, effectively Closed
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
      // self.auto_collapse_children(n);
      if *node == n {
        break;
      }
      n = n.get_parent().unwrap();
    }

    self.set_node(&closeto);
    Ok(())
  }

  pub fn open_text_internal(&mut self, text: &str, state: &mut State) -> Result<()> {
    if self.node.get_type() == Some(NodeType::TextNode) {
      // current node already is a text node.
      // if self.debug {
      //   debug!("Appending text \"{:?}\" to {:?}",
      //                   text,
      //                   self.document.node_to_string(&self.node));
      // }
      self.node.append_text(text)?;
    } else if HAS_NONSPACE_RE.is_match(text) || self.can_contain(&self.node, "#PCDATA", state) {
      // or text allowed here
      let mut point = self.find_insertion_point("#PCDATA", state)?;
      let mut node = Node::new_text(text, &self.document).unwrap();
      if self.debug {
        debug!(
          "Inserting text node for {:?} into {:?}",
          text,
          self.document.node_to_string(&point)
        );
      }
      point.add_child(&mut node)?;
      self.set_node(&node);
    }
    Ok(())
  }

  // Question: Why do I have math ligatures handled within openMathText_internal,
  // but text ligatures handled within closeText_internal ???

  fn open_math_text_internal(&mut self, text: &str, state: &mut State) -> Result<Node> {
    // And if there's already text???
    let mut node = self.node.clone();
    // my $font = $self->getNodeFont($node);
    node.append_text(text)?;
    // print STDERR "Trying Math Ligatures at \"$string\"\n";
    if !state.nomathparse {
      self.apply_math_ligatures(&node);
    }
    Ok(node)
  }

  // New stategy (but inefficient): apply ligatures until one succeeds,
  // then remove it, and repeat until ALL (remaining) fail.
  fn apply_math_ligatures(&self, _node: &Node) {
    // my ($self, $node) = @_;
    // if (my $ligatures = $STATE->lookupValue('MATH_LIGATURES')) {
    //   my @ligatures = @$ligatures;
    //   while (@ligatures) {
    //     my $matched = 0;
    //     foreach my $ligature (@ligatures) {
    //       if ($self->applyMathLigature($node, $ligature)) {
    //         @ligatures = grep { $_ ne $ligature } @ligatures;
    //         $matched = 1;
    //         last; } }
    //     return unless $matched; } }
  }

  /// Note that a box has been absorbed creating $node;
  /// This does book keeping so that we can return the sequence of nodes
  /// that were added by absorbing material.
  pub fn record_constructed_node(&mut self, node_opt: Option<&Node>) {
    let node = match node_opt {
      None => &self.node,
      Some(ref n) => n,
    };
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

  //**********************************************************************
  // Low level internal interface

  /// Return a string indicating the path to the current insertion point in the document.
  /// if $levels is defined, show only that many levels
  pub fn get_insertion_context(&self, levels_opt: Option<usize>, state: &State) -> String {
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
    if node_type != Some(NodeType::TextNode)
      && node_type != Some(NodeType::ElementNode)
      && node_type != Some(NodeType::DocumentNode)
    {
      error!(target: "internal:context", "Insertion point is not an element, document or text: {:?}", self.document.node_to_string(&node));
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
  pub fn find_insertion_point(&mut self, qname: &str, state: &mut State) -> Result<Node> {
    self.close_text_internal(state)?; // Close any current text node.
    let cur_qname = state.model.get_node_qname(&self.node);
    // let inter;
    // If `qname` is allowed at the current point, we're done.
    if self.can_contain_qname(&cur_qname, qname, state) {
      return Ok(self.node.clone());
    // Else, if we can create an intermediate node that accepts $qname, we'll do
    // that.
    } else if let Some(inter) = self.can_contain_indirect(&cur_qname, qname, state) {
      if (inter != qname) && (inter != cur_qname) {
        self.open_element(&inter, None, None, state)?; // font => self.getNodeFont(self.node}));
        return self.find_insertion_point(qname, state); // And retry insertion (should work now).
      }
    } else {
      // Now we're getting more desparate...
      // Check if we can auto close some nodes, and _then_ insert the `qname`.
      let mut node = self.node.clone();
      let mut close_to = None;
      while (node.get_type() != Some(NodeType::DocumentNode)) && self.can_auto_close(&node) {
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
        return self.find_insertion_point(qname, state); // Then retry, possibly w/auto open's
      } else {
        // Didn't find a legit place.
        error!(
          target: &s!("malformed:{}", qname),
          "{:?} isn't allowed in <{}>",
          qname,
          cur_qname
        );
        // ($qname eq "#PCDATA" ? $qname : '<' . $qname . '>') . " isn't allowed
        // in <$cur_qname>", "Currently in " .
        // self.getInsertionContext()); return self.node}; } } }

        // But we'll do it anyway, unless Error => Fatal.
        return Ok(self.node.clone());
      }
    }
    Ok(self.node.clone())
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
    if key == "xml:id" {
      // If it's an ID attribute
      // value = self.record_id(value, node);    // Do id book keeping
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
      // let model = self.model;
      // let qname = model.get_node_qname(node);
      // if key.starts_with("_") || model.can_have_attribute(qname, key) {
      node.set_attribute(key, value)?;
      // }
    }
    // else {                   // Accept any namespaced attributes
    //   my ($ns, $name) = self.model}->decodeQName($key);
    //   if ($ns) {             // If namespaced attribute (must have prefix!
    // let prefix = node.lookupNamespacePrefix($ns);    // namespace already
    // declared? if (!$prefix) {                                    // if
    // namespace not already declared $prefix =
    // self.model}->getDocumentNamespacePrefix($ns, 1);    // get the prefix to use
    // self.getDocument->documentElement->setNamespace($ns, $prefix, 0); }
    // // and declare it if ($prefix eq '//default') {    // Probably
    // shouldn't happen...?       node.setAttribute($name => $value); }
    //     else {
    //       node.setAttributeNS($ns, "$prefix:$name" => $value); } }
    //   else {
    //     node.setAttribute($name => $value); } } }    // redundant case...
    Ok(())
  }

  //**********************************************************************
  /// Record the Box that created this node.
  pub fn set_node_box(&mut self, node: &Node, digested: Digested) {
    let nodeid = node.to_hashable();
    self.node_boxes.insert(nodeid, digested);
  }

  pub fn get_node_box(&self, node: &Node) -> Option<&Digested> {
    if node.get_type() == Some(NodeType::ElementNode) {
      let nodeid = node.to_hashable();
      self.node_boxes.get(&nodeid)
    } else {
      None
    }
  }

  //**********************************************************************
  /// Record the Font of a node
  pub fn set_node_font(&mut self, node: &Node, font: &Font) {
    let nodeid = node.to_hashable();
    self.node_fonts.insert(nodeid, font.clone());
  }

  pub fn get_node_font(&self, node: &Node) -> Option<&Font> {
    if node.get_type() == Some(NodeType::ElementNode) {
      let nodeid = node.to_hashable();
      self.node_fonts.get(&nodeid)
    } else {
      None
    }
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
    mut point: Node,
    qname: &str,
    attributes: Option<HashMap<String, String>>,
    font_opt: Option<&Font>,
    state: &mut State,
  ) -> Result<Node>
  {
    let (decoded_ns, tag) = state.model.decode_qname(qname);
    let mut font_opt_cloned: Option<Font> = match font_opt {
      // TODO: lifetime trouble forced me into cloning, there is a better way...
      Some(f) => Some(f.clone()),
      None => None,
    };
    let mut newnode;
    // box = self.node_boxes.get(box);    // may already be the string key

    // If this will be the document root node, things are slightly more involved.
    if point.get_type() == Some(NodeType::DocumentNode) {
      // First node! (?)
      debug!("adding schema declaration, new node will be : {}", tag);
      state.model.add_schema_declaration(self);
      newnode = Node::new(&tag, None, &self.document).unwrap();
      self.document.set_root_element(&newnode);
      for mut node in &mut self.pending {
        newnode.add_prev_sibling(&mut node)?; // Add saved comments, PI's
      }

      if let Some(ns) = decoded_ns {
        // Here, we're creating the initial, document element, which will hold ALL of
        // the namespace declarations. If there is a default namespace (no
        // prefix), that will also be declared, and applied here. However, if
        // there is ALSO a prefix associated with that namespace, we have to declare it
        // FIRST due to the (apparently) buggy way that XML::LibXML works with
        // namespaces in setAttributeNS.
        let prefix = state.model.get_document_namespace_prefix(&ns, false, false);
        let attprefix = state.model.get_document_namespace_prefix(&ns, true, true);
        if prefix.is_none() && attprefix.is_some() {
          let attr_ns_node = Namespace::new(&attprefix.unwrap(), &ns, &mut newnode).unwrap();
          newnode.set_namespace(&attr_ns_node)?;
        }
        // TODO: Figure out a better way to achieve the "activate" effect in
        // XML:LibXML::Element it seems just creating the namespace without
        // setting it is equivalent ??
        let ns_node = Namespace::new("", &ns, &mut newnode).unwrap();
        newnode.set_namespace(&ns_node)?;
      }
      self.record_constructed_node(Some(&newnode));
    } else {
      if font_opt.is_none() {
        font_opt_cloned = match self.get_node_font(&point) {
          // TODO: lifetime trouble here...
          Some(f) => Some(f.clone()),
          None => None,
        };
      }
      // box  = self.get_node_box(point); // unless $box
      newnode = self.open_element_internal(&mut point, decoded_ns, &tag, state)?;
    }

    if let Some(attrs) = attributes {
      let mut sorted_keys = attrs.keys().map(|k| k.to_string()).collect::<Vec<_>>();
      sorted_keys.sort();
      for key in &sorted_keys {
        if key == "font" || key == "locator" {
          continue;
        }
        self.set_attribute(&mut newnode, key, &attrs[key])?;
      }
    }
    if let Some(font) = font_opt_cloned {
      self.set_node_font(&newnode, &font);
    }

    // The .clone on boxes is potentially *VERY SLOW* and a code smell.
    // It can be eventually avoided by using a "memory arena" for all intermediate
    // objects - tokens, boxes, etc. and a well-designed referncing scheme into
    // the driver structs, such as Gullet, Stomach and Document
    if let Some(digested) = self.box_to_absorb.clone() {
      self.set_node_box(&newnode, digested);
    }

    if self.debug {
      debug!(
        "Inserting {:?} into {:?}",
        newnode.get_name(),
        point.get_name()
      ); // if $LaTeXML::Core::Document::DEBUG;
    }

    // Run afterOpen operations
    self.after_open(&mut newnode, state)?;

    Ok(newnode)
  }

  fn open_element_internal(
    &mut self,
    point: &mut Node,
    ns_opt: Option<String>,
    tag: &str,
    state: &mut State,
  ) -> Result<Node>
  {
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
            if let Some(prefix) = state
              .model
              .get_document_namespace_prefix(&ns_uri, false, false)
            {
              if !prefix.is_empty() {
                let mut root = self.document.get_root_element().unwrap();
                match Namespace::new(&prefix, &ns_uri, &mut root) {
                  Ok(ns) => Some(ns),
                  Err(_) => {
                    error!(target: "document:open_element_internal", "failed to create namespace: {:?}", prefix);
                    None
                  },
                }
              } else {
                // default namespace?
                None
              }
            } else {
              error!(target: "document:open_element_internal", "no namespace prefix found for {:?}", ns_uri);
              None
            }
          },
          Some(prefix) => {
            if !prefix.is_empty() {
              let mut root = self.document.get_root_element().unwrap();
              match Namespace::new(&prefix, &ns_uri, &mut root) {
                Ok(ns) => Some(ns),
                Err(_) => {
                  error!(target: "document:open_element_internal", "failed to create namespace: {:?}", prefix);
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
      newnode.set_namespace(&point.get_namespace().unwrap())?;
    }

    self.record_constructed_node(Some(&newnode));
    Ok(newnode)
  }

  /// Whenever a node has been created using openElementAt,
  /// closeElementAt ought to be used to close it, when you're finished inserting into $node.
  /// Basically, this just runs any afterClose operations.
  pub fn close_element_at(&mut self, mut node: &mut Node, state: &mut State) -> Result<()> {
    self.after_close(&mut node, state)
  }

  pub fn after_open(&mut self, node: &mut Node, state: &mut State) -> Result<()> {
    // Set current point to this node, just in case the afterOpen's use it.
    let savenode = self.node.clone();
    self.set_node(&node);
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

  pub fn trim_node_whitespace(&mut self, node: &mut Node) -> Result<()> {
    self.trim_node_left_whitespace(node)?;
    self.trim_node_right_whitespace(node)?;
    Ok(())
  }

  pub fn trim_node_left_whitespace(&mut self, node: &mut Node) -> Result<()> {
    if let Some(mut first_child) = node.get_first_child() {
      match first_child.get_type() {
        Some(NodeType::TextNode) => {
          let content = first_child.get_content();
          let trimmed_content = content.trim_left();
          if !content.is_empty() && (trimmed_content != content) {
            first_child.set_content(trimmed_content)?;
          }
        },
        Some(NodeType::ElementNode) => self.trim_node_left_whitespace(&mut first_child)?,
        _ => {},
      };
    }
    Ok(())
  }

  pub fn trim_node_right_whitespace(&mut self, node: &mut Node) -> Result<()> {
    if let Some(mut last_child) = node.get_last_child() {
      match last_child.get_type() {
        Some(NodeType::TextNode) => {
          let content = last_child.get_content();
          let trimmed_content = content.trim_right();
          if !content.is_empty() && (trimmed_content != content) {
            last_child.set_content(trimmed_content)?;
          }
        },
        Some(NodeType::ElementNode) => self.trim_node_right_whitespace(&mut last_child)?,
        _ => {},
      };
    }
    Ok(())
  }

  pub fn add_resource(&mut self, resource: Resource, state: &mut State) -> Result<()> {
    // let savenode_opt = self.float_to_element("ltx:resource", false);
    let savenode_opt = None;
    let mut attrib: HashMap<String, String> = HashMap::new();
    attrib.insert(s!("src"), resource.name);
    attrib.insert(s!("type"), resource.mimetype);
    attrib.insert(s!("media"), resource.media);
    let content_box = Digested::TBox(Box::new(Tbox {
      text: resource.content,
      ..Tbox::default()
    }));
    self.insert_element("ltx:resource", vec![content_box], Some(attrib), state)?;
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

  pub fn make_error(&mut self, error_class: &str, content: &str, state: &mut State) -> Result<()> {
    let savenode_opt = if !self.is_openable("ltx:ERROR", state) {
      self.float_to_element("ltx:ERROR", false)
    } else {
      None
    };
    self.open_element(
      "ltx:ERROR",
      Some(string_map!("class"=>error_class)),
      None,
      state,
    )?;
    self.close_element("ltx:ERROR", state)?;
    if let Some(savenode) = savenode_opt {
      self.set_node(&savenode);
    }
    Ok(())
  }

  // TODO!
  fn float_to_element(&mut self, element: &str, flag: bool) -> Option<Node> { None }
}

// Auxiliary

fn serialize_string(string: &str) -> String {
  // Basic entities
  let mut serialized = string.replace("&", "&amp;");
  serialized = serialized.replace(">", "&gt;");
  serialized = serialized.replace("<", "&lt;");
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
  serialized = serialized.replace("\"", "&quot;");
  serialized = serialized.replace("\n", "&#10;");
  serialized = serialized.replace("\t", "&#9;");
  serialized
}

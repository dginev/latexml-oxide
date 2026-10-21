//! Block insertion helpers: vertical attachment, demotion, panel floats, captions and their hoisting.

use super::*;

/// Perl `isVAttached` (TeX_Box.pool.ltxml:433-440): the node, or its only child all the way
/// down, has `vattach`. Every child node counts, text included: a `<p>` holding a v-attached
/// box and the text after it is not v-attached (`\parbox{5cm}{\parbox{2cm}{Inner} after
/// inner.}`, which since 57d builds that `<p>`; counting elements alone dropped the outer box's
/// `vattach`). Guard `perfect_kernel_batch56::a_nested_parbox_keeps_its_vattach`. The function
/// matches Perl's; the DOM it reads may not: a paragraph's trimmed trailing whitespace node is
/// freed here and kept empty by Perl (KNOWN_PERL_ERRORS #311).
pub(super) fn is_v_attached(node: &Node) -> bool {
  let mut current = node.clone();
  loop {
    if current.get_attribute("vattach").is_some() {
      return true;
    }
    match current.get_child_nodes().as_slice() {
      [only] if only.get_type() == Some(NodeType::ElementNode) => current = only.clone(),
      _ => return false,
    }
  }
}

/// Make `node`'s content Block.class-valid so `node` can safely become a `<ltx:block>`,
/// demoting the Para.class descendants in place. Every direct `<ltx:logical-block>`/
/// `<ltx:sectional-block>` child is renamed to `<ltx:block>` — KEEPING its attributes (a
/// minipage's `width`, alignment, …) — and every direct `<ltx:para>` child is unwrapped
/// (its Block.model children float up, already valid in a block; a `<para>` never
/// directly holds Para.class, so no pre-demotion is needed). Other children (a `<p>`,
/// `<tabular>`, `<itemize>`, a nested `<block>`, …) are valid there and left untouched,
/// so paragraphs inside a still-valid Para.model container (an `<item>`) are never
/// over-flattened.
///
/// BOTTOM-UP: a logical-block child is demoted (its own Para.class content fixed) BEFORE
/// it is renamed to `<block>`, and the caller renames `node` itself only AFTER calling
/// this — so no Para.class node is ever moved into a `<block>` while still Para.class,
/// which would fire the constructor's `malformed`-content check (a spurious `Error:`).
///
/// Memory-safe snapshot iteration: `rename_node` recreates the child (its handle dies —
/// we recurse into the child BEFORE renaming, via the live handle) and `unwrap_nodes`
/// splices the child's kids up; neither moves the SIBLING handles held in the
/// `get_child_nodes` snapshot, and floated/renamed nodes are never revisited through it.
/// True when `node`'s content can be losslessly demoted into a `<ltx:block>`: every
/// element that would end up as a direct child of the block is valid there. Descends
/// only through the transformable Para.class wrappers (`ltx:para`/`ltx:logical-block`/
/// `ltx:sectional-block` — para is unwrapped, the others renamed to `<block>`), whose
/// own content must likewise be demotable; any OTHER element is a leaf kept as-is, so it
/// must itself be block-valid. A container holding a Para.model-only element that
/// `<block>` cannot hold — a `<float>`, `<TOC>`, a sectioning unit, a `<figure>`/`<table>`
/// — is NOT demotable (renaming to block would strand it, e.g. stanli/webquiz), so the
/// caller leaves it at Perl-parity rather than emitting a fresh `malformed` error.
pub(super) fn subtree_is_block_demotable(node: &Node) -> bool {
  node.get_child_nodes().iter().all(|c| {
    if c.get_type() != Some(NodeType::ElementNode) {
      return true;
    }
    let transformable = with(document::get_node_qname(c), |q| {
      matches!(q, "ltx:para" | "ltx:logical-block" | "ltx:sectional-block")
    });
    if transformable {
      subtree_is_block_demotable(c)
    } else {
      document::can_contain_qsym(pin_static("ltx:block"), document::get_node_qname(c))
    }
  })
}

pub(super) fn demote_para_class_content(document: &mut Document, node: &Node) -> Result<()> {
  for child in node.get_child_nodes() {
    if child.get_type() != Some(NodeType::ElementNode) {
      continue;
    }
    let kind = with(document::get_node_qname(&child), |q| match q {
      "ltx:logical-block" | "ltx:sectional-block" => 1u8,
      "ltx:para" => 2,
      _ => 0,
    });
    match kind {
      1 => {
        demote_para_class_content(document, &child)?; // fix content while child still allows Para.class
        document.rename_node(child, "ltx:block", true)?; // THEN rename (content now Block.class-valid)
      },
      2 => document.unwrap_nodes(child)?,
      _ => {},
    }
  }
  Ok(())
}

/// A sectioning unit, read from the schema: admitted by `ltx:sectional-block` (model
/// `document.body.class*`) but not by `ltx:logical-block` (`Para.model`) — part,
/// chapter, section, subsection, subsubsection, paragraph, slide, slidesequence,
/// sidebar, sectional-block; never a figure/table/theorem/para (those are
/// Para.model). `ltx:subparagraph` is not in `document.body` (LaTeXML.model), so a
/// lone top-level `\subparagraph` in a box keeps Perl's shape; one under a floated
/// `\paragraph` rides along inside it.
pub(super) fn is_sectioning_unit(node: &Node) -> bool {
  let q = document::get_node_qname(node);
  document::can_contain_qsym(pin_static("ltx:sectional-block"), q)
    && !document::can_contain_qsym(pin_static("ltx:logical-block"), q)
}

/// Can `unit` be floated out of `context`? Walk up while each node either admits
/// the unit (then a live `\\section` would open there) or auto-closes (a section,
/// a paragraph, the box's own wrappers). A `<quote>`, a list, a `<figure>` does
/// neither, so a box inside one keeps Perl's shape (#250).
pub(super) fn sectioning_floatable(context: &Node, unit: SymStr) -> bool {
  let mut n = context.clone();
  loop {
    if document::can_contain_qsym(document::get_node_qname(&n), unit) {
      return true;
    }
    if !document::can_auto_close(&n) {
      return false;
    }
    match n.get_parent() {
      Some(p) if p.get_type() == Some(NodeType::ElementNode) => n = p,
      _ => return false,
    }
  }
}

/// The floats whose children are arranged into panels when they close (the `Tag!` `after_close`
/// hooks of sect09.rs: `arrange_panels`, `collapse_float`).
pub(crate) const PANEL_FLOATS: [&str; 3] = ["ltx:figure", "ltx:table", "ltx:float"];

/// Would text arriving at `tag` open an `ltx:p`? Follows the model's auto-open route for
/// `#PCDATA` (a section opens `ltx:para`, a para `ltx:p`); a route that ends in another text
/// holder (`svg:foreignObject` in a drawing, `ltx:bibblock` in a bibitem) opens none.
pub(super) fn text_opens_paragraph(tag: SymStr) -> bool {
  let (text, p) = (pin_static("#PCDATA"), pin_static("ltx:p"));
  let mut step = tag;
  // The route is a handful of steps in LaTeXML.model; the bound guards a model cycle.
  for _ in 0..8 {
    match document::sym_can_contain_somehow(step, text) {
      Some(Some(open)) if open == p => return true,
      Some(Some(open)) => step = open,
      _ => return false,
    }
  }
  false
}

/// Figure material: what a figure shows beside its caption — graphics, pictures, tabulars — and the
/// wrappers a box body puts around them (`para`/`p`, a `\scalebox` inline block, a font `text`),
/// with no prose: text nodes are whitespace only.
fn is_figure_material(node: &Node) -> bool {
  match node.get_type() {
    Some(NodeType::TextNode) => node.get_content().trim().is_empty(),
    Some(NodeType::ElementNode) => with(document::get_node_qname(node), |q| match q {
      "ltx:graphics" | "ltx:picture" | "svg:svg" | "ltx:tabular" | "ltx:rule" | "ltx:break" => true,
      "ltx:para" | "ltx:p" | "ltx:block" | "ltx:inline-block" | "ltx:text" => {
        node.get_child_nodes().iter().all(is_figure_material)
      },
      _ => false,
    }),
    _ => true,
  }
}

/// A `para`/`p` left holding nothing but whitespace and such shells once its material moved.
fn is_empty_shell(node: &Node) -> bool {
  with(document::get_node_qname(node), |q| {
    q == "ltx:para" || q == "ltx:p"
  }) && node
    .get_child_nodes()
    .iter()
    .all(|child| match child.get_type() {
      Some(NodeType::TextNode) => child.get_content().trim().is_empty(),
      Some(NodeType::ElementNode) => is_empty_shell(child),
      _ => true,
    })
}

/// The figure material inside `node`, its `para`/`p` wrappers peeled off: the leaves a figure holds.
fn figure_material_leaves(node: &Node, leaves: &mut Vec<Node>) {
  match node.get_type() {
    Some(NodeType::ElementNode)
      if with(document::get_node_qname(node), |q| {
        q == "ltx:para" || q == "ltx:p"
      }) =>
    {
      for child in node.get_child_nodes() {
        figure_material_leaves(&child, leaves);
      }
    },
    Some(NodeType::ElementNode) => leaves.push(node.clone()),
    _ => {},
  }
}

/// A box that is clearly a figure (user ruling 2026-09-27, 57g): its content is one float holding
/// only its caption — the shape `\captionof` makes, wrapping only the caption in the float
/// (`\@captionof@`, caption_sty.rs; Perl the same, KNOWN_PERL_ERRORS #312) and leaving the image
/// beside it — and figure material. The float takes the material in, in source order (what
/// precedes the caption goes before it), so the box becomes the float with its image:
/// `<figure class="ltx_minipage"><graphics/><caption/></figure>` (arXiv 2605.27134's parbox
/// panels, 2605.06890's `\captionof{table}` minipages). A float that already holds content of its
/// own (`\begin{figure}[H]…`) takes nothing: its order with the material is the author's.
/// Returns the float.
pub(super) fn absorb_figure_material(
  document: &mut Document,
  nodes: &[Node],
) -> Result<Option<Node>> {
  let is_float = |n: &Node| {
    n.get_type() == Some(NodeType::ElementNode)
      && with(document::get_node_qname(n), |q| PANEL_FLOATS.contains(&q))
  };
  let floats: Vec<usize> = (0..nodes.len()).filter(|&i| is_float(&nodes[i])).collect();
  let [at] = floats.as_slice() else {
    return Ok(None);
  };
  let float = nodes[*at].clone();
  let float_children = float.get_child_elements();
  let caption_only = float_children.iter().all(|c| {
    with(document::get_node_qname(c), |q| {
      q == "ltx:tags" || q == "ltx:toccaption" || q == "ltx:caption"
    })
  });
  let caption = float_children.into_iter().find(|c| {
    with(document::get_node_qname(c), |q| {
      q == "ltx:toccaption" || q == "ltx:caption"
    })
  });
  let (true, Some(mut caption)) = (caption_only, caption) else {
    return Ok(None);
  };
  let others: Vec<&Node> = nodes
    .iter()
    .enumerate()
    .filter(|(i, _)| *i != *at)
    .map(|(_, n)| n)
    .collect();
  if !others
    .iter()
    .any(|n| n.get_type() == Some(NodeType::ElementNode))
    || !others.iter().all(|n| is_figure_material(n))
  {
    return Ok(None);
  }
  let mut float = float;
  for (i, node) in nodes.iter().enumerate() {
    if i == *at || node.get_type() != Some(NodeType::ElementNode) {
      continue;
    }
    let mut leaves = Vec::new();
    figure_material_leaves(node, &mut leaves);
    for mut leaf in leaves {
      leaf.unlink();
      if i < *at {
        caption.add_prev_sibling(&mut leaf)?;
      } else {
        float.add_child(&mut leaf)?;
      }
    }
    // The emptied `para`/`p` shells, nested ones included (a leaf that was its own node has
    // moved, not emptied).
    if is_empty_shell(node) {
      document.remove_node(node.clone());
    }
  }
  Ok(Some(float))
}

/// Is `context` a float's own content, seen through the captures of the boxes and environments
/// (`{center}`, `{minipage}`) built in it? A float's content is arranged into panels
/// (`arrange_panels`, Perl's `arrange_panels_and_breaks`, latex_constructs.pool.ltxml:3229-3349).
pub(super) fn in_panel_float(context: &Node) -> bool {
  let mut node = context.clone();
  while document::is_capture_block(&node) {
    match node.get_parent() {
      Some(parent) if parent.get_type() == Some(NodeType::ElementNode) => node = parent,
      _ => return false,
    }
  }
  with(document::get_node_qname(&node), |q| {
    PANEL_FLOATS.contains(&q)
  })
}

pub fn insert_block(
  document: &mut Document,
  contents: &Digested,
  block_attr: HashMap<String, String>,
) -> Result<Vec<Node>> {
  insert_block_as(document, contents, block_attr, false, true)
}

/// [`insert_block`] for a box that is paragraph material: its definition's `\leavevmode`
/// (`enter_horizontal`) left TeX in a paragraph, the whatsit's `in_paragraph` property
/// (`Constructor::digest_to_whatsit`). latex.ltx's `\parbox` ends in `\@iiiparbox` and `{minipage}`
/// in `\@iiiminipage`, which begin with `\leavevmode` (16249-16250, 16305-16306): between
/// paragraphs the box starts one, and the text after it continues that paragraph (KNOWN_PERL_ERRORS
/// #309, DIVERGENCES #338).
pub fn insert_block_in_paragraph(
  document: &mut Document,
  contents: &Digested,
  block_attr: HashMap<String, String>,
  props: &SymHashMap<Stored>,
) -> Result<Vec<Node>> {
  let in_paragraph = matches!(props.get("in_paragraph"), Some(Stored::Bool(true)));
  let Some(id) = props.get("caption_id").map(|id| id.to_string()) else {
    return insert_block_as(document, contents, block_attr, in_paragraph, true);
  };
  let captype = props
    .get("caption_type")
    .map(|t| t.to_string())
    .unwrap_or_default();
  // The float the caption is of — the nearest enclosing float element of its type — numbered by it for now (when no
  // caption of its own did), so that what the box's content builds forms under that number as before: its ids, and
  // the `\label`s `float_to_label` gives the nearest numbered ancestor (an `\fbox`, a `\colorbox` or an `lrbox` around
  // the box makes no panel of it, and the float keeps the number).
  let float = caption_float(document, &captype);
  // A float an earlier box's panel took the number from holds a placeholder id meanwhile (`adopt_box_caption`).
  let float_id = float.as_ref().and_then(node_xml_id);
  let placeholder = float_id
    .clone()
    .filter(|held| lookup_bool(&s!("{FLOAT_PLACEHOLDER_ID}{held}")));
  let provisional = match float.clone() {
    Some(mut float) if float_id.is_none() || placeholder.is_some() => {
      if let Some(held) = &placeholder {
        document.unrecord_id(held);
        remove_xml_id(&mut float);
        remove_value(&s!("{FLOAT_PLACEHOLDER_ID}{held}"));
      }
      document.set_attribute(&mut float, "xml:id", &id)?;
      Some((float, placeholder))
    },
    _ => None,
  };
  let open = ancestor_labels(document);
  let mut nodes = insert_block_as(document, contents, block_attr, in_paragraph, true)?;
  adopt_box_caption(document, &mut nodes, props, &id, provisional, open)?;
  Ok(nodes)
}

/// The state key that marks an id as a float's placeholder (`adopt_box_caption`), by id.
const FLOAT_PLACEHOLDER_ID: &str = "lx@float@placeholder@";

fn node_xml_id(node: &Node) -> Option<String> { node.get_attribute_ns("id", XML_NS) }

/// An `xml:id` is the `id` attribute in the XML namespace, which only the namespaced accessor removes.
pub(crate) fn remove_xml_id(node: &mut Node) { let _ = node.remove_attribute_ns("id", XML_NS); }

/// The nearest enclosing float element of a caption's type: `ltx:figure`, `ltx:table`, or `ltx:float` of class
/// `ltx_float_<type>` (sect09.rs).
fn caption_float(document: &Document, captype: &str) -> Option<Node> {
  let mut node = document.get_element();
  while let Some(n) = node {
    if with(document::get_node_qname(&n), |q| PANEL_FLOATS.contains(&q)) {
      return is_float_of_type(&n, captype).then_some(n);
    }
    node = n.get_parent();
  }
  None
}

fn is_float_of_type(node: &Node, captype: &str) -> bool {
  with(document::get_node_qname(node), |q| match q {
    "ltx:figure" => captype == "figure",
    "ltx:table" => captype == "table",
    "ltx:float" => node.get_attribute("class").is_some_and(|classes| {
      let class = s!("ltx_float_{captype}");
      classes.split_whitespace().any(|c| c == class)
    }),
    _ => false,
  })
}

/// Each element from the insertion point up — the nodes open while a box is built — with its `labels` as they stand.
fn ancestor_labels(document: &Document) -> Vec<(Node, String)> {
  let mut ancestors = Vec::new();
  let mut node = document.get_element();
  while let Some(n) = node {
    if !matches!(n.get_type(), Some(NodeType::ElementNode)) {
      break;
    }
    ancestors.push((n.clone(), n.get_attribute("labels").unwrap_or_default()));
    node = n.get_parent();
  }
  ancestors
}

/// A box that holds its own caption in a float (`end_caption_box`) is that caption's float: the panel it became —
/// itself, or the figure panel it holds, never a node that was open around it — takes the caption's id (from the
/// float that held it for now, `provisional_float`), tags and list, becomes the element of the float it is in when
/// that float is of the caption's type (a captioned minipage in a `table` is a `table`, which `insert_block` never
/// picks; a `\newfloat` one keeps its `ltx_float_<type>` class), and takes the `\label`s its content left on the nodes
/// open around it. With no panel, a float numbered for now keeps the number, and takes its tags and list too.
fn adopt_box_caption(
  document: &mut Document,
  nodes: &mut [Node],
  props: &SymHashMap<Stored>,
  id: &str,
  provisional_float: Option<(Node, Option<String>)>,
  open: Vec<(Node, String)>,
) -> Result<()> {
  let caption_qname = pin_static("ltx:caption");
  let is_open = |n: &Node| open.iter().any(|(o, _)| o == n);
  let in_math = |n: &Node| {
    let mut node = n.get_parent();
    while let Some(p) = node {
      if with(document::get_node_qname(&p), |q| {
        q.starts_with("ltx:XM") || q == "ltx:Math"
      }) {
        return true;
      }
      node = p.get_parent();
    }
    false
  };
  // A panel built here: a float element holding the caption, outside math (whose ids are the math's own).
  let built_panel = |n: &Node| {
    with(document::get_node_qname(n), |q| PANEL_FLOATS.contains(&q))
      && !is_open(n)
      && !in_math(n)
      && n
        .get_child_elements()
        .iter()
        .any(|c| document::get_node_qname(c) == caption_qname)
  };
  let panel = nodes.iter().find(|n| built_panel(n)).cloned().or_else(|| {
    nodes
      .first()
      .and_then(|n| n.get_parent())
      .filter(|p| built_panel(p))
  });
  let Some(mut panel) = panel else {
    if let Some((float, _)) = provisional_float {
      number_caption_float(document, float, props)?;
    }
    return Ok(());
  };
  // The id: the float's, held for it, or the caption's own. The float keeps an id for what is built in it later, a
  // placeholder the next box holds its caption's id in: without one a `\label` there would reach the document.
  if let Some((mut float, placeholder)) = provisional_float {
    if let Some(held) = node_xml_id(&float) {
      document.unrecord_id(&held);
    }
    remove_xml_id(&mut float);
    match placeholder {
      Some(held) => document.set_attribute(&mut float, "xml:id", &held)?,
      None => {
        let prefix = if with(document::get_node_qname(&float), |q| q == "ltx:figure") {
          "fig"
        } else {
          "tab"
        };
        document.generate_id(&mut float, prefix)?;
      },
    }
    if let Some(held) = node_xml_id(&float) {
      assign_value(
        &s!("{FLOAT_PLACEHOLDER_ID}{held}"),
        true,
        Some(Scope::Global),
      );
    }
  }
  if let Some(old) = node_xml_id(&panel) {
    document.unrecord_id(&old);
    remove_xml_id(&mut panel);
  }
  document.set_attribute(&mut panel, "xml:id", id)?;
  // The element of the float it is in, when that float is of the caption's type.
  let captype = props
    .get("caption_type")
    .map(|t| t.to_string())
    .unwrap_or_default();
  if let Some(float) = open
    .iter()
    .map(|(o, _)| o)
    .find(|o| with(document::get_node_qname(o), |q| PANEL_FLOATS.contains(&q)))
    && is_float_of_type(float, &captype)
  {
    let float_qname = document::get_node_qname(float);
    if document::get_node_qname(&panel) != float_qname
      && panel
        .get_parent()
        .is_some_and(|p| with(float_qname, |q| document::can_contain(&p, q)))
    {
      let old = panel.clone();
      panel = document.rename_node_qsym(panel, float_qname, true)?;
      for node in nodes.iter_mut() {
        if *node == old {
          *node = panel.clone();
        }
      }
    }
    if with(float_qname, |q| q == "ltx:float") {
      document.add_class(&mut panel, &s!("ltx_float_{captype}"))?;
    }
  }
  let panel = number_caption_float(document, panel, props)?;
  // The labels its content left on the nodes open around it.
  let mut moved: Vec<String> = Vec::new();
  for (mut ancestor, before) in open {
    let Some(after) = ancestor.get_attribute("labels") else {
      continue;
    };
    let kept: Vec<&str> = before.split_whitespace().collect();
    let added: Vec<&str> = after
      .split_whitespace()
      .filter(|l| !kept.contains(l))
      .collect();
    if added.is_empty() {
      continue;
    }
    moved.extend(added.iter().map(|l| l.to_string()));
    if kept.is_empty() {
      ancestor.remove_attribute("labels")?;
    } else {
      document.set_attribute(&mut ancestor, "labels", &kept.join(" "))?;
    }
  }
  if !moved.is_empty() {
    let mut panel = panel;
    let mut labels: Vec<String> = panel
      .get_attribute("labels")
      .map(|l| l.split_whitespace().map(str::to_string).collect())
      .unwrap_or_default();
    for label in moved {
      if !labels.contains(&label) {
        labels.push(label);
      }
    }
    document.set_attribute(&mut panel, "labels", &labels.join(" "))?;
  }
  Ok(())
}

/// `float` takes a caption's tags (first, where a float's own template places them) and list (`end_caption_box`'s
/// `caption_tags` and `caption_inlist`).
fn number_caption_float(
  document: &mut Document,
  mut float: Node,
  props: &SymHashMap<Stored>,
) -> Result<Node> {
  if let Some(inlist) = props.get("caption_inlist").map(|l| l.to_string())
    && !inlist.trim().is_empty()
  {
    document.set_attribute(&mut float, "inlist", inlist.trim())?;
  }
  if let Some(Stored::Digested(tags)) = props.get("caption_tags") {
    let save = document.get_node().clone();
    document.set_node(&float);
    let before = float.get_last_element_child();
    document.absorb(tags, None)?;
    document.set_node(&save);
    let mut added = float.get_last_element_child();
    if added != before
      && let Some(tags) = added.as_mut()
      && let Some(mut first) = float.get_first_element_child()
      && &first != tags
    {
      tags.unlink();
      first.add_prev_sibling(tags)?;
    }
  }
  Ok(float)
}

/// Whether the element `node` was built from one thing that shows: its box is a whatsit whose last argument — a
/// constructor's body (`\rotatebox`, `\scalebox`, `\resizebox`, `\fbox`, `\colorbox`) — has exactly one item,
/// nested groups flattened, that shows something ([`typesets_content`]). Any other box refuses. An alignment
/// environment around the transform is read by [`environment_is_sole_content`] instead.
fn box_is_sole_content(document: &Document, node: &Node) -> bool {
  fn shown(d: &Digested) -> usize {
    match d.data() {
      DigestedData::List(l) => l.borrow().boxes.iter().map(shown).sum(),
      _ => usize::from(typesets_content(d)),
    }
  }
  let Some(whatsit) = document.get_node_box(node) else {
    return false;
  };
  let DigestedData::Whatsit(w) = whatsit.data() else {
    return false;
  };
  let w = w.borrow();
  matches!(w.get_args().last(), Some(Some(body)) if shown(body) == 1)
}

/// Whether the environment an `ltx:_CaptureBlock_` is being built for is an alignment wrapper (`{center}`,
/// `{flushleft}`, `{flushright}`: its box is the environment's whatsit, whose body is what it holds between `\begin`
/// and `\end`) holding one thing that shows: the transform alone, not a transform and a tabular after it. Any other
/// environment refuses (`{minipage}`, `{framed}`).
fn environment_is_sole_content(document: &Document, capture: &Node) -> bool {
  fn shown(d: &Digested) -> usize {
    match d.data() {
      DigestedData::List(l) => l.borrow().boxes.iter().map(shown).sum(),
      _ => usize::from(typesets_content(d)),
    }
  }
  let Some(whatsit) = document.get_node_box(capture) else {
    return false;
  };
  let DigestedData::Whatsit(w) = whatsit.data() else {
    return false;
  };
  let w = w.borrow();
  // only the alignment wrappers, which make no box of their own (a `{minipage}` keeps its width and class as a panel,
  // which the caption would leave: two minipages side by side shrank to the rotated box; a `{framed}` frame would move
  // onto the float)
  [
    "\\begin{center}",
    "\\begin{flushleft}",
    "\\begin{flushright}",
  ]
  .contains(&w.get_definition().get_cs_name().as_ref())
    && matches!(w.get_body(), Ok(Some(body)) if shown(&body) == 1)
}

/// Whether a box's closing caption may be moved from inside the transform `context` to the float `home` its climb
/// found ([`insert_block_as`]): every element from `context` up to `home` was built from this box alone
/// ([`box_is_sole_content`]: no content beside or after it would follow the moved insertion point out of a frame
/// or a transform) and none is a paragraph, and `home` is a float whose own content holds no other caption (one
/// `\lx@@caption`: a second would make the two one float, with one number for two `\label`s — two `\resizebox`ed
/// captioned panels, a float's own `\caption` after the box, `\subcaption` panels). Otherwise Perl's container
/// stands. Witnesses heria/heria-proposal (heria.cls:737-745 `SidewaysFigure`), 1810.08269; guards
/// `perfect_kernel_batch65a::caption_in_a_rotated_parbox_stays_with_its_figure`,
/// `caption_hoist_never_merges_two_captions`, `caption_hoist_never_moves_content_beside_the_box`.
/// OXIDIZED_DESIGN #529.
pub(super) fn caption_hoist_is_safe(document: &Document, context: &Node, home: &Node) -> bool {
  fn captions(d: &Digested) -> usize {
    match d.data() {
      DigestedData::List(l) => l.borrow().boxes.iter().map(captions).sum(),
      DigestedData::Whatsit(w) => {
        let w = w.borrow();
        usize::from(w.get_definition().get_cs_name() == "\\lx@@caption")
          + w.get_args().iter().flatten().map(captions).sum::<usize>()
          + w
            .get_body()
            .ok()
            .flatten()
            .map_or(0, |body| captions(&body))
      },
      _ => 0,
    }
  }
  let is_float = |n: &Node| with(document::get_node_qname(n), |q| PANEL_FLOATS.contains(&q));
  // The home an alignment environment's capture block (`{center}`, `{flushleft}`, `{flushright}` around the
  // transform: the caption lands in what the environment builds, which its float then holds): the environment holds
  // the transform alone, and the float right above it is the one whose captions count.
  let float = if is_float(home) {
    home.clone()
  } else if with(document::get_node_qname(home), |q| {
    q == "ltx:_CaptureBlock_"
  }) && environment_is_sole_content(document, home)
    && let Some(parent) = home.get_parent().filter(is_float)
  {
    parent
  } else {
    return false;
  };
  let mut node = Some(context.clone());
  while let Some(n) = node {
    if &n == home {
      return document
        .get_node_box(&float)
        .is_some_and(|float| captions(&float) == 1);
    }
    // A paragraph on the way is open to what follows the box in it (`\fbox{\rotatebox…} trailing text`): its
    // node box is only what opened it, so nothing tells the rest of it is not there.
    if with(document::get_node_qname(&n), |q| {
      q == "ltx:p" || q == "ltx:para"
    }) {
      return false;
    }
    if !box_is_sole_content(document, &n) {
      return false;
    }
    node = n.get_parent();
  }
  false
}

/// Where the content a box cannot hold is placed after it ([`insert_block_as`]'s hoist): the nearest ancestor of
/// `context` that holds all of `tail_tags`, climbing only through text containers, never out of a list, an alignment
/// or a float, and the node it follows there (the box itself, or the ancestor's child the climb came from).
pub(super) fn tail_home(
  context: &Node,
  container: &Node,
  tail_tags: &[SymStr],
  kept_empty: bool,
) -> Option<(Node, Node)> {
  let flow = |n: &Node| {
    document::can_contain(n, "#PCDATA")
      || with(document::get_node_qname(n), |t| {
        document::can_contain_somehow(t, "ltx:p")
      })
  };
  // LIST: the climb never leaves a list. Perl floats nothing out of a list
  // (`adjustBackmatterElement` autocloses only `canAutoClose` nodes and a
  // drawing's `svg:g` is not one, latex_constructs.pool.ltxml:3936 /
  // Document.pm find_insertion_point): a bibliography arriving ALONE from a
  // tcolorbox inside an `\item` (`kept` empty, so the TEXT guard above did
  // not hold it) climbed through the `ltx:item`/`ltx:itemize` to the
  // section, which closed the list under the remaining items — they then
  // nested inside one another (biblatex-ext.tex:1301/1326 `optionlist` +
  // `bibexample`, 4 schema errors; RUST-ONLY). The block stays in the box
  // and the model reports it, as Perl does (SHARED `ltx:bibliography` in
  // `ltx:block`). The list-free hoists keep their surpass (xebaposter,
  // juradiss, the parbox caption). Guard
  // `bibliography_in_a_list_item_box_keeps_the_list`.
  let list_boundary = |n: &Node| {
    with(document::get_node_qname(n), |t| {
      matches!(
        t,
        "ltx:item"
          | "ltx:inline-item"
          | "ltx:itemize"
          | "ltx:enumerate"
          | "ltx:description"
          | "ltx:inline-itemize"
          | "ltx:inline-enumerate"
          | "ltx:inline-description"
      )
    })
  };
  let mut anchor = container.clone();
  let mut ancestor = context.clone();
  loop {
    if tail_tags
      .iter()
      .all(|t| with(*t, |s| document::can_contain(&ancestor, s)))
    {
      return Some((anchor, ancestor));
    }
    if list_boundary(&ancestor) || (!flow(&ancestor) && !kept_empty) {
      break;
    }
    // Never out of an alignment: the hoist moves the insertion point too, past a cell whose `</td>` (and row, and
    // table) the alignment still owes at its `&`, `\\` and `\end` — a caption in a minipage in a tabular cell left
    // `<td>` open in the figure (1601.03744, 5 malformed errors); a math `array` likewise. Perl's `floatToElement`
    // (Document.pm:1061-1072) floats past such nodes without closing them and restores the insertion point after;
    // a box the climb leaves here (the rotated `\parbox`) is closed already.
    let alignment = |n: &Node| {
      with(document::get_node_qname(n), |t| {
        matches!(
          t,
          "ltx:td"
            | "ltx:tr"
            | "ltx:thead"
            | "ltx:tbody"
            | "ltx:tfoot"
            | "ltx:tabular"
            | "ltx:XMCell"
            | "ltx:XMRow"
            | "ltx:XMArray"
        )
      })
    };
    // Nor out of a float: an appendix section (or a bibliography) in a minipage in a `table` was lifted past it,
    // leaving the table empty and its caption at the document's top level, with no diagnostic; it now stays in the
    // float and errors there, as Perl's does (a sectioning unit in a float errors, OXIDIZED_DESIGN_DIVERGENCES #189
    // ruling). Witness 2609.05763 (a nomenclature glossary in a framed minipage in a table); guard
    // `perfect_kernel_batch61::appendix_in_a_minipage_stays_in_its_float`.
    // The climb now has three per-tag stops (list, alignment, float), and a quote still leaks
    // (repros/boxes-groups/bibliography_in_minipage_in_quote_keeps_the_quote.tex): SYNC_STATUS 62zd.
    let float = |n: &Node| {
      with(document::get_node_qname(n), |t| {
        matches!(t, "ltx:table" | "ltx:figure" | "ltx:float")
      })
    };
    if alignment(&ancestor) || float(&ancestor) {
      break;
    }
    match ancestor.get_parent() {
      Some(p) if matches!(p.get_type(), Some(NodeType::ElementNode)) => {
        anchor = ancestor;
        ancestor = p;
      },
      _ => break,
    }
  }
  None
}

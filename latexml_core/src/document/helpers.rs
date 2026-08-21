use libxml::tree::Node;

use super::{Document, get_node_qname};
use crate::common::{error::*, xml::XML_NS};

/// Does an `ltx:para` element immediately precede `node`? Both `prune_empty_para`
/// (to decide `ltx_pruned_first`) and the first-paragraph `ltx_noindent` stamp
/// (`tex_paragraph.rs`, OXIDIZED_DESIGN #143) ask this: a paragraph with no
/// preceding `ltx:para` sibling is the first paragraph of its parent, structurally
/// — a signal robust to how many stray `\par`s fired before it (which varies by
/// TeX Live year / dump vs no-dump).
pub fn preceding_para_sibling(node: &Node) -> bool {
  match node.get_prev_element_sibling() {
    None => false,
    Some(prev) => {
      if prev.get_name() == "_spilled_" {
        // Streaming pass 1: earlier siblings were spilled; the placeholder
        // records the LAST spilled node's qname (see `spill_run`) so this
        // test matches what the eager walk would have seen.
        prev.get_attribute("last").as_deref() == Some("ltx:para")
      } else {
        get_node_qname(&prev) == crate::pin!("ltx:para")
      }
    },
  }
}

/// In some cases we could have e.g. a \noindent followed by a {table},
/// in which case we end up with an empty ltx:para which we can prune.
pub fn prune_empty_para(document: &mut Document, node: &mut Node) -> Result<()> {
  let children = node.get_child_elements();
  if children.is_empty() {
    let prev_is_para = preceding_para_sibling(node);
    if !prev_is_para {
      // If `node` WAS the 1st child
      document.add_class(&mut node.get_parent().unwrap(), "ltx_pruned_first")?;
    }
    // Decrement the ID counter on the ancestor that generated this node's id,
    // so that the pruned para's id slot gets reused by the next para.
    if let Some(id) = node.get_attribute_ns("id", XML_NS) {
      // Extract the prefix from the id (e.g. "p7" → prefix "p", counter "7")
      if let Some(pos) = id.rfind('.') {
        let suffix = &id[pos + 1..];
        let prefix: String = suffix.chars().take_while(|c| !c.is_ascii_digit()).collect();
        // Perl `Package.pm:939` — empty prefix uses `_ID_counter_` (single
        // trailing underscore), not `_ID_counter__`.
        let ctrkey = if prefix.is_empty() {
          "_ID_counter_".to_string()
        } else {
          format!("_ID_counter_{}_", prefix)
        };
        if let Some(mut ancestor) = node.get_parent()
          && let Some(ctr_str) = ancestor.get_attribute(&ctrkey)
          && let Ok(ctr) = ctr_str.parse::<u32>()
          && ctr > 0
        {
          ancestor.set_attribute(&ctrkey, &(ctr - 1).to_string())?;
        }
      } else {
        // No dot — top-level id like "p7"
        let prefix: String = id.chars().take_while(|c| !c.is_ascii_digit()).collect();
        // Perl `Package.pm:939` — empty prefix uses `_ID_counter_` (single
        // trailing underscore), not `_ID_counter__`.
        let ctrkey = if prefix.is_empty() {
          "_ID_counter_".to_string()
        } else {
          format!("_ID_counter_{}_", prefix)
        };
        // Find the ancestor with the counter (root element)
        if let Some(root) = document.get_document().get_root_element() {
          let mut root = root;
          if let Some(ctr_str) = root.get_attribute(&ctrkey)
            && let Ok(ctr) = ctr_str.parse::<u32>()
            && ctr > 0
          {
            root.set_attribute(&ctrkey, &(ctr - 1).to_string())?;
          }
        }
      }
      document.unrecord_id(&id);
    }
    node.unlink();
  }
  Ok(())
}

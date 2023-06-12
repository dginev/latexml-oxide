use super::Document;
use crate::common::arena;
use crate::common::error::*;
use crate::state::State;
use libxml::tree::Node;

/// In some cases we could have e.g. a \noindent followed by a {table},
/// in which case we end up with an empty ltx:para which we can prune.
pub fn prune_empty_para(document: &mut Document, node: &mut Node, state: &mut State) -> Result<()> {
  let children = node.get_child_elements();
  if children.is_empty() {
    let prev_opt = node.get_prev_element_sibling();
    if prev_opt.is_none()
      || document.get_node_qname(&prev_opt.unwrap(), state) != arena::pin_static("ltx:para")
    {
      // If `node` WAS the 1st child
      document.add_class(&mut node.get_parent().unwrap(), "ltx_pruned_first")?;
    }
    node.unlink();
  }
  Ok(())
}

use crate::data::{get_grammatical_role, get_token_meaning};
use crate::semantics::ActionContext;
use crate::semantics::XProps;
use crate::semantics::tree::XM;
use crate::semantics::tree::lookup_lex_node;
use latexml_core::binding::def::dialect::get_xmarg_id;
use libxml::tree::{Node, NodeType};
use std::borrow::Cow;
use std::error::Error;

/// Generate a textual token for each node; The parser operates on this encoded
/// string.
pub fn node_to_grammar_lexemes(mathnode: &Node, idx: &mut usize) -> (Vec<String>, Vec<Node>) {
  let child_nodes = filter_hints(mathnode.get_child_nodes());
  node_to_grammar_lexemes_from(mathnode, child_nodes, idx)
}

/// Same as `node_to_grammar_lexemes` but with pre-filtered child nodes.
/// Used when `filter_hints` has already been called (to avoid double-filtering).
pub fn node_to_grammar_lexemes_from(
  mathnode: &Node,
  child_nodes: Vec<Node>,
  idx: &mut usize,
) -> (Vec<String>, Vec<Node>) {
  let mut lexemes = Vec::new();
  let mut nodes = Vec::new();
  let top_role_opt = mathnode.get_attribute("role");
  if let Some(ref top_role) = top_role_opt {
    *idx += 1;
    lexemes.push(format!("start_{top_role}:start:{idx}"));
    nodes.push(mathnode.clone());
  }
  for node in child_nodes.into_iter() {
    if node.get_name() == "XMApp" && node.get_attribute("role").is_some() {
      let role = node.get_attribute("role").unwrap();
      // ARROW-role XMApps (decorated arrows like \xrightarrow{over}) should be
      // atomic terminals. Extract the arrow meaning from the ARROW child token.
      if role == "ARROW" {
        let arrow_meaning = node.get_child_elements().into_iter()
          .find(|ch| ch.get_attribute("role").as_deref() == Some("ARROW"))
          .and_then(|ch| {
            ch.get_attribute("meaning")
              .or_else(|| ch.get_attribute("name"))
              .or_else(|| Some(ch.get_content()))
          })
          .unwrap_or_else(|| "ARROW".to_string());
        *idx += 1;
        let lexeme = format!("ARROW:{arrow_meaning}:{idx}").replace(' ', "");
        lexemes.push(lexeme);
        nodes.push(node);
      } else {
        // Only recurse into XMApp nodes that have a role (scripts, etc.)
        // Role-less XMApps (e.g. \sqrt, already-parsed structures) are atomic.
        let (mut inner_lexes, mut inner_nodes) = node_to_grammar_lexemes(&node, idx);
        for (inner_lex, inner_node) in inner_lexes.drain(..).zip(inner_nodes.drain(..)) {
          lexemes.push(inner_lex);
          nodes.push(inner_node);
        }
      }
    } else {
      let role = get_grammatical_role(&node);
      let mut text = get_token_meaning(&node);
      if text.is_empty() {
        text = "UNKNOWN".to_string();
      }
      *idx += 1;
      // Remap angle brackets to parentheses for parsing (grammar can't handle
      // OPEN:langle without massive ambiguity, but OPEN:( works fine).
      // The original node preserves the actual ⟨⟩ characters for XML output.
      let lexeme = if role == "OPEN" && (text == "langle" || text == "⟨") {
        format!("OPEN:(:{idx}")
      } else if role == "CLOSE" && (text == "rangle" || text == "⟩") {
        format!("CLOSE:):{idx}")
      } else {
        format!("{role}:{text}:{idx}").replace(' ', "")
      };
      lexemes.push(lexeme);
      nodes.push(node);
    }
  }
  if let Some(top_role) = top_role_opt {
    *idx += 1;
    lexemes.push(format!("end_{top_role}:end:{idx}"));
    nodes.push(mathnode.clone());
  }
  (lexemes, nodes)
}

/// Auxiliary separator for ROLE:style-lexeme into ("ROLE:style", '-', lexeme)
pub fn distill_lexeme(name: &str) -> (&str, &str, &str) {
  // dash separates styles, colons separate grammatical roles, and we are
  // only trying to distill the last pure lexeme
  // note that we are only trying to do this reasonably for letter-based names (UNKNOWN:italic-x),
  // since some of the content symbols contain dashes themselves (e.g.
  // OPERATOR:partial-differential)
  if let Some(position) = name.rfind('-') {
    let (base, trailer) = name.split_at(position);
    let (sep, lexeme) = trailer.split_at(1);
    (base, sep, lexeme)
  } else if let Some(position) = name.rfind(':') {
    let (base, trailer) = name.split_at(position);
    let (sep, lexeme) = trailer.split_at(1);
    (base, sep, lexeme)
  } else {
    ("", "", name)
  }
}

/// Parse an XMHint `width` attribute string to points.
/// Supports "3.0mu" (mu → pt by dividing by 1.8) and "1.667pt"/"0.16667em".
/// Handles glue specs like "2.77pt plus 2.77pt" by extracting base dimension.
fn get_xmhint_spacing(width: &str) -> f64 {
  let width = width.trim();
  if width.is_empty() {
    return 0.0;
  }
  // Strip glue stretch/shrink: "2.77pt plus 2.77pt" → "2.77pt"
  let base = width
    .split_once(" plus")
    .or_else(|| width.split_once(" minus"))
    .map_or(width, |(base, _)| base)
    .trim();
  let unit_start = base.find(|c: char| c.is_alphabetic()).unwrap_or(base.len());
  let (number_str, unit) = base.split_at(unit_start);
  let number: f64 = number_str.trim().parse().unwrap_or(0.0);
  match unit.trim() {
    "mu" => number / 1.8,
    "pt" => number,
    "em" => number * 10.0, // assume 10pt font size
    _ => 0.0,
  }
}

/// Filter XMHint nodes from a list of child nodes, transferring their spacing
/// info to adjacent tokens as `lpadding`/`rpadding` attributes (matching Perl).
/// XMHints are also unlinked from the XML tree so they won't be seen again.
/// Large spacings (≥10pt, e.g. \quad) become virtual PUNCT nodes (Perl MathParser.pm L483-490).
pub fn filter_hints(nodes: Vec<Node>) -> Vec<Node> {
  const HINT_PUNCT_THRESHOLD: f64 = 10.0;
  let mut prefiltered: Vec<Node> = Vec::new();
  let mut pending_space: f64 = 0.0;
  // Save hint nodes that contributed to _space, for possible PUNCT reuse
  let mut last_hint_for: Vec<Option<Node>> = Vec::new(); // parallel to prefiltered

  for mut node in nodes {
    if node.get_type() != Some(NodeType::ElementNode) {
      continue;
    }
    if node.get_name() == "XMHint" {
      if let Some(width_str) = node.get_attribute("width") {
        let pts = get_xmhint_spacing(&width_str);
        if pts != 0.0 {
          let prev_role = prefiltered.last().and_then(|n| n.get_attribute("role"));
          if prefiltered.last().is_some() && prev_role.as_deref() != Some("OPEN") {
            let prev = prefiltered.last_mut().unwrap();
            let s: f64 = prev
              .get_attribute("_space")
              .and_then(|v| v.parse().ok())
              .unwrap_or(0.0);
            let _ = prev.set_attribute("_space", &format!("{}", s + pts));
            // Save this hint node for potential PUNCT reuse
            let idx = prefiltered.len() - 1;
            if idx < last_hint_for.len() {
              last_hint_for[idx] = Some(node.clone());
            }
          } else {
            pending_space += pts;
          }
        }
      }
      // Unlink from the XML tree; XMHints are ephemeral
      node.unlink();
    } else {
      if pending_space > 0.0 {
        let _ = node.set_attribute("lpadding", &format!("{:.1}pt", pending_space));
        pending_space = 0.0;
      }
      prefiltered.push(node);
      last_hint_for.push(None);
    }
  }

  // Second pass: convert _space to rpadding (or PUNCT XMHint if above threshold)
  let mut filtered: Vec<Node> = Vec::new();
  for (i, mut node) in prefiltered.into_iter().enumerate() {
    filtered.push(node.clone());
    if let Some(s_str) = node.get_attribute("_space") {
      let _ = node.remove_attribute("_space");
      let s: f64 = s_str.parse().unwrap_or(0.0);
      if s >= HINT_PUNCT_THRESHOLD && node.get_attribute("role").as_deref() != Some("PUNCT") {
        // Perl MathParser.pm L487: create virtual PUNCT XMHint
        // Reuse the saved hint node, setting role="PUNCT"
        if let Some(Some(mut hint)) = last_hint_for.get(i).cloned() {
          let _ = hint.set_attribute("role", "PUNCT");
          // Clean width: round to integer if close, matching Perl format
          let s_rounded = if (s - s.round()).abs() < 0.01 { s.round() } else { s };
          let width = format!("{}pt", s_rounded);
          let _ = hint.set_attribute("width", &width);
          // Remove extraneous attributes from the reused hint node
          let _ = hint.remove_attribute("depth");
          let _ = hint.remove_attribute("height");
          let quads = "q".repeat((s / 10.0) as usize);
          let _ = hint.set_attribute("name", &format!("{quads}uad"));
          filtered.push(hint);
        }
      } else {
        let _ = node.set_attribute("rpadding", &format!("{:.1}pt", s));
      }
    }
  }
  filtered
}

/// Given a list of XML nodes (either libxml nodes, or array representations)
/// return a list of XMRef's referring to those nodes;
/// ensure each source node has an ID (if already instanciated as XML)
/// or _xmkey if still in array rep. since it will get an ID later, and the connection re-made)
/// Note that ltx:XMHint nodes are ephemeral and shouldn't be ref'd!
/// likewise, we avoid creating XMRefs to XMRefs
pub fn create_xmrefs(args: &mut [&mut XM], ctxt: ActionContext) -> Result<Vec<XM>, Box<dyn Error>> {
  let nodes = ctxt.nodes;
  let document = ctxt.document;
  let mut refs = Vec::new();
  for arg in args {
    match arg {
      XM::Token(ref mut props, _meta) => {
        if let Some(id) = props.id.as_ref() {
          refs.push(XM::Ref(XProps {
            id: Some(id.clone()),
            ..XProps::default()
          }));
        } else {
          // Parser-created token without id — use _xmkey for deferred resolution
          let key = get_xmarg_id()?.to_string();
          props.xmkey = Some(Cow::Owned(key.clone()));
          refs.push(XM::Ref(XProps {
            xmkey: Some(Cow::Owned(key)),
            ..XProps::default()
          }));
        }
      },
      XM::Lexeme(lex, _) => {
        // If arg is already XML, it's too late to get automatic ID's
        let node = lookup_lex_node(lex, nodes).expect("lexemes should only have valid ids.");
        // let qname   = document::get_node_qname(node, state);
        // let nodebox     = document.get_node_box(node);

        match node.get_attribute("xml:id") {
          //  already has id, so refer to it.
          Some(id) => refs.push(XM::Ref(XProps {
            id: Some(Cow::Owned(id)),
            ..XProps::default()
          })),
          None => {
            // Generate xml:id for this node so we can reference it
            document.generate_id(&mut node.clone(), "")?;
            let generated_id = node.get_attribute("xml:id")
              .or_else(|| node.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace"))
              .or_else(|| node.get_attribute("id"));
            if let Some(id) = generated_id {
              refs.push(XM::Ref(XProps {
                id: Some(Cow::Owned(id)),
                ..XProps::default()
              }));
            }
          },
        }
      },
      XM::Apply(_op, _args, ref mut props, _meta) => {
        if let Some(id) = props.id.as_ref() {
          refs.push(XM::Ref(XProps {
            id: Some(id.clone()),
            ..XProps::default()
          }));
        } else {
          // not yet instanciated, so hasn't had chance to get auto-id; use _xmkey
          let key = get_xmarg_id()?.to_string();
          props.xmkey = Some(Cow::Owned(key.clone()));
          refs.push(XM::Ref(XProps {
            xmkey: Some(Cow::Owned(key)),
            ..XProps::default()
          }));
        }
      },
      // clone an XMRef (w/o any attributes or id ?) rather than create an XMRef to an XMRef
      XM::Ref(props) => {
        refs.push(XM::Ref(props.clone()));
      },
      XM::Dual(_, _, ref mut props, _) | XM::Wrap(_, ref mut props, _) => {
        if let Some(id) = props.id.as_ref() {
          refs.push(XM::Ref(XProps {
            id: Some(id.clone()),
            ..XProps::default()
          }));
        } else {
          let key = get_xmarg_id()?.to_string();
          props.xmkey = Some(Cow::Owned(key.clone()));
          refs.push(XM::Ref(XProps {
            xmkey: Some(Cow::Owned(key)),
            ..XProps::default()
          }));
        }
      },
      _ => {
        // XMHint's are ephemeral — clone without id
        // Other variants: skip with warning
      },
    }
  }
  Ok(refs)
}

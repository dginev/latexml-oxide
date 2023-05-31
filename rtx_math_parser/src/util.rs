use libxml::tree::Node;
use std::error::Error;
use std::borrow::Cow;
use rtx_core::binding::def::dialect::get_xmarg_id;
use crate::data::{get_grammatical_role, get_token_meaning};
use crate::semantics::tree::lookup_lex_node;
use crate::semantics::tree::XM;
use crate::semantics::XProps;
use crate::semantics::ActionContext;

/// Generate a textual token for each node; The parser operates on this encoded
/// string.
pub fn node_to_grammar_lexemes(mathnode: &Node, idx: &mut usize) -> (Vec<String>, Vec<Node>) {
  let mut lexemes = Vec::new();
  let mut nodes = Vec::new();
  let top_role_opt = mathnode.get_attribute("role");
  if let Some(ref top_role) = top_role_opt {
    *idx += 1;
    lexemes.push(format!("start_{top_role}:start:{idx}"));
    nodes.push(mathnode.clone());
  }
  let child_nodes = filter_hints(mathnode.get_child_nodes());
  for node in child_nodes.into_iter() {
    if node.get_name() == "XMApp" {
      let (mut inner_lexes, mut inner_nodes) = node_to_grammar_lexemes(&node, idx);
      for (inner_lex, inner_node) in inner_lexes.drain(..).zip(inner_nodes.drain(..)) {
        lexemes.push(inner_lex);
        nodes.push(inner_node);
      }
    } else {
      let role = get_grammatical_role(&node);
      let mut text = get_token_meaning(&node);
      if text.is_empty() {
        text = "UNKNOWN".to_string();
      }
      *idx += 1;
      let lexeme = format!("{role}:{text}:{idx}").replace(' ', "");
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

pub fn filter_hints(nodes: Vec<Node>) -> Vec<Node> { nodes }

/// Given a list of XML nodes (either libxml nodes, or array representations)
/// return a list of XMRef's referring to those nodes;
/// ensure each source node has an ID (if already instanciated as XML)
/// or _xmkey if still in array rep. since it will get an ID later, and the connection re-made)
/// Note that ltx:XMHint nodes are ephemeral and shouldn't be ref'd!
/// likewise, we avoid creating XMRefs to XMRefs
pub fn create_xmrefs(args: &mut [&mut XM], ctxt: ActionContext) -> Result<Vec<XM>, Box<dyn Error>> {
  let nodes = ctxt.nodes;
  let document = ctxt.document;
  let state = ctxt.state;
  let mut refs = Vec::new();
  for arg in args {
    match arg {
      XM::Token(props, _meta) => {
        if let Some(id) = props.id.as_ref() {
          refs.push(XM::Ref(XProps{id: Some(id.clone()), ..XProps::default()}));
        }
      },
      XM::Lexeme(lex, _) => {
        // If arg is already XML, it's too late to get automatic ID's
        let node = lookup_lex_node(lex, nodes).expect("lexemes should only have valid ids.");
        // let qname   = document.get_node_qname(node, state);
        // let nodebox     = document.get_node_box(node);

        match node.get_attribute("id") {
          //  already has id, so refer to it.
          Some(id) => refs.push(XM::Ref(XProps{id: Some(Cow::Owned(id)), ..XProps::default()})),
          None => {
            // If arg is already XML, it's too late to get automatic ID's
            document.generate_id(&mut node.clone(), "", state)?;
            refs.push(XM::Ref(
              XProps{ id: Some(Cow::Owned(
                node .get_attribute("id")
                .expect("generate_id should always succeed in setting an id"))),
                .. XProps::default()
              }
            ));
          },
        }
      },
      XM::Apply(_op,_args, ref mut props, _meta) => {
        if let Some(id) = props.id.as_ref() {
          refs.push(XM::Ref(XProps{id: Some(id.clone()), ..XProps::default()}));
        } else {
          // not yet instanciated, so hasn't had chance to get auto-id; use _xmkey
          // DG: Interior mutability Trick to grab a gullet!
          let stomach = state.stomach.clone();
          let key = get_xmarg_id(stomach.borrow_mut().get_gullet_mut(), state)?.to_string();
          props.xmkey = Some(Cow::Owned(key.clone()));
          refs.push(XM::Ref(XProps{xmkey: Some(Cow::Owned(key)), ..XProps::default()}));
        }
      },
      // clone an XMRef (w/o any attributes or id ?) rather than create an XMRef to an XMRef
      XM::Ref(props) => {
        refs.push(XM::Ref(props.clone()));
      }
      // TODO:
      //   # XMHint's are ephemeral, they may disappear; so just clone it w/o id
      //   if ($qname eq 'ltx:XMHint') {
      //     my %attr = ($isarray ? %{ $$arg[1] }
      //       : (map { $document->getNodeQName($_) => $_->getValue } $arg->attributes));
      //     delete $attr{'xml:id'};
      //     push(@refs, [$qname, {%attr}]); }
      _ => unimplemented!(),
    }
  }
  Ok(refs)
}

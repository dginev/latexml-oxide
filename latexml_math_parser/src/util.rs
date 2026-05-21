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
  node_to_grammar_lexemes_ctx(mathnode, child_nodes, idx, false)
}

fn node_to_grammar_lexemes_ctx(
  mathnode: &Node,
  child_nodes: Vec<Node>,
  idx: &mut usize,
  bigop_context: bool,
) -> (Vec<String>, Vec<Node>) {
  let mut lexemes = Vec::new();
  let mut nodes = Vec::new();
  let top_role_opt = mathnode.get_attribute("role");
  if let Some(ref top_role) = top_role_opt {
    *idx += 1;
    // When in bigop context, emit BIGOPSUB/BIGOPSUP instead of POSTSUBSCRIPT/POSTSUPERSCRIPT
    let mapped_role = if bigop_context {
      match top_role.as_str() {
        "POSTSUBSCRIPT" => "BIGOPSUB".to_string(),
        "POSTSUPERSCRIPT" => "BIGOPSUP".to_string(),
        _ => top_role.clone(),
      }
    } else {
      top_role.clone()
    };
    lexemes.push(format!("start_{mapped_role}:start:{idx}"));
    nodes.push(mathnode.clone());
  }
  // Track whether the last emitted token was a bigop (SUMOP/INTOP/etc.)
  // so we can emit bigop-specific script tokens to reduce earley chart ambiguity.
  let mut last_was_bigop = false;
  for node in child_nodes.into_iter() {
    // For XMRef nodes: resolve to get the target's role/meaning for lexing,
    // but keep the ORIGINAL XMRef node in the output so the parse tree
    // preserves XMRef indirection (matching Perl behavior).
    // The get_grammatical_role/get_token_meaning functions already resolve XMRef
    // internally, so we just need to handle the case where the target is a
    // compound node (XMApp without role) — these need flattening.
    // Note: we do NOT replace the node variable — we keep the XMRef.

    if node.get_name() == "XMApp" && node.get_attribute("role").is_some() {
      let role = node.get_attribute("role").unwrap();
      // ARROW/METARELOP-role XMApps (decorated arrows like \xrightarrow{over},
      // \xleftrightarrow[under]{over}) should be atomic terminals.
      // Extract the meaning from the inner ARROW/METARELOP child token.
      if role == "ARROW" || role == "METARELOP" {
        let arrow_meaning = node
          .get_child_elements()
          .into_iter()
          .find(|ch| {
            let cr = ch.get_attribute("role");
            cr.as_deref() == Some("ARROW") || cr.as_deref() == Some("METARELOP")
          })
          .and_then(|ch| {
            ch.get_attribute("meaning")
              .or_else(|| ch.get_attribute("name"))
              .or_else(|| Some(ch.get_content()))
          })
          .unwrap_or_else(|| role.to_string());
        *idx += 1;
        let lexeme = format!("{role}:{arrow_meaning}:{idx}").replace(' ', "");
        lexemes.push(lexeme);
        nodes.push(node);
      } else if node.has_attribute("_rewrite") {
        // Rewrite-created: treat as atomic token with the assigned role.
        // Don't recurse — the inner structure was pre-parsed, and the role
        // on this node overrides whatever the children contain.
        let gram_role = get_grammatical_role(&node);
        let mut text = get_token_meaning(&node);
        if text.is_empty() {
          text = "UNKNOWN".to_string();
        }
        *idx += 1;
        lexemes.push(format!("{gram_role}:{text}:{idx}").replace(' ', ""));
        nodes.push(node);
      } else {
        // Only recurse into XMApp nodes that have a role (scripts, etc.)
        // Role-less XMApps (e.g. \sqrt, already-parsed structures) are atomic.
        // Pass bigop_context for POSTSUBSCRIPT/POSTSUPERSCRIPT following a bigop
        let is_script = matches!(role.as_str(), "POSTSUBSCRIPT" | "POSTSUPERSCRIPT");
        let ctx = last_was_bigop && is_script;
        let children = filter_hints(node.get_child_nodes());
        let (mut inner_lexes, mut inner_nodes) =
          node_to_grammar_lexemes_ctx(&node, children, idx, ctx);
        for (inner_lex, inner_node) in inner_lexes.drain(..).zip(inner_nodes.drain(..)) {
          lexemes.push(inner_lex);
          nodes.push(inner_node);
        }
        // Script following a bigop is still "bigop context" for the next script
        if !is_script {
          last_was_bigop = false;
        }
      }
    } else if node.get_name() == "XMArg" {
      // XMArg is a transparent wrapper (e.g. from \lx@post@subscript).
      // Recurse into its children so the grammar can parse them individually.
      // E.g. _{ij} should emit UNKNOWN:i + UNKNOWN:j, not ATOM:ij.
      let arg_children = filter_hints(node.get_child_nodes());
      let (mut inner_lexes, mut inner_nodes) =
        node_to_grammar_lexemes_ctx(&node, arg_children, idx, false);
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
      // Track bigop tokens for bigop-specific script token emission
      let is_bigop = matches!(
        role.as_str(),
        "SUMOP" | "INTOP" | "LIMITOP" | "DIFFOP" | "BIGOP"
      );
      last_was_bigop = is_bigop;
      // Normalize langle/rangle meaning for consistent grammar matching.
      // Do NOT remap to parentheses — langle_open/rangle_close tokens need
      // distinct identity for QM bra-ket rules vs conditional probability.
      //
      // Perf: Split generic OPEN/CLOSE into OTHER_OPEN/OTHER_CLOSE for delimiters
      // OTHER than parens/brackets/braces/langle. This prevents the grammar's
      // generic `open ~ "OPEN"` prefix match from ALSO matching the specific
      // `lparen = "OPEN:("`, etc. — which was producing 2× duplicate grammar
      // derivations for every `(x)`, `[x]`, `{x}` expression.
      let lexeme = if role == "OPEN" && (text == "⟨" || text == "langle") {
        format!("OPEN:langle:{idx}")
      } else if role == "CLOSE" && (text == "⟩" || text == "rangle") {
        format!("CLOSE:rangle:{idx}")
      } else if role == "OPEN" && !matches!(text.as_str(), "(" | "[" | "{") {
        format!("OTHER_OPEN:{text}:{idx}").replace(' ', "")
      } else if role == "CLOSE" && !matches!(text.as_str(), ")" | "]" | "}") {
        format!("OTHER_CLOSE:{text}:{idx}").replace(' ', "")
      } else if role == "UNKNOWN" && text == "d" {
        // M4: Emit XDIFFUNK for possible differential-d tokens.
        // Only "d" tokens can be diffops; other unknowns skip the diffop rule.
        format!("XDIFFUNK:{text}:{idx}")
      } else if role == "ID" && text == "d" {
        format!("XDIFFID:{text}:{idx}")
      } else if role == "VERTBAR" && text == "|"
        && node.get_attribute("stretchy").as_deref() == Some("true")
      {
        // `\left|...\right|` produces a balanced pair of VERTBAR tokens
        // with `stretchy="true"` (whereas bare `|x|` is `stretchy="false"`).
        // Further distinguish by side: the `\@left` constructor tags
        // the emitted XMTok with `role_side="left"`, `\@right` with
        // `role_side="right"` (tex_math.rs). This refines the `role`
        // attribute (delimiter direction) without changing it, so
        // the grammar can use distinct LEFT_STRETCHY_VERTBAR /
        // RIGHT_STRETCHY_VERTBAR tokens. The kerned-stack norm rules
        // (\vertii, \vertiii, …) then don't have to enumerate every
        // pairing of identical bars. Eliminates the VERTBAR-pairing
        // combinatorial explosion in patterns like
        // `\log^+ ∫ \left| f \right|^k dm(z) ≲ ...` (see
        // docs/MATH_AMBIGUITY_AUDIT.md §2) and unlocks task #263's
        // norm-fenced grammar rules.
        // Defensive fallback: if `role_side` is missing — legacy DOM
        // input or a path that bypassed `\@left`/`\@right` — keep the
        // old undirected STRETCHY_VERTBAR lexeme so legacy rules
        // (eval_at, modulus fence) still work.
        match node.get_attribute("role_side").as_deref() {
          Some("left") => format!("LEFT_STRETCHY_VERTBAR:|:{idx}"),
          Some("right") => format!("RIGHT_STRETCHY_VERTBAR:|:{idx}"),
          _ => format!("STRETCHY_VERTBAR:|:{idx}"),
        }
      } else if role == "PUNCT" && punct_followed_by_wide_space(&node) {
        // PUNCT followed by `\quad` / `\qquad` etc. carries an `rpadding`
        // attribute. This wide spacing is a strong arXiv idiom for
        // "formula separator (with side-condition)" rather than
        // "list-element separator". Tag as WIDE_PUNCT so the grammar
        // can prefer `formulae_apply` over `list_apply` for this
        // separator unambiguously. See docs/MATH_AMBIGUITY_AUDIT.md §2.
        format!("WIDE_PUNCT:,:{idx}")
      } else {
        format!("{role}:{text}:{idx}").replace(' ', "")
      };
      lexemes.push(lexeme);
      nodes.push(node);
    }
  }
  if let Some(ref top_role) = top_role_opt {
    *idx += 1;
    let mapped_end = if bigop_context {
      match top_role.as_str() {
        "POSTSUBSCRIPT" => "BIGOPSUB".to_string(),
        "POSTSUPERSCRIPT" => "BIGOPSUP".to_string(),
        _ => top_role.clone(),
      }
    } else {
      top_role.clone()
    };
    lexemes.push(format!("end_{mapped_end}:end:{idx}"));
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
/// `pub(crate)` so semantics.rs can re-use the same parser when
/// inspecting `rpadding` (task #263 stretchy_norm_fenced guards).
/// Supports "3.0mu" (mu → pt by dividing by 1.8) and "1.667pt"/"0.16667em".
/// Handles glue specs like "2.77pt plus 2.77pt" by extracting base dimension.
pub(crate) fn get_xmhint_spacing(width: &str) -> f64 {
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

/// True iff the given PUNCT-like XMTok carries an `rpadding` attribute
/// large enough to indicate a `\quad`-class spacing — TeX's idiom for
/// separating a main formula from a side-condition (e.g.
/// `A = B, \quad r \notin E`). Threshold of ≥5pt ignores `\,`/`\;`
/// thin-space touch-ups while catching `\quad` (10pt) and `\qquad`.
/// Used by `node_to_grammar_lexemes_ctx` to emit a `WIDE_PUNCT` token
/// the grammar can route through `formulae_apply` unambiguously.
fn punct_followed_by_wide_space(node: &Node) -> bool {
  match node.get_attribute("rpadding") {
    Some(s) => get_xmhint_spacing(&s) >= 5.0,
    None => false,
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
          let s_rounded = if (s - s.round()).abs() < 0.01 {
            s.round()
          } else {
            s
          };
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
  let mut refs = Vec::with_capacity(args.len());
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
        // If arg is already XML, it's too late to get automatic ID's.
        // lookup_lex_node now returns Err for malformed lex strings instead
        // of panicking; skip the arg on failure rather than abort the whole
        // ref-building pass.
        let node = match lookup_lex_node(lex, nodes) {
          Ok(n) => n,
          Err(e) => {
            // Perl MathParser.pm:151 — Error('expected', 'id', undef,
            //   "Cannot find a node with xml:id='$idref'", ...)
            // We don't always have an idref string at this layer; the
            // lookup error itself carries enough context.
            log_math_error!(
              "expected", "id",
              "create_xmrefs: skipping lexeme with invalid node lookup: {}", e
            );
            continue;
          },
        };

        match node.get_attribute("xml:id") {
          //  already has id, so refer to it.
          Some(id) => refs.push(XM::Ref(XProps {
            id: Some(Cow::Owned(id)),
            ..XProps::default()
          })),
          None => {
            // Generate xml:id for this node so we can reference it
            document.generate_id(&mut node.clone(), "")?;
            let generated_id = node
              .get_attribute("xml:id")
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn distill_lexeme_dash_separator() {
    // Dash takes precedence over colon.
    let (base, sep, lex) = distill_lexeme("UNKNOWN:italic-x");
    assert_eq!(base, "UNKNOWN:italic");
    assert_eq!(sep, "-");
    assert_eq!(lex, "x");
  }

  #[test]
  fn distill_lexeme_colon_only() {
    let (base, sep, lex) = distill_lexeme("ROLE:foo");
    assert_eq!(base, "ROLE");
    assert_eq!(sep, ":");
    assert_eq!(lex, "foo");
  }

  #[test]
  fn distill_lexeme_no_separator() {
    let (base, sep, lex) = distill_lexeme("bare");
    assert_eq!(base, "");
    assert_eq!(sep, "");
    assert_eq!(lex, "bare");
  }

  #[test]
  fn distill_lexeme_empty() {
    let (base, sep, lex) = distill_lexeme("");
    assert_eq!(base, "");
    assert_eq!(sep, "");
    assert_eq!(lex, "");
  }

  #[test]
  fn distill_lexeme_trailing_dash() {
    // Edge: last-dash splits after the last hyphen even when the tail is empty.
    let (base, sep, lex) = distill_lexeme("foo-");
    assert_eq!(base, "foo");
    assert_eq!(sep, "-");
    assert_eq!(lex, "");
  }

  #[test]
  fn get_xmhint_spacing_mu_divides_by_1_8() {
    assert!((get_xmhint_spacing("1.8mu") - 1.0).abs() < 1e-6);
    assert!((get_xmhint_spacing("3.6mu") - 2.0).abs() < 1e-6);
  }

  #[test]
  fn get_xmhint_spacing_pt_passes_through() {
    assert!((get_xmhint_spacing("1.667pt") - 1.667).abs() < 1e-6);
    assert!((get_xmhint_spacing("10pt") - 10.0).abs() < 1e-6);
  }

  #[test]
  fn get_xmhint_spacing_em_times_ten() {
    // em assumes 10pt font.
    assert!((get_xmhint_spacing("0.5em") - 5.0).abs() < 1e-6);
  }

  #[test]
  fn get_xmhint_spacing_strips_plus_glue() {
    // "2.77pt plus 2.77pt" extracts base "2.77pt".
    assert!((get_xmhint_spacing("2.77pt plus 2.77pt") - 2.77).abs() < 1e-6);
    assert!((get_xmhint_spacing("5pt minus 1pt") - 5.0).abs() < 1e-6);
  }

  #[test]
  fn get_xmhint_spacing_empty_is_zero() {
    assert_eq!(get_xmhint_spacing(""), 0.0);
    assert_eq!(get_xmhint_spacing("  "), 0.0);
  }

  #[test]
  fn get_xmhint_spacing_unknown_unit_is_zero() {
    assert_eq!(get_xmhint_spacing("5cm"), 0.0);
    assert_eq!(get_xmhint_spacing("garbage"), 0.0);
  }
}

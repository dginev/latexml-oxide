use std::{cell::RefCell, rc::Rc};

use latexml_core::digested::DigestedData;
use latexml_package::prelude::*;

use crate::discard_env::{discard_body_until_cs, read_env_body_tokens};

/// A node in the parsed forest bracket tree.
#[derive(Debug, Clone)]
pub struct ForestNode {
  pub label_tokens:   Vec<Token>,
  pub label_digested: Option<Digested>,
  pub children:       Vec<ForestNode>,
}

/// A parsed forest tree with optional environment configuration, preamble keys, and root nodes.
#[derive(Debug, Clone)]
pub struct ForestTree {
  pub config:   String,
  pub preamble: String,
  pub roots:    Vec<ForestNode>,
}

thread_local! {
  /// The tree read at `before_digest` time, handed to the same whatsit's
  /// `properties` a moment later (the two hooks of one `\begin{forest}` run
  /// back to back, so this is the Rust form of a lexical shared by two Perl
  /// closures). From there it rides on the whatsit as `Stored::Opaque`, so
  /// construction — later, in whatever order deferred whatsits construct —
  /// finds each whatsit's OWN tree (a LIFO pop at construction time once
  /// handed the first `\fbox{\begin{forest}…}` the last tree read; sweep-63
  /// guard).
  static PENDING_FOREST_TREES: RefCell<Vec<ForestTree>> = const { RefCell::new(Vec::new()) };
}

fn is_token_char(t: &Token, ch: char) -> bool {
  t.get_charcode() == ch as u32 && t.get_catcode() != Catcode::CS
}

fn is_space_token(t: &Token) -> bool {
  t.get_catcode() == Catcode::SPACE || t.get_catcode() == Catcode::EOL
}

fn is_ignorable_token(t: &Token) -> bool {
  is_space_token(t) || t.get_catcode() == Catcode::COMMENT
}

/// Reads all tokens belonging to the `{forest}` environment body up to `\end{forest}`.
fn read_forest_env_tokens() -> Result<Vec<Token>> { read_env_body_tokens("forest") }

/// Parses forest bracket tokens according to forest.sty grammar:
/// - (config): optional configuration (forest.sty:8506)
/// - preamble: key-value options before the first unbraced `[` (forest.sty:8666-8680)
/// - `[label, options [child] [child]]`: bracket grammar (forest.sty:1413-1655, 3992-4003)
pub fn parse_forest_tokens(tokens: &[Token]) -> ForestTree {
  let mut idx = 0;

  // 1. Skip leading space / comments
  while idx < tokens.len() && is_ignorable_token(&tokens[idx]) {
    idx += 1;
  }

  // 2. Environment config: (config)
  let mut config = String::new();
  if idx < tokens.len() && is_token_char(&tokens[idx], '(') {
    idx += 1;
    let mut config_tokens = Vec::new();
    let mut paren_depth = 1;
    while idx < tokens.len() {
      let t = &tokens[idx];
      idx += 1;
      if is_token_char(t, '(') {
        paren_depth += 1;
        config_tokens.push(*t);
      } else if is_token_char(t, ')') {
        paren_depth -= 1;
        if paren_depth == 0 {
          break;
        }
        config_tokens.push(*t);
      } else {
        config_tokens.push(*t);
      }
    }
    config = Tokens::new(config_tokens).to_string();
  }

  // 3. Skip whitespace and collect tree preamble before the first unbraced '['
  let mut preamble_tokens = Vec::new();
  let mut brace_depth: usize = 0;
  while idx < tokens.len() {
    let t = &tokens[idx];
    if t.get_catcode() == Catcode::BEGIN {
      brace_depth += 1;
      preamble_tokens.push(*t);
      idx += 1;
    } else if t.get_catcode() == Catcode::END {
      brace_depth = brace_depth.saturating_sub(1);
      preamble_tokens.push(*t);
      idx += 1;
    } else if brace_depth == 0 && is_token_char(t, '[') {
      break;
    } else {
      preamble_tokens.push(*t);
      idx += 1;
    }
  }

  let preamble = Tokens::new(preamble_tokens).to_string().trim().to_string();

  // 4. Parse root nodes: '[' ... ']'
  let mut roots = Vec::new();
  while idx < tokens.len() {
    while idx < tokens.len() && is_ignorable_token(&tokens[idx]) {
      idx += 1;
    }
    if idx >= tokens.len() {
      break;
    }
    if is_token_char(&tokens[idx], '[') {
      let root = parse_node(tokens, &mut idx);
      roots.push(root);
    } else {
      idx += 1;
    }
  }

  ForestTree { config, preamble, roots }
}

fn parse_node(tokens: &[Token], idx: &mut usize) -> ForestNode {
  // Consume opening '['
  if *idx < tokens.len() && is_token_char(&tokens[*idx], '[') {
    *idx += 1;
  }

  let mut brace_depth: usize = 0;
  let mut first_comma_seen = false;
  let mut current_part: Vec<Token> = Vec::new();
  let mut label_tokens: Vec<Token> = Vec::new();
  let mut stopped_by_child = false;

  while *idx < tokens.len() {
    let t = &tokens[*idx];

    if t.get_catcode() == Catcode::BEGIN {
      brace_depth += 1;
      current_part.push(*t);
      *idx += 1;
    } else if t.get_catcode() == Catcode::END {
      brace_depth = brace_depth.saturating_sub(1);
      current_part.push(*t);
      *idx += 1;
    } else if brace_depth > 0 {
      current_part.push(*t);
      *idx += 1;
    } else {
      // brace_depth == 0
      if is_token_char(t, ',') {
        *idx += 1;
        if !first_comma_seen {
          first_comma_seen = true;
          label_tokens = std::mem::take(&mut current_part);
        } else {
          current_part.clear();
        }
      } else if is_token_char(t, '[') {
        stopped_by_child = true;
        break;
      } else if is_token_char(t, ']') {
        break;
      } else {
        current_part.push(*t);
        *idx += 1;
      }
    }
  }

  if !first_comma_seen {
    label_tokens = std::mem::take(&mut current_part);
  }

  let mut children = Vec::new();
  if stopped_by_child {
    while *idx < tokens.len() {
      while *idx < tokens.len() && is_ignorable_token(&tokens[*idx]) {
        *idx += 1;
      }
      if *idx >= tokens.len() {
        break;
      }
      if is_token_char(&tokens[*idx], '[') {
        let child = parse_node(tokens, idx);
        children.push(child);
      } else if is_token_char(&tokens[*idx], ']') {
        *idx += 1; // consume closing ']'
        break;
      } else {
        *idx += 1;
      }
    }
  } else if *idx < tokens.len() && is_token_char(&tokens[*idx], ']') {
    *idx += 1; // consume closing ']'
  }

  ForestNode {
    label_tokens,
    label_digested: None,
    children,
  }
}

fn digest_forest_node(node: &mut ForestNode) -> Result<()> {
  let stripped = Tokens::new(node.label_tokens.clone()).strip_braces();
  let has_content = stripped.unlist_ref().iter().any(|t| !is_ignorable_token(t));
  if has_content {
    let digested = stripped.be_digested()?;
    node.label_digested = Some(digested);
  }
  for child in &mut node.children {
    digest_forest_node(child)?;
  }
  Ok(())
}

fn digest_forest_tree(tree: &mut ForestTree) -> Result<()> {
  for root in &mut tree.roots {
    digest_forest_node(root)?;
  }
  Ok(())
}

// A forest tree is an inline, picture-like object (real forest draws a
// tikzpicture, i.e. `<ltx:picture>`, which is Inline.class): it is emitted as
// the schema's INLINE list (`inline-enumerate`/`inline-item`, Misc.class —
// allowed in text, table cells, figures and paragraphs alike). A block
// `ltx:para`/`ltx:enumerate` wrapper was rejected inside `<ltx:text>`,
// `<ltx:td>` and `<ltx:figure>` (forest-doc 13→133, milsymb 0→1, sweep 63).
// `options`/`config`/`text` attributes are not declared for these elements
// and would be dropped by the model, so they are not computed.
fn emit_forest_tree(document: &mut Document, tree: &ForestTree) -> Result<()> {
  if tree.roots.is_empty() {
    return Ok(());
  }
  let mut enum_attrs = HashMap::default();
  enum_attrs.insert("class".into(), "ltx_forest".into());
  document.open_element("ltx:inline-enumerate", Some(enum_attrs), None)?;
  for root in &tree.roots {
    emit_forest_node(document, root)?;
  }
  document.close_element("ltx:inline-enumerate")?;
  Ok(())
}

fn emit_forest_node(document: &mut Document, node: &ForestNode) -> Result<()> {
  let mut item_attrs: HashMap<String, String> = HashMap::default();
  item_attrs.insert("class".into(), "ltx_forest_node".into());
  document.open_element("ltx:inline-item", Some(item_attrs), None)?;

  if let Some(ref dig) = node.label_digested {
    let mut text_attrs: HashMap<String, String> = HashMap::default();
    text_attrs.insert("class".into(), "ltx_forest_node_content".into());
    document.open_element("ltx:text", Some(text_attrs), None)?;
    document.absorb(dig, None)?;
    document.close_element("ltx:text")?;
  }

  if !node.children.is_empty() {
    let mut enum_attrs = HashMap::default();
    enum_attrs.insert("class".into(), "ltx_forest_children".into());
    document.open_element("ltx:inline-enumerate", Some(enum_attrs), None)?;
    for child in &node.children {
      emit_forest_node(document, child)?;
    }
    document.close_element("ltx:inline-enumerate")?;
  }

  document.close_element("ltx:inline-item")?;
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  // Per-conversion reset (a mid-body fatal can leave a tree behind).
  PENDING_FOREST_TREES.with(|c| c.borrow_mut().clear());
  RequirePackage!("tikz");
  RequirePackage!("etoolbox");
  RawTeX!(r"\ProvidesPackage{forest}[2017/07/14 v2.1.5 Drawing (linguistic) trees]");
  // Semantic tree parser: \begin{forest} reads bracket grammar into a nested
  // inline-enumerate/inline-item tree.
  DefConstructor!(
    T_CS!("\\begin{forest}"), None,
    sub [document, _args, props] {
      // The tree rides on the whatsit's own properties (`Stored::Opaque`),
      // so deferred or nested whatsits construct in any order.
      if let Some(Stored::Opaque(payload)) = props.get("forest_tree")
        && let Some(tree) = payload.downcast_ref::<ForestTree>()
      {
        emit_forest_tree(document, tree)?;
      }
    },
    mode => "text",
    locked => true,
    before_digest => {
      let tokens = read_forest_env_tokens()?;
      let tree = parse_forest_tokens(&tokens);
      PENDING_FOREST_TREES.with(|c| c.borrow_mut().push(tree));
    },
    properties => {
      let mut props = stored_map!();
      if let Some(mut tree) = PENDING_FOREST_TREES.with(|c| c.borrow_mut().pop()) {
        digest_forest_tree(&mut tree)?;
        props.insert("forest_tree", Stored::Opaque(Rc::new(tree)));
      }
      Ok(props)
    }
  );
  // \Forest command: \Forest*(config){ [root [child]] }
  // forest.sty:8511 defines \NewDocumentCommand{\Forest}{s D(){} m} (the star
  // is consumed; its `\forest@group@env` grouping is not modelled).
  // We handle optional * and optional (config) before delegating to \lx@forest@exec.
  RawTeX!(
    r"\def\Forest{\@ifstar{\lx@forest@opt}{\lx@forest@opt}}
\def\lx@forest@opt{\@ifnextchar({\lx@forest@withconfig}{\lx@forest@noconfig}}
\def\lx@forest@withconfig(#1){\lx@forest@exec{#1}}
\def\lx@forest@noconfig{\lx@forest@exec{}}"
  );
  DefConstructor!(
    "\\lx@forest@exec {} Undigested",
    sub [document, _args, props] {
      if let Some(Stored::Opaque(payload)) = props.get("forest_tree")
        && let Some(tree) = payload.downcast_ref::<ForestTree>()
      {
        emit_forest_tree(document, tree)?;
      }
    },
    mode => "text",
    locked => true,
    properties => sub[args] {
      let mut props = stored_map!();
      let config = args[0].as_ref().map(|d| d.to_string()).unwrap_or_default();
      if let Some(Some(body_dig)) = args.get(1) {
        let tokens = match body_dig.data() {
          DigestedData::Postponed(t) => t.unlist_ref().clone(),
          _ => Vec::new(),
        };
        let mut tree = parse_forest_tokens(&tokens);
        if !config.is_empty() && tree.config.is_empty() {
          tree.config = config;
        }
        digest_forest_tree(&mut tree)?;
        props.insert("forest_tree", Stored::Opaque(Rc::new(tree)));
      }
      Ok(props)
    }
  );
  // The bare-CS form `\forest … \endforest` that `\NewDocumentEnvironment
  // {forest}{D(){}}` (forest.sty:8506) also defines — neoschool.cls:8567-8581
  // builds its `neotree` env on it; without it `\forest` was undefined and
  // the tree body leaked as text (`\frac` XMApp errors). Guard:
  // `perfect_kernel_batch54::forest_bare_cs_form_discards_body`.
  DefConstructor!(
    T_CS!("\\forest"), None,
    "<ltx:ERROR>{forest}</ltx:ERROR>",
    bounded => true,
    mode    => "text",
    locked  => true,
    before_digest => { discard_body_until_cs("forest", "\\endforest", "forest.sty.ltxml")?; }
  );
  DefMacro!("\\endforest", "\\relax");
  // forest.sty:1413 bracket-parser configuration (neoschool.cls:8568
  // `\bracketset{action character=@}`); nothing to configure in a stub.
  DefMacro!("\\bracketset{}", "\\relax");
  DefMacro!("\\forestset{}", "\\relax");
  DefMacro!("\\forestoption{}", "\\relax");
  DefMacro!("\\foresteoption{}", "\\relax");
  DefMacro!("\\forestregister{}", "\\relax");
  DefMacro!("\\foresteregister{}", "\\relax");
  DefMacro!("\\useforestlibrary[]{}", "\\relax");
});

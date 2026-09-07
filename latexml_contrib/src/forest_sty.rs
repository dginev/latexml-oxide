use std::cell::RefCell;

use latexml_core::digested::DigestedData;
use latexml_package::prelude::*;

use crate::discard_env::discard_body_until_cs;

/// A node in the parsed forest bracket tree.
#[derive(Debug, Clone)]
pub struct ForestNode {
  pub label_tokens: Vec<Token>,
  pub label_text:   String,
  pub options_text: String,
  pub children:     Vec<ForestNode>,
}

/// A parsed forest tree with optional environment configuration, preamble keys, and root nodes.
#[derive(Debug, Clone)]
pub struct ForestTree {
  pub config:   String,
  pub preamble: String,
  pub roots:    Vec<ForestNode>,
}

thread_local! {
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
fn read_forest_env_tokens() -> Result<Vec<Token>> {
  let end_delim = Tokens!(T_CS!("\\end"));
  let mut body_tokens: Vec<Token> = Vec::new();
  loop {
    let upto_end = read_until(&end_delim)?;
    if let Some(toks) = upto_end {
      body_tokens.extend(toks.unlist());
    }
    let Some(drop_open) = read_token()? else {
      break;
    };
    let env = read_balanced(ExpansionLevel::Off, false, false)?;
    if env.to_string() == "forest" {
      break;
    } else {
      // Nested environment inside forest! E.g. \begin{tabular}...\end{tabular}
      body_tokens.push(T_CS!("\\end"));
      body_tokens.push(drop_open);
      body_tokens.extend(env.unlist());
      body_tokens.push(T_OTHER!("}"));
    }
  }
  Ok(body_tokens)
}

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
  let mut options: Vec<String> = Vec::new();
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
          let opt = Tokens::new(std::mem::take(&mut current_part))
            .to_string()
            .trim()
            .to_string();
          if !opt.is_empty() {
            options.push(opt);
          }
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
  } else {
    let opt = Tokens::new(std::mem::take(&mut current_part))
      .to_string()
      .trim()
      .to_string();
    if !opt.is_empty() {
      options.push(opt);
    }
  }

  let stripped_label_tokens = Tokens::new(label_tokens.clone()).strip_braces();
  let label_text = stripped_label_tokens.to_string().trim().to_string();
  let options_text = options.join(", ");

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
    label_text,
    options_text,
    children,
  }
}

fn emit_forest_tree(document: &mut Document, tree: &ForestTree) -> Result<()> {
  let mut para_attrs = HashMap::default();
  para_attrs.insert("class".into(), "ltx_forest_tree".into());
  if !tree.preamble.is_empty() {
    para_attrs.insert("options".into(), tree.preamble.clone());
  }
  if !tree.config.is_empty() {
    para_attrs.insert("config".into(), tree.config.clone());
  }
  document.open_element("ltx:para", Some(para_attrs), None)?;

  if !tree.roots.is_empty() {
    let mut enum_attrs = HashMap::default();
    enum_attrs.insert("class".into(), "ltx_forest".into());
    document.open_element("ltx:enumerate", Some(enum_attrs), None)?;

    for root in &tree.roots {
      emit_forest_node(document, root)?;
    }

    document.close_element("ltx:enumerate")?;
  }

  document.close_element("ltx:para")?;
  Ok(())
}

fn emit_forest_node(document: &mut Document, node: &ForestNode) -> Result<()> {
  let mut item_attrs: HashMap<String, String> = HashMap::default();
  item_attrs.insert("class".into(), "ltx_forest_node".into());
  if !node.label_text.is_empty() {
    item_attrs.insert("text".into(), node.label_text.clone());
  }
  if !node.options_text.is_empty() {
    item_attrs.insert("options".into(), node.options_text.clone());
  }
  document.open_element("ltx:item", Some(item_attrs), None)?;

  if !node.label_text.is_empty() {
    document.open_element("ltx:tags", None, None)?;
    document.open_element("ltx:tag", None, None)?;
    document.absorb_string(&node.label_text, &SymHashMap::default())?;
    document.close_element("ltx:tag")?;
    document.close_element("ltx:tags")?;

    let mut content_attrs = HashMap::default();
    content_attrs.insert("class".into(), "ltx_forest_node_content".into());
    document.open_element("ltx:para", Some(content_attrs), None)?;
    document.absorb_string(&node.label_text, &SymHashMap::default())?;
    document.close_element("ltx:para")?;
  }

  if !node.children.is_empty() {
    let mut enum_attrs = HashMap::default();
    enum_attrs.insert("class".into(), "ltx_forest_children".into());
    document.open_element("ltx:enumerate", Some(enum_attrs), None)?;
    for child in &node.children {
      emit_forest_node(document, child)?;
    }
    document.close_element("ltx:enumerate")?;
  }

  document.close_element("ltx:item")?;
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("tikz");
  RequirePackage!("etoolbox");
  RawTeX!(r"\ProvidesPackage{forest}[2017/07/14 v2.1.5 Drawing (linguistic) trees]");
  // Semantic tree parser: \begin{forest} reads bracket grammar into a nested
  // semantic ltx:para/ltx:enumerate tree.
  DefConstructor!(
    T_CS!("\\begin{forest}"), None,
    sub [document, _args, _props] {
      let tree_opt = PENDING_FOREST_TREES.with(|c| c.borrow_mut().pop());
      if let Some(tree) = tree_opt {
        emit_forest_tree(document, &tree)?;
      }
    },
    mode => "text",
    locked => true,
    before_digest => {
      let tokens = read_forest_env_tokens()?;
      let tree = parse_forest_tokens(&tokens);
      PENDING_FOREST_TREES.with(|c| c.borrow_mut().push(tree));
    }
  );
  // \Forest command: \Forest*{ [root [child]] }
  DefConstructor!(
    "\\Forest OptionalMatch:* Undigested",
    sub [document, args, _props] {
      if let Some(Some(body_dig)) = args.get(1) {
        let tokens = match body_dig.data() {
          DigestedData::Postponed(t) => t.unlist_ref().clone(),
          _ => Vec::new(),
        };
        let tree = parse_forest_tokens(&tokens);
        emit_forest_tree(document, &tree)?;
      }
    },
    mode => "text",
    locked => true
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

use std::{cell::RefCell, rc::Rc};

use latexml_core::{common::cleaners::clean_class_name, digested::DigestedData};
use latexml_package::prelude::*;

use crate::discard_env::{discard_body_until_cs, read_env_body_tokens};

/// One item of a forest keylist, `key` or `key=value`, split by pgfkeys'
/// rules (see `parse_forest_keylist`). `value` is `None` for a bare key.
#[derive(Debug, Clone)]
pub struct ForestKey {
  pub key:   String,
  pub value: Option<Vec<Token>>,
}

/// The node options that shape the emitted tree, resolved from every keylist
/// that reaches the node (`resolve_forest_node`): forest.sty:3923 `edge
/// label` (TikZ code drawn after the edge path, :3913-3920), :3903 `tier`,
/// :3924 `phantom` (the node is not drawn, :7628-7631) and :3780-3788
/// `name`/`name'`. Every other forest option is layout and stays ignored.
#[derive(Debug, Clone, Default)]
pub struct ForestNodeKeys {
  pub edge_label: Option<Vec<Token>>,
  pub tier:       Option<String>,
  pub phantom:    bool,
  pub name:       Option<String>,
}

/// A node in the parsed forest bracket tree.
#[derive(Debug, Clone)]
pub struct ForestNode {
  pub label_tokens:   Vec<Token>,
  pub label_digested: Option<Digested>,
  /// The label as a string when its tokens are not plain horizontal-mode
  /// content (see `is_plain_label`).
  pub label_text:     Option<String>,
  /// The node's own options: the keylist after its label.
  pub options:        Vec<ForestKey>,
  /// The options resolved at digestion (`resolve_forest_node`).
  pub keys:           ForestNodeKeys,
  /// The texts of the edge label's TikZ `node`s, digested; empty when the
  /// edge is not drawn.
  pub edge_label:     Vec<Digested>,
  pub children:       Vec<ForestNode>,
}

/// A parsed forest tree with optional environment configuration, preamble keys, and root nodes.
#[derive(Debug, Clone)]
pub struct ForestTree {
  pub config:   String,
  /// The keys before the first bracket, processed at the root before its own
  /// options (forest.sty:6045-6049 `stages`: `for root'={process keylist
  /// register=default preamble, process keylist register=preamble}`).
  pub preamble: Vec<ForestKey>,
  pub roots:    Vec<ForestNode>,
  /// The tree's text uses the bracket parser's action character at brace
  /// depth 0 (`\bracketset{action character=@}`, forest.sty:1419; none by
  /// default, :1422; matched at :1491). An action expands macros while the
  /// tree is read (`@{…}`, `@+`, `@<token>`, :1598-1627) or hands the
  /// reading to user code that resumes it with `\bracketResume` (`@@`,
  /// :1622-1623, :1450). The token-level parser performs no action, so the
  /// nodes it reads are not forest's: forest-doc.tex:1784-1795 builds the
  /// phantom root's children ×1…×6 and f o r e s t through `\x`'s `@@`,
  /// which the parser reads as the root's label. `phantom` is then not
  /// applied (`resolve_forest_node`), so no text pdflatex prints is hidden —
  /// at the price of showing text it does not print: the action characters
  /// and the bracket text forest turns into nodes (`@@[×1[f]]…` in one label
  /// where pdflatex draws 13 nodes on two tiers), and a phantom node's own
  /// label and its children's edge labels, which forest does not draw
  /// (forest.sty:7628-7631, 7643-7651). Reach: forest-doc's two phantom
  /// action trees (:1784-1795, :5379-5410).
  pub actions:  bool,
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
  /// Libraries named by package options, loaded at the end of the package
  /// (forest.sty:146-151 `\forest@loadlibrarieslater`, :160-161).
  static LIBRARIES_LATER: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
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

// ---------------------------------------------------------------------------
// Keylists (pgfkeys syntax)

/// Splits `tokens` at every `sep` character outside braces.
fn split_unbraced(tokens: &[Token], sep: char) -> Vec<&[Token]> {
  let mut parts = Vec::new();
  let mut depth: usize = 0;
  let mut start = 0;
  for (i, t) in tokens.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && is_token_char(t, sep) => {
        parts.push(&tokens[start..i]);
        start = i + 1;
      },
      _ => {},
    }
  }
  parts.push(&tokens[start..]);
  parts
}

/// A keylist item's key or value as pgfkeys reads it: surrounding spaces
/// trimmed, then one pair of braces around the whole removed
/// (pgfkeys.code.tex:507-520 `\pgfkeys@spdef`).
fn pgfkeys_trim(tokens: &[Token]) -> Vec<Token> {
  let start = tokens
    .iter()
    .position(|t| !is_ignorable_token(t))
    .unwrap_or(tokens.len());
  let end = tokens
    .iter()
    .rposition(|t| !is_ignorable_token(t))
    .map_or(start, |i| i + 1);
  let trimmed = &tokens[start..end];
  if trimmed.len() >= 2
    && trimmed[0].get_catcode() == Catcode::BEGIN
    && trimmed[trimmed.len() - 1].get_catcode() == Catcode::END
  {
    // The outer braces go only when they enclose the whole item (`{a}{b}`
    // keeps both groups).
    let mut depth: usize = 0;
    let encloses = trimmed.iter().enumerate().all(|(i, t)| {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth = depth.saturating_sub(1),
        _ => {},
      }
      depth > 0 || i == trimmed.len() - 1
    });
    if encloses {
      return trimmed[1..trimmed.len() - 1].to_vec();
    }
  }
  trimmed.to_vec()
}

fn tokens_text(tokens: &[Token]) -> String {
  Tokens::new(tokens.to_vec()).to_string().trim().to_string()
}

/// Splits a forest keylist the way pgfkeys does (forest hands a node's
/// specification to pgfkeys as `content=<spec>`, forest.sty:1423-1426 `new
/// node`, and processes it at :6050 `process keylist=given options`): items
/// at unbraced commas (pgfkeys.code.tex:357 `\pgfkeys@@normal#1,`), the key
/// at the first unbraced `=` (:369 `\pgfkeys@unpack#1=#2=#3`), each part
/// trimmed (`pgfkeys_trim`); an empty key is skipped (:372-373).
pub fn parse_forest_keylist(tokens: &[Token]) -> Vec<ForestKey> {
  let mut keys = Vec::new();
  for item in split_unbraced(tokens, ',') {
    let mut parts = split_unbraced(item, '=');
    let key_tokens = parts.remove(0);
    let key = tokens_text(&pgfkeys_trim(key_tokens));
    if key.is_empty() {
      continue;
    }
    // The value is everything after the first `=`, further `=` included.
    let value = (!parts.is_empty()).then(|| {
      let value_start = key_tokens.len() + 1;
      pgfkeys_trim(&item[value_start..])
    });
    keys.push(ForestKey { key, value });
  }
  keys
}

// ---------------------------------------------------------------------------
// Styles and tree-wide defaults

/// Limits of keylist processing. A style that invokes itself loops in TeX
/// until a capacity is exceeded; here the processing stops, with one error
/// (`KeyContext::report_runaway`), at the first limit it hits:
/// - `MAX_KEY_DEPTH` nested styles: the self-invoking style (`a/.style={a}`,
///   and `a/.style={a,a}`, whose first branch nests without end). No style
///   expands after it; plain keys still apply. The binding enters no
///   conditional (`if …`, `where …`, `delay`), so a nested self-invocation
///   only ends when a value runs out (`a/.style={#1}` used as `a=a`); a
///   style found on its own expansion stack is therefore not an error by
///   itself.
/// - the work budget, a pool of `KEY_WORK_BASE` units plus
///   `KEY_WORK_PER_NODE` for each node of the tree (`KEY_WORK_BASE` alone for
///   a `\forestset` or one library's defaults): one unit per key processed
///   plus the tokens of its name and value (`key_work`; a key replayed from
///   an ancestor's `for tree` pays again at every node), and one per token a
///   key copies — a style's expansion (its body with the value substituted,
///   measured before it is built), a style or `default preamble` definition
///   (the new body). No key is processed after it. As every piece of work is
///   paid for, fan-out (`a/.style={b,b}`, `b/.style={c,c}`, … over a large
///   style body), a value that doubles per level (`a/.style={a={#1#1}}`), a
///   style that appends to itself (`t/.style={t/.append style={…}}`) and a
///   large value replayed down a deep tree all run out, and no single
///   expansion outgrows the pool. Measured need: at most 1,206 units per
///   node and 12,534 per tree (forest-doc, milsymb), 34,260 for one
///   `\forestset` (prooftrees.sty).
const MAX_KEY_DEPTH: usize = 64;
const KEY_WORK_BASE: usize = 1_000_000;
const KEY_WORK_PER_NODE: usize = 10_000;

/// The limit keylist processing ran into (`KeyContext::runaway`).
#[derive(Debug, Clone, Copy)]
enum KeyLimit {
  Depth,
  Budget,
}

/// A key's pgfkeys path relative to `/forest`, and its handler (`style`,
/// `append style`, `try`, …) when the last path component is `.handler`.
fn split_key_path(key: &str) -> (&str, Option<&str>) {
  let key = key.strip_prefix("/forest/").unwrap_or(key);
  match key.rsplit_once("/.") {
    Some((path, handler)) => (path, Some(handler)),
    None => (key, None),
  }
}

fn style_value_key(path: &str) -> String { s!("forest@style@{path}") }

/// The `\forestset` register holding `default preamble` (forest.sty:3793).
const DEFAULT_PREAMBLE: &str = "forest@default@preamble";

fn lookup_tokens(key: &str) -> Option<Vec<Token>> {
  match lookup_value(key) {
    Some(Stored::Tokens(tokens)) => Some(tokens.unlist()),
    _ => None,
  }
}

/// Keylist concatenation, as forest's keylist registers append with a `,`
/// infix (forest.sty:3343-3344).
fn join_keylists(first: Vec<Token>, second: Vec<Token>) -> Vec<Token> {
  if first.is_empty() {
    return second;
  }
  let mut joined = first;
  if !second.is_empty() {
    joined.push(T_OTHER!(","));
    joined.extend(second);
  }
  joined
}

/// A `.style` invoked with a value: pgfkeys stores the style as the code
/// `\pgfkeysalso{<body>}` of a one-parameter key (pgfkeys.code.tex:826), so
/// `#1` is the value and `##` a literal `#`.
fn substitute_style_argument(body: &[Token], value: Option<&[Token]>) -> Vec<Token> {
  let mut out = Vec::with_capacity(body.len());
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    if t.get_catcode() == Catcode::PARAM && i + 1 < body.len() {
      let next = body[i + 1];
      if next.get_catcode() == Catcode::PARAM {
        out.push(next);
        i += 2;
        continue;
      }
      if is_token_char(&next, '1') {
        out.extend_from_slice(value.unwrap_or(&[]));
        i += 2;
        continue;
      }
    }
    out.push(t);
    i += 1;
  }
  out
}

/// An upper bound on the length of `substitute_style_argument(body, value)`,
/// computed without building it: the body plus the value once per `#1`.
fn substituted_len(body: &[Token], value: Option<&[Token]>) -> usize {
  let uses = body
    .windows(2)
    .filter(|w| w[0].get_catcode() == Catcode::PARAM && is_token_char(&w[1], '1'))
    .count();
  body
    .len()
    .saturating_add(uses.saturating_mul(value.map_or(0, <[Token]>::len)))
}

/// Where style definitions land: the document state (`\forestset`), or the
/// tree being read (a style defined among a node's options is "local to the
/// current tree", forest-doc.tex:2545). Carries the work budget (see
/// `MAX_KEY_DEPTH`).
struct KeyContext {
  local_styles: HashMap<String, Vec<Token>>,
  persist:      bool,
  /// Work units left; 0 once spent, and no key is processed after that.
  budget:       usize,
  /// What the budget was granted for, for the error message.
  budget_scope: String,
  /// The first limit hit and the style (or key) that hit it.
  runaway:      Option<(String, KeyLimit)>,
  /// The styles being run, innermost last (`with_style`).
  running:      Vec<String>,
}

impl KeyContext {
  /// A `\forestset` or one library's defaults (`what`): the base pool.
  fn document(what: &str) -> Self {
    KeyContext {
      local_styles: HashMap::default(),
      persist:      true,
      budget:       KEY_WORK_BASE,
      budget_scope: s!("the {KEY_WORK_BASE} units of {what}"),
      runaway:      None,
      running:      Vec::new(),
    }
  }

  fn tree(nodes: usize) -> Self {
    let nodes = nodes.max(1);
    KeyContext {
      local_styles: HashMap::default(),
      persist:      false,
      budget:       KEY_WORK_PER_NODE
        .saturating_mul(nodes)
        .saturating_add(KEY_WORK_BASE),
      budget_scope: s!(
        "the tree's {KEY_WORK_BASE} units plus {KEY_WORK_PER_NODE} per node, {nodes} node{}",
        if nodes == 1 { "" } else { "s" }
      ),
      runaway:      None,
      running:      Vec::new(),
    }
  }

  /// Spends `work` units on `key`; false, and the runaway recorded, once the
  /// budget cannot pay.
  fn spend(&mut self, key: &str, work: usize) -> bool {
    if work > self.budget {
      self.budget = 0;
      if self.runaway.is_none() {
        // Name the innermost style being run, where the work multiplied.
        let culprit = self.running.last().map_or(key, String::as_str).to_string();
        self.runaway = Some((culprit, KeyLimit::Budget));
      }
      return false;
    }
    self.budget -= work;
    true
  }

  /// Expands the style `path` (`expand_style`) and runs its keylist with
  /// `run`, recording it as the innermost style while it runs.
  fn with_style(
    &mut self,
    path: &str,
    value: Option<&[Token]>,
    depth: usize,
    run: impl FnOnce(&[ForestKey], &mut Self),
  ) {
    if let Some(style) = self.expand_style(path, value, depth) {
      self.running.push(path.to_string());
      run(&style, self);
      self.running.pop();
    }
  }

  /// Reports, once, the first limit keylist processing hit.
  fn report_runaway(&self) -> Result<()> {
    let Some((key, limit)) = &self.runaway else {
      return Ok(());
    };
    let message = match limit {
      KeyLimit::Depth if self.budget > 0 => s!(
        "forest style '{key}' nests more than {MAX_KEY_DEPTH} styles deep (it invokes \
         itself); style expansion stopped"
      ),
      KeyLimit::Depth => s!(
        "forest style '{key}' nests more than {MAX_KEY_DEPTH} styles deep (it invokes \
         itself); style expansion stopped, then the keylist work budget ({}) ran out and \
         keylist processing stopped",
        self.budget_scope
      ),
      KeyLimit::Budget => s!(
        "forest key '{key}' ran out of the keylist work budget ({}: styles multiply or grow); \
         keylist processing stopped",
        self.budget_scope
      ),
    };
    Error!("misdefined", key, message);
    Ok(())
  }

  fn lookup_style(&self, path: &str) -> Option<Vec<Token>> {
    self
      .local_styles
      .get(path)
      .cloned()
      .or_else(|| lookup_tokens(&style_value_key(path)))
  }

  /// pgfkeys' `.style`, `.append style` and `.prefix style` handlers
  /// (pgfkeys.code.tex:826, 836-837); other handlers define code or
  /// defaults, which carry no structure. The new body is paid for.
  fn define_style(&mut self, path: &str, handler: &str, body: Vec<Token>) {
    let old = match handler {
      "style" => Vec::new(),
      "append style" | "prefix style" => self.lookup_style(path).unwrap_or_default(),
      _ => return,
    };
    if !self.spend(
      path,
      1usize.saturating_add(old.len()).saturating_add(body.len()),
    ) {
      return;
    }
    let new = if handler == "prefix style" {
      join_keylists(body, old)
    } else {
      join_keylists(old, body)
    };
    if self.persist {
      // pgfkeys defines the key with `\def`: local to the TeX group.
      assign_value(
        &style_value_key(path),
        Stored::Tokens(Tokens::new(new.clone())),
        None,
      );
    }
    self.local_styles.insert(path.to_string(), new);
  }

  /// The style `path` invoked with `value` at nesting `depth`, as a keylist,
  /// its expansion paid for; `None` for an unknown key, and for every style
  /// once a limit is hit.
  fn expand_style(
    &mut self,
    path: &str,
    value: Option<&[Token]>,
    depth: usize,
  ) -> Option<Vec<ForestKey>> {
    if self.runaway.is_some() {
      return None;
    }
    let body = self.lookup_style(path)?;
    if depth >= MAX_KEY_DEPTH {
      self.runaway = Some((path.to_string(), KeyLimit::Depth));
      return None;
    }
    if !self.spend(path, substituted_len(&body, value).saturating_add(1)) {
      return None;
    }
    Some(parse_forest_keylist(&substitute_style_argument(
      &body, value,
    )))
  }
}

/// `\forestset{<keylist>}` outside a tree (forest.sty:130 `\pgfqkeys{/forest}`):
/// defines styles, extends `default preamble` — the keylist forest runs at
/// the root of every tree before its own preamble (forest.sty:3793-3794,
/// 6045-6049) — and runs styles, which may do either.
fn forestset(keys: &[ForestKey], ctx: &mut KeyContext, depth: usize) {
  for key in keys {
    if !ctx.spend(&key.key, key_work(key)) {
      return;
    }
    let value = key.value.as_deref();
    match split_key_path(&key.key) {
      // A keylist register: the bare and `+` forms append, `'` sets
      // (forest.sty:3343-3347).
      ("default preamble" | "default preamble+", None) => {
        let old = lookup_tokens(DEFAULT_PREAMBLE).unwrap_or_default();
        let value = value.unwrap_or(&[]);
        if ctx.spend(&key.key, old.len().saturating_add(value.len())) {
          let new = join_keylists(old, value.to_vec());
          assign_value(DEFAULT_PREAMBLE, Stored::Tokens(Tokens::new(new)), None);
        }
      },
      ("default preamble'", None) => {
        let value = value.unwrap_or(&[]);
        if ctx.spend(&key.key, value.len()) {
          assign_value(
            DEFAULT_PREAMBLE,
            Stored::Tokens(Tokens::new(value.to_vec())),
            None,
          );
        }
      },
      (path, Some(handler)) if handler != "try" => {
        ctx.define_style(path, handler, value.unwrap_or(&[]).to_vec());
      },
      // A style (or `.try` of one), run where it is used; an unknown key does
      // nothing.
      (path, _) => {
        ctx.with_style(path, value, depth, |style, ctx| {
          forestset(style, ctx, depth + 1)
        });
      },
    }
  }
}

/// What processing `key` costs: one unit, plus the tokens of its name and
/// value, which the key reads or copies (see `MAX_KEY_DEPTH`).
fn key_work(key: &ForestKey) -> usize {
  1usize
    .saturating_add(key.key.len())
    .saturating_add(key.value.as_ref().map_or(0, Vec::len))
}

/// A forest boolean's value (forest.sty:2656-2666: `\forestmath@if`, default
/// 1).
fn forest_boolean(value: Option<&[Token]>) -> bool {
  value.is_none_or(|v| !matches!(tokens_text(v).as_str(), "0" | "false"))
}

/// Runs a keylist on one node, in order. A `for tree={…}` keylist applies to
/// the node and, through `for_descendants`, to every node below it
/// (forest.sty:5592 `define long step={tree}`); it runs while the node's own
/// options are processed, so it precedes the descendants' own options
/// (forest.sty:6050, given options in processing order).
fn apply_node_keys(
  keys: &[ForestKey],
  node: &mut ForestNodeKeys,
  ctx: &mut KeyContext,
  mut for_descendants: Option<&mut Vec<Vec<ForestKey>>>,
  depth: usize,
) {
  for key in keys {
    if !ctx.spend(&key.key, key_work(key)) {
      return;
    }
    let value = key.value.as_deref();
    match split_key_path(&key.key) {
      (path, Some(handler)) if handler != "try" => {
        ctx.define_style(path, handler, value.unwrap_or(&[]).to_vec());
      },
      ("edge label", None) => node.edge_label = value.map(<[Token]>::to_vec),
      ("tier", None) => node.tier = value.map(tokens_text).filter(|t| !t.is_empty()),
      ("phantom", None) => node.phantom = forest_boolean(value),
      ("not phantom", None) => node.phantom = false,
      ("name" | "name'", None) => node.name = value.map(tokens_text).filter(|n| !n.is_empty()),
      ("for tree", None) => {
        let subtree_keys = parse_forest_keylist(value.unwrap_or(&[]));
        // The descendants re-run the whole keylist, so a nested `for tree`
        // reaches them through it rather than through another push.
        apply_node_keys(&subtree_keys, node, ctx, None, depth + 1);
        // An empty keylist applies nothing at any node: not pushed, so the
        // replay never walks it.
        if !subtree_keys.is_empty()
          && let Some(pushed) = for_descendants.as_deref_mut()
        {
          pushed.push(subtree_keys);
        }
      },
      // A style (or `.try` of one), run where it is used.
      (path, _) => {
        ctx.with_style(path, value, depth, |style, ctx| {
          apply_node_keys(style, node, ctx, for_descendants.as_deref_mut(), depth + 1);
        });
      },
    }
  }
}

// ---------------------------------------------------------------------------
// The bracket parser

/// The bracket parser's special tokens, `\bracketset{opening bracket=…,
/// closing bracket=…, action character=…}` (forest.sty:1413-1422, defaults
/// `[` and `]` and no action character).
#[derive(Debug, Clone, Copy)]
struct Brackets {
  open:   Token,
  close:  Token,
  action: Option<Token>,
}

const OPENING_BRACKET: &str = "forest@bracket@open";
const CLOSING_BRACKET: &str = "forest@bracket@close";
const ACTION_CHARACTER: &str = "forest@bracket@action";

impl Brackets {
  fn current() -> Self {
    let get = |key: &str| match lookup_value(key) {
      Some(Stored::Token(t)) => Some(t),
      _ => None,
    };
    Brackets {
      open:   get(OPENING_BRACKET).unwrap_or(T_OTHER!("[")),
      close:  get(CLOSING_BRACKET).unwrap_or(T_OTHER!("]")),
      action: get(ACTION_CHARACTER),
    }
  }
}

/// `\ifx#1\bracket@openingBracket` (forest.sty:1483, likewise the closing
/// bracket and the action character, :1487, :1491), compared by token, not
/// by meaning: a character by character code and catcode — the letter `@`
/// that prooftrees.sty:940 and neoschool.cls:8568 set under `\makeatletter`
/// is no action in a document whose `@` is other — and a control sequence
/// by name. (`\ifx` compares the meaning `\bracketset` captured with `\let`,
/// forest.sty:1416, so a control sequence `\let` to `[` would also match
/// there.)
///
/// Of the node options, the parser models only what `ForestNodeKeys`
/// lists; not modelled are, among others, the append/prepend forms `edge
/// label+`/`+edge label` (forest.sty:2413-2418) and the `'` forms, the
/// `.expanded` handler, `/.cd` path changes, and a `content=` option
/// overriding the label.
fn is_bracket(t: &Token, bracket: &Token) -> bool { t == bracket }

/// Parses forest bracket tokens according to forest.sty grammar:
/// - (config): optional configuration (forest.sty:8506)
/// - preamble: key-value options before the first unbraced `[` (forest.sty:8666-8680)
/// - `[label, options [child] [child]]`: bracket grammar (forest.sty:1413-1655, 3992-4003)
pub fn parse_forest_tokens(tokens: &[Token]) -> ForestTree {
  let brackets = Brackets::current();
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
  let body_start = idx;
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
    } else if brace_depth == 0 && is_bracket(t, &brackets.open) {
      break;
    } else {
      preamble_tokens.push(*t);
      idx += 1;
    }
  }

  let preamble = parse_forest_keylist(&preamble_tokens);

  // 4. Parse root nodes: '[' ... ']'
  let mut roots = Vec::new();
  while idx < tokens.len() {
    while idx < tokens.len() && is_ignorable_token(&tokens[idx]) {
      idx += 1;
    }
    if idx >= tokens.len() {
      break;
    }
    if is_bracket(&tokens[idx], &brackets.open) {
      let root = parse_node(tokens, &mut idx, &brackets);
      roots.push(root);
    } else if roots.is_empty() {
      idx += 1;
    } else {
      // A non-bracket token after the tree is trailing material, not another
      // root: forest-doc.tex:1055's `\measureydistance[…=#1]{…}` follows the
      // tree, and reading its optional argument as a root emitted a bogus
      // node (batch 56bv).
      break;
    }
  }

  // The action character anywhere the bracket parser read (the preamble, a
  // node's specification, between nodes), outside TeX groups (a group is
  // appended whole, forest.sty:1476-1477, 1527).
  let actions = brackets.action.is_some_and(|action| {
    let mut depth: usize = 0;
    tokens[body_start..idx].iter().any(|t| {
      match t.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth = depth.saturating_sub(1),
        _ => return depth == 0 && is_bracket(t, &action),
      }
      false
    })
  });

  ForestTree {
    config,
    preamble,
    roots,
    actions,
  }
}

fn parse_node(tokens: &[Token], idx: &mut usize, brackets: &Brackets) -> ForestNode {
  // Consume the opening bracket
  if *idx < tokens.len() && is_bracket(&tokens[*idx], &brackets.open) {
    *idx += 1;
  }

  // The node specification runs to the first unbraced bracket; forest hands
  // it to pgfkeys as `content=<spec>` (forest.sty:1423-1426 `new node`), so
  // its first keylist item is the label and the rest are the node's options.
  let mut brace_depth: usize = 0;
  let mut spec: Vec<Token> = Vec::new();
  let mut stopped_by_child = false;
  while *idx < tokens.len() {
    let t = tokens[*idx];
    match t.get_catcode() {
      Catcode::BEGIN => brace_depth += 1,
      Catcode::END => brace_depth = brace_depth.saturating_sub(1),
      _ if brace_depth == 0 && is_bracket(&t, &brackets.open) => {
        stopped_by_child = true;
        break;
      },
      _ if brace_depth == 0 && is_bracket(&t, &brackets.close) => break,
      _ => {},
    }
    spec.push(t);
    *idx += 1;
  }
  let mut parts = split_unbraced(&spec, ',').into_iter();
  let label_tokens = parts.next().unwrap_or_default().to_vec();
  let option_tokens = &spec[label_tokens.len().min(spec.len())..];
  // Skip the comma that ends the label.
  let options = parse_forest_keylist(option_tokens.get(1..).unwrap_or_default());

  let mut children = Vec::new();
  if stopped_by_child {
    while *idx < tokens.len() {
      while *idx < tokens.len() && is_ignorable_token(&tokens[*idx]) {
        *idx += 1;
      }
      if *idx >= tokens.len() {
        break;
      }
      if is_bracket(&tokens[*idx], &brackets.open) {
        let child = parse_node(tokens, idx, brackets);
        children.push(child);
      } else if is_bracket(&tokens[*idx], &brackets.close) {
        *idx += 1; // consume the closing bracket
        break;
      } else {
        *idx += 1;
      }
    }
  } else if *idx < tokens.len() && is_bracket(&tokens[*idx], &brackets.close) {
    *idx += 1; // consume the closing bracket
  }

  ForestNode {
    label_tokens,
    label_digested: None,
    label_text: None,
    options,
    keys: ForestNodeKeys::default(),
    edge_label: Vec::new(),
    children,
  }
}

// ---------------------------------------------------------------------------
// Digestion

/// A plain forest node label is horizontal-mode content and is digested as
/// TeX (a `$x^2$` label becomes `<Math>`). PARAM or ALIGN tokens, and the
/// alignment-only primitives, mark a stream that is not a label at all —
/// forest-doc.tex:1055 `\measureydistance[…=#1]{…}` (a macro's optional
/// argument after the tree, mis-read as a root by the stub parser) and :3142
/// `special value&actual value\\\hline…` (forest's `align` feature, a
/// tabular inside a node) — which the stomach can only report (`#` should
/// never reach Stomach, stray `&`, `\noalign`); such a label is kept as its
/// string (sweep 75, round-10 N3 follow-up).
/// A bare `\\\\` is NOT in the list: a multi-line `align=center` node label
/// digests it to a line break.
fn is_plain_label(tokens: &Tokens) -> bool {
  tokens.unlist_ref().iter().all(|t| {
    !matches!(t.get_catcode(), Catcode::PARAM | Catcode::ALIGN)
      && !(t.get_catcode() == Catcode::CS
        && t.with_str(|s| {
          matches!(
            s,
            "\\noalign" | "\\cr" | "\\crcr" | "\\omit" | "\\span" | "\\hline"
          )
        }))
  })
}

/// Whether `tokens[i..]` starts the word `word` (letters, not followed by a
/// letter).
fn starts_word(tokens: &[Token], i: usize, word: &str) -> bool {
  let n = word.chars().count();
  i + n <= tokens.len()
    && word
      .chars()
      .zip(&tokens[i..i + n])
      .all(|(c, t)| t.get_catcode() == Catcode::LETTER && t.get_charcode() == c as u32)
    && tokens
      .get(i + n)
      .is_none_or(|t| t.get_catcode() != Catcode::LETTER)
}

/// The index just past the group opening at `tokens[i]` and closing at the
/// matching `close` character (or the brace group, for `{`).
fn skip_delimited(tokens: &[Token], i: usize, open: char, close: char) -> usize {
  let mut depth: usize = 0;
  let mut j = i;
  while j < tokens.len() {
    let t = &tokens[j];
    j += 1;
    let (opens, closes) = if open == '{' {
      (
        t.get_catcode() == Catcode::BEGIN,
        t.get_catcode() == Catcode::END,
      )
    } else {
      (is_token_char(t, open), is_token_char(t, close))
    };
    if opens {
      depth += 1;
    } else if closes {
      depth = depth.saturating_sub(1);
      if depth == 0 {
        break;
      }
    }
  }
  j
}

/// The texts of the TikZ `node` operations in an `edge label` (the value is
/// path code, forest.sty:3913-3920). After `node`, TikZ scans `at (…)`,
/// `(name)`, `[options]` and `:anim=…` in any order up to the `{text}` group
/// (tikz.code.tex:3783-3834 `\tikz@@scan@fig`).
fn edge_label_texts(code: &[Token]) -> Vec<Vec<Token>> {
  let mut texts = Vec::new();
  let mut i = 0;
  while i < code.len() {
    let word_start = i == 0 || code[i - 1].get_catcode() != Catcode::LETTER;
    if !(word_start && starts_word(code, i, "node")) {
      i += 1;
      continue;
    }
    i += 4;
    loop {
      while i < code.len() && is_ignorable_token(&code[i]) {
        i += 1;
      }
      let Some(t) = code.get(i) else { break };
      if starts_word(code, i, "at") {
        i += 2;
      } else if is_token_char(t, '(') {
        i = skip_delimited(code, i, '(', ')');
      } else if is_token_char(t, '[') {
        i = skip_delimited(code, i, '[', ']');
      } else if t.get_catcode() == Catcode::BEGIN {
        let end = skip_delimited(code, i, '{', '}');
        texts.push(code[i + 1..end.saturating_sub(1).max(i + 1)].to_vec());
        i = end;
        break;
      } else {
        break;
      }
    }
  }
  texts
}

/// Digests a label's tokens: plain content as TeX, anything else as its
/// string (`is_plain_label`).
fn digest_label(tokens: Tokens) -> Result<(Option<Digested>, Option<String>)> {
  let has_content = tokens.unlist_ref().iter().any(|t| !is_ignorable_token(t));
  if !has_content {
    Ok((None, None))
  } else if is_plain_label(&tokens) {
    Ok((Some(tokens.be_digested()?), None))
  } else {
    Ok((None, Some(tokens.to_string())))
  }
}

/// Resolves a node's options and digests its label and edge label. The
/// options reaching a node run in forest's order: the `for tree` keylists of
/// its ancestors (root first), then — at the root only — `default preamble`
/// and the tree preamble (`root_keys`), then its own options. In a tree
/// with `actions`, `phantom` is not applied (see `ForestTree::actions`).
/// `inherited` is a stack shared by the whole tree: a node pushes its own
/// `for tree` keylists on top of its ancestors', its subtree reads them, and
/// it pops them when done, so no keylist is copied per node.
fn resolve_forest_node(
  node: &mut ForestNode,
  root_keys: &[ForestKey],
  inherited: &mut Vec<Vec<ForestKey>>,
  parent_drawn: Option<bool>,
  actions: bool,
  ctx: &mut KeyContext,
) -> Result<()> {
  let mut keys = ForestNodeKeys::default();
  let ancestors = inherited.len();
  // Once the budget is spent a keylist applies nothing, so the replay stops:
  // walking the rest of the stack at every node would be unpaid work, O(nodes
  // × stack) (a `for tree={}` fan-out at the root, then thousands of leaves).
  for keylist in inherited.iter() {
    if ctx.budget == 0 {
      break;
    }
    apply_node_keys(keylist, &mut keys, ctx, None, 0);
  }
  apply_node_keys(root_keys, &mut keys, ctx, Some(inherited), 0);
  let options = std::mem::take(&mut node.options);
  apply_node_keys(&options, &mut keys, ctx, Some(inherited), 0);
  node.options = options;
  if actions {
    keys.phantom = false;
  }

  // A phantom node's content is still typeset (forest typesets every node)
  // but never drawn (forest.sty:7628-7631), so it is digested, not emitted.
  let (digested, text) = digest_label(Tokens::new(node.label_tokens.clone()).strip_braces())?;
  node.label_digested = digested;
  node.label_text = text;
  // An edge, and the label drawn with its path, exists only when the node
  // and its parent are both drawn (forest.sty:7643-7651).
  let edge_drawn = !keys.phantom && parent_drawn == Some(true);
  node.edge_label.clear();
  if edge_drawn && let Some(code) = &keys.edge_label {
    for text in edge_label_texts(code) {
      if let (Some(digested), _) = digest_label(Tokens::new(text))? {
        node.edge_label.push(digested);
      }
    }
  }
  let drawn = !keys.phantom;
  node.keys = keys;
  for child in &mut node.children {
    resolve_forest_node(child, &[], inherited, Some(drawn), actions, ctx)?;
  }
  inherited.truncate(ancestors);
  Ok(())
}

fn count_nodes(nodes: &[ForestNode]) -> usize {
  nodes
    .iter()
    .map(|node| 1 + count_nodes(&node.children))
    .sum()
}

fn digest_forest_tree(tree: &mut ForestTree) -> Result<()> {
  let mut ctx = KeyContext::tree(count_nodes(&tree.roots));
  let mut root_keys = parse_forest_keylist(&lookup_tokens(DEFAULT_PREAMBLE).unwrap_or_default());
  root_keys.extend(tree.preamble.iter().cloned());
  let mut inherited = Vec::new();
  for root in &mut tree.roots {
    resolve_forest_node(
      root,
      &root_keys,
      &mut inherited,
      None,
      tree.actions,
      &mut ctx,
    )?;
  }

  ctx.report_runaway()
}

// ---------------------------------------------------------------------------
// Construction

// A forest tree is a picture-like object (real forest draws a tikzpicture,
// `<ltx:picture>`), and its nodes are emitted as the schema's INLINE list
// (`inline-enumerate`/`inline-item`, Misc.class — allowed in text, table
// cells, figures and paragraphs alike), in an `ltx:inline-block
// class="ltx_forest_tree"` wrapper. Every form — the `{forest}` environment,
// `\Forest` and `\Forest*` — draws its tree as an inline box in pdflatex
// (forest.sty:8506-8514: the star only drops the group, :8528;
// forest-doc.tex:1927-1929), so `text \begin{forest}…\end{forest} more`
// stays one paragraph; a block wrapper was also a schema error inside
// `\fbox` and table cells (forest-doc 13→133, milsymb 0→1, sweep 63).
// `options`/`config`/`text` attributes are not declared for these elements
// and would be dropped by the model, so they are not computed.
fn emit_forest_tree(document: &mut Document, tree: &ForestTree) -> Result<()> {
  if tree.roots.is_empty() {
    return Ok(());
  }
  let mut wrapper_attrs = HashMap::default();
  wrapper_attrs.insert("class".into(), "ltx_forest_tree".into());
  document.open_element("ltx:inline-block", Some(wrapper_attrs), None)?;
  let mut enum_attrs = HashMap::default();
  enum_attrs.insert("class".into(), "ltx_forest".into());
  document.open_element("ltx:inline-enumerate", Some(enum_attrs), None)?;
  for root in &tree.roots {
    emit_forest_node(document, root)?;
  }
  document.close_element("ltx:inline-enumerate")?;
  document.close_element("ltx:inline-block")?;
  Ok(())
}

fn emit_forest_node(document: &mut Document, node: &ForestNode) -> Result<()> {
  let mut class = String::from("ltx_forest_node");
  if node.keys.phantom {
    class.push_str(" ltx_forest_phantom");
  }
  if let Some(tier) = node.keys.tier.as_deref().map(clean_class_name)
    && !tier.is_empty()
  {
    class.push_str(&s!(" ltx_forest_tier_{tier}"));
  }
  let mut item_attrs: HashMap<String, String> = HashMap::default();
  item_attrs.insert("class".into(), class);
  if let Some(name) = &node.keys.name {
    item_attrs.insert("xml:id".into(), clean_id(&s!("forest.{name}")));
  }
  document.open_element("ltx:inline-item", Some(item_attrs), None)?;

  // The edge label is drawn on the edge from the parent, ahead of the node.
  if !node.edge_label.is_empty() {
    let mut label_attrs: HashMap<String, String> = HashMap::default();
    label_attrs.insert("class".into(), "ltx_forest_edge_label".into());
    document.open_element("ltx:text", Some(label_attrs), None)?;
    for text in &node.edge_label {
      document.absorb(text, None)?;
    }
    document.close_element("ltx:text")?;
  }

  if !node.keys.phantom && (node.label_digested.is_some() || node.label_text.is_some()) {
    let mut text_attrs: HashMap<String, String> = HashMap::default();
    text_attrs.insert("class".into(), "ltx_forest_node_content".into());
    document.open_element("ltx:text", Some(text_attrs), None)?;
    if let Some(ref dig) = node.label_digested {
      document.absorb(dig, None)?;
    } else if let Some(ref text) = node.label_text {
      document.absorb_string(text, &SymHashMap::default())?;
    }
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

/// A comma list as etoolbox's `\forcsvlist` walks it (items trimmed, empty
/// items skipped), forest.sty:164, 171.
fn csv_items(tokens: &Tokens) -> Vec<String> {
  split_unbraced(tokens.unlist_ref(), ',')
    .into_iter()
    .map(|item| tokens_text(&pgfkeys_trim(item)))
    .filter(|item| !item.is_empty())
    .collect()
}

/// Records a loaded forest library where forest's `\ProvidesForestLibrary`
/// records it, `\csdef{forest@libraries@loaded@#1}{}` (forest.sty:173-176),
/// which `\forest@iflibraryloaded` reads (:177).
fn record_forest_library(library: &str) -> Result<()> {
  def_macro(
    T_CS!(&s!("\\forest@libraries@loaded@{library}")),
    None,
    Tokens::new(Vec::new()),
    None,
  )
}

/// `\forestapplylibrarydefaults{<libraries>}` (forest.sty:171-172): runs each
/// library's `libraries/<name>/defaults` style, when defined, through
/// `\forestset`.
fn apply_forest_library_defaults(libraries: &[String]) -> Result<()> {
  for library in libraries {
    // One `\forestset` per library (forest.sty:171-172 `\forcsvlist`), so
    // one budget each.
    let mut ctx = KeyContext::document(&s!("the defaults of forest library '{library}'"));
    let keys = [ForestKey {
      key:   s!("libraries/{library}/defaults/.try"),
      value: None,
    }];
    forestset(&keys, &mut ctx, 0);
    ctx.report_runaway()?;
  }
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  // Per-conversion reset (a mid-body fatal can leave a tree behind).
  PENDING_FOREST_TREES.with(|c| c.borrow_mut().clear());
  LIBRARIES_LATER.with(|c| c.borrow_mut().clear());
  // forest.sty:140-157: a package option that is none of forest's settings
  // (`external`, `tikzcshack`, `tikzinstallkeys`, `compat`, `debug`) names a
  // library, loaded with its defaults at the end of the package
  // (`\usepackage[linguistics]{forest}`, forest-doc.tex:1836). Processed
  // before the requires below, while the options are forest's own.
  DefPrimitive!("\\lx@forest@packageoption{}", sub[(option)] {
    let option = option.to_string();
    let setting = option.split(['=', '/']).next().unwrap_or_default().trim().to_string();
    if !setting.is_empty()
      && !matches!(setting.as_str(), "external" | "tikzcshack" | "tikzinstallkeys" | "compat" | "debug")
    {
      LIBRARIES_LATER.with(|c| c.borrow_mut().push(setting));
    }
    Ok(Vec::new())
  });
  DeclareOption!(None, {
    Digest!("\\expandafter\\lx@forest@packageoption\\expandafter{\\CurrentOption}")?;
  });
  ProcessOptions!();
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
  // forest.sty:8511 defines \NewDocumentCommand{\Forest}{s D(){} m}. The star
  // only drops forest's `\forest@group@env` group (:8513, :8528;
  // forest-doc.tex:1927-1929), which lets the node names and styles a tree
  // defines outlive it; the binding keeps those tree-local for every form
  // (the constructor digests inside a mode group), so the star is read and
  // ignored. We handle optional * and optional (config) before delegating to
  // \lx@forest@exec.
  RawTeX!(
    r"\def\Forest{\@ifstar{\lx@forest@opt*}{\lx@forest@opt{}}}
\def\lx@forest@opt#1{\@ifnextchar({\lx@forest@withconfig{#1}}{\lx@forest@exec{#1}{}}}
\def\lx@forest@withconfig#1(#2){\lx@forest@exec{#1}{#2}}"
  );
  DefConstructor!(
    "\\lx@forest@exec {} {} Undigested",
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
      let config = args[1].as_ref().map(|d| d.to_string()).unwrap_or_default();
      if let Some(Some(body_dig)) = args.get(2) {
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
  // forest.sty:1413-1422 `\bracketset` configures the bracket parser
  // (neoschool.cls:8568, forest-doc.tex:1771, luacas tut3.tex:97
  // `\bracketset{action character=@}`): the opening and closing brackets
  // and the action character, `\let` locally. The action character drives
  // macro expansion while parsing (forest.sty:1491-1492, 1598-1627), which
  // the token-level parser has no stage for; it only marks the trees that
  // use it (`ForestTree::actions`). A bare `action character` (the default,
  // :1422) lets it to `\pgfkeysnovalue`: no action character.
  DefPrimitive!("\\bracketset{}", sub[(keys)] {
    for key in parse_forest_keylist(keys.unlist_ref()) {
      let register = match key.key.as_str() {
        "opening bracket" => OPENING_BRACKET,
        "closing bracket" => CLOSING_BRACKET,
        "action character" => ACTION_CHARACTER,
        _ => continue,
      };
      match key.value.unwrap_or_default().into_iter().find(|t| !is_ignorable_token(t)) {
        Some(token) => assign_value(register, Stored::Token(token), None),
        None if register == ACTION_CHARACTER => assign_value(register, Stored::None, None),
        None => {},
      }
    }
    Ok(Vec::new())
  });
  // forest resumes the bracket parser with `\bracketResume` after a `@@`
  // action handed it to user code (forest.sty:1450, 1622-1623;
  // forest-doc.tex:1787-1795 `\x#1{@@… \expandafter\bracketResume\xtemp}`);
  // a digested label expands into it.
  DefMacro!("\\bracketResume", "\\relax");
  DefPrimitive!("\\forestset{}", sub[(keys)] {
    let mut ctx = KeyContext::document("one \\forestset");
    forestset(&parse_forest_keylist(keys.unlist_ref()), &mut ctx, 0);
    ctx.report_runaway()?;
    Ok(Vec::new())
  });
  DefMacro!("\\forestoption{}", "\\relax");
  DefMacro!("\\foresteoption{}", "\\relax");
  DefMacro!("\\forestregister{}", "\\relax");
  DefMacro!("\\foresteregister{}", "\\relax");
  // forest.sty:162-170 `\useforestlibrary{s O{} m}`. The library name is
  // recorded where forest's `\ProvidesForestLibrary` records it
  // (`\csdef{forest@libraries@loaded@#1}{}`, forest.sty:173-176; read by
  // `\forest@iflibraryloaded`, :177); the library file itself is not loaded
  // (its styles are TikZ layout). The star also applies the defaults.
  DefPrimitive!("\\useforestlibrary OptionalMatch:* [] {}", sub[(star, _options, libraries)] {
    let libraries = csv_items(&libraries);
    for library in &libraries {
      record_forest_library(library)?;
    }
    if star.is_some() {
      apply_forest_library_defaults(&libraries)?;
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\forestapplylibrarydefaults{}", sub[(libraries)] {
    apply_forest_library_defaults(&csv_items(&libraries))?;
    Ok(Vec::new())
  });
  RawTeX!(r"\def\forest@iflibraryloaded#1#2#3{\ifcsdef{forest@libraries@loaded@#1}{#2}{#3}}");
  // forest.sty:160-161 `\AtEndOfPackage{\forest@loadlibrarieslater}`.
  for library in LIBRARIES_LATER.with(|c| std::mem::take(&mut *c.borrow_mut())) {
    record_forest_library(&library)?;
    apply_forest_library_defaults(&[library])?;
  }
});

//! Schema-doc post-processing — visual customizations for `--splitat=section`
//! schema documentation.
//!
//! Three string-level passes run against each split sub-page after the
//! standard LaTeXML XSLT has produced HTML:
//!
//! 1. **Content-model rendering** — pretty-print RelaxNG-style structural
//!    expressions (`A , B | C?`) from one-line walls into operator-leading
//!    multi-line layout. Replaces `tools/render-content-models.py`.
//!
//! 2. **Definition-card decoration** — promote `schema.X` anchor ids onto
//!    parent `<dt>` elements, wrap kind words ("Pattern" / "Element" /
//!    "Attribute" / "Add to") in chip spans, and append `§` permalink
//!    anchors. Replaces `tools/decorate-definitions.py`.
//!
//! 3. **Sidebar item index + module narrative** — collect each page's
//!    Pattern/Element/Attribute definitions and inject a per-module item
//!    index into the navbar, and prepend a curated narrative aside above
//!    the section heading (loaded from a TOML file). Replaces
//!    `tools/inject-module-sidebar.py`.
//!
//! All three passes are idempotent: re-running on already-processed HTML
//! is a no-op. The driver is `process_page`; `load_summaries` reads the
//! per-module narrative TOML once.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use crate::object_db::{ObjectDB, Value};

// ---------- public API ----------------------------------------------------

/// Run all three schema-doc passes on a single page's HTML.
///
/// `module` is the module identifier extracted from the page filename
/// (e.g. `scholarly-ltx-blocks` for `Ch1/schema.scholarly-ltx-blocks.html`);
/// pass `None` for the chapter index page.
///
/// Per-module annotations (the prose paragraph rendered above the
/// definitions) are read from `db` under the `MODULE:<name>` key with
/// field `annotation`. Populate via `load_module_annotations` or by any
/// other path that writes into the same ObjectDB.
pub fn process_page(html: &str, module: Option<&str>, db: &ObjectDB) -> String {
  let html = render_content_models(html);
  let html = decorate_definitions(&html);
  let annotation = module
    .and_then(|m| db.lookup(&format!("MODULE:{}", m)))
    .and_then(|e| e.get_string("annotation"))
    .map(String::from);
  inject_module_sidebar(&html, annotation.as_deref())
}

/// Load per-module annotations from a TOML file in the simple form:
///
/// ```toml
/// [scholarly-ltx-blocks]
/// annotation = """
/// Block-level content — paragraphs, lists, …
/// """
/// ```
///
/// Each entry is registered in `db` as a `MODULE:<name>` key with the
/// `annotation` field set to the trimmed body. Anything more elaborate
/// in the TOML (nested keys, arrays) is silently ignored.
///
/// This is one input path for module annotations; future paths (e.g.
/// RNC `<a:documentation>` annotations extracted by genschema, or
/// inline `\moduleabstract` macros) write into the same ObjectDB key
/// space.
pub fn load_module_annotations(db: &mut ObjectDB, path: &Path) {
  let text = match fs::read_to_string(path) {
    Ok(t) => t,
    Err(_) => return,
  };
  let re = Regex::new(
    r#"(?ms)^\[([^\]\n]+)\][^\[]*?^annotation\s*=\s*"""(.*?)"""\s*$"#,
  )
  .unwrap();
  for cap in re.captures_iter(&text) {
    let key = cap[1].trim().to_string();
    let body = cap[2].trim().to_string();
    if body.is_empty() {
      continue;
    }
    db.register(
      &format!("MODULE:{}", key),
      vec![("annotation", Value::from(body.as_str()))],
    );
  }
}

/// Extract the schema-module identifier from a sub-page path. Used by
/// callers to look up the right TOML entry.
pub fn module_name_for(path: &str) -> Option<String> {
  static RE: OnceLock<Regex> = OnceLock::new();
  let re = RE.get_or_init(|| Regex::new(r"schema\.(scholarly-ltx[\w-]*)\.html$").unwrap());
  re.captures(path).map(|c| c[1].to_string())
}

// ---------- pass 1: content-model rendering -------------------------------

#[derive(Debug)]
enum Tok {
  A(String),       // <a>...</a>
  SpanRef(String), // <span class="ltx_ref ...">...</span> (self-page ref)
  SpanTt(String),  // <span class="ltx_text ltx_font_typewriter">...</span>
  SpanLit(String), // <span class="ltx_text ltx_font_italic">...</span>
  Sup(String),     // <sup class="ltx_sup">[?*+]</sup>
  LParen,
  RParen,
  OpOr,
  OpAnd,
  OpSeq,
}

fn tokenize(s: &str) -> Option<Vec<Tok>> {
  static RE: OnceLock<Regex> = OnceLock::new();
  let re = RE.get_or_init(|| {
    Regex::new(concat!(
      r#"(?P<a><a\s[^>]*>.*?</a>)"#,
      r#"|(?P<spanref><span\s+class="ltx_ref\b[^"]*">.*?</span>)"#,
      r#"|(?P<spantt><span\s+class="ltx_text\s+ltx_font_typewriter">.*?</span>)"#,
      r#"|(?P<spanlit><span\s+class="ltx_text\s+ltx_font_italic">.*?</span>)"#,
      r#"|(?P<sup><sup\s+class="ltx_sup">[?*+]</sup>)"#,
      r"|(?P<lparen>\()",
      r"|(?P<rparen>\))",
      r"|(?P<opor>\s*\|\s*)",
      r"|(?P<opand>\s*(?:&amp;|&)\s*)",
      r"|(?P<opseq>\s*,\s*)",
      r"|(?P<ws>\s+)",
    ))
    .unwrap()
  });

  let mut tokens = Vec::new();
  let mut pos = 0;
  while pos < s.len() {
    let m = re.captures_at(s, pos)?;
    let mat = m.get(0).unwrap();
    if mat.start() != pos {
      return None; // unexpected character — refuse to mangle
    }
    if let Some(t) = m.name("a") {
      tokens.push(Tok::A(t.as_str().to_string()));
    } else if let Some(t) = m.name("spanref") {
      tokens.push(Tok::SpanRef(t.as_str().to_string()));
    } else if let Some(t) = m.name("spantt") {
      tokens.push(Tok::SpanTt(t.as_str().to_string()));
    } else if let Some(t) = m.name("spanlit") {
      tokens.push(Tok::SpanLit(t.as_str().to_string()));
    } else if let Some(t) = m.name("sup") {
      tokens.push(Tok::Sup(t.as_str().to_string()));
    } else if m.name("lparen").is_some() {
      tokens.push(Tok::LParen);
    } else if m.name("rparen").is_some() {
      tokens.push(Tok::RParen);
    } else if m.name("opor").is_some() {
      tokens.push(Tok::OpOr);
    } else if m.name("opand").is_some() {
      tokens.push(Tok::OpAnd);
    } else if m.name("opseq").is_some() {
      tokens.push(Tok::OpSeq);
    } // ws: skip
    pos = mat.end();
  }
  Some(tokens)
}

#[derive(Debug)]
enum Node {
  Atom { html: String, quantifier: String },
  Group { op: Option<&'static str>, items: Vec<Node>, quantifier: String },
}

fn parse(tokens: &[Tok], mut pos: usize) -> (Node, usize) {
  let mut items: Vec<Node> = Vec::new();
  let mut op: Option<&'static str> = None;
  while pos < tokens.len() {
    match &tokens[pos] {
      Tok::RParen => return (Node::Group { op, items, quantifier: String::new() }, pos),
      Tok::LParen => {
        let (inner, np) = parse(tokens, pos + 1);
        pos = np;
        let mut group = inner;
        if pos < tokens.len() && matches!(tokens[pos], Tok::RParen) {
          pos += 1;
        }
        if let (Node::Group { quantifier, .. }, Some(Tok::Sup(s))) =
          (&mut group, tokens.get(pos))
        {
          *quantifier = s.clone();
          pos += 1;
        }
        items.push(group);
      },
      Tok::A(html) | Tok::SpanRef(html) | Tok::SpanTt(html) | Tok::SpanLit(html) => {
        let mut atom = Node::Atom { html: html.clone(), quantifier: String::new() };
        pos += 1;
        if let (Node::Atom { quantifier, .. }, Some(Tok::Sup(s))) = (&mut atom, tokens.get(pos)) {
          *quantifier = s.clone();
          pos += 1;
        }
        items.push(atom);
      },
      Tok::OpOr => {
        if op.is_none() {
          op = Some("OpOr");
        }
        pos += 1;
      },
      Tok::OpAnd => {
        if op.is_none() {
          op = Some("OpAnd");
        }
        pos += 1;
      },
      Tok::OpSeq => {
        if op.is_none() {
          op = Some("OpSeq");
        }
        pos += 1;
      },
      Tok::Sup(_) => {
        pos += 1;
      },
    }
  }
  (Node::Group { op, items, quantifier: String::new() }, pos)
}

fn op_html(op: &str) -> String {
  let (class, glyph) = match op {
    "OpOr" => ("op op-or", "|"),
    "OpAnd" => ("op op-and", "&"),
    "OpSeq" => ("op op-seq", ","),
    _ => ("op", "?"),
  };
  format!(r#"<span class="{}">{}</span>"#, class, glyph)
}

fn is_short(node: &Node) -> bool {
  match node {
    Node::Atom { .. } => true,
    Node::Group { items, .. } => {
      !items.iter().any(|c| matches!(c, Node::Group { .. })) && items.len() <= 4
    },
  }
}

fn render(node: &Node, indent: usize) -> String {
  let pad = "  ".repeat(indent);
  match node {
    Node::Atom { html, quantifier } => format!("{}{}", html, quantifier),
    Node::Group { op, items, quantifier } => {
      if items.is_empty() {
        return String::new();
      }
      if items.len() == 1 && op.is_none() {
        return format!("{}{}", render(&items[0], indent), quantifier);
      }
      if is_short(node) {
        let sep = match op {
          Some(o) => format!(" {} ", op_html(o)),
          None => " ".to_string(),
        };
        let parts: Vec<String> = items.iter().map(|c| render(c, indent)).collect();
        return format!("({}){}", parts.join(&sep), quantifier);
      }
      let inner_pad = "  ".repeat(indent + 1);
      let op_seg = op.map(op_html).unwrap_or_default();
      let mut lines = vec![String::from("(")];
      for (i, c) in items.iter().enumerate() {
        let prefix = if i == 0 { String::from("  ") } else { format!("{} ", op_seg) };
        lines.push(format!("{}{}{}", inner_pad, prefix, render(c, indent + 1)));
      }
      lines.push(format!("{}){}", pad, quantifier));
      lines.join("\n")
    },
  }
}

fn render_content_models(html: &str) -> String {
  if html.contains(r#"class="schema-content-model""#) {
    return html.to_string();
  }
  static RE: OnceLock<Regex> = OnceLock::new();
  let re = RE.get_or_init(|| Regex::new(r#"(?s)<p class="ltx_p">(\s*\(.+?)</p>"#).unwrap());
  re.replace_all(html, |caps: &regex::Captures| {
    let inner = caps[1].trim();
    let Some(tokens) = tokenize(inner) else {
      return caps[0].to_string();
    };
    if !matches!(tokens.first(), Some(Tok::LParen)) {
      return caps[0].to_string();
    }
    let (mut ast, mut pos) = parse(&tokens, 1);
    if !matches!(tokens.get(pos), Some(Tok::RParen)) {
      return caps[0].to_string();
    }
    pos += 1;
    if let Some(Tok::Sup(s)) = tokens.get(pos) {
      if let Node::Group { quantifier, .. } = &mut ast {
        *quantifier = s.clone();
      }
      pos += 1;
    }
    if pos != tokens.len() {
      return caps[0].to_string();
    }
    let body = render(&ast, 0);
    format!(r#"<p class="ltx_p"><code class="schema-content-model">{}</code></p>"#, body)
  })
  .into_owned()
}

// ---------- pass 2: definition cards --------------------------------------

fn decorate_definitions(html: &str) -> String {
  if html.contains("schema-kind-chip") {
    return html.to_string();
  }
  static DT_RE: OnceLock<Regex> = OnceLock::new();
  static ANCHOR_RE: OnceLock<Regex> = OnceLock::new();

  let dt_re = DT_RE.get_or_init(|| {
    Regex::new(concat!(
      r#"(?s)<dt id="(I\d+\.ix\d+)" class="ltx_item">"#,
      r#"<span class="ltx_tag ltx_tag_item">"#,
      r#"<span class="ltx_text ltx_font_bold ltx_font_italic">"#,
      r"([A-Za-z]+(?:\s+[A-Za-z]+)?)\s+",
      r#"<span class="ltx_text ltx_font_sansserif[^"]*">"#,
      "([^<]+)</span>",
      r"</span></span></dt>",
    ))
    .unwrap()
  });
  // Rust's `regex` crate is RE2-based and doesn't support backreferences,
  // so we can't anchor `id` to equal the captured `name`. Match both
  // independently (LaTeXML emits them identical) and accept the small
  // theoretical risk of a mismatched pair.
  let anchor_re = ANCHOR_RE.get_or_init(|| {
    Regex::new(
      r#"<a name="(schema\.[^"]+)" id="schema\.[^"]+" class="ltx_anchor">"#,
    )
    .unwrap()
  });

  let kind_class = |kind: &str| -> Option<&'static str> {
    match kind {
      "Pattern" => Some("kind-pattern"),
      "Element" => Some("kind-element"),
      "Attribute" => Some("kind-attribute"),
      "Add to" => Some("kind-pattern-add"),
      _ => None,
    }
  };

  let dts: Vec<regex::Captures<'_>> = dt_re.captures_iter(html).collect();
  if dts.is_empty() {
    return html.to_string();
  }
  let anchors: Vec<regex::Match<'_>> = anchor_re.find_iter(html).collect();

  // (start, end, replacement) tuples, applied in reverse so positions stay valid.
  let mut rewrites: Vec<(usize, usize, String)> = Vec::new();

  for (i, dt) in dts.iter().enumerate() {
    let dt_match = dt.get(0).unwrap();
    let next_pos = dts.get(i + 1).map(|n| n.get(0).unwrap().start()).unwrap_or(html.len());
    let old_id = &dt[1];
    let kind = &dt[2];
    let name = &dt[3];
    let Some(class) = kind_class(kind) else {
      continue;
    };

    let matching = anchors
      .iter()
      .find(|a| a.start() >= dt_match.end() && a.start() < next_pos);

    let new_id = matching
      .and_then(|a| anchor_re.captures(a.as_str()).map(|c| c[1].to_string()))
      .unwrap_or_else(|| old_id.to_string());

    let new_dt = format!(
      concat!(
        r##"<dt id="{id}" class="ltx_item schema-def">"##,
        r##"<span class="ltx_tag ltx_tag_item">"##,
        r##"<span class="schema-kind-chip {class}">{kind}</span>"##,
        r##"<span class="schema-name">{name}</span>"##,
        r##"<a class="schema-permalink" href="#{id}" "##,
        r##"aria-label="permalink to this definition">§</a>"##,
        r"</span></dt>",
      ),
      id = new_id,
      class = class,
      kind = kind,
      name = name,
    );
    rewrites.push((dt_match.start(), dt_match.end(), new_dt));

    if let Some(a) = matching {
      static STRIP_ID_RE: OnceLock<Regex> = OnceLock::new();
      let strip_id_re = STRIP_ID_RE
        .get_or_init(|| Regex::new(r#" id="schema\.[^"]+""#).unwrap());
      let stripped = strip_id_re.replace(a.as_str(), "").into_owned();
      rewrites.push((a.start(), a.end(), stripped));
    }
  }

  rewrites.sort_by_key(|(s, _, _)| std::cmp::Reverse(*s));
  let mut out = html.to_string();
  for (s, e, replacement) in rewrites {
    out.replace_range(s..e, &replacement);
  }
  out
}

// ---------- pass 3: sidebar item index + module narrative -----------------

fn inject_module_sidebar(html: &str, summary: Option<&str>) -> String {
  let html = inject_sidebar_index(html);
  if let Some(s) = summary {
    inject_module_narrative(&html, s)
  } else {
    html
  }
}

fn inject_sidebar_index(html: &str) -> String {
  if html.contains(r#"class="schema_module_index""#) {
    return html.to_string();
  }
  static ITEM_RE: OnceLock<Regex> = OnceLock::new();
  static NAVBAR_RE: OnceLock<Regex> = OnceLock::new();

  let item_re = ITEM_RE.get_or_init(|| {
    Regex::new(concat!(
      r#"<dt id="([^"]+)" class="ltx_item schema-def">"#,
      r#"<span class="ltx_tag ltx_tag_item">"#,
      r#"<span class="schema-kind-chip kind-([a-z-]+)">([^<]+)</span>"#,
      r#"<span class="schema-name">([^<]+)</span>"#,
    ))
    .unwrap()
  });
  let navbar_re = NAVBAR_RE.get_or_init(|| {
    Regex::new(concat!(
      r#"(?s)(<nav class="ltx_page_navbar">"#,
      r#"(?:[^<]*<a [^>]+rel="start"[^>]*>.*?</a>)?\s*)"#,
      r#"(<nav class="ltx_TOC">)"#,
    ))
    .unwrap()
  });

  let mut seen: HashSet<(String, String)> = HashSet::new();
  // Insertion-ordered groups keyed by kind word.
  let mut by_kind: HashMap<&str, Vec<(String, String)>> = HashMap::new();
  let kinds_order = ["Pattern", "Element", "Attribute", "Add to"];
  let kinds_plural: HashMap<&str, &str> = [
    ("Pattern", "Patterns"),
    ("Element", "Elements"),
    ("Attribute", "Attributes"),
    ("Add to", "Pattern Additions"),
  ]
  .iter()
  .copied()
  .collect();

  for cap in item_re.captures_iter(html) {
    let dt_id = cap[1].to_string();
    let kind = cap[3].to_string();
    let name = cap[4].to_string();
    if !kinds_plural.contains_key(kind.as_str()) {
      continue;
    }
    if !seen.insert((kind.clone(), name.clone())) {
      continue;
    }
    let bucket: &str = kinds_order.iter().find(|k| **k == kind.as_str()).copied().unwrap();
    by_kind.entry(bucket).or_default().push((name, dt_id));
  }

  if by_kind.is_empty() {
    return html.to_string();
  }

  let mut fragment = String::from(r#"<section class="schema_module_index">"#);
  for kind in kinds_order {
    let Some(entries) = by_kind.get_mut(kind) else { continue };
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    fragment.push_str(&format!(
      r#"<h6 class="schema_index_heading">{}</h6>"#,
      html_escape(kinds_plural[kind])
    ));
    fragment.push_str(r#"<ul class="schema_index_list">"#);
    for (name, dt_id) in entries.iter() {
      fragment.push_str(&format!(
        r##"<li><a href="#{}">{}</a></li>"##,
        html_escape(dt_id),
        html_escape(name),
      ));
    }
    fragment.push_str("</ul>");
  }
  fragment.push_str("</section>");

  let in_schema = r#"<h6 class="schema_in_schema">In schema</h6>"#;

  let result = navbar_re.replace(html, |caps: &regex::Captures| {
    format!("{}{}{}{}", &caps[1], fragment, in_schema, &caps[2])
  });
  result.into_owned()
}

fn inject_module_narrative(html: &str, summary: &str) -> String {
  if html.contains(r#"class="schema_module_narrative""#) {
    return html.to_string();
  }
  static HEADING_RE: OnceLock<Regex> = OnceLock::new();
  let heading_re =
    HEADING_RE.get_or_init(|| Regex::new(r#"(?s)(<h1 class="ltx_title ltx_title_section">.*?</h1>)"#).unwrap());
  let narrative = format!(
    r#"<aside class="schema_module_narrative"><p>{}</p></aside>"#,
    summary
  );
  let result = heading_re.replace(html, |caps: &regex::Captures| {
    format!("{}\n{}", &caps[1], narrative)
  });
  result.into_owned()
}

// ---------- helpers -------------------------------------------------------

fn html_escape(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
}

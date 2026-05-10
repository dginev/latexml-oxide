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
use std::sync::OnceLock;

/// Filename of the rustdoc-styled theme stylesheet that ships next
/// to each schema-doc site. Auto-prepended to `--css` when
/// `--schemadocs` is on, so callers don't need to remember it.
/// The orchestration shell copies the source CSS into `$output_dir`
/// under this same basename so the relative `<link>` resolves.
pub const THEME_CSS_BASENAME: &str = "relaxng-schema-rustdoc-theme.css";

/// Filename of the runtime script (theme-boot, popover wiring,
/// in-page filter). Auto-prepended to `--javascript` when
/// `--schemadocs` is on. The orchestration shell copies the source
/// JS into `$output_dir` under the same basename so the relative
/// `<script src>` resolves.
pub const THEME_JS_BASENAME: &str = "relaxng-schema-rustdoc-theme.js";

// ---------- public API ----------------------------------------------------

/// Run the schema-doc passes on a single page's HTML.
///
/// Layout: defs are description-list items inside the per-module
/// section page (`--splitat=section`). No kind-bucket subsections;
/// Patterns and Elements interleave in source order so cross-refs
/// between them stay on one page. Long pages get a JS-driven filter
/// input (browser Ctrl-F still works since items default to visible).
pub fn process_page(html: &str) -> String {
  let html = lift_module_narrative(html);
  let html = render_content_models(&html);
  let html = decorate_definitions(&html);
  let html = inject_sidebar_index(&html);
  let html = inject_theme_switcher(&html);
  inject_experimental_banner(&html)
}

/// Inject the rustdoc-styled Settings popover *markup* — only the
/// HTML widget that exposes Theme (Light / Dark / Ayu / System) and
/// the Hide-sidebar toggle. **No `<script>` element is injected from
/// here**; the runtime
/// (`resources/javascript/relaxng-schema-rustdoc-theme.js`) is
/// pulled in via the standard `--javascript=…` flag of `latexml_oxide`,
/// which the orchestration shell (`tools/generate-scholarly-schema-docs`)
/// passes alongside `--css=…`. The XSLT then emits a non-deferred
/// `<script src>` in `<head>` for us — same code path the CSS
/// `<link>` uses — so `applyTheme()` runs synchronously before paint.
///
/// The runtime handles three pieces of behaviour, all on the
/// pre-existing widget markup this function injects:
///
/// 1. Pre-paint application of `data-theme` / `data-pref-*` from
///    `localStorage`.
/// 2. Settings popover wiring (radios + checkboxes + click-outside +
///    system colour-scheme listener) on `DOMContentLoaded`.
/// 3. The in-page schema-def filter (sticky search above long def
///    lists) — replaces the prior `inject_filter_script` pass.
///
/// Settings widget shape:
///
/// * Theme fieldset — 4 radios (Light / Dark / Ayu / System), keyed
///   to `localStorage["schema-theme"]`.
/// * Display fieldset — 1 checkbox:
///
///   | localStorage key   | `<html>` attribute  | CSS effect |
///   |--------------------|---------------------|------------|
///   | `schema-pref-sidebar` | `data-pref-sidebar="on"` | hide `nav.ltx_page_navbar` |
///
///   (Sans-serif font swap and content-model wrap were tried but
///   removed — neither was easy to use, and the results were
///   marginal compared to the existing layout.)
///
/// * Tasteful credit line linking to the rustdoc reference docs.
///
/// Other rustdoc settings (auto-hide methods, search single-result
/// jump, line numbers on examples, deprecation, keyboard shortcuts)
/// are out of scope — they don't apply to schema docs.
fn inject_theme_switcher(html: &str) -> String {
  if html.contains("data-schema-theme-widget") {
    return html.to_string();
  }
  static BODY_OPEN_RE: OnceLock<Regex> = OnceLock::new();
  let body_open_re = BODY_OPEN_RE.get_or_init(|| Regex::new(r"(?i)<body[^>]*>").unwrap());

  // Settings widget — markup only; behaviour wired up by the JS
  // runtime fetched via `--javascript=relaxng-schema-rustdoc-theme.js`.
  let widget = r##"<details class="schema-theme-switcher" data-schema-theme-widget>
<summary aria-label="Settings" title="Settings"><span class="schema-gear" aria-hidden="true">⚙</span></summary>
<div class="schema-theme-popover" role="dialog" aria-label="Settings">
<fieldset>
<legend>Theme</legend>
<label><input type="radio" name="schema-theme-radio" value="light"> Light</label>
<label><input type="radio" name="schema-theme-radio" value="dark"> Dark</label>
<label><input type="radio" name="schema-theme-radio" value="ayu"> Ayu</label>
<label><input type="radio" name="schema-theme-radio" value="system"> System</label>
</fieldset>
<fieldset class="schema-pref-block">
<legend>Display</legend>
<label><input type="checkbox" data-schema-pref="sidebar"> Hide sidebar</label>
</fieldset>
<p class="schema-theme-credit">Theme inspired by <a href="https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html" rel="noopener">rustdoc</a>.</p>
</div>
</details>"##;

  body_open_re
    .replace(html, |caps: &regex::Captures| {
      format!("{}{}", caps.get(0).unwrap().as_str(), widget)
    })
    .into_owned()
}

/// Inject a thin "Experimental Draft" ribbon along the right edge of
/// every schema-doc page, vertically centered. Mirrors the spirit of
/// the "W3C Editor's Draft" banner on W3C Working Draft pages —
/// signals that the docs are not yet a stable / canonical reference.
/// Themed: the ribbon background uses `var(--banner-bg)` /
/// `var(--banner-fg)` tokens (defined per palette in the theme CSS).
/// Pure CSS positioning; `pointer-events: none` so the banner doesn't
/// block clicks on overlapping content.
fn inject_experimental_banner(html: &str) -> String {
  if html.contains("data-schema-experimental-banner") {
    return html.to_string();
  }
  static BODY_OPEN_RE: OnceLock<Regex> = OnceLock::new();
  let body_open_re = BODY_OPEN_RE.get_or_init(|| Regex::new(r"(?i)<body[^>]*>").unwrap());
  let banner = r##"<aside class="schema-experimental-banner" data-schema-experimental-banner aria-label="Experimental Draft notice">Experimental Draft</aside>"##;
  body_open_re
    .replace(html, |caps: &regex::Captures| {
      format!("{}{}", caps.get(0).unwrap().as_str(), banner)
    })
    .into_owned()
}


/// `\moduleabstract` produces `<ltx:para class="schema_module_narrative">`
/// which LaTeXML's HTML output renders as a marked
/// `<div class="ltx_para schema_module_narrative">` for the first
/// paragraph, *plus* one unmarked `<div class="ltx_para">` per
/// subsequent paragraph (the marker class doesn't survive across
/// `\par` breaks inside the macro arg). Trang emits one
/// `<a:documentation>` per `## comment` block in the source RNC,
/// and our `extract_docs` joins them with blank lines, so a module
/// with multiple `## comment` paragraphs lands as multiple `<p>`s
/// in this run.
///
/// The post-pass walks the marked div *and every immediately-
/// following `<div class="ltx_para">` sibling* up to the next
/// non-paragraph element (typically the description-list opener),
/// then folds the whole run into one left-bordered
/// `<aside class="schema_module_narrative">` block right after the
/// section heading. Each source paragraph stays in its own `<p>`
/// inside the aside.
fn lift_module_narrative(html: &str) -> String {
  if html.contains(r#"<aside class="schema_module_narrative">"#) {
    return html.to_string();
  }
  static NARRATIVE_OPEN_RE: OnceLock<Regex> = OnceLock::new();
  static EXTRA_PARA_RE: OnceLock<Regex> = OnceLock::new();
  static P_RE: OnceLock<Regex> = OnceLock::new();
  static HEADING_RE: OnceLock<Regex> = OnceLock::new();

  // Step 1: locate the first `schema_module_narrative` div, capturing
  // the whole `<div …>…</div>` (class order is liberal — either
  // `ltx_para` or `schema_module_narrative` may come first).
  let narrative_open_re = NARRATIVE_OPEN_RE.get_or_init(|| {
    Regex::new(
      r#"(?s)<div [^>]*class="[^"]*schema_module_narrative[^"]*"[^>]*>.*?</div>"#,
    )
    .unwrap()
  });
  // Step 2: anchor-at-start regex that matches whitespace + one
  // additional `<div class="…schema_module_narrative…">…</div>`
  // sibling — `genschema_oxide`'s lift emits one `\moduleabstract`
  // per source paragraph so every paragraph carries the marker
  // class. Walking ONLY marked siblings (not generic `ltx_para`
  // divs) avoids accidentally consuming the wrapper around the
  // description list (which is `<div class="ltx_para">` itself in
  // some splits).
  let extra_para_re = EXTRA_PARA_RE.get_or_init(|| {
    Regex::new(
      r#"(?s)\A\s*<div [^>]*class="[^"]*schema_module_narrative[^"]*"[^>]*>.*?</div>"#,
    )
    .unwrap()
  });
  // Step 3: pull every `<p class="ltx_p">…</p>` out of the combined
  // run — they're the paragraphs to splice into the aside.
  let p_re = P_RE.get_or_init(|| {
    Regex::new(r#"(?s)<p class="ltx_p[^"]*">.*?</p>"#).unwrap()
  });
  let heading_re = HEADING_RE.get_or_init(|| {
    Regex::new(r#"(?s)(<h1 class="ltx_title ltx_title_section">.*?</h1>)"#).unwrap()
  });

  let first = match narrative_open_re.find(html) {
    Some(m) => m,
    None => return html.to_string(),
  };
  // Extend the match through any contiguous trailing
  // `<div class="ltx_para">` siblings.
  let mut end = first.end();
  while end < html.len() {
    let rest = &html[end..];
    match extra_para_re.find(rest) {
      Some(m) => end += m.end(),
      None => break,
    }
  }
  let block = &html[first.start()..end];
  let paragraphs: Vec<&str> = p_re.find_iter(block).map(|m| m.as_str()).collect();
  let inner = paragraphs.join("\n");
  let aside = format!(
    r#"<aside class="schema_module_narrative">{}</aside>"#,
    inner
  );

  // Strip the original block, then insert the aside right after
  // the section heading.
  let mut stripped = String::with_capacity(html.len());
  stripped.push_str(&html[..first.start()]);
  stripped.push_str(&html[end..]);
  let result = heading_re.replace(&stripped, |caps: &regex::Captures| {
    format!("{}\n{}", &caps[1], aside)
  });
  result.into_owned()
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

/// Decorate each `<dt>` definition heading: promote the
/// `\hypertarget{schema.<name>}` anchor onto the `<dt>` element,
/// wrap the kind word in a chip span, and append a `§` permalink.
///
/// With defs as description-list items (matching upstream Perl), each
/// `\elementdef` / `\patterndef` / etc. renders as
///
/// ```html
/// <dt id="I1.ix1" class="ltx_item">
///   <span class="ltx_tag ltx_tag_item">
///     <span class="ltx_text ltx_font_bold ltx_font_italic">Element </span>
///     <span class="ltx_text ltx_font_sansserif ltx_font_bold">name</span>
///   </span>
/// </dt>
/// <dd class="ltx_item">
///   <p class="ltx_p"><a name="schema.X" id="schema.X" class="ltx_anchor">…doc…</a></p>
///   …
/// </dd>
/// ```
///
/// We rewrite the `<dt>` to carry `id="schema.X"` plus a chip + name
/// + § permalink, and strip the redundant `id=` from the inner anchor.
fn decorate_definitions(html: &str) -> String {
  if html.contains("schema-kind-chip") {
    return html.to_string();
  }
  static DT_RE: OnceLock<Regex> = OnceLock::new();
  static ANCHOR_RE: OnceLock<Regex> = OnceLock::new();

  // Match `<dt class="ltx_item">` with the kicker structure
  // `<bold-italic>KIND <sansserif>NAME</></></dt>` at any nesting depth
  // — top-level (e.g. `id="I1.ix3"`) and nested (e.g.
  // `id="I1.ix3.I3.ix2"`) both qualify. Nested matches are how a
  // pattern like `ltx.span.elem = element span {...}` exposes its
  // inner `\elementdef{xhtml:span}` so that `\elementref{xhtml:span}`
  // cross-refs resolve. Duplicate-id collisions across multiple
  // nested defs of the same name are guarded below by `seen_ids`.
  let dt_re = DT_RE.get_or_init(|| {
    Regex::new(concat!(
      r#"(?s)<dt id="([^"]+)" class="ltx_item">"#,
      r#"<span class="ltx_tag ltx_tag_item">"#,
      r#"<span class="ltx_text ltx_font_bold ltx_font_italic">"#,
      r"([A-Za-z]+(?:\s+[A-Za-z]+)?)\s+",
      r#"<span class="ltx_text ltx_font_sansserif[^"]*">"#,
      "([^<]+)</span>",
      r"</span></span></dt>",
    ))
    .unwrap()
  });
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

  let mut rewrites: Vec<(usize, usize, String)> = Vec::new();
  // The same xhtml:NAME often appears as a nested elementdef under
  // several wrapping pattern defs on one page (e.g. `xhtml:div`
  // appears 5x in scaffold.html as the body of distinct ltx.*.elem
  // patterns). We can only assign `id="schema.xhtml..div"` to one of
  // them; the rest stay as ordinary nested kicker rows. Pick the
  // first occurrence — that's what LaTeXML's `\hypertarget` would
  // also pick.
  let mut seen_ids: HashSet<String> = HashSet::new();
  // Skip nested attribute promotion: attribute names (`dir`, `class`,
  // `id`, …) routinely repeat across patterns and would collide.
  // Top-level attribute defs don't exist in this schema flavour.
  let promotable = |kind: &str, depth: usize| -> bool {
    if depth == 0 {
      return true; // top-level: always (Pattern/Element/Attribute/Add to).
    }
    matches!(kind, "Pattern" | "Element")
  };

  for (i, dt) in dts.iter().enumerate() {
    let dt_match = dt.get(0).unwrap();
    let next_pos = dts.get(i + 1).map(|n| n.get(0).unwrap().start()).unwrap_or(html.len());
    let raw_id = &dt[1];
    let kind = &dt[2];
    let name = &dt[3];
    let Some(class) = kind_class(kind) else {
      continue;
    };
    // `I1.ix3` is depth-0 (top-level dl), `I1.ix3.I3.ix2` is depth-1
    // (nested dl), etc. Each `.I\d+.ix\d+` pair past the first counts
    // one level of nesting.
    let depth = raw_id.matches(".ix").count().saturating_sub(1);
    if !promotable(kind, depth) {
      continue;
    }

    // Derive the def's anchor id from kind + name. Doing this from
    // the heading text (rather than searching for a sibling `<a
    // name="schema.X">`) is robust to empty-doc defs — when the
    // doc-arg is empty, `\hypertarget{schema.X}{}` produces no anchor
    // element in the HTML, but the `<dt>` itself still needs the id
    // so cross-page links to `#schema.X` resolve here. Patternadds
    // get a separate `schema.add.<name>` so they don't clash with
    // the canonical def's `schema.<name>`.
    //
    // Pass the name through `clean_anchor_name` so the id matches the
    // hrefs that LaTeXML's `\hyperlink{\cleanhypername{schema.X}}`
    // emits in `\elementref` / `\patternref` body text — `:` becomes
    // `..`, otherwise xhtml:foo dt ids never resolve their cross-refs.
    let cleaned_name = clean_anchor_name(name);
    let new_id = if kind == "Add to" {
      format!("schema.add.{}", cleaned_name)
    } else {
      format!("schema.{}", cleaned_name)
    };
    if !seen_ids.insert(new_id.clone()) {
      // Another dt already claimed this id on this page — leave the
      // duplicate as a plain kicker row.
      continue;
    }

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

    // If a sibling `<a name="schema.X" id="schema.X">` exists (the
    // \hypertarget rendering when doc was non-empty), strip its
    // duplicate `id=` so the page doesn't carry two elements with
    // the same id. Keep the `name=` so legacy `#name` URLs still
    // resolve to the inner anchor's position too.
    let matching = anchors
      .iter()
      .find(|a| a.start() >= dt_match.end() && a.start() < next_pos);
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

// ---------- pass 3: sidebar item index ------------------------------------

/// Collect every decorated `schema-def` on the page, group by kind,
/// and inject a per-page kind index at the top of the navbar (above
/// the cross-page `<nav class="ltx_TOC">` module list).
fn inject_sidebar_index(html: &str) -> String {
  if html.contains(r#"class="schema_module_index""#) {
    return html.to_string();
  }
  static ITEM_RE: OnceLock<Regex> = OnceLock::new();
  static NAVBAR_RE: OnceLock<Regex> = OnceLock::new();

  // Matches the post-decorate `<dt class="schema-def">` heading
  // (chip + name + permalink). Description-list shape, mirroring
  // upstream Perl `latexmlman.sty`.
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
  // Top-level navbar buckets, in order:
  //   Patterns are SUBDIVIDED by their last dot-suffix
  //   ("PATTERNS — ELEM", "PATTERNS — ATTRS", ...) so a long flat list
  //   becomes a structured outline. Patterns whose name has no dot
  //   land in the catch-all "PATTERNS — OTHER".
  // Elements / Attribute / Add to render as single buckets.
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

  // Insertion-ordered subgroups: for "Pattern", key = suffix. For
  // every other kind, key = "" (single bucket).
  let mut by_kind: HashMap<&str, Vec<(String, Vec<(String, String)>)>> = HashMap::new();

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
    let subkey = if bucket == "Pattern" {
      pattern_suffix(&name).unwrap_or("Other").to_string()
    } else {
      String::new()
    };
    let kind_subgroups = by_kind.entry(bucket).or_default();
    let pos = kind_subgroups.iter().position(|(k, _)| k == &subkey);
    let entries = match pos {
      Some(idx) => &mut kind_subgroups[idx].1,
      None => {
        kind_subgroups.push((subkey, Vec::new()));
        &mut kind_subgroups.last_mut().unwrap().1
      },
    };
    entries.push((name, dt_id));
  }

  if by_kind.is_empty() {
    return html.to_string();
  }

  let mut fragment = String::from(r#"<section class="schema_module_index">"#);
  for kind in kinds_order {
    let Some(subgroups) = by_kind.get_mut(kind) else { continue };
    // Sort subgroups for Patterns alphabetically by suffix, with
    // "Other" last; non-Pattern kinds keep insertion order (single
    // empty-key entry).
    if kind == "Pattern" {
      subgroups.sort_by(|a, b| match (a.0.as_str(), b.0.as_str()) {
        ("Other", _) => std::cmp::Ordering::Greater,
        (_, "Other") => std::cmp::Ordering::Less,
        (x, y) => x.cmp(y),
      });
    }
    for (suffix, entries) in subgroups {
      entries.sort_by(|a, b| a.0.cmp(&b.0));
      let heading = if kind == "Pattern" {
        format!("{} — {}", kinds_plural[kind], suffix.to_uppercase())
      } else {
        kinds_plural[kind].to_string()
      };
      fragment.push_str(&format!(
        r#"<h6 class="schema_index_heading">{}</h6>"#,
        html_escape(&heading)
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
  }
  fragment.push_str("</section>");

  let in_schema = r#"<h6 class="schema_in_schema">In schema</h6>"#;

  let result = navbar_re.replace(html, |caps: &regex::Captures| {
    format!("{}{}{}{}", &caps[1], fragment, in_schema, &caps[2])
  });
  result.into_owned()
}

/// Extract a pattern's "suffix" — the last dot-separated segment of
/// its name. Returns None when the name has no dot (sidebar bucket
/// then folds it into "Other"). Used to subdivide the PATTERNS bucket
/// in the navbar so a long flat list becomes
/// "PATTERNS — ELEM" / "PATTERNS — ATTRS" / etc.
fn pattern_suffix(name: &str) -> Option<&str> {
  // Skip obvious namespace-prefix names (we shouldn't see them in
  // the Pattern bucket, but be defensive).
  let after_colon = name.rsplit_once(':').map(|(_, t)| t).unwrap_or(name);
  after_colon.rsplit_once('.').map(|(_, suffix)| suffix)
}

// ---------- helpers -------------------------------------------------------

/// Mirror `latexmlman.sty`'s `\cleanhypername` macro for HTML anchor
/// ids. The macro splits on `:` and rejoins with `..` (because `:`
/// inside a TeX `\hypertarget` argument is brittle), so a raw name
/// like `xhtml:header` ends up in HTML hrefs as `xhtml..header`. The
/// schema-doc decorator builds dt ids from raw names, so without this
/// transform every `\elementref{xhtml:foo}` / `\patternref{xhtml:foo}`
/// produced by LaTeXML lands on a non-existent `#schema.xhtml..foo`
/// while the dt sits at `#schema.xhtml:foo`. Underscores survive as
/// `_` in both forms; only `:` needs the substitution.
fn clean_anchor_name(name: &str) -> String {
  name.replace(':', "..")
}

fn html_escape(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clean_anchor_name_replaces_colon_with_double_dot() {
    assert_eq!(clean_anchor_name("xhtml:header"), "xhtml..header");
    assert_eq!(clean_anchor_name("m:annotation-xml"), "m..annotation-xml");
    // No colons → unchanged.
    assert_eq!(clean_anchor_name("ltx.span.elem"), "ltx.span.elem");
    // Multiple colons all flip.
    assert_eq!(clean_anchor_name("a:b:c"), "a..b..c");
    // Underscores survive — only `:` is cleaned.
    assert_eq!(clean_anchor_name("foo_bar"), "foo_bar");
  }

  #[test]
  fn nested_element_dt_is_promoted_to_schema_def() {
    // Synthesise a tiny page with a nested `\elementdef{xhtml:span}`-
    // shaped dt sitting inside a parent `\patterndef{ltx.span.elem}`-
    // shaped dt. The post-pass should give *both* a `schema.X` id so
    // cross-refs to either resolve.
    let html = r##"<dl class="ltx_description">
<dt id="I1.ix3" class="ltx_item"><span class="ltx_tag ltx_tag_item"><span class="ltx_text ltx_font_bold ltx_font_italic">Pattern <span class="ltx_text ltx_font_sansserif ltx_font_bold">ltx.span.elem</span></span></span></dt>
<dd class="ltx_item"><dl class="ltx_description">
<dt id="I1.ix3.I3.ix2" class="ltx_item"><span class="ltx_tag ltx_tag_item"><span class="ltx_text ltx_font_bold ltx_font_italic">Element <span class="ltx_text ltx_font_sansserif ltx_font_upright">xhtml:span</span></span></span></dt>
</dl></dd>
</dl>"##;
    let out = decorate_definitions(html);
    assert!(
      out.contains(r#"id="schema.ltx.span.elem""#),
      "top-level pattern dt should get schema.ltx.span.elem:\n{}",
      out
    );
    assert!(
      out.contains(r#"id="schema.xhtml..span""#),
      "nested element dt should be promoted with cleaned name:\n{}",
      out
    );
  }

  #[test]
  fn duplicate_nested_name_keeps_only_first_id() {
    // `xhtml:div` defined twice as nested elementdef — second occurrence
    // must NOT claim the same id (would be invalid HTML).
    let html = r##"<dl class="ltx_description">
<dt id="I1.ix1.I1.ix2" class="ltx_item"><span class="ltx_tag ltx_tag_item"><span class="ltx_text ltx_font_bold ltx_font_italic">Element <span class="ltx_text ltx_font_sansserif ltx_font_upright">xhtml:div</span></span></span></dt>
<dt id="I1.ix2.I2.ix2" class="ltx_item"><span class="ltx_tag ltx_tag_item"><span class="ltx_text ltx_font_bold ltx_font_italic">Element <span class="ltx_text ltx_font_sansserif ltx_font_upright">xhtml:div</span></span></span></dt>
</dl>"##;
    let out = decorate_definitions(html);
    let count = out.matches(r#"id="schema.xhtml..div""#).count();
    assert_eq!(count, 1, "only the first nested dt should claim the id:\n{}", out);
  }
}

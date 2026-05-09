//! Schema-doc TeX emission. Port of `RelaxNG.pm` lines 545–815.
//!
//! Walks the simplified AST (in [`Relaxng::modules`] and the lookup
//! tables populated by [`super::simplify`]) and produces a single
//! `schema.tex` string consumable by `latexmlman.sty`'s `\schemamodule`
//! / `\patterndef` / `\elementdef` / `\attrdef` / `\moduleref` /
//! `\patternref` / `\elementref` macros.
//!
//! Behaviour intentionally byte-equivalent (modulo whitespace settled
//! at the unit-test level) with the Perl `documentModules` for the
//! same simplified state. The schema-doc-style omissions
//! (`SKIP_SVG`/`SKIP_ARIA`/`SKIP_XHTML`) live on [`Options`] with the
//! same defaults as upstream.

use rustc_hash::FxHashMap as HashMap;

use super::{CombineOp, DefCombiner, Pattern, Relaxng};

/// Schema-doc emission options. Defaults match Perl's
/// `$SKIP_SVG=1;$SKIP_ARIA=1;$SKIP_XHTML=1` constants.
#[derive(Debug, Clone, Copy)]
pub struct Options {
  pub skip_svg:   bool,
  pub skip_aria:  bool,
  pub skip_xhtml: bool,
}

impl Default for Options {
  fn default() -> Self {
    Options { skip_svg: true, skip_aria: true, skip_xhtml: true }
  }
}

/// Emission state — mutated as `document_modules` walks the AST.
struct EmitState<'a> {
  rng:               &'a Relaxng,
  opts:              Options,
  /// Mirrors Perl's `$$self{defined_patterns}{$name}`.
  /// `1`  = at least one `\patterndef{name}` already emitted,
  /// `-1` = at least one `\patternadd{name}` emitted but no `\patterndef` yet.
  /// Final pass upgrades `\patternadd` → `\patterndefadd` for any -1.
  defined_patterns: HashMap<String, i8>,
}

/// Top-level emission. Returns a single `schema.tex` string.
pub fn document_modules(rng: &Relaxng, opts: Options) -> String {
  let mut emit = EmitState { rng, opts, defined_patterns: HashMap::default() };
  let mut docs = String::new();
  for module in &rng.modules {
    let (op, name, content) = match module {
      Pattern::Module { name, body } => ("module", name.clone(), body),
      _ => continue,
    };
    let _ = op;
    if emit.opts.skip_svg && is_svg_module(&name) {
      continue;
    }
    let mod_name = strip_urn_prefix(&name);
    docs.push_str(&format!("\n\\begin{{schemamodule}}{{{}}}", mod_name));
    for item in content {
      docs.push('\n');
      docs.push_str(&emit.to_tex(item));
    }
    docs.push_str("\n\\end{schemamodule}");
  }
  // Final pass: any pattern emitted only as `\patternadd` becomes
  // `\patterndefadd`. Mirrors Perl `$docs =~ s/\\patternadd\{$name\}/\\patterndefadd{$name}/s`
  // — single substitution per name.
  let mut keys: Vec<String> = emit
    .defined_patterns
    .iter()
    .filter(|(_, v)| **v < 0)
    .map(|(k, _)| k.clone())
    .collect();
  keys.sort();
  for name in keys {
    let from = format!("\\patternadd{{{}}}", name);
    let to = format!("\\patterndefadd{{{}}}", name);
    if let Some(idx) = docs.find(&from) {
      docs.replace_range(idx..idx + from.len(), &to);
    }
  }
  docs
}

// ----- string-escape helpers ---------------------------------------------

/// `cleanTeX`: escape `#`, escape `_`, wrap `<...>` in `\texttt{...}`,
/// strip URN prefix, recognise `#PCDATA`.
pub fn clean_tex(s: &str) -> String {
  if s == "#PCDATA" {
    return String::from(r"\typename{text}");
  }
  let mut out = strip_urn_prefix(s);
  // Order matters: escape # before \texttt{...} expansion (no #s in
  // the wrapper), and _ at the end so the others' inserted text
  // doesn't collide.
  out = out.replace('#', "\\#");
  out = wrap_angle_text(&out);
  out = out.replace('_', "\\_");
  out
}

/// `cleanTeXName`: clean_tex + strip a leading `ltx:` prefix.
fn clean_tex_name(s: &str) -> String {
  let cleaned = clean_tex(s);
  cleaned.strip_prefix("ltx:").map(String::from).unwrap_or(cleaned)
}

fn strip_urn_prefix(s: &str) -> String {
  s.strip_prefix("urn:x-LaTeXML:RelaxNG:").unwrap_or(s).to_string()
}

/// Replace each `<TEXT>` substring with `\texttt{TEXT}`.
fn wrap_angle_text(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let bytes = s.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    if bytes[i] == b'<' {
      if let Some(end) = s[i + 1..].find('>') {
        let inner = &s[i + 1..i + 1 + end];
        out.push_str("\\texttt{");
        out.push_str(inner);
        out.push('}');
        i += 1 + end + 1;
        continue;
      }
    }
    out.push(bytes[i] as char);
    i += 1;
  }
  out
}

// ----- main dispatcher ----------------------------------------------------

impl EmitState<'_> {
  fn to_tex(&mut self, p: &Pattern) -> String {
    match p {
      Pattern::Doc(s) => format!("{}\n", clean_tex(s)),
      Pattern::Ref { qname } => self.to_tex_ref(qname),
      Pattern::Def { combiner, name, body } => {
        let combiner_label = match combiner {
          DefCombiner::Group => "",
          DefCombiner::Choice => "choice",
          DefCombiner::Interleave => "interleave",
        };
        self.to_tex_def(combiner_label, name, body)
      },
      Pattern::Element { name, body } => self.to_tex_element(name, body),
      Pattern::Attribute { name, body } => self.to_tex_attribute(name, body),
      Pattern::Combination { op, body } => self.to_tex_combination(*op, body),
      Pattern::Data(t) => format!("\\typename{{{}}}", clean_tex(t)),
      Pattern::Value(v) => format!("\\attrval{{{}}}", clean_tex(v)),
      Pattern::Start { body } => {
        let (docs, spec) = self.extract_docs(body);
        let content = spec
          .iter()
          .map(|p| self.to_tex(p))
          .collect::<Vec<_>>()
          .join(" ");
        let mut s = format!("\\item[\\textit{{Start}}]\\textbf{{==}}\\ {}", content);
        if !docs.is_empty() {
          s.push_str(&format!(" \\par{}", docs));
        }
        s
      },
      Pattern::Grammar { body, .. } => {
        // Collapse leading module includes into a single \item[Included]
        let mut mods: Vec<String> = Vec::new();
        let mut rest: Vec<&Pattern> = Vec::new();
        for d in body {
          match d {
            Pattern::Module { name, .. } => mods.push(name.clone()),
            other => rest.push(other),
          }
        }
        let mut parts: Vec<String> = Vec::new();
        if !mods.is_empty() {
          let refs: Vec<String> = mods
            .iter()
            .map(|m| format!("\\moduleref{{{}}}", clean_tex(m)))
            .collect();
          parts.push(format!("\\item[\\textit{{Included}}]{}", refs.join(", ")));
        }
        for r in rest {
          parts.push(self.to_tex(r));
        }
        parts.join("\n")
      },
      Pattern::Module { name, .. } => {
        if self.opts.skip_svg && is_svg_module(name) {
          format!("\\item[\\textit{{Module }}{}] included.", clean_tex(name))
        } else {
          format!(
            "\\item[\\textit{{Module }}\\moduleref{{{}}}] included.",
            clean_tex(name)
          )
        }
      },
      Pattern::ParentRef { qname } => self.to_tex_ref(qname),
      Pattern::ElementRef { qname } => format!("\\elementref{{{}}}", clean_tex_name(qname)),
      Pattern::Override { module, .. } => self.to_tex(module),
      Pattern::Text => clean_tex("#PCDATA"),
    }
  }

  fn to_tex_ref(&self, name: &str) -> String {
    if let Some(el) = self.rng.elementdefs.get(name) {
      let cleaned = clean_tex_name(el);
      if self.opts.skip_xhtml && cleaned == "xhtml:*" {
        return String::from("\\texttt{xhtml:*}");
      }
      return format!("\\elementref{{{}}}", cleaned);
    }
    if name.ends_with("_attributes") || name.ends_with("_model") {
      if let Some(def) = self.rng.defs.get(name) {
        // Read-only recursion is fine here; we don't mutate state on
        // the ref-expansion path (Perl doesn't either).
        let cloned = def.clone();
        let mut tmp = EmitState {
          rng:              self.rng,
          opts:             self.opts,
          defined_patterns: HashMap::default(),
        };
        return tmp.to_tex(&cloned);
      }
    }
    let stripped = strip_first_qualifier(name);
    if self.opts.skip_svg && stripped == "svg" {
      return String::from("\\texttt{svg:svg}");
    }
    format!("\\patternref{{{}}}", clean_tex(&stripped))
  }

  fn to_tex_def(&mut self, combiner: &str, qname: &str, data: &[Pattern]) -> String {
    if self.opts.skip_aria && qname.contains("aria") {
      return String::new();
    }
    if qname.ends_with("_attributes") || qname.ends_with("_model") {
      return String::new();
    }
    let stripped = strip_first_qualifier(qname);
    if self.opts.skip_svg && stripped.starts_with("svg") {
      return String::new();
    }
    let cleaned_name = clean_tex(&stripped);
    let (docs, spec) = self.extract_docs(data);
    let (attr, content) = self.to_tex_body(&spec);

    if !combiner.is_empty() {
      let mut body = attr;
      if !content.is_empty() {
        let sep = if combiner == "choice" { "\\textbar=" } else { "\\&=" };
        body.push_str(&format!("\\item[{}] {}", sep, content));
      }
      self
        .defined_patterns
        .entry(cleaned_name.clone())
        .or_insert(-1);
      return format!("\\patternadd{{{}}}{{{}}}{{{}}}\n", cleaned_name, docs, body);
    }

    // Bare def
    let mut attr = attr;
    let mut content = content;
    if attr.is_empty() && cleaned_name.contains("\\_attributes") {
      attr = String::from("\\item[\\textit{Attributes:}] \\textit{empty}");
    }
    if content.is_empty() && cleaned_name.contains("\\_model") {
      content = String::from("\\textit{empty}");
    }
    let mut body = attr;
    if !content.is_empty() {
      body.push_str(&format!("\\item[\\textit{{Content}}:] {}", content));
    }
    // Expansion line (when defs[qname] is content-shaped and differs).
    if !cleaned_name.contains("\\_attributes") {
      if let Some(stored) = self.rng.defs.get(qname) {
        if self.is_content(stored) && !self.is_attributes(stored) {
          let (xattr, xcontent) = self.to_tex_body(std::slice::from_ref(stored));
          if xattr.is_empty() && !xcontent.is_empty() && xcontent != content {
            body.push_str(&format!("\\item[\\textit{{Expansion}}:] {}", xcontent));
          }
        }
      }
    }
    if let Some(uses) = self.symbol_uses(qname) {
      body.push_str(&format!("\\item[\\textit{{Used by}}:] {}", uses));
    }
    if matches!(self.defined_patterns.get(&cleaned_name), Some(v) if *v > 0) {
      return String::new();
    }
    self.defined_patterns.insert(cleaned_name.clone(), 1);
    format!("\\patterndef{{{}}}{{{}}}{{{}}}\n", cleaned_name, docs, body)
  }

  fn to_tex_element(&mut self, qname: &str, data: &[Pattern]) -> String {
    let local = qname.strip_prefix("ltx:").unwrap_or(qname);
    if self.opts.skip_xhtml && local == "xhtml:*" {
      return String::new();
    }
    let cleaned = clean_tex_name(qname);
    let (docs, spec) = self.extract_docs(data);
    let (attr, content) = self.to_tex_body(&spec);
    let content = if content.is_empty() {
      String::from("\\typename{empty}")
    } else {
      content
    };
    let mut body = attr;
    body.push_str(&format!("\\item[\\textit{{Content}}:] {}", content));
    if let Some(ename) = self.rng.element_reverse_defs.get(qname) {
      if let Some(uses) = self.symbol_uses(ename) {
        body.push_str(&format!("\\item[\\textit{{Used by}}:] {}", uses));
      }
    }
    format!("\\elementdef{{{}}}{{{}}}{{{}}}\n", cleaned, docs, body)
  }

  fn to_tex_attribute(&mut self, name: &str, data: &[Pattern]) -> String {
    let cleaned = clean_tex_name(name);
    let (docs, spec) = self.extract_docs(data);
    let content = if spec.is_empty() {
      String::from("\\typename{text}")
    } else {
      spec
        .iter()
        .map(|p| self.to_tex(p))
        .collect::<Vec<_>>()
        .join(" ")
    };
    if let Some(rest) = cleaned.strip_prefix('!') {
      return format!("\\item[\\textit{{Exluding attribute }}]\\texttt{{{}}}", rest);
    }
    format!("\\attrdef{{{}}}{{{}}}{{{}}}", cleaned, docs, content)
  }

  fn to_tex_combination(&mut self, op: CombineOp, data: &[Pattern]) -> String {
    let inner: Vec<String> = data.iter().map(|p| self.to_tex(p)).collect();
    match op {
      CombineOp::Group => {
        if inner.len() == 1 {
          inner.into_iter().next().unwrap()
        } else {
          format!("({})", inner.join(", "))
        }
      },
      CombineOp::Interleave => format!("({})", inner.join(" ~\\&~ ")),
      CombineOp::Choice => format!("({})", inner.join(" ~\\textbar~ ")),
      CombineOp::Optional => {
        // Single attribute body: emit without the textsuperscript wrapper.
        if inner.len() == 1 && matches!(data[0], Pattern::Attribute { .. }) {
          inner.into_iter().next().unwrap()
        } else {
          format!("{}\\textsuperscript{{?}}", inner.first().cloned().unwrap_or_default())
        }
      },
      CombineOp::ZeroOrMore | CombineOp::OneOrMore => {
        // Note: Perl emits ^{*} for both zeroOrMore and oneOrMore — preserved.
        format!(
          "{}\\textsuperscript{{*}}",
          inner.first().cloned().unwrap_or_default()
        )
      },
      CombineOp::List => format!("({})", inner.join(", ")),
    }
  }

  // ----- helpers ----------------------------------------------------------

  /// Pull leading `Doc` items from `data`, return (docs-joined, rest).
  /// Mirrors `toTeXExtractDocs`.
  fn extract_docs(&mut self, data: &[Pattern]) -> (String, Vec<Pattern>) {
    let mut docs = String::new();
    let mut rest = Vec::with_capacity(data.len());
    for item in data {
      if let Pattern::Doc(_) = item {
        docs.push_str(&self.to_tex(item));
      } else {
        rest.push(item.clone());
      }
    }
    (docs, rest)
  }

  /// Partition `data` into `(attrs_string, content_string)`, with the
  /// same heuristics as Perl `toTeXBody`. Recursive expansion of
  /// `*_attributes` / `*_model` refs, and pattern refs whose name ends
  /// with `attributes` flow into the attribute list as-is.
  fn to_tex_body(&mut self, data: &[Pattern]) -> (String, String) {
    let mut attributes: Vec<String> = Vec::new();
    let mut content: Vec<String> = Vec::new();
    let mut attr_patterns: Vec<String> = Vec::new();
    // Perl uses shift+unshift to inline-expand `*_attributes`/`*_model`
    // refs and attribute-shaped Combinations as their members are
    // encountered. A front-poppable deque mirrors that traversal.
    let mut deque: std::collections::VecDeque<Pattern> = data.iter().cloned().collect();
    while let Some(item) = deque.pop_front() {
      match &item {
        Pattern::Attribute { .. } => {
          attributes.push(self.to_tex(&item));
        },
        Pattern::Combination { body, .. } if self.is_attributes(&item) => {
          for c in body.iter().cloned().rev() {
            deque.push_front(c);
          }
        },
        Pattern::Ref { qname } if qname.ends_with("_attributes") || qname.ends_with("_model") => {
          if let Some(def) = self.rng.defs.get(qname).cloned() {
            deque.push_front(def);
          }
        },
        Pattern::Ref { qname } if qname_ends_with_attributes(qname) => {
          attr_patterns.push(self.to_tex(&item));
        },
        _ => content.push(self.to_tex(&item)),
      }
    }

    let mut attr_str = String::new();
    if !attr_patterns.is_empty() {
      attr_str.push_str("\\item[\\textit{Attributes}:] ");
      attr_str.push_str(&attr_patterns.join(", "));
    }
    for a in attributes {
      attr_str.push_str(&a);
    }
    let content_str = content.join(", ");
    (attr_str, content_str)
  }

  /// Pred: does `item` describe purely attribute content?
  fn is_attributes(&self, item: &Pattern) -> bool {
    match item {
      Pattern::Attribute { .. } => true,
      Pattern::Ref { qname } => self
        .rng
        .defs
        .get(qname)
        .map(|p| self.is_attributes(p))
        .unwrap_or(false),
      Pattern::Combination { op, body }
        if matches!(
          op,
          CombineOp::Optional
            | CombineOp::Choice
            | CombineOp::Group
            | CombineOp::ZeroOrMore
            | CombineOp::OneOrMore
        ) =>
      {
        body.iter().all(|p| self.is_attributes(p))
      },
      _ => false,
    }
  }

  /// Pred: does `item` describe purely element / `#PCDATA` content?
  fn is_content(&self, item: &Pattern) -> bool {
    match item {
      Pattern::Element { .. } | Pattern::Grammar { .. } => true,
      Pattern::Ref { qname } => {
        if self.rng.elementdefs.contains_key(qname) {
          return true;
        }
        self
          .rng
          .defs
          .get(qname)
          .map(|p| self.is_content(p))
          .unwrap_or(false)
      },
      Pattern::Combination { op, body }
        if matches!(
          op,
          CombineOp::Optional
            | CombineOp::Choice
            | CombineOp::Group
            | CombineOp::ZeroOrMore
            | CombineOp::OneOrMore
        ) =>
      {
        body.iter().all(|p| self.is_content(p))
      },
      Pattern::Text => true,
      _ => false,
    }
  }

  /// Format the "Used by:" link list for `qname`. Returns `None` when
  /// the symbol has no recorded uses.
  fn symbol_uses(&self, qname: &str) -> Option<String> {
    let uses = self.rng.uses_name.get(qname)?;
    let mut sorted: Vec<&String> = uses.iter().collect();
    sorted.sort();
    let mut transformed: Vec<String> = Vec::new();
    for u in sorted {
      if self.opts.skip_svg && u.contains("SVG.") {
        continue;
      }
      // pattern:[^:]*:NAME_(attributes|model) → element:NAME
      if let Some(rest) = u.strip_prefix("pattern:") {
        if let Some(idx) = rest.find(':') {
          let after = &rest[idx + 1..];
          if let Some(name) = after
            .strip_suffix("_attributes")
            .or_else(|| after.strip_suffix("_model"))
          {
            transformed.push(format!("element:{}", name));
            continue;
          }
        }
      }
      transformed.push(u.clone());
    }
    let mut parts: Vec<String> = Vec::new();
    for t in &transformed {
      if let Some(rest) = t.strip_prefix("pattern:") {
        if let Some(idx) = rest.find(':') {
          let name = &rest[idx + 1..];
          parts.push(format!("\\patternref{{{}}}", clean_tex(name)));
        }
      }
    }
    for t in &transformed {
      if let Some(name) = t.strip_prefix("element:") {
        parts.push(format!("\\elementref{{{}}}", clean_tex_name(name)));
      }
    }
    if parts.is_empty() {
      None
    } else {
      Some(parts.join(", "))
    }
  }
}

/// Heuristic: is this module name an SVG module? Matches both the
/// URN-prefixed form (`urn:x-LaTeXML:RelaxNG:svg:…`, the path-aware
/// LaTeXML pipeline form) and the bare `svg…` filename stems trang
/// emits when expanding LaTeXML.rnc with the OASIS catalog (which
/// strips the `urn:` prefix). LaTeXML's own modules don't start with
/// `svg`, so the prefix match doesn't false-positive.
fn is_svg_module(name: &str) -> bool {
  name.contains(":svg:") || name.starts_with("svg")
}

fn strip_first_qualifier(s: &str) -> String {
  // `s/^\w+://` — strip a leading prefix up to the first colon, IF
  // that prefix is `\w+`. Otherwise return as-is.
  if let Some(idx) = s.find(':') {
    let prefix = &s[..idx];
    if !prefix.is_empty() && prefix.chars().all(|c| c.is_alphanumeric() || c == '_') {
      return s[idx + 1..].to_string();
    }
  }
  s.to_string()
}

fn qname_ends_with_attributes(qname: &str) -> bool {
  // Matches the Perl regex `[^a-zA-Z]attributes$`.
  let rest = qname.strip_suffix("attributes").unwrap_or("");
  if qname == "attributes" {
    return false;
  }
  match rest.chars().last() {
    Some(c) => !c.is_ascii_alphabetic() && qname.ends_with("attributes"),
    None => false,
  }
}

// ----- unit tests ---------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clean_tex_pcdata() {
    assert_eq!(clean_tex("#PCDATA"), r"\typename{text}");
  }

  #[test]
  fn clean_tex_underscore() {
    assert_eq!(clean_tex("foo_bar"), r"foo\_bar");
  }

  #[test]
  fn clean_tex_strips_urn() {
    assert_eq!(clean_tex("urn:x-LaTeXML:RelaxNG:foo"), "foo");
  }

  #[test]
  fn clean_tex_wraps_angles() {
    // <text> in the middle of a name → \texttt{text}.
    assert_eq!(clean_tex("a<b>c"), r"a\texttt{b}c");
  }

  #[test]
  fn clean_tex_escapes_hash() {
    assert_eq!(clean_tex("foo#bar"), r"foo\#bar");
  }

  #[test]
  fn clean_tex_name_strips_ltx_prefix() {
    assert_eq!(clean_tex_name("ltx:para"), "para");
    assert_eq!(clean_tex_name("xhtml:div"), "xhtml:div");
  }

  #[test]
  fn document_modules_emits_schemamodule() {
    let mut rng = Relaxng::default();
    rng.modules.push(Pattern::Module {
      name: "test".into(),
      body: vec![],
    });
    let out = document_modules(&rng, Options::default());
    assert!(out.contains("\\begin{schemamodule}{test}"));
    assert!(out.contains("\\end{schemamodule}"));
  }

  #[test]
  fn document_modules_skips_svg_module_when_skip_svg() {
    let mut rng = Relaxng::default();
    rng.modules.push(Pattern::Module {
      name: "x:svg:foo".into(),
      body: vec![],
    });
    let out = document_modules(&rng, Options::default());
    assert!(!out.contains("schemamodule"));
  }

  #[test]
  fn combination_rendering() {
    let rng = Relaxng::default();
    let mut emit = EmitState { rng: &rng, opts: Options::default(), defined_patterns: HashMap::default() };
    let body = vec![
      Pattern::Ref { qname: "g:A".into() },
      Pattern::Ref { qname: "g:B".into() },
    ];
    let group = emit.to_tex_combination(CombineOp::Group, &body);
    assert_eq!(group, "(\\patternref{A}, \\patternref{B})");
    let choice = emit.to_tex_combination(CombineOp::Choice, &body);
    assert_eq!(choice, "(\\patternref{A} ~\\textbar~ \\patternref{B})");
    let inter = emit.to_tex_combination(CombineOp::Interleave, &body);
    assert_eq!(inter, "(\\patternref{A} ~\\&~ \\patternref{B})");
  }

  #[test]
  fn singleton_group_collapses_in_combination() {
    let rng = Relaxng::default();
    let mut emit = EmitState { rng: &rng, opts: Options::default(), defined_patterns: HashMap::default() };
    let body = vec![Pattern::Ref { qname: "g:Only".into() }];
    let result = emit.to_tex_combination(CombineOp::Group, &body);
    assert_eq!(result, "\\patternref{Only}");
  }

  #[test]
  fn element_renders_with_content_and_used_by() {
    let mut rng = Relaxng::default();
    rng.element_reverse_defs.insert("foo".into(), "g:Foo".into());
    rng
      .uses_name
      .entry("g:Foo".into())
      .or_default()
      .insert("element:bar".into());
    let mut emit = EmitState { rng: &rng, opts: Options::default(), defined_patterns: HashMap::default() };
    let out = emit.to_tex_element("foo", &[Pattern::Text]);
    assert!(out.contains("\\elementdef{foo}"));
    assert!(out.contains("\\item[\\textit{Content}:]"));
    assert!(out.contains("\\elementref{bar}"));
  }

  #[test]
  fn def_emits_patterndef_then_skips_duplicates() {
    let rng = Relaxng::default();
    let mut emit = EmitState { rng: &rng, opts: Options::default(), defined_patterns: HashMap::default() };
    let body = vec![Pattern::Text];
    let first = emit.to_tex_def("", "g:X", &body);
    let second = emit.to_tex_def("", "g:X", &body);
    assert!(first.contains("\\patterndef{X}"));
    assert_eq!(second, "");
  }

  #[test]
  fn def_combine_choice_emits_patternadd() {
    let rng = Relaxng::default();
    let mut emit = EmitState { rng: &rng, opts: Options::default(), defined_patterns: HashMap::default() };
    let out = emit.to_tex_def("choice", "g:X", &[Pattern::Text]);
    assert!(out.contains("\\patternadd{X}"));
    // -1 marker recorded so the post-pass can upgrade if no \patterndef was emitted.
    assert_eq!(emit.defined_patterns.get("X").copied(), Some(-1));
  }

  #[test]
  fn unmatched_patternadd_upgrades_to_patterndefadd() {
    let mut rng = Relaxng::default();
    rng.modules.push(Pattern::Module {
      name: "m".into(),
      body: vec![Pattern::Def {
        combiner: DefCombiner::Choice,
        name:     "g:Lonely".into(),
        body:     vec![Pattern::Text],
      }],
    });
    let out = document_modules(&rng, Options::default());
    assert!(
      out.contains("\\patterndefadd{Lonely}"),
      "expected upgrade, got:\n{}",
      out
    );
    assert!(
      !out.contains("\\patternadd{Lonely}"),
      "patternadd should have been replaced"
    );
  }
}

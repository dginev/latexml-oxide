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
use std::collections::BTreeMap;

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
  // Each Module renders as one page (`--splitat=section`), regardless
  // of def count. Page-size is mitigated client-side by CSS lazy
  // paint and a JS search/filter input — splitting the module across
  // multiple pages would break Ctrl-F across the whole module, which
  // is the primary navigation affordance.
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

    // Modules typically wrap their content in a single
    // `Pattern::Grammar`; descend into it so the iteration sees each
    // individual def directly.
    let to_emit: Vec<&Pattern> = content
      .iter()
      .flat_map(|item| match item {
        Pattern::Grammar { body, .. } => body.iter().collect::<Vec<_>>(),
        other => vec![other],
      })
      .collect();

    let mut preamble = String::new();
    // Outer-Grammar "Includes:" preamble line (collected once,
    // emitted on the first synthetic group's page if partitioning).
    for item in content {
      if let Pattern::Grammar { body, .. } = item {
        let mods: Vec<String> = body
          .iter()
          .filter_map(|d| match d {
            Pattern::Module { name, .. } => Some(name.clone()),
            _ => None,
          })
          .collect();
        if !mods.is_empty() {
          let refs: Vec<String> = mods
            .iter()
            .map(|m| format!("\\moduleref{{{}}}", clean_tex(m)))
            .collect();
          if !preamble.is_empty() {
            preamble.push('\n');
          }
          preamble.push_str(&format!(
            "\\par\\noindent\\textit{{Includes:}} {}.",
            refs.join(", ")
          ));
        }
      }
    }

    // Render each def, preserving source order in `defs`.
    let mut defs: Vec<String> = Vec::new();
    for item in &to_emit {
      // Module entries already accounted for in the Includes line.
      if matches!(item, Pattern::Module { .. }) {
        continue;
      }
      let rendered = emit.to_tex(item);
      if rendered.is_empty() {
        continue;
      }
      match item {
        Pattern::Doc(_) | Pattern::Start { .. } => {
          if !preamble.is_empty() {
            preamble.push('\n');
          }
          preamble.push_str(&rendered);
        },
        _ => defs.push(rendered),
      }
    }

    docs.push_str(&format!("\n\\begin{{schemamodule}}{{{}}}", mod_name));
    if !preamble.is_empty() {
      docs.push('\n');
      docs.push_str(&preamble);
    }
    let body = defs.join("\n");
    if !body.is_empty() {
      docs.push_str(&format!(
        "\n\\begin{{description}}\n{}\n\\end{{description}}",
        body
      ));
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
        // Module-level <start>: emit as a paragraph, not a description-
        // list item. The Perl original emitted
        // `\item[\textit{Start}]\textbf{==}\ root`, but that depended
        // on living inside the moduledescription environment that the
        // old section-split layout opened. With per-def subsection
        // splitting there's no enclosing list at module scope, so
        // module-preamble notes flow as prose.
        let (docs, spec) = self.extract_docs(body);
        let content = spec
          .iter()
          .map(|p| self.to_tex(p))
          .collect::<Vec<_>>()
          .join(" ");
        let mut s = format!("\\par\\noindent\\textit{{Start symbol:}} {}", content);
        if !docs.is_empty() {
          s.push_str(&format!(" \\par{}", docs));
        }
        s
      },
      Pattern::Grammar { body, .. } => {
        // The grammar's leading <include>'s become an "Includes" line
        // of `\moduleref{…}`s, then the rest of the body (defs, doc,
        // etc.) flows through normally. Module preamble is paragraph
        // text — see Pattern::Start above for rationale.
        let mut mods: Vec<String> = Vec::new();
        let mut rest: Vec<&Pattern> = Vec::new();
        for d in body {
          match d {
            Pattern::Module { name, .. } => mods.push(name.clone()),
            other => rest.push(other),
          }
        }
        let mut out = String::new();
        if !mods.is_empty() {
          let refs: Vec<String> = mods
            .iter()
            .map(|m| format!("\\moduleref{{{}}}", clean_tex(m)))
            .collect();
          out.push_str(&format!(
            "\\par\\noindent\\textit{{Includes:}} {}.\n",
            refs.join(", ")
          ));
        }
        for r in rest {
          out.push_str(&self.to_tex(r));
          out.push('\n');
        }
        out
      },
      Pattern::Module { name, .. } => {
        // Standalone Module reference (rare — most are absorbed into
        // the parent Grammar's "Includes" line). Emit as a brief
        // paragraph note rather than a list item.
        if self.opts.skip_svg && is_svg_module(name) {
          format!(
            "\\par\\noindent\\textit{{Module}} \\texttt{{{}}} \\textit{{included}}.",
            clean_tex(name)
          )
        } else {
          format!(
            "\\par\\noindent\\textit{{Module}} \\moduleref{{{}}} \\textit{{included}}.",
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
          // Suppress when the stored form differs from `content` only by
          // the outer `(...)` wrap that `to_tex_combination(Group)` adds:
          // for patterns like `anyElement` the def-args path renders the
          // body unwrapped while the stored-Combination path re-wraps it,
          // and emitting both yields a near-duplicate Expansion block.
          let unwrapped = strip_outer_parens(&xcontent);
          if xattr.is_empty() && !xcontent.is_empty() && xcontent != content && unwrapped != content {
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
    // Wildcard element names (`*`, `*:*`, `prefix:*`) come from `<anyName/>`
    // / `<nsName/>` in the source schema — they describe "an element of
    // any name", not a real definable element. Render them inline as a
    // content-model expression so they sit gracefully inside the parent
    // pattern's body. Emitting `\elementdef{*}{...}` here would inject a
    // nested definition card (with its own Content/Attribute rows) into
    // the parent card, which is the rendering bug visible on patterns
    // like `anyElement`.
    if is_wildcard_name(qname) {
      return self.render_inline_element(qname, data);
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
    if let Some(rest) = cleaned.strip_prefix('!') {
      return format!("\\item[\\textit{{Exluding attribute }}]\\texttt{{{}}}", rest);
    }
    // Same wildcard-handling rationale as `to_tex_element`: render inline
    // so the parent pattern's body doesn't pick up a nested `\attrdef`
    // item card for a name like `*` or `*:*`.
    if is_wildcard_name(&cleaned) {
      return self.render_inline_attribute(&cleaned, data);
    }
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
    format!("\\attrdef{{{}}}{{{}}}{{{}}}", cleaned, docs, content)
  }

  /// Inline content-model rendering of an `<element><anyName/>...</element>`
  /// (or `<nsName/>`) pattern. Returns a TeX fragment of the form
  /// `\textit{element}~\texttt{NAME}~\{ BODY \}` — text-shaped, suitable
  /// for embedding inside another pattern's content model.
  fn render_inline_element(&mut self, qname: &str, data: &[Pattern]) -> String {
    let cleaned = clean_tex_name(qname);
    let (_docs, spec) = self.extract_docs(data);
    let parts: Vec<String> = spec.iter().map(|p| self.to_tex(p)).collect();
    let parts: Vec<String> = parts.into_iter().filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
      format!("\\textit{{element}}~\\texttt{{{}}}", cleaned)
    } else {
      format!(
        "\\textit{{element}}~\\texttt{{{}}}~\\{{{}\\}}",
        cleaned,
        parts.join(", ")
      )
    }
  }

  /// Inline rendering of an `<attribute><anyName/>...</attribute>` (or
  /// `<nsName/>`) pattern: `\textit{attribute}~\texttt{NAME}=CONTENT`.
  fn render_inline_attribute(&mut self, cleaned: &str, data: &[Pattern]) -> String {
    let (_docs, spec) = self.extract_docs(data);
    let content = if spec.is_empty() {
      String::from("\\typename{text}")
    } else {
      spec
        .iter()
        .map(|p| self.to_tex(p))
        .collect::<Vec<_>>()
        .join(" ")
    };
    format!("\\textit{{attribute}}~\\texttt{{{}}}={}", cleaned, content)
  }

  fn to_tex_combination(&mut self, op: CombineOp, data: &[Pattern]) -> String {
    // Collapse adjacent wildcard pairs (`*` followed by `*:*`) — they
    // come from a single `<anyName/>` and would otherwise render twice.
    let dedup_owned: Vec<Pattern>;
    let data: &[Pattern] = if has_wildcard_pair(data) {
      dedup_owned = dedupe_wildcard_pairs(data);
      &dedup_owned
    } else {
      data
    };
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
    // Dedupe `*`/`*:*` wildcard pairs at this layer too: the def-args
    // path (`<define>` with a single wildcard `<element>` body) feeds
    // them straight in without a Combination wrapper.
    let dedup_owned: Vec<Pattern>;
    let data: &[Pattern] = if has_wildcard_pair(data) {
      dedup_owned = dedupe_wildcard_pairs(data);
      &dedup_owned
    } else {
      data
    };
    // Group "trivial-body" attributes by their datatype so a long run
    // of identical `attribute foo {text}` rows collapses into a single
    // `Text attributes: a, b, c` line. Wildcards (`*`, `*:*`) skip the
    // grouping path — they have no enumerable name. Attributes carrying
    // a `Doc` annotation also skip it (we'd lose the doc otherwise).
    // Key = the type label ("text" / "string" / …); BTreeMap gives a
    // stable alphabetical render order across types.
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut deque: std::collections::VecDeque<Pattern> = data.iter().cloned().collect();
    while let Some(item) = deque.pop_front() {
      match &item {
        Pattern::Attribute { name, body } => {
          if let Some(t) = simple_attr_type(body) {
            if !is_wildcard_name(name) {
              grouped.entry(t).or_default().push(clean_tex_name(name));
              continue;
            }
          }
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
    // Grouped lines render before the per-attribute cards: the bulk
    // overview reads first, then any non-trivial typed attributes.
    // Each name is wrapped in `\texttt{...}` so it lands under
    // `.ltx_font_typewriter` (var(--font-code) — SF Mono / Fira Mono
    // / etc.) in the rendered HTML; commas stay in body type so the
    // names visually separate.
    for (type_name, mut names) in grouped {
      names.sort();
      let monospaced: Vec<String> =
        names.iter().map(|n| format!("\\texttt{{{}}}", n)).collect();
      attr_str.push_str(&format!(
        "\\item[\\textit{{{}}}:] {}",
        attr_group_label(&type_name),
        monospaced.join(", "),
      ));
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

/// Classify an `<attribute>` body as a "simple type" suitable for
/// grouping in the `to_tex_body` compression line. Returns the type
/// label (e.g. `"text"`, `"string"`, `"integer"`) when the body is one
/// of the trivial shapes:
///
/// * empty (`<attribute name="foo"/>` — implicitly text-valued),
/// * `[Pattern::Text]` (RNC `attribute foo {text}`),
/// * `[Pattern::Data(t)]` (RNC `attribute foo {xsd:t}`).
///
/// A body carrying a `Doc` annotation is rejected: the per-attribute
/// docstring would be lost in the grouped form, so those keep their
/// individual `\attrdef` cards.
fn simple_attr_type(body: &[Pattern]) -> Option<String> {
  if body.iter().any(|p| matches!(p, Pattern::Doc(_))) {
    return None;
  }
  match body {
    [] => Some("text".into()),
    [Pattern::Text] => Some("text".into()),
    [Pattern::Data(t)] => Some(t.clone()),
    _ => None,
  }
}

/// Format the kicker label for a grouped-attribute line:
/// `"text" → "Text attributes"`, `"string" → "String attributes"`,
/// `"anyURI" → "AnyURI attributes"`. Capitalises the first character
/// of the type name and appends ` attributes`.
fn attr_group_label(type_name: &str) -> String {
  let cleaned = clean_tex(type_name);
  let mut chars = cleaned.chars();
  match chars.next() {
    None => "Attributes".into(),
    Some(c) => format!(
      "{}{} attributes",
      c.to_uppercase().collect::<String>(),
      chars.as_str()
    ),
  }
}

/// True if `name` is an `<anyName/>` / `<nsName/>` wildcard:
/// `*` (no namespace), `*:*` (any namespace, any local) or `prefix:*`
/// (any local within a namespace). These names come from the scanner's
/// expansion of `<anyName/>` / `<nsName/>` in `scan_name_class` and
/// don't denote real definable element / attribute names.
fn is_wildcard_name(name: &str) -> bool {
  name == "*" || name == "*:*" || name.ends_with(":*")
}

/// True if `data` contains an adjacent `*` then `*:*` Element pair
/// (or the same shape for Attribute). Used as a cheap pre-check so the
/// dedupe path only allocates when there's actually something to fold.
fn has_wildcard_pair(data: &[Pattern]) -> bool {
  data.windows(2).any(|w| is_wildcard_pair(&w[0], &w[1]))
}

fn is_wildcard_pair(a: &Pattern, b: &Pattern) -> bool {
  match (a, b) {
    (Pattern::Element { name: n1, .. }, Pattern::Element { name: n2, .. })
    | (Pattern::Attribute { name: n1, .. }, Pattern::Attribute { name: n2, .. }) => {
      n1 == "*" && n2 == "*:*"
    },
    _ => false,
  }
}

/// Walk `data` and collapse each adjacent `*` / `*:*` Element- or
/// Attribute-pair (produced by the scanner from a single `<anyName/>`)
/// into the single `*:*` form. The pair always shares its body since
/// `scan_pattern_element` / `scan_pattern_attribute` build both members
/// from the same `body_proto.clone()`, so dropping the `*` half loses
/// no information.
fn dedupe_wildcard_pairs(data: &[Pattern]) -> Vec<Pattern> {
  let mut out = Vec::with_capacity(data.len());
  let mut i = 0;
  while i < data.len() {
    if i + 1 < data.len() && is_wildcard_pair(&data[i], &data[i + 1]) {
      // Keep the `*:*` member (data[i+1]) — it's the broader form and
      // reads more clearly in the rendered content model.
      out.push(data[i + 1].clone());
      i += 2;
    } else {
      out.push(data[i].clone());
      i += 1;
    }
  }
  out
}

/// If `s` is wrapped in matching outer parentheses (no other unbalanced
/// content at top level), return the unwrapped slice; otherwise `s`.
/// Used by the Expansion suppression in `to_tex_def` to detect when the
/// stored-Combination form differs from the raw def-args form by only
/// an outer `(...)` wrap.
fn strip_outer_parens(s: &str) -> &str {
  let bytes = s.as_bytes();
  if bytes.first() != Some(&b'(') || bytes.last() != Some(&b')') {
    return s;
  }
  // Confirm the outer `(` matches the outer `)` (no `(A)(B)` slip).
  let mut depth = 0i32;
  for (i, b) in bytes.iter().enumerate() {
    match b {
      b'(' => depth += 1,
      b')' => {
        depth -= 1;
        if depth == 0 && i + 1 != bytes.len() {
          return s;
        }
      },
      _ => {},
    }
  }
  if depth != 0 {
    return s;
  }
  &s[1..s.len() - 1]
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
  fn trivial_text_attributes_collapse_into_grouped_line() {
    // A long run of `attribute foo {text}?, ...` (the MathML on-event
    // attribute pattern) used to render as 30+ ATTRIBUTE / = text rows.
    // Compressed form: a single `Text attributes: a, b, c` line, names
    // sorted alphabetically.
    let xml = r##"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <define name="OnEvent">
          <group>
            <optional><attribute name="onclick"><text/></attribute></optional>
            <optional><attribute name="onabort"><text/></attribute></optional>
            <optional><attribute name="onblur"><text/></attribute></optional>
          </group>
        </define>
      </grammar>
    "##;
    use crate::common::relaxng::scan::scan_string;
    let mut rng = Relaxng::default();
    let raw = scan_string(&mut rng, xml).expect("scan");
    let wrapped = vec![Pattern::Module { name: "m".into(), body: raw }];
    let _ = crate::common::relaxng::simplify::simplify_top(&mut rng, wrapped);
    let out = document_modules(&rng, Options::default());

    assert!(
      out.contains(
        "\\item[\\textit{Text attributes}:] \\texttt{onabort}, \\texttt{onblur}, \\texttt{onclick}"
      ),
      "expected sorted Text attributes line with monospaced names, got:\n{}",
      out
    );
    assert!(
      !out.contains("\\attrdef{onclick}"),
      "trivial text attribute should not render as a per-attribute card:\n{}",
      out
    );
  }

  #[test]
  fn typed_attributes_grouped_per_type_label() {
    // Mixed simple types: text, xsd:string, xsd:integer. Each type
    // gets its own grouped line; non-trivial bodies stay as cards.
    let xml = r##"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0"
               datatypeLibrary="http://www.w3.org/2001/XMLSchema-datatypes">
        <define name="P">
          <group>
            <attribute name="a"><text/></attribute>
            <attribute name="b"><data type="string"/></attribute>
            <attribute name="c"><data type="integer"/></attribute>
            <attribute name="d">
              <choice><value>x</value><value>y</value></choice>
            </attribute>
          </group>
        </define>
      </grammar>
    "##;
    use crate::common::relaxng::scan::scan_string;
    let mut rng = Relaxng::default();
    let raw = scan_string(&mut rng, xml).expect("scan");
    let wrapped = vec![Pattern::Module { name: "m".into(), body: raw }];
    let _ = crate::common::relaxng::simplify::simplify_top(&mut rng, wrapped);
    let out = document_modules(&rng, Options::default());

    assert!(out.contains("\\item[\\textit{Text attributes}:] \\texttt{a}"), "{}", out);
    assert!(out.contains("\\item[\\textit{String attributes}:] \\texttt{b}"), "{}", out);
    assert!(out.contains("\\item[\\textit{Integer attributes}:] \\texttt{c}"), "{}", out);
    // The enum-bodied attribute must keep its individual card.
    assert!(out.contains("\\attrdef{d}"), "{}", out);
  }

  #[test]
  fn anyelement_renders_inline_without_nested_cards() {
    // Regression: `anyElement = element (*) {(attribute * {text}|text|anyElement)*}`
    // used to emit `\elementdef{*}{...}` and `\attrdef{*}{...}` cards
    // nested inside the `\patterndef{anyElement}{...}` body, and the
    // `*` / `*:*` wildcard pair from `<anyName/>` was double-rendered.
    let xml = r##"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <define name="anyElement">
          <element>
            <anyName/>
            <zeroOrMore>
              <choice>
                <attribute><anyName/><text/></attribute>
                <text/>
                <ref name="anyElement"/>
              </choice>
            </zeroOrMore>
          </element>
        </define>
      </grammar>
    "##;
    use crate::common::relaxng::scan::scan_string;
    let mut rng = Relaxng::default();
    let raw = scan_string(&mut rng, xml).expect("scan");
    let wrapped = vec![Pattern::Module { name: "m".into(), body: raw }];
    let _ = crate::common::relaxng::simplify::simplify_top(&mut rng, wrapped);
    let out = document_modules(&rng, Options::default());

    assert!(
      out.contains("\\patterndef{anyElement}"),
      "expected anyElement patterndef, got:\n{}",
      out
    );
    assert!(
      !out.contains("\\elementdef{*}"),
      "wildcard element rendered as nested elementdef card:\n{}",
      out
    );
    assert!(
      !out.contains("\\elementdef{*:*}"),
      "wildcard element rendered as nested elementdef card:\n{}",
      out
    );
    assert!(
      !out.contains("\\attrdef{*}"),
      "wildcard attribute rendered as nested attrdef card:\n{}",
      out
    );
    assert!(
      !out.contains("\\attrdef{*:*}"),
      "wildcard attribute rendered as nested attrdef card:\n{}",
      out
    );
    assert!(
      out.contains("\\textit{element}~\\texttt{*:*}"),
      "expected inline element render for wildcard:\n{}",
      out
    );
    assert!(
      out.contains("\\textit{attribute}~\\texttt{*:*}"),
      "expected inline attribute render for wildcard:\n{}",
      out
    );
    // Expansion line should be suppressed — content already shows the full body.
    assert!(
      !out.contains("\\textit{Expansion}"),
      "Expansion duplicates Content for anyElement; should be suppressed:\n{}",
      out
    );
  }

  #[test]
  fn dedupe_wildcard_pairs_collapses_adjacent() {
    let body = vec![
      Pattern::Element { name: "*".into(), body: vec![Pattern::Text] },
      Pattern::Element { name: "*:*".into(), body: vec![Pattern::Text] },
      Pattern::Ref { qname: "x".into() },
    ];
    let folded = dedupe_wildcard_pairs(&body);
    assert_eq!(folded.len(), 2);
    match &folded[0] {
      Pattern::Element { name, .. } => assert_eq!(name, "*:*"),
      other => panic!("expected Element *:*, got {:?}", other),
    }
  }

  #[test]
  fn strip_outer_parens_only_when_outer_match() {
    assert_eq!(strip_outer_parens("(abc)"), "abc");
    assert_eq!(strip_outer_parens("(a)(b)"), "(a)(b)");
    assert_eq!(strip_outer_parens("abc"), "abc");
    assert_eq!(strip_outer_parens("(a"), "(a");
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

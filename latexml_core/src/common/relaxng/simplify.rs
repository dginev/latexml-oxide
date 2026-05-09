//! AST normalization. Port of `RelaxNG.pm` lines 397–525.
//!
//! Walks the raw AST produced by [`super::scan`] under a binding context
//! (the enclosing `<grammar>` name) and:
//!
//! * resolves `Ref` / `ParentRef` qnames against the binding/parent-binding,
//! * records every reference site in [`Relaxng::uses_name`] (powers the
//!   "Used by:" lists in the schema docs),
//! * registers `Element` body patterns under [`Relaxng::elements`],
//! * combines `Def`s into a single canonical `Combination` per qname in
//!   [`Relaxng::defs`] (with [`Relaxng::def_combiner`] tracking which
//!   combiner won), tracking the singleton-element-def shortcut into
//!   [`Relaxng::elementdefs`] / [`Relaxng::element_reverse_defs`],
//! * resolves `Override`s by patching the wrapped module before
//!   re-running simplify on the patched form,
//! * preserves document-order of [`Relaxng::modules`].
//!
//! The simplifier is shape-preserving in its return value: every input
//! pattern (except Override and singleton-element-defs) emerges with
//! the same shape but possibly-rewritten qnames and recursively
//! simplified bodies. Side effects on `rng` are the substantive output.

use super::{CombineOp, DefCombiner, Pattern, Relaxng};

/// Top-level simplifier. Maps [`simplify`] across each top-level form.
pub fn simplify_top(rng: &mut Relaxng, raw: Vec<Pattern>) -> Vec<Pattern> {
  raw
    .into_iter()
    .flat_map(|p| simplify(rng, p, "", "", None))
    .collect()
}

/// Recursive normalizer.
///
/// `binding` is the qname-prefix of the currently-enclosing `<grammar>`
/// (so a `<ref name="X"/>` inside `<grammar>foo</grammar>` resolves to
/// `foo:X`). `parent` is the binding of the enclosing grammar one
/// level up — used when this node is a `<parentRef/>`. `container`,
/// when present, names the current host (`element:NAME` or
/// `pattern:NAME`) and seeds the "Used by" graph.
pub fn simplify(
  rng: &mut Relaxng,
  form: Pattern,
  binding: &str,
  parent: &str,
  container: Option<&str>,
) -> Vec<Pattern> {
  match form {
    Pattern::Grammar { name, body } => {
      let new_body = simplify_args(rng, body, &name, binding, container);
      vec![Pattern::Grammar { name, body: new_body }]
    },

    Pattern::Override { module, replacements } => {
      simplify_override(rng, *module, replacements, binding, parent, container)
    },

    Pattern::Module { name, body } => {
      // Push a placeholder FIRST so document-order in `rng.modules` is
      // preserved (any nested modules surfaced during the body simplify
      // appear after this one).
      let idx = rng.modules.len();
      rng.modules.push(Pattern::Module { name: name.clone(), body: Vec::new() });
      let new_body: Vec<Pattern> = body
        .into_iter()
        .flat_map(|p| simplify(rng, p, binding, parent, container))
        .collect();
      if let Some(Pattern::Module { body: slot, .. }) = rng.modules.get_mut(idx) {
        *slot = new_body.clone();
      }
      vec![Pattern::Module { name, body: new_body }]
    },

    Pattern::Element { name, body } => {
      let elem_container = format!("element:{}", name);
      let new_body: Vec<Pattern> = body
        .into_iter()
        .flat_map(|p| simplify(rng, p, binding, parent, Some(&elem_container)))
        .collect();
      rng
        .elements
        .entry(name.clone())
        .or_default()
        .extend(new_body.clone());
      vec![Pattern::Element { name, body: new_body }]
    },

    Pattern::Ref { qname } => simplify_ref(rng, &qname, binding, container, false),
    Pattern::ParentRef { qname } => simplify_ref(rng, &qname, parent, container, true),

    Pattern::Def { combiner, name, body } => {
      simplify_def(rng, combiner, name, body, binding, parent, container)
    },

    // Pass-through: simplify children, keep wrapper.
    Pattern::Combination { op, body } => {
      let new_body = simplify_args(rng, body, binding, parent, container);
      vec![Pattern::Combination { op, body: new_body }]
    },
    Pattern::Start { body } => {
      let new_body = simplify_args(rng, body, binding, parent, container);
      vec![Pattern::Start { body: new_body }]
    },
    Pattern::Attribute { name, body } => {
      let new_body = simplify_args(rng, body, binding, parent, container);
      vec![Pattern::Attribute { name, body: new_body }]
    },

    // Leaves: pass through unchanged.
    other @ (Pattern::Value(_)
    | Pattern::Data(_)
    | Pattern::Doc(_)
    | Pattern::Text
    | Pattern::ElementRef { .. }) => vec![other],
  }
}

fn simplify_args(
  rng: &mut Relaxng,
  forms: Vec<Pattern>,
  binding: &str,
  parent: &str,
  container: Option<&str>,
) -> Vec<Pattern> {
  forms
    .into_iter()
    .flat_map(|p| simplify(rng, p, binding, parent, container))
    .collect()
}

fn simplify_ref(
  rng: &mut Relaxng,
  name: &str,
  bind: &str,
  container: Option<&str>,
  _is_parent: bool,
) -> Vec<Pattern> {
  // ParentRef and Ref both return a `Ref` after qname-rewriting; the
  // distinction was only in which binding to use, which the caller has
  // already chosen by passing the right `bind`.
  let qname = format!("{}:{}", bind, name);
  if let Some(c) = container {
    rng
      .uses_name
      .entry(qname.clone())
      .or_default()
      .insert(c.to_string());
  }
  vec![Pattern::Ref { qname }]
}

fn simplify_def(
  rng: &mut Relaxng,
  combiner: DefCombiner,
  name: String,
  body: Vec<Pattern>,
  binding: &str,
  parent: &str,
  container: Option<&str>,
) -> Vec<Pattern> {
  let qname = format!("{}:{}", binding, name);
  if let Some(c) = container {
    rng
      .uses_name
      .entry(qname.clone())
      .or_default()
      .insert(c.to_string());
  }
  let pattern_container = format!("pattern:{}", qname);
  let args = simplify_args(rng, body, binding, parent, Some(&pattern_container));

  // Special case: a plain `<define>` with one Element body folds into
  // `elementdefs[qname] -> tag` and is replaced by the bare element.
  if combiner == DefCombiner::Group && args.len() == 1 {
    if let Pattern::Element { name: el_name, .. } = &args[0] {
      rng.elementdefs.insert(qname.clone(), el_name.clone());
      rng
        .element_reverse_defs
        .insert(el_name.clone(), qname.clone());
      return args;
    }
  }

  // Combine with any prior definition under the same qname.
  let xargs: Vec<Pattern> = args
    .iter()
    .filter(|p| !matches!(p, Pattern::Doc(_)))
    .cloned()
    .collect();
  let prev = rng.defs.get(&qname).cloned();
  let prev_combiner = rng.def_combiner.get(&qname).copied();

  let mut effective = combiner;
  let mut prev_args: Vec<Pattern> = Vec::new();
  let mut keep_prev = prev.is_some();
  if let Some(prev_pat) = &prev {
    if let Pattern::Combination { body, .. } = prev_pat {
      prev_args = body.clone();
    } else {
      prev_args = vec![prev_pat.clone()];
    }
    match (combiner, prev_combiner) {
      (DefCombiner::Group, Some(DefCombiner::Group)) => {
        // Apparent re-definition — drop the previous value.
        keep_prev = false;
      },
      (DefCombiner::Group, Some(other)) => {
        // Inherit the previous combiner so nested Group definitions
        // join under the previous combine="choice" / "interleave".
        effective = other;
      },
      _ => {},
    }
  }
  if !keep_prev {
    prev_args.clear();
  }

  let combination_op = match effective {
    DefCombiner::Group => CombineOp::Group,
    DefCombiner::Choice => CombineOp::Choice,
    DefCombiner::Interleave => CombineOp::Interleave,
  };
  let mut combined = prev_args;
  combined.extend(xargs);
  let combined_pat = simplify_combination(Pattern::Combination {
    op:   combination_op,
    body: combined,
  });
  rng.defs.insert(qname.clone(), combined_pat);
  rng.def_combiner.insert(qname.clone(), effective);

  // Returned pattern keeps the original combiner (matches Perl: stored
  // op stays $op even when effective combiner shifts).
  vec![Pattern::Def { combiner, name: qname, body: args }]
}

/// Recursively flatten same-op `Group`/`Choice` nests and collapse a
/// singleton `Group` to its only member. Port of `simplifyCombination`.
pub fn simplify_combination(pat: Pattern) -> Pattern {
  match pat {
    Pattern::Combination { op, body } => {
      let recursed: Vec<Pattern> = body.into_iter().map(simplify_combination).collect();
      let flattened: Vec<Pattern> = if matches!(op, CombineOp::Group | CombineOp::Choice) {
        let mut out = Vec::with_capacity(recursed.len());
        for s in recursed {
          match s {
            Pattern::Combination { op: inner_op, body: inner_body } if inner_op == op => {
              out.extend(inner_body);
            },
            other => out.push(other),
          }
        }
        out
      } else {
        recursed
      };
      if op == CombineOp::Group && flattened.len() == 1 {
        flattened.into_iter().next().unwrap()
      } else {
        Pattern::Combination { op, body: flattened }
      }
    },
    other => other,
  }
}

fn simplify_override(
  rng: &mut Relaxng,
  module: Pattern,
  replacements: Vec<Pattern>,
  binding: &str,
  parent: &str,
  container: Option<&str>,
) -> Vec<Pattern> {
  let (mod_name, mut patterns) = match module {
    Pattern::Module { name, body } => (name, body),
    other => {
      // Defensive: shouldn't happen, but fall back to the inner item.
      return simplify(rng, other, binding, parent, container);
    },
  };

  // If replacements include a <start>, drop the module's <start>.
  let has_replacement_start = replacements
    .iter()
    .any(|p| matches!(p, Pattern::Start { .. }));
  if has_replacement_start {
    patterns.retain(|p| !matches!(p, Pattern::Start { .. }));
  }

  // For each Def in replacements, remove the same-symbol Def from the
  // module's patterns. (Any combine="..." Defs in replacements just
  // accumulate — they don't strip the original.)
  let replacement_defs: Vec<(DefCombiner, String)> = replacements
    .iter()
    .filter_map(|p| match p {
      Pattern::Def { combiner, name, .. } => Some((*combiner, name.clone())),
      _ => None,
    })
    .collect();
  patterns.retain(|p| match p {
    Pattern::Def { combiner: c, name: n, .. } => {
      !replacement_defs.iter().any(|(rc, rn)| rc == c && rn == n)
    },
    _ => true,
  });

  let mut combined = patterns;
  combined.extend(replacements);
  let new_module = Pattern::Module {
    name: format!("{} (overridden)", mod_name),
    body: combined,
  };
  simplify(rng, new_module, binding, parent, container)
}

/// Recursively pull `Pattern::Start` bodies out of `Module` / `Grammar`
/// wrappers. Mirrors `extractStart` and powers
/// `Model::add_tag_content('#Document', ...)` later in the chain.
pub fn extract_start(items: &[Pattern]) -> Vec<Pattern> {
  let mut out = Vec::new();
  for item in items {
    match item {
      Pattern::Start { body } => out.extend(body.iter().cloned()),
      Pattern::Module { body, .. } | Pattern::Grammar { body, .. } => {
        out.extend(extract_start(body));
      },
      _ => {},
    }
  }
  out
}

// ----- unit tests ---------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;
  use crate::common::relaxng::scan::scan_string;

  fn simplify_xml(xml: &str) -> (Relaxng, Vec<Pattern>) {
    let mut rng = Relaxng::default();
    let raw = scan_string(&mut rng, xml).expect("scan");
    let simp = simplify_top(&mut rng, raw);
    (rng, simp)
  }

  #[test]
  fn simplify_collapses_singleton_group() {
    let inner = Pattern::Element { name: "x".into(), body: vec![] };
    let combo = Pattern::Combination { op: CombineOp::Group, body: vec![inner.clone()] };
    let result = simplify_combination(combo);
    assert!(matches!(result, Pattern::Element { ref name, .. } if name == "x"));
  }

  #[test]
  fn simplify_flattens_nested_choice() {
    let inner_choice = Pattern::Combination {
      op:   CombineOp::Choice,
      body: vec![
        Pattern::Element { name: "a".into(), body: vec![] },
        Pattern::Element { name: "b".into(), body: vec![] },
      ],
    };
    let outer = Pattern::Combination {
      op:   CombineOp::Choice,
      body: vec![inner_choice, Pattern::Element { name: "c".into(), body: vec![] }],
    };
    let result = simplify_combination(outer);
    let body = match &result {
      Pattern::Combination { op: CombineOp::Choice, body } => body,
      other => panic!("expected flat Choice, got {:?}", other),
    };
    assert_eq!(body.len(), 3);
  }

  #[test]
  fn simplify_records_modules_in_document_order() {
    // `scan_string` returns a flat Vec<Pattern> (no Module wrapper —
    // that's what `scan_external` adds for files); to exercise the
    // Module branch of simplify, we wrap manually here.
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <define name="A"><element name="a"><empty/></element></define>
        <define name="B"><element name="b"><empty/></element></define>
      </grammar>
    "#;
    let mut rng = Relaxng::default();
    let raw = scan_string(&mut rng, xml).expect("scan");
    let wrapped = vec![Pattern::Module {
      name: "wrapper".into(),
      body: raw,
    }];
    let _ = simplify_top(&mut rng, wrapped);
    assert!(!rng.modules.is_empty(), "modules should be recorded");
    let names: Vec<&str> = rng
      .modules
      .iter()
      .filter_map(|m| match m {
        Pattern::Module { name, .. } => Some(name.as_str()),
        _ => None,
      })
      .collect();
    assert_eq!(names, vec!["wrapper"]);
  }

  #[test]
  fn simplify_singleton_element_def_records_elementdefs() {
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <define name="MY"><element name="my-el"><empty/></element></define>
      </grammar>
    "#;
    let (rng, _) = simplify_xml(xml);
    // Binding is the synthesized `grammar1`, so qname is `grammar1:MY`.
    assert_eq!(rng.elementdefs.get("grammar1:MY"), Some(&"my-el".to_string()));
    assert_eq!(rng.element_reverse_defs.get("my-el"), Some(&"grammar1:MY".to_string()));
  }

  #[test]
  fn simplify_complex_def_recorded_in_defs() {
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <define name="X">
          <choice>
            <element name="a"><empty/></element>
            <element name="b"><empty/></element>
          </choice>
        </define>
      </grammar>
    "#;
    let (rng, _) = simplify_xml(xml);
    assert!(rng.defs.contains_key("grammar1:X"));
    assert_eq!(
      rng.def_combiner.get("grammar1:X").copied(),
      Some(DefCombiner::Group)
    );
  }

  #[test]
  fn simplify_combine_choice_accumulates() {
    // Use `<ref>` bodies — single-element bodies of a plain `<define>`
    // hit the elementdefs shortcut and bypass the defs table, so we'd
    // never see them merged. `<ref>` bodies sidestep that path.
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <define name="X"><ref name="A"/></define>
        <define name="X" combine="choice"><ref name="B"/></define>
      </grammar>
    "#;
    let (rng, _) = simplify_xml(xml);
    let combined = rng.defs.get("grammar1:X").expect("defs entry");
    let body = match combined {
      Pattern::Combination { op: CombineOp::Choice, body } => body,
      other => panic!("expected Choice combination, got {:?}", other),
    };
    let qnames: Vec<&str> = body
      .iter()
      .filter_map(|p| match p {
        Pattern::Ref { qname } => Some(qname.as_str()),
        _ => None,
      })
      .collect();
    assert!(qnames.contains(&"grammar1:A"), "A ref missing: {:?}", qnames);
    assert!(qnames.contains(&"grammar1:B"), "B ref missing: {:?}", qnames);
    assert_eq!(rng.def_combiner.get("grammar1:X").copied(), Some(DefCombiner::Choice));
  }

  #[test]
  fn simplify_records_uses_name() {
    let xml = r#"
      <grammar xmlns="http://relaxml.org/ns/structure/1.0"
               xmlns:rng="http://relaxng.org/ns/structure/1.0">
      </grammar>
    "#;
    // Using a real example that exercises ref tracking:
    let xml = r#"
      <grammar xmlns="http://relaxng.org/ns/structure/1.0">
        <define name="P">
          <element name="p"><ref name="Q"/></element>
        </define>
        <define name="Q"><text/></define>
      </grammar>
    "#;
    let (rng, _) = simplify_xml(xml);
    // P body recorded an Element whose body has a Ref to grammar1:Q.
    let p_uses = rng.uses_name.get("grammar1:Q");
    assert!(
      p_uses.is_some(),
      "uses_name for grammar1:Q should be populated, got {:?}",
      rng.uses_name.keys().collect::<Vec<_>>()
    );
    // The Ref appears inside element:p (not pattern:grammar1:P, because
    // simplify resets the container when entering an Element).
    let containers = p_uses.unwrap();
    assert!(
      containers.contains("element:p"),
      "expected Q usage under element:p, got {:?}",
      containers
    );
  }

  #[test]
  fn simplify_extract_start_descends_grammar_and_module() {
    let inner = Pattern::Start {
      body: vec![Pattern::Element { name: "root".into(), body: vec![] }],
    };
    let nested = vec![Pattern::Module {
      name: "m".into(),
      body: vec![Pattern::Grammar { name: "g".into(), body: vec![inner] }],
    }];
    let starts = extract_start(&nested);
    assert_eq!(starts.len(), 1);
    assert!(matches!(starts[0], Pattern::Element { ref name, .. } if name == "root"));
  }

  #[test]
  fn simplify_override_drops_overridden_def_and_keeps_replacement() {
    // Build the AST manually since trang typically flattens includes,
    // and `<include>` with overrides goes through scan_grammar_item ->
    // Pattern::Override, which we want to verify here directly.
    let module = Pattern::Module {
      name: "m".into(),
      body: vec![
        Pattern::Def {
          combiner: DefCombiner::Group,
          name:     "X".into(),
          body:     vec![Pattern::Element {
            name: "original".into(),
            body: vec![],
          }],
        },
        Pattern::Def {
          combiner: DefCombiner::Group,
          name:     "Y".into(),
          body:     vec![Pattern::Element {
            name: "y-el".into(),
            body: vec![],
          }],
        },
      ],
    };
    let override_pat = Pattern::Override {
      module:       Box::new(module),
      replacements: vec![Pattern::Def {
        combiner: DefCombiner::Group,
        name:     "X".into(),
        body:     vec![Pattern::Element {
          name: "replacement".into(),
          body: vec![],
        }],
      }],
    };

    let mut rng = Relaxng::default();
    let _ = simplify(&mut rng, override_pat, "g", "", None);

    // After simplification, only the "replacement" element wins for X.
    assert_eq!(rng.elementdefs.get("g:X"), Some(&"replacement".to_string()));
    // Y survives untouched.
    assert_eq!(rng.elementdefs.get("g:Y"), Some(&"y-el".to_string()));
    assert_eq!(rng.element_reverse_defs.get("original"), None);
  }
}

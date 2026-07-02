//! Content MathML rendering rules.
//!
//! Port of `LaTeXML::Post::MathML::Content` (31 lines) + the content
//! portion of `LaTeXML::Post::MathML` (main module, ~400 lines).
//! Converts XMath nodes to Content MathML elements (apply, ci, cn, csymbol, etc.).
//!
//! Content MathML aims to capture the semantic structure of math:
//! - Variables → `m:ci`
//! - Numbers → `m:cn` (with type: integer, float, etc.)
//! - Known symbols → `m:csymbol` (with cd attribute)
//! - Standard symbols → dedicated elements (`m:plus`, `m:eq`, `m:int`, etc.)
//! - Application → `m:apply`
//! - XMDual → follows the content branch

use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;

use crate::document::{NodeData, PostDocument, element_children};

/// Well-known meaning → Content MathML element mappings.
///
/// Port of the Token:?:* DefMathML content entries.
/// Also accessible via `meaning_to_cmml_element_pub` for cross-module access.
pub(crate) fn meaning_to_cmml_element_pub(meaning: &str) -> Option<&'static str> {
  meaning_to_cmml_element(meaning)
}

fn meaning_to_cmml_element(meaning: &str) -> Option<&'static str> {
  match meaning {
    // Arithmetic
    "plus" => Some("m:plus"),
    "minus" | "uminus" => Some("m:minus"),
    "times" => Some("m:times"),
    "divide" => Some("m:divide"),
    "power" => Some("m:power"),
    "quotient" => Some("m:quotient"),
    "factorial" => Some("m:factorial"),
    "remainder" => Some("m:rem"),
    "maximum" => Some("m:max"),
    "minimum" => Some("m:min"),
    "gcd" => Some("m:gcd"),
    "lcm" => Some("m:lcm"),
    "absolute-value" => Some("m:abs"),
    "conjugate" => Some("m:conjugate"),
    "argument" => Some("m:arg"),
    "real-part" => Some("m:real"),
    "imaginary-part" => Some("m:imaginary"),
    "floor" => Some("m:floor"),
    "ceiling" => Some("m:ceiling"),
    // Logic
    "and" => Some("m:and"),
    "or" => Some("m:or"),
    "xor" => Some("m:xor"),
    "not" => Some("m:not"),
    "implies" => Some("m:implies"),
    "forall" => Some("m:forall"),
    "exists" => Some("m:exists"),
    // Relations
    "equals" => Some("m:eq"),
    "not-equals" => Some("m:neq"),
    "greater-than" => Some("m:gt"),
    "less-than" => Some("m:lt"),
    "greater-than-or-equals" => Some("m:geq"),
    "less-than-or-equals" => Some("m:leq"),
    "equivalent-to" => Some("m:equivalent"),
    "approximately-equals" => Some("m:approx"),
    "factor-of" => Some("m:factorof"),
    // Calculus
    "integral" => Some("m:int"),
    "differential" => Some("m:diff"),
    "partial-differential" => Some("m:partialdiff"),
    "divergence" => Some("m:divergence"),
    "gradient" => Some("m:grad"),
    "curl" => Some("m:curl"),
    "laplacian" => Some("m:laplacian"),
    // Sets
    "union" => Some("m:union"),
    "intersection" => Some("m:intersect"),
    "element-of" => Some("m:in"),
    "not-element-of" => Some("m:notin"),
    "subset-of" | "subset-of-or-equals" => Some("m:subset"),
    "subset-of-and-not-equals" => Some("m:prsubset"),
    "set-minus" => Some("m:setdiff"),
    "cardinality" => Some("m:card"),
    "cartesian-product" => Some("m:cartesianproduct"),
    // Sequences and Series
    "sum" => Some("m:sum"),
    "prod" => Some("m:prod"),
    "limit" => Some("m:limit"),
    "tends-to" => Some("m:tendsto"),
    // Elementary Classical Functions (trig, exp, log)
    "exponential" => Some("m:exp"),
    "natural-logarithm" => Some("m:ln"),
    "logarithm" => Some("m:log"),
    "sine" => Some("m:sin"),
    "cosine" => Some("m:cos"),
    "tangent" => Some("m:tan"),
    "secant" => Some("m:sec"),
    "cosecant" => Some("m:csc"),
    "cotangent" => Some("m:cot"),
    "hyperbolic-sine" => Some("m:sinh"),
    "hyperbolic-cosine" => Some("m:cosh"),
    "hyperbolic-tangent" => Some("m:tanh"),
    "hyperbolic-secant" => Some("m:sech"),
    "hyperbolic-cosecant" => Some("m:csch"),
    "hyperbolic-cotantent" => Some("m:coth"),
    "inverse-sine" => Some("m:arcsin"),
    "inverse-cosine" => Some("m:arccos"),
    "inverse-tangent" => Some("m:arctan"),
    "inverse-secant" => Some("m:arcsec"),
    "inverse-cosecant" => Some("m:arccsc"),
    "inverse-cotangent" => Some("m:arccot"),
    "inverse-hyperbolic-sine" => Some("m:arcsinh"),
    "inverse-hyperbolic-cosine" => Some("m:arccosh"),
    "inverse-hyperbolic-tangent" => Some("m:arctanh"),
    "inverse-hyperbolic-secant" => Some("m:arcsech"),
    "inverse-hyperbolic-cosecant" => Some("m:arccsch"),
    "inverse-hyperbolic-cotangent" => Some("m:arccoth"),
    // Statistics
    "mean" => Some("m:mean"),
    "standard-deviation" => Some("m:sdev"),
    "variance" => Some("m:var"),
    "median" => Some("m:median"),
    "mode" => Some("m:mode"),
    "moment" => Some("m:moment"),
    // Linear Algebra
    "determinant" => Some("m:determinant"),
    "transpose" => Some("m:transpose"),
    "selector" => Some("m:selector"),
    "vector-product" => Some("m:vectorproduct"),
    "scalar-product" => Some("m:scalarproduct"),
    "outer-product" => Some("m:outerproduct"),
    // Constants
    "integers" => Some("m:integers"),
    "reals" => Some("m:reals"),
    "rationals" => Some("m:rationals"),
    "numbers" => Some("m:naturalnumbers"),
    "complexes" => Some("m:complexes"),
    "primes" => Some("m:primes"),
    "exponential-e" => Some("m:exponentiale"),
    "imaginary-i" => Some("m:imaginaryi"),
    "notanumber" => Some("m:notanumber"),
    "true" => Some("m:true"),
    "false" => Some("m:false"),
    "empty-set" => Some("m:emptyset"),
    "circular-pi" => Some("m:pi"),
    "Euler-constant" => Some("m:eulergamma"),
    "infinity" => Some("m:infinity"),
    // Other
    "inverse" => Some("m:inverse"),
    "lambda" => Some("m:lambda"),
    "compose" => Some("m:compose"),
    "identity" => Some("m:ident"),
    "domain" => Some("m:domain"),
    "codomain" => Some("m:codomain"),
    "image" => Some("m:image"),
    "square-root" => Some("m:root"),
    _ => None,
  }
}

/// Convert an XMath tree to Content MathML.
///
/// Port of `MathML::Content::convertNode` + `cmml_top`.
pub fn convert_to_cmml(doc: &PostDocument, xmath: &Node) -> NodeData {
  reset_share_counter();
  CMML_DEPTH.with(|d| d.set(0));
  CMML_PATH.with(|p| p.borrow_mut().clear());
  cmml_contents(doc, xmath)
}

// Recursion guard. Perl uses `no warnings 'recursion'` and relies on
// its native stack; we cap to avoid blowing the 256 MB worker stack
// when malformed/cyclic XMath (e.g. duplicate xml:id whose target
// loops via XMRef) drives `cmml` into unbounded descent. Two failure
// modes seen on the second-500K canvas (stage_53/54):
//
// 1. Plain stack overflow on linearly-deep XMath — witnesses arXiv:1505.06709, 1505.06978 (both
//    emitted `Info:malformed:id Duplicated attribute xml:id` then overflowed during
//    MathML[Content]). Cap at `CMML_MAX_DEPTH` (256, well above any legitimate XMApp nesting we've
//    measured).
//
// 2. Cyclic XMRef chains: an `ltx:XMRef` whose `find_node_by_id` target's subtree contains another
//    XMRef back into an ancestor of the current cmml frame. Linear depth cap alone is not enough —
//    each cycle iteration doubles the generated NodeData::Element subtree, exhausting the 6 GB
//    worker memory budget long before the depth limit fires. Witness: arXiv:1508.06324 (stage_54) —
//    440 maths, OOMs with 5-byte alloc failure mid-CMML. Track a `CMML_PATH` set of node pointers
//    along the current recursion path; if we visit a node already on the path, return
//    `cmml_error("cycle")` and unwind.
const CMML_MAX_DEPTH: u32 = 256;
thread_local! {
  static CMML_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
  static CMML_PATH: std::cell::RefCell<rustc_hash::FxHashSet<usize>>
    = std::cell::RefCell::new(rustc_hash::FxHashSet::default());
}

enum CmmlEnter {
  Ok,
  DepthExceeded,
  Cycle,
}

fn cmml_enter(node: &Node) -> CmmlEnter {
  let ptr = node.node_ptr() as usize;
  let in_path = CMML_PATH.with(|p| !p.borrow_mut().insert(ptr));
  if in_path {
    // Diagnostic: dump the current path so the math-parser bisect can see
    // exactly which node-ids form the cycle. Behind an env so production
    // canvas runs stay silent. Format: `cmml_cycle: <id_or_tag>(<ptr>) <-
    // ... <- <cycling node>` — most recent ancestor first; the cycling
    // node is repeated at the end so the loop is visually obvious.
    if std::env::var_os("LATEXML_CMML_TRACE_CYCLE").is_some() {
      let id = node
        .get_attribute("xml:id")
        .or_else(|| node.get_attribute("id"))
        .unwrap_or_else(|| format!("({})", node.get_name()));
      eprintln!("cmml_cycle: re-entering {} ptr=0x{:x}", id, ptr);
    }
    return CmmlEnter::Cycle;
  }
  CMML_DEPTH.with(|d| {
    let cur = d.get();
    if cur >= CMML_MAX_DEPTH {
      // Pop the path entry we just inserted before returning.
      CMML_PATH.with(|p| {
        p.borrow_mut().remove(&ptr);
      });
      CmmlEnter::DepthExceeded
    } else {
      d.set(cur + 1);
      CmmlEnter::Ok
    }
  })
}

fn cmml_exit(node: &Node) {
  CMML_PATH.with(|p| {
    p.borrow_mut().remove(&(node.node_ptr() as usize));
  });
  CMML_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
}

/// Convert the contents of a node (which normally has a single child).
///
/// Port of `cmml_contents`.
fn cmml_contents(doc: &PostDocument, node: &Node) -> NodeData {
  let children = element_children(node);
  if children.is_empty() {
    cmml_error("missing-subexpression")
  } else if children.len() == 1 {
    cmml(doc, &children[0])
  } else {
    cmml_unparsed(doc, &children)
  }
}

/// Core dispatch: convert a single XMath node to Content MathML.
///
/// Port of `cmml` + `cmml_internal`.
fn cmml(doc: &PostDocument, node: &Node) -> NodeData {
  match cmml_enter(node) {
    CmmlEnter::Cycle => return cmml_error("cycle"),
    CmmlEnter::DepthExceeded => return cmml_error("recursion-depth-exceeded"),
    CmmlEnter::Ok => {},
  }
  let result = cmml_impl(doc, node);
  cmml_exit(node);
  result
}

fn cmml_impl(doc: &PostDocument, node: &Node) -> NodeData {
  let tag = doc.get_qname(node).unwrap_or_default();

  // Follow XMRef
  if tag == "ltx:XMRef" {
    if let Some(idref) = node.get_attribute("idref") {
      if let Some(target) = doc.find_node_by_id(&idref) {
        return cmml(doc, target);
      }
    }
    return cmml_error("unresolved-reference");
  }

  match tag.as_str() {
    "ltx:XMDual" => {
      let children = element_children(node);
      if !children.is_empty() {
        cmml(doc, &children[0]) // Content branch
      } else {
        cmml_error("empty-dual")
      }
    },
    "ltx:XMWrap" | "ltx:XMArg" => cmml_contents(doc, node),
    "ltx:XMApp" => {
      // Check if XMApp has a meaning (token-like application)
      if let Some(meaning) = node.get_attribute("meaning") {
        return cmml_token_by_meaning(&meaning, node);
      }
      // Check if role=ID (decorated symbol)
      if node.get_attribute("role").as_deref() == Some("ID") {
        return cmml_decorated_symbol(doc, node);
      }
      // Normal application
      let children = element_children(node);
      if children.is_empty() {
        return cmml_error("missing-operator");
      }
      let op = &children[0];
      let args = &children[1..];

      // Realize operator
      let rop = if doc.is_qname(op, "ltx:XMRef") {
        op.get_attribute("idref")
          .and_then(|id| doc.find_node_by_id(&id).cloned())
          .unwrap_or_else(|| op.clone())
      } else {
        op.clone()
      };

      let meaning = rop.get_attribute("meaning").unwrap_or_default();
      let _role = rop.get_attribute("role").unwrap_or_default();

      // Special meanings with dedicated structure
      match meaning.as_str() {
        // Perl `Apply:?:multirelation` cmml (L1713-1729): chained relations
        // a<b<c become pairwise applies sharing the middle operands, under
        // m:and when there is more than one.
        "multirelation" if !args.is_empty() => {
          let lhs0 = cmml(doc, &args[0]);
          if args.len() == 1 {
            return lhs0;
          }
          let mut lhs = lhs0;
          let mut relations = Vec::new();
          let mut i = 1;
          while i + 1 < args.len() + 1 && i < args.len() {
            let rel = &args[i];
            let Some(rhs) = args.get(i + 1) else { break };
            relations.push(NodeData::Element {
              tag:        "m:apply".to_string(),
              attributes: None,
              children:   vec![cmml(doc, rel), lhs, cmml_shared(doc, rhs)],
            });
            lhs = cmml_share(rhs);
            i += 2;
          }
          if relations.len() > 1 {
            let mut children = vec![NodeData::Element {
              tag:        "m:and".to_string(),
              attributes: None,
              children:   vec![],
            }];
            children.extend(relations);
            NodeData::Element {
              tag: "m:apply".to_string(),
              attributes: None,
              children,
            }
          } else {
            relations
              .pop()
              .unwrap_or_else(|| cmml_error("multirelation"))
          }
        },
        // Perl `Apply:?:less-than-or-approximately-equals` →
        // cmml_or_compose(['m:lt','m:approx']) (L1436-1445): each relation
        // applied to the (shared) operands, disjoined under m:or.
        "less-than-or-approximately-equals" if !args.is_empty() => {
          let first: Vec<NodeData> = args.iter().map(|a| cmml_shared(doc, a)).collect();
          let second: Vec<NodeData> = args.iter().map(cmml_share).collect();
          let mk = |op: &str, ops: Vec<NodeData>| {
            let mut children = vec![NodeData::Element {
              tag:        op.to_string(),
              attributes: None,
              children:   vec![],
            }];
            children.extend(ops);
            NodeData::Element {
              tag: "m:apply".to_string(),
              attributes: None,
              children,
            }
          };
          NodeData::Element {
            tag:        "m:or".to_string(),
            attributes: None,
            children:   vec![mk("m:lt", first), mk("m:approx", second)],
          }
        },
        "square-root" if !args.is_empty() => NodeData::Element {
          tag:        "m:apply".to_string(),
          attributes: None,
          children:   vec![
            NodeData::Element {
              tag:        "m:root".to_string(),
              attributes: None,
              children:   vec![],
            },
            cmml(doc, &args[0]),
          ],
        },
        "nth-root" if args.len() >= 2 => NodeData::Element {
          tag:        "m:apply".to_string(),
          attributes: None,
          children:   vec![
            NodeData::Element {
              tag:        "m:root".to_string(),
              attributes: None,
              children:   vec![],
            },
            NodeData::Element {
              tag:        "m:degree".to_string(),
              attributes: None,
              // Perl L1648: degree is args[0], radicand args[1] (was swapped).
              children:   vec![cmml(doc, &args[0])],
            },
            cmml(doc, &args[1]),
          ],
        },
        "set" => {
          let items: Vec<NodeData> = args.iter().map(|a| cmml(doc, a)).collect();
          NodeData::Element {
            tag:        "m:set".to_string(),
            attributes: None,
            children:   items,
          }
        },
        "list" => {
          let items: Vec<NodeData> = args.iter().map(|a| cmml(doc, a)).collect();
          NodeData::Element {
            tag:        "m:list".to_string(),
            attributes: None,
            children:   items,
          }
        },
        "vector" => {
          let items: Vec<NodeData> = args.iter().map(|a| cmml(doc, a)).collect();
          NodeData::Element {
            tag:        "m:vector".to_string(),
            attributes: None,
            children:   items,
          }
        },
        // Interval types
        "open-interval" => {
          let items: Vec<NodeData> = args.iter().map(|a| cmml(doc, a)).collect();
          NodeData::Element {
            tag:        "m:interval".to_string(),
            attributes: Some(HashMap::from_iter([(
              "closure".to_string(),
              "open".to_string(),
            )])),
            children:   items,
          }
        },
        "closed-interval" => {
          let items: Vec<NodeData> = args.iter().map(|a| cmml(doc, a)).collect();
          NodeData::Element {
            tag:        "m:interval".to_string(),
            attributes: Some(HashMap::from_iter([(
              "closure".to_string(),
              "closed".to_string(),
            )])),
            children:   items,
          }
        },
        "closed-open-interval" => {
          let items: Vec<NodeData> = args.iter().map(|a| cmml(doc, a)).collect();
          NodeData::Element {
            tag:        "m:interval".to_string(),
            attributes: Some(HashMap::from_iter([(
              "closure".to_string(),
              "closed-open".to_string(),
            )])),
            children:   items,
          }
        },
        "open-closed-interval" => {
          let items: Vec<NodeData> = args.iter().map(|a| cmml(doc, a)).collect();
          NodeData::Element {
            tag:        "m:interval".to_string(),
            attributes: Some(HashMap::from_iter([(
              "closure".to_string(),
              "open-closed".to_string(),
            )])),
            children:   items,
          }
        },
        // Complement operations: reverse argument order
        "contains" | "superset-of" | "superset-of-or-equals" | "superset-of-and-not-equals" => {
          let cmml_op = match meaning.as_str() {
            "contains" => "m:in",
            "superset-of" | "superset-of-or-equals" => "m:subset",
            "superset-of-and-not-equals" => "m:prsubset",
            _ => "m:subset",
          };
          let mut reversed_args: Vec<NodeData> = args.iter().rev().map(|a| cmml(doc, a)).collect();
          reversed_args.insert(0, NodeData::Element {
            tag:        cmml_op.to_string(),
            attributes: None,
            children:   vec![],
          });
          NodeData::Element {
            tag:        "m:apply".to_string(),
            attributes: None,
            children:   reversed_args,
          }
        },
        // Not-contains: not(in(reversed))
        "not-contains" => {
          let mut reversed_args: Vec<NodeData> = args.iter().rev().map(|a| cmml(doc, a)).collect();
          reversed_args.insert(0, NodeData::Element {
            tag:        "m:notin".to_string(),
            attributes: None,
            children:   vec![],
          });
          NodeData::Element {
            tag:        "m:apply".to_string(),
            attributes: None,
            children:   reversed_args,
          }
        },
        // Not-approximately-equals: not(approx(args))
        "not-approximately-equals" => {
          let inner_args: Vec<NodeData> = args.iter().map(|a| cmml(doc, a)).collect();
          let mut apply_children = vec![NodeData::Element {
            tag:        "m:approx".to_string(),
            attributes: None,
            children:   vec![],
          }];
          apply_children.extend(inner_args);
          NodeData::Element {
            tag:        "m:apply".to_string(),
            attributes: None,
            children:   vec![
              NodeData::Element {
                tag:        "m:not".to_string(),
                attributes: None,
                children:   vec![],
              },
              NodeData::Element {
                tag:        "m:apply".to_string(),
                attributes: None,
                children:   apply_children,
              },
            ],
          }
        },
        // Hack for definite integrals with explicit limits
        "hack-definite-integral" if args.len() >= 4 => NodeData::Element {
          tag:        "m:apply".to_string(),
          attributes: None,
          children:   vec![
            NodeData::Element {
              tag:        "m:int".to_string(),
              attributes: None,
              children:   vec![],
            },
            NodeData::Element {
              tag:        "m:bvar".to_string(),
              attributes: None,
              children:   vec![cmml(doc, &args[3])],
            },
            NodeData::Element {
              tag:        "m:lowlimit".to_string(),
              attributes: None,
              children:   vec![cmml(doc, &args[0])],
            },
            NodeData::Element {
              tag:        "m:uplimit".to_string(),
              attributes: None,
              children:   vec![cmml(doc, &args[1])],
            },
            cmml(doc, &args[2]),
          ],
        },
        // Formulae: sequence of expressions
        "formulae" => {
          let items: Vec<NodeData> = args.iter().map(|a| cmml(doc, a)).collect();
          let mut children = vec![NodeData::Element {
            tag:        "m:csymbol".to_string(),
            attributes: Some(HashMap::from_iter([(
              "cd".to_string(),
              "ambiguous".to_string(),
            )])),
            children:   vec![NodeData::Text("formulae-sequence".to_string())],
          }];
          children.extend(items);
          NodeData::Element {
            tag: "m:apply".to_string(),
            attributes: None,
            children,
          }
        },
        _ => {
          // Generic application: <m:apply> op args... </m:apply>
          let mut apply_children = vec![cmml(doc, op)];
          apply_children.extend(args.iter().map(|a| cmml(doc, a)));
          NodeData::Element {
            tag:        "m:apply".to_string(),
            attributes: None,
            children:   apply_children,
          }
        },
      }
    },
    "ltx:XMTok" => cmml_leaf(doc, node),
    "ltx:XMHint" => {
      // Hints are ignored in Content MathML
      NodeData::Text(String::new())
    },
    "ltx:XMArray" => cmml_array(doc, node),
    "ltx:XMText" => cmml_decorated_symbol(doc, node),
    _ => cmml_decorated_symbol(doc, node),
  }
}

/// Convert an XMTok to a Content MathML leaf element.
///
/// Port of `cmml_leaf`.
fn cmml_leaf(_doc: &PostDocument, node: &Node) -> NodeData {
  let meaning = node.get_attribute("meaning");
  let role = node.get_attribute("role").unwrap_or_default();

  if let Some(ref m) = meaning {
    // Known meaning → check for dedicated MathML element
    if let Some(element_name) = meaning_to_cmml_element(m) {
      return NodeData::Element {
        tag:        element_name.to_string(),
        attributes: None,
        children:   vec![],
      };
    }

    // Has omcd → csymbol with cd
    if let Some(cd) = node.get_attribute("omcd") {
      return NodeData::Element {
        tag:        "m:csymbol".to_string(),
        attributes: Some(HashMap::from_iter([("cd".to_string(), cd)])),
        children:   vec![NodeData::Text(m.clone())],
      };
    }

    // Number with meaning
    if role == "NUMBER" {
      // Perl /^[+-]?\d+$/ — arbitrary-length integers, unlike i64::parse.
      let cn_type = if is_perl_integer(m) {
        "integer"
      } else {
        "float"
      };
      return NodeData::Element {
        tag:        "m:cn".to_string(),
        attributes: Some(HashMap::from_iter([(
          "type".to_string(),
          cn_type.to_string(),
        )])),
        children:   vec![NodeData::Text(m.clone())],
      };
    }

    // Default: csymbol with latexml cd
    return NodeData::Element {
      tag:        "m:csymbol".to_string(),
      attributes: Some(HashMap::from_iter([(
        "cd".to_string(),
        "latexml".to_string(),
      )])),
      children:   vec![NodeData::Text(m.clone())],
    };
  }

  // No meaning: variable or number
  let content = node.get_content();
  match role.as_str() {
    "NUMBER" => {
      let cn_type = if is_perl_integer(&content) {
        "integer"
      } else {
        "float"
      };
      NodeData::Element {
        tag:        "m:cn".to_string(),
        attributes: Some(HashMap::from_iter([(
          "type".to_string(),
          cn_type.to_string(),
        )])),
        children:   vec![NodeData::Text(content)],
      }
    },
    "SUPERSCRIPTOP" => NodeData::Element {
      tag:        "m:csymbol".to_string(),
      attributes: Some(HashMap::from_iter([(
        "cd".to_string(),
        "ambiguous".to_string(),
      )])),
      children:   vec![NodeData::Text("superscript".to_string())],
    },
    "SUBSCRIPTOP" => NodeData::Element {
      tag:        "m:csymbol".to_string(),
      attributes: Some(HashMap::from_iter([(
        "cd".to_string(),
        "ambiguous".to_string(),
      )])),
      children:   vec![NodeData::Text("subscript".to_string())],
    },
    _ => {
      // Variable / identifier (Perl cmml_leaf L1388-1391: stylized ci).
      let name = if content.is_empty() {
        node
          .get_attribute("name")
          .unwrap_or_else(|| "?".to_string())
      } else {
        content
      };
      NodeData::Element {
        tag:        "m:ci".to_string(),
        attributes: None,
        children:   vec![NodeData::Text(stylize_ci_content(node, name))],
      }
    },
  }
}

/// Convert a token by its meaning attribute (for XMApp with meaning).
fn cmml_token_by_meaning(meaning: &str, _node: &Node) -> NodeData {
  if let Some(element_name) = meaning_to_cmml_element(meaning) {
    NodeData::Element {
      tag:        element_name.to_string(),
      attributes: None,
      children:   vec![],
    }
  } else {
    NodeData::Element {
      tag:        "m:csymbol".to_string(),
      attributes: Some(HashMap::from_iter([(
        "cd".to_string(),
        "latexml".to_string(),
      )])),
      children:   vec![NodeData::Text(meaning.to_string())],
    }
  }
}

// Per-formula counter for generated share ids (Perl uses the document
// idcache to find the next free `sh<n>`; ancestor-id scoping makes a
// per-formula counter equivalent). Reset by `convert_to_cmml`.
thread_local! {
  static SH_COUNTER: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}
pub(super) fn reset_share_counter() { SH_COUNTER.with(|c| c.set(0)); }

/// Port of Perl `generateNodeID` + `cmml_shared` (MathML.pm L1420-1424,
/// Post.pm generateNodeID): make sure the XMath node has an xml:id (and a
/// fragid mirroring its ancestor's) so it can be shared, then convert.
fn cmml_shared(doc: &PostDocument, node: &Node) -> NodeData {
  if crate::document::get_xml_id(node).is_none() {
    // Closest ancestor with an id (Perl walks parentNode chain).
    // NB xml:id is namespaced — always read via get_xml_id (WISDOM).
    let mut parent = node.get_parent();
    while let Some(ref p) = parent {
      if crate::document::get_xml_id(p).is_some() {
        break;
      }
      parent = p.get_parent();
    }
    let n = SH_COUNTER.with(|c| {
      let v = c.get() + 1;
      c.set(v);
      v
    });
    let pid = parent
      .as_ref()
      .and_then(crate::document::get_xml_id)
      .map(|id| format!("{id}."))
      .unwrap_or_default();
    let mut handle = node.clone();
    handle.set_attribute("xml:id", &format!("{pid}sh{n}")).ok();
    if let Some(pfragid) = parent.as_ref().and_then(|p| p.get_attribute("fragid")) {
      handle
        .set_attribute("fragid", &format!("{pfragid}.sh{n}"))
        .ok();
    }
  }
  cmml(doc, node)
}

/// Port of Perl `cmml_share` (L1426-1434): reference a shared operand.
fn cmml_share(node: &Node) -> NodeData {
  if let Some(fragid) = node.get_attribute("fragid") {
    // NB Perl appends $MATHPROCESSOR->IDSuffix; our cmml runs with the
    // primary suffix '' (parallel-markup suffix wiring is an audit residual).
    NodeData::Element {
      tag:        "m:share".to_string(),
      attributes: Some(HashMap::from_iter([(
        "href".to_string(),
        format!("#{fragid}"),
      )])),
      children:   vec![],
    }
  } else {
    crate::Warn!(
      "expected",
      "fragid",
      "Shared node is missing fragid (multirelation/or-compose share)"
    );
    NodeData::Element {
      tag:        "m:share".to_string(),
      attributes: None,
      children:   vec![],
    }
  }
}

/// Perl integer test `/^[+-]?\d+$/` (arbitrary length, unlike i64::parse).
fn is_perl_integer(s: &str) -> bool {
  let digits = s.strip_prefix(['+', '-']).unwrap_or(s);
  !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// The identifier-content half of Perl `stylizeContent($item, 'm:ci')`
/// (cmml_leaf L1388-1391): font → mathvariant → plane1 unicode conversion;
/// a variant that could not be baked into characters prefixes the content
/// ("bold-x"). This is why Perl's cmml says `<ci>𝑥</ci>`, not `<ci>x</ci>`.
fn stylize_ci_content(node: &Node, text: String) -> String {
  let Some(font) = node.get_attribute("font") else {
    return text;
  };
  let variant = crate::unicode::unicode_mathvariant(&font);
  if variant.is_empty() || variant == "normal" {
    return text;
  }
  if let Some(u) = crate::unicode::unicode_convert(&text, variant)
    && !u.is_empty()
  {
    return u;
  }
  format!("{variant}-{text}")
}

/// Convert a "decorated symbol" — an XMApp with role=ID treated as a ci.
///
/// Port of `cmml_decoratedSymbol` (L1396-1404): with a meaning it is a
/// csymbol (cd from omcd, default latexml); otherwise a ci whose content is
/// the node's PRESENTATION conversion.
fn cmml_decorated_symbol(doc: &PostDocument, node: &Node) -> NodeData {
  if let Some(meaning) = node.get_attribute("meaning") {
    let cd = node
      .get_attribute("omcd")
      .unwrap_or_else(|| "latexml".to_string());
    return NodeData::Element {
      tag:        "m:csymbol".to_string(),
      attributes: Some(HashMap::from_iter([("cd".to_string(), cd)])),
      children:   vec![NodeData::Text(meaning)],
    };
  }
  NodeData::Element {
    tag:        "m:ci".to_string(),
    attributes: None,
    children:   vec![super::presentation::pmml_for_ci(doc, node)],
  }
}

/// Convert an XMArray to Content MathML.
///
/// Port of `Array:?:cases` DefMathML content handler.
fn cmml_array(doc: &PostDocument, node: &Node) -> NodeData {
  let meaning = node.get_attribute("meaning").unwrap_or_default();

  if meaning == "cases" {
    // Piecewise construct
    let mut pieces = Vec::new();
    let mut otherwises = Vec::new();

    for row in element_children(node) {
      let items = element_children(&row);
      if items.is_empty() {
        continue;
      }
      if items.len() == 1 {
        otherwises.push(cmml_contents(doc, &items[0]));
      } else if items[1].get_content().contains("otherwise") {
        otherwises.push(cmml_contents(doc, &items[0]));
      } else {
        pieces.push(NodeData::Element {
          tag:        "m:piece".to_string(),
          attributes: None,
          children:   vec![cmml_contents(doc, &items[0]), cmml_contents(doc, &items[1])],
        });
      }
    }

    if let Some(ow) = otherwises.into_iter().next() {
      pieces.push(NodeData::Element {
        tag:        "m:otherwise".to_string(),
        attributes: None,
        children:   vec![ow],
      });
    }

    NodeData::Element {
      tag:        "m:piecewise".to_string(),
      attributes: None,
      children:   pieces,
    }
  } else {
    // Generic array → matrix-like structure
    let mut rows = Vec::new();
    for row in element_children(node) {
      let cells: Vec<NodeData> = element_children(&row)
        .iter()
        .map(|cell| cmml_contents(doc, cell))
        .collect();
      rows.push(NodeData::Element {
        tag:        "m:matrixrow".to_string(),
        attributes: None,
        children:   cells,
      });
    }
    NodeData::Element {
      tag:        "m:matrix".to_string(),
      attributes: None,
      children:   rows,
    }
  }
}

/// Convert unparsed (multiple) nodes to Content MathML error.
///
/// Port of `cmml_unparsed`.
fn cmml_unparsed(doc: &PostDocument, nodes: &[Node]) -> NodeData {
  let mut results = vec![NodeData::Element {
    tag:        "m:csymbol".to_string(),
    attributes: Some(HashMap::from_iter([(
      "cd".to_string(),
      "ambiguous".to_string(),
    )])),
    children:   vec![NodeData::Text("fragments".to_string())],
  }];

  for node in nodes {
    let tag = doc.get_qname(node).unwrap_or_default();
    if tag == "ltx:XMTok" && node.get_attribute("role").as_deref() == Some("UNKNOWN") {
      results.push(NodeData::Element {
        tag:        "m:csymbol".to_string(),
        attributes: Some(HashMap::from_iter([(
          "cd".to_string(),
          "unknown".to_string(),
        )])),
        children:   vec![NodeData::Text(node.get_content())],
      });
    } else {
      results.push(cmml(doc, node));
    }
  }

  NodeData::Element {
    tag:        "m:cerror".to_string(),
    attributes: None,
    children:   results,
  }
}

/// Create a Content MathML error element.
fn cmml_error(symbol: &str) -> NodeData {
  NodeData::Element {
    tag:        "m:cerror".to_string(),
    attributes: None,
    children:   vec![NodeData::Element {
      tag:        "m:csymbol".to_string(),
      attributes: Some(HashMap::from_iter([(
        "cd".to_string(),
        "ambiguous".to_string(),
      )])),
      children:   vec![NodeData::Text(symbol.to_string())],
    }],
  }
}

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
pub fn convert_to_cmml(doc: &PostDocument, xmath: &Node) -> NodeData { cmml_contents(doc, xmath) }

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
              children:   vec![cmml(doc, &args[1])],
            },
            cmml(doc, &args[0]),
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
            attributes: Some(HashMap::from_iter([("closure".to_string(), "open".to_string())])),
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
            attributes: Some(HashMap::from_iter([("cd".to_string(), "ambiguous".to_string())])),
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
      let cn_type = if m.parse::<i64>().is_ok() {
        "integer"
      } else {
        "float"
      };
      return NodeData::Element {
        tag:        "m:cn".to_string(),
        attributes: Some(HashMap::from_iter([("type".to_string(), cn_type.to_string())])),
        children:   vec![NodeData::Text(m.clone())],
      };
    }

    // Default: csymbol with latexml cd
    return NodeData::Element {
      tag:        "m:csymbol".to_string(),
      attributes: Some(HashMap::from_iter([("cd".to_string(), "latexml".to_string())])),
      children:   vec![NodeData::Text(m.clone())],
    };
  }

  // No meaning: variable or number
  let content = node.get_content();
  match role.as_str() {
    "NUMBER" => {
      let cn_type = if content.parse::<i64>().is_ok() {
        "integer"
      } else {
        "float"
      };
      NodeData::Element {
        tag:        "m:cn".to_string(),
        attributes: Some(HashMap::from_iter([("type".to_string(), cn_type.to_string())])),
        children:   vec![NodeData::Text(content)],
      }
    },
    "SUPERSCRIPTOP" => NodeData::Element {
      tag:        "m:csymbol".to_string(),
      attributes: Some(HashMap::from_iter([("cd".to_string(), "ambiguous".to_string())])),
      children:   vec![NodeData::Text("superscript".to_string())],
    },
    "SUBSCRIPTOP" => NodeData::Element {
      tag:        "m:csymbol".to_string(),
      attributes: Some(HashMap::from_iter([("cd".to_string(), "ambiguous".to_string())])),
      children:   vec![NodeData::Text("subscript".to_string())],
    },
    _ => {
      // Variable / identifier
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
        children:   vec![NodeData::Text(name)],
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
      attributes: Some(HashMap::from_iter([("cd".to_string(), "latexml".to_string())])),
      children:   vec![NodeData::Text(meaning.to_string())],
    }
  }
}

/// Convert a "decorated symbol" — text or complex node treated as identifier.
///
/// Port of `cmml_decoratedSymbol`.
fn cmml_decorated_symbol(_doc: &PostDocument, node: &Node) -> NodeData {
  let content = node.get_content();
  NodeData::Element {
    tag:        "m:ci".to_string(),
    attributes: None,
    children:   vec![NodeData::Text(content)],
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
    attributes: Some(HashMap::from_iter([("cd".to_string(), "ambiguous".to_string())])),
    children:   vec![NodeData::Text("fragments".to_string())],
  }];

  for node in nodes {
    let tag = doc.get_qname(node).unwrap_or_default();
    if tag == "ltx:XMTok" && node.get_attribute("role").as_deref() == Some("UNKNOWN") {
      results.push(NodeData::Element {
        tag:        "m:csymbol".to_string(),
        attributes: Some(HashMap::from_iter([("cd".to_string(), "unknown".to_string())])),
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
      attributes: Some(HashMap::from_iter([("cd".to_string(), "ambiguous".to_string())])),
      children:   vec![NodeData::Text(symbol.to_string())],
    }],
  }
}

use crate::common::arena;
use crate::common::error::*;
use crate::document::{get_node_qname, Document};
use crate::state::Scope;
use crate::tokens::Tokens;
use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

pub type RewriteReplaceClosure = Rc<dyn Fn(&mut Document, Vec<&mut Node>) -> Result<()>>;

// ======================================================================
// Defining Rewrite rules that act on the DOM
// These are applied after the document is completely constructed
#[derive(Clone, Default)]
pub struct RewriteOptions {
  pub label:          Option<String>,
  pub scope:          Option<Scope>,
  pub xpath:          Option<String>,
  pub on_match:       Option<Tokens>,
  pub attributes:     Option<String>,
  pub attributes_map: Option<HashMap<String, String>>,
  pub replace:        Option<RewriteReplaceClosure>,
  pub regexp:         Option<String>,
  pub select:         Option<String>,
  pub select_count:   Option<usize>,
  pub is_math:        bool,
  pub wildcard_paths: Option<Vec<WildPath>>,
}
impl fmt::Debug for RewriteOptions {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "<RewriteOptions>") }
}
impl PartialEq for RewriteOptions {
  fn eq(&self, other: &RewriteOptions) -> bool { self.select == other.select }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewriteOperator {
  // only uncompiled:
  Label,
  Scope,
  Xpath,
  Match,
  // with available actions:
  Regexp,
  Attributes,
  Action,
  Replace,
  Test,
  MultiSelect,
  Select,
  Ignore,
  Trace,
}
#[derive(Clone)]
pub enum RewritePattern {
  String(String),
  Scope(Scope),
  Tokens(Tokens),
  Closure(RewriteReplaceClosure),
}
impl fmt::Debug for RewritePattern {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      RewritePattern::String(x) => write!(f, "{x:?}"),
      RewritePattern::Scope(x) => write!(f, "{x:?}"),
      RewritePattern::Tokens(x) => write!(f, "{x:?}"),
      RewritePattern::Closure(_) => write!(f, "<Rewrite Replacement Closure>"),
    }
  }
}
#[derive(Debug, Clone)]
pub struct RewriteClause {
  compiled: bool,
  op:       RewriteOperator,
  pattern:  RewritePattern,
}
impl RewriteClause {
  pub fn new_uncompiled(op: RewriteOperator, pattern: RewritePattern) -> Self {
    RewriteClause { compiled: false, op, pattern }
  }

  pub fn new_compiled(op: RewriteOperator, pattern: RewritePattern) -> Self {
    RewriteClause { compiled: true, op, pattern }
  }
}

#[derive(Debug, Clone, Default)]
pub struct Rewrite {
  pub options: RewriteOptions,
  pub clauses: Vec<RewriteClause>,
}
impl PartialEq for Rewrite {
  fn eq(&self, other: &Rewrite) -> bool { self.options == other.options }
}

impl Rewrite {
  pub fn new(_kind: &str, mut options: RewriteOptions) -> Self {
    use RewriteOperator::*;
    let mut clauses = Vec::new();
    // collect the non-compiling, early phase clauses from the options
    if let Some(xpath) = options.select.take() {
      clauses.push(RewriteClause {
        compiled: true,
        op:       Select,
        pattern:  RewritePattern::String(xpath),
      })
    }
    // collect the actionable clauses from the options
    if let Some(label) = options.label.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op:       Label,
        pattern:  RewritePattern::String(label),
      });
    }
    if let Some(scope) = options.scope.take() {
      // Convert Scope to string for compile_clause:
      // Perl uses strings like "label:sec:restricted" or "id:S1"
      let scope_str = match scope {
        crate::state::Scope::Named(s) => arena::with(s, |r| r.to_string()),
        crate::state::Scope::Global => String::from("global"),
        crate::state::Scope::Local => String::from("local"),
      };
      clauses.push(RewriteClause {
        compiled: false,
        op:       Scope,
        pattern:  RewritePattern::String(scope_str),
      });
    }
    if let Some(xpath) = options.xpath.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op:       Xpath,
        pattern:  RewritePattern::String(xpath),
      });
    }
    if let Some(tokens) = options.on_match.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op:       Match,
        pattern:  RewritePattern::Tokens(tokens),
      });
    }
    if let Some(replace) = options.replace.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op:       Replace,
        pattern:  RewritePattern::Closure(replace),
      });
    }
    if let Some(r) = options.regexp.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op:       Regexp,
        pattern:  RewritePattern::String(r),
      });
    }
    // If attributes string is set but attributes_map is not, parse string into map.
    // Perl format: "role='ID'" or "role='ID', meaning='foo'"
    if options.attributes_map.is_none() {
      if let Some(ref attrs_str) = options.attributes {
        let mut map = HashMap::default();
        for part in attrs_str.split(',') {
          let part = part.trim();
          if let Some((key, val)) = part.split_once('=') {
            let val = val.trim().trim_matches('\'').trim_matches('"');
            map.insert(key.trim().to_string(), val.to_string());
          }
        }
        if !map.is_empty() {
          options.attributes_map = Some(map);
        }
      }
    }
    if options.attributes_map.is_some() {
      clauses.push(RewriteClause {
        compiled: true,
        op:       Attributes,
        pattern:  RewritePattern::String(String::new()), // attributes stored in options
      });
    }
    Rewrite { options, clauses }
  }

  pub fn compile_clauses(&mut self, document: &mut Document) {
    let current_clauses: Vec<RewriteClause> = self.clauses.drain(..).collect();
    let mut new_clauses: Vec<RewriteClause> = Vec::new();
    for clause in current_clauses {
      if !clause.compiled {
        new_clauses.push(self.compile_clause(document, clause));
      } else {
        new_clauses.push(clause);
      }
    }
    self.clauses = new_clauses;
  }

  pub fn compile_clause(
    &mut self,
    document: &mut Document,
    clause: RewriteClause,
  ) -> RewriteClause {
    let op = clause.op;
    let pattern = clause.pattern;

    if op == RewriteOperator::Xpath {
      if self.options.select_count.is_none() {
        self.options.select_count = Some(1);
      }
      return RewriteClause {
        compiled: true,
        op: RewriteOperator::Select,
        pattern,
      };
    }
    // scope => 'label:...' compiles to select with xpath via label ID resolution
    // Perl: $op = 'select'; $pattern = ["descendant-or-self::*[@xml:id='<id>']", 1];
    if op == RewriteOperator::Scope {
      if let RewritePattern::String(scope_str) = &pattern {
        if let Some(label_part) = scope_str.strip_prefix("label:") {
          if let Some(id) = document.rewrite_labels.get(label_part).cloned() {
            if self.options.select_count.is_none() {
              self.options.select_count = Some(1);
            }
            let xpath = format!("descendant-or-self::*[@xml:id='{}']", id);
            return RewriteClause {
              compiled: true,
              op: RewriteOperator::Select,
              pattern: RewritePattern::String(xpath),
            };
          }
          // Try with LABEL: prefix (clean_label adds it)
          let clean_key = format!("LABEL:{}", label_part);
          if let Some(id) = document.rewrite_labels.get(&clean_key).cloned() {
            if self.options.select_count.is_none() {
              self.options.select_count = Some(1);
            }
            let xpath = format!("descendant-or-self::*[@xml:id='{}']", id);
            return RewriteClause {
              compiled: true,
              op: RewriteOperator::Select,
              pattern: RewritePattern::String(xpath),
            };
          }
          // Label not found — ignore this clause
          return RewriteClause {
            compiled: true,
            op: RewriteOperator::Ignore,
            pattern: RewritePattern::String(String::new()),
          };
        } else if let Some(id_part) = scope_str.strip_prefix("id:") {
          if self.options.select_count.is_none() {
            self.options.select_count = Some(1);
          }
          let xpath = format!(
            "descendant-or-self::*[@xml:id='{0}' or @id='{0}']", id_part);
          return RewriteClause {
            compiled: true,
            op: RewriteOperator::Select,
            pattern: RewritePattern::String(xpath),
          };
        }
        return RewriteClause {
          compiled: true,
          op: RewriteOperator::Ignore,
          pattern: RewritePattern::String(String::new()),
        };
      }
    }
    // Match => pre-compiled XPath string (for .latexml loader)
    // When match is already a RewritePattern::String, it's a pre-compiled xpath
    if op == RewriteOperator::Match {
      if let RewritePattern::String(xpath) = pattern {
        if self.options.select_count.is_none() {
          self.options.select_count = Some(1);
        }
        return RewriteClause {
          compiled: true,
          op: RewriteOperator::Select,
          pattern: RewritePattern::String(xpath),
        };
      }
    }
    RewriteClause { compiled: true, op, pattern }
  }

  pub fn invoke(&mut self, document: &mut Document, root: &Node) -> Result<()> {
    // Debug(('=' x 40)) if $LaTeXML::DEBUG{rewrite};
    // What goes into self.clauses ???
    let clauses = self.clauses.iter().collect();
    self.apply_clause(document, root, 0, clauses)?;
    Ok(())
  }
  // Rewrite spec as input
  //   scope  => $scope  : a scope like "section:1.2.3" or "label:eq.one"; translated to xpath
  //   select => $xpath  : selects subtrees based on xpath expression.
  //   match  => $code   : called on $document and current $node: tests current node, returns
  // $nnodes, if match   match  => $string : Treats as TeX, converts Box, then DOM tree, to xpath
  //                      (The matching top-level nodes will be replaced, if replace is the next
  // op.)   replace=> $code   : removes the current $nnodes, calls $code with $document and
  // removed nodes   replace=> $string : removes $nnodes
  //                       Treats $string as TeX, converts to Box and inserts to replace
  //                       the removed nodes.
  //   attributes=>$hash : adds data from hash as attributes to the current node.
  //   regexp  => $string: apply regexp (subst) to all text nodes in/under the current node.

  // Compiled rewrite spec:
  //   select => $xpath  : operate on nodes selected by $xpath.
  //   test   => $code   : Calls $code on $document and current $node.
  //                       Returns number of nodes matched.
  //   replace=> $code   : removes the current $nnodes, calls $code on them.
  //   action => $code   : invoke $code on current $node, without removing them.
  //   regexp  => $string: apply regexp (subst) to all text nodes in/under the current node.

  fn apply_clause(
    &self,
    document: &mut Document,
    tree: &Node,
    nmatched: usize,
    mut clauses: VecDeque<&RewriteClause>,
  ) -> Result<()> {
    use RewriteOperator::*;
    if let Some(RewriteClause { compiled: _, op, pattern }) = clauses.pop_front() {
      match op {
        Select => {
          if let RewritePattern::String(xpath) = pattern {
            let matches = document.findnodes(xpath, Some(tree));
            // Only apply wildcard filtering on content Selects, not scope Selects
            let is_content_select = !xpath.contains("xml:id") && !xpath.contains("@id=");
            let wilds = if is_content_select {
              self.options.wildcard_paths.clone()
            } else { None };
            // Get declare pattern metadata for Rust-side filtering
            // Only apply on content Selects, not scope Selects
            let declare_type = if is_content_select {
              self.options.attributes_map.as_ref()
                .and_then(|a| a.get("_declare_type")).cloned()
            } else { None };
            let declare_base = if is_content_select {
              self.options.attributes_map.as_ref()
                .and_then(|a| a.get("_declare_base")).cloned()
            } else { None };
            let declare_sub = if is_content_select {
              self.options.attributes_map.as_ref()
                .and_then(|a| a.get("_declare_sub")).cloned()
            } else { None };
            let declare_accent = if is_content_select {
              self.options.attributes_map.as_ref()
                .and_then(|a| a.get("_declare_accent")).cloned()
            } else { None };
            for node in matches {
              if node.has_attribute("_matched") {
                continue;
              }
              // Rust-side filtering for declare pattern types (content Selects only)
              if let Some(ref dtype) = declare_type {
                let passes = declare_node_matches(
                  document, &node, dtype, declare_base.as_deref(),
                  declare_sub.as_deref(), declare_accent.as_deref(),
                );
                if !passes {
                  continue;
                }
              }
              let marked = if let Some(ref wpaths) = wilds {
                mark_wildcards(&node, wpaths)
              } else {
                vec![]
              };
              // Scope Selects always pass nmatched=1; content Selects use select_count
              let nmatched_for_clause = if is_content_select {
                self.options.select_count.unwrap_or(1)
              } else { 1 };
              self.apply_clause(
                document,
                &node,
                nmatched_for_clause,
                clauses.clone(),
              )?;
              if !marked.is_empty() {
                unmark_wildcards(&marked);
              }
            }
          }
        },
        Replace => {
          let mut parent = tree.get_parent().unwrap();
          // Remove & separate nodes to be replaced, and sibling nodes following them.
          let mut following = VecDeque::new(); // Collect the matching and following nodes
          while let Some(mut sib) = parent.get_last_child() {
            sib.unbind_node();
            if *tree == sib {
              following.push_front(sib);
              break;
            } else {
              following.push_front(sib);
            }
          }
          let mut replaced = Vec::new();
          for _idx in 0..nmatched {
            // Remove the nodes to be replaced
            if let Some(popped) = following.pop_front() {
              replaced.push(popped);
            } else {
              break; // nmatched larger than available nodes — stop
            }
          }
          for rnode in replaced.iter() {
            document.unrecord_node_ids(rnode);
          }
          // Carry out the operation, inserting whatever nodes.
          document.set_node(&parent);
          let point_opt = parent.get_last_child();
          if let RewritePattern::Closure(closure) = pattern {
            closure(document, replaced.iter_mut().collect())?; // Carry out the insertion.
          }

          // Now collect the newly inserted nodes for any needed patching
          let inserted = if let Some(point) = point_opt {
            let mut ins_queue = VecDeque::new();
            let mut sibs = parent.get_child_nodes();
            while let Some(sib) = sibs.pop() {
              if sib == point {
                break;
              }
              ins_queue.push_front(sib);
            }
            ins_queue.into_iter().collect::<Vec<Node>>()
          } else {
            parent.get_child_nodes()
          };

          // Now make any adjustments to the new nodes
          for ins in inserted.iter() {
            document.record_node_ids(ins)?;
          }
          // TODO: Can we avoid this clone?
          let font = document.get_node_font(tree).clone();
          // the font of the matched node
          for ins in inserted.iter() {
            // Copy the non-semantic parts of font to the replacement
            document.merge_node_font_rec(ins, &font)?;
          }
          // Now, replace the following nodes.
          for mut follow_node in following {
            parent.add_child(&mut follow_node)?;
          }
        },
        Attributes => {
          if let Some(ref attrs) = self.options.attributes_map {
            let has_wc = tree.has_attribute("_has_wildcards");
            if has_wc {
              // Perl: setAttributes_wild — wildcards present in matched tree
              let mut nodes = vec![tree.clone()];
              // Collect nmatched siblings
              let mut cur = tree.clone();
              for _ in 1..nmatched {
                if let Some(sib) = cur.get_next_sibling() {
                  cur = sib.clone();
                  nodes.push(sib);
                } else { break; }
              }
              set_attributes_wild(document, attrs, nodes, nmatched)?;
            } else if nmatched > 1 {
              // Multi-node: collect nmatched element siblings starting from tree
              let mut nodes = vec![tree.clone()];
              let mut cur = tree.clone();
              for _ in 1..nmatched {
                while let Some(sib) = cur.get_next_sibling() {
                  cur = sib.clone();
                  if sib.get_type() == Some(libxml::tree::NodeType::ElementNode) {
                    nodes.push(sib);
                    break;
                  }
                }
              }
              // Perl: skip if ALL nodes already matched
              if nodes.iter().any(|n| !n.has_attribute("_matched")) {
                if let Ok(Some(mut wrapper)) = document.wrap_nodes("ltx:XMWrap", nodes) {
                  for (key, value) in attrs {
                    if !key.starts_with('_') {
                      let _ = wrapper.set_attribute(key, value);
                    }
                  }
                  let _ = wrapper.set_attribute("_rewrite", "1");
                }
              }
            } else if !tree.has_attribute("_matched") {
              // Single node: set attributes directly
              let mut node = tree.clone();
              for (key, value) in attrs {
                if !key.starts_with('_') {
                  let _ = node.set_attribute(key, value);
                }
              }
              if node.get_name() == "XMApp" && attrs.contains_key("role") {
                let _ = node.set_attribute("_rewrite", "1");
              }
            }
          }
          mark_seen(tree, nmatched);
          self.apply_clause(document, tree, nmatched, clauses)?;
        },
        Ignore => {
          // Perl: $self->applyClause($document, $tree, $nmatched, @more_clauses);
          self.apply_clause(document, tree, nmatched, clauses)?;
        },
        Regexp => {
          // Perl: tests the matched node's text content against a regex.
          // If it matches, continue with remaining clauses; otherwise skip.
          if let RewritePattern::String(regex_str) = pattern {
            let content = tree.get_content();
            let re = regex::Regex::new(regex_str)
              .unwrap_or_else(|_| regex::Regex::new("$^").unwrap()); // never-match fallback
            if re.is_match(&content) {
              self.apply_clause(document, tree, nmatched, clauses)?;
            }
            // else: no match → skip this node (don't mark as seen)
          }
        },
        Label => {
          // Label clause stores the label on the node. Perl: $$self{label} usage.
          // Typically compiled away in compile_clause, but if it reaches here, record it.
          if let RewritePattern::String(label_str) = pattern {
            let id = tree.get_attribute("xml:id").unwrap_or_default();
            if !id.is_empty() {
              document.rewrite_labels.insert(label_str.clone(), id);
            }
          }
          self.apply_clause(document, tree, nmatched, clauses)?;
        },
        Trace => {
          // Debug tracing — just continue
          self.apply_clause(document, tree, nmatched, clauses)?;
        },
        Action => {
          // Perl: $code->($document, $tree, $nmatched)
          // Action invokes a closure on the matched node without removing it.
          if let RewritePattern::Closure(closure) = pattern {
            let mut node = tree.clone();
            closure(document, vec![&mut node])?;
          }
          // Continue with remaining clauses
          self.apply_clause(document, tree, nmatched, clauses)?;
        },
        Test => {
          // Perl: $nnodes = $code->($document, $tree)
          // Test invokes a closure that returns the number of matched nodes.
          // If 0, skip remaining clauses. Otherwise continue with nmatched.
          if let RewritePattern::Closure(closure) = pattern {
            let mut node = tree.clone();
            // The closure modifies the node count; for now just invoke and continue
            closure(document, vec![&mut node])?;
            self.apply_clause(document, tree, nmatched, clauses)?;
          }
        },
        MultiSelect => {
          // Perl: like Select but matches multiple adjacent nodes
          if let RewritePattern::String(xpath) = pattern {
            let count = self.options.select_count.unwrap_or(1);
            let matches = document.findnodes(xpath, Some(tree));
            for node in matches {
              if node.has_attribute("_matched") {
                continue;
              }
              self.apply_clause(document, &node, count, clauses.clone())?;
            }
          }
        },
        _ => {
          // Remaining unimplemented operators — skip silently
          self.apply_clause(document, tree, nmatched, clauses)?;
        },
      }
    } else {
      // No more clauses — mark the matched nodes as seen
      // Perl: markSeen($tree, $nmatched) when no more clauses
      mark_seen(tree, nmatched);
    }

    Ok(())
  }
}

// ======================================================================
// WildCard support: domToXPath, wildcard marking, XMDual wrapping
// ======================================================================

/// Wildcard position path: indices to navigate from matched root to wildcard node.
/// First index uses nth_sibling (sibling offset), rest use nth_child (child position).
pub type WildPath = Vec<usize>;

/// Result of domToXPath compilation: xpath string, node count, wildcard paths.
pub type CompiledMatch = (String, usize, Vec<WildPath>);

/// Convert a DOM subtree to an XPath expression + wildcard position tracking.
/// Perl: domToXPath() → domToXPath_rec() → domToXPath_seq()
pub fn dom_to_xpath(document: &Document, node: &Node) -> CompiledMatch {
  let (xpath, nnodes, _nwilds, wilds) =
    dom_to_xpath_rec(document, node, "descendant-or-self", None);
  (xpath, nnodes, wilds)
}

/// Attributes excluded from XPath match predicates.
fn is_excluded_match_attr(key: &str) -> bool {
  matches!(key, "scriptpos" | "mathstyle" | "xml:id" | "fontsize" | "_font" | "_pvis" | "_cvis")
    || key.starts_with('_')
}

/// Recursive DOM-to-XPath conversion.
/// Returns (xpath_fragment, node_count, wildcard_count, wildcard_paths)
fn dom_to_xpath_rec(
  document: &Document, node: &Node, axis: &str, pos: Option<usize>,
) -> (String, usize, usize, Vec<WildPath>) {
  let node_type = node.get_type();
  // NodeList / DocumentFragment: sequence of children
  if node_type == Some(libxml::tree::NodeType::DocumentFragNode) {
    let children = node.get_child_nodes();
    let (xpath, nnodes, wilds) = dom_to_xpath_seq(document, axis, pos, &children);
    return (xpath, nnodes, 0, wilds);
  }
  if node_type == Some(libxml::tree::NodeType::ElementNode) {
    let qname = arena::with(get_node_qname(node), |s| s.to_string());
    let children = node.get_child_nodes();

    // _WildCard_ element → matches anything
    if qname == "_WildCard_" {
      if !children.is_empty() {
        // WildCard WITH children: recurse on children
        let child_list = node.get_child_nodes();
        // Create a fragment-like approach: process children as a sequence
        let (xpath, _nnodes, _nwilds, _wilds) =
          dom_to_xpath_rec(document, &child_list[0], axis, pos);
        let n = children.len().max(1);
        return (xpath, n, n, vec![]);
      } else {
        return (format!("{axis}::*"), 1, 1, vec![]);
      }
    }
    // XMRef pointing to a _WildCard_ is also a wildcard
    if qname == "ltx:XMRef" {
      if let Some(idref) = node.get_property("idref") {
        if let Some(target) = document.lookup_id(&idref).cloned() {
          let tqname = arena::with(get_node_qname(&target), |s| s.to_string());
          // Check if target is XMArg/XMWrap with single WildCard child
          let is_wild = if tqname.ends_with("XMArg") || tqname.ends_with("XMWrap") {
            let tc = target.get_child_nodes();
            tc.len() == 1 && arena::with(get_node_qname(&tc[0]), |s| s == "_WildCard_")
          } else {
            tqname == "_WildCard_"
          };
          if is_wild {
            return (format!("{axis}::*"), 1, 1, vec![]);
          }
        }
      }
    }
    // XMArg/XMWrap with single _WildCard_ child
    if (qname.ends_with("XMArg") || qname.ends_with("XMWrap"))
      && children.len() == 1
      && arena::with(get_node_qname(&children[0]), |s| s.to_string()) == "_WildCard_"
    {
      let wc_children = children[0].get_child_nodes();
      if !wc_children.is_empty() {
        let (child_xpath, _nn, _nw, _w) =
          dom_to_xpath_rec(document, &wc_children[0], "child", Some(1));
        let mut preds = vec![];
        if let Some(p) = pos { preds.push(format!("position()={p}")); }
        preds.push(child_xpath);
        return (format!("{axis}::{qname}[{}]", preds.join(" and ")), 1, 1, vec![]);
      } else {
        return (format!("{axis}::*"), 1, 1, vec![]);
      }
    }

    // Standard element: build predicates from attributes and children
    let mut predicates = Vec::new();
    let mut wilds = Vec::new();

    // Attribute predicates
    let attrs = node.get_attributes();
    for (key, value) in &attrs {
      if !is_excluded_match_attr(key) {
        predicates.push(format!("@{key}='{}'", value.replace('\'', "&apos;")));
      }
    }
    // Child predicates
    if !children.is_empty() {
      let all_text = children.iter().all(|c|
        c.get_type() == Some(libxml::tree::NodeType::TextNode));
      let all_elem = children.iter().all(|c|
        c.get_type() == Some(libxml::tree::NodeType::ElementNode));
      if all_text {
        let text = node.get_content();
        predicates.push(format!("text()='{}'", text.replace('\'', "&apos;")));
      } else if all_elem {
        let (xp, _nn, w) = dom_to_xpath_seq(document, "child", Some(1), &children);
        predicates.push(xp);
        wilds.extend(w);
      }
      // Mixed content: skip (rare in math patterns)
    }

    // Position-based matching (when this is a child in a sequence)
    let tag = if let Some(p) = pos {
      predicates.insert(0, format!("self::{qname}"));
      predicates.insert(0, format!("position()={p}"));
      "*".to_string()
    } else {
      qname
    };
    let preds = predicates.join(" and ");
    let xpath = if preds.is_empty() {
      format!("{axis}::{tag}")
    } else {
      format!("{axis}::{tag}[{preds}]")
    };
    return (xpath, 1, 0, wilds);
  }
  if node_type == Some(libxml::tree::NodeType::TextNode) {
    let text = node.get_content();
    return (format!("*[text()='{}']", text.replace('\'', "&apos;")), 1, 0, vec![]);
  }
  (String::new(), 0, 0, vec![])
}

/// Convert a sequence of sibling nodes to XPath with wildcard tracking.
/// Perl: domToXPath_seq()
fn dom_to_xpath_seq(
  document: &Document, axis: &str, pos: Option<usize>, nodes: &[Node],
) -> (String, usize, Vec<WildPath>) {
  if nodes.is_empty() {
    return (String::new(), 0, vec![]);
  }
  let mut i: usize = 1;
  let mut sib_xpaths = Vec::new();
  let mut wilds = Vec::new();

  // First node
  let (xpath, _nn, nwilds, w0) = dom_to_xpath_rec(document, &nodes[0], axis, pos);
  if nwilds > 0 {
    for _ in 0..nwilds {
      wilds.push(vec![i]);
      i += 1;
    }
  } else {
    for w in &w0 {
      let mut path = vec![1usize];
      path.extend(w);
      wilds.push(path);
    }
    i += 1;
  }
  // Remaining siblings
  for sib in &nodes[1..] {
    let (xp, _nn, nw, w) =
      dom_to_xpath_rec(document, sib, "following-sibling", Some(i - 1));
    sib_xpaths.push(xp);
    if nw > 0 {
      for _ in 0..nw {
        wilds.push(vec![i]);
        i += 1;
      }
    } else {
      for ww in &w {
        let mut path = vec![i];
        path.extend(ww);
        wilds.push(path);
      }
      i += 1;
    }
  }
  let mut result = xpath;
  for sp in &sib_xpaths {
    result = format!("{result}[{sp}]");
  }
  (result, i - 1, wilds)
}

/// Navigate to the nth sibling (1-based) from a starting node.
fn nth_sibling(node: &Node, n: usize) -> Option<Node> {
  let mut current = Some(node.clone());
  for _ in 1..n {
    current = current.and_then(|n| {
      let mut next = n.get_next_sibling();
      // Skip non-element nodes
      while let Some(ref s) = next {
        if s.get_type() == Some(libxml::tree::NodeType::ElementNode) { break; }
        next = s.get_next_sibling();
      }
      next
    });
  }
  current
}

/// Navigate to the nth child (1-based) of a node.
fn nth_child(node: &Node, n: usize) -> Option<Node> {
  node.get_child_nodes().into_iter().nth(n - 1)
}

/// Mark wildcard nodes in the matched tree.
/// Perl: markWildcards($node, @wilds)
pub fn mark_wildcards(node: &Node, wilds: &[WildPath]) -> Vec<Node> {
  if wilds.is_empty() { return vec![]; }
  let mut n = node.clone();
  let _ = n.set_attribute("_has_wildcards", "1");
  let mut marked = Vec::new();
  for wild in wilds {
    let mut current = Some(node.clone());
    let mut first = true;
    for &idx in wild {
      if current.is_none() { break; }
      current = if first {
        first = false;
        nth_sibling(current.as_ref().unwrap(), idx)
      } else {
        nth_child(current.as_ref().unwrap(), idx)
      };
    }
    if let Some(ref c) = current {
      if c.get_type() == Some(libxml::tree::NodeType::ElementNode) {
        let mut mc = c.clone();
        let _ = mc.set_attribute("_wildcard", "1");
        marked.push(mc);
      }
    }
  }
  marked
}

/// Unmark wildcard nodes after processing.
pub fn unmark_wildcards(nodes: &[Node]) {
  for n in nodes {
    if n.get_type() == Some(libxml::tree::NodeType::ElementNode) {
      let mut mc = n.clone();
      let _ = mc.remove_attribute("_has_wildcards");
      let _ = mc.remove_attribute("_wildcard");
    }
  }
}

/// Collect xml:ids of wildcard-marked nodes, generating IDs if needed.
/// Collect xml:ids of wildcard-marked nodes, generating IDs if needed.
/// Perl: set_wildcard_ids($document, $node) — Rewrite.pm L219-231
/// Faithfully translated: if node has _wildcard, return its ID.
/// If node has _matched, skip (already processed by prior rule).
/// Otherwise recurse into children.
pub fn set_wildcard_ids(document: &mut Document, node: &Node) -> Vec<String> {
  if node.get_type() != Some(libxml::tree::NodeType::ElementNode) {
    return vec![];
  }
  if node.has_attribute("_matched") {
    return vec![];
  }
  if node.has_attribute("_wildcard") {
    // Perl: unconditionally returns the wildcard's ID.
    // Even if all descendants are already matched, the ID is still needed
    // for XMRef in the content arm. pruneXMDuals handles collapsing later.
    let id = if let Some(existing) = node.get_property("xml:id").or_else(|| node.get_property("id")) {
      existing
    } else {
      // Generate an ID for this node
      let mut n = node.clone();
      let _ = document.generate_id(&mut n, "");
      node.get_property("xml:id").or_else(|| node.get_property("id")).unwrap_or_default()
    };
    return vec![id];
  }
  // Recurse into children
  let mut ids = Vec::new();
  for child in node.get_child_nodes() {
    ids.extend(set_wildcard_ids(document, &child));
  }
  ids
}

/// Set attributes on a tree containing wildcards, creating XMDual wrappers.
/// Perl: setAttributes_wild($document, $attributes, @nodes) — Rewrite.pm L195-217
///
/// Faithfully translated from Perl. The structure created is:
/// ```xml
/// <XMDual role="...">
///   <XMApp>                    <!-- content arm -->
///     <XMTok decl_id="..." />  <!-- semantic operator -->
///     <XMRef idref="..." />    <!-- references to wildcards -->
///   </XMApp>
///   <XMWrap>                   <!-- presentation arm -->
///     [original nodes]         <!-- with _wildcard markers -->
///   </XMWrap>
/// </XMDual>
/// ```
pub fn set_attributes_wild(
  document: &mut Document, attrs: &HashMap<String, String>,
  nodes: Vec<Node>, _nmatched: usize,
) -> Result<()> {
  // Perl L197: return unless grep { !$_->getAttribute('_matched'); } @nodes;
  if nodes.iter().all(|n| n.has_attribute("_matched")) {
    return Ok(());
  }
  let nowrap = attrs.contains_key("_nowrap");
  // Perl L199-203: _nowrap or single XMDual → set attrs on first non-wildcard node
  if nowrap || (nodes.len() == 1 && nodes[0].get_name() == "XMDual") {
    if let Some(nonwild) = nodes.iter().find(|n| !n.has_attribute("_wildcard")) {
      let mut n = nonwild.clone();
      for (key, value) in attrs {
        if !key.starts_with('_') {
          let _ = n.set_attribute(key, value);
        }
      }
    }
    return Ok(());
  }

  // Collect wildcard IDs from the nodes BEFORE wrapping in XMDual.
  let mut wild_ids = Vec::new();
  for n in &nodes {
    wild_ids.extend(set_wildcard_ids(document, n));
  }

  // Wrap matched nodes in XMDual.
  let wrapper = document.wrap_nodes("ltx:XMDual", nodes)?;
  let Some(mut dual_node) = wrapper else { return Ok(()); };

  // Set role on XMDual (Perl L209)
  if let Some(role) = attrs.get("role") {
    let _ = dual_node.set_attribute("role", role);
  }

  // Build content arm: XMApp > XMTok[attrs] + XMRef[wildcard_ids]
  let doc = document.get_document();
  let mut content_app = Node::new("XMApp", None, doc)?;
  let mut content_op = Node::new("XMTok", None, doc)?;
  for (key, value) in attrs {
    if key != "role" && !key.starts_with('_') {
      let _ = content_op.set_attribute(key, value);
    }
  }
  content_app.add_child(&mut content_op)?;
  for rid in &wild_ids {
    let mut xmref = Node::new("XMRef", None, doc)?;
    let _ = xmref.set_attribute("idref", rid);
    content_app.add_child(&mut xmref)?;
  }

  // Insert content arm as first child (before presentation nodes)
  if let Some(mut first_child) = dual_node.get_first_child() {
    first_child.add_prev_sibling(&mut content_app)?;
  } else {
    dual_node.add_child(&mut content_app)?;
  }

  mark_seen_rec(&dual_node);
  Ok(())
}

/// Mark a node (and nsibs following siblings) as matched, preventing re-matching.
/// Perl: markSeen($node, $nsibs) + markSeen_rec($node)
fn mark_seen(node: &Node, nsibs: usize) {
  let mut current = Some(node.clone());
  for _i in 0..nsibs {
    if let Some(n) = current {
      mark_seen_rec(&n);
      current = n.get_next_sibling();
    } else {
      break;
    }
  }
}


fn mark_seen_rec(node: &Node) {
  if node.has_attribute("_wildcard") {
    return;
  }
  let mut n = node.clone();
  let _ = n.set_attribute("_matched", "1");
  for child in node.get_child_nodes() {
    if child.get_type() == Some(libxml::tree::NodeType::ElementNode) {
      mark_seen_rec(&child);
    }
  }
}

/// Rust-side filtering for \lxDeclare pattern matching.
/// XPath matches are broad (to avoid nested predicate bugs); this function
/// verifies the matched node's children match the specific pattern.
///
/// Pattern types:
/// - "subscript": node is XMApp[@role='POSTSUBSCRIPT'], check base text + optional sub text
/// - "prime": node is XMApp[@role='POSTSUPERSCRIPT'], check base text
/// - "accent": node is XMApp, check accent name in first child, optional base text
/// - "simple": no extra filtering needed (XPath is specific enough)
fn declare_node_matches(
  document: &Document, node: &Node, pattern_type: &str, base_text: Option<&str>,
  sub_text: Option<&str>, accent_name: Option<&str>,
) -> bool {
  let children = node.get_child_nodes();
  match pattern_type {
    "literal_subscript" => {
      // Matched node is the BASE XMTok. Check that next sibling is POSTSUBSCRIPT
      // with specific subscript content.
      let next_sib = node.get_next_sibling();
      let next_role = next_sib.as_ref().and_then(|s| s.get_property("role"));
      if next_role.as_deref() != Some("POSTSUBSCRIPT") {
        return false;
      }
      // Check subscript content text
      if let Some(sub) = sub_text {
        let sub_content = next_sib.as_ref().map(|s| s.get_content()).unwrap_or_default();
        if sub_content.trim() != sub {
          return false;
        }
      }
      true
    },
    "subscript" => {
      // Wildcard subscript: matched node is BASE XMTok.
      // Check that next sibling is POSTSUBSCRIPT.
      let next_sib = node.get_next_sibling();
      let next_role = next_sib.as_ref().and_then(|s| s.get_property("role"));
      next_role.as_deref() == Some("POSTSUBSCRIPT")
    },
    "prime" => {
      // Matched node is BASE XMTok. Check that next sibling is POSTSUPERSCRIPT
      // with prime content.
      let next_sib = node.get_next_sibling();
      let next_role = next_sib.as_ref().and_then(|s| s.get_property("role"));
      if next_role.as_deref() != Some("POSTSUPERSCRIPT") {
        return false;
      }
      // Check prime content
      let sup_content = next_sib.as_ref().map(|s| s.get_content()).unwrap_or_default();
      sup_content.contains('′')
    },
    "accent" => {
      // XMApp with children: [accent_op, base_content]
      if children.len() < 2 { return false; }
      // Check accent name on first child
      if let Some(accent) = accent_name {
        let first_name = children[0].get_property("name")
          .or_else(|| children[0].get_property("meaning"));
        if first_name.as_deref() != Some(accent) {
          return false;
        }
        // Accent ops should have OVERACCENT or UNDERACCENT role
        let role = children[0].get_property("role");
        let is_accent = role.as_deref().map(|r|
          r.contains("ACCENT")).unwrap_or(false);
        if !is_accent { return false; }
      }
      // Check base content text if specified
      if let Some(base) = base_text {
        if !declare_base_matches(&children[1], base) {
          return false;
        }
      }
      true
    },
    "simple" => {
      // Font check: plain declarations (e.g. $x$) should NOT match tokens with
      // non-default fonts (bold, caligraphic, typewriter).
      // Perl: font_match_xpaths generates XPath predicates from _font attribute.
      let font = document.get_node_font(node);
      if let Some(series) = font.get_series() {
        if series.as_ref() == "bold" {
          return false;
        }
      }
      if let Some(family) = font.get_family() {
        let fam = family.as_ref();
        if fam == "caligraphic" || fam == "typewriter" {
          return false;
        }
      }
      true
    }
    _ => true,        // Unknown type: pass through
  }
}

/// Check if a node matches a base text specification.
/// Handles both plain text (e.g. "x") and command names (e.g. "\varepsilon").
fn declare_base_matches(node: &Node, base_spec: &str) -> bool {
  if base_spec.starts_with('\\') {
    // Command base: match by meaning or name attribute
    let cmd = base_spec.trim_start_matches('\\');
    // Handle \mathcal{X} → check font=caligraphic + text=X
    if let Some(inner) = cmd.strip_prefix("mathcal{").and_then(|s| s.strip_suffix('}')) {
      let font = node.get_property("font").unwrap_or_default();
      let text = node.get_content();
      return font == "caligraphic" && text.trim() == inner;
    }
    // General command: check meaning attribute
    let meaning = node.get_property("meaning").unwrap_or_default();
    meaning == cmd
  } else {
    // Plain text base: match node text content
    let text = node.get_content();
    text.trim() == base_spec
  }
}

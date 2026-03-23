use crate::common::arena;
use crate::common::error::*;
use crate::document::Document;
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
          let xpath = format!("descendant-or-self::*[@xml:id='{}']", id_part);
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
          // my ($xpath, $nnodes, @wilds) = @$pattern;
          if let RewritePattern::String(xpath) = pattern {
            let matches = document.findnodes(xpath, Some(tree));
            for node in matches {
              // next unless node.get_owner_document()->isSameNode($tree->ownerDocument); # If still
              // attached to original document!
              if node.has_attribute("_matched") {
                continue;
              }
              // let w = mark_wildcards(node, wilds);
              self.apply_clause(
                document,
                &node,
                self.options.select_count.unwrap_or(1),
                clauses.clone(),
              )?;
              // unmark_wildcards(node, w);
            }
          } else {
            // unsupported rewrite pattern? should never happen.
            todo!()
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
              todo!(); // should we report an error if nmatched was too large?
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
          // Perl: setAttributes_encapsulate — set attributes on the matched node(s)
          if let Some(ref attrs) = self.options.attributes_map {
            if nmatched > 1 {
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
                // Wrap in XMWrap and set attributes on wrapper.
                // Mark as rewrite-created so the parser treats it as an atomic
                // token (not recursively parsed like parser-created XMWraps).
                if let Ok(Some(mut wrapper)) = document.wrap_nodes("ltx:XMWrap", nodes) {
                  for (key, value) in attrs {
                    let _ = wrapper.set_attribute(key, value);
                  }
                  let _ = wrapper.set_attribute("_rewrite", "1");
                }
              }
            } else if !tree.has_attribute("_matched") {
              // Single node: set attributes directly
              let mut node = tree.clone();
              for (key, value) in attrs {
                let _ = node.set_attribute(key, value);
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
        _ => todo!(),
      }
    } else {
      // No more clauses — mark the matched nodes as seen
      // Perl: markSeen($tree, $nmatched) when no more clauses
      mark_seen(tree, nmatched);
    }

    Ok(())
  }
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

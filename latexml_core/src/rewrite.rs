use crate::common::error::*;
use crate::document::Document;
use crate::state::Scope;
use crate::tokens::Tokens;
use libxml::tree::Node;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

pub type RewriteReplaceClosure = Rc<dyn Fn(&mut Document, Vec<&mut Node>) -> Result<()>>;

// ======================================================================
// Defining Rewrite rules that act on the DOM
// These are applied after the document is completely constructed
#[derive(Clone, Default)]
pub struct RewriteOptions {
  pub label: Option<String>,
  pub scope: Option<Scope>,
  pub xpath: Option<String>,
  pub on_match: Option<Tokens>,
  pub attributes: Option<String>,
  pub replace: Option<RewriteReplaceClosure>,
  pub regexp: Option<String>,
  pub select: Option<String>,
  pub select_count: Option<usize>,
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
  op: RewriteOperator,
  pattern: RewritePattern,
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
        op: Select,
        pattern: RewritePattern::String(xpath),
      })
    }
    // collect the actionable clauses from the options
    if let Some(label) = options.label.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op: Label,
        pattern: RewritePattern::String(label),
      });
    }
    if let Some(scope) = options.scope.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op: Scope,
        pattern: RewritePattern::Scope(scope),
      });
    }
    if let Some(xpath) = options.xpath.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op: Xpath,
        pattern: RewritePattern::String(xpath),
      });
    }
    if let Some(tokens) = options.on_match.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op: Match,
        pattern: RewritePattern::Tokens(tokens),
      });
    }
    if let Some(replace) = options.replace.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op: Replace,
        pattern: RewritePattern::Closure(replace),
      });
    }
    if let Some(r) = options.regexp.take() {
      clauses.push(RewriteClause {
        compiled: false,
        op: Regexp,
        pattern: RewritePattern::String(r),
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
    _document: &mut Document,
    clause: RewriteClause,
  ) -> RewriteClause {
    let op = clause.op;
    let pattern = clause.pattern;
    //   my ($oop, $opattern) = ($op, $pattern);
    //   if ($op eq 'label') {
    //     if (ref $pattern eq 'ARRAY') {
    //       #      $op='multi_select'; $pattern = [map(["descendant-or-self::*[\@label='$_']",1],
    // @$pattern)]; }

    //       $op = 'multi_select'; $pattern = [map { ["descendant-or-self::*[\@xml:id='$_']", 1] }
    //           map { $self->getLabelID($_) } @$pattern]; }
    //     else {
    //       #      $op='select'; $pattern=["descendant-or-self::*[\@label='$pattern']",1]; }}
    //       $op      = 'select';
    //       $pattern = ["descendant-or-self::*[\@xml:id='" . $self->getLabelID($pattern) . "']",
    // 1]; } }   elsif ($op eq 'scope') {
    //     $op = 'select';
    //     if ($pattern =~ /^label:(.*)$/) {
    //       #      $pattern=["descendant-or-self::*[\@label='$1']",1]; }
    //       $pattern = ["descendant-or-self::*[\@xml:id='" . $self->getLabelID($1) . "']", 1]; }
    //     elsif ($pattern =~ /^id:(.*)$/) {
    //       $pattern = ["descendant-or-self::*[\@xml:id='$1']", 1]; }
    // ### Is this pattern ever used? <elementname>:<refnum> expects attribute!!!
    // ###    elsif ($pattern =~ /^(.*):(.*)$/) {
    // ###      $pattern = ["descendant-or-self::*[local-name()='$1' and \@refnum='$2']", 1]; }
    //     else {
    //       Error('misdefined', '<rewrite>', undef,
    //         "Unrecognized scope pattern in Rewrite clause: \"$pattern\"; Ignoring it.");
    //       $op = 'ignore'; $pattern = []; } }
    if op == RewriteOperator::Xpath {
      self.options.select_count = Some(1);
      return RewriteClause {
        compiled: true,
        op: RewriteOperator::Select,
        pattern,
      };
    }
    //   elsif ($op eq 'match') {
    //     if (ref $pattern eq 'CODE') {
    //       $op = 'test'; }
    //     elsif (ref $pattern eq 'ARRAY') {    # Multiple patterns!
    //       $op      = 'multi_select';
    //       $pattern = [map { $self->compile_match($document, $_) } @$pattern]; }
    //     else {
    //       $op = 'select'; $pattern = $self->compile_match($document, $pattern); } }
    //   elsif ($op eq 'replace') {
    //     if (ref $pattern eq 'CODE') { }
    //     else {
    //       $pattern = $self->compile_replacement($document, $pattern); } }
    //   elsif ($op eq 'regexp') {
    //     $pattern = $self->compile_regexp($pattern); }
    //   Debug("Compiled clause $oop=>" . ToString($opattern) . "  ==> $op=>" . ToString($pattern))
    //     if $LaTeXML::DEBUG{rewrite};
    RewriteClause {
      compiled: true,
      op,
      pattern,
    }
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
    if let Some(RewriteClause {
      compiled: _,
      op,
      pattern,
    }) = clauses.pop_front()
    {
      match op {
        Select => {
          // my ($xpath, $nnodes, @wilds) = @$pattern;
          if let RewritePattern::String(xpath) = pattern {
            let matches = document.findnodes(xpath, Some(tree));
            // Debug("Rewrite selecting \"$xpath\" => " . scalar(@matches) . " matches") if
            // $LaTeXML::DEBUG{rewrite};
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
          //     Debug("Rewrite replace at " . $tree->toString . " using $pattern")
          // if $LaTeXML::DEBUG{rewrite};
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
        _ => todo!(),
      }
    }

    Ok(())
  }
}

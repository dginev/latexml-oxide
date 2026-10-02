use std::{borrow::Cow, error::Error};

use latexml_core::{binding::def::dialect::get_xmarg_id, common::font::Font, document::Document};
use libxml::tree::{Node, NodeType};

use crate::{
  data::{get_grammatical_role, get_token_meaning, resolve_xmref},
  semantics::{
    ActionContext, XProps,
    tree::{XM, lookup_lex_node},
  },
};

/// Generate a textual token for each node; The parser operates on this encoded
/// string.
pub fn node_to_grammar_lexemes(mathnode: &Node, idx: &mut usize) -> (Vec<String>, Vec<Node>) {
  let child_nodes = filter_hints(mathnode.get_child_nodes());
  node_to_grammar_lexemes_from(mathnode, child_nodes, idx, None)
}

/// Same as `node_to_grammar_lexemes` but with pre-filtered child nodes.
/// Used when `filter_hints` has already been called (to avoid double-filtering). The document, when given, decodes the
/// tokens' fonts (a `d` token's identity, `DifferentialEvidence`).
pub fn node_to_grammar_lexemes_from(
  mathnode: &Node,
  child_nodes: Vec<Node>,
  idx: &mut usize,
  document: Option<&Document>,
) -> (Vec<String>, Vec<Node>) {
  let (mut lexemes, nodes) = node_to_grammar_lexemes_ctx(mathnode, child_nodes, idx, false);
  // Which letter `d`s can be an integral's differential: the grammar's differential branches (`diffunk`/`diffid`:
  // `diffop_apply`, `raised_differential_d`, grammar/builder.rs) are the only place `XDIFFUNK`/`XDIFFID` differ from a
  // plain `unknown`/`id`, so a `d` that is none is lexed as the plain letter and the branch is never built (M4: ≈71
  // and-nodes per `d<var>` in an INTOP-free formula, `\frac{dx}{dt}`, `d` as a variable). A `d` is a differential only
  // inside an integral's operand (user ruling 2026-10-01, SYNC (17); `in_an_integral_operand`; Perl's `diffd`, an
  // INTOP's argument only, MathGrammar:633-638), not anywhere in a formula holding an integral: `\int f\,dx\leq
  // d\pi^d` reads the second `d` a letter (2605.03853), as the raised `d` of divergence #395 was since 57cj.20 — unless
  // the formula or its document shows a differential outside (user rulings 2026-10-01: an SDE, a differential form, a
  // measure's set; the document's own uses, `DifferentialMap`; `DifferentialEvidence`).
  // A closed group holding only integrals is an integral operator (user ruling 2026-10-01; `integral_operator_group`):
  // its OPEN lexes `INTOP_GROUP_OPEN`, a big operator the grammar applies to the integrand after it
  // (`integral_operator_group`, grammar/builder.rs), and the `d`s of that integrand are its differentials
  // (`in_an_integral_operand`).
  if lexemes.iter().any(|l| l.starts_with("INTOP:"))
    && lexemes.iter().any(|l| l.starts_with("OPEN:"))
  {
    let levels = operand_levels(&nodes, true);
    for (open, lexeme) in lexemes.iter_mut().enumerate() {
      if lexeme.starts_with("OPEN:") && integral_operator_group(&nodes, &levels, open).is_some() {
        *lexeme = lexeme.replacen("OPEN:", "INTOP_GROUP_OPEN:", 1);
      }
    }
  }
  if lexemes.iter().any(|l| l.starts_with("XDIFF")) {
    let levels = operand_levels(&nodes, true);
    // (whether an earlier row left an integral open is asked only of a `d` the formula's own walk leaves open at its
    // start, once: `continues_an_open_integral` reads the document)
    let opened = std::cell::OnceCell::new();
    // (a root's or a fraction part's content inside an integral's operand is all in it, `in_an_enclosing_integrand`)
    let in_an_integrand = in_an_enclosing_integrand(mathnode);
    let outside: Vec<usize> = lexemes
      .iter()
      .enumerate()
      .filter(|(at, lex)| {
        lex.starts_with("XDIFF")
          && !in_an_integrand
          && !in_an_integral_operand(&nodes, &levels, *at, false)
          && !(in_an_integral_operand(&nodes, &levels, *at, true)
            && *opened.get_or_init(|| continues_an_open_integral(mathnode)))
      })
      .map(|(at, _)| at)
      .collect();
    let evidence = DifferentialEvidence::of(&nodes, &levels, &outside, document, mathnode, true);
    for at in outside {
      if evidence.reads_a_differential(at) {
        continue;
      }
      let lex = &mut lexemes[at];
      if let Some(rest) = lex.strip_prefix("XDIFFUNK:") {
        *lex = format!("UNKNOWN:{rest}");
      } else if let Some(rest) = lex.strip_prefix("XDIFFID:") {
        *lex = format!("ID:{rest}");
      }
    }
  }
  (lexemes, nodes)
}

/// Is the node at `at` inside an integral's operand (user ruling 2026-10-01, SYNC (17))? Perl reads `diffd` only among
/// an INTOP's arguments (`moreIntOpArgFactors`, MathGrammar:633-638); here, as the token stream shows it: an INTOP
/// before the node at its own level, with no relation, arrow or wide punctuation between at that level. The `d` keeps its
/// differential reading past a sign, where Perl's integrand ends (divergence #398; `\int_M a+b\,dV`,
/// `\int|\mathrm{d}P-\mathrm{d}Q|`, 2605.17001, 2605.15207: 697 such sites in 129 of the 3,003 A/B papers, all sum
/// integrands), past a colon, a product in an operand (the Frobenius `\int_\Omega\mathbf{H}:\mathbf{Q}_t\,d\mathbf{x}`,
/// 2605.24758, 2605.12883, 2605.19415), and past narrow punctuation, mostly a comma typed for `\,`
/// (`\int_0^L\bm u\cdot\bm e_x,dx`, 2605.21567, 2605.04013) — even where the grammar ends the integral's tree before it
/// (`\int f-g\,dx` reads integral@(f) − g·dx); wide punctuation (`,\quad`, a `\qquad`) separates formulas (2605.02034). A
/// group closed before the node is skipped whole — an integral inside it is no operand the node is in,
/// `(\int_0^1 f\,dx)\,d\pi`, `\left|\int f\,dx\right|d\pi`, nor does a relation inside it end anything,
/// `\int\mathbb{1}\{x\le y\}\,d^2x` —, and a group enclosing the node is left for the level outside it: the measures
/// `\int f\,P(y,dx')` (2605.08485), `\int K\tilde\mu(ds,de)` (2605.20593). A node inside a script is in none (an exponent's
/// `d`), nor is a `d` that a big operator in between binds, its index (`\int f\,dx+\sum_{d=1}^D d\,w_d`, the 57cj.20 review;
/// until a sign at that level ends the big operator's operand; `binds_the_letter_d`). The realized role is read, as a
/// gathered/split row's content branch lexes XMRefs whose INTOP is their target's (golden
/// tests/parse/integrals_and_differentials.tex#gathered_row_keeps_its_differential). A row that continues an integral an
/// earlier row of its alignment left open (`opened`, `continues_an_open_integral`) is inside it up to its first relation
/// (user ruling 2026-10-01: `&\times\Bigl[…\Bigr]\mathrm d\xi` after `=\int_0^T\Bigl[…`, 2605.15926).
pub(crate) fn in_an_integral_operand(
  nodes: &[Node],
  levels: &OperandLevels,
  at: usize,
  opened: bool,
) -> bool {
  integral_operand_reach(nodes, levels, at, opened).is_some()
}

/// `in_an_integral_operand`, and if so whether the walk alone reaches the node — a sign, punctuation or a plain bar
/// between the integral and the node at the integral's level (`\int f\,dx + C\,dh`, `\int f\,dx;\,d\pi`,
/// `|\int f\,dx|\,d\pi`: the grammar ends the integral's tree there; the node is in its operand by this walk only).
fn integral_operand_reach(
  nodes: &[Node],
  levels: &OperandLevels,
  at: usize,
  opened: bool,
) -> Option<bool> {
  let operand_levels = levels;
  let OperandLevels { roles, levels } = levels;
  let &(mut level) = levels.get(at)?;
  let mut ended = false;
  let mut signed = false;
  let mut weak = false;
  for at in (0..at).rev() {
    let (node, node_level, role) = (&nodes[at], levels[at], roles[at].as_str());
    if node_level > level {
      // inside a group closed before the node
      continue;
    }
    if node_level < level {
      // the opening of a group enclosing the node: an OPEN, a left bar, a script's start (an exponent's `d` is no
      // differential) or another role-carrying application's
      if matches!(
        role,
        "POSTSUBSCRIPT" | "POSTSUPERSCRIPT" | "FLOATSUBSCRIPT" | "FLOATSUPERSCRIPT"
      ) {
        return None;
      }
      level = node_level;
      ended = false;
      signed = false;
      weak = false;
      continue;
    }
    match role {
      "INTOP" if !ended => return Some(signed || weak),
      // (a group closed before the node that holds only integrals is one, `integral_operator_group`)
      "CLOSE"
        if !ended
          && (0..at)
            .rev()
            .find(|&open| levels[open] == node_level && roles[open] == "OPEN")
            .and_then(|open| integral_operator_group(nodes, operand_levels, open))
            == Some(at) =>
      {
        return Some(signed || weak);
      },
      "RELOP" | "ARROW" => ended = true,
      "METARELOP" if !is_a_colon(node) => ended = true,
      "PUNCT" if node.get_name() == "XMHint" || punct_followed_by_wide_space(node) => ended = true,
      "PUNCT" | "VERTBAR" => weak = true,
      "ADDOP" => signed = true,
      "SUMOP" | "BIGOP" | "LIMITOP" if !signed && binds_the_letter_d(nodes, at) => return None,
      _ => {},
    }
  }
  // … or before the formula, an integral an earlier row left open (`continues_an_open_integral`)
  (opened && !ended).then_some(signed || weak)
}

/// Is `container`, the content of a root or of a fraction's part, parsed on its own, inside an integral's operand in the
/// formula around it (57cj.23.7; the integrand outranks every scalar context, user ruling 2026-10-01e) —
/// `\int\sqrt{dP\,dQ}` (2605.09119), `\int\frac{dx}{2+x}` (ruling 2026-10-01c)? A sub-parse runs before the formula
/// around it, which is read raw; a script's content stays out (an exponent's `d`, `\int e^{-dx}\,dx`).
pub(crate) fn in_an_enclosing_integrand(container: &Node) -> bool {
  if container.get_name() != "XMArg" {
    return false;
  }
  let Some(app) = container.get_parent() else {
    return false;
  };
  let root_or_fraction = app.get_name() == "XMApp"
    && app.get_child_elements().first().is_some_and(|op| {
      matches!(
        op.get_attribute("meaning").as_deref(),
        Some("square-root" | "nth-root")
      ) || op.get_attribute("role").as_deref() == Some("FRACOP")
    });
  let Some(outer) = app.get_parent().filter(|_| root_or_fraction) else {
    return false;
  };
  let nodes: Vec<Node> = outer
    .get_child_elements()
    .into_iter()
    .filter(|node| node.get_name() != "XMHint")
    .collect();
  let Some(at) = nodes.iter().position(|node| *node == app) else {
    return false;
  };
  integral_operand_reach(&nodes, &operand_levels(&nodes, false), at, false).is_some()
    || in_an_enclosing_integrand(&outer)
}

/// The evidence that a letter `d` before a variable outside an integral's operand is a differential all the same (user
/// ruling 2026-10-01, widening SYNC (17); Perl reads it a letter, `diffd` being an INTOP's argument only,
/// MathGrammar:633-638). The lexer offers the differential (XDIFF) wherever the evidence holds — the grammar keeps both
/// readings and `LetterDsBeforeVariablesAreDifferentials` prefers the differential — and the plain letter elsewhere. The
/// evidence, each a shape of the corpus (A/B km21's 92 lost readings; precision sampled over the 3,003 papers), then the
/// document's (`DifferentialMap`; an upright `d` is no evidence alone since 57cj.23.6, user rulings 2026-10-01c/d/e):
/// - a scalar context of its own formula is none (`ArgContext::is_scalar`, `scalar_by_shape`, `in_an_order_argument`);
/// - a Leibniz or Radon-Nikodym quotient's part, italic too (`\frac{dy}{dx}`), a Fréchet derivative applied to a direction
///   (`\mathrm{d}H(u)[\psi]`, 2605.04766);
/// - two or more `d<var>` in the formula, no other `d` a variable there: an SDE or a differential form
///   (`dX_t=\mu\,dt+\sigma\,dW_t`, 2605.14643; `ds^2=dX^2+dY^2`, 2605.27600; `dJ/dK`; ~92 % of 70);
/// - a whole item of a group applied to a letter, a measure's set (`\pi(du):=`, 2605.02070; `F(du\mid x)`, 2605.18724;
///   `\lambda(\mathrm{d}z)`, `\widetilde N(ds,dz)`) — not after an order symbol, `O(dk^3)` a dimension (~85 % of 30);
/// - the opening of a formula's first side before a relation, with an integral in the formula (`dU(z)=-\int…`,
///   2605.26170; 45 of 45);
/// - beside a wedge, a differential form (`dx\wedge dy`; 29 of 29).
/// - after `\in`, an infinitesimal set (`\tau_1\in dt`; 2605.03594, 2605.11264, 2605.15276) — checked after the formula's own
///   scalar `d`; not before a set's letter the `d` scales (`k\in d\mathbb Z`, `y\in dZ`), nor where the document uses the
///   same `d` as a number unless the set is a measure's argument (`d\geq 2` … `q\in dn`; `P(X_t\in dx)`).
///
/// No `d` reads so that heads an italic word (`dist`: its one-letter variable runs on into another letter) or is Pearl's
/// `do(` (2605.31254); nor, as the only evidence of an upright or relation-opening `d`, one whose variable runs on into a
/// letter unspaced (a time step `\mathrm dt^2L_R^2`, 2605.29194); a dimension stays a letter (`\leq d\pi^d`, 2605.03853),
/// as does a `d` whose token the formula itself uses as a scalar (`scalar_here`). Past the formula: the same `d<var>` a
/// scalar elsewhere in the document is a letter, a differential elsewhere a differential (`\mathrm dS` beside
/// `\int_\Sigma f\,\mathrm dS`, 2605.16486); an upright `d` the document never uses as a scalar is a differential.
/// A raised `d` takes a variable only after an integer or one-letter order (`d^2x`, not `d^\top`).
struct DifferentialEvidence<'a> {
  nodes:                 &'a [Node],
  levels:                &'a OperandLevels,
  /// the lexer's stream (a script opens and closes around its content), or a formula's element children (a script is
  /// one node: the document map's reading, `DifferentialMap::read`)
  stream:                bool,
  /// the outside sites that are differential candidates (a variable after them, not in a script, no word head)
  candidates:            Vec<Candidate>,
  /// two or more candidates and no other `d` a variable in the formula
  a_form:                bool,
  /// an INTOP in the formula
  integral:              bool,
  /// a wedge past the formula's first top-level relation (`\mathrm d\eta^a=-\frac12 f\eta^b\wedge\eta^c`)
  wedge_past_a_relation: bool,
  document:              Option<&'a Document>,
  /// where the formula sits, a sub-parse's `XMArg` (`ArgContext`)
  context:               ArgContext,
  /// the `d` tokens (`d_identity`) this formula itself uses as a scalar: a lone `d` (`d\pi^d`'s exponent, a
  /// big operator's index `\sum_{d=1}`, `x\in\mathbb R^d`), a `d<var>` in a script outside an integrand
  scalar_here:           rustc_hash::FxHashSet<String>,
}

/// A `d` before a variable (`DifferentialEvidence`): where its variable and the variable's scripts end, and whether a
/// letter runs on after them unspaced.
struct Candidate {
  at:       usize,
  /// the variable's own token (`dW_t`'s `W`), for its name and font
  variable: usize,
  end:      usize,
  runs_on:  bool,
}

/// What a `d<var>`'s own formula shows (`DifferentialEvidence::local`), before its document is asked.
#[derive(Clone, Copy, PartialEq)]
enum Local {
  /// a differential: a Leibniz quotient's part, a measure's argument, a form, a wedge, a Fréchet direction, …
  Differential,
  /// a scalar context (a root, a script, a d-less denominator's part) or a scalar shape (`\to 0`, a number assigned):
  /// the document map records the `d` a scalar
  Scalar,
  /// a letter for this formula only (an order symbol's argument, a run-on variable, a bold `d`, the formula's own
  /// scalar `d`)
  Letter,
  /// nothing decides: `\in d<var>`, then the document
  Open,
}

impl<'a> DifferentialEvidence<'a> {
  fn of(
    nodes: &'a [Node],
    levels: &'a OperandLevels,
    outside: &[usize],
    document: Option<&'a Document>,
    mathnode: &Node,
    stream: bool,
  ) -> Self {
    let mut candidates = Vec::new();
    let mut a_variable = false;
    for &at in outside {
      match differential_variable_in(nodes, at, stream) {
        Some((variable, end)) => {
          if heads_a_word(nodes, at, end) {
            continue;
          }
          let runs_on = nodes.get(end).is_some_and(|next| {
            is_a_letter(next)
              && !(token_text(next) == "d"
                && differential_variable_in(nodes, end, stream).is_some())
          }) && !has_trailing_space(&nodes[end - 1]);
          let candidate = Candidate { at, variable, end, runs_on };
          // (a script's `d<var>` is none, but a measure's argument, which outranks the script: user ruling
          // 2026-10-02, `\|F\|_{L^1(d\mu)}`)
          if in_a_script(nodes, levels, at) && !is_a_measure_s_item(nodes, levels, &candidate) {
            continue;
          }
          candidates.push(candidate);
        },
        // a `d` with no variable is a variable itself, unless a group, an operator or a structure follows it
        // (`d(\mu_1\times\mu_2)`, `\mathrm d\frac{\partial L}{\partial M}`, `\mathrm d\langle W\rangle`)
        None => {
          a_variable |= !in_a_script(nodes, levels, at)
            && !nodes.get(at + 1).is_some_and(|next| {
              next.get_name() == "XMApp"
                || matches!(
                  levels.roles[at + 1].as_str(),
                  "OPEN" | "OPERATOR" | "OPFUNCTION" | "TRIGFUNCTION" | "FUNCTION" | "FRACOP"
                )
            })
        },
      }
    }
    let a_form = candidates.len() >= 2 && !a_variable;
    let integral = levels.roles.iter().any(|role| role == "INTOP");
    let wedge_past_a_relation = (0..nodes.len())
      .find(|&k| levels.levels[k] == 0 && levels.roles[k] == "RELOP")
      .is_some_and(|relation| nodes[relation + 1..].iter().any(is_a_wedge));
    let mut scalar_here = rustc_hash::FxHashSet::default();
    for k in 0..nodes.len() {
      if !matches!(nodes[k].get_name().as_str(), "XMTok" | "XMRef") || token_text(&nodes[k]) != "d"
      {
        continue;
      }
      // (a lone upright `d` is an operator or a label, never a scalar: `\frac{\mathrm d}{\mathrm dt}`, `\mathrm d^2=0`,
      // `M_{\rm d}`)
      let lone = differential_variable_in(nodes, k, stream).is_none();
      let scalar = if lone && is_upright(&nodes[k], document) {
        false
      } else if in_a_script(nodes, levels, k) {
        // (an integrand's `d` in an exponent is a differential, and a measure's argument one)
        integral_operand_reach(nodes, levels, k, false).is_none()
          && !candidates.iter().any(|candidate| candidate.at == k)
      } else {
        lone
          && !nodes.get(k + 1).is_some_and(|next| {
            next.get_name() == "XMApp"
              || matches!(
                levels.roles[k + 1].as_str(),
                "OPEN" | "OPERATOR" | "OPFUNCTION" | "TRIGFUNCTION" | "FUNCTION" | "FRACOP"
              )
          })
      };
      if scalar {
        scalar_here.insert(d_identity(&nodes[k], document));
      }
    }
    DifferentialEvidence {
      nodes,
      levels,
      stream,
      candidates,
      a_form,
      integral,
      wedge_past_a_relation,
      document,
      context: ArgContext::of(mathnode),
      scalar_here,
    }
  }

  /// The `(d token, variable)` pair of a candidate (`DifferentialMap`).
  fn pair(&self, candidate: &Candidate) -> (String, String) {
    (
      d_identity(&self.nodes[candidate.at], self.document),
      token_text(&self.nodes[candidate.variable]),
    )
  }

  /// Is the candidate a differential by the strong evidence of its formula's shape (user ruling 2026-10-01e): a form,
  /// a measure's item, a relation's first side with an integral or a wedge past the relation, beside a wedge, a Fréchet
  /// derivative applied to a direction?
  fn strong(&self, candidate: &Candidate) -> bool {
    let (nodes, levels, at) = (self.nodes, self.levels, candidate.at);
    self.a_form
      || is_a_measure_s_item(nodes, levels, candidate)
      || (self.integral || self.wedge_past_a_relation)
        && !candidate.runs_on
        && opens_the_first_side(levels, candidate)
      || at
        .checked_sub(1)
        .is_some_and(|before| is_a_wedge(&nodes[before]))
      || nodes.get(candidate.end).is_some_and(is_a_wedge)
      || applies_to_a_direction(nodes, levels, candidate)
  }

  /// What the candidate's own formula shows. The order of the evidence (user rulings 2026-10-01d/e and 2026-10-02,
  /// after the intent study of the 377 formulas the upright font alone made differentials): a Leibniz quotient's part,
  /// a fraction's or a slash's, whatever its container (`\frac{\sigma}{dE/dx}`, `\frac{1}{\mathrm ds/\mathrm d\lambda}`);
  /// a measure's argument in a script; a scalar context of the formula itself; then evidence its shape gives, whatever
  /// the font; then what the formula itself shows of its `d` token. The font decides nothing on its own: a document
  /// keeps one `d` token for its differentials.
  fn local(&self, candidate: &Candidate) -> Local {
    let (nodes, levels, at) = (self.nodes, self.levels, candidate.at);
    if self.context.is_leibniz() || in_a_slash_quotient(nodes, levels, at, self.stream) {
      return Local::Differential;
    }
    if self.context == ArgContext::Script && is_a_measure_s_item(nodes, levels, candidate) {
      return Local::Differential;
    }
    if self.context.is_scalar()
      || scalar_by_shape(nodes, at, candidate.end, self.candidates.len() > 1)
    {
      return Local::Scalar;
    }
    // (an order symbol's argument is a dimension or a step, unless the same `d<var>` is a differential elsewhere in the
    // formula: user ruling 2026-10-02, `\phi+d\phi … o(\|d\phi\|^2)`)
    if in_an_order_argument(nodes, levels, candidate) {
      let pair = self.pair(candidate);
      let elsewhere = self.candidates.iter().any(|other| {
        other.at != at
          && !in_an_order_argument(nodes, levels, other)
          && self.pair(other) == pair
          && self.strong(other)
      });
      return if elsewhere {
        Local::Differential
      } else {
        Local::Letter
      };
    }
    if self.strong(candidate) {
      return Local::Differential;
    }
    // A bold `d` bolder than its variable is the vector convention (`\mathbf{d}x`): only shape evidence makes it a
    // differential (user ruling 2026-10-01d: the bold font is not evidence), never its token's use in the document;
    // under `\boldmath` both are bold and the font stays evidence. What the formula itself shows of its `d` token
    // outranks the document (`\int_{[-1,1]^d}f\,dx\leq d\pi^d`, 2605.03853; `\sum_{d=1}^D d\,w_d`).
    if candidate.runs_on
      || self.bold_contrast(candidate)
      || self
        .scalar_here
        .contains(&d_identity(&nodes[at], self.document))
    {
      return Local::Letter;
    }
    Local::Open
  }

  /// Is the candidate's `d` bold and its variable not (`\mathbf{d}x`)?
  fn bold_contrast(&self, candidate: &Candidate) -> bool {
    // (the token's font as the document records it, `_font`, specialized as the serializer does: a plain `d` is italic)
    let bold = |node: &Node| {
      token_font(node, self.document)
        .is_some_and(|font| font.get_series().is_some_and(|series| series == "bold"))
    };
    bold(&self.nodes[candidate.at]) && !bold(&self.nodes[candidate.variable])
  }

  fn reads_a_differential(&self, at: usize) -> bool {
    let Some(candidate) = self.candidates.iter().find(|candidate| candidate.at == at) else {
      return false;
    };
    match self.local(candidate) {
      Local::Differential => return true,
      Local::Scalar | Local::Letter => return false,
      Local::Open => {},
    }
    let nodes = self.nodes;
    let variable = &nodes[candidate.variable];
    let variable_font = token_font(variable, self.document);
    let upright = is_upright(&nodes[at], self.document) && !self.bold_contrast(candidate);
    // `\in d<var>`, an infinitesimal set (`\tau_1\in dt`; 2605.03594, 2605.11264, 2605.15276) — behind the formula's
    // own scalar `d` (`d\geq 2,\ q\in dn`), and not before a set's letter, which the `d` scales: a blackboard,
    // calligraphic, fraktur, script or bold letter, or a capital (`k\in d\mathbb Z`, `x\in d\Lambda`, `y\in dZ`; the
    // 57cj.23.6 review)
    let set_letter = token_text(variable)
      .chars()
      .next()
      .is_some_and(char::is_uppercase)
      || variable_font
        .as_ref()
        .is_some_and(|font| font.get_series().is_some_and(|series| series == "bold"))
      || variable_font.as_ref().is_some_and(|font| {
        font.get_family().is_some_and(|family| {
          matches!(
            family.to_string().as_str(),
            "blackboard" | "caligraphic" | "fraktur" | "script"
          )
        })
      });
    if at
      .checked_sub(1)
      .is_some_and(|before| token_text(&nodes[before]) == "\u{2208}")
    {
      // (… and not where the document uses the same `d` as a number, `d\geq 2` … `q\in dn`, unless the set is a
      // measure's argument, `P(X_t\in dx)`)
      let a_number_here = crate::data::with_differential_map(|map| {
        map.is_some_and(|map| {
          map
            .lone_scalar_ids
            .contains(&d_identity(&nodes[at], self.document))
        })
      });
      return !set_letter
        && (!a_number_here || measure_group_open(nodes, self.levels, at, true).is_some());
    }
    let pair = self.pair(candidate);
    crate::data::with_differential_map(|map| match map {
      Some(map) if map.scalar_pairs.contains(&pair) => false,
      Some(map) if map.differential_pairs.contains(&pair) => true,
      // The token's identity decides for an upright `d`, a document's convention token (`\mathrm d`, `{\rm d}`; the
      // ruling's population): never a scalar in the document, a differential. The italic `d` is every document's default
      // letter as well: alone it needs its own pair integrated elsewhere (`\int f\,dx\le d^2n`, `abcde` stay letters).
      Some(map) => upright && !map.scalar_ids.contains(&pair.0),
      // (no document map, a lexer called outside the parser: the font is the document's convention)
      None => upright,
    })
  }
}

/// Where a sub-parse's content sits (`ArgContext::of`): the argument of a root or a script, a fraction's numerator or
/// denominator, or elsewhere.
#[derive(Clone, Copy, PartialEq)]
enum ArgContext {
  Root,
  Script,
  /// a fraction's part; `true` when both parts open with a `d` (a Leibniz or Radon-Nikodym quotient)
  Numerator(bool),
  /// a fraction's denominator; the flag as for the numerator, and whether the numerator holds a `d` at all
  Denominator(bool, bool),
  Other,
}

impl ArgContext {
  /// The context of `container`, an `XMArg` whose parent `XMApp` is a root, a script or a fraction.
  fn of(container: &Node) -> ArgContext {
    if container.get_name() != "XMArg" {
      return ArgContext::Other;
    }
    let Some(app) = container.get_parent() else {
      return ArgContext::Other;
    };
    if app.get_name() != "XMApp" {
      return ArgContext::Other;
    }
    if matches!(
      app.get_attribute("role").as_deref(),
      Some(
        "POSTSUBSCRIPT"
          | "POSTSUPERSCRIPT"
          | "SUBSCRIPT"
          | "SUPERSCRIPT"
          | "FLOATSUBSCRIPT"
          | "FLOATSUPERSCRIPT"
      )
    ) {
      // An under- or overbrace's label restates the braced term: no script (user ruling 2026-10-02;
      // `\underbrace{…}_{h_{\alpha\beta}\mathrm dy^\alpha\mathrm dy^\beta}`, 2605.15276)
      let a_brace = app.get_prev_element_sibling().is_some_and(|base| {
        base.get_name() == "XMApp"
          && base.get_child_elements().first().is_some_and(|op| {
            matches!(
              op.get_attribute("name").as_deref(),
              Some("underbrace" | "overbrace" | "underbracket" | "overbracket")
            ) || matches!(
              token_text(op).as_str(),
              "\u{23DF}" | "\u{23DE}" | "\u{23B5}" | "\u{23B4}"
            )
          })
      });
      return if a_brace {
        ArgContext::Other
      } else {
        ArgContext::Script
      };
    }
    let children: Vec<Node> = app.get_child_elements();
    let Some(op) = children.first() else {
      return ArgContext::Other;
    };
    if matches!(
      op.get_attribute("meaning").as_deref(),
      Some("square-root" | "nth-root")
    ) {
      return ArgContext::Root;
    }
    if op.get_attribute("role").as_deref() == Some("FRACOP") && children.len() == 3 {
      // (a part parsed already, or unwrapped to its one token, opens with its first visible token)
      let opens_with_d =
        |arg: &Node| first_visible_token(arg).is_some_and(|first| token_text(&first) == "d");
      // (a denominator is lexed before it is parsed: its `d` takes a variable, `\frac{dn}{d+2}` is no quotient)
      let opens_with_a_differential = |arg: &Node| {
        if arg.get_name() != "XMArg" {
          return opens_with_d(arg);
        }
        let nodes: Vec<Node> = arg
          .get_child_elements()
          .into_iter()
          .filter(|node| node.get_name() != "XMHint")
          .collect();
        nodes.first().is_some_and(|first| token_text(first) == "d")
          && raw_differential_variable(&nodes, 0).is_some()
      };
      // A Leibniz numerator is a differential operator or a differential (`numerator_of_a_quotient`), read raw —
      // when the numerator is lexed, before it is parsed —, and recorded on the fraction for the denominator, lexed
      // after the numerator's parse (`_lx_leibniz`, a bookkeeping attribute the finalize drops).
      let numerator_is_a_differential = |arg: &Node| {
        // (the verdict recorded when the numerator was read raw, so its two parts always agree)
        if let Some(recorded) = app.get_attribute("_lx_leibniz") {
          return recorded == "1";
        }
        if arg.get_name() == "XMArg" {
          let nodes: Vec<Node> = arg
            .get_child_elements()
            .into_iter()
            .filter(|node| node.get_name() != "XMHint")
            .collect();
          return numerator_of_a_quotient(&nodes);
        }
        // (unwrapped to its one token: a lone `d`)
        arg.get_name() == "XMTok" && token_text(arg) == "d"
      };
      let numerator = numerator_is_a_differential(&children[1]);
      let leibniz = numerator && opens_with_a_differential(&children[2]);
      if children[1] == *container {
        let mut app = app.clone();
        let _ = app.set_attribute("_lx_leibniz", if numerator { "1" } else { "0" });
        return ArgContext::Numerator(leibniz);
      }
      if children[2] == *container {
        // (the numerator, parsed by now, anywhere in its tree: `\frac{p\,\mathrm dV}{\mathrm dt}`)
        let mut tokens = Vec::new();
        collect_named(&children[1], "XMTok", &mut tokens);
        let holds_d = children[1].get_name() == "XMTok" && token_text(&children[1]) == "d"
          || tokens.iter().any(|node| token_text(node) == "d");
        return ArgContext::Denominator(leibniz, holds_d);
      }
    }
    ArgContext::Other
  }

  /// A scalar context (user ruling 2026-10-01e): a root, a script, a denominator whose numerator holds no `d`
  /// (`\sqrt{\mathrm dt}`, `x_{t+\mathrm dt}`, `\frac{3}{2\,\mathrm dt}`, 2605.09779, 2605.00250, 2605.29194).
  fn is_scalar(self) -> bool {
    matches!(
      self,
      ArgContext::Root | ArgContext::Script | ArgContext::Denominator(false, false)
    )
  }

  /// A Leibniz or Radon-Nikodym quotient's part (`\frac{dy}{dx}`, `\frac{\mathrm d}{\mathrm dt}`; ruling 2026-10-01d:
  /// a d-fraction is evidence, italic too).
  fn is_leibniz(self) -> bool {
    matches!(
      self,
      ArgContext::Numerator(true) | ArgContext::Denominator(true, _)
    )
  }
}

/// Is the token's font upright (as the document records it)?
fn is_upright(node: &Node, document: Option<&Document>) -> bool {
  token_font(node, document)
    .is_some_and(|font| font.get_shape().is_some_and(|shape| shape == "upright"))
}

/// A token's font as the document records it (`_font`, through an XMRef), specialized for its text as the serializer
/// does (a plain `d` is italic): the one decoding behind `d_identity`, `is_upright` and the evidence's font tests.
fn token_font(node: &Node, document: Option<&Document>) -> Option<Font> {
  let node = resolve_xmref(node).unwrap_or_else(|| node.clone());
  let hash = node.get_attribute("_font")?;
  document?
    .decode_font(&hash)
    .map(|font| font.specialize(&token_text(&node)))
}

/// Is the raw content of a fraction's numerator (its element children) a differential operator or a differential, as
/// a Leibniz quotient's numerator: a `d`, with its power (`d`, `d^2`), before a variable (`dy`, `d^2y`), an accented
/// letter (`d\vec r`), a function (`d\ln Z`, `d\sin\phi`) or a group holding no `d` with nothing after it but
/// factors (`d(uv)`, `d\left(q^2\right)`, `d|\psi\rangle`, `d\langle A\rangle`) — not a sum or a product that merely
/// opens with `d` (`\frac{d-1}{dk}`, `\frac{d(d+1)}{dn}`; the 57cj.23.6 review).
fn numerator_of_a_quotient(nodes: &[Node]) -> bool {
  if !nodes.first().is_some_and(|first| token_text(first) == "d") {
    return false;
  }
  let mut k = 1;
  while nodes.get(k).is_some_and(is_a_script_marker) {
    k += 1;
  }
  if k == nodes.len() || raw_differential_variable(nodes, 0).is_some() {
    return true;
  }
  let first = &nodes[k];
  let role = get_grammatical_role(first);
  let accent = first.get_name() == "XMApp"
    && first.get_child_elements().first().is_some_and(|op| {
      matches!(
        op.get_attribute("role").as_deref(),
        Some("OVERACCENT" | "UNDERACCENT")
      )
    });
  if accent || matches!(role.as_str(), "OPFUNCTION" | "TRIGFUNCTION" | "OPERATOR") {
    return true;
  }
  if !(matches!(role.as_str(), "OPEN" | "VERTBAR") || token_text(first) == "\u{27E8}") {
    return false;
  }
  let holds_a_d = nodes[k..].iter().any(|node| {
    let mut tokens = Vec::new();
    if node.get_name() == "XMTok" {
      tokens.push(node.clone());
    } else {
      collect_named(node, "XMTok", &mut tokens);
    }
    tokens.iter().any(|token| token_text(token) == "d")
  });
  if holds_a_d {
    return false;
  }
  // nothing but factors after the group: no sign or relation outside it
  let mut depth: i32 = 0;
  for node in &nodes[k..] {
    match get_grammatical_role(node).as_str() {
      "OPEN" => depth += 1,
      "CLOSE" => depth -= 1,
      "ADDOP" | "RELOP" | "METARELOP" | "ARROW" if depth <= 0 => return false,
      _ => {},
    }
  }
  true
}

/// The first token of `node` in reading order that prints (an application's invisible times skipped).
fn first_visible_token(node: &Node) -> Option<Node> {
  if node.get_name() == "XMTok" {
    return (token_text(node) != "\u{2062}" && !token_text(node).is_empty()).then(|| node.clone());
  }
  node
    .get_child_elements()
    .iter()
    .find_map(first_visible_token)
}

/// A `d` token's identity in its document: its font's family, series and shape — not its size, which a script
/// changes —, the font a document keeps for its differential `d` (user ruling 2026-10-01e: the same `d` token never
/// used as a scalar in the document is a differential).
fn d_identity(node: &Node, document: Option<&Document>) -> String {
  match token_font(node, document) {
    Some(font) => {
      format!(
        "{}/{}/{}",
        font.get_family().map(|f| f.to_string()).unwrap_or_default(),
        font.get_series().map(|f| f.to_string()).unwrap_or_default(),
        font.get_shape().map(|f| f.to_string()).unwrap_or_default()
      )
    },
    None => resolve_xmref(node)
      .unwrap_or_else(|| node.clone())
      .get_attribute("_font")
      .unwrap_or_default(),
  }
}

/// Is `d<var>` at `at` (variable ending at `end`) a scalar by its formula's own shape: `d<var>\to 0`, or a side
/// assigned a nonzero number (`\mathrm dt=1/12`, `{\rm d}r=2.3`) when no other `d<var>` is in the formula — an
/// equation between differentials outranks it (user ruling 2026-10-01e; 2605.09717 keeps its SDE)?
fn scalar_by_shape(nodes: &[Node], at: usize, end: usize, other_candidates: bool) -> bool {
  let next = |k: usize| nodes.get(k).map(token_text).unwrap_or_default();
  if next(end) == "\u{2192}" && next(end + 1) == "0" {
    return true;
  }
  if other_candidates || at != 0 || next(end) != "=" {
    return false;
  }
  let rest = &nodes[end + 1..];
  !rest.is_empty()
    && rest.iter().all(|node| {
      matches!(
        get_grammatical_role(node).as_str(),
        "NUMBER" | "ADDOP" | "MULOP" | "PUNCT"
      ) || node.get_name() == "XMApp"
        && node
          .get_child_elements()
          .first()
          .is_some_and(|op| op.get_attribute("role").as_deref() == Some("FRACOP"))
    })
    && rest.iter().any(|node| {
      get_grammatical_role(node) == "NUMBER" && token_text(node) != "0"
        || node.get_name() == "XMApp"
    })
}

/// What a document's formulas show about its `d` tokens, read before any is parsed (user rulings 2026-10-01d/e):
/// which `d` identities are ever a scalar — a `d<var>` in a scalar context —, and
/// which `(identity, variable)` pairs are a scalar or a differential (an integrand, a Leibniz quotient) somewhere.
#[derive(Default)]
pub(crate) struct DifferentialMap {
  scalar_ids:         rustc_hash::FxHashSet<String>,
  /// the identities a lone `d` is a scalar of somewhere (`d\geq 2`; italic only, an upright lone `d` being an
  /// operator): the evidence `\in d<var>` gives yields to them
  lone_scalar_ids:    rustc_hash::FxHashSet<String>,
  scalar_pairs:       rustc_hash::FxHashSet<(String, String)>,
  differential_pairs: rustc_hash::FxHashSet<(String, String)>,
}

impl DifferentialMap {
  pub(crate) fn of(xmaths: &[Node], document: &Document) -> DifferentialMap {
    let mut map = DifferentialMap::default();
    map.absorb(xmaths, document);
    map
  }

  /// Add what `xmaths` show.
  pub(crate) fn absorb(&mut self, xmaths: &[Node], document: &Document) {
    for xmath in xmaths {
      let mut containers = vec![xmath.clone()];
      collect_named(xmath, "XMArg", &mut containers);
      for container in &containers {
        self.read(container, document);
      }
    }
  }

  fn read(&mut self, container: &Node, document: &Document) {
    // (the element children, hints skipped: `filter_hints` accumulates spacing onto the tokens, which the
    // parse reads later, so this pre-pass must not run it)
    let nodes: Vec<Node> = container
      .get_child_elements()
      .into_iter()
      .filter(|node| node.get_name() != "XMHint")
      .collect();
    // (a `d` token, or a reference to one, as the lexer realizes it)
    let is_a_d = |node: &Node| {
      matches!(node.get_name().as_str(), "XMTok" | "XMRef") && token_text(node) == "d"
    };
    if !nodes.iter().any(is_a_d) {
      return;
    }
    let context = ArgContext::of(container);
    let levels = operand_levels(&nodes, false);
    let pairs: Vec<(usize, usize, usize)> = (0..nodes.len())
      .filter(|&k| is_a_d(&nodes[k]))
      .filter_map(|k| {
        raw_differential_variable(&nodes, k).map(|(variable, end)| (k, variable, end))
      })
      .collect();
    // A lone `d` is a scalar of an italic token (`d\geq 2`, `\mathbb R^d`), unless a group, an operator or a structure
    // follows it past its own scripts; an upright one is an operator or a label, never a scalar (`\frac{\mathrm
    // d}{\mathrm dt}`, `\mathrm d^2=0`, `M_{\rm d}`; the 57cj.23.6 review). Only the `\in` evidence reads it.
    for k in (0..nodes.len()).filter(|&k| is_a_d(&nodes[k])) {
      // (a Leibniz quotient's lone `d` is its operator, `\frac{d}{dt}`)
      if pairs.iter().any(|(at, ..)| *at == k)
        || is_upright(&nodes[k], Some(document))
        || context.is_leibniz()
      {
        continue;
      }
      let next = (k + 1..)
        .find(|&j| nodes.get(j).is_none_or(|node| !is_a_script_marker(node)))
        .unwrap_or(k + 1);
      // (a bar introduces a group only when a bar or a `⟩` closes it later: `d|\psi\rangle`, `d|x|` — `d|n` is
      // divisibility, its `d` a number)
      let closed_bar = get_grammatical_role(&nodes[next.min(nodes.len().saturating_sub(1))])
        == "VERTBAR"
        && nodes.get(next + 1..).is_some_and(|after| {
          after
            .iter()
            .any(|node| get_grammatical_role(node) == "VERTBAR" || token_text(node) == "\u{27E9}")
        });
      let introduces = closed_bar
        || nodes.get(next).is_some_and(|next| {
          next.get_name() == "XMApp"
            || token_text(next) == "\u{27E8}"
            || matches!(
              get_grammatical_role(next).as_str(),
              "OPEN" | "OPERATOR" | "OPFUNCTION" | "TRIGFUNCTION" | "FUNCTION" | "FRACOP"
            )
        });
      if !introduces {
        self
          .lone_scalar_ids
          .insert(d_identity(&nodes[k], Some(document)));
      }
    }
    // The formula's own evidence for the `d<var>` outside its integrals, read as the lexer reads it (one verdict,
    // `DifferentialEvidence::local`): a differential by any local evidence is the document's pair (57cj.23.7: an
    // equation between differentials, a measure, a wedge, a form, a Leibniz quotient in any container; 2605.00250),
    // a scalar context's `d` its scalar.
    let in_an_integrand = in_an_enclosing_integrand(container);
    let outside: Vec<usize> = pairs
      .iter()
      .map(|&(at, ..)| at)
      .filter(|&at| integral_operand_reach(&nodes, &levels, at, false).is_none())
      .collect();
    let evidence =
      DifferentialEvidence::of(&nodes, &levels, &outside, Some(document), container, false);
    for &(at, variable, _) in &pairs {
      let id = d_identity(&nodes[at], Some(document));
      let pair = (id.clone(), token_text(&nodes[variable]));
      // An integrand's `d` is a differential wherever the integral sits (an exponent's `e^{-\int r\,ds}`, a root's
      // `\sqrt{\int|f|^2dx}`, a root or a fraction inside the integrand, `\int\sqrt{dP\,dQ}`), and never a scalar; it is
      // evidence only where the integral reaches it cleanly — past a sign, punctuation or a plain bar the walk alone
      // puts it in the operand (`\int f\,dx+C\,dh`, `\int f\,dx;\,d\pi`), and the reading would spread to every
      // `d<var>` of the document.
      if let Some(walked) = integral_operand_reach(&nodes, &levels, at, false) {
        if !walked {
          self.differential_pairs.insert(pair);
        }
        continue;
      }
      if in_an_integrand {
        self.differential_pairs.insert(pair);
        continue;
      }
      let Some(candidate) = evidence
        .candidates
        .iter()
        .find(|candidate| candidate.at == at)
      else {
        continue;
      };
      match evidence.local(candidate) {
        Local::Differential => {
          self.differential_pairs.insert(pair);
        },
        Local::Scalar => {
          self.scalar_ids.insert(id);
          self.scalar_pairs.insert(pair);
        },
        Local::Letter | Local::Open => {},
      }
    }
  }
}

/// The descendants of `node` named `name`, in document order.
pub(crate) fn collect_named(node: &Node, name: &str, found: &mut Vec<Node>) {
  for child in node.get_child_elements() {
    if child.get_name() == name {
      found.push(child.clone());
    }
    collect_named(&child, name, found);
  }
}

/// The realized text of a token (through an XMRef).
fn token_text(node: &Node) -> String {
  resolve_xmref(node)
    .unwrap_or_else(|| node.clone())
    .get_content()
}

/// A letter: an UNKNOWN or ID token of one character (`x`, `\theta`, `\mathbf x`).
fn is_a_letter(node: &Node) -> bool {
  matches!(get_grammatical_role(node).as_str(), "UNKNOWN" | "ID")
    && node.get_name() != "XMApp"
    && token_text(node).chars().count() == 1
}

/// An explicit space after `node` (`filter_hints` folds `\,`, `\;`, `\ ` onto the token or script before it).
fn has_trailing_space(node: &Node) -> bool {
  resolve_xmref(node)
    .unwrap_or_else(|| node.clone())
    .get_attribute("rpadding")
    .is_some_and(|width| get_xmhint_spacing(&width) > 0.0)
}

/// A script's bracketing node in the lexer's stream (its first occurrence opens it, its second closes it).
fn is_a_script_marker(node: &Node) -> bool {
  node.get_name() == "XMApp"
    && matches!(
      node.get_attribute("role").as_deref(),
      Some("POSTSUBSCRIPT" | "POSTSUPERSCRIPT" | "FLOATSUBSCRIPT" | "FLOATSUPERSCRIPT")
    )
}

/// Past the scripts starting at `at` in the lexer's stream: the index after the last one's closing occurrence.
fn past_scripts(nodes: &[Node], mut at: usize) -> usize {
  while let Some(marker) = nodes.get(at)
    && is_a_script_marker(marker)
  {
    match nodes[at + 1..].iter().position(|node| node == marker) {
      Some(end) => at = at + 1 + end + 1,
      None => return at + 1,
    }
  }
  at
}

/// `differential_variable` over a formula's element children, where a script is one `XMApp` (the lexer's stream opens
/// and closes it around its content): the document map's reading (`DifferentialMap::read`).
fn raw_differential_variable(nodes: &[Node], at: usize) -> Option<(usize, usize)> {
  let mut variable = at + 1;
  let marker = nodes.get(variable)?;
  if is_a_script_marker(marker) {
    if marker.get_attribute("role").as_deref() != Some("POSTSUPERSCRIPT") {
      return None;
    }
    let mut order = Vec::new();
    collect_named(marker, "XMTok", &mut order);
    let integer_or_letter = |node: &Node| {
      let text = token_text(node);
      get_grammatical_role(node) == "NUMBER" && text.chars().all(|c| c.is_ascii_digit())
        || is_a_letter(node)
    };
    if !matches!(order.as_slice(), [only] if integer_or_letter(only)) {
      return None;
    }
    variable += 1;
  }
  let token = nodes.get(variable)?;
  is_a_letter(token).then(|| {
    let end = (variable + 1..)
      .find(|&j| nodes.get(j).is_none_or(|node| !is_a_script_marker(node)))
      .unwrap_or(variable + 1);
    (variable, end)
  })
}

/// The variable of a `d` at `at` and where the variable's scripts end (`DifferentialEvidence`): a letter, after a raised
/// order that is an integer or one letter (`d^2x`, `d^nx`; not `d^\top x`, `d^{-1}x`) — `dW_t`'s `W`, `d^2x`'s `x`; None
/// when no variable follows.
fn differential_variable(nodes: &[Node], at: usize) -> Option<(usize, usize)> {
  let mut variable = at + 1;
  let marker = nodes.get(variable)?;
  if is_a_script_marker(marker) {
    if marker.get_attribute("role").as_deref() != Some("POSTSUPERSCRIPT") {
      return None;
    }
    let close = variable
      + 1
      + nodes[variable + 1..]
        .iter()
        .position(|node| node == marker)?;
    let order = &nodes[variable + 1..close];
    let integer_or_letter = |node: &Node| {
      let text = token_text(node);
      get_grammatical_role(node) == "NUMBER" && text.chars().all(|c| c.is_ascii_digit())
        || is_a_letter(node)
    };
    if !matches!(order, [only] if integer_or_letter(only)) {
      return None;
    }
    variable = close + 1;
  }
  let token = nodes.get(variable)?;
  is_a_letter(token).then(|| (variable, past_scripts(nodes, variable + 1)))
}

/// `differential_variable` in the lexer's `stream`, `raw_differential_variable` over a formula's element children.
fn differential_variable_in(nodes: &[Node], at: usize, stream: bool) -> Option<(usize, usize)> {
  if stream {
    differential_variable(nodes, at)
  } else {
    raw_differential_variable(nodes, at)
  }
}

/// Is the `d<var>` at `at` a part of a slash Leibniz quotient (57cj.23.7; the ruled Leibniz evidence, 2026-10-01d/e) —
/// `dE'/dx` (2605.21289), `\mathrm dh(s)/\mathrm ds` (2605.09779), `\mathrm ds/\mathrm d\lambda` (2605.29065): a `/` at
/// its level, with only factors between, whose numerator — from its operand's start — is a quotient's
/// (`numerator_of_a_quotient`) and whose denominator opens with a `d<var>`?
fn in_a_slash_quotient(nodes: &[Node], levels: &OperandLevels, at: usize, stream: bool) -> bool {
  let level = levels.levels[at];
  let ends_an_operand = |k: usize| {
    levels.levels[k] < level
      || levels.levels[k] == level
        && matches!(
          levels.roles[k].as_str(),
          "ADDOP" | "RELOP" | "METARELOP" | "ARROW" | "PUNCT"
        )
  };
  let is_a_slash = |k: usize| {
    levels.levels[k] == level && levels.roles[k] == "MULOP" && token_text(&nodes[k]) == "/"
  };
  let start = (0..at)
    .rev()
    .find(|&k| ends_an_operand(k))
    .map_or(0, |k| k + 1);
  let a_numerator = |slash: usize| {
    if stream {
      // (the lexer's stream: a `d` alone or before its variable)
      token_text(&nodes[start]) == "d"
        && (start + 1 == slash
          || differential_variable(nodes, start).is_some_and(|(_, end)| end <= slash))
    } else {
      numerator_of_a_quotient(&nodes[start..slash])
    }
  };
  if at > start && is_a_slash(at - 1) {
    return a_numerator(at - 1);
  }
  at == start
    && (at + 1..nodes.len())
      .take_while(|&k| !ends_an_operand(k))
      .find(|&k| is_a_slash(k))
      .is_some_and(|slash| {
        a_numerator(slash)
          && nodes
            .get(slash + 1)
            .is_some_and(|next| token_text(next) == "d")
          && differential_variable_in(nodes, slash + 1, stream).is_some()
      })
}

/// Does the `d` at `at`, its variable ending at `end`, head an italic word — its one-letter, unscripted variable running on
/// unspaced into another letter other than a `d` (`dist`, `dim`, `depth`) — or spell Pearl's `do(` (2605.31254)?
fn heads_a_word(nodes: &[Node], at: usize, end: usize) -> bool {
  let variable = &nodes[end - 1];
  if end == at + 2
    && token_text(variable) == "o"
    && nodes.get(end).is_some_and(|next| token_text(next) == "(")
  {
    return true;
  }
  end == at + 2
    && token_text(variable)
      .chars()
      .all(|c| c.is_ascii_alphabetic())
    && !has_trailing_space(variable)
    && nodes.get(end).is_some_and(|next| {
      is_a_letter(next)
        && token_text(next).chars().all(|c| c.is_ascii_alphabetic())
        && token_text(next) != "d"
    })
}

/// Is the node at `at` inside a script (an exponent's `d`, `\mathbb R^d`)?
fn in_a_script(nodes: &[Node], levels: &OperandLevels, at: usize) -> bool {
  let mut level = levels.levels[at];
  for k in (0..at).rev() {
    if levels.levels[k] < level {
      if is_a_script_marker(&nodes[k]) {
        return true;
      }
      level = levels.levels[k];
    }
  }
  false
}

/// The `(` or `[` (with `event`, an `\in` set's: also `{`, or after a `\Pr`) of the group applied to a letter that the
/// node at `at` sits in, at its own level (a measure's
/// argument: `\pi(du)`, `P(X_t\in dx)`) — not after an order symbol (`O(dk^3)`, `\mathcal O(d\delta^2)`, `\Theta`,
/// `\Omega`, a constant `C`).
fn measure_group_open(
  nodes: &[Node],
  levels: &OperandLevels,
  at: usize,
  event: bool,
) -> Option<usize> {
  let level = levels.levels[at];
  if level == 0 {
    return None;
  }
  let open = (0..at).rev().find(|&k| levels.levels[k] < level)?;
  if levels.roles[open] != "OPEN"
    || !(matches!(token_text(&nodes[open]).as_str(), "(" | "[")
      || event && token_text(&nodes[open]) == "{")
    || open == 0
  {
    return None;
  }
  // the head: a letter, or a scripted one (its base before the script). An order symbol (`O`, `o`, `𝒪`, Θ, Ω) or `C`
  // heads a dimension's argument, `O(dk^3)`; the list is glyph-anchored and provisional (open categories,
  // OXIDIZED_DESIGN_MATH): Ω and `C` also head measures (`\Omega(dx)`), read letters here.
  // (in the lexer's stream a script closes with the node that opened it; among a formula's children it is one node)
  let mut head = open - 1;
  while is_a_script_marker(&nodes[head]) {
    head = match nodes[..head].iter().position(|node| *node == nodes[head]) {
      Some(start) => start.checked_sub(1)?,
      None => head.checked_sub(1)?,
    };
  }
  // (an event's — a `\in` set's — group may also be a probability's `\Pr(Z\in dz)`, an OPFUNCTION, or open with a
  // brace, `\mathbb P\{Y\in dv\}`; elsewhere those are `\log(dk)`, `f\{dn\}`, the 57cj.23.6 review)
  if !(is_a_letter(&nodes[head]) || event && levels.roles[head] == "OPFUNCTION")
    || matches!(
      token_text(&nodes[head]).as_str(),
      "d" | "O" | "o" | "\u{1D4AA}" | "\u{0398}" | "\u{03A9}" | "C"
    )
  {
    return None;
  }
  Some(open)
}

/// Is the candidate a whole item of a group applied to a letter — a measure's set, `\pi(du)`, `F(du\mid x)`, `\widetilde
/// N(ds,dz)` — and not after an order symbol (`O(dk^3)`, `\mathcal O(d\delta^2)`, `\Theta`, `\Omega`, a constant `C`)?
fn is_a_measure_s_item(nodes: &[Node], levels: &OperandLevels, candidate: &Candidate) -> bool {
  let level = levels.levels[candidate.at];
  let Some(open) = measure_group_open(nodes, levels, candidate.at, false) else {
    return false;
  };
  let separates = |k: usize| {
    levels.roles[k] == "PUNCT" && matches!(token_text(&nodes[k]).as_str(), "," | ";")
      || matches!(token_text(&nodes[k]).as_str(), "\u{2223}" | "|")
  };
  let before = (open + 1..candidate.at)
    .rev()
    .find(|&k| levels.levels[k] == level);
  let after = candidate.end;
  before.is_none_or(separates)
    && nodes.get(after).is_some_and(|_| {
      levels.levels[after] == level && separates(after)
        || levels.levels[after] < level && levels.roles[after] == "CLOSE"
    })
}

/// Is the candidate inside the argument of an order symbol — `\mathcal O(dh)`, `O(dk^3)`, `o(\sqrt{dt})` — a
/// dimension or a step, a scalar (user ruling 2026-10-01e, Landau argument; 2605.00250)?
fn in_an_order_argument(nodes: &[Node], levels: &OperandLevels, candidate: &Candidate) -> bool {
  let level = levels.levels[candidate.at];
  if level == 0 {
    return false;
  }
  let Some(open) = (0..candidate.at).rev().find(|&k| levels.levels[k] < level) else {
    return false;
  };
  levels.roles[open] == "OPEN"
    && open > 0
    && matches!(
      token_text(&nodes[open - 1]).as_str(),
      "O" | "o" | "\u{1D4AA}"
    )
}

/// Is the candidate a Fréchet derivative applied to a direction — `\mathrm dH(u)[\psi]`, `dF(x)[h]` (2605.04766):
/// its variable takes an argument group and then a bracketed one?
fn applies_to_a_direction(nodes: &[Node], levels: &OperandLevels, candidate: &Candidate) -> bool {
  let opens = |k: usize, text: &str| {
    levels.roles.get(k).is_some_and(|role| role == "OPEN") && token_text(&nodes[k]) == text
  };
  let mut k = candidate.end;
  if !opens(k, "(") {
    return false;
  }
  let level = levels.levels[k];
  let Some(close) = (k + 1..nodes.len()).find(|&j| levels.levels[j] == level) else {
    return false;
  };
  k = close + 1;
  opens(k, "[")
}

/// Does the candidate open a formula's first side — only signs before it — with a relation after it at the top level
/// (`dU(z)=-\int…`, 2605.26170)?
fn opens_the_first_side(levels: &OperandLevels, candidate: &Candidate) -> bool {
  levels.levels[candidate.at] == 0
    && levels.roles[..candidate.at]
      .iter()
      .all(|role| role == "ADDOP")
    && (candidate.end..levels.roles.len())
      .any(|k| levels.levels[k] == 0 && levels.roles[k] == "RELOP")
}

/// A wedge, ∧ (`dx\wedge dy`).
fn is_a_wedge(node: &Node) -> bool { token_text(node) == "\u{2227}" }

/// Does the formula `mathnode` (an `ltx:Math`'s XMath) continue an integral the row before it left open — an integral
/// split across the rows of an alignment (user ruling 2026-10-01, widening SYNC (17): continuation rows count as inside
/// the integral; 2605.15926, 2605.12025, 2605.26008, 2605.30839)? Only a continuation row continues: one whose first
/// cell is empty or which opens with a sign or a MulOp (`&\quad\times b(t)\Bigr]\mathrm dt`), in an alignment, not a
/// gather (its rows are formulas of their own); and only the row right before it counts (a row with its own left side,
/// `dk &\le n` after `\|u\|^2 &= \int_\Omega|u|^2`, keeps its letters; the merge review's finding). A cell counts the
/// cells before it in its own row too (`\int_0^T & \langle…\rangle\mathrm d\xi`, 2605.15926). The rows are parsed
/// already: each INTOP opens an integral, each differential closes one (a `d` before a letter where the row did not
/// parse), never below none. Called only for a `d` the formula leaves open at its start (`node_to_grammar_lexemes_from`).
fn continues_an_open_integral(mathnode: &Node) -> bool {
  let Some(math) = mathnode
    .get_parent()
    .filter(|parent| parent.get_name() == "Math")
  else {
    return false;
  };
  let Some(equation) = ancestor_named(&math, "equation") else {
    return false;
  };
  let in_an_alignment = ancestor_named(&equation, "equationgroup").is_some_and(|group| {
    !group
      .get_attribute("class")
      .is_some_and(|class| class.contains("gather"))
  });
  if !in_an_alignment {
    return false;
  }
  let parent = math.get_parent();
  let mut open = 0usize;
  if let Some(td) = parent.filter(|parent| parent.get_name() == "td") {
    // a cell: its column pair (an `&`-pair of an alignment, `lhs & rhs`), in its row and the row before
    let Some(row) = td.get_parent() else {
      return false;
    };
    let tds = element_children(&row, "td");
    let Some(column) = tds.iter().position(|cell| *cell == td) else {
      return false;
    };
    let pair = column_pair(&tds, column);
    if is_a_continuation(&pair)
      && let Some(previous) = previous_cell_row(&row)
    {
      for formula in pair_formulas(&column_pair(&element_children(&previous, "td"), column)) {
        open = integral_balance(&formula, open);
      }
    }
    for formula in pair_formulas(&pair) {
      if formula == math {
        break;
      }
      open = integral_balance(&formula, open);
    }
  } else {
    // a row's formula: its MathFork's, the k-th of the equation's (one per column pair), or the equation's own
    let forks = element_children(&equation, "MathFork");
    let k = forks
      .iter()
      .position(|fork| math.get_parent().is_some_and(|parent| parent == *fork))
      .unwrap_or(0);
    let continuation = match forks.get(k) {
      Some(fork) => {
        let branch_cells = element_children(fork, "MathBranch")
          .first()
          .map(|branch| {
            let rows = element_children(branch, "tr");
            element_children(rows.first().unwrap_or(branch), "td")
          })
          .unwrap_or_default();
        is_a_continuation(&branch_cells) || starts_with_a_joining_operator(&math)
      },
      None => starts_with_a_joining_operator(&math),
    };
    let previous = previous_element(&equation, "equation");
    if continuation && let Some(previous) = previous {
      let previous_forks = element_children(&previous, "MathFork");
      let formulas = match previous_forks.get(k) {
        Some(fork) => element_children(fork, "Math"),
        None if k == 0 => element_children(&previous, "Math"),
        None => Vec::new(),
      };
      for formula in formulas {
        open = integral_balance(&formula, open);
      }
    }
  }
  open > 0
}

/// The child elements of `node` named `name`.
fn element_children(node: &Node, name: &str) -> Vec<Node> {
  node
    .get_child_elements()
    .into_iter()
    .filter(|child| child.get_name() == name)
    .collect()
}

/// The previous sibling element of `node` named `name`.
fn previous_element(node: &Node, name: &str) -> Option<Node> {
  let mut node = node.get_prev_sibling();
  while let Some(sibling) = node {
    if sibling.get_name() == name {
      return Some(sibling);
    }
    node = sibling.get_prev_sibling();
  }
  None
}

/// The nearest ancestor of `node` named `name`.
fn ancestor_named(node: &Node, name: &str) -> Option<Node> {
  let mut node = node.get_parent();
  while let Some(ancestor) = node {
    if ancestor.get_name() == name {
      return Some(ancestor);
    }
    node = ancestor.get_parent();
  }
  None
}

/// The column pair of `column` among a row's cells: an alignment's `lhs & rhs` pairs, (0,1), (2,3), … — an eqnarray's
/// third column alone.
fn column_pair(tds: &[Node], column: usize) -> Vec<Node> {
  let start = column - column % 2;
  tds[start..(start + 2).min(tds.len())].to_vec()
}

/// The formulas of a column pair's cells.
fn pair_formulas(pair: &[Node]) -> Vec<Node> {
  pair
    .iter()
    .flat_map(|td| element_children(td, "Math"))
    .collect()
}

/// The cell row before `row`: its previous `tr`, or the last cell row of the previous equation's MathBranch (a `tr`, or the
/// branch itself).
fn previous_cell_row(row: &Node) -> Option<Node> {
  if row.get_name() == "tr"
    && let Some(previous) = previous_element(row, "tr")
  {
    return Some(previous);
  }
  let equation = ancestor_named(row, "equation")?;
  let previous = previous_element(&equation, "equation")?;
  // (the same fork's branch when the row is a fork's: the k-th MathFork's)
  let fork = ancestor_named(row, "MathFork");
  let forks = element_children(&equation, "MathFork");
  let k = fork
    .and_then(|fork| forks.iter().position(|candidate| *candidate == fork))
    .unwrap_or(0);
  let previous_fork = element_children(&previous, "MathFork").into_iter().nth(k)?;
  let branch = element_children(&previous_fork, "MathBranch")
    .into_iter()
    .next()?;
  let rows = element_children(&branch, "tr");
  Some(rows.last().cloned().unwrap_or(branch))
}

/// Does a column pair continue the row before it — its first cell empty, or its first formula opening with a sign or a
/// MulOp (`&\quad\times b(t)\Bigr]\mathrm dt`)?
fn is_a_continuation(pair: &[Node]) -> bool {
  match pair.first() {
    Some(first) if first.get_child_elements().is_empty() => true,
    _ => pair_formulas(pair)
      .first()
      .is_some_and(starts_with_a_joining_operator),
  }
}

/// Does `formula` open with a sign or a MulOp?
fn starts_with_a_joining_operator(formula: &Node) -> bool {
  let mut tokens = Vec::new();
  collect_tokens(formula, &mut tokens);
  tokens.first().is_some_and(|token| {
    matches!(
      token.get_attribute("role").as_deref(),
      Some("ADDOP" | "MULOP" | "BINOP")
    )
  })
}

/// `open` after the integrals and differentials of `formula` (`continues_an_open_integral`).
fn integral_balance(formula: &Node, mut open: usize) -> usize {
  let parsed = !formula
    .get_attribute("class")
    .is_some_and(|class| class.contains("ltx_math_unparsed"));
  let mut tokens = Vec::new();
  collect_tokens(formula, &mut tokens);
  for (k, token) in tokens.iter().enumerate() {
    let role = token.get_attribute("role");
    if role.as_deref() == Some("INTOP") {
      open += 1;
    } else if token.get_content() == "d"
      && (token.get_attribute("meaning").as_deref() == Some("differential-d")
        || role.as_deref() == Some("DIFFOP")
        || !parsed && tokens.get(k + 1).is_some_and(is_a_letter))
    {
      open = open.saturating_sub(1);
    }
  }
  open
}

/// The XMToks under `node`, in document order.
fn collect_tokens(node: &Node, tokens: &mut Vec<Node>) {
  for child in node.get_child_elements() {
    if child.get_name() == "XMTok" {
      tokens.push(child);
    } else {
      collect_tokens(&child, tokens);
    }
  }
}

/// Is the group opening at `open` an integral operator — at its own level only integral signs, each an INTOP or a big
/// operator before one (`\sum_{i=1}^6\int_{a_i}^{b_i}`), with their scripts, joined by signs, no operand (user ruling
/// 2026-10-01: a closed group holding only integrals counts as an integral; `\left[\int_G+\sum_{i=1}^6\int_{a_i}^{b_i}\right]
/// f(\theta)\,\mathrm d\theta`, 2605.15451; `\left(\int_{-\infty}^{-\varepsilon}+\int_\varepsilon^\infty\right)f(z)\,\mathrm dz`,
/// 2605.02925)? Its CLOSE's index when it is.
fn integral_operator_group(nodes: &[Node], levels: &OperandLevels, open: usize) -> Option<usize> {
  // (a parenthesis, bracket or brace group: the grammar's `integral_operator_group` closes no other)
  if !matches!(token_text(&nodes[open]).as_str(), "(" | "[" | "{") {
    return None;
  }
  let level = levels.levels[open];
  let close = (open + 1..nodes.len()).find(|&k| levels.levels[k] <= level)?;
  if levels.roles[close] != "CLOSE" || levels.levels[close] != level {
    return None;
  }
  let mut integral = false;
  let mut expects_an_operator = true;
  for k in open + 1..close {
    // (a big operator's scripts: their bracketing nodes and content — any other application is an operand, a fraction,
    // a root, an accent: `\left(\int_\Omega\frac{|f|^2}{w}\right)^{1/2}` is a group, the merge review's finding)
    if levels.levels[k] > level + 1 || is_a_script_marker(&nodes[k]) {
      continue;
    }
    match levels.roles[k].as_str() {
      "ADDOP" if !expects_an_operator || k == open + 1 => expects_an_operator = true,
      "INTOP" => {
        integral = true;
        expects_an_operator = false;
      },
      "SUMOP" | "BIGOP" if expects_an_operator => {},
      _ => return None,
    }
  }
  (integral && !expects_an_operator).then_some(close)
}

/// A colon, `:` or `\colon` (METARELOP; `in_an_integral_operand`).
fn is_a_colon(node: &Node) -> bool {
  let token = resolve_xmref(node).unwrap_or_else(|| node.clone());
  matches!(token.get_content().as_str(), ":" | "\u{2236}")
}

/// Does the big operator at `at` bind the letter `d` in its subscript (`in_an_integral_operand`): `\sum_d`, `\sum_{d=1}`,
/// `\prod_{d\in D}`, `\lim_{d\to\infty}`, `\sum_{1\le d\le D}`, `\sum_{d\mid n}`, `\sum_{d,e}` — the letter, an item of a list,
/// or an operand of a relation, a relation chain or a condition other than its last (`\sum_{i\le d}` binds `i`)? Its scripts
/// follow it, each a node whose argument is parsed already or, unparsed, its tokens (then: the letter before a relation).
/// Read through XMRefs, as a split row's content branch holds them.
fn binds_the_letter_d(nodes: &[Node], at: usize) -> bool {
  let realized = |node: &Node| resolve_xmref(node).unwrap_or_else(|| node.clone());
  let is_d = |node: &Node| {
    let token = realized(node);
    token.get_name() == "XMTok" && token.get_content() == "d"
  };
  let relates = |node: &Node| {
    matches!(
      get_grammatical_role(node).as_str(),
      "RELOP" | "ARROW" | "MODIFIEROP"
    )
  };
  let binds = |script: &Node| {
    let Some(argument) = script.get_first_element_child() else {
      return false;
    };
    let mut argument = realized(&argument);
    if argument.get_name() == "XMArg" {
      let items = argument.get_child_elements();
      match items.as_slice() {
        [only] => argument = only.clone(),
        [first, second, ..] => return is_d(first) && relates(second),
        [] => return false,
      }
      argument = realized(&argument);
    }
    if argument.get_name() == "XMDual" {
      match argument.get_first_element_child() {
        Some(content) => argument = realized(&content),
        None => return false,
      }
    }
    if is_d(&argument) {
      return true;
    }
    if argument.get_name() != "XMApp" {
      return false;
    }
    let children = argument.get_child_elements();
    let Some((operator, operands)) = children.split_first() else {
      return false;
    };
    let operator = realized(operator);
    match operator.get_attribute("meaning").as_deref() {
      Some("list") => operands.iter().any(is_d),
      _ if operator.get_attribute("meaning").as_deref() == Some("multirelation")
        || relates(&operator) =>
      {
        operands
          .split_last()
          .is_some_and(|(_, leading)| leading.iter().any(is_d))
      },
      _ => false,
    }
  };
  let mut next = at + 1;
  // (its two scripts, a subscript and a superscript in either order)
  for _ in 0..2 {
    let Some(marker) = nodes.get(next) else {
      return false;
    };
    let script = realized(marker);
    let role = script.get_attribute("role");
    if script.get_name() != "XMApp"
      || !matches!(role.as_deref(), Some("POSTSUBSCRIPT" | "POSTSUPERSCRIPT"))
    {
      return false;
    }
    if role.as_deref() == Some("POSTSUBSCRIPT") && binds(&script) {
      return true;
    }
    // past the script: its end in a lexer stream, or itself among a formula's children or as an XMRef
    next = nodes[next + 1..]
      .iter()
      .position(|node| node == marker)
      .map_or(next + 1, |end| next + 1 + end + 1);
  }
  false
}

/// Each node's realized role and nesting level, for `in_an_integral_operand`.
pub(crate) struct OperandLevels {
  roles:  Vec<String>,
  levels: Vec<usize>,
}

/// The nesting level of each node (`in_an_integral_operand`): an OPEN, a left bar (`\left|`, `role_side`) or, in the
/// lexer's `stream`, a bracketing node's first occurrence (a script's or another role-carrying application's start, which
/// the lexer recurses into: it occurs again as the end) opens a level its content is on; a CLOSE, a right bar or the
/// bracketing node's second occurrence closes it and is on the outer level — a bracketing node closes what is still open
/// inside it. A stray CLOSE (an unbalanced alignment cell's `\right)`) closes nothing. A formula's children (`stream`
/// false) hold each script as one node.
pub(crate) fn operand_levels(nodes: &[Node], stream: bool) -> OperandLevels {
  let roles: Vec<String> = nodes.iter().map(get_grammatical_role).collect();
  let mut levels = Vec::with_capacity(nodes.len());
  let mut open: Vec<Option<&Node>> = Vec::new();
  for (node, role) in nodes.iter().zip(&roles) {
    let side = || {
      (role == "VERTBAR")
        .then(|| {
          resolve_xmref(node)
            .unwrap_or_else(|| node.clone())
            .get_attribute("role_side")
        })
        .flatten()
    };
    if let Some(at) = open
      .iter()
      .rposition(|entry| entry.is_some_and(|entry| entry == node))
    {
      open.truncate(at);
      levels.push(open.len());
    } else if role == "OPEN"
      || side().as_deref() == Some("left")
      || (stream
        && node.get_name() == "XMApp"
        && !node.has_attribute("_rewrite")
        && node
          .get_attribute("role")
          .is_some_and(|role| !matches!(role.as_str(), "ARROW" | "METARELOP" | "RELOP")))
    {
      levels.push(open.len());
      open.push((node.get_name() == "XMApp").then_some(node));
    } else if (role == "CLOSE" || side().as_deref() == Some("right"))
      && open.last().is_some_and(Option::is_none)
    {
      open.pop();
      levels.push(open.len());
    } else {
      levels.push(open.len());
    }
  }
  OperandLevels { roles, levels }
}

fn node_to_grammar_lexemes_ctx(
  mathnode: &Node,
  child_nodes: Vec<Node>,
  idx: &mut usize,
  bigop_context: bool,
) -> (Vec<String>, Vec<Node>) {
  let mut lexemes = Vec::new();
  let mut nodes = Vec::new();
  let top_role_opt = mathnode.get_attribute("role");
  if let Some(ref top_role) = top_role_opt {
    *idx += 1;
    // When in bigop context, emit BIGOPSUB/BIGOPSUP instead of POSTSUBSCRIPT/POSTSUPERSCRIPT
    let mapped_role = if bigop_context {
      match top_role.as_str() {
        "POSTSUBSCRIPT" => "BIGOPSUB".to_string(),
        "POSTSUPERSCRIPT" => "BIGOPSUP".to_string(),
        _ => top_role.clone(),
      }
    } else {
      top_role.clone()
    };
    lexemes.push(format!("start_{mapped_role}:start:{idx}"));
    nodes.push(mathnode.clone());
  }
  // Track whether the last emitted token was a bigop (SUMOP/INTOP/etc.)
  // so we can emit bigop-specific script tokens to reduce earley chart ambiguity.
  let mut last_was_bigop = false;
  for node in child_nodes.into_iter() {
    // For XMRef nodes: resolve to get the target's role/meaning for lexing,
    // but keep the ORIGINAL XMRef node in the output so the parse tree
    // preserves XMRef indirection (matching Perl behavior).
    // The get_grammatical_role/get_token_meaning functions already resolve XMRef
    // internally, so we just need to handle the case where the target is a
    // compound node (XMApp without role) — these need flattening.
    // Note: we do NOT replace the node variable — we keep the XMRef.

    if node.get_name() == "XMApp" && node.get_attribute("role").is_some() {
      let role = node.get_attribute("role").unwrap();
      // A complete operator that has been decorated or scripted is still a
      // single grammatical operator and must lex as ONE atomic terminal:
      //   - ARROW/METARELOP: decorated arrows (\xrightarrow{over},
      //     \xleftrightarrow[under]{over}).
      //   - RELOP: a scripted relation (\stackrel{?}{=}, \overset/\underset over
      //     a relation) — `=^?` is an `<XMApp role=RELOP>` whose recursion would
      //     emit `start_RELOP SUPERSCRIPTOP RELOP:= UNKNOWN:? end_RELOP`, a
      //     sequence no grammar rule consumes, so the relation failed to parse as
      //     a standalone item (e.g. `a \quad \stackrel{?}{=} \quad b` →
      //     ltx_math_unparsed). Perl recurses with start_/end_ markers its Marpa
      //     grammar handles; the Rust grammar treats decorated operators as
      //     atomic terminals instead (the whole node is preserved as the
      //     terminal's value, so rendering still shows the script). The lexeme
      //     keys on the inner base operator's meaning, so `=^?` lexes like `=`.
      if role == "ARROW" || role == "METARELOP" || role == "RELOP" {
        let op_meaning = node
          .get_child_elements()
          .into_iter()
          .find(|ch| {
            let cr = ch.get_attribute("role");
            match role.as_str() {
              // A scripted relation's base is a RELOP leaf.
              "RELOP" => cr.as_deref() == Some("RELOP"),
              // Decorated arrows may carry either an ARROW or METARELOP base.
              _ => cr.as_deref() == Some("ARROW") || cr.as_deref() == Some("METARELOP"),
            }
          })
          .and_then(|ch| {
            ch.get_attribute("meaning")
              .or_else(|| ch.get_attribute("name"))
              .or_else(|| Some(ch.get_content()))
          })
          .unwrap_or_else(|| role.to_string());
        *idx += 1;
        let lexeme = grammar_lexeme(&role, &op_meaning, *idx);
        lexemes.push(lexeme);
        nodes.push(node);
      } else if node.has_attribute("_rewrite") {
        // Rewrite-created or a parse_children replacement: treat as atomic
        // token with the assigned role. Don't recurse — the inner structure
        // was pre-parsed, and the role on this node overrides whatever the
        // children contain. An atomic bigop (e.g. `\mathop{...}` → BIGOP)
        // must set bigop context so a following script lexes as
        // BIGOPSUB/BIGOPSUP, same as the plain-token arm below.
        let gram_role = get_grammatical_role(&node);
        let mut text = get_token_meaning(&node);
        if text.is_empty() {
          text = "UNKNOWN".to_string();
        }
        *idx += 1;
        last_was_bigop = matches!(
          gram_role.as_str(),
          "SUMOP" | "INTOP" | "LIMITOP" | "DIFFOP" | "BIGOP"
        );
        lexemes.push(grammar_lexeme(&gram_role, &text, *idx));
        nodes.push(node);
      } else {
        // Only recurse into XMApp nodes that have a role (scripts, etc.)
        // Role-less XMApps (e.g. \sqrt, already-parsed structures) are atomic.
        // Pass bigop_context for POSTSUBSCRIPT/POSTSUPERSCRIPT following a bigop
        let is_script = matches!(role.as_str(), "POSTSUBSCRIPT" | "POSTSUPERSCRIPT");
        let ctx = last_was_bigop && is_script;
        let children = filter_hints(node.get_child_nodes());
        let (mut inner_lexes, mut inner_nodes) =
          node_to_grammar_lexemes_ctx(&node, children, idx, ctx);
        for (inner_lex, inner_node) in inner_lexes.drain(..).zip(inner_nodes.drain(..)) {
          lexemes.push(inner_lex);
          nodes.push(inner_node);
        }
        // Script following a bigop is still "bigop context" for the next script
        if !is_script {
          last_was_bigop = false;
        }
      }
    } else if node.get_name() == "XMArg" {
      // XMArg is a transparent wrapper (e.g. from \lx@post@subscript).
      // Recurse into its children so the grammar can parse them individually.
      // E.g. _{ij} should emit UNKNOWN:i + UNKNOWN:j, not ATOM:ij.
      let arg_children = filter_hints(node.get_child_nodes());
      let (mut inner_lexes, mut inner_nodes) =
        node_to_grammar_lexemes_ctx(&node, arg_children, idx, false);
      for (inner_lex, inner_node) in inner_lexes.drain(..).zip(inner_nodes.drain(..)) {
        lexemes.push(inner_lex);
        nodes.push(inner_node);
      }
    } else {
      let mut role = get_grammatical_role(&node);
      // An accented differential operator is one (57cj review): `\partial\bar\partial\phi` is ∂(∂̄φ)
      // (2605.01526, 2605.15276, 2605.01646), where Perl reads the accent's application an ATOM that
      // `\partial` multiplies. It keeps its text (`DIFFOP:¯∂`), so it is no plain `\partial`.
      if role == "ATOM" && is_accented_differential_operator(&node) {
        role = "DIFFOP".to_string();
      }
      // A fraction of numbers (`\frac12`, `\tfrac{3}{4}`) is an ATOM that is a number: its own category,
      // `ATOM_NUMBER`, which the grammar's `numeric_monomial` leads (57cj.1 review, latent: its probes; the
      // actions read it as an ATOM, `semantics::lexeme_category`). Perl reads it an ATOM. So is any constant atom
      // (`is_numeric_constant`: `\sqrt2`, `\frac{\pi}{2}`, `\sqrt[3]{2}`, `\frac{\pi^2}{6}`; the actions' notion,
      // `semantics::is_constant`; 57cj.7, 57cj.8 reviews, latent: their probes).
      if role == "ATOM" && is_numeric_constant(&node) {
        role = "ATOM_NUMBER".to_string();
      }
      let mut text = get_token_meaning(&node);
      if text.is_empty() {
        text = "UNKNOWN".to_string();
      }
      // Perl's grammar terminal for a bar is content-agnostic (`/VERTBAR:\S*:\d+/`,
      // MathGrammar:797) and its fences key on the bar's content (`'|@|'`, MathParser.pm:1361);
      // Rust's rules key the bar lexeme, so a bar with a name but no meaning (`\bigl\lvert`,
      // `\arrowvert`, `\bigl\lVert`) reads by its glyph: `|` single, `∥`/`‖` double. The
      // bar is read through an XMRef (a gathered/split row's content branch), as the role is
      // (arXiv 2605.02713 `\arrowvert\psi_n\arrowvert^2`, 2605.19292 `\bigl\lvert
      // a_{ij}\bigr\rvert`). Golden
      // tests/parse/bar_pairs.tex#named_vertbar_parses_as_absolute_value.
      if role == "VERTBAR" {
        let bar = resolve_xmref(&node).unwrap_or_else(|| node.clone());
        if bar.get_attribute("meaning").is_none() {
          match bar.get_content().as_str() {
            "|" => text = "|".to_string(),
            "\u{2225}" | "\u{2016}" => text = "||".to_string(),
            _ => {},
          }
        }
      }
      *idx += 1;
      // Track bigop tokens for bigop-specific script token emission
      let is_bigop = matches!(
        role.as_str(),
        "SUMOP" | "INTOP" | "LIMITOP" | "DIFFOP" | "BIGOP"
      );
      last_was_bigop = is_bigop;
      // Normalize langle/rangle meaning for consistent grammar matching.
      // Do NOT remap to parentheses — langle_open/rangle_close tokens need
      // distinct identity for QM bra-ket rules vs conditional probability.
      //
      // Perf: Split generic OPEN/CLOSE into OTHER_OPEN/OTHER_CLOSE for delimiters
      // OTHER than parens/brackets/braces/langle. This prevents the grammar's
      // generic `open ~ "OPEN"` prefix match from ALSO matching the specific
      // `lparen = "OPEN:("`, etc. — which was producing 2× duplicate grammar
      // derivations for every `(x)`, `[x]`, `{x}` expression.
      let lexeme = if role == "OPEN" && (text == "⟨" || text == "langle") {
        format!("OPEN:langle:{idx}")
      } else if role == "CLOSE" && (text == "⟩" || text == "rangle") {
        format!("CLOSE:rangle:{idx}")
      } else if matches!(role.as_str(), "OPEN" | "CLOSE")
        && text == "||"
        && resolve_xmref(&node)
          .unwrap_or_else(|| node.clone())
          .get_attribute("stretchy")
          .as_deref()
          == Some("true")
      {
        // A `\left`/`\right` double bar — `\left\|`, `\left\Vert`, `\left\lVert` — is a bar
        // pair in Perl: `augmentDelimiterProperties` re-roles its ∥ a VERTBAR on either side
        // (DELIMITER_MAP, TeX_Math.pool.ltxml:755, :813-816), so `VERTBAR absExpression
        // VERTBAR` reads it as an `aBarearg` (MathGrammar:329-330) — `\exp\left\|x\right\|^2` is
        // exp@(‖x‖²), not a group `\exp` applies to. DELIMITER_MAP here keeps its OPEN/CLOSE
        // role (`name="||"`) for the markup; its grammar view is the stretchy bar's, sided by
        // the role. Read through an XMRef, as a split row's content branch holds it: unlike a
        // single `|` (below) no divider or bra/ket rule reads a double bar, and on the XMRef itself
        // a split row's `\left\|` would lex `OTHER_OPEN:||`, a group `\exp` applies to (57am
        // review round 10). Golden tests/parse/bar_pairs.tex#left_double_bar_is_a_bar_pair.
        if role == "OPEN" {
          format!("LEFT_STRETCHY_VERTBAR:||:{idx}")
        } else {
          format!("RIGHT_STRETCHY_VERTBAR:||:{idx}")
        }
      } else if role == "OPEN" && !matches!(text.as_str(), "(" | "[" | "{") {
        grammar_lexeme("OTHER_OPEN", &text, *idx)
      } else if role == "CLOSE" && !matches!(text.as_str(), ")" | "]" | "}") {
        grammar_lexeme("OTHER_CLOSE", &text, *idx)
      } else if role == "UNKNOWN" && text == "d" {
        // M4: Emit XDIFFUNK for possible differential-d tokens.
        // Only "d" tokens can be diffops; other unknowns skip the diffop rule.
        format!("XDIFFUNK:{text}:{idx}")
      } else if role == "ID" && text == "d" {
        format!("XDIFFID:{text}:{idx}")
      } else if role == "VERTBAR"
        && text == "|"
        && node.get_attribute("stretchy").as_deref() == Some("true")
      {
        // `\left|...\right|` produces a balanced pair of VERTBAR tokens
        // with `stretchy="true"` (whereas bare `|x|` is `stretchy="false"`).
        // Further distinguish by side: the `\lx@delim@left` constructor tags
        // the emitted XMTok with `role_side="left"`, `\lx@delim@right` with
        // `role_side="right"` (tex_math.rs). This refines the `role`
        // attribute (delimiter direction) without changing it, so
        // the grammar can use distinct LEFT_STRETCHY_VERTBAR /
        // RIGHT_STRETCHY_VERTBAR tokens. The kerned-stack norm rules
        // (\vertii, \vertiii, …) then don't have to enumerate every
        // pairing of identical bars. Eliminates the VERTBAR-pairing
        // combinatorial explosion in patterns like
        // `\log^+ ∫ \left| f \right|^k dm(z) ≲ ...` (see
        // docs/archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md §2) and unlocks task #263's
        // norm-fenced grammar rules.
        // Defensive fallback: if `role_side` is missing — legacy DOM
        // input or a path that bypassed `\lx@delim@left`/`\lx@delim@right` — keep the
        // old undirected STRETCHY_VERTBAR lexeme so legacy rules
        // (eval_at, modulus fence) still work.
        // Read on the node itself: a split row's content branch holds XMRefs, which lex as the
        // unsided `VERTBAR:|`. The divider and bra/ket rules take the sided bar too since 57ap
        // (`divider_bar`, `bra_bar`, `ket_bar`), so a split row may read its bars sided next — SYNC
        // "A split row reads its bars unsided"; 57am review round 8: sided before 57ap, the split rows
        // of 2605.11264 and 2605.20326 lost their parse.
        match node.get_attribute("role_side").as_deref() {
          Some("left") => format!("LEFT_STRETCHY_VERTBAR:|:{idx}"),
          Some("right") => format!("RIGHT_STRETCHY_VERTBAR:|:{idx}"),
          _ => format!("STRETCHY_VERTBAR:|:{idx}"),
        }
      } else if role == "VERTBAR" && (text == "mid" || text == "\u{2223}") {
        // Canonicalize the single "divides / such-that / given" bar to one
        // grammar token. `\mid` (∣ U+2223), bare `|` (U+007C) and `\vert`
        // (`\Let` to `|`) are all the SAME bar grammatically, but they reach
        // the lexer with different content: `|`/`\vert` carry "|" while `\mid`
        // carries either the glyph ∣ or the CS name "mid" — the latter because
        // `def_math` infers a name from the CS (`binding/def/dialect.rs`)
        // whereas Perl's `DefMathI('\mid', undef, "\x{2223}", role=>'VERTBAR')`
        // (math_common.pool.ltxml:290) leaves the name undef. Either way the
        // lexeme came out as `VERTBAR:mid` / `VERTBAR:∣`, NOT `VERTBAR:|`.
        // Every divides-bar grammar rule (modulus `|x|`, conditional `(a|b)`,
        // set-builder `{x|P}`, Dirac bra/ket/braket) keys on the exact
        // `singlevertbar = VERTBAR:|` terminal, so `\mid` inside any fence
        // failed to parse (→ ltx_math_unparsed) even though `|` did — breaking
        // ubiquitous notation: conditional probability `P(A\mid B)`,
        // set-builder `\{x\mid x>0\}`, Dirac `\langle a\mid b\rangle`.
        // Perl's grammar terminal is content-agnostic (`/VERTBAR:\S*:\d+/`,
        // MathGrammar:797), so Perl parses all of these; Rust's is
        // glyph-specific, so we canonicalize here instead. The rendered glyph
        // is unaffected — the XMTok node keeps its ∣ content; the lexeme is a
        // parallel grammar-only view looked up by idx. (Doubled bars
        // `\|`/`\Vert` → `VERTBAR:||` and `\parallel` → `VERTBAR:parallel-to`
        // are NOT single divides bars; they keep their content for the
        // norm / parallel rules.)
        format!("VERTBAR:|:{idx}")
      } else if role == "PUNCT" && punct_followed_by_wide_space(&node) {
        // PUNCT followed by `\quad` / `\qquad` etc. carries an `rpadding`
        // attribute. This wide spacing is a strong arXiv idiom for
        // "formula separator (with side-condition)" rather than
        // "list-element separator". Tag as WIDE_PUNCT so the grammar
        // can prefer `formulae_apply` over `list_apply` for this
        // separator unambiguously. See docs/archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md §2.
        format!("WIDE_PUNCT:,:{idx}")
      } else {
        grammar_lexeme(&role, &text, *idx)
      };
      lexemes.push(lexeme);
      nodes.push(node);
    }
  }
  if let Some(ref top_role) = top_role_opt {
    *idx += 1;
    let mapped_end = if bigop_context {
      match top_role.as_str() {
        "POSTSUBSCRIPT" => "BIGOPSUB".to_string(),
        "POSTSUPERSCRIPT" => "BIGOPSUP".to_string(),
        _ => top_role.clone(),
      }
    } else {
      top_role.clone()
    };
    lexemes.push(format!("end_{mapped_end}:end:{idx}"));
    nodes.push(mathnode.clone());
  }
  (lexemes, nodes)
}

/// The lexeme the lexer gives a letter token (its role, `get_token_meaning`, the index): the plain branch
/// of `node_to_grammar_lexemes_ctx` for an UNKNOWN token — what an expectation's or probability's typed
/// lexeme was before `parser::type_expectation_lexemes` (`semantics::expectation_as_letter`, M3).
pub(crate) fn letter_lexeme(node: &Node, idx: usize) -> String {
  let mut text = get_token_meaning(node);
  if text.is_empty() {
    text = "UNKNOWN".to_string();
  }
  grammar_lexeme(&get_grammatical_role(node), &text, idx)
}

/// Is `node` a constant atom: an application of constants — numbers, π — as a fraction, root, nth root, sum,
/// product or power of them, one at least (`\frac12`, `\tfrac{3}{4}`, `\sqrt2`, `\sqrt[3]{2}`, `\frac{\pi}{2}`,
/// `\frac{\pi^2}{6}`, `\sqrt{2\pi}`, `\frac{1+\sqrt5}{2}`, `\frac{1}{2\pi i}`)? π is one unscripted or raised to a
/// constant power (`\frac{\pi^a}{2}` is none: `\pi^a` names a field); an imaginary unit `i` right after a constant in
/// a product is one (`is_imaginary_unit_token`); a subscript makes none. The notion `semantics::is_constant` reads
/// from the lexemes (57cj.8 review; latent, its probes). A Dual is read by its presentation, which holds its tokens,
/// and a reference only through the math idstore (`resolve_xmref_in_store`): one pointing outside the formula walked
/// the document (`\pi(t)` in a script, 2605.06431: a reference per atom doubled the paper's math parsing time).
pub(crate) fn is_numeric_constant(node: &Node) -> bool {
  let node = crate::data::resolve_xmref_in_store(node).unwrap_or_else(|| node.clone());
  node.get_name() == "XMApp" && is_constant_node(&node)
}

/// A constant: a number or π token, an application of a fraction, root, sum, product or constant power to constants,
/// a Dual by its presentation, an argument or wrapper holding one (`is_numeric_constant`).
pub(crate) fn is_constant_node(node: &Node) -> bool {
  match node.get_name().as_str() {
    "XMTok" => {
      node.get_attribute("role").as_deref() == Some("NUMBER")
        || node.get_attribute("name").as_deref() == Some("pi")
    },
    "XMRef" => {
      crate::data::resolve_xmref_in_store(node).is_some_and(|target| is_constant_node(&target))
    },
    "XMDual" => node
      .get_last_element_child()
      .is_some_and(|presentation| is_constant_node(&presentation)),
    "XMArg" | "XMWrap" => is_constant_run(&node.get_child_elements()),
    "XMApp" => {
      let children = node.get_child_elements();
      let Some((op, args)) = children.split_first() else {
        return false;
      };
      if op.get_name() != "XMTok" || args.is_empty() {
        return false;
      }
      let role = op.get_attribute("role");
      let meaning = op.get_attribute("meaning");
      match (role.as_deref(), meaning.as_deref()) {
        // a constant power: `\pi^2`, `\sqrt2^3`, `2^{-1}`
        (Some("SUPERSCRIPTOP"), _) => args.iter().all(is_constant_node),
        (Some("MULOP"), _) => is_constant_run(args),
        (Some("FRACOP" | "ADDOP"), _) | (_, Some("divide" | "square-root" | "nth-root")) => {
          args.iter().all(is_constant_node)
        },
        _ => false,
      }
    },
    _ => false,
  }
}

/// Are `nodes`, factors of a product, constants all — a spacing hint and a group's delimiters aside (`(2\pi)^3`, as
/// `semantics::is_constant` reads a group; 57cj.9 review), an imaginary unit right after a constant counting as one
/// (`\frac{1}{2\pi i}`)?
fn is_constant_run(nodes: &[Node]) -> bool {
  let mut constants = 0;
  nodes.iter().all(|node| {
    if node.get_name() == "XMHint"
      || node.get_name() == "XMTok"
        && matches!(
          node.get_attribute("role").as_deref(),
          Some("OPEN" | "CLOSE")
        )
    {
      return true;
    }
    let constant = is_constant_node(node) || constants > 0 && is_imaginary_unit_token(node);
    constants += usize::from(constant);
    constant
  }) && constants > 0
}

/// The imaginary unit: `i`, upright or italic, `\imath`, or a token meaning it — a constant right after another
/// (`is_constant_run`, `semantics::constant_run`).
pub(crate) fn is_imaginary_unit_token(node: &Node) -> bool {
  node.get_name() == "XMTok"
    && (node.get_attribute("meaning").as_deref() == Some("imaginary-unit")
      || node.get_attribute("name").as_deref() == Some("imath")
      || node.get_attribute("role").as_deref() == Some("UNKNOWN") && node.get_content() == "i")
}

/// An accent over a differential operator (`\bar\partial`, `\overline\partial`): an application of an
/// OVERACCENT or UNDERACCENT to a DIFFOP token, read through an XMRef.
fn is_accented_differential_operator(node: &Node) -> bool {
  let node = resolve_xmref(node).unwrap_or_else(|| node.clone());
  let role = |child: &Node| {
    let child = resolve_xmref(child).unwrap_or_else(|| child.clone());
    (child.get_name() == "XMTok")
      .then(|| child.get_attribute("role"))
      .flatten()
  };
  node.get_name() == "XMApp"
    && matches!(node.get_child_elements().as_slice(), [accent, base]
      if matches!(role(accent).as_deref(), Some("OVERACCENT" | "UNDERACCENT"))
        && role(base).as_deref() == Some("DIFFOP"))
}

/// Auxiliary separator for ROLE:style-lexeme into ("ROLE:style", '-', lexeme)
pub fn distill_lexeme(name: &str) -> (&str, &str, &str) {
  // dash separates styles, colons separate grammatical roles, and we are
  // only trying to distill the last pure lexeme
  // note that we are only trying to do this reasonably for letter-based names (UNKNOWN:italic-x),
  // since some of the content symbols contain dashes themselves (e.g.
  // OPERATOR:partial-differential)
  if let Some(position) = name.rfind('-') {
    let (base, trailer) = name.split_at(position);
    let (sep, lexeme) = trailer.split_at(1);
    (base, sep, lexeme)
  } else if let Some(position) = name.rfind(':') {
    let (base, trailer) = name.split_at(position);
    let (sep, lexeme) = trailer.split_at(1);
    (base, sep, lexeme)
  } else {
    ("", "", name)
  }
}

/// Parse an XMHint `width` attribute string to points.
/// `pub(crate)` so semantics.rs can re-use the same parser when
/// inspecting `rpadding` (task #263 stretchy_norm_fenced guards).
/// Supports "3.0mu" (mu → pt by dividing by 1.8) and "1.667pt"/"0.16667em".
/// Handles glue specs like "2.77pt plus 2.77pt" by extracting base dimension.
pub(crate) fn get_xmhint_spacing(width: &str) -> f64 {
  let width = width.trim();
  if width.is_empty() {
    return 0.0;
  }
  // Strip glue stretch/shrink: "2.77pt plus 2.77pt" → "2.77pt"
  let base = width
    .split_once(" plus")
    .or_else(|| width.split_once(" minus"))
    .map_or(width, |(base, _)| base)
    .trim();
  let unit_start = base.find(|c: char| c.is_alphabetic()).unwrap_or(base.len());
  let (number_str, unit) = base.split_at(unit_start);
  let number: f64 = number_str.trim().parse().unwrap_or(0.0);
  match unit.trim() {
    "mu" => number / 1.8,
    "pt" => number,
    "em" => number * 10.0, // assume 10pt font size
    _ => 0.0,
  }
}

/// True iff the given PUNCT-like XMTok carries an `rpadding` attribute
/// large enough to indicate a `\quad`-class spacing — TeX's idiom for
/// separating a main formula from a side-condition (e.g.
/// `A = B, \quad r \notin E`). Threshold of ≥10pt catches `\quad` (10pt)
/// and `\qquad` (20pt) while EXCLUDING the interword control space `\ `
/// (exactly 5pt) and thin-space touch-ups (`\,`/`\;`/`\:`). This matches
/// `filter_hints`'s own `HINT_PUNCT_THRESHOLD` (10pt). An earlier ≥5pt
/// threshold mis-tagged `\ ` as a separator, so a comma-list element with
/// an interword space before a signed/`±` term — e.g. `(3,\ -5)` or
/// `\textit{Held\_For}\;(300,\ -50,\ +50)` (witness 1506.03557) — became a
/// fenced WIDE_PUNCT routed through `formulae_apply`, which doesn't apply
/// inside a fence, so the whole formula fell to `ltx_math_unparsed`.
/// Used by `node_to_grammar_lexemes_ctx` to emit a `WIDE_PUNCT` token
/// the grammar can route through `formulae_apply` unambiguously.
fn punct_followed_by_wide_space(node: &Node) -> bool {
  match node.get_attribute("rpadding") {
    Some(s) => get_xmhint_spacing(&s) >= 10.0,
    None => false,
  }
}

/// Filter XMHint nodes from a list of child nodes, transferring their spacing
/// info to adjacent tokens as `lpadding`/`rpadding` attributes (Perl MathParser.pm
/// `filter_hints` L417-491). XMHints are also unlinked from the XML tree so they won't be
/// seen again. Large spacings (≥10pt, e.g. \quad) become virtual PUNCT nodes (L476-487),
/// unless a phantom (`name` ending in `phantom`, L443) contributed to them (arXiv 2605.24688).
/// An APPLYOP token is kept but takes no space (L458-461): "they tend to disappear". A space
/// pending at the next node is its `lpadding`, negative (`\!`) too (L466). The OPEN and PUNCT
/// tests read the realized role, where Perl reads the node's own (L445, L482-483): in a
/// gathered/split row's content branch the nodes are XMRefs, and Perl's `,\quad` there became
/// two punctuations, leaving the formula unparsed (KNOWN_PERL_ERRORS #365, OXIDIZED_DESIGN
/// #349; golden tests/parse/aligned_content_branch.tex#content_branch_reads_its_delimiters).
pub fn filter_hints(nodes: Vec<Node>) -> Vec<Node> {
  const HINT_PUNCT_THRESHOLD: f64 = 10.0;
  let mut prefiltered: Vec<Node> = Vec::new();
  // Perl's `$prev`: the last kept node that is not an APPLYOP.
  let mut prev: Option<usize> = None;
  let mut pending_space: f64 = 0.0;
  let mut pending_phantom = false;
  // Save hint nodes that contributed to _space, for possible PUNCT reuse
  let mut last_hint_for: Vec<Option<Node>> = Vec::new(); // parallel to prefiltered
  // Perl's `_phantom`, parallel to prefiltered
  let mut phantom: Vec<bool> = Vec::new();

  for mut node in nodes {
    if node.get_type() != Some(NodeType::ElementNode) {
      continue;
    }
    if node.get_name() == "XMHint" {
      if let Some(width_str) = node.get_attribute("width") {
        let pts = get_xmhint_spacing(&width_str);
        if pts != 0.0 {
          let ph = node
            .get_attribute("name")
            .is_some_and(|name| name.ends_with("phantom"));
          match prev {
            Some(idx) if get_grammatical_role(&prefiltered[idx]) != "OPEN" => {
              let prev = &mut prefiltered[idx];
              let s: f64 = prev
                .get_attribute("_space")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);
              let _ = prev.set_attribute("_space", &format!("{}", s + pts));
              phantom[idx] |= ph;
              // Save this hint node for potential PUNCT reuse
              last_hint_for[idx] = Some(node.clone());
            },
            _ => {
              pending_space += pts;
              pending_phantom = ph;
            },
          }
        }
      }
      // Unlink from the XML tree; XMHints are ephemeral
      node.unlink();
    } else if node.get_name() == "XMTok" && node.get_attribute("role").as_deref() == Some("APPLYOP")
    {
      prefiltered.push(node);
      last_hint_for.push(None);
      phantom.push(false);
    } else {
      let mut ph = false;
      if pending_space != 0.0 {
        let _ = node.set_attribute("lpadding", &format!("{:.1}pt", pending_space));
        ph = pending_phantom;
        pending_space = 0.0;
        pending_phantom = false;
      }
      prev = Some(prefiltered.len());
      prefiltered.push(node);
      last_hint_for.push(None);
      phantom.push(ph);
    }
  }

  // Second pass: convert _space to rpadding (or PUNCT XMHint if above threshold)
  let mut filtered: Vec<Node> = Vec::new();
  for (i, mut node) in prefiltered.into_iter().enumerate() {
    filtered.push(node.clone());
    if let Some(s_str) = node.get_attribute("_space") {
      let _ = node.remove_attribute("_space");
      let s: f64 = s_str.parse().unwrap_or(0.0);
      if !phantom[i] && s >= HINT_PUNCT_THRESHOLD && get_grammatical_role(&node) != "PUNCT" {
        // Perl MathParser.pm L487: create virtual PUNCT XMHint
        // Reuse the saved hint node, setting role="PUNCT"
        if let Some(Some(mut hint)) = last_hint_for.get(i).cloned() {
          let _ = hint.set_attribute("role", "PUNCT");
          // Clean width: round to integer if close, matching Perl format
          let s_rounded = if (s - s.round()).abs() < 0.01 {
            s.round()
          } else {
            s
          };
          let width = format!("{}pt", s_rounded);
          let _ = hint.set_attribute("width", &width);
          // Remove extraneous attributes from the reused hint node
          let _ = hint.remove_attribute("depth");
          let _ = hint.remove_attribute("height");
          let quads = "q".repeat((s / 10.0) as usize);
          let _ = hint.set_attribute("name", &format!("{quads}uad"));
          filtered.push(hint);
        }
      } else {
        let _ = node.set_attribute("rpadding", &format!("{:.1}pt", s));
      }
    }
  }
  filtered
}

/// Given a list of XML nodes (either libxml nodes, or array representations)
/// return a list of XMRef's referring to those nodes;
/// ensure each source node has an ID (if already instanciated as XML)
/// or _xmkey if still in array rep. since it will get an ID later, and the connection re-made)
/// Note that ltx:XMHint nodes are ephemeral and shouldn't be ref'd!
/// likewise, we avoid creating XMRefs to XMRefs
pub fn create_xmrefs(args: &mut [&mut XM], ctxt: ActionContext) -> Result<Vec<XM>, Box<dyn Error>> {
  let nodes = ctxt.nodes;
  let document = ctxt.document;
  let mut refs = Vec::with_capacity(args.len());
  for arg in args {
    match arg {
      XM::Token(props, _meta) => {
        if let Some(id) = props.id.as_ref() {
          refs.push(XM::Ref(XProps {
            id: Some(id.clone()),
            ..XProps::default()
          }));
        } else if let Some(existing) = props.xmkey.clone() {
          // Preserve an existing _xmkey (see the XM::Apply arm) — faithful to
          // Perl `createXMRefs`, which never clobbers an arg's `_xmkey`.
          refs.push(XM::Ref(XProps {
            xmkey: Some(existing),
            ..XProps::default()
          }));
        } else {
          // Parser-created token without id — use _xmkey for deferred resolution
          let key = get_xmarg_id()?.to_string();
          props.xmkey = Some(Cow::Owned(key.clone()));
          refs.push(XM::Ref(XProps {
            xmkey: Some(Cow::Owned(key)),
            ..XProps::default()
          }));
        }
      },
      XM::Lexeme(lex, _) => {
        // If arg is already XML, it's too late to get automatic ID's.
        // lookup_lex_node now returns Err for malformed lex strings instead
        // of panicking; skip the arg on failure rather than abort the whole
        // ref-building pass.
        let node = match lookup_lex_node(lex, nodes) {
          Ok(n) => n,
          Err(e) => {
            // Perl MathParser.pm:151 — Error('expected', 'id', undef,
            //   "Cannot find a node with xml:id='$idref'", ...)
            // We don't always have an idref string at this layer; the
            // lookup error itself carries enough context.
            log_math_error!(
              "expected",
              "id",
              "create_xmrefs: skipping lexeme with invalid node lookup: {}",
              e
            );
            continue;
          },
        };

        // Perl `createXMRefs` (Package.pm L1557-1561): "clone an XMRef
        // rather than create an XMRef to an XMRef" — reuse the target's
        // idref so a content branch never points at a presentation-side
        // reference. Without this branch the ref-to-a-ref shape reaches
        // the output (Perl's whole golden corpus has zero of them).
        if node.get_name() == "XMRef" {
          // Carry BOTH spellings, as Perl does — `_xmkey` is the deferred
          // form used before ids exist, `idref` the resolved one, and either
          // may be absent. Taking this branch unconditionally matters: on a
          // key-only XMRef, falling through would mint an xml:id ON the ref
          // and reference THAT, which is the ref-to-a-ref this branch exists
          // to prevent.
          refs.push(XM::Ref(XProps {
            id: node.get_attribute("idref").map(Cow::Owned),
            xmkey: node.get_attribute("_xmkey").map(Cow::Owned),
            ..XProps::default()
          }));
          continue;
        }

        // NS-aware read (xml:id = local "id" in XML_NS; the bare
        // get_attribute("xml:id") form always returns None, which sent
        // every node — id or not — through the generate_id branch).
        match node.get_attribute_ns("id", latexml_core::common::xml::XML_NS) {
          //  already has id, so refer to it.
          Some(id) => refs.push(XM::Ref(XProps {
            id: Some(Cow::Owned(id)),
            ..XProps::default()
          })),
          None => {
            // Generate xml:id for this node so we can reference it
            document.generate_id(&mut node.clone(), "")?;
            if let Some(id) = node.get_attribute_ns("id", latexml_core::common::xml::XML_NS) {
              refs.push(XM::Ref(XProps {
                id: Some(Cow::Owned(id)),
                ..XProps::default()
              }));
            }
          },
        }
      },
      XM::Apply(_op, _args, props, _meta) => {
        if let Some(id) = props.id.as_ref() {
          refs.push(XM::Ref(XProps {
            id: Some(id.clone()),
            ..XProps::default()
          }));
        } else if let Some(existing) = props.xmkey.clone() {
          // PRESERVE an existing _xmkey. Faithful to Perl `createXMRefs`
          // (Package.pm:1527): it `GenerateID`s an XML arg but NEVER clobbers
          // its `_xmkey`, so a node that is ALREADY an arg of an outer
          // package XMDual (via `\lx@xmarg`, e.g. braket `\braket{a|op|c}`)
          // keeps that key while also being referenced by THIS (grammar) dual
          // — both content XMRefs resolve to the same shared node. Overwriting
          // the key (as before) dangled the package's XMRef →
          // `expected:id Cannot find a node` (compound op; witness 2205.06843).
          refs.push(XM::Ref(XProps {
            xmkey: Some(existing),
            ..XProps::default()
          }));
        } else {
          // not yet instanciated, so hasn't had chance to get auto-id; use _xmkey
          let key = get_xmarg_id()?.to_string();
          props.xmkey = Some(Cow::Owned(key.clone()));
          refs.push(XM::Ref(XProps {
            xmkey: Some(Cow::Owned(key)),
            ..XProps::default()
          }));
        }
      },
      // clone an XMRef (w/o any attributes or id ?) rather than create an XMRef to an XMRef:
      // only its `_xmkey` and `idref` (Package.pm:1557-1561), not the role/meaning a parse
      // annotated it with.
      XM::Ref(props) => {
        refs.push(XM::Ref(XProps {
          id: props.id.clone(),
          xmkey: props.xmkey.clone(),
          ..XProps::default()
        }));
      },
      XM::Dual(_, _, props, _) | XM::Wrap(_, props, _) => {
        if let Some(id) = props.id.as_ref() {
          refs.push(XM::Ref(XProps {
            id: Some(id.clone()),
            ..XProps::default()
          }));
        } else if let Some(existing) = props.xmkey.clone() {
          // Preserve an existing _xmkey (see the XM::Apply arm) — faithful to
          // Perl `createXMRefs`, which never clobbers an arg's `_xmkey`.
          refs.push(XM::Ref(XProps {
            xmkey: Some(existing),
            ..XProps::default()
          }));
        } else {
          let key = get_xmarg_id()?.to_string();
          props.xmkey = Some(Cow::Owned(key.clone()));
          refs.push(XM::Ref(XProps {
            xmkey: Some(Cow::Owned(key)),
            ..XProps::default()
          }));
        }
      },
      _ => {
        // XMHint's are ephemeral — clone without id
        // Other variants: skip with warning
      },
    }
  }
  Ok(refs)
}

/// A grammar lexeme `ROLE:text:index`, whitespace removed as Perl does (`s/\s//g`, MathParser.pm:
/// 803-804): a text token holding a newline or an em-space no longer breaks its lexeme, and a colon
/// in the text is the grammar's to allow (`lex_char`), the index being the last `:digits` (57bm;
/// `\text{(OCP): }R(s)`, 2605.11979, 2605.03556) — a text's colon, or a parsed script's, which
/// reaches the formula as one ATOM holding its content (`\sum_{i:a_i>0}x_i`, `ATOM::i>ai0:3`). A text of whitespace only, which Perl's `\S*`
/// matches empty, is the placeholder `␣` (`BLANK_LEXEME_TEXT`) — the grammar's text needs one
/// character.
pub(crate) fn grammar_lexeme(role: &str, text: &str, idx: usize) -> String {
  let strip = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
  let text = strip(text);
  let text = if text.is_empty() {
    BLANK_LEXEME_TEXT.to_string()
  } else {
    text
  };
  format!("{}:{text}:{idx}", strip(role))
}

/// The lexeme text of a token whose text is whitespace only (`grammar_lexeme`).
pub(crate) const BLANK_LEXEME_TEXT: &str = "\u{2423}";

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn grammar_lexeme_strips_all_whitespace_and_keeps_colons() {
    assert_eq!(grammar_lexeme("ATOM", "(OCP): ", 1), "ATOM:(OCP)::1");
    assert_eq!(
      grammar_lexeme("ATOM", "max x for\nall z", 2),
      "ATOM:maxxforallz:2"
    );
    assert_eq!(
      grammar_lexeme("ATOM", "\u{2003}\u{2003}", 3),
      "ATOM:\u{2423}:3"
    );
    assert_eq!(
      grammar_lexeme("OTHER_OPEN", "\u{27E6}", 4),
      "OTHER_OPEN:\u{27E6}:4"
    );
  }

  #[test]
  fn distill_lexeme_dash_separator() {
    // Dash takes precedence over colon.
    let (base, sep, lex) = distill_lexeme("UNKNOWN:italic-x");
    assert_eq!(base, "UNKNOWN:italic");
    assert_eq!(sep, "-");
    assert_eq!(lex, "x");
  }

  #[test]
  fn distill_lexeme_colon_only() {
    let (base, sep, lex) = distill_lexeme("ROLE:foo");
    assert_eq!(base, "ROLE");
    assert_eq!(sep, ":");
    assert_eq!(lex, "foo");
  }

  #[test]
  fn distill_lexeme_no_separator() {
    let (base, sep, lex) = distill_lexeme("bare");
    assert_eq!(base, "");
    assert_eq!(sep, "");
    assert_eq!(lex, "bare");
  }

  #[test]
  fn distill_lexeme_empty() {
    let (base, sep, lex) = distill_lexeme("");
    assert_eq!(base, "");
    assert_eq!(sep, "");
    assert_eq!(lex, "");
  }

  #[test]
  fn distill_lexeme_trailing_dash() {
    // Edge: last-dash splits after the last hyphen even when the tail is empty.
    let (base, sep, lex) = distill_lexeme("foo-");
    assert_eq!(base, "foo");
    assert_eq!(sep, "-");
    assert_eq!(lex, "");
  }

  #[test]
  fn get_xmhint_spacing_mu_divides_by_1_8() {
    assert!((get_xmhint_spacing("1.8mu") - 1.0).abs() < 1e-6);
    assert!((get_xmhint_spacing("3.6mu") - 2.0).abs() < 1e-6);
  }

  #[test]
  fn get_xmhint_spacing_pt_passes_through() {
    assert!((get_xmhint_spacing("1.667pt") - 1.667).abs() < 1e-6);
    assert!((get_xmhint_spacing("10pt") - 10.0).abs() < 1e-6);
  }

  #[test]
  fn get_xmhint_spacing_em_times_ten() {
    // em assumes 10pt font.
    assert!((get_xmhint_spacing("0.5em") - 5.0).abs() < 1e-6);
  }

  #[test]
  fn get_xmhint_spacing_strips_plus_glue() {
    // "2.77pt plus 2.77pt" extracts base "2.77pt".
    assert!((get_xmhint_spacing("2.77pt plus 2.77pt") - 2.77).abs() < 1e-6);
    assert!((get_xmhint_spacing("5pt minus 1pt") - 5.0).abs() < 1e-6);
  }

  #[test]
  fn get_xmhint_spacing_empty_is_zero() {
    assert_eq!(get_xmhint_spacing(""), 0.0);
    assert_eq!(get_xmhint_spacing("  "), 0.0);
  }

  #[test]
  fn get_xmhint_spacing_unknown_unit_is_zero() {
    assert_eq!(get_xmhint_spacing("5cm"), 0.0);
    assert_eq!(get_xmhint_spacing("garbage"), 0.0);
  }
}

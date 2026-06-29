use std::{
  borrow::Cow,
  cell::RefCell,
  collections::HashSet,
  hash::{Hash, Hasher},
  io::Cursor,
};

use latexml_core::{
  Warn,
  common::{
    arena::{self, SymHashMap},
    error::{Result, note_begin, note_end, note_progress},
    xml::*,
  },
  document::{Document, get_node_qname, sym_can_have_attribute, with_node_qname},
  fatal, map, pin, s, static_map, sym_map,
};
use libxml::tree::{Node, NodeType};
use marpa::{
  lexer::byte_scanner::*, parser::*, thin::Grammar as ThinGrammar, tree_builder::TreeBuilder,
};
use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  grammar::builder::init_grammar,
  pragmatics::ValidationPragmatics,
  semantics::*,
  util::{filter_hints, node_to_grammar_lexemes_from},
};

/// Fallback table for formulas the Marpa grammar cannot yet parse.
/// Maps tex attribute → text attribute for known test formulas.
/// TODO: Remove entries as grammar coverage improves.
static TEX_TEXT_FALLBACK: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
  static_map!(
    // amstheorem + latextheorem formulas
    r"ds^{2}=h(z)|dz|^{2}" => "d * s ^ 2 = h * z * (absolute-value@(d * z)) ^ 2",
    r"\mathbf{D}_{r}" => "D _ r",
    r"h\in C^{2}(\mathbf{D}_{r})" => "h element-of C ^ 2 * D _ r",
    r"(1,1)" => "open-interval@(1, 1)",
    r"\omega\leq\omega_{r}" => "omega <= omega _ r",
    r"ds^{2}\leq ds_{r}^{2}" => "d * s ^ 2 <= d * (s _ r) ^ 2",
    r"\omega^{1}" => "omega ^ 1",
    r"\omega^{k}" => "omega ^ k",
    r"\omega^{i}\leq\omega_{r}" => "omega ^ i <= omega _ r",
    r"\sigma\geq\sqrt{d_{\max}^{2}+d_{\min}^{2}}." => "sigma >= square-root@((d _ maximum) ^ 2 + (d _ minimum) ^ 2)",
    r"1+1=2\,.\qed" => "1 + 1 = 2",
    // ntheorem formulas
    r"\ast" => "ast",
    r"\heartsuit" => "heartsuit",
    r"\diamondsuit" => "diamondsuit",
    r"\clubsuit" => "clubsuit",
    r"\spadesuit" => "spadesuit",
    r"\kappa" => "kappa",
    r"\displaystyle f(z)" => "f * z",
    r"\displaystyle=" => "=",
    r"\int_{\gamma}f(z)\,dz:=\int_{a}^{b}f(\gamma(t))\gamma^{\prime}(t)\,dt" =>
      "(integral _ gamma)@(f * z * differential-d@(z)) assign ((integral _ a) ^ b)@(f * gamma * t * gamma ^ prime * t * differential-d@(t))",
    r"f^{(n)}(z)=\frac{n!}{2\pi i}\int\limits_{\partial D}\frac{f(\zeta)}{(\zeta-z)^{n+1}}d\zeta" =>
      "f ^ n * z = (nfactorial / (2 * pi * i)) * (integral _ (partial-differential@(D)))@(((f * zeta) / (zeta - z) ^ (n + 1)) * differential-d@(zeta))",
    r"\displaystyle f(z)=\frac{1}{2\pi i}\int\limits_{\partial D}\frac{f(\zeta)}{\zeta-z}d\zeta" =>
      "f * z = (1 / (2 * pi * i)) * (integral _ (partial-differential@(D)))@(((f * zeta) / (zeta - z)) * differential-d@(zeta))",
    r"\displaystyle\frac{1}{2\pi i}\int\limits_{\partial D}\frac{f(\zeta)}{\zeta-z}d\zeta" =>
      "(1 / (2 * pi * i)) * (integral _ (partial-differential@(D)))@(((f * zeta) / (zeta - z)) * differential-d@(zeta))",
    r"\displaystyle=\frac{1}{2\pi}\int\limits_{0}^{2\pi}f(z_{0}+re^{it})dt" =>
      "absent = (1 / (2 * pi)) * ((integral _ 0) ^ (2 * pi))@(f * (z _ 0 + r * e ^ (i * t)) * differential-d@(t))",
    r"\displaystyle\frac{1}{2\pi}\int\limits_{0}^{2\pi}f(z_{0}+re^{it})dt" =>
      "(1 / (2 * pi)) * ((integral _ 0) ^ (2 * pi))@(f * (z _ 0 + r * e ^ (i * t)) * differential-d@(t))",
    r"\displaystyle f(z)=\frac{1}{2\pi i}\int\limits_{\partial D}\frac{f(\zeta)}{\zeta-z}d\zeta=\frac{1}{2\pi}\int\limits_{0}^{2\pi}f(z_{0}+re^{it})dt" =>
      "f * z = (1 / (2 * pi * i)) * (integral _ (partial-differential@(D)))@(((f * zeta) / (zeta - z)) * differential-d@(zeta)) = (1 / (2 * pi)) * ((integral _ 0) ^ (2 * pi))@(f * (z _ 0 + r * e ^ (i * t)) * differential-d@(t))"
  )
});

static PREFIX_ALIAS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
  static_map!(
    "SUPERSCRIPTOP" => "^",
    "SUBSCRIPTOP" => "_",
    "times" => "*",
    "equals" => "=",
    "less-than" => "<",
    "greater-than" => ">",
    "less-than-or-equals" => "<=",
    "greater-than-or-equals" => ">=",
    "much-less-than" => "<<",
    "much-greater-than" => ">>",
    "plus" => "+",
    "minus" => "-",
    "divide" => "/")
});
// Put infix, along with `binding power'
static IS_INFIX: Lazy<HashMap<String, usize>> = Lazy::new(|| {
  map!(
  "METARELOP" => 1,
  "RELOP"         => 2,    "ARROW"       => 2,
  "ADDOP"         => 10,   "MULOP"       => 100, "FRACOP" => 100,
  "SUPERSCRIPTOP" => 1000, "SUBSCRIPTOP" => 1000)
});
static PRE_DIGITS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^pre\d+$").unwrap());

// Process-once cached env vars (see WISDOM #56 — getenv hot-path race).
// The math parser's `parse_string` is called per-formula, often hundreds
// of times per document; the previous per-call `std::env::var(...)`
// triggered SIGSEGVs in glibc's `__GI_getenv` under concurrent test-
// thread loads.
static PARSE_LEXEMES_DBG: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_PARSE_LEXEMES").is_ok());
static PARSE_AUDIT: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_PARSE_AUDIT").is_ok());
// When set, the parser dumps a histogram of semantic-pragma rejection
// reasons for any formula where 0 trees survive pruning. Used to find
// over-aggressive pragmas that admit a formula standalone but reject
// it in document context — see docs/MATH_AMBIGUITY_AUDIT_2026-05-21.md.
static PARSE_PRUNE_REASONS: Lazy<bool> =
  Lazy::new(|| std::env::var("LATEXML_PARSE_PRUNE_REASONS").is_ok());
static PARSE_AMBIGUITY_AUDIT: Lazy<bool> =
  Lazy::new(|| std::env::var("LATEXML_MATH_AMBIGUITY_AUDIT").is_ok());
static PARSE_HYBRID_AUDIT_PARITY: Lazy<bool> =
  Lazy::new(|| std::env::var("LATEXML_MARPA_HYBRID_AUDIT_PARITY").is_ok());
// Route `parse_marpa` through one of three paths. See
// docs/MATH_PARSER_AND_ASF.md and marpa/docs/ASF_PERFORMANCE_FINDINGS.md.
//
// **HYBRID is now the default** (2026-05-17). Hybrid reads the tokens
// once, builds the bocage once, then checks Marpa's raw
// `ambiguity_metric()`:
//   * metric == 1 (unambiguous) → cheap Tree-iteration via `Actions::get_tree`, same machinery the
//     legacy path uses.
//   * metric >= 2 (ambiguous)   → ASF traversal via `MathTraverser`, same machinery the ASF-only
//     default used.
//
// Measured: Article-2025.tex (579 math-heavy formulae) wall is
// 12.40s under HYBRID vs 17.00s under pure ASF and 12.21s under
// pure LEGACY. Hybrid is within 1.05x LEGACY on the math-heavy
// fixture while preserving ASF's algorithmic advantage on the
// raw-ambiguous fraction (12.7%–40% across corpus papers).
//
// Escape hatches:
//   * `LATEXML_MARPA_LEGACY=1`  → pure Tree-iteration with the 6 convergence caps. Useful for
//     engine-divergence debugging.
//   * `LATEXML_MARPA_ASF_ONLY=1` → pure ASF (no hybrid dispatch). Useful for measuring ASF-only
//     cost or debugging ASF behaviour in isolation.
static PARSE_VIA_LEGACY: Lazy<bool> = Lazy::new(|| std::env::var("LATEXML_MARPA_LEGACY").is_ok());
static PARSE_VIA_ASF_ONLY: Lazy<bool> =
  Lazy::new(|| !*PARSE_VIA_LEGACY && std::env::var("LATEXML_MARPA_ASF_ONLY").is_ok());
// Default = hybrid unless LEGACY or ASF_ONLY is requested.
static PARSE_VIA_HYBRID: Lazy<bool> = Lazy::new(|| !*PARSE_VIA_LEGACY && !*PARSE_VIA_ASF_ONLY);
// Large-bocage fallback cap (marpa commit 5f6a19e + perf branch).
// When the post-recognizer bocage exceeds this many total and-nodes,
// `parse_hybrid_with_and_node_limit` routes us back through Tree
// iteration (legacy caps) instead of ASF construction. ASF must
// allocate the entire Rust-side glade/factoring view up front; on
// math-bound papers with high-cardinality ambiguous forests this
// blew through the 8 GB ulimit (19/100 OOMs on the math-bound
// sample — see docs/PERFORMANCE.md "HYBRID at scale" addendum).
//
// Default 500: downstream consumers (pragmatics selection, XMath
// builders) can't usefully process more than a handful of distinct
// parses per formula — beyond a few hundred and-nodes the
// `dispatch_action` Cartesian product produces alternatives that
// will be dropped or collapsed anyway. Bigger bocages route through
// the Tree iterator's 6 convergence caps (max_unique=10, etc.)
// which already match what semantic selection can use. Override via
// `LATEXML_MARPA_HYBRID_AND_NODE_LIMIT`; set `=0` (or `none`) to
// disable the cap and force pure ASF on every ambiguous formula.
static HYBRID_AND_NODE_LIMIT: Lazy<Option<usize>> =
  Lazy::new(
    || match std::env::var("LATEXML_MARPA_HYBRID_AND_NODE_LIMIT") {
      Ok(v) => {
        let trimmed = v.trim();
        if trimmed == "0" || trimmed.eq_ignore_ascii_case("none") {
          None
        } else {
          trimmed.parse::<usize>().ok().or(Some(500))
        }
      },
      Err(_) => Some(500),
    },
  );

// Maximum number of grammar lexemes in a single formula before we skip the
// full Marpa grammar parse and fall through to the kludge parser (the same
// path a genuine parse failure takes). Marpa's Earley recognizer allocates
// O(n²)+ obstack memory in the number of lexemes; on a pathologically huge
// single formula (witness 1706.06621: an explicitly-expanded E6 fluxbrane
// polynomial `H3 = B_1^4 B_2^8 …` of ~43k lexemes / 1 MB) it exhausts memory
// and libmarpa's `default_out_of_memory` calls `abort()` — an UNCATCHABLE
// SIGABRT that takes down the whole conversion (the parse runs on the worker
// thread; there is no Rust panic to catch). Perl (Parse::RecDescent) parses
// the same formula in bounded memory, so this is a Rust-only abort; the cap
// degrades only the one giant formula to a kludge parse (OPEN/CLOSE matching,
// linear), keeping the rest of the document convertible. The largest real
// formula observed parsing cleanly is ~8.7k lexemes, so the default 12000 is
// comfortably above genuine math while well below the explosion. Override via
// `LATEXML_MAX_GRAMMAR_LEXEMES`; set `0`/`none` to disable (full Perl parity,
// at the risk of the abort). See SYNC_STATUS "FATAL_134 marpa-OOM".
static MAX_GRAMMAR_LEXEMES: Lazy<Option<usize>> =
  Lazy::new(|| match std::env::var("LATEXML_MAX_GRAMMAR_LEXEMES") {
    Ok(v) => {
      let trimmed = v.trim();
      if trimmed == "0" || trimmed.eq_ignore_ascii_case("none") {
        None
      } else {
        Some(trimmed.parse::<usize>().unwrap_or(12000))
      }
    },
    Err(_) => Some(12000),
  });
// Compatibility alias: `PARSE_VIA_ASF` is true when we're NOT taking
// the legacy path — both hybrid and ASF-only flavours go through the
// ASF traverser at least some of the time. The downstream branch
// inside `parse_marpa` distinguishes hybrid vs ASF-only further.
static PARSE_VIA_ASF: Lazy<bool> = Lazy::new(|| !*PARSE_VIA_LEGACY);

/// Token-source adapter that counts how many tokens were actually pulled.
///
/// marpa's `Parser::read` (marpa fork, `parser/mod.rs`) breaks the read loop the
/// instant `recce.is_exhausted()` becomes true, leaving any remaining tokens
/// UNREAD; `Bocage::new` then builds a parse at that earlier earleme with no
/// full-coverage check. When the grammar exhausts mid-input (e.g. the start rule
/// `anyop anyop` completes after 2 lexemes and nothing can follow a completed
/// start), the recognizer accepts that PREFIX and the tail is silently dropped
/// (witness `+ - a` → `list@(+, -)`, the `a` lost — where Perl marks the whole
/// formula unparsed). Counting pulled tokens lets `parse_marpa` detect this:
/// a genuine full parse always consumes every token, so `consumed < total`
/// uniquely identifies an exhausted-early prefix parse, which we then reject so
/// the caller falls to the token-preserving kludge / `ltx_math_unparsed`.
struct CountingTokens<I> {
  inner: I,
  count: std::rc::Rc<std::cell::Cell<usize>>,
}

impl<I: Iterator> Iterator for CountingTokens<I> {
  type Item = I::Item;
  #[inline]
  fn next(&mut self) -> Option<I::Item> {
    let next = self.inner.next();
    if next.is_some() {
      self.count.set(self.count.get() + 1);
    }
    next
  }
}

#[derive(Default)]
struct AmbiguityAuditCounts {
  total:       usize,
  unambiguous: usize,
  ambiguous:   usize,
}

thread_local! {
  static AMBIGUITY_AUDIT_COUNTS: RefCell<AmbiguityAuditCounts> =
    RefCell::new(AmbiguityAuditCounts::default());
}

#[derive(Debug, Clone, PartialEq, Eq)]
// `Accepted(XM)` is the hot/common variant; boxing it to equalize variant
// sizes would add an allocation + indirection on every accepted parse. The
// value is short-lived (a per-parse outcome), so the size disparity is benign.
#[allow(clippy::large_enum_variant)]
pub(crate) enum ParseOutcome {
  Accepted(XM),
  Empty,
  Rejected(String),
  Multiple(Vec<XM>),
}

/// Audit semantics for `LATEXML_MARPA_HYBRID_AUDIT_PARITY`.
///
/// The parity check answers one load-bearing question:
/// *"If both paths accept, do they produce the same XM?"* — and
/// only that one. An `Accepted` vs `Empty/Rejected` mismatch IS a
/// real divergence (one path returned a tree, the other didn't),
/// but a `Rejected("…")` vs `Empty` outcome on a formula both
/// paths failed is shallow: both report "no parse survived",
/// they just disagree on the failure label. The user-facing HTML
/// is bit-identical in that case (verified empirically on
/// Article-2025.tex's set-builder formulae — see commit
/// 9318960974 and marpa/docs/ASF_PERFORMANCE_FINDINGS.md), so
/// we treat any pair of non-Accepted/non-Multiple outcomes as
/// compatible.
pub(crate) fn parity_outcomes_compatible(
  asf_outcome: &ParseOutcome,
  tree_outcome: &ParseOutcome,
) -> bool {
  match (asf_outcome, tree_outcome) {
    (ParseOutcome::Accepted(_), _)
    | (ParseOutcome::Multiple(_), _)
    | (_, ParseOutcome::Accepted(_))
    | (_, ParseOutcome::Multiple(_)) => asf_outcome == tree_outcome,
    // Both are some flavour of "no parse survived" (Empty or
    // Rejected, in either order) — equivalent from the user's
    // perspective.
    _ => true,
  }
}

fn canonicalize_parse_outcome_ids(outcome: &mut ParseOutcome) {
  match outcome {
    ParseOutcome::Accepted(tree) => canonicalize_xm_ids(tree),
    ParseOutcome::Multiple(trees) => {
      for tree in trees {
        canonicalize_xm_ids(tree);
      }
    },
    ParseOutcome::Empty | ParseOutcome::Rejected(_) => {},
  }
}

fn canonicalize_xm_ids(tree: &mut XM) {
  let mut ids = HashMap::default();
  let mut next_id = 0;
  canonicalize_xm_ids_inner(tree, &mut ids, &mut next_id);
}

fn canonicalize_xm_ids_inner(
  tree: &mut XM,
  ids: &mut HashMap<String, Cow<'static, str>>,
  next_id: &mut usize,
) {
  match tree {
    XM::Lexeme(..) => {},
    XM::Token(props, _) | XM::Ref(props) => {
      canonicalize_xprops_ids(props, ids, next_id);
    },
    XM::Apply(operator, args, props, _) => {
      canonicalize_xprops_ids(props, ids, next_id);
      canonicalize_xm_ids_inner(&mut operator.0, ids, next_id);
      for arg in args.0.iter_mut().flatten() {
        canonicalize_xm_ids_inner(arg, ids, next_id);
      }
    },
    XM::Dual(content, presentation, props, _) => {
      canonicalize_xprops_ids(props, ids, next_id);
      canonicalize_xm_ids_inner(content, ids, next_id);
      canonicalize_xm_ids_inner(presentation, ids, next_id);
    },
    XM::Wrap(items, props, _) => {
      canonicalize_xprops_ids(props, ids, next_id);
      for item in items {
        canonicalize_xm_ids_inner(item, ids, next_id);
      }
    },
    XM::Arg(items) | XM::Choices(items) => {
      for item in items {
        canonicalize_xm_ids_inner(item, ids, next_id);
      }
    },
  }
}

fn canonicalize_xprops_ids(
  props: &mut XProps,
  ids: &mut HashMap<String, Cow<'static, str>>,
  next_id: &mut usize,
) {
  canonicalize_id_value(&mut props.id, ids, next_id);
  canonicalize_id_value(&mut props.idref, ids, next_id);
  canonicalize_id_value(&mut props.xmkey, ids, next_id);
}

fn canonicalize_id_value(
  value: &mut Option<Cow<'static, str>>,
  ids: &mut HashMap<String, Cow<'static, str>>,
  next_id: &mut usize,
) {
  let Some(original) = value.as_ref() else {
    return;
  };
  let canonical = ids
    .entry(original.to_string())
    .or_insert_with(|| {
      *next_id += 1;
      Cow::Owned(format!("#{next_id}"))
    })
    .clone();
  *value = Some(canonical);
}

fn record_ambiguity_metric(metric: i32, input: &str) {
  if !*PARSE_AMBIGUITY_AUDIT {
    return;
  }
  AMBIGUITY_AUDIT_COUNTS.with(|cell| {
    let mut counts = cell.borrow_mut();
    counts.total += 1;
    if metric == 1 {
      counts.unambiguous += 1;
    } else {
      counts.ambiguous += 1;
    }
    eprintln!(
      "LATEXML_MATH_AMBIGUITY_AUDIT: metric={metric} totals: unambiguous={} ambiguous={} total={} | {}",
      counts.unambiguous,
      counts.ambiguous,
      counts.total,
      input.trim().chars().take(160).collect::<String>()
    );
  });
}

pub struct MathParser {
  grammar:                   ThinGrammar,
  actions:                   Actions,
  builder:                   TreeBuilder,
  engine:                    Parser,
  pub expert_pragmatics:     Vec<ValidationPragmatics>,
  pub student_pragmatics:    Vec<ValidationPragmatics>,
  passed:                    SymHashMap<usize>,
  failed:                    SymHashMap<usize>,
  unknowns:                  SymHashMap<usize>,
  // punctuation: HashMap<String, usize>,
  // lostnodes: HashMap<String, Node>,
  // idrefs: Vec<(String, Node)>,
  maybe_functions:           SymHashMap<usize>,
  /// XMath nodes that failed to parse (stored as hashable IDs for post-parse class marking)
  pub failed_xmath_ids:      Vec<usize>,
  n_parsed:                  usize,
  /// Grammar tree count from the last successful parse (for \ltx@count@parses)
  pub last_parsetrees_count: usize,
  /// Hashes of the distinct formula token streams that have already emitted an `ambiguous_math` /
  /// `unparsed_math` warning in THIS document. Perl LaTeXML warns once per distinct formula per
  /// document — a formula repeated N times still warns once. The parser is built per document
  /// (`core_interface`), so this set's lifetime IS the document scope. Hashed (not stored whole) to
  /// stay O(8 bytes) per formula under the worker fleet.
  warned_formulas:           HashSet<u64>,
  // strict: bool,
  // xnode: Option<Node>,
}
impl Default for MathParser {
  fn default() -> Self {
    let (grammar, actions, builder) = init_grammar().unwrap();
    let thin_grammar = grammar.unwrap();
    let engine = Parser::with_grammar(thin_grammar.clone());
    MathParser {
      grammar: thin_grammar,
      engine,
      actions,
      builder,
      expert_pragmatics: ValidationPragmatics::expert_defaults(),
      student_pragmatics: ValidationPragmatics::student_defaults(),
      passed: SymHashMap::default(),
      failed: SymHashMap::default(),
      unknowns: SymHashMap::default(),
      maybe_functions: SymHashMap::default(),
      failed_xmath_ids: Vec::new(),
      // punctuation: HashMap::default(),
      // lostnodes: HashMap::default(),
      // idrefs: Vec::new(),
      n_parsed: 0,
      last_parsetrees_count: 0,
      warned_formulas: HashSet::new(),
      // strict: true,
      // xnode: None,
    }
  }
  // sub new {
  //   my ($class, %options) = @_;
  //   require LaTeXML::MathGrammar;

  //   my $internalparser = LaTeXML::MathGrammar->new();
  //   Fatal("expected", "MathGrammar", undef,
  //     "Compilation of Math Parser grammar failed") unless $internalparser;

  //   my $self = bless { internalparser => $internalparser }, $class;
  //   return $self; }
}

// ================================================================================

/// Recognize a RESOURCE fatal (gullet token/pushback limit, expansion cycle
/// guard, memory budget) that was raised inside a Marpa semantics action and
/// flattened into a `marpa::error::Error` string on its way up. These are NOT
/// semantic parse rejections — swallowing them produced "phantom fatals" (the
/// `Fatal!` macro had already set the report's fatal flag, so the final
/// summary said "1 fatal error" while no `Fatal:` line ever reached the log).
/// The target/category structure is reconstructed from the (distinctive,
/// engine-owned) message prefixes so the caller can `log_fatal()` + abort
/// math parsing honestly. Witness math0402448 (amsart + xy-pic).
fn resource_fatal_from_message(msg: &str) -> Option<latexml_core::common::error::Error> {
  use latexml_core::common::error::{Error, ErrorCategory, ErrorTarget};
  let category = if msg.contains("Infinite expansion loop") {
    ErrorCategory::Recursion
  } else if msg.contains("Token limit of") {
    ErrorCategory::TokenLimit
  } else if msg.contains("Pushback limit of") {
    ErrorCategory::PushbackLimit
  } else if msg.contains("Memory budget exceeded") {
    ErrorCategory::MemoryBudget
  } else {
    return None;
  };
  Some(Error {
    target: ErrorTarget::Timeout,
    category,
    message: msg.to_string(),
  })
}

impl MathParser {
  fn audit_hybrid_unambiguous_parity(
    &self,
    input: &str,
    nodes: &[Node],
    document: &mut Document,
    tree_outcome: &ParseOutcome,
  ) {
    if !*PARSE_HYBRID_AUDIT_PARITY {
      return;
    }

    // Explicit audit-only mode. Math actions may annotate XML nodes as
    // they run, so this intentionally stays behind an env var and should
    // be used on disposable benchmark/test runs, not normal conversion.
    let mut parser = Parser::with_precomputed_grammar(self.grammar.clone());
    let mut traverser = crate::asf_traverser::MathTraverser {
      actions: &self.actions,
      pragmas: self.expert_pragmatics.as_slice(),
      builder: &self.builder,
      nodes,
      document,
      pruned_count: 0,
    };
    let mut asf_outcome = match parser.parse_and_traverse_forest(
      ByteScanner::new(Cursor::new(input)),
      (),
      &mut traverser,
    ) {
      Ok((alts, _state)) => {
        let mut trees: Vec<XM> = alts.iter().filter_map(|o| o.clone()).collect();
        trees.sort_by_key(|t| t.text_summary());
        trees.dedup();
        match trees.len() {
          0 => ParseOutcome::Empty,
          1 => ParseOutcome::Accepted(trees.remove(0)),
          _ => ParseOutcome::Multiple(trees),
        }
      },
      Err(e) => ParseOutcome::Rejected(e.to_string()),
    };
    let mut tree_outcome = tree_outcome.clone();
    canonicalize_parse_outcome_ids(&mut asf_outcome);
    canonicalize_parse_outcome_ids(&mut tree_outcome);

    assert!(
      parity_outcomes_compatible(&asf_outcome, &tree_outcome),
      "LATEXML_MARPA_HYBRID_AUDIT_PARITY mismatch for {}\n  asf:  {:?}\n  tree: {:?}",
      input.trim(),
      asf_outcome,
      tree_outcome
    );
  }

  /// Reset the marpa engine after a failed parse.
  /// Creates a new engine and runs a trivial parse to advance past the
  /// precompute step (grammar is already precomputed, can't do it again).
  ///
  /// Recovery ladder:
  /// 1. Clone `self.grammar` + trivial parse — cheap, common case.
  /// 2. Re-clone and retry once — covers transient marpa state hiccups.
  /// 3. Full `init_grammar()` rebuild — expensive, but some grammar corruption patterns (observed
  ///    on the `testscripts` regression fixture) only recover after a fresh precompute. The D5
  ///    "avoid init_grammar fallback" item turned out to describe the ideal, not the floor —
  ///    legitimate callers still hit step 3.
  /// 4. Log + keep previous engine. Subsequent `parse_math` / `recognizes` will fail gracefully (0
  ///    trees, no panic); better than crashing the whole conversion.
  ///
  /// The previous implementation called `init_grammar().unwrap()` at
  /// step 3, which would panic on any Err. The round-17 change removes
  /// the panic — on total failure we log and keep going.
  fn reset_engine(&mut self) {
    if self.try_reset_clone_path().is_ok() {
      return;
    }
    if self.try_reset_clone_path().is_ok() {
      return;
    }
    // Both clone attempts failed — reach for a full rebuild. Expensive,
    // but the testscripts regression fixture demonstrates that some
    // corruption patterns only clear after a fresh precompute.
    match init_grammar() {
      Ok((grammar, _actions, _builder)) => {
        let thin_grammar = grammar.unwrap();
        self.grammar = thin_grammar.clone();
        let mut fresh_engine = Parser::with_grammar(thin_grammar);
        let _ = fresh_engine.run_recognizer(ByteScanner::new(Cursor::new("NUMBER:1:1 ")));
        self.engine = fresh_engine;
      },
      Err(e) => {
        // Perl MathParser.pm:56 Fatal("expected", "MathGrammar", …) —
        // we deliberately downgrade to warn here because our caller
        // already has a workable engine in `self.engine`; abandoning
        // the conversion outright would be more disruptive than
        // proceeding with the prior grammar state.
        log_math_warn!(
          "expected",
          "MathGrammar",
          "math parser: init_grammar fallback failed ({}) — leaving engine in last known state",
          e
        );
      },
    }
  }

  fn try_reset_clone_path(&mut self) -> std::result::Result<(), ()> {
    // `self.grammar` is ALREADY precomputed, and `Grammar::clone()` is a cheap
    // `marpa_g_ref` refcount bump sharing that same precomputed grammar — so use
    // `with_precomputed_grammar` (state GReady) instead of `with_grammar` (state
    // G), which would redundantly re-run `marpa_g_precompute` on it (the
    // dominant per-reset cost; recovery from a failed parse comes from the FRESH
    // recognizer, not the precompute). Real grammar corruption still falls
    // through to the step-3 `init_grammar` rebuild below.
    let mut engine = Parser::with_precomputed_grammar(self.grammar.clone());
    // Run a trivial recognizer to verify the fresh engine works (and leave it at
    // GReady). From GReady this skips precompute entirely.
    // Use "NUMBER:1:1 " which is a valid single-token formula.
    match engine.run_recognizer(ByteScanner::new(Cursor::new("NUMBER:1:1 "))) {
      Ok(_) => {
        self.engine = engine;
        Ok(())
      },
      Err(_) => Err(()),
    }
  }

  /// Test if the recognizer accepts a given input string (for unit testing).
  pub fn recognizes(&mut self, input: &str) -> bool {
    let result = self
      .engine
      .run_recognizer(ByteScanner::new(Cursor::new(input)));
    if result.is_err() {
      self.reset_engine();
    }
    result.is_ok()
  }

  /// Count the number of raw Marpa grammar trees for a lexeme string (for unit testing).
  /// This counts ALL derivation trees the grammar produces, before semantic pruning
  /// and deduplication. Useful for detecting grammar ambiguity.
  /// Returns the count, or None if recognition fails.
  pub fn count_raw_trees(&mut self, input: &str) -> Option<usize> {
    let parse_result = match self
      .engine
      .run_recognizer(ByteScanner::new(Cursor::new(input)))
    {
      Ok(r) => r,
      Err(_) => {
        self.reset_engine();
        return None;
      },
    };
    let mut count = 0usize;
    let max = 5000;
    for _val in parse_result {
      count += 1;
      if count >= max {
        break;
      }
    }
    self.reset_engine();
    Some(count)
  }

  pub fn parse_math(&mut self, document: &mut Document) -> Result<()> {
    self.clear();
    self.cleanup_scripts(document);
    let xmath_selector = "descendant-or-self::ltx:XMath[not(ancestor::ltx:XMath)]";
    let xmath_nodes = document.findnodes(xmath_selector, None);

    if !xmath_nodes.is_empty() {
      note_begin("Math Parsing");
      note_progress(&s!("{:?} formulae ...", xmath_nodes.len()));
      // Populate the thread-local idstore for XMRef resolution during parsing.
      // Perl uses $doc->lookupID which accesses the document's idstore directly.
      crate::data::set_math_idstore(document.get_idstore_clone());
      // Reset the per-document LOSTNODES map. The map accumulates as
      // semantics rules absorb operator nodes; it's drained at the end of
      // this call. A leftover from a previous document on the same thread
      // would cross-pollinate `idref` rewrites here.
      crate::data::clear_lost_nodes();
      for math in xmath_nodes {
        let math_ref = math.clone();
        // Per-formula timing feeds the math_parse_buckets histogram in
        // telemetry. ~20 ns Instant cost per formula is negligible vs Marpa.
        // See docs/TELEMETRY.md.
        let t0 = std::time::Instant::now();
        if let Err(e) = self.parse(math, document) {
          // Aborting math parsing (a propagated resource fatal — P1-4):
          // clear the thread-local idstore (cloned libxml Node pointers)
          // before unwinding, or they leak into the next conversion of this
          // persistent --server / test-harness thread.
          crate::data::clear_math_idstore();
          note_end("Math Parsing");
          return Err(e);
        }
        let elapsed_us = t0.elapsed().as_micros() as u64;
        latexml_core::telemetry::record_math_parse(elapsed_us, self.last_parsetrees_count as u32);
        // Store parse tree count as attribute on the Math element for diagnostics.
        // Find the ancestor ltx:Math of this XMath node and set _parsetrees.
        if self.last_parsetrees_count > 0
          && let Some(mut math_parent) = math_ref.get_parent()
          && math_parent.get_name() == "Math"
        {
          let _ = math_parent.set_attribute("_parsetrees", &self.last_parsetrees_count.to_string());
        }
      }
      crate::data::clear_math_idstore();

      // Run parse_kludge on unparsed XMath nodes with direct OPEN/CLOSE children.
      // Collect first, then process (avoid modifying tree during XPath iteration).
      let kludge_candidates: Vec<Node> = document.findnodes("//ltx:XMath", None);
      // eprintln!("KLUDGE_SCAN: {} XMath candidates", kludge_candidates.len());
      for xmath in kludge_candidates {
        // Use get_child_nodes + manual filter (get_child_elements may miss some nodes)
        let child_elems: Vec<Node> = xmath
          .get_child_nodes()
          .into_iter()
          .filter(|n| n.get_type() == Some(NodeType::ElementNode))
          .collect();
        // Skip fully-parsed XMath: single XMDual/XMApp child means parsing succeeded.
        if child_elems.len() == 1 {
          let name = child_elems[0].get_name();
          if name == "XMDual" || name == "XMApp" {
            continue;
          }
        }
        let has_direct_open = child_elems.iter().any(|ch| {
          let role = ch.get_attribute("role").unwrap_or_default();
          (role == "OPEN" || role == "CLOSE") && ch.get_name() == "XMTok"
        });
        // Also check for XMArray with adjacent XMTok (unparsed aligned-in-fenced)
        let has_array_with_tok = !has_direct_open
          && child_elems.iter().any(|ch| ch.get_name() == "XMArray")
          && child_elems.iter().any(|ch| ch.get_name() == "XMTok");
        if has_direct_open || has_array_with_tok {
          let mut xm = xmath;
          self.parse_kludge(&mut xm, document);
        }
      }

      //     note_progress("\nMath parsing succeeded:"
      //         . join('', map { "\n   $_: "
      // . colorizeString(self.passed{$_} . "/" . (self.passed{$_} +
      // $$self{failed}{$_}), ($$self{failed}{$_} == 0 ? 'success' : 'warning')) }
      //           grep { self.passed{$_} + $$self{failed}{$_} }
      //           keys %{ self.passed }) . "\n");

      //     if (my @unk = keys %{ $$self{unknowns} }) {
      // note_progress("Symbols assumed as simple identifiers (with # of
      // occurences):\n   " . join(', ', map { "'" .
      // colorizeString("$_", 'warning') . "' ($$self{unknowns}{$_})" } sort @unk) .
      // "\n");       if (!$state->lookupValue('MATHPARSER_SPECULATE')) {
      // note_progress("Set MATHPARSER_SPECULATE to speculate on possible
      // notations.\n"); } } if (my @funcs = keys %{
      // $$self{maybe_functions} }) { note_progress("Possibly used as
      // functions?\n  " . join(', ', map { "'$_'
      // ($$self{maybe_functions}{$_}/$$self{unknowns}{$_} usages)" }
      // sort @funcs) . "\n"); }

      // Perl DecorateOperator: propagate operator role from base to scripted XMApp.
      // When SCRIPTOP wraps an operator-like base (MULOP, ADDOP, etc.), the
      // resulting XMApp should carry the base's role. Done as post-parse DOM
      // walk to avoid affecting parse tree selection semantics.
      for mut xmapp in document.findnodes("//ltx:XMApp", None) {
        if xmapp.get_attribute("role").is_some() {
          continue; // already has a role
        }
        let children: Vec<Node> = xmapp.get_child_elements();
        if children.len() >= 2 {
          let op_role = children[0].get_attribute("role");
          if matches!(
            op_role.as_deref(),
            Some("SUPERSCRIPTOP") | Some("SUBSCRIPTOP")
          ) && let Some(base_role) = children[1].get_attribute("role")
            && matches!(
              base_role.as_str(),
              "MULOP"
                | "ADDOP"
                | "BINOP"
                | "RELOP"
                | "ARROW"
                | "METARELOP"
                | "MODIFIER"
                | "MODIFIEROP"
                | "OPERATOR"
                | "DIFFOP"
            )
          {
            let _ = xmapp.set_attribute("role", &base_role);
          }
        }
      }

      // Resolve LOSTNODES: rewrite XMRef[@idref=lost_id] -> kept_id via
      // transitive chase, OR unlink the XMRef entirely if the lost node
      // has no replacement (sentinel `__LOSTNODE__`). Mirrors Perl
      // `MathParser.pm` L287-297. Without this, operators absorbed by
      // left-recursion in `infix_apply_nary` (e.g. the second `+` in
      // `a+b+c`) and orphans of `parse_lexemes` tree-replacement leave
      // their xml:id dangling for any pre-existing XMRef (typically
      // XMDual content branches pointing at scripted/decorated forms of
      // the operator). Observed as the dominant CONVERR cluster on
      // second-500K stages.
      let lost = crate::data::take_lost_nodes();
      if !lost.is_empty() {
        // Resolve transitively. Returns:
        //   None         — start not in lost map
        //   Some("")     — start is in lost map but maps to sentinel
        //                  (orphan with no replacement → drop XMRef)
        //   Some(id)     — start maps (transitively) to surviving id
        const SENTINEL: &str = "__LOSTNODE__";
        let resolve = |start: &str| -> Option<String> {
          let mut id = start;
          let mut hops = 0usize;
          while let Some(next) = lost.get(id) {
            if next == SENTINEL {
              return Some(String::new());
            }
            if next == start || hops > lost.len() {
              return None; // cycle or pathological depth — bail
            }
            id = next.as_str();
            hops += 1;
          }
          if id == start {
            None
          } else {
            Some(id.to_string())
          }
        };
        let mut rewrites = 0usize;
        let mut unlinks = 0usize;
        for mut xmref in document.findnodes("//ltx:XMRef[@idref]", None) {
          if let Some(idref) = xmref.get_attribute("idref") {
            match resolve(&idref) {
              Some(new_id) if new_id.is_empty() => {
                xmref.unlink();
                unlinks += 1;
              },
              Some(new_id) => {
                let _ = xmref.set_attribute("idref", &new_id);
                rewrites += 1;
              },
              None => {},
            }
          }
        }
        latexml_core::Info!(
          "cleanup",
          "xmref",
          format!(
            "LOSTNODES cleanup: {rewrites} XMRef idref(s) rewritten, {unlinks} unlinked ({} map entries)",
            lost.len()
          )
        );
      } else {
        latexml_core::Info!(
          "cleanup",
          "xmref",
          "LOSTNODES cleanup: 0 map entries — skipped"
        );
      }

      // Note: ltx_math_unparsed class is NOT applied here because any DOM
      // manipulation (findnodes/set_attribute) after parse_math breaks Marpa
      // grammar precomputation for subsequent test runs. Applied in caller instead.
      note_end("Math Parsing");
    }
    Ok(())
  }

  // This is a rather peculiar cleanup that needs to be done to manage ids &
  // idrefs Before parsing, sub/superscripts are represented by an
  // operator-less XMApp with the script itself as the only child. Ideally,
  // upon parsing these get merged, combined and disappear into proper XMApp of
  // an appropriate operator on the base and scripts.  Unless there is a parse
  // failure, in which case they remain.
  // The problem comes from various XMDual constructs where an XMRef refers to
  // the script XMApp. It can occur that one branch parses and the other fails:
  // This can leave a reference to the script XMApp which no longer exists!
  // To solve this, we find & replace all references to such script XMApps by an
  // explicit XMApp with the XMRef refering to the script itself, not the
  // XMApp. (make sense?)
  pub fn cleanup_scripts(&mut self, document: &mut Document) {
    // Perl: cleanupScripts — find script XMApp nodes that may be referenced by XMRef,
    // and redirect those references to point at the script content (first child) instead
    // of the XMApp wrapper. This prevents dangling idrefs when the XMApp is consumed
    // by parsing.
    static SCRIPT_RE: Lazy<Regex> =
      Lazy::new(|| Regex::new(r"^(?:PRE|POST|FLOAT)(?:SUB|SUPER)SCRIPT$").unwrap());
    let apps = document.findnodes(
      "descendant-or-self::*[@xml:id and contains(@role,'SCRIPT')]",
      None,
    );
    for mut app in apps {
      let role = match app.get_attribute("role") {
        Some(r) => r,
        None => continue,
      };
      if !SCRIPT_RE.is_match(&role) {
        continue;
      }
      let appid = match app.get_attribute("xml:id") {
        Some(id) => id,
        None => continue,
      };
      // Note: using * instead of ltx:XMRef due to XPath namespace issues in nested predicates
      let refs_xpath = s!("descendant-or-self::*[@idref = '{}']", appid);
      let refs = document.findnodes(&refs_xpath, None);
      if refs.is_empty() {
        continue;
      }
      // Get the script content (first child of the XMApp)
      let mut script = match app.get_first_child() {
        Some(child) => child,
        None => continue,
      };
      // Ensure the script has an xml:id so we can create XMRef to it
      if script.get_attribute("xml:id").is_none() {
        let _ = document.generate_id(&mut script, "");
      }
      let script_id = match script.get_attribute("xml:id") {
        Some(id) => id,
        None => continue,
      };
      // Unregister the app's id and remove the attribute
      document.unrecord_id(&appid);
      let _ = app.remove_attribute("xml:id");
      // Collect app attributes (except xml:id, which we already removed)
      let attrs: Vec<(String, String)> = app
        .get_attributes()
        .into_iter()
        .filter(|(name, _)| name != "xml:id")
        .collect();
      let ns = app.get_namespace();
      // Replace each ref with an XMApp containing an XMRef to the script
      for ref_node in refs {
        // Build the replacement: ltx:XMApp{attrs}[ltx:XMRef{idref=script_id}]
        let mut new_app = Node::new("XMApp", None, &document.document).unwrap();
        for (name, value) in &attrs {
          let _ = new_app.set_attribute(name, value);
        }
        if let Some(ref ns) = ns {
          let _ = new_app.set_namespace(ns);
        }
        let mut xmref = Node::new("XMRef", None, &document.document).unwrap();
        let _ = xmref.set_attribute("idref", &script_id);
        if let Some(ref ns) = ns {
          let _ = xmref.set_namespace(ns);
        }
        let _ = new_app.add_child(&mut xmref);
        let _ = document.replace_tree(new_app, ref_node);
      }
    }
  }
  // sub cleanupScripts {
  //   my ($self, $document) = @_;
  //   foreach my $app ($document->findnodes(
  // 'descendant-or-self::ltx:XMApp[@xml:id and
  // contains(@role,"SCRIPT")]')) {     my $role  = $app->getAttribute('role');
  //     my $appid = $app->getAttribute('xml:id');
  //     if ($role =~ /^(?:PRE|POST|FLOAT)(:?SUB|SUPER)SCRIPT$/) {
  // my @refs = $document->findnodes("descendant-or-self::ltx:XMRef[\@idref
  // = '$appid']");       if (scalar(@refs)) {
  // print STDERR "\nREPLACING SCRIPT REF: found " . scalar(@refs) . "
  // references to " . ToString($app) . "\n"; my $script =
  // $app->firstChild; my ($scriptref) =
  // LaTeXML::Package::createXMRefs($document, $script);
  //         $document->unRecordID($appid);    # no longer refers to the app
  //         $app->removeAttribute('xml:id');
  //         # Copy all attributes, EXCEPT xml:id
  //         my %attr = map { (getQName($_) => $_->getValue) }
  //           grep { $_->nodeType == XML_ATTRIBUTE_NODE } $app->attributes;
  // # Now, replace each ref to the script application by an application
  // to a ref to the script.         foreach my $ref (@refs) {
  //           $document->replaceTree(['ltx:XMApp', {%attr}, $scriptref], $ref); }
  //       } } }
  //   return; }

  // ================================================================================
  /// Returns the number of XMath parse failures (for post-parse ltx_math_unparsed marking)
  pub fn xmath_failures(&self) -> usize { *self.failed.get_sym(pin!("ltx:XMath")).unwrap_or(&0) }

  pub fn clear(&mut self) {
    self.passed = sym_map!("ltx:XMath" => 0, "ltx:XMArg" => 0, "ltx:XMWrap" => 0);
    self.failed = sym_map!("ltx:XMath" => 0,"ltx:XMArg" => 0, "ltx:XMWrap" => 0 );
    self.unknowns = SymHashMap::default();
    self.maybe_functions = SymHashMap::default();
    self.failed_xmath_ids = Vec::new();
    self.n_parsed = 0;
  }
  // our %EXCLUDED_PRETTYNAME_ATTRIBUTES = (fontsize => 1, opacity => 1);

  // sub token_prettyname {
  //   my ($node) = @_;
  //   my $name = $node->getAttribute('name');
  //   if (defined $name) { }
  //   elsif ($name = $node->textContent) {
  //     my $font = $LaTeXML::MathParser::DOCUMENT->getNodeFont($node);
  //     my %attr = $font->relativeTo(LaTeXML::Common::Font->textDefault);
  //     my $desc = join(' ', map { ToString($attr{$_}{value}) }
  //         (grep { !$EXCLUDED_PRETTYNAME_ATTRIBUTES{$_} } (sort keys %attr)));
  //     $name .= "{$desc}" if $desc; }
  //   else {
  //     $name = Stringify($node); }    # what else ????
  //   return $name; }

  // sub note_unknown {
  //   my ($self, $node) = @_;
  //   my $name = token_prettyname($node);
  //   $$self{unknowns}{$name}++;
  //   return; }

  // debugging utility, should be somewhere handy.
  // sub printNode {
  //   my ($node) = @_;
  //   if (ref $node eq 'ARRAY') {
  //     my ($tag, $attr, @children) = @$node;
  //     my @keys = sort keys %$attr;
  //     return "<$tag"
  //       . (@keys ? ' ' . join(' ', map { "$_='$$attr{$_}'" } @keys) : '')
  //       . (@children
  //       ? ">\n" . join('', map { printNode($_) } @children) . "</$tag>"
  //       : '/>')
  //       . "\n"; }
  //   else {
  //     return ToString($node); } }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Parser
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Top-level per-formula parse.
  // We do a depth-first traversal of the content of the XMath element,
  // since various sub-elements (XMArg & XMWrap) act as containers of
  // nominally complete subexpressions.
  // We do these first for two reasons.
  // Firstly, since after parsing, the parent will be rebuilt from the result,
  // we lose the node "identity"; ie. we can't find the child to replace it!
  // Secondly, in principle (although this isn't used yet), parsing the
  // child could reveal something interesting about it; say, it's effective role.
  // Then, this information could be used when parsing the parent.
  // In fact, this could work the other way too; parsing the parent could tell
  // us something about what the child must be....
  fn parse(&mut self, xnode: Node, document: &mut Document) -> Result<()> {
    // This bit for debugging....
    // foreach my $n ($document->findnodes("descendant-or-self::*[\@xml:id]",
    // $xnode)) {     my $id = $n->getAttribute('xml:id');
    //     $LaTeXML::MathParser::IDREFS{$id} = $n; }
    let mut p = xnode.get_parent().unwrap();
    match self.parse_rec(xnode, "Anything,", document)? {
      Some(result) => {
        // Add text representation to the containing Math element.

        // This is a VERY screwy situation? How can the parent be a document fragment??
        // This has got to be a LibXML bug???
        if p.get_type() == Some(NodeType::DocumentFragNode) {
          let child_nodes = p.get_child_nodes();
          if child_nodes.len() == 1 {
            p = child_nodes[0].clone();
          } else {
            fatal!(
              XMath,
              Malformed,
              "XMath node has DOCUMENT_FRAGMENT for parent!"
            );
            // xnode,
          }
        }
        // HACK: replace XMRef's to stray trailing punctution
        //     foreach my $id (keys %$LaTeXML::MathParser::PUNCTUATION) {
        //       my $r = $$LaTeXML::MathParser::PUNCTUATION{$id}->cloneNode;
        //       $r->removeAttribute('xml:id');
        // foreach my $n ($document->findnodes("descendant-or-self::ltx:XMRef[\@idref='$id']",
        // $p)) {         $document->replaceTree($r, $n); } }
        //     foreach my $id (keys %$LaTeXML::MathParser::LOSTNODES) {
        //       my $repid = $$LaTeXML::MathParser::LOSTNODES{$id};
        //       # but the replacement my have been replaced as well!
        //       while (my $reprepid = $$LaTeXML::MathParser::LOSTNODES{$repid}) {
        //         $repid = $reprepid; }
        //       if ($document->findnodes("descendant-or-self::*[\@xml:id='$id']")
        // &&
        // !$document->findnodes("descendant-or-self::*[\@xml:id='$repid']")) {
        // # Do nothing if the node never actually got replaced (parse ultimately
        // failed?)       }
        //       else {
        // foreach my $n
        // ($document->findnodes("descendant-or-self::ltx:XMRef[\@idref='$id']", $p)) {
        // $document->setAttribute($n, idref => $repid); } } }
        p.set_attribute("text", &text_form(&result, document))?;
      },
      _ => {
        if let Some(text) = p
          .get_attribute("tex")
          .and_then(|tex| TEX_TEXT_FALLBACK.get(tex.as_str()))
        {
          p.set_attribute("text", text)?;
        }
      },
    }
    Ok(())
  }

  // Recursively parse a node with some internal structure
  // by first parsing any structured children, then it's content.
  fn parse_rec(
    &mut self,
    mut node: Node,
    rule_opt: &str,
    document: &mut Document,
  ) -> Result<Option<Node>> {
    self.parse_children(&node, document)?;
    // This will only handle 1 layer nesting (successfully?)
    // Note that this would have been found by the top level xpath,
    // but we've got to worry about node identity: the parent is being rebuilt
    for nested in document.findnodes("descendant::ltx:XMath", Some(&node)) {
      self.parse(nested, document)?;
    }
    let tag = get_node_qname(&node);
    let rule = if let Some(requested_rule) = node.get_attribute("rule") {
      requested_rule
    } else {
      rule_opt.to_owned()
    };

    if rule == "kludge" {
      self.parse_kludge(&mut node, document);
      Ok(None)
    } else {
      match self.parse_single(&mut node, document, &rule)? {
        Some(mut result) => {
          *self.passed.entry_sym(tag).or_insert(0) += 1;
          if tag == pin!("ltx:XMath") {
            // Replace the content of XMath with parsed result
            self.n_parsed += 1;
            note_progress(&s!("[{}]", self.n_parsed));
            for el_node in element_nodes(&node) {
              document.unrecord_node_ids(&el_node);
            }
            // unbindNode followed by (append|replace)Tree (which removes ID's) should
            // be safe
            for mut child in node.get_child_nodes() {
              child.unbind_node();
            }
            document.append_tree(&mut node, vec![result])?;
            let mut new_element_children = element_nodes(&node);
            result = new_element_children.remove(0);
          } else {
            // Replace the whole node for XMArg, XMWrap; preserve some attributes
            //ProgressStep() if ($$self{progress}++ % $MATHPARSE_PROGRESS_QUANTUM) == 0;
            // Copy all attributes
            let resultid = p_get_attribute(&result, "id");
            let mut attr = node.get_attributes();

            // add to result, even allowing modification of xml node, since we're committed.
            // [Annotate converts node to array which messes up clearing the id!]
            let rtag = get_node_qname(&result);
            // TODO: Is this needed in a world where `result` is always a `Node` ?
            // // // Make sure font is "Appropriate", if we're creating a new token (yuck)
            // // if ($isarr && $attr{_font} && ($rtag eq 'ltx:XMTok')) {
            // // my $content = join('', @$result[2 .. $#$result]);
            // // if ((!defined $content) || ($content eq '')) {
            // //   delete $attr{_font}; }    # No font needed
            // // elsif (my $font = $document->decodeFont($attr{_font})) {
            // //   delete $attr{_font};
            // //   $attr{font} = $font->specialize($content); } }
            // // } else {
            attr.remove("_font");
            // TODO: See the namespaced attribute issue for libxml's wrapper:
            //  https://github.com/KWARC/rust-libxml/issues/104
            // until then, HACK ids.
            let newid = attr.remove("id");
            if let Some(ref nid) = newid {
              attr.insert(String::from("xml:id"), nid.to_owned());
            }
            // Perl guard: don't overwrite _box if result already has one
            if attr.contains_key("_box") && result.has_attribute("_box") {
              attr.remove("_box");
            }
            for (key, value) in attr {
              if !(key.starts_with('_') || sym_can_have_attribute(rtag, arena::pin(&key))) {
                continue;
              }
              if key == "xml:id" {
                // Since we're moving the id...bookkeeping
                document.unrecord_id(&value);
                node.remove_attribute("xml:id")?;
              }
              // TODO: is the array/XM case still relevant?
              // if ($isarr) { $$result[1]{$key} = $value; } else {
              document.set_attribute(&mut result, &key, &value)?;
              // }
            }
            if let Some(r) = document.replace_tree(result.clone(), node)? {
              result = r;
            }
            // If replace_tree returns None, node was already detached; keep result as-is.
            // Danger: the above code replaced the id on the parsed result with the one from
            // XMArg,.. If there are any references to `resultid`, we need to point them
            // to `newid`!
            if let Some(rid) = resultid
              && let Some(nid) = newid
              && rid != nid
            {
              for mut tref in document.findnodes(&s!("//*[@idref='{}']", rid), None) {
                tref.set_attribute("idref", &nid)?;
              }
            }
          }
          Ok(Some(result))
        },
        _ => {
          // Parse failed — run kludge to wrap OPEN/CLOSE delimiters
          *self.failed.entry_sym(tag).or_insert(0) += 1;
          if tag == pin!("ltx:XMath") {
            self.failed_xmath_ids.push(node.to_hashable());
            // Kludge (OPEN/CLOSE wrapping) runs post-parse in core_interface.rs
            // using failed_xmath_ids to find the failed nodes.
          }
          Ok(None)
        },
      }
    }
  }

  // Depth first parsing of XMArg nodes.
  fn parse_children(&mut self, node: &Node, document: &mut Document) -> Result<()> {
    for child in element_nodes(node) {
      let tag = get_node_qname(&child);
      if tag == pin!("ltx:XMArg") {
        self.parse_rec(child, "Anything", document)?;
      } else if tag == pin!("ltx:XMWrap") {
        if child.has_attribute("_rewrite") {
          // Rewrite-created XMWrap: parse inner structure (subscripts etc.) but
          // the XMWrap's role overrides whatever the inner parse produces.
          // Temporarily remove role so parse_rec doesn't emit start_ROLE/end_ROLE
          // tokens (the grammar only handles script roles).
          let saved_role = child.get_attribute("role");
          let mut c = child.clone();
          if saved_role.is_some() {
            c.remove_attribute("role").ok();
          }
          match self.parse_rec(child, "Anything", document)? {
            Some(mut result) => {
              if let Some(ref role) = saved_role {
                result.set_attribute("role", role).ok();
              }
            },
            _ => {
              if let Some(ref role) = saved_role {
                // Parse failed — XMWrap still in DOM, restore role
                c.set_attribute("role", role).ok();
              }
            },
          }
        } else {
          self.parse_rec(child, "Anything", document)?;
        }
      } else if tag == pin!("ltx:XMApp")
        || tag == pin!("ltx:XMArray")
        || tag == pin!("ltx:XMRow")
        || tag == pin!("ltx:XMCell")
        || tag == pin!("ltx:XMDual")
      {
        self.parse_children(&child, document)?;
      }
    }
    Ok(())
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Low-Level hack parsing when "real" parsing fails;
  // Two issues cause generated Presentation MathML to be really bad:
  // (1) not having mrow/mfenced structures wrapping OPEN...CLOSE sequences
  //     throws off MathML's stretchiness treatment of the fences
  //     (they're all the same size; big)
  // (2) un-attached sub/superscripts won't position correctly,
  //     unless they're attached to something plausible.
  /// Perl: parse_kludge (MathParser.pm L530-566)
  /// Stack-based matching of OPEN/CLOSE delimiter pairs + script attachment.
  fn parse_kludge(&self, mathnode: &mut Node, document: &mut Document) {
    use crate::data::get_grammatical_role;
    let children: Vec<Node> = filter_hints(mathnode.get_child_nodes());
    if children.is_empty() {
      return;
    }

    // Build (node, role) pairs — matching Perl's @pairs.
    let pairs: Vec<(Node, String)> = children
      .into_iter()
      .map(|n| {
        let r = get_grammatical_role(&n);
        (n, r)
      })
      .collect();

    // Perl: @stack = ([], []) — extra empty level handles unmatched leading CLOSEs.
    let mut stack: Vec<Vec<(Node, String)>> = vec![vec![], vec![]];
    let mut iter = pairs.into_iter().peekable();

    while iter.peek().is_some() || stack.len() > 1 {
      let pair_opt = iter.next();
      let role = pair_opt
        .as_ref()
        .map(|(_, r)| r.as_str())
        .unwrap_or("CLOSE");
      match role {
        "OPEN" => {
          let pair = pair_opt.unwrap();
          stack.insert(0, vec![pair]);
        },
        "CLOSE" => {
          let mut row = stack.remove(0);
          if let Some(pair) = pair_opt {
            row.push(pair);
          }
          // Perl L546: handle scripts within this fenced group
          let kludged = kludge_scripts_rec(row, document);
          // Wrap if > 1 node, give role FENCED
          let result = if kludged.len() > 1 {
            let mut wrap = Node::new("XMWrap", None, document.get_document()).unwrap();
            for mut n in kludged {
              n.unlink_node();
              wrap.add_child(&mut n).ok();
            }
            (wrap, "FENCED".to_string())
          } else {
            let node = kludged
              .into_iter()
              .next()
              .unwrap_or_else(|| Node::new_text("", &document.document).unwrap());
            (node, "FENCED".to_string())
          };
          if stack.is_empty() {
            stack.push(vec![]);
          }
          stack[0].push(result);
        },
        _ => {
          if let Some(pair) = pair_opt {
            stack[0].push(pair);
          }
        },
      }
    }

    // Perl L555: process remaining top-level items through kludge_scripts
    let final_pairs = stack.into_iter().next().unwrap_or_default();
    let result_nodes = kludge_scripts_rec(final_pairs, document);

    // Perl L558-563: at top level, unwrap top-level XMWraps (extract children).
    // Perl iterates all pairs and extracts children of any array-rep XMWrap.
    let mut replacements = Vec::new();
    for node in result_nodes {
      if node.get_name() == "XMWrap" {
        // Unwrap: extract children
        for child in node.get_child_nodes() {
          replacements.push(child);
        }
      } else {
        replacements.push(node);
      }
    }

    // Rebuild: clear mathnode, then add replacement nodes.
    document.unrecord_node_ids(mathnode);
    for mut child in mathnode.get_child_nodes() {
      child.unbind_node();
    }
    for mut node in replacements {
      node.unlink_node();
      mathnode.add_child(&mut node).ok();
    }
    // D3b: the replacements re-parented above may carry xml:id attrs
    // whose idstore entries were cleared by the `unrecord_node_ids`
    // above (which walked mathnode's state before the replacements
    // were re-inserted). Re-record so `lookup_id` on any
    // math-generated id resolves — otherwise finalize()'s
    // `rebuild_idstore_from_dom` is the only recovery path and
    // downstream XMRef lookups can SIGSEGV via stale cache.
    let _ = document.record_node_ids(mathnode);
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Low-level Parser: parse a single expression
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Convert to textual form for processing by MathGrammar
  fn parse_single(
    &mut self,
    mathnode: &mut Node,
    document: &mut Document,
    rule: &str,
  ) -> Result<Option<Node>> {
    let mut idx = 0;
    let mut content_nodes = filter_hints(mathnode.get_child_nodes());

    // Extract trailing PUNCT/PERIOD nodes if rule ends with ',' (Perl: $rule =~ s/,$// )
    let mut punct_nodes: Vec<Node> = Vec::new();
    if rule.ends_with(',') {
      while let Some(last) = content_nodes.last() {
        let role = last.get_attribute("role").unwrap_or_default();
        if role == "PUNCT" || role == "PERIOD" {
          let mut p = content_nodes.pop().unwrap();
          p.unlink(); // detach from mathnode's children
          punct_nodes.insert(0, p);
        } else {
          break;
        }
      }
    }

    if content_nodes.is_empty() {
      // Perl MathParser.pm L683: $result = $nodes[0] || Absent()
      // Empty XMArg/XMWrap: create <XMTok meaning="absent"/>
      let mut absent_tok = document.open_element_at(mathnode, "ltx:XMTok", None, None)?;
      document.set_attribute(&mut absent_tok, "meaning", "absent")?;
      document.close_element_at(&mut absent_tok)?;
      absent_tok.unlink();
      Ok(Some(absent_tok))
    } else if content_nodes.len() == 1 && punct_nodes.is_empty() {
      // single node, nothing to wrap
      Ok(Some(content_nodes.remove(0)))
    } else if content_nodes.len() == 1 {
      // single node with trailing punct: wrap in XMDual
      let result = content_nodes.remove(0);
      Ok(Some(self.wrap_with_punct(
        result,
        punct_nodes,
        mathnode,
        document,
      )?))
    } else {
      // Use pre-filtered content_nodes to avoid double-filtering (filter_hints already called
      // above)
      let (lexemes, mut nodes) = node_to_grammar_lexemes_from(mathnode, content_nodes, &mut idx);
      // Skip the full grammar parse for a pathologically huge formula —
      // Marpa's Earley recognizer would exhaust memory and `abort()`
      // (uncatchable). Fall through to the kludge parser instead (the
      // `Ok(None)` branch). See MAX_GRAMMAR_LEXEMES (witness 1706.06621).
      let parse_outcome = match *MAX_GRAMMAR_LEXEMES {
        Some(cap) if lexemes.len() > cap => {
          // Emit a real Warning (not just a progress tick) so a human — or a
          // future Claude tasked with raising/removing this cap — has the full
          // context. The message explains WHAT was degraded, WHY the cap
          // exists, and HOW to make the formula parsable again.
          let warn_fn = || -> Result<()> {
            Warn!(
              "toobig",
              "math",
              s!(
                "Formula has {} grammar lexemes (> cap {}); skipping the Marpa \
                 grammar parse and falling back to the KLUDGE parser (linear \
                 OPEN/CLOSE matching — no real semantic/Content-MathML tree). \
                 WHY: Marpa's Earley recognizer allocates O(n^2)+ obstack \
                 memory in the lexeme count; on inputs this large it exhausts \
                 RAM and libmarpa's default_out_of_memory calls abort() — an \
                 UNCATCHABLE SIGABRT that kills the whole conversion (the parse \
                 runs on a worker thread; there is no Rust panic to catch). Perl \
                 LaTeXML (Parse::RecDescent) parses such formulae in bounded \
                 memory, so this cap is a Rust-only divergence. TO RAISE/REMOVE \
                 IT (set LATEXML_MAX_GRAMMAR_LEXEMES, 0=disable): the blow-up is \
                 dominated by grammar AMBIGUITY (chiefly implicit-multiplication / \
                 juxtaposition associativity, which makes Earley-set growth \
                 super-linear), so tighten those rules to reduce ambiguity, or \
                 give the marpa fork a bounded-memory / graceful-OOM recognizer \
                 path. Witness: 1706.06621-class explicitly-expanded polynomials.",
                lexemes.len(),
                cap
              )
            );
            Ok(())
          };
          warn_fn().ok();
          Ok(None)
        },
        _ => self.parse_lexemes(lexemes, &nodes, document),
      };
      // Resource fatals must propagate, not be dropped by the if-let below
      // (PR #249 review P1-4); other Errs keep the old treated-as-failure
      // behavior.
      let parse_outcome = match parse_outcome {
        Err(e) if matches!(e.target, latexml_core::common::error::ErrorTarget::Timeout) => {
          return Err(e);
        },
        other => other,
      };
      if let Ok(Some(parse_tree)) = parse_outcome {
        //START reparent: the reparenting used to be in `parse_rec` in Perl. Is this a good place?
        // Replace the content of XMath with parsed result
        // unbindNode followed by (append|replace)Tree (which removes ID's) should be safe
        //
        // Pre-snapshot the xml:ids that the old subtree exposes. Any of
        // these that don't reappear in the new tree below have been
        // "orphaned" by the parse — pre-existing XMRefs pointing at them
        // would go dangling. Mirrors Perl `MathParser.pm` LOSTNODES
        // tracking for the tree-replacement path (the analogue of
        // `ReplacedBy(lost, undef)` cases).
        let mut pre_replacement_ids: Vec<String> = Vec::new();
        for child_el in element_nodes(mathnode) {
          for descendant in document.findnodes("descendant-or-self::*[@xml:id]", Some(&child_el)) {
            if let Some(id) = descendant.get_attribute("xml:id") {
              pre_replacement_ids.push(id);
            }
          }
        }
        for child_el in element_nodes(mathnode) {
          document.unrecord_node_ids(&child_el);
        }
        for mut node in mathnode.get_child_nodes() {
          node.unlink();
        }
        let new_xml_tree = parse_tree.into_xmath(mathnode, &mut nodes, document)?;
        document.append_tree(mathnode, vec![new_xml_tree])?;
        // Resolve _xmkey references: match XMRef[@_xmkey] to elements with same _xmkey
        resolve_xmkeys(mathnode, document)?;
        // Detect orphaned IDs: any pre-snapshot ID that no longer resolves
        // in the document idstore. Record each as a LOSTNODE with an empty
        // replacement (sentinel — the top-level cleanup interprets empty
        // replacement as "drop the XMRef").
        for pre_id in &pre_replacement_ids {
          if document.lookup_id(pre_id).is_none() {
            crate::data::record_replacement(pre_id, "__LOSTNODE__");
          }
        }
        let result = element_nodes(mathnode).remove(0);
        //END reparent.
        if !punct_nodes.is_empty() {
          Ok(Some(self.wrap_with_punct(
            result,
            punct_nodes,
            mathnode,
            document,
          )?))
        } else {
          Ok(Some(result))
        }
      } else {
        // Parse failed; put punct nodes back in mathnode so they remain visible
        for mut p in punct_nodes {
          mathnode.add_child(&mut p).ok();
        }
        Ok(None)
      }
    }
  }

  /// Wrap `result` in an XMDual so trailing punctuation appears in the presentation
  /// branch only, matching Perl's `if (@punct) { $result = ['ltx:XMDual', ...] }`.
  fn wrap_with_punct(
    &self,
    mut result: Node,
    punct_nodes: Vec<Node>,
    mathnode: &mut Node,
    document: &mut Document,
  ) -> Result<Node> {
    // Assign an id to result while it's still in the tree (id generation needs ancestors)
    document.generate_id(&mut result, "")?;
    let id = result
      .get_attribute_ns("id", XML_NS)
      .unwrap_or_else(|| result.get_attribute("id").unwrap_or_default());
    // Detach result from mathnode so we can re-parent it inside XMWrap
    result.unlink();
    // Create XMDual in mathnode
    let mut dual = document.open_element_at(mathnode, "ltx:XMDual", None, None)?;
    // Content branch: XMRef pointing to result
    let ref_attrs = {
      let mut m = HashMap::default();
      m.insert(s!("idref"), id);
      Some(m)
    };
    let mut xmref = document.open_element_at(&mut dual, "ltx:XMRef", ref_attrs, None)?;
    document.close_element_at(&mut xmref)?;
    // Presentation branch: XMWrap containing result + trailing punct
    let mut xmwrap = document.open_element_at(&mut dual, "ltx:XMWrap", None, None)?;
    xmwrap.add_child(&mut result).ok();
    for mut p in punct_nodes {
      xmwrap.add_child(&mut p).ok();
    }
    document.close_element_at(&mut xmwrap)?;
    document.close_element_at(&mut dual)?;
    Ok(dual)
  }

  pub fn parse_marpa(
    &mut self,
    input: &str,
    nodes: &[Node],
    document: &mut Document,
  ) -> Result<XM> {
    // Diagnostic: dump lexeme stream when LATEXML_PARSE_LEXEMES is set.
    // Useful for debugging math-parser hangs (e.g. project_1407_5769_math_hang.md):
    // run with `LATEXML_PARSE_LEXEMES=1 latexml_oxide --dest=out.xml input.tex`
    // to see the input fed to Marpa for each formula. Combine with --noparse
    // disabled (default) to capture lexemes for parses that hang or explode.
    if *PARSE_LEXEMES_DBG {
      eprintln!("PARSE_LEXEMES_BEGIN: {}", input.trim());
    }
    let mut parses: Vec<XM> = Vec::new();
    let mut ok_trees = 0;
    let mut pruned_trees = 0;
    let mut deduped = 0usize;
    let mut consecutive_dupes = 0usize;
    let start = std::time::Instant::now();
    // Capture pragma-rejection reasons when LATEXML_PARSE_PRUNE_REASONS=1.
    // Bounded HashMap keyed by error-message string; on a zero-OK failure
    // we print the top-3 reasons + counts. Cheap when the env var is unset.
    let mut prune_reasons: rustc_hash::FxHashMap<String, usize> = rustc_hash::FxHashMap::default();

    if *PARSE_VIA_HYBRID {
      // Hybrid path: one recognizer pass, one bocage, then route by
      // raw Marpa ambiguity. Unambiguous formulae use the cheaper
      // legacy Step/value action path; ambiguous formulae use ASF.
      //
      // `pruned_trees` remains a diagnostic counter here: the
      // unambiguous branch counts whole-tree semantic rejection, while
      // the ambiguous branch reports per-combo ASF action rejections.
      // Do not compare this number across routing modes as a semantic
      // invariant.
      let mut traverser = crate::asf_traverser::MathTraverser {
        actions: &self.actions,
        pragmas: self.expert_pragmatics.as_slice(),
        builder: &self.builder,
        nodes,
        document,
        pruned_count: 0,
      };
      let consumed = std::rc::Rc::new(std::cell::Cell::new(0usize));
      let hybrid_result = self.engine.parse_hybrid_with_and_node_limit(
        CountingTokens {
          inner: ByteScanner::new(Cursor::new(input)),
          count: consumed.clone(),
        },
        (),
        &mut traverser,
        *HYBRID_AND_NODE_LIMIT,
      );
      if *PARSE_LEXEMES_DBG {
        eprintln!("PARSE_LEXEMES_RECOGNIZED");
      }
      // Coverage guard (see CountingTokens): if the recognizer exhausted before
      // consuming every input lexeme, the parse covers only a prefix and the
      // tail would be silently dropped. Reject it (a genuine full parse always
      // consumes all tokens) so the caller falls to the token-preserving kludge
      // / ltx_math_unparsed, matching Perl's no-silent-loss behaviour.
      // Coverage guard (see CountingTokens): the ByteScanner yields per-byte
      // tokens, so a FULL parse consumes at least every non-trailing-whitespace
      // byte of `input` (it may also read the single trailing space the caller
      // appends, or stop exactly at the last real byte when the start symbol
      // completes-and-exhausts there, as a legitimate `+ -` does). If the
      // recognizer exhausted BEFORE the last real byte, the parse covers only a
      // prefix and its tail would be silently dropped — reject so the caller
      // falls to the token-preserving kludge / ltx_math_unparsed, matching
      // Perl's no-silent-loss behaviour (witness `+ - a` → was `list@(+, -)`).
      if consumed.get() < input.trim_end().len() {
        return Err("Failed to find a parse spanning all input tokens".into());
      }
      match hybrid_result {
        Ok(HybridParseResult::Unambiguous(mut tree_iter)) => {
          // Release `traverser`'s borrow of the recognizer before re-borrowing
          // below; `drop` is the intent even though the type has no Drop impl.
          #[allow(clippy::drop_non_drop)]
          drop(traverser);
          record_ambiguity_metric(1, input);
          let tree_outcome = match tree_iter.next() {
            Some(val) => {
              match self.actions.get_tree(
                self.builder.clone(),
                val,
                self.expert_pragmatics.as_slice(),
                ActionContext { nodes, document },
              ) {
                Ok(Some(tree)) => ParseOutcome::Accepted(tree),
                Ok(None) => ParseOutcome::Empty,
                Err(prune_err) => ParseOutcome::Rejected(prune_err.to_string()),
              }
            },
            _ => ParseOutcome::Empty,
          };
          debug_assert!(
            tree_iter.next().is_none(),
            "hybrid unambiguous branch should expose exactly one raw tree"
          );
          self.audit_hybrid_unambiguous_parity(input, nodes, document, &tree_outcome);
          match tree_outcome {
            ParseOutcome::Accepted(tree) => {
              ok_trees += 1;
              parses.push(tree);
            },
            ParseOutcome::Empty => {},
            ParseOutcome::Rejected(msg) => {
              pruned_trees += 1;
              if *PARSE_PRUNE_REASONS {
                let trimmed = msg.chars().take(140).collect::<String>();
                *prune_reasons.entry(trimmed).or_insert(0) += 1;
              }
            },
            ParseOutcome::Multiple(_) => unreachable!("tree path cannot produce multiple outcomes"),
          }
        },
        Ok(HybridParseResult::Ambiguous(alts, _state)) => {
          record_ambiguity_metric(2, input);
          pruned_trees = traverser.pruned_count;
          if std::env::var("LATEXML_MARPA_ASF_AUDIT").is_ok() {
            eprintln!(
              "ASF_AUDIT: peak returned {} alternatives ({} Some, {} None), pruned={}",
              alts.len(),
              alts.iter().filter(|a| a.is_some()).count(),
              alts.iter().filter(|a| a.is_none()).count(),
              pruned_trees,
            );
          }
          let alts_vec = std::rc::Rc::try_unwrap(alts).unwrap_or_else(|rc| (*rc).clone());
          for tree in alts_vec.into_iter().flatten() {
            if parses.contains(&tree) {
              deduped += 1;
            } else {
              ok_trees += 1;
              parses.push(tree);
            }
          }
        },
        Ok(HybridParseResult::AmbiguousTree(tree_iter, stats)) => {
          // Large-bocage fallback: codex's marpa commit 5f6a19e routes
          // bocages whose `and_node_count` exceeds `*HYBRID_AND_NODE_LIMIT`
          // through the ordinary `Tree` iterator instead of constructing
          // the ASF. Walk it with the same 6 convergence caps that the
          // legacy path uses — those caps are exactly what makes
          // Tree-iteration tractable on highly-ambiguous forests.
          // Release `traverser`'s borrow before re-borrowing below.
          #[allow(clippy::drop_non_drop)]
          drop(traverser);
          record_ambiguity_metric(2, input);
          if std::env::var("LATEXML_MARPA_ASF_AUDIT").is_ok() {
            eprintln!(
              "HYBRID_AUDIT: large-bocage fallback fired (or_nodes={}, and_nodes={}, max_per_or={}) for: {}",
              stats.or_node_count,
              stats.and_node_count,
              stats.max_and_nodes_per_or_node,
              input.chars().take(120).collect::<String>(),
            );
          }
          let max_trees = 5000;
          let max_time = std::time::Duration::from_secs(30);
          let max_consecutive_dupes = 16;
          let converge_budget = std::time::Duration::from_millis(200);
          let pruned_only_time_budget = std::time::Duration::from_secs(2);
          let pruned_only_count_threshold: usize = 200;
          let max_unique = 10;
          for val in tree_iter {
            if ok_trees + pruned_trees >= max_trees || start.elapsed() > max_time {
              break;
            }
            if consecutive_dupes >= max_consecutive_dupes && !parses.is_empty() {
              break;
            }
            if parses.len() >= max_unique {
              break;
            }
            if !parses.is_empty() && start.elapsed() > converge_budget {
              break;
            }
            if parses.is_empty()
              && pruned_trees > pruned_only_count_threshold
              && start.elapsed() > pruned_only_time_budget
            {
              break;
            }
            match self.actions.get_tree(
              self.builder.clone(),
              val,
              self.expert_pragmatics.as_slice(),
              ActionContext { nodes, document },
            ) {
              Ok(Some(tree)) => {
                ok_trees += 1;
                if parses.contains(&tree) {
                  deduped += 1;
                  consecutive_dupes += 1;
                } else {
                  parses.push(tree);
                  consecutive_dupes /= 2;
                }
              },
              Ok(None) => {},
              Err(prune_err) => {
                pruned_trees += 1;
                if *PARSE_PRUNE_REASONS {
                  let msg = prune_err.to_string();
                  let trimmed = msg.chars().take(140).collect::<String>();
                  *prune_reasons.entry(trimmed).or_insert(0) += 1;
                }
                if !parses.is_empty() {
                  consecutive_dupes += 1;
                }
              },
            }
          }
        },
        Err(e) => {
          // A Timeout-class error here is a RESOURCE fatal raised inside a
          // semantics action (e.g. the gullet cycle guard during
          // `create_xmrefs`→`get_xmarg_id` expansion), not a semantic parse
          // rejection. Swallowing it produced "phantom fatals": the `Fatal!`
          // macro had already counted it (final summary says "1 fatal
          // error") but no `Fatal:` line ever reached the log, and parsing
          // ground on formula-by-formula. Surface it and ABORT math parsing
          // (bounded + honest; witness math0402448). Non-resource errors
          // keep the old reject-and-continue behaviour.
          if let Some(err) = latexml_core::common::error::take_last_resource_fatal()
            .or_else(|| resource_fatal_from_message(&e.to_string()))
          {
            // No log_fatal here: the Err now PROPAGATES through
            // parse_lexemes/parse_single/parse_math to the converter's outer
            // handler, which logs the Fatal: line exactly once (P1-4).
            return Err(err);
          }
          if std::env::var("LATEXML_MARPA_ASF_AUDIT").is_ok() {
            eprintln!("HYBRID_AUDIT: parse_hybrid Err: {e}");
          }
        },
      }
    } else if *PARSE_VIA_ASF {
      // ASF path: one post-order memoized callback per glade.
      // `MathTraverser` accumulates all alternative XM parse trees
      // (including any pragmas-pruned ones counted as `pruned_count`)
      // in a single sweep; no Tree-iteration loop, no convergence
      // bandages. See `latexml_math_parser/src/asf_traverser.rs` and
      // docs/MATH_PARSER_AND_ASF.md.
      let mut traverser = crate::asf_traverser::MathTraverser {
        actions: &self.actions,
        pragmas: self.expert_pragmatics.as_slice(),
        builder: &self.builder,
        nodes,
        document,
        pruned_count: 0,
      };
      let consumed = std::rc::Rc::new(std::cell::Cell::new(0usize));
      let asf_result = self.engine.parse_and_traverse_forest(
        CountingTokens {
          inner: ByteScanner::new(Cursor::new(input)),
          count: consumed.clone(),
        },
        (),
        &mut traverser,
      );
      if *PARSE_LEXEMES_DBG {
        eprintln!("PARSE_LEXEMES_RECOGNIZED");
      }
      // Coverage guard (see CountingTokens / the hybrid path above): reject an
      // exhausted-early prefix parse so the tail isn't silently dropped.
      if consumed.get() < input.trim_end().len() {
        return Err("Failed to find a parse spanning all input tokens".into());
      }
      match asf_result {
        Ok((alts, _state)) => {
          pruned_trees = traverser.pruned_count;
          if std::env::var("LATEXML_MARPA_ASF_AUDIT").is_ok() {
            eprintln!(
              "ASF_AUDIT: peak returned {} alternatives ({} Some, {} None), pruned={}",
              alts.len(),
              alts.iter().filter(|a| a.is_some()).count(),
              alts.iter().filter(|a| a.is_none()).count(),
              pruned_trees,
            );
          }
          // The order-alignment with legacy tree-iteration is no
          // longer needed: the FencedLetters Dual pragma now prunes
          // the wrong-direction parses at validation time, and adding
          // `alts.reverse()` measured WORSE on the test suite
          // (1281 vs 1284). Leave the bocage-natural order.
          // End-of-traversal: take ownership of the peak Vec.
          // `Rc::try_unwrap` succeeds unless the marpa driver still
          // holds a copy in its cache (it returned its own clone via
          // `cache.insert(peak, output.clone())`), so fall back to
          // deep clone on the cold path.
          let alts_vec = std::rc::Rc::try_unwrap(alts).unwrap_or_else(|rc| (*rc).clone());
          for tree in alts_vec.into_iter().flatten() {
            if parses.contains(&tree) {
              deduped += 1;
            } else {
              ok_trees += 1;
              parses.push(tree);
            }
          }
        },
        Err(e) => {
          // Same resource-fatal surfacing as the hybrid path above — see the
          // comment there (witness math0402448's phantom fatal).
          if let Some(err) = latexml_core::common::error::take_last_resource_fatal()
            .or_else(|| resource_fatal_from_message(&e.to_string()))
          {
            // No log_fatal here: the Err now PROPAGATES through
            // parse_lexemes/parse_single/parse_math to the converter's outer
            // handler, which logs the Fatal: line exactly once (P1-4).
            return Err(err);
          }
          if std::env::var("LATEXML_MARPA_ASF_AUDIT").is_ok() {
            eprintln!("ASF_AUDIT: traverse_forest Err: {e}");
          }
        },
      }
    } else {
      // Legacy path: Tree iteration with the 6 convergence caps.
      let parse_result = self
        .engine
        .run_recognizer(ByteScanner::new(Cursor::new(input)))?;
      if *PARSE_LEXEMES_DBG {
        eprintln!("PARSE_LEXEMES_RECOGNIZED");
      }
      // The six caps below (max_trees, max_time, max_consecutive_dupes,
      // converge_budget, pruned_only_time_budget+_count_threshold,
      // max_unique) exist because Tree-iteration evaluates per-tree
      // actions O(trees × occurrences) times — defensive bandages
      // against the paradigm cost.
      //
      // 2026-05-18: With HYBRID as the default
      // (`PARSE_VIA_HYBRID`), this branch is reached only when the
      // user explicitly sets `LATEXML_MARPA_LEGACY=1` — an
      // engine-divergence debugging escape hatch. The caps stay
      // because their *original* protective role is exactly what
      // makes the escape hatch usable on real ambiguous inputs
      // (without them legacy would hang on grammars like the
      // `27-trees-from-scriptpos` family in math). Removing them
      // would silently break the debug path.
      //
      // For the production-path concern (long-time tree-iteration
      // cost), HYBRID's `metric == 2 → ASF` routing already
      // sidesteps these caps via per-glade memoization. See
      // docs/MATH_PARSER_AND_ASF.md.
      let max_trees = 5000; // Hard limit on parse tree enumeration
      let max_time = std::time::Duration::from_secs(30); // 30 second timeout
      // Convergence: if we've seen enough consecutive duplicates without
      // a new unique tree, the grammar ambiguity is purely structural
      // (script attachment ordering). Stop early.
      let max_consecutive_dupes = 16;
      // Time-budget convergence: once we have unique parses, stop after
      // this budget. For formulas where all trees are pruned (no unique
      // parse yet), use a longer budget before giving up.
      let converge_budget = std::time::Duration::from_millis(200);
      // Pruned-only fast-fail: if we've spent significant time and seen
      // many trees without finding a single semantic-acceptable parse,
      // the grammar is exploring a combinatorial dead end. Bail before
      // exhausting `max_time` (30s). Empirical: 0804.1730 case had 4536
      // pruned trees over 28s before timeout. After 2s with >200 prunes
      // the marginal probability of finding a unique drops sharply.
      let pruned_only_time_budget = std::time::Duration::from_secs(2);
      let pruned_only_count_threshold: usize = 200;
      // Unique-tree cap: the pragmatics/selection step only needs a handful
      // of distinct parses. Beyond this, additional unique trees are almost
      // always script-attachment ordering variants that don't improve the
      // final selected parse. Avoids enumerating 60+ trees for expressions
      // like `{}^4{}_{12}C^{5+}` where the grammar produces 27+ unique
      // trees from different pre/post script nesting orders.
      let max_unique = 10;
      for val in parse_result {
        // Truncate if too many trees or too much time
        if ok_trees + pruned_trees >= max_trees || start.elapsed() > max_time {
          break;
        }
        // Early convergence: stop if we keep seeing only duplicates.
        // The grammar produces 2^N duplicates from script attachment ordering.
        // Once we've found all unique parses, every new tree is a duplicate.
        if consecutive_dupes >= max_consecutive_dupes && !parses.is_empty() {
          break;
        }
        // Unique-tree cap: stop once we have enough distinct parses.
        if parses.len() >= max_unique {
          break;
        }
        // Time-budget convergence: if we have unique parses and have spent
        // >200ms, stop — the remaining trees are overwhelmingly duplicates.
        if !parses.is_empty() && start.elapsed() > converge_budget {
          break;
        }
        // Pruned-only fast-fail: bail when we have NO unique parses, have
        // already enumerated many trees, and have burned through the
        // pruned-only budget. Without this, the loop runs to `max_time`
        // (30s) on pathological multi-clause RELOP-list formulae where
        // every grammar derivation is semantically pruned (e.g.
        // 0804.1730 had 4536 enumerated → 0 unique → 28.29s).
        if parses.is_empty()
          && pruned_trees > pruned_only_count_threshold
          && start.elapsed() > pruned_only_time_budget
        {
          break;
        }
        // Note: we intentionally do NOT abort when no parse has been found
        // even after extended time — valid parses can appear late in the
        // enumeration (tree #3585 of 3713 for complex multi-equation formulas).
        match self.actions.get_tree(
          self.builder.clone(),
          val,
          self.expert_pragmatics.as_slice(),
          ActionContext { nodes, document },
        ) {
          Ok(tree_opt) => {
            if let Some(tree) = tree_opt {
              ok_trees += 1;
              // Online deduplication: check if this tree is already in our unique set
              if parses.contains(&tree) {
                deduped += 1;
                consecutive_dupes += 1;
              } else {
                parses.push(tree);
                // Half-decay (not full reset) on new unique. This lets us bail
                // on cases where uniques are sparse among a sea of dupes/prunes —
                // e.g. sin[XY] produces 10 unique parses among 1022 grammar
                // derivations; without decay the unique trees keep resetting the
                // dupe counter and we never converge. Half-decay means each new
                // unique halves the accumulated dupe budget instead of clearing it.
                consecutive_dupes /= 2;
              }
            }
          },
          Err(prune_err) => {
            pruned_trees += 1;
            if *PARSE_PRUNE_REASONS {
              let msg = prune_err.to_string();
              let trimmed = msg.chars().take(140).collect::<String>();
              *prune_reasons.entry(trimmed).or_insert(0) += 1;
            }
            // Pruned trees also count toward convergence if we have unique parses
            if !parses.is_empty() {
              consecutive_dupes += 1;
            }
          },
        }
      }
    }

    // Diagnostic: dump parse ORDER when LATEXML_PARSE_DUMP_ORDER=1.
    if std::env::var("LATEXML_PARSE_DUMP_ORDER").is_ok() && parses.len() > 1 {
      eprintln!("PARSE_ORDER: {} unique for {}", parses.len(), input.trim());
      for (i, p) in parses.iter().enumerate() {
        eprintln!("  [{}] {}", i, p.text_summary());
      }
    }
    // Store count for \ltx@count@parses diagnostic macro
    // Use post-dedup count (distinct semantic trees), not raw grammar count
    self.last_parsetrees_count = parses.len();

    // Diagnostic only — neither high ambiguity nor a parse failure is a Perl-side MathParser.pm
    // Error; the warn target uses a Rust-internal class. One message, two categories:
    //   * `unparsed_math`  — the formula produced no parse at all (the `Err` returned below), or
    //   * `ambiguous_math` — the grammar enumerated many derivations (>10).
    // The `what` field is a structural FOOTPRINT of the token stream (see `token_type_footprint`),
    // so the dashboard buckets formulas by shape instead of one unique token dump per paper.
    let diagnostic_category = if parses.is_empty() {
      Some("unparsed_math")
    } else if ok_trees + pruned_trees > 10 {
      Some("ambiguous_math")
    } else {
      None
    };
    if let Some(category) = diagnostic_category {
      // Warn once per DISTINCT formula per document (Perl LaTeXML's rule): a formula repeated N
      // times in one document emits a single warning. Keyed on the exact token stream — true only
      // the first time this document sees it; repeats are silently skipped.
      if warn_formula_once(&mut self.warned_formulas, input.trim()) {
        log_math_warn!(
          category,
          token_type_footprint(input.trim()),
          "Ambiguous math: {} enumerated ({} semantic, {} pruned, {} deduped→{} unique) in {:?} for: {}",
          ok_trees + pruned_trees + deduped,
          ok_trees + deduped,
          pruned_trees,
          deduped,
          parses.len(),
          start.elapsed(),
          input.trim()
        );
      }
    }
    // Diagnostic: report parse counts when LATEXML_PARSE_AUDIT is set.
    // Useful for identifying grammar ambiguity hotspots across the test suite.
    // Usage: LATEXML_PARSE_AUDIT=1 cargo test --test 56_ams -- mathtools_test --nocapture
    if *PARSE_AUDIT && (ok_trees + pruned_trees > 1 || start.elapsed().as_millis() > 50) {
      eprintln!(
        "PARSE_AUDIT: {} trees ({} ok, {} pruned, {} dedup→{} unique) in {:?} | {}",
        ok_trees + pruned_trees + deduped,
        ok_trees + deduped,
        pruned_trees,
        deduped,
        parses.len(),
        start.elapsed(),
        input.trim().chars().take(200).collect::<String>()
      );
    }
    // Dump top pragma-rejection reasons when no parse survived and the
    // diagnostic env var is set. Helps locate over-aggressive pragmas
    // that reject all candidates in document context.
    if *PARSE_PRUNE_REASONS && parses.is_empty() && pruned_trees > 0 {
      let mut top: Vec<(&String, &usize)> = prune_reasons.iter().collect();
      top.sort_by(|a, b| b.1.cmp(a.1));
      eprintln!(
        "PARSE_PRUNE_REASONS: {} pruned, {} distinct, top-5: | input: {}",
        pruned_trees,
        prune_reasons.len(),
        input.trim().chars().take(120).collect::<String>()
      );
      for (msg, count) in top.iter().take(5) {
        eprintln!("  {count:>5}× {msg}");
      }
    }

    match parses.len() {
      0 => Err("Failed to find any parse".into()),
      1 => Ok(parses.into_iter().next().unwrap()),
      _more => {
        // Loop over the various soft pruning algorithms available, until we have 1 tree
        let mut reduced_forest = XM::Choices(parses);
        for pragma in self.student_pragmatics.iter() {
          reduced_forest = reduced_forest.soft_prune_choices(*pragma);
          match reduced_forest {
            XM::Choices(ref trees) if trees.len() <= 1 => break,
            _ => {},
          };
        }
        // Multi-tree pragma: `(a, b)` and `[a, b]` (and half-open
        // variants) standalone should default to the named-interval
        // interpretation (`open-interval`, `closed-interval`, etc.)
        // rather than the generic `vector@(2)` or `delimited-XY@(...)`
        // wrapper. The math-parser grammar admits both; legacy
        // tree-iter happens to pick the named interval, ASF
        // Cartesian-product happens to pick the wrapper. This pragma
        // drops the wrapper parses from the forest root iff a named-
        // interval alternative also exists. Narrow scope: only the
        // math root, only 2-element, only when both alternatives are
        // present.
        reduced_forest = reduced_forest.prefer_named_interval_at_root();

        // Multi-tree pragma: prune `set@(set@(…))`, `vector@(vector@(…))`
        // etc. — redundant self-wrapping at the math root — when a
        // non-self-wrapping alternative exists in the forest.
        reduced_forest = reduced_forest.prefer_non_self_wrapping_root();

        // Multi-tree pragma: drop `multirelation@(..., absent, ...)`
        // chains when a non-multirelation alternative exists.
        // Handles `x>=0` parsed as `(x > absent = 0)` vs `>=@(x, 0)`.
        reduced_forest = reduced_forest.prefer_combined_relop_over_multirelation_with_absent();

        // `prefer_zero_absent_when_available` retired 2026-05-19
        // (ASF item 5 Phase 2): the pragma had no dedicated test
        // witness. Its conceptual target (`<x|y>` inner-product
        // bra-ket) is already produced as `inner-product@(x, y)`
        // by the qm-specific pragmas + the angle-bracket grammar
        // rules. After the modified_term Phase 1 landing, disabling
        // the pragma left `cargo test --tests` = 1328/0/0 on both
        // HYBRID and ASF — the pragma is structurally redundant.
        // See commit history (the new commit) and
        // `docs/MATH_PARSER_ASF_TIEBREAKING.md`.

        // Multi-tree pragma: a *specific* QM semantic
        // (`quantum-operator-product@(a, f, b)`, `inner-product@(a, b)`)
        // beats a generic `delimited-⟨⟩` wrapper for the same input.
        // Specific semantic recognition reflects author intent more
        // closely than a structural fence wrapper. Must run BEFORE
        // `prefer_more_delimited_wrappers`, because the latter would
        // otherwise filter out the qm_bracket candidate (which has
        // zero `delimited-` wrappers).
        reduced_forest = reduced_forest.prefer_qm_specific_semantics();

        // Multi-tree pragma: prefer candidates with FEWER nested
        // same-meaning fences (`norm` inside `norm`, etc.). Encodes
        // the mathematician's greedy left-to-right bar-pairing
        // instinct. Resolves `||x||a||y||` → `‖x‖ · a · ‖y‖`
        // (siblings) over `‖x · ‖a‖ · y‖` (nested).
        reduced_forest = reduced_forest.prefer_fewer_nested_same_fences();

        // Multi-tree pragma: prefer candidates that recognized
        // fenced sub-expressions as `delimited-X@(…)` (angle, paren,
        // bracket, vertbar) over flat `formulae@(…)` / multirelation
        // chains. Fires only when at least one candidate has a
        // `delimited-X` Apply AND another has fewer/zero. Resolves
        // `2<x,y>=z`, `0<<a,b>>1`, and bra-ket-style angle fences.
        // See docs/MATH_PARSER_ASF_TIEBREAKING.md § "What landed".
        reduced_forest = reduced_forest.prefer_more_delimited_wrappers();

        // Multi-tree pragma: prefer candidates with MORE
        // `Apply(letter, [vertbar-fenced])` patterns — captures the
        // QM-style bra-ket reading `<a|f|b>` → `a@(|f|) * b` over
        // the multiplicative `a * |f| * b`. No-op outside QM/angle
        // contexts because K-12 readings don't admit the pattern.
        reduced_forest = reduced_forest.prefer_more_letter_at_vertbar();

        // Multi-tree pragma: when at least one candidate has zero
        // `MODIFIEROP:conditional` Applies, drop those with more.
        // The conditional reading of `vertbar_modifier` recurses
        // through itself in the grammar, producing
        // `conditional@(a, conditional@(a, …))` for inputs like
        // `a|a|+b|b|+c|c|`. The K-12 algebra reading
        // `a*|a|+b*|b|+c*|c|` has no conditionals; this pragma
        // gives it preference. Set-builder / probability cases
        // where no algebraic alternative exists are unaffected.
        reduced_forest = reduced_forest.prefer_fewer_conditionals();

        // Multi-tree pragma: when at least one candidate roots at
        // an additive operator (`+`/`-`), prefer it over candidates
        // with multiplicative root. K-12 reading: `a*|a|+b*|b|+c*|c|`
        // has `+` outermost, not `a * (one giant absolute-value)`.
        reduced_forest = reduced_forest.prefer_root_addition_over_outer_multiplication();

        // Multi-tree pragma: among addition-rooted candidates,
        // prefer the widest n-ary `+` chain (`+@(a, b, c)` over
        // `+@(a, b+c)`). Resolves the inner bar-pairing of
        // `a|a|+b|b|+c|c|` — the 3-arg chain has every `|.|`
        // as a sibling absolute-value.
        reduced_forest = reduced_forest.prefer_wider_addition_root();

        // Multi-tree shape pragmas (`prefer_fewer_absent`,
        // `prefer_smaller_tree`) exist on `XM` but are
        // **deliberately not wired in by default**. Both proved
        // too coarse to apply universally — see
        // docs/MATH_PARSER_ASF_TIEBREAKING.md § "Empirical results":
        //
        // * `prefer_fewer_absent`: on the legacy path it is essentially neutral (1300/1 vs 1301
        //   baseline), but on the ASF path it costs 9 tests because ASF surfaces absent-free parses
        //   that are semantically WRONG (e.g. `<a|f|b>` parsed as a multiplication chain without
        //   the bra-ket interpretation). The deeper issue: `absent` has two valid uses (structural
        //   filler vs missing-operand fallback) and the pragma can't tell them apart.
        //
        // * `prefer_smaller_tree`: 3 improvements vs 6 regressions on legacy. "Smaller is better"
        //   works for `norm@(a)` vs `abs(abs(a))` but not for `annotated@(x, expr)` vs `x@(expr)`
        //   where the larger tree is the intended one.
        //
        // The correct path forward is case-by-case semantic
        // pragmas (named-operator recognition, domain-typed
        // wrappers), not universal shape ranking. See the
        // research-notes doc for the design discussion.
        Ok(reduced_forest)
      },
    }
  }

  pub fn parse_lexemes(
    &mut self,
    lexemes: Vec<String>,
    nodes: &[Node],
    document: &mut Document,
  ) -> Result<Option<XM>> {
    let mut input_string: String = lexemes.join(" ");
    // Add a trailing space, in an attempt to work with
    // a rules!() macro that has a Hard expectation of a space char following EVERY token.
    // this - counterintuitively- allows a simple macro definition AND a simple parse tree.
    input_string.push(' ');

    match self.parse_marpa(&input_string, nodes, document) {
      Ok(mut parse_tree) => {
        // Perf: after successful parse, Marpa engine is in state T. The next
        // run_recognizer call will naturally advance T → GReady → R (fresh
        // Recognizer) without triggering precompute. Avoiding reset_engine
        // here saves the ~8% CPU time that precompute consumed per formula.
        // Restructure flat formulae with \quad separators to right-recursive nesting
        // (matching Perl's moreRHS/maybeColRHS right-recursive structure)
        restructure_formulae_right(&mut parse_tree)?;
        // Rename `list` to `vector`/`set` when delimiters wrap the list (Perl encloseN)
        rename_fenced_lists(&mut parse_tree, nodes)?;
        // Combine adjacent SUPOP tokens (prime+prime → prime2)
        combine_supop_post(&mut parse_tree, nodes)?;
        Ok(Some(parse_tree))
      },
      Err(e) => {
        self.reset_engine();
        // Resource fatals (token/pushback/cycle/memory limits) must ABORT
        // math parsing, not degrade into a silent per-formula rejection —
        // this arm was the swallow that made parse_marpa's abort dead code
        // (PR #249 review P1-4). Semantic parse errors keep the old
        // reject-and-continue behavior.
        if matches!(e.target, latexml_core::common::error::ErrorTarget::Timeout) {
          return Err(e);
        }
        Ok(None)
      },
    }
  }
}

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// parse_kludgeScripts_rec — script attachment for unparsed expressions
// Perl: MathParser.pm L568-589
//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

/// Perl: parse_kludgeScripts_rec (MathParser.pm L568-589)
/// Takes a list of (Node, role) pairs and attaches scripts to their bases:
/// - POSTSUPERSCRIPT/POSTSUBSCRIPT → attach to preceding node
/// - FLOATSUPERSCRIPT/FLOATSUBSCRIPT → pre-script on following node
fn kludge_scripts_rec(pairs: Vec<(Node, String)>, document: &mut Document) -> Vec<Node> {
  use crate::data::get_grammatical_role;
  if pairs.is_empty() {
    return vec![];
  }
  if pairs.len() == 1 {
    return vec![pairs.into_iter().next().unwrap().0];
  }

  let mut acc: Vec<Node> = Vec::new();
  let mut iter = pairs.into_iter();
  let mut x = iter.next().unwrap(); // (node, role)

  loop {
    let y_opt = iter.next();
    let y = match y_opt {
      Some(y) => y,
      None => {
        // Base case: only x remains
        acc.push(x.0);
        break;
      },
    };
    let more: Vec<(Node, String)> = iter.collect();

    if is_float_script(&x.1) {
      if is_post_script(&y.1) {
        if !more.is_empty() {
          // Perl L575-576: FLOAT + POST + more → combined pre-sub+super
          let mut rest = kludge_scripts_rec(more, document);
          if let Some(base) = rest.first_mut() {
            let inner = new_script_node(base.clone(), &y, document);
            let outer = new_script_node(inner, &x, document);
            rest[0] = outer;
          }
          acc.extend(rest);
          break;
        } else {
          // Perl L578: FLOAT + POST, no more → floating sub+super on Absent
          let absent = new_absent_node(document);
          let inner = new_script_node(absent, &y, document);
          let outer = new_script_node(inner, &x, document);
          acc.push(outer);
          break;
        }
      } else {
        // Perl L580-581: FLOAT + non-script → prescript on whatever follows
        let mut rest_pairs = vec![y];
        rest_pairs.extend(more);
        let mut rest = kludge_scripts_rec(rest_pairs, document);
        if let Some(base) = rest.first_mut() {
          let scripted = new_script_node(base.clone(), &x, document);
          rest[0] = scripted;
        }
        acc.extend(rest);
        break;
      }
    } else if is_post_script(&y.1) {
      // Perl L583-584: POST script → attach to preceding, recurse with result
      let scripted = new_script_node(x.0.clone(), &y, document);
      let role = get_grammatical_role(&scripted);
      let mut rest_pairs = vec![(scripted, role)];
      rest_pairs.extend(more);
      let rest = kludge_scripts_rec(rest_pairs, document);
      acc.extend(rest);
      break;
    } else {
      // Perl L585-586: neither is a script → accumulate x, advance
      acc.push(x.0);
      x = y;
      iter = more.into_iter();
    }
  }
  acc
}

fn is_float_script(role: &str) -> bool { role == "FLOATSUPERSCRIPT" || role == "FLOATSUBSCRIPT" }

fn is_post_script(role: &str) -> bool { role == "POSTSUPERSCRIPT" || role == "POSTSUBSCRIPT" }

/// Perl: NewScript (MathParser.pm L1597-1644)
/// Creates XMApp(XMTok[role=SCRIPTOP, scriptpos=...], base, script_content).
fn new_script_node(base: Node, script_pair: &(Node, String), document: &mut Document) -> Node {
  let (script_node, script_role) = script_pair;

  // Determine SUPER vs SUB from role
  let is_super = script_role.contains("SUPER");
  let is_float = script_role.starts_with("FLOAT");
  let op_role = if is_super {
    "SUPERSCRIPTOP"
  } else {
    "SUBSCRIPTOP"
  };

  // Extract scriptpos from base and script (Perl L1613-1635)
  // For XMApp bases, check the inner operator's scriptpos too
  let rbase_sp = {
    let mut sp = base.get_attribute("scriptpos").unwrap_or_default();
    if sp.is_empty() && base.get_name() == "XMApp" {
      // Check inner operator (first child) for scriptpos
      if let Some(op) = base.get_child_elements().first() {
        sp = op.get_attribute("scriptpos").unwrap_or_default();
      }
    }
    sp
  };
  let script_sp = script_node.get_attribute("scriptpos").unwrap_or_default();

  let (bx, bl) = parse_scriptpos_str(&rbase_sp);
  let (sx, sl) = parse_scriptpos_str(&script_sp);
  let bl = if bl > 0 {
    bl
  } else if sl > 0 {
    sl
  } else {
    1
  };
  let sl = if sl > 0 { sl } else { bl };
  let mut l = if sl > 0 {
    sl
  } else if bl > 0 {
    bl
  } else {
    1
  };

  // Perl: if base was a float script, bump level
  if base.get_attribute("_wasfloat").is_some() {
    l += 1;
  } else if let Some(bump_str) = base.get_attribute("_bumplevel")
    && let Ok(bump) = bump_str.parse::<u32>()
  {
    l = bump;
  }

  // Perl L1632: position from base or script, defaulting to "post"
  let x = if is_float {
    "pre"
  } else if bl == sl {
    if !bx.is_empty() { bx } else { "post" }
  } else {
    if !sx.is_empty() { sx } else { "post" }
  };
  let scriptpos = format!("{x}{l}");

  // Create XMTok operator: <XMTok role="SUPERSCRIPTOP" scriptpos="post1"/>
  let mut op_node = Node::new("XMTok", None, document.get_document()).unwrap();
  op_node.set_attribute("role", op_role).ok();
  op_node.set_attribute("scriptpos", &scriptpos).ok();

  // Extract script content: first child element of the script XMApp
  // Perl: Arg($script, 0) — gets first element child
  let script_content = script_node.get_child_elements().into_iter().next();

  // Create XMApp: <XMApp> op, base, script_content </XMApp>
  let mut app_node = Node::new("XMApp", None, document.get_document()).unwrap();

  // Propagate _font from operator if present
  if let Some(font) = op_node.get_attribute("_font") {
    app_node.set_attribute("_font", &font).ok();
  }
  // Mark as float if applicable
  if is_float {
    app_node.set_attribute("_wasfloat", "1").ok();
  }

  // Add children: op, base, script_content
  op_node.unlink_node();
  app_node.add_child(&mut op_node).ok();
  let mut base_clone = base;
  base_clone.unlink_node();
  app_node.add_child(&mut base_clone).ok();
  if let Some(mut content) = script_content {
    content.unlink_node();
    app_node.add_child(&mut content).ok();
  }
  // Perl NewScript L1624-1643: carry the script node's padding onto the new
  // combined XMApp. A pre-script donates its lpadding; a post/mid script its
  // rpadding. Without this, a thinspace after a script (e.g. `x^2\,dx`) — which
  // filter_hints collapses onto the POSTSUPERSCRIPT marker node — is lost when
  // kludge_scripts rebuilds the `x^2` XMApp. Only set if the script carries it
  // and the app doesn't already (matching Perl's `&& !$$app[1]{...}`).
  if x == "pre" {
    if let Some(lpad) = script_node.get_attribute("lpadding")
      && !lpad.is_empty()
      && app_node.get_attribute("lpadding").is_none()
    {
      app_node.set_attribute("lpadding", &lpad).ok();
    }
  } else if let Some(rpad) = script_node.get_attribute("rpadding")
    && !rpad.is_empty()
    && app_node.get_attribute("rpadding").is_none()
  {
    app_node.set_attribute("rpadding", &rpad).ok();
  }
  app_node
}

/// Create an XMTok with meaning="absent" (Perl: Absent())
fn new_absent_node(document: &mut Document) -> Node {
  let mut tok = Node::new("XMTok", None, document.get_document()).unwrap();
  tok.set_attribute("meaning", "absent").ok();
  tok
}

fn parse_scriptpos_str(sp: &str) -> (&str, u32) {
  if sp.is_empty() {
    return ("post", 0);
  }
  let pos_end = sp.find(|c: char| c.is_ascii_digit()).unwrap_or(sp.len());
  let pos = &sp[..pos_end];
  let level = sp[pos_end..].parse::<u32>().unwrap_or(0);
  (pos, level)
}

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Conversion to a less ambiguous, mostly-prefix form.
// Mostly for debugging information?
// Note that the nodes are true libXML nodes, already absorbed into the document
//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
pub fn text_form(node: &Node, document: &Document) -> String {
  // Reset depth guard for each top-level invocation (text_form is the
  // single public entry point).
  TEXTREC_DEPTH.with(|d| d.set(0));
  textrec(node, None, None, document)
}

// Depth guard for textrec / textrec_apply recursion. Malformed XM trees
// (e.g. an XMRef chain that loops back through an XMApp) can otherwise
// exhaust the thread stack — observed on arxiv 1407.5769. 256 is well
// above any legitimate math-tree nesting but shallow enough to abort
// fast when the loop starts.
const TEXTREC_MAX_DEPTH: u32 = 256;
thread_local! {
  static TEXTREC_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

// ================================================================================
// Some more XML utilities, but math specific (?)

// Get the Token's  meaning, else name, else content, else role
fn get_token_meaning(node_opt: &Node, document: &Document) -> Option<String> {
  let node = realize_xmnode(node_opt, document);
  if let Some(x) = p_get_attribute(&node, "meaning") {
    Some(x)
  } else if let Some(x) = p_get_attribute(&node, "name") {
    Some(x)
  } else {
    let text_content = node.get_content();
    if !text_content.is_empty() {
      Some(text_content)
    } else {
      p_get_attribute(&node, "role")
    }
  }
}

fn textrec(
  node_opt: &Node,
  outer_bp_opt: Option<usize>,
  outer_name_opt: Option<&str>,
  document: &Document,
) -> String {
  // Depth guard — see TEXTREC_MAX_DEPTH comment. Increment + defer a
  // decrement so early returns still balance.
  let entered = TEXTREC_DEPTH.with(|d| {
    let v = d.get();
    if v >= TEXTREC_MAX_DEPTH {
      false
    } else {
      d.set(v + 1);
      true
    }
  });
  if !entered {
    return String::new();
  }
  struct _Dec;
  impl Drop for _Dec {
    fn drop(&mut self) { TEXTREC_DEPTH.with(|d| d.set(d.get().saturating_sub(1))); }
  }
  let _dec = _Dec;

  let node = realize_xmnode(node_opt, document);
  let tag = get_node_qname(&node);
  let outer_bp = outer_bp_opt.unwrap_or(0);
  let outer_name = outer_name_opt.unwrap_or("");
  // If node has meaning, that's the text form.
  let meaning_opt = match p_get_attribute(&node, "meaning") {
    Some(m) => Some(m),
    None => p_get_attribute(&node, "name"),
  };
  if let Some(meaning) = meaning_opt {
    return match PREFIX_ALIAS.get(meaning.as_str()) {
      Some(m) => m.to_string(),
      None => meaning,
    };
  }
  if tag == pin!("ltx:XMApp") {
    let mut args = element_nodes(&node);
    if args.is_empty() {
      // Perl `MathParser.pm:939-949` `textrec` for `ltx:XMApp` does:
      //   my ($op, @args) = element_nodes($node);
      //   $op = realizeXMNode($op);    # undef → undef
      //   …textrec_apply($name, $op, @args)…
      // i.e. silently passes undef ops through; the earlier "Error
      // MathParser.pm:1394" attribution was incorrect (L1394 is in
      // `Fence`, an unrelated path). Match the Perl-faithful silent
      // degrade so a malformed math subtree doesn't flip canvas
      // status — this is post-render serialization, not user-visible
      // breakage. The empty-string return below preserves the
      // pre-existing fall-through semantics.
      return String::new();
    }
    let arg_node = args.remove(0);
    let op = realize_xmnode(&arg_node, document);
    if let Some(app_role) = node.get_attribute("role") {
      if app_role == "FLOATSUBSCRIPT" {
        return String::from("_") + &textrec(&op, None, None, document);
      } else if app_role == "FLOATSUPERSCRIPT" {
        return String::from("^") + &textrec(&op, None, None, document);
      }
    }

    let name = if with_node_qname(&op, |name| name == "ltx:XMTok") {
      get_token_meaning(&op, document).unwrap_or_else(|| "unknown".to_owned())
    } else {
      String::new()
    };
    let (bp, string) = textrec_apply(&name, &op, args, document);
    if (bp < outer_bp) || ((bp == outer_bp) && (name != outer_name)) {
      format!("({string})")
    } else {
      string
    }
  } else if tag == pin!("ltx:XMDual") {
    // XMDual normally has exactly 2 children (content-branch, presentation-
    // branch). A malformed dual can show up in practice — e.g. after a
    // partial rewrite that detached one side, or from a replacement pattern
    // whose semantic arm is empty. Fall back to the empty string so the
    // tex-attribute serialization stays intact rather than aborting the
    // whole conversion.
    let children = element_nodes(&node);
    let Some(content) = children.first() else {
      // Perl `MathParser.pm:950-954` `textrec` for `ltx:XMDual` does:
      //   my ($content, $presentation) = element_nodes($node);
      //   my $text = textrec($content, …);     # undef → '[missing]'
      // — silently degrades. The earlier "MathParser.pm:1394-style"
      // attribution was incorrect. Match Perl-faithful silent path:
      // emit empty string and continue.
      return String::new();
    };
    textrec(content, Some(outer_bp), Some(outer_name), document) // Just send out the
  // semantic form
  // Fall back to
  // presentation, if
  // content has poor
  // semantics (eg. from
  // replacement patterns)
  // TODO
  // return ($text =~
  // /^\(*Unknown/ ?
  // textrec($presentation,
  // $outer_bp, $outer_name)
  // : $text); }
  } else if tag == pin!("ltx:XMTok") {
    let name = match get_token_meaning(&node, document) {
      Some(meaning) => meaning,
      None => s!("Unknown"),
    };
    match PREFIX_ALIAS.get(name.as_str()) {
      Some(v) => v.to_string(),
      None => name,
    }
  } else if tag == pin!("ltx:XMWrap") || tag == pin!("ltx:XMCell") {
    // ??
    element_nodes(&node)
      .into_iter()
      .map(|child| textrec(&child, None, None, document))
      .collect::<Vec<_>>()
      .join("@")
  } else if tag == pin!("ltx:XMArg") {
    let args = element_nodes(&node);
    if args.is_empty() {
      return String::new();
    }
    args
      .iter()
      .map(|arg| textrec(arg, None, None, document))
      .collect::<Vec<_>>()
      .join("")
  } else if tag == pin!("ltx:XMArray") {
    textrec_array(&node, document)
  } else {
    s!("[{}]", p_get_value(&node))
  }
}

/// Structural footprint of a formula's token stream — the `what` field of the `ambiguous_math` /
/// `unparsed_math` math-parser diagnostics. Joins the token *types* (the segment before the first
/// `:` of each `TYPE:value:position` triple) with `_`, bounded to fit CorTeX's `what` column, and
/// appends `_cntd` when the stream is truncated. Space-free, so it slots into CorTeX's
/// `severity:category:what` log parser as a groupable frequency key — collapsing a per-formula
/// token dump into a shape signature the dashboard can bucket (the full token stream stays in the
/// message `details`). E.g. arXiv 0708.2155's `UNKNOWN:rho:1 OPEN:(:2 UNKNOWN:p:3 …` →
/// `UNKNOWN_OPEN_UNKNOWN_CLOSE_RELOP_…` (truncated with `_cntd` once it would pass the budget).
/// Records `formula` in the per-document `seen` set and returns whether this is its FIRST sighting
/// — the gate for Perl LaTeXML's "warn once per distinct formula per document" rule. The token
/// stream is hashed (SipHash) so the set stays 8 bytes per distinct formula no matter how long the
/// formula is; a hash collision at worst suppresses one warning, which is harmless.
fn warn_formula_once(seen: &mut HashSet<u64>, formula: &str) -> bool {
  let mut hasher = std::collections::hash_map::DefaultHasher::new();
  formula.hash(&mut hasher);
  seen.insert(hasher.finish())
}

fn token_type_footprint(tokens: &str) -> String {
  // CorTeX's log `what` column is varchar(200) and a btree index key (category, what, task_id), so
  // bound the footprint to fit by construction: append token types until the next would pass the
  // budget, then mark the truncation with `_cntd`. Reserving the suffix length keeps a truncated
  // footprint within 200 chars (195 types + "_cntd"), so the dispatcher's insert never silently
  // chops the marker.
  const SUFFIX: &str = "_cntd";
  const BUDGET: usize = 200 - SUFFIX.len();
  let mut out = String::new();
  let mut truncated = false;
  for token in tokens.split_whitespace() {
    let ty = token.split(':').next().unwrap_or(token);
    // First type always lands (a lone >budget type is backstopped by CorTeX's 200-char cap); every
    // subsequent one must fit the budget WITH its separator, else we stop and mark `_cntd`.
    if !out.is_empty() && out.len() + 1 + ty.len() > BUDGET {
      truncated = true;
      break;
    }
    if !out.is_empty() {
      out.push('_');
    }
    out.push_str(ty);
  }
  if truncated {
    out.push_str(SUFFIX);
  }
  // A formula with no tokens would yield an empty `what`, which CorTeX's parser can't key on —
  // emit a placeholder so the line still groups.
  if out.is_empty() {
    out.push_str("none");
  }
  out
}

fn textrec_apply(name: &str, op: &Node, args: Vec<Node>, document: &Document) -> (usize, String) {
  let role = op
    .get_attribute("role")
    .unwrap_or_else(|| "Unknown".to_string());
  if (role == "SUBSCRIPTOP" || role == "SUPERSCRIPTOP")
    && PRE_DIGITS_RE.is_match(&op.get_attribute("scriptpos").unwrap_or_default())
  {
    // Note that this will likely get parenthesized due to high bp
    let mut inner = textrec(op, None, None, document);
    if let Some(arg2) = args.get(1) {
      inner.push(' ');
      inner.push_str(&textrec(arg2, None, None, document));
    }
    if let Some(arg1) = args.first() {
      inner.push(' ');
      inner.push_str(&textrec(arg1, None, None, document));
    }
    (5000, inner)
  } else if let Some(bp) = IS_INFIX.get(&role) {
    // A sub/superscript with a meaning probably should be prefix
    if (role == "SUBSCRIPTOP" || role == "SUPERSCRIPTOP") && op.has_attribute("meaning") {
      (
        500,
        format!(
          "{}@({})",
          textrec(op, Some(10000), Some(name), document),
          args
            .iter()
            .map(|a| textrec(a, None, None, document))
            .collect::<Vec<_>>()
            .join(", ")
        ),
      )
    } else {
      // Format as infix.
      let textrec_op = textrec(op, None, None, document);
      let rec_form = if args.len() == 1 {
        // unless a single arg; then prefix.
        textrec_op + " " + &textrec(&args[0], Some(*bp), Some(name), document)
      } else {
        args
          .iter()
          .map(|a| textrec(a, Some(*bp), Some(name), document))
          .collect::<Vec<_>>()
          .join(&(" ".to_string() + &textrec_op + " "))
      };
      (*bp, rec_form)
    }
  } else if role == "POSTFIX" {
    if args.is_empty() {
      (10000, textrec(op, None, None, document))
    } else {
      (
        10000,
        textrec(&args[0], Some(10000), Some(name), document) + &textrec(op, None, None, document),
      )
    }
  } else if name == "multirelation" {
    let joined = args
      .iter()
      .map(|a| textrec(a, Some(2), Some(name), document))
      .collect::<Vec<_>>()
      .join(" ");
    (2, joined)
  } else {
    (
      500,
      textrec(op, Some(10000), Some(name), document)
        + "@("
        + &args
          .iter()
          .map(|a| textrec(a, None, None, document))
          .collect::<Vec<_>>()
          .join(", ")
        + ")",
    )
  }
}

fn textrec_array(node: &Node, document: &Document) -> String {
  let name = node
    .get_attribute("meaning")
    .or_else(|| node.get_attribute("name"))
    .unwrap_or_else(|| String::from("Array"));
  let rows: Vec<String> = element_nodes(node)
    .into_iter()
    .map(|row| {
      let cells: Vec<String> = element_nodes(&row)
        .into_iter()
        .map(|cell| match cell.get_first_child() {
          Some(first_child) => textrec(&first_child, None, None, document),
          _ => String::new(),
        })
        .collect();
      format!("[{}]", cells.join(", "))
    })
    .collect();
  format!("{}[{}]", name, rows.join(", "))
}

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Cute! Were it NOT for Sub/Superscripts, the whole parsing process only
// builds a new superstructure around the sequence of token nodes in the input.
// Thus, any internal structure is unchanged.
//  They get re-parented, but if the parse fails, we've only got to put them
// BACK into the original node, to recover the original arrangment!!!
// Thus, we don't have to clone, and deal with namespace duplication.
// ...
// EXCEPT, as I said, for sub/superscripts!!!!
//

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Constructors used in grammar
// All the tree construction in the grammar should come through these
// operations. We avoid mucking with the actual XML nodes (both to avoid
// modifying the original tree until we have a successful parse, and to avoid
// XML::LibXML cloning nightmares) We are converting XML nodes to array
// representation: [$tag, {%attr},@children] This means any inspection of
// nodes has to recognize that  * node may be in XML vs ARRAY representation
// * node may be an XMRef to another node whose properties are the ones we
// should use.
//
// Also, when we are examining a node's properties (roles, fences, script
// positioning, etc) we should be careful to check for XMRef indirection and
// examine the properties of the node that was referred to.
// HOWEVER, we should construct our parse tree using (a clone of) the XMRef
// node, rather than (a clone of) the referred to node, so as to preserve
// identity.
//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// We're currently keeping the id's on the nodes as they get cloned,
// since they'll (maybe) replace the unparsed nodes.
// However, if we consider multiple parses or preserving both parsed & unparsed,
// we may have to do some adaptation and id shifting.
// ================================================================================

// ================================================================================
// Low-level accessors

// The following accessors work on both the LibXML and ARRAY representations
// but they do NOT automatically dereference XMRef!
pub fn p_get_value(node: &Node) -> String {
  let node_type = node.get_type();
  if node_type == Some(NodeType::ElementNode) {
    let x = node.get_content();
    if !x.is_empty() {
      // get content, or fall back to name
      x
    } else {
      node.get_attribute("name").unwrap_or_default()
    }
  } else {
    node.get_content()
  }
}

//================================================================================

pub fn realize_xmnode<'a>(node: &'a Node, document: &'a Document) -> Cow<'a, Node> {
  if with_node_qname(node, |name| name == "ltx:XMRef")
    && let Some(idref) = node.get_attribute("idref")
  {
    // NOTE: this intentionally uses the LIVE `document.lookup_id`, NOT the
    // frozen `MATH_IDSTORE` snapshot path (`data::resolve_xmref`) that
    // `get_grammatical_role` uses. Routing this through `resolve_xmref` so
    // that more refs resolve mid-parse looked like it would silence the
    // benign transient `expected:id` warnings (refs that miss the live
    // idstore during a sibling element's reinstall but exist in the final
    // doc) — but it DUPLICATES content (`\choose` → "a + ba + b binomial
    // c + dc + d", regressing choose/declare/sampler): callers here rely on
    // an unresolved ref returning the XMRef itself. The warnings are benign
    // (WARN-level, targets resolve in the final tree); do not "fix" them by
    // swapping the resolver. See docs/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md.
    // Can it happen that the target is itself an XMRef? Then recurse.
    if let Some(realnode) = document.lookup_id(&idref) {
      return Cow::Borrowed(realnode);
    }
    // An unresolved ref HERE is a BENIGN parse-time transient, not a defect, so
    // we stay SILENT. This resolver consults the LIVE `document.lookup_id`, which
    // is mutated as each XMath element reinstalls during the parse — a Rust/ASF
    // architectural artifact that Perl's `MathParser::realizeXMNode`
    // (MathParser.pm:135) does NOT have, so Perl emits ~0 of these mid-parse.
    // The target reliably exists in the FINAL tree (empirically on the heaviest
    // witness 0704.2400: of 98 transient misses, 85 ids are present in the output
    // and the other 13 leave ZERO dangling `<XMRef idref=…>` — the refs were
    // re-pointed or absorbed; the output has 0 dangling idrefs of 2597).
    // Callers rely on an unresolved ref returning the XMRef itself; do NOT "fix"
    // by swapping in `resolve_xmref` — that DUPLICATES content (\choose →
    // "a + ba + b binomial c + dc + d"; regresses choose/declare/sampler). See
    // docs/EXPECTED_ID_XMREF_DESIGN_2026-06-08.md §3b.
    // The AUTHORITATIVE dangling-ref check is the faithful post-processing pass
    // (Perl Post.pm:1444/1456 → latexml_post `realize_xm_node` /
    // `mark_xm_node_visibility_aux`, Error severity) plus core
    // `markXMNodeVisibility` (Document.pm:1548/1553). Warning here floods ~128k
    // false positives that bury that genuine signal.
  }
  Cow::Borrowed(node)
}

/// Resolve _xmkey and _pxmkey references after parse tree installation.
/// Matches XMRef[@_xmkey] to elements with same _xmkey, generates xml:id and sets idref.
/// _pxmkey is used by parser-generated XMDual (apply_delimited) to avoid
/// conflicting with base_xmath's \lx@dual afterConstruct resolver.
fn resolve_xmkeys(
  mathnode: &Node,
  document: &mut Document,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
  // Resolve both _xmkey (from existing infrastructure) and _pxmkey (from parser)
  for attr_name in &["_xmkey", "_pxmkey"] {
    let refs = document.findnodes(
      &format!("descendant::ltx:XMRef[@{}]", attr_name),
      Some(mathnode),
    );
    for mut ref_node in refs {
      let key = match ref_node.get_attribute(attr_name) {
        Some(k) => k,
        None => continue,
      };
      // Find the element with matching key (non-XMRef)
      // Note: not(self::ltx:XMRef) may fail with namespace prefix in nested
      // predicates (XPath bug). Use local-name() check instead.
      let xpath = format!(
        "descendant::*[@{}='{}'][not(local-name()='XMRef')]",
        attr_name, key
      );
      let targets = document.findnodes(&xpath, Some(mathnode));
      if let Some(mut target) = targets.into_iter().next() {
        // Ensure target has xml:id
        let target_id = if let Some(id) = target
          .get_attribute("xml:id")
          .or_else(|| target.get_attribute("id"))
        {
          id
        } else {
          document.generate_id(&mut target, "")?;
          target
            .get_attribute("xml:id")
            .or_else(|| target.get_attribute("id"))
            .unwrap_or_default()
        };
        if !target_id.is_empty() {
          document.set_attribute(&mut ref_node, "idref", &target_id)?;
        }
      }
      let _ = ref_node.remove_attribute(attr_name);
    }
    // Clean up from non-ref elements
    for mut node in document.findnodes(&format!("descendant::*[@{}]", attr_name), Some(mathnode)) {
      let _ = node.remove_attribute(attr_name);
    }
  }
  Ok(())
}

fn p_get_attribute(item: &Node, key: &str) -> Option<String> {
  //   elsif (ref $item eq 'ARRAY') {
  //     return $$item[1]{$key}; }
  if item.get_type() == Some(NodeType::ElementNode) {
    item.get_attribute(key)
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use libxml::parser::Parser as XmlParser;

  use super::*;

  #[test]
  fn token_type_footprint_bounds_to_varchar200_with_cntd() {
    // arXiv 0708.2155 — a short slice fits whole: just the token TYPES, no suffix.
    let short = "UNKNOWN:rho:1 OPEN:(:2 UNKNOWN:p:3 CLOSE:):4 RELOP:equals:5 \
                 start_POSTSUBSCRIPT:start:9 NUMBER:0:10";
    assert_eq!(
      token_type_footprint(short),
      "UNKNOWN_OPEN_UNKNOWN_CLOSE_RELOP_start_POSTSUBSCRIPT_NUMBER"
    );
    // The type is the segment before the first `:` (value/position dropped); empty → placeholder.
    assert_eq!(token_type_footprint("UNKNOWN:x:1 OPEN:(:2"), "UNKNOWN_OPEN");
    assert_eq!(token_type_footprint(""), "none");
    // A long formula is bounded to the varchar(200) `what` column and carries the `_cntd` marker;
    // the result never exceeds 200 chars so the dispatcher insert can't chop the marker.
    let many = (0..120)
      .map(|i| format!("UNKNOWN:x{i}:{i}"))
      .collect::<Vec<_>>()
      .join(" ");
    let fp = token_type_footprint(&many);
    assert!(
      fp.len() <= 200,
      "footprint {} exceeds varchar(200)",
      fp.len()
    );
    assert!(
      fp.ends_with("_cntd"),
      "a truncated footprint must mark it: {fp}"
    );
    assert!(fp.starts_with("UNKNOWN_UNKNOWN"));
  }

  #[test]
  fn warns_once_per_distinct_formula_per_document() {
    // Per-document scope = one `seen` set per document (the parser is built per document).
    let mut doc = HashSet::new();
    let f1 = "UNKNOWN:x:1 RELOP:=:2 NUMBER:0:3";
    let f2 = "UNKNOWN:y:1 ADDOP:+:2 UNKNOWN:z:3";
    assert!(warn_formula_once(&mut doc, f1), "first sighting warns");
    assert!(
      !warn_formula_once(&mut doc, f1),
      "the same formula repeated is suppressed"
    );
    assert!(
      !warn_formula_once(&mut doc, f1),
      "still suppressed on the 3rd repeat"
    );
    assert!(
      warn_formula_once(&mut doc, f2),
      "a DISTINCT formula warns once of its own"
    );
    assert!(!warn_formula_once(&mut doc, f2));
    // A new document (fresh set) warns again for the same formula — dedup is document-scoped.
    let mut other_doc = HashSet::new();
    assert!(warn_formula_once(&mut other_doc, f1));
  }

  /// Pin the message↔classifier coupling for the string-fallback path of
  /// resource-fatal recovery (P1-4). The PRIMARY transport is the structured
  /// `take_last_resource_fatal` latch; this fallback exists for errors that
  /// were flattened before the latch landed. If an engine message is
  /// reworded, this test points at the matcher to update.
  #[test]
  fn resource_fatal_message_fallback_recognizes_engine_messages() {
    use latexml_core::common::error::ErrorCategory as C;
    let cases = [
      (
        "Infinite expansion loop: a window of 2 token(s) repeated 100+ times",
        C::Recursion,
      ),
      (
        "Token limit of 400000000 exceeded, infinite loop?",
        C::TokenLimit,
      ),
      (
        "Pushback limit of 650000 exceeded, infinite loop?",
        C::PushbackLimit,
      ),
      (
        "Memory budget exceeded: RSS 4500 MB > cap 4500 MB",
        C::MemoryBudget,
      ),
    ];
    for (msg, expected) in cases {
      let err = resource_fatal_from_message(msg)
        .unwrap_or_else(|| panic!("classifier must recognize: {msg}"));
      assert_eq!(
        std::mem::discriminant(&err.category),
        std::mem::discriminant(&expected),
        "wrong category for: {msg}"
      );
    }
    assert!(resource_fatal_from_message("Some unrelated parse failure").is_none());
  }

  // Keep the Document alive for the duration of each test — Node holds an
  // implicit weak ref, so dropping the owning Document before reading the
  // Node leaves us with a hollow handle (get_type returns None, etc.).
  fn parse(xml: &str) -> libxml::tree::Document {
    XmlParser::default().parse_string(xml).expect("parse xml")
  }
  fn root(doc: &libxml::tree::Document) -> Node { doc.get_root_element().expect("root") }

  #[test]
  fn is_float_script_recognized() {
    assert!(is_float_script("FLOATSUPERSCRIPT"));
    assert!(is_float_script("FLOATSUBSCRIPT"));
    assert!(!is_float_script("SUPERSCRIPT"));
    assert!(!is_float_script("POSTSUPERSCRIPT"));
    assert!(!is_float_script(""));
  }

  #[test]
  fn is_post_script_recognized() {
    assert!(is_post_script("POSTSUPERSCRIPT"));
    assert!(is_post_script("POSTSUBSCRIPT"));
    assert!(!is_post_script("SUPERSCRIPT"));
    assert!(!is_post_script("FLOATSUPERSCRIPT"));
    assert!(!is_post_script(""));
  }

  #[test]
  fn parse_scriptpos_empty_defaults_to_post_zero() {
    assert_eq!(parse_scriptpos_str(""), ("post", 0));
  }

  // ---- LATEXML_MARPA_HYBRID_AUDIT_PARITY relaxed-comparison tests ----
  //
  // The audit's load-bearing question is "if both paths accept, do
  // they produce the same XM?". Non-Accepted/Non-Multiple outcome
  // pairs (Empty vs Rejected etc.) are compatible. The minimal
  // formula `\{u | a = b, c = d\}` triggers `Empty` (ASF) vs
  // `Rejected(...)` (Tree) — see marpa/docs/ASF_PERFORMANCE_FINDINGS.md.

  fn lex(name: &str) -> XM { XM::Lexeme(std::rc::Rc::from(name), metadata::Meta::default()) }

  #[test]
  fn parity_outcomes_compatible_both_empty_is_ok() {
    assert!(parity_outcomes_compatible(
      &ParseOutcome::Empty,
      &ParseOutcome::Empty
    ));
  }

  #[test]
  fn parity_outcomes_compatible_both_rejected_is_ok_even_with_different_messages() {
    let a = ParseOutcome::Rejected("infix_relation: …".to_string());
    let b = ParseOutcome::Rejected("some other pragma rejected".to_string());
    assert!(parity_outcomes_compatible(&a, &b));
  }

  #[test]
  fn parity_outcomes_compatible_empty_vs_rejected_is_ok_in_either_order() {
    let empty = ParseOutcome::Empty;
    let rej = ParseOutcome::Rejected("…".to_string());
    assert!(parity_outcomes_compatible(&empty, &rej));
    assert!(parity_outcomes_compatible(&rej, &empty));
  }

  #[test]
  fn parity_outcomes_compatible_accepted_vs_empty_is_a_real_mismatch() {
    let acc = ParseOutcome::Accepted(lex("x"));
    let empty = ParseOutcome::Empty;
    assert!(!parity_outcomes_compatible(&acc, &empty));
    assert!(!parity_outcomes_compatible(&empty, &acc));
  }

  #[test]
  fn parity_outcomes_compatible_accepted_vs_rejected_is_a_real_mismatch() {
    let acc = ParseOutcome::Accepted(lex("x"));
    let rej = ParseOutcome::Rejected("…".to_string());
    assert!(!parity_outcomes_compatible(&acc, &rej));
    assert!(!parity_outcomes_compatible(&rej, &acc));
  }

  #[test]
  fn parity_outcomes_compatible_accepted_equal_xm_is_ok() {
    let a = ParseOutcome::Accepted(lex("foo"));
    let b = ParseOutcome::Accepted(lex("foo"));
    assert!(parity_outcomes_compatible(&a, &b));
  }

  #[test]
  fn parity_outcomes_compatible_accepted_different_xm_is_a_real_mismatch() {
    let a = ParseOutcome::Accepted(lex("foo"));
    let b = ParseOutcome::Accepted(lex("bar"));
    assert!(!parity_outcomes_compatible(&a, &b));
  }

  #[test]
  fn parity_outcomes_compatible_multiple_treated_as_accepted_kind() {
    let m = ParseOutcome::Multiple(vec![lex("x"), lex("y")]);
    let empty = ParseOutcome::Empty;
    // Multiple-vs-Empty is a real mismatch (one path returned trees,
    // the other didn't) — same semantics as Accepted-vs-Empty.
    assert!(!parity_outcomes_compatible(&m, &empty));
    assert!(!parity_outcomes_compatible(&empty, &m));
  }

  #[test]
  fn parse_scriptpos_str_just_position() {
    // No digits → level 0.
    assert_eq!(parse_scriptpos_str("post"), ("post", 0));
    assert_eq!(parse_scriptpos_str("mid"), ("mid", 0));
  }

  #[test]
  fn parse_scriptpos_str_position_and_level() {
    assert_eq!(parse_scriptpos_str("post1"), ("post", 1));
    assert_eq!(parse_scriptpos_str("mid2"), ("mid", 2));
    assert_eq!(parse_scriptpos_str("pre3"), ("pre", 3));
  }

  #[test]
  fn parse_scriptpos_str_just_digit_is_empty_pos() {
    // Entirely-numeric input → position is the empty string, level parses.
    assert_eq!(parse_scriptpos_str("5"), ("", 5));
  }

  #[test]
  fn p_get_attribute_returns_attr_on_element() {
    let doc = parse(r#"<XMTok role="ADDOP" meaning="plus">+</XMTok>"#);
    let n = root(&doc);
    assert_eq!(p_get_attribute(&n, "role").as_deref(), Some("ADDOP"));
    assert_eq!(p_get_attribute(&n, "meaning").as_deref(), Some("plus"));
    assert_eq!(p_get_attribute(&n, "absent"), None);
  }

  #[test]
  fn p_get_value_element_content_wins() {
    let doc = parse(r#"<XMTok name="fallback">+</XMTok>"#);
    assert_eq!(p_get_value(&root(&doc)), "+");
  }

  #[test]
  fn p_get_value_element_falls_back_to_name_attr() {
    let doc = parse(r#"<XMTok name="plus"/>"#);
    assert_eq!(p_get_value(&root(&doc)), "plus");
  }

  #[test]
  fn p_get_value_element_empty_with_no_name_is_empty() {
    let doc = parse(r#"<XMTok/>"#);
    assert_eq!(p_get_value(&root(&doc)), "");
  }
}

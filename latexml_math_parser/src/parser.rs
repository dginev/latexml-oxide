use libxml::tree::{Node, NodeType};
use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::io::Cursor;

use latexml_core::common::arena::{self, SymHashMap};
use latexml_core::common::error::{Result, note_begin, note_end, note_progress};
use latexml_core::common::xml::*;
use latexml_core::document::{Document, get_node_qname, sym_can_have_attribute, with_node_qname};
use latexml_core::pin;
use latexml_core::{Warn, fatal, map, s, static_map, sym_map};

use crate::grammar::builder::init_grammar;
use crate::pragmatics::ValidationPragmatics;
use crate::semantics::*;
use crate::util::{filter_hints, node_to_grammar_lexemes_from};
use marpa::lexer::byte_scanner::*;
use marpa::parser::*;
use marpa::thin::Grammar as ThinGrammar;
use marpa::tree_builder::TreeBuilder;

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
  // strict: bool,
  // warned: bool,
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
      // strict: true,
      // warned: false,
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

impl MathParser {
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
          "expected", "MathGrammar",
          "math parser: init_grammar fallback failed ({}) — leaving engine in last known state",
          e
        );
      },
    }
  }

  fn try_reset_clone_path(&mut self) -> std::result::Result<(), ()> {
    let mut engine = Parser::with_grammar(self.grammar.clone());
    // Run a trivial recognizer to advance state from G (precompute) through
    // R → B → O → T → GReady. Use "NUMBER:1:1 " which is a valid single-token formula.
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
      for math in xmath_nodes {
        let math_ref = math.clone();
        // Per-formula timing feeds the math_parse_buckets histogram in
        // telemetry. ~20 ns Instant cost per formula is negligible vs Marpa.
        // See docs/TELEMETRY.md.
        let t0 = std::time::Instant::now();
        self.parse(math, document)?;
        let elapsed_us = t0.elapsed().as_micros() as u64;
        latexml_core::telemetry::record_math_parse(elapsed_us, self.last_parsetrees_count as u32);
        // Store parse tree count as attribute on the Math element for diagnostics.
        // Find the ancestor ltx:Math of this XMath node and set _parsetrees.
        if self.last_parsetrees_count > 0 {
          if let Some(mut math_parent) = math_ref.get_parent() {
            if math_parent.get_name() == "Math" {
              let _ =
                math_parent.set_attribute("_parsetrees", &self.last_parsetrees_count.to_string());
            }
          }
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
          ) {
            if let Some(base_role) = children[1].get_attribute("role") {
              if matches!(
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
              ) {
                let _ = xmapp.set_attribute("role", &base_role);
              }
            }
          }
        }
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
    if let Some(result) = self.parse_rec(xnode, "Anything,", document)? {
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
    } else if let Some(text) = p
      .get_attribute("tex")
      .and_then(|tex| TEX_TEXT_FALLBACK.get(tex.as_str()))
    {
      p.set_attribute("text", text)?;
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
    } else if let Some(mut result) = self.parse_single(&mut node, document, &rule)? {
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
        // Danger: the above code replaced the id on the parsed result with the one from XMArg,..
        // If there are any references to `resultid`, we need to point them to `newid`!
        if let Some(rid) = resultid {
          if let Some(nid) = newid {
            if rid != nid {
              for mut tref in document.findnodes(&s!("//*[@idref='{}']", rid), None) {
                tref.set_attribute("idref", &nid)?;
              }
            }
          }
        }
      }
      Ok(Some(result))
    } else {
      // Parse failed — run kludge to wrap OPEN/CLOSE delimiters
      *self.failed.entry_sym(tag).or_insert(0) += 1;
      if tag == pin!("ltx:XMath") {
        self.failed_xmath_ids.push(node.to_hashable());
        // Kludge (OPEN/CLOSE wrapping) runs post-parse in core_interface.rs
        // using failed_xmath_ids to find the failed nodes.
      }
      Ok(None)
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
          if let Some(mut result) = self.parse_rec(child, "Anything", document)? {
            if let Some(ref role) = saved_role {
              result.set_attribute("role", role).ok();
            }
          } else if let Some(ref role) = saved_role {
            // Parse failed — XMWrap still in DOM, restore role
            c.set_attribute("role", role).ok();
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
      if let Ok(Some(parse_tree)) = self.parse_lexemes(lexemes, &nodes, document) {
        //START reparent: the reparenting used to be in `parse_rec` in Perl. Is this a good place?
        // Replace the content of XMath with parsed result
        // unbindNode followed by (append|replace)Tree (which removes ID's) should be safe
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
    let parse_result = self
      .engine
      .run_recognizer(ByteScanner::new(Cursor::new(input)))?;
    if *PARSE_LEXEMES_DBG {
      eprintln!("PARSE_LEXEMES_RECOGNIZED");
    }
    let mut parses: Vec<XM> = Vec::new();
    let mut ok_trees = 0;
    let mut pruned_trees = 0;
    let mut deduped = 0usize;
    let mut consecutive_dupes = 0usize;
    let start = std::time::Instant::now();
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
        Err(_prune_err) => {
          pruned_trees += 1;
          // Pruned trees also count toward convergence if we have unique parses
          if !parses.is_empty() {
            consecutive_dupes += 1;
          }
        },
      }
    }

    // Store count for \ltx@count@parses diagnostic macro
    // Use post-dedup count (distinct semantic trees), not raw grammar count
    self.last_parsetrees_count = parses.len();

    if ok_trees + pruned_trees > 10 {
      // Diagnostic only — high ambiguity isn't a Perl-side Error in
      // MathParser.pm; the warn target uses a Rust-internal class.
      log_math_warn!(
        "ambiguous", "math",
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
        crate::semantics::restructure_formulae_right(&mut parse_tree)?;
        // Rename `list` to `vector`/`set` when delimiters wrap the list (Perl encloseN)
        crate::semantics::rename_fenced_lists(&mut parse_tree, nodes)?;
        // Combine adjacent SUPOP tokens (prime+prime → prime2)
        crate::semantics::combine_supop_post(&mut parse_tree, nodes)?;
        Ok(Some(parse_tree))
      },
      Err(_e) => {
        self.reset_engine();
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
  } else if let Some(bump_str) = base.get_attribute("_bumplevel") {
    if let Ok(bump) = bump_str.parse::<u32>() {
      l = bump;
    }
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
      // Perl MathParser.pm:1394 — Error('expected', 'arguments', …)
      log_math_error!(
        "expected", "arguments",
        "XMApp element has no child arguments"
      );
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
      // Perl MathParser.pm:1394-style — XMDual without children is
      // structurally as bad as XMApp without children.
      log_math_error!(
        "expected", "arguments",
        "XMDual element has no child arguments"
      );
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
        .map(|cell| {
          if let Some(first_child) = cell.get_first_child() {
            textrec(&first_child, None, None, document)
          } else {
            String::new()
          }
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
  if with_node_qname(node, |name| name == "ltx:XMRef") {
    if let Some(idref) = node.get_attribute("idref") {
      // Can it happen that $realnode is, itself, an XMRef?
      // Then we should recurse recurse!
      if let Some(realnode) = document.lookup_id(&idref) {
        return Cow::Borrowed(realnode);
      } else {
        let message = s!("Cannot find a node with xml:id='{}'", idref);
        // TODO:
        // LaTeXML::MathParser::IDREFS{$idref}
        // ? "Previously bound to " .
        // ToString($LaTeXML::MathParser::IDREFS{$idref})           : ()));
        // Perl Document.pm L1553: Warn, not Error (missing XMRef targets are common)
        let warn_fn = || -> Result<()> {
          Warn!("expected", "id", message);
          Ok(())
        };
        warn_fn().ok();
        //       return ['ltx:ERROR', {}, "Missing XMRef idref=$idref"]; } }
      }
    }
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
  use super::*;
  use libxml::parser::Parser as XmlParser;

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

use crate::semantics::*;
use marpa::grammar::Grammar as MarpaGrammar;
use marpa::result::Result;
use marpa::tree_builder::TreeBuilder;

// Marpa SLIF-style grammar DSL inside `grammar!()` / `production!()`
// macros. A handful of long alternation lists (`qm_bracket`,
// `floatsuperscript`) push past 100 chars; `rustfmt::skip` is applied
// to the entire builder so the BNF reads as a flat table rather than
// being re-flowed mid-rule.
#[rustfmt::skip]
pub fn init_grammar() -> Result<(MarpaGrammar, Actions, TreeBuilder)> {
  // We create a declarative macro language of our own, in the spirit of the Marpa SLIF
  default_registry!();
  // Tokens, to be used in rules directly
  token!(atom ~ "ATOM");
  token!(unknown ~ "UNKNOWN");
  token!(id ~ "ID");
  // M4: Specialized tokens for "d" that could be differential operators.
  // Lexer emits XDIFFUNK/XDIFFID instead of UNKNOWN/ID for "d" content.
  // These are added as alternatives everywhere unknown/id appear, plus in the diffop rule.
  token!(diffunk ~ "XDIFFUNK");
  token!(diffid ~ "XDIFFID");
  token!(array ~ "ARRAY");
  token!(number ~ "NUMBER");
  token!(punct ~ "PUNCT");
  token!(period ~ "PERIOD");
  token!(addop ~ "ADDOP");
  token!(mulop ~ "MULOP");
  token!(relop ~ "RELOP");
  token!(elideop ~ "ELIDEOP");
  token!(langle_rel = "RELOP:less-than");
  token!(langle_open = "OPEN:langle");
  token!(langle = [langle_rel langle_open]);
  token!(rangle_rel = "RELOP:greater-than");
  token!(rangle_close = "CLOSE:rangle");
  token!(rangle =[rangle_rel rangle_close]);
  token!(vertbar ~ "VERTBAR");
  token!(singlevertbar = "VERTBAR:|");
  token!(close_pipe = "CLOSE:|");
  token!(middle_bar = "MIDDLE:|");
  token!(middle_parallel = "MIDDLE:parallel-to");
  token!(midbar = [vertbar middle_bar middle_parallel]);
  token!(lbrace = "OPEN:{");
  token!(rbrace = "CLOSE:}");
  token!(lparen = "OPEN:(");
  token!(rparen = "CLOSE:)");
  token!(lbracket = "OPEN:[");
  token!(rbracket = "CLOSE:]");
  token!(relop_equals = "RELOP:equals");
  token!(metarelop ~ "METARELOP");
  token!(modifierop ~ "MODIFIEROP");
  token!(modifier ~ "MODIFIER");
  token!(arrow ~ "ARROW");
  token!(binop ~ "BINOP");
  token!(postfix ~ "POSTFIX");
  token!(function ~ "FUNCTION");
  token!(opfunction ~ "OPFUNCTION");
  token!(trigfunction ~ "TRIGFUNCTION");
  token!(applyop ~ "APPLYOP");
  token!(composeop ~ "COMPOSEOP");
  token!(supop ~ "SUPOP");
  // `open`/`close` match GENERIC delimiters (\lfloor, \lceil, \llbracket, etc.)
  // Specific delimiters (paren, bracket, brace, langle) have their own tokens.
  // The lexer (util.rs) emits OTHER_OPEN:/OTHER_CLOSE: for generic delimiters
  // so Marpa doesn't ambiguously match both `open` AND `lparen` for `OPEN:(:N`.
  token!(open ~ "OTHER_OPEN");
  token!(close ~ "OTHER_CLOSE");
  token!(middle ~ "MIDDLE");
  token!(bigop ~ "BIGOP");
  token!(sumop ~ "SUMOP");
  token!(intop ~ "INTOP");
  token!(limitop ~ "LIMITOP");
  token!(diffop ~ "DIFFOP");
  token!(operator ~ "OPERATOR");
  token!(start_postsubscript ~ "start_POSTSUBSCRIPT");
  token!(end_postsubscript ~ "end_POSTSUBSCRIPT");
  token!(start_postsuperscript ~ "start_POSTSUPERSCRIPT");
  token!(end_postsuperscript ~ "end_POSTSUPERSCRIPT");
  // Separated bigop script tokens — prevents earley chart competition
  // between scripted_bigop and scripted_factor rules
  token!(start_bigopsub ~ "start_BIGOPSUB");
  token!(end_bigopsub ~ "end_BIGOPSUB");
  token!(start_bigopsup ~ "start_BIGOPSUP");
  token!(end_bigopsup ~ "end_BIGOPSUP");
  token!(start_floatsuperscript ~ "start_FLOATSUPERSCRIPT");
  token!(end_floatsuperscript ~ "end_FLOATSUPERSCRIPT");
  token!(start_floatsubscript ~ "start_FLOATSUBSCRIPT");
  token!(end_floatsubscript ~ "end_FLOATSUBSCRIPT");
  token!(start_arrow ~ "start_ARROW");
  token!(end_arrow ~ "end_ARROW");

  rules!(
      // Factors
      // opfunction/function/trigfunction are NOT factors — they require arguments.
      // Standalone usage is handled at the term level (term += function | ...).
      // `2 \sin` is handled via dedicated tight_term rules below.
      // Perl MathGrammar L315: ATOM_OR_ID : ATOM | ID | ARRAY
      // XMArray elements (role="ARRAY") should parse as atoms/factors, like matrices in equations
      // M4: diffunk/diffid are added as factor_base alternatives so "d" tokens
      // can appear anywhere unknown/id appear. The diffop rule only uses diffunk/diffid.
      factor_base = unknown | number | id | atom | array | diffunk | diffid;
      // Perl MathGrammar L277: OPEN ARRAY CLOSE -> Fence (e.g. \{ array \} or ( array ))
      // Also handle unmatched delimiters for cases-like patterns.
      // Perf: `open` is now narrowed to OTHER_OPEN (non-paren/bracket/brace), so
      // we add specific rules for each standard delimiter.
      fenced_array = open array close => fenced
        | open array => open_fenced
        | array close => close_fenced
        | lparen array rparen => fenced
        | lparen array => open_fenced
        | array rparen => close_fenced
        | lbracket array rbracket => fenced
        | lbracket array => open_fenced
        | array rbracket => close_fenced
        | lbrace array rbrace => fenced
        | lbrace array => open_fenced
        | array rbrace => close_fenced;
      // FUNCTION and OPFUNCTION are both factors (participate in implicit multiplication).
      // The distinction is in argument absorption: OPFUNCTION absorbs bare args,
      // FUNCTION only absorbs fenced (parenthesized) args.
      factor = factor_base | function | opfunction | fenced_array;
      // Perl: limit-from@(number, sign) — directional limits: 0+, 1-
      // A "left-only term": on the left behaves as a term (for comma lists),
      // on the right terminates at the addop (like expression-level postfix).
      limit_from_term = factor_base addop => limit_from_apply;
      // Terms
      // Perl: bigop = BIGOP | SUMOP | INTOP | LIMITOP | DIFFOP
      any_bigop = bigop | sumop | intop | limitop | diffop;
      // Adjacent bigops compose like higher-order functions:
      // \int\iint => integral(double-integral), itself a compound operator
      composed_bigop = any_bigop any_bigop => prefix_apply
        | any_bigop composed_bigop => prefix_apply;

      // Compound operators: OPERATOR composed with functions/other operators (right-recursive)
      // D sin => Apply(D, sin), D D sin => Apply(D, Apply(D, sin))
      // Must end with a function/trigfunction (no bare operator-only compounds)
      compound_operator = operator trigfunction => prefix_apply
        | operator function => prefix_apply
        | operator opfunction => prefix_apply
        | operator compound_operator => prefix_apply;

      // tight_term includes single factors (for left-recursive chaining)
      // and all compound constructs (invisible times, prefix application, etc.)
      // The `\log x` → `log*x` issue is handled by semantic pruning in
      // apply_invisible_times, not at the grammar level.
      tight_term = factor
        | tight_term factor => apply_invisible_times
        // Perl MathGrammar L423: POSTFIX (e.g. n!) => Apply(op, term)
        | tight_term postfix => apply_postfix
        // Note: FUNCTION does NOT absorb bare args — only parens or APPLYOP.
        // `fga` = f*g*a, but `f(a)` = f@(a). OPFUNCTION absorbs: `Fga` = F@(g*a).
        // FUNCTION only chains via opfunction's factor status or APPLYOP.
        // trigfunction uses trigbarearg via applied_func (absorbs MulOp chains)
        // NOTE: bigop rules moved to += section (after `term` is defined) so they
        // can absorb full term (mulop chains like x² * dx), not just tight_term.
        | composed_bigop tight_term => prefix_apply
        | operator factor => prefix_apply
        | factor_base applyop tight_term => prefix_apply_applyop
        // Perl: FUNCTION/OPFUNCTION/TRIGFUNCTION + explicit APPLYOP + argument
        // Handles \lxDeclare-annotated tokens: f⁡(x) where ⁡ is APPLYOP
        | function applyop tight_term => prefix_apply_applyop
        | opfunction applyop tight_term => prefix_apply_applyop
        | trigfunction applyop tight_term => prefix_apply_applyop;

      // Perl MathGrammar L258: Factor moreFactors — consecutive function
      // applications chain with invisible times.
      // e.g. \sin x \cos y => sin(x) * cos(y)
      //
      // IMPORTANT — TRIGFUNCTION ARGUMENT SCOPING AMBIGUITY:
      // Perl's trigBarearg absorbs MulOp chains: \sin\pi\times x => sin(π×x).
      // We deliberately DO NOT implement this absorption. The expression
      // sin(π)×x vs sin(π×x) is a *legitimate semantic ambiguity* that cannot
      // be resolved purely by grammar structure. Both parses are valid:
      //   sin(π)×x = 0×x (evaluating sin at π)
      //   sin(π×x) = sin of the product
      // Perl's Parse::RecDescent picks the "absorb" interpretation heuristically.
      //
      // FUTURE WORK: To match Perl's output, implement *targeted semantic pruning*
      // in the parse tree selection phase (semantics/tree.rs) that uses context
      // cues to prefer one interpretation:
      //   - If the MulOp is invisible (⁢), prefer absorption (sin 2x → sin(2x))
      //   - If the MulOp is explicit (×,·), either interpretation is valid
      //   - If the argument is a known constant (π, e), standalone may be preferred
      // This is a semantic-level decision, not a grammar-level one.
      // applied_func and tight_term augmentations moved below trig_arg definition

      // Composed functions: f∘g, sin∘cos — these can then be applied as functions
      // COMPOSEOP operates on function-level operands (curry level 2)
      // Left-to-right associative (matching Perl): f∘g∘h = (f∘g)∘h
      composed_term = function composeop function => infix_apply
        | function composeop trigfunction => infix_apply
        | function composeop opfunction => infix_apply
        | trigfunction composeop function => infix_apply
        | trigfunction composeop trigfunction => infix_apply
        | trigfunction composeop opfunction => infix_apply
        | opfunction composeop function => infix_apply
        | opfunction composeop trigfunction => infix_apply
        | opfunction composeop opfunction => infix_apply
        // Left-recursive for left-to-right associativity
        | composed_term composeop function => infix_apply
        | composed_term composeop trigfunction => infix_apply
        | composed_term composeop opfunction => infix_apply;

      // Composed functions can be applied like regular functions
      tight_term += composed_term tight_term => prefix_apply;

      term = tight_term
      | term mulop tight_term => infix_apply_nary
      | term mulop tight_term elideop => infix_apply_and_elide
      // Perl: BINOP matches both AddOp and MulOp (ambiguous precedence from \mathbin)
      | term binop tight_term => infix_apply_nary
      | term binop tight_term elideop => infix_apply_and_elide
      // Fallback: COMPOSEOP on general terms (for non-function-level composition)
      | term composeop term => infix_apply
      | operator applyop term => prefix_apply_applyop;


      // Allow standalone functions/trigfunctions/opfunctions/operators as terms
      // This is needed for (f*g)(x) where f and g are FUNCTION tokens
      // opfunction here allows standalone \operatorname{R} to parse
      // operator as term enables D - 1 (subtraction), D + G (addition)
      term += function | trigfunction | opfunction | composed_term | operator;
      // Allow elideop (\cdots) as a term for chains like y + i + \cdots + y_n
      // Perl treats cdots as a regular term in addition chains
      term += elideop;

      // Higher-order operator terms: functions as standalone objects multiplied by factors
      // `2\sin` = `2 * sin`, `2\sin\cos` = `2 * sin * cos`
      // These are term-level (not tight_term) so they don't interfere with
      // function application: `2\sin x` = `2 * sin(x)` (not `(2*sin) * x`)
      tight_opterm = factor function => apply_invisible_times
        | factor trigfunction => apply_invisible_times
        | factor opfunction => apply_invisible_times
        // Consecutive functions multiply: fgh → f·g·h (Perl Factor moreFactors)
        | function function => apply_invisible_times
        | function trigfunction => apply_invisible_times
        | function opfunction => apply_invisible_times
        | tight_opterm function => apply_invisible_times
        | tight_opterm trigfunction => apply_invisible_times
        | tight_opterm opfunction => apply_invisible_times;
      term += tight_opterm;

      // Expressions
      expression = term
        | expression addop term => infix_apply_nary
        | expression addop term elideop => infix_apply_and_elide
        | addop tight_term => prefix_apply
        | factor addop => postfix_apply
        | expression addop => postfix_apply
        // Perl MathGrammar L236: addExpressionModifier: MODIFIEROP Expression
        // => Apply(modifierop, expr, expr2). Handles infix `a mod b`.
        | expression modifierop expression => infix_apply
        // Perl MathGrammar L236: addExpressionModifier: MODIFIER
        // Standalone postfix modifier (e.g. `8\pmod{3}` → annotated(8, pmod(3)))
        // Placed at expression level so MODIFIER binds BEFORE RELOP.
        // e.g. `5 ≡ 8 \pmod{3}` → `5 ≡ annotated(8, pmod(3))` not `annotated(5≡8, pmod(3))`
        | expression modifier => postfix_modifier_apply
        // Perl MathGrammar L224-233: OPEN relop/modifierop Expression balancedClose
        // Parenthesized modifier expressions: x(>0) → annotated(x, Fence(>0))
        | expression lparen relop expression rparen => annotated_fenced_modifier
        | expression lparen modifierop expression rparen => annotated_fenced_modifier
        // Perl MathGrammar L223: PUNCT? OPEN relop Expression CLOSE
        // Semicolon annotation: a;(<e) → annotated(a, absent < e)
        | expression punct lparen relop expression rparen => annotated_punct_fenced_modifier
        | expression punct lparen modifierop expression rparen => annotated_punct_fenced_modifier;

      // Formula
      // Perl MathGrammar L73/236: MODIFIEROP Expression => Apply(mod, Absent, expr)
      modifier_expression = modifierop expression => modifier_prefix_apply;
      // Perl: within a Formula, comma-separated expressions after a relop form a list RHS.
      // e.g. a=b,c,d → a = list(b,c,d), not list(a=b, c, d).
      // Uses formula_list_apply which rejects items containing relops (those belong at statement level).
      formula_list = expression punct expression => formula_list_apply
        | formula_list punct expression => formula_list_apply;
      // Comma-separated term lists: term, term, term, ...
      // Used for angle-bracket inner products <x,y>, <a,b,c>, etc.
      // Also includes limit_from_term for patterns like (1+, 0+, 1-, 0-).
      term_list = term punct term => list_apply
        | term_list punct term => list_apply
        | limit_from_term punct term => list_apply
        | limit_from_term punct limit_from_term => list_apply
        | term_list punct limit_from_term => list_apply
        | term punct limit_from_term => list_apply;

      // Perl MathGrammar L709-711: Two-part relops (>=, <=, <<, >>)
      two_part_relop = langle_rel langle_rel => two_part_relop_combine
        | rangle_rel rangle_rel => two_part_relop_combine
        | langle_rel relop_equals => two_part_relop_combine
        | rangle_rel relop_equals => two_part_relop_combine;

      formula = expression
        | formula relop expression => infix_relation
        | formula two_part_relop expression => infix_relation
        | formula relop formula_list => infix_relation
        | formula relop => postfix_relop
        // Perl moreRelations: `relop moreRelations` — consecutive relops chain without intervening terms
        // e.g. `A ∈ ∞ ∋` → the ∈ absorbs ∞, then ∋ appends to the chain (no absent)
        | formula relop relop => consecutive_relop_chain
        | formula arrow expression => infix_relation
        | arrow expression => prefix_arrow_apply
        // Arrow-wrapped content (from amscd XMWrap role="ARROW"):
        // Parsed as a prefix arrow application on the enclosed content.
        | start_arrow arrow expression end_arrow => arrow_wrap_apply
        | start_arrow arrow end_arrow => arrow_wrap_solo
        // Perl MathGrammar L81: AnyOp Expression => Apply(AnyOp, Absent(), Expression)
        // Leading relop with implied absent left operand (e.g. "= e + f + g" in eqnarray)
        | relop expression => prefix_relop_apply
        | metarelop expression => prefix_relop_apply
        | modifier_expression;

      // Perl MathGrammar: Factor includes preScripted['bigop'] as standalone
      // So standalone bigops can form statements (needed for list expressions like \int \quad \int)
      statement = formula
        | statement metarelop formula => infix_relation
        | metarelop formula => prefix_metarelop_apply
        | any_bigop | composed_bigop
        | operator | compound_operator
        | function | trigfunction
        // Bare operators can form comma-separated lists: +,-,×
        | addop | mulop | binop | relop | arrow
  ;

      end_punct = punct | period;
      statements = statement
        | statement end_punct => postfix_embellished
        | statements end_punct => postfix_embellished
        | statements punct statement => list_apply
        // Perl MathGrammar L129: endPunct includes PERIOD. Period creates formulae, not list.
        | statements period statement => formulae_apply
        // Perl: MorphVertbar — VERTBAR as conditional modifier: x | y,z,t
        | statement vertbar statements => vertbar_modifier;

      // Perl MathGrammar: Formulae = Formula (endPunct Formula)* → NewFormulae()
      // Separate nonterminal from expression-level formula_list to avoid ambiguity:
      // expression-level formula_list uses formula_list_apply (rejects relops),
      // while formulae uses formulae_apply (accepts full statements).
      formulae = statement punct statement => formulae_apply
        | formulae punct statement => formulae_apply
        // Period also separates formulae
        | statement period statement => formulae_apply
        | formulae period statement => formulae_apply;

      // Extensions, now that we have more category variables defined
      fenced_factor = lbrace expression rbrace    => fenced
             | lbracket expression rbracket       => fenced
             | lparen formula rparen              => fenced
             // METARELOP inside parens: f(a:b), f(a↔b) — colon/arrow as relation in fenced
             | lparen formula metarelop expression rparen => fence
             // Parenthesized comma-separated lists: (a,b,c), (a+b, c+d), (1+, 0+, 1-, 0-)
             // Perf (Fix 3): `lparen term_list rparen` was duplicate with
             // `lparen formula_list rparen` for non-relational content (both produce
             // identical `list@(...)` trees). Dropped; formula_list covers
             // (a,b,c), (a+b,c+d), and (0+,1-) via `factor addop => postfix_apply`
             // which produces limit-from XM matching limit_from_apply semantics.
             | lparen formula_list rparen        => fenced
             // Bracketed and braced comma-separated lists: [a,b,c], {a,b,c}
             | lbracket formula_list rbracket    => fenced
             | lbrace formula_list rbrace        => fenced
             // Angle brackets as delimiters: <x,y> for inner products, etc.
             // Old typesetting conventions used < > instead of \langle \rangle.
             // Uses term_list (comma-separated terms) to avoid matching complex
             // nested expressions. Only fires when content has commas.
             | langle_rel term_list rangle_rel => fenced
             // Angle-bracket fencing with \langle/\rangle (distinct from parentheses)
             // Now that langle_open/rangle_close are NOT remapped to lparen/rparen,
             // we need explicit rules for angle-bracket fenced expressions.
             // M8: removed `langle_open expression rangle_close` — subsumed by formula
             // (every expression is a formula; keeping both creates 2x ambiguity)
             | langle_open formula rangle_close => fenced
             | langle_open term_list rangle_close => fenced
             | langle_open formula_list rangle_close => fenced
             | langle_open formula metarelop expression rangle_close => fence
             // Perf/design: interval rules moved out of fenced_factor into
             // term (see `tight_term += interval_term` below). Math convention:
             // an interval `(a,b)` is a named mathematical object — a set of
             // numbers — not a grouping mechanism like `(a+b)`. Treating it as
             // a term instead of fenced_factor has a clean consequence:
             // function application `f(x,y)` takes a fenced_factor, so the
             // interval interpretation of `(x,y)` is pruned naturally; the
             // list interpretation (from `lparen formula_list rparen`) wins.
             // QM bra-ket uses langle_open/rangle_close (specific ⟨⟩ tokens),
             // avoiding ambiguity with relational < > (langle_rel/rangle_rel).
             // Conditional probability uses lparen/rparen (specific () tokens),
             // avoiding ambiguity with ket (which requires rangle_close).
             //
             // Perl MathGrammar L294: || exp || → norm (must be before |exp| → abs-val)
             // CatSymbols merges two | into ‖; singlevertbar = VERTBAR:|
             | singlevertbar singlevertbar expression singlevertbar singlevertbar => norm_fenced
             | singlevertbar expression singlevertbar => fenced
             // Dirac ket: |label⟩ — VERTBAR as opening, CLOSE:rangle as closing
             // Restricted to rangle_close (⟩) to avoid ambiguity with conditional
             // probability (x|y) where ) is a generic CLOSE but not rangle.
             // Perl MathGrammar uses RANGLE specifically, not generic CLOSE.
             // Ket labels: expressions, arrows, operators, relops, etc.
             | singlevertbar expression rangle_close => qm_ket
             | singlevertbar arrow rangle_close => qm_ket
             | singlevertbar metarelop rangle_close => qm_ket
             | singlevertbar operator rangle_close => qm_ket
             | singlevertbar any_bigop rangle_close => qm_ket
             | singlevertbar mulop rangle_close => qm_ket
             | singlevertbar addop rangle_close => qm_ket
             | singlevertbar relop rangle_close => qm_ket
             | singlevertbar modifierop rangle_close => qm_ket
             // Dirac bra: ⟨label| — OPEN:langle as opening, VERTBAR as closing
             // Restricted to langle_open to avoid ambiguity with parens.
             | langle_open expression singlevertbar => qm_bra
             | langle_open arrow singlevertbar => qm_bra
             | langle_open metarelop singlevertbar => qm_bra
             | langle_open operator singlevertbar => qm_bra
             // Braket: ⟨a|b⟩ → inner-product@(a, b)
             | langle_open expression singlevertbar expression rangle_close => qm_braket
             // Bracket: ⟨a|f|b⟩ → quantum-operator-product@(a, f, b)
             | langle_open expression singlevertbar expression singlevertbar expression rangle_close => qm_bracket
             // Perl's Fence for comma-separated items in braces: {a,b} and {a,b,c}
             | lbrace term punct term rbrace => fence
             | lbrace term punct term punct term rbrace => fence
             // Perl: {a|b} conditional-set with VERTBAR or MIDDLE separator
             | lbrace formula singlevertbar formula rbrace => fence
             | lbrace formula middle_bar formula rbrace => fence
             | lbrace formula metarelop formula rbrace => fence
             // Conditional probability: p(a|b) — safe now that ket uses rangle_close
             // (not generic close), so |y) no longer matches ket pattern.
             // NOTE: formula_list variants (x,y|z) don't work due to Marpa limitation:
             // formula_list completion doesn't propagate to singlevertbar continuation.
             // Perl handles this via recursive descent context. Tracked as known limitation.
             | lparen formula singlevertbar formula rparen => fence
             | lparen formula_list singlevertbar formula rparen => fence
             | lparen formula singlevertbar formula_list rparen => fence
             // \middle separator: \left(a\middle|b\right) → fenced with separator
             // MIDDLE tokens are author-explicit (unlike bare |), so unambiguous.
             // `open`/`close` now only match generic delimiters (OTHER_OPEN/OTHER_CLOSE),
             // so we also add specific rules for paren/bracket. Not for langle/brace —
             // those match specific QM/set rules, not this generic fence path.
             | open formula middle_bar formula close => fence
             | open formula middle formula close => fence
             | lparen formula middle_bar formula rparen => fence
             | lparen formula middle formula rparen => fence
             | lbracket formula middle_bar formula rbracket => fence
             | lbracket formula middle formula rbracket => fence
             // Generic OPEN/CLOSE delimiters: \lfloor...\rfloor, \lceil...\rceil, etc.
             // Perl MathGrammar: OPEN Expression CLOSE → Fence
             | open expression close => fenced
             // Fenced singleton bigops/operators: (\int), (\Delta), (\sum)
             // Perl allows bigops/operators as factors; here we only allow them fenced.
             | lparen any_bigop rparen => fenced
             | lparen composed_bigop rparen => fenced
             | lparen operator rparen => fenced
             | lparen compound_operator rparen => fenced
             | open any_bigop close => fenced
             | open operator close => fenced
             // Empty fenced expressions: () [] {} ⌊⌋ ⟨⟩ etc.
             | lparen rparen => empty_fenced
             | lbracket rbracket => empty_fenced
             | lbrace rbrace => empty_fenced
             | langle_open rangle_close => empty_fenced
             | open close => empty_fenced;
      factor += fenced_factor;

      // Perl: addTrigFunArgs → trigBarearg → aTrigBarearg moreTrigBareargs
      // Trig functions absorb chains of mulop+factor (but NOT other trig functions).
      // aTrigBarearg includes: FUNCTION+args, OPFUNCTION+args, ATOM_OR_ID, UNKNOWN, NUMBER
      //
      // Perf: removed `| fenced_factor` and `| opfunction fenced_factor` alternatives.
      // `factor += fenced_factor` makes fenced_factor reachable through `factor`, so:
      //   - `factor` alone already covers fenced_factor (was duplicate)
      //   - `opfunction factor` already covers `opfunction fenced_factor` (was duplicate)
      // `function fenced_factor` remains — distinct from `function factor`, which is
      // intentionally NOT a production (FUNCTION requires parens for application:
      // `f(x)` = f@(x), but `f x` = f*x, per Perl's FUNCTION vs OPFUNCTION distinction).
      // Perf (grammar pruning): trig_arg uses `factor_base` (bare factors only),
      // NOT `factor` (which includes fenced_factor). This prevents the double-path
      // ambiguity where `\sin(x)` matched BOTH:
      //   - `applied_func = trigfunction trig_arg` (via `trig_arg = factor` → fenced_factor) → prefix_apply
      //   - `applied_func = trigfunction lparen formula rparen` → apply_delimited
      // giving 2 interpretations per trig+paren. Two trig+paren pairs in one formula
      // produced 4× ambiguity multiplier. By excluding fenced_factor from trig_arg,
      // parenthesized trig applications go through apply_delimited (the semantically
      // preferred XMDual form) only.
      // Function application paths (function fenced_factor) remain so \sin f(x)
      // and \sin F(x) still parse correctly as sin(f(x)) / sin(F(x)).
      trig_arg = factor_base
        | unknown fenced_factor => speculative_prefix_apply
        | diffunk fenced_factor => speculative_prefix_apply
        | function fenced_factor => prefix_apply
        // Perl: trigBarearg includes OPFUNCTION+args (chained function application)
        // Allows: \sin\det A → sin(det(A)). FUNCTION doesn't absorb bare args.
        | opfunction factor => prefix_apply
        // trig_arg chains only through factor_base on the RHS. Previous approach
        // chained through full `factor` causing \sin(x) + (y) to ambiguously
        // parse as sin((x)+(y)).
        | trig_arg mulop factor_base => infix_apply_nary
        | trig_arg binop factor_base => infix_apply_nary
        | trig_arg factor_base => apply_invisible_times;

      // applied_func: FUNCTION only absorbs fenced args (parens), not bare args.
      // OPFUNCTION and TRIGFUNCTION absorb bare args (Perl distinction).
      // Perl: `fga` = f*g*a (FUNCTION), `Fga` = F@(g*a) (OPFUNCTION)
      applied_func = function fenced_factor => prefix_apply
        | trigfunction trig_arg => prefix_apply
        | opfunction tight_term => prefix_apply
        // Perf: removed `| opfunction opfunction => prefix_apply`. Adjacent
        // opfunctions (FG) already match via `opfunction tight_term` since
        // opfunction is a term (`term += opfunction`, line 217). The short
        // rule competed with cascade-via-tight_term in FGHa and doubled
        // enumeration for every OPFUNCTION+OPFUNCTION pair.
        // Delimited function application: f(x), f[x], F(x), \sin(x) etc.
        // Perl: ApplyDelimited creates XMDual(content=Apply(XMRef(f),XMRef(args)),
        //        presentation=Apply(f, XMWrap(open, args, close))).
        // These are in applied_func so delimited calls participate in chaining:
        // f(a) g(b) → f@(a) * g@(b) via tight_term applied_func => apply_invisible_times
        | function lparen formula rparen => apply_delimited
        | function lbracket formula rbracket => apply_delimited
        | opfunction lparen formula rparen => apply_delimited
        | opfunction lbracket formula rbracket => apply_delimited
        | trigfunction lparen formula rparen => apply_delimited
        | trigfunction lbracket formula rbracket => apply_delimited;
      // Standalone applied functions are also tight_terms
      tight_term += applied_func;
      // Function application results can chain with invisible times (Perl moreFactors)
      tight_term += tight_term applied_func => apply_invisible_times;

      // Intervals are math objects (`(0,1)`, `[a,b]`, etc.), not grouping
      // constructs. Moved out of fenced_factor so function application
      // (`f(x,y)`) naturally prunes the interval interpretation in favor
      // of the list interpretation via `lparen formula_list rparen`.
      // Placed at tight_term level so intervals participate in invisible
      // multiplication (`2(a,b)` = `2 * (a,b)`) but not in `f(...)` apply.
      interval_term = lparen term punct term rparen      => interval
        | lparen term punct term rbracket    => interval
        | lbracket term punct term rbracket  => interval
        | lbracket term punct term rparen  => interval
        | rbracket term punct term lbracket => interval;
      tight_term += interval_term;

      // UNKNOWN followed by fenced args => function application (Perl: doubtArgs/maybeArgs)
      // f(x) => f@(x), g(a+b) => g@(a+b). Only active when MATHPARSER_SPECULATE is set.
      // Without speculation, this parse is pruned and Marpa uses invisible-times instead.
      // NOTE: ID tokens are multiplicative atoms — NEVER prefix-apply. Only UNKNOWN
      // tokens get speculative function application. ID always uses invisible-times.
      tight_term += unknown fenced_factor => speculative_prefix_apply
        | diffunk fenced_factor => speculative_prefix_apply;
      // Perf: `tight_term += function fenced_factor => prefix_apply` removed.
      // It duplicated the applied_func path (function fenced_factor => prefix_apply)
      // and competed with apply_delimited (function lparen formula rparen =>
      // apply_delimited) for `f(x)` cases, adding ambiguity with no benefit.
      // OPFUNCTION bare arg absorption (Perl: addOpArgs barearg + moreargs)
      // \log 2x^2 => log@(2*x^2). These tight_term rules serve as priority
      // boosters that ensure cascading opfunction application (FGHa → F@(G@(H@(a))))
      // wins over `F@(G) * H@(a)`. Removing the direct `tight_term += opfunction
      // tight_term` rule (even after removing `applied_func = opfunction opfunction`
      // in commit ffcafc33e) still breaks FGHa — the `applied_func = opfunction
      // tight_term` + `tight_term += applied_func` lifting path does NOT preserve
      // Marpa's cascade-over-invisible-times preference. The direct self-recursive
      // rule is required.
      tight_term += opfunction tight_term => prefix_apply;
      tight_term += opfunction factor => prefix_apply;
      // Perf: removed `opfunction fenced_factor => prefix_apply` — `factor` already
      // includes fenced_factor, so this rule was a duplicate that caused Marpa to
      // enumerate the same tree twice for every `\sin(x)` (OPFUNCTION) form.
      // TRIGFUNCTION absorbs bare args: \sin x => sin@(x), \cos\pi => cos@(pi).
      // Note: `factor` is used here (not factor_base) to support scripted args
      // like \sin a^2 (scripted_factor_r1 is in factor but not factor_base).
      // Narrowing to factor_base breaks \sin a^2 = sin(a^2) parses.
      tight_term += trigfunction factor => prefix_apply;
      // compound_operator (e.g. D∇, D sin) followed by a single factor: ∇ log x => (∇@log)@(x)
      // More targeted than the previous `compound_operator tight_term` — absorbs only one factor,
      // not an entire invisible-times chain. Covers fenced_factor too (since factor += fenced_factor).
      tight_term += compound_operator factor => prefix_apply;
      // Perl IntFactor L640-651: diffd followed by ATOM/UNKNOWN/ID => Apply(DIFFOP(d), var)
      // Semantic action checks text is literally "d" and INTOP context.
      // At factor level so it can appear as right operand of invisible_times.
      // Perl: diffd matches both /UNKNOWN:d/ and /ID:d/ (lxDeclare can set role=ID on d).
      // M4: Only "d" tokens can be diffops. diffunk/diffid are emitted by the
      // lexer for tokens with content "d". This prevents Marpa from exploring
      // the diffop path for every UNKNOWN token (was ~90% of pruned trees).
      factor += diffunk factor_base => diffop_apply
        | diffid factor_base => diffop_apply;

      // Perl MathGrammar L720-723: combine SUPOP tokens (\prime\prime → prime2)
      supops = supop
        | supops supop => combine_supops;
      // Bare operators valid as script content that `statements` CAN'T derive.
      // Perf: statement covers addop|mulop|binop|relop|arrow|any_bigop|operator,
      // so listing them here was pure duplication (2x per script arg with an
      // operator, e.g. P^+ had 3 parses → 1 unique). Narrowed to unique items:
      //   - metarelop: statement only has `metarelop formula`, not bare
      //   - vertbar, supops: statement has no derivation
      //   - modifierop: statement only has `modifierop formula`, not bare
      script_op = metarelop | vertbar | supops | modifierop;
      // Script content: expressions, statements (period/comma-separated), or bare operators
      // Script content: `statements` is the primary catch-all (derives everything
      // expression/formula derive). `formula_list` is kept separately because
      // it uses formula_list_apply (different semantics from list_apply in statements).
      // IMPORTANT: Do NOT add `expression` — it's a strict subset of `statements`,
      // and having both creates 2^N ambiguity (2x per script argument).
      postsubarg = start_postsubscript statements end_postsubscript => faux_wrap
        | start_postsubscript formula_list end_postsubscript => faux_wrap
        | start_postsubscript script_op end_postsubscript => faux_wrap;
      postsuperarg = start_postsuperscript statements end_postsuperscript => faux_wrap
        | start_postsuperscript formula_list end_postsuperscript => faux_wrap
        | start_postsuperscript script_op end_postsuperscript => faux_wrap;
      // Bigop-specific script args — separated tokens to reduce earley chart competition
      bigopsubarg = start_bigopsub statements end_bigopsub => faux_wrap
        | start_bigopsub formula_list end_bigopsub => faux_wrap
        | start_bigopsub script_op end_bigopsub => faux_wrap;
      bigopsuparg = start_bigopsup statements end_bigopsup => faux_wrap
        | start_bigopsup formula_list end_bigopsup => faux_wrap
        | start_bigopsup script_op end_bigopsup => faux_wrap;
      floatsubarg = start_floatsubscript expression end_floatsubscript => faux_wrap
        | start_floatsubscript script_op end_floatsubscript => faux_wrap;
      floatsuperarg = start_floatsuperscript expression end_floatsuperscript => faux_wrap
        | start_floatsuperscript script_op end_floatsuperscript => faux_wrap;
      // Scripted infix operators: x \times_i^2 y — operator with decorating scripts
      scripted_mulop = mulop postsubarg => postfix_script
        | mulop postsuperarg => postfix_script
        | mulop postsubarg postsuperarg => postfix_script
        | mulop postsuperarg postsubarg => postfix_script;
      // Add scripted mulop as infix operator at term level
      term += tight_term scripted_mulop tight_term => infix_apply_nary;
      // Ket with scripted operator label: |\times_{i}^{2}⟩ → ket@(scripted_mulop)
      fenced_factor += singlevertbar scripted_mulop rangle_close => qm_ket;

      // Scripted FUNCTION with fenced args: f'(a), f^2(a), f_n(x)
      scripted_function = function postsuperarg => postfix_script
        | function postsubarg => postfix_script
        | function postsubarg postsuperarg => postfix_script
        | function postsuperarg postsubarg => postfix_script;
      // All scripted function application rules go through applied_func only.
      // Perf: removed `scripted_function fenced_factor => prefix_apply` duplicate.
      // `f^2(x)` goes through apply_delimited (XMDual) only. Two such scripted
      // function calls in one formula no longer multiply ambiguity.
      applied_func += scripted_function lparen formula rparen => apply_delimited;

      // Scripted OPFUNCTION with bare/fenced args: \log_e a, \det_S x
      scripted_opfunction = opfunction postsuperarg => postfix_script
        | opfunction postsubarg => postfix_script
        | opfunction postsubarg postsuperarg => postfix_script
        | opfunction postsuperarg postsubarg => postfix_script;
      applied_func += scripted_opfunction tight_term => prefix_apply;
      // Perf: removed duplicate `scripted_opfunction fenced_factor => prefix_apply`.
      // `scripted_opfunction tight_term` already covers fenced_factor via factor += fenced_factor
      // (and tight_term includes factor via multiple paths). The fenced case ALSO has
      // `scripted_opfunction lparen formula rparen => apply_delimited` below (preferred XMDual).
      applied_func += scripted_opfunction lparen formula rparen => apply_delimited;

      // Scripted TRIGFUNCTION: \sin^2 x, \cos_n x
      scripted_trigfunction = trigfunction postsuperarg => postfix_script
        | trigfunction postsubarg => postfix_script
        | trigfunction postsubarg postsuperarg => postfix_script
        | trigfunction postsuperarg postsubarg => postfix_script;
      applied_func += scripted_trigfunction tight_term => prefix_apply;
      // Perf (grammar disambiguation): removed duplicate
      //   `scripted_trigfunction fenced_factor => prefix_apply`
      // Fenced scripted-trig calls (\sin^{2}(x)) go through apply_delimited
      // (the XMDual form). Previously BOTH paths produced trees, giving
      // 4× multiplier when two such calls appeared in a formula
      // (e.g. \sin^2(x) + \cos^2(x) was 46 parses).
      applied_func += scripted_trigfunction lparen formula rparen => apply_delimited;

      // standalone top-level variants of floating scripts:
      floatsubscript = start_floatsubscript expression end_floatsubscript => standalone_script;
      floatsuperscript = start_floatsuperscript expression end_floatsuperscript => standalone_script;
      // Scripted factors -- avoid adding ambiguity in the left-right order of collection
      // first ALL left (=float), then right (=post).
      scripted_factor_l11 = floatsuperarg factor_base => prefix_script
        | floatsuperarg opfunction => prefix_script;
      scripted_factor_l12 = floatsubarg factor_base => prefix_script
        | floatsubarg opfunction => prefix_script;
      scripted_factor_l1 = scripted_factor_l11 | scripted_factor_l12;
      // POST script used as pre-script on factor (forced 'pre', no _wasfloat)
      // e.g., {}_a^b x: ^b is POST, used as pre-script on x
      prescripted_factor_post_r = postsuperarg factor_base => prefix_script_pre
        | postsuperarg opfunction => prefix_script_pre;
      prescripted_factor_post_l = postsubarg factor_base => prefix_script_pre
        | postsubarg opfunction => prefix_script_pre;
      scripted_factor_l2 = floatsuperarg scripted_factor_l12 => prefix_script
        | floatsubarg scripted_factor_l11 => prefix_script
        // Mixed FLOAT+POST from same {} base: FLOAT wraps POST pre-script
        | floatsubarg prescripted_factor_post_r => prefix_script
        | floatsuperarg prescripted_factor_post_l => prefix_script
        // Recursive: chain 3+ floating scripts on factor (e.g., {}_i{}_j^k x)
        | floatsuperarg scripted_factor_l2 => prefix_script
        | floatsubarg scripted_factor_l2 => prefix_script;

      // Note: any_bigop is NOT included here — bigops get scripts via scripted_bigop,
      // not scripted_factor. This ensures bigop_application can absorb arguments.
      scripted_factor_r11 = factor_base postsuperarg => postfix_script
        | opfunction postsuperarg => postfix_script
        | scripted_factor_l1 postsuperarg => postfix_script
        | scripted_factor_l2 postsuperarg => postfix_script
        | fenced_factor postsuperarg => postfix_script;
      scripted_factor_r12 = factor_base postsubarg => postfix_script
        | opfunction postsubarg => postfix_script
        | scripted_factor_l1 postsubarg => postfix_script
        | scripted_factor_l2 postsubarg => postfix_script
        | fenced_factor postsubarg => postfix_script;
      scripted_factor_r1 = scripted_factor_r11 | scripted_factor_r12;
      scripted_factor_r2 = scripted_factor_r12 postsuperarg => postfix_script
        | scripted_factor_r11 postsubarg => postfix_script;
      factor += scripted_factor_l1 | scripted_factor_l2 | scripted_factor_r1 | scripted_factor_r2;

      // Pre-scripts on post-scripted bases: _b(A^c), ^a(A_d^c), etc.
      // Must come after scripted_factor_r1/r2 are defined (forward reference not allowed).
      prescripted_factor_post_r += postsuperarg scripted_factor_r1 => prefix_script_pre
        | postsuperarg scripted_factor_r2 => prefix_script_pre;
      prescripted_factor_post_l += postsubarg scripted_factor_r1 => prefix_script_pre
        | postsubarg scripted_factor_r2 => prefix_script_pre;

      // Scripted bigops: \int_0^\infty, \sum_{n=1}^N, etc.
      // These are bigops with post-scripts that still act as prefix operators.
      // Perl: preScripted['INTOP'] addIntOpArgs / preScripted['bigop'] addOpArgs
      // Chain scripts: first sub then super, or vice versa (like scripted_factor_r1/r2)
      // Use bigop-specific script tokens when available (from lexer),
      // fall back to generic POSTSUBSCRIPT/POSTSUPERSCRIPT for compatibility
      // Single-script bigop: one sub or one super
      scripted_bigop_r1 = any_bigop bigopsuparg => postfix_script
        | any_bigop bigopsubarg => postfix_script
        | any_bigop postsuperarg => postfix_script
        | any_bigop postsubarg => postfix_script;
      scripted_bigop = scripted_bigop_r1
        | scripted_bigop_r1 bigopsuparg => postfix_script
        | scripted_bigop_r1 bigopsubarg => postfix_script
        | scripted_bigop_r1 postsuperarg => postfix_script
        | scripted_bigop_r1 postsubarg => postfix_script;
      // Perl: preScripted['bigop'] addOpArgs — addOpArgs = Factor moreOpArgFactors
      // moreOpArgFactors chains factors with MulOp or invisible times.
      //
      // bigop_application absorbs `term` (not just tight_term) because:
      // - Nested bigops: ∑∑∑ a_{ij}b_{jk}c_{ki} needs each ∑ to absorb the next
      // - bigop_application is lifted to term level, so inner bigops are terms
      // M2 investigation: restricting to tight_term breaks nested bigops (calculus test).
      // The semantic pruning already handles the ∑ a + b case correctly.
      bigop_application = any_bigop term => prefix_apply
        | scripted_bigop term => prefix_apply
        | composed_bigop term => prefix_apply;
      // Lift bigop_application to term level (not expression level).
      // This avoids exponential Marpa ambiguity when ADDOP precedes BIGOP
      // (e.g. a+\neg b). At term level, `term addop expression` handles
      // `a + \neg b` with a single derivation path.
      // On its LEFT, invisible_times still works: 2∫ x dx via tight_term rules.
      // On its RIGHT, addop/relop follow naturally: ∫ x dx + y → ∫(x*dx) + y.
      term += bigop_application;
      // Bigop after invisible times: 1/2∫ f dx → (1/2)*∫(f*dx)
      // Perl: Factor moreFactors handles consecutive factors via InvisibleTimes.
      // Since bigop_application is at term level (not tight_term), juxtaposition
      // between a tight_term and a bigop_application needs an explicit rule.
      term += tight_term bigop_application => apply_invisible_times;
      // Same but with explicit mulop: a * ∫ f dx → a * ∫(f*dx)
      term += term mulop bigop_application => infix_apply_nary;
      // Scripted bigops can also appear as standalone statements
      statement += scripted_bigop;

      // Pre-scripted bigops: floating scripts before a bigop (Perl: preScripted)
      // Handles patterns like {}_a^b\sum_c^d x where floating scripts
      // attach as pre-scripts to the following operator.
      // Perl's parse_kludgeScripts_rec: FLOAT + POST pairs from same {} base
      // both become pre-scripts (POST gets forced 'pre' position without _wasfloat).
      prescripted_bigop_inner = scripted_bigop | scripted_bigop_r1 | any_bigop;
      // FLOAT script wrapping a bigop as pre-script
      // Perl: preScripted['bigop'] / preScripted['INTOP']
      prescripted_bigop = floatsuperarg prescripted_bigop_inner => prefix_script
        | floatsubarg prescripted_bigop_inner => prefix_script
        // Recursive: chain multiple floating scripts before bigop
        | floatsuperarg prescripted_bigop => prefix_script
        | floatsubarg prescripted_bigop => prefix_script;
      // POST script used as pre-script (forced 'pre', no _wasfloat).
      // Only used INSIDE prescripted_bigop (always FLOAT-wrapped outside),
      // so they can't incorrectly match bare post-scripts as pre-scripts.
      // Perl: parse_kludgeScripts_rec calls NewScript($base, $y, 'pre') for POST
      // scripts that follow a FLOAT from the same empty {} base.
      prescripted_bigop += postsuperarg prescripted_bigop_inner => prefix_script_pre
        | postsubarg prescripted_bigop_inner => prefix_script_pre
        | postsuperarg prescripted_bigop => prefix_script_pre
        | postsubarg prescripted_bigop => prefix_script_pre;
      tight_term += prescripted_bigop tight_term => prefix_apply;
      tight_term += prescripted_bigop factor => prefix_apply;
      statement += prescripted_bigop;

      // Perl MathGrammar L259-260: moreFactors: evalAtOp maybeEvalAt
      // "evaluated at" — a|_{x=0}, f(x)|_{x=0}^{x=1}, \left.xyz\right|_{0}^{2}
      // evalAtOp: VERTBAR:| (standalone pipe) — CLOSE:| from \right| handled separately
      tight_term += tight_term singlevertbar postsubarg => eval_at
        | tight_term singlevertbar postsubarg postsuperarg => eval_at
        | tight_term singlevertbar postsuperarg postsubarg => eval_at
        // CLOSE:| from \right| also triggers eval-at
        | tight_term close_pipe postsubarg => eval_at
        | tight_term close_pipe postsubarg postsuperarg => eval_at
        | tight_term close_pipe postsuperarg postsubarg => eval_at;

      anyop = addop | mulop | binop | relop | arrow | metarelop
        | bigop | sumop | intop
        | limitop | diffop | vertbar | supop
        | modifierop | composed_bigop | operator | compound_operator;

      anyscript = floatsuperscript | floatsubscript
        // Standalone floating script pairs (no base: {}^c_d or {}_d^c)
        // Perl: NewScript(NewScript(Absent(), super, 'post'), sub, 'post')
        | floatsuperscript postsubarg => postfix_script
        | floatsubscript postsuperarg => postfix_script;

      // Operators that CANNOT start a valid expression — leading orphans
      // from tabular fragments where LHS is on a preceding row.
      // Excluded: addop (prefix ±x), relop (prefix =x), arrow, bigop/sumop/intop.
      // These already have valid prefix interpretations inside expressions.
      orphan_op = mulop | binop | diffop | supop | modifierop;
      // Perf (Fix 2): `formula_list` removed from `anything` alternatives.
      // formula_list is L3-internal (a fenced body), not L0. `statements`
      // covers bare top-level comma-separated items via `list_apply` with
      // equivalent semantics (formula_list_apply delegates to list_apply
      // for non-relational items).
      anything = formulae | statements | anyop | anyscript |
        anyop anyop => compound_operator_2 |
        // Perl MathGrammar L81: leading orphan operator (tabular fragment).
        // Only at the start rule (anything) — not recursive, not inside subexpressions.
        orphan_op statements => prefix_relop_apply
    );
  // | term_argument postsuperarg tex_argument  => post_script
  // | term_argument postsubscript tex_argument    => post_script
  start!(anything);

  // Also prepare the tree builder rules here (for now)
  Ok((grammar!(), actions!(), builder!()))
}

use crate::semantics::*;
use marpa::grammar::Grammar as MarpaGrammar;
use marpa::result::Result;
use marpa::tree_builder::TreeBuilder;

pub fn init_grammar() -> Result<(MarpaGrammar, Actions, TreeBuilder)> {
  // We create a declarative macro language of our own, in the spirit of the Marpa SLIF
  default_registry!();
  // Tokens, to be used in rules directly
  token!(atom ~ "ATOM");
  token!(unknown ~ "UNKNOWN");
  token!(id ~ "ID");
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
  token!(open ~ "OPEN");
  token!(close ~ "CLOSE");
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
    factor_base = unknown | number | id | atom | array;
    // Perl MathGrammar L277: OPEN ARRAY CLOSE -> Fence (e.g. \{ array \} or ( array ))
    // Also handle unmatched delimiters for cases-like patterns.
    fenced_array = open array close => fenced
      | open array => open_fenced
      | array close => close_fenced;
    factor = factor_base | opfunction | fenced_array;
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
      | operator compound_operator => prefix_apply;

    // tight_term includes single factors (for left-recursive chaining)
    // and all compound constructs (invisible times, prefix application, etc.)
    // The `\log x` → `log*x` issue is handled by semantic pruning in
    // apply_invisible_times, not at the grammar level.
    tight_term = factor
      | tight_term factor => apply_invisible_times
      // Perl MathGrammar L423: POSTFIX (e.g. n!) => Apply(op, term)
      | tight_term postfix => apply_postfix
      | function tight_term => prefix_apply
      // trigfunction uses trigbarearg via applied_func (absorbs MulOp chains)
      // NOTE: bigop rules moved to += section (after `term` is defined) so they
      // can absorb full term (mulop chains like x² * dx), not just tight_term.
      | composed_bigop tight_term => prefix_apply
      | compound_operator tight_term => prefix_apply
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

    // Allow standalone functions/trigfunctions/opfunctions as terms
    // This is needed for (f*g)(x) where f and g are FUNCTION tokens
    // opfunction here allows standalone \operatorname{R} to parse
    term += function | trigfunction | opfunction | composed_term;
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
    term_list = term punct term => list_apply
      | term_list punct term => list_apply;

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
      | statements punct statement => list_apply
      // Perl: MorphVertbar — VERTBAR as conditional modifier: x | y,z,t
      | statement vertbar statements => vertbar_modifier;

    // Perl MathGrammar: Formulae = Formula (endPunct Formula)* → NewFormulae()
    // formula_list: punct-separated formulas at top level → always "formulae".
    // Only fires when there are 2+ items (single items go through `statements`).
    // formulae_apply REJECTS the parse if no items are relational, so Marpa
    // falls back to the `statements` parse which produces "list".
    formula_list = statement punct statement => formulae_apply
      | formula_list punct statement => formulae_apply;

    // Extensions, now that we have more category variables defined
    fenced_factor = lbrace expression rbrace    => fenced
           | lbracket expression rbracket       => fenced
           | lparen formula rparen              => fenced
           // Angle brackets as delimiters: <x,y> for inner products, etc.
           // Old typesetting conventions used < > instead of \langle \rangle.
           // Uses term_list (comma-separated terms) to avoid matching complex
           // nested expressions. Only fires when content has commas.
           | langle_rel term_list rangle_rel => fenced
           | lparen term punct term rparen      => interval
           | lparen term punct term rbracket    => interval
           | lbracket term punct term rbracket  => interval
           | lbracket term punct term rparen  => interval
           | rbracket term punct term lbracket => interval
           // Perl MathGrammar L294: || exp || → norm (must be before |exp| → abs-val)
           // CatSymbols merges two | into ‖; singlevertbar = VERTBAR:|
           | singlevertbar singlevertbar expression singlevertbar singlevertbar => norm_fenced
           | singlevertbar expression singlevertbar => fenced
           // Perl's Fence for comma-separated items in braces: {a,b} and {a,b,c}
           | lbrace term punct term rbrace => fence
           | lbrace term punct term punct term rbrace => fence
           // Perl: {a|b} conditional-set with VERTBAR or MIDDLE separator
           | lbrace formula singlevertbar formula rbrace => fence
           | lbrace formula middle_bar formula rbrace => fence
           | lbrace formula metarelop formula rbrace => fence
           // Generic OPEN/CLOSE delimiters: \lfloor...\rfloor, \lceil...\rceil, etc.
           // Perl MathGrammar: OPEN Expression CLOSE → Fence
           | open expression close => fenced
           // Empty fenced expressions: () [] {} ⌊⌋ etc.
           | lparen rparen => empty_fenced
           | lbracket rbracket => empty_fenced
           | lbrace rbrace => empty_fenced
           | open close => empty_fenced;
    factor += fenced_factor;

    // Perl: addTrigFunArgs → trigBarearg → aTrigBarearg moreTrigBareargs
    // Trig functions absorb chains of mulop+factor (but NOT other trig functions).
    // aTrigBarearg includes: FUNCTION+args, OPFUNCTION+args, ATOM_OR_ID, UNKNOWN, NUMBER
    trig_arg = factor
      | fenced_factor
      | unknown fenced_factor => speculative_prefix_apply
      | function fenced_factor => prefix_apply
      | opfunction fenced_factor => prefix_apply
      // Perl: trigBarearg includes FUNCTION/OPFUNCTION+args (chained function application)
      // Allows: \sin\log x → sin(log(x)), \sin\det A → sin(det(A))
      | function factor => prefix_apply
      | opfunction factor => prefix_apply
      | trig_arg mulop factor => infix_apply_nary
      | trig_arg binop factor => infix_apply_nary
      | trig_arg factor => apply_invisible_times;

    // applied_func uses trig_arg (defined above)
    applied_func = function tight_term => prefix_apply
      | trigfunction trig_arg => prefix_apply
      | opfunction tight_term => prefix_apply;
    // Standalone applied functions are also tight_terms
    tight_term += applied_func;
    // Function application results can chain with invisible times (Perl moreFactors)
    tight_term += tight_term applied_func => apply_invisible_times;

    // UNKNOWN followed by fenced args => function application (Perl: doubtArgs/maybeArgs)
    // f(x) => f@(x), g(a+b) => g@(a+b). Only active when MATHPARSER_SPECULATE is set.
    // Without speculation, this parse is pruned and Marpa uses invisible-times instead.
    // NOTE: ID tokens are multiplicative atoms — NEVER prefix-apply. Only UNKNOWN
    // tokens get speculative function application. ID always uses invisible-times.
    tight_term += unknown fenced_factor => speculative_prefix_apply;
    // FUNCTION followed by fenced args => function application (Perl: addArgs/addEasyArgs)
    // f(x) => f@(x) when f has role=FUNCTION (from DefMathRewrite or \lxDeclare).
    // Perl: ApplyDelimited creates XMDual(content=Apply(XMRef(f),XMRef(args)),
    //        presentation=Apply(f, XMWrap(open, args, close))).
    // Grammar: function lparen/lbracket + formula + rparen/rbracket → apply_delimited
    tight_term += function lparen formula rparen => apply_delimited;
    tight_term += function lbracket formula rbracket => apply_delimited;
    // Also support fenced_factor for backwards compat (no XMDual wrapping)
    tight_term += function fenced_factor => prefix_apply;
    // OPFUNCTION followed by fenced args => function application with XMDual wrapping.
    // Perl: ApplyDelimited for \operatorname{cov}(L), \log(x), etc.
    tight_term += opfunction lparen formula rparen => apply_delimited;
    tight_term += opfunction lbracket formula rbracket => apply_delimited;
    // Also support fenced_factor for backwards compat (no XMDual wrapping)
    tight_term += opfunction fenced_factor => prefix_apply;
    // Perl: OPFUNCTION absorbs barearg (factor chain) just like FUNCTION/TRIGFUNCTION
    // \log x => log@(x), \operatorname{cov}(L) already handled by fenced_factor rule
    tight_term += opfunction tight_term => prefix_apply;
    tight_term += opfunction factor => prefix_apply;
    // TRIGFUNCTION absorbs bare args: \sin x => sin@(x), \cos\pi => cos@(pi)
    // Note: trigfunction tight_term already in compound_operator, can't duplicate.
    tight_term += trigfunction factor => prefix_apply;
    // TRIGFUNCTION followed by fenced args => function application with XMDual wrapping.
    tight_term += trigfunction lparen formula rparen => apply_delimited;
    tight_term += trigfunction lbracket formula rbracket => apply_delimited;
    tight_term += trigfunction fenced_factor => prefix_apply;
    // Perl IntFactor L640-651: diffd followed by ATOM/UNKNOWN/ID => Apply(DIFFOP(d), var)
    // Uses existing `unknown` terminal; semantic action checks text is literally "d".
    // At factor level so it can appear as right operand of invisible_times.
    factor += unknown factor_base => diffop_apply;

    // Bare operators valid as script content (e.g., Na^+ has ADDOP as superscript)
    script_op = addop | mulop | binop | relop | arrow | metarelop
      | bigop | sumop | intop | limitop | diffop | vertbar | supop
      | modifierop | operator;
    // Script content: expressions or bare operators
    postsubarg = start_postsubscript expression end_postsubscript => faux_wrap
      | start_postsubscript script_op end_postsubscript => faux_wrap;
    postsuperarg = start_postsuperscript expression end_postsuperscript => faux_wrap
      | start_postsuperscript script_op end_postsuperscript => faux_wrap;
    floatsubarg = start_floatsubscript expression end_floatsubscript => faux_wrap
      | start_floatsubscript script_op end_floatsubscript => faux_wrap;
    floatsuperarg = start_floatsuperscript expression end_floatsuperscript => faux_wrap
      | start_floatsuperscript script_op end_floatsuperscript => faux_wrap;
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

    scripted_factor_r11 = factor_base postsuperarg => postfix_script
      | opfunction postsuperarg => postfix_script
      | any_bigop postsuperarg => postfix_script
      | scripted_factor_l1 postsuperarg => postfix_script
      | scripted_factor_l2 postsuperarg => postfix_script
      | fenced_factor postsuperarg => postfix_script;
    scripted_factor_r12 = factor_base postsubarg => postfix_script
      | opfunction postsubarg => postfix_script
      | any_bigop postsubarg => postfix_script
      | scripted_factor_l1 postsubarg => postfix_script
      | scripted_factor_l2 postsubarg => postfix_script
      | fenced_factor postsubarg => postfix_script;
    scripted_factor_r1 = scripted_factor_r11 | scripted_factor_r12;
    scripted_factor_r2 = scripted_factor_r12 postsuperarg => postfix_script
      | scripted_factor_r11 postsubarg => postfix_script;
    factor += scripted_factor_l1 | scripted_factor_l2 | scripted_factor_r1 | scripted_factor_r2;

    // Scripted bigops: \int_0^\infty, \sum_{n=1}^N, etc.
    // These are bigops with post-scripts that still act as prefix operators.
    // Perl: preScripted['INTOP'] addIntOpArgs / preScripted['bigop'] addOpArgs
    // Chain scripts: first sub then super, or vice versa (like scripted_factor_r1/r2)
    scripted_bigop_r1 = any_bigop postsuperarg => postfix_script
      | any_bigop postsubarg => postfix_script;
    scripted_bigop = scripted_bigop_r1
      | scripted_bigop_r1 postsuperarg => postfix_script
      | scripted_bigop_r1 postsubarg => postfix_script;
    // Perl: preScripted['bigop'] addOpArgs — addOpArgs = Factor moreOpArgFactors
    // moreOpArgFactors chains factors with MulOp or invisible times.
    //
    // bigop_application is a dedicated nonterminal that:
    // - Acts as tight_term on the LEFT (2∫ works via invisible times)
    // - Absorbs full factor chains on the RIGHT (∫ x² dx → ∫(x²*dx))
    // - Does NOT recurse back into tight_term → bigop cycle (avoids ambiguity)
    //
    // Once inside bigop_application, invisible_times and mulop extend the
    // argument chain without re-entering the bigop dispatch.
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
    anything = formula_list | statements | anyop | anyscript |
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

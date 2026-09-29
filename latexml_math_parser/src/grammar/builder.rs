use marpa::{grammar::Grammar as MarpaGrammar, result::Result, tree_builder::TreeBuilder};

use crate::semantics::*;

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
  // PUNCT with `\quad`-class spacing (rpadding ≥ 5pt). The lexer
  // (util.rs::punct_followed_by_wide_space) emits these as
  // `WIDE_PUNCT:,:idx` so the grammar can prefer formulae_apply
  // for the `formula , \quad condition` arXiv idiom without
  // enumerating the `list_apply` alternatives that the pragma
  // would only reject post-hoc. See docs/archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md §2.
  token!(wide_punct ~ "WIDE_PUNCT");
  token!(addop_t ~ "ADDOP");
  token!(mulop_t ~ "MULOP");
  token!(relop_t ~ "RELOP");
  token!(elideop ~ "ELIDEOP");
  token!(langle_rel = "RELOP:less-than");
  token!(langle_open = "OPEN:langle");
  token!(langle = [langle_rel langle_open]);
  token!(rangle_rel = "RELOP:greater-than");
  token!(rangle_close = "CLOSE:rangle");
  token!(rangle =[rangle_rel rangle_close]);
  token!(vertbar ~ "VERTBAR");
  token!(singlevertbar = "VERTBAR:|");
  // `\|` and `\Vert` lex to a single doubled-bar token `VERTBAR:||` (vs four
  // single `VERTBAR:|` for `||x||`). Used by the `\|x\|` norm rule below.
  token!(doublevertbar = "VERTBAR:||");
  // `\left|...\right|` produces VERTBAR tokens tagged `stretchy="true"`.
  // The lexer (util.rs) further distinguishes those emitted by `\lx@delim@left`
  // from those emitted by `\lx@delim@right` via the `lx@side` property set in
  // the respective constructors (tex_math.rs). Side-distinct lexemes
  // (LEFT_STRETCHY_VERTBAR / RIGHT_STRETCHY_VERTBAR) eliminate
  // combinatorial pairing ambiguity for the kerned-stack norm idioms
  // (\vertii, \vertiii, …) where multiple identical bars appear in
  // sequence. `stretchy_vertbar` is the union — used by legacy rules
  // (e.g. eval_at) that don't care which side a stretchy bar came from.
  // See docs/archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md §2 (delimiter-pairing) and Task #263.
  token!(left_stretchy_vertbar ~ "LEFT_STRETCHY_VERTBAR");
  token!(right_stretchy_vertbar ~ "RIGHT_STRETCHY_VERTBAR");
  // Fallback for stretchy bars that arrived without a side tag
  // (legacy DOM input, or a code path that bypassed `\lx@delim@left`/`\lx@delim@right`).
  // The lexer in util.rs emits this when `role_side` is absent.
  // Rules that legitimately accept any side (eval_at) enumerate both
  // alternatives explicitly rather than relying on a union token —
  // the Marpa tree builder doesn't roll up alternation-of-tokens cleanly.
  token!(undirected_stretchy_vertbar ~ "STRETCHY_VERTBAR");
  // A single stretchy bar, not a `\left\|` double one (`LEFT_STRETCHY_VERTBAR:||`, util.rs): the
  // kerned stacks (`stretchy_norm_fenced`) are made of single bars only — `\left|\!\left\|A\right\|
  // \!\right|` is |‖A‖|, as Perl, never a merged glyph that drops two bars (57am review round 7).
  token!(left_stretchy_single_bar = "LEFT_STRETCHY_VERTBAR:|");
  token!(right_stretchy_single_bar = "RIGHT_STRETCHY_VERTBAR:|");
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
  token!(colon_metarelop = "METARELOP:colon");
  token!(modifierop ~ "MODIFIEROP");
  token!(modifier ~ "MODIFIER");
  token!(arrow_t ~ "ARROW");
  token!(binop_t ~ "BINOP");
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
      // The operators, each extended below with its decorating scripts (`addOpDecoration`).
      relop = relop_t;
      arrow = arrow_t;
      addop = addop_t;
      mulop = mulop_t;
      binop = binop_t;
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
      // FUNCTION is a factor (participates in implicit multiplication).
      // OPFUNCTION is intentionally NOT here. Including OPFUNCTION in
      // `factor` would let `tight_term factor → apply_invisible_times`
      // admit a bare OPFUNCTION as its LEFT operand — a reading the
      // pragma `apply_invisible_times: left is OPFUNCTION/.../FUNCTION,
      // prefer prefix_apply` correctly rejects, but only after the
      // grammar has enumerated thousands of such derivations
      // (~52% of trees on complex math-heavy equations such as
      // 1911.09517 eq.993). Excluding OPFUNCTION from `factor`
      // pushes the constraint upstream into the grammar: bare
      // OPFUNCTION cannot anchor an invisible-times chain on the LEFT.
      // To keep legitimate trailing-OPFUNCTION cases (`c \not`,
      // `a b \not`, ...), the `tight_term += tight_term opfunction =>
      // apply_invisible_times` rule below admits OPFUNCTION as a
      // chain-terminating RIGHT operand. OPFUNCTION as the head of an
      // applied form (`\log x`, `\sin x`) enters via `applied_func`
      // (`opfunction group_factor`, a group; `opfunction op_bare_item|op_bare_arg`, Perl's
      // bare argument, after `op_bare_arg`).
      // An ellipsis stands among juxtaposed factors and as a MulOp/BinOp operand (user ruling
      // 2026-09-29: `\cdots` keeps ELIDEOP, divergence #3, with product rules): `a_1a_2\cdots a_n`,
      // `a\times\cdots\times b`, `x_{i_1\cdots i_k}`, as Perl's `\cdots` (an ID, math_common.pool.ltxml:479)
      // does (57bs; ~1,035 formulas in 236 A/B papers were unparsed). Not a `factor_base`: a trig or
      // operator bare argument, a limit-from and a differential take no ellipsis.
      factor = factor_base | function | fenced_array | elideop;
      // Perl: limit-from@(number, sign) — directional limits: 0+, 1-
      // A "left-only term": on the left behaves as a term (for comma lists),
      // on the right terminates at the addop (like expression-level postfix).
      limit_from_term = factor_base addop => limit_from_apply;
      // Terms
      // Perl: bigop = BIGOP | SUMOP | INTOP | LIMITOP | DIFFOP
      any_bigop = bigop | sumop | intop | limitop | diffop;
      // Adjacent bigops apply in turn, as Perl's `Factor : preScripted['bigop'] addOpArgs`
      // (MathGrammar:292) does: `\partial\partial f` is ∂@(∂@(f)) through `bigop_operand`
      // (a Rust-only `composed_bigop` gave (∂@∂)@(f); golden
      // tests/parse/bigop_operands.tex#stacked_bigops_apply_in_turn).

      // Compound operators: OPERATOR composed with functions/other operators (right-recursive)
      // D sin => Apply(D, sin), D D sin => Apply(D, Apply(D, sin)), and D D => Apply(D, D):
      // Perl `nestOperators` (MathGrammar:663-671) nests operators until a function, and
      // `recApply` (MathParser.pm:1313-1315) applies each to the rest — `\nabla\nabla f` is
      // (∇@∇)@(f).
      compound_operator = operator trigfunction => prefix_apply
        | operator function => prefix_apply
        | operator opfunction => prefix_apply
        | operator operator => prefix_apply
        | operator compound_operator => prefix_apply;

      // tight_term includes single factors (for left-recursive chaining)
      // and all compound constructs (invisible times, prefix application, etc.)
      // The `\log x` → `log*x` issue is handled by semantic pruning in
      // apply_invisible_times, not at the grammar level.
      tight_term = factor
        | tight_term factor => factor_product
        // Perl MathGrammar L423: POSTFIX (e.g. n!) => Apply(op, term)
        | tight_term postfix => apply_postfix
        // Note: FUNCTION does NOT absorb bare args — only parens or APPLYOP.
        // `fga` = f*g*a, but `f(a)` = f@(a). OPFUNCTION absorbs: `Fga` = F@(g*a).
        // FUNCTION only chains via opfunction's factor status or APPLYOP.
        // trigfunction uses trigbarearg via applied_func (absorbs MulOp chains)
        // NOTE: bigop rules moved to += section (after `term` is defined) so they
        // can absorb full term (mulop chains like x² * dx), not just tight_term.
        // An operator applied to its argument is `op_application`, below.
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
      // A trig function's bare argument is `trig_arg` (below): Perl's greedy `trigBarearg`
      // (MathGrammar:340-357), MulOp chains included (`\sin x\cdot y` sin@(x·y)), ended only by
      // explicit space or a differential `d` (#367).
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
      // an operator as a term, `D - 1`, `D + G`, is `bare_op_term` (below)
      term += function | trigfunction | opfunction | composed_term;
      // (An ellipsis in a sum, `y + i + \cdots + y_n`, is a term through `factor`.)

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
        // Perl MathGrammar:246 `SignedTerm : AddOp Term`: the sign takes a whole term — a mulop
        // chain (`-a/b`, `x^{-1/2}`) or a bigop application (`-J\sum_{ij}…`, `-\int f`). A
        // `tight_term` left those with no derivation (RUST-ONLY; golden
        // tests/parse/bigop_operands.tex#signed_term_is_a_whole_term).
        | addop term => prefix_apply
        // A trailing sign, `0+`, `a^2+` (a factor is an expression: one rule, 57bi)
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
        // Perl's `relop` includes the arrows (MathGrammar:713 `ARROW addOpDecoration`), which this
        // grammar keeps apart: `\mathrm{prob}(\rightarrow j^*)` is annotated(prob, absent → j^*)
        // (2605.13374 S2.E12; golden
        // tests/parse/decorated_relations.tex#arrow_operand_in_parentheses_parses).
        | expression lparen arrow expression rparen => annotated_fenced_modifier
        // Perl MathGrammar L223: PUNCT? OPEN relop Expression CLOSE
        // Semicolon annotation: a;(<e) → annotated(a, absent < e)
        | expression punct lparen relop expression rparen => annotated_punct_fenced_modifier
        | expression punct lparen modifierop expression rparen => annotated_punct_fenced_modifier
        | expression punct lparen arrow expression rparen => annotated_punct_fenced_modifier;

      // Formula
      // Perl MathGrammar L73/236: MODIFIEROP Expression => Apply(mod, Absent, expr)
      modifier_expression = modifierop expression => modifier_prefix_apply;
      // Perl: within a Formula, comma-separated expressions after a relop form a list RHS.
      // e.g. a=b,c,d → a = list(b,c,d), not list(a=b, c, d).
      // Uses formula_list_apply which rejects items containing relops (those belong at statement level).
      formula_list = expression punct expression => formula_list_apply
        | formula_list punct expression => formula_list_apply;
      // A colon list between delimiters — an index `[G:H]`, a range `[1:t]`, a ratio `(x:y:z)`, a
      // projective point `[x_0:\dots:x_n]` — is a `list`, as `(a:b)` is (#366; Perl `delimited-[]@(a colon b)`,
      // a metarelation, MathGrammar:69, :118-125). Bracket items are expressions (`[k+1:d]`); paren items
      // are terms, three or more (two is `lparen formula metarelop expression rparen`): a sum between
      // colons in parens is a double contraction, `(\nabla x:\nabla y-\nabla z:\nabla w)` (2605.21445,
      // 2605.01156, 2605.09779), no list (57bm; 2605.14715, 2605.08808, 2605.01646, 2605.25087, 2605.00473).
      colon_list = expression colon_metarelop expression => list_apply
        | colon_list colon_metarelop expression => list_apply;
      // A paren item may carry a sign (`(1:-1:0)`, Perl `SignedTerm`, MathGrammar:246), not a sum.
      colon_item = term | addop term => prefix_apply;
      colon_pair = colon_item colon_metarelop colon_item => list_apply;
      colon_terms = colon_pair colon_metarelop colon_item => list_apply
        | colon_terms colon_metarelop colon_item => list_apply;

      // ASF migration item 5 (Option A semantics, 2026-05-19, user-
      // articulated): `modified_term` is a `tight_term` carrying ONE
      // relop modifier. Single-relop only — multi-relop chains
      // (`x < y < 0`) stay at the formula level via the existing
      // multirelation flattening.
      //
      // Action: reuse `infix_relation`. For a single relop call it
      // produces `Apply(relop, [tight_term, expression])` — the same
      // shape as `formula relop expression` reduces to for the base
      // case. So `x = 0` parses identically via either route, ASF /
      // tree-iter dedup eliminates the duplicate.
      //
      // Why this category exists: it enables comma-list contexts
      // where each item carries its own relop — most notably
      // function arguments like `P(x = 0, y < 0)`. Today such
      // input is `ltx_math_unparsed` because `formula_list_apply`
      // rejects relational items (a comma-list-of-relations is
      // semantically a different beast from a relop chain RHS).
      // Modified_term threaded through `list_apply` (not
      // `formula_list_apply`) gives the desired list-of-relations
      // shape. Surpass-Perl improvement — the corresponding Perl
      // grammar via `addEasyArgs` handles this through a different
      // path with the same effect; here we keep Perl's semantic
      // outcome.
      //
      // See `docs/math/MATH_PARSER_ASF_TIEBREAKING.md` (commit
      // 5cde377610) for the proposal context.
      modified_term = tight_term relop expression => infix_relation;
      // Phase 1 was the all-modified-terms variants only; the mixed-content
      // variants were deferred "until a witness shows them needed", to keep
      // ambiguity growth tight (the `parse_tree_count_limits` regression test
      // is the canary).
      //
      // Phase 2 (2026-07-25) — the witness arrived. arXiv 2605.17646 carries
      // `m_S(t \mid T_i \geq t_{\text{crit}}, \mathbf{Z})`: a conditional whose
      // RHS is a comma list of ONE relation plus ONE plain term. With only the
      // all-modified rule, `f(a\geq 0, b\leq 1)` parsed while
      // `f(a\geq 0, b)` did not — the whole equation fell to
      // ltx_math_unparsed. Both orders are admitted; the longer chains are
      // already covered by the `formula_list punct …` extensions.
      formula_list += modified_term punct modified_term => modified_list_apply
        | modified_term punct expression => modified_list_apply
        | expression punct modified_term => modified_list_apply
        | formula_list punct modified_term => modified_list_apply;
      // Comma-separated term lists: term, term, term, ...
      // Used for angle-bracket inner products <x,y>, <a,b,c>, etc.
      // Also includes limit_from_term for patterns like (1+, 0+, 1-, 0-).
      term_list = term punct term => list_apply
        | term_list punct term => list_apply
        | limit_from_term punct term => list_apply
        | limit_from_term punct limit_from_term => list_apply
        | term_list punct limit_from_term => list_apply
        | term punct limit_from_term => list_apply;

      // A bare operator standing in for an OPERAND — the argument-slot
      // notation `f(\cdot)`, `\langle\cdot,\cdot\rangle`, and operators named
      // rather than applied, `(+)` / `(=)` / `(\times)`. Perl's grammar admits
      // operators as factors generally; we admit them only where they are
      // FENCED (see `fenced_factor`), the same containment the bigop/operator
      // lines there already use — so a stray `a + \times b` still fails.
      // Without this the whole formula died: Perl parsed 7 of 8 such shapes,
      // we parsed 0 of 8. Witness: arXiv 2605.17646 (`f(\cdot)`, `S(\cdot)`).
      placeholder = mulop | addop | binop | relop;
      // The argument slot a pair of bars holds (`bare_abs`, divergence #355): a product's operator.
      bar_placeholder = mulop | binop;
      // A single bar that divides a conditional, closes a bra or opens a ket. Perl re-roles a
      // `\left|`/`\right|` a VERTBAR (DELIMITER_MAP, TeX_Math.pool.ltxml:754, :813-816) and its bar
      // terminal ignores the side (MathGrammar:797), so a stretchy bar divides as `|` does:
      // `P\left(\left.A\right|B,C\right)` (a `\right|` after `\left.`), `\left\{x\left|x>0\right.\right\}`
      // (a `\left|` before `\right.`); a `\right|` closes a bra (`\left\langle\Psi\right|`), a `\left|`
      // opens a ket (`\left|\Psi\right\rangle`). Witnesses 2605.11264, 2605.20326.
      divider_bar = singlevertbar | right_stretchy_single_bar | left_stretchy_single_bar;
      bra_bar = singlevertbar | right_stretchy_single_bar;
      ket_bar = singlevertbar | left_stretchy_single_bar;
      // Comma list carrying AT LEAST ONE placeholder. The "≥1" shape is
      // deliberate: an all-`expression` list is already `formula_list`, so
      // admitting it here too would duplicate every ordinary `(a,b)` parse and
      // double the forest for no new coverage.
      placeholder_list = placeholder punct placeholder => list_apply
        | placeholder punct expression => list_apply
        | expression punct placeholder => list_apply
        | placeholder_list punct placeholder => list_apply
        | placeholder_list punct expression => list_apply;

      // Perl MathGrammar L709-711: Two-part relops (>=, <=, <<, >>)
      two_part_relop = langle_rel langle_rel => two_part_relop_combine
        | rangle_rel rangle_rel => two_part_relop_combine
        | langle_rel relop_equals => two_part_relop_combine
        | rangle_rel relop_equals => two_part_relop_combine;

      bare_operator_operand = addop | mulop | binop;
      formula = expression
        | formula relop expression => infix_relation
        | formula two_part_relop expression => infix_relation
        // NOTE: deliberately NO `formula relop formula_list` rule. A bare
        // (unparenthesized) comma-list is NOT a single expression, so it can
        // never be a relation operand: `0 < x,y` can only mean
        // `list(0<x, y)` (the comma splits at the statements level via
        // `statements punct statement`), never `0 < list(x,y)`. Admitting the
        // list as a relop RHS produced a spurious parse for every
        // comma-after-relation, multiplying enumeration combinatorially down a
        // formula_list (a top ambiguity-explosion source — `1510.03361`'s
        // 5000-tree-cap equations). Parenthesized lists `(x,y)` remain single
        // expressions via `lparen formula_list rparen => fenced`.
        | formula relop => postfix_relop
        // A bare operator as the right side, `a=\pm`, `x=\bot`, `x\to-` (Perl `Expression : AnyOp
        // ...anyOpIsolator`, MathGrammar:204-206; 57bj, RED-drain survey P10, 2605.03453, 2605.01293).
        // One derivation each; unlike Perl's isolator (end, PUNCT or CLOSE only) a relation may follow,
        // `x=\bot\le y` x = bottom <= y (Perl unparsed; 57bj review).
        | formula relop bare_operator_operand => infix_relation
        | formula arrow bare_operator_operand => infix_relation
        // Perl moreRelations: `relop moreRelations` — consecutive relops chain without intervening terms
        // e.g. `A ∈ ∞ ∋` → the ∈ absorbs ∞, then ∋ appends to the chain (no absent)
        | formula relop relop => consecutive_relop_chain
        | formula arrow expression => infix_relation
        // A trailing ARROW with no RHS (e.g. `a \to`, `x \mapsto`) — postfix
        // relation with an absent right operand, like `formula relop =>
        // postfix_relop`. Perl yields `a to absent`; without this it fell to
        // ltx_math_unparsed.
        | formula arrow => postfix_relop
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
        // A METARELOP (e.g. `:`) whose RHS is a `\quad`-separated list, mirroring
        // the RELOP rule `formula relop formula_list` (line ~375): `a : b \quad c`
        // → `a colon list@(b, c)`. Without it, only the bare RELOP form parsed
        // (`a = b \quad c` worked, `a : b \quad c` fell to ltx_math_unparsed) —
        // common in `\forall x : P \quad Q`-style notation.
        | statement metarelop formula_list => infix_relation
        // A trailing METARELOP with no RHS (e.g. `x \mapsto`) — postfix relation
        // with an absent right operand, mirroring `formula relop => postfix_relop`
        // for plain relops. Perl yields `x maps-to absent`; without this the
        // formula fell to ltx_math_unparsed.
        | statement metarelop => postfix_relop
        | metarelop formula => prefix_metarelop_apply
        | operator
        | function | trigfunction
        // Bare operators can form comma-separated lists: +,-,×
        | addop | mulop | binop | relop | arrow
  ;

      end_punct = punct | period;
      statements = statement
        | statement end_punct => postfix_embellished
        | statements end_punct => postfix_embellished
        | statements punct statement => list_apply
        // `\quad`/`\qquad`-spaced PUNCT (WIDE_PUNCT) admits as a
        // list-separator for non-relational items — e.g.
        // `\ring{x},\qquad\accentset{\star}{d},\qquad...` is a list
        // of decorated atoms, not separate formulae. PUNCT and
        // WIDE_PUNCT are distinct lexemes (a given input comma is
        // exactly one), so this rule doesn't compete with the bare
        // PUNCT rule above on the same input position — the pragmas
        // (list_apply: both items relational / formulae_apply: no
        // relational items) decide which interpretation survives.
        | statements wide_punct statement => list_apply
        // Perl MathGrammar L129: endPunct includes PERIOD. Period creates formulae, not list.
        | statements period statement => formulae_apply
        // Perl: MorphVertbar — VERTBAR as conditional modifier: x | y,z,t
        | statement vertbar statements => vertbar_modifier
        // Comma-LIST left of the bar: `a,b | c` → conditional(list@(a,b), c).
        // Explicit `statements punct statement vertbar …` shape (NOT a generalized
        // `statements vertbar statements`, which over-applies to abs-value `|a|`
        // and explodes the forest). Root fix for the Class-B `\Pr(s_A,s_B|\Omega)`
        // dangling-XMRef (its argument previously failed to parse). See
        // EXPECTED_ID_XMREF_DESIGN 2026-06-26p/q.
        | statements punct statement vertbar statements => vertbar_modifier_listlhs;

      // Perl MathGrammar: Formulae = Formula (endPunct Formula)* → NewFormulae()
      // Separate nonterminal from expression-level formula_list to avoid ambiguity:
      // expression-level formula_list uses formula_list_apply (rejects relops),
      // while formulae uses formulae_apply (accepts full statements).
      formulae = statement punct statement => formulae_apply
        | formulae punct statement => formulae_apply
        // Period also separates formulae
        | statement period statement => formulae_apply
        | formulae period statement => formulae_apply
        // `\quad`/`\qquad`-spaced PUNCT (lexer-tagged WIDE_PUNCT) also
        // separates formulae. The token-level distinction from generic
        // PUNCT means the grammar enumerates a DIFFERENT input alternative
        // (a comma in the input is either PUNCT or WIDE_PUNCT, never
        // both), avoiding the cross-contamination of treating one comma
        // both ways simultaneously. Combined with `statements
        // wide_punct statement → list_apply` below, the pragma decides
        // which interpretation survives based on item-relationality.
        | statement wide_punct statement => formulae_apply
        | formulae wide_punct statement => formulae_apply;

      // Extensions, now that we have more category variables defined
      // A group — Perl's `OPEN … CLOSE` (`addEasyArgs`, MathGrammar:571-576) and the fences
      // built like it — what a function applies to; the bar pairs are `bare_abs`. Its content is
      // Perl's `Argument`, an expression a relation may extend (:581-587), whatever the delimiters:
      // `\{x\in A\}` set@(x ∈ A), `\Pr[X=1]` Pr@(X = 1) (57bb; were unparsed, 2605.01547; golden
      // tests/parse/fenced_lists.tex, "A function takes the arguments between any delimiters").
      group_factor = lbrace formula rbrace    => fenced
             | lbracket formula rbracket          => fenced
             | lparen formula rparen              => fenced
             // METARELOP inside parens: f(a:b), f(a↔b) — colon/arrow as relation in fenced
             | lparen formula metarelop expression rparen => fence
             // Parenthesized comma-separated lists: (a,b,c), (a+b, c+d), (1+, 0+, 1-, 0-)
             // Perf (Fix 3): `lparen term_list rparen` was duplicate with
             // `lparen formula_list rparen` for non-relational content (both produce
             // identical `list@(...)` trees). Dropped; formula_list covers
             // (a,b,c), (a+b,c+d), and (0+,1-) via `expression addop => postfix_apply`
             // which produces limit-from XM matching limit_from_apply semantics.
             | lparen formula_list rparen        => fenced
             // Bracketed and braced comma-separated lists: [a,b,c], {a,b,c}
             | lbracket formula_list rbracket    => fenced
             | lbrace formula_list rbrace        => fenced
             // Colon lists: `[a:b]`, `[x_0:x_1:\dots:x_n]`, `(x:y:z)` (57bm, above)
             | lbracket colon_list rbracket      => fenced
             | lparen colon_terms rparen         => fenced
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
             // (a `term_list` is a `formula_list`: one rule, 57bi)
             | langle_open formula rangle_close => fenced
             | langle_open formula_list rangle_close => fenced
             | langle_open formula metarelop expression rangle_close => fence
             // Perf/design: interval rules moved out of fenced_factor into
             // term (see `tight_term += interval_term` below): function application
             // `f(x,y)` takes a fenced_factor, so the interval derivation of `(x,y)` is
             // pruned there and the list one (from `lparen formula_list rparen`) wins.
             // A balanced `(a,b)` elsewhere has both derivations; its name is not theirs
             // but the slot's, given after the parse (`rename_fenced_lists`, #371).
             // QM bra-ket uses langle_open/rangle_close (specific ⟨⟩ tokens),
             // avoiding ambiguity with relational < > (langle_rel/rangle_rel).
             // Conditional probability uses lparen/rparen (specific () tokens),
             // avoiding ambiguity with ket (which requires rangle_close).
             //
             // The bar pairs — `|x|`, `\|x\|`, `\left|x\right|` and the norms — are `bare_abs`,
             // below.
             // Dirac ket: |label⟩ — VERTBAR as opening, CLOSE:rangle as closing
             // Restricted to rangle_close (⟩) to avoid ambiguity with conditional
             // probability (x|y) where ) is a generic CLOSE but not rangle.
             // Perl MathGrammar uses RANGLE specifically, not generic CLOSE.
             // Ket labels: expressions, arrows, operators, relops, etc.
             | ket_bar expression rangle_close => qm_ket
             | ket_bar arrow rangle_close => qm_ket
             | ket_bar metarelop rangle_close => qm_ket
             | ket_bar operator rangle_close => qm_ket
             | ket_bar any_bigop rangle_close => qm_ket
             | ket_bar mulop rangle_close => qm_ket
             | ket_bar addop rangle_close => qm_ket
             | ket_bar relop rangle_close => qm_ket
             | ket_bar modifierop rangle_close => qm_ket
             // Dirac bra: ⟨label| — OPEN:langle as opening, VERTBAR as closing
             // Restricted to langle_open to avoid ambiguity with parens.
             | langle_open expression bra_bar => qm_bra
             | langle_open arrow bra_bar => qm_bra
             | langle_open metarelop bra_bar => qm_bra
             | langle_open operator bra_bar => qm_bra
             // Braket: ⟨a|b⟩ → inner-product@(a, b)
             | langle_open expression divider_bar expression rangle_close => qm_braket
             // Bracket: ⟨a|f|b⟩ → quantum-operator-product@(a, f, b); with sided bars the middle may
             // hold bar pairs, where Perl's `ketExpression` forbids bars (divergence #356)
             | langle_open expression bra_bar expression ket_bar expression rangle_close => qm_bracket
             // Same Dirac shapes when the divider is a stretchy `\middle|`
             // (`MIDDLE:|`) — the ubiquitous physics form
             // `\left\langle a \middle| b \right\rangle`. Perl matches `|` and
             // `\middle|` with one terminal (MathGrammar L10084); here we add the
             // MIDDLE:| variants explicitly. (Mixed |/\middle| dividers are not
             // attempted — authors are consistent within a braket.)
             | langle_open expression middle_bar expression rangle_close => qm_braket
             | langle_open expression middle_bar expression middle_bar expression rangle_close => qm_bracket
             // Same Dirac shapes with plain ASCII `<` `>` (langle_rel/rangle_rel
             // — RELOP-classed angles) — physicists commonly write `<a|f|b>` even
             // outside `\langle/\rangle` macros. Semantics match the
             // `\langle…\rangle` forms above so downstream MathML sees a single
             // inner-product / quantum-operator-product Apply.
             | langle_rel expression singlevertbar expression rangle_rel => qm_braket
             | langle_rel expression singlevertbar expression singlevertbar expression rangle_rel => qm_bracket
             // (Comma-separated items in braces, {a,b} and {a,b,c}, are `lbrace formula_list rbrace`,
             // Perl's Fence through `fenced`; 57bi removed the `lbrace term punct term…` twins.)
             // Perl: {a|b} conditional-set with VERTBAR or MIDDLE separator
             | lbrace formula divider_bar formula rbrace => fence
             | lbrace formula middle_bar formula rbrace => fence
             | lbrace formula metarelop formula rbrace => fence
             // … whose element or condition is a list (Perl `FormulaNOBar suchThatOp Formulae`,
             // MathGrammar:487-491): `\{x,y|z\}`, `\{x : a<1, b<2\}`, `\{[a,b]:a\in A,b\in B\}`
             // (57bh; RED-drain survey P6, 2605.24529, 2605.08004). Only a colon makes it a set-builder
             // (`fence`): another metarelation is a set of one relation, `\{\Gamma\vdash A,B\}`
             // set@(Gamma proves list@(A, B)) (57bk; 57bh review).
             | lbrace formula_list divider_bar formula rbrace => fence
             | lbrace formula divider_bar formula_list rbrace => fence
             | lbrace formula_list divider_bar formula_list rbrace => fence
             | lbrace formula metarelop formula_list rbrace => fence
             | lbrace formula_list metarelop formula rbrace => fence
             | lbrace formula_list metarelop formula_list rbrace => fence
             // Conditional probability: p(a|b) — safe now that ket uses rangle_close
             // (not generic close), so |y) no longer matches ket pattern.
             | lparen formula divider_bar formula rparen => fence
             | lparen formula_list divider_bar formula rparen => fence
             | lparen formula divider_bar formula_list rparen => fence
             // A list on both sides: the joint given several, `p(x,y|z,w)` (57bh; divergence #364)
             | lparen formula_list divider_bar formula_list rparen => fence
             // Bracketed conditional `[a|b]` / `E[X|Y]` (conditional expectation).
             // Perl: delimited-[]@(conditional@(a,b)). Unlike (a|b)/{a|b}, the bare
             // a|b conditional isn't an `expression`, so [a|b] had no fence rule
             // and fell to ltx_math_unparsed (though [(a|b)] worked). `singlevertbar`
             // also covers `\mid` (canonicalized VERTBAR:mid → VERTBAR:|).
             | lbracket formula divider_bar formula rbracket => bracket_conditional
             // … with a list on either side, `E[Y\mid A=1,X]`, `\Pr[X=1,Y=1|Z=0]` (57bh; RED-drain survey
             // P4, 2605.05890)
             | lbracket formula divider_bar formula_list rbracket => bracket_conditional
             | lbracket formula_list divider_bar formula rbracket => bracket_conditional
             | lbracket formula_list divider_bar formula_list rbracket => bracket_conditional
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
             // (A fenced bare operator — (\nabla), \lfloor\nabla\rfloor — is a formula since
             // `term += bare_op_term`; 57bi removed the `lparen operator rparen` twins.)
             // Fenced bare-operator placeholders: (\cdot), [\cdot], \langle\cdot,\cdot\rangle,
             // (+), (=), (\times), f(\cdot,x). See `placeholder` above for why
             // these are admitted only when fenced.
             | lparen placeholder rparen => fenced
             | lbracket placeholder rbracket => fenced
             | lbrace placeholder rbrace => fenced
             | langle_open placeholder rangle_close => fenced
             | open placeholder close => fenced
             | lparen placeholder_list rparen => fenced
             | lbracket placeholder_list rbracket => fenced
             | lbrace placeholder_list rbrace => fenced
             | langle_open placeholder_list rangle_close => fenced
             | open placeholder_list close => fenced
             // Empty fenced expressions: () [] {} ⌊⌋ ⟨⟩ etc.
             | lparen rparen => empty_fenced
             | lbracket rbracket => empty_fenced
             | lbrace rbrace => empty_fenced
             | langle_open rangle_close => empty_fenced
             | open close => balanced_empty_fenced;
      // Perl `aBarearg` (MathGrammar:323-331): the bar pairs, which an operator's or OPFUNCTION's
      // bare argument takes as an item, never as its group — `VERTBAR absExpression VERTBAR`
      // (:329-330; `\|` and `\left|…\right|`, `\left\|…\right\|` are VERTBARs too, `\lvert` an
      // OPEN), and the norms of four bars, merged or kerned (divergence #353: `\nabla u||v||` is
      // ∇@(u·‖v‖), as `\nabla u\|v\|`). A fence of its own, not a filtered group: every rejected
      // reading was a tree of its own on the tree-iterator route, multiplying across a sum of
      // operator terms (`\log\|x\|+\log\|y\|+…`).
      //
      // Perl MathGrammar L294: || exp || → norm (must be before |exp| → abs-val)
      // CatSymbols merges two | into ‖; singlevertbar = VERTBAR:|
      bare_abs = singlevertbar singlevertbar expression singlevertbar singlevertbar => norm_fenced
        // `\|x\|` / `\Vert x\Vert`: the doubled bar arrives as a single
        // `VERTBAR:||` token (not two `|`), so it forms `|| expr ||` —
        // the standard norm notation. Without this it was unparsed; Perl
        // parses it to norm@(x). (Subscripted `\|x\|_p` then parses via
        // fenced_factor + POSTSUBSCRIPT.)
        | doublevertbar expression doublevertbar => double_norm_fenced
        | singlevertbar expression singlevertbar => fenced
        // Kerned-stack norm / operator-norm from `\left|\kern\left|...`
        // (community idioms: \vertii, \vertiii, \Vert, \tnorm, …).
        // The lexer in util.rs distinguishes bars emitted by `\lx@delim@left`
        // (LEFT_STRETCHY_VERTBAR) from those emitted by `\lx@delim@right`
        // (RIGHT_STRETCHY_VERTBAR) via the `lx@side` property set in
        // those constructors (tex_math.rs:\lx@delim@left and :\lx@delim@right). This
        // pre-distinction collapses what would be a combinatorial
        // pairing of identical stretchy bars into a single ungrammatical
        // shape — the parser never enumerates `right-...-left` invalid
        // pairings. The actions still verify the negative-rpadding
        // (\kern) signal as a side condition so that two intentionally
        // separate `\left|...\right|\left|...\right|` fences aren't
        // accidentally merged. Task #263. Witness arXiv:2211.13044 §S4.Ex17.
        | left_stretchy_single_bar left_stretchy_single_bar left_stretchy_single_bar expression
            right_stretchy_single_bar right_stretchy_single_bar right_stretchy_single_bar
            => stretchy_triple_norm_fenced
        | left_stretchy_single_bar left_stretchy_single_bar expression
            right_stretchy_single_bar right_stretchy_single_bar
            => stretchy_norm_fenced
        // … and around a placeholder (#355): `\vertii{\cdot}` is norm@(·), `\vertiii{\cdot}`
        // operator-norm@(·), not nested bars around it (57am review round 8).
        | left_stretchy_single_bar left_stretchy_single_bar left_stretchy_single_bar bar_placeholder
            right_stretchy_single_bar right_stretchy_single_bar right_stretchy_single_bar
            => stretchy_triple_norm_fenced
        | left_stretchy_single_bar left_stretchy_single_bar bar_placeholder
            right_stretchy_single_bar right_stretchy_single_bar
            => stretchy_norm_fenced
        // Balanced modulus: `\left| expr \right|` (stretchy bars), and `\left\| expr \right\|`.
        // The lexer (util.rs) tags `\left/\right`-paired bars as
        // LEFT_STRETCHY_VERTBAR / RIGHT_STRETCHY_VERTBAR so we don't
        // enumerate every alternative pairing of bare `|`s. With this
        // rule, `\left|f\right|^k` unambiguously pairs the two stretchy
        // bars regardless of surrounding parens / scripts.
        | left_stretchy_vertbar expression right_stretchy_vertbar => fenced
        // A placeholder between `\left`/`\right` bars, the norm or absolute value of an argument
        // slot: `\left\|\cdot\right\|_\infty` (divergence #355; Perl leaves it unparsed). Sided
        // bars only: between plain ones, `|x|\cdot|y|` would offer a `|\cdot|` of its own. A MulOp or
        // BinOp slot (`\cdot`, `\bullet`): no `|+|`, `|=|`.
        | left_stretchy_vertbar bar_placeholder right_stretchy_vertbar => fenced;
      fenced_factor = group_factor | bare_abs;
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
      // Perf (grammar pruning): trig_arg uses `factor_base` (bare factors only), NOT `factor`
      // (which includes fenced_factor): a trig function's group is `trig_factor_arg` (below, 57bg),
      // one derivation per application.
      // Function application paths (function fenced_factor) remain so \sin f(x)
      // and \sin F(x) still parse correctly as sin(f(x)) / sin(F(x)).
      trig_arg = factor_base
        | unknown group_factor => speculative_prefix_apply
        | diffunk group_factor => speculative_prefix_apply
        | function fenced_factor => prefix_apply
        // Perl: trigBarearg includes OPFUNCTION+args (chained function application)
        // Allows: \sin\det A → sin(det(A)). FUNCTION doesn't absorb bare args.
        | opfunction factor => prefix_apply
        // trig_arg chains only through factor_base on the RHS. Previous approach
        // chained through full `factor` causing \sin(x) + (y) to ambiguously
        // parse as sin((x)+(y)).
        | trig_arg mulop factor_base => infix_apply_nary
        | trig_arg binop factor_base => infix_apply_nary
        // explicit space ends the argument (#367): `\sin\theta\,d\theta` is sin@(θ)·dθ
        | trig_arg factor_base => trig_argument_juxtaposition;

      // applied_func: FUNCTION only absorbs fenced args (parens), not bare args.
      // OPFUNCTION and TRIGFUNCTION absorb bare args (Perl distinction).
      // Perl: `fga` = f*g*a (FUNCTION), `Fga` = F@(g*a) (OPFUNCTION)
      applied_func = function fenced_factor => prefix_apply
        | trigfunction trig_arg => prefix_apply
        // Perl `addOpFunArgs` (MathGrammar:553-558): an OPFUNCTION applies to a group first
        // (`addEasyArgs`, :571-576), and the application ends with it — `\log(a)\nabla b` is
        // log@(a)·∇@(b); its bare argument (`opfunction op_bare_arg`, after `op_bare_arg`)
        // takes no group.
        | opfunction group_factor => prefix_apply;
      // Delimited function application — f(x), f[x], \max\{a,b\} — goes through `prefix_apply`
      // over `fenced_factor`/`group_factor`, which builds Perl's `ApplyDelimited` Dual
      // (MathParser.pm:1291-1299; `addEasyArgs`, MathGrammar:571-576) and spreads a list. The
      // `function|opfunction lparen formula rparen => apply_delimited` twins built the same tree a
      // second time, doubling the trees per application (57bi: `\log(x)\log(y)` 6 trees → 2).
      // Being applied_funcs, applications chain: f(a) g(b) → f@(a) * g@(b) (`tight_term
      // applied_func => apply_invisible_times`).
      // (A trig function's group is `trig_factor_arg`, below.)
      // Standalone applied functions are also tight_terms
      tight_term += applied_func;
      // Function application results can chain with invisible times (Perl moreFactors)
      tight_term += tight_term applied_func => apply_invisible_times;

      // Intervals are math objects (`[a,b]`, `(a,b]`, etc.), not grouping
      // constructs. Moved out of fenced_factor so function application
      // (`f(x,y)`) naturally prunes the interval interpretation in favor
      // of the list interpretation via `lparen formula_list rparen`. A balanced paren
      // pair is named after the parse by the slot it fills: a vector unless a set
      // relation asks for a set (`rename_fenced_lists`, divergence #371).
      // Placed at tight_term level so intervals participate in invisible
      // multiplication (`2(a,b)` = `2 * (a,b)`) but not in `f(...)` apply.
      // A half-open (or French open) interval's endpoints are expressions, as Perl's
      // `factorOpenExpr` reads `Expression (PUNCT Expression)(s)` before any close (MathGrammar:472-475):
      // `x\in(-1,0]`, `t\in[0,T+1)`, `(-\infty,0]` (57bj; RED-drain survey P7, 2605.31172, 2605.01053,
      // 2605.03240). Open: a scripted half-open interval (`(0,1]^n`) and one after a factor (`2(a,b]`).
      // Its unbalanced delimiters have no other derivation; a balanced pair is also a
      // `formula_list` group, so it keeps `term` endpoints (one derivation each).
      interval_term = lparen term punct term rparen      => interval
        | lparen expression punct expression rbracket    => interval
        | lbracket term punct term rbracket  => interval
        | lbracket expression punct expression rparen  => interval
        | rbracket expression punct expression lbracket => interval;
      tight_term += interval_term;

      // UNKNOWN followed by fenced args => function application (Perl: doubtArgs/maybeArgs)
      // f(x) => f@(x), g(a+b) => g@(a+b). Only active when MATHPARSER_SPECULATE is set.
      // Without speculation, this parse is pruned and Marpa uses invisible-times instead.
      // NOTE: ID tokens are multiplicative atoms — NEVER prefix-apply. Only UNKNOWN
      // tokens get speculative function application. ID always uses invisible-times.
      tight_term += unknown group_factor => speculative_prefix_apply
        | diffunk group_factor => speculative_prefix_apply;
      // Perf: `tight_term += function fenced_factor => prefix_apply` removed: it duplicated the
      // applied_func path (function fenced_factor => prefix_apply).
      // OPFUNCTION as the RIGHT operand of an implicit-times chain
      // (`c \not`, `a b \not`, the trailing-OPFUNCTION cases in
      // tests/math/not.tex and the recognizer_trailing_opfunction
      // unit test). Replaces the `tight_term factor` path that used
      // to admit OPFUNCTION via `factor → opfunction` — the LEFT
      // restriction was the whole point of dropping OPFUNCTION from
      // `factor` (see comment at the `factor` definition above).
      // Such a product ending in a bare OPFUNCTION is followed only by what the function does
      // not take (`apply_invisible_times`: a big operator's or an operator's application,
      // `\tfrac12\log\det(\Sigma)`).
      tight_term += tight_term opfunction => apply_invisible_times;
      // (A trig function's scripted or fenced argument is `trig_factor_arg`, below: an `applied_func`,
      // so it can follow another factor.)
      // A compound operator (`D\nabla`, `D\sin`), applied or not, is `op_application` /
      // `bare_op_term`, below: `\nabla\log x` is (∇@log)@(x).
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
      // Perl's operator pseudo-terminals take decorating scripts (MathGrammar:681-712):
      // `relop : RELOP addOpDecoration | ARROW addOpDecoration`, `AddOp : ADDOP|BINOP
      // addOpDecoration`, `MulOp : MULOP|BINOP addOpDecoration`, where `addOpDecoration` is any
      // run of POSTSUPERSCRIPT/POSTSUBSCRIPT, each applied by `DecorateOperator` (MathParser.pm:
      // 1649-1654: the scripted operator keeps the operator's role). So every rule that reads an
      // operator reads a decorated one too: `a\leq_k b`, `a\to_n b`, `a+_k b`, `A\cup_i B`,
      // `x\times_i^2 y`. METARELOP stays bare, as Perl's AnyOp. Golden
      // tests/parse/decorated_relations.tex#scripted_relop_is_decorated (arXiv 2605.03594,
      // 2605.28533, 2605.20841).
      relop += relop postsubarg => decorate_operator
        | relop postsuperarg => decorate_operator;
      arrow += arrow postsubarg => decorate_operator
        | arrow postsuperarg => decorate_operator;
      addop += addop postsubarg => decorate_operator
        | addop postsuperarg => decorate_operator;
      mulop += mulop postsubarg => decorate_operator
        | mulop postsuperarg => decorate_operator;
      binop += binop postsubarg => decorate_operator
        | binop postsuperarg => decorate_operator;

      // Scripted FUNCTION with fenced args: f'(a), f^2(a), f_n(x)
      scripted_function = function postsuperarg => postfix_script
        | function postsubarg => postfix_script
        | function postsubarg postsuperarg => postfix_script
        | function postsuperarg postsubarg => postfix_script;
      // All scripted function application rules go through applied_func only, one rule each (two
      // such calls in one formula multiplied ambiguity). A scripted function takes a group as a bare
      // one does (Perl `preScripted['FUNCTION'] addArgs`, MathGrammar:323-325, :543-548: any OPEN,
      // not bars): `f_n(x,y)`, `f_n\{a,b\}` parse, and `prefix_apply` lifts the application (57bf).
      applied_func += scripted_function group_factor => prefix_apply;

      // Scripted OPFUNCTION with bare/fenced args: \log_e a, \det_S x
      scripted_opfunction = opfunction postsuperarg => postfix_script
        | opfunction postsubarg => postfix_script
        | opfunction postsubarg postsuperarg => postfix_script
        | opfunction postsuperarg postsubarg => postfix_script;
      // A scripted OPFUNCTION applies as a bare one (`addOpFunArgs`): to a group, or to a bare
      // argument (`scripted_opfunction op_bare_arg`, after `op_bare_arg`).
      applied_func += scripted_opfunction group_factor => prefix_apply;
      // Divergence #351 (OXIDIZED_DESIGN_DIVERGENCES): an OPFUNCTION applied to a group takes the
      // scripts after it — `\log(n)^2` is (log@(n))², `\operatorname{Var}(X)_k` (Var@(X))_k,
      // `\max(a,b)^2` (max@(a,b))² — where Perl's `addEasyArgs` application ends the Factor and the
      // script has no base (unparsed; the old `opfunction tight_term` read log@(n²); 2605.24357,
      // 2605.01408). Not a TRIGFUNCTION: `\sin(x)^2` stays sin@(x²) (`trig_arg`).
      // Only an unscripted head: a scripted one (`\min_w(y-w)^2`, `\max_i(\lambda_s)_i`) reads
      // as a limit, its operand the scripted group — min_w@((y−w)²) — as `\sup_i(a_i)^2` does
      // (`scripted_group_apply`, after the scripted factors; 2605.04340, 2605.23087, 2605.19263).
      // The `apply_delimited` alternative is the only derivation of a paren group holding a fenced
      // modifier (`\log(\to x)^2`, `\log(>0)^2`: `group_apply` refuses `is_fenced_modifier_dual`),
      // and a twin of `group_apply` for every other group (four `\log(x)^2` still enumerate 31
      // trees; 57bi review). Its eager `create_xmrefs` spends an xml:id on a tree the ASF may prune
      // (`physics_test` under `LATEXML_MARPA_ASF_ONLY=1`). Follow-up: let `group_apply` take a
      // paren-fenced modifier and drop it.
      opfunction_group_application = opfunction group_factor => group_apply
        | opfunction lparen formula rparen => apply_delimited;
      scripted_opfunction_application = opfunction_group_application postsuperarg => postfix_script
        | opfunction_group_application postsubarg => postfix_script
        | opfunction_group_application postsubarg postsuperarg => postfix_script
        | opfunction_group_application postsuperarg postsubarg => postfix_script;
      tight_term += scripted_opfunction_application;
      tight_term += tight_term scripted_opfunction_application => apply_invisible_times;
      // An interval whose close does not balance its open (`interval_term`: `(0,1]`, `[a,b)`) is
      // no OPFUNCTION's argument — `addEasyArgs` needs a `balancedClose` (MathGrammar:571-576) —
      // but the factor after the bare function (`addOpFunArgs`' `{ $arg[0] }`, :557): `\log(0,1]`
      // is log·(0,1], `\max[a,b)` max·[a,b), as Perl (golden
      // tests/parse/opfunction_arguments.tex#function_before_an_unbalanced_interval_multiplies).
      function_times_interval = opfunction interval_term => apply_invisible_times
        | scripted_opfunction interval_term => apply_invisible_times;
      tight_term += function_times_interval;
      tight_term += tight_term function_times_interval => apply_invisible_times;

      // Scripted OPERATOR applied to an operand: `\nabla^2 \phi` (Laplacian),
      // `\nabla_x f`, `\nabla^2(f)`, through `op_application` below, as the unscripted
      // operator is. Without it, a superscripted/subscripted OPERATOR applied
      // to an argument was unparsed (→ ltx_math_unparsed); Perl parses
      // `\nabla^2 \phi` to `(nabla ^ 2)@(phi)`. (`\partial^2 f` already worked —
      // `\partial` is a DIFFOP/any_bigop with its own scripted path.)
      scripted_operator = operator postsuperarg => postfix_script
        | operator postsubarg => postfix_script
        | operator postsubarg postsuperarg => postfix_script
        | operator postsuperarg postsubarg => postfix_script;
      // Not a `factor`: a scripted operator with no argument is a `bare_op_term` (below).
      // Perl `OPERATOR addScripts nestOperators` (MathGrammar:312-313, :663-671): a scripted
      // operator nests over a following function as the unscripted `compound_operator` does —
      // `\nabla_x\log p(y)` is ((∇_x)@(log))@(p) · y.
      compound_operator += scripted_operator trigfunction => prefix_apply
        | scripted_operator function => prefix_apply
        | scripted_operator opfunction => prefix_apply
        | scripted_operator compound_operator => prefix_apply;

      // Scripted TRIGFUNCTION: \sin^2 x, \cos_n x
      scripted_trigfunction = trigfunction postsuperarg => postfix_script
        | trigfunction postsubarg => postfix_script
        | trigfunction postsubarg postsuperarg => postfix_script
        | trigfunction postsuperarg postsubarg => postfix_script;
      // (A scripted trig function takes the bare one's arguments, `trig_arg` and `trig_factor_arg`,
      // below: Perl `preScripted['TRIGFUNCTION'] addTrigFunArgs`, MathGrammar:284, :430-433, 57bo.)


      // standalone top-level variants of floating scripts:
      floatsubscript = start_floatsubscript expression end_floatsubscript => standalone_script;
      floatsuperscript = start_floatsuperscript expression end_floatsuperscript => standalone_script;
      // A script whose argument is one float script alone, `F^{{}^{\prime}}`, `H_{{}_{\mathrm I}}`: Perl
      // parses a one-node script argument as is (MathParser.pm:680-681); a float followed by more
      // (`x^{{}^{\prime}\prime}`) stays unparsed, as in Perl (57bj; RED-drain survey P3, 2605.21192,
      // 2605.04817, 2605.18286).
      lone_float_script = floatsuperscript | floatsubscript
        | start_floatsuperscript script_op end_floatsuperscript => standalone_script
        | start_floatsubscript script_op end_floatsubscript => standalone_script
        // … or a bare `+`, `-`, `*` (`x^{{}^{*}}`, Perl `A ^ ^ast * B`)
        | start_floatsuperscript bare_operator_operand end_floatsuperscript => standalone_script
        | start_floatsubscript bare_operator_operand end_floatsubscript => standalone_script;
      postsuperarg += start_postsuperscript lone_float_script end_postsuperscript => faux_wrap;
      postsubarg += start_postsubscript lone_float_script end_postsubscript => faux_wrap;
      bigopsuparg += start_bigopsup lone_float_script end_bigopsup => faux_wrap;
      bigopsubarg += start_bigopsub lone_float_script end_bigopsub => faux_wrap;
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
      // TWO OR MORE post-scripts, chained left-recursively. Perl's `addScripts`
      // (`MathGrammar` L419-423) recurses without a depth bound and without
      // caring which KIND of script it just consumed, so `{x^a}^b` (two supers)
      // and `{{x_a}^b}_c` (three scripts) are ordinary parses there. This used to
      // be hand-unrolled to exactly two, alternating —
      // `r12 postsuperarg | r11 postsubarg` — which made every same-kind repeat
      // and every chain of three or more `ltx_math_unparsed`. `x^a^b` without
      // braces is not a counter-example: TeX rejects that as "Double superscript"
      // long before the parser sees it, so the depth cap was buying nothing.
      //
      // Left recursion keeps the derivation unique — each script attaches to
      // everything to its left, in order — so this adds no ambiguity for the
      // grammar to prune. It also does not disturb the float-before-post
      // collection order noted above, which is about `scripted_factor_l*` vs
      // `_r*`, not about chain depth.
      scripted_factor_r2 = scripted_factor_r1 postsuperarg => postfix_script
        | scripted_factor_r1 postsubarg => postfix_script
        | scripted_factor_r2 postsuperarg => postfix_script
        | scripted_factor_r2 postsubarg => postfix_script;
      factor += scripted_factor_l1 | scripted_factor_l2 | scripted_factor_r1 | scripted_factor_r2;

      // Perl's trig application is one Factor whatever it takes (`preScripted['TRIGFUNCTION']
      // addTrigFunArgs`, MathGrammar:284, :562-567: `addEasyArgs` for any delimiters, :571-576, or a
      // `trigBarearg`, :340-356), so it follows other factors (`Term : Factor moreFactors`, :249-264):
      // `\cos\{x\}\sin\{y\}`, `2\sin\theta_i`, `r\cos\theta_i\sin\phi_j` (57bg; were unparsed:
      // `tight_term += trigfunction factor` could start a product but not continue one; 2605.20503,
      // 2605.09037, 2605.11904). `trig_arg` keeps the bare chains (`\sin 2x`).
      trig_factor_arg = function | fenced_array | fenced_factor
        | scripted_factor_l1 | scripted_factor_l2 | scripted_factor_r1 | scripted_factor_r2;
      applied_func += trigfunction trig_factor_arg => prefix_apply;
      // A scripted atom, identifier, unknown or number is a `trigBarearg` item too (Perl `aTrigBarearg`,
      // MathGrammar:341-348: `preScripted['ATOM_OR_ID']`, `NUMBER addScripts`), anywhere in the chain:
      // `\cos 2\theta_i` cos@(2·θ_i), `\sin\omega_0 t` sin@(ω₀·t) (57bo; were cos@(2)·θ_i and unparsed,
      // 2605.08634, 2605.22136, 2605.02925, 2605.05470). A scripted first item heads only a chain; alone
      // it is `trig_factor_arg` (one derivation each).
      trig_scripted_item = scripted_factor_l1 => trig_bare_argument_item
        | scripted_factor_l2 => trig_bare_argument_item
        | scripted_factor_r1 => trig_bare_argument_item
        | scripted_factor_r2 => trig_bare_argument_item;
      trig_chain_item = factor_base | trig_scripted_item;
      trig_arg += trig_arg trig_scripted_item => trig_argument_juxtaposition
        | trig_arg mulop trig_scripted_item => infix_apply_nary
        | trig_arg binop trig_scripted_item => infix_apply_nary
        | trig_scripted_item trig_chain_item => trig_argument_juxtaposition
        | trig_scripted_item mulop trig_chain_item => infix_apply_nary
        | trig_scripted_item binop trig_chain_item => infix_apply_nary;
      // A scripted trig function takes the bare one's arguments (Perl `preScripted['TRIGFUNCTION']
      // addTrigFunArgs`, MathGrammar:284, :430-433): `\sin^2x\cos^2y` (sin²)@(x)·(cos²)@(y), not
      // (sin²)@(x·(cos²)@(y)) — `scripted_trigfunction tight_term` took any product (57bo; 2605.01844,
      // 2605.28758, 2605.17056, 2605.25849).
      applied_func += scripted_trigfunction trig_arg => prefix_apply
        | scripted_trigfunction trig_factor_arg => prefix_apply;

      // Pre-scripts on post-scripted bases: _b(A^c), ^a(A_d^c), etc.
      // Must come after scripted_factor_r1/r2 are defined (forward reference not allowed).
      prescripted_factor_post_r += postsuperarg scripted_factor_r1 => prefix_script_pre
        | postsuperarg scripted_factor_r2 => prefix_script_pre;
      prescripted_factor_post_l += postsubarg scripted_factor_r1 => prefix_script_pre
        | postsubarg scripted_factor_r2 => prefix_script_pre;

      // Perl `OPERATOR addScripts nestOperators addOpFunArgs` is a Factor (MathGrammar:312-313):
      // the operator, scripted or nested (`compound_operator`), applies to what `addOpFunArgs`
      // reads (:553-558) — a parenthesized group, or `APPLYOP(?) barearg`: one `aBarearg`, or a
      // chain of them joined by juxtaposition or a MulOp, left-associative (`moreBareargs`,
      // :321-337). So `\nabla u\cdot v` is ∇@(u·v), `\nabla uv\cdot w` ∇@((u v)·w),
      // `\nabla_\theta\log\max_i p_i` ((∇_θ)@(log))@(max_i@(p_i)), and an operator applies mid-term
      // too: `\eta\nabla L(\theta)` is η·∇@(L)·θ, `k\nabla T` k·∇@(T) (optimisation and PDE
      // papers, 2605.19037, 2605.06657, 2605.25194, 2605.02202; golden
      // tests/parse/operator_application.tex#operator_takes_a_bare_argument).
      // `aBarearg` (:323-331) is a factor with no fence but `|…|`, no operator or big operator, no
      // speculative `f(x)`: `bare_argument_item` keeps those; `operator_bare_apply` takes no
      // operator and leaves a leading function or operator to an open nest; the narrower parses
      // are pruned in `apply_invisible_times` / `infix_apply_nary` (`leaves_a_bare_argument`,
      // `operator_takes`).
      // `nestOperators` (:663-671) nests operators until a function, scripted ones too (`OPERATOR
      // addScripts`, `FUNCTION addScripts`), and `recApply` (MathParser.pm:1313-1315) applies each
      // to the rest: `\nabla_x f^2` is (∇_x)@(f²), `\nabla_x\sin^2 x` ((∇_x)@(sin²))@(x),
      // `\nabla_x\nabla_y u` ((∇_x)@(∇_y))@(u) (golden
      // tests/parse/operator_application.tex#operator_nests_over_an_operator).
      compound_operator += operator scripted_function => prefix_apply
        | operator scripted_opfunction => prefix_apply
        | operator scripted_trigfunction => prefix_apply
        | operator scripted_operator => prefix_apply
        | scripted_operator scripted_function => prefix_apply
        | scripted_operator scripted_opfunction => prefix_apply
        | scripted_operator scripted_trigfunction => prefix_apply
        | scripted_operator operator => prefix_apply
        | scripted_operator scripted_operator => prefix_apply;
      // A scripted OPFUNCTION before a scripted group reads it as a limit's operand (divergence
      // #351): `\min_w(y-w)^2` is min_w@((y−w)²), `\min_j(y_j)_{j=1}^n` min_j@(((y_j)_{j=1})^n).
      applied_func += scripted_opfunction scripted_factor_r1 => scripted_group_apply
        | scripted_opfunction scripted_factor_r2 => scripted_group_apply;
      // `preScripted['UNKNOWN'] doubtArgs` (:327) as divergence #18 reads it (OXIDIZED_DESIGN_MATH):
      // an unknown applies to its group, alone or in a chain — `\operatorname{minimize} f(x)` is
      // minimize@(f@(x)), `\nabla f(x)` ∇@(f@(x)).
      speculative_item = unknown group_factor => speculative_prefix_apply
        | diffunk group_factor => speculative_prefix_apply;
      // A letter applied after an application (#18 applies it at the start of a product): after a
      // group its function closed nothing suggests a monomial — `P(A|B)P(B|C,D)`, `\Gamma(s)\zeta(s)`,
      // `\log(x)f(y)` (57bl; 1,607 formulas / 373 papers read `X@(…) * g * y`, 2605.09849, 2605.08899,
      // 2605.05133). A juxtaposed coefficient before a letter (`\lambda g(x)`, `2x(1+x)`) stays the
      // open #18 ruling (RED repro application_after_a_leading_factor). Only the left side of a
      // letter's application, so no tree gets two derivations; the fenced-letters pragma picks it.
      // The product that would leave the letter and its group separate factors is refused at once
      // (`factor_product`), mirroring these rules exactly: each chain of L letters otherwise also
      // stopped after any k, (L+1) readings per chain multiplying across a formula (57bl review: five
      // chains of three, 1,024 trees and 855 MB).
      delimited_application = function fenced_factor => prefix_apply
        | opfunction group_factor => prefix_apply
        | scripted_opfunction group_factor => prefix_apply
        | trigfunction fenced_factor => prefix_apply
        | scripted_trigfunction fenced_factor => prefix_apply;
      // The letter takes a group in parentheses or brackets only, as at the start of a product: a
      // brace, bar, floor or angle group after it multiplies (`U(t)H|\psi\rangle` H·ket, 57bn).
      application_before_a_letter = speculative_item => letter_application_to_a_group
        | delimited_application
        | tight_term delimited_application => apply_invisible_times
        | application_before_a_letter speculative_item => letter_after_an_application_apply;
      tight_term += application_before_a_letter speculative_item => letter_after_an_application_apply;
      op_bare_item = factor_base
        | function
        | speculative_item
        // An OPFUNCTION's group application with its scripts (divergence #351): `\log\exp(x)^2` is
        // log@((exp@(x))²).
        | scripted_opfunction_application => bare_argument_item
        // A trig function applied to a scripted argument or a group is an `applied_func` (57bg):
        // `\max_i\sin\theta_i` is max_i@(sin@(θ_i)) (2605.05043), `\max_j\cos(t,r_j)` max_j@(cos@(t, r_j)),
        // `\sum_i\ln\cosh(\cdot)` (2605.06229, 2605.08116, 2605.31371) — through `applied_func` below.
        | bare_abs
        | scripted_factor_l1 => bare_argument_item
        | scripted_factor_l2 => bare_argument_item
        | scripted_factor_r1 => bare_argument_item
        | scripted_factor_r2 => bare_argument_item
        | applied_func => bare_argument_item;
      // A bare function in a bare argument (Perl `aBarearg`'s OPFUNCTION/TRIGFUNCTION with no
      // argument of its own, :324-325): the whole argument (`\log\exp`), or the chain's last item
      // — before a bare item it would take it: `\max_A\tfrac12\log\det(B)` is
      // max_A@(½·log)·det(B), as Perl.
      // (A scripted OPFUNCTION is a factor already, `scripted_factor_r1/r2` → `op_bare_item`, 57bi.)
      bare_function_head = opfunction | trigfunction | scripted_trigfunction;
      op_bare_arg = op_bare_item op_bare_item => apply_invisible_times
        | op_bare_item mulop op_bare_item => infix_apply_nary
        | op_bare_item binop op_bare_item => infix_apply_nary
        | op_bare_arg op_bare_item => apply_invisible_times
        | op_bare_arg mulop op_bare_item => infix_apply_nary
        | op_bare_arg binop op_bare_item => infix_apply_nary
        | op_bare_item bare_function_head => apply_invisible_times
        | op_bare_arg bare_function_head => apply_invisible_times;
      // Perl `addOpFunArgs : APPLYOP(?) barearg` (MathGrammar:553-558) for an OPFUNCTION, bare or
      // scripted: the greedy chain of bare arguments an operator takes — `\log x y` is log@(x y),
      // `\max_i a_i b_i` max_i@(a_i b_i) (2605.10282, 2605.30776, 2605.24123, 2605.00332; golden
      // tests/parse/opfunction_arguments.tex#opfunction_argument_ends_at_its_group). A bare
      // function as the whole argument: `\log\exp` is log@(exp); `\log\exp x` log@(exp@(x)) (`\det`
      // is a LIMITOP, a big operator: `\log\det A` is log·det@(A), as Perl).
      applied_func += opfunction bare_function_head => operator_bare_apply
        | scripted_opfunction bare_function_head => operator_bare_apply;
      applied_func += opfunction op_bare_item => operator_bare_apply
        | opfunction op_bare_arg => operator_bare_apply
        | scripted_opfunction op_bare_item => operator_bare_apply
        | scripted_opfunction op_bare_arg => operator_bare_apply;
      op_head = operator | scripted_operator | compound_operator;
      op_application = op_head factor => operator_bare_apply
        // Divergence #18 in a single bare argument as in a chain: `\nabla f(x)` is ∇@(f@(x)).
        | op_head speculative_item => operator_bare_apply
        // Only a nest takes one applied function (an operator nests over it instead):
        // `\nabla\log\max_i p_i` is (∇@log)@(max_i@(p_i)).
        | compound_operator applied_func => operator_bare_apply
        | op_head op_bare_arg => operator_bare_apply;
      // An operator applied to a group applies to the next group too, `D(a)(b)` (D@(a))@(b) (Perl
      // `nestOperators`' OPEN branch then `addOpFunArgs` → `addEasyArgs`, MathGrammar:312-313,
      // :553-558, :669-671; Perl's own golden t/parse/operators.xml; 57bl, 40 `\nabla(…)(…)` formulas
      // in 9 papers, 2605.01526, 2605.01702, 2605.25503). Bare and scripted operators only: a nest
      // ending at a function takes one group (`\nabla\log(p)(q)`, as Perl).
      operator_group_application = operator group_factor => operator_bare_apply
        | scripted_operator group_factor => operator_bare_apply;
      op_application += operator_group_application group_factor => operator_application_apply;
      // … and a letter after it is applied, as after any application (57bl ruling).
      application_before_a_letter += operator_group_application
        | tight_term operator_group_application => apply_invisible_times
        // after a function or a closed nest, which take no operator (below): `\log\nabla(u)g(y)` is
        // log·∇@(u)·g@(y) (57bn; was unparsed since 57bl, the refused product its only reading)
        | opfunction operator_group_application => apply_invisible_times;
      tight_term += op_application;
      tight_term += tight_term op_application => apply_invisible_times;
      // A bare OPFUNCTION takes no operator (`aBarearg` has none) and multiplies it: `\log\nabla f`
      // is log·∇@(f), as Perl (`addOpFunArgs` returns the bare function, `moreFactors` goes on).
      // (A scripted OPFUNCTION is a factor, so `tight_term op_application` covers it, 57bi.)
      tight_term += opfunction op_application => apply_invisible_times;
      // A trig function takes no operator either (`aTrigBarearg` has none): `\sin\nabla(a-b)` is
      // sin·∇@(a−b), `a\sin\nabla u` a·sin·∇@(u), `\sin^2\nabla u` sin²·∇@(u), as Perl (57bo; were
      // unparsed or (sin²)@(∇@(u))). Built as the flat product `function_times_bigop` builds: the
      // left-function pruning of `apply_invisible_times` would refute the only reading.
      trig_head = trigfunction | scripted_trigfunction;
      tight_term += trig_head op_application => function_times_bigop
        | tight_term trig_head op_application => function_times_bigop;
      // … and a letter after the operator's application to a group is applied, as after an
      // OPFUNCTION's (57bn): `\sin\nabla(u)g(y)` sin·∇@(u)·g@(y) (57bo review; was unparsed).
      application_before_a_letter += trig_head operator_group_application => function_times_bigop;
      // An operator taking no argument is a Factor too (`addOpFunArgs`' `{ $arg[0]; }`), bare,
      // scripted or a nest, alone or after other factors — `a\nabla`, `2\nabla\log`, `\mu\nabla^2`,
      // `(u\cdot\nabla)u`, `\nabla\times\nabla\times u` — followed only by what it does not take: a
      // MulOp or the end, an operator's application after a closed nest (`\nabla\log\nabla^2 u` is
      // ∇@(log)·(∇²)@(u)), a big operator (below: `\nabla_x\log\det(A)` is
      // (∇_x)@(log)·det(A), 2605.03984, 2605.24401, 2605.25592, 2605.14289). It is no `tight_term`,
      // so no factor follows it: what the operator takes is its argument, not a product to prune
      // (repro math-parse/operator_terms_in_a_long_sum). Only an open nest before an operator is
      // still split and pruned (`tight_term op_head`, `bare_op_term op_head`: `\nabla\nabla` is
      // ∇@∇, not ∇·∇).
      bare_op_term = op_head
        | tight_term op_head => apply_invisible_times
        // after a closed nest: `\nabla\log\nabla^2` is ∇@(log)·∇² (an open one nests instead)
        | bare_op_term op_head => apply_invisible_times
        // after a function, which takes no operator (`aBarearg`): `\log\nabla^2` is log·∇²
        | opfunction op_head => apply_invisible_times
        | trigfunction op_head => apply_invisible_times;
      // Scripts' POSTFIX and evaluation bars apply to it as to any factor (`addScripts`,
      // MathGrammar:419-423; `evalAtOp`): `\nabla^2!`, `\nabla^2|_{x=0}`.
      tight_term += bare_op_term postfix => apply_postfix
        | bare_op_term singlevertbar postsubarg => eval_at
        | bare_op_term singlevertbar postsubarg postsuperarg => eval_at;
      term += bare_op_term
        | term mulop bare_op_term => infix_apply_nary
        | term binop bare_op_term => infix_apply_nary;
      tight_term += bare_op_term op_application => apply_invisible_times;
      // (and a letter after its application to a group is applied, as after an OPFUNCTION's, above)
      application_before_a_letter += bare_op_term operator_group_application => apply_invisible_times;

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
        | scripted_bigop term => prefix_apply;
      // A bigop is an operand of its own when no term follows it — Perl MathGrammar:292
      // `Factor : preScripted['bigop'] addOpArgs`, whose `addOpArgs` (:605-609; `addIntOpArgs`
      // :626-630) applies the bigop to a following Factor and otherwise yields it bare (`{ $arg[0]; }`).
      // A term is only ever followed by an operator, punctuation, a closer or `(`+relop, never by
      // a factor, so the bare reading never competes with an application: `\var + y` is
      // `variation + y`, `\var + \dd x` a sum, `y + \var`, `\var = 0`, `\partial_\mu + A_\mu`,
      // `a \var + b` all parse (RUST-ONLY: a bigop could only apply, or stand alone as a whole
      // statement, fenced singleton or left of `/`). This also covers Leibniz notation
      // `\partial/\partial t` (Perl `partial-differential / partial-differential@(t)`), which had
      // its own `/`-only rule — `\partial \times B` stays a flat product. Repro:
      // tools/perfect_kernel/repros/math-parse/diffop_before_addop_is_an_operand.tex.
      bigop_operand = bigop_application | any_bigop | scripted_bigop;
      // Lift bigop_application to term level (not expression level).
      // This avoids exponential Marpa ambiguity when ADDOP precedes BIGOP
      // (e.g. a+\neg b). At term level, `term addop expression` handles
      // `a + \neg b` with a single derivation path.
      // On its LEFT, invisible_times still works: 2∫ x dx via tight_term rules.
      // On its RIGHT, addop/relop follow naturally: ∫ x dx + y → ∫(x*dx) + y.
      term += bigop_operand;
      // Bigop after invisible times: 1/2∫ f dx → (1/2)*∫(f*dx)
      // Perl: Factor moreFactors handles consecutive factors via InvisibleTimes.
      // Since bigop_application is at term level (not tight_term), juxtaposition
      // between a tight_term and a bigop_application needs an explicit rule.
      term += tight_term bigop_operand => apply_invisible_times;
      term += bare_op_term bigop_operand => apply_invisible_times;
      // A function or operator, scripted or not, that STARTS a term before a bigop is a factor
      // of its own (Perl `Factor moreFactors`): `\min_\theta\sum_i \ell_i` is min_θ * ∑…,
      // `\log\int f` log * ∫f, `\nabla\int f` nabla * ∫f (witnesses 2605.02116, 2605.05081;
      // golden tests/parse/bigop_operands.tex#function_before_a_bigop_is_a_factor), and so is one
      // mid-term, after the factors before it (`2\sin\int f` is 2 * sin * ∫f, `x\nabla\int f` x *
      // nabla * ∫f; golden tests/parse/bigop_operands.tex#function_before_a_bigop_mid_term). A
      // `tight_term` on the left, not a `term`: `∫f \sin ∫g` would otherwise have two derivations.
      // Mid-term, an OPFUNCTION, bare or scripted, is not a `function_factor`: it is derivable
      // there already (a scripted one is a factor, `opfunction postsubarg`; `a\log\int f` parsed
      // before), so `\alpha\max_\theta\sum_i\ell_i` and `a\log\int f` are one derivation each
      // (`parse_tree_count_limits`).
      // An operator before a bigop is a `bare_op_term` (above), alone or mid-term.
      function_factor = function | trigfunction | opfunction
        | scripted_function | scripted_trigfunction | scripted_opfunction;
      midterm_function_factor = function | trigfunction
        | scripted_function | scripted_trigfunction;
      term += function_factor bigop_operand => function_times_bigop
        | tight_term midterm_function_factor bigop_operand => function_times_bigop;
      // Same but with explicit mulop: a * ∫ f dx → a * ∫(f*dx); ∂/∂t → ∂ / ∂(t)
      term += term mulop bigop_operand => infix_apply_nary;

      // Pre-scripted bigops: floating scripts before a bigop (Perl: preScripted)
      // Handles patterns like {}_a^b\sum_c^d x where floating scripts
      // attach as pre-scripts to the following operator.
      // Perl's parse_kludgeScripts_rec: FLOAT + POST pairs from same {} base
      // both become pre-scripts (POST gets forced 'pre' position without _wasfloat).
      prescripted_bigop_inner = scripted_bigop | any_bigop;
      // FLOAT script wrapping a bigop as pre-script
      // Perl: preScripted['bigop'] / preScripted['INTOP']
      // The rest of the chain after its leading FLOAT script: more FLOAT scripts, or POST
      // scripts used as pre-scripts (forced 'pre', no _wasfloat — Perl
      // parse_kludgeScripts_rec's NewScript($base, $y, 'pre') for a POST script that
      // follows a FLOAT from the same empty {} base). Only a FLOAT script may START the
      // chain: a POST script there is the preceding base's own (`\|f\|_2\int g` is
      // norm_2 * ∫g, not norm * (_2 ∫)g; witnesses 2605.05081, 2605.00581; golden
      // tests/parse/bigop_operands.tex#base_script_is_not_a_bigop_prescript).
      prescripted_bigop_tail = prescripted_bigop_inner
        | floatsuperarg prescripted_bigop_tail => prefix_script
        | floatsubarg prescripted_bigop_tail => prefix_script
        | postsuperarg prescripted_bigop_tail => prefix_script_pre
        | postsubarg prescripted_bigop_tail => prefix_script_pre;
      prescripted_bigop = floatsuperarg prescripted_bigop_tail => prefix_script
        | floatsubarg prescripted_bigop_tail => prefix_script;
      // A pre-scripted bigop is a bigop like any other (Perl MathGrammar:292
      // `Factor : preScripted['bigop'] addOpArgs`): it applies to the term after it, a
      // following bigop included (`{}^a\sum\sum b`), and is an operand of its own
      // (`{}^a\sum + b`, the bare statement). Golden:
      // tests/parse/bigop_operands.tex#prescripted_bigop_is_an_operand.
      bigop_application += prescripted_bigop term => prefix_apply;
      bigop_operand += prescripted_bigop;

      // Perl MathGrammar L259-260: moreFactors: evalAtOp maybeEvalAt
      // "evaluated at" — a|_{x=0}, f(x)|_{x=0}^{x=1}, \left.xyz\right|_{0}^{2}
      // evalAtOp: VERTBAR:| (standalone pipe) — CLOSE:| from \right| handled separately
      tight_term += tight_term singlevertbar postsubarg => eval_at
        | tight_term singlevertbar postsubarg postsuperarg => eval_at
        | tight_term singlevertbar postsuperarg postsubarg => eval_at
        // CLOSE:| from \right| also triggers eval-at
        | tight_term close_pipe postsubarg => eval_at
        | tight_term close_pipe postsubarg postsuperarg => eval_at
        | tight_term close_pipe postsuperarg postsubarg => eval_at
        // \left.expr\right|_… — the closing \right| is stretchy VERTBAR
        // tagged RIGHT_STRETCHY_VERTBAR by the lexer (util.rs) via the
        // `role_side="right"` property set in `\lx@delim@right` (tex_math.rs).
        // The `\left.` null-delimiter doesn't appear in the lexeme
        // stream, so the resulting shape is `tight_term RIGHT_STRETCHY_VERTBAR
        // postsubarg`. The undirected variant covers legacy DOM input
        // or code paths that bypassed `\lx@delim@right`.
        | tight_term right_stretchy_vertbar postsubarg => eval_at
        | tight_term right_stretchy_vertbar postsubarg postsuperarg => eval_at
        | tight_term right_stretchy_vertbar postsuperarg postsubarg => eval_at
        | tight_term undirected_stretchy_vertbar postsubarg => eval_at
        | tight_term undirected_stretchy_vertbar postsubarg postsuperarg => eval_at
        | tight_term undirected_stretchy_vertbar postsuperarg postsubarg => eval_at;

      anyop = addop | mulop | binop | relop | arrow | metarelop
        | bigop | sumop | intop
        | limitop | diffop | vertbar | supop
        | modifierop | operator | compound_operator;

      anyscript = floatsuperscript | floatsubscript
        // Standalone floating script pairs (no base: {}^c_d or {}_d^c)
        // Perl: NewScript(NewScript(Absent(), super, 'post'), sub, 'post')
        | floatsuperscript postsubarg => postfix_script
        | floatsubscript postsuperarg => postfix_script;

      // Operators that CANNOT start a valid expression — leading orphans
      // from tabular fragments where LHS is on a preceding row.
      // Excluded: addop (prefix ±x), relop (prefix =x), arrow, bigop/sumop/intop/diffop.
      // These already have valid prefix interpretations inside expressions (a bare
      // diffop is an operand, `bigop_operand`, so `\var = 0` has one reading).
      orphan_op = mulop | binop | supop | modifierop;
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

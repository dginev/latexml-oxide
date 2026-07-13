# Math Grammar from First Principles

Two complementary perspectives for reasoning about the Marpa math grammar
design. The goal is a grammar that accepts every mathematically meaningful
LaTeX formula exactly once — keeping ambiguity bounded to where
mathematical practice is inherently ambiguous, and resolving grammatical
ambiguity as early as possible.

---

## 1. Top-Down View: Category Hierarchy and Subexpression Taxonomy

We are modeling **visible math notation as written by humans** — not the
underlying mathematical object itself, but its *printed / typeset form*.
The grammar consumes a linear stream of tokens (each carrying a role like
NUMBER, UNKNOWN, TRIGFUNCTION, ADDOP, OPEN, CLOSE, SUPSCRIPT, etc.) and
produces a structured XML tree whose shape preserves both the surface
presentation and the inferred mathematical meaning.

### Principal categories, from large to small

A compact taxonomy with four levels is enough to cover standard math:

**(L0) Top-level: statement.** Something that can stand alone in a
display: a single formula, a relation like `a = b + c`, a multi-line
alignment, or a system (`\begin{cases}`), or a comma-separated list of
statements in a parenthesized tuple. A statement is closed off by a top
relation (`=`, `≠`, `≈`, `<`, `≤`), by a metarelation (`↔`, `:`) or by
the end of the formula.

**(L1) Formula / expression.** The content of *one side* of a relation.
An expression denotes a single mathematical value, built from *terms*
combined by additive operators and similar peer-level operators. A
formula may chain relations (`a ≤ b < c`) into a compound statement.

**(L2) Term.** Each additive summand of an expression — a product or
single factor. Terms combine by multiplicative operators: explicit
(`*`, `\cdot`, `\times`) or invisible (juxtaposition). A term may also
be a bigop application (`\int f dx`), a standalone function symbol, or
a single factor that stands alone.

**(L3) Factor.** The atomic unit of a term after function application
has been resolved. A factor is one of: an atom (single identifier or
number), a scripted atom (`a^2`, `a_i`, `a^i_j`), a fenced expression
(`(…)`, `[…]`, `{…}`, `⟨…⟩`), a function application (`f(x)`, `\sin(x)`),
or a composed factor (`\bar{a}`, `\vec{v}`, etc.).

### Subexpressions and their uniqueness constraint

A grammar is *unambiguous* when every accepted input has exactly one
derivation tree. For math notation, three kinds of ambiguity show up:

1. **Structural**, resolvable by precedence and associativity:
   `a + b * c` has two candidate interpretations until we declare that
   `*` binds tighter than `+`. This is cheap to encode.
2. **Compositional**, resolvable by requiring one category per slot:
   `sin x` should parse once — sin applied to x. It should NOT also
   parse as "the product of `sin` and `x`". We resolve by making
   trig-application a more specific rule than implicit multiplication.
3. **Semantic**, genuinely ambiguous in practice: `2x` could be product
   or a single mixed-number object. `a(b)` with `a` a variable vs a
   function is a semantic distinction that no grammar can decide.

**The grammar's job is to collapse types 1 and 2 to a single parse
tree and to keep type 3 ambiguity to the minimum unavoidable number
of trees** — ideally by using sparse, disjoint alternatives at each
category level. The dedup-at-semantic-layer approach we've been using
is a fallback when the grammar can't disambiguate earlier; it is
strictly more expensive because Marpa enumerates all trees in the ASF
before we filter.

### Top-down design principle

A compact taxonomy minimizes the number of nonterminal categories an
input token can flow through, and therefore the number of derivation
paths Marpa must explore. Each "unifying" non-terminal (`tight_term`,
`factor`, `term`) should be defined with *one* canonical decomposition;
augmentations (`+=`) should add genuinely new shapes, not parallel
paths to the same shape. The rule of thumb: if two categories accept
the same token sequence, merge them or disambiguate by introducing a
tighter category boundary.

---

## 2. Bottom-Up View: Primitives, Precedence, and Composition

From the lexer we receive a stream of classified tokens. Reasoning
bottom-up, the grammar is a recipe for building meaning from those
atoms.

### Atoms and primitives

The atomic tokens are the leaves of every parse tree. In LaTeXML's
lexer we categorize them by *mathematical role*, not by visual shape:

- **Identifiers:** UNKNOWN (user-declared variable), ID (atomic
  identifier like `\alpha`), NUMBER, ATOM (digested sub-expressions,
  e.g. already-parsed exponent contents).
- **Operator symbols:** ADDOP (`+`, `-`), MULOP (`*`, `·`, `×`, and
  invisible times `⁢`), BINOP (generic math binary, `\mathbin`), RELOP
  (`=`, `<`, `≤`), METARELOP (`:`, `↔`), APPLYOP (invisible function
  application `⁡`).
- **Applied identifiers (named functions):** FUNCTION (f, g, arbitrary
  user-declared), OPFUNCTION (`\log`, `\det`), TRIGFUNCTION (`\sin`,
  `\cos`, `\tan`), BIGOP / SUMOP / INTOP / LIMITOP.
- **Scripts as tokens:** `start_POSTSUPERSCRIPT … end_POSTSUPERSCRIPT`
  (and sub/float variants) bracket script content into a single
  sub-stream that the parser lifts into a script argument.
- **Delimiters:** OPEN/CLOSE with shape (paren, bracket, brace,
  angle), plus specialized `|` (VERTBAR) for kets and norms.
- **Punctuation:** PUNCT (`,`, `;`) for lists; tokens like ELIDEOP
  (`\cdots`) that participate as pseudo-elements.

The key distinction: **applied identifiers** (FUNCTION, OPFUNCTION,
TRIGFUNCTION, BIGOP) *require* an argument to close themselves off,
while **visual operator symbols** (ADDOP, MULOP, RELOP, …) need
*both* operands. This asymmetry is the source of most grammar
ambiguity: `f g` reads as `f · g` if both are FUNCTION, as `f(g)` if
we speculate application, as `F(g)` if `f` is OPFUNCTION. Perl's
LaTeXML encodes this as the FUNCTION / OPFUNCTION distinction —
FUNCTION requires parens, OPFUNCTION absorbs bare args.

### Operator precedence (tightest → loosest)

From tightest-binding to loosest:

1. Script attachment (`^`, `_`) — attaches to a single factor.
2. Function application — explicit via APPLYOP or implicit via
   adjacency for FUNCTION / OPFUNCTION / TRIGFUNCTION.
3. Multiplication — explicit MULOP or juxtaposition.
4. Additive BINOP — `+`, `-`.
5. Relations — `=`, `≤`, `∈`.
6. Meta-relations — `:`, `↔` (only loose in specific idioms).
7. Punctuation — `,`, `;` in tuples and argument lists.

Each level should have **one** entry-point category. Any rule that
crosses a level boundary should be an explicit conversion, not a
parallel alternative. That forces the grammar into a layered shape,
keeping derivation counts bounded.

### Invisible juxtaposition

The trickiest operator. Two adjacent factors mean different things:

- `2x` = `2 · x` — coefficient times variable.
- `f(x)` = `f` applied to `x` when `f` is a function-role token.
- `a b` inside `\sin a b` = `\sin(a · b)` (OPFUNCTION absorbs).
- `\sin f(x)` = `sin(f(x))` — trig absorbs the entire applied
  function.

We model this by giving each *applied-identifier* token its own
category that feeds into a common `applied_func` rule, and by making
implicit multiplication the fallback only when applied forms don't
match. The grammar must be layered so that `trigfunction X` wins over
`X · sin`: the trig rule is at a tighter binding than the implicit
multiplication rule.

### Delimiters and grouping

Grouping is how the writer signals "treat this as a single factor".
We have four delimiter shapes:

- Visible math parens `(…)` / `[…]` — create a **fenced expression**
  (fenced_factor) that participates as a term/factor.
- Curly braces `{…}` — visually invisible, but lexically close a
  sub-formula. The grammar treats `{expr}` as equivalent to `expr` at
  the factor level.
- Angle brackets `⟨…⟩` — semantically a pair, typically for inner
  products or kets.
- Implicit groups formed by scripts (`^{…}`, `_{…}`) — handled by the
  lexer emitting `start_/end_POSTSUPERSCRIPT` tokens around the
  content, so the parser sees a single script argument.

A compact grammar uses **one** non-terminal per role: one
`fenced_factor` regardless of the outer delimiter, disambiguated later
if the XML output needs different shapes. Every parallel alternative
that builds the same semantic shape *must be eliminated* — this is the
primary source of the 2^N ambiguity explosions we have been fighting.

### Bottom-up design principle

Give every token-combination exactly one shortest path through the
grammar. Where multiple rule alternatives look similar, either prove
they produce semantically different trees (different constructors) or
collapse them into a single form. Concretely: if rule `A : B C` and
rule `A : D E` would both match the same input through distinct
derivations that produce the same XML, then one of them must go.

---

## Using these views together

Top-down gives us the **taxonomy** — the layered nonterminal shape the
grammar should have. Bottom-up gives us the **composition rules** —
how each token / token-sequence rises through the layers. The grammar
is healthy when:

1. Every token sequence that is syntactically valid has exactly one
   path through the layered taxonomy.
2. Where mathematical practice is inherently ambiguous (`2x` =
   product vs. mixed atom), the grammar emits 2 trees, labeled so the
   semantic pass can pick. No accidental multipliers.
3. Adding a rule to an existing category is explicitly checked
   against the ambiguity budget: each parallel alternative must
   demonstrate a semantically distinct output.

The recent fixes have been attacking (3) — removing parallel rules
that produced the same tree through different paths. Remaining work
is to re-derive the core nonterminal structure from (1) and prune
rules that break the layering.

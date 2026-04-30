# Rewrite Subsystem Audit: Perl Rewrite.pm vs Rust rewrite.rs

> **Status: technical reference (snapshotted 2026-03-28, Session 59).**
> The Rewrite subsystem has been substantively ported (see
> `archive/TRANSLATION_GAPS.md` Section 4). The audit below documents the
> remaining R1–R15 divergences in detail; check current code before
> acting on any "MISSING" / "WRONG" classification, since some have
> been fixed since the snapshot.
>
> Current priority is tracked in [`SYNC_STATUS.md`](SYNC_STATUS.md)'s
> dashboard.

> **Date:** 2026-03-28 (Session 59)
> **Perl source:** `LaTeXML/lib/LaTeXML/Core/Rewrite.pm` (561 lines)
> **Rust source:** `latexml_core/src/rewrite.rs` (1091 lines)
> **Rust helper:** `latexml_package/src/package/latexml_sty.rs` (compile_declare_pattern)

## Methodology

Side-by-side comparison of every operation (compilation + execution) in both implementations.
Each finding categorized as MISSING, WRONG, or MINOR.

---

## 1. Compilation Phase

### R1. MISSING: Match → XPath compilation (HIGH)

**Perl (L334-372):** `compile_match()` → `compile_match1()` digests tokens into a temporary DOM fragment, runs `domToXPath()` to produce XPath + wildcard paths.

**Rust (L274-285):** If `Match` pattern is a `String`, passes it through as `Select`. If it's `Tokens`, the tokens are never digested into DOM and never compiled to XPath. Only `compile_declare_pattern` in latexml_sty.rs provides a string-based shortcut for `\lxDeclare` patterns.

**Impact:** Any rewrite rule using `match => Tokens(...)` (rather than pre-compiled XPath) will produce no matches. The full `domToXPath` pipeline exists in Rust but is never invoked from compilation.

### R2. MISSING: Math decoration filter (MEDIUM)

**Perl (L362-366):** For math-mode rewrites, appends `[@_pvis and @_cvis]` to XPath. This filters "decoration" nodes that are only visible in one arm of an XMDual (presentation-only or content-only).

**Rust:** No equivalent filter. Math patterns can match decoration-only nodes.

**Impact:** Overmatching in math patterns, especially in contexts with XMDual structures. The `_pvis`/`_cvis` attributes are set by `markXMNodeVisibility` which is in `Document.pm` and has a Rust counterpart, but the filter is not applied.

### R3. MINOR: Select pattern stores (xpath, nnodes, wilds) vs global options

**Perl (L93):** Select pattern is `[$xpath, $nnodes, @wilds]` — a tuple. Each Select clause carries its own `$nnodes` and `@wilds`.

**Rust (L328-384):** Select pattern is `String` (xpath only). `nnodes` is in `self.options.select_count`, `wilds` is in `self.options.wildcard_paths` — shared across all clauses.

**Impact:** For single-Select rules (99% of cases), equivalent. For multi_select or rules with multiple Select clauses, the global options can't represent per-clause differences.

---

## 2. Execution Phase — Operator Semantics

### R4. WRONG: Test operator return value ignored (HIGH)

**Perl (L114-118):**
```perl
my $nnodes = &$pattern($document, $tree);
$self->applyClause($document, $tree, $nnodes, @more_clauses) if $nnodes;
```
Closure returns node count. If 0/undef, remaining clauses are SKIPPED.

**Rust (L547-556):** Closure return value is ignored. Always continues with original `nmatched`.

**Impact:** Test clauses act as no-ops instead of conditional gates. Rules relying on Test to filter will apply to ALL matched nodes.

### R5. WRONG: Regexp doesn't modify text (MEDIUM)

**Perl (L171-178):** Finds ALL `descendant-or-self::text()` nodes, applies pattern (a closure that modifies the string), calls `$text->setData()`.

**Rust (L509-520):** Gets content of the ROOT node only, tests regex match, and if matched continues to next clause. Does NOT traverse descendants. Does NOT modify any text nodes.

**Impact:** Regexp rewrite rules don't substitute text. Only used by a few packages (e.g., some contrib bindings), so practical impact is limited.

### R6. WRONG: MultiSelect structure mismatch (MEDIUM)

**Perl (L103-113):** Pattern is array of `[$xpath, $nnodes, @wilds]` tuples. Each sub-pattern has independent node count and wildcards.

**Rust (L558-569):** Pattern is a single `String` xpath. All matches use the same `select_count`.

**Impact:** Rules with multiple independent match patterns can't express per-pattern node counts.

### R7. MINOR: Action closure signature

**Perl (L156):** `&$pattern($tree)` — passes only the matched node.

**Rust (L540-542):** `closure(document, vec![&mut node])` — passes document + node vector.

**Impact:** Rust closures are Rust-defined and use the Rust API, so this is a design difference, not a bug. Any Rust Action closures must use the `(&mut Document, Vec<&mut Node>)` signature.

### R8. MINOR: ownerDocument check missing

**Perl (L98):** `next unless $node->ownerDocument->isSameNode($tree->ownerDocument)`

**Rust:** No equivalent check in Select handler.

**Impact:** Could process detached or cross-document nodes. Low practical risk since all operations use single documents.

---

## 3. WildCard System

### R9. MISSING: Font predicate generation (HIGH for \lxDeclare)

**Perl (L487-490, Font.pm L493-508):** For nodes that can have a `font` attribute, generates XPath predicate from `_font` value using `font_match_xpaths()`. Checks family, series, shape (ignores size, color, encoding, language).

**Rust:** No font predicate in XPath. The `_font` attribute is excluded from match predicates (L607). Partial workaround: `declare_node_matches` can check base text but not font.

**Impact:** `\mathbf{x}` incorrectly matches declaration for `$x$` (bold vs non-bold not distinguished). Any font-sensitive pattern matching fails. This is the root cause of ~50 declare.xml diffs.

### R10. MINOR: Attribute exclusion differences

**Perl (L423-425):** Excludes: `scriptpos`, `mathstyle`, `xml:id`, `fontsize`.

**Rust (L607):** Excludes above plus `_font`, `_pvis`, `_cvis`, and ALL `_`-prefixed attributes.

**Impact:** Rust generates fewer XPath predicates (more permissive matching). The `_`-prefix exclusion is correct since internal attributes shouldn't affect matching, but it diverges from Perl's explicit list.

### R11. WRONG: XMDual content arm structure

**Perl (L204-216):** Creates XMDual as:
1. `wrapNodes('ltx:XMWrap', @nodes)` → wraps original nodes in XMWrap
2. `wrapNodes('ltx:XMDual', $wrapper)` → XMDual wraps XMWrap
3. Creates XMApp child with XMTok + XMRef (presentation arm)
4. Removes XMWrap from XMDual, appends it back (reorders)
5. Result: first child = XMApp (content/semantic), second = XMWrap (presentation)

**Rust (L934-955):** Creates XMDual differently:
1. `wrap_nodes("ltx:XMDual", nodes)` → wraps original nodes directly
2. Creates XMApp and inserts as FIRST child before original nodes
3. Result: first child = XMApp (presentation), remaining = original nodes (content)

**Impact:** Child ordering differs (content-first vs presentation-first). The original nodes are NOT wrapped in XMWrap. This produces structurally different XMDual trees that may not compact correctly.

---

## 4. Post-processing (MISSING in Rust)

### R12. MISSING: pruneXMDuals / collapseXMDual / compactXMDual

**Perl (Document.pm L1565-1633):**
- `pruneXMDuals()`: After all rewrites, scans all XMDual nodes
- `collapseXMDual()`: If content or presentation arm has no visible nodes, replaces XMDual with the survivor
- `compactXMDual()`: If both arms are mirror XMApp nodes, merges them by combining attributes
- Uses `markXMNodeVisibility()` to set `_pvis`/`_cvis` on nodes

**Rust:** `compact_xmdual` exists in document.rs but `pruneXMDuals` is not called after rewrite processing. The optimization in R14 (skip XMDual when wildcard IDs empty) partially compensates.

**Impact:** Redundant XMDual nodes remain in the tree. The math parser handles `_rewrite`-marked nodes specially, but the extra nesting can affect output structure.

### R13. MISSING: markXMNodeVisibility

**Perl (Document.pm):** Recursively marks nodes as `_pvis` (presentation-visible) or `_cvis` (content-visible) based on XMDual arm membership. Used by pruneXMDuals and the decoration filter (R2).

**Rust:** Not implemented as a separate pass. The `_pvis`/`_cvis` attributes are excluded from XPath matching but never set.

---

## 5. \lxDeclare-Specific Issues (latexml_sty.rs)

### R14. PARTIALLY FIXED: Pattern compilation

**Perl:** `compile_match1()` digests pattern tokens → DOM → `domToXPath()` for ANY pattern.

**Rust (latexml_sty.rs L11-155):** `compile_declare_pattern()` does string-based pattern recognition. Handles:
- Simple tokens: `x` ✓
- Wildcard subscripts: `x_{\WildCard}` ✓
- Braced multi-wildcards: `x_{\WildCard,\WildCard}` ✓
- Literal subscripts: `x_{1}` ✓ (NEW in session 59)
- Prime patterns: `x^{\prime}` ✓ (NEW in session 59)
- Literal accents: `\hat{x}` ✓ (NEW in session 59)
- Wildcard accents: `\hat{\WildCard}` ✓

**Not handled:**
- Function application: `f\WildCard[(\WildCard)]` → returns empty (nowrap patterns)
- Multi-pattern: `\WildCard[a]b` → returns empty
- Arbitrary patterns: anything not matching known forms

### R15. PARTIALLY FIXED: Rust-side filtering for XMApp patterns

**Rust (rewrite.rs declare_node_matches):** Added in session 59. Filters XMApp matches by:
- Subscript: checks base child text, optional literal subscript text
- Prime: checks base child text, prime superscript content
- Accent: checks accent name in first child, optional base text

**Not implemented:**
- Font matching (see R9)
- Complex nested pattern matching

---

## Summary: Actionable Items by Priority

### Must-fix for declare.xml parity
1. **R9** Font predicate — \mathbf{x} false match (~50 diffs)
2. **R11** XMDual content arm structure — structural diffs for every XMDual
3. **R12** pruneXMDuals — redundant XMDual nodes in output

### Must-fix for general rewrite correctness
4. **R4** Test operator return value — conditional gates broken
5. **R1** Match → XPath compilation — needed for non-\lxDeclare match patterns

### Should-fix for completeness
6. **R5** Regexp text modification
7. **R6** MultiSelect per-pattern counts
8. **R2** Math decoration filter
9. **R13** markXMNodeVisibility

### Low priority
10. **R3** Per-clause nnodes/wilds
11. **R8** ownerDocument check
12. **R10** Attribute exclusion normalization

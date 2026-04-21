# Performance Optimization Principles

> **Repeatable checklist.** Review before release milestones, after major features, and during periodic optimization passes. Each principle includes rationale and examples.

---

## 1. Avoid String Allocation on Hot Paths

**Principle:** Never use `.to_string()`, `String::from()`, or `format!()` when a string is already in the interner arena. For string *literals*, prefer the call-site-cached `pin!("…")` macro — it interns once per call site and resolves subsequent lookups with a `u32` load. For runtime strings, use `arena::pin(s)`. For comparisons and reads, use the `arena::with*` family.

**Why:** String allocation is one of the most frequent hidden costs. The arena interner exists precisely to avoid repeated heap allocations for the same string. Every `.to_string()` on an interned symbol defeats this purpose. `pin!("foo")` caches the interned `SymStr` in a per-site `thread_local! OnceCell`, avoiding the 30-50 ns intern-table probe on subsequent calls (just a branch + load).

**Examples:**
```rust
// BAD: allocates a new String just to compare
if arena::to_string(sym) == "foo" { ... }

// BETTER: resolve in-place without allocation
if arena::with(sym, |s| s == "foo") { ... }

// BEST (for literals): compare interned SymStrs directly
if sym == pin!("foo") { ... }

// BAD: allocate to store under a literal key
let name = "some_key".to_string();
state::assign_value(&name, ...);

// GOOD: use the sym-keyed state API with pin!
state::assign_value_sym(pin!("some_key"), ..., None);
```

**When to apply:** Any code that runs per-token, per-macro-expansion, per-digest, or per-node. State lookups, token comparisons, CS name checks. The sym-keyed state API (`lookup_bool_sym`/`assign_value_sym`/`with_value_sym`) takes `SymStr` *by value* — `SymStr` is a `u32` wrapper (Copy), so passing by value is cheaper than borrowing.

**`pin!` vs `arena::pin`:**
- `pin!("literal")` (macro) — per-site OnceCell cache, for string literals. Cheapest.
- `arena::pin(runtime_str)` (function) — single intern on a dynamic `&str` / `String`.
- Both return the same `SymStr` on equal input; the macro is just a fast path for known-at-compile-time strings.

---

## 2. Minimize `.clone()` — Borrow or Reorder Instead

**Principle:** Avoid `.clone()` for data that can be borrowed or is only needed for a short instruction scope. Rearrange code so that short flag-like methods run first and assign to local variables, preventing Rust ownership conflicts without cloning. For more complex cases, consider borrowing from first principles, using the string interner, or a `Cow<>` copy-on-write approach.

**Why:** Cloning complex structures (Tokens, Vec, HashMap) is expensive. Many clones exist only to satisfy the borrow checker, not because the data is truly needed in two places. Reordering operations or extracting small values first often eliminates the need entirely.

**Examples:**
```rust
// BAD: clone to avoid borrow conflict
let tokens = state.get_tokens().clone();
let flag = state.is_active();  // borrows state again
process(tokens, flag);

// GOOD: extract flag first (short borrow), then get tokens
let flag = state.is_active();
let tokens = state.get_tokens();  // no conflict now
process(tokens, flag);

// BAD: clone a large structure for a single read
let map = self.entries.clone();
if map.contains_key("foo") { ... }

// GOOD: borrow directly
if self.entries.contains_key("foo") { ... }

// ACCEPTABLE: Cow for conditional mutation
fn process(input: Cow<str>) -> Cow<str> {
    if needs_change(&input) { Cow::Owned(transform(&input)) }
    else { input }
}
```

**When to apply:** Every `.clone()` call is a candidate for review. Prioritize clones inside loops, hot paths, and clones of `Tokens`, `Vec<Token>`, `HashMap`, or `String`. Single-field scalar clones (bool, u32, Option<usize>) are fine.

---

## 3. Run Clippy and Study Lint Neighborhoods

**Principle:** Run `cargo clippy` and apply all fixes. Then study code in the vicinity of each lint — performance issues often cluster in clumps of poorly designed code. A clippy lint is a signal to review the surrounding 20-50 lines.

**Why:** Clippy catches redundant allocations, unnecessary conversions, suboptimal patterns (e.g., `map().flatten()` → `flat_map()`, `.iter().cloned().collect()` → `.to_vec()`). But more importantly, lint locations correlate with code that was written hastily. Fixing the lint often reveals adjacent inefficiencies that clippy doesn't flag.

**How to run:**
```bash
cargo clippy --workspace -- -W clippy::perf -W clippy::redundant_clone
```

**When to apply:** Before every release milestone. After major feature work. As a periodic sweep (monthly or quarterly). Focus on `clippy::perf` and `clippy::redundant_clone` warning groups.

---

## 4. Minimize Math Parse Ambiguity

**Principle:** Reduce the number of successful math parses while ensuring at least one survives. This is the highest-leverage optimization for documents with heavy math. Three complementary tools, ordered by effectiveness:

**Tool 1 — Grammar restructuring (highest impact):**
Restrict grammar category scopes, reduce the number of paths an expression can take toward the start category. Fewer ambiguous derivations = exponentially fewer parse trees.

**Tool 2 — Semantic pruning (high impact):**
Return `Err` from `semantics.rs` action functions for nonsensical constructions. This prunes during tree construction, before full parse completion. Examples: reject `f(x)(y)` as double-application when `f` isn't higher-order, reject mismatched fence pairs, reject empty operator sequences.

**Tool 3 — Pragmatic rules (representation quality):**
Add `Pragma` rules that check mathematical conventions (e.g., consistency of variable use, operator precedence expectations). Less useful for raw performance (all parses must complete first) but valuable for selecting the best parse from surviving candidates.

**Why:** The Marpa parser produces all valid derivations. For ambiguous math like `a+b*c/d`, the number of parse trees can be combinatorial. Each surviving parse is a full tree that consumes memory and CPU. Reducing parses from 50 to 3 can be a 10-20x speedup on math-heavy documents.

**When to apply:** When profiling shows math parsing as a bottleneck (common for documents with 100+ formulas). During grammar design reviews. When adding new math operators or constructs.

---

> **Adding new principles:** Number sequentially. Include: principle statement, **Why** (rationale), **Examples** (good vs bad code), **When to apply** (scope/triggers).

---

## Standing Performance Corpus

The following papers form the regression corpus for engine / arena / gullet /
marpa changes. Run with a direct idle-serial invocation (no `cortex_worker`,
no parallel load):

```bash
/home/deyan/git/latexml-oxide/target/release/latexml_oxide \
  --preload=ar5iv.sty \
  --path=/home/deyan/git/ar5iv-bindings/bindings \
  --dest=/tmp/out.html --timeout=60 <main.tex>
```

Papers live as zipped sources under `/home/deyan/data/10k_sandbox/<id>.zip`;
`complex/si.tex` is in-tree at `latexml_oxide/tests/complex/si.tex`. The
helper script `tools/run_perf_corpus.sh` unzips each into a tmpdir and
records `exit` + wall-clock.

### Round-17 baseline (2026-04-21)

| paper          | main.tex                           | dt (s) | class / note                    |
|----------------|------------------------------------|-------:|----------------------------------|
| 0906.1883      | VanNeervenWeis_final_version.tex   |  0.67  | aa, birkmult (stub-guard fix)   |
| 1011.1955      | 1011.1955.tex                      |  3.49  | amsart `\DeclareMathSymbol`     |
| 1009.1431      | 1009.1431.tex                      |  2.11  | —                                |
| 1008.4386      | genealogy_final_CPAM.tex           |  2.59  | —                                |
| 0909.2656      | main.tex                           |  2.74  | —                                |
| 0911.4739      | lhc7.tex                           |  5.04  | JHEP — over 3s                  |
| 1005.1610      | OAM100507.tex                      |  7.38  | iopart — over 3s                |
| 0803.0466      | IIpaper15.tex                      |  2.31  | aa                               |
| complex/si.tex | si.tex                             |  2.06  | siunitx-heavy                    |

### Round-17 refresh (2026-04-21, after `aa3c7c1bb` graphics parallelism)

| paper          | main.tex                           | dt (s) | Δ vs baseline |
|----------------|------------------------------------|-------:|--------------:|
| 0906.1883      | VanNeervenWeis_final_version.tex   |  0.69  |   +3% (noise) |
| 1011.1955      | 1011.1955.tex                      |  3.49  |         flat  |
| 1009.1431      | 1009.1431.tex                      |  2.11  |         flat  |
| 1008.4386      | genealogy_final_CPAM.tex           |  2.69  |   +4% (noise) |
| 0909.2656      | main.tex                           |  1.95  |          −29% |
| 0911.4739      | lhc7.tex                           |  1.71  |          −66% |
| 1005.1610      | OAM100507.tex                      |  2.57  |          −65% |
| 0803.0466      | IIpaper15.tex                      |  1.35  |          −42% |
| complex/si.tex | si.tex                             |  2.22  |   +8% (noise) |

All Tier A papers now under 3.5 s — round-17 outliers resolved.
Commit `aa3c7c1bb` parallelises the Graphics phase's `convert` subprocess
fork-execs via `std::thread::scope` (no new dependency) with a worker
cap of `min(available_parallelism, 8)`.

### Regression trigger

Any corpus entry drifting wall-clock **> +15%** from its last recorded
baseline between commits is a regression signal. Record the new row in a
dated sub-heading here (don't overwrite); keep the old baseline so the drift
is visible in history.

### Known outliers (as of round 17)

- **0911.4739** and **1005.1610** exceed the old "all under 3s" claim from
  the session 124 memory. Either workload drift (new markup or upstream
  engine changes) or cold-bench sensitivity. Candidate papers for the next
  perf investigation cycle.

### Validation: pathological-for-ImageMagick PDFs (issue #902)

The vector-SVG graphics path (opt-in via `--graphics-svg-threshold-kb N`,
round 17) is validated against `fig8.pdf` from
[brucemiller/LaTeXML#902](https://github.com/brucemiller/LaTeXML/issues/902)
(attached from arxiv:1807.01606), a 41 KB vector-authored PDF that
`convert` rasterises absurdly slowly.

End-to-end through `latexml_oxide --post` on a minimal 4-line document
containing only `\includegraphics{fig8.pdf}`:

| path                                | Graphics phase | total wall |
|-------------------------------------|---------------:|-----------:|
| default (ImageMagick `convert`)     |       32.4 s   |    32.4 s  |
| `--graphics-svg-threshold-kb 200`   |       0.25 s   |     0.3 s  |
| **speedup**                         |     **130×**   |   **111×** |

arxiv:1807.01606 has 15 such PDFs; serial convert would be ~8 minutes
(likely times out). Inkscape path: 15 × 0.25 s ≈ 4 s.

Regression coverage: `test_vector_svg_pathological_convert_case` in
`latexml_post/tests/integration.rs` asserts the inkscape path completes
in <5 s on this fixture (silently skipped when inkscape is absent from
PATH).

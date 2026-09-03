# Architectural & Performance Improvement Ideas for latexml-oxide

> **Mission Context.** latexml-oxide is a mature, high-fidelity Rust port of
> [LaTeXML](https://github.com/brucemiller/latexml), driving both faithful Perl
> parity and large-scale (2.8M-doc) arXiv and TeX Live documentation conversions.
> This document records concrete, actionable improvement proposals across
> architecture, maintainability, runtime performance, and memory efficiency,
> aiming for best-in-class performance while preserving strict TeX/Perl fidelity.

---

## 1. Architectural Modularity & Maintainability

### 1.1 De-monolithizing `latex_constructs.rs` (Landed in Batch 54s)

* **Status:** **Implemented** in commit `6762e29966` (Batch 54s). `latex_constructs.rs` was cleanly split into section modules under `latex_constructs/` preserving bitwise dump identity.
* **Proposed Solution:**
  - Group `latex_constructs` definitions by domain into internal sub-modules within a `latex_constructs/` directory (or child modules in `latexml_engine`):
    - `layout_and_pages.rs`: Document classes, margins, page geometry, pagestyles.
    - `sectioning_and_titles.rs`: Section levels, titles, abstract, frontmatter.
    - `environments.rs`: Lists, theorems, verbatim, quote, itemize/enumerate.
    - `fonts_and_accents.rs`: Font switches, text accents, symbol alphabets.
    - `tables_and_boxes.rs`: Tabular/array helpers, framed boxes, minipages.
    - `counters_and_hooks.rs`: Counter manipulation, LaTeX2e hook management.
  - Each module exposes a `pub(crate) fn register_*(...) -> Result<()>` called in sequence by `load_definitions()`.
  - **Parity Guard:** The registration order of control sequences directly influences dump table layout; the orchestrator calls each sub-module's register function in the exact historical order, ensuring 100% bitwise dump compatibility.

### 1.2 Macro Expansion Overhead in `latexml_codegen`

* **Current Reality:** Definitions heavily rely on `DefMacro!`, `DefConstructor!`, and `DefPrimitive!` proc macros in `latexml_codegen`. These macros expand at compile time into large AST trees with boxed closures, string literals, and argument unpackers.
* **Proposed Solution:**
  - Standardize common pattern expansions into const declarative descriptor structs (`ConstructorSpec`, `MacroSpec`) interpreted by a uniform registration engine, rather than generating bespoke code blocks per definition.
  - For simple string-to-string macro aliases, replace closure generation with direct token-table insertions, cutting binary code bloat and compile times.

### 1.3 Strict `pub(crate)` Visibility Discipline for Dead-Code Elimination

* **Current Reality:** Several historical audits (such as the 2026-07-30 font-selection chain audit in `SYNC_STATUS.md`) revealed fully translated helper functions and tables (e.g. `ding_fontmap.rs`, `lookup_tex_font`) that had zero callers and were completely inert, but went undetected because they were marked `pub` in library crates.
* **Proposed Solution:**
  - Enforce a strict crate-internal visibility policy: no function or type should be `pub` unless it is explicitly part of the public crate interface consumed across crate boundaries.
  - Enable `#![deny(unreachable_pub)]` in crate roots or workspace lint configuration. When an internal helper loses its last caller, `dead_code` immediately raises a hard compiler error in CI.

---

## 2. Memory Architecture & Allocation Optimization

### 2.1 Small-Token List Optimization (`SmallVec` in `Tokens`) [Settled Dead-End]

* **Status:** **Empirically Refuted (Settled Dead-End).** Measured across the arXiv corpus: `SmallVec<Token>` regressed performance across every size threshold $N$ due to copy/move overhead and stack spill. Do not re-attempt without changing the Token representation itself.
* **Proposed Solution:**
  - Back `Tokens` with an inline small-vector representation, such as `smallvec::SmallVec<[Token; 4]>`, or a custom enum:
    ```rust
    #[derive(Clone)]
    pub enum TokensInner {
      Inline(u8, [Token; 3]),
      Heap(Vec<Token>),
    }
    ```
  - For 64-bit systems with 8-byte `Token`:
    - `[Token; 3]` is 24 bytes + 1 byte tag + 7 bytes padding = 32 bytes (exactly the size of a standard 3-word `Vec<Token>`).
    - Up to 3 tokens (covering >70% of macro arguments, delimiters, punctuation, and math atoms) require **zero heap allocations**.
  - For immutable macro bodies stored in definitions, use `Rc<[Token]>` to share token bodies across expansions without cloning the underlying buffer.

### 2.2 Probe vs. Pin in `SymHashMap` and String Interning

* **Current Reality:**
  In `latexml_core/src/common/arena/data.rs`:
  ```rust
  impl<T> SymHashMap<T> {
    #[inline]
    pub fn get(&self, key: &str) -> Option<&T> { self.0.get(&arena::pin(key)) }
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool { self.0.contains_key(&arena::pin(key)) }
  }
  ```
  Calling `get(&str)` or `contains_key(&str)` unconditionally calls `arena::pin(key)`. This inserts the key into the thread-local string interner arena **even when the key does not exist in the hash map**!
* **The Cost:**
  Negative lookups (such as probing for undeclared package options, optional keyvals, or unbound control sequences) permanently pollute the global string interner arena with dead strings, expanding memory footprint and causing unnecessary arena resizes.
* **Proposed Solution:**
  - Leverage `arena::get(key) -> Option<SymStr>` (which probes without interning):
    ```rust
    #[inline]
    pub fn get(&self, key: &str) -> Option<&T> {
      let sym = arena::get(key)?;
      self.0.get(&sym)
    }

    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
      arena::get(key).is_some_and(|sym| self.0.contains_key(&sym))
    }
    ```
  - If the string was never interned anywhere in the system, it cannot possibly be a key in any `SymHashMap`. This eliminates interner pollution on all negative lookups.

### 2.3 `libxml2` DOM Memory Footprint & Native Rust DOM Path

* **Current Reality:**
  - Empirical measurement ([`performance/STREAMING_CORE_DESIGN_2026-07-29.md`](performance/STREAMING_CORE_DESIGN_2026-07-29.md)): `libxml2` DOM nodes account for **~57% of total conversion RSS** (~1.84 GB per MB of TeX source in large multi-megabyte documents).
  - Each `xmlNode` in C is an individually allocated 144-byte structure with 18 pointer fields (`children`, `parent`, `next`, `prev`, `properties`, `ns`, etc.), plus separate allocations for attributes and strings.
* **Proposed Strategic Direction:**
  1. **Short-Term (DOM Scratch Pooling):** During temporary measurement runs (e.g. `\widthof`, `\settowidth`, or trial tabular passes), avoid building full `libxml2` nodes if only box metrics or token reverts are inspected.
  2. **Long-Term (Typed Native Rust DOM):** Transition the construction stage to an arena-allocated Rust tree (or index-based slot map such as `indextree` or a purpose-built Typed AST):
     - Reduces per-node overhead from 144+ bytes to ~32–48 bytes.
     - Guarantees cache locality during document traversal and DOM rewriting.
     - Makes DOM manipulation 100% memory-safe without FFI boundaries.
     - Enables true thread-safe multi-document joins and streaming output serialization.

### 2.4 Struct Sizing & Cache-Line Alignment

* **Audit Candidate:** `latexml_core::digested::Digested` and `Whatsit`.
* Inspect layout using `-Z print-type-sizes` on nightly. Heavy variant fields (like reversion token buffers, font specifications, or keyvals) should be boxed or `Rc`-shared so that standard box and glyph nodes remain small and cache-friendly during digestion walks.

---

## 3. TeX Engine Emulation & Gullet Performance

### 3.1 Early Heuristic Cycle Detection in Gullet Expansion

* **Current Reality:**
  - The default runaway token limit was recently raised from 400M to 1G (`DEFAULT_TOKEN_LIMIT`) to accommodate massive pgf/tikz drawings (e.g. `tikzlings-doc` completing at 444M tokens).
  - Genuine infinite expansion loops (e.g., self-referential macro definitions or runaway delimited scans) must execute up to 1,000,000,000 tokens before failing with `Fatal:Timeout:TokenLimit`, wasting 10–60 seconds of CPU time per document in test sweeps.
* **Proposed Solution:**
  - Implement a sliding-window cycle detector on the pushback queue:
    - Keep a small circular buffer of recent token hashes (e.g. 16 to 32 tokens).
    - If the exact same 32-token sequence is repeatedly encountered at the head of the pushback queue with zero progress in document digestion, trigger a fast-fail `Fatal:Stomach:Recursion` after $K$ cycles (e.g. 10,000 iterations) rather than waiting for 1G tokens.
    - Preserves pgf/tikz drawings (which produce high volumes of *varying* coordinates and path commands) while killing true static cycles in milliseconds.

### 3.2 Tail-Call Optimization for Deep `\expandafter` Chains

* **Current Reality:** Packages like `biblatex`, `csquotes`, and `expl3` routinely generate chains of 7 to 15 consecutive `\expandafter` primitives (`\expandafter\a\expandafter\b\expandafter\c...`).
* **Proposed Solution:**
  - In `tex_macro.rs`, implement an iterative expansion loop that collects the sequence of targets and performs batch lookahead expansions without recursive function calls, reducing call-stack depth and avoiding repeated temporary token pushes.

### 3.3 Unified Virtual File Store (VFS) for TeX I/O

* **Current Reality:**
  - As noted in [`perfect_kernel/PLANS.md`](perfect_kernel/PLANS.md) §96 (Architectural Queue #1), file write/read loops (`\openout`, `\write`, `\input`, `\read`) are handled by ad-hoc writers across `filecontents`, `fancyvrb VerbatimOut`, `fancybox VerbatimOut`, and `memoir writeverbatim`.
* **Proposed Solution:**
  - Consolidate all in-memory file write/read operations into a formal `VirtualFileSystem` in `latexml_core`.
  - All `\write`-to-stream operations write into the VFS, and all file loaders consult the VFS before falling back to disk or kpathsea.
  - Standardizes temporary file handling, eliminates duplicated verbatim scanners, and ensures packages like `answers.sty`, `tutodoc`, and `exercisebank` work automatically.

---

## 4. Math Parser & Ambiguity Pipeline (Marpa ASF)

### 4.1 Recognition-Time Pruning vs. Post-Parse Tree Pruning

* **Current Reality:**
  - The math parser uses a highly ambiguous Marpa CFG to capture all syntactic interpretations of arXiv math.
  - Highly ambiguous expressions (such as nested subscripts, multiple prime marks, or complex operator sequences) can produce thousands of parse trees, which are then pruned during post-parse tree walks in `latexml_math_parser/src/semantics.rs`.
* **Proposed Solution:**
  - Where syntax is fundamentally invalid in standard LaTeX math (e.g. duplicate consecutive relational operators without conjunctions, or invalid fence combinations), introduce recognition-time grammar constraints or lexical grouping tokens before Marpa input.
  - Pruning 500 parses to 5 at the Recognizer stage yields a 10× to 50× reduction in parse-time CPU and memory allocation on math-dense documents.

### 4.2 Sub-expression Memoization in Repetitive Tables

* **Current Reality:** Large matrices and tables (`tabular`, `array`, `nicematrix`) repeatedly parse identical or symmetric math expressions across cells (e.g. `0`, `1`, `x_{i,j}`, `\dots`).
* **Proposed Solution:**
  - Maintain a thread-local LRU cache of recently parsed formula ASTs keyed by normalized `XMath` token hashes.
  - If a cell's math expression matches an active cache entry, clone the resulting XM tree directly, bypassing Marpa recognizer init, bocage traversal, and semantic actions entirely.

---

## 5. Diagnostic Architecture & Observability

### 5.1 Structured Diagnostic Events vs. Stderr Scraping

* **Current Reality:**
  - Tooling, sweeps, and triage scripts (`sweep.sh`, `benchmark_canvas.sh`, `tally.sh`) count errors by piping stderr through `sed 's/\x1b\[[0-9;]*m//g'` and grepping for `^Error:[a-z]`.
  - Stderr parsing is vulnerable to color escape corruption, multi-line interleaving, and accidental masking (as historically occurred when `Error:` lines were suppressed or stripped).
* **Proposed Solution:**
  - Introduce a structured `DiagnosticCollector` in `latexml_core`:
    ```rust
    pub struct DiagnosticRecord {
      pub severity: Severity, // Warning, Error, Fatal
      pub category: SymStr,   // e.g. "undefined", "malformed", "syntax"
      pub locus: Option<SourceSpan>,
      pub message: String,
    }
    ```
  - Telemetry and conversion results serialize these records directly into the conversion summary / JSON output (`--telemetry-out`).
  - Eliminates fragile text regexes, provides exact error categorization at zero parsing cost, and enables unambiguous assertion testing in integration test suites.

---

## 6. Test Suite & Developer Workflow

### 6.1 Dynamic Test Discovery (Eliminating `cargo clean`)

* **Current Reality:**
  - `latexml_oxide` uses a compile-time procedural macro / build script to discover test cases (`.tex`/`.xml` pairs).
  - Adding a new test pair requires running `cargo clean` or touching `build.rs` to force cargo to rebuild the test list.
* **Proposed Solution:**
  - In `build.rs`, emit `cargo:rerun-if-changed=tests/fixtures` (watching the directory itself) so adding or renaming a file automatically invalidates the test discovery cache without requiring `cargo clean`.
  - Alternatively, use dynamic filesystem discovery in test runners (e.g. `libtest-mimic` or directory walk at test execution time).

### 6.2 Automated Markdown & Link Validation in CI

* **Current Reality:**
  - Documentation files occasionally develop broken internal links when files are moved to `docs/archive/` or headings are renamed.
* **Proposed Solution:**
  - Add a lightweight markdown link validation check to `.github/workflows/CI.yml` (as part of the `lint` job).
  - Scans all `[label](path)` links in `docs/` and root `.md` files to ensure targets exist, preventing link rot permanently.

---

## 7. Prioritized Implementation Roadmap

| Priority | Initiative | Complexity | Primary Benefit |
|:---:|---|:---:|---|
| **P1** | **`SymHashMap` probe-before-pin fix** (`arena::get` on negative lookups) | Low | Stops permanent arena memory leakage on missing keys |
| **P2** | **Automated doc-link linter in CI** | Low | Prevents documentation link rot permanently |
| **P3** | **`pub(crate)` visibility audit** on engine internals | Low-Med | Restores compiler `dead_code` enforcement across crates |
| **P4** | **`SmallVec` inline optimization for `Tokens`** | Medium | Eliminates 60–80% of small heap allocations in digestion |
| **P5** | **Heuristic cycle detector for gullet expansion** | Medium | Fast-fails infinite macro loops in milliseconds instead of waiting for 1G tokens |
| **P6** | **Sub-modularize `latex_constructs.rs`** | Medium | Cuts compile times and RAM; fixes rust-analyzer IDE hangs |
| **P7** | **Unified Virtual File Store (VFS)** in `latexml_core` | Med-High | Standardizes file I/O; enables multi-package write/read support |
| **P8** | **Structured Diagnostic Collector** | Med-High | Eliminates stderr regex scraping; enables robust telemetry |
| **P9** | **Native Typed Rust DOM (Post-1.0 Architecture)** | High | Eliminates libxml2 C DOM (saving ~57% RSS); enables zero-copy transforms |

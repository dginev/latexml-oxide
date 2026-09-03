# latexml-oxide — Gemini Agent Guidelines & Operational Memory

> **Ground Truth Rule:** This is a faithful Perl-to-Rust translation of [LaTeXML](https://github.com/brucemiller/latexml). The Perl source (`LaTeXML/`) is ground truth. When surpassing Perl on malformed edge cases, `pdflatex` (TeX Live) is the surpass oracle. Never invent speculative semantics or rename concepts unless documented in `docs/parity/OXIDIZED_DESIGN.md`.
>
> **Document Primacy:** [`CLAUDE.md`](CLAUDE.md) is the original and primary instruction document for this repository. All contributors and agents honor its directives, build/profile topologies, and parity contracts.

---

## 1. Quick Navigation & Daily Workflow

* **Active Engineering Worklist:** [`docs/SYNC_STATUS.md`](docs/SYNC_STATUS.md) — ordered by unblocked priority (R1…R7). Always start here.
* **Full Documentation Index:** [`docs/README.md`](docs/README.md) — multi-level table of contents for all architectural and design docs.
* **Release Contract & Portability:** [`docs/release/RELEASE_CRITERIA.md`](docs/release/RELEASE_CRITERIA.md) and [`docs/release/WASM_COMPATIBILITY_PLAN.md`](docs/release/WASM_COMPATIBILITY_PLAN.md).

---

## 2. Core Non-Negotiable Directives

1. **Never Downgrade Diagnostics:** Never downgrade `Fatal:` or `Error:` to `Warning:` or silence diagnostics to make a paper pass. Parity requires fixing the underlying compiler bug. Always classify with `latexml --verbose`, never `--quiet`.
2. **Never Run Destructive Git Commands:**
   * **NEVER run `git clean -fd`** (destroys untracked scratch files, repros, local data).
   * **NEVER run `git checkout <file>` or `git restore` mid-task** (discards uncommitted work in that file; work forward, commit, and revert if needed).
   * **Stash is SHARED across worktrees:** When parallel agents run, `git stash` interleaves. Use diff patches (`git diff > /tmp/p.patch`) instead.
3. **Respect Hardware & Thermal Budgets ([`docs/THERMALS.md`](docs/THERMALS.md)):**
   * Development host: i7-12800H (20 threads), 31 GB RAM + 8 GB swap.
   * **NEVER overlap heavy workloads:** Never run corpus sweeps (`sweep.sh`, `oracle.sh`, `validate.sh`) concurrently with `cargo nextest`.
   * Concurrency caps: `JOBS=8` alone, `JOBS=4` beside another process.
   * Bulk outputs must go to `~/data/`, **NEVER** `/tmp` (tmpfs RAM exhaustion risk).
4. **Shell & Script Safety:**
   * No unescaped `!` in bash commands (history expansion risk).
   * No busy-wait `until` / `while` polling loops.

---

## 3. Specialized Workspace Rules (`.agents/rules/`)

Detailed, distilled guidelines for specific domains live in `.agents/rules/`:

| Rule File | Domain & Guidance |
|---|---|
| [`.agents/rules/latex_soundness.md`](.agents/rules/latex_soundness.md) | **TeX/LaTeX Semantics**: `tex.web` kernel invariants, nest vs save stack, alignment tab parameter scan vs cell read, binding priority over raw, and semantic completeness over stubs. |
| [`.agents/rules/thermals_and_resources.md`](.agents/rules/thermals_and_resources.md) | **Thermals & Concurrency**: Hardware boundaries, thread caps, sweep discipline, and swap leak mitigation. |
| [`.agents/rules/git_safety.md`](.agents/rules/git_safety.md) | **Git & Subagent Safety**: Banned destructive commands, stash collisions in worktrees, and squash PR conventions. |
| [`.agents/rules/triage_recipes.md`](.agents/rules/triage_recipes.md) | **Diagnostic Decision Tree**: First-error extraction, symptom-to-cause mappings (catcodes, undefined CS, mode leaks, cropping, alignments). |
| [`.agents/rules/rust_idioms.md`](.agents/rules/rust_idioms.md) | **Rust Idioms & Settled Dead-Ends**: `FxHashMap`, arena resets, `thread_local!(RefCell)`, `T_CS!`, and empirically refuted dead-ends (`SmallVec<Token>`). |

---

## 4. Canonical Datasets & Triage Scripts

| Resource | Path / Command | Purpose |
|---|---|---|
| **Parity Classifier** | `tools/parity_check.sh <arxiv_id>` | 180s timeout parity check against same-host Perl. |
| **Failure Triage** | `tools/triage_failure.sh <arxiv_id>` | Full backtrace triage under test profile. |
| **Bisection Tool** | `tools/bisect_repro.sh <arxiv_id>` | Coarse window bisection from first-error line. |
| **Corpus Data** | `~/data/recent_warning_papers/` | Primary triage dataset on disk. |
| **Source Archive** | `~/data/arxmliv/<bucket>/<id>/<id>.zip` | Full arXiv source tarballs. |

---

## 5. Multi-Agent & Cross-System Context

* This codebase is co-developed across multiple assistants (Codex on the performance track, Claude, and Gemini).
* Commit history and `PERFORMANCE.md` reflect contributions from both agents and human maintainers; do not refactor working conventions solely for aesthetic uniformity.
* Global knowledge intended to persist across all assistants and machines belongs committed in `docs/` and `GEMINI.md`.

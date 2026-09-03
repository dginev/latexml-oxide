# Claude-to-Codex knowledge migration (2026-09-03)

**Status:** durable repository layer implemented. Root `AGENTS.md` and ten
project-owned Codex skills under `.agents/skills/latexml-*` now contain the
curated rules and workflows. Local Codex memory remains unimported and disabled;
`~/.codex/config.toml` and generated `~/.codex/memories/` were not changed. On
Codex CLI 0.153.0, `codex features list` reports both `memories` and
`external_agent_memory_import` disabled.

The goal is not to copy every Claude memory into every Codex prompt. It is to
put each kind of durable knowledge on the Codex surface that matches its scope:

- required repository rules in `AGENTS.md`;
- reusable workflows in repo-scoped `.agents/skills/`;
- detailed subsystem knowledge and current state in checked-in `docs/`;
- helpful preferences and cross-session context in generated local Codex
  memories.

This follows the official Codex guidance for
[`AGENTS.md`](https://learn.chatgpt.com/docs/agent-configuration/agents-md.md),
[`skills`](https://learn.chatgpt.com/docs/build-skills.md),
[`memories`](https://learn.chatgpt.com/docs/customization/memories.md), and
[`/import`](https://learn.chatgpt.com/docs/import.md). Required team guidance
must not exist only in memory, and generated `~/.codex/memories/` files should
not be hand-maintained as the primary control surface.

## Source inventory

Claude project memory lives at:

```text
~/.claude/projects/-home-deyan-git-latexml-oxide/memory/
```

Inventory on 2026-09-03:

| Source | Count | Character |
|---|---:|---|
| `feedback_*` | 84 | user preferences and workflow rules |
| `wisdom_*` | 74 | subsystem mechanisms, traps, and settled experiments |
| `project_*` | 27 | decisions, active work, and sprint-scoped state |
| `reference_*` | 19 | external systems, commands, and infrastructure pointers |
| indexes/support | 5 | `MEMORY.md`, maintenance contract, indexes, debugging references |
| **Total** | **209** | live files after the 2026-09-02 consolidation |

The memory set is already curated. `README_memory.md` separates durable rules
from state, and `MEMORY.md` plus `wisdom_index.md` provide a bounded entry point.
Six subsystem hubs consolidate libxml, catcode/tokenization, constructors,
vendor packages, kernel dumps, and mode/digestion knowledge.

The Claude setup contains ten skills and two read-only agent definitions under
`.claude/`. The repository now has a root `AGENTS.md` and ten corresponding
project skills under `.agents/skills/`. The observed `~/.codex/config.toml` does
not enable the memories feature, and no generated `~/.codex/memories/` directory
was created during this work.

`.agents/` is shared with Gemini. Gemini's agent-local rules live under the
ignored `.agents/rules/`; this migration neither edits nor tracks them. The
`.gitignore` policy allowlists only the project-owned `latexml-*` skill
directories, leaving local rules and third-party skills installed under
`.agents/` ignored. Durable rules common to both agents were independently
reconciled into `AGENTS.md`; agent-specific recipes remain outside it.

Do not import `memory.bak.20260902-prune/`; it is a pre-consolidation backup and
would reintroduce duplicates deliberately removed by the memory-maintenance
contract.

## Target model

### 1. Checked-in documentation remains canonical

Current project state, measured performance, parity status, and architecture
belong in `docs/`, not agent memory. In particular:

- start at [`README.md`](README.md) and [`SYNC_STATUS.md`](SYNC_STATUS.md);
- use `performance/PERFORMANCE.md` and dated performance studies for measured
  claims;
- use `parity/KNOWN_PERL_ERRORS.md`, `parity/WISDOM.md`, and the
  `OXIDIZED_DESIGN` family for parity and intentional divergences;
- use source, tests, and current tool output to resolve stale memory.

Memory should point to these documents rather than quote changing percentages,
test counts, open rows, or recent commit lists.

### 2. `AGENTS.md` holds the non-negotiable repository contract

Codex loads repository `AGENTS.md` before work and composes nested instruction
files from root to current directory. The default combined project-instruction
budget is 32 KiB, so the root file should be a concise contract, not a dump of
209 memories.

The root `AGENTS.md` distills these durable rules:

1. Read `docs/README.md` and the task's subsystem docs before changing code;
   read `docs/THERMALS.md` before heavy work.
2. Perl LaTeXML is the translation oracle; TeX/LaTeX sources and same-host live
   probes establish mechanism. Intentional beyond-Perl changes require an
   explicit documented decision.
3. Never reduce severity, suppress diagnostics, stub semantics, or raise/lower
   a guard merely to make a failure disappear. Name and fix the mechanism.
4. Performance changes must be output-neutral unless separately authorized:
   release-mode same-session A/B, production profile, exact output comparison,
   status/error parity, phase time, CPU, and max RSS.
5. Use the correct comparison path: verbose same-host Perl, installed-versus-
   vendored version check, full output structure, and post-stage diagnostics.
6. Preserve unrelated working-tree changes. Avoid destructive git operations;
   one deliverable per branch/PR when publication is requested.
7. Use the documented test topology (`cargo nextest run --workspace` for the
   suite); do not combine heavy suites and sweeps on this host.
8. Preserve architectural invariants: `SymStr`, engine state, and libxml nodes
   are thread-affine; dynamic `DefRewrite` XPath keeps libxml2 in the core path;
   Rust hot-path maps use the project hashing/interner conventions.
9. Keep state in checked-in docs, reusable procedures in skills, and only
   preference/context recall in memories.

Machine-specific deployment endpoints, current sprint percentages, agent model
names, and one-off shell restrictions do not belong in the root contract.

### 3. `.agents/skills/` holds progressive workflows

Codex discovers repo skills under `.agents/skills/` and initially loads only
their names/descriptions. Detailed instructions and references load when a
skill activates, making skills the right home for the larger Claude procedures.

Implemented port mapping:

| Priority | Claude source | Codex skill | Porting note |
|---:|---|---|---|
| 1 | `perf-check` | `latexml-perf-check` | preserve settled non-levers, production profile, thermal and output gates |
| 2 | `perl-port` | `latexml-perl-port` | preserve source-first mechanism derivation and parity boundaries |
| 3 | `canvas-triage` | `latexml-canvas-triage` | preserve verbose same-host oracle and fail-toward-flagging rules |
| 4 | `min-repro` | `latexml-min-repro` | keep reductions semantic and verify both engines at every stage |
| 5 | `resolve-issue` | `latexml-resolve-issue` | compose the first four; remove Claude-specific Task syntax |
| 6 | `dump-debug` | `latexml-dump-debug` | attach kernel-dump hub material as focused references |
| 7 | `cluster-classify` | `latexml-cluster-classify` | remove Claude Workflow assumptions; use the documented thermal/job budget |
| 8 | `start-session` | `latexml-start-session` | keep it short; current state remains in `SYNC_STATUS.md` |
| 9 | `surpass-perl` | `latexml-surpass-perl` | retain authorization and output-quality gates |
| 10 | `next-release` | `latexml-next-release` | retain branch, publication, and dependency checks |

The implemented skills do not copy Claude tool calls literally. `Workflow`,
`TaskOutput`, model names, memory-file dependencies, and Claude permission syntax
were removed. Descriptions provide narrow activation gates, while the bodies
retain project-specific inputs, outputs, failure boundaries, and canonical-doc
pointers.

The port also corrected stale operational detail instead of preserving it:
`latex_constructs.rs` is now an ordered `latex_constructs/` module split, so the
dump skill requires checking `mod.rs` and the current last-loaded section before
placing a final-phase definition.

The two Claude agents were not copied as Codex subagent definitions. Their
durable read-only log aggregation and root-cause responsibilities are already
represented by the classification, triage, and porting skills; Claude model
pins and task-tool syntax were intentionally excluded. Add separate Codex
subagent definitions only if a future workflow needs those roles independently.

### 4. Local Codex memories hold recall, not policy

Local Codex memories are separate from ChatGPT web memory. When enabled, Codex
generates and consolidates them under `~/.codex/memories/`; `/memories` controls
whether a chat may use existing memories or contribute to future ones.

Suitable memory candidates from this project are slow-changing preferences:

- the user prefers principled mechanism fixes over symptom patches;
- performance work is one lever per measured run, performance before size;
- parity is the origin of a defect, not an excuse to leave it unfixed;
- same-host oracle comparisons and exact output checks matter;
- heavy workloads must respect the laptop's thermal/RSS budget;
- contradictory historical claims should be resolved empirically.

Do not rely on memory alone for these rules if violating them could damage the
repo or invalidate results; the enforceable form belongs in `AGENTS.md` or a
skill.

## What not to migrate as memory

- `project_current_state`, sprint percentages, test counts, pending PR lists,
  and dated availability notes: point to `SYNC_STATUS.md` or current tools.
- Fixed issue narratives already captured by tests, commits, or closed docs:
  retain only the reusable mechanism or settled dead end.
- Absolute file line numbers without a durable symbol name: source lines move.
- Backups, duplicate pre-prune files, raw chat transcripts, and tool-result
  dumps.
- Credentials, secrets, private tokens, or authentication headers. Review
  infrastructure references before import even when the source claims to be
  secret-free.
- Claude-specific model/delegation directives and shell permission syntax.
- Claims about current external services, deployments, package versions, or
  fleet health without live verification.

## Supported import sequence

The official Codex import flow can import Claude Code instructions, settings,
skills, project memories, recent chats, hooks, and agents. It leaves the Claude
setup unchanged.

1. Finish or stop the current task. `/import` is unavailable during a running
   task, a remote session, or a local app-server daemon connection.
2. Start a fresh interactive Codex CLI session in this repository with both
   features enabled for that session, without changing global configuration:

   ```bash
   codex --enable memories --enable external_agent_memory_import
   ```

   Use `codex features enable memories` and
   `codex features enable external_agent_memory_import` instead only when the
   user wants persistent enablement in `~/.codex/config.toml`.
3. Run `/import`, then select **Claude Code**, this project, and project memories.
   Do not select the
   `memory.bak.20260902-prune` backup. Prefer the project-memory import only: the
   instructions and skills already have curated Codex-native replacements, so
   importing their Claude forms would create duplicate and possibly conflicting
   guidance. Import settings, hooks, chats, or agents only after reviewing their
   permissions, privacy, recency, and tool assumptions.
4. Review the imported output before enabling automatic synchronization. An
   import does not delete the existing Claude setup.
5. Enable local memories deliberately with `/memories` or, in a later
   configuration pass, `[features] memories = true`. Keep generation/use
   controls explicit.
6. Keep the checked-in `AGENTS.md` and focused repo skills authoritative rather
   than treating the imported memory set as mandatory instructions.
7. Test instruction discovery and skill activation in a new Codex session.

The import is the supported way to seed Codex-generated memory. Manual copying
into `~/.codex/memories/` is intentionally not part of this plan.

## Acceptance checks

The checked-in layer can be tested now; memory-specific checks follow the later
interactive import. Use representative prompts:

| Prompt | Expected behavior |
|---|---|
| “What should I read before starting?” | names `docs/README.md`, `SYNC_STATUS.md`, and the subsystem docs |
| “Optimize this conversion” | selects the performance skill and requires production-profile A/B plus output parity |
| “Rust has fewer errors than Perl; is it correct?” | compares semantics/output and treats parity as origin, not absolution |
| “Run the sweep and full suite together” | refuses the overlap and cites `THERMALS.md` |
| “Triage these 20 failing papers” | selects classification workflow, preserves parse misses, and uses bounded concurrency |
| “Port this primitive” | reads Perl and TeX sources first and names the mechanism |

Also verify:

- `codex ... "Summarize the current instructions"` reports the intended root
  `AGENTS.md` and any nested override in precedence order;
- skill descriptions activate on positive examples and stay dormant on
  unrelated requests;
- imported memories contain no secrets and do not duplicate current state;
- a fresh session can find detailed wisdom through a skill/reference without
  loading the full 209-file corpus into its initial prompt;
- stale memory loses to current source, tests, and checked-in documentation.

## Handoff and remaining step

The checked-in migration is complete:

- `AGENTS.md` is the concise mandatory contract;
- `.agents/skills/latexml-*` contains all ten curated workflows;
- `.gitignore` tracks only those project-owned skill directories;
- Gemini's `.agents/rules/` files remain local and untouched;
- each skill passes the official `skill-creator` `quick_validate.py` check.

A fresh read-only ephemeral `codex exec` smoke test on Codex CLI 0.153.0 also
loaded `AGENTS.md`, selected `.agents/skills/latexml-perf-check/SKILL.md` for a
benchmark prompt, identified `docs/SYNC_STATUS.md` and `docs/README.md` as the
state/index pair, and returned the same-host oracle and no-overlapping-heavy-work
rules. No build, conversion, benchmark, or test suite was run for this
configuration-only validation.

The remaining operation is user-controlled because `/import` is unavailable
inside a running task. In a fresh interactive Codex CLI session launched with
the two feature flags above, import only the live Claude project-memory set,
exclude the pre-prune backup and duplicated skills/instructions, review the
generated memory for secrets and stale state, then confirm the desired behavior
with `/memories`. Afterwards, run the representative activation prompts above
and verify that current docs and source win over any recalled snapshot.

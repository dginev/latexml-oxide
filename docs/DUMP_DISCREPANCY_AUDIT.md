# Rust Dump Discrepancy Audit Patterns

> Created 2026-04-26 in response to user request: "Based on this finding
> and the previous one, is there a pattern of discrepancies we can audit
> for in the rust dumps? the perl originals are the correct baseline to
> follow."

The Perl LaTeXML engine's dump (`make formats`) is the authoritative
baseline. As the Rust dump infrastructure improves and captures more
of the LaTeX kernel state correctly, regressions surface elsewhere
because workaround code was authored under the assumption that the
dump was incomplete. This document catalogs the recurring discrepancy
patterns and provides audit recipes.

---

## Pattern 1: Lt-alias flattening (writer-side)

**Perl semantics**: `Lt('\\foo','\\bar')` is `\let \foo = \bar`. In
Perl, `\foo` and `\bar` SHARE THE SAME Definition object (an in-memory
hash reference). Anywhere code reads `$$defn{cs}[0]` for the canonical
name, it gets `\bar` regardless of which name was used to look up.

**Rust dump_writer behavior**: emits Lt-aliases as separate Expandable
entries:
```
M  \foo  E  \bar  <nargs>  <flags>  <body>  <params>  <extras>
```
where col 4 (`\bar`) is the alias and col 2 (`\foo`) is the local cs.
On dump-load, Rust creates a SEPARATE `Expandable` with `cs=\foo` and
(post-fix) `alias=\bar` — but the `cs` field is `\foo`, not `\bar`.

**Symptoms**:
- `defn.get_cs().text` returns `\foo`, not `\bar`.
- Any HashSet/Vec gating on canonical-name lookups misses the alias.
- Concrete instance: `DEFERRED_COMMANDS` in `gullet.rs` missed
  `\exp_not:n` (alias for `\unexpanded`) — caused the `\q_no_value`
  recursion cascade (commit `e3d4f8532`).

**Audit recipe** (writer):
```bash
# All Perl Lt-aliases:
grep -E "Lt\\('\\\\\\\\[^']+','\\\\\\\\[^']+'\\)" \
  LaTeXML/blib/lib/LaTeXML/Engine/latex_dump.pool.ltxml | wc -l
# 2026-04-26 count: 1066 entries in latex.dump.txt

# Rust dump entries where alias ≠ cs (the same set):
awk -F'\t' '$1=="M" && $3=="E" && $2!=$4 {print $2, "→", $4}' \
  resources/dumps/latex.dump.txt | wc -l
```

**Audit recipe** (runtime):
```bash
# Rust callsites that look up by defn.get_cs().text:
grep -rnE "defn\.get_cs\(\)\.text|defn\.get_cs\(\)\.with_str" \
  latexml_core/src latexml_package/src
# Each callsite should also check defn.get_alias() if the lookup
# needs canonical-name semantics (matching Perl's $$defn{cs}[0]).
```

**Fix shape (per callsite)**:
- Either: also check `defn.get_alias()` against the same set.
- Or: limit the alias propagation in `dump_reader.rs` to ONLY the
  CSes that need canonical-name semantics (the narrow approach
  taken in commit `e3d4f8532`).

**Status (2026-04-26)**:
- Fixed: gullet's `DEFERRED_COMMANDS` gate (commit `e3d4f8532`).
- Pending audit: other callsites — TBD by grepping for
  `defn.get_cs().text` use against HashSet/Vec membership.

---

## Pattern 2: Stale fix-up `raw_tex` blocks (binding-side)

**Origin**: package bindings in `latexml_package/src/package/*.rs`
were authored when raw expl3.sty / expl3-code.tex load failed to
install many CSes (forward references, expansion-chain breaks, etc.).
The bindings compensate by calling `raw_tex(...)` with `\quark_new:N`,
`\msg_new:nnn`, `\seq_gclear_new:N`, etc. to create those CSes
post-load.

**Discrepancy**: as the dump captures more CSes correctly (e.g.
`\q__file_nil`, `\msg text > cmd/define-command`, `\g__file_record_seq`),
the compensating `raw_tex` calls now hit `\__kernel_chk_if_free_cs:N`
→ `\msg_error:nnee{kernel}{command-already-defined}` → message-system
chain that mismatches our boxing/non-boxing group accounting.

**Symptoms**:
- 49 deterministic boxing-group errors per `\usepackage{expl3}` load
  (sandbox 2026-04-26 — 12+ papers exhibited this exactly).
- `\__msg_interrupt:NnnnN` chain firing on otherwise-successful
  package loads.
- Long cascades into "undefined: \author" / "undefined: \sqrt"
  (4974+ per paper) when the boxing errors corrupt later state.

**Audit recipe**:
```bash
# Wrappers that fire \__kernel_chk_if_free_cs:N in expl3 source
# (each triggers already-defined error if target CS is in dump):
CHK_FREE_WRAPPERS='quark_new|msg_new|seq_new|seq_gclear_new|
  scan_new|tl_new|prop_new|str_new|bool_new|fp_new|skip_new|
  dim_new|muskip_new|toks_new|int_new|intarray_new|fparray_new|
  flag_new|coffin_new|color_new|regex_new|cs_new:Npn|
  cs_new_protected:Npn|cs_new_eq:NN|__kernel_quark_new_conditional|
  __kernel_quark_new_test'

# Find raw_tex / RawTeX! / r"..." calls invoking these wrappers:
grep -rnE "raw_tex.*($CHK_FREE_WRAPPERS)" \
  latexml_package/src/package/ latexml_package/src/engine/

# Each found line should be guarded with \cs_if_exist:NF{...}
# OR replaced with the _set / _gset variant when redefinition
# is intended (the _new variants ALL fire chk_free when the CS
# is already defined).
```

**Fix shape (per callsite)**:
- If the binding intends "create-if-missing": wrap with
  `\cs_if_exist:NF{...}` or `\cs_if_exist:NTF{ ... \X_set:N|... }{...\X_new:N|... }`.
- If the binding intends "always-overwrite": replace `\X_new:N` with
  `\cs_gset:Npn` directly (or the _set variant of the data-type
  constructor).
- For messages: replace `\msg_new:nnn` with `\msg_set:nnn` (calls
  `\msg_set:nnnn` which uses `\cs_set:cpn` — no chk_free).

**Status (2026-04-26)**:
- Fixed in `expl3_sty.rs`: msg_new → msg_set; quark_new/seq/scan
  wrapped with `\cs_if_exist:NF` (commit `959d25e9d`). Reduced
  49 → 14 boxing errors.
- Pending: 14 remaining boxing errors from `\msg_redirect_module:nnn`
  in expl3_sty.rs — the redirect uses `\__msg_class_chk_exist:nT`
  which fires error if class isn't registered. Investigation deferred.
- Pending audit: scan ALL packages (not just expl3_sty.rs) for
  same anti-pattern.

---

## Pattern 3 (hypothetical, not yet observed): Late-bind CS overwrite

**Hypothesis**: bindings that use `\let \foo = \bar` to alias may now
clobber a properly-loaded dump definition with a less-complete one.
Symptom would be: dump-defined CS works correctly UNTIL a binding's
late `\let` reassigns it.

**Audit recipe**:
```bash
# Bindings with raw \let aliases targeting CSes:
grep -rnE 'raw_tex.*\\\\let\\s|Let!\\(' \
  latexml_package/src/package/

# Cross-reference with dump entries to find conflicts.
```

**Status (2026-04-26)**: not yet exercised. Add to backlog.

---

## Methodology summary

For each new sandbox failure cluster:

1. **Reproduce** with minimal `.tex` (e.g. `\documentclass{article}\usepackage{X}`).
2. **Bisect the binding** (`latexml_package/src/package/X_sty.rs`)
   by selectively disabling raw_tex blocks. Use `python3` patch
   helpers to comment-out blocks; rebuild + re-run; restore.
3. **Identify the chk_free firing CS** by reading expl3-code.tex /
   bound source for the wrapper call.
4. **Cross-reference** with `resources/dumps/latex.dump.txt` to
   confirm dump now captures the target.
5. **Apply the narrowest possible fix** (existence guard, _set
   variant) — DO NOT broadly rewrite the binding.
6. **Document** in this file under the appropriate Pattern.
7. **Re-run** sandbox via `./tools/rerun_failures.sh` after each fix
   to measure paper-recovery delta.

---

## Cross-references

- [wisdom_deferred_commands_alias.md](../.claude/projects/-home-deyan-git-latexml-oxide/memory/wisdom_deferred_commands_alias.md)
- [SYNC_STATUS.md](SYNC_STATUS.md) — Mission section
- [PERL_LOADFORMAT_AUDIT.md](PERL_LOADFORMAT_AUDIT.md) — strict-Perl LoadFormat
- Commit `e3d4f8532` — Pattern 1 fix (DEFERRED_COMMANDS alias)
- Commit `959d25e9d` — Pattern 2 fix (expl3_sty.rs guards)

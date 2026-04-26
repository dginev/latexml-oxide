# Sandbox Failures Worksheet — 181 papers

> **Active priority (2026-04-26):** strict-Perl dump parity. See
> [`SYNC_STATUS.md`](SYNC_STATUS.md) "Mission" and
> [`PERL_LOADFORMAT_AUDIT.md`](PERL_LOADFORMAT_AUDIT.md). Sandbox
> work continues opportunistically but is **not the gating front**.
> Sandbox regressions during the dump-parity push are accepted —
> re-validate after dumps stabilize.

Tracks per-cluster Rust→Perl translation gaps for the focused
~/data/sandbox_failures sandbox of error-producing papers. Each
row tracks the cluster size, root cause, fix approach, and
status.

Workflow: edit code → rebuild → `./tools/rerun_failures.sh` →
diff `~/data/sandbox_failures_<TS>/results.tsv` against the saved
baseline `docs/sandbox_failure_181_triage.tsv` → mark recovered
papers with `[x]`.

## Initial baseline (post-AR-flip, 2026-04-26)

`results.tsv` totals: 5119 Status:0 + 2598 Status:1 + 172 Status:2 +
3 Status:3 + 6 empty = **181 problem papers** (97.71% clean of
7898). Cluster shape captured in `docs/sandbox_failure_181_triage.tsv`.

## Active investigation tracks

### Track A — Plain TeX dump coverage gap

**Symptom.** Plain-TeX papers using `\settabs N \columns` (5 papers:
`astro-ph9308008, astro-ph9708022, funct-an9711006, hep-th9404085,
q-alg9505016`) error with `\columns undefined`.

**Status (2026-04-26): largely addressed by strict-Perl LoadFormat
work.** The new `plain.dump.txt` (1196 entries, runtime-loaded by
`plain_dump.rs`) captures `\settabs`/`\sett@b`/`\sett@bb`/`\s@tt@b`/
`\columns` directly (verified post-`1e04a96c8`). Re-run the
worksheet to confirm; expect these 5 papers cleared. Latex side is
the next-up gap (302/752 `\tex_*:D` aliases missing — see
`PERL_LOADFORMAT_AUDIT.md` "Remaining dump gaps").

### Other clusters (181 - 5 = 176 remaining, deferred behind Track A)

| Cluster | Papers | Class breakdown | Notes |
|---|---|---|---|
| `XMApp` in `<ltx:text>` | 19 | mixed | task #11 — math-parser shape |
| `XMTok` in `<ltx:text>` | 11 | mixed | task #11 — math-parser shape |
| `\regex_const:Nn` (mhchem/expl3) | 11 | various | task #11 — expl3 regex |
| `XMApp` in `<ltx:p>` | 7 | mixed | task #11 — math-parser shape |
| `\end{equation}` mode mismatch | 7 | mixed | math env close |
| `}` brace mismatch | 6 | mixed | gullet/parameter |
| `\columns` (plain-TeX) | 5 | (plain) | **Track A** |
| `\section` (AmSTeX dispatch) | 4 | amsppt | `project_amstex_pool_dispatcher.md` |
| `\@nil` (pgf cascade) | 4 | mixed | pre-existing pgf catcode |
| `\gnuplot` (gnuplot.sty) | 4 | mixed | per-package |
| `\+` undefined | 3 | mixed | LaTeX tabbing CS gap |
| `\columns` undefined | 3 | mixed | plain-TeX (subset of Track A) |
| `\CITE` undefined | 3 | mixed | custom .sty per-paper |
| `<box> was supposed to be here` | 3 | mixed | brace mismatch |
| `\affil` undefined | 3 | revtex | per-paper |
| `\lx@end@gen@cases` | 3 | mixed | amsmath cases |
| `XMArray` in `<ltx:para>` | 3 | mixed | math-parser shape |
| Other singletons + per-class | ~95 | mostly article | long-tail |

## Fix log

| Date | Commit | Cluster | Papers cleared | Total problem |
|---|---|---|---|---|
| 2026-04-26 (baseline) | — | — | 0 | 181 |
| 2026-04-26 (rerun #2) | `b315c86ec` | Constructor/Register PA | 2 (`hep-th9609235`, `math9712228`) | ~165 fatal-3 from new `\q_no_value` regression |

(Append rows here after each run.)

## Active regression — `\q_no_value` recursion cascade (NEW)

**Symptom.** 165 of 181 sandbox papers now hit
`Error:recursion:\q_no_value Token \q_no_value expands into itself!`
during textcomp.sty / article.cls load, fatally cascading at 10k
errors. The error fires only when the LaTeX dump is loaded; with
`LATEXML_NODUMP=1` the same papers convert with zero `\q_no_value`
errors.

**Root cause (decisive bisection).**
- Pre-`209083ff4`: expl3-code.tex aborted before line 3205
  (`\quark_new:N \q_no_value`), so `\q_no_value` never got into the
  dump. Anything that touched `\q_no_value` got the undefined-CS
  path (silent-recovery in our binding loaders).
- Post-`209083ff4` (LaTeX.pool preload): expl3 finishes loading.
  `\q_no_value` is now installed as `Stored::Expandable` with body
  `\q_no_value` (self-referential sentinel — matches Perl's
  `latex_dump.pool.ltxml` L16030
  `I(E(C('\\q_no_value'),undef,T(C('\\q_no_value'))))`).
- Our engine's `Expandable::invoke` recursion-detect (`expandable.rs`
  L149-162) emits `Error!` and returns empty Tokens — same as
  Perl's `Expandable.pm` invoke. **But Perl's textcomp.sty load
  doesn't trigger the expansion path** (Perl emits the diagnostic
  ONLY when something accidentally tries to expand a quark, which
  Perl's normal package-load flow doesn't).

**Deeper bug.** Some Rust-engine code path during package binding
load (textcomp_sty.rs's DefAccent/DefPrimitive/DefMath chain, OR
the surrounding LaTeX kernel autoload tail) is eagerly expanding
quark tokens that Perl leaves alone. Need to instrument
`expandable.rs:149` with backtrace capture to find the call site.

**Why this matters.** User directive: "it is critical we emulate
the original behavior accurately with our translation." Demoting
`Error!` to `Warn!` (band-aid) was reverted — that diverges from
Perl's error reporting. The right fix is finding and removing the
unwanted eager expansion of `\q_no_value`.

**Investigation complete (2026-04-26 iteration 2):** instrumented and
captured. Cleanly comparing Perl vs Rust:

* `\edef`/`\xdef` definitions match exactly (Perl
  `TeX_Macro.pool.ltxml:176-177` ↔ Rust `tex_macro.rs:101-110`):
  `DefPrimitive('\edef SkipSpaces Token UntilBrace DefExpanded',
  sub { do_def(0, @_); }, locked => 1)`. Both use `DefExpanded`
  parameter type, both call `do_def(global=false/true, ...)`, both
  `locked`.
* `DefExpanded` parameter type calls `gullet::read_balanced` with
  `ExpansionLevel::Partial` — same in both.
* `Expandable::invoke` recursion-detect emits `Error!`-equivalent in
  both when body == `[cs]` (Perl `Expandable.pm` invoke L72-92,
  Rust `expandable.rs:149-162`).

**The asymmetry is procedural, not semantic.** Backtrace shows the
Rust path:
```
expandable.rs:160 (\q_no_value recursion)
gullet.rs:866 (read_balanced expanding tokens)
base_parameter_types.rs:297 (DefExpanded closure)
parameter::read → read_arguments
primitive::invoke_primitive (\xdef → \edef)
stomach::invoke_token
binding/content.rs:492 (input_definitions for textcomp.sty)
binding/content.rs:1176 (require_package)
prelude/setup_binding_language.rs:56 (LoadDefinitions wrapper)
```

So during textcomp.sty's binding-load, an `\xdef` runs whose body
contains an `\edef` whose body contains `\q_no_value`. That `\edef`
expanding `\q_no_value` triggers the recursion error.

**The most likely culprit**: dump installs expl3 `\hook_use:n` chain
(`M  \hook_use:n  E  \hook_use:n  1  LP
\__hook_preamble_hook:n {#1} \__hook_use_initialized:n {#1}`).
This expands to chains that eventually contain `\q_no_value` refs
inside an `\edef` arg. Perl's `_constructs` either redefines
`\hook_use:n` to a no-op stub OR Perl's package-load architecture
intercepts BEFORE the dump's `\hook_use:n` fires.

**Fix path for next iteration:**
1. Add `\hook_use:n` (and the kernel hook family) as no-op stubs
   in `expl3_sty.rs` or `latex_constructs.rs` — they'd run AFTER
   the dump install (per strict-Perl LoadFormat ordering:
   `bootstrap → dump → constructs`), overriding the kernel chain
   with stub bodies.
2. Verify Perl-parity: confirm Perl's runtime actually runs a
   stub for these (per `wisdom_lhook_perl_parity_stub.md`).
3. Re-run sandbox to measure recovery.

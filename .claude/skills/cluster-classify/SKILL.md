---
name: cluster-classify
description: Post-sweep triage protocol. After a stage sweep produces N failing arxiv_ids, use this to group them into clusters, sample representatives, and decide which clusters are worth root-causing vs which are SHARED-FAILURE or PERL_REGRESSION noise. Avoids the recurring "we fixed 100 papers" inflation that turns out to be 8 real regressions.
---

# Cluster-classify — post-sweep triage protocol

After a stage sweep produces a TSV of N failing arxiv IDs, the
naive next step — start fixing them — loses 1–2 hours per
session to clusters that turn out to be SHARED-FAILURE with
Perl (no Rust bug to fix) or Perl-capped (uncomparable). This
skill formalises the four-step protocol that keeps the fix
queue honest.

For the per-paper verdict matrix (REAL_REGRESSION / SHARED /
PERL_REGRESSION / …) see skill `canvas-triage`. This skill
operates one level up — across many papers — to decide which
clusters deserve engineering effort vs which are documentation
work vs simply not Rust's problem.

## Mental model — the honest queue

Two principles drive the protocol:

1. **A failing paper is not a regression until classified.**
   "100 papers failed" is a measurement, not a problem. Most
   stage sweeps split roughly: 40% SHARED-FAILURE, 30%
   PERL_REGRESSION (Rust wins!), 20% Perl-capped/timeout (can't
   compare), 10% real Rust regressions. The 10% is what you
   work on; the rest is administrivia.
2. **Cluster fixes compound, paper fixes don't.** A root-cause
   fix to one paper in a 17-paper cluster typically halos to
   all 17. A symptom-fix to one paper helps that paper only.
   The protocol exists to make sure you're working at the
   cluster level.

The result is that an honestly-classified failing set is
usually 5–10× smaller than the headline count, and the work
that survives is 2–3× higher leverage per fix.

## The four steps

```
1. Tally first-error class       → grep + sort + uniq -c
2. Group papers by class          → cluster table
3. Sample 5-10 per cluster        → parity_check.sh with 180s
4. Decide per cluster              → root-cause, defer, or close
```

Run all four in order. Skipping step 3 is the canonical mistake.

## Fan-out via Workflow (default when ≥3 clusters need sampling)

Steps 2–4 are a fan-out: each cluster's sampling and interpretation is
independent until the SYNC_STATUS write-up. When step 1's tally yields
3 or more clusters worth sampling, orchestrate them with the
**Workflow tool** instead of sampling serially. The Workflow tool needs the
user's opt-in: a user-typed `/cluster-classify` supplies it; if you reached
this skill on your own, sample serially or ask before fanning out.

Shape (hybrid: scout inline, then pipeline):

1. Run step 1 inline (cheap bash; or delegate to the `log-scanner`
   agent when the log directory is huge) and build the cluster list.
2. `Workflow` with `args` = the cluster table, **pipelined per
   cluster** (no barrier between stages):
   - **Sample stage** (`effort: 'low'`, label `sample:<class>`):
     shuf-sample 5–10 papers, run
     `TIMEOUT_SECS=180 tools/parity_check.sh -`, and return a schema
     object `{cluster, size, verdicts: [{id, verdict}]}` — raw
     parity_check verdicts only, no interpretation.
   - **Verdict stage** (inherit effort): interpret the sample against
     the statistical table below, explicitly checking the
     misclassification traps (Perl-capped, Perl-timeout, staleness,
     heterogeneous split). Returns
     `{cluster, verdict, confidence, needs_subgrouping}`.
   - **Adversarial verify** (REAL_REGRESSION verdicts only): one
     skeptic agent per cluster prompted to REFUTE — "is this actually
     SHARED? re-check a sampled paper's Perl log for cap/timeout." A
     refuted cluster re-enters the verdict stage's traps; it does not
     reach SYNC_STATUS.
3. The step-4 SYNC_STATUS write-up stays in the main session — that is
   the part the user reviews, and notes in SYNC_STATUS compound while
   agent context evaporates.

Concurrency caution: every sample agent runs Perl+Rust conversions.
The workflow pool cap (~10 concurrent) is the throttle — do not add
parallelism inside the sample stage, and respect the per-paper
RAM-guard discipline (memory `feedback_sandbox_run_discipline` § feedback_sandbox_ram_guard).

## Step 1 — tally first-error classes

```bash
RESULTS=~/data/<corpus>_stage_<N>_html/results.tsv
LOG_DIR=$(dirname "$RESULTS")

# Verify the verdict column index against the TSV header before
# trusting $7 — schemas change. `head -1 "$RESULTS"` should show
# columns including "status" (or similar).
awk -F'\t' 'NR>1 && $7 != "ok" {print $1}' "$RESULTS" \
  > /tmp/stage_fails.txt

# Extract first-error class per failing paper. Strip ANSI before
# matching (logs from older binaries / cortex zips carry \x1b[31m),
# and use [a-zA-Z_] not [a-z] (categories can be uppercase/underscore).
# See skill `canvas-triage` § "Signal integrity" for why this matters.
while IFS= read -r id; do
  log="$LOG_DIR/$id.log"
  [[ -f "$log" ]] || continue
  cls=$(sed 's/\x1b\[[0-9;]*m//g' "$log" | grep -oE 'Error:[a-zA-Z_]+:[a-zA-Z_]+' | head -1)
  printf '%s\t%s\n' "$id" "$cls"
done < /tmp/stage_fails.txt > /tmp/stage_first_err.tsv

# Rank by frequency
cut -f2 /tmp/stage_first_err.tsv | sort | uniq -c | sort -rn | head -20
```

Empty `cls` (paper failed but no `Error:` matched) is a **parse miss,
not a clean paper** — it usually means the failure is a `Fatal:`,
a timeout, or an OOM. Fail-safe: bucket empty-`cls` papers separately
and inspect them, never drop them as "no error found".

Output is a ranked headline list:

```
  17 Error:latex:\GenericError
  11 Error:undefined:\foo
  10 Error:unexpected:_
   7 Error:malformed:ltx:section
   ...
```

These are headlines, NOT yet decisions. Every line above needs
sampling in step 3 before any scoping. The historical pattern
is that the top headline turns out to be 90% SHARED-FAILURE
roughly half the time.

## Step 2 — group papers by class

```bash
# Show papers in a specific cluster
awk -F'\t' '$2 == "Error:unexpected:_" {print $1}' \
  /tmp/stage_first_err.tsv > /tmp/cluster_underscore.txt
wc -l /tmp/cluster_underscore.txt
```

For each cluster you intend to investigate, save its paper list
to a file. You'll feed those into the sampler in step 3.

## Step 3 — sample 5–10 per cluster

This is the step that prevents 90% of misclassification.

```bash
# Random sample of 8 papers from the cluster
shuf -n8 /tmp/cluster_underscore.txt \
  | TIMEOUT_SECS=180 tools/parity_check.sh -
```

For each sampled paper, parity_check.sh emits one of:

- `BOTH CLEAN` — the sweep was stale; paper now passes.
- `REAL_REGRESSION` — Rust > Perl. Investigate.
- `OUT-OF-SCOPE` — Rust == Perl. SHARED-FAILURE, no Rust bug.
- `PERL_REGRESSION` — Rust < Perl. Rust wins; don't "fix".
- `OUT-OF-SCOPE? (Perl-capped)` — Perl=101, can't compare.
- `OUT-OF-SCOPE? (Perl-timeout)` — Perl needs >180s; grep partial.

**Statistical interpretation.** A random sample of 8 from a
homogeneous cluster gives strong signal:

| Sample result | Interpretation |
|---------------|----------------|
| 8/8 REAL_REGRESSION | Cluster is solidly Rust-only. Worth root-causing. |
| 8/8 SHARED-FAILURE | Cluster is solidly upstream. Defer / document. |
| 6/8 SHARED, 2/8 REAL | Heterogeneous — re-sample 4 more to see if a sub-cluster emerges. |
| Mixed across all verdicts | Not actually one cluster — error-class grouping wasn't enough. Sub-group by NEXT error class. |

Sample size of 8 is a heuristic, not a rule. For a 5-paper
cluster, sample all 5. For a 50-paper cluster, 8 is plenty —
diminishing returns beyond.

## Step 4 — decide and record per cluster

Map the step-3 sample result to an outcome, then write the
cluster up in `docs/SYNC_STATUS.md` under the active round's
cluster section. Future sessions read SYNC_STATUS, not your
session notes.

| Cluster verdict | Outcome | Record where |
|------------------|---------|--------------|
| Solid REAL_REGRESSION | Root-cause one witness; the fix usually halos across the cluster. Land as one focused commit. Re-sweep to measure halo. | SYNC_STATUS active cluster section |
| Solid SHARED-FAILURE | Document; check whether it's a surpass-Perl candidate (skill `surpass-perl`). | SYNC_STATUS "OUT-OF-SCOPE" or "surpass candidate" |
| Solid PERL_REGRESSION | Record paper IDs; do NOT "fix" — these are Rust wins. | `memory/project_rust_supersedes_perl.md` |
| Heterogeneous | Sub-group by next-error class and re-classify, OR pick the largest REAL_REGRESSION sub-cluster and treat the rest as long-tail. | Re-enter step 1 on the sub-group |

The act of writing the cluster up is part of the protocol —
notes left in conversation context evaporate; notes in
SYNC_STATUS compound across rounds.

## Sizing the work — when to act on a cluster

Cluster size tells you the leverage of a fix; verdict tells you
its legitimacy:

| Size × verdict | Action priority |
|----------------|-----------------|
| ≥10 papers, REAL_REGRESSION | **Top priority** — single fix, large halo. |
| ≥10 papers, SHARED | Consider surpass-Perl if catcode/mode-quality issue. |
| 3–9 papers, REAL_REGRESSION | Medium priority — fix when in the neighborhood. |
| 3–9 papers, SHARED | Document; do nothing unless trivial. |
| 1–2 papers, REAL | Long-tail. Pick up only if drift-prone (e.g., schema). |
| 1–2 papers, SHARED | Ignore — not worth recording per-paper. |

## Common misclassification traps

1. **First-error grouping isn't always semantic grouping.**
   `Error:undefined:\foo` might mean 17 different `\foo`s in
   17 different contexts. Sample to verify the cluster is real
   before scoping the fix.
2. **Perl-capped papers look like REAL_REGRESSION at first
   glance.** Perl=101 is the MAX_ERRORS cap, not the true count.
   Parity_check.sh flags this as "OUT-OF-SCOPE? (Perl-capped)"
   — respect that, don't reclassify.
3. **Perl-timeout papers are NOT errors.** Some papers take Perl
   45 minutes legitimately. On timeout, grep the partial Perl
   log for `Error:[a-z]+:`. If zero, Perl was clean so far —
   treat as suspected REAL_REGRESSION; if non-zero, Perl is also
   failing.
4. **Sweep staleness.** Between the sweep and the
   classification, the binary may have been rebuilt and some
   papers now pass. Re-run sampled papers individually before
   counting them as failures.

## Anti-patterns

1. **Scoping a fix from the cluster headline alone.** "17 papers
   fail with `Error:X:Y` — let's fix `X:Y`" without sampling has
   historically produced ~10× scope inflation. Always run step 3
   before step 4.
2. **Sampling 1 paper per cluster.** A single witness collapses
   the cluster to anecdote. Sample 5+ for any cluster size ≥10.
3. **Treating mixed clusters as homogeneous.** When sample
   verdicts split (REAL, SHARED, PERL_REGRESSION mixed in one
   cluster), the error CLASS isn't the right grouping — the
   ROOT cause is. Sub-group on the next-error class and
   re-classify.
4. **Recording per-paper findings instead of per-cluster.**
   Per-paper notes don't aggregate across rounds. Per-cluster
   headlines in SYNC_STATUS compound: "Cluster A — Catcode-leak
   through `[]`" survives 5 sessions; "Paper 2604.00193
   fails" doesn't.
5. **Fixing without measuring halo.** After landing a
   cluster-level fix, re-sweep the same papers. If only the
   witness recovers and the others don't, the fix is local; the
   cluster isn't what you thought. Sub-classify again.
6. **Investing in a long-tail cluster before clearing the big
   ones.** A 17-paper REAL_REGRESSION cluster outranks a
   3-paper REAL cluster. The priority table above is the
   guidance; respect it across the round.

## Output format — what goes into SYNC_STATUS

A clean cluster entry in `docs/SYNC_STATUS.md` looks like:

```
### Cluster <N> — <first-error class> (<size> papers)

**Status**: REAL_REGRESSION / SHARED-FAILURE / mixed

**Sample** (8 random): N BOTH CLEAN, M REAL_REGRESSION, K SHARED, ...

**Root cause** (if REAL): [one paragraph]

**Action**: [root-cause witness X; fix path; expected halo]
or [document under OUT-OF-SCOPE; revisit if surpass-Perl
candidate emerges]
```

This is the format other sessions can pick up cold.

## Related

- Verdict matrix per paper: skill `canvas-triage`
- Decide if SHARED warrants Rust-side divergence: skill `surpass-perl`
- 180s rule rationale: memory `feedback_cluster_shared_failure_check`
- Perl-cap / Perl-timeout handling: memory `wisdom_perl_max_errors_cap`,
  `feedback_perl_parity_timeout_handling`
- Where cluster decisions land: `docs/SYNC_STATUS.md`

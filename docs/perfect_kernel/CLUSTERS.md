# Perfect Kernel — failure-cluster worklist (living)

Rebuilt after each sweep from `~/data/perfect_kernel/sweep_verdicts.tsv` +
first-error extraction. Discipline: cluster by first-error signature, sample
2–3 representatives before believing a cluster, count *clusters* fixed, not
documents.

## Current clusters

_(baseline sweep in progress; pre-sweep findings below)_

| Rank | Signature | Docs | Representative | Verdict / plan |
|---|---|---|---|---|
| — | `Error:expected:{ Expected opening '{'` from `DefPlain`/required-brace args when the `{...}` body sits on the next line | TBD (sweep) | abraces-doc (via raw `ltxdockit.sty` `\lstnewenvironment`); 10-line repro: `\lstnewenvironment{x}[1][]` + body on next line | **SHARED with Perl 0.8.8** (verified same-host). Real TeX skips blanks before an undelimited argument (tex.web macro_call), so both engines are wrong vs the kernel. Candidate **surpass-perl** divergence: make `readBalanced(require_open)` skip spaces/comments before the `{`. Qualifies on tests 1 (catcode/argument-scanning quality) and 3 (Perl benefits identically); awaiting witness count ≥5 from sweep + user approval per protocol. |
| — | `Error:undefined:\newunit` etc. in a4wide | n/a | a4wide | **DOCUMENT-STALE**: manual written against siunitx v1; TL2025 pdflatex fails with 39 errors. Excluded from S1 bar via oracle pass. |

Settled protocol point (user directive 2026-08-31): compiled `.rs` bindings
keep precedence under rawclasses/rawstyles; an experiment demoting the contrib
tier to raw was reverted the same day. Corpus focus = bindingless packages.
The `rawclasses` no-OmniBus requirement was verified already-true for
bindingless classes and is guarded by
`cluster_package_guards::rawclasses_binding_precedence_and_no_omnibus`.

## Retired clusters

| Signature | Resolution | Guard |
|---|---|---|
| | | |

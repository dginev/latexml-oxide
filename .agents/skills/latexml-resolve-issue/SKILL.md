---
name: latexml-resolve-issue
description: Drive a latexml-oxide GitHub issue from current reproduction through the correct test layer, principled implementation, validation, and a focused pull request. Use when asked to investigate, fix, or prepare a PR for a numbered public bug, feature, documentation issue, or build problem.
---

# Resolve a public issue

Read the complete issue and current comments with `gh issue view`. Restate every
requested behavior and classify the ticket as bug, feature, documentation, or
other before changing code.

## Protect the checkout

Inspect branch and status first. If publication is requested, use one ticket per
branch and PR, but do not switch, pull, delete, or move branches in a way that
risks unrelated dirty work. Never bundle a second ticket or ambient changes into
the issue commit.

## Reproduce and classify

1. Reproduce the reported symptom on the current checkout using the issue's MWE
   verbatim where possible.
2. For conversion behavior, use `latexml-canvas-triage`: compare current Rust
   with verbose same-host Perl before deciding the fix policy.
3. Reduce a large witness with `latexml-min-repro`, preserving the exact canary.
4. Read the owning code, Perl oracle, current docs, and relevant history. Record
   a root-cause hypothesis only when evidence distinguishes cause from cascade.

## Put the red guard at the failing layer

| Failure surface | Test shape |
|---|---|
| core conversion XML | wired `.tex`/`.xml` fixture pair |
| post-processing | Rust test using the relevant post pipeline or existing post golden pattern |
| final HTML or CLI option | invoke the built binary or the whole-pipeline library entrypoint |
| docs | verify each changed claim against current source or behavior |

Hand-author the expected behavior so the guard is red on the buggy implementation
for the reported reason. Do not bless output from the buggy binary. A new fixture
pair requires `cargo clean` once for compile-time discovery.

For a feature or a multi-session change to shared machinery, maintain a concise
scratch approach containing the issue, canary, root-cause evidence, Perl/TeX
source, proposed behavior, adjacent risks, and validation gates. Do not commit
ephemeral scratch notes.

## Implement the complete deficiency

- Use `latexml-perl-port` for parity behavior. A shared Perl failure requires the
  `latexml-surpass-perl` approval protocol, not a silent improvement.
- Fix the most general responsible mechanism within ticket scope. Avoid both
  per-witness patches and unrelated refactors.
- Extend the same guard with adjacent cases when they exercise the same mechanism;
  use a separate test only when the harness layer differs.
- Re-read the issue after the focused test is green. Enumerate every explicit and
  implied case and verify the change does not leave a platform, option, or sibling
  behavior unresolved.

## Validate and publish

1. Run focused red-to-green evidence and original witnesses.
2. Run `cargo nextest run --workspace` to completion before opening a code PR.
3. Run nightly clippy with `-D warnings` and formatting when applicable.
4. Reconfirm verbose same-host Perl parity or cite the approved divergence.
5. Review the exact diff for unrelated files, assertion weakening, unguarded
   side effects, and claims not backed by a run.
6. Open a concise PR explaining diagnostic, approach, and actual validation.
   Use `Closes #N` only for the ticket meant to auto-close; avoid accidental
   closing keywords for other referenced issues.
7. When asked to carry publication through, watch all CI platforms to completion
   and investigate platform-specific native-library failures rather than assuming
   a local Linux pass is sufficient.

Do not merge or deploy unless the user explicitly requests that action.


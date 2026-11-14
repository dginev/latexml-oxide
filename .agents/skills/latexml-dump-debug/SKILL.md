---
name: latexml-dump-debug
description: Diagnose latexml-oxide kernel dump and raw-load divergences. Use when default dump mode and LATEXML_NODUMP behave differently, an engine override appears clobbered, format initialization fails, a dump seems stale, or changes affect bootstrap/base/construct definition ordering.
---

# Debug the kernel dump pipeline

Read `docs/parity/DUMP_DESIGN.md` first. Dump files are generated, TeX
Live-year-specific state and are not committed. If the ambient dump is absent,
generate it with `tools/make_formats.sh` before diagnosing it.

## Establish the load path

The normal alternatives are:

```text
bootstrap -> dump replay -> constructs
bootstrap -> base        -> constructs
```

Only one path may run. Dump replay assigns every record unconditionally, so the
last definition wins. The constructs source is now split under
`latexml_engine/src/latex_constructs/`; inspect `mod.rs` for current order rather
than relying on historical monolithic-file line numbers.

## Always start with the differential

Run the same input twice with the same binary and options, once with
`LATEXML_NODUMP=1` and once normally. Capture full logs, strip ANSI, and compare
diagnostic class/severity plus output:

| NODUMP | Default | Interpretation |
|---|---|---|
| clean | fails | likely dump replay clobbered a prior definition |
| fails identically | fails identically | not primarily a dump bug; classify normally |
| fails | clean | dump masks a broken raw-load path; fix raw loading |
| clean | clean | stale binary/input or non-reproducing report |

Do not infer a dump bug from one mode alone.

## Locate the first wrong definition

1. Search the generated dump for the control sequence and identify its record
   kind.
2. Search the current Rust engine for every writer of that control sequence.
3. Compare the writer order with the actual load phases.
4. When necessary, compare focused Perl and Rust `--debug=defining,assigning`
   traces and stop at the first divergence; later differences are usually
   cascade.

A confirmed replay clobber may require reapplying the intended binding in the
final constructs phase. Put it in the current last-loaded constructs section,
after verifying that load order in `latex_constructs/mod.rs`. Do not patch a
historical filename or edit the generated dump directly.

## Regenerate only from a clean initializer

Regenerate after relevant bootstrap/base/construct changes, a TeX Live upgrade,
or proof that the local dump is stale:

```bash
PROFILE=debug tools/make_formats.sh
```

Use the release profile only when the owning release procedure requires it. The
plain and LaTeX initializer runs must have zero errors before accepting their
dumps. Broken state captured in a dump affects every later conversion.

## Route non-clobber cases correctly

- Lazy pool loading and path-aware loaded-state bugs need engine/package loading
  fixes, not a final-phase alias.
- Closure-backed definitions cannot be serialized as token bodies; correct the
  representation or phase according to `DUMP_DESIGN.md`.
- If both paths fail, use `latexml-canvas-triage` and
  `latexml-perl-port` rather than adding a dump-mode workaround.

Validate both load paths, the focused guard, the original witness, and the full
suite before publication.


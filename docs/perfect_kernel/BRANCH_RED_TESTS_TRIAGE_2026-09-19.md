# Branch red-tests triage — 2026-09-19

**Queued for a fix (user directive: "unrelated failures should still be queued
for a fix, we want complete improvement").** 9 guard tests are RED on clean HEAD
of `perfect_kernel` — confirmed pre-existing (stashing unrelated in-flight edits
did not change them), so NOT caused by the 56eg/56eh/56ei semantic-markup work.
The pre-push lint gate does NOT run the full test suite, so these do not block
pushes — but the local `nextest` suite cannot go fully green until they are fixed.

## The 9 tests (binaries `cluster_package_guards`, `06_cluster_regressions`)

`openout_then_input_same_run`, `raw_stex_sty_loads_not_the_perl_ltxml`,
`maketitle_executes_dropped_class_body_after_frontmatter`,
`uspatent_maketitle_defines_parnum_counter`, `istgame_and_tikz_trees_child_nodes`,
`tcolorbox_self_terminating_hands_to_end`, `tikz_and_tcolorbox_styles_match`,
`spill_gated_node_boxes_stays_bounded`, `codehigh_dochighinput_is_bounded`.

## Shared symptom

Every one prints, during conversion, the same two LaTeX-kernel errors and then
its `error_count == 0` (or structural) assertion fails on those 2 extra errors:

```
Error:errmessage:\errmessage LaTeX Error: Cannot run piped system commands.
Error:errmessage:\errmessage LaTeX Error: Mismatched LaTeX support files detected.
```

`Mismatched LaTeX support files detected` is a real LaTeX-kernel message (not
emitted by our Rust code — grep clean), near the `latexrelease.sty` format-vs-
support-file version check. Prime suspect: the **dual-TeX-Live schism** — this
host has distro `/usr/share/texlive` (its `kpsewhich` = `/usr/bin/kpsewhich`,
first on PATH) and vendor `/usr/local/texlive/2025`; the engine's in-process
libkpathsea anchors on its compile-time distro tree unless `TEXMF*` is overridden
(`tools/perfect_kernel/run_doc.sh` pins them; the test harness does not). The
committed dumps `resources/dumps/{plain,latex}.2025.dump.txt` (mtime 2026-09-18)
carry a baked-in kernel version that may not match the support `.sty` files the
in-process kpathsea reads at test time. See memory `wisdom_dual_texlive_kpathsea_schism`.

## Status

Read-only root-cause in flight (determining exact trigger + version strings, the
env-vs-real verdict per test, and the ranked fix: regen the dump via
`tools/make_formats.sh` against the right TL / pin `TEXMF*` in the test harness /
make the guards robust to the 2 env errors / a real code fix). **This doc is the
queue entry; update with the fix and move to `docs/archive/` when green.**

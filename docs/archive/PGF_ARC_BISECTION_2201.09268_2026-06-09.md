# pgf line–arc intersection bisection non-termination (2201.09268) — 2026-06-09

**Status:** root-caused; mitigated by the stomach cycle guard
(`Fatal:Stomach:Recursion`, ~528 MB / 2.5 s instead of a 4.5 GB OOM). One
related faithfulness fix landed (`02a5b2103f`). The final bit-exact-trig fix is
deferred as deep/high-risk for a single paper.

## Symptom

`2201.09268` (an `acmart` paper with a complex tikzpicture: `block`/`native`
nodes that are `rounded rectangle` / `rectangle callout` shapes, `transform
shape`, `scale=0.75`, `remember picture`, several `to [loop]` edges) drives an
**unbounded loop** in Rust that Perl converts cleanly (1 error). Each loop
iteration opens a `{` group whose empty box accumulates in `box_list` → memory
runaway. Before the cycle guard it OOM'd at 4.5 GB in 31 s.

## Root cause (definitive)

The loop is pgf's `\pgfmathpointintersectionoflineandarc`
(`pgfmathcalc.code.tex`), which finds where an edge crosses a node's elliptical
boundary by **bisection** over an angle. The bisection runs in an **unbounded**
`\pgfmathloop` whose ONLY exit is the *exact* comparison
`\ifdim\x pt=\q pt` (line 447). Its apparent convergence guard
`\ifdim\p pt=\s pt` (line 418) merely *skips the body* and then spins — there is
no break there. So the loop terminates **only** if the probe angle `\q` becomes
*exactly* equal to the target angle `\x`.

Side-by-side Rust-vs-Perl traces (hooking `\pgfmathanglebetweenpoints` and
`\pgfmathdivide@`) show the two engines are **byte-identical** through ~29
bisection steps — same `\s`/`\e`/`\p` midpoints, same probe angles — and then
diverge: Perl's bisection hits the exact match and breaks (a fresh bisection
begins); Rust's misses it and keeps narrowing until `\p == \s`, then spins.

The miss traces to a **last-digit (1e-5) difference in
`\pgfmathanglebetweenpoints`** for specific inputs (observed e.g. `90.00006`
Perl vs `90.00007` Rust). When such a value is a bisection *target* `\x`, the
exact `\ifdim\x pt=\q pt` can never be satisfied in the engine whose `\x` is off
by one unit in the last place.

## What was ruled out (all match Perl bit-for-bit)

- `\pgfmathsin@` / `\pgfmathcos@` (0.5, 0.86603, …)
- `atan2` / `atan` via `\pgfmathparse` (45.0, 66.80141, …)
- TeX dimension×factor arithmetic (`\d=10pt \d=0.86603\d` → 8.66028pt)
- `\pgf@sys@tonumber` (dimension → number)
- `\pgfmathmod@` / `\pgfmathadd@` / `\pgfmathdivide@` (the bisection arithmetic)
- `rad2deg` vs Rust `f64::to_degrees` (both `_RD * rad`, same constant)

Every *isolated* link matches; only the *composed* `\pgfmathanglebetweenpoints`
chain produces the 1e-5 drift for some inputs. Both engines call the same glibc
`atan2`, so the residual is an ordering/rounding artifact of the chained pgf
computation that is extremely delicate to reproduce in isolation.

## Landed faithfulness fix (`02a5b2103f`)

While investigating, found and fixed an unrelated real divergence: Rust's
`@`-internal pgfmath functions formatted integers with a spurious `.0`
(`\pgfmathmod@{370}{360}` → `10.0`) where Perl/real-pgf return raw `10` (only the
public `\pgfmathparse` adds `.0`). `pgfmath_result_str` now strips it. Did NOT
resolve this loop (the `.0` is dimension-equal under `\ifdim`), but it is a
correct improvement (suite 1400/0).

## Mitigation (landed `7190b48b8e`)

The stomach windowed cycle guard catches the box-accumulation runaway
(period-2) and aborts with a clean `Fatal:Stomach:Recursion` at ~528 MB / 2.5 s.
So the kernel is protected; the paper still fails (empty output) pending the
root fix.

## Path to a real fix (deferred)

Make Rust's `\pgfmathanglebetweenpoints` bit-exact to Perl for all inputs. Since
the divergence is a 1e-5 last-place artifact of the chained pgf TeX computation
(not a single op), the tractable options are (a) a native Rust
`\pgfmathanglebetweenpoints` binding mirroring Perl's exact sequence, or (b)
making pgf's bisection tolerant (break on `\p == \s` convergence, upstream pgf
behavior change — not faithful). Both carry broad tikz/pgf regression risk and
are not justified for one paper while the guard holds. Revisit if the bisection
non-termination recurs across multiple canvas papers.

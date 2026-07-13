# `\kbordermatrix` `\halign`-in-math IfLimit loop — OPEN, **HIGH DIFFICULTY**, post-release

> **Status: OPEN. Difficulty: HIGH. Priority: post-release.**
> Genuine Rust-only divergence (Perl completes the same input). Root-caused
> 2026-07-10 to the alignment/math-mode frame interaction; **not fixed** — the
> real fix is deep `\halign`/`\lx@begin@alignment`-family work. A first fix
> attempt targeted an *orthogonal* bug (see "Red herring" below) and was
> reverted. This file is the detailed record + minimal reproducer to resume from.

Witness: **arXiv:2605.23849** (Cluster H in
[`../../performance/STABILITY_WITNESSES.md`](../../performance/STABILITY_WITNESSES.md)).
Surfaced by mining the 2605+2606 60k-doc telemetry as a ~149s digest-runaway →
fatal. It is one instance of the broader **`\lx@begin@alignment` family** (the
full-arXiv corpus shows ~12.1k `\lx@begin@alignment` fatals), so a real fix here
likely pays off far beyond this one paper.

## TL;DR

`\kbordermatrix{…}` (a bordered-matrix macro built on `\ialign` with per-cell
`$##$` math) inside `\begin{equation}` makes Rust emit
`Error:unexpected:\halign Attempt to close a group that switched to mode math`,
which cascades into `Fatal:Timeout:IfLimit` (16M conditional cap) after ~25–107s.
**Perl converts the same document in ~0.4s** (2 errors: undefined `\@arstrut` and
`\\`, from which it recovers). So this is GENUINE-RUST-ONLY.

## Reproducer (this directory)

`kbordermatrix.sty` is **not in TeX Live** (the arXiv author bundled it), so it is
committed here alongside the 9-line driver.

```bash
cd docs/known_crashes/kbordermatrix_halign_math
# Rust — loops → Fatal:Timeout:IfLimit (bounded here by --timeout):
latexml_oxide --includestyles --path=. --timeout=25 --log=rust.log kbm.tex
sed 's/\x1b\[[0-9;]*m//g' rust.log | grep -E '^(Error|Fatal):'
#   Error:unexpected:\halign Attempt to close a group that switched to mode math   (x2)
#   Error:unexpected:\lx@end@inline@math Attempt to end mode math                   (x…)
#   Fatal:Timeout:IfLimit  Conditional limit of 16000000 exceeded, infinite loop?

# Perl (same host, ar5iv-equivalent --includestyles) — completes:
latexml --includestyles --path=. kbm.tex
#   Conversion complete: 2 errors; 2 undefined macros[\@arstrut, \\]  (reqd. ~0.4s)
```

## Root cause (what is actually happening)

The error is raised in `stomach.rs::egroup` (the branch at ~L459-467, "Attempt to
close a group that switched to mode {}"). The diagnostic frame message is the key:

```
Error:unexpected:\halign Attempt to close a group that switched to mode math
    at Main; line 647 col 6
    current frame is mode-switch to math due to T_CS[\lx@begin@inline@math] at …:647
```

`\kbordermatrix` runs `\ialign` (= `\halign` with `\everycr{}`/`\tabskip=0`) whose
column template wraps **each cell in inline math**: `…\hfil\@arstrut$\kbcolstyle
##$\hfil…` and `…\hfil$\@kbrowstyle ##$…`. The whole construct is digested inside
the surrounding **display math** of `\begin{equation}`.

The divergence: as the alignment closes a boxing group (`\halign`'s group, or a
`\crcr`/cell boundary), an inline-math frame opened by `\lx@begin@inline@math`
(the `$` in a cell) is **still on the frame stack** — Rust's `egroup` refuses to
pop a frame that "switched to mode math" and raises the error instead of
recovering. The error path then re-enters and spins the conditional machinery to
the 16M `IfLimit`. Perl's `endMode`/`egroup` recovery for the same mismatch is
softer: it emits the undefined-`\@arstrut`/`\\` errors and continues to a complete
(if imperfect) document.

So the fix lives in the **alignment × math-mode frame accounting**: either the
inline-math frame from a `$##$` cell must be closed at the cell/`\crcr` boundary
before the `\halign` group closes (TeX semantics: `$…$` inside an `\halign`
template is balanced within the cell), or `egroup`'s "switched to mode math"
recovery must degrade like Perl instead of looping. This is the same class as the
broader `\lx@begin@alignment` / `\halign`-in-math cluster.

Entry points for whoever resumes:
- `latexml_core/src/stomach.rs` — `egroup` (the raised error), `begin_mode`/
  `end_mode`, the `boxing`/frame stack, `BOUND_MODE` bookkeeping.
- `latexml_engine/src/tex_tables.rs` (or wherever `\halign`/`\ialign`/`\crcr`/
  cell templates are digested) — how a cell's `$…$` opens/closes an inline-math
  frame relative to the alignment's boxing group.
- Trace with `LXML_TRACE_BOUND_MODE=1` (already wired in `stomach.rs`) to watch
  who binds/leaves `BOUND_MODE` across the alignment.
- Ground-truth the intended TeX behaviour in `background/tex.web` (`\halign`,
  `\cr`, math-mode-in-alignment) and `background/texbook.tex`.

## Reduction map (what triggers, what does NOT) — 2026-07-10

Establishes that the trigger is **kbordermatrix's specific machinery**, not raw
`\ialign`, and separates it from a look-alike SHARED loop. All Rust runs used the
current `--release` binary with `--timeout`; Perl same-host with `--includestyles`.

| # | Construct | Rust | Perl | Note |
|---|---|---|---|---|
| **kbm.tex** | full `\kbordermatrix{…}` in `equation` | **loops → IfLimit** | **completes 0.4s** | ★ the faithful repro (this dir) |
| bm.tex | `\bordermatrix{…}` (plain-TeX built-in, no sty) in `equation` | clean 0.2s | clean 0.35s | so it's kbordermatrix-**specific**, not bordermatrix |
| A | raw `\ialign{$##$&$##$\crcr a & b\crcr}` in `\vbox` in display math | clean | — | raw ialign + `$##$` alone does NOT trigger |
| B | A + `$\left[\vcenter{\vbox{\unvbox0}}\right]$` wrap | clean | — | the delimiter wrap alone does NOT trigger |
| C | B + `\@arstrut` in the template | clean | — | the undefined `\@arstrut` alone does NOT trigger |
| D | wider `\ialign` + `\hfil`s + 2 rows + wrap | clean | — | — |
| E | + `\let\\\@arraycr` and `\\` row separators | clean | — | — |
| F/G/H | **raw `\loop \setbox2=\hbox{\unhbox5 \unskip \setbox3=\lastbox}\ifhbox3 …\repeat`** peel over an hbox of `$a$…$b$` (or plain) cells | loops (`Fatal:Timeout:Convert`) | **also loops (rc=124)** | **SHARED pathology — a RED HERRING, not this bug** |

The lesson from F/G/H: an isolated `\lastbox`/`\unhbox` box-peel loop over an
hbox **also infinite-loops in Perl**, so it is *not* the Rust-only bug and must not
be used as the reproducer. The Rust-only failure only appears with the **full
`\kbordermatrix` `\ialign`-in-math** machinery (kbm.tex). Reducing kbm.tex below
the whole macro was not achieved (B–E all fail to trigger) — the interaction is
between several kbordermatrix pieces at once; cracking that reduction is part of
the "high difficulty".

## Red herring / orthogonal side finding (a real bug, but NOT this one) — do not conflate

Chasing kbm.tex via the F/G/H box-peel repro surfaced a **separate, genuine
faithfulness divergence** that was fixed-in-a-branch, verified to change hbox
content to byte-match Perl, then **reverted** because it does not fix this witness
and is an unvalidated hot-path change:

> Rust's **hbox** content carries spurious `{`/`}` brace-marker `Box`es that Perl's
> `readBoxContents` (`LaTeXML/lib/LaTeXML/Engine/TeX_Box.pool.ltxml` L164-185) does
> not. The horizontal branch of
> `latexml_engine/src/base_utilities.rs::predigest_box_contents_in_mode` shortcuts
> through `invoke_token(T_BEGIN)` → the `{` primitive → `digest_next_body`, which
> **invokes the closing `}`** and appends its `isEmpty` T_END marker box. The
> *vertical* branch (and Perl) instead STOP at `T_END` **without** invoking it.
> **VBox was already migrated off this** (`VBoxContents` carries a `{}` `reversion`
> in `tex_box.rs` to compensate); HBox was not.

A parallel fix — route the horizontal mode through the same explicit
stop-at-`T_END` loop and add an HBoxContents `{}` reversion — makes hbox content
match Perl and passes local box tests, but (a) does not fix this witness (which
fails earlier, in `\halign`-in-math) and (b) is a broad hot-path change needing a
corpus output-neutrality diff. **Recorded as a candidate future consistency fix
in `../../performance/STABILITY_WITNESSES.md` Cluster H; not shipped.** If someone
takes it up: re-apply, run the full 1534-test suite + an isolated before/after
byte-diff on a corpus sample, and ship it on its own merits — separately from this
crash.

## Cross-references

- [`../../performance/STABILITY_WITNESSES.md`](../../performance/STABILITY_WITNESSES.md) — Cluster H (the four digest-runaway witnesses; this is #1).
- [`../../SYNC_STATUS.md`](../../SYNC_STATUS.md) — Beyond-Perl levers section (BP-4 retired; Cluster H reclassified as Target-1 parity loop bugs).
- Full-arXiv `\lx@begin@alignment` 12.1k cluster (memory `full-arxiv-corpus-reference-2026-06-30`).

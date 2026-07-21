# `\kbordermatrix` `\halign`-in-math IfLimit loop — ✅ **FIXED 2026-07-20**

> **Status: FIXED** (`latexml_engine/src/latex_constructs.rs`, one `Let!` beside
> the existing `\@tabularcr` retraction). The witness **arXiv:2605.23849** now
> converts in **1.9 s with 0 errors** (1.34 MB XML, 985 formulae, 8 XMArrays) —
> it previously ran ~149 s into a token-limit Fatal emitting **0** formulae.
> Same-host Perl needs 52.7 s and still reports 3 errors on the same paper, so
> this is now a **surpass-Perl** result with identical structure counts.
> Regression guard: `latexml_oxide/tests/alignment/arraycr_halign.{tex,xml}`.
> The history below is retained because the *reduction* and the ruled-out
> hypotheses are the reusable part.
>
> ## The actual root cause (2026-07-20) — NOT the frame accounting
>
> The 2026-07-10 hypothesis ("`egroup` refuses to pop a math frame; make the
> recovery degrade like Perl") was **wrong**, and so was the framing of this as
> deep `\lx@begin@alignment` surgery. The bug was an **inherited-kernel-macro
> leak**, the same class as the `\+` retraction at the `latex.rs` seam:
>
> * `\kbordermatrix` does the documented `\bordermatrix` idiom
>   `\let\\\@arraycr` inside its own `\ialign`.
> * **Perl LaTeXML never defines `\@arraycr`** — it does not raw-load
>   `latex.ltx`, so Perl's `\\` is simply *undefined* there. That is exactly the
>   2-error "recovery" everyone read as Perl being more robust; Perl was
>   **skipping the construct**, not surviving it.
> * **Rust's kernel dump DOES carry the real `\@arraycr`/`\@xarraycr`**
>   (latex.ltx L16583-16585). Its body balances TeX's `align_state` with the
>   classic ``${\ifnum0=`}\fi … \ifnum0=`{\fi}${}\cr`` brace/`$` trick, which is
>   only correct when `\cr` is scanned by a **real** `\halign`. Digested by
>   LaTeXML it re-opens an inline-math frame that the alignment's column-*after*
>   template can no longer balance → `Attempt to close a group that switched to
>   mode math` → runaway.
>
> Perl already retracts the sibling kernel macro
> (`latex_constructs.pool.ltxml:3612`, `Let('\@tabularcr','\lx@alignment@newline')`);
> it simply never needed the `\@arraycr` half. Rust does. `\lx@alignment@newline`
> is the faithful model of `\\`-in-an-alignment — it reads the same `*` and
> `[dim]` arguments `\@arraycr`/`\@argarraycr` do — so aliasing the entry point
> retracts the whole chain.
>
> **The decisive experiment** (worth reusing): hand-expanding `\@arraycr` inline
> was *clean* in both engines, while the macro itself failed. That pinned the
> fault to the inherited kernel definition rather than to `$`/brace handling or
> the frame stack.
>
> ## Breadth measured
>
> `\@arraycr` appears in **6 of a 6,000-paper 2605 sample (0.1%)**, three of them
> via the direct `\let\\\@arraycr` / `\def\\{\@arraycr}` idiom. A second
> independent witness recovered outright: **arXiv:2605.05194** went from
> **125 errors + `Fatal:TooManyErrors` and a 39-byte (totally empty) document**
> to **0 errors / 422 KB**. The other two hits are byte-unchanged, confirming the
> change is inert wherever the idiom is not used — as it must be, since no Rust
> binding references `\@arraycr` and no `.ltxml` defines it.
>
> ## What this does NOT fix
>
> The `blkarray` sibling (both engines fail; binding-shadowed 2026-07-18) and
> the wider `_`/`^` and `\lx@end@inline@math` truncation families are separate.
> Do not assume the ~12.1k full-arXiv `\lx@begin@alignment` fatals all collapse
> to this; re-mine the corpus before quoting a number.

---

# ⚠️ EVERYTHING BELOW THIS LINE IS THE PRE-FIX RECORD (2026-07-10)

It is kept for the **reduction map** and the **ruled-out hypotheses**, which are
the reusable part. Its root-cause analysis (`stomach.rs::egroup` frame
accounting) and its "Entry points for whoever resumes" are **superseded and
wrong** — see the banner above. Read it as history, not as a worklist.

---

Witness: **arXiv:2605.23849** (Cluster H in
[`../../performance/STABILITY_WITNESSES.md`](../../performance/STABILITY_WITNESSES.md)).
Surfaced by mining the 2605+2606 60k-doc telemetry as a ~149s digest-runaway →
fatal. It was believed to be one instance of the broader
**`\lx@begin@alignment` family** (the full-arXiv corpus showed ~12.1k
`\lx@begin@alignment` fatals) — that attribution was never verified and the
actual root turned out to be narrower.

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

The error is raised in `stomach.rs::egroup` (the branch at ~L471, "Attempt to
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
takes it up: re-apply, run the full suite (1617/0 as of 2026-07-20) + an isolated before/after
byte-diff on a corpus sample, and ship it on its own merits — separately from this
crash.

## Cross-references

- [`../../performance/STABILITY_WITNESSES.md`](../../performance/STABILITY_WITNESSES.md) — Cluster H (the four digest-runaway witnesses; this is #1).
- [`../../SYNC_STATUS.md`](../../SYNC_STATUS.md) — Beyond-Perl levers section (BP-4 retired; Cluster H reclassified as Target-1 parity loop bugs).
- Full-arXiv `\lx@begin@alignment` 12.1k cluster (memory `full-arxiv-corpus-reference-2026-06-30`).

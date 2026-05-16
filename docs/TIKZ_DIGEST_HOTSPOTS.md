# TikZ / pgfplots digest hotspots — 2026-05-16

Profiling study of where wall time is spent during digestion of
TikZ/pgfplots-heavy papers. Driven by Win #5 of the 2026-05-16
performance sprint (see `docs/PERFORMANCE.md`). **This document is
research-only**; no engine edits land here. The findings are a punch
list for a follow-up macro-engine sweep.

## Witnesses

| Paper | Digest (s) | Note |
|---|---:|---|
| `2103.00971` | 8.8 | matlab2tikz figure: 12 000-line `exp3.tex` of `\addplot table {…}` blocks with thousands of numeric rows |
| `2208.10851` | 1.5 | local `ieeeconf.cls` (198 KB); driver for the profile below |
| `2406.11624` | 9.4 | 19 `tikzpicture` + 40 `pgfplots` axes |

## Method

`perf record` requires `perf_event_paranoid<2`, which the dev box
locks to `4` (no sudo). Substituted **`valgrind --tool=callgrind`** on
`2208.10851` (the smaller witness; the matlab2tikz one would take 4+ h
under callgrind's 10–50× overhead). Top frames decoded against the
`--profile bench` binary via `addr2line`.

```bash
cargo build --profile bench --bin cortex_worker --features cortex
LATEXML_GRAPHICS_CACHE_OFF=1 valgrind --tool=callgrind \
  --callgrind-out-file=/tmp/callgrind.out \
  ./target/release/cortex_worker --standalone \
    --input ~/round22_validate/inputs_perl_timeout/2208.10851.zip \
    --output /tmp/cg_test.zip
callgrind_annotate /tmp/callgrind.out | grep '???:' | head -60
# (paired with addr2line to resolve binary addresses, since
#  callgrind doesn't auto-load DWARF symbols for our binary)
```

The annotated profile is reproducible from `/tmp/callgrind.out` and
the script in `/tmp/bucket_hot.py` (also archived inline below).

## Top-60 instruction-share buckets (44.0% of total Ir)

| % | Bucket |
|---:|---|
| **7.28%** | `latexml_engine::latex_constructs::load_definitions::{closure#NNN}` — raw-TeX macro bodies |
| **4.95%** | `Option<Cow<str>>::as_ref` — state lookups |
| **3.41%** | `Vec<Token>` alloc/build (`spec_from_iter_nested::from_iter`) |
| **3.38%** | unresolved address (likely inlined hot leaf) |
| **3.20%** | `RawVecInner::non_null::<(state::TableName, SymbolU32, Stored)>` — state-map entry alloc |
| **2.78%** | `<Tokens as fmt::{Debug,Display}>::fmt` |
| **2.78%** | libxml ops (`<Result<Option<Node>, …>>::branch` etc.) |
| 1.65% | `Vec<u8>::len` |
| 1.54% | `Vec<u8>::append_elements` |
| 1.43% | `Mouth::open` / `Mouth::open_file` |
| 1.19% | `Arc<InnerReadDir>::drop` (find-file walks) |
| 0.90% | btree ops |
| 0.88% | `common::cleaners::roman_aux` |
| 0.81% | `stomach::end_mode_opt` |
| 0.78% | arena interner |
| 0.77% | rewrite stage |
| 0.72% | `memcpy` / `copy_nonoverlapping` |
| rest | scattered (each ≤0.5%) |

## What the buckets mean

### 1. State lookups — **partially retracted** (`Option<Cow<str>>` not the leaf)

The initial bucket attributed 4.95% to `Option<Cow<str>>::as_ref`
and proposed converting `lookup_value`/`lookup_string`/`lookup_int`
to return `Option<SymStr>` directly. Runtime verification (a counter
on `Font::merge_ref`, the largest single consumer of those
`Option<Cow<str>>` fields) shows:

| Paper | Wall | `Font::merge_ref` calls |
|---|---:|---:|
| 2208.10851 | 2.13 s | **6,013** |
| 2112.10748 | 17.73 s | **108,917** |
| 2103.00971 | 9.73 s | **6,889** |

For 2112.10748 (the only paper where Font::merge fires often)
that's 109k calls × ~6 `Option<Cow<str>>::as_ref` internal calls
≈ 650k `.as_ref()` invocations. At ~2 ns each that's ~1.3 ms
total, **NOT 4.95% of 18 seconds**. Same thin-LTO phantom as
the Tokens::fmt bucket (§3 below).

**No action item from this sub-bucket.** State lookups still take
real time — but the cost is in the lookup body, not in the
`Cow::Borrowed` wrapper. A Cow→SymStr refactor would not move
the needle.

The companion bucket — `RawVecInner::non_null` for the state map's
`(TableName, SymbolU32, Stored)` triple at 3.20% — is **real**.
Counter on `State::assign_internal` (the actual map mutation entry
point):

| Paper | Wall | `assign_internal` calls | Calls/s |
|---|---:|---:|---:|
| 2208.10851 | 2.13 s | 1,417,498 | ~665 k/s |
| 2112.10748 | 17.73 s | 555,308 | ~31 k/s |
| 2103.00971 | 9.73 s | **8,802,754** | **~905 k/s** |

The TikZ paper (`2103.00971`) writes **8.8 million** state entries
in 9.7 s. Even at 50 ns per entry (HashMap insert + alloc) that's
~440 ms / 4.5% of wall — credible.

**Action item**: A small-vec backing for the per-key chain (the
`VecDeque<Stored>` stored as the map value) would skip the heap
allocation when a key has ≤4 generations of stacked values, which
is the overwhelming common case (counter values, font frames,
keyword toggles). The HashMap itself is fine; the per-entry inner
collection is the hot allocator.

### 2. Token-stream allocation — **REAL** (verified at 26.3M calls on TikZ)

`<Vec<Token> as SpecFromIterNested<…>>::from_iter` — building new
token vectors from iterators. Counter on `Tokens::new` (the
wrapper that every `.collect::<Tokens>()` and every direct Vec
construction passes through), plus a per-call size histogram:

| Paper | Wall | `Tokens::new` calls | Calls/s |
|---|---:|---:|---:|
| 2208.10851 | 2.13 s | 3,911,588 | ~1.8 M/s |
| 2112.10748 | 17.73 s | 1,055,520 | ~60 k/s |
| 2103.00971 | 9.73 s | **26,314,781** | **~2.7 M/s** |

Per-call size distribution on the TikZ paper `2103.00971`
(percentages of 26.3 M total):

| Size | Count | Share |
|---:|---:|---:|
| 0 (empty) | 1,469,918 | 6 % |
| 1 token | **14,495,920** | **55 %** |
| 2–4 tokens | 2,762,931 | 10 % |
| 5–8 tokens | 945,272 | 4 % |
| 9–16 tokens | 2,364,592 | 9 % |
| 17–32 tokens | 2,270,439 | 9 % |
| 33+ tokens | 2,005,709 | 8 % |

**~70 % of all Tokens are ≤4 tokens long.** A `SmallVec<[Token;
4]>` backing would skip the heap on ~70 % of allocations
(empty + 1-token + 2-4 buckets); a `SmallVec<[Token; 8]>` would
cover ~75 %.

**Action item**: change `pub struct Tokens(Vec<Token>);` to
`pub struct Tokens(SmallVec<[Token; 4]>);` (or `[Token; 8]`). The
public API of `Tokens` already abstracts over the inner storage;
the change is contained behind the existing `.unlist*()` methods.
Risk: `Token` is `Copy` and small (16 bytes per current inspection,
let me confirm), so `SmallVec<[Token; 4]>` is 64 + tag bytes inline
— acceptable footprint. The 33+ bucket (8 % of allocs, 2 M calls
on this paper) still pays for a heap alloc, which is correct.

### 3. Tokens fmt — **retracted** (addr2line under thin-LTO mis-attribution)

The initial bucketing attributed ~2.78% of instructions to
`<Tokens as fmt::{Debug,Display}>::fmt`. A follow-up audit
(2026-05-16) instrumented `Tokens::Debug::fmt` with a per-call
counter and a sampling backtrace, then re-ran on four heavy papers
(2208.10851, 2112.10748, 2103.00971, 2406.11624). **The counter
reported zero calls on every paper.**

What the original 2.78% actually was: `addr2line` sometimes
labels an instruction address with the *nearest preceding* symbol,
and under `lto = "thin"` the release binary inlines aggressively
enough that the byte ranges originally owned by `Tokens::*::fmt`
end up filled with unrelated code (state-lookup `Option<Cow<str>>`
plumbing, `Vec<Token>` builds, raw-TeX macro bodies). The bucketer's
substring match `'Tokens' in sym and 'fmt' in sym` happily collected
those phantom hits.

**Lesson learned (worth recording in the audit playbook):** when
callgrind+addr2line surfaces a fmt::{Debug,Display} symbol as hot
under thin-LTO, **always verify with a runtime call counter
before designing fixes around it**. Phantom symbols are most likely
when the suspected hot function is small, the surrounding code is
heavily monomorphised, and the LTO mode is `thin` (the boundary
between codegen units is preserved but inlining still moves code
across symbol ranges).

**No action item from this bucket.** The 2.78% is genuine hot code,
but it's attributed to the wrong function — the actual costs land
in the other buckets above (state lookups, token-vec alloc, raw-TeX
closures).

### 4. `latex_constructs` closures (7.28%)

The `{closure#NNN}` symbols are the auto-numbered macro-expansion
closures built by `setup_binding_language` in `latex_constructs.rs`.
The hot-list shows multiple distinct closure IDs (`#79`, `#236`,
`#268`, `#295`, `#877`) each at 0.3–2.3% individually — i.e. the
macro engine is genuinely walking a lot of macros, not stuck in
one.

**No grammar/binding edit obviously helps here** — these are doing
the work the user asked for. But the closures all share the same
state-lookup + token-vec patterns called out above; a 2× speedup on
those primitives lifts this whole bucket too.

### 5. Mouth I/O (1.43% + 1.19% drop)

Each TeX `\input` opens a file via `Mouth::open` and the
`InnerReadDir` find-file walk. For papers with 30+ `.tex`/`.sty`
files this adds up. The `Arc<InnerReadDir>::drop` cost suggests we
clone the readdir iterator more than necessary.

**Proposed**: cache the per-`\input`-name resolved path so the
find-file walk runs once per name, not once per `Mouth::open`. The
existing `find_file` already has the `binding_available` flag fast
path; extending it to a per-paper resolved-path cache should be
cheap.

### 6. `roman_aux` (0.88%)

`common::cleaners::roman_aux::<i64>` — Perl `Util::Cleaners`-style
roman-numeral renderer. 0.88% is a lot for a function that only
fires on `\roman{ctr}` invocations. Likely called via list-counter
secondary forms ("page i, ii, iii") that pgfplots' axis labels
inadvertently trigger.

## Aggregate (verified 2026-05-16)

Two of the original four leaf-level buckets were thin-LTO phantoms
(Tokens fmt, Option<Cow<str>>::as_ref — see §3 and §1 above). The
remaining two are real and concentrated on TikZ-style papers:

| Bucket | Verification | Hottest paper | Calls (s⁻¹) | Plausible fix |
|---|---|---|---:|---|
| Tokens::new (Vec<Token> alloc) | 26.3 M counted | 2103.00971 | ~2.7 M/s | `SmallVec<[Token; 4]>` |
| assign_internal (state-map entry) | 8.8 M counted | 2103.00971 | ~905 k/s | small-vec backing on per-key chain |

SmallVec backing for `Tokens` is the highest-confidence win: ~70 %
of `Tokens::new` calls allocate ≤4 tokens; making the inline path
heap-free would eliminate ~18 M of the 26 M heap allocations on
2103.00971. At ~50 ns per alloc that's ~0.9 s — roughly **10 % of
digest wall on the matlab2tikz paper**, and proportional smaller
gains on other macro-heavy papers.

Both fixes are real refactors (touch `Tokens` field type or
`State` per-key VecDeque), not one-liners. They are worth a sprint
but won't change the dominant story for TikZ papers: matlab2tikz
output is *genuinely large* (12 000 numeric rows × per-row macro
expansion); the only way to halve digest on it is to bypass the
pgfplots `\addplot table {...}` expansion in Rust, which is a much
larger project.

## Nuclear option: pgfplots `\addplot table {...}` Rust bypass

The numeric-row body of an `\addplot table {x_sep=tab,row sep=crcr]{
x1 y1\\ x2 y2\\ … }` block is purely numeric data with no LaTeX
catcodes worth honoring — it could be parsed in Rust as a CSV-like
table and emitted as a single `<ltx:tabular>` node, skipping the
gullet entirely.

* **Win**: ~95% of digest for matlab2tikz-class papers
  (`2103.00971` would drop from 8.8s to ~0.5s).
* **Cost**: a dedicated Rust parser for the pgfplots table syntax,
  plus a binding override that intercepts `\addplot table` before
  it reaches the raw pgfplots.code.tex chain. Risk: pgfplots admits
  many variants (`row sep`, `col sep`, custom delimiters, `point
  meta`, math expressions in cells).
* **Scope**: 2–3 days of engineering; needs its own design doc.

## Reproducing this audit

```bash
# Build with debug info preserved (this is a release-optimization build).
cargo build --profile bench --bin cortex_worker --features cortex
# Capture instruction-level profile.
LATEXML_GRAPHICS_CACHE_OFF=1 valgrind --tool=callgrind \
  --callgrind-out-file=/tmp/callgrind.out --dump-instr=no \
  ./target/release/cortex_worker --standalone \
    --input ~/round22_validate/inputs_perl_timeout/2208.10851.zip \
    --output /tmp/cg.zip
# Decode the top frames.
python3 docs/scripts/bucket_callgrind_hot.py /tmp/callgrind.out \
  ./target/release/cortex_worker
```

A reusable copy of the bucketing script lives at
`docs/scripts/bucket_callgrind_hot.py` (see commit).

## Next steps (handoff)

1. Investigate `Tokens::Debug` hot path — the lazy-eval candidate is
   the highest-confidence quick win.
2. Convert `lookup_value` / `lookup_string` /  `lookup_int` to
   return `Option<SymStr>` where the caller permits, eliminating
   the `Cow::Borrowed` wrapper.
3. SmallVec-back `Tokens` for the ≤16-token-body majority case
   (instrument first to confirm the distribution).
4. Scope-design the pgfplots `\addplot table` Rust bypass.

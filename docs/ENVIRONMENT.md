# Environment variables

Every environment variable that latexml-oxide reads or sets, with the values it accepts, its default and its effect.
A command-line flag always wins over the variable of the same meaning. Most variables are read once, when a
conversion starts.

**Audience legend**

| Mark | Who it is for |
|---|---|
| **user** | anyone running `latexml_oxide` from the command line |
| **library** | Rust code embedding the `latexml` crate |
| **packager** | building, packaging or releasing the binaries |
| **fleet** | `cortex_worker` deployments |
| **dev** | diagnosing or measuring the converter; output and speed may change |
| **test** | the test suite only |
| **internal** | set by latexml-oxide for its own child processes; never set these yourself |

**How values are read.** Conventions differ between variables, so each row gives its own:

- *presence*: the variable being set at all switches the feature on, whatever its value. `LATEXML_NODUMP=0` also skips the dumps.
- *`1`*: only the exact value `1` switches it on.
- *`1|true|yes`*: one of those three values, case-sensitive.
- *`0` disables*: the feature is on unless the value is exactly `0`.
- A number that does not parse falls back to the default, except `LATEXML_MAX_MEMORY`, which is a command-line error.

## Running conversions (user)

### Memory and limits

| Variable | Values | Default | Effect |
|---|---|---|---|
| `LATEXML_MAX_MEMORY` | MiB; `0` = no ceiling | 90 % of available RAM, at most 64 GiB, at least 2048 MiB | `latexml_oxide` only (not `cortex_worker`). The one memory knob of `latexml_oxide`, the same as `--max-memory`. The watchdog stops the run above it; the cooperative fuse sits at 3/4 of it; streaming spills at a third of the fuse. |
| `LATEXML_STREAMING` | `true` / `false` | unset = automatic | `latexml_oxide` only (not `cortex_worker`). The same as `--streaming`. `true` streams from the start; `false` never streams. `1` is rejected. |
| `LATEXML_TOKEN_LIMIT` | tokens; `0` disables | 20,000,000,000, raised to 1000 × the source size | Runaway-expansion backstop (a Fatal). `--token-limit` wins. Setting either at all turns off the scaling with source size. |
| `LATEXML_EXPAND_DEPTH_LIMIT` | integer; `0` disables | 12,000 | Nested expansion depth before `Fatal:Timeout:Recursion`. |
| `LATEXML_MAX_GRAMMAR_LEXEMES` | integer; `0` or `none` disables | 12,000 | Formulas longer than this skip the grammar parser for the simpler fallback parser. |

**Under an address-space limit** (`ulimit -v`, as some batch systems impose), the allocators' reservations count
against the limit even though they use no memory. Two variables of the allocators themselves, not ours, cut that
reservation by about half without changing memory use, speed or output:

| Variable | Recommended | Effect |
|---|---|---|
| `MALLOC_ARENA_MAX` | `2` | glibc: two malloc arenas instead of one per thread (each reserves 64-128 MiB). |
| `MIMALLOC_ARENA_RESERVE` | `128MiB` | mimalloc: reserve 128 MiB at a time instead of 1 GiB. |

Without an address-space limit neither is needed. The fleet sets both for its children (see below).

### Format dumps

| Variable | Values | Default | Effect |
|---|---|---|---|
| `LATEXML_FORMAT_CACHE` | directory | `$XDG_CACHE_HOME/latexml-oxide/formats`, else `~/.cache/latexml-oxide/formats`, else the temp directory | Where the format built for your TeX tree is cached. It is built once, on the first conversion with a tree the release's embedded dumps were not built from. |
| `LATEXML_DUMP_DIR` | directory | unset | Also look here for `plain.YYYY.dump.txt` / `latex.YYYY.dump.txt`. |
| `LATEXML_DUMP_PATH` | file | unset | An explicit LaTeX dump, used when it matches this TeX tree. |
| `LATEXML_PLAIN_DUMP_PATH` | file | unset | An explicit plain TeX dump. |
| `LATEXML_NODUMP` | presence | unset | Load no dump; build the formats from the engine's definitions on every run (slow). |
| `LATEXML_SKIP_DUMP_STAMP_CHECK` | presence | unset | Silence the TeX Live year-mismatch warning for the LaTeX dump. |

### Post-processing, split and render

| Variable | Values | Default | Effect |
|---|---|---|---|
| `LATEXML_RENDER_JOBS` | integer ≥ 1 | `latexml_oxide`: min(cores, 8); the library and `cortex_worker`: 1 | Worker processes that render the pages of a split run with 64 or more pages. `1` renders serially. |
| `LATEXML_POST_STREAM_SPLIT` | `0` = off, any other value = forced on | automatic for `--splitat` runs | The streaming split, which splits from the core result on disk instead of one whole document in memory. |

### Graphics

| Variable | Values | Default | Effect |
|---|---|---|---|
| `LATEXML_GRAPHICS_CACHE_DIR` | directory; empty is ignored | `$XDG_CACHE_HOME/latexml-oxide/graphics`, else `$HOME/.cache/…`; neither set = no cache | The shared cache of converted images. |
| `LATEXML_GRAPHICS_CACHE_OFF` | `1\|true\|yes` | cache on | Convert every image afresh. |
| `LATEXML_GRAPHICS_CACHE_MAX_MB` | MiB; `0` = never prune | 2048 | The cache's size; the least recently used entries go first. |
| `LATEXML_GRAPHICS_VECTOR_AUTO_OFF` | `1\|true\|yes` | on | Turn off the automatic PDF-to-SVG conversion of small vector figures (see `--graphics-svg-threshold-kb`). |
| `LATEXML_RASTER_DENSITY` | dpi, 50-600 | 120 (the longest side capped at 2048 px) | Resolution of rasterized figures. |
| `LATEXML_CONVERT_TIMEOUT_SECS` | seconds > 0 | 60 | Time limit for one ImageMagick or Ghostscript conversion. |
| `LATEXML_SVG_CONVERT_TIMEOUT_SECS` | seconds > 0 | 15 | Time limit for one mutool or pdftocairo SVG conversion. |

### Output, logging and safety

| Variable | Values | Default | Effect |
|---|---|---|---|
| `LATEXML_TELEMETRY_OUT` | file | none | `latexml_oxide` only. Append one JSON line of timings and counts per conversion; the same as `--telemetry-out`. |
| `LATEXML_SOURCE_MAP` | presence | off | `latexml_oxide` only. The same as `--source-map`: `data-sourcepos` attributes in the output. |
| `LATEXML_DISABLE_SHELL_ESCAPE` | presence | allowed | Rhai bindings may not run external commands (builds with runtime bindings). `cortex_worker` loads no runtime binding at all, so it needs no such setting. |
| `SOURCE_DATE_EPOCH` | Unix seconds | the clock | Fixes `\today`, `\time` and the log banner's time (UTC), and the member times of `--format=zip`/EPUB output, for reproducible builds. |
| `NO_COLOR` | presence | colour on a terminal | No colour codes on stderr. |
| `TMPDIR` (`TMP`/`TEMP` on Windows) | directory | the system default | Scratch files and temporary caches. |

### TeX Live

latexml-oxide finds packages, classes and fonts through kpathsea, so the usual TeX Live variables apply:
`TEXMFROOT`, `TEXMFDIST`, `TEXMFMAIN`, `TEXMFLOCAL`, `TEXMFHOME`, `TEXMFVAR`, `TEXMFCONFIG`, `TEXMFSYSVAR`,
`TEXMFSYSCONFIG`, `TEXMFCNF`, `TEXINPUTS`, `BIBINPUTS` and the other `*INPUTS` paths. A trailing `/` on a tree root is
trimmed and logged. `KPSEWHICH` names the `kpsewhich` program that anchors kpathsea; `PATH` also finds the external
tools (kpsewhich, ImageMagick, Ghostscript, mutool, pdftocairo, texlua).

## Embedding the library (library)

| Variable | Values | Default | Effect |
|---|---|---|---|
| `LATEXML_RSS_CAP_BYTES` | bytes; `0` = off | 4,500,000,000 | The cooperative memory fuse (a graceful Fatal). The `latexml_oxide` binary sets its own from `--max-memory`, `cortex_worker` from `--max-rss-mb`; both ignore this. |
| `LATEXML_NO_DEFAULT_PUSHBACK_LIMIT` | presence | a 5,000,000-token cap | Remove the pushback cap. |
| `LATEXML_STACK_RED_ZONE_BYTES` | bytes | 262,144 | Remaining stack below which a new stack segment is allocated. |
| `LATEXML_STACK_SEGMENT_BYTES` | bytes | 8,388,608 | The size of each new stack segment. |

## Building and packaging (packager)

| Variable | Values | Default | Effect |
|---|---|---|---|
| `LATEXML_GIT_SHA` | string | `git rev-parse --short HEAD`, else `unknown` | The revision printed by `--version` and the log banner. |
| `LATEXML_GIT_SHA_OVERRIDE` | string | the git short sha | The revision recorded in telemetry. |
| `LATEXML_NO_EMBEDDED_DUMP` | presence | unset | Run time: do not fall back to the dumps compiled into the binary. |
| `LATEXML_DUMP_DIR_ONLY` | presence | unset | Run time: use only the dumps named by `LATEXML_DUMP_PATH` / `LATEXML_DUMP_DIR` or built for the tree. |
| `KPATHSEA_SKIP_TOOLCHAIN_CHECK` | `1` | set in `.cargo/config.toml` | Build without a TeX installation. |
| `PKG_CONFIG_PATH`, `LIBCLANG_PATH`, `VCPKG_ROOT`, `KPATHSEA_LIB_DIR`, `KPATHSEA_STATIC`, `LIBXML2_STATIC`, `LIBXSLT_STATIC` | | | Native library discovery and static linking; see BUILDING.md. |

`tools/make_formats.sh` (and `.ps1`) reads `PROFILE` (the cargo profile, default `debug`), `CARGO_TARGET_DIR`,
`CARGO_LOCKED=1` (build `--locked`) and `LATEXML_DUMP_DIR` (copy the built dumps there). `tools/make_release.sh`
reads `GITHUB_REF_NAME`, `RELEASE_TARGET`, `RELEASE_EXTRA_FEATURES` and `RELEASE_LOCKED`. The other `tools/` scripts
document their variables in their headers.

## The cortex worker fleet (fleet)

`cortex_worker` is configured by its command-line options. For each child process it starts it sets:

| Variable | Value | Why |
|---|---|---|
| `TMPDIR`, `MAGICK_TEMPORARY_PATH`, `MAGICK_TMPDIR` | a fresh directory per worker slot | Scratch space emptied between spawns. |
| `MALLOC_ARENA_MAX`, `MIMALLOC_ARENA_RESERVE` | `2`, `128MiB` | The children run under an address-space limit (`--child-mem-limit-mb`); see "Under an address-space limit". |
| `MALLOC_CONF`, `_RJEM_MALLOC_CONF` | `background_thread:true,dirty_decay_ms:500,muzzy_decay_ms:0` | Prompt return of freed memory in jemalloc builds. |
| `LATEXML_RSS_CAP_BYTES` | `--child-mem-limit-mb` − 256 MiB, else (`--child-mem-limit-mb=0`) `--max-rss-mb` | The per-child memory fuse: the child's `--max-rss-mb`, which its conversions set it from. |

These replace any value of the same name in the worker's own environment. The worker loads no runtime `.rhai`
binding in any build, nor do the children it spawns (each re-executes its `main`). The worker itself reads:

| Variable | Values | Default | Effect |
|---|---|---|---|
| `CORTEX_WORKER_THROTTLE_SECS` | seconds | 60 | How long an idle worker sleeps before asking the dispatcher for work again. |
| `CORTEX_WORKER_IO_TIMEOUT_SECS` | seconds > 0 | 120 | How long a worker waits on a dead or wedged dispatcher before giving up that exchange. |
| `CORTEX_HARNESS_RAM_HEADROOM_PCT` | percent, 10-100 | 85 | pericortex's worker-count clamp to RAM; `cortex_worker` never calls it, so it has no effect there (`--workers` and the memory governor size the fleet). |

The Docker entrypoint reads `WORKERS` and
`PROFILE` (`ar5iv` or `generic`).

## Diagnosing and measuring (dev)

These switch on traces, audits or alternative code paths. They are for developers; output and speed may change.

| Variable | Values | Effect |
|---|---|---|
| `LATEXML_DEBUG_FATAL` | presence | A backtrace, the pushback and the last tokens at the first Fatal. |
| `LATEXML_DEBUG_MEMBUDGET` | presence | A backtrace and box statistics when the memory fuse trips. |
| `LATEXML_CSNAME_TRACE`, `LATEXML_EXPL_TRACE`, `LATEXML_PGFKEYS_TRACE`, `LATEXML_CMML_TRACE_CYCLE` | presence | Traces of `\csname` errors, expl3 syntax state at file exit, pgfkeys dispatch, Content MathML cycles. |
| `LXML_TRACE_BOUND_MODE`, `LXML_TRACE_FRAMES`, `LXML_TRACE_TERMINAL`, `LXML_TRACE_GROUP_END`, `LXML_TRACE_ALIGN_STATE`, `LXML_SIZE_TRACE`, `LXML_TRACE_NODE_BOXES`, `LXML_TRACE_FRAGMENT_RC`, `LXML_TRACE_INSERT_BLOCK` | presence | Engine traces: modes, frames, group ends, alignments, box sizes, streaming spills, fragment references, block insertion. |
| `LXML_TRACE_ARGS` | a control sequence name | Print the arguments that definition reads. |
| `LATEXML_REWRITE_TIMING`, `LATEXML_POST_AUDIT`, `LATEXML_POST_MEMDIAG` | presence | Timings of rewrite rules and post-processing stages; heap statistics every 1024 pages (Linux). |
| `LATEXML_PARSE_LEXEMES`, `LATEXML_PARSE_AUDIT`, `LATEXML_PARSE_PRUNE_REASONS`, `LATEXML_MATH_AMBIGUITY_AUDIT`, `LATEXML_MARPA_HYBRID_AUDIT_PARITY`, `LATEXML_MARPA_ASF_AUDIT`, `LATEXML_PARSE_DUMP_ORDER` | presence | Math parser audits and dumps. |
| `LATEXML_MARPA_LEGACY`, `LATEXML_MARPA_ASF_ONLY` | presence | Force one math parse strategy (tree iteration or ASF). |
| `LATEXML_MARPA_HYBRID_AND_NODE_LIMIT` | integer; `0`/`none` = no cap (500) | Bocage size above which the parser falls back to tree iteration. |
| `LATEXML_L3FP_NATIVE`, `LATEXML_PGFKEYS_NATIVE`, `LATEXML_DATATOOL_NATIVE` | `0` disables | Turn a native fast path off, to compare with the macro path. |
| `LATEXML_L3FP_TRACE`, `LATEXML_L3FP_CHECK` | `1` | Count the native l3fp calls; check them against the macro results. |
| `LATEXML_NO_KPATHSEA_PREWARM` | presence | Skip the kpathsea prewarm thread. |
| `LATEXML_SPILL_AT_MIB`, `LATEXML_SEGMENT_CHUNK_MIB`, `LATEXML_SOFT_YIELD_MIN_BOXES`, `LXML_NODE_BOXES_SWEEP` | integers | Streaming watermark, segment size and yield points. |
| `LATEXML_CHECK_READ_BUDGET`, `LATEXML_REHEARSAL_READ_BUDGET` | tokens; `0` = unlimited | Budgets of the frontmatter's speculative reads (8,000,000). |
| `LATEXML_POST_STREAM_THRESHOLD`, `LATEXML_POST_MEM_PARSE_LIMIT`, `LATEXML_POST_SPILL_PAGE` | bytes / `1` | Streaming-split threshold, in-memory parse ceiling, staging an unsplit page on disk. |
| `LATEXML_INIT_DEBUG` | presence | Keep and count the errors of an `--init` format build. |
| `LATEXML_DUMP` | file | Load an extra dump after the preloads. |

## Tests (test)

`LATEXML_BLESS`, `LATEXML_SAVE_ACTUAL`, `LATEXML_TEST_LOG`, `LATEXML_SIGSEGV_TRACE`, `LATEXML_TEST_TEXLIVE`,
`LATEXML_TEST_SKIP_LOG`, `LATEXML_SVG_TESTS`, `LATEXML_BROWSER_TESTS`, `LATEXML_EPUBCHECK_TESTS`, `LATEXML_FIREFOX`,
`LATEXML_BROWSER_ENGINE`, `LATEXML_FIREFOX_PATH`, `LATEXML_INJECT_PANIC`, `LATEXML_RENDER_TEST_SERIAL_STOP`,
`LATEXML_RENDER_WORKER_TEST_ABORT`, `CORTEX_WORKER_TEST_FAULT`, `CORTEX_WORKER_TEST_GROW_MB`, `UPDATE_SUPPORTED_PACKAGES`,
`UPDATE_API_DOC` and `LATEXML_PERL_LIB` drive the test suite: blessing goldens, requiring optional tools (a missing
mutool, browser or epubcheck becomes a failure instead of a skip), and fault injection. BUILDING.md describes the
ones a contributor uses.

## Internal (do not set)

| Variable | Set by | Effect if set by hand |
|---|---|---|
| `LATEXML_INI_MODE` | `latexml_oxide --init` | A normal conversion stops after the bootstrap pools. |
| `LATEXML_RENDER_WORKER` | the render fleet, for its workers | `latexml_oxide` ignores its command line and waits as a render worker. |

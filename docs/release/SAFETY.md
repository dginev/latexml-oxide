# Safety Profile and `unsafe` Inventory

## Threat model

`latexml_oxide` is a **batch document compiler**, not a network service or a multi-tenant process. It runs from a
shell or a CI runner on LaTeX input the user chose to compile: arXiv-style preprints, their own sources, test
fixtures.

- **No untrusted-network surface.** Nothing is parsed from a socket, and no `unsafe` block touches a network-derived
  buffer.
- **No privilege boundary.** The process holds the invoking user's UID and drops no privileges; nothing crosses a
  sudo, setuid or capability boundary.
- **One conversion per thread.** `cortex_worker` runs a pool of workers, but each conversion runs on exactly one OS
  thread: the engine state is a thread-local global (CLAUDE.md design decision 0.3.2).
- **Memory-safe by default.** The only code that is not Rust is linked C (libxml2/libxslt through the `libxml` and
  `libxslt` crates, libkpathsea, libmarpa, mimalloc) and the external programs a conversion runs: TeX Live's
  `texlua` (document Lua), `kpsewhich` and `pdflatex` (queries of the TeX tree), and the image converters
  Ghostscript (`gs`, `ps2pdf`), MuPDF (`mutool`), Poppler (`pdftocairo`) and ImageMagick (`convert`/`magick`). The
  image converters run under a timeout in their own process group (`Graphics::run_with_timeout` in
  `latexml_post/src/graphics.rs`); `texlua` is sandboxed as the next section describes.

The output is HTML, MathML or XML rendered from LaTeX. Its *content* may need sanitising downstream (CSS or
`<script>` injection through an author's `\href` or `\url`), but that is a content-policy concern, not a
memory-safety one. **Document-driven** `\write18` (shell escape) never runs. A **trusted binding** can run external
commands through the `Command` API, allowed by default and blocked by `LATEXML_DISABLE_SHELL_ESCAPE`; see the
runtime-bindings section below.

## Untrusted input stays in the job

- **Document Lua is sandboxed** (OXIDIZED_DESIGN_DIVERGENCES #1081). `\directlua`, `\luaexec` and `{luacode}`
  chunks run in `texlua` under TeX's own defaults for untrusted input, set by the bridge's prelude before any
  document chunk: `shell_escape=f` (no program runs, no C module or network library loads) and `openout_any=p`
  (writes only to a relative path below the job directory, no `..`, no dotfile, no symbolic link on the way; on
  Unix no `\` in the name). A refused call never takes effect. A call lualatex refuses too returns what lualatex
  `-no-shell-escape` returns (`os.execute("…")`: `nil, "All command execution disabled."`; `io.popen`: `nil, "all
  command execution is disabled"`; a forbidden `io.open` write: `nil`; `os.execute()`: 0, no shell) and is a
  `Warning`; one only the sandbox refuses (lualatex would perform it) is an `Error` and a Lua error. A C module, a
  network library and `package.loadlib` are that `Error` too, though lualatex refuses them as well (its
  `luatex-core.lua` empties `package.loadlib` and drops the C searchers without shell escape): no manual needs them,
  so they keep the stricter answer. A chunk that fails after a refusal (it used the refused call's `nil`) is an
  `Error`, as lualatex stops with an error there. The record is reported on a best-effort basis: the
  checks use functions held before any document code runs, and the bridge accepts only frames carrying a nonce the
  interpreter read before any other code loaded, so a refusal record cannot be forged and a chunk's
  `print`/`io.write`/`texio.write` reach the null device; but a chunk can suppress its OWN record (exhausting its C
  stack just before the record is written, or reading its own pipe through `/proc/self/fd`), so the record is
  evidence, not a guarantee — the guarantee is that the call was refused. A chunk lists at
  most 20 refusals of each severity, the rest in one summary per severity. An interpreter that ends or sends anything but a frame
  mid-chunk is an `Error` (its output is lost); the next chunk starts a fresh one, up to 3 times per conversion.
  Each conversion gets its own interpreter, in its source directory, with an environment reduced to the texmf
  search, library path, locale, date and temp-dir variables; a timeout or memory exit kills it. Not covered: a link
  created between the check and the write by another process; and reads, which stay open as under TeX
  (`openin_any=a`), so a chunk can read any file the user can, its own process memory (and so the nonce) included.
  kpathsea's own font generation (`mktextfm` and METAFONT) never runs: the interpreter's environment carries no
  `MKTEX*` variable, and the prelude sets them all to 0 so that kpathsea ignores a `texmf.cnf` that enables them (one
  an operator's `TEXMFCNF` reaches included). This is stricter than lualatex, which may run `mktextfm` for a missing
  TFM: fail-safe by choice.
- **Archive sources unpack inside their directory.** A `.tar`/`.tar.gz` symbolic link is made only when its target,
  resolved through the links already there, stays inside the extraction directory, a hard link only to a file
  inside it; other links, and entries whose path would leave the directory, are skipped with a counted `Warning`.
  `.zip` extraction (`latexml::util::unpack_zip`, used by both `latexml_oxide` and `cortex_worker`) validates every
  entry path with `enclosed_name()`, creates a symlink only when its target resolves inside the extraction directory
  (unpacking escaping symlinks as plain files with a counted `Warning` instead of failing the paper), and normalizes
  permissions across the unpacked tree (`a-x,u+rwX` on Unix; clearing read-only on Windows) so read-only directories
  never leak temp files and archive exec bits do not survive. The search for the main file walks real directories only,
  so a link to a directory above it (`a -> .`) cannot make it cycle; a persistent `cortex_worker` drops the held
  archive `Warning`s of a job that failed before its report. A `00README` main file that is absolute, has a `..` or
  resolves (symbolic links followed) outside the source is ignored with a `Warning:archive:path`, and one that is not a
  file is passed over, for an archive and a local folder alike (OXIDIZED_DESIGN_DIVERGENCES #1166).
- **Fully untrusted sources belong in a container or VM.** TeX itself, and Perl LaTeXML, can read any file the
  invoking user can read, and so can a conversion: run sources you do not trust inside a container or virtual
  machine.
- **Image copies stay under `--dest`.** A graphic whose source-relative path would leave the destination
  directory is copied under a generated `xN` name inside it (Perl `Post/Graphics.pm`'s `pathname_is_contained`
  fallback).
- **Stylesheet copies stay in their folders.** A stylesheet and each `@import` it follows are read only from inside
  the folder they were found in (the archive, the document's folder, a `--path` folder) and written only inside the
  destination (or site) directory, symbolic links followed on both sides; one that would leave is neither read nor
  written, with a `Warning`, an operator's `--css` included. A stylesheet the document itself names by a path outside
  those folders is read where it lies, as a document's `\input` is (OXIDIZED_DESIGN_DIVERGENCES #1166).

## `unsafe` budget

Reconciled 2026-11-24 against the ratchet below: **83 sites** in all, counting every `unsafe` keyword in code. That
is 77 `unsafe {}` blocks, 3 `unsafe impl Send/Sync`, 0 `unsafe fn`, two `unsafe extern "C"` blocks
(`latexml_post/src/xslt.rs`, `latexml_oxide/src/util/test.rs`) and one `#[unsafe(link_section)]` (`util/test.rs`).
Of the 80 blocks and impls, 57 are outside test code (3 of them in the `test-utils`-gated `util/test.rs`) and 23 are
test-only (`latexml_oxide/tests/`, `latexml_post/src/test_env.rs`, `latexml_contrib/src/script_bindings/tests.rs`).

**The count may only fall** (issue #937). `unsafe_sites_ratchet::unsafe_sites_only_fall`
(`latexml_oxide/tests/cluster_package_guards/unsafe_sites_ratchet.rs`, run by CI's workspace nextest) pins the sites
per file, read from the lexed source (comments and literals excluded, `#[cfg(test)]` code counted). A change that
removes sites lowers its file's pin (the failure prints the table to paste); one that adds a site fails it, and
raising a pin is an exception its commit argues. That table is the live per-file index; the categories below say why
each site exists, at file and function level so they survive line drift. To list the sites by hand:

```
rg -n 'unsafe\s*\{|unsafe impl|unsafe fn' latexml_* -g '*.rs'
```

Still `unsafe` because no safe API covers them yet (later phases of #937): `getrusage` (rustix has none), `raise`
(thread-directed; `kill_process(getpid())` is process-directed), `sysconf(_SC_PHYS_PAGES)`, the logger's
`ftruncate`/`write` on a raw fd shared with the signal path, `setsid` in `pre_exec`, the `mmap` probe. Every
production `unsafe` block carries a `// SAFETY:` comment at the call site.

---

### A. `unsafe impl Send/Sync` — single-thread markers (3 sites)

| Site | Marker | Why |
|---|---|---|
| `latexml_core/src/state.rs` | `Send for State` | `State` holds `Rc`/`RefCell`/`libxml::tree::Node`, all `!Send`. It is marked `Send` so the harness can build a `State` on one thread and move it to another before first use. From then on it is pinned to one OS thread by the `#[thread_local]` switchers (`use_main_state` / `use_std_state` / `use_sty_state`); no `State` is read from two threads at once. |
| `latexml_core/src/common/store.rs` | `Send/Sync for Stored` | `Stored` variants embed `Rc<RefCell<…>>` and libxml nodes. The marker exists only to satisfy the `Box<dyn Error + Send + Sync>` bounds of error returns, which reach every embedded type. No code shares a `Stored` across threads. |

**Could we refactor?** Only by replacing every `Rc<RefCell<…>>` in `State` and `Stored` with `Arc<Mutex<…>>`. That
contradicts the single-threaded-per-conversion design, puts a lock on the hot path and adds no safety: the markers
are sound because the runtime upholds the invariant by construction.

### B. Arena `resolve_unchecked` — performance carve-out (1 site)

In `latexml_core/src/common/arena.rs`, the private `buffer_text()` that every lend (`with`, `with2`, `with3`,
`with_many`, `to_string`, `first_bytes`) resolves a buffer symbol through. It uses `string-interner`'s unchecked
`Symbol → &str` lookup on the arena's buffer, under the lend's shared `RefCell` borrow.

The safety invariant is **append-only buffer, never grown under a lend**: every symbol below the overflow base
(`u32::MAX - 2^24`; the page takes the top 2^24 symbols) comes from the buffer's `get_or_intern` of a valid `&str` at its
true offset (string-interner casts the offset `as u32` unchecked, so `push` refuses a string that would end past the base,
before the buffer takes it; issue 1011), symbols are never invalidated between `reset`s, and an
overflow symbol never reaches the unchecked lookup (it resolves to the overflow page's own `Rc<str>`). A string interned
while a lend is live goes to the overflow page instead of the buffer, so the buffer cannot move under a lent `&str`;
`reset` inside any lend, an overflow symbol's included, panics (its `try_borrow_mut` fails). Miri checks the arena tests
under Stacked and Tree Borrows (`tools/miri_check.sh`).

**Why we accept it:** callgrind measured the checked `resolve` (a `from_utf8` validation per call) at ~3 % of the total
instruction count; the arena is the engine's hottest read site, since every token-to-string serialisation goes
through it. **Refactor option:** `arena.resolve(sym).expect("interned")` removes the block at that ~3 % cost.

### C. Arena re-entrant `&mut *ptr` — removed (0 sites)

A re-entrant `with_arena_mut` used to take `&mut *ptr` to the interner while a closure held a `&str` into its buffer,
so a new string could move the buffer under that `&str` (a use-after-free). The lends now hold a shared borrow and an
intern under one goes to the overflow page (category B). Guard:
`latexml_core::common::arena::tests::lend_survives_interning_many_new_strings` (it fails under Miri on the old code).

### D. Process, memory and environment FFI

| Sites | Calls | Why |
|---|---|---|
| `latexml_oxide/bin/latexml_oxide.rs` (2) | `std::env::set_var("LATEXML_INI_MODE", …)`; `libc::getrusage(RUSAGE_CHILDREN, …)` | `set_var` is `unsafe` in Rust 2024; it runs before any other thread reads the environment. `getrusage` captures child CPU time; `rustix` has no wrapper. |
| `latexml_oxide/bin/cortex_worker.rs` (4) | `mmap`, the raw allocator, `raise`, `getrusage` | Child CPU time, and the harness tests' injected faults (an address-space reservation in debug builds, allocator exhaustion, a `SIGSEGV`). |
| `latexml_oxide/bin/cortex_worker/exit_report.rs` (4) | `getpid`, the signal recorders | An async-signal-safe crash and exit reporter. |
| `latexml_core/src/watchdog.rs` (18) | RSS, RAM and disk queries | Portable resource queries for the `--timeout` / `--max-memory` watchdog. |
| `latexml_oxide/src/{core_interface,heap_trim,post,streaming_restart}.rs` (7) | `malloc_trim`, `mi_collect`, `mallinfo2` | Heap trims between phases (the cadenced trims share `heap_trim::release_free_heap`) and heap measurements. |
| `latexml_core/src/util/logger.rs` (2) | `ftruncate`, `write` | The log mirror on a raw fd shared with the signal path. |
| `latexml_core/src/util/pathname.rs` (1) | `set_var("TEXMFROOT", …)` | Anchors the linked libkpathsea to the ambient TeX Live tree before it initialises. |
| `latexml_oxide/src/identity.rs` (2), `latexml_oxide/src/util/test.rs` (5) | test `set_var`/`remove_var`; `signal`, `transmute(SIG_DFL)`, `raise`, `set_var("LATEXML_RSS_CAP_BYTES", …)`, an `extern "C"` block, a `link_section` | **Test-only** (`util/test.rs` is `test-utils`-gated): environment setup, and a SIGSEGV/SIGBUS/SIGABRT trap that writes a backtrace to a per-pid file before the signal ends the test binary. |

None can be removed without losing its function until a safe API exists (the list above the categories).

### E. Subprocess process groups, and the test environment (5 sites)

Image converters (`gs`, `pdftocairo`, `mutool draw`, `ps2pdf`, `convert`) can spawn grandchildren, which a plain
`Child::kill()` does not reach, so each runs as a process-group leader.

| Sites | Call | Why |
|---|---|---|
| `latexml_post/src/graphics.rs` (1) | `cmd.pre_exec(|| setsid(); setpgid(0,0); prctl(…))` | Runs between `fork()` and `exec()`, so it must be async-signal-safe. `setsid(2)` makes the child a group leader so the whole group can be killed; the signalling itself is safe `rustix::process` (`watchdog.rs`). `Command::process_group` gives no `setsid` + `killpg`. |
| `latexml_post/src/test_env.rs` (3), `latexml_post/src/graphics_cache.rs` (1) | `set_var`/`remove_var` in `EnvGuard`; one `OnceLock`-initialised test `set_var` | **Test-only.** Both hold the binary-wide `env_lock` across set, use and restore. The residual, documented in `test_env.rs`: a linked C library can call `getenv` concurrently, which no Rust lock serialises, so no correctness rests on these variables (the graphics cache takes an explicit `CachePolicy`). |

### F. libxslt global configuration (3 sites)

`latexml_post/src/xslt.rs`: a `Once`-guarded write to libxslt's process-global `xsltMaxDepth` recursion cap (1000, as
Perl's `XML::LibXSLT->max_depth(1000)`), through an `unsafe extern "C"` static declaration resolved at link time
(WISDOM #56 says why not the crate's binding or `dlsym`), and a read-back in a parity test. The `libxslt` crate has no
safe setter; libxslt only reads the value when it builds a transform context, so the single guarded write cannot race
a transform. It stops pathologically deep stylesheet recursion gracefully, as Perl does.

### G. Editor server child processes (1 site)

`latexml_oxide/src/lsp_server/unix.rs`, the off-by-default `--server` editor mode (design archived at
`docs/archive/LSP_SERVER.md`), forks a worker child per request and drives it over a pipe. Only `fork(2)` is
`unsafe`; `pipe`, `read`/`write`, `poll`, `waitpid`, `kill_process` and the child's `open("/dev/null")` +
`dup2_stdout` are safe `rustix`, with the pipe ends `OwnedFd`s closed exactly once by their drop. The invariant is
**single-threaded at fork** (asserted before `fork()`), with plain syscalls between `fork()` and the child's
conversion.

### H. Script-bindings context bridge (4 sites; 3 more in its tests)

`latexml_contrib/src/script_bindings/{engine.rs,mod.rs}`: the `runtime-bindings` (Rhai) front-end re-mints `&`/`&mut`
references to the in-flight whatsit, document and properties from raw pointers the core publishes on a thread-local
context stack (`WHATSIT_CTX` and its siblings) for the duration of one hook body. The pointer is the only live
reference for the call; a `mutable` flag, checked first, gates the `&mut` sites; a call outside a hook context returns
an error, not undefined behaviour. Provenance and lifetime are argued at `mod.rs::with_doc` (the B1 re-entrancy
soundness note), and modelled under Miri by the test-only `latexml_core/src/runtime_bindings_reentrancy_model.rs`
(1 site).

### I. Integration tests (17 sites)

`latexml_oxide/tests/`: `set_var`/`remove_var` of the variables a test switches (`117_post_2gib_handoff.rs`,
`118_streaming_split_parity.rs`), `mallinfo2` (`112_fragment_yield.rs`), `wait4` with its `rusage`
(`perfect_kernel_batch106t.rs`), and the exit-report harness's signal and process calls
(`cortex_worker_exit_reports.rs`).

### Runtime bindings in the shipped binaries

**Ships on.** `runtime-bindings` is `latexml`'s only default feature, and `make_release.sh` names it explicitly;
`test-utils` is never on in a release build.

| Artifact | Build | These 4 sites |
|---|---|---|
| GitHub-release binaries | `--no-default-features --features runtime-bindings` | **present** |
| `cargo install latexml` | `default` | **present** |
| cli image | `--no-default-features --features runtime-bindings` | **present** |
| cortex-worker image (arXiv fleet) | `--no-default-features --features cortex` | **absent** |

Convenience vs deployment: the first three convert documents you already trust; the
worker batch-converts unvetted arXiv trees.

**`cortex_worker` never loads a runtime binding, in any build.** Features are additive,
so a worker built with `--features cortex` alone also compiles `runtime-bindings` in.
Its `main` therefore calls `converter::forbid_runtime_bindings()` first: every binding
chain the process installs (each conversion's session, a streaming restart's, the
post-only bibliography's) then has neither `.rhai` tier, and a `<pkg>.sty.rhai`
shipped in a source resolves as in a build without the feature (the raw `.sty` if
there is one, else `Warning:missing_file`). The children it spawns re-execute its
`main`, so the switch holds there too. Guards `cortex_worker_untrusted_bindings` (run
with `--features cortex`) and `forbid_runtime_bindings` (the switch, run by CI).

**Reachable without a flag.** `converter.rs::rhai_dispatch` is *first* in the
binding chain and runs on every package/class request, resolving `<pkg>.sty.rhai`
against the search paths — which include the document's own directory. So a `.rhai`
beside an untrusted source executes on a plain conversion and, being first,
*overrides* the compiled binding of the same name (`article.cls.rhai` shadows the
built-in). See `script_bindings_plan.md` §7.

**Also reachable from the host TeX tree (#345).** A `<pkg>.sty.rhai`
distributed inside a texmf tree on `$TEXINPUTS` is loaded too, so the executable
surface is not just the document's directory: it is every directory kpathsea
searches, including `TEXMFHOME`/`TEXMFLOCAL` and anything an operator has put on
`TEXINPUTS`. This tier is **last** in the chain, so it can only supply a binding
nothing else claimed — a texmf `.rhai` cannot displace a compiled one — but it is
still *code the conversion executes*. Treat write access to the texmf tree (and to
`TEXINPUTS`) as equivalent to code execution in any deployment that ships
`runtime-bindings`.

**Bound, not absent.** Rhai runs `no_module` + `no_time` — a script gets the
registered binding API, not host I/O. That is a *sandboxed execution* claim, not an
*inert code* one. Deployments on untrusted input should drop `--features
runtime-bindings`.

**External commands from bindings (#318).** The binding API exposes `Command`, a
thin mirror of `std::process::Command`, so a trusted binding can shell out
(BookML runs `latexmk`/`dvisvgm` during digestion) — as Perl `.ltxml` does with
`system()`. This is **allowed by default** and **blockable** by setting
`LATEXML_DISABLE_SHELL_ESCAPE` (any value), which makes `Command::output()` return
a Rhai error. Because a `.rhai` beside an untrusted source **auto-loads** (the
"Reachable without a flag" note above), a bundled binding could otherwise run
arbitrary commands: **any untrusted-input deployment that ships `--features
runtime-bindings` MUST set `LATEXML_DISABLE_SHELL_ESCAPE`, or build without the
feature.** (`cortex_worker` forbids runtime bindings at start, as above, so no binding of its runs a command.) The
command runs to completion (no per-call timeout, like `system()`); the conversion's wall-time limit is the outer bound.

---

## Audit posture

When adding `unsafe`:

First look for a safe API (`std`, `rustix`); the ratchet fails a new site. When one is truly needed:

1. **Document the invariant in a `// SAFETY:` comment at the site**, not in a separate file, so a reader sees the
   justification in context. Every production `unsafe {}` block carries one.
2. **Prefer category B** (a measured performance carve-out) over FFI (categories D–H) when both would work: B can be
   reverted without losing a function.
3. **Avoid `unsafe fn`**; there are none, on purpose. A helper takes an `unsafe {}` at the call site instead.
4. **Never add `unsafe impl Send/Sync` for a value the runtime actually shares.** The three existing markers are sound
   because the single-thread invariant holds by construction; a new marker must prove the same.
5. **Reconcile this inventory and run Miri.** When the count changes, lower the ratchet's pin and update the budget
   section and the category. If you touch the pure-Rust `unsafe` (the arena, category B), run `tools/miri_check.sh`,
   which checks those sites under Miri (CI runs it as the `miri` job). The FFI sites cannot run under Miri; their
   safety rests on the documented invariants.

---

## Out-of-scope risks

- **`\write18` shell escape**: not implemented. We do not invoke the
  shell on **author-supplied** (document) strings. (A **trusted binding** can
  run external commands via the `Command` API — allowed by default, opt-out
  `LATEXML_DISABLE_SHELL_ESCAPE`; see the runtime-bindings section above.)
- **Output sanitisation**: author-supplied `\href{javascript:…}` would
  pass through the HTML pipeline unchanged. Downstream consumers of
  the HTML output (web servers, viewers) are responsible for CSP /
  XSS sanitisation — this is content-policy, not memory-safety.
- **Resource limits**: the engine has internal timeouts and iteration
  limits (`IF_LIMIT`, `MAX_ERRORS`, gullet pushback caps) for
  pathological input. Subprocess rasterizers run under a hard wall-time
  guard. These are best-effort denial-of-service mitigations, not
  hardened isolation.

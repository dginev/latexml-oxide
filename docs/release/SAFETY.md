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

Reconciled **2026-11-25** (issue #937 Phases 1 and 3, and Phase 2's first step): the workspace contains **62 `unsafe {}`
blocks + 0 `unsafe impl Send/Sync` + 0 `unsafe fn` = 62 non-comment sites**, plus 3 the ratchet also counts (it counts every
`unsafe` keyword in code): two `unsafe extern "C"` blocks (`latexml_post/src/xslt.rs`,
`latexml_oxide/src/util/test.rs`) and one `#[unsafe(link_section)]` (`util/test.rs`) — **65 in all**: 34 in shipped
code, 5 in the `test-utils`-gated `latexml_oxide/src/util/test.rs`, and 26 test-only (`latexml_oxide/tests/`,
`latexml_post/src/test_env.rs`, `latexml_contrib/src/script_bindings/tests.rs`, the `#[cfg(test)]` code of
`graphics_cache.rs`, `xslt.rs` and `runtime_bindings_reentrancy_model.rs`).

The 34 shipped sites, each with its contract (a `// SAFETY:` comment at the site says it in context):

| File | Sites | What | Contract / why no safe API | Phase |
|---|---|---|---|---|
| `latexml_core/src/watchdog.rs` | 1 | `getrusage` (`rusage`, for `children_cpu_time_us` and macOS's peak RSS) | writes only into a zeroed `rusage` it owns, read after success; neither `std` nor `rustix` 1.1 wraps it | stays |
| | 1 | `sysconf(_SC_PHYS_PAGES)` (`total_memory_bytes`) | a pointer-free query, a negative return mapped to `None`; `rustix::system::sysinfo` is Linux-only, so macOS would keep the call | stays |
| | 1 | `_exit` (`exit_now`) | takes no pointers, never returns; `std` has no exit that skips the atexit handlers and destructors | 6 |
| | 4 | macOS `proc_pidinfo`, `host_statistics64` (with their zeroed structs) | fill at most the size passed into a zeroed struct we own | 6 |
| | 9 | Windows `K32GetProcessMemoryInfo` ×2, `GlobalMemoryStatusEx` ×2 (with their zeroed structs), `GetDiskFreeSpaceExW` | a correctly sized struct with its size field set; a NUL-terminated path that outlives the call | 6 |
| `latexml_core/src/util/pathname.rs` | 1 | `set_var("TEXMFROOT")` in `select_kpaths` | category D | stays |
| `latexml_core/src/common/arena.rs` | 1 | `resolve_unchecked` | category B | perf |
| `latexml_oxide/bin/cortex_worker/exit_report.rs` | 2 | `sigaction` installation; the handler's chain call, `signal`, `raise` | async-signal-safe; the handler table is written once at start-up | 6 |
| `latexml_oxide/bin/cortex_worker.rs` | 3 | `mmap(PROT_NONE)`, `std::alloc::alloc`, `raise(SIGSEGV)` | debug-assertion builds only (the harness tests' fault injection); `rustix::mm::mmap_anonymous` is itself `unsafe`, `raise` is thread-directed (`tgkill`), which `rustix` 1.1 lacks | stays |
| `latexml_oxide/src/heap_trim.rs` | 3 | `mallinfo2` (`c_heap_mb`), `mi_collect` (`collect_rust_heap`), `malloc_trim` (`release_free_heap`) | allocator queries and trims, the only shipped callers of these three (`tests/112_fragment_yield.rs` reads `mallinfo2` itself); live allocations stay valid. the `mimalloc` crate exposes no collect, and `mallinfo2`/`malloc_trim` have no safe wrapper in `libc` or `rustix` (the C heap is glibc's) | 2 |
| `latexml_oxide/src/lsp_server/unix.rs` | 1 | `fork` | category G | stays |
| `latexml_post/src/graphics.rs` | 1 | `pre_exec(setsid, setpgid, prctl(PR_SET_PDEATHSIG), getppid, _exit)` | category E | stays |
| `latexml_post/src/xslt.rs` | 2 | `xsltMaxDepth`: its `extern` declaration and the `Once`-guarded write | category F | 4 |
| `latexml_contrib/src/script_bindings/{engine,mod}.rs` | 4 | whatsit / document / properties pointers | category H | 6 |

**The count may only fall** (issue #937). `unsafe_sites_ratchet::unsafe_sites_only_fall`
(`latexml_oxide/tests/cluster_package_guards/unsafe_sites_ratchet.rs`, run by CI's workspace nextest) pins the sites
per file, read from the lexed source (comments and literals excluded, `#[cfg(test)]` code counted). A change that
removes sites lowers its file's pin (the failure prints the table to paste); one that adds a site fails it, and
raising a pin is an exception its commit argues.

Major changes since the 2026-07-20 reconciliation (55 sites):
- **2026-11-25 (issue #937 Phase 2, first step), 66 → 62 blocks (69 → 65 counting every `unsafe` keyword), no
  behaviour change:** the allocator calls sit behind two safe functions in `heap_trim.rs`, the one shipped caller of each:
  `c_heap_mb()` (`mallinfo2`; the streaming spill trace and fragment telemetry, `core_interface.rs` 3 → 0, and the render loop's
  `LATEXML_POST_MEMDIAG` probe, `post.rs` 1 → 0) and `collect_rust_heap()` (`mi_collect`; `release_free_heap` and
  `streaming_restart.rs` 1 → 0). Their `cfg` gates are unchanged: `mallinfo2` on Linux, `mi_collect` off under
  `dhat-heap`. `heap_trim.rs` 2 → 3; `malloc_trim` stays where it was (Phase 2's next step measures it first).
- **2026-11-25 (issue #937 Phases 1 and 3, the rest), 77 → 66 blocks (80 → 69 counting every `unsafe` keyword), no
  behaviour change:** `logger.rs` 2 → 0 (the log mirror is a `&'static File`, cortex_worker's claim's, behind a
  lock-free on/off flag: `rustix::fs::ftruncate` and `rustix::io::write`, EINTR retried as before). `exit_report.rs`
  4 → 2 (`rustix::process::getpid`; the `cause` line is a `rustix::io::write` to the claim's `File`, reached by
  `OnceLock::get`, one atomic load, so still async-signal-safe). `watchdog.rs` 18 → 16 (`write_stderr_unlocked` by
  `rustix::io::write(stderr())`; the macOS `getrusage` and the two binaries' `RUSAGE_CHILDREN` copies are one
  `rusage` helper). `cortex_worker.rs` 4 → 3 and `latexml_oxide.rs` 2 → 0 (that helper; `--init` is
  `dump_paths::set_ini_mode()`, an in-process flag, instead of `set_var("LATEXML_INI_MODE")`, which stays a read
  input). `identity.rs` 2 → 0 (its test passes the epoch to `start_time_at` instead of setting
  `SOURCE_DATE_EPOCH`). No shipped code writes the environment after start-up but `pathname.rs`'s `TEXMFROOT`.
- **2026-11-25 (issue #937 Phase 5), 80 → 77 blocks/impls (83 → 80 counting every `unsafe` keyword), no behaviour
  change:** the 3 `unsafe impl Send/Sync` markers are gone (`common/store.rs` 2 → 0, `state.rs` 1 → 0); `State` and
  `Stored` are now `!Send + !Sync`, as their `Rc`s and libxml nodes make them. No `State` ever crossed a thread (each
  is built inside its `#[thread_local]` static), and no error type carries a `Stored`: the one bound that needed the
  markers was the `static NO_PROPERTIES: LazyLock<HashMap<Stored>>` (a `static` must be `Sync`), now a `const`
  reference to an empty map, which needs neither.
- **2026-11-24 (issue #937 Phases 0 and 1, first part), 106 → 80 blocks/impls (83 counting every `unsafe` keyword),
  no behaviour change:** Unix syscalls with a safe
  equivalent call it. `lsp_server/unix.rs` 16 → 1 (`rustix::pipe::pipe` + `OwnedFd`/`File::from` for `pipe`, `close`
  and `from_raw_fd`; `rustix::io::{read, write}`, `rustix::event::poll`, `rustix::process::{waitpid, kill_process}`,
  `rustix::fs::open` + `rustix::stdio::dup2_stdout` for the child's `/dev/null`; only `fork` stays). `watchdog.rs`
  22 → 18 (`rustix::param::page_size` ×2, `rustix::fs::statvfs`). `cortex_worker/exit_report.rs` 6 → 4
  (`File::lock`/`try_lock` for `flock`, `rustix::process::test_kill_process` for `kill(pid, 0)`). `graphics_cache.rs`
  2 → 1 (`File::try_lock`). `private_files.rs` 1 → 0 (`rustix::process::geteuid`). `xslt.rs` 5 → 2 (its test's
  stderr capture: `rustix::io::dup` + `rustix::stdio::dup2_stderr`). Left for later phases: `getrusage` (rustix has
  none), `raise` (thread-directed; `kill_process(getpid())` is process-directed), `sysconf(_SC_PHYS_PAGES)`, the
  logger's `ftruncate`/`write` on a raw fd shared with the signal path, `setsid` in `pre_exec`, the `mmap` probe.
- **2026-11-15 (ARENAR):** `latexml_core/src/common/arena.rs` 10 blocks → 1 (category B's eight resolves and `first_bytes`'s raw read fold into one; category C is gone).
- **2026-11-20:** removed `unsafe impl Send/Sync for Error` (2 sites; `ErrorCategory::Generic` now bounds `Box<dyn Error + Send + Sync>` so `Error` is safely `Send + Sync`), replaced the 2 `unsafe { libc::killpg }` calls in `latexml_post/src/graphics.rs` with safe `rustix::process` helpers in `latexml_core/src/watchdog.rs` (0 new `unsafe`; `graphics.rs` 3 → 1, `watchdog.rs` unchanged at 22), and added `// SAFETY:` comments to all glibc/mimalloc heap-trim & `mallinfo2` sites (`core_interface.rs`, `post.rs`, `render_workers.rs`, `streaming_restart.rs`).
- **New OS/runtime FFI sites added since July 2026:** `latexml_core/src/watchdog.rs` (22 sites: portable RSS/RAM/disk queries), `latexml_oxide/src/lsp_server/unix.rs` (16 sites: POSIX fork/pipe/poll worker for `--server`), `latexml_oxide/src/{core_interface,heap_trim,post,streaming_restart}.rs` (7 sites: `malloc_trim`/`mi_collect`/`mallinfo2`; the cadenced trims share `heap_trim::release_free_heap`), `latexml_oxide/bin/cortex_worker/exit_report.rs` (6 sites: async-signal-safe crash/exit reporter), `latexml_post/src/xslt.rs` (5 sites), `latexml_oxide/bin/cortex_worker.rs` (4 sites: `mmap`/`raise`/`getrusage`), `latexml_oxide/src/util/test.rs` (3 sites). Every production `unsafe` block carries a `// SAFETY:` comment at the call site.

Counts and exact line numbers drift as the tree changes — this is a
point-in-time reconciliation, not a live index. Regenerate the authoritative
enumeration with:

```
rg -n 'unsafe\s*\{|unsafe impl|unsafe fn' latexml_* -g '*.rs'
```

Still `unsafe` because no safe API covers them yet (later phases of #937): `getrusage` (rustix has none), `raise`
(thread-directed; `kill_process(getpid())` is process-directed), `sysconf(_SC_PHYS_PAGES)`, the logger's
`ftruncate`/`write` on a raw fd shared with the signal path, `setsid` in `pre_exec`, the `mmap` probe. Every
production `unsafe` block carries a `// SAFETY:` comment at the call site.

---

### A. `unsafe impl Send/Sync` — none (removed 2026-11-25, issue #937 Phase 5)

`State` (`latexml_core/src/state.rs`) and `Stored` (`latexml_core/src/common/store.rs`) hold `Rc`/`RefCell` and
`libxml::tree::Node`, so the compiler makes them `!Send + !Sync`, which matches the design: each conversion's engine
state lives in `#[thread_local]` statics (`STATE`, `STD_STATE`, `STY_STATE`) and is built on the thread that uses it.
Errors carry no `Stored` (`Error` is `Send + Sync` by construction since 2026-11-20, through
`ErrorCategory::Generic`'s `Box<dyn Error + Send + Sync>`). A shared empty value is a `const` reference
(`NO_PROPERTIES`), never a `static`, which would need `Sync`.

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

### D. Process start-up: the environment, rusage, a test crash trap (7 sites)

In `latexml_core/src/util/pathname.rs`, `latexml_core/src/watchdog.rs` and `latexml_oxide/src/util/test.rs`:

| Call | Why unsafe is unavoidable |
|---|---|
| `std::env::set_var("TEXMFROOT", …)` (pathname.rs `select_kpaths`) | The linked libkpathsea (C) reads `TEXMFROOT` from `environ` while it initializes; it has no other input for it, and the `kpsewhich`/`texlua` children inherit it. `init_kpathsea()` forces the write on the converting thread before any other thread runs (both binaries; `main` is blocked joining it), so no concurrent `getenv` walks the reallocated `environ` (STABILITY_WITNESSES Cluster J; `tools/lint_env_writes.sh`). A library host converting on several threads calls `init_kpathsea()` once first. |
| `libc::getrusage(…)` (watchdog.rs `rusage`) | Child user/sys CPU time (`children_cpu_time_us`, both binaries' timing lines) and macOS's peak RSS. No safe wrapper in `std` or `rustix` (1.1); one call site. |
| `libc::signal(SIGSEGV/SIGBUS/SIGABRT, handler)`, `mem::transmute(SIG_DFL)`, `raise(sig)` (util/test.rs) | A **test-only** SIGSEGV/SIGBUS/SIGABRT trap that dumps a backtrace to a per-pid file before the signal kills the test binary. `signal(3)` is `unsafe extern "C"`; the handler-reset path transmutes `SIG_DFL` to `sighandler_t`. |

`--init` writes no environment since 2026-11-25 (`dump_paths::set_ini_mode()`, an in-process flag).

Image converters (`gs`, `pdftocairo`, `mutool draw`, `ps2pdf`, `convert`) can spawn grandchildren, which a plain
`Child::kill()` does not reach, so each runs as a process-group leader.

| Sites | Call | Why |
|---|---|---|
| `latexml_post/src/graphics.rs` (1) | `cmd.pre_exec(|| setsid(); setpgid(0,0); prctl(…))` | Runs between `fork()` and `exec()`, so it must be async-signal-safe. `setsid(2)` makes the child a group leader so the whole group can be killed; the signalling itself is safe `rustix::process` (`watchdog.rs`). `Command::process_group` gives no `setsid` + `killpg`. |
| `latexml_post/src/test_env.rs` (3), `latexml_post/src/graphics_cache.rs` (1) | `set_var`/`remove_var` in `EnvGuard`; one `OnceLock`-initialised test `set_var` | **Test-only.** Both hold the binary-wide `env_lock` across set, use and restore. The residual, documented in `test_env.rs`: a linked C library can call `getenv` concurrently, which no Rust lock serialises, so no correctness rests on these variables (the graphics cache takes an explicit `CachePolicy`). |

| Call | Why |
|---|---|
| `cmd.pre_exec(|| setsid(); setpgid(0,0); prctl(…))` | Runs post-`fork()`/pre-`exec()`; must be async-signal-safe. `setsid(2)` makes the child a process-group leader so the whole group is killable (signaling itself now uses safe `rustix::process` via `watchdog.rs`). |
| `std::env::set_var`/`remove_var` in `EnvGuard` (test_env.rs) + a test `set_var` (graphics_cache.rs) | **Test-only.** `set_var`/`remove_var` are `unsafe` in Rust 2024 (concurrent env mutation races). `EnvGuard` is used only in test setup, but the default `cargo test` harness runs a binary's tests multi-threaded and other tests read the env — so the comment documents the invariant and keeps a `FIXME` proposing `serial_test` to make it airtight. The `graphics_cache.rs` test `set_var` is `OnceLock`-serialized. |

`Command::process_group(0)` is `setpgid(0, 0)`, not `setsid()`: no new session, and the child keeps the controlling
terminal; and `PR_SET_PDEATHSIG` can only be set between `fork` and `exec`. So the `pre_exec` stays. (The prune
lock's `flock` is `File::try_lock` since 2026-11-24.)

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

1. **Document the invariant in a `// SAFETY:` comment at the site**, not
   in a separate file. Future readers should see the justification in
   context. Every `unsafe {}` block must carry one (the sole documented
   exception is the test-only `EnvGuard`, cat. E, whose comment states the
   invariant but flags a `FIXME` that the harness doesn't yet enforce it).
2. **Prefer category B** (perf carve-out with explicit measurement) over
   category D/E (FFI) when both would work, because B is reversible
   without losing functionality.
3. **Avoid `unsafe fn`** — we have zero, intentionally. Internal helpers
   can take an `unsafe {}` at the call site without polluting the
   function signature.
4. **Never add `unsafe impl Send/Sync`.** There are none (issue #937
   Phase 5): a bound that asks for `Send`/`Sync` on a thread-confined
   type is removed at its source (an error carries a `Send` rendering, a
   shared empty value is a `const`), not asserted.
5. **Reconcile this inventory and run Miri.** When the `unsafe` count
   changes, refresh the budget section (the `rg` one-liner above). If you
   touch the FFI-free pure-Rust `unsafe` (the arena/interner, cat. B),
   run `tools/miri_check.sh` — it UB-checks those sites under Miri (CI runs
   it as the `miri` job). The FFI sites (cat. D–H) can't run under Miri;
   their safety rests on the documented invariants, not interpretation.

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

# Safety Profile and `unsafe` Inventory

## Threat model

`latexml_oxide` is a **batch document compiler**, not a network service or
multi-tenant process. The program is invoked from a shell or CI runner on
locally-trusted LaTeX input that the user already chose to compile. Inputs
come from arXiv-style preprints, user sources, or test fixtures.

What follows from that:

- **No untrusted-network surface.** We do not parse data over a socket, and
  no `unsafe` block touches a network-derived buffer.
- **No privilege boundary.** The process holds the invoking user's UID and
  performs no privilege drops; nothing we do crosses a sudo / setuid /
  capability boundary.
- **Single conversion job per process** (in practice). The `cortex_worker`
  spawns a worker pool, but each worker conversion runs on exactly one
  OS thread — the engine state is pinned to a thread-local global
  (CLAUDE.md design decision 0.3.2).
- **Memory-safe by default.** The dependency graph (libxml2 via the `libxml`
  crate, gs/pdftocairo/mutool as subprocess CLIs, kpathsea via FFI) is
  the only place we are not pure-Rust, and we sandbox each subprocess to a
  short timeout (`Graphics::run_with_timeout`, see graphics.rs).

The output is HTML / MathML / XML rendered from LaTeX — the *content* may
need sanitisation downstream (CSS / `<script>` injection in author-supplied
`\href`, `\url`, `\write18`), but those are content-policy concerns, not
memory-safety concerns. We do not execute `\write18` (the `\immediate\write`
shell-escape) at all.

## `unsafe` budget

Reconciled **2026-06-24**: the project contains **48 `unsafe {}` blocks +
5 `unsafe impl Send/Sync` + 0 `unsafe fn` = 53 sites**. Every `unsafe` block
carries a `// SAFETY:` comment at the call site naming the invariant it
depends on (the one caveat is category E's *test-only* `EnvGuard`, which
documents the env data-race invariant but carries a `FIXME` that the default
test harness does not yet *enforce* single-threaded env access).

Counts and exact line numbers drift as the tree changes — this is a
point-in-time reconciliation, not a live index. Regenerate the authoritative
enumeration with:

```
rg -n 'unsafe\s*\{|unsafe impl|unsafe fn' -g '*.rs' -g '!target'
```

The categories below describe what is unavoidable, what is a deliberate
performance trade, and what could be refactored away at some cost. Locations
are given at the file/function level (not exact line) to stay robust against
drift. **When this count changes, update this section** (see Audit posture).

---

### A. `unsafe impl Send/Sync` — pragmatic single-thread markers (5 sites)

| Site | Marker | Why |
|---|---|---|
| `latexml_core/src/state.rs` | `Send for State` | `State` holds `Rc`/`RefCell`/`libxml::tree::Node`, all `!Send`. We mark it `Send` so the harness can build a State on one thread and *transition* it to another thread before any first use. After that first use, the value is pinned to a single OS thread via `#[thread_local]` switchers (`use_main_state` / `use_std_state` / `use_sty_state`). The crate never reads a `State` from two threads concurrently. |
| `latexml_core/src/common/store.rs` | `Send/Sync for Stored` | Same shape as `State`: `Stored` variants embed `Rc<RefCell<…>>` and libxml nodes. The marker exists *only* to satisfy `Box<dyn Error + Send + Sync>` trait bounds on error returns — error variants transitively require `Send + Sync` on every embedded type. No code actually shares a `Stored` across threads. |
| `latexml_core/src/common/error.rs` | `Send/Sync for Error` | `Error` carries a `Locator` whose `Mouth` is `!Send`. Identical justification to `Stored`: needed for the std `Error + Send + Sync` bound; not for actual cross-thread sharing. |

**Could we refactor?** Only by swapping every `Rc<RefCell<…>>` in `State`,
`Stored`, and `Error` for `Arc<Mutex<…>>`. That contradicts the
deliberately single-threaded-per-conversion design, adds Mutex acquisition
to the hot path, and would not improve safety — the existing markers are
sound because the runtime guarantees the invariant by construction.

### B. Arena `resolve_unchecked` — performance carve-out (8 sites)

All in `latexml_core/src/common/arena.rs`, inside `with`, `with2`, `with3`,
`with_many`, and `to_string`. They use `string-interner`'s unchecked
`Symbol → &str` lookup.

The safety invariant is **append-only buffer**: every `SymStr` in the
codebase originates from a successful `get_or_intern(_static|_char)` call
on a valid `&str`. Symbols are never invalidated, so the unchecked lookup
is a valid bounds-skip.

**Why we accept the `unsafe`**: callgrind measured the checked `resolve`
(`from_utf8` validation per call) at ~3% of total instruction count. The
arena is the hottest read site in the engine — every Token-to-string
serialisation goes through it.

**Refactor option**: replace each `arena.resolve_unchecked(sym)` with
`arena.resolve(sym).expect("interned")`. This is mechanical and eliminates
eight `unsafe` blocks. The cost is a re-validating `from_utf8` per lookup,
~3% of total runtime.

### C. Arena re-entrant `&mut *ptr` (1 site)

`latexml_core/src/common/arena.rs` — inside `with_arena_mut`. The outermost
caller acquires `RefCell::borrow_mut()` and caches a raw pointer to the
interner; nested re-entrant callers on the same thread reuse the pointer and
skip the `RefCell` guard, which would otherwise panic ("already borrowed").

The safety invariant is documented in the function header: re-entrance is
nested on the same stack and same thread (`#[thread_local]`), an
`ArenaCleanup` guard clears the pointer before the outer `RefMut` drops.

**Cannot be refactored without changing semantics** — there is no safe
mechanism for nested re-entrant mutable access through `RefCell`.

### D. Binary entry-point FFI — env + rusage + crash-handler (~7 sites)

In `latexml_oxide/bin/latexml_oxide.rs`, `latexml_oxide/bin/cortex_worker.rs`,
and `latexml_oxide/src/util/test.rs`:

| Call | Why unsafe is unavoidable |
|---|---|
| `std::env::set_var("LATEXML_INI_MODE", …)` (latexml_oxide.rs) | Rust 2024 marked `set_var` `unsafe` by design — pre-thread-spawn env mutation isn't race-free in general. We call it before spawning anything. |
| `libc::getrusage(RUSAGE_CHILDREN, …)` (latexml_oxide.rs, cortex_worker.rs) | C FFI to capture child user/sys CPU time. No stable safe wrapper in stdlib. |
| `libc::signal(SIGSEGV/SIGBUS/SIGABRT, handler)`, `mem::transmute(SIG_DFL)`, `raise(sig)` (util/test.rs) | A **test-only** SIGSEGV/SIGBUS/SIGABRT trap that dumps a backtrace to a per-pid file before the signal kills the test binary. `signal(3)` is `unsafe extern "C"`; the handler-reset path transmutes `SIG_DFL` to `sighandler_t`. |

None can be refactored without losing the functionality (env mutation,
rusage capture, signal-handler debugging).

### E. Subprocess process-group lifecycle + test EnvGuard in graphics (~8 sites)

`latexml_post/src/graphics.rs` + `latexml_post/src/graphics_cache.rs` —
process-group control is needed because subprocess rasterizers (`gs`,
`pdftocairo`, `mutool draw`, `ps2pdf`, `convert`) can spawn grandchildren,
and a plain `Child::kill()` does not reap those.

| Call | Why |
|---|---|
| `cmd.pre_exec(|| setsid(); setpgid(0,0); prctl(…))` | Runs post-`fork()`/pre-`exec()`; must be async-signal-safe. `setsid(2)` makes the child a process-group leader so the whole group is killable. |
| `libc::killpg(pid, SIGTERM)` / `killpg(pid, SIGKILL)` | Graceful then hard kill of the whole group on timeout (mirrors `timeout(1) --kill-after`). |
| `libc::flock(fd, …)` (graphics_cache.rs) | Advisory lock on an owned, open lock-file fd; operates on the fd without aliasing Rust memory. |
| `std::env::set_var`/`remove_var` in `EnvGuard` (graphics.rs) + a test `set_var` (graphics_cache.rs) | **Test-only.** `set_var`/`remove_var` are `unsafe` in Rust 2024 (concurrent env mutation races). `EnvGuard` is used only in test setup, but the default `cargo test` harness runs a binary's tests multi-threaded and other tests read the env — so the comment documents the invariant and keeps a `FIXME` proposing `serial_test` to make it airtight. The `graphics_cache.rs` test `set_var` is `OnceLock`-serialized. |

`Command::process_group` was stabilised but does not give us
`setsid + killpg`, so the FFI path stays.

### F. libxslt/libxml global config via FFI (2 sites)

`latexml_post/src/xslt.rs` — a `Once`-guarded `dlsym` write to libxslt's
process-global `xsltMaxDepth` recursion cap (`= 1000`, a faithful port of Perl
`XML::LibXSLT->max_depth(1000)`), plus a `dlsym` read-back in a parity test.
The `libxslt` crate exposes no safe setter; libxslt only READS the value (when
building each transform context), so the single guarded write cannot race a
transform. Hardens the post-processor against stack-overflow/OOM on
pathologically-deep stylesheet recursion (it aborts gracefully, like Perl).

### G. LSP server POSIX child-process management (16 sites)

`latexml_oxide/src/lsp_server/unix.rs` — the **off-by-default** `--server`
editor mode (design archived at `docs/archive/LSP_SERVER.md`) forks a worker
child per request and drives it over a pipe with raw POSIX calls: `pipe(2)`,
`fork(2)`, `open("/dev/null")`/`dup2`, `poll(2)`, `waitpid(2)`, `kill`/`killpg`,
`read`/`write`, and `File::from_raw_fd` on the pipe ends. Each block is
annotated. The invariant for the fork/child window is **single-threaded at
fork** (asserted before `fork()`) and **async-signal-safe-only** libc calls
between `fork()` and `exec()`/`_exit()` — no Rust heap allocation or drop glue
in that window. Pipe-end fds are owned and closed exactly once.

### H. Script-bindings whatsit-pointer bridge (5 sites)

`latexml_contrib/src/script_bindings/{engine.rs,mod.rs}` — the `runtime-bindings`
(Rhai) front-end re-mints `&`/`&mut`
references to the in-flight whatsit / document / properties from raw pointers
the core publishes onto a thread-local active-context stack (`WHATSIT_CTX`)
for the duration of a single hook body. The pointer is the sole live reference
for the call; a `mutable` flag (checked first) gates the `&mut` sites; calling
outside a hook context returns an error, not UB. Provenance + lifetime are
documented in the `mod.rs::with_doc` *B1 SOUNDNESS CAVEAT*.

**In the downloaded binaries this ships COMPILED IN AND LIVE — it is not off by
default.** This section said "off-by-default `script-bindings`" until 2026-07-17;
both halves were wrong. The feature is `runtime-bindings` (the `script-bindings`
alias was removed pre-publish), it is in `latexml`'s `default`, and
`make_release.sh` *deliberately keeps it* while dropping `test-utils` — so these 5
sites are in every GitHub-release binary (tarballs, `.zip`, `.deb`).

**Where it ships, as of 2026-07-17:**

| Artifact | Build | These 5 sites |
|---|---|---|
| GitHub-release binaries | `--no-default-features --features runtime-bindings` | **present** |
| `cargo install latexml` | `default` (includes `runtime-bindings`) | **present** |
| `ghcr.io/dginev/latexml-oxide` (cli image) | `--no-default-features --features runtime-bindings` | **present** |
| cortex-worker image (the arXiv fleet) | `--no-default-features --features cortex` | **absent** |

The line is **end-user convenience vs production deployment**, not
binary-vs-container. The first three are ways to convert *your own* documents, and
there the feature is the point: drop a `.rhai` beside a file and override a binding
with no toolchain. The worker is our production path — batch-converting arXiv
source trees nobody vetted — and it builds without the feature.

**A deployment converting untrusted input should switch `runtime-bindings` off**,
including one built from the cli image's recipe: drop `--features runtime-bindings`
and rebuild. The convenience and the exposure are the same mechanism, so a
deployment cannot keep one without the other.

Nor is the path dormant. `converter.rs::rhai_dispatch` sits **first** in the
binding-resolution chain and runs on *every* package/class request: for `foo.sty`
it does a `find_file` for `foo.sty.rhai` with `search_paths_only: true`, and
executes it if found. So a `.rhai` file placed in a **search path — which includes
the document's own directory** — is loaded and run by a plain conversion, with no
flag. Converting an untrusted source tree (e.g. an arXiv tarball) therefore means
executing any `<pkg>.sty.rhai` it ships that the document `\usepackage`s. And
because the `.rhai` tier is checked **first**, such a file *overrides the compiled
binding of the same name* — an `article.cls.rhai` shadows the built-in
`article_cls` (`script_bindings_plan.md` §7).

Bounding it: the interpreter is Rhai with `no_module` + `no_time`, so a script
gets the registered LaTeXML binding API, not arbitrary host I/O, and
`search_paths_only` keeps the lookup off the TeX tree. But this is a *sandboxed
execution* claim, not an *inert code* claim, and it should be reviewed as such.
Builds that want it truly gone must use `--no-default-features` *without*
`--features runtime-bindings`.

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
4. **Never add `unsafe impl Send/Sync` for a value the runtime actually
   shares.** The five existing markers are sound because the
   single-thread invariant is upheld by construction; new markers must
   prove the same.
5. **Reconcile this inventory and run Miri.** When the `unsafe` count
   changes, refresh the budget section (the `rg` one-liner above). If you
   touch the FFI-free pure-Rust `unsafe` (the arena/interner, cat. B/C),
   run `tools/miri_check.sh` — it UB-checks those sites under Miri (CI runs
   it as the `miri` job). The FFI sites (cat. D–H) can't run under Miri;
   their safety rests on the documented invariants, not interpretation.

---

## Out-of-scope risks

- **`\write18` shell escape**: not implemented. We do not invoke the
  shell on author-supplied strings.
- **Output sanitisation**: author-supplied `\href{javascript:…}` would
  pass through the HTML pipeline unchanged. Downstream consumers of
  the HTML output (web servers, viewers) are responsible for CSP /
  XSS sanitisation — this is content-policy, not memory-safety.
- **Resource limits**: the engine has internal timeouts and iteration
  limits (`IF_LIMIT`, `MAX_ERRORS`, gullet pushback caps) for
  pathological input. Subprocess rasterizers run under a hard wall-time
  guard. These are best-effort denial-of-service mitigations, not
  hardened isolation.

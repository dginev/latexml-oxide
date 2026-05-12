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

As of 2026-05-12 the project contains **22 `unsafe` sites across 8 files**
— five `unsafe impl Send/Sync` markers, sixteen `unsafe {}` blocks, and
no `unsafe fn`. Every site carries a `// SAFETY:` comment at the call
site that names the invariant it depends on.

The five categories below describe what is unavoidable, what is a deliberate
performance trade, and what could be refactored away at some cost.

---

### A. `unsafe impl Send/Sync` — pragmatic single-thread markers (5 sites)

| File:line | Marker | Why |
|---|---|---|
| `latexml_core/src/state.rs:279` | `Send for State` | `State` holds `Rc`/`RefCell`/`libxml::tree::Node`, all `!Send`. We mark it `Send` so the harness can build a State on one thread and *transition* it to another thread before any first use. After that first use, the value is pinned to a single OS thread via `#[thread_local]` switchers (`use_main_state` / `use_std_state` / `use_sty_state`). The crate never reads a `State` from two threads concurrently. |
| `latexml_core/src/common/store.rs:557-558` | `Send/Sync for Stored` | Same shape as `State`: `Stored` variants embed `Rc<RefCell<…>>` and libxml nodes. The marker exists *only* to satisfy `Box<dyn Error + Send + Sync>` trait bounds on error returns — error variants transitively require `Send + Sync` on every embedded type. No code actually shares a `Stored` across threads. |
| `latexml_core/src/common/error.rs:471-472` | `Send/Sync for Error` | `Error` carries a `Locator` whose `Mouth` is `!Send`. Identical justification to `Stored`: needed for the std `Error + Send + Sync` bound; not for actual cross-thread sharing. |

**Could we refactor?** Only by swapping every `Rc<RefCell<…>>` in `State`,
`Stored`, and `Error` for `Arc<Mutex<…>>`. That contradicts the
deliberately single-threaded-per-conversion design, adds Mutex acquisition
to the hot path, and would not improve safety — the existing markers are
sound because the runtime guarantees the invariant by construction.

### B. Arena `resolve_unchecked` — performance carve-out (8 sites)

All in `latexml_core/src/common/arena.rs` (lines 191, 201, 202, 211, 212,
213, 224, 233), inside `with`, `with2`, `with3`, `with_many`, and
`to_string`. They use `string-interner`'s unchecked `Symbol → &str` lookup.

The safety invariant is **append-only buffer**: every `SymStr` in the
codebase originates from a successful `get_or_intern(_static|_char)` call
on a valid `&str`. Symbols are never invalidated, so the unchecked lookup
is a valid bounds-skip.

**Why we accept the `unsafe`**: callgrind measured the checked `resolve`
(`from_utf8` validation per call) at ~3% of total instruction count. The
arena is the hottest read site in the engine — every Token-to-string
serialisation goes through it.

**Refactor option**: replace each
`arena.resolve_unchecked(sym)`
with
`arena.resolve(sym).expect("interned")`.
This is mechanical and eliminates eight `unsafe` blocks. The cost is a
re-validating `from_utf8` per lookup, ~3% of total runtime.

### C. Arena re-entrant `&mut *ptr` (1 site)

`latexml_core/src/common/arena.rs:78` — inside `with_arena_mut`. The
outermost caller acquires `RefCell::borrow_mut()` and caches a raw pointer
to the interner; nested re-entrant callers on the same thread reuse the
pointer and skip the `RefCell` guard, which would otherwise panic ("already
borrowed").

The safety invariant is documented in the function header: re-entrance is
nested on the same stack and same thread (`#[thread_local]`), an
`ArenaCleanup` guard clears the pointer before the outer `RefMut` drops.

**Cannot be refactored without changing semantics** — there is no safe
mechanism for nested re-entrant mutable access through `RefCell`.

### D. Binary entry-point FFI (5 sites)

| File:line | Call | Why unsafe is unavoidable |
|---|---|---|
| `latexml_oxide.rs:423` | `std::env::set_var("LATEXML_INI_MODE", "1")` | Rust 2024 marked `set_var` `unsafe` by design — pre-thread-spawn env mutation isn't race-free in general. We call it before spawning anything. |
| `latexml_oxide.rs:708`, `cortex_worker.rs:329` | `libc::getrusage(RUSAGE_CHILDREN, …)` | C FFI to capture child user/sys CPU time. No stable safe wrapper in stdlib. |
| `latexml_oxide/src/util/test.rs:130, 137` | `libc::signal(SIGSEGV/SIGBUS/SIGABRT, handler)` and `raise(sig)` | Installing a SIGSEGV/SIGBUS/SIGABRT trap to dump a backtrace to a per-pid file before the signal kills the test binary. `signal(3)` is `unsafe extern "C"` and the handler-reset path uses `mem::transmute` of `SIG_DFL`. |

None can be refactored without losing the functionality (env mutation,
rusage capture, signal-handler debugging).

### E. Process-group lifecycle in graphics (3 sites)

`latexml_post/src/graphics.rs:890, 911, 915` — needed because subprocess
rasterizers (`gs`, `pdftocairo`, `mutool draw`, `inkscape`, `convert`) can
spawn child grandchildren, and a plain `Child::kill()` does not reap those.

| Site | Call | Why |
|---|---|---|
| `:890` | `cmd.pre_exec(|| libc::setsid(); libc::setpgid(0,0))` | `pre_exec` is `unsafe fn` per std API — the closure runs between `fork()` and `exec()` and must be async-signal-safe. `setsid(2)` is the documented way to make the child a process-group leader. |
| `:911` | `libc::killpg(pid, SIGTERM)` | Graceful kill of the whole group on timeout. |
| `:915` | `libc::killpg(pid, SIGKILL)` | Hard kill if SIGTERM grace expires. |

These mirror what `timeout(1) --kill-after` does for the bench script's
outer guard. They cannot be expressed via safe stdlib APIs:
`Command::process_group` was stabilised but does not give us
`setsid + killpg`.

---

## Audit posture

When adding `unsafe`:

1. **Document the invariant in a `// SAFETY:` comment at the site**, not
   in a separate file. Future readers should see the justification in
   context.
2. **Prefer category B** (perf carve-out with explicit measurement) over
   category D/E (FFI) when both would work, because B is reversible
   without losing functionality.
3. **Avoid `unsafe fn`** — we have zero, intentionally. Internal helpers
   can take an `unsafe {}` at the call site without polluting the
   function signature.
4. **Never add `unsafe impl Send/Sync` for a value the runtime actually
   shares.** The three existing markers are sound because the
   single-thread invariant is upheld by construction; new markers must
   prove the same.

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

# WebAssembly (WASM) Compatibility Plan — Stage 4

**Status:** Planned / Architecture Locked (2026-09-03).  
Operationalizes **Stage 4 — WASM** of [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §portability and operationalizes the findings in [`WASM_COMPATIBILITY_AUDIT.md`](WASM_COMPATIBILITY_AUDIT.md).

---

## 1. Architectural Strategy & Locked Decisions

Following the successful precedent of [`WINDOWS_COMPATIBILITY_PLAN.md`](WINDOWS_COMPATIBILITY_PLAN.md), the architectural strategy locks key decisions with the maintainer:

| Dimension | Decision | Rationale |
|---|---|---|
| **Compilation Target** | **`wasm32-wasip1`** (formerly `wasm32-wasi`) | Single unified target that addresses **both** serverless/edge runtimes and client-side web browsers. |
| **Client-Side Browser Execution** | **`@bjorn3/browser_wasi_shim`** (in Web Worker) | Lightweight (~15 KB) JavaScript shim mapping WASI Preview 1 syscalls to browser APIs and providing an in-memory virtual filesystem (`memfs`). No Emscripten runtime bloat. |
| **C Dependencies Approach** | **`wasi-sdk`** (Clang targeting `wasm32-wasip1` + `wasi-libc`) | Preserves 100% bug-for-bug DOM, XPath 1.0, EXSLT, and Marpa ASF math parsing fidelity. Avoids high-risk multi-month pure-Rust rewrites. |
| **Crate Ownership & Maintenance** | **Author-owned upstream crates (`dginev`)** | `rust-libxml`, `rust-libxslt`, and `marpa-asf-sys` are authored and owned by the project maintainer on crates.io. WASI fixes land directly upstream (iterated via `[patch]`, then published), preserving full control. |
| **No Unmanaged Dependencies** | Reject dormant third-party crates (e.g. `sxd-xpath`) | `sxd-xpath` has unoptimized allocation-heavy tree-walking interpreters and zero upstream governance. Compiling `rust-libxml` gives native C performance with full ownership. |

---

## 2. In-Browser Runtime Model

When running client-side inside a browser (e.g., VS Code Web, Overleaf-like editors, documentation previews):

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Web Browser (Web Worker)                        │
│                                                                        │
│   JavaScript Application / Editor UI (e.g. Monaco, VS Code Web)        │
│                                │                                       │
│   ┌────────────────────────────▼───────────────────────────────────┐   │
│   │           `@bjorn3/browser_wasi_shim` (15 KB JS)              │   │
│   │   • Maps stdin/stdout/stderr to JS Uint8Arrays / Strings       │   │
│   │   • Provides in-memory virtual FS (TeX sources + stylesheets)  │   │
│   │   • Maps system clocks to performance.now()                    │   │
│   └────────────────────────────┬───────────────────────────────────┘   │
│                                │ WebAssembly.instantiate()             │
│   ┌────────────────────────────▼───────────────────────────────────┐   │
│   │               latexml.wasm (`wasm32-wasip1`)                   │   │
│   │   ┌──────────────────┐  ┌──────────────────┐  ┌─────────────┐  │   │
│   │   │  latexml-oxide   │  │   libxml2.a      │  │ libxslt.a   │  │   │
│   │   │  (Rust Engine)   │  │ (C XPath Engine) │  │  (C EXSLT)  │  │   │
│   │   └──────────────────┘  └──────────────────┘  └─────────────┘  │   │
│   └────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────────────┘
```

* **Zero UI Freezes:** Runs in a standard browser Web Worker.
* **Instant Parity:** HTML5 output generated inside Chrome/Safari is **byte-for-byte identical** to output generated on native Linux CLI.
* **Cached Footprint:** ~8–12 MB uncompressed, ~3–5 MB compressed (gzip/brotli), cached permanently in the browser after initial load.

---

## 3. Phased Implementation Roadmap

### Phase 0: Workspace Sandboxing & Platform Gating

*Objective:* Ensure all Rust code in `latexml-oxide` compiles cleanly for `target_arch = "wasm32"` without attempting forbidden host operations.

- [ ] **0.1 Gate Process Spawning (`Command::new`):**
  - `latexml_core/src/util/pathname.rs:258,291`: Gate ambient `kpsewhich` version & root probes with `#[cfg(not(target_arch = "wasm32"))]`; return `None` immediately on WASM.
  - `latexml_engine/src/dump_paths.rs:97,111`: Gate format year detection via `pdflatex`; fall back to built-in embedded dump headers without subprocesses.
  - `latexml_engine/src/lua_bridge.rs:185,315`: Gate LuaTeX execution (`texlua`); cleanly disable LuaTeX scripts on WASM with a non-fatal warning.
  - `latexml_package/src/package/line_fontmap.rs:95,103`: Catch `tftopl` / `kpsewhich` failure gracefully when extracting font metrics.
  - `latexml_post/src/graphics.rs:784+` & `latexml_post/src/latex_images.rs:449`: Disable subprocess rasterizers (`convert`, `gs`, `dvisvgm`). For HTML5, MathML is emitted directly; vector SVG and web image formats pass through untouched.
  - `latexml_oxide/src/render_workers.rs:385`: Enforce single-process execution in WASM mode.
- [ ] **0.2 Memory Allocator Selection:**
  - Gate `mimalloc` off on `target_arch = "wasm32"` via `#[cfg(not(target_arch = "wasm32"))]`. Allow Rust standard library's built-in `dlmalloc` to manage linear memory.
- [ ] **0.3 Concurrency & Watchdog Thread Gating:**
  - `latexml_oxide/src/cli/watchdog.rs`: Gate the OS watchdog background thread behind `#[cfg(not(target_arch = "wasm32"))]`. In the browser, execution is single-threaded in a Web Worker, and cancellation/timeout is handled by the host JavaScript via Worker termination or `AbortController`.
- [ ] **0.4 Stack Guard Recursion Protection:**
  - `latexml_core/src/stack_guard.rs`: On `target_arch = "wasm32"`, replace `stacker::maybe_grow` with a deterministic macro expansion depth counter (since WebAssembly cannot dynamically grow the native host execution stack).
- [ ] **0.5 Error Decoupling:**
  - Move `impl From<marpa::error::Error> for Error` from `latexml_core/src/common/error.rs` into `latexml_math_parser`, severing `latexml_core`'s unnecessary build-time link to `marpa`.

---

### Phase 1: Upstream Native C Dependencies (`wasi-sdk`)

*Objective:* Bring up static library builds of all three native C dependencies targeting `wasm32-wasip1`.

- [ ] **1.1 `marpa-asf-sys` (`dginev/marpa`):**
  - Update `build.rs` to detect `target_arch = "wasm32"` and compile with `wasi-sdk` Clang.
  - Libmarpa consists purely of obstack, AVL trees, and Earley tables with **zero OS system calls** (no sockets, no files, no threads). Compiles with minimal flags.
- [ ] **1.2 `rust-libxml` (`dginev/rust-libxml`):**
  - Add WASI configuration in `build.rs` to compile static `libxml2.a` with `wasi-sdk` Clang.
  - Configure flags: `--without-http --without-ftp --without-zlib --without-lzma --without-threads`.
  - Validate that in-memory parsing (`xmlReadMemory`) and DOM/XPath tree operations function cleanly.
- [ ] **1.3 `rust-libxslt` (`dginev/rust-libxslt`):**
  - Add WASI compilation for `libxslt.a` and `libexslt.a` linking against the WASI `libxml2.a`.
  - Verify EXSLT standard extension functions (`exsl:node-set()`, `func:function`, `set:distinct()`) compile cleanly.
- [ ] **1.4 `kpathsea-sys` Verification:**
  - Verify that `latexml_core::pathname` safely falls back to `KpathseaBackend::Unavailable` when unlinked on WASI.
- [ ] **1.5 Workspace Integration Verification:**
  - Wire upstream branches via `[patch.crates-io]` in root `Cargo.toml`.
  - Run `cargo check --target wasm32-wasip1 --workspace` to confirm clean compilation.

---

### Phase 2: In-Memory Virtual Assets & I/O Harness

*Objective:* Provide the engine with required TeX package macros and XSLT stylesheets inside the WASM sandbox.

- [ ] **2.1 Package & Binding Distribution:**
  - Package core distribution bindings (`.latexml`, `.sty`) into a bundle that can be loaded into `@bjorn3/browser_wasi_shim`'s `PreopenDirectory` virtual filesystem.
- [ ] **2.2 Stylesheet Distribution:**
  - Provide `LaTeXML-html5.xsl` and related transformation templates in the virtual filesystem so `latexml_post` can read them via standard file paths.
- [ ] **2.3 Virtual I/O Interface:**
  - Enable input reading via virtual `/input.tex` (or `stdin`) and write HTML5 output directly to `/output.html` (or `stdout`).

---

### Phase 3: In-Browser Verification & Demonstration

*Objective:* Validate end-to-end execution directly inside a web browser.

- [ ] **3.1 Create Browser Test Harness (`examples/wasm-browser/`):**
  - Create a minimal HTML/JS harness: `index.html` + `worker.js`.
  - Use `@bjorn3/browser_wasi_shim` to instantiate `latexml.wasm`.
  - Feed a test LaTeX document containing text, math formulae, and an environment.
- [ ] **3.2 End-to-End Parity Verification:**
  - Verify that MathML rendering and HTML5 structure generated in Chrome/Firefox match the desktop CLI output exactly.

---

### Phase 4: CI & Release Pipeline Integration

*Objective:* Automate WASM builds and prevent regressions.

- [ ] **4.1 GitHub Actions WASM CI:**
  - Add a dedicated CI workflow (`.github/workflows/wasm.yml`) that installs `wasi-sdk` and runs `cargo check --target wasm32-wasip1 --workspace`.
- [ ] **4.2 Release Artifact Packaging:**
  - In `release.yml`, produce optimized `latexml.wasm` using `wasm-opt -Oz --strip-debug` alongside the Linux, macOS, and Windows releases.

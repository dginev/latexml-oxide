//! Persistent server for editor/preview integration (`latexml_oxide --server`).
//!
//! This is a JSON-RPC-over-stdio server using LSP message framing. It speaks a
//! subset of LSP (`initialize`, `textDocument/did{Open,Change,Close}` →
//! `publishDiagnostics`, `shutdown`, `exit`) **plus** a custom
//! `latexml/convert` request that returns `{html, log, diagnostics, sources,
//! status, statusCode}` — the response shape the `ar5iv-editor` client
//! consumes for its live source↔preview loop (see `docs/SOURCE_PROVENANCE.md`).
//!
//! Performance model: the preamble (everything up to and including
//! `\begin{document}`) is digested once and cached in this (parent) process.
//! Each body conversion `fork()`s a child that inherits the warm post-preamble
//! state via copy-on-write, digests only the body, builds the DOM, and writes
//! the result back over a pipe before exiting. The child is a throwaway, so a
//! body conversion can never pollute the cache, and a panicking/looping body
//! can't take down the server.
//!
//! Concurrency model: a **single thread** drives everything. While a body
//! child runs, the parent `poll(2)`s `{stdin, child-pipe}`; a newer
//! `latexml/convert` for the same document `SIGKILL`s the in-flight child
//! (a pid we still own — reaped here, so no PID-recycle race) and supersedes
//! it. Keeping it single-threaded is also what makes the `fork()` safe: there
//! is no second thread that could hold the allocator lock at fork time.

mod diagnostics;
mod document;
#[cfg(not(unix))]
mod generic;
mod json;
mod protocol;
mod server;
#[cfg(unix)]
mod unix;

pub(crate) use diagnostics::*;
pub(crate) use document::*;
pub(crate) use json::*;
pub(crate) use protocol::*;
pub(crate) use server::*;

/// Run the server. `timeout_secs` is the per-conversion wall-clock budget
/// (`--timeout`; 0 disables) and `max_memory_mb` the resident-memory ceiling
/// (`--max-memory`; 0 disables). Both are applied **fresh per conversion** by
/// the forked body child's shared [`latexml_core::watchdog::Watchdog`] — so a
/// child never runs against the parent's stale warm-up deadline, and is reaped
/// if it exceeds the RAM ceiling. The extension surfaces both as VSCode
/// settings and passes them on the spawn.
pub fn run_lsp_server(
  timeout_secs: u64,
  max_memory_mb: u64,
) -> Result<(), Box<dyn std::error::Error>> {
  let max_rss_kb = max_memory_mb.saturating_mul(1024);
  #[cfg(unix)]
  {
    unix::run(timeout_secs, max_rss_kb)
  }
  #[cfg(not(unix))]
  {
    generic::run(timeout_secs, max_rss_kb)
  }
}

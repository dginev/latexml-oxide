//! Autoload the LaTeX format when an *undefined* control sequence turns out to
//! be one the LaTeX kernel defines.
//!
//! # Why this exists
//!
//! In real LaTeX there is no such thing as "before the kernel": `latex.ltx`
//! **is** the format, so every kernel command is live from token one. LaTeXML
//! (Perl and Rust alike) instead loads `LaTeX.pool` lazily, on first sight of a
//! *trigger* control sequence — the list ported from Perl `TeX.pool.ltxml`
//! L33-56 (`\documentclass`, `\newcommand`, `\begin`, …), installed in
//! `latexml_engine::tex`.
//!
//! A curated trigger list is incomplete by construction. Any kernel command
//! that is *not* on it — and a document may legitimately use one before
//! `\documentclass` — is simply undefined, gets an `<ltx:ERROR/>` stub, and the
//! document derails. The canonical case is the "use this class if installed"
//! idiom
//!
//! ```tex
//! \IfFileExists{proc-l.cls}{\documentclass{proc-l}}{\documentclass{amsproc}}
//! ```
//!
//! where the collapsed conditional means *no class is ever selected* and the
//! run cascades into `Fatal:TooManyErrors`. Same-host Perl LaTeXML fails
//! identically (see `docs/parity/KNOWN_PERL_ERRORS.md` — shared defect, upstream
//! candidate), so this is a "at parity, still a bug" fix rather than a
//! divergence repair.
//!
//! # What this module is
//!
//! The single funnel through which the undefined-CS paths ask "should the LaTeX
//! kernel be loaded for this token instead of erroring?". It holds no policy of
//! its own: the answer comes from a hook the engine registers at `TeX.pool`
//! load time ([`set_hook`]), because deciding it needs the kernel dump and the
//! pool loader, neither of which `latexml_core` owns.
//!
//! The eager Perl trigger list is *kept* — it fires on a legitimate use before
//! any error is raised, which this hook cannot do. This is the safety net
//! beneath it, not a replacement.
//!
//! # Call sites: two, deliberately not three
//!
//! `read_x_token`'s `Outcome::Undefined` arm and `invoke_token_undefined` are
//! the paths a CS reaches when it is actually being *used*. `read_balanced`'s
//! "cs SHOULD have defn by now; report early!" branch — the third
//! `generate_error_stub` caller, inside token-list scanning — is left alone on
//! purpose: it fires while collecting an `\edef`-style body rather than
//! executing it, and loading a format mid-scan buys a rare case
//! (`\edef\x{\IfFileExists…}` before `\documentclass`) at the price of running
//! the whole pool from inside a partially-read token list. Wire it up only with
//! a reproducer that needs it.

use std::sync::OnceLock;

use crate::token::Token;

/// Engine-supplied decision procedure for "is `token` a LaTeX kernel control
/// sequence, and if so load the kernel and report whether it is now defined".
///
/// Returning `true` means the caller must **retry** `token` (it now has a real
/// meaning); returning `false` means "carry on and report it undefined exactly
/// as before". Implementations own the once-only guard — see
/// `latexml_engine::latex_kernel::autoload_latex_kernel`.
pub type KernelAutoloadHook = fn(&Token) -> bool;

/// Process-global because the hook is a plain `fn` pointer with no state: the
/// per-session bookkeeping (already-loaded, already-attempted) lives in the
/// State, so re-registering across sessions is a no-op.
static HOOK: OnceLock<KernelAutoloadHook> = OnceLock::new();

/// Register the engine's kernel-autoload decision procedure. Called from
/// `TeX.pool`'s definition load, which precedes every conversion. Repeat calls
/// are ignored (the first registration wins).
pub fn set_hook(hook: KernelAutoloadHook) { let _ = HOOK.set(hook); }

/// Ask the registered hook whether `token` should pull the LaTeX kernel in.
///
/// `true` ⇒ the kernel was loaded *and* `token` now has a meaning, so the
/// caller must push it back and re-resolve. `false` ⇒ nothing happened; take
/// the ordinary bounded `Error:undefined` path.
///
/// Cold path only — every caller is a site that was already about to raise an
/// undefined-CS error. With no hook registered (a bare `latexml_core` embedding)
/// this is an atomic load and a `false`.
pub fn try_autoload(token: &Token) -> bool {
  match HOOK.get() {
    Some(hook) => hook(token),
    None => false,
  }
}

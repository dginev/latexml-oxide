use std::{
  cell::{RefCell, RefMut},
  collections::VecDeque,
};

use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashSet as HashSet;

// use std::mem;
// use std::rc::Rc;
use crate::alignment::Alignment;
use crate::{
  DigestedData,
  common::{
    arena::{self, SymStr},
    dimension::Dimension,
    error::*,
    float::Float,
    glue::{FillCode, Glue},
    locator::Locator,
    mudimension::MuDimension,
    muglue::MuGlue,
    number::Number,
    numeric_ops::{NumericOps, UNITY, fixpoint, fixpoint_unit},
    object::Object,
    store::Stored,
  },
  definition::{
    Definition,
    conditional::ConditionalType,
    register::{Register, RegisterType, RegisterValue},
  },
  mouth::Mouth,
  state::*,
  token::{Catcode, TOKEN_ENDCSNAME, TOKEN_RELAX, Token},
  tokens::Tokens,
};

static DIGIT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9]").unwrap());
static OCT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-7]").unwrap());
static HEX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9A-F]").unwrap());

/// Cached snapshot of `LXML_TRACE_GROUP_END` env var, sampled exactly
/// once per process. Inlining `std::env::var(...)` on the hot
/// `read_x_token` path was triggering SIGSEGVs in `__GI_getenv` under
/// concurrent test-thread execution: glibc's `getenv` walks the
/// process-global `environ` array unprotected, and the volume of
/// concurrent calls (millions/sec across N threads) made the unsafe
/// concurrent walks visible. The fix is to read the env var ONCE at
/// static-init time; subsequent checks are a free atomic load.
pub static TRACE_GROUP_END: Lazy<bool> =
  Lazy::new(|| std::env::var("LXML_TRACE_GROUP_END").is_ok());

// `\noexpand`'d tokens are represented per-token by the `\special_relax` family
// (`Token::is_noexpand_family` / `token::noexpand_family`): the shadowed token's
// identity is encoded in the CS name, so it survives storage and dumps without a
// global smuggle slot. The family resolves to `\relax` meaning via the
// `state::lookup_meaning` fallback. Faithful to TeX's `no_expand_flag`, which
// preserves the shadowed `cur_cs` while giving it relax meaning for one access.
use std::cell::Cell;

use crate::pin;

/// True when `token` is a `\noexpand`'d form (`\special_relax` family) shadowing
/// `target` — used by delimited-parameter / keyword matching so that, faithful to
/// TeX, a `\noexpand`'d token still matches its underlying identity.
fn special_relax_matches(token: &Token, target: &Token) -> bool {
  token.noexpand_shadowed().as_ref() == Some(target)
}
#[thread_local]
static DEFERRED_COMMANDS: Lazy<HashSet<SymStr>> = Lazy::new(|| {
  set!(
    pin!("\\the"),
    pin!("\\showthe"),
    pin!("\\unexpanded"),
    pin!("\\detokenize")
  )
});

// If it is a column ending token, Returns the token, a keyword and whether it is "hidden"
#[thread_local]
static COLUMN_ENDS: Lazy<[(Token, &'static str, bool); 6]> = Lazy::new(|| {
  [
    // besides T_ALIGN
    (T_CS!("\\cr"), "cr", false),
    (T_CS!("\\crcr"), "crcr", false),
    (T_CS!("\\lx@hidden@cr"), "cr", true),
    (T_CS!("\\lx@hidden@crcr"), "crcr", true),
    (T_CS!("\\lx@hidden@align"), "insert", true),
    (T_CS!("\\span"), "span", false),
  ]
});

#[derive(PartialEq, Debug)]
pub struct MouthRuntime {
  pub autoclose: bool,
  pub mouth:     Mouth,
  /// Pushback LIFO stack: the "next to read" token is at `pushback.last()`.
  /// Invariant: reading pops from the back; `unread_one` pushes to the back;
  /// `unread_vec` iterates its input in reverse and pushes each — so the
  /// first element of an unread Vec ends up on top (= next to read).
  /// See `flush_mouth` for the rare FIFO-prepend semantics (\endinput).
  ///
  /// Previously a `VecDeque<Token>` — switched to a plain Vec because the
  /// hot-path is pure LIFO and VecDeque's push_front/pop_front machinery
  /// (head-pointer + wrap arithmetic) showed up at ~3.3% of total Ir in
  /// callgrind on siunitx-heavy fixtures.
  pub pushback:  Vec<Token>,
}

#[derive(Debug, Default)]
pub struct Gullet {
  pub runtime:              Option<MouthRuntime>,
  pub mouthstack:           VecDeque<MouthRuntime>,
  pub pending_comments:     VecDeque<Token>,
  pub token_limit:          Option<usize>,
  pub pushback_limit:       Option<usize>,
  pub progress:             usize,
  /// Token-progress floor above which [`cycle_guard`](Self::cycle_guard)
  /// engages. Defaults to [`CYCLE_GUARD_ACTIVATE`] (20M). Graphics packages
  /// whose healthy expansion legitimately runs to 100M+ tokens (pgf/tikz/xy)
  /// raise it via [`raise_cycle_guard_activate`] so their streams stay out of
  /// the per-token fingerprint regime; the 400M `token_limit` remains the hard
  /// backstop. `#[derive(Default)]` would zero this (guard-always-on), so it is
  /// set explicitly in the constructor and the per-conversion reset.
  pub cycle_guard_activate: usize,
  /// Windowed cycle detector over the expansion (read-token) stream — catches
  /// small-period infinite expansion loops (`\def\x{a\x}` etc.) far earlier
  /// and more cheaply than `token_limit`/`pushback_limit`. Gated on a high
  /// `progress` so normal documents never touch it. See [`crate::cycle_guard`].
  pub cycle_guard:          crate::cycle_guard::CycleGuard,
  /// Reading-context serial, mixed into every cycle-guard fingerprint so that
  /// windows never match ACROSS `reading_from_mouth` contexts: a cycle is only
  /// a cycle within one expansion context. Each `reading_from_mouth` entry
  /// allocates a fresh serial (from `ctx_next`) and restores the outer one on
  /// exit (`ctx_stack`), so (a) consecutive IDENTICAL short expansions — the
  /// math0402448 xymatrix per-cell `get_xmarg_id` stream — get distinct
  /// serials and can never concatenate into a pseudo-periodic window (the
  /// false positive an earlier blanket `reset()` suppressed), while (b) an
  /// OUTER loop's tokens keep their serial across inner expansions, so a
  /// runaway whose body calls `do_expand` each iteration (~164 call sites)
  /// remains detectable — the blind spot the blanket reset had (PR #249
  /// review P2-7).
  pub ctx_serial:           u64,
  ctx_next:                 u64,
  ctx_stack:                Vec<u64>,
}

thread_local! {
  /// Debug-only (LATEXML_DEBUG_FATAL): ring of the most recent read tokens,
  /// dumped when the gullet cycle guard trips so the repeating window is
  /// identifiable from logs (this is how the math0402448 xymatrix
  /// false-positive was diagnosed).
  static DEBUG_RECENT_TOKENS: RefCell<VecDeque<String>> =
    RefCell::new(VecDeque::with_capacity(512));
}

/// Hoisted env probe for the LATEXML_DEBUG_FATAL diagnostics (shared seam in
/// `common::error`; read once so the per-token hot path pays one bool test).
static DEBUG_FATAL: Lazy<bool> = Lazy::new(debug_fatal_enabled);

#[thread_local]
pub static GULLET: Lazy<RefCell<Gullet>> = Lazy::new(|| {
  RefCell::new(Gullet {
    // Safety BACKSTOP against corrupted-macro-state loops (real runaways are cut
    // far earlier by the cycle guards / pushback limit / byte budget, so erring
    // high costs no detection latency). 400M = 5× the heaviest measured legit
    // paper (math0402448, amsart + xy-pic, 80.2M end-of-run progress under the
    // 2026-06-10 all-three-reader-loop accounting; the old "80M" figure predated
    // that multi-counting). `LATEXML_TOKEN_LIMIT` overrides (0 disables).
    token_limit: match std::env::var("LATEXML_TOKEN_LIMIT")
      .ok()
      .and_then(|v| v.parse::<usize>().ok())
    {
      Some(0) => None,
      Some(n) => Some(n),
      None => Some(400_000_000),
    },
    // Explicit: `#[derive(Default)]` would set this to 0 (guard active from the
    // first token). Graphics packages raise it at load (see
    // `raise_cycle_guard_activate`).
    cycle_guard_activate: CYCLE_GUARD_ACTIVATE,
    ..Gullet::default()
  })
});

/// Eagerly initialize this thread's gullet-phase `#[thread_local]` roots
/// (`DEFERRED_COMMANDS`, `COLUMN_ENDS`, `GULLET`). Their initializers intern
/// `SymStr`s / build `Token`s via the arena, so force them AFTER
/// [`arena::force_init`](crate::common::arena::force_init) /
/// [`token::force_init`](crate::token::force_init). Forcing them at
/// conversion entry keeps them from initializing re-entrantly from within
/// another root's init mid-conversion — the macOS `#[thread_local]` hazard
/// behind issue #217. No behavioral change on Linux.
pub(crate) fn force_init() {
  Lazy::force(&DEFERRED_COMMANDS);
  Lazy::force(&COLUMN_ENDS);
  Lazy::force(&GULLET);
}

macro_rules! gullet {
  () => {
    (*GULLET).borrow()
  };
}
macro_rules! gullet_mut {
  () => {
    (*GULLET).borrow_mut()
  };
}
/// Set the token limit and reset progress. Returns previous (limit, progress) for restoration.
pub fn set_token_limit(limit: Option<usize>) -> (Option<usize>, usize) {
  let mut g = gullet_mut!();
  let prev = (g.token_limit, g.progress);
  g.token_limit = limit;
  g.progress = 0;
  prev
}

/// Set the pushback limit (maximum pushback stack size before fatal error).
pub fn set_pushback_limit(limit: Option<usize>) { gullet_mut!().pushback_limit = limit; }

/// The conversion's final token-read progress (for end-of-run telemetry —
/// the calibration basis for `token_limit` / `CYCLE_GUARD_ACTIVATE`).
pub fn final_progress() -> usize { gullet!().progress }

/// Restore the token limit and progress from a previous set_token_limit call.
pub fn restore_token_limit(saved: (Option<usize>, usize)) {
  let mut g = gullet_mut!();
  g.token_limit = saved.0;
  g.progress = saved.1;
}

macro_rules! runtime {
  () => {
    (*GULLET).borrow_mut().runtime
  };
}
macro_rules! runtime_mut {
  () => {
    (*GULLET).borrow_mut().runtime.as_mut()
  };
}

/// Initialize (or reset, if reentrant) a Gullet to its default empty state
pub fn initialize_gullet() {
  let mut gullet = gullet_mut!();
  gullet.runtime = None;
  gullet.mouthstack = VecDeque::new();
  gullet.pending_comments = VecDeque::new();
  // Fresh per-conversion progress + cycle-guard history (the engine is a
  // thread-local singleton reused across conversions in the test harness).
  gullet.progress = 0;
  gullet.cycle_guard.reset();
  // Reset the Cluster F expansion-depth counter + re-read its env limit
  // (independent thread-locals, no GULLET borrow — safe to call here).
  reset_expand_depth();
  // Restore the default activation floor: a prior tikz/xy conversion in this
  // reused thread-local engine must not leak its raised floor into the next doc.
  gullet.cycle_guard_activate = CYCLE_GUARD_ACTIVATE;
  gullet.ctx_serial = 0;
  gullet.ctx_next = 0;
  gullet.ctx_stack.clear();
}

/// Get the current location of input getting read
pub fn get_locator() -> Locator {
  let gullet = gullet!();
  let mut runtime_opt = gullet.runtime.as_ref();
  let mut mouthstack_iter = gullet.mouthstack.iter();
  while runtime_opt.is_some() && runtime_opt.as_ref().unwrap().mouth.get_source().is_empty() {
    runtime_opt = mouthstack_iter.next();
  }
  // The free fn stays `-> Locator` ("where the parser is now" — always a real
  // position during digestion; the workhorse for errors + box creation). A
  // Mouth's `get_locator` is `Option` (per the `Object` trait) but is always
  // `Some`, so unwrap to the default only in the no-mouth backup.
  if let Some(runtime) = runtime_opt {
    // First exit condition: we found a mouth with a source, and asked it for a locator
    runtime.mouth.get_locator().unwrap_or_default()
  } else if let Some(runtime) = gullet.mouthstack.front() {
    // Backup strategy: return the first locator in the mouthstack:
    runtime.mouth.get_locator().unwrap_or_default()
  } else {
    // Final backup -- the default locator
    Locator::default()
  }
}

/// `get_locator`'s accurate-start sibling (§1, docs/SOURCE_PROVENANCE.md): same
/// mouthstack walk, but reads the mouth's `get_locator_from_start` (`from` = the
/// last token's captured start) instead of the heuristic `from`. Used for the
/// construct-START snapshot at constructor digest under `--source-map`.
pub fn get_locator_from_start() -> Locator {
  let gullet = gullet!();
  let mut runtime_opt = gullet.runtime.as_ref();
  let mut mouthstack_iter = gullet.mouthstack.iter();
  while runtime_opt.is_some() && runtime_opt.as_ref().unwrap().mouth.get_source().is_empty() {
    runtime_opt = mouthstack_iter.next();
  }
  if let Some(runtime) = runtime_opt {
    runtime.mouth.get_locator_from_start()
  } else if let Some(runtime) = gullet.mouthstack.front() {
    runtime.mouth.get_locator_from_start()
  } else {
    Locator::default()
  }
}

/// Comment-oriented location string, based on `get_locator`
pub fn get_location() -> String {
  let loc = get_locator();
  s!("at {}", loc)
}

pub fn mouth_is_open(mouth: &Mouth) -> bool {
  let gullet = gullet!();
  if let Some(ref runtime) = gullet.runtime
    && mouth == &runtime.mouth
  {
    return true;
  }
  gullet
    .mouthstack
    .iter()
    .any(|runtime| &runtime.mouth == mouth)
}

/// Push the `tokens` back into the input stream to be re-read.
pub fn unread(tokens: Tokens) { unread_vec(tokens.unlist()); }
/// Variant of `unread`, but drains the contents of `tokens` without taking ownership.
pub fn unread_mut(tokens: &mut Tokens) {
  if let Some(ref mut runtime) = gullet_mut!().runtime {
    // Iterate in reverse and push to the stack top — the first element
    // of `tokens` ends up on top (= next to read). Same semantics as
    // the old VecDeque push_front pattern.
    for token in tokens.unlist_mut().drain(..).rev() {
      runtime.pushback.push(token);
    }
  };
}
/// Unreads a single `Token` to the start of the token stream.
/// Perl: unread() always adjusts $ALIGN_STATE when unreading { or } tokens.
pub fn unread_one(token: Token) {
  match token.get_catcode() {
    Catcode::BEGIN => decrement_align_group_count(), // Retract scanned brace
    Catcode::END => increment_align_group_count(),
    _ => {},
  }
  if let Some(ref mut runtime) = gullet_mut!().runtime {
    runtime.pushback.push(token);
  };
}
/// Unreads a `Vec<Token>` to the start of the token stream
/// Perl: also adjusts ALIGN_STATE by retracting scanned braces (Gullet.pm lines 343-358)
pub fn unread_vec(tokens: Vec<Token>) {
  let mut level: i64 = 0;
  if let Some(ref mut runtime) = gullet_mut!().runtime {
    // Reserve once, push each token in reverse-iteration order so the
    // first element of `tokens` ends up at the stack top. Same
    // semantics as the old VecDeque push_front loop, but without
    // per-element head-pointer arithmetic.
    runtime.pushback.reserve(tokens.len());
    for token in tokens.into_iter().rev() {
      match token.get_catcode() {
        Catcode::BEGIN => level -= 1, // Retract scanned braces
        Catcode::END => level += 1,
        _ => {},
      }
      runtime.pushback.push(token);
    }
  }
  if level != 0 {
    set_align_group_count(align_group_count() + level as i32);
  }
}

//**********************************************************************
// Start reading tokens from a new Mouth.
// This pushes the mouth as the current source that $gullet->readToken (etc) will read from.
// Once this Mouth has been exhausted, readToken, etc, will return undef,
// until you call $gullet->closeMouth to clear the source.
// Exception: if $toplevel=1, readXToken will step to next source
// Note that a Tokens can act as a Mouth.
pub fn open_mouth(mouth: Mouth, autoclose: bool) {
  let mut gullet = gullet_mut!();
  if let Some(runtime) = gullet.runtime.take() {
    gullet.mouthstack.push_front(runtime);
  };
  gullet.runtime = Some(MouthRuntime {
    mouth,
    autoclose,
    pushback: Vec::with_capacity(128),
  });
}

pub fn close_mouth(forced: bool) -> Result<()> {
  let mut shift_from_mouthstack = false;
  let mut error_has_more_input = false;
  if let Some(ref mut runtime) = runtime!()
    && !forced
    && (!runtime.pushback.is_empty() || runtime.mouth.has_more_input())
  {
    error_has_more_input = true
  }
  if error_has_more_input {
    let next = match read_token()? {
      Some(t) => t.stringify(),
      None => String::from("Empty"),
    };
    let message = s!("Closing mouth with input remaining '{}'", next);
    Error!("unexpected", next, message);
  }
  let mut gullet = gullet_mut!();
  if let Some(ref mut runtime) = gullet.runtime {
    runtime.mouth.finish();
    shift_from_mouthstack = true;
  }
  if shift_from_mouthstack {
    gullet.runtime = gullet.mouthstack.pop_front();
  }
  Ok(())
}
/// This flushes a mouth so that it will be automatically closed, next time it's read
/// Corresponds to TeX's \endinput
pub fn flush_mouth() {
  if let Some(ref mut runtime) = runtime!() {
    // Collect remaining mouth tokens in mouth order (t1, t2, t3, …),
    // then splice them into the stack's BOTTOM in reverse order so
    // that after the stack's existing top is popped, the mouth tokens
    // come out in the original mouth order (t1 first, then t2, …).
    let mut trailer: Vec<Token> = Vec::new();
    while !runtime.mouth.is_eol() {
      if let Some(token) = runtime.mouth.read_token() {
        trailer.push(token);
      }
    }
    if !trailer.is_empty() {
      trailer.reverse();
      runtime.pushback.splice(0..0, trailer);
    }
    // Stop reading (clear buffers, close file) but do NOT restore catcodes.
    // Catcodes are restored by close_mouth → finish() when the mouth is
    // properly popped from the stack.
    runtime.mouth.stop_reading();
  }
}

//**********************************************************************
// Low-level readers: read token, read expanded token
//**********************************************************************
// # Get the next pending comment token (if any)
pub fn get_pending_comment() -> Option<Token> { gullet_mut!().pending_comments.pop_front() }

/// Note that every char (token) comes through here (maybe even twice, through args parsing),
/// So, be Fast & Clean!  This method only reads from the current input stream (Mouth).
fn handle_template(
  mut alignment: RefMut<Alignment>,
  token: Token,
  vtype: &str,
  hidden: bool,
) -> Result<()> {
  //  Append expansion to end!?!?!?!
  local_current_token(token);
  let post = alignment.get_column_after();
  set_align_group_count(1000000);
  // ### NOTE: Truly fishy smuggling w/ \lx@hidden@cr
  let arg_opt = if (vtype == "cr") && hidden {
    // \lx@hidden@cr gets an argument as payload!!!!!
    Some(read_arg(ExpansionLevel::Off)?)
  } else {
    None
  };
  // eprintln!("Halign: column after {post}");// . ToString($post) if $LaTeXML::DEBUG{halign};
  if (vtype == "cr" || vtype == "crcr")
    && alignment.is_in_row()
    && !alignment
      .current_row()
      .map(|v| v.is_pseudo())
      .unwrap_or(false)
  {
    unread_one(T_CS!("\\lx@alignment@row@after"));
  }
  if let Some(arg) = arg_opt {
    // slippery - to unread {arg} we first unread } then arg then {, as we push to the front.
    unread_one(T_END!());
    unread(arg);
    unread_one(T_BEGIN!());
  }
  unread_one(token);
  unread(post);
  expire_current_token();
  Ok(())
}

// internal low-level reader that extracts a token from a mouth,
// but always keeps comment tokens pending.
fn read_internal_token() -> Option<Token> {
  let mut next_token = None;
  let Gullet {
    ref mut runtime,
    ref mut pending_comments,
    ..
  } = *gullet_mut!();
  // Defensive: gullet runtime can be None during early shutdown /
  // recovery from a fatal error. Treat as "no more tokens" instead
  // of panicking. Driver: 2404.06289 (natbib \NAT@@wrout cascade
  // landed here after the conversion was already in error-recovery
  // mode).
  let rt = runtime.as_mut()?;
  let pushback = &mut rt.pushback;
  // Check in pushback first....
  while let Some(pushback_token) = pushback.pop() {
    match pushback_token.get_catcode() {
      Catcode::COMMENT => pending_comments.push_back(pushback_token),
      Catcode::MARKER => handle_marker(pushback_token),
      _ => {
        next_token = Some(pushback_token);
        break;
      },
    };
  }
  // Not in pushback, read from the current Mouth
  if next_token.is_none() {
    while let Some(token) = runtime.as_mut().unwrap().mouth.read_token() {
      match token.get_catcode() {
        Catcode::COMMENT => pending_comments.push_back(token),
        Catcode::MARKER => handle_marker(token),
        _ => {
          next_token = Some(token);
          break;
        },
      };
    }
  }
  next_token
}

/// Per-token-read resource checkpoint — runtime probe, progress/token-limit
/// accounting, pushback-limit probe, and cycle-guard activation — in ONE
/// mutable borrow. Returns `Ok(None)` when the gullet has no runtime (caller
/// returns end-of-input), else `Ok(Some(cycle_guard_active))` for the caller to
/// pass to [`cycle_guard_checkpoint`] (single-borrow fast path, PR #249 P2-9).
///
/// Shared by all FOUR reader loops (`read_token`, `read_x_token`,
/// `read_balanced`, `read_next_conditional`) — siblings over
/// `read_internal_token`, NOT a delegation chain, so each must run its own or
/// full-expansion paths (`\edef`, csname, conditional skipping) bypass every
/// gullet guard and a runaway grinds to the watchdog (gap found on math0402448).
#[inline]
fn read_resource_checkpoint() -> Result<Option<bool>> {
  let mut g = gullet_mut!();
  if g.runtime.is_none() {
    return Ok(None);
  }
  // Progress counts UNCONDITIONALLY: the cycle guard's activation gate
  // (`progress > CYCLE_GUARD_ACTIVATE`) feeds off it, so nesting the
  // increment inside the token-limit branch made `LATEXML_TOKEN_LIMIT=0`
  // (and `set_token_limit(None)` during format init) silently disable the
  // cycle guard as well — exactly when an operator disables the limit to
  // let a big document through is when the loop guard matters most.
  // (PR #249 review P2-5.) Only the limit COMPARISON stays conditional.
  g.progress += 1;
  if let Some(limit) = g.token_limit
    && g.progress > limit
  {
    let msg = s!("Token limit of {} exceeded, infinite loop?", limit);
    drop(g);
    Fatal!(Timeout, TokenLimit, msg);
  }
  if let Some(limit) = g.pushback_limit {
    let pb_len = g.runtime.as_ref().map(|r| r.pushback.len()).unwrap_or(0);
    if pb_len > limit {
      // Diagnostic: the looping token window is right here in the
      // pushback — dump its head so the cycle is identifiable from logs.
      if *DEBUG_FATAL && let Some(rt) = g.runtime.as_ref() {
        let head: Vec<String> = rt
          .pushback
          .iter()
          .take(48)
          .map(|t| format!("{t:?}"))
          .collect();
        eprintln!("[debug-fatal] pushback head: {}", head.join(" "));
      }
      let msg = s!("Pushback limit of {} exceeded, infinite loop?", limit);
      drop(g);
      Fatal!(Timeout, PushbackLimit, msg);
    }
  }
  Ok(Some(g.progress > g.cycle_guard_activate))
}

/// Windowed cycle-guard checkpoint over the token-read stream. Only engaged
/// once `progress` is already pathologically high — the caller passes the
/// activation state computed by [`read_resource_checkpoint`] in the SAME
/// borrow, so below the gate this costs one branch and zero borrows (and no
/// fingerprint hash). A small-period infinite expansion loop then trips a
/// clean Fatal in O(window) extra tokens instead of grinding to the token
/// limit (and the gigabytes of RSS that implies). Shared by all four reader
/// loops (see [`read_resource_checkpoint`] on why each loop needs its own
/// checkpoints — they do NOT delegate to one another).
#[inline]
fn cycle_guard_checkpoint(active: bool, nextt: &Token) -> Result<()> {
  if !active {
    return Ok(());
  }
  if *DEBUG_FATAL {
    DEBUG_RECENT_TOKENS.with(|ring| {
      let mut ring = ring.borrow_mut();
      if ring.len() >= 512 {
        ring.pop_front();
      }
      ring.push_back(format!("{nextt:?}"));
    });
  }
  let mut g = gullet_mut!();
  {
    // Mix the reading-context serial into the fingerprint (see the
    // `Gullet::ctx_serial` field doc): tokens read in different
    // `reading_from_mouth` contexts can then never form a matching window,
    // scoping cycle detection to ONE expansion context without destroying
    // the outer context's history. The multiplier spreads the serial across
    // the hash bits (splitmix64's odd constant).
    let fp = nextt.cycle_fingerprint() ^ g.ctx_serial.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    if let Some(period) = g.cycle_guard.push(fp) {
      drop(g);
      let msg = s!(
        "Infinite expansion loop: a window of {} token(s) repeated {}+ times",
        period,
        crate::cycle_guard::REPEAT
      );
      if *DEBUG_FATAL {
        eprintln!("[debug-fatal] gullet cycle guard tripping on token {nextt:?}: {msg}");
        DEBUG_RECENT_TOKENS.with(|ring| {
          let ring = ring.borrow();
          let recent: Vec<&str> = ring.iter().map(String::as_str).collect();
          eprintln!("[debug-fatal] last 512 read tokens: {}", recent.join(" "));
        });
      }
      Fatal!(Timeout, Recursion, msg);
    }
  }
  Ok(())
}

/// Read a token that the calling macro/primitive REQUIRES, holding the
/// "argument expected but input ended" diagnostic in one place.
///
/// `read_token()` returning `None` (input exhausted) is a normal, expected
/// control-flow signal in most contexts (end of file/group/optional-arg scan) —
/// so the primitive deliberately keeps the `Option`. But for a caller that
/// genuinely requires a token here, `None` is TeX's *"File ended while scanning
/// use of \cs"* error state (real `pdftex` raises an `! Emergency stop`). This
/// helper emits that parity `Error!` once, centrally, instead of every call site
/// `.unwrap()`-panicking on the `None`. It STILL returns the `Option` (it does
/// NOT fabricate a token), so the type system keeps each caller honest about how
/// it degrades — close its group, substitute a default, etc.
pub fn read_token_required(what: &str) -> Result<Option<Token>> {
  let tok = read_token()?;
  if tok.is_none() {
    Error!(
      "expected",
      what,
      format!("input ended while scanning use of {what}")
    );
  }
  Ok(tok)
}

pub fn read_token() -> Result<Option<Token>> {
  let mut next_token: Option<Token>;
  loop {
    // Single combined borrow: runtime probe + limits + activation state.
    let guard_active = match read_resource_checkpoint()? {
      None => return Ok(None),
      Some(active) => active,
    };
    // internal low-level reader that extracts a token from a mouth,
    // but always keeps comment tokens pending.
    next_token = read_internal_token();
    // ProgressStep() if ($$self{progress}++ % $TOKEN_PROGRESS_QUANTUM) == 0;

    // Strict-Perl translation of Gullet.pm `readToken`:
    //   alignment column-end check → \dont_expand check → break
    //   ALIGN_STATE tracking happens AFTER the loop, on the FINAL
    //   token to be returned (Perl L320-324).
    if let Some(ref nextt) = next_token {
      // Uniform placement across all reader loops: fingerprint the token AS
      // READ (before alignment/\dont_expand special-casing), so the cycle
      // guard sees the same stream regardless of which loop reads it.
      cycle_guard_checkpoint(guard_active, nextt)?;
      if (align_group_count() == 0)
        && has_reading_alignment()
        && let Some((atoken, atype, ahidden)) = is_column_end(nextt)
      {
        let reading_alignment = get_reading_alignment().unwrap();
        if let DigestedData::Alignment(data) = reading_alignment.data() {
          handle_template(data.borrow_mut(), atoken, atype, ahidden)?;
        } else {
          return Err("reading_alignment should always contain DigestedData::Alignment".into());
        }
        continue; // Perl: handleTemplate then continue while(1) loop
      }
      if nextt.code == Catcode::CS && nextt.text == pin!("\\dont_expand") {
        // `\noexpand <tok>`: collapse to the per-token `\special_relax` family,
        // encoding <tok>'s identity in the name (faithful to TeX's
        // `no_expand_flag`, which keeps `cur_cs`). End-of-input ⇒ bare
        // `\special_relax` (nothing to shadow).
        next_token = Some(match read_token()? {
          Some(tok) => crate::token::noexpand_family(&tok),
          None => T_CS!("\\special_relax"),
        });
      }
      break;
    } else {
      break;
    }
  }
  // Perl Gullet.pm L320-324: ALIGN_STATE tracking happens AFTER the loop,
  // applied only to the FINAL returned token. Previously this was inside
  // the loop BEFORE the alignment check, which prevented column-end
  // template handling on `{` tokens (count became 1 before the check).
  if let Some(ref nextt) = next_token {
    match nextt.get_catcode() {
      Catcode::BEGIN => increment_align_group_count(),
      Catcode::END => decrement_align_group_count(),
      _ => {},
    }
  }
  Ok(next_token)
}

/// Engage the expansion-stream cycle guard only after this many tokens — above
/// the ordinary range (measured known-good papers 0.6–7.5M under the 2026-06-10
/// all-three-loop accounting; 20M keeps them fingerprint-free with ~2.7×
/// headroom), so only a runaway (heading for the 400M `token_limit` / RSS cap)
/// records fingerprints, cut off in O(window) tokens (false positives guarded by
/// the period-`REPEAT` requirement, not this bound). DEFAULT floor; graphics-heavy
/// packages legitimately reach ~100–155M (math0402448 xy-pic, 1805.03265 tikz-cd)
/// and raise it to [`CYCLE_GUARD_ACTIVATE_GRAPHICS`] at load via
/// [`raise_cycle_guard_activate`], the 400M `token_limit` staying the backstop.
const CYCLE_GUARD_ACTIVATE: usize = 20_000_000;

/// Cycle-guard activation floor for graphics-heavy bindings (pgf/tikz/xy).
/// These packages legitimately expand 100M+ tokens; this floor sits above the
/// heaviest measured healthy graphics doc (1805.03265 tikz-cd ~155M) so they
/// stay out of the per-token fingerprint regime, while remaining far below the
/// 400M `token_limit` backstop. Raised — never lowered — per [`raise_cycle_guard_activate`].
pub const CYCLE_GUARD_ACTIVATE_GRAPHICS: usize = 150_000_000;

/// Raise the cycle-guard activation floor for the current (thread-local) gullet,
/// only ever upward. Called from graphics package bindings (pgf/tikz/xy) whose
/// healthy expansion runs to 100M+ tokens — see [`CYCLE_GUARD_ACTIVATE_GRAPHICS`].
/// Idempotent and order-independent: loading several graphics packages just
/// re-asserts the same floor. The per-conversion reset (`initialize_gullet`)
/// restores the default so the raise does not leak across documents.
pub fn raise_cycle_guard_activate(floor: usize) {
  let mut g = gullet_mut!();
  if floor > g.cycle_guard_activate {
    g.cycle_guard_activate = floor;
  }
}

// Cluster F: bound gullet expansion-recursion depth (= `read_x_token`
// re-entrancy) so a runaway — an xint number-arg chain, a self-referential
// `\csname`/`\number`/`\romannumeral` — raises a fast `Fatal:Timeout:Recursion`
// rather than grinding to the watchdog / RSS fuse. Legit docs nest ≲20; the cap
// is 12_000. Env override `LATEXML_EXPAND_DEPTH_LIMIT` (0 disables).
#[thread_local]
static EXPAND_DEPTH: Cell<usize> = Cell::new(0);
#[thread_local]
static EXPAND_DEPTH_LIMIT: Cell<usize> = Cell::new(12_000);

/// Reset the counter + re-read the env limit each conversion (the thread-local
/// engine is reused; a caught unwind could otherwise leave the counter high).
fn reset_expand_depth() {
  EXPAND_DEPTH.set(0);
  EXPAND_DEPTH_LIMIT.set(
    std::env::var("LATEXML_EXPAND_DEPTH_LIMIT")
      .ok()
      .and_then(|v| v.trim().parse().ok())
      .unwrap_or(12_000),
  );
}

/// RAII depth counter for `read_x_token`: `enter` increments (Fatals past the
/// limit), drop decrements — so every return path stays balanced.
struct ExpandDepthGuard;
impl ExpandDepthGuard {
  #[inline]
  fn enter() -> Result<ExpandDepthGuard> {
    let d = EXPAND_DEPTH.get() + 1;
    EXPAND_DEPTH.set(d);
    let limit = EXPAND_DEPTH_LIMIT.get();
    if limit != 0 && d > limit {
      EXPAND_DEPTH.set(d - 1); // Drop won't run — decrement here.
      Fatal!(
        Timeout,
        Recursion,
        format!("Excessive expansion recursion (depth {d} > {limit}); infinite macro loop?")
      );
    }
    Ok(ExpandDepthGuard)
  }
}
impl Drop for ExpandDepthGuard {
  #[inline]
  fn drop(&mut self) { EXPAND_DEPTH.set(EXPAND_DEPTH.get().saturating_sub(1)); }
}

/// Read the next non-expandable token, expanding until one appears. Hot path —
/// `read_token` is folded in. `toplevel` (default true): on mouth exhaustion,
/// step to the containing mouth. `fully_expand` (default = toplevel): expand
/// even protected defns ("for execution"). Unlike `read_balanced`, does NOT
/// defer `\the` & friends; `\noexpand`'d tokens act like `\relax`. For `\if`/
/// `\ifx` arguments pass `for_conditional=true` (handles `\noexpand` and CS
/// `\let` to tokens specially).
pub fn read_x_token(
  toplevel_opt: Option<bool>,
  for_conditional: bool,
  fully_expand_opt: Option<bool>,
) -> Result<Option<Token>> {
  // toplevel should be true by default
  let toplevel = toplevel_opt.unwrap_or(true);
  let fully_expand = fully_expand_opt.unwrap_or(toplevel);
  let _depth_guard = ExpandDepthGuard::enter()?; // Cluster F expansion-depth cap
  loop {
    // Resource + cycle checkpoints: this loop reads via the low-level
    // `read_internal_token` (NOT `read_token`), so it must run the same
    // guards itself — otherwise every full-expansion read path (`\edef`,
    // `read_balanced`, csname construction) bypasses the token/pushback
    // limits and the expansion cycle guard entirely, and a `\def\x{a\x}`
    // runaway grinds to the multi-GB watchdog instead of a clean Fatal.
    // (One combined borrow; the returned activation state feeds the cycle
    // checkpoint below so the fast path pays a single RefCell borrow.)
    let guard_active = read_resource_checkpoint()?.unwrap_or(false);
    // internal low-level reader that extracts a token from a mouth,
    // but always keeps comment tokens pending.
    let next_token = read_internal_token();
    //ProgressStep() if ($$self{progress}++ % $TOKEN_PROGRESS_QUANTUM) == 0;
    if next_token.is_none() {
      {
        let gullet = gullet!();
        let current_is_autoclose = gullet
          .runtime
          .as_ref()
          .map(|r| r.autoclose)
          .unwrap_or(false);
        // Drain a *transparent autoclose injection* (\scantokens, raw_tex) and
        // resume the enclosing mouth even for a BOUNDED reader (toplevel==false,
        // e.g. the InputDefinitions file loop) — these are part of the current
        // logical stream. Faithful to tex.web `get_next` §362-365 (exhausting any
        // input level resumes the enclosing one; \scantokens is a pseudo-file
        // level). DIVERGES from Perl `Gullet.pm` readXToken, which gates on
        // `autoclose = toplevel` and so returns at the first exhausted mouth —
        // truncating a `.sty` whenever `\scantokens` runs mid-load (witness
        // 1906.03240: real babel.sty `\selectlanguage` dropped every later def →
        // undefined-CS cascade; Perl's hand-written babel.ltxml dodges it).
        // A non-autoclose boundary is left to its owner (return None).
        if !current_is_autoclose || gullet.mouthstack.is_empty() {
          return Ok(None);
        }
      }
      close_mouth(false)?; // Drain the autoclose injection; resume the parent.
      continue;
    }
    // we got a token
    let token = next_token.unwrap();
    cycle_guard_checkpoint(guard_active, &token)?;
    if token.get_catcode() == Catcode::CS && token.text == pin!("\\dont_expand") {
      let unexpanded = match read_token()? {
        Some(t) => t,
        None => return Ok(Some(T_CS!("\\special_relax"))), // \dont_expand at end-of-input
      };
      if for_conditional && unexpanded.code == Catcode::ACTIVE {
        return Ok(Some(unexpanded));
      } else {
        // `\noexpand <tok>`: per-token `\special_relax` family encoding <tok>'s
        // identity in the name — faithful to TeX's `no_expand_flag`, which keeps
        // `cur_cs` while giving relax meaning for this one access. (Perl
        // readXToken returns a bare `\special_relax`, dropping the identity;
        // recovering it is a deliberate, SURPASS-PERL fidelity fix so a
        // `\noexpand`'d delimiter — e.g. xint's `\XINTfstop`, witness
        // 1804.01117 — survives a number/macro scan for the surrounding parser.)
        return Ok(Some(crate::token::noexpand_family(&unexpanded)));
      }
    }
    // Wow!!!!! See TeX the Program \S 309
    // SHOULD count nesting of { }!!! when SCANNED (not digested)
    let check_alignment_data = {
      if has_reading_alignment() && align_group_count() == 0 {
        if let Some((_atoken, atype, ahidden)) = is_column_end(&token) {
          let reading_alignment = get_reading_alignment().unwrap();
          Some((reading_alignment, atype, ahidden))
        } else {
          None
        }
      } else {
        None
      }
    };
    if let Some((reading_alignment, atype, ahidden)) = check_alignment_data {
      if let DigestedData::Alignment(data) = reading_alignment.data() {
        handle_template(data.borrow_mut(), token, atype, ahidden)?;
      } else {
        panic!("malformed alignmed was stored?");
      }
      // And *then* continue the main loop checks
    } else if token.get_catcode().is_active_or_cs() {
      // Read the meaning via closure so we can branch on the borrowed
      // Stored without cloning (Stored::clone was ~1% of total on
      // siunitx-heavy profiles; this site fires on every CS/ACTIVE
      // expansion — the hottest lookup_meaning caller).
      enum Outcome {
        LetTo(Token),
        Undefined,
        NonExpandable,
        Invoke(std::rc::Rc<dyn Definition>),
      }
      let outcome = with_meaning(&token, |defn_opt| match defn_opt {
        Some(Stored::Token(t)) => Outcome::LetTo(*t),
        Some(Stored::None) | None => Outcome::Undefined,
        Some(other) => match other.to_definition() {
          Some(defn) => {
            if !defn.is_expandable() || (defn.is_protected() && !fully_expand) {
              Outcome::NonExpandable
            } else {
              Outcome::Invoke(defn)
            }
          },
          None => Outcome::Undefined,
        },
      });
      match outcome {
        Outcome::LetTo(let_token) => {
          return Ok(Some(if for_conditional { let_token } else { token }));
        },
        Outcome::Undefined => {
          if token.get_catcode() == Catcode::CS {
            return Ok(Some(generate_error_stub(&token)?));
          } else {
            return Ok(Some(token));
          }
        },
        Outcome::NonExpandable => {
          return Ok(Some(token));
        },
        Outcome::Invoke(defn) => {
          local_current_token(token);
          // Grow the native stack ahead of deep expansion recursion (xint
          // `\XINT_…` number-arg chains nest tens of thousands deep) so
          // finite-deep recursion completes instead of overflowing the conversion
          // thread's 256 MB stack → SIGABRT (Perl degrades via `$MAXSTACK`). This
          // only grows the stack; the depth CAP is `ExpandDepthGuard` at the top
          // of `read_x_token`. Same idiom as the recursive walks in `document.rs`
          // / the math parser; params in `crate::stack_guard`.
          #[cfg_attr(not(feature = "token-locators"), allow(unused_mut))]
          let mut invoked = crate::stack_guard::maybe_grow(|| defn.invoke(false))?;
          // token-locators: fill-only origin inheritance. A macro that
          // expands into synthesized tokens with no origin — e.g.
          // `\today → ExplodeText!(Today!())` yielding "May 25, 2026" —
          // would leave its output unlocatable. Attribute such tokens to
          // the invocation site so the rendered text is source-mapped.
          // The inherited handle is flagged `inherited` (one push per
          // expansion, shared by every result token), so `child_span`'s
          // genuine-origin-first scan never lets a macro's structural body
          // literals widen its arguments' content-exact span. We also never
          // overwrite a token that already carries an origin. See
          // SOURCE_PROVENANCE.md §3.1.3.
          #[cfg(feature = "token-locators")]
          {
            let inv_loc = token.loc;
            if inv_loc != 0 {
              let mut inherited = 0u32;
              for t in invoked.unlist_mut() {
                if t.loc == 0 {
                  if inherited == 0 {
                    inherited = crate::token::push_inherited_origin(inv_loc);
                  }
                  t.loc = inherited;
                }
              }
            }
          }
          if *TRACE_GROUP_END {
            // Print per-event {macro, delta} so post-processing can sum
            // by-macro to find which expandable CS contributes net +/- 1
            // imbalance across the run. Format: TRACE_GE delta CS
            let (mut begs, mut ends) = (0, 0);
            for t in invoked.unlist_ref() {
              if *t == T_CS!("\\group_begin:") || *t == T_CS!("\\begingroup") {
                begs += 1;
              } else if *t == T_CS!("\\group_end:") || *t == T_CS!("\\endgroup") {
                ends += 1;
              }
            }
            if begs > 0 || ends > 0 {
              eprintln!(
                "TRACE_GE delta={} begs={} ends={} cs={}",
                begs - ends,
                begs,
                ends,
                token
              );
            }
          }
          unread(invoked);
          expire_current_token();
          continue;
        },
      }
    } else {
      // Perl Gullet.pm L421-422: track { and } at scan level for ALIGN_STATE
      match token.get_catcode() {
        Catcode::BEGIN => increment_align_group_count(),
        Catcode::END => decrement_align_group_count(),
        _ => {},
      }
      return Ok(Some(token));
    }
  }
}

/// Read the next raw line (string);
/// primarily to read from the Mouth, but keep any unread input!
pub fn read_raw_line() -> Option<String> {
  // If we've got unread tokens, they presumably should come before the Mouth's raw data
  // but we'll convert them back to string.
  let mut gullet = gullet_mut!();
  if let Some(ref mut runtime) = gullet.runtime {
    // Vec-as-stack stores bottom-to-top, but the caller expects
    // "next to read" first — reverse the drained order to match the
    // old VecDeque drain(..) which was front-to-back (= next-to-read).
    let tokens: Vec<Token> = runtime.pushback.drain(..).rev().collect();

    // TODO
    // let markers : Vec<&Token> = tokens.iter().filter(|t:Token| t.get_catcode() ==
    // Catcode::MARKER).collect(); if !markers.is_empty() {    // Whoops, profiling markers!

    // @tokens = grep { $_->getCatcode != Catcode::MARKER } @tokens;    // Remove
    // map { LaTeXML::Core::Definition::stopProfiling($_, 'expand') } @markers;
    // }

    // If we still have peeked tokens, we ONLY want to combine it with the remainder
    // of the current line from the Mouth (NOT reading a new line)
    if !tokens.is_empty() {
      Some(Tokens::new(tokens).to_string() + &runtime.mouth.read_raw_line(true).unwrap_or_default())
    } else {
      // Otherwise, read the next line from the Mouth.
      runtime.mouth.read_raw_line(false)
    }
  } else {
    None
  }
}

//**********************************************************************
// Mid-level readers: checking and matching tokens, strings etc.
//**********************************************************************
// The following higher-level parsing methods are built upon readToken & `.

/// Read a single non-space token
pub fn read_non_space() -> Result<Option<Token>> {
  loop {
    match read_token()? {
      None => return Ok(None),
      Some(t) => {
        if t.get_catcode() != Catcode::SPACE {
          return Ok(Some(t));
        }
      },
    }
  }
}

/// Read a single expanded, non-space, token
pub fn read_x_non_space() -> Result<Option<Token>> {
  loop {
    match read_x_token(Some(false), false, None)? {
      None => return Ok(None),
      Some(t) => {
        if t.get_catcode() != Catcode::SPACE {
          return Ok(Some(t));
        }
      },
    }
  }
}

/// A directive describing to what degree a gullet reader should perform TeX's expansion
#[derive(Copy, Debug, Clone, PartialEq, Default)]
pub enum ExpansionLevel {
  // No expansion, reads currently present tokens
  #[default]
  Off,
  /// Expands while reading, but deferring `\the` and `\protected`
  Partial,
  /// Expands completely while reading
  Full,
}

/// Approximates TeX's scan_toks (but doesn't parse \def parameter lists)
/// and only optionally requires the openning "{".
///
/// It may return comments in the token lists.
/// The `is_macrodef` flag affects whether # parameters are "packed" for macro bodies.
/// If `require_open` is true, the opening T_BEGIN has not yet been read, and is required.
///
/// If `toplevel` is true, it will automatically close empty mouths as it reads,
/// and will also fully expand macros (unless overridden by `expansion_level` being explicitly Off).
pub fn read_balanced(
  expansion_level: ExpansionLevel,
  is_macrodef: bool,
  require_open: bool,
) -> Result<Tokens> {
  use ExpansionLevel::*;
  if !require_open {
    decrement_align_group_count();
  }
  local_align_group_count(1000000);
  // let startloc = if lookup_verbosity() > 0 { Some(get_locator()) } else { None };
  // Do we need to expand to get the { ???
  if require_open {
    let token_opt = if expansion_level != Off {
      read_x_token(Some(false), false, None)?
    } else {
      read_token()?
    };
    let is_open = match token_opt {
      None => false,
      Some(token) => {
        token.get_catcode() == Catcode::BEGIN
          || with_meaning(
            &token,
            |m| matches!(m, Some(Stored::Token(t)) if *t == T_BEGIN!()),
          )
      },
    };
    if !is_open {
      // Push the token back so subsequent reads (especially alignment `&`,
      // newline `\\`, or `\end{tabular}`) recover gracefully when an
      // upstream macro had a required `{}` arg with no `{...}` available
      // (e.g. mn2e/multirow with a missing 3rd arg — 0903.4199 cascade).
      // Without this, the consumed `&` was lost from alignment context,
      // turning a 1-error "missing arg" into a 10001-cap `&`-cascade.
      if let Some(t) = token_opt {
        unread_one(t);
      }
      Error!("expected", "{", s!("Expected opening '{{'"));
      return Ok(Tokens!());
    }
  }
  // Pre-size the token accumulator: most balanced reads are short
  // macro arguments (~4–16 tokens). This skips the Vec's early
  // doublings that the callgrind profile attributes to
  // `raw_vec::finish_grow` (1% of total instructions in read_balanced
  // alone).
  let mut tokens: Vec<Token> = Vec::with_capacity(16);
  let mut level = 1;
  loop {
    // Resource checkpoint: this loop reads RAW (pushback.pop / mouth
    // read_token below), not via read_token/read_x_token — without its own
    // checkpoint, `\edef`-body expansion loops (`\def\x{a\x}\edef\y{\x}`)
    // bypass the token/pushback limits AND the expansion cycle guard, and
    // grind to the multi-GB process watchdog instead of a clean early Fatal.
    let guard_active = read_resource_checkpoint()?.unwrap_or(false);
    // we'll keep comments in the result
    let mut next_token = None;
    if !gullet!().pending_comments.is_empty() {
      tokens.extend(gullet_mut!().pending_comments.drain(..));
    }
    // Examine pushback first
    while let Some(pushback_token) = runtime_mut!().unwrap().pushback.pop() {
      match pushback_token.get_catcode() {
        Catcode::COMMENT => tokens.push(pushback_token),
        Catcode::MARKER => handle_marker(pushback_token),
        _ => {
          next_token = Some(pushback_token);
          break;
        },
      };
    }
    // Not in pushback, read from the current Mouth
    if next_token.is_none() {
      while let Some(token) = runtime_mut!().unwrap().mouth.read_token() {
        match token.get_catcode() {
          Catcode::COMMENT => tokens.push(token),
          Catcode::MARKER => handle_marker(token),
          _ => {
            next_token = Some(token);
            break;
          },
        };
      }
    }
    // ProgressStep() if ($$self{progress}++ % $TOKEN_PROGRESS_QUANTUM) == 0;
    if let Some(ref t) = next_token {
      // Cycle-guard checkpoint on the raw read stream (see the resource
      // checkpoint at the loop top for why read_balanced needs its own).
      cycle_guard_checkpoint(guard_active, t)?;
    }
    match next_token {
      // Mouth exhausted mid-balanced-read: mirror read_x_token / tex.web get_next
      // §362-365 — a transparent autoclose injection (\scantokens, raw_tex) is
      // part of the current logical stream, so drain it and resume the enclosing
      // mouth rather than reporting an unbalanced read. xint's
      // `\edef\X{\scantokens{...}}` opens an autoclose mouth mid-edef whose
      // matching `}` lives in the PARENT file; not crossing breaks the read at the
      // boundary and leaks `\xintexprSafeCatcodes`' `\begingroup`, corrupting
      // everything after. SURPASS-PERL: Perl readBalanced (Gullet.pm:466) `last`s
      // here and also fails this xint input.
      //
      // Gate on the mouth KIND, not just the autoclose bit: `\input` file
      // mouths are ALSO opened autoclose, and a truncated/unbalanced included
      // file must ERROR like TeX ("File ended while scanning use of …") and
      // Perl — not silently absorb the parent document into the argument
      // (PR_READINESS should-fix 9). Only string/literal injections
      // (\scantokens, RawTeX) are transparent to a balanced read.
      None => {
        let cross = {
          let gullet = gullet!();
          gullet
            .runtime
            .as_ref()
            .map(|r| r.autoclose && r.mouth.foodtype() != crate::mouth::FoodType::File)
            .unwrap_or(false)
            && !gullet.mouthstack.is_empty()
        };
        if cross {
          close_mouth(false)?;
          continue;
        }
        break;
      },
      Some(token) => match token.get_catcode() {
        Catcode::CS if token.text == pin!("\\dont_expand") => {
          if let Some(next_t) = read_token()? {
            tokens.push(next_t); // Pass on NEXT token, unchanged.
          }
        },
        Catcode::END => {
          // Perl Gullet.pm L476: track ALIGN_STATE for } inside readBalanced
          decrement_align_group_count();
          level -= 1;
          if level <= 0 {
            break;
          }
          tokens.push(token);
        },
        Catcode::BEGIN => {
          // Perl Gullet.pm L482: track ALIGN_STATE for { inside readBalanced
          increment_align_group_count();
          level += 1;
          tokens.push(token);
        },
        cc => {
          // Wow!!!!! See TeX the Program \S 309
          // Not sure if this code still applies within scan_toks???
          // SHOULD count nesting of { }!!! when SCANNED (not digested)
          if has_reading_alignment()
            && align_group_count() == 0
            && let Some((_atoken, atype, ahidden)) = is_column_end(&token)
          {
            match get_reading_alignment().unwrap().data() {
              DigestedData::Alignment(data) => {
                handle_template(data.borrow_mut(), token, atype, ahidden)?;
              },
              _ => {
                panic!("malformed alignmed was stored?");
              },
            }
            continue;
          }
          // Note: use general-purpose lookup, since we may reexamine $defn below
          if expansion_level != Off && cc.is_active_or_cs() {
            // Borrow the stored meaning via with_meaning so the Stored
            // enum is not cloned per token. We extract (a) whether a
            // meaning exists at all (for the undefined-CS diagnostic
            // below) and (b) the Rc<dyn Definition> if it's a proper
            // definition — both are cheap (bool + Rc-clone).
            let (has_meaning, defn_opt) =
              with_meaning(&token, |m| (m.is_some(), m.and_then(|s| s.to_definition())));
            if let Some(defn) = defn_opt {
              if defn.is_expandable() && (!defn.is_protected() || expansion_level == Full) {
                local_current_token(token);
                let expansion = defn.invoke(false)?;
                if expansion.is_empty() {
                  expire_current_token();
                  continue;
                }
                // If a special \the type command, push the expansion directly into the result
                // Well, almost directly: handle any MARKER tokens now, and possibly un-pack T_PARAM
                //
                // Perl `Gullet.pm:505` checks `$$defn{cs}[0]` — but in Perl the Lt-aliases
                // (e.g. `Lt('\\exp_not:n','\\unexpanded')`) share the SAME Definition, so its
                // cs field IS `\unexpanded`. In Rust the dump-writer emits `\exp_not:n` as a
                // separate Expandable with alias=`\unexpanded`; check the alias too so the
                // DEFERRED_COMMANDS gate fires for `\exp_not:n {…}` inside `\edef` bodies.
                // Without this, expl3's `\seq_gpush:Nn` (which uses `\exp_not:n` to wrap
                // `\__seq_item:n {…}`) loses its item — the item gets re-expanded into the
                // expandable-error trap, leaving the seq stack empty and triggering
                // `extra-pop-label`/`\q_no_value`-recursion cascades during `\@pushfilename`.
                let cs_matches = DEFERRED_COMMANDS.contains(&defn.get_cs().text);
                let alias_matches = defn
                  .get_alias()
                  .map(|a| DEFERRED_COMMANDS.contains(&arena::pin(a)))
                  .unwrap_or(false);
                if expansion_level != Full && (cs_matches || alias_matches) {
                  for t in expansion.unlist() {
                    match t.get_catcode() {
                      Catcode::MARKER => handle_marker(t),
                      Catcode::PARAM if is_macrodef => {
                        // "unpack" to cover the packParameters at end!
                        tokens.push(t);
                        tokens.push(t);
                      },
                      _ => tokens.push(t),
                    }
                  }
                } else {
                  // otherwise, prepend to pushback to be expanded further.
                  unread(expansion);
                }
                expire_current_token();
                continue;
              }
            } else if cc == Catcode::CS && !has_meaning {
              // cs SHOULD have defn by now; report early!
              generate_error_stub(&token)?;
            }
          }
          // Return the token — EXCEPT a `\special_relax` (noexpand'd) family token
          // collected into an expanded token list reverts to its plain shadowed
          // identity: TeX's no_expand_flag is transient (tex.web §1149-1153), so
          // `\edef`/`\xdef` store the PLAIN token, not a relax marker (etex ground
          // truth: `\def\s{\noexpand\s}\edef\r{\romannumeral0\s}` → `\meaning\r` =
          // "macro:->\s", xint's f-stop idiom). Otherwise the family token persists
          // into the `\edef` body and a later number scan hits "Missing number".
          // Gated on CS/active so the hot per-token push pays only a catcode check.
          if cc.is_active_or_cs() {
            tokens.push(token.noexpand_shadowed().unwrap_or(token));
          } else {
            tokens.push(token);
          }
        },
      },
    }
  }
  if level > 0 {
    // Reached for a genuinely unbalanced read: a balancing end in a LITERAL
    // (string-injection) mouth IS recognized via the autoclose crossing above;
    // a FILE boundary deliberately is not (TeX/Perl parity — "file ended
    // while scanning"), so a truncated \input lands here with the loud Error.
    // TODO: add the startloc details
    // my $loc_message = $startloc ? ("Started at " . ToString($startloc)) : ("Ended at " .
    // ToString($self->getLocator));
    Error!(
      "expected",
      "}",
      "Gullet->readBalanced ran out of input in an unbalanced state"
    );
  }
  expire_align_group_count();
  if tokens.is_empty() {
    Ok(Tokens!())
  } else {
    Ok(if is_macrodef {
      Tokens::new(tokens).pack_parameters()?
    } else {
      Tokens::new(tokens)
    })
  }
}

/// Match the input against a set of keywords; Similar to readMatch, but the keywords are strings,
/// and Case and catcodes are ignored; additionally, leading spaces are skipped.
/// AND, macros are expanded.
///
/// Perf: zero-allocation char-wise comparison against each keyword.
/// The previous version allocated two Strings per char-match (via `to_uppercase()`
/// and `char::to_string()`), which was expensive in hot parameter parsing loops.
pub fn read_keyword(keywords: &[&str]) -> Result<Option<String>> {
  skip_spaces()?;
  for keyword in keywords.iter() {
    // Pre-size to the keyword length — `matched` holds one token per
    // matched char, and we unread them on no-match. Keyword-match
    // runs on every parameter/keyword read; small win per call.
    let mut matched = Vec::with_capacity(keyword.len());
    let mut ok = true;
    for expected in keyword.chars() {
      let Some(tok) = read_x_token(Some(false), false, None)? else {
        ok = false;
        break;
      };
      // Compare char-by-char against the token's text, case-insensitively.
      let eq = tok.with_str(|s| {
        let mut it = s.chars();
        match it.next() {
          Some(c) if it.next().is_none() => {
            // single-char token: case-insensitive compare
            c.to_uppercase().eq(expected.to_uppercase())
          },
          _ => false,
        }
      });
      matched.push(tok);
      if !eq {
        ok = false;
        break;
      }
    }
    if ok {
      return Ok(Some(keyword.to_string()));
    } else {
      unread(matched.into());
    }
  }
  Ok(None)
}

/// Return a (balanced) sequence tokens until a match against one of the Tokens in @delims.
///
/// Note that Braces on input hides the contents from matching,
/// so this assumes there wont be braces in $delim!
/// But, see readUntilBrace for that case.
pub fn read_until(delim: &Tokens) -> Result<Tokens> {
  // Pre-size like `read_balanced`: the accumulator is grown one token at a time
  // in the loops below, so an unsized `Vec::new()` pays the 0→1→2→4→8 doubling
  // reallocations on every call. 16 covers the common short delimited read in a
  // single allocation (a top `grow_one` site in the allocation profile).
  let mut tokens: Vec<Token> = Vec::with_capacity(16);
  let mut nbraces = 0;
  let want = delim.unlist_ref();
  let ntomatch = want.len();
  let mut has_matched;

  if ntomatch == 1 {
    let want = &want[0];
    loop {
      let token = match read_token()? {
        Some(t) => t,
        None => {
          // Ran out!
          unread(Tokens::new(tokens));
          return Ok(Tokens!()); // Not more correct, but maybe less confusing?
        },
      };
      // Perl: check direct match OR \special_relax smuggling (Gullet.pm line 662)
      if token == *want || special_relax_matches(&token, want) {
        break;
      }
      match token.get_catcode() {
        Catcode::MARKER => {
          // would have been handled by readToken, but we're bypassing
          handle_marker(token);
        },
        Catcode::BEGIN => {
          // And if it's a BEGIN, copy till balanced END
          nbraces += 1;
          tokens.push(token);
          let balanced_arg = read_balanced(ExpansionLevel::Off, false, false)?;
          if !balanced_arg.is_empty() {
            tokens.extend(balanced_arg.unlist());
          }
          tokens.push(T_END!());
        },
        _ => {
          tokens.push(token);
        },
      }
    }
  } else {
    let mut ring = VecDeque::new();
    loop {
      // prefill the required number of tokens
      while ring.len() < ntomatch {
        let token = match read_token()? {
          Some(t) => t,
          None => {
            // Ran out!
            unread(Tokens::new(tokens));
            return Ok(Tokens!()); // Not more correct, but maybe less confusing?
          },
        };
        // Perl: $$token[1] == CC_BEGIN — direct catcode check
        if token.get_catcode() == Catcode::BEGIN {
          // read balanced, and refill ring.
          nbraces += 1;
          for r_token in ring {
            tokens.push(r_token);
          }
          tokens.push(token);
          let balanced_arg = read_balanced(ExpansionLevel::Off, false, false)?;
          if !balanced_arg.is_empty() {
            tokens.append(&mut balanced_arg.unlist());
          }
          tokens.push(T_END!()); // Copy directly to result
          ring = VecDeque::new(); // and retry
        } else {
          ring.push_back(token);
        }
      }
      has_matched = &ring == want; // Test match
      if has_matched {
        break;
      } // Matched all!
      if let Some(ring_token) = ring.pop_front() {
        tokens.push(ring_token);
      }
    }
  }
  // Notice that IFF the arg looks like {balanced}, the outer braces are stripped
  // so that delimited arguments behave more similarly to simple, undelimited arguments.
  // Perl: ($nbraces == 1) && ($tokens[0][1] == CC_BEGIN) && ($tokens[-1][1] == CC_END)
  if nbraces == 1
    && tokens.first().unwrap().get_catcode() == Catcode::BEGIN
    && tokens.last().unwrap().get_catcode() == Catcode::END
  {
    tokens.remove(0);
    tokens.pop();
  }
  Ok(Tokens::new(tokens))
}

/// Convenience method wrapping around `read_until`
/// TODO: This seems to be the wrong Rust type interface, we need to rework...
pub fn read_until_token(t: Token) -> Result<Tokens> { read_until(&Tokens!(t)) }
/// reads until it encounters a Catcode::BEGIN token
/// Note: Perl uses $$token[1] == CC_BEGIN (catcode check, not defined_as)
pub fn read_until_brace() -> Result<Option<Tokens>> {
  let mut tokens = Vec::new();
  while let Some(token) = read_token()? {
    if token.get_catcode() == Catcode::BEGIN {
      unread_one(token); // Unread with proper agc adjustment
      break;
    } else {
      tokens.push(token);
    }
  }
  if tokens.is_empty() {
    Ok(None)
  } else {
    let tks = Tokens::new(tokens);
    Ok(Some(tks))
  }
}

pub fn read_cs_name() -> Result<Token> { read_cs_name_inner(false) }

/// Quiet version of read_cs_name — used by \ifcsname.
/// In TeX, \ifcsname silently skips non-expandable CS tokens and returns the constructed name
/// without emitting errors (unlike \csname which DOES emit errors).
pub fn read_cs_name_quiet() -> Result<Token> { read_cs_name_inner(true) }

fn read_cs_name_inner(quiet: bool) -> Result<Token> {
  // TeX does NOT store the csname with the leading `\`, BUT stores active chars with a flag
  // However, so long as the Mouth's CS and \string properly respect \escapechar, all's well!

  // Safety bound: a real CS name fits in well under 256 chars. We've seen
  // pathological cases (lipsum.sty with malformed \cs_set_nopar:Npe expansion,
  // or expl3 raw-load before \endcsname is bound) where `\csname` reads
  // thousands of tokens accumulating into `cs`, eventually OOMing the
  // `read_x_token` pushback Vec. Cap at 4096 bytes — beyond that there's no
  // legitimate CS name, just a runaway. Emit one clear error and break.
  const MAX_CS_NAME_BYTES: usize = 4096;
  let mut cs = String::from("\\");
  // keep newlines from having \n inside!
  while let Some(token) = read_x_token(Some(true), false, None)? {
    if token.defined_as(&TOKEN_ENDCSNAME) {
      break;
    }
    if cs.len() > MAX_CS_NAME_BYTES {
      Error!(
        "runaway",
        "csname",
        format!(
          "CS-name read exceeded {MAX_CS_NAME_BYTES} bytes; aborting at partial cs: {:?}",
          // Truncate by CHARS, not bytes — a byte slice at 200 can split a
          // multi-byte UTF-8 char (e.g. 'ä') and panic. Witness 2601.03403.
          cs.chars().take(200).collect::<String>()
        )
      );
      break;
    }
    match token.get_catcode() {
      Catcode::CS => {
        // Soft-substitute the underlying char for a character-equivalent CS
        // token in the \csname stream — a documented divergence from Knuth TeX
        // (tex.web L7745-7758 hard-errors "Missing \endcsname inserted" for any
        // CS that isn't \endcsname). Our expansion pipeline surfaces PA-aliased
        // `Stored::Token` CSes (expl3 `\exp_stop_f:` = frozen space, `\lx@NBSP`
        // from CLUSTER-NBSP) into the csname stream where real TeX wouldn't reach
        // this state; erroring like Knuth would break real expl3/mhchem/glossaries
        // loads, so we substitute the char the author meant. Witnesses: `\lx@NBSP`
        // (CLUSTER-NBSP, 18 papers; `~`→U+00A0 in `\csname r@LABEL\endcsname`),
        // `\exp_stop_f:` (mhchem raw-load). The `\lx@NBSP` carve-out below stays
        // for clarity; the general `Stored::Token` case handles the rest uniformly.
        let cs_str = token.with_str(|s| s.to_string());
        // Well-known `\text…` primitives that map to a single char in real
        // pdflatex's csname-stream interpretation. The `DefPrimitive!(name,
        // "char")` body is a closure that wraps a Tbox — not statically
        // inspectable from here — so we maintain an explicit table for the
        // canonical set. Witnesses (stage-1..3 of 100k warning corpus):
        //   \\textquoteright surfacing in `\twemoji flag: Côte d` cluster
        //   (≥4 papers across 2603.08303, 2604.13899, 2604.17338, 2604.20621).
        let soft_char: Option<char> = match cs_str.as_str() {
          "\\lx@NBSP" | "\\lx@nobreakspace" | "\\nobreakspace" => Some('\u{00A0}'),
          "\\textquoteright" => Some('\u{2019}'),
          "\\textquoteleft" => Some('\u{2018}'),
          "\\textquotedblright" => Some('\u{201D}'),
          "\\textquotedblleft" => Some('\u{201C}'),
          "\\textquotedbl" => Some('"'),
          "\\textemdash" => Some('\u{2014}'),
          "\\textendash" => Some('\u{2013}'),
          "\\textbackslash" => Some('\u{005C}'),
          "\\textbar" => Some('|'),
          "\\textbraceleft" => Some('{'),
          "\\textbraceright" => Some('}'),
          "\\textless" => Some('<'),
          "\\textgreater" => Some('>'),
          "\\textdollar" => Some('$'),
          "\\textasciigrave" => Some('`'),
          "\\textasciicircum" => Some('^'),
          "\\textasciitilde" => Some('~'),
          "\\textunderscore" => Some('_'),
          "\\textasteriskcentered" => Some('*'),
          // NFSS encoding-specific glyph CS names (`\<encoding>\<glyph>`)
          // built by \DeclareTextSymbol for the i/j dotless letters. These
          // surface when a paper composes `\'\i` style accented chars
          // that travel through `\lx@applyaccent` and a downstream
          // encoding-specific dispatcher. Substitute the dotless glyph
          // (U+0131 / U+0237) so the constructed csname carries the
          // character the author meant.
          // Witnesses: arXiv:2603.22193, 2603.23433, 2604.20621 (twemoji
          // São Tomé & Príncipe / St. Barthélemy / Côte d'Ivoire cluster).
          "\\T1\\i" | "\\OT1\\i" | "\\LY1\\i" => Some('\u{0131}'),
          "\\T1\\j" | "\\OT1\\j" | "\\LY1\\j" => Some('\u{0237}'),
          _ => None,
        };
        if let Some(c) = soft_char {
          cs.push(c);
        } else if cs_str == "\\lx@applyaccent" {
          // Accent macros (`\'`, `\"`, `\^`, …) expand to
          // `\lx@applyaccent <accent> <combining> <standalone> {<letter>}`
          // (tex_character.rs::accent_def). pdflatex's `\csname` skips the
          // accented char (the accent runs in the gullet); ours is a stomach
          // `DefPrimitive`, so a literal `\lx@applyaccent` surfaces in the csname
          // stream and aborts the read (witnesses: twemoji.sty, arXiv:2603.22193 /
          // 2603.23433). Faithful fix: peek the 4 args, append the standalone char
          // (arg 3, T_OTHER!) to the name, discard the rest — mirrors the
          // implicit-character substitution above.
          let _accent = read_x_token(Some(true), false, None)?;
          let _combiner = read_x_token(Some(true), false, None)?;
          let standalone = read_x_token(Some(true), false, None)?;
          // The 4th arg is a brace group `{<letter>}` — consume the
          // T_BEGIN, then read tokens until matching T_END.
          if let Some(t) = read_x_token(Some(true), false, None)?
            && t.get_catcode() == Catcode::BEGIN
          {
            let mut depth: i32 = 1;
            while depth > 0 {
              match read_x_token(Some(true), false, None)? {
                Some(t2) => match t2.get_catcode() {
                  Catcode::BEGIN => depth += 1,
                  Catcode::END => depth -= 1,
                  _ => {},
                },
                None => break,
              }
            }
          }
          if let Some(c) = standalone {
            c.with_str(|s| cs.push_str(s));
          }
        } else {
          match lookup_meaning(&token) {
            Some(Stored::Token(letted)) => {
              // CS is \let-equivalent to a single token. If that token is
              // a character (LETTER/OTHER/SPACE), append its string repr
              // to the constructed csname — mirrors real TeX's behaviour
              // of substituting the let-target into the csname stream.
              // Non-character lets (Catcode::CS, MATH, etc.) fall through
              // to the error branches below.
              let target_cc = letted.get_catcode();
              if matches!(target_cc, Catcode::LETTER | Catcode::OTHER | Catcode::SPACE) {
                if target_cc == Catcode::SPACE {
                  cs.push(' ');
                } else {
                  letted.with_str(|s| cs.push_str(s));
                }
              } else if !quiet {
                let message = s!(
                  "The control sequence {:?} should not appear between \\csname and \\endcsname (partial cs so far: {:?})",
                  token,
                  cs
                );
                Error!("unexpected", token, message);
              }
            },
            _ => {
              if !quiet {
                if lookup_definition(&token)?.is_some() {
                  let message = s!(
                    "The control sequence {:?} should not appear between \\csname and \\endcsname (partial cs so far: {:?})",
                    token,
                    cs
                  );
                  Error!("unexpected", token, message);
                } else {
                  let message = s!("The token {:?} is not defined", token);
                  Error!("undefined", token, message);
                }
              }
            },
          }
        }
        // In quiet mode (ifcsname), just skip the CS token
      },
      Catcode::SPACE => cs.push(' '), // Keep newlines from having \n!
      _ => {
        token.with_str(|s| cs.push_str(s));
      },
    };
  }
  Ok(T_CS!(cs))
}

/// reads and discards tokens, until it encounters a conditional, if any.
/// Perl: skipConditionalBody inner loop (Conditional.pm L127-133) reads tokens directly
/// from pushback/mouth (NOT through readToken) and manually tracks
/// `$LaTeXML::ALIGN_STATE` for `{` and `}`. Critically, this bypasses the
/// "alignment-template trigger" check that fires `handleTemplate` on `&`/`\cr`
/// when align_group_count==0 — that check belongs to digestion, not to
/// `\else`-skip. Rust's `read_token` includes the trigger; calling it from
/// here would let pmatrix's `&` get treated as the OUTER alignment's
/// column-end during `\ifx.#1.\else…\fi` skip when `#1` contains
/// `\begin{pmatrix}…&…\end{pmatrix}`. (REG-2 / math-ph0501074: the 9-line
/// `\nonumber+\lefteqn+\pmatrix` repro.) Use `read_internal_token` instead
/// and track BEGIN/END manually, matching Perl byte-for-byte.
pub fn read_next_conditional() -> Result<Option<(Token, ConditionalType)>> {
  loop {
    // The FOURTH reader loop over `read_internal_token` (the `\else`/`\fi`
    // skipper — it must not delegate to read_token, whose alignment trigger
    // misfires during conditional-skip; see the fn doc). It still needs the
    // resource/cycle checkpoints: a skip over a pathologically large stream
    // must count progress and remain loop-detectable like every other read
    // path (PR #249 review P2-9).
    let guard_active = match read_resource_checkpoint()? {
      None => return Ok(None),
      Some(active) => active,
    };
    let next_low = read_internal_token();
    match next_low {
      Some(token) => {
        cycle_guard_checkpoint(guard_active, &token)?;
        let cc = token.get_catcode();
        // Perl L128-130: manual ALIGN_STATE tracking for `{` / `}` (avoids
        // the trigger check that read_token applies).
        match cc {
          Catcode::BEGIN => increment_align_group_count(),
          Catcode::END => decrement_align_group_count(),
          _ => {},
        }
        if cc.is_active_or_cs()
          && let Some(cond_type) = lookup_conditional(&token)
        {
          return Ok(Some((token, cond_type)));
        }
      },
      None => {
        // Current mouth exhausted. Try closing if autoclosable and there are
        // more mouths on the stack (TeX continues reading across input boundaries).
        let (autoclose, stack_len) = {
          let gullet = gullet!();
          let ac = gullet
            .runtime
            .as_ref()
            .map(|r| r.autoclose)
            .unwrap_or(false);
          let sl = gullet.mouthstack.len();
          (ac, sl)
        };
        if autoclose && stack_len > 0 {
          close_mouth(false)?;
          continue;
        }
        return Ok(None);
      },
    }
  }
}

//**********************************************************************
// Higher-level readers: Read various types of things from the input:
//  tokens, non-expandable tokens, args, Numbers, ...
//**********************************************************************

/// Read and return a "normal" TeX argument
///
/// The next Token or Tokens (if surrounded by braces).
/// `expansion_level` controls expansion as if the argument were read
///  and then expanded in isolation:
///
/// In the case of a single unbraced expandable token,
///  it will **not** read any macro arguments from the following input!
pub fn read_arg(expansion_level: ExpansionLevel) -> Result<Tokens> {
  match read_non_space()? {
    None => Ok(Tokens!()),
    Some(token) => {
      // Perl: $$token[1] == CC_BEGIN — checks actual catcode, NOT defined_as.
      // \bgroup (catcode CS) does NOT match here; only literal { does.
      if token.get_catcode() == Catcode::BEGIN {
        read_balanced(expansion_level, false, false)
      } else if matches!(expansion_level, ExpansionLevel::Off) {
        // A `\noexpand`'d token captured as an (undelimited) macro argument
        // reverts to its plain shadowed identity — faithful to TeX, where the
        // `no_expand_flag` is transient and never stored in the captured arg
        // (`#1` is `cur_cs`, the plain token). This lets `\ifx\X#1` match (xint's
        // `\def\XINTfstop{\noexpand\XINTfstop}` f-stop, witness 1804.01117).
        // `\let`/`\ifx` of the LIVE token read via `read_token` (the `Token`
        // param), NOT here, so their relax-meaning capture is unaffected.
        Ok(Tokens!(token.noexpand_shadowed().unwrap_or(token)))
      } else {
        // Perl Gullet.pm `readArg`:
        //   return $self->readingFromMouth(Tokens(T_BEGIN, $token, T_END), sub {
        //       readBalanced($self, $expanded, 0, 1); });
        // Use an isolated mouth so leftover tokens (e.g. an extra `}` when
        // `$token` itself happens to be T_END) cannot leak back into the
        // caller's stream. `unread_vec` here would pollute the parent mouth.
        let synth = Tokens::new(vec![T_BEGIN!(), token, T_END!()]);
        reading_from_mouth(Mouth::default(), move || -> Result<Tokens> {
          unread(synth);
          read_balanced(expansion_level, false, true)
        })
      }
    },
  }
}
/// Read and return a LaTeX optional argument
///
/// returns `default` if there is no '[', otherwise the contents of the array.
/// Note that this returns an empty array if `[]` is present,
/// i.e. `[contents]` in TeX will lead to `Tokens(contents)`, otherwise returns `None`
pub fn read_optional(default: Option<Tokens>) -> Result<Option<Tokens>> {
  match read_non_space()? {
    None => Ok(None),
    Some(t) => {
      if t.get_catcode() == Catcode::OTHER && t.get_sym() == pin!("[") {
        Ok(Some(read_until(&Tokens!(T_OTHER!("]")))?))
      } else {
        unread_one(t);
        Ok(default)
      }
    },
  }
}

/// <filler> = <optional spaces> | <filler>\relax<optional spaces>
/// TeX Book p.276 "<left brace> can be implicit", and experimentation, indicate Expansion!!!
pub fn skip_filler() -> Result<()> {
  while let Some(tok) = read_x_non_space()? {
    if !tok.defined_as(&TOKEN_RELAX) {
      unread_one(tok);
      break;
    }
  }
  Ok(())
}

pub fn if_next(token: Token) -> Result<bool> {
  let mut is_next = false;
  if let Some(tok) = read_token()? {
    is_next = tok == token;
    unread_one(tok);
  }
  Ok(is_next)
}

/// Perl: peekToken — peek at the next token without triggering alignment
/// Sets ALIGN_STATE to 1000000 to suppress alignment template handling (Perl line 331-337)
pub fn peek_token() -> Result<Option<Token>> {
  local_align_group_count(1000000);
  let result = read_token()?;
  if let Some(ref tok) = result {
    unread_one(*tok);
  }
  expire_align_group_count();
  Ok(result)
}

/// Perl: showUnexpected — returns a debug message about the next available token
pub fn show_unexpected() -> String {
  match peek_token() {
    Ok(Some(token)) => {
      let meaning = lookup_meaning(&token)
        .map(|m| format!("{:?}", m))
        .unwrap_or_else(|| "undef".to_string());
      s!("Next token is {} ( == {})", token.stringify(), meaning)
    },
    _ => "Input is empty".to_string(),
  }
}

//**********************************************************************
//  Numbers, Dimensions, Glue
// See TeXBook, Ch.24, pp.269-271.
//**********************************************************************

pub fn read_value(value_type: RegisterType) -> Result<RegisterValue> {
  match value_type {
    RegisterType::Number => Ok(read_number()?.into()),
    RegisterType::Dimension => Ok(read_dimension()?.into()),
    RegisterType::MuDimension => Ok(read_mu_dimension()?.into()),
    RegisterType::Glue => Ok(read_glue()?.into()),
    RegisterType::MuGlue => Ok(read_mu_glue()?.into()),
    RegisterType::Tokens => Ok(read_tokens_value()?.into()),
    RegisterType::Token => {
      // Perl: readValue('Token') checks for \csname (Gullet.pm line 770-775)
      #[thread_local]
      static TOKEN_CSNAME: Lazy<Token> = Lazy::new(|| T_CS!("\\csname"));
      let token = read_non_space()?.unwrap_or(*TOKEN_RELAX);
      if token.defined_as(&TOKEN_CSNAME) {
        Ok(read_cs_name()?.into())
      } else {
        Ok(token.into())
      }
    },
    RegisterType::CharDef => Ok(read_number()?.into()),
    RegisterType::Any => Ok(read_arg(ExpansionLevel::Off)?.into()),
  }
}

pub fn read_register_value(value_type: RegisterType) -> Result<Option<RegisterValue>> {
  read_register_value_coerce(value_type, false)
}

/// Read a register value, optionally coercing from a compatible larger type.
/// Perl: readRegisterValue($self, $type, $sign, $coerce)
/// Coercion rules (from Perl %RegisterCoercionTypes):
///   Number    <- Dimension, Glue     (extract raw i64)
///   Dimension <- Glue                (extract skip as Dimension)
///   MuDimension <- MuGlue            (extract skip as MuDimension)
pub fn read_register_value_coerce(
  value_type: RegisterType,
  coerce: bool,
) -> Result<Option<RegisterValue>> {
  match read_x_token(None, false, None)? {
    None => Ok(None),
    Some(token) => {
      let _is_fontdimen = token.with_str(|s| s == "\\fontdimen");
      match lookup_register_definition(&token) {
        Some(defn) => {
          if let Some(mut register_type) = defn.register_type() {
            if register_type == RegisterType::CharDef {
              // CharDefs treated as numbers here
              register_type = RegisterType::Number;
            }
            if register_type == value_type {
              let args = defn.read_arguments()?;
              Ok(defn.value_of(args))
            } else if coerce {
              // Try type coercion per Perl's %RegisterCoercionTypes
              if let Some(coerced) = coerce_register(value_type, register_type, &defn)? {
                Ok(Some(coerced))
              } else {
                unread_one(token);
                Ok(None)
              }
            } else {
              unread_one(token); // Unread
              Ok(None)
            }
          } else {
            unread_one(token); // Unread
            Ok(None)
          }
        },
        _ => {
          unread_one(token); // Unread
          Ok(None)
        },
      }
    },
  }
}

/// Attempt to coerce a register value from `source_type` to `target_type`.
fn coerce_register(
  target_type: RegisterType,
  source_type: RegisterType,
  defn: &Register,
) -> Result<Option<RegisterValue>> {
  use crate::common::numeric_ops::NumericOps;
  // Perl fix 50f0061d: include self-coercions (Number→Number, etc.)
  // so \number \fam works when \fam is already a Number register
  let can_coerce = matches!(
    (target_type, source_type),
    (RegisterType::Number, RegisterType::Number)
      | (RegisterType::Number, RegisterType::Dimension)
      | (RegisterType::Number, RegisterType::Glue)
      | (RegisterType::Dimension, RegisterType::Dimension)
      | (RegisterType::Dimension, RegisterType::Glue)
      | (RegisterType::MuDimension, RegisterType::MuDimension)
      | (RegisterType::MuDimension, RegisterType::MuGlue)
      | (RegisterType::Glue, RegisterType::Glue)
      | (RegisterType::MuGlue, RegisterType::MuGlue)
  );
  if !can_coerce {
    return Ok(None);
  }
  let args = defn.read_arguments()?;
  if let Some(val) = defn.value_of(args) {
    let raw = match val {
      RegisterValue::Dimension(d) => d.value_of(),
      RegisterValue::Glue(g) => g.value_of(),
      RegisterValue::MuGlue(mg) => mg.value_of(),
      RegisterValue::Number(n) => n.value_of(),
      RegisterValue::MuDimension(md) => md.value_of(),
      _ => return Ok(None),
    };
    let coerced = match target_type {
      RegisterType::Number => RegisterValue::Number(Number::new(raw)),
      RegisterType::Dimension => RegisterValue::Dimension(Dimension::new(raw)),
      RegisterType::MuDimension => RegisterValue::MuDimension(MuDimension::new(raw)),
      _ => return Ok(None),
    };
    Ok(Some(coerced))
  } else {
    Ok(None)
  }
}

/// Match the input against one of the Token or Tokens in @choices; return the matching one or
/// undef.
pub fn read_match(choices: &[&Tokens]) -> Result<Option<Tokens>> {
  for choice in choices {
    let mut to_match: Vec<&Token> = choice.unlist_ref().iter().rev().collect();
    // `matched` accumulates tokens read so far, bounded by `choice.len()`.
    // Pre-size to avoid reallocations on multi-token match attempts.
    let mut matched = Vec::with_capacity(choice.unlist_ref().len());
    while !to_match.is_empty() {
      match read_token()? {
        None => break,
        Some(token) => {
          let cc = token.get_catcode();
          // Perl: also check smuggled \special_relax token (Gullet.pm line 612)
          let was_last_match = if let Some(&&want) = to_match.last() {
            token == want || special_relax_matches(&token, &want)
          } else {
            false
          };
          matched.push(token);
          if was_last_match {
            to_match.pop();
          } else {
            break;
          }

          if cc == Catcode::SPACE {
            // If this was space, SKIP any following!!!
            while let Some(space_token) = read_token()? {
              if space_token.get_catcode() != Catcode::SPACE {
                // Unread non-space and end — use unread_one for proper agc adjustment
                unread_one(space_token);
                break;
              } else {
                matched.push(space_token);
              }
            }
          }
        },
      }
    }
    if to_match.is_empty() {
      return Ok(Some((*choice).clone())); // All matched!!!
    } else {
      // Put 'em back and try next — use unread_vec for proper agc adjustment
      unread_vec(matched);
    }
  }
  Ok(None)
}

//======================================================================
// Integer, Number
//======================================================================
// ```
// <number> = <optional signs><unsigned number>
// <unsigned number> = <normal integer> | <coerced integer>
// <coerced integer> = <internal dimen> | <internal glue>
// ```
pub fn read_number() -> Result<Number> {
  let is_negative = read_optional_signs()?;
  let s = if is_negative { -1 } else { 1 };
  if let Some(n) = read_normal_integer()? {
    if is_negative { Ok(n.negate()) } else { Ok(n) }
  } else if let Some(n) = read_internal_dimension()? {
    Ok(Number::new(s * n.value_of()))
  } else if let Some(n) = read_internal_glue()? {
    Ok(Number::new(s * n.value_of()))
  } else {
    let next = read_token()?;
    // Perl Gullet.pm:904-905: the primary message is just "Missing number,
    // treated as zero"; the processing context and the unexpected-token
    // (showUnexpected) are SEPARATE Error details rendered on their own
    // lines. Render the tokens with ToString/Stringify, not Rust-Debug
    // (which leaks `Some("\\relax")` into user-facing diagnostics).
    let current = get_current_token()
      .map(|t| t.to_string())
      .unwrap_or_default();
    let unexpected = match next {
      Some(t) => s!("Next token is {}", t.stringify()),
      None => s!("Input is empty"),
    };
    Warn!(
      "expected",
      "<number>",
      "Missing number, treated as zero",
      s!("while processing {current}"),
      unexpected
    );
    if let Some(next) = next {
      unread_one(next);
    }
    Ok(Number::new(0))
  }
}

/// ```bnf
/// <normal integer> = <internal integer> | <integer constant>
///   | '<octal constant><one optional space> | "<hexadecimal constant><one optional space>
///   | `<character token><one optional space>
/// ```
pub fn read_normal_integer() -> Result<Option<Number>> {
  match read_x_token(None, false, None)? {
    None => Ok(None),
    Some(token) => {
      let cc = token.get_catcode();
      let mut text = token.to_string();
      if cc == Catcode::OTHER && text.chars().all(|c| c.is_ascii_digit()) {
        // Read decimal literal. Overflow is rare but possible on weird
        // input (digit runs wider than i64::MAX); Perl's TeX silently
        // truncates such values, so we fall back to i64::MAX / MIN on
        // parse failure rather than panicking with .expect().
        text.push_str(&read_digits(&DIGIT_RE, true)?);
        let n = text.parse::<i64>().unwrap_or_else(|_| {
          if text.starts_with('-') {
            i64::MIN
          } else {
            i64::MAX
          }
        });
        Ok(Some(Number::new(n)))
      } else if token == T_OTHER!("'") {
        // Read Octal literal. Perl: `Number(oct(readDigits(...)))`, and
        // Perl's `oct("")` is 0 — so a `'` with no octal digit following
        // yields 0 (TeX's "Missing number, treated as zero"), NOT a fatal
        // error. Mirror that, and clamp overflow to i64::MAX like the
        // decimal arm rather than propagating a ParseIntError.
        let digits = read_digits(&OCT_RE, true)?;
        let decimal = if digits.is_empty() {
          0
        } else {
          i64::from_str_radix(&digits, 8).unwrap_or(i64::MAX)
        };
        Ok(Some(Number::new(decimal)))
      } else if token == T_OTHER!("\"") {
        //  Read Hex literal. Perl: `Number(hex(readDigits(...)))`, and
        // Perl's `hex("")` is 0 — so a `"` with no hex digit following
        // yields 0, NOT a fatal error. (Witness 2008.10843: mdwmath.sty
        // raw-load reads a bare `"` with no hex digit → previously a
        // `Fatal:Document:Generic(ParseIntError)` aborting the run.)
        let digits = read_digits(&HEX_RE, true)?;
        let decimal = if digits.is_empty() {
          0
        } else {
          i64::from_str_radix(&digits, 16).unwrap_or(i64::MAX)
        };
        Ok(Some(Number::new(decimal)))
      } else if token == T_OTHER!("`") {
        //  Read Charcode: `<character token><one optional space>
        let mut s = match read_token()? {
          None => String::new(),
          Some(next) => next.to_string(),
        };
        if s.starts_with('\\') {
          s.remove(0);
        }
        let s_char = s.chars().next().unwrap_or('\0');
        // Perl: skip1Space($self, 1); — expanded space-skip after charcode
        skip_one_space(true)?;
        Ok(Some(Number::new(s_char as i64))) //  Only a character token!!! NOT expanded!!!!
      } else {
        unread_one(token); // Unread
        read_internal_integer()
      }
    },
  }
}

///======================================================================
/// Float, a floating point number.
/// Similar to factor, but does NOT accept comma!
/// This is NOT part of TeX, but is convenient.
pub fn read_float() -> Result<Float> {
  let is_negative = read_optional_signs()?;
  let s = if is_negative { -1.0 } else { 1.0 };
  let mut string = read_digits(&DIGIT_RE, true)?;
  let mut token = read_x_token(None, false, None)?;
  if token.is_some() && token.as_ref().unwrap().get_sym() == pin!(".") {
    string = s!("{string}.{}", read_digits(&DIGIT_RE, true)?);
    token = read_x_token(None, false, None)?;
  }
  let n_opt: Option<f64> = if !string.is_empty() {
    if let Some(t) = token
      && t.get_catcode() != Catcode::SPACE
    {
      unread_one(t);
    }
    // Same rationale as read_normal_integer above: malformed float
    // literals (e.g. very long digit runs, "1e" without exponent)
    // should degrade to 0.0 rather than panic.
    Some(string.parse::<f64>().unwrap_or(0.0))
  } else {
    if let Some(t) = token {
      unread_one(t); // Unread
    }
    read_normal_integer()?.map(|v| v.value_of() as f64)
  };

  if let Some(n) = n_opt {
    Ok(Float::new_f64(s * n))
  } else {
    Ok(Float::new_f64(0.0))
  }
}

fn read_internal_integer() -> Result<Option<Number>> {
  match read_register_value(RegisterType::Number)? {
    None => Ok(None),
    Some(val) => Ok(Some(val.into())),
  }
}
fn read_internal_dimension() -> Result<Option<Dimension>> {
  match read_register_value(RegisterType::Dimension)? {
    None => Ok(None),
    Some(val) => Ok(Some(val.into())),
  }
}
fn read_internal_glue() -> Result<Option<Glue>> {
  match read_register_value(RegisterType::Glue)? {
    None => Ok(None),
    Some(val) => Ok(Some(val.into())),
  }
}

//======================================================================
// Dimensions
//======================================================================
// ```
// <dimen> = <optional signs><unsigned dimen>
// <unsigned dimen> = <normal dimen> | <coerced dimen>
// <coerced dimen> = <internal glue>
// ```
pub fn read_dimension() -> Result<Dimension> {
  let is_negative = read_optional_signs()?;
  if let Some(d) = read_internal_dimension()? {
    Ok(if is_negative { d.negate() } else { d })
  } else if let Some(d) = read_internal_glue()? {
    Ok(Dimension::new(if is_negative {
      d.negate().value_of()
    } else {
      d.value_of()
    }))
  } else if let Some(d) = read_factor()? {
    let (num, den) = match read_unit()? {
      Some(ratio) => ratio,
      None => {
        Warn!(
          "expected",
          "<unit>",
          "Illegal unit of measure (pt inserted)."
        );
        (1, 1)
      },
    };
    let d_signed = if is_negative { -d } else { d };
    Ok(Dimension::new(fixpoint_unit(d_signed, num, den)))
  } else {
    // Perl Gullet.pm:972: the type is named in the primary message
    // ("(Dimension)") and "while processing X" is a separate detail
    // (ToString of the current token, not Rust-Debug).
    let cur = get_current_token()
      .map(|t| t.to_string())
      .unwrap_or_default();
    Warn!(
      "expected",
      "<number>",
      "Missing number (Dimension), treated as zero.",
      s!("while processing {cur}")
    );
    Ok(Dimension::new(0))
  }
}

// ```
// <unit of measure> = <optional spaces><internal unit>
//     | <optional true><physical unit><one optional space>
// <internal unit> = em <one optional space> | ex <one optional space>
//     | <internal integer> | <internal dimen> | <internal glue>
// <physical unit> = pt | pc | in | bp | cm | mm | dd | cc | sp
// ```

/// Read a unit, returning the exact TeX `(num, den)` conversion fraction (see
/// [`convert_unit_ratio`] / `numeric_ops::fixpoint_unit`). Internal/coerced units
/// (`\wd0`, `\dimen`, glue) yield `(value_sp, 65536)` — the `floor(fix·v/65536)`
/// path of tex.web §8983, exact in integer arithmetic.
pub fn read_unit() -> Result<Option<(i64, i64)>> {
  let unit_opt = if let Some(u) = read_keyword(&["ex", "em"])? {
    skip_one_space(true)?;
    Some(convert_unit_ratio(&u))
  } else if let Some(u) = read_internal_integer()? {
    Some((u.value_of(), UNITY)) // These are coerced to number=>sp
  } else if let Some(u) = read_internal_dimension()? {
    Some((u.value_of(), UNITY))
  } else if let Some(u) = read_internal_glue()? {
    Some((u.value_of(), UNITY))
  } else {
    read_keyword(&["true"])?; // But ignore, we're not bothering with mag...
    if let Some(u) = read_keyword(&["pt", "pc", "in", "bp", "cm", "mm", "dd", "cc", "sp", "px"])? {
      skip_one_space(true)?;
      Some(convert_unit_ratio(&u))
    } else {
      None
    }
  };
  Ok(unit_opt)
}

//======================================================================
// Glue
//======================================================================
// <glue> = <optional signs><internal glue> | <dimen><stretch><shrink>
// <stretch> = plus <dimen> | plus <fil dimen> | <optional spaces>
// <shrink>  = minus <dimen> | minus <fil dimen> | <optional spaces>
pub fn read_glue() -> Result<Glue> {
  let is_negative = read_optional_signs()?;
  if let Some(n) = read_internal_glue()? {
    if is_negative { Ok(n.negate()) } else { Ok(n) }
  } else {
    let mut d = read_dimension()?;
    if is_negative {
      d = d.negate();
    }
    let (r1, f1) = match read_keyword(&["plus"])? {
      Some(_) => read_rubber(false)?,
      None => (None, None),
    };
    let (r2, f2) = match read_keyword(&["minus"])? {
      Some(_) => read_rubber(false)?,
      None => (None, None),
    };

    Ok(Glue::new_spec(
      &d.value_of().to_string(),
      r1.map(|v| v as f64),
      f1,
      r2.map(|v| v as f64),
      f2,
    ))
  }
}

pub fn read_rubber(mu: bool) -> Result<(Option<i64>, Option<FillCode>)> {
  let is_negative = read_optional_signs()?;
  let s = if is_negative { -1 } else { 1 };
  match read_factor()? {
    None => {
      let f = if mu {
        read_mu_dimension()?.value_of()
      } else {
        read_dimension()?.value_of()
      };
      Ok((Some(f * s), None))
    },
    Some(f) => match read_keyword(&["filll", "fill", "fil"])? {
      Some(fil) => Ok((Some(fixpoint(s as f64 * f, None)), FillCode::from(&fil))),
      None => {
        let ratio = if mu {
          match read_mu_unit()? {
            None => {
              Warn!(
                "expected",
                "<unit>",
                "Illegal unit of measure (mu inserted)."
              );
              None
            },
            some => some,
          }
        } else {
          match read_unit()? {
            None => {
              Warn!(
                "expected",
                "<unit>",
                "Illegal unit of measure (pt inserted)."
              );
              None
            },
            some => some,
          }
        };
        let val = s as f64 * f;
        let sp = match ratio {
          Some((num, den)) => fixpoint_unit(val, num, den),
          None => fixpoint(val, None),
        };
        Ok((Some(sp), None))
      },
    },
  }
}

//======================================================================
// Mu Glue
//======================================================================
// <muglue> = <optional signs><internal muglue> | <mudimen><mustretch><mushrink>
// <mustretch> = plus <mudimen> | plus <fil dimen> | <optional spaces>
// <mushrink> = minus <mudimen> | minus <fil dimen> | <optional spaces>
pub fn read_mu_glue() -> Result<MuGlue> {
  let is_negative = read_optional_signs()?;
  if let Some(n) = read_internal_mu_glue()? {
    Ok(if is_negative { n.negate() } else { n })
  } else {
    let mut d = read_mu_dimension()?;
    if is_negative {
      d = d.negate()
    }
    let (r1, f1) = if read_keyword(&["plus"])?.is_some() {
      read_rubber(true)?
    } else {
      (None, None)
    };
    let (r2, f2) = if read_keyword(&["minus"])?.is_some() {
      read_rubber(true)?
    } else {
      (None, None)
    };
    Ok(MuGlue::new_full(d.value_of(), r1, f1, r2, f2))
  }
}

//======================================================================
// Mu Dimensions
//======================================================================
// <mudimen> = <optional signs><unsigned mudimem>
// <unsigned mudimen> = <normal mudimen> | <coerced mudimen>
// <normal mudimen> = <factor><mu unit>
// <mu unit> = <optional spaces><internal muglue> | mu <one optional space>
// <coerced mudimen> = <internal muglue>
pub fn read_mu_dimension() -> Result<MuDimension> {
  let is_negative = read_optional_signs()?;
  if let Some(mut m) = read_factor()? {
    let munit = read_mu_unit()?;
    if munit.is_none() {
      Warn!(
        "expected",
        "<unit>",
        "Illegal unit of measure (mu inserted)."
      );
    }
    if is_negative {
      m *= -1.0;
    }
    let sp = match munit {
      Some((num, den)) => fixpoint_unit(m, num, den),
      None => fixpoint(m, None),
    };
    Ok(MuDimension::new(sp))
  } else if let Some(mglue) = read_internal_mu_glue()? {
    let m = if is_negative { mglue.negate() } else { mglue };
    Ok(MuDimension::new(m.value_of()))
  } else {
    Warn!("expected", "<mudimen>", "Expecting mudimen; assuming 0");
    Ok(MuDimension::new(0))
  }
}

pub fn read_mu_unit() -> Result<Option<(i64, i64)>> {
  if read_keyword(&["mu"])?.is_some() {
    skip_one_space(true)?;
    Ok(Some((UNITY, UNITY))) // effectively, scaled mu
  } else if let Some(m) = read_internal_mu_glue()? {
    Ok(Some((m.value_of(), UNITY)))
  } else {
    Ok(None)
  }
}

fn read_internal_mu_glue() -> Result<Option<MuGlue>> {
  match read_register_value(RegisterType::MuGlue)? {
    None => Ok(None),
    Some(val) => Ok(Some(val.into())),
  }
}

/// Apparent behaviour of a token value (ie `\toks#=<arg>`)
pub fn read_tokens_value() -> Result<Tokens> {
  match read_non_space()? {
    None => Ok(Tokens!()),
    Some(token) => {
      // Perl: $$token[1] == CC_BEGIN — direct catcode check
      if token.get_catcode() == Catcode::BEGIN {
        Ok(read_balanced(ExpansionLevel::Off, false, false)?)
      } else {
        match lookup_register_definition(&token) {
          Some(defn) => {
            match defn.register_type() {
              Some(RegisterType::Tokens) | Some(RegisterType::Token) => {
                // TODO: The mismatch between Vec<Tokens> for read_arguments and Vec<Token> for
                // value_of feels incorrect       but in which direction should it be
                // resolved?
                let args = defn.read_arguments()?;
                match defn.value_of(args) {
                  None => Ok(Tokens!()),
                  Some(v) => Ok(v.into()),
                }
              },
              _ => Ok(Tokens!(token)),
            }
          },
          _ => {
            match lookup_definition(&token)? {
              Some(defn) => {
                // TODO: we are doing two lookups to avoid the type restriction of .read_arguments,
                // any way to circumvent? Is it slow in the first place?
                if defn.is_expandable() {
                  let x = defn.invoke(false)?;
                  if !x.is_empty() {
                    unread(x);
                  }
                  read_tokens_value()
                } else {
                  Ok(Tokens!(token))
                }
              },
              _ => Ok(Tokens!(token)),
            }
          },
        }
      }
    },
  }
}

pub fn skip_spaces() -> Result<()> {
  if let Some(t) = read_non_space()? {
    unread_one(t);
  }
  Ok(())
}

/// Check if a token is a space token (catcode SPACE) or an "implicit space"
/// (a CS or ACTIVE token `\let` to a space token).
/// See TeXbook p269: `<one optional space>` absorbs both explicit and implicit spaces.
fn is_space_or_implicit_space(token: &Token) -> bool {
  if token.get_catcode() == Catcode::SPACE {
    return true;
  }
  // Check for implicit space: CS/ACTIVE let to a space token
  if token.get_catcode() == Catcode::CS || token.get_catcode() == Catcode::ACTIVE {
    return with_meaning(
      token,
      |m| matches!(m, Some(Stored::Token(t)) if t.get_catcode() == Catcode::SPACE),
    );
  }
  false
}

/// Skip one optional space.
/// If `expanded` is true, acts like `<one optional space>` and expands tokens (readXToken).
/// Perl: skip1Space($self, $expanded)
pub fn skip_one_space(expanded: bool) -> Result<()> {
  let token = if expanded {
    read_x_token(None, false, None)?
  } else {
    read_token()?
  };
  if let Some(t) = token
    && !is_space_or_implicit_space(&t)
  {
    unread_one(t);
  }
  Ok(())
}

//======================================================================
// some helpers...

// <optional signs> = <optional spaces> | <optional signs><plus or minus><optional spaces>
// returns false if None, or positive, true if negative
pub fn read_optional_signs() -> Result<bool> {
  let mut sign = false;
  while let Some(t) = read_x_token(None, false, None)? {
    let sym = t.get_sym();
    if sym == pin!("-") {
      sign = !sign;
    } else if (sym != pin!("+")) && !is_space_or_implicit_space(&t) {
      unread_one(t); // Unread and end
      break;
    }
  }
  Ok(sign)
}

fn read_digits(range_regex: &Regex, skip: bool) -> Result<String> {
  let mut result = String::new();
  while let Some(token) = read_x_token(None, false, None)? {
    let digit_opt = token.with_str(|s| {
      if s.len() == 1 && range_regex.is_match(s) {
        s.chars().next()
      } else {
        None
      }
    });
    if let Some(digit) = digit_opt {
      result.push(digit);
    } else {
      if !(skip && is_space_or_implicit_space(&token)) {
        unread_one(token);
      }
      break;
    }
  }
  Ok(result)
}

// ```
// <factor> = <normal integer> | <decimal constant>
// <decimal constant> = . | , | <digit><decimal constant> | <decimal constant><digit>
// ```
/// Return a number (Rust f64 number)
pub fn read_factor() -> Result<Option<f64>> {
  let mut factor = read_digits(&DIGIT_RE, false)?;
  let mut token_opt = read_x_token(None, false, None)?;
  if let Some(ref token) = token_opt {
    let sym = token.get_sym();
    if sym == pin!(".") || sym == pin!(",") {
      factor = s!("{}.{}", factor, read_digits(&DIGIT_RE, false)?);
      token_opt = read_x_token(None, false, None)?;
    }
  }

  // Note: zero is an edge case with the unwrap_or fallback, handle it
  if !factor.is_empty() {
    let factor_f64: f64 = factor.parse::<f64>().unwrap_or(0.0);
    if let Some(token) = token_opt
      && token.get_catcode() != Catcode::SPACE
    {
      unread_one(token);
    }
    Ok(Some(factor_f64))
  } else {
    if let Some(token) = token_opt {
      unread_one(token);
    }
    match read_normal_integer()? {
      None => Ok(None),
      Some(n) => Ok(Some(n.value_of() as f64)),
    }
  }
}

pub fn do_expand<T: Into<Tokens>>(tokens: T) -> Result<Tokens> {
  let tokens: Tokens = tokens.into();
  reading_from_mouth(Mouth::default(), move || -> Result<Tokens> {
    {
      unread_one(T_END!());
      unread(tokens);
      unread_one(T_BEGIN!());
    }
    read_balanced(ExpansionLevel::Full, false, true)
  })
}

pub fn do_expand_partially<T: Into<Tokens>>(tokens: T) -> Result<Tokens> {
  let tokens: Tokens = tokens.into();
  reading_from_mouth(Mouth::default(), move || -> Result<Tokens> {
    {
      unread_one(T_END!());
      unread(tokens);
      unread_one(T_BEGIN!());
    }
    read_balanced(ExpansionLevel::Partial, false, true)
  })
}

pub fn is_column_end(token: &Token) -> Option<(Token, &'static str, bool)> {
  match token.get_catcode() {
    Catcode::ALIGN => Some((*token, "align", false)),
    Catcode::CS | Catcode::ACTIVE => {
      // Embedded version of Equals, knowing both are tokens
      let defn = lookup_meaning(token).unwrap_or_else(|| Stored::Token(*token));
      // Perl Gullet.pm L273: if meaning is a Token with CC_ALIGN, treat as alignment tab
      if let Stored::Token(t) = &defn
        && t.get_catcode() == Catcode::ALIGN
      {
        return Some((*token, "align", false));
      }
      for end in *COLUMN_ENDS {
        let e = &end.0;
        // Would be nice to cache the defns, but don't know when they're present & constant!
        if defn == lookup_meaning(e).unwrap_or_else(|| Stored::Token(*e)) {
          return Some(end);
        }
      }
      None
    },
    _ => None,
  }
}
/// Handle a marker token, by updating the current alignment group count
fn handle_marker(marker_token: Token) {
  marker_token.with_str(|arg| match arg {
    "before-column" => {
      // Were in before-column template
      set_align_group_count(0);
    }, // switch to column proper!
    "after-column" => { // Were in before-column template
      // let alignment = lookup_alignment();
      // Debug("Halign $alignment: alignment  after column") if $LaTeXML::DEBUG{halign};
    },
    _ => {},
  });
}

/// Do something, while reading tokens from a specific Mouth.
///
/// This reads ONLY from that mouth (or any mouth openned by code in that source),
/// and the mouth should end up empty afterwards, and only be closed here.
pub fn reading_from_mouth<R, FnR>(mouth: Mouth, reader: FnR) -> Result<R>
where FnR: FnOnce() -> Result<R> {
  let context_mouth_source = arena::pin(mouth.get_source());
  // A cycle is only a cycle WITHIN one expansion context. Rather than
  // resetting the guard history here (an earlier fix that also BLINDED the
  // guard to outer loops whose body calls `do_expand` each iteration), give
  // this reading context a fresh SERIAL that `cycle_guard_checkpoint` mixes
  // into every fingerprint: windows can never match across contexts, so
  // consecutive identical short expansions — the math0402448 xymatrix
  // per-cell `get_xmarg_id` stream that false-positived as a "loop" — stay
  // inert, while the OUTER context's serial (restored on exit) keeps outer
  // periodicity intact and detectable. PR #249 review P2-7.
  {
    let mut g = gullet_mut!();
    let outer = g.ctx_serial;
    g.ctx_stack.push(outer);
    g.ctx_next += 1;
    g.ctx_serial = g.ctx_next;
  }
  open_mouth(mouth, false); // only allow mouth to be explicitly closed here.
  let reader_result = reader();
  // Reading in this context is over (whether Ok or Err): restore the outer
  // context's serial before any cleanup/return path below.
  {
    let mut g = gullet_mut!();
    let restored = g.ctx_stack.pop().unwrap_or(0);
    g.ctx_serial = restored;
  }
  // If the reader returned an error (e.g., Fatal from token limit),
  // we STILL need to clean up the mouth to preserve the caller's state.
  let results: R = match reader_result {
    Ok(v) => v,
    Err(e) => {
      // Force-close our mouth and any autoclosable mouths above it
      loop {
        let current = gullet!()
          .runtime
          .as_ref()
          .map(|r| arena::pin(r.mouth.get_source()));
        if current == Some(context_mouth_source) {
          close_mouth(true).ok();
          break;
        } else if gullet!().mouthstack.is_empty() {
          break; // Our mouth was already consumed
        } else {
          close_mouth(true).ok(); // Close stale mouth above ours
        }
      }
      // Reset progress counter so subsequent processing isn't immediately killed
      gullet_mut!().progress = 0;
      return Err(e);
    },
  };
  // `mouth` must still be open, with (at worst) empty autoclosable mouths in front of it.
  // Rate-limit the "mouth closed" error — when the gullet gets into a state
  // where the cleanup loop keeps finding stale mouths above the target, the
  // same error can fire on EVERY caller of reading_from_mouth. Arxiv 0906.1883
  // (birkmult + local .cls) can trigger 10K+ such firings, one per stack frame.
  // Fatal out after 50 repeat firings so the process surfaces a clear "we lost
  // the mouth stack" signal instead of filling the log with identical messages.
  thread_local! {
    static MOUTH_CLOSED_ERRORS: Cell<usize> = const { Cell::new(0) };
  }
  fn record_mouth_closed_error() { MOUTH_CLOSED_ERRORS.with(|c| c.set(c.get().saturating_add(1))); }
  fn should_emit_mouth_closed() -> bool { MOUTH_CLOSED_ERRORS.with(|c| c.get() < 10) }
  fn mouth_closed_budget_exhausted() -> bool { MOUTH_CLOSED_ERRORS.with(|c| c.get() >= 50) }
  loop {
    let mouth_source = gullet!()
      .runtime
      .as_ref()
      .map(|r| arena::pin(r.mouth.get_source()));
    if mouth_source == Some(context_mouth_source) {
      close_mouth(true)?;
      break;
    } else if gullet!().mouthstack.is_empty() {
      if should_emit_mouth_closed() {
        // `arena::to_string` clones the resolved &str into an owned String
        // BEFORE we hand it to Error! — a following `arena::pin` triggered
        // deep inside generate_message!/get_location() can grow the
        // interner's buffer and invalidate a borrowed &str (observed as
        // garbled, buffer-adjacent symbol content in 0906.1883 errors).
        let src = arena::to_string(context_mouth_source);
        Error!(
          "unexpected",
          "<closed>",
          "Mouth is unexpectedly already closed",
          s!("Reading from {src}, but it has already been closed.")
        );
      }
      record_mouth_closed_error();
      if mouth_closed_budget_exhausted() {
        Fatal!(
          Stomach,
          Recursion,
          "Too many unexpectedly-closed mouth errors (>50); gullet mouth-stack state is inconsistent"
        );
      }
      break;
    } else {
      let is_autoclosable = gullet!()
        .runtime
        .as_ref()
        .map(|r| r.autoclose)
        .unwrap_or(false);
      if is_autoclosable {
        // Auto-closable mouth (e.g. from \scantokens, raw_tex) — safe to close
        close_mouth(true)?;
      } else {
        // Non-autoclosable mouth that isn't our target — this means our target
        // mouth was already consumed. Don't close this mouth (it belongs to an
        // outer reading_from_mouth call). Just error and stop.
        if should_emit_mouth_closed() {
          let src = arena::to_string(context_mouth_source);
          Error!(
            "unexpected",
            "<closed>",
            "Mouth is unexpectedly already closed",
            s!(
              "Reading from {src}, but it has already been closed (found different non-closable mouth on top)."
            )
          );
        }
        record_mouth_closed_error();
        if mouth_closed_budget_exhausted() {
          Fatal!(
            Stomach,
            Recursion,
            "Too many unexpectedly-closed mouth errors (>50); gullet mouth-stack state is inconsistent"
          );
        }
        break;
      }
    }
  }
  Ok(results)
}

/// Check if there is more input to be read from the current mouth
pub fn has_more_input() -> bool {
  match runtime!() {
    Some(ref mut runtime) => runtime.mouth.has_more_input(),
    None => false,
  }
}

/// Obscure, but the only way I can think of to End!! (see \bye or \end{document})
/// Flush all sources (close all pending mouth's)
pub fn flush() {
  let mut g = gullet_mut!();
  if let Some(ref mut runtime) = g.runtime {
    runtime.mouth.finish();
  }
  while !g.mouthstack.is_empty() {
    if let Some(mut entry) = g.mouthstack.pop_front() {
      entry.mouth.finish();
    }
  }
  g.runtime = Some(MouthRuntime {
    mouth:     Mouth::default(),
    pushback:  Vec::with_capacity(128),
    autoclose: true,
  });
  g.mouthstack = VecDeque::new();
}

/// Execute a function with a mutable reference to the current mouth
pub fn with_mouth_mut<FnR, R>(caller: FnR) -> R
where FnR: FnOnce(Option<&mut Mouth>) -> R {
  let mut gullet = gullet_mut!();
  let mouth_opt = match gullet.runtime {
    None => None,
    Some(ref mut runtime) => Some(&mut runtime.mouth),
  };
  caller(mouth_opt)
}

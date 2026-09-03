use std::{
  borrow::Cow,
  cell::{Cell, RefCell},
  collections::VecDeque,
  rc::Rc,
  time::Instant,
};

use once_cell::sync::Lazy;

use crate::{digested::DigestedData, pin};

/// Cached snapshot of `LXML_TRACE_BOUND_MODE` env var. Like the
/// `TRACE_GROUP_END` cache in gullet.rs, this avoids per-digest
/// `getenv` calls — glibc's `getenv` is unsafe under high-volume
/// concurrent reads from many test threads, manifesting as SIGSEGV
/// in `__GI_getenv` when running `cargo test --release --tests`.
/// Sample once at static-init; subsequent reads are an atomic load.
static TRACE_BOUND_MODE: Lazy<bool> = Lazy::new(|| std::env::var("LXML_TRACE_BOUND_MODE").is_ok());
/// `LXML_TRACE_FRAMES=1`: one line per stack-frame push/pop with the depth, the
/// owning token and (on pop) the bound mode — the save-stack view of a
/// group/box/mode imbalance.
static TRACE_FRAMES: Lazy<bool> = Lazy::new(|| std::env::var("LXML_TRACE_FRAMES").is_ok());

// Conversion timeout: thread-local deadline. When set, digest loops check it.
thread_local! {
  static CONVERSION_DEADLINE: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// Set a conversion timeout (seconds from now). 0 = no timeout.
pub fn set_timeout(seconds: u64) {
  if seconds > 0 {
    CONVERSION_DEADLINE.with(|d| {
      d.set(Some(
        Instant::now() + std::time::Duration::from_secs(seconds),
      ))
    });
  } else {
    CONVERSION_DEADLINE.with(|d| d.set(None));
  }
}

// Explicit override for the cooperative soft-RSS budget (bytes). When set it
// takes precedence over the `LATEXML_RSS_CAP_BYTES` env; when `None` the env /
// built-in default applies. The binary sets it from the single `--max-memory`
// knob (via [`soft_cap_from_ceiling`]) so that ONE flag governs the whole
// memory limit — this cooperative fuse rides a fixed fraction below the hard
// Watchdog ceiling rather than being an independent number, and `--max-memory=0`
// disables both (this fuse via a `0` cap → `None`, the Watchdog via
// `max_rss_kb == 0`).
thread_local! {
  static RSS_CAP_OVERRIDE: Cell<Option<u64>> = const { Cell::new(None) };
}

/// Override the cooperative soft-RSS memory budget, in bytes. `Some(0)`
/// disables the budget; `Some(n)` caps at `n` bytes; `None` restores the
/// `LATEXML_RSS_CAP_BYTES` env / built-in default. Mirrors the `--max-memory`
/// CLI convention where `0` means "no limit". See `resolve_rss_cap` (private)
/// for the precedence order.
pub fn set_memory_cap(bytes: Option<u64>) { RSS_CAP_OVERRIDE.with(|c| c.set(bytes)); }

/// Derive the cooperative soft-RSS budget (bytes) from the hard `--max-memory`
/// ceiling (MiB). The soft fuse sits at 75% of the ceiling, leaving ~25%
/// headroom for the post-processing phase (libxml DOM + XSLT) that runs above
/// digestion and which this cooperative guard cannot see. `0` in → `0` out
/// (disabled), so `--max-memory=0` disables the whole memory limit. This keeps
/// `--max-memory` the single knob: the hard Watchdog rides the ceiling, this
/// fuse rides a fixed fraction below it — no independent second number.
///
/// The 75% factor reproduces the historical ~4.5 GB-under-6 GiB relationship at
/// the 6144 MiB default (→ 4608 MiB) while scaling with any user-chosen ceiling
/// (so a tight `--max-memory` also gets the graceful cooperative failure first,
/// and a generous one raises both guards together).
pub fn soft_cap_from_ceiling(max_memory_mib: u64) -> u64 {
  (max_memory_mib.saturating_mul(3) / 4).saturating_mul(1024 * 1024)
}

/// Minimum boxes that must accumulate before the **soft-RSS** yield branch may
/// fire (the box-budget branch is unaffected and still yields on its own).
///
/// The soft-RSS test is a LEVEL test with no hysteresis: `rss > watermark`. A
/// document whose irreducible resident floor sits above the watermark therefore
/// latches it on permanently and yields at every legal seam, accumulating
/// almost nothing between yields. Measured on the 131 MB witness at
/// `--max-memory 48000` (watermark 12 GB, pass-1 RSS 13.3-14.9 GB — above it
/// for the entire run): **24,051,712 yields** producing **459,579 segments
/// averaging 5.5 KB**, against a box budget of ~2.0 M boxes that would on its
/// own have yielded ~12 times. The same binary on a witness that never crosses
/// its watermark yields **8** times.
///
/// A floor restores the trigger's intent — "respond to memory pressure sooner
/// than the box budget would" — without the degenerate per-seam case. 1024
/// boxes is ~2.5 MB of box memory at the measured 2416 B/box, i.e. negligible
/// against any watermark large enough to matter, so the pressure response stays
/// effectively immediate while the yield count drops by ~3 orders of magnitude.
///
/// **The floor is waived under real pressure** — see `soft_yield_is_urgent`.
/// The soft-RSS branch exists because "the box budget alone assumes a per-box
/// footprint; on content whose real cost per box is higher (math-dense trees),
/// RSS crosses the ceiling long before the box count does". A floor that
/// applied unconditionally would blunt that valve for exactly the pathological
/// input it was added for: 1024 boxes of ordinary content is ~2.5 MB, but 1024
/// boxes of something pathological is unbounded, and the fuse could fire inside
/// one un-yielded window. So above a higher RSS mark the floor is ignored.
///
/// Env-overridable for calibration only (`LATEXML_SOFT_YIELD_MIN_BOXES`),
/// deliberately not a CLI flag — same reasoning as `LATEXML_SPILL_AT_MIB`.
/// Override the soft-RSS floor directly, bypassing the env lookup. For tests
/// that need to drive the degenerate (floor = 1) and fixed (floor = N) regimes
/// in one process — see `115_soft_yield_floor`.
pub fn set_soft_yield_min_boxes(boxes: usize) { SOFT_YIELD_MIN_BOXES.set(Some(boxes)); }

/// Is memory pressure URGENT enough to waive the soft-yield floor?
///
/// Halfway from the spill watermark to the cooperative fuse. Below this the
/// floor applies and yields stay coarse; above it every legal seam yields, as
/// before this floor existed — so a document whose per-box footprint is wildly
/// larger than the 2416 B the box budget assumes still gets the immediate
/// response the soft-RSS branch was introduced to provide, instead of Fatal-ing
/// inside a 1024-box window.
fn soft_yield_is_urgent(rss_kb: u64) -> bool {
  soft_yield_urgency(rss_kb, spill_watermark_bytes(), resolve_rss_cap())
}

/// The pure predicate behind [`soft_yield_is_urgent`], split out so the
/// arithmetic is unit-testable (an integration test would need a real RSS cap
/// near the test process's actual footprint — flaky by construction on the
/// 16 GB CI runner). `watermark`/`fuse` in bytes, `rss_kb` in KiB.
fn soft_yield_urgency(rss_kb: u64, watermark: Option<u64>, fuse: Option<u64>) -> bool {
  match (watermark, fuse) {
    (Some(watermark), Some(fuse)) if fuse > watermark => {
      rss_kb.saturating_mul(1024) >= watermark + (fuse - watermark) / 2
    },
    // No fuse to divide (`--max-memory=0`): the floor always applies, matching
    // the watermark fallback's own "no ceiling to race" reasoning.
    _ => false,
  }
}

/// Cached: this sits on the per-seam yield predicate, which the 131 MB witness
/// evaluates tens of millions of times — an `std::env::var` there would be its
/// own hotspot.
pub fn soft_yield_min_boxes() -> usize {
  const DEFAULT: usize = 1024;
  if let Some(cached) = SOFT_YIELD_MIN_BOXES.get() {
    return cached;
  }
  let resolved = std::env::var("LATEXML_SOFT_YIELD_MIN_BOXES")
    .ok()
    .and_then(|v| v.parse::<usize>().ok())
    .unwrap_or(DEFAULT);
  SOFT_YIELD_MIN_BOXES.set(Some(resolved));
  resolved
}

/// The RAM watermark, in bytes, at which streaming pass 1 begins spilling
/// closed subtrees to disk — the second derived quantity of the single
/// `--max-memory` knob, and deliberately NOT a flag of its own (a watermark a
/// user could raise above the fuse would Fatal before it ever spilled).
///
/// **A third of the cooperative fuse.** Not a half: the yields fire only at
/// legal seams (a large alignment digests straight through any threshold), the
/// RSS sample lags by up to 1024 guard ticks, and per-run bookkeeping creeps
/// monotonically — measured on the 131 MB witness, a half-of-fuse watermark
/// steadied pass 1 around 33 GB and the creep then walked it into the 37.7 GB
/// fuse and died, where a third completed at 28.1 GB peak.
///
/// **With `--max-memory=0` there is no fuse to divide, and the watermark must
/// still exist**: disabling the death ceiling says "do not kill me", not "let
/// the machine run out". Fall back to an eighth of physical RAM, which lands
/// the same 12 GiB on a 96 GB host that the validated 48 GiB ceiling derives.
pub fn spill_watermark_bytes() -> Option<u64> {
  // Calibration override (`LATEXML_SPILL_AT_MIB`), deliberately env-only and
  // NOT a CLI flag: a user-settable watermark could be raised above the fuse,
  // producing a run that Fatals before it ever spills. It exists so the
  // fuse-fraction below can be re-derived by measurement rather than argued.
  if let Some(mib) = std::env::var("LATEXML_SPILL_AT_MIB")
    .ok()
    .and_then(|v| v.parse::<u64>().ok())
    .filter(|mib| *mib > 0)
  {
    return Some(mib.saturating_mul(1024 * 1024));
  }
  match resolve_rss_cap() {
    Some(fuse) => Some(fuse / 3),
    None => crate::watchdog::total_memory_bytes().map(|ram| ram / 8),
  }
}

/// Apply the single `--max-memory` ceiling (MiB) to this thread's cooperative
/// soft fuse, so the one knob means the same thing on every conversion path.
///
/// **`--max-memory` wins over `LATEXML_RSS_CAP_BYTES`, unconditionally.** The
/// flag is the single knob; an env var must not silently override what the user
/// typed. This deliberately overwrites the env, which is why the env keeps its
/// meaning exactly where no flag exists to contradict it: embedders that never
/// parse CLI arguments and so never reach this function — the library test
/// harness (`util::test`, which pins 9 GB) and the `cortex_worker` fleet (which
/// pins each child to its `--max-rss-mb`). Both are unaffected.
///
/// Callers must be EVERY conversion path: the plain one, the `--server` forked
/// body child, and the in-process fallback. When only the first called it,
/// `--server --max-memory=0` still ran against a live 4.5 GB fuse while the help
/// text promised the limit was off.
pub fn apply_memory_ceiling(max_memory_mib: u64) {
  set_memory_cap(Some(soft_cap_from_ceiling(max_memory_mib)));
}

/// Resolve the effective soft-RSS budget: `None` = disabled (no ceiling),
/// `Some(n)` = abort above `n` bytes. Precedence: the explicit
/// [`set_memory_cap`] override, else `LATEXML_RSS_CAP_BYTES`, else the 4.5 GB
/// default. A cap of `0` from EITHER source resolves to `None`, so
/// `--max-memory=0` / `LATEXML_RSS_CAP_BYTES=0` mean "no limit" — not "abort
/// immediately" (a literal `0` compared as `rss_bytes > 0` is always true).
///
/// Note the env is consulted only when nothing set the override — i.e. only for
/// embedders that never call [`apply_memory_ceiling`]. Every path in the
/// `latexml_oxide` binary calls it, so there `--max-memory` always wins.
pub fn resolve_rss_cap() -> Option<u64> {
  let cap = RSS_CAP_OVERRIDE.with(|c| c.get()).unwrap_or_else(|| {
    std::env::var("LATEXML_RSS_CAP_BYTES")
      .ok()
      .and_then(|v| v.parse::<u64>().ok())
      .unwrap_or(4_500_000_000)
  });
  (cap > 0).then_some(cap)
}

/// Check if conversion has timed out. Returns Err if deadline exceeded.
///
/// Also samples RSS via /proc/self/status every ~1024 calls and raises
/// `Fatal:oom:memory_budget` if the process is approaching the worker
/// memory cap. R35.A witnesses (plain-TeX `$$\displaylines{ … \picture
/// … }$$`, 7 sandbox papers from 1999–2006) trigger a runaway where
/// `set_alloc_error_hook` fires AFTER the process has already allocated
/// ~5+ GB; that hook can't easily walk back the call site under
/// `panic="unwind"`. Sampling RSS here at well below the OS ulimit
/// gives us a clean diagnostic and a unwound stack via `fatal!`.
pub fn check_timeout() -> Result<()> {
  // Box-list cycle guard fired in `push_box_list` (which cannot unwind) —
  // surface it here, the regular Result-returning digestion checkpoint.
  // (The `stomach_mut!` macro is defined textually below; use STOMACH
  // directly, with a try-borrow so a transient borrow just defers to the
  // next tick.)
  let pending = STOMACH
    .try_borrow_mut()
    .ok()
    .and_then(|mut s| s.pending_cycle_fatal.take());
  if let Some((category, msg)) = pending {
    use crate::common::error::{Error as LatexmlError, ErrorTarget};
    return Err(LatexmlError {
      target: ErrorTarget::Stomach,
      category,
      message: msg,
    });
  }
  CONVERSION_DEADLINE.with(|d| {
    if let Some(deadline) = d.get()
      && Instant::now() > deadline
    {
      fatal!(Timeout, Convert, "Conversion timed out!");
    }
    Ok(())
  })?;
  // Soft memory budget: every ~1024 calls, peek at our own RSS.
  // 1024-call cadence keeps overhead negligible on the hot path
  // (each call reads /proc/self/statm — a single syscall).
  std::thread_local! {
    static MEM_TICK: Cell<usize> = const { Cell::new(0) };
  }
  let tick = MEM_TICK.with(|t| {
    let v = t.get().wrapping_add(1);
    t.set(v);
    v
  });
  if tick & 0x3FF == 0 {
    // Single RSS-reading seam: `watchdog::process_rss_kb` (this was a second
    // hand-rolled /proc parser; PR #249 review P3-12). When the watchdog
    // grows macOS/Windows backends, this cap follows for free.
    {
      {
        if let Some(rss_kb) = crate::watchdog::process_rss_kb() {
          LAST_SAMPLED_RSS_KB.set(rss_kb);
          let rss_bytes = rss_kb * 1024;
          // R35.A safety cap: 4.5 GB RSS. Real documents in the wp5 /
          // canvas3 corpus stay below 1 GB peak RSS, so this is well
          // into pathological territory while leaving headroom for
          // post-processing (XSLT, MathML chain).
          // Override via LATEXML_RSS_CAP_BYTES env or `set_memory_cap`; a
          // value of 0 (or `--max-memory=0`) disables it — see
          // `resolve_rss_cap`.
          //
          // This is a *per-process* fuse, deliberately kept LOW. It must
          // bound ONE conversion: in production the binary is
          // single-conversion (one paper per process), and a massively
          // parallel fleet runs many such processes at once — so the
          // aggregate host RSS is `N_processes × this_cap`. Raising the
          // default would let a busy fleet OOM the machine. The
          // `cortex_worker` fleet OVERRIDES this env to its own per-child
          // ceiling (`--child-mem-limit-mb`).
          //
          // The ONE multi-conversion-in-one-process case is the test
          // harness: libtest spawns a thread per test, so at `cargo
          // test`'s default parallelism on a many-core box (e.g. -j128)
          // the process-wide RSS is the *sum* over all in-flight
          // conversions and trips this single-conversion cap on
          // otherwise-fine documents. That is handled NOT by raising this
          // default but by the harness setting LATEXML_RSS_CAP_BYTES at
          // test setup (latexml_oxide `util::test::init_test_rss_cap`).
          // Any other single-process-many-conversion driver should do the
          // same.
          if let Some(cap) = resolve_rss_cap()
            && rss_bytes > cap
          {
            // R35.A debug: when LATEXML_DEBUG_MEMBUDGET=1 is set, dump
            // a stack backtrace before exiting so we can identify the
            // expansion loop responsible. Backtrace allocation is
            // fine here — we haven't hit the OS ulimit yet (we're
            // 1.5 GB below it by default).
            if std::env::var_os("LATEXML_DEBUG_MEMBUDGET").is_some() {
              eprintln!(
                "[membudget] RSS {} MB > cap {} MB — dumping backtrace",
                rss_bytes / 1_000_000,
                cap / 1_000_000
              );
              // Permanent LATEXML_DEBUG_MEMBUDGET diagnostic: which
              // accumulating list is growing? (MEMORY.md's OOM-diagnosis
              // recipe depends on this dump — do not remove as "temp".)
              if let Ok(st) = STOMACH.try_borrow() {
                eprintln!(
                  "[membudget] box_list={} (~{} MB est) token_stack={} boxing={} localized_box_list_total={}",
                  st.box_list.len(),
                  estimate_box_list_bytes(&st.box_list) / 1_000_000,
                  st.token_stack.len(),
                  st.boxing.len(),
                  st.localized_box_list.iter().map(|v| v.len()).sum::<usize>(),
                );
              }
              if let Ok(g) = gullet::GULLET.try_borrow() {
                let pb = g.runtime.as_ref().map(|r| r.pushback.len()).unwrap_or(0);
                eprintln!("[membudget] gullet pushback={pb} progress={}", g.progress);
              }
              let bt = std::backtrace::Backtrace::force_capture();
              eprintln!("{bt}");
            }
            // The actionable half of the message is a KNOWN NEED, not an
            // anomaly: a document's peak scales with macro expansion and math
            // density (the 131 MB witness needs ~23 GB resident just to
            // stream through core), so the honest advice is "raise the
            // ceiling", plus the derived flag value so the user knows which
            // number they are raising — the cap here is the 75% fuse, not
            // the `--max-memory` figure they typed. The latch lets the
            // binary's end-of-run report add the kernel-tracked peak
            // (`watchdog::peak_memory_report`, emitted only when this fired).
            crate::watchdog::note_memory_fatal();
            fatal!(
              Timeout,
              MemoryBudget,
              format!(
                "Memory budget exceeded: RSS {} MB > cap {} MB (the cooperative fuse at 75% of \
                 --max-memory={}). This document needs a larger ceiling: rerun with a higher \
                 --max-memory on a machine with enough free RAM.",
                rss_bytes / 1_000_000,
                cap / 1_000_000,
                (cap * 4).div_ceil(3 * 1024 * 1024),
              )
            );
          }
        }
      }
    }
  }
  Ok(())
}

use crate::{
  BoxOps, Digested, TexMode,
  comment::Comment,
  common::{arena, arena::SymHashMap as HashMap, error::*, font, font::Font},
  definition::{
    Definition, constructor::Constructor, expandable::Expandable, register::RegisterValue,
  },
  gullet,
  list::List,
  mouth::{Mouth, MouthOptions},
  state::*,
  tbox::*,
  token::{Catcode, Token},
  tokens::Tokens,
};

static MAXSTACK: usize = 200;

/// The Stomach is responsible for digesting tokens into boxes, lists, etc.
#[derive(Default)]
pub struct Stomach {
  /// currently invoked tokens
  pub token_stack:     Vec<Token>,
  /// tracks the tokens of boxing groups(?)
  pub boxing:          Vec<Token>,
  /// localized box lists for stacked digestion calls
  localized_box_list:  Vec<Vec<Digested>>,
  /// collects the intermediate boxes resulting from a `digest` call.
  pub box_list:        Vec<Digested>,
  /// Windowed cycle detector over the accumulated digest list — the stomach
  /// analog of the gullet's expansion-stream guard. Catches box-accumulation
  /// runaways (a recursive macro/path that digests the same boxes forever, e.g.
  /// pgf's `to [loop]` arc on a pathological picture, 2201.09268) that bypass
  /// the gullet read loop entirely. Engaged only once `box_list` has grown far
  /// past any flushed-document size. See [`crate::cycle_guard`].
  cycle_guard:         crate::cycle_guard::CycleGuard,
  /// Set by the guarded box appenders when a stomach guard fires; consumed
  /// and turned into a `Fatal` by `check_timeout` (the next
  /// `Result`-returning checkpoint — `push_box_list` itself returns `()` and
  /// cannot unwind). Carries the structured category so size/byte/depth
  /// breaches report as `Stomach:MemoryBudget` while only genuine detected
  /// cycles report as `Stomach:Recursion` — canvas/telemetry clustering on
  /// `target:category` can tell them apart (PR #249 review P2-8).
  pending_cycle_fatal: Option<(ErrorCategory, String)>,
}

#[thread_local]
pub static STOMACH: Lazy<RefCell<Stomach>> = Lazy::new(|| RefCell::new(Stomach::default()));

// ---- Fragment yield (streaming mode) -------------------------------------
//
// Deliberately OUTSIDE the `Stomach` struct: these are driver-level
// configuration, like the RSS cap — `initialize_stomach` resets the digestion
// state between documents but must not forget that the driver asked for
// fragmented digestion.

/// When `Some(n)`, `digest_next_body` may YIELD — return the boxes accumulated
/// so far, gullet and State untouched — once the current level holds `n` boxes
/// AND the position is a legal fragment seam. `None` (default) = eager.
#[thread_local]
static FRAGMENT_YIELD_BUDGET: Cell<Option<usize>> = Cell::new(None);
/// Set on yield; read-and-cleared by the streaming driver to distinguish
/// "more to come" from EOF.
#[thread_local]
static FRAGMENT_YIELDED: Cell<bool> = Cell::new(false);
/// Total yields this conversion (telemetry + test probe).
#[thread_local]
static FRAGMENT_YIELD_COUNT: Cell<usize> = Cell::new(0);
/// Soft RSS threshold (KiB) above which the yield predicate fires regardless
/// of the box count. The box budget alone assumes a per-box footprint; on
/// content whose real cost per box is higher (math-dense trees, debug
/// builds), RSS crosses the ceiling long before the box count does —
/// measured on the 19.8 MB witness at cap 24 GB, where the fuse fired
/// during early fragments while the 2.6M-box budget sat untouched.
#[thread_local]
static FRAGMENT_YIELD_RSS_SOFT_KB: Cell<Option<u64>> = Cell::new(None);
/// Resolved-once floor for the soft-RSS branch — see [`soft_yield_min_boxes`].
#[thread_local]
static SOFT_YIELD_MIN_BOXES: Cell<Option<usize>> = Cell::new(None);
/// The most recent RSS sample from `check_timeout`'s 1024-call cadence, so
/// the yield predicate reads a cell instead of `/proc`.
#[thread_local]
static LAST_SAMPLED_RSS_KB: Cell<u64> = Cell::new(0);

/// Set (or clear) the soft-RSS yield threshold, in KiB.
pub fn set_fragment_yield_rss_soft_kb(kb: Option<u64>) { FRAGMENT_YIELD_RSS_SOFT_KB.set(kb); }

/// The most recent sampled RSS in KiB (0 until the first sample).
pub fn last_sampled_rss_kb() -> u64 { LAST_SAMPLED_RSS_KB.get() }

/// Ask digestion to yield at legal fragment seams once `budget` boxes have
/// accumulated at the current level (`None` restores eager digestion). Set by
/// the streaming pass-1 driver; the budget is a box COUNT — the driver derives
/// it from the byte ceiling via the measured per-box footprint, the same basis
/// as the box-list guards.
pub fn set_fragment_yield_budget(budget: Option<usize>) {
  let enabling = budget.is_some();
  FRAGMENT_YIELD_BUDGET.set(budget);
  FRAGMENT_YIELDED.set(false);
  // The count is a per-conversion probe: reset when a driver ENABLES
  // yielding, and preserved when it disables at end-of-digestion (the driver
  // clears the budget before the tail phases, and telemetry/tests read the
  // count after the conversion returns).
  if enabling {
    FRAGMENT_YIELD_COUNT.set(0);
  }
}

/// Did the last `digest_next_body` return because of the yield budget (rather
/// than EOF / terminal / depth-drop)? Read-and-clear.
pub fn take_fragment_yielded() -> bool { FRAGMENT_YIELDED.replace(false) }

/// How many times digestion has yielded since the budget was last set.
pub fn fragment_yield_count() -> usize { FRAGMENT_YIELD_COUNT.get() }

macro_rules! stomach {
  () => {
    (*STOMACH).borrow()
  };
}
macro_rules! stomach_mut {
  () => {
    (*STOMACH).borrow_mut()
  };
}

/// Initialize various stomach parameters, preload, etc.
pub fn initialize_stomach() {
  let mut stomach = stomach_mut!();
  stomach.boxing = Vec::new();
  stomach.token_stack = Vec::new();
  stomach.box_list = Vec::new();
  stomach.localized_box_list = Vec::new();
  stomach.cycle_guard.reset();
  stomach.pending_cycle_fatal = None;

  assign_value("BOUND_MODE", "vertical", Some(Scope::Global));
  assign_value("MODE", "vertical", Some(Scope::Global));
  assign_value("IN_MATH", false, Some(Scope::Global));
  assign_value("PRESERVE_NEWLINES", 1, Some(Scope::Global));
  assign_value(
    "afterGroup",
    Stored::VecDequeStored(VecDeque::new()),
    Some(Scope::Global),
  );
  assign_value("afterAssignment", Stored::None, Some(Scope::Global)); // undef ???
  assign_value_sym(
    crate::pin!("groupInitiator"),
    "Initialization",
    Some(Scope::Global),
  );
  // Setup default fonts.
  assign_value("font", Font::text_default(), Some(Scope::Global));
  assign_value("mathfont", Font::math_default(), Some(Scope::Global));
}

/// steal the previously digested boxes from the current level.
pub fn regurgitate() -> Vec<Digested> { std::mem::take(&mut stomach_mut!().box_list) }

//**********************************************************************
// Maintaining state
//**********************************************************************
// state changes that the Stomach needs to moderate and know about (?)

//======================================================================
// Dealing with TeX's bindings & grouping.
// Note that lookups happen more often than bgroup/egroup (which open/close frames).

/// Adds a new stack frame for a TeX group.
pub fn push_stack_frame(nobox: bool) {
  let current_token = get_current_token().unwrap_or_else(|| T_CS!("\\relax"));
  if *TRACE_FRAMES {
    eprintln!(
      "[frames] push nobox={nobox} depth {} -> {} at {current_token}",
      get_frame_depth(),
      get_frame_depth() + 1
    );
  }
  push_frame();
  assign_value(
    "beforeAfterGroup",
    Stored::VecDequeStored(VecDeque::new()),
    Some(Scope::Local),
  ); // ALWAYS bind this!
  assign_value(
    "afterGroup",
    Stored::VecDequeStored(VecDeque::new()),
    Some(Scope::Local),
  ); // ALWAYS bind this!
  assign_value("afterAssignment", Stored::None, Some(Scope::Local)); // ALWAYS bind this!
  assign_value_sym(crate::pin!("groupNonBoxing"), nobox, Some(Scope::Local)); // ALWAYS bind this!
  assign_value_sym(
    crate::pin!("groupInitiator"),
    current_token,
    Some(Scope::Local),
  );
  assign_value_sym(
    crate::pin!("groupInitiatorLocator"),
    gullet::get_locator(),
    Some(Scope::Local),
  );
  if !nobox {
    // For begingroup/endgroup
    stomach_mut!().boxing.push(current_token)
  }
}
/// Execute tokens stored on beforeAfterGroup (if any); done before popping a stack frame.
/// Perl: sub executeBeforeAfterGroup (Stomach.pm lines 286-295)
pub fn execute_before_after_group() -> Result<()> {
  if let Some(Stored::VecDequeStored(beforeafter)) = remove_value("beforeAfterGroup")
    && !beforeafter.is_empty()
  {
    let mut result = Vec::with_capacity(beforeafter.len());
    for beforeafter_frame in beforeafter.into_iter() {
      match beforeafter_frame {
        Stored::Tokens(frametoks) => result.push(frametoks.be_digested()?),
        Stored::Token(frametok) => result.push(frametok.be_digested()?),
        _ => {
          // Unexpected value type in beforeAfterGroup — skip silently
          // rather than panic (could occur with non-standard TeX constructs)
        },
      }
    }
    // Perl Stomach.pm:182-183 — every digested item must be Box-like
    // (TBox / List / Whatsit / Alignment); anything else is a binding
    // bug. Emit Error per offender; the Box-like items still flow
    // through to box_list so partial output is preserved.
    // Perl additionally calls `@result = (makeMisdefinedError(@result))`
    // collapsing everything to a single error sentinel — we keep
    // the partial-output behaviour (Rust-side divergence; surfacing
    // *the* offending item via Error! is what the harness needs to
    // report, while the rest of the box stream is still useful).
    //
    // Implementation note: walk the result list with an index loop
    // rather than `retain(|d| {…})`. The Error! macro can `return
    // Err(…)` on the max-errors / runaway-loop guards, and a closure
    // returning `bool` can't propagate that out — only an explicit
    // for-loop in the surrounding `Result<()>` body can.
    let mut filtered = Vec::with_capacity(result.len());
    for d in result {
      let is_box = matches!(
        d.data(),
        DigestedData::TBox(_)
          | DigestedData::List(_)
          | DigestedData::Whatsit(_)
          | DigestedData::Alignment(_)
      );
      if is_box {
        filtered.push(d);
      } else {
        let kind_label = match d.data() {
          DigestedData::Postponed(_) => "Postponed",
          DigestedData::KeyVals(_) => "KeyVals",
          DigestedData::RegisterValue(_) => "RegisterValue",
          DigestedData::Comment(_) => "Comment",
          _ => "non-Box",
        };
        Error!(
          "misdefined",
          "<beforeAfterGroup>",
          format!(
            "Expected a Box|List|Whatsit, but got '{}' — dropping",
            kind_label
          )
        );
      }
    }
    // Route the group's digested boxes through the GUARDED appender (not a
    // raw `box_list.extend`) so the stomach's cycle / count / byte-budget
    // runaway guards see them. This is the path a grouped drawing loop
    // (`\@whiledim{…\hbox{…}…}`) flushes through, so bypassing it let a
    // heavy-box runaway accumulate unguarded until only the Linux RSS cap
    // caught it. Witness math0102053.
    extend_box_list(filtered);
  }
  Ok(())
}

/// Removes the last/current stack frame, ending a TeX group
pub fn pop_stack_frame(nobox: bool) -> Result<()> {
  if *TRACE_FRAMES {
    let current_token = get_current_token().unwrap_or_else(|| T_CS!("\\relax"));
    eprintln!(
      "[frames] pop  nobox={nobox} depth {} -> {} at {current_token} (bound_mode={})",
      get_frame_depth(),
      get_frame_depth().saturating_sub(1),
      lookup_string_from_sym(crate::pin!("BOUND_MODE"))
    );
  }
  let after = remove_value("afterGroup");
  execute_before_after_group()?;
  pop_frame()?;
  if !nobox {
    {
      stomach_mut!().boxing.pop(); // For begingroup/endgroup
    }
  }
  if let Some(Stored::VecDequeStored(after_entries)) = after {
    for entry in after_entries.into_iter().rev() {
      match entry {
        Stored::Tokens(t) => gullet::unread(t),
        Stored::Token(t) => gullet::unread_one(t),
        other => panic!(r"\aftergroup should be used with tokens, got instead: {other:?}"),
      };
    }
  }
  Ok(())
}

/// explain the current frame
pub fn current_frame_message() -> String {
  let target = if is_value_bound("MODE", Some(0)) {
    // SET mode in CURRENT frame ?
    Cow::Owned(s!(
      "mode-switch to {}",
      lookup_string_from_sym(crate::pin!("MODE"))
    ))
  } else if lookup_bool_sym(crate::pin!("groupNonBoxing")) {
    // Current frame is a non-boxing group?
    Cow::Borrowed("non-boxing group")
  } else {
    Cow::Borrowed("boxing group")
  };

  let initiator = if let Some(t) = lookup_token_sym(crate::pin!("groupInitiator")) {
    t.stringify()
  } else {
    String::new()
  };
  // Render the initiator's source locator as a readable "file; line N"
  // (the raw Stored Debug is redacted to `Stored::Locator[[...]]`, which is
  // useless for diagnosing where an unbalanced group opened).
  let locator = match lookup_value("groupInitiatorLocator") {
    Some(Stored::Locator(loc)) => s!("at {}", loc),
    Some(other) => other.to_string(),
    None => String::new(),
  };
  s!(
    "current frame is {} due to {} {}",
    target,
    initiator,
    locator
  )
}

//======================================================================
// Grouping pushes a new stack frame for binding definitions, etc.
//======================================================================

/// Begin a new level of binding by pushing a new stack frame,
/// and a new level of boxing the digested output.
pub fn bgroup() {
  push_stack_frame(false);
  // Perl's bgroup does NOT touch $ALIGN_STATE — it's tracked only at the scan level
  // (in read_token/read_x_token). The scan-level tracking in gullet.rs is sufficient.
}
/// End a level of binding by popping the last stack frame,
/// undoing whatever bindings appeared there, and also
/// decrementing the level of boxing.
pub fn egroup() -> Result<()> {
  if is_value_bound("BOUND_MODE", Some(0)) {
    // Diagnostic for cluster investigation (project_explsyntax_midload.md).
    if *TRACE_BOUND_MODE {
      let mode = lookup_string_from_sym(crate::pin!("MODE"));
      let bound = lookup_string_from_sym(crate::pin!("BOUND_MODE"));
      let cur_tok = get_current_token()
        .map(|t| t.to_string())
        .unwrap_or_default();
      eprintln!(
        "[trace] egroup ERROR: cur_tok={cur_tok} BOUND_MODE={bound} MODE={mode}\n{}",
        std::backtrace::Backtrace::force_capture()
      );
    }
    // Last stack frame was a mode switch!?!?!
    // Don't pop if there's an error; maybe we'll recover?
    // Perl Stomach.pm:347-349 passes currentFrameMessage as a SEPARATE
    // Error detail (its own line), not merged into the primary message.
    Error!(
      "unexpected",
      get_current_token().unwrap_or_else(|| T_CS!("\\?")),
      s!(
        "Attempt to close a group that switched to mode {}",
        lookup_string_from_sym(crate::pin!("MODE"))
      ),
      current_frame_message()
    );
  } else if lookup_bool_sym(crate::pin!("groupNonBoxing")) {
    // or group was opened with \begingroup
    Error!(
      "unexpected",
      get_current_token().unwrap_or_else(|| T_CS!("\\?")),
      "Attempt to close boxing group",
      current_frame_message()
    );
  } else {
    // Don't pop if there's an error; maybe we'll recover?
    pop_stack_frame(false)?;
  }
  // Perl's egroup does NOT touch $ALIGN_STATE — tracked at scan level only.
  Ok(())
}
/// Begin a new level of binding by pushing a new stack frame.
pub fn begingroup() {
  if *TRACE_BOUND_MODE {
    let depth = get_frame_depth();
    let loc = gullet::get_locator();
    let tok = get_current_token().unwrap_or_else(|| T_CS!("\\?"));
    eprintln!("[trace] begingroup pre-depth={depth} tok={tok} at {}", loc);
  }
  push_stack_frame(true);
}
/// End a level of binding by popping the last stack frame,
/// undoing whatever bindings appeared there.
pub fn endgroup() -> Result<()> {
  if *TRACE_BOUND_MODE {
    let depth = get_frame_depth();
    let bound = is_value_bound("BOUND_MODE", Some(0));
    let loc = gullet::get_locator();
    let tok = get_current_token().unwrap_or_else(|| T_CS!("\\?"));
    if depth == 0 {
      eprintln!(
        "[trace] endgroup at locked frame: tok={} at {}\n{}",
        tok,
        loc,
        std::backtrace::Backtrace::force_capture()
      );
    } else {
      eprintln!(
        "[trace] endgroup pre-depth={depth} bound_top={bound} tok={} at {}",
        tok, loc
      );
    }
  }
  // BAND-AID (commit 3088dbd17 — under root-cause investigation, see
  // `project_explsyntax_midload.md`): during raw .sty/.tex load
  // (INTERPRETING_DEFINITIONS=true), suppress strict BOUND_MODE check.
  // Empirically Perl emits zero errors on the same inputs while strict
  // checks fire 19 times in our Rust during expl3-code.tex raw load.
  // Latent bugs found 2026-04-25 when removing this guard:
  //   - `#` (catcode PARAM) escapes to stomach
  //   - `\q_stop` recursion
  //   - residual `\group_end:` mode-switch error (not caught by strict end_mode_opt either —
  //     separate divergence point)
  //   - `\xparse-2018-04-12.sty-h@@k` undefined
  // Each of those needs its own root-cause investigation.
  let interpreting = lookup_bool_sym(crate::pin!("INTERPRETING_DEFINITIONS"));
  if interpreting {
    // Diagnostic: capture band-aid suppression occurrences for analysis.
    if *TRACE_BOUND_MODE && is_value_bound("BOUND_MODE", Some(0)) {
      let mode = lookup_string_from_sym(crate::pin!("MODE"));
      let bound = lookup_string_from_sym(crate::pin!("BOUND_MODE"));
      let frame_keys = dump_top_frame_keys();
      eprintln!(
        "[trace] endgroup SUPPRESSED-ERR: BOUND_MODE={bound} MODE={mode} frame0_keys={frame_keys:?}",
      );
    }
    pop_stack_frame(true)?;
  } else if is_value_bound("BOUND_MODE", Some(0)) {
    // Diagnostic: dump BOUND_MODE binding context for cluster investigation.
    if *TRACE_BOUND_MODE {
      let mode = lookup_string_from_sym(crate::pin!("MODE"));
      let bound = lookup_string_from_sym(crate::pin!("BOUND_MODE"));
      eprintln!(
        "[trace] endgroup ERROR: BOUND_MODE={bound} MODE={mode}\n{}",
        std::backtrace::Backtrace::force_capture()
      );
    }
    // Last stack frame was a mode switch!?!?!
    // Don't pop if there's an error; maybe we'll recover?
    // Perl Stomach.pm:367-369: currentFrameMessage is a SEPARATE detail.
    Error!(
      "unexpected",
      get_current_token()
        .map(|t| t.to_string())
        .unwrap_or_else(|| String::from("\\?")),
      s!(
        "Attempt to close a group that switched to mode {}",
        lookup_string_from_sym(crate::pin!("MODE"))
      ),
      current_frame_message()
    );
  } else if !lookup_bool_sym(crate::pin!("groupNonBoxing")) {
    // or group was opened with \bgroup
    Error!(
      "unexpected",
      get_current_token()
        .map(|t| t.to_string())
        .unwrap_or_else(|| String::from("\\?")),
      "Attempt to close non-boxing group",
      current_frame_message()
    );
  } else {
    pop_stack_frame(true)?;
  }
  Ok(())
}

//======================================================================
// Mode (minimal so far; math vs text)
// Could (should?) be taken up by Stomach by building horizontal, vertical or math lists ?

/// Sets the mode without doing any grouping (NOR does it stack the modes!!)
///
/// Useful for environments, where the group has already been established.
/// (presumably, in the long run, modes & groups should be much less coupled)
pub fn set_mode(mode: &str) -> Result<()> {
  let prevmode = lookup_string_from_sym(crate::pin!("MODE"));
  let ismath = mode.ends_with("math");
  // Perl: beginMode maps to internal mode names, but set_mode stores as-is
  // We also set BOUND_MODE so end_mode can find it
  let bound_mode = bindable_mode(mode).unwrap_or(mode);
  // Diagnostic
  if *TRACE_BOUND_MODE {
    eprintln!(
      "[trace] set_mode mode={mode} bound_mode={bound_mode}\n{}",
      std::backtrace::Backtrace::force_capture()
    );
  }
  assign_value("BOUND_MODE", arena::pin(bound_mode), Some(Scope::Local));
  assign_value("MODE", arena::pin(bound_mode), Some(Scope::Local));
  assign_value("IN_MATH", ismath, Some(Scope::Local));
  if mode == prevmode {
  } else if ismath {
    let curfont = lookup_font().unwrap();
    // When entering math mode, we set the font to the default math font,
    // and save the text font for any embedded text.
    assign_value("savedfont", curfont.clone(), Some(Scope::Local));
    // see get_script_level()
    assign_value("script_base_level", stomach!().boxing.len(), None);
    let isdisplay = mode.starts_with("display");
    assign_value("IN_MATH_DISPLAY", isdisplay, Some(Scope::Local));
    let new_font = Rc::new(lookup_mathfont().unwrap().merge(Font {
      color: curfont.color,
      bg: curfont.bg,
      size: curfont.size,
      mathstyle: if isdisplay {
        Some("display".into())
      } else {
        Some("text".into())
      },
      ..Font::default()
    }));
    assign_value(
      "initial_math_font",
      Stored::Font(new_font.clone()),
      Some(Scope::Local),
    );
    assign_font(new_font, Some(Scope::Local));
    // Perl Stomach.pm:505 — `$STATE->assignValue(fontfamily => -1, 'local');`
    // Resets `\fam` (whose getter reads `fontfamily`) on math entry so that
    // text-mode `\rm` (which sets `fontfamily=0`) doesn't leak into math.
    assign_value("fontfamily", -1_i64, Some(Scope::Local));
  } else {
    let curfont = lookup_font().unwrap();
    // When entering text mode, we should set the font to the text font in use before the math
    // but inherit color and size
    let saved_opt = lookup_value("savedfont");
    if let Some(Stored::Font(saved_font)) = saved_opt {
      assign_font(
        Rc::new(saved_font.merge(Font {
          color: curfont.color,
          bg: curfont.bg,
          size: curfont.size,
          ..Font::default()
        })),
        Some(Scope::Local),
      );
    }
  }
  Ok(())
}

/// Map user-facing mode names to internal bound mode names.
/// Perl: our %bindable_mode = (text => 'restricted_horizontal', ...)
fn bindable_mode(umode: &str) -> Option<&'static str> {
  match umode {
    "text" | "restricted_horizontal" => Some("restricted_horizontal"),
    "vertical" | "internal_vertical" => Some("internal_vertical"),
    // Perl #2798: inline_internal_vertical binds to internal_vertical but does
    // NOT leaveHorizontal (inline blocks: \vbox/\vtop/\parbox/minipage/picture/
    // footnotes) — see begin_mode_opt.
    "inline_internal_vertical" => Some("internal_vertical"),
    "math" | "inline_math" => Some("math"),
    "display_math" => Some("display_math"),
    _ => None,
  }
}

/// Begin processing in `mode`; one of "text", "display-math" or "inline-math".
/// This also begins a new level of grouping and switches to a font
/// appropriate for the mode.
/// If `noframe` is true, skip pushing a stack frame (e.g. for \begin{document}).
/// Perl: sub beginMode (Stomach.pm lines 474-517)
pub fn begin_mode(mode: &str) -> Result<()> { begin_mode_opt(mode, false) }
/// Like `begin_mode`, but with an explicit `noframe` option.
/// When `noframe` is true, no stack frame is pushed (the caller already did bgroup).
pub fn begin_mode_opt(mode: &str, noframe: bool) -> Result<()> {
  if let Some(bound_mode) = bindable_mode(mode) {
    // Perl #2798: beginning a vertical or display-math mode ends the current
    // paragraph first (leaveHorizontal), UNLESS the *user* mode is an inline
    // form (inline_internal_vertical / inline_math) — inline blocks must not
    // break the surrounding paragraph. `leave_horizontal` is itself a no-op
    // unless mid-paragraph (MODE==horizontal), so this only fires when a
    // vertical/display construct is encountered inside a paragraph.
    let is_display = bound_mode.starts_with("display");
    let is_vertical = is_display || bound_mode.contains("vertical");
    let is_inline = mode.contains("inline");
    if is_vertical && !is_inline {
      leave_horizontal()?;
    }
    if !noframe {
      push_stack_frame(false); // Effectively bgroup
    }
    // Diagnostic: tracking who binds BOUND_MODE during raw .sty load
    // (gated by LXML_TRACE_BOUND_MODE env var to avoid noise in normal runs).
    // See project_explsyntax_midload.md memory for the active investigation.
    if *TRACE_BOUND_MODE {
      eprintln!(
        "[trace] begin_mode_opt mode={mode} noframe={noframe} bound_mode={bound_mode}\n{}",
        std::backtrace::Backtrace::force_capture()
      );
    }
    // Perl: $STATE->assignValue(BOUND_MODE => $mode, 'local');
    assign_value("BOUND_MODE", arena::pin(bound_mode), Some(Scope::Local));
    // tex.web §211's inner sign, kept as a frame-bound flag: a FRAMED mode
    // switch is a box or math interior (`\hbox`/`\vbox`/`\parbox`/minipage,
    // `$…$`), where `\ifinner` is true; display math is positive `mmode`
    // (outer); the document body's frameless `internal_vertical` and plain
    // `{…}` groups bind nothing, so the main galley stays outer. The mode
    // STRINGS serve both galley and box (`internal_vertical`), so `\ifinner`
    // on them said inner at the body top after `\par` — paracol.sty:1996
    // `\ifinner\@parmoderr` (tidyres); Perl identical. The binding is
    // undone with the frame at `end_mode`. Guard:
    // `perfect_kernel_batch54::ifinner_is_the_box_frame_sign`.
    if !noframe {
      assign_value_sym(
        crate::pin!("INNER_BOX"),
        bound_mode != "display_math",
        Some(Scope::Local),
      );
    }
    set_mode(bound_mode)?;
    // Perl Stomach.pm lines 504-507: inject \everymath or \everydisplay tokens
    // Display math gets \everydisplay, inline math gets \everymath (not both).
    if bound_mode.contains("math") {
      let is_display = bound_mode == "display_math";
      let reg_name = if is_display {
        "\\everydisplay"
      } else {
        "\\everymath"
      };
      if let Some(RegisterValue::Tokens(toks)) = lookup_register(reg_name, Vec::new())? {
        let toks = toks.unlist();
        if !toks.is_empty() {
          gullet::unread(Tokens::new(toks));
        }
      }
    }
    Ok(())
  } else {
    Warn!("unexpected", mode, s!("Cannot enter {mode} mode"));
    Ok(())
  }
}
/// End processing in `mode`; an error is signalled if `stomach` is not
/// currently in `mode`.  This also ends a level of grouping.
/// Perl: sub endMode (Stomach.pm lines 522-541)
pub fn end_mode(mode: &str) -> Result<()> { end_mode_opt(mode, false) }
/// Like `end_mode`, but with an explicit `noframe` option.
/// When `noframe` is true, executeBeforeAfterGroup is run but the stack frame is not popped.
pub fn end_mode_opt(mode: &str, noframe: bool) -> Result<()> {
  if let Some(bound_mode) = bindable_mode(mode) {
    // Perl Stomach.pm L527-528:
    //   if ((!$STATE->isValueBound('BOUND_MODE', 0))     # Last stack frame was NOT a mode switch
    //     || ($STATE->lookupValue('BOUND_MODE') ne $mode))  # OR switch to a different mode
    // Strict Perl-faithful: error if BOUND_MODE is not bound on the top
    // frame, OR if its value doesn't match the mode being closed. (Earlier
    // versions of this file used a lax value-only check as a workaround
    // for the 1112.6246 halign frame-balance issue, since fixed in
    // d162803d2.)
    let current_bound = lookup_string_from_sym(crate::pin!("BOUND_MODE"));
    let bound_on_top = is_value_bound("BOUND_MODE", Some(0));
    let make_mode_error = || {
      // Perl Stomach.pm:550: Error('unexpected', $CURRENT_TOKEN, $self,
      //   "Attempt to end mode $mode", currentFrameMessage($self)) — where
      // $mode is the BOUND (bindable) mode, and currentFrameMessage is a
      // SEPARATE detail (added at the call sites below). The earlier Rust
      // wording ("...mode `X` in `Y`") was not Perl-faithful.
      let message = s!("Attempt to end mode {}", bound_mode);
      let category = match get_current_token() {
        Some(ref token) => token.to_string(),
        None => String::from("mode"),
      };
      (category, message)
    };
    if !bound_on_top || current_bound != bound_mode {
      // Last stack frame was NOT a mode switch, or was a switch to a different mode.
      // Perl: Don't pop if there's an error; maybe we'll recover?
      if *TRACE_BOUND_MODE {
        let cur_tok = get_current_token()
          .map(|t| t.to_string())
          .unwrap_or_default();
        eprintln!(
          "[trace] end_mode ERROR: mode={mode} cur_tok={cur_tok} bound_on_top={bound_on_top} current_bound={current_bound} depth={}\n  {}\n{}",
          get_frame_depth(),
          current_frame_message(),
          std::backtrace::Backtrace::force_capture()
        );
      }
      let (category, message) = make_mode_error();
      Error!("unexpected", category, &message, current_frame_message());
    } else {
      // Perl: leaveHorizontal_internal($self) if $mode =~ /vertical$/;
      if bound_mode.ends_with("vertical") {
        leave_horizontal_internal();
      }
      if noframe {
        // No pop, but at least do beforeAfterGroup
        execute_before_after_group()?;
      } else if current_frame_locked() {
        // After `leave_horizontal_internal` the only frame left is the LOCKED
        // bottom frame — there is no mode-switch frame to pop, so
        // `pop_stack_frame` → `pop_frame` would FATAL ("pop last locked stack
        // frame"). This happens on a STRAY mode-ender with no matching begin:
        // e.g. `$Proof.$ … \quad \endproof` (no `\begin{proof}`) leaves
        // BOUND_MODE bound on the bottom frame, so the value-guard above passes
        // but the pop is illegal. Emit a recoverable Error and DON'T pop (Perl's
        // "maybe we'll recover" intent — Perl completes such papers; Rust used
        // to crash). Note the check is HERE (after `leave_horizontal_internal`,
        // which can repack a horizontal frame that legitimately becomes the
        // pop target — e.g. a normal document's `\end{document}`), not at the
        // value-guard above. Witness 1703.05010 (svjour3 + bare `\endproof`).
        let (category, message) = make_mode_error();
        Error!("unexpected", category, &message);
      } else {
        pop_stack_frame(false)?;
      }
    }
  } else {
    Warn!("unexpected", mode, s!("Cannot end {mode} mode"));
  }
  Ok(())
}

thread_local! {
  // Re-entrancy guard so `\everypar`'s own digestion can't recursively re-fire it.
  static EVERYPAR_FIRING: Cell<bool> = const { Cell::new(false) };
  // How many isolated argument digestions (`digest(tokens)`) are open, with
  // `digest_next_body` resetting the count to 0 for the list it captures.
  // tex.web §1091 `new_graf` fires `\everypar` when a *list* — the main
  // vertical list or a `\vbox`'s internal one — starts a paragraph; a
  // constructor's digested `{}` argument is macro-parameter text that TeX
  // never typesets as such, so a paragraph "started" while reverting it is
  // not a `new_graf`. An armed `\everypar` (latex.ltx `\@afterheading`'s
  // `{\setbox\z@\lastbox}`, left by ltugboat.cls:1214 `\aftergroup
  // \@afterheading` in `\@maketitle`) used to fire inside
  // `\@@numbered@section`'s *type* argument and revert into it as
  // `{}section` — counter `\c@{}section`, tag `ltx:{}section` (lazylist,
  // parnotes; guard
  // `perfect_kernel_batch54::everypar_does_not_fire_inside_a_constructor_argument`).
  static ARG_DIGEST_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// RAII bookkeeping for `ARG_DIGEST_DEPTH`: restores the saved depth on
/// drop, so early `?` returns inside a digestion loop cannot leave the count
/// skewed.
struct ArgDigestScope(u32);

impl ArgDigestScope {
  /// Enter an isolated argument digestion (depth + 1).
  fn enter_argument() -> Self {
    let saved = ARG_DIGEST_DEPTH.with(|d| d.replace(d.get() + 1));
    ArgDigestScope(saved)
  }

  /// Enter a body capture: the captured material is a list of its own
  /// (a box, an environment body), so `new_graf` applies again inside it.
  fn enter_body() -> Self {
    let saved = ARG_DIGEST_DEPTH.with(|d| d.replace(0));
    ArgDigestScope(saved)
  }
}

impl Drop for ArgDigestScope {
  fn drop(&mut self) { ARG_DIGEST_DEPTH.with(|d| d.set(self.0)); }
}

/// Fire `\the\everypar` when a paragraph enters horizontal mode, the way tex.web's
/// `new_graf` (background/tex.web L21117) does `begin_token_list(every_par)`.
///
/// Guarded two ways, because LaTeXML's `\everypar` is not TeX's:
/// * `\everypar` is empty for every ordinary paragraph (post-`\begin{document}` the
///   register is cleared — see `latex_constructs.rs`), so this is a cheap early
///   return except where a package populates it (algorithm2e line numbering sets
///   `\everypar`→`\algocf@everypar`→`\nl` inside a listing).
/// * We fire ONLY in the document body. In the preamble / during kernel load
///   `\everypar` holds the unmodelled LaTeX3 para-hook list
///   `\g__para_standard_everypar_tl` (from raw-loading `ltpara`); firing it trips
///   `\@nodocument` ("Missing \begin{document}"). `\begin{document}` lets
///   `\@nodocument`→`\relax`, so "document started" is exactly that test.
///
/// The digested boxes are pushed to the current box list BEFORE the triggering box
/// (the caller `extend_box_list`s that after), so `\nl`'s tag lands at the head of
/// the listingline. Errors are swallowed (this rides the infallible mode-switch
/// path); a genuine fatal is re-detected at the next digest-loop checkpoint.
fn fire_everypar() {
  if EVERYPAR_FIRING.with(|f| f.get()) || ARG_DIGEST_DEPTH.with(|d| d.get()) > 0 {
    return;
  }
  let toks = match lookup_register("\\everypar", Vec::new()) {
    Ok(Some(RegisterValue::Tokens(t))) if !t.is_empty() => t,
    _ => return, // empty \everypar — the normal body paragraph
  };
  // Skip the preamble/kernel-load para-hook \everypar (see doc comment).
  if !x_equals(&T_CS!("\\@nodocument"), &T_CS!("\\relax")) {
    return;
  }
  EVERYPAR_FIRING.with(|f| f.set(true));
  if let Ok(digested) = digest(toks) {
    // A List box is unwound (flattened) on absorption, so `\nl`'s tag-whatsit runs
    // inline in the current listingline rather than under a wrapper.
    push_box_list(digested);
  }
  EVERYPAR_FIRING.with(|f| f.set(false));
}

/// Switch to horizontal mode without stacking the mode.
/// Can only switch from vertical|internal_vertical to horizontal.
/// Perl: sub enterHorizontal.
/// tex.web `new_graf` (L21117) fires `\everypar` here (`begin_token_list`).
pub fn enter_horizontal() {
  let mode = lookup_string_from_sym(crate::pin!("MODE"));
  if mode.ends_with("vertical") {
    assign_value_inplace_sym(crate::pin!("MODE"), crate::pin!("horizontal"));
    fire_everypar();
  } else if !mode.ends_with("horizontal") && !mode.ends_with("math") {
    // Perl L420-422: warn on unexpected mode
    Warn!(
      "unexpected",
      "enterHorizontal",
      s!("Unexpected mode '{}' for enterHorizontal", mode)
    );
  }
  // else: already horizontal or math — fine
}

/// Resume vertical mode by executing \par, in TeX-like fashion.
/// Perl: sub leaveHorizontal
pub fn leave_horizontal() -> Result<()> {
  let mode = lookup_string_from_sym(crate::pin!("MODE"));
  let bound = lookup_string_from_sym(crate::pin!("BOUND_MODE"));
  if mode == "horizontal" && bound.ends_with("vertical") {
    // This needs to be an invisible, and slightly gentler, \par
    assign_value("INTERNAL_PAR", true, Some(Scope::Local));
    let par_result = invoke_token(&T_CS!("\\par"))?;
    push_box_list_vec(par_result);
    assign_value("INTERNAL_PAR", false, Some(Scope::Local));
  }
  Ok(())
}

/// Resume vertical mode internally: reset mode without firing \par.
/// Used within argument digestion, e.g. endMode for vertical modes.
/// Perl: sub leaveHorizontal_internal
pub fn leave_horizontal_internal() {
  let mode = lookup_string_from_sym(crate::pin!("MODE"));
  let bound = lookup_string_from_sym(crate::pin!("BOUND_MODE"));
  if mode == "horizontal" && bound.ends_with("vertical") {
    repack_horizontal();
    assign_value_inplace_sym(crate::pin!("MODE"), arena::pin(&bound));
  }
}

/// Repack recently digested horizontal items into single horizontal List.
/// Note that TeX would have done paragraph line-breaking, resulting in essentially
/// a vertical list.
/// Perl: sub repackHorizontal (Stomach.pm lines 440-454)
pub fn repack_horizontal() {
  let mut stomach = stomach_mut!();
  let mut para: Vec<Digested> = Vec::new();
  let mut keep = false;

  loop {
    let should_pop = if let Some(item) = stomach.box_list.last() {
      // Perf: compare as &str via with() instead of allocating a String each iter.
      // Default mode is "horizontal" (matches previous unwrap_or).
      let mode_prop = item.get_property("mode");
      let (is_horiz_family, is_plain_horizontal) = match mode_prop.as_deref() {
        Some(Stored::String(sym)) => arena::with(*sym, |s| {
          let plain = s == "horizontal";
          let fam = plain || s == "restricted_horizontal" || s == "math";
          (fam, plain)
        }),
        None => (true, true), // default "horizontal"
        Some(other) => {
          // Rare path — fall back to Display formatting.
          let s = other.to_string();
          let plain = s == "horizontal";
          let fam = plain || s == "restricted_horizontal" || s == "math";
          (fam, plain)
        },
      };
      if is_horiz_family {
        if !is_plain_horizontal || !item.get_property_bool("isSpace") {
          keep = true;
        }
        true
      } else {
        false
      }
    } else {
      false
    };

    if should_pop {
      para.push(stomach.box_list.pop().unwrap());
    } else {
      break;
    }
  }

  // Items were popped in reverse order, so reverse them back
  para.reverse();

  if keep {
    let mut list = List::new(para);
    list.mode = Some(TexMode::Text); // "horizontal" in Perl
    // Perl: List(@para, mode => 'horizontal') — set mode property string
    // This is needed for compute_boxes_size vertical layout to detect paragraph Lists
    list.set_property("mode", Stored::String(pin!("horizontal")));
    // Perl #2798 (S4): a finished paragraph List records BOTH the fill width
    // (\hsize) and the \baselineskip, so the sizing pass (compute_boxes_size)
    // can line-break and stack with the right inter-line spacing.
    //   $list->setProperty(width    => LookupDimension('\hsize'));
    //   $list->setProperty(baseline => LookupDimension('\baselineskip', 1));
    if let Some(hsize) = lookup_dimension("\\hsize") {
      list.set_property("width", hsize);
    }
    if let Some(baseline) = lookup_dimension("\\baselineskip") {
      list.set_property("baseline", baseline);
    }
    stomach.box_list.push(Digested::from(list));
  }
}

pub fn new_local_box_list() {
  let mut buffer = Vec::new();
  let mut stomach = stomach_mut!();
  // Guard the OTHER aberrant accumulation path: the boxing stack. When a loop
  // builds *inside* boxes (`\setbox`/`\hbox`), each nesting suspends the partial
  // outer list here and opens a fresh `box_list`; an unbounded `\hbox{\hbox{…}}`
  // nest grows this stack without ever touching the byte/cycle guards on the
  // (small, innermost) `box_list`. A depth cap is O(1) and safe — no real
  // document nests boxes anywhere near this deep (typical depth is tens; the
  // math0102053 line-drawing loop sits at 13). Platform-independent, fires long
  // before any RSS/OOM ceiling.
  if stomach.localized_box_list.len() > STOMACH_BOXING_DEPTH_CAP
    && stomach.pending_cycle_fatal.is_none()
  {
    stomach.pending_cycle_fatal = Some((
      ErrorCategory::MemoryBudget,
      s!(
        "Boxing-stack runaway: box nesting depth exceeded {} \
         (unbounded \\hbox/\\setbox nesting)",
        STOMACH_BOXING_DEPTH_CAP
      ),
    ));
  }
  std::mem::swap(&mut stomach.box_list, &mut buffer);
  stomach.localized_box_list.push(buffer);
}

/// Hard cap on box-nesting depth (the `localized_box_list` boxing stack). No
/// real document nests `\hbox`/`\setbox` more than tens deep; a runaway nest
/// grows this without bound while the per-level `box_list` stays small, evading
/// the byte/cycle guards. Platform-independent.
const STOMACH_BOXING_DEPTH_CAP: usize = 100_000;
pub fn expire_local_box_list() -> Vec<Digested> {
  let mut stomach = stomach_mut!();
  let mut buffer = stomach.localized_box_list.pop().unwrap_or_default();
  std::mem::swap(&mut stomach.box_list, &mut buffer);
  buffer
}

/// Recover the boxes a failed `digest_next_body` left stranded, in document
/// order, and reset the accumulation stack.
///
/// `digest_next_body` accumulates into `box_list` (with outer levels suspended
/// on `localized_box_list`) and only hands them back via `expire_local_box_list`
/// on the SUCCESS path — so a mid-body Fatal drops every box digested during
/// that call. `digest_internal` is written to keep partial output after a
/// recoverable Fatal ("Perl finishDigestion L219-220: loop consuming input even
/// after errors"), but that intent was defeated whenever the failure landed in
/// the FIRST body: the caller's `boxes` was still empty, so the run produced a
/// 39-byte empty document instead of the text preceding the bad construct.
/// Witness arXiv:2508.07407 (ar5iv #556) — its whole document was lost, though
/// only one `\tikz` picture is pathological.
///
/// `drop_innermost` is for the runaway guards (`Stomach:Recursion`), where the
/// innermost level IS the pathology — a 50k-box repeating window. Salvaging it
/// would graft the garbage into the document, so drop that level and keep the
/// suspended outer ones, which is precisely "drop the offending construct, keep
/// the document". For every other recoverable Fatal the current level is honest
/// content and is kept.
pub fn salvage_pending_box_lists(drop_innermost: bool) -> Vec<Digested> {
  let mut stomach = stomach_mut!();
  let mut acc = std::mem::take(&mut stomach.box_list);
  if drop_innermost {
    acc.clear();
  }
  // Unwind the suspended levels innermost-parent first, each time prefixing the
  // parent's own content so the result stays in document order.
  while let Some(mut parent) = stomach.localized_box_list.pop() {
    parent.append(&mut acc);
    acc = parent;
  }
  // Refuse a salvage that is itself pathological. `drop_innermost` removes the
  // runaway level for the STOMACH box-cycle guard, where that level is the
  // pathology — but the GULLET cycle guard (`Timeout:Recursion`) fires on the
  // token stream, and there the bloated boxes can sit in the suspended outer
  // levels instead, so dropping the innermost does not bound anything.
  //
  // `STOMACH_CYCLE_ACTIVATE` is exactly the engine's own "no honest document
  // accumulates this many undrained boxes" line, so reuse it rather than invent
  // a second threshold: a salvage at or past it is runaway output, and handing
  // it to the builder is worse than handing over nothing. Measured on
  // arXiv:2605.25400, where an unbounded salvage turned a 9.7 s fatal into a
  // 120 s wall-clock timeout that wrote a ZERO-byte file — strictly worse than
  // the 39-byte stub it replaced.
  if acc.len() >= STOMACH_CYCLE_ACTIVATE {
    acc.clear();
  }
  acc
}

/// Stomach-level cycle guard: only once `box_list` has grown far past any
/// flushed-document size (a normal `box_list` is drained as paragraphs/boxes
/// complete and stays small) do we record the digest-push stream and look for
/// a short repeating window — a box-accumulation infinite loop. Cuts it off
/// with a clean Fatal long before the RSS soft cap. Caller must already hold
/// the stomach borrow and have appended past the activation size.
#[inline]
fn cycle_guard_record(st: &mut Stomach, d: &Digested) {
  // Once a fatal is pending, further detection work is pointless — the raise
  // happens at the NEXT `check_timeout` tick, which (since PR #249 review
  // P2-6) every digestion loop runs per iteration (`digest_next_body`,
  // `digest`, `raw_tex`), so the window between detection and raise is at
  // most one `invoke_token`. (Before that fix, a runaway confined to
  // `digest()` set the flag and the guards then self-disabled while the list
  // grew unbounded — the flag was never raised on that path.)
  if st.pending_cycle_fatal.is_none() {
    // Hard size backstop — platform-INDEPENDENT (the RSS soft cap in
    // `check_timeout` reads `/proc/self/statm` and is therefore Linux-only;
    // on macOS/Windows it is inactive). This bounds `box_list` everywhere and
    // also catches APERIODIC runaways the windowed cycle detector cannot
    // (boxes that vary per iteration, e.g. a `\@whilenum` loop with a
    // counter, or period > MAX_WINDOW). 40× the validated cycle-activation
    // size, far past any flushed-document list. Analogous to the gullet's
    // platform-independent `token_limit`.
    if let Some(cap) = box_count_cap()
      && st.box_list.len() > cap
    {
      st.pending_cycle_fatal = Some((
        ErrorCategory::MemoryBudget,
        s!(
          "Box-list runaway: {} accumulated boxes exceeded the hard cap of {} \
           (unbounded digestion with no detectable cycle); raise --max-memory, \
           or --max-memory=0 to lift the ceiling",
          st.box_list.len(),
          cap
        ),
      ));
      return;
    }
    // Portable, BYTE-based memory guard. The count caps above are a proxy for
    // memory, but per-box weight varies several-fold (a bare text box vs a
    // deeply nested `\hbox{\raise…\hbox{…}}`), so a count calibrated for light
    // boxes lets a HEAVY-box runaway sail past it — only the Linux-only RSS cap
    // in `check_timeout` (4.5 GB) then catches it, late and non-portably.
    // Here we estimate the box list's actual heap footprint (by sampling, so
    // it stays O(1) amortised) and `Fatal` once it crosses a budget set BELOW
    // the RSS cap. This fires EARLIER than the external RSS guard AND works on
    // macOS/Windows where `/proc/self/statm` is unavailable. Driver:
    // math0102053 (plain-TeX `\@whiledim` line-drawing loop — Perl OOMs too;
    // ~1.87 M heavy line-segment boxes reached 4.5 GB RSS before the 2 M count
    // cap could fire).
    let len = st.box_list.len();
    if let Some(budget) = box_bytes_budget()
      && len >= BYTE_CHECK_ACTIVATE
      && len.is_multiple_of(BYTE_CHECK_EVERY)
    {
      let est = estimate_box_list_bytes(&st.box_list);
      if est > budget {
        st.pending_cycle_fatal = Some((
          ErrorCategory::MemoryBudget,
          s!(
            "Box-list memory runaway: ~{} MB estimated across {} boxes exceeded \
             the {} MB budget (unbounded accumulation); raise --max-memory, or \
             --max-memory=0 to lift the ceiling. NOTE: the estimate is a LOWER \
             BOUND (each box is walked at most {} nodes deep), so true RSS at \
             this point is typically several times larger",
            est / 1_000_000,
            len,
            budget / 1_000_000,
            crate::digested::EB_BUDGET
          ),
        ));
        return;
      }
    }
    let fp = d.cycle_fingerprint();
    if let Some(period) = st.cycle_guard.push(fp, 0) {
      st.pending_cycle_fatal = Some((
        ErrorCategory::Recursion,
        s!(
          "Infinite digestion loop: a window of {} box(es) repeated {}+ times \
           while the box list grew past {}",
          period,
          crate::cycle_guard::REPEAT,
          STOMACH_CYCLE_ACTIVATE
        ),
      ));
    }
  }
}

/// Hard, platform-independent ceiling on `box_list` length — `None` when the
/// memory limit is disabled. A normal list is flushed continuously and stays
/// tiny; reaching this is an unbounded accumulation. The backstop for
/// very-LIGHT-box runaways, which the byte budget below can under-weigh.
///
/// **Rides `--max-memory`**, like every other memory ceiling: the resolved soft
/// cap divided by [`BYTES_PER_LIGHT_BOX`], which reproduces the historical fixed
/// 2 M at the stock `--max-memory=6144` (soft cap 4608 MiB), scales linearly
/// with the flag, and is `None` at `--max-memory=0`.
///
/// It used to be a hardcoded `const`, which made `--max-memory=0` a documented
/// lie: the binary prints "memory limiting disabled entirely" and then Fatal'd
/// on a memory ceiling anyway, with no flag able to raise it. Witness: a
/// ~10 000-page notes document (Nasser Abbasi, rc4 report 2026-07-28) died on
/// the byte budget below after 8 h at ~58 GB RSS having explicitly passed
/// `--max-memory=0`. Guard: `box_ceilings_follow_the_memory_knob`.
fn box_count_cap() -> Option<usize> {
  resolve_rss_cap().map(|cap| (cap / BYTES_PER_LIGHT_BOX) as usize)
}

/// Calibration for [`box_count_cap`]: the per-box footprint of a *light* box,
/// chosen so the stock ceiling yields the validated 2 M-box cap.
const BYTES_PER_LIGHT_BOX: u64 = 2_416;

/// Portable byte-budget for the accumulated `box_list` — `None` when the memory
/// limit is disabled. `estimate_bytes` counts each box's OWNED heavy data (the
/// `properties` HashMap, the `Tbox` `tokens` source-TeX vector, args/children
/// vectors + nested children). Works on macOS/Windows, where the `/proc` RSS
/// check is inactive and this is the ONLY memory guard for a heavy-box runaway.
///
/// **Rides `--max-memory`** (see [`box_count_cap`]): two thirds of the resolved
/// soft cap, i.e. 3.22 GB at the stock `--max-memory=6144` — the historical
/// fixed 3.2 GB — so on Linux it still `Fatal`s well before the RSS fuse, and
/// `None` at `--max-memory=0`.
///
/// **The estimate is a LOWER BOUND, not an RSS prediction.**
/// [`Digested::estimate_bytes`] walks at most `EB_BUDGET` (256) nodes per box,
/// so a deep document tree is undercounted by however much hangs below that
/// horizon — and the shortfall is content-dependent, not a constant. Measured:
/// a flat 600 k-paragraph synthetic crosses the 3.2 GB budget at 5.8 GB true RSS
/// (est ≈ 58 % of RSS), while Nasser's deeply-nested notes crossed the *same*
/// budget at ~58 GB (est ≈ 6 %). A ~10× spread — so do not read the budget as a
/// megabyte ceiling on the process. (The "tracks true RSS within ~10 %" claim
/// this doc used to carry held only for its calibration paper, math0102053: a
/// plain-TeX `\@whiledim` line-drawing loop whose ~1.87 M boxes are shallow.)
fn box_bytes_budget() -> Option<usize> { resolve_rss_cap().map(|cap| (cap / 3 * 2) as usize) }
/// Don't bother byte-sampling until the list is already well past the cycle
/// activation size (a normal list never gets here).
const BYTE_CHECK_ACTIVATE: usize = 200_000;
/// Re-estimate the box-list footprint every this-many boxes (amortises the
/// sampling cost to O(1) per push).
const BYTE_CHECK_EVERY: usize = 50_000;
/// Boxes sampled per byte estimate. Box weights are bimodal (light text
/// segments vs heavy nested structures), so a *dense* sample is needed to keep
/// the extrapolation from aliasing against the heavy-box stride.
const BYTE_SAMPLE_N: usize = 8192;

/// Cost-bounded estimate of the heap bytes held by `list`, via even sampling +
/// extrapolation (each sampled box is itself depth-bounded — see
/// [`crate::digested::Digested::estimate_bytes`]). O(`BYTE_SAMPLE_N`) regardless
/// of list length. The sample is taken as contiguous *blocks* spread across the
/// list rather than a single large stride, which is far more robust to clustered
/// heavy boxes than evenly-strided point sampling.
fn estimate_box_list_bytes(list: &[Digested]) -> usize {
  let len = list.len();
  if len == 0 {
    return 0;
  }
  if len <= BYTE_SAMPLE_N {
    return list.iter().map(Digested::estimate_bytes).sum();
  }
  // 32 blocks of (BYTE_SAMPLE_N/32) contiguous boxes, evenly spaced — captures
  // local clustering of heavy boxes that point sampling misses.
  const BLOCKS: usize = 32;
  let block = (BYTE_SAMPLE_N / BLOCKS).max(1);
  let gap = len / BLOCKS;
  let mut sum = 0usize;
  let mut n = 0usize;
  for b in 0..BLOCKS {
    let start = b * gap;
    let end = (start + block).min(len);
    for d in &list[start..end] {
      sum += d.estimate_bytes();
      n += 1;
    }
  }
  // average-per-box × len; usize (64-bit) cannot overflow at realistic sizes.
  (sum / n.max(1)) * len
}

pub fn extend_box_list<I>(arg: I)
where I: IntoIterator<Item = Digested> {
  let mut st = stomach_mut!();
  // Fast path (the overwhelming common case): box list still small — just
  // extend, no per-box fingerprinting.
  if st.box_list.len() <= STOMACH_CYCLE_ACTIVATE {
    st.box_list.extend(arg);
    return;
  }
  // Runaway territory: record each appended box into the cycle guard.
  for d in arg {
    cycle_guard_record(&mut st, &d);
    st.box_list.push(d);
  }
}
pub fn push_box_list(arg: Digested) {
  let mut st = stomach_mut!();
  if st.box_list.len() > STOMACH_CYCLE_ACTIVATE {
    cycle_guard_record(&mut st, &arg);
  }
  st.box_list.push(arg);
}
fn push_box_list_vec(args: Vec<Digested>) { extend_box_list(args) }

/// Engage the stomach's box-list cycle guard only once the (normally
/// flushed-small) `box_list` has grown past this. A real document's list is
/// drained continuously; a runaway accumulates boxes without bound. Keeps the
/// guard inert for every ordinary conversion. (~50k boxes is already well past
/// any sane un-flushed list yet ~30× below the 4.5 GB OOM ceiling.)
const STOMACH_CYCLE_ACTIVATE: usize = 50_000;
pub fn pop_box_list() -> Option<Digested> { stomach_mut!().box_list.pop() }
pub fn with_box_list<R, FnR>(caller: FnR) -> R
where FnR: FnOnce(&[Digested]) -> R {
  let stomach = stomach!();
  let list = &stomach.box_list;
  caller(list)
}
pub fn with_box_list_mut<R, FnR>(caller: FnR) -> R
where FnR: FnOnce(&mut [Digested]) -> R {
  let mut stomach = stomach_mut!();
  let list = &mut stomach.box_list;
  caller(list)
}
/// Access to the current box_list as a `&mut Vec` — allows push/pop operations.
pub fn with_box_list_mut_vec<R, FnR>(caller: FnR) -> R
where FnR: FnOnce(&mut Vec<Digested>) -> R {
  let mut stomach = stomach_mut!();
  caller(&mut stomach.box_list)
}

// **********************************************************************
// Digestion
// **********************************************************************

/// Digest a list of tokens independent from any current Gullet.
/// Typically used to digest arguments to primitives or constructors.
/// Returns a List containing the digested material.
pub fn digest<T: Into<Tokens>>(tokens: T) -> Result<Digested> {
  let tokens: Tokens = tokens.into();
  if tokens.is_empty() {
    return Ok(Digested::default());
  }
  gullet::reading_from_mouth(Mouth::default(), || {
    gullet::unread(tokens);
    clear_prefixes(); // prefixes shouldn't apply here.
    let _arg_scope = ArgDigestScope::enter_argument();
    let mode = if lookup_bool_sym(crate::pin!("IN_MATH")) {
      TexMode::Math
    } else {
      TexMode::Text
    };
    let initdepth = stomach!().boxing.len();
    let depth = initdepth;
    new_local_box_list();
    while let Some(token) = match gullet::get_pending_comment() {
      Some(comment) => Some(comment),
      None => gullet::read_x_token(Some(true), false, None)?,
    } {
      // Raise any pending stomach-guard fatal + deadline/RSS checks. This
      // loop is a digestion path of its own — without a tick here, a runaway
      // confined to constructor-argument digestion set `pending_cycle_fatal`
      // at detection but nothing ever RAISED it (check_timeout's only call
      // site was digest_next_body), and the RSS soft cap / wall-clock
      // deadline were equally dead on this path. PR #249 review P2-6.
      check_timeout()?;
      // Done if we run out of tokens
      let invoked = invoke_token(&token)?;
      extend_box_list(invoked);

      if initdepth > stomach!().boxing.len() {
        // if we've closed the initial mode.
        break;
      }
      if initdepth < depth {
        // TODO
        fatal!(Internal, EoF, "We've fallen off the end, somehow !?!?!?");
        //     Fatal('internal', '<EOF>', self,
        //       "We've fallen off the end, somehow!?!?!",
        //       "Last token " . ToString($LaTeXML::CURRENT_TOKEN)
        //         . " (Boxing depth was $initdepth, now $depth: Boxing generated by "
        //         . join(', ', map { ToString($_) } @{ $self{boxing} }))
        //       if $initdepth < $depth;
      }
    }

    let mut digested_list = List::new(expire_local_box_list());
    digested_list.mode = Some(mode);
    digested_list.into()
  })
}

/// Return the digested `List` after reading and digesting a body from the its Gullet.
/// The body extends until the current level of boxing or environment is closed.
pub fn digest_next_body(terminal_opt: Option<Token>) -> Result<Vec<Digested>> {
  let start_location = { gullet::get_locator() };

  let init_depth = { stomach!().boxing.len() };
  // Did the loop end because the INPUT RAN OUT (as opposed to reaching the
  // terminal or closing the initial mode)? Perl `Stomach.pm` L130 keys the
  // trailer box on `unless $token`, and `$token` is undef exactly when the
  // `while (defined($token = ...))` condition failed — i.e. on EOF, whether or
  // not tokens were read before it. See the trailer push below.
  let mut ran_out = true;
  let mut found_terminal = false;
  let _body_scope = ArgDigestScope::enter_body();
  new_local_box_list();
  let alignment_opt = lookup_alignment();
  // TODO: bookkeep for "expected" warning
  //let mut aug = Vec::new();

  // try reading a executable token
  while let Some(token) = match gullet::get_pending_comment() {
    Some(comment) => Some(comment),
    None => gullet::read_x_token(Some(true), false, None)?,
  } {
    // Check conversion timeout
    check_timeout()?;
    // first, check for alignment case
    // Perl #2775: only fire at the original alignment nesting level,
    // not inside deeper boxing groups (e.g. \vbox inside a tabular cell).
    if alignment_opt.is_some()
      && !stomach!().box_list.is_empty()
      && (stomach!().boxing.len() <= init_depth)
      && (token == T_ALIGN!()
        || token == T_CS!("\\cr")
        || token == T_CS!("\\lx@hidden@cr")
        || token == T_CS!("\\lx@hidden@crcr"))
    {
      gullet::unread_one(token);
      return Ok(expire_local_box_list());
    }
    // normal case
    let invoked = invoke_token(&token)?;
    extend_box_list(invoked);

    if let Some(ref terminal) = terminal_opt
      && &token == terminal
    {
      found_terminal = true;
      ran_out = false;
      break;
    }
    if init_depth > stomach!().boxing.len() {
      ran_out = false;
      break;
    }
    // Fragment yield (streaming pass 1): between top-level constructs, at a
    // legal seam, hand back the boxes accumulated so far so the driver can
    // build + spill and re-enter. Everything digestion carries — gullet mouth
    // stack, State undo frames, mode, fonts — is thread-local and survives
    // between `digest_next_body` calls by construction, so resuming is the
    // same operation `digest_internal`'s outer loop already performs; the
    // alignment early-return above is the established precedent for
    // returning early with a partial list.
    //
    // Seam legality (probed 2026-07-29 on a real conversion, not assumed):
    // only the DRIVER call — `digest_internal` is the one caller that enters
    // with an empty boxing stack (`init_depth == 0`); constructor argument
    // digests also pass `None` but always sit inside an open box. The boxing
    // stack must be back at 0, and the mode VERTICAL-family: the document
    // body runs in `internal_vertical` (the `\begin{document}` environment's
    // mode — plain `vertical` occurs only before it), and at depth 0 that
    // cannot be a vbox/minipage interior, which always sits at deeper boxing.
    // A horizontal-mode cut would split the run `repack_horizontal` folds
    // into one paragraph; alignment, math, and open conditionals must all be
    // closed. A single construct larger than the whole budget simply digests
    // through — the existing hard ceilings still protect.
    //
    // Checked AFTER the terminal/depth exits so a real exit always wins, and
    // before the next `read_x_token` so nothing is consumed-then-unread.
    if let Some(budget) = FRAGMENT_YIELD_BUDGET.get()
      && init_depth == 0
      && terminal_opt.is_none()
      && alignment_opt.is_none()
      && {
        // The box budget yields on its own. The soft-RSS branch additionally
        // requires a MINIMUM accumulation: it is a level test (`rss > soft`)
        // with no hysteresis, so a document whose resident floor sits above
        // the watermark latches it on for the whole run and yields at every
        // seam with nothing accumulated — see `soft_yield_min_boxes` for the
        // measured degeneracy (24 M yields / 5.5 KB segments on the witness).
        let accumulated = stomach!().box_list.len();
        let rss_kb = LAST_SAMPLED_RSS_KB.get();
        accumulated >= budget
          || (FRAGMENT_YIELD_RSS_SOFT_KB
            .get()
            .is_some_and(|soft| rss_kb > soft)
            // The floor is waived once pressure is urgent, so pathological
            // per-box footprints keep the immediate response this branch
            // exists to give (`soft_yield_is_urgent`).
            && (accumulated >= soft_yield_min_boxes() || soft_yield_is_urgent(rss_kb)))
      }
      && stomach!().boxing.is_empty()
      && lookup_alignment().is_none()
      && !lookup_bool_sym(crate::pin!("IN_MATH"))
      && open_conditional_count() == 0
      && matches!(
        lookup_string_from_sym(crate::pin!("MODE")).as_str(),
        "vertical" | "internal_vertical"
      )
    {
      FRAGMENT_YIELDED.set(true);
      FRAGMENT_YIELD_COUNT.set(FRAGMENT_YIELD_COUNT.get() + 1);
      // No EOF trailer (`ran_out` stays true only through the loop's own
      // exhaustion path — we return before reaching it), and no
      // `gullet::flush()`: both are end-of-input actions, and input remains.
      return Ok(expire_local_box_list());
    }
  }

  if let Some(ref terminal) = terminal_opt
    && !found_terminal
  {
    let message = s!(
      "body should have ended with {:?}. current body started at {:?}",
      terminal,
      start_location
    );
    Warn!("expected", terminal, message);
  }
  // and add a Dummy `trailer' if none explicit — Perl `Stomach.pm` L130,
  // `push(@LaTeXML::LIST, Box()) unless $token;`.
  //
  // This was mistranslated as "if we never read ANY token", which is a strictly
  // narrower condition: it agrees with Perl only on a body that was empty from
  // the start. The case it missed is a body that read content and THEN hit EOF —
  // and that is the case the trailer exists for. `readDigested`
  // (`Base_ParameterTypes.pool.ltxml` L374, ported in `base_parameter_types.rs`)
  // does `push(@list, digestNextBody()); pop(@list);` to strip the closing `}`
  // box; with no trailer pushed, that `pop` silently ate a box of REAL CONTENT.
  // Concretely: one runaway `.bib` field swallowed the rest of the entry into
  // its own argument, and the `pop` then removed the boxes carrying every
  // following entry — an empty bibliography where Perl renders all of them.
  if ran_out {
    push_box_list(Digested::from(Tbox::default()));
  }
  Ok(expire_local_box_list())
}

/// a convenience function for including chunks of raw TeX (or LaTeX) code
/// It is useful for copying portions of the normal
/// implementation that can be handled simply using macros and primitives.
pub fn raw_tex(text: &str) -> Result<()> {
  // It could be as simple as this, except if catcodes get changed, it's too late!!!
  //  Digest(TokenizeInternal($text));
  let raw_tex_mouth = Mouth::new(
    text,
    Some(MouthOptions {
      fordefinitions: true,
      at_letter: true,
      ..MouthOptions::default()
    }),
  )?;
  gullet::reading_from_mouth(raw_tex_mouth, || -> Result<()> {
    while let Some(token) = gullet::read_x_token(Some(false), false, None)? {
      // Same per-iteration guard tick as digest()/digest_next_body — see the
      // comment in `digest` (PR #249 review P2-6): raw-loaded .sty/.cls
      // digestion must raise pending stomach fatals and honor the deadline
      // and RSS caps too.
      check_timeout()?;
      if token.get_catcode() != Catcode::SPACE {
        invoke_token(&token)?;
      }
    }
    Ok(())
  })?;
  Ok(())
}

/// Invoke a token
///
/// If it is a primitive or constructor, the definition will be invoked,
/// possibly arguments will be parsed from the Gullet.
/// Otherwise, the token is simply digested: turned into an appropriate box.
/// Returns a list of boxes/whatsits.
pub fn invoke_token(input_token: &Token) -> Result<Vec<Digested>> {
  // Perf: Token is Copy (SymStr + Catcode, ~5 bytes), so we pass by value
  // directly instead of wrapping in Cow<Token>.
  let mut maybe_token: Option<Token> = Some(*input_token);
  let mut result: Vec<Digested> = Vec::new();
  // INVOKE:
  while let Some(token) = maybe_token.take() {
    // RAII guard: auto-pops current_token on scope exit (even on early return/panic)
    let _token_guard = local_current_token_guard(token);
    {
      stomach_mut!().token_stack.push(token);
    }
    if { stomach!().token_stack.len() } > MAXSTACK {
      fatal!(
        Stomach,
        Recursion,
        s!(
          "Excessive recursion(?): Tokens on stack: {:?}",
          stomach!().token_stack
        )
      );
    }
    result = Vec::new();

    // Rust notes: It would be ideal if we could unify the cases for (Primtive, Constructor,
    // MathPrimitive), as well as (Expandable, Conditional) since the
    // API is identical. However, as the types are different, Rust
    // constrains us here, we need separate match arms for each
    // distinctly typed enum case.
    let digestable_def = lookup_digestable_definition(&token);
    match digestable_def {
      None | Some(Stored::None) => {
        result = invoke_token_undefined(&token)?;
      },
      Some(Stored::Token(meaning)) => {
        // Common case
        let cc = meaning.get_catcode();
        if cc == Catcode::CS {
          result = invoke_token_undefined(&token)?;
        } else if cc.is_absorbable() {
          if let Some(digested) = invoke_token_simple(meaning)? {
            result.push(digested);
          }
        } else {
          // Perl L187-189: deactivate T_ALIGN to prevent error flood in tables
          if token.get_catcode() == Catcode::ALIGN
            && let Some(relax_meaning) = lookup_meaning(&T_CS!("\\relax"))
          {
            assign_meaning(&token, relax_meaning, Some(Scope::Local));
          }
          let message = s!(
            "The token {:?} (catcode {:?}) should never reach Stomach!",
            token,
            cc
          );
          Error!("misdefined", token, &message);
          if let Some(digested) = invoke_token_simple(meaning)? {
            result.push(digested);
          }
        }
      },
      Some(Stored::Expandable(meaning)) => {
        // A math-active character will (typically) be a macro,
        // but it isn't expanded in the gullet, but later when digesting, in math mode
        // (? I think)
        let invoked_meaning = meaning.invoke(false)?;
        if !invoked_meaning.is_empty() {
          {
            gullet::unread(invoked_meaning);
          }
        }
        // replace the token by it's expansion!!!
        maybe_token = gullet::read_x_token(None, false, None)?;
        {
          stomach_mut!().token_stack.pop();
        }
        drop(_token_guard); // expire current token via RAII
        continue;
      },
      Some(Stored::Conditional(meaning)) => {
        // Conditionals are "expandable", use the regular invoke.
        let invoked_meaning = meaning.invoke(false)?;
        gullet::unread(invoked_meaning);
        maybe_token = gullet::read_x_token(None, false, None)?;
        {
          stomach_mut!().token_stack.pop();
        }
        drop(_token_guard); // expire current token via RAII
        continue;
      },
      Some(Stored::Constructor(meaning)) => {
        // Perl Stomach.pm L187-189: deactivate T_ALIGN to `\relax` LOCAL
        // on first non-table encounter, to prevent error flood. The
        // existing guard at the Stored::Token branch (above) only fires
        // when `&` has been Let'd to another token, but the `&`
        // CC_ALIGN char-token is bound to a Constructor (TeX_Tables.pool
        // L49: `DefConstructorI('&', undef, sub { Error('unexpected', '&',
        // $_[0], "Stray alignment \"&\"") })`), so it falls into THIS
        // branch instead. Without this guard, papers with multiple stray
        // `&` (e.g. astro-ph0107583's bibitem with unescaped `Hirose &
        // Osaki`) emit one Error per occurrence; Perl emits ONE total
        // because of the LOCAL `\relax` rebinding. Self-deactivate here
        // too so subsequent `&` invocations no-op.
        if token.get_catcode() == Catcode::ALIGN
          && let Some(relax_meaning) = lookup_meaning(&T_CS!("\\relax"))
        {
          assign_meaning(&token, relax_meaning, Some(Scope::Local));
        }
        // `meaning` IS the state table's `Rc<Constructor>`, so hand it to the
        // Whatsit rather than letting `invoke_primitive` deep-clone the
        // definition once per invocation (see `Constructor::invoke_primitive_shared`).
        result = Constructor::invoke_primitive_shared(&meaning)?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      Some(Stored::Primitive(meaning)) => {
        // Otherwise, a normal primitive or constructor
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      Some(Stored::MathPrimitive(meaning)) => {
        // Copy of regular Primitive
        // Otherwise, a normal primitive or constructor
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      Some(Stored::Register(meaning)) => {
        // Registers are special primitives
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      meaning => {
        // Perl: Error + makeMisdefinedError (non-fatal). Don't crash.
        Error!(
          "misdefined",
          token,
          s!("Unexpected object in Stomach: {:?}", meaning)
        );
      },
    }
    // _token_guard drops here, auto-expiring current token
    break;
  }
  stomach_mut!().token_stack.pop();
  Ok(result)
}

fn invoke_token_undefined(token: &Token) -> Result<Vec<Digested>> {
  // The LaTeX format may not be loaded yet (a document may use a kernel CS
  // before `\documentclass` — real LaTeX has no "before the kernel"). If this
  // is a kernel CS, pull the format in and re-digest instead of stubbing it as
  // `<ltx:ERROR/>`. Fires at most once per session; see
  // `binding::kernel_autoload`. Same retry shape as the `\ifsomething` arm below.
  if crate::binding::kernel_autoload::try_autoload(token) {
    gullet::unread_one(*token); // Retry, now that the kernel is in state.
    return Ok(Vec::new());
  }
  let cs = token.with_cs_name(|cs| String::from(cs));
  // Gate the undefined-CS summary tally and the Error! emission by
  // SUPPRESS_UNDEFINED_ERRORS. During expl3-code.tex raw load we install
  // the ERROR stub silently — forward references resolve when subsequent
  // post-load fixups rebind the canonical CS (see expl3_sty.rs L161-167
  // for \iow_wrap stubs that overwrite ERROR after the raw load). Mirrors
  // the existing gate at state.rs::generate_error_stub L1018-L1030.
  let suppressed = lookup_bool_sym(crate::pin!("SUPPRESS_UNDEFINED_ERRORS"));
  if !suppressed {
    note_status(LogStatus::Undefined, Some(&cs));
  }

  // To minimize chatter, go ahead and define it...
  if cs.starts_with("\\if") {
    // Apparently an \ifsomething ???
    let name = cs.replace("\\if", "");
    if !suppressed {
      let message = s!("The token {} is not defined.", token.stringify());
      Error!(
        "undefined",
        token,
        &message,
        "Defining it now as with \\newif"
      );
    }
    // install stub definitions for new conditional
    install_definition(
      Expandable::new(
        T_CS!(s!("\\{}true", name)),
        None,
        Tokens!(T_CS!("\\let"), T_CS!(&cs), T_CS!("\\iftrue")).into(),
        None,
      )?,
      None,
    );
    install_definition(
      Expandable::new(
        T_CS!(s!("\\{}false", name)),
        None,
        Tokens!(T_CS!("\\let"), T_CS!(cs), T_CS!("\\iffalse")).into(),
        None,
      )?,
      None,
    );

    let_i(token, &T_CS!("\\iffalse"), None);
    gullet::unread_one(*token); // Retry
    Ok(Vec::new())
  } else {
    if !suppressed {
      let message = s!("The token {} is not defined.", token.stringify());
      Error!(
        "undefined",
        token,
        &message,
        "Defining it now as <ltx:ERROR/>"
      );
    }
    install_definition(
      Constructor {
        cs: *token,
        paramlist: None,
        replacement: Some(Rc::new(move |document, _args, _props| {
          document.make_error("undefined", &cs)
        })),
        ..Constructor::default()
      },
      Some(Scope::Global),
    );
    // Perl: unread the token and return empty, so the outer loop re-reads
    // and dispatches through the normal path (with the newly installed stub).
    // This ensures gullet-level side effects (filtering, expansion) are applied.
    gullet::unread_one(*token);
    Ok(Vec::new())
  }
}

fn invoke_token_simple(meaning: Token) -> Result<Option<Digested>> {
  let cc = meaning.get_catcode();
  let font = lookup_font();
  // token-locators: the leaf char box's exact source position comes from the
  // token's origin handle — the position that survived expansion to digestion
  // (Experiments 1–3 showed it cannot be re-derived from the mouth here, which
  // is past the construct). `None` → `Tbox::new` falls back to the gullet's
  // current locator (the eating-disorder heuristic). See SOURCE_PROVENANCE §3.1.1.
  // Stamp a leaf box only from a *genuine* (read-from-source) origin. An
  // inherited origin — a macro's expansion attributed to its call site, e.g.
  // `\section`'s structural body literals at the `\section` column — must not
  // become a located leaf, or box-level `get_locator()` aggregation would widen
  // a construct past its content (the `\section{Intro}` title would start at the
  // command, not at "Intro"). The inherited origin still rides the token, so
  // `constructor::child_span`'s genuine-first scan can recover it as the
  // fallback for the origin-less case (`\today`). See SOURCE_PROVENANCE §3.1.3.
  #[cfg(feature = "token-locators")]
  let origin_loc: Option<crate::common::locator::Locator> =
    crate::token::get_token_origin(meaning.loc)
      .filter(|o| !o.inherited)
      .map(|o| {
        crate::common::arena::with(o.source, |s| {
          crate::common::locator::Locator::new(s, o.line, o.col, o.line, o.col)
        })
      });
  #[cfg(not(feature = "token-locators"))]
  let origin_loc: Option<crate::common::locator::Locator> = None;
  match cc {
    Catcode::SPACE => {
      clear_prefixes(); // Perl Stomach.pm line 234: prefixes shouldn't apply here.
      // Perl: if($STATE->lookupValue('MODE') =~ /(?:math|vertical)$/) { return (); }
      let mode = lookup_string_from_sym(crate::pin!("MODE"));
      if mode.ends_with("math") || mode.ends_with("vertical") {
        Ok(None)
      } else {
        enter_horizontal();
        Ok(Some(Digested::from(Tbox::new(
          meaning.get_sym(),
          font,
          origin_loc,
          Tokens!(meaning),
          HashMap::default(),
        ))))
      }
    },
    Catcode::COMMENT => {
      // Perl Stomach.pm lines 241-244: decode comment via font encoding
      let decoded = font::decode_string(meaning.get_sym(), None, true);
      let comment = arena::with(decoded, |s| {
        // However, spaces normally would have be digested away as positioning...
        // Replace NBSP + combining strikethrough (OT1 space position) with actual space
        s.replace("\u{00A0}\u{0335}", " ")
      });
      // Perl: returns LaTeXML::Core::Comment->new($comment)
      // which gets absorbed as an XML comment node via Document::insertComment
      Ok(Some(Digested::from(Comment(comment))))
    },
    _ => {
      clear_prefixes(); // Perl Stomach.pm line 247: prefixes shouldn't apply here.
      // Perl: check mathcode for IN_MATH characters (Stomach.pm lines 248-251)
      // In Perl, all math chars go through decodeMathChar which decodes via
      // the font encoding. In Rust, Tbox::new already handles IN_MATH:
      // it sets mode="math", looks up math_token_attributes for role/meaning/name,
      // and specializes the font. This produces the correct LaTeXML-level properties.
      // The mathchar parsing handles non-ASCII chars needing font map lookup.
      // TODO: Use for chars where font-encoding glyph differs from input.
      // Perl L248-257: if IN_MATH && mathcode → decodeMathChar (math box)
      // else → enterHorizontal + text box (covers non-math AND math-but-no-mathcode)
      if lookup_bool_sym(crate::pin!("IN_MATH"))
        && let Some(mathcode) = lookup_mathcode_sym(meaning.get_sym())
      {
        return crate::common::mathchar::decode_math_char_for_stomach(mathcode, meaning);
      }
      // Fallthrough: either not in math, or in math but no mathcode
      enter_horizontal();
      let text = font::decode_string(meaning.get_sym(), None, true);
      Ok(Some(Digested::from(Tbox::new(
        text,
        None,
        origin_loc,
        Tokens!(meaning),   // tokens
        HashMap::default(), // properties
      ))))
    },
  }
}

pub fn set_stomach(new_stomach: Stomach) {
  let mut singleton = stomach_mut!();
  *singleton = new_stomach;
}
pub fn clone_box_list() -> Vec<Digested> { stomach!().box_list.clone() }

/// get the current boxing level
pub fn get_boxing_level() -> usize { stomach!().boxing.len() }

/// ScriptLevel is similar to boxing level, but relative to current Math mode's level
///
/// This is used for the scriptpos attribute to recognize overlapping sccripts.
/// Making it relative to the math's level avoids unnecessary changes
pub fn get_script_level() -> usize {
  let boxlevel = get_boxing_level();
  with_value("script_base_level", |val_opt| {
    if let Some(Stored::Int(prevlevel)) = val_opt {
      boxlevel - (*prevlevel as usize) + 1
    } else {
      boxlevel
    }
  })
}

#[cfg(test)]
mod memory_cap_tests {
  use super::{
    apply_memory_ceiling, box_bytes_budget, box_count_cap, resolve_rss_cap, set_memory_cap,
    soft_cap_from_ceiling, soft_yield_urgency,
  };

  // The box-list ceilings are memory ceilings, so they must ride the SAME
  // `--max-memory` knob as the RSS fuse — including its "0 = no limit" meaning.
  // They were hardcoded `const`s (2 M boxes / 3.2 GB estimate) read
  // unconditionally, which made `--max-memory=0` a documented lie: the binary
  // prints "memory limiting disabled entirely" and then Fatal'd on a memory
  // ceiling no flag could raise. Witness: a ~10 000-page notes document
  // (Nasser Abbasi, rc4 report 2026-07-28) died on the byte budget after 8 h at
  // ~58 GB RSS having explicitly passed `--max-memory=0`.
  #[test]
  fn box_ceilings_follow_the_memory_knob() {
    // Stock ceiling reproduces the historical fixed values.
    apply_memory_ceiling(6144);
    assert_eq!(
      box_count_cap(),
      Some(1_999_933),
      "≈ the validated 2 M boxes"
    );
    let budget = box_bytes_budget().expect("stock ceiling has a byte budget");
    assert_eq!(budget, 3_221_225_472, "≈ the historical 3.2 GB");

    // `--max-memory=0` lifts BOTH — that is the whole point of the flag.
    apply_memory_ceiling(0);
    assert_eq!(box_count_cap(), None, "--max-memory=0 lifts the count cap");
    assert_eq!(
      box_bytes_budget(),
      None,
      "--max-memory=0 lifts the byte budget"
    );

    // A tighter ceiling scales them down rather than leaving them at the
    // stock value (which would sit far ABOVE the requested ceiling).
    apply_memory_ceiling(2000);
    assert!(box_count_cap().unwrap() < 1_999_933);
    assert!(box_bytes_budget().unwrap() < budget);

    // The byte budget stays UNDER the RSS fuse, so on Linux the portable
    // estimate still fires first for an accurately-estimated runaway.
    assert!(box_bytes_budget().unwrap() < resolve_rss_cap().unwrap() as usize);

    set_memory_cap(None);
  }

  // The single knob must reach the fuse through `apply_memory_ceiling`, which is
  // what every conversion path calls. `--max-memory=0` has to leave NO ceiling:
  // the plain path, the `--server` forked body child and the in-process fallback
  // all rely on this one function, and the LSP pair used to skip it entirely.
  //
  // The env-precedence half is deliberately NOT asserted here: proving it needs
  // `set_var`, and this workspace has a standing rule against touching the
  // process env from tests (a concurrent read races glibc's getenv). It holds by
  // construction instead — `apply_memory_ceiling` sets the override
  // unconditionally, and `resolve_rss_cap` only consults the env when no
  // override is present.
  #[test]
  fn apply_memory_ceiling_drives_the_fuse() {
    apply_memory_ceiling(6144);
    assert_eq!(resolve_rss_cap(), Some(4608 * 1024 * 1024));

    // 0 means "no limit", not "abort immediately".
    apply_memory_ceiling(0);
    assert_eq!(resolve_rss_cap(), None, "--max-memory=0 leaves no ceiling");

    // A tight ceiling is honored rather than ignored in favour of the old
    // built-in 4.5 GB default, which sat far ABOVE such a ceiling.
    apply_memory_ceiling(2000);
    assert_eq!(resolve_rss_cap(), Some(1500 * 1024 * 1024));

    set_memory_cap(None);
  }

  // The override path is thread-local (race-free across libtest threads) and
  // never reads the process-global env, so these assertions are deterministic.
  #[test]
  fn override_zero_disables_budget() {
    // `--max-memory=0` maps to `set_memory_cap(Some(0))`, which must resolve
    // to "no ceiling" — NOT a 0-byte cap that fatals every conversion.
    set_memory_cap(Some(0));
    assert_eq!(resolve_rss_cap(), None, "cap 0 disables the soft budget");
    // A positive override is honored verbatim.
    set_memory_cap(Some(1_000));
    assert_eq!(resolve_rss_cap(), Some(1_000));
    // Restore the default so we don't leak state onto other tests sharing
    // this thread.
    set_memory_cap(None);
  }

  #[test]
  fn soft_cap_tracks_the_single_knob() {
    // 0 in → 0 out: `--max-memory=0` disables the soft fuse (and, via
    // resolve_rss_cap, the whole memory limit).
    assert_eq!(soft_cap_from_ceiling(0), 0);
    // The soft fuse always sits strictly below the hard ceiling (graceful
    // cooperative failure fires first), and reproduces the historical
    // ~4.5 GB-under-6 GiB relationship at the 6144 MiB default.
    let hard = 6144u64 * 1024 * 1024;
    let soft = soft_cap_from_ceiling(6144);
    assert_eq!(soft, 4608u64 * 1024 * 1024);
    assert!(soft < hard, "soft fuse must be below the hard ceiling");
    // It scales with the knob, so a tight ceiling still gets a cooperative
    // guard below it (fixing the old fixed-4.5 GB decoupling).
    assert!(soft_cap_from_ceiling(2000) < 2000 * 1024 * 1024);
    assert!(soft_cap_from_ceiling(20000) > 6144 * 1024 * 1024);
  }

  /// The soft-yield floor waiver: waived at/above the watermark→fuse midpoint,
  /// applied below it, and never waived when either bound is absent or
  /// degenerate. Guards the safety valve `soft_yield_min_boxes` sits on — a
  /// pathological per-box footprint must regain per-seam yielding before the
  /// fuse fires inside one un-yielded window.
  #[test]
  fn soft_yield_floor_waiver_boundaries() {
    const MB: u64 = 1024 * 1024;
    let wm = Some(12_000 * MB); // the witness's watermark at --max-memory 48000
    let fuse = Some(36_000 * MB); // and its fuse; midpoint = 24_000 MB
    let mark_kb = 24_000 * 1024;
    assert!(
      !soft_yield_urgency(mark_kb - 1, wm, fuse),
      "below the midpoint the floor applies"
    );
    assert!(
      soft_yield_urgency(mark_kb, wm, fuse),
      "at the midpoint the floor is waived"
    );
    assert!(soft_yield_urgency(mark_kb + 1, wm, fuse), "above it too");
    // --max-memory=0 shapes: no fuse, no watermark, or fuse <= watermark
    // (a calibration LATEXML_SPILL_AT_MIB above the fuse) — never urgent.
    assert!(!soft_yield_urgency(u64::MAX / 1024, wm, None));
    assert!(!soft_yield_urgency(u64::MAX / 1024, None, fuse));
    assert!(
      !soft_yield_urgency(u64::MAX / 1024, fuse, wm),
      "fuse below watermark is degenerate"
    );
    // saturating_mul: an absurd rss_kb must not overflow into false.
    assert!(soft_yield_urgency(u64::MAX, wm, fuse));
  }
}

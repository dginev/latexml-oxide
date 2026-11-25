//! Content-addressed graphics cache.
//!
//! Every `convert_image` / `convert_image_svg` call rasterises a
//! self-contained source file (PDF / EPS / PNG / SVG) with a small set
//! of render-shaping inputs (page, density, target format). Across a
//! canvas of arXiv submissions the same byte content recurs constantly
//! — common journal logos, reused author-affiliation marks, and
//! regenerated runs of the same paper. PERFORMANCE.md §5 records the
//! `graphics` phase as **36.5%** of corpus wall, so a cache that hits
//! even a third of the time shaves ~11% off corpus wall.
//!
//! ## Heritage: Perl `LaTeXML.cache`
//!
//! The Perl post-processor stores a tied BerkeleyDB hash named
//! `LaTeXML.cache` per output directory. Keys are built from the
//! processor class, the source path, and the transform string; values
//! are `"dest|width|height"` strings. The features we mirror are
//! listed below.
//!
//! - **Disable flag** (Perl: `nocache`) → env `LATEXML_GRAPHICS_CACHE_OFF`.
//! - **Names** (Perl: `key → dest`) → the document cache, `LaTeXML-oxide.cache` in the destination directory
//!   ([`crate::document::DocCache`], [`crate::resource_cache`]); this cache holds the converted bytes only.
//! - **Re-use across runs** (Perl: one .cache per dest dir) → global XDG cache, shared across all
//!   conversions on the host. This is a strict superset of the Perl behaviour: cross-paper sharing,
//!   plus reproducible content-keying instead of path-keying.
//! - **Source-staleness check** (Perl compared source mtime to cached output mtime) → unnecessary
//!   here: the cache key is SHA-256 of the source *bytes*, so any source edit produces a different
//!   key.
//! - **Cached dimensions** (Perl: width/height in the cached value) → sidecar `<hash>.<ext>.dims`
//!   file containing `width\nheight\n`. Lookup returns `(success, dims)`; callers skip the
//!   `read_image_dimensions` syscall on hits.
//!
//! Beyond Perl: content-hash keying gives stricter staleness; XDG
//! placement gives cross-paper reuse; multi-process safety (below)
//! lets canvas sweeps share the cache without corruption.
//!
//! ## Cache key
//!
//! SHA-256 of:
//!   `source bytes ‖ page ‖ density ‖ target-extension [‖ SVG_KEY_VERSION]`
//!
//! The page/density bytes are appended as 4-byte little-endian; the
//! extension is appended as ASCII lower-case bytes, and an `svg` key ends
//! with `SVG_KEY_VERSION`, which retires cached SVGs. Bytes-only keying
//! makes the cache reproducible across machines and gives near-zero
//! collision risk for our use.
//!
//! ## On-disk layout
//!
//!   `$XDG_CACHE_HOME/latexml-oxide/graphics/<aa>/<full-hash>.<ext>`
//!   `$XDG_CACHE_HOME/latexml-oxide/graphics/<aa>/<full-hash>.<ext>.dims`
//!   `$XDG_CACHE_HOME/latexml-oxide/graphics/.prune.lock`
//!   `$XDG_CACHE_HOME/latexml-oxide/graphics/.usage`
//!
//! Sharded by the first two hex characters of the hash to keep any one
//! directory's entry count bounded (worst case ~1/256th of the total
//! cache size).
//!
//! `$XDG_CACHE_HOME` falls back to `$HOME/.cache` per the XDG spec.
//!
//! ## Multi-process safety
//!
//! The cache is designed to be shared by parallel `cortex_worker`
//! processes within a canvas sweep. Concurrency guarantees:
//!
//! 1. **Concurrent writes to the same key**: each writer copies into a private
//!    `.<stem>.tmp.<pid>.<seq>.<ext>` file ([`crate::writer::Staged`]) then issues a single
//!    `rename(2)` into the final path. `rename` is atomic on the same filesystem; if two writers
//!    race, the last `rename` wins, and because both sources are byte-equivalent the outcome is
//!    correct.
//! 2. **Concurrent reads + writes**: a reader copies the cached file into the destination the same
//!    way (a staged copy renamed onto it). An entry the writers replace by rename is never written
//!    through, so the reader's open file keeps the bytes it opened. If the cache file is unlinked
//!    between hash computation and the copy, the reader gets a miss and falls through — no data
//!    corruption.
//! 3. **Concurrent prunes**: prune holds `flock(LOCK_EX | LOCK_NB)` on `.prune.lock`. If another
//!    process already holds the lock, this one skips its prune attempt. Only one prune runs at a
//!    time per machine. The prune itself walks the dir, sorts by mtime, deletes oldest until under
//!    the low-water mark — and tolerates `ENOENT` (another writer could have re-used the entry
//!    concurrently). It also removes every `.tmp.` sidecar changed (ctime on Unix) over an hour ago:
//!    a killed process's unfinished install.
//! 4. **The usage tally**: `.usage` holds the bytes the last walk counted plus every insertion
//!    since, and when that walk ran; an insertion adds its size under an exclusive lock on the file
//!    (held for one read and one write). Drift — an entry two racing writers both count, one
//!    removed by hand, one an older binary added — lasts until the next walk recounts.
//! 5. **Destination ↔ cache**: every destination holds its own copy, so an output edited in place
//!    (a post-processing step, a user) changes neither the cache entry nor another document's copy,
//!    and a prune that deletes the entry leaves every destination whole.
//!
//! These guarantees hold on any POSIX filesystem, and on Windows, where `File::lock` is
//! `LockFileEx`. (The prune lock there was a `create_new` sentinel no one removed, so no prune ran
//! after a cache's first insertion.)
//!
//! ## Lifecycle
//!
//! * **Hit**: copy the cache file into the destination (`fs::copy`, which shares the extents where the
//!   filesystem can: Linux `copy_file_range` on btrfs or XFS). Read the `.dims` sidecar if present.
//! * **Miss-then-success**: copy the produced destination back into the cache. Write the `.dims`
//!   sidecar alongside.
//! * **LRU prune**: each insertion adds its size to the usage tally and checks it against the cap
//!   (`LATEXML_GRAPHICS_CACHE_MAX_MB`, default 2048 = 2 GB). Over the cap, or an hour after the
//!   last walk (which also recounts the tally and reclaims stale `.tmp.` files), it holds the prune
//!   lock, walks the cache, sorts entries by mtime ascending and deletes the oldest until the total
//!   is under 90 % of the cap, so the next tenth of the cap is inserted without a walk (each walk
//!   stats every entry: 20-29 ms on a 2 GB cache, GCACHECHURN). File access on read also refreshes
//!   the mtime so frequently-hit entries survive.
//!
//! ## Disable / tune
//!
//! * `LATEXML_GRAPHICS_CACHE_OFF=1` — bypass entirely (read+write both skipped). The wrapper
//!   devolves to the bare conversion call.
//! * `LATEXML_GRAPHICS_CACHE_DIR=/path` — override the cache directory.
//! * `LATEXML_GRAPHICS_CACHE_MAX_MB=N` — cache size cap.

use std::{
  fs,
  io::Read,
  path::{Path, PathBuf},
  sync::{
    OnceLock,
    atomic::{AtomicU32, Ordering},
  },
};

use sha2::{Digest, Sha256};

const DEFAULT_MAX_MB: u64 = 2048;
/// How old a `.tmp.` sidecar must be before a prune takes it for a killed install's.
const STALE_TMP_AGE: std::time::Duration = std::time::Duration::from_secs(3600);

/// Mixed into every SVG key (DG 2026-11-16): before mutool's page ≥ 2 fix, a page ≥ 2 SVG was pdftocairo's
/// render; every SVG key is retired so each page comes from the same converter (the fleet re-renders
/// each vector SVG once). Change it whenever cached SVGs must not be served again.
const SVG_KEY_VERSION: &str = "mutool-page-fix-2026-11-16";

/// Memoised disabled flag (read env once at first call).
fn disabled() -> bool {
  static CELL: OnceLock<bool> = OnceLock::new();
  *CELL.get_or_init(|| {
    matches!(
      std::env::var("LATEXML_GRAPHICS_CACHE_OFF")
        .ok()
        .as_deref()
        .map(|s| s.trim()),
      Some("1") | Some("true") | Some("yes")
    )
  })
}

/// Cache root, recomputed on each call.
///
/// Originally OnceLock-cached, but that bakes in whatever value the
/// env-var has at first call and locks out any later setter. In tests
/// (where one test binary contains both `graphics::*` and
/// `graphics_cache::*` tests), an earlier `graphics::*` test that
/// invokes `Graphics::process` triggers `cache_root()` with no
/// `LATEXML_GRAPHICS_CACHE_DIR` set — pinning the cache to
/// `~/.cache/latexml-oxide/graphics`. Subsequent `graphics_cache::*`
/// tests then set the env var via `shared_cache_dir()` but the
/// OnceLock ignores it, causing cache files to land outside the test
/// dir and the assertions to fail. Re-reading the env var on each
/// call costs ~one syscall in production (LATEXML_GRAPHICS_CACHE_DIR
/// never changes there) — negligible compared to the disk I/O the
/// cache layer drives.
fn cache_root() -> Option<PathBuf> {
  if let Ok(p) = std::env::var("LATEXML_GRAPHICS_CACHE_DIR") {
    if !p.is_empty() {
      return Some(PathBuf::from(p));
    }
  }
  let base = std::env::var_os("XDG_CACHE_HOME")
    .map(PathBuf::from)
    .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
  Some(base.join("latexml-oxide").join("graphics"))
}

/// Cache size cap in bytes, read once from env.
fn max_bytes() -> u64 {
  static CELL: OnceLock<u64> = OnceLock::new();
  *CELL.get_or_init(|| {
    std::env::var("LATEXML_GRAPHICS_CACHE_MAX_MB")
      .ok()
      .and_then(|s| s.parse::<u64>().ok())
      .unwrap_or(DEFAULT_MAX_MB)
      .saturating_mul(1024 * 1024)
  })
}

static HITS: AtomicU32 = AtomicU32::new(0);
static MISSES: AtomicU32 = AtomicU32::new(0);

/// Return `(hits, misses)` since process start. Useful for telemetry
/// and the post-run summary log line.
pub fn stats() -> (u32, u32) { (HITS.load(Ordering::Relaxed), MISSES.load(Ordering::Relaxed)) }

/// Reset stats (test-only).
#[cfg(test)]
pub fn reset_stats() {
  HITS.store(0, Ordering::Relaxed);
  MISSES.store(0, Ordering::Relaxed);
}

/// Whether a given conversion consults the shared graphics cache.
///
/// The cache root is *process-global, host-persistent* state: it comes
/// from `LATEXML_GRAPHICS_CACHE_DIR`, else
/// `$XDG_CACHE_HOME/latexml-oxide/graphics`, which survives across runs
/// and across unrelated conversions. A caller that needs its conversions
/// to actually *run* — rather than be served from whatever a previous
/// run on this host happened to leave behind — must be able to opt out,
/// and must be able to do so **without mutating the environment**
/// (`set_var` is process-wide and racy against every other thread).
/// Hence an explicit, caller-threaded policy rather than another env var.
///
/// The env kill-switch `LATEXML_GRAPHICS_CACHE_OFF=1` still applies
/// independently, inside [`lookup`] / [`store`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CachePolicy {
  /// Consult and populate the shared cache. The production default.
  #[default]
  Shared,
  /// Bypass the cache entirely: every conversion runs, nothing is read
  /// and nothing is stored. Used where the *conversion itself* is the
  /// observable under test, so a prior run's cached output must not be
  /// able to substitute for it.
  Bypass,
}

/// Render-shaping inputs that go into the cache key alongside source
/// bytes. Two calls with the same `RenderKey` MUST produce
/// byte-equivalent output (modulo metadata variation tools like
/// ImageMagick are known to introduce — see `compare_outputs_strict`
/// audit in the spawn paths).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderKey {
  /// 1-based page selector (graphicx convention). `None` ⇒ default.
  pub page:    Option<u32>,
  /// DPI for raster conversions; 0 for pure vector (SVG) paths.
  pub density: u32,
  /// Target format, derived from destination extension (lowercase, no dot).
  /// e.g. `"png"`, `"svg"`, `"jpg"`. Empty if missing.
  pub ext:     &'static str,
}

/// Cached dimensions for an image. Mirrors Perl `LaTeXML.cache`
/// `"dest|width|height"` value triple (we already have the dest path
/// at the call site — only the dimensions need round-tripping).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachedDims {
  pub width:  u32,
  pub height: u32,
}

/// A source file's SHA-256 state after its bytes: its content identity (the graphics pass's dedupe key) and, extended
/// by a [`RenderKey`], its cache key, from one read of the file.
#[derive(Clone)]
pub struct SourceDigest(Sha256);

impl SourceDigest {
  /// Hash the file at `source_path`; `None` if it can't be opened or read.
  pub fn of_file(source_path: &Path) -> Option<Self> {
    let mut file = fs::File::open(source_path).ok()?;
    let mut hasher = Sha256::new();
    // 64 KB scratch buffer; large enough to dwarf syscall overhead, small
    // enough to keep stack/heap pressure low under high concurrency.
    let mut buf = [0u8; 64 * 1024];
    loop {
      match file.read(&mut buf) {
        Ok(0) => break,
        Ok(n) => hasher.update(&buf[..n]),
        Err(_) => return None,
      }
    }
    Some(SourceDigest(hasher))
  }

  /// The SHA-256 of the file's bytes.
  pub fn content(&self) -> [u8; 32] { self.0.clone().finalize().into() }

  /// The cache-key hash of `(source bytes, key)`, as lowercase hex.
  fn key_hash(&self, key: RenderKey) -> String {
    let mut hasher = self.0.clone();
    hasher.update(key.page.unwrap_or(0).to_le_bytes());
    hasher.update(key.density.to_le_bytes());
    hasher.update(key.ext.as_bytes());
    if key.ext == "svg" {
      hasher.update(SVG_KEY_VERSION.as_bytes());
    }
    // sha2 0.11 returns a `hybrid_array::Array`, which (unlike 0.10's
    // `GenericArray`) does not implement `LowerHex`, so `format!("{:x}", _)`
    // no longer compiles. Render the digest to lowercase hex by hand — same
    // output as the old `{:x}`, and version-agnostic.
    use std::fmt::Write as _;
    let digest = hasher.finalize();
    let mut hash = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
      let _ = write!(hash, "{byte:02x}");
    }
    hash
  }
}

/// Compute the cache-key hash for `(source_bytes, render_key)`.
fn hash_key(source_path: &Path, key: RenderKey) -> Option<String> {
  Some(SourceDigest::of_file(source_path)?.key_hash(key))
}

/// The cache key of `(source, key)`: from `digest`, the source already read, or else from the file.
fn source_key(source: &str, digest: Option<&SourceDigest>, key: RenderKey) -> Option<String> {
  match digest {
    Some(digest) => Some(digest.key_hash(key)),
    None => hash_key(Path::new(source), key),
  }
}

/// Build the cache file path for a given hash + extension.
fn cache_path(root: &Path, hash: &str, ext: &str) -> PathBuf {
  let shard = &hash[..2.min(hash.len())];
  let mut p = root.join(shard);
  if ext.is_empty() {
    p.push(hash);
  } else {
    p.push(format!("{hash}.{ext}"));
  }
  p
}

/// Path to the `.dims` sidecar for a cache entry.
fn dims_sidecar(cache_file: &Path) -> PathBuf {
  let mut s = cache_file.as_os_str().to_owned();
  s.push(".dims");
  PathBuf::from(s)
}

/// Copy `src` to `dst` through a staged file and one rename ([`crate::writer::copy_atomic`]): a
/// reader of `dst` never sees it half-written, and the copy is `dst`'s own, so an output edited in
/// place changes neither the cache entry nor another document's copy (a hard link did, GFXHARDLINK).
/// `fs::copy` shares the extents where the filesystem can (Linux `copy_file_range` on btrfs, XFS).
fn deliver(src: &Path, dst: &Path) -> bool {
  if let Some(parent) = dst.parent()
    && fs::create_dir_all(parent).is_err()
  {
    return false;
  }
  crate::writer::copy_atomic(src, dst).is_ok()
}

/// Read a `.dims` sidecar if present.
fn read_dims_sidecar(sidecar: &Path) -> Option<CachedDims> {
  let raw = fs::read_to_string(sidecar).ok()?;
  let mut lines = raw.split('\n').map(|s| s.trim());
  let width = lines.next()?.parse::<u32>().ok()?;
  let height = lines.next()?.parse::<u32>().ok()?;
  Some(CachedDims { width, height })
}

/// Write a `.dims` sidecar atomically ([`crate::writer::write_atomic`]). Idempotent
/// and best-effort — failures leave no state behind.
fn write_dims_sidecar(sidecar: &Path, dims: CachedDims) {
  if let Some(parent) = sidecar.parent()
    && fs::create_dir_all(parent).is_err()
  {
    return;
  }
  let body = format!("{}\n{}\n", dims.width, dims.height);
  let _ = crate::writer::write_atomic(sidecar, body.as_bytes());
}

/// Result of a cache lookup.
#[derive(Debug, Clone, Copy)]
pub struct CacheHit {
  pub dims: Option<CachedDims>,
}

/// Look the cache up. On hit: copy into `dest`, refresh the
/// cache entry's mtime, return `Some(CacheHit{dims})`. On miss or any
/// I/O hiccup, return `None` so the caller falls through to a real
/// conversion.
pub fn lookup(source: &str, dest: &str, key: RenderKey) -> Option<CacheHit> {
  lookup_digest(source, None, dest, key)
}

/// [`lookup`], with the source's [`SourceDigest`] when the caller has it.
fn lookup_digest(
  source: &str,
  digest: Option<&SourceDigest>,
  dest: &str,
  key: RenderKey,
) -> Option<CacheHit> {
  if disabled() {
    return None;
  }
  let root = cache_root()?;
  let hash = source_key(source, digest, key)?;
  let cached = cache_path(&root, &hash, key.ext);
  if !cached.exists() {
    MISSES.fetch_add(1, Ordering::Relaxed);
    return None;
  }
  if !deliver(&cached, Path::new(dest)) {
    MISSES.fetch_add(1, Ordering::Relaxed);
    return None;
  }
  // Refresh mtime so the LRU prune doesn't evict an active entry.
  // Errors here are non-fatal; the hit already succeeded.
  let _ = touch_now(&cached);
  let dims = read_dims_sidecar(&dims_sidecar(&cached));
  HITS.fetch_add(1, Ordering::Relaxed);
  Some(CacheHit { dims })
}

/// The cache's entry for `(source, key)` when `policy` consults the cache and it holds one: what a conversion of the
/// source installs (`digest`: the source's, when read already).
pub fn cached_output(
  policy: CachePolicy,
  source: &str,
  digest: Option<&SourceDigest>,
  key: RenderKey,
) -> Option<PathBuf> {
  if policy == CachePolicy::Bypass || disabled() {
    return None;
  }
  let cached = cache_path(&cache_root()?, &source_key(source, digest, key)?, key.ext);
  cached.exists().then_some(cached)
}

/// Insert `dest` (and optionally its dimensions) into the cache under
/// `(source, key)`. Idempotent and best-effort: any I/O failure leaves
/// the cache unchanged.
pub fn store(source: &str, dest: &str, key: RenderKey, dims: Option<CachedDims>) {
  store_digest(source, None, dest, key, dims)
}

/// [`store`], with the source's [`SourceDigest`] when the caller has it.
fn store_digest(
  source: &str,
  digest: Option<&SourceDigest>,
  dest: &str,
  key: RenderKey,
  dims: Option<CachedDims>,
) {
  if disabled() {
    return;
  }
  let Some(root) = cache_root() else { return };
  let Some(hash) = source_key(source, digest, key) else {
    return;
  };
  let cached = cache_path(&root, &hash, key.ext);
  // A sidecar without its entry is never reclaimed: the prune removes one only with its entry.
  let added = install(Path::new(dest), &cached);
  if added.is_some()
    && let Some(d) = dims
  {
    write_dims_sidecar(&dims_sidecar(&cached), d);
  }
  // Best-effort LRU prune after insert (process-locked).
  prune_if_over_cap(&root, added.unwrap_or(0));
}

/// Install `dest` as the cache entry `cached`, unless an entry is already there: the bytes it added (`0` when an
/// entry was in place), `None` when no entry is in place after.
/// Concurrent writers each stage a private copy and race the rename ([`deliver`]): the last wins,
/// and both are the same bytes. A killed install's staged `.tmp.` file is reclaimed by the prune's
/// `STALE_TMP_AGE` sweep (33k such files in the fleet's cache by 2026-11-16).
fn install(dest: &Path, cached: &Path) -> Option<u64> {
  if cached.is_file() {
    let _ = touch_now(cached);
    return Some(0);
  }
  if deliver(dest, cached) {
    return Some(fs::metadata(cached).map_or(0, |meta| meta.len()));
  }
  cached.is_file().then_some(0)
}

fn touch_now(p: &Path) -> std::io::Result<()> {
  let now = std::time::SystemTime::now();
  let f = fs::File::options().write(true).open(p)?;
  f.set_modified(now)?;
  drop(f);
  Ok(())
}

/// Acquire an advisory exclusive lock on the prune sentinel file (`File::try_lock`: `flock` on
/// Unix, `LockFileEx` on Windows), held for the lifetime of the returned handle and released when
/// its process dies. `None` => another process already holds the lock; skip the prune.
fn acquire_prune_lock(root: &Path) -> Option<fs::File> {
  let lock_path = root.join(".prune.lock");
  let _ = fs::create_dir_all(root);
  let file = fs::File::options()
    .create(true)
    .write(true)
    .truncate(false)
    .open(&lock_path)
    .ok()?;
  file.try_lock().is_ok().then_some(file)
}

/// The share of the cap a prune leaves filled: the next tenth of the cap is inserted without a walk.
const LOW_WATER_PERCENT: u64 = 90;

/// The usage tally, `.usage` in the cache root: the bytes of the entries the last walk kept plus
/// every insertion's since, and when (Unix seconds) that walk ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Usage {
  bytes:  u64,
  walked: u64,
}

/// The usage tally's file, opened under an exclusive lock (blocking; held for one read and one
/// write, released when the handle drops).
fn lock_usage(root: &Path) -> Option<fs::File> {
  let file = fs::File::options()
    .read(true)
    .write(true)
    .create(true)
    .truncate(false)
    .open(root.join(".usage"))
    .ok()?;
  file.lock().ok()?;
  Some(file)
}

/// The tally `file` holds; `None` when it is empty (new) or unreadable (a write a kill cut short).
fn read_usage(file: &mut fs::File) -> Option<Usage> {
  use std::io::{Seek, SeekFrom};
  let mut raw = String::new();
  file.seek(SeekFrom::Start(0)).ok()?;
  file.read_to_string(&mut raw).ok()?;
  let mut lines = raw.lines();
  Some(Usage {
    bytes:  lines.next()?.parse().ok()?,
    walked: lines.next()?.parse().ok()?,
  })
}

fn write_usage(file: &mut fs::File, usage: Usage) {
  use std::io::{Seek, SeekFrom, Write};
  let body = format!("{}\n{}\n", usage.bytes, usage.walked);
  let _ = file
    .seek(SeekFrom::Start(0))
    .and_then(|_| file.set_len(0))
    .and_then(|_| file.write_all(body.as_bytes()));
}

fn unix_secs(t: std::time::SystemTime) -> u64 {
  t.duration_since(std::time::UNIX_EPOCH)
    .map_or(0, |d| d.as_secs())
}

/// Count an insertion of `added` bytes, and prune when that is due ([`prune_if_due`]). Errors are
/// non-fatal. Without a cap (`0`) it returns at once, so stale tmp files stay too.
fn prune_if_over_cap(root: &Path, added: u64) {
  prune_if_due(root, added, max_bytes(), std::time::SystemTime::now())
}

/// [`prune_if_over_cap`] under `cap` at time `now`: the cache is walked and pruned when the tally
/// passes the cap, when its last walk is over [`STALE_TMP_AGE`] old, or when there is no tally.
fn prune_if_due(root: &Path, added: u64, cap: u64, now: std::time::SystemTime) {
  if cap == 0 {
    return;
  }
  // A clock under an hour past the epoch has no stale tmp; the size cap still applies.
  let stale_before = now
    .checked_sub(STALE_TMP_AGE)
    .unwrap_or(std::time::UNIX_EPOCH);
  let due = lock_usage(root).is_none_or(|mut file| match read_usage(&mut file) {
    Some(usage) => {
      let bytes = usage.bytes.saturating_add(added);
      write_usage(&mut file, Usage { bytes, ..usage });
      bytes > cap || usage.walked < unix_secs(stale_before)
    },
    None => true,
  });
  if due {
    prune(root, cap, now, stale_before);
  }
}

/// When a file's inode last changed: a tmp an older binary hard-linked from an old output keeps the
/// output's mtime, but `link()` sets its ctime.
#[cfg(unix)]
fn changed(meta: &fs::Metadata) -> Option<std::time::SystemTime> {
  use std::os::unix::fs::MetadataExt;
  let since_epoch = std::time::Duration::new(
    u64::try_from(meta.ctime()).ok()?,
    u32::try_from(meta.ctime_nsec()).ok()?,
  );
  std::time::UNIX_EPOCH.checked_add(since_epoch)
}

#[cfg(not(unix))]
fn changed(meta: &fs::Metadata) -> Option<std::time::SystemTime> { meta.modified().ok() }

/// Walk the cache and total up its entries; over `cap`, delete the oldest (by mtime) until the total
/// is under [`LOW_WATER_PERCENT`] of it; remove the `.tmp.` files changed before `stale_before`; and
/// restart the usage tally from the total kept, walked at `now`. Only one process walks at a time
/// (advisory lock); an insertion during the walk stays counted.
fn prune(root: &Path, cap: u64, now: std::time::SystemTime, stale_before: std::time::SystemTime) {
  let Some(_lock) = acquire_prune_lock(root) else {
    // Another process is pruning; let it work.
    return;
  };
  let tallied = || {
    lock_usage(root)
      .and_then(|mut file| read_usage(&mut file))
      .map_or(0, |usage| usage.bytes)
  };
  let before = tallied();
  let mut entries: Vec<(PathBuf, u64, std::time::SystemTime)> = Vec::new();
  let mut total: u64 = 0;
  let Ok(shard_iter) = fs::read_dir(root) else {
    return;
  };
  for shard in shard_iter.flatten() {
    let path = shard.path();
    let Ok(meta) = shard.metadata() else { continue };
    // Skip non-dirs (e.g. .prune.lock, .usage).
    if !meta.is_dir() {
      continue;
    }
    let Ok(entry_iter) = fs::read_dir(&path) else {
      continue;
    };
    for entry in entry_iter.flatten() {
      let path = entry.path();
      let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        continue;
      };
      // A tmp sidecar is an install in flight, or one a killed process never finished: removed once
      // older than any install takes. Dim metadata follows its parent's lifecycle.
      if name.contains(".tmp.") {
        let stale = entry
          .metadata()
          .ok()
          .and_then(|meta| changed(&meta))
          .is_some_and(|changed| changed < stale_before);
        if stale {
          let _ = fs::remove_file(&path);
        }
        continue;
      }
      if name.ends_with(".dims") {
        continue;
      }
      let Ok(meta) = entry.metadata() else { continue };
      let size = meta.len();
      let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
      total = total.saturating_add(size);
      entries.push((path, size, mtime));
    }
  }
  if total > cap {
    let low_water = cap / 100 * LOW_WATER_PERCENT;
    entries.sort_by_key(|(_, _, t)| *t);
    for (path, size, _) in entries {
      if total <= low_water {
        break;
      }
      // Tolerate ENOENT: another writer may have raced this entry away.
      if fs::remove_file(&path).is_ok() {
        // Also drop the .dims sidecar if present (best-effort).
        let mut dims = path.into_os_string();
        dims.push(".dims");
        let _ = fs::remove_file(PathBuf::from(dims));
        total = total.saturating_sub(size);
      }
    }
  }
  if let Some(mut file) = lock_usage(root) {
    let since = read_usage(&mut file).map_or(0, |usage| usage.bytes.saturating_sub(before));
    write_usage(&mut file, Usage {
      bytes:  total.saturating_add(since),
      walked: unix_secs(now),
    });
  }
  // _lock drops here, releasing the prune lock.
}

/// Three-state result from a cached conversion. Disambiguates the two
/// failure modes that would otherwise collapse to `None`:
///
/// * `Ok { dims: Some(_) }` — conversion succeeded (or hit), dims known
/// * `Ok { dims: None }`     — conversion succeeded but dims unknown
/// * `Failed`                — conversion did not produce a usable output
#[derive(Debug, Clone, Copy)]
pub enum ConvertResult {
  Ok { dims: Option<CachedDims> },
  Failed,
}

impl ConvertResult {
  pub fn is_ok(&self) -> bool { matches!(self, ConvertResult::Ok { .. }) }
  pub fn dims(&self) -> Option<CachedDims> {
    match self {
      ConvertResult::Ok { dims } => *dims,
      ConvertResult::Failed => None,
    }
  }
}

/// Cache-aware wrapper around any `(source, dest) -> bool` conversion
/// that also wants to round-trip dimensions through the cache.
///
/// `measure` runs on the freshly-produced `dest` and feeds the cache
/// for next time. On a cache hit `measure` is NOT called — the cached
/// `.dims` value is returned instead.
///
/// Callers without a dimension hook can pass `measure = || None` to
/// get the bytes-only cache behaviour.
///
/// `policy` decides whether the shared cache is consulted at all; see
/// [`CachePolicy`].
pub fn with_cache_dims<F, M>(
  policy: CachePolicy,
  source: &str,
  digest: Option<&SourceDigest>,
  dest: &str,
  key: RenderKey,
  convert: F,
  measure: M,
) -> ConvertResult
where
  F: FnOnce() -> bool,
  M: FnOnce() -> Option<CachedDims>,
{
  if policy == CachePolicy::Bypass {
    if !convert() {
      return ConvertResult::Failed;
    }
    return ConvertResult::Ok { dims: measure() };
  }
  if let Some(hit) = lookup_digest(source, digest, dest, key) {
    if let Some(d) = hit.dims {
      return ConvertResult::Ok { dims: Some(d) };
    }
    // Dimensions sidecar missing or unreadable — measure now and
    // attach so future hits get it for free.
    let m = measure();
    if let Some(d) = m {
      if let (Some(root), Some(hash)) = (cache_root(), source_key(source, digest, key)) {
        let cached = cache_path(&root, &hash, key.ext);
        if cached.exists() {
          write_dims_sidecar(&dims_sidecar(&cached), d);
        }
      }
    }
    return ConvertResult::Ok { dims: m };
  }
  if !convert() {
    return ConvertResult::Failed;
  }
  let dims = measure();
  store_digest(source, digest, dest, key, dims);
  ConvertResult::Ok { dims }
}

/// Bytes-only cache wrapper. Use when the caller doesn't need
/// dimensions cached (e.g. SVG path where viewBox dims are cheap to
/// re-read from disk). Returns `true` on success.
///
/// `policy` decides whether the shared cache is consulted at all; see
/// [`CachePolicy`].
pub fn with_cache<F>(
  policy: CachePolicy,
  source: &str,
  dest: &str,
  key: RenderKey,
  convert: F,
) -> bool
where
  F: FnOnce() -> bool,
{
  if policy == CachePolicy::Bypass {
    return convert();
  }
  if lookup(source, dest, key).is_some() {
    return true;
  }
  let ok = convert();
  if ok {
    store(source, dest, key, None);
  }
  ok
}

#[cfg(test)]
mod tests {
  use std::sync::atomic::{AtomicU32, Ordering};

  use super::*;

  /// A test's scratch directory, removed when the test ends (they filled `/tmp`'s inodes).
  struct TempDir(PathBuf);
  impl std::ops::Deref for TempDir {
    type Target = Path;

    fn deref(&self) -> &Path { &self.0 }
  }
  impl Drop for TempDir {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
  }

  fn temp_dir(label: &str) -> TempDir {
    let nanos = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|d| d.as_nanos())
      .unwrap_or(0);
    let p = std::env::temp_dir().join(format!("gcache-{label}-{nanos}"));
    fs::create_dir_all(&p).unwrap();
    TempDir(p)
  }

  fn write_bytes(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, bytes).unwrap();
  }

  // These tests exercise the cache itself, so they need a real cache
  // root — pointed at a scratch directory rather than the developer's
  // `~/.cache`. One directory is shared across them (with unique source
  // bytes per test) because the root is read from the environment, which
  // is process-global.
  //
  // The env write is deliberately NOT undone: it stays set for the rest
  // of the binary's life, and `LATEXML_GRAPHICS_CACHE_DIR` is leaked into
  // any test that runs afterwards. That is tolerable only because no
  // other test's correctness depends on the cache root any more — the one
  // that used to, `graphics::tests::process_coalesces_only_matching_
  // conversion_options`, now bypasses the cache explicitly via
  // `CachePolicy::Bypass` instead of racing this setter (issue 401).
  // Taking the shared `env_lock` keeps the write from interleaving with
  // that test's `PATH` window.
  static SHARED_DIR: OnceLock<PathBuf> = OnceLock::new();
  fn shared_cache_dir() -> &'static Path {
    SHARED_DIR.get_or_init(|| {
      // Lives as long as the process (a `static`), so not a removed-on-drop `TempDir`.
      let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
      let dir = std::env::temp_dir().join(format!("gcache-shared-{nanos}"));
      fs::create_dir_all(&dir).unwrap();
      let _lock = crate::test_env::env_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
      // SAFETY: `set_var` is `unsafe` in edition 2024 because concurrent
      // env access from another thread is a data race. We hold the
      // binary-wide `env_lock`, so no other env-mutating test runs
      // concurrently, and only the first caller runs this initializer.
      unsafe {
        std::env::set_var("LATEXML_GRAPHICS_CACHE_DIR", &dir);
      }
      dir
    })
  }

  static SUFFIX: AtomicU32 = AtomicU32::new(0);
  fn unique_source_bytes(label: &str) -> Vec<u8> {
    let n = SUFFIX.fetch_add(1, Ordering::Relaxed);
    format!("{label}::{n}\n").into_bytes()
  }

  #[test]
  fn hash_changes_with_render_key() {
    let dir = temp_dir("hash");
    let src = dir.join("a.bin");
    write_bytes(&src, b"hello");
    let h_png = hash_key(&src, RenderKey {
      page:    Some(1),
      density: 90,
      ext:     "png",
    })
    .unwrap();
    let h_svg = hash_key(&src, RenderKey {
      page:    Some(1),
      density: 0,
      ext:     "svg",
    })
    .unwrap();
    let h_page2 = hash_key(&src, RenderKey {
      page:    Some(2),
      density: 90,
      ext:     "png",
    })
    .unwrap();
    assert_ne!(h_png, h_svg);
    assert_ne!(h_png, h_page2);
  }

  /// SVG keys carry [`SVG_KEY_VERSION`], so an SVG cached before mutool's page ≥ 2 fix misses; every
  /// other key is the unsalted hash of before, so the cached PNGs still hit.
  #[test]
  fn svg_keys_are_salted_and_png_keys_are_not() {
    let dir = temp_dir("salt");
    let src = dir.join("a.pdf");
    write_bytes(&src, b"%PDF-1.5 two pages");
    let unsalted = |key: RenderKey| {
      let mut hasher = Sha256::new();
      hasher.update(b"%PDF-1.5 two pages");
      hasher.update(key.page.unwrap_or(0).to_le_bytes());
      hasher.update(key.density.to_le_bytes());
      hasher.update(key.ext.as_bytes());
      let digest = hasher.finalize();
      digest
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
    };
    for (ext, density, salted) in [("svg", 0, true), ("png", 90, false), ("jpg", 90, false)] {
      for page in [None, Some(1), Some(2)] {
        let key = RenderKey { page, density, ext };
        let hash = hash_key(&src, key).unwrap();
        assert_eq!(hash != unsalted(key), salted, "{ext} page {page:?}");
      }
    }
  }

  #[test]
  fn first_call_misses_second_call_hits() {
    let _ = shared_cache_dir();
    reset_stats();
    let dir = temp_dir("hit");
    let bytes = unique_source_bytes("hit");
    let src = dir.join("src.bin");
    write_bytes(&src, &bytes);
    let dest1 = dir.join("out1.png");
    let dest2 = dir.join("out2.png");
    let key = RenderKey {
      page:    None,
      density: 90,
      ext:     "png",
    };
    let mut spawn_calls = 0u32;
    let ok1 = with_cache(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      dest1.to_str().unwrap(),
      key,
      || {
        spawn_calls += 1;
        fs::write(&dest1, b"converted-output").unwrap();
        true
      },
    );
    assert!(ok1);
    assert_eq!(spawn_calls, 1, "first call must spawn");

    let ok2 = with_cache(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      dest2.to_str().unwrap(),
      key,
      || {
        spawn_calls += 1;
        true
      },
    );
    assert!(ok2);
    assert_eq!(spawn_calls, 1, "second call must NOT spawn");
    assert_eq!(
      fs::read(&dest2).unwrap(),
      b"converted-output",
      "cache hit must deliver the original bytes"
    );
  }

  #[test]
  fn dimensions_round_trip_through_cache() {
    let _ = shared_cache_dir();
    let dir = temp_dir("dims");
    let bytes = unique_source_bytes("dims");
    let src = dir.join("src.bin");
    write_bytes(&src, &bytes);
    let dest1 = dir.join("out1.png");
    let dest2 = dir.join("out2.png");
    let key = RenderKey {
      page:    None,
      density: 90,
      ext:     "png",
    };
    let mut measure_calls = 0u32;
    // First call: miss → spawn + measure → store dims.
    let dims1 = with_cache_dims(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      None,
      dest1.to_str().unwrap(),
      key,
      || {
        fs::write(&dest1, b"png-bytes").unwrap();
        true
      },
      || {
        measure_calls += 1;
        Some(CachedDims { width: 640, height: 480 })
      },
    );
    assert!(matches!(dims1, ConvertResult::Ok {
      dims: Some(CachedDims { width: 640, height: 480 }),
    }));
    assert_eq!(measure_calls, 1, "miss measures dims");

    // Second call: hit → sidecar replay, NO spawn, NO measure.
    let dims2 = with_cache_dims(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      Some(&SourceDigest::of_file(&src).unwrap()),
      dest2.to_str().unwrap(),
      key,
      || {
        panic!("hit must skip the conversion closure");
      },
      || {
        measure_calls += 1;
        Some(CachedDims { width: 999, height: 999 })
      },
    );
    assert!(matches!(dims2, ConvertResult::Ok {
      dims: Some(CachedDims { width: 640, height: 480 }),
    }));
    assert_eq!(measure_calls, 1, "hit replays sidecar dims, no re-measure");
  }

  /// Each destination gets its own copy of a cache entry (GFXHARDLINK): an output edited in place
  /// (an append) leaves the cache entry and every other destination's file as they were.
  #[test]
  fn an_output_edited_in_place_leaves_the_cache_and_its_siblings() {
    use std::io::Write as _;
    let root = shared_cache_dir();
    let dir = temp_dir("edit-in-place");
    let src = dir.join("src.bin");
    write_bytes(&src, &unique_source_bytes("edit-in-place"));
    let key = RenderKey {
      page:    None,
      density: 0,
      ext:     "svg",
    };
    let cached = cache_path(root, &hash_key(&src, key).unwrap(), key.ext);
    let [out1, out2, out3] = ["out1", "out2", "out3"].map(|d| dir.join(d).join("fig.svg"));
    fs::create_dir_all(out1.parent().unwrap()).unwrap();
    // out1 converts and stores; out2 is served from the cache.
    let convert = || {
      fs::write(&out1, b"<svg/>").unwrap();
      true
    };
    assert!(with_cache(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      out1.to_str().unwrap(),
      key,
      convert
    ));
    assert!(lookup(src.to_str().unwrap(), out2.to_str().unwrap(), key).is_some());
    for edited in [&out1, &out2] {
      let mut file = fs::OpenOptions::new().append(true).open(edited).unwrap();
      file.write_all(b"<!-- edited -->").unwrap();
    }
    assert_eq!(fs::read(&cached).unwrap(), b"<svg/>", "the cache entry");
    assert!(lookup(src.to_str().unwrap(), out3.to_str().unwrap(), key).is_some());
    let read = |p: &Path| String::from_utf8(fs::read(p).unwrap()).unwrap();
    assert_eq!([read(&out1), read(&out2), read(&out3)], [
      "<svg/><!-- edited -->",
      "<svg/><!-- edited -->",
      "<svg/>"
    ]);
  }

  #[test]
  fn different_render_keys_dont_share_cache() {
    let _ = shared_cache_dir();
    let dir = temp_dir("keys");
    let bytes = unique_source_bytes("keys");
    let src = dir.join("src.bin");
    write_bytes(&src, &bytes);
    let dest_png = dir.join("out.png");
    let dest_svg = dir.join("out.svg");
    let key_png = RenderKey {
      page:    None,
      density: 90,
      ext:     "png",
    };
    let key_svg = RenderKey {
      page:    None,
      density: 0,
      ext:     "svg",
    };
    let mut calls = 0u32;
    let ok_png = with_cache(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      dest_png.to_str().unwrap(),
      key_png,
      || {
        calls += 1;
        fs::write(&dest_png, b"png-bytes").unwrap();
        true
      },
    );
    let ok_svg = with_cache(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      dest_svg.to_str().unwrap(),
      key_svg,
      || {
        calls += 1;
        fs::write(&dest_svg, b"svg-bytes").unwrap();
        true
      },
    );
    assert!(ok_png && ok_svg);
    assert_eq!(calls, 2, "distinct render keys must spawn separately");
  }

  #[test]
  fn miss_failure_does_not_pollute_cache() {
    let _ = shared_cache_dir();
    let dir = temp_dir("miss");
    let bytes = unique_source_bytes("miss");
    let src = dir.join("src.bin");
    write_bytes(&src, &bytes);
    let dest = dir.join("out.png");
    let key = RenderKey {
      page:    None,
      density: 90,
      ext:     "png",
    };
    let mut calls = 0u32;
    let ok = with_cache(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      dest.to_str().unwrap(),
      key,
      || {
        calls += 1;
        // simulate spawn failure: did NOT write dest, returned false
        false
      },
    );
    assert!(!ok);
    assert_eq!(calls, 1);
    let ok2 = with_cache(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      dest.to_str().unwrap(),
      key,
      || {
        calls += 1;
        false
      },
    );
    assert!(!ok2);
    assert_eq!(calls, 2, "cache must not memoise failures");
  }

  #[test]
  fn missing_disk_file_triggers_quiet_regeneration() {
    // Scenario: an earlier `store()` registered a cache entry, but
    // its on-disk file has since been removed (manual `rm`, another
    // tool clearing /tmp, or an aggressive prune from a parallel
    // worker on a different machine sharing the same network FS).
    // The next lookup must NOT raise any error, must NOT panic,
    // must report a miss, and the next `with_cache` call must
    // regenerate from the conversion closure and rewrite the entry.
    let _ = shared_cache_dir();
    let dir = temp_dir("missing-disk");
    let bytes = unique_source_bytes("missing-disk");
    let src = dir.join("src.bin");
    write_bytes(&src, &bytes);
    let dest1 = dir.join("out1.png");
    let dest2 = dir.join("out2.png");
    let key = RenderKey {
      page:    None,
      density: 90,
      ext:     "png",
    };
    let mut calls = 0u32;
    // Step 1: register a fresh entry via with_cache. Cache file is on disk.
    let ok = with_cache(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      dest1.to_str().unwrap(),
      key,
      || {
        calls += 1;
        fs::write(&dest1, b"v1-bytes").unwrap();
        true
      },
    );
    assert!(ok);
    assert_eq!(calls, 1);

    // Step 2: locate the cache file and DELETE it from disk, simulating
    // an externally-deleted entry. The `.dims` sidecar (if present) we
    // leave behind to test that orphan-sidecar tolerance also works.
    let root = shared_cache_dir();
    let hash = hash_key(&src, key).unwrap();
    let cached = cache_path(root, &hash, key.ext);
    assert!(cached.exists(), "step-1 should have written the cache file");
    fs::remove_file(&cached).unwrap();
    assert!(!cached.exists(), "cache file should now be gone");

    // Step 3: a fresh lookup must return None — no panic, no error log.
    // (The function returns Option, so we just check the variant.)
    let hit = lookup(src.to_str().unwrap(), dest2.to_str().unwrap(), key);
    assert!(
      hit.is_none(),
      "lookup must report a miss when disk file is gone"
    );

    // Step 4: with_cache must regenerate quietly. The closure should fire,
    // producing fresh output bytes. After this, the cache should hold the
    // new entry again.
    let ok2 = with_cache(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      dest2.to_str().unwrap(),
      key,
      || {
        calls += 1;
        fs::write(&dest2, b"v2-bytes-regenerated").unwrap();
        true
      },
    );
    assert!(ok2);
    assert_eq!(
      calls, 2,
      "regeneration must run the conversion closure exactly once"
    );
    assert_eq!(
      fs::read(&dest2).unwrap(),
      b"v2-bytes-regenerated",
      "dest must contain the regenerated bytes"
    );
    assert!(
      cached.exists(),
      "cache file should be restored on disk after regeneration"
    );

    // Step 5: a third lookup should now hit again (full self-heal).
    let dest3 = dir.join("out3.png");
    let hit3 = lookup(src.to_str().unwrap(), dest3.to_str().unwrap(), key);
    assert!(hit3.is_some(), "self-heal: subsequent lookup must hit");
    assert_eq!(fs::read(&dest3).unwrap(), b"v2-bytes-regenerated");
  }

  #[test]
  fn orphan_dims_sidecar_is_silently_overwritten() {
    // Scenario: cache file missing, but a stale .dims sidecar
    // (perhaps from an aborted concurrent write) is still on disk.
    // The lookup must miss (no error), the regeneration must succeed,
    // and the new .dims sidecar must contain the FRESH dimensions —
    // not the stale ones.
    let _ = shared_cache_dir();
    let dir = temp_dir("orphan-dims");
    let bytes = unique_source_bytes("orphan-dims");
    let src = dir.join("src.bin");
    write_bytes(&src, &bytes);
    let key = RenderKey {
      page:    None,
      density: 90,
      ext:     "png",
    };
    let root = shared_cache_dir();
    let hash = hash_key(&src, key).unwrap();
    let cached = cache_path(root, &hash, key.ext);
    // Manually plant a stale .dims sidecar without the main file.
    let sidecar = dims_sidecar(&cached);
    if let Some(parent) = sidecar.parent() {
      fs::create_dir_all(parent).unwrap();
    }
    fs::write(&sidecar, "1234\n5678\n").unwrap();
    assert!(sidecar.exists());
    assert!(!cached.exists());

    let dest = dir.join("out.png");
    let result = with_cache_dims(
      CachePolicy::Shared,
      src.to_str().unwrap(),
      None,
      dest.to_str().unwrap(),
      key,
      || {
        fs::write(&dest, b"fresh").unwrap();
        true
      },
      || Some(CachedDims { width: 100, height: 200 }),
    );
    assert!(matches!(result, ConvertResult::Ok {
      dims: Some(CachedDims { width: 100, height: 200 }),
    }));
    // The .dims sidecar should now contain the FRESH dims, not the stale.
    let after = read_dims_sidecar(&sidecar).unwrap();
    assert_eq!(after, CachedDims { width: 100, height: 200 });
  }

  #[test]
  fn concurrent_writers_converge_to_one_cache_entry() {
    // Multi-process safety simulation: spawn 8 threads, all writing
    // the SAME key, all writing distinct dests. After all join, the
    // cache must contain exactly one final file (no leftover .tmp.*
    // sidecars), and a follow-up hit must succeed.
    let _ = shared_cache_dir();
    let dir = temp_dir("concurrent");
    let bytes = unique_source_bytes("concurrent");
    let src = dir.join("src.bin");
    write_bytes(&src, &bytes);
    let key = RenderKey {
      page:    None,
      density: 90,
      ext:     "png",
    };

    let src_s = src.to_string_lossy().into_owned();
    let dir_s = dir.to_string_lossy().into_owned();

    std::thread::scope(|s| {
      for i in 0..8 {
        let src_s = src_s.clone();
        let dir_s = dir_s.clone();
        s.spawn(move || {
          let dest = format!("{dir_s}/dest_{i}.png");
          with_cache(CachePolicy::Shared, &src_s, &dest, key, || {
            // Tiny artificial work to encourage interleaving.
            std::thread::sleep(std::time::Duration::from_millis(5));
            fs::write(&dest, b"final-bytes").unwrap();
            true
          });
        });
      }
    });

    // Verify exactly one cache file for this hash, no leftover tmp.
    let root = shared_cache_dir();
    let hash = hash_key(&src, key).unwrap();
    let cached = cache_path(root, &hash, key.ext);
    assert!(
      cached.exists(),
      "cache entry must exist after concurrent writes"
    );
    let shard_dir = cached.parent().unwrap();
    let tmp_count = fs::read_dir(shard_dir)
      .unwrap()
      .filter_map(Result::ok)
      .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
      .count();
    assert_eq!(
      tmp_count, 0,
      "no leftover tmp sidecars after concurrent writes"
    );

    // A fresh hit must still succeed and deliver the correct bytes.
    let dest_check = dir.join("dest_check.png");
    let hit = lookup(&src_s, dest_check.to_str().unwrap(), key);
    assert!(hit.is_some(), "follow-up lookup must hit");
    assert_eq!(fs::read(&dest_check).unwrap(), b"final-bytes");
  }

  /// A shard directory's file names holding `.tmp.`.
  fn tmp_names(shard: &Path) -> Vec<String> {
    fs::read_dir(shard)
      .unwrap()
      .flatten()
      .map(|e| e.file_name().to_string_lossy().into_owned())
      .filter(|name| name.contains(".tmp."))
      .collect()
  }

  /// A store whose install fails leaves neither its tmp file nor a `.dims` sidecar, which the prune
  /// reclaims only with its entry: a missing output (the copy fails before the tmp exists), an output
  /// that fails mid-copy (a partial tmp), a directory in the entry's place (the rename fails).
  #[test]
  fn a_failed_install_leaves_neither_tmp_nor_dims() {
    let root = shared_cache_dir();
    let dir = temp_dir("install");
    let key = RenderKey {
      page:    None,
      density: 90,
      ext:     "png",
    };
    let missing = dir.join("missing.png");
    let in_the_way = dir.join("in-the-way.png");
    write_bytes(&in_the_way, b"png");
    let mut cases = vec![("missing", missing), ("in-the-way", in_the_way)];
    // Reading `/proc/self/mem` at offset 0 (unmapped) fails with EIO after the copy has created its target.
    if cfg!(target_os = "linux") {
      cases.push(("unreadable", PathBuf::from("/proc/self/mem")));
    }
    for (label, dest) in cases {
      let src = dir.join(format!("{label}.src"));
      write_bytes(&src, &unique_source_bytes(label));
      let cached = cache_path(root, &hash_key(&src, key).unwrap(), key.ext);
      if label == "in-the-way" {
        fs::create_dir_all(cached.join("occupied")).unwrap();
      }
      store(
        src.to_str().unwrap(),
        dest.to_str().unwrap(),
        key,
        Some(CachedDims { width: 1, height: 2 }),
      );
      let entry = cached.file_name().unwrap().to_string_lossy().into_owned();
      let shard = cached.parent().unwrap();
      let leftovers: Vec<String> = fs::read_dir(shard)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(&entry) && *name != entry)
        .collect();
      assert_eq!(leftovers, Vec::<String>::new(), "{label}");
      assert_eq!(cached.is_dir(), label == "in-the-way", "{label}");
      if label == "in-the-way" {
        fs::remove_dir_all(&cached).unwrap();
      }
    }
  }

  /// Over the cap a prune leaves 90 % of it (GCACHECHURN); an insertion that keeps the usage tally
  /// under the cap walks nothing, which an entry planted past the tally shows by surviving; an hour
  /// after the last walk the next insertion walks again, reclaiming the stale tmp files.
  #[test]
  fn a_prune_leaves_the_low_water_mark_and_a_later_insertion_walks_nothing() {
    let root = temp_dir("low-water");
    let cap = 1000;
    let start = std::time::SystemTime::now();
    let entry = |i: u64| {
      root
        .join(format!("{:02x}", i % 4))
        .join(format!("{i:064x}.png"))
    };
    // Twelve entries of 100 bytes, oldest first.
    for i in 0..12 {
      write_bytes(&entry(i), &[b'x'; 100]);
      let mtime = start - std::time::Duration::from_secs(1200 - i * 60);
      fs::File::options()
        .write(true)
        .open(entry(i))
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    }
    let present = || {
      (0..20)
        .filter(|&i| entry(i).is_file())
        .collect::<Vec<u64>>()
    };
    let tally = || read_usage(&mut lock_usage(&root).unwrap());
    // No tally yet: the first insertion walks, and 1200 bytes over a cap of 1000 leave 900.
    prune_if_due(&root, 100, cap, start);
    assert_eq!(present(), (3..12).collect::<Vec<u64>>());
    assert_eq!(
      tally(),
      Some(Usage {
        bytes:  900,
        walked: unix_secs(start),
      })
    );
    // 500 bytes the tally never saw: an insertion of 50 that keeps it at 950 walks nothing.
    write_bytes(&entry(12), &[b'x'; 500]);
    prune_if_due(&root, 50, cap, start);
    assert_eq!(present(), (3..13).collect::<Vec<u64>>());
    assert_eq!(
      tally(),
      Some(Usage {
        bytes:  950,
        walked: unix_secs(start),
      })
    );
    // The next 100 pass the cap: the walk finds 1400 and keeps the newest 900 (entry 12 is newest).
    prune_if_due(&root, 100, cap, start);
    assert_eq!(present(), vec![8, 9, 10, 11, 12]);
    assert_eq!(
      tally(),
      Some(Usage {
        bytes:  900,
        walked: unix_secs(start),
      })
    );
    // Two hours on, an insertion under the cap walks: the stale tmp goes, the entries stay.
    let tmp = entry(13).with_extension("png.tmp.17.1");
    write_bytes(&tmp, b"partial");
    let later = start + std::time::Duration::from_secs(7200);
    prune_if_due(&root, 0, cap, later);
    assert!(!tmp.exists(), "the stale tmp is reclaimed");
    assert_eq!(present(), vec![8, 9, 10, 11, 12]);
    assert_eq!(
      tally(),
      Some(Usage {
        bytes:  900,
        walked: unix_secs(later),
      })
    );
  }

  #[test]
  fn a_prune_reclaims_the_tmp_files_of_killed_installs() {
    let root = temp_dir("prune-tmp");
    let shard = root.join("ab");
    let old = shard.join("abcdef.png.tmp.17.1");
    let old_dims = shard.join("abcdef.png.dims.tmp.17.2");
    let entry = shard.join("abcdef.png");
    let output = root.join("out.png");
    for path in [&old, &old_dims, &entry, &output] {
      write_bytes(path, b"bytes");
    }
    let two_hours_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(7200);
    for path in [&entry, &output] {
      fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(two_hours_ago)
        .unwrap();
    }
    // The kernel stamps inode times from a coarse clock: a margin on each side of the cutoff.
    std::thread::sleep(std::time::Duration::from_millis(50));
    let stale_before = std::time::SystemTime::now();
    std::thread::sleep(std::time::Duration::from_millis(50));
    // An install in flight, hard-linked from an old output: its mtime is the output's, its ctime the link's.
    fs::hard_link(&output, shard.join("abcdef.png.tmp.18.3")).unwrap();
    prune(
      &root,
      DEFAULT_MAX_MB << 20,
      std::time::SystemTime::now(),
      stale_before,
    );
    // On Unix the link's ctime keeps the in-flight tmp. Elsewhere the age is the mtime (std exposes no ctime), so it
    // is reclaimed: the install's rename then fails and the image is just not cached that time.
    let in_flight_kept: Vec<String> = if cfg!(unix) {
      vec!["abcdef.png.tmp.18.3".to_string()]
    } else {
      Vec::new()
    };
    assert_eq!(tmp_names(&shard), in_flight_kept);
    assert!(
      entry.is_file(),
      "an entry under the cap is kept, however old"
    );
  }
}

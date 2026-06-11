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
//!   `source bytes ‖ page ‖ density ‖ target-extension`
//!
//! The page/density bytes are appended as 8-byte little-endian; the
//! extension is appended as ASCII lower-case bytes. Bytes-only keying
//! makes the cache reproducible across machines and gives near-zero
//! collision risk for our use.
//!
//! ## On-disk layout
//!
//!   `$XDG_CACHE_HOME/latexml-oxide/graphics/<aa>/<full-hash>.<ext>`
//!   `$XDG_CACHE_HOME/latexml-oxide/graphics/<aa>/<full-hash>.<ext>.dims`
//!   `$XDG_CACHE_HOME/latexml-oxide/graphics/.prune.lock`
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
//! 1. **Concurrent writes to the same key**: each writer renders to a private
//!    `<final>.tmp.<pid>.<nanos>` sidecar then issues a single `rename(2)` into the final path.
//!    `rename` is atomic on the same filesystem; if two writers race, the last `rename` wins, and
//!    because both sources are byte-equivalent the outcome is correct. `link_or_copy` retries once
//!    on `EEXIST` for the same reason.
//! 2. **Concurrent reads + writes**: a reader hardlinks the cached file into the destination. POSIX
//!    `link(2)` either succeeds (returning a new directory entry that survives any subsequent
//!    `unlink` of the source) or fails cleanly. If the cache file is unlinked between hash
//!    computation and link, the reader gets a miss and falls through — no data corruption.
//! 3. **Concurrent prunes**: prune holds `flock(LOCK_EX | LOCK_NB)` on `.prune.lock`. If another
//!    process already holds the lock, this one skips its prune attempt. Only one prune runs at a
//!    time per machine. The prune itself walks the dir, sorts by mtime, deletes oldest until under
//!    cap — and tolerates `ENOENT` (another writer could have re-used the entry concurrently).
//! 4. **Hardlinked dest ↔ cache**: when `link_or_copy` hardlinks the cache file into the
//!    destination, that hardlink is independent of the cache lifecycle. Even if a prune deletes the
//!    cache entry afterwards, the destination's hardlink keeps the data alive on the filesystem.
//!
//! These guarantees hold on any POSIX filesystem. On Windows (which
//! lacks robust `flock`), the prune lock is a best-effort `OpenOptions`
//! create-new sentinel — same correctness, slightly more retry traffic.
//!
//! ## Lifecycle
//!
//! * **Hit**: hardlink the cache file into the destination (zero-copy on the same filesystem). If
//!   hardlink fails (cross-filesystem, EXDEV), fall back to a regular file copy. Read the `.dims`
//!   sidecar if present.
//! * **Miss-then-success**: copy the produced destination back into the cache. Write the `.dims`
//!   sidecar alongside.
//! * **LRU prune**: each insertion checks the on-disk total against the cap
//!   (`LATEXML_GRAPHICS_CACHE_MAX_MB`, default 2048 = 2 GB). When over cap, holds the prune lock,
//!   sorts entries by mtime ascending, deletes until under cap. File access on read also refreshes
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

/// Compute the cache-key hash for `(source_bytes, render_key)`.
fn hash_key(source_path: &Path, key: RenderKey) -> Option<String> {
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
  hasher.update(key.page.unwrap_or(0).to_le_bytes());
  hasher.update(key.density.to_le_bytes());
  hasher.update(key.ext.as_bytes());
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
  Some(hash)
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

/// Hardlink `src` → `dst`; on cross-filesystem or other failure, fall
/// back to plain copy. Returns `true` on success.
fn link_or_copy(src: &Path, dst: &Path) -> bool {
  if let Some(parent) = dst.parent() {
    if fs::create_dir_all(parent).is_err() {
      return false;
    }
  }
  // Try hardlink first (zero-copy). Hardlinking from cache to dest is
  // safe because graphics outputs are immutable artefacts.
  if fs::hard_link(src, dst).is_ok() {
    return true;
  }
  // Hard-link can fail for cross-FS (EXDEV), permission, or because
  // the destination already exists. Try to remove and retry once.
  let _ = fs::remove_file(dst);
  if fs::hard_link(src, dst).is_ok() {
    return true;
  }
  // Final fallback: plain copy.
  fs::copy(src, dst).is_ok()
}

/// Read a `.dims` sidecar if present.
fn read_dims_sidecar(sidecar: &Path) -> Option<CachedDims> {
  let raw = fs::read_to_string(sidecar).ok()?;
  let mut lines = raw.split('\n').map(|s| s.trim());
  let width = lines.next()?.parse::<u32>().ok()?;
  let height = lines.next()?.parse::<u32>().ok()?;
  Some(CachedDims { width, height })
}

/// Write a `.dims` sidecar via tmp+rename for atomicity. Idempotent
/// and best-effort — failures leave no state behind.
fn write_dims_sidecar(sidecar: &Path, dims: CachedDims) {
  if let Some(parent) = sidecar.parent() {
    if fs::create_dir_all(parent).is_err() {
      return;
    }
  }
  let pid = std::process::id();
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_nanos())
    .unwrap_or(0);
  let mut tmp = sidecar.as_os_str().to_owned();
  tmp.push(format!(".tmp.{pid}.{nanos}"));
  let tmp_path = PathBuf::from(tmp);
  let body = format!("{}\n{}\n", dims.width, dims.height);
  if fs::write(&tmp_path, body).is_err() {
    let _ = fs::remove_file(&tmp_path);
    return;
  }
  if fs::rename(&tmp_path, sidecar).is_err() {
    let _ = fs::remove_file(&tmp_path);
  }
}

/// Result of a cache lookup.
#[derive(Debug, Clone, Copy)]
pub struct CacheHit {
  pub dims: Option<CachedDims>,
}

/// Look the cache up. On hit: hardlink/copy into `dest`, refresh the
/// cache entry's mtime, return `Some(CacheHit{dims})`. On miss or any
/// I/O hiccup, return `None` so the caller falls through to a real
/// conversion.
pub fn lookup(source: &str, dest: &str, key: RenderKey) -> Option<CacheHit> {
  if disabled() {
    return None;
  }
  let root = cache_root()?;
  let hash = hash_key(Path::new(source), key)?;
  let cached = cache_path(&root, &hash, key.ext);
  if !cached.exists() {
    MISSES.fetch_add(1, Ordering::Relaxed);
    return None;
  }
  if !link_or_copy(&cached, Path::new(dest)) {
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

/// Insert `dest` (and optionally its dimensions) into the cache under
/// `(source, key)`. Idempotent and best-effort: any I/O failure leaves
/// the cache unchanged.
pub fn store(source: &str, dest: &str, key: RenderKey, dims: Option<CachedDims>) {
  if disabled() {
    return;
  }
  let Some(root) = cache_root() else { return };
  let Some(hash) = hash_key(Path::new(source), key) else {
    return;
  };
  let cached = cache_path(&root, &hash, key.ext);
  // Atomic install: write to .tmp.<pid>.<nanos>, rename into place.
  // Concurrent writers each get a private tmp and race the rename —
  // last rename wins, both sources are byte-equivalent.
  let pid = std::process::id();
  let nanos = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_nanos())
    .unwrap_or(0);
  let mut tmp = cached.clone();
  let mut filename = cached
    .file_name()
    .map(|n| n.to_string_lossy().into_owned())
    .unwrap_or_else(|| hash.clone());
  filename.push_str(&format!(".tmp.{pid}.{nanos}"));
  tmp.set_file_name(filename);
  if !link_or_copy(Path::new(dest), &tmp) {
    return;
  }
  if !cached.exists() {
    // rename is atomic on the same filesystem; failures are non-fatal.
    let _ = fs::rename(&tmp, &cached);
  } else {
    // Cache file already present (race with another writer); clean up.
    let _ = fs::remove_file(&tmp);
    let _ = touch_now(&cached);
  }
  if let Some(d) = dims {
    write_dims_sidecar(&dims_sidecar(&cached), d);
  }
  // Best-effort LRU prune after insert (process-locked).
  prune_if_over_cap(&root);
}

fn touch_now(p: &Path) -> std::io::Result<()> {
  let now = std::time::SystemTime::now();
  let f = fs::File::options().write(true).open(p)?;
  f.set_modified(now)?;
  drop(f);
  Ok(())
}

/// Acquire an advisory exclusive lock on the prune sentinel file. The
/// lock is held for the lifetime of the returned handle. `None` =>
/// another process already holds the lock; skip the prune.
///
/// On Unix uses `flock(LOCK_EX | LOCK_NB)`. On Windows the lock is a
/// best-effort `create_new` sentinel; multi-process correctness is
/// still preserved because the prune itself is `ENOENT`-tolerant.
#[cfg(unix)]
fn acquire_prune_lock(root: &Path) -> Option<fs::File> {
  use std::os::fd::AsRawFd;
  let lock_path = root.join(".prune.lock");
  let _ = fs::create_dir_all(root);
  let file = fs::File::options()
    .create(true)
    .write(true)
    .truncate(false)
    .open(&lock_path)
    .ok()?;
  let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
  if rc == 0 { Some(file) } else { None }
}

#[cfg(not(unix))]
fn acquire_prune_lock(root: &Path) -> Option<fs::File> {
  let lock_path = root.join(".prune.lock");
  let _ = fs::create_dir_all(root);
  fs::File::options()
    .create_new(true)
    .write(true)
    .open(&lock_path)
    .ok()
}

#[cfg(not(unix))]
fn release_prune_lock(root: &Path, _f: fs::File) {
  let _ = fs::remove_file(root.join(".prune.lock"));
}

/// Walk the cache, total up sizes, and if over the cap delete oldest
/// entries (by mtime) until under. Errors are non-fatal. Only one
/// process performs a prune at a time (advisory lock).
fn prune_if_over_cap(root: &Path) {
  let cap = max_bytes();
  if cap == 0 {
    return;
  }
  let Some(_lock) = acquire_prune_lock(root) else {
    // Another process is pruning; let it work.
    return;
  };
  let mut entries: Vec<(PathBuf, u64, std::time::SystemTime)> = Vec::new();
  let mut total: u64 = 0;
  let Ok(shard_iter) = fs::read_dir(root) else {
    return;
  };
  for shard in shard_iter.flatten() {
    let path = shard.path();
    let Ok(meta) = shard.metadata() else { continue };
    // Skip non-dirs (e.g. .prune.lock).
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
      // Skip in-flight tmp sidecars and dim metadata (they're tiny
      // and follow their parent's lifecycle when removed).
      if name.contains(".tmp.") || name.ends_with(".dims") {
        continue;
      }
      let Ok(meta) = entry.metadata() else { continue };
      let size = meta.len();
      let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
      total = total.saturating_add(size);
      entries.push((path, size, mtime));
    }
  }
  if total <= cap {
    // Lock auto-released when `_lock` goes out of scope at fn end.
    return;
  }
  entries.sort_by_key(|(_, _, t)| *t);
  let mut to_free = total.saturating_sub(cap);
  for (path, size, _) in entries {
    if to_free == 0 {
      break;
    }
    // Tolerate ENOENT: another writer may have raced this entry away.
    if fs::remove_file(&path).is_ok() {
      // Also drop the .dims sidecar if present (best-effort).
      let mut dims = path.into_os_string();
      dims.push(".dims");
      let _ = fs::remove_file(PathBuf::from(dims));
      to_free = to_free.saturating_sub(size);
    }
  }
  // _lock drops here, releasing the flock.
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
pub fn with_cache_dims<F, M>(
  source: &str,
  dest: &str,
  key: RenderKey,
  convert: F,
  measure: M,
) -> ConvertResult
where
  F: FnOnce() -> bool,
  M: FnOnce() -> Option<CachedDims>,
{
  if let Some(hit) = lookup(source, dest, key) {
    if let Some(d) = hit.dims {
      return ConvertResult::Ok { dims: Some(d) };
    }
    // Dimensions sidecar missing or unreadable — measure now and
    // attach so future hits get it for free.
    let m = measure();
    if let Some(d) = m {
      if let (Some(root), Some(hash)) = (cache_root(), hash_key(Path::new(source), key)) {
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
  store(source, dest, key, dims);
  ConvertResult::Ok { dims }
}

/// Bytes-only cache wrapper. Use when the caller doesn't need
/// dimensions cached (e.g. SVG path where viewBox dims are cheap to
/// re-read from disk). Returns `true` on success.
pub fn with_cache<F>(source: &str, dest: &str, key: RenderKey, convert: F) -> bool
where F: FnOnce() -> bool {
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

  fn temp_dir(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|d| d.as_nanos())
      .unwrap_or(0);
    let p = std::env::temp_dir().join(format!("gcache-{label}-{nanos}"));
    fs::create_dir_all(&p).unwrap();
    p
  }

  fn write_bytes(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, bytes).unwrap();
  }

  // The cache_root() OnceLock locks the cache root on first call. To
  // isolate per-test directories we'd need to reset the OnceLock,
  // which the public API doesn't support. Instead, share one cache
  // directory across tests and use unique source bytes per test.
  static SHARED_DIR: OnceLock<PathBuf> = OnceLock::new();
  fn shared_cache_dir() -> &'static Path {
    SHARED_DIR.get_or_init(|| {
      let dir = temp_dir("shared");
      // SAFETY: tests run in their own process; the env-var write is
      // serialized by the OnceLock initializer (only the first caller
      // runs the closure).
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
    let ok1 = with_cache(src.to_str().unwrap(), dest1.to_str().unwrap(), key, || {
      spawn_calls += 1;
      fs::write(&dest1, b"converted-output").unwrap();
      true
    });
    assert!(ok1);
    assert_eq!(spawn_calls, 1, "first call must spawn");

    let ok2 = with_cache(src.to_str().unwrap(), dest2.to_str().unwrap(), key, || {
      spawn_calls += 1;
      true
    });
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
      src.to_str().unwrap(),
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
      src.to_str().unwrap(),
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
    let ok = with_cache(src.to_str().unwrap(), dest.to_str().unwrap(), key, || {
      calls += 1;
      // simulate spawn failure: did NOT write dest, returned false
      false
    });
    assert!(!ok);
    assert_eq!(calls, 1);
    let ok2 = with_cache(src.to_str().unwrap(), dest.to_str().unwrap(), key, || {
      calls += 1;
      false
    });
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
    let ok = with_cache(src.to_str().unwrap(), dest1.to_str().unwrap(), key, || {
      calls += 1;
      fs::write(&dest1, b"v1-bytes").unwrap();
      true
    });
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
    let ok2 = with_cache(src.to_str().unwrap(), dest2.to_str().unwrap(), key, || {
      calls += 1;
      fs::write(&dest2, b"v2-bytes-regenerated").unwrap();
      true
    });
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
      src.to_str().unwrap(),
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
          with_cache(&src_s, &dest, key, || {
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
}

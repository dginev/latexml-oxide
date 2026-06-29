//! Unsaved-buffer overlay (`docs/archive/LSP_MULTIFILE_PLAN.md` §3B), riding
//! the engine's **existing** `{file}_contents` state-value channel —
//! the Perl-faithful `\begin{filecontents}` cache that `find_file_aux`
//! (existence), the definitions loader, the `\input` open path, and the
//! raw cls/sty dep-scan all already consult
//! (`latexml_core/src/binding/content.rs:1252/1343/1820/2494`). The
//! plan's original sketch hooked `Mouth::create`; this channel is
//! strictly better: zero engine diff, and fork children inherit the
//! state values via COW automatically.
//!
//! Per conversion the server re-applies every open buffer under several
//! lookup keys (the engine probes `_contents` both by the *literal
//! requested* name — possibly extension-less, e.g. `sections/ch2` —
//! and by the *resolved* path):
//!   absolute, project-relative, basename — each with and without the
//!   extension. kpathsea/texmf paths can never collide with these keys,
//!   so the texmf tree is structurally exempt (plan §2).
//!
//! Also home to the warm-cache dependency snapshot: the set of sources
//! the preamble warm-up actually opened
//! (`state::opened_sources_snapshot()` — the dedicated
//! `Mouth::create`-level read-log), each pinned as `Overlay(version)`
//! or `Disk(mtime)`.

use std::{path::Path, time::SystemTime};

use latexml_core::state;
use rustc_hash::FxHashMap;

/// One open editor buffer.
#[derive(Debug, Clone)]
pub(crate) struct Buffer {
  pub version: i64,
  pub text:    String,
}

/// The lookup keys a buffer is registered under. Basename keys are
/// skipped when two buffers share a basename (ambiguous — disk wins).
fn buffer_keys(path: &str, project_dir: &Path, ambiguous_base: bool) -> Vec<String> {
  let p = Path::new(path);
  let mut keys = vec![path.to_string()];
  if let Ok(rel) = p.strip_prefix(project_dir)
    && let Some(rel_str) = rel.to_str()
    && rel_str != path
  {
    keys.push(rel_str.to_string());
  }
  if !ambiguous_base
    && let Some(base) = p.file_name().and_then(|s| s.to_str())
    && !keys.iter().any(|k| k == base)
  {
    keys.push(base.to_string());
  }
  // Extension-less variants: `\input{sections/ch2}` probes the literal
  // name before any extension is appended.
  let mut extless: Vec<String> = Vec::new();
  for k in &keys {
    if let Some(stripped) = k.strip_suffix(".tex") {
      extless.push(stripped.to_string());
    }
  }
  keys.extend(extless);
  keys
}

/// Apply the open buffers of `project_dir` to the engine's
/// `{key}_contents` channel. Returns the full key set that is now live;
/// pass the previous apply's keys so entries for closed/renamed buffers
/// are cleared (an empty string is a cache miss for every consumer).
///
/// MUST be called after any `reset_thread_state` and before digestion —
/// on the cache-hit path, before the body fork (children inherit the
/// values via COW).
pub(crate) fn overlay_apply(
  buffers: &FxHashMap<String, Buffer>,
  project_dir: &Path,
  previous_keys: &[String],
) -> Vec<String> {
  let mut base_counts: FxHashMap<&str, usize> = FxHashMap::default();
  for path in buffers.keys() {
    if let Some(base) = Path::new(path).file_name().and_then(|s| s.to_str()) {
      *base_counts.entry(base).or_insert(0) += 1;
    }
  }
  let mut live: Vec<String> = Vec::new();
  for (path, buf) in buffers {
    let ambiguous = Path::new(path)
      .file_name()
      .and_then(|s| s.to_str())
      .map(|b| base_counts.get(b).copied().unwrap_or(0) > 1)
      .unwrap_or(true);
    for key in buffer_keys(path, project_dir, ambiguous) {
      state::assign_value(
        &format!("{key}_contents"),
        buf.text.clone(),
        Some(state::Scope::Global),
      );
      live.push(key);
    }
  }
  for old in previous_keys {
    if !live.iter().any(|k| k == old) {
      state::assign_value(
        &format!("{old}_contents"),
        String::new(),
        Some(state::Scope::Global),
      );
    }
  }
  live
}

/// The pinned state of one warm-up dependency.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DepState {
  /// Served from an open buffer at this version.
  Overlay(i64),
  /// Read from disk with this mtime.
  Disk(Option<SystemTime>),
  /// Neither an open buffer nor on disk (literal/anonymous source).
  Ephemeral,
}

/// Compute the current `DepState` for a recorded source path.
fn dep_state(source: &str, buffers: &FxHashMap<String, Buffer>) -> DepState {
  if let Some(buf) = buffers.get(source) {
    return DepState::Overlay(buf.version);
  }
  // Overlay keys are also registered relative / basename — match by
  // path-segment suffix (boundary `/`) so a literal-keyed open
  // ("sections/ch2.tex") pins to its buffer but "my_ch2.tex" doesn't.
  let seg_suffix = |hay: &str, suf: &str| {
    hay.len() > suf.len() && hay.ends_with(suf) && hay.as_bytes()[hay.len() - suf.len() - 1] == b'/'
  };
  for (path, buf) in buffers {
    if seg_suffix(path, source) || seg_suffix(source, path) {
      return DepState::Overlay(buf.version);
    }
  }
  let p = Path::new(source);
  if (p.is_absolute() || p.exists())
    && let Ok(meta) = std::fs::metadata(p)
  {
    return DepState::Disk(meta.modified().ok());
  }
  DepState::Ephemeral
}

/// Snapshot the warm-up read-log: every named source the engine opened
/// during the preamble digest (`state::opened_sources_snapshot()` — the
/// `Mouth::create`-level log), pinned at its current state. Call right
/// after the preamble digest, on the warm-up thread.
///
/// NOT the locator `source_table` — that one is populated lazily at
/// *document-construction* time (which happens in the forked body
/// child, after this snapshot) and filters to user sources (no `.sty`),
/// so it is empty/blind exactly when this snapshot runs. Using it
/// shipped a stale-preamble bug: unsaved edits of a preamble-consumed
/// file never invalidated the warm cache.
///
/// `root` (the document whose preamble was digested) is EXCLUDED: its
/// preamble half is already keyed by exact string equality in the
/// cache-hit check, and pinning the whole root buffer's version here
/// would invalidate on every *body* keystroke — defeating the warm
/// cache for the most common editing flow.
pub(crate) fn warmup_dep_snapshot(
  buffers: &FxHashMap<String, Buffer>,
  root: &Path,
) -> Vec<(String, DepState)> {
  let root_str = root.to_string_lossy();
  state::opened_sources_snapshot()
    .iter()
    .filter_map(|sym| {
      let name = latexml_core::common::arena::with(*sym, |s| s.to_string());
      if name == root_str {
        return None;
      }
      let st = dep_state(&name, buffers);
      Some((name, st))
    })
    .collect()
}

/// Is a previously-taken snapshot still current?
pub(crate) fn deps_still_current(
  snapshot: &[(String, DepState)],
  buffers: &FxHashMap<String, Buffer>,
) -> bool {
  snapshot
    .iter()
    .all(|(name, pinned)| dep_state(name, buffers) == *pinned)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn buffers(entries: &[(&str, i64, &str)]) -> FxHashMap<String, Buffer> {
    entries
      .iter()
      .map(|(p, v, t)| {
        (p.to_string(), Buffer {
          version: *v,
          text:    t.to_string(),
        })
      })
      .collect()
  }

  #[test]
  fn buffer_keys_cover_literal_and_resolved_probes() {
    let keys = buffer_keys("/proj/sections/ch2.tex", Path::new("/proj"), false);
    for expected in [
      "/proj/sections/ch2.tex",
      "sections/ch2.tex",
      "ch2.tex",
      "/proj/sections/ch2",
      "sections/ch2",
      "ch2",
    ] {
      assert!(
        keys.iter().any(|k| k == expected),
        "missing key {expected}: {keys:?}"
      );
    }
  }

  #[test]
  fn ambiguous_basenames_get_no_bare_key() {
    let keys = buffer_keys("/proj/a/refs.tex", Path::new("/proj"), true);
    assert!(!keys.iter().any(|k| k == "refs.tex"), "{keys:?}");
    assert!(keys.iter().any(|k| k == "a/refs.tex"));
  }

  #[test]
  fn dep_state_prefers_overlay_then_disk() {
    let tmp = tempfile::tempdir().unwrap();
    let on_disk = tmp.path().join("real.tex");
    std::fs::write(&on_disk, "x").unwrap();
    let bufs = buffers(&[("/proj/sections/ch2.tex", 7, "body")]);

    assert_eq!(
      dep_state("/proj/sections/ch2.tex", &bufs),
      DepState::Overlay(7)
    );
    // Literal-keyed source pins to the same buffer by suffix.
    assert_eq!(dep_state("sections/ch2.tex", &bufs), DepState::Overlay(7));
    assert!(matches!(
      dep_state(on_disk.to_str().unwrap(), &bufs),
      DepState::Disk(Some(_))
    ));
    assert_eq!(dep_state("Anonymous String", &bufs), DepState::Ephemeral);
  }

  #[test]
  fn warmup_snapshot_uses_read_log_and_excludes_root() {
    use latexml_core::common::arena;
    state::reset_thread_state();
    state::record_opened_source(arena::pin("/proj/main.tex"));
    state::record_opened_source(arena::pin("/proj/defs.tex"));
    state::record_opened_source(arena::pin("/proj/defs.tex")); // deduped
    let bufs = buffers(&[("/proj/main.tex", 3, "root"), ("/proj/defs.tex", 5, "defs")]);
    let snap = warmup_dep_snapshot(&bufs, Path::new("/proj/main.tex"));
    assert!(
      snap.iter().all(|(n, _)| n != "/proj/main.tex"),
      "the root must not be pinned (body edits bump its version): {snap:?}"
    );
    assert_eq!(snap, vec![(
      "/proj/defs.tex".to_string(),
      DepState::Overlay(5)
    )]);
    // Bumping the ROOT's version must not invalidate (preamble equality is
    // its key); bumping the preamble-consumed dep's version must.
    let mut bufs2 = bufs.clone();
    bufs2.get_mut("/proj/main.tex").unwrap().version = 99;
    assert!(deps_still_current(&snap, &bufs2));
    bufs2.get_mut("/proj/defs.tex").unwrap().version = 6;
    assert!(!deps_still_current(&snap, &bufs2));
  }

  #[test]
  fn snapshot_invalidates_on_version_bump_and_disk_touch() {
    let tmp = tempfile::tempdir().unwrap();
    let sty = tmp.path().join("macros.sty");
    std::fs::write(&sty, "v1").unwrap();
    let mut bufs = buffers(&[("/proj/sections/ch2.tex", 1, "body")]);
    let snapshot = vec![
      (
        "/proj/sections/ch2.tex".to_string(),
        dep_state("/proj/sections/ch2.tex", &bufs),
      ),
      (
        sty.to_str().unwrap().to_string(),
        dep_state(sty.to_str().unwrap(), &bufs),
      ),
    ];
    assert!(deps_still_current(&snapshot, &bufs));
    // Buffer version bump invalidates.
    bufs.get_mut("/proj/sections/ch2.tex").unwrap().version = 2;
    assert!(!deps_still_current(&snapshot, &bufs));
    bufs.get_mut("/proj/sections/ch2.tex").unwrap().version = 1;
    assert!(deps_still_current(&snapshot, &bufs));
    // Disk change invalidates: pin a snapshot at an mtime that can't be
    // current (UNIX_EPOCH) and verify it reads as stale.
    let stale = vec![(
      sty.to_str().unwrap().to_string(),
      DepState::Disk(Some(SystemTime::UNIX_EPOCH)),
    )];
    assert!(!deps_still_current(&stale, &bufs));
  }

  #[test]
  fn recorded_rhai_dep_pins_to_disk_and_invalidates_on_edit() {
    // Models the runtime `.rhai` binding flow: `rhai_dispatch` records the
    // resolved `.rhai` path in the opened-sources read-log (the binding is
    // loaded via a raw `std::fs` read, never through a `Mouth`, so nothing
    // else logs it). The warm-cache snapshot must pin it as `Disk(mtime)` —
    // it is never an open editor buffer — and an edit (mtime change) must
    // read as stale, otherwise an edited binding survives in the warm
    // preamble cache and the user sees no effect.
    use std::time::Duration;

    use latexml_core::common::arena;

    let tmp = tempfile::tempdir().unwrap();
    let rhai = tmp.path().join("example.sty.rhai");
    std::fs::write(&rhai, "DefMacro(\"\\\\foo\", || \"FOO\");\n").unwrap();
    let rhai_path = rhai.to_str().unwrap().to_string();

    state::reset_thread_state();
    state::record_opened_source(arena::pin(&rhai_path)); // what rhai_dispatch now does

    // The binding only ever exists on disk (never an open editor buffer).
    let bufs: FxHashMap<String, Buffer> = FxHashMap::default();
    let snap = warmup_dep_snapshot(&bufs, &tmp.path().join("main.tex"));
    assert!(
      matches!(snap.as_slice(), [(n, DepState::Disk(Some(_)))] if *n == rhai_path),
      "the recorded .rhai must be pinned as Disk(mtime): {snap:?}"
    );
    assert!(deps_still_current(&snap, &bufs), "an unedited binding stays current");

    // Edit the binding (new content + a distinct mtime): the warm cache,
    // gated on `deps_still_current`, must now miss and re-digest the preamble.
    std::fs::write(&rhai, "DefMacro(\"\\\\foo\", || \"BAR\");\n").unwrap();
    std::fs::File::open(&rhai)
      .unwrap()
      .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000))
      .unwrap();
    assert!(!deps_still_current(&snap, &bufs), "an edited binding reads as stale");
  }
}

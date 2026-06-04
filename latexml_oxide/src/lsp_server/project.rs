//! Project-root resolution: map an edited buffer to the document the
//! engine should actually convert (`docs/LSP_MULTIFILE_PLAN.md` §3A).
//!
//! Resolution order (first hit wins):
//!  1. client override (`initializationOptions.rootDocument`),
//!  2. `% !TEX root = <path>` magic comment in the buffer (texlab /
//!     LaTeX-Workshop convention),
//!  3. buffer carries an un-commented `\documentclass`/`\documentstyle`
//!     → the buffer IS a root (fast path; v1-identical for the common
//!     single-file case, no directory scan),
//!  4. directory detection via [`crate::main_tex::find_main_tex`] — the
//!     `--whatsin=directory` arXiv heuristic (00README.json/XXX,
//!     Pack.pm likelihood scoring with `\input` vetoes) — walking up at
//!     most [`WALKUP_MAX`] parents. A candidate ≠ the buffer is only
//!     trusted when it (textually) *references* the buffer, so a
//!     directory of unrelated documents can never hijack a fragment,
//!  5. the buffer itself (safe degradation).

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashMap;

/// How many parent directories detection may climb when the buffer's
/// own directory yields no trusted root. Guarded: a parent is only
/// scanned when it contains at least one `.tex` file, so the heuristic
/// never runs on `$HOME`.
const WALKUP_MAX: usize = 2;

/// `% !TEX root = ../main.tex` (texlab / LaTeX-Workshop magic comment).
static MAGIC_ROOT: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"(?mi)^[ \t]*%+[ \t]*!?[ \t]*TEX[ \t]+root[ \t]*=[ \t]*(.+?)[ \t]*$").unwrap()
});

/// Lexically normalize `path` (resolve `.`/`..` components, no symlink
/// traversal — overlay-only files may not exist on disk).
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
  let mut out = PathBuf::new();
  for comp in path.components() {
    match comp {
      std::path::Component::CurDir => {},
      std::path::Component::ParentDir => {
        if !out.pop() {
          out.push("..");
        }
      },
      other => out.push(other),
    }
  }
  out
}

/// The directory a project root presides over (its parent directory).
pub(crate) fn project_dir(root: &Path) -> PathBuf {
  root.parent().map(Path::to_path_buf).unwrap_or_default()
}

/// Is `other` part of the project rooted at `root`? (Lexical: lives at
/// or under the root's directory.) Used for preemption + coalescing:
/// any conversion trigger inside the same project supersedes the
/// in-flight compile of that project.
pub(crate) fn same_project(root: &Path, other: &Path) -> bool {
  let dir = project_dir(root);
  !dir.as_os_str().is_empty() && normalize_path(other).starts_with(&dir)
}

/// Does `text` contain an un-commented occurrence of `needle`?
/// Comment-awareness reuses the `\%`-escape-aware scanner from the
/// preamble splitter.
fn contains_uncommented(text: &str, needle: &str) -> bool {
  let spans = super::comment_spans(text);
  let mut from = 0;
  while let Some(rel) = text[from..].find(needle) {
    let pos = from + rel;
    if !spans.iter().any(|&(s, e)| s < pos && pos < e) {
      return true;
    }
    from = pos + needle.len();
  }
  false
}

/// Does the buffer text declare a document class — i.e. is it a
/// self-contained root rather than an `\input` fragment?
pub(crate) fn is_self_contained(text: &str) -> bool {
  contains_uncommented(text, "\\documentclass") || contains_uncommented(text, "\\documentstyle")
}

/// Does candidate-root content reference the buffer (its stem appearing
/// on an un-commented line that also carries an input-like macro)?
/// Token-boundary check on the stem so `ch2` doesn't match `ch20`.
fn references_buffer(root_text: &str, buffer: &Path) -> bool {
  let Some(stem) = buffer.file_stem().and_then(|s| s.to_str()) else {
    return false;
  };
  for line in root_text.lines() {
    let line = match line.find('%') {
      // Cheap comment strip (a literal `\%` before an `\input` on the
      // same line is unusual enough to accept the false negative).
      Some(i) => &line[..i],
      None => line,
    };
    if !(line.contains("\\input")
      || line.contains("\\include")
      || line.contains("\\subfile")
      || line.contains("\\import"))
    {
      continue;
    }
    let bytes = line.as_bytes();
    let mut from = 0;
    while let Some(rel) = line[from..].find(stem) {
      let pos = from + rel;
      let before_ok = pos == 0 || !bytes[pos - 1].is_ascii_alphanumeric();
      let after = pos + stem.len();
      let after_ok = after >= bytes.len() || !bytes[after].is_ascii_alphanumeric();
      if before_ok && after_ok {
        return true;
      }
      from = after;
    }
  }
  false
}

/// Extract a `% !TEX root` target from buffer text, resolved against
/// the buffer's directory. Only honored when the target exists on disk
/// or is the buffer itself (a dangling magic comment falls through to
/// detection rather than converting a phantom path).
pub(crate) fn magic_root(text: &str, buffer_path: &Path) -> Option<PathBuf> {
  let m = MAGIC_ROOT.captures(text)?;
  let target = m.get(1)?.as_str();
  let dir = buffer_path.parent()?;
  let mut joined = normalize_path(&dir.join(target));
  if joined.extension().is_none() {
    joined.set_extension("tex");
  }
  if joined == buffer_path || joined.exists() {
    Some(joined)
  } else {
    None
  }
}

#[derive(Debug, Clone)]
struct CachedRoot {
  root:      Option<PathBuf>,
  /// mtime of the directory the detection ran on, so saving/creating
  /// files re-runs detection. (Content edits of existing files do not
  /// change a POSIX directory mtime — nor detection inputs, materially.)
  dir_mtime: Option<SystemTime>,
}

/// Per-directory cache of `find_main_tex` outcomes. Detection scans and
/// scores every TeX file in a directory, so it must not run per
/// keystroke.
#[derive(Default)]
pub(crate) struct RootCache {
  map: FxHashMap<PathBuf, CachedRoot>,
}

fn dir_mtime(dir: &Path) -> Option<SystemTime> {
  std::fs::metadata(dir).and_then(|m| m.modified()).ok()
}

fn dir_has_tex(dir: &Path) -> bool {
  std::fs::read_dir(dir)
    .map(|entries| {
      entries.flatten().any(|e| {
        e.path()
          .extension()
          .and_then(|x| x.to_str())
          .is_some_and(|x| x.eq_ignore_ascii_case("tex"))
      })
    })
    .unwrap_or(false)
}

impl RootCache {
  /// `find_main_tex(dir)`, cached on the directory's mtime.
  fn detect_dir(&mut self, dir: &Path) -> Option<PathBuf> {
    let mtime = dir_mtime(dir);
    if let Some(cached) = self.map.get(dir) {
      if cached.dir_mtime == mtime {
        return cached.root.clone();
      }
    }
    let root = if dir_has_tex(dir) {
      crate::main_tex::find_main_tex(dir).ok().map(|p| normalize_path(&p))
    } else {
      None
    };
    self.map.insert(dir.to_path_buf(), CachedRoot {
      root: root.clone(),
      dir_mtime: mtime,
    });
    root
  }
}

/// Resolve the project root for an edited buffer. `text` is the
/// buffer's current content when available; `override_root` is the
/// client-configured root, honored verbatim when it exists on disk.
pub(crate) fn resolve_root(
  cache: &mut RootCache,
  override_root: Option<&Path>,
  buffer_path: &Path,
  text: Option<&str>,
) -> PathBuf {
  let buffer_path = normalize_path(buffer_path);
  if let Some(ov) = override_root {
    let ov = normalize_path(ov);
    if ov.exists() || ov == buffer_path {
      return ov;
    }
    log::warn!("rootDocument override {} does not exist; ignoring", ov.display());
  }
  if let Some(t) = text {
    if let Some(magic) = magic_root(t, &buffer_path) {
      return magic;
    }
    // A self-contained buffer IS a root: the common single-file case,
    // resolved with zero filesystem scanning (v1-identical).
    if is_self_contained(t) {
      return buffer_path;
    }
  }
  // Fragment (or content unknown): look for a root that references us,
  // in this directory then up to WALKUP_MAX parents.
  let mut dir = buffer_path.parent().map(Path::to_path_buf);
  for _ in 0..=WALKUP_MAX {
    let Some(d) = dir else { break };
    if let Some(candidate) = cache.detect_dir(&d) {
      if candidate != buffer_path {
        let references = std::fs::read_to_string(&candidate)
          .map(|t| references_buffer(&t, &buffer_path))
          .unwrap_or(false);
        if references {
          return candidate;
        }
      }
    }
    dir = d.parent().map(Path::to_path_buf);
  }
  buffer_path
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;

  fn write(dir: &Path, rel: &str, content: &str) -> PathBuf {
    let p = dir.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, content).unwrap();
    p
  }

  #[test]
  fn magic_comment_resolves_relative_to_buffer() {
    let tmp = tempfile::tempdir().unwrap();
    let main = write(tmp.path(), "main.tex", "\\documentclass{article}");
    let ch = write(
      tmp.path(),
      "sections/ch2.tex",
      "% !TEX root = ../main.tex\nBody",
    );
    assert_eq!(
      magic_root("% !TEX root = ../main.tex\nBody", &ch),
      Some(normalize_path(&main))
    );
    // Extension-less target gains `.tex`; case-insensitive tag.
    assert_eq!(
      magic_root("%!tex ROOT = ../main", &ch),
      Some(normalize_path(&main))
    );
    // Dangling target is ignored (falls through to detection).
    assert_eq!(magic_root("% !TEX root = ../nothere.tex", &ch), None);
  }

  #[test]
  fn fragment_resolves_to_referencing_parent_root() {
    let tmp = tempfile::tempdir().unwrap();
    let main = write(
      tmp.path(),
      "main.tex",
      "\\documentclass{article}\n\\begin{document}\n\\input{sections/ch2}\n\\end{document}\n",
    );
    let ch = write(tmp.path(), "sections/ch2.tex", "chapter body, no class");
    let mut cache = RootCache::default();
    assert_eq!(
      resolve_root(&mut cache, None, &ch, Some("chapter body, no class")),
      normalize_path(&main)
    );
    // Second resolve hits the directory cache (observable only as: same answer).
    assert_eq!(
      resolve_root(&mut cache, None, &ch, Some("edited body")),
      normalize_path(&main)
    );
  }

  #[test]
  fn unrelated_sibling_root_cannot_hijack_a_fragment() {
    let tmp = tempfile::tempdir().unwrap();
    // A self-contained sibling that does NOT reference the fragment.
    write(
      tmp.path(),
      "other.tex",
      "\\documentclass{article}\\begin{document}standalone\\end{document}",
    );
    let frag = write(tmp.path(), "notes.tex", "just a fragment");
    let mut cache = RootCache::default();
    assert_eq!(
      resolve_root(&mut cache, None, &frag, Some("just a fragment")),
      normalize_path(&frag),
      "no referencing root -> fall back to the buffer"
    );
  }

  #[test]
  fn precedence_override_then_magic_then_class() {
    let tmp = tempfile::tempdir().unwrap();
    let main = write(tmp.path(), "main.tex", "\\documentclass{article}");
    let other = write(tmp.path(), "other.tex", "\\documentclass{article}");
    let ch = write(tmp.path(), "sections/ch2.tex", "x");
    let mut cache = RootCache::default();
    let magic_text = format!("%!TEX root = {}\nbody", other.display());
    // Magic comment wins over detection.
    assert_eq!(
      resolve_root(&mut cache, None, &ch, Some(&magic_text)),
      normalize_path(&other)
    );
    // Override wins over the magic comment.
    assert_eq!(
      resolve_root(&mut cache, Some(&main), &ch, Some(&magic_text)),
      normalize_path(&main)
    );
    // Self-contained buffer is its own root (no scan).
    assert_eq!(
      resolve_root(&mut cache, None, &other, Some("\\documentclass{article}")),
      normalize_path(&other)
    );
    // ... but a COMMENTED documentclass is not self-contained.
    assert!(!is_self_contained("% \\documentclass{article}\nfragment"));
  }

  #[test]
  fn resolve_falls_back_to_buffer() {
    let tmp = tempfile::tempdir().unwrap();
    let solo = write(tmp.path(), "solo.tex", "\\documentclass{article}");
    let mut cache = RootCache::default();
    assert_eq!(
      resolve_root(&mut cache, None, &solo, Some("\\documentclass{article}")),
      normalize_path(&solo)
    );
    // Unsaved buffer in an empty dir: fallback to the buffer path.
    let empty = tempfile::tempdir().unwrap();
    let ghost = empty.path().join("untitled.tex");
    assert_eq!(
      resolve_root(&mut cache, None, &ghost, Some("fragment")),
      normalize_path(&ghost)
    );
  }

  #[test]
  fn same_project_membership() {
    let root = Path::new("/proj/main.tex");
    assert!(same_project(root, Path::new("/proj/sections/ch2.tex")));
    assert!(same_project(root, Path::new("/proj/macros.sty")));
    assert!(!same_project(root, Path::new("/elsewhere/doc.tex")));
  }

  #[test]
  fn references_buffer_token_boundaries() {
    let buf = Path::new("/p/sections/ch2.tex");
    assert!(references_buffer("\\input{sections/ch2}", buf));
    assert!(references_buffer("\\include{ch2}", buf));
    assert!(!references_buffer("\\input{ch20}", buf), "ch2 must not match ch20");
    assert!(!references_buffer("% \\input{ch2}", buf), "commented include ignored");
    assert!(!references_buffer("plain ch2 mention without input", buf));
  }
}

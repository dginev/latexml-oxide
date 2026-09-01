//! The in-memory virtual file store — TeX's write-out/read-back round trip.
//!
//! Real TeX documents constantly write auxiliary files and read them back:
//! `\openout`/`\write`/`\closeout` streams, `{filecontents}` environments,
//! fancyvrb-style `{VerbatimOut}` captures, memoir `\writeverbatim`, and then
//! `\input`/`\openin`/`\verbatiminput`/`\IfFileExists` on the produced files.
//! Our conversion never touches the user's disk for these (the design
//! requirement allows writing only into the DESTINATION directory); instead
//! every virtual write lands in this store and every file read consults it
//! before kpathsea. The store is the single owner of the `VIRTUAL_FILES`
//! state key (Perl keeps one `<name>_contents` string value per file) — do
//! not reach into that map elsewhere.
//!
//! Contents live in ONE owned `HashMap<name, contents>` mutated in place,
//! NOT as interned strings: the interner never frees, so re-pinning the whole
//! file on every `\write` line (the earlier shape) grew the arena
//! quadratically in the line count — a `{VerbatimOut}` left unterminated at
//! end of input (fancyvrb `\FV@Scan` re-runs `\write` per empty read) walked
//! the interner's buffer offset past its `u32` range and aborted on a
//! `get_unchecked` precondition (witness: fancyvrb/fancyvrb-doc cut at the
//! open `{SideBySideExample}`, `eof1.tex` in the batch-48 guards).
//!
//! Scope is per-conversion GLOBAL state (Scope::Global), reset with the rest
//! of State between runs.

use rustc_hash::FxHashMap as HashMap;

use crate::{
  common::store::Stored,
  state::{Scope, assign_value, checkin_value, checkout_value, with_value},
};

/// The single state key holding the `name → contents` map.
const VFS_KEY: &str = "VIRTUAL_FILES";

/// Take the map out of State for in-place mutation (creating it on first use),
/// apply `f`, and put it back.
fn with_vfs_mut<R>(f: impl FnOnce(&mut HashMap<String, String>) -> R) -> R {
  let mut map = match checkout_value(VFS_KEY) {
    Some(Stored::HashString(map)) => map,
    Some(_) | None => {
      // Not yet assigned (or a foreign value under our key): establish the
      // map globally so a later checkin has a slot to land in.
      assign_value(
        VFS_KEY,
        Stored::HashString(HashMap::default()),
        Some(Scope::Global),
      );
      match checkout_value(VFS_KEY) {
        Some(Stored::HashString(map)) => map,
        _ => HashMap::default(),
      }
    },
  };
  let result = f(&mut map);
  checkin_value(VFS_KEY, Stored::HashString(map));
  result
}

/// Create/overwrite a virtual file with `content`.
pub fn vfs_store(name: &str, content: &str) {
  with_vfs_mut(|map| {
    map.insert(name.to_string(), content.to_string());
  });
}

/// Append one line (newline-terminated) to a virtual file, creating it if
/// absent — the `\write`-to-stream shape.
pub fn vfs_append_line(name: &str, line: &str) {
  with_vfs_mut(|map| {
    let contents = map.entry(name.to_string()).or_default();
    contents.push_str(line);
    contents.push('\n');
  });
}

/// Drop a virtual file, so later reads fall through to the real file system
/// (the LSP overlay retracts an editor buffer this way).
pub fn vfs_remove(name: &str) {
  with_vfs_mut(|map| {
    map.remove(name);
  });
}

/// Read a virtual file's full content.
pub fn vfs_read(name: &str) -> Option<String> {
  with_value(VFS_KEY, |v| match v {
    Some(Stored::HashString(map)) => map.get(name).cloned(),
    _ => None,
  })
}

/// Does a virtual file exist?
pub fn vfs_exists(name: &str) -> bool {
  with_value(VFS_KEY, |v| match v {
    Some(Stored::HashString(map)) => map.contains_key(name),
    _ => false,
  })
}

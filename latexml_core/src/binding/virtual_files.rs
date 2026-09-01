//! The in-memory virtual file store — TeX's write-out/read-back round trip.
//!
//! Real TeX documents constantly write auxiliary files and read them back:
//! `\openout`/`\write`/`\closeout` streams, `{filecontents}` environments,
//! fancyvrb-style `{VerbatimOut}` captures, memoir `\writeverbatim`, and then
//! `\input`/`\openin`/`\verbatiminput`/`\IfFileExists` on the produced files.
//! Our conversion never touches the user's disk for these (the design
//! requirement allows writing only into the DESTINATION directory); instead
//! every virtual write lands in this store and every file read consults it
//! before kpathsea. The store is the single owner of the `{name}_contents`
//! state-key convention — do not build those keys by hand elsewhere.
//!
//! Scope is per-conversion GLOBAL state (Scope::Global), reset with the rest
//! of State between runs.

use crate::{
  common::{
    arena::{pin, with},
    store::Stored,
  },
  state::{Scope, assign_value, lookup_value},
};

/// The state key under which a virtual file's content is stored.
fn key(name: &str) -> String { format!("{name}_contents") }

/// Create/overwrite a virtual file with `content`.
pub fn vfs_store(name: &str, content: &str) {
  assign_value(
    &key(name),
    Stored::String(pin(content)),
    Some(Scope::Global),
  );
}

/// Append one line (newline-terminated) to a virtual file, creating it if
/// absent — the `\write`-to-stream shape.
pub fn vfs_append_line(name: &str, line: &str) {
  let k = key(name);
  let mut contents = match lookup_value(&k) {
    Some(Stored::String(sym)) => with(sym, |v| v.to_string()),
    _ => String::new(),
  };
  contents.push_str(line);
  contents.push('\n');
  assign_value(&k, Stored::String(pin(&contents)), Some(Scope::Global));
}

/// Read a virtual file's full content.
pub fn vfs_read(name: &str) -> Option<String> {
  match lookup_value(&key(name)) {
    Some(Stored::String(sym)) => Some(with(sym, |v| v.to_string())),
    _ => None,
  }
}

/// Does a virtual file exist?
pub fn vfs_exists(name: &str) -> bool {
  matches!(lookup_value(&key(name)), Some(Stored::String(_)))
}

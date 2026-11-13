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

/// The VFS key for a file name: a leading `./` is dropped on EVERY side of
/// the store, the way one directory holds `./foo.tex` and `foo.tex`. The
/// read side (`find_file_aux`) stripped it while `\openout ./foo.tex` +
/// `\write` stored the raw name, so fancyvrb's `{VerbatimOut}{./x}` →
/// `\VerbatimInput{./x}` (xpicture-doc ×6, checklistings ×4; RUST-ONLY,
/// pdflatex and Perl clean) missed its own file. Guard:
/// `perfect_kernel_batch56::verbatimout_dotslash_round_trips_through_the_vfs`.
fn vfs_key(name: &str) -> &str { name.strip_prefix("./").unwrap_or(name) }

/// The name TeX writes an output file under. `\openout` completes a name
/// without an extension to `<name>.tex` (tex.web §1374 "if cur_ext="" then
/// cur_ext:=".tex"", tex.web:24928), and the extension is what follows the
/// LAST `.` after the last `/` (web2c's `more_name`); TeX Live reads a braced
/// name `{…}` as its group's content (`scan_file_name_braced`). Reads look for
/// `<name>.tex` first as well (§537), so `\openout\f=notes` + `\input{notes}`
/// meet in the store either way, while `\IfFileExists{notes.tex}` only finds
/// the completed name. Every writer that stands in for `\openout` — the
/// primitive, `{filecontents}` (latex.ltx:18998, :19023), expl3's
/// `\iow_open:Nn` for tcolorbox's `\tcbverbatimwrite`, listings'
/// `\lst@WFBegin` (lstmisc.sty:61) — names its file through this. Perl stores
/// the bare name (TeX_FileIO.pool.ltxml:120-126; KNOWN_PERL_ERRORS #279).
/// Witness latex4wp (latexdemo.sty:97-101 `{filecontents*}{democode}` then
/// `\IfFileExists{democode.tex}`: 98 of 105 words of its examples missing).
pub fn output_file_name(name: &str) -> String {
  let name = name
    .strip_prefix('{')
    .and_then(|inner| inner.strip_suffix('}'))
    .unwrap_or(name);
  let base = name.rsplit('/').next().unwrap_or(name);
  if base.is_empty() || base.contains('.') {
    name.to_string()
  } else {
    format!("{name}.tex")
  }
}

/// Create/overwrite a virtual file with `content`.
pub fn vfs_store(name: &str, content: &str) {
  with_vfs_mut(|map| {
    map.insert(vfs_key(name).to_string(), content.to_string());
  });
}

/// Append one line (newline-terminated) to a virtual file, creating it if
/// absent — the `\write`-to-stream shape.
pub fn vfs_append_line(name: &str, line: &str) {
  with_vfs_mut(|map| {
    let contents = map.entry(vfs_key(name).to_string()).or_default();
    contents.push_str(line);
    contents.push('\n');
  });
}

/// Drop a virtual file, so later reads fall through to the real file system
/// (the LSP overlay retracts an editor buffer this way).
pub fn vfs_remove(name: &str) {
  with_vfs_mut(|map| {
    map.remove(vfs_key(name));
  });
}

/// Read a virtual file's full content.
pub fn vfs_read(name: &str) -> Option<String> {
  with_value(VFS_KEY, |v| match v {
    Some(Stored::HashString(map)) => map.get(vfs_key(name)).cloned(),
    _ => None,
  })
}

/// Does a virtual file exist?
pub fn vfs_exists(name: &str) -> bool {
  with_value(VFS_KEY, |v| match v {
    Some(Stored::HashString(map)) => map.contains_key(vfs_key(name)),
    _ => false,
  })
}

#[cfg(test)]
mod tests {
  use super::output_file_name;

  /// tex.web §1374: only a name without an extension is completed, the
  /// extension being what follows the last `.` of the last path component.
  #[test]
  fn output_file_name_completes_as_openout_does() {
    assert_eq!(output_file_name("democode"), "democode.tex");
    assert_eq!(output_file_name("notes.aux"), "notes.aux");
    assert_eq!(output_file_name("my.notes.txt"), "my.notes.txt");
    assert_eq!(output_file_name("dir.d/notes"), "dir.d/notes.tex");
    assert_eq!(output_file_name("./notes"), "./notes.tex");
    assert_eq!(output_file_name("{notes}"), "notes.tex");
    assert_eq!(output_file_name(""), "");
  }
}

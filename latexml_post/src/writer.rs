//! XML/HTML output sink — port of `LaTeXML::Post::Writer`.
//!
//! Two related concerns live here:
//!
//! 1. The [`Writer`] post-processor (last in the chain) that serializes a `PostDocument` to its
//!    `destination`, handling DOCTYPE removal, TEMPORARY_DOCUMENT_ID cleanup, and HTML vs XML
//!    serialization.
//! 2. Free-standing helpers ([`write_output`], [`ensure_parent_dir`]) used by binary main()s that
//!    already have the serialized string in hand (post-processing returns a `String`) and need to
//!    route it to a destination path or stdout. Replaces the duplicated `File::create + write! +
//!    ensure_parent_dir` boilerplate that used to live in `latexml_oxide.rs` (and the now-retired
//!    `latexmlpost_oxide.rs`).
//!
//! Companion module: [`crate::pack`] (the `LaTeXML::Post::Pack` analog)
//! handles archive bundling when the destination is a zip.

use std::{
  fs,
  io::{self, Write},
  path::{Path, PathBuf},
  sync::atomic::{AtomicU64, Ordering},
};

use libxml::tree::{Node, SaveOptions};

use crate::{
  document::{PostDocument, get_xml_id},
  processor::{PostError, ProcessResult, Processor},
};

/// Write the serialized output `content` to `dest` if `Some`, else to
/// stdout. Creates parent directories as needed.
///
/// Used by `latexml_oxide.rs`'s main() (XML-input mode included) for
/// the "write a single HTML/XML file" exit path. For the zip-archive
/// exit path, use [`crate::pack::pack_archive`].
pub fn write_output(content: &str, dest: Option<&str>) -> io::Result<()> {
  match dest {
    Some(path) => {
      ensure_parent_dir(path)?;
      fs::write(path, content)?;
      Info!(
        "writer",
        "wrote",
        "Wrote '{}' ({} bytes)",
        path,
        content.len()
      );
      Ok(())
    },
    None => io::stdout().write_all(content.as_bytes()),
  }
}

/// Like [`write_output`] but writes several segments back-to-back without
/// concatenating them into one buffer first. Used for the conversion log,
/// where the core and post-phase segments are each already-allocated and
/// large for real articles — a `format!("{core}{post}")` would allocate a
/// third copy of their combined size on the conversion hot path. Segments are
/// written verbatim and in order through a single `BufWriter` (one file
/// open/truncate); insert any separators (e.g. `"\n"`) as their own segments.
pub fn write_output_segments(segments: &[&str], dest: Option<&str>) -> io::Result<()> {
  match dest {
    Some(path) => {
      ensure_parent_dir(path)?;
      let mut writer = io::BufWriter::new(fs::File::create(path)?);
      let mut total = 0usize;
      for seg in segments {
        writer.write_all(seg.as_bytes())?;
        total += seg.len();
      }
      writer.flush()?;
      Info!("writer", "wrote", "Wrote '{}' ({} bytes)", path, total);
      Ok(())
    },
    None => {
      let stdout = io::stdout();
      let mut handle = stdout.lock();
      for seg in segments {
        handle.write_all(seg.as_bytes())?;
      }
      Ok(())
    },
  }
}

/// Ensure the parent directory of `path` exists, creating it (and any
/// missing ancestors) as needed. No-op when `path` has no parent or
/// the parent is the current directory.
pub fn ensure_parent_dir(path: &str) -> io::Result<()> {
  if let Some(parent) = Path::new(path).parent() {
    if !parent.as_os_str().is_empty() {
      fs::create_dir_all(parent)?;
    }
  }
  Ok(())
}

/// The directories a page render already made: a split run writes thousands of pages into a handful of
/// directories, and each `create_dir_all` is a `mkdir` and a `statx` (Nasser's 71,476-page document,
/// #777 FAQ).
#[derive(Debug, Default)]
pub struct MadeDirs(std::collections::HashSet<PathBuf>);

std::thread_local! {
  static DIRS_MADE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// How many directories [`MadeDirs::ensure_parent`] asked the OS for on this thread (a test seam).
pub fn made_dir_count() -> usize { DIRS_MADE.get() }

impl MadeDirs {
  /// [`ensure_parent_dir`], once per directory; an error leaves the directory to be retried.
  pub fn ensure_parent(&mut self, path: &str) -> io::Result<()> {
    let Some(parent) = Path::new(path).parent() else {
      return Ok(());
    };
    if parent.as_os_str().is_empty() || self.0.contains(parent) {
      return Ok(());
    }
    DIRS_MADE.set(DIRS_MADE.get() + 1);
    fs::create_dir_all(parent)?;
    self.0.insert(parent.to_path_buf());
    Ok(())
  }
}

/// Monotonic per-process counter for unique sibling temp files ([`Staged::beside`]).
static ATOMIC_TMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// A private file beside a destination that parallel render workers may share (the CSS, a figure
/// every page uses): written or converted there, then [installed](Staged::install) by one rename,
/// so no reader sees a partial file and no writer touches another's. Named
/// `.<stem>.tmp.<pid>.<seq>.<ext>` (the stem [bounded](bounded_name)), its extension kept last
/// for converters that read the format from it; registered with the watchdog's exit sweep, and
/// removed when dropped — after an install too, since a rename onto a hard link of the same file
/// succeeds and leaves both names (POSIX `rename(2)`; a graphics-cache entry linked into the
/// destination by a sibling worker). A name some file already holds — a killed run's leftover,
/// perhaps a link to a cache entry, or a process with the same pid in another PID namespace
/// sharing the cache — is skipped, never written through.
pub struct Staged {
  path:     PathBuf,
  _on_exit: latexml_core::watchdog::RemoveOnExit,
}

impl Staged {
  /// A fresh private path beside `dest` that no file holds, for a converter to write (nothing
  /// is created).
  pub fn beside(dest: &Path) -> Self { Self::at(free_path(dest)) }

  /// [`Self::beside`], created empty by `create_new`: the name is this caller's alone.
  fn create(dest: &Path) -> io::Result<(Self, fs::File)> {
    loop {
      let path = free_path(dest);
      match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
      {
        Ok(file) => return Ok((Self::at(path), file)),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
        Err(e) => return Err(e),
      }
    }
  }

  /// [`Self::beside`], made a hard link to `src`; `None` where the filesystem can't link it here.
  fn link(src: &Path, dest: &Path) -> io::Result<Option<Self>> {
    loop {
      let path = free_path(dest);
      match fs::hard_link(src, &path) {
        Ok(()) => return Ok(Some(Self::at(path))),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
        // Another filesystem, or one without (more) hard links: the caller copies.
        Err(e)
          if matches!(
            e.kind(),
            io::ErrorKind::CrossesDevices
              | io::ErrorKind::PermissionDenied
              | io::ErrorKind::TooManyLinks
              | io::ErrorKind::Unsupported
          ) =>
        {
          return Ok(None);
        },
        Err(e) => return Err(e),
      }
    }
  }

  fn at(path: PathBuf) -> Self {
    let _on_exit = latexml_core::watchdog::remove_on_exit(&path);
    Staged { path, _on_exit }
  }

  pub fn path(&self) -> &Path { &self.path }

  /// Install the staged file as `dest` by rename. When the rename fails (Windows'
  /// `MoveFileExW` refuses a file another worker holds open) but `dest` already holds these very
  /// bytes, a sibling installed the same output and the install stands; the bytes are compared
  /// before the temp is removed, so a stale file of the same length is never taken for it.
  pub fn install(self, dest: &Path) -> io::Result<()> {
    match fs::rename(&self.path, dest) {
      Ok(()) => Ok(()),
      Err(_) if same_bytes(&self.path, dest) => Ok(()),
      Err(e) => Err(e),
    }
  }
}

/// Bytes of a file name [`bounded_name`] keeps.
const NAME_KEPT: usize = 64;

/// `name`, or for a longer one its first 64 bytes and a hash of the whole: every name derived from a
/// destination (its staged file, a stage of that, a converter's temp) then stays under the 255-byte
/// NAME_MAX, which a figure name of 220 characters or more took them past.
pub fn bounded_name(name: &str) -> std::borrow::Cow<'_, str> {
  use std::hash::{Hash, Hasher};
  if name.len() <= NAME_KEPT {
    return name.into();
  }
  let mut cut = NAME_KEPT;
  while !name.is_char_boundary(cut) {
    cut -= 1;
  }
  let mut hasher = std::hash::DefaultHasher::new();
  name.hash(&mut hasher);
  format!("{}~{:016x}", &name[..cut], hasher.finish()).into()
}

/// The staged name for `dest` with counter `seq`: `.<stem>.tmp.<pid>.<seq>.<ext>`, the stem
/// [bounded](bounded_name).
fn staged_path(dest: &Path, seq: u64) -> PathBuf {
  let pid = std::process::id();
  let stem = bounded_name(
    dest
      .file_stem()
      .and_then(|n| n.to_str())
      .unwrap_or("resource"),
  );
  let name = match dest.extension().and_then(|e| e.to_str()) {
    Some(ext) => format!(".{stem}.tmp.{pid}.{seq}.{ext}"),
    None => format!(".{stem}.tmp.{pid}.{seq}"),
  };
  match dest.parent() {
    Some(p) if !p.as_os_str().is_empty() => p.join(name),
    _ => PathBuf::from(name),
  }
}

/// The next [`staged_path`] for `dest` that names no file (a failing `lstat` other than "not
/// found" is left to the write that follows).
fn free_path(dest: &Path) -> PathBuf {
  loop {
    let path = staged_path(dest, ATOMIC_TMP_SEQ.fetch_add(1, Ordering::Relaxed));
    if fs::symlink_metadata(&path).is_err() {
      return path;
    }
  }
}

impl Drop for Staged {
  fn drop(&mut self) { let _ = fs::remove_file(&self.path); }
}

/// Whether `a` and `b` are regular files holding the same bytes.
pub fn same_bytes(a: &Path, b: &Path) -> bool {
  match (fs::metadata(a), fs::metadata(b)) {
    (Ok(ma), Ok(mb)) if ma.is_file() && mb.is_file() && ma.len() == mb.len() => {
      matches!((fs::read(a), fs::read(b)), (Ok(x), Ok(y)) if x == y)
    },
    _ => false,
  }
}

/// Whether `a` and `b` name the same existing file (by canonical path).
pub fn same_file(a: &Path, b: &Path) -> bool {
  match (fs::canonicalize(a), fs::canonicalize(b)) {
    (Ok(ca), Ok(cb)) => ca == cb,
    _ => false,
  }
}

/// Atomically write `bytes` to `dest` through a [`Staged`] file.
///
/// Safe when multiple parallel render workers publish the same shared resource
/// (`LaTeXML.css`, `LaTeXML.js`, etc.) concurrently: Unix readers never observe
/// a truncated/interleaved file, and Windows avoids `ERROR_SHARING_VIOLATION`
/// on concurrent writers.
pub fn write_atomic(dest: &Path, bytes: &[u8]) -> io::Result<()> {
  let (staged, mut file) = Staged::create(dest)?;
  file.write_all(bytes)?;
  drop(file);
  staged.install(dest)
}

/// Atomically copy `src` to `dest` through a [`Staged`] file. Copying a file onto itself is
/// refused: it would only rewrite it, and a caller that then changes `dest` would change its source.
pub fn copy_atomic(src: &Path, dest: &Path) -> io::Result<()> { place_atomic(src, dest, false) }

/// [`copy_atomic`] by a hard link where the filesystem allows one, for an immutable artefact
/// (a graphics-cache entry) whose readers replace it, never write through it.
pub fn link_atomic(src: &Path, dest: &Path) -> io::Result<()> { place_atomic(src, dest, true) }

fn place_atomic(src: &Path, dest: &Path, link: bool) -> io::Result<()> {
  if same_file(src, dest) {
    return Err(io::Error::other(format!(
      "{} is the source itself",
      dest.display()
    )));
  }
  if link && let Some(staged) = Staged::link(src, dest)? {
    return staged.install(dest);
  }
  let (staged, file) = Staged::create(dest)?;
  drop(file);
  fs::copy(src, staged.path())?;
  staged.install(dest)
}

/// Output format for the writer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
  Xml,
  Html,
}

/// Writer post-processor: serializes document to file.
///
/// Port of `LaTeXML::Post::Writer`.
pub struct Writer {
  name:         String,
  format:       OutputFormat,
  omit_doctype: bool,
  is_html:      bool,
}

impl Writer {
  pub fn new(format: Option<OutputFormat>, omit_doctype: bool, is_html: bool) -> Self {
    Writer {
      name: "Writer".to_string(),
      format: format.unwrap_or(OutputFormat::Xml),
      omit_doctype,
      is_html,
    }
  }
}

impl Processor for Writer {
  fn get_name(&self) -> &str { &self.name }

  fn to_process(&self, doc: &PostDocument) -> Vec<Node> {
    match doc.get_document_element() {
      Some(el) => vec![el],
      None => vec![],
    }
  }

  fn process(&mut self, mut doc: PostDocument, nodes: Vec<Node>) -> ProcessResult {
    let mut root = match nodes.into_iter().next() {
      Some(r) => r,
      None => return Ok(vec![doc]),
    };

    // Remove the internal DTD subset if requested (Perl Writer.pm L38:
    // `$doc->getDocument->removeInternalSubset if $$self{omit_doctype}`).
    // Backed by `libxml::tree::Document::remove_internal_subset`, added
    // in rust-libxml 0.3.11 specifically to close this gap.
    if self.omit_doctype {
      doc.get_document_mut().remove_internal_subset();
    }

    // Remove TEMPORARY_DOCUMENT_ID if present (Perl Writer.pm L41-42).
    // NS-aware forms: the bare get/remove_attribute("xml:id") pair silently
    // no-opped, so this last-chance strip never fired (the split path
    // removes the temp id earlier via core_interface, but this writer-side
    // guard is the one Perl relies on).
    if let Some(id) = get_xml_id(&root) {
      if id == "TEMPORARY_DOCUMENT_ID" {
        let _ = root.remove_attribute_ns("id", latexml_core::common::xml::XML_NS);
      }
    }

    // Serialize: HTML uses toStringHTML, XML uses toString(1)  (Perl Writer.pm L44-47)
    let serialized = if self.is_html {
      doc.get_document().to_string_with_options(SaveOptions {
        as_html: true,
        format: true,
        ..SaveOptions::default()
      })
    } else {
      doc.get_document().to_string_with_options(SaveOptions {
        format: true,
        ..SaveOptions::default()
      })
    };

    if let Some(destination) = doc.get_destination() {
      // Create destination directory if needed
      if let Some(destdir) = doc.get_destination_directory() {
        fs::create_dir_all(destdir).map_err(|e| {
          PostError::Io(io::Error::new(
            e.kind(),
            format!("Couldn't create directory '{}': {}", destdir, e),
          ))
        })?;
      }
      fs::write(destination, &serialized).map_err(|e| {
        PostError::Io(io::Error::new(
          e.kind(),
          format!("Couldn't write '{}': {}", destination, e),
        ))
      })?;
      Info!(
        "writer",
        "wrote",
        "Wrote '{}' ({})",
        destination,
        serialized.len()
      );
    } else {
      print!("{}", serialized);
    }

    Ok(vec![doc])
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn output_format_variants() {
    let _ = OutputFormat::Xml;
    let _ = OutputFormat::Html;
  }

  #[test]
  fn output_format_partial_eq() {
    assert_eq!(OutputFormat::Xml, OutputFormat::Xml);
    assert_ne!(OutputFormat::Xml, OutputFormat::Html);
    assert_eq!(OutputFormat::Html, OutputFormat::Html);
  }

  #[test]
  fn output_format_copy_clone() {
    // Copy trait: move-after-use still works.
    let a = OutputFormat::Xml;
    let b = a;
    let _ = a; // still usable due to Copy
    assert_eq!(a, b);
  }

  #[test]
  fn writer_new_default_format_xml() {
    let w = Writer::new(None, false, false);
    assert_eq!(w.get_name(), "Writer");
    assert_eq!(w.format, OutputFormat::Xml);
    assert!(!w.omit_doctype);
    assert!(!w.is_html);
  }

  #[test]
  fn writer_new_explicit_format() {
    let w = Writer::new(Some(OutputFormat::Html), true, true);
    assert_eq!(w.format, OutputFormat::Html);
    assert!(w.omit_doctype);
    assert!(w.is_html);
  }

  #[test]
  fn writer_get_name_is_writer() {
    let w = Writer::new(None, false, false);
    assert_eq!(w.get_name(), "Writer");
  }

  #[test]
  /// `Writer::process` with `omit_doctype = true` drops the
  /// `<!DOCTYPE …>` preamble from the output, mirroring Perl
  /// `Post::Writer` L38 behaviour.
  fn writer_omit_doctype_strips_doctype_preamble() {
    use crate::document::{PostDocument, PostDocumentOptions};

    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE root SYSTEM "example.dtd">
<root><child>hi</child></root>"#;
    let doc = PostDocument::new_from_string(xml, PostDocumentOptions::default())
      .expect("parse test fixture");

    // omit_doctype=true → DOCTYPE stripped after Writer::process.
    let mut writer = Writer::new(
      None, /* omit_doctype= */ true, /* is_html= */ false,
    );
    let to_process = writer.to_process(&doc);
    let result = writer.process(doc, to_process).expect("process");
    let after = result
      .into_iter()
      .next()
      .expect("at least one doc")
      .get_document()
      .to_string();
    assert!(
      !after.contains("<!DOCTYPE"),
      "expected DOCTYPE stripped, got: {after}"
    );
    assert!(after.contains("<root>"));
  }

  #[test]
  /// `Writer::process` with `omit_doctype = false` (the default)
  /// preserves the `<!DOCTYPE …>` preamble — opt-in behaviour.
  fn writer_default_preserves_doctype() {
    use crate::document::{PostDocument, PostDocumentOptions};

    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE root SYSTEM "example.dtd">
<root><child>hi</child></root>"#;
    let doc = PostDocument::new_from_string(xml, PostDocumentOptions::default())
      .expect("parse test fixture");

    let mut writer = Writer::new(
      None, /* omit_doctype= */ false, /* is_html= */ false,
    );
    let to_process = writer.to_process(&doc);
    let result = writer.process(doc, to_process).expect("process");
    let after = result
      .into_iter()
      .next()
      .expect("at least one doc")
      .get_document()
      .to_string();
    assert!(
      after.contains("<!DOCTYPE"),
      "expected DOCTYPE preserved, got: {after}"
    );
  }

  #[test]
  fn write_and_copy_atomic_succeed_and_clean_up_on_error() {
    let dir = tempfile::tempdir().unwrap();
    let dst = dir.path().join("LaTeXML.css");
    write_atomic(&dst, b"body { color: black; }\n").unwrap();
    assert_eq!(fs::read(&dst).unwrap(), b"body { color: black; }\n");

    let src = dir.path().join("custom.css");
    fs::write(&src, b".a { color: red; }\n").unwrap();
    copy_atomic(&src, &dst).unwrap();
    assert_eq!(fs::read(&dst).unwrap(), b".a { color: red; }\n");

    // Failing install onto a directory must return Err and leave no stray .tmp file.
    let bad = dir.path().join("blocked.css");
    fs::create_dir(&bad).unwrap();
    assert!(write_atomic(&bad, b"x").is_err());
    assert!(copy_atomic(&src, &bad).is_err());
    let strays: Vec<_> = fs::read_dir(dir.path())
      .unwrap()
      .filter_map(|e| e.ok())
      .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
      .collect();
    assert!(strays.is_empty(), "leftover atomic temp files: {strays:?}");
  }

  /// A failed rename stands only when the destination already holds the staged bytes — compared
  /// before the temp is removed, so a stale file of the same length is never taken for them
  /// (Windows' `MoveFileExW` refuses a file a sibling worker holds open). A read-only directory
  /// stands in for the refusal; a root run renames regardless and skips visibly.
  #[cfg(unix)]
  #[test]
  fn a_refused_install_stands_only_on_identical_bytes() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.svg");
    let b = dir.path().join("b.svg");
    fs::write(&a, b"<svg>1</svg>").unwrap();
    fs::write(&b, b"<svg>2</svg>").unwrap();
    assert!(same_bytes(&a, &a.clone()));
    assert!(!same_bytes(&a, &b), "same length, other bytes");
    assert!(!same_bytes(&a, &dir.path().join("missing.svg")));
    assert!(
      !same_bytes(dir.path(), dir.path()),
      "a directory holds no bytes"
    );

    let stage = |bytes: &[u8]| {
      let staged = Staged::beside(&a);
      fs::write(staged.path(), bytes).unwrap();
      staged
    };
    let (same, other) = (stage(b"<svg>1</svg>"), stage(b"<svg>9</svg>"));
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let probe = fs::rename(&b, dir.path().join("probe.svg"));
    let refused = probe.is_err();
    let (same, other) = (same.install(&a), other.install(&a));
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
    if !refused {
      eprintln!(
        "SKIP a_refused_install_stands_only_on_identical_bytes: renames are not refused (root)"
      );
      return;
    }
    assert!(same.is_ok(), "{same:?}");
    assert!(
      other.is_err(),
      "a stale file of the same length is not the install"
    );
    assert_eq!(fs::read(&a).unwrap(), b"<svg>1</svg>");
    let staged = Staged::beside(&a);
    let temp = staged.path().to_path_buf();
    fs::write(&temp, b"x").unwrap();
    drop(staged);
    assert!(
      !temp.exists(),
      "an uninstalled stage is removed when dropped"
    );
  }

  /// A rename onto a hard link of the same file succeeds and keeps both names (POSIX): a
  /// graphics-cache entry a sibling worker linked into the destination left every later worker's
  /// staged link behind.
  #[test]
  fn a_stage_installed_onto_its_own_hard_link_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("fig.svg");
    fs::write(&a, b"<svg/>").unwrap();
    let staged = Staged::beside(&a);
    let temp = staged.path().to_path_buf();
    fs::hard_link(&a, &temp).unwrap();
    staged.install(&a).unwrap();
    assert!(!temp.exists(), "the staged link is left behind");
    assert_eq!(fs::read(&a).unwrap(), b"<svg/>");
  }

  /// A 240-character figure name stays under NAME_MAX (255 bytes) through its stage and a stage of
  /// that stage (a crop's `write_atomic` of a staged render), the extension kept last; two names
  /// that differ only past the kept bytes stage apart.
  #[test]
  fn the_stages_of_a_long_figure_name_stay_under_name_max() {
    let dir = tempfile::tempdir().unwrap();
    let long = dir.path().join(format!("{}.svg", "g".repeat(240)));
    let stage = Staged::beside(&long);
    let again = Staged::beside(stage.path());
    for path in [stage.path(), again.path()] {
      let name = path.file_name().unwrap().to_str().unwrap();
      assert!(name.len() <= 128, "{} bytes: {name}", name.len());
      assert!(name.ends_with(".svg"), "{name}");
      fs::write(path, b"<svg/>").unwrap();
    }
    write_atomic(&long, b"<svg>1</svg>").unwrap();
    assert_eq!(fs::read(&long).unwrap(), b"<svg>1</svg>");
    let (a, b) = (
      format!("{}a", "é".repeat(40)),
      format!("{}b", "é".repeat(40)),
    );
    assert_ne!(bounded_name(&a), bounded_name(&b));
    assert_eq!(bounded_name("fig.svg"), "fig.svg", "a short name is kept");
  }

  /// A temp name already taken — a killed run's leftover, perhaps a hard link to a graphics-cache
  /// entry, or a process with the same pid in another PID namespace sharing the cache — is never
  /// written through: `write_atomic`, `link_atomic` and `copy_atomic` take the next name.
  #[test]
  fn a_taken_temp_name_is_never_written_through() {
    let dir = tempfile::tempdir().unwrap();
    let sentinel = dir.path().join("site.svg");
    fs::write(&sentinel, b"<svg>site</svg>").unwrap();
    let src = dir.path().join("cached.svg");
    fs::write(&src, b"<svg>new</svg>").unwrap();
    let dests = ["w.svg", "l.svg", "c.svg"].map(|name| dir.path().join(name));
    let next = ATOMIC_TMP_SEQ.load(Ordering::Relaxed);
    for dest in &dests {
      for seq in next..next + 64 {
        fs::hard_link(&sentinel, staged_path(dest, seq)).unwrap();
      }
    }
    write_atomic(&dests[0], b"<svg>new</svg>").unwrap();
    link_atomic(&src, &dests[1]).unwrap();
    copy_atomic(&src, &dests[2]).unwrap();
    assert_eq!(fs::read(&sentinel).unwrap(), b"<svg>site</svg>");
    for dest in &dests {
      assert_eq!(
        fs::read(dest).unwrap(),
        b"<svg>new</svg>",
        "{}",
        dest.display()
      );
    }
  }

  #[test]
  fn concurrent_write_atomic_produces_intact_file_without_errors() {
    let dir = tempfile::tempdir().unwrap();
    let dst = dir.path().join("shared.css");
    let payload: Vec<u8> = b"/* css */\n".repeat(4096);
    std::thread::scope(|s| {
      for _ in 0..8 {
        let dst = &dst;
        let payload = &payload;
        s.spawn(move || {
          for _ in 0..16 {
            write_atomic(dst, payload).expect("concurrent atomic write");
          }
        });
      }
    });
    assert_eq!(fs::read(&dst).unwrap(), payload);
  }
}

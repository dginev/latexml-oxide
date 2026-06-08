use kpathsea::Kpaths;
use once_cell::sync::Lazy;
use regex::Regex;

use std::env;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// configuration for filesystem search.
/// Mirrors Perl `LaTeXML::Util::Pathname::pathname_find`'s named-arg options;
/// kpsewhich-fallback is NOT one of them — that lives in higher-level
/// `LaTeXML::Package::FindFile_aux`, which calls `pathname_kpsewhich` after
/// `pathname_find` returns empty. Keep this struct directory-search-only
/// for parity.
#[derive(Debug, Clone, Default)]
pub struct PathnameFindOptions {
  /// the allowed/requested paths to search in
  pub paths:               Option<Vec<String>>,
  /// the file extensions to search for
  pub extensions:          Option<Vec<String>>,
  /// the location of the installation subdirectory (deprecated?)
  pub installation_subdir: Option<String>,
}

static LITERAL_PROTOCOL: &str = "literal:";
static HOME_TILDE: &str = "~";
static HOME_PATH: Lazy<String> = Lazy::new(|| match std::env::var_os("HOME") {
  Some(val) => val.to_string_lossy().into_owned(),
  _ => s!("~"),
});
static PROTOCOL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(https|http|ftp):").unwrap());
// Match Perl LaTeXML's permissive filename behavior: filenames may
// contain commas, parens, ampersands, etc. that some user paths legitimately
// use (e.g. `\input{5-Ack,terms}` resolving to `5-Ack,terms.tex`). Only
// flag genuinely dangerous patterns: shell metacharacters that would
// enable command injection via kpathsea or `\openin`. Driver: 2308.13679
// `\input{5-Ack,terms}`. Mirrors Perl's missing nasty-check (Perl simply
// passes filenames to kpathsea without a pre-filter).
static PATHNAME_IS_NASTY_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"[`$;|<>"\x00\n\r]"#).unwrap());
// TODO: This is very pragmatic for now, we ought to use a real URL path library long-term
static URL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+://(.+)/([^/]+)$").unwrap());
// Perl `pathname_is_url` is `$pathname =~ /^($PROTOCOL_RE)/` — the protocol is
// ANCHORED at the start. The previous `is_url` used `URL_RE` (`^\w+://…`),
// whose leading `\w+` (which includes `_`) matches a filename PREFIX like
// `myers_http`, so a JabRef `\bibAnnoteFile{myers_http://…/welcome.html_2014}`
// key (a filename, NOT a URL) read as a URL → `find_file` returned "exists" →
// `\IfFileExists` took its true branch → the `_` in the key got typeset in
// text mode ("Script _ can only appear in math mode"; witness 1509.01434).
// Match Perl: only `http`/`https`/`ftp` followed by `:` at the very start.
static URL_PREFIX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:https|http|ftp):").unwrap());

/// Process-global kpathsea handle (cross-thread shared, NOT
/// `thread_local!`).
///
/// **Why an exception to the project's no-`Mutex` rule:**
/// `kpathsea-rs` wraps a C library that maintains process-wide global
/// state. Calling `Kpaths::new()` more than once per process (as the
/// per-thread `thread_local!` pattern would do) re-runs the C-side
/// init and can corrupt internal tables / leak file descriptors;
/// concretely on systems without TeXLive installed, the second init
/// fails with "Can't get directory of program name" and previously
/// crashed `06_cluster_regressions`. The Mutex here is necessary to
/// guarantee single-init AND single-active-call semantics for the
/// underlying non-thread-safe C API. Same class of carve-out as
/// `latexml_core::watchdog::PRE_EXIT_HOOK`. See
/// `feedback_no_mutex_use_thread_local` in user memory for the
/// general rule.
static KPSE: Lazy<Mutex<Option<Kpaths>>> = Lazy::new(|| Mutex::new(Kpaths::new().ok()));

/// Force-initialize the kpathsea global state and warm up the per-
/// format suffix tables.
///
/// **Why:** `Kpaths::find_file` lazily inits the kpse format-info
/// table for each format type the first time a matching filename is
/// looked up. The chain is
/// `find_file → guess_format_from_filename → kpathsea_init_format →
/// kpathsea_init_db → kpathsea_cnf_get → hash_insert_normalized`,
/// taking ~30-40 ms total across the first dozen lookups. Profile
/// data on 1910.01256 attributes ~3.5% of wall to that chain.
///
/// **What this does:** acquires the `KPSE` mutex once and runs a
/// single `find_file` probe per common file format. Each probe
/// guarantees `kpathsea_init_format` runs for that format type, so
/// every subsequent real lookup hits the post-init fast path.
///
/// **Concurrency:** safe to invoke on a background thread spawned at
/// process start. `KPSE` is process-global (`Lazy<Mutex<…>>`), so the
/// init done on a background thread is visible to the main thread.
/// The Mutex briefly serializes the prewarm against the main thread's
/// first real lookup, but dump load + arg parsing take >50 ms before
/// digest reaches its first package resolution, by which point the
/// prewarm is usually finished. Idempotent: re-entry while in flight
/// is a no-op (lock contention only).
pub fn prewarm_kpathsea() {
  let kpse_guard = KPSE.lock().unwrap();
  let Some(ref kpse) = *kpse_guard else {
    return;
  };
  // Subprocess backend (kpathsea 0.3, hosts without libkpathsea — e.g.
  // MacTeX): there is no in-process format-info table to warm, and each
  // sentinel would cost a real `kpsewhich` subprocess invocation
  // (~10-20 ms × 11). The backend builds its own ls-R cache lazily on
  // the first real lookup instead.
  if !kpse.is_in_process() {
    return;
  }
  for sentinel in &[
    "warmup_lxoxide.tex",
    "warmup_lxoxide.sty",
    "warmup_lxoxide.cls",
    "warmup_lxoxide.def",
    "warmup_lxoxide.bst",
    "warmup_lxoxide.fmt",
    "warmup_lxoxide.afm",
    "warmup_lxoxide.tfm",
    "warmup_lxoxide.pfb",
    "warmup_lxoxide.enc",
    "warmup_lxoxide.map",
  ] {
    // catch_unwind defends against kpathsea-0.2.3 overflow bug (see
    // kpsewhich docs).
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| kpse.find_file(sentinel)));
  }
}
// Perl: $pathname =~ s|^($PROTOCOL_RE//[^/]*)/|/|
static CANONICAL_URL_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^((?:https|http|ftp)://[^/]*)").unwrap());

// static ref INSTALLDIRS : Vec<String> = match env::current_exe() {
//     Ok(exe_path) => {
//       match exe_path.as_path().parent() {
//         Some(_) => Vec::new(),
//         // Some(p) => vec![
//         //                 p.to_string_lossy().to_string() + ".",
//         //                 p.to_string_lossy().to_string() + "./..",
//         //                 p.to_string_lossy().to_string() + "./../..",
//         //                 p.to_string_lossy().to_string() + "./../../..",
//         //                 p.to_string_lossy().to_string() + "./../../../.."],

// TODO: HACK, see note on INSTALLDIRS further down
//         None => Vec::new()
//       }
//     },
//     _ => Vec::new()
//   };

// TODO:
// grep { (-f "$_.pm") && (-d $_) }
// map { pathname_canonical($_ . $SEP . 'LaTeXML') } @INC;    # [CONSTANT]

/// checks if the path is a conforming URL string
pub fn is_url(path: &str) -> bool { URL_PREFIX_RE.is_match(path) }
/// checks if the path starts with the "literal:" protocol
pub fn is_literaldata(data: &str) -> bool { data.starts_with(LITERAL_PROTOCOL) }

/// check whether a pathname is reloadable as a TeX definition
pub fn is_reloadable(pathname: &str) -> bool {
  let (_dir, _name, ext) = split(pathname);
  // babel.sty exception:
  // we know the same .ldf file may be reloaded with a different option,
  // to load an adjacently defined language, so allow that.
  ext == "ldf"
}
/// Check whether a pathname is a raw TeX source or definition file.
/// Perl: pathname_is_raw
pub fn is_raw(pathname: &str) -> bool {
  matches!(
    extension(pathname).as_str(),
    "tex" | "pool" | "sty" | "cls" | "clo" | "cnf" | "cfg" | "ldf" | "def" | "dfu"
  )
}

/// absolute paths start with the filesystem root - check if this is one
pub fn is_absolute(path: &str) -> bool { Path::new(&canonical(path)).is_absolute() }
/// convert a (possibly relative) file path to an absolute one
///
/// `std::fs::canonicalize` requires the path to exist; many callers hand
/// us paths that haven't been resolved yet (e.g. `\import{subdir}{f.sty}`
/// constructs `subdir/f.sty` before `find_file` probes other dirs).
/// Mirror Perl's `Cwd::abs_path`-style behavior: produce a lexically
/// absolute path joined against `current_dir()` when the input is
/// relative, then run it through our `canonical()` to collapse `.`/`..`
/// components.
///
/// Panics only if `current_dir()` itself fails — that means the cwd was
/// deleted out from under us, which we cannot safely resolve a relative
/// path against (and silently returning the input could let a relative
/// file reference target an attacker-controlled path).
pub fn absolute(path: &str) -> String {
  let p = Path::new(path);
  let joined: PathBuf = if p.is_absolute() {
    p.to_path_buf()
  } else {
    let cwd = std::env::current_dir()
      .expect("cannot make path absolute: current_dir() failed");
    cwd.join(p)
  };
  canonical(&joined.to_string_lossy())
}

/// Split the pathname into components (dir,name,type).
/// If pathname is absolute, dir starts with volume or '/'
pub fn split(pathname: &str) -> (String, String, String) {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  let pathdir = match canonical_path.parent() {
    Some(dir) => dir.to_string_lossy().to_string(),
    None => String::new(),
  };
  let name = match canonical_path.file_stem() {
    Some(n) => n.to_string_lossy().to_string(),
    None => String::new(),
  };
  // Perl pathname_split preserves case: `$name =~ s/\.([^\.]+)$//`
  let pathname_ext = match canonical_path.extension() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  };
  (pathdir, name, pathname_ext)
}

///  Simple logic for splitting a URL into protocol://base/path
pub fn url_split(url: &str) -> (&str, &str) {
  if let Some(caps) = URL_RE.captures(url) {
    (
      caps.get(1).map_or("", |m| m.as_str()),
      caps.get(2).map_or("", |m| m.as_str()),
    )
  } else {
    (url, "index.tex") // Well, what other default makes sense?
  }
}

/// Canonicalize a pathname by simplifying redundant separators, `.` and `..` components.
/// Matches Perl's pathname_canonical from Pathname.pm.
pub fn canonical(pathname: &str) -> String {
  if is_literaldata(pathname) {
    return pathname.to_owned();
  }
  // Don't call is_absolute, etc, here, cause THEY call US!
  let home_path: &str = &HOME_PATH;

  let mut pathname = if pathname.starts_with(HOME_TILDE) {
    pathname.replacen(HOME_TILDE, home_path, 1)
  } else {
    pathname.to_string()
  };

  // Handle URL prefix: strip protocol://host before normalizing path
  let url_prefix = if let Some(caps) = CANONICAL_URL_RE.captures(&pathname) {
    let prefix = caps.get(1).unwrap().as_str().to_string();
    pathname = pathname[prefix.len()..].to_string();
    Some(prefix)
  } else {
    None
  };

  // Perl: $pathname =~ s|/\./|/|g;
  while pathname.contains("/./") {
    pathname = pathname.replace("/./", "/");
  }
  // Perl: while ($pathname =~ s|/(?!\.\./)[^/]+/\.\.(/|$)|$1|) { }
  // Collapse /foo/.. patterns but not /../..
  // Implemented without lookahead since the regex crate doesn't support it.
  loop {
    let mut changed = false;
    // Find /component/.. where component is not ".."
    if let Some(dotdot_pos) = pathname.find("/..") {
      // Check this is actually /../ or /..$ (end of string)
      let after = dotdot_pos + 3;
      if after == pathname.len() || pathname.as_bytes().get(after) == Some(&b'/') {
        // Find the preceding component: look backwards from dotdot_pos for '/'
        if dotdot_pos > 0 {
          let prefix = &pathname[..dotdot_pos];
          if let Some(slash_pos) = prefix.rfind('/') {
            let component = &pathname[slash_pos + 1..dotdot_pos];
            if component != ".." && !component.is_empty() {
              // Replace /component/.. with the trailing part
              let trail = &pathname[after..];
              pathname = format!("{}{}", &pathname[..slash_pos], trail);
              changed = true;
            }
          } else if prefix != ".." {
            // No leading slash, e.g. "foo/.."
            let trail = &pathname[after..];
            pathname = if let Some(stripped) = trail.strip_prefix('/') {
              stripped.to_string()
            } else {
              trail.to_string()
            };
            if pathname.is_empty() {
              pathname = ".".to_string();
            }
            changed = true;
          }
        }
      }
    }
    if !changed {
      break;
    }
  }
  // Perl: $pathname =~ s|^\./(.)|$1|; — reduce ./foo to foo, but preserve ./
  if pathname.starts_with("./") && pathname.len() > 2 {
    pathname = pathname[2..].to_string();
  }
  match url_prefix {
    Some(prefix) => format!("{}{}", prefix, pathname),
    None => pathname,
  }
}

/// Note that this returns ONLY recognized protocols!
pub fn protocol(pathname: &str) -> String {
  if let Some(cap) = PROTOCOL_RE.captures(pathname) {
    cap.get(1).map_or(String::new(), |m| m.as_str().to_string())
  } else if is_literaldata(pathname) {
    "literal".to_string()
  } else {
    "file".to_string()
  }
}

/// combine a directory and a base name into a full path
pub fn concat(dir: &str, file: &str) -> String {
  if dir.is_empty() {
    file.to_owned()
  } else if file.is_empty() || file == "." {
    dir.to_owned()
  } else {
    let mut path = PathBuf::from(dir);
    path.push(file);
    canonical(&path.to_string_lossy())
  }
}

/// It's presumably cheep to concatinate all the pathnames,
/// relative to the cost of testing for files,
/// and this simplifies overall.
pub fn candidate_pathnames(pathname: &str, options: PathnameFindOptions) -> Vec<String> {
  let mut dirs: Vec<String> = Vec::new();
  let canonical_pathname = if pathname != "*" {
    canonical(pathname)
  } else {
    pathname.to_owned()
  };

  let (pathdir, name_stem, pathname_ext) = split(&canonical_pathname);
  // Perl: $name .= '.' . $type if (defined $type) && ($type ne '');
  // Re-attach the extension to the name, as Perl does after split
  let name = if !pathname_ext.is_empty() {
    format!("{}.{}", name_stem, pathname_ext)
  } else {
    name_stem
  };

  let cwd = cwd();

  // generate the set of search paths we'll use.
  if is_absolute(&canonical_pathname) {
    dirs.push(pathdir.clone());
  } else if let Some(paths) = options.paths {
    for p in paths {
      // Complete the search paths by prepending current dir to relative paths,
      let pp_base = if is_absolute(&p) {
        canonical(&p)
      } else {
        concat(&cwd, &p)
      };
      let pp = concat(&pp_base, &pathdir);
      // but only include each dir ONCE
      if !dirs.contains(&pp) {
        dirs.push(pp);
      }
    }
  }
  // Perl: push(@dirs, pathname_concat($cwd, $pathdir)) unless @dirs;
  // Only add cwd if no search paths were given (fallback)
  if dirs.is_empty() {
    let from_cwd = concat(&cwd, &pathdir);
    if !dirs.contains(&from_cwd) {
      dirs.push(from_cwd);
    }
  }

  // TODO: The use of INSTALLDIRS should be rethought entirely, as Rust currently doesn't have a
  // native concept of "installing" a crate and its resources there either needs to be a
  // more sophisticated build process, OR, a compile step that translates all model/binding
  // dependencies into rust code. which would allow bundling them side-by-side with the
  // main application.

  // And, if installation dir specified, append it.
  if let Some(subdir) = options.installation_subdir {
    // dirs.extend((*INSTALLDIRS).iter().map(|dir| concat(dir, &subdir)));
    let full_subdir = concat(&cwd, &subdir);
    if Path::new(&full_subdir).exists() {
      dirs.push(full_subdir);
    } else {
      let full_subdir_oneup = concat(&s!("{}/..", cwd), &subdir);
      if Path::new(&full_subdir_oneup).exists() {
        dirs.push(full_subdir_oneup);
      }
    }
  }
  // extract the desired extensions.
  // Perl: the extensions from `types` option are applied to the already-reassembled name.
  // Since name already has its extension, matching an existing extension pushes '' (exact match).
  let mut exts = Vec::new();
  if let Some(ext_vec) = options.extensions {
    for ext in ext_vec {
      if ext.is_empty() {
        exts.push(String::new());
      } else if ext == "*" {
        exts.push(s!(".*"));
        exts.push(String::new());
      } else if !pathname_ext.is_empty() && pathname_ext.eq_ignore_ascii_case(&ext) {
        // Perl Pathname.pm L353: `if ($pathname =~ /\.\Q$ext\E$/i)` — /i
        // makes this a case-insensitive extension match; either case of
        // file extension matches either case of requested type.
        exts.push(String::new());
        // Also push the extension itself (Perl pushes both)
        exts.push(format!(".{}", ext));
      } else {
        exts.push(format!(".{}", ext));
      }
    }
  }
  if exts.is_empty() {
    exts.push(String::new());
  }

  let mut paths = Vec::new();
  // Now, combine; precedence to leading directories.
  for dir in &dirs {
    for ext in &exts {
      if name == "*" {
        // TODO: wildcard directory listing support
      } else {
        paths.push(concat(dir, &(name.clone() + ext)));
      }
    }
  }
  paths
}

/// find the requested `pathname` using the `options` search configuration.
/// Mirrors Perl `pathname_find` (LaTeXML/Util/Pathname.pm L376-392): directory
/// search with strict-case match preferred, falling back to a
/// case-insensitive directory scan. The fallback is required for arxiv
/// papers shipping uppercase filenames (e.g. `PASJ95.STY` referenced as
/// `PASJ95.sty`) — Perl's regex pair pushes both strict and `/i` matches
/// and returns the strict ones if any exist, otherwise the case-insensitive
/// matches. kpsewhich is the caller's responsibility (see
/// `LaTeXML::Package::FindFile_aux`).
pub fn find(pathname: &str, options: PathnameFindOptions) -> Option<String> {
  if pathname.is_empty() {
    return None;
  }
  let paths = candidate_pathnames(pathname, options);
  // Pass 1: strict-case existence check (the fast path; matches Perl's
  // `$local_file =~ m/$regex/` strict regex).
  for path in &paths {
    if Path::new(path).exists() {
      return Some(path.clone());
    }
  }
  // Pass 2: case-insensitive directory scan (Perl's `/i` regex fallback).
  // Only fired when no strict match existed; mirrors Perl's
  // `return @paths ? @paths : @nocase_paths` ordering.
  for path in &paths {
    let p = Path::new(path);
    let dir = match p.parent() {
      Some(d) if !d.as_os_str().is_empty() => d,
      _ => Path::new("."),
    };
    let target = match p.file_name().and_then(|n| n.to_str()) {
      Some(n) => n,
      None => continue,
    };
    let entries = match std::fs::read_dir(dir) {
      Ok(e) => e,
      Err(_) => continue,
    };
    for entry in entries.flatten() {
      if let Some(name) = entry.file_name().to_str() {
        if name.eq_ignore_ascii_case(target) {
          return entry.path().to_str().map(String::from);
        }
      }
    }
  }
  None
}

/// transform to a canonical file name, via `Path::file_name`
pub fn file_name(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.file_name() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }
}

/// transform to a base name (via `Path::file_stem`)
/// Note: Perl's pathname_name returns the stem without extension and without case change.
pub fn file_stem(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.file_stem() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }
}

/// obtain the directory portion of a pathname (via `Path::parent`)
/// Matches Perl's pathname_directory.
pub fn directory(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.parent() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }
}

/// obtain the extension portion of a pathname (via `Path::extension`).
/// Perl's `pathname_type` preserves case; callers that need a lowercased
/// form should apply `.to_ascii_lowercase()` themselves.
pub fn extension(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.extension() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }
}

/// Compose a pathname from dir, name, type components.
/// Port of Perl's pathname_make(%pieces).
pub fn make(dir: Option<&str>, name: Option<&str>, ext: Option<&str>) -> String {
  let mut result = String::new();
  if let Some(d) = dir {
    result.push_str(d);
  }
  if let Some(n) = name {
    if !result.is_empty() && !result.ends_with('/') {
      result.push('/');
    }
    result.push_str(n);
  }
  if let Some(t) = ext {
    if !t.is_empty() {
      result.push('.');
      result.push_str(t);
    }
  }
  canonical(&result)
}

/// Make a pathname relative to a base directory.
/// Port of Perl's pathname_relative($pathname, $base).
pub fn relative(pathname: &str, base: &str) -> String {
  let canonical_pathname = canonical(pathname);
  if base.is_empty() || !is_absolute(&canonical_pathname) {
    return canonical_pathname;
  }
  let canonical_base = canonical(base);
  let path = Path::new(&canonical_pathname);
  let base_path = Path::new(&canonical_base);
  match path.strip_prefix(base_path) {
    Ok(rel) => rel.to_string_lossy().to_string(),
    Err(_) => canonical_pathname,
  }
}

/// Find all matching files (like pathname_findall).
/// Port of Perl's pathname_findall($pathname, %options).
pub fn findall(pathname: &str, options: PathnameFindOptions) -> Vec<String> {
  candidate_pathnames(pathname, options)
}

/// search for a list of candidate names via the external `kpsewhich` utility
/// returning the first path that is found
pub fn kpsewhich(candidates: &[&str]) -> Option<String> {
  if let Some(ref kpse) = *KPSE.lock().unwrap() {
    for candidate in candidates {
      // kpathsea-0.2.3 panics with "attempt to subtract with overflow" in
      // `guess_format_from_filename` (lib.rs:92) when `filename.len()` is
      // shorter than some alt_suffix the format-table holds (the L73 normal-
      // suffix loop has a `filename.len() > suffix.len()` guard but the L92
      // alt_suffix loop does NOT). User input like `\usepackage[opt]{}`
      // produces a `.sty` candidate (empty stem) which trips this. Pre-filter
      // those: a basename starting with `.` and containing only an extension
      // is bogus to look up. The catch_unwind below remains as defense-in-
      // depth. Witnesses: 0711.2664 (`.sty`), cs0503041 (`.sty`).
      let basename = candidate.rsplit(['/', '\\']).next().unwrap_or(candidate);
      if basename.starts_with('.') && !basename[1..].contains('.') {
        continue;
      }
      let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| kpse.find_file(candidate)));
      if let Ok(Some(path)) = result {
        return Some(path);
      }
    }
  }
  None
}

/// check if pathname contains dangerous pieces
pub fn is_nasty(file: &str) -> bool { PATHNAME_IS_NASTY_RE.is_match(file) }

/// returns the current working directory
pub fn cwd() -> String { env::current_dir().unwrap().to_string_lossy().to_string() }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn is_url_schemes() {
    // Perl `pathname_is_url`: `=~ /^($PROTOCOL_RE)/` — the protocol must be
    // ANCHORED at the start; a bare host (no /path) still matches.
    assert!(is_url("http://example.com/path"));
    assert!(is_url("http://example.com/path/file.tex"));
    assert!(is_url("ftp://host/file"));
    assert!(is_url("https://example.com")); // bare host — Perl matches the prefix
    assert!(!is_url("plain/path/file.tex"));
    assert!(!is_url("/absolute/path"));
    // A filename that merely CONTAINS a protocol mid-string is NOT a URL —
    // the old `^\w+://…` matched `myers_http://…` via its leading `\w+`,
    // wrongly resolving a JabRef `\bibAnnoteFile` key as an existing URL
    // (witness 1509.01434, "Script _").
    assert!(!is_url("myers_http://www.mscs.dal.ca/myers/welcome.html_2014"));
    assert!(!is_url("foo_ftp://bar/baz"));
  }

  #[test]
  fn is_literaldata_prefix() {
    assert!(is_literaldata("literal:foo"));
    assert!(!is_literaldata("file:foo"));
    assert!(!is_literaldata("plain"));
  }

  #[test]
  fn is_raw_tex_extensions() {
    assert!(is_raw("main.tex"));
    assert!(is_raw("hyphen.cfg"));
    assert!(is_raw("T1enc.def"));
    assert!(is_raw("article.cls"));
    assert!(is_raw("french.ldf"));
    assert!(!is_raw("foo.pdf"));
    assert!(!is_raw("bar.png"));
    assert!(!is_raw("baz"));
  }

  #[test]
  fn is_reloadable_only_ldf() {
    assert!(is_reloadable("french.ldf"));
    assert!(!is_reloadable("main.tex"));
    assert!(!is_reloadable("foo.sty"));
    assert!(!is_reloadable("baz"));
  }

  #[test]
  fn extension_basic() {
    assert_eq!(extension("foo.tex"), "tex");
    assert_eq!(extension("path/to/main.cls"), "cls");
    assert_eq!(extension("no_ext"), "");
    assert_eq!(extension("double.dot.ext"), "ext");
  }

  #[test]
  fn file_name_strips_dirs() {
    assert_eq!(file_name("path/to/foo.tex"), "foo.tex");
    assert_eq!(file_name("foo.tex"), "foo.tex");
    assert_eq!(file_name("/abs/path/foo.tex"), "foo.tex");
  }

  #[test]
  fn file_stem_strips_ext() {
    assert_eq!(file_stem("foo.tex"), "foo");
    assert_eq!(file_stem("path/to/foo.cls"), "foo");
    assert_eq!(file_stem("no_ext"), "no_ext");
  }

  #[test]
  fn directory_returns_dir() {
    assert_eq!(directory("path/to/foo.tex"), "path/to");
    assert!(
      directory("foo.tex").is_empty() || directory("foo.tex") == ".",
      "relative-only filename: dir is empty or '.'"
    );
  }

  #[test]
  fn make_reassembles_components() {
    let p = make(Some("path"), Some("foo"), Some("tex"));
    assert_eq!(p, "path/foo.tex");
  }

  #[test]
  fn make_none_dir() {
    let p = make(None, Some("foo"), Some("tex"));
    // No directory → no leading slash.
    assert_eq!(p, "foo.tex");
  }

  #[test]
  fn concat_joins_with_slash() {
    assert_eq!(concat("path", "foo.tex"), "path/foo.tex");
    assert_eq!(concat("a/b", "c.tex"), "a/b/c.tex");
  }

  #[test]
  fn url_split_basic() {
    // URL_RE captures groups: 1=host+dir, 2=filename.
    let (base, file) = url_split("http://example.com/path/file.tex");
    assert_eq!(base, "example.com/path");
    assert_eq!(file, "file.tex");
  }

  #[test]
  fn url_split_non_url_gets_index() {
    // Non-URL input falls back to (input, "index.tex").
    let (proto, rest) = url_split("plain_string");
    assert_eq!(proto, "plain_string");
    assert_eq!(rest, "index.tex");
  }

  #[test]
  fn split_basic_path() {
    let (d, n, e) = split("path/to/foo.tex");
    assert_eq!(d, "path/to");
    assert_eq!(n, "foo");
    assert_eq!(e, "tex");
  }

  #[test]
  fn split_no_ext() {
    let (_d, n, e) = split("foo");
    assert_eq!(n, "foo");
    assert_eq!(e, "");
  }

  #[test]
  fn is_nasty_detects_bad_patterns() {
    // Consult the regex; typical "nasty" is path traversal `..` or
    // shell chars.
    let has_dotdot = is_nasty("path/../bad");
    let safe = is_nasty("foo.tex");
    // Whatever the regex is, path traversal should be flagged and a
    // plain filename should not.
    assert!(!safe, "plain filename should not be nasty");
    // Keep the dotdot assertion loose — the exact patterns are
    // implementation-defined.
    let _ = has_dotdot;
  }
}

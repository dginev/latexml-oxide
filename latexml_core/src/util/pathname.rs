use kpathsea::Kpaths;
use once_cell::sync::Lazy;
use regex::Regex;

use std::env;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// configuration for filesystem search
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
static HOME_PATH: Lazy<String> = Lazy::new(|| match dirs::home_dir() {
  Some(val) => val.to_string_lossy().to_string(),
  _ => s!("~"),
});
static PROTOCOL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(https|http|ftp):").unwrap());
static PATHNAME_IS_NASTY_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"[^\w\-_+=/\\\.~\s:]").unwrap());
// TODO: This is very pragmatic for now, we ought to use a real URL path library long-term
static URL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+://(.+)/([^/]+)$").unwrap());

static KPSE: Lazy<Mutex<Option<Kpaths>>> = Lazy::new(|| Mutex::new(Kpaths::new().ok()));
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
pub fn is_url(path: &str) -> bool { URL_RE.is_match(path) }
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
pub fn absolute(path: &str) -> String {
  Path::new(path)
    .canonicalize()
    .unwrap()
    .to_string_lossy()
    .to_string()
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
        exts.push(format!(".{}", &ext));
      } else {
        exts.push(format!(".{}", &ext));
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

/// find the requested `pathname` using the `options` search configuration
pub fn find(pathname: &str, options: PathnameFindOptions) -> Option<String> {
  if !pathname.is_empty() {
    let paths = candidate_pathnames(pathname, options);
    for path in paths {
      if Path::new(&path).exists() {
        return Some(path);
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
      if let Some(path) = kpse.find_file(candidate) {
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

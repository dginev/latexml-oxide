use kpathsea::Kpaths;
use once_cell::sync::Lazy;
use regex::Regex;

use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;

/// configuration for filesystem search
#[derive(Debug, Clone, Default)]
pub struct PathnameFindOptions {
  /// the allowed/requested paths to search in
  pub paths: Option<Vec<String>>,
  /// the file extensions to search for
  pub extensions: Option<Vec<String>>,
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

static KPSE: Lazy<Arc<Mutex<Kpaths>>> = Lazy::new(|| Arc::new(Mutex::new(Kpaths::new().unwrap())));
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
  let name = match canonical_path.file_name() {
    Some(n) => n.to_string_lossy().to_string(),
    None => String::new(),
  };
  let pathname_ext = match canonical_path.extension() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }
  .to_lowercase();
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

/// This likely needs portability work!!! (particularly regarding urls, separators, ...)
/// AND, care about symbolic links and collapsing ../ !!!
pub fn canonical(pathname: &str) -> String {
  if is_literaldata(pathname) {
    return pathname.to_owned();
  }
  // Don't call is_absolute, etc, here, cause THEY call US!
  let home_path: &str = &HOME_PATH;

  // TODO: consider using Path's .canonicalize()

  // TODO: Finish fleshing out, just a mock for now
  if pathname.starts_with(HOME_TILDE) {
    pathname.replacen(HOME_TILDE, home_path, 1)
  } else {
    pathname.to_string()
  }
  // We CAN canonicalize urls, but we need to be careful about the // before host!
  // OHHH, but we DON'T want \ for separator!
  // let urlprefix = None;
  // if ($pathname =~ s|^($PROTOCOL_RE//[^/]*)/|/|) {
  //   $urlprefix = $1; }

  // if ($pathname =~ m|//+/|) {
  //   Carp::cluck "Recursive pathname? : $pathname\n"; }

  // $pathname =~ s|/\./|/|g;
  // Collapse any foo/.. patterns, but not ../..
  // while ($pathname =~ s|/(?!\.\./)[^/]+/\.\.(/\|$)|$1|) { }
  // $pathname =~ s|^\./||;
  // return (defined $urlprefix ? $urlprefix . $pathname : $pathname); }
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

  let (pathdir, name, pathname_ext) = split(&canonical_pathname);

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
  // Always have the current directory!
  let from_cwd = concat(&cwd, &pathdir);
  if !dirs.contains(&from_cwd) {
    dirs.push(from_cwd);
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
  let mut exts = Vec::new();
  if let Some(ext_vec) = options.extensions {
    for ext in ext_vec {
      if ext.is_empty() || pathname_ext == ext.to_lowercase() {
        exts.push(String::new());
      } else if ext == "*" {
        exts.push(s!(".*"));
        exts.push(String::new());
      } else {
        exts.push(s!(".{}", &ext));
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
        // Unfortunately, we've got to test the file system NOW...
        //       if ext == ".*" {    // everything
        // //         opendir(DIR, $dir) or next;
        // //         push(@paths, map { concat($dir, $_) } grep { !/^\./ } readdir(DIR));
        // //         closedir(DIR);
        //       } else {
        // //         opendir(DIR, $dir) or next;    // ???
        // //         push(@paths, map { concat($dir, $_) } grep { /\Q$ext\E$/ } readdir(DIR));
        // //         closedir(DIR); } }
        //       }
        //     } else if ext == ".*" { // Unfortunately, we've got to test the file system NOW...
        // //       opendir(DIR, $dir) or next;      // ???
        // //       push(@paths, map { concat($dir, $_) } grep { /^\Q$name\E\.\w+$/ } readdir(DIR));
        // //       closedir(DIR);
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
  .to_lowercase()
}

/// transform to a base name (via `Path::file_stem`)
pub fn file_stem(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.file_stem() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }
  .to_lowercase()
}

/// obtain the directory portion of a pathname (via `Path::parent`)
pub fn directory(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.parent() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }
  .to_lowercase()
}

/// obtain the extension portion of a pathname (via `Path::extension`)
pub fn extension(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.extension() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }
  .to_lowercase()
}

/// search for a list of candidate names via the external `kpsewhich` utility
/// returning the first path that is found
pub fn kpsewhich(candidates: &[&str]) -> Option<String> {
  for candidate in candidates {
    if let Some(path) = KPSE.lock().unwrap().find_file(candidate) {
      return Some(path);
    }
  }
  None
}

/// check if pathname contains dangerous pieces
pub fn is_nasty(file: &str) -> bool { PATHNAME_IS_NASTY_RE.is_match(file) }

/// returns the current working directory
pub fn cwd() -> String { env::current_dir().unwrap().to_string_lossy().to_string() }

use std::env;
use std::path::{Path, PathBuf};
// use regex::Regex;

#[derive(Debug, Clone)]
pub struct FindOptions {
  pub paths: Option<Vec<String>>,
  pub types: Option<Vec<String>>,
  pub installation_subdir: Option<String>,
}
impl Default for FindOptions {
  fn default() -> Self {
    FindOptions {
      paths: None,
      types: None,
      installation_subdir: None,
    }
  }
}

static LITERAL: &'static str = "literal:";
static HOME_TILDE: &'static str = "~";
lazy_static! {
  static ref HOME_PATH : String = match env::home_dir() {
    Some(val) => val.to_string_lossy().to_string(),
    _ => s!("~"),
  };
  // static ref PROTOCOL_RE : Regex = Regex::new(r"(https|http|ftp):").unwrap();
  // static ref INSTALLDIRS : Vec<String> = match env::current_exe() {
  //     Ok(exe_path) => {
  //       match exe_path.as_path().parent() {
  //         Some(_) => Vec::new(),
  //         // Some(p) => vec![
  //         //                 p.to_string_lossy().to_string() + ".",
  //         //                 p.to_string_lossy().to_string() + "./..",
  //         //                 p.to_string_lossy().to_string() + "./../..",
  //         //                 p.to_string_lossy().to_string() + "./../../..",
  //         //                 p.to_string_lossy().to_string() + "./../../../.."], // TODO: HACK, see note on INSTALLDIRS further down
  //         None => Vec::new()
  //       }
  //     },
  //     _ => Vec::new()
  //   };

  // TODO:
  // grep { (-f "$_.pm") && (-d $_) }
  // map { pathname_canonical($_ . $SEP . 'LaTeXML') } @INC;    # [CONSTANT]

}

pub fn is_url(_path: &str) -> bool {
  // TODO
  false
}

pub fn is_literaldata(_data: &str) -> bool {
  // TODO
  false
}

pub fn is_absolute(path: &str) -> bool { Path::new(&canonical(path)).is_absolute() }

pub fn absolute(path: &str) -> String {
  // TODO, just a mock now
  path.to_string()
}

/// This likely needs portability work!!! (particularly regarding urls, separators, ...)
/// AND, care about symbolic links and collapsing ../ !!!
pub fn canonical(pathname: &str) -> String {
  if pathname.starts_with(LITERAL) {
    return pathname.to_owned();
  }
  // Don't call is_absolute, etc, here, cause THEY call US!
  let home_path: &str = &*HOME_PATH;

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

pub fn concat(dir: &str, file: &str) -> String {
  if dir.is_empty() {
    file.to_owned()
  } else if file.is_empty() || file == "." {
    dir.to_owned()
  } else {
    let mut path = PathBuf::from(dir);
    path.push(file);
    canonical(&path.to_string_lossy().to_string())
  }
}

/// It's presumably cheep to concatinate all the pathnames,
/// relative to the cost of testing for files,
/// and this simplifies overall.
pub fn candidate_pathnames(pathname: &str, options: FindOptions) -> Vec<String> {
  let mut dirs: Vec<String> = Vec::new();
  let canonical_pathname = if pathname != "*" {
    canonical(pathname)
  } else {
    pathname.to_owned()
  };
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
  }.to_lowercase();

  let cwd = env::current_dir().unwrap().to_string_lossy().to_string();

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
  // At least have the current directory!
  if dirs.is_empty() {
    dirs.push(concat(&cwd, &pathdir));
  }

  // TODO: The use of INSTALLDIRS should be rethought entirely, as Rust currently doesn't have a
  // native concept of "installing" a crate and its resources there either needs to be a
  // more sophisticated build process, OR, a compile step that translates all model/binding
  // dependencies into rust code. which would allow bundling them side-by-side with the
  // main application.

  // And, if installation dir specified, append it.
  if let Some(subdir) = options.installation_subdir {
    // dirs.extend((*INSTALLDIRS).iter().map(|dir| concat(dir, &subdir)));
    dirs.push(concat(&cwd, &subdir));
  }
  // extract the desired extensions.
  let mut exts = Vec::new();
  if let Some(ext_vec) = options.types {
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

pub fn find(pathname: &str, options: FindOptions) -> Option<String> {
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

pub fn file_name(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.file_name() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }.to_lowercase()
}

pub fn file_stem(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.file_stem() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }.to_lowercase()
}

pub fn directory(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.parent() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }.to_lowercase()
}

pub fn extension(pathname: &str) -> String {
  let canonical_pathname = canonical(pathname);
  let canonical_path = Path::new(&canonical_pathname);
  match canonical_path.extension() {
    Some(e) => e.to_string_lossy().to_string(),
    None => String::new(),
  }.to_lowercase()
}

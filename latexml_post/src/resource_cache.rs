//! The resource cache: the names the generated and copied resources of one destination directory take, kept across
//! the split pages of a run and across runs into the same directory.
//!
//! Port of the part of Perl's document cache the resource names live in ([`crate::document::DocCache`], persisted as
//! [`crate::document::CACHE_FILE`] in the destination directory): `generateResourcePathname`'s counter under
//! `_max_<subdir>_<prefix>_counter_` (`Post.pm` L192-200) and Graphics' `key → "dest|w|h"` entries
//! (`Post/Graphics.pm` L245-262, L331). Two kinds of entry are ours: `_content_<content>` (the destination a content
//! already has, so one content is one file) and `_file_<name>` (what the file `name` holds, so a name another file
//! claimed is never reused). Divergences (OXIDIZED_DESIGN_DIVERGENCES #1157): a key carries the source's content
//! digest instead of Perl's timestamp check, so a changed image is a new key and takes a new name; an entry stores the
//! name only (Perl's width and height let it skip a conversion, which the content-addressed `graphics_cache` serves
//! here); and a kept name (a copy's relative path, a converted image's stem) already claimed by other content falls
//! back to a generated `xN`, where Perl overwrote it (issue 995). Each [`resolve`] is one `IMMEDIATE` transaction, so
//! processes writing into one folder at once never hand out one name twice.

use rusqlite::Connection;

use crate::document::{kv_get, kv_put};

/// What a resource's name is first tried as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Proposal {
  /// A copy's path relative to the destination directory, kept when no other content claimed it
  /// (`Post/Graphics.pm` L292-300).
  Kept(String),
  /// A converted image's source stem (`plot` for `a/plot.pdf`), with the destination type appended: a readability
  /// name Perl does not keep (it generates `xN` for every conversion, `Post/Graphics.pm` L318-319; witness
  /// 2606.30620).
  Stem(String),
  /// A generated `<prefix>N` (`generateResourcePathname`).
  Generated,
}

/// One resource to name.
#[derive(Clone, Debug)]
pub struct Request {
  /// Perl's cache key (`ref($self):reldir/name.srctype.type|transform`) with the source's content digest.
  pub key:      String,
  /// What makes two requests one resource: requests with the same content share one file.
  pub content:  String,
  /// What the destination file's bytes are: a name another file already claimed is not reused (a copy's options
  /// make another content but the same file).
  pub file:     String,
  pub proposal: Proposal,
  /// The destination's file type (extension), empty for none.
  pub ext:      String,
  /// A vector render `<name>.svg` may be written beside it, so that name is claimed too.
  pub svg_alt:  bool,
  /// A copy (named after the conversions of its page when it cannot keep its path) or a conversion.
  pub copy:     bool,
}

/// A resource's destination, relative to the destination directory, and its vector render's when it may have one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceName {
  pub dest: String,
  pub alt:  Option<String>,
}

/// How a request was named.
#[derive(Clone, Debug)]
pub struct Resolved {
  pub name:      ResourceName,
  /// A new name (a kept path, a stem or a generated one), not an entry already in the table.
  pub allocated: bool,
}

/// The counter Perl's `generateResourcePathname` keeps for `prefix` (`Post.pm` L194-196; no `resource_directory`).
pub fn counter_key(prefix: &str) -> String { format!("_max__{prefix}_counter_") }

/// Name every request, page by page, on `conn` (in the caller's `IMMEDIATE` transaction): an entry already stored
/// under its key or its content, else, within a page, the copies that keep their path first, then the conversions,
/// then the copies that take a generated name, each in node order (the order the page's names were handed out in
/// before). A stored name `usable` refuses (a copy's kept path outside this run's site directory) is not reused.
pub fn resolve(
  conn: &Connection,
  prefix: &str,
  pages: &[Vec<Request>],
  usable: &dyn Fn(&ResourceName) -> bool,
) -> rusqlite::Result<Vec<Vec<Resolved>>> {
  let counter = counter_key(prefix);
  let mut out = Vec::with_capacity(pages.len());
  for page in pages {
    let mut names: Vec<Option<Resolved>> = vec![None; page.len()];
    // Copies keeping their path.
    for (i, r) in page.iter().enumerate() {
      if let (true, Proposal::Kept(rel)) = (r.copy, &r.proposal) {
        names[i] = place(conn, r, Some(name_of(rel, false)), usable)?;
      }
    }
    // Conversions, then the copies left.
    for copies in [false, true] {
      for (i, r) in page.iter().enumerate() {
        if r.copy != copies || names[i].is_some() {
          continue;
        }
        let proposed = match &r.proposal {
          Proposal::Stem(stem) if !copies => Some(name_of(&with_ext(stem, &r.ext), r.svg_alt)),
          _ => None,
        };
        names[i] = match place(conn, r, proposed, usable)? {
          Some(found) => Some(found),
          None => Some(generate(conn, r, prefix, &counter)?),
        };
      }
    }
    out.push(names.into_iter().flatten().collect());
  }
  Ok(out)
}

fn with_ext(stem: &str, ext: &str) -> String {
  if ext.is_empty() {
    stem.to_string()
  } else {
    format!("{stem}.{ext}")
  }
}

/// `dest`, and the `.svg` of its stem when a vector render may be written beside it.
fn name_of(dest: &str, svg_alt: bool) -> ResourceName {
  let stem = dest.rsplit_once('.').map_or(dest, |(stem, _)| stem);
  ResourceName {
    dest: dest.to_string(),
    alt:  svg_alt.then(|| format!("{stem}.svg")),
  }
}

/// The entry under the request's key or content (stored under its key too), else `proposed` when no other file
/// claimed its names (stored), else `None`. A stored name `usable` refuses is passed over.
fn place(
  conn: &Connection,
  r: &Request,
  proposed: Option<ResourceName>,
  usable: &dyn Fn(&ResourceName) -> bool,
) -> rusqlite::Result<Option<Resolved>> {
  // Perl's value, `dest|w|h`.
  let by_key = kv_get(conn, &r.key)?.map(|value| {
    let dest = value.split('|').next().unwrap_or_default();
    name_of(dest, r.svg_alt)
  });
  if let Some(name) = by_key.filter(|name| usable(name)) {
    return Ok(Some(Resolved { name, allocated: false }));
  }
  let shared =
    kv_get(conn, &format!("_content_{}", r.content))?.map(|dest| name_of(&dest, r.svg_alt));
  if let Some(name) = shared.filter(|name| usable(name)) {
    insert(conn, r, &name)?;
    return Ok(Some(Resolved { name, allocated: false }));
  }
  let Some(name) = proposed else {
    return Ok(None);
  };
  if claimed(conn, &name, Some(&r.file))? {
    return Ok(None);
  }
  insert(conn, r, &name)?;
  Ok(Some(Resolved { name, allocated: true }))
}

/// The next `<prefix>N` whose files no entry claims (Perl `generateResourcePathname`, which skips none).
fn generate(
  conn: &Connection,
  r: &Request,
  prefix: &str,
  counter: &str,
) -> rusqlite::Result<Resolved> {
  let mut n: u64 = kv_get(conn, counter)?
    .and_then(|n| n.parse().ok())
    .unwrap_or(0);
  let name = loop {
    n += 1;
    let name = name_of(&with_ext(&format!("{prefix}{n}"), &r.ext), r.svg_alt);
    if !claimed(conn, &name, None)? {
      break name;
    }
  };
  kv_put(conn, counter, Some(&n.to_string()))?;
  insert(conn, r, &name)?;
  Ok(Resolved { name, allocated: true })
}

/// Whether another file than `file` (any, for `None`) holds one of `name`'s files.
fn claimed(conn: &Connection, name: &ResourceName, file: Option<&str>) -> rusqlite::Result<bool> {
  for part in std::iter::once(&name.dest).chain(&name.alt) {
    if let Some(owner) = kv_get(conn, &format!("_file_{part}"))?
      && file != Some(owner.as_str())
    {
      return Ok(true);
    }
  }
  Ok(false)
}

fn insert(conn: &Connection, r: &Request, name: &ResourceName) -> rusqlite::Result<()> {
  kv_put(conn, &r.key, Some(&format!("{}||", name.dest)))?;
  kv_put(conn, &format!("_content_{}", r.content), Some(&name.dest))?;
  for part in std::iter::once(&name.dest).chain(&name.alt) {
    kv_put(conn, &format!("_file_{part}"), Some(&r.file))?;
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::document::{CACHE_FILE, DocCache};

  /// Name `pages` in `cache`'s store of `dir`, in one transaction.
  fn resolve_in(cache: &DocCache, dir: Option<&str>, pages: &[Vec<Request>]) -> Vec<Vec<Resolved>> {
    cache.with_store(dir, |store| {
      store.immediate(|tx| resolve(tx, "x", pages, &|_| true))
    })
  }

  fn copy(key: &str, content: &str, kept: Option<&str>, ext: &str) -> Request {
    Request {
      key:      key.to_string(),
      content:  content.to_string(),
      file:     content.to_string(),
      proposal: kept.map_or(Proposal::Generated, |k| Proposal::Kept(k.to_string())),
      ext:      ext.to_string(),
      svg_alt:  false,
      copy:     true,
    }
  }

  fn convert(key: &str, content: &str, stem: Option<&str>, svg_alt: bool) -> Request {
    Request {
      key: key.to_string(),
      content: content.to_string(),
      file: content.to_string(),
      proposal: stem.map_or(Proposal::Generated, |s| Proposal::Stem(s.to_string())),
      ext: "png".to_string(),
      svg_alt,
      copy: false,
    }
  }

  fn dests(pages: &[Vec<Resolved>]) -> Vec<Vec<String>> {
    pages
      .iter()
      .map(|page| page.iter().map(|r| r.name.dest.clone()).collect())
      .collect()
  }

  /// Within a page the kept copies first, then the conversions, then the generated copies; a stem or path another
  /// content claimed takes the next `xN`; one content, one file; a key met again keeps its name.
  #[test]
  fn names_follow_the_page_order_and_never_clash() {
    let cache = DocCache::new(false);
    let pages = vec![
      vec![
        copy("c:../A.svg", "copy|A", None, "svg"),
        convert("v:a/plot.pdf", "png|a", Some("plot"), true),
        copy("c:x1.png", "copy|author", Some("x1.png"), "png"),
      ],
      vec![
        convert("v:b/plot.pdf", "png|b", Some("plot"), false),
        copy("c:plot.svg", "copy|svg", Some("plot.svg"), "svg"),
        copy("c:../A2.svg", "copy|A", None, "svg"),
      ],
    ];
    let first = resolve_in(&cache, None, &pages);
    assert_eq!(dests(&first), [vec!["x1.svg", "plot.png", "x1.png"], vec![
      "x2.png", "x3.svg", "x1.svg"
    ]]);
    assert_eq!(first[0][1].name.alt.as_deref(), Some("plot.svg"));
    // A second run (or another document into the folder): the same keys keep their names, new content continues
    // the counter.
    let again = resolve_in(&cache, None, &[vec![
      convert("v:b/plot.pdf", "png|b", Some("plot"), false),
      convert("v:c/plot.pdf", "png|c", Some("plot"), false),
    ]]);
    assert_eq!(dests(&again), [vec!["x2.png", "x4.png"]]);
    assert!(!again[0][0].allocated && again[0][1].allocated);
  }

  /// Two processes allocating into one cache file at once never hand out one name twice.
  #[test]
  fn concurrent_tables_on_one_file_never_share_a_name() {
    let dir = tempfile::tempdir().unwrap();
    let dir_str = dir.path().to_string_lossy().into_owned();
    let names: Vec<Vec<String>> = std::thread::scope(|s| {
      let handles: Vec<_> = (0..8)
        .map(|t| {
          let dir_str = dir_str.clone();
          s.spawn(move || {
            let cache = DocCache::new(false);
            (0..20)
              .map(|i| {
                let r = copy(&format!("k{t}_{i}"), &format!("c{t}_{i}"), None, "png");
                resolve_in(&cache, Some(&dir_str), &[vec![r]])[0][0]
                  .name
                  .dest
                  .clone()
              })
              .collect()
          })
        })
        .collect();
      handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all: Vec<String> = names.into_iter().flatten().collect();
    all.sort();
    let count = all.len();
    all.dedup();
    assert_eq!((all.len(), count), (160, 160));
    assert!(dir.path().join(CACHE_FILE).exists());
    assert!(!dir.path().join(format!("{CACHE_FILE}-wal")).exists());
  }
}

//! The resource cache: the names the generated and copied resources of one destination directory take, kept across
//! the split pages of a run and across runs into the same directory.
//!
//! Port of the part of Perl's document cache the resource names live in ([`crate::document::DocCache`], persisted as
//! [`crate::document::CACHE_FILE`] in the destination directory): `generateResourcePathname`'s counter under
//! `_max_<subdir>_<prefix>_counter_` (`Post.pm` L192-200) and Graphics' `key → "dest|w|h"` entries
//! (`Post/Graphics.pm` L245-262, L331). A name's owner is its source (path relative to the destination directory,
//! page, crop and turn, type; never a size), as a Perl key is: our entries `_owner_<name>`, `_name_<owner>` and
//! `_source_<name>` keep it, and `_holds_<name>` the bytes the name was last written with, so an edited image is a new
//! key (the content digest replaces Perl's timestamp check) written again into the name its source owns.
//! `_content_<bytes>` lets a source with the same bytes share that file while its owner is named in the same run with
//! them (the pages of one document); another document's file is never shared, as that document may write other bytes
//! into it later, under a page this run does not rewrite. A copy keeping its path shares none (Perl L289-300). Divergences (OXIDIZED_DESIGN_DIVERGENCES #1157): an entry
//! stores the name only (Perl's width and height let it skip a conversion, which the content-addressed
//! `graphics_cache` serves here); a converted or generated name is kept across an edit (Perl generates a new `xN`);
//! a kept name (a copy's relative path, a converted image's stem) another source owns falls back to a generated `xN`,
//! where Perl overwrote it (issue 995); and a stored name is written only when the request could have been given it
//! ([`Checks`]). Each [`resolve`] is one `IMMEDIATE` transaction, so processes naming into one folder at once through
//! the file never hand out one name to two sources. A name new to the store is never given over a file in the folder
//! the request may not write ([`Checks::on_disk`]): with the store on its file (a new one too, made where a cache was
//! deleted) any file it does not own; in memory in place of its file (it could not be opened, or the transaction
//! failed and runs again there) a file holding other bytes than the request writes, so a rerun takes its own files
//! again. One that could not copy the file's entries does not know a file's owner: a file of another document holding
//! the same bytes is taken as the request's own (OD #1157 (10)).

use std::collections::HashMap;

use rusqlite::Connection;

use crate::document::{kv_get, kv_put};

/// What a resource's name is first tried as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Proposal {
  /// A copy's path relative to the destination directory, kept when no other source owns it
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
  /// What the destination file's bytes are made of (the source's bytes, page, type, crop and turn, never a size):
  /// requests with the same bytes share one file.
  pub bytes:    String,
  /// Who owns the names the request is given: its source, transform and type, so an edited source keeps them.
  pub owner:    String,
  /// The source, relative to the destination directory, kept with the names it owns ([`Checks::vouch`]).
  pub source:   String,
  /// The source's content digest (hex), or its path when unreadable.
  pub identity: String,
  pub proposal: Proposal,
  /// The destination's file type (extension), empty for none.
  pub ext:      String,
  /// A vector render `<name>.svg` may be written beside it, so that name is claimed too.
  pub svg_alt:  bool,
  /// A copy (named after the conversions of its page when it cannot keep its path) or a conversion.
  pub copy:     bool,
  /// What the destination file will hold ([`Checks::on_disk`]).
  pub output:   Output,
}

/// What a resource's file will hold, so that a store in memory tells the file an earlier run wrote for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
  /// The source's bytes (a copy).
  Source,
  /// The source's page (`None`: the default) rendered, as the graphics cache keeps it.
  Render(Option<u32>),
  /// Known only once made (a cropped or turned image).
  Made,
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
  /// A name not stored under the request's key: a kept path, a stem, a generated one, or its source's earlier name.
  pub allocated: bool,
}

/// Whether a file of the name exists in the destination directory that the request may not write ([`Checks::on_disk`]).
pub type OnDisk<'a> = &'a dyn Fn(&Request, &ResourceName) -> bool;

/// What the caller knows of the destination directory.
pub struct Checks<'a> {
  /// Perl's `resource_prefix` (`x` unless set).
  pub prefix:  &'a str,
  /// Whether a name may be written: a copy's kept path outside this run's site directory may not.
  pub usable:  &'a dyn Fn(&ResourceName) -> bool,
  /// Whether `name`, owned by the source at `source` (relative to the destination directory), is the name that source
  /// takes and the source holds the request's bytes: another source's kept path or stem a request may share. Only a
  /// relative path to a regular file is read.
  pub vouch:   &'a dyn Fn(&Request, &ResourceName, &str) -> bool,
  /// Whether a file of the name exists in the destination directory that the request may not write, asked of a name
  /// the store does not know: the store may not know every file earlier runs wrote (a deleted cache, an unusable one).
  /// `None` for a store in memory from the start (`--nocache`), whose names restart and overwrite as Perl's do.
  pub on_disk: Option<OnDisk<'a>>,
  /// The cache file a Warning names.
  pub cache:   &'a str,
  /// Report a Warning (`unexpected`, `cache`): the caller emits it once the transaction is done, as a transaction run
  /// again in memory meets the same entries.
  pub warn:    &'a dyn Fn(String),
}

/// The counter Perl's `generateResourcePathname` keeps for `prefix` (`Post.pm` L194-196; no `resource_directory`).
pub fn counter_key(prefix: &str) -> String { format!("_max__{prefix}_counter_") }

/// Name every request, page by page, on `conn` (in the caller's `IMMEDIATE` transaction): a name stored for its key,
/// its source or its content, else, within a page, the copies that keep their path first, then the conversions, then
/// the copies that take a generated name, each in node order (the order the page's names were handed out in before).
pub fn resolve(
  conn: &Connection,
  checks: &Checks,
  pages: &[Vec<Request>],
) -> rusqlite::Result<Vec<Vec<Resolved>>> {
  let counter = counter_key(checks.prefix);
  // The sources this run names, with their bytes: only their files are shared, while they hold the same bytes.
  let now: HashMap<&str, &str> = pages
    .iter()
    .flatten()
    .map(|r| (r.owner.as_str(), r.bytes.as_str()))
    .collect();
  let mut out = Vec::with_capacity(pages.len());
  for page in pages {
    let mut names: Vec<Option<Resolved>> = vec![None; page.len()];
    // Copies keeping their path.
    for (i, r) in page.iter().enumerate() {
      if let (true, Proposal::Kept(rel)) = (r.copy, &r.proposal) {
        names[i] = place(conn, checks, &now, r, Some(name_of(rel, false)), true)?;
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
        // A copy whose kept path is taken looks its stored names up again (a copy since may share its bytes), its
        // corrupt ones reported already.
        let warn = !matches!((r.copy, &r.proposal), (true, Proposal::Kept(_)));
        names[i] = match place(conn, checks, &now, r, proposed, warn)? {
          Some(found) => Some(found),
          None => Some(generate(conn, checks, r, &counter)?),
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

/// The name stored under the request's key, its source's or its content's, else `proposed` when no other source
/// owns it, else `None`. A stored name no request could have is reported when `warn`.
fn place(
  conn: &Connection,
  checks: &Checks,
  now: &HashMap<&str, &str>,
  r: &Request,
  proposed: Option<ResourceName>,
  warn: bool,
) -> rusqlite::Result<Option<Resolved>> {
  // A copy offered its kept path is that path or none: it shares no other source's file of the same bytes, as Perl
  // copies each to its own path (`Post/Graphics.pm` L289-300).
  let keeps = r.copy && matches!(r.proposal, Proposal::Kept(_)) && proposed.is_some();
  // Perl's value, `dest|w|h`.
  let by_key =
    kv_get(conn, &r.key)?.map(|value| value.split('|').next().unwrap_or_default().to_string());
  let by_owner = kv_get(conn, &format!("_name_{}", r.owner))?;
  let by_content = if keeps {
    None
  } else {
    kv_get(conn, &format!("_content_{}", r.bytes))?
  };
  // The first two are names this source was given: one no request could have is a corrupt or planted entry.
  let mut tried: Vec<String> = Vec::new();
  for (dest, given, keyed) in [
    (by_key, true, true),
    (by_owner, true, false),
    (by_content, false, false),
  ] {
    let Some(dest) = dest.filter(|dest| !tried.contains(dest)) else {
      continue;
    };
    tried.push(dest.clone());
    let name = name_of(&dest, r.svg_alt);
    let Some(own) = reusable(conn, checks, now, r, &name, warn && given, !keeps)? else {
      continue;
    };
    if own {
      take(conn, r, &name)?;
    } else {
      share(conn, r, &name)?;
    }
    return Ok(Some(Resolved { name, allocated: !keyed }));
  }
  let Some(name) = proposed else {
    return Ok(None);
  };
  // A kept path or a stem the store does not know may be a file of another document's (issue 995 review: an unusable
  // cache, and a second document's `figs/a.png` written over the first's).
  let taken = claimed(conn, &name, Some(&r.owner))?
    || checks.on_disk.is_some_and(|on_disk| on_disk(r, &name));
  if taken {
    return Ok(None);
  }
  take(conn, r, &name)?;
  Ok(Some(Resolved { name, allocated: true }))
}

/// Whether a stored `name` serves `r`: `Some(true)` when `r`'s source owns it (or no source does and `r` could be
/// given it), `Some(false)` when `r` may `share` and shares the file of another source of this run holding its bytes,
/// `None` when it does not serve. A name no request of `r`'s type could have is reported when `warn` (a Warning) and
/// never written.
fn reusable(
  conn: &Connection,
  checks: &Checks,
  now: &HashMap<&str, &str>,
  r: &Request,
  name: &ResourceName,
  warn: bool,
  share: bool,
) -> rusqlite::Result<Option<bool>> {
  let given = own_name(r, &name.dest) || generated_name(checks, r, &name.dest);
  if !given && !shareable_name(r, &name.dest) {
    if warn {
      (checks.warn)(format!(
        "The cache {} names {} for {}, a name no {} image is given; it is ignored",
        checks.cache, name.dest, r.source, r.ext
      ));
    }
    return Ok(None);
  }
  if !(checks.usable)(name) {
    return Ok(None);
  }
  // A source owns only its kept path, its stem or a generated name.
  let owner = kv_get(conn, &format!("_owner_{}", name.dest))?;
  let Some(owner) = owner.filter(|owner| *owner != r.owner) else {
    return Ok(given.then_some(true));
  };
  if !share {
    return Ok(None);
  }
  // Another source's file, only while that source is named in this run with these bytes: a source of another
  // document (or not in this run) may write other bytes into it later, under a page this run does not rewrite.
  let holds = kv_get(conn, &format!("_holds_{}", name.dest))?;
  if holds.as_deref() != Some(r.bytes.as_str())
    || now.get(owner.as_str()) != Some(&r.bytes.as_str())
  {
    return Ok(None);
  }
  if given {
    return Ok(Some(false));
  }
  let source = kv_get(conn, &format!("_source_{}", name.dest))?.unwrap_or_default();
  Ok((checks.vouch)(r, name, &source).then_some(false))
}

/// Its kept path or its stem.
fn own_name(r: &Request, dest: &str) -> bool {
  match &r.proposal {
    Proposal::Kept(rel) => dest == rel,
    Proposal::Stem(stem) => dest == with_ext(stem, &r.ext),
    Proposal::Generated => false,
  }
}

/// A `<prefix>N` of its type.
fn generated_name(checks: &Checks, r: &Request, dest: &str) -> bool {
  let stem = if r.ext.is_empty() {
    Some(dest)
  } else {
    dest.strip_suffix(&format!(".{}", r.ext))
  };
  stem
    .and_then(|stem| stem.strip_prefix(checks.prefix))
    .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// A name another source of the same bytes may have: of `r`'s type, no hidden file or directory on its path.
fn shareable_name(r: &Request, dest: &str) -> bool {
  let family = |ext: &str| match ext.to_ascii_lowercase().as_str() {
    "jpeg" => "jpg".to_string(),
    "tiff" => "tif".to_string(),
    ext => ext.to_string(),
  };
  let ext = std::path::Path::new(dest)
    .extension()
    .map_or(String::new(), |e| e.to_string_lossy().into_owned());
  !r.ext.is_empty()
    && family(&ext) == family(&r.ext)
    && dest
      .split(['/', '\\'])
      .all(|part| !part.is_empty() && (part == ".." || !part.starts_with('.')))
}

/// The next `<prefix>N` no source owns (Perl `generateResourcePathname`, which skips none) and whose file the request
/// may write ([`Checks::on_disk`]). A counter that is no count, or has no next, is reported and counted again from 0.
fn generate(
  conn: &Connection,
  checks: &Checks,
  r: &Request,
  counter: &str,
) -> rusqlite::Result<Resolved> {
  // A planted `u64::MAX` wrapped to `x0` (issue 995 review, round 7).
  let unusable = |value: &str| {
    (checks.warn)(format!(
      "The cache {} holds {value} for {counter}, no count with a next one; names are counted again from {}1",
      checks.cache, checks.prefix
    ))
  };
  let stored = kv_get(conn, counter)?;
  let mut n: u64 = match stored.as_deref().map(str::parse) {
    None => 0,
    Some(Ok(n)) => n,
    Some(Err(_)) => {
      unusable(stored.as_deref().unwrap_or_default());
      0
    },
  };
  let name = loop {
    n = n.checked_add(1).unwrap_or_else(|| {
      unusable(&n.to_string());
      1
    });
    let name = name_of(
      &with_ext(&format!("{}{n}", checks.prefix), &r.ext),
      r.svg_alt,
    );
    if !claimed(conn, &name, None)? && !checks.on_disk.is_some_and(|on_disk| on_disk(r, &name)) {
      break name;
    }
  };
  kv_put(conn, counter, Some(&n.to_string()))?;
  take(conn, r, &name)?;
  Ok(Resolved { name, allocated: true })
}

/// Whether another source than `owner` (any, for `None`) owns one of `name`'s files.
fn claimed(conn: &Connection, name: &ResourceName, owner: Option<&str>) -> rusqlite::Result<bool> {
  for part in std::iter::once(&name.dest).chain(&name.alt) {
    if let Some(found) = kv_get(conn, &format!("_owner_{part}"))?
      && owner != Some(found.as_str())
    {
      return Ok(true);
    }
  }
  Ok(false)
}

/// `r`'s source owns `name`, which holds `r`'s bytes.
fn take(conn: &Connection, r: &Request, name: &ResourceName) -> rusqlite::Result<()> {
  share(conn, r, name)?;
  for part in std::iter::once(&name.dest).chain(&name.alt) {
    kv_put(conn, &format!("_owner_{part}"), Some(&r.owner))?;
  }
  kv_put(conn, &format!("_holds_{}", name.dest), Some(&r.bytes))?;
  kv_put(conn, &format!("_source_{}", name.dest), Some(&r.source))?;
  kv_put(conn, &format!("_name_{}", r.owner), Some(&name.dest))
}

/// `r`'s key and content name `name`.
fn share(conn: &Connection, r: &Request, name: &ResourceName) -> rusqlite::Result<()> {
  kv_put(conn, &r.key, Some(&format!("{}||", name.dest)))?;
  kv_put(conn, &format!("_content_{}", r.bytes), Some(&name.dest))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::document::{CACHE_FILE, DocCache};

  fn checks<'a>(
    vouch: &'a dyn Fn(&Request, &ResourceName, &str) -> bool,
    on_disk: Option<OnDisk<'a>>,
  ) -> Checks<'a> {
    Checks {
      prefix: "x",
      usable: &|_| true,
      vouch,
      on_disk,
      cache: CACHE_FILE,
      warn: &|_| {},
    }
  }

  /// Name `pages` in `cache`'s store of `dir`, in one transaction, every shared name vouched for.
  fn resolve_in(cache: &DocCache, dir: Option<&str>, pages: &[Vec<Request>]) -> Vec<Vec<Resolved>> {
    cache
      .with_store(dir, |store| {
        store.immediate(|tx, _| resolve(tx, &checks(&|_, _, _| true, None), pages))
      })
      .unwrap()
  }

  /// A copy of `source` holding `bytes`, proposing its kept path.
  fn copy(source: &str, bytes: &str, kept: Option<&str>, ext: &str) -> Request {
    Request {
      key:      format!("c:{source}|{bytes}"),
      bytes:    format!("copy|{bytes}"),
      owner:    format!("copy|{source}"),
      source:   source.to_string(),
      identity: bytes.to_string(),
      proposal: kept.map_or(Proposal::Generated, |k| Proposal::Kept(k.to_string())),
      ext:      ext.to_string(),
      svg_alt:  false,
      copy:     true,
      output:   Output::Source,
    }
  }

  /// A conversion of `source` holding `bytes` to PNG, proposing its stem.
  fn convert(source: &str, bytes: &str, stem: Option<&str>, svg_alt: bool) -> Request {
    Request {
      key: format!("v:{source}|{bytes}"),
      bytes: format!("png|{bytes}"),
      owner: format!("png|{source}"),
      source: source.to_string(),
      identity: bytes.to_string(),
      proposal: stem.map_or(Proposal::Generated, |s| Proposal::Stem(s.to_string())),
      ext: "png".to_string(),
      svg_alt,
      copy: false,
      output: Output::Render(None),
    }
  }

  fn dests(pages: &[Vec<Resolved>]) -> Vec<Vec<String>> {
    pages
      .iter()
      .map(|page| page.iter().map(|r| r.name.dest.clone()).collect())
      .collect()
  }

  /// Within a page the kept copies first, then the conversions, then the generated copies; a stem or path another
  /// source owns takes the next `xN`; one content, one file; a key met again keeps its name.
  #[test]
  fn names_follow_the_page_order_and_never_clash() {
    let cache = DocCache::new(false);
    let pages = vec![
      vec![
        copy("../A.svg", "A", None, "svg"),
        convert("a/plot.pdf", "a", Some("plot"), true),
        copy("x1.png", "author", Some("x1.png"), "png"),
      ],
      vec![
        convert("b/plot.pdf", "b", Some("plot"), false),
        copy("plot.svg", "svg", Some("plot.svg"), "svg"),
        copy("../A2.svg", "A", None, "svg"),
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
      convert("b/plot.pdf", "b", Some("plot"), false),
      convert("c/plot.pdf", "c", Some("plot"), false),
    ]]);
    assert_eq!(dests(&again), [vec!["x2.png", "x4.png"]]);
    assert!(!again[0][0].allocated && again[0][1].allocated);
  }

  /// A name's owner is its source: an edited source is a new key written again into the names its source has, a kept
  /// path, a stem and a generated name alike (issue 995 review B1: `x1.png`, `x2.png`... after an edit, the kept
  /// file left with the old picture).
  #[test]
  fn an_edited_source_keeps_its_names() {
    let cache = DocCache::new(false);
    let run = |v: &str| {
      dests(&resolve_in(&cache, None, &[vec![
        copy("figs/a.png", v, Some("figs/a.png"), "png"),
        convert("figs/p.eps", v, Some("p"), false),
        convert("figs/r.png", &format!("turned {v}"), None, false),
        copy("../o.svg", v, None, "svg"),
      ]]))
    };
    let names = [vec!["figs/a.png", "p.png", "x1.png", "x2.svg"]];
    assert_eq!(run("1"), names);
    assert_eq!(run("2"), names);
    assert_eq!(run("1"), names);
  }

  /// A source sharing another's file (same bytes) leaves it when its owner writes other bytes into it, whichever of
  /// the two is named first; a name no source vouches for is not shared. A copy keeping its path shares none, also
  /// where a cache of before stored the shared name (KEPTDEDUP: `figs/a.svg` and `figs/b.svg`, one SVG, both
  /// `figs/a.svg`; Perl `Post/Graphics.pm` L289-300).
  #[test]
  fn a_shared_file_is_left_when_its_owner_changes() {
    let cache = DocCache::new(false);
    let first = resolve_in(&cache, None, &[vec![
      copy("../a.png", "X", None, "png"),
      copy("../b.png", "X", None, "png"),
    ]]);
    assert_eq!(dests(&first), [vec!["x1.png", "x1.png"]]);
    let second = resolve_in(&cache, None, &[vec![
      copy("../b.png", "X", None, "png"),
      copy("../a.png", "Y", None, "png"),
    ]]);
    assert_eq!(dests(&second), [vec!["x2.png", "x1.png"]]);
    let kept = [vec![
      copy("a.png", "X", Some("a.png"), "png"),
      copy("b.png", "X", Some("b.png"), "png"),
      copy("c.png", "X", Some("c.png"), "png"),
    ]];
    let shared_before = DocCache::new(false);
    shared_before.with_store(None, |store| {
      store.immediate(|tx, _| {
        kv_put(tx, "c:b.png|X", Some("a.png||"))?;
        kv_put(tx, "_content_copy|X", Some("a.png"))?;
        kv_put(tx, "_owner_a.png", Some("copy|a.png"))?;
        kv_put(tx, "_holds_a.png", Some("copy|X"))
      })
    });
    for cache in [DocCache::new(false), shared_before] {
      for _run in 0..2 {
        assert_eq!(dests(&resolve_in(&cache, None, &kept)), [vec![
          "a.png", "b.png", "c.png"
        ]]);
      }
    }
    let unvouched = DocCache::new(false);
    let pages = [vec![
      convert("a/plot.pdf", "X", Some("plot"), false),
      convert("b/fig.pdf", "X", Some("fig"), false),
    ]];
    let names = unvouched
      .with_store(None, |store| {
        store.immediate(|tx, _| resolve(tx, &checks(&|_, _, _| false, None), &pages))
      })
      .unwrap();
    assert_eq!(dests(&names), [vec!["plot.png", "fig.png"]]);
    // Vouched for, two conversions of one content share one file, as two copies with generated names do.
    let vouched = resolve_in(&DocCache::new(false), None, &[pages[0].clone(), vec![
      copy("../p.svg", "S", None, "svg"),
      copy("../q.svg", "S", None, "svg"),
    ]]);
    assert_eq!(dests(&vouched), [vec!["plot.png", "plot.png"], vec![
      "x1.svg", "x1.svg"
    ]]);
  }

  /// A file is shared with another document's image of the same bytes never: that document may later write other
  /// bytes into it, under a page this run does not rewrite (issue 995 review: `d1` and `d2` into one folder both on
  /// `x1.png`; `d1`'s image edited and `d1` alone run again, `d2.html` showed `d1`'s new picture). The pages of one
  /// run share it.
  #[test]
  fn another_documents_file_is_never_shared() {
    let cache = DocCache::new(false);
    let d1 = resolve_in(&cache, None, &[vec![copy("../c1/a.png", "X", None, "png")]]);
    let d2 = resolve_in(&cache, None, &[vec![copy("../c2/b.png", "X", None, "png")]]);
    let d1_edited = resolve_in(&cache, None, &[vec![copy("../c1/a.png", "Y", None, "png")]]);
    let both = resolve_in(&cache, None, &[
      vec![copy("../c3/c.png", "Z", None, "png")],
      vec![copy("../c4/d.png", "Z", None, "png")],
    ]);
    let all = [dests(&d1), dests(&d2), dests(&d1_edited), dests(&both)].concat();
    assert_eq!(all, [["x1.png"], ["x2.png"], ["x1.png"], ["x3.png"], [
      "x3.png"
    ]]);
  }

  /// A stored name no request could have been given (another type, a hidden path) is never written, nor a name a
  /// source is recorded to own that is neither its kept path, its stem nor a generated one; a generated name of the
  /// request's type no source owns is.
  #[test]
  fn a_planted_name_is_ignored() {
    let cache = DocCache::new(false);
    let pages = [vec![
      copy("a.png", "A", Some("a.png"), "png"),
      copy("../b.png", "B", None, "png"),
      copy("../c.png", "C", None, "png"),
      copy("../d.png", "D", None, "png"),
    ]];
    cache.with_store(None, |store| {
      store.immediate(|tx, _| {
        kv_put(tx, "c:a.png|A", Some("Ch1.html||"))?;
        kv_put(tx, "c:../b.png|B", Some(".hidden/evil.png||"))?;
        kv_put(tx, "c:../c.png|C", Some("x7.png||"))?;
        kv_put(tx, "_name_copy|../d.png", Some("figs/other.png"))?;
        kv_put(tx, "_owner_figs/other.png", Some("copy|../d.png"))
      })
    });
    assert_eq!(dests(&resolve_in(&cache, None, &pages)), [vec![
      "a.png", "x1.png", "x7.png", "x2.png"
    ]]);
  }

  /// A store that fell back to memory skips a generated name, a stem or a kept path whose file holds other bytes, and
  /// takes one holding the request's own: `x1.png` holds `a/plot.pdf`'s render, `plot.png`, `x2.png` and
  /// `figs/k.png` other images (issue 995 review, round 7: a kept path was not asked, and a second document's
  /// `figs/a.png` was written over the first's).
  #[test]
  fn a_store_in_memory_skips_files_on_disk() {
    let on_disk = |r: &Request, name: &ResourceName| match name.dest.as_str() {
      "x1.png" => r.identity != "a",
      "plot.png" | "x2.png" | "figs/k.png" => true,
      _ => false,
    };
    let pages = [vec![
      convert("a/plot.pdf", "a", Some("plot"), false),
      copy("../b.png", "b", None, "png"),
      copy("figs/k.png", "k", Some("figs/k.png"), "png"),
      copy("figs/m.png", "m", Some("figs/m.png"), "png"),
    ]];
    let names = DocCache::new(true)
      .with_store(None, |store| {
        store.immediate(|tx, _| resolve(tx, &checks(&|_, _, _| true, Some(&on_disk)), &pages))
      })
      .unwrap();
    assert_eq!(dests(&names), [vec![
      "x1.png",
      "x3.png",
      "x4.png",
      "figs/m.png"
    ]]);
  }

  /// A counter planted with no next count (`u64::MAX`, which wrapped to `x0`) or with none at all is reported and
  /// counted again from `x1`, past the files the on-disk check finds (`x1.png` here).
  #[test]
  fn a_counter_with_no_next_counts_again() {
    for planted in [u64::MAX.to_string(), "seven".to_string()] {
      let warnings = std::cell::RefCell::new(Vec::new());
      let warn = |w: String| warnings.borrow_mut().push(w);
      let on_disk = |_: &Request, name: &ResourceName| name.dest == "x1.png";
      let pages = [vec![copy("../b.png", "b", None, "png")]];
      let names = DocCache::new(false)
        .with_store(None, |store| {
          store.immediate(|tx, _| {
            kv_put(tx, &counter_key("x"), Some(&planted))?;
            let checks = Checks {
              warn: &warn,
              ..checks(&|_, _, _| true, Some(&on_disk))
            };
            let names = resolve(tx, &checks, &pages)?;
            Ok((names, kv_get(tx, &counter_key("x"))?))
          })
        })
        .unwrap();
      assert_eq!(
        (dests(&names.0), names.1.as_deref(), warnings.into_inner()),
        (vec![vec!["x2.png".to_string()]], Some("2"), vec![format!(
          "The cache {CACHE_FILE} holds {planted} for _max__x_counter_, no count with a next one; names are \
           counted again from x1"
        )])
      );
    }
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
                let r = copy(&format!("../k{t}_{i}"), &format!("c{t}_{i}"), None, "png");
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

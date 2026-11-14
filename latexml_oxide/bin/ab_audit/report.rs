//! The `compare` and `audit` drivers: rows over a tree (or two), the TSV, the per-paper detail files and
//! the summary, regressions ranked first.

use std::{
  collections::BTreeMap,
  fs,
  io::Write,
  path::{Path, PathBuf},
};

use crate::{
  diff::{AUDIT_COLUMNS, COMPARE_COLUMNS, Row, Tier, audit, compare},
  tree::{Discovery, discover, load, par_map, stamp_lines},
};

/// Where a report goes: `<prefix>.tsv`, `<prefix>.summary.txt`, `<prefix>.details/<key>.txt`.
pub struct Output {
  pub prefix: PathBuf,
}

impl Output {
  fn path(&self, suffix: &str) -> PathBuf {
    let mut p = self.prefix.clone().into_os_string();
    p.push(suffix);
    PathBuf::from(p)
  }
}

fn progress(total: usize) -> impl Fn(usize) + Sync {
  move |done| {
    if done % 500 == 0 || done == total {
      eprintln!("ab_audit: {done}/{total}");
    }
  }
}

/// The keys of a `--keys` file: the first cell of each line, as written (a compare key is a path under the
/// roots, not a paper id to normalise), first occurrence kept.
fn read_keys(path: &Path) -> std::io::Result<Vec<String>> {
  let mut keys: Vec<String> = Vec::new();
  let text = fs::read_to_string(path)
    .map_err(|e| std::io::Error::new(e.kind(), format!("--keys {}: {e}", path.display())))?;
  for line in text.lines() {
    let key = line.split('\t').next().unwrap_or("").trim();
    if !key.is_empty() && !key.starts_with('#') && !keys.iter().any(|k| k == key) {
      keys.push(key.to_string());
    }
  }
  Ok(keys)
}

/// A root that is not a directory is an error (exit 2), never an empty comparison that passes.
fn require_dir(root: &Path) -> std::io::Result<()> {
  if root.is_dir() {
    Ok(())
  } else {
    Err(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!("{}: not a directory", root.display()),
    ))
  }
}

/// No document at all is an error (exit 2): an empty report must not read as "no regression".
fn require_keys(keys: &[String], what: &str) -> std::io::Result<()> {
  if keys.is_empty() {
    Err(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!("no documents in {what}"),
    ))
  } else {
    Ok(())
  }
}

/// The header line for a tree's unrecognised leaf directories.
fn unrecognised_line(tag: &str, found: &Discovery) -> String {
  let first: Vec<&str> = found
    .unrecognised
    .iter()
    .take(5)
    .map(String::as_str)
    .collect();
  format!(
    "{tag}: {} unrecognised director{} (leaves without document files, or cut by the depth limit){}{}",
    found.unrecognised.len(),
    if found.unrecognised.len() == 1 {
      "y"
    } else {
      "ies"
    },
    if first.is_empty() { "" } else { ": " },
    first.join(" ")
  )
}

/// Write rows: the TSV (header lines start with `#`), the detail files of every non-`same` row.
fn write_rows(
  out: &Output,
  header: &[String],
  columns: &[&str],
  rows: &[Row],
) -> std::io::Result<()> {
  if let Some(parent) = out.prefix.parent()
    && !parent.as_os_str().is_empty()
  {
    fs::create_dir_all(parent)?;
  }
  let details = out.path(".details");
  let _ = fs::remove_dir_all(&details);
  fs::create_dir_all(&details)?;
  let mut tsv = std::io::BufWriter::new(fs::File::create(out.path(".tsv"))?);
  for line in header {
    writeln!(tsv, "# {line}")?;
  }
  writeln!(tsv, "key\ttier\tflags\t{}\tdetail", columns.join("\t"))?;
  for row in rows {
    let tier = row.tier.unwrap_or(Tier::Same);
    let detail = if tier == Tier::Same {
      String::new()
    } else {
      let name = format!("{}.txt", row.key.replace('/', "__"));
      let mut text = format!("{}\t{}\t{}\n", row.key, tier.name(), row.flags.join(","));
      for (c, v) in columns.iter().zip(&row.cells) {
        text.push_str(&format!("{c}: {v}\n"));
      }
      text.push_str(&row.details);
      fs::write(details.join(&name), text)?;
      name
    };
    writeln!(
      tsv,
      "{}\t{}\t{}\t{}\t{detail}",
      row.key,
      tier.name(),
      row.flags.join(","),
      row.cells.join("\t")
    )?;
  }
  tsv.flush()
}

/// The summary: header, tier and flag tallies, then the ranked rows (regressions, then review, then the
/// first 200 changes).
fn summary(
  out: &Output,
  header: &[String],
  rows: &[Row],
  totals: &[String],
) -> std::io::Result<String> {
  let mut text = String::new();
  for line in header {
    text.push_str(&format!("{line}\n"));
  }
  text.push('\n');
  for line in totals {
    text.push_str(&format!("{line}\n"));
  }
  let mut tiers: BTreeMap<Tier, usize> = BTreeMap::new();
  for row in rows {
    *tiers.entry(row.tier.unwrap_or(Tier::Same)).or_default() += 1;
  }
  // Per-paper family counts (dedup within a paper): `lost-text:p` and `lost-text:td` are `lost-text`;
  // `lost-elements:<qname>` stays apart, and so do a problem on both sides (`timeout:both`) and text or
  // attributes of an `ERROR` (`lost-text:ERROR`), which rank in other tiers.
  let mut counts: BTreeMap<(usize, String), (usize, Tier)> = BTreeMap::new();
  for row in rows {
    let mut seen: Vec<(usize, String)> = Vec::new();
    for f in &row.flags {
      let (base, detail) = f.split_once(':').unwrap_or((f.as_str(), ""));
      let family = if base == "lost-elements" || f == "gained-elements:ERROR" {
        f.clone()
      } else if detail == "both" {
        format!("{base}:both")
      } else if detail.starts_with("ERROR") {
        format!("{base}:ERROR")
      } else {
        base.to_string()
      };
      let (tier, at) = crate::diff::rank(f);
      let k = (at, family);
      if !seen.contains(&k) {
        seen.push(k.clone());
        counts.entry(k).or_insert((0, tier)).0 += 1;
      }
    }
  }
  text.push_str(&format!("\npapers: {}\n", rows.len()));
  for (tier, n) in &tiers {
    text.push_str(&format!("tier {}: {n}\n", tier.name()));
  }
  text.push_str("\nflags (papers):\n");
  for ((_, family), (n, tier)) in &counts {
    text.push_str(&format!("  {family:<32} {n:>7}  [{}]\n", tier.name()));
  }
  let mut ranked: Vec<&Row> = rows.iter().filter(|r| r.tier != Some(Tier::Same)).collect();
  ranked.sort_by_key(|r| (r.tier, r.severity(), r.key.clone()));
  for tier in [Tier::Regression, Tier::Review, Tier::Change] {
    let of_tier: Vec<&&Row> = ranked.iter().filter(|r| r.tier == Some(tier)).collect();
    if of_tier.is_empty() {
      continue;
    }
    text.push_str(&format!(
      "\n{} ({}):\n",
      tier.name().to_uppercase(),
      of_tier.len()
    ));
    let limit = if tier == Tier::Change {
      200
    } else {
      usize::MAX
    };
    for row in of_tier.iter().take(limit) {
      text.push_str(&format!(
        "  {}\t{}\t{}\n",
        row.key,
        row.flags.join(","),
        row.cells.join(" | ")
      ));
    }
    if of_tier.len() > limit {
      text.push_str(&format!("  … {} more in the TSV\n", of_tier.len() - limit));
    }
  }
  fs::write(out.path(".summary.txt"), &text)?;
  Ok(text)
}

/// `compare A_root B_root`: rows for the union of both trees' documents (or `keys`). Returns whether any
/// row is a regression or needs review.
pub fn run_compare(
  a_root: &Path,
  b_root: &Path,
  keys: Option<&Path>,
  out: &Output,
  jobs: usize,
  extra_header: &[String],
) -> std::io::Result<bool> {
  require_dir(a_root)?;
  require_dir(b_root)?;
  let (found_a, found_b) = (discover(a_root, 3), discover(b_root, 3));
  let keys = match keys {
    Some(path) => read_keys(path)?,
    None => {
      let mut k = found_a.keys.clone();
      k.extend(found_b.keys.iter().cloned());
      k.sort();
      k.dedup();
      k
    },
  };
  if found_a.unrecognised.is_empty() && found_b.unrecognised.is_empty() {
    require_keys(&keys, "either tree")?;
  }
  let start = std::time::Instant::now();
  let mut rows = par_map(
    &keys,
    jobs,
    |key| compare(key, &load(a_root, key), &load(b_root, key)),
    |key| panic_row(key, COMPARE_COLUMNS.len()),
    progress(keys.len()),
  );
  add_unrecognised(&mut rows, &found_a, ":A", COMPARE_COLUMNS.len());
  add_unrecognised(&mut rows, &found_b, ":B", COMPARE_COLUMNS.len());
  let secs = start.elapsed().as_secs_f64();
  let mut header = vec![
    format!(
      "ab_audit compare  A={}  B={}",
      a_root.display(),
      b_root.display()
    ),
    format!("A {}", stamp_lines(a_root).join("  ")),
    format!("B {}", stamp_lines(b_root).join("  ")),
    unrecognised_line("A", &found_a),
    unrecognised_line("B", &found_b),
  ];
  header.extend(extra_header.iter().cloned());
  header.push(format!(
    "{} papers in {secs:.1} s at -j {jobs} ({:.1} papers/s)",
    keys.len(),
    keys.len() as f64 / secs.max(1e-9)
  ));
  write_rows(out, &header, COMPARE_COLUMNS, &rows)?;
  let totals = compare_totals(&rows);
  let text = summary(out, &header, &rows, &totals)?;
  print!("{}", text.lines().take(60).collect::<Vec<_>>().join("\n"));
  println!("\n… full summary: {}", out.path(".summary.txt").display());
  Ok(rows.iter().any(|r| r.tier <= Some(Tier::Review)))
}

/// Totals over the `a>b` count cells.
fn compare_totals(rows: &[Row]) -> Vec<String> {
  let mut lines = Vec::new();
  for (i, name) in ["errors", "fatals", "warnings"].iter().enumerate() {
    let (mut a, mut b) = (0u64, 0u64);
    for row in rows {
      if let Some((x, y)) = row.cells.get(i + 1).and_then(|c| c.split_once('>')) {
        a += x.parse::<u64>().unwrap_or(0);
        b += y.parse::<u64>().unwrap_or(0);
      }
    }
    lines.push(format!("{name}: A={a} B={b}"));
  }
  lines
}

/// Flag every unrecognised directory: on its document's row when the other tree has one under that key,
/// else on a regression row of its own, so an unknown layout can never pass unseen.
fn add_unrecognised(rows: &mut Vec<Row>, found: &Discovery, tag: &str, columns: usize) {
  for dir in &found.unrecognised {
    let flag = format!("unrecognised{tag}");
    if let Some(row) = rows.iter_mut().find(|r| &r.key == dir) {
      row.flags.push(flag);
      row.finish();
      continue;
    }
    rows.push(Row {
      key:     dir.clone(),
      tier:    Some(Tier::Regression),
      flags:   vec![flag],
      cells:   vec!["-".to_string(); columns],
      details: "no XML, log or run record this tool knows, or below its depth limit\n".to_string(),
    });
  }
}

fn panic_row(key: &str, columns: usize) -> Row {
  let mut row = Row {
    key: key.to_string(),
    tier: Some(Tier::Regression),
    flags: vec!["extractor-panic".to_string()],
    ..Row::default()
  };
  row.cells = vec!["-".to_string(); columns];
  row
}

/// `audit root`: one row per document of one tree. Returns whether any row is a regression or needs review.
pub fn run_audit(
  root: &Path,
  keys: Option<&Path>,
  out: &Output,
  jobs: usize,
) -> std::io::Result<bool> {
  require_dir(root)?;
  let found = discover(root, 3);
  let keys = match keys {
    Some(path) => read_keys(path)?,
    None => found.keys.clone(),
  };
  if found.unrecognised.is_empty() {
    require_keys(&keys, "the tree")?;
  }
  let start = std::time::Instant::now();
  let mut rows = par_map(
    &keys,
    jobs,
    |key| audit(key, &load(root, key)),
    |key| panic_row(key, AUDIT_COLUMNS.len()),
    progress(keys.len()),
  );
  add_unrecognised(&mut rows, &found, "", AUDIT_COLUMNS.len());
  let secs = start.elapsed().as_secs_f64();
  let header = vec![
    format!("ab_audit audit  root={}", root.display()),
    stamp_lines(root).join("  "),
    unrecognised_line("root", &found),
    format!(
      "{} papers in {secs:.1} s at -j {jobs} ({:.1} papers/s)",
      keys.len(),
      keys.len() as f64 / secs.max(1e-9)
    ),
  ];
  write_rows(out, &header, AUDIT_COLUMNS, &rows)?;
  let mut status: BTreeMap<String, usize> = BTreeMap::new();
  for row in &rows {
    *status
      .entry(row.cells.first().cloned().unwrap_or_default())
      .or_default() += 1;
  }
  let totals = vec![format!(
    "status histogram: {}",
    status
      .iter()
      .map(|(s, n)| format!("{s}:{n}"))
      .collect::<Vec<_>>()
      .join(" ")
  )];
  let text = summary(out, &header, &rows, &totals)?;
  print!("{}", text.lines().take(40).collect::<Vec<_>>().join("\n"));
  println!("\n… full summary: {}", out.path(".summary.txt").display());
  Ok(rows.iter().any(|r| r.tier <= Some(Tier::Review)))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ab_audit_report_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[test]
  fn an_empty_or_missing_tree_is_an_error_not_a_pass() {
    let dir = scratch("empty");
    let out = Output { prefix: dir.join("out/cmp") };
    fs::create_dir_all(dir.join("a")).unwrap();
    fs::create_dir_all(dir.join("b")).unwrap();
    assert!(run_compare(&dir.join("a"), &dir.join("b"), None, &out, 1, &[]).is_err());
    assert!(run_compare(&dir.join("a"), &dir.join("nope"), None, &out, 1, &[]).is_err());
    assert!(run_audit(&dir.join("nope"), None, &out, 1).is_err());
    assert!(run_audit(&dir.join("a"), None, &out, 1).is_err());
    // An unrecognised directory is a regression row (exit 1), never a silent skip.
    fs::create_dir_all(dir.join("a/odd")).unwrap();
    fs::write(dir.join("a/odd/x.dat"), "").unwrap();
    assert!(run_audit(&dir.join("a"), None, &out, 1).unwrap());
    fs::remove_dir_all(dir.join("a/odd")).unwrap();
    let missing = run_audit(&dir.join("a"), Some(&dir.join("no-keys.txt")), &out, 1).unwrap_err();
    assert!(missing.to_string().contains("no-keys.txt"), "{missing}");
    fs::write(dir.join("keys.txt"), "# only a comment\n").unwrap();
    assert!(run_audit(&dir.join("a"), Some(&dir.join("keys.txt")), &out, 1).is_err());
    fs::remove_dir_all(&dir).unwrap();
  }

  #[test]
  fn keys_are_read_as_written() {
    let dir = scratch("keys");
    fs::write(
      dir.join("k.txt"),
      "bundle/1234567\nhep-th/9703192\tx\n1001.0155\n1001.0155\n",
    )
    .unwrap();
    assert_eq!(read_keys(&dir.join("k.txt")).unwrap(), vec![
      "bundle/1234567".to_string(),
      "hep-th/9703192".to_string(),
      "1001.0155".to_string()
    ]);
    fs::remove_dir_all(&dir).unwrap();
  }
}

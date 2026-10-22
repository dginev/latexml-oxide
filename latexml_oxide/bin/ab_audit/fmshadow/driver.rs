//! The driver: per paper the comparison (re-resolved from the dump's events under `--re-resolve`), the per-paper and
//! per-disagreement tables and the per-family summary.

use super::*;

/// The reasons a block went unrehearsed that leave its names unseen (`shadow.rs` `SkipReason::name`): out of positions,
/// a Fatal of its own, its assignments not undone, no journal at any flush — not an empty argument, nor a call no flush
/// came after.
pub(super) const UNOBSERVED: &[&str] = &[
  "unstamped: out of positions",
  "rehearsal ended in a Fatal error",
  "rehearsal could not be undone",
  "journal unavailable",
];

/// One paper's result.
#[derive(Debug, Clone, Default)]
pub struct PaperRow {
  pub key:           String,
  /// `agree`, `disagree`, `no-block`, `held`, `unobserved` (an author block the rehearsal could not see:
  /// [`UNOBSERVED`]), `missing` (no dump or no XML), `unreadable`
  pub status:        String,
  pub family:        String,
  pub blocks:        usize,
  pub disagreements: Vec<Disagreement>,
  /// the shadow's tuples, for `<prefix>.shadow.jsonl` (disagreeing papers: the PDF checks read them)
  pub shadow:        Option<Tuples>,
}

impl PaperRow {
  fn unclassified(&self) -> bool { self.disagreements.iter().any(|d| !d.classified) }
}

/// Compare one paper directory (`out.xml` and `out.xml.fmshadow.json`).
pub fn compare_paper(root: &Path, key: &str, re_resolve: bool) -> PaperRow {
  let dir = root.join(key);
  let mut row = PaperRow {
    key: key.to_string(),
    ..PaperRow::default()
  };
  let (Ok(json), Ok(xml)) = (
    fs::read_to_string(dir.join("out.xml.fmshadow.json")),
    fs::read(dir.join("out.xml")),
  ) else {
    row.status = "missing".into();
    return row;
  };
  let Ok(dump) = serde_json::from_str::<Value>(&json) else {
    row.status = "unreadable".into();
    return row;
  };
  let blocks: Vec<&Value> = dump["entries"]
    .as_array()
    .into_iter()
    .flatten()
    .filter(|e| e["kind"] == "author" && e["status"] == "events")
    .collect();
  row.blocks = blocks.len();
  row.family = blocks
    .first()
    .map(|b| {
      // (`latexml.sty`: a dump of a binary that let the binding loading the kernel's pool own the kernel `\author`)
      let binding = s(&b["binding"]);
      if binding.is_empty() || binding == "latexml.sty" {
        "kernel".to_string()
      } else {
        binding
      }
    })
    .unwrap_or_else(|| "none".into());
  if blocks.is_empty() {
    row.status = "no-block".into();
    return row;
  }
  if blocks.iter().any(|b| b["errors"].as_u64().unwrap_or(0) > 0) {
    row.status = "held".into();
    return row;
  }
  // (a block the rehearsal could not see — out of positions, BESIII's 600-name `\author`, 2012.14750 — leaves the
  // shadow without its names, which the XML has: outside the gate's denominator, as a held one)
  if dump["entries"].as_array().into_iter().flatten().any(|e| {
    e["kind"] == "author"
      && e["status"] == "skipped"
      && UNOBSERVED.contains(&e["reason"].as_str().unwrap_or(""))
  }) {
    row.status = "unobserved".into();
    return row;
  }
  let resolved = if re_resolve {
    fm_resolve(&json_inputs(&dump))
  } else {
    json_resolved(&dump["resolved"])
  };
  let xml = match xml_tuples(&xml) {
    Ok(t) => t,
    Err(_) => {
      row.status = "unreadable".into();
      return row;
    },
  };
  let shadow = shadow_tuples(&resolved);
  row.disagreements = compare(&shadow, &xml);
  row.status = if row.disagreements.is_empty() {
    "agree"
  } else {
    "disagree"
  }
  .into();
  if !row.disagreements.is_empty() {
    row.shadow = Some(shadow);
  }
  row
}

/// Run the comparison over `root` and write the reports.
pub fn run(
  root: &Path,
  keys: Option<&Path>,
  prefix: &Path,
  jobs: usize,
  re_resolve: bool,
) -> std::io::Result<i32> {
  let keys: Vec<String> = match keys {
    Some(path) => fs::read_to_string(path)?
      .lines()
      .map(|l| l.split('\t').next().unwrap_or("").trim().replace('/', ""))
      .filter(|l| !l.is_empty())
      .collect(),
    None => discover(root, 2).keys,
  };
  let rows = par_map(
    &keys,
    jobs,
    |key| compare_paper(root, key, re_resolve),
    |key| PaperRow {
      key: key.clone(),
      status: "panic".into(),
      ..PaperRow::default()
    },
    |_| {},
  );
  let path = |suffix: &str| {
    let mut p = prefix.as_os_str().to_owned();
    p.push(suffix);
    PathBuf::from(p)
  };
  let mut tsv = fs::File::create(path(".tsv"))?;
  writeln!(
    tsv,
    "key\tfamily\tstatus\tblocks\tunclassified\tclassified\tkinds\trules"
  )?;
  let mut dis = fs::File::create(path(".disagree.tsv"))?;
  let mut shadow_out = fs::File::create(path(".shadow.jsonl"))?;
  writeln!(dis, "key\tfamily\tkind\tclassified\trule\tdetail\tclass")?;
  #[derive(Default)]
  struct Fam {
    papers:       usize,
    blocks:       usize,
    agree:        usize,
    classified:   usize,
    unclassified: usize,
    no_block:     usize,
    held:         usize,
    unobserved:   usize,
    other:        usize,
  }
  let mut fams: BTreeMap<String, Fam> = BTreeMap::new();
  let mut kinds: BTreeMap<(String, String, bool), (usize, usize)> = BTreeMap::new();
  for row in &rows {
    let un = row.disagreements.iter().filter(|d| !d.classified).count();
    let cl = row.disagreements.len() - un;
    let mut k: Vec<&str> = row.disagreements.iter().map(|d| d.kind).collect();
    k.sort_unstable();
    k.dedup();
    let mut r: Vec<&str> = row.disagreements.iter().map(|d| d.rule.as_str()).collect();
    r.sort_unstable();
    r.dedup();
    writeln!(
      tsv,
      "{}\t{}\t{}\t{}\t{un}\t{cl}\t{}\t{}",
      row.key,
      row.family,
      row.status,
      row.blocks,
      k.join(","),
      r.join(";")
    )?;
    let mut seen: Vec<(String, String, bool)> = Vec::new();
    for d in &row.disagreements {
      writeln!(
        dis,
        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
        row.key,
        row.family,
        d.kind,
        d.classified,
        d.rule,
        d.detail.replace(['\t', '\n'], " "),
        d.class.unwrap_or("")
      )?;
      let key = (
        d.kind.to_string(),
        d.rule
          .trim_start_matches("moved ")
          .trim_start_matches("overlaps ")
          .to_string(),
        d.classified,
      );
      let entry = kinds.entry(key.clone()).or_default();
      entry.1 += 1;
      if !seen.contains(&key) {
        entry.0 += 1;
        seen.push(key);
      }
    }
    if let Some(shadow) = &row.shadow {
      let creators: Vec<Value> = shadow
        .creators
        .iter()
        .map(|c| {
          serde_json::json!({
            "name": c.name,
            "affiliations": c.affiliations,
            "emails": c.contacts.iter().filter(|(r, _)| r == "email").map(|(_, t)| t.clone()).collect::<Vec<_>>(),
            "contacts": c.contacts,
            "notes": c.notes,
            "rules": c.rules,
          })
        })
        .collect();
      writeln!(
        shadow_out,
        "{}",
        serde_json::json!({ "key": row.key, "creators": creators, "block": shadow.block })
      )?;
    }
    let fam = fams.entry(row.family.clone()).or_default();
    fam.papers += 1;
    fam.blocks += row.blocks;
    match row.status.as_str() {
      "agree" => fam.agree += 1,
      "disagree" if row.unclassified() => fam.unclassified += 1,
      "disagree" => fam.classified += 1,
      "no-block" => fam.no_block += 1,
      "held" => fam.held += 1,
      "unobserved" => fam.unobserved += 1,
      _ => fam.other += 1,
    }
  }
  let mut summary = fs::File::create(path(".summary.txt"))?;
  writeln!(
    summary,
    "ab_audit fmshadow root={} papers={} re_resolve={re_resolve}",
    root.display(),
    rows.len()
  )?;
  writeln!(
    summary,
    "family\tpapers\tblocks\tagree\tdisagree_classified\tdisagree_unclassified\tno_block\theld\tunobserved\tother\tgate_unclassified_pct"
  )?;
  let mut ranked: Vec<(&String, &Fam)> = fams.iter().collect();
  ranked.sort_by(|a, b| b.1.papers.cmp(&a.1.papers).then(a.0.cmp(b.0)));
  let mut total = Fam::default();
  for (name, f) in &ranked {
    let counted = f.agree + f.classified + f.unclassified;
    let pct = if counted > 0 {
      100.0 * f.unclassified as f64 / counted as f64
    } else {
      0.0
    };
    writeln!(
      summary,
      "{name}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{pct:.1}",
      f.papers,
      f.blocks,
      f.agree,
      f.classified,
      f.unclassified,
      f.no_block,
      f.held,
      f.unobserved,
      f.other
    )?;
    total.papers += f.papers;
    total.blocks += f.blocks;
    total.agree += f.agree;
    total.classified += f.classified;
    total.unclassified += f.unclassified;
    total.no_block += f.no_block;
    total.held += f.held;
    total.unobserved += f.unobserved;
    total.other += f.other;
  }
  let counted = total.agree + total.classified + total.unclassified;
  writeln!(
    summary,
    "TOTAL\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.1}",
    total.papers,
    total.blocks,
    total.agree,
    total.classified,
    total.unclassified,
    total.no_block,
    total.held,
    total.unobserved,
    total.other,
    if counted > 0 {
      100.0 * total.unclassified as f64 / counted as f64
    } else {
      0.0
    }
  )?;
  writeln!(summary, "\nxml_wrong_class\tpapers\twitnesses")?;
  for class in XML_WRONG {
    let papers = rows
      .iter()
      .filter(|r| r.disagreements.iter().any(|d| d.class == Some(class.name)))
      .count();
    writeln!(
      summary,
      "{}\t{papers}\t{}",
      class.name,
      class.witnesses.join(" ")
    )?;
  }
  writeln!(summary, "\nkind\trule\tclassified\tpapers\toccurrences")?;
  let mut kinds: Vec<_> = kinds.into_iter().collect();
  kinds.sort_by_key(|a| std::cmp::Reverse(a.1));
  for ((kind, rule, classified), (papers, n)) in kinds {
    writeln!(summary, "{kind}\t{rule}\t{classified}\t{papers}\t{n}")?;
  }
  Ok(i32::from(total.unclassified > 0))
}

//! `<prefix>.outcomes.txt`: who decides each paper's frontmatter today, per family (E0;
//! `docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, definition of done items 1, 2), read from each dump's `readback`
//! (`shadow.rs` `json_read_back`). The old engine goes before the merge, so every count here but `same` and `written` is
//! a debt on the way to deleting it: a paper the old engine decided (`declined`, `no-block`, `held-or-skipped`) or one a
//! pass after the writer changed (the `changed_after` column).
//!
//! The list of reasons is closed: a decline's or a held paper's reason outside [`DECLINE_REASONS`] and [`STAY_OUT_REASONS`]
//! fails the run, as does one `written-different` paper. A new `declined(…)` in `writer/` fails
//! `every_decline_in_the_writer_is_in_the_closed_list` until it is listed here, with what pays it down.

use std::{collections::BTreeMap, fmt::Write as _, fs, path::Path};

use serde_json::Value;

use super::driver::PaperRow;

/// A paper's outcomes, in the table's order.
pub(super) const OUTCOMES: [&str; 7] = [
  "no-reading",
  "no-block",
  "held-or-skipped",
  "same",
  "written",
  "written-different",
  "declined",
];

/// A closed decline reason: the head of the writer's own text (before its first `:`), what the plan's definition of done
/// calls it, and what pays it down: `stays` (the definition's own closed list, the fallbacks the merge keeps), a batch of
/// the plan's, or `open` (a refusal no batch has taken yet).
pub(super) const DECLINE_REASONS: &[(&str, &str, &str)] = &[
  (
    "a name of today's the reading makes none",
    "today's name dropped",
    "stays",
  ),
  (
    "an item given more authors by no ruling",
    "more authors by no ruling",
    "stays",
  ),
  (
    "an item read from its words alone",
    "read from words alone",
    "stays",
  ),
  (
    "a creator of today's made an item on no structural evidence",
    "read from words alone",
    "stays",
  ),
  (
    "a person made an item on no structural evidence",
    "read from words alone",
    "stays",
  ),
  (
    "a creator of today's made a block item on no structural evidence",
    "read from words alone",
    "stays",
  ),
  ("a conceded creator's ink", "read from words alone", "stays"),
  ("a creator with no name", "no name", "stays"),
  ("no named creator", "no name", "stays"),
  // (the re-part from tokens: `repart.rs`)
  (
    "a name today's names do not print",
    "re-part: from tokens",
    "E3",
  ),
  (
    "an item with nothing to write it from",
    "re-part: from tokens",
    "E3",
  ),
  (
    "a block item with nothing to write it from",
    "re-part: from tokens",
    "E3",
  ),
  // (per-item writes: an unexplained item stays as today's instead of declining the paper, `plan.rs`)
  ("names differ", "plan: per item", "E4"),
  ("an owner change no ruling decided", "plan: per item", "E4"),
  ("a difference no operation explains", "plan: per item", "E4"),
  ("no item to copy", "plan: per item", "E4"),
  ("an owner added by no ruling", "plan: per item", "E4"),
  ("an item to remove that cannot move", "plan: per item", "E4"),
  (
    "an address added beside the creator it spells",
    "plan: per item",
    "E4",
  ),
  (
    "an item printing a later mark its creator requests",
    "plan: per item",
    "E4",
  ),
  (
    "an affiliation spelling an author's name",
    "plan: per item",
    "E4",
  ),
  (
    "an affiliation printing another author's name",
    "plan: per item",
    "E4",
  ),
  ("no API call", "plan: per item", "E4"),
  ("a cut entry of another role", "plan: per item", "E4"),
  ("the entry does not cut", "plan: per item", "E4"),
  (
    "today's item is not the pieces joined",
    "plan: per item",
    "E4",
  ),
  ("a piece the reading lacks", "plan: per item", "E4"),
  ("a contact piece", "plan: per item", "E4"),
  ("a legend of no API call", "plan: per item", "E4"),
  ("a url's owners differ", "plan: per item", "E4"),
  // (the re-part's other refusals, and what an unwritten family holds back: no batch has taken them yet)
  (
    "a name whose brackets do not close",
    "re-part: the reading's name",
    "open",
  ),
  (
    "a name naming no person",
    "re-part: the reading's name",
    "open",
  ),
  (
    "a name joined across an alignment's cells",
    "re-part: the reading's name",
    "open",
  ),
  (
    "a one-word name parted at a comma",
    "re-part: the reading's name",
    "open",
  ),
  (
    "a name cut from a row on no structure",
    "re-part: the reading's name",
    "open",
  ),
  ("a name with no run", "re-part: the reading's name", "open"),
  (
    "an item whose brackets do not close",
    "re-part: the reading's item",
    "open",
  ),
  (
    "a block e-mail no name spells",
    "re-part: the reading's item",
    "open",
  ),
  (
    "the re-part did not read as the reading",
    "re-part: did not read back",
    "open",
  ),
  (
    "an unwritten family's call beside a re-part",
    "an unwritten family",
    "open",
  ),
  (
    "an unwritten family's items kept as today's",
    "an unwritten family",
    "open",
  ),
];

/// Why the writer stayed out of a paper with a reading (`writer::writer_stays_out`): the blocks' `SkipReason`s, held
/// diagnostics, and the two reasons of the ledger's own.
pub(super) const STAY_OUT_REASONS: &[&str] = &[
  "unstamped: out of positions",
  "no flush after the call",
  "journal unavailable",
  "rehearsal ended in a Fatal error",
  "rehearsal could not be undone",
  "held diagnostics",
  "a family not written",
  "an API call not rehearsed",
  "the writer did not run",
];

/// A reason's head, before its first `:`.
pub(super) fn reason_head(reason: &str) -> &str {
  reason.split(':').next().unwrap_or(reason).trim_end()
}

/// One paper's outcome, from its dump.
#[derive(Debug, Clone, Default)]
pub struct PaperOutcome {
  /// one of [`OUTCOMES`], or `no-readback` (a dump of a binary before E0), or what an unknown writer state read
  pub outcome:     String,
  /// a decline's or a held paper's reason, a `written-different` paper's difference; empty for the rest
  pub reason:      String,
  /// the family as the resolver names it: the first of its policies' names, sorted, `no-block` without one
  pub family:      String,
  /// the read-back ran and found the final document reading as the reading; `None` where it never ran
  pub reads_back:  Option<bool>,
  /// the blocks' skip reasons (`summary.skipped`) and its blocks whose rehearsal held diagnostics (`summary.held`)
  pub skipped:     Vec<(String, u64)>,
  pub held_blocks: u64,
}

impl PaperOutcome {
  /// Read from the dump's `readback`, `resolved.policies` and `summary`.
  pub(super) fn of(dump: &Value) -> PaperOutcome {
    let text = |v: &Value| v.as_str().unwrap_or("").to_string();
    let read_back = &dump["readback"];
    let mut families: Vec<&str> = dump["resolved"]["policies"]
      .as_array()
      .into_iter()
      .flatten()
      .filter_map(|p| p[1].as_str())
      .collect();
    families.sort_unstable();
    PaperOutcome {
      outcome:     match read_back["outcome"].as_str() {
        Some(outcome) => outcome.to_string(),
        None => "no-readback".to_string(),
      },
      reason:      text(&read_back["reason"]),
      family:      families.first().copied().unwrap_or("no-block").to_string(),
      reads_back:  read_back["reached"]
        .as_bool()
        .filter(|reached| *reached)
        .map(|_| read_back["difference"].is_null()),
      skipped:     dump["summary"]["skipped"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(reason, n)| (reason.clone(), n.as_u64().unwrap_or(0)))
        .collect(),
      held_blocks: dump["summary"]["held"].as_u64().unwrap_or(0),
    }
  }
}

/// One family's counts.
#[derive(Debug, Default, Clone)]
struct Counts {
  papers:        usize,
  /// by [`OUTCOMES`]' order
  outcomes:      [usize; 7],
  /// `same` or `written`, and `ab_audit fmshadow`'s comparator agrees with the final XML
  whole_same:    usize,
  whole_written: usize,
  /// `same` or `written`, and the comparator disagrees with the final XML
  part:          usize,
  /// `same` or `written`, and the read-back found the final document differing from the reading: a pass after the writer changed a kind, a text or (E6's debt) an owner
  changed_after: usize,
  /// a paper with no dump, or none this table can read
  no_dump:       usize,
  no_readback:   usize,
  unknown:       usize,
}

impl Counts {
  fn add(&mut self, row: &PaperRow) {
    self.papers += 1;
    let o = &row.outcome;
    match o.outcome.as_str() {
      "" => self.no_dump += 1,
      "no-readback" => self.no_readback += 1,
      outcome => match OUTCOMES.iter().position(|known| *known == outcome) {
        Some(at) => self.outcomes[at] += 1,
        None => self.unknown += 1,
      },
    }
    let writer_holds = matches!(o.outcome.as_str(), "same" | "written");
    if writer_holds {
      match (o.outcome.as_str(), row.status.as_str()) {
        ("same", "agree") => self.whole_same += 1,
        ("written", "agree") => self.whole_written += 1,
        (_, "disagree") => self.part += 1,
        _ => {},
      }
      if o.reads_back == Some(false) {
        self.changed_after += 1;
      }
    }
  }

  fn cells(&self) -> String {
    let mut cells = self.papers.to_string();
    for n in self.outcomes {
      let _ = write!(cells, "\t{n}");
    }
    let _ = write!(
      cells,
      "\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
      self.whole_same,
      self.whole_written,
      self.part,
      self.changed_after,
      self.no_dump,
      self.no_readback,
      self.unknown
    );
    cells
  }
}

/// What the run's exit reads.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OutcomeGate {
  pub written_different: usize,
  /// reasons outside the closed lists, and outcomes no reader knows
  pub unknown:           usize,
  /// dumps with no `readback`, a binary before E0: nothing to read
  pub no_readback:       usize,
  /// papers with an outcome
  pub counted:           usize,
}

impl OutcomeGate {
  /// A defect whatever else the run reports.
  pub fn defect(&self) -> bool { self.written_different > 0 || self.unknown > 0 }

  /// `--outcomes`: the defects, and a run that read no outcome or a dump that has none, never a clean 0.
  pub fn fails(&self) -> bool { self.defect() || self.no_readback > 0 || self.counted == 0 }
}

/// Write `path` and read the gate.
pub(super) fn write_outcomes(
  rows: &[PaperRow],
  root: &Path,
  path: &Path,
) -> std::io::Result<OutcomeGate> {
  let mut families: BTreeMap<&str, Counts> = BTreeMap::new();
  let mut total = Counts::default();
  // (head, class, debt) → papers; the texts outside the list
  let mut declines: BTreeMap<&str, usize> = BTreeMap::new();
  let mut stays_out: BTreeMap<&str, usize> = BTreeMap::new();
  let mut skipped: BTreeMap<&str, u64> = BTreeMap::new();
  let mut held_blocks = 0u64;
  let mut outside: BTreeMap<String, usize> = BTreeMap::new();
  let mut different: Vec<(&str, &str)> = Vec::new();
  for row in rows {
    let o = &row.outcome;
    let family = if o.family.is_empty() {
      "no-dump"
    } else {
      o.family.as_str()
    };
    families.entry(family).or_default().add(row);
    total.add(row);
    for (reason, n) in &o.skipped {
      *skipped.entry(reason).or_default() += n;
    }
    held_blocks += o.held_blocks;
    match o.outcome.as_str() {
      "declined" => match DECLINE_REASONS
        .iter()
        .find(|(head, ..)| *head == reason_head(&o.reason))
      {
        Some((head, ..)) => *declines.entry(head).or_default() += 1,
        None => *outside.entry(s_or_none(&o.reason)).or_default() += 1,
      },
      "held-or-skipped" => match STAY_OUT_REASONS.iter().find(|known| **known == o.reason) {
        Some(known) => *stays_out.entry(known).or_default() += 1,
        None => *outside.entry(s_or_none(&o.reason)).or_default() += 1,
      },
      "written-different" => different.push((&row.key, &o.reason)),
      "" | "no-readback" => {},
      outcome if !OUTCOMES.contains(&outcome) => {
        *outside.entry(format!("outcome {outcome}")).or_default() += 1;
      },
      _ => {},
    }
  }
  let mut out = String::new();
  let _ = writeln!(
    out,
    "ab_audit fmshadow outcomes root={} papers={}",
    root.display(),
    rows.len()
  );
  let _ = writeln!(
    out,
    "family\tpapers\t{}\twhole_same\twhole_written\tpart\tchanged_after\tno_dump\tno_readback\tunknown",
    OUTCOMES.join("\t")
  );
  let mut ranked: Vec<(&&str, &Counts)> = families.iter().collect();
  ranked.sort_by(|a, b| b.1.papers.cmp(&a.1.papers).then(a.0.cmp(b.0)));
  for (family, counts) in ranked {
    let _ = writeln!(out, "{family}\t{}", counts.cells());
  }
  let _ = writeln!(out, "TOTAL\t{}", total.cells());
  let counted: usize = total.outcomes.iter().sum();
  let pct = |n: usize| 100.0 * n as f64 / counted.max(1) as f64;
  let mut cells = String::from("%\t");
  for n in total.outcomes {
    let _ = write!(cells, "\t{:.2}", pct(n));
  }
  for n in [
    total.whole_same,
    total.whole_written,
    total.part,
    total.changed_after,
  ] {
    let _ = write!(cells, "\t{:.2}", pct(n));
  }
  let _ = writeln!(out, "{cells}\t\t\t");
  // (each a count on the way to zero, but `stays`: the plan's closed list of fallbacks)
  let _ = writeln!(
    out,
    "\ndecline_reason\tpapers\tclass\tpaid_by (a debt to zero, but stays)"
  );
  let mut listed: Vec<(&&str, &usize)> = declines.iter().collect();
  listed.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
  for (head, n) in listed {
    let (_, class, debt) = DECLINE_REASONS
      .iter()
      .find(|(known, ..)| known == head)
      .expect("a listed reason");
    let _ = writeln!(out, "{head}\t{n}\t{class}\t{debt}");
  }
  let mut by_debt: BTreeMap<&str, usize> = BTreeMap::new();
  for (head, n) in &declines {
    let (.., debt) = DECLINE_REASONS
      .iter()
      .find(|(known, ..)| known == head)
      .expect("a listed reason");
    *by_debt.entry(debt).or_default() += n;
  }
  let _ = writeln!(out, "\ndeclined_by_debt\tpapers");
  for (debt, n) in &by_debt {
    let _ = writeln!(out, "{debt}\t{n}");
  }
  let _ = writeln!(out, "\nstay_out_reason\tpapers");
  for (reason, n) in &stays_out {
    let _ = writeln!(out, "{reason}\t{n}");
  }
  let _ = writeln!(out, "\nskip_reason (blocks)\tblocks");
  for (reason, n) in &skipped {
    let _ = writeln!(out, "{reason}\t{n}");
  }
  let _ = writeln!(out, "held diagnostics (blocks)\t{held_blocks}");
  let unknown = outside.values().sum::<usize>();
  let _ = writeln!(out, "\noutside_the_closed_list\tpapers");
  for (reason, n) in &outside {
    let _ = writeln!(out, "{}\t{n}", reason.replace(['\t', '\n'], " "));
  }
  let _ = writeln!(out, "\nwritten_different\tdifference");
  for (key, difference) in &different {
    let _ = writeln!(out, "{key}\t{}", difference.replace(['\t', '\n'], " "));
  }
  let gate = OutcomeGate {
    written_different: different.len(),
    unknown,
    no_readback: total.no_readback,
    counted,
  };
  let _ = writeln!(
    out,
    "\ngate written_different={} outside_the_list={} no_readback={} counted={} fails={}",
    gate.written_different,
    gate.unknown,
    gate.no_readback,
    gate.counted,
    gate.fails()
  );
  fs::write(path, out)?;
  Ok(gate)
}

/// A reason's text, or the name of its absence.
fn s_or_none(reason: &str) -> String {
  if reason.is_empty() {
    "(no reason recorded)".to_string()
  } else {
    reason_head(reason).to_string()
  }
}

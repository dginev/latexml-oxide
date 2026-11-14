//! One paper, two conversions (`compare`), or one conversion (`audit`): the flagged row.
//!
//! It fails toward flagging (CLAUDE.md "Canvas signal integrity"): a document missing on either side, an
//! unparsable XML, a log without its `Status:conversion:N` line, a timeout or an extractor panic is a
//! regression row, never a pass.

use std::collections::BTreeMap;

use rustc_hash::FxHashMap;

use crate::{
  canon::{Canon, canonical, diff as canon_diff},
  facts::{
    BOUNDARY, LogFacts, XmlFacts, collapse, numbers, numbers_with_digits, parse, read_log,
    read_xml, strip_ansi, tolerance,
  },
  front::diff_front,
  tree::{DocInput, RunMeta},
};

/// Severity tiers, most severe first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
  Regression = 1,
  Review = 2,
  Change = 3,
  Same = 4,
}

impl Tier {
  pub fn name(self) -> &'static str {
    match self {
      Tier::Regression => "regression",
      Tier::Review => "review",
      Tier::Change => "change",
      Tier::Same => "same",
    }
  }
}

/// Flags in severity order; a flag `prefix:detail` ranks by its prefix. The first block is a regression,
/// the second needs review (a reason), the third is a change (an improvement or markup-only).
const REGRESSION: &[&str] = &[
  "unrecognised",
  "doc-missing",
  "input-missing",
  "extractor-panic",
  "xml-missing",
  "xml-unparsable",
  "log-missing",
  "no-status",
  "status-unparsable",
  "timeout",
  "oom",
  "crash",
  "contract-violation",
  "fatal-in-stderr",
  "fatal+",
  "fatal",
  "fatal-midline",
  "fatal-midline+",
  "status-worse",
  "errors+",
  "error-midline",
  "error-midline+",
  "errors",
  "error-elements",
  "creator-lost",
  "creator-nameless",
  "contact-emptied",
  "contact-empty",
  "contact-lost",
  "contact-split",
  "contact-link-fragment",
  "title-lost",
  "abstract-lost",
  "keywords-lost",
  "geometry-collapsed",
  "lost-elements",
  "gained-elements:ERROR",
  "lost-text",
  "leaked-dimension",
  // The same problem on both sides (`xml-unparsable:both`): unverifiable, so still a regression row, but
  // ranked after every row that B made worse.
  "both",
];
const REVIEW: &[&str] = &[
  "warnings+",
  "diag-classes-changed",
  "status-diag-mismatch",
  "diag-midline",
  "creator-changed",
  "creator-added",
  "contact-link-changed",
  "attrs-changed",
  "wrappers-lost",
  "space-lost",
  "text-jitter",
  "gained-number",
  "title-changed",
  "abstract-changed",
  "keywords-changed",
  "ref-retargeted",
  "moved-text",
  "gained-text",
  "slower",
  "xml-gained",
  "xml-only",
];

/// A flag's tier and its rank within all flags (lower is more severe).
pub fn rank(flag: &str) -> (Tier, usize) {
  // The same problem on both sides keeps the problem's tier; a regression one ranks after every row that B
  // made worse.
  if let Some(problem) = flag.strip_suffix(":both") {
    return match rank(problem) {
      (Tier::Regression, _) => (Tier::Regression, REGRESSION.len() - 1),
      other => other,
    };
  }
  let (base, detail) = flag.split_once(':').unwrap_or((flag, ""));
  // Text or attributes leaving an `ERROR` (an undefined macro now defined) is not a regression.
  if matches!(base, "lost-text" | "moved-text" | "attrs-changed") && detail.starts_with("ERROR") {
    return (Tier::Change, REGRESSION.len() + REVIEW.len());
  }
  let base = if flag == "gained-elements:ERROR" {
    flag
  } else {
    base
  };
  if let Some(i) = REGRESSION.iter().position(|f| *f == base || *f == flag) {
    (Tier::Regression, i)
  } else if let Some(i) = REVIEW.iter().position(|f| *f == base) {
    (Tier::Review, REGRESSION.len() + i)
  } else {
    (Tier::Change, REGRESSION.len() + REVIEW.len())
  }
}

/// A finished row.
#[derive(Debug, Default, Clone)]
pub struct Row {
  pub key:     String,
  pub tier:    Option<Tier>,
  pub flags:   Vec<String>,
  /// Already formatted TSV cells after `key`, `tier`, `flags`.
  pub cells:   Vec<String>,
  /// The detail text for the per-paper file.
  pub details: String,
}

impl Row {
  /// Sort, dedup and tier the flags.
  pub fn finish(&mut self) {
    self.flags.sort_by_key(|f| (rank(f).1, f.clone()));
    self.flags.dedup();
    self.tier = Some(
      self
        .flags
        .iter()
        .map(|f| rank(f).0)
        .min()
        .unwrap_or(Tier::Same),
    );
  }
  pub fn severity(&self) -> usize { self.flags.first().map(|f| rank(f).1).unwrap_or(usize::MAX) }
}

/// What one side of a paper yields after reading.
struct Side {
  log:           Option<LogFacts>,
  xml:           Result<XmlFacts, String>,
  /// The canonical lines, when asked for (two documents that differ).
  canon:         Option<Canon>,
  meta:          Option<RunMeta>,
  present:       bool,
  /// The diagnostics of the captured stderr/stdout, read when the log is missing or unfinished or the run
  /// was cut short (see `DocInput::stderr`).
  stderr:        Option<LogFacts>,
  /// Its first `Fatal:` lines, for the detail file.
  stderr_fatals: Vec<String>,
}

/// Read one side, parsing its XML once for the facts and, with `want_canon`, the canonical lines.
fn read_side(input: &DocInput, want_canon: bool, reuse: Option<&Result<XmlFacts, String>>) -> Side {
  let mut canon = None;
  let xml = match (&input.xml, reuse) {
    (_, Some(xml)) => xml.clone(),
    (None, None) => Err("missing".to_string()),
    (Some(bytes), None) => parse(bytes).map(|doc| {
      if want_canon {
        canon = Some(canonical(&doc));
      }
      read_xml(&doc)
    }),
  };
  Side {
    log: input.log.as_deref().map(read_log),
    xml,
    canon,
    meta: input.meta.clone(),
    present: input.present,
    stderr: input.stderr.as_deref().map(read_log),
    stderr_fatals: input
      .stderr
      .as_deref()
      .map(|t| {
        t.lines()
          .map(|l| strip_ansi(l).into_owned())
          .filter(|l| l.starts_with("Fatal:"))
          .take(5)
          .collect()
      })
      .unwrap_or_default(),
  }
}

/// The problems of one side that make its row a regression: missing pieces, unreadable XML, no status,
/// a timeout.
fn side_problems(side: &Side, tag: &str, flags: &mut Vec<String>) {
  if !side.present {
    flags.push(format!("doc-missing:{tag}"));
    return;
  }
  if side
    .meta
    .as_ref()
    .is_some_and(|m| m.note == "input-missing")
  {
    flags.push(format!("input-missing:{tag}"));
  }
  match &side.xml {
    Err(e) if e == "missing" => flags.push(format!("xml-missing:{tag}")),
    Err(_) => flags.push(format!("xml-unparsable:{tag}")),
    Ok(_) => {},
  }
  match &side.log {
    None => {
      flags.push(format!("log-missing:{tag}"));
      if side.stderr.as_ref().is_some_and(|e| e.fatals > 0) {
        flags.push(format!("fatal-in-stderr:{tag}"));
      }
    },
    Some(log) if log.status_bad => flags.push(format!("status-unparsable:{tag}")),
    Some(log) if log.status.is_none() => {
      flags.push(format!("no-status:{tag}"));
      if side.stderr.as_ref().is_some_and(|e| e.fatals > 0) {
        flags.push(format!("fatal-in-stderr:{tag}"));
      }
    },
    Some(log) => {
      if log.status_mismatch() {
        flags.push(format!("status-diag-mismatch:{tag}"));
      }
      // A `Fatal:`/`Error:` printed inside a line (after a `(Loading …` mark) is as real as one at a
      // line start, though cortex does not count it; a mid-line warning only needs a look.
      if log.midline_fatals > 0 {
        flags.push(format!("fatal-midline:{tag}"));
      }
      if log.midline_errors > 0 {
        flags.push(format!("error-midline:{tag}"));
      }
      if log.midline_warnings > 0 {
        flags.push(format!("diag-midline:{tag}"));
      }
    },
  }
  if side.meta.as_ref().is_some_and(|m| m.timed_out) {
    // Exit 137 with a `Fatal:oom` (the allocation-failure exit, `bin/latexml_oxide.rs` `alloc_error`) is
    // out of memory, not a timeout.
    let oom = |l: &LogFacts| l.classes.keys().any(|c| c.starts_with("Fatal:oom"));
    let is_oom = side.meta.as_ref().and_then(|m| m.rc) == Some(137)
      && (side.log.as_ref().is_some_and(oom) || side.stderr.as_ref().is_some_and(oom));
    flags.push(format!("{}:{tag}", if is_oom { "oom" } else { "timeout" }));
  }
  // A Fatal (status 3) must exit non-zero (`bin/latexml_oxide.rs`, Perl bin/latexml:151).
  if side.meta.as_ref().and_then(|m| m.rc) == Some(0)
    && side.log.as_ref().is_some_and(|l| l.status == Some(3))
  {
    flags.push(format!("contract-violation:{tag}"));
  }
  // An exit code that neither a clean run (0), a Fatal (1 with status 3: `bin/latexml_oxide.rs`, Perl
  // bin/latexml:151), nor a timeout (124 TERM, 137 KILL or the allocation-failure exit) explains: a
  // crash, a signal (-1), a refused start.
  let fatal_status = side.log.as_ref().is_some_and(|l| l.status == Some(3));
  if let Some(rc) = side.meta.as_ref().and_then(|m| m.rc)
    && !matches!(rc, 0 | 124 | 137)
    && !(rc == 1 && fatal_status)
  {
    flags.push(format!("crash:{tag}"));
  }
}

fn opt<T: ToString>(v: Option<T>) -> String {
  v.map(|v| v.to_string()).unwrap_or_else(|| "-".into())
}

fn pair<T: ToString>(a: Option<T>, b: Option<T>) -> String { format!("{}>{}", opt(a), opt(b)) }

/// The words of segments, a boundary taken as a space: a word that gains a footnote mark or an index
/// entry beside it is still the same word (spacing is compared apart, [`boundary_spacing`]).
fn tokens<'a>(segments: impl Iterator<Item = &'a String>) -> FxHashMap<&'a str, i64> {
  let mut m: FxHashMap<&str, i64> = FxHashMap::default();
  for s in segments {
    for t in s.split([' ', BOUNDARY]).filter(|t| !t.is_empty()) {
      *m.entry(t).or_default() += 1;
    }
  }
  m
}

/// The segments with each boundary a space, collapsed: what the words say, element positions aside.
fn words_only(segments: &[String]) -> Vec<String> {
  segments
    .iter()
    .map(|s| collapse(&s.replace(BOUNDARY, " ")))
    .collect()
}

/// Per boundary (a non-wrapper child inside the text), keyed by the words before and after it: whether it
/// is glued to the word before and to the word after. `see <Math/> and` is `(see, and) → (false, false)`,
/// `see<Math/> and` is `(see, and) → (true, false)`.
fn boundary_spacing(segments: &[String]) -> FxHashMap<(String, String), Vec<(bool, bool)>> {
  #[derive(PartialEq)]
  enum Piece<'a> {
    Word(&'a str),
    Space,
    Boundary,
  }
  let mut out: FxHashMap<(String, String), Vec<(bool, bool)>> = FxHashMap::default();
  for segment in segments {
    if !segment.contains(BOUNDARY) {
      continue;
    }
    let mut pieces = Vec::new();
    let mut word_start = None;
    for (i, c) in segment.char_indices() {
      if c == ' ' || c == BOUNDARY {
        if let Some(start) = word_start.take() {
          pieces.push(Piece::Word(&segment[start..i]));
        }
        pieces.push(if c == ' ' {
          Piece::Space
        } else {
          Piece::Boundary
        });
      } else if word_start.is_none() {
        word_start = Some(i);
      }
    }
    if let Some(start) = word_start {
      pieces.push(Piece::Word(&segment[start..]));
    }
    let word = |p: &Piece<'_>| match p {
      Piece::Word(w) => Some(w.to_string()),
      _ => None,
    };
    for (i, piece) in pieces.iter().enumerate() {
      if *piece != Piece::Boundary {
        continue;
      }
      let before = pieces[..i].iter().rev().find_map(word).unwrap_or_default();
      let after = pieces[i + 1..].iter().find_map(word).unwrap_or_default();
      let glued_before = i > 0 && matches!(pieces[i - 1], Piece::Word(_));
      let glued_after = matches!(pieces.get(i + 1), Some(Piece::Word(_)));
      out
        .entry((before, after))
        .or_default()
        .push((glued_before, glued_after));
    }
  }
  out
}

/// Whether a boundary present on both sides lost a space before or after it.
fn space_lost(a: &[String], b: &[String]) -> Vec<String> {
  // Per key, the totals of glued sides: one boundary splitting into two (`see<Math/><Math/>`, s1e) still
  // compares.
  let totals = |v: &Vec<(bool, bool)>| {
    (
      v.iter().filter(|x| x.0).count(),
      v.iter().filter(|x| x.1).count(),
    )
  };
  let (sa, sb) = (boundary_spacing(a), boundary_spacing(b));
  let mut lost = Vec::new();
  for (key, va) in &sa {
    if let Some(vb) = sb.get(key) {
      let ((ga, aa), (gb, ab)) = (totals(va), totals(vb));
      if gb > ga || ab > aa {
        lost.push(format!("{:?}␣{:?}", key.0, key.1));
      }
    }
  }
  lost.sort();
  lost
}

/// A token's pieces for the global-loss check: alphanumeric runs and single other characters, so
/// `foo,` that became `foo` + `,` (a run moved into its own element) loses nothing.
fn pieces(token: &str) -> Vec<&str> {
  let mut out = Vec::new();
  let mut run: Option<usize> = None;
  for (i, c) in token.char_indices() {
    if c.is_alphanumeric() {
      run.get_or_insert(i);
    } else {
      if let Some(start) = run.take() {
        out.push(&token[start..i]);
      }
      out.push(&token[i..i + c.len_utf8()]);
    }
  }
  if let Some(start) = run {
    out.push(&token[start..]);
  }
  out
}

/// The multiset of [`pieces`] of every token of some segments.
fn piece_counts<'a>(segments: impl Iterator<Item = &'a String>) -> FxHashMap<&'a str, i64> {
  let mut m: FxHashMap<&str, i64> = FxHashMap::default();
  for s in segments {
    for t in s.split([' ', BOUNDARY]).filter(|t| !t.is_empty()) {
      for p in pieces(t) {
        *m.entry(p).or_default() += 1;
      }
    }
  }
  m
}

/// Pair lost and gained tokens that differ only by numbers within [`JITTER`] of each other (`3.80254pt` and
/// `3.80252pt` in a `Math@tex`): removed from both, and counted.
fn take_jitter<'a>(lost: &mut BTreeMap<&'a str, i64>, gained: &mut BTreeMap<&'a str, i64>) -> i64 {
  let mut pairs = 0;
  let numeric: Vec<&str> = lost
    .keys()
    .copied()
    .filter(|t| t.bytes().any(|b| b.is_ascii_digit()))
    .collect();
  for l in numeric {
    let (nl, dl, sl) = numbers_with_digits(l);
    let partners: Vec<&str> = gained
      .keys()
      .copied()
      .filter(|g| {
        let (ng, dg, sg) = numbers_with_digits(g);
        // An equal pair always agrees; a differing one is rounding only between computed values printed
        // with at least 3 fractional digits on both sides (`3.80254pt`, mhchem `28.04677pt`/`28.04684pt`)
        // within [`JITTER`] and within ten units of the last printed place (so `2101.00001`, five places, is
        // not `2102.00001`). Everywhere, attributes included: `2101.00001`→`2102.00001` in an href,
        // `Smith2019`→`Smith2020`, `fig1000`→`fig1001` and `2019`→`2020` are changed content.
        sg == sl
          && ng.len() == nl.len()
          && (0..ng.len()).all(|k| {
            let (p, q) = (nl[k], ng[k]);
            let last_place = 10f64.powi(-(dl[k].min(dg[k]) as i32));
            p == q
              || (dl[k] >= 3
                && dg[k] >= 3
                && (p - q).abs() <= JITTER * p.abs().max(q.abs())
                && (p - q).abs() <= 10.0 * last_place)
          })
      })
      .collect();
    for g in partners {
      let n = lost
        .get(l)
        .copied()
        .unwrap_or(0)
        .min(gained.get(g).copied().unwrap_or(0));
      if n <= 0 {
        continue;
      }
      for (m, t) in [(&mut *lost, l), (&mut *gained, g)] {
        let left = m.get(t).copied().unwrap_or(0) - n;
        if left > 0 {
          m.insert(t, left);
        } else {
          m.remove(t);
        }
      }
      pairs += n;
    }
  }
  pairs
}

/// The relative difference below which two printed values are one value rounded differently (0.1 %):
/// `3.80254pt` and `3.80252pt` are, `0.001` and `0.005` are not.
const JITTER: f64 = 1e-3;

/// What a gained token is when it is a number: `Some(true)` a TeX dimension or a dotted register value
/// (`24pt`, `-36.83386pt`, `0.70.2`), `Some(false)` a bare number (`3`, `0.5`, `2003`), `None` other text.
/// Surrounding `()[],;:` and a trailing `.` are stripped first (`24pt,`, `(24pt)`, `2003.`).
fn number_kind(token: &str) -> Option<bool> {
  let t = token.trim_matches(|c| matches!(c, '(' | ')' | '[' | ']' | ',' | ';' | ':'));
  let t = t.strip_suffix('.').unwrap_or(t);
  let body = t.strip_prefix(['-', '+']).unwrap_or(t);
  let digits_end = body
    .find(|c: char| !c.is_ascii_digit() && c != '.')
    .unwrap_or(body.len());
  let (number, unit) = body.split_at(digits_end);
  if !number.bytes().any(|b| b.is_ascii_digit()) {
    return None;
  }
  let tex_unit = matches!(
    unit,
    "pt" | "em" | "ex" | "bp" | "mm" | "cm" | "in" | "sp" | "pc" | "dd" | "cc" | "mu"
  );
  match unit {
    _ if tex_unit => Some(true),
    "" => Some(number.matches('.').count() >= 2),
    _ => None,
  }
}

/// [`number_kind`] of a token, or else of its [`pieces`]: `heading,24pt` and `24pt\relax` hold a dimension.
/// A dotted section number (`1.2.3`) reads as a dotted register value: a false flag the rules accept, since
/// a section number newly typeset in prose is worth a look either way.
fn token_number_kind(token: &str) -> Option<bool> {
  number_kind(token).or_else(|| pieces(token).into_iter().filter_map(number_kind).max())
}

/// Classes whose numbers are not typeset prose: generated ones (equation and section tags, references,
/// citations) and code, typed by the author (`verbatim`, listings).
const NOT_PROSE_CLASSES: &[&str] = &[
  "tag",
  "ref",
  "bibref",
  "cite",
  "verbatim",
  "listing",
  "listingline",
];

/// `a - b` as a multiset, in a stable order.
fn minus<'a>(a: &FxHashMap<&'a str, i64>, b: &FxHashMap<&'a str, i64>) -> BTreeMap<&'a str, i64> {
  a.iter()
    .filter_map(|(t, n)| {
      let d = n - b.get(t).copied().unwrap_or(0);
      (d > 0).then_some((*t, d))
    })
    .collect()
}

fn show_tokens(m: &BTreeMap<&str, i64>, limit: usize) -> String {
  let mut parts: Vec<String> = m
    .iter()
    .take(limit)
    .map(|(t, n)| {
      if *n > 1 {
        format!("{t}×{n}")
      } else {
        t.to_string()
      }
    })
    .collect();
  if m.len() > limit {
    parts.push(format!("…+{}", m.len() - limit));
  }
  parts.join(" ")
}

/// Text by class: `lost-text:<class>` for tokens gone from the class and from the whole document,
/// `moved-text:<class>` for tokens that left the class (or a class reordered or resegmented),
/// `gained-text:<class>` for new tokens.
fn diff_text(
  a: &XmlFacts,
  b: &XmlFacts,
  flags: &mut Vec<String>,
  cell: &mut Vec<String>,
  details: &mut String,
) {
  let is_attr = |c: &&String| c.contains('@');
  // The document-wide multisets, by piece (`pieces`).
  fn all(f: &XmlFacts, attr: bool) -> FxHashMap<&str, i64> {
    piece_counts(
      f.text
        .iter()
        .filter(move |(c, _)| c.contains('@') == attr)
        .flat_map(|(_, s)| s.iter()),
    )
  }
  let (ta, tb, aa, ab) = (all(a, false), all(b, false), all(a, true), all(b, true));
  let delta = |x: &FxHashMap<&str, i64>, y: &FxHashMap<&str, i64>| -> FxHashMap<String, i64> {
    minus(x, y)
      .into_iter()
      .map(|(t, n)| (t.to_string(), n))
      .collect()
  };
  let (lost_text, gained_text) = (delta(&ta, &tb), delta(&tb, &ta));
  let (lost_attr, gained_attr) = (delta(&aa, &ab), delta(&ab, &aa));
  let mut classes: Vec<&String> = a.text.keys().chain(b.text.keys()).collect();
  classes.sort();
  classes.dedup();
  let empty = Vec::new();
  for class in classes {
    let (sa, sb) = (
      a.text.get(class).unwrap_or(&empty),
      b.text.get(class).unwrap_or(&empty),
    );
    if sa == sb {
      continue;
    }
    let spaces = space_lost(sa, sb);
    if !spaces.is_empty() {
      flags.push(format!("space-lost:{class}"));
      details.push_str(&format!(
        "space lost {class}: around the element between {}\n",
        spaces.join(", ")
      ));
      cell.push(format!(
        "{class}: space lost {}",
        spaces.iter().take(3).cloned().collect::<Vec<_>>().join(" ")
      ));
    }
    if words_only(sa) == words_only(sb) {
      // Only element positions inside the text moved (a footnote mark, an index entry): the element
      // counts and the spacing say the rest.
      continue;
    }
    let (global_lost, global_gained) = if is_attr(&class) {
      (&lost_attr, &gained_attr)
    } else {
      (&lost_text, &gained_text)
    };
    let (ca, cb) = (tokens(sa.iter()), tokens(sb.iter()));
    let (mut lost, mut gained) = (minus(&ca, &cb), minus(&cb, &ca));
    let jitter = take_jitter(&mut lost, &mut gained);
    if jitter > 0 {
      flags.push(format!("text-jitter:{class}"));
    }
    // A token is lost (or gained) for good when one of its pieces is lost (gained) from the whole
    // document: a word, not just a split of `foo,` into `foo` + `,`.
    let truly = |tokens: &BTreeMap<&str, i64>, global: &FxHashMap<String, i64>| -> Vec<String> {
      tokens
        .keys()
        .filter(|t| {
          pieces(t)
            .iter()
            .any(|p| global.get(*p).is_some_and(|n| *n > 0))
        })
        .map(|t| t.to_string())
        .collect()
    };
    let (truly_lost, truly_gained) = (truly(&lost, global_lost), truly(&gained, global_gained));
    if !truly_lost.is_empty() {
      flags.push(format!("lost-text:{class}"));
    }
    if !truly_gained.is_empty() {
      flags.push(format!("gained-text:{class}"));
      // A number typeset from nowhere in prose: a TeX dimension (a register printed by mistake) is a
      // regression, a bare number needs a look. Not when the class lost a number too (a value changed),
      // not in math, attribute, generated-number or code classes, and not amid recovered prose: when the
      // numbers are under half of what the class gained (a document that recovered a section), they are
      // reviewed, not called a leak.
      let prose = !is_attr(&class)
        && !class.starts_with("XM")
        && !NOT_PROSE_CLASSES.contains(&class.as_str());
      // A number moved elsewhere (a `3` now inside a `ref`) is not a lost number: `truly_lost` only.
      let lost_number = truly_lost.iter().any(|t| token_number_kind(t).is_some());
      if prose {
        let kinds: Vec<bool> = truly_gained
          .iter()
          .filter_map(|t| token_number_kind(t))
          .collect();
        let mostly_numbers = 2 * kinds.len() >= truly_gained.len();
        if !lost_number && mostly_numbers && kinds.iter().any(|k| *k) {
          flags.push(format!("leaked-dimension:{class}"));
        } else if !kinds.is_empty() {
          flags.push(format!("gained-number:{class}"));
        }
      }
    }
    if truly_lost.len() < lost.len()
      || truly_gained.len() < gained.len()
      || (lost.is_empty() && gained.is_empty() && (jitter == 0 || sa.len() != sb.len()))
    {
      flags.push(format!("moved-text:{class}"));
    }
    let mut summary = format!("{class}:");
    if !lost.is_empty() {
      summary.push_str(&format!(" -[{}]", show_tokens(&lost, 6)));
    }
    if !gained.is_empty() {
      summary.push_str(&format!(" +[{}]", show_tokens(&gained, 6)));
    }
    if lost.is_empty() && gained.is_empty() {
      summary.push_str(&format!(" segments {}>{}", sa.len(), sb.len()));
    }
    details.push_str(&format!(
      "text {class}: lost [{}] gained [{}] segments {}>{}\n",
      show_tokens(&lost, 200),
      show_tokens(&gained, 200),
      sa.len(),
      sb.len()
    ));
    cell.push(summary);
  }
}

/// Element counts: every qname whose count moved, losses first.
fn diff_elements(a: &XmlFacts, b: &XmlFacts, flags: &mut Vec<String>) -> String {
  let mut names: Vec<&String> = a.elements.keys().chain(b.elements.keys()).collect();
  names.sort();
  names.dedup();
  let mut deltas: Vec<(&String, i64)> = names
    .into_iter()
    .map(|q| {
      let n = |f: &XmlFacts| i64::from(f.elements.get(q).copied().unwrap_or(0));
      (q, n(b) - n(a))
    })
    .filter(|(_, d)| *d != 0)
    .collect();
  deltas.sort_by_key(|(q, d)| (*d, (*q).clone()));
  for (q, d) in &deltas {
    let math = q.starts_with("XM") || q.starts_with("m:");
    let wrapper = matches!(q.as_str(), "text" | "emph");
    let flag = match (*d < 0, q.as_str()) {
      (true, "ERROR") => None,
      (false, "ERROR") => Some("gained-elements:ERROR".to_string()),
      _ if math => Some("math-changed".to_string()),
      // A lost font or emphasis run is lost markup (a bold word no longer bold): review. A gained one is
      // a change.
      _ if wrapper && *d < 0 => Some("wrappers-lost".to_string()),
      _ if wrapper => Some("wrappers-changed".to_string()),
      (true, _) => Some(format!("lost-elements:{q}")),
      (false, _) => Some("gained-elements".to_string()),
    };
    flags.extend(flag);
  }
  deltas
    .iter()
    .map(|(q, d)| format!("{q}:{d:+}"))
    .collect::<Vec<_>>()
    .join(",")
}

/// Attribute values: the multiset of `qname@attr=value` over every non-id attribute, so a lost class,
/// font or `name="E-mail: "` shows even when no count and no text moved. One `attrs-changed:<qname@attr>`
/// per attribute whose values changed.
fn diff_attrs(a: &XmlFacts, b: &XmlFacts, flags: &mut Vec<String>, details: &mut String) -> String {
  let mut keys: Vec<&String> = a.attrs.keys().chain(b.attrs.keys()).collect();
  keys.sort();
  keys.dedup();
  let mut by_attr: BTreeMap<&str, (Vec<String>, Vec<String>)> = BTreeMap::new();
  for key in keys {
    let n = |f: &XmlFacts| i64::from(f.attrs.get(key).copied().unwrap_or(0));
    let d = n(b) - n(a);
    if d == 0 {
      continue;
    }
    let (attr, value) = key.split_once('=').unwrap_or((key.as_str(), ""));
    let entry = by_attr.entry(attr).or_default();
    let shown = if d.abs() > 1 {
      format!("{value:?}×{}", d.abs())
    } else {
      format!("{value:?}")
    };
    if d < 0 {
      entry.0.push(shown)
    } else {
      entry.1.push(shown)
    }
  }
  let mut cells = Vec::new();
  for (attr, (lost, gained)) in &by_attr {
    flags.push(format!("attrs-changed:{attr}"));
    details.push_str(&format!(
      "attr {attr}: lost [{}] gained [{}]\n",
      lost.join(" "),
      gained.join(" ")
    ));
    let mut cell = format!("{attr}:");
    if !lost.is_empty() {
      cell.push_str(&format!(
        " -[{}]",
        lost.iter().take(4).cloned().collect::<Vec<_>>().join(" ")
      ));
    }
    if !gained.is_empty() {
      cell.push_str(&format!(
        " +[{}]",
        gained.iter().take(4).cloned().collect::<Vec<_>>().join(" ")
      ));
    }
    cells.push(cell);
  }
  cells.join(" | ")
}

/// Whether two numbers are equal within the [`tolerance`].
fn near(p: f64, q: f64) -> bool { (p - q).abs() <= tolerance(p, q) }

/// Whether a picture or SVG canvas size, or a path, collapsed between `x` (A) and `y` (B): a size
/// (`picture`/`svg:svg` width or height, the viewBox extent) shrinking by more than half or to within the
/// tolerance of zero, or a path whose coordinates all became one point (`M 0 0 L 0 0`, or none).
fn collapsed(key: &str, x: &str, y: &str) -> bool {
  let ((nx, _), (ny, _)) = (numbers(x), numbers(y));
  let size = |n: &[f64]| -> Vec<f64> {
    if key.ends_with("@viewBox") {
      n.iter().skip(2).take(2).copied().collect()
    } else {
      n.first().copied().into_iter().collect()
    }
  };
  if key == "svg:path@d" {
    // Coordinates are (x, y) pairs: `M 7 3 L 7 3` is one point. An odd trailing number is ignored; no
    // pair at all (an empty path) is no extent.
    let one_point = |n: &[f64]| {
      let (pairs, _) = n.as_chunks::<2>();
      pairs.first().is_none_or(|f| pairs.iter().all(|p| p == f))
    };
    return !one_point(&nx) && one_point(&ny);
  }
  if matches!(
    key,
    "picture@width" | "picture@height" | "svg:svg@width" | "svg:svg@height" | "svg:svg@viewBox"
  ) {
    let (sx, sy) = (size(&nx), size(&ny));
    return sx
      .iter()
      .zip(&sy)
      .any(|(p, q)| *p > 0.0 && (*q < 0.5 * p || q.abs() <= tolerance(*p, *q)));
  }
  false
}

/// Geometry (`svg:*` coordinates, a picture's size), in document order: numbers within the [`tolerance`]
/// are equal; a collapse ([`collapsed`]) is `geometry-collapsed` (a regression: the drawing is gone or a
/// dot); anything else is a `geometry-changed` change.
fn diff_geometry(a: &XmlFacts, b: &XmlFacts, flags: &mut Vec<String>, details: &mut String) {
  let mut keys: Vec<&String> = a.geometry.keys().chain(b.geometry.keys()).collect();
  keys.sort();
  keys.dedup();
  let empty = Vec::new();
  let (mut moved, mut gone) = (Vec::new(), Vec::new());
  for key in keys {
    let (va, vb) = (
      a.geometry.get(key).unwrap_or(&empty),
      b.geometry.get(key).unwrap_or(&empty),
    );
    if va.iter().zip(vb).any(|(x, y)| collapsed(key, x, y)) {
      gone.push(key.as_str());
    }
    let same = va.len() == vb.len()
      && va.iter().zip(vb).all(|(x, y)| {
        let ((nx, sx), (ny, sy)) = (numbers(x), numbers(y));
        sx == sy && nx.len() == ny.len() && nx.iter().zip(&ny).all(|(p, q)| near(*p, *q))
      });
    if !same {
      moved.push(key.as_str());
    }
  }
  if !gone.is_empty() {
    flags.push("geometry-collapsed".into());
    details.push_str(&format!("geometry collapsed: {}\n", gone.join(" ")));
  }
  if !moved.is_empty() {
    flags.push("geometry-changed".into());
    details.push_str(&format!(
      "geometry changed beyond the tolerance: {}\n",
      moved.join(" ")
    ));
  }
}

/// Diagnostic class deltas, Fatal then Error then Warning, larger moves first.
fn diag_delta(a: &LogFacts, b: &LogFacts) -> Vec<String> {
  let mut names: Vec<&String> = a.classes.keys().chain(b.classes.keys()).collect();
  names.sort();
  names.dedup();
  let mut deltas: Vec<(usize, i64, &String)> = names
    .into_iter()
    .filter_map(|c| {
      let n = |f: &LogFacts| i64::from(f.classes.get(c).copied().unwrap_or(0));
      let d = n(b) - n(a);
      let level = ["Fatal:", "Error:", "Warning:"]
        .iter()
        .position(|l| c.starts_with(l))
        .unwrap_or(3);
      (d != 0).then_some((level, d, c))
    })
    .collect();
  deltas.sort_by_key(|(level, d, c)| (*level, -*d, (*c).clone()));
  deltas.iter().map(|(_, d, c)| format!("{c}{d:+}")).collect()
}

/// Compare one paper's two conversions.
pub fn compare(key: &str, a_in: &DocInput, b_in: &DocInput) -> Row {
  let same_bytes = a_in.xml.is_some() && a_in.xml == b_in.xml;
  let want_canon = !same_bytes && a_in.xml.is_some() && b_in.xml.is_some();
  let a = read_side(a_in, want_canon, None);
  // Byte-identical XML: its facts once, for both sides.
  let b = read_side(b_in, want_canon, same_bytes.then_some(&a.xml));
  let mut row = Row {
    key: key.to_string(),
    ..Row::default()
  };
  let mut flags = Vec::new();
  side_problems(&a, "A", &mut flags);
  side_problems(&b, "B", &mut flags);
  // A problem on both sides is one `<problem>:both` flag.
  let twins: Vec<String> = flags
    .iter()
    .filter_map(|f| f.strip_suffix(":A"))
    .filter(|p| flags.iter().any(|g| g.strip_suffix(":B") == Some(*p)))
    .map(str::to_string)
    .collect();
  for p in twins {
    flags.retain(|f| f != &format!("{p}:A") && f != &format!("{p}:B"));
    flags.push(format!("{p}:both"));
  }
  // Mid-line Fatals and Errors compare by count, like `errors+`: more in B is a regression, fewer a fix.
  for kind in ["fatal", "error"] {
    let count = |s: &Side| {
      s.log.as_ref().map_or(0, |l| {
        if kind == "fatal" {
          l.midline_fatals
        } else {
          l.midline_errors
        }
      })
    };
    let (x, y) = (count(&a), count(&b));
    let prefix = format!("{kind}-midline:");
    flags.retain(|f| !f.starts_with(&prefix));
    let flag = match y.cmp(&x) {
      std::cmp::Ordering::Greater if x == 0 => Some(format!("{kind}-midline:B")),
      std::cmp::Ordering::Greater => Some(format!("{kind}-midline+")),
      std::cmp::Ordering::Less if y == 0 => Some(format!("{kind}-midline-fixed")),
      std::cmp::Ordering::Less => Some(format!("{kind}-midline-")),
      std::cmp::Ordering::Equal if x > 0 => Some(format!("{kind}-midline:both")),
      std::cmp::Ordering::Equal => None,
    };
    flags.extend(flag);
  }
  // A timeout or out-of-memory of the base alone is a fix; the base without XML while B has one is a gain
  // to review.
  for problem in ["timeout", "oom"] {
    if !flags.iter().any(|f| *f == format!("{problem}:B"))
      && let Some(i) = flags.iter().position(|f| *f == format!("{problem}:A"))
    {
      flags[i] = format!("{problem}-fixed");
    }
  }
  if b.xml.is_ok()
    && let Some(i) = flags.iter().position(|f| f == "xml-missing:A")
  {
    flags[i] = "xml-gained".into();
  }
  let mut details = String::new();
  // Diagnostics.
  let (la, lb) = (
    a.log.clone().unwrap_or_default(),
    b.log.clone().unwrap_or_default(),
  );
  let diags = if let (Some(x), Some(y)) = (&a.log, &b.log) {
    let worse = |p: u32, q: u32, up: &str, down: &str, flags: &mut Vec<String>| {
      if q > p {
        flags.push(up.into());
      } else if q < p {
        flags.push(down.into());
      }
    };
    worse(x.fatals, y.fatals, "fatal+", "fatal-", &mut flags);
    worse(x.errors, y.errors, "errors+", "errors-", &mut flags);
    worse(x.warnings, y.warnings, "warnings+", "warnings-", &mut flags);
    if let (Some(s), Some(t)) = (x.status, y.status) {
      worse(
        u32::from(s),
        u32::from(t),
        "status-worse",
        "status-better",
        &mut flags,
      );
    }
    let delta = diag_delta(x, y);
    if !delta.is_empty() && x.errors == y.errors && x.fatals == y.fatals && x.warnings == y.warnings
    {
      flags.push("diag-classes-changed".into());
    }
    if !delta.is_empty() {
      details.push_str(&format!("diagnostics: {}\n", delta.join(" ")));
    }
    delta
  } else {
    Vec::new()
  };
  // Content.
  let (mut elem_cell, mut attr_cell, mut text_cells, mut front_cell, mut xml_cell) = (
    String::new(),
    String::new(),
    Vec::new(),
    String::new(),
    String::from("-"),
  );
  if let (Ok(x), Ok(y)) = (&a.xml, &b.xml) {
    if same_bytes {
      xml_cell = "same".into();
    } else {
      let before_content = flags.len();
      elem_cell = diff_elements(x, y, &mut flags);
      attr_cell = diff_attrs(x, y, &mut flags, &mut details);
      diff_geometry(x, y, &mut flags, &mut details);
      diff_text(x, y, &mut flags, &mut text_cells, &mut details);
      let fd = diff_front(&x.front, &y.front);
      flags.extend(fd.flags.iter().cloned());
      front_cell = fd
        .notes
        .iter()
        .take(4)
        .cloned()
        .collect::<Vec<_>>()
        .join("; ");
      for note in &fd.notes {
        details.push_str(&format!("frontmatter: {note}\n"));
      }
      if let (Some(ca), Some(cb)) = (&a.canon, &b.canon) {
        let d = canon_diff(ca, cb, 20);
        xml_cell = if d.deleted + d.inserted == 0 && d.ids_renamed == 0 {
          "equivalent".to_string()
        } else if d.deleted + d.inserted == 0 {
          format!("ids-only({})", d.ids_renamed)
        } else {
          format!(
            "-{}/+{}{}",
            d.deleted,
            d.inserted,
            if d.approximate { " approx" } else { "" }
          )
        };
        if d.refs_retargeted > 0 {
          flags.push("ref-retargeted".into());
          xml_cell.push_str(&format!(" refs-retargeted:{}", d.refs_retargeted));
        }
        if d.deleted + d.inserted == 0 {
          if flags.len() == before_content {
            // Canonically equal: ids renumbered, or only what the canonical form normalises (geometry
            // within the tolerance, host paths).
            flags.push(
              if d.ids_renamed > 0 {
                "ids-only"
              } else {
                "equivalent"
              }
              .into(),
            );
          }
        } else if flags[before_content..]
          .iter()
          .all(|f| rank(f).0 == Tier::Change)
        {
          // The canonical XML moved and nothing that needs a reason explains it (no content flag, or only
          // change-tier ones such as `math-changed`): review, never a silent change.
          flags.push("xml-only".into());
        }
        for hunk in &d.hunks {
          details.push_str(hunk);
        }
      }
    }
  }
  // Wall time.
  let secs = |m: &Option<RunMeta>| m.as_ref().and_then(|m| m.secs);
  if let (Some(p), Some(q)) = (secs(&a.meta), secs(&b.meta))
    && q > 1.5 * p
    && q - p > 5.0
  {
    flags.push("slower".into());
  }
  row.flags = flags;
  row.finish();
  let fmt_secs = |m: &Option<RunMeta>| secs(m).map(|s| format!("{s:.1}"));
  row.cells = vec![
    pair(
      la.status.filter(|_| a.log.is_some()),
      lb.status.filter(|_| b.log.is_some()),
    ),
    pair(
      a.log.as_ref().map(|l| l.errors),
      b.log.as_ref().map(|l| l.errors),
    ),
    pair(
      a.log.as_ref().map(|l| l.fatals),
      b.log.as_ref().map(|l| l.fatals),
    ),
    pair(
      a.log.as_ref().map(|l| l.warnings),
      b.log.as_ref().map(|l| l.warnings),
    ),
    clip(&diags.join(" "), 400),
    clip(&elem_cell, 400),
    clip(&attr_cell, 400),
    clip(&text_cells.join(" | "), 600),
    clip(&front_cell, 600),
    xml_cell,
    pair(fmt_secs(&a.meta), fmt_secs(&b.meta)),
    pair(
      a.log.as_ref().map(status_diag_cell),
      b.log.as_ref().map(status_diag_cell),
    ),
  ];
  for (tag, side) in [("A", &a), ("B", &b)] {
    if let Err(e) = &side.xml
      && side.present
    {
      details.push_str(&format!("xml {tag}: {e}\n"));
    }
    for line in &side.stderr_fatals {
      details.push_str(&format!("stderr {tag}: {line}\n"));
    }
  }
  row.details = details;
  row
}

/// Compare columns after `key`, `tier`, `flags`.
pub const COMPARE_COLUMNS: &[&str] = &[
  "status",
  "errors",
  "fatals",
  "warnings",
  "diag_delta",
  "elem_delta",
  "attr_delta",
  "text_delta",
  "frontmatter",
  "xml",
  "secs",
  "status_diag",
];

/// Audit one conversion: absolute flags (a missing or unparsable piece, a Fatal, errors, a nameless
/// creator, an empty, split or fragment-linked contact) and its counts.
pub fn audit(key: &str, input: &DocInput) -> Row {
  let side = read_side(input, false, None);
  let mut row = Row {
    key: key.to_string(),
    ..Row::default()
  };
  let mut flags = Vec::new();
  side_problems(&side, "", &mut flags);
  for f in flags.iter_mut() {
    if let Some(stripped) = f.strip_suffix(':') {
      *f = stripped.to_string();
    }
  }
  let log = side.log.clone().unwrap_or_default();
  // The status alone raises the severity too: a status 3 without a `Fatal:` line is still a Fatal run
  // (with `status-diag-mismatch` as the reason).
  if log.fatals > 0 || log.status == Some(3) {
    flags.push("fatal".into());
  }
  if log.errors > 0 || log.status == Some(2) {
    flags.push("errors".into());
  }
  let mut top: Vec<(&String, &u32)> = log
    .classes
    .iter()
    .filter(|(c, _)| !c.starts_with("Warning:"))
    .collect();
  top.sort_by_key(|(c, n)| (std::cmp::Reverse(**n), (*c).clone()));
  let top = top
    .iter()
    .take(5)
    .map(|(c, n)| format!("{c}×{n}"))
    .collect::<Vec<_>>()
    .join(" ");
  let (mut counts, mut front_cell, mut words) = (String::new(), String::new(), String::from("-"));
  if let Ok(x) = &side.xml {
    let f = &x.front;
    for (n, flag) in [
      (f.nameless(), "creator-nameless"),
      (f.empty_contacts(), "contact-empty"),
      (f.split_contacts(), "contact-split"),
      (f.fragment_links(), "contact-link-fragment"),
    ] {
      if n > 0 {
        flags.push(flag.into());
      }
    }
    if x.elements.get("ERROR").is_some_and(|n| *n > 0) {
      flags.push("error-elements".into());
    }
    counts = AUDIT_ELEMENTS
      .iter()
      .map(|q| format!("{q}:{}", x.elements.get(*q).copied().unwrap_or(0)))
      .collect::<Vec<_>>()
      .join(",");
    front_cell = format!(
      "creators:{} nameless:{} empty-contacts:{} split:{} fragment-links:{} title:{} abstract:{}",
      f.creators.len(),
      f.nameless(),
      f.empty_contacts(),
      f.split_contacts(),
      f.fragment_links(),
      !f.title.is_empty(),
      !f.abstract_text.is_empty()
    );
    words = x.words.to_string();
  } else if let Err(e) = &side.xml {
    row.details = format!("xml: {e}\n");
  }
  for line in &side.stderr_fatals {
    row.details.push_str(&format!("stderr: {line}\n"));
  }
  row.flags = flags;
  row.finish();
  row.cells = vec![
    opt(side.log.as_ref().and_then(|l| l.status)),
    opt(side.log.as_ref().map(|l| l.errors)),
    opt(side.log.as_ref().map(|l| l.fatals)),
    opt(side.log.as_ref().map(|l| l.warnings)),
    clip(&top, 400),
    counts,
    front_cell,
    words,
    opt(
      side
        .meta
        .as_ref()
        .and_then(|m| m.secs)
        .map(|s| format!("{s:.0}")),
    ),
    opt(side.log.as_ref().map(status_diag_cell)),
  ];
  row
}

/// The element counts the audit row shows.
pub const AUDIT_ELEMENTS: &[&str] = &[
  "creator", "contact", "section", "bibitem", "graphics", "picture", "Math", "equation", "figure",
  "table", "float", "ERROR",
];

/// Audit columns after `key`, `tier`, `flags`.
pub const AUDIT_COLUMNS: &[&str] = &[
  "status",
  "errors",
  "fatals",
  "warnings",
  "top_diags",
  "elements",
  "frontmatter",
  "words",
  "secs",
  "status_diag",
];

/// One side's status/diagnostics agreement, `ok` or `mismatch`.
fn status_diag_cell(log: &LogFacts) -> &'static str {
  if log.status_mismatch() {
    "mismatch"
  } else {
    "ok"
  }
}

/// Cut a cell to `n` characters, tabs and newlines made spaces.
pub fn clip(s: &str, n: usize) -> String {
  let s = s.replace(['\t', '\n', '\r'], " ");
  if s.chars().count() <= n {
    s
  } else {
    format!("{}…", s.chars().take(n).collect::<String>())
  }
}

#[cfg(test)]
#[path = "diff_tests.rs"]
mod tests;

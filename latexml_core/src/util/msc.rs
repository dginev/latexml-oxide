//! The Mathematics Subject Classification editions a list of codes fits, and the edition a classification is
//! stamped with: the call's year, the author's own label, the codes, or the document's date (OXIDIZED_DESIGN_DIVERGENCES
//! #1168). Class-neutral: a binding passes its evidence in and gets the decision back.

use std::{fmt, ops::BitAnd, str::FromStr};

mod table;

/// A published edition of the MSC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Edition {
  Msc1991,
  Msc2000,
  Msc2010,
  Msc2020,
}

impl Edition {
  pub const ALL: [Edition; 4] = [
    Edition::Msc1991,
    Edition::Msc2000,
    Edition::Msc2010,
    Edition::Msc2020,
  ];

  pub const fn year(self) -> u16 {
    match self {
      Edition::Msc1991 => 1991,
      Edition::Msc2000 => 2000,
      Edition::Msc2010 => 2010,
      Edition::Msc2020 => 2020,
    }
  }

  pub fn from_year(year: u16) -> Option<Self> { Self::ALL.into_iter().find(|e| e.year() == year) }

  /// The edition current in `year`: the latest published by then, 1991 before it.
  pub fn current_at(year: u16) -> Self {
    Self::ALL
      .into_iter()
      .rev()
      .find(|e| e.year() <= year)
      .unwrap_or(Edition::Msc1991)
  }

  /// The first edition year a label writes (`\textup{2010} Mathematics Subject Classification`, `MSC2020`).
  pub fn named_in(text: &str) -> Option<Self> {
    text
      .split(|c: char| !c.is_ascii_digit())
      .filter(|digits| digits.len() == 4)
      .find_map(|digits| Self::from_year(digits.parse().ok()?))
  }

  const fn bit(self) -> u8 { 1 << self as u8 }
}

impl fmt::Display for Edition {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.year()) }
}

/// A set of editions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Editions(u8);

impl Editions {
  pub const ALL: Editions = Editions(0b1111);
  pub const NONE: Editions = Editions(0);

  pub fn contains(self, edition: Edition) -> bool { self.0 & edition.bit() != 0 }

  pub fn is_empty(self) -> bool { self.0 == 0 }

  pub fn single(self) -> Option<Edition> {
    let mut all = self.iter();
    all.next().filter(|_| all.next().is_none())
  }

  pub fn iter(self) -> impl Iterator<Item = Edition> {
    Edition::ALL.into_iter().filter(move |e| self.contains(*e))
  }
}

impl BitAnd for Editions {
  type Output = Editions;

  fn bitand(self, other: Editions) -> Editions { Editions(self.0 & other.0) }
}

impl FromIterator<Edition> for Editions {
  fn from_iter<I: IntoIterator<Item = Edition>>(editions: I) -> Self {
    Editions(editions.into_iter().fold(0, |bits, e| bits | e.bit()))
  }
}

impl fmt::Display for Editions {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.is_empty() {
      return f.write_str("none");
    }
    let years: Vec<String> = self.iter().map(|e| e.to_string()).collect();
    f.write_str(&years.join("/"))
  }
}

/// An MSC code: a class `00`–`99`, a subclass `-` or `A`–`Z`, a leaf `00`–`99` or `xx` (`LEAF_XX`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Code {
  class: u8,
  sub:   u8,
  leaf:  u8,
}

const LEAF_XX: u8 = 100;

impl Code {
  /// The editions the code is in; none for a typo or a code from before 1991.
  pub fn editions(self) -> Editions {
    let class = usize::from(self.class);
    let Some(&[start, end]) = table::CLASS_START.get(class..class + 2) else {
      return Editions::NONE;
    };
    let entries = &table::ENTRIES[usize::from(start)..usize::from(end)];
    entries
      .binary_search_by_key(&self.key(), |entry| entry >> 4)
      .map_or(Editions::NONE, |i| Editions((entries[i] & 0xf) as u8))
  }

  /// The code's place in its class: `(sub_index * 101 + leaf_index)`, `-` first, then `A`–`Z`; `xx` last.
  fn key(self) -> u16 {
    let sub = if self.sub == b'-' {
      0
    } else {
      self.sub - b'A' + 1
    };
    u16::from(sub) * 101 + u16::from(self.leaf)
  }

  fn parse(text: &[u8]) -> Option<Self> {
    let &[c1, c2, sub, l1, l2] = text else {
      return None;
    };
    let digits = |a: u8, b: u8| {
      (a.is_ascii_digit() && b.is_ascii_digit()).then(|| (a - b'0') * 10 + (b - b'0'))
    };
    let sub = sub.to_ascii_uppercase();
    let leaf = if l1.eq_ignore_ascii_case(&b'x') && l2.eq_ignore_ascii_case(&b'x') {
      LEAF_XX
    } else {
      digits(l1, l2)?
    };
    (sub == b'-' || sub.is_ascii_uppercase()).then_some(Code {
      class: digits(c1, c2)?,
      sub,
      leaf,
    })
  }
}

impl FromStr for Code {
  type Err = ();

  /// `NN-NN`, `NNLNN`, `NNLxx`, `NN-XX`, in either case.
  fn from_str(text: &str) -> Result<Self, ()> { Code::parse(text.as_bytes()).ok_or(()) }
}

impl fmt::Display for Code {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:02}{}", self.class, self.sub as char)?;
    match (self.leaf, self.sub) {
      (LEAF_XX, b'-') => f.write_str("XX"),
      (LEAF_XX, _) => f.write_str("xx"),
      (leaf, _) => write!(f, "{leaf:02}"),
    }
  }
}

/// The code-shaped words of `text` (a hyphen joins words, as in a date), everything else skipped: no list structure
/// is read.
pub fn codes(text: &str) -> impl Iterator<Item = Code> + '_ {
  let bytes = text.as_bytes();
  let word = |i: usize| {
    bytes
      .get(i)
      .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'-')
  };
  (0..bytes.len().saturating_sub(4))
    .filter(move |&i| (i == 0 || !word(i - 1)) && !word(i + 5))
    .filter_map(move |i| Code::parse(&bytes[i..i + 5]))
}

/// What a class and its author say about the edition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Evidence {
  /// The edition the class prints.
  pub print:    Edition,
  /// The edition the call names.
  pub declared: Option<Edition>,
  /// The edition of the author's own label, which the class discards.
  pub label:    Option<Edition>,
  /// The year of the document's source, when it carries a date (`state::source_date_epoch`).
  pub year:     Option<u16>,
}

impl Evidence {
  /// The document's year, a year before 1991 being none: a zip entry's "no time", 1980-01-01, read in local time once
  /// unpacked (`latexml::source_date`).
  pub fn dated(&self) -> Option<u16> { self.year.filter(|y| *y >= Edition::Msc1991.year()) }
}

/// Why an edition was chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
  Declared,
  AuthorLabel,
  UniqueFit,
  ClassPrint,
  Era,
  Nearest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inference {
  pub edition: Edition,
  pub reason:  Reason,
  /// The editions every known code is in.
  pub fits:    Editions,
  /// The codes in no edition, each once.
  pub unknown: Vec<Code>,
}

/// The edition the codes and the evidence support: the call's or the author's when the codes fit it, the one edition
/// they fit, the class's print when they fit it (or none), else the edition current at the document's year when they fit
/// it, else the fitting one nearest to that edition, or to the print for an undated document (the older on a tie).
pub fn infer(codes: impl IntoIterator<Item = Code>, evidence: &Evidence) -> Inference {
  let (mut fits, mut known, mut unknown) = (Editions::ALL, false, Vec::new());
  for code in codes {
    match code.editions() {
      editions if editions.is_empty() => {
        if !unknown.contains(&code) {
          unknown.push(code);
        }
      },
      editions => (fits, known) = (fits & editions, true),
    }
  }
  let fitting = |e: Option<Edition>| e.filter(|e| known && fits.contains(*e));
  let era = evidence.dated().map(Edition::current_at);
  let (edition, reason) = if let Some(e) = fitting(evidence.declared) {
    (e, Reason::Declared)
  } else if let Some(e) = fitting(evidence.label) {
    (e, Reason::AuthorLabel)
  } else if let Some(e) = fits.single().filter(|_| known) {
    (e, Reason::UniqueFit)
  } else if !known || fits.is_empty() || fits.contains(evidence.print) {
    (evidence.print, Reason::ClassPrint)
  } else if let Some(era) = era.filter(|era| fits.contains(*era)) {
    (era, Reason::Era)
  } else {
    let anchor = era.unwrap_or(evidence.print) as i8;
    let distance = |e: &Edition| ((*e as i8 - anchor).abs(), *e);
    (
      fits.iter().min_by_key(distance).unwrap_or(evidence.print),
      Reason::Nearest,
    )
  };
  Inference {
    edition,
    reason,
    fits: if known { fits } else { Editions::ALL },
    unknown,
  }
}

impl Inference {
  /// The edition when it is not the class's print.
  pub fn correction(&self, evidence: &Evidence) -> Option<Edition> {
    (self.edition != evidence.print).then_some(self.edition)
  }

  /// The Info lines (name, text): one for a correction of the class's print, one per code in no edition.
  pub fn notes(&self, evidence: &Evidence) -> Vec<(&'static str, String)> {
    let mut notes = Vec::new();
    if let Some(edition) = self.correction(evidence) {
      let why = match self.reason {
        Reason::Declared => "the edition the call names".to_string(),
        Reason::AuthorLabel => "the edition of the author's own label".to_string(),
        Reason::UniqueFit => "the only edition the codes are in".to_string(),
        Reason::Era => format!(
          "the edition current in the document's year {}",
          evidence.dated().unwrap_or_default()
        ),
        Reason::Nearest | Reason::ClassPrint => match evidence.dated() {
          Some(year) => format!("the codes' edition nearest the document's year {year}"),
          None => "the codes' edition nearest the class's print, the source undated".to_string(),
        },
      };
      let text = format!(
        "MSC {} (the class's print) -> {edition}: {why} (codes fit {})",
        evidence.print, self.fits
      );
      notes.push(("msc_edition", text));
    }
    for code in &self.unknown {
      notes.push((
        "msc_unknown_code",
        format!("MSC code {code} is in no edition (1991, 2000, 2010, 2020)"),
      ));
    }
    notes
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn code(text: &str) -> Code { text.parse().unwrap() }

  #[test]
  fn code_shapes() {
    for (text, canonical) in [
      ("35Q55", "35Q55"),
      ("35q55", "35Q55"),
      ("35-02", "35-02"),
      ("35Qxx", "35Qxx"),
      ("35QXX", "35Qxx"),
      ("35-XX", "35-XX"),
      ("35-xx", "35-XX"),
    ] {
      assert_eq!(code(text).to_string(), canonical, "{text}");
    }
    for bad in [
      "35Q5", "35Q555", "3AQ55", "35Q5x", "35.55", "35QX5", "", "35Q55 ",
    ] {
      assert!(bad.parse::<Code>().is_err(), "{bad}");
    }
  }

  #[test]
  fn codes_are_scanned_words() {
    let found: Vec<String> =
      codes(r"\textbf{Primary} 35Q55; Secondary 35-XX, 11Fxx \sep 05C10.(2010-05-11) x35Q55 35Q55x 9.F.4.1")
        .map(|c| c.to_string())
        .collect();
    assert_eq!(found, ["35Q55", "35-XX", "11Fxx", "05C10"]);
    assert_eq!(codes("D.3.1, F.4.1, Theory of computation").count(), 0);
  }

  #[test]
  fn editions_set() {
    let set: Editions = [Edition::Msc2010, Edition::Msc2020].into_iter().collect();
    assert!(set.contains(Edition::Msc2010) && !set.contains(Edition::Msc2000));
    assert_eq!(set.single(), None);
    assert_eq!(
      (set & [Edition::Msc2020].into_iter().collect()).single(),
      Some(Edition::Msc2020)
    );
    assert!((set & Editions::NONE).is_empty());
    assert_eq!(set.to_string(), "2010/2020");
    assert_eq!(Editions::NONE.to_string(), "none");
    assert_eq!(Edition::current_at(1985), Edition::Msc1991);
    assert_eq!(Edition::current_at(2009), Edition::Msc2000);
    assert_eq!(Edition::current_at(2026), Edition::Msc2020);
    assert_eq!(
      Edition::named_in(r"\textup {2010} Mathematics Subject Classification"),
      Some(Edition::Msc2010)
    );
    assert_eq!(
      Edition::named_in("MSC2000 (revised 2010)"),
      Some(Edition::Msc2000)
    );
    assert_eq!(Edition::named_in("12010 or 2011"), None);
  }

  #[test]
  fn table_lookup_and_invariants() {
    let starts = &table::CLASS_START;
    assert!(
      starts.windows(2).all(|w| w[0] <= w[1]),
      "CLASS_START is monotone"
    );
    assert_eq!(usize::from(starts[starts.len() - 1]), table::ENTRIES.len());
    for class in starts.windows(2) {
      let entries = &table::ENTRIES[usize::from(class[0])..usize::from(class[1])];
      assert!(
        entries.windows(2).all(|w| w[0] >> 4 < w[1] >> 4),
        "each class's keys strictly sorted"
      );
      assert!(
        entries.iter().all(|e| e & 0xf != 0),
        "every code is in an edition"
      );
    }
    let years = |c: &str| code(c).editions().to_string();
    assert_eq!(years("16R10"), "1991/2000/2010/2020");
    assert_eq!(years("57K12"), "2020");
    assert_eq!(years("58G25"), "1991");
    assert_eq!(years("15A33"), "1991/2000");
    assert_eq!(years("20E45"), "2000/2010/2020");
    assert_eq!(years("53D05"), "2000/2010/2020");
    assert_eq!(years("00-XX"), "1991/2000/2010/2020");
    assert_eq!(years("35Qxx"), "1991/2000/2010/2020");
    assert_eq!(years("99A99"), "none");
    assert_eq!(years("35Q00"), "none");
  }

  fn evidence(
    print: u16,
    declared: Option<u16>,
    label: Option<u16>,
    year: Option<u16>,
  ) -> Evidence {
    let e = |y: u16| Edition::from_year(y).unwrap();
    Evidence {
      print: e(print),
      declared: declared.map(e),
      label: label.map(e),
      year,
    }
  }

  fn stamp(list: &str, ev: Evidence) -> (u16, Reason) {
    let inference = infer(codes(list), &ev);
    (inference.edition.year(), inference.reason)
  }

  /// The study's cases on its witnesses' codes (ADV_MSCINFER_2026-11-26 §5).
  #[test]
  fn infer_cases() {
    use Reason::*;
    // C1: 1204.4468's codes under a `[2010]` they fit.
    assert_eq!(
      stamp("35Q55, 35A01", evidence(2010, Some(2010), None, Some(2012))),
      (2010, Declared)
    );
    // C2: 1501.02464, yearless, the discarded `\subjclassname` names 2010.
    assert_eq!(
      stamp(
        "16R10; 17A70, 16R30",
        evidence(2020, None, Some(2010), Some(2015))
      ),
      (2010, AuthorLabel)
    );
    // C3: 2111.04526 `[2010]{57K12}`; 0904.0450 yearless; dg-ga9604003 yearless.
    assert_eq!(
      stamp("57K12", evidence(2010, Some(2010), None, Some(2021))),
      (2020, UniqueFit)
    );
    assert_eq!(
      stamp(
        "15A33, 20G40, 20E45",
        evidence(2020, None, None, Some(2009))
      ),
      (2000, UniqueFit)
    );
    assert_eq!(
      stamp(
        "58G25, 58G30, 53C20; 35P15",
        evidence(2020, None, None, Some(1996))
      ),
      (1991, UniqueFit)
    );
    // ClassPrint: 2102.08651's codes, in every edition; math0201109's, the print among them.
    assert_eq!(
      stamp("41A25, 94A12", evidence(2020, None, None, Some(2021))),
      (2020, ClassPrint)
    );
    assert_eq!(
      stamp(
        "60E07, 60F15, 91E40",
        evidence(2020, None, None, Some(2002))
      ),
      (2020, ClassPrint)
    );
    // R-era: 1204.4468 `[2000]` with 2010/2020 codes in 2012.
    assert_eq!(
      stamp(
        "35Q55, 35A01, 35B40",
        evidence(2000, Some(2000), None, Some(2012))
      ),
      (2010, Era)
    );
    // R-nearest: 2211.06115 `[2020]` with 1991/2000/2010 codes in 2022.
    assert_eq!(
      stamp(
        "18E30, 14F05; 14M15, 18A05",
        evidence(2020, Some(2020), None, Some(2022))
      ),
      (2010, Nearest)
    );
    // R-nearest, undated (the harness's 1980 too): nearest the print, 1204.4468 and 2211.06115 as arXiv sends them.
    for year in [None, Some(1980)] {
      assert_eq!(
        stamp("35Q55, 35B51", evidence(2000, Some(2000), None, year)),
        (2010, Nearest)
      );
      assert_eq!(
        stamp(
          "18E30, 14F05; 14M15",
          evidence(2020, Some(2020), None, year)
        ),
        (2010, Nearest)
      );
    }
    // R-era where the nearest edition is another: 2022, `[2000]`, 2010/2020 codes.
    assert_eq!(
      stamp("35Q55, 35B51", evidence(2000, Some(2000), None, Some(2022))),
      (2020, Era)
    );
    // No fit: 1009.4045 (58F06 is 1991's, 53D05 2000's on): the print.
    assert_eq!(
      stamp(
        "53D05; 53C55; 58F06",
        evidence(2000, Some(2000), None, Some(2010))
      ),
      (2000, ClassPrint)
    );
    // A code in no edition constrains nothing and is reported.
    let ev = evidence(2020, None, None, Some(2021));
    let inference = infer(codes("99A99, 41A25, 99a99"), &ev);
    assert_eq!(
      (inference.edition, inference.reason),
      (Edition::Msc2020, ClassPrint)
    );
    assert_eq!(inference.notes(&ev), [(
      "msc_unknown_code",
      "MSC code 99A99 is in no edition (1991, 2000, 2010, 2020)".to_string()
    )]);
    // Only unknown codes: the author's label is no evidence.
    assert_eq!(
      stamp("99A99", evidence(2020, None, Some(2010), Some(2015))),
      (2020, ClassPrint)
    );
  }

  #[test]
  fn notes_name_the_correction() {
    let ev = evidence(2020, None, Some(2010), Some(2015));
    let inference = infer(codes("16R10"), &ev);
    assert_eq!(inference.notes(&ev), [(
      "msc_edition",
      "MSC 2020 (the class's print) -> 2010: the edition of the author's own label (codes fit \
       1991/2000/2010/2020)"
        .to_string()
    )]);
    let ev = evidence(2020, None, None, Some(2021));
    assert!(
      infer(codes("57K12"), &ev).notes(&ev).is_empty(),
      "no Info when the print stands"
    );
  }

  /// Rewrites `table.rs` from the sources in `$MSC_SOURCES` (`background/msc/`, see `background/SOURCES.md`):
  /// `MSC_2020.csv`, `classifications2010.pdf`, `classifications2000.pdf` (read by `pdftotext`), and the 1991 hypertext
  /// pages unpacked from `mr-html.tar.Z` into `hw1991/`.
  /// `MSC_SOURCES=$PWD/background/msc cargo test -p latexml_core --lib util::msc::tests::regenerate_table -- --ignored`
  #[test]
  #[ignore = "regenerates the table from downloaded sources"]
  fn regenerate_table() {
    use std::{collections::BTreeMap, fmt::Write, fs, path::Path, process::Command};
    let dir = std::env::var("MSC_SOURCES").expect("MSC_SOURCES names the source directory");
    let dir = Path::new(&dir);
    let whole_line =
      |text: &str| -> Vec<Code> { text.lines().filter_map(|l| l.trim().parse().ok()).collect() };
    let pdf = |name: &str| {
      let out = Command::new("pdftotext")
        .arg(dir.join(name))
        .arg("-")
        .output()
        .expect("pdftotext");
      whole_line(&String::from_utf8_lossy(&out.stdout))
    };
    let csv = fs::read(dir.join("MSC_2020.csv")).unwrap();
    let csv = String::from_utf8_lossy(&csv);
    let c2020: Vec<Code> = csv
      .lines()
      .skip(1)
      .filter_map(|l| l.split('\t').next()?.trim_matches('"').parse().ok())
      .collect();
    // 1991: the code opening each `<LI>` entry and `<H3>` heading, after an `<A …>` tag.
    let mut c1991 = Vec::new();
    for page in fs::read_dir(dir.join("hw1991")).unwrap() {
      let path = page.unwrap().path();
      if path.extension().is_none_or(|x| x != "html") {
        continue;
      }
      let html = String::from_utf8_lossy(&fs::read(&path).unwrap()).into_owned();
      for entry in html.split("<LI>").skip(1).chain(html.split("<H3>").skip(1)) {
        let mut rest = entry.trim_start();
        if rest.starts_with("<A ") {
          rest = rest.split_once('>').map_or("", |(_, r)| r.trim_start());
        }
        c1991.extend(rest.get(..5).and_then(|c| c.parse::<Code>().ok()));
      }
    }
    let sources = [
      (Edition::Msc1991, c1991, 4811),
      (Edition::Msc2000, pdf("classifications2000.pdf"), 5531),
      (Edition::Msc2010, pdf("classifications2010.pdf"), 6198),
      (Edition::Msc2020, c2020, 6603),
    ];
    let mut table: BTreeMap<Code, u8> = BTreeMap::new();
    for (edition, codes, published) in sources {
      let before = table.values().filter(|m| *m & edition.bit() != 0).count();
      for code in codes {
        *table.entry(code).or_default() |= edition.bit();
      }
      let count = table.values().filter(|m| *m & edition.bit() != 0).count() - before;
      assert_eq!(
        count, published,
        "MSC{edition}: the published number of codes"
      );
    }
    let mut starts = vec![0u16; 99];
    for code in table.keys() {
      for start in &mut starts[usize::from(code.class) + 1..] {
        *start += 1;
      }
    }
    let entries: Vec<u16> = table
      .iter()
      .map(|(c, mask)| (c.key() << 4) | u16::from(*mask))
      .collect();
    let hex = |values: &[u16]| {
      values
        .chunks(16)
        .map(|line| {
          line
            .iter()
            .map(|v| format!("0x{v:04x},"))
            .collect::<Vec<_>>()
            .join(" ")
        })
        .map(|line| format!("  {line}\n"))
        .collect::<String>()
    };
    let mut out = String::new();
    write!(
      out,
      "// @generated by `util::msc::tests::regenerate_table` from background/msc/ (fetched 2026-11-26); do not edit.
//
// Derived facts: the MSC codes (identifiers only, no descriptions) of each edition, as a set of editions per code.
// MSC2020 (msc2020.org/MSC_2020.csv) (c) zbMATH Open and AMS Mathematical Reviews, published under CC BY-NC-SA 4.0;
// MSC2010 and MSC2000 from AMS's published lists (mathscinet.ams.org/msc/pdfs/classifications{{2010,2000}}.pdf);
// MSC1991 from the Heriot-Watt hypertext copy (www.macs.hw.ac.uk/~chris/mr-html.tar.Z).
// Regenerate: MSC_SOURCES=$PWD/background/msc \\
//   cargo test -p latexml_core --lib util::msc::tests::regenerate_table -- --ignored
//
// ENTRIES holds one entry per code, by class (CLASS_START[class]..CLASS_START[class + 1]), sorted:
// `(sub_index * 101 + leaf_index) << 4 | editions`, sub_index 0 for `-` then A-Z 1-26, leaf_index 100 for `xx`,
// editions bit 0 = 1991, 1 = 2000, 2 = 2010, 3 = 2020.

#[rustfmt::skip]
pub(super) static CLASS_START: [u16; 99] = [
{}];

#[rustfmt::skip]
pub(super) static ENTRIES: [u16; {}] = [
{}];
",
      hex(&starts),
      entries.len(),
      hex(&entries)
    )
    .unwrap();
    fs::write(
      Path::new(env!("CARGO_MANIFEST_DIR")).join("src/util/msc/table.rs"),
      out,
    )
    .unwrap();
  }
}

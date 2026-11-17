//! The reading's record of each mark a name prints (ADVARCH's `NameMark`, MARKKEY R2a; OXIDIZED_DESIGN_DIVERGENCES
//! #1062): one per label a mark requests, made where the names line reads the mark (`lines.rs`), with what keys it, what
//! printed it and how it printed. A creator's requests, its class's requests, its footnote marks, its parted labels and
//! the marks its notes print are views of it ([`RCreator::requests`] and the rest), never parallel stores reconciled by
//! count.

use super::*;

/// What printed a name's mark ([`NameMark::by`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkBy {
  /// typed on the name: `$^{1}$`, `\textsuperscript{a}`, a paper's mark macro
  Hand,
  /// the class's own request command: llncs's and aa's `\inst`, a binding's `\lx@request@frontmatter@annotation`, a
  /// block's `annotations=`
  Class,
  /// a `\footnotemark`, keyed by its number ([`NameMark::number`])
  Footnote,
  /// a note's own mark on the name it is beside (`\thanks`, `\footnote`): no request, its note answers it (an entry led
  /// by the same `\footnotemark` is that note's print: `answer_pending`, 1607.07280)
  Note,
  /// no mark printed: a creator call's own label (`\lx@add@creator[annotations={ed}]`)
  Call,
  /// authblk's `\AB@thenote` number, printed by the class once every call is read (`Resolver::thenote_finish`)
  TheNote,
}

/// Whether a mark's glyph is inked in the source or implied by a counter ([`NameMark::origin`]; DG 2026-11-16: an inked
/// mark keeps its glyph, an implied one never reuses a glyph inked elsewhere).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkOrigin {
  Inked,
  Implied,
}

/// One mark printed on a name, as the reading reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameMark {
  /// the label it requests, its key (`1`, `affiliation:1`, `*`, `fn:c1`)
  pub label:      String,
  /// a footnote mark's number, the key TeX pairs it by (`Event::number`, OD #1058)
  pub number:     Option<i64>,
  /// the event that printed it
  pub span:       Option<(u16, u16)>,
  pub by:         MarkBy,
  /// the glued mark of letters and digits it was parted from, as printed (`part_glued_labels`): a piece no entry
  /// answers shows the whole — BESIII's `$^{50A,50C}$` "50A,50C" beside its entry 50, never "A,C" (1705.11109,
  /// 1507.02068, 2204.09614)
  pub whole:      Option<String>,
  /// read apart from the mark it was glued to (ruling 1: `\inst{1, 2\footnotemark[1]}`, 2404.09857; `\inst{1*}`,
  /// 2506.20360)
  pub parted:     bool,
  /// a comma printed between it and the mark before it in its run (`$^{1,*}$`; a formula's comma,
  /// `\endnotemark[1]$^,$\endnotemark[2]`, 1703.08733), or its class prints a name's marks as one list
  /// ([`MARKS_LISTED`]: elsarticle, 2605.13555)
  pub sep_before: bool,
  /// printed on top of the zero-width title mark before it, of its glyph (`mark_ledger::overprints`: 2012.08663,
  /// 2103.01544): TeX shows the glyph once. Unset until MARKKEY R2b, whose display fills `MarkLedger::overprints`
  /// (always false now)
  pub overprint:  bool,
}

impl NameMark {
  /// A mark of `by` requesting `label`, printed by the event at `span`.
  pub(super) fn new(label: String, by: MarkBy, span: Option<(u16, u16)>) -> NameMark {
    NameMark {
      label,
      number: None,
      span,
      by,
      whole: None,
      parted: false,
      sep_before: false,
      overprint: false,
    }
  }

  /// Whether its glyph is inked in the source (typed, or a class's command printing its operand) or implied by a
  /// counter (a `\footnotemark`, a note's mark) — the bit DG's inked/implied rule reads.
  pub fn origin(&self) -> MarkOrigin {
    match self.by {
      MarkBy::Hand | MarkBy::Class => MarkOrigin::Inked,
      MarkBy::Footnote | MarkBy::Note | MarkBy::Call | MarkBy::TheNote => MarkOrigin::Implied,
    }
  }

  /// Whether it asks for an entry: every mark but a note's own.
  pub fn requests(&self) -> bool { self.by != MarkBy::Note }
}

impl RCreator {
  /// The labels its marks request, in print order (ruling 1, the marks shown on a name are its requests).
  pub fn requests(&self) -> impl Iterator<Item = &String> + '_ {
    self.marks.iter().filter(|m| m.requests()).map(|m| &m.label)
  }

  /// Whether it requests `label` exactly.
  pub fn requests_label(&self, label: &str) -> bool { self.requests().any(|r| r == label) }

  /// Whether it requests nothing.
  pub fn requests_nothing(&self) -> bool { !self.marks.iter().any(NameMark::requests) }

  /// The labels its class's own request command made ([`MarkBy::Class`]): a `_share=authors` entry only the class's
  /// `\inst{n}` picks; the emitter's kept `\inst` marks.
  pub fn class_requests(&self) -> impl Iterator<Item = &String> + '_ {
    self
      .marks
      .iter()
      .filter(|m| m.by == MarkBy::Class)
      .map(|m| &m.label)
  }

  /// Whether a mark other than a `\footnotemark` requests what `label` names ([`label_matches`]): a numbered
  /// affiliation answers no request a `\footnotemark` made, its number being its footnote's (2404.09857).
  pub fn asks_not_by_footnote(&self, label: &str) -> bool {
    self
      .marks
      .iter()
      .any(|m| m.requests() && m.by != MarkBy::Footnote && label_matches(&m.label, label))
  }

  /// Whether authblk's `\AB@thenote` number it shows is one of `labels` (ruling 1 on that mark).
  pub fn thenote_requests(&self, labels: &[String]) -> bool {
    self
      .marks
      .iter()
      .any(|m| m.by == MarkBy::TheNote && labels.contains(&m.label))
  }

  /// Whether a mark it was parted into requests a label `matches` accepts (ruling 1 on that mark).
  pub fn parted(&self, matches: impl Fn(&str) -> bool) -> bool {
    self.marks.iter().any(|m| m.parted && matches(&m.label))
  }
}

impl Resolver {
  /// The marks a `\footnotemark` printed on a name, each with its creator, in creator order.
  pub(super) fn footnote_marks(&self) -> impl Iterator<Item = (usize, &NameMark)> + '_ {
    self
      .out
      .creators
      .iter()
      .enumerate()
      .flat_map(|(c, creator)| {
        creator
          .marks
          .iter()
          .filter(|m| m.by == MarkBy::Footnote)
          .map(move |m| (c, m))
      })
  }
}

/// The state key a binding sets whose class prints all of a name's marks as one list, a comma between each two
/// (elsarticle.cls:250-262 `\sep`, :159-163 `\comma`: `\fnref{fn2,fn3}` "1,2", 2605.13555): every mark after the first
/// of a name is `sep_before` in the reading's mark record.
pub const MARKS_LISTED: &str = "lx_frontmatter_marks_listed";

/// Declares [`MARKS_LISTED`]: the binding's class prints a name's marks as one comma-separated list.
pub fn declare_marks_listed() { assign_value(MARKS_LISTED, true, Some(Scope::Global)); }

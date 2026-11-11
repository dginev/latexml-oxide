//! The title-page reading (TP1; OXIDIZED_DESIGN_DIVERGENCES #953, #996): what the class's whole title page prints at
//! this `\maketitle`, from one speculative run of it with the document's own values ([`title_page_print_tokens`]),
//! kept while the state it read is unchanged (`watch_state`, and the LaTeXML-internal values it reads,
//! `READ_VALUES`), so every consumer reads the same run:
//! - the title-page check's words ([`super::note_title_page_print`]), and what they are without what a document
//!   appended to `\@author`;
//! - the confirming check of the stores' sentinel runs (frontmatter_stores.rs `confirmed_printed`), its text;
//! - whether that appended tail prints, which decides whether the frontmatter reads it (frontmatter_stores.rs
//!   `author_tail_calls`).
//!
//! The run marks where `\@author` begins and where its tail begins and ends (sect05.rs `title_page_mark`), records the
//! `\and` family where `\@author` first prints (for the separator query, TP2), and whether article.cls's stand-in ran
//! (the kernel's `\@maketitle`, `\lx@kernel@maketitle`). Its tail answer is its own only where code a raw file or the
//! document wrote ran and the stand-in did not; else the binding's declaration answers
//! (`fm_author_tail_declared`, `Policy::title_page`). Perl reads no title page (latex_constructs.pool.ltxml:1099-1113).

use std::cell::{Cell, RefCell};

use super::{INK_AUTHOR, INK_TAIL, INK_TAIL_END, *};

/// Which code the reading's run ran for the title page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TitlePageSource {
  /// the class's (or the document's) own refused `\maketitle`, replayed ([`title_page_replay_tokens`]'s conditions)
  Replay,
  /// a `\@maketitle` a raw file, a binding or the document defined, run as written (with article.cls's page at the
  /// kernel's marker, if it holds one), where it is the title page in force (`own_maketitle`: the `\maketitle` that runs
  /// runs it, TP1b)
  Own(DefinitionOrigin),
  /// article.cls's page, for the kernel's `\@maketitle`, one no raw file, binding or document defined, or one the
  /// `\maketitle` that runs never runs (elsarticle's, acmart's)
  StandIn,
}

/// How the reading's run ended ([`SpeculativeRun`], its texts aside).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunOutcome {
  Printed,
  Aborted,
  Unavailable,
  OverBudget(usize),
}

/// What the run saw of `\@author`.
#[derive(Debug, Clone, Default)]
pub(crate) struct AuthorPrint {
  /// `\@author` reached the print
  pub(crate) printed: bool,
  /// what a document appended to `\@author` reached the print (`None`: nothing was appended)
  pub(crate) tail:    Option<bool>,
  /// the meanings of `\and`, `\And` and `\AND` where `\@author` first printed (TP2's separator query)
  pub(crate) family:  Option<[Option<Stored>; 3]>,
}

/// The title-page reading ([`title_page_reading`]).
#[derive(Debug, Clone)]
pub(crate) struct TitlePageReading {
  pub(crate) source:   TitlePageSource,
  /// article.cls's stand-in ran (the kernel's marker reached, or no page of a file's or the document's own)
  pub(crate) stand_in: bool,
  pub(crate) outcome:  RunOutcome,
  /// the words it prints ([`ink_word_text`]), the marks taken out
  pub(crate) words:    String,
  /// [`Self::words`] without what a document appended to `\@author`
  pub(crate) untailed: String,
  /// its text ([`ink_text`])
  pub(crate) text:     String,
  pub(crate) author:   AuthorPrint,
}

impl TitlePageReading {
  /// Does the title page in force print what a document appended to `\@author`? The run's own answer where code a raw
  /// file or the document wrote ran to its end, not in place of a raw class's own `\@author` reader
  /// (`CLASS_AUTHOR_READER`, which the run prints, not `\@author`): where article.cls's stand-in ran in it (the kernel's
  /// marker, which an addition to LaTeX's `\@maketitle` or a `\let` copy of it keeps), LaTeX's page is the class's, whose
  /// answer is the declaration's (`fm_author_tail_declared`: amsart's prints its own `\authors`). Else the walk of the
  /// title page of the file's or the document's own (frontmatter_stores.rs `own_title_page_prints_author`), else the
  /// declaration (`None`, unknown, under a binding declaring none).
  pub(crate) fn tail_printed(&self) -> Option<bool> {
    if self.outcome == RunOutcome::Printed
      && self.source != TitlePageSource::StandIn
      && lookup_value(CLASS_AUTHOR_READER).is_none()
    {
      if self.stand_in {
        return fm_author_tail_declared();
      }
      if let Some(tail) = self.author.tail {
        return Some(tail);
      }
    }
    crate::frontmatter_stores::own_title_page_prints_author().or_else(fm_author_tail_declared)
  }
}

/// The LaTeXML-internal values the reading reads, which the state's watch ([`watch_state`]) does not see:
/// `\@author`'s prefix kept by the frontmatter (`author_taken_prefix`), whether a deposit runs the class's title page
/// (`replays_class_maketitle`) and a raw class's own `\@author` reader ([`CLASS_AUTHOR_READER`]).
const READ_VALUES: [&str; 4] = [
  "lx_author_handed",
  "lx_author_seen",
  "lx_depositing_class_maketitle",
  CLASS_AUTHOR_READER,
];

/// What a reading was made of: the state's watch, begun with it ([`watch_state`]), and [`READ_VALUES`].
type ReadingKey = (u64, [Option<Stored>; 4]);

/// A reading made, with what it was made of.
type KeptReading = (ReadingKey, Rc<TitlePageReading>);

/// The key of a reading made now: it begins the state's watch.
fn reading_key() -> ReadingKey { (watch_state(), read_values()) }

/// The values of [`READ_VALUES`] now.
fn read_values() -> [Option<Stored>; 4] { READ_VALUES.map(lookup_value) }

/// The reading made, while the state it read is unchanged.
fn kept_reading() -> Option<Rc<TitlePageReading>> {
  READING.with_borrow(|r| {
    r.as_ref()
      .filter(|((watch, values), _)| state_unchanged_since(*watch) && *values == read_values())
      .map(|(_, reading)| Rc::clone(reading))
  })
}

thread_local! {
  /// The reading made, with what it was made of ([`reading_key`]).
  static READING: RefCell<Option<KeptReading>> = const { RefCell::new(None) };
  /// What `\@author` held before a document appended to it, as `\lx@author@tail` found it at this `\maketitle`
  /// ([`note_author_prefix`]): the frontmatter's records of it are cleared there, the check reads after.
  static PREFIX: RefCell<Option<Tokens>> = const { RefCell::new(None) };
  /// The run's `\and` family where `\@author` first printed ([`note_family`]).
  static FAMILY: RefCell<Option<[Option<Stored>; 3]>> = const { RefCell::new(None) };
  /// Whether the run reached article.cls's stand-in ([`note_stand_in`]).
  static STAND_IN: Cell<bool> = const { Cell::new(false) };
}

/// `\lx@title@page@family`: the `\and` family's meanings, at `\@author`'s first print in the run.
pub(super) fn note_family() {
  FAMILY.with_borrow_mut(|family| {
    if family.is_none() {
      *family = Some(["\\and", "\\And", "\\AND"].map(|cs| lookup_meaning(&T_CS!(cs))));
    }
  });
}

/// `\lx@title@page@stand@in`: article.cls's stand-in runs.
pub(super) fn note_stand_in() { STAND_IN.set(true); }

/// `\lx@author@tail` (frontmatter_stores.rs): what `\@author` held before a document appended to it at this
/// `\maketitle` (`None`: no tail), kept for the readings that follow it until `\lx@maketitle@cleanup`
/// ([`end_title_page_epoch`]).
pub(crate) fn note_author_prefix(prefix: Option<Tokens>) {
  PREFIX.set(prefix);
  READING.set(None);
}

/// The end of a `\maketitle` (`\lx@maketitle@cleanup`), and of a conversion: its readings and its `\@author` prefix go.
pub(crate) fn end_title_page_epoch() {
  PREFIX.set(None);
  READING.set(None);
}

/// latex.ltx's `\thanks` (ltsect.dtx `\thanks`): a footnote mark, its text appended to `\@thanks`, which only the
/// `\maketitle` that runs `\@thanks` prints (article.cls's, after `\@maketitle`).
const LATEX_THANKS: &str = r"\long\def\thanks#1{\footnotemark\protected@xdef\@thanks{\@thanks\protect\footnotetext[\the\c@footnote]{#1}}}";

/// The title-page reading at this point: the one made while the state is unchanged, else a new run. Its run
/// ([`title_page_print_tokens`], under the title-page check's read budget, `checks_run`): the fields hold the document's
/// values; `\today` prints nothing (the conversion's date is no word of the document's: a class printing `\@date` at its
/// default). An author's note (`\thanks`) is the class's own when a raw file defined one (`\lx@dropped@thanks` where
/// the kernel's lock refused it), else latex.ltx's ([`LATEX_THANKS`]): its text prints where `\@thanks` runs — LaTeX's
/// `\maketitle` (`\lx@print@thanks`), or a class's own that runs it —, set apart as its footnote is (`Ann
/// Able\thanks{Lunar Institute}` is no word "AbleLunar"), and nowhere when the class never runs it (review probe p3:
/// drcls's `\maketitle` runs `\thanks{Draft}` with no `\@thanks`, and pdflatex prints no "Draft"). `\@author` is marked
/// where it begins and where what a document appended to it begins and ends ([`author_marks`]), unless a raw class's own
/// `\@author` reader replaces it in the run.
pub(crate) fn title_page_reading() -> Result<Rc<TitlePageReading>> {
  if let Some(reading) = kept_reading() {
    return Ok(reading);
  }
  let key = reading_key();
  let source = if replays_class_maketitle() {
    TitlePageSource::Replay
  } else if own_maketitle() {
    match lookup_meaning(&T_CS!("\\@maketitle")) {
      Some(Stored::Expandable(d)) => TitlePageSource::Own(d.get_origin()),
      _ => TitlePageSource::StandIn,
    }
  } else {
    TitlePageSource::StandIn
  };
  let class_thanks = |cs: &str| matches!(lookup_meaning(&T_CS!(cs)), Some(Stored::Expandable(ref d)) if !d.get_origin().is_latexml_owned());
  let thanks = if lookup_meaning(&T_CS!("\\lx@dropped@thanks")).is_some() {
    r"\let\thanks\lx@dropped@thanks"
  } else if class_thanks("\\thanks") {
    r"\let\thanks\lx@check@thanks"
  } else {
    LATEX_THANKS
  };
  let mut inject = mouth::tokenize_internal(TeXString::assembled(s!(
    r"\let\today\@empty{thanks}\def\lx@print@thanks{{\@thanks}}"
  )))
  .unlist();
  let marks = author_marks()?;
  let tailed = marks.as_ref().is_some_and(|(_, tailed)| *tailed);
  if let Some((marks, _)) = marks {
    inject.extend(marks);
  }
  // (the class's own `\thanks` kept aside before the run sets the net's in its place)
  let mut tokens = mouth::tokenize_internal(r"{\let\lx@check@thanks\thanks").unlist();
  // (and the Infos building the run gives — a `\maketitle` body not replayable — are the deposit's, which gives them)
  let hold = util::logger::DiagnosticsHold::begin();
  tokens.extend(title_page_print_tokens(&inject).unlist());
  hold.discard();
  tokens.push(T_END!());
  FAMILY.set(None);
  STAND_IN.set(false);
  let run = checks_run(Tokens::new(tokens), false, &[
    (ink_word_text, Some(INK_INSERT)),
    (ink_text, None),
  ])?;
  let family = FAMILY.take();
  let stand_in = STAND_IN.replace(false);
  let (outcome, marked, text) = match run {
    Ok(mut texts) => {
      let text = texts.pop().unwrap_or_default();
      (RunOutcome::Printed, texts.pop().unwrap_or_default(), text)
    },
    Err(SpeculativeRun::OverBudget(read)) => {
      (RunOutcome::OverBudget(read), String::new(), String::new())
    },
    Err(SpeculativeRun::Aborted) => (RunOutcome::Aborted, String::new(), String::new()),
    Err(_) => (RunOutcome::Unavailable, String::new(), String::new()),
  };
  let is_mark = |c: char| matches!(c, INK_AUTHOR | INK_TAIL | INK_TAIL_END);
  let mut untailed = String::with_capacity(marked.len());
  let mut in_tail = false;
  for c in marked.chars() {
    match c {
      INK_TAIL => in_tail = true,
      INK_TAIL_END => in_tail = false,
      _ if in_tail || is_mark(c) => {},
      _ => untailed.push(c),
    }
  }
  let reading = TitlePageReading {
    source,
    stand_in,
    outcome,
    words: marked.chars().filter(|c| !is_mark(*c)).collect(),
    untailed,
    text,
    author: AuthorPrint {
      printed: marked.contains(INK_AUTHOR),
      tail: tailed.then(|| marked.contains(INK_TAIL)),
      family,
    },
  };
  DebugFeature!(
    "frontmatter",
    "FRONT the title page reading: {:?}, stand-in {}, {:?}; \\@author printed {}, its tail {:?}, the \\and family {}",
    reading.source,
    reading.stand_in,
    reading.outcome,
    reading.author.printed,
    reading.author.tail,
    if reading.author.family.is_some() {
      "recorded"
    } else {
      "not reached"
    }
  );
  let reading = Rc::new(reading);
  READING.set(Some((key, Rc::clone(&reading))));
  Ok(reading)
}

/// The definition of `\@author` the reading's run makes, marked ([`title_page_mark`](super::title_page_mark)):
/// `\@author`'s value, after `\lx@title@page@author` (and the `\and` family's note), the part a document appended to it
/// since the frontmatter took its authors — after the value the kernel's `\author` stored (`lx_author_handed`), or the
/// value `\@author` held as a binding's setter queued them (`lx_author_seen`), as `\lx@author@tail` found it at this
/// `\maketitle` ([`note_author_prefix`]) — between `\lx@title@page@tail` and `\lx@title@page@tail@end`; and whether
/// such a part was marked. `None`: `\@author` is empty, takes parameters, or a raw class's own reader replaces it.
fn author_marks() -> Result<Option<(Vec<Token>, bool)>> {
  if lookup_value(CLASS_AUTHOR_READER).is_some() {
    return Ok(None);
  }
  let Some(current) = crate::frontmatter_stores::store_value("author")? else {
    return Ok(None);
  };
  if current.is_empty() {
    return Ok(None);
  }
  let prefix = PREFIX
    .with_borrow(Clone::clone)
    .or_else(crate::frontmatter_stores::author_taken_prefix);
  let current = current.unlist_ref();
  let split = prefix.and_then(|prefix| {
    let prefix = prefix.unlist_ref();
    (prefix.len() < current.len() && current[..prefix.len()] == *prefix).then_some(prefix.len())
  });
  let mut body = vec![
    T_CS!("\\lx@title@page@family"),
    T_CS!("\\lx@title@page@author"),
  ];
  // (a parameter token of the stored value doubled, as a definition's body holds it)
  let push = |body: &mut Vec<Token>, tokens: &[Token]| {
    for token in tokens {
      body.push(*token);
      if token.get_catcode() == Catcode::PARAM {
        body.push(*token);
      }
    }
  };
  match split {
    Some(n) => {
      push(&mut body, &current[..n]);
      body.push(T_CS!("\\lx@title@page@tail"));
      push(&mut body, &current[n..]);
      body.push(T_CS!("\\lx@title@page@tail@end"));
    },
    None => push(&mut body, current),
  }
  let mut def = mouth::tokenize_internal(r"\def\@author{").unlist();
  def.extend(body);
  def.push(T_END!());
  Ok(Some((def, split.is_some())))
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A kept reading goes with a LaTeXML-internal value it read ([`READ_VALUES`]) or any TeX write; another `lx…` value
  /// keeps it (review revTP1 S1).
  #[test]
  fn a_kept_reading_goes_with_what_it_read() {
    set_state(State::new(StateOptions::default()));
    let reading = Rc::new(TitlePageReading {
      source:   TitlePageSource::StandIn,
      stand_in: true,
      outcome:  RunOutcome::Printed,
      words:    String::new(),
      untailed: String::new(),
      text:     String::new(),
      author:   AuthorPrint::default(),
    });
    let writes = READ_VALUES
      .map(|key| (key, false))
      .into_iter()
      .chain([("lx_rk1_unread", true), ("rk1_tex_value", false)]);
    for (key, kept) in writes {
      READING.set(Some((reading_key(), Rc::clone(&reading))));
      assert!(kept_reading().is_some(), "{key}: kept before the write");
      assign_value(key, true, Some(Scope::Global));
      assert_eq!(kept_reading().is_some(), kept, "{key}");
    }
  }
}

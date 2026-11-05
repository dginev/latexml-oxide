//! The title page's printed words against the output (FIDELITY_CHARTER §4, every residual loss counted by a Warning;
//! OXIDIZED_DESIGN_DIVERGENCES #953 "106s"): a check only — it writes no content and changes no XML.
//!
//! At each `\maketitle` the kernel's deposit (sect05.rs `\lx@deposit@maketitle`) runs the whole title page once more,
//! speculatively, as pdflatex runs it — the class's own refused `\maketitle` or its `\@maketitle`, or LaTeX's printing
//! `\@title`, `\@author` and `\@date` — with the document's values (`note_title_page_print`), and notes the text it
//! prints ([`note_title_page_printed`]; the latest before the authors are sealed). When the document closes
//! ([`check_title_page_words`]) every content word of that text ([`net_content_words`], the scorer's) is looked for in
//! the output, cheapest first while one is unheld: the frontmatter's text, the whole document's, the subtrees a
//! `--streaming` run spilled ([`note_spilled_words`]), its attribute values (a contact's `name`, a link's `href`, what
//! a constructor set from an argument). A word the output does not hold is lost; it is counted when the document's own
//! source writes it ([`source_words`]: the files read as content, `binding::content::CONTENT_FILES`, and those of its
//! own directory read as definitions, [`document_source_words`]), else it is the
//! class's text — a label, a placeholder at its default, a running head's — tallied apart in an Info. Every lost word
//! is named in one `Warning:frontmatter:unclaimed`, in printed order; a word the unclaimed-text net already counted
//! (insert.rs `keep_unclaimed_net_notes`, [`note_net_counted`]) is not counted again. Witnesses: 2007.07526 (amsart's
//! author lines), the cpc-hepnp `\email` (1504.05444), raa 1304.5354, memsait astro-ph0102098, JUSTC 2409.17603.
//!
//! Beside the print, a second source (#953 "106t"): what TeX makes of what the document gave its frontmatter commands —
//! the calls of LaTeXML's own macros that read their arguments from the document's text, inside whose expansion the
//! frontmatter API queues an entry ([`note_frontmatter_entry`]) — is wanted too, as digested, never read off the tokens:
//! the entry's own digested content ([`end_frontmatter_entry_digestion`]), and the arguments no entry digests
//! (`never_digested`, and those a binding's macro drops while the entry is digested), digested once speculatively. A class binding that writes its own frontmatter has no print
//! run, and a binding's command may drop what pdflatex prints (quantumarticle's `\ead`, 2209.07123;
//! `\begin{abstract}[Summary]`, elsarticle 2104.14704). What a later call replaced is not wanted
//! ([`note_frontmatter_clear`]).

use std::cell::RefCell;

use super::*;
use crate::latex_constructs::{SpeculativeRun, speculative_word_run};

/// The text the title page printed ([`note_title_page_printed`]).
const PRINTED: &str = "lx_title_page_printed";
/// The words the unclaimed-text net counted in a Warning of its own ([`note_net_counted`]).
const NET_COUNTED: &str = "lx_net_counted_words";

/// Note the text a title page printed (sect05.rs `note_title_page_print`, read by `ink_word_text`): the frontmatter's,
/// in place of an earlier `\maketitle`'s, which it supersedes (DIVERGENCES #406: a later author list replaces the one
/// flushed); none once the authors are sealed, a supplement's title page after the body began, whose entries the seal
/// drops (`seal_digested_frontmatter`).
pub fn note_title_page_printed(text: &str) {
  if frontmatter_entry_sealed("ltx:creator", "author") {
    return;
  }
  assign_value(PRINTED, Stored::String(pin(text)), Some(Scope::Global));
  // (the words spilled so far, cut to those this print wants, which alone the later spills keep)
  let wanted = wanted_words(&resolver::base_letters(text).to_lowercase());
  SPILLED.with_borrow_mut(|spilled| {
    let words = std::mem::take(&mut spilled.words);
    for word in &words {
      claim(&wanted, word, &mut spilled.words);
    }
    spilled.wanted = Some(wanted);
  });
}

/// The content words of the subtrees a `--streaming` run spilled ([`note_spilled_words`]), this conversion's: each
/// distinct word once, and once a title page printed only those it wants (`wanted`), so the notes stay as small as the
/// title page however much the run spills (review rev106s: an 8,000-section document's spill notes, a copy of all the
/// words so far per spilled subtree, ran it out of its memory budget). Reset with the conversion
/// ([`reset_title_page_check`]); held apart from the state, so nothing of it is pinned in the arena or journaled.
#[derive(Default)]
struct SpilledWords {
  words:  HashSet<String>,
  wanted: Option<Wanted>,
}

thread_local! {
  static SPILLED: RefCell<SpilledWords> = RefCell::new(SpilledWords::default());
}

/// Forget the words spilled (`SPILLED`) and what the frontmatter commands were given (`CALLED`): a new conversion starts
/// with none (latexml_oxide's `prepare_session` and `initialize_session`), watching the document's calls
/// ([`note_frontmatter_entry`]).
pub fn reset_title_page_check() {
  SPILLED.with_borrow_mut(|spilled| *spilled = SpilledWords::default());
  CALLED.with_borrow_mut(Vec::clear);
  NEXT_ID.set(0);
  reset_document_calls();
  definition::origin::watch_document_calls(true);
  // (a conversion a caught panic ended inside an entry's digestion left the drop watch on: OD #953 "106t")
  definition::origin::watch_dropped_arguments(false);
}

/// What the document gave one of its frontmatter commands, as digested ([`note_frontmatter_entry`]): the entry queued
/// inside it (its tag and role), the text its digested content prints and that of the arguments it never digests.
struct CalledText {
  /// the record's own number, for [`begin_frontmatter_entry_digestion`]
  id:       usize,
  /// the number of the record of the first entry made inside the same document call: the call's (an `\author`'s
  /// creator, then the contacts it parts off), which a later call that replaces it replaces whole
  /// ([`note_frontmatter_clear`])
  call:     usize,
  tag:      String,
  role:     String,
  /// the entry's queued invocation (`\lx@add@frontmatter@now…{<content>}`), which its digestion is matched to; none for
  /// an entry whose body is digested where it stands (`\lx@add@frontmatter@until`, an abstract)
  command:  Option<Tokens>,
  /// the arguments of the calls the entry was made in that no entry digests (`never_digested`), digested
  /// speculatively once the entry is ([`end_frontmatter_entry_digestion`])
  pending:  Vec<DroppedArgument>,
  /// the arguments of the calls the entry was made in, as the document wrote them: an argument a binding's macro drops
  /// while the entry is digested is the document's text when one of them holds it ([`end_frontmatter_entry_digestion`])
  written:  Vec<CallArguments>,
  /// the entry was digested: `text` is what it prints
  digested: bool,
  text:     String,
}

thread_local! {
  /// What the document gave its frontmatter commands ([`note_frontmatter_entry`]), this conversion's, in call order:
  /// the check's second source, read beside the title page's print; a call a later one replaced taken out
  /// ([`note_frontmatter_clear`]).
  static CALLED: RefCell<Vec<CalledText>> = const { RefCell::new(Vec::new()) };
  /// The number the next record of `CALLED` takes.
  static NEXT_ID: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The frontmatter API removes the entries `tag` (of role `role`, when given) queued before (`queue.rs`
/// `dequeue_front_matter`, `\lx@clear@frontmatter`): what the commands that queued them were given is no longer wanted —
/// the binding declares them replaced, as LaTeX's `\author` replaces the author before it (`\def\@author{#1}`: of
/// article's `\author{Ann Able}\author{Bob Baker}` the title page prints Bob Baker alone, DIVERGENCES #406).
pub fn note_frontmatter_clear(tag: &str, role: Option<&str>) {
  if definition::origin::copy_provides() != 0 {
    return;
  }
  let cleared = |call: &CalledText| call.tag == tag && role.is_none_or(|role| role == call.role);
  CALLED.with_borrow_mut(|called| {
    // (with the entries made inside the same call: the e-mails a replaced author block parted off, perfect_kernel_batch59
    // `author_superseded_keeps_contacts_with_owners`)
    let calls: HashSet<usize> = called
      .iter()
      .filter(|call| call.id == call.call && cleared(call))
      .map(|call| call.id)
      .collect();
    called.retain(|call| !cleared(call) && !calls.contains(&call.call));
  });
}

/// The frontmatter API queues an entry `tag` of role `role` (`queue.rs` `queue_front_matter`, its invocation
/// `command`; `\lx@add@frontmatter@until`, none): when it is made inside the calls of LaTeXML's own macros that read
/// their arguments from the document's own text (`state::claim_open_document_calls`, noted by `Expandable::invoke`),
/// those are the document's frontmatter commands, and what TeX makes of what it gave them is wanted in the output as
/// the title page's print is — a binding's `\maketitle` has no print run (daj, elsarticle, aastex: their bindings write
/// the frontmatter), and a binding's command may drop what pdflatex prints. TeX settles every shape: the entry's own
/// digested content is read when it is digested ([`end_frontmatter_entry_digestion`], by [`ink_word_text`] as the
/// print is), and an argument no entry digests (`never_digested`, or one a binding's macro drops while the entry is
/// digested) is digested once then, speculatively. The record's
/// number, for an entry digested where it stands. What the binding declares is what it places: a command it only
/// stores (a running head's `\def\@shorttitle{…}`, a field its class never prints) queues nothing and is no
/// frontmatter command; nor is a running head the API holds as one (`ltx:toctitle`). Not after the authors are sealed
/// (a supplement's), nor after the title page was made (but an entry printed where it stands, an abstract), nor while a
/// raw class's setter replays, a speculative run digests or a stand-in copy is read.
pub fn note_frontmatter_entry(tag: &str, role: &str, command: Option<&Tokens>) -> Option<usize> {
  if tag == "ltx:toctitle"
    || definition::origin::copy_provides() != 0
    || lookup_bool("lx_replaying_class_setter")
    || frontmatter_entry_sealed("ltx:creator", "author")
    // (an entry queued after the title page was made is printed by none, but one printed where it stands — an
    // abstract after `\maketitle`: lxaddr's late `\author{Bob B}`, perfect_kernel_batch59)
    || (lookup_bool("lx_frontmatter_here_digested") && !PRINTED_WHERE_THEY_STAND.contains(&tag))
  {
    return None;
  }
  let id = NEXT_ID.replace(NEXT_ID.get() + 1);
  let (call, calls, written) = claim_open_document_calls(id)?;
  let mut pending = Vec::new();
  for call in &calls {
    pending.extend(never_digested(call));
  }
  CALLED.with_borrow_mut(|called| {
    called.push(CalledText {
      id,
      call,
      tag: tag.to_string(),
      role: role.to_string(),
      command: command.cloned(),
      pending,
      written,
      digested: false,
      text: String::new(),
    });
    Some(id)
  })
}

/// The arguments of the document's frontmatter command `call` that no entry digests, which the check digests itself
/// ([`note_frontmatter_entry`]): a required one the macro's body never names (`state::DocumentArgument::used`: the
/// binding drops it — quantumarticle's `\ead[]{}` → `\email{#1}` passes on the optional label, not the address,
/// KNOWN_PERL_ERRORS #1208, 2209.07123; not one the binding declares dropped by design, `drops => [n]`: aomart's
/// `\subject` scheme, which the class prints only under `printscheme`), and an optional one an internal command (a name with `@`) reads from the
/// document's text, which is the document's text pdflatex prints: `\begin{abstract}[Summary]`, where LaTeX's
/// `{abstract}` takes none and the kernel's `\lx@begin@abstract[]` reads it as the entry's attributes (#1209,
/// elsarticle 2104.14704). Every other argument is handed on, digested into an entry, whose text is read; an optional
/// argument of a command the document writes is LaTeX's, unprinted (`\title[<running head>]`, `\author[<labels>]`).
fn never_digested(call: &DocumentCall) -> impl Iterator<Item = DroppedArgument> + '_ {
  let internal = call.cs.with_cs_name(|name| name.contains('@'));
  call
    .args
    .iter()
    .filter(move |arg| !arg.tokens.is_empty() && if arg.optional { internal } else { !arg.used })
    .map(|arg| DroppedArgument {
      cs:     call.cs,
      number: arg.number,
      tokens: arg.tokens.clone(),
    })
}

/// The watch over the digestion of an entry a frontmatter command of the document's queued
/// ([`begin_frontmatter_entry_digestion`]).
pub struct EntryDigestion {
  /// the record's number
  id:   usize,
  /// the count of dropped arguments when it began (`state::dropped_argument_count`)
  from: usize,
  /// whether the watch was on before it (an entry digested inside another's)
  was:  bool,
}

/// The frontmatter API is about to digest an entry (`\lx@add@frontmatter@now`, `\lx@annotate@frontmatter@now`:
/// `content`, its queued content; `\lx@add@frontmatter@until`: the record `id` [`note_frontmatter_entry`] gave): when a
/// frontmatter command of the document's queued it, the arguments a binding's macro drops while it is digested are
/// watched (`state::note_dropped_argument`: IEEEtran's `\lx@IEEE@membership@drop` over a grade it cannot place,
/// 1511.03982; jmlr's `\titletag[1]{}` over a published-version note, 2211.07413) until
/// [`end_frontmatter_entry_digestion`]. Not while a raw class's setter replays or a speculative run digests.
pub fn begin_frontmatter_entry_digestion(
  content: Option<&Tokens>,
  id: Option<usize>,
) -> Option<EntryDigestion> {
  if lookup_bool("lx_replaying_class_setter") {
    return None;
  }
  let id = CALLED.with_borrow_mut(|called| {
    let call = called.iter_mut().find(|call| {
      !call.digested
        && match (id, content, call.command.as_ref()) {
          (Some(id), ..) => call.id == id,
          (None, Some(content), Some(command)) => queued_content_is(command, content),
          _ => false,
        }
    })?;
    call.digested = true;
    Some(call.id)
  })?;
  Some(EntryDigestion {
    id,
    from: dropped_argument_count(),
    was: definition::origin::watch_dropped_arguments(true),
  })
}

/// The frontmatter API digested the entry `watch` watches ([`begin_frontmatter_entry_digestion`]) into `digested` (none
/// when its digestion failed or the API removed it, printing nothing): what it prints is wanted ([`ink_word_text`], as
/// the title page's print is read), and then what the arguments no entry digests print — a frontmatter command's
/// (`never_digested`) and those a binding's macro dropped while it was digested —, each digested once,
/// speculatively, as the title page's run is (sect05.rs `speculative_word_run`): when no run can be made or kept,
/// nothing of it is wanted.
pub fn end_frontmatter_entry_digestion(
  watch: Option<EntryDigestion>,
  digested: Option<&Digested>,
) -> Result<()> {
  let Some(EntryDigestion { id, from, was }) = watch else {
    return Ok(());
  };
  definition::origin::watch_dropped_arguments(was);
  let dropped = take_dropped_arguments(from);
  let Some(digested) = digested else {
    return Ok(());
  };
  let Some(pending) = CALLED.with_borrow_mut(|called| {
    let call = record_mut(called, id)?;
    ink_word_text(digested, &mut call.text);
    let mut pending = std::mem::take(&mut call.pending);
    // (only what the document wrote: a binding's own plumbing — xkeyval's `\@nil`-delimited pieces, 2101.10506 — is no
    // text of the document's)
    pending.extend(
      dropped
        .into_iter()
        .filter(|dropped| writes(&call.written, &dropped.tokens)),
    );
    Some(pending)
  }) else {
    return Ok(());
  };
  let mut text = String::new();
  for DroppedArgument { cs, number, tokens } in pending {
    DebugFeature!(
      "frontmatter",
      "FRONT the check digests {cs}'s #{number}, which no entry digests: {tokens}"
    );
    // (a note's text set apart from the text around its mark, as its footnote prints it: jmlr's
    // `\titletag{\thanks{Published as an article in …}}`, 2211.07413)
    let mut run = vec![T_BEGIN!()];
    run.extend(mouth::tokenize_internal(DROPPED_THANKS).unlist());
    run.extend(tokens.unlist());
    run.push(T_END!());
    if let SpeculativeRun::Printed(printed) = speculative_word_run(Tokens::new(run))? {
      text.push('\n');
      text.push_str(&printed);
    }
  }
  CALLED.with_borrow_mut(|called| {
    if let Some(call) = record_mut(called, id) {
      call.text.push_str(&text);
      // (a print noted before cut the spill notes to its words: these are wanted too)
      let words = wanted_words(&resolver::base_letters(&call.text).to_lowercase()).words;
      SPILLED.with_borrow_mut(|spilled| {
        if let Some(wanted) = spilled.wanted.as_mut() {
          wanted.words.extend(words);
        }
      });
    }
  });
  Ok(())
}

/// The record `id` of `CALLED` (whose records are in number order: [`note_frontmatter_entry`]).
fn record_mut(called: &mut [CalledText], id: usize) -> Option<&mut CalledText> {
  let at = called.binary_search_by_key(&id, |call| call.id).ok()?;
  called.get_mut(at)
}

/// Does one of the arguments of the calls `written` hold the tokens `tokens` as written, its spaces at either end aside?
fn writes(written: &[CallArguments], tokens: &Tokens) -> bool {
  let tokens = tokens.unlist_ref();
  let space = |token: &Token| token.get_catcode() == Catcode::SPACE;
  let Some(start) = tokens.iter().position(|token| !space(token)) else {
    return false;
  };
  let end = tokens
    .iter()
    .rposition(|token| !space(token))
    .map_or(start, |end| end + 1);
  let tokens = &tokens[start..end];
  written.iter().flat_map(|call| call.iter()).any(|argument| {
    argument
      .tokens
      .unlist_ref()
      .windows(tokens.len())
      .any(|window| window == tokens)
  })
}

/// LaTeX's `\thanks` in a dropped argument's run ([`end_frontmatter_entry_digestion`]): its text, apart from the words
/// around its mark.
const DROPPED_THANKS: &str = r"\long\def\thanks#1{ #1 }";

/// Is `content` the content of the queued invocation `command` (`…{<content>}`, its last argument)?
fn queued_content_is(command: &Tokens, content: &Tokens) -> bool {
  let (command, content) = (command.unlist_ref(), content.unlist_ref());
  command.len() >= content.len() + 2
    && command
      .last()
      .is_some_and(|t| t.get_catcode() == Catcode::END)
    && command[command.len() - content.len() - 2].get_catcode() == Catcode::BEGIN
    && command[command.len() - content.len() - 1..command.len() - 1] == *content
}

/// How many distinct words the spill notes hold (`SPILLED`), for `--debug=frontmatter`.
fn spilled_word_count() -> usize { SPILLED.with_borrow(|spilled| spilled.words.len()) }

/// Add to `claimed` what the output word `word` claims of `wanted`: itself when the print wants it, and each word the
/// print's own text opens or ends with that `word` continues ([`continues`]).
fn claim(wanted: &Wanted, word: &str, claimed: &mut HashSet<String>) {
  if wanted.words.contains(word) {
    claimed.insert(word.to_string());
  }
  for open in wanted
    .opens
    .iter()
    .filter(|open| continues(word, open, false))
  {
    claimed.insert(open.clone());
  }
  for end in wanted.ends.iter().filter(|end| continues(word, end, true)) {
    claimed.insert(end.clone());
  }
}

/// Note the words the unclaimed-text net named in its Warning (insert.rs `keep_unclaimed_net_notes`): counted once.
pub(super) fn note_net_counted(words: &[String]) {
  let mut all = lookup_string(NET_COUNTED);
  for word in words {
    all.push(' ');
    all.push_str(word);
  }
  assign_value(NET_COUNTED, Stored::String(pin(all)), Some(Scope::Global));
}

/// The document's frontmatter elements, whose text [`check_title_page_words`] reads first: the title page's words are
/// mostly theirs.
const FRONT_ELEMENTS: &[&str] = &[
  "title",
  "subtitle",
  "toctitle",
  "creator",
  "date",
  "abstract",
  "keywords",
  "classification",
  "note",
  "titlepage",
];

/// The mark [`output_text`] sets at an element's edges: a word of the output may run across it (`Bob
/// <text>Bak</text>and`, "Bakand") or end at it (`…Labs</creator><creator>Bob…`).
const EDGE: char = '\u{2}';

/// The text of `node`'s descendants, an [`EDGE`] at each element's edges, and apart (`attributes`, when given) their
/// attribute values ([`attribute_text`]): an attribute between two pieces of a word does not part them
/// (`<XMTok>p</XMTok><XMTok role=…>p</XMTok>`, "pp"). No comment's text.
fn output_text(node: &Node, out: &mut String, attributes: &mut Option<String>) {
  for child in node.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::ElementNode) => {
        if let Some(attributes) = attributes.as_mut() {
          attribute_text(&child, attributes);
        }
        out.push(EDGE);
        output_text(&child, out, attributes);
        out.push(EDGE);
      },
      Some(NodeType::TextNode | NodeType::CDataSectionNode) => out.push_str(&child.get_content()),
      _ => {},
    }
  }
}

/// The attributes whose value is no word the output holds: a formula's source and its text rendering (`tex`, `text`:
/// long, and the formula's letters are its tokens'), an element's id.
const WORDLESS_ATTRIBUTES: &[&str] = &["tex", "text", "xml:id"];

/// The values of `node`'s attributes, but our internal ones (`_…`) and [`WORDLESS_ATTRIBUTES`], each set apart by
/// spaces: a contact's or a heading's `name` ("Email:"), a link's target, and what a constructor set from an argument
/// that never prints (`\@@cite{cite}`'s `class="ltx_citemacro_cite"`, a rule's `width="0pt"`).
fn attribute_text(node: &Node, out: &mut String) {
  for (name, value) in node.get_attributes() {
    if !name.starts_with('_') && !WORDLESS_ATTRIBUTES.contains(&name.as_str()) {
      out.push(' ');
      out.push_str(&value);
      out.push(' ');
    }
  }
}

/// The most pieces of a run [`output_words`] joins into a word: a word set in pieces by markup (a font, a
/// superscript, an accent) holds a few.
const JOINED_PIECES: usize = 6;

/// The words of the title page's print the check looks for in the output ([`output_words`]): its content words, and
/// those its first and last runs open and end with against no space (`opens`, `ends`), which an output word may
/// continue (`unclaimed_words`).
#[derive(Default)]
struct Wanted {
  words: HashSet<String>,
  opens: Vec<String>,
  ends:  Vec<String>,
}

/// The shortest word the title page's first or last run holds that an output word may continue ([`continues`]): a
/// shorter one is too common a piece of other words to read so (an address's `com`, `cn`).
const RUN_ON_LEAST: usize = 4;

/// Does the output word `word` continue the print's edge word `edge` — begin with it, the print's last (`ends`), or end
/// with it, its first — being longer, `edge` of [`RUN_ON_LEAST`] characters or more ("shownbody" continues "shown")?
fn continues(word: &str, edge: &str, ends: bool) -> bool {
  edge.chars().count() >= RUN_ON_LEAST
    && word.len() > edge.len()
    && if ends {
      word.starts_with(edge)
    } else {
      word.ends_with(edge)
    }
}

/// The content words of `node` and what it holds ([`output_text`], folded): of each run of letters and digits, the
/// words of every stretch of up to [`JOINED_PIECES`] consecutive pieces between element edges — so both "Bak" and
/// "Bakand" of `Bob <text>Bak</text>and`, and "Labs" and "Bob" across two creators; with `attributes`, their attribute
/// values' words too. With `wanted`, only its words are kept, and those it opens or ends with that an output word ends
/// or begins with.
fn output_words(node: &Node, wanted: Option<&Wanted>, attributes: bool) -> HashSet<String> {
  let mut text = String::new();
  let mut values = attributes.then(String::new);
  if let Some(values) = values.as_mut() {
    attribute_text(node, values);
  }
  output_text(node, &mut text, &mut values);
  if let Some(values) = values {
    text.push(' ');
    text.push_str(&values);
  }
  let folded = if text.is_ascii() {
    text.to_ascii_lowercase()
  } else {
    resolver::base_letters(&text).to_lowercase()
  };
  let mut words = HashSet::default();
  let mut keep = |word: &str| match wanted {
    None => {
      if !words.contains(word) {
        words.insert(word.to_string());
      }
    },
    Some(wanted) => claim(wanted, word, &mut words),
  };
  let mut joined = String::new();
  for run in folded.split(|c: char| !(c.is_alphanumeric() || c == EDGE)) {
    let pieces: Vec<&str> = run.split(EDGE).filter(|piece| !piece.is_empty()).collect();
    for from in 0..pieces.len() {
      joined.clear();
      for piece in pieces.iter().skip(from).take(JOINED_PIECES) {
        joined.push_str(piece);
        // (a stretch of ASCII letters is its own one word, or none: no allocation for it)
        if joined.bytes().all(|b| b.is_ascii_lowercase()) {
          if joined.len() >= 2 {
            keep(&joined);
          }
        } else {
          for word in net_content_words(&joined) {
            keep(&word);
          }
        }
      }
    }
  }
  words
}

/// Note the content words of a subtree a `--streaming` run spills (latexml_oxide's `Document::spill_observer`): the
/// output holds them, though the document's tree no longer does when it closes. What it spills before a title page
/// printed is noted too, each distinct word once: the title page may print it (an abstract set before `\maketitle`);
/// after, only the words the print wants (`SPILLED`). A later title page superseding the first wants its own words
/// among those the first kept (a spill between two `\maketitle`s of words only the second prints is not seen).
pub fn note_spilled_words(node: &Node) {
  SPILLED.with_borrow_mut(|spilled| {
    let SpilledWords { words, wanted } = spilled;
    words.extend(output_words(node, wanted.as_ref(), true));
  });
}

/// The runs of letters and digits of the folded print `folded` ([`resolver::base_letters`]; an [`INK_SEAM`] inside
/// a run), and the indices of the ones its own text opens and ends with, against no space — the title page's text ends
/// where the page foot's begins (`INK_INSERT`).
fn print_runs(folded: &str) -> (Vec<&str>, usize) {
  let in_run = |c: char| c.is_alphanumeric() || c == INK_SEAM;
  let runs: Vec<&str> = folded.split(|c: char| !in_run(c)).collect();
  let last = folded
    .split(INK_INSERT)
    .next()
    .map_or(0, |main| main.split(|c: char| !in_run(c)).count())
    .saturating_sub(1);
  (runs, last)
}

/// The words [`output_words`] looks for: every content word of the print's runs and of their pieces, and those of its
/// first and last runs.
fn wanted_words(folded: &str) -> Wanted {
  let (runs, last) = print_runs(folded);
  let mut wanted = Wanted::default();
  for (at, run) in runs.iter().enumerate() {
    let whole: String = run.chars().filter(|c| *c != INK_SEAM).collect();
    let words = net_content_words(&whole);
    if at == 0 {
      wanted.opens.extend(words.iter().cloned());
    }
    if at == last {
      wanted.ends.extend(words.iter().cloned());
    }
    wanted.words.extend(words);
    if run.contains(INK_SEAM) {
      wanted
        .words
        .extend(run.split(INK_SEAM).flat_map(net_content_words));
    }
  }
  wanted
}

/// The title page's printed words the output does not hold, distinct, in printed order: of each run of letters and
/// digits in `printed` (folded, [`resolver::base_letters`]), its content words ([`net_content_words`]) `claimed`
/// lacks. Where the run crosses the edge of a whatsit that prints (`INK_SEAM`), its pieces are the words when the
/// output holds one of them (`\centerline{Ann Able}\centerline{MIT}`: "ablemit" is "able" and "mit"), or when the
/// whole is no word the document's source writes (`source`, [`source_words`]: an address before a note's mark,
/// "com" and "footnote"); else the whole is (an accent's whatsit inside `Jos\'e`). `folded`: the print, folded
/// ([`resolver::base_letters`]); `claimed`: its words the output holds ([`output_words`]), among them a word its own
/// text opens or ends with against no space that an output word ends or begins with — it runs on in the document's
/// text around the title page (`\maketitle` swallows the line end after it: `SHOWN` and `Body.` are "SHOWNBody.").
fn unclaimed_words(
  folded: &str,
  claimed: &HashSet<String>,
  source: &mut dyn FnMut() -> HashSet<String>,
) -> Vec<String> {
  let (runs, _) = print_runs(folded);
  let mut lost: Vec<String> = Vec::new();
  let mut written: Option<HashSet<String>> = None;
  for run in runs {
    if run.is_empty() {
      continue;
    }
    let whole: String = run.chars().filter(|c| *c != INK_SEAM).collect();
    let mut missing: Vec<String> = net_content_words(&whole)
      .into_iter()
      .filter(|word| !claimed.contains(word))
      .collect();
    if missing.is_empty() {
      continue;
    }
    if run.contains(INK_SEAM) {
      let pieces: Vec<String> = run.split(INK_SEAM).flat_map(net_content_words).collect();
      let written = written.get_or_insert_with(&mut *source);
      if pieces.iter().any(|word| claimed.contains(word))
        || !missing.iter().all(|word| written.contains(word))
      {
        missing = pieces
          .into_iter()
          .filter(|word| !claimed.contains(word))
          .collect();
      }
    }
    for word in missing {
      if !lost.contains(&word) {
        lost.push(word);
      }
    }
  }
  lost
}

/// The text symbols a control word prints as letters (OT1's `\o`, `\ss`, `\ae`, …), for [`source_words`]: `Bj{\o}rn`
/// is "Bjorn".
const LETTER_SYMBOLS: &[(&str, &str)] = &[
  ("o", "o"),
  ("O", "O"),
  ("l", "l"),
  ("L", "L"),
  ("i", "i"),
  ("j", "j"),
  ("ss", "ss"),
  ("ae", "ae"),
  ("AE", "AE"),
  ("oe", "oe"),
  ("OE", "OE"),
  ("aa", "aa"),
  ("AA", "AA"),
  ("dj", "d"),
  ("DJ", "D"),
];

/// The accents written as a control word of one letter (`\c{c}`, `\v{s}`, `\H{o}`), which print no letter of their own.
const ACCENT_WORDS: &[&str] = &["c", "v", "u", "H", "d", "b", "t", "k", "r"];

/// How many required arguments of the control word `name` are keys or options, which print no word of the document's
/// (its optional ones all are): a class's or package's name and options (`\documentclass[draft]{…}`: review probe p4,
/// where a class prints "Draft" for the option), a cross-reference's or citation's key (`\label`, `\ref`, `\eqref`,
/// `\cref`, `\cite`, `\citep`, …), a file's name, a counter's, a key-value family's, and the hyphenation exceptions a
/// class lists (au.cls's `\hyphenation{… key-stroke}`, 1205.1106: its "Key words" label is the class's). `None` for
/// any other.
fn keyed_arguments(name: &str) -> Option<usize> {
  match name {
    "PassOptionsToPackage" | "PassOptionsToClass" | "define@key" => Some(2),
    "hyphenation" | "ExecuteOptions" => Some(1),
    "documentclass"
    | "documentstyle"
    | "usepackage"
    | "RequirePackage"
    | "RequirePackageWithOptions"
    | "LoadClass"
    | "LoadClassWithOptions"
    | "ProvidesClass"
    | "ProvidesPackage"
    | "ProvidesFile"
    | "NeedsTeXFormat"
    | "DeclareOption"
    | "label"
    | "bibliography"
    | "bibliographystyle"
    | "bibitem"
    | "input"
    | "include"
    | "includeonly"
    | "IfFileExists"
    | "InputIfFileExists"
    | "includegraphics"
    | "graphicspath"
    | "usetikzlibrary"
    | "hypersetup"
    | "newcounter"
    | "setcounter"
    | "addtocounter"
    | "stepcounter"
    | "refstepcounter"
    | "pagestyle"
    | "thispagestyle" => Some(1),
    _ if name != "hyperref" && name.ends_with("ref") => Some(1),
    _ if name.to_ascii_lowercase().contains("cite") => Some(1),
    _ => None,
  }
}

/// The definers whose replacement texts a definitions file's words leave out ([`source_words`]), by the shape of what
/// follows them: `\def\cs<parameters>{…}`, `\newcommand{\cs}[n][default]{…}`, `\newenvironment{name}[n]{…}{…}`, and
/// xparse's, which take an argument specification before the body.
#[derive(Clone, Copy)]
enum Definer {
  Def,
  Command,
  Environment,
  DocumentCommand,
  DocumentEnvironment,
}

impl Definer {
  fn of(name: &str) -> Option<Definer> {
    Some(match name {
      "def" | "gdef" | "edef" | "xdef" => Definer::Def,
      "newcommand" | "renewcommand" | "providecommand" | "DeclareRobustCommand" | "@namedef" => {
        Definer::Command
      },
      "newenvironment" | "renewenvironment" => Definer::Environment,
      "NewDocumentCommand"
      | "RenewDocumentCommand"
      | "ProvideDocumentCommand"
      | "DeclareDocumentCommand" => Definer::DocumentCommand,
      "NewDocumentEnvironment"
      | "RenewDocumentEnvironment"
      | "ProvideDocumentEnvironment"
      | "DeclareDocumentEnvironment" => Definer::DocumentEnvironment,
      _ => return None,
    })
  }
}

/// A reader of source text for [`source_words`], its comments out.
struct SourceScan {
  chars: Vec<char>,
  at:    usize,
}

impl SourceScan {
  fn peek(&self) -> Option<char> { self.chars.get(self.at).copied() }

  fn skip_spaces(&mut self) {
    while self.peek().is_some_and(char::is_whitespace) {
      self.at += 1;
    }
  }

  /// Skip a group from `open` to its `close`, nested, an escaped character (`\}`) none of them; `false` when the next
  /// character does not open one.
  fn skip_group(&mut self, open: char, close: char) -> bool {
    if self.peek() != Some(open) {
      return false;
    }
    let mut depth = 0usize;
    while let Some(c) = self.peek() {
      self.at += 1;
      if c == '\\' {
        self.at += 1;
      } else if c == open {
        depth += 1;
      } else if c == close {
        depth -= 1;
        if depth == 0 {
          break;
        }
      }
    }
    true
  }

  /// Skip a control sequence (`\cs`, `\@x`, `\{`) or one character.
  fn skip_token(&mut self) {
    if self.peek() == Some('\\') {
      self.at += 1;
      let from = self.at;
      while self
        .peek()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '@')
      {
        self.at += 1;
      }
      if self.at == from {
        self.at += 1;
      }
    } else {
      self.at += 1;
    }
  }

  /// Skip the optional arguments ahead (`[…]`, a star).
  fn skip_options(&mut self) {
    loop {
      self.skip_spaces();
      if self.peek() == Some('*') {
        self.at += 1;
      } else if !self.skip_group('[', ']') {
        break;
      }
    }
  }

  /// Skip a name: a braced one (`{\cs}`, `{env}`) or a control sequence.
  fn skip_name(&mut self) {
    self.skip_spaces();
    if !self.skip_group('{', '}') {
      self.skip_token();
    }
  }

  /// Skip `count` required arguments, and the optional ones around them.
  fn skip_arguments(&mut self, count: usize) {
    for _ in 0..count {
      self.skip_options();
      if !self.skip_group('{', '}') {
        break;
      }
    }
    self.skip_options();
  }

  /// Skip what follows a definer: the name defined, its parameters and its replacement text(s).
  fn skip_definition(&mut self, definer: Definer) {
    match definer {
      Definer::Def => {
        self.skip_spaces();
        self.skip_token();
        while self.peek().is_some_and(|c| c != '{') {
          self.skip_token();
        }
        self.skip_group('{', '}');
      },
      Definer::Command => {
        self.skip_options();
        self.skip_name();
        self.skip_arguments(1);
      },
      Definer::Environment => {
        self.skip_options();
        self.skip_name();
        self.skip_arguments(2);
      },
      Definer::DocumentCommand => {
        self.skip_name();
        self.skip_arguments(2);
      },
      Definer::DocumentEnvironment => {
        self.skip_name();
        self.skip_arguments(3);
      },
    }
  }
}

/// The words the document's own source writes: the content words of `text` read as letters — its comments, a
/// `filecontents` environment's body (a file it writes, a class's or a package's text), the names of `\begin`/`\end`'s
/// environments and the keys and options of [`keyed_arguments`] out; a control word nothing and the spaces after it
/// skipped, as TeX skips them (`Babe\cb{s}` "Babes", 2007.07526), but a text symbol its letters ([`LETTER_SYMBOLS`]:
/// `Bj{\o}rn` "Bjorn") — and, since some print a space (`\quad`), a space too, in a second reading; an accent nothing
/// (`M\"{u}ller` "Muller", `Fran\c{c}ois` "Francois"); the braces read both as nothing and as a space (`\author{Ann}{MIT}`,
/// `{\bf A}ble`). With `definitions` (a package's or class's file of the document's: [`document_source_words`]), the
/// replacement texts of its definitions out too ([`Definer`]): a class's label is its own text, not the document's.
fn source_words(text: &str, definitions: bool, words: &mut HashSet<String>) {
  let mut plain = String::new();
  let mut rest = text;
  while let Some(at) = rest.find("\\begin{filecontents") {
    plain.push_str(&rest[..at]);
    rest = &rest[at..];
    match rest.find("\\end{filecontents") {
      Some(end) => {
        rest = &rest[end + 1..];
        rest = rest.find('}').map_or("", |close| &rest[close + 1..]);
      },
      None => rest = "",
    }
  }
  plain.push_str(rest);
  // (the comments out, each line ending in a space)
  let mut chars = Vec::with_capacity(plain.len());
  for line in plain.lines() {
    let mut line = line.chars();
    while let Some(c) = line.next() {
      match c {
        '%' => break,
        '\\' => {
          chars.push(c);
          chars.extend(line.next());
        },
        c => chars.push(c),
      }
    }
    chars.push('\n');
  }
  let mut scan = SourceScan { chars, at: 0 };
  let (mut glued, mut spaced) = (String::new(), String::new());
  while let Some(c) = scan.peek() {
    scan.at += 1;
    match c {
      '\\' => {
        let from = scan.at;
        while scan
          .peek()
          .is_some_and(|next| next.is_ascii_alphabetic() || next == '@')
        {
          scan.at += 1;
        }
        let name: String = scan.chars[from..scan.at].iter().collect();
        // (a control word the author spaced off ends the word in the glued reading too: `Ann Able\break and Bob` is
        // "Able" and "and" — `\break` digests to nothing, so the print runs them on, and only the source's own space
        // keeps "ableand" from reading as the document's word (batch77d `author_and_after_a_break_in_a_wrapper.tex`);
        // `Babe\cb{s}` written on runs on, "Babes", 2007.07526)
        let spaced_off = scan.peek().is_some_and(char::is_whitespace);
        if name.is_empty() {
          // (a control symbol: an accent's, a discretionary's, an italic correction's and `\%` print no letter, any
          // other a space)
          let accent = matches!(scan.peek(), Some('\'' | '`' | '^' | '"' | '~' | '=' | '.'));
          match scan.peek() {
            Some('\'' | '`' | '^' | '"' | '~' | '=' | '.' | '%' | '-' | '/') | None => {},
            Some(_) => {
              glued.push(' ');
              spaced.push(' ');
            },
          }
          scan.at += 1;
          // (an accent reads its argument past the spaces before it: `Universit{\" a}t` is "Universitat", 2406.18504)
          if accent {
            scan.skip_spaces();
          }
          continue;
        }
        scan.skip_spaces();
        if name == "begin" || name == "end" {
          // (the environment's name: `\begin{abstract}`)
          scan.skip_group('{', '}');
          glued.push(' ');
          spaced.push(' ');
        } else if let Some(count) = keyed_arguments(&name) {
          scan.skip_arguments(count);
          glued.push(' ');
          spaced.push(' ');
        } else if let Some(definer) = Definer::of(&name).filter(|_| definitions) {
          scan.skip_definition(definer);
          glued.push(' ');
          spaced.push(' ');
        } else if let Some((_, letters)) = LETTER_SYMBOLS.iter().find(|(n, _)| *n == name) {
          glued.push_str(letters);
          spaced.push_str(letters);
        } else if !ACCENT_WORDS.contains(&name.as_str()) {
          spaced.push(' ');
          if spaced_off {
            glued.push(' ');
          }
        }
      },
      '{' | '}' => spaced.push(' '),
      c => {
        glued.push(c);
        spaced.push(c);
      },
    }
  }
  words.extend(net_content_words(&glued));
  words.extend(net_content_words(&spaced));
}

/// The text of the file at `path`: UTF-8, else Latin-1 (an older paper's source in ISO-8859-1, 0802.2579: its accented
/// letters are its words', never a replacement character parting them).
fn read_source(path: &str) -> Option<String> {
  let bytes = std::fs::read(path).ok()?;
  Some(
    String::from_utf8(bytes)
      .unwrap_or_else(|e| e.into_bytes().into_iter().map(char::from).collect()),
  )
}

/// The words the document's own source writes ([`source_words`]): its content files' (`binding::content::CONTENT_FILES`:
/// a virtual file's text, `filecontents`, else the file's on disk; a `literal:` source its own text), and those of
/// each file its own directory holds that was read as definitions (`binding::content::raw_read_paths`: a style the
/// author wrote, `\AtBeginDocument{\corr{ann.able@lunar.org}}`, review probe p7_sty), their definitions' replacement
/// texts out. A file a `filecontents` environment wrote is not read again: its body is a class's or a package's text.
fn document_source_words() -> HashSet<String> {
  let mut words = HashSet::default();
  let files = lookup_string(CONTENT_FILES);
  for file in files.split('\0').filter(|file| !file.is_empty()) {
    if let Some(text) = file.strip_prefix("literal:") {
      source_words(text, false, &mut words);
    } else if let Some(text) = vfs_read(file) {
      source_words(&text, false, &mut words);
    } else if let Some(text) = read_source(file) {
      source_words(&text, false, &mut words);
    }
  }
  let source_directory = lookup_string("SOURCEDIRECTORY");
  if source_directory.is_empty() {
    return words;
  }
  let Ok(directory) = std::fs::canonicalize(&source_directory) else {
    return words;
  };
  for file in raw_read_paths() {
    let path = std::path::Path::new(&file);
    if path.starts_with(&directory)
      && vfs_read(&file).is_none()
      && path.is_file()
      && let Some(text) = read_source(&file)
    {
      source_words(&text, true, &mut words);
    }
  }
  words
}

/// The check, when the document closes: the words its title pages printed ([`note_title_page_printed`]) that its
/// output does not hold (`unclaimed_words`), but those the unclaimed-text net counted. Those the document's source
/// writes (`document_source_words`) are lost, named in one `Warning:frontmatter:unclaimed` in printed order; the
/// others are the class's own text (a label, a placeholder at its class default, a running head's words), tallied in
/// an Info (`frontmatter:class_text`, the placeholder ruling pending). Writes nothing.
pub fn check_title_page_words(document: &Document) -> Result<()> {
  // (and after it what the document gave its frontmatter commands, the second source: [`note_frontmatter_entry`])
  let print = lookup_string(PRINTED);
  let mut printed = print.clone();
  CALLED.with_borrow(|called| {
    for call in called {
      printed.push(INK_INSERT);
      printed.push_str(&call.text);
    }
  });
  if printed
    .trim_matches(|c: char| c.is_whitespace() || c == INK_INSERT)
    .is_empty()
  {
    return Ok(());
  }
  let Some(root) = document.get_document().get_root_element() else {
    return Ok(());
  };
  let folded = resolver::base_letters(&printed).to_lowercase();
  let wanted = wanted_words(&folded);
  DebugFeature!(
    "frontmatter",
    "FRONT the spill notes hold {} word(s)",
    spilled_word_count()
  );
  // (in three steps, each only while a printed word is unheld — cheapest first: the frontmatter's text and attribute
  // values, the whole document's text, its attribute values)
  let front: Vec<Node> = element_nodes(&root)
    .into_iter()
    .filter(|child| FRONT_ELEMENTS.contains(&child.get_name().as_str()))
    .collect();
  let mut claimed: HashSet<String> = HashSet::default();
  for node in &front {
    // (with their attributes: a contact's `name`, an e-mail's `href`, which the frontmatter commands' words often are)
    claimed.extend(output_words(node, Some(&wanted), true));
  }
  let mut source: Option<HashSet<String>> = None;
  let mut held_all = |claimed: &HashSet<String>| {
    unclaimed_words(&folded, claimed, &mut || {
      source.get_or_insert_with(document_source_words).clone()
    })
    .is_empty()
  };
  if held_all(&claimed) {
    return Ok(());
  }
  DebugFeature!(
    "frontmatter",
    "FRONT the check's words the frontmatter does not hold: {}",
    unclaimed_words(&folded, &claimed, &mut HashSet::default).join(" ")
  );
  claimed.extend(output_words(&root, Some(&wanted), false));
  SPILLED.with_borrow(|spilled| {
    for word in &spilled.words {
      claim(&wanted, word, &mut claimed);
    }
  });
  for word in lookup_string(NET_COUNTED).split(' ') {
    claim(&wanted, word, &mut claimed);
  }
  if held_all(&claimed) {
    return Ok(());
  }
  claimed.extend(output_words(&root, Some(&wanted), true));
  let unclaimed = unclaimed_words(&folded, &claimed, &mut || {
    source.get_or_insert_with(document_source_words).clone()
  });
  if unclaimed.is_empty() {
    return Ok(());
  }
  let source = source.unwrap_or_else(document_source_words);
  let (lost, class): (Vec<String>, Vec<String>) = unclaimed
    .into_iter()
    .partition(|word| source.contains(word));
  if !lost.is_empty() {
    // (the words the frontmatter commands were given alone, not the print's, are said so)
    let print_words = wanted_words(&resolver::base_letters(&print).to_lowercase()).words;
    let printed = lost
      .iter()
      .filter(|word| print_words.contains(*word))
      .count();
    let source = if printed == lost.len() {
      "The title page prints"
    } else if printed == 0 {
      "The document's frontmatter commands were given"
    } else {
      "The title page prints, or the document's frontmatter commands were given,"
    };
    Warn!(
      "frontmatter",
      "unclaimed",
      s!(
        "{source} {} word(s){} that its output does not hold: {}",
        lost.len(),
        if printed == 0 {
          ""
        } else {
          " of the document's"
        },
        lost.join(" ")
      )
    );
  }
  if !class.is_empty() {
    Info!(
      "frontmatter",
      "class_text",
      s!(
        "The title page prints {} word(s) of the class's own text (a label, a placeholder at its default) that the output does not hold: {}",
        class.len(),
        class.join(" ")
      )
    );
  }
  Ok(())
}

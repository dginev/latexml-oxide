//! BibTeX.pool.ltxml — bibliographic-entry processing for LaTeXML.
//!
//! Perl: `LaTeXML/blib/lib/LaTeXML/Engine/BibTeX.pool.ltxml`. Loaded
//! via `LoadPool('BibTeX')` (e.g. from `amsrefs.sty.ltxml`) or as a
//! preload when the conversion mode is BibTeX
//! (`Common/Config.pm:406`: `unshift(... 'BibTeX.pool')`).
//!
//! ## Status (2026-05-15): Phase 1 foundation
//!
//! Per [`docs/archive/BIBTEX_PORT_PLAN_2026-06-20.md`](../../../docs/archive/BIBTEX_PORT_PLAN_2026-06-20.md),
//! the full port is split into 6 phases. This file is at **Phase 1**:
//! the `BibEntry` data structure + current-entry tracking + field
//! accessors are in place; no Def\* bindings yet.
//!
//! The Perl pool stores entries in the State Value table under keys
//! like `BIBENTRY@<normalized-key>`. Rust uses a thread-local
//! registry instead to avoid threading a custom `Stored::BibEntry`
//! variant through dump_writer / dump_reader / Stored::PartialEq.
//! (BibEntries don't round-trip through dumps anyway — they're
//! created and consumed entirely within a single conversion.)
//!
//! Bind‐wise still TODO (Phases 2-6):
//! - `\bib` / `\bibitem` family (Perl ~L80-200)
//! - Bib entry-type constructors (`@article`, `@book`, ... ~L220-500)
//! - Field handlers (`bib@field@*` family ~60 macros)
//! - BibTeX special-character handling (~L800-956)
//! - `bibAddToContainer` / `processBibNameList` (need Document API integration)

use std::{cell::RefCell, rc::Rc};

use latexml_core::tokens::Tokens;

use crate::prelude::*;

// Note: do NOT add `use std::collections::HashMap` here. The
// `DefConstructor!(... literal-template)` proc-macro expansion
// references an unqualified `HashMap` and expects the
// `rustc_hash::FxHashMap as HashMap` re-export from `crate::prelude::*`
// — bringing std's HashMap into scope shadows it and breaks the
// literal-template flavor of constructors.

/// One BibTeX-style bibliography entry. Mirrors Perl's per-entry hash
/// object (`LaTeXML/Engine/BibTeX.pool.ltxml` uses
/// `$entry->getField(...)`, `$entry->addField(...)` etc.).
///
/// `fields` are *processed* token values (the post-digestion form).
/// `raw_fields` are the *raw* verbatim source strings as they
/// appeared in the input (used by BibTeX-flavoured re-rendering in
/// `\bib@@title`'s case-folding and the `\bib@@origbibentry`
/// roundtrip).
#[derive(Debug, Clone, Default)]
pub struct BibEntry {
  pub key:        String,
  pub entry_type: String,
  fields:         Vec<(String, Tokens)>,
  raw_fields:     Vec<(String, String)>,
}

impl BibEntry {
  pub fn new(key: impl Into<String>, entry_type: impl Into<String>) -> Self {
    Self {
      key: key.into(),
      entry_type: entry_type.into(),
      ..Self::default()
    }
  }

  /// Perl: `$entry->getField($name)` — first match wins (Perl uses
  /// the same first-set-wins semantics; multiple `addField` calls
  /// for the same name are stored in order, and `getField` returns
  /// the first).
  pub fn get_field(&self, name: &str) -> Option<&Tokens> {
    self.fields.iter().find(|(k, _)| k == name).map(|(_, v)| v)
  }

  /// Perl: `$entry->addField($name, $value)`. If the field already
  /// exists, the new value is *appended* — Perl's behaviour for
  /// authors/editors that name-merge across entries. For typical
  /// single-value fields the caller adds at most once.
  pub fn add_field(&mut self, name: impl Into<String>, value: Tokens) {
    self.fields.push((name.into(), value));
  }

  /// Perl: `$entry->getRawField($name)` — raw source string.
  pub fn get_raw_field(&self, name: &str) -> Option<&str> {
    self
      .raw_fields
      .iter()
      .find(|(k, _)| k == name)
      .map(|(_, v)| v.as_str())
  }

  pub fn add_raw_field(&mut self, name: impl Into<String>, value: impl Into<String>) {
    self.raw_fields.push((name.into(), value.into()));
  }

  /// Iterate all field names (in insertion order). Used by
  /// crossref-copy and entry-completion logic.
  pub fn field_names(&self) -> impl Iterator<Item = &str> {
    self.fields.iter().map(|(k, _)| k.as_str())
  }

  /// Render the entry in BibTeX source format. Perl
  /// `LaTeXML::Util::BibTeX::Entry::prettyPrint`. Output shape:
  /// ```text
  /// @article{Smith2020,
  ///   author = {John Smith},
  ///   title = {On Examples},
  ///   year = {2020}
  /// }
  /// ```
  /// Uses raw fields (not digested), since the goal is to capture
  /// the original BibTeX-flavoured source for the
  /// `\bib@@origbibentry` round-trip.
  pub fn pretty_print(&self) -> String {
    let mut out = format!("@{}{{{}", self.entry_type, self.key);
    if self.raw_fields.is_empty() {
      out.push_str("}\n");
      return out;
    }
    for (k, v) in &self.raw_fields {
      // Skip internal `_*` raw fields (e.g. `_raw_keyvals` added by
      // the amsrefs Phase-2 stub).
      if k.starts_with('_') {
        continue;
      }
      // Match Perl's source reconstruction: each field on its own line,
      // the name right-justified so the `=` aligns — `max(1, 10 - len)`
      // leading spaces (≥1 space even for names ≥10 chars). The entry's
      // closing `}` follows the last value directly (no newline before it),
      // and the whole entry ends with one — Perl `Pre/BibTeX/Entry.pm:60-64`
      // joins the lines and appends `"}\n"`.
      let lead = 10usize.saturating_sub(k.len()).max(1);
      out.push_str(",\n");
      out.push_str(&" ".repeat(lead));
      out.push_str(k);
      out.push_str(" = {");
      out.push_str(v);
      out.push('}');
    }
    out.push_str("}\n");
    out
  }
}

thread_local! {
  /// Map from normalized bibkey (Perl: `NormalizeBibKey(<raw-key>)`)
  /// to the registered entry. Populated by `\bib`'s entry-create
  /// path (Phase 4); read by `current_entry`-based field helpers.
  static BIB_ENTRIES: RefCell<HashMap<String, Rc<RefCell<BibEntry>>>> =
    RefCell::new(HashMap::default());

  /// The normalized key of the entry currently being processed.
  /// Set by the per-entry pipeline at `\bib@entry@<type>@prepare`
  /// time; cleared after `\bib@entry@<type>@complete`.
  static CURRENT_ENTRY_KEY: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Register a fresh `BibEntry` under its normalized key, and set it
/// as the current entry. Returns an `Rc<RefCell<...>>` so callers
/// can hold a handle while the registry retains its copy.
///
/// Perl roughly: `AssignValue('BIBENTRY@'.NormalizeBibKey($key),
/// $entry, 'global'); AssignValue('current_bib_entry', $entry,...)`.
pub fn register_entry(key: &str, entry: BibEntry) -> Rc<RefCell<BibEntry>> {
  use latexml_core::common::cleaners::normalize_bib_key;
  let normkey = normalize_bib_key(key);
  let cell = Rc::new(RefCell::new(entry));
  BIB_ENTRIES.with(|m| {
    m.borrow_mut().insert(normkey.clone(), cell.clone());
  });
  CURRENT_ENTRY_KEY.with(|k| *k.borrow_mut() = Some(normkey));
  cell
}

/// Switch the "current" pointer to a previously-registered entry.
/// Returns `false` if `key` isn't registered.
pub fn set_current_entry(key: &str) -> bool {
  use latexml_core::common::cleaners::normalize_bib_key;
  let normkey = normalize_bib_key(key);
  let exists = BIB_ENTRIES.with(|m| m.borrow().contains_key(&normkey));
  if exists {
    CURRENT_ENTRY_KEY.with(|k| *k.borrow_mut() = Some(normkey));
  }
  exists
}

/// Clear the "current" pointer. Called at entry-complete time so a
/// stray `currentBibEntry()` outside a `\bib{...}` block returns
/// `None` instead of leaking the previous entry.
pub fn clear_current_entry() { CURRENT_ENTRY_KEY.with(|k| *k.borrow_mut() = None); }

/// Perl: `currentBibEntry()` — return a handle to the entry being
/// processed, or `None` if not inside a `\bib{...}` block.
pub fn current_entry() -> Option<Rc<RefCell<BibEntry>>> {
  CURRENT_ENTRY_KEY.with(|k| {
    let key = k.borrow();
    let key_ref = key.as_deref()?;
    BIB_ENTRIES.with(|m| m.borrow().get(key_ref).cloned())
  })
}

/// Perl `currentBibKey()` (`BibTeX.pool.ltxml:192`) — return the
/// normalized key of the entry currently being processed, or `None`
/// if not inside a `\bib{...}` block.
///
/// Divergence B1 (see `docs/archive/BIBTEX_PORT_PLAN_2026-06-20.md`): Perl stores this
/// as a State Value `CURRENT@BIBKEY` (group-scoped via Perl
/// `\bgroup`/`\egroup`); Rust stores it as a thread-local, which
/// does NOT auto-pop on group exit. The current Phase 1-3 code never
/// nests `\bib{...}` calls so this divergence is latent; will need
/// to be revisited when Phase 4's `\bibentry@prepare` DefPrimitive
/// ports the `$stomach->bgroup; AssignValue; ...; $stomach->egroup`
/// dance from Perl L126-132.
pub fn current_bib_key() -> Option<String> { CURRENT_ENTRY_KEY.with(|k| k.borrow().clone()) }

/// Look up a *registered* entry by raw (un-normalized) key. Perl
/// equivalent: `LookupValue('BIBENTRY@'.NormalizeBibKey($k))`. Used
/// by `\bib@@field`'s crossref path (`\bib@entry@default@prepare`
/// pulls listed fields from the crossref'd parent).
pub fn lookup_entry(key: &str) -> Option<Rc<RefCell<BibEntry>>> {
  use latexml_core::common::cleaners::normalize_bib_key;
  let normkey = normalize_bib_key(key);
  BIB_ENTRIES.with(|m| m.borrow().get(&normkey).cloned())
}

/// Perl: `currentBibEntryField('fieldname')` — get the *processed*
/// value of a field on the current entry.
///
/// Falls back to the raw source field, and that fallback is what makes this
/// function work at all: Perl's `Pre::BibTeX::Entry` is built with BOTH lists
/// populated (for amsrefs, `new(…, [@fields], [@fields])` passes the *same*
/// list twice), whereas here [`BibEntry::add_field`] is only ever called by
/// [`copy_crossref_fields`] — so outside a `crossref` the `fields` store is
/// EMPTY and an unguarded `get_field` returned `None` for every field that
/// exists. That silently disabled every caller which asks Perl's question:
/// `\bib@synthesize@mr`/`@zbl` never emitted a MathReview/ZentralBlatt link,
/// and `\bib@field@default@year`'s "is `date` already set?" guard never fired,
/// so an entry with both emitted a duplicate `<ltx:bib-date>`.
///
/// Perl's `getField` yields the field's STRING; callers that need the source
/// text verbatim (e.g. `\bib@field@unknownasdata`) should therefore prefer
/// [`current_entry_raw_field`] — tokenizing here is lossy in one direction, in
/// Text for an `ltx:bib-extract` field, read from the entry's stored field
/// rather than from the macro argument.
///
/// Perl's own idiom for a `Verbatim`-read field — `\bib@field@default@default
/// Verbatim Verbatim` -> `\bib@field@unknownasdata{#1}`, whose comment reads
/// "IGNORE the tokenized data" (`BibTeX.pool.ltxml` L346) — is to consume the
/// value verbatim so it cannot be interpreted, then recover it from
/// `currentBibEntryField` at construction time. The verbatim token form is a
/// dead end: it does not survive into a macro expansion body (measured — `#1`
/// substitutes to nothing, leaving `\bib@@field`'s `Digested` slot empty, which
/// opens `ltx:bib-extract` and never closes it: 203 errors on 2605.00184 as
/// every later field nests inside).
///
/// Raw-first, same as [`bib@field@unknownasdata`](install_bibtex_definitions)
/// and for the same reason: a Tokens round-trip eats the space that terminates
/// a control word.
fn bib_extract_text(field: &str) -> String {
  current_entry_raw_field(field)
    .or_else(|| current_entry_field(field).map(|t| t.to_string()))
    .unwrap_or_default()
}

/// The four TeX specials that a `.bib` field can only have meant literally.
///
/// Not `~` (a tie is plausible in a name) and not `^` (it only ever shows up in
/// math here) — this is the maintainer-authorized set, no wider.
const BIB_DATA_SPECIALS: [char; 4] = ['_', '&', '#', '%'];

/// The `.bib`-data -> TeX-source boundary: escape the specials a `.bst` would
/// have escaped, so the synthesized entry line is valid TeX.
///
/// Real BibTeX has TWO regimes and latexml-oxide collapses them into one pass.
/// In the DATA regime (`.bib` read by `bibtex(1)`) a field's bytes are data:
/// `%` is not a comment, `&` is not an alignment tab, `_` is not a subscript,
/// `#` is not a parameter. Only in the TeX regime (the `.bbl` read by
/// `pdflatex`) do those catcodes exist — and what lands in the `.bbl` is
/// whatever the `.bst` chose to write. We read `.bib` directly, with no `.bst`
/// in the loop, so we are the component that decides what reaches the
/// tokenizer, and the author's intent for a bare `&` in `AT&T` is plainly the
/// character. Escaping here — emitting what a careful `.bst` author would have
/// written — is "be bibtex first, then be pdflatex on the `.bbl` you just
/// synthesized". OXIDIZED_DESIGN #74.
///
/// Escaping at the boundary rather than neutralizing catcodes during digestion
/// is what keeps the TeX regime intact **by construction**: `\emph{…}`,
/// `{\v S}pakov` and `$x_1+x_2$` are already valid TeX and simply pass through.
/// Two hazards, both guarded by
/// `06_cluster_bibliography::bib_field_specials_are_data_not_tex`:
///
/// * **Math.** `_` inside `$…$` is a real subscript, so math spans are skipped.
///   `$`/`$$` toggle; `\(`/`\[` open and `\)`/`\]` close.
/// * **Idempotency.** Most `.bib` files already write `\&`, `\%`, `\_`. A
///   backslash therefore consumes the next character as a pair and neither is
///   re-examined, so `\&` stays `\&`. The tricky case falls out of the same
///   rule: in `\\&` the `\\` is consumed as one pair, leaving a genuinely bare
///   `&` that IS escaped.
fn escape_bib_data_specials(value: &str) -> String {
  let mut out = String::with_capacity(value.len() + 8);
  let mut chars = value.chars().peekable();
  let mut in_math = false;
  while let Some(c) = chars.next() {
    match c {
      '\\' => {
        out.push('\\');
        match chars.peek() {
          // A control WORD. Copy it whole (the terminating space is emitted by
          // the next iteration, so `\ndash 693` keeps its space), then — if it
          // reads its own argument verbatim — copy that argument untouched too.
          Some(n) if n.is_alphabetic() => {
            let mut word = String::new();
            while let Some(&n) = chars.peek() {
              if !n.is_alphabetic() {
                break;
              }
              word.push(n);
              chars.next();
            }
            out.push_str(&word);
            if VERBATIM_ARG_COMMANDS.contains(&word.as_str()) {
              copy_balanced_group(&mut chars, &mut out);
            }
          },
          // A control SYMBOL. Copy the pair. This one arm is the whole of the
          // idempotency rule (`\&` stays `\&`) AND of "leave `\` alone" — and
          // it is what makes the tricky `\\&` case fall out for free: `\\` is
          // consumed here as one pair, leaving a genuinely bare `&` to escape.
          Some(_) => {
            let n = chars.next().unwrap_or('\\');
            out.push(n);
            match n {
              '(' | '[' => in_math = true,
              ')' | ']' => in_math = false,
              _ => {},
            }
          },
          None => {},
        }
      },
      '$' => {
        out.push('$');
        // `$$` is one display delimiter, not two toggles.
        if chars.peek() == Some(&'$') {
          chars.next();
          out.push('$');
        }
        in_math = !in_math;
      },
      _ if !in_math && BIB_DATA_SPECIALS.contains(&c) => {
        out.push('\\');
        out.push(c);
      },
      _ => out.push(c),
    }
  }
  out
}

/// Control words that read their own next argument as DATA, so the escaper must
/// leave that argument alone.
///
/// url.sty's `\url`/`\nolinkurl`/`\path` take a Verbatim argument and `\href`
/// takes a Semiverbatim URL — a percent-encoded `%20` inside one is not a
/// comment and must not become `\%`, which would land a literal backslash in
/// the href. `howpublished = {\url{http://x.org/a%20b}}` is the measured case
/// (`docs/parity/BIBLIOGRAPHY_WORKLIST.md` records it as a working probe, and
/// blind escaping regressed it). Only ONE group is skipped, so `\href`'s second
/// argument — real prose — is still escaped.
const VERBATIM_ARG_COMMANDS: [&str; 5] = ["url", "nolinkurl", "path", "href", "doi"];

/// Copy one balanced `{...}` group through untouched, leading spaces included.
/// A no-op when the next non-space character is not `{`.
fn copy_balanced_group(chars: &mut std::iter::Peekable<std::str::Chars>, out: &mut String) {
  let mut pending = String::new();
  while let Some(&c) = chars.peek() {
    if c == ' ' {
      pending.push(c);
      chars.next();
    } else {
      break;
    }
  }
  if chars.peek() != Some(&'{') {
    out.push_str(&pending);
    return;
  }
  out.push_str(&pending);
  out.push('{');
  chars.next();
  let mut depth = 1usize;
  for c in chars.by_ref() {
    out.push(c);
    match c {
      '{' => depth += 1,
      '}' => {
        depth -= 1;
        if depth == 0 {
          break;
        }
      },
      _ => {},
    }
  }
}

/// The field value to write into the synthesized entry line for `handler`.
///
/// A handler that declares a `Verbatim`/`Semiverbatim` first parameter reads the
/// field's characters itself, under its own catcode table, and hands them on as
/// a string — a `url` href, a `doi` id. Escaping those would put a literal
/// backslash in the value, so they are passed through raw. Perl declares
/// exactly which fields those are (`BibTeX.pool.ltxml` L740 `url` **Verbatim**,
/// L684/L750-783 `crossref`/`doi`/`isbn`/`issn`/`lccn`/`pii` **Semiverbatim**),
/// and this reads that declaration back off the definition instead of keeping a
/// second copy of the list that could drift from it.
fn bib_field_source(value: &str, handler: &Rc<dyn Definition>) -> String {
  let reads_raw = handler
    .get_parameters()
    .and_then(|params| params.get_parameters().first().map(|p| reads_field_raw(p)))
    .unwrap_or(false);
  if reads_raw {
    value.to_string()
  } else {
    escape_bib_data_specials(value)
  }
}

/// Whether `p` consumes its argument's characters as data rather than as TeX.
///
/// Both halves are needed, and the asymmetry is a trap worth naming: only
/// `Semiverbatim` sets the `semiverbatim` DESCRIPTOR field; `Verbatim` calls
/// `begin_semiverbatim(Some(&['%', '\\']))` inside its reader closure instead
/// and leaves the descriptor empty (`base_parameter_types.rs`). Testing the
/// descriptor alone therefore silently misses `url` — measured, it put a
/// literal `\%` in a `url` href. `Parameter::name` is the declared type name
/// (it is explicitly preserved when a descriptor is merged in), so it is the
/// reliable half for `Verbatim`.
fn reads_field_raw(p: &Parameter) -> bool {
  p.semiverbatim.is_some()
    || with(p.name, |name| {
      matches!(name, "Verbatim" | "Semiverbatim" | "OptionalSemiverbatim")
    })
}

/// that a control word's terminating space is consumed (`\ndash 693`).
pub fn current_entry_field(name: &str) -> Option<Tokens> {
  let entry = current_entry()?;
  let entry = entry.borrow();
  if let Some(tokens) = entry.get_field(name) {
    return Some(tokens.clone());
  }
  // `assembled`: a *raw* field is the .bib source text verbatim, straight off
  // the BibTeX lexer — never a flattened `Tokens`, so no control word can have
  // been welded to what follows it.
  entry
    .get_raw_field(name)
    .map(|raw| tokenize_bib_field(TeXString::assembled(raw.to_string())))
}

/// `Tokenize!` for a string that came out of the BibTeX lexer.
///
/// The standard catcode table, as Perl's `Tokenize` uses — except that `%`, `&`
/// and `#` are ordinary characters. BibTeX has no comment syntax inside an entry
/// (`Pre::BibTeX` only skips `%` in the junk BETWEEN entries), no alignment and
/// no parameters, so all three are data; under catcode 14 a `%` comments out the
/// rest of the string and takes any brace still open with it, under catcode 4 an
/// `&` is a stray alignment tab, and under catcode 6 a `#` reaches the Stomach as
/// a parameter. Witnesses 2605.02131 (a percent-encoded URL in a `title`'s
/// `\href`, 24 `malformed:ltx:bibentry`) and 2605.01936
/// (`publisher = {Taylor & Francis}` and six more, 7 `unexpected:&`).
///
/// **Treatment 1 of `OXIDIZED_DESIGN #74`** — the same rule the per-entry Mouth
/// applies (see `\ProcessBibTeXEntry` below); it has to be repeated here because
/// the handlers that re-read a raw field — `\bib@@title` recasing, name
/// splitting, date/pages/MR/Zbl assembly — build their tokens from the stored
/// string and never go through that mouth.
///
/// `_` is NOT in this set: a catcode is fixed at tokenization, before anything
/// knows whether it is inside `$…$`, and a subscript in a title's math is
/// legitimate TeX. It is handled by treatment 2 ([`bib_tex_tokens`]) instead.
fn tokenize_bib_field(text: impl Into<TeXString>) -> Tokens { mouth::tokenize_bib_literal(text) }

/// Both treatments, for a raw `.bib` string that is about to become TeX tokens.
///
/// Treatment 2 (`escape_bib_data_specials`) makes the data valid TeX — it is the
/// only one of the two that can be math-aware, so it is what covers `_`; then
/// treatment 1 ([`tokenize_bib_field`]) reads the result with `% & #` still inert,
/// covering whatever the escaper deliberately left alone (the inside of a
/// `\url{…}`).
///
/// Every site that re-reads a RAW field and hands it to the tokenizer uses this:
/// `\bib@@title`, `\bib@@pages`, name splitting, date assembly, MR/Zbl. They are
/// listed explicitly in `BIBLIOGRAPHY_WORKLIST.md` — "any change here must cover
/// all three, plus the name-, date- and MR/Zbl-assembly sites that share that
/// path". Escaping is idempotent, so a value already escaped upstream (a recased
/// title) is unharmed by passing through again.
///
/// The `impl Into<TeXString>` is the guard, and this is the site that has needed
/// it: a caller must hand over either a raw `.bib` string (`TeXString::assembled`
/// — lexer output, never a flattened `Tokens`) or a reverted token list
/// ([`Tokens::untex_string`]). A bare `Tokens::to_string()` no longer compiles
/// here, which is what made `MRREVIEWER = {Fran\c cois\ Digne}` come back as
/// `undefined:\ccois`. See [`latexml_core::tokens::TeXString`].
fn bib_tex_tokens(text: impl Into<TeXString>) -> Tokens {
  // `escape_bib_data_specials` is a TeX-preserving transform of TeX-valid input,
  // so re-asserting `assembled` over its output carries the caller's assertion
  // through rather than laundering a new one.
  let text = text.into();
  tokenize_bib_field(TeXString::assembled(escape_bib_data_specials(
    text.as_str(),
  )))
}

/// Collapse an HTML-escaped ampersand — `&amp;` — back to the single `&` the
/// author wrote, in whichever of its three spellings a `.bib` export used.
///
/// This is a DOUBLE escape, not a TeX construct: a reference manager rendered
/// the field to HTML (`&` -> `&amp;`), then a second pass TeX-escaped the
/// ampersand of that entity, so the file carries `\&amp;` / `{\&}amp;` /
/// `&amp;` where the source said `&`. TeX has no idea: `\&` produces the glyph
/// and `amp;` is four more ordinary characters, so the entry renders
/// "Computer Engineering, &amp; Applied Computing". pdflatex prints exactly the
/// same thing — this is not a parity gap in either direction, it is an input
/// corruption that only the `.bib` reader is positioned to undo, and we read
/// `.bib` directly (see OXIDIZED_DESIGN #73's framing of that position).
///
/// Distinct from a BARE `&`, which is catcode 4 and is neutralized at the mouth
/// (`tokenize_bib_field` above, `#75`). Decoding here deliberately yields a
/// plain `&` and lets that mechanism give it its catcode, rather than escaping
/// it to `\&` behind the mechanism's back.
///
/// Measured over 6000 arXiv/2605 sources: `\&amp;` in `booktitle` (2605.00833,
/// 2605.01362), in `journal` (2605.00922, 2605.01200, 2605.01224), in `title`
/// (2605.01353 `{{A}}\&amp;{{AS}}`); `{\&}amp;` in `title` (2605.01224 "Shake
/// {\&}amp; Bake"); bare `&amp;` in `publisher` (2605.00859 "European
/// Association of Geoscientists &amp; Engineers") and `journal` (2605.01187).
///
/// Only `amp;` DIRECTLY after an ampersand is touched, so a field that merely
/// contains the letters "amp" is untouched, and a lone `&` is left exactly as
/// it was.
fn undouble_escaped_ampersand(value: &str) -> String {
  if !value.contains("amp;") {
    return value.to_string();
  }
  let mut out = String::with_capacity(value.len());
  let mut rest = value;
  while let Some(i) = rest.find("amp;") {
    let (head, tail) = rest.split_at(i);
    out.push_str(head);
    // `{\&}` first: its last character is `}`, not `&`.
    if !(head.ends_with("{\\&}") || head.ends_with('&')) {
      out.push_str("amp;");
    }
    rest = &tail["amp;".len()..];
  }
  out.push_str(rest);
  out
}

/// Perl: `currentBibEntryRawField('fieldname')` — get the *raw*
/// source string of a field on the current entry.
pub fn current_entry_raw_field(name: &str) -> Option<String> {
  current_entry()?
    .borrow()
    .get_raw_field(name)
    .map(str::to_string)
}

/// Perl: `copyCrossrefFields(@fields)` — for each named field, if
/// the current entry doesn't already have it but its crossref'd
/// parent does, copy the value over (processed and raw paths). The
/// crossref field itself is read from `current_entry().get_field`.
pub fn copy_crossref_fields(fields: &[&str]) {
  let Some(current) = current_entry() else {
    return;
  };
  // Get the crossref target's raw key; if no crossref, nothing to do.
  let xref_key = current
    .borrow()
    .get_raw_field("crossref")
    .map(str::to_string);
  let Some(xref_key) = xref_key else {
    return;
  };
  let Some(parent) = lookup_entry(&xref_key) else {
    return;
  };
  // Self-crossref is a paper bug; skip silently to avoid infinite
  // looping if a user writes `crossref={selfkey}`.
  if Rc::ptr_eq(&current, &parent) {
    return;
  }
  let parent_b = parent.borrow();
  let mut current_b = current.borrow_mut();
  for field in fields {
    if current_b.get_field(field).is_none()
      && let Some(v) = parent_b.get_field(field)
    {
      current_b.add_field(*field, v.clone());
    }
    if current_b.get_raw_field(field).is_none()
      && let Some(v) = parent_b.get_raw_field(field)
    {
      current_b.add_raw_field(*field, v.to_string());
    }
  }
}

/// Reset all bibtex thread-local state. Used between conversions so
/// one paper's entries don't leak into the next. Future hook for
/// `Converter::reset_session` integration.
pub fn reset() {
  BIB_ENTRIES.with(|m| m.borrow_mut().clear());
  CURRENT_ENTRY_KEY.with(|k| *k.borrow_mut() = None);
}

/// FxHashMap alias used by `bib_add_to_container` — matches
/// `latexml_core::document::Document::insert_element`'s expected
/// attribute-map type.
type FxAttrMap = rustc_hash::FxHashMap<String, String>;

/// Build the find-or-create XPath used by `bib_add_to_container`.
/// Perl `BibTeX.pool.ltxml:248-250`: `$tag[@k1='v1' and @k2='v2']`
/// with the attribute keys sorted so identical attr sets always
/// produce identical xpaths (cache hit on the second call).
fn bib_container_xpath(tag: &str, attrs: &FxAttrMap) -> String {
  if attrs.is_empty() {
    return tag.to_string();
  }
  let mut keys: Vec<&String> = attrs.keys().collect();
  keys.sort();
  let preds: Vec<String> = keys
    .iter()
    .map(|k| format!("@{}='{}'", k, attrs.get(*k).unwrap()))
    .collect();
  format!("{tag}[{}]", preds.join(" and "))
}

/// Perl `bibAddToContainer($document, $tag, $data, %attr)` —
/// `BibTeX.pool.ltxml:242-257`. Find-or-create a child of the
/// current `<ltx:bibentry>` / `<ltx:bib-related>` ancestor with
/// matching tag+attrs. If found, absorb `data` into it; if not,
/// insert a fresh element with `data` as content.
///
/// Used by `\bib@addto@related` (deduplicates same-(type,role)
/// bib-related groups, e.g. all authors collected in one
/// `<ltx:bib-related role='authors'>`).
pub fn bib_add_to_container(
  doc: &mut Document,
  tag: &str,
  data: Option<&Digested>,
  attrs: FxAttrMap,
) -> Result<()> {
  let current = doc.get_node().clone();
  let entry = doc.findnode(
    "ancestor-or-self::ltx:bibentry | ancestor-or-self::ltx:bib-related",
    Some(&current),
  );
  let xpath = bib_container_xpath(tag, &attrs);
  match doc.findnode(&xpath, entry.as_ref()) {
    Some(rel) => {
      doc.set_node(&rel);
      if let Some(d) = data {
        doc.absorb(d, None)?;
      }
      doc.set_node(&current);
    },
    _ => {
      let content: Vec<&Digested> = match data {
        Some(d) => vec![d],
        None => vec![],
      };
      doc.insert_element(tag, content, Some(attrs))?;
    },
  }
  Ok(())
}

/// A single parsed BibTeX-style author/editor name.
///
/// Perl returns this implicitly inside `Invocation(T_CS('\bib@surname'),
/// Tokenize($surname))` etc. We expose the parsed triple so later
/// phases can wrap it however they need (Tokens, XML, post-processed
/// metadata) — splitting concerns at the parser/output boundary.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BibName {
  pub given:   String,
  pub surname: String,
  /// "Jr.", "Sr.", "III" — Perl `$jr`, `\bib@lineage` in TeX output.
  pub lineage: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BibNameList {
  pub names: Vec<BibName>,
  /// True if the input ended with `and others` / `and et al.` —
  /// Perl's `$etal` flag, post-processed to append a final
  /// `\bib@surname{others}` invocation.
  pub etal:  bool,
}

/// Perl `splitWords` (`BibTeX.pool.ltxml` L921-949). Split a name
/// string into words on whitespace / comma / tilde, but treat
/// `{balanced}` groups as a single word. Commas survive as their
/// own tokens. `\~` is preserved (Perl protects it via the
/// `####` placeholder trick — we use a sentinel `\x00` byte
/// which can't appear in TeX source).
fn split_words(input: &str) -> Vec<String> {
  // 1. Protect `\~`, normalise leading whitespace + `%\n` line continuations.
  const PLACEHOLDER: &str = "\x00";
  let s = input.replace("\\~", PLACEHOLDER);
  let s = s.trim_start_matches(|c: char| c.is_whitespace() || c == '~');
  let s = s.replace("%\n", "");

  let mut words: Vec<String> = Vec::new();
  let mut word = String::new();
  // Scanned by character, not by byte index — see `CharCursor`'s module docs.
  // Author names are the most reliably non-ASCII field in a `.bib`.
  let mut cur = CharCursor::new(&s);
  const SEP: [char; 5] = [' ', '\t', '\n', '\r', '~'];
  while let Some(c) = cur.peek() {
    // Check for `(comma?) whitespace+` separators. Perl regex:
    // s/^(,?)[\s~]+//
    if c == ',' || SEP.contains(&c) {
      let had_comma = c == ',';
      cur.next();
      // Either a leading comma we just consumed, or a pure whitespace/tilde
      // run — either way, absorb the trailing whitespace.
      cur.take_while(|c| SEP.contains(&c));
      // Flush accumulated word
      if !word.is_empty() {
        words.push(std::mem::take(&mut word));
      }
      if had_comma {
        words.push(",".to_string());
      }
    } else if c == '{' {
      // Extract balanced group; include the braces.
      let start = cur.pos();
      let mut depth = 0i32;
      while let Some(c) = cur.next() {
        match c {
          '{' => depth += 1,
          '}' => {
            depth -= 1;
            if depth == 0 {
              break;
            }
          },
          _ => {},
        }
      }
      // Append the entire braced chunk (including the braces) to the
      // current word so it stays atomic across word splits — matches
      // Perl `$word .= $t`.
      word.push_str(cur.slice_from(start));
    } else {
      // Greedily accumulate until the next separator / `{`.
      let start = cur.pos();
      cur.take_while(|c| !SEP.contains(&c) && c != ',' && c != '{');
      word.push_str(cur.slice_from(start));
    }
  }
  if !word.is_empty() {
    words.push(word);
  }
  // 6. Restore `\~`.
  for w in words.iter_mut() {
    if w.contains(PLACEHOLDER) {
      *w = w.replace(PLACEHOLDER, "\\~");
    }
  }
  words
}

/// Perl `processBibNameList` (`BibTeX.pool.ltxml` L872-918): parse a
/// BibTeX-style author/editor list into structured names. The input
/// is a raw string (Perl uses `UnTeX($names, 1)` to flatten Tokens
/// first). Returns the parsed names plus an `etal` flag.
///
/// Name shapes recognised (BibTeX `bibtex.web` convention):
///   - `First Last` — given before surname; the surname starts at the first lowercase word (e.g.
///     `von`, `de la`) or, in the no-lowercase case, at the LAST word.
///   - `Last, First` — surname first; one comma.
///   - `Last, Jr., First` — surname + lineage + given; two commas.
///
/// Multiple names separated by ` and ` (case-insensitive). A
/// trailing `and others` / `and et al.` sets `etal = true`.
pub fn process_bib_name_list(input: &str) -> BibNameList {
  let mut words = split_words(input);
  let mut etal = false;
  // Detect trailing `and others` / `and et al(.)?` and strip the
  // last two words if present. Perl L878.
  if words.len() >= 2 {
    let last = &words[words.len() - 1];
    let prev = &words[words.len() - 2];
    if prev.eq_ignore_ascii_case("and") {
      let last_lc = last.to_ascii_lowercase();
      let last_lc = last_lc.trim_end_matches('.');
      if matches!(last_lc, "others" | "et al" | "etal") {
        words.pop();
        words.pop();
        etal = true;
      }
    }
  }
  let mut names: Vec<BibName> = Vec::new();
  while !words.is_empty() {
    // Collect words for one name, splitting comma-delimited phrases.
    let mut phrases: Vec<Vec<String>> = Vec::new();
    let mut phrase: Vec<String> = Vec::new();
    while !words.is_empty() {
      let word = words.remove(0);
      if word.eq_ignore_ascii_case("and") {
        break;
      }
      if word == "," {
        phrases.push(std::mem::take(&mut phrase));
      } else {
        phrase.push(word);
      }
    }
    if phrase.is_empty() && phrases.is_empty() {
      // Empty name (consecutive `and`s, or stray comma+and). Perl
      // emits Warn; we silently skip — the caller's Tokens output
      // would be a no-op anyway.
      continue;
    }
    if !phrase.is_empty() {
      phrases.push(phrase);
    }

    let (given, surname, lineage) = match phrases.len() {
      3 => {
        // "von Last, Jr, First"
        let surname = phrases[0].join(" ");
        let lineage = phrases[1].join(" ");
        let given = phrases[2].join(" ");
        (given, surname, lineage)
      },
      2 => {
        // "von Last, First"
        let surname = phrases[0].join(" ");
        let given = phrases[1].join(" ");
        (given, surname, String::new())
      },
      _ => {
        // "First [von] Last" — words before the FIRST lowercase
        // word are given; the rest are surname. If no lowercase
        // word, the LAST word is surname and the rest are given.
        let pwords = &phrases[0];
        let mut first: Vec<String> = Vec::new();
        let mut rest: Vec<String> = pwords.clone();
        while !rest.is_empty() && !starts_lowercase(&rest[0]) {
          first.push(rest.remove(0));
        }
        if rest.is_empty() && !first.is_empty() {
          // No lowercase word — move the last `first` word into rest.
          rest.push(first.pop().unwrap());
        }
        let given = first.join(" ");
        let surname = rest.join(" ");
        (given, surname, String::new())
      },
    };
    names.push(BibName { given, surname, lineage });
  }
  BibNameList { names, etal }
}

/// Helper for `process_bib_name_list`: does a word start with a
/// lowercase letter? Perl test: `$word !~ /^[a-z]/` (NOT lowercase).
/// We invert that. Word may have a leading `{...}` group — peek past
/// it to find the first actual letter. Words starting with a control
/// sequence like `\von` are treated as lowercase by inspecting the
/// CS name's first letter (matches Perl behaviour for `\von Last`
/// where the regex sees `\` which IS lowercase via the
/// `[a-z]` heuristic — Perl's comment "This case test is not
/// correct!!!" acknowledges this is approximate).
fn starts_lowercase(word: &str) -> bool {
  // Strip a single leading `{...}` envelope (BibTeX convention for
  // forcing capitalisation: `{Smith}` should be opaque).
  let stripped = if let Some(rest) = word.strip_prefix('{') {
    rest.trim_end_matches('}')
  } else {
    word
  };
  // Skip a leading `\` (control sequence) — peek at the next char.
  let stripped = stripped.strip_prefix('\\').unwrap_or(stripped);
  stripped.chars().next().is_some_and(|c| c.is_lowercase())
}

/// BibTeX title-case modes recognised by `recase_title`. Perl
/// `BibTeX.pool.ltxml:283-291`. Default is `Capitalize1` per the
/// pool's `LookupValue('BibTeX_title_case') || 'capitalize1'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleCaseMode {
  /// Leave the title alone.
  AsIs,
  /// Downcase everything, then capitalise the first word only.
  Capitalize1,
  /// Downcase everything, then capitalise each word.
  Capitalize,
  /// All-uppercase.
  Uppercase,
  /// All-lowercase.
  Lowercase,
}

impl TitleCaseMode {
  /// Parse a Perl-style mode string. Unknown values fall back to
  /// `Capitalize1` (matching `LookupValue(...) || 'capitalize1'`).
  pub fn parse(s: &str) -> Self {
    match s {
      "asis" => Self::AsIs,
      "capitalize" => Self::Capitalize,
      "uppercase" => Self::Uppercase,
      "lowercase" => Self::Lowercase,
      _ => Self::Capitalize1,
    }
  }
}

/// Perl `\bib@@title` body (`BibTeX.pool.ltxml:293-332`): re-case a
/// title string while preserving brace-grouped and `$math$`-delimited
/// regions verbatim. Words are runs of `\w` chars or `\<word>`/
/// `\<single-char>` control-sequence escapes; whitespace separates
/// words and may modify the next word's "first" status.
///
/// `wb` (word-beginning) is 1 at the start of a word run; `wc` (word
/// counter) increments at each word start. Together they let the
/// `Capitalize1` mode capitalise the FIRST word and lowercase the
/// rest, while `Capitalize` uppercases every word.
pub fn recase_title(title: &str, mode: TitleCaseMode) -> String {
  // Scanned through `char_indices`, NOT `as_bytes()` + a hand-rolled index.
  //
  // Every index this yields is a char boundary by construction, so the
  // `&title[a..b]` slices below cannot panic — which they previously could:
  // the `\<char>` escape arm advanced a fixed ONE byte past the backslash, so
  // `\“` left the cursor inside the codepoint and the next slice aborted the
  // whole document (a panic is not a recoverable conversion error). Witness
  // 2605.22125, found by the 2026-07-26 sandbox sweep.
  //
  // The class, not just the instance: a byte walker re-admits the bug on every
  // future edit, because `usize` carries no boundary invariant and nothing in
  // the type system objects. Perl cannot have it at all — `\bib@@title`
  // (BibTeX.pool.ltxml L293-333) works in characters. Rust rules
  // `anti-index-over-iter` / `perf-iter-over-index`.
  let mut cur = CharCursor::new(title);
  let mut out = String::with_capacity(title.len());
  let mut wb: bool = true;
  let mut wc: u32 = 0;

  while let Some(c) = cur.peek() {
    // Whitespace run — copy verbatim, set word-beginning.
    if matches!(c, ' ' | '\t' | '\n' | '\r') {
      let start = cur.pos();
      cur.take_while(|c| matches!(c, ' ' | '\t' | '\n' | '\r'));
      out.push_str(cur.slice_from(start));
      wb = true;
      continue;
    }
    // Balanced `{...}` — copy verbatim, atomic word.
    if c == '{' {
      let start = cur.pos();
      let mut depth = 0i32;
      while let Some(c) = cur.next() {
        match c {
          '{' => depth += 1,
          '}' => {
            depth -= 1;
            if depth == 0 {
              break;
            }
          },
          _ => {},
        }
      }
      if wb {
        wc += 1;
      }
      out.push_str(cur.slice_from(start));
      wb = false;
      continue;
    }
    // Balanced `$...$` — copy verbatim, no word-counter bump.
    if c == '$' {
      let start = cur.pos();
      cur.next();
      cur.take_while(|c| c != '$');
      cur.next(); // the closing `$`, if any
      out.push_str(cur.slice_from(start));
      wb = false;
      continue;
    }
    // Word: ASCII alphanumeric/underscore OR `\<word>` / `\<char>` escape.
    let word_start = cur.pos();
    let mut consumed_word = false;
    while let Some(c) = cur.peek() {
      if c.is_ascii_alphanumeric() || c == '_' {
        cur.next();
        consumed_word = true;
        continue;
      }
      if c == '\\' && cur.peek_second().is_some() {
        cur.next(); // the backslash
        // `\<word>`: a run of ASCII letters. `\<char>`: exactly one character,
        // of whatever width — the cursor advances by the CHARACTER, so a
        // multi-byte escape can no longer split.
        if cur.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
          cur.take_while(|c| c.is_ascii_alphabetic());
        } else {
          cur.next();
        }
        consumed_word = true;
        continue;
      }
      break;
    }
    if consumed_word {
      let word = cur.slice_from(word_start);
      let recased = match mode {
        TitleCaseMode::AsIs => word.to_string(),
        TitleCaseMode::Uppercase => word.to_uppercase(),
        _ if !wb
          || (mode == TitleCaseMode::Capitalize1 && wc > 0)
          || mode == TitleCaseMode::Lowercase =>
        {
          word.to_lowercase()
        },
        TitleCaseMode::Capitalize | TitleCaseMode::Capitalize1 => ucfirst(word),
        _ => word.to_string(),
      };
      out.push_str(&recased);
      if wb {
        wc += 1;
      }
      wb = false;
      continue;
    }
    // Fallback single char (e.g. punctuation).
    let ch_start = cur.pos();
    cur.next();
    out.push_str(cur.slice_from(ch_start));
    wb = true;
  }
  out
}

/// Perl `processIdentifier($string)` (`BibTeX.pool.ltxml:784-789`) —
/// trim leading and trailing whitespace from a stringified identifier.
/// Used by the doi/isbn/issn/lccn/pii constructors to normalise raw
/// Semiverbatim input before urlencoding or attribute-emitting.
pub fn process_identifier(s: &str) -> String { s.trim().to_string() }

/// Parse an amsrefs-style keyval string `"key = {value}, key = value, ..."`
/// into a list of `(name, value)` pairs. Names are lowercased; values
/// have their outer `{...}` envelope stripped (if any) and surrounding
/// whitespace trimmed.
///
/// Used by the amsrefs `\bib{key}{type}{keyvals}` stub to populate
/// `BibEntry::raw_fields` for downstream `\ProcessBibTeXEntry`
/// dispatch. Mirrors Perl `$keyvals->getPairs` + `lc()` + `UnTeX()`
/// from `amsrefs.sty.ltxml:42-50`.
pub fn parse_amsrefs_keyvals(s: &str) -> Vec<(String, String)> {
  // `CharCursor`, not `as_bytes()` + an index: see its module docs. This walker
  // happens to be safe as written (it only ever STOPS at ASCII delimiters, and
  // an ASCII byte is never a UTF-8 continuation byte), but "happens to be safe"
  // is the state `recase_title` was in until someone added one `i += 1` —
  // witness 2605.22125. Scanning by character removes the possibility rather
  // than relying on every future edit to re-derive the argument.
  let mut cur = CharCursor::new(s);
  let mut out: Vec<(String, String)> = Vec::new();
  while !cur.is_done() {
    // Skip whitespace + commas.
    cur.take_while(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | ','));
    if cur.is_done() {
      break;
    }
    // Read key up to `=` (or end / `,`).
    let key_start = cur.pos();
    cur.take_while(|c| !matches!(c, '=' | ','));
    let key = cur.slice_from(key_start).trim().to_ascii_lowercase();
    if key.is_empty() {
      // Stray comma or trailing garbage; skip the separator and continue.
      cur.next();
      continue;
    }
    if cur.peek() != Some('=') {
      // No `=` — key without value; record empty value.
      out.push((key, String::new()));
      continue;
    }
    cur.next(); // skip `=`
    cur.take_while(|c| matches!(c, ' ' | '\t' | '\n' | '\r'));
    // Value: balanced `{...}` group OR until next top-level `,`.
    let value: String;
    if cur.peek() == Some('{') {
      cur.next(); // the opening brace, excluded from the value
      let start = cur.pos();
      let mut depth = 1i32;
      while let Some(c) = cur.peek() {
        match c {
          '{' => depth += 1,
          '}' => depth -= 1,
          _ => {},
        }
        if depth == 0 {
          break;
        }
        cur.next();
      }
      value = cur.slice_from(start).to_string();
      cur.next(); // skip the closing `}`, if present
    } else {
      let start = cur.pos();
      cur.take_while(|c| c != ',');
      value = cur.slice_from(start).trim().to_string();
    }
    out.push((key, value));
  }
  out
}

/// Perl `ucfirst($s)` — uppercase the first char, leave the rest.
fn ucfirst(s: &str) -> String {
  let mut chars = s.chars();
  match chars.next() {
    Some(c) => {
      let mut out = String::with_capacity(s.len());
      for upper in c.to_uppercase() {
        out.push(upper);
      }
      out.push_str(chars.as_str());
      out
    },
    None => String::new(),
  }
}

LoadDefinitions!({
  // Perl BibTeX.pool.ltxml L19: `LoadPool('LaTeX')` — BibTeX
  // pool is built on top of the full LaTeX format, since bib
  // entries digest LaTeX-flavored markup in titles/authors/etc.
  LoadPool!("LaTeX");

  // -------- Phase 2: core supporters (Perl L230-278, L951-953) --------

  // \bib@@field {} OptionalKeyVals Digested
  // Perl L230-232: insert element with tag, attrs, and content.
  // Tag comes in as digested tokens (e.g. "ltx:bib-title"); attrs is
  // an OptionalKeyVals digested arg (or absent); content is the
  // already-digested body to absorb under the new element.
  DefConstructor!("\\bib@@field {} OptionalKeyVals Digested",
  sub [document, args] {
    let tag = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let attrs = if let Some(kv_d) = &args[1] {
      if let DigestedData::KeyVals(kv) = kv_d.data() {
        kv.get_hash()
      } else {
        FxAttrMap::default()
      }
    } else {
      FxAttrMap::default()
    };
    let content: Vec<&Digested> = match &args[2] {
      Some(d) => vec![d],
      None => vec![],
    };
    document.insert_element(&tag, content, Some(attrs))?;
  });

  // \bib@addtype{}
  // Perl L235-240: emit `\bib@field@default@type{<type>}` only if
  // the current entry has no `type` field set yet. Used by
  // entry-type prepare macros to add a default type after copying
  // crossref fields. Returning an empty Tokens stream is the
  // Perl "do nothing" branch.
  DefMacro!("\\bib@addtype{}", sub[args] {
    if current_entry_field("type").is_some() {
      Ok(Tokens!())
    } else {
      Ok(Invocation!(T_CS!("\\bib@field@default@type"),
        vec![args[0].clone().owned_tokens().unwrap_or_default()]))
    }
  });

  // \bib@addto@related {}{} Digested
  // Perl L261-263: find-or-create `<ltx:bib-related type='#1' role='#2'>`
  // and absorb the digested body into it. Uses bib_add_to_container's
  // sorted-xpath dedup so two `\bib@addto@related{book}{host}{...}`
  // calls accumulate into one bib-related node.
  DefConstructor!("\\bib@addto@related {}{} Digested",
  sub [document, args] {
    let type_s = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let role_s = args[1].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let mut attrs = FxAttrMap::default();
    attrs.insert("type".to_string(), type_s);
    attrs.insert("role".to_string(), role_s);
    let data = args[2].as_ref();
    bib_add_to_container(document, "ltx:bib-related", data, attrs)?;
  });

  // \bib@@@name{}{} → emit `<ltx:bib-name role='#1'>#2</ltx:bib-name>`.
  // Perl L266-267.
  DefConstructor!(
    "\\bib@@@name{}{}",
    "<ltx:bib-name role='#1'>#2</ltx:bib-name>"
  );

  // \bib@@@names{} — wrapper that turns multi-name tokens into one
  // Whatsit. Perl L270.
  DefConstructor!("\\bib@@@names{}", "#1");

  // \bib@@names{}{} — DefMacro that processes a names string and
  // emits a `\bib@@@names{ <name-invocations> }` group. Perl L271-278.
  //
  // The Perl version processes names via `processBibNameList(UnTeX($names))`
  // which returns a list of name Tokens (each Tokens being the
  // surname/given/lineage invocations for one name). We mirror that:
  // parse the names string with `process_bib_name_list`, then for each
  // name emit `Invocation(\bib@@@name, field, name_tokens)`.
  DefMacro!("\\bib@@names{}{}", sub[args] {
    let field_tokens = args[0].clone().owned_tokens().unwrap_or_default();
    // `untex()`, NOT `to_string()` — Perl L277 is `processBibNameList(UnTeX($names, 1))`,
    // and the two are not interchangeable. `Display for Tokens` says so itself:
    // "NOT for creating valid TeX (use revert or UnTeX for that!)". It concatenates
    // token texts, so the SPACE that terminated a control word — consumed by the
    // tokenizer, and therefore absent as data — is never re-emitted: `\v` + `S`
    // comes back as `\vS`, a control sequence that exists in no LaTeX.
    //
    // That silently mangles every space-form accent in an author name — `{\v S}`,
    // `{\c c}`, `{\"a}`, `{\'e}`, i.e. most non-English names — into an undefined
    // macro, rendering `\vSpakov` where Perl renders `Špakov`. Measured on the
    // 2026-07-26 sweep of sandbox-arxiv-2605/2606: ~+2800 error documents per
    // corpus. `\v{S}` (braced argument) was unaffected, which is what made it look
    // like a brace bug rather than a reversion bug.
    // Guard: `bib_name_space_form_accent_survives_reversion`.
    let names_str = args[1]
      .clone()
      .owned_tokens()
      .map(Tokens::untex)
      .unwrap_or_default();
    let parsed = process_bib_name_list(&names_str);

    // Build the `\bib@@@names{ <name-invocations> }` Tokens stream.
    // Each name expands into invocations of `\bib@surname`,
    // `\bib@given`, `\bib@lineage` over `\bib@@@name{field}{...}`.
    //
    // Note: `Invocation!()` internally uses `?` (it calls `build_invocation(...)?`),
    // so the enclosing closure body must propagate Result. The outer
    // `sub[args] { ... }` closure does (via `.into_tokens_result()`),
    // so we can use `?` at this level. We inline name-construction
    // here (rather than a nested `|name| -> Tokens` helper) because
    // `?` only propagates one level up — from a nested closure it
    // would try to early-return from that closure.
    // Perl L912-917 runs the name parts through Tokenize() — full
    // catcode-aware tokenization — so TeX accents in .bib names
    // (`S\"okmen`, `Hu\ssmann`) become live macros and render as
    // o-umlaut / sharp-s. Explode!() here (verbatim catcode-Other
    // chars) leaked the raw `\"okmen` text into the HTML (witness
    // 2605.00223). mouth::tokenize is synchronous, like Explode.
    let mut body: Vec<Token> = Vec::new();
    body.push(T_CS!("\\bib@@@names"));
    body.push(T_BEGIN!());
    // `assembled`: every part below is a slice of `names_str`, which came out of
    // `Tokens::untex` just above — already valid TeX, with the spaces that
    // terminate control words restored. That is exactly the obligation
    // `TeXString::assembled` names.
    let part = |s: &str| TeXString::assembled(s.to_string());
    for name in &parsed.names {
      let mut name_tks: Vec<Token> = Vec::new();
      if !name.surname.is_empty() {
        let inv = Invocation!(T_CS!("\\bib@surname"),
          vec![bib_tex_tokens(part(&name.surname))]);
        name_tks.extend(inv.unlist());
      }
      if !name.given.is_empty() {
        let inv = Invocation!(T_CS!("\\bib@given"),
          vec![bib_tex_tokens(part(&name.given))]);
        name_tks.extend(inv.unlist());
      }
      if !name.lineage.is_empty() {
        let inv = Invocation!(T_CS!("\\bib@lineage"),
          vec![bib_tex_tokens(part(&name.lineage))]);
        name_tks.extend(inv.unlist());
      }
      let inv = Invocation!(T_CS!("\\bib@@@name"),
        vec![field_tokens.clone(), Tokens::new(name_tks)]);
      body.extend(inv.unlist());
    }
    if parsed.etal {
      // Perl L917: trailing `\bib@surname{others}` as etal marker.
      let others = Invocation!(T_CS!("\\bib@surname"),
        vec![Tokenize!("others")]);
      let inv = Invocation!(T_CS!("\\bib@@@name"),
        vec![field_tokens, others]);
      body.extend(inv.unlist());
    }
    body.push(T_END!());
    Ok(Tokens::new(body))
  });

  // Name-component constructors. Perl L951-953. Schema nodes:
  // <ltx:surname>, <ltx:givenname>, <ltx:lineage>.
  DefConstructor!("\\bib@surname{}", "<ltx:surname>#1</ltx:surname>");
  DefConstructor!("\\bib@given{}", "<ltx:givenname>#1</ltx:givenname>");
  DefConstructor!("\\bib@lineage{}", "<ltx:lineage>#1</ltx:lineage>");

  // -------- Phase 3: title-case logic + default field handlers --------

  // \bib@@title{field}{tag}{ignoretitle}
  // Perl L293-333: re-case the named field per the BibTeX_title_case
  // state value, then emit `\bib@@field{tag}{}{<recased>}`. The
  // `ignoretitle` arg is unused (matches Perl — likely vestigial).
  DefMacro!("\\bib@@title{}{}{}", sub[args] {
    let field_name = if args[0].is_some() { args[0].to_string() } else { String::new() };
    let tag_tokens = args[1].clone().owned_tokens().unwrap_or_default();
    // Perl: `LookupValue('BibTeX_title_case') || 'capitalize1'`.
    let mode_str = lookup_value("BibTeX_title_case")
      .and_then(|s| match s {
        Stored::String(sym) => Some(to_string(sym)),
        Stored::Tokens(t) => Some(t.to_string()),
        _ => None,
      })
      .unwrap_or_else(|| "capitalize1".to_string());
    let mode = TitleCaseMode::parse(&mode_str);
    // Both treatments, in the order the real toolchain applies them, because
    // `\bib@@title` re-reads the RAW field rather than the argument the entry
    // line passed it (Perl L294 — that slot is the vestigial `ignoretitle`), so
    // neither the decode nor the escaping in `\ProcessBibTeXEntry` reaches a
    // title. Witnesses: 2605.01353, a `title` reading `{{A}}\&amp;{{AS}}`
    // (decode); `title = {AT&T dataset AT1G01010_v2}` (escape).
    //
    // Escaping runs BEFORE `recase_title`, not after: `recase_title` splits on
    // words and treats a `\…` escape as part of its word, so raw `AT&T` recases
    // to `AT&t` (three words) while raw `AT\&T` recases to `AT&T` (one). Every
    // real `.bib` mixes both spellings and they must render the same string, so
    // the data becomes TeX first and the casing pass sees one consistent input.
    // Guard case: `barespecials` vs `preescaped` in `bib_field_specials.bib`.
    let raw = escape_bib_data_specials(&undouble_escaped_ampersand(
      &current_entry_raw_field(&field_name).unwrap_or_default(),
    ));
    let recased = recase_title(&raw, mode);
    // Emit `\bib@@field{tag}{}{<recased>}`. The empty `{}` slot
    // is the OptionalKeyVals arg (absent → no attributes).
    // Perl L333: Tokenize($recap) — catcode-aware, so TeX macros in
    // titles stay live (accents, math). Explode leaked them verbatim.
    // `tokenize_bib_field`, not `Tokenize!`: treatment 1 is the safety net under
    // the escaping above — anything the escaper deliberately left alone (a
    // `%` inside `\url{…}`) is still read as data rather than as a comment.
    // `assembled`: `recased` is a re-cased *raw* field, i.e. .bib source text.
    let recased_tokens = bib_tex_tokens(TeXString::assembled(recased));
    let inv = Invocation!(T_CS!("\\bib@@field"),
      vec![tag_tokens, Tokens!(), recased_tokens]);
    Ok(inv)
  });

  // \bib@@booktitle{field}{tag} — Perl L336-337.
  // Aliased to `\bib@@field`, NOT to `\bib@@title`. Perl L335
  // explicitly notes "I'd thought booktitle were treated like title,
  // but I think I was mistaken."
  DefMacro!("\\bib@@booktitle{}{}", "\\bib@@field{#1}{#2}");

  // Field handlers — Perl L342-351.

  // Ignore the field (used for fields the entry-type doesn't want).
  // `Verbatim Verbatim` reads two raw arg slots and discards both.
  def_macro_noop("\\bib@field@@ignore Verbatim Verbatim")?;

  // Default field handler: route to `\bib@field@unknownasdata`, which
  // emits a `<ltx:bib-data role='<field>'>` with the raw value. The
  // second `Verbatim` arg is captured but the body discards it (Perl
  // L346 comment: "IGNORE the tokenized data.").
  DefMacro!(
    "\\bib@field@default@default Verbatim Verbatim",
    "\\bib@field@unknownasdata{#1}"
  );

  // Emit `<ltx:bib-data role='<field>'>#rawdata</ltx:bib-data>` where
  // `#rawdata` is the current entry's PROCESSED (digested) field value.
  //
  // Perl L347-351: the `afterDigest` closure calls
  // `currentBibEntryField($field)` which returns the DIGESTED Tokens
  // form of the field (not the raw source string). Falls back to the
  // raw source only if the processed form is absent — handles fields
  // that were never digested.
  DefConstructor!("\\bib@field@unknownasdata Verbatim",
  "<ltx:bib-data role='#1'>#rawdata</ltx:bib-data>",
  // The `#rawdata` content must be set in `properties` (evaluated at
  // construction, BEFORE the body), NOT `after_digest` (which runs after the
  // body is already built — the previous code set the property too late, so
  // every unknown bib field came out as an EMPTY `<ltx:bib-data role=.../>`,
  // dropping its value entirely; Perl emits the value). `args[0]` is the field
  // name (#1).
  properties => sub[args] {
    let field = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    // The value goes out as a STRING, not Tokens: the constructor's `#prop`
    // content-insertion handles `Stored::String` and silently drops
    // `Stored::Tokens` (the old `after_digest` + `Stored::Tokens(...)` produced
    // an EMPTY `<ltx:bib-data role=.../>`, dropping every unknown field's value).
    // Read the RAW field first: this element reproduces the field's source text,
    // and a Tokens round-trip eats the space that terminates a control word
    // (`\ndash 693` -> `\ndash693`). Perl reads `currentBibEntryField`, but
    // there that IS the string.
    let s = current_entry_raw_field(&field)
      .or_else(|| current_entry_field(&field).map(|t| t.to_string()))
      .unwrap_or_default();
    Ok(stored_map!("rawdata" => Stored::String(pin(&s))))
  });

  // -------- Phase 4: entry-type prepare/complete + field aliases --------
  // Perl `BibTeX.pool.ltxml:355-543` — every standard BibTeX entry
  // type (article, book, inbook, incollection, inproceedings,
  // manual, thesis variants, proceedings, report, unpublished) gets
  // a `*@prepare` macro that calls copyCrossrefFields() for its
  // required-field set, plus per-(type,field) routing macros that
  // direct fields into the right XML containers.

  // --- Default entry handlers (Perl L207-211) ---

  // \bib@entry@default@prepare — copy date/year/month/day across.
  DefMacro!("\\bib@entry@default@prepare", sub[_args] {
    copy_crossref_fields(&["date", "year", "month", "day"]);
    Ok(Tokens!())
  });

  // \bib@entry@default@complete — invoke MR/Zbl/origbibentry synth.
  // (The synth macros themselves are Phase 5 — not yet ported. The
  // bindings will resolve to undefined CSes until then, but the
  // entry-complete macro itself must exist for Phase 4 dispatch.)
  DefMacro!(
    "\\bib@entry@default@complete",
    "\\bib@synthesize@mr\\bib@synthesize@zbl\\bib@@origbibentry"
  );

  // --- article (Perl L366-370) ---
  DefMacro!("\\bib@entry@article@prepare", sub[_args] {
    copy_crossref_fields(&["author", "title", "journal"]);
    Ok(Tokens!())
  });
  DefMacro!(
    "\\bib@field@article@journal",
    "\\bib@addto@related{journal}{host}\\bib@@field{ltx:bib-title}"
  );

  // --- book (Perl L377) ---
  DefMacro!("\\bib@entry@book@prepare", sub[_args] {
    copy_crossref_fields(&["author", "editor", "title", "publisher"]);
    Ok(Tokens!())
  });

  // --- booklet (Perl L384) ---
  DefMacro!("\\bib@entry@booklet@prepare", sub[_args] {
    copy_crossref_fields(&["title"]);
    Ok(Tokens!())
  });

  // --- conference alias → inproceedings (Perl L388) ---
  DefMacro!("\\bib@entry@conference@alias", "inproceedings");

  // --- inbook (Perl L396-413) ---
  DefMacro!("\\bib@entry@inbook@prepare", sub[_args] {
    copy_crossref_fields(&["author", "editor", "title", "chapter", "pages", "publisher"]);
    Ok(Tokens!())
  });
  DefMacro!(
    "\\bib@field@inbook@booktitle",
    "\\bib@addto@related{book}{host}\\bib@@booktitle{ltx:bib-title}"
  );
  DefMacro!(
    "\\bib@field@inbook@editor",
    "\\bib@addto@related{book}{host}\\bib@@names{editor}"
  );
  DefMacro!(
    "\\bib@field@inbook@publisher",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-publisher}"
  );
  DefMacro!(
    "\\bib@field@inbook@number",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-part}[role=number]"
  );
  DefMacro!(
    "\\bib@field@inbook@volume",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-part}[role=volume]"
  );
  DefMacro!(
    "\\bib@field@inbook@series",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-part}[role=series]"
  );
  DefMacro!(
    "\\bib@field@inbook@address",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-place}"
  );
  DefMacro!(
    "\\bib@field@inbook@edition",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-edition}"
  );

  // --- incollection (Perl L422-440) ---
  DefMacro!("\\bib@entry@incollection@prepare", sub[_args] {
    copy_crossref_fields(&["author", "title", "booktitle", "publisher"]);
    Ok(Tokens!())
  });
  DefMacro!(
    "\\bib@field@incollection@booktitle",
    "\\bib@addto@related{book}{host}\\bib@@booktitle{ltx:bib-title}"
  );
  DefMacro!(
    "\\bib@field@incollection@editor",
    "\\bib@addto@related{book}{host}\\bib@@names{editor}"
  );
  DefMacro!(
    "\\bib@field@incollection@publisher",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-publisher}"
  );
  DefMacro!(
    "\\bib@field@incollection@number",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-part}[role=number]"
  );
  DefMacro!(
    "\\bib@field@incollection@volume",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-part}[role=volume]"
  );
  DefMacro!(
    "\\bib@field@incollection@series",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-part}[role=series]"
  );
  DefMacro!(
    "\\bib@field@incollection@address",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-place}"
  );
  DefMacro!(
    "\\bib@field@incollection@edition",
    "\\bib@addto@related{book}{host}\\bib@@field{ltx:bib-edition}"
  );

  // --- inproceedings (Perl L449-474) ---
  DefMacro!("\\bib@entry@inproceedings@prepare", sub[_args] {
    copy_crossref_fields(&["author", "title", "booktitle", "publisher"]);
    Ok(Tokens!())
  });
  DefMacro!(
    "\\bib@field@inproceedings@booktitle",
    "\\bib@addto@related{proceedings}{host}\\bib@@booktitle{ltx:bib-title}"
  );
  DefMacro!(
    "\\bib@field@inproceedings@editor",
    "\\bib@addto@related{proceedings}{host}\\bib@@names{editor}"
  );
  DefMacro!(
    "\\bib@field@inproceedings@number",
    "\\bib@addto@related{proceedings}{host}\\bib@@field{ltx:bib-part}[role=number]"
  );
  DefMacro!(
    "\\bib@field@inproceedings@volume",
    "\\bib@addto@related{proceedings}{host}\\bib@@field{ltx:bib-part}[role=volume]"
  );
  DefMacro!(
    "\\bib@field@inproceedings@series",
    "\\bib@addto@related{proceedings}{host}\\bib@@field{ltx:bib-part}[role=series]"
  );
  DefMacro!(
    "\\bib@field@inproceedings@organization",
    "\\bib@addto@related{proceedings}{host}\\bib@@field{ltx:bib-organization}"
  );
  DefMacro!(
    "\\bib@field@inproceedings@publisher",
    "\\bib@addto@related{proceedings}{host}\\bib@@field{ltx:bib-publisher}"
  );
  DefMacro!(
    "\\bib@field@inproceedings@conference",
    "\\bib@addto@related{conference}{event}\\bib@@field{ltx:bib-title}"
  );
  DefMacro!(
    "\\bib@field@inproceedings@meeting",
    "\\bib@addto@related{conference}{event}\\bib@@field{ltx:bib-title}"
  );
  DefMacro!(
    "\\bib@field@inproceedings@location",
    "\\bib@addto@related{conference}{event}\\bib@@field{ltx:bib-place}"
  );
  DefMacro!(
    "\\bib@field@inproceedings@place",
    "\\bib@addto@related{conference}{event}\\bib@@field{ltx:bib-place}"
  );

  // --- manual (Perl L481) ---
  DefMacro!("\\bib@entry@manual@prepare", sub[_args] {
    copy_crossref_fields(&["title"]);
    Ok(Tokens!())
  });

  // --- thesis / mastersthesis / phdthesis (Perl L488-504) ---
  DefMacro!("\\bib@entry@thesis@prepare", sub[_args] {
    copy_crossref_fields(&["author", "title", "school"]);
    Ok(Tokens!())
  });
  DefMacro!("\\bib@entry@mastersthesis@alias", "thesis");
  DefMacro!(
    "\\bib@entry@mastersthesis@complete",
    "\\bib@addtype{Master's Thesis}"
  );
  DefMacro!("\\bib@entry@phdthesis@alias", "thesis");
  DefMacro!(
    "\\bib@entry@phdthesis@complete",
    "\\bib@addtype{Ph.D. Thesis}"
  );

  // --- proceedings (Perl L512-520) ---
  DefMacro!("\\bib@entry@proceedings@prepare", sub[_args] {
    copy_crossref_fields(&["title"]);
    Ok(Tokens!())
  });
  // Perl L514: if entry already has a title field, EAT the booktitle
  // arg and emit nothing; else route booktitle as the title.
  DefMacro!("\\bib@field@proceedings@booktitle {}", sub[args] {
    if current_entry_field("title").is_some() {
      // Discard the arg; emit nothing.
      Ok(Tokens!())
    } else {
      // Re-emit as `\bib@@field{ltx:bib-title}{<arg>}`.
      let body = args[0].clone().owned_tokens().unwrap_or_default();
      Ok(Invocation!(T_CS!("\\bib@@field"),
        vec![Tokens::new(Explode!("ltx:bib-title")), Tokens!(), body]))
    }
  });

  // --- techreport / report (Perl L526-528) ---
  DefMacro!("\\bib@entry@techreport@alias", "report");
  DefMacro!("\\bib@entry@report@prepare", sub[_args] {
    copy_crossref_fields(&["author", "title", "institution"]);
    Ok(Tokens!())
  });
  DefMacro!(
    "\\bib@entry@techreport@complete",
    "\\bib@addtype{Technical report}"
  );

  // --- unpublished (Perl L535) ---
  DefMacro!("\\bib@entry@unpublished@prepare", sub[_args] {
    copy_crossref_fields(&["author", "title"]);
    Ok(Tokens!())
  });

  // --- website aliases (Perl L539-542) ---
  DefMacro!("\\bib@entry@online@alias", "website");
  DefMacro!("\\bib@entry@electronic@alias", "website");
  DefMacro!("\\bib@entry@www@alias", "website");
  DefMacro!("\\bib@entry@webpage@alias", "website");

  // -------- Phase 4 (continued): default field handlers --------
  // Perl L549-783. Routing macros that direct each known field name
  // to the right XML container/role.

  // Agents — name lists.
  DefMacro!("\\bib@field@default@author", "\\bib@@names{author}");
  DefMacro!("\\bib@field@default@editor", "\\bib@@names{editor}");
  DefMacro!("\\bib@field@default@translator", "\\bib@@names{translator}");

  // Titles.
  DefMacro!(
    "\\bib@field@default@title",
    "\\bib@@title{title}{ltx:bib-title}"
  );
  DefMacro!(
    "\\bib@field@default@subtitle",
    "\\bib@@field{ltx:bib-subtitle}"
  );

  // Origin info.
  DefMacro!(
    "\\bib@field@default@date",
    "\\bib@@field{ltx:bib-date}[role=publication]"
  );
  DefMacro!(
    "\\bib@field@default@edition",
    "\\bib@@field{ltx:bib-edition}"
  );
  DefMacro!("\\bib@field@default@address", "\\bib@@field{ltx:bib-place}");
  DefMacro!(
    "\\bib@field@default@publisher",
    "\\bib@@field{ltx:bib-publisher}"
  );
  DefMacro!(
    "\\bib@field@default@institution",
    "\\bib@@field{ltx:bib-organization}"
  );
  DefMacro!(
    "\\bib@field@default@organization",
    "\\bib@@field{ltx:bib-organization}"
  );
  DefMacro!(
    "\\bib@field@default@school",
    "\\bib@@field{ltx:bib-organization}"
  );
  DefMacro!("\\bib@field@default@status", "\\bib@@field{ltx:bib-status}");

  // year — Perl L623-638: synthesize a date if no date field exists.
  // Take the raw year, optionally append `-<month>` (mapped from
  // English/abbrev names to digit) and `-<day>`. Then re-emit as a
  // `\bib@field@default@date{<date>}` invocation. If a `date` field
  // is already set, ignore the year (per Perl `currentBibEntryField('date')`).
  DefMacro!("\\bib@field@default@year {}", sub[args] {
    if current_entry_field("date").is_some() {
      return Ok(Tokens!());
    }
    let mut date = if args[0].is_some() { args[0].to_string() } else {
      current_entry_field("year").map(|t| t.to_string())
        .or_else(|| current_entry_raw_field("year"))
        .unwrap_or_default()
    };
    let month_lookup = |s: &str| -> Option<&'static str> {
      match s.to_lowercase().as_str() {
        "jan" | "january" => Some("1"),
        "feb" | "february" => Some("2"),
        "mar" | "march" => Some("3"),
        "apr" | "april" => Some("4"),
        "may" => Some("5"),
        "jun" | "june" => Some("6"),
        "jul" | "july" => Some("7"),
        "aug" | "august" => Some("8"),
        "sep" | "september" => Some("9"),
        "oct" | "october" => Some("10"),
        "nov" | "november" => Some("11"),
        "dec" | "december" => Some("12"),
        _ => None,
      }
    };
    let month_str: Option<String> = current_entry_raw_field("month")
      .as_deref()
      .and_then(month_lookup)
      .map(str::to_string)
      .or_else(|| {
        current_entry_field("month").and_then(|t| {
          let s = t.to_string();
          month_lookup(&s).map(str::to_string).or(Some(s))
        })
      });
    if let Some(mstr) = month_str {
      // Pad single digit month with leading 0.
      let mpad = if mstr.len() == 1 && mstr.chars().all(|c| c.is_ascii_digit()) {
        format!("0{mstr}")
      } else { mstr };
      date.push('-');
      date.push_str(&mpad);
      if let Some(day) = current_entry_field("day") {
        let day_s = day.to_string();
        let dpad = if day_s.len() == 1 && day_s.chars().all(|c| c.is_ascii_digit()) {
          format!("0{day_s}")
        } else { day_s };
        date.push('-');
        date.push_str(&dpad);
      }
    }
    // Emit `\bib@field@default@date{<date>}` literally — `Invocation!`
    // would strip the arg here since `\bib@field@default@date` is a
    // parameter-less DefMacro whose expansion expects the value to be
    // sitting in the input stream (so `\bib@@field`'s Digested arg
    // picks it up post-expansion). Mirrors Perl L638:
    // `(T_CS('\bib@field@default@date'), T_BEGIN, Tokenize($date)->unlist, T_END)`.
    let mut out_toks: Vec<Token> = Vec::new();
    out_toks.push(T_CS!("\\bib@field@default@date"));
    out_toks.push(T_BEGIN!());
    // `assembled`: `date` is assembled here from digits and `-` separators.
    out_toks.extend(bib_tex_tokens(TeXString::assembled(date)).unlist());
    out_toks.push(T_END!());
    Ok(Tokens::new(out_toks))
  });

  DefMacro!(
    "\\bib@field@default@howpublished",
    "\\bib@@field{ltx:bib-note}[role=publication]"
  );

  // Part info.
  DefMacro!(
    "\\bib@field@default@chapter",
    "\\bib@@field{ltx:bib-part}[role=chapter]"
  );
  DefMacro!(
    "\\bib@field@default@number",
    "\\bib@@field{ltx:bib-part}[role=number]"
  );
  DefMacro!(
    "\\bib@field@default@volume",
    "\\bib@@field{ltx:bib-part}[role=volume]"
  );
  DefMacro!(
    "\\bib@field@default@part",
    "\\bib@@field{ltx:bib-part}[role=part]"
  );
  DefMacro!(
    "\\bib@field@default@series",
    "\\bib@@field{ltx:bib-part}[role=series]"
  );
  DefMacro!(
    "\\bib@field@default@pages",
    "\\bib@@field{ltx:bib-part}[role=pages]\\bib@@pages"
  );

  // \bib@@pages — Perl L670-674: post-digestion fixup that takes the
  // raw `pages` field, normalises `-` runs to `--` (so the em-dash
  // ligature kicks in), and stuffs the result back into the
  // construction property.
  DefConstructor!("\\bib@@pages{}", "#pages",
  after_digest => sub[whatsit] {
    let raw = current_entry_raw_field("pages").unwrap_or_default();
    // Collapse runs of `-` to `--` (matches Perl `s/-+/--/g`).
    let mut normalised = String::with_capacity(raw.len() + 2);
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
      if c == '-' {
        while chars.peek() == Some(&'-') { chars.next(); }
        normalised.push_str("--");
      } else {
        normalised.push(c);
      }
    }
    // Perl L674: `Digest(Tokenize($pages))` — tokenize (so `\ndash` &co stay
    // live control sequences), then DIGEST. The digest step is load-bearing:
    // `#pages` is a content slot, and `prop_digested!` only renders
    // `Stored::Digested`/`VecDigested` — a `Stored::Tokens` fell through to its
    // catch-all and rendered as NOTHING, so every amsrefs `pages` field came
    // out as an empty `<ltx:bib-part role="pages"/>`. Witness arXiv 2508.17585.
    // Third and last door from `.bib` data into the TeX regime — `\bib@@pages`
    // also re-reads the raw field rather than using the entry line's value, so
    // it escapes too, on the same rule (`escape_bib_data_specials`).
    whatsit.set_property(
      "pages",
      // `assembled`: `normalised` is the raw `pages` field, `-` runs collapsed.
      Stored::Digested(digest(bib_tex_tokens(TeXString::assembled(normalised)))?),
    );
  });

  // Standard BibTeX fields.
  DefMacro!(
    "\\bib@field@default@annote",
    "\\bib@@field{ltx:bib-note}[role=annotation]"
  );

  // crossref — Perl L684-686: emit a `<ltx:bib-related role='host'
  // bibrefs='<key>'>` empty placeholder. Used by post-processing to
  // resolve the cross-reference into the actual bib-related XML.
  DefConstructor!("\\bib@field@default@crossref Semiverbatim",
  sub [document, args] {
    let raw_key = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let clean = clean_bib_key(&raw_key);
    let mut attrs = FxAttrMap::default();
    attrs.insert("role".to_string(), "host".to_string());
    attrs.insert("bibrefs".to_string(), clean);
    bib_add_to_container(document, "ltx:bib-related", None, attrs)?;
  });

  DefConstructor!(
    "\\bib@field@default@key Digested",
    "<ltx:bib-key>#1</ltx:bib-key>"
  );

  DefMacro!(
    "\\bib@field@default@note",
    "\\bib@@field{ltx:bib-note}[role=annotation]"
  );

  DefConstructor!(
    "\\bib@field@default@type Digested",
    "<ltx:bib-type>#1</ltx:bib-type>"
  );

  // Non-standard fields.
  // The three `ltx:bib-extract` fields are read VERBATIM, not digested as TeX.
  //
  // Perl digests them (BibTeX.pool.ltxml L708/716/732) and that is a bug both
  // engines shared: `abstract` and `keywords` are bulk prose, and real BibTeX
  // never lets them reach LaTeX at all. A `.bst` declares a closed ENTRY field
  // list — plain/unsrt/alpha/abbrv all omit `abstract` and `keywords` — so
  // bibtex(1) drops them when writing the `.bbl`, and pdflatex never sees them.
  // Verified by running bibtex 0.99d: the `.bbl` for an entry with
  // `abstract = {... 64.84% ...}` contains author/title/journal/year and no
  // trace of the abstract.
  //
  // Digesting them is therefore an artifact of reading `.bib` DIRECTLY instead
  // of running bibtex+bst, and it is destructive: a `%` in an abstract (a
  // percentage — utterly routine) is a TeX comment, so it eats the rest of the
  // line including the field's closing brace, the entry's group never closes,
  // and `\end{bib@entry}` fails. Entries then stack open and the WHOLE
  // bibliography is lost. Witness 2605.00184 (`warm-ref.bib`, a Mendeley export
  // with an abstract on every entry): 52 entries, 0 emitted, 102 errors — and
  // same-host Perl is worse, 101 errors plus a `too_many_errors` Fatal and no
  // output at all. Perl's own `\bib@field@default@abstract` cannot survive its
  // own input.
  //
  // The KEY is still supported, exactly as Perl supports it — the same
  // `ltx:bib-extract[@role]` element is emitted with its value (recovered via
  // `bib_extract_text`, Perl's own "IGNORE the tokenized data" idiom — see that
  // helper). Only the READING changes: `Verbatim` neutralizes `%` and `\`
  // (`base_parameter_types.rs`, `begin_semiverbatim(Some(&['%', '\\']))`), so
  // the content cannot break the entry. Nothing renders `ltx:bib-extract` —
  // no FMT_SPEC in `make_bibliography.rs` queries it — so losing markup inside
  // it costs nothing.
  //
  // SURPASS-PERL, with pdflatex as ground truth rather than preference.
  // OXIDIZED_DESIGN #73. Guard:
  // `06_cluster_bibliography::bib_abstract_percent_does_not_sink_the_entry`.
  DefConstructor!(
    "\\bib@field@default@abstract Verbatim",
    "?#rawdata(<ltx:bib-extract role='abstract'>#rawdata</ltx:bib-extract>)()",
    properties => sub[_args] {
      // Emit NOTHING when the field is empty. `abstract = {},` is a routine
      // reference-manager export (witness 2605.00555 `refs.bib` L956), and an
      // element opened with no content is never closed — every later field then
      // nests inside `ltx:bib-extract` and the entry is malformed. Leaving the
      // property undefined makes the `?#rawdata(...)()` conditional pick the
      // empty branch. Same guard fixture case: `emptyabstract`.
      let text = bib_extract_text("abstract");
      Ok(if text.is_empty() {
        stored_map!()
      } else {
        stored_map!("rawdata" => Stored::String(pin(&text)))
      })
    }
  );
  DefMacro!("\\bib@field@default@archive", "\\bib@@field{ltx:bib-links}");
  DefConstructor!(
    "\\bib@field@default@contents Verbatim",
    "?#rawdata(<ltx:bib-extract role='contents'>#rawdata</ltx:bib-extract>)()",
    properties => sub[_args] {
      // Emit NOTHING when the field is empty. `abstract = {},` is a routine
      // reference-manager export (witness 2605.00555 `refs.bib` L956), and an
      // element opened with no content is never closed — every later field then
      // nests inside `ltx:bib-extract` and the entry is malformed. Leaving the
      // property undefined makes the `?#rawdata(...)()` conditional pick the
      // empty branch. Same guard fixture case: `emptyabstract`.
      let text = bib_extract_text("contents");
      Ok(if text.is_empty() {
        stored_map!()
      } else {
        stored_map!("rawdata" => Stored::String(pin(&text)))
      })
    }
  );
  DefMacro!(
    "\\bib@field@default@copyright",
    "\\bib@@field{ltx:bib-date}[role=copyright]"
  );
  DefMacro!("\\bib@field@default@eprint", "\\bib@@field{ltx:bib-links}");
  DefMacro!(
    "\\bib@field@default@preprint",
    "\\bib@@field{ltx:bib-links}"
  );
  DefConstructor!(
    "\\bib@field@default@keywords Verbatim",
    "?#rawdata(<ltx:bib-extract role='keywords'>#rawdata</ltx:bib-extract>)()",
    properties => sub[_args] {
      // Emit NOTHING when the field is empty. `abstract = {},` is a routine
      // reference-manager export (witness 2605.00555 `refs.bib` L956), and an
      // element opened with no content is never closed — every later field then
      // nests inside `ltx:bib-extract` and the entry is malformed. Leaving the
      // property undefined makes the `?#rawdata(...)()` conditional pick the
      // empty branch. Same guard fixture case: `emptyabstract`.
      let text = bib_extract_text("keywords");
      Ok(if text.is_empty() {
        stored_map!()
      } else {
        stored_map!("rawdata" => Stored::String(pin(&text)))
      })
    }
  );
  DefMacro!(
    "\\bib@field@default@language",
    "\\bib@@field{ltx:bib-language}"
  );

  DefConstructor!(
    "\\bib@field@default@url Verbatim",
    "<ltx:bib-url href='#1'>Link</ltx:bib-url>"
  );

  DefMacro!(
    "\\bib@field@default@enote",
    "\\bib@@field{ltx:bib-note}[role=electronic-annotation]"
  );

  // Identifiers — doi/isbn/issn/lccn/pii. All take a Semiverbatim
  // value, run it through `process_identifier` (trim+sanitise), and
  // emit a `<ltx:bib-identifier scheme=...>` element. doi additionally
  // builds an https://dx.doi.org/ href and percent-encodes any non
  // url-safe chars.
  DefConstructor!("\\bib@field@default@doi Semiverbatim",
  "<ltx:bib-identifier scheme='doi' id='#id' href='#href'>Document</ltx:bib-identifier>",
  properties => sub[args] {
    let raw = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let id = process_identifier(&raw);
    // Percent-encode `[^0-9a-zA-Z./\-+]` chars for the URL.
    let mut href = String::from("https://dx.doi.org/");
    for c in id.chars() {
      if c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | '-' | '+') {
        href.push(c);
      } else {
        let mut buf = [0u8; 4];
        for &b in c.encode_utf8(&mut buf).as_bytes() {
          href.push_str(&format!("%{:02X}", b));
        }
      }
    }
    Ok(stored_map!("id" => id, "href" => href))
  });

  DefConstructor!("\\bib@field@default@isbn Semiverbatim",
  "<ltx:bib-identifier scheme='isbn' id='#id'>ISBN #1</ltx:bib-identifier>",
  properties => sub[args] {
    let raw = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("id" => process_identifier(&raw)))
  });

  DefConstructor!("\\bib@field@default@issn Semiverbatim",
  "<ltx:bib-identifier scheme='issn' id='#id'>ISSN #1</ltx:bib-identifier>",
  properties => sub[args] {
    let raw = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("id" => process_identifier(&raw)))
  });

  DefConstructor!("\\bib@field@default@lccn Semiverbatim",
  "<ltx:bib-identifier scheme='lccn' id='#id'>LCCN #1</ltx:bib-identifier>",
  properties => sub[args] {
    let raw = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("id" => process_identifier(&raw)))
  });

  DefConstructor!("\\bib@field@default@pii Semiverbatim",
  "<ltx:bib-identifier scheme='pii' id='#id'>PII #1</ltx:bib-identifier>",
  properties => sub[args] {
    let raw = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("id" => process_identifier(&raw)))
  });

  // Review (Perl L794-795).
  DefConstructor!(
    "\\bib@field@default@review Digested",
    "<ltx:bib-review>Review #1</ltx:bib-review>"
  );

  // -------- Phase 5: MR / Zbl synthesis + origbibentry --------
  // Perl L803-860. Emits `<ltx:bib-review>` / `<ltx:bib-identifier>`
  // nodes for AMS MathSciNet (mrnumber, mrreviewer) and
  // Zentralblatt (zblno, zblreviewer) fields; embeds a verbatim
  // BibTeX-source roundtrip of the entry as a `<ltx:bib-data role='self'>`.

  // \bib@synthesize@mr — Perl L803-810. Emit \bib@@mr if either
  // mrnumber or mrreviewer is set, else nothing.
  // `untex()`, NOT `to_string()`, on all four MR/Zbl fields. Perl's
  // `currentBibEntryField` returns a plain STRING (`Pre/BibTeX/Entry.pm` L38:
  // `$$self{fieldmap}{$key}`), so `Tokenize($mrnumber)` at L806-807 never had a
  // Tokens round trip to get wrong. Our `current_entry_field` returns `Tokens`,
  // so recovering Perl's string requires `untex()` — `to_string()` drops the
  // space that terminates a control word and welds it to the following text.
  // Witness 2605.11579 (`biblo.bib`), five reviewer names, one per shape:
  // `MRREVIEWER = {Fran\c cois\ Digne}` -> `undefined:\ccois`;
  // `{S. I. Gel\cprime fand}` -> `\cprimefand`; `{Sait Hal\i c\i o\u{g}lu}`
  // -> `\ic` and `\io`; `{Dragomir \v{Z}. \Dbar okovi\'{c}}` -> `\Dbarokovi`.
  // Third site of this defect after `\bib@@names` (PR #399) and dcolumn/overpic
  // (PR #400). Guard: `06_cluster_bibliography::bib_mr_reviewer_accent_survives_reversion`.
  DefMacro!("\\bib@synthesize@mr", sub[_args] {
    let mrnumber = current_entry_field("mrnumber").map(Tokens::untex_string);
    let mrreviewer = current_entry_field("mrreviewer").map(Tokens::untex_string);
    if mrnumber.is_none() && mrreviewer.is_none() {
      return Ok(Tokens!());
    }
    let mr_tks = bib_tex_tokens(mrnumber.unwrap_or_default());
    let rev_tks = match mrreviewer {
      Some(r) => bib_tex_tokens(r),
      None => Tokens!(),
    };
    let inv = Invocation!(T_CS!("\\bib@@mr"), vec![mr_tks, rev_tks]);
    Ok(inv)
  });

  // \bib@@mr {}{} — Perl L812-826. Conditional template:
  //   isreview=true, reviewer set → bib-review (MR with reviewer)
  //   isreview=true, no reviewer → bib-review (plain MR)
  //   isreview=false → bib-identifier (just the MR id)
  // Id may arrive as "MR12345" or "12345" or "12345 (foo)"; strip
  // any MR prefix and a trailing parenthesised note, and flag
  // isreview if a note appears.
  DefConstructor!("\\bib@@mr {}{}",
  "?#isreview\
   (?#reviewer\
     (<ltx:bib-review scheme='mr' id='#id' href='#href'>MathReview (#reviewer)</ltx:bib-review>)\
     (<ltx:bib-review scheme='mr' id='#id' href='#href'>MathReview</ltx:bib-review>))\
   (<ltx:bib-identifier scheme='mr' id='#id' href='#href'>MathReview Entry</ltx:bib-identifier>)",
  properties => sub[args] {
    let raw_id = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let reviewer = args[1].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let mut id = raw_id.trim().to_string();
    let mut isreview = !reviewer.is_empty();
    // Perl regex: /^\s*(?:MR)?(\d+)\s+\(.*\)\s*$/ — strip optional
    // MR prefix; if a trailing `(...)` note exists, flag isreview.
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
      Regex::new(r"^(?:MR)?(\d+)\s+\(.*\)$").unwrap()
    });
    if let Some(caps) = re.captures(&id) {
      id = caps[1].to_string();
      isreview = true;
    }
    let href = format!("https://www.ams.org/mathscinet-getitem?mr={}", id);
    Ok(stored_map!(
      "isreview" => isreview,
      "id" => id,
      "href" => href,
      "reviewer" => reviewer))
  });

  // \bib@synthesize@zbl — Perl L828-835. Same shape as mr but
  // unconditional `isreview` (no MR-style id stripping).
  DefMacro!("\\bib@synthesize@zbl", sub[_args] {
    let zblno = current_entry_field("zblno").map(Tokens::untex_string);
    let zblreviewer = current_entry_field("zblreviewer").map(Tokens::untex_string);
    if zblno.is_none() && zblreviewer.is_none() {
      return Ok(Tokens!());
    }
    let zbl_tks = bib_tex_tokens(zblno.unwrap_or_default());
    let rev_tks = match zblreviewer {
      Some(r) => bib_tex_tokens(r),
      None => Tokens!(),
    };
    let inv = Invocation!(T_CS!("\\bib@@zbl"), vec![zbl_tks, rev_tks]);
    Ok(inv)
  });

  // \bib@@zbl {}{} — Perl L837-845. Simpler than MR; always emits
  // bib-review, but the suffix `(reviewer)` is conditional.
  DefConstructor!("\\bib@@zbl {}{}",
  "?#reviewer\
   (<ltx:bib-review scheme='zbl' id='#id' href='#href'>ZentralBlatt (#reviewer)</ltx:bib-review>)\
   (<ltx:bib-review scheme='zbl' id='#id' href='#href'>ZentralBlatt</ltx:bib-review>)",
  properties => sub[args] {
    let id = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let reviewer = args[1].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let href = format!("https://zbmath.org/{}", id);
    Ok(stored_map!(
      "id" => id,
      "href" => href,
      "reviewer" => reviewer))
  });

  // \bib@field@default@links — Perl L850-851.
  DefConstructor!(
    "\\bib@field@default@links Digested",
    "<ltx:bib-links>#1</ltx:bib-links>"
  );

  // \bib@@origbibentry — Perl L856-860. Embed the BibTeX-source
  // form of the current entry into the XML as `<ltx:bib-data
  // role='self' type='BibTeX'>`. Uses `BibEntry::pretty_print`.
  DefConstructor!("\\bib@@origbibentry",
  "<ltx:bib-data role='self' type='BibTeX'>#bibentry</ltx:bib-data>",
  // Set `#bibentry` in `properties` (before the body) as a `Stored::String` —
  // same fix as `\bib@field@unknownasdata`: the constructor's `#prop`
  // content-insertion drops `Stored::Tokens`, so the old `after_digest` +
  // `Stored::Tokens(...)` left the embedded BibTeX source EMPTY.
  properties => sub[_args] {
    let pp = current_entry()
      .map(|e| e.borrow().pretty_print())
      .unwrap_or_default();
    Ok(stored_map!("bibentry" => Stored::String(pin(&pp))))
  });

  // -------- Phase 6: orchestration (Perl L111-190) --------
  // `\ProcessBibTeXEntry{key}` drives the per-entry pipeline:
  //   1. resolve type alias chain
  //   2. dispatch prepare macros (type → alias → default)
  //   3. open `<ltx:bibentry>` (via the `{bib@entry}` environment)
  //   4. dispatch each field via the most specific handler
  //   5. dispatch complete macros (type → alias → default)
  //   6. close `<ltx:bibentry>`
  //
  // The Perl pool splits this across two DefPrimitives
  // (`\bibentry@prepare` + `\bibentry@create`) with a `\stomach->bgroup;
  // AssignValue('CURRENT@BIBKEY' => $key); ...; egroup` dance that
  // gives the per-entry state automatic group-scope restore. The
  // Rust port does it all in one DefMacro returning a Tokens stream,
  // since our gullet handles the tokens-back path natively (no
  // openMouth needed). Divergence is documented under audit B1
  // (`current_bib_key` rustdoc + `docs/archive/BIBTEX_PORT_PLAN_2026-06-20.md`).
  DefMacro!("\\ProcessBibTeXEntry Semiverbatim", sub[args] {
    let key = if args[0].is_some() { args[0].to_string() } else {
      return Ok(Tokens!());
    };
    let entry_rc = match lookup_entry(&key) {
      Some(e) => e,
      None => return Ok(Tokens!()),
    };
    let entry = entry_rc.borrow();
    let origtype = entry.entry_type.clone();

    // Alias resolution: if `\bib@entry@<origtype>@alias` is defined,
    // its expansion is the resolved type. Otherwise resolved == orig.
    let alias_cs_name = format!("\\bib@entry@{}@alias", origtype);
    let alias_tok = T_CS!(alias_cs_name.as_str());
    let resolved_type = match lookup_definition(&alias_tok)? {
      Some(_) => {
        // Expand the alias CS via the gullet to get the target type
        // name. Most aliases are pure-text DefMacros (e.g. "thesis"),
        // so a single do_expand is enough.
        match do_expand(alias_tok) {
          Ok(toks) => toks.to_string(),
          Err(_) => origtype.clone(),
        }
      },
      None => origtype.clone(),
    };

    // Set the current bib key so the per-field handlers (which call
    // `current_entry_field` etc.) see the right entry.
    set_current_entry(&key);

    // Perl L134-166 assembles the entry as TeX *source lines* and hands
    // the join to a fresh Mouth. That is load-bearing, not incidental:
    // the field values come from the document, so they must be tokenized
    // under live catcodes (`\MR{...}` / `\ndash` in an amsrefs entry are
    // real control sequences), and tokenizing lazily from a Mouth is what
    // lets the handlers' `Verbatim`/`Semiverbatim` parameter types set
    // their catcodes *first* — which is what keeps `%`/`#`/`~` in a `url`
    // value intact. Building a pre-tokenized stream with `Explode!`
    // (catcode-12 OTHER throughout) did neither: `\MR{849427}` reached the
    // XML as the literal characters `\MR{849427}`. Witness arXiv 2508.17585.
    //
    // Handler names carry `@`, which is catcode-OTHER under the standard
    // table a Mouth reads with, so — exactly as Perl does — they are
    // invoked through `\csname ...\endcsname` rather than written bare.
    let mut lines: Vec<String> = Vec::new();
    // `\begin{bib@entry}{<type>}{<key>}`
    lines.push(format!("\\begin{{bib@entry}}{{{}}}{{{}}}", resolved_type, key));

    // Dispatch prepare macros. Perl L128-131: prepare for the
    // resolved type, then the orig type if different, then default.
    let prepare_csnames = [
      format!("\\bib@entry@{}@prepare", resolved_type),
      if origtype != resolved_type { format!("\\bib@entry@{}@prepare", origtype) } else { String::new() },
      "\\bib@entry@default@prepare".to_string(),
    ];
    for cs_name in &prepare_csnames {
      if cs_name.is_empty() { continue; }
      if lookup_definition(&T_CS!(cs_name.as_str()))?.is_some() {
        lines.push(format!("\\csname {}\\endcsname", &cs_name[1..]));
      }
    }

    // Dispatch each field via the most specific handler. Perl L147-157.
    for (field, value) in entry.raw_fields.iter() {
      if field.starts_with('_') { continue; }  // internal fields
      let candidates = [
        format!("\\bib@field@{}@{}", resolved_type, field),
        if origtype != resolved_type { format!("\\bib@field@{}@{}", origtype, field) } else { String::new() },
        format!("\\bib@field@default@{}", field),
      ];
      let mut handler: Option<(&str, Rc<dyn Definition>)> = None;
      for c in candidates.iter() {
        if c.is_empty() { continue; }
        if let Some(def) = lookup_definition(&T_CS!(c.as_str()))? {
          handler = Some((c.as_str(), def));
          break;
        }
      }
      let value = &undouble_escaped_ampersand(value);
      match handler {
        Some((h, def)) => {
          let v = bib_field_source(value, &def);
          lines.push(format!("\\csname {}\\endcsname{{{}}}", &h[1..], v));
        },
        None => {
          // Fallback per Perl L157: `\bib@field@default@default{field}{value}`.
          // Both of its slots are `Verbatim`, so the value is passed raw — the
          // same rule `bib_field_source` applies, reached without a lookup.
          lines.push(format!(
            "\\csname bib@field@default@default\\endcsname{{{}}}{{{}}}", field, value));
        },
      }
    }

    // Dispatch complete macros (Perl L158-164).
    let complete_csnames = [
      format!("\\bib@entry@{}@complete", resolved_type),
      if origtype != resolved_type { format!("\\bib@entry@{}@complete", origtype) } else { String::new() },
      "\\bib@entry@default@complete".to_string(),
    ];
    for cs_name in &complete_csnames {
      if cs_name.is_empty() { continue; }
      if lookup_definition(&T_CS!(cs_name.as_str()))?.is_some() {
        lines.push(format!("\\csname {}\\endcsname", &cs_name[1..]));
      }
    }

    // `\end{bib@entry}`
    lines.push("\\end{bib@entry}".to_string());

    // Release the entry borrow before handing control to the gullet.
    drop(entry);
    // Perl L165-166: `openMouth(Mouth->new($tex))` — autoclose (Perl passes
    // no `$noautoclose`), so the mouth pops itself once drained.
    //
    // OPAQUE to a balanced read, which is what Perl's readBalanced is for every
    // mouth (Gullet.pm L465-472 reads the current mouth only). This entry is a
    // self-contained input, not a continuation of the enclosing wrapper, so an
    // unbalanced field must cost only the rest of THIS entry. Left transparent,
    // one runaway argument — a bare `%` in a title is enough, since BibTeX has
    // no comment syntax inside an entry but TeX does — consumed every following
    // `\ProcessBibTeXEntry` and `\end{bibtex@bibliography}` too, so a 3-entry
    // `.bib` produced ONE entry and a single error where Perl produces all three
    // and four errors. Guard: `55_bibtex::runaway_field_costs_only_its_own_entry`.
    //
    // `% & # _` are read as ordinary characters in this mouth — treatment 1 of
    // OXIDIZED_DESIGN #74, "be `bibtex`". BibTeX's lexer has no comment syntax
    // inside an entry, no alignment, no parameters and no subscripts, so the
    // values `PreBibTeX` just handed us hold all four literally. Under TeX's
    // catcodes each misfires: `%` (14) comments out the rest of its line — the
    // field's own closing `}` included — so the entry's group never closes and
    // every later entry nests inside it; `&` (4) is a stray alignment tab and
    // is dropped, so "Taylor & Francis" printed as "Taylor Francis"; `#` (6)
    // reaches the Stomach as a parameter; `_` (8) is a subscript outside math.
    //
    // Witnesses — `%`: 2605.01196 (`doi={%doi:...}`) and 2605.02131 (a
    // percent-encoded URL in a `title`'s `\href`), 28 errors each with Perl
    // breaking identically (29 / 31 same-host), 2605.00879 (103).
    // `&`: 2605.01936 (7), 2605.06249 (3), 2605.00462 / 2605.03054 /
    // 2605.06624 / 2605.08753 / 2605.10409 (1 each).
    // `_`: 2605.06926 (8), 2605.01936 (6), 2605.04604 / 2605.08986 (2 each),
    // 2605.11300 / 2605.05898 (1 each).
    // Same-host Perl and bibtex+pdflatex break the same way on all three — the
    // decision that a field's content is DATA is what overrides them, and it is
    // ours to make because we read the `.bib` directly.
    //
    // Treatment 2 (`escape_bib_data_specials`, applied to the values written
    // into `lines` above) makes the synthesized `.bbl` valid TeX in the first
    // place; this mouth is the safety net under it, covering the values that
    // deliberately go through unescaped — a `url` href, a `doi` id, the inside
    // of a `\url{…}`.
    //
    // Guards: `06_cluster_bibliography::bib_field_percent_is_an_ordinary_character`,
    // `::bib_bare_ampersand_is_literal_data`, `::bib_field_specials_are_data_not_tex`.
    open_mouth_with(
      Mouth::new(&lines.join("\n"), None)?.with_bib_data_literals(),
      true,
      BalancedBoundary::Opaque,
    );
    Ok(Tokens!())
  });

  // `{bib@entry}` environment — Perl L185-190. Wraps an entry's
  // dispatched contents in `<ltx:bibentry>`, sets the auto-id via
  // `RefStepCounter('@bibitem')`, and snapshots the key on the
  // Whatsit. The CURRENT@BIBKEY state-value is NOT mirrored here —
  // see audit divergence B1 (Rust uses a thread-local already set by
  // `\ProcessBibTeXEntry`).
  DefEnvironment!("{bib@entry} Semiverbatim Semiverbatim",
  "<ltx:bibentry type='#1' key='#key' xml:id='#id'>#body</ltx:bibentry>",
  after_digest_begin => sub[whatsit] {
    let key_arg = whatsit.get_arg(2).map(|a| a.to_string()).unwrap_or_default();
    set_current_entry(&key_arg);
    whatsit.set_property("key", Stored::String(pin(&key_arg)));
    // Merge in the {id, refnum, ...} from the @bibitem counter step.
    let id_props = RefStepCounter!("@bibitem")?;
    whatsit.set_properties(id_props);
  });

  // `{bibtex@bibliography}` environment — Perl
  // `BibTeX.pool.ltxml:175-183`. The outer wrapper for the entries
  // emitted by `Pre::BibTeX::toTeX`. Delegates the heavy lifting
  // (id allocation, title resolution, bibstyle/citestyle lookup) to
  // `before_digest_bibliography` / `begin_bibliography_clean` in
  // `latex_constructs.rs`, which already port the Perl helpers used by
  // `\thebibliography`.
  //
  // OXIDIZED_DESIGN #75: `begin_bibliography_clean`, NOT
  // `begin_bibliography` — the `setupPseudoBibitem` half is deliberately
  // skipped here. Perl `BibTeX.pool.ltxml:183` calls the full
  // `beginBibliography`, which `\let`s `\par` AND `\\` to
  // `\par@in@bibliography`, a heuristic that starts a fresh
  // `\save@bibitem{}` whenever it fires. That rescue exists for
  // HAND-WRITTEN `thebibliography` lists whose author separated entries
  // with blank lines instead of `\bibitem`. This body is machine-generated
  // — one `\ProcessBibTeXEntry{key}` per line, never a missing `\bibitem`
  // — so the heuristic can only misfire, and it does: a blank line or a
  // `\\` inside a `.bib` FIELD VALUE injects a spurious `<ltx:bibitem>`
  // mid-field, which the model rejects and which is then never closed, so
  // every later entry nests inside the dangling `<ltx:surname>` /
  // `<ltx:bib-title>` / `<ltx:bib-note>`.
  // Witnesses: 2605.03313 (7 errors), 2605.03693 (7), 2605.11080 (1).
  DefEnvironment!("{bibtex@bibliography}",
  "<ltx:bibliography xml:id='#id' \
     bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort'>\
     <ltx:title font='#titlefont' _force_font='1'>#title</ltx:title>\
     <ltx:biblist>#body</ltx:biblist>\
   </ltx:bibliography>",
  before_digest => sub {
    crate::latex_constructs::before_digest_bibliography()?;
  },
  after_digest_begin => sub[whatsit] {
    crate::latex_constructs::begin_bibliography_clean(whatsit)?;
  });
});

#[cfg(test)]
mod tests {
  use super::*;

  fn fresh() { reset(); }

  #[test]
  fn entry_round_trips_fields_and_raw_fields() {
    fresh();
    let mut e = BibEntry::new("Smith2020", "article");
    e.add_field("title", Tokens::new(Vec::new())); // placeholder Tokens
    e.add_raw_field("title", "On Examples");
    e.add_raw_field("year", "2020");
    assert_eq!(e.key, "Smith2020");
    assert_eq!(e.entry_type, "article");
    assert_eq!(e.get_raw_field("title"), Some("On Examples"));
    assert_eq!(e.get_raw_field("year"), Some("2020"));
    assert_eq!(e.get_raw_field("missing"), None);
    let names: Vec<&str> = e.field_names().collect();
    assert_eq!(names, vec!["title"]);
  }

  #[test]
  fn current_entry_round_trip() {
    fresh();
    let mut e = BibEntry::new("MyKey", "book");
    e.add_raw_field("year", "1999");
    register_entry("MyKey", e);
    let cur = current_entry().expect("current entry registered");
    assert_eq!(cur.borrow().key, "MyKey");
    assert_eq!(current_entry_raw_field("year").as_deref(), Some("1999"));
    clear_current_entry();
    assert!(current_entry().is_none());
    // Re-set via set_current_entry
    assert!(set_current_entry("MyKey"));
    assert!(current_entry().is_some());
    // Unknown key
    assert!(!set_current_entry("Nope"));
  }

  #[test]
  fn normalized_key_makes_lookup_case_insensitive() {
    fresh();
    register_entry("FooBar", BibEntry::new("FooBar", "article"));
    // Perl NormalizeBibKey lowercases; both `foobar` and `FOOBAR`
    // resolve to the same entry.
    assert!(lookup_entry("foobar").is_some());
    assert!(lookup_entry("FOOBAR").is_some());
    assert!(lookup_entry("foo bar").is_some()); // SPACES_RE strips ws
    assert!(lookup_entry("Other").is_none());
  }

  #[test]
  fn copy_crossref_fields_pulls_from_parent() {
    fresh();
    // Set up a parent with author + journal, then a child with
    // crossref=parent. copy_crossref_fields should copy author into
    // child but not overwrite child's existing journal.
    let mut parent = BibEntry::new("Parent2020", "article");
    parent.add_raw_field("author", "Parent A.");
    parent.add_raw_field("journal", "Parent Journal");
    register_entry("Parent2020", parent);

    let mut child = BibEntry::new("Child2020", "article");
    child.add_raw_field("crossref", "Parent2020");
    child.add_raw_field("journal", "Child Journal");
    register_entry("Child2020", child);

    copy_crossref_fields(&["author", "journal"]);

    let c = current_entry().unwrap();
    assert_eq!(c.borrow().get_raw_field("author"), Some("Parent A."));
    // child's journal NOT overwritten
    assert_eq!(c.borrow().get_raw_field("journal"), Some("Child Journal"));
  }

  #[test]
  fn copy_crossref_handles_missing_crossref() {
    fresh();
    register_entry("Solo", BibEntry::new("Solo", "article"));
    // No-op: no crossref → no panic, no copy
    copy_crossref_fields(&["author", "title"]);
    let c = current_entry().unwrap();
    assert_eq!(c.borrow().get_raw_field("author"), None);
  }

  #[test]
  fn copy_crossref_handles_self_crossref() {
    fresh();
    let mut e = BibEntry::new("Loop", "article");
    e.add_raw_field("crossref", "Loop");
    e.add_raw_field("title", "Self-referential");
    register_entry("Loop", e);
    // Should NOT recurse / overwrite. Silently skip.
    copy_crossref_fields(&["title"]);
    let c = current_entry().unwrap();
    assert_eq!(c.borrow().get_raw_field("title"), Some("Self-referential"));
  }

  #[test]
  fn current_entry_returns_none_outside_block() {
    fresh();
    assert!(current_entry().is_none());
    assert!(current_entry_field("title").is_none());
    assert!(current_entry_raw_field("title").is_none());
  }

  // --- process_bib_name_list tests (Perl `processBibNameList`) ---

  #[test]
  fn name_first_last() {
    let r = process_bib_name_list("Jane Smith");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "Jane");
    assert_eq!(r.names[0].surname, "Smith");
    assert_eq!(r.names[0].lineage, "");
    assert!(!r.etal);
  }

  #[test]
  fn name_last_comma_first() {
    let r = process_bib_name_list("Smith, Jane");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "Jane");
    assert_eq!(r.names[0].surname, "Smith");
    assert_eq!(r.names[0].lineage, "");
  }

  #[test]
  fn name_with_lineage() {
    let r = process_bib_name_list("Smith, Jr., Bob");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].surname, "Smith");
    assert_eq!(r.names[0].lineage, "Jr.");
    assert_eq!(r.names[0].given, "Bob");
  }

  #[test]
  fn name_with_von_particle() {
    // "First von Last" — lowercase `von` triggers the split:
    // first = ["Ludwig"], rest = ["von", "Beethoven"] → surname.
    let r = process_bib_name_list("Ludwig von Beethoven");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "Ludwig");
    assert_eq!(r.names[0].surname, "von Beethoven");
  }

  #[test]
  fn name_all_capital_falls_back_to_last_word() {
    // No lowercase word — last word becomes surname per Perl
    // L909 `push(@pwords, pop(@first)) unless @pwords;`.
    let r = process_bib_name_list("John Q Public");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "John Q");
    assert_eq!(r.names[0].surname, "Public");
  }

  #[test]
  fn multiple_names_separated_by_and() {
    let r = process_bib_name_list("Jane Smith and John Doe and Alice Brown");
    assert_eq!(r.names.len(), 3);
    assert_eq!(r.names[0].surname, "Smith");
    assert_eq!(r.names[1].surname, "Doe");
    assert_eq!(r.names[2].surname, "Brown");
    assert!(!r.etal);
  }

  #[test]
  fn etal_others_marker() {
    let r = process_bib_name_list("Jane Smith and others");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].surname, "Smith");
    assert!(r.etal);
  }

  #[test]
  fn etal_etal_single_word_marker() {
    // Perl's etal regex (`^(others|et\s*al\.?)$/i`) requires the
    // whole "et al" token to be ONE word — splitWords splits on
    // whitespace so "et al." becomes two words and the regex
    // doesn't fire. Match Perl-faithful: only `etal` / `etal.` as
    // a single word triggers etal detection. The multi-word
    // "et al." case becomes a second author with surname="et al."
    // — same as Perl LaTeXML.
    let r = process_bib_name_list("Smith and etal.");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].surname, "Smith");
    assert!(r.etal);
  }

  #[test]
  fn et_al_multi_word_is_not_etal_per_perl() {
    // Perl-faithful: split words case stays as a second "author".
    let r = process_bib_name_list("Smith and et al.");
    assert_eq!(r.names.len(), 2);
    assert_eq!(r.names[0].surname, "Smith");
    assert_eq!(r.names[1].surname, "et al.");
    assert!(!r.etal);
  }

  #[test]
  fn braced_group_stays_atomic() {
    // BibTeX convention: `{De Long}` is a single surname token, the
    // braces protect "De" from being treated as a separate (capital)
    // word. After unwrapping for the lowercase check, it starts with
    // 'D' which is uppercase, so it falls into the "First Last"
    // branch and our heuristic puts the whole `{De Long}` as
    // surname.
    let r = process_bib_name_list("John {De Long}");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "John");
    assert_eq!(r.names[0].surname, "{De Long}");
  }

  #[test]
  fn tilde_treated_as_space() {
    // `~` is the LaTeX non-breaking space; BibTeX names use it for
    // initials. Perl's splitWords treats it as a hard space.
    let r = process_bib_name_list("J.~K. Rowling");
    assert_eq!(r.names.len(), 1);
    assert_eq!(r.names[0].given, "J. K.");
    assert_eq!(r.names[0].surname, "Rowling");
  }

  #[test]
  fn empty_input_returns_empty_list() {
    let r = process_bib_name_list("");
    assert!(r.names.is_empty());
    assert!(!r.etal);
  }

  // --- bib_container_xpath (find-or-create xpath construction) ---

  #[test]
  fn xpath_bare_tag_when_no_attrs() {
    let attrs = FxAttrMap::default();
    assert_eq!(bib_container_xpath("ltx:bib-name", &attrs), "ltx:bib-name");
  }

  #[test]
  fn xpath_single_attr() {
    let mut attrs = FxAttrMap::default();
    attrs.insert("role".to_string(), "authors".to_string());
    assert_eq!(
      bib_container_xpath("ltx:bib-related", &attrs),
      "ltx:bib-related[@role='authors']"
    );
  }

  #[test]
  fn xpath_sorts_attr_keys_for_cache_stability() {
    // Perl's `sort keys %attr` ensures the same (type,role) pair
    // always produces the same xpath, regardless of hash iteration
    // order. We mirror that — alphabetic order by attribute name.
    let mut attrs = FxAttrMap::default();
    attrs.insert("type".to_string(), "book".to_string());
    attrs.insert("role".to_string(), "host".to_string());
    let xpath = bib_container_xpath("ltx:bib-related", &attrs);
    assert_eq!(xpath, "ltx:bib-related[@role='host' and @type='book']");
  }

  // --- escape_bib_data_specials (the `.bib`-data -> TeX-source boundary) ---

  #[test]
  fn escape_specials_bare_are_escaped() {
    assert_eq!(
      escape_bib_data_specials("AT&T AT1G01010_v2 95% #3"),
      r"AT\&T AT1G01010\_v2 95\% \#3"
    );
  }

  /// Idempotency. Most real `.bib` files already write `\&`, `\%`, `\_` — a
  /// blind escaper would turn `\&` into `\\&`, i.e. a line break followed by an
  /// ampersand. Escaping twice must equal escaping once.
  #[test]
  fn escape_specials_is_idempotent() {
    let once = escape_bib_data_specials("AT&T at 95% with x_1");
    assert_eq!(escape_bib_data_specials(&once), once);
    assert_eq!(
      escape_bib_data_specials(r"AT\&T at 95\% with x\_1"),
      r"AT\&T at 95\% with x\_1"
    );
  }

  /// The tricky case the idempotency rule has to get right: in `\\&` the `\\`
  /// is a genuine line break, so the `&` after it is BARE and must be escaped —
  /// the second backslash is not an escape character, it is escaped itself.
  #[test]
  fn escape_specials_double_backslash_then_bare_special() {
    assert_eq!(
      escape_bib_data_specials(r"break \\& more"),
      r"break \\\& more"
    );
    assert_eq!(escape_bib_data_specials(r"break \\_x"), r"break \\\_x");
  }

  /// Math is the other hazard: `_` inside `$…$` is a real subscript and must
  /// stay catcode 8, while the same character outside is data.
  #[test]
  fn escape_specials_skips_math() {
    assert_eq!(
      escape_bib_data_specials("Bounds on $x_1+x_2$ for AT1G_v2"),
      r"Bounds on $x_1+x_2$ for AT1G\_v2"
    );
    assert_eq!(
      escape_bib_data_specials("display $$a_1$$ then b_2"),
      r"display $$a_1$$ then b\_2"
    );
    assert_eq!(
      escape_bib_data_specials(r"open \(x_1\) then y_2"),
      r"open \(x_1\) then y\_2"
    );
  }

  /// Markup passes through by construction — the escaper never touches `\`,
  /// `{` or `}`, so control words and their braces are untouched, and a
  /// space-form accent keeps the space that terminates its control word.
  #[test]
  fn escape_specials_leaves_markup_and_accents_alone() {
    assert_eq!(
      escape_bib_data_specials(r"\emph{Drosophila} genetics"),
      r"\emph{Drosophila} genetics"
    );
    assert_eq!(escape_bib_data_specials(r"{\v S}pakov"), r"{\v S}pakov");
    assert_eq!(escape_bib_data_specials(r"\ndash 693"), r"\ndash 693");
    // …but a special INSIDE an ordinary command's argument is still data.
    assert_eq!(escape_bib_data_specials(r"\emph{AT&T}"), r"\emph{AT\&T}");
  }

  /// url.sty reads `\url`'s argument verbatim, so it is a nested DATA region
  /// and a percent-encoded `%20` inside it must NOT become `\%20`.
  /// `\href`'s SECOND argument is prose and is still escaped.
  #[test]
  fn escape_specials_skips_a_verbatim_argument() {
    assert_eq!(
      escape_bib_data_specials(r"\url{http://x.org/a%20b?c=1&d=2}"),
      r"\url{http://x.org/a%20b?c=1&d=2}"
    );
    assert_eq!(
      escape_bib_data_specials(r"\href{http://x.org/a%20b}{AT&T}"),
      r"\href{http://x.org/a%20b}{AT\&T}"
    );
  }

  // --- recase_title (Perl `\bib@@title` body) ---

  #[test]
  fn recase_asis_is_identity() {
    assert_eq!(
      recase_title("On The Theory Of LATEX", TitleCaseMode::AsIs),
      "On The Theory Of LATEX"
    );
  }

  #[test]
  fn recase_uppercase_is_all_caps() {
    assert_eq!(
      recase_title("Hello World", TitleCaseMode::Uppercase),
      "HELLO WORLD"
    );
  }

  #[test]
  fn recase_lowercase_is_all_lower() {
    assert_eq!(
      recase_title("Hello WORLD", TitleCaseMode::Lowercase),
      "hello world"
    );
  }

  #[test]
  fn recase_capitalize1_caps_first_only() {
    // Perl `capitalize1` calls `ucfirst($word)` on the first word —
    // which uppercases the leading char and leaves the rest of the
    // word untouched (does NOT downcase). The docstring at
    // BibTeX.pool.ltxml:286 ("downcase all, then Capitalize 1st word")
    // mis-describes the implementation. Match Perl-actual, not docs.
    // Subsequent words: `lc($word)` → full lowercase.
    assert_eq!(
      recase_title("ON THE THEORY OF NUMBERS", TitleCaseMode::Capitalize1),
      "ON the theory of numbers"
    );
  }

  #[test]
  fn recase_capitalize_caps_every_word() {
    assert_eq!(
      recase_title("on the theory of numbers", TitleCaseMode::Capitalize),
      "On The Theory Of Numbers"
    );
  }

  #[test]
  fn recase_preserves_braced_groups() {
    // BibTeX convention: `{SomeName}` is opaque — the contents stay
    // verbatim regardless of mode, but the group counts as a word.
    // First word `THE`: `ucfirst` leaves it as `THE` (no downcasing
    // of the rest — see `recase_capitalize1_caps_first_only`).
    // `{LaTeX}` is the braced group (atomic word #2, kept verbatim).
    // `BOOK` is subsequent word #3: lowercased to `book`.
    assert_eq!(
      recase_title("THE {LaTeX} BOOK", TitleCaseMode::Capitalize1),
      "THE {LaTeX} book"
    );
  }

  /// `recase_title` must survive EVERY character width at every scanner
  /// position — the deterministic stand-in for a property test.
  ///
  /// The hazard is small and fully enumerable (4 widths x 6 positions x 5
  /// modes), so a table gives complete coverage of it where random generation
  /// would only sample; and it needs no new dev-dependency, which for this tree
  /// means no addition to the license inventory or `cargo-deny` surface.
  ///
  /// What it is guarding: the scanner used to walk raw BYTE indices, and its
  /// `\<char>` arm advanced a fixed one byte past the backslash — fine for
  /// `\&`, fatal for `\“`, where the cursor landed inside the codepoint and the
  /// next `&title[a..b]` slice PANICKED, aborting the whole document. Witness
  /// 2605.22125 from the 2026-07-26 sandbox sweep. The scanner is now written
  /// on `char_indices` (see `CharCursor`), so the class is gone structurally
  /// rather than patched at the one arm that happened to fire.
  ///
  /// Perl is immune by construction: `\bib@@title` (BibTeX.pool.ltxml
  /// L293-333) works in characters, not bytes.
  #[test]
  fn recase_title_handles_every_character_width_at_every_position() {
    // One representative per UTF-8 encoded length.
    let chars = ['a', 'é', '“', '𝔄'];
    let modes = [
      TitleCaseMode::AsIs,
      TitleCaseMode::Uppercase,
      TitleCaseMode::Lowercase,
      TitleCaseMode::Capitalize,
      TitleCaseMode::Capitalize1,
    ];
    for c in chars {
      // Each position exercises a different scanner arm: after a backslash
      // (the arm that broke), inside a word run, in a brace group, in math,
      // as bare punctuation, and at end-of-input.
      for title in [
        format!("\\{c} tail"),      // `\<char>` escape  <-- the witness shape
        format!("word\\{c}more"),   // escape mid-word
        format!("{{brace {c}}} x"), // inside a braced group
        format!("$math {c}$ x"),    // inside math
        format!("bare {c} text"),   // standalone
        format!("trailing \\{c}"),  // escape at end of input
      ] {
        for mode in modes {
          // Returning at all is the primary assertion: the bug was a panic.
          let out = recase_title(&title, mode);
          // And nothing may be dropped — a "fix" that skips the escape would
          // pass a panic-only check while silently eating the author's text.
          // Checked across case forms, since re-casing is the function's JOB:
          // `Uppercase` turns `a` into `A`, which is not a loss.
          let survives = out.contains(c)
            || c.to_lowercase().all(|lc| out.contains(lc))
            || c.to_uppercase().all(|uc| out.contains(uc));
          assert!(
            survives,
            "recase_title({title:?}, {mode:?}) lost {c:?}: {out:?}"
          );
        }
      }
    }
    // A lone trailing backslash: the `peek_second` guard's edge.
    for mode in modes {
      assert_eq!(recase_title("x \\", mode).matches('\\').count(), 1);
    }
  }

  #[test]
  fn recase_preserves_math_groups() {
    // `$...$` math: copied verbatim. wb stays false after.
    // `PROOF` first word: ucfirst → `PROOF` (no rest-lowercase).
    // `OF` → lowercased.
    assert_eq!(
      recase_title("PROOF OF $\\pi^2/6$", TitleCaseMode::Capitalize1),
      "PROOF of $\\pi^2/6$"
    );
  }

  #[test]
  fn recase_handles_cs_escape_in_word() {
    // `\foo` should be consumed as part of the word run.
    // Lowercase mode → entire word run including the `\foo`
    // becomes lowercase (per Perl's `lc($1)`).
    let r = recase_title("Hello \\TeX World", TitleCaseMode::Lowercase);
    assert_eq!(r, "hello \\tex world");
  }

  #[test]
  fn recase_empty_string() {
    assert_eq!(recase_title("", TitleCaseMode::Capitalize1), "");
  }

  // --- TitleCaseMode::parse ---

  #[test]
  fn title_case_mode_parse_known_values() {
    assert_eq!(TitleCaseMode::parse("asis"), TitleCaseMode::AsIs);
    assert_eq!(TitleCaseMode::parse("uppercase"), TitleCaseMode::Uppercase);
    assert_eq!(TitleCaseMode::parse("lowercase"), TitleCaseMode::Lowercase);
    assert_eq!(
      TitleCaseMode::parse("capitalize"),
      TitleCaseMode::Capitalize
    );
    assert_eq!(
      TitleCaseMode::parse("capitalize1"),
      TitleCaseMode::Capitalize1
    );
  }

  #[test]
  fn title_case_mode_parse_unknown_falls_back_to_capitalize1() {
    // Perl: `LookupValue(...) || 'capitalize1'` — but if a non-empty
    // garbage value is stored, Perl treats it as that string. We
    // treat unknown strings as `capitalize1` to avoid silent
    // misbehavior; matches the documented default per L286.
    assert_eq!(TitleCaseMode::parse(""), TitleCaseMode::Capitalize1);
    assert_eq!(TitleCaseMode::parse("nonsense"), TitleCaseMode::Capitalize1);
  }

  // --- BibEntry::pretty_print (Perl `prettyPrint` for `\bib@@origbibentry`) ---

  #[test]
  fn pretty_print_no_fields_is_empty_braced() {
    let e = BibEntry::new("Solo", "misc");
    // Trailing newline included — Perl `Pre/BibTeX/Entry.pm:64` appends `"}\n"`
    // on every entry, fields or not.
    assert_eq!(e.pretty_print(), "@misc{Solo}\n");
  }

  #[test]
  fn pretty_print_emits_bibtex_source_shape() {
    let mut e = BibEntry::new("Smith2020", "article");
    e.add_raw_field("author", "John Smith");
    e.add_raw_field("title", "On Examples");
    e.add_raw_field("year", "2020");
    let out = e.pretty_print();
    // Order matches insertion order (Vec<(String,String)>). Field names are
    // right-justified to width 10 so the `=` aligns (Perl source-reconstruction
    // shape): author→4sp, title→5sp, year→6sp; entry `}` follows the last value,
    // and the whole thing ends with a newline (Perl `Entry.pm:64` `. "}\n"`).
    assert_eq!(
      out,
      "@article{Smith2020,\n    author = {John Smith},\n     title = {On Examples},\n      year = {2020}}\n"
    );
  }

  #[test]
  fn pretty_print_skips_underscore_internal_fields() {
    // `_raw_keyvals` is a Phase-2-stub-internal field added by the
    // amsrefs `\bib{}{}{}` closure; it shouldn't surface in
    // `\bib@@origbibentry`'s BibTeX-source output.
    let mut e = BibEntry::new("X", "misc");
    e.add_raw_field("_raw_keyvals", "author=John");
    e.add_raw_field("title", "T");
    let out = e.pretty_print();
    assert!(!out.contains("_raw_keyvals"));
    assert!(out.contains("title = {T}"));
  }

  // --- process_identifier (Perl L784) ---

  #[test]
  fn process_identifier_trims_whitespace() {
    assert_eq!(process_identifier("  10.1234/foo  "), "10.1234/foo");
    assert_eq!(process_identifier("\tabc\n"), "abc");
    assert_eq!(process_identifier("no-whitespace"), "no-whitespace");
    assert_eq!(process_identifier(""), "");
  }

  // --- parse_amsrefs_keyvals ---

  #[test]
  fn parse_kv_simple_pairs() {
    let r = parse_amsrefs_keyvals("author = {Smith}, year = 2020");
    assert_eq!(r, vec![
      ("author".to_string(), "Smith".to_string()),
      ("year".to_string(), "2020".to_string()),
    ]);
  }

  #[test]
  fn parse_kv_lowercases_keys() {
    let r = parse_amsrefs_keyvals("Author={S}, TITLE={T}");
    assert_eq!(r[0].0, "author");
    assert_eq!(r[1].0, "title");
  }

  #[test]
  fn parse_kv_preserves_braced_commas() {
    // Internal commas in `{...}` shouldn't split the value.
    let r = parse_amsrefs_keyvals("author = {Smith, John and Doe, Jane}, year=2020");
    assert_eq!(r.len(), 2);
    assert_eq!(r[0].1, "Smith, John and Doe, Jane");
    assert_eq!(r[1].1, "2020");
  }

  #[test]
  fn parse_kv_nested_braces() {
    let r = parse_amsrefs_keyvals("title = {On {Foo} bar}");
    assert_eq!(r, vec![("title".to_string(), "On {Foo} bar".to_string())]);
  }

  #[test]
  fn parse_kv_unbraced_value() {
    let r = parse_amsrefs_keyvals("year = 1999, volume = 12");
    assert_eq!(r, vec![
      ("year".to_string(), "1999".to_string()),
      ("volume".to_string(), "12".to_string()),
    ]);
  }

  #[test]
  fn parse_kv_empty_input() {
    assert!(parse_amsrefs_keyvals("").is_empty());
    assert!(parse_amsrefs_keyvals("   ").is_empty());
    assert!(parse_amsrefs_keyvals(",,, ,").is_empty());
  }

  #[test]
  fn parse_kv_key_without_value() {
    let r = parse_amsrefs_keyvals("draft, year=2020");
    assert_eq!(r, vec![
      ("draft".to_string(), String::new()),
      ("year".to_string(), "2020".to_string()),
    ]);
  }
}

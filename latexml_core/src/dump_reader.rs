//! Reader for Rust-native kernel dump files (produced by dump_writer.rs).
//!
//! Loads a dump file produced by `latexml_oxide --init=latex.ltx --dest=dump`
//! and replays the state assignments into the engine.
//!
//! **Loading policy:** `M` and `V` entries replay with Perl-style global
//! assignment semantics, matching `Core/Dumper.pm`'s `I()` / `V()` helpers.
//! Runtime-state filters below are narrow exceptions for entries that should
//! never be useful from a format dump.
//!
//! Format: tab-separated lines:
//!   V\tKEY\tTYPE\tDATA             — value assignment
//!   M\tKEY\tE\tCS\tNARGS\tFLAGS\tTOKENS — Expandable definition
//!   M\tKEY\tN                       — None meaning (undefined)
//!   M\tKEY\tT\tCC:TEXT              — Token meaning (let-assignment)
//!   C\tCHAR\tCC\tVALUE             — catcode assignment
//!   LC\tCHAR\tCH\tVALUE            — lccode assignment
//!   UC\tCHAR\tCH\tVALUE            — uccode assignment
//!   SC\tCHAR\tCH\tVALUE            — sfcode assignment
//!   DC\tCHAR\tCH\tVALUE            — delcode assignment
//!   MC\tCHAR\tCH\tVALUE            — mathcode assignment

use std::path::Path;

use crate::{
  common::{arena, numeric_ops::NumericOps, store::Stored},
  definition::expandable::{Expandable, ExpandableOptions},
  state::{self, Scope, TableName},
  token::{Catcode, Token},
  tokens::Tokens,
};

/// Load a Rust-native dump file into the current State.
/// Returns the number of entries loaded.
pub fn load_native_dump(path: &Path) -> Result<usize, String> {
  let content = std::fs::read_to_string(path)
    .map_err(|e| format!("Failed to read dump file {}: {}", path.display(), e))?;
  let count = load_from_str_internal(&content, &path.display().to_string())?;
  Ok(count)
}

/// Load dump data from a string (used by the embedded LaTeX kernel dump
/// module — `latexml_engine/src/latex_dump.rs`).
/// Returns the number of entries loaded.
pub fn load_from_str(content: &str) -> Result<usize, String> {
  // Labelled `<embedded:latex>` (mirroring `<embedded:plain>`) so the
  // "Loaded N entries from ..." info line names which dump was loaded.
  load_from_str_internal(content, "<embedded:latex>")
}

/// Backwards-compat alias kept until call sites are migrated. Both
/// entry points now load unconditionally, mirroring Perl `I(...)` /
/// `V(...)` (Core/Dumper.pm) which call `assign_internal('global')`
/// without filters.
pub fn load_from_str_plain(content: &str) -> Result<usize, String> {
  load_from_str_internal(content, "<embedded:plain>")
}

/// Load a dump, naming `source` (a real path or an `<embedded TLyyyy>` label) in
/// the single `dump_reader:loaded` Info line. The dump wrappers
/// (`plain_dump`/`latex_dump`) use this so there is exactly ONE "loaded N
/// entries from <source>" message per dump — they no longer emit a second,
/// redundant `*_dump:loaded` line of their own.
pub fn load_from_str_labeled(content: &str, source: &str) -> Result<usize, String> {
  load_from_str_internal(content, source)
}

// Per-load context used to attach a nominal Locator to dump-installed
// Expandables. Matches Perl #aaacdba2 (2026): dump-loaded definitions
// should be traceable to the dump file + line, not report the arena's
// internal location. Thread-local so concurrent loads (there are none
// today, but the state is cooperative) don't clobber each other.
thread_local! {
  static CURRENT_LOAD_CTX: std::cell::Cell<Option<(arena::SymStr, u32)>> =
    const { std::cell::Cell::new(None) };
  /// PA/MPA aliases whose target wasn't defined at dump-load time.
  /// Populated by `load_meaning`'s PA arm, drained by
  /// `flush_deferred_aliases()` after `_constructs` finishes.
  static DEFERRED_ALIASES: std::cell::RefCell<Vec<(Token, Token)>> =
    const { std::cell::RefCell::new(Vec::new()) };
}

/// Replay any PA/MPA aliases that were deferred during dump load
/// because their target was not yet defined. Call once after the
/// post-dump definition pass (`_constructs`) has loaded.
/// Returns `(applied, skipped)`.
pub fn flush_deferred_aliases() -> (usize, usize) {
  let pending: Vec<(Token, Token)> =
    DEFERRED_ALIASES.with(|cell| std::mem::take(&mut *cell.borrow_mut()));
  let mut applied = 0usize;
  let mut skipped = 0usize;
  for (cs_tok, target_tok) in pending {
    // Target still undefined — the alias's target must be defined
    // in some source we never load (e.g. expl3 intarrays that the
    // short-circuit skips). Leave the key undefined; the engine's
    // undefined-CS handler will cope at runtime.
    if !state::has_meaning(&target_tok) {
      skipped += 1;
      continue;
    }
    // Perl `Lt()` parity: look up target's meaning, write it at
    // alias key via `assign_internal('meaning', ..., 'global')`.
    match state::lookup_meaning(&target_tok) {
      Some(meaning) => {
        state::assign_internal(
          TableName::Meaning,
          cs_tok.get_cs_name(),
          meaning,
          Some(Scope::Global),
        );
        applied += 1;
      },
      _ => {
        skipped += 1;
      },
    }
  }
  (applied, skipped)
}

fn current_dump_locator() -> crate::common::locator::Locator {
  if let Some((source, lineno)) = CURRENT_LOAD_CTX.with(|c| c.get()) {
    crate::common::locator::Locator {
      source,
      from_line: lineno,
      to_line: lineno,
      from_column: 1,
      to_column: 1,
    }
  } else {
    crate::common::locator::Locator::default()
  }
}

fn load_from_str_internal(content: &str, source_name: &str) -> Result<usize, String> {
  let mut count = 0;
  let mut skipped = 0;
  let mut errors = 0;
  let source_sym = arena::pin(source_name);

  for (lineno, line) in content.lines().enumerate() {
    // Trim only CR (from CRLF line endings); `lines()` already strips LF.
    // Do NOT use `trim()` here — it strips trailing tabs, which are part of
    // the tab-separated format for entries with empty trailing fields (e.g.
    // E-entries with empty body: `E\t<cs>\t<nargs>\t<flags>\t`).
    let line = line.trim_end_matches('\r');
    if line.is_empty() || line.starts_with('#') {
      continue;
    }

    CURRENT_LOAD_CTX.with(|c| c.set(Some((source_sym, (lineno + 1) as u32))));

    match parse_and_load(line) {
      Ok(true) => count += 1,
      Ok(false) => skipped += 1,
      Err(e) => {
        errors += 1;
        if errors <= 10 {
          Warn!(
            "dump_reader",
            "line",
            s!(
              "Line {}: {}: {}",
              lineno + 1,
              e,
              &line[..line.len().min(80)]
            )
          );
        }
      },
    }
  }

  if errors > 10 {
    Warn!(
      "dump_reader",
      "errors",
      s!("... and {} more errors", errors - 10)
    );
  }
  Info!(
    "dump_reader",
    "loaded",
    s!(
      "Loaded {} entries from {} ({} skipped, {} errors)",
      count,
      source_name,
      skipped,
      errors
    )
  );

  Ok(count)
}

/// Parse a single dump line and load it. Returns Ok(true) if loaded,
/// Ok(false) if filtered (e.g. corrupt MC/DC), Err on parse error.
fn parse_and_load(line: &str) -> Result<bool, String> {
  // Direct splitn iteration — saves the per-line Vec<&str> allocation
  // that splitn(3).collect() was doing × 110k dump entries.
  let mut it = line.splitn(3, '\t');
  let table = it.next().ok_or("Too few fields")?;
  let raw_key = match it.next() {
    Some(k) => k,
    None => return Err("Too few fields".into()),
  };
  // Key decode: Cow borrows the original &str when no `%` escape is
  // present (the overwhelming majority). Saves a per-line allocation
  // for the ~25k dump entries that have plain CS-name keys.
  let key_cow: std::borrow::Cow<'_, str> = if raw_key.contains('%') {
    std::borrow::Cow::Owned(url_decode(raw_key))
  } else {
    std::borrow::Cow::Borrowed(raw_key)
  };
  let key = key_cow.as_ref();
  let data = it.next().unwrap_or("");

  match table {
    // V: Value entries (registers, fontdimen, font metadata).
    // Add-only policy: only loads if key has no existing value.
    //
    // Skip MAX_ERRORS: it was set to 1_000_000 in `ini_tex.rs` during
    // dump-build (to let raw latex.ltx run through transient errors)
    // and got captured into the dump. Loading that into a regular
    // conversion lets runaway error cascades (e.g. AmS-TeX `\cases`
    // mis-parse → 1M `\hbox`/`&` errors per paper) bypass the 10000
    // default cap. Filter at read time so existing dumps are clean.
    "V" if key == "MAX_ERRORS" => Ok(false),
    "V" => load_value(key, data),
    // IA: consolidated expl3 intarray (one record per (font, size); dump_writer
    // collapses ~17k V-records into one IA). Body is `<len>\t<rle>` where rle
    // is a comma-list of `v` or `v*n` runs. Expansion assigns the same V
    // entries that the per-slot records would have, so the runtime state
    // post-replay is identical.
    "IA" => load_intarray(key, data),
    // M: Meaning entries (Expandable, Let-alias, Register, etc.).
    //
    // Perl-faithful: `plain_dump.pool.ltxml` and `latex_dump.pool.ltxml`
    // emit one `I(...)` per Meaning entry, which is
    // `assign_internal($STATE, 'meaning', $cs, $def, 'global')` —
    // unconditional global write. No admission gate, no skip-if-defined.
    // Match it: route every M entry to `load_meaning` directly.
    "M" => load_meaning(key, data),
    // LC/UC: case-mapping codes — safe, always load
    "LC" => load_lccode(key, data),
    "UC" => load_uccode(key, data),
    // SC: space factor codes — safe, always load
    "SC" => load_sfcode(key, data),
    // C: catcodes — only for non-ASCII (>127). ASCII catcodes are set by
    // the engine; loading from dump would conflict.
    "C" => {
      let ch = decode_char_key(key);
      if ch.is_some_and(|c| c as u32 > 127) {
        load_catcode(key, data)
      } else {
        Ok(false)
      }
    },
    // Perl `Core/Dumper.pm:dump_mathcode/dump_delcode` write MC/DC for
    // every state-set entry; the matching reader is unconditional apply
    // (CLAUDE.md "Unconditional dump apply"). plain.tex / latex.ltx need
    // letter mathcodes and `\delcode\(="0028300` etc. replayed from dump
    // so `\cal abc` (cmsy fam 2) and delimited symbols decode correctly
    // — without this, `decode_math_char` never fires for letters in the
    // dump path and they get default-decoded to ASCII (no meaning/role).
    "MC" => load_mathcode(key, data),
    "DC" => load_delcode(key, data),
    _ => Ok(false),
  }
}

/// V entries to unconditionally skip (runtime state, never useful from dump).
const SKIP_VALUE_KEYS: &[&str] = &[
  "INTERPRETING_DEFINITIONS",
  "if_count",
  "absorb_count",
  "if_stack",
  "INCLUDE_COMMENTS",
  "INCLUDE_STYLES",
  "INPUT_ENCODING",
  "CURRENT_INPUT_ENCODING",
  "SUPPRESS_UNEXPECTED_ERRORS",
  "SUPPRESS_UNDEFINED_ERRORS",
  // Upstream Perl IGNORED_SYMBOLS (TeX_Job.pool.ltxml): runtime-only
  // tables that re-populate from the engine — can't meaningfully round-
  // trip through the dump.
  "DOCUMENT_REWRITE_RULES",
  "PARAMETER_TYPES",
  "TAG_PROPERTIES",
  "MATH_LIGATURES",
  "TEXT_LIGATURES",
];

/// V entry key prefixes to skip.
const SKIP_VALUE_PREFIXES: &[&str] = &["input_file:", "output_file:", "texsys"];

/// V entry key substrings to skip.
///
/// Note: `_loaded` / `_raw_loaded` flags are present in the dump (correctly,
/// since `--init=latex.ltx` sees expl3-code.tex, hyphenation patterns, and
/// hundreds of other raw-loaded files). But carrying them through into state
/// at dump-load time blows up in practice:
///
///  - Hyphenation `loadhyph-*.tex_loaded` flags make subsequent raw-loading of babel's language.def
///    skip files that set `\l@<lang>` registers our engine then discovers aren't present,
///    triggering a flood of error recovery that can consume gigabytes of RAM.
///  - `expl3.ltx_loaded=1` plus `expl3.sty_loaded=` NOT being set means `\usepackage{expl3}`
///    doesn't short-circuit AT the .sty layer, but the raw .ltx re-load now enters a stranger code
///    path with partial flags.
///
/// The proper fix, tracked as the "dump/_base mutual-exclusivity" item in
/// SYNC_STATUS D0, is to have exactly ONE loading path (dump-cache or raw-load)
/// active at a time, mirroring Perl's `LoadFormat` branching. Until that lands,
/// keep the skip list conservative so mixed paths don't trigger recovery loops.
const SKIP_VALUE_CONTAINS: &[&str] = &[
  "_loaded", /* Package loading flags — see doc comment above.
             * Substring also matches `_raw_loaded` (OXIDIZED_DESIGN #23). */
];

/// Load a value entry: V\tKEY\tTYPE\tDATA
///
/// Uses add-only policy: only loads if the key does not already have a value.
/// This ensures compiled engine state takes priority over dump state.
fn load_value(key: &str, data: &str) -> Result<bool, String> {
  // Skip unconditional keys
  for skip in SKIP_VALUE_KEYS {
    if key == *skip {
      return Ok(false);
    }
  }
  // Skip by prefix
  for prefix in SKIP_VALUE_PREFIXES {
    if key.starts_with(prefix) {
      return Ok(false);
    }
  }
  // Skip by substring.
  for substr in SKIP_VALUE_CONTAINS {
    if key.contains(substr) {
      return Ok(false);
    }
  }

  // Perl `V()` parity (`Core/Dumper.pm` L59): every dumped Value entry
  // maps to `assign_internal($STATE, 'value', $key, $val, 'global')` —
  // unconditional global write. No skip-if-defined.

  // Avoid the per-line Vec<&str> allocation — direct iter destructure
  // matches the pattern used in load_meaning and parse_and_load.
  let mut top_it = data.splitn(2, '\t');
  let kind = top_it.next().ok_or("Missing value type")?;
  let rest = top_it.next().unwrap_or("");
  // Helper to default to "0" for numeric parses (the prior code used
  // `rest_or_zero.parse()`).
  let rest_or_zero = if rest.is_empty() { "0" } else { rest };

  let value = match kind {
    "N" => return Ok(false), // Don't load None values (would erase existing)
    "B" => Stored::Bool(rest == "1"),
    "I" => {
      let n: i64 = rest_or_zero
        .parse()
        .map_err(|e| format!("Bad int: {}", e))?;
      Stored::Int(n)
    },
    // "Nm": Stored::Number marker (distinct from "I" Stored::Int) —
    // see dump_writer's Number serializer for rationale.
    "Nm" => {
      let n: i64 = rest_or_zero
        .parse()
        .map_err(|e| format!("Bad number: {}", e))?;
      Stored::Number(crate::common::number::Number(n))
    },
    "S" => Stored::from(url_decode(rest)),
    "F" => {
      // Stored::Font — written by dump_writer's Stored::Font arm.
      // Format: F\tname=...\x1fsize=...\x1ffamily=...\x1f...
      // Each unit-separator-delimited segment is `key=urlencoded_value`.
      // Mirrors Perl `dump_font` (Core/Dumper.pm L281-284).
      use std::borrow::Cow;

      use crate::common::font::Font;
      let mut font = Font::default();
      for kv in rest.split('\x1f') {
        if let Some((k, v)) = kv.split_once('=') {
          let v_dec = url_decode(v);
          match k {
            "name" => font.name = Some(Cow::Owned(v_dec)),
            "family" => font.family = Some(Cow::Owned(v_dec)),
            "series" => font.series = Some(Cow::Owned(v_dec)),
            "shape" => font.shape = Some(Cow::Owned(v_dec)),
            "encoding" => font.encoding = Some(Cow::Owned(v_dec)),
            "language" => font.language = Some(Cow::Owned(v_dec)),
            "mathstyle" => font.mathstyle = Some(Cow::Owned(v_dec)),
            "opacity" => font.opacity = Some(Cow::Owned(v_dec)),
            "size" => font.size = v_dec.parse().ok(),
            "scale" => font.scale = v_dec.parse().ok(),
            "emph" => font.emph = Some(v_dec == "1"),
            "scripted" => font.scripted = Some(v_dec == "1"),
            "mathstylestep" => font.mathstylestep = v_dec.parse().ok(),
            "flags" => font.flags = v_dec.parse().ok(),
            _ => {},
          }
        }
      }
      Stored::Font(std::rc::Rc::new(font))
    },
    "CH" => {
      let n: u16 = rest_or_zero
        .parse()
        .map_err(|e| format!("Bad charcode: {}", e))?;
      Stored::Charcode(n)
    },
    "CC" => {
      let n: u8 = rest_or_zero
        .parse()
        .map_err(|e| format!("Bad catcode: {}", e))?;
      Stored::Catcode(Catcode::from(n))
    },
    "T" => {
      let tok = parse_token(rest)?;
      Stored::Token(tok)
    },
    "TK" => {
      let toks = parse_token_list(rest)?;
      Stored::Tokens(Tokens::from(toks))
    },
    "D" => {
      let n: i64 = rest_or_zero
        .parse()
        .map_err(|e| format!("Bad dimension: {}", e))?;
      Stored::Dimension(crate::common::dimension::Dimension(n))
    },
    "G" => Stored::Glue(parse_glue(rest_or_zero)?),
    "MD" => {
      let n: i64 = rest_or_zero
        .parse()
        .map_err(|e| format!("Bad mudimension: {}", e))?;
      Stored::MuDimension(crate::common::mudimension::MuDimension(n))
    },
    "MG" => Stored::MuGlue(parse_muglue(rest_or_zero)?),
    "VD" => return Ok(false), // Don't load empty VecDeque (runtime state)
    _ => return Ok(false),    // Unknown value type
  };

  // Perl `V()` (`Core/Dumper.pm` L59):
  //   sub V { State::assign_internal($STATE,'value',$_[0],$_[1],'global'); }
  // Direct table mutation, no dialect.
  state::assign_internal(
    TableName::Value,
    arena::pin(key),
    value,
    Some(Scope::Global),
  );
  Ok(true)
}

/// Expand an `IA` (intarray) record into the per-slot Dimension V entries
/// that the runtime expects. Format: key = `<prefix>` (e.g.
/// `fontdimen_fontinfo_cmr10 at 15sp`), data = `<len>\t<rle>`. RLE tokens
/// are comma-separated; each is either `<v>` (one entry) or `<v>x<n>`
/// (n consecutive entries of value v). Slots are written at indices
/// 1..=len. Mismatched RLE-length vs declared len is an error.
fn load_intarray(key: &str, data: &str) -> Result<bool, String> {
  let mut it = data.splitn(2, '\t');
  let len_s = it.next().unwrap_or("");
  let rle = it.next().unwrap_or("");
  let len: usize = len_s.parse().map_err(|e| format!("Bad IA length: {}", e))?;
  let values = rle_decode_i64(rle)?;
  if values.len() != len {
    return Err(format!(
      "IA length mismatch for {:?}: declared {} but RLE decoded to {}",
      key,
      len,
      values.len()
    ));
  }
  for (i, val) in values.into_iter().enumerate() {
    let slot_key = format!("{}_{}", key, i + 1);
    state::assign_internal(
      TableName::Value,
      arena::pin(&slot_key),
      Stored::Dimension(crate::common::dimension::Dimension(val)),
      Some(Scope::Global),
    );
  }
  Ok(true)
}

/// Inverse of `dump_writer::rle_encode_i64`. Parses a comma-separated
/// list of tokens, each `v` (single) or `vxn` (n copies of v). Empty
/// input decodes to an empty vector.
fn rle_decode_i64(s: &str) -> Result<Vec<i64>, String> {
  let mut out = Vec::new();
  if s.is_empty() {
    return Ok(out);
  }
  for tok in s.split(',') {
    if let Some(xi) = tok.find('x') {
      let val: i64 = tok[..xi]
        .parse()
        .map_err(|e| format!("Bad RLE value in {:?}: {}", tok, e))?;
      let cnt: usize = tok[xi + 1..]
        .parse()
        .map_err(|e| format!("Bad RLE count in {:?}: {}", tok, e))?;
      for _ in 0..cnt {
        out.push(val);
      }
    } else {
      let val: i64 = tok
        .parse()
        .map_err(|e| format!("Bad RLE value {:?}: {}", tok, e))?;
      out.push(val);
    }
  }
  Ok(out)
}

/// Load a meaning entry: M\tKEY\tTYPE\t...
///
/// Uses add-only policy: skip if the CS already has a meaning.
/// Additionally, only loads "safe" definitions — those that won't interfere
/// with the compiled engine's processing during normal LaTeX operation:
/// - expl3 internals (contain `:`) — safe because `:` is OTHER under normal catcodes
/// - Private LaTeX internals (contain `@`) — only invoked by other macros
/// - Skip all "public" macros that could be invoked during normal expansion and might reference
///   hooks/primitives not supported by our engine
fn load_meaning(key: &str, data: &str) -> Result<bool, String> {
  let cs_tok = Token {
    text: arena::pin(key),
    code: Catcode::CS,
    #[cfg(feature = "token-locators")]
    loc: 0,
  };

  // Perl `I(...)` parity (`Core/Dumper.pm` L67): every dumped Meaning
  // entry maps to `assign_internal($STATE, 'meaning', $cs, $def,
  // 'global')` — unconditional global write. No skip-if-defined, no
  // admission filter.

  // Avoid the per-line Vec<&str> allocation — this fn runs ~80k times
  // during latex.dump load (every M entry).
  let mut top_it = data.splitn(2, '\t');
  let kind = top_it.next().ok_or("Missing meaning type")?;
  let rest = top_it.next().unwrap_or("");

  match kind {
    "N" => {
      // None meaning — skip (don't define as undefined)
      Ok(false)
    },
    "E" => {
      // Expandable: E\tCSNAME\tNARGS\tFLAGS\tTOKENS[\tPROTO[\tV3_PARAMS]]
      //
      // Three historical shapes, read in fallback order:
      //   v3 — 6th field present: structured Parameter records; bypasses
      //        parse_parameters entirely. Only format that round-trips
      //        Until:/Match: with catcoded delimiter tokens intact.
      //   v2 — 5th field present: url-decoded prototype string fed to
      //        parse_parameters. Good for {} / [] / DefToken / simple
      //        typed params; loses brace-in-delimiter forms.
      //   v1 — nargs only: "{}".repeat(nargs), all params flattened to
      //        Plain. Kept as last resort so ancient dumps still load.
      // Direct iter destructure — saves the Vec<&str>::collect()
      // allocation × ~80k E entries.
      let mut eit = rest.splitn(6, '\t');
      let alias_field = eit.next().ok_or("Incomplete Expandable entry")?;
      let nargs_field = eit.next().ok_or("Incomplete Expandable entry")?;
      let flags_field = eit.next().ok_or("Incomplete Expandable entry")?;
      let tok_field = eit.next().ok_or("Incomplete Expandable entry")?;
      let proto_field = eit.next();
      let v3_field = eit.next();

      // eparts[0] is the alias-cs from the dump (Perl-side: the cs of
      // the Definition object that this entry was let-aliased from).
      //
      // We propagate the alias ONLY when the target is a known deferred
      // command (`\unexpanded`, `\the`, `\detokenize`, `\showthe`) — that
      // narrow case is what makes `\exp_not:n {…}` inside `\edef` bodies
      // correctly skip re-expansion (Perl `Gullet.pm:505`'s DEFERRED
      // path), preserving `\__seq_item:n {…}` inside `\seq_gpush:Nn`'s
      // `\unexpanded`-wrapped body. Without this, the seq stack stays
      // empty after push, leading to `extra-pop-label` and the
      // `\q_no_value` recursion cascade during `\@pushfilename`.
      //
      // We DON'T propagate alias for the ~1k other Lt-aliased entries
      // (e.g. `\bool_if_exist:NTF` → `\cs_if_exist:NTF`) — those would
      // change `defn.get_cs_name()`'s return value, which feeds into
      // many lookup paths and triggers infinite-loop regressions in
      // `\@nil` handling, etc. Keep blast radius tight.
      const DEFERRED_NAMES: &[&str] = &["\\unexpanded", "\\the", "\\detokenize", "\\showthe"];
      let alias_decoded = url_decode(alias_field);
      let is_alias_diff = cs_tok.with_cs_name(|s| s != alias_decoded.as_str());
      let alias_for_traits = if is_alias_diff && DEFERRED_NAMES.contains(&alias_decoded.as_str()) {
        Some(alias_decoded)
      } else {
        None
      };
      let nargs: usize = nargs_field.parse().unwrap_or(0);
      let flags = flags_field;
      let tok_data = tok_field;
      let proto_opt = proto_field.map(url_decode).filter(|s| !s.is_empty());
      let v3_opt = v3_field.filter(|s| !s.is_empty());

      let is_long = flags.contains('L');
      let is_protected = flags.contains('P');

      let expansion = parse_token_list(tok_data)?;

      // Build parameter spec, preferring v3 structured → v2 proto →
      // v1 nargs-repeat fallback. init_flag=true for both fallbacks:
      // state is live at runtime so Parameter::init() can resolve
      // readers via PARAMETER_TYPES.
      let paramlist = if let Some(v3) = v3_opt {
        match parse_parameters_v3(v3) {
          Ok(pl) => pl,
          Err(_) => proto_opt
            .as_ref()
            .and_then(|p| crate::common::def_parser::parse_parameters(p, &cs_tok, true).ok())
            .flatten()
            .or_else(|| {
              if nargs > 0 {
                let fallback = "{}".repeat(nargs);
                crate::common::def_parser::parse_parameters(&fallback, &cs_tok, true)
                  .ok()
                  .flatten()
              } else {
                None
              }
            }),
        }
      } else {
        // v2 path: no v3 field, fall back to proto-parsing (with the
        // original silent-degrade-to-nargs behavior).
        match proto_opt {
          Some(proto) => match crate::common::def_parser::parse_parameters(&proto, &cs_tok, true) {
            Ok(pl) => pl,
            Err(_) if nargs > 0 => {
              let fallback = "{}".repeat(nargs);
              crate::common::def_parser::parse_parameters(&fallback, &cs_tok, true)
                .map_err(|e| format!("Param parse fallback: {}", e))?
            },
            Err(_) => None,
          },
          None if nargs > 0 => {
            let proto = "{}".repeat(nargs);
            crate::common::def_parser::parse_parameters(&proto, &cs_tok, true)
              .map_err(|e| format!("Param parse: {}", e))?
          },
          None => None,
        }
      };

      let options = Some(ExpandableOptions {
        long: is_long,
        protected: is_protected,
        nopack_parameters: true, // tokens already have ARG catcode
        alias: alias_for_traits,
        ..ExpandableOptions::default()
      });

      let expansion_body = Tokens::from(expansion).into();
      match Expandable::new(cs_tok, paramlist, Some(expansion_body), options) {
        Ok(mut exp) => {
          // Perl #aaacdba2: stamp dump-loaded definitions with a
          // nominal Locator pointing at the dump file + line. Helps
          // diagnostics attribute errors to the dump source rather
          // than the arena's compile-site default.
          exp.locator = current_dump_locator();
          // Perl `I()` (`Core/Dumper.pm` L67):
          //   sub I { State::assign_internal($STATE,'meaning',
          //           $_[0]->getCSName, $_[0], 'global'); }
          // Direct table mutation — no `:locked` gate, no add-only.
          // CONFIRMED via probe (2026-04-27): Perl dump load bypasses
          // the :locked gate. `installDefinition` (State.pm L502-517)
          // checks :locked and refuses; `assign_internal` (State.pm
          // L140) does not. Dumper's `I` shorthand calls `assign_internal`
          // directly, so locked defs ARE silently overwritten by dump.
          // Rust matches: this code calls `state::assign_internal`, not
          // `install_definition`. Verified `\hidewidth` and `\leavevmode`
          // get overwritten by dump entries despite earlier bootstrap defs.
          state::assign_internal(
            TableName::Meaning,
            cs_tok.get_cs_name(),
            Stored::from(exp),
            Some(Scope::Global),
          );
          Ok(true)
        },
        Err(e) => Err(format!("Expandable creation failed: {}", e)),
      }
    },
    "T" => {
      // Token meaning. Perl `Im()` (`Core/Dumper.pm` L66):
      //   sub Im { State::assign_internal($STATE,'meaning',
      //            $_[0], $_[1], 'global'); }
      // Direct write — no `\let`-chase, no chain follow.
      let tok = parse_token(rest)?;
      state::assign_internal(
        TableName::Meaning,
        cs_tok.get_cs_name(),
        Stored::Token(tok),
        Some(Scope::Global),
      );
      Ok(true)
    },
    "FD" => {
      // FontDef: `FD\t<font_id>` — Perl `dump_primitive` (Core/Dumper.pm L383-389)
      // emits this for `\font`-defined primitives. Install a Primitive whose
      // `before_digest` mirrors `LaTeXML::Core::Definition::FontDef::invoke`
      // (FontDef.pm L38-45):
      //   1. lookup the fontinfo hash at <font_id>
      //   2. assignValue(current_FontDef => $cs)
      //   3. merge the font into $STATE->lookupValue('font')
      // The fontinfo Stored::Font rides through the dump as a `V` entry with
      // `F\t...` payload (see Stored::Font arm in dump_writer + the `F` arm
      // in parse_value above).
      use crate::definition::{BeforeDigestClosure, primitive::Primitive};
      let font_id_raw = url_decode(rest);
      let font_id_pin = arena::pin(&font_id_raw);
      let font_id_str = font_id_raw;
      let cs_for_fontdef = cs_tok;
      let merge_closure: BeforeDigestClosure = std::rc::Rc::new(move || {
        state::assign_value("current_FontDef", Stored::Token(cs_for_fontdef), None);
        if let Some(Stored::Font(f)) = state::lookup_value(&font_id_str) {
          crate::binding::content::merge_font((*f).clone());
        }
        Ok(Vec::new())
      });
      let prim = Primitive {
        cs: cs_tok,
        before_digest: vec![merge_closure],
        font_id: Some(font_id_pin),
        ..Primitive::default()
      };
      state::assign_internal(
        TableName::Meaning,
        cs_tok.get_cs_name(),
        Stored::Primitive(std::rc::Rc::new(prim)),
        Some(Scope::Global),
      );
      Ok(true)
    },
    "PA" | "MPA" => {
      // Primitive alias: PA\t<target_cs> — the entry's meaning is an
      // Rc<Primitive> whose own cs is <target_cs>. If <target_cs> == key
      // this is the "primary" entry (already provided by compiled bindings
      // in _base.rs etc.); skip. Otherwise, replay `\let <key> <target>`
      // so the Rc<Primitive> is shared just as it was when the dump was
      // generated. This is how \tex_let:D, \tex_def:D, etc. survive the
      // dump — without this the expl3.sty short-circuit guard
      // `\ifx\csname tex_let:D\endcsname\relax` never fires and the 36k
      // lines of expl3-code.tex get reprocessed on every run.
      let target_cs_raw = url_decode(rest);
      if target_cs_raw == key {
        return Ok(false);
      }
      let target_tok = Token {
        text: arena::pin(&target_cs_raw),
        code: Catcode::CS,
        #[cfg(feature = "token-locators")]
        loc: 0,
      };
      // Perl `Lt()` (`Core/Dumper.pm` L69-72):
      //   sub Lt { my $d = State::lookupDefinition($STATE, T_CS($_[1]));
      //            State::assign_internal($STATE,'meaning',$_[0],$d,'global'); }
      // Look up the target's current Meaning entry, then writes that
      // very Stored value at the alias key. Sharing the Rc preserves
      // identity (e.g. \let\tex_let:D\let keeps the same Primitive Rc).
      //
      // If the target is not yet defined (load order has _constructs
      // running after the dump for some let-aliases — e.g.
      // `\let\a=\@tabacckludge`), defer until flush_deferred_aliases().
      if !state::has_meaning(&target_tok) {
        DEFERRED_ALIASES.with(|cell| {
          cell.borrow_mut().push((cs_tok, target_tok));
        });
        return Ok(false);
      }
      let target_meaning = state::lookup_meaning(&target_tok);
      if let Some(meaning) = target_meaning {
        state::assign_internal(
          TableName::Meaning,
          cs_tok.get_cs_name(),
          meaning,
          Some(Scope::Global),
        );
      }
      Ok(true)
    },
    "R" => {
      // Register: R\tCS\tTYPE\tVALUE[\tMATHGLYPH][\tADDRESS]
      // rparts[0] (internal CS name) is redundant with the outer key —
      // same reasoning as the E arm; we skip the decode + alloc.
      // ADDRESS field is a url-encoded address-slot key for allocated
      // registers (Perl `\newcount\m@ne` → address='\count22'). When
      // absent, address defaults to the CS name. Without this, dump_reader
      // wrote `\m@ne`'s -1 to its CS-name slot, but `\m@ne`'s actual
      // address (`\count22`) held the default 0 — `\settabs 20\columns`
      // looped infinitely because `\m@ne == 0` never advanced `\count@`
      // toward 0 in `\loop\ifnum\count@>\z@\@nother\repeat`.
      // Direct iter destructure, mirroring the E-branch pattern.
      let mut rit = rest.splitn(5, '\t');
      let r_cs = rit.next().ok_or("Incomplete Register entry")?;
      let r_type = rit.next().ok_or("Incomplete Register entry")?;
      let r_value = rit.next().ok_or("Incomplete Register entry")?;
      let r_glyph_field = rit.next();
      let r_addr_field = rit.next();
      let rtype = r_type;
      let value_str = r_value;
      let mathglyph = r_glyph_field
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<u32>().ok())
        .and_then(char::from_u32);
      let dump_address: Option<String> = r_addr_field.filter(|s| !s.is_empty()).map(url_decode);
      // For register-aliases (M-line key != register's internal cs), the
      // storage slot lives at the cs name, not the alias key. e.g.
      //   M  \tex_endlinechar:D  R  \endlinechar  N  0
      // means "\tex_endlinechar:D" is meaning-installed but the underlying
      // register storage is at "\endlinechar". Without this, assignments
      // through the alias (\tex_endlinechar:D = 32) write to a separate
      // slot and the real \endlinechar stays unchanged — breaking
      // \ExplSyntaxOn's `\tex_endlinechar:D = 32 \scan_stop:` line, which
      // in turn breaks the entire dump-path expl3 whitespace handling
      // (8 expl3 tests). Mirror Perl's address-via-internal-cs semantics.
      let internal_cs_decoded = url_decode(r_cs);
      let dump_address: Option<String> = dump_address.or_else(|| {
        if internal_cs_decoded != *key && !internal_cs_decoded.is_empty() {
          Some(internal_cs_decoded)
        } else {
          None
        }
      });

      use crate::{
        common::number::Number,
        definition::register::{Register, RegisterType, RegisterValue},
      };

      let (reg_type, reg_value) = match rtype {
        "N" | "CD" => {
          let n: i64 = value_str.parse().unwrap_or(0);
          let rt = if rtype == "CD" {
            RegisterType::CharDef
          } else {
            RegisterType::Number
          };
          (rt, Some(RegisterValue::Number(Number::new(n))))
        },
        "D" => {
          let n: i64 = value_str.parse().unwrap_or(0);
          (
            RegisterType::Dimension,
            Some(RegisterValue::Dimension(
              crate::common::dimension::Dimension(n),
            )),
          )
        },
        "G" => (
          RegisterType::Glue,
          Some(RegisterValue::Glue(parse_glue(value_str)?)),
        ),
        "MG" => (
          RegisterType::MuGlue,
          Some(RegisterValue::MuGlue(parse_muglue(value_str)?)),
        ),
        "TK" => {
          // Token register: value is comma-separated token list, or "0" for empty
          let toks = if value_str == "0" || value_str.is_empty() {
            Vec::new()
          } else {
            parse_token_list(value_str)?
          };
          (
            RegisterType::Tokens,
            Some(RegisterValue::Tokens(Tokens::from(toks))),
          )
        },
        _ => return Ok(false),
      };

      // Perl-parity with def_register (binding/def/dialect.rs): store the
      // initial value at the Register's `address` slot AND set `default`, so
      // a subsequent `value_of` lookup — which reads state::with_value(address)
      // and falls back to `default` — actually sees the dump's initial value.
      // CharDefs read their immediate `value` field instead, so we skip the
      // storage write for them.
      let mut reg = Register {
        cs: cs_tok,
        register_type: reg_type,
        value: reg_value.clone(),
        default: if matches!(reg_type, RegisterType::CharDef) {
          None
        } else {
          reg_value.clone()
        },
        mathglyph,
        locator: current_dump_locator(),
        ..Register::default()
      };
      // Set address: prefer dump-supplied address (allocated registers),
      // fall back to CS name (direct registers like `\count1`).
      let has_explicit_address = dump_address.is_some();
      reg.address = dump_address.unwrap_or_else(|| key.to_string());
      // Copy parameters from the base register at the address slot if
      // present. The dump R-line carries no parameter spec, so without
      // this an alias like `\tex_skip:D` (R \skip G 0) loses the
      // `Number` index parameter that the base `\skip` register has.
      // At digest time this caused `\tex_skip:D 0 = ... sp \scan_stop:`
      // to skip the index reading entirely — the `0`, `=`, and rest got
      // treated as a glue value and stranded tokens. Driver: expl3
      // regex VM through `\__tl_analysis_a_store:`. See
      // project_expl3_regex_vm_engine.md item #2.
      if reg.parameters.is_none() && reg.address != key {
        let address_tok = Token {
          text: arena::pin(&reg.address),
          code: Catcode::CS,
          #[cfg(feature = "token-locators")]
          loc: 0,
        };
        if let Some(base_defn) = state::lookup_register_definition(&address_tok)
          && let Some(params) = base_defn.parameters.clone()
        {
          reg.parameters = Some(params);
        }
      }
      if !matches!(reg_type, RegisterType::CharDef)
        && let Some(ref rv) = reg_value
      {
        // Perl `R(...)` register dump-restore: the address slot
        // gets the initial value via `assign_internal('value', ...,
        // 'global')`. Mirror Perl `def_register` behavior: when the
        // address is allocated (different from CS) AND already has a
        // value (from an earlier V entry), DO NOT overwrite — the V
        // entry holds the runtime value (e.g. `\m@ne`'s `\count22 =
        // -1`), and the Register's `value` field is just the default
        // (typically 0). Without this guard, the M entry resets
        // `\count22` to 0, breaking `\settabs 20\columns` (loops
        // because `\m@ne` reads as 0 instead of -1, so
        // `\advance\count@\m@ne` doesn't decrement).
        let should_assign = !has_explicit_address || !state::has_value(&reg.address);
        if should_assign {
          state::assign_internal(
            TableName::Value,
            arena::pin(&reg.address),
            rv.clone(),
            Some(Scope::Global),
          );
        }
      }
      // Perl `I(...)` for the Register meaning entry — direct
      // `assign_internal('meaning', ..., 'global')`, bypassing the
      // `:locked` and add-only checks of install_definition.
      state::assign_internal(
        TableName::Meaning,
        cs_tok.get_cs_name(),
        Stored::from(reg),
        Some(Scope::Global),
      );
      Ok(true)
    },
    _ => Ok(false),
  }
}

/// Decode a character key from the dump. Handles:
/// - Single characters: "A", "è"
/// - URL-encoded control chars: "%19" (→ char 0x19), "%0A" (→ char 0x0A)
fn decode_char_key(key: &str) -> Option<char> {
  let decoded = url_decode(key);
  decoded.chars().next()
}

/// Char-keyed table key: dump uses the single character as the key.
/// `assign_internal` wants a SymStr — pin the single-char string.
fn char_key(ch: char) -> arena::SymStr {
  let mut buf = [0u8; 4];
  arena::pin(ch.encode_utf8(&mut buf))
}

/// Load a catcode entry: C\tCHAR\tCC\tVALUE.
/// Perl `Cc()` (`Core/Dumper.pm` L60): `assign_internal('catcode', ..., 'global')`.
fn load_catcode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad catcode char: {}", key))?;
  let (tag, val_str) = data
    .split_once('\t')
    .ok_or_else(|| format!("Bad catcode data: {}", data))?;
  if tag != "CC" {
    return Err(format!("Bad catcode data: {}", data));
  }
  let cc: u8 = val_str
    .parse()
    .map_err(|e| format!("Bad catcode value: {}", e))?;
  state::assign_internal(
    TableName::Catcode,
    char_key(ch),
    Stored::Catcode(Catcode::from(cc)),
    Some(Scope::Global),
  );
  Ok(true)
}

/// Load a lccode entry: LC\tCHAR\tCH\tVALUE.
/// Perl `Lc()` (`Core/Dumper.pm` L63): `assign_internal('lccode', ..., 'global')`.
fn load_lccode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad lccode char: {}", key))?;
  let (tag, val_str) = data
    .split_once('\t')
    .ok_or_else(|| format!("Bad lccode data: {}", data))?;
  if tag != "CH" {
    return Err(format!("Bad lccode data: {}", data));
  }
  let val: u16 = val_str
    .parse()
    .map_err(|e| format!("Bad lccode value: {}", e))?;
  state::assign_internal(
    TableName::Lccode,
    char_key(ch),
    Stored::Charcode(val),
    Some(Scope::Global),
  );
  Ok(true)
}

/// Load a uccode entry: UC\tCHAR\tCH\tVALUE.
/// Perl `Uc()` (`Core/Dumper.pm` L64): `assign_internal('uccode', ..., 'global')`.
fn load_uccode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad uccode char: {}", key))?;
  let (tag, val_str) = data
    .split_once('\t')
    .ok_or_else(|| format!("Bad uccode data: {}", data))?;
  if tag != "CH" {
    return Err(format!("Bad uccode data: {}", data));
  }
  let val: u16 = val_str
    .parse()
    .map_err(|e| format!("Bad uccode value: {}", e))?;
  state::assign_internal(
    TableName::Uccode,
    char_key(ch),
    Stored::Charcode(val),
    Some(Scope::Global),
  );
  Ok(true)
}

/// Load an sfcode entry: SC\tCHAR\tCH\tVALUE.
/// Perl `Sc()` (`Core/Dumper.pm` L62): `assign_internal('sfcode', ..., 'global')`.
fn load_sfcode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad sfcode char: {}", key))?;
  let (tag, val_str) = data
    .split_once('\t')
    .ok_or_else(|| format!("Bad sfcode data: {}", data))?;
  if tag != "CH" {
    return Err(format!("Bad sfcode data: {}", data));
  }
  let val: u16 = val_str
    .parse()
    .map_err(|e| format!("Bad sfcode value: {}", e))?;
  state::assign_internal(
    TableName::Sfcode,
    char_key(ch),
    Stored::Charcode(val),
    Some(Scope::Global),
  );
  Ok(true)
}

/// Load a delcode entry: DC\tCHAR\tCH\tVALUE
/// Mirrors Perl `Core/Dumper.pm:dump_delcode` round-trip.
fn load_delcode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad delcode char: {}", key))?;
  let (tag, val_str) = data
    .split_once('\t')
    .ok_or_else(|| format!("Bad delcode data: {}", data))?;
  if tag != "CH" {
    return Err(format!("Bad delcode data: {}", data));
  }
  let val: u16 = val_str
    .parse()
    .map_err(|e| format!("Bad delcode value: {}", e))?;
  state::assign_delcode(ch, val, Some(Scope::Global));
  Ok(true)
}

/// Load a mathcode entry: MC\tCHAR\tCH\tVALUE
/// Mirrors Perl `Core/Dumper.pm:dump_mathcode` round-trip.
fn load_mathcode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad mathcode char: {}", key))?;
  let (tag, val_str) = data
    .split_once('\t')
    .ok_or_else(|| format!("Bad mathcode data: {}", data))?;
  if tag != "CH" {
    return Err(format!("Bad mathcode data: {}", data));
  }
  let val: u16 = val_str
    .parse()
    .map_err(|e| format!("Bad mathcode value: {}", e))?;
  state::assign_mathcode(ch, val, Some(Scope::Global));
  Ok(true)
}

/// Parse a single token from "CC:TEXT" format
fn parse_token(s: &str) -> Result<Token, String> {
  let (cc_str, text) = s.split_once(':').ok_or("Missing ':' in token")?;
  let cc: u8 = cc_str.parse().map_err(|e| format!("Bad CC: {}", e))?;
  // Fast path: most token text fields have no `%` escapes, so pin the
  // &str directly — avoids the String allocation url_decode would make
  // even on its own fast path. Parsing the expl3 kernel alone produces
  // hundreds of thousands of token entries; every `to_owned()` avoided
  // here matters.
  let text_sym = if text.contains('%') {
    arena::pin(url_decode(text))
  } else {
    arena::pin(text)
  };
  Ok(Token {
    text: text_sym,
    code: Catcode::from(cc),
    #[cfg(feature = "token-locators")]
    loc: 0,
  })
}

/// Parse comma-separated token list
fn parse_token_list(s: &str) -> Result<Vec<Token>, String> {
  if s.is_empty() {
    return Ok(Vec::new());
  }
  // Pre-size the Vec. Avg ~19.6 tokens/list across the 16k E-entries
  // in latex.dump; the default `.collect()` size_hint is (0, None) so
  // Vec resizes ~log2(19) ≈ 5 times per call. Counting commas first
  // (one extra pass over the str) eliminates those re-allocs.
  let n = s.bytes().filter(|b| *b == b',').count() + 1;
  let mut out = Vec::with_capacity(n);
  for tok in s.split(',') {
    out.push(parse_token(tok)?);
  }
  Ok(out)
}

/// Decode the v3 structured Parameters encoding emitted by
/// `dump_writer::serialize_parameters_v3` (see that function's docstring
/// and `docs/archive/DUMP_FORMAT_PERL_ANALYSIS_2026-04-30.md` for the layout).
///
/// Returns `Ok(None)` for an empty record (no parameters); `Ok(Some(ps))`
/// on success; `Err` if any record is malformed. Each Parameter is
/// constructed via `Parameter::new(name, spec, Some(extras))`, which
/// calls `init()` — the reader function is resolved against the live
/// PARAMETER_TYPES table, mirroring the runtime path.
pub(crate) fn parse_parameters_v3(
  v3: &str,
) -> Result<Option<crate::parameter::Parameters>, String> {
  if v3.is_empty() {
    return Ok(None);
  }
  let mut params = Vec::new();
  for record in v3.split('\x1e') {
    // <name>\x1f<spec>\x1f<flags>\x1f<extras>
    let fields: Vec<&str> = record.splitn(4, '\x1f').collect();
    if fields.len() != 4 {
      return Err(format!(
        "v3 Parameter record has {} fields, expected 4",
        fields.len()
      ));
    }
    let name = url_decode(fields[0]);
    let spec = url_decode(fields[1]);
    let flags = fields[2];
    let extras_str = fields[3];

    let extras = if extras_str.is_empty() {
      Vec::new()
    } else {
      extras_str
        .split('\x1d')
        .map(|tok_list| parse_token_list(tok_list).map(Tokens::new))
        .collect::<Result<Vec<_>, _>>()?
    };

    let mut param = crate::parameter::Parameter::new(name, spec, Some(extras))
      .map_err(|e| format!("Parameter::new failed: {}", e))?;

    // Apply flags after construction — Parameter::new + init() handle
    // the reader side, but novalue/optional are struct-level booleans
    // that the spec-driven init() may or may not have set (e.g. "Optional"
    // prefix auto-sets optional, but we want explicit round-trip).
    for flag in flags.split(';').filter(|s| !s.is_empty()) {
      match flag {
        "n=1" => param.novalue = true,
        "o=1" => param.optional = true,
        _ => {
          // Unknown flag — ignore for forward compat; future flags
          // added to the writer shouldn't break older readers.
        },
      }
    }
    params.push(param);
  }
  if params.is_empty() {
    Ok(None)
  } else {
    Ok(Some(crate::parameter::Parameters::new(params)))
  }
}

pub(crate) fn url_decode(s: &str) -> String {
  // Fast path: the overwhelming majority of dump entries have no `%`
  // escapes in their key or proto fields, so a single memcpy via
  // `to_owned()` beats char-by-char iteration for ~19k key loads.
  if !s.contains('%') {
    return s.to_owned();
  }
  let mut result = String::with_capacity(s.len());
  let mut chars = s.chars();
  while let Some(ch) = chars.next() {
    if ch == '%' {
      let hex: String = chars.by_ref().take(2).collect();
      if let Ok(byte) = u8::from_str_radix(&hex, 16) {
        result.push(byte as char);
      }
    } else {
      result.push(ch);
    }
  }
  result
}

/// Parse a serialized Glue value: "skip,pN,pfN,mN,mfN"
fn parse_glue(s: &str) -> Result<crate::common::glue::Glue, String> {
  use crate::common::glue::{FillCode, Glue};
  let mut skip = 0i64;
  let mut plus = None;
  let mut pfill = None;
  let mut minus = None;
  let mut mfill = None;
  for (i, part) in s.split(',').enumerate() {
    if i == 0 {
      skip = part.parse().map_err(|e| format!("Bad glue skip: {}", e))?;
    } else if let Some(rest) = part.strip_prefix("pf") {
      pfill = FillCode::new(rest.parse::<usize>().unwrap_or(0));
    } else if let Some(rest) = part.strip_prefix('p') {
      plus = Some(rest.parse().map_err(|e| format!("Bad glue plus: {}", e))?);
    } else if let Some(rest) = part.strip_prefix("mf") {
      mfill = FillCode::new(rest.parse::<usize>().unwrap_or(0));
    } else if let Some(rest) = part.strip_prefix('m') {
      minus = Some(rest.parse().map_err(|e| format!("Bad glue minus: {}", e))?);
    }
  }
  Ok(Glue {
    skip,
    plus,
    pfill,
    minus,
    mfill,
  })
}

/// Parse a serialized MuGlue value (same format as Glue)
fn parse_muglue(s: &str) -> Result<crate::common::muglue::MuGlue, String> {
  use crate::common::{glue::FillCode, muglue::MuGlue};
  let mut skip = 0i64;
  let mut plus = None;
  let mut pfill = None;
  let mut minus = None;
  let mut mfill = None;
  for (i, part) in s.split(',').enumerate() {
    if i == 0 {
      skip = part
        .parse()
        .map_err(|e| format!("Bad muglue skip: {}", e))?;
    } else if let Some(rest) = part.strip_prefix("pf") {
      pfill = FillCode::new(rest.parse::<usize>().unwrap_or(0));
    } else if let Some(rest) = part.strip_prefix('p') {
      plus = Some(
        rest
          .parse()
          .map_err(|e| format!("Bad muglue plus: {}", e))?,
      );
    } else if let Some(rest) = part.strip_prefix("mf") {
      mfill = FillCode::new(rest.parse::<usize>().unwrap_or(0));
    } else if let Some(rest) = part.strip_prefix('m') {
      minus = Some(
        rest
          .parse()
          .map_err(|e| format!("Bad muglue minus: {}", e))?,
      );
    }
  }
  Ok(MuGlue {
    skip,
    plus,
    pfill,
    minus,
    mfill,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_load_native_dump_inline() {
    // Test with inline tab-separated dump content (no external file dependency)
    let content = "V\tcount@\tI\t42\nM\t\\mymacro\tE\t\\mymacro\t1\t\t6:1,6:2\n";
    let count = load_from_str(content).unwrap();
    assert!(
      count > 0,
      "Expected entries loaded from inline dump, got {}",
      count
    );
  }

  #[test]
  fn test_catcode_loading_nonascii() {
    // Only non-ASCII catcodes are loaded from the dump
    let content = "C\t\u{00e8}\tCC\t12\n"; // è → catcode 12 (OTHER)
    let count = load_from_str(content).unwrap();
    assert!(count > 0, "Expected non-ASCII catcode entry loaded");
  }

  #[test]
  fn test_lccode_loading() {
    let content = "LC\t\u{00c8}\tCH\t232\n"; // È → lccode 232 (è)
    let count = load_from_str(content).unwrap();
    assert!(count > 0, "Expected lccode entry loaded");
  }

  // --- RLE decoder tests (intarray consolidation) ---

  #[test]
  fn rle_decode_empty() {
    assert_eq!(rle_decode_i64("").unwrap(), Vec::<i64>::new());
  }

  #[test]
  fn rle_decode_single() {
    assert_eq!(rle_decode_i64("5").unwrap(), vec![5]);
  }

  #[test]
  fn rle_decode_single_run() {
    assert_eq!(rle_decode_i64("5x3").unwrap(), vec![5, 5, 5]);
  }

  #[test]
  fn rle_decode_mixed() {
    assert_eq!(rle_decode_i64("1,2x2,3x3,1").unwrap(), vec![
      1, 2, 2, 3, 3, 3, 1
    ]);
  }

  #[test]
  fn rle_decode_negative() {
    assert_eq!(rle_decode_i64("-5").unwrap(), vec![-5]);
    assert_eq!(rle_decode_i64("-5x3").unwrap(), vec![-5, -5, -5]);
  }

  #[test]
  fn rle_decode_long_run() {
    let v = rle_decode_i64("218x10000").unwrap();
    assert_eq!(v.len(), 10000);
    assert!(v.iter().all(|&x| x == 218));
  }

  #[test]
  fn rle_decode_malformed_returns_err() {
    assert!(rle_decode_i64("abc").is_err());
    assert!(rle_decode_i64("5xabc").is_err());
    assert!(rle_decode_i64("5x").is_err());
  }

  // --- IA load → state assignment tests ---

  #[test]
  fn ia_load_writes_per_slot_values() {
    // Use a unique prefix so the test doesn't collide with the engine's
    // ambient state (other tests may have populated fontdimen_* keys).
    let prefix = "ia_test_prefix";
    let content = format!("IA\t{}\t3\t10,20x2\n", prefix);
    load_from_str(&content).unwrap();

    use crate::{
      common::{dimension::Dimension, store::Stored},
      state,
    };

    assert_eq!(
      state::lookup_value(&format!("{}_1", prefix)),
      Some(Stored::Dimension(Dimension(10)))
    );
    assert_eq!(
      state::lookup_value(&format!("{}_2", prefix)),
      Some(Stored::Dimension(Dimension(20)))
    );
    assert_eq!(
      state::lookup_value(&format!("{}_3", prefix)),
      Some(Stored::Dimension(Dimension(20)))
    );
    // One past the end should NOT be set by the IA record.
    assert_eq!(state::lookup_value(&format!("{}_4", prefix)), None);
  }

  #[test]
  fn ia_load_length_mismatch_errors() {
    // Declared len 5 but RLE only decodes to 3 → error
    let content = "IA\tia_mismatch_prefix\t5\t10,20,30\n";
    // load_from_str collects per-line errors; verify the malformed IA
    // line did NOT successfully load anything.
    let count = load_from_str(content).unwrap_or(0);
    assert_eq!(count, 0, "Length-mismatch IA should not load");
  }

  // --- Backward-compat: V-records-only dumps (pre-IA format) ---

  #[test]
  fn v_record_dimension_still_loads() {
    // This is the pre-IA storage format: one V record per slot.
    // dump_reader must still accept these so older / partner-machine
    // dumps load correctly.
    let prefix = "v_backcompat_prefix";
    let content = format!("V\t{}_1\tD\t111\nV\t{}_2\tD\t222\n", prefix, prefix);
    load_from_str(&content).unwrap();

    use crate::{
      common::{dimension::Dimension, store::Stored},
      state,
    };

    assert_eq!(
      state::lookup_value(&format!("{}_1", prefix)),
      Some(Stored::Dimension(Dimension(111)))
    );
    assert_eq!(
      state::lookup_value(&format!("{}_2", prefix)),
      Some(Stored::Dimension(Dimension(222)))
    );
  }
}

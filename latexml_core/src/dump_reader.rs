//! Reader for Rust-native kernel dump files (produced by dump_writer.rs).
//!
//! Loads a dump file produced by `latexml_oxide --init=latex.ltx --dest=dump`
//! and replays the state assignments into the engine.
//!
//! **Loading policy:** The dump loads AFTER the compiled engine definitions.
//! Meanings (M entries) use add-only policy: skip if the CS is already defined.
//! Values (V entries) use add-only policy: skip if the key already has a value.
//! This ensures compiled engine semantics (constructors, etc.) take priority
//! over raw TeX definitions from the dump, matching Perl's approach where
//! `latex_constructs` overrides the dump.
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

use crate::common::arena;
use crate::common::numeric_ops::NumericOps;
use crate::common::store::Stored;
use crate::definition::expandable::{Expandable, ExpandableOptions};
use crate::state::{self, Scope};
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;

/// Load a Rust-native dump file into the current State.
/// Returns the number of entries loaded.
pub fn load_native_dump(path: &Path) -> Result<usize, String> {
  let content = std::fs::read_to_string(path)
    .map_err(|e| format!("Failed to read dump file {}: {}", path.display(), e))?;
  let count = load_from_str_internal(&content, &path.display().to_string())?;
  Ok(count)
}

/// Load dump data from a string (used by embedded dump modules).
/// Returns the number of entries loaded.
pub fn load_from_str(content: &str) -> Result<usize, String> {
  load_from_str_internal(content, "<embedded>")
}

// Per-load context used to attach a nominal Locator to dump-installed
// Expandables. Matches Perl #aaacdba2 (2026): dump-loaded definitions
// should be traceable to the dump file + line, not report the arena's
// internal location. Thread-local so concurrent loads (there are none
// today, but the state is cooperative) don't clobber each other.
thread_local! {
  static CURRENT_LOAD_CTX: std::cell::Cell<Option<(crate::common::arena::SymStr, u32)>> =
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
    // Add-only: if compiled definitions since dump-load have defined
    // the key themselves, don't override.
    if state::has_meaning(&cs_tok) {
      skipped += 1;
      continue;
    }
    // Target still undefined — the alias's target must be defined
    // in some source we never load (e.g. expl3 intarrays that the
    // short-circuit skips). Leave the key undefined; the engine's
    // undefined-CS handler will cope at runtime.
    if !state::has_meaning(&target_tok) {
      skipped += 1;
      continue;
    }
    state::let_i(&cs_tok, &target_tok, Some(Scope::Global));
    applied += 1;
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
  let source_sym = crate::common::arena::pin(source_name);

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
          log::warn!(
            "[dump_reader] Line {}: {}: {}",
            lineno + 1,
            e,
            &line[..line.len().min(80)]
          );
        }
      },
    }
  }

  if errors > 10 {
    log::warn!("[dump_reader] ... and {} more errors", errors - 10);
  }
  log::info!(
    "[dump_reader] Loaded {} entries from {} ({} skipped, {} errors)",
    count,
    source_name,
    skipped,
    errors
  );

  Ok(count)
}

/// Parse a single dump line and load it. Returns Ok(true) if loaded,
/// Ok(false) if skipped (already defined or filtered), Err on parse error.
fn parse_and_load(line: &str) -> Result<bool, String> {
  let parts: Vec<&str> = line.splitn(3, '\t').collect();
  if parts.len() < 2 {
    return Err("Too few fields".into());
  }

  let table = parts[0];
  // Key decode: Cow borrows the original &str when no `%` escape is
  // present (the overwhelming majority). Saves a per-line allocation
  // for the ~25k dump entries that have plain CS-name keys.
  let key_cow: std::borrow::Cow<'_, str> = if parts[1].contains('%') {
    std::borrow::Cow::Owned(url_decode(parts[1]))
  } else {
    std::borrow::Cow::Borrowed(parts[1])
  };
  let key = key_cow.as_ref();
  let data = if parts.len() > 2 { parts[2] } else { "" };

  match table {
    // V: Value entries (registers, fontdimen, font metadata).
    // Add-only policy: only loads if key has no existing value.
    "V" => load_value(key, data),
    // M: Meaning entries (expandable definitions + primitive aliases).
    //
    // Current policy: only route `@`-internal entries (whose body does
    // not reference the expl3 hook system) to `load_meaning`. Those
    // include `@`-internal PA/MPA aliases, which DO get consumed via
    // the PA arm of `load_meaning`. The @-internal gate is still the
    // right safety fence: it keeps us out of the expl3 short-circuit
    // hazard zone.
    //
    // `:`-style expl3 Meanings and public-CS PAs (like
    // `\tex_let:D → \let`) remain gated off because enabling them
    // causes two failure modes, both observed in earlier experiments:
    //
    //   - PA alone: `\tex_let:D` becomes let-aliased to `\let` via the dump → `expl3.sty`'s own
    //     guard fires → raw `\input expl3-code.tex` is skipped → post-guard code hits
    //     `\__kernel_dependency_version_check:Nn`, `\ProcessOptions`, `\keys_define:nn { sys }`,
    //     which are `:`-style macros we don't load → undefined-CS recovery loop (60 s timeout,
    //     memory climbing, SIGTERM-by-watchdog).
    //
    //   - PA + `:`-style M: the `:`-style bodies themselves trigger the same pattern via
    //     cross-references.
    //
    // Both must be unblocked TOGETHER, in coordination with
    // `expl3_sty.rs` short-circuiting its whole `load_definitions`
    // when the dump already supplies expl3 state. See SYNC_STATUS
    // D0 (d.5).
    "M" => {
      let name = key.trim_start_matches('\\');
      let is_at_internal = name.contains('@') && !name.contains(':');
      // Safe additional gate: public CharDef/Register entries (payload
      // starts with `R\t…`). These set a character code or register value
      // and never chain into expl3/hook machinery, so they can't trigger
      // the cascade the expl3 short-circuit is guarding against. Allows
      // plain-TeX math chardefs like `\ldotp`, `\cdotp`, `\intop` to load
      // without opening the door to public Expandable bodies.
      //
      // Targeted exclusion: `\BooleanTrue`/`\BooleanFalse` are defined
      // in latex.ltx L4408-4409 (TL2023 kernel xparse merge) AND
      // re-defined in xparse-2018-04-12.sty L2264-2265. Admitting them
      // from the dump means the legacy xparse-2018 raw-load (triggered
      // when `\NewDocumentCommand` isn't admitted) hits expl3's strict
      // "command-already-defined" check, producing 2 cosmetic LaTeX
      // errors + 2 undefineds (`\iow_wrap:nnnN`/`\iow_wrap:nenN`) per
      // document that does `\usepackage{xparse}`. Excluding these two
      // names lets xparse-2018 re-define them cleanly. See
      // project_kernel_dump_parity.md Stage 5 option (b).
      let is_public_register = data.starts_with("R\t")
        && name != "BooleanTrue"
        && name != "BooleanFalse";
      // Safe additional gate: Let-alias records (`PA\t<target>` or
      // `MPA\t<target>`) where NEITHER the key NOR the target is an
      // expl3 `:`-style identifier. These replay `\let <key> <target>`
      // at load time — the target must be an existing (Rc<Primitive>)
      // binding, so there's no body cascade, and without `:` in either
      // name we can't trip the expl3 short-circuit hazard the main
      // gate guards against. Recovers plain-LaTeX public aliases like
      // `\let\a=\@tabacckludge` (latex.ltx L10007) that previously
      // required hand-written `Let!(...)` in `latex_constructs.rs`.
      let is_safe_let_alias = {
        let (prefix, rest) = if let Some(r) = data.strip_prefix("PA\t") {
          ("PA", r)
        } else if let Some(r) = data.strip_prefix("MPA\t") {
          ("MPA", r)
        } else {
          ("", "")
        };
        let target_raw = if rest.contains('%') {
          url_decode(rest)
        } else {
          rest.to_string()
        };
        !prefix.is_empty()
          && !name.contains(':')
          && !target_raw.trim_start_matches('\\').contains(':')
      };
      // Round 17 — deep dumper parity, progressive widening of the
      // `:`-named entry admission. Each class is added only after
      // empirical verification that 83_expl3 + the full workspace
      // test suite pass. The record-type classification of the
      // 8,914 `:`-named M entries in the baseline latex.dump.txt:
      //   8,484 E  (Expandable — most can cascade via expansion)
      //     216 R  (Register — already admitted via is_public_register)
      //     156 PA (let-alias — trips expl3.sty guard; needs coord)
      //      44 N  (None — no-op)
      //      14 T  (Token — single assign_meaning, no chain)
      //
      // Step 1 (b44a065b6): N + T records — no cascade risk.
      // Step 2 (this commit): :-named E records with nargs=0 AND
      //   empty body — 43 of 8,484. These define empty macros,
      //   which expand to nothing. Canary-safe because the expl3.sty
      //   guard tests `\tex_let:D` (a PA, not an E), so admitting
      //   bodyless E entries doesn't trip it. The add-only policy in
      //   load_meaning means any later raw-expl3 redefinition wins.
      let is_safe_colon_noncascade =
        name.contains(':') && (data.starts_with("N") || data.starts_with("T\t"));
      // Step 2 & 3 combined: :-named E records with nargs=0 whose
      // body is either empty (43) or contains no CS tokens (303 more
      // — literal characters, digits, punctuation only). "No CS"
      // means no `16:` catcode marker in the comma-separated token
      // list, so there's no expansion chain to cascade.
      //
      // Step 4 attempted 1-CS bodies, step 5 attempted whole-E
      // admission. Both regressed 83_expl3 on `\ifdefined\X`
      // branch mis-selection. The lesson: widening the E gate
      // further requires porting the underlying kernel primitives
      // (\hook_*, \group_*, \keys_*, etc.) so the bodies we admit
      // execute correctly, not just parse. See the "Deep expl3 /
      // LaTeX 3 kernel parity" long-horizon task — the dumper
      // widening and the kernel port must advance together.
      let is_safe_colon_safe_e = name.contains(':') && data.starts_with("E\t") && {
        let eparts: Vec<&str> = data.split('\t').collect();
        let nargs_zero = eparts.get(2).map(|s| *s == "0").unwrap_or(false);
        let body = eparts.get(4).copied().unwrap_or("");
        let body_has_cs = body.starts_with("16:") || body.contains(",16:");
        nargs_zero && !body_has_cs
      };
      // Backwards alias preserved.
      let is_safe_colon_empty_e = is_safe_colon_safe_e;
      if (is_at_internal
        || is_public_register
        || is_safe_let_alias
        || is_safe_colon_noncascade
        || is_safe_colon_empty_e)
        && !data.contains("\\\\hook")
        && !data.contains("16:\\hook")
      {
        load_meaning(key, data)
      } else {
        Ok(false)
      }
    },
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
    // MC/DC: mathcodes and delcodes from the dump are corrupted by expl3
    // format initialization (e.g., mathcode('v')=618 maps to '|').
    // The engine sets correct math/delcodes. Skip.
    "MC" | "DC" => Ok(false),
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
/// Note: `_loaded` / `_found_loaded` flags are present in the dump (correctly,
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
  "_loaded",       // Package loading flags — see doc comment above
  "_found_loaded", // Package found+loaded flags
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

  // Add-only policy: don't overwrite any existing value.
  // This preserves engine-configured state (e.g., \everymath, \everypar,
  // named skips, etc.) while filling in gaps from the dump (e.g., expl3
  // fontdimen intarrays, font metadata).
  if state::has_value(key) {
    return Ok(false);
  }

  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.is_empty() {
    return Err("Missing value type".into());
  }

  let value = match parts[0] {
    "N" => return Ok(false), // Don't load None values (would erase existing)
    "B" => Stored::Bool(parts.get(1).map(|s| *s == "1").unwrap_or(false)),
    "I" => {
      let n: i64 = parts
        .get(1)
        .unwrap_or(&"0")
        .parse()
        .map_err(|e| format!("Bad int: {}", e))?;
      Stored::Int(n)
    },
    "S" => Stored::from(url_decode(parts.get(1).unwrap_or(&""))),
    "CH" => {
      let n: u16 = parts
        .get(1)
        .unwrap_or(&"0")
        .parse()
        .map_err(|e| format!("Bad charcode: {}", e))?;
      Stored::Charcode(n)
    },
    "CC" => {
      let n: u8 = parts
        .get(1)
        .unwrap_or(&"0")
        .parse()
        .map_err(|e| format!("Bad catcode: {}", e))?;
      Stored::Catcode(Catcode::from(n))
    },
    "T" => {
      let tok = parse_token(parts.get(1).unwrap_or(&""))?;
      Stored::Token(tok)
    },
    "TK" => {
      let toks = parse_token_list(parts.get(1).unwrap_or(&""))?;
      Stored::Tokens(Tokens::from(toks))
    },
    "D" => {
      let n: i64 = parts
        .get(1)
        .unwrap_or(&"0")
        .parse()
        .map_err(|e| format!("Bad dimension: {}", e))?;
      Stored::Dimension(crate::common::dimension::Dimension(n))
    },
    "G" => Stored::Glue(parse_glue(parts.get(1).unwrap_or(&"0"))?),
    "MD" => {
      let n: i64 = parts
        .get(1)
        .unwrap_or(&"0")
        .parse()
        .map_err(|e| format!("Bad mudimension: {}", e))?;
      Stored::MuDimension(crate::common::mudimension::MuDimension(n))
    },
    "MG" => Stored::MuGlue(parse_muglue(parts.get(1).unwrap_or(&"0"))?),
    "VD" => return Ok(false), // Don't load empty VecDeque (runtime state)
    _ => return Ok(false),    // Unknown value type
  };

  state::assign_value(key, value, Some(Scope::Global));
  Ok(true)
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
  };

  // Add-only policy: don't override ANY existing definition.
  if state::has_meaning(&cs_tok) {
    return Ok(false);
  }

  // Safety filter: only load definitions that won't interfere with normal
  // LaTeX processing. Public macros from the dump (like \document, \hook,
  // \UseOneTimeHook) can reference expl3 hooks and internal state that
  // our engine doesn't fully support, causing cascading errors.
  //
  // Safe: expl3 internals (with `:` or `__`), LaTeX internals (with `@`),
  //       public Register/CharDef entries (they set char codes and
  //       can't chain into expl3 cascades), AND public PA/MPA let-aliases
  //       whose target is not a `:`-style expl3 identifier (they replay
  //       `\let <key> <target>` against an existing Rc<Primitive>, so
  //       there's no body cascade either).
  // Unsafe: public Expandable macros without `:` or `@` (e.g., \document,
  //         \hook) — their bodies reference the hook system we don't
  //         fully support. `_base.rs` + `_constructs.rs` already define
  //         the public CSes the engine cares about; public-CS Expandable
  //         dump entries are redundant.
  let name = key.trim_start_matches('\\');
  let is_internal = name.contains(':') || name.contains('@');
  let is_public_register = data.starts_with("R\t");
  let is_public_let_alias = {
    let rest_opt = data
      .strip_prefix("PA\t")
      .or_else(|| data.strip_prefix("MPA\t"));
    rest_opt.is_some_and(|rest| {
      let target_raw = if rest.contains('%') {
        url_decode(rest)
      } else {
        rest.to_string()
      };
      !target_raw.trim_start_matches('\\').contains(':')
    })
  };
  if !is_internal && !is_public_register && !is_public_let_alias {
    return Ok(false);
  }

  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.is_empty() {
    return Err("Missing meaning type".into());
  }

  match parts[0] {
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
      let eparts: Vec<&str> = parts.get(1).unwrap_or(&"").splitn(6, '\t').collect();
      if eparts.len() < 4 {
        return Err("Incomplete Expandable entry".into());
      }

      // Note: eparts[0] is the internal CS name carried by the E
      // serialization (the Expandable's `self.cs`), which for non-alias
      // entries matches `key`. We don't decode it here — the outer key
      // is already parsed above into `cs_tok`, and `Expandable::new`
      // doesn't need a distinct internal name.
      let nargs: usize = eparts[1].parse().unwrap_or(0);
      let flags = eparts[2];
      let tok_data = eparts[3];
      let proto_opt = eparts
        .get(4)
        .map(|s| url_decode(s))
        .filter(|s| !s.is_empty());
      let v3_opt = eparts.get(5).filter(|s| !s.is_empty());

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
          state::install_definition(exp, Some(Scope::Global));
          Ok(true)
        },
        Err(e) => Err(format!("Expandable creation failed: {}", e)),
      }
    },
    "T" => {
      // Token meaning (let-assignment)
      let tok = parse_token(parts.get(1).unwrap_or(&""))?;
      state::assign_meaning(&cs_tok, tok, Some(Scope::Global));
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
      let target_cs_raw = url_decode(parts.get(1).unwrap_or(&""));
      if target_cs_raw == key {
        return Ok(false);
      }
      let target_tok = Token {
        text: arena::pin(&target_cs_raw),
        code: Catcode::CS,
      };
      // Target not yet defined — defer the alias. Load order is
      // `bootstrap → _base → dump → _constructs`, and some aliases
      // (e.g. `\let\a=\@tabacckludge` from latex.ltx L10007) point
      // at CSes defined in _constructs, which runs after the dump.
      // `flush_deferred_aliases()` retries these after _constructs
      // finishes.
      if !state::has_meaning(&target_tok) {
        DEFERRED_ALIASES.with(|cell| {
          cell.borrow_mut().push((cs_tok, target_tok));
        });
        return Ok(false);
      }
      state::let_i(&cs_tok, &target_tok, Some(Scope::Global));
      Ok(true)
    },
    "R" => {
      // Register: R\tCS\tTYPE\tVALUE[\tMATHGLYPH]
      // rparts[0] (internal CS name) is redundant with the outer key —
      // same reasoning as the E arm; we skip the decode + alloc.
      let rparts: Vec<&str> = parts.get(1).unwrap_or(&"").splitn(4, '\t').collect();
      if rparts.len() < 3 {
        return Err("Incomplete Register entry".into());
      }
      let rtype = rparts[1];
      let value_str = rparts[2];
      let mathglyph = rparts
        .get(3)
        .and_then(|s| s.parse::<u32>().ok())
        .and_then(char::from_u32);

      use crate::common::number::Number;
      use crate::definition::register::{Register, RegisterType, RegisterValue};

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
      // Set address from CS name
      reg.address = key.to_string();
      if !matches!(reg_type, RegisterType::CharDef) {
        if let Some(ref rv) = reg_value {
          state::assign_value(&reg.address, rv.clone(), Some(Scope::Global));
        }
      }
      state::install_definition(reg, Some(Scope::Global));
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

/// Load a catcode entry: C\tCHAR\tCC\tVALUE
fn load_catcode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad catcode char: {}", key))?;
  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.len() < 2 || parts[0] != "CC" {
    return Err(format!("Bad catcode data: {}", data));
  }
  let cc: u8 = parts[1]
    .parse()
    .map_err(|e| format!("Bad catcode value: {}", e))?;
  state::assign_catcode(ch, Catcode::from(cc), Some(Scope::Global));
  Ok(true)
}

/// Load a lccode entry: LC\tCHAR\tCH\tVALUE
fn load_lccode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad lccode char: {}", key))?;
  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.len() < 2 || parts[0] != "CH" {
    return Err(format!("Bad lccode data: {}", data));
  }
  let val: u16 = parts[1]
    .parse()
    .map_err(|e| format!("Bad lccode value: {}", e))?;
  state::assign_lccode(ch, val, Some(Scope::Global));
  Ok(true)
}

/// Load a uccode entry: UC\tCHAR\tCH\tVALUE
fn load_uccode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad uccode char: {}", key))?;
  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.len() < 2 || parts[0] != "CH" {
    return Err(format!("Bad uccode data: {}", data));
  }
  let val: u16 = parts[1]
    .parse()
    .map_err(|e| format!("Bad uccode value: {}", e))?;
  state::assign_uccode(ch, val, Some(Scope::Global));
  Ok(true)
}

/// Load a sfcode entry: SC\tCHAR\tCH\tVALUE
fn load_sfcode(key: &str, data: &str) -> Result<bool, String> {
  let ch = decode_char_key(key).ok_or_else(|| format!("Bad sfcode char: {}", key))?;
  let parts: Vec<&str> = data.splitn(2, '\t').collect();
  if parts.len() < 2 || parts[0] != "CH" {
    return Err(format!("Bad sfcode data: {}", data));
  }
  let val: u16 = parts[1]
    .parse()
    .map_err(|e| format!("Bad sfcode value: {}", e))?;
  state::assign_sfcode(ch, val, Some(Scope::Global));
  Ok(true)
}

// load_delcode and load_mathcode were implemented but never wired — the
// "MC"/"DC" arm in parse_entry returns Ok(false) because the dumped values
// are corrupted by expl3 format init (see comment on that arm). If we
// eventually harvest clean delcode/mathcode data, restore them from git
// history and point the "MC"/"DC" arm at them.

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
  })
}

/// Parse comma-separated token list
fn parse_token_list(s: &str) -> Result<Vec<Token>, String> {
  if s.is_empty() {
    return Ok(Vec::new());
  }
  s.split(',').map(parse_token).collect()
}

/// Decode the v3 structured Parameters encoding emitted by
/// `dump_writer::serialize_parameters_v3` (see that function's docstring
/// and `docs/DUMP_FORMAT_PERL_ANALYSIS.md` for the layout).
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
        .map(|tok_list| parse_token_list(tok_list).map(crate::tokens::Tokens::new))
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
  use crate::common::glue::FillCode;
  use crate::common::muglue::MuGlue;
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
}

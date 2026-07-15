//! Pre-BibTeX: low-level `.bib` file parser.
//!
//! Perl: `LaTeXML/blib/lib/LaTeXML/Pre/BibTeX.pm` (~350 LOC) +
//! `LaTeXML/blib/lib/LaTeXML/Pre/BibTeX/Entry.pm` (~70 LOC).
//!
//! Responsible for the lowest-level parsing of BibTeX files: scans
//! `.bib` source, identifies `@preamble`, `@string`, `@comment`, and
//! entry blocks, performs `@string`-macro substitution, and stores
//! parsed entries. It does NOT analyze fields semantically — that is
//! `BibTeX.pool.ltxml` (Rust: `bibtex.rs`).
//!
//! Per Perl `Pre/BibTeX.pm` L23-32, the result `toTeX()` is wrapped in
//! a `\begin{bibtex@bibliography}…\end{bibtex@bibliography}` block
//! whose body is a sequence of `\ProcessBibTeXEntry{<key>}` macro
//! calls. The bibtex.rs orchestration then drives field dispatch +
//! XML construction from the registered `BibEntry`.
//!
//! ## Faithful-port notes
//!
//! - `parse_entry_type` / `parse_entry_name` / `parse_field_name` character classes are
//!   byte-for-byte the Perl regex `[BIBNAME_re ++ BIBNOISE_re]` (Perl L221-222). Entry-name
//!   additionally allows `"#%&'()={` (Perl L233); field-name additionally allows `&` (Perl L238).
//!   Entry-type and field-name are lower-cased; entry-name preserves case at the parse layer (the
//!   registry then normalises via `register_entry`).
//!
//! - `parse_string` reads `"…"` (allowing balanced `{…}` inside that defeat the closing quote) OR
//!   `{…}` (balanced braces, outer braces stripped) — exact match for Perl L252-274.
//!
//! - `parse_value` concatenates simple values separated by `#`. A simple value is either a
//!   delimited string or a name; a name matching `^\d+$` is taken literally, otherwise looked up in
//!   the macro table.
//!
//! - `skip_junk` is greedy: everything up to the next `@` is discarded (Perl L335-346: "anything
//!   until @ as an implied comment").

use std::collections::VecDeque;

use latexml_core::s;
use rustc_hash::FxHashMap as HashMap;

use crate::bibtex::{BibEntry, register_entry};

/// Default `@string`-style macros, populated on every new
/// `PreBibTeX`. Mirrors Perl L34-57 `%default_macros`.
fn default_macros() -> HashMap<String, String> {
  let pairs: &[(&str, &str)] = &[
    ("jan", "January"),
    ("feb", "February"),
    ("mar", "March"),
    ("apr", "April"),
    ("may", "May"),
    ("jun", "June"),
    ("jul", "July"),
    ("aug", "August"),
    ("sep", "September"),
    ("oct", "October"),
    ("nov", "November"),
    ("dec", "December"),
    ("acmcs", "ACM Computing Surveys"),
    ("acta", "Acta Informatica"),
    ("cacm", "Communications of the ACM"),
    ("ibmjrd", "IBM Journal of Research and Development"),
    ("ibmsj", "IBM Systems Journal"),
    ("ieeese", "IEEE Transactions on Software Engineering"),
    ("ieeetc", "IEEE Transactions on Computers"),
    (
      "ieeetcad",
      "IEEE Transactions on Computer-Aided Design of Integrated Circuits",
    ),
    ("ipl", "Information Processing Letters"),
    ("jacm", "Journal of the ACM"),
    ("jcss", "Journal of Computer and System Sciences"),
    ("scp", "Science of Computer Programming"),
    ("sicomp", "SIAM Journal on Computing"),
    ("tocs", "ACM Transactions on Computer Systems"),
    ("tods", "ACM Transactions on Database Systems"),
    ("tog", "ACM Transactions on Graphics"),
    ("toms", "ACM Transactions on Mathematical Software"),
    ("toois", "ACM Transactions on Office Information Systems"),
    (
      "toplas",
      "ACM Transactions on Programming Languages and Systems",
    ),
    ("tcs", "Theoretical Computer Science"),
  ];
  pairs
    .iter()
    .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
    .collect()
}

/// Match Perl's `%CLOSE = ("{" => "}", "(" => ")")` (L154).
fn close_for(open: char) -> char {
  match open {
    '{' => '}',
    '(' => ')',
    _ => open,
  }
}

/// List of `(field-name, value)` pairs returned by `parse_fields`.
/// Aliased to keep the clippy::type_complexity check happy on
/// `parse_fields`' tuple return.
type BibFieldList = Vec<(String, String)>;

/// `Pre::BibTeX` — low-level .bib parser state.
///
/// Perl object hash keys → Rust fields:
/// * `source` → `source`
/// * `file`   → `file_label` (display only)
/// * `lines`  → `lines` (remaining tail; current line lives in `line`)
/// * `line`   → `line` (current scan buffer)
/// * `lineno` → `lineno`
/// * `preamble`  → `preamble`
/// * `entries`   → `entries` as `(type, key, fields, rawfields)` quadruples
/// * `macros`    → `macros`
/// * `parsed` flag → `parsed`
pub struct PreBibTeX {
  pub source:     Option<String>,
  pub file_label: String,
  /// The unread tail, popped from the FRONT one physical line at a time.
  /// A `VecDeque` (not a `Vec`) because `Vec::remove(0)` shifts the whole
  /// remainder per line, making a `.bib` parse O(lines²): a 16k-entry file
  /// took 1.42 s, 15× the per-entry cost of a 1k-entry one. `pop_front` is
  /// O(1) and makes it linear.
  lines:          VecDeque<String>,
  line:           String,
  lineno:         usize,
  pub preamble:   Vec<String>,
  pub entries:    Vec<ParsedEntry>,
  macros:         HashMap<String, String>,
  parsed:         bool,
  /// Raw-source witness for `parse_value`. When `Some`, it holds the
  /// value's starting `line` plus every continuation `extend_line`
  /// has appended since — so `line` stays a true *suffix* of it, and
  /// the consumed span is `witness[..witness.len() - line.len()]`.
  /// Without this, a value spanning physical lines made `line` longer
  /// than (and unrelated to) the snapshot it was diffed against.
  raw_witness:    Option<String>,
}

/// One parsed entry. Mirrors Perl `LaTeXML::Pre::BibTeX::Entry` (only
/// the data is kept here; processed‐Tokens fields live in
/// `bibtex::BibEntry` and are populated by the pool orchestration).
#[derive(Debug, Clone)]
pub struct ParsedEntry {
  pub entry_type: String,
  pub key:        String,
  /// (field-name, macro-expanded-value) — Perl `fieldlist`.
  pub fields:     Vec<(String, String)>,
  /// (field-name, raw-source-value, *before* macro expansion) — Perl
  /// `rawfieldlist`.
  pub raw_fields: Vec<(String, String)>,
}

/// Parse errors. The Perl side calls `Error(...)` (recoverable) for
/// most issues; we surface them as `Err` instead so the caller can
/// decide whether to keep parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BibParseError {
  /// `parse_match` couldn't find any of `delims` at the head of the
  /// current line.
  Expected { delims: String, lineno: usize },
  /// `parse_value` saw neither a delimited string nor a name.
  ExpectedValue { lineno: usize },
  /// `parse_string` saw something that wasn't `"` or `{`.
  ExpectedDelimitedString { lineno: usize },
  /// `extend_line` hit EOF while scanning a string.
  UnexpectedEof { lineno: usize },
  /// `parse_value` encountered a name that isn't a defined macro.
  /// (Perl emits Error but defaults the value to the name itself; we
  /// follow that recovery — this variant is for tests / diagnostics.)
  UndefinedMacro { name: String, lineno: usize },
}

impl PreBibTeX {
  /// Perl `newFromString` (L77-86).
  pub fn new_from_string(s: &str) -> Self {
    let mut lines: VecDeque<String> = s.split('\n').map(String::from).collect();
    let first = lines.pop_front().unwrap_or_default();
    Self {
      source: None,
      file_label: s!("<anonymous>"),
      lines,
      line: first,
      lineno: 1,
      preamble: Vec::new(),
      entries: Vec::new(),
      macros: default_macros(),
      parsed: false,
      raw_witness: None,
    }
  }

  /// Perl `newFromFile` (L60-75). Reads the file at `path` directly;
  /// search-path resolution is the caller's responsibility (we do not
  /// reach into `state` here so the parser stays unit-testable).
  ///
  /// Decoded via `decode_input_bytes`, not `read_to_string`: a `.bib` has no
  /// `\inputencoding` to declare, real `bibtex` is 8-bit clean, and a strict
  /// UTF-8 read turns one stray Cp1252 byte into a lost bibliography
  /// (witness 2605.00490).
  pub fn new_from_file(path: &str) -> std::io::Result<Self> {
    let content = latexml_core::mouth::decode_input_bytes(&std::fs::read(path)?);
    let mut me = Self::new_from_string(&content);
    me.source = Some(path.to_string());
    me.file_label = path.to_string();
    Ok(me)
  }

  /// Perl `newFromGullet` (L89-104). Drains *raw* lines from the
  /// gullet mouths (with `close_mouth` between exhausted mouths) and
  /// builds a `PreBibTeX` from the concatenated body. Mirrors the
  /// integration point in `Core.pm` L160-162: after `InputContent`
  /// has opened a mouth onto the `.bib` file, drain that mouth here.
  ///
  /// Requires that the calling thread has a live `State`/gullet
  /// (i.e. is being run as part of a `digest_file` flow). Not
  /// available from `#[cfg(test)]` unit tests.
  pub fn new_from_gullet(name: &str) -> Self {
    use latexml_core::gullet;
    let mut lines: Vec<String> = Vec::new();
    while gullet::has_more_input() {
      while let Some(line) = gullet::read_raw_line() {
        lines.push(s!("{line}\n"));
      }
      // `close_mouth` may pop a stacked mouth; we then re-check
      // `has_more_input` on the next iteration. Forced=false is fine
      // because we've drained the mouth to EOF.
      let _ = gullet::close_mouth(false);
    }
    let joined = lines.join("");
    let mut me = Self::new_from_string(&joined);
    me.source = Some(name.to_string());
    me.file_label = name.to_string();
    me
  }

  /// Convenience for callers that want both the entries AND the
  /// `\begin{bibtex@bibliography}…\end{...}` wrapper TeX.
  pub fn into_parsed(mut self) -> Result<Self, BibParseError> {
    self.parse_top_level()?;
    Ok(self)
  }

  // ---- toString / locator ---------------------------------------------------

  /// Perl `toString` (L106-108).
  pub fn to_string_label(&self) -> String {
    s!(
      "Bibliography[{}]",
      self.source.as_deref().unwrap_or("<Unknown>")
    )
  }

  /// Perl `getLocator` (L125-129). We don't model the full
  /// `LaTeXML::Common::Locator` here; just `(source, lineno)`.
  pub fn locator(&self) -> (Option<&str>, usize) { (self.source.as_deref(), self.lineno) }

  // ---- top-level parse ------------------------------------------------------

  /// Perl `parseTopLevel` (L138-151).
  ///
  /// DIVERGENCE (OXIDIZED_DESIGN #58): a malformed entry is reported and
  /// **resynced at the next `@`** instead of aborting the file. Perl lets the
  /// first parse error propagate out of `parseTopLevel`, so a single unbalanced
  /// `{` anywhere in a `.bib` silently costs every LATER entry — the tail of the
  /// bibliography just disappears. Real BibTeX does not: on "I was expecting a
  /// `,' or a `}'" it reports the error and skips to the next entry
  /// (`bibtex.web`), which is the behaviour authors' `.bbl` files are built
  /// against. `skip_junk` is already exactly that resync, so the loop simply
  /// keeps going. The malformed entry itself is dropped (as BibTeX drops it).
  /// Corpus: 19 papers / 298 messages carry `bibtex:unbalanced`.
  pub fn parse_top_level(&mut self) -> Result<(), BibParseError> {
    while self.skip_junk() {
      let typ = match self.parse_entry_type() {
        Some(t) => t,
        None => continue,
      };
      let outcome = match typ.as_str() {
        "preamble" => self.parse_preamble(),
        "string" => self.parse_macro(),
        "comment" => self.parse_comment(),
        _ => self.parse_entry(&typ),
      };
      if let Err(e) = outcome {
        // Loud, never silent: the entry IS lost, and the reader must know
        // WHICH one. NB `Warn!` takes a pre-formatted message and appends any
        // further args as separate detail LINES — it does NOT interpolate
        // `{}`, so the message must be built with `s!` first.
        latexml_core::Warn!(
          "bibtex",
          "unbalanced",
          s!(
            "{} line {}: {:?}; resyncing at the next '@'",
            self.to_string_label(),
            self.lineno,
            e
          )
        );
      }
    }
    self.parsed = true;
    Ok(())
  }

  /// Perl `parsePreamble` (L158-164).
  fn parse_preamble(&mut self) -> Result<(), BibParseError> {
    let open = self.parse_match("({")?;
    let (value, _raw) = self.parse_value()?;
    self.preamble.push(value);
    self.parse_match(&close_for(open).to_string())?;
    Ok(())
  }

  /// Perl `parseMacro` (L168-174).
  fn parse_macro(&mut self) -> Result<(), BibParseError> {
    let open = self.parse_match("({")?;
    let (fields, _raw) = self.parse_fields("@string", open)?;
    for (name, value) in fields {
      self.macros.insert(name, value);
    }
    Ok(())
  }

  /// Perl `parseComment` (L177-181). The value is parsed and discarded.
  ///
  /// An **undelimited** `@Comment` is not an error: BibTeX simply ignores
  /// everything after `@comment` up to the next entry (`bibtex.web` never
  /// scans it), and `.bib` files in the wild use bare separator banners like
  /// `@Comment ------AAAAAA------`. Perl demands a delimited string here and
  /// errors out; we discard the junk and carry on, which `skip_junk` in the
  /// caller does anyway. OXIDIZED_DESIGN #60, witness 2605.06974 (26 such
  /// banners, each costing the entry that followed it).
  fn parse_comment(&mut self) -> Result<(), BibParseError> {
    self.skip_white();
    if self.line.starts_with('"') || self.line.starts_with('{') {
      let _ = self.parse_string()?;
    }
    Ok(())
  }

  /// Perl `parseEntry` (L185-193).
  fn parse_entry(&mut self, entry_type: &str) -> Result<(), BibParseError> {
    let open = self.parse_match("({")?;
    let key = self.parse_entry_name().unwrap_or_default();
    self.parse_match(",")?;
    let (fields, raw_fields) = self.parse_fields("@entry", open)?;
    self.entries.push(ParsedEntry {
      entry_type: entry_type.to_string(),
      key,
      fields,
      raw_fields,
    });
    Ok(())
  }

  /// Perl `parseFields` (L195-209).
  fn parse_fields(
    &mut self,
    _for: &str,
    open: char,
  ) -> Result<(BibFieldList, BibFieldList), BibParseError> {
    let mut fields = Vec::new();
    let mut raw_fields = Vec::new();
    let close = close_for(open);
    loop {
      let name = match self.parse_field_name() {
        Some(n) if !n.is_empty() => n,
        _ => break,
      };
      self.parse_match("=")?;
      let (value, raw) = self.parse_value()?;
      fields.push((name.clone(), value));
      raw_fields.push((name, raw));
      self.skip_white();
      // Perl: `($$self{line} =~ s/^,//) && skipWhite && ($$self{line} !~ /^\Q$CLOSE{$open}\E/)`
      if !self.consume_prefix(",") {
        break;
      }
      self.skip_white();
      if self.line.starts_with(close) {
        break;
      }
    }
    self.parse_match(&close.to_string())?;
    Ok((fields, raw_fields))
  }

  // ---- low-level lexers -----------------------------------------------------

  /// Perl `parseEntryType` (L224-227). Lowercased.
  fn parse_entry_type(&mut self) -> Option<String> {
    self.skip_white();
    let s = self.consume_while(is_bib_name_or_noise);
    if s.is_empty() {
      None
    } else {
      Some(s.to_ascii_lowercase())
    }
  }

  /// Perl `parseEntryName` (L229-233). Case preserved.
  fn parse_entry_name(&mut self) -> Option<String> {
    self.skip_white();
    // Allows the BibName+BibNoise classes *plus* the literal set:
    //   "  #  %  &  '  (  )  =  {
    let s = self.consume_while(|c| {
      is_bib_name_or_noise(c) || matches!(c, '"' | '#' | '%' | '&' | '\'' | '(' | ')' | '=' | '{')
    });
    if s.is_empty() { None } else { Some(s) }
  }

  /// Perl `parseFieldName` (L235-238). Lowercased; allows `&` in
  /// addition to the BibName+BibNoise classes.
  fn parse_field_name(&mut self) -> Option<String> {
    self.skip_white();
    let s = self.consume_while(|c| is_bib_name_or_noise(c) || c == '&');
    if s.is_empty() {
      None
    } else {
      Some(s.to_ascii_lowercase())
    }
  }

  /// Perl `parseMatch` (L240-249). On success consumes exactly one
  /// of `delims` and returns it. On miss returns
  /// `BibParseError::Expected`.
  fn parse_match(&mut self, delims: &str) -> Result<char, BibParseError> {
    self.skip_white();
    if delims.is_empty() {
      return Err(BibParseError::Expected {
        delims: String::new(),
        lineno: self.lineno,
      });
    }
    let head = self.line.chars().next();
    if let Some(c) = head
      && delims.contains(c)
    {
      // pop the matched character (always ASCII single-byte here)
      let mut new_line = String::with_capacity(self.line.len() - c.len_utf8());
      let mut it = self.line.chars();
      it.next();
      for ch in it {
        new_line.push(ch);
      }
      self.line = new_line;
      return Ok(c);
    }
    Err(BibParseError::Expected {
      delims: delims.to_string(),
      lineno: self.lineno,
    })
  }

  /// Perl `parseString` (L252-274).
  fn parse_string(&mut self) -> Result<String, BibParseError> {
    self.skip_white();
    let mut out = String::new();
    if self.line.starts_with('"') {
      // opening "; consume it
      self.line.replace_range(0..1, "");
      // loop until we find the closing "
      loop {
        if self.line.starts_with('"') {
          self.line.replace_range(0..1, "");
          break;
        }
        if self.line.is_empty() {
          if !self.extend_line()? {
            return Err(BibParseError::UnexpectedEof { lineno: self.lineno });
          }
        } else if self.line.starts_with('{') {
          out.push_str(&self.parse_balanced_braces()?);
        } else {
          // pull off everything except a brace or "
          let idx = self.line.find(['"', '{']).unwrap_or(self.line.len());
          out.push_str(&self.line[..idx]);
          self.line.replace_range(..idx, "");
        }
      }
    } else if self.line.starts_with('{') {
      let mut s = self.parse_balanced_braces()?;
      // strip the surrounding delimiters (Perl L267-268)
      if s.starts_with('{') {
        s.remove(0);
      }
      if s.ends_with('}') {
        s.pop();
      }
      out = s;
    } else {
      return Err(BibParseError::ExpectedDelimitedString { lineno: self.lineno });
    }
    // trim leading/trailing whitespace (Perl L272-273)
    Ok(out.trim().to_string())
  }

  /// Perl `parseBalancedBraces` (L276-283). Uses our own balanced-
  /// brace extractor (no Perl `Text::Balanced` dependency). Pulls
  /// in further lines via `extend_line` until a balanced pair is
  /// found. Brace counting follows `Text::Balanced::extract_bracketed`
  /// semantics for `'{}'`: nested braces count, and a `\{` / `\}`
  /// pair is treated as a single escaped brace (Perl's
  /// `extract_bracketed` honours backslash-escaped delimiters when
  /// the delimiter class is just the pair).
  fn parse_balanced_braces(&mut self) -> Result<String, BibParseError> {
    // Make sure there's at least one `}` to *potentially* close.
    while !self.line.contains('}') {
      if !self.extend_line()? {
        return Err(BibParseError::UnexpectedEof { lineno: self.lineno });
      }
    }
    loop {
      if let Some(end) = find_balanced_brace_end(&self.line) {
        let s = self.line[..end].to_string();
        self.line.replace_range(..end, "");
        return Ok(s);
      }
      if !self.extend_line()? {
        return Err(BibParseError::UnexpectedEof { lineno: self.lineno });
      }
    }
  }

  /// Perl `extendLine` (L285-295). Returns Ok(true) if a line was
  /// appended, Ok(false) on EOF (Perl's `Error('unexpected', ...)`).
  fn extend_line(&mut self) -> Result<bool, BibParseError> {
    if self.lines.is_empty() {
      return Ok(false);
    }
    let next = match self.lines.pop_front() {
      Some(n) => n,
      None => return Ok(false),
    };
    // Perl `<$BIB>` lines keep the trailing newline; `split /\n/` on a
    // string does not. Re-insert one to preserve line semantics so
    // that a `%`-comment inside a continued value still terminates on
    // the next physical line (BibTeX itself doesn't treat `%` as a
    // comment but other downstream digesters might).
    self.line.push('\n');
    self.line.push_str(&next);
    // Mirror the append into the raw witness so `line` remains a suffix
    // of it across a multi-line value (see `raw_witness`).
    if let Some(w) = self.raw_witness.as_mut() {
      w.push('\n');
      w.push_str(&next);
    }
    self.lineno += 1;
    Ok(true)
  }

  /// Perl `parseValue` (L299-318). Returns `(macro_expanded, raw)`.
  /// `raw` is the byte-for-byte concatenation of the simple values'
  /// source — Perl actually stores the same `raw` per element of
  /// `rawfieldlist`, but never re-emits it as a single string. We
  /// preserve the convention here so `BibEntry.add_raw_field` gets the
  /// pre-macro-expansion text.
  fn parse_value(&mut self) -> Result<(String, String), BibParseError> {
    let mut value = String::new();
    let mut raw = String::new();
    loop {
      self.skip_white();
      let head = self.line.chars().next();
      if matches!(head, Some('"') | Some('{')) {
        // Pull the raw form *before* macro expansion (parse_string
        // strips delimiters; for the raw side we want the actual
        // source bytes, delimiters and all).
        self.raw_witness = Some(self.line.clone());
        let s = self.parse_string()?;
        let witness = self.raw_witness.take().unwrap_or_default();
        let raw_consumed = consumed_diff(&witness, &self.line);
        raw.push_str(&raw_consumed);
        value.push_str(&s);
      } else if let Some(name) = self.parse_field_name() {
        let is_digits = !name.is_empty() && name.chars().all(|c| c.is_ascii_digit());
        let resolved = if is_digits {
          name.clone()
        } else {
          self.macros.get(&name).cloned().unwrap_or_else(|| {
            // Perl recovery: leave the name as-is.
            name.clone()
          })
        };
        raw.push_str(&name);
        value.push_str(&resolved);
      } else {
        return Err(BibParseError::ExpectedValue { lineno: self.lineno });
      }
      self.skip_white();
      if !self.consume_prefix("#") {
        break;
      }
      // Perl spelling: a `#`-separated value re-emits as a `#` in raw.
      raw.push('#');
    }
    Ok((value, raw))
  }

  /// Perl `skipWhite` (L320-330). Returns true if the cursor is now
  /// on a non-empty line, false on EOF.
  fn skip_white(&mut self) -> bool {
    loop {
      // Trim leading whitespace (any unicode whitespace, like Perl `\s`).
      let trimmed = self.line.trim_start_matches(char::is_whitespace);
      if trimmed.len() < self.line.len() {
        self.line = trimmed.to_string();
      }
      if !self.line.is_empty() {
        return true;
      }
      if self.lines.is_empty() {
        return false;
      }
      self.line = self.lines.pop_front().unwrap_or_default();
      self.lineno += 1;
    }
  }

  /// Perl `skipJunk` (L335-346). Discards everything up to the next
  /// `@` (or `%`, which Perl treats as the start of a one-line
  /// comment though BibTeX itself doesn't). Returns true when an `@`
  /// was found and consumed; false on EOF.
  fn skip_junk(&mut self) -> bool {
    loop {
      if let Some(idx) = self.line.find(['@', '%']) {
        // Drop everything before the marker.
        self.line.replace_range(..idx, "");
        // Perl L340: `return '@' if $$self{line} =~ s/^@//;`. If we
        // landed on `%`, the line is treated as a comment — drop the
        // rest of it and fetch the next line.
        if self.line.starts_with('@') {
          self.line.replace_range(0..1, "");
          return true;
        }
        // %-comment: drop the rest of this line and continue.
        self.line.clear();
      } else {
        self.line.clear();
      }
      if self.lines.is_empty() {
        return false;
      }
      self.line = self.lines.pop_front().unwrap_or_default();
      self.lineno += 1;
    }
  }

  // ---- small helpers --------------------------------------------------------

  fn consume_while(&mut self, predicate: impl Fn(char) -> bool) -> String {
    let end = self
      .line
      .char_indices()
      .find(|(_, c)| !predicate(*c))
      .map(|(i, _)| i)
      .unwrap_or(self.line.len());
    let head = self.line[..end].to_string();
    self.line.replace_range(..end, "");
    head
  }

  fn consume_prefix(&mut self, p: &str) -> bool {
    if self.line.starts_with(p) {
      self.line.replace_range(..p.len(), "");
      true
    } else {
      false
    }
  }

  // ---- public emitters ------------------------------------------------------

  /// Perl `toTeX` (L110-122). Registers every parsed entry into the
  /// `bibtex.rs` thread-local registry (Perl: `assignValue
  /// 'BIBENTRY@<lc-key>'`), then returns the wrapper TeX that the
  /// digester will read back.
  pub fn to_tex(&mut self) -> Result<String, BibParseError> {
    if !self.parsed {
      self.parse_top_level()?;
    }
    for parsed in &self.entries {
      let mut entry = BibEntry::new(parsed.key.clone(), parsed.entry_type.clone());
      // Perl `Pre::BibTeX::Entry::new` stores both processed `fields`
      // (post-macro-expansion, delimiter-stripped) and `rawfields`
      // (verbatim source). The processed form is what the field
      // dispatcher consumes; the verbatim form survives only for
      // `\bib@@origbibentry`'s source-style re-rendering. The Rust
      // BibEntry's `raw_fields: Vec<(String,String)>` slot is the
      // dispatcher input and `pretty_print` source. Feed the
      // *processed* value so dispatched handlers see macro-expanded
      // text (e.g. `tcs` → "Theoretical Computer Science").
      for (name, value) in &parsed.fields {
        entry.add_raw_field(name.clone(), value.clone());
      }
      register_entry(&parsed.key, entry);
    }
    let mut out = String::new();
    for pre in &self.preamble {
      out.push_str(pre);
      out.push('\n');
    }
    out.push_str("\\begin{bibtex@bibliography}\n");
    for entry in &self.entries {
      out.push_str("\\ProcessBibTeXEntry{");
      out.push_str(&entry.key);
      out.push_str("}\n");
    }
    out.push_str("\\end{bibtex@bibliography}");
    Ok(out)
  }
}

// =============================================================================
// Pure helpers

fn is_bib_name_or_noise(c: char) -> bool {
  // BIBNAME: [a-zA-Z0-9]
  if c.is_ascii_alphanumeric() {
    return true;
  }
  // ...plus `\`. Perl excludes it ON PURPOSE (`BibTeX.pm` L215-217: "Especially
  // \", which BibTeX allows, but it throws us off (semiverbatim vs verbatim)
  // when we store the bibentries before digesting the key!"). But excluding it
  // does not avoid the hazard, it just loses the entry: `@misc{apple\_rl,`
  // ends the key at the backslash and the whole entry is dropped, and a bogus
  // `\author={...}` field name kills its entry outright. BibTeX accepts both —
  // it takes `apple\_rl` as the key verbatim, and treats `\author` as an
  // unknown field (hence its "empty author" warning), keeping the entry. We do
  // the same: the key is matched byte-for-byte against the `\cite`, which
  // carries the identical bytes, and an unknown field name is simply never
  // consumed by any downstream handler.
  // OXIDIZED_DESIGN #60. Witnesses 2605.14212 (`apple\_rl`), 2605.06974
  // (`\author=`/`\title=` field names).
  if c == '\\' {
    return true;
  }
  // ...plus any non-ASCII. Perl's class is the literal `a-zA-Z0-9`
  // (`BibTeX.pm` L221), so it stops dead at the first accent — a Zotero-style
  // key like `alvarado-leañosLasing2022` yields `Expected ","` and the entry
  // is lost. BibTeX itself is byte-oriented and accepts such keys (verified:
  // `bibtex` 0.99d cites `alvarado-leañosLasing2022` with only a benign
  // "empty journal" warning), and the `\cite` in the .tex carries the same
  // bytes, so the two match. Non-ASCII is never a BibTeX *delimiter*, so
  // admitting it here cannot swallow structure.
  // OXIDIZED_DESIGN #60. Witnesses 2605.28695 (`ñ`), 2605.00121 (a stray
  // U+FE0F VARIATION SELECTOR-16 the author typed into the key).
  if !c.is_ascii() {
    return true;
  }
  // BIBNOISE: . + - * / ^ _ : ; @ ` ? ! ~ | < > $ [ ]
  matches!(
    c,
    '.'
      | '+'
      | '-'
      | '*'
      | '/'
      | '^'
      | '_'
      | ':'
      | ';'
      | '@'
      | '`'
      | '?'
      | '!'
      | '~'
      | '|'
      | '<'
      | '>'
      | '$'
      | '['
      | ']'
  )
}

/// Compute the byte-slice that was consumed from `before` to produce
/// `after`. Used by `parse_value` to recover the raw source of a
/// just-parsed string without re-parsing.
///
/// `after` must be a *suffix* of `before` — the parser only ever pops
/// a prefix off `line`, and `raw_witness` mirrors `extend_line`'s
/// appends so that holds across multi-line values too. Slicing at
/// `before.len() - after.len()` is then a char boundary by
/// construction. Callers that break the suffix invariant get an empty
/// raw rather than a panic or a torn string: byte arithmetic on
/// unrelated strings used to slice mid-codepoint and panic on any
/// non-ASCII `.bib` (witnesses 2605.02644, 2605.15313).
fn consumed_diff(before: &str, after: &str) -> String {
  match before.len().checked_sub(after.len()) {
    Some(idx) if before.is_char_boundary(idx) && &before[idx..] == after => {
      before[..idx].to_string()
    },
    _ => {
      debug_assert!(
        false,
        "consumed_diff: {after:?} is not a suffix of {before:?}"
      );
      String::new()
    },
  }
}

/// Find the byte index immediately after the closing brace of a
/// `{`-prefixed balanced-brace group in `s`. Returns `None` if no
/// balanced close exists in `s`.
///
/// Braces are counted **literally**: a backslash does NOT escape them.
/// This is BibTeX's own rule (`bibtex.web`'s brace-depth scan knows
/// nothing about `\`), and it is deliberately NOT Perl's — Perl uses
/// `Text::Balanced::extract_bracketed($line, '{}')`, which treats `\}`
/// as escaped and so returns undef for a title like
/// `"...boldsymbol\{Q\}..."`. Perl then extends line after line to EOF
/// and abandons the whole file, losing every remaining entry.
///
/// Ground truth is the real tool: `bibtex` 0.99d parses that same entry
/// with only a benign "empty journal" warning, so the references exist
/// in the author's PDF. OXIDIZED_DESIGN #60, KNOWN_PERL_ERRORS #51.
/// Witness 2605.00264 (`\{Q\}` in `chen2017ucb`): 1144/1169 entries
/// parsed before, 1169 after.
fn find_balanced_brace_end(s: &str) -> Option<usize> {
  let bytes = s.as_bytes();
  if bytes.first() != Some(&b'{') {
    return None;
  }
  let mut depth: i32 = 0;
  for (i, b) in bytes.iter().enumerate() {
    match b {
      b'{' => depth += 1,
      b'}' => {
        depth -= 1;
        if depth == 0 {
          return Some(i + 1);
        }
      },
      _ => {},
    }
  }
  None
}

// =============================================================================
// Tests

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(s: &str) -> PreBibTeX {
    let mut p = PreBibTeX::new_from_string(s);
    p.parse_top_level().expect("parse_top_level");
    p
  }

  fn raw_field<'a>(e: &'a ParsedEntry, name: &str) -> &'a str {
    e.raw_fields
      .iter()
      .find(|(n, _)| n == name)
      .map(|(_, v)| v.as_str())
      .unwrap_or_else(|| panic!("no raw field {name:?} in {:?}", e.raw_fields))
  }

  /// A value spanning physical lines whose first line holds a
  /// multi-byte char used to slice mid-codepoint and panic:
  /// `end byte index 3 is not a char boundary` — every non-ASCII
  /// `.bib` with a wrapped field. Witnesses: 2605.02644, 2605.15313
  /// (21 papers in the 2605 rerun).
  #[test]
  fn multiline_value_with_utf8_does_not_panic() {
    let p = parse("@article{k,\n  title = {Müller and Sons\n    and More}, year = {20},\n}\n");
    assert_eq!(p.entries.len(), 1);
    let e = &p.entries[0];
    assert_eq!(
      e.fields
        .iter()
        .find(|(n, _)| n == "title")
        .map(|(_, v)| v.as_str()),
      Some("Müller and Sons\n    and More")
    );
  }

  /// `\{`/`\}` inside a value do NOT escape the brace count — BibTeX's own
  /// rule. Perl's `Text::Balanced` disagrees, fails to extract, extends to
  /// EOF and abandons the rest of the file; real `bibtex` 0.99d parses this
  /// entry fine (only "empty journal"), so the reference is in the PDF.
  /// Witness 2605.00264 `chen2017ucb` — 25 entries were lost this way.
  #[test]
  fn escaped_braces_do_not_escape_the_brace_count() {
    let p = parse(concat!(
      "@article{chen2017ucb,\n",
      "    title = \"{UCB} via {\\textdollar}{\\textbackslash}boldsymbol\\{Q\\}{\\textdollar}-Ensembles\"\n",
      "}\n\n",
      "@article{after2018,\n  title = \"An entry that follows\"\n}\n"
    ));
    // BOTH entries survive: the escaped-brace title parses, and nothing ran
    // away to EOF and swallowed its successor.
    let keys: Vec<&str> = p.entries.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys, vec!["chen2017ucb", "after2018"], "entries: {keys:?}");
  }

  /// A non-ASCII cite key (Zotero writes these constantly) must not end the
  /// key. Perl's class is the literal `a-zA-Z0-9`, so it stops at the accent
  /// and reports `Expected ","`; real `bibtex` 0.99d accepts the key, and the
  /// `\cite` carries the same bytes. Witness 2605.28695.
  #[test]
  fn non_ascii_cite_key_is_not_truncated() {
    let p = parse(concat!(
      "@article{alvarado-lea\u{f1}osLasing2022,\n  title = {Lasing},\n  year = {2022}\n}\n\n",
      "@article{after2018,\n  title = {Follows}\n}\n"
    ));
    let keys: Vec<&str> = p.entries.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(
      keys,
      vec!["alvarado-lea\u{f1}osLasing2022", "after2018"],
      "entries: {keys:?}"
    );
  }

  /// A backslash inside a cite key must not end the key: `bibtex` 0.99d emits
  /// `\bibitem{apple\_rl}`, so the entry exists. Perl excludes `\` from the
  /// name class deliberately, which merely loses the entry instead.
  ///
  /// SCOPE — this pins the PARSER only. Perl's stated worry
  /// (`BibTeX.pm` L215-217, "semiverbatim vs verbatim ... before digesting the
  /// key") is REAL and still unresolved downstream: `\cite{apple\_rl}` digests
  /// to `bibrefs="apple_rl"` while the entry keeps the verbatim key
  /// `apple\_rl`, so the citation still dangles (`Missing bibkeys: apple_rl`).
  /// Admitting `\` is still strictly better — the entry survives rather than
  /// being dropped, and the sibling `\author=` case (below) resolves fully —
  /// but making such a cite LINK needs key normalisation at the
  /// `\ProcessBibTeXEntry` seam. Witness 2605.14212.
  #[test]
  fn backslash_in_cite_key_is_not_truncated() {
    let p = parse(concat!(
      "@misc{apple\\_rl,\n  title={Reinforcement Learning},\n  year={2024}\n}\n\n",
      "@article{after2018,\n  title = {Follows}\n}\n"
    ));
    let keys: Vec<&str> = p.entries.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys, vec!["apple\\_rl", "after2018"], "entries: {keys:?}");
  }

  /// A bogus `\author={...}` field name must not cost the whole entry.
  /// `bibtex` 0.99d keeps `DrmotaTichy2006` and merely warns "empty author
  /// and editor" — i.e. the unrecognised field is ignored, the entry stands.
  /// Witness 2605.06974.
  #[test]
  fn backslash_field_name_does_not_drop_the_entry() {
    let p = parse(concat!(
      "@book {DrmotaTichy2006,\n",
      "\\author={Drmota, M.},\n",
      "\\title={Sequences and applications},\n",
      "year={2006}\n}\n"
    ));
    let keys: Vec<&str> = p.entries.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys, vec!["DrmotaTichy2006"], "entries: {keys:?}");
    // `year` still lands as a real field; the bogus ones do not masquerade
    // as `author`/`title` (bibtex reports those as EMPTY).
    let e = &p.entries[0];
    assert_eq!(
      e.fields
        .iter()
        .find(|(n, _)| n == "year")
        .map(|(_, v)| v.as_str()),
      Some("2006")
    );
    assert!(
      e.fields.iter().all(|(n, _)| n != "author" && n != "title"),
      "a `\\author=` field must not be read as `author`: {:?}",
      e.fields
    );
  }

  /// A bare `@Comment` banner (no braces/quotes) is legal junk that BibTeX
  /// ignores. Demanding a delimited string made it an Err, which
  /// `parse_top_level` then reported as `bibtex:unbalanced ... resyncing` —
  /// a warning that claims an entry was LOST when none was (`skip_junk`
  /// recovers the next `@` regardless). Witness 2605.06974: 26 banners, 26
  /// false alarms, 0 real losses.
  ///
  /// Asserted at the `parse_comment` layer because that is exactly what
  /// changed: at the `parse_top_level` layer the resync masks it, so an
  /// entries-based assertion would pass either way (it did).
  #[test]
  fn undelimited_comment_banner_is_not_an_error() {
    let mut p = PreBibTeX::new_from_string(" -----------AAAAAAA-------------\n");
    assert!(
      p.parse_comment().is_ok(),
      "a bare `@Comment` banner must not raise a parse error"
    );
    // A properly delimited comment still parses (and is discarded).
    let mut q = PreBibTeX::new_from_string(" {a delimited comment}\n");
    assert!(q.parse_comment().is_ok());
  }

  /// The same defect without the panic: on an all-ASCII wrapped value
  /// the bogus arithmetic still landed on a char boundary and silently
  /// returned a *truncated* raw. Guards the torn-string half.
  #[test]
  fn multiline_value_raw_keeps_the_whole_consumed_span() {
    let p = parse("@article{k,\n  title = {A title that\n    wraps},\n}\n");
    assert_eq!(
      raw_field(&p.entries[0], "title"),
      "{A title that\n    wraps}"
    );
  }

  #[test]
  fn simple_article() {
    let p = parse(
      r#"
@article{Smith2020,
  author = {John Smith},
  title  = {On Examples},
  journal = {JMP},
  year   = 2020
}
"#,
    );
    assert_eq!(p.entries.len(), 1);
    let e = &p.entries[0];
    assert_eq!(e.entry_type, "article");
    assert_eq!(e.key, "Smith2020");
    let fields: Vec<(&str, &str)> = e
      .fields
      .iter()
      .map(|(k, v)| (k.as_str(), v.as_str()))
      .collect();
    assert_eq!(fields, vec![
      ("author", "John Smith"),
      ("title", "On Examples"),
      ("journal", "JMP"),
      ("year", "2020"),
    ]);
  }

  #[test]
  fn preamble_block() {
    let p = parse(
      r#"
@preamble{ "\newcommand{\noopsort}[1]{}" }
@article{a,k={v}}
"#,
    );
    assert_eq!(p.preamble, vec![r"\newcommand{\noopsort}[1]{}".to_string()]);
    assert_eq!(p.entries.len(), 1);
  }

  #[test]
  fn string_macro_then_use() {
    let p = parse(
      r#"
@string{tcs = "Theoretical CS"}
@article{a, journal = tcs, year = 2024}
"#,
    );
    let e = &p.entries[0];
    assert_eq!(
      e.fields[0],
      ("journal".to_string(), "Theoretical CS".to_string())
    );
    assert_eq!(e.fields[1], ("year".to_string(), "2024".to_string()));
    // Raw side keeps the macro NAME, not the expansion.
    assert_eq!(e.raw_fields[0], ("journal".to_string(), "tcs".to_string()));
  }

  #[test]
  fn default_macro_jan() {
    let p = parse(
      r#"
@article{a, month = jan}
"#,
    );
    assert_eq!(p.entries[0].fields[0].1, "January");
  }

  #[test]
  fn quoted_string_with_braces_inside() {
    let p = parse(
      r#"
@article{a, title = "On {Theorems} and proofs"}
"#,
    );
    assert_eq!(p.entries[0].fields[0].1, "On {Theorems} and proofs");
  }

  #[test]
  fn brace_string_with_nested_braces() {
    let p = parse(
      r#"
@article{a, title = {On {Hot} {Topics}}}
"#,
    );
    assert_eq!(p.entries[0].fields[0].1, "On {Hot} {Topics}");
  }

  #[test]
  fn value_concat_with_hash() {
    // Per Perl L272-273, `parseString` trims surrounding whitespace
    // off each delimited string before storing. So `"Hello, "` is
    // saved as `"Hello,"` in the macro table, and the `#`-concat
    // produces `"Hello,world!"`. We mirror that behaviour.
    let p = parse(
      r#"
@string{first = "Hello, "}
@article{a, title = first # "world!"}
"#,
    );
    assert_eq!(p.entries[0].fields[0].1, "Hello,world!");
  }

  #[test]
  fn case_preservation_of_key_but_lowercase_field_and_type() {
    let p = parse(
      r#"
@Article{MyKey, Author = {X}, TITLE = {y}}
"#,
    );
    let e = &p.entries[0];
    assert_eq!(e.entry_type, "article");
    assert_eq!(e.key, "MyKey");
    let names: Vec<&str> = e.fields.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(names, vec!["author", "title"]);
  }

  #[test]
  fn skip_junk_before_at() {
    let p = parse(
      r#"This text should be discarded
even multi-line junk... @article{a, k = "v"}
"#,
    );
    assert_eq!(p.entries.len(), 1);
    assert_eq!(p.entries[0].key, "a");
  }

  #[test]
  fn paren_delimited_entry() {
    let p = parse(
      r#"
@article(a, k = "v")
"#,
    );
    assert_eq!(p.entries.len(), 1);
    assert_eq!(p.entries[0].key, "a");
  }

  #[test]
  fn trailing_comma_after_last_field() {
    let p = parse(
      r#"
@article{a, k1 = {v1}, k2 = {v2}, }
"#,
    );
    let e = &p.entries[0];
    assert_eq!(e.fields.len(), 2);
    assert_eq!(e.fields[0], ("k1".to_string(), "v1".to_string()));
    assert_eq!(e.fields[1], ("k2".to_string(), "v2".to_string()));
  }

  #[test]
  fn multiline_brace_value() {
    let p = parse(
      r#"
@article{a, title = {Some
multi-line
value} }
"#,
    );
    let v = &p.entries[0].fields[0].1;
    assert!(v.contains("multi-line"));
  }

  #[test]
  fn comment_entry_is_skipped() {
    let p = parse(
      r#"
@comment{this is ignored}
@article{a, k = {v}}
"#,
    );
    assert_eq!(p.entries.len(), 1);
  }

  #[test]
  fn multiple_string_definitions_in_one_block() {
    let p = parse(
      r#"
@string{a = "X", b = "Y"}
@article{e, f1 = a, f2 = b}
"#,
    );
    let e = &p.entries[0];
    assert_eq!(e.fields, vec![
      ("f1".to_string(), "X".to_string()),
      ("f2".to_string(), "Y".to_string()),
    ]);
  }

  #[test]
  fn to_tex_emits_processbibtexentry_for_each_entry() {
    crate::bibtex::reset();
    let mut p = PreBibTeX::new_from_string(
      r#"
@article{Smith2020, title = {T}}
@book{Doe1999, title = {B}}
"#,
    );
    let tex = p.to_tex().expect("to_tex");
    assert!(tex.contains("\\begin{bibtex@bibliography}"));
    assert!(tex.contains("\\ProcessBibTeXEntry{Smith2020}"));
    assert!(tex.contains("\\ProcessBibTeXEntry{Doe1999}"));
    assert!(tex.contains("\\end{bibtex@bibliography}"));
    // Registry side-effect:
    assert!(crate::bibtex::lookup_entry("Smith2020").is_some());
    assert!(crate::bibtex::lookup_entry("Doe1999").is_some());
    crate::bibtex::reset();
  }

  #[test]
  fn to_tex_includes_preamble() {
    crate::bibtex::reset();
    let mut p = PreBibTeX::new_from_string(
      r#"@preamble{ "\providecommand{\noop}[1]{}" }
@article{x, k = {v}}
"#,
    );
    let tex = p.to_tex().unwrap();
    assert!(tex.starts_with("\\providecommand{\\noop}[1]{}"));
    crate::bibtex::reset();
  }

  // Internal helper tests.

  #[test]
  fn balanced_brace_finder() {
    assert_eq!(find_balanced_brace_end("{abc}xyz"), Some(5));
    assert_eq!(find_balanced_brace_end("{a{b}c}d"), Some(7));
    // A backslash does NOT escape the brace count (BibTeX's rule), so the
    // group closes at the FIRST `}` — this asserted Some(6) while we mirrored
    // Perl's Text::Balanced, which is what lost real entries (#58).
    assert_eq!(find_balanced_brace_end(r"{a\}b}"), Some(4));
    assert_eq!(find_balanced_brace_end("{abc"), None);
    assert_eq!(find_balanced_brace_end("noleadingbrace"), None);
  }

  /// Witness 2605.00490: a JabRef `.bib` self-declaring `% Encoding: Cp1252`.
  /// `read_to_string` rejected the whole file ("stream did not contain valid
  /// UTF-8"), so the paper rendered a References section with zero entries and
  /// NO error — a silent, total loss. Real `bibtex` 0.99d is 8-bit clean, and
  /// Perl passes the raw bytes through (`Mouth.pm` L75-80).
  #[test]
  fn non_utf8_bib_file_is_read_not_rejected() {
    // `\xe9` is `é` in Cp1252/Latin-1 — a lone continuation byte that is
    // invalid UTF-8, exactly what a JabRef-era file carries in an author name.
    let mut bytes = b"@article{Cafe2020,\n  author = {Caf".to_vec();
    bytes.push(0xe9);
    bytes.extend_from_slice(b" and R\xe9mi},\n  year = {2020}\n}\n");

    let path = std::env::temp_dir().join("latexml_oxide_cp1252_witness.bib");
    std::fs::write(&path, &bytes).expect("write fixture");
    let mut p = PreBibTeX::new_from_file(path.to_str().unwrap()).expect("non-UTF-8 .bib must read");
    p.parse_top_level().expect("parse_top_level");
    let _ = std::fs::remove_file(&path);

    let keys: Vec<&str> = p.entries.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys, vec!["Cafe2020"], "entries: {keys:?}");
    // The accented bytes survive as the Latin-1 characters they encode,
    // rather than being replaced or dropped.
    let author = p.entries[0]
      .fields
      .iter()
      .find(|(n, _)| n == "author")
      .map(|(_, v)| v.as_str())
      .unwrap_or_default();
    assert_eq!(author, "Café and Rémi", "author: {author:?}");
  }

  /// The Latin-1 fallback is per LINE, so one stray byte in a mostly-UTF-8
  /// `.bib` must not mojibake the correctly-encoded names around it.
  #[test]
  fn one_bad_byte_does_not_mojibake_the_rest_of_the_file() {
    let mut bytes = b"@article{Mixed2021,\n  author = {Zo\xebe},\n".to_vec();
    // A properly UTF-8-encoded name on its own line: it must survive verbatim.
    bytes.extend_from_slice("  title = {Ünicode Bewahren},\n  year = {2021}\n}\n".as_bytes());

    let path = std::env::temp_dir().join("latexml_oxide_mixed_encoding_witness.bib");
    std::fs::write(&path, &bytes).expect("write fixture");
    let mut p = PreBibTeX::new_from_file(path.to_str().unwrap()).expect("mixed-encoding .bib");
    p.parse_top_level().expect("parse_top_level");
    let _ = std::fs::remove_file(&path);

    let field = |n: &str| {
      p.entries[0]
        .fields
        .iter()
        .find(|(f, _)| f == n)
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
    };
    // The bad line decodes as Latin-1 ...
    assert_eq!(field("author"), "Zoëe", "author: {:?}", field("author"));
    // ... while the valid-UTF-8 line is untouched (a whole-buffer Latin-1
    // fallback would have turned this into "Ãnicode").
    assert_eq!(
      field("title"),
      "Ünicode Bewahren",
      "title: {:?}",
      field("title")
    );
  }
}

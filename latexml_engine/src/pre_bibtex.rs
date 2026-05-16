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
//! - `parse_entry_type` / `parse_entry_name` / `parse_field_name`
//!   character classes are byte-for-byte the Perl regex
//!   `[BIBNAME_re ++ BIBNOISE_re]` (Perl L221-222). Entry-name
//!   additionally allows `"#%&'()={` (Perl L233); field-name
//!   additionally allows `&` (Perl L238). Entry-type and field-name
//!   are lower-cased; entry-name preserves case at the parse layer
//!   (the registry then normalises via `register_entry`).
//!
//! - `parse_string` reads `"…"` (allowing balanced `{…}` inside that
//!   defeat the closing quote) OR `{…}` (balanced braces, outer
//!   braces stripped) — exact match for Perl L252-274.
//!
//! - `parse_value` concatenates simple values separated by `#`. A
//!   simple value is either a delimited string or a name; a name
//!   matching `^\d+$` is taken literally, otherwise looked up in the
//!   macro table.
//!
//! - `skip_junk` is greedy: everything up to the next `@` is
//!   discarded (Perl L335-346: "anything until @ as an implied
//!   comment").

use std::collections::HashMap;

use latexml_core::s;

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
    ("ieeetcad", "IEEE Transactions on Computer-Aided Design of Integrated Circuits"),
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
    ("toplas", "ACM Transactions on Programming Languages and Systems"),
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
  pub source:    Option<String>,
  pub file_label: String,
  lines:         Vec<String>,
  line:          String,
  lineno:        usize,
  pub preamble:  Vec<String>,
  pub entries:   Vec<ParsedEntry>,
  macros:        HashMap<String, String>,
  parsed:        bool,
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
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    let first = if lines.is_empty() {
      String::new()
    } else {
      lines.remove(0)
    };
    Self {
      source:      None,
      file_label:  s!("<anonymous>"),
      lines,
      line:        first,
      lineno:      1,
      preamble:    Vec::new(),
      entries:     Vec::new(),
      macros:      default_macros(),
      parsed:      false,
    }
  }

  /// Perl `newFromFile` (L60-75). Reads the file at `path` directly;
  /// search-path resolution is the caller's responsibility (we do not
  /// reach into `state` here so the parser stays unit-testable).
  pub fn new_from_file(path: &str) -> std::io::Result<Self> {
    let content = std::fs::read_to_string(path)?;
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
    s!("Bibliography[{}]", self.source.as_deref().unwrap_or("<Unknown>"))
  }

  /// Perl `getLocator` (L125-129). We don't model the full
  /// `LaTeXML::Common::Locator` here; just `(source, lineno)`.
  pub fn locator(&self) -> (Option<&str>, usize) {
    (self.source.as_deref(), self.lineno)
  }

  // ---- top-level parse ------------------------------------------------------

  /// Perl `parseTopLevel` (L138-151).
  pub fn parse_top_level(&mut self) -> Result<(), BibParseError> {
    while self.skip_junk() {
      let typ = match self.parse_entry_type() {
        Some(t) => t,
        None => continue,
      };
      match typ.as_str() {
        "preamble" => self.parse_preamble()?,
        "string" => self.parse_macro()?,
        "comment" => self.parse_comment()?,
        _ => self.parse_entry(&typ)?,
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

  /// Perl `parseComment` (L177-181). The value is parsed and
  /// discarded.
  fn parse_comment(&mut self) -> Result<(), BibParseError> {
    let _ = self.parse_string()?;
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
    if let Some(c) = head {
      if delims.contains(c) {
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
    let next = self.lines.remove(0);
    // Perl `<$BIB>` lines keep the trailing newline; `split /\n/` on a
    // string does not. Re-insert one to preserve line semantics so
    // that a `%`-comment inside a continued value still terminates on
    // the next physical line (BibTeX itself doesn't treat `%` as a
    // comment but other downstream digesters might).
    self.line.push('\n');
    self.line.push_str(&next);
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
        let raw_start_line = self.line.clone();
        let s = self.parse_string()?;
        let raw_consumed = consumed_diff(&raw_start_line, &self.line);
        raw.push_str(&raw_consumed);
        value.push_str(&s);
      } else if let Some(name) = self.parse_field_name() {
        let is_digits = !name.is_empty() && name.chars().all(|c| c.is_ascii_digit());
        let resolved = if is_digits {
          name.clone()
        } else {
          self
            .macros
            .get(&name)
            .cloned()
            .unwrap_or_else(|| {
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
      self.line = self.lines.remove(0);
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
      self.line = self.lines.remove(0);
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
  // BIBNOISE: . + - * / ^ _ : ; @ ` ? ! ~ | < > $ [ ]
  matches!(
    c,
    '.' | '+' | '-' | '*' | '/' | '^' | '_' | ':' | ';' | '@' | '`' | '?' | '!' | '~' | '|' |
    '<' | '>' | '$' | '[' | ']'
  )
}

/// Compute the byte-slice that was consumed from `before` to produce
/// `after`. Used by `parse_value` to recover the raw source of a
/// just-parsed string without re-parsing.
fn consumed_diff(before: &str, after: &str) -> String {
  if before.len() < after.len() {
    return String::new();
  }
  before[..before.len() - after.len()].to_string()
}

/// Find the byte index immediately after the closing brace of a
/// `{`-prefixed balanced-brace group in `s`. Returns `None` if no
/// balanced close exists in `s`.
///
/// Honors backslash-escaped `\{` and `\}` as a single token, matching
/// the Perl `Text::Balanced::extract_bracketed($s, '{}')` semantics
/// for the simple pair specification.
fn find_balanced_brace_end(s: &str) -> Option<usize> {
  let bytes = s.as_bytes();
  if bytes.first() != Some(&b'{') {
    return None;
  }
  let mut depth: i32 = 0;
  let mut i = 0;
  while i < bytes.len() {
    match bytes[i] {
      b'\\' if i + 1 < bytes.len() => {
        // Skip the escaped char
        i += 2;
        continue;
      },
      b'{' => depth += 1,
      b'}' => {
        depth -= 1;
        if depth == 0 {
          return Some(i + 1);
        }
      },
      _ => {},
    }
    i += 1;
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

  #[test]
  fn simple_article() {
    let p = parse(r#"
@article{Smith2020,
  author = {John Smith},
  title  = {On Examples},
  journal = {JMP},
  year   = 2020
}
"#);
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
    let p = parse(r#"
@preamble{ "\newcommand{\noopsort}[1]{}" }
@article{a,k={v}}
"#);
    assert_eq!(p.preamble, vec![r"\newcommand{\noopsort}[1]{}".to_string()]);
    assert_eq!(p.entries.len(), 1);
  }

  #[test]
  fn string_macro_then_use() {
    let p = parse(r#"
@string{tcs = "Theoretical CS"}
@article{a, journal = tcs, year = 2024}
"#);
    let e = &p.entries[0];
    assert_eq!(e.fields[0], ("journal".to_string(), "Theoretical CS".to_string()));
    assert_eq!(e.fields[1], ("year".to_string(), "2024".to_string()));
    // Raw side keeps the macro NAME, not the expansion.
    assert_eq!(e.raw_fields[0], ("journal".to_string(), "tcs".to_string()));
  }

  #[test]
  fn default_macro_jan() {
    let p = parse(r#"
@article{a, month = jan}
"#);
    assert_eq!(p.entries[0].fields[0].1, "January");
  }

  #[test]
  fn quoted_string_with_braces_inside() {
    let p = parse(r#"
@article{a, title = "On {Theorems} and proofs"}
"#);
    assert_eq!(p.entries[0].fields[0].1, "On {Theorems} and proofs");
  }

  #[test]
  fn brace_string_with_nested_braces() {
    let p = parse(r#"
@article{a, title = {On {Hot} {Topics}}}
"#);
    assert_eq!(p.entries[0].fields[0].1, "On {Hot} {Topics}");
  }

  #[test]
  fn value_concat_with_hash() {
    // Per Perl L272-273, `parseString` trims surrounding whitespace
    // off each delimited string before storing. So `"Hello, "` is
    // saved as `"Hello,"` in the macro table, and the `#`-concat
    // produces `"Hello,world!"`. We mirror that behaviour.
    let p = parse(r#"
@string{first = "Hello, "}
@article{a, title = first # "world!"}
"#);
    assert_eq!(p.entries[0].fields[0].1, "Hello,world!");
  }

  #[test]
  fn case_preservation_of_key_but_lowercase_field_and_type() {
    let p = parse(r#"
@Article{MyKey, Author = {X}, TITLE = {y}}
"#);
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
    let p = parse(r#"
@article(a, k = "v")
"#);
    assert_eq!(p.entries.len(), 1);
    assert_eq!(p.entries[0].key, "a");
  }

  #[test]
  fn trailing_comma_after_last_field() {
    let p = parse(r#"
@article{a, k1 = {v1}, k2 = {v2}, }
"#);
    let e = &p.entries[0];
    assert_eq!(e.fields.len(), 2);
    assert_eq!(e.fields[0], ("k1".to_string(), "v1".to_string()));
    assert_eq!(e.fields[1], ("k2".to_string(), "v2".to_string()));
  }

  #[test]
  fn multiline_brace_value() {
    let p = parse(r#"
@article{a, title = {Some
multi-line
value} }
"#);
    let v = &p.entries[0].fields[0].1;
    assert!(v.contains("multi-line"));
  }

  #[test]
  fn comment_entry_is_skipped() {
    let p = parse(r#"
@comment{this is ignored}
@article{a, k = {v}}
"#);
    assert_eq!(p.entries.len(), 1);
  }

  #[test]
  fn multiple_string_definitions_in_one_block() {
    let p = parse(r#"
@string{a = "X", b = "Y"}
@article{e, f1 = a, f2 = b}
"#);
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
    assert_eq!(find_balanced_brace_end(r"{a\}b}"), Some(6));
    assert_eq!(find_balanced_brace_end("{abc"), None);
    assert_eq!(find_balanced_brace_end("noleadingbrace"), None);
  }
}

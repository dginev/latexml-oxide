use core::ops::RangeBounds;
use std::{
  collections::VecDeque,
  fmt,
  fs::File,
  io,
  io::{BufReader, prelude::*},
  str,
};

// TODO:
// use encoding::all::ISO_8859_1;
// use encoding::{EncoderTrap, Encoding};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
  common::{
    arena::SymStr,
    error::{emit_warn, *},
    locator::Locator,
    numeric_ops::NumericOps,
    object::Object,
  },
  state::*,
  token::*,
  tokens::{NO_TOKENS, TeXString, Tokens},
  util::pathname,
};

static TRAILING_SPACE_CHARS: Lazy<Regex> = Lazy::new(|| Regex::new("(?s) +$").unwrap());

const READLINE_PROGRESS_QUANTUM: usize = 25;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum FoodType {
  File,
  // Binding,
  HTTP,
  HTTPS,
  Literal,
}

impl FoodType {
  /// TODO: Should be a From trait implementation, but am not allowed due to both &str and Option
  /// being external. Argh.
  pub fn opt_from_str(text: &str) -> Option<FoodType> {
    use self::FoodType::*;
    match text.to_lowercase().as_str() {
      "file" => Some(File),
      // "binding" => Some(Binding),
      "http" => Some(HTTP),
      "https" => Some(HTTPS),
      "literal" => Some(Literal),
      _ => None,
    }
  }
}

static LINEBREAK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s:\r\n?)|(?s:\n)").unwrap());
// LOWERHEX_REGEX removed — replaced with direct matches!() check in tex_hex_caret path.
static _SANITIZE_LINE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"((\\ )*)\s*$").unwrap());

#[derive(Debug, Default)]
pub struct MouthOptions {
  pub fordefinitions: bool,
  pub at_letter:      bool,
  pub notes:          bool,
  pub content:        Option<String>,
  pub foodtype:       Option<FoodType>,
  pub source:         Option<String>,
  pub shortsource:    Option<String>,
}

#[derive(Debug)]
pub struct Mouth {
  fordefinitions:         bool,
  at_letter:              bool,
  notes:                  bool,
  at_eof:                 bool,
  nchars:                 usize,
  colno:                  usize,
  lineno:                 usize,
  /// Source start `(lineno, colno)` of the most recently *begun* token,
  /// captured in `read_token` after inter-token skips and before the token's
  /// first char is consumed. Foundation for §1 accurate construct-start ranges
  /// (docs/performance/SOURCE_PROVENANCE.md). Semantically inert until consumed by a ranged
  /// locator; written unconditionally — two writes, below the hot-path noise
  /// floor and cheaper than a per-read flag check.
  last_token_start:       (usize, usize),
  foodtype:               FoodType,
  /// Read `% & #` as ordinary characters — not comment, alignment tab,
  /// parameter — for this mouth only. Set by `with_bib_data_literals()`; see
  /// that method for the BibTeX rationale, and for why `_` is NOT here.
  /// Deliberately a per-Mouth field rather than a State catcode assignment: a
  /// nested mouth (a `.sty` raw-load triggered from inside the text) is a
  /// separate object and keeps TeX's meanings, which a State-level assignment
  /// could not guarantee.
  bib_data_literals:      bool,
  saved_at_cc:            Option<Catcode>,
  saved_include_comments: Option<bool>,
  note_message:           Option<String>,
  source:                 String,
  /// `source` pre-interned at construction, so per-token / per-conditional
  /// locator building ([`Object::get_locator`], [`Mouth::get_locator_from_start`])
  /// is pure field copies instead of an interner probe over the path string.
  /// `source` is never mutated after construction, so the two cannot drift.
  source_sym:             SymStr,
  shortsource:            String,
  skipping_spaces:        bool,
  // pub handle : Option<File>,
  chars:                  VecDeque<char>,
  buffer:                 VecDeque<String>,
  raw_buffer:             VecDeque<Vec<u8>>,
  reader:                 Option<BufReader<File>>,
}

impl PartialEq for Mouth {
  fn eq(&self, other: &Mouth) -> bool { self.source == other.source }
}

impl Default for Mouth {
  fn default() -> Self {
    // Historically the source was `"Anonymous String {gid}"` with a
    // per-instance gid, which Locator::source then pinned into the arena.
    // The gid served no functional purpose and made every anonymous mouth
    // unique at the SymStr layer — fine for a handful of mouths, but
    // catastrophic when a runaway error-recovery path creates millions
    // (arxiv 1210.4211 under parallel load: 50M anonymous mouths saturated
    // the u32 interner offset). Collapsing onto a shared static label makes
    // the per-mouth cost arena-free, and the pin-count sentinel remains as
    // a symptom detector for the *actual* bug (something is still creating
    // 50M anonymous mouths — that's a runaway loop to track down, now with
    // the arena side-effect removed).
    Mouth {
      notes:                  false,
      note_message:           None,
      fordefinitions:         false,
      at_letter:              false,
      at_eof:                 false,
      skipping_spaces:        false,
      lineno:                 0,
      colno:                  0,
      last_token_start:       (0, 0),
      chars:                  VecDeque::new(),
      nchars:                 0,
      source:                 String::from("Anonymous String"),
      source_sym:             crate::pin!("Anonymous String"),
      shortsource:            s!("String"),
      // handle : None,
      foodtype:               FoodType::File,
      bib_data_literals:      false,
      saved_at_cc:            None,
      saved_include_comments: None,
      buffer:                 VecDeque::new(),
      raw_buffer:             VecDeque::new(),
      reader:                 None,
    }
  }
}

impl fmt::Display for Mouth {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "Mouth[{}]", self.source) }
}
impl Object for Mouth {
  fn stringify(&self) -> String { s!("Mouth[<string>{}x{}]", self.lineno, self.colno) }
  fn get_locator(&self) -> Option<Locator> {
    let (to_line, to_column) = (self.lineno, self.colno);
    let max_col = if self.nchars > 0 {
      self.nchars - 1
    } else {
      self.nchars
    }; // There is always a trailing EOL char, if any
    let (from_line, from_column) = if to_column > 0 && to_column >= max_col {
      (to_line, 0)
    } else {
      (to_line, to_column)
    };
    // Perl Mouth.pm L199 (#2671): columns in Locator are 1-indexed; the Mouth's
    // internal colno counter is 0-indexed (character array index), so we add 1
    // when producing the Locator for error-message display.
    // A Mouth always has a position, so this is always `Some`.
    Some(Locator::from_sym(
      self.source_sym,
      from_line as u32,
      (from_column + 1) as u32,
      to_line as u32,
      (to_column + 1) as u32,
    ))
  }
}

/// Decode raw input bytes as text when no encoding has been declared:
/// valid UTF-8 is taken as-is, anything else is a Latin-1 passthrough
/// (byte → char).
///
/// Mirrors Perl `Mouth.pm` L75-80: when `PERL_INPUT_ENCODING` is undef Perl
/// never decodes, so the bytes pass through untouched and the read cannot
/// fail. The point is that a non-UTF-8 file is **never lost** — only decoded
/// conservatively. `std::fs::read_to_string` gives the opposite behaviour
/// (hard error on the first stray byte), which silently cost witness
/// 2605.00490 its entire bibliography: a JabRef-written `.bib` self-declaring
/// `% Encoding: Cp1252`. Real `bibtex` 0.99d is 8-bit clean and reads it fine.
///
/// Latin-1 (rather than `from_utf8_lossy`) is the better fallback here
/// because it is lossless byte → char: legacy `.bib` files are overwhelmingly
/// Latin-1/Cp1252, whose accented names survive intact instead of collapsing
/// to U+FFFD.
///
/// The fallback is applied **per line**, not per buffer. `raw` is a single
/// line when the Mouth calls this, but a whole file when a `.bib` reader does,
/// and decoding a whole file as Latin-1 because of one stray byte would
/// mojibake every correctly-UTF-8-encoded name in it (`é` → `Ã©`). Per-line
/// keeps the damage to the offending line and matches the Mouth's own
/// granularity. The all-valid-UTF-8 case (the overwhelming majority) still
/// costs exactly one `from_utf8` SIMD validation of the whole buffer.
pub fn decode_input_bytes(raw: &[u8]) -> String {
  match str::from_utf8(raw) {
    Ok(s) => s.to_string(),
    Err(_) => {
      let mut out = String::with_capacity(raw.len());
      for (i, line) in raw.split(|&b| b == b'\n').enumerate() {
        if i > 0 {
          out.push('\n');
        }
        match str::from_utf8(line) {
          Ok(s) => out.push_str(s),
          Err(_) => out.extend(line.iter().map(|&b| b as char)),
        }
      }
      out
    },
  }
}

impl Mouth {
  // Factory method;
  // Create an appropriate Mouth
  // options are
  //  quiet,
  //  atletter,
  //  content
  //
  // DG: For now we are using a `foodtype` field instead of subclassing mouth, as it feels more
  // compact in this particular application     we're really looking at a unified Mouth
  // application logic, with a capacity of reading different kinds of sources
  pub fn create(source: &str, mut options: MouthOptions) -> Result<Self> {
    if let Some(content) = options.content.take() {
      // we've cached the content of this source
      let (_dir, name, ext) = pathname::split(source);
      options.source = Some(source.to_string());
      options.shortsource = Some(s!("{}.{}", name, ext));
      // Read-log: a named cached-content open (filecontents / LSP overlay).
      record_opened_source(crate::common::arena::pin(source));
      Mouth::new(&content, Some(options))
    } else if source.starts_with("literal:") {
      let source = source.replacen("literal:", "", 1);
      // we've supplied literal data
      options.source = None; // the source does not have a corresponding file name
      options.foodtype = FoodType::opt_from_str("literal");
      Mouth::new(&source, Some(options))
    } else if source.is_empty() {
      Mouth::new("", Some(options))
    } else {
      let (_dir, name, ext) = pathname::split(source);
      options.foodtype = FoodType::opt_from_str(&pathname::protocol(source));
      options.source = Some(source.to_string());
      if options.shortsource.is_none() {
        options.shortsource = Some(if ext.is_empty() {
          name
        } else {
          s!("{}.{}", name, ext)
        });
      }
      // Read-log: a named file open (recorded even when the open then
      // fails — a pinned-but-missing path that later APPEARS must read
      // as a dependency change).
      record_opened_source(crate::common::arena::pin(source));
      Mouth::new(source, Some(options))
    }
  }

  /// What kind of source feeds this mouth (file vs literal/string injection).
  pub fn foodtype(&self) -> FoodType { self.foodtype }

  pub fn new(text: &str, options: Option<MouthOptions>) -> Result<Self> {
    let mut mouth = match options {
      None => Mouth {
        foodtype: FoodType::Literal,
        ..Mouth::default()
      },
      Some(opts) => {
        let shortsource = opts.shortsource.unwrap_or_else(|| s!("String"));
        let source = opts.source.unwrap_or_default();
        Mouth {
          foodtype: opts.foodtype.unwrap_or(FoodType::Literal),
          fordefinitions: opts.fordefinitions,
          at_letter: opts.at_letter,
          notes: opts.notes,
          source_sym: crate::common::arena::pin(&source),
          source,
          shortsource,
          ..Mouth::default()
        }
      },
    };
    mouth.open(text)?;
    Ok(mouth)
  }

  /// Read `% & #` as ordinary characters (catcode 12) instead of comment,
  /// alignment tab and parameter, for the whole life of this mouth.
  ///
  /// **Treatment 1 of two** (see `OXIDIZED_DESIGN #74`): this is "be `bibtex`".
  /// BibTeX's lexer interprets only braces and the entry/field delimiters — it
  /// has no comment syntax inside an entry (`%` is significant only in the junk
  /// BETWEEN entries, `Pre::BibTeX::skipJunk`), no alignment and no parameters.
  /// So a field value it hands back is a string in which all three are ordinary
  /// characters: a percent-encoded URL, a publisher's name ("Taylor &
  /// Francis"), an issue number.
  ///
  /// Re-injected as TeX source (BibTeX.pool's `\bibentry@create`) under the
  /// default catcodes, each misfires: `%` (14) comments out the rest of its
  /// line — the field's own closing brace included — so the entry's group never
  /// closes; `&` (4) is a stray alignment tab and is dropped; `#` (6) reaches
  /// the Stomach as a parameter token. Reading the injected text with all three
  /// neutralized preserves the value BibTeX actually parsed, **without altering
  /// a byte of it**.
  ///
  /// **`_` is deliberately NOT in this set**, and the reason is the boundary
  /// between the two treatments. A catcode is decided at tokenization, before
  /// anything knows whether it is inside `$…$` — and a subscript in a `.bib`
  /// title's math (`title = {Bounds on $x_1+x_2$}`) is *legitimate TeX* that
  /// must keep working. `_` therefore belongs to treatment 2
  /// (`bibtex.rs::escape_bib_data_specials`), which walks the value and skips
  /// math spans. Measured: putting `_` here silently flattened every
  /// subscript in a bibliography title. The other three have no legitimate
  /// meaning inside a `.bib` field, in math or out.
  ///
  /// A `\catcode` in the injected text cannot do this job either: the catcode
  /// would still be a State assignment, so a raw `.sty` opened from inside a
  /// field handler would inherit it — and so would the document. Scoping to the
  /// Mouth keeps the rule attached to the *text that BibTeX lexed*, which is
  /// exactly where it belongs.
  ///
  /// Only the TeX-special meaning is removed: a character that has been given
  /// some other catcode (LETTER, say) keeps it. And `\%`, `\&`, `\#` still
  /// work, because the backslash is untouched.
  pub fn with_bib_data_literals(mut self) -> Self {
    self.bib_data_literals = true;
    self
  }

  pub fn get_source(&self) -> &str { &self.source }

  pub fn open(&mut self, content: &str) -> Result<()> {
    match self.foodtype {
      FoodType::File => self.open_file(content)?,
      FoodType::Literal => self.open_literal(content),
      FoodType::HTTP => self.open_http(content),
      FoodType::HTTPS => self.open_https(content),
    };
    self.initialize();
    Ok(())
  }

  fn open_file(&mut self, pathname: &str) -> Result<()> {
    if self.foodtype == FoodType::File {
      // Perl: check readable, then check binary (non-empty), then open
      let metadata = std::fs::metadata(pathname);
      match &metadata {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
          fatal!(Mouth, MissingFile, s!("Can't find file {}", pathname));
        },
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
          Error!(
            "I/O",
            "unreadable",
            s!("File {} is not readable. Ignoring.", pathname),
            "",
            "",
            self.get_location()
          );
          return Ok(());
        },
        Err(e) => {
          return Err(io::Error::new(e.kind(), e.to_string()).into());
        },
        Ok(meta) => {
          // Every opened source file raises the runaway-token backstop in
          // proportion (see `gullet::scale_token_limit_to_source`).
          crate::gullet::scale_token_limit_to_source(meta.len() as usize);
          // Check for binary file (non-empty and appears binary)
          // Perl's -B heuristic: check first block for high proportion of non-text bytes
          if meta.len() > 0
            && let Ok(mut f) = File::open(pathname)
          {
            let mut buf = [0u8; 512];
            if let Ok(n) = f.read(&mut buf)
              && n > 0
            {
              let non_text = buf[..n]
                .iter()
                .filter(|&&b| {
                  b == 0 || (b < 0x20 && b != b'\n' && b != b'\r' && b != b'\t' && b != 0x1b)
                })
                .count();
              if non_text * 3 > n {
                // High ratio of non-text bytes — likely binary
                Error!(
                  "invalid",
                  "binary",
                  s!("Input file {} appears to be binary. Ignoring.", pathname),
                  "",
                  "",
                  self.get_location()
                );
                return Ok(());
              }
            }
          }
        },
      }
      let f = match File::open(pathname) {
        Ok(f) => f,
        Err(e) => {
          Error!(
            "I/O",
            "open",
            s!("Can't open {} for reading: {}", pathname, e),
            "",
            "",
            self.get_location()
          );
          return Err(e.into());
        },
      };
      let reader = BufReader::new(f);
      self.reader = Some(reader);
      self.buffer = VecDeque::new();
      self.raw_buffer = VecDeque::new();
    }
    Ok(())
  }
  fn open_literal(&mut self, content: &str) { self.buffer = Mouth::split_lines(content); }
  fn open_http(&mut self, url: &str) {
    emit_warn(
      "unsupported",
      "http_input",
      &format!("HTTP input not supported: {url}"),
    );
  }
  fn open_https(&mut self, url: &str) {
    emit_warn(
      "unsupported",
      "http_input",
      &format!("HTTPS input not supported: {url}"),
    );
  }
  // fn open_binding(&mut self, _content: &str) {}

  fn initialize(&mut self) {
    self.note_message = if self.notes {
      let source = if !self.source.is_empty() {
        &self.source
      } else {
        "Anonymous String"
      };
      let kind = if self.fordefinitions {
        "definitions"
      } else {
        "content"
      };
      let at_note = if self.fordefinitions && !self.at_letter {
        " w/@ other"
      } else {
        ""
      };
      Some(s!("Processing {}{} {}", kind, at_note, source))
    } else {
      None
    };
    // Perl Mouth.pm L97: ProgressSpinup($$self{note_message}) — emit
    // `(Processing definitions <source>...` when this mouth begins reading.
    // The matching ProgressSpindown (Mouth.pm L121) is in `finish()` below.
    if let Some(ref msg) = self.note_message {
      note_begin(msg);
    }
    // Perl Mouth.pm:98-100: at_letter saves/restores `@`'s catcode with a
    // LOCAL assignment (no scope argument), independently of fordefinitions.
    // It was global here (2121a02d09, "persist across scope frame pops"),
    // which erased the enclosing group's undo entry for `@`: a package that
    // `\input`s a file inside `\bgroup\catcode`\@0 … \egroup` (CoverPage.sty:
    // 60-70 reading a BibTeX-keyword file) then kept `@` as an escape after
    // the group, and every later `\CP@…`/`\define@key` split (coverpage
    // SimpleSample, ~20 errors; RUST-ONLY, Perl 0).
    // Guard: `perfect_kernel_batch56::at_letter_mouth_keeps_group_catcode_undo`.
    if self.at_letter {
      self.saved_at_cc = lookup_catcode('@');
      assign_catcode('@', Catcode::LETTER, None);
    }
    // Perl: fordefinitions saves/restores INCLUDE_COMMENTS
    if self.fordefinitions {
      self.saved_include_comments = match lookup_value("INCLUDE_COMMENTS") {
        Some(Stored::Bool(x)) => Some(x),
        _ => None,
      };
      assign_value("INCLUDE_COMMENTS", false, Some(Scope::Local));
    }
  }
  /// Stop reading from this mouth: clear buffers and close file handle.
  /// Called by flush_mouth (\endinput) to prevent further reading.
  /// Does NOT restore catcodes — that's done by finish().
  pub fn stop_reading(&mut self) {
    self.buffer = VecDeque::new();
    self.raw_buffer = VecDeque::new();
    self.chars = VecDeque::new();
    self.lineno = 0;
    self.colno = 0;
    self.nchars = 0;
    self.reader.take(); // close file handle
  }

  /// Fully finish this mouth: stop reading AND restore catcodes/state.
  /// Called by close_mouth when the mouth is popped from the stack.
  pub fn finish(&mut self) {
    self.stop_reading();
    // Perl Mouth.pm:117: at_letter restores `@`'s catcode locally (see
    // `initialize`).
    if self.at_letter {
      let cc = self.saved_at_cc.take().unwrap_or(Catcode::OTHER);
      assign_catcode('@', cc, None);
    }
    // Perl: fordefinitions restores INCLUDE_COMMENTS
    if let Some(sic) = self.saved_include_comments.take() {
      assign_value("INCLUDE_COMMENTS", sic, Some(Scope::Local))
    }
    if self.notes
      && let Some(ref msg) = self.note_message
    {
      note_end(msg);
    }
  }
  // Auxiliaries

  /// This is (hopefully) a platform independent way of splitting a string
  /// into "lines" ending with CRLF, CR or LF (DOS, Mac or Unix).
  /// Note that TeX considers newlines to be \r, ie CR, ie ^^M
  fn split_lines(lines: &str) -> VecDeque<String> {
    let mut lines: VecDeque<String> = LINEBREAK_REGEX.split(lines).map(str::to_owned).collect();
    if let Some(last_line) = lines.back()
      && last_line.is_empty()
    {
      lines.pop_back();
    }
    lines
  }

  /// Split raw bytes into lines without decoding, splitting on \r\n, \r, or \n.
  fn split_raw_lines(bytes: &[u8]) -> VecDeque<Vec<u8>> {
    let mut lines = VecDeque::new();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
      if bytes[i] == b'\r' {
        lines.push_back(bytes[start..i].to_vec());
        if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
          i += 1; // skip \n after \r
        }
        start = i + 1;
      } else if bytes[i] == b'\n' {
        lines.push_back(bytes[start..i].to_vec());
        start = i + 1;
      }
      i += 1;
    }
    // Add remaining bytes (last line without trailing newline)
    if start < bytes.len() {
      lines.push_back(bytes[start..].to_vec());
    }
    lines
  }

  /// Decode a raw byte line using the current encoding setting.
  /// Matches Perl's per-line decode behavior.
  fn decode_bytes(raw_line: &[u8], location: String) -> String {
    if let Some(ref encoding_sym) = get_input_encoding() {
      // Probe the encoding without allocating — this fires per input
      // line, so even a small heap alloc per call adds up on large
      // documents. Only resolve the symbol to an owned String when we
      // actually need it for the misdefined-encoding Info! message.
      let is_latin1 = crate::common::arena::with(*encoding_sym, |s| {
        s.eq_ignore_ascii_case("iso-8859-1")
          || s.eq_ignore_ascii_case("latin1")
          || s.eq_ignore_ascii_case("latin-1")
      });
      let file_str = if is_latin1 {
        raw_line.iter().map(|&b| b as char).collect::<String>()
      } else {
        // Fast path for valid UTF-8 (overwhelming majority of TeX
        // source under `inputenc[utf8]`). `str::from_utf8` validates
        // the whole slice in a tight SIMD loop and returns a
        // borrow on success — no `Utf8Chunks::next` iteration
        // looking for invalid bytes. Fall back to `from_utf8_lossy`
        // only when validation fails.
        match str::from_utf8(raw_line) {
          Ok(s) => s.to_string(),
          Err(_) => String::from_utf8_lossy(raw_line).into_owned(),
        }
      };
      // Replace the U+FFFD inserted by lossy decode with space. For
      // valid-UTF-8 inputs (no FFFD), skip the replace+log scan
      // entirely. The original logic compared `replaced.len()` to
      // `file_str.len()` to detect FFFD presence indirectly; the
      // explicit `contains` is cheaper and lets us avoid the
      // unconditional `replace` walk on every input line.
      let has_fffd = file_str.contains('\u{FFFD}');
      if has_fffd {
        let encoding_name = crate::common::arena::to_string(*encoding_sym);
        Info!(
          "misdefined",
          &encoding_name,
          s!("input isn't valid under encoding {}", &encoding_name),
          "",
          "",
          location
        );
        file_str.replace('\u{FFFD}', " ")
      } else {
        file_str
      }
    } else {
      // No encoding set — interpret as UTF-8, falling back to a Latin-1
      // passthrough for non-UTF-8 bytes. This happens after inputenc
      // disables PERL_INPUT_ENCODING and the remaining file lines contain
      // high bytes.
      decode_input_bytes(raw_line)
    }
  }

  /// Original LaTeXML:
  /// This is (hopefully) a correct way to split a line into "chars",
  /// or what is probably more desired is "Grapheme clusters" (even "extended")
  /// These are unicode characters that include any following combining chars, accents & such.
  /// I am thinking that when we deal with unicode this may be the most correct way?
  /// If it's not the way XeTeX does it, perhaps, it must be that ALL combining chars
  /// have to be converted to the proper accent control sequences!
  fn get_next_line(&mut self) -> Option<String> {
    if self.buffer.is_empty() && !self.raw_buffer.is_empty() {
      // Decode the next raw byte line lazily using the current encoding.
      // This matches Perl's approach: each line is decoded with the encoding
      // that is active at the time the line is read, allowing inputenc to
      // change encoding mid-file.
      if let Some(raw_line) = self.raw_buffer.pop_front() {
        let decoded = Mouth::decode_bytes(&raw_line, self.get_location());
        self.buffer.push_back(decoded);
      }
    }
    if self.buffer.is_empty()
      && let Some(ref mut reader) = self.reader
    {
      // file mouth case — read all bytes, split into raw lines, decode lazily
      let mut file_bytes = Vec::new();
      let _num_bytes = match reader.read_to_end(&mut file_bytes) {
        Ok(count) => count,
        Err(e) => {
          let message = s!("BufReader::read_to_end returned an error: {:?}", e);
          Warn!("mouth", "io", message, "", "", self.get_location());
          0
        },
      };
      // remove the now exhausted reader
      self.reader.take();
      // Split raw bytes into lines without decoding (preserving raw bytes).
      // Each line is decoded lazily via decode_bytes() using the CURRENT encoding.
      self.raw_buffer = Mouth::split_raw_lines(&file_bytes);
      // Decode the first line now
      if let Some(raw_line) = self.raw_buffer.pop_front() {
        let decoded = Mouth::decode_bytes(&raw_line, self.get_location());
        self.buffer.push_back(decoded);
      }
    }
    self.buffer.pop_front()
  }

  /// Get the next character & it's catcode from the current line of input, even ignored chars,
  /// handling TeX's "^^" encoding.
  /// Note that this is the only place where catcode lookup is done (well almost),
  /// and that it is somewhat `inlined'.
  fn get_next_char(&mut self) -> Option<(char, Catcode)> {
    if self.colno >= self.nchars {
      return None;
    };
    let ch_opt = self.chars.get(self.colno);
    self.colno += 1;
    if let Some(ch) = ch_opt {
      let mut ch = *ch;
      let mut cc = self.catcode_of(ch);
      // Possible convert ^^x
      // Perl: (cc == CC_SUPER) && (colno + 1 < nchars) && (ch == chars[colno])
      if cc == Catcode::SUPER
        && self.colno + 1 < self.nchars
        && Some(&ch) == self.chars.get(self.colno)
      {
        // XeTeX/LuaTeX extended caret notation, longest-match-first:
        // ^^^^^^hhhhhh (6 hex) and ^^^^hhhh (4 hex) produce one Unicode
        // scalar. This engine is Unicode-native (same precedent as
        // providing \Ucharcat despite pdfTeX lacking it), and packages
        // PROBE for a Unicode engine with exactly this notation —
        // newunicodechar.sty L52-56 `\edef\next{\@gobble^^^^0021}` takes
        // its broken 8-bit branch without it (9-doc corpus cluster,
        // witnesses eigo, verifica, tikz-trackschematic, uspace).
        let is_lowerhex_c = |c: char| -> bool { matches!(c, '0'..='9' | 'a'..='f') };
        for (extra_carets, ndigits) in [(4usize, 6usize), (2, 4)] {
          // self.colno-1 is the first ^; require extra ^s then ndigits hex.
          let carets_ok =
            (0..extra_carets).all(|k| self.chars.get(self.colno + 1 + k) == Some(&ch));
          let dig_start = self.colno + 1 + extra_carets;
          if carets_ok
            && dig_start + ndigits <= self.nchars
            && (0..ndigits).all(|k| {
              self
                .chars
                .get(dig_start + k)
                .is_some_and(|c| is_lowerhex_c(*c))
            })
          {
            let hex: String = (0..ndigits)
              .filter_map(|k| self.chars.get(dig_start + k))
              .collect();
            if let Some(newch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
              let removed = 2 + extra_carets + ndigits;
              self.splice(self.colno - 1..self.colno - 1 + removed, &[newch]);
              self.nchars -= removed - 1;
              let cc2 = self.catcode_of(newch);
              return Some((newch, cc2));
            }
          }
        }
        let c1_opt = self.chars.get(self.colno + 1);
        let c2_opt = self.chars.get(self.colno + 2);
        let mut two_hex = false;
        // ^^ followed by TWO LOWERCASE Hex digits???
        if let Some(c1) = c1_opt
          && let Some(c2) = c2_opt
        {
          // Perf: avoid per-char String alloc + regex match by using
          // direct ASCII class check. LOWERHEX_REGEX = ^[0-9a-f]$, i.e.
          // lowercase hex digits only.
          let is_lowerhex = |c: char| -> bool { matches!(c, '0'..='9' | 'a'..='f') };
          if (self.colno + 2 < self.nchars) && is_lowerhex(*c1) && is_lowerhex(*c2) {
            // TODO: Maybe Result type warranted here?
            let hex = u8::from_str_radix(&s!("{}{}", c1, c2), 16).unwrap();
            ch = hex as char;
            self.splice(self.colno - 1..self.colno + 3, &[ch]);
            self.nchars -= 3;
            two_hex = true;
          }
        }
        if !two_hex {
          // OR ^^ followed by a SINGLE Control char type code???
          let c = self.chars[self.colno + 1];
          let cn = c as i16;

          ch = (cn + if cn >= 64 { -64 } else { 64 }) as u8 as char;
          self.splice(self.colno - 1..self.colno + 2, &[ch]);
          self.nchars -= 2;
        }
        cc = self.catcode_of(ch);
      }
      Some((ch, cc))
    } else {
      None
    }
  }

  /// The catcode this mouth reads `ch` with: the State's, except that a
  /// [`Self::with_bib_data_literals`] mouth downgrades the four BibTeX-data
  /// characters to OTHER when — and only when — they still carry their TeX
  /// meaning.
  fn catcode_of(&self, ch: char) -> Catcode {
    let cc = lookup_catcode(ch).unwrap_or(Catcode::OTHER);
    if !self.bib_data_literals {
      return cc;
    }
    match (cc, ch) {
      (Catcode::COMMENT, '%') | (Catcode::ALIGN, '&') | (Catcode::PARAM, '#') => Catcode::OTHER,
      _ => cc,
    }
  }

  /// Checks if there is more input to process.
  ///
  /// Note: we need mutability, as we may refill the internal BufReader
  /// when performing the check.
  pub fn has_more_input(&mut self) -> bool {
    if !self.is_eol() || !self.buffer.is_empty() || !self.raw_buffer.is_empty() {
      return true;
    }
    // Peek the underlying reader if present. A fill_buf I/O error is treated
    // as end-of-input (return false) rather than panicking — the caller will
    // naturally stop requesting tokens and the Mouth will be closed out.
    match self.reader.as_mut() {
      Some(r) => r.fill_buf().map(|buf| !buf.is_empty()).unwrap_or(false),
      None => false,
    }
  }

  /// Read the next token, or undef if exhausted.
  /// Note that this also returns COMMENT tokens containing source comments,
  /// and also locator comments (file, line# info).
  /// LaTeXML::Core::Gullet intercepts them and passes them on at appropriate times.
  pub fn read_token(&mut self) -> Option<Token> {
    loop {
      // Iterate till we find a token, or run out. (use return)
      // ===== Get next line, if we need to.
      if self.colno >= self.nchars {
        self.lineno += 1;
        self.colno = 0;
        let line_opt = self.get_next_line();
        // For \read, we have to return something for EOL, and handle implicit final newline
        let read_mode = lookup_int("PRESERVE_NEWLINES") > 1;
        let eolch = match lookup_definition(&T_CS!("\\endlinechar")).unwrap() {
          Some(defn) => {
            if defn.is_register() {
              if let Some(eol) = defn.value_of(Vec::new()) {
                let eol = eol.value_of() as i16;
                if eol > 0 && eol <= 255 {
                  let mch = (eol as u8) as char;
                  Some(mch)
                } else {
                  None
                }
              } else {
                None
              }
            } else {
              None
            }
          },
          _ => Some('\r'),
        };
        if line_opt.is_none() {
          // Exhausted the input.
          let eolcc = if let Some(ch) = eolch {
            lookup_catcode(ch).unwrap_or(Catcode::OTHER)
          } else {
            Catcode::OTHER
          };
          let eoftoken = if let Some(eolch_content) = eolch {
            if read_mode && !self.at_eof && !self.source.is_empty() {
              // The synthetic final line is empty + `\endlinechar`, read in
              // state N like any other line (tex.web §345-349): an EOL char
              // is `\par`, a SPACE or IGNORE char is dropped (§349 "goto
              // switch" — it can never become a token), anything else is a
              // char token. Perl Mouth.pm:303-307 emits the IGNORE char and
              // it reaches the Stomach as `misdefined` (KNOWN_PERL_ERRORS
              // #125); witness liftarm (pgfmanual `codeexample` sets
              // `\catcode`\^^M=9` around `\scantokens`, and animate.sty's
              // `\@anim@buildtmln` `\read`s the timeline to EOF inside it —
              // 501 errors capped).
              match eolcc {
                Catcode::EOL => Some(T_CS!("\\par")),
                Catcode::SPACE | Catcode::IGNORE => None,
                _ => Some(CharToken!(eolch_content, eolcc)),
              }
            } else {
              None
            }
          } else {
            None
          };
          self.at_eof = true;
          self.chars = VecDeque::new();
          self.nchars = 0;
          return eoftoken;
        }
        // Remove trailing spaces from external sources
        let mut line = line_opt.unwrap();
        if !self.source.is_empty() && line.ends_with(' ') {
          line = TRAILING_SPACE_CHARS.replace(&line, "").to_string();
        }
        // Then append the appropriate \endlinechar, or "\r";
        if let Some(ch) = eolch {
          line.push(ch);
        }

        self.chars = line.chars().collect::<VecDeque<char>>();
        self.nchars = self.chars.len();
        // In state N, skip leading spaces & ignored, possibly decoding (trailing space removed
        // above)
        while let Some((_ch, cc)) = self.get_next_char() {
          match cc {
            Catcode::SPACE | Catcode::IGNORE => {},
            Catcode::EOL => {
              // Eolch already? empty line!
              self.colno = self.nchars; // ignore rest of line.
              return Some(T_CS!("\\par"));
            },
            _ => break,
          }
        }
        if self.nchars == 0 || self.colno > self.nchars {
          // Past end of line?
          // If upcoming line is empty, and there is no recognizable EOL, fake one
          if read_mode && eolch != Some('\r') {
            return Some(T_MARKER!("EOL"));
          }
        } else {
          // Back up over peeked char
          self.colno -= 1;
        }
        // Sneak a comment out, every so often.
        if self.lineno.is_multiple_of(READLINE_PROGRESS_QUANTUM) && lookup_bool("INCLUDE_COMMENTS")
        {
          // Perl T_COMMENT prepends '%' (Token.pm L81)
          return Some(T_COMMENT!(s!(
            "%**** {} Line {} ****",
            &self.shortsource,
            &self.lineno.to_string()
          )));
        }
      }
      // In state::S, skip spaces
      if self.skipping_spaces {
        let mut cc = None;
        // This is very awkward as a loop,
        //  but I had to port the Perl logic without going crazy...
        // tokenizer/verb.tex depends on it.
        while let Some((_, ncc)) = self.get_next_char() {
          cc = Some(ncc);
          if ncc != Catcode::SPACE {
            break;
          }
        }
        if self.colno <= self.nchars && cc.is_some() && cc != Some(Catcode::SPACE) {
          self.colno -= 1;
        }
        if cc == Some(Catcode::EOL) {
          // If we've got an EOL
          self.get_next_char();
          if self.colno < self.nchars {
            self.colno -= 1;
          }
        }
        self.skipping_spaces = false;
      }
      // ==== Extract next token from line.
      // §1 (docs/performance/SOURCE_PROVENANCE.md): record the token's source start now —
      // after all inter-token skips (line fetch, leading / skipping spaces) and
      // before `get_next_char` advances past its first char. `colno` is 0-indexed
      // here; the +1 to 1-indexed columns happens in `get_locator`.
      self.last_token_start = (self.lineno, self.colno);
      if let Some((ch, cc)) = self.get_next_char() {
        #[cfg(not(feature = "token-locators"))]
        if let Some(token) = Mouth::dispatch_char(self, ch, cc) {
          return Some(token);
        } // Else, repeat till we get something or run out.
        // token-locators: stamp the token with an origin handle into the side
        // arena, using `last_token_start` (the token's first char — captured
        // above, before `dispatch_char` reads the rest, e.g. a CS name). This is
        // what survives expansion to digestion (Experiments 1–3 showed the mouth
        // position at digest time cannot recover it). See SOURCE_PROVENANCE §3.1.1.
        #[cfg(feature = "token-locators")]
        if let Some(mut token) = Mouth::dispatch_char(self, ch, cc) {
          let (line, col0) = self.last_token_start;
          token.loc =
            crate::token::push_token_origin(self.source_sym, line as u32, (col0 + 1) as u32);
          return Some(token);
        } // Else, repeat till we get something or run out.
      }
    }
  }

  //**********************************************************************
  /// Read all tokens until a token equal to $until (if given), or until exhausted.
  /// Returns an empty Tokens list, if there is no input
  pub fn read_tokens(&mut self) -> Tokens {
    // Pre-size to skip the early doubling reallocations of the per-token push
    // loop below (a `grow_one` site in the allocation profile); 16 covers a
    // typical line/group in one allocation.
    let mut tokens = Vec::with_capacity(16);
    while let Some(token) = self.read_token() {
      tokens.push(token);
    }
    while let Some(Token { code: Catcode::SPACE, .. }) = tokens.last() {
      // Remove trailing space
      tokens.pop();
    }
    Tokens::new(tokens)
  }

  //**********************************************************************
  // Read a raw lines; there are so many variants of how it should end,
  // that the Mouth API is left as simple as possible.
  // Alas: $noread true means NOT to read a new line, but only return
  // the remainder of the current line, if any. This is useful when combining
  // with previously peeked tokens from the Gullet.
  /// The WHOLE current line as raw text, from column 0 — including the part
  /// already tokenized — and consume the rest of it. `None` when no line is
  /// loaded. Used by the listings raw-line reader when an optional-argument
  /// probe has already tokenized the body's first token (OXIDIZED_DESIGN #162).
  pub fn read_raw_line_from_start(&mut self) -> Option<String> {
    if self.nchars == 0 && self.colno == 0 {
      return None;
    }
    let mut line: String = self.chars.iter().collect();
    if line.ends_with('\r') {
      line.pop();
    }
    self.colno = self.nchars;
    Some(line)
  }

  pub fn read_raw_line(&mut self, noread: bool) -> Option<String> {
    let mut line = String::new();
    if self.colno < self.nchars {
      line = self.chars.iter().skip(self.colno).collect();
      // Strip the final carriage return, if it has been added back (Perl: s/\r$//s)
      if line.ends_with('\r') {
        line.pop();
      }
      self.colno = self.nchars;
    } else if !noread {
      match self.get_next_line() {
        None => {
          // We've exhausted this mouth
          self.at_eof = true;
          self.chars = VecDeque::new();
          self.nchars = 0;
          self.colno = 0;
          return None;
        },
        Some(next_line) => {
          // Strip trailing spaces (Perl: s/ *$//s)
          line = next_line.trim_end_matches(' ').to_string();
          self.lineno += 1;
          self.chars = line.chars().collect();
          self.nchars = self.chars.len();
          self.colno = self.nchars;
        },
      }
    }
    Some(line)
  }

  fn dispatch_char(&mut self, ch: char, cc: Catcode) -> Option<Token> {
    // Possibly want to think about caching (common) letters, etc to keep from
    // creating tokens like crazy... or making them more compact... or ???
    use crate::token::Catcode::*;
    match cc {
      ESCAPE => self.handle_escape(), // T_ESCAPE
      BEGIN => {
        if ch == '{' {
          Some(T_BEGIN!())
        } else {
          Some(CharToken!(ch, BEGIN))
        }
      },
      END => {
        if ch == '}' {
          Some(T_END!())
        } else {
          Some(CharToken!(ch, END))
        }
      },
      MATH => {
        if ch == '$' {
          Some(T_MATH!())
        } else {
          Some(CharToken!(ch, MATH))
        }
      },
      ALIGN => {
        if ch == '&' {
          Some(T_ALIGN!())
        } else {
          Some(CharToken!(ch, ALIGN))
        }
      },
      EOL => Some(self.handle_end_of_line()),
      PARAM => {
        if ch == '#' {
          Some(T_PARAM!())
        } else {
          Some(CharToken!(ch, PARAM))
        }
      }, // T_PARAM
      SUPER => {
        if ch == '^' {
          Some(T_SUPER!())
        } else {
          Some(CharToken!(ch, SUPER))
        }
      }, // T_SUPER
      SUB => {
        if ch == '_' {
          Some(T_SUB!())
        } else {
          Some(CharToken!(ch, SUB))
        }
      }, // T_SUB
      SPACE => self.handle_space(),
      LETTER => Some(CharToken!(ch, LETTER)),
      OTHER => Some(CharToken!(ch, OTHER)),
      ACTIVE => Some(T_ACTIVE!(ch)),
      COMMENT => self.handle_comment(),
      INVALID => Some(CharToken!(ch, OTHER)), // T_INVALID (we could get unicode!)
      _ => None,                              // IGNORE, others
    }
  }

  fn handle_end_of_line(&mut self) -> Token {
    self.colno = self.nchars; // Ignore any remaining characters after EOL
    if lookup_int("PRESERVE_NEWLINES") != 0 {
      Token!("\n", Catcode::SPACE)
    } else {
      T_SPACE!()
    }
  }

  fn handle_space(&mut self) -> Option<Token> {
    // Skip any following spaces!
    while let Some((_ch, cc)) = self.get_next_char() {
      if (cc != Catcode::SPACE) && (cc != Catcode::EOL) {
        // backup at nonspace/eol
        if self.colno <= self.nchars {
          self.colno -= 1;
        }
        break;
      }
    }
    Some(T_SPACE!())
  }

  fn handle_comment(&mut self) -> Option<Token> {
    let n = self.colno;
    self.colno = self.nchars;
    let mut comment = String::new();
    for c in self.chars.iter().skip(n).take(self.nchars - n) {
      comment.push(*c);
    }
    let trimmed_comment = comment.trim();
    if !trimmed_comment.is_empty() && lookup_bool("INCLUDE_COMMENTS") {
      // Perl T_COMMENT prepends '%' to the comment text (Token.pm L81)
      Some(T_COMMENT!(s!("%{}", trimmed_comment)))
    } else if lookup_int("PRESERVE_NEWLINES") > 1 {
      Some(T_MARKER!("EOL")) // Required EOL during \read
    } else {
      None
    }
  }

  //**********************************************************************
  // See The TeXBook, Chapter 8, The Characters You Type, pp.46--47.
  //**********************************************************************

  /// Read control sequence
  fn handle_escape(&mut self) -> Option<Token> {
    // NOTE: We're using control sequences WITH the \ prepended!!!
    if let Some((ch, mut cc)) = self.get_next_char() {
      // Knuth, p.46 says that Newlines are converted to spaces,
      // Bit I believe that he does NOT mean within control sequences
      let mut cs = s!("\\{}", ch);
      if cc == Catcode::LETTER {
        // For letter, read more letters for csname.
        while let Some((nch, ncc)) = self.get_next_char() {
          cc = ncc;
          if ncc == Catcode::LETTER {
            cs.push(nch);
          } else {
            break;
          }
        }
        // We WILL skip spaces, but not till next token is read (in case catcode changes!!!!)
        self.skipping_spaces = true;
        if cc != Catcode::LETTER {
          self.colno -= 1;
        }
      }
      Some(T_CS!(cs))
    } else {
      None
    }
  }

  /// TODO: Can we use/build a generic that does this reliably for VecDeque
  fn splice<R>(&mut self, range: R, with: &[char])
  where R: RangeBounds<usize> {
    let mut v: Vec<char> = self.chars.drain(..).collect();
    v.splice(range, with.iter().cloned());
    self.chars = v.into_iter().collect();
  }

  /// Checks if Mouth read is at the end of a line.
  ///
  /// Careful:
  /// used BOTH for flushing input for `\endinput`
  /// and for detecting line end for `\read`
  /// tex.web §485-486: a `\read` consumes the WHOLE physical line
  /// (`input_ln`); the catcode regime only decides which tokens the line
  /// yields. Called by `\read` after its token loop so a residual
  /// `\endlinechar` character left behind under one regime (an IGNORE-catcode
  /// space under `\ExplSyntaxOn`) is never re-read as a spurious empty line by
  /// a later `\read` under another (l3prefixes: header `\ior_get` in the
  /// preamble, `\ior_map_inline` in the body → an empty first row →
  /// `Until:,` runaway).
  pub fn finish_physical_line(&mut self) { self.colno = self.nchars; }

  pub fn is_eol(&mut self) -> bool {
    let savecolno = self.colno;
    // We have to peek past any ignored tokens & also spaces, if skipping
    let mut cc = None;
    while let Some((_, ncc)) = self.get_next_char() {
      if ncc != Catcode::IGNORE && (!self.skipping_spaces || ncc != Catcode::SPACE) {
        cc = Some(ncc);
        break;
      }
    }
    if self.colno <= self.nchars && cc.is_some() {
      // Back-up if too far.
      self.colno -= 1;
    }
    // If skipping spaces (really, reading for input (\endinput) ?), jump to end of EOL or comments
    if self.skipping_spaces && (cc == Some(Catcode::EOL) || cc == Some(Catcode::COMMENT)) {
      // If we've got an EOL | COMMENT
      self.colno = self.nchars
    }
    let eol = self.colno >= self.nchars;
    self.colno = savecolno;
    eol
  }

  pub fn at_eof(&self) -> bool { self.at_eof }

  /// §1 accurate-start locator (docs/performance/SOURCE_PROVENANCE.md): `from` = the captured
  /// start of the most recently *begun* token (`last_token_start`), `to` = the
  /// mouth's current position. Unlike `get_locator`, whose `from` is the
  /// eating-disorder heuristic (line start vs current col), this `from` is exact
  /// for the token currently being processed — the basis for accurate
  /// construct-start ranges under `--source-map`. `lineno` is already 1-indexed
  /// (it counts from 1 after the first line fetch); `colno` is 0-indexed, +1 to
  /// 1-indexed columns, matching `get_locator`.
  pub fn get_locator_from_start(&self) -> Locator {
    let (from_line, from_col0) = self.last_token_start;
    Locator::from_sym(
      self.source_sym,
      from_line as u32,
      (from_col0 + 1) as u32,
      self.lineno as u32,
      (self.colno + 1) as u32,
    )
  }

  pub fn get_location(&self) -> String {
    let loc = self.get_locator().unwrap_or_default();
    s!("at {}", loc)
  }
}

/// Tokenize a string under the **standard** catcode table — Perl
/// `Package.pm:Tokenize` L1019-1023.
///
/// "Standard" is the document-level table: `@` is an ordinary letter-less
/// character, so this is how user-facing text should be read. The current
/// state's catcodes are deliberately NOT consulted; the table is swapped in for
/// the duration and restored afterwards, exactly as Perl's `local $STATE =
/// $STD_CATTABLE` does, so a document that has been playing with catcodes
/// cannot change what a binding's own string means.
///
/// See [`tokenize_internal`] for the `.sty`-style table that treats `@` as a
/// letter.
///
/// The argument is an `impl Into<`[`TeXString`]`>`, not a `&str`: a string
/// literal converts implicitly, but a `String` — the shape a control-word-welding
/// `Tokens::to_string()` arrives in — must declare itself via
/// [`Tokens::untex_string`] or [`TeXString::assembled`]. See the [`TeXString`]
/// docs for why.
pub fn tokenize(text: impl Into<TeXString>) -> Tokens {
  let text = text.into();
  // special case! empty input is empty Tokens
  if text.is_empty() {
    return NO_TOKENS;
  }
  use_std_state();
  let result = Mouth::new(text.as_str(), None).unwrap().read_tokens();
  use_main_state();
  result
}
/// Tokenize a string under the standard catcode table, reading `% & #` as
/// ordinary characters rather than comment, alignment tab and parameter.
///
/// For text that came out of the BibTeX lexer, which has none of those
/// constructs, so all three are data — treatment 1 of `OXIDIZED_DESIGN #74`,
/// see [`Mouth::with_bib_data_literals`] (including why `_` is not in the set).
/// Plain [`tokenize`] would let a `%` comment out the rest of the string, which
/// for a `.bib` field means losing its closing brace and leaving whatever it
/// opened unclosed, and would make the `&` in "Taylor & Francis" a stray
/// alignment tab.
///
/// This exists because the handlers that re-read a raw field — `\bib@@title`
/// recasing, name splitting, date/pages assembly — build their tokens from the
/// stored string and never pass through the per-entry mouth.
///
/// Takes an `impl Into<`[`TeXString`]`>` for the same reason as [`tokenize`] —
/// and it is the sink that most needs it: the bibliography is where the
/// control-word weld has surfaced three times (PR #399, PR #400, issue 410).
pub fn tokenize_bib_literal(text: impl Into<TeXString>) -> Tokens {
  let text = text.into();
  // special case! empty input is empty Tokens
  if text.is_empty() {
    return NO_TOKENS;
  }
  use_std_state();
  let result = Mouth::new(text.as_str(), None)
    .unwrap()
    .with_bib_data_literals()
    .read_tokens();
  use_main_state();
  result
}

/// Tokenize a string under the **style-file** catcode table — Perl
/// `Package.pm:TokenizeInternal` L1026-1030.
///
/// Same swap-and-restore discipline as [`tokenize`], but with `@` a letter, so
/// internal control sequences (`\@ifnextchar`, `\@currentlabel`, …) tokenize as
/// single names. This is the right choice for a macro body a binding writes
/// itself, and the wrong one for text that came from the document.
///
/// Takes an `impl Into<`[`TeXString`]`>` for the same reason as [`tokenize`].
pub fn tokenize_internal(text: impl Into<TeXString>) -> Tokens {
  let text = text.into();
  // special case! empty input is empty Tokens
  if text.is_empty() {
    return NO_TOKENS;
  }
  use_sty_state();
  let result = Mouth::new(text.as_str(), None).unwrap().read_tokens();
  use_main_state();
  result
}

#[cfg(test)]
mod newline_tests {
  use super::*;

  /// CRLF-input regression guard (WINDOWS_COMPATIBILITY_PLAN risk #5): a
  /// document saved with Windows line endings must tokenize identically to
  /// its LF twin — split_raw_lines implements TeX's universal end-of-line
  /// (\r\n, \r, and \n all terminate a line, and the terminator itself
  /// never reaches the catcode machinery).
  #[test]
  fn split_raw_lines_universal_newlines() {
    let lf = Mouth::split_raw_lines(b"a\nb\nc");
    let crlf = Mouth::split_raw_lines(b"a\r\nb\r\nc");
    let cr = Mouth::split_raw_lines(b"a\rb\rc");
    assert_eq!(lf, crlf, "CRLF must split identically to LF");
    assert_eq!(lf, cr, "bare CR must split identically to LF");
    assert_eq!(lf.len(), 3);
    assert_eq!(lf[0], b"a");
    // A lone \r inside the terminator pair is consumed, not leaked into
    // the following line.
    assert!(
      crlf.iter().all(|line| !line.contains(&b'\r')),
      "no line may retain a raw CR byte"
    );
  }
}

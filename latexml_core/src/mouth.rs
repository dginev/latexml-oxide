use std::collections::VecDeque;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::str;

use core::ops::RangeBounds;
// TODO:
// use encoding::all::ISO_8859_1;
// use encoding::{EncoderTrap, Encoding};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::common::error::*;
use crate::common::locator::Locator;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::state;
use crate::state::*;
use crate::token::*;
use crate::tokens::{NO_TOKENS, Tokens};
use crate::util::pathname;

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
  foodtype:               FoodType,
  saved_at_cc:            Option<Catcode>,
  saved_include_comments: Option<bool>,
  note_message:           Option<String>,
  source:                 String,
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
      chars:                  VecDeque::new(),
      nchars:                 0,
      source:                 String::from("Anonymous String"),
      shortsource:            s!("String"),
      // handle : None,
      foodtype:               FoodType::File,
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
  fn get_locator(&self) -> Locator {
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
    Locator::new(
      &self.source,
      from_line as u32,
      (from_column + 1) as u32,
      to_line as u32,
      (to_column + 1) as u32,
    )
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
      Mouth::new(source, Some(options))
    }
  }

  pub fn new(text: &str, options: Option<MouthOptions>) -> Result<Self> {
    let mut mouth = match options {
      None => Mouth {
        foodtype: FoodType::Literal,
        ..Mouth::default()
      },
      Some(opts) => {
        let shortsource = opts.shortsource.unwrap_or_else(|| s!("String"));
        Mouth {
          foodtype: opts.foodtype.unwrap_or(FoodType::Literal),
          fordefinitions: opts.fordefinitions,
          at_letter: opts.at_letter,
          notes: opts.notes,
          source: opts.source.unwrap_or_default(),
          shortsource,
          ..Mouth::default()
        }
      },
    };
    mouth.open(text)?;
    Ok(mouth)
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
          // Check for binary file (non-empty and appears binary)
          // Perl's -B heuristic: check first block for high proportion of non-text bytes
          if meta.len() > 0 {
            if let Ok(mut f) = File::open(pathname) {
              let mut buf = [0u8; 512];
              if let Ok(n) = f.read(&mut buf) {
                if n > 0 {
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
    log::warn!("HTTP input not supported: {url}");
  }
  fn open_https(&mut self, url: &str) {
    log::warn!("HTTPS input not supported: {url}");
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
    // `Processing definitions <source>` when this mouth begins reading.
    // For now this is a simple Note-style line (no spinner / no timing);
    // the matching ProgressSpindown (Mouth.pm L121) becomes a no-op,
    // since `note_progress` doesn't pair.
    if let Some(ref msg) = self.note_message {
      note_progress(msg);
    }
    // Perl: at_letter saves/restores @ catcode independently of fordefinitions.
    // Use Scope::Global to ensure it persists across scope frame pops during file loading.
    if self.at_letter {
      self.saved_at_cc = lookup_catcode('@');
      assign_catcode('@', Catcode::LETTER, Some(Scope::Global));
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
    // Perl: at_letter restores @ catcode (independent of fordefinitions).
    // Use Scope::Global to ensure it takes effect regardless of scope frame state.
    if self.at_letter {
      let cc = self.saved_at_cc.take().unwrap_or(Catcode::OTHER);
      assign_catcode('@', cc, Some(Scope::Global));
    }
    // Perl: fordefinitions restores INCLUDE_COMMENTS
    if let Some(sic) = self.saved_include_comments.take() {
      assign_value("INCLUDE_COMMENTS", sic, Some(Scope::Local))
    }
    // Perl Mouth.pm L121 ProgressSpindown is a no-op for us — `initialize`
    // emits a single Note-style line via `note_progress`, no matching
    // close-tag is needed (no spinner/timer to terminate).
  }
  // Auxiliaries

  /// This is (hopefully) a platform independent way of splitting a string
  /// into "lines" ending with CRLF, CR or LF (DOS, Mac or Unix).
  /// Note that TeX considers newlines to be \r, ie CR, ie ^^M
  fn split_lines(lines: &str) -> VecDeque<String> {
    let mut lines: VecDeque<String> = LINEBREAK_REGEX.split(lines).map(str::to_owned).collect();
    if let Some(last_line) = lines.back() {
      if last_line.is_empty() {
        lines.pop_back();
      }
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
        // Fallback: try UTF-8 with lossy conversion
        String::from_utf8_lossy(raw_line).into_owned()
      };
      let replaced = file_str.replace('\u{FFFD}', " ");
      if replaced.len() != file_str.len() {
        let encoding_name = crate::common::arena::to_string(*encoding_sym);
        Info!(
          "misdefined",
          &encoding_name,
          s!("input isn't valid under encoding {}", &encoding_name),
          "",
          "",
          location
        );
      }
      replaced
    } else {
      // No encoding set — interpret as UTF-8, with fallback.
      // For non-UTF-8 bytes, we keep them as raw Latin-1 chars.
      // This matches Perl which passes raw bytes through when
      // PERL_INPUT_ENCODING is undef (disabled by inputenc).
      match str::from_utf8(raw_line) {
        Ok(s) => s.to_string(),
        Err(_) => {
          // When no encoding is set but bytes aren't valid UTF-8,
          // treat as raw bytes (Latin-1 passthrough).
          // This happens after inputenc disables PERL_INPUT_ENCODING
          // and the remaining file lines contain high bytes.
          raw_line.iter().map(|&b| b as char).collect::<String>()
        },
      }
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
    if self.buffer.is_empty() {
      if let Some(ref mut reader) = self.reader {
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
      let mut cc = lookup_catcode(ch).unwrap_or(Catcode::OTHER);
      // Possible convert ^^x
      // Perl: (cc == CC_SUPER) && (colno + 1 < nchars) && (ch == chars[colno])
      if cc == Catcode::SUPER
        && self.colno + 1 < self.nchars
        && Some(&ch) == self.chars.get(self.colno)
      {
        let c1_opt = self.chars.get(self.colno + 1);
        let c2_opt = self.chars.get(self.colno + 2);
        let mut two_hex = false;
        // ^^ followed by TWO LOWERCASE Hex digits???
        if let Some(c1) = c1_opt {
          if let Some(c2) = c2_opt {
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
        }
        if !two_hex {
          // OR ^^ followed by a SINGLE Control char type code???
          let c = self.chars[self.colno + 1];
          let cn = c as i16;

          ch = (cn + if cn >= 64 { -64 } else { 64 }) as u8 as char;
          self.splice(self.colno - 1..self.colno + 2, &[ch]);
          self.nchars -= 2;
        }
        cc = lookup_catcode(ch).unwrap_or(Catcode::OTHER);
      }
      Some((ch, cc))
    } else {
      None
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
        let eolch = if let Some(defn) = lookup_definition(&T_CS!("\\endlinechar")).unwrap() {
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
        } else {
          Some('\r')
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
              if eolcc == Catcode::EOL {
                Some(T_CS!("\\par"))
              } else {
                Some(CharToken!(eolch_content, eolcc))
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
      if let Some((ch, cc)) = self.get_next_char() {
        if let Some(token) = Mouth::dispatch_char(self, ch, cc) {
          return Some(token);
        } // Else, repeat till we get something or run out.
      }
    }
  }

  //**********************************************************************
  /// Read all tokens until a token equal to $until (if given), or until exhausted.
  /// Returns an empty Tokens list, if there is no input
  pub fn read_tokens(&mut self) -> Tokens {
    let mut tokens = Vec::new();
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
      LETTER => Some(CharToken!(ch, Catcode::LETTER)),
      OTHER => Some(CharToken!(ch, Catcode::OTHER)),
      ACTIVE => Some(T_ACTIVE!(ch)),
      COMMENT => self.handle_comment(),
      INVALID => Some(CharToken!(ch, Catcode::OTHER)), // T_INVALID (we could get unicode!)
      _ => None,                                       // IGNORE, others
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

  pub fn get_location(&self) -> String {
    let loc = self.get_locator();
    s!("at {}", loc)
  }
}

pub fn tokenize(text: &str) -> Tokens {
  // special case! empty input is empty Tokens
  if text.is_empty() {
    return NO_TOKENS;
  }
  state::use_std_state();
  let result = Mouth::new(text, None).unwrap().read_tokens();
  state::use_main_state();
  result
}
pub fn tokenize_internal(text: &str) -> Tokens {
  // special case! empty input is empty Tokens
  if text.is_empty() {
    return NO_TOKENS;
  }
  state::use_sty_state();
  let result = Mouth::new(text, None).unwrap().read_tokens();
  state::use_main_state();
  result
}

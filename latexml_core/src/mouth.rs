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

#[thread_local]
static mut LASTID: usize = 0;

static LINEBREAK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s:\r\n?)|(?s:\n)").unwrap());
static LOWERHEX_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9a-f]$").unwrap());
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
  reader:                 Option<BufReader<File>>,
}

impl PartialEq for Mouth {
  fn eq(&self, other: &Mouth) -> bool { self.source == other.source }
}

impl Default for Mouth {
  fn default() -> Self {
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
      source:                 s!("Anonymous String {}", &Mouth::gid()),
      shortsource:            s!("String"),
      // handle : None,
      foodtype:               FoodType::File,
      saved_at_cc:            None,
      saved_include_comments: None,
      buffer:                 VecDeque::new(),
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
    Locator::new(
      &self.source,
      from_line as u32,
      from_column as u32,
      to_line as u32,
      to_column as u32,
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
      options.foodtype = FoodType::opt_from_str(&pathname::protocol(source));
      options.source = Some(source.to_string());
      Mouth::new(source, Some(options))
    }
  }

  pub fn new(text: &str, options: Option<MouthOptions>) -> Result<Self> {
    let mut mouth = match options {
      None => Mouth {
        foodtype: FoodType::Literal,
        ..Mouth::default()
      },
      Some(opts) => Mouth {
        foodtype: opts.foodtype.unwrap_or(FoodType::Literal),
        fordefinitions: opts.fordefinitions,
        at_letter: opts.at_letter,
        notes: opts.notes,
        source: opts.source.unwrap_or_default(),
        ..Mouth::default()
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
          Error!("I/O", "unreadable", s!("File {} is not readable. Ignoring.", pathname), "", "",
            self.get_location());
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
                  let non_text = buf[..n].iter().filter(|&&b| b == 0 || (b < 0x20 && b != b'\n' && b != b'\r' && b != b'\t' && b != 0x1b)).count();
                  if non_text * 3 > n {
                    // High ratio of non-text bytes — likely binary
                    Error!("invalid", "binary",
                      s!("Input file {} appears to be binary. Ignoring.", pathname), "", "",
                      self.get_location());
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
          Error!("I/O", "open", s!("Can't open {} for reading: {}", pathname, e), "", "",
            self.get_location());
          return Err(e.into());
        },
      };
      let reader = BufReader::new(f);
      self.reader = Some(reader);
      self.buffer = VecDeque::new();
    }
    Ok(())
  }
  fn open_literal(&mut self, content: &str) { self.buffer = Mouth::split_lines(content); }
  fn open_http(&mut self, _content: &str) {
    todo!();
  }
  fn open_https(&mut self, _content: &str) {
    todo!();
  }
  // fn open_binding(&mut self, _content: &str) {}

  fn initialize(&mut self) {
    self.note_message = if self.notes {
      let source = if !self.source.is_empty() {
        &self.source
      } else {
        "Anonymous String"
      };
      let kind = if self.fordefinitions { "definitions" } else { "content" };
      let at_note = if self.fordefinitions && !self.at_letter { " w/@ other" } else { "" };
      Some(s!("Processing {}{} {}", kind, at_note, source))
    } else {
      None
    };
    // Perl: at_letter saves/restores @ catcode independently of fordefinitions
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
  pub fn finish(&mut self) {
    self.buffer = VecDeque::new();
    self.chars = VecDeque::new();
    self.lineno = 0;
    self.colno = 0;
    self.nchars = 0;
    // Perl: at_letter restores @ catcode (independent of fordefinitions)
    if let Some(cc) = self.saved_at_cc.take() {
      assign_catcode('@', cc, None);
    }
    // Perl: fordefinitions restores INCLUDE_COMMENTS
    if let Some(sic) = self.saved_include_comments.take() {
      assign_value("INCLUDE_COMMENTS", sic, Some(Scope::Local))
    }
    if self.notes {
      if let Some(ref msg) = self.note_message {
        note_end(msg);
      }
    }
    self.reader.take(); // if we have a reader, this will force a Drop at the end of finish(), which
    // will close the file handle
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

  /// Original LaTeXML:
  /// This is (hopefully) a correct way to split a line into "chars",
  /// or what is probably more desired is "Grapheme clusters" (even "extended")
  /// These are unicode characters that include any following combining chars, accents & such.
  /// I am thinking that when we deal with unicode this may be the most correct way?
  /// If it's not the way XeTeX does it, perhaps, it must be that ALL combining chars
  /// have to be converted to the proper accent control sequences!
  fn get_next_line(&mut self) -> Option<String> {
    if self.buffer.is_empty() {
      if let Some(ref mut reader) = self.reader {
        // file mouth case
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
        // Note: the original latexml code first split the perl string into lines, and only THEN
        // decoded it however, executing a rust regex on a Vec<u8> is
        // just not going to be a sane way forward. we will first decode
        // the read-in bytes to the right String form, and THEN split lines.
        // as such, decoding is the first action taken on bytes read in from a file.
        if let Some(ref encoding_sym) = get_input_encoding() {
          // Perl: decode with Encode::FB_DEFAULT, then replace \x{FFFD} with space
          let encoding_name = crate::common::arena::to_string(*encoding_sym);
          // For now, support latin-1/iso-8859-1 (most common non-UTF-8 TeX encoding)
          let file_str = if encoding_name.eq_ignore_ascii_case("iso-8859-1")
            || encoding_name.eq_ignore_ascii_case("latin1")
            || encoding_name.eq_ignore_ascii_case("latin-1")
          {
            file_bytes.iter().map(|&b| b as char).collect::<String>()
          } else {
            // Fallback: try UTF-8 with lossy conversion
            String::from_utf8_lossy(&file_bytes).into_owned()
          };
          // Perl: replace replacement chars with space and warn
          let replaced = file_str.replace('\u{FFFD}', " ");
          if replaced.len() != file_str.len() {
            Info!("misdefined", &encoding_name,
              s!("input isn't valid under encoding {}", &encoding_name), "", "",
              self.get_location());
          }
          self.buffer = Mouth::split_lines(&replaced);
        } else {
          // no encoding, interpret as unicode!
          match str::from_utf8(&file_bytes) {
            Ok(file_str) => {
              self.buffer = Mouth::split_lines(file_str);
            },
            Err(e) => {
              let message = s!("input isn't valid under encoding utf8: {:?}", e);
              Info!("misdefined", "utf8", message, "", "", self.get_location());
              let file_str = String::from_utf8_lossy(&file_bytes);
              // Perl: replace \x{FFFD} with space
              let replaced = file_str.replace('\u{FFFD}', " ");
              self.buffer = Mouth::split_lines(&replaced);
            },
          };
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
      if cc == Catcode::SUPER && self.colno + 1 < self.nchars && Some(&ch) == self.chars.get(self.colno) {
        let c1_opt = self.chars.get(self.colno + 1);
        let c2_opt = self.chars.get(self.colno + 2);
        let mut two_hex = false;
        // ^^ followed by TWO LOWERCASE Hex digits???
        if let Some(c1) = c1_opt {
          if let Some(c2) = c2_opt {
            if (self.colno + 2 < self.nchars)
              && LOWERHEX_REGEX.is_match(&c1.to_string())
              && LOWERHEX_REGEX.is_match(&c2.to_string())
            {
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
    !self.is_eol()
      || !self.buffer.is_empty()
      || (self.reader.is_some()
        && !self
          .reader
          .as_mut()
          .unwrap()
          .fill_buf()
          .expect("fill_buf should have no reason to fail.")
          .is_empty())
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
        if self.lineno.is_multiple_of(READLINE_PROGRESS_QUANTUM) && lookup_bool("INCLUDE_COMMENTS") {
          return Some(T_COMMENT!(s!(
            "**** {} Line {} ****",
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
      Some(T_COMMENT!(trimmed_comment))
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

  fn gid() -> usize {
    // assume all mouths are spawned by a single thread, in which case
    // this expedient global counter is safe.
    unsafe {
      LASTID += 1;
      LASTID
    }
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

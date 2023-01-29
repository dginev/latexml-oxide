use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::str;
use std::sync::{Mutex, RwLock};
use tinyvec::ArrayVec;

use core::ops::RangeBounds;
// TODO:
// use encoding::all::ISO_8859_1;
// use encoding::{EncoderTrap, Encoding};
use lazy_static::lazy_static;
use regex::Regex;

use crate::common::error::*;
use crate::common::locator::Locator;
use crate::common::numeric_ops::NumericOps;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::state::{Catcodes, Scope, State, StateOptions};
use crate::token::*;
use crate::tokens::Tokens;
use crate::util::pathname;

lazy_static! {
  static ref STY_STATE: RwLock<State> = RwLock::new(State::new(StateOptions {
    catcodes: Some(Catcodes::Style),
    ..StateOptions::default()
  }));
  static ref STD_STATE: RwLock<State> = RwLock::new(State::new(StateOptions {
    catcodes: Some(Catcodes::Standard),
    ..StateOptions::default()
  }));
  static ref CS_ENDLINECHAR: Token = T_CS!("\\endlinechar");
  static ref TRAILING_SPACE_CHARS: Regex = Regex::new("(?s) +$").unwrap();
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum FoodType {
  File,
  // Binding,
  HTTP,
  HTTPS,
  Literal,
}

impl FoodType {
  /// TODO: Should be a From trait implementation, but am not allowed due to both &str and Option being external. Argh.
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

lazy_static! {
  static ref LASTID: Mutex<u32> = Mutex::new(0);
  static ref LINEBREAK_REGEX: Regex = Regex::new(r"(?s)\r\n|\r|\n").unwrap();
  static ref LOWERHEX_REGEX: Regex = Regex::new(r"^[0-9a-f]$").unwrap();
  static ref SANITIZE_LINE_REGEX: Regex = Regex::new(r"((\\ )*)\s*$").unwrap();
}

#[derive(Debug, Default)]
pub struct MouthOptions {
  pub fordefinitions: bool,
  pub notes: bool,
  pub content: Option<String>,
  pub foodtype: Option<FoodType>,
  pub source: Option<String>,
  pub shortsource: Option<String>,
}

#[derive(Debug)]
pub struct Mouth {
  fordefinitions: bool,
  notes: bool,
  at_eof: bool,
  nchars: usize,
  colno: usize,
  lineno: usize,
  foodtype: FoodType,
  saved_at_cc: Option<Catcode>,
  saved_include_comments: Option<bool>,
  note_message: Option<String>,
  source: String,
  shortsource: String,
  skipping_spaces: bool,
  // pub handle : Option<File>,
  chars: VecDeque<char>,
  buffer: VecDeque<String>,
  reader: Option<BufReader<File>>,
}

impl PartialEq for Mouth {
  fn eq(&self, other: &Mouth) -> bool { self.source == other.source }
}

impl Default for Mouth {
  fn default() -> Self {
    Mouth {
      notes: false,
      note_message: None,
      fordefinitions: false,
      at_eof: false,
      skipping_spaces: false,
      lineno: 0,
      colno: 0,
      chars: VecDeque::new(),
      nchars: 0,
      source: s!("Anonymous String {}", &Mouth::gid()),
      shortsource: s!("String"),
      // handle : None,
      foodtype: FoodType::File,
      saved_at_cc: None,
      saved_include_comments: None,
      buffer: VecDeque::new(),
      reader: None,
    }
  }
}

impl fmt::Display for Mouth {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "Mouth[{}]", self.source) }
}
impl Object for Mouth {
  fn stringify(&self) -> String { s!("Mouth[<string>{}x{}]", self.lineno, self.colno) }
  fn get_locator(&self) -> Option<Cow<Locator>> {
    let (to_line, to_column) = (self.lineno, self.colno);
    let max_col = if self.nchars > 0 { self.nchars - 1 } else { self.nchars }; // There is always a trailing EOL char, if any
    let (from_line, from_column) = if to_column > 0 && to_column >= max_col {
      (to_line, 0)
    } else {
      (to_line, to_column)
    };
    Some(Cow::Owned(Locator::new(self.source.clone(), from_line, from_column, to_line, to_column)))
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
  // DG: For now we are using a `foodtype` field instead of subclassing mouth, as it feels more compact in this particular application
  //     we're really looking at a unified Mouth application logic, with a capacity of reading different kinds of sources
  pub fn create(source: &str, mut options: MouthOptions, state: &mut State) -> Result<Self> {
    if let Some(content) = options.content.take() {
      // we've cached the content of this source
      let (dir, name, ext) = pathname::split(source);
      options.source = Some(source.to_string());
      options.shortsource = Some(s!("{}.{}", name, ext));
      Mouth::new(&content, Some(options), state)
    } else if source.starts_with("literal:") {
      let source = source.replacen("literal:", "", 1);
      // we've supplied literal data
      options.source = None; // the source does not have a corresponding file name
      options.foodtype = FoodType::opt_from_str("literal");
      Mouth::new(&source, Some(options), state)
    } else if source.is_empty() {
      Mouth::new("", Some(options), state)
    } else {
      options.foodtype = FoodType::opt_from_str(&pathname::protocol(source));
      options.source = Some(source.to_string());
      Mouth::new(source, Some(options), state)
    }
  }

  pub fn new(text: &str, options: Option<MouthOptions>, state: &mut State) -> Result<Self> {
    let mut mouth = match options {
      None => Mouth {
        foodtype: FoodType::Literal,
        ..Mouth::default()
      },
      Some(opts) => Mouth {
        foodtype: opts.foodtype.unwrap_or(FoodType::Literal),
        fordefinitions: opts.fordefinitions,
        notes: opts.notes,
        source: opts.source.unwrap_or_default(),
        ..Mouth::default()
      },
    };
    mouth.open(text, state)?;
    Ok(mouth)
  }

  pub fn get_source(&self) -> &str { &self.source }

  pub fn open(&mut self, content: &str, state: &mut State) -> Result<()> {
    match self.foodtype {
      FoodType::File => self.open_file(content)?,
      FoodType::Literal => self.open_literal(content),
      FoodType::HTTP => self.open_http(content),
      FoodType::HTTPS => self.open_https(content),
    };
    self.initialize(state);
    Ok(())
  }

  fn open_file(&mut self, pathname: &str) -> Result<()> {
    if self.foodtype == FoodType::File {
      // TODO: Handle errors
      //   Fatal('I/O', $pathname, $self, "File $pathname is not readable."); }
      // elsif ((!-z $pathname) && (-B $pathname)) {
      //   Fatal('I/O', $pathname, $self, "Input file $pathname appears to be binary."); }
      // open($IN, '<', $pathname)
      //   || Fatal('I/O', $pathname, $self, "Can't open $pathname for reading", $!);
      let mut f = match File::open(pathname) {
        Ok(handle) => handle,
        Err(e) => {
          if e.kind() == io::ErrorKind::NotFound {
            fatal!(Mouth, MissingFile, s!("Can't find file {}", pathname));
          } else {
            return Err(e.into());
          }
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
    unimplemented!();
  }
  fn open_https(&mut self, _content: &str) {
    unimplemented!();
  }
  // fn open_binding(&mut self, _content: &str) {}

  fn initialize(&mut self, state: &mut State) {
    self.note_message = if self.notes {
      if self.fordefinitions {
        Some(s!("Processing definitions"))
      } else {
        Some(s!("Processing content"))
      }
    } else {
      None
    };
    if self.fordefinitions {
      self.saved_at_cc = state.lookup_catcode('@');
      self.saved_include_comments = match state.lookup_value("INCLUDE_COMMENTS") {
        Some(Stored::Bool(x)) => Some(*x),
        _ => None,
      };
      state.assign_catcode('@', Catcode::LETTER, None);
      state.assign_value("INCLUDE_COMMENTS", false, Some(Scope::Local));
    }
  }
  pub fn finish(&mut self, state: &mut State) {
    self.buffer = VecDeque::new();
    self.chars = VecDeque::new();
    self.lineno = 0;
    self.colno = 0;
    self.nchars = 0;
    if self.fordefinitions {
      if let Some(cc) = self.saved_at_cc {
        state.assign_catcode('@', cc, None);
      }
      if let Some(sic) = self.saved_include_comments {
        state.assign_value("INCLUDE_COMMENTS", sic, Some(Scope::Local))
      }
    }
    if self.notes {
      if let Some(ref msg) = self.note_message {
        note_end(msg);
      }
    }
    self.reader.take(); // if we have a reader, this will force a Drop at the end of finish(), which will close the file handle
  }
  // Auxiliaries

  /// This is (hopefully) a platform independent way of splitting a string
  /// into "lines" ending with CRLF, CR or LF (DOS, Mac or Unix).
  /// Note that TeX considers newlines to be \r, ie CR, ie ^^M
  fn split_lines(lines: &str) -> VecDeque<String> {
    let mut lines: VecDeque<String> = LINEBREAK_REGEX.split(lines).map(ToString::to_string).collect(); // And split.
    if lines.iter().last() == Some(&String::new()) {
      lines.pop_back();
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

  fn get_next_line(&mut self, state: &State) -> Option<String> {
    if self.buffer.is_empty() {
      if let Some(ref mut reader) = self.reader {
        // file mouth case
        let mut file_bytes = Vec::new();
        let num_bytes = match reader.read_to_end(&mut file_bytes) {
          Ok(count) => count,
          Err(e) => {
            let message = s!("BufReader::read_to_end returned an error: {:?}", e);
            Warn!("mouth", "io", self, state, message);
            0
          },
        };
        self.reader.take(); // remove the now exhausted reader
                            // Note: the original latexml code first split the perl string into lines, and only THEN decoded it
                            // however, executing a rust regex on a Vec<u8> is just not going to be a sane way forward.
                            // we will first decode the read-in bytes to the right String form, and THEN split lines.
                            // as such, decoding is the first action taken on bytes read in from a file.
        if let Some(ref encoding) = state.input_encoding {
          // TODO: What are characters that fail to decode replaced by in Rust?
          // Bruce suggested that for TeX's behaviour we actually should turn such un-decodeable chars to space(?).
          unimplemented!();
          //let message = s!("input isn't valid under encoding {}", encoding);
          //Info!("misdefined", encoding, self, state, message);
          //unsafe { str::from_utf8_unchecked(&line_bytes).to_owned() }
        } else {
          // no encoding, interpret as unicode!
          let file_str = match str::from_utf8(&file_bytes) {
            Ok(fstr) => fstr,
            Err(e) => {
              let message = s!("input isn't valid under encoding utf8: {:?}", e);
              Info!("misdefined", "utf8", self, state, message);
              unsafe { str::from_utf8_unchecked(&file_bytes) }
            },
          };
          self.buffer = Mouth::split_lines(file_str);
        }
      }
    }
    self.buffer.pop_front()
  }

  /// Get the next character & it's catcode from the input,
  /// handling TeX's "^^" encoding.
  /// Note that this is the only place where catcode lookup is done,
  /// and that it is somewhat `inlined'.
  fn get_next_char(&mut self, state: &State) -> Option<(char, Catcode)> {
    if self.colno >= self.nchars {
      return None;
    };
    let ch_opt = self.chars.get(self.colno);
    self.colno += 1;
    if let Some(ch) = ch_opt {
      let mut ch = *ch;
      let mut cc = state.lookup_catcode(ch).unwrap_or(Catcode::OTHER);
      // Possible convert ^^x
      if cc == Catcode::SUPER && Some(&ch) == self.chars.get(self.colno) {
        let c1_opt = self.chars.get(self.colno + 1);
        let c2_opt = self.chars.get(self.colno + 2);
        let mut two_hex = false;
        // ^^ followed by TWO LOWERCASE Hex digits???
        if let Some(c1) = c1_opt {
          if let Some(c2) = c2_opt {
            if (self.colno + 2 < self.nchars) && LOWERHEX_REGEX.is_match(&c1.to_string()) && LOWERHEX_REGEX.is_match(&c2.to_string()) {
              let hex = u8::from_str_radix(&s!("{}{}", c1, c2), 16).unwrap(); // TODO: Maybe Result type warranted here?
              ch = hex as char;
              self.splice(self.colno - 1..self.colno + 3, &[ch]);
              self.nchars -= 3;
              two_hex = true;
            }
          }
        }
        if !two_hex {
          // OR ^^ followed by a SINGLE Control char type code???
          let mut c = self.chars[self.colno + 1];
          let mut cn = c as i32;

          ch = (cn + if cn >= 64 { -64 } else { 64 }) as u8 as char;
          self.splice(self.colno - 1..self.colno + 2, &[ch]);
          self.nchars -= 2;
        }
        cc = state.lookup_catcode(ch).unwrap_or(Catcode::OTHER);
      }
      Some((ch, cc))
    } else {
      None
    }
  }
  pub fn has_more_input(&mut self) -> bool {
    self.colno < self.nchars || !self.buffer.is_empty() || (self.reader.is_some() && !self.reader.as_mut().unwrap().fill_buf().unwrap().is_empty())
  }

  /// Read the next token, or undef if exhausted.
  /// Note that this also returns COMMENT tokens containing source comments,
  /// and also locator comments (file, line# info).
  /// LaTeXML::Core::Gullet intercepts them and passes them on at appropriate times.
  pub fn read_token(&mut self, state: &State) -> Option<Token> {
    loop {
      // Iterate till we find a token, or run out. (use return)
      // ===== Get next line, if we need to.
      if self.colno >= self.nchars {
        self.lineno += 1;
        self.colno = 0;
        let line_opt = self.get_next_line(state);
        // For \read, we have to return something for EOL, and handle implicit final newline
        let read_mode = state.lookup_int("PRESERVE_NEWLINES") > 1;
        let eolch = if let Some(defn) = state.lookup_definition(&CS_ENDLINECHAR) {
          if defn.is_register() {
            if let Some(eol) = defn.value_of(ArrayVec::default(), state) {
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
            state.lookup_catcode(ch).unwrap_or(Catcode::OTHER)
          } else {
            Catcode::OTHER
          };
          let eoftoken = if let Some(eolch_content) = eolch {
            if read_mode && !self.at_eof && !self.source.is_empty() {
              if eolcc == Catcode::EOL {
                Some(T_CS!("\\par"))
              } else {
                Some(Token!(eolch_content, eolcc))
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
        // In state N, skip spaces
        while self.colno < self.nchars {
          let cc_next = match self.chars.get(self.colno) {
            None => Catcode::OTHER,
            Some(c) => match state.lookup_catcode(*c) {
              Some(cc) => cc,
              None => Catcode::OTHER,
            },
          };
          if cc_next == Catcode::SPACE {
            self.colno += 1;
          } else {
            break;
          }
        }
        // If upcoming line is empty, and there is no recognizable EOL, fake one
        if read_mode && (self.colno >= self.nchars) && (eolch.is_none() || eolch != Some('\r')) {
          return Some(T_MARKER!("EOL"));
        }
        // Sneak a comment out, every so often.
        if (self.lineno % 25) == 0 && state.lookup_bool("INCLUDE_COMMENTS") {
          return Some(T_COMMENT!(s!("**** {} Line {} ****", &self.shortsource, &self.lineno.to_string())));
        }
      }
      if self.skipping_spaces {
        // In state S, skip spaces
        let mut cc = None;
        // This is very awkward as a loop,
        //  but I had to port the Perl logic without going crazy...
        // tokenizer/verb.tex depends on it.
        while let Some((_, ncc)) = self.get_next_char(state) {
          if ncc != Catcode::SPACE {
            cc = Some(ncc);
            break;
          } else {
            cc = None;
          }
        }
        if self.colno <= self.nchars && cc.is_some() && cc != Some(Catcode::SPACE) {
          self.colno -= 1;
        }
        if cc == Some(Catcode::EOL) {
          // If we've got an EOL
          self.get_next_char(state);
          if self.colno < self.nchars {
            self.colno -= 1;
          }
        }
        self.skipping_spaces = false;
      }
      // ==== Extract next token from line.
      if let Some((ch, cc)) = self.get_next_char(state) {
        if let Some(token) = Mouth::dispatch_char(self, ch, cc, state) {
          return Some(token);
        } // Else, repeat till we get something or run out.
      }
    }
  }

  //**********************************************************************
  /// Read all tokens until a token equal to $until (if given), or until exhausted.
  /// Returns an empty Tokens list, if there is no input
  pub fn read_tokens(&mut self, state: &State) -> Tokens {
    let mut tokens = Vec::new();
    while let Some(token) = self.read_token(state) {
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
  pub fn read_raw_line(&mut self, noread: bool, state: &State) -> Option<String> {
    let mut line = String::new();
    if self.colno < self.nchars {
      // DG: Can't slice a VecDeque really? Oh well...
      // Please refactor if you know a better way!
      for (index, c) in self.chars.iter().enumerate() {
        if self.colno <= index && index < self.nchars {
          line.push(*c);
        }
      }
      // End lines with \n, not CR, since the result will be treated as strings
      self.colno = self.nchars;
    } else if noread {
      line = String::new();
    } else {
      match self.get_next_line(state) {
        None => {
          // We've exhausted this mouth
          self.at_eof = true;
          self.chars = VecDeque::new();
          self.nchars = 0;
          self.colno = 0;
          return None;
        },
        Some(next_line) => {
          line = next_line;
          self.lineno += 1;
          self.chars = line.chars().collect();
          self.nchars = self.chars.len();
          self.colno = self.nchars;
        },
      }
    }
    line = line.trim_end().to_owned(); // Even empty lines are valid here
    Some(line)
  }

  fn dispatch_char(&mut self, ch: char, cc: Catcode, state: &State) -> Option<Token> {
    // Possibly want to think about caching (common) letters, etc to keep from
    // creating tokens like crazy... or making them more compact... or ???
    use crate::token::Catcode::*;
    match cc {
      ESCAPE => self.handle_escape(state), // T_ESCAPE
      BEGIN => {
        if ch == '{' {
          Some(T_BEGIN!())
        } else {
          Some(Token!(ch, BEGIN))
        }
      },
      END => {
        if ch == '}' {
          Some(T_END!())
        } else {
          Some(Token!(ch, END))
        }
      },
      MATH => {
        if ch == '$' {
          Some(T_MATH!())
        } else {
          Some(Token!(ch, MATH))
        }
      },
      ALIGN => {
        if ch == '&' {
          Some(T_ALIGN!())
        } else {
          Some(Token!(ch, ALIGN))
        }
      },
      EOL => self.handle_end_of_line(state),
      PARAM => {
        if ch == '#' {
          Some(T_PARAM!())
        } else {
          Some(Token!(ch, PARAM))
        }
      }, // T_PARAM
      SUPER => {
        if ch == '^' {
          Some(T_SUPER!())
        } else {
          Some(Token!(ch, SUPER))
        }
      }, // T_SUPER
      SUB => {
        if ch == '_' {
          Some(T_SUB!())
        } else {
          Some(Token!(ch, SUB))
        }
      }, // T_SUB
      SPACE => self.handle_space(state),
      LETTER => Some(T_LETTER!(ch.to_string())),
      OTHER => Some(T_OTHER!(ch.to_string())),
      ACTIVE => Some(T_ACTIVE!(ch.to_string())),
      COMMENT => self.handle_comment(state),
      INVALID => Some(T_OTHER!(ch.to_string())), // T_INVALID (we could get unicode!)
      _ => None,                                 // IGNORE, others
    }
  }

  fn handle_end_of_line(&mut self, state: &State) -> Option<Token> {
    // Note that newines should be converted to space (with " " for content)
    // but it makes nicer XML with occasional \n. Hopefully, this is harmless?
    let token = if self.colno == 1 {
      T_CS!("\\par")
    } else if state.lookup_int("PRESERVE_NEWLINES") > 0 {
      Token!("\n", Catcode::SPACE)
    } else {
      T_SPACE!()
    };
    self.colno = self.nchars; // Ignore any remaining characters after EOL
    Some(token)
  }

  fn handle_space(&mut self, state: &State) -> Option<Token> {
    // Skip any following spaces!
    while let Some((ch, cc)) = self.get_next_char(state) {
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

  fn handle_comment(&mut self, state: &State) -> Option<Token> {
    let n = self.colno;
    self.colno = self.nchars;
    let mut comment = String::new();
    for c in self.chars.iter().skip(n).take(self.nchars - n) {
      comment.push(*c);
    }
    let trimmed_comment = comment.trim();
    if !trimmed_comment.is_empty() && state.lookup_bool("INCLUDE_COMMENTS") {
      Some(T_COMMENT!(trimmed_comment))
    } else if state.lookup_int("PRESERVE_NEWLINES") > 1 {
      Some(T_MARKER!("EOL")) // Required EOL during \read
    } else {
      None
    }
  }

  //**********************************************************************
  // See The TeXBook, Chapter 8, The Characters You Type, pp.46--47.
  //**********************************************************************

  /// Read control sequence
  fn handle_escape(&mut self, state: &State) -> Option<Token> {
    // NOTE: We're using control sequences WITH the \ prepended!!!
    if let Some((ch, mut cc)) = self.get_next_char(state) {
      // Knuth, p.46 says that Newlines are converted to spaces,
      // Bit I believe that he does NOT mean within control sequences
      let mut cs = s!("\\{}", ch); // I need this standardized to be able to lookup tokens (A better way???)
      if cc == Catcode::LETTER {
        // For letter, read more letters for csname.
        while let Some((nch, ncc)) = self.get_next_char(state) {
          cc = ncc;
          if ncc == Catcode::LETTER {
            cs.push(nch);
          } else {
            break;
          }
        }
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

  fn gid() -> String {
    let mut lastid = LASTID.lock().unwrap();
    *lastid += 1;
    lastid.to_string()
  }

  pub fn is_eol(&mut self, state: &State) -> bool {
    let savecolno = self.colno;
    // We have to peek past any to-be-skipped spaces!!!!
    if self.skipping_spaces {
      let mut cc = None;
      while let Some((_, ncc)) = self.get_next_char(state) {
        if ncc != Catcode::SPACE {
          cc = Some(ncc);
          break;
        } else {
          cc = None;
        }
      }
      if self.colno <= self.nchars && cc.is_some() && cc != Some(Catcode::SPACE) {
        self.colno -= 1;
      }
      if cc == Some(Catcode::EOL) {
        // If we've got an EOL
        self.get_next_char(state);
        if self.colno < self.nchars {
          self.colno -= 1;
        }
      }
    }
    let eol = self.colno >= self.nchars;
    self.colno = savecolno;
    eol
  }

  pub fn at_eof(&self) -> bool { self.at_eof }
}

// WARNING: These two utilities bind $STATE to simple State objects with known fixed catcodes.
// The State normally contains ALL the bindings, etc and links to other important objects.
// We CAN do that here, since we are ONLY tokenizing from a new Mouth, bypassing stomach & gullet.
// However, be careful with any changes.
//
// We also allow for explicitly passing the state in, so that one could memoize state creation
// using lazy_static doesnt work here as State is too complex an object

// Rust note: 1) can we avoid reinitializing a state for each tokenize call? I am not sure if that is actually slow in practice,
// but it ought to be at least suboptimal.
// 2) If we move the $literal argument Tokenize/TokenizeInternal calls into rtx_codegen at compile_time,
// we can bunch them together as a global object in codegen maybe? Then at least we can optimize the compile pass
// + avoid runtime tokenization in the literal binding definitions.

pub fn tokenize(text: &str, state_opt: Option<&mut State>) -> Tokens {
  // special case! empty input is empty Tokens
  if text.is_empty() {
    return Tokens::default();
  }
  match state_opt {
    None => {
      let mut state = STD_STATE.write().unwrap();
      Mouth::new(text, None, &mut state).unwrap().read_tokens(&state)
    },
    Some(s) => Mouth::new(text, None, s).unwrap().read_tokens(s),
  }
}
pub fn tokenize_internal(text: &str, state_opt: Option<&mut State>) -> Tokens {
  // special case! empty input is empty Tokens
  if text.is_empty() {
    return Tokens::default();
  }
  match state_opt {
    None => {
      let mut state = STY_STATE.write().unwrap();
      Mouth::new(text, None, &mut state).unwrap().read_tokens(&state)
    },
    Some(s) => Mouth::new(text, None, s).unwrap().read_tokens(s),
  }
}

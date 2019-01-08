use core::ops::RangeBounds;
use regex::Regex;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::sync::Mutex;

use crate::common::error::*;
use crate::common::store::Stored;
use crate::state::{Catcodes, Scope, State, StateOptions};
use crate::token::*;
use crate::tokens::Tokens;
use crate::util::pathname;

#[derive(PartialEq, Clone)]
pub enum FoodType {
  File,
  Binding,
  HTTP,
  HTTPS,
  Literal,
}

impl FoodType {
  /// TODO: Should be a From trait implementation, but am not allowed due to both &str and Option being external. Argh.
  pub fn from_str(text: &str) -> Option<FoodType> {
    use self::FoodType::*;
    match text.to_lowercase().as_str() {
      "file" => Some(File),
      "binding" => Some(Binding),
      "http" => Some(HTTP),
      "https" => Some(HTTPS),
      "literal" => Some(Literal),
      _ => None,
    }
  }
}

lazy_static! {
  static ref LASTID: Mutex<u32> = Mutex::new(0);
  static ref LINEBREAK_REGEX: Regex = Regex::new(r"\r\n?|\n\r?").unwrap();
  static ref LOWERHEX_REGEX: Regex = Regex::new(r"^[0-9a-f]$").unwrap();
  static ref SANITIZE_LINE_REGEX: Regex = Regex::new(r"((\\ )*)\s*$").unwrap();
}

#[derive(Clone)]
pub struct Mouth {
  pub fordefinitions: bool,
  pub notes: bool,
  pub nchars: usize,
  pub colno: usize,
  pub lineno: usize,
  pub foodtype: FoodType,
  pub saved_at_cc: Option<Catcode>,
  pub saved_include_comments: Option<bool>,
  pub note_message: Option<String>,
  pub source: String,
  pub shortsource: String,
  // pub handle : Option<File>,
  pub chars: VecDeque<char>,
  pub buffer: VecDeque<String>,
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
    }
  }
}

pub struct MouthOptions {
  pub fordefinitions: bool,
  pub content: Option<String>,
  pub foodtype: Option<FoodType>,
  pub source: Option<String>,
  pub shortsource: Option<String>,
}
impl Default for MouthOptions {
  fn default() -> Self {
    MouthOptions {
      fordefinitions: false,
      content: None,
      foodtype: None,
      source: None,
      shortsource: None,
    }
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
    if let Some(content) = options.content.clone() {
      // we've cached the content of this source
      let (dir, name, ext) = pathname::split(source);
      options.source = Some(source.to_string());
      options.shortsource = Some(s!("{}.{}", name, ext));
      Mouth::new(&content, Some(options), state)
    } else if source.starts_with("literal:") {
      // we've supplied literal data
      options.source = None; // the source does not have a corresponding file name
      options.foodtype = FoodType::from_str("literal");
      Mouth::new(source, Some(options), state)
    } else if source.is_empty() {
      Mouth::new("", Some(options), state)
    } else {
      options.foodtype = FoodType::from_str(&pathname::protocol(source));
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
        ..Mouth::default()
      },
    };

    mouth.open(text, state)?;
    Ok(mouth)
  }

  pub fn open<'open>(&'open mut self, content: &str, mut state: &mut State) -> Result<()> {
    match self.foodtype {
      FoodType::File | FoodType::Binding => self.open_file(content)?,
      FoodType::Literal => self.open_literal(content),
      FoodType::HTTP => self.open_http(content),
      FoodType::HTTPS => self.open_https(content),
    };
    self.initialize(&mut state);
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
      let mut content = String::new();
      f.read_to_string(&mut content)?;
      self.open_literal(&content);
    }
    Ok(())
  }
  fn open_literal(&mut self, content: &str) { self.buffer = Mouth::split_lines(content); }
  fn open_http(&mut self, _content: &str) {}
  fn open_https(&mut self, _content: &str) {}
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
        Some(&Stored::Bool(ref x)) => Some(*x),
        _ => None,
      };
      state.assign_catcode('@', Catcode::LETTER, None);
      state.assign_value("INCLUDE_COMMENTS", false, Some(Scope::Local));
    }
    return;
  }
  pub fn finish(&mut self, state: &mut State) {
    self.buffer = VecDeque::new();
    self.lineno = 0;
    self.colno = 0;
    self.chars = VecDeque::new();
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
  }
  // Auxiliaries

  /// This is (hopefully) a platform independent way of splitting a string
  /// into "lines" ending with CRLF, CR or LF (DOS, Mac or Unix).
  /// Note that TeX considers newlines to be \r, ie CR, ie ^^M
  fn split_lines(lines: &str) -> VecDeque<String> {
    LINEBREAK_REGEX.split(lines).map(|s| s.to_string()).collect() // And split.
  }

  /// Original LaTeXML:
  /// This is (hopefully) a correct way to split a line into "chars",
  /// or what is probably more desired is "Grapheme clusters" (even "extended")
  /// These are unicode characters that include any following combining chars, accents & such.
  /// I am thinking that when we deal with unicode this may be the most correct way?
  /// If it's not the way XeTeX does it, perhaps, it must be that ALL combining chars
  /// have to be converted to the proper accent control sequences!
  // sub splitChars {
  //   my ($line) = @_;
  // return $line =~ m/\X/g; }
  // fn split_chars(line : &str) -> Vec<char> {
  //   // I am wondering if this is still needed or we can use a Rust iterator?
  // }

  fn get_next_line(&mut self) -> Option<String> {
    match self.buffer.pop_front() {
      Some(line) => {
        // No CR on last line!
        if self.buffer.is_empty() {
          Some(line.to_string() + "\r")
        } else {
          Some(line.to_string())
        }
      },
      None => None,
    }
  }

  /// Get the next character & it's catcode from the input,
  /// handling TeX's "^^" encoding.
  /// Note that this is the only place where catcode lookup is done,
  /// and that it is somewhat `inlined'.
  fn get_next_char(&mut self, state: &mut State) -> Option<(char, Catcode)> {
    if self.colno >= self.nchars {
      return None;
    };
    let ch_opt = self.chars.get(self.colno);
    self.colno += 1;
    match ch_opt {
      None => None,
      Some(ch) => {
        let mut ch = *ch;
        let mut cc: Option<Catcode> = state.lookup_catcode(ch);
        let next_ch = self.chars.get(self.colno);
        // Possible convert ^^x
        if cc == Some(Catcode::SUPER) && Some(&ch) == next_ch {
          let c1 = self.chars.get(self.colno + 1);
          let c2 = self.chars.get(self.colno + 2);
          // ^^ followed by TWO LOWERCASE Hex digits???
          if (self.colno + 2 < self.nchars)
            && c1.is_some()
            && c2.is_some()
            && LOWERHEX_REGEX.is_match(&c1.unwrap().to_string())
            && LOWERHEX_REGEX.is_match(&c2.unwrap().to_string())
          {
            let hex = u8::from_str_radix(&s!("{}{}", c1.unwrap(), c2.unwrap()), 16).unwrap(); // TODO: Maybe Result type warranted here?
            ch = hex as char;
            self.splice(self.colno - 1..self.colno + 3, &[ch]);
            self.nchars -= 3;
          } else {
            // OR ^^ followed by a SINGLE Control char type code???
            let mut c = self.chars.get(self.colno + 1).unwrap();
            let mut cn = *c as i32;

            ch = (cn + if cn > 64 { -64 } else { 64 }) as u8 as char;
            self.splice(self.colno - 1..self.colno + 2, &[ch]);
            self.nchars -= 2;
          }
          cc = state.lookup_catcode(ch);
        }
        if cc.is_none() {
          cc = Some(Catcode::OTHER);
        }
        Some((ch, cc.unwrap()))
      },
    }
  }
  pub fn has_more_input(&self) -> bool { self.colno < self.nchars || !self.buffer.is_empty() }
  // fn stringify(&self) -> String {
  //   // TODO
  //   s!("mouth stringify")
  // } // This should be an implementation of Debug?
  // fn get_locator(&self, length: usize) -> String {
  //   // TODO
  //   s!("mouth locator")
  // }
  // fn get_source(&self) -> String {
  //   self.source.to_string()
  // }

  /// Read the next token, or undef if exhausted.
  /// Note that this also returns COMMENT tokens containing source comments,
  /// and also locator comments (file, line# info).
  /// LaTeXML::Core::Gullet intercepts them and passes them on at appropriate times.
  pub fn read_token(&mut self, state: &mut State) -> Option<Token> {
    loop {
      // Iterate till we find a token, or run out. (use return)
      // ===== Get next line, if we need to.
      if self.colno >= self.nchars {
        self.lineno += 1;
        self.colno = 0;
        match self.get_next_line() {
          None => {
            // Exhausted the input.
            self.chars = VecDeque::new();
            self.nchars = 0;
            return None;
          },
          Some(line) => {
            // Remove trailing space, but NOT a control space!  End with CR (not \n) since this
            // gets tokenized!
            let sanitized_line = SANITIZE_LINE_REGEX.replace_all(&line, "$1\r");
            self.chars = sanitized_line.chars().collect();
            self.nchars = self.chars.len();
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

            // Sneak a comment out, every so often.
            if (self.lineno % 25) == 0 {
              let include_comments: Option<&Stored> = state.lookup_value("INCLUDE_COMMENTS");
              if let Some(&Stored::Bool(ref x)) = include_comments {
                if *x {
                  return Some(T_COMMENT!(s!("**** {} Line {} ****", &self.shortsource, &self.lineno.to_string())));
                }
              }
            }
          },
        };
      }
      // ==== Extract next token from line.
      match self.get_next_char(state) {
        None => {},
        Some((ch, cc)) => {
          if let Some(token) = Mouth::dispatch_char(self, ch, cc, state) {
            return Some(token);
          } // Else, repeat till we get something or run out.
        },
      }
    }
  }

  //**********************************************************************
  /// Read all tokens until a token equal to $until (if given), or until exhausted.
  /// Returns an empty Tokens list, if there is no input
  pub fn read_tokens(&mut self, until: Option<&Token>, state: &mut State) -> Tokens {
    let mut tokens = Vec::new();
    let has_until = until.is_some();
    let until_string = if let Some(until_token) = until {
      until_token.get_string().to_owned()
    } else {
      String::new()
    };
    while let Some(token) = self.read_token(state) {
      if has_until && token.get_string() == until_string {
        break;
      }
      tokens.push(token);
    }
    while !tokens.is_empty() && tokens.last().unwrap().get_catcode() == Catcode::SPACE {
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
      match self.get_next_line() {
        None => {
          // We've exhausted this mouth
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

  fn dispatch_char(&mut self, ch: char, cc: Catcode, state: &mut State) -> Option<Token> {
    // Possibly want to think about caching (common) letters, etc to keep from
    // creating tokens like crazy... or making them more compact... or ???
    use crate::token::Catcode::*;
    match cc {
      ESCAPE => self.handle_escape(ch, state), // T_ESCAPE
      BEGIN => {
        if ch == '{' {
          Some(T_BEGIN!())
        } else {
          Some(Token {
            text: ch.to_string().into(),
            code: BEGIN,
          })
        }
      }, // T_BEGIN
      END => {
        if ch == '}' {
          Some(T_END!())
        } else {
          Some(Token {
            text: ch.to_string().into(),
            code: END,
          })
        }
      }, // T_END
      MATH => {
        if ch == '$' {
          Some(T_MATH!())
        } else {
          Some(Token {
            text: ch.to_string().into(),
            code: MATH,
          })
        }
      }, // T_MATH
      ALIGN => {
        if ch == '&' {
          Some(T_ALIGN!())
        } else {
          Some(Token {
            text: ch.to_string().into(),
            code: ALIGN,
          })
        }
      }, // T_ALIGN
      EOL => self.handle_end_of_line(ch, state), // T_EOL
      PARAM => {
        if ch == '#' {
          Some(T_PARAM!())
        } else {
          Some(Token {
            text: ch.to_string().into(),
            code: PARAM,
          })
        }
      }, // T_PARAM
      SUPER => {
        if ch == '^' {
          Some(T_SUPER!())
        } else {
          Some(Token {
            text: ch.to_string().into(),
            code: SUPER,
          })
        }
      }, // T_SUPER
      SUB => {
        if ch == '_' {
          Some(T_SUB!())
        } else {
          Some(Token {
            text: ch.to_string().into(),
            code: SUB,
          })
        }
      }, // T_SUB
      SPACE => self.handle_space(ch, state),
      LETTER => Some(T_LETTER!(ch.to_string())),
      OTHER => Some(T_OTHER!(ch.to_string())),
      ACTIVE => Some(T_ACTIVE!(ch.to_string())),
      COMMENT => self.handle_comment(ch, state),
      INVALID => Some(T_OTHER!(ch.to_string())), // T_INVALID (we could get unicode!)
      _ => None,                                 // IGNORE, others
    }
  }

  fn handle_end_of_line(&mut self, _c: char, state: &mut State) -> Option<Token> {
    // Note that newines should be converted to space (with " " for content)
    // but it makes nicer XML with occasional \n. Hopefully, this is harmless?
    let token = if self.colno == 1 {
      T_CS!("\\par")
    } else if state.lookup_bool("PRESERVE_NEWLINES") {
      Token!("\n", Some(Catcode::SPACE))
    } else {
      T_SPACE!()
    };
    self.colno = self.nchars; // Ignore any remaining characters after EOL
    Some(token)
  }

  fn handle_space(&mut self, _c: char, state: &mut State) -> Option<Token> {
    // Skip any following spaces!
    loop {
      match self.get_next_char(state) {
        None => break,
        Some((_ch, cc)) => {
          if (cc != Catcode::SPACE) && (cc != Catcode::EOL) {
            break;
          }
        },
      }
    }
    if self.colno < self.nchars {
      self.colno -= 1;
    }
    Some(T_SPACE!())
  }

  fn handle_comment(&mut self, _c: char, state: &mut State) -> Option<Token> {
    let n = self.colno;
    self.colno = self.nchars;
    let mut comment = String::new();
    // TODO: Probably too slow to do so many .get()s, ideally we want an iterator on a slice.
    for c in n..self.nchars {
      // warning: .. range is half-open in rust
      match self.chars.get(c) {
        None => {},
        Some(c) => comment.push(*c),
      };
    }
    comment.trim();
    // TODO: Handle properly
    let include_comments: bool = match state.lookup_value("INCLUDE_COMMENTS") {
      Some(&Stored::Bool(x)) => x,
      _ => false,
    };
    if !comment.is_empty() && include_comments {
      Some(T_COMMENT!(comment))
    } else {
      None
    }
  }

  //**********************************************************************
  // See The TeXBook, Chapter 8, The Characters You Type, pp.46--47.
  //**********************************************************************

  /// Read control sequence
  fn handle_escape(&mut self, _c: char, state: &mut State) -> Option<Token> {
    // NOTE: We're using control sequences WITH the \ prepended!!!
    let mut cs = s!("\\"); // I need this standardized to be able to lookup tokens (A better way???)
    match self.get_next_char(state) {
      None => {},
      Some((ch, cc)) => {
        // Knuth, p.46 says that Newlines are converted to spaces,
        // Bit I believe that he does NOT mean within control sequences
        cs.push(ch);
        let mut cc_after_letter = None;
        if cc == Catcode::LETTER {
          // For letter, read more letters for csname.
          loop {
            match self.get_next_char(state) {
              None => break,
              Some((ch, cc)) => {
                if cc == Catcode::LETTER {
                  cs.push_str(&ch.to_string());
                } else {
                  cc_after_letter = Some(cc);
                  break;
                }
              },
            };
          }
          self.colno -= 1;
        }

        if cc_after_letter == Some(Catcode::SPACE) {
          // We'll skip whitespace here.
          loop {
            match self.get_next_char(state) {
              None => break,
              Some((_ch, cc)) => {
                if cc != Catcode::SPACE {
                  cc_after_letter = Some(cc);
                  break;
                }
              },
            };
          }
          if self.colno < self.nchars {
            self.colno -= 1;
          }
        }

        if cc_after_letter == Some(Catcode::EOL) {
          // If we've got an EOL
          // if in \read mode, leave the EOL to be turned into a T_SPACE
          // TODO: preserve_newlines NYI
          // if state.lookup_value("PRESERVE_NEWLINES") > 1 {
          // else skip it.
          self.get_next_char(state);
          if self.colno < self.nchars {
            self.colno -= 1;
          }
          // }
        }
      },
    };
    Some(T_CS!(cs))
  }

  /// TODO: Can we use/build a generic that does this reliably for VecDeque
  fn splice<R>(&mut self, range: R, with: &[char])
  where R: RangeBounds<usize> {
    let mut v: Vec<char> = self.chars.drain(..).collect();
    v.splice(range, with.into_iter().cloned());
    self.chars = v.into_iter().collect();
  }

  fn gid() -> String {
    let mut lastid = LASTID.lock().unwrap();
    *lastid += 1;
    lastid.to_string()
  }
}

// WARNING: These two utilities bind $STATE to simple State objects with known fixed catcodes.
// The State normally contains ALL the bindings, etc and links to other important objects.
// We CAN do that here, since we are ONLY tokenizing from a new Mouth, bypassing stomach & gullet.
// However, be careful with any changes.
//
// We also allow for explicitly passing the state in, so that one could memoize state creation
// using lazy_static doesnt work here as State is too complex an object

pub fn tokenize(text: &str, state_opt: Option<&mut State>) -> Tokens {
  match state_opt {
    None => {
      let mut std_state = State::new(StateOptions {
        catcodes: Some(Catcodes::Standard),
        ..StateOptions::default()
      });
      Mouth::new(text, None, &mut std_state).unwrap().read_tokens(None, &mut std_state)
    },
    Some(s) => Mouth::new(&text, None, s).unwrap().read_tokens(None, s),
  }
}
pub fn tokenize_internal(text: &str, state_opt: Option<&mut State>) -> Tokens {
  match state_opt {
    None => {
      let mut sty_state = State::new(StateOptions {
        catcodes: Some(Catcodes::Style),
        ..StateOptions::default()
      });
      Mouth::new(text, None, &mut sty_state).unwrap().read_tokens(None, &mut sty_state)
    },
    Some(s) => Mouth::new(&text, None, s).unwrap().read_tokens(None, s),
  }
}

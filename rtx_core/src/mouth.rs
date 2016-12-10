use std::io::prelude::*;
use std::fs::File;
use std::collections::VecDeque;
use std::sync::Mutex;
use regex::Regex;

use common::error::*;
use state::{State, Scope, ObjectStore};
use token::*;

#[derive(PartialEq, Clone)]
pub enum FoodType {
  File,
  Binding,
  HTTP,
  HTTPS,
  Literal,
}

lazy_static! {
  static ref LASTID : Mutex<u32> = Mutex::new(0);
  static ref LINEBREAK_REGEX : Regex = Regex::new(r"(?s:\015\012|\015|\012|\r)").unwrap();
  static ref LOWERHEX_REGEX : Regex = Regex::new(r"^[0-9a-f]$").unwrap();
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
  fn eq(&self, other: &Mouth) -> bool {
    self.source == other.source
  }
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
      source: "Anonymous String".to_string() + &Mouth::gid(),
      shortsource: "String".to_string(),
      // handle : None,
      foodtype: FoodType::File,
      saved_at_cc: None,
      saved_include_comments: None,
      buffer: VecDeque::new(),
    }
  }
}

impl Mouth {
  pub fn open<'open>(&'open mut self, content: &str, mut state: &mut State) {
    match self.foodtype {
      FoodType::File => self.open_file(content),
      FoodType::Literal => self.open_literal(content),
      FoodType::HTTP => self.open_http(content),
      FoodType::HTTPS => self.open_https(content),
      FoodType::Binding => self.open_file(content),
    };
    self.initialize(&mut state);
  }
  fn open_file(&mut self, pathname: &str) {
    match self.foodtype {
      FoodType::File => {
        // TODO: Handle errors
        //   Fatal('I/O', $pathname, $self, "File $pathname is not readable."); }
        // elsif ((!-z $pathname) && (-B $pathname)) {
        //   Fatal('I/O', $pathname, $self, "Input file $pathname appears to be binary."); }
        // open($IN, '<', $pathname)
        //   || Fatal('I/O', $pathname, $self, "Can't open $pathname for reading", $!);


        let mut f = File::open(pathname).unwrap();
        let mut content = String::new();
        match f.read_to_string(&mut content) {
          _ => {}
        };
        self.open_literal(&content);
      }
      _ => {}
    }
  }
  fn open_literal(&mut self, content: &str) {
    self.buffer = Mouth::split_lines(content);
  }
  fn open_http(&mut self, _content: &str) {}
  fn open_https(&mut self, _content: &str) {}
  // fn open_binding(&mut self, _content: &str) {}

  fn initialize(&mut self, state: &mut State) {
    self.note_message = match self.notes {
      true => {
        match self.fordefinitions {
          true => Some("Processing definitions".to_string()),
          false => Some("Processing content".to_string()),
        }
      }
      false => None,
    };
    if self.fordefinitions {
      self.saved_at_cc = state.lookup_catcode(&'@');
      self.saved_include_comments = match state.lookup_value("INCLUDE_COMMENTS") {
        Some(&ObjectStore::Bool(ref x)) => Some(*x),
        _ => None,
      };
      state.assign_catcode('@', Catcode::LETTER, None);
      state.assign_value("INCLUDE_COMMENTS",
                         ObjectStore::Bool(false),
                         Some(Scope::Local));
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
      match self.saved_at_cc.clone() {
        Some(cc) => state.assign_catcode('@', cc, None),
        None => {}
      };
      match self.saved_include_comments {
        Some(sic) => {
          state.assign_value("INCLUDE_COMMENTS",
                             ObjectStore::Bool(sic),
                             Some(Scope::Local))
        }
        None => {}
      };
    }
    if self.notes {
      match self.note_message {
        Some(ref msg) => note_end(msg),
        _ => {}
      };
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
      }
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
        let mut cc: Option<Catcode> = state.lookup_catcode(ch);
        let next_ch = self.chars.get(self.colno);
        if cc == Some(Catcode::SUPER) && // Possible convert ^^x
          next_ch.is_some() && (ch == next_ch.unwrap()) {
          let c1: Option<&char> = self.chars.get(self.colno + 1);
          let c2: Option<&char> = self.chars.get(self.colno + 2);
          if (self.colno + 2 < self.nchars) &&   // ^^ followed by TWO LOWERCASE Hex digits???
            c1.is_some() && c2.is_some() && LOWERHEX_REGEX.is_match(&c1.unwrap().to_string()) && LOWERHEX_REGEX.is_match(&c2.unwrap().to_string()) {
            // TODO
            // ch = chr(hex($c1 . $c2));
            // splice(@{ self.chars }, self.colno - 1, 4, $ch);
            // self.nchars -= 3;
          } else {
            // OR ^^ followed by a SINGLE Control char type code???
            // TODO:
            // let mut c  = self.chars.get(self.colno + 1);
            // let mut cn = ord($c);
            // $ch = chr($cn + ($cn > 64 ? -64 : 64));
            // splice(@{ self.chars }, self.colno - 1, 3, $ch);
            // self.nchars -= 2;
          }
          cc = state.lookup_catcode(ch);
        }
        if cc.is_none() {
          cc = Some(Catcode::OTHER);
        }
        Some((*ch, cc.unwrap()))
      }
    }
  }
  pub fn has_more_input(&self) -> bool {
    self.colno < self.nchars || !self.buffer.is_empty()
  }
  // fn stringify(&self) -> String {
  //   // TODO
  //   "mouth stringify".to_string()
  // } // This should be an implementation of Debug?
  // fn get_locator(&self, length: usize) -> String {
  //   // TODO
  //   "mouth locator".to_string()
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
          }
          Some(line) => {
            // Remove trailing space, but NOT a control space!  End with CR (not \n) since this gets tokenized!
            let sanitized_line = SANITIZE_LINE_REGEX.replace_all(&line, "$1\r");
            self.chars = sanitized_line.chars().collect();
            self.nchars = self.chars.len();
            while self.colno < self.nchars {
              let cc_next = match self.chars.get(self.colno) {
                None => Catcode::OTHER,
                Some(c) => {
                  match state.lookup_catcode(c) {
                    Some(cc) => cc,
                    None => Catcode::OTHER,
                  }
                }
              };
              if cc_next == Catcode::SPACE {
                self.colno += 1;
              } else {
                break;
              }
            }

            // Sneak a comment out, every so often.
            if (self.lineno % 25) == 0 {
              let include_comments: Option<&ObjectStore> = state.lookup_value("INCLUDE_COMMENTS");
              match include_comments {
                Some(&ObjectStore::Bool(ref x)) => {
                  if *x {
                    return Some(T_COMMENT!("**** ".to_string() + &self.shortsource + " Line " + &self.lineno.to_string() + " ****"));
                  }
                }
                _ => {}
              }
            }
          }
        };
      }
      // ==== Extract next token from line.
      match self.get_next_char(state) {
        None => {}
        Some((ch, cc)) => {
          match Mouth::dispatch_char(self, ch, cc, state) {
            Some(token) => {
              return Some(token);
            }
            None => {}// Else, repeat till we get something or run out.
          };
        }
      }
    }
  }

  fn dispatch_char(&mut self, ch: char, cc: Catcode, state: &mut State) -> Option<Token> {
    // Possibly want to think about caching (common) letters, etc to keep from
    // creating tokens like crazy... or making them more compact... or ???
    use token::Catcode::*;
    match cc {
      ESCAPE => self.handle_escape(ch, state), // T_ESCAPE
      BEGIN => {
        if ch == '{' {
          Some(T_BEGIN!())
        } else {
          Some(Token {
            text: ch.to_string(),
            code: BEGIN,
          })
        }
      }    // T_BEGIN
      END => {
        if ch == '}' {
          Some(T_END!())
        } else {
          Some(Token {
            text: ch.to_string(),
            code: END,
          })
        }
      }      // T_END
      MATH => {
        if ch == '$' {
          Some(T_MATH!())
        } else {
          Some(Token {
            text: ch.to_string(),
            code: MATH,
          })
        }
      }     // T_MATH
      ALIGN => {
        if ch == '&' {
          Some(T_ALIGN!())
        } else {
          Some(Token {
            text: ch.to_string(),
            code: ALIGN,
          })
        }
      }    // T_ALIGN
      EOL => self.handle_end_of_line(ch, state),                                                 // T_EOL
      PARAM => {
        if ch == '#' {
          Some(T_PARAM!())
        } else {
          Some(Token {
            text: ch.to_string(),
            code: PARAM,
          })
        }
      }    // T_PARAM
      SUPER => {
        if ch == '^' {
          Some(T_SUPER!())
        } else {
          Some(Token {
            text: ch.to_string(),
            code: SUPER,
          })
        }
      }    // T_SUPER
      SUB => {
        if ch == '_' {
          Some(T_SUB!())
        } else {
          Some(Token {
            text: ch.to_string(),
            code: SUB,
          })
        }
      }      // T_SUB
      IGNORE => None,
      SPACE => self.handle_space(ch, state),
      LETTER => Some(T_LETTER!(ch.to_string())),
      OTHER => Some(T_OTHER!(ch.to_string())),
      ACTIVE => Some(T_ACTIVE!(ch.to_string())),
      COMMENT => self.handle_comment(ch, state),
      INVALID => Some(T_OTHER!(ch.to_string())), // T_INVALID (we could get unicode!)
      _ => None,
    }
  }

  fn handle_end_of_line(&mut self, _c: char, state: &mut State) -> Option<Token> {
    // Note that newines should be converted to space (with " " for content)
    // but it makes nicer XML with occasional \n. Hopefully, this is harmless?
    let token = if self.colno == 1 {
      T_CS!("\\par".to_string())
    } else {
      match state.lookup_value("PRESERVE_NEWLINES") {
        Some(&ObjectStore::Bool(ref x)) => {
          if *x {
            Token!("\n".to_string(), Some(Catcode::SPACE))
          } else {
            T_SPACE!()
          }
        }
        _ => T_SPACE!(),
      }
    };
    self.colno = self.nchars; // Ignore any remaining characters after EOL
    return Some(token);
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
        }
      }
    }
    if self.colno < self.nchars {
      self.colno -= 1;
    }
    return Some(T_SPACE!());
  }

  fn handle_comment(&mut self, _c: char, state: &mut State) -> Option<Token> {
    let n = self.colno;
    self.colno = self.nchars;
    let mut comment = String::new();
    // TODO: Probably too slow to do so many .get()s, ideally we want an iterator on a slice.
    for c in n..(self.nchars - 1) {
      match self.chars.get(c) {
        None => {}
        Some(c) => comment.push_str(&c.to_string()),
      };
    }
    comment.trim();
    // TODO: Handle properly
    let include_comments: bool = match state.lookup_value("INCLUDE_COMMENTS") {
      Some(&ObjectStore::Bool(x)) => x,
      _ => false,
    };
    if !comment.is_empty() && include_comments {
      Some(T_COMMENT!(comment))
    } else {
      None
    }
  }

  fn handle_escape(&mut self, _c: char, state: &mut State) -> Option<Token> {
    // NOTE: We're using control sequences WITH the \ prepended!!!
    let mut cs = "\\".to_string();  // I need this standardized to be able to lookup tokens (A better way???)
    match self.get_next_char(state) {
      None => {}
      Some((ch, cc)) => {
        // Knuth, p.46 says that Newlines are converted to spaces,
        // Bit I believe that he does NOT mean within control sequences
        cs.push_str(&ch.to_string());
        match cc {
          Catcode::LETTER => {
            // For letter, read more letters for csname.
            loop {
              match self.get_next_char(state) {
                None => break,
                Some((ch, cc)) => {
                  if cc == Catcode::LETTER {
                    cs.push_str(&ch.to_string());
                  } else {
                    break;
                  }
                }
              };
            }
            self.colno -= 1;
          }

          Catcode::SPACE => {
            // We'll skip whitespace here.
            loop {
              match self.get_next_char(state) {
                None => break,
                Some((_ch, cc)) => {
                  if cc != Catcode::SPACE {
                    break;
                  }
                }
              };
            }
            if self.colno < self.nchars {
              self.colno -= 1;
            }
          }

          Catcode::EOL => {
            // If we've got an EOL
            // if in \read mode, leave the EOL to be turned into a T_SPACE
            // TODO: preserve_newlines NYI
            let preserve_newlines: bool = state.lookup_value("PRESERVE_NEWLINES").is_some();
            if !preserve_newlines {
              // else skip it.
              self.get_next_char(state);
              if self.colno < self.nchars {
                self.colno -= 1;
              }
            }
          }
          _ => {}
        };
      }
    };
    Some(T_CS!(cs))
  }

  fn gid() -> String {
    let mut lastid = LASTID.lock().unwrap();
    *lastid = *lastid + 1;
    lastid.to_string()
  }

}

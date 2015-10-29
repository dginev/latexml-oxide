use std::fs::File;
use std::hash::Hash;
use std::path::Path;
use std::collections::VecDeque;

use common::error::*;
use state::{State};
use core::token::*;

#[derive(PartialEq)]
pub enum FoodType {
  File,
  Binding,
  HTTP,
  HTTPS,
  Literal
}

pub struct Mouth {
  pub fordefinitions : bool,
  pub notes : bool,
  pub nchars : usize,
  pub colno : usize,
  pub lineno : usize,
  pub foodtype : FoodType,
  saved_at_cc : Option<Catcode>,
  saved_include_comments : Option<bool>,
  note_message : Option<String>,
  pub source : String,
  pub shortsource : String,
  handle : Option<File>,
  chars : VecDeque<char>,
  buffer : VecDeque<String>
}

impl Default for Mouth {
  fn default() -> Self {
    Mouth {
      notes : false,
      note_message : None,
      fordefinitions : false,
      lineno : 0,
      colno : 0,
      chars : VecDeque::new(),
      nchars : 0,
      source : "Anonymous String".to_string(),
      shortsource : "String".to_string(),
      handle : None,
      foodtype : FoodType::File,
      saved_at_cc : None,
      saved_include_comments : None,
      buffer : VecDeque::new()
    }
  }
}

impl Mouth {
  fn open(&mut self, content : &str, mut state : &mut State) {
    match self.foodtype {
      FoodType::File => self.open_file(content),
      FoodType::Literal => self.open_literal(content),
      FoodType::HTTP => self.open_http(content),
      FoodType::HTTPS => self.open_https(content),
      FoodType::Binding => self.open_file(content)
    };
    self.initialize(&mut state);
  }
    fn open_file(&mut self, pathname : &str) {
    match self.foodtype {
      FoodType::File => {
        // TODO
      }
      _ => {}
    }
  }
  fn open_literal(&mut self, content : &str) {
    self.buffer = Mouth::split_lines(content);
  }
  fn open_http(&mut self, content : &str ) {}
  fn open_https(&mut self, content : &str ) {}
  fn open_binding(&mut self, content : &str ) {}

  
  fn initialize(&mut self, state : &mut State) {
    self.note_message = match self.notes {
      true => match self.fordefinitions {
        true => Some("Processing definitions".to_string()),
        false => Some("Processing content".to_string())
      },
      false => None
    };
    if self.fordefinitions {
      self.saved_at_cc = state.lookup_catcode(&'@');
      self.saved_include_comments = match state.lookup_value("include_comments") {
        None => None,
        Some(x) => *x
      };
      state.assign_catcode(&'@', Catcode::LETTER);
      state.assign_value("include_comments",Box::new(0)); 
    }
    return;
  }
  fn finish(&mut self, state : &mut State) {
    self.buffer = VecDeque::new();
    self.lineno = 0;
    self.colno = 0;
    self.chars = VecDeque::new();
    self.nchars = 0;
    if self.fordefinitions {
      match self.saved_at_cc.clone() {
        Some(cc) => state.assign_catcode(&'@', cc),
        None => {}
      };
      match self.saved_include_comments {
        Some(sic) => state.assign_value("include_comments", Box::new(sic)),
        None => {}
      };
    }
    if self.notes && self.note_message.is_some() {
      note_end(self.note_message.clone().unwrap());
    }
  }
  // Auxiliaries

  /// This is (hopefully) a platform independent way of splitting a string
  /// into "lines" ending with CRLF, CR or LF (DOS, Mac or Unix).
  /// Note that TeX considers newlines to be \r, ie CR, ie ^^M
  fn split_lines(lines : &str) -> VecDeque<String> {
    // regexes:
    let linebreak_regex = regex!(r"(?s:\015\012|\015|\012|\r)");
    linebreak_regex.split(lines).map(|s| s.to_string()).collect() // And split.
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
      None => None
    }
  }


  /// Get the next character & it's catcode from the input,
  /// handling TeX's "^^" encoding.
  /// Note that this is the only place where catcode lookup is done,
  /// and that it is somewhat `inlined'.
  fn get_next_char(&mut self, state :&mut State) -> Option<(char,Catcode)> {
    if self.colno >= self.nchars { return None };
    let mut ch_opt = self.chars.get(self.colno);
    self.colno+=1;
    match ch_opt {
      None => None,
      Some(ch) => {
        let mut cc : Option<Catcode> = state.lookup_catcode(ch);
        let next_ch = self.chars.get(self.colno);
        if cc == Some(Catcode::SUPER) && // Possible convert ^^x
          next_ch.is_some() && (ch == next_ch.unwrap() ) {
          let lowerhex_regex = regex!(r"^[0-9a-f]$");
          let c1 : Option<&char> = self.chars.get(self.colno + 1);
          let c2 : Option<&char> = self.chars.get(self.colno + 2);
          if (self.colno + 2 < self.nchars) &&   // ^^ followed by TWO LOWERCASE Hex digits???
            c1.is_some() && c2.is_some() &&
            lowerhex_regex.is_match(&c1.unwrap().to_string()) && lowerhex_regex.is_match(&c2.unwrap().to_string())
          {
            // TODO
            // ch = chr(hex($c1 . $c2));
            // splice(@{ self.chars }, self.colno - 1, 4, $ch);
            // self.nchars -= 3; 
          }
          else {// OR ^^ followed by a SINGLE Control char type code???
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
  fn stringify(&self) -> String{
    // TODO
    "mouth stringify".to_string()
  } // This should be an implementation of Debug?
  fn get_locator(&self, length : usize) -> String {
    // TODO
    "mouth locator".to_string()
  }
  fn get_source(&self) -> String {
    self.source.to_string()
  }

  fn handle_escape(&self) -> Token {
    // TODO
    T_CS("\\foo")
  }

}
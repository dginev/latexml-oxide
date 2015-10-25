use std::fs::File;
use std::hash::Hash;
use std::path::Path;

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

pub struct Mouth<'mouth> {
  pub fordefinitions : bool,
  pub notes : bool,
  pub nchars : usize,
  pub colno : usize,
  pub lineno : usize,
  pub foodtype : FoodType,
  saved_at_cc : Option<Catcode>,
  saved_include_comments : Option<bool>,
  note_message : Option<&'mouth str>,
  pub source : Option<&'mouth str>,
  pub shortsource : Option<&'mouth str>,
  handle : Option<File>,
  chars : Vec<char>,
  buffer : Vec<char>
}

impl<'mouth> Default for Mouth<'mouth> {
  fn default() -> Self {
    Mouth {
      notes : false,
      note_message : None,
      fordefinitions : false,
      lineno : 0,
      colno : 0,
      chars : Vec::new(),
      nchars : 0,
      source : None,
      shortsource : None,
      handle : None,
      foodtype : FoodType::File,
      saved_at_cc : None,
      saved_include_comments : None,
      buffer : Vec::new()
    }
  }
}

impl<'mouth> Mouth<'mouth> {
  // fn create(source : String, content : Option<String>, fordefinitions: bool, notes: bool);
  
  fn initialize(&mut self, state : &mut State) {
    self.note_message = match self.notes {
      true => match self.fordefinitions {
        true => Some("Processing definitions"),
        false => Some("Processing content")
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
  fn finish(&self) {
    // TODO
  }
  // Auxiliaries

  /// This is (hopefully) a platform independent way of splitting a string
  /// into "lines" ending with CRLF, CR or LF (DOS, Mac or Unix).
  /// Note that TeX considers newlines to be \r, ie CR, ie ^^M
  fn split_lines(lines : &str) -> Vec<&str> {
    // regexes:
    let linebreak_regex = regex!(r"(?s:\015\012|\015|\012|\r)");
    linebreak_regex.split(lines).collect() // And split.
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

  fn get_next_line(&self) -> Option<String> {
    // TODO
    None
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
    false
  }
  fn stringify(&self) -> String{
    // TODO
    "mouth stringify".to_string()
  } // This should be an implementation of Debug?
  fn get_locator(&self, length : usize) -> String {
    // TODO
    "mouth locator".to_string()
  }
  fn get_source(&self) -> Option<String> {
    match self.source {
      None => None,
      Some(s) => Some(s.to_string())
    }
  }

  fn handle_escape(&self) -> Token {
    T_CS("\\foo")
  }

  fn open_file(&self, pathname : String) {
    match self.foodtype {
      FoodType::File => {
        // TODO  
      }
      _ => {}
    }
  }

}
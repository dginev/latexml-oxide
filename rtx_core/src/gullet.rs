use lazy_static::lazy_static;
use log::*;
use regex::Regex;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::glue::Glue;
use crate::common::locator::Locator;
use crate::common::number::Number;

use crate::definition::conditional::ConditionalType;
use crate::definition::register::{NumericOps, RegisterType, RegisterValue};
use crate::definition::Definition;
use crate::mouth::Mouth;
use crate::state::State;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;

lazy_static! {
  static ref DIGIT_RE: Regex = Regex::new(r"[0-9]").unwrap();
  static ref OCT_RE: Regex = Regex::new(r"[0-7]").unwrap();
  static ref HEX_RE: Regex = Regex::new(r"[0-9A-F]").unwrap();
}

#[derive(PartialEq, Clone)]
pub struct MouthRuntime {
  pub autoclose: bool,
  pub mouth: Mouth,
  pub pushback: VecDeque<Token>,
}

pub struct Gullet {
  pub mouth: Option<MouthRuntime>,
  pub mouthstack: VecDeque<MouthRuntime>,
  pub pending_comments: VecDeque<Token>,
}

impl Default for Gullet {
  fn default() -> Self {
    Gullet {
      mouth: None,
      mouthstack: VecDeque::new(),
      pending_comments: VecDeque::new(),
    }
  }
}

impl Gullet {
  /// This flushes a mouth so that it will be automatically closed, next time it's read
  /// Corresponds (I think) to TeX's \endinput
  pub fn flush_mouth(&mut self, state: &mut State) {
    if let Some(ref mut runtime) = self.mouth {
      runtime.mouth.finish(state); // but not close!
      runtime.pushback.drain(..); // And don't read anytyhing more from it.
      runtime.autoclose = true;
    }
    return;
  }

  /// Obscure, but the only way I can think of to End!! (see \bye or \end{document})
  /// Flush all sources (close all pending mouth's)
  pub fn flush(&mut self, state: &mut State) {
    if let Some(ref mut runtime) = self.mouth {
      runtime.mouth.finish(state);
    }
    while !self.mouthstack.is_empty() {
      if let Some(mut entry) = self.mouthstack.pop_front() {
        entry.mouth.finish(state);
      }
    }
    self.mouth = Some(MouthRuntime {
      mouth: Mouth::default(),
      pushback: VecDeque::new(),
      autoclose: true,
    });
    self.mouthstack = VecDeque::new();
  }

  pub fn has_more_input(&self) -> bool {
    match self.mouth {
      Some(ref runtime) => runtime.mouth.has_more_input(),
      None => false,
    }
  }

  pub fn open_mouth(&mut self, mouth: Mouth, autoclose: bool) {
    if let Some(runtime) = self.mouth.take() {
      self.mouthstack.push_front(runtime);
    };
    self.mouth = Some(MouthRuntime {
      mouth,
      autoclose,
      pushback: VecDeque::new(),
    });
  }

  pub fn close_mouth<'close>(&'close mut self, forced: bool, state: &mut State) {
    let mut shift_from_mouthstack = false;
    match self.mouth {
      None => {},
      Some(ref mut runtime) => {
        if !forced && (!runtime.pushback.is_empty()) || runtime.mouth.has_more_input() {
          // TODO:
          // let next = Stringify(self.read_token());
          // Error('unexpected', $next, self, "Closing mouth with input remaining '$next'");
        }
        runtime.mouth.finish(state);
        // I think I can refactor from the original state into this simple assignment, because of
        // the Option type
        shift_from_mouthstack = true;
      },
    }
    if shift_from_mouthstack {
      self.mouth = self.mouthstack.pop_front();
    }
    return;
  }

  pub fn get_locator(&self) -> Locator {
    // TODO
    Locator::default()
  }

  pub fn get_mouth_mut(&mut self) -> Option<&mut Mouth> {
    match self.mouth {
      None => None,
      Some(ref mut runtime) => Some(&mut runtime.mouth),
    }
  }

  //**********************************************************************
  // Not really 100% sure how this is supposed to work
  // See TeX Ch 20, p216 regarding noexpand, \edef with token list registers, etc.
  // Solution: Duplicate param tokens, stick NOTEXPANDED infront of expandable tokens.
  pub fn neutralize_tokens(&mut self, tokens: &[Token], state: &mut State) -> Vec<Token> {
    let mut result = Vec::new();
    for token in tokens.iter() {
      match token.get_catcode() {
        Catcode::PARAM => result.push(token.clone()),
        _ => {
          if let Some(defn) = state.lookup_definition(token) {
            result.push(T_NOTEXPANDED!("\\noexpand"));
          }
        },
      };
      result.push(token.clone());
    }
    result
  }

  ///**********************************************************************
  /// Low-level readers: read token, read expanded token
  ///**********************************************************************
  /// Note that every char (token) comes through here (maybe even twice, through args parsing),
  /// So, be Fast & Clean!  This method only reads from the current input stream (Mouth).
  pub fn read_token(&mut self, state: &mut State) -> Option<Token> {
    let mut next_token: Option<Token> = None;
    // Check in pushback first....
    if let Some(ref mut runtime) = self.mouth {
      while let Some(pushback_token) = runtime.pushback.pop_front() {
        match pushback_token.code {
          Catcode::COMMENT => self.pending_comments.push_back(pushback_token),
          Catcode::MARKER => {
            // TODO:
            // LaTeXML::Definition::stopProfiling($token, 'expand'); } }
          },
          _ => {
            next_token = Some(pushback_token);
            break;
          },
        };
      }
      if next_token.is_none() {
        while let Some(token) = runtime.mouth.read_token(state) {
          match token.code {
            Catcode::COMMENT => self.pending_comments.push_back(token),
            Catcode::MARKER => {
              // TODO:
              // LaTeXML::Definition::stopProfiling($token, 'expand'); } }
            },
            _ => {
              next_token = Some(token);
              break;
            },
          };
        }
      }
      next_token
    } else {
      None
    }
  }

  // Read the next non-expandable token (expanding tokens until there's a non-expandable one).
  // Note that most tokens pass through here, so be Fast & Clean! readToken is folded in.
  // `Toplevel' processing, (if $toplevel is true), used at the toplevel processing by Stomach,
  //  will step to the next input stream (Mouth) if one is available,
  // If $commentsok is true, will also pass comments.
  pub fn read_x_token(&mut self, toplevel: bool, commentsok: bool, state: &mut State) -> Result<Option<Token>> {
    // toplevel should be true by default
    if commentsok && !self.pending_comments.is_empty() {
      return Ok(self.pending_comments.pop_front());
    }

    loop {
      let read_token: Option<Token>;
      let cc: Catcode;
      let mut defn_next: Option<Rc<Definition>> = None;
      let mut needs_close = false;
      let mut return_next = false;
      let mut expand_next = false;
      match self.mouth {
        None => return Ok(None),
        Some(ref mut runtime) => {
          read_token = if !runtime.pushback.is_empty() {
            runtime.pushback.pop_front()
          } else {
            runtime.mouth.read_token(state)
          };
          match read_token {
            None => {
              if !(runtime.autoclose && toplevel && !self.mouthstack.is_empty()) {
                return Ok(None);
              }
              needs_close = true; // Close mouth
            },
            Some(token) => {
              // info!(target:"read_x_token", "at: {:?}", token);
              match token.code {
                Catcode::NOTEXPANDED => {
                  // NOTE: Inlined ->getCatcode
                  // Should only occur IMMEDIATELY after expanding \noexpand (by readXToken),
                  // so this token should never leak out through an EXTERNAL call to readToken.
                  return_next = true; //just return next token
                },
                Catcode::COMMENT => {
                  if commentsok {
                    return Ok(Some(token));
                  } else {
                    self.pending_comments.push_back(token);
                  } // What to do with comments???
                },
                // Catcode::MARKER => {
                //   LaTeXML::Definition::stopProfiling($token, 'expand'); }
                // }
                _ => {
                  let looked_up_definition: Option<Rc<Definition>> = state.lookup_definition(&token);
                  if let Some(defn) = looked_up_definition {
                    if (*defn).is_expandable() && (toplevel || !(*defn).is_protected()) {
                      // is this the right logic here? don't expand unless digesting?
                      state.current_token = Some(Rc::new(token));
                      defn_next = Some(defn);
                      expand_next = true;
                    } else {
                      // info!(target:"read_x_token", "returning x {:?}", token);
                      return Ok(Some(token));
                    }
                  } else {
                    // info!(target:"read_x_token", "returning non-def {:?}", token);
                    return Ok(Some(token));
                  }
                },
              };
            },
          }
        },
      };
      if needs_close {
        // info!(target:"read_x_token", "needs_close");
        self.close_mouth(false, state); // Next input stream.
      } else if return_next {
        // info!(target:"read_x_token", "return_next");
        return Ok(self.read_token(state)); // Just return the next token.
      } else if expand_next {
        // info!(target:"read_x_token", "expand_next");
        // Do the check here, to be more forgiving and more informative
        let expansion = match defn_next {
          Some(defn) => defn.invoke(self, state)?,
          None => Tokens!(),
        };
        // _ => Error("misdefined", token, undef,
        //         "Expected a Token in expansion of " . ToString($token),
        //         "got " . Stringify($_))

        // already checked tokens, so just push to be re-read (like ->unread(@expansion); )
        match self.mouth {
          None => {
            return Ok(None);
          },
          Some(ref mut runtime) => {
            for expansion_token in expansion.unlist().into_iter().rev() {
              runtime.pushback.push_front(expansion_token);
            }
          },
        };
      }
    }
  }

  /// Read the next raw line (string);
  /// primarily to read from the Mouth, but keep any unread input!
  pub fn read_raw_line(&mut self) -> Option<String> {
    // If we've got unread tokens, they presumably should come before the Mouth's raw data
    // but we'll convert them back to string.
    if let Some(ref mut runtime) = self.mouth {
      let tokens: Vec<Token> = runtime.pushback.drain(..).collect();

      // TODO
      // let markers : Vec<&Token> = tokens.iter().filter(|t:Token| t.get_catcode() ==
      // Catcode::MARKER).collect(); if !markers.is_empty() {    // Whoops, profiling markers!

      // @tokens = grep { $_->getCatcode != Catcode::MARKER } @tokens;    // Remove
      // map { LaTeXML::Core::Definition::stopProfiling($_, 'expand') } @markers;
      // }

      // If we still have peeked tokens, we ONLY want to combine it with the remainder
      // of the current line from the Mouth (NOT reading a new line)
      if !tokens.is_empty() {
        Some(Tokens::new(tokens).to_string() + &runtime.mouth.read_raw_line(true).unwrap_or_default())
      } else {
        // Otherwise, read the next line from the Mouth.
        runtime.mouth.read_raw_line(false)
      }
    } else {
      None
    }
  }

  pub fn unread(&mut self, tokens: &Tokens) {
    if let Some(ref mut runtime) = self.mouth {
      for token in tokens.unlist().into_iter().rev() {
        runtime.pushback.push_front(token);
      }
    };
  }

  ///**********************************************************************
  /// Mid-level readers: checking and matching tokens, strings etc.
  ///**********************************************************************
  /// The following higher-level parsing methods are built upon readToken & `.
  pub fn read_non_space(&mut self, state: &mut State) -> Option<Token> {
    loop {
      match self.read_token(state) {
        None => return None,
        Some(t) => {
          if t.code != Catcode::SPACE {
            return Some(t);
          }
        },
      }
    }
  }

  /// Read a sequence of tokens balanced in {}
  /// assuming the { has already been read.
  /// Returns a Tokens list of the balanced sequence, omitting the closing }
  pub fn read_balanced(&mut self, state: &mut State) -> Result<Tokens> {
    let mut tokens = Vec::new();
    let mut level = 1;
    while let Some(t) = self.read_token(state) {
      // TODO: add $expanded flag for read_x_token(0,1) alternative read
      match t.code {
        // Inline ->getCatcode!
        Catcode::BEGIN => level += 1,
        Catcode::END => {
          level -= 1;
          if level <= 0 {
            break;
          }
        },
        // TODO: Marker case
        _ => {},
      };
      tokens.push(t);
    }
    if level > 0 {
      error!(target: "expected:}", "Gullet->readBalanced ran out of input in an unbalanced state.");
    }
    if tokens.is_empty() {
      // Default to empty token list, to signify success (TODO, or improve to
      // Result<Option<Tokens>> ??)
      tokens.push(T_OTHER!(""));
    }
    Ok(Tokens::new(tokens))
  }

  /// Match the input against a set of keywords; Similar to readMatch, but the keywords are strings,
  /// and Case and catcodes are ignored; additionally, leading spaces are skipped.
  /// AND, macros are expanded.
  pub fn read_keyword(&mut self, keywords: &[&str], state: &mut State) -> Result<Option<Tokens>> {
    self.skip_spaces(state);
    for keyword in keywords.iter() {
      let mut to_match: VecDeque<char> = keyword.to_uppercase().chars().collect();
      let mut matched = Vec::new();
      while !to_match.is_empty() {
        if let Some(tok) = self.read_x_token(false, false, state)? {
          let cmp_tok = tok.get_string().to_uppercase();
          matched.push(tok);
          if cmp_tok == to_match[0].to_string() {
            to_match.pop_front();
          } else {
            break;
          }
        } else {
          break;
        }
      }
      if to_match.is_empty() {
        // All matched!!!
        return Ok(Some(T_OTHER!(keyword.to_string()).into()));
      } else {
        self.unread(&matched.into()); // Put 'em back and try next!
      }
    }
    Ok(None)
  }

  /// Return a (balanced) sequence tokens until a match against one of the Tokens in @delims.
  /// In list context, also returns the found delimiter.
  pub fn read_until(&mut self, delims: Vec<Token>, state: &mut State) -> Result<Tokens> {
    let mut n = 0;
    let mut found;
    let mut tokens: Vec<Token> = Vec::new();
    loop {
      found = self.read_match(&delims, state)?;
      if found.is_some() {
        break;
      } else {
        match self.read_token(state) {
          // Copy next token to args
          None => return Ok(Tokens!()),
          Some(token) => {
            let catcode = token.get_catcode();
            tokens.push(token);
            n += 1;
            if catcode == Catcode::BEGIN {
              // And if it's a BEGIN, copy till balanced END
              let mut balanced_tokens = self.read_balanced(state)?.unlist();
              tokens.append(&mut balanced_tokens);
              tokens.push(T_END!());
            }
          },
        }
      }
    }

    // Notice that IFF the arg looks like {balanced}, the outer braces are stripped
    // so that delimited arguments behave more similarly to simple, undelimited arguments.
    if n == 1 && tokens[0].get_catcode() == Catcode::BEGIN {
      tokens = tokens[1..tokens.len() - 1].to_vec();
    }
    Ok(Tokens::new(tokens))
  }

  pub fn read_until_brace(&mut self, state: &mut State) -> Result<Tokens> {
    let mut tokens = Vec::new();
    while let Some(token) = self.read_token(state) {
      if token.code == Catcode::BEGIN {
        // INLINE Catcode
        if let Some(mouth) = self.mouth.as_mut() {
          mouth.pushback.push_front(token); // Unread
        } else {
          fatal!(Mouth, NotFound, "No Mouth in Gullet.read_until_brace")
        }
        break;
      }
      tokens.push(token);
    }
    if tokens.is_empty() {
      tokens.push(T_OTHER!(""));
      // TODO: we need a non-empty Tokens object to pass the success check in parameter::read
      // is there a better approach? Is returning Result<Option<Tokens>> cleaner? (I guess yes ...)
    }
    Ok(Tokens::new(tokens))
  }

  /// Skipping over conditional branches is used heavily when processing raw TeX (eg. tikz).
  pub fn read_next_conditional(&mut self, state: &mut State) -> Option<(Token, ConditionalType)> {
    while let Some(token) = self.read_token(state) {
      if let Some(cond_type) = state.lookup_conditional(&token) {
        return Some((token, cond_type));
      }
    }
    None
  }

  ///**********************************************************************
  /// Higher-level readers: Read various types of things from the input:
  ///  tokens, non-expandable tokens, args, Numbers, ...
  ///**********************************************************************
  pub fn read_arg(&mut self, state: &mut State) -> Result<Tokens> {
    match self.read_non_space(state) {
      None => Ok(Tokens!()),
      Some(token) => {
        match token.code {
          Catcode::BEGIN => {
            // Inline ->getCatcode!
            self.read_balanced(state)
          },
          _ => Ok(Tokens!(token)),
        }
      },
    }
  }
  // Note that this returns an empty array if [] is present,
  // otherwise $default or undef.
  pub fn read_optional(&mut self, state: &mut State) -> Result<Tokens> {
    // TODO: default
    match self.read_non_space(state) {
      None => Ok(Tokens!()),
      Some(t) => {
        if t.code == Catcode::OTHER && t.text == "[" {
          self.read_until(vec![T_OTHER!("]")], state)
        } else {
          self.unread(&Tokens!(t));
          Ok(Tokens!()) // TODO: default
        }
      },
    }
  }

  pub fn if_next(&mut self, token: Token, state: &mut State) -> Result<bool> {
    let mut is_next = false;
    if let Some(tok) = self.read_token(state) {
      is_next = tok == token;
      if let Some(mouth) = self.mouth.as_mut() {
        mouth.pushback.push_front(tok); // Unread
      } else {
        fatal!(Mouth, NotFound, "No Mouth found in Gullet.if_next")
      }
    }
    Ok(is_next)
  }

  //**********************************************************************
  //  Numbers, Dimensions, Glue
  // See TeXBook, Ch.24, pp.269-271.
  //**********************************************************************

  pub fn read_value(&mut self, value_type: RegisterType, state: &mut State) -> Result<RegisterValue> {
    match value_type {
      RegisterType::Number => Ok(self.read_number(state)?.into()),
      RegisterType::Dimension => Ok(self.read_dimension(state)?.into()),
      RegisterType::Glue => Ok(self.read_glue(state)?.into()),
      RegisterType::MuGlue => Ok(self.read_mu_glue(state)?.into()),
      RegisterType::Tokens => Ok(self.read_tokens_value(state)?.into()),
      // TODO: unwrap should be a proper error, value is expected
      RegisterType::Token => Ok(self.read_token(state).unwrap().into()),
      RegisterType::CharDef => Ok(self.read_number(state)?.into()),
      RegisterType::Any => Ok(self.read_arg(state)?.into()),
    }
  }

  pub fn read_register_value(&mut self, value_type: RegisterType, state: &mut State) -> Result<Option<RegisterValue>> {
    match self.read_x_token(false, false, state)? {
      None => Ok(None),
      Some(token) => {
        if let Some(defn) = state.lookup_register_definition(&token) {
          if let Some(mut register_type) = defn.register_type() {
            if register_type == RegisterType::CharDef {
              // CharDefs treated as numbers here
              register_type = RegisterType::Number;
            }
            if register_type == value_type {
              let args: Vec<Token> = defn.read_arguments(self, state)?.iter().map(|ts| ts.into()).collect();
              Ok(defn.value_of(args, state))
            } else {
              self.unread(&Tokens!(token)); // Unread
              Ok(None)
            }
          } else {
            self.unread(&Tokens!(token)); // Unread
            Ok(None)
          }
        } else {
          self.unread(&Tokens!(token)); // Unread
          Ok(None)
        }
      },
    }
  }

  /// Match the input against one of the Token or Tokens in @choices; return the matching one or
  /// undef.
  pub fn read_match(&mut self, choices: &[Token], state: &mut State) -> Result<Option<Token>> {
    for choice in choices {
      let mut to_match: Vec<Token> = choice.unlist().into_iter().rev().collect();
      let mut matched = Vec::new();
      while !to_match.is_empty() {
        match self.read_token(state) {
          None => break,
          Some(token) => {
            let is_to_match: bool = Some(&token) == to_match.last();
            let cc = token.get_catcode();
            matched.push(token);
            if is_to_match {
              to_match.pop();
            } else {
              break;
            }

            if cc == Catcode::SPACE {
              // If this was space, SKIP any following!!!
              while let Some(space_token) = self.read_token(state) {
                if space_token.code != Catcode::SPACE {
                  // Unread non-space and end
                  match self.mouth.as_mut() {
                    Some(mouth) => mouth.pushback.push_front(space_token),
                    None => fatal!(Mouth, NotFound, "No Mouth in Gullet.read_match"),
                  }
                  break;
                } else {
                  matched.push(space_token);
                }
              }
            }
          },
        }
      }
      if to_match.is_empty() {
        return Ok(Some(choice.clone())); // All matched!!!
      } else {
        for matched_token in matched.into_iter().rev() {
          match self.mouth.as_mut() {
            Some(mouth) => mouth.pushback.push_front(matched_token), // Put 'em back and try next!
            None => fatal!(Mouth, NotFound, "No Mouth in Gullet.read_match"),
          }
        }
      }
    }
    Ok(None)
  }

  ///======================================================================
  /// Integer, Number
  ///======================================================================
  /// <number> = <optional signs><unsigned number>
  /// <unsigned number> = <normal integer> | <coerced integer>
  /// <coerced integer> = <internal dimen> | <internal glue>
  pub fn read_number(&mut self, state: &mut State) -> Result<Number> {
    let is_negative = self.read_optional_signs(state)?;
    if let Some(n) = self.read_normal_integer(state)? {
      if is_negative {
        Ok(n.negate())
      } else {
        Ok(n)
      }
    } else {
      // elsif (defined($n = self.readInternalDimension)) { return Number($s * $n->valueOf); }
      // elsif (defined($n = self.readInternalGlue))      { return Number($s * $n->valueOf); }
      // else {
      let next = self.read_token(state);
      warn!(target:"expected:<number>", "Missing number, treated as zero while processing {:?}, next token is {:?}", state.current_token, next);
      if let Some(next) = next {
        self.unread(&Tokens!(next));
      }
      Ok(Number::new(0.0))
    }
  }

  /// <normal integer> = <internal integer> | <integer constant>
  ///   | '<octal constant><one optional space> | "<hexadecimal constant><one optional space>
  ///   | `<character token><one optional space>
  pub fn read_normal_integer(&mut self, state: &mut State) -> Result<Option<Number>> {
    match self.read_x_token(false, false, state)? {
      None => Ok(None),
      Some(token) => {
        let cc = token.get_catcode();
        let mut text = token.get_string().to_string();
        if cc == Catcode::OTHER && text.chars().all(|c| c.is_digit(10)) {
          // Read decimal literal
          text.push_str(&self.read_digits(&DIGIT_RE, true, state)?);
          Ok(Some(Number::new(text.parse::<f32>()?)))
        } else if token == T_OTHER!("'") {
          // Read Octal literal
          let decimal = i32::from_str_radix(&self.read_digits(&OCT_RE, true, state)?, 8)?;
          Ok(Some(Number::new(decimal as f32)))
        } else if token == T_OTHER!("\"") {
          //  Read Hex literal
          let decimal = i32::from_str_radix(&self.read_digits(&HEX_RE, true, state)?, 16)?;
          Ok(Some(Number::new(decimal as f32)))
        } else if token == T_OTHER!("`") {
          //  Read Charcode
          let mut s = match self.read_token(state) {
            None => String::new(),
            Some(next) => next.get_string().to_string(),
          };
          if s.starts_with('\\') {
            s.remove(0);
          }
          let s_char = s.chars().next().unwrap();
          let s_char = s_char as u8;
          let s_char = f32::from(s_char);
          Ok(Some(Number::new(s_char))) //  Only a character token!!! NOT expanded!!!!
        } else {
          self.unread(&Tokens!(token)); // Unread
          self.read_internal_integer(state)
        }
      },
    }
  }

  ///======================================================================
  /// Float, a floating point number.
  /// Similar to factor, but does NOT accept comma!
  /// This is NOT part of TeX, but is convenient.
  pub fn read_float(&mut self, state: &mut State) -> Result<Number> {
    let is_negative = self.read_optional_signs(state)?;
    let s = if is_negative { -1.0 } else { 1.0 };
    let mut string = self.read_digits(&DIGIT_RE, true, state)?;
    match self.read_x_token(false, false, state)? {
      None => {
        warn!(target:"expected:<float>", "Missing number, treated as zero while processing {:?}", state.current_token);
        Ok(Number::new(0.0))
      },
      Some(mut token) => {
        if token.get_string() == "." {
          string = s!("{}.{}", string, self.read_digits(&DIGIT_RE, true, state)?);
          token = self.read_x_token(false, false, state)?.unwrap();
        }

        let mut n_opt: Option<Number>;
        if !string.is_empty() {
          if token.get_catcode() != Catcode::SPACE {
            // Inline ->getCatcode, unread
            self.unread(&Tokens!(token));
          }
          n_opt = Some(string.into());
        } else {
          self.unread(&Tokens!(token)); // Unread
          n_opt = self.read_normal_integer(state)?;
        }
        if let Some(n) = n_opt {
          Ok(Number::new(s * n.value_of()))
        } else {
          warn!(target:"expected:<float>", "Missing number, treated as zero while processing {:?}", state.current_token);
          Ok(Number::new(0.0))
        }
      },
    }
  }

  fn read_internal_integer(&mut self, state: &mut State) -> Result<Option<Number>> {
    match self.read_register_value(RegisterType::Number, state)? {
      None => Ok(None),
      Some(val) => Ok(Some(val.into())),
    }
  }
  fn read_internal_dimension(&mut self, state: &mut State) -> Result<Option<Dimension>> {
    match self.read_register_value(RegisterType::Dimension, state)? {
      None => Ok(None),
      Some(val) => Ok(Some(val.into())),
    }
  }
  fn read_internal_glue(&mut self, state: &mut State) -> Result<Option<Glue>> {
    match self.read_register_value(RegisterType::Glue, state)? {
      None => Ok(None),
      Some(val) => Ok(Some(val.into())),
    }
  }

  //======================================================================
  // Dimensions
  //======================================================================
  // <dimen> = <optional signs><unsigned dimen>
  // <unsigned dimen> = <normal dimen> | <coerced dimen>
  // <coerced dimen> = <internal glue>
  pub fn read_dimension(&mut self, state: &mut State) -> Result<Dimension> {
    let is_negative = self.read_optional_signs(state)?;
    let s = if is_negative { -1.0 } else { 1.0 };
    if let Some(d) = self.read_internal_dimension(state)? {
      Ok(if is_negative { d.negate() } else { d })
    } else if let Some(d) = self.read_internal_glue(state)? {
      Ok(Dimension::new(s * d.value_of()))
    } else if let Some(d) = self.read_factor(state)? {
      let unit = match self.read_unit(state)? {
        Some(u) => u,
        None => {
          warn!(target:"expected:<unit>", "Illegal unit of measure (pt inserted).");
          65536.0
        },
      };
      Ok(Dimension::new(s * d * unit))
    } else {
      warn!(target: "expected:<number>", "Missing number, treated as zero. while processing {:?}", state.current_token);
      Ok(Dimension::new(0.0))
    }
  }

  // <unit of measure> = <optional spaces><internal unit>
  //     | <optional true><physical unit><one optional space>
  // <internal unit> = em <one optional space> | ex <one optional space>
  //     | <internal integer> | <internal dimen> | <internal glue>
  // <physical unit> = pt | pc | in | bp | cm | mm | dd | cc | sp

  /// Read a unit, returning the equivalent number of scaled points,
  fn read_unit(&mut self, state: &mut State) -> Result<Option<f32>> {
    let unit_opt = if let Some(u) = self.read_keyword(&["ex", "em"], state)? {
      self.skip_one_space(state);
      Some(state.convert_unit(u))
    } else if let Some(u) = self.read_internal_integer(state)? {
      Some(u.value_of()) // These are coerced to number=>sp
    } else if let Some(u) = self.read_internal_dimension(state)? {
      Some(u.value_of())
    } else if let Some(u) = self.read_internal_glue(state)? {
      Some(u.value_of())
    } else {
      self.read_keyword(&["true"], state)?; // But ignore, we're not bothering with mag...
      if let Some(u) = self.read_keyword(&["pt", "pc", "in", "bp", "cm", "mm", "dd", "cc", "sp"], state)? {
        self.skip_one_space(state);
        Some(state.convert_unit(u))
      } else {
        None
      }
    };
    Ok(unit_opt)
  }

  //======================================================================
  // Glue
  //======================================================================
  // <glue> = <optional signs><internal glue> | <dimen><stretch><shrink>
  // <stretch> = plus <dimen> | plus <fil dimen> | <optional spaces>
  // <shrink>  = minus <dimen> | minus <fil dimen> | <optional spaces>
  pub fn read_glue(&mut self, state: &mut State) -> Result<Glue> {
    let is_negative = self.read_optional_signs(state)?;
    if let Some(n) = self.read_internal_glue(state)? {
      if is_negative {
        Ok(n.negate())
      } else {
        Ok(n)
      }
    } else {
      let mut d = self.read_dimension(state)?;
      if is_negative {
        d = d.negate();
      }
      let (r1, f1) = match self.read_keyword(&["plus"], state)? {
        Some(v) => self.read_rubber(false, state)?,
        None => (0.0, 0.0),
      };
      let (r2, f2) = match self.read_keyword(&["minus"], state)? {
        Some(v) => self.read_rubber(false, state)?,
        None => (0.0, 0.0),
      };

      Ok(Glue::new(d.value_of())) //TODO:, $r1, $f1, $r2, $f2); } }
    }
  }

  pub fn read_rubber(&mut self, mu: bool, state: &mut State) -> Result<(f32, f32)> {
    let is_negative = self.read_optional_signs(state)?;
    let s = if is_negative { -1.0 } else { 1.0 };
    match self.read_factor(state)? {
      None => {
        let f = if mu {
          self.read_mu_dimension(state)?
        } else {
          self.read_dimension(state)?
        };
        Ok((f.value_of() * s, 0.0))
      },
      Some(f) => {
        match self.read_keyword(&["filll", "fill", "fil"], state)? {
          Some(fil) => Ok((s * f, 0.0)), // TODO: $FILLS{$fil}),
          None => {
            let u = if mu {
              match self.read_mu_unit(state)? {
                None => {
                  warn!(target:"expected<unit>", "Illegal unit of measure (mu inserted).");
                  state.convert_unit("mu")
                },
                Some(v) => v,
              }
            } else {
              match self.read_unit(state)? {
                None => {
                  warn!(target:"expected<unit>", "Illegal unit of measure (pt inserted).");
                  65536.0
                },
                Some(v) => v,
              }
            };
            Ok((s * f * u, 0.0))
          },
        }
      },
    }
  }

  pub fn read_mu_glue(&mut self, _state: &mut State) -> Result<Glue> { unimplemented!() }
  pub fn read_mu_dimension(&mut self, _state: &mut State) -> Result<Dimension> { unimplemented!() }
  pub fn read_mu_unit(&mut self, state: &mut State) -> Result<Option<f32>> {
    if let Some(m) = self.read_keyword(&["mu"], state)? {
      self.skip_one_space(state);
      Ok(Some(state.convert_unit(m)))
    } else if let Some(m) = self.read_internal_mu_glue(state)? {
      Ok(Some(m.value_of()))
    } else {
      Ok(None)
    }
  }
  fn read_internal_mu_glue(&mut self, state: &mut State) -> Result<Option<Glue>> { unimplemented!() }

  /// Apparent behaviour of a token value (ie \toks#=<arg>)
  pub fn read_tokens_value(&mut self, state: &mut State) -> Result<Tokens> {
    match self.read_non_space(state) {
      None => Ok(Tokens!()),
      Some(token) => {
        if token.get_catcode() == Catcode::BEGIN {
          self.read_balanced(state)
        } else if let Some(defn) = state.lookup_register_definition(&token) {
          match defn.register_type() {
            Some(RegisterType::Tokens) | Some(RegisterType::Token) => {
              // TODO: The mismatch between Vec<Tokens> for read_arguments and Vec<Token> for value_of feels incorrect
              //       but in which direction should it be resolved?
              let args: Vec<Token> = defn.read_arguments(self, state)?.iter().map(|ts| ts.into()).collect();
              match defn.value_of(args, state) {
                None => Ok(Tokens!()),
                Some(v) => Ok(v.into()),
              }
            },
            _ => Ok(Tokens!(token)),
          }
        } else if let Some(defn) = state.lookup_definition(&token) {
          // TODO: we are doing two lookups to avoid the type restriction of .read_arguments, any way to circumvent? Is it slow in the first place?
          if defn.is_expandable() {
            let x = defn.invoke(self, state)?;
            if !x.is_empty() {
              self.unread(&x)
            }
            self.read_tokens_value(state)
          } else {
            Ok(Tokens!(token))
          }
        } else {
          Ok(Tokens!(token))
        }
      },
    }
  }

  pub fn skip_spaces(&mut self, state: &mut State) {
    match self.read_non_space(state) {
      None => {},
      Some(t) => {
        self.unread(&Tokens!(t));
      },
    }
  }

  pub fn skip_one_space(&mut self, state: &mut State) {
    if let Some(token) = self.read_token(state) {
      if token.get_catcode() != Catcode::SPACE {
        self.unread(&Tokens!(token));
      }
    }
  }

  pub fn reading_from_mouth<R>(&mut self, mouth: Mouth, state: &mut State, mut reader: Box<FnMut(&mut Gullet, &mut State) -> R>) -> R {
    let mouth_source = mouth.source.clone();
    {
      self.open_mouth(mouth, false); // only allow mouth to be explicitly closed here.
    }
    let results: R = reader(self, state);
    // `mouth` must still be open, with (at worst) empty autoclosable mouths in front of it
    loop {
      let mut is_mouth = false;
      {
        if let Some(ref mut runtime) = self.mouth {
          if runtime.mouth.source == mouth_source {
            is_mouth = true;
          }
        } else {
          error!(target: "unexpected:runtime", "TODO: gullet had no active runtime");
          break;
        }
      }
      if is_mouth {
        self.close_mouth(true, state);
        break;
      } else if self.mouthstack.is_empty() {
        error!(target: "unexpected:<closed>", "TODO: Mouth is unexpectedly already closed");
        // Error('unexpected', '<closed>', $gullet, "Mouth is unexpectedly already closed",
        //   "Reading from " . Stringify($mouth) . ", but it has already been closed.");
        break;
      } else {
        let mut ready_to_read = false;
        {
          if let Some(ref mut runtime) = self.mouth {
            if !runtime.autoclose || !runtime.pushback.is_empty() || runtime.mouth.has_more_input() {
              ready_to_read = true;
            }
          }
        }
        if ready_to_read {
          let _next = self.read_token(state); // stringify( ?
          error!(target: "unexpected:next", "TODO: unexpected input remaining");
          // Error('unexpected', $next, $gullet, "Unexpected input remaining: '$next'",
          //   "Finished reading from " . Stringify($mouth) . ", but it still has input.");
          {
            if let Some(ref mut runtime) = self.mouth {
              runtime.mouth.finish(state);
            }
          }
          self.close_mouth(true, state);
        }
        // ?? if we continue?
        else {
          self.close_mouth(false, state);
        }
      }
    }
    results
  }

  //======================================================================
  // some helpers...

  // <optional signs> = <optional spaces> | <optional signs><plus or minus><optional spaces>
  // returns false if None, or positive, true if negative
  fn read_optional_signs(&mut self, state: &mut State) -> Result<bool> {
    let mut sign = false;
    while let Some(t) = self.read_x_token(false, false, state)? {
      let token_text = t.get_string().to_owned();
      if token_text == "-" {
        sign = true;
      } else if (token_text != "+") && t.get_catcode() != Catcode::SPACE {
        self.unread(&Tokens!(t)); // Unread and end
        break;
      }
    }
    Ok(sign)
  }

  fn read_digits(&mut self, range_regex: &Regex, skip: bool, state: &mut State) -> Result<String> {
    let mut result = String::new();
    while let Some(token) = self.read_x_token(false, false, state)? {
      let digit = token.get_string().to_string();
      if digit.len() == 1 && range_regex.is_match(&digit) {
        result.push_str(&digit);
      } else {
        if !(skip && token.get_catcode() == Catcode::SPACE) {
          self.unread(&Tokens!(token));
        }
        break;
      }
    }
    Ok(result)
  }

  // <factor> = <normal integer> | <decimal constant>
  // <decimal constant> = . | , | <digit><decimal constant> | <decimal constant><digit>
  // Return a number (perl number)
  fn read_factor(&mut self, state: &mut State) -> Result<Option<f32>> {
    let mut factor = self.read_digits(&DIGIT_RE, false, state)?;
    let mut token_opt = self.read_x_token(false, false, state)?;
    if let Some(ref token) = token_opt {
      let token_string = token.get_string();
      if token_string == "." || token_string == "," {
        factor = s!("{}.{}", factor, self.read_digits(&DIGIT_RE, false, state)?);
        token_opt = self.read_x_token(false, false, state)?;
      }
    }

    let factor: f32 = factor.parse::<f32>().unwrap_or(0.0);
    if factor > 0.0 {
      if let Some(token) = token_opt {
        if token.get_catcode() != Catcode::SPACE {
          self.unread(&Tokens!(token));
        }
      }
      Ok(Some(factor))
    } else {
      if let Some(token) = token_opt {
        self.unread(&Tokens!(token));
      }
      match self.read_normal_integer(state)? {
        None => Ok(None),
        Some(n) => Ok(Some(n.value_of())),
      }
    }
  }
}

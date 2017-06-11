use std::collections::VecDeque;
use std::rc::Rc;
use state::{State, ObjectStore};
use common::object::Object;
use common::error::*;
use definition::Definition;
use mouth::Mouth;
use token::{Token, Catcode};
use tokens::Tokens;

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
    if let Some(ref runtime) = self.mouth {
      self.mouthstack.push_front(runtime.clone());
    };
    self.mouth = Some(MouthRuntime {
      mouth: mouth,
      pushback: VecDeque::new(),
      autoclose: autoclose,
    });
  }

  pub fn close_mouth<'close>(&'close mut self, forced: bool, state: &mut State) {
    let mut shift_from_mouthstack = false;
    match self.mouth {
      None => {}
      Some(ref mut runtime) => {
        if !forced && (!runtime.pushback.is_empty()) || runtime.mouth.has_more_input() {
          // TODO:
          // let next = Stringify(self.read_token());
          // Error('unexpected', $next, $self, "Closing mouth with input remaining '$next'");
        }
        runtime.mouth.finish(state);
        // I think I can refactor from the original state into this simple assignment, because of the Option type
        shift_from_mouthstack = true;
      }
    }
    if shift_from_mouthstack {
      self.mouth = self.mouthstack.pop_front();
    }
    return;
  }

  pub fn get_locator(&self) -> String {
    String::new()
  }

  ///**********************************************************************
  /// Low-level readers: read token, read expanded token
  ///**********************************************************************
  /// Note that every char (token) comes through here (maybe even twice, through args parsing),
  /// So, be Fast & Clean!  This method only reads from the current input stream (Mouth).
  pub fn read_token(&mut self, state: &mut State) -> Option<Token> {
    let mut next_token: Option<Token> = None;
    // Check in pushback first....
    match self.mouth {
      None => None,
      Some(ref mut runtime) => {
        loop {
          match runtime.pushback.pop_front() {
            None => break,
            Some(pushback_token) => {
              match pushback_token.code {
                Catcode::COMMENT => self.pending_comments.push_back(pushback_token),
                Catcode::MARKER => {
                  // TODO:
                  // LaTeXML::Definition::stopProfiling($token, 'expand'); } }
                }
                _ => {
                  next_token = Some(pushback_token);
                  break;
                }
              };
            }
          }
        }
        if next_token.is_some() {
          return next_token
        };

        loop {
          match runtime.mouth.read_token(state) {
            None => break,
            Some(token) => {
              match token.code {
                Catcode::COMMENT => self.pending_comments.push_back(token),
                Catcode::MARKER => {
                  // TODO:
                  // LaTeXML::Definition::stopProfiling($token, 'expand'); } }
                }
                _ => {
                  next_token = Some(token);
                  break;
                }
              };
            }
          }
        }

        next_token
      }
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
      return Ok(self.pending_comments.pop_front())
    }

    loop {
      let read_token: Option<Token>;
      let cc: Catcode;
      let mut defn_next: Option<Rc<Definition>> = None;
      let mut needs_close = false;
      let mut return_next = false;
      let mut expand_next = false;
      match self.mouth {
        None => {
          return Ok(None)
        }
        Some(ref mut runtime) => {

          read_token = if runtime.pushback.is_empty() {
            runtime.mouth.read_token(state)
            } else {
              runtime.pushback.pop_front()
            };
          match read_token {
            None => {
              if !(runtime.autoclose && toplevel && !self.mouthstack.is_empty()) {
                return Ok(None)
              }
              needs_close = true; // Close mouth
            }
            Some(token) => {
              cc = token.code;
              match cc {
                Catcode::NOTEXPANDED => {
                  // NOTE: Inlined ->getCatcode
                  // Should only occur IMMEDIATELY after expanding \noexpand (by readXToken),
                  // so this token should never leak out through an EXTERNAL call to readToken.
                  return_next = true; //just return next token
                }
                Catcode::COMMENT => {
                  if commentsok {
                    return Ok(Some(token))
                  } else {
                    self.pending_comments.push_back(token);
                  }    // What to do with comments???
                }
                // Catcode::MARKER => {
                //   LaTeXML::Definition::stopProfiling($token, 'expand'); }
                // }
                _ => {
                  let looked_up_definition: Option<ObjectStore> = state.lookup_definition(&token);
                  match looked_up_definition {
                    Some(defn_store) => {
                      match defn_store {
                        ObjectStore::Expandable(defn) => {
                          if (*defn).is_expandable() && (toplevel || !(*defn).is_protected()) {
                            // is this the right logic here? don't expand unless digesting?
                            state.current_token = Some(token);
                            defn_next = Some(defn.clone());
                            expand_next = true;
                          } else {
                            return Ok(Some(token));
                          }
                        }
                        _ => return Ok(Some(token)),
                      }
                    }
                    None => return Ok(Some(token)),
                  };
                }
              };
            }
          }
        }
      };
      if needs_close {
        self.close_mouth(false, state); // Next input stream.
      } else if return_next {
        return Ok(self.read_token(state));    // Just return the next token.
      } else if expand_next {
        // Do the check here, to be more forgiving and more informative
        let expansion = match defn_next {
          Some(defn) => try!(defn.invoke(self, state)),
          None => Vec::new(),
        };
        // _ => Error("misdefined", token, undef,
        //         "Expected a Token in expansion of " . ToString($token),
        //         "got " . Stringify($_))

        // already checked tokens, so just push to be re-read (like ->unread(@expansion); )
        match self.mouth {
          None => {
            return Ok(None);
          }
          Some(ref mut runtime) => {
            for expansion_token in expansion.into_iter().rev() {
              runtime.pushback.push_front(expansion_token);
            }
          }
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
      let tokens : Vec<Token> = runtime.pushback.drain(..).collect();

      // TODO
      // let markers : Vec<&Token> = tokens.iter().filter(|t:Token| t.get_catcode() == Catcode::MARKER).collect();
      // if !markers.is_empty() {    // Whoops, profiling markers!

        // @tokens = grep { $_->getCatcode != CC_MARKER } @tokens;    // Remove
        // map { LaTeXML::Core::Definition::stopProfiling($_, 'expand') } @markers;
      // }

      // If we still have peeked tokens, we ONLY want to combine it with the remainder
      // of the current line from the Mouth (NOT reading a new line)
      if !tokens.is_empty() {
        Some(Tokens{tokens: tokens}.to_string() + &runtime.mouth.read_raw_line(true).unwrap_or_default())
      } else { // Otherwise, read the next line from the Mouth.
        runtime.mouth.read_raw_line(false)
      }
    } else {
      None
    }
  }

  pub fn unread(&mut self, tokens: Vec<Token>) {
    if let Some(ref mut runtime) = self.mouth {
      for token in tokens.into_iter().rev() {
        runtime.pushback.push_front(token);
      }
    };
  }

  ///**********************************************************************
  /// Mid-level readers: checking and matching tokens, strings etc.
  ///**********************************************************************
  /// The following higher-level parsing methods are built upon readToken & unread.
  pub fn read_non_space(&mut self, state: &mut State) -> Option<Token> {
    loop {
      match self.read_token(state) {
        None => return None,
        Some(t) => {
          if t.code != Catcode::SPACE {
            return Some(t);
          }
        }
      }
    }
  }

  /// Read a sequence of tokens balanced in {}
  /// assuming the { has already been read.
  /// Returns a Tokens list of the balanced sequence, omitting the closing }
  pub fn read_balanced(&mut self, state: &mut State) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut level = 1;
    while level > 0 {
      match self.read_token(state) {
        None => break,
        Some(t) => {
          match t.code { // Inline ->getCatcode!
            Catcode::BEGIN => level += 1,
            Catcode::END => level -= 1,
            _ => {}
          };
          if level > 0 {
            tokens.push(t);
          }
        }
      };
    }
    Ok(tokens)
  }

  /// Return a (balanced) sequence tokens until a match against one of the Tokens in @delims.
  /// In list context, also returns the found delimiter.
  pub fn read_until(&mut self, _delims: Vec<Token>, _state: &mut State) -> Result<Vec<Token>> {
    // my ($n, $found, @tokens) = (0);
    // while (!defined($found = $self->readMatch(@delims))) {
    //   my $token = $self->readToken();    # Copy next token to args
    //   return unless defined $token;
    //   push(@tokens, $token);
    //   $n++;
    //   if ($$token[1] == CC_BEGIN) {      # And if it's a BEGIN, copy till balanced END
    //     push(@tokens, $self->readBalanced->unlist, T_END); } }
    // # Notice that IFF the arg looks like {balanced}, the outer braces are stripped
    // # so that delimited arguments behave more similarly to simple, undelimited arguments.
    // if (($n == 1) && ($tokens[0][1] == CC_BEGIN)) {
    //   shift(@tokens); pop(@tokens); }
    // return (wantarray ? (Tokens(@tokens), $found) : Tokens(@tokens)); }

    // TODO
    Ok(Vec::new())
  }

  pub fn read_until_brace(&mut self, state: &mut State) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    while let Some(token) = self.read_token(state) {
      if token.code == Catcode::BEGIN {    // INLINE Catcode
        if let Some(mouth) = self.mouth.as_mut() {
          mouth.pushback.push_front(token);    // Unread
        } else {
          fatal!(Mouth, NotFound, "No Mouth in Gullet.read_until_brace".to_owned())
        }
        break;
      }
      tokens.push(token);
    }
    Ok(tokens)
  }


  ///**********************************************************************
  /// Higher-level readers: Read various types of things from the input:
  ///  tokens, non-expandable tokens, args, Numbers, ...
  ///**********************************************************************
  pub fn read_arg(&mut self, state: &mut State) -> Result<Vec<Token>> {
    match self.read_non_space(state) {
      None => Ok(Vec::new()),
      Some(token) => {
        match token.code {
          Catcode::BEGIN => {
            // Inline ->getCatcode!
            self.read_balanced(state)
          }
          _ => Ok(vec![token]),
        }
      }
    }
  }
  // Note that this returns an empty array if [] is present,
  // otherwise $default or undef.
  pub fn read_optional(&mut self, state: &mut State) -> Result<Vec<Token>> {
    // TODO: default
    match self.read_non_space(state) {
      None => Ok(Vec::new()),
      Some(t) => {
        if t.code == Catcode::OTHER && t.text == "[" {
          self.read_until(vec![T_OTHER!("]".to_string())], state)
        } else {
          self.unread(vec![t]);
          Ok(Vec::new()) // TODO: default
        }
      }
    }
  }

  pub fn if_next(&mut self, token: Token, state: &mut State) -> Result<bool> {
    let mut is_next = false;
    if let Some(tok) = self.read_token(state) {
      is_next = tok == token;
      if let Some(mouth) = self.mouth.as_mut() {
        mouth.pushback.push_front(tok);  // Unread
      } else {
        fatal!(Mouth, NotFound, "No Mouth found in Gullet.if_next".to_owned())
      }
    }
    Ok(is_next)
  }

  /// Match the input against one of the Token or Tokens in @choices; return the matching one or undef.
  pub fn read_match(&mut self, choices: Vec<Token>, state: &mut State) -> Result<Vec<Token>> {
    for choice in choices {
      let mut to_match : Vec<Token> = choice.unlist().into_iter().rev().collect();
      let mut matched = Vec::new();
      while !to_match.is_empty() {
        match self.read_token(state) {
          None => break,
          Some(token) => {
            matched.push(token.clone());
            if Some(&token) == to_match.last() {
              to_match.pop();
            } else {
              break;
            }

            if token.code == Catcode::SPACE { // If this was space, SKIP any following!!!
              while let Some(space_token) = self.read_token(state) {
                if space_token.code != Catcode::SPACE {
                  break;
                } else {
                  matched.push(space_token);
                }
              }

              match self.mouth.as_mut() {
                Some(mouth) => mouth.pushback.push_front(token), // Unread
                None => fatal!(Mouth, NotFound, "No Mouth in Gullet.read_match".to_owned())
              }
            }
          }
        }
      }
      if to_match.is_empty() {
        return Ok(vec![choice]); // All matched!!!
      } else {
        for matched_token in matched.into_iter().rev() {
          match self.mouth.as_mut() {
            Some(mouth) => mouth.pushback.push_front(matched_token),  // Put 'em back and try next!
            None => fatal!(Mouth, NotFound, "No Mouth in Gullet.read_match".to_owned())
          }
        }
      }
    }
    Ok(Vec::new())
  }



  ///======================================================================
  /// Integer, Number
  ///======================================================================
  /// <number> = <optional signs><unsigned number>
  /// <unsigned number> = <normal integer> | <coerced integer>
  /// <coerced integer> = <internal dimen> | <internal glue>
  pub fn read_number(&mut self, _state: &mut State) -> Result<Vec<Token>> {
    // let s = $self->readOptionalSigns;
    // if (defined(my $n = $self->readNormalInteger)) { return ($s < 0 ? $n->negate : $n); }
    // elsif (defined($n = $self->readInternalDimension)) { return Number($s * $n->valueOf); }
    // elsif (defined($n = $self->readInternalGlue))      { return Number($s * $n->valueOf); }
    // else {
    //   my $next = $self->readToken();
    //   unshift(@{ $$self{pushback} }, $next);    # Unread
    //   Warn('expected', '<number>', $self, "Missing number, treated as zero",
    //     "while processing " . ToString($LaTeXML::CURRENT_TOKEN),
    //     "next token is " . ToString($next));
    //   return Number(0); } }

    // TODO
    Ok(Vec::new())
  }

  pub fn skip_spaces(&mut self, state: &mut State) {
    match self.read_non_space(state) {
      None => {}
      Some(t) => {
        self.unread(vec![t]);
      }
    }
  }
}

//! Base Parameter Types
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;

// ======================================================================
// Define parsers for standard parameter types.
LoadDefinitions!({
  DefParameterType!(Plain, sub[inner, _extra] {
    let mut value = ArgWrap::Tokens(gullet::read_arg(ExpansionLevel::Off)?);
    if let Some(inner_ps) = inner {
      // TODO: How many arguments can we expect back? One? Many?
      //       Currently only passing through the first
      value = inner_ps.reparse_argument(value)?.remove(0);
    }
    Ok(value)
  },
  reversion => sub[arg, inner, _extra] {
    // let mut reverted_inner;
    let mut read_tokens: Vec<Token> = vec![T_BEGIN!()];
    read_tokens.extend(if let Some(inner_ps) = inner {
      inner_ps.revert_arguments(vec![Some(Tokens::new(arg))])?
    } else {
      arg.iter().map(|t| t.revert()).collect()
    });
    read_tokens.push(T_END!());
    Ok(Tokens::new(read_tokens))
  });

  DefParameterType!(DefPlain, sub[inner, _extra] {
    let mut value = ArgWrap::Tokens(gullet::read_balanced(ExpansionLevel::Off, true, true)?);
    if let Some(inner_ps) = inner {
      value = inner_ps.reparse_argument( value)?.remove(0);
    }
    Ok(value)
  },
  reversion => sub[arg, inner, _extra] {
  // let mut reverted_inner;
  let mut read_tokens: Vec<Token> = vec![T_BEGIN!()];
  read_tokens.extend(if let Some(inner_ps) = inner {
    inner_ps.revert_arguments(vec![Some(Tokens::new(arg))])?
  } else {
    arg.iter().map(|t| t.revert()).collect()
  });
  read_tokens.push(T_END!());
  Ok(Tokens::new(read_tokens))
  });

  DefParameterType!(Optional, sub[inner, default] {
    let value = gullet::read_optional(None)?;
    if value.is_none() && !default.is_empty() {
      // TODO: Is the default really multiple Vec<Tokens> ? Or just a single Tokens?
      //       the default[0] is suspicious, compared to the original perl "$default"
      ArgWrap::Tokens(default[0].clone())
    } else if let Some(inner_ps) = inner {
      let mut reparsed = inner_ps.reparse_argument( value.into())?;
      if !reparsed.is_empty() {
        reparsed.remove(0)
      } else {
        ArgWrap::None
      }
    } else {
      value.into()
    }
  },
  optional => true,
  reversion => sub[arg, inner, _extra] {
    // TODO: Same question for the type of "arg" as the one above "default" above:
    //  should this be a single `Tokens` rather than a `Vec<Token>`?
    if !arg.is_empty() {
      let mut read_tokens: Vec<Token> = vec![T_OTHER!("[")];
      read_tokens.extend(match inner {
        None => arg.into_iter().map(Token::revert).collect(),
        Some(inner_ps) => inner_ps.revert_arguments(vec![Some(Tokens::new(arg))])?,
      });
      read_tokens.push(T_OTHER!("]"));
      Ok(Tokens::new(read_tokens))
    } else {
      Ok(Tokens!())
    }
  });

  // This is a peculiar type of argument of the form
  //   <general text> = <filler>{<balanced text><right brace>
  // however, <filler> does get expanded while searching for the initial {
  // which IS required in contrast to a general argument; ie a single token is not correct.
  DefParameterType!(GeneralText, sub[_inner, _extra] {
    gullet::skip_filler()?;
    gullet::read_balanced(ExpansionLevel::Off,false,true)
  });

  // This is like GeneralText, but it Partially expands the argument (not `\protected`, nor `\the`)
  DefParameterType!(XGeneralText, sub[_inner, _extra] {
    gullet::skip_filler()?;
    gullet::read_balanced(ExpansionLevel::Partial,false,true)
  });

  DefParameterType!(Until, sub[_inner, until_extra] {
    // TODO: how many tokens are in extra?
    gullet::read_until(&until_extra[0])
  },
  reversion => sub[arg, _inner, until] {
    let mut rev = Vec::new();
    for t in arg {
      rev.push(t.revert());
    }
    for ts in until {
      rev.extend(ts.clone().revert());
    }
    Ok(Tokens::new(rev))
  });

  // Skip any spaces, but don't contribute an argument.
  DefParameterType!(SkipSpaces, sub[_inner, _extra] {
    gullet::skip_spaces()?;
  }, novalue => true);

  DefParameterType!(Skip1Space, sub[_inner, _extra] {
    gullet::skip_one_space()?;
  }, novalue => true);

  // Read the next token
  DefParameterType!(Token, sub[_inner, _extra] {
    match gullet::read_token()? {
      Some(t) => Ok(ArgWrap::Token(t)),
      None => {
        Error!("expected", "Token", "Paramater <Token> found None.");
        Ok(ArgWrap::Tokens(Tokens!()))
      }
    }
  });

  // Read the next token, after expanding any expandable ones.
  DefParameterType!(XToken, sub[_inner, _extra] {
    if let Some(t) = gullet::read_x_token(None, false, None)? {
      Ok(ArgWrap::Token(t))
    } else {
      Error!("expected","XToken", "Paramater <XToken> found None.");
      Ok(ArgWrap::Tokens(Tokens!()))
    }
  });

  // Perl (2026-03-18): Relation parameter type for numeric comparisons (<, =, >)
  // Perl: $gullet->skipSpaces; return $gullet->readXToken(0, 1);
  //   toplevel=0, for_conditional=1 => autoclose=0, fully_expand=0
  // Skips spaces, then reads with expansion (but not full expansion).
  DefParameterType!(Relation, sub[_inner, _extra] {
    gullet::skip_spaces()?;
    if let Some(t) = gullet::read_x_token(Some(false), true, None)? {
      Ok(ArgWrap::Token(t))
    } else {
      Error!("expected","Relation", "Parameter <Relation> found None.");
      Ok(ArgWrap::Tokens(Tokens!()))
    }
  });

  // Read a number
  DefParameterType!(Number, sub[_inner, _extra] {
    gullet::read_number()?
  });

  // Read a floating point number
  DefParameterType!(Float, sub[_inner, _extra] {
    gullet::read_float()?
  });

  // ??? DG: is this needed?
  // sub ReadFloat {
  //   my ($gullet) = @_;
  //   $gullet->skipSpaces;
  //   return ($gullet->readFloat || Float(0)); }

  // Read a dimension
  DefParameterType!(Dimension, sub[_inner, _extra] {
    gullet::read_dimension()? });
  // Read a Glue (aka skip)
  DefParameterType!(Glue, sub[_inner, _extra] { gullet::read_glue()? });
  // Read a MuDimension (math)
  DefParameterType!(MuDimension, sub[_inner, _extra] {
    gullet::read_mu_dimension()? });
  // Read a MuGlue (math)
  DefParameterType!(MuGlue, sub[_inner, _extra] { gullet::read_mu_glue()? });

  // Read a Pair (x,y) — parenthesized, comma-separated pair of Float values.
  // Perl: ReadPair in latex_constructs.pool.ltxml
  // Returns ArgWrap::Pair if ( is found, ArgWrap::None otherwise (for Optional).
  DefParameterType!(Pair, sub[_inner, _extra] {
    use latexml_core::common::pair::Pair;
    let _ = gullet::skip_spaces();
    if gullet::if_next(T_OTHER!("("))? {
      gullet::read_token()?; // consume (
      let _ = gullet::skip_spaces();
      let x = gullet::read_float()?;
      let _ = gullet::skip_spaces();
      // Skip comma separator
      if let Some(tok) = gullet::read_token()? {
        if tok.to_string() != "," {
          gullet::unread_one(tok);
        }
      }
      let _ = gullet::skip_spaces();
      let y = gullet::read_float()?;
      let _ = gullet::skip_spaces();
      // Skip closing )
      if let Some(tok) = gullet::read_token()? {
        if tok.to_string() != ")" {
          gullet::unread_one(tok);
        }
      }
      let _ = gullet::skip_spaces();
      Ok(ArgWrap::Pair(Pair::new(x, y)))
    } else {
      Ok(ArgWrap::None)
    }
  });

  // Read until the next (balanced) open brace {
  // used for the last TeX-style delimited argument
  DefParameterType!(UntilBrace, sub[_inner, _extra] {
    gullet::read_until_brace()?.unwrap_or_default()
  });

  // Yet another special case: Require a { but do not read it!!!
  DefParameterType!(RequireBrace, sub[_inner, _extra] {
    gullet::read_token()?.inspect(|&tok| {
      gullet::unread_one(tok);
      if tok.get_catcode() != Catcode::BEGIN {
        let err = || {Error!("expected","{","Expected a {{ here."); Ok(())};
        err().ok();
      }
    })
  },
  novalue => true);

  DefParameterType!(XUntil, sub[_inner, untils] {
    // Make sure it's a single token!!!
    let until : Token = untils.first().expect("XUntil needs a token Extra.").into();
    let mut tokens : Vec<Token> = Vec::new();
    while let Some(token) = gullet::read_x_token(Some(false), false, None)? {
      if token == until {
        break;
      } else if token.get_catcode() == Catcode::BEGIN {
        tokens.push(token);
        tokens.extend(gullet::read_balanced(ExpansionLevel::Off,false,false)?.unlist());
        tokens.push(T_END!());
      } else if let Some(defn) = lookup_definition_stored(&token)? {
        let args = defn.read_arguments()?;
        tokens.extend(Invocation!(token, args).unlist());
      } else {
        tokens.push(token);
      }
    }
    Ok(Tokens::new(tokens))
  });

  //  This reads a braced tokens list, expanding as it goes,
  // but expanding \the-like commands only once.
  DefParameterType!(Expanded, sub[_inner, _untils] {
    gullet::read_arg(ExpansionLevel::Full)
  },
  reversion => sub[arg, _inner, _extra] {
    // TODO: Consider a briefer syntax, maybe flat_vec ?
    // https://docs.rs/flat_vec/latest/flat_vec/macro.flat_vec.html
    let mut tks = vec![T_BEGIN!()];
    tks.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
    tks.push(T_END!());
    Ok(Tokens::new(tks))
  });

  // Like Expanded, but defers \protected, and \the expanded only once.
  // Similar to when \edef is used.
  DefParameterType!(ExpandedPartially, sub[_inner, _untils] {
    gullet::read_arg(ExpansionLevel::Partial)
  },
  reversion => sub[arg, _inner, _extra] {
    // TODO: Consider a briefer syntax, maybe flat_vec ?
    // https://docs.rs/flat_vec/latest/flat_vec/macro.flat_vec.html
    let mut tks = vec![T_BEGIN!()];
    tks.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
    tks.push(T_END!());
    Ok(Tokens::new(tks))
  });

  // This reads an expanded definition body,
  // a braced tokens list, expanding as it goes,
  // but expanding \the-like commands only once,
  // and also packing # parameters
  DefParameterType!(DefExpanded, sub[_inner, _extra] {
      gullet::read_balanced(ExpansionLevel::Partial, true, true)
    },
    reversion => sub[arg, _inner, _extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens!(arg).revert(), T_END!())) }
  );

  // Read a matching keyword, eg. Match:=
  // Perl: returns undef on no-match. We must return ArgWrap::None, NOT empty Tokens.
  DefParameterType!(Match, sub[_inner, extra] {
    let extra_refs = extra.iter().collect::<Vec<&Tokens>>();
    match gullet::read_match(&extra_refs)? {
      Some(tks) => ArgWrap::Tokens(tks),
      None => ArgWrap::None,
    }
  });

  // Read a keyword; eg. Keyword:to
  // (like Match, but ignores catcodes)
  // Perl: returns undef on no-match.
  DefParameterType!(Keyword, sub[_inner, extra] {
    let extra_string : String = extra.iter().map(ToString::to_string)
      .collect::<Vec<String>>().join("");
    match gullet::read_keyword(&[&extra_string])? {
      Some(t) => ArgWrap::Tokens(Tokens!(T_OTHER!(t))),
      None => ArgWrap::None,
    }
  });

  // Read balanced material (?)
  DefParameterType!(Balanced, sub[_inner, _extra] {
    gullet::read_balanced(ExpansionLevel::Off,false,false)
  });

  // Read a Semiverbatim argument; ie w/ most catcodes neutralized.
  DefParameterType!(Semiverbatim,
    sub[_inner, _extra] { gullet::read_arg(ExpansionLevel::Off) },
    reversion => sub[arg, inner, _extra] {
      // let mut reverted_inner;
      let mut read_tokens: Vec<Token> = vec![T_BEGIN!()];
      read_tokens.extend(if let Some(inner_ps) = inner {
        inner_ps.revert_arguments(vec![Some(Tokens::new(arg))])?
      } else {
        arg.iter().map(|t| t.revert()).collect()
      });
      read_tokens.push(T_END!());
      Ok(Tokens::new(read_tokens))
    },
    semiverbatim => Some(Vec::new()));

  // Read a LaTeX-style optional argument (ie. in []), but the contents read as Semiverbatim.
  DefParameterType!(OptionalSemiverbatim,
    sub[_inner, _extra] { gullet::read_optional(None) },
    semiverbatim => Some(Vec::new()),
    optional => true,
    reversion => sub[arg, _inner, _extra] {
    if !arg.is_empty() {
      let mut read_tokens = vec![T_OTHER!(s!("["))];
      read_tokens.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
      read_tokens.push(T_OTHER!(s!("]")));
      Ok(Tokens::new(read_tokens))
    } else {
      Ok(Tokens!())
    }
    }
  );

  // Be careful here: if % appears before the initial {, it's still a comment!
  // Also, note that non-typewriter fonts will mess up some chars on digestion!
  DefParameterType!(Verbatim, sub[_inner, _extra] {
      gullet::read_until(&Tokens!(T_BEGIN!()))?;
    begin_semiverbatim(Some(&['%', '\\']));
      let arg = gullet::read_balanced(ExpansionLevel::Off,false,false)?;
      end_semiverbatim()?;
      Ok(arg)
    },
    before_digest => {
      bgroup();
      MergeFont!(family => "typewriter");
    },
    after_digest => {
      egroup()?;
    },
    reversion => sub[arg, _inner, _extra] {
      let mut reverted = vec![T_BEGIN!()];
      reverted.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
      reverted.push(T_END!());
      Ok(Tokens::new(reverted))
    }
  );

  // Read Verbatim, but allows expanding command sequences
  DefParameterType!(HyperVerbatim, sub[_inner, _extra] {
      gullet::read_until(&Tokens!(T_BEGIN!()))?;
    begin_semiverbatim(Some(&['%']));
      DefMacro!(T_CS!("\\%"),              None, T_OTHER!("%"), scope => Some(Scope::Local));
      DefMacro!(T_CS!("\\#"),              None, T_OTHER!("#"), scope => Some(Scope::Local));
      DefMacro!(T_CS!("\\&"),              None, T_OTHER!("&"), scope => Some(Scope::Local));
      DefMacro!(T_CS!("\\textunderscore"), None, T_OTHER!("_"), scope => Some(Scope::Local));
      state::let_i(&T_CS!("\\_"), &T_CS!("\\textunderscore"), None);
      DefMacro!(T_CS!("\\hyper@tilde"), None, T_OTHER!("~"), scope => Some(Scope::Local));
      state::let_i(&T_CS!("\\~"), &T_CS!("\\hyper@tilde"), None);
      state::let_i(&T_CS!("\\textasciitilde"), &T_CS!("\\hyper@tilde"), None);
      state::let_i(&T_CS!("\\\\"), &T_CS!("\\@backslashchar"), None);
      // Having prepared, read in the argument, expanding as we go
      let arg = gullet::read_balanced(ExpansionLevel::Partial,false,false)?;
      end_semiverbatim()?;
      arg
    },
    before_digest => {
      bgroup();
      MergeFont!(family => "typewriter"); },
    after_digest => {
      egroup()?; },
    reversion => sub[arg, _inner, _extra] {
      let mut reverted = vec![T_BEGIN!()];
      reverted.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
      reverted.push(T_END!());
      Ok(Tokens::new(reverted))
    }
  );
  // Read an argument that will not be digested.
  DefParameterType!(Undigested, sub[_inner, _extra] { gullet::read_arg(ExpansionLevel::Off)},
  predigest => sub[arg]{ Ok(arg.undigested()) }
  reversion => sub[arg, _inner, _extra] {
    if arg.is_empty() {
      Ok(Tokens!())
    } else {
      let mut read_tokens = vec!(T_BEGIN!());
      read_tokens.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
      read_tokens.push(T_END!());
      Ok(Tokens::new(read_tokens))
    }
  });

  // Read a LaTeX-style optional argument (ie. in []), but it will not be digested.
  DefParameterType!(OptionalUndigested,
    sub[_inner, _extra] { gullet::read_optional(None) },
    predigest => sub[arg]{ Ok(arg.undigested()) }
    optional => true,
    reversion => sub[arg, _inner, _extra] {
      if arg.is_empty() {
        Ok(Tokens!())
      } else {
        let mut read_tokens = vec!(T_OTHER!("["));
        read_tokens.extend(arg.into_iter().map(Token::revert).collect::<Vec<_>>());
        read_tokens.push(T_OTHER!("]"));
        Ok(Tokens::new(read_tokens))
      }
  });

  // Read a keyword value (KeyVals), that will not be digested.
  DefParameterType!(UndigestedKey, sub[_inner, _extra] {
    gullet::read_arg(ExpansionLevel::Off) },
  predigest => sub[arg]{ Ok(arg.undigested()) });
  DefParameterType!(UndigestedDefKey, sub[_inner, _extra] {
    gullet::read_arg(ExpansionLevel::Off)?.pack_parameters() },
  predigest => sub[arg]{ Ok(arg.undigested()) });

  // Read a token as used when defining it, ie. it may be enclosed in braces.
  DefParameterType!(DefToken, sub[_inner, _extra] {
    let mut token_opt = gullet::read_token()?;
    while let Some(token) = token_opt {
      if token.get_catcode() != Catcode::BEGIN { break; }
      let mut toks : Vec<Token> = gullet::read_balanced(ExpansionLevel::Off,false,false)?
        .unlist().into_iter().filter(|t| {
          let cc = t.get_catcode();
          cc != Catcode::SPACE && cc != Catcode::COMMENT
        }).collect();
      if !toks.is_empty() {
        token_opt = Some(toks.remove(0));
        if !toks.is_empty() {
          gullet::unread_vec(toks);
        }
      } else {
        token_opt = None;
      }
    }
    match token_opt {
      Some(t) => Ok(ArgWrap::Token(t)),
      None => {
        Error!("expected","DefToken",
          "Expected a DefToken parameter, found nothing.");
        Ok(ArgWrap::None)
      }
    }
  },
  predigest => sub[arg]{ Ok(arg.undigested()) });

  // Stub register for misdefinitions, to avoid a cascade of Errors.
  DefRegister!("\\lx@DUMMY@REGISTER", Tokens!());

  // Read a variable, ie. a token (after expansion) that is a writable register.
  DefParameterType!(Variable, sub[_inner, _extra] {
    let token_opt = gullet::read_x_token(None, false, None)?;
    let defn_opt = match token_opt {
      Some(ref token) => state::lookup_register_definition(token),
      None => None
    };
    if let Some(defn) = defn_opt {
        if defn.is_register() && !defn.is_readonly() {
          let args = defn.read_arguments()?;
          // TODO: What is this datatype ?
          // How does it fit the latexml-oxide typed interfaces for parameter types?
          // An extension seems required, also due to the Register parameter type right under.
          // Ok(Tokens!(defn_tok, defn_args))
          Ok(ArgWrap::RegisterDefinition(Box::new((token_opt.unwrap(), args))))
        } else {
          let message = s!("A <variable> was supposed to be here\n Got {:?}", token_opt);
          Error!("expected","<variable>", message);
          Ok(ArgWrap::Tokens(Tokens!()))
        }
    } else {
      let message = s!("A <variable> was supposed to be here\n Got {:?}", token_opt);
      Error!("expected","<variable>", message);
      Ok(ArgWrap::Tokens(Tokens!()))
    }
  },
  reversion => sub[args, _inner, _extra] {
    // Perl: revert Variable by getting CS + reverting register args.
    // The reversion closure receives tokens. Return them as-is.
    Ok(Tokens::new(args))
  });

  DefParameterType!(TeXFileName, sub[_inner, _extra] {
    use Catcode::*;
    gullet::skip_spaces()?;
    let mut tokens = Vec::new();
    while let Some(token) = gullet::read_x_token(Some(false), false, None)? {
      let cc = token.get_catcode();
      if matches!(cc, SPACE | EOL | COMMENT | CS) {
        if matches!(cc, CS) {
          gullet::unread_one(token);
        }
        break
      }
      tokens.push(token);
    }
    // Strip outer "" ???
    let quote = T_OTHER!("\"");
    if tokens.len() > 1 && tokens.first().unwrap() == &quote
      && tokens.last().unwrap() == &quote {
      tokens.remove(0);
      tokens.pop();
    }
    tokens
  });

  DefPrimitive!("\\ltx@loadpool {}", sub[(name)] {
    LoadPool!(&name.to_string());
  });

  // A LaTeX style directory List
  DefParameterType!(DirectoryList, sub[__inner, _extra] {
      // my ($gullet) = @_;
      // $gullet->skipSpaces;
      // if ($gullet->ifNext(T_BEGIN)) {
      //   $gullet->readToken;
      //   my @dirs = ();
      //   $gullet->skipSpaces;
      //   while ($gullet->ifNext(T_BEGIN)) {
      //     # Should these be Semiverbatim ??
      //     push(@dirs, $gullet->readArg);
      //     $gullet->readMatch(T_OTHER(',')); }
      //   $gullet->skipSpaces;
      //   if ($gullet->ifNext(T_END)) {
      //     $gullet->readToken; }
      //   else {
      //     Error('expected', '}', $gullet, "A closing } was supposed to be here"); }
      //   LaTeXML::Core::Array->new(
      //     open => T_BEGIN, close => T_END, itemopen => T_BEGIN, itemclose => T_END,
      //     type => LaTeXML::Package::parseParameters(ToString("Semiverbatim"), "CommaList")->[0],
      //     values => [@dirs]); }
      // else {
      //   Error('expected', 'DirectoryList', $gullet,
      //          "A DirectoryList was supposed to be here"); } });
      // Stub: DirectoryList parameter type not yet ported
      Tokens!()
  });

  // This reads a Box as needed by \raise, \lower, \moveleft, \moveright.
  // Hopefully there are no issues with the box being digested
  // as part of the reader???
  DefParameterType!(MoveableBox, sub[_inner, _extra] {
    gullet::skip_spaces()?;
    if let Some(xtoken) = gullet::read_x_token(None, false, None)? {
      Tokens!(xtoken)
    } else {
      Tokens!()
    }
  }, predigest => sub[arg] {
    let token = arg.unlist().remove(0);
    let mut stuff = stomach::invoke_token(&token)?;
    if !stuff.is_empty() {
      let tbox = stuff.remove(0);
      let csname = match tbox.data() {
        DigestedData::Whatsit(ref w) =>
          w.borrow().definition.get_cs_name().to_string(),
        _ => tbox.to_string()
      };
      if csname != "\\hbox" && csname != "\\vbox" && csname != "\\vtop" {
        let message = s!("A <box> was supposed to be here.\nGot {}", csname);
        Error!("expected","<box>", message);
        None
      } else {
        Some(tbox)
      }
    } else {
      let message = s!("A <box> was supposed to be here.\nGot none.");
      Error!("expected","<box>", message);
      None
    }
  });

  // Read a parenthesis delimited argument.
  // Note that this does NOT balance () within the argument.
  DefParameterType!(BalancedParen, sub[_inner, _extra] {
    let tok_opt = gullet::read_x_token(None,false, None)?;
    let is_paren = match tok_opt {
      Some(ref t) => t.with_str(|ts| ts == "("),
      _ => false
    };
    if is_paren {
      gullet::read_until(&Tokens!(T_OTHER!(")"))).map(Some)
    } else {
      if let Some(tok) = tok_opt {
        gullet::unread_one(tok);
      }
      Ok(None)
    }
  },
  reversion => sub[args, _inner, _extra] {
    Ok(Tokens!(
      T_OTHER!("("), Tokens::new(args).revert(), T_OTHER!(")")
    ))
  });

  // Read a digested argument, digesting as it is being read.
  // The usual macro parameter (generally written as {}) gets tokenized and digested
  // in separate stages, w/o recognizing any special macros or catcode changes within (eg. \url).
  // Rarely, you need a parameter that gets digested AS IT'S READ until ending }.
  // Note that this also recognizes args as \bgroup ... \engroup
  // It is useful when the content would usually need to have been \protect'd
  // in order to correctly deal with catcodes.
  // BEWARE: This is NOT a shorthand for a simple digested {}!

  // Perl PR#2596: TeXDelimiter parameter type for \left, \right, \big, \bigl, etc.
  // Reads like {} (balanced group) for correct math digestion, but reverts WITHOUT braces.
  // Also: unwraps one level of braces, replaces "." with \lx@delimiterdot hint.
  DefParameterType!(TeXDelimiter, sub[_inner, _extra] {
    gullet::skip_filler()?;
    gullet::read_arg(ExpansionLevel::Partial)
  },
  digested_reversion => sub[arg] {
    // Revert without adding braces (unlike {} parameter)
    let mut toks = arg.revert()?;
    // Strip outer braces if present from the reversion
    let list = toks.unlist_ref();
    if list.len() >= 2
      && list.first().map(|t| t.get_catcode()) == Some(Catcode::BEGIN)
      && list.last().map(|t| t.get_catcode()) == Some(Catcode::END)
    {
      // Return inner content without braces
      let inner: Vec<Token> = list[1..list.len()-1].to_vec();
      Ok(Tokens::from(inner))
    } else {
      Ok(toks)
    }
  });

  DefParameterType!(Digested, sub[_inner, _extra] {
      gullet::skip_spaces()?;
      Ok(Tokens!())
    },
    predigest => sub[_arg] {
      let ismath = lookup_bool("IN_MATH");
      let mut list = Vec::new();
      let mut next_token = None;
      while let Some(token) = gullet::read_x_token(Some(false), false, None)? {
        let is_last = token.get_catcode() != Catcode::SPACE && token != T_RELAX!();
        next_token = Some(token);
        if is_last {
          break;
        }
      }

      if let Some(token) = next_token {
        if token.get_catcode() == Catcode::BEGIN {
          stomach::digest(token)?;
          list.extend(stomach::digest_next_body(None)?);
          list.pop();
        } else {
          list = stomach::invoke_token(&token)?;
        }
      }

      list.retain(|tbox| ! matches!(tbox.data(), DigestedData::Comment(_)));
      let mode = Some(if ismath { TexMode::Math } else { TexMode::Text });
      List { boxes:list,  mode, ..List::default() }
    },
    reversion => sub[args,_inner,_extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(args).revert(), T_END!())) }
  );

  // A variation: Digest until we encounter a given token!
  // DefParameterType!(DigestUntil, sub[_inner, _untils] {
  //     gullet::skip_spaces()?;
  //     Ok(Tokens!())
  //   },
  //   // TODO: To implement this natively, we need "untils" i.e.
  //   // "extra" passed into "predigest" as well.
  //   predigest => {
  //     todo!();
  //     //   let ismath = lookup_bool("IN_MATH");
  //     //   stomach::digest_next_body(Some(until))?
  //   //   my @list   = $state->getStomach->digestNextBody($until);
  //   //   @list = grep { ref $_ ne 'LaTeXML::Core::Comment' } @list;
  //   //   List(@list, mode => ($ismath ? 'math' : 'text'));
  //     ()
  //   },
  //   reversion => sub[args,_inner,_extra] {
  //     Ok(Tokens!(T_BEGIN!(), Tokens::new(args).revert(), T_END!())) }
  // );

  // Reads until the current group has ended.
  // This is useful for environment-like constructs,
  // particularly alignments (which may or may not be actual environments),
  // but which need special treatment of some of their content
  // as the expansion is carried out.
  DefParameterType!(DigestedBody, sub[__inner, _extra] {
      Ok(Tokens!()) // all done in predigestion
    },
    predigest => {
      let ismath   = lookup_bool("IN_MATH");
      let mut list     = stomach::digest_next_body(None)?;
      // In most (all?) cases, we're really looking for a single Whatsit here...
      list.retain(|tbox| !tbox.is_comment());
      let mut digested = List::new(list);
      digested.mode = if ismath { Some(TexMode::Math) } else { Some(TexMode::Text) };
      digested
    }
  );

  // In addition to the standard TeX Dimension, there are various LaTeX constructs
  // (particularly, the LaTeX picture environment, and the various pstricks packages)
  // that take a different sort of length.  They differ in two ways.
  //   (1) They do not accept a comma as decimal separator
  //      (they generally use it to separate coordinates), and
  //   (2) They accept a plain float which is scaled against a Dimension register.
  //      Actually, there are two subcases:
  //     (a) picture accepts a float, which is scaled against \unitlength
  //     (b) pstricks accepts a float, and optionally a unit,
  //        If the unit is omitted, it is relative to \psxunit or \psyunit.
  // How to capture these ?
  //DefParameterType!(Length, sub {
  ////   my($gullet,$unit)=@_;

  // CommaList expects something like {balancedstuff,...}
  DefParameterType!(CommaList, sub[__inner, _extra] {
      // my ($gullet, $type) = @_;
      // my $typedef = $type &&
      //       LaTeXML::Package::parseParameters(ToString($type), "CommaList")->[0];
      // my @items = ();
      // if ($gullet->ifNext(T_BEGIN)) {
      //   $gullet->readToken;
      //   my @tokens = ();
      //   my $comma  = T_OTHER(',');
      //   while (my $token = $gullet->readToken) {
      //     my $cc = $token->getCatcode;
      //     if ($cc == CC_END) {
      //       push(@items, Tokens(@tokens));
      //       last; }
      //     elsif ($token->equals($comma)) {
      //       push(@items, Tokens(@tokens)); @tokens = (); }
      //     elsif ($cc == CC_BEGIN) {
      //       push(@tokens, $token, $gullet->readBalanced->unlist, T_END); }
      //     else {
      //       push(@tokens, $token); } }
      //   if ($typedef) {
      //     @items = map { [$typedef->reparseArgument($gullet, $_)]->[0] } @items; } }
      // else {
      //   # If no brace, just read one item or token, but still make Array!
      //   push(@items, ($typedef ? $typedef->readArguments($gullet, "CommaList")
      //       : ($gullet->readToken))); }
      // LaTeXML::Core::Array->new(open => T_BEGIN, close => T_END, type => $typedef,
      //   values => [@items]); });
      // Stub: CommaList parameter type not yet ported
      Tokens!()
  });

  // Support for Key / Value arguments.
  // The very basic form is
  //   RequiredKeyVals: $keyset
  //   OptionalKeyVals: $keyset
  // to parse Key-Value pairs from a given keyset (see the 'keyval' package
  // documentation for more information). These types of KeyVal
  // parameters will return a LaTeXML::Core::KeyVals object, which can then be
  // used to access the values of the individual items.
  // The difference between the two forms is that RequiredKeyVals expects a set of
  // key-value pairs wrapped in T_BEGIN T_END, where as OptionalKeyVals optionally
  // expects a set of KeyValue pairs wrapped in T_OTHER('[') T_OTHER(']')
  //
  // Several extension of the keyval package exist, the most common one we support
  // is the xkeyval package. This introduces further variations on the keyval
  // arguments parsing, in particular it allows to read keys from more than one
  // keyset at once. These can be specified by giving comma-seperated values in
  // the keyset argument. By default, a key will only be set in the **first**
  // keyset it occurs in. By using
  //   RequiredKeyVals+: $keysets
  //   OptionalKeyVals+: $keysets
  // the key will be set in all keysets instead.
  //
  // All keys to be parsed with these arguments should be declared using
  // DefKeyVal in LaTeXML::Package. By default, an error is thrown if an unknown
  // key is encountered. To surpress this behaviour, and instead store all
  // undefined keys, use
  //   RequiredKeyVals*: $keysets
  //   OptionalKeyVals*: $keysets
  // instead. The '*' and '+' modifiers can be combined by using:
  //   RequiredKeyVals*+: $keysets
  //   OptionalKeyVals*+: $keysets
  //
  // Furthermore, the xkeyval package supports giving prefixes to keys,
  //   RequiredKeyVals[*][+]: $prefix|$keysets
  //   OptionalKeyVals[*][+]: $prefix|$keysets
  //
  // Finally, it is possible to specify specific keys to skip when digesting the
  // object. This can be achieved using comma-seperated key values in
  //   RequiredKeyVals[*][+]: $prefix|$keysets|$skip
  //   OptionalKeyVals[*][+]: $prefix|$keysets|$skip

  pub fn required_key_vals(
    star: bool,
    plus: bool,
    _inner: Option<&Parameters>,
    extra: &[Tokens],
  ) -> Result<KeyVals> {
    if gullet::if_next(T_BEGIN!())? {
      let mut extra_iter = extra.iter();
      // subtle!!! The first extra is the prefix, according to the Perl use.
      let prefix = extra_iter.next().map(ToString::to_string);
      // TODO: is the last extra field actually a "skip" ? Example?
      let keysets = extra_iter.map(ToString::to_string).collect();
      keyvals_aux(Some(T_END!()), KVSpec {
        star,
        plus,
        prefix,
        keysets,
        ..KVSpec::default()
      })
    } else {
      Error!("Expected", "{", "Missing keyval arguments");
      Ok(KeyVals::default())
    }
  }

  DefParameterType!(RequiredKeyVals, sub[inner, extra] {
    required_key_vals(false, false, inner, extra)
  },
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
  });
  DefParameterType!(RequiredKeyValsStar, sub[inner, extra] {
    required_key_vals(true, false, inner, extra)
  },
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
  });
  DefParameterType!(RequiredKeyValsPlus, sub[inner, extra] {
    required_key_vals(false, true, inner, extra)
  },
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
  });
  DefParameterType!(RequiredKeyValsStarPlus, sub[inner, extra] {
    required_key_vals(true, true, inner, extra)
  }, reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
  });

  pub fn optional_key_vals(
    star: bool,
    plus: bool,
    _inner: Option<&Parameters>,
    extra: &[Tokens],
  ) -> Result<Option<KeyVals>> {
    if gullet::if_next(T_OTHER!("["))? {
      let mut extra_iter = extra.iter();
      // subtle!!! The first extra is the prefix, according to the Perl use.
      let prefix = extra_iter.next().map(ToString::to_string);
      // TODO: is the last extra field actually a "skip" ? Example?
      let keysets = extra_iter.map(ToString::to_string).collect();
      let kvs: KeyVals = keyvals_aux(Some(T_OTHER!("]")), KVSpec {
        star,
        plus,
        prefix,
        keysets,
        ..KVSpec::default()
      })?;
      Ok(Some(kvs))
    } else {
      Ok(None)
    }
  }

  DefParameterType!(OptionalKeyVals, sub[inner, extra] {
    optional_key_vals(false, false, inner, extra)
  }, optional=>true,
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_OTHER!("["), Tokens::new(arg).revert(), T_OTHER!("]")))
  });
  DefParameterType!(OptionalKeyValsStar, sub[inner, extra] {
    optional_key_vals(true, false, inner, extra)
  }, optional=>true,
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_OTHER!("["), Tokens::new(arg).revert(), T_OTHER!("]")))
  });
  DefParameterType!(OptionalKeyValsPlus, sub[inner, extra] {
    optional_key_vals(false, true, inner, extra)
  }, optional=>true,
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_OTHER!("["), Tokens::new(arg).revert(), T_OTHER!("]")))
  });
  DefParameterType!(OptionalKeyValsPlusStar, sub[inner, extra] {
    optional_key_vals(true, true, inner, extra)
  }, optional=>true,
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_OTHER!("["), Tokens::new(arg).revert(), T_OTHER!("]")))
  });

  // Not sure that this is the most elegant solution, but...
  // What I'd really like are some sort of parameter modifiers, mathstyle, font... until...?
  DefParameterType!(DisplayStyle, sub[_inner, _extra] { gullet::read_arg(ExpansionLevel::Off) },
  before_digest => {
    bgroup();
    MergeFont!(mathstyle => "display");
  },
  after_digest => { egroup()?; },
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
  });
  DefParameterType!(TextStyle, sub[_inner, _extra] { gullet::read_arg(ExpansionLevel::Off) },
  before_digest => {
    bgroup();
    MergeFont!(mathstyle => "text");
  },
  after_digest => { egroup()?; },
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
  });
  DefParameterType!(ScriptStyle, sub[_inner, _extra] { gullet::read_arg(ExpansionLevel::Off) },
  before_digest => {
    bgroup();
    MergeFont!(mathstyle => "script");
  },
  after_digest => { egroup()?; },
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
  });
  // # Perverse naming convention: not script style, but in the style of a script relative to
  // current.
  DefParameterType!(InScriptStyle, sub[_inner, _extra] {
    gullet::read_arg(ExpansionLevel::Off) },
  before_digest => {
    bgroup();
    MergeFont!(scripted => true);
  },
  after_digest => { egroup()?; },
  reversion => sub[arg, _inner, _extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(arg).revert(), T_END!()))
  });
  // # NOTE: the various parameter features don't combine easily!!
  // # I need a ScriptStyleUntil for \root!!!
  // # I also need to redo fractions using these new types....
  DefParameterType!(OptionalInScriptStyle, sub[_inner, _extra] {
    gullet::read_optional(None)
  },
  before_digest => {
    bgroup();
    MergeFont!(scripted => true);
  },
  after_digest => {
    egroup()?;
  },
  optional => true,
  reversion => sub[arg,_inner,_extra] {
    if arg.is_empty() { Ok(Tokens!()) }
    else {
      let mut tks = vec![T_OTHER!("[")];
      tks.extend(arg.into_iter().map(|t| t.revert()));
      tks.push(T_OTHER!("]"));
      Ok(Tokens::new(tks))
    }
  });
  DefParameterType!(InFractionStyle, sub[_inner, _extra] {
    gullet::read_arg(ExpansionLevel::Off)
  },
  before_digest => {
    bgroup();
    MergeFont!(fraction => true);
  },
  after_digest => {
    egroup()?;
  },
  reversion => sub[arg,_inner,_extra] {
    let mut reverted = vec![T_BEGIN!()];
    reverted.extend(arg.into_iter().map(Token::revert));
    reverted.push(T_END!());
    Ok(Tokens::new(reverted))
  });
});

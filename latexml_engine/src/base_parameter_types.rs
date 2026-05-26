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
    gullet::skip_one_space(false)?;
  }, novalue => true);

  // Read the next token
  DefParameterType!(Token, sub[_inner, _extra] {
    // Perl Base_ParameterTypes.pool.ltxml L91:
    //   DefParameterType('Token', sub { $_[0]->readToken; });
    // No error raised on EOF — silently returns undef. Our prior impl
    // raised an error, which manifested as a cascade after upstream
    // recovery (e.g. \^ with no argument in math mode). EOF fallback
    // returns `\relax` (a no-op token) so downstream `try_to_token`
    // succeeds — using empty Tokens here would cascade
    // "Error:expected:argument: try_to_token: empty Tokens" at the
    // primitive's arg-coerce step (witness: 0910.2125 sub/superscript
    // cascade where 13 errors match Perl + 1 extra Rust empty-Tokens).
    match gullet::read_token()? {
      Some(t) => Ok(ArgWrap::Token(t)),
      None => Ok(ArgWrap::Token(T_CS!("\\relax"))),
    }
  });

  // Read the next token, after expanding any expandable ones.
  DefParameterType!(XToken, sub[_inner, _extra] {
    // Perl Base_ParameterTypes.pool.ltxml L94:
    //   DefParameterType('XToken', sub { $_[0]->readXToken; });
    // Same `\relax`-sentinel passthrough on EOF — see Token comment above.
    if let Some(t) = gullet::read_x_token(None, false, None)? {
      Ok(ArgWrap::Token(t))
    } else {
      Ok(ArgWrap::Token(T_CS!("\\relax")))
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
  //
  // Helper: read a float that may be wrapped in braces (e.g. `{36.5}`).
  // Some authors brace pair coordinates to disambiguate negative-number
  // tokenization or to keep \multiput / \put pair args together.
  // Witness: hep-th/9610147 — `\multiput(-89,{36.5})(-6,-1){6}{…}`.
  DefParameterType!(Pair, sub[_inner, _extra] {
    use latexml_core::common::pair::Pair;
    use latexml_core::token::Catcode;
    fn read_pair_float() -> Result<latexml_core::common::float::Float> {
      let _ = gullet::skip_spaces();
      // If next is BEGIN brace, consume to matching END brace and read float inside.
      if let Some(tok) = gullet::read_token()? {
        if tok.get_catcode() == Catcode::BEGIN {
          let _ = gullet::skip_spaces();
          let f = gullet::read_float()?;
          let _ = gullet::skip_spaces();
          // Consume matching close brace
          if let Some(close) = gullet::read_token()? {
            if close.get_catcode() != Catcode::END {
              gullet::unread_one(close);
            }
          }
          return Ok(f);
        }
        gullet::unread_one(tok);
      }
      gullet::read_float()
    }
    let _ = gullet::skip_spaces();
    if gullet::if_next(T_OTHER!("("))? {
      gullet::read_token()?; // consume (
      let _ = gullet::skip_spaces();
      let x = read_pair_float()?;
      // Perl latex_constructs.pool.ltxml:ReadPair L4910-4912 uses
      //   $gullet->skipSpaces; $gullet->readUntil(T_OTHER(',')); $gullet->skipSpaces;
      //   my $y = ...;
      //   $gullet->skipSpaces; $gullet->readUntil(T_OTHER(')')); $gullet->skipSpaces;
      // — `readUntil` is tolerant of extra junk between the float and the
      // separator. Witness: physics/9709007 line 1594
      //   \multiput(3.2,3,8)(.3,0){2}{\circle*{.1}}
      // The user typoed `3,8` for `3.8`; Perl reads x=3.2, swallows nothing
      // up to the comma, reads y=3, then `readUntil(')')` consumes the
      // extra `,8` silently. The earlier Rust port read one token after y
      // and only consumed it if literally `)` — bailing the second pair.
      let _ = gullet::skip_spaces();
      let _ = gullet::read_until(&Tokens!(T_OTHER!(",")));
      let _ = gullet::skip_spaces();
      let y = read_pair_float()?;
      let _ = gullet::skip_spaces();
      let _ = gullet::read_until(&Tokens!(T_OTHER!(")")));
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
    // XUntil requires a delimiter token in its Extras slot. A malformed
    // parameter spec (bare `XUntil` with no `:delim` attached) would panic
    // with the prior .expect(); degrade to reading nothing so the caller
    // sees an empty Tokens result instead of an abort.
    let Some(until_tks) = untils.first() else {
      Warn!("expected", "token", "XUntil parameter missing delimiter token");
      return Ok(ArgWrap::Tokens(Tokens!()));
    };
    let until : Token = until_tks.into();
    let mut tokens : Vec<Token> = Vec::new();
    while let Some(token) = gullet::read_x_token(Some(false), false, None)? {
      if token == until {
        break;
      } else if token.get_catcode() == Catcode::BEGIN {
        tokens.push(token);
        tokens.extend(gullet::read_balanced(ExpansionLevel::Off,false,false)?.unlist());
        tokens.push(T_END!());
      } else {
        // After read_x_token, an expandable macro should already have been
        // expanded — what remains here is non-expandable (Primitive,
        // Constructor, Conditional, Register, MathPrimitive) or a Let to
        // a literal token. Eagerly calling `read_arguments()` on those
        // was a bug: e.g. for `\hspace*{-4mm} $^*\,$` inside an XUntil
        // body, `\hspace`'s primitive `Dimension` reader would
        // over-consume past `}`, eating the following `$` token and
        // causing the math frame to leak (witness: astro-ph9903386
        // \institute math-mode leak). Only re-Invocation-emit for
        // GENUINE Expandable defs that for some reason escaped
        // `read_x_token`'s expansion (e.g. \protected macros). Inspect
        // the raw `Stored` variant rather than `lookup_definition_stored`
        // (which synthesizes a no-op Expandable around `Stored::Token`
        // entries like `\sb`-Let-to-T_SUB; rebuilding such a synthesized
        // invocation calls `build_invocation`, which rejects Token
        // meanings and erroneously fires "Can't invoke; it is undefined"
        // — witness: math0610119 `\sb` inside amsppt
        // `\@bibfield XUntil:\@end@bibfield`).
        let is_real_expandable =
          matches!(state::lookup_meaning(&token), Some(Stored::Expandable(_)));
        // Perl XUntil L144-146: ALWAYS calls `readArguments` for any defined
        // token, wrapping in `Invocation`. Without this, a Constructor
        // body like `\href{u}{t}` → `\lx@hyper@url@\href{}{}{u}{t}` lets
        // XUntil's outer `read_x_token` re-expand the `\href` token (which
        // the Constructor is supposed to consume as Undigested arg #1),
        // producing infinite recursion. Witness: 1902.01143 elsarticle
        // `\begin{keyword} ... \href{...}{...}` hangs at 100M token limit.
        // Targeted to Constructors only — Primitives have side-effecting
        // arg readers (e.g. `\hspace`'s Dimension reader over-consumed
        // past `}` in astro-ph9903386 — the recorded regression that
        // motivated the original "bare-token push" else-branch).
        let is_constructor =
          matches!(state::lookup_meaning(&token), Some(Stored::Constructor(_)));
        // Definitional primitives (\def/\edef/\gdef/\xdef/\let/\futurelet) consume
        // their target token from the input AT EXECUTION TIME — but XUntil's outer
        // `read_x_token` would expand the target away if it's `\let`-bound to
        // something expandable (e.g. `\@date` Let'd to `\@empty`). Witness:
        // 0805.1712 elsart `\date{X}` inside `\begin{keyword}` body, where
        // `\date` expands to `\def\@date{X}\@add@frontmatter{ltx:date}[...]{X}`,
        // and inside the keyword's XUntil read, `\@date` (Let'd to `\@empty`)
        // gets consumed by `read_x_token` BEFORE `\def` ever runs, leaving the
        // captured tokens malformed (`\def {X} ...` with no def-target).
        // Re-Invoke these primitives to read their parameters from the gullet
        // here, matching Perl's XUntil L144-146 behavior. Targeted to the def
        // family to avoid the `\hspace`/dimension-reader over-read issue
        // recorded in the comment above (astro-ph9903386 leak).
        let is_def_family = token.with_cs_name(|cs| {
          matches!(cs, "\\def" | "\\gdef"
            | "\\futurelet" | "\\global" | "\\protected" | "\\long" | "\\outer")
        });
        // \edef, \xdef AND \let need their target/body captured RAW — not
        // through read_arguments→Invocation. Reasons:
        //   - \edef/\xdef: DefExpanded body parameter expands eagerly
        //     (`\itdefault` → `it`); Invocation revert drops the target
        //     Token and the {…}-braces, leaving `\edef i t \selectfont`
        //     stream that re-reads as malformed at EOF.
        //     Driver: 2403.14274 IEEEconf `\itshape …`.
        //   - \let: DefToken/Token reversion in some test paths emits the
        //     `\let` token alone without the cs1/cs2 args, then the body
        //     re-reads `\let \let \let …` recursively as `\let {…}` with
        //     the next `\let`'s target = `{` — triggering 100-deep
        //     recursion through `\lx@acronym`. Driver: 2103.11356 elsart
        //     `\begin{keyword}\ac{CNNs} …`.
        // Capture all three RAW: `<token> <skipped-spaces> <Token>
        // <until-brace> {<balanced-raw>}`. Preserves original input
        // exactly so re-emission re-reads cleanly.
        let is_def_raw_capture =
          token.with_cs_name(|cs| matches!(cs, "\\edef" | "\\xdef" | "\\let"));
        if is_def_raw_capture {
          tokens.push(token);
          gullet::skip_spaces()?;
          if let Some(target) = gullet::read_token()? {
            tokens.push(target);
          }
          // \let has no UntilBrace + body — just `<target> <value>`.
          // \edef/\xdef have UntilBrace + balanced-body.
          let is_let = token.with_cs_name(|cs| cs == "\\let");
          if is_let {
            // Read second token (the value to alias to).
            // Optional `=` and one space are also valid in real TeX
            // — DefToken handles those, but Token doesn't. Peek for `=`.
            gullet::skip_spaces()?;
            if let Some(value) = gullet::read_token()? {
              tokens.push(value);
            }
          } else {
            if let Some(prebrace) = gullet::read_until_brace()? {
              tokens.extend(prebrace.unlist());
            }
            tokens.push(T_BEGIN!());
            tokens.extend(gullet::read_balanced(ExpansionLevel::Off, false, true)?.unlist());
            tokens.push(T_END!());
          }
        } else if is_real_expandable || is_def_family || is_constructor {
          if let Some(defn) = lookup_definition_stored(&token)? {
            let args = defn.read_arguments()?;
            tokens.extend(Invocation!(token, args).unlist());
          } else {
            tokens.push(token);
          }
        } else {
          tokens.push(token);
        }
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
    tks.extend(arg.into_iter().map(Token::revert));
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
    tks.extend(arg.into_iter().map(Token::revert));
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
    // Concatenate the extras' string forms directly — collect::<String>
    // handles Iterator<Item=String> by appending each in turn. The old
    // `.collect::<Vec<String>>().join("")` allocated an intermediate Vec.
    let extra_string: String = extra.iter().map(ToString::to_string).collect();
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
      read_tokens.extend(arg.into_iter().map(Token::revert));
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
      reverted.extend(arg.into_iter().map(Token::revert));
      reverted.push(T_END!());
      Ok(Tokens::new(reverted))
    }
  );

  // Read verbatim, as if with LaTeX's \@sanitize; useful for \index (maybe others?)
  // Perl: latex_constructs.pool.ltxml L4433-4451
  DefParameterType!(SanitizedVerbatim, sub[_inner, _extra] {
      gullet::read_until(&Tokens!(T_BEGIN!()))?;
      // Deactivate the backslash to avoid activating command sequences.
      // Chars switched to CC_OTHER by \@sanitize: ' ', '\\', '$', '&', '#',
      // '^', '_', '%', '~'. Some are already in state's SPECIALS, so only
      // adding the rest:
      begin_semiverbatim(Some(&[' ', '\\', '%']));
      let arg = gullet::read_balanced(ExpansionLevel::Off, false, false)?;
      end_semiverbatim()?;
      // Now that we have the semiverbatim tokens, retokenize.
      // This may seem like wasted work, but it avoids very unfortunate error
      // propagation in cases where the \index argument was malformed for one
      // reason or another. The strangeness comes from the original TeX
      // workflow requiring multiple conversion calls, alongside a call to the
      // `makeidx` binary, which we don't do in latexml. This parameter type
      // emulates one important aspect implied by those steps.
      Ok(mouth::tokenize_internal(&arg.untex()))
    },
    reversion => sub[arg, _inner, _extra] {
      let mut reverted = vec![T_BEGIN!()];
      reverted.extend(arg.into_iter().map(Token::revert));
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
      reverted.extend(arg.into_iter().map(Token::revert));
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
      read_tokens.extend(arg.into_iter().map(Token::revert));
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
        read_tokens.extend(arg.into_iter().map(Token::revert));
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

  // Strict translation of Perl Base_ParameterTypes.pool.ltxml L309-322:
  //   sub {
  //     my ($gullet)   = @_;
  //     my $arg_string = ToString($gullet->readArg);
  //     my @dirs       = ();
  //     for my $dir (split(/,|\\par|\n+/, $arg_string)) {
  //       $dir =~ s/^\s+//; $dir =~ s/\s+$//;
  //       next unless $dir;
  //       while ($dir =~ s/^\s*\{([^\}]*)\}//) {
  //         push @dirs, $1 if $1; }
  //       push @dirs, $dir if $dir; }
  //     LaTeXML::Core::Array->new(...);  };
  //
  // Accepts both `\graphicspath{dir}` and `\graphicspath{{dir1}{dir2}}`.
  // Rust output: emit `{dir1}{dir2}...` so the `\graphicspath`
  // constructor can split on `}` to recover individual entries.
  // Semiverbatim catcodes prevent `_`, `/`, `#` in path names from
  // tokenizing as SUB/ACTIVE.
  DefParameterType!(DirectoryList, sub[_inner, _extra] {
    use once_cell::sync::Lazy;
    use regex::Regex;
    static SPLIT_RE: Lazy<Regex> =
      Lazy::new(|| Regex::new(r",|\\par|\n+").unwrap());
    static TRIM_LEAD: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s+").unwrap());
    static TRIM_TAIL: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+$").unwrap());
    static STRIP_BRACE: Lazy<Regex> =
      Lazy::new(|| Regex::new(r"^\s*\{([^\}]*)\}").unwrap());

    let arg = gullet::read_arg(ExpansionLevel::Off)?;
    let arg_string = arg.untex();
    let mut dirs: Vec<String> = Vec::new();
    for entry in SPLIT_RE.split(&arg_string) {
      let mut dir = TRIM_LEAD.replace(entry, "").into_owned();
      dir = TRIM_TAIL.replace(&dir, "").into_owned();
      if dir.is_empty() {
        continue;
      }
      // Iteratively strip `^\s*\{([^\}]*)\}` sub-groups, pushing the
      // captured content (Perl `push @dirs, $1 if $1`).
      while let Some(caps) = STRIP_BRACE.captures(&dir) {
        let whole = caps.get(0).unwrap().as_str().to_string();
        let inner = caps.get(1).unwrap().as_str().to_string();
        if !inner.is_empty() {
          dirs.push(inner);
        }
        dir = dir[whole.len()..].to_string();
      }
      if !dir.is_empty() {
        dirs.push(dir);
      }
    }
    // Emit `{dir1}{dir2}...` as Tokens (consumed by \graphicspath).
    let mut collected: Vec<Token> = Vec::new();
    for d in dirs {
      collected.push(T_BEGIN!());
      for c in d.chars() {
        collected.push(T_OTHER!(&c.to_string()));
      }
      collected.push(T_END!());
    }
    Tokens::new(collected)
  },
  // Treat as semiverbatim for tokenization-sensitive chars: _ / # & $ ~ ^ % @.
  semiverbatim => Some(Vec::new()));

  // This reads a Box as needed by \raise, \lower, \moveleft, \moveright.
  //
  // Strict translation of Perl Base_ParameterTypes.pool.ltxml L327-337:
  //   DefParameterType('MoveableBox', sub {
  //       my ($gullet) = @_;
  //       $gullet->skipSpaces;
  //       my ($box, @stuff) = $STATE->getStomach->invokeToken($gullet->readXToken);
  //       if (!$box) {
  //         Error('expected', '<box>', $gullet,
  //               "A <box> was supposed to be here", "Got " . Stringify($box));
  //         $box = Box(); }                # ← empty-box fallback
  //   ###  && $box->isa('LaTeXML::Core::Whatsit')
  //   ###  && ($box->getDefinition->getCSName =~ /^(\\hbox|\\vbox||\\vtop)$/);
  //       $box; });
  //
  // The CS-name check (hbox/vbox/vtop) is COMMENTED OUT in Perl —
  // any digested first-element is accepted. On the empty-result branch
  // Perl substitutes `Box()` (an empty box) so the caller of
  // `\raise <Dim> <MoveableBox>` always receives a defined box value.
  //
  // Previously Rust (a) ran the commented-out CS-name check as live
  // code (mis-rejecting `\begin{minipage}`, picture overlays, etc.)
  // and (b) returned None on the empty branch (caller sees no value
  // where Perl sees an empty Box). Caught on 1812.04267 (xy-pic
  // content wrapped in \begin{minipage}: 3× spurious "expected:<box>"
  // where Perl is clean).
  DefParameterType!(MoveableBox, sub[_inner, _extra] {
    // TeX's `<box>` argument scanner accepts `<filler>` prefix
    // (TeXbook p. 270, TeX-the-program §403 `scan_filler`): spaces
    // AND `\relax`-meaning tokens. The LaTeX kernel's robust-command
    // convention (`\DeclareRobustCommand`) expands `\rlap{x}` to
    // `\protect\rlap {x}`, where `\protect` is `\let` to `\relax` at
    // typesetting time. A one-token-only read here returns
    // `\protect`/`\relax`, predigest invokes it as a no-op, leaving
    // the actual `\rlap ` (with space) and its argument unread — and
    // `expected:<box>` fires spuriously.
    //
    // Skip leading `\relax`-equivalents (resolving `\let` chains) so
    // the reader lands on the box-producing token. Perl
    // `Base_ParameterTypes.pool.ltxml:327-337` doesn't yet do this —
    // anticipates the same upcoming Perl PR class as the `mode =>`
    // machinery work.
    //
    // Driver: astro-ph0004263, lsim.tex repro:
    //   `\def\lsim{\hbox{\raise.35ex\rlap{$<$}\lower.6ex\hbox{$\sim$}\ }}`
    // — `\rlap{$<$}` after `\raise.35ex` previously hit the
    // `expected:<box>` cascade. pdflatex handles this cleanly.
    gullet::skip_spaces()?;
    let mut result: Tokens = Tokens!();
    loop {
      let Some(xtoken) = gullet::read_x_token(None, false, None)? else {
        break;
      };
      // Treat as filler iff the token's meaning is x-equal to `\relax`
      // (covers `\let\protect=\relax`, which `let_i` stores by copying
      // `\relax`'s Primitive into `\protect`'s meaning slot — so
      // `lookup_meaning(\protect)` is the Primitive, not a Stored::Token).
      let is_filler = xtoken.get_catcode() == Catcode::CS
        && state::x_equals(&xtoken, &T_CS!("\\relax"));
      if is_filler {
        gullet::skip_spaces()?;
        continue;
      }
      result = Tokens!(xtoken);
      break;
    }
    result
  }, predigest => sub[arg] {
    let token = arg.unlist().remove(0);
    // R35.A: hard depth cap on MoveableBox::predigest recursion.
    // Witness math0102089 (plain-TeX `\picture`/`\put` inside
    // `$$\displaylines{...}$$`) recurses through this closure
    // unboundedly via mutual recursion with predigest_box_contents
    // (see backtrace via `LATEXML_DEBUG_MEMBUDGET=1`). The wp5/canvas3
    // corpus shows real documents stay well below 50 levels of
    // \raise/\lower nesting; 1000 is the safety cap.
    std::thread_local! {
      static MOVEABLE_BOX_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    }
    let depth = MOVEABLE_BOX_DEPTH.with(|d| { let v = d.get() + 1; d.set(v); v });
    if depth > 1000 {
      MOVEABLE_BOX_DEPTH.with(|d| d.set(0));
      let message = s!("Recursion depth exceeded in MoveableBox::predigest \
        (limit 1000). Likely runaway in `\\raise`/`\\lower`/`\\move`/\
        picture-mode `\\hbox\\bgroup…\\egroup` chain.");
      fatal!(Timeout, MemoryBudget, message);
    }
    let result = stomach::invoke_token(&token);
    MOVEABLE_BOX_DEPTH.with(|d| d.set(d.get() - 1));
    let mut stuff = result?;
    if !stuff.is_empty() {
      // Perl: `($box, @stuff) = invokeToken(...)` — first element is the box.
      Some(stuff.remove(0))
    } else {
      // Perl L332-334: report and substitute an empty Box() so callers
      // (\raise/\lower etc.) always receive a defined value.
      let message = s!("A <box> was supposed to be here.\nGot none.");
      Error!("expected","<box>", message);
      Some(latexml_core::digested::Digested::from(latexml_core::tbox::Tbox::default()))
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
  // Reads a balanced {}-arg via read_arg, reverts WITHOUT braces.
  //
  // INCOMPLETE vs Perl TeX_Math.pool.ltxml:709 — see docs/WISDOM.md #41
  // for the full enhancement plan. Current impl works for
  // \big/\Big/\bigg/\Bigg (math_common.rs:962-964) but \left/\lx@right
  // plus revsymb's \biglb family fall back to DefMacro workarounds.
  //
  // Gap has two dimensions:
  //   - Reader shape (3 branches missing): single-X-token read instead of read_arg,
  //     BEGIN-unwrap-and-re-read, `.`/undef → \lx@delimiterdot substitution.
  //   - Architectural `undigested=>1`: ArgWrap has no Digested variant and Parameter has no
  //     `undigested: bool` flag; closing this needs latexml_core changes. Required for
  //     \left\delimiter<num>.
  //
  // Reader-only partial port closes ZERO DP audit entries because the
  // call-site migrations need BOTH dimensions — don't commit half.
  DefParameterType!(TeXDelimiter, sub[_inner, _extra] {
    gullet::skip_filler()?;
    // Peek at the next token. If it's END (catcode 2) or EOF,
    // do NOT consume — leave it for the surrounding scope to handle.
    // Otherwise the `}` closing the enclosing group would be eaten
    // here, producing an unbalanced math env. Witness arXiv:1207.4709
    // (paper invokes `\smalltwomatrix{B}{x}{}{t}\big|...` with only 4
    // brace-groups — `\big` is read as the 5th arg; the body then
    // expands to `{... {\big} ...}` and our `\big` consumes the `}`).
    // Perl's TeX_Math.pool.ltxml:709 uses readXToken (peek-like) and
    // falls back to `\lx@delimiterdot` on undef/`.` — matching its
    // tolerance here.
    let peeked = gullet::read_token()?;
    match peeked {
      None => Ok(Tokens!(T_CS!("\\lx@delimiterdot"))),
      Some(tok) if tok.get_catcode() == latexml_core::token::Catcode::END => {
        gullet::unread_one(tok);
        Ok(Tokens!(T_CS!("\\lx@delimiterdot")))
      },
      Some(tok) if tok.get_catcode() == latexml_core::token::Catcode::OTHER
                   && tok.text == latexml_core::common::arena::pin(".") =>
      {
        Ok(Tokens!(T_CS!("\\lx@delimiterdot")))
      },
      Some(tok) => {
        gullet::unread_one(tok);
        gullet::read_arg(ExpansionLevel::Partial)
      },
    }
  },
  digested_reversion => sub[arg] {
    // Revert without adding braces (unlike {} parameter)
    let toks = arg.revert()?;
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
      let ismath = state::lookup_bool_sym(pin!("IN_MATH"));
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
  // Perl: Base_ParameterTypes.pool.ltxml L384-394
  DefParameterType!(DigestUntil, sub[_inner, _extra] {
      gullet::skip_spaces()?;
      Ok(Tokens!())
    },
    predigest => sub[_arg, extra] {
      let ismath = state::lookup_bool_sym(pin!("IN_MATH"));
      // Perl: ($until) = $until->unlist — first token of the extra Tokens.
      let until = extra.first()
        .and_then(|toks| toks.unlist_ref().first().cloned());
      let mut list = stomach::digest_next_body(until)?;
      list.retain(|tbox| !tbox.is_comment());
      let mut digested = List::new(list);
      digested.mode = if ismath { Some(TexMode::Math) } else { Some(TexMode::Text) };
      digested
    },
    reversion => sub[args, _inner, _extra] {
      Ok(Tokens!(T_BEGIN!(), Tokens::new(args).revert(), T_END!()))
    }
  );

  // Reads until the current group has ended.
  // This is useful for environment-like constructs,
  // particularly alignments (which may or may not be actual environments),
  // but which need special treatment of some of their content
  // as the expansion is carried out.
  DefParameterType!(DigestedBody, sub[__inner, _extra] {
      Ok(Tokens!()) // all done in predigestion
    },
    predigest => {
      let ismath   = state::lookup_bool_sym(pin!("IN_MATH"));
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

  // CommaList expects something like {item1,item2,...}; items may be
  // `{balanced}` groups or plain token runs. Perl returns a
  // `LaTeXML::Core::Array`; Rust has no Array type and follows the
  // `DirectoryList` convention — emit a token stream where each item
  // is wrapped in its own `{...}`.
  //
  // Perl supports a parameterised form `CommaList:Type` (e.g.
  // `CommaList:Number`) where each item is re-parsed through `Type`'s
  // own reader (`$typedef->reparseArgument`). When the parameter is
  // declared as `CommaList:Number`, `inner` carries the parsed
  // `Number` Parameter; we route each item through
  // `inner.reparse_argument(...)` and emit the reverted form so the
  // result is canonical-shape tokens of the typed value. Untyped
  // `CommaList` (no `inner`) keeps the original brace-delimit form.
  DefParameterType!(CommaList, sub[inner, _extra] {
    gullet::skip_spaces()?;
    // Phase 1: gather items as raw Tokens (one Tokens per comma-
    // separated piece). Done in a single pass to mirror the Perl
    // gullet-reader logic; the typed-reparse step happens after.
    let mut items: Vec<Tokens> = Vec::new();
    if gullet::if_next(T_BEGIN!())? {
      gullet::read_token()?; // consume outer `{`
      let mut current: Vec<Token> = Vec::new();
      let comma = T_OTHER!(",");
      while let Some(token) = gullet::read_token()? {
        let cc = token.get_catcode();
        if cc == Catcode::END {
          items.push(Tokens::new(std::mem::take(&mut current)));
          break;
        } else if token == comma {
          items.push(Tokens::new(std::mem::take(&mut current)));
        } else if cc == Catcode::BEGIN {
          // Nested `{balanced}` — preserve brace wrapping.
          current.push(token);
          let balanced = gullet::read_balanced(ExpansionLevel::Off, false, false)?;
          current.extend(balanced.unlist());
          current.push(T_END!());
        } else {
          current.push(token);
        }
      }
    } else if let Some(token) = gullet::read_token()? {
      // No outer brace — read a single token as the sole item.
      items.push(Tokens::new(vec![token]));
    }
    // Phase 2: if a typed form was declared, reparse each item via
    // the inner type's reader (`Perl reparseArgument` parity). The
    // reverted form is emitted so the downstream callsite sees
    // canonical tokens of the typed value.
    let mut collected: Vec<Token> = Vec::new();
    if let Some(inner_ps) = inner {
      for item in items {
        let reparsed = inner_ps.reparse_argument(ArgWrap::Tokens(item))?;
        collected.push(T_BEGIN!());
        for wrap in reparsed {
          collected.extend(wrap.revert()?.unlist());
        }
        collected.push(T_END!());
      }
    } else {
      for item in items {
        collected.push(T_BEGIN!());
        collected.extend(item.unlist());
        collected.push(T_END!());
      }
    }
    Tokens::new(collected)
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
  // keyset at once. These can be specified by giving comma-separated values in
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
  // object. This can be achieved using comma-separated key values in
  //   RequiredKeyVals[*][+]: $prefix|$keysets|$skip
  //   OptionalKeyVals[*][+]: $prefix|$keysets|$skip

  pub fn required_key_vals(
    star: bool,
    plus: bool,
    _inner: Option<&Parameters>,
    extra: &[Tokens],
  ) -> Result<KeyVals> {
    // Skip whitespace between this arg and the previous one. TeX-style
    // parameter matching `#1#2{...}` skips spaces before each `{`-delimited
    // group; our `RequiredKeyVals` parameter type checks `if_next(T_BEGIN)`
    // directly, so without an explicit skip it errors on user input like
    //   \newglossaryentry{RIS}\n{ name={...}, ... }
    // (\n + indentation between args[0] and args[1]). Driver: 2203.11854
    // R=1 → R=0 — Perl raw-loads glossaries.sty so `\newglossaryentry` is
    // a 2-arg `\newcommand` whose TeX matching handles this natively.
    gullet::skip_spaces()?;
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
      // Perl-faithful lowercase category — matches Base_ParameterTypes.pool.ltxml
      // `Error('expected', '{', $gullet, "Missing keyval arguments")`. The
      // engine convention is lowercase categories throughout.
      Error!("expected", "{", "Missing keyval arguments");
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
  // Perl: Base_ParameterTypes.pool.ltxml L586-593
  DefParameterType!(ScriptscriptStyle,
    sub[_inner, _extra] { gullet::read_arg(ExpansionLevel::Off) },
  before_digest => {
    bgroup();
    MergeFont!(mathstyle => "scriptscript");
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

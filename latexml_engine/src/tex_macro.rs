//! TeX Macro
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Macro Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  //
  //======================================================================
  // Basics
  //----------------------------------------------------------------------
  // \begingroup       c  starts a group that must be ended by \endgroup.
  // \endgroup         c  ends a group that was begun by \begingroup.
  // \relax            c  is a control sequence which typesets nothing.
  // \afterassignment  c  saves a token and inserts it after the next assignment.
  // \aftergroup       c  saves a token and inserts it after the current group is complete.
  DefPrimitive!("\\begingroup", {
    begingroup();
  });
  DefPrimitive!("\\endgroup", {
    endgroup()?;
  });
  // This makes \relax disappear completely after digestion
  // (which seems most TeX like).
  DefPrimitive!(T_CS!("\\relax"), None, {});
  // \protect is used in LaTeX for robust commands; acts like \relax in the base engine.
  Let!("\\protect", "\\relax");
  //## However, this keeps a box, so it can appear in UnTeX
  //## DefPrimitive('\relax',undef);
  //# But if you do that, you've got to watch out since it usually
  //## shouldn't be a box; See the isRelax code in handleScripts, below

  // NON-STANDARD: Internal token produced by Gullet in response to \dont_expand;
  // Acts like \relax, but isn't equal to it.
  DefPrimitive!(T_CS!("\\special_relax"), None, {});

  // \afterassignment saves ONE token (globally!) to execute after the next assignment
  DefPrimitive!("\\afterassignment Token", sub[(t)] {
    assign_value("afterAssignment", t, Some(Scope::Global));
  });
  // \aftergroup saves ALL tokens (from repeated calls) to be executed IN ORDER after the next
  // egroup or }
  DefPrimitive!("\\aftergroup Token", sub[(t)] { push_value("afterGroup", t) });

  //======================================================================
  // CSName
  //----------------------------------------------------------------------
  // \csname           c  forms a control sequence name from the characters making up a collection
  // of tokens. \endcsname        c  is used with \csname to make a control sequence name.
  DefParameterType!(CSName, reader => reader!( _inner, _extra, { read_cs_name() }));
  // Quiet version: used by \ifcsname — no errors for non-expandable CS tokens
  DefParameterType!(CSNameQuiet, reader => reader!( _inner, _extra, { read_cs_name_quiet() }));
  DefMacro!("\\csname CSName", sub[(token)] {
    if !has_meaning(&token) {
      let relax_meaning = lookup_meaning(&TOKEN_RELAX).unwrap();
      assign_meaning(&token, relax_meaning, None);
    }
    token
  });

  DefPrimitive!("\\endcsname", {
    if !state::lookup_bool("SUPPRESS_UNEXPECTED_ERRORS") {
      Error!("unexpected", "\\endcsname", "Extra \\endcsname");
    }
  });

  //======================================================================
  // Definition flags
  //----------------------------------------------------------------------
  // \global        c  is an assignment prefix which makes the assignment transcend its group.
  // \long          c  is a prefix for definitions which require multi-paragraph arguments.
  // \outer         c  is a prefix for a definition which restricts where the definition may be
  // used. \globaldefs    pi if positive, all assignments are global; if negative, \global is
  // ignored.

  DefPrimitive!("\\global",{ state::set_prefix("global"); }, is_prefix => true);
  DefPrimitive!("\\long",  { state::set_prefix("long");   }, is_prefix => true);
  DefPrimitive!("\\outer", { state::set_prefix("outer");  }, is_prefix => true);

  DefRegister!("\\globaldefs", Number!(0));
  //======================================================================
  // Definitions
  //----------------------------------------------------------------------
  // \def          c  defines a macro.
  // \edef         c  is similar to \def, except control sequences in the replacement
  //                  text are expanded when the definition is made.
  // \gdef         d  is equivalent to `\global\def'.
  // \xdef         d  is equivalent to `\global\edef'.
  DefPrimitive!("\\def SkipSpaces Token UntilBrace DefPlain",
    sub[(cs,params,body)] {
      do_def(false,cs,params,body)?;
    },
    locked => true);
  DefPrimitive!("\\gdef SkipSpaces Token UntilBrace DefPlain",
    sub[(cs,params,body)] {
      do_def(true,cs,params,body)?;
    },
    locked => true);
  DefPrimitive!("\\edef SkipSpaces Token UntilBrace DefExpanded",
    sub[(cs,params,body)] {
      do_def(false,cs,params,body)?;
    },
    locked => true);
  DefPrimitive!("\\xdef SkipSpaces Token UntilBrace DefExpanded",
    sub[(cs,params,body)] {
      do_def(true,cs,params,body)?;
    },
    locked => true);
  //======================================================================
  // Copying definitions
  //----------------------------------------------------------------------
  //  \let       c  gives a control sequence a token's current meaning.
  // \futurelet  c  `<cs> <token1> <token2>' is equivalent to `\let <cs> = <token2> <token1>
  // <token2>'.
  // `\let` — TeXbook Ch 24:
  //   `\let <cs> <equals> <one optional space> <token>`
  // where `<equals> = <optional spaces> | <optional spaces> =`,
  // and `<one optional space>` is exactly one space if present.
  //
  // Perl uses the parameter spec
  //   `\let SkipSpaces Token SkipSpaces SkipMatch:= Skip1Space Token`
  // which in our engine routed the optional-`=` consumption through
  // the generic Match parameter type's unread-on-no-match recovery.
  // Implementing the read sequence explicitly here is Perl-faithful
  // and TeX-faithful, and avoids a subtle interaction with the
  // generic recovery path: `<one optional space>` is always tried,
  // independent of whether `=` was consumed.
  DefPrimitive!("\\let SkipSpaces Token", sub[(token1)] {
    // <equals> = optional-spaces ['=']
    gullet::skip_spaces()?;
    if let Some(t) = gullet::read_token()? {
      let is_eq =
        t.get_catcode() == latexml_core::token::Catcode::OTHER && t.text == pin!("=");
      if !is_eq {
        gullet::unread_one(t);
      }
    }
    // <one optional space>
    gullet::skip_one_space(false)?;
    // <token>
    let token2 = gullet::read_token()?.unwrap_or_else(|| T_CS!("\\relax"));
    Let!(token1, token2);
  });
  DefPrimitive!("\\futurelet Token Token Token", sub[(cs, token1, token2)] {
    // NOT expandable, but puts tokens back
    gullet::unread(Tokens!(token1,token2));
    Let!(cs, token2);
  });
  //======================================================================
  // Expansion control
  //----------------------------------------------------------------------
  // \expandafter      c  `<token1><token2>' is equivalent to `<token1> expansion of <token2>'.
  // \noexpand         c  prevents the expansion of the following token.
  DefMacro!("\\expandafter Token Token", sub[(tok, xtok)] {
    let mut xtok = xtok;
    let mut skipped : Vec<Token> = vec![tok];
    while xtok.defined_as(&TOKEN_EXPANDAFTER) {
      if let Some(ntok) = gullet::read_token()? {
        skipped.push(ntok);
        if let Some(nxtok) = gullet::read_token()? {
          xtok = nxtok;
        } else {
          Error!("expected","expandafter", "\\expandafter wrongly used without 2 arguments.");
        }
      } else {
        Error!("expected", "expandafter", "\\expandafter wrongly used without 2 arguments.");
      }
    }
    match lookup_expandable(&xtok, None)? { Some(defn) => {
      state::local_current_token(xtok);
      let invoked = defn.invoke(true)?;
      if !invoked.is_empty() {
        skipped.extend(invoked.unlist()); // Expand `xtok` ONCE ONLY!
      }
      state::expire_current_token();
    } _ => if !has_meaning(&xtok) {
      // Undefined token is an error, as expansion is expected.
      // BUT The unknown token is NOT consumed, (see TeX B book, item 367)
      // since probably in a real TeX run it would have been defined.
      state::generate_error_stub(&xtok)?;
      skipped.push(xtok);
    } else {
      skipped.push(xtok);
    }};
    Ok(Tokens::new(skipped))
  });
  // If next token is expandable, prefix it with the internal marker \dont_expand
  // That token is never defined, explicitly handled in Gullet & should never escape the Gullet.
  //
  // Mirrors Perl `TeX_Macro.pool.ltxml:228-235`: uses `isDontExpandable`
  // (not `lookupExpandable`). The predicate is true for CS/Active that are
  // expandable OR undefined — so `\noexpand` smuggles `\dont_expand` for
  // both, which is what makes `\noexpand\undef` ≠ `\undef` under `\ifx`
  // (Gullet rewrites `\dont_expand X` into `\special_relax` with `X`
  // smuggled in slot[2]; that asymmetry is the whole point).
  DefMacro!(T_CS!("\\noexpand"), None, {
    if let Some(token) = gullet::read_token()? {
      let cc = token.get_catcode();
      if matches!(cc, Catcode::CS | Catcode::ACTIVE) && state::is_dont_expandable(&token) {
        vec![T_CS!("\\dont_expand"), token]
      } else {
        vec![token]
      }
    } else {
      // Missing token likely the result of "{\noexpand}" for which TeX would be unperturbed
      Vec::new()
    }
  });
  // NON-STANDARD:
  DefPrimitive!(T_CS!("\\dont_expand"), None, {
    Error!(
      "misdefined",
      "\\dont_expand",
      "The token \\dont_expand should never reach Stomach!"
    );
  });

  //======================================================================
  // \the
  //----------------------------------------------------------------------
  // \the              c  returns character tokens for an internal quantity's or parameter's current
  // value. The argument to \the is a variety of "Internal Quantities", being parameters,
  // registers, internal registers, codenames, etc. See TeX Book, pp.214--215.
  // [Since \the is expandable, perhaps should just be built into \the's code? Never need to revert]
  DefMacro!("\\the", {
    if let Some(token) = read_x_token(None, false, None)? {
      // Follow `\let`-alias chain so that `\the \tex_count:D` (= `\count`)
      // and `\the \__int_eval:w` (= `\numexpr`) — and any other expl3
      // primitive-alias to a register-style CS — find the underlying
      // register definition. Without this, `lookup_definition` returns
      // None for `Stored::Token` let-aliases and `\the` falls through
      // to the "undefined" branch, emitting a 0 plus a stub-installation.
      // Driver: 2406.14142 expl3 regex VM (`\the \__int_eval:w ... \__int_eval_end:`).
      let mut effective_token = token;
      for _ in 0..16 {
        match state::with_meaning(&effective_token, |m| match m {
          Some(Stored::Token(t)) => Some(*t),
          _ => None,
        }) {
          Some(t) if t != effective_token => effective_token = t,
          _ => break,
        }
      }
      match lookup_definition(&effective_token)? { Some(defn) => {
        if defn.is_register() {
          // SOME kind of register is acceptable
          let args = if let Some(params) = defn.get_parameters() {
            params.read_arguments(None)?
          } else {
            Vec::new()
          };
          return match defn.value_of(args) {
            Some(RegisterValue::Token(t)) => Ok(Tokens!(t)),
            Some(RegisterValue::Tokens(ts)) => Ok(ts),
            Some(other) => Ok(Tokens!(Explode!(other.to_string()))),
            None => Ok(Tokens!()),
          };
        } else if defn.get_cs_name() == "\\font" {
          // HACK to get the \fontcmd that would have selected the current font (see FontDef)
          match state::lookup_value("current_FontDef") { Some(Stored::Token(t)) => {
            return Ok(Tokens!(t));
          } _ => {
            return Ok(Tokens!(T_CS!("\\lx@default@font")));
          }}
        } else {
          // Perl: elsif ($defn->isFontDef) { return $defn->getCS; }
          // Check if this is a font CS defined by \font (has fontinfo in state)
          let cs_str = token.to_string();
          if let Some(Stored::Font(_)) = state::lookup_value(&s!("fontinfo_{cs_str}")) {
            return Ok(Tokens!(token));
          }
        }
      } _ => {
        // the token is Undefined
        if token.get_catcode() == Catcode::CS {
          // but IS a cs \something
          Error!(
            "expected",
            "<register>",
            "A <register> was supposed to be here",
            s!("Got {} Defining it now.", token)
          );
          // Hackery: to avoid potential repeated errors, define it now as a number register
          def_register(token, None, Number!(0), None)?; // Dimension, or what?
          return Ok(Tokens!(T_OTHER!("0")));
        }
      }}
      // If we fall through to here, whatever $token is shouldn't have been used with \the
      let (the_t, msg) =
        token.with_str(|tstr| (s!("\\the{tstr}"), s!("You can't use {tstr} after \\the")));
      Error!("unexpected", the_t, msg);
      T_OTHER!("0")
    } else {
      Error!(
        "expected",
        "<register>",
        "A <register> was supposed to be here. Got nothing."
      );
      T_OTHER!("0")
    }
  });
});

//! Token utilities: `split_tokens` and its delimiters, writable tokens, key/value specs, token case.

use super::*;

// is_definable — defined in latexml_core::binding::def::dialect, re-exported via prelude.
// Check if a token is "definable" — undefined or equivalent to `\relax`.
// Port of Perl `isDefinable($token)` (Base_Utility.pool.ltxml L33-40).

// split_tokens is defined below (moved from base_functions.rs) — includes meaning-based matching.
// Perl: SplitTokens($tokens, @delims) — Base_Utility.pool.ltxml L106-132.
// Splits a token list by delimiter tokens, respecting brace nesting and math mode.

//======================================================================
// A random collection of utility functions.
// [maybe need to do some reorganization?]
// Since this is used for textual tokens, typically to split author lists,
// we don't split within braces or math

/// A `SplitTokens` delimiter: Perl PR #2767 allows each delimiter to be a
/// single `Token`, OR a `Tokens` sequence to match in order.
#[derive(Debug, Clone)]
pub enum SplitDelim {
  /// A single Token delimiter (matched with the meaning-aware Equals below)
  Token(Token),
  /// A token sequence delimiter (matched literally, with the
  /// `T_SPACE` ~ `\ ` "HACK space" equivalence)
  Tokens(Tokens),
}
impl From<Token> for SplitDelim {
  fn from(t: Token) -> Self { SplitDelim::Token(t) }
}
impl From<Tokens> for SplitDelim {
  fn from(t: Tokens) -> Self { SplitDelim::Tokens(t) }
}

/// Does the remaining `stream` contain a `)` that balances the `(` just
/// popped? Used by `split_tokens` to protect delimiters inside *balanced*
/// parens only — an unbalanced `(` must NOT trigger paren-protection (else it
/// would greedily swallow the rest of the author block, including `\\`
/// name/affiliation separators).
pub(super) fn paren_closes_ahead(stream: &VecDeque<Token>) -> bool {
  let mut level = 1usize;
  for t in stream {
    if *t == T_OTHER!("(") {
      level += 1;
    } else if *t == T_OTHER!(")") {
      level -= 1;
      if level == 0 {
        return true;
      }
    }
  }
  false
}

/// Perl: SplitTokens($tokens, @delims) — Base_Utility.pool.ltxml (PR #2767).
/// Each of `delims` is a Token, OR Tokens to match a sequence.
/// Returns a list of Tokens for the sub-sequences, with leading/trailing
/// spaces trimmed from each piece, and any empty trailing piece dropped.
///
/// Like Perl, delimiters are not matched inside `{…}` braces or `$…$` math.
/// As an INTENTIONAL DIVERGENCE FROM PERL, they are also not matched inside
/// balanced `(…)` parentheses (see the paren branch below + OXIDIZED_DESIGN).
pub fn split_tokens(tokens: Tokens, delims: Vec<SplitDelim>) -> Vec<Tokens> {
  split_tokens_delimited(tokens, delims)
    .into_iter()
    .map(|(_, item)| item)
    .collect()
}

/// [`split_tokens`], each piece paired with the delimiter tokens that preceded it (none for the first), so a caller
/// that joins pieces back can put the delimiter where it stood.
pub fn split_tokens_delimited(
  tokens: Tokens,
  delims: Vec<SplitDelim>,
) -> Vec<(Vec<Token>, Tokens)> {
  let mut items: Vec<(Vec<Token>, Tokens)> = Vec::new();
  let mut before: Vec<Token> = Vec::new();
  let mut toks: Vec<Token> = Vec::new();
  let trim_spaces = |toks: &mut Vec<Token>| {
    while toks.first().is_some_and(|x| *x == T_SPACE!()) {
      toks.remove(0);
    }
    while toks.last().is_some_and(|x| *x == T_SPACE!()) {
      toks.pop();
    }
  };
  if !tokens.is_empty() {
    let mut stream: VecDeque<Token> = VecDeque::from(tokens.unlist());
    while let Some(t) = stream.pop_front() {
      let mut matched = false;
      let mut delimiter: Vec<Token> = Vec::new();
      for delim in &delims {
        match delim {
          SplitDelim::Token(d) => {
            // Perl: Equals($t, $delim); the Rust port additionally matches an alias of
            // the delimiter's own definition, so \AND (let to \and) matches \and. Only
            // its own: a delimiter `\let` to another command (`\and` disabled to `\relax`
            // by a `\maketitle`) would split at every alias of that command — `\protect`,
            // a disabled `\thanks` — in a later `\author` (59e; DIVERGENCES #36).
            if *d == t
              || (t.get_catcode() == Catcode::CS && d.get_catcode() == Catcode::CS && {
                let meaning_t = lookup_definition(&t).ok().flatten();
                let meaning_d = lookup_definition(d).ok().flatten();
                meaning_t.is_some()
                  && meaning_t == meaning_d
                  && meaning_d.is_some_and(|own| *own.get_cs() == *d)
              })
            {
              matched = true;
              delimiter = vec![t];
              break;
            }
          },
          SplitDelim::Tokens(seq) => {
            let mut tomatch: &[Token] = seq.unlist_ref();
            let mut peeked: Vec<Token> = Vec::new(); // tokens consumed beyond `t`
            let mut cur: Option<Token> = Some(t);
            while let (Some(c), Some(m)) = (cur, tomatch.first()) {
              // HACK space! A control space or thin space stands for the space of a space-bearing delimiter
              // (`Aghil Alaee\footnote{…} \,\,and Hari K. Kunduri`, 1407.0988: " and ")
              if c == *m || (*m == T_SPACE!() && (c == T_CS!("\\ ") || c == T_CS!("\\,"))) {
                tomatch = &tomatch[1..];
                if !tomatch.is_empty() {
                  cur = stream.pop_front();
                  if let Some(p) = cur {
                    peeked.push(p);
                  }
                }
              } else {
                break;
              }
            }
            if tomatch.is_empty() {
              matched = true;
              delimiter = std::iter::once(t).chain(peeked).collect();
              break;
            } else {
              // failed to match all: put back the peeked tokens
              for p in peeked.into_iter().rev() {
                stream.push_front(p);
              }
            }
          },
        }
      }
      if matched {
        trim_spaces(&mut toks);
        items.push((
          std::mem::replace(&mut before, delimiter),
          Tokens::new(std::mem::take(&mut toks)),
        ));
      } else if t.defined_as(&T_BEGIN!()) {
        toks.push(t);
        let mut level = 1;
        while let Some(t) = stream.pop_front() {
          match t.get_catcode() {
            Catcode::BEGIN => level += 1,
            Catcode::END => level -= 1,
            _ => {},
          }
          toks.push(t);
          if level < 1 {
            // done if balanced.
            break;
          }
        }
      } else if t.defined_as(&T_MATH!()) {
        // The span closes on a math shift as it opened, by meaning: an active `$` let to it (aastex's tables,
        // aas_support_sty.rs; witness 2609.05675) closes what it opens. Perl closes on any catcode-3 token
        // (Base_Utility.pool.ltxml:160-165), OXIDIZED_DESIGN_DIVERGENCES #450.
        toks.push(t);
        while let Some(t) = stream.pop_front() {
          let is_math = t.defined_as(&T_MATH!());
          toks.push(t);
          if is_math {
            break;
          }
        }
      } else if t == T_CS!("\\begin")
        && let Some(span) = environment_closes_ahead(&stream)
      {
        // INTENTIONAL DIVERGENCE FROM PERL (as for the parentheses below): an unbraced `\begin{name} ... \end{name}`
        // in an author block is one unit — a `{center}` affiliation under the names, a `{tabular}` holding
        // `COSIC, KU Leuven` — not split at its inner `and`/`,`/`\\`, which left the environment's ends in different
        // `\lx@personname`s (Perl's SplitTokens protects only braces and math, Base_Utility.pool.ltxml:152-165;
        // 2609.01563, 17357, 33831). Only when the matching `\end{name}` follows: an unbalanced `\begin` stays a token.
        toks.push(t);
        toks.extend(stream.drain(..span));
      } else if t == T_CS!("\\textsuperscript")
        && stream
          .front()
          .is_some_and(|next| !matches!(next.get_catcode(), Catcode::BEGIN | Catcode::SPACE))
      {
        // A superscript's one-token argument stays with it (`\textsuperscript*`); when that argument is a delimiter
        // (`Bob Baker\textsuperscript, Carl Cole`, the separator typeset raised; 2609.15009) the split is there, and the
        // superscript, with nothing left to raise, goes — it took the piece's closing brace as its argument.
        let next = *stream.front().unwrap();
        let splits_here = delims
          .iter()
          .any(|delim| matches!(delim, SplitDelim::Token(d) if *d == next));
        if !splits_here {
          toks.push(t);
          toks.extend(stream.pop_front());
        }
      } else if t == T_OTHER!("(") && paren_closes_ahead(&stream) {
        // INTENTIONAL DIVERGENCE FROM PERL: also protect delimiters inside
        // BALANCED parentheses (mirroring the brace/math skipping above), so a
        // parenthesized affiliation like "(Scuola Normale Superiore, Pisa)" is
        // NOT split at its internal comma into a spurious second author. Perl's
        // SplitTokens has no paren-awareness and makes exactly this mistake
        // (witness arXiv 0804.0870 — "Pisa)" became a second <personname>).
        // The `paren_closes_ahead` guard means an UNBALANCED `(` (rare/
        // malformed) is treated as an ordinary token, so it never swallows a
        // later `\\` name/affiliation separator. See OXIDIZED_DESIGN
        // "Intentional divergences". NOTE: bare (unparenthesized) commas in an
        // affiliation ("MIT, Cambridge") remain genuinely ambiguous — the same
        // tokens read as either one comma-affiliation or two authors — so we
        // match Perl's recall-oriented over-split there rather than guess.
        toks.push(t);
        let mut level = 1;
        while let Some(t) = stream.pop_front() {
          if t == T_OTHER!("(") {
            level += 1;
          } else if t == T_OTHER!(")") {
            level -= 1;
          }
          toks.push(t);
          if level < 1 {
            break;
          }
        }
      } else {
        toks.push(t);
      }
    }
  }
  trim_spaces(&mut toks);
  if !toks.is_empty() {
    items.push((before, Tokens::new(toks)));
  }
  items
}

/// Converts tokens to a string in the fashion of \message and others
///
/// doubles #, converts to string; optionally adds spaces after control sequences
/// in the spirit of the B Book, "show_token_list" routine, in 292.
/// [This could be a $tokens->unpackParameters, but for the curious space treatment]
pub fn writable_tokens(tokens: &Tokens) -> String {
  // Control sequences are written with the CURRENT `\escapechar`
  // (tex.web §1594 print_esc / eTeX: `\scantokens` writes as `\write`
  // would) — NOT a hardcoded backslash. xint's `\XINT_NewExpr` capture
  // (xintexpr.sty L4713-4740) relies on this: stage 1 sets
  // `\escapechar 126` with active `~` so re-scanned CS names become
  // inert `$noexpand$name` text; our old hardcoded `\` let `\the`/
  // `\romannumeral` EXECUTE inside the capturing edef, corrupting every
  // user-defined xint function body (tkz-grapheur ×4 recursion fatals,
  // the roman-numeral csname monster). Perl shares the hardcode
  // (TeX_Debugging.pool L52) and dies the same way — pdflatex is the
  // oracle. `escapechar()` reads the live register per call: xint flips
  // it between capture stages.
  let esc = escapechar();
  let mut out = String::new();
  for t in tokens.unlist_ref().iter() {
    match t.code {
      Catcode::CS => {
        with(t.text, |s| {
          let name = s.strip_prefix('\\').unwrap_or(s);
          out.push_str(&esc);
          out.push_str(name);
          // tex.web §262 print_cs: multi-letter names get a trailing
          // space; a single-char name gets one iff that char's CURRENT
          // catcode is LETTER.
          let mut chars = name.chars();
          let first = chars.next();
          if chars.next().is_some() {
            out.push(' ');
          } else if let Some(c) = first
            && lookup_catcode(c) == Some(Catcode::LETTER)
          {
            out.push(' ');
          }
        });
      },
      Catcode::SPACE => out.push(' '),
      Catcode::PARAM => t.with_str(|ts| {
        out.push_str(ts);
        out.push_str(ts);
      }),
      Catcode::ARG => {
        // B Book, 294. Reduce to param+integer
        out.push('#');
        t.with_str(|ts| out.push_str(ts));
      },
      _ => t.with_str(|ts| out.push_str(ts)),
    }
  }
  out
}

/// Support for Key / Value arguments.
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

// function to handle all the
#[derive(Default)]
pub struct KVSpec {
  pub star:    bool,
  pub plus:    bool,
  pub prefix:  Option<String>,
  pub keysets: Vec<String>,
  pub skip:    Vec<String>,
}
pub fn keyvals_aux(until: Option<Token>, spec: KVSpec) -> Result<KeyVals> {
  let KVSpec {
    mut star,
    plus,
    mut prefix,
    mut keysets,
    skip,
  } = spec;
  // support both "keysets" and "prefix|keysets"
  if keysets.is_empty() {
    if let Some(pfx) = prefix.take() {
      keysets = vec![pfx];
    }

    // to emulate old behaviour, throw no errors
    // when we have a single keyset and no prefix (or no keyset at all)
    if keysets.is_empty() {
      star = true;
    }
  }

  // create a new set of Key-Value arguments
  let mut keyvals = KeyVals::new(KeyvalsConfig {
    prefix,
    keysets,
    set_all: plus,
    set_internals: true,
    skip,
    skip_missing: if star {
      keyvals::SkipMissing::All
    } else {
      keyvals::SkipMissing::None
    },
    hook_missing: None,
  });
  // and read it from the gullet
  if let Some(until_token) = until {
    keyvals.read_from(until_token, false)?;
  }
  // we still want to make use of the hash
  Ok(keyvals)
}

pub fn uppercase_token(token: Token) -> Token { either_case_token(token, true) }
pub fn lowercase_token(token: Token) -> Token { either_case_token(token, false) }

fn either_case_token(token: Token, is_upper: bool) -> Token {
  let (chars_count, thischar) = token.with_str(|s| (s.chars().count(), s.chars().next()));
  // DG: new idea, short-circuit if more than 1 char, since our lccode/uccode tables are single
  // char-based (for now?)
  if chars_count != 1 {
    return token;
  }
  let mut result = String::new();
  let cased = if is_upper {
    lookup_uccode(thischar.unwrap())
  } else {
    lookup_lccode(thischar.unwrap())
  };
  if let Some(code) = cased {
    if code != 0 {
      result.push_str(
        &decode_utf16([code])
          .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
          .collect::<String>(),
      )
    } else {
      result.push(thischar.unwrap());
    }
  } else {
    result.push(thischar.unwrap());
  }
  if token.with_str(|initial_str| initial_str != result) {
    // the case changes the character, not the token's out-of-band field provenance (`Token::prov`)
    Token {
      prov: token.prov,
      ..Token::new(result, token.get_catcode())
    }
  } else {
    token
  }
}

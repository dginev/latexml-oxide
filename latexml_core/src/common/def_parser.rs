use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
  common::{
    arena::{self},
    error::*,
  },
  mouth,
  parameter::{Parameter, Parameters},
  token::*,
  tokens::Tokens,
};

static CSNAME_MACRO_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\\csname\s+(.*)\\endcsname").unwrap());
// Includes `_` (expl3 private/word-internal) and an optional `:<letters>`
// suffix (expl3 parameter-type sigil) so prototype strings like
// "\\draw_path_arc:nnn{}{}{}" parse with the entire `\draw_path_arc:nnn`
// as the control-sequence name. Under normal LaTeX catcodes `_` is SUB
// and `:` is OTHER, so these names only round-trip through the
// tokenizer under expl3 catcode regime — but compile-time prototype
// strings bypass the tokenizer, so this is purely a string-parsing
// concern. Witness: l3draw_sty stubs would previously fail with
// "Unrecognized parameter type with name '_begin', spec '_begin:'".
static CS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\\[a-zA-Z@_]+(?::[a-zA-Z]*)?)").unwrap());
static SINGLE_CHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\\.)").unwrap());
static ACTIVE_CHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.)").unwrap());

/// If calling at compile-time, pass `None` for state, to avoid initialization.
pub fn parse_prototype(proto: &str, init_flag: bool) -> Result<(Token, Option<Parameters>)> {
  let cs;
  let normalized_proto = if let Some(captures) = CSNAME_MACRO_RE.captures(proto) {
    let csname_content = captures.get(1).map_or("", |m| m.as_str());
    // At compile time, reject \csname patterns with braces — these produce CS names
    // like \begin{env} that can cause infinite loops in the proc macro expansion.
    // Starred environments (\csname eqnarray*\endcsname) are fine.
    if !init_flag && (csname_content.contains('{') || csname_content.contains('}')) {
      panic!(
        "\\csname...\\endcsname with braces in definition prototype is not supported at compile time: \"{}\". \
         Use RawTeX!() or runtime DefMacroI() for CS names containing {{}} characters.",
        proto
      );
    }
    cs = T_CS!(s!("\\{}", csname_content));
    // also replace in proto
    CSNAME_MACRO_RE.replace(proto, "")
  } else if let Some(captures) = CS_RE.captures(proto) {
    // Match a cs
    let csname = captures.get(1).map_or("", |m| m.as_str()).to_string();
    cs = T_CS!(csname);
    // also replace in proto
    CS_RE.replace(proto, "")
  } else if let Some(captures) = SINGLE_CHAR_RE.captures(proto) {
    // Match a single char cs, env name,...
    cs = T_CS!(captures.get(1).map_or("", |m| m.as_str()));
    // also replace in proto
    SINGLE_CHAR_RE.replace(proto, "")
  } else if let Some(captures) = ACTIVE_CHAR_RE.captures(proto) {
    // Match an active char
    cs = mouth::tokenize_internal(captures.get(1).map_or("", |m| m.as_str()))
      .unlist()
      .remove(0);
    // also replace in proto
    ACTIVE_CHAR_RE.replace(proto, "")
  } else {
    let message = s!(
      "Definition prototype doesn't have proper control sequence: \"{}\"",
      proto
    );
    fatal!(Prototype, Misdefined, message);
  };
  let final_proto = normalized_proto.trim();
  let paramlist = parse_parameters(final_proto, &cs, init_flag)?;
  Ok((cs, paramlist))
}

/// If calling at compile-time, pass `None` for state, to avoid initialization.
pub fn parse_parameters(
  outer_prototype: &str,
  cs: &Token,
  init_flag: bool,
) -> Result<Option<Parameters>> {
  // The prototype grammar, as a winnow parse (#171 family; the golden corpus
  // in `golden_tests` pins byte-equivalence with the prior regex munch loop):
  //   item := "{" inner "}" \s*          (Plain, recursive inner)
  //         | "[" inner "]" \s*          (Optional; inner may be Default:…)
  //         | word (":" extra)? \s*      (named type, extra |-split)   [nonempty]
  //         | any-single-char            (literal Token; spaces too — each
  //                                       space is its own Token, faithfully)
  use winnow::{
    combinator::{delimited, opt, preceded},
    prelude::*,
    token::{any, take_while},
  };

  fn ws0(input: &mut &str) -> ModalResult<()> {
    take_while(0.., |c: char| c.is_whitespace())
      .void()
      .parse_next(input)
  }
  /// `{inner}` / `[inner]` group: returns the inner slice; trailing \s* eaten.
  fn group<'a>(open: char, close: char) -> impl FnMut(&mut &'a str) -> ModalResult<&'a str> {
    move |input: &mut &'a str| {
      let inner = delimited(open, take_while(0.., move |c| c != close), close).parse_next(input)?;
      ws0(input)?;
      Ok(inner)
    }
  }
  /// `word(:extra)?` with word ∈ \w+ or a bare nonempty `:extra`; \s* eaten.
  fn paramspec<'a>(input: &mut &'a str) -> ModalResult<(&'a str, Option<&'a str>)> {
    let word = take_while(0.., |c: char| c.is_alphanumeric() || c == '_').parse_next(input)?;
    let extra = opt(preceded(
      ':',
      take_while(0.., |c: char| !c.is_whitespace() && c != '{' && c != '['),
    ))
    .parse_next(input)?;
    if word.is_empty() && extra.is_none() {
      return Err(winnow::error::ErrMode::Backtrack(
        winnow::error::ContextError::new(),
      ));
    }
    ws0(input)?;
    Ok((word, extra))
  }

  let mut parameters = Vec::with_capacity(4);
  let mut rest: &str = outer_prototype;
  let input = &mut rest;
  while !input.is_empty() {
    // Probe-and-commit on a copy per branch: a failing branch must not consume
    // (`delimited` advances past its opening token before failing otherwise —
    // e.g. a lone "{" from the non-nesting inner of "{{}}").
    let mut probe;
    let res = {
      probe = *input;
      group('{', '}')
        .parse_next(&mut probe)
        .inspect(|_| *input = probe)
    };
    let mut p: Parameter = match res {
      Ok(inner_spec) => {
        // Plain (possibly typed-inner) braced group, spec keeps its braces.
        let inner: Option<Parameters> = if inner_spec.is_empty() {
          None
        } else {
          parse_parameters(inner_spec, cs, init_flag)?
        };
        Parameter {
          name: arena::pin_static("Plain"),
          spec: arena::pin(format!("{{{inner_spec}}}")),
          inner: inner.map(|ps| ps.into()).unwrap_or_default(),
          ..Parameter::default()
        }
      },
      _ => {
        let res = {
          probe = *input;
          group('[', ']')
            .parse_next(&mut probe)
            .inspect(|_| *input = probe)
        };
        match res {
          Ok(inner_spec) => {
            let spec = arena::pin(format!("[{inner_spec}]"));
            if let Some(default_str) = inner_spec.strip_prefix("Default:") {
              let extra = if default_str.is_empty() {
                vec![]
              } else {
                vec![mouth::tokenize_internal(default_str)]
              };
              Parameter {
                name: arena::pin_static("Optional"),
                spec,
                extra,
                ..Parameter::default()
              }
            } else if !inner_spec.is_empty() {
              Parameter {
                name: arena::pin_static("Optional"),
                spec,
                inner: parse_parameters(inner_spec, cs, init_flag)?
                  .map(|ps| ps.into())
                  .unwrap_or_default(),
                ..Parameter::default()
              }
            } else {
              Parameter {
                name: arena::pin_static("Optional"),
                spec,
                ..Parameter::default()
              }
            }
          },
          _ => {
            let res = {
              probe = *input;
              paramspec.parse_next(&mut probe).inspect(|_| *input = probe)
            };
            match res {
              Ok((word, extra_opt)) => {
                let spec_str = match extra_opt {
                  Some(extra) => format!("{word}:{extra}"),
                  None => word.to_string(),
                };
                let extra: Vec<Tokens> = match extra_opt {
                  None | Some("") => Vec::new(),
                  Some(extra_str) => extra_str
                    .split('|')
                    .map(|t| Tokens::new(mouth::tokenize_internal(t).unlist()))
                    .collect(),
                };
                Parameter {
                  name: arena::pin(word),
                  spec: arena::pin(&spec_str),
                  extra,
                  ..Parameter::default()
                }
              },
              _ => {
                // Literal single char (incl. each whitespace char) as a Token parameter.
                let ch = any.parse_next(input).map_err(
                  |_: winnow::error::ErrMode<winnow::error::ContextError>| {
                    Error::from(s!(
                      "parse_parameters: unreadable prototype tail for {:?}",
                      cs
                    ))
                  },
                )?;
                let ch_token = CharToken!(ch, Catcode::OTHER);
                Parameter {
                  name: arena::pin_static("Token"),
                  spec: arena::pin_static("Token"),
                  extra: vec![Tokens::new(vec![ch_token])],
                  ..Parameter::default()
                }
              },
            }
          },
        }
      },
    };
    if init_flag {
      p = p.init()?;
    }
    parameters.push(p);
  }
  if parameters.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters::new(parameters)))
  }
}

#[cfg(test)]
mod golden_tests {
  /// Render a parse result in a stable, comparison-friendly form.
  fn describe(proto: &str) -> String {
    match super::parse_parameters(proto, &crate::T_CS!("\\x"), false) {
      Ok(None) => "None".to_string(),
      Ok(Some(ps)) => ps
        .get_parameters()
        .iter()
        .map(|p| {
          format!(
            "{}:{}/x{}/i{}",
            crate::common::arena::with(p.name, |s| s.to_string()),
            crate::common::arena::with(p.spec, |s| s.to_string()),
            p.extra.len(),
            p.inner
              .as_ref()
              .map(|ps| ps.get_parameters().len())
              .unwrap_or(0)
          )
        })
        .collect::<Vec<_>>()
        .join(" | "),
      Err(e) => format!("ERR:{e}"),
    }
  }

  /// Golden corpus pinning the prototype grammar's behavior (captured from
  /// the regex implementation 2026-06-10) — the gate for the winnow rewrite:
  /// any divergence here is a semantics change, not a refactor.
  #[test]
  fn golden_prototype_corpus() {
    crate::state::set_state(crate::state::State::new(
      crate::state::StateOptions::default(),
    ));
    let golden: &[(&str, &str)] = &[
      ("{}", "Plain:{}/x0/i0"),
      ("{}{}", "Plain:{}/x0/i0 | Plain:{}/x0/i0"),
      ("[]", "Optional:[]/x0/i0"),
      ("[]{}", "Optional:[]/x0/i0 | Plain:{}/x0/i0"),
      ("{Number}", "Plain:{Number}/x0/i1"),
      (
        "{Float}{Float} {}",
        "Plain:{Float}/x0/i1 | Plain:{Float}/x0/i1 | Plain:{}/x0/i0",
      ),
      (
        "OptionalMatch:* [][] Semiverbatim",
        "OptionalMatch:OptionalMatch:*/x1/i0 | Optional:[]/x0/i0 | Optional:[]/x0/i0 | \
         Semiverbatim:Semiverbatim/x0/i0",
      ),
      (
        "OptionalKeyVals:LST",
        "OptionalKeyVals:OptionalKeyVals:LST/x1/i0",
      ),
      (
        "RequiredKeyVals:RH {}",
        "RequiredKeyVals:RequiredKeyVals:RH/x1/i0 | Plain:{}/x0/i0",
      ),
      ("Until:\\end", "Until:Until:\\end/x1/i0"),
      (
        "XUntil:\\fi {}",
        "XUntil:XUntil:\\fi/x1/i0 | Plain:{}/x0/i0",
      ),
      (
        "[Default:0]{}",
        "Optional:[Default:0]/x1/i0 | Plain:{}/x0/i0",
      ),
      ("Semiverbatim", "Semiverbatim:Semiverbatim/x0/i0"),
      (
        "SkipSpaces {}",
        "SkipSpaces:SkipSpaces/x0/i0 | Plain:{}/x0/i0",
      ),
      ("Digested", "Digested:Digested/x0/i0"),
      ("DigestedBody", "DigestedBody:DigestedBody/x0/i0"),
      (
        "(){}",
        "Token:Token/x1/i0 | Token:Token/x1/i0 | Plain:{}/x0/i0",
      ),
      (
        "( {Float} , {Float} )",
        "Token:Token/x1/i0 | Token:Token/x1/i0 | Plain:{Float}/x0/i1 | Token:Token/x1/i0 | \
         Token:Token/x1/i0 | Plain:{Float}/x0/i1 | Token:Token/x1/i0",
      ),
      ("Optional:=Default:9", "Optional:Optional:=Default:9/x1/i0"),
      ("{Until:;}", "Plain:{Until:;}/x0/i1"),
      ("+", "Token:Token/x1/i0"),
      ("Match:- {}", "Match:Match:-/x1/i0 | Plain:{}/x0/i0"),
      // pst_all_sty shapes: non-nesting braced inner ("{{}}" -> Plain with a
      // lone "{" inner Token, then a dangling "}" Token) — regex-faithful.
      (
        "OptionalMatch:* {{}} [] {}",
        "OptionalMatch:OptionalMatch:*/x1/i0 | Plain:{{}/x0/i1 | Token:Token/x1/i0 | Token:Token/x1/i0 | \
         Optional:[]/x0/i0 | Plain:{}/x0/i0",
      ),
      ("{", "Token:Token/x1/i0"),
    ];
    for (proto, expected) in golden {
      let expected = expected.split_whitespace().collect::<Vec<_>>().join(" ");
      let actual = describe(proto)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
      assert_eq!(actual, expected, "prototype grammar diverged on {proto:?}");
    }
  }
}

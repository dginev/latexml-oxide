use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;

use crate::common::arena::{self};
use crate::common::error::*;

use crate::mouth;
use crate::parameter::{Parameter, Parameters};
use crate::pin;
use crate::token::*;
use crate::tokens::Tokens;

static CSNAME_MACRO_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\\csname\s+(.*)\\endcsname").unwrap());
static CS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\\[a-zA-Z@]+)").unwrap());
static SINGLE_CHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\\.)").unwrap());
static ACTIVE_CHAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.)").unwrap());
static DEFAULT_CHECK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^Default:(.*)$").unwrap());
static NESTED_CHECK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\{([^\}]*)\})\s*").unwrap());
static OPTIONAL_CHECK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\[([^\]]*)\])\s*").unwrap());
static PARAMSPECT_CHECK_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^((\w*)(:([^\s\{\[]*))?)\s*").unwrap());

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
  let mut prototype = Cow::Borrowed(outer_prototype);
  // parameters is capped at MAX_STEPS; most prototypes have ≤ 4 params,
  // so pre-size conservatively for the common case.
  let mut parameters = Vec::with_capacity(4);
  let mut steps = 0;
  const MAX_STEPS: usize = 50;
  while !prototype.is_empty() {
    steps += 1;
    if steps > MAX_STEPS {
      fatal!(
        Parameter,
        Misdefined,
        s!(
          "parse_parameters exceeded {} steps for {:?} (remaining: {:?})",
          MAX_STEPS,
          cs,
          prototype
        )
      );
    }
    let next_proto: Cow<str>;
    // Handle possibly nested cases, such as {Number}
    if NESTED_CHECK_RE.is_match(&prototype) {
      let captures = NESTED_CHECK_RE.captures(&prototype).unwrap();
      next_proto = NESTED_CHECK_RE.replace(&prototype, "");
      let spec = captures.get(1).map_or("", |m| m.as_str());
      let inner_spec = captures.get(2).map_or("", |m| m.as_str());
      let inner: Option<Parameters> = if inner_spec.is_empty() {
        None
      } else {
        parse_parameters(inner_spec, cs, init_flag)?
      };
      let mut p = Parameter {
        name: arena::pin_static("Plain"),
        spec: if spec.is_empty() {
          pin!("")
        } else {
          arena::pin(spec)
        },
        inner: inner.map(|ps| ps.into()).unwrap_or_default(),
        ..Parameter::default()
      };
      if init_flag {
        p = p.init()?;
      }
      parameters.push(p);
    } else if let Some(captures) = OPTIONAL_CHECK_RE.captures(&prototype) {
      // Ditto for Optional
      let spec = captures.get(1).map_or("", |m| m.as_str());
      let inner_spec = captures.get(2).map_or("", |m| m.as_str());
      next_proto = OPTIONAL_CHECK_RE.replace(&prototype, "");
      if let Some(default_captures) = DEFAULT_CHECK_RE.captures(inner_spec) {
        // Store the default value as extra tokens so the Optional parameter
        // returns it when no [...] is present (Perl: $param->{default}).
        let default_str = default_captures.get(1).map_or("", |m| m.as_str());
        let default_tokens = if default_str.is_empty() {
          vec![]
        } else {
          vec![crate::mouth::tokenize_internal(default_str)]
        };
        let mut p = Parameter {
          name: arena::pin_static("Optional"),
          spec: if spec.is_empty() {
            pin!("")
          } else {
            arena::pin(spec)
          },
          extra: default_tokens,
          ..Parameter::default()
        };
        if init_flag {
          p = p.init()?;
        }
        parameters.push(p);
      } else if !inner_spec.is_empty() {
        let mut p = Parameter {
          name: arena::pin_static("Optional"),
          spec: if spec.is_empty() {
            pin!("")
          } else {
            arena::pin(spec)
          },
          inner: parse_parameters(inner_spec, cs, init_flag)?
            .map(|ps| ps.into())
            .unwrap_or_default(),
          ..Parameter::default()
        };
        if init_flag {
          p = p.init()?;
        }
        parameters.push(p);
      } else {
        let mut p = Parameter {
          name: arena::pin_static("Optional"),
          spec: arena::pin(spec),
          ..Parameter::default()
        };
        if init_flag {
          p = p.init()?;
        }
        parameters.push(p);
      }
    } else if let Some(captures) = PARAMSPECT_CHECK_RE.captures(&prototype) {
      let full_match = captures.get(0).map_or("", |m| m.as_str());
      if full_match.is_empty() || full_match.chars().all(|c| c.is_whitespace()) {
        // PARAMSPECT_CHECK_RE matched empty string (e.g. on '+', '-').
        // Perl fallback: consume next char as a literal Token parameter.
        // Perl: push(@params, LaTeXML::Core::Parameter->new('Token','Token',
        //   extra => [T_OTHER($ch)]));
        let ch = prototype.chars().next().unwrap();
        let ch_token = CharToken!(ch, Catcode::OTHER);
        let mut p = Parameter {
          name: arena::pin_static("Token"),
          spec: arena::pin_static("Token"),
          extra: vec![Tokens::new(vec![ch_token])],
          ..Parameter::default()
        };
        if init_flag {
          p = p.init()?;
        }
        parameters.push(p);
        next_proto = Cow::Owned(prototype[ch.len_utf8()..].to_string());
      } else {
        let spec = arena::pin(captures.get(1).map_or("", |m| m.as_str()));
        let name = arena::pin(captures.get(2).map_or("", |m| m.as_str()));
        let extra_str = captures.get(4).map_or("", |m| m.as_str()).to_string();
        next_proto = PARAMSPECT_CHECK_RE.replace(&prototype, "");
        let extra: Vec<Tokens> = if extra_str.is_empty() {
          Vec::new()
        } else {
          extra_str
            .split('|')
            .map(|t| Tokens::new(mouth::tokenize_internal(t).unlist()))
            .collect()
        };
        let mut p = Parameter {
          name,
          spec,
          extra,
          ..Parameter::default()
        };
        if init_flag {
          p = p.init()?;
        }
        parameters.push(p);
      }
    } else {
      // Perl fallback: consume next char as a literal Token parameter.
      let ch = prototype.chars().next().unwrap();
      let ch_token = CharToken!(ch, Catcode::OTHER);
      let mut p = Parameter {
        name: arena::pin_static("Token"),
        spec: arena::pin_static("Token"),
        extra: vec![Tokens::new(vec![ch_token])],
        ..Parameter::default()
      };
      if init_flag {
        p = p.init()?;
      }
      parameters.push(p);
      next_proto = Cow::Owned(prototype[ch.len_utf8()..].to_string());
    }
    prototype = Cow::Owned(next_proto.to_string());
  }
  if parameters.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters::new(parameters)))
  }
}

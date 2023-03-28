use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;

use crate::common::error::*;

use crate::mouth;
use crate::tokens::Tokens;
use crate::parameter::{Parameter, Parameters};
use crate::state::State;
use crate::token::*;

lazy_static! {
  static ref CSNAME_MACRO_RE: Regex = Regex::new(r"^\\csname\s+(.*)\\endcsname").unwrap();
  static ref CS_RE: Regex = Regex::new(r"^(\\[a-zA-Z@]+)").unwrap();
  static ref SINGLE_CHAR_RE: Regex = Regex::new(r"^(\\.)").unwrap();
  static ref ACTIVE_CHAR_RE: Regex = Regex::new(r"^(.)").unwrap();
  static ref DEFAULT_CHECK_RE: Regex = Regex::new(r"^Default:(.*)$").unwrap();
  static ref NESTED_CHECK_RE: Regex = Regex::new(r"^(\{([^\}]*)\})\s*").unwrap();
  static ref OPTIONAL_CHECK_RE: Regex = Regex::new(r"^(\[([^\]]*)\])\s*").unwrap();
  static ref PARAMSPECT_CHECK_RE: Regex = Regex::new(r"^((\w*)(:([^\s\{\[]*))?)\s*").unwrap();
}

/// If calling at compile-time, pass `None` for state, to avoid initialization.
pub fn parse_prototype(proto: &str, state_opt: Option<&mut State>) -> Result<(Token, Option<Parameters>)> {
  let cs;
  let final_proto = if let Some(captures) = CSNAME_MACRO_RE.captures(proto) {
    cs = T_CS!(s!("\\{}", captures.get(1).map_or("", |m| m.as_str())));
    // also replace in proto
    CSNAME_MACRO_RE.replace(proto, "").to_string()
  } else if let Some(captures) = CS_RE.captures(proto) {
    // Match a cs
    let csname = captures.get(1).map_or("", |m| m.as_str()).to_string();
    cs = T_CS!(csname);
    // also replace in proto
    CS_RE.replace(proto, "").to_string()
  } else if let Some(captures) = SINGLE_CHAR_RE.captures(proto) {
    // Match a single char cs, env name,...
    cs = T_CS!(captures.get(1).map_or("", |m| m.as_str()).to_string());
    // also replace in proto
    SINGLE_CHAR_RE.replace(proto, "").to_string()
  } else if let Some(captures) = ACTIVE_CHAR_RE.captures(proto) {
    // Match an active char
    cs = mouth::tokenize_internal(captures.get(1).map_or("", |m| m.as_str())).unlist().remove(0);
    // also replace in proto
    ACTIVE_CHAR_RE.replace(proto, "").to_string()
  } else {
    let message = s!("Definition prototype doesn't have proper control sequence: \"{}\"", proto);
    fatal!(Prototype, Misdefined, None, state, message);
  }
  .trim()
  .to_string();
  let paramlist = parse_parameters(final_proto, &cs, state_opt)?;
  Ok((cs, paramlist))
}

/// If calling at compile-time, pass `None` for state, to avoid initialization.
pub fn parse_parameters(mut prototype: String, cs: &Token, mut state_opt: Option<&mut State>) -> Result<Option<Parameters>> {
  let mut parameters = Vec::new();
  while !prototype.is_empty() {
    let next_proto: String;
    // Handle possibly nested cases, such as {Number}
    if NESTED_CHECK_RE.is_match(&prototype) {
      let captures = NESTED_CHECK_RE.captures(&prototype).unwrap();
      next_proto = NESTED_CHECK_RE.replace(&prototype, "").to_string();
      let spec = captures.get(1).map_or("", |m| m.as_str());
      let inner_spec = captures.get(2).map_or("", |m| m.as_str());
      let inner: Option<Parameters> = if inner_spec.is_empty() {
        None
      } else {
        parse_parameters(inner_spec.to_string(), cs, state_opt.as_deref_mut())?
      };
      let mut p = Parameter {
        name: Cow::Borrowed("Plain"),
        spec: if spec.is_empty() {
          Cow::Borrowed("")
        } else {
          Cow::Owned(spec.to_string())
        },
        inner: inner.map(|ps| ps.into()).unwrap_or_default(),
        ..Parameter::default()
      };
      if let Some(state) = &mut state_opt {
        p = p.init(state)?;
      }
      parameters.push(p);
    } else if let Some(captures) = OPTIONAL_CHECK_RE.captures(&prototype) {
      // Ditto for Optional
      let spec = captures.get(1).map_or("", |m| m.as_str());
      let inner_spec = captures.get(2).map_or("", |m| m.as_str());
      next_proto = OPTIONAL_CHECK_RE.replace(&prototype, "").to_string();
      if let Some(_default_captures) = DEFAULT_CHECK_RE.captures(inner_spec) {
        // TODO: Add the defaults !
        let mut p = Parameter {
          name: Cow::Borrowed("Optional"),
          spec: if spec.is_empty() {
            Cow::Borrowed("")
          } else {
            Cow::Owned(spec.to_string())
          },
          // extra: vec![TokenizeInternal!(default_captures.get(0).map_or("", |m| m.as_str()))],
          ..Parameter::default()
        };
        if let Some(ref mut state) = &mut state_opt {
          p = p.init(state)?;
        }
        parameters.push(p);
      } else if !inner_spec.is_empty() {
        let mut p = Parameter {
          name: Cow::Borrowed("Optional"),
          spec: if spec.is_empty() {
            Cow::Borrowed("")
          } else {
            Cow::Owned(spec.to_string())
          },
          inner: parse_parameters(inner_spec.to_string(), cs, state_opt.as_deref_mut())?
            .map(|ps| ps.into()).unwrap_or_default(),
          ..Parameter::default()
        };
        if let Some(ref mut state) = &mut state_opt {
          p = p.init(state)?;
        }
        parameters.push(p);
      } else {
        let mut p = Parameter {
          name: Cow::Borrowed("Optional"),
          spec: Cow::Owned(spec.to_string()),
          ..Parameter::default()
        };
        if let Some(state) = &mut state_opt {
          p = p.init(state)?;
        }
        parameters.push(p);
      }
    } else if let Some(captures) = PARAMSPECT_CHECK_RE.captures(&prototype) {
      let spec = captures.get(1).map_or("", |m| m.as_str()).to_string();
      let name = captures.get(2).map_or("", |m| m.as_str()).to_string();
      let extra_str = captures.get(4).map_or("", |m| m.as_str()).to_string();
      next_proto = PARAMSPECT_CHECK_RE.replace(&prototype, "").to_string();
      let extra : Vec<Tokens> = if extra_str.is_empty() { Vec::new() } else {
        extra_str.split('|')
          .map(|t| Tokens::new(mouth::tokenize_internal(t).unlist())).collect()
      };
      let mut p = Parameter {
        name: name.into(),
        spec: spec.into(),
        extra,
        ..Parameter::default()
      };
      if let Some(ref mut state) = &mut state_opt {
        p = p.init(state)?;
      }
      parameters.push(p);
    } else {
      fatal!(
        Parameter,
        Misdefined,
        s!("Unrecognized parameter specification at \"prototype\" {:?}", cs)
      );
    }
    prototype = next_proto.to_string();
  }
  if parameters.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters::new(parameters)))
  }
}

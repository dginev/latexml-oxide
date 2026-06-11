//! Unit tests for `latexml_core::common::def_parser`.
//!
//! These cover the CS-name extraction branches of `parse_prototype` (plain
//! `\foo`, `\csname foo\endcsname`, single-char `\(`, active char) and the
//! parameter-spec extraction branches of `parse_parameters` (Nested `{...}`,
//! Optional `[...]`, named spec `Name:extra`, and literal-Token fallback for
//! bare delimiters like `+`).
//!
//! All tests run with `init_flag = false` (compile-time mode) to keep the
//! parameters unvalidated — validation requires a running State which these
//! unit tests don't set up.

use latexml_core::{
  T_CS,
  common::{
    arena,
    def_parser::{parse_parameters, parse_prototype},
  },
  state::{State, StateOptions, set_state},
};

fn sym_str(s: arena::SymStr) -> String { arena::to_string(s) }

fn setup() {
  // parse_prototype / parse_parameters use mouth::tokenize_internal
  // which needs thread-local State. An empty state is fine.
  set_state(State::new(StateOptions::default()));
}

#[test]
fn prototype_plain_cs() {
  setup();
  let (cs, params) = parse_prototype("\\foo", false).unwrap();
  assert_eq!(cs, T_CS!("\\foo"));
  assert!(params.is_none(), "no params for bare CS");
}

#[test]
fn prototype_cs_with_trailing_param_spec() {
  setup();
  let (cs, params) = parse_prototype("\\foo{}", false).unwrap();
  assert_eq!(cs, T_CS!("\\foo"));
  let ps = params.expect("nested {} produces a Plain parameter");
  assert_eq!(ps.get_parameters().len(), 1);
  assert_eq!(sym_str(ps.get_parameters()[0].name), "Plain");
}

#[test]
fn prototype_csname_endcsname() {
  setup();
  // `\csname theta\endcsname` should parse as `\theta`
  let (cs, params) = parse_prototype("\\csname theta\\endcsname", false).unwrap();
  assert_eq!(cs, T_CS!("\\theta"));
  assert!(params.is_none());
}

#[test]
fn prototype_csname_rejects_braces_at_compile_time() {
  setup();
  // init_flag=false → panics on brace-containing csname (see compile-time guard).
  let result =
    std::panic::catch_unwind(|| parse_prototype("\\csname begin{foo}\\endcsname", false));
  assert!(
    result.is_err(),
    "compile-time csname with braces must panic"
  );
}

#[test]
fn prototype_single_char_cs() {
  setup();
  // `\(` is a single-char CS (punctuation after backslash).
  let (cs, _) = parse_prototype("\\(", false).unwrap();
  assert_eq!(cs, T_CS!("\\("));
}

#[test]
fn prototype_expl3_underscore_in_csname() {
  setup();
  // expl3 word-internal `_` — `\draw_begin:` is a single CS, not
  // `\draw` + `_begin:` parameter spec. See `a8010f606a`.
  let (cs, params) = parse_prototype("\\draw_begin:", false).unwrap();
  assert_eq!(cs, T_CS!("\\draw_begin:"));
  assert!(params.is_none());
}

#[test]
fn prototype_expl3_colon_sigil_with_letters() {
  setup();
  // expl3 parameter-type sigil `:nnn` is part of the CS name.
  let (cs, params) = parse_prototype("\\draw_path_arc:nnn", false).unwrap();
  assert_eq!(cs, T_CS!("\\draw_path_arc:nnn"));
  assert!(params.is_none());
}

#[test]
fn prototype_expl3_colon_sigil_with_trailing_braces() {
  setup();
  // expl3 CS name + 3 mandatory args. The CS name must absorb the
  // `:nnn` sigil; the trailing `{}{}{}` is the parameter spec.
  let (cs, params) = parse_prototype("\\draw_path_arc:nnn{}{}{}", false).unwrap();
  assert_eq!(cs, T_CS!("\\draw_path_arc:nnn"));
  let ps = params.expect("three mandatory args");
  assert_eq!(ps.get_parameters().len(), 3);
  for p in ps.get_parameters() {
    assert_eq!(sym_str(p.name), "Plain");
  }
}

#[test]
fn prototype_empty_is_error() {
  setup();
  // Empty proto triggers the `fatal!` error path (returns Err, not panic).
  let result = parse_prototype("", false);
  assert!(result.is_err(), "empty prototype should return Err");
}

#[test]
fn parse_parameters_empty_is_none() {
  setup();
  let cs = T_CS!("\\foo");
  let ps = parse_parameters("", &cs, false).unwrap();
  assert!(ps.is_none());
}

#[test]
fn parse_parameters_nested_plain() {
  setup();
  let cs = T_CS!("\\foo");
  let ps = parse_parameters("{}", &cs, false).unwrap().unwrap();
  let params = ps.get_parameters();
  assert_eq!(params.len(), 1);
  assert_eq!(sym_str(params[0].name), "Plain");
}

#[test]
fn parse_parameters_optional() {
  setup();
  let cs = T_CS!("\\foo");
  let ps = parse_parameters("[]", &cs, false).unwrap().unwrap();
  let params = ps.get_parameters();
  assert_eq!(params.len(), 1);
  assert_eq!(sym_str(params[0].name), "Optional");
}

#[test]
fn parse_parameters_optional_with_default() {
  setup();
  let cs = T_CS!("\\foo");
  let ps = parse_parameters("[Default:0]", &cs, false)
    .unwrap()
    .unwrap();
  let params = ps.get_parameters();
  assert_eq!(params.len(), 1);
  assert_eq!(sym_str(params[0].name), "Optional");
  // Default value is stored in extra
  assert_eq!(params[0].extra.len(), 1);
}

#[test]
fn parse_parameters_named_spec() {
  setup();
  let cs = T_CS!("\\foo");
  let ps = parse_parameters("Number", &cs, false).unwrap().unwrap();
  let params = ps.get_parameters();
  assert_eq!(params.len(), 1);
  assert_eq!(sym_str(params[0].name), "Number");
}

#[test]
fn parse_parameters_multiple() {
  setup();
  let cs = T_CS!("\\foo");
  let ps = parse_parameters("Number Number", &cs, false)
    .unwrap()
    .unwrap();
  let params = ps.get_parameters();
  assert_eq!(params.len(), 2);
}

#[test]
fn parse_parameters_mixed_optional_then_plain() {
  setup();
  let cs = T_CS!("\\foo");
  let ps = parse_parameters("[]{}", &cs, false).unwrap().unwrap();
  let params = ps.get_parameters();
  assert_eq!(params.len(), 2);
  assert_eq!(sym_str(params[0].name), "Optional");
  assert_eq!(sym_str(params[1].name), "Plain");
}

#[test]
fn parse_parameters_bare_delimiter_falls_back_to_literal_token() {
  setup();
  // A lone '+' doesn't match any named spec — the fallback produces a
  // Token parameter with the '+' stored as OTHER in `extra`.
  let cs = T_CS!("\\foo");
  let ps = parse_parameters("+", &cs, false).unwrap().unwrap();
  let params = ps.get_parameters();
  assert_eq!(params.len(), 1);
  assert_eq!(sym_str(params[0].name), "Token");
  assert_eq!(sym_str(params[0].spec), "Token");
  assert!(!params[0].extra.is_empty());
}

#[test]
fn parse_parameters_param_with_extra() {
  setup();
  let cs = T_CS!("\\foo");
  let ps = parse_parameters("Until:stop", &cs, false).unwrap().unwrap();
  let params = ps.get_parameters();
  assert_eq!(params.len(), 1);
  assert_eq!(sym_str(params[0].name), "Until");
  assert_eq!(params[0].extra.len(), 1);
}

#[test]
fn parse_parameters_long_prototypes_terminate() {
  setup();
  // The winnow grammar consumes >=1 char per parameter, so termination is
  // structural — the old MAX_STEPS=50 fatal cap is gone and long (if
  // pathological) prototypes now parse instead of erroring.
  let cs = T_CS!("\\foo");
  let proto = "{}".repeat(60);
  let ps = parse_parameters(&proto, &cs, false)
    .expect("long prototype parses")
    .expect("non-empty");
  assert_eq!(ps.get_parameters().len(), 60);
}

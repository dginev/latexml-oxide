#![crate_name = "rtx_macros"]
#![crate_type="dylib"]
#![feature(quote, plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;
extern crate regex;
// extern crate libxml;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rtx_core;

use syntax::codemap::Span;
use syntax::parse::token;
use syntax::ast;
use syntax::ast::TokenTree;
use syntax::ext::base::{ExtCtxt, MacResult, DummyResult, MacEager};
use syntax::ext::build::AstBuilder;  // trait for expr_usize
use syntax::print::pprust;
use syntax::fold::Folder;

use rustc_plugin::Registry;

use regex::{Captures, Regex};
use rtx_core::document::Document as Doc;
use rtx_core::state::State;
use rtx_core::tbox::TBox;
use std::collections::HashMap;
// use libxml::tree::Node;

// impl Constructor {
//   pub fn compile_replacement(&self) -> Option<ReplacementClosure> {
//     if self.replacement.is_empty() {
//       return None;
//     }

//     let cs = self.get_cs();
//     let name = NONW_RE.replace_all(&self.get_cs_name(), "");
//     let nargs = self.get_num_args();

//     let mut floats: Option<String> = None;
//     let replacement = FLOAT_RE.replace(&self.replacement, |caps: &Captures| {
//       floats = match caps.at(1) { // Grab float marker.
//         None => None,
//         Some(subs) => Some(subs.to_owned()),
//       };
//       String::new()
//     });

//     // println_stderr!("-- Preparing translation closure for: \n{:?}\n",
//     //                 replacement);
//     Some(Arc::new(|document, args, props, state| {
//       let mut savenode: Option<Node> = None;
//       TranslateConstructor!(replacement, floats, savenode);
//       match savenode {
//         None => {}
//         Some(savenode) => document.set_node(savenode),
//       };
//       return;
//     }))
//   }
// }

#[macro_export]
macro_rules! QNAME_STR(
  () => (r"((?:\p{Ll}|\p{Lu}|\p{Lo}|\p{Lt}|\p{Nl}|_|:)(?:\p{Ll}|\p{Lu}|\p{Lo}|\p{Lt}|\p{Nl}|_|:|\p{M}|\p{Lm}|\p{Nd}|\.|-)*)")
);

#[macro_export]
macro_rules! PI_STR(
  () => (concat!(r"^\s*<\?",QNAME_STR!()))
);

lazy_static! {
  static ref VALUE_RE : Regex = Regex::new(r"(\#|\&[\w\:]*\()").unwrap();
  static ref COND_RE : Regex = Regex::new(r"\?(\#|\&[\w\:]*\()").unwrap();
// Attempt to follow XML Spec, Appendix B
  static ref QNAME_RE : Regex = Regex::new(QNAME_STR!()).unwrap();
  static ref TEXT_RE : Regex = Regex::new(r"(.[^\#<\?\)\&\,]*)").unwrap();
  static ref NONW_RE : Regex = Regex::new(r"\W").unwrap();
  static ref FLOAT_RE : Regex = Regex::new(r"^(\^+)\s*").unwrap();
  static ref PI_RE : Regex = Regex::new(PI_STR!()).unwrap();
}


fn build_replacement(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
  if args.len() != 1 {
    cx.span_err(sp,
                &format!("argument should be a single identifier, but got {} arguments",
                         args.len()));
    return DummyResult::any(sp);
  }


  let replacement = match parse(cx, args) {
    Some(r) => r,
    // error is logged in 'parse' with cx.span_err
    None => return DummyResult::any(sp),
  };

  let mut replacement = &*replacement;
  let input_replacement = replacement;
  println_stderr!("replacement IN : {}", input_replacement);
  let mut floats: String = String::new();
  let mut has_floats: bool = false;
  FLOAT_RE.replace_all(replacement, |refs: &Captures| -> String {
    floats = refs.at(1).unwrap_or("").to_owned();
    has_floats = true;
    String::new()
  });

  let mut consumed = "";
  while !replacement.is_empty() {
    let mut is_match = false;
    // Processing instruction: <?name a=v ...?>
    PI_RE.replace(replacement, |refs: &Captures| -> String {
      let node_def = refs.at(1).unwrap_or("").to_owned();
      let (match_start, match_end) = refs.pos(1).unwrap();
      println_stderr!("-- PI between {:?} and {:?}", match_start, match_end);
      consumed = &replacement[0..match_end];
      String::new()
    });

    // Close tag: </name>
    if !is_match && replacement.starts_with("</") {
      println_stderr!("-- close tag");
      consumed = "</";
    }
    // Open tag: <name a=v ...> or .../> (for empty element)
    if !is_match && replacement.starts_with("<") {
      println_stderr!("-- open tag");
      consumed = "<";
    }
    // Substitutable value: argument, property...
    if !is_match && replacement.starts_with("#") {
      println_stderr!("-- argument hole");
      consumed = "#";
    }
    // Attribute: a=v; assigns in current node? [May conflict with random replacement!?!]
    if !is_match && replacement.find("=").is_some() {
      println_stderr!("-- Attribute");
      consumed = &replacement[0..1 + replacement.find("=").unwrap()];
    }
    // Else random text
    else {
      println_stderr!("-- random text");
      consumed = &replacement[0..1];
    }

    replacement = &replacement[consumed.len()..];
  }

  // Stub for now, just return a string
  let mock = quote_expr!(cx,
    |doc: &mut Doc, args: &Vec<TBox>, props: &HashMap<String, String>, state: &mut State| {
      println_stderr!("-- replacement mock executed.");
      if $has_floats {
        println_stderr!("-- has floats.") }
      else {
        println_stderr!("-- no floats.")
      }
    });
  MacEager::expr(cx.expr_some(sp, mock))

}

fn translate_avpairs(text: &str) {
  // # Parse a set of attribute value pairs from a constructor pattern,
  // # substituting argument and property values from the whatsit.
  // sub translate_avpairs {
  //   my @avs = ();
  //   s|^\s*||;
  //   while ($_) {
  //     if (/^$COND_RE/o) {
  //       my ($bool, $if, $else) = parse_conditional();
  //       my $code = "($bool ? (";
  //       { local $_ = $if; $code .= translate_avpairs(); }
  //       $code .= ") : (";
  //       { local $_ = $else; $code .= translate_avpairs() if $else; }
  //       $code .= "))";
  //       push(@avs, $code); }
  //     elsif (/^%$VALUE_RE/) {    # Hash?  Assume the value can be turned into a hash!
  //       s/^%//;                  # Eat the "%"
  //       push(@avs, '%{' . translate_value() . '}'); }
  //     elsif (s|^$QNAME_RE\s*=\s*||o) {
  //       my ($key, $value) = ($1, translate_string());
  //       push(@avs, "'$key'=>$value"); }    # if defined $value; }
  //     else { last; }
  //     s|^\s*||; }
  //   return join(', ', @avs); }
  println_stderr!("AV pairs: {:?}", text);
  return;
}

/// DG: Stolen from regex_macros, as I need a way to obtain a string literal
/// Looks for a single string literal and returns it.
/// Otherwise, logs an error with cx.span_err and returns None.
fn parse(cx: &mut ExtCtxt, tts: &[ast::TokenTree]) -> Option<String> {
  let mut parser = cx.new_parser_from_tts(tts);
  if let Ok(expr) = parser.parse_expr() {
    let entry = cx.expander().fold_expr(expr);
    let regex = match entry.node {
      ast::ExprKind::Lit(ref lit) => {
        match lit.node {
          ast::LitKind::Str(ref s, _) => s.to_string(),
          _ => {
            cx.span_err(entry.span,
                        &format!("expected string literal but got `{}`",
                                 pprust::lit_to_string(&**lit)));
            return None;
          }
        }
      }
      _ => {
        cx.span_err(entry.span,
                    &format!("expected string literal but got `{}`",
                             pprust::expr_to_string(&*entry)));
        return None;
      }
    };
    if !parser.eat(&token::Eof) {
      cx.span_err(parser.span, "only one string literal allowed");
      return None;
    }
    Some(regex)
  } else {
    cx.parse_sess().span_diagnostic.err("failure parsing token tree");
    None
  }
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_macro("build_replacement", build_replacement);
}

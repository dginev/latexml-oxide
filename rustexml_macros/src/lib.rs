#![crate_name = "rustexml_macros"]
#![crate_type="dylib"]
#![feature(quote, plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;
extern crate regex;

use syntax::codemap::Span;
use syntax::parse::token;
use syntax::ast;
use syntax::ast::TokenTree;
use syntax::parse::token::InternedString;
use syntax::ext::base::{ExtCtxt, MacResult, DummyResult, MacEager};
use syntax::ext::build::AstBuilder;  // trait for expr_usize
use syntax::print::pprust;
use syntax::fold::Folder;

use rustc_plugin::Registry;

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
  println!("replacement IN : {}", input_replacement);
  let mut consumed;
  while !replacement.is_empty() {
    // Processing instruction: <?name a=v ...?>
    if replacement.starts_with("<?") {
      println!("-- PI");
      consumed = "<?";
    }
    // Close tag: </name>
    else if replacement.starts_with("</") {
      println!("-- close tag");
      consumed = "</";
    }
    // Open tag: <name a=v ...> or .../> (for empty element)
    else if replacement.starts_with("<") {
      println!("-- open tag");
      consumed = "<";
    }
    // Substitutable value: argument, property...
    else if replacement.starts_with("#") {
      println!("-- argument hole");
      consumed = "#";
    }
    // Attribute: a=v; assigns in current node? [May conflict with random replacement!?!]
    else if replacement.find("=").is_some() {
      println!("-- Attribute");
      consumed = &replacement[0..1 + replacement.find("=").unwrap()];
    }
    // Else random text
    else {
      println!("-- random text");
      consumed = &replacement[0..1];
    }

    replacement = &replacement[consumed.len()..];
  }

  // Stub for now, just return a string
  MacEager::expr(cx.expr_str(sp, InternedString::new("stub")))
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

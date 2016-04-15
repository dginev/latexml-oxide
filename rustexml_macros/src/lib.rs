#![crate_name = "rustexml_macros"]
#![crate_type="dylib"]
#![feature(plugin_registrar, rustc_private)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;
extern crate regex;

use syntax::codemap::Span;
use syntax::parse::token;
use syntax::parse::token::Lit;
use syntax::ast::TokenTree;
use syntax::ext::base::{ExtCtxt, MacResult, DummyResult}; // MacEager
// use syntax::ext::build::AstBuilder;  // trait for expr_usize
use rustc_plugin::Registry;

fn expand_xml(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
  if args.len() != 1 {
    cx.span_err(sp,
                &format!("argument should be a single identifier, but got {} arguments",
                         args.len()));
    return DummyResult::any(sp);
  }

  let text = match args[0] {
    TokenTree::Token(_, token::Literal(Lit::Str_(s), _)) => s.as_str().to_string(),
    _ => {
      cx.span_err(sp, "argument should be a single identifier");
      return DummyResult::any(sp);
    }
  };

  let mut text = &*text;
  println!("TEXT IN : {}", text);
  let mut consumed;
  while !text.is_empty() {
    // Processing instruction: <?name a=v ...?>
    if text.starts_with("<?") {
      println!("-- PI");
      consumed = "<?";
    }
    // Close tag: </name>
    else if text.starts_with("</") {
      println!("-- close tag");
      consumed = "</";
    }
    // Open tag: <name a=v ...> or .../> (for empty element)
    else if text.starts_with("<") {
      println!("-- open tag");
      consumed = "<";
    }
    // Substitutable value: argument, property...
    else if text.starts_with("#") {
      println!("-- argument hole");
      consumed = "#";
    }
    // Attribute: a=v; assigns in current node? [May conflict with random text!?!]
    else if text.find("=").is_some() {
      println!("-- Attribute");
      consumed = &text[0..1 + text.find("=").unwrap()];
    }
    // Else random text
    else {
      println!("-- random text");
      consumed = &text[0..1];
    }

    text = &text[consumed.len()..];
  }
  return DummyResult::any(sp);
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
  reg.register_macro("xml", expand_xml);
}

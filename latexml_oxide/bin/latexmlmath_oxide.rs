#[macro_use]
extern crate latexml_core;
use libxml::tree::SaveOptions;
use std::env;
use std::process;

use latexml::util::test::{lex_single_tex_formula, new_test_engine};
use latexml_core::common::error::Result;
use latexml_core::state;
use latexml_math_parser::*;

fn main() -> Result<()> {
  if latexml_core::util::logger::init(log::LevelFilter::Info).is_err() {
    Error!(
      "latexml_oxide",
      "logger",
      "Failed to load logger, aborting. Please check latexml_core::util::logger installed correctly."
    );
  }
  let mut argv = env::args();
  argv.next();

  let source = match argv.next() {
    Some(s) => s,
    None => {
      Error!(
        "latexml_oxide",
        "",
        "Please provide a TeX formula on input! Exiting..."
      );
      process::exit(1);
    },
  };
  let mut core_engine = new_test_engine();
  let (lexemes, mut lex_nodes, xmath_opt, mut doc) =
    lex_single_tex_formula(&source, &mut core_engine);
  assert!(!lexemes.is_empty());
  eprintln!("\n\nlexemes: {lexemes:?}\n");

  state::set_nomathparse_flag(false); // nomathparse is "true" while lexing, but "false" while parsing
  let mut parser = MathParser::default();
  if let Ok(Some(parse_tree)) = parser.parse_lexemes(lexemes, &lex_nodes, &mut doc) {
    let mut xmath = xmath_opt.unwrap();
    for mut node in xmath.get_child_nodes() {
      node.unlink();
    }
    let xml_tree = parse_tree.into_xmath(&mut xmath, &mut lex_nodes, &mut doc)?;
    xmath
      .get_parent()
      .unwrap()
      .set_attribute("text", &text_form(&xml_tree, &doc))
      .unwrap();

    println!(
      "\n{}",
      doc.get_document().to_string_with_options(SaveOptions {
        format: true,
        ..SaveOptions::default()
      })
    );
  } else {
    Warn!("math", "parse", "Grammar did not recognize expression.");
    process::exit(1);
  }
  Ok(())
}

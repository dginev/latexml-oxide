#[macro_use]
extern crate rtx_core;
use libxml::tree::SaveOptions;
use std::env;
use std::process;

use rtx::util::test::{new_test_engine,lex_single_tex_formula};
use rtx_core::common::error::Result;
use rtx_math_parser::*;

fn main() -> Result<()> {
  if rtx_core::util::logger::init(log::LevelFilter::Info).is_err() {
    Error!(
      "rtx",
      "logger",
      None,
      "Failed to load logger, aborting. Please check rtx_core::util::logger installed correctly."
    );
  }
  let mut argv = env::args();
  argv.next();

  let source = match argv.next() {
    Some(s) => s,
    None => {
      Error!(
        "rtx",
        "",
        None,
        "Please provide a TeX formula on input! Exiting..."
      );
      process::exit(1);
    },
  };
  let mut core_engine = new_test_engine();
  let (lexemes, mut lex_nodes, xmath_opt, mut doc) = lex_single_tex_formula(&source, &mut core_engine);
  assert!(!lexemes.is_empty());
  eprintln!("\n\nlexemes: {lexemes:?}\n");

  let state = core_engine.get_state_mut();
  state.nomathparse = false; // nomathparse is "true" while lexing, but "false" while parsing
  let mut parser = MathParser::default();
  if let Ok(Some(parse_tree)) = parser.parse_lexemes(lexemes, &lex_nodes, &mut doc, state) {
    let mut xmath = xmath_opt.unwrap();
    for mut node in xmath.get_child_nodes() {
      node.unlink();
    }
    let xml_tree = parse_tree.into_xmath(&mut xmath, &mut lex_nodes, &mut doc, state)?;
    xmath
      .get_parent()
      .unwrap()
      .set_attribute("text", &text_form(&xml_tree, &mut doc, state))
      .unwrap();

    println!(
      "\n{}",
      doc.get_document().to_string_with_options(SaveOptions {
        format: true,
        ..SaveOptions::default()
      })
    );
  } else {
    Warn!(
      "math",
      "parse",
      None,
      "Grammar did not recognize expression."
    );
    process::exit(1);
  }
  Ok(())
}

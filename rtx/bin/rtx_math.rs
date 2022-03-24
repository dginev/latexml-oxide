#[macro_use]
extern crate rtx_core;
use libxml::tree::SaveOptions;
use rtx::util::test::lex_single_tex_formula;
use rtx_math_parser::*;
use std::env;
use std::process;

fn main() {
  if rtx_core::util::logger::init(log::LevelFilter::Info).is_err() {
    Error!(
      "rtx",
      "logger",
      None,
      None,
      "Failed to load logger, aborting early. Please check rtx_core::util::logger installed correctly."
    );
  }
  let mut argv = env::args();
  argv.next();

  let source = match argv.next() {
    Some(s) => s,
    None => {
      Error!("rtx", "", None, None, "Please provide a TeX formula on input! Exiting...");
      process::exit(1);
    },
  };

  let (lexemes, lex_nodes, xmath_opt, mut doc) = lex_single_tex_formula(&source);
  assert!(!lexemes.is_empty());
  eprintln!("lexemes: {:?}", lexemes);

  let mut parser = MathParser::default();
  if let Ok(Some(mut parse_tree)) = parser.parse_lexemes(lexemes, lex_nodes, &mut doc) {
    let mut xmath = xmath_opt.unwrap();
    for mut node in xmath.get_child_nodes() {
      node.unlink();
    }
    xmath.add_child(&mut parse_tree).unwrap();
    println!(
      "\n{}",
      doc.get_document().to_string_with_options(SaveOptions {
        format: true,
        ..SaveOptions::default()
      })
    );
  } else {
    Warn!("math", "parse", None, None, "Grammar did not recognize expression.");
    process::exit(1);
  }
}

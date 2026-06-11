#[macro_use]
extern crate latexml_core;
use libxml::tree::SaveOptions;
use std::env;
use std::process;

/// Use mimalloc to avoid glibc arena contention in multi-process workloads.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use latexml::util::preset::{lex_single_tex_formula, new_test_engine};
use latexml_core::common::error::Result;
use latexml_core::state;
use latexml_math_parser::*;

fn main() -> Result<()> {
  // 256 MB stack — see cortex_worker.rs for rationale (#17).
  std::thread::Builder::new()
    .stack_size(256 * 1024 * 1024)
    .spawn(|| real_main().map_err(|e| e.to_string()))
    .expect("spawn worker thread")
    .join()
    .expect("worker thread panicked")
    .map_err(|s| s.into())
}

fn real_main() -> Result<()> {
  let mut argv: Vec<String> = env::args().skip(1).collect();

  // Parse flags
  let pmml_flag = argv
    .iter()
    .any(|a| a == "--pmml" || a == "--presentationmathml");
  let cmml_flag = argv.iter().any(|a| a == "--cmml" || a == "--contentmathml");
  let quiet_flag = argv.iter().any(|a| a == "--quiet" || a == "-q");
  argv.retain(|a| {
    ![
      "--pmml",
      "--presentationmathml",
      "--cmml",
      "--contentmathml",
      "--quiet",
      "-q",
    ]
    .contains(&a.as_str())
  });

  let log_level = if quiet_flag {
    log::LevelFilter::Warn
  } else {
    log::LevelFilter::Info
  };
  latexml_core::util::logger::init(log_level).ok();

  let source = match argv.first() {
    Some(s) => s.clone(),
    None => {
      eprintln!("Usage: latexmlmath_oxide [--pmml] [--cmml] [--quiet] '<formula>'");
      process::exit(1);
    },
  };

  let mut core_engine = new_test_engine();
  let (lexemes, mut lex_nodes, xmath_opt, mut doc) =
    lex_single_tex_formula(&source, &mut core_engine);
  if lexemes.is_empty() {
    Error!("latexmlmath", "lex", "No lexemes produced from input");
    process::exit(1);
  }
  if !quiet_flag {
    eprintln!("lexemes: {lexemes:?}");
  }

  state::set_nomathparse_flag(false);
  let mut parser = MathParser::default();
  match parser.parse_lexemes(lexemes, &lex_nodes, &mut doc) { Ok(Some(parse_tree)) => {
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

    if pmml_flag || cmml_flag {
      // Post-process with MathML
      let xml_str = doc.get_document().to_string_with_options(SaveOptions {
        format: true,
        ..SaveOptions::default()
      });
      use latexml_post::document::{PostDocument, PostDocumentOptions};
      use latexml_post::processor::Processor;
      let post_doc = PostDocument::new_from_string(&xml_str, PostDocumentOptions::default())
        .expect("parse XML for MathML post-processing");
      let mut post = latexml_post::Post::new();
      let mut processors: Vec<Box<dyn Processor>> = Vec::new();
      if pmml_flag {
        processors.push(Box::new(latexml_post::mathml::MathML::new_presentation()));
      }
      if cmml_flag {
        processors.push(Box::new(latexml_post::mathml::MathML::new_content()));
      }
      match post.process_chain(vec![post_doc], &mut processors) {
        Ok(results) => println!("{}", results[0].to_xml_string()),
        Err(e) => {
          eprintln!("MathML post-processing failed: {}", e);
          process::exit(1);
        },
      }
    } else {
      // Raw XML output
      println!(
        "{}",
        doc.get_document().to_string_with_options(SaveOptions {
          format: true,
          ..SaveOptions::default()
        })
      );
    }
  } _ => {
    Warn!("math", "parse", "Grammar did not recognize expression.");
    process::exit(1);
  }}
  Ok(())
}

use glob::glob;
use libxml::parser::Parser;
use libxml::tree::Document as XmlDoc;
use libxml::tree::{Node, SaveOptions};
use std::path::PathBuf;
use std::sync::Arc;

use crate::core_interface::DigestionAPI;
use rtx_core::common::BindingDispatcher;
use rtx_core::document::Document;
use rtx_core::state::State;
use rtx_core::{s, Core, CoreOptions};
use rtx_math_parser::node_to_grammar_lexemes;
use rtx_package::package;

pub fn rtx_tests(
  dirpath: &str,
  requires: Option<&phf::Map<&str, &str>>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  rtx_tests_internal(dirpath, requires, dispatcher_opt)
}
pub fn rtx_tests_internal(
  dirpath: &str,
  requires: Option<&phf::Map<&str, &str>>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  rtx_core::util::logger::init(log::LevelFilter::Warn).unwrap();
  if !validate_requirements(dirpath, requires) {
    return; // test group only if required files are found.
  }
  for tex_file in glob(&s!("{}/*.tex", dirpath)).unwrap().flatten() {
    let name = tex_file.file_stem().unwrap().to_str().unwrap();
    let xml_file = tex_file.with_extension("xml");

    let tex_file_string = tex_file.to_str().unwrap();
    let xml_file_str = xml_file.to_str().unwrap();
    if xml_file.exists() {
      rtx_ok_internal(
        tex_file_string,
        xml_file_str,
        name,
        dispatcher_opt.clone(),
      );
    } else {
      // Skip, these could be tex fragment files.
    }
  }
}

pub fn rtx_test_single(tex_file_str:&str, name:&str, dirpath:&str, requires:Option<&phf::Map<&str, &str>>, dispatcher_opt: Option<BindingDispatcher>) {
  if !validate_requirements(dirpath, requires) {
    return; // test group only if required files are found.
  }
  let tex_file = PathBuf::from(tex_file_str);
  let xml_file = tex_file.with_extension("xml");
  if xml_file.exists() {
    rtx_ok_internal(
      tex_file_str,
      &xml_file.to_string_lossy(),
      name,
      dispatcher_opt,
    );
  } else {
    // Skip, these could be tex fragment files.
  }

}

fn validate_requirements(_dirpath: &str, _requires: Option<&phf::Map<&str, &str>>) -> bool {
  // TODO
  true
}

// fn rtx_ok(tex_path: &str, xml_path: &str, name: &str) { rtx_ok_internal(tex_path, xml_path, name,
// None) }

fn rtx_ok_internal(
  tex_path: &str,
  xml_path: &str,
  name: &str,
  extra_bindings_dispatcher: Option<BindingDispatcher>,
) {
  let tex_strings = process_texfile(tex_path, name, extra_bindings_dispatcher);
  if !tex_strings.is_empty() {
    let xml_strings = process_xmlfile(xml_path, name);
    if !xml_strings.is_empty() {
      for (lineno, (tex_line, xml_line)) in tex_strings.iter().zip(xml_strings.iter()).enumerate() {
        assert_eq!(
          tex_line, xml_line,
          "rtx result (left) differs from expected XML (right), file {xml_path}; line {lineno}"
        );
      }
      assert_eq!(
        tex_strings.len() - xml_strings.len(),
        0,
        "Conversion of {name:?} had more/fewer lines of content than expected"
      );
    }
  }
}

/// Returns the list-of-strings form of whatever was requested, if successful,
/// otherwise empty; and they will have reported the failure
fn process_texfile(
  tex_path: &str,
  name: &str,
  extra_bindings_dispatcher: Option<BindingDispatcher>,
) -> Vec<String> {
  // TODO: continue here...
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    search_paths: None,
    preload: None,
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  // Add the package bindings
  latexml.get_state_mut().bindings_dispatch = Some(Arc::new(package::dispatch));
  // If we want to test the rtx_contrib bindings, we need to pass in the additional binding
  // dispatcher, which makes the contrib bindings visible
  // this would have been equivalent to a latexml --path argument, except we require access to
  // compiled functions, hence the rust-native pass
  if extra_bindings_dispatcher.is_some() {
    latexml.get_state_mut().extra_bindings_dispatch = extra_bindings_dispatcher;
  }

  match latexml.convert_file(tex_path.to_owned()) {
    Err(e) => panic!("{:?}: Couldn't convert {:?}; {:?}", name, tex_path, e),
    Ok(doc) => process_ltx_doc(doc, name, latexml.get_state_mut()),
  }
}

/// Loads and serialized the resulting XML for a test file target,
/// returning it as a vector of line strings for the serialization
fn process_xmlfile<'a>(xml_path: &'a str, name: &'a str) -> Vec<String> {
  let parser = Parser::default();
  match parser.parse_file(xml_path) {
    Err(e) => panic!("Faield to parse XML file for {:?}: {:?}", name, e),
    Ok(dom) => process_dom(dom, name),
  }
}
fn process_ltx_doc(doc: Document, _name: &str, state: &mut State) -> Vec<String> {
  let doc_str = doc.serialize_to_string(state);
  // eprintln!("{doc_str}");
  doc_str.split('\n').map(ToString::to_string).collect()
}

/// Serializes and splits by line a given `XmlDoc`
fn process_dom(dom: XmlDoc, _name: &str) -> Vec<String> {
  dom
    .to_string_with_options(SaveOptions {
      format: true,
      ..SaveOptions::default()
    })
    .split('\n')
    .map(ToString::to_string)
    .collect()
}

/// Simple tokenization of a single formula, without any custom preloads
/// byond latex and amsmath
pub fn lex_single_tex_formula(tex: &str) -> (Vec<String>, Vec<Node>, Option<Node>, Document) {
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    search_paths: None,
    preload: Some(
      ["article.cls", "amsmath.sty"]
        .map(|x| x.to_string())
        .to_vec(),
    ),
    nomathparse: Some(true),
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  latexml.get_state_mut().bindings_dispatch = Some(Arc::new(package::dispatch));
  let xml_result = latexml.convert_file(format!("literal:\\[ {tex} \\]"));
  assert!(xml_result.is_ok(), "{:?}", xml_result.err());
  let doc = xml_result.unwrap();

  // grab the first formula
  let state = latexml.get_state_mut();
  match doc.findnode("//*[local-name()='XMath']", None, state) {
    Some(math) => {
      let mut idx = 0;
      let (lexemes, nodes) = node_to_grammar_lexemes(&math, &mut idx);
      (lexemes, nodes, Some(math), doc)
    },
    None => (Vec::new(), Vec::new(), None, doc),
  }
}

/// Build a test function for each "*.tex" source found in a given directory path.
/// The path should be absolute, or relative to the root rtx checkout.
#[macro_export]
macro_rules! tex_tests {
    ($dir:literal, $requires:expr, $dispatch:expr) => {
      macro_rules! this_test_requires {
        () => {$requires}
      }
      macro_rules! this_test_dispatch {
        () => {$dispatch};
      }
      use rtx_codegen::GlobTeXTests;
      #[derive(GlobTeXTests)]
      #[directory=$dir]
      struct _TestDirective;
    };
}
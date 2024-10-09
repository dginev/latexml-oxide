use glob::glob;
use libxml::parser::Parser;
use libxml::tree::Document as XmlDoc;
use libxml::tree::{Node, SaveOptions};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Once;

use crate::core_interface::DigestionAPI;
use latexml_core::common::BindingDispatcher;
use latexml_core::document::Document;
use latexml_core::{s, Core, CoreOptions, state};
use latexml_math_parser::node_to_grammar_lexemes;
use latexml_codegen::LoadModel;

pub fn latexml_tests(
  dirpath: &str,
  requires: Option<&phf::Map<&str, &str>>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  latexml_tests_internal(dirpath, requires, dispatcher_opt)
}
pub fn latexml_tests_internal(
  dirpath: &str,
  requires: Option<&phf::Map<&str, &str>>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  if !validate_requirements(dirpath, requires) {
    return; // test group only if required files are found.
  }
  for tex_file in glob(&s!("{}/*.tex", dirpath)).unwrap().flatten() {
    let name = tex_file.file_stem().unwrap().to_str().unwrap();
    let xml_file = tex_file.with_extension("xml");

    let tex_file_string = tex_file.to_str().unwrap();
    let xml_file_str = xml_file.to_str().unwrap();
    if xml_file.exists() {
      latexml_ok_internal(tex_file_string, xml_file_str, name, dispatcher_opt.clone());
    } else {
      // Skip, these could be tex fragment files.
    }
  }
}

static INIT_LOGGER: Once = Once::new();
pub fn init_logger() {
  INIT_LOGGER.call_once(|| {
    latexml_core::util::logger::init(log::LevelFilter::Warn).unwrap();
    // Initializing the libxml parser ONCE is a recommendation for thread-safety,
    // which should hopefully avoid any hangs in a threaded "cargo test"
    // this may also be needed in web servers running parallel rtx conversion jobs
    // See: https://dev.w3.org/XInclude-Test-Suite/libxml2-2.4.24/libxml2-2.4.24/doc/threads.html
    unsafe {
      libxml::bindings::xmlInitParser();
    }
  });
}

pub fn latexml_test_single(
  tex_file_str: &str,
  name: &str,
  dirpath: &str,
  requires: Option<&phf::Map<&str, &str>>,
  dispatcher_opt: Option<BindingDispatcher>,
) {
  init_logger();
  if !validate_requirements(dirpath, requires) {
    return; // test group only if required files are found.
  }
  let tex_file = PathBuf::from(tex_file_str);
  let xml_file = tex_file.with_extension("xml");
  if matches!(xml_file.try_exists(), Ok(true)) {
    latexml_ok_internal(
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

// fn latexml_ok(tex_path: &str, xml_path: &str, name: &str) { latexml_ok_internal(tex_path, xml_path, name,
// None) }

fn latexml_ok_internal(
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
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  // If we want to test the latexml_contrib bindings, we need to pass in the additional binding
  // dispatcher, which makes the contrib bindings visible
  // this would have been equivalent to a latexml --path argument, except we require access to
  // compiled functions, hence the rust-native pass
  if let Some(dispatcher) = extra_bindings_dispatcher {
    state::set_extra_bindings_dispatch(dispatcher);
  }
  match latexml.convert_file(tex_path.to_owned()) {
    Err(e) => panic!("{:?}: Couldn't convert {:?}; {:?}", name, tex_path, e),
    Ok(doc) => process_ltx_doc(doc, name),
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
fn process_ltx_doc(doc: Document, _name: &str) -> Vec<String> {
  let doc_str = doc.serialize_to_string();
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

/// Provide a default test `Core` engine for simple operations
pub fn new_test_engine() -> Core {
  let core_engine = Core::new(CoreOptions {
    preload: Some(
      ["article.cls", "amsmath.sty"]
        .map(|x| x.to_string())
        .to_vec(),
    ),
    verbosity: Some(-2),
    search_paths: None,
    nomathparse: Some(true),
    include_comments: Some(false),
    ..CoreOptions::default()
  });
  load_model!("LaTeXML");
  state::set_bindings_dispatch(Rc::new(latexml_package::dispatch));
  core_engine
}

/// Simple tokenization of a single formula, without any custom preloads
/// beyond latex and amsmath
pub fn lex_single_tex_formula(tex: &str, latexml: &mut Core) -> (Vec<String>, Vec<Node>, Option<Node>, Document) {
  let xml_result = latexml.convert_file(format!("literal:\\[ {tex} \\]"));
  assert!(xml_result.is_ok(), "{:?}", xml_result.err());
  let mut doc = xml_result.unwrap();

  // grab the first formula
  match doc.findnode("//*[local-name()='XMath']", None) {
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
  ($dir:literal) => {
    tex_tests!($dir, None, None);
  };
  ($dir:literal, $requires:expr, $dispatch:expr) => {
    macro_rules! this_test_requires {
      () => {
        $requires
      };
    }
    macro_rules! this_test_dispatch {
      () => {
        $dispatch
      };
    }
    use latexml_codegen::GlobTeXTests;
    #[derive(GlobTeXTests)]
    #[directory=$dir]
    struct _TestDirective;
  };
}

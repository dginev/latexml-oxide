extern crate rtx_core;

use glob::glob;
use libxml::parser::Parser;
use std::collections::HashMap;

use libxml::tree::Document as XmlDoc;
use rtx_core::document::Document;
use rtx_core::state::State;
use rtx_core::{Core, CoreOptions};

use crate::core::DigestionAPI;

pub fn rtx_tests(dirpath: &str, requires: Option<HashMap<&str, &str>>) {
  assert!(rtx_core::util::logger::init(log::LevelFilter::Info).is_ok());

  if !validate_requirements(dirpath, requires) {
    return; // test group only if required files are found.
  }
  for tex_file in glob(&s!("{}/*.tex", dirpath)).unwrap() {
    if let Ok(tex_file) = tex_file {
      let name = tex_file.file_stem().unwrap().to_str().unwrap();
      let xml_file = tex_file.with_extension("xml");

      let tex_file_string = tex_file.to_str().unwrap();
      let xml_file_str = xml_file.to_str().unwrap();
      if xml_file.exists() {
        rtx_ok(tex_file_string, xml_file_str, name);
      } else {
        // Skip, these could be tex fragment files.
      }
    }
  }
}

fn validate_requirements(_dirpath: &str, _requires: Option<HashMap<&str, &str>>) -> bool {
  // TODO
  true
}

fn rtx_ok(tex_path: &str, xml_path: &str, name: &str) {
  let tex_strings = process_texfile(tex_path, name);
  if !tex_strings.is_empty() {
    let xml_strings = process_xmlfile(xml_path, name);
    if !xml_strings.is_empty() {
      for (tex_line, xml_line) in tex_strings.iter().zip(xml_strings.iter()) {
        assert_eq!(tex_line, xml_line, "rtx result (left) differs from expected XML (right)");
      }
      assert_eq!(
        tex_strings.len() - xml_strings.len(),
        0,
        "Conversion of {:?} had more/fewer lines of content than expected",
        name
      );
    }
  }
}

/// Returns the list-of-strings form of whatever was requested, if successful,
/// otherwise empty; and they will have reported the failure
fn process_texfile(tex_path: &str, name: &str) -> Vec<String> {
  // TODO: continue here...
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    search_paths: None,
    preload: None,
    include_comments: Some(false),
    ..CoreOptions::default()
  });

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
  doc.to_string(state).split('\n').map(|line| line.to_string()).collect()
}

/// Serializes and splits by line a given `XmlDoc`
fn process_dom(dom: XmlDoc, _name: &str) -> Vec<String> { dom.to_string(true).split('\n').map(|line| line.to_string()).collect() }

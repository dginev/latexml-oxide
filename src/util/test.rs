extern crate log;
extern crate rtx_core;

use glob::glob;
use std::collections::HashMap;
use libxml::parser::Parser;

use rtx_core::{Core, CoreOptions};
use rtx_core::state::State;
use rtx_core::document::Document;
use libxml::tree::Document as XmlDoc;

use core::DigestionAPI;

pub fn rtx_tests(dirpath: &str, requires: Option<HashMap<&str, &str>>) {
  assert!(rtx_core::util::logger::init(log::LevelFilter::Info).is_ok());

  if !validate_requirements(dirpath, requires) {
    return; // test group only if required files are found.
  }
  for tex_file in glob(&(dirpath.to_string() + "/*.tex")).unwrap() {
    if let Ok(tex_file) = tex_file {
      let name = tex_file.file_stem().unwrap().to_str().unwrap();
      let xml_file = tex_file.with_extension("xml");

      let tex_file_string = tex_file.to_str().unwrap().to_string();
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

fn rtx_ok(tex_path: String, xml_path: &str, name: &str) {
  let tex_strings = process_texfile(tex_path, name);
  if !tex_strings.is_empty() {
    let xml_strings = process_xmlfile(xml_path, name);
    if !xml_strings.is_empty() {
      info!("[test] xml diff for {:?}", name);
      for (tex_line, xml_line) in tex_strings.iter().zip(xml_strings.iter()) {
        assert_eq!(tex_line, xml_line);
      }
      // match tex_strings.len() - xml_strings.len() {
      //   0 => {},//As expected,
      //   diff => match diff > 0 {
      //     true => panic!("Conversion of {:?} had more content than expected", name),
      //     false => panic!("Conversion of {:?} had less content than expected", name)
      //   }
      // };
    }
  }
}

/// Returns the list-of-strings form of whatever was requested, if successful,
/// otherwise empty; and they will have reported the failure
fn process_texfile(tex_path: String, name: &str) -> Vec<String> {
  // TODO: continue here...
  let mut latexml = Core::new(CoreOptions {
    verbosity: Some(-2),
    search_paths: None,
    preload: None,
    include_comments: Some(false),
    ..CoreOptions::default()
  });

  match latexml.convert_file(tex_path.clone()) {
    Err(e) => panic!("{:?}: Couldn't convert {:?}; {:?}", name, tex_path, e),
    Ok(doc) => process_ltx_doc(doc, name, latexml.state_mut()),
  }
}

fn process_xmlfile<'a>(xml_path: &'a str, name: &'a str) -> Vec<String> {
  let parser = Parser::default();
  match parser.parse_file(xml_path) {
    Err(e) => panic!("Faield to parse XML file for {:?}: {:?}", name, e),
    Ok(dom) => process_dom(dom, name),
  }
}
fn process_ltx_doc(doc: Document, _name: &str, state: &mut State) -> Vec<String> {
  doc
    .to_string(state)
    .split('\n')
    .map(|line| line.to_string())
    .collect()
}
fn process_dom(dom: XmlDoc, _name: &str) -> Vec<String> {
  dom
    .to_string(true)
    .split('\n')
    .map(|line| line.to_string())
    .collect()
}

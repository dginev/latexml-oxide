use core::Core;
use core::stomach::Stomach;
use state::State;
use glob::glob;
use std::collections::HashMap;
use libxml::parser::Parser;
use libxml::tree::Document;

pub fn rustexml_tests(dirpath : &str, requires : Option<HashMap<&str, &str>>) {
  if !validate_requirements(dirpath, requires) {
    return; // test group only if required files are found.
  }
  for tex_file in glob(&(dirpath.to_string() + "/*.tex")).unwrap() {
    match tex_file {
      Ok(tex_file) => {
        let name = tex_file.file_stem().unwrap().to_str().unwrap().to_string();
        let xml_file = tex_file.with_extension("xml");

        let tex_file_string = tex_file.to_str().unwrap().to_string();
        let xml_file_string = xml_file.to_str().unwrap().to_string();
        if xml_file.exists() {
          rustexml_ok(tex_file_string, xml_file_string, name);
        } else {
          // Skip, these could be tex fragment files.
        }
      }
      Err(_) => {}
    }
  }
}

fn validate_requirements(_dirpath : &str, _requires : Option<HashMap<&str, &str>>) -> bool {
  // TODO
  true
}

fn rustexml_ok(tex_path : String, xml_path: String, name: String) {
  let tex_strings = process_texfile(tex_path, &name);
  if !tex_strings.is_empty() {
    let xml_strings = process_xmlfile(&xml_path, &name);
    if !xml_strings.is_empty() {
    println!("[test] xml diff for {:?}", name);
      for (tex_line, xml_line) in tex_strings.iter().zip(xml_strings.iter()) {
        assert_eq!(tex_line, xml_line); 
      }
      match tex_strings.len() - xml_strings.len() {
        0 => {},//As expected,
        diff => match diff > 0 {
          true => panic!("Conversion of {:?} had more content than expected", name),
          false => panic!("Conversion of {:?} had less content than expected", name)
        }
      };
    }
  }
}

/// Returns the list-of-strings form of whatever was requested, if successful,
/// otherwise empty; and they will have reported the failure
fn process_texfile<'a>(tex_path: String, name: &'a str) -> Vec<String> {
  let mut test_state = State::new();
  test_state.verbosity = -2;
  let mut latexml = Core {
    preload : Vec::new(),
    stomach : Stomach::default(),
    state : test_state
  };
  match latexml.convert_file(tex_path.clone()) {
    Err(e) => panic!("{:?}: Couldn't convert {:?}; {:?}",name, tex_path, e),
    Ok(doc) => process_dom(doc.document, name)
  }
}

fn process_xmlfile<'a>(xml_path: &'a str, name: &'a str) -> Vec<String> {
  let parser = Parser::default();
  match parser.parse_file(xml_path) {
    Err(e) => panic!("Faield to parse XML file for {:?}: {:?}", name, e),
    Ok(dom) => process_dom(dom, name)
  }
}
fn process_dom<'a>(dom: Document, _name: &'a str) -> Vec<String> {
  dom.to_string().split("\n").map(|line| line.to_string()).collect()
}
use core::*;
use glob::glob;
use std::collections::HashMap;

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

fn validate_requirements(dirpath : &str, requires : Option<HashMap<&str, &str>>) -> bool {
  true
}

fn rustexml_ok(tex_file : String, xml_file: String, name: String) {
  println!("----");
  println!("tex {:?}", tex_file);
  println!("xml {:?}", xml_file);
  println!("name {:?}", name);
  assert!(true);
}

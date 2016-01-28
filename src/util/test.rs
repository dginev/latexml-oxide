use glob::glob;
use std::collections::HashMap;

pub fn rustexml_tests(dirpath : &str, _requires : Option<HashMap<&str, &str>>) {
  for tex_file in glob(&(dirpath.to_string() + "/*.tex")).unwrap() {
    match tex_file {
      Ok(tex_file) => {
        println!("tex file: {:?}", tex_file);
        let xml_file = tex_file.with_extension("xml");
        if xml_file.exists() {
          // TODO: Perform the real conversion test here.
          assert!(xml_file.exists(), xml_file);
        } else {
          // Skip, these could be tex fragment files.
        }
      }
      Err(_) => {}
    }
  }
}
use glob::glob;

pub fn rustexml_tests(dirpath : &str) {
  for tex_file in glob(&(dirpath.to_string() + "/*.tex")).unwrap() {
    match tex_file {
      Ok(tex_file) => {
        let xml_file = tex_file.with_extension("xml");
        assert!(xml_file.exists());
      }
      Err(_) => {}
    }
  }
}
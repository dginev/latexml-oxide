extern crate rtx_core;
extern crate rtx;

use rtx::converter::Converter;
use rtx_core::common::{Config, OutputFormat};
#[test]
fn can_convert_hello() {
  let hello_source = "tests/hello/hello.tex";
  // let hello_expected = "tests/hello.html";
  let html_config = Config { format: OutputFormat::HTML5, ..Config::new() };
  let converter = Converter::from_config(html_config);
  let conversion_result = converter.convert(hello_source.to_string());
  assert!(conversion_result.is_ok());
  let response = conversion_result.unwrap();
  assert!(response.log.len() > 0);
  println!("Log: \n{:?}", response.log);
  assert!(response.result.is_some());
  println!("Result: \n{:?}", response.result);
  assert!(response.status_code == 0);
}

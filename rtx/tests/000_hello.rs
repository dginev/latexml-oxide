#![feature(macro_literal_matcher)]
extern crate rtx;
extern crate rtx_core;

use rtx::converter::Converter;
use rtx_core::common::{Config, OutputFormat};
#[test]
fn can_convert_hello() {
  let hello_source = "tests/hello/hello.tex";
  // let hello_expected = "tests/hello.html";
  let html_config = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let converter = Converter::from_config(html_config);
  let conversion_result = converter.convert(hello_source.to_string());
  assert!(conversion_result.result.is_some());
  let response = conversion_result;
  assert!(!response.log.is_empty());
  assert!(response.result.is_some());
  assert!(response.status_code == 0);
}

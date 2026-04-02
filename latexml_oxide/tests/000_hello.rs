use latexml::converter::Converter;
use latexml_core::common::{Config, OutputFormat};

#[test]
fn can_convert_hello() {
  assert!(latexml_core::util::logger::init(log::LevelFilter::Warn).is_ok());
  let hello_source = "tests/hello/hello.tex";
  let html_config = Config {
    format: OutputFormat::HTML5,
    ..Config::default()
  };
  let mut converter = Converter::from_config(html_config);
  converter.initialize_session().expect("can initialize.");

  let conversion_result = converter.convert(hello_source.to_string());
  assert!(conversion_result.result.is_some());
  let response = conversion_result;
  assert!(response.result.is_some());
  assert!(response.status_code == 0);
  assert_eq!(response.status, "No obvious problems");
}

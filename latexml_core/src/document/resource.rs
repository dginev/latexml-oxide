//======================================================================
// Support for requiring "Resources", ie CSS, Javascript, whatever
#[derive(Debug, Clone, Default)]
pub struct Resource {
  pub name:     String,
  pub media:    String,
  pub mimetype: String,
  pub content:  String,
}

pub fn resource_type(abbrev: &str) -> String {
  match abbrev {
    "css" => "text/css",
    "js" => "text/javascript",
    _ => "",
  }
  .to_string()
}

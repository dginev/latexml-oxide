//======================================================================
// Support for requiring "Resources", ie CSS, Javascript, whatever
#[derive(Debug, Clone)]
pub struct Resource {
  pub name: String,
  pub media: String,
  pub mimetype: String,
  pub content: String,
}

impl Default for Resource {
  fn default() -> Self {
    Resource {
      name: String::new(),
      media: String::new(),
      mimetype: String::new(),
      content: String::new()
    }
  }
}

pub fn resource_type(abbrev: &str) -> String {
  match abbrev {
    "css" => "text/css",
    "js" => "text/javascript",
    _ => ""
  }.to_string()
}

use crate::state::Scope;

// ======================================================================
// Defining Rewrite rules that act on the DOM
// These are applied after the document is completely constructed
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RewriteOptions {
  pub label: Option<String>,
  pub scope: Option<Scope>,
  pub xpath: Option<String>,
  pub xmatch: Option<String>,  
  pub attributes: Option<String>,
  pub replace: Option<String>,
  pub regexp: Option<String>,
  pub select: Option<String>
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rewrite {
  options: RewriteOptions
}


impl Rewrite {
 pub fn new(kind:&str, options: RewriteOptions) -> Self {
   Rewrite { options }
 }
}

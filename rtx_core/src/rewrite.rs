use crate::state::{State,Scope};
use crate::document::Document;
use crate::common::error::Result;
use libxml::tree::Node;
use std::rc::Rc;
use std::fmt;

pub type RewriteReplaceClosure = Rc<dyn Fn(&mut Document, &mut Node, &mut State) -> Result<()>>;

// ======================================================================
// Defining Rewrite rules that act on the DOM
// These are applied after the document is completely constructed
#[derive(Clone, Default)]
pub struct RewriteOptions {
  pub label: Option<String>,
  pub scope: Option<Scope>,
  pub xpath: Option<String>,
  pub xmatch: Option<String>,
  pub attributes: Option<String>,
  pub replace: Option<RewriteReplaceClosure>,
  pub regexp: Option<String>,
  pub select: Option<String>,
}
impl fmt::Debug for RewriteOptions {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "<RewriteOptions>") }
}
impl PartialEq for RewriteOptions {
  fn eq(&self, other: &RewriteOptions) -> bool { self.select == other.select }
}


#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rewrite {
  options: RewriteOptions
}


impl Rewrite {
 pub fn new(kind:&str, options: RewriteOptions) -> Self {
   Rewrite { options }
 }
}

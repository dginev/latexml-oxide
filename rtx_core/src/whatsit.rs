use std::fmt;
use std::collections::HashMap;
use std::sync::Arc;
use state::State;
use {Digested, BoxOps};
use definition::Definition;
use document::Document;

pub struct Whatsit {
  pub args: Vec<Option<Digested>>,
  pub properties: HashMap<String, String>, // TODO: This will be an issue, LaTeXML traditionally takes advantage of the fully untyped nature of Perl hashes
  pub definition: Arc<Definition>,
}

impl Whatsit {
  pub fn get_arg(&self, n: usize) -> Option<&Digested> {
    match self.args.get(n - 1) {
      None => None,
      Some(&None) => None,
      Some(&Some(ref opt)) => Some(&opt)
    }
  }

  pub fn get_args(&self) -> &Vec<Option<Digested>> {
    &self.args
  }

  pub fn get_properties(&self) -> &HashMap<String, String> {
    &self.properties
  }
}

impl fmt::Debug for Whatsit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,
           "Whatsit {{ args: {:?}, properties: {:?} }}",
           self.args,
           self.properties)
  }
}

impl BoxOps for Whatsit {
  fn unlist(self) -> Vec<Digested> {
    Vec::new()
  }

  fn be_absorbed(&mut self, document: &mut Document, state: &mut State) {
    // Significant time is consumed here, and associated with a specific CS,
    // so we should be profiling as well!
    // Hopefully the csname is the same that was charged in the digestioned phase!

    // my $profiled = $STATE->lookupValue('PROFILING') && $defn->getCS;
    // LaTeXML::Definition::startProfiling($profiled, 'absorb') if $profiled;
    let result = self.definition.do_absorbtion(document, self, state);
    // LaTeXML::Definition::stopProfiling($profiled, 'absorb') if $profiled;
    return result;
  }
}

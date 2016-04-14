use std::collections::HashMap;
use state::State;
use core::tbox::TBox;
use core::definition::Definition;
use core::document::Document;

pub struct Whatsit {
  args: Vec<TBox>,
  properties: HashMap<String, String>, // TODO: This will be an issue, LaTeXML traditionally takes advantage of the fully untyped nature of Perl hashes
  definition: Box<Definition>,
}

impl Whatsit {
  pub fn get_arg(&self, n: usize) -> Option<&TBox> {
    self.args.get(n - 1)
  }

  pub fn get_args(&self) -> &Vec<TBox> {
    &self.args
  }

  pub fn get_properties(&self) -> &HashMap<String, String> {
    &self.properties
  }

  pub fn be_absorbed(&mut self, document: &mut Document, state: &mut State) {
    // Significant time is consumed here, and associated with a specific CS,
    // so we should be profiling as well!
    // Hopefully the csname is the same that was charged in the digestioned phase!

    // my $profiled = $STATE->lookupValue('PROFILING') && $defn->getCS;
    // LaTeXML::Core::Definition::startProfiling($profiled, 'absorb') if $profiled;
    let result = self.definition.do_absorbtion(document, self, state);
    // LaTeXML::Core::Definition::stopProfiling($profiled, 'absorb') if $profiled;
    return result;
  }
}

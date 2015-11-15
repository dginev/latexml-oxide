use core::token::Token;

#[derive(Clone)]
pub struct Parameter {
  pub name : String,
  pub spec : String,
  pub extra : Vec<Option<Parameters>>
}

#[derive(Clone)]
pub struct Parameters {
  pub params : Vec<Parameter>
}

impl Parameters {
  pub fn get_num_args(&self) -> usize {
    self.params.len()
  }

  pub fn revert_arguments(&self, args : Vec<Token>) -> Vec<Token> {
    println_stderr!("--- Someone called revert_arguments!!! ");
    Vec::new()
  }
}
#[derive(Clone)]
pub struct Parameter {
  stuff : String
}

#[derive(Clone)]
pub struct Parameters {
  params : Vec<Parameter>
}

impl Parameters {
  pub fn get_num_args(&self) -> usize {
    self.params.len()
  }
}
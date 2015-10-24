use core::stomach::{Stomach};
pub struct State {
  pub verbosity : i32,
  pub map : Vec<String>,
  pub status_code : usize,
  pub stomach : Stomach
}

impl State {
  pub fn get_stomach(&mut self) -> &mut Stomach {
    &mut self.stomach
  }
}
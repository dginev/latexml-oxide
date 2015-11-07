use core::tbox::TBox;

pub struct Whatsit {
  args : Vec<TBox>
}

impl Whatsit {
  pub fn get_arg(&self, n : usize) -> Option<&TBox> {
    self.args.get(n - 1)
  }
}
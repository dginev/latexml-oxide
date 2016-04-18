pub fn note_end(note: &str) {
  println_stderr!("--|End:  | {:?}", note);
}

pub fn note_begin(note: &str) {
  println_stderr!("--|Begin:| {:?}", note);
}

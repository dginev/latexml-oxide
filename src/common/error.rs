pub fn note_end(note : String) {
  println_stderr!("--|End:  | {:?}", note);
}

pub fn note_begin(note : String) {
  println_stderr!("--|Begin:| {:?}", note);
}
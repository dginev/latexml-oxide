use std::collections::VecDeque;

use rtx_core::common::store::Stored;
use rtx_core::definition::register::NumericOps;
use rtx_core::state::State;

pub fn reenter_text_mode(vertical_mode: bool, state: &mut State) {
  BindState!(state);
  let bindings_val = if vertical_mode {
    LookupValue!("VTEXT_MODE_BINDINGS")
  } else {
    LookupValue!("HTEXT_MODE_BINDINGS")
  };

  let mut bindings: VecDeque<Stored> = match bindings_val {
    Some(Stored::VecDequeStored(ref vdq)) => vdq.clone(),
    _ => VecDeque::new(),
  };
  if let Some(Stored::VecDequeStored(ref text_mode_bindings)) = LookupValue!("TEXT_MODE_BINDINGS") {
    bindings.extend(text_mode_bindings.clone());
  }
  for binding in bindings {
    if let Stored::VecToken(vt) = binding {
      LetI!(&vt[0], vt[1].clone());
    }
  }
  return;
}

pub fn only_preamble(cs: &str, state: &mut State) {
  if !state.lookup_bool("inPreamble") {
    let category_object = s!("unexpected:{}", cs);
    error!(target: &category_object, "The current command can only appear in the preamble");
  }
}

pub fn today(state: &State) -> String {
  let month_names = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];
  let month = month_names[state.lookup_register("\\month", vec![]).unwrap().value_of() as usize - 1];
  let day = state.lookup_register("\\day", vec![]).unwrap().value_of();
  let year = state.lookup_register("\\year", vec![]).unwrap().value_of();
  s!("{} {}, {}", month, day, year)
}

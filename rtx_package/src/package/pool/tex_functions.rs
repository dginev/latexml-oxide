use std::collections::VecDeque;

use rtx_core::common::store::Stored;
use rtx_core::state::State;

pub fn reenter_text_mode(vertical_mode: bool, state: &mut State) {
  SetupBindingMacros!(state);
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

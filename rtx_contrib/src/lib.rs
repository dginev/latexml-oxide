#[macro_use]
extern crate rtx_codegen;
use rtx_core::common::error::*;

// =======================
// Adding custom bindings:
// =======================
//
// I. Add your custom binding definition as a module delcaration here
pub mod mytemplate_sty;
pub mod scopemacro_sty;

pub fn dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    // II. Connect the filename to the `load_definitions` function of your .rs binding:
    "mytemplate.sty" => Some(mytemplate_sty::load_definitions()),
    "scopemacro.sty" => Some(scopemacro_sty::load_definitions()),
    _ => None,
  }
}

// III. That's all! Run "cargo test" in the rtx/ root and your binding will be compiled and made
// visible to the main rtx executable

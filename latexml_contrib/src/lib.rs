#[macro_use]
extern crate latexml_package;
#[macro_use]
extern crate latexml_codegen;
use latexml_core::common::error::*;

// =======================
// Adding custom bindings:
// =======================
//
// I. Add your custom binding definition as a module delcaration here
pub mod mytemplate_sty;
pub mod scopemacro_tex;
pub mod xkvdop1_sty;
pub mod xkvdop2_sty;
pub mod xkvdop3_sty;
pub mod xkvdop4_sty;
pub mod xkvdop5_cls;
pub mod xkvdop6_cls;

pub fn dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    // II. Connect the filename to the `load_definitions` function of your .rs binding:
    "mytemplate.sty" => Some(mytemplate_sty::load_definitions()),
    // Document-level binding: loaded by load_external_binding("scopemacro")
    // when processing scopemacro.tex — mirrors scopemacro.latexml in Perl
    "scopemacro" => Some(scopemacro_tex::load_definitions()),
    // xkeyval test packages — passthrough to raw TeX (noltxml)
    "xkvdop1.sty" => Some(xkvdop1_sty::load_definitions()),
    "xkvdop2.sty" => Some(xkvdop2_sty::load_definitions()),
    "xkvdop3.sty" => Some(xkvdop3_sty::load_definitions()),
    "xkvdop4.sty" => Some(xkvdop4_sty::load_definitions()),
    "xkvdop5.cls" => Some(xkvdop5_cls::load_definitions()),
    "xkvdop6.cls" => Some(xkvdop6_cls::load_definitions()),
    _ => None,
  }
}

// III. That's all! Run "cargo test" in the latexml_oxide/ root and your binding will be compiled
// and made visible to the main latexml-oxide executable

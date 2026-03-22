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
pub mod mykeyval_sty;
pub mod mytemplate_sty;
pub mod myxkeyval_sty;
pub mod scopemacro_tex;
pub mod simplemath_src;
pub mod xkvdop1_sty;
pub mod xkvdop2_sty;
pub mod xkvdop3_sty;
pub mod xkvdop4_sty;
pub mod xkvdop5_cls;
pub mod xkvdop6_cls;
pub mod xkvview_sty;
pub mod apackage_sty;
pub mod filelistclass_cls;
pub mod myclass_cls;

pub fn dispatch(filename: &str) -> Option<Result<()>> {
  match filename {
    // II. Connect the filename to the `load_definitions` function of your .rs binding:
    "apackage.sty" => Some(apackage_sty::load_definitions()),
    "filelistclass.cls" => Some(filelistclass_cls::load_definitions()),
    "myclass.cls" => Some(myclass_cls::load_definitions()),
    "mykeyval.sty" => Some(mykeyval_sty::load_definitions()),
    "mytemplate.sty" => Some(mytemplate_sty::load_definitions()),
    "myxkeyval.sty" => Some(myxkeyval_sty::load_definitions()),
    // Document-level binding: loaded by load_external_binding("scopemacro")
    // when processing scopemacro.tex — mirrors scopemacro.latexml in Perl
    "scopemacro" => Some(scopemacro_tex::load_definitions()),
    // Source-level bindings: *.src files mirror Perl's *.latexml mechanism.
    // In the .tex file, add \input{name.src} to load the binding.
    // The dispatcher routes "name.src" to name_src::load_definitions().
    "simplemath.src" | "simplemath" => Some(simplemath_src::load_definitions()),
    // xkeyval test packages — passthrough to raw TeX (noltxml)
    "xkvdop1.sty" => Some(xkvdop1_sty::load_definitions()),
    "xkvdop2.sty" => Some(xkvdop2_sty::load_definitions()),
    "xkvdop3.sty" => Some(xkvdop3_sty::load_definitions()),
    "xkvdop4.sty" => Some(xkvdop4_sty::load_definitions()),
    "xkvdop5.cls" => Some(xkvdop5_cls::load_definitions()),
    "xkvdop6.cls" => Some(xkvdop6_cls::load_definitions()),
    "xkvview.sty" => Some(xkvview_sty::load_definitions()),
    _ => None,
  }
}

// III. That's all! Run "cargo test" in the latexml_oxide/ root and your binding will be compiled
// and made visible to the main latexml-oxide executable

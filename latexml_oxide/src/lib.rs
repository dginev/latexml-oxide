// Macros (DefMacro!, load_model!, …) live in latexml_engine since the
// extraction; latexml_package is a regular dep (its prelude is `use`d
// from binding files, not macros).
#[macro_use]
extern crate latexml_engine;
extern crate latexml_package;

pub mod converter;
pub mod core_interface;
pub mod ini_tex;
pub mod main_tex;
pub mod post;
pub mod util;

/// Load the embedded LaTeXML schema and return its compiled-model
/// serialisation in the `.model` plain-text format. Mirrors Perl
/// `LaTeXML::Common::Model::compileSchema` (Model.pm L121-136). Used
/// by `tools/compileschema.sh` stage 2 (and the `--dump-model` flag
/// on the `latexml_oxide` binary) to regenerate `LaTeXML.model` from
/// the same source the runtime sees.
pub fn dump_compiled_latexml_model() -> String {
  use latexml_codegen::LoadModel;
  load_model!("LaTeXML");
  latexml_core::common::model::MODEL
    .borrow()
    .dump_compiled_schema()
}

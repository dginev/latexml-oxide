// Macros (DefMacro!, load_model!, …) live in latexml_engine since the
// extraction; latexml_package is a regular dep (its prelude is `use`d
// from binding files, not macros).
#[macro_use]
extern crate latexml_engine;
extern crate latexml_package;

pub mod api;
pub mod converter;
pub mod core_interface;
pub mod ini_tex;
pub mod lsp_server;
pub mod main_tex;
pub mod post;
pub mod util;

/// Load the embedded LaTeXML schema (compile-time) into the runtime
/// `MODEL`. Single home for the `load_model!("LaTeXML")` macro
/// expansion — the macro generates a fresh `_ModelLoader::build_model`
/// at each call site (~600 KiB of `.text` per copy), so funnelling
/// all callers through this one function lets LTO keep exactly one
/// instance in the final binary. Mirrors Perl
/// `LaTeXML::Common::Model::compileSchema` (Model.pm L121-136).
pub fn load_latexml_default_model() {
  use latexml_codegen::LoadModel;
  load_model!("LaTeXML");
}

/// Load the embedded LaTeXML schema and return its compiled-model
/// serialisation in the `.model` plain-text format. Used by
/// `tools/compileschema.sh` stage 2 (and the `--dump-model` flag on
/// the `latexml_oxide` binary) to regenerate `LaTeXML.model` from the
/// same source the runtime sees.
pub fn dump_compiled_latexml_model() -> String {
  load_latexml_default_model();
  latexml_core::common::model::MODEL
    .borrow()
    .dump_compiled_schema()
}

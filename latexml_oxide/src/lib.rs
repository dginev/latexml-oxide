//! latexml-oxide converts LaTeX documents into HTML5 with MathML, XML and EPUB 3: a Rust port of
//! [LaTeXML](https://github.com/brucemiller/latexml). This crate, published as `latexml`, holds the command-line
//! binaries (`latexml_oxide`, `latexmlmath_oxide`, `genschema_oxide`, and `cortex_worker` with the `cortex` feature)
//! and the library that drives a conversion.
//!
//! - [`api`]: the stable entry points, `convert_to_xml`, `convert_to_html` and `convert_to_html_with`.
//! - [`converter::Converter`]: a full conversion with its own configuration (preloads, search paths, split, …).
//! - [`post`]: the post-processing to HTML, XHTML or EPUB (Perl's `latexmlpost`).
//! - [`format_dumps`]: the format dumps embedded in the binary or built once for the host TeX tree.
//! - [`environment`] and [`intermediate_files`]: every environment variable read, and every file written.
//!
//! The engine is in `latexml_core` and `latexml_engine`, the bindings in `latexml_package` and `latexml_contrib`,
//! the math parser in `latexml_math_parser` and the post-processors in `latexml_post`. Start with the
//! [README](https://github.com/dginev/latexml-oxide/blob/main/README.md) and [USAGE.md](https://github.com/dginev/latexml-oxide/blob/main/docs/USAGE.md).

// Macros (DefMacro!, load_model!, …) live in latexml_engine since the
// extraction; latexml_package is a regular dep (its prelude is `use`d
// from binding files, not macros).
#[macro_use]
extern crate latexml_engine;
extern crate latexml_package;

pub mod api;
// Every environment variable latexml-oxide reads or sets. The text is docs/ENVIRONMENT.md, copied beside this file so
// the crate package (and docs.rs) has it; `cluster_cli::environment_doc` fails when the two differ.
#[doc = include_str!("ENVIRONMENT.md")]
pub mod environment {}
// Every file a conversion writes besides its output. The text is docs/INTERMEDIATE_FILES.md, copied beside this
// file for the crate package; `cluster_cli::intermediate_files_doc` fails when the two differ.
#[doc = include_str!("INTERMEDIATE_FILES.md")]
pub mod intermediate_files {}
pub mod bib_session;
#[cfg(feature = "test-utils")]
pub mod conformance;
pub mod converter;
pub mod core_interface;
pub mod format_dumps;
pub mod heap_trim;
pub mod identity;
pub mod ini_tex;
pub mod lsp_server;
pub mod main_tex;
pub mod multidoc;
pub mod post;
pub mod render_workers;
pub mod source_date;
pub mod streaming_restart;
pub mod util;

/// Load the embedded LaTeXML schema (compile-time) into the runtime
/// `MODEL`. Single home for the `load_model!("LaTeXML")` macro
/// expansion — the macro generates a fresh `_ModelLoader::build_model`
/// at each call site (~600 KiB of `.text` per copy), so funnelling
/// all callers through this one function lets LTO keep exactly one
/// instance in the final binary. Mirrors Perl
/// `LaTeXML::Common::Model::compileSchema` (Model.pm L121-136).
// The generated loader builds the whole schema in one frame (opt-level 0); it runs once per process at start-up. Splitting
// it per element in the `load_model!` proc macro is a post-merge refactor.
#[allow(clippy::large_unopt_stack_frames)]
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

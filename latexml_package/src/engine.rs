// Engine files — 1:1 match with Perl LaTeXML/Engine/*.pool.ltxml
//
// Loading hierarchy (mirrors Perl):
//   LaTeX.pool  →  TeX.pool  →  Base.pool  →  Base_*, TeX_*, eTeX, pdfTeX, Base_Deprecated
//                              LoadFormat('plain')  →  plain_bootstrap → plain_base → plain_dump →
// plain_constructs → math_common                  LoadFormat('latex')  →  latex_bootstrap →
// latex_base → latex_dump → latex_constructs

// Base.pool.ltxml — loaded via TeX.pool (Base.pool is a pure loader, no definitions)
mod base_deprecated;
mod base_parameter_types; // Perl: Base_ParameterTypes.pool.ltxml
mod base_schema; // Perl: Base_Schema.pool.ltxml
pub mod base_utilities; // Perl: Base_Utility.pool.ltxml
pub mod base_xmath; // Perl: Base_XMath.pool.ltxml // Perl: Base_Deprecated.pool.ltxml

// TeX_*.pool.ltxml
mod tex_box; // Perl: TeX_Box.pool.ltxml
mod tex_character; // Perl: TeX_Character.pool.ltxml
mod tex_debugging; // Perl: TeX_Debugging.pool.ltxml
mod tex_file_io; // Perl: TeX_FileIO.pool.ltxml
mod tex_fonts; // Perl: TeX_Fonts.pool.ltxml
mod tex_glue; // Perl: TeX_Glue.pool.ltxml
mod tex_hyphenation; // Perl: TeX_Hyphenation.pool.ltxml
mod tex_inserts; // Perl: TeX_Inserts.pool.ltxml
mod tex_job; // Perl: TeX_Job.pool.ltxml
mod tex_kern; // Perl: TeX_Kern.pool.ltxml
mod tex_logic; // Perl: TeX_Logic.pool.ltxml
mod tex_macro; // Perl: TeX_Macro.pool.ltxml
mod tex_marks; // Perl: TeX_Marks.pool.ltxml
pub(crate) mod tex_math; // Perl: TeX_Math.pool.ltxml (includes tex_scripts content)
mod tex_page; // Perl: TeX_Page.pool.ltxml
mod tex_paragraph; // Perl: TeX_Paragraph.pool.ltxml
mod tex_penalties; // Perl: TeX_Penalties.pool.ltxml
mod tex_registers; // Perl: TeX_Registers.pool.ltxml
pub mod tex_tables; // Perl: TeX_Tables.pool.ltxml

// eTeX + pdfTeX extensions
pub mod etex; // Perl: eTeX.pool.ltxml
pub mod pdftex; // Perl: pdfTeX.pool.ltxml

// plain TeX format — LoadFormat('plain') chain called by tex.rs
mod math_common;
mod plain_base; // Perl: plain_base.pool.ltxml
mod plain_bootstrap; // Perl: plain_bootstrap.pool.ltxml
mod plain_constructs; // Perl: plain_constructs.pool.ltxml
pub mod plain_dump; // Rust: precompiled plain.ltx state (auto-generated) // Perl: math_common.pool.ltxml

// LaTeX format — LoadFormat('latex') chain called by latex.rs
pub mod latex_base; // Perl: latex_base.pool.ltxml
pub mod latex_bootstrap; // Perl: latex_bootstrap.pool.ltxml
pub mod latex_constructs;
pub mod latex_dump; // Rust: precompiled latex.ltx state (auto-generated) // Perl: latex_constructs.pool.ltxml (C.1-C.15)

// Top-level entry points
pub mod latex;
pub mod tex; // Perl: TeX.pool.ltxml (loads Base + LoadFormat('plain')) // Perl: LaTeX.pool.ltxml (loads TeX + LoadFormat('latex'))

// AmSTeX format — LoadPool('AmSTeX'), routed via `\input amstex`.
pub mod amstex; // Perl: AmSTeX.pool.ltxml

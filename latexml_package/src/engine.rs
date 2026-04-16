// TeX Pool
mod base_parameter_types;
mod base_schema;
mod base_utilities;
pub mod base_xmath;
pub mod tex;
mod tex_box;
mod tex_character;
mod tex_debugging;
mod tex_file_io;
mod tex_fonts;
mod tex_glue;
mod tex_hyphenation;
mod tex_inserts;
mod tex_job;
mod tex_kern;
mod tex_logic;
mod tex_macro;
mod tex_marks;
mod tex_math;
mod tex_page;
mod tex_paragraph;
mod tex_penalties;
mod tex_registers;
pub mod tex_tables;

pub mod base_functions; // auxiliary functions
pub(crate) mod tex_scripts;
// Deprecated aliases
mod base_deprecated;
// eTeX Pool
pub mod etex;
// pdfTeX Pool
pub mod pdftex;
// plain TeX Pool — matches Perl Engine/ structure:
//   plain (→ plain_bootstrap → plain_dump → plain_constructs → math_common)
mod math_common;       // Perl: math_common.pool.ltxml
mod plain_bootstrap;   // Perl: plain_bootstrap.pool.ltxml
mod plain_constructs;  // Perl: plain_constructs.pool.ltxml
mod plain;             // Perl: plain_base.pool.ltxml (content matches, file name kept for compatibility)

// LaTeX Pool — matches Perl Engine/ structure:
//   latex (→ latex_bootstrap → latex_dump → latex_constructs)
mod latex_base;        // Perl: latex_base.pool.ltxml (infrastructure, no constructors)
mod latex_bootstrap;   // Perl: latex_bootstrap.pool.ltxml
pub mod latex_constructs;  // Perl: latex_constructs.pool.ltxml (all C.1-C.15 definitions)
pub mod latex;
pub mod latex_functions; // auxiliary functions
// latex_hook.rs removed — content moved to tex.rs (Perl: TeX.pool.ltxml L33-56)
// latex_other_in_appendices.rs removed — content moved to latex_base.rs and latex_constructs.rs
// latex_semi_undocumented.rs removed — content moved to latex_base.rs and latex_constructs.rs
// Precompiled kernel dumps (auto-generated, loads definitions from format dumps)
// Perl equivalent: LoadFormat('plain') / LoadFormat('latex')
pub mod plain_dump;
pub mod latex_dump;

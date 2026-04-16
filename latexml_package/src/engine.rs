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
// plain TeX Pool �� matches Perl Engine/ structure:
//   plain (→ plain_bootstrap → plain_dump → plain_constructs → math_common)
mod math_common;       // Perl: math_common.pool.ltxml
mod plain_bootstrap;   // Perl: plain_bootstrap.pool.ltxml
mod plain_constructs;  // Perl: plain_constructs.pool.ltxml
mod plain;             // Perl: plain_base.pool.ltxml (content matches, file name kept for compatibility)

// LaTeX Pool — matches Perl Engine/ structure:
//   latex (→ latex_bootstrap → latex_dump → latex_constructs)
mod latex_base;        // Perl: latex_base.pool.ltxml (infrastructure, no constructors)
mod latex_bootstrap;   // Perl: latex_bootstrap.pool.ltxml
mod latex_constructs;  // Perl: latex_constructs.pool.ltxml (wraps all latex_ch*.rs files)
pub mod latex;
mod latex_ch10_array_and_tabular;
mod latex_ch10_tabbing_environment;
mod latex_ch11_index_and_glossary;
pub(crate) mod latex_ch11_moving_information;
mod latex_ch11_splitting_the_input;
mod latex_ch11_terminal_io;
mod latex_ch12_line_and_page_breaking;
mod latex_ch13_boxes;
mod latex_ch14_pictures_and_color;
mod latex_ch15_font_selection;
mod latex_ch15_special_symbol;
mod latex_ch1_break_command;
mod latex_ch1_documentclass;
mod latex_ch1_environments;
mod latex_ch1_fragile_commands;
mod latex_ch2_document;
mod latex_ch3_sentences_and_paragraphs;
mod latex_ch4_sectioning_and_toc;
mod latex_ch5_packages;
mod latex_ch5_page_styles;
mod latex_ch5_title_page_and_abstract;
mod latex_ch6_displayed_paragraphs;
mod latex_ch6_list_and_trivlist_environments;
mod latex_ch6_list_making_environments;
mod latex_ch6_quotations_and_verse;
mod latex_ch6_verbatim;
mod latex_ch7_math_common_delimiters;
mod latex_ch7_math_common_structures;
mod latex_ch7_math_mode_changing_style;
mod latex_ch7_math_mode_environments;
mod latex_ch8_defining_commands;
mod latex_ch8_defining_environments;
mod latex_ch8_numbering;
pub mod latex_ch8_theoremlike_environments;
pub mod latex_ch9_figures_and_tables;
mod latex_ch9_marginal_notes;
pub mod latex_functions; // auxiliary functions
// latex_hook.rs removed — content moved to tex.rs (Perl: TeX.pool.ltxml L33-56)
// latex_other_in_appendices.rs removed — content moved to latex_base.rs and latex_constructs.rs
// latex_semi_undocumented.rs removed — content moved to latex_base.rs and latex_constructs.rs
mod latex_tables_3;
// Precompiled kernel dumps (auto-generated, loads definitions from format dumps)
// Perl equivalent: LoadFormat('plain') / LoadFormat('latex')
pub mod plain_dump;
pub mod latex_dump;

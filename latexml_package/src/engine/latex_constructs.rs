//! latex_constructs — LaTeX semantic definitions (constructors, environments)
//!
//! Perl: latex_constructs.pool.ltxml (6014 lines)
//! Loaded AFTER latex_dump in the Perl loading order.
//! Contains DefConstructor, DefEnvironment, Tag!, and other semantic
//! definitions that build on the basic infrastructure from latex_base.
//!
//! In our Rust port, these are organized by Lamport chapter (latex_ch*.rs files).
use crate::prelude::*;

LoadDefinitions!({
  // C.1 Commands and Environments
  InnerPool!(latex_ch1_documentclass);
  InnerPool!(latex_ch1_environments);
  InnerPool!(latex_ch1_fragile_commands);
  InnerPool!(latex_ch1_break_command);

  // C.2 The Structure of the Document
  InnerPool!(latex_ch2_document);

  // C.3 Sentences and Paragraphs
  InnerPool!(latex_ch3_sentences_and_paragraphs);

  // C.4 Sectioning and Table of Contents
  InnerPool!(latex_ch4_sectioning_and_toc);

  // C.5 Classes, Packages and Page Styles
  InnerPool!(latex_ch5_packages);
  InnerPool!(latex_ch5_page_styles);
  InnerPool!(latex_ch5_title_page_and_abstract);

  // C.6 Displayed Paragraphs
  InnerPool!(latex_ch6_displayed_paragraphs);
  InnerPool!(latex_ch6_quotations_and_verse);
  InnerPool!(latex_ch6_list_making_environments);
  InnerPool!(latex_ch6_list_and_trivlist_environments);
  InnerPool!(latex_ch6_verbatim);

  // C.7 Mathematical Formulas
  InnerPool!(latex_ch7_math_mode_environments);
  InnerPool!(latex_ch7_math_common_structures);
  InnerPool!(latex_ch7_math_common_delimiters);
  InnerPool!(latex_ch7_math_mode_changing_style);

  // C.8 Definitions, Numbering and Programming
  InnerPool!(latex_ch8_defining_commands);
  InnerPool!(latex_ch8_defining_environments);
  InnerPool!(latex_ch8_theoremlike_environments);
  InnerPool!(latex_ch8_numbering);

  // C.9 Figures and Other Floating Bodies
  InnerPool!(latex_ch9_figures_and_tables);
  InnerPool!(latex_ch9_marginal_notes);

  // C.10 Lining It Up in Columns
  InnerPool!(latex_ch10_tabbing_environment);
  InnerPool!(latex_ch10_array_and_tabular);

  // C.11 Moving Information Around
  InnerPool!(latex_ch11_moving_information);
  InnerPool!(latex_ch11_splitting_the_input);
  InnerPool!(latex_ch11_index_and_glossary);
  InnerPool!(latex_ch11_terminal_io);

  // C.12-C.13 Line/Page Breaking, Boxes
  InnerPool!(latex_ch12_line_and_page_breaking);
  InnerPool!(latex_ch13_boxes);

  // C.14-C.15 Pictures, Fonts, Symbols
  InnerPool!(latex_ch14_pictures_and_color);
  InnerPool!(latex_ch15_font_selection);
  InnerPool!(latex_ch15_special_symbol);

  // Additional appendix definitions and semi-documented commands
  InnerPool!(latex_other_in_appendices);
  InnerPool!(latex_semi_undocumented);

  // Perl latex_constructs.pool.ltxml L5937-5938:
  // LaTeX now includes textcomp by default.
  RequirePackage!("textcomp");
});

///**********************************************************************
/// Organized following
///  "`LaTeX`: A Document Preparation System"
///   by Leslie Lamport
///   2nd edition
/// Addison Wesley, 1994
/// Appendix C. Reference Manual
///**********************************************************************
/// NOTE: This will be loaded after `TeX.pool`, so it inherits.
///**********************************************************************
use crate::package::*;

LoadDefinitions!(state, {
  //**********************************************************************
  // Organized following
  //  "LaTeX: A Document Preparation System"
  //   by Leslie Lamport
  //   2nd edition
  // Addison Wesley, 1994
  // Appendix C. Reference Manual
  //**********************************************************************
  // NOTE: This will be loaded after TeX.pool.ltxml, so it inherits.
  //**********************************************************************

  LoadPool!("TeX");
  // lines 31-110
  InnerPool!(latex_ch1_documentclass);
  // lines 110-180
  InnerPool!(latex_ch1_environments);
  // lines 180-250
  InnerPool!(latex_ch1_fragile_commands);
  // lines 251-276
  InnerPool!(latex_ch1_break_command);
  // lines 276-372
  InnerPool!(latex_ch2_document);
  // lines 372-530
  InnerPool!(latex_ch3_sentences_and_paragraphs);
  // lines 530-687
  InnerPool!(latex_ch4_sectioning_and_toc);
  // lines 687-1066
  InnerPool!(latex_ch5_packages);
  // lines 1066-1125
  InnerPool!(latex_ch5_page_styles);
  // lines 1102-1310 (05.2022)
  InnerPool!(latex_ch5_title_page_and_abstract);
  // lines 1311-1376  (05.2022)
  InnerPool!(latex_ch6_displayed_paragraphs);
  // lines 1377-1395  (05.2022)
  InnerPool!(latex_ch6_quotations_and_verse);

  InnerPool!(latex_ch6_list_making_environments);
  // lines 1396-1550
  InnerPool!(latex_ch6_list_and_trivlist_environments);
  // lines 1551-1646
  InnerPool!(latex_ch6_verbatim);
  // lines 1646-2164
  InnerPool!(latex_ch7_math_mode_environments);
  // lines 2164-2180
  InnerPool!(latex_ch7_math_common_structures);
  // lines 2180-2216
  InnerPool!(latex_ch7_math_common_delimiters);
  // lines 2216-2246
  InnerPool!(latex_ch7_math_mode_changing_style);
  // lines 2247-2511
  InnerPool!(latex_ch8_defining_commands);
  // lines 2512-2536
  InnerPool!(latex_ch8_defining_environments);
  // lines 2536-2712
  InnerPool!(latex_ch8_theoremlike_environments);
  // lines 2712-2785
  InnerPool!(latex_ch8_numbering);
  // lines 2786-2975
  InnerPool!(latex_ch9_figures_and_tables);
  // lines 2975-2985
  InnerPool!(latex_ch9_marginal_notes);
  // lines 2985-3086
  InnerPool!(latex_ch10_tabbing_environment);
  // lines 3086-3229
  InnerPool!(latex_ch10_array_and_tabular);
  // lines 3230-3567
  InnerPool!(latex_ch11_moving_information);
  // lines 3568-3626
  InnerPool!(latex_ch11_splitting_the_input);
  // lines 3627-3821
  InnerPool!(latex_ch11_index_and_glossary);
  // lines 3821-3832
  InnerPool!(latex_ch11_terminal_io);
  // lines 3832-3866
  InnerPool!(latex_ch12_line_and_page_breaking);
  // lines 3866-4123
  InnerPool!(latex_ch13_boxes);
  // lines 4124-4414
  InnerPool!(latex_ch14_pictures_and_color);
  // lines 4414-4568
  InnerPool!(latex_ch15_font_selection);
  // lines 4568-4665
  InnerPool!(latex_ch15_special_symbol);
  // lines 4666-5200
  InnerPool!(latex_other_in_appendices);
  // lines 5200-5366
  InnerPool!(latex_semi_undocumented);
});

use crate::package::*;
use rtx_core::state::State;

LoadDefinitions!(state, {
  // TeX.pool.ltxml
  //   commit 22db863d7358d56e197a3845375775714577cc82
  //   Author: bruce miller <bruce.miller@nist.gov>
  //   Date:   Wed Nov 28 10:47:09 2018 -0500

  // lines 1-695
  InnerPool!(tex_setup);
  // lines 695-949
  InnerPool!(tex_expandable_primitives);
  // lines 950-1102
  InnerPool!(tex_registers);
  // lines 1102-1408
  InnerPool!(tex_assignment);
  // lines 1408-1773
  InnerPool!(tex_fonts);
  // lines 1773-2085
  InnerPool!(tex_boxes);

  // -----------------------------------------
  //  29.01.2019:
  //  updated upto (and including) here
  // -----------------------------------------

  // lines 2086-2335
  InnerPool!(tex_ch24_primitives);
  // lines 2335-2985
  InnerPool!(tex_alignment);
  // lines 2985-3070
  InnerPool!(tex_para);
  // lines 3070-3158
  InnerPool!(tex_ch25_primitives);
  // lines 3158-3625
  InnerPool!(tex_math_mode);
  // lines 3625-3905
  InnerPool!(tex_scripts);
  // lines 3905-4090
  InnerPool!(tex_math_style);
  // lines 4090-4401
  InnerPool!(tex_appendix_b_to_p349);
  // lines 4401-4660
  InnerPool!(tex_appendix_b_p350_to_p355);
  // lines 4660-4932
  InnerPool!(tex_frontmatter);
  // lines 4932-4965
  InnerPool!(tex_appendix_b_p356);
  // lines 4965-5086
  InnerPool!(tex_accents);
  // lines 5086-5159
  InnerPool!(tex_appendix_b_p357);
  // lines 5159-5607
  InnerPool!(tex_appendix_b_p358);
  // lines 5607-5653
  InnerPool!(tex_appendix_b_p359);
  // lines 5653-5701
  InnerPool!(tex_math_accents);
  // lines 5701-5898
  InnerPool!(latex_delimiters);
  // lines 5898-5952
  InnerPool!(tex_appendix_b_p360);
  // lines 5952-6037
  InnerPool!(tex_appendix_b_p361);
  // lines 6037-6225
  InnerPool!(tex_appendix_b_p362);
  // lines 6225-6254
  InnerPool!(tex_appendix_b_p363);
  // lines 6254-6261
  InnerPool!(tex_appendix_b_p364);

  //======================================================================
  // End of TeX Book definitions.
  //======================================================================

  //**********************************************************************
  // Stray stuff .... where to ?
  //**********************************************************************
  // lines 6269-6291
  InnerPool!(tex_stray_math_style);
  // lines 6005-6392
  InnerPool!(tex_special_chars);
  // lines 6392-6434
  InnerPool!(latex_hook);
  // lines 6434-6752
  InnerPool!(tex_rtx_specific);
  // lines 6753-END
  InnerPool!(etex);
  InnerPool!(pdftex);
});

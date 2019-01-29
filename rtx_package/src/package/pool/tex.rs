use crate::package::*;
use rtx_core::state::State;

LoadDefinitions!(state, {
  // TeX.pool.ltxml
  //   commit 22db863d7358d56e197a3845375775714577cc82
  //   Author: bruce miller <bruce.miller@nist.gov>
  //   Date:   Wed Nov 28 10:47:09 2018 -0500

  // lines 1-695
  // XML language, DefParameterType
  InnerPool!(tex_setup);
  // lines 695-949
  InnerPool!(tex_expandable_primitives);

  //**********************************************************************
  // Primitives
  // See The TeXBook, Chapter 24, Summary of Vertical Mode
  //  and Chapter 25, Summary of Horizontal Mode.
  // Parsing of basic types (pp.268--271) is (mostly) handled in Gullet.pm
  //**********************************************************************
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

  // lines 2086-2192
  InnerPool!(tex_ch24_primitives);

  // lines 2192-2840
  InnerPool!(tex_alignment);

  // lines 2840-2918
  InnerPool!(tex_para);

  // lines 2918-3009
  InnerPool!(tex_ch25_primitives);

  // lines 3009-3474
  InnerPool!(tex_math_mode);

  // lines 3474-3751
  InnerPool!(tex_scripts);

  // lines 3751-3938
  InnerPool!(tex_math_style);

  // lines 3938-4490
  InnerPool!(tex_appendix_b);

  // lines 4490-4606
  // General support for Front Matter.
  InnerPool!(tex_frontmatter);

  // lines 4606-4648
  InnerPool!(tex_references);

  // lines 4648-4801
  InnerPool!(tex_accents);

  // lines 4801-4920
  InnerPool!(tex_appendix_b_p357);

  // lines,v2 5162-5609
  InnerPool!(tex_appendix_b_p358);

  // lines 5321-5367
  InnerPool!(tex_appendix_b_p359);

  // lines 5367-5414
  InnerPool!(tex_math_accents);

  // lines 5414-5611
  InnerPool!(latex_delimiters);

  // lines 5611-5691
  InnerPool!(tex_appendix_b_p360);
  InnerPool!(tex_appendix_b_p361);

  // lines 5691-5750
  InnerPool!(latex_loglike_functions);

  // lines 5750-5976
  InnerPool!(tex_appendix_b_p362); // includes 363?
  InnerPool!(tex_appendix_b_p364);

  // lines 5976-6005
  InnerPool!(tex_stray_math_style);

  // lines 6005-6103
  InnerPool!(tex_special_chars);

  // lines 6103-6144
  InnerPool!(latex_hook);

  // lines 6144-6433
  InnerPool!(tex_rtx_specific);

  // lines 6433-END
  LoadPool!("eTeX");
  LoadPool!("pdfTeX");
});

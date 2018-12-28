use crate::package::*;
use rtx_core::state::State;

pub fn load_definitions(mut state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  // TeX.pool.ltxml
  //   commit 73f27e8ca8a4179fd3d37744141647d1e604cc97
  //   Author: Bruce Miller <bruce.miller@nist.gov>
  //   Date:   Mon Jul 17 16:34:13 2017 -0400

  // lines 1-604
  // XML language, DefParameterType
  InnerPool!(tex_setup);

  // lines 604-912
  InnerPool!(tex_expandable_primitives);

  // lines 912-979
  // Dimen registers; TeXBook p. 274
  InnerPool!(tex_registers);

  // lines 979-1278
  InnerPool!(tex_assignment);

  // lines 1278-1649
  InnerPool!(tex_fonts);

  // lines 1649-1954
  InnerPool!(tex_boxes);

  // lines 1954-2192
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
  InnerPool!(tex_appendix_b_p358);

  // lines 4920-5321
  InnerPool!(latex_tables_3);

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

  Ok(())
}

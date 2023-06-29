use crate::package::*;
LoadDefinitions!({
  // port of TeX.pool.ltxml
  // commit 6b748e367af3b15c11f41454bfe79bfc27763577
  // Author: Deyan Ginev <deyan.ginev@gmail.com>
  // Date:   Wed May 10 16:36:04 2023 -0400

  // lines 1-758
  InnerPool!(tex_setup);
  // lines 758-1075
  InnerPool!(tex_expandable_primitives);
  // lines 1076-1216
  InnerPool!(tex_registers);
  // lines 1217-1524
  InnerPool!(tex_assignment);
  // lines 1524-1888
  InnerPool!(tex_fonts);
  // lines 1889-2416
  InnerPool!(tex_boxes);
  // lines 2417-2785
  InnerPool!(tex_ch24_primitives);
  // lines 2786-3523
  InnerPool!(tex_alignment);
  // lines 3524-3660
  InnerPool!(tex_para);
  // lines 3661-3783
  InnerPool!(tex_ch25_primitives);
  // lines 3784-4006
  InnerPool!(tex_math_mode);
  // lines 4007-4279
  InnerPool!(tex_math_fork);
  // lines 4280-4510
  InnerPool!(tex_scripts);
  // lines 4511-4688
  InnerPool!(tex_math_style);
  // lines 4689-5041
  InnerPool!(tex_appendix_b_to_p349);
  // lines 5042-5290
  InnerPool!(tex_appendix_b_p350_to_p355);
  // lines 5291-5620
  InnerPool!(tex_frontmatter);
  // lines 5621-5655
  InnerPool!(tex_appendix_b_p356);
  // lines 5656-5783
  InnerPool!(tex_accents);
  // lines 5784-5832
  InnerPool!(tex_appendix_b_p357);
  // lines 5833-6278
  InnerPool!(tex_appendix_b_p358);
  // lines 6279-6329
  InnerPool!(tex_appendix_b_p359);
  // lines 6330-6377
  InnerPool!(tex_math_accents);
  // lines 6378-6574
  InnerPool!(latex_delimiters);
  // lines 6575-6629
  InnerPool!(tex_appendix_b_p360);
  // lines 6630-6714
  InnerPool!(tex_appendix_b_p361);
  // lines 6715-6960
  InnerPool!(tex_appendix_b_p362);
  // lines 6961-6998
  InnerPool!(tex_appendix_b_p363);
  // lines 6999-7010
  InnerPool!(tex_appendix_b_p364);

  //======================================================================
  // End of TeX Book definitions.
  //======================================================================

  //**********************************************************************
  // Stray stuff .... where to ?
  //**********************************************************************

  // lines 7013-7036
  InnerPool!(tex_stray_math_style);
  // lines 7037-7140
  InnerPool!(tex_special_chars);
  // lines 7141-7203
  InnerPool!(latex_hook);
  // lines 7204-7221
  InnerPool!(tex_rtx_specific);
  // lines 7222-7544
  InnerPool!(rtx_math_macros);
  // lines 7545-7720
  InnerPool!(tex_misc_tweaks);

  // lines 7721 - 7725
  InnerPool!(etex);
  InnerPool!(pdftex);
});

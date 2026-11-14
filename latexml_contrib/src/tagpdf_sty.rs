use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // tagpdf.sty — PDF/UA tagging layer. Tag-structure commands drive the
  // PDF backend's marked-content operators; our XML output IS the
  // accessible structure, so the surface is absorbed (sweep-16 tail:
  // tagpdf's own manual + tex-vpat).
  def_macro_noop("\\tagpdfsetup{}")?;
  def_macro_noop("\\tagstructbegin{}")?;
  def_macro_noop("\\tagstructend")?;
  def_macro_noop("\\tagmcbegin{}")?;
  def_macro_noop("\\tagmcend")?;
  def_macro_noop("\\tagpdfparaOn")?;
  def_macro_noop("\\tagpdfparaOff")?;
  DefEnvironment!("{tagpdfsuppress}", "#body");
  def_macro_noop("\\tagpdfsuppressmarks{}")?;

  // Role/namespace tables. tagpdf.sty:1329-1665 allocates one
  // `\g__tag_role_NS_<ns>_prop` per structure namespace and fills it from
  // the `tagpdf-ns-<ns>.def` CSV files (`tag,role,rolens,type,`), storing
  // `{role}{}` per tag under the PDF<2.0 branch (`\pdf_version_compare:NnTF
  // < {2.0}`, tagpdf.sty:1594-1615). The manual (tagpdf.tex:2161-2200)
  // iterates those props to list the PDF 1.7 / 2.0 structure names, so
  // the tables are real document content, not backend state: read the
  // same TL data files with the same line reader. Only the rolemap /
  // Namespace PDF-object plumbing is absorbed. Guard:
  // `perfect_kernel_batch52::tagpdf_role_namespace_props_are_populated`.
  raw_tex(r"
    \ExplSyntaxOn
    \prop_new:N \g__tag_role_NS_prop
    \prop_new:N \g__tag_role_tags_NS_prop
    \prop_new:N \g__tag_role_tags_class_prop
    \cs_new_protected:Npn \__tag_role_NS_new:nnn #1 #2 #3
      {
        \prop_new:c { g__tag_role_NS_#1_prop }
        \prop_new:c { g__tag_role_NS_#1_class_prop }
        \prop_gput:Nne \g__tag_role_NS_prop {#1}{}
      }
    \__tag_role_NS_new:nnn {pdf}  {http://iso.org/pdf/ssn}{}
    \__tag_role_NS_new:nnn {pdf2} {http://iso.org/pdf2/ssn}{}
    \__tag_role_NS_new:nnn {mathml}{http://www.w3.org/1998/Math/MathML}{}
    \__tag_role_NS_new:nnn {latex} {https://latex-project.org/ns/pdf/latex}{}
    \__tag_role_NS_new:nnn {latex-book} {https://latex-project.org/ns/pdf/latex-book}{}
    \__tag_role_NS_new:nnn {latex-inline} {https://latex-project.org/ns/pdf/latex-inline}{}
    \__tag_role_NS_new:nnn {user}{}{}
    \cs_new_protected:Npn \__tag_role_alloctag:nnn #1 #2 #3
      {
        \prop_gput:Nnn \g__tag_role_tags_NS_prop   {#1}{#2}
        \prop_gput:cnn {g__tag_role_NS_#2_prop}  {#1}{{}{}}
        \prop_gput:Nnn \g__tag_role_tags_class_prop {#1}{#3}
        \prop_gput:cnn {g__tag_role_NS_#2_class_prop}  {#1}{#3}
      }
    \cs_generate_variant:Nn \__tag_role_alloctag:nnn {nno}
    \cs_new_protected:Npn \__tag_role_read_namespace_line:nw #1#2,#3,#4,#5,#6\q_stop
      {
        \tl_if_empty:nF { #2 }
         {
          \tl_if_empty:nTF {#5}
            {
              \prop_get:NnN \g__tag_role_tags_class_prop  {#3}\l__tag_tmpa_tl
              \quark_if_no_value:NT \l__tag_tmpa_tl
                { \tl_set:Nn\l__tag_tmpa_tl{--UNKNOWN--} }
            }
            { \tl_set:Nn \l__tag_tmpa_tl {#5} }
          \__tag_role_alloctag:nno {#2} {#1} { \l__tag_tmpa_tl }
          \prop_gput:cnn {g__tag_role_NS_#1_prop}  {#2}{{#3}{}}
         }
      }
    \cs_new_protected:Npn \__tag_role_read_namespace:nn #1 #2
      {
        \file_if_exist:nT { tagpdf-ns-#2.def }
         {
           \ior_open:Nn \g_tmpa_ior {tagpdf-ns-#2.def}
           \ior_map_inline:Nn \g_tmpa_ior
             { \__tag_role_read_namespace_line:nw {#1} ##1,,,,\q_stop }
           \ior_close:N\g_tmpa_ior
         }
      }
    \cs_new_protected:Npn \__tag_role_read_namespace:n #1
      { \__tag_role_read_namespace:nn {#1}{#1} }
    \tl_new:N \l__tag_tmpa_tl
    \__tag_role_read_namespace:n {pdf}
    \__tag_role_read_namespace:n {pdf2}
    \__tag_role_read_namespace:n {mathml}
    \__tag_role_read_namespace:n {latex-book}
    \__tag_role_read_namespace:n {latex}
    \__tag_role_read_namespace:nn {latex} {latex-lab}
    \ExplSyntaxOff
  ")?;
});

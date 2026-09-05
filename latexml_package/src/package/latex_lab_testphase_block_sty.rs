//! latex-lab-testphase-block.sty — the template types, interfaces and standard
//! instances the module declares, with inert template code.
//!
//! `\DocumentMetadata{tagging=on}` loads this module through
//! `latex-lab-testphase-latest.sty:43`. Raw classes and packages written for
//! the tagging project then edit or use its declarations: ltx-talk.cls:1860
//! `\EditInstance{item}{basic}{…}` ("instance 'basic' of type 'item' is
//! unknown", ×10 manuals), tagpdfdocu-patches.sty:127
//! `\DeclareInstance{blockenv}{docCommand}{display}` + `\UseInstance` per
//! `{docCommand}` and `\endblockenv` (tagpdf manual, 25 template errors; all
//! SHARED, pdflatex clean). The real template CODE (block.sty:202-640)
//! re-implements LaTeX's list/paragraph layout with PDF tagging recipes — our
//! XML lists are the tagged structure already — so the code bodies here only
//! absorb the keys (`\SetKnownTemplateKeys`, as the real blockenv code does
//! for its pass-through keys) and `\endblockenv` ends the paragraph. Loading
//! the raw module is not an option: it `\cs_new`s `\DebugBlocksOn`/`Off` over
//! the kernel stubs and redefines `{itemize}` over our constructors.
//! Interfaces are block.sty:101-170 verbatim; instances carry their `name`.
//! Guard: `perfect_kernel_batch56::testphase_tagging_sockets_and_block_templates_are_declared`.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!(r"\ExplSyntaxOn
\NewTemplateType{blockenv}{1}
\NewTemplateType{block}{1}
\NewTemplateType{para}{1}
\NewTemplateType{list}{1}
\NewTemplateType{item}{1}
\DeclareTemplateInterface{blockenv}{display}{1}
{
  name                   : tokenlist ,
  tag-name               : tokenlist = ,
  tag-attr-class         : tokenlist = ,
  tagging-recipe         : tokenlist = standard,
  increment-level        : boolean   = true ,
  setup-code             : tokenlist = ,
  block-instance         : tokenlist = displayblock ,
  para-instance          : tokenlist ,
  inner-level-counter    : tokenlist ,
  max-inner-levels       : tokenlist = 4,
  inner-instance-type    : tokenlist = list ,
  inner-instance         : tokenlist = ,
  tagging-suppress-paras : boolean = false ,
  final-code             : tokenlist = \ignorespaces ,
}
\DeclareTemplateInterface{block}{display}{1}
{
  begin-vspace       : skip = \topsep ,
  begin-extra-vspace : skip = \partopsep ,
  para-vspace        : skip = \parsep ,
  end-vspace         : skip = \KeyValue{begin-vspace} ,
  end-extra-vspace   : skip = \KeyValue{begin-extra-vspace} ,
  item-vspace        : skip = \itemsep ,
  begin-penalty      : integer = \UseName{@beginparpenalty} ,
  end-penalty        : integer = \UseName{@endparpenalty} ,
  left-margin        : length = \leftmargin ,
  right-margin       : length = \rightmargin ,
  para-indent        : length = 0pt ,
}
\DeclareTemplateInterface{para}{std}{1}
{
  para-attr-class       : tokenlist = justify ,
  para-indent           : length = \parindent ,
  begin-hspace          : skip = 0pt  ,
  left-hspace           : skip = 0pt ,
  right-hspace          : skip = 0pt ,
  end-hspace            : skip = \@flushglue ,
  fixed-word-spaces     : boolean = false ,
  final-hyphen-demerits : integer = 5000 ,
  newline-cmd           : tokenlist = \@normalcr ,
}
\DeclareTemplateInterface{list}{std}{1}
{
  counter         : tokenlist = ,
  item-label      : tokenlist = ,
  start           : integer = 1 ,
  resume          : boolean = false ,
  item-instance   : instance{item} = basic ,
  item-vspace     : skip = \itemsep ,
  item-penalty    : integer = \UseName{@itempenalty} ,
  item-indent     : length = \itemindent ,
  label-width     : length = \labelwidth ,
  label-sep       : length = \labelsep ,
  legacy-support  : boolean = false ,
}
\DeclareTemplateInterface{item}{std}{1}
  {
    counter-label : function{1} = \arabic{#1} ,
    counter-ref   : function{1} = \KeyValue{counter-label} ,
    label-ref     : function{1} = #1 ,
    label-autoref : function{1} = item~#1 ,
    label-format  : function{1} = #1 ,
    label-strut   : boolean = false ,
    label-align   : choice {left,center,right,parleft} = right ,
    label-boxed   : boolean = true ,
    next-line     : boolean = false ,
    text-font     : tokenlist ,
    compatibility : boolean = true ,
  }
\tl_new:N \l__block_env_name_tl
\tl_new:N \l__block_tag_name_tl
\tl_new:N \l__block_tag_class_tl
\tl_new:N \l__block_tagging_recipe_tl
\bool_new:N \l__block_level_incr_bool
\tl_new:N \l__block_setup_code_tl
\tl_new:N \l__block_block_instance_tl
\tl_new:N \l__block_para_instance_tl
\bool_new:N \l__block_para_flattened_bool
\tl_new:N \l__block_inner_level_counter_tl
\tl_new:N \l__block_max_inner_levels_tl
\tl_new:N \l__block_inner_instance_type_tl
\tl_new:N \l__block_inner_instance_tl
\tl_new:N \l__block_final_code_tl
\DeclareTemplateCode{blockenv}{display}{1}
{
  name                = \l__block_env_name_tl ,
  tag-name            = \l__block_tag_name_tl ,
  tag-attr-class      = \l__block_tag_class_tl ,
  tagging-recipe      = \l__block_tagging_recipe_tl ,
  increment-level     = \l__block_level_incr_bool ,
  setup-code          = \l__block_setup_code_tl ,
  block-instance      = \l__block_block_instance_tl ,
  para-instance       = \l__block_para_instance_tl ,
  tagging-suppress-paras = \l__block_para_flattened_bool ,
  inner-level-counter = \l__block_inner_level_counter_tl ,
  max-inner-levels    = \l__block_max_inner_levels_tl ,
  inner-instance-type = \l__block_inner_instance_type_tl ,
  inner-instance      = \l__block_inner_instance_tl ,
  final-code          = \l__block_final_code_tl ,
}
{ \SetKnownTemplateKeys{blockenv}{display}{#1} }
\skip_new:N \l__block_begin_vspace_skip
\skip_new:N \l__block_begin_extra_vspace_skip
\skip_new:N \l__block_para_vspace_skip
\skip_new:N \l__block_end_vspace_skip
\skip_new:N \l__block_end_extra_vspace_skip
\skip_new:N \l__block_item_vspace_skip
\int_new:N \l__block_begin_penalty_int
\int_new:N \l__block_end_penalty_int
\dim_new:N \l__block_left_margin_dim
\dim_new:N \l__block_right_margin_dim
\dim_new:N \l__block_para_indent_dim
\DeclareTemplateCode{block}{display}{1}
{
  begin-vspace       = \l__block_begin_vspace_skip ,
  begin-extra-vspace = \l__block_begin_extra_vspace_skip ,
  para-vspace        = \l__block_para_vspace_skip ,
  end-vspace         = \l__block_end_vspace_skip ,
  end-extra-vspace   = \l__block_end_extra_vspace_skip ,
  item-vspace        = \l__block_item_vspace_skip ,
  begin-penalty      = \l__block_begin_penalty_int ,
  end-penalty        = \l__block_end_penalty_int ,
  left-margin        = \l__block_left_margin_dim ,
  right-margin       = \l__block_right_margin_dim ,
  para-indent        = \l__block_para_indent_dim ,
}
{ \SetKnownTemplateKeys{block}{display}{#1} }
\tl_new:N \l__block_para_class_tl
\skip_new:N \l__block_para_begin_skip
\skip_new:N \l__block_para_left_skip
\skip_new:N \l__block_para_right_skip
\skip_new:N \l__block_para_end_skip
\bool_new:N \l__block_para_fixed_spaces_bool
\int_new:N \l__block_para_hyphen_demerits_int
\tl_new:N \l__block_para_newline_tl
\DeclareTemplateCode{para}{std}{1}
{
  para-attr-class       = \l__block_para_class_tl ,
  para-indent           = \l__block_para_indent_dim ,
  begin-hspace          = \l__block_para_begin_skip ,
  left-hspace           = \l__block_para_left_skip ,
  right-hspace          = \l__block_para_right_skip ,
  end-hspace            = \l__block_para_end_skip ,
  fixed-word-spaces     = \l__block_para_fixed_spaces_bool ,
  final-hyphen-demerits = \l__block_para_hyphen_demerits_int ,
  newline-cmd           = \l__block_para_newline_tl ,
}
{ \SetKnownTemplateKeys{para}{std}{#1} }
\tl_new:N \l__block_counter_tl
\tl_new:N \l__block_item_label_tl
\int_new:N \l__block_counter_start_int
\bool_new:N \l__block_resume_bool
\int_new:N \l__block_item_penalty_int
\dim_new:N \l__block_item_indent_dim
\dim_new:N \l__block_label_width_dim
\dim_new:N \l__block_label_sep_dim
\bool_new:N \l__block_legacy_support_bool
\DeclareTemplateCode{list}{std}{1}
{
  counter         = \l__block_counter_tl ,
  item-label      = \l__block_item_label_tl ,
  start           = \l__block_counter_start_int ,
  resume          = \l__block_resume_bool ,
  item-instance   = \__block_item_instance:n ,
  item-vspace     = \l__block_item_vspace_skip ,
  item-penalty    = \l__block_item_penalty_int ,
  item-indent     = \l__block_item_indent_dim ,
  label-width     = \l__block_label_width_dim ,
  label-sep       = \l__block_label_sep_dim ,
  legacy-support  = \l__block_legacy_support_bool ,
}
{ \SetKnownTemplateKeys{list}{std}{#1} }
\tl_new:N \l__block_item_align_tl
\bool_new:N \l__block_label_strut_bool
\bool_new:N \l__block_label_boxed_bool
\bool_new:N \l__block_next_line_bool
\tl_new:N \l__block_text_font_tl
\bool_new:N \l__block_item_compatibility_bool
\DeclareTemplateCode{item}{std}{1}
  {
    counter-label   = \__block_counter_label:n ,
    counter-ref     = \__block_counter_ref:n ,
    label-ref       = \__block_label_ref:n ,
    label-autoref   = \__block_label_autoref:n ,
    label-format    = \__block_label_format:n ,
    label-strut     = \l__block_label_strut_bool ,
    label-boxed     = \l__block_label_boxed_bool ,
    next-line       = \l__block_next_line_bool ,
    text-font       = \l__block_text_font_tl ,
    compatibility   = \l__block_item_compatibility_bool ,
    label-align     = {
      left    = \tl_set:Nn \l__block_item_align_tl { \relax \hss } ,
      center  = \tl_set:Nn \l__block_item_align_tl { \hss \hss } ,
      right   = \tl_set:Nn \l__block_item_align_tl { \hss \relax } ,
      parleft = \tl_set:Nn \l__block_item_align_tl { \relax \hss } ,
    } ,
  }
  { \SetKnownTemplateKeys{item}{std}{#1} }
\cs_new:Npn \endblockenv { \par }
\newcounter{maxblocklevels}
\setcounter{maxblocklevels}{6}
\DeclareInstance{blockenv}{displayblock}{display}{ name = displayblock, increment-level = false }
\DeclareInstance{blockenv}{displayblockflattened}{display}{ name = displayblockflattened, increment-level = false }
\DeclareInstance{blockenv}{center}{display}{ name = center }
\DeclareInstance{blockenv}{flushleft}{display}{ name = flushleft }
\DeclareInstance{blockenv}{flushright}{display}{ name = flushright }
\DeclareInstance{blockenv}{quotation}{display}{ name = quotation }
\DeclareInstance{blockenv}{quote}{display}{ name = quote }
\DeclareInstance{blockenv}{theorem}{display}{ name = theorem }
\DeclareInstance{blockenv}{verbatim}{display}{ name = verbatim }
\DeclareInstance{blockenv}{verbatim*}{display}{ name = verbatim* }
\DeclareInstance{blockenv}{itemize}{display}{ name = itemize, block-instance = listblock, inner-instance = itemize }
\DeclareInstance{blockenv}{enumerate}{display}{ name = enumerate, block-instance = listblock, inner-instance = enum }
\DeclareInstance{blockenv}{description}{display}{ name = description, block-instance = listblock, inner-instance = description }
\DeclareInstance{blockenv}{list}{display}{ name = list, block-instance = listblock, inner-instance = legacy }
\DeclareInstance{block}{displayblock-0}{display}{}
\DeclareInstance{block}{listblock-0}{display}{}
\DeclareInstance{list}{itemize-1}{std}{ item-label = \labelitemi }
\DeclareInstance{list}{itemize-2}{std}{ item-label = \labelitemii }
\DeclareInstance{list}{itemize-3}{std}{ item-label = \labelitemiii }
\DeclareInstance{list}{itemize-4}{std}{ item-label = \labelitemiv }
\DeclareInstance{list}{enum-1}{std}{ counter = enumi, item-label = \labelenumi }
\DeclareInstance{list}{enum-2}{std}{ counter = enumii, item-label = \labelenumii }
\DeclareInstance{list}{enum-3}{std}{ counter = enumiii, item-label = \labelenumiii }
\DeclareInstance{list}{enum-4}{std}{ counter = enumiv, item-label = \labelenumiv }
\DeclareInstance{list}{legacy}{std}{ legacy-support = true }
\DeclareInstance{list}{description}{std}{ item-instance = description }
\DeclareInstance{item}{basic}{std}{ label-align = right }
\DeclareInstance{item}{description}{std}{ label-format = \normalfont\bfseries #1 , label-align = left }
\DeclareInstance{para}{center}{std}{ para-attr-class = center }
\DeclareInstance{para}{raggedright}{std}{ para-attr-class = raggedright }
\DeclareInstance{para}{raggedleft}{std}{ para-attr-class = raggedleft }
\DeclareInstance{para}{justify}{std}{ para-attr-class = justify }
\ExplSyntaxOff");
});

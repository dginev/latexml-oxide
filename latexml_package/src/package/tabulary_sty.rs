use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: tabulary.sty.ltxml
  RequirePackage!("array");

  // \tabulary{Dimension}[]{} — Perl L22/L24 both carry `locked => 1`,
  // which keeps user-level \renewcommand or a later package's
  // redefinition from replacing the tabulary→alignment trampoline
  // (the raw tabulary.sty itself does this; the Perl lock prevents us
  // from losing the binding when the raw sty is loaded alongside).
  DefMacro!("\\tabulary{}[]{}",
    "\\@tabular@bindings{#3}[vattach=#2,width=#1]\\@@tabulary{#1}[#2]{#3}\\lx@begin@alignment",
    locked => true);
  DefMacro!("\\endtabulary",
    "\\lx@end@alignment\\@end@tabulary",
    locked => true);
  DefPrimitive!(T_CS!("\\@end@tabulary"), None, { stomach::egroup()?; });
  DefConstructor!("\\@@tabulary{Dimension}[] Undigested DigestedBody",
    "#4",
    reversion => "\\begin{tabulary}{#1}[#2]{#3}#4\\end{tabulary}",
    before_digest => { stomach::bgroup(); },
    mode => "restricted_horizontal");

  // Like l,c,r,j, but set like p w/o explicit width...
  DefColumnType!("L", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\vtop"), T_BEGIN!())),
        after: Some(Tokens!(T_END!())),
        align: Some(latexml_core::alignment::template::Align::Left),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("C", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\vtop"), T_BEGIN!())),
        after: Some(Tokens!(T_END!())),
        align: Some(latexml_core::alignment::template::Align::Center),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("R", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\vtop"), T_BEGIN!())),
        after: Some(Tokens!(T_END!())),
        align: Some(latexml_core::alignment::template::Align::Right),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("J", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\vtop"), T_BEGIN!())),
        after: Some(Tokens!(T_END!())),
        align: Some(latexml_core::alignment::template::Align::Justify),
        ..Cell::default()
      })
    });
  });

  // stub in for macros that try to redefine it.
  DefMacro!("\\TY@tabular", "\\relax");
});

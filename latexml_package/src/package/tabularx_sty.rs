use crate::prelude::*;

LoadDefinitions!({
  RequirePackage!("array");

  // \tabularx{Dimension}[]{}
  DefMacro!(
    "\\tabularx{}[]{}",
    "\\@tabular@bindings{#3}[vattach=#2,width=#1]\\@@tabularx{#1}[#2]{#3}\\lx@begin@alignment"
  );
  DefMacro!("\\endtabularx", "\\lx@end@alignment\\@end@tabularx");
  DefPrimitive!(T_CS!("\\@end@tabularx"), None, {
    stomach::egroup()?;
  });
  DefConstructor!("\\@@tabularx{Dimension}[] Undigested DigestedBody",
    "#4",
    reversion => "\\begin{tabularx}{#1}[#2]{#3}#4\\end{tabularx}",
    before_digest => { stomach::bgroup(); },
    mode => "restricted_horizontal");

  // Like p, but w/o explicit width...
  DefColumnType!("X", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\vtop"), T_BEGIN!())),
        after: Some(Tokens!(T_END!())),
        align: Some(latexml_core::alignment::template::Align::Justify),
        ..Cell::default()
      })
    });
  });
});

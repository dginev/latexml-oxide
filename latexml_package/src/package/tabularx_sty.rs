use crate::{engine::tex_tables::restore_dollar_outside_alignment, prelude::*};

LoadDefinitions!({
  RequirePackage!("array");

  // \tabularx{Dimension}[]{}
  // Inside the environment `X` is tabularx's own column whatever the document made of it: tabularx.sty's
  // `\TX@endtabularx` runs `\TX@newcol` = `\newcol@{X}[0]{p{\TX@col@width}}` (tabularx.sty:90, :157-158) before it
  // reads the preamble, so a document's `\newcolumntype{X}[1]{…p{#1}}` does not reach it (2606.05563: `|l|c|X|` read
  // `|` as the width of every X cell; KNOWN_PERL_ERRORS #463).
  DefMacro!(
    "\\tabularx{}[]{}",
    "\\let\\NC@rewrite@X\\lx@tabularx@X\\@tabular@bindings{#3}[vattach=#2,width=#1]\\@@tabularx{#1}[#2]{#3}\\lx@begin@alignment"
  );
  DefMacro!("\\endtabularx", "\\lx@end@alignment\\@end@tabularx");
  DefPrimitive!(T_CS!("\\@end@tabularx"), None, {
    egroup()?;
    restore_dollar_outside_alignment();
  });
  DefConstructor!("\\@@tabularx{Dimension}[] Undigested DigestedBody",
    "#4",
    reversion => "\\begin{tabularx}{#1}[#2]{#3}#4\\end{tabularx}",
    before_digest => { bgroup(); },
    mode => "restricted_horizontal");

  // Like p, but w/o explicit width...
  DefColumnType!("X", {
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens!(
          T_CS!("\\vtop"),
          T_BEGIN!(),
          T_CS!("\\lx@restore@interline")
        )),
        after: Some(Tokens!(T_END!())),
        align: Some(Align::Justify),
        ..Cell::default()
      })
    });
  });
  Let!("\\lx@tabularx@X", "\\NC@rewrite@X");
});

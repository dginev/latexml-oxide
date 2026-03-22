/// Perl: wrapfig.sty.ltxml — wrapfigure/wraptable environments
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // {wrapfigure} [Number] {} [Dimension] {Dimension}
  DefEnvironment!("{wrapfigure}[Number]{} [Dimension] {Dimension}",
    "<ltx:figure xml:id='#id' inlist='#inlist' float='#float'>#tags#body</ltx:figure>",
    mode => "internal_vertical",
    before_digest => {
      crate::engine::latex_ch9_figures_and_tables::before_float("figure", None);
    },
    after_digest_begin => sub[whatsit] {
      let dir = whatsit.get_arg(2).map(|a| a.to_attribute()).unwrap_or_default();
      let float_val = match dir.as_str() {
        "r" | "R" => "right",
        "l" | "L" => "left",
        _ => "",
      };
      if !float_val.is_empty() {
        whatsit.set_property("float", Stored::from(float_val));
      }
    },
    after_digest => sub[whatsit] {
      crate::engine::latex_ch9_figures_and_tables::after_float(whatsit);
    });

  // {wraptable} [Number] {} [Dimension] {Dimension}
  DefEnvironment!("{wraptable}[Number]{} [Dimension] {Dimension}",
    "<ltx:table xml:id='#id' inlist='#inlist' float='#float'>#tags#body</ltx:table>",
    mode => "internal_vertical",
    before_digest => {
      crate::engine::latex_ch9_figures_and_tables::before_float("table", None);
    },
    after_digest_begin => sub[whatsit] {
      let dir = whatsit.get_arg(2).map(|a| a.to_attribute()).unwrap_or_default();
      let float_val = match dir.as_str() {
        "r" | "R" => "right",
        "l" | "L" => "left",
        _ => "",
      };
      if !float_val.is_empty() {
        whatsit.set_property("float", Stored::from(float_val));
      }
    },
    after_digest => sub[whatsit] {
      crate::engine::latex_ch9_figures_and_tables::after_float(whatsit);
    });

  DefMacro!("\\WFclear", "\\par");
  DefRegister!("\\wrapoverhang", Dimension!("0pt"));
});

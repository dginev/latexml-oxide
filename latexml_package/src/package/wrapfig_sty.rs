use crate::engine::latex_constructs::{after_float, before_float};
use crate::prelude::*;

// wrapfig.sty — wrapping figures/tables around text
LoadDefinitions!({
  DefEnvironment!("{wrapfigure} [Number] {} [Dimension] {Dimension}",
    "<ltx:figure xml:id='#id' inlist='#inlist' float='#float'>#tags#body</ltx:figure>",
    mode => "internal_vertical",
    after_digest_begin => sub[whatsit] {
      let dir = whatsit.get_arg(2).map(|a| a.to_string()).unwrap_or_default();
      // Perl wrapfig.sty.ltxml L29/L43: `eq 'r'` / `eq 'l'` — exact
      // case-sensitive match. Prior Rust also matched `R`/`L` (uppercase);
      // drop per feedback_case_folding_parity.
      let float_val = match dir.trim() {
        "r" => "right",
        "l" => "left",
        _ => "",
      };
      if !float_val.is_empty() {
        whatsit.set_property("float", Stored::String(arena::pin(float_val)));
      }
    },
    before_digest => { before_float("figure", None) },
    after_digest => sub[whatsit] { after_float(whatsit); Ok(Vec::new()) }
  );

  DefEnvironment!("{wraptable} [Number] {} [Dimension] {Dimension}",
    "<ltx:table xml:id='#id' inlist='#inlist' float='#float'>#tags#body</ltx:table>",
    mode => "internal_vertical",
    after_digest_begin => sub[whatsit] {
      let dir = whatsit.get_arg(2).map(|a| a.to_string()).unwrap_or_default();
      // Perl wrapfig.sty.ltxml L29/L43: `eq 'r'` / `eq 'l'` — exact
      // case-sensitive match. Prior Rust also matched `R`/`L` (uppercase);
      // drop per feedback_case_folding_parity.
      let float_val = match dir.trim() {
        "r" => "right",
        "l" => "left",
        _ => "",
      };
      if !float_val.is_empty() {
        whatsit.set_property("float", Stored::String(arena::pin(float_val)));
      }
    },
    before_digest => { before_float("table", None) },
    after_digest => sub[whatsit] { after_float(whatsit); Ok(Vec::new()) }
  );

  DefMacro!("\\WFclear", "\\par");
  DefRegister!("\\wrapoverhang", Dimension!("0pt"));
});

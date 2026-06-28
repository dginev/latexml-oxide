use crate::{
  engine::latex_constructs::{after_float, before_float},
  prelude::*,
};

// wrapfig.sty — wrapping figures/tables around text
LoadDefinitions!({
  DefEnvironment!("{wrapfigure} [Number] {} [Dimension] {Dimension}",
    "<ltx:figure xml:id='#id' inlist='#inlist' float='#float' ?#width(width='#width')>#tags#body</ltx:figure>",
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
        whatsit.set_property("float", Stored::String(pin(float_val)));
      }
      // INTENTIONAL DIVERGENCE from Perl wrapfig.sty.ltxml (which captures the
      // mandatory {Dimension} wrap width as arg 4 but then DISCARDS it): emit it
      // as the figure @width (→ base-styling `width:`) so the float — image AND
      // caption — is capped to the declared wrap width instead of expanding to
      // fit a single-line caption (which leaves a small figure in an enormous
      // box). Mirrors the {minipage} width idiom. See OXIDIZED_DESIGN.
      set_wrap_width(whatsit);
    },
    before_digest => { before_float("figure", None) },
    after_digest => sub[whatsit] { after_float(whatsit); Ok(Vec::new()) }
  );

  DefEnvironment!("{wraptable} [Number] {} [Dimension] {Dimension}",
    "<ltx:table xml:id='#id' inlist='#inlist' float='#float' ?#width(width='#width')>#tags#body</ltx:table>",
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
        whatsit.set_property("float", Stored::String(pin(float_val)));
      }
      // Same intentional divergence as {wrapfigure}: cap the float to the
      // declared wrap width (Perl discards it). See OXIDIZED_DESIGN.
      set_wrap_width(whatsit);
    },
    before_digest => { before_float("table", None) },
    after_digest => sub[whatsit] { after_float(whatsit); Ok(Vec::new()) }
  );

  DefMacro!("\\WFclear", "\\par");
  DefRegister!("\\wrapoverhang", Dimension!("0pt"));
});

// Emit the wrapfig/wraptable mandatory {Dimension} (arg 4 = the declared wrap
// width) as the float's `width` property so it renders as a CSS `width:`,
// capping the float (image + caption) to that width. Perl's wrapfig binding
// discards this arg; applying it is an intentional ar5iv-rendering divergence.
//
// Emit it as a PERCENTAGE of \textwidth, not the resolved absolute pt. A
// wrapfigure width is a layout reservation -- the fraction of the line the float
// occupies (authors write it proportionally, e.g. 0.48\linewidth). In print the
// column is fixed so absolute == relative; in responsive HTML the main column is
// far wider than the print \textwidth, so an absolute pt renders far narrower
// than the intended fraction (0.48\linewidth -> ~165pt -> ~26% of an 832px
// column instead of ~48%). A percentage preserves the proportion and scales.
// This also makes wrapfig consistent with the floatfig/floatflt sibling
// packages, which already emit `100 * dimen / \textwidth . '%'` (Perl toPercent).
fn set_wrap_width(whatsit: &mut Whatsit) {
  let Some(width_arg) = whatsit.get_arg(4) else { return };
  let v = width_arg.value_of();
  if v == 0 {
    return;
  }
  let Some(tw) = lookup_dimension("\\textwidth") else { return };
  let tw_sp = tw.value_of();
  if tw_sp == 0 {
    return;
  }
  let pct = (100 * v) / tw_sp;
  whatsit.set_property("width", Stored::from(s!("{pct}%")));
}

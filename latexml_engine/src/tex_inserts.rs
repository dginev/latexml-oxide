//! TeX Inserts
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Inserts Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // Inserting material
  //----------------------------------------------------------------------
  // \insert           c  places material into an insertions class.
  // \insert<8bit><filler>{<vertical mode material>}
  DefPrimitive!("\\insert Number", None);

  //======================================================================
  // Splitting a box
  //----------------------------------------------------------------------
  // \vsplit c removes a specified amount of material from a box register .
  // \splitbotmark c is the mark text of the last mark in the most recent \vsplit operation .
  // \splitfirstmark c is the mark text of the first mark in the most recent \vsplit operation .
  // tex.web §977-979: `\vsplit N to D` removes the top D-worth of material
  // from box N, RETURNS it, and leaves the REMAINDER in the register (void
  // once exhausted). Perl returns the whole box and leaves the register
  // untouched — so the classic drain idiom
  //   \def\do{\setbox2=\vsplit0 to\dimen@ … \ifdim\ht0>\z@ \expandafter\do\fi}
  // (short-math-guide's column splitter, 6 docs of the Stomach:Recursion
  // family) both duplicates content every pass AND never terminates.
  // We approximate vert_break at BOX BOUNDARIES: accumulate top-level items
  // until the split height is exceeded (always taking at least one item so
  // progress is guaranteed), return them as the split-off part, and store
  // the remainder back (void when empty) — faithful in the limit, no
  // glue/penalty breakpoints.
  DefPrimitive!("\\vsplit Number Match:to Dimension", sub[(number,_to,dimension)] {
    let box_key   = s!("box{}", number.value_of());
    match lookup_value(&box_key) { Some(Stored::Digested(stuff)) => {
      adjust_box_color(&stuff)?;
      // tex.web §977: `box(n):=null` / `box(n):=vpack(q)` store at the
      // register's EXISTING eq_level ("the eq_level of the box stays the
      // same") — no save-stack entry, so the drain survives the caller's
      // group. eledmac.sty:1363-1369 `\do@line` splits one line per pass
      // inside `{…\global\setbox\one@line=\vsplit\raw@text to\baselineskip}`
      // and its `\loop\ifvbox\raw@text` (L1304) ends only when the register
      // goes void; a local store was undone by the `}` every pass (eledform
      // example: box-list memory runaway). `Scope::InPlace` is the analog.
      // Guard: `perfect_kernel_batch54::vsplit_drain_survives_the_enclosing_group`.
      if stuff.is_empty()? {
        assign_value(&box_key, Stored::None, Some(Scope::InPlace));
        Digested::from(List::default())
      } else {
        let target = dimension.value_of();
        let items: Vec<Digested> = match stuff.data() {
          DigestedData::List(l) => match l.try_borrow() {
            Ok(l) => l.boxes.clone(),
            Err(_) => vec![stuff.clone()],
          },
          _ => vec![stuff.clone()],
        };
        let mut split_off: Vec<Digested> = Vec::new();
        let mut rest: Vec<Digested> = Vec::new();
        let mut used: i64 = 0;
        for item in items {
          if !rest.is_empty() {
            rest.push(item);
            continue;
          }
          let (_w, h, d) = item.compute_size(Default::default())?;
          let item_v = h.value_of() + d.value_of();
          if split_off.is_empty() || used + item_v <= target {
            used += item_v;
            split_off.push(item);
          } else {
            rest.push(item);
          }
        }
        if rest.is_empty() {
          assign_value(&box_key, Stored::None, Some(Scope::InPlace));
        } else {
          assign_value(
            &box_key,
            Stored::Digested(Digested::from(List::new(rest))),
            Some(Scope::InPlace),
          );
        }
        Digested::from(List::new(split_off))
      }
    } _ => {
      Digested::from(List::default())
    }}
  });
  DefMacro!(T_CS!("\\splitfirstmark"), None, Tokens!());
  DefMacro!(T_CS!("\\splitbotmark"), None, Tokens!());

  //======================================================================
  // Insertion parameters
  //----------------------------------------------------------------------
  // \insertpenalties  iq is a quantity used by TeX in two different ways.
  // \splitmaxdepth    pd is the maximum depth of boxes created by \vsplit.
  // \splittopskip     pg is special glue placed inside the box created by \vsplit.
  // \holdinginserts   pi is positive if insertions should remain dormant when \output is called.
  DefRegister!("\\insertpenalties", Number!(0));
  DefRegister!("\\splitmaxdepth", Dimension!("16383.99999pt"));
  DefRegister!("\\splittopskip", Glue!("10pt"));
  DefRegister!("\\holdinginserts", Number!(0));
});

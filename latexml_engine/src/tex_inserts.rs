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
  // `vert_break` (§970-974) breaks only at glue that follows a non-discardable item, at a kern
  // that glue follows, or at a penalty, and takes the LAST such break whose piece fits the split
  // height (none fitting: the first; §974 `awful_bad`). LaTeXML's vertical lists carry no
  // interline glue, so a box — or any item with a size — after a non-discardable item other than
  // a rule stands for the interline glue before it (and a kern before one is a break); glue at the top
  // of the box or after other glue is no break, nor is a zero-size item that is no box (the `\par`
  // of an `\endgraf`, an anchor), which stays with the piece before it — split off as its own last
  // piece it gave reledmac's `\do@line` an empty numbered line after every wrapped `\pstart`
  // paragraph. The remainder loses the glue and kerns at its top (§968 `prune_page_top`) and is
  // stored back, void when empty (§977). Residuals: `\penalty` leaves no item, so it is no break;
  // a paragraph is one item, never split into its lines (reledmac numbers a wrapped `\pstart`
  // once); no `\splittopskip`. Perl's `\vsplit` returns the whole box and never empties the
  // register (TeX_Inserts.pool.ltxml:36-40, KNOWN_PERL_ERRORS #402). Guard
  // `perfect_kernel_batch58::vsplit_breaks_only_where_tex_can`.
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
        // A register filled by `\setbox0=\vbox{…}` holds the `\vbox` WHATSIT,
        // whose vertical list is its `content_box` (tex_box.rs after_digest);
        // splitting the whatsit itself as one opaque item swept the whole
        // box into the first split-off and voided the register — the
        // discard-top idiom `\setbox2=\vsplit0 to\baselineskip` then threw
        // away every line (short-math-guide's amssymb tables, 361 words,
        // zero errors). tex.web §977 splits the box's list; the remainder
        // is stored back as the same kind of box (`vpack(q)`).
        let (items, vbox): (Vec<Digested>, Option<Whatsit>) = match stuff.data() {
          DigestedData::List(l) => match l.try_borrow() {
            Ok(l) => (l.boxes.clone(), None),
            Err(_) => (vec![stuff.clone()], None),
          },
          DigestedData::Whatsit(w) => match w.try_borrow() {
            Ok(w) if w.get_property_bool("is_vbox") => {
              let items = match w.get_arg(2).map(|c| (c.clone(), c.data())) {
                Some((_, DigestedData::List(l))) => match l.try_borrow() {
                  Ok(l) => l.boxes.clone(),
                  Err(_) => Vec::new(),
                },
                Some((c, _)) => vec![c],
                None => Vec::new(),
              };
              (items, Some(w.clone()))
            },
            _ => (vec![stuff.clone()], None),
          },
          _ => (vec![stuff.clone()], None),
        };
        let glue = |item: &Digested| item.get_property_bool("isVerticalSpace");
        let discardable = |item: &Digested| glue(item) || item.get_property_bool("isKern");
        let defined_by = |item: &Digested, names: &[&str]| match item.data() {
          DigestedData::Whatsit(w) => w.try_borrow().is_ok_and(|w| {
            let definition = w.get_definition();
            names.contains(&definition.get_cs_name().as_ref())
          }),
          _ => false,
        };
        let boxlike = |item: &Digested| defined_by(item, &["\\hbox", "\\vbox", "\\vtop"]);
        // A rule is no breakpoint, and no interline glue follows one (§1056: `prev_depth` becomes
        // `ignore_depth`), so the box after it is none either.
        let rule = |item: &Digested| defined_by(item, &["\\hrule"]);
        // Each item's vertical size and whether it has any size at all, computed as the break
        // search reaches it (one item ahead for a kern): a drain loop sizes each item about once.
        let mut sizes = ItemSizes { items: &items, known: Vec::new() };
        let is_breakpoint = |i: usize, sizes: &mut ItemSizes| -> Result<bool> {
          let after_material = i > 0 && !discardable(&items[i - 1]);
          Ok(if glue(&items[i]) {
            after_material
          } else if items[i].get_property_bool("isKern") {
            // a kern before glue — or before a box, which stands for its interline glue
            i + 1 < items.len()
              && (glue(&items[i + 1])
                || (!rule(&items[i + 1]) && (boxlike(&items[i + 1]) || sizes.get(i + 1)?.1)))
          } else {
            after_material
              && !rule(&items[i - 1])
              && !rule(&items[i])
              && (boxlike(&items[i]) || sizes.get(i)?.1)
          })
        };
        // §974: the piece ends at the last breakpoint whose piece fits; none fitting, at the
        // first breakpoint (`awful_bad`), else the whole list.
        let mut end = items.len();
        let mut last_fit: Option<usize> = None;
        let mut used: i64 = 0;
        for i in 0..items.len() {
          if i > 0 && is_breakpoint(i, &mut sizes)? {
            if used > target {
              end = last_fit.unwrap_or(i);
              break;
            }
            last_fit = Some(i);
          }
          used += sizes.get(i)?.0;
        }
        if end == items.len()
          && used > target
          && let Some(fit) = last_fit
        {
          end = fit;
        }
        let mut split_off = items;
        let mut rest = split_off.split_off(end);
        let pruned = rest.iter().take_while(|item| discardable(item)).count();
        rest.drain(..pruned);
        if rest.is_empty() {
          assign_value(&box_key, Stored::None, Some(Scope::InPlace));
        } else {
          let remainder = match vbox {
            Some(mut w) => {
              // `vpack(q)`: the same box with the remaining list.
              if let Some(arg) = w.get_arg_mut(2) {
                *arg = Digested::from(List::new(rest));
              }
              w.set_property("content_box", w.get_arg(2).cloned());
              Digested::from(w)
            },
            None => Digested::from(List::new(rest)),
          };
          assign_value(&box_key, Stored::Digested(remainder), Some(Scope::InPlace));
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

/// The vertical sizes of a `\vsplit` list's items, computed on first use: each item's height plus
/// depth, and whether it has any size at all.
struct ItemSizes<'a> {
  items: &'a [Digested],
  known: Vec<(i64, bool)>,
}

impl ItemSizes<'_> {
  fn get(&mut self, i: usize) -> Result<(i64, bool)> {
    while self.known.len() <= i {
      let (w, h, d) = self.items[self.known.len()].compute_size(Default::default())?;
      let v = h.value_of() + d.value_of();
      self.known.push((v, v != 0 || w.value_of() != 0));
    }
    Ok(self.known[i])
  }
}

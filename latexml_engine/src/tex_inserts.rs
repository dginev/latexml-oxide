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
  // stored back, void when empty (§977). A penalty below 10000 is a break, at the top of the box
  // too, and one of -10000 or less ends the piece there (§974 `eject_penalty`: short-math-guide's
  // `\null\penalty-\@M` gives the `\null` alone, its first symbol row was thrown away with it);
  // a penalty is discardable, so the box after `\nobreak` is no break (§970). The heights follow
  // §970-977 below: interline glue, `\splitmaxdepth`, the piece packed to the split height and
  // `\splittopskip` atop the remainder. Residuals: a paragraph is one item, never split into its
  // lines (reledmac numbers a wrapped `\pstart` once); the piece is a list, not a vbox (R2); no
  // stretch, shrink or penalty costs (§974-975: the last fitting break wins), and `\vfil` leaves no
  // item; `\nointerlineskip` (`\prevdepth`) leaves none either, so interline glue is assumed there;
  // a paragraph joins the box above with its whole height, not its first line's.
  // Perl's `\vsplit` returns the whole box and never empties the
  // register (TeX_Inserts.pool.ltxml:36-40, KNOWN_PERL_ERRORS #402). Guards
  // `perfect_kernel_batch58::vsplit_breaks_only_where_tex_can`, `box_primitives::vsplit`.
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
      // Guard: `box_primitives::vsplit`.
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
        // The interline parameters the box was built with (base_utilities.rs `predigest_box_contents_in_mode`).
        let recorded = |l: &List| -> [Option<i64>; 3] {
          ["baseline", "lineskip", "lineskiplimit"].map(|key| match l.properties.get(key) {
            Some(Stored::Dimension(d)) => Some(d.value_of()),
            _ => None,
          })
        };
        let (items, vbox, interline): (Vec<Digested>, Option<Whatsit>, [Option<i64>; 3]) =
          match stuff.data() {
            DigestedData::List(l) => match l.try_borrow() {
              Ok(l) => (l.boxes.clone(), None, recorded(&l)),
              Err(_) => (vec![stuff.clone()], None, [None; 3]),
            },
            DigestedData::Whatsit(w) => match w.try_borrow() {
              Ok(w) if w.get_property_bool("is_vbox") => {
                let (items, interline) = match w.get_arg(2).map(|c| (c.clone(), c.data())) {
                  Some((_, DigestedData::List(l))) => match l.try_borrow() {
                    Ok(l) => (l.boxes.clone(), recorded(&l)),
                    Err(_) => (Vec::new(), [None; 3]),
                  },
                  Some((c, _)) => (vec![c], [None; 3]),
                  None => (Vec::new(), [None; 3]),
                };
                (items, Some(w.clone()), interline)
              },
              _ => (vec![stuff.clone()], None, [None; 3]),
            },
            _ => (vec![stuff.clone()], None, [None; 3]),
          };
        // A vertical kern is vertical space too, but only glue is a breakpoint after material (§973).
        let glue = |item: &Digested| {
          item.get_property_bool("isVerticalSpace") && !item.get_property_bool("isKern")
        };
        let penalty = |item: &Digested| {
          item.get_property_bool("isPenalty").then(|| match item.get_property("penalty").as_deref() {
            Some(Stored::Int(value)) => *value,
            _ => 0,
          })
        };
        let discardable = |item: &Digested| {
          glue(item) || item.get_property_bool("isKern") || item.get_property_bool("isPenalty")
        };
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
        // Each item's height and depth and whether it has any size at all, computed as the break
        // search reaches it (one item ahead for a kern): a drain loop sizes each item about once.
        let mut sizes = ItemSizes { items: &items, known: Vec::new() };
        // `interline`: a box before, so interline glue precedes the next box (none at the top of the list or after a
        // rule, §1056, §1083 `ignore_depth`).
        let is_breakpoint = |i: usize, interline: bool, sizes: &mut ItemSizes| -> Result<bool> {
          let after_material = i > 0 && !discardable(&items[i - 1]);
          Ok(if let Some(value) = penalty(&items[i]) {
            value < 10000
          } else if glue(&items[i]) {
            after_material
          } else if items[i].get_property_bool("isKern") {
            // a kern before glue — or before a box, which stands for its interline glue
            i + 1 < items.len()
              && (glue(&items[i + 1])
                || (interline
                  && !rule(&items[i + 1])
                  && (boxlike(&items[i + 1]) || sizes.get(i + 1)?.2)))
          } else {
            after_material
              && interline
              && !rule(&items[i])
              && (boxlike(&items[i]) || sizes.get(i)?.2)
          })
        };
        // A box or rule of the list: what `cur_height` and the interline glue measure.
        let is_box = |i: usize, sizes: &mut ItemSizes| -> Result<bool> {
          Ok(rule(&items[i]) || boxlike(&items[i]) || (!discardable(&items[i]) && sizes.get(i)?.2))
        };
        // §970-976 `vert_break`: `cur_height` runs to the baseline of the last box and `prev_dp` is that box's depth,
        // at most `\splitmaxdepth` (§976); glue and kerns add their size. LaTeXML's lists carry no interline glue, so
        // a box after a box adds it as §679 `append_to_vlist` made it, with the box's own parameters: `\baselineskip`
        // less the depth above and the height below, `\lineskip` when that is under `\lineskiplimit`; glue in between keeps the depth above, a rule
        // cancels it (`ignore_depth`, §1056). The piece ends at the last breakpoint whose piece fits (§974-975: with
        // no stretch an underfull break costs `deplorable`, and the later of equal costs wins); none fitting, at the
        // first breakpoint (`awful_bad`), else the whole list. Summing the items alone put three lines in a piece of
        // `2.5\baselineskip`, where TeX puts two.
        let register = |name: &str| lookup_register_quiet(name).map_or(0, |value| value.value_of());
        let [baselineskip, lineskip, lineskiplimit] = interline;
        let baselineskip = baselineskip.unwrap_or_else(|| register("\\baselineskip"));
        let lineskip = lineskip.unwrap_or_else(|| register("\\lineskip"));
        // As the sizing (font.rs): a list that recorded no limit is stacked with LaTeX's 0pt.
        let lineskiplimit = lineskiplimit.unwrap_or(0);
        let max_depth = register("\\splitmaxdepth");
        let mut end = items.len();
        let mut last_fit: Option<usize> = None;
        let (mut cur_height, mut prev_dp): (i64, i64) = (0, 0);
        let mut depth_above: Option<i64> = None;
        for (i, item) in items.iter().enumerate() {
          let interline = depth_above.is_some();
          if (i > 0 || penalty(item).is_some()) && is_breakpoint(i, interline, &mut sizes)? {
            if cur_height > target {
              end = last_fit.unwrap_or(i);
              break;
            }
            last_fit = Some(i);
            if penalty(item).is_some_and(|value| value <= -10000) {
              end = i;
              break;
            }
          }
          let (height, depth, _) = sizes.get(i)?;
          if glue(item) || item.get_property_bool("isKern") {
            cur_height += prev_dp + height + depth;
            prev_dp = 0;
          } else if is_box(i, &mut sizes)? {
            if let Some(above) = depth_above.filter(|_| !rule(item)) {
              let gap = baselineskip - above - height;
              cur_height += prev_dp + if gap >= lineskiplimit { gap } else { lineskip };
              prev_dp = 0;
            }
            cur_height += prev_dp + height;
            prev_dp = depth;
            depth_above = (!rule(item)).then_some(depth);
            if prev_dp > max_depth {
              cur_height += prev_dp - max_depth;
              prev_dp = max_depth;
            }
          }
        }
        if end == items.len()
          && cur_height > target
          && let Some(fit) = last_fit
        {
          end = fit;
        }
        // §977 `vpackage(p, h, exactly, split_max_depth)`: the piece is exactly the split height, as deep as its last
        // box (0pt under glue or a kern after it), at most `\splitmaxdepth`.
        let mut piece_depth = 0;
        for j in (0..end).rev() {
          if glue(&items[j]) || items[j].get_property_bool("isKern") {
            break;
          }
          if is_box(j, &mut sizes)? {
            piece_depth = sizes.get(j)?.1.min(max_depth);
            break;
          }
        }
        // §968 `prune_page_top`: the remainder loses the glue, kerns and penalties above its first box or rule (marks
        // and whatsits stay), and `\splittopskip` glue goes before that box, less its height (§969).
        let mut first_box = None;
        let mut pruned = Vec::new();
        for (j, item) in items.iter().enumerate().skip(end) {
          if is_box(j, &mut sizes)? {
            first_box = Some(j);
            break;
          }
          if discardable(item) {
            pruned.push(j);
          }
        }
        let top_skip = match first_box {
          Some(j) => {
            let skip = lookup_register_quiet("\\splittopskip").map_or(0, |value| value.value_of());
            Some((j, (skip - sizes.get(j)?.0).max(0)))
          },
          None => None,
        };
        let mut split_off = items;
        let tail = split_off.split_off(end);
        let mut rest = Vec::with_capacity(tail.len() + 1);
        for (j, item) in (end..).zip(tail) {
          if pruned.contains(&j) {
            continue;
          }
          if let Some((first, skip)) = top_skip
            && first == j
          {
            rest.push(split_top_skip(skip));
          }
          rest.push(item);
        }
        if rest.is_empty() {
          assign_value(&box_key, Stored::None, Some(Scope::InPlace));
        } else {
          // The remaining list keeps what its list recorded (mode, interline parameters), not its size (§977
          // `vpack(q, natural)`: a re-split piece's remainder is not the piece's height).
          let remaining = |list: Option<&Digested>| {
            let mut remaining = List::new(rest);
            if let Some(DigestedData::List(old)) = list.map(Digested::data)
              && let Ok(old) = old.try_borrow()
            {
              remaining.mode = old.mode.clone();
              remaining.properties = old.properties.clone();
              for key in ["height", "depth", "width", "cached_width", "cached_height", "cached_depth"] {
                remaining.properties.remove(key);
              }
            }
            // `vpack`: the remainder is a vbox — the remainder of a split piece (a list without a mode) too.
            if !remaining.properties.contains_key("mode") {
              remaining
                .properties
                .insert("mode", Stored::String(pin!("internal_vertical")));
            }
            remaining.properties.insert("vattach", Stored::String(pin!("bottom")));
            Digested::from(remaining)
          };
          let remainder = match vbox {
            Some(mut w) => {
              // `vpack(q, natural)`: the same box with the remaining list, at its natural size — not the split
              // box's `to`/`spread` height, a `\wd` set on it, nor its cached size. Residual: a `\vtop`'s remainder
              // is sized as a vbox but still typeset top-attached (the `\vtop` constructor's own attachment).
              let list = remaining(w.get_arg(2));
              if let Some(arg) = w.get_arg_mut(2) {
                *arg = list;
              }
              w.set_property("content_box", w.get_arg(2).cloned());
              for key in ["height", "depth", "width", "cached_width", "cached_height", "cached_depth"] {
                w.get_properties_mut().remove(key);
              }
              Digested::from(w)
            },
            None => remaining(Some(&stuff)),
          };
          assign_value(&box_key, Stored::Digested(remainder), Some(Scope::InPlace));
        }
        // §977 packs the piece as a vbox; here it stays a list, which `\unvbox` (Perl `unlist`, TeX_Box.pool.ltxml:
        // 725-733) unpacks — a vbox whatsit it would not, and reledmac's `\do@line` and short-math-guide's
        // `\vtop{\unvbox2}` columns rely on it. A piece of several lines typeset directly runs them together (RED
        // boxes-groups/box_primitives_vsplit, R2).
        let mut piece = List::new(split_off);
        for (key, value) in ["baseline", "lineskip", "lineskiplimit"].into_iter().zip(interline) {
          if let Some(value) = value {
            piece.properties.insert(key, Stored::Dimension(Dimension::new(value)));
          }
        }
        piece.properties.insert("height", Stored::Dimension(Dimension::new(target)));
        piece.properties.insert("depth", Stored::Dimension(Dimension::new(piece_depth)));
        Digested::from(piece)
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

/// The vertical sizes of a `\vsplit` list's items, computed on first use: each item's height and
/// depth, and whether it has any size at all.
struct ItemSizes<'a> {
  items: &'a [Digested],
  known: Vec<(i64, i64, bool)>,
}

impl ItemSizes<'_> {
  fn get(&mut self, i: usize) -> Result<(i64, i64, bool)> {
    while self.known.len() <= i {
      let item = &self.items[self.known.len()];
      let (w, h, d) = item.compute_size(Default::default())?;
      let (h, d) = (h.value_of(), d.value_of());
      // A paragraph's list is a line however small (tex.web §1096: a paragraph that is not null leaves at least one
      // line box, `\mbox{}\par` and the empty line `end_graf` leaves included).
      let sized = matches!(item.data(), DigestedData::List(_))
        && item
          .get_property("mode")
          .is_some_and(|mode| mode.to_string() == "horizontal")
        || h != 0
        || d != 0
        || w.value_of() != 0;
      self.known.push((h, d, sized));
    }
    Ok(self.known[i])
  }
}

/// §969: the `\splittopskip` glue above a `\vsplit` remainder's first box, `skip` (in sp) its size less the box's
/// height. An empty box of that height, as the vertical penalty items are (tex_penalties.rs).
fn split_top_skip(skip: i64) -> Digested {
  let mut reversion = vec![T_CS!("\\vskip")];
  reversion.extend(ExplodeText!(&Dimension::new(skip).to_string()));
  let mut props = SymHashMap::default();
  props.insert("isVerticalSpace", Stored::Bool(true));
  props.insert("isSkip", Stored::Bool(true));
  props.insert("width", Stored::Dimension(Dimension::default()));
  props.insert("height", Stored::Dimension(Dimension::new(skip)));
  props.insert("depth", Stored::Dimension(Dimension::default()));
  Digested::from(Tbox::new(
    pin!(""),
    None,
    None,
    Tokens::new(reversion),
    props,
  ))
}

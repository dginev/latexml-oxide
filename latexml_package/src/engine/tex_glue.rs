//! TeX Glue
//!
//! Core TeX Implementation for LaTeXML
use crate::prelude::*;
static UNICODE_EM_SPACES: [(f64, char); 7] = [
  // Spaces to fake spacing, with width in ems
  (0.100, '\u{200A}'), // Hair space (thinner than thin space)
  (0.167, '\u{2006}'), // six-per-em
  (0.200, '\u{2009}'), // five-per-em, thin space
  (0.250, '\u{2005}'), // four-per-em, mid space
  (0.333, '\u{2004}'), // three-per-em, thick space
  (0.500, '\u{2002}'), // en-quad, "nut"
  (1.000, '\u{2003}'), // em-quad, "mutton"
];
/// String of spacing chars with width roughly equivalent to $dimen
pub(crate) fn dimension_to_spaces(dimen: Dimension) -> String {
  let fs = lookup_font().unwrap().get_size().unwrap_or(1.0); // 1 em
  let mut ems = dimen.pt_value(None) / fs;
  let mut s = String::default();
  for (w, space_char) in UNICODE_EM_SPACES.into_iter().rev() {
    if ems <= 0.0 {
      break;
    }
    if ems + 0.01 > w {
      let n = ((ems + 0.01) / w).floor() as usize;
      ems -= n as f64 * w;
      for _ in 0..n {
        s.push(space_char);
      }
    }
  }
  s
}

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Glue Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  DefRegister!("\\lx@default@jot", Dimension::from_str("3pt")?);

  //======================================================================
  // Inserting, removing glue
  //----------------------------------------------------------------------
  // \hskip            c  inserts horizontal glue in a horizontal or math list.
  // \vskip            c  inserts vertical glue in a vertical list.
  // \unskip           c  removes a glue item from the current list.

  // \hskip handled similarly to \kern
  // \hskip can be ignored in certain situations...
  DefConstructor!("\\hskip Glue", sub[document, args, _props] {
    unref!(args => length_digested);
    let length = match  length_digested.data() {
      DigestedData::RegisterValue(v) => v.into(),
      _ => Dimension::default()
    };
    let parent = document.get_node();
    if document::with_node_qname(parent, |name| name == "svg:g") {
      // Perl: TeX_Glue.pool.ltxml L71-76
      // SVG translate handling — append translate to parent's transform attribute
      let x = length.px_value(None);
      if x != 0.0 {
        let parent = document.get_node_mut();
        let transform = parent.get_attribute("transform").unwrap_or_default();
        let new_transform = if transform.is_empty() {
          s!("translate({x},0)")
        } else {
          s!("{transform} translate({x},0)")
        };
        parent.set_attribute("transform", &new_transform)?;
      }
    } else if in_svg(document) {
      Warn!("unexpected", "kern", s!("Lost hskip in SVG {length}"));
    } else {
      let spaces = dimension_to_spaces(length);
      document.absorb_string(&spaces, &SymHashMap::default())?;
    }
  },
  // Perl: enterHorizontal => 1
  enter_horizontal => true,
  properties => sub[args] {
    unref!(args => length_digested);
    let width: Dimension = match length_digested.data() {
      DigestedData::RegisterValue(v) => v.into(),
      _ => Dimension::default()
    };
    Ok(stored_map!(
      "width" => width, "isSpace" => true, "isSkip" => true))
  });

  //======================================================================
  // If this is the right solution...
  // then we also should put the desired spacing on a style attribute?!?!?!
  DefConstructor!("\\vskip Glue", sub[document, args, _props] {
    unref!(args => length);
    let pt = length.pt_value(None);
    if pt <= 0.0 {
      // Negative or zero skip: do nothing
    } else if pt < 4.0 && document.is_closeable("ltx:p").is_some() {
      document.close_element("ltx:p")?;
    } else if document.is_closeable("ltx:para").is_some() {
      document.close_element("ltx:para")?;
    }},
    // Perl: leaveHorizontal => 1
    before_digest => { leave_horizontal()?; },
    // Perl: height => $_[1] — stores glue value as height property
    // so getSize() returns it, making \noalign{\vskip X} contribute to row spacing
    properties => sub[args] {
      // The Glue argument arrives as a RegisterValue wrapping the glue.
      // Extract its base dimension for the height property.
      let height: Stored = match args[0].as_ref() {
        Some(d) => {
          if let Some(dim) = d.get_dimension() {
            Stored::Dimension(dim)
          } else {
            // Fallback: try converting via Stored
            Stored::from(d.clone())
          }
        },
        None => Stored::None,
      };
      Ok(stored_map!("isSpace" => true, "isSkip" => true,
        "isVerticalSpace" => true, "isBreak" => true,
        "height" => height))
    }
  );
  // Remove skip, if last on LIST
  DefPrimitive!("\\unskip", {
    let mut comments = Vec::new();
    while let Some(last_box) = pop_box_list() {
      // Scan past any Comment boxes
      if matches!(last_box.data(), DigestedData::Comment(_)) {
        comments.push(last_box);
      } else if last_box.get_property_bool("isSkip") {
        break;
      } else {
        // return a non-skip box to the list.
        push_box_list(last_box);
        break;
      }
    }
    // Restore any comment boxes that were scanned past
    for comment in comments.into_iter().rev() {
      push_box_list(comment);
    }
  });

  //======================================================================
  // Horizontal skips
  //----------------------------------------------------------------------
  // \hfil             d  inserts first order infinitely stretchable horizontal glue in a horizontal
  // or math list. \hfill            d  inserts second order infinitely stretchable horizontal
  // glue in a horizontal or math list. \hfilneg          d  cancels the stretchability of \hfil.
  // \hss              d  inserts infinitely stretchable and shrinkable horizontal glue in a
  // horizontal or math list.
  // Perl: all have enterHorizontal => 1
  DefPrimitive!("\\hss", None, enter_horizontal => true);
  DefPrimitive!("\\hfilneg", None, enter_horizontal => true);
  DefPrimitive!("\\hfil", {
    enter_horizontal();
    Tbox::new(
      arena::pin_static(" "),
      None,
      None,
      Tokens!(T_CS!("\\hfil")),
      stored_map!("isSpace" => true, "isFill" => true),
    )
  });
  DefPrimitive!("\\hfill", {
    enter_horizontal();
    Tbox::new(
      arena::pin_static(" "),
      None,
      None,
      Tokens!(T_CS!("\\hfill")),
      stored_map!("isSpace" => true, "isFill" => true),
    )
  });

  //======================================================================
  // Vertical skips
  //----------------------------------------------------------------------
  // \vfil             d  inserts first order infinitely stretchable vertical glue in a vertical
  // list. \vfill            d  inserts second order infinitely stretchable vertical glue in a
  // vertical list. \vfilneg          d  cancels the stretchability of \vfil.
  // \vss              d  insert infinitely stretchable and shrinkable vertical glue in a vertical
  // list.

  // Perl: all have leaveHorizontal => 1
  DefPrimitive!("\\vfil", None, leave_horizontal => true);
  DefPrimitive!("\\vfill", None, leave_horizontal => true);
  DefPrimitive!("\\vss", None, leave_horizontal => true);
  DefPrimitive!("\\vfilneg", None, leave_horizontal => true);

  //======================================================================
  // Lastskip
  //----------------------------------------------------------------------
  // \lastskip         iq is 0.0 pt or the last glue or muglue on the current list.

  DefRegister!("\\lastskip", Dimension::new(0), readonly => true, getter => {
    stomach::with_box_list(|stomach_box_list| {
      let box_iter = stomach_box_list.iter().rev();
      for box_in_list in box_iter {
        if !matches!(box_in_list.data(), DigestedData::Comment(_)) {
          if box_in_list.get_property_bool("isSkip") {
            let Some(width_stored) = box_in_list.get_property("width") else {
              break;
            };
            if let Stored::Dimension(ref width_d) = *width_stored {
              return *width_d;
            } else {
              log::warn!("Unexpected type of \"width\" value in State: {width_stored:?}");
              break;
            }
          } else {
            break;
          }
        }
      }
      Dimension::new(0)
    })
  });
});

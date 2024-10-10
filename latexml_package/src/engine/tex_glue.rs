//! TeX Glue
//! 
//! Core TeX Implementation for LaTeXML
use crate::prelude::*;
static UNICODE_EM_SPACES : [(f64,char); 7] = [ // Spaces to fake spacing, with width in ems
  (0.100, '\u{200A}'),    // Hair space (thinner than thin space)
  (0.167, '\u{2006}'),    // six-per-em
  (0.200, '\u{2009}'),    // five-per-em, thin space
  (0.250, '\u{2005}'),    // four-per-em, mid space
  (0.333, '\u{2004}'),    // three-per-em, thick space
  (0.500, '\u{2002}'),    // en-quad, "nut"
  (1.000, '\u{2003}'),    // em-quad, "mutton"
];
/// String of spacing chars with width roughly equivalent to $dimen
fn dimension_to_spaces(dimen: Dimension) -> String {
  let fs      = lookup_font().unwrap().get_size().unwrap_or(1.0);    // 1 em
  let mut ems     = dimen.pt_value(None) / fs;
  let mut s       = String::default();
  for (w,space_char) in UNICODE_EM_SPACES.into_iter().rev() {
    if ems <= 0.0 {
      break;
    }
    if ems + 0.01 > w {
      let n = ((ems + 0.01).floor() / w) as usize;
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
      _ => Dimension::default() // should this also be an error?
    };
    let parent = document.get_node();
    //    Debug("HSKIP ".ToString($length)." at ".$document->getNodeQName($parent));
    if document::with_node_qname(parent, |name| name == "svg:g") {
      todo!();
  //     if (my $x = $length->pxValue) {
  //       # HACK HACK HACK
  //       my $transform = $parent->getAttribute('transform');
  //       $parent->setAttribute(transform => ($transform ? $transform . ' ' : '') . "translate($x,0)");
  //   } }
  //   elsif (inSVG()) {
  //     Warn('unexpected', 'kern', $_[0], "Lost hskip in SVG " . ToString($length)); }
  //   elsif ($props{isMath}) {
  //     $document->insertElement('ltx:XMHint', undef, width => $length); }
    } else {
      document.absorb_string(&dimension_to_spaces(length), &SymHashMap::default())?; 
    } 
  },
  properties => sub[args] {
    unref!(args => length);
    Ok(stored_map!(
      "width" => length, "isSpace" => true, "isSkip" => true))
  });

 
  //======================================================================
  // If this is the right solution...
  // then we also should put the desired spacing on a style attribute?!?!?!
  DefConstructor!("\\vskip Glue", sub[document, args, _props] {
    unref!(args => length);
    let length = length.pt_value(None);

    if length > 10.0 {    // Or what!?!?!?!
      if document.is_closeable("ltx:para").is_some() {
        document.close_element("ltx:para")?;
      } else if document.is_openable("ltx:break") {
        document.insert_element("ltx:break", Vec::new(), None)?;
      }
    }},
     // TODO: "height" property
    properties => {stored_map!("isSpace" => true, "isSkip"=>true,
      "isVerticalSpace" => true, "isBreak" => true) }
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
      } else { // return a non-skip box to the list.
        push_box_list(last_box);
        break;
      }
    }
  });
  
  //======================================================================
  // Horizontal skips
  //----------------------------------------------------------------------
  // \hfil             d  inserts first order infinitely stretchable horizontal glue in a horizontal or math list.
  // \hfill            d  inserts second order infinitely stretchable horizontal glue in a horizontal or math list.
  // \hfilneg          d  cancels the stretchability of \hfil.
  // \hss              d  inserts infinitely stretchable and shrinkable horizontal glue in a horizontal or math list.
  DefPrimitive!("\\hss", None);
  DefPrimitive!("\\hfilneg", None);
  DefPrimitive!("\\hfil", {
    Tbox::new(arena::pin_static(" "), None, None, Tokens!(T_CS!("\\hfil")),
    stored_map!("isSpace" => true, "isFill" => true))});
  DefPrimitive!("\\hfill", {
    Tbox::new(arena::pin_static(" "), None, None, Tokens!(T_CS!("\\hfill")),
    stored_map!("isSpace" => true, "isFill" => true)) });

  //======================================================================
  // Vertical skips
  //----------------------------------------------------------------------
  // \vfil             d  inserts first order infinitely stretchable vertical glue in a vertical list.
  // \vfill            d  inserts second order infinitely stretchable vertical glue in a vertical list.
  // \vfilneg          d  cancels the stretchability of \vfil.
  // \vss              d  insert infinitely stretchable and shrinkable vertical glue in a vertical list.
  
  // Stuff to ignore for now...
  DefPrimitive!("\\vfil", None);
  DefPrimitive!("\\vfill", None);
  DefPrimitive!("\\vss", None);
  DefPrimitive!("\\vfilneg", None);

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
            let width_stored = box_in_list.get_property("width").unwrap(); 
            if let Stored::Dimension(ref width_d) = *width_stored {
              return *width_d;
            } else {
              panic!("Unexpected type of \"width\" value in State: {width_stored:?}");
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
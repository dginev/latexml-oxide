//! TeX Kern
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Kern Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // Basic kerning
  //----------------------------------------------------------------------
  // \kern             c  adds a kern item to the current list.
  // \unkern           c  removes a kern from the current list.
  // \lastkern         iq is 0.0 pt or the last kern on the current list.

  // \kern is heavily used by xy.
  // Note that \kern should add vertical spacing in vertical modes!
  // Perl: TeX_Kern.pool.ltxml L31-52
  DefConstructor!("\\kern Dimension", sub[document,args, props] {
    let length : Dimension = if let DigestedData::RegisterValue(RegisterValue::Dimension(d)) =
      args[0].as_ref().unwrap().data() {
        *d
      } else { Dimension::default() };
    let is_svg_g = document::with_node_qname(document.get_node(),
      |qname| qname == "svg:g");
    if is_svg_g {
      let x = length.px_value(None);
      if x != 0.0 {
        let shift = s!("translate({x},0)");
        let parent = document.get_node_mut();
        let has_children = !parent.get_child_nodes().is_empty();
        if has_children {
          // Perl L37-38: If already have positioned children, open new svg:g
          let attrs = string_map!("_autoclose" => "1", "transform" => shift);
          document.open_element("svg:g", Some(attrs), None)?;
        } else {
          // Perl L40-41: No children yet — append to parent's transform attribute
          let prev = parent.get_attribute("transform").unwrap_or_default();
          let new_transform = if prev.is_empty() { shift } else { s!("{prev} {shift}") };
          parent.set_attribute("transform", &new_transform)?;
        }
      }
    } else if in_svg(document) {
      Warn!("unexpected", "kern", s!("Lost kern in SVG {length}"));
    } else if props.get("isMath") == Some(&Stored::Bool(true)) {
      document.insert_element("ltx:XMHint", Vec::new(), Some(map!("width" => length.to_attribute())))?;
    } else {
      // Add space to document
      // Use the precise Unicode space mapping from tex_glue (matching Perl's TeX_Glue algorithm),
      // not the simple threshold version from base_functions.
      let spaces = super::tex_glue::dimension_to_spaces(length);
      document.absorb_string(&spaces, &SymHashMap::default())?;
    }
  },
  enter_horizontal => true,
  properties => sub[args] {
    unref!(args => length);
    Ok(stored_map!("width" => length, "isSpace" => true, "isKern" => true))
  });

  // Remove kern, if last on LIST
  DefPrimitive!("\\unkern", {
    let mut comments = Vec::new();
    // Scan past any Comment boxes
    while let Some(last_box) = pop_box_list() {
      if matches!(last_box.data(), DigestedData::Comment(_)) {
        comments.push(last_box);
      } else {
        if !last_box.get_property_bool("isKern") {
          push_box_list(last_box);
        }
        break;
      }
    }
    let comments_rev_iter = comments.into_iter().rev();
    for comment in comments_rev_iter {
      push_box_list(comment);
    }
  });
  // Get kern, if last on LIST
  DefRegister!("\\lastkern" => Dimension::new(0), readonly => true,
  getter => {
    stomach::with_box_list(|stomach_box_list| {
      let box_iter = stomach_box_list.iter().rev();
      for box_in_list in box_iter {
        if !matches!(box_in_list.data(), DigestedData::Comment(_)) {
          if box_in_list.get_property_bool("isKern") {
            let width_stored = box_in_list.get_property("width").unwrap();
            match &*width_stored {
              Stored::Dimension(width_d) => return *width_d,
              Stored::Digested(d) => {
                if let DigestedData::RegisterValue(RegisterValue::Dimension(dim)) = d.data() {
                  return *dim;
                }
                return Dimension::new(0);
              }
              _ => return Dimension::new(0),
            }
          } else {
            break;
          }
        }
      }
      Dimension::new(0)
    })
  });

  //======================================================================
  // Moving Vertically
  //----------------------------------------------------------------------
  // \raise            c  shifts a box up and appends it to the current horizontal or math list.
  // \lower            c  shifts a box down and appends it to the current horizontal or math list.
  // \lower <dimen> <box>
  // \raise <dimen> <box>
  // But <box> apparently must really explicitly be an \hbox, \vbox or \vtop (?)
  // OR something that expands into one!!
  // Perl: TeX_Kern.pool.ltxml L94-103
  // Template: ?&inSVG()(<svg:g transform='#transform' _noautoclose='1'>#2</svg:g>)
  //                    (<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>)
  DefConstructor!("\\lower Dimension MoveableBox",
    sub[document, args, props] {
      let svg = in_svg(document);
      // Move the initial attrs map unconditionally into whichever branch
      // runs — only one executes per call, so no clone is needed.
      let mut attrs = string_map!("_noautoclose" => "1");
      let node = if svg {
        let transform = match props.get("transform") {
          Some(Stored::String(s)) => arena::to_string(*s), _ => String::new()
        };
        if !transform.is_empty() { attrs.insert(String::from("transform"), transform); }
        document.open_element("svg:g", Some(attrs), None)?
      } else {
        let y_attr = match props.get("y") {
          Some(Stored::Dimension(d)) => d.to_attribute(), _ => String::new()
        };
        if !y_attr.is_empty() { attrs.insert(String::from("yoffset"), y_attr); }
        document.open_element("ltx:text", Some(attrs), None)?
      };
      if let Some(Some(content)) = args.get(1) {
        document.absorb(content, None)?;
      }
      document.maybe_close_node(&node)?;
    },
    // Perl: sizer => sub { raisedSizer($_[0]->getArg(2), $_[0]->getArg(1)->negate); }
    sizer => sub[whatsit] {
      let y_val = -(whatsit.get_arg(1).map(|a| a.value_of()).unwrap_or(0));
      if let Some(content) = whatsit.get_arg(2) {
        let (w, h, d) = content.compute_size(Default::default())?;
        let new_h = Dimension::new((h.value_of() + y_val).max(0));
        let new_d = Dimension::new((d.value_of() - y_val).max(0));
        Ok((w, new_h, new_d))
      } else {
        Ok((Dimension::new(0), Dimension::new(0), Dimension::new(0)))
      }
    },
    enter_horizontal => true,
    after_digest => sub[whatsit] {
      let y         = Dimension(-whatsit.get_arg(1).unwrap().value_of());
      let ypx       = y.px_value(None);
      let transform = if ypx != 0.0 { s!("translate(0,{ypx})") } else { String::new() };
      whatsit.set_property("y", y);
      whatsit.set_property("transform", transform);
    }
  );

  // Perl: TeX_Kern.pool.ltxml L105-114
  // Template: ?&inSVG()(<svg:g transform='#transform' _noautoclose='1'>#2</svg:g>)
  //                    (<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>)
  DefConstructor!("\\raise Dimension MoveableBox",
  sub[document, args, props] {
    let svg = in_svg(document);
    let mut attrs = string_map!("_noautoclose" => "1");
    let node = if svg {
      let transform = match props.get("transform") {
        Some(Stored::String(s)) => arena::to_string(*s), _ => String::new()
      };
      if !transform.is_empty() { attrs.insert(String::from("transform"), transform); }
      document.open_element("svg:g", Some(attrs), None)?
    } else {
      let y_attr = match props.get("y") {
        Some(Stored::Dimension(d)) => d.to_attribute(), _ => String::new()
      };
      if !y_attr.is_empty() { attrs.insert(String::from("yoffset"), y_attr); }
      document.open_element("ltx:text", Some(attrs), None)?
    };
    if let Some(Some(content)) = args.get(1) {
      document.absorb(content, None)?;
    }
    document.maybe_close_node(&node)?;
  },
  // Perl: sizer => sub { raisedSizer($_[0]->getArg(2), $_[0]->getArg(1)); }
  // Adjusts reported height/depth by the raise amount so \ht/\dp reflect the shift.
  sizer => sub[whatsit] {
    let y_val = whatsit.get_arg(1).map(|a| a.value_of()).unwrap_or(0);
    if let Some(content) = whatsit.get_arg(2) {
      let (w, h, d) = content.compute_size(Default::default())?;
      let new_h = Dimension::new((h.value_of() + y_val).max(0));
      let new_d = Dimension::new((d.value_of() - y_val).max(0));
      Ok((w, new_h, new_d))
    } else {
      Ok((Dimension::new(0), Dimension::new(0), Dimension::new(0)))
    }
  },
  enter_horizontal => true,
  after_digest => sub[whatsit] {
    let y         = Dimension(whatsit.get_arg(1).unwrap().value_of());
    let ypx       = y.px_value(None);
    let transform = if ypx != 0.0 { s!("translate(0,{ypx})") } else { String::new() };
    whatsit.set_property("y", y);
    whatsit.set_property("transform", transform);
  });

  //======================================================================
  // Moving Horizontally
  //----------------------------------------------------------------------
  // \moveleft         c  shifts a box left and appends it to the current vertical list.
  // \moveright        c  shifts a box right and appends it to the current vertical list.
  // \moveleft<dimen><box>, \moveright<dimen><box>
  // \moveleft<dimen><box>, \moveright<dimen><box>
  // Perl: enterHorizontal => 1
  DefConstructor!("\\moveleft Dimension MoveableBox",
  "<ltx:text xoffset='#x' _noautoclose='true'>#2</ltx:text>",
  enter_horizontal => true,
  after_digest => sub[whatsit] {
    if let DigestedData::RegisterValue(d) = whatsit.get_arg(1).unwrap().data() {
      whatsit.set_property("x", d.clone().multiply(Number::new(-1)));
    }
  });
  // Perl: enterHorizontal => 1
  DefConstructor!("\\moveright Dimension MoveableBox",
  "<ltx:text xoffset='#x' _noautoclose='true'>#2</ltx:text>",
  enter_horizontal => true,
  after_digest => sub[whatsit] {
    if let Some(dimension) = whatsit.get_arg(1) {
      whatsit.set_property("x", dimension.clone());
    }
  });
});

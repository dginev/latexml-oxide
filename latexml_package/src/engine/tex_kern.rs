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
  // Completely HACK version for the moment
  // Note that \kern should add vertical spacing in vertical modes!
  // Perl: enterHorizontal => 1
  DefConstructor!("\\kern Dimension", sub[document,args, props] {
    // TODO: We definitely need a cleaner Dimension cast here.
    let length : Dimension = if let DigestedData::RegisterValue(RegisterValue::Dimension(d)) =
      args[0].as_ref().unwrap().data() {
        *d
      } else { Dimension::default() };
    let is_svg_g = document::with_node_qname(document.get_node(),
      |qname| qname == "svg:g");
    let parent = document.get_node_mut();
    if is_svg_g {
      let x = length.px_value(None);
      if x > 0.0 {
        // HACK HACK HACK
        let mut transform = parent.get_attribute("transform").unwrap_or_default();
        if !transform.is_empty() {
          transform.push(' ');
        }
        transform.push_str(&s!("translate({x},0)"));
        parent.set_attribute("transform", &transform)?;
      }
    } else if in_svg(document) {
      Warn!("unexpected", "kern", s!("Lost kern in SVG {length}"));
    } else if props.get("isMath") == Some(&Stored::Bool(true)) {
      // TODO: Reconsider if the insert_element API needs to be based around
      // Stored map values, rather than String map values.
      document.insert_element("ltx:XMHint", Vec::new(), Some(map!("width" => length.to_string())))?;
    } else {
      // Add space to document?
      document.absorb_string(&dimension_to_spaces(length), &SymHashMap::default())?;
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
              Stored::Dimension(ref width_d) => return *width_d,
              Stored::Digested(ref d) => {
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
  // Perl: enterHorizontal => 1
  DefConstructor!("\\lower Dimension MoveableBox",
  "<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>",
    // sizer => sub { raisedSizer($_[0]->getArg(2), $_[0]->getArg(1)->negate); },
    enter_horizontal => true,
    after_digest => sub[whatsit] {
      let y         = Dimension(-whatsit.get_arg(1).unwrap().value_of());
      let ypx       = y.px_value(None);
      let transform = if ypx != 0.0 { s!("translate(0,{ypx})") } else { String::new() };
      whatsit.set_property("y", y);
      whatsit.set_property("transform", transform);
    }
  );

  // Perl: enterHorizontal => 1
  DefConstructor!("\\raise Dimension MoveableBox",
  "<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>",
  //sizer       => sub { raisedSizer($_[0]->getArg(2), $_[0]->getArg(1)); },
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

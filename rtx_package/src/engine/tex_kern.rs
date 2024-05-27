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
  DefConstructor!("\\kern Dimension", sub[document,args] {
    let length = if let DigestedData::RegisterValue(RegisterValue::Dimension(d)) =
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
    }
  });
  DefPrimitive!("\\unkern", None);
  DefRegister!("\\lastkern" => Dimension::new(0), readonly => true);

  //======================================================================
  // Moving Vertically
  //----------------------------------------------------------------------
  // \raise            c  shifts a box up and appends it to the current horizontal or math list.
  // \lower            c  shifts a box down and appends it to the current horizontal or math list.
  // \lower <dimen> <box>
  // \raise <dimen> <box>
  // But <box> apparently must really explicitly be an \hbox, \vbox or \vtop (?)
  // OR something that expands into one!!
  DefConstructor!("\\lower Dimension MoveableBox",
  // TODO: SVG
  // "?&inSVG()(<svg:g transform='#transform' _noautoclose='1'>#2</svg:g>)\
  // (<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>)",
  "<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>",
    // sizer => sub { raisedSizer($_[0]->getArg(2), $_[0]->getArg(1)->negate); },
    after_digest => sub[whatsit] {
      let y         = Dimension(-whatsit.get_arg(1).unwrap().value_of());
      let ypx       = y.px_value(None);
      let transform = if ypx != 0.0 { s!("translate(0,{ypx})") } else { String::new() };
      whatsit.set_property("y", y);
      whatsit.set_property("transform", transform);
    }
  );

  DefConstructor!("\\raise Dimension MoveableBox",
  // TODO: SVG
  // "?&inSVG()(<svg:g transform='#transform' _noautoclose='1'>#2</svg:g>)"
  //   . "(<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>)",
  "<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>",
  //sizer       => sub { raisedSizer($_[0]->getArg(2), $_[0]->getArg(1)); },
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
  DefConstructor!("\\moveleft Dimension MoveableBox",
    "<ltx:text xoffset='#x' _noautoclose='true'>#2</ltx:text>",
    after_digest => sub[whatsit] {
      if let DigestedData::RegisterValue(d) = whatsit.get_arg(1).unwrap().data() {
        whatsit.set_property("x", d.clone().multiply(Number::new(-1)));
      }
    });
  DefConstructor!("\\moveright Dimension MoveableBox",
    "<ltx:text xoffset='#x' _noautoclose='true'>#2</ltx:text>",
    after_digest => sub[whatsit] {
      if let Some(dimension) = whatsit.get_arg(1) {
        whatsit.set_property("x", dimension.clone());
      }
    });
  
});
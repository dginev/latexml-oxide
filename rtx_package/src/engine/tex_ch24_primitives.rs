use crate::prelude::*;
// use super::tex_boxes::adjust_box_color;

//**********************************************************************
// Primitives
// See The TeXBook, Chapter 24, Summary of Vertical Mode
//  and Chapter 25, Summary of Horizontal Mode.
// Parsing of basic types (pp.268--271) is (mostly) handled in Gullet.pm
//**********************************************************************

LoadDefinitions!({
  //======================================================================
  // Remaining Mode independent primitives in Ch.24, pp.279-280
  // \relax was done as expandable (isn't that right?)
  // }
  // Note, we don't bother making sure begingroup is ended by endgroup.

  // These define the handler for { } (or anything of catcode BEGIN, END)

  DefPrimitive!(
  "\\begingroup", {
    begingroup();
  });
  DefPrimitive!(
  "\\endgroup", {
    endgroup()?;
  });

  // DefPrimitive('\shipout ??
  DefPrimitive!("\\ignorespaces SkipSpaces", None);

  // \afterassignment saves ONE token (globally!) to execute after the next assignment
  DefPrimitive!("\\afterassignment Token", sub[(t)] {
    assign_value("afterAssignment", t, Some(Scope::Global));
  });
  // \aftergroup saves ALL tokens (from repeated calls) to be executed IN ORDER after the next
  // egroup or }
  DefPrimitive!("\\aftergroup Token", sub[(t)] {
    push_value("afterGroup", t)
  });


  DefConditional!("\\ifeof Number", sub[(port)] {
    with_value(&s!("input_file:{}", port), |val_opt|
      if let Some(Stored::Mouth(mouth)) = val_opt {
        mouth.borrow().at_eof()
      } else {
        true
      })
  });

  //======================================================================
  // Remaining semi- Vertical Mode primitives in Ch.24, pp.280--281


  DefPrimitive!("\\penalty Number", None);
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
  DefMacro!(
    "\\mkern MuGlue",
    "\\ifmmode\\@math@mskip #1\\relax\\else\\@text@mskip #1\\relax\\fi"
  );
  DefPrimitive!("\\unpenalty", None);
  DefPrimitive!("\\unkern", None);
  // Worrisome, but...
  DefPrimitive!("\\unskip", {
    // pop until a non-empty box is found
    while let Some(last_box) = pop_box_list() {
      if !last_box.is_empty()? {
        push_box_list(last_box);
        break;
      }
    }
  });

  DefPrimitive!("\\mark{}", None);
  // \insert<8bit><filler>{<vertical mode material>}
  DefPrimitive!("\\insert Number", None);
  // \vadjust<filler>{<vertical mode material>}
  // Note: \vadjust ignores in vertical mode...
  DefPrimitive!("\\vadjust {}", sub[(arg)] { push_tokens("vAdjust", arg); });

  //======================================================================
  // Remaining Vertical Mode primitives in Ch.24, pp.281--283
  // \vskip<glue>, \vfil, \vfill, \vss, \vfilneg
  // <leaders> = \leaders | \cleaders | \xleaders
  // <box or rule> = <box> | <vertical rule> | <horizontal rule>
  // <vertical rule> = \vrule<rule specification>
  // <horizontal rule> = \hrule<rule specification>
  // <rule specification> = <optional spaces> | <rule dimension><rule specification>
  // <rule dimension> = width <dimen> | height <dimen> | depth <dimen>

  // Stuff to ignore for now...
  DefPrimitive!("\\vfil", None);
  DefPrimitive!("\\vfill", None);
  DefPrimitive!("\\vss", None);
  DefPrimitive!("\\vfilneg", None);

  // \moveleft<dimen><box>, \moveright<dimen><box>
  DefConstructor!("\\moveleft Dimension MoveableBox",
    "<ltx:text xoffset='#x' _noautoclose='true'>#2</ltx:text>",
    after_digest => sub[whatsit] {
      if let DigestedData::RegisterValue(d) = whatsit.get_arg(1).unwrap().data() {
        whatsit.set_property("x", d.clone().multiply(Number::new(-1)));
      }});
  DefConstructor!("\\moveright Dimension MoveableBox",
    "<ltx:text xoffset='#x' _noautoclose='true'>#2</ltx:text>",
    after_digest => sub[whatsit] {
      if let Some(dimension) = whatsit.get_arg(1) {
        whatsit.set_property("x", dimension.clone());
      }});

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
    properties => {stored_map!("isSpace" => true, "isVerticalSpace" => true, "isBreak" => true)}
  );
});

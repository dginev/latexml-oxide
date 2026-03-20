use crate::prelude::*;
static SCRIPT_NAME_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\\lx@(floating|post)@(subscript|superscript)$").unwrap());

//======================================================================
// Scripts are a bit of a strange beast, with respect to when the arguments
// are processed, and what kind of object should be created.
//
// While scripts look like they take a normal TeX argument, they really
// take the next BOX (AFTER expansion & digestion)!  Thus, while
//   a^\frac{b}{c} and a^\mathcal{B}
// DO work in TeX, other things like
//   a^\sqrt{3} or a^\acute{b}
// DO NOT! (Hint: consider the expansions)
// Note that with
//  \def\xyz{xyz}
//   a^\xyz   =>  a^{x}yz
// So, we try to mimic, but note that our boxes don't correspond 100% to TeX's
//
// Normally, sub/super scripts should be turned into a sort of postfix operator:
// The parser will attach the script to the appropriate preceding object.
// However, there are a few special cases involving empty boxes {}.
// If the argument is an empty box $x^{}$, the whole script should just disappear.
// If the PRECEDING box is {} (in ${}^{p}$, a sort of `floating' script should be created.
// This may combine, in the parser, with the following object to generate
// a prescript.

// DG: Note: TeX.pool's isEmpty seems best reorganized as Digested::is_empty
// implemented by each concrete data structure. That should now be the case.

// Remember a "safe" way to test a script Whatsit.
// Returns [ (FLOATING|POST) , (SUBSCRIPT|SUPERSCRIPT) ] or nothing
pub fn is_script(object: &Digested) -> Option<(String, Catcode)> {
  let box_opt = match object.data() {
    DigestedData::List(obj) => obj.borrow().boxes.last().map(|v| Cow::Owned(v.clone())),
    _ => Some(Cow::Borrowed(object)),
  };
  if let Some(boxobj) = box_opt {
    if let DigestedData::Whatsit(ref obj) = boxobj.data() {
      // careful w/alias in getCSName!
      obj.borrow().get_definition().get_cs().with_cs_name(|name| {
        SCRIPT_NAME_RE.captures(name).map(|cap| {
          (
            cap.get(1).map_or("", |m| m.as_str()).to_uppercase(),
            if cap.get(2).map_or("", |m| m.as_str()) == "subscript" {
              Catcode::SUB
            } else {
              Catcode::SUPER
            },
          )
        })
      })
    } else {
      None
    }
  } else {
    None
  }
}

// TODO: Something is really off with the Rust version of the data model for
// Digested here, we keep having to *clone* incorrectly. The Perl expectation
// was for something Rust deems highly illegal/unsafe: multiple owners of a
// mutable reference to a Digested object. This needs to be disentangled
// in the new codebase, so as to avoid both 1) cloning and
// 2) mutably referencing the same Digested object from multiple unrelated pieces of code.
//
fn script_handler(cc: Catcode) -> Result<Vec<Digested>> {
  //
  //   gullet::skip_spaces();
  let font = lookup_font().unwrap();
  if font.get_mathstyle().is_some() {
    let mut putback = VecDeque::new();
    let mut nscripts = 0;

    let mut cs = if cc == Catcode::SUPER {
      "\\lx@floating@superscript"
    } else {
      "\\lx@floating@subscript"
    };
    let mut prevscript = None;
    let mut prevspace = false;
    let mut base = None;
    // Check preceding boxes to determine possible attachment (floating vs post),
    // Note that this analysis has to be done now (or sometime like it) before grouping lists go
    // away; and whether there are conflicting preceding scripts, which is an error
    // Parsing is too late!
    while let Some(prev) = { pop_box_list() } {
      if prev.get_property_bool("isSpace") || prev.get_property_bool("isEmpty") {
        // Explicitly empty (isSpace) or explicitly marked isEmpty (e.g. \limits):
        // put back and keep looking for the base. A space also avoids double-scripts.
        // Mirrors Perl: getProperty('isSpace') || getProperty('isEmpty') => put back, next
        prevspace = true;
        putback.push_front(prev);
        continue;
      } else if prev.is_empty()? {
        // Structurally empty (e.g. bare `{}`): script floats, don't put back
        // Mirrors Perl: IsEmpty($prev) => last (no unshift)
        break;
      } else if let Some(prevop) = is_script(&prev) {
        if prevop.1 == cc {
          // Whoops, duplicated; better use FLOATING
          putback.push_front(prev);
          let lcode = if prevop.1 == Catcode::SUPER {
            "superscript"
          } else {
            "subscript"
          };
          if !prevspace {
            Error!("unexpected", s!("double-{lcode}"), s!("Double {lcode}"));
          }
          cs = if cc == Catcode::SUPER {
            "\\lx@floating@superscript"
          } else {
            "\\lx@floating@subscript"
          };
          break;
        } else {
          // Else, is OK (so far) assume POST (it will stack previous script)
          prevscript = Some(prev.clone()); // we'll overlap the width of the previous.
          putback.push_front(prev);
          cs = if cc == Catcode::SUPER {
            "\\lx@post@superscript"
          } else {
            "\\lx@post@subscript"
          };
        }
        // if we hit a FLOATING script, terminate, as the floating empty group avoids double scripts
        if prevop.0 == "FLOATING" {
          break;
        }
        nscripts += 1;
        if nscripts > 1 {
          break;
        }
      } else {
        //  We found something "normal", so assume we'll attach to it, and we're done.
        base = Some(prev.clone());
        putback.push_front(prev);
        cs = if cc == Catcode::SUPER {
          "\\lx@post@superscript"
        } else {
          "\\lx@post@subscript"
        };
        break;
      }
    }
    extend_box_list(putback);
    MergeFont!(scripted => true);
    // Now, get following boxes (may have to process several tokens!)
    let mut stuff = Vec::new();
    while let Some(tok) = gullet::read_x_token(Some(false), false, None)? {
      stuff = stomach::invoke_token(&tok)?;
      if !stuff.is_empty() {
        break;
      }
    }
    if stuff.is_empty() {
      Error!("expected", "{", "Missing sub/superscript argument"); //$gullet->showUnexpected);
      stuff.push(Digested::default());
    }
    let script = stuff.remove(0); // ONLY the first box is the script!

    if !script.is_empty()? {
      let mut properties = {
        stored_map!(
          "isMath" => true,
          "base"        => if let Some(b) = base { Stored::Digested(b) }
            else { Stored::None },                      // for sizing/positioning
          "scriptlevel" => get_script_level(),
          "level"       => get_boxing_level()
        )
      };
      if let Some(pvs) = prevscript {
        properties.insert("prevscript", pvs.into());
      }
      if let Some(font) = script.get_font()? {
        properties.insert("font", font.into());
      }
      let mut with_script = vec![Digested::from(Whatsit {
        definition: lookup_definition(&T_CS!(cs))?.unwrap(),
        args: vec![Some(script)],
        properties,
        // TODO:
        // locator: stomach.get_gullet().get_locator(),
        ..Whatsit::default()
      })];
      with_script.extend(stuff);
      stuff = with_script;
    }
    assign_font(font, Some(Scope::Local)); // revert
    Ok(stuff)
  } else {
    let c = if cc == Catcode::SUPER { '^' } else { '_' };
    Error!(
      "Unexpected",
      c,
      format!("Script {c} can only appear in math mode")
    );
    let placeholder = if cc == Catcode::SUPER {
      T_SUPER!()
    } else {
      T_SUB!()
    };
    Ok(vec![Digested::from(Tbox::new(
      arena::pin_char(c),
      None,
      None,
      Tokens!(placeholder),
      SymHashMap::default(),
    ))])
  }
}

// The `argument' to a sub or superscript will typically be processed as a box,
// and either has braces, or is something that results in a single box.
// When we revert these, we DON'T want to wrap extra braces around, because they'll accumulate;
// at the least they're ugly; in some applications they affect "round trip" processing.
// OTOH, direct use of \lx@post@superscript, etal, MAY need to have extra braces around them.
// So, when reverting, we're going to a bit of extra trouble to make sure we have ONE set
// of braces, but no extras!!  [Worry about lists of lists...]
pub fn revert_script(script: &Digested) -> Result<Vec<Token>> {
  let tokens = script.revert()?;
  let mut ts = tokens.unlist();
  // let mut level = 0;
  if ts.len() > 1
    && ts.first().unwrap().code == Catcode::BEGIN
    && ts.last().unwrap().code == Catcode::END
  {
    Ok(ts)
  } else {
    let mut wrapped = vec![T_BEGIN!()];
    wrapped.append(&mut ts);
    wrapped.push(T_END!());
    Ok(wrapped)
  }
}

// Compute the 'advance' of this script.
// can we do this before parsing? we can do the advance or something.... Hmmmm.
// * Need to know scriptpos (mid or post) to determine position.
// * need to know sub/super
fn script_sizer(
  script: &Digested,
  base_opt: Option<&Stored>,
  prev_opt: Option<&Stored>,
  op: &str,
  pos: &str,
) -> Result<(Dimension, Dimension, Dimension)> {
  // Perl: scriptSizer in TeX_Math.pool.ltxml
  // Uses font metrics for proper positioning of super/subscripts.
  let script_size = script.clone().get_size(None)?;
  let (mut ws, hs, ds) = (
    script_size.0.value_of() as f64,
    script_size.1.value_of() as f64,
    script_size.2.value_of() as f64,
  );
  // Get base font info for mathstyle and font size
  let (base_font_size, mathstyle) = if let Some(Stored::Digested(ref base)) = base_opt {
    let bfont = base.get_font()?.map(|f| f.into_owned());
    let fs = bfont.as_ref().and_then(|f| f.get_size()).unwrap_or(10.0);
    let ms = bfont
      .as_ref()
      .and_then(|f| f.mathstyle.as_deref().map(|s| s.to_string()))
      .unwrap_or_else(|| "text".to_string());
    (fs, ms)
  } else {
    let f = lookup_font().unwrap();
    let fs = f.get_size().unwrap_or(10.0);
    let ms = f
      .mathstyle
      .as_deref()
      .map(|s| s.to_string())
      .unwrap_or_else(|| "text".to_string());
    (fs, ms)
  };
  let (_wb, hb, db) = if let Some(Stored::Digested(ref base)) = base_opt {
    let base_size = base.clone().get_size(None)?;
    (
      base_size.0.value_of() as f64,
      base_size.1.value_of() as f64,
      base_size.2.value_of() as f64,
    )
  } else {
    let nominal_size = lookup_font().unwrap().get_nominal_size();
    (
      nominal_size.0.value_of() as f64,
      nominal_size.1.value_of() as f64,
      nominal_size.2.value_of() as f64,
    )
  };
  let w;
  let (mut h, mut d) = (0.0, 0.0);
  // Nominal font info ratios (from Perl's TeX_Fonts.pool.ltxml $nominal_fontinfo)
  // These are ratios of font design size, used for cmsy font dimens.
  const XHEIGHT_RATIO: f64 = 0.430555; // param #5
  const SUPERSCRIPT1_RATIO: f64 = 0.412892; // param #13 (displaystyle)
  const SUPERSCRIPT2_RATIO: f64 = 0.362892; // param #14 (text/scriptstyle)
  const SUPERSCRIPT3_RATIO: f64 = 0.288889; // param #15 (scriptscriptstyle)
  const SUBSCRIPT1_RATIO: f64 = 0.15; // param #16
  // Font dimen values: font_size_sp * ratio
  let font_scale = base_font_size * 65536.0; // font size in scaled points
  let xheight = font_scale * XHEIGHT_RATIO;
  // Fishing for the scriptpos on the base (if any)
  let inferred_pos = if let Some(Stored::Digested(ref base)) = base_opt {
    let base_pos = base
      .get_property("scriptpos")
      .map(|s| s.to_string())
      .unwrap_or_default();
    if base_pos.is_empty() {
      Cow::Borrowed("post")
    } else {
      // Strip any existing level number to get just "mid" or "post"
      let stripped: String = base_pos.chars().take_while(|c| !c.is_ascii_digit()).collect();
      Cow::Owned(if stripped.is_empty() { base_pos } else { stripped })
    }
  } else {
    Cow::Borrowed("post")
  };
  if inferred_pos == "mid" {
    w = (ws - _wb).max(0.0); // as if max width of base & script
    if op == "SUPERSCRIPT" {
      h = hb + ds + hs;
    } else {
      d = db + hs + ds;
    }
  } else {
    // as if max of width & prev script's width
    let wp = if let Some(Stored::Digested(ref prev)) = prev_opt {
      prev.get_width(None)?.unwrap_or_default().value_of() as f64
    } else {
      0.0
    };
    // Perl: $ws += $space (scriptspace register, default 0.5pt)
    let scriptspace = state::lookup_register("\\scriptspace", Vec::new())
      .ok()
      .flatten()
      .map(|rv| match rv {
        RegisterValue::Dimension(d) => d.value_of() as f64,
        _ => 32768.0, // 0.5pt fallback
      })
      .unwrap_or(32768.0);
    ws += scriptspace;
    w = (ws - wp).max(0.0);
    if op == "SUPERSCRIPT" {
      // Perl: $supshift = getFontDimen($syfont, display:13, scriptscript:15, else:14)
      //       $h = max($hb, $hs + max($ds + $xheight / 4, $supshift))
      let supshift = font_scale
        * match mathstyle.as_str() {
          "display" => SUPERSCRIPT1_RATIO,
          "scriptscript" => SUPERSCRIPT3_RATIO,
          _ => SUPERSCRIPT2_RATIO,
        };
      h = hb.max(hs + (ds + xheight / 4.0).max(supshift));
    } else {
      // Perl: $subshift = getFontDimen($syfont, 16)
      //       $d = max($db, $ds + max($hs - $xheight * 0.8, $subshift))
      let subshift = font_scale * SUBSCRIPT1_RATIO;
      d = db.max(ds + (hs - xheight * 0.8).max(subshift));
    }
  }
  Ok((
    Dimension::new_f64(w),
    Dimension::new_f64(h),
    Dimension::new_f64(d),
  ))
}

LoadDefinitions!({
  // TODO: Should I add a special macro case that takes an arbitrary token as argument?
  // DefPrimitiveT ?
  def_primitive(
    T_SUPER!(),
    None,
    Some(PrimitiveBody::Closure(Rc::new(|_args: Vec<ArgWrap>| {
      script_handler(Catcode::SUPER)
    }))),
    PrimitiveOptions::default(),
  )?;
  def_primitive(
    T_SUB!(),
    None,
    Some(PrimitiveBody::Closure(Rc::new(|_args: Vec<ArgWrap>| {
      script_handler(Catcode::SUB)
    }))),
    PrimitiveOptions::default(),
  )?;

  // NOTE: The When reverting these, the
  DefConstructor!("\\lx@post@superscript InScriptStyle",r###"
  <ltx:XMApp role="POSTSUPERSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Superscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###,
    reversion => sub[_whatsit,args] {
      unref!(args=>arg);
      Ok(Tokens!(T_SUPER!(), revert_script(arg)?)) },
    sizer => sub[w] {
      script_sizer(w.get_arg(1).unwrap(), w.get_property("base").as_deref(),
        w.get_property("prevscript").as_deref(), "SUPERSCRIPT", "") }
  );

  DefConstructor!("\\lx@post@subscript InScriptStyle",r###"
  <ltx:XMApp role="POSTSUBSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Subscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###
    ,
    reversion => sub[_whatsit,args] {
      unref!(args=>arg);
      Ok(Tokens!(T_SUB!(), revert_script(arg)?)) },
    sizer => sub[w] {
      script_sizer(w.get_arg(1).unwrap(), w.get_property("base").as_deref(),
        w.get_property("prevscript").as_deref(), "SUBSCRIPT", "") }
  );

  DefConstructor!("\\lx@floating@superscript InScriptStyle",r###"
  <ltx:XMApp role="FLOATSUPERSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Superscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###,
    reversion => sub[_whatsit,args] {
      unref!(args=>arg);
      Ok(Tokens!(T_BEGIN!(), T_END!(), T_SUPER!(), revert_script(arg)?)) }
    sizer => sub[w] {
      script_sizer(w.get_arg(1).unwrap(), None, None, "SUPERSCRIPT", "post") }
  );
  DefConstructor!("\\lx@floating@subscript InScriptStyle",r###"
  <ltx:XMApp role="FLOATSUBSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Subscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###,
    reversion => sub[_whatsit,args] {
      unref!(args=>arg);
      Ok(Tokens!(T_BEGIN!(), T_END!(), T_SUB!(), revert_script(arg)?)) }
      sizer => sub[w] {
        script_sizer(w.get_arg(1).unwrap(), None, None, "SUBSCRIPT", "post") }
  );

  // Experiment: When we detect a math element containing solely a floating superscript in the
  //             *Frontmatter* of a document, assume it is a note mark, and normalize it down to
  //             plain text.
  DefRewrite!(xpath =>
    concat!(
      "descendant::ltx:Math[child::ltx:XMath[child::ltx:XMApp[",
      "(@role='FLOATSUPERSCRIPT' or @role='FLOATSUBSCRIPT') and ",
      "not(preceding-sibling::*) and not(following-sibling::*) ",
      "and not(./*/*[not(self::ltx:XMTok)]) ]]]"
    ),
    replace => sub[document, nodes] {
      let math = nodes.pop().unwrap();
      // Navigate: Math -> XMath -> XMApp -> XMArg -> text
      let mut replaced = false;
      let xmath_children: Vec<Node> = math.get_child_nodes().into_iter()
        .filter(|n| n.get_type() == Some(NodeType::ElementNode)).collect();
      if let Some(xmath) = xmath_children.first() {
        let xmapp_children: Vec<Node> = xmath.get_child_nodes().into_iter()
          .filter(|n| n.get_type() == Some(NodeType::ElementNode)).collect();
        if let Some(xmapp) = xmapp_children.first() {
          let role = xmapp.get_attribute("role").unwrap_or_default();
          let xmarg_children: Vec<Node> = xmapp.get_child_nodes().into_iter()
            .filter(|n| n.get_type() == Some(NodeType::ElementNode)).collect();
          if let Some(xmarg) = xmarg_children.first() {
            let text = xmarg.get_content();
            let qname = if role == "FLOATSUPERSCRIPT" { "ltx:sup" } else { "ltx:sub" };
            // Perl: local $LaTeXML::BOX = $document->getNodeBox($xmarg[0]);
            // Check if child XMTok has a font attribute to preserve
            // Perl: local $LaTeXML::BOX = $document->getNodeBox($xmarg[0]);
            // Check child XMTok font to preserve italic/bold styling (from math mode)
            // Perl: uses $document->getNodeBox($args[0])->getFont with openText,
            // where $args[0] is the XMArg. The BOX font has family="math" for math
            // content, which relativeTo maps to font="italic".
            let font_attr = {
              // First check XML font attribute on child XMTok elements
              let from_attr = xmarg.get_child_nodes().into_iter()
                .filter(|n| n.get_type() == Some(NodeType::ElementNode))
                .find_map(|n| {
                  let attr = n.get_attribute("font");
                  if attr.is_some() { return attr; }
                  let node_font = document.get_node_font(&n);
                  node_font.get_shape().and_then(|s|
                    if s.as_ref() == "italic" { Some("italic".to_string()) } else { None }
                  )
                });
              if from_attr.is_some() {
                from_attr
              } else {
                // Check the XMArg's stored box font (pre-specialization)
                document.get_node_box(xmarg).and_then(|tbox| {
                  tbox.get_font().ok().flatten().and_then(|font| {
                    if font.get_family().map(|f| f.as_ref() == "math").unwrap_or(false) {
                      Some("italic".to_string())
                    } else {
                      None
                    }
                  })
                })
              }
            };
            document.open_element(qname, None, None)?;
            if let Some(ref font) = font_attr {
              // Create ltx:text and set font attribute directly
              // (open_element skips "font" key in attributes hash)
              let mut text_node = document.open_element("ltx:text", None, None)?;
              document.set_attribute(&mut text_node, "font", font)?;
              document.get_node_mut().append_text(&text)?;
              document.close_element("ltx:text")?;
            } else {
              document.get_node_mut().append_text(&text)?;
            }
            document.close_element(qname)?;
            replaced = true;
          }
        }
      }
      if !replaced {
        // should never happen, but just in case: put the math node back
        document.get_node_mut().add_child(math)?;
      }
    }
  );
});

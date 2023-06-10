use crate::package::*;
static SCRIPT_NAME_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\\@@(FLOATING|POST)(SUBSCRIPT|SUPERSCRIPT)$").unwrap());

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
pub fn is_script(object: &Digested, _state: &State) -> Option<(String, Catcode)> {
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
            cap.get(1).map_or("", |m| m.as_str()).to_owned(),
            if cap.get(2).map_or("", |m| m.as_str()) == "SUBSCRIPT" {
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
fn script_handler(stomach: &mut Stomach, cc: Catcode, state: &mut State) -> Result<Vec<Digested>> {
  //   let mut gullet = stomach.get_gullet_mut();
  //   gullet.skip_spaces(state);
  let font = state.lookup_font().unwrap();
  if font.get_mathstyle().is_some() {
    let mut putback = VecDeque::new();
    let mut nscripts = 0;

    let mut cs = if cc == Catcode::SUPER {
      "\\@@FLOATINGSUPERSCRIPT"
    } else {
      "\\@@FLOATINGSUBSCRIPT"
    };
    let mut prevscript = None;
    let mut prevspace = false;
    let mut base = None;
    // Check preceding boxes to determine possible attachment (floating vs post),
    // Note that this analysis has to be done now (or sometime like it) before grouping lists go
    // away; and whether there are conflicting preceding scripts, which is an error
    // Parsing is too late!
    while let Some(prev) = stomach.box_list.pop() {
      if prev.get_property_bool("isSpace") {
        prevspace = true; // a space avoids double-scripts
        putback.push_front(prev); // put back? assuming it will add rpadding to previous???
        continue;
      } else if prev.is_empty() {
        // If empty, the script floats, can't conflict, but don't put back
        break;
      } else if let Some(prevop) = is_script(&prev, state) {
        if prevop.1 == cc {
          // Whoops, duplicated; better use FLOATING
          putback.push_front(prev);
          let lcode = if prevop.1 == Catcode::SUPER {
            "superscript"
          } else {
            "subscript"
          };
          if !prevspace {
            Error!(
              "unexpected",
              s!("double-{}", lcode),
              stomach,
              state,
              s!("Double {}", lcode)
            );
          }
          cs = if cc == Catcode::SUPER {
            "\\@@FLOATINGSUPERSCRIPT"
          } else {
            "\\@@FLOATINGSUBSCRIPT"
          };
          break;
        } else {
          // Else, is OK (so far) assume POST (it will stack previous script)
          prevscript = Some(prev.clone()); // we'll overlap the width of the previous.
          putback.push_front(prev);
          cs = if cc == Catcode::SUPER {
            "\\@@POSTSUPERSCRIPT"
          } else {
            "\\@@POSTSUBSCRIPT"
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
          "\\@@POSTSUPERSCRIPT"
        } else {
          "\\@@POSTSUBSCRIPT"
        };
        break;
      }
    }
    stomach.box_list.extend(putback);
    MergeFont!(scripted => true, state);
    // Now, get following boxes (may have to process several tokens!)
    let mut stuff = Vec::new();
    while let Some(tok) = stomach
      .get_gullet_mut()
      .read_x_token(Some(false), false, state)?
    {
      stuff = stomach.invoke_token(&tok, state)?;
      if !stuff.is_empty() {
        break;
      }
    }
    if stuff.is_empty() {
      Error!(
        "expected",
        "{",
        stomach,
        state,
        "Missing sub/superscript argument"
      ); //$gullet->showUnexpected);
      stuff.push(Digested::default());
    }
    let script = stuff.remove(0); // ONLY the first box is the script!

    if !script.is_empty() {
      let mut properties = stored_map!(
        "isMath" => true,
        "base"        => if let Some(b) = base { Stored::Digested(b) }
          else { Stored::None },                      // for sizing/positioning
        "scriptlevel" => stomach.get_script_level(state),
        "level"       => stomach.get_boxing_level()
      );
      if let Some(pvs) = prevscript {
        properties.insert("prevscript".to_string(), pvs.into());
      }
      if let Some(font) = script.get_font(state)? {
        properties.insert("font".to_string(), font.into());
      }
      let mut with_script = vec![Digested::from(Whatsit {
        definition: state.lookup_definition(&T_CS!(cs)).unwrap(),
        args: vec![Some(script)],
        properties,
        // TODO:
        // locator: stomach.get_gullet().get_locator(),
        ..Whatsit::default()
      })];
      with_script.extend(stuff);
      stuff = with_script;
    }
    state.assign_font(font, Some(Scope::Local)); // revert
    Ok(stuff)
  } else {
    let c = if cc == Catcode::SUPER { '^' } else { '_' };
    Error!(
      "Unexpected",
      c,
      stomach,
      state,
      format!("Script {} can only appear in math mode", c)
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
      HashMap::default(),
      state,
    ))])
  }
}

// The `argument' to a sub or superscript will typically be processed as a box,
// and either has braces, or is something that results in a single box.
// When we revert these, we DON'T want to wrap extra braces around, because they'll accumulate;
// at the least they're ugly; in some applications they affect "round trip" processing.
// OTOH, direct use of \@@POSTSUPERSCRIPT, etal, MAY need to have extra braces around them.
// So, when reverting, we're going to a bit of extra trouble to make sure we have ONE set
// of braces, but no extras!!  [Worry about lists of lists...]
pub fn revert_script(script: &Digested, state: &State) -> Result<Vec<Token>> {
  let tokens = script.revert(state)?;
  let mut ts = tokens.unlist();
  // let mut level = 0;
  if ts.len() > 1
    && ts.get(0).unwrap().code == Catcode::BEGIN
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
  state: &mut State,
) -> Result<(Dimension, Dimension, Dimension)> {
  eprintln!("SCRIPT SIZER IS ON!");
  // NOTE: Currently, the mathstyle is NOT reflected in the font of the script!!!!
  // Or is it now ?????
  // [unless it's different from the 'expected' style!!!]
  let script_size = script.clone().get_size(None, state)?;
  let (mut ws, mut hs, mut ds) = (
    script_size.0.value_of() as f64,
    script_size.1.value_of() as f64,
    script_size.2.value_of() as f64,
  );
  ws *= 0.8;
  hs *= 0.8;
  ds *= 0.8; // HACK!@!!
  let (wb, hb, db) = if let Some(Stored::Digested(ref base)) = base_opt {
    let base_size = base.clone().get_size(None, state)?;
    (
      base_size.0.value_of() as f64,
      base_size.1.value_of() as f64,
      base_size.2.value_of() as f64,
    )
  } else {
    let nominal_size = state.lookup_font().unwrap().get_nominal_size();
    (
      nominal_size.0.value_of() as f64,
      nominal_size.1.value_of() as f64,
      nominal_size.2.value_of() as f64,
    )
  };
  let w;
  let (mut h, mut d) = (0.0, 0.0);
  // Fishing for the scriptpos on the base (if any)
  let inferred_pos = if pos.is_empty() {
    if let Some(Stored::Digested(ref base)) = base_opt {
      let base_pos = base
        .get_property("scriptpos")
        .map(|s| s.to_string())
        .unwrap_or_default();
      if base_pos.is_empty() {
        Cow::Borrowed("post")
      } else {
        Cow::Owned(base_pos)
      }
    } else {
      Cow::Borrowed("post")
    }
  } else {
    Cow::Borrowed("post")
  };
  if inferred_pos == "mid" {
    w = (ws - wb).max(0.0); // as if max width of base & script
    if op == "SUPERSCRIPT" {
      h = hb + ds + hs;
    } else {
      d = db + hs + ds;
    }
  } else {
    // as if max of width & prev script's width
    let wp = if let Some(Stored::Digested(ref prev)) = prev_opt {
      prev.get_width(None, state)?.unwrap_or_default().value_of() as f64
    } else {
      0.0
    };
    w = (ws - wp).max(0.0);
    if op == "SUPERSCRIPT" {
      h = hb + hs / 2.0;
    } else {
      d = hs / 2.0 + ds;
    }
  }
  Ok((
    Dimension::new_f64(w),
    Dimension::new_f64(h),
    Dimension::new_f64(d),
  ))
}

LoadDefinitions!(state, {
  // TODO: Should I add a special macro case that takes an arbitrary token as argument?
  // DefPrimitiveT ?
  def_primitive(
    T_SUPER!(),
    None,
    Some(Rc::new(
      |stomach: &mut Stomach, _args: Vec<ArgWrap>, state: &mut State| {
        script_handler(stomach, Catcode::SUPER, state)
      },
    )),
    PrimitiveOptions::default(),
    state,
  );
  def_primitive(
    T_SUB!(),
    None,
    Some(Rc::new(
      |stomach: &mut Stomach, _args: Vec<ArgWrap>, state: &mut State| {
        script_handler(stomach, Catcode::SUB, state)
      },
    )),
    PrimitiveOptions::default(),
    state,
  );

  // NOTE: The When reverting these, the
  DefConstructor!("\\@@POSTSUPERSCRIPT InScriptStyle",r###"
  <ltx:XMApp role="POSTSUPERSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Superscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###,
    reversion => sub[_whatsit,args,state] {
      unref!(args=>arg);
      Ok(Tokens!(T_SUPER!(), revert_script(arg,state)?)) },
    sizer => sub[w,state] {
      script_sizer(w.get_arg(1).unwrap(), w.get_property("base").as_deref(),
        w.get_property("prevscript").as_deref(), "SUPERSCRIPT", "post", state) }
  );

  DefConstructor!("\\@@POSTSUBSCRIPT InScriptStyle",r###"
  <ltx:XMApp role="POSTSUBSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Subscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###
    ,
    reversion => sub[_whatsit,args,state] {
      unref!(args=>arg);
      Ok(Tokens!(T_SUB!(), revert_script(arg,state)?)) },
    sizer => sub[w,state] {
      script_sizer(w.get_arg(1).unwrap(), w.get_property("base").as_deref(),
        w.get_property("prevscript").as_deref(), "SUBSCRIPT", "post", state) }
  );

  DefConstructor!("\\@@FLOATINGSUPERSCRIPT InScriptStyle",r###"
  <ltx:XMApp role="FLOATSUPERSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Superscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###,
    reversion => sub[_whatsit,args,state] {
      unref!(args=>arg);
      Ok(Tokens!(T_BEGIN!(), T_END!(), T_SUPER!(), revert_script(arg,state)?)) }
    sizer => sub[w,state] {
      script_sizer(w.get_arg(1).unwrap(), None, None, "SUPERSCRIPT", "post", state) }
  );
  DefConstructor!("\\@@FLOATINGSUBSCRIPT InScriptStyle",r###"
  <ltx:XMApp role="FLOATSUBSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Subscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###,
    reversion => sub[_whatsit,args,state] {
      unref!(args=>arg);
      Ok(Tokens!(T_BEGIN!(), T_END!(), T_SUB!(), revert_script(arg,state)?)) }
      sizer => sub[w,state] {
        script_sizer(w.get_arg(1).unwrap(), None, None, "SUBSCRIPT", "post", state) }
  );

  DefMacro!("'", sub[gullet,_args,state] {
    let mut sup = vec![T_CS!("\\prime")];
    // Collect up all ', convering to \prime
    while gullet.if_next(&T_OTHER!("'"), state)? {
      gullet.read_token(state)?;
      sup.push(T_CS!("\\prime"));
    }
    // Combine with any following superscript!
    // However, this is semantically screwed up!
    // We really need to set up separate superscripts, but at same level!
    if TOKEN_SUPER.with(|ts| gullet.if_next(ts, state))? {
      gullet.read_token(state)?;
      sup.extend(gullet.read_arg(state)?.unlist());
    }
    Tokens!(T_SUPER!(), T_BEGIN!(), sup, T_END!())
  },
  mathactive => true); // Only in math!

  DefMacro!("\\active@math@prime", sub[gullet,(),state] {
    let mut sup = vec![T_CS!("\\prime")];
    // Collect up all ', convering to \prime
    let prime_token = T_OTHER!("\'");
    while gullet.if_next(&prime_token, state)? {
      gullet.read_token(state)?;
      sup.push(T_CS!("\\prime"));
    }
    // Combine with any following superscript!
    // However, this is semantically screwed up!
    // We really need to set up separate superscripts, but at same level!
    if gullet.if_next(&T_SUPER!(), state)? {
      gullet.read_token(state)?;
      let arg = gullet.read_arg(state)?;
      let arg_tks = arg.unlist();
      sup.extend(arg_tks);
    }
    let mut activated = vec![T_SUPER!(), T_BEGIN!()];
    activated.extend(sup);
    activated.push(T_END!());
    activated
  },
  locked => true);    // Only in math!
  // TODO
  // AssignMathcode!("'" => 0x8000);
  Let!("'", "\\active@math@prime");
});

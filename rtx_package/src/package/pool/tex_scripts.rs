use crate::package::*;
lazy_static! {
  static ref SCRIPT_NAME_RE: Regex = Regex::new(r"^\\@@(FLOATING|POST)(SUBSCRIPT|SUPERSCRIPT)$").unwrap();
}
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

// Note that this is also being used by alignment.
//
// TODO: We may want to rename the auxiliary function - there is a standard
// rust `.is_empty()` call that implies a very strict "no elements" semantics
// for e.g. vectors and strings.
// Maybe "is_invisible" or "without_ink" or ...
pub fn is_empty(digested: &Digested, state: &State) -> bool {
  use DigestedData::*;
  if digested.get_property_bool("isEmpty") || digested.get_property_bool("isSpace") {
    // A space-like thing
    true
  } else {
    match digested.data() {
      TBox(tbox) => match tbox.get_string(state) {
        Ok(s) => s.trim().is_empty(),
        _ => true,
      },
      List(list) => list.boxes.iter().all(|b| is_empty(b, state)),
      Whatsit(ws_arc) => {
        let ws = ws_arc.read().unwrap();
        *(*ws).get_definition() == *state.lookup_definition(&T_BEGIN!()).unwrap() &&
          ws.get_body().unwrap_or_default().all(|b| is_empty(b, state))
      },
      Comment(_) => true,
      _ => unimplemented!()
    }
  }
}

// Remember a "safe" way to test a script Whatsit.
// Returns [ (FLOATING|POST) , (SUBSCRIPT|SUPERSCRIPT) ] or nothing
pub fn is_script(object: &Digested, state: &State) -> Option<(String, Catcode)> {
  let box_opt = match object.data() {
    DigestedData::List(obj) => obj.boxes.last(),
    _ => Some(object),
  };
  if let Some(boxobj) = box_opt {
    if let DigestedData::Whatsit(ref obj) = boxobj.data() {
      // careful w/alias in getCSName!
      let name = obj.read().unwrap().get_definition().get_cs().get_cs_name().to_string();
      SCRIPT_NAME_RE.captures(&name).map(|cap| {
        (
          cap.get(1).map_or("", |m| m.as_str()).to_owned(),
          if cap.get(2).map_or("", |m| m.as_str()) == "SUBSCRIPT" {
            Catcode::SUB
          } else {
            Catcode::SUPER
          },
        )
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
// in the new codebase, so as to avoid both 1) cloning and 2) mutably referencing the same Digested object from multiple unrelated pieces of code.
//
fn script_handler(stomach: &mut Stomach, cc: Catcode, state: &mut State) -> Result<Vec<Digested>> {
  //   let mut gullet = stomach.get_gullet_mut();
  //   gullet.skip_spaces(state);
  let font = state.lookup_font().unwrap();
  if let Some(style) = font.get_mathstyle() {
    let mut putback = VecDeque::new();
    let mut nscripts = 0;

    let mut cs = if cc == Catcode::SUPER {
      "\\@@FLOATINGSUPERSCRIPT"
    } else {
      "\\@@FLOATINGSUBSCRIPT"
    };
    let mut _prevscript = None;
    let mut prevspace = false;
    let mut _base = None;
    // Check preceding boxes to determine possible attachment (floating vs post),
    // Note that this analysis has to be done now (or sometime like it) before grouping lists go away;
    // and whether there are conflicting preceding scripts, which is an error
    // Parsing is too late!
    while let Some(prev) = stomach.box_list.pop() {
      if prev.get_property_bool("isSpace") {
        prevspace = true; // a space avoids double-scripts
        putback.push_front(prev); // put back? assuming it will add rpadding to previous???
        continue;
      } else if is_empty(&prev, state) {
        // If empty, the script floats, can't conflict, but don't put back
        break;
      } else if let Some(prevop) = is_script(&prev, state) {
        if prevop.1 == cc {
          // Whoops, duplicated; better use FLOATING
          putback.push_front(prev);
          let lcode = if prevop.1 == Catcode::SUPER { "superscript" } else { "subscript" };
          if !prevspace {
            Error!("unexpected", s!("double-{}", lcode), stomach, state, s!("Double {}", lcode));
          }
          cs = if cc == Catcode::SUPER {
            "\\@@FLOATINGSUPERSCRIPT"
          } else {
            "\\@@FLOATINGSUBSCRIPT"
          };
          break;
        } else {
          // Else, is OK (so far) assume POST (it will stack previous script)
          _prevscript = Some(prev.clone()); // we'll overlap the width of the previous.
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
        _base = Some(prev.clone());
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
    while let Some(tok) = stomach.get_gullet_mut().read_x_token(Some(false), false, state)? {
      stuff = stomach.invoke_token(&tok, state)?;
      if !stuff.is_empty() {
        break;
      }
    }
    if stuff.is_empty() {
      Error!("expected", "{", stomach, state, "Missing sub/superscript argument"); //$gullet->showUnexpected);
      stuff.push(Digested::default());
    }
    let script = stuff.remove(0); // ONLY the first box is the script!
    if !is_empty(&script, state) {
      let mut properties = stored_map!(
        "isMath" => true
        // TODO:
        // "level"       => stomach.get_boxing_level(),
        // "scriptlevel" => stomach.get_script_level(),
        //"base"        => base,                      // for sizing/positioning
        //"prevscript"  => prevscript
      );
      if let Some(font) = script.get_font() {
        properties.insert("font".to_string(), font.into());
      }
      let mut with_script = vec![Digested::from(Whatsit {
        definition: state.lookup_definition(&T_CS!(cs)).unwrap(),
        args: vec![Some(script)],
        properties,
        //         locator     => $gullet->getLocator,
        ..Whatsit::default()
      })];
      with_script.append(&mut stuff);
      stuff = with_script;
    }
    state.assign_font(font, Some(Scope::Local)); // revert
    Ok(stuff)
  } else {
    let c = if cc == Catcode::SUPER { '^' } else { '_' };
    Error!("Unexpected", c, stomach, state, s!("Script {} can only appear in math mode", c));
    let placeholder = if cc == Catcode::SUPER { T_SUPER!() } else { T_SUB!() };
    Ok(vec![Digested::from(Tbox::new(
      c.to_string(),
      None,
      None,
      Tokens!(placeholder),
      HashMap::new(),
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
  let mut level = 0;
  if ts.len() > 1 && ts.get(0).unwrap().code == Catcode::BEGIN && ts.last().unwrap().code == Catcode::END {
    Ok(ts)
  } else {
    let mut wrapped = vec![T_BEGIN!()];
    wrapped.append(&mut ts);
    wrapped.push(T_END!());
    Ok(wrapped)
  }
}

LoadDefinitions!(state, {
  // TODO: Should I add a special macro case that takes an arbitrary token as argument? DefPrimitiveT ?
  def_primitive(
    T_SUPER!(),
    None,
    Some(Arc::new(|stomach: &mut Stomach, _args: Vec<ArgWrap>, state: &mut State| {
      script_handler(stomach, Catcode::SUPER, state)
    })),
    PrimitiveOptions::default(),
    state,
  );
  def_primitive(
    T_SUB!(),
    None,
    Some(Arc::new(|stomach: &mut Stomach, _args: Vec<ArgWrap>, state: &mut State| {
      script_handler(stomach, Catcode::SUB, state)
    })),
    PrimitiveOptions::default(),
    state,
  );

  // NOTE: The When reverting these, the
  DefConstructor!("\\@@POSTSUPERSCRIPT InScriptStyle",r###"
  <ltx:XMApp role="POSTSUPERSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Superscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###,
    reversion => sub[whatsit,args,state] {
      unref!(args=>arg);
      Ok(Tokens!(T_SUPER!(), revert_script(arg,state)?)) }
    // sizer     => sub { script_sizer($_[0]->getArg(1), $_[0].get_property("base"),
    //     $_[0].get_property("prevscript"), "SUPERSCRIPT", "post"); }
  );

  DefConstructor!("\\@@POSTSUBSCRIPT InScriptStyle",r###"
  <ltx:XMApp role="POSTSUBSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Subscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###
    ,
    reversion => sub[whatsit,args,state] {
      unref!(args=>arg);
      Ok(Tokens!(T_SUB!(), revert_script(arg,state)?)) }
    // sizer     => sub { script_sizer($_[0]->getArg(1), $_[0].get_property("base"),
    //     $_[0].get_property("prevscript"),
    //     "SUBSCRIPT", "post"); }
  );

  DefConstructor!("\\@@FLOATINGSUPERSCRIPT InScriptStyle",r###"
  <ltx:XMApp role="FLOATSUPERSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Superscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###,
    reversion => sub[whatsit,args,state] {
      unref!(args=>arg);
      Ok(Tokens!(T_BEGIN!(), T_END!(), T_SUPER!(), revert_script(arg,state)?)) }
    // sizer     => sub { script_sizer($_[0]->getArg(1), undef, undef, "SUPERSCRIPT", 'post"); }
  );
  DefConstructor!("\\@@FLOATINGSUBSCRIPT InScriptStyle",r###"
  <ltx:XMApp role="FLOATSUBSCRIPT" scriptpos="?#scriptpos(#scriptpos)(#scriptlevel)">
    <ltx:XMArg rule="Subscript">#1</ltx:XMArg>
  </ltx:XMApp>
  "###,
    reversion => sub[whatsit,args,state] {
      unref!(args=>arg);
      Ok(Tokens!(T_BEGIN!(), T_END!(), T_SUB!(), revert_script(arg,state)?)) }
    // sizer     => sub { script_sizer($_[0]->getArg(1), undef, undef, 'SUBSCRIPT', 'post"); }
  );

  DefMacro!("'", sub[gullet,args,state] {
    let mut sup = vec![T_CS!("\\prime")];
    // Collect up all ', convering to \prime
    while gullet.if_next(T_OTHER!("'"), state)? {
      gullet.read_token(state);
      sup.push(T_CS!("\\prime"));
    }
    // Combine with any following superscript!
    // However, this is semantically screwed up!
    // We really need to set up separate superscripts, but at same level!
    if gullet.if_next(T_SUPER!(), state)? {
      gullet.read_token(state);
      sup.extend(gullet.read_arg(state)?.unlist());
    }
    Tokens!(T_SUPER!(), T_BEGIN!(), sup, T_END!())
  },
  mathactive => true); // Only in math!
});

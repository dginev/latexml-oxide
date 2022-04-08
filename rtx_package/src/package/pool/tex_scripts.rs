use crate::package::*;
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
pub fn is_empty(digested: Digested, state: &State) -> bool {
  if digested.get_property_bool("isEmpty")
    || digested.get_property_bool("isSpace") { // A space-like thing
    true
  } else if let Digested::TBox(tbox) = digested {
    let s = tbox.get_string();
    s.trim().is_empty()
  } else if let Digested::List(list) = digested {
    list.unlist().any(|b| !is_empty(b, state))
  }
  else if let Digested::Whatsit(ws_arc) = digested {
    let ws = ws_arc.read().unwrap();
    if *(*ws).get_definition() == state.lookup_definition(&T_BEGIN!())
      && ws.get_body().unlist().any(|b| !is_empty(b, state)) {
      true
    } else {
      false
    }
  } else {
    false
  }
}

// Remember a "safe" way to test a script Whatsit.
// Returns [ (FLOATING|POST) , (SUBSCRIPT|SUPERSCRIPT) ] or nothing
pub fn is_script(object: Digested, state: &State) -> Option<(String,String)> {
  // if (ref $object eq 'LaTeXML::Core::List') {
  //   $object = [$object->unlist]->[-1]; }
  // if ((ref $object eq 'LaTeXML::Core::Whatsit')    # careful w/alias in getCSName!
  //   && ($object->getDefinition->getCS->getCSName =~ /^\\@@(FLOATING|POST)(SUBSCRIPT|SUPERSCRIPT)$/)) {
  //   return [$1, $2]; }
  // return; }
  None
}


fn script_handler(stomach: &mut Stomach, cc: Catcode, state: &mut State) -> Digested {
//   let mut gullet = stomach.get_gullet_mut();
//   gullet.skip_spaces(state);
  let font     = state.lookup_font().unwrap();
  if let Some(style)    = font.get_mathstyle() {
    let mut putback  = VecDeque::new();
    let mut nscripts = 0;

    let mut cs = if cc == Catcode::SUPER { "\\@@FLOATINGSUPERSCRIPT" }
      else { "\\@@FLOATINGSUBSCRIPT" };
    let mut prevscript = None;
    let mut prevspace = false;
    let mut base = None;
    // Check preceding boxes to determine possible attachment (floating vs post),
    // Note that this analysis has to be done now (or sometime like it) before grouping lists go away;
    // and whether there are conflicting preceding scripts, which is an error
    // Parsing is too late!
    while let Some(prev) = stomach.box_list.pop() {
      if prev.get_property_bool("isSpace") {
        prevspace = true;              // a space avoids double-scripts
        putback.push_front(prev); // put back? assuming it will add rpadding to previous???
        continue;
      } else if is_empty(prev, state) { // If empty, the script floats, can't conflict, but don't put back
        break;
      } else if let Some(prevop) = is_script(prev) {
        putback.push_front(prev);
        if prevop.code == cc { // Whoops, duplicated; better use FLOATING
          let lcode = prevop.code.to_string().to_lowercase();
          if !prevspace {
            Error!("unexpected", s!("double-{}", lcode), stomach, state, s!("Double {}", lcode)); }
          cs = if cc == Catcode::SUPER { "\\@@FLOATINGSUPERSCRIPT" }
            else { "\\@@FLOATINGSUBSCRIPT" };
          break;
        } else { // Else, is OK (so far) assume POST (it will stack previous script)
          prevscript = Some(prev); // we'll overlap the width of the previous.
          cs = if cc == Catcode::SUPER { "\\@@POSTSUPERSCRIPT" }
            else { "\\@@POSTSUBSCRIPT" };
        }
        // if we hit a FLOATING script, terminate, as the floating empty group avoids double scripts
        if prevop.text == "FLOATING" {
          break;
        }
        nscripts+=1;
        if nscripts > 1 { break; }
      } else {
        //  We found something "normal", so assume we'll attach to it, and we're done.
        base = Some(prev);
        putback.push_front(prev);
        cs = if cc == Catcode::SUPER { "\\@@POSTSUPERSCRIPT" }
            else { "\\@@POSTSUBSCRIPT" };
        break;
      }
    }
    stomach.box_list.extend(putback);

    // MergeFont(scripted => 1);

    // Now, get following boxes (may have to process several tokens!)
//     my @stuff = ();
//     while (my $tok = $gullet->readXToken(0)) {
//       @stuff = $stomach->invokeToken($tok);
//       last if @stuff; }
//     if (!@stuff) {
//       Error('expected', '{', $stomach, "Missing sub/superscript argument", $gullet->showUnexpected);
//       push(@stuff, Box()); }
//     my $script = shift(@stuff);    # ONLY the first box is the script!
//     unshift(@stuff,
//       LaTeXML::Core::Whatsit->new(LookupDefinition(T_CS($cs)), [$script],
//         locator     => $gullet->getLocator,
//         font        => $script->getFont, isMath => 1,
//         level       => $stomach->getBoxingLevel,
//         scriptlevel => $stomach->getScriptLevel,
//         base        => $base,                      # for sizing/positioning
//         prevscript  => $prevscript))
//       unless IsEmpty($script);
//     AssignValue(font => $font);                    # revert
//     return @stuff; }
    unimplemented!();
  } else {
    let c = if cc == Catcode::SUPER { '^' } else { '_' };
    Error!("Unexpected", c, stomach, state, s!("Script {} can only appear in math mode", c));
    let placeholder = if cc == Catcode::SUPER { T_SUPER!() } else { T_SUB!() };
    Digested::TBox(Arc::new(
      Tbox::new(c.to_string(), None, None, Tokens!(placeholder), HashMap::new(), state)))
  }
}

LoadDefinitions!(state, {
// TODO: Should I add a special macro case that takes an arbitrary token as argument? DefPrimitiveT ?
def_primitive(T_SUPER!(), None,
  Arc::new(|stomach: &mut Stomach ,_args: Vec<Tokens>, state: &mut State| {
    script_handler(stomach, Catcode::SUPER, state);
    Ok(Vec::new())
  }), PrimitiveOptions::default(), state);
def_primitive(T_SUB!(), None,
  Arc::new(|stomach: &mut Stomach ,_args: Vec<Tokens>, state: &mut State| {
    script_handler(stomach, Catcode::SUB, state);
    Ok(Vec::new())
  }), PrimitiveOptions::default(), state);

});

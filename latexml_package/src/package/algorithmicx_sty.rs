use crate::prelude::*;

LoadDefinitions!({
  // Perl: algorithmicx.sty.ltxml
  // Was algorithmic.sty loaded? If so: BAIL immediately. (deeply incompatible)
  if state::has_meaning(&T_CS!("\\algorithmic")) {
    Warn!(
      "unexpected",
      "\\algorithmic",
      "Another package has already defined \\algorithmic, will not load algorithmicx.sty"
    );
    return Ok(());
  }

  // Load core, make a few redefinitions
  InputDefinitions!("algorithmicx", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  state::let_i(
    &T_CS!("\\lx@orig@algorithmic"),
    &T_CS!("\\algorithmic"),
    None,
  );
  DefMacro!(
    "\\algorithmic",
    "\\lx@setup@algorithmicx\\lx@orig@algorithmic"
  );

  DefPrimitive!("\\lx@setup@algorithmicx", sub [_args] {
    ResetCounter!("ALG@line");
    // If we are not within an algorithm environment, step the counter for its id's
    let in_algorithm = state::with_stacked_values_sym(pin!("current_environment"), |vals| {
      vals.iter().any(|v| {
        matches!(v, Stored::String(s) if arena::with(*s, |v| v == "algorithm"))
      })
    });
    if !in_algorithm {
      ref_step_id("algorithm")?;
    }
    state::let_i(&T_CS!("\\list"), &T_CS!("\\lx@algorithmicx@beginlist"), None);
    state::let_i(&T_CS!("\\endlist"), &T_CS!("\\lx@algorithmicx@endlist"), None);
    state::let_i(&T_CS!("\\item"), &T_CS!("\\lx@algorithmicx@item"), None);
    state::let_i(&T_CS!("\\hfill"), &T_CS!("\\lx@algorithmicx@hfill"), None);
  });

  // IGNORE \list 1st arg (we'll handle counter stepping in \item)
  DefMacro!(
    "\\lx@algorithmicx@beginlist{}{}",
    "\\lx@algorithmicx@beginlist@{#2}"
  );
  DefConstructor!("\\lx@algorithmicx@beginlist@{}", "<ltx:listing>");

  DefConstructor!("\\lx@algorithmicx@endlist", "</ltx:listing>",
    before_construct => sub[document] {
      document.maybe_close_element("ltx:listingline")?;
    }
  );

  // Empty lines still get an \item, but they're followed by \nointerlineskip!
  // We do NOT want to generate a listingline in those cases.
  DefMacro!(
    "\\lx@algorithmicx@item[]",
    "\\@ifnextchar\\nointerlineskip{}{\\lx@algorithmicx@@item}"
  );

  // This imitates \item; just opens the ltx:listingline, but somebody's got to close it.
  DefConstructor!("\\lx@algorithmicx@@item",
    "<ltx:listingline xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[_args] {
      // Perl: my $step = Digest(T_BEGIN, T_CS('\ALG@step'), T_END);
      let step = Digest!(Tokens::new(vec![
        T_BEGIN!(), T_CS!("\\ALG@step"), T_END!()
      ]))?;
      // Perl: my $id = Digest(T_CS('\theALG@line@ID'));
      let id = Digest!(Tokens::new(vec![T_CS!("\\theALG@line@ID")]))?;
      // Perl: my $tags = Digest(T_BEGIN,
      //   T_CS('\def'), T_CS('\fnum@ALG@line'), T_BEGIN, Tokens(Revert($step)), T_END,
      //   Invocation(T_CS('\lx@make@tags'), T_OTHER('ALG@line')), T_END);
      let step_revert = step.revert()?;
      let invocation = Invocation!("\\lx@make@tags", vec![Some(Tokens::new(vec![T_OTHER!("ALG@line")]))]);
      let mut tag_tokens = vec![T_BEGIN!(), T_CS!("\\def"), T_CS!("\\fnum@ALG@line"), T_BEGIN!()];
      tag_tokens.extend(step_revert.unlist());
      tag_tokens.push(T_END!());
      tag_tokens.extend(invocation.unlist());
      tag_tokens.push(T_END!());
      let tags = Digest!(Tokens::new(tag_tokens))?;

      Ok(stored_map!("id" => id, "tags" => tags))
    },
    before_construct => sub[document] {
      document.maybe_close_element("ltx:listingline")?;
    }
  );

  // Ideally, these appear within an algorithm environment, and we'd like to number lines within it.
  // BUT algorithm package isn't required, so define it here!
  NewCounter!("algorithm", "", idprefix => "alg");
  NewCounter!("ALG@line", "algorithm", idprefix => "l");

  // Hopefully this will only get used for right justifying a comment;
  // the ltx:text should autoclose at end of line?
  DefConstructor!(
    "\\lx@algorithmicx@hfill",
    "<ltx:text cssstyle='float:right'>"
  );

  // Protect against obsolete versions of algorithmicx source
  DefMacro!("\\ALG@g{}", "");
  DefMacro!("\\endALG@g", "");
});

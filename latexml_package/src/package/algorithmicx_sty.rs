use crate::prelude::*;

LoadDefinitions!({
  // Perl: algorithmicx.sty.ltxml
  // Was algorithmic.sty loaded? If so: BAIL immediately. (deeply incompatible)
  // NOTE: must use `is_defined_token` (Perl `IsDefined`) — not `has_meaning` —
  // because users routinely do `\let\algorithmic\relax` before loading
  // algpseudocode (which loads us via the algorithmicx chain) to opt out
  // of algorithmic.sty. `\let X \relax` is *defined* in the state machine
  // but is "LaTeX-y undefined" — Perl's `IsDefined` treats it as undefined,
  // so the bail does not fire and the algorithmicx setup proceeds.
  // Witness: arXiv:2603.09221 (`\let\algorithmic\relax` + algpseudocode).
  if is_defined_token(&T_CS!("\\algorithmic")) {
    Warn!(
      "unexpected",
      "\\algorithmic",
      "Another package has already defined \\algorithmic, will not load algorithmicx.sty"
    );
    // Defensive stubs for algorithmicx top-level commands so user
    // preambles that call \algdef / \algnewcommand / \algnewlanguage
    // / \alglanguage after the bail don't crash. The actual line-
    // formatting machinery is gone, but the preamble setup commands
    // need to gobble cleanly. Witness 2410.03000 (3 papers using
    // algorithmic + algpseudocode together).
    def_macro_noop("\\algdef OptionalKeyVals:algdef SkipSpaces {} [] [] {}")?;
    def_macro_noop("\\algblock [] {}{}")?;
    def_macro_noop("\\algcblock [] {}{}")?;
    def_macro_noop("\\algblockx [] {}{}")?;
    def_macro_noop("\\algcblockx [] {}{}")?;
    def_macro_noop("\\algnewlanguage{}")?;
    def_macro_noop("\\algdeflanguage{}")?;
    def_macro_noop("\\alglanguage{}")?;
    DefMacro!("\\algnewcommand", "\\newcommand");
    DefMacro!("\\algrenewcommand", "\\renewcommand");
    def_macro_noop("\\algdefaulttext[]{}")?;
    return Ok(());
  }

  // Load core, make a few redefinitions
  InputDefinitions!("algorithmicx", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  let_i(
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
    let in_algorithm = with_stacked_values_sym(pin!("current_environment"), |vals| {
      vals.iter().any(|v| {
        matches!(v, Stored::String(s) if with(*s, |v| v == "algorithm"))
      })
    });
    if !in_algorithm {
      ref_step_id("algorithm")?;
    }
    let_i(&T_CS!("\\list"), &T_CS!("\\lx@algorithmicx@beginlist"), None);
    let_i(&T_CS!("\\endlist"), &T_CS!("\\lx@algorithmicx@endlist"), None);
    let_i(&T_CS!("\\item"), &T_CS!("\\lx@algorithmicx@item"), None);
    let_i(&T_CS!("\\hfill"), &T_CS!("\\lx@algorithmicx@hfill"), None);
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
  def_macro_noop("\\ALG@g{}")?;
  def_macro_noop("\\endALG@g")?;
});

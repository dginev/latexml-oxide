use crate::package::*;

//======================================================================
// C.7.1 Math Mode Environments
//======================================================================

// # This provides {equation} with the capabilities for tags, nonumber, etc
// # even though stock LaTeX provides no means to override them.
// #   preset => boolean
// #   postset => boolean
// #   deferretract=>boolean
fn prepare_equation_counter(options: HashMap<String, Stored>, state: &mut State) {
  state.assign_value(
    "EQUATION_NUMBERING",
    Stored::HashStored(options),
    Some(Scope::Global),
  );
}

fn before_equation(stomach: &mut Stomach, state: &mut State) -> Result<()> {
  let mut has_preset = false;
  let mut is_numbered = false;
  let ctr = if let Some(Stored::HashStored(ref mut numbering)) =
    state.lookup_value_mut("EQUATION_NUMBERING")
  {
    numbering.insert("in_equation".to_owned(), true.into());
    // MaybePeekLabel();
    is_numbered = matches!(numbering.get("numbered"), Some(&Stored::Bool(true)));
    has_preset = numbering.contains_key("preset");
    match numbering.get("counter") {
      Some(Stored::String(v)) => arena::to_string(*v),
      Some(other) => panic!("eq counter should be stored as string, was instead: {other:?}"),
      _ => String::from("equation"),
    }
  } else {
    String::from("equation")
  };
  if has_preset {
    let mut tags = if is_numbered {
      ref_step_counter(&ctr, false, stomach, state)?
    } else {
      ref_step_id(&ctr, stomach, state)?
    };
    tags.insert("preset".to_owned(), true.into());
    state.assign_value("EQUATIONROW_TAGS", tags, Some(Scope::Global));
  } else {
    state.assign_value(
      "EQUATIONROW_TAGS",
      Stored::HashStored(HashMap::default()),
      Some(Scope::Global),
    );
  }
  let gullet = stomach.get_gullet_mut();
  state.let_i(
    &T_CS!("\\@@ENDDISPLAYMATH"),
    T_CS!("\\lx@eDM@in@equation"),
    None,
    gullet,
  );
  state.let_i(
    &T_CS!("\\@@BEGINDISPLAYMATH"),
    T_CS!("\\lx@bDM@in@equation"),
    None,
    gullet,
  );
  Ok(())
}

fn after_equation(stomach: &mut Stomach, whatsit: &mut Whatsit, state: &mut State) -> Result<()> {
  let mut ctr: Option<String> = None;
  let mut tags_numbered_update = false;
  let mut is_aligned = false;
  if let Some(Stored::HashStored(ref numbering)) = state.lookup_value("EQUATION_NUMBERING") {
    is_aligned = matches!(numbering.get("aligned"), Some(&Stored::Bool(true)));
    if let Some(Stored::HashStored(ref tags)) = state.lookup_value("EQUATIONROW_TAGS") {
      ctr = Some(
        tags
          .get("counter")
          .map_or_else(|| numbering.get("counter"), Some)
          .map(ToString::to_string)
          .unwrap_or_else(|| String::from("equation")),
      );

      if !matches!(tags.get("noretract"), Some(&Stored::Bool(true)))
        && (matches!(tags.get("retract"), Some(&Stored::Bool(true)))
          || (matches!(numbering.get("retract"), Some(&Stored::Bool(true)))
            && matches!(numbering.get("preset"), Some(&Stored::Bool(true)))
            && matches!(tags.get("preset"), Some(&Stored::Bool(true)))))
      {
        retract_equation(state);
      } else if matches!(numbering.get("postset"), Some(&Stored::Bool(true)))
        && matches!(tags.get("reset"), Some(&Stored::Bool(true)))
      {
        //   AssignValue(EQUATIONROW_TAGS => {
        //       ($$numbering{numbered} ? RefStepCounter($ctr) : RefStepID($ctr)) }, 'global'); }
        unimplemented!();
      } else if !matches!(tags.get("reset"), Some(&Stored::Bool(true)))
        && matches!(numbering.get("numbered"), Some(&Stored::Bool(true)))
      {
        tags_numbered_update = true;
      }
    }
  }

  if let Some(Stored::HashStored(ref mut numbering)) = state.lookup_value_mut("EQUATION_NUMBERING")
  {
    numbering.insert("in_equation".to_string(), Stored::Bool(false));
  }
  if tags_numbered_update {
    let invoked_tags = build_invocation(
      T_CS!("\\lx@make@tags"),
      vec![Some(Tokens::new(Explode!(ctr.unwrap())))],
      stomach.get_gullet_mut(),
      state,
    )?;
    let stored_tags_update = Stored::Digested(stomach.digest(invoked_tags, state)?);
    if let Some(Stored::HashStored(ref mut tags)) = state.lookup_value_mut("EQUATIONROW_TAGS") {
      // TODO: Invocation!() feels really awkward to use, should we reinvent it?
      // especially the magical `.into()` that it does behind the scenes is concerning.
      tags.insert("tags".to_string(), stored_tags_update);
    }
  }
  // Now install the tags in $whatsit or current Row, as appropriate.
  let props = match state.remove_value("EQUATIONROW_TAGS") {
    Some(Stored::HashStored(hs)) => hs,
    _ => HashMap::default(),
  };
  if is_aligned {
    unimplemented!();
  //   if (my $alignment = LookupValue('Alignment')) {
  //     my $row = $alignment->currentRow;
  //     $$row{id}   = $$props{id};
  //     $$row{tags} = $$props{tags}; }
  } else {
    whatsit.set_properties(props);
  }
  Ok(())
}

fn retract_equation(_state: &mut State) {
  unimplemented!();
}

LoadDefinitions!(state, {
  DefMacro!("\\@eqnnum", "(\\theequation)", locked => true);
  DefMacro!("\\fnum@equation", "\\@eqnnum");

  // Redefined from TeX.pool, since with LaTeX we presumably have a more complete numbering system
  DefConstructor!("\\@@BEGINDISPLAYMATH", "<ltx:equation xml:id='#id'>\
  <ltx:Math mode='display'>\
  <ltx:XMath>#body</ltx:XMath>\
  </ltx:Math>\
  </ltx:equation>",
  alias        => "$$",
  before_digest => sub[stomach, state] {
    stomach.begin_mode("display_math", state)?;
    if let Some(RegisterValue::Tokens(everymath_toks)) = state.lookup_register("\\everymath", Vec::new()) {
      let everymath_toks = everymath_toks.unlist();
      if !everymath_toks.is_empty() {
        stomach.get_gullet_mut().unread(Tokens::new(everymath_toks));
      }
    }
    if let Some(RegisterValue::Tokens(everydisplay_toks)) = state.lookup_register("\\everydisplay", Vec::new()) {
      let everydisplay_toks = everydisplay_toks.unlist();
      if !everydisplay_toks.is_empty() {
        stomach.get_gullet_mut().unread(Tokens::new(everydisplay_toks));
      }
    }
  },
  properties  => sub[stomach,_args,state] { ref_step_id("equation", stomach, state) },
  capture_body => true);

  DefEnvironment!("{displaymath}",
  "<ltx:equation xml:id='#id'><ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
  mode       => "display_math",
  properties   => sub[stomach, _args, state] { ref_step_id("equation", stomach, state) },
  locked     => true);
  DefEnvironment!("{math}",
    "<ltx:Math mode=\"inline\"><ltx:XMath>#body</ltx:XMath></ltx:Math>",
    mode => "inline_math"
  );
  // My first inclination is to Lock {math}, but it is surprisingly common to redefine it in silly
  // ways... So...?
  DefEnvironment!(
    "{equation}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    mode => "display_math",
    before_digest => sub[stomach, state] {
      prepare_equation_counter(stored_map!("numbered" => true, "preset" => true), state);
      before_equation(stomach, state)?;
    },
    after_digest_body => sub[stomach, whatsit, state] {
      after_equation(stomach, whatsit, state)?;
    },
    locked => true);

  DefMacro!("\\[", "\\@@BEGINDISPLAYMATH");
  DefMacro!("\\]", "\\@@ENDDISPLAYMATH");
  DefMacro!("\\(", "\\@@BEGININLINEMATH");
  DefMacro!("\\)", "\\@@ENDINLINEMATH");

  // Keep from expanding too early, if in alignments, or such.
  DefMacro!(
    T_CS!("\\ensuremath"),
    None,
    Tokens!(T_CS!("\\protect"), T_CS!("\\@ensuremath"))
  );
  DefMacro!("\\@ensuremath{}", sub[gullet, (stuff), state] {
    if state.lookup_bool("IN_MATH") {
      stuff.unlist()
    } else {
      let mut result = vec![T_MATH!()];
      result.extend(stuff.unlist());
      result.push(T_MATH!());
      result
    }
  });
});

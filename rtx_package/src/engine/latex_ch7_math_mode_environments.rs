use crate::prelude::*;

//======================================================================
// C.7.1 Math Mode Environments
//======================================================================

// # This provides {equation} with the capabilities for tags, nonumber, etc
// # even though stock LaTeX provides no means to override them.
// #   preset => boolean
// #   postset => boolean
// #   deferretract=>boolean
fn prepare_equation_counter(options: SymHashMap<Stored>) {
  state::assign_value(
    "EQUATION_NUMBERING",
    Stored::HashStored(options),
    Some(Scope::Global),
  );
}

fn before_equation() -> Result<()> {
  let mut has_preset = false;
  let mut is_numbered = false;
  let ctr = with_value_mut("EQUATION_NUMBERING", |val_opt|
    if let Some(Stored::HashStored(ref mut numbering)) = val_opt {
      numbering.insert("in_equation", true.into());
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
    });
  if has_preset {
    let mut tags = if is_numbered {
      ref_step_counter(&ctr, false)?
    } else {
      ref_step_id(&ctr)?
    };
    tags.insert("preset", true.into());
    state::assign_value("EQUATIONROW_TAGS", tags, Some(Scope::Global));
  } else {
    state::assign_value(
      "EQUATIONROW_TAGS",
      Stored::HashStored(SymHashMap::default()),
      Some(Scope::Global),
    );
  }
  state::let_i(
    &T_CS!("\\@@ENDDISPLAYMATH"),
    &T_CS!("\\lx@eDM@in@equation"),
    None
  );
  state::let_i(
    &T_CS!("\\@@BEGINDISPLAYMATH"),
    &T_CS!("\\lx@bDM@in@equation"),
    None
  );
  Ok(())
}

fn after_equation(whatsit: &mut Whatsit) -> Result<()> {
  let mut ctr: Option<String> = None;
  let mut tags_numbered_update = false;
  let mut is_aligned = false;
  with_value("EQUATION_NUMBERING", |eq_num_opt|
  if let Some(Stored::HashStored(ref numbering)) = eq_num_opt {
    is_aligned = matches!(numbering.get("aligned"), Some(&Stored::Bool(true)));
    with_value("EQUATIONROW_TAGS", |tags_opt| if let Some(Stored::HashStored(ref tags)) = tags_opt {
      ctr = Some(
        tags.get("counter")
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
        retract_equation();
      } else if matches!(numbering.get("postset"), Some(&Stored::Bool(true)))
        && matches!(tags.get("reset"), Some(&Stored::Bool(true)))
      {
        //   AssignValue(EQUATIONROW_TAGS => {
        //       ($$numbering{numbered} ? RefStepCounter($ctr) : RefStepID($ctr)) }, 'global'); }
        todo!();
      } else if !matches!(tags.get("reset"), Some(&Stored::Bool(true)))
        && matches!(numbering.get("numbered"), Some(&Stored::Bool(true)))
      {
        tags_numbered_update = true;
      }
    });
  });

  with_value_mut("EQUATION_NUMBERING", |eq_num_opt|
    if let Some(Stored::HashStored(ref mut numbering)) = eq_num_opt
    {
      numbering.insert("in_equation", Stored::Bool(false));
    });
  if tags_numbered_update {
    let invoked_tags = build_invocation(
      T_CS!("\\lx@make@tags"),
      vec![Some(Tokens::new(Explode!(ctr.unwrap())))]
      )?;
    let stored_tags_update = Stored::Digested(stomach::digest(invoked_tags)?);
    with_value_mut("EQUATIONROW_TAGS", |tags_opt|
      if let Some(Stored::HashStored(ref mut tags)) = tags_opt {
        // TODO: Invocation!() feels really awkward to use, should we reinvent it?
        // especially the magical `.into()` that it does behind the scenes is concerning.
        tags.insert("tags", stored_tags_update);
      }
    );
  }
  // Now install the tags in $whatsit or current Row, as appropriate.
  #[allow(clippy::manual_unwrap_or_default)]
  let props = match state::remove_value("EQUATIONROW_TAGS") {
    Some(Stored::HashStored(hs)) => hs,
    _ => SymHashMap::default(),
  };
  if is_aligned {
    todo!();
  //   if (my $alignment = LookupValue('Alignment')) {
  //     my $row = $alignment->currentRow;
  //     $$row{id}   = $$props{id};
  //     $$row{tags} = $$props{tags}; }
  } else {
    whatsit.set_properties(props);
  }
  Ok(())
}

fn retract_equation() {
  todo!();
}

LoadDefinitions!({
  DefMacro!("\\@eqnnum", "(\\theequation)", locked => true);
  DefMacro!("\\fnum@equation", "\\@eqnnum");

  // Redefined from TeX.pool, since with LaTeX we presumably have a more complete numbering system
  DefConstructor!("\\@@BEGINDISPLAYMATH", "<ltx:equation xml:id='#id'>\
  <ltx:Math mode='display'>\
  <ltx:XMath>#body</ltx:XMath>\
  </ltx:Math>\
  </ltx:equation>",
  alias        => "$$",
  before_digest => {
    begin_mode("display_math")?;
    if let Some(RegisterValue::Tokens(everymath_toks)) = state::lookup_register("\\everymath", Vec::new())? {
      let everymath_toks = everymath_toks.unlist();
      if !everymath_toks.is_empty() {
        gullet::unread(Tokens::new(everymath_toks));
      }
    }
    if let Some(RegisterValue::Tokens(everydisplay_toks)) = state::lookup_register("\\everydisplay", Vec::new())? {
      let everydisplay_toks = everydisplay_toks.unlist();
      if !everydisplay_toks.is_empty() {
        gullet::unread(Tokens::new(everydisplay_toks));
      }
    }
  },
  properties  => { ref_step_id("equation") },
  capture_body => true);

  DefEnvironment!("{displaymath}",
  "<ltx:equation xml:id='#id'><ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
  mode       => "display_math",
  properties   => { ref_step_id("equation") },
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
    before_digest => {
      prepare_equation_counter(stored_map!("numbered" => true, "preset" => true));
      before_equation()?;
    },
    after_digest_body => sub[whatsit] {
      after_equation(whatsit)?;
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
  DefMacro!("\\@ensuremath{}", sub[(stuff)] {
    if lookup_bool("IN_MATH") {
      stuff.unlist()
    } else {
      let mut result = vec![T_MATH!()];
      result.extend(stuff.unlist());
      result.push(T_MATH!());
      result
    }
  });

  // Since the arXMLiv folks keep wanting ids on all math, let's try this!
  Tag!("ltx:Math", after_open => sub[document, node] {
    document.generate_id(node, "m")?;
  });
  
});

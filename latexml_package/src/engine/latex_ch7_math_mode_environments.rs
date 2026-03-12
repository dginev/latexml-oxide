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
  let ctr = with_value_mut("EQUATION_NUMBERING", |val_opt| {
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
    }
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
    &T_CS!("\\lx@end@display@math"),
    &T_CS!("\\lx@eDM@in@equation"),
    None,
  );
  state::let_i(
    &T_CS!("\\lx@begin@display@math"),
    &T_CS!("\\lx@bDM@in@equation"),
    None,
  );
  Ok(())
}

fn after_equation(whatsit: &mut Whatsit) -> Result<()> {
  // Phase 1: Gather all needed data from state (immutable borrows only)
  enum EqAction { Retract, Postset, TagsUpdate, None }
  let mut action = EqAction::None;
  let mut is_aligned = false;
  let mut is_numbered_for_postset = false;
  let mut ctr = String::from("equation");

  with_value("EQUATION_NUMBERING", |eq_num_opt| {
    if let Some(Stored::HashStored(ref numbering)) = eq_num_opt {
      is_aligned = matches!(numbering.get("aligned"), Some(&Stored::Bool(true)));
      is_numbered_for_postset =
        matches!(numbering.get("numbered"), Some(&Stored::Bool(true)));
      with_value("EQUATIONROW_TAGS", |tags_opt| {
        if let Some(Stored::HashStored(ref tags)) = tags_opt {
          ctr = tags
            .get("counter")
            .map_or_else(|| numbering.get("counter"), Some)
            .map(ToString::to_string)
            .unwrap_or_else(|| String::from("equation"));

          if !matches!(tags.get("noretract"), Some(&Stored::Bool(true)))
            && (matches!(tags.get("retract"), Some(&Stored::Bool(true)))
              || (matches!(numbering.get("retract"), Some(&Stored::Bool(true)))
                && matches!(numbering.get("preset"), Some(&Stored::Bool(true)))
                && matches!(tags.get("preset"), Some(&Stored::Bool(true)))))
          {
            action = EqAction::Retract;
          } else if matches!(numbering.get("postset"), Some(&Stored::Bool(true)))
            && !matches!(tags.get("reset"), Some(&Stored::Bool(true)))
          {
            action = EqAction::Postset;
          } else if !matches!(tags.get("reset"), Some(&Stored::Bool(true)))
            && matches!(numbering.get("numbered"), Some(&Stored::Bool(true)))
          {
            action = EqAction::TagsUpdate;
          }
        }
      });
    }
  });

  // Phase 2: Act on gathered data (borrows released, safe to mutate state)
  match action {
    EqAction::Retract => {
      retract_equation();
    },
    EqAction::Postset => {
      // Perl: AssignValue(EQUATIONROW_TAGS => {
      //   ($$numbering{numbered} ? RefStepCounter($ctr) : RefStepID($ctr)) }, 'global');
      let new_tags = if is_numbered_for_postset {
        ref_step_counter(&ctr, false)?
      } else {
        ref_step_id(&ctr)?
      };
      state::assign_value(
        "EQUATIONROW_TAGS",
        Stored::HashStored(new_tags),
        Some(Scope::Global),
      );
    },
    EqAction::TagsUpdate => {
      let invoked_tags = build_invocation(
        T_CS!("\\lx@make@tags"),
        vec![Some(Tokens::new(Explode!(ctr)))],
      )?;
      let stored_tags_update =
        Stored::Digested(stomach::digest(invoked_tags)?);
      with_value_mut("EQUATIONROW_TAGS", |tags_opt| {
        if let Some(Stored::HashStored(ref mut tags)) = tags_opt {
          tags.insert("tags", stored_tags_update);
        }
      });
    },
    EqAction::None => {},
  }

  // Phase 3: Reset in_equation flag
  with_value_mut("EQUATION_NUMBERING", |eq_num_opt| {
    if let Some(Stored::HashStored(ref mut numbering)) = eq_num_opt {
      numbering.insert("in_equation", Stored::Bool(false));
    }
  });

  // Phase 4: Install tags in $whatsit or current Row, as appropriate.
  #[allow(clippy::manual_unwrap_or_default)]
  let props = match state::remove_value("EQUATIONROW_TAGS") {
    Some(Stored::HashStored(hs)) => hs,
    _ => SymHashMap::default(),
  };
  if is_aligned {
    // Perl: propagate id/tags to current alignment row.
    // The Alignment struct's current_row is not easily accessible from a Digested wrapper.
    // This is a stub — aligned equation numbering may not fully work yet.
    // TODO: when Alignment is accessible, set row id/tags from props.
  } else {
    whatsit.set_properties(props);
  }
  Ok(())
}

/// Perl: latex_constructs.pool.ltxml lines 2025-2035
fn retract_equation() {
  // Phase 1: Gather data (immutable borrows)
  let (ctr, is_preset, is_numbered) =
    with_value("EQUATION_NUMBERING", |eq_num_opt| {
      let numbering = match eq_num_opt {
        Some(Stored::HashStored(n)) => n,
        _ => return (String::from("equation"), false, false),
      };
      let is_numbered =
        matches!(numbering.get("numbered"), Some(&Stored::Bool(true)));
      with_value("EQUATIONROW_TAGS", |tags_opt| {
        let tags = match tags_opt {
          Some(Stored::HashStored(t)) => t,
          _ => return (String::from("equation"), false, is_numbered),
        };
        let ctr = tags
          .get("counter")
          .map_or_else(|| numbering.get("counter"), Some)
          .map(ToString::to_string)
          .unwrap_or_else(|| String::from("equation"));
        let is_preset =
          matches!(tags.get("preset"), Some(&Stored::Bool(true)));
        (ctr, is_preset, is_numbered)
      })
    });

  // Phase 2: Mutate state (borrows released)
  if is_preset {
    // counter (or ID counter) was stepped, so decrement it.
    let counter_name =
      if is_numbered { ctr.clone() } else { s!("UN{}", ctr) };
    let _ = add_to_counter(&counter_name, Number::new(-1));
  }
  if let Ok(mut new_tags) = ref_step_id(&ctr) {
    new_tags.insert("reset", true.into());
    state::assign_value(
      "EQUATIONROW_TAGS",
      Stored::HashStored(new_tags),
      Some(Scope::Global),
    );
  }
}

// TODO: Perl: latex_constructs.pool.ltxml lines 2287-2325
// Full eqnarrayBindings() with alignment template, custom containers,
// row hooks, and rearrangeEqnarray post-processing. Deferred until
// alignment-based eqnarray infrastructure is fully ported.

LoadDefinitions!({
  DefMacro!("\\@eqnnum", "(\\theequation)", locked => true);
  DefMacro!("\\fnum@equation", "\\@eqnnum");

  // Redefined from TeX.pool, since with LaTeX we presumably have a more complete numbering system
  DefConstructor!("\\lx@begin@display@math", "<ltx:equation xml:id='#id'>\
  <ltx:Math mode='display'>\
  <ltx:XMath>#body</ltx:XMath>\
  </ltx:Math>\
  </ltx:equation>",
  alias        => "$$",
  before_digest => {
    // begin_mode handles \everydisplay injection (Stomach.pm lines 504-507)
    begin_mode("display_math")?;
  },
  properties  => { ref_step_id("equation") },
  capture_body => true);

  // Perl: latex_constructs.pool.ltxml lines 2011-2023
  // Save display math delimiters for use within equation environments
  Let!("\\lx@saved@begin@display@math", "\\lx@begin@display@math");
  Let!("\\lx@saved@end@display@math", "\\lx@end@display@math");

  // Within an equation, \[ restores saved display math and re-enters
  DefMacro!(
    "\\lx@bDM@in@equation",
    "\\lx@saved@begin@display@math\\let\\lx@end@display@math\\lx@saved@end@display@math"
  );
  // Within an equation, \] or $$ triggers "cheap intertext":
  // retract the equation number, end equation, insert text, re-begin equation
  DefMacro!(
    "\\lx@eDM@in@equation",
    "\\lx@retract@eqnno\\lx@begin@fake@intertext\\let\\lx@saved@begin@display@math\\lx@begin@display@math\\let\\lx@saved@bdm\\[\\let\\lx@begin@display@math\\lx@end@fake@intertext\\let\\[\\lx@end@fake@intertext"
  );
  DefMacro!("\\lx@begin@fake@intertext", "\\end{equation}");
  DefMacro!(
    "\\lx@end@fake@intertext",
    "\\let\\lx@begin@display@math\\lx@saved@begin@display@math\\let\\[\\lx@saved@bdm\\begin{equation}"
  );
  DefPrimitive!("\\lx@retract@eqnno", { retract_equation(); });

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

  // Perl: latex_constructs.pool.ltxml lines 2109-2125
  // Note: In ams, this DOES get a number if \tag is used!
  DefEnvironment!(
    "{equation*}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    mode => "display_math",
    before_digest => {
      prepare_equation_counter(stored_map!("preset" => true));
      before_equation()?;
    },
    after_digest_body => sub[whatsit] {
      after_equation(whatsit)?;
    },
    locked => true);

  // Perl: latex_constructs.pool.ltxml lines 2039-2057
  DefMacro!("\\nonumber", "\\lx@equation@nonumber");
  DefPrimitive!("\\lx@equation@nonumber", {
    let (in_equation, defer_retract) =
      with_value("EQUATION_NUMBERING", |v| match v {
        Some(Stored::HashStored(n)) => (
          matches!(n.get("in_equation"), Some(&Stored::Bool(true))),
          matches!(n.get("deferretract"), Some(&Stored::Bool(true))),
        ),
        _ => (false, false),
      });
    if in_equation {
      if defer_retract {
        with_value_mut("EQUATIONROW_TAGS", |tags_opt| {
          if let Some(Stored::HashStored(ref mut tags)) = tags_opt {
            tags.insert("retract", true.into());
          }
        });
      } else {
        retract_equation();
      }
    }
  });

  // Perl: latex_constructs.pool.ltxml line 2051-2057
  DefMacro!(
    "\\lx@equation@settag",
    "\\lx@equation@retract\\lx@equation@settag@"
  );
  DefPrimitive!("\\lx@equation@retract", { retract_equation(); });
  DefPrimitive!(
    "\\lx@equation@settag@ {}",
    sub[(content)] {
      // Perl uses Digested parameter type; we manually digest here
      let digested = stomach::digest(content)?;
      with_value_mut("EQUATIONROW_TAGS", |tags_opt| {
        if let Some(Stored::HashStored(ref mut tags)) = tags_opt {
          tags.insert("tags", Stored::Digested(digested));
        }
      });
      Ok(Vec::new())
    },
    mode => "restricted_horizontal"
  );

  DefMacro!("\\[", "\\lx@begin@display@math");
  DefMacro!("\\]", "\\lx@end@display@math");
  DefMacro!("\\(", "\\lx@begin@inline@math");
  DefMacro!("\\)", "\\lx@end@inline@math");

  // Keep from expanding too early, if in alignments, or such.
  DefMacro!(
    T_CS!("\\ensuremath"),
    None,
    Tokens!(T_CS!("\\protect"), T_CS!("\\@ensuremath"))
  );
  // protected => true prevents read_x_token(fully_expand=false) from expanding this
  // (needed for lx_change_case_tokens to preserve \ensuremath{} content unchanged)
  DefMacro!("\\@ensuremath{}", sub[(stuff)] {
    if lookup_bool("IN_MATH") {
      stuff.unlist()
    } else {
      let mut result = vec![T_MATH!()];
      result.extend(stuff.unlist());
      result.push(T_MATH!());
      result
    }
  }, protected => true);

  // TODO: Perl latex_constructs.pool.ltxml lines 2237-2239
  // \@equationgroup@numbering RequiredKeyVals — full impl needs alignment-based eqnarray
  // DefPrimitive!("\\@equationgroup@numbering RequiredKeyVals", sub[(kv_opt)] { ... });

  // Perl: latex_constructs.pool.ltxml lines 2262-2335
  // Full eqnarray with alignment is complex; using simplified environment for now
  // that produces equationgroup > equation > Math structure.
  // TODO: implement full eqnarrayBindings with alignment template
  DefEnvironment!("{eqnarray}",
    "<ltx:equationgroup class='ltx_eqn_eqnarray' xml:id='#id'>\
      <ltx:equation xml:id='#eq_id'>#tags\
        <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
      </ltx:equation>\
    </ltx:equationgroup>",
    mode => "display_math",
    before_digest => {
      prepare_equation_counter(stored_map!(
        "numbered" => true, "preset" => true,
        "deferretract" => true, "grouped" => true));
      before_equation()?;
    },
    properties => sub[_args] {
      let mut props = ref_step_id("@equationgroup")?;
      let eq_props = ref_step_id("equation")?;
      if let Some(eq_id) = eq_props.get("id") {
        props.insert("eq_id", eq_id.clone());
      }
      Ok(props)
    },
    after_digest_body => sub[whatsit] {
      after_equation(whatsit)?;
    },
    locked => true);

  DefEnvironment!("{eqnarray*}",
    "<ltx:equationgroup class='ltx_eqn_eqnarray' xml:id='#id'>\
      <ltx:equation xml:id='#eq_id'>#tags\
        <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
      </ltx:equation>\
    </ltx:equationgroup>",
    mode => "display_math",
    before_digest => {
      prepare_equation_counter(stored_map!(
        "numbered" => true, "preset" => true,
        "retract" => true, "grouped" => true));
      before_equation()?;
    },
    properties => sub[_args] {
      let mut props = ref_step_id("@equationgroup")?;
      let eq_props = ref_step_id("equation")?;
      if let Some(eq_id) = eq_props.get("id") {
        props.insert("eq_id", eq_id.clone());
      }
      Ok(props)
    },
    after_digest_body => sub[whatsit] {
      after_equation(whatsit)?;
    },
    locked => true);

  // Perl: latex_constructs.pool.ltxml lines 2258-2259
  Let!("\\displ@y", "\\displaystyle");
  DefMacro!("\\@lign", None, None);

  Tag!("ltx:equationgroup", auto_close => true);

  // Since the arXMLiv folks keep wanting ids on all math, let's try this!
  Tag!("ltx:Math", after_open => sub[document, node] {
    document.generate_id(node, "m")?;
  });
});

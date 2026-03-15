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
  maybe_peek_label()?;
  let ctr = with_value_mut("EQUATION_NUMBERING", |val_opt| {
    if let Some(Stored::HashStored(ref mut numbering)) = val_opt {
      numbering.insert("in_equation", true.into());
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

fn after_equation(whatsit: Option<&mut Whatsit>) -> Result<()> {
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
    // Perl: propagate id/tags to current alignment row properties.
    // In Perl, these get stored as $$row{id}, $$row{tags} and later passed to
    // the openRow hook. We store them in EQUATIONROW_PROPS for the open_row hook.
    state::assign_value(
      "EQUATIONROW_PROPS",
      Stored::HashStored(props),
      Some(Scope::Global),
    );
  } else if let Some(w) = whatsit {
    w.set_properties(props);
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

/// Perl: latex_constructs.pool.ltxml lines 2287-2325
/// eqnarrayBindings — creates alignment with equationgroup/equation/_Capture_ hooks
pub fn eqnarray_bindings() -> Result<()> {
  use latexml_core::alignment::cell::Cell;
  use latexml_core::alignment::template::TemplateConfig;

  // Perl: 3-column template: col1=right, col2=center, col3=left
  let col1 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"), T_MATH!(), T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!()])),
    empty: true,
    ..Cell::default()
  };
  let col2 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"), T_MATH!(), T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };
  let col3 = Cell {
    before: Some(Tokens::new(vec![
      T_MATH!(), T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };

  let template = Template::new(TemplateConfig {
    columns: Some(vec![col1, col2, col3]),
    ..TemplateConfig::default()
  });

  let mut xml_attrs = HashMap::default();
  xml_attrs.insert(String::from("class"), String::from("ltx_eqn_eqnarray"));
  // Perl: colsep => LookupDimension('\arraycolsep')->multiply(2)
  // TODO: compute colsep from \arraycolsep

  let mut properties = SymHashMap::default();
  properties.insert("preserve_structure", Stored::Bool(true));

  // Use custom alignment hooks for equationgroup/equation/_Capture_
  let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    open_container: Rc::new(|document, mut props| {
      // Perl: my %attr = RefStepID('@equationgroup');
      if let Ok(id_props) = ref_step_id("@equationgroup") {
        if let Some(id) = id_props.get("id") {
          props.insert(String::from("xml:id"), id.to_string());
        }
      }
      props.insert(String::from("class"), String::from("ltx_eqn_eqnarray"));
      document
        .open_element("ltx:equationgroup", Some(props), None)
        .map(Option::Some)
    }),
    close_container: Rc::new(|document| document.close_element("ltx:equationgroup")),
    open_row: Rc::new(|document, mut props| {
      // Perl: my $tags = $props{tags}; $doc->openElement('ltx:equation', %props);
      //       $doc->absorb($tags) if $tags;
      // Read equation props from state (set by after_equation in aligned mode)
      if let Some(Stored::HashStored(eq_props)) =
        state::remove_value("EQUATIONROW_PROPS")
      {
        if let Some(id) = eq_props.get("id") {
          props.insert(String::from("xml:id"), id.to_string());
        }
      }
      document
        .open_element("ltx:equation", Some(props), None)
        .and(Ok(()))
    }),
    close_row: Rc::new(|document| document.close_element("ltx:equation")),
    open_column: Rc::new(|document, props| {
      document
        .open_element("ltx:_Capture_", Some(props), None)
        .map(Option::Some)
    }),
    close_column: Rc::new(|document| document.close_element("ltx:_Capture_")),
    is_math: true,
    properties,
    xml_attributes: xml_attrs,
  });

  assign_alignment(alignment, None);
  state::let_i(&T_MATH!(), &T_CS!("\\lx@dollar@in@mathmode"), None);
  state::let_i(
    &T_CS!("\\\\"),
    &T_CS!("\\lx@alignment@newline"),
    None,
  );
  state::let_i(
    &T_CS!("\\lx@intercol"),
    &T_CS!("\\lx@math@intercol"),
    None,
  );
  state::let_i(
    &T_CS!("\\lx@alignment@row@before"),
    &T_CS!("\\eqnarray@row@before"),
    None,
  );
  state::let_i(
    &T_CS!("\\lx@alignment@row@after"),
    &T_CS!("\\eqnarray@row@after"),
    None,
  );
  // Perl: Let('\label', '\lx@eqnarray@label');
  // TODO: eqnarray label handling
  Ok(())
}

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
      after_equation(Some(whatsit))?;
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
      after_equation(Some(whatsit))?;
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

  // Perl: latex_constructs.pool.ltxml lines 2237-2239
  // \@equationgroup@numbering{numbered=1,postset=1,...}
  DefPrimitive!("\\@equationgroup@numbering{}", sub[(kv_arg)] {
    let kv_str = kv_arg.to_string();
    let mut options = SymHashMap::default();
    for part in kv_str.split(',') {
      let part = part.trim();
      if let Some((key, value)) = part.split_once('=') {
        let key = key.trim();
        let value = value.trim();
        if value == "1" {
          options.insert(key, Stored::Bool(true));
        } else if value == "0" {
          options.insert(key, Stored::Bool(false));
        } else {
          options.insert(key, Stored::from(value.to_string()));
        }
      }
    }
    prepare_equation_counter(options);
    Ok(())
  });

  // Perl: latex_constructs.pool.ltxml lines 2282-2285
  DefPrimitive!("\\eqnarray@row@before@", { before_equation()?; });
  DefPrimitive!("\\eqnarray@row@after@", {
    after_equation(None)?;
  });
  DefMacro!("\\eqnarray@row@before", "\\lx@hidden@noalign{\\eqnarray@row@before@}");
  DefMacro!("\\eqnarray@row@after", "\\lx@hidden@noalign{\\eqnarray@row@after@}");

  // Perl: latex_constructs.pool.ltxml lines 2262-2335
  // eqnarray and eqnarray* — alignment-based environments
  DefPrimitive!("\\@eqnarray@bindings", {
    eqnarray_bindings()?;
  });

  DefMacro!("\\eqnarray",
    "\\@eqnarray@bindings\\@@eqnarray\
     \\@equationgroup@numbering{numbered=1,preset=1,deferretract=1,grouped=1,aligned=1}\
     \\lx@begin@alignment",
    locked => true);
  DefMacro!("\\endeqnarray",
    "\\cr\\lx@end@alignment\\end@eqnarray",
    locked => true);
  DefMacro!("\\csname eqnarray*\\endcsname",
    "\\@eqnarray@bindings\\@@eqnarray\
     \\@equationgroup@numbering{numbered=1,preset=1,retract=1,grouped=1,aligned=1}\
     \\lx@begin@alignment",
    locked => true);
  DefMacro!("\\csname endeqnarray*\\endcsname",
    "\\lx@end@alignment\\end@eqnarray",
    locked => true);

  DefConstructor!("\\@@eqnarray SkipSpaces DigestedBody",
    "#1",
    before_digest => {
      bgroup();
    },
    after_construct => sub[document, _whatsit] {
      if let Some(mut last) = document.get_node().get_last_child() {
        rearrange_eqnarray(document, &mut last)?;
      }
    },
    mode => "restricted_horizontal",
    enter_horizontal => true);
  DefPrimitive!("\\end@eqnarray", {
    egroup()?;
  });

  // Perl: latex_constructs.pool.ltxml lines 2243-2247
  DefConditional!("\\if@in@firstcolumn", {
    if let Some(alignment_digested) = lookup_alignment() {
      if let Some(alignment_cell) = alignment_digested.alignment_cell() {
        let alignment = alignment_cell.borrow();
        !alignment.is_in_row()
          || (!alignment.is_in_column() && alignment.current_column_number() < 2)
      } else {
        false
      }
    } else {
      false
    }
  });

  // Perl: latex_constructs.pool.ltxml lines 2251-2254
  DefMacro!("\\lefteqn{}",
    "\\ifx.#1.\\else\
      \\if@in@firstcolumn\\multicolumn{3}{l}{\\@ADDCLASS{ltx_eqn_lefteqn}\\lx@begin@inline@math \\displaystyle #1\\lx@end@inline@math\\mbox{}}\
      \\else\\rlap{\\lx@begin@inline@math\\displaystyle #1\\lx@end@inline@math}\\fi\\fi");

  // Perl: latex_constructs.pool.ltxml lines 2258-2259
  Let!("\\displ@y", "\\displaystyle");
  DefMacro!("\\@lign", None, None);

  Tag!("ltx:equationgroup", auto_close => true);

  // Since the arXMLiv folks keep wanting ids on all math, let's try this!
  Tag!("ltx:Math", after_open => sub[document, node] {
    document.generate_id(node, "m")?;
  });
});

/// Perl: rearrangeEqnarray (latex_constructs.pool.ltxml L2356-2445)
/// Analyzes column patterns in eqnarray and rearranges into MathFork structures.
fn rearrange_eqnarray(
  document: &mut Document,
  equationgroup: &mut Node,
) -> Result<()> {
  use crate::engine::base_xmath::{equationgroup_join_cols, equationgroup_join_rows};

  struct EqRow {
    node: Node,
    cols: Vec<Node>,
    has_l: bool,
    has_m: bool,
    has_r: bool,
    numbered: bool,
    _labelled: bool,
  }

  // Scan the equations (rows)
  let mut rows: Vec<EqRow> = Vec::new();
  let equation_nodes: Vec<Node> = document.findnodes("ltx:equation", Some(equationgroup));
  for rownode in equation_nodes {
    let cells: Vec<Node> = document.findnodes("ltx:_Capture_", Some(&rownode));
    let has_l = cells.get(0).map_or(false, |c| !c.get_child_nodes().is_empty());
    let has_m = cells.get(1).map_or(false, |c| !c.get_child_nodes().is_empty());
    let has_r = cells.get(2).map_or(false, |c| !c.get_child_nodes().is_empty());
    let numbered = !document.findnodes("ltx:tags", Some(&rownode)).is_empty();
    let labelled = rownode.get_attribute("label").is_some();
    rows.push(EqRow {
      node: rownode,
      cols: cells,
      has_l,
      has_m,
      has_r,
      numbered,
      _labelled: labelled,
    });
  }

  let n_l = rows.iter().filter(|r| r.has_l).count();
  let n_m = rows.iter().filter(|r| r.has_m).count();
  let n_r = rows.iter().filter(|r| r.has_r).count();

  // Only a single column was used
  if (n_l > 0 && n_m == 0 && n_r == 0)
    || (n_l == 0 && n_m > 0 && n_r == 0)
    || (n_l == 0 && n_m == 0 && n_r > 0)
  {
    let keepcol = if n_l > 0 { 0 } else if n_m > 0 { 1 } else { 2 };
    // Remove empty columns (in reverse order to preserve indices)
    for c in (0..3).rev() {
      if c == keepcol {
        continue;
      }
      for row in rows.iter() {
        if let Some(col) = row.cols.get(c) {
          let mut col_clone = col.clone();
          col_clone.unlink_node();
        }
      }
    }
    // Check if any column begins with a RELOP → join rows
    let begins_with_relop = rows.iter().any(|row| {
      row.cols.get(keepcol).and_then(|c| {
        c.get_child_elements().into_iter().next().and_then(|first| {
          first.get_attribute("role").map(|r| r == "RELOP")
        })
      }).unwrap_or(false)
    });

    if begins_with_relop {
      let nodes: Vec<Node> = rows.into_iter().map(|r| r.node).collect();
      equationgroup_join_rows(document, equationgroup, nodes)?;
    } else {
      for mut row in rows {
        equationgroup_join_cols(document, 1, &mut row.node)?;
      }
    }
    return Ok(());
  }

  // All 3 columns case — analyze continuation patterns
  let mut eqs: Vec<Vec<Node>> = Vec::new();
  let mut numbered = false;

  for row in &rows {
    let class;
    if row.has_l {
      class = "new";
    } else if row.has_m {
      if eqs.is_empty() {
        class = "odd";
      } else if numbered && row.numbered {
        class = "new";
      } else {
        class = "continue";
      }
    } else if row.has_r {
      if eqs.is_empty() {
        class = "odd";
      } else if numbered && row.numbered && row._labelled {
        class = "odd";
      } else {
        class = "continue";
      }
    } else {
      // All columns empty
      class = "remove";
    }

    if class == "remove" {
      let mut node = row.node.clone();
      node.unlink_node();
    } else if class == "new" || class == "odd" {
      numbered = row.numbered;
      eqs.push(vec![row.node.clone()]);
    } else {
      // "continue"
      numbered |= row.numbered;
      if let Some(last) = eqs.last_mut() {
        last.push(row.node.clone());
      }
    }
  }

  // Now rearrange
  for eqset in eqs {
    equationgroup_join_rows(document, equationgroup, eqset)?;
  }
  Ok(())
}

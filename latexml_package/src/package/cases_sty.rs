use crate::prelude::*;

/// Perl: cases.sty.ltxml — numcases and subnumcases environments
/// Perl: numcasesBindings($lhs) — creates 3-column alignment for numcases
fn numcases_bindings(lhs: Tokens) -> Result<()> {
  use latexml_core::alignment::cell::Cell;
  let col1 = Cell {
    before: Some(Tokens!(
      T_CS!("\\hfil"),
      T_MATH!(),
      T_CS!("\\lx@hidden@bgroup"),
      T_CS!("\\displaystyle")
    )),
    after: Some(Tokens!(T_CS!("\\lx@hidden@egroup"), T_MATH!())),
    ..Cell::default()
  };
  let col2 = Cell {
    before: Some(Tokens!(
      T_MATH!(),
      T_CS!("\\lx@hidden@bgroup"),
      T_CS!("\\displaystyle")
    )),
    after: Some(Tokens!(
      T_CS!("\\lx@hidden@egroup"),
      T_MATH!(),
      T_CS!("\\hfil")
    )),
    ..Cell::default()
  };
  let col3 = Cell {
    before: Some(Tokens!(T_CS!("\\lx@hidden@bgroup"))),
    after: Some(Tokens!(T_CS!("\\lx@hidden@egroup"), T_CS!("\\hfil"))),
    ..Cell::default()
  };
  use latexml_core::alignment::template::TemplateConfig;
  let mut template = Template::new(TemplateConfig::default());
  template.add_column(col1);
  template.add_column(col2);
  template.add_column(col3);

  let mut attrs = string_map! { "class" => "ltx_eqn_numcases" };
  if let Ok(Some(colsep)) = lookup_register("\\arraycolsep", Vec::new()) {
    let colsep_dim: Dimension = colsep.into();
    let doubled = Dimension::new(colsep_dim.value_of() * 2);
    attrs.insert(String::from("colsep"), doubled.to_attribute());
  }

  let alignment = Alignment::new(AlignmentConfig {
    template:        Some(template),
    open_container:  Rc::new(move |document, mut props| {
      if let Ok(id_props) = ref_step_id("@equationgroup") {
        if let Some(id) = id_props.get("id") {
          props.insert(String::from("xml:id"), id.to_string());
        }
      }
      // Perl: %attributes has class => 'ltx_eqn_numcases' which overrides
      // the openContainer's default class. Use 'ltx_eqn_numcases' directly.
      props
        .entry(String::from("class"))
        .or_insert_with(|| String::from("ltx_eqn_numcases"));
      document
        .open_element("ltx:equationgroup", Some(props), None)
        .map(Option::Some)
    }),
    close_container: Rc::new(|document| document.close_element("ltx:equationgroup")),
    open_row:        Rc::new(|document, mut props| {
      if let Some(Stored::HashStored(eq_props)) = state::remove_value("EQUATIONROW_PROPS") {
        if let Some(id) = eq_props.get("id") {
          props.insert(String::from("xml:id"), Stored::from(id.to_string()));
        }
      }
      let tags_digested = props.remove("tags");
      let str_props: HashMap<String, String> =
        props.into_iter().map(|(k, v)| (k, v.to_string())).collect();
      document.open_element("ltx:equation", Some(str_props), None)?;
      if let Some(Stored::Digested(d)) = tags_digested {
        document.absorb(&d, None)?;
      }
      Ok(())
    }),
    close_row:       Rc::new(|document| document.close_element("ltx:equation")),
    open_column:     Rc::new(|document, props| {
      document
        .open_element("ltx:_Capture_", Some(props), None)
        .map(Option::Some)
    }),
    close_column:    Rc::new(|document| document.close_element("ltx:_Capture_")),
    is_math:         true,
    properties:      SymHashMap::default(),
    xml_attributes:  attrs,
  });
  assign_alignment(alignment, None);
  state::let_i(&T_MATH!(), &T_CS!("\\lx@dollar@in@mathmode"), None);
  let mut lhs_expansion = lhs.unlist();
  lhs_expansion.push(T_ALIGN!());
  def_macro(
    T_CS!("\\@numcases@LHS"),
    None,
    Tokens::new(lhs_expansion),
    None,
  )?;
  Let!("\\\\", "\\@numcases@newline");
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
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  DefPrimitive!("\\@numcases@bindings{}", sub[(lhs)] {
    numcases_bindings(lhs)?;
  });

  // Perl cases.sty.ltxml L41/L44/L51/L54 all carry `locked => 1`: the
  // raw cases.sty is loaded alongside our binding for layout fidelity
  // and tries to redefine \numcases / \endnumcases / \subnumcases /
  // \endsubnumcases over our alignment trampolines. The lock keeps
  // our versions in place.
  DefMacro!("\\numcases{}",
    "\\@numcases@bindings{#1}\\@@numcases\\@equationgroup@numbering{numbered=1,preset=1,deferretract=1,grouped=1,aligned=1}\\lx@begin@alignment\\@numcases@LHS",
    locked => true);
  DefMacro!("\\endnumcases", "\\lx@end@alignment\\end@numcases",
    locked => true);

  DefMacro!("\\subnumcases{}",
    "\\@numcases@bindings{#1}\\lx@numcases@subnumbering@begin\\@@numcases\\@equationgroup@numbering{numbered=1,preset=1,deferretract=1,grouped=1,aligned=1}\\lx@begin@alignment\\@numcases@LHS",
    locked => true);
  DefMacro!("\\endsubnumcases", "\\lx@end@alignment\\end@numcases\\lx@numcases@subnumbering@end",
    locked => true);

  // Sub-numbering: step parent equation, save counter, reset, redefine \theequation
  // Perl: cases.sty.ltxml L57-64
  DefPrimitive!("\\lx@numcases@subnumbering@begin", sub[_args] {
    use latexml_core::binding::counter::dialect::reset_counter;
    use latexml_core::mouth;
    // Step the equation counter and get properties (id, refnum)
    let eqn_props = ref_step_counter("equation", false)?;
    // Expand \theequation to get the parent equation number text (e.g. "3")
    let eqnum_toks = gullet::do_expand(T_CS!("\\theequation"))?;
    let eqnum_str = eqnum_toks.to_string();
    // Save current equation counter value
    let saved = state::lookup_register("\\c@equation", Vec::new())?.map_or(0, |rv| {
      match rv {
        RegisterValue::Number(n) => n.0,
        _ => 0,
      }
    });
    state::assign_value("SAVED_EQUATION_NUMBER", Stored::Number(Number::new(saved)), Some(Scope::Global));
    // Reset equation counter to 0 for sub-lettering
    reset_counter(&T_OTHER!("equation"))?;
    // Redefine \theequation to parent_number + \alph{equation} (e.g. "3a", "3b")
    let new_theequation = format!("{}\\alph{{equation}}", eqnum_str);
    def_macro(T_CS!("\\theequation"), None, mouth::tokenize_internal(&new_theequation), None)?;
    // Redefine \theequation@ID for xml:id generation (e.g. "S0.E3.\@equation@ID")
    let id_str = eqn_props.iter().find_map(|(k, v)| {
      if arena::with(*k, |ks| ks == "id") { Some(v.to_string()) } else { None }
    }).unwrap_or_default();
    if !id_str.is_empty() {
      let new_id_macro = format!("{}.\\@equation@ID", id_str);
      def_macro(T_CS!("\\theequation@ID"), None, mouth::tokenize_internal(&new_id_macro), None)?;
    }
  });
  DefPrimitive!("\\lx@numcases@subnumbering@end", sub[_args] {
    // Restore saved equation counter
    if let Some(Stored::Number(n)) = state::lookup_value("SAVED_EQUATION_NUMBER") {
      let _ = state::assign_register("\\c@equation", RegisterValue::Number(n), Some(Scope::Global), Vec::new());
    }
  });

  DefMacro!("\\@numcases@newline[]",
    "\\ifx.#1.\\lx@alignment@newline\\else\\lx@alignment@newline[#1]\\fi\\@numcases@LHS");
  DefMacro!("\\@numcases@cr", "\\lx@alignment@cr\\@numcases@LHS");

  DefConstructor!("\\@@numcases SkipSpaces DigestedBody", "#1",
    before_digest => sub { bgroup(); },
    after_construct => sub[document, _whatsit] {
      let node = document.get_node();
      if let Some(equationgroup) = node.get_last_child() {
        rearrange_numcases(document, &equationgroup)?;
      }
    });

  DefPrimitive!("\\end@numcases", sub[_args] { egroup()?; });
});

fn rearrange_numcases(document: &mut Document, equationgroup: &Node) -> Result<()> {
  use crate::engine::base_xmath::equationgroup_join_cols;
  let equations: Vec<Node> = document.findnodes("ltx:equation", Some(equationgroup));
  for equation in equations {
    let cells: Vec<Node> = document.findnodes("ltx:_Capture_", Some(&equation));
    if cells.is_empty() {
      continue;
    }
    let cell1cont: Vec<Node> = cells[0].get_child_elements();
    if cells.len() == 1
      && cell1cont.len() == 1
      && cell1cont[0]
        .get_attribute("class")
        .unwrap_or_default()
        .contains("ltx_intertext")
    {
      let mut cell1cont0 = cell1cont[0].clone();
      cell1cont0.unlink_node();
      let mut eq = equation.clone();
      let _ = eq.add_prev_sibling(&mut cell1cont0);
      eq.unlink_node();
    } else if cells.len() == 1 && cell1cont.is_empty() {
      let mut eq = equation.clone();
      eq.unlink_node();
    } else {
      equationgroup_join_cols(document, 3, &mut equation.clone())?;
    }
  }
  Ok(())
}

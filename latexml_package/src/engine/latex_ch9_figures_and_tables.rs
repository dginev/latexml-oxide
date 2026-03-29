use crate::prelude::*;
use latexml_core::document::Document;

/// Perl: beforeFloat (latex_constructs.pool.ltxml L3430-3438)
/// Sets \@captype, adjusts \hsize for single/double column floats.
/// `preincrement`: if Some("figure"), pre-increments the parent float counter
///   on first subfloat entry (before main caption), storing result for later use.
pub fn before_float(float_type: &str, preincrement: Option<&str>) {
  before_float_ex(float_type, preincrement, false);
}

/// Extended version with `double` flag for `*` variants (span both columns).
pub fn before_float_ex(float_type: &str, preincrement: Option<&str>, double: bool) {
  def_macro(
    T_CS!("\\@captype"), None,
    Tokens::new(ExplodeText!(float_type)),
    None,
  ).ok();
  // Perl #2775: rebind \\ to \lx@newline in floats to prevent
  // alignment-token early-return when floats are inside tabulars.
  Let!("\\\\", "\\lx@newline");
  // Perl: AssignRegister('\hsize' => LookupDimension($options{double} ? '\textwidth' : '\columnwidth'));
  let dim_name = if double { "\\textwidth" } else { "\\columnwidth" };
  let dim_val = state::lookup_dimension(dim_name).unwrap_or_default();
  state::assign_register("\\hsize", dim_val.into(), None, Vec::new()).ok();
  // Perl: if (my $main = $options{preincrement}) {
  //   if (($type ne (LookupValue('LAST_FLOATTYPE') || ''))
  //     && !IfCondition('\iflx@donecaption')) {
  //     AssignValue('PREINCREMENTED_' . $main => { RefStepCounter($main) }, 'global'); } }
  if let Some(main_counter) = preincrement {
    let last_type = state::lookup_value("LAST_FLOATTYPE")
      .map(|s| s.to_string())
      .unwrap_or_default();
    let done_caption = if_condition(&T_CS!("\\iflx@donecaption"))
      .unwrap_or(None)
      .unwrap_or(false);
    if float_type != last_type && !done_caption {
      if let Ok(props) = ref_step_counter(main_counter, false) {
        let prekey = s!("PREINCREMENTED_{main_counter}");
        state::assign_value(&prekey, props, Some(Scope::Global));
      }
    }
  }
}

/// Perl: afterFloat (latex_constructs.pool.ltxml L3440-3448)
/// Rescues caption counters into the whatsit properties.
pub fn after_float(whatsit: &mut Whatsit) {
  let captype = stomach::digest(T_CS!("\\@captype"))
    .map(|d| d.to_string())
    .unwrap_or_default();
  // Perl: AssignValue('PREINCREMENTED_' . $type => undef, 'global');
  let prekey = s!("PREINCREMENTED_{captype}");
  state::remove_value(&prekey);
  rescue_caption_counters(&captype, whatsit);
  state::assign_value("LAST_FLOATTYPE", Stored::String(arena::pin(&captype)), Some(Scope::Global));
}

/// Simplified version of Perl's arrange_panels_and_breaks().
/// When a figure/table/float has 2+ child figure/table/float elements (panels),
/// add the ltx_figure_panel class to each panel.
fn arrange_panels(document: &mut Document, node: &mut libxml::tree::Node) -> Result<()> {
  // Perl: arrange_panels_and_breaks (latex_constructs L3286-3406)
  // Simplified: we mark panel children with ltx_figure_panel class
  // but skip the full break-insertion / width-based row-splitting logic.
  //
  // panel_break_names (Perl L3302-3307): elements that are NOT panels.
  // Includes: ltx:break, Caption class (caption, toccaption),
  // SectionalFrontMatter class (title, toctitle, subtitle, creator, contact, date,
  // tags, classification, acknowledgements), Meta class (resource, navigation, etc.)
  let is_panel_break = |qname: arena::SymStr| -> bool {
    arena::with(qname, |name| {
      matches!(
        name,
        "ltx:break"
          | "ltx:caption"
          | "ltx:toccaption"
          | "ltx:title"
          | "ltx:toctitle"
          | "ltx:subtitle"
          | "ltx:creator"
          | "ltx:contact"
          | "ltx:date"
          | "ltx:tags"
          | "ltx:classification"
          | "ltx:acknowledgements"
          | "ltx:resource"
          | "ltx:navigation"
      )
    })
  };
  let note_qname = arena::pin_static("ltx:note");
  let caption_qname = arena::pin_static("ltx:caption");
  let mut panels: Vec<libxml::tree::Node> = Vec::new();
  let mut notes: Vec<libxml::tree::Node> = Vec::new();
  let mut caption: Option<libxml::tree::Node> = None;
  for child in node.get_child_elements() {
    let qname = latexml_core::document::get_node_qname(&child);
    if qname == note_qname {
      notes.push(child);
    } else if is_panel_break(qname) {
      if qname == caption_qname {
        caption = Some(child);
      }
    } else {
      // Perl L3342-3390: non-break children are potential panels
      // (Perl also checks child_width > 0 at L3390, but we skip width checks)
      panels.push(child);
    }
  }
  // Perl BuildPanelsAndID L3317-3324: move top-level ltx:note to nearest caption
  if let Some(mut cap) = caption {
    for mut note in notes {
      note.unlink_node();
      cap.add_child(&mut note).ok();
    }
  }
  // Perl L3403-3405: only add class if >1 panel (complex figure)
  if panels.len() >= 2 {
    // Perl: standalone panels get breaks between them.
    // Perl has width-based row-splitting logic, but without box width tracking,
    // we use a simpler heuristic: insert break after each "standalone" panel
    // (p, listing, equation, equationgroup, itemize, enumerate, quote, theorem,
    // proof, description, verbatim, math) when there are multiple panels.
    let is_standalone = |p: &libxml::tree::Node| -> bool {
      let qname = latexml_core::document::get_node_qname(p);
      arena::with(qname, |name| {
        matches!(name,
          "ltx:p" | "ltx:listing" | "ltx:math" | "ltx:itemize" | "ltx:enumerate"
          | "ltx:quote" | "ltx:theorem" | "ltx:proof" | "ltx:description"
          | "ltx:equation" | "ltx:equationgroup" | "ltx:verbatim")
      })
    };
    for i in 0..panels.len() {
      document.add_class(&mut panels[i], "ltx_figure_panel")?;
    }
    // Insert breaks between panels.
    // Perl inserts break before a standalone panel (if there are prior panels in the row),
    // and after standalone panels at the start. We simplify: insert break between consecutive
    // panels where either the current or next panel is standalone.
    for i in 0..panels.len().saturating_sub(1) {
      if is_standalone(&panels[i]) || is_standalone(&panels[i + 1]) {
        let ns = panels[i].get_namespace();
        let mut break_node =
          libxml::tree::Node::new("break", ns, document.get_document()).unwrap();
        let _ = break_node.set_attribute("class", "ltx_break");
        panels[i].add_next_sibling(&mut break_node)?;
      }
    }
  }
  Ok(())
}

/// Perl: collapseFloat (latex_constructs.pool.ltxml L3493-3520)
/// If a figure/table/float contains exactly one inner float child,
/// and they don't BOTH have captions, collapse the inner into the outer.
fn collapse_float(document: &mut Document, float: &mut libxml::tree::Node) -> Result<()> {
  let caption_qname = arena::pin_static("ltx:caption");
  let figure_qname = arena::pin_static("ltx:figure");
  let table_qname = arena::pin_static("ltx:table");
  let float_qname = arena::pin_static("ltx:float");
  // Find inner float/figure/table children
  let mut inners: Vec<libxml::tree::Node> = Vec::new();
  for child in float.get_child_elements() {
    let qname = latexml_core::document::get_node_qname(&child);
    if qname == figure_qname || qname == table_qname || qname == float_qname {
      inners.push(child);
    }
  }
  if inners.len() != 1 {
    return Ok(());
  }
  let mut inner = inners.into_iter().next().unwrap();
  // Check captions: collapse only if they don't BOTH have captions
  let outer_has_caption = float.get_child_elements().iter()
    .any(|c| latexml_core::document::get_node_qname(c) == caption_qname);
  let inner_has_caption = inner.get_child_elements().iter()
    .any(|c| latexml_core::document::get_node_qname(c) == caption_qname);
  if outer_has_caption && inner_has_caption {
    return Ok(());
  }
  // Copy inner's attributes to outer (except xml:id)
  let attrs = inner.get_attributes();
  for (name, value) in &attrs {
    // get_attributes() may return the key as "id" (local name) or "xml:id" (prefixed)
    if name != "xml:id" && name != "id" {
      document.set_attribute(float, name, value)?;
    }
  }
  // If inner has caption, promote inner's xml:id to outer
  if inner_has_caption {
    let inner_id = inner.get_attribute("xml:id")
      .or_else(|| inner.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace"));
    if let Some(id) = inner_id {
      // Unrecord the outer's old ID and remove the attribute before setting the new one
      if let Some(old_id) = float.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace") {
        document.unrecord_id(&old_id);
      }
      float.remove_attribute("xml:id").ok();
      document.unrecord_id(&id);
      document.set_attribute(float, "xml:id", &id)?;
    }
  }
  // Replace inner element with its children (unwrap inner)
  let children: Vec<libxml::tree::Node> = inner.get_child_nodes();
  for mut child in children {
    child.unlink_node();
    float.add_child(&mut child).ok();
  }
  inner.unlink_node();
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // C.9.1 Figures and Tables
  //======================================================================

  // Note that, the number is associated with the caption.
  // (to allow multiple figures per figure environment?).
  // Whatever reason, that causes complications: We can only increment
  // counters with the caption, but then have to arrange for the counters,
  // refnums, ids, get passed on to the figure, table when needed.
  // AND, as soon as possible, since other items may base their id's on the id of the table!

  DefMacro!("\\figurename", "Figure");
  DefMacro!("\\figuresname", "Figures"); // Never used?
  DefMacro!("\\tablename", "Table");
  DefMacro!("\\tablesname", "Tables");

  // Let the fonts for float be the default for all floats, figures, tables, etc.
  DefMacro!("\\fnum@font@float", "\\@empty");
  DefMacro!("\\format@title@font@float", "\\@empty");

  DefMacro!("\\fnum@font@figure", "\\fnum@font@float");
  DefMacro!("\\fnum@font@table", "\\fnum@font@float");
  DefMacro!("\\format@title@font@figure", "\\format@title@font@float");
  DefMacro!("\\format@title@font@table", "\\format@title@font@float");

  // Could perhaps parameterize further with a separator?
  DefMacro!(
    "\\format@title@figure{}",
    "\\lx@tag[][: ]{\\lx@fnum@@{figure}}#1"
  );
  DefMacro!(
    "\\format@title@table{}",
    "\\lx@tag[][: ]{\\lx@fnum@@{table}}#1"
  );

  DefMacro!("\\ext@figure", "lof");
  DefMacro!("\\ext@table", "lot");

  DefConditional!("\\iflx@donecaption");
  DefMacro!(
    "\\caption",
    r"\lx@donecaptiontrue\@ifundefined{@captype}{\@@generic@caption}{\expandafter\@caption\expandafter{\@captype}}"
  );
  // First, check for trailing \label, move it into the caption as a standard position
  // NOTE: If one day we want to unlock \@caption, make sure to test against arXiv:cond-mat/0001395
  // for a passing build.
  DefMacro!(
    "\\@caption{}[]{}",
    r"\@ifnext\label{\@caption@postlabel{#1}{#2}{#3}}{\@caption@{#1}{#2}{#3}}",
    locked=>true
  );
  // Check for trailing \label, move it into the caption
  DefMacro!(
    r"\@caption@postlabel{}{}{} SkipMatch:\label Semiverbatim",
    r"\@caption@{#1}{#2}{#3\label{#4}}"
  );
  DefMacro!(
    r"\@caption@{}{}{}",
    r"\@hack@caption@{#1}{#2}{}#3\label\endcaption"
  );
  DefMacro!(
    r"\@hack@caption@{}{}{} Until:\label Until:\endcaption",
    r"\ifx.#5.\@caption@@@{#1}{#2}{#3#4}\else\@@@hack@caption@{#1}{#2}{#3#4}#5\endcaption\fi"
  );
  DefMacro!(
    r"\@@@hack@caption@{}{}{} Semiverbatim Until:\label Until:\endcaption",
    r"\lx@note@caption@label{#4}\@hack@caption@{#1}{#2}{#3\label{#4}#5}\label#6\endcaption"
  );

  DefPrimitive!("\\lx@note@caption@label{}", sub[(label)] {
    let label = label.to_string();
    maybe_note_label(&label); });

  DefMacro!(
    "\\@caption@@@{}{}{}",
    r"\@@add@caption@counters\@@toccaption{\lx@format@toctitle@@{#1}{\ifx.#2.#3\else#2\fi}}\@@caption{\lx@format@title@@{#1}{#3}}"
  );

  // Note that the counters only get incremented by \caption, NOT by \table, \figure, etc.
  // Perl: latex_constructs.pool.ltxml L3250-3258
  // Checks PREINCREMENTED_ first (set by beforeFloat with preincrement option).
  DefPrimitive!("\\@@add@caption@counters", {
    let captype = stomach::digest(T_CS!("\\@captype"))?.to_string();
    let prekey = s!("PREINCREMENTED_{captype}");
    let props = if let Some(Stored::HashStored(pre)) = state::remove_value(&prekey) {
      pre
    } else {
      ref_step_counter(&captype, false)?
    };
    let inlist  = stomach::digest(T_CS!(s!("\\ext@{}", captype)))?.to_string();
    state::assign_value(&s!("{}_tags", captype), props.get("tags"), Some(Scope::Global));
    state::assign_value(&s!("{}_id", captype), props.get("id"),   Some(Scope::Global));
    state::assign_value(&s!("{}_inlist", captype), inlist,      Some(Scope::Global));
  });

  DefConstructor!("\\@@generic@caption[]{}", "<ltx:text class='ltx_caption'>#2</ltx:text>",
  before_digest => {
    Error!("unexpected", "\\caption", "Use of \\caption outside any known float"); });

  // Note that even without \caption, we'd probably like to have xml:id.
  // Perl: BuildPanelsAndID + collapseFloat (afterClose hooks)
  Tag!("ltx:figure", after_close => sub[document, node] {
    document.generate_id(node, "fig")?;
    arrange_panels(document, node)?;
    collapse_float(document, node)?;
  });
  Tag!("ltx:table",  after_close => sub[document, node] {
    document.generate_id(node, "tab")?;
    arrange_panels(document, node)?;
    collapse_float(document, node)?;
  });
  Tag!("ltx:float",  after_close => sub[document, node] {
    document.generate_id(node, "tab")?;
    arrange_panels(document, node)?;
    collapse_float(document, node)?;
  });

  // # These may need to float up to where they're allowed,
  // # or they may need to close <p> or similar.
  // Perl: latex_constructs.pool.ltxml L3423-3427
  // ^^ prefix means "float up" in LaTeXML's document model
  DefConstructor!("\\@@caption{}", "^^<ltx:caption>#1</ltx:caption>",
    mode => "text");
  DefConstructor!(
    "\\@@toccaption{}",
    "^^<ltx:toccaption>#1</ltx:toccaption>", //sizer => 0
    mode => "text");

  // Perl: latex_constructs.pool.ltxml L3450-3458
  // Uses beforeFloat('figure') / afterFloat — sets LAST_FLOATTYPE, rescues counters.
  DefEnvironment!("{figure}[]",
    "<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
    #tags\
    #body\
    </ltx:figure>",
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { before_float("figure", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );
  // Perl: latex_constructs.pool.ltxml line 3460
  DefEnvironment!("{figure*}[]",
    "<ltx:figure xml:id='#id' inlist='#inlist' ?#1(placement='#1')>\
    #tags\
    #body\
    </ltx:figure>",
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { before_float_ex("figure", None, true); }, // double=true for *
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );
  // Perl: latex_constructs.pool.ltxml L3469-3477
  DefEnvironment!("{table}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>#tags#body</ltx:table>",
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { before_float("table", None); },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");
  // Perl: latex_constructs.pool.ltxml line 3478
  DefEnvironment!("{table*}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>#tags#body</ltx:table>",
    properties   => { stored_map!("layout" => "vertical") },
    before_digest => { before_float_ex("table", None, true); }, // double=true for *
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");

  DefPrimitive!("\\flushbottom",      None);
  DefPrimitive!("\\suppressfloats[]", None);

  NewCounter!("topnumber");
  DefMacro!("\\topfraction", "0.25");
  NewCounter!("bottomnumber");
  DefMacro!("\\bottomfraction", "0.25");
  NewCounter!("totalnumber");
  DefMacro!("\\textfraction", "0.25");
  DefMacro!("\\floatpagefraction", "0.25");
  NewCounter!("dbltopnumber");
  DefMacro!("\\dbltopfraction",       "0.7");
  DefMacro!("\\dblfloatpagefraction", "0.25");
  DefRegister!("\\floatsep"         => Glue!("12.0pt plus 2.0pt minus 2.0pt"));
  DefRegister!("\\textfloatsep"     => Glue!("20.0pt plus 2.0pt minus 4.0pt"));
  DefRegister!("\\intextsep"        => Glue!("12.0pt plus 2.0pt minus 2.0pt"));
  DefRegister!("\\dblfloatsep"      => Glue!("12.0pt plus 2.0pt minus 2.0pt"));
  DefRegister!("\\dbltextfloatsep"  => Glue!("20.0pt plus 2.0pt minus 4.0pt"));
  DefRegister!("\\@maxsep"          => Dimension::new(0));
  DefRegister!("\\@dblmaxsep"       => Dimension::new(0));
  DefRegister!("\\@fptop"           => Glue::new(0));
  DefRegister!("\\@fpsep"           => Glue::new(0));
  DefRegister!("\\@fpbot"           => Glue::new(0));
  DefRegister!("\\@dblfptop"        => Glue::new(0));
  DefRegister!("\\@dblfpsep"        => Glue::new(0));
  DefRegister!("\\@dblfpbot"        => Glue::new(0));
  DefRegister!("\\abovecaptionskip" => Glue::new(0));
  DefRegister!("\\belowcaptionskip" => Glue::new(0));
  Let!("\\topfigrule", "\\relax");
  Let!("\\botfigrule", "\\relax");
  Let!("\\dblfigrule", "\\relax");

  DefMacro!("\\figurename",  "Figure");
  DefMacro!("\\figuresname", "Figures");    // Never used?
  DefMacro!("\\tablename",   "Table");
  DefMacro!("\\tablesname",  "Tables");

  Let!("\\outer@nobreak", "\\@empty");
  DefMacro!("\\@dbflt{}",           "#1");
  DefMacro!("\\@xdblfloat{}[]",     "\\@xfloat{#1}[#2]");
  DefMacro!("\\@floatplacement",    "");
  DefMacro!("\\@dblfloatplacement", "");

});

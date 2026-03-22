use crate::engine::tex_tables::alignment_bindings;
use crate::prelude::*;

static NOTE_TEXT_END: Lazy<Regex> = Lazy::new(|| Regex::new("^(\\w+?)text$").unwrap());
static NOTE_MARK_END: Lazy<Regex> = Lazy::new(|| Regex::new("^(\\w+?)mark$").unwrap());

pub fn start_appendices(kind: &str) { begin_appendices(kind) }

// Class files should define \@appendix to call this as startAppendices('section') or chapter...
// counter is also the element name!

pub fn begin_appendices(counter: &str) {
  // Save current definitions for restoration
  let the_ctr = s!("\\the{counter}");
  let the_ctr_id = s!("\\the{counter}@ID");
  let cs_ctr = T_CS!(s!("\\{counter}"));
  state::let_i(
    &T_CS!("\\lx@save@theappendex"),
    &T_CS!(the_ctr.clone()),
    Some(Scope::Global),
  );
  state::let_i(
    &T_CS!("\\lx@save@theappendex@ID"),
    &T_CS!(the_ctr_id.clone()),
    Some(Scope::Global),
  );
  state::let_i(
    &T_CS!("\\lx@save@appendix"),
    &cs_ctr,
    Some(Scope::Global),
  );
  state::let_i(
    &T_CS!("\\lx@save@@appendix"),
    &T_CS!("\\@appendix"),
    Some(Scope::Global),
  );
  state::assign_mapping(
    "BACKMATTER_ELEMENT",
    "ltx:appendix",
    Some(s!("ltx:{counter}")),
  );
  let has_chapter = lookup_definition(&T_CS!("\\c@chapter")).ok().flatten().is_some();
  if has_chapter && counter != "chapter" {
    // Appendices are below the chapter level
    let _ = new_counter(counter, "chapter", Some(NewDefault!(NewCounterOptions, idprefix => "A")));
    let expansion: String = s!("\\thechapter.\\Alph{{{counter}}}");
    let _ = def_macro(
      T_CS!(the_ctr),
      None,
      Some(ExpansionBody::from(expansion)),
      Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))),
    );
  } else {
    let _ = new_counter(counter, "document", Some(NewDefault!(NewCounterOptions, idprefix => "A")));
    let expansion: String = s!("\\Alph{{{counter}}}");
    let _ = def_macro(
      T_CS!(the_ctr),
      None,
      Some(ExpansionBody::from(expansion)),
      Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))),
    );
  }
  // Reset the counter to 0
  let _ = state::assign_register(
    &s!("\\c@{counter}"),
    RegisterValue::Number(Number::new(0)),
    None,
    Vec::new(),
  );
  state::assign_mapping("counter_for_type", "appendix", Some(counter.to_string()));
  state::let_i(&cs_ctr, &T_CS!("\\@@appendix"), Some(Scope::Global));
  state::let_i(&T_CS!("\\@appendix"), &T_CS!("\\relax"), Some(Scope::Global));
}

pub fn end_appendices() {
  if let Some(counter_stored) = state::lookup_mapping("BACKMATTER_ELEMENT", "ltx:appendix") {
    let counter_full = counter_stored.to_string();
    let counter = counter_full.strip_prefix("ltx:").unwrap_or(&counter_full);
    let the_ctr = s!("\\the{counter}");
    let the_ctr_id = s!("\\the{counter}@ID");
    state::let_i(
      &T_CS!(the_ctr),
      &T_CS!("\\lx@save@theappendex"),
      Some(Scope::Global),
    );
    state::let_i(
      &T_CS!(the_ctr_id),
      &T_CS!("\\lx@save@theappendex@ID"),
      Some(Scope::Global),
    );
    state::let_i(
      &T_CS!(s!("\\{counter}")),
      &T_CS!("\\lx@save@appendix"),
      Some(Scope::Global),
    );
    state::let_i(
      &T_CS!("\\@appendix"),
      &T_CS!("\\lx@save@@appendix"),
      Some(Scope::Global),
    );
  }
}

pub fn make_note_tags(
  counter: &str,
  mark_opt: Option<&Digested>,
  tag_opt: Option<Cow<Digested>>,
) -> Result<SymHashMap<Stored>> {
  if let Some(tag) = tag_opt {
    let mut props = ref_step_id(counter)?;
    let mark = match mark_opt {
      None => tag.clone(),
      Some(mark) => Cow::Borrowed(mark),
    };
    props.insert("mark", mark.into());
    props.insert(
      "tags",
      stomach::digest(Tokens!(
        T_BEGIN!(),
        T_CS!("\\def"),
        T_CS!(s!("\\the{counter}")),
        T_BEGIN!(),
        tag.revert()?,
        T_END!(),
        T_CS!("\\def"),
        T_CS!(s!("\\typerefnum@{counter}")),
        T_BEGIN!(),
        T_CS!(s!("\\{counter}typerefname")),
        T_SPACE!(),
        tag.revert()?,
        T_END!(),
        T_CS!("\\lx@make@tags"),
        T_BEGIN!(),
        T_OTHER!(counter),
        T_END!(),
        T_END!()
      ))?
      .into(),
    );
    Ok(props)
  } else {
    let mut props = ref_step_counter(counter, false)?;
    let mark = Stored::Digested(match mark_opt {
      None => digest_text(Tokens!(T_CS!(s!("\\the{counter}"))))?,
      Some(mark) => mark.clone(),
    });
    props.insert("mark", mark);
    Ok(props)
  }
}

// Find any pairs of footnotemark & footnotetext;
// Move the contents of the text to the mark, removing the text node.
pub fn relocate_footnote(document: &mut Document, node: &mut Node) -> Result<()> {
  if let Some(caps) = NOTE_TEXT_END.captures(&node.get_attribute("role").unwrap_or_default()) {
    let notetype = caps.get(1).map_or("", |m| m.as_str()); // Eg "footnote", "endnote",...
    if let Some(mark) = node.get_attribute("mark") {
      for mut marknote in document.findnodes(
        &format!(".//ltx:note[@role='{notetype}mark'][@mark='{mark}']"),
        None,
      ) {
        relocate_footnote_aux(document, notetype, &mut marknote, node)?;
      }
    }
  } else if let Some(caps) = NOTE_MARK_END.captures(&node.get_attribute("role").unwrap_or_default())
  {
    let notetype = caps.get(1).map_or("", |m| m.as_str()); // Eg "footnote", "endnote",...
    if let Some(mark) = node.get_attribute("mark") {
      for mut textnote in document.findnodes(
        &format!(".//ltx:note[@role='{notetype}text'][@mark='{mark}']"),
        None,
      ) {
        relocate_footnote_aux(document, notetype, node, &mut textnote)?;
      }
    }
  }
  Ok(())
}

// Move the contents of the $textnote to the $marknote, remove $textnote.
fn relocate_footnote_aux(
  document: &mut Document,
  notetype: &str,
  marknote: &mut Node,
  textnote: &mut Node,
) -> Result<()> {
  // textnote.get_parent().unwrap().remove_child(textnote);
  textnote.unlink();
  document.append_clone(marknote, textnote.get_child_nodes())?;
  document.set_attribute(marknote, "role", notetype)?;
  if let Some(labels) = textnote.get_attribute("labels") {
    document.generate_id(marknote, "")?;
    document.set_attribute(marknote, "labels", &labels)?;
  }
  Ok(())
}

pub fn only_preamble(cs: &str) -> Result<()> {
  if !lookup_bool("inPreamble") {
    Error!(
      "unexpected",
      cs,
      s!("The current command '{cs}' can only appear in the preamble")
    );
  }
  Ok(())
}

pub fn tabular_bindings(
  mut template: Template,
  mut properties: SymHashMap<Stored>,
  mut xml_attributes: HashMap<String, String>,
) -> Result<()> {
  // Ensure has_intercol_after flag is set on template columns.
  // The flag may be lost during parameter passing (Stored::Template serialization).
  // Re-derive it from the after tokens: presence of \lx@intercol indicates
  // regular intercolumn spacing (non-@{} column).
  // Re-derive has_intercol_after from template tokens.
  // The flag may be lost during parameter passing.
  for col in template.get_columns_mut() {
    if let Some(ref after) = col.after {
      if after.unlist_ref().iter().any(|t| t.to_string().contains("intercol")) {
        col.has_intercol_after = true;
      }
    }
  }
  for col in template.get_repeated_mut() {
    if let Some(ref after) = col.after {
      if after.unlist_ref().iter().any(|t| t.to_string().contains("intercol")) {
        col.has_intercol_after = true;
      }
    }
  }
  if !properties.contains_key("guess_headers") {
    if let Some(v) = lookup_value("GUESS_TABULAR_HEADERS") {
      properties.insert("guess_headers", v);
    }
  }
  if !xml_attributes.contains_key("colsep") {
    let sep_opt = lookup_dimension("\\tabcolsep");
    if let Some(sep) = sep_opt {
      if sep.value_of()
        != lookup_dimension("\\lx@default@tabcolsep")
          .unwrap()
          .value_of()
      {
        xml_attributes.insert(String::from("colsep"), sep.to_attribute());
      }
    }
  }
  if !xml_attributes.contains_key("rowsep") {
    let astr = gullet::do_expand(T_CS!("\\arraystretch"))?.to_string();
    if astr != "1" {
      if let Ok(astr_f) = astr.parse::<f64>() {
        if astr_f != 1.0 {
          let rowsep = Dimension::from_str(&s!("{}em", astr_f - 1.0))?;
          xml_attributes.insert(String::from("rowsep"), rowsep.to_attribute());
        }
      }
    }
  }

  // Perl latex_constructs L3685-3687: set isLaTeX + strut for LaTeX tabulars
  if !properties.contains_key("strut") {
    properties.insert("isLaTeX", Stored::Bool(true));
    if let Ok(Some(bs)) = lookup_register("\\baselineskip", Vec::new()) {
      properties.insert("strut", bs.into());
    }
  }
  alignment_bindings(template, String::from("text"), properties, xml_attributes);
  state::let_i(&T_CS!("\\\\"), &T_CS!("\\@tabularcr"), None);
  // Perl latex_constructs L3689: Let('\lx@intercol', '\lx@text@intercol')
  state::let_i(&T_CS!("\\lx@intercol"), &T_CS!("\\lx@text@intercol"), None);
  state::let_i(&T_CS!("\\tabularnewline"), &T_CS!("\\\\"), None);
  // NOTE: Fit this back in!!!!!!!
  // Do like AddToMacro, but NOT global!
  for name in [
    "@row@before",
    "@row@after",
    "@column@before",
    "@column@after",
  ] {
    let cs = T_CS!(s!("\\lx@alignment{name}"));
    let cs_def = lookup_definition(&cs)?.unwrap();
    let mut expansion = cs_def.get_expansion().cloned().unwrap_or_default();
    expansion.push(T_CS!(s!("\\@tabular{name}")));
    def_macro(cs, None, expansion, None)?;
  }
  Ok(())
}

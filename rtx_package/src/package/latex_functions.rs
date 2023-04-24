use rustc_hash::FxHashMap as HashMap;
use libxml::tree::Node;
use once_cell::sync::Lazy;
use regex::Regex;

use rtx_core::alignment::template::Template;
use crate::package::*;
use crate::package::tex_alignment::alignment_bindings;

static NOTE_TEXT_END: Lazy<Regex> = Lazy::new(|| Regex::new("^(\\w+?)text$").unwrap());
static NOTE_MARK_END: Lazy<Regex> = Lazy::new(|| Regex::new("^(\\w+?)mark$").unwrap());

pub fn start_appendices(kind: &str, state: &mut State) { begin_appendices(kind, state) }

// Class files should define \@appendix to call this as startAppendices('section') or chapter...
// counter is also the element name!

pub fn begin_appendices(_counter: &str, _state: &mut State) {
  unimplemented!();
  // Let('\lx@save@theappendex',    '\the' . $counter,         'global');
  // Let('\lx@save@theappendex@ID', '\the' . $counter . '@ID', 'global');
  // Let('\lx@save@appendix',       T_CS!('\\' . $counter),     'global');
  // Let('\lx@save@@appendix',      T_CS!('\@appendix'),        'global');
  // AssignMapping('BACKMATTER_ELEMENT', 'ltx:appendix' => 'ltx:' . $counter);
  // if (LookupDefinition(T_CS!('\c@chapter'))    # Has \chapter defined
  //   && ($counter ne 'chapter')) {             # And appendices are below the chapter level.
  //   NewCounter($counter, 'chapter', idprefix => 'A');
  //   DefMacroI('\the' . $counter, undef, '\thechapter.\Alph{' . $counter . '}', scope =>
  // 'global'); } else {
  //   NewCounter($counter, 'document', idprefix => 'A');
  //   DefMacroI('\the' . $counter, undef, '\Alph{' . $counter . '}', scope => 'global'); }
  // AssignMapping('counter_for_type', appendix => $counter);
  // Let(T_CS!('\\' . $counter), T_CS!('\@@appendix'), 'global');
  // Let(T_CS!('\@appendix'),    T_CS!('\relax'),      'global');
}

pub fn end_appendices(_state: &mut State) {
  unimplemented!();
  // if (my $counter = LookupMapping('BACKMATTER_ELEMENT', 'ltx:appendix')) {
  //   $counter =~ s/^ltx://;
  //   Let('\the' . $counter,         '\lx@save@theappendex',    'global');
  //   Let('\the' . $counter . '@ID', '\lx@save@theappendex@ID', 'global');
  //   Let(T_CS!('\\' . $counter),     '\lx@save@appendix',       'global');
  //   Let(T_CS!('\@appendix'),        '\lx@save@@appendix',      'global'); }
}

pub fn make_note_tags(
  counter: &str,
  mark_opt: &Option<Digested>,
  tag_opt: Option<Cow<Digested>>,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<HashMap<String, Stored>> {
  if let Some(tag) = tag_opt {
    let mut props = ref_step_id(counter, stomach, state)?;
    let mark = match mark_opt {
      None => tag.clone(),
      Some(mark) => Cow::Borrowed(mark),
    };
    props.insert("mark".to_string(), mark.into());
    props.insert(
      "tags".to_string(),
      stomach
        .digest(
          Tokens!(
            T_BEGIN!(),
            T_CS!("\\def"),
            T_CS!(s!("\\the{counter}")),
            T_BEGIN!(),
            tag.revert(state)?,
            T_END!(),
            T_CS!("\\def"),
            T_CS!(s!("\\typerefnum@{counter}")),
            T_BEGIN!(),
            T_CS!(s!("\\{counter}typerefname")),
            T_SPACE!(),
            tag.revert(state)?,
            T_END!(),
            T_CS!("\\lx@make@tags"),
            T_BEGIN!(),
            T_OTHER!(counter),
            T_END!(),
            T_END!()
          ),
          state,
        )?
        .into(),
    );
    Ok(props)
  } else {
    let mut props = ref_step_counter(counter, false, stomach, state)?;
    let mark = Stored::Digested(match mark_opt {
      None => digest_text(Tokens!(T_CS!(s!("\\the{counter}"))), stomach, state)?,
      Some(mark) => mark.clone(),
    });
    props.insert("mark".to_string(), mark);
    Ok(props)
  }
}

// Find any pairs of footnotemark & footnotetext;
// Move the contents of the text to the mark, removing the text node.
pub fn relocate_footnote(
  document: &mut Document,
  node: &mut Node,
  state: &mut State,
) -> Result<()> {
  if let Some(caps) = NOTE_TEXT_END.captures(&node.get_attribute("role").unwrap_or_default()) {
    let notetype = caps.get(1).map_or("", |m| m.as_str()); // Eg "footnote", "endnote",...
    if let Some(mark) = node.get_attribute("mark") {
      for mut marknote in document.findnodes(
        &format!(".//ltx:note[@role='{notetype}mark'][@mark='{mark}']"),
        None,
        state,
      ) {
        relocate_footnote_aux(document, notetype, &mut marknote, node, state)?;
      }
    }
  } else if let Some(caps) = NOTE_MARK_END.captures(&node.get_attribute("role").unwrap_or_default())
  {
    let notetype = caps.get(1).map_or("", |m| m.as_str()); // Eg "footnote", "endnote",...
    if let Some(mark) = node.get_attribute("mark") {
      for mut textnote in document.findnodes(
        &format!(".//ltx:note[@role='{notetype}text'][@mark='{mark}']"),
        None,
        state,
      ) {
        relocate_footnote_aux(document, notetype, node, &mut textnote, state)?;
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
  state: &mut State,
) -> Result<()> {
  // textnote.get_parent().unwrap().remove_child(textnote);
  textnote.unlink();
  document.append_clone(marknote, textnote.get_child_nodes(), state)?;
  document.set_attribute(marknote, "role", notetype, state)?;
  if let Some(labels) = textnote.get_attribute("labels") {
    document.generate_id(marknote, "", state)?;
    document.set_attribute(marknote, "labels", &labels, state)?;
  }
  Ok(())
}

pub fn only_preamble(cs: &str, stomach: &mut Stomach, state: &mut State) {
  if !state.lookup_bool("inPreamble") {
    Error!(
      "unexpected",
      cs,
      stomach,
      state,
      "The current command '{cs}' can only appear in the preamble"
    );
  }
}

pub fn tabular_bindings(template:Template, mut properties: HashMap<String,Stored>, gullet:&mut Gullet, state:&mut State) -> Result<()> {
  if !properties.contains_key("guess_headers") {
    if let Some(v) = state.lookup_value("GUESS_TABULAR_HEADERS") {
      properties.insert(String::from("guess_headers"), v.clone());
    }
  }
  let attributes_entry = properties.entry(String::from("attributes")).or_insert_with(|| Stored::HashStored(HashMap::default()));
  if let Stored::HashStored(ref mut attributes) = attributes_entry {
    if ! attributes.contains_key("colsep") {
      let sep = state.lookup_dimension("\\tabcolsep");
      if sep.is_some() && (sep.unwrap().value_of() != state.lookup_dimension("\\lx@default@tabcolsep").unwrap().value_of()) {
        attributes.insert(String::from("colsep"), sep.into());
      }
    }
    if ! attributes.contains_key("rowsep") {
      let astr = gullet.do_expand(T_CS!("\\arraystretch"), state)?.to_string();
      if astr != "1" {
        let astr_int = astr.parse::<i64>()?;
        attributes.insert(String::from("rowsep"), s!("{}em",Dimension::new(astr_int - 1)).into());
      }
    }
  }
  if !properties.contains_key("strut") {
    properties.insert(String::from("strut"), state.lookup_register("\\baselineskip", Vec::new())
      .unwrap().multiply(Float::new_f64(1.5)).into());
  }    // Account for html space
  alignment_bindings(template, "text", properties);
  state.let_i(&T_CS!("\\\\"),            T_CS!("\\@tabularcr"), None, gullet);
  state.let_i(&T_CS!("\\tabularnewline"), T_CS!("\\\\"), None, gullet);
  // NOTE: Fit this back in!!!!!!!
  // # Do like AddToMacro, but NOT global!
  for name in ["@row@before", "@row@after", "@column@before", "@column@after"] {
    let cs = T_CS!(s!("\\{name}"));
    let cs_def = state.lookup_definition(&cs).unwrap();
    let mut expansion = cs_def.get_expansion().unwrap().clone();
    expansion.push(T_CS!(s!("\\@tabular{name}")));
    def_macro(cs, None, expansion, None, state);
  }
  Ok(())
}
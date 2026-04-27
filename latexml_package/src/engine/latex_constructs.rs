///**********************************************************************
/// Rust port of LaTeXML's `latex_constructs.pool.ltxml`.
///
/// Organized following
///  "`LaTeX`: A Document Preparation System"
///   by Leslie Lamport
///   2nd edition
/// Addison Wesley, 1994
/// Appendix C. Reference Manual
///**********************************************************************
/// NOTE: This will be loaded after `TeX.pool`, so it inherits.
///**********************************************************************
use crate::engine::base_utilities::insert_frontmatter;
use crate::engine::tex_tables::alignment_bindings;
use crate::prelude::*;
use latexml_core::alignment::template::TemplateConfig;
use latexml_core::digested::DigestedData;
use std::collections::VecDeque;

/// Walk a `Digested` and concatenate its text content (for attribute use,
/// matching Perl's `setAttribute(..., DigestText(...))` semantics). Tbox
/// children contribute their text; nested Lists recurse; `\hskip`-style
/// Whatsits (which are side-effect-only constructors with no text content)
/// fall back to `dimension_to_spaces(width)` instead of reverting to the
/// macro name. All other Whatsits use their normal `get_string` path.
fn digested_to_text(d: &latexml_core::digested::Digested) -> Result<String> {
  use std::ops::Deref;
  let mut out = String::new();
  match d.data() {
    DigestedData::TBox(b) => out.push_str(&b.borrow().get_string()?),
    DigestedData::List(l) => {
      for child in l.borrow().boxes.iter() {
        out.push_str(&digested_to_text(child)?);
      }
    },
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      if let Some(Stored::Dimension(width)) = w.get_property("width").as_ref().map(Deref::deref) {
        out.push_str(&super::tex_glue::dimension_to_spaces(*width));
      } else {
        out.push_str(&w.get_string()?);
      }
    },
    _ => out.push_str(&d.to_string()),
  }
  Ok(out)
}

// Mirrors Perl `Package.pm` (`split(/\s*,\s*/, $options)`) — strips
// whitespace on BOTH sides of each comma so option names are normalized
// when LaTeX line-wraps the option list (e.g. `[twocolumn,amsmath\n
// ,amssymb]`). Without leading `\s*` we'd get `"amsmath\n"` and the
// declared option callback wouldn't fire — silently turning the option
// into an unused-global.
static OPTS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*,\s*").unwrap());
static SEMIVERBATIM_CHARS: [char; 4] = ['%', '\\', '{', '}'];
static NOTE_TEXT_END: Lazy<Regex> = Lazy::new(|| Regex::new("^(\\w+?)text$").unwrap());
static NOTE_MARK_END: Lazy<Regex> = Lazy::new(|| Regex::new("^(\\w+?)mark$").unwrap());

//======================================================================
// LaTeX helper functions (moved from latex_functions.rs)
// Perl: inline in latex_constructs.pool.ltxml
//======================================================================

pub fn start_appendices(kind: &str) { begin_appendices(kind) }

pub fn begin_appendices(counter: &str) {
  let the_ctr = s!("\\the{counter}");
  let the_ctr_id = s!("\\the{counter}@ID");
  let cs_ctr = T_CS!(s!("\\{counter}"));
  state::let_i(
    &T_CS!("\\lx@save@theappendex"),
    &T_CS!(&the_ctr),
    Some(Scope::Global),
  );
  state::let_i(
    &T_CS!("\\lx@save@theappendex@ID"),
    &T_CS!(&the_ctr_id),
    Some(Scope::Global),
  );
  state::let_i(&T_CS!("\\lx@save@appendix"), &cs_ctr, Some(Scope::Global));
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
  let has_chapter = lookup_definition(&T_CS!("\\c@chapter"))
    .ok()
    .flatten()
    .is_some();
  if has_chapter && counter != "chapter" {
    let _ = new_counter(
      counter,
      "chapter",
      Some(NewDefault!(NewCounterOptions, idprefix => "A")),
    );
    let expansion: String = s!("\\thechapter.\\Alph{{{counter}}}");
    let _ = def_macro(
      T_CS!(the_ctr),
      None,
      Some(ExpansionBody::from(expansion)),
      Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))),
    );
  } else {
    let _ = new_counter(
      counter,
      "document",
      Some(NewDefault!(NewCounterOptions, idprefix => "A")),
    );
    let expansion: String = s!("\\Alph{{{counter}}}");
    let _ = def_macro(
      T_CS!(the_ctr),
      None,
      Some(ExpansionBody::from(expansion)),
      Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))),
    );
  }
  let _ = state::assign_register(
    &s!("\\c@{counter}"),
    RegisterValue::Number(Number::new(0)),
    None,
    Vec::new(),
  );
  state::assign_mapping("counter_for_type", "appendix", Some(counter.to_string()));
  state::let_i(&cs_ctr, &T_CS!("\\@@appendix"), Some(Scope::Global));
  state::let_i(
    &T_CS!("\\@appendix"),
    &T_CS!("\\relax"),
    Some(Scope::Global),
  );
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

pub fn relocate_footnote(document: &mut Document, node: &mut Node) -> Result<()> {
  if let Some(caps) = NOTE_TEXT_END.captures(&node.get_attribute("role").unwrap_or_default()) {
    let notetype = caps.get(1).map_or("", |m| m.as_str());
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
    let notetype = caps.get(1).map_or("", |m| m.as_str());
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

fn relocate_footnote_aux(
  document: &mut Document,
  notetype: &str,
  marknote: &mut Node,
  textnote: &mut Node,
) -> Result<()> {
  document.append_clone(marknote, textnote.get_child_nodes())?;
  document.set_attribute(marknote, "role", notetype)?;
  if let Some(labels) = textnote.get_attribute("labels") {
    document.generate_id(marknote, "")?;
    document.set_attribute(marknote, "labels", &labels)?;
  }
  document.safe_unlink(textnote.clone());
  Ok(())
}

pub fn only_preamble(cs: &str) -> Result<()> {
  if !state::lookup_bool_sym(pin!("inPreamble")) {
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
  for col in template.get_columns_mut() {
    if let Some(ref after) = col.after {
      if after
        .unlist_ref()
        .iter()
        .any(|t| t.with_str(|s| s.contains("intercol")))
      {
        col.has_intercol_after = true;
      }
    }
  }
  for col in template.get_repeated_mut() {
    if let Some(ref after) = col.after {
      if after
        .unlist_ref()
        .iter()
        .any(|t| t.with_str(|s| s.contains("intercol")))
      {
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
  if !properties.contains_key("strut") {
    properties.insert("isLaTeX", Stored::Bool(true));
    if let Ok(Some(bs)) = lookup_register("\\baselineskip", Vec::new()) {
      properties.insert("strut", bs.into());
    }
  }
  alignment_bindings(template, String::from("text"), properties, xml_attributes);
  state::let_i(&T_CS!("\\\\"), &T_CS!("\\@tabularcr"), None);
  state::let_i(&T_CS!("\\lx@intercol"), &T_CS!("\\lx@text@intercol"), None);
  state::let_i(&T_CS!("\\tabularnewline"), &T_CS!("\\\\"), None);
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

/// Port of Perl's `latexChangeCase` function.
/// Applies Unicode case conversion (not TeX uccode/lccode tables) to tokens.
/// Converts CC_SPACE to T_SPACE (matching latex3 behavior).
/// Handles \protect + excluded CS tokens (text_case_exclude mapping).
fn lx_change_case_tokens(req_case: &str, tokens: &Tokens) -> Result<Vec<Token>> {
  let mouth = Mouth::new("", None)?;
  gullet::open_mouth(mouth, false);
  gullet::unread(tokens.clone());
  let result = lx_read_and_change_case(req_case)?;
  gullet::close_mouth(true)?;
  Ok(result)
}

fn lx_read_and_change_case(req_case: &str) -> Result<Vec<Token>> {
  let mut result = vec![];
  let mut in_math = false;
  let mut is_upper = req_case == "upper" || req_case == "sentence" || req_case == "title";
  loop {
    let tok = match gullet::read_x_token(Some(false), false, None)? {
      None => break,
      Some(t) => t,
    };
    let cc = tok.get_catcode();
    if cc == Catcode::MATH {
      in_math = !in_math;
      result.push(tok);
    } else if in_math {
      result.push(tok);
    } else if cc == Catcode::LETTER || cc == Catcode::OTHER {
      let new_str: String = tok.with_str(|s| {
        if is_upper {
          s.chars().flat_map(|c| c.to_uppercase()).collect()
        } else {
          s.chars().flat_map(|c| c.to_lowercase()).collect()
        }
      });
      let changed = tok.with_str(|s| s != new_str.as_str());
      let new_tok = if changed {
        Token::new(new_str, cc)
      } else {
        tok
      };
      result.push(new_tok);
      if req_case == "sentence" || req_case == "title" {
        is_upper = false;
      }
    } else if cc == Catcode::SPACE {
      result.push(T_SPACE!());
      if req_case == "title" {
        is_upper = true;
      }
    } else if cc == Catcode::CS && tok.with_str(|s| s == "\\protect") {
      if let Some(next_tok) = gullet::read_token()? {
        // Perl: $cs->getString (full CS name). Munged-robust CSes carry a
        // trailing space — canonicalise to NO trailing space for the
        // exclude lookup (matches \AddToNoCaseChangeList storage format),
        // and to "CS + trailing space" for the case-mapping lookup
        // (matches `\lx@prepare@case@mapping` storage format, which is
        // `$lower->getString . ' '` in Perl).
        let next_key_bare = next_tok.with_str(|s| s.trim_end().to_string());
        let next_key_case = format!("{} ", next_key_bare);
        if lookup_mapping("text_case_exclude", &next_key_bare).is_some() {
          let opt = gullet::read_optional(None)?;
          let arg = gullet::read_arg(ExpansionLevel::Off)?;
          result.push(tok);
          result.push(next_tok);
          if let Some(opt_tokens) = opt {
            let converted = lx_change_case_tokens(req_case, &opt_tokens)?;
            result.push(T_OTHER!("["));
            result.extend(converted);
            result.push(T_OTHER!("]"));
          }
          result.push(T_BEGIN!());
          result.extend(arg.unlist());
          result.push(T_END!());
        } else if let Some(changed) = lookup_mapping(
          if is_upper {
            "text_uppercase"
          } else {
            "text_lowercase"
          },
          &next_key_case,
        ) {
          if let Stored::Token(changed_tok) = changed {
            result.push(changed_tok);
          } else {
            result.push(tok);
            result.push(next_tok);
          }
          if req_case == "sentence" || req_case == "title" {
            is_upper = false;
          }
        } else {
          result.push(tok);
          result.push(next_tok);
        }
      }
    } else {
      result.push(tok);
    }
  }
  Ok(result)
}

const PM_ORDINAL_SUFFICES: &[&str] = &["th", "st", "nd", "rd", "th", "th", "th", "th", "th", "th"];
const FNSYMBOLS: &[&str] = &[
  "*",
  "\u{2020}",
  "\u{2021}",
  "\u{00A7}",
  "\u{00B6}",
  "\u{2225}",
  "**",
  "\u{2020}\u{2020}",
  "\u{2021}\u{2021}",
];

//**********************************************************************
// C.6 Displayed Paragraphs
//**********************************************************************
/// Perl: setupAligningContext — saves [node, lastChild] for deferred class application.
fn setup_aligning_context(doc: &mut Document) {
  if let Some(node) = doc.get_element() {
    // Save node and its current last child so we only apply to NEW children later
    state::assign_value("ALIGNING_NODE", Stored::Node(node.clone()), None);
    if let Some(last) = node.get_last_child() {
      state::assign_value("ALIGNING_PREV_CHILD", Stored::Node(last), None);
    } else {
      state::assign_value("ALIGNING_PREV_CHILD", Stored::None, None);
    }
  }
}
/// Perl: applyAligningContext — applies align/class to children added AFTER \centering.
fn apply_aligning_context(document: &mut Document, align: &str, class: &str) -> Result<()> {
  // with_value avoids two Stored envelope clones; Node is Rc-backed so we
  // still pay a Rc::clone inside the closure but skip the enum match work.
  let node_opt = state::with_value("ALIGNING_NODE", |v| match v {
    Some(Stored::Node(node)) => Some(node.clone()),
    _ => None,
  });
  if let Some(node) = node_opt {
    let previous_opt = state::with_value("ALIGNING_PREV_CHILD", |v| match v {
      Some(Stored::Node(prev)) => Some(prev.clone()),
      _ => None,
    });
    let children = node.get_child_nodes();
    let mut past_previous = previous_opt.is_none(); // if no previous, apply to all
    for mut child in children {
      if !past_previous {
        if let Some(ref prev) = previous_opt {
          if child == *prev {
            past_previous = true;
          }
        }
        continue;
      }
      if child.get_type() == Some(libxml::tree::NodeType::ElementNode) {
        crate::engine::base_utilities::set_align_or_class(document, &mut child, align, class)?;
      }
    }
  }
  Ok(())
}

fn before_digest_verbatim() -> Result<Vec<Digested>> {
  bgroup();
  let mut stuff = Vec::new();
  if let Some(b) = state::lookup_tokens("@environment@verbatim@atbegin") {
    stuff.push(stomach::digest(b.unlist())?);
  }
  AssignValue!("current_environment", "verbatim");
  DefMacro!("\\@currenvir", "verbatim");
  MergeFont!(family => "typewriter");
  Ok(stuff)
}

fn after_digest_verbatim(starred: bool, whatsit: &mut Whatsit) -> Result<()> {
  // makes you wonder if the `get_font` API should be working with Rc<Font> in the first place...
  let font: Option<Rc<Font>> = whatsit.get_font()?.map(|ft| Rc::new((*ft).to_owned()));
  let loc = whatsit.get_locator();
  let (end, space) = if starred {
    ("\\end{verbatim*}", '\u{2423}')
  } else {
    ("\\end{verbatim}", ' ')
  };
  let mut lines: Vec<_> = Vec::new();
  while let Some(next_line) = gullet::read_raw_line() {
    let mut line = next_line.as_str();
    let mut exiting = false;
    if let Some((final_line, remaining)) = line.split_once(end) {
      line = final_line;
      gullet::unread_one(T_CR!());
      gullet::unread(Tokenize!(remaining));
      exiting = true;
    }
    // The raw chars will still have to be decoded (but not space!!)
    let mut decoded_line: String = String::new();
    for c in line.chars() {
      if c == ' ' {
        decoded_line.push(space);
      } else {
        let decoded_c = font::decode_string(arena::pin_char(c), Some("OT1_typewriter"), true);
        arena::with(decoded_c, |c_str| decoded_line.push_str(c_str));
      }
    }
    decoded_line.push('\n');
    lines.push(arena::pin(decoded_line));
    if exiting {
      break;
    }
  }
  if let Some(last_line) = lines.last() {
    if *last_line == arena::pin_static("\n") {
      lines.pop();
    }
  }
  // Note last line ends up as Whatsit's "trailer"
  if let Some(b) = state::lookup_tokens("@environment@verbatim@atend") {
    lines.push(arena::pin(stomach::digest(b)?.to_string()));
  }
  egroup()?;
  lines.push(arena::pin_static(end));
  let boxes = lines
    .into_iter()
    .map(|line| {
      Tbox::new(
        line,
        font.clone(),
        Some(loc),
        Token {
          text: line,
          code: Catcode::OTHER,
        }
        .into(),
        SymHashMap::default(),
      )
      .into()
    })
    .collect();
  whatsit.set_body(boxes);
  Ok(())
}

//======================================================================
// C.7.1 Math Mode Environments
//======================================================================
// # This provides {equation} with the capabilities for tags, nonumber, etc
// # even though stock LaTeX provides no means to override them.
// #   preset => boolean
// #   postset => boolean
// #   deferretract=>boolean
pub fn prepare_equation_counter(options: SymHashMap<Stored>) {
  // Guard: ensure the equation counter exists — normally created by article.cls,
  // but standalone classes (jpsj2, appolb, etc.) may not define it.
  if lookup_definition(&T_CS!("\\theequation@ID"))
    .ok()
    .flatten()
    .is_none()
  {
    let _ = new_counter(
      "equation",
      "section",
      Some(NewDefault!(NewCounterOptions, idprefix => "E")),
    );
  }
  state::assign_value(
    "EQUATION_NUMBERING",
    Stored::HashStored(options),
    Some(Scope::Global),
  );
}
pub fn before_equation() -> Result<()> {
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
        Some(other) => {
          log::warn!("eq counter should be stored as string, was instead: {other:?}");
          String::from("equation")
        },
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
pub fn after_equation(whatsit: Option<&mut Whatsit>) -> Result<()> {
  // Phase 1: Gather all needed data from state (immutable borrows only)
  enum EqAction {
    Retract,
    Postset,
    TagsUpdate,
    None,
  }
  let mut action = EqAction::None;
  let mut is_aligned = false;
  let mut is_numbered_for_postset = false;
  let mut ctr = String::from("equation");
  with_value("EQUATION_NUMBERING", |eq_num_opt| {
    if let Some(Stored::HashStored(ref numbering)) = eq_num_opt {
      is_aligned = matches!(numbering.get("aligned"), Some(&Stored::Bool(true)));
      is_numbered_for_postset = matches!(numbering.get("numbered"), Some(&Stored::Bool(true)));
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
      let invoked_tags = build_invocation(T_CS!("\\lx@make@tags"), vec![Some(Tokens::new(
        Explode!(ctr),
      ))])?;
      let stored_tags_update = Stored::Digested(stomach::digest(invoked_tags)?);
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
    // In Perl, these get stored as $$row{id}, $$row{tags} on the row object.
    // Store on the current alignment row so each row retains its own props.
    if let Some(alignment_digested) = lookup_alignment() {
      if let Some(alignment_cell) = alignment_digested.alignment_cell() {
        let mut alignment = alignment_cell.borrow_mut();
        if let Some(row) = alignment.current_row_mut() {
          for (key, val) in &props {
            row.properties.insert(arena::to_string(*key), val.clone());
          }
        }
      }
    }
  } else if let Some(w) = whatsit {
    w.set_properties(props);
  }
  Ok(())
}
/// Perl: latex_constructs.pool.ltxml lines 2025-2035
fn retract_equation() {
  // Phase 1: Gather data (immutable borrows)
  let (ctr, is_preset, is_numbered) = with_value("EQUATION_NUMBERING", |eq_num_opt| {
    let numbering = match eq_num_opt {
      Some(Stored::HashStored(n)) => n,
      _ => return (String::from("equation"), false, false),
    };
    let is_numbered = matches!(numbering.get("numbered"), Some(&Stored::Bool(true)));
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
      let is_preset = matches!(tags.get("preset"), Some(&Stored::Bool(true)));
      (ctr, is_preset, is_numbered)
    })
  });
  // Phase 2: Mutate state (borrows released)
  if is_preset {
    // counter (or ID counter) was stepped, so decrement it.
    let counter_name = if is_numbered {
      ctr.clone()
    } else {
      s!("UN{}", ctr)
    };
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
  // Ensure @equationgroup counter exists — it's normally created by article.cls,
  // but standalone classes (appolb, jpsj2, etc.) may not define it.
  if lookup_definition(&T_CS!("\\the@equationgroup@ID"))?.is_none() {
    NewCounter!("@equationgroup", "document", idprefix => "EG", idwithin => "section");
  }

  // Perl: 3-column template: col1=right, col2=center, col3=left
  let col1 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"),
      T_MATH!(),
      T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!()])),
    empty: true,
    ..Cell::default()
  };
  let col2 = Cell {
    before: Some(Tokens::new(vec![
      T_CS!("\\hfil"),
      T_MATH!(),
      T_CS!("\\displaystyle"),
    ])),
    after: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\hfil")])),
    empty: true,
    ..Cell::default()
  };
  let col3 = Cell {
    before: Some(Tokens::new(vec![T_MATH!(), T_CS!("\\displaystyle")])),
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
  if let Ok(Some(acol)) = state::lookup_register("\\arraycolsep", Vec::new()) {
    let colsep = acol.pt_value(None) * 2.0;
    if colsep > 0.0 {
      xml_attrs.insert(String::from("colsep"), s!("{}pt", colsep));
    }
  }
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
      // Perl: $$row{id} and $$row{tags} are passed via props from be_absorbed.
      // The id was stored on the row during after_equation.
      if let Some(id) = props.remove("id") {
        props.insert(String::from("xml:id"), Stored::from(id.to_string()));
      }
      // Extract tags (Digested) before converting to string props
      let tags_digested = props.remove("tags");
      let str_props: HashMap<String, String> =
        props.into_iter().map(|(k, v)| (k, v.to_string())).collect();
      document.open_element("ltx:equation", Some(str_props), None)?;
      // If we have digested tags, absorb them into the opened element
      if let Some(Stored::Digested(d)) = tags_digested {
        document.absorb(&d, None)?;
      }
      Ok(())
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
  // NOTE: Perl's eqnarrayBindings does NOT set Let(T_MATH, '\lx@dollar@in@mathmode').
  // eqnarray creates the alignment directly (not through alignmentBindings),
  // so the $ tokens in its template use \lx@dollar@default — same as amsRearrangeableBindings.
  state::let_i(&T_CS!("\\\\"), &T_CS!("\\lx@alignment@newline"), None);
  state::let_i(&T_CS!("\\lx@intercol"), &T_CS!("\\lx@math@intercol"), None);
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
  // Perl: Let('\lx@eqnarray@save@label', '\lx@label');
  // Save the original \label as \lx@eqnarray@save@label
  state::let_i(&T_CS!("\\lx@eqnarray@save@label"), &T_CS!("\\label"), None);
  // Perl: Let('\label', '\lx@eqnarray@label');
  // Redirect \label to the noalign version so it runs at the equation (row) level
  state::let_i(&T_CS!("\\label"), &T_CS!("\\lx@eqnarray@label"), None);
  Ok(())
}

/// Perl: rearrangeEqnarray (latex_constructs.pool.ltxml L2356-2445)
/// Analyzes column patterns in eqnarray and rearranges into MathFork structures.
fn rearrange_eqnarray(document: &mut Document, equationgroup: &mut Node) -> Result<()> {
  use crate::engine::base_xmath::{equationgroup_join_cols, equationgroup_join_rows};

  struct EqRow {
    node:      Node,
    cols:      Vec<Node>,
    has_l:     bool,
    has_m:     bool,
    has_r:     bool,
    numbered:  bool,
    _labelled: bool,
  }

  // Scan the equations (rows)
  let mut rows: Vec<EqRow> = Vec::new();
  let equation_nodes: Vec<Node> = document.findnodes("ltx:equation", Some(equationgroup));
  for rownode in equation_nodes {
    let cells: Vec<Node> = document.findnodes("ltx:_Capture_", Some(&rownode));
    let has_l = cells
      .first()
      .is_some_and(|c| !c.get_child_nodes().is_empty());
    let has_m = cells
      .get(1)
      .is_some_and(|c| !c.get_child_nodes().is_empty());
    let has_r = cells
      .get(2)
      .is_some_and(|c| !c.get_child_nodes().is_empty());
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
    let keepcol = if n_l > 0 {
      0
    } else if n_m > 0 {
      1
    } else {
      2
    };
    // Remove empty columns (in reverse order to preserve indices)
    for c in (0..3).rev() {
      if c == keepcol {
        continue;
      }
      for row in rows.iter() {
        if let Some(col) = row.cols.get(c) {
          document.safe_unlink(col.clone());
        }
      }
    }
    // Check if any column begins with a RELOP → join rows
    let begins_with_relop = rows.iter().any(|row| {
      row
        .cols
        .get(keepcol)
        .and_then(|c| {
          c.get_child_elements()
            .into_iter()
            .next()
            .and_then(|first| first.get_attribute("role").map(|r| r == "RELOP"))
        })
        .unwrap_or(false)
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
      if eqs.is_empty() || (numbered && row.numbered && row._labelled) {
        class = "odd";
      } else {
        class = "continue";
      }
    } else {
      // All columns empty
      class = "remove";
    }

    if class == "remove" {
      document.safe_unlink(row.node.clone());
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

fn clean_class_name(name: &str) -> String {
  name
    .trim()
    .chars()
    .filter(|c| c.is_alphanumeric())
    .collect()
}

fn stored_string_list(keys: &[&str]) -> Stored {
  let deque: VecDeque<Stored> = keys.iter().map(|k| Stored::from(k.to_string())).collect();
  Stored::VecDequeStored(deque)
}

fn init_savable_theorem_parameters(keys: Vec<&str>) {
  state::assign_value(
    "SAVABLE_THEOREM_PARAMETERS",
    stored_string_list(&keys),
    Some(Scope::Global),
  );
}

pub fn get_savable_keys() -> Vec<String> {
  match state::lookup_value("SAVABLE_THEOREM_PARAMETERS") {
    Some(Stored::VecDequeStored(keys)) => keys.iter().map(|k| k.to_string()).collect(),
    _ => vec![
      "\\thm@bodyfont".into(),
      "\\thm@headpunct".into(),
      "\\thm@styling".into(),
      "\\thm@headstyling".into(),
      "thm@swap".into(),
    ],
  }
}

pub fn set_savable_theorem_parameters(keys: Vec<&str>) {
  state::assign_value(
    "SAVABLE_THEOREM_PARAMETERS",
    stored_string_list(&keys),
    Some(Scope::Global),
  );
}

pub fn save_theorem_style(name: &str, saved: Vec<(String, Stored)>) {
  let key = s!("THEOREM_{name}_PARAMETERS");
  let deque: VecDeque<Stored> = saved
    .into_iter()
    .flat_map(|(k, v)| vec![Stored::from(k), v])
    .collect();
  state::assign_value(&key, Stored::VecDequeStored(deque), Some(Scope::Global));
}

pub fn use_theorem_style(name: &str) {
  let savable_keys = get_savable_keys();
  let params_key = s!("THEOREM_{name}_PARAMETERS");
  if let Some(Stored::VecDequeStored(params)) = state::lookup_value(&params_key) {
    let params_vec: Vec<Stored> = params.into_iter().collect();
    let mut i = 0;
    while i + 1 < params_vec.len() {
      let key = params_vec[i].to_string();
      let val = params_vec[i + 1].clone();
      if savable_keys.iter().any(|k| k == &key) {
        if key.starts_with('\\') {
          let tokens = match val {
            Stored::Tokens(t) => t,
            Stored::Bool(_) => {
              // bool stored for a register key — skip
              i += 2;
              continue;
            },
            // Values round-tripping through tokens — use internal cattable so
            // any `\lx@…` names re-tokenize as single CS (not `\lx`+`@…`).
            _ => mouth::tokenize_internal(&val.to_string()),
          };
          let _ = state::assign_register(&key, RegisterValue::Tokens(tokens), None, vec![]);
        } else {
          state::assign_value(&key, val, None);
        }
      }
      i += 2;
    }
  }
}

pub fn define_new_theorem(
  flag: Option<Tokens>,
  thmset: Tokens,
  otherthmset: Option<Tokens>,
  typ: Option<Tokens>,
  within: Option<Tokens>,
) -> Result<()> {
  let thmset_str = thmset.to_string();
  let classname = clean_class_name(&thmset_str);
  let listname = {
    let mut ln = s!("theorem:{thmset_str}");
    ln.retain(|c| !c.is_whitespace());
    ln = ln.replace('\'', "prime");
    ln = ln.replace('?', "question");
    ln = ln.replace('#', "hash");
    ln
  };
  let otherthmset_str = otherthmset
    .as_ref()
    .map(|t| t.to_string())
    .filter(|s| !s.is_empty());
  let has_type = typ.as_ref().is_some_and(|t| !t.is_empty());
  let is_starred = flag.is_some();

  let within_str = if let Some(ref w) = within {
    let ws = digest_literal(w.clone())?.to_string();
    if ws.is_empty() { None } else { Some(ws) }
  } else {
    None
  };

  let counter = otherthmset_str
    .clone()
    .unwrap_or_else(|| thmset_str.clone());
  let counter = counter.replace(' ', ".");

  // If counter != thmset, record mapping
  if counter != thmset_str {
    AssignMapping!("counter_for_type", &thmset_str => &counter);
    DefMacro!(
      T_CS!(s!("\\the{thmset_str}")),
      None,
      Some(ExpansionBody::Tokens(Tokens::new(vec![T_CS!(s!("\\the{counter}"))]))),
      scope => Some(Scope::Global)
    );
  }

  let numbering = {
    let reg = LookupRegister!("\\thm@numbering");
    if let RegisterValue::Tokens(t) = reg {
      t.to_string()
    } else {
      "\\arabic".into()
    }
  };

  let is_starred = is_starred || numbering.is_empty();

  if otherthmset_str.is_none() {
    let idprefix = s!("Thm{}", classname.replace('*', "."));
    let c_counter = s!("\\c@{counter}");
    if !is_defined(&c_counter) {
      let within_ref = within_str.as_deref().unwrap_or("");
      NewCounter!(&counter, within_ref, idprefix => &idprefix);
    }
    // Define \the<counter>
    if !numbering.is_empty() {
      let the_counter_body = if let Some(ref w) = within_str {
        s!("\\csname the{w}\\endcsname\\@thmcountersep{numbering}{{{counter}}}")
      } else {
        s!("{numbering}{{{counter}}}")
      };
      DefMacro!(
        T_CS!(s!("\\the{counter}")),
        None,
        Some(ExpansionBody::Tokens(mouth::tokenize_internal(&the_counter_body))),
        scope => Some(Scope::Global)
      );
    }
  }

  // Save current theorem style params for this theorem name
  let savable_keys = get_savable_keys();
  let mut saved_params: Vec<(String, Stored)> = Vec::new();
  for key in &savable_keys {
    if key.starts_with('\\') {
      let reg = LookupRegisterOrDefault!(key);
      let tokens = match reg {
        RegisterValue::Tokens(t) => t,
        _ => Tokens!(),
      };
      saved_params.push((key.clone(), Stored::Tokens(tokens)));
    } else {
      let val = state::lookup_value(key).unwrap_or(Stored::None);
      saved_params.push((key.clone(), val));
    }
  }
  save_theorem_style(&thmset_str, saved_params);

  // Define \lx@name@<thmset>
  let thmname_cs = s!("\\lx@name@{thmset_str}");
  if has_type {
    let type_tokens = typ.clone().unwrap();
    DefMacro!(
      T_CS!(&thmname_cs),
      None,
      Some(ExpansionBody::Tokens(type_tokens)),
      scope => Some(Scope::Global)
    );
  } else {
    DefMacro!(
      T_CS!(&thmname_cs),
      None,
      Some(ExpansionBody::Tokens(Tokens!())),
      scope => Some(Scope::Global)
    );
  }

  // Read swap value
  let swap = state::lookup_value("thm@swap")
    .map(|v| match v {
      Stored::Int(n) => n != 0,
      Stored::Bool(b) => b,
      _ => false,
    })
    .unwrap_or(false);

  // Define \fnum@<thmset>
  let fnum_cs = s!("\\fnum@{thmset_str}");
  let fnum_tokens = if is_starred || counter.is_empty() {
    Tokens::new(vec![T_CS!(&thmname_cs)])
  } else if swap {
    let mut toks = vec![T_CS!(s!("\\the{counter}"))];
    if has_type {
      toks.push(T_SPACE!());
    }
    toks.push(T_CS!(&thmname_cs));
    Tokens::new(toks)
  } else {
    let mut toks = vec![T_CS!(&thmname_cs)];
    if has_type {
      toks.push(T_SPACE!());
    }
    toks.push(T_CS!(s!("\\the{counter}")));
    Tokens::new(toks)
  };
  DefMacro!(
    T_CS!(&fnum_cs),
    None,
    Some(ExpansionBody::Tokens(fnum_tokens)),
    scope => Some(Scope::Global)
  );

  // Define \format@title@<thmset>
  let format_title_cs = s!("\\format@title@{thmset_str}");
  let headformatter = LookupRegisterOrDefault!("\\thm@headformatter");
  let headformatter_tokens = match headformatter {
    RegisterValue::Tokens(t) => t,
    _ => Tokens!(),
  };

  let format_cs_token = T_CS!(&format_title_cs);
  if !headformatter_tokens.is_empty() {
    // amsthm-style head formatter
    let mut fmt_toks = vec![T_CS!("\\the"), T_CS!("\\thm@headfont")];
    fmt_toks.extend(headformatter_tokens.unlist());
    fmt_toks.push(T_BEGIN!());
    if has_type {
      fmt_toks.extend(typ.clone().unwrap().unlist());
    }
    fmt_toks.push(T_END!());
    fmt_toks.push(T_CS!(s!("\\the{counter}")));
    fmt_toks.push(T_BEGIN!());
    fmt_toks.push(T_PARAM!());
    fmt_toks.push(T_OTHER!("1"));
    fmt_toks.push(T_END!());
    fmt_toks.push(T_CS!("\\the"));
    fmt_toks.push(T_CS!("\\thm@headpunct"));

    let params = parse_parameters("{}", &format_cs_token, true)?;
    DefMacro!(
      format_cs_token,
      params,
      Some(ExpansionBody::Tokens(Tokens::new(fmt_toks))),
      scope => Some(Scope::Global)
    );
  } else {
    // Standard format
    let note_part = if has_type {
      "\\ifx.#1.\\else\\space\\the\\thm@notefont(#1)\\fi"
    } else {
      "#1"
    };
    let fmt_str = s!(
      "{{\\the\\thm@headfont\\lx@tag{{\\csname fnum@{thmset_str}\\endcsname}}{{{note_part}}}\\the\\thm@headpunct}}"
    );
    let params = parse_parameters("{}", &format_cs_token, true)?;
    DefMacro!(
      format_cs_token,
      params,
      Some(ExpansionBody::Tokens(mouth::tokenize_internal(&fmt_str))),
      scope => Some(Scope::Global)
    );
  }

  // Define the environment
  let thmset_for_env = thmset_str.clone();

  // Hand-written replacement closure (compile_replacement! only works with literals)
  let inlist_val = s!("thm {listname}");
  let class_val = s!("ltx_theorem_{classname}");
  let compiled_replacement: Option<ReplacementClosure> = Some(Rc::new(
    move |document: &mut Document, _args: &Vec<Option<Digested>>, props: &SymHashMap<Stored>| {
      let mut av_props: HashMap<String, String> = HashMap::default();
      if let Some(stored) = props.get("id") {
        av_props.insert("xml:id".into(), stored.to_string());
      }
      av_props.insert("inlist".into(), inlist_val.clone());
      av_props.insert("class".into(), class_val.clone());
      let this_font_opt = match props.get("font") {
        Some(Stored::Font(f)) => Some(Cow::Borrowed(&**f)),
        Some(Stored::FontDirective(FontDirective::Asset(fa))) => Some(Cow::Borrowed(&**fa)),
        Some(Stored::FontDirective(FontDirective::Closure(code))) => Some(Cow::Owned(code(None)?)),
        _ => None,
      };
      if let Some(this_font) = this_font_opt {
        document.open_element("ltx:theorem", Some(av_props), Some(&this_font))?;
      } else {
        document.open_element("ltx:theorem", Some(av_props), None)?;
      }
      // #tags
      if let Some(stored_digested) = props.get("tags") {
        let digested_opt: Option<Digested> = stored_digested.into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      // <ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>
      let mut title_av: HashMap<String, String> = HashMap::default();
      if let Some(stored) = props.get("titlefont") {
        title_av.insert("font".into(), stored.to_string());
      }
      title_av.insert("_force_font".into(), "true".into());
      let title_font_opt = match props.get("titlefont") {
        Some(Stored::Font(f)) => Some(Cow::Borrowed(&**f)),
        Some(Stored::FontDirective(FontDirective::Asset(fa))) => Some(Cow::Borrowed(&**fa)),
        Some(Stored::FontDirective(FontDirective::Closure(code))) => Some(Cow::Owned(code(None)?)),
        _ => None,
      };
      if let Some(title_font) = title_font_opt {
        document.open_element("ltx:title", Some(title_av), Some(&title_font))?;
      } else {
        document.open_element("ltx:title", Some(title_av), None)?;
      }
      if let Some(stored_digested) = props.get("title") {
        let digested_opt: Option<Digested> = stored_digested.into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      document.close_element("ltx:title")?;
      // #body
      if let Some(stored_digested) = props.get("body") {
        let digested_opt: Option<Digested> = stored_digested.into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      Ok(())
    },
  ));

  // `thmset_for_before` is for the before_digest closure; clone needed
  // because `thmset_str` is moved into `thmset_for_tags` below.
  let thmset_for_before = thmset_str.clone();
  // `thmset_for_tags` and `counter_for_tags` are the last uses of
  // thmset_str and counter — move instead of clone.
  let thmset_for_tags = thmset_str;
  let counter_for_tags = counter;
  let is_starred_for_props = is_starred;
  let has_type_for_props = has_type;

  let mut options = ConstructorOptions {
    mode: Some("internal_vertical".into()),
    scope: Some(Scope::Global),
    ..Default::default()
  };

  // before_digest
  let before_digest_closure: BeforeDigestClosure = Rc::new(move || {
    use_theorem_style(&thmset_for_before);
    let digested = stomach::digest(mouth::tokenize_internal("\\normalfont\\the\\thm@prework"))?;
    Ok(vec![digested])
  });
  options.before_digest.push(before_digest_closure);

  // after_digest_begin
  let after_digest_begin_closure: DigestionClosure = Rc::new(move |whatsit| {
    let name_opt = whatsit.get_arg(1);
    let name_str = name_opt
      .map(|n| n.revert().map(|t| t.to_string()).unwrap_or_default())
      .unwrap_or_default();
    let digest_str = s!("\\the\\thm@bodyfont\\the\\thm@styling\\def\\lx@thistheorem{{{name_str}}}");
    let digested = stomach::digest(mouth::tokenize_internal(&digest_str))?;
    Ok(vec![digested])
  });
  options.after_digest_begin.push(after_digest_begin_closure);

  // before_digest_end
  let before_digest_end_closure: BeforeDigestClosure = Rc::new(move || {
    let digested = stomach::digest(mouth::tokenize_internal(
      "\\thm@doendmark\\the\\thm@postwork",
    ))?;
    Ok(vec![digested])
  });
  options.before_digest_end.push(before_digest_end_closure);

  // after_construct
  let after_construct_closure: ConstructionClosure =
    Rc::new(move |document: &mut Document, _whatsit: &Whatsit| {
      document.maybe_close_element("ltx:theorem")?;
      Ok(())
    });
  options.after_construct.push(after_construct_closure);

  // properties — capture thmset_for_tags / counter_for_tags by move.
  let props_closure: PropertiesClosure = Rc::new(
    #[allow(clippy::ptr_arg)]
    move |args: &Vec<Option<Digested>>| {
      let mut props = SymHashMap::default();

      if !counter_for_tags.is_empty() {
        if is_starred_for_props {
          let ctr_props = ref_step_id(&counter_for_tags)?;
          for (k, v) in ctr_props.iter() {
            props.insert_sym(*k, v.clone());
          }
          // For starred theorems with a type, create tags without the counter number
          if has_type_for_props {
            let tag_tokens = Tokens::new(vec![
              T_BEGIN!(),
              T_CS!("\\let"),
              T_CS!(s!("\\the{}", counter_for_tags)),
              T_CS!("\\@empty"),
              T_CS!("\\lx@make@tags"),
              T_BEGIN!(),
            ]);
            let mut full_toks = tag_tokens.unlist();
            full_toks.extend(mouth::tokenize_internal(&thmset_for_tags).unlist());
            full_toks.push(T_END!());
            full_toks.push(T_END!());
            let tags = stomach::digest(Tokens::new(full_toks))?;
            props.insert("tags", tags.into());
          }
        } else {
          let ctr_props = ref_step_counter(&thmset_for_tags, false)?;
          for (k, v) in ctr_props.iter() {
            props.insert_sym(*k, v.clone());
          }
        }
      }

      // Compute title
      let format_title_cs = s!("\\format@title@{}", thmset_for_tags);
      let mut title_tokens = vec![
        T_BEGIN!(),
        T_CS!("\\the"),
        T_CS!("\\thm@headstyling"),
        T_CS!(&format_title_cs),
        T_BEGIN!(),
      ];
      if let Some(Some(ref arg)) = args.first() {
        title_tokens.extend(arg.revert()?.unlist());
      }
      title_tokens.push(T_END!());
      title_tokens.push(T_END!());

      let title = digest_text(Tokens::new(title_tokens))?;
      let titlefont = title.get_font()?.map(|f| f.into_owned());
      props.insert("title", title.into());
      if let Some(f) = titlefont {
        props.insert("titlefont", Stored::Font(Rc::new(f)));
      }

      Ok(props)
    },
  );
  options.properties = props_closure;

  // Use the OptionalUndigested parameter
  let env_cs = T_CS!(s!("\\begin{{{thmset_for_env}}}"));
  let paramlist = parse_parameters("OptionalUndigested", &env_cs, true)?;
  def_environment(thmset_for_env, paramlist, compiled_replacement, options);

  Ok(())
}

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
    T_CS!("\\@captype"),
    None,
    Tokens::new(ExplodeText!(float_type)),
    None,
  )
  .ok();
  // Perl #2775: rebind \\ to \lx@newline in floats to prevent
  // alignment-token early-return when floats are inside tabulars.
  Let!("\\\\", "\\lx@newline");
  // Perl: AssignRegister('\hsize' => LookupDimension($options{double} ? '\textwidth' :
  // '\columnwidth'));
  let dim_name = if double {
    "\\textwidth"
  } else {
    "\\columnwidth"
  };
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
  state::assign_value(
    "LAST_FLOATTYPE",
    Stored::String(arena::pin(&captype)),
    Some(Scope::Global),
  );
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
        matches!(
          name,
          "ltx:p"
            | "ltx:listing"
            | "ltx:math"
            | "ltx:itemize"
            | "ltx:enumerate"
            | "ltx:quote"
            | "ltx:theorem"
            | "ltx:proof"
            | "ltx:description"
            | "ltx:equation"
            | "ltx:equationgroup"
            | "ltx:verbatim"
        )
      })
    };
    for panel in &mut panels {
      document.add_class(panel, "ltx_figure_panel")?;
    }
    // Insert breaks between panels.
    // Perl inserts break before a standalone panel (if there are prior panels in the row),
    // and after standalone panels at the start. We simplify: insert break between consecutive
    // panels where either the current or next panel is standalone.
    for i in 0..panels.len().saturating_sub(1) {
      if is_standalone(&panels[i]) || is_standalone(&panels[i + 1]) {
        let ns = panels[i].get_namespace();
        let mut break_node = libxml::tree::Node::new("break", ns, document.get_document()).unwrap();
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
  let inner = inners.into_iter().next().unwrap();
  // Check captions: collapse only if they don't BOTH have captions
  let outer_has_caption = float
    .get_child_elements()
    .iter()
    .any(|c| latexml_core::document::get_node_qname(c) == caption_qname);
  let inner_has_caption = inner
    .get_child_elements()
    .iter()
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
    let inner_id = inner
      .get_attribute("xml:id")
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
  document.safe_unlink(inner);
  Ok(())
}

/// Perl: tabbingBindings() — sets up alignment with repeated template and rebinds control chars
fn tabbing_bindings() -> Result<()> {
  // Template: repeated column with before=\lx@text@intercol, after=\hfil\lx@text@intercol
  let col = Cell {
    before: Some(Tokens::new(vec![T_CS!("\\lx@text@intercol")])),
    after: Some(Tokens::new(vec![
      T_CS!("\\hfil"),
      T_CS!("\\lx@text@intercol"),
    ])),
    empty: true,
    ..Cell::default()
  };
  let template = Template::new(TemplateConfig {
    repeated: vec![col],
    ..TemplateConfig::default()
  });

  let mut xml_attrs = HashMap::default();
  xml_attrs.insert(String::from("class"), String::from("ltx_tabbing"));

  let alignment = Alignment::new(AlignmentConfig {
    template:        Some(template),
    open_container:  Rc::new(|document, props| {
      document
        .open_element("ltx:tabular", Some(props), None)
        .map(Option::Some)
    }),
    close_container: Rc::new(|document| document.close_element("ltx:tabular")),
    open_row:        Rc::new(|document, props| {
      let str_props: HashMap<String, String> =
        props.into_iter().map(|(k, v)| (k, v.to_string())).collect();
      document
        .open_element("ltx:tr", Some(str_props), None)
        .and(Ok(()))
    }),
    close_row:       Rc::new(|document| document.close_element("ltx:tr")),
    open_column:     Rc::new(|document, props| {
      document
        .open_element("ltx:td", Some(props), None)
        .map(Option::Some)
    }),
    close_column:    Rc::new(|document| document.close_element("ltx:td")),
    is_math:         false,
    properties:      SymHashMap::default(),
    xml_attributes:  xml_attrs,
  });
  assign_alignment(alignment, None);

  // Rebind control characters within tabbing
  // Perl: Let("\\=", '\@tabbing@tabset') etc.
  state::let_i(&T_CS!("\\="), &T_CS!("\\@tabbing@tabset"), None);
  state::let_i(&T_CS!("\\>"), &T_CS!("\\@tabbing@nexttab"), None);
  state::let_i(&T_CS!("\\\\"), &T_CS!("\\@tabbing@newline"), None);
  state::let_i(&T_CS!("\\kill"), &T_CS!("\\@tabbing@kill"), None);
  state::let_i(&T_CS!("\\+"), &T_CS!("\\@tabbing@increment"), None);
  state::let_i(&T_CS!("\\-"), &T_CS!("\\@tabbing@decrement"), None);
  state::let_i(&T_CS!("\\<"), &T_CS!("\\@tabbing@untab"), None);
  // Save accent definitions before rebinding \' and \`
  state::let_i(&T_CS!("\\@tabbing@'"), &T_CS!("\\'"), None);
  state::let_i(&T_CS!("\\@tabbing@`"), &T_CS!("\\`"), None);
  state::let_i(&T_CS!("\\a"), &T_CS!("\\@tabbing@accent"), None);
  // Rebind \' and \` to tabbing-specific (flush right / hfil)
  state::let_i(&T_CS!("\\'"), &T_CS!("\\@tabbing@flushright"), None);
  state::let_i(&T_CS!("\\`"), &T_CS!("\\@tabbing@hfil"), None);
  state::let_i(&T_CS!("\\pushtabs"), &T_CS!("\\@tabbing@pushtabs"), None);
  state::let_i(&T_CS!("\\poptabs"), &T_CS!("\\@tabbing@poptabs"), None);

  Ok(())
}

pub(crate) fn note_backmatter_element(whatsit: &mut Whatsit, backelement: &str) {
  if let Some(val) = state::lookup_mapping("BACKMATTER_ELEMENT", backelement) {
    whatsit.set_property("backmatterelement", val);
  }
}

pub(crate) fn adjust_backmatter_element(document: &mut Document, whatsit: &Whatsit) -> Result<()> {
  let asif_opt =
    if let Some(Stored::String(asif_sym)) = whatsit.get_property("backmatterelement").as_deref() {
      Some(arena::to_string(*asif_sym))
    } else {
      None
    };
  // Note: We allocate a string here, since
  // it looks like arena::with can deadlock with find_insertion_point
  // we may need a find_insertion_point_sym to avoid that...
  if let Some(asif) = asif_opt {
    let point = document.find_insertion_point(&asif, None)?;
    document.set_node(&point);
  }
  Ok(())
}

// Do this before digesting the body of a bibliography
// Perl: beforeDigestBibliography in latex_constructs.pool.ltxml L3900
pub(crate) fn before_digest_bibliography() -> Result<()> {
  AssignValue!("inPreamble" => false);
  Digest!("\\@lx@inbibliographytrue")?;
  DefMacro!("\\bibliographystyle{}", "");
  DefMacro!("\\bibliography {}", "");
  // avoid \let-based redefinitions of the ending.
  Let!("\\endthebibliography", "\\saved@endthebibliography");
  ResetCounter!("@bibitem");
  Ok(())
}

// Since SOME people seem to write bibliographies w/o \bibitem,
// just blank lines between apparent entries,
// Making \par do a \bibitem{} works, but screws up valid
// bibliographies with blank lines!
// So, let's do some redirection!
fn setup_pseudo_bibitem() -> Result<()> {
  Let!("\\save@bibitem", "\\bibitem");
  Let!("\\save@par", "\\par");
  Let!("\\save@backbackslash", "\\\\");
  Let!("\\bibitem", "\\restoring@bibitem");
  Let!("\\par", "\\par@in@bibliography");
  Let!("\\\\", "\\par@in@bibliography");
  Let!("\\vskip", "\\vskip@in@bibliography");
  // Moreover some people use \item instead of \bibitem
  Let!("\\item", "\\item@in@bibliography");
  // And protect from redefinitions.
  Let!("\\newblock", "\\lx@bibnewblock");
  // Risky, but when bibliography immediatesly starts with text (no implied \par)
  if let Some(token) = gullet::read_non_space()? {
    gullet::unread_one(token);
    if !token.is_executable() {
      gullet::unread_one(T_CS!("\\par"));
    }
  }
  Ok(())
}
// This sub does things that would commonly be needed when starting a bibliography
// setting the ID, etc...
pub(crate) fn begin_bibliography(whatsit: &mut Whatsit) -> Result<()> {
  begin_bibliography_clean(whatsit)?;
  // Fix for missing \bibitems!
  setup_pseudo_bibitem()
}

pub(crate) fn begin_bibliography_clean(whatsit: &mut Whatsit) -> Result<()> {
  // Check if \bibsection is defined and try to decipher it.
  // Expecting something like \section*{sometext}
  // Perl: beginBibliography_clean in latex_constructs.pool.ltxml
  let mut bibtitle: Option<Tokens> = None;
  if let Some(bs) = lookup_definition(&T_CS!("\\bibsection"))? {
    if bs.is_expandable() {
      if let Some(ExpansionBody::Tokens(expansion_toks)) = bs.get_expansion() {
        let mut tokens = expansion_toks.clone().unlist();
        if !tokens.is_empty() {
          let bibunitmap: &[(&str, &str)] = &[
            ("\\part", "ltx:part"),
            ("\\chapter", "ltx:chapter"),
            ("\\section", "ltx:section"),
            ("\\subsection", "ltx:subsection"),
            ("\\subsubsection", "ltx:subsubsection"),
            ("\\paragraph", "ltx:paragraph"),
            ("\\subparagraph", "ltx:subparagraph"),
          ];
          let first_cs = tokens.remove(0).to_string();
          if let Some((_, unit)) = bibunitmap.iter().find(|(cs, _)| *cs == first_cs) {
            state::assign_mapping(
              "BACKMATTER_ELEMENT",
              "ltx:bibliography",
              Some(arena::pin(unit)),
            );
            // Strip * if present
            if !tokens.is_empty() && tokens[0].text == pin!("*") {
              tokens.remove(0);
            }
            if !tokens.is_empty() {
              bibtitle = Some(Tokens::new(tokens));
            }
          }
        }
      }
    }
  }

  note_backmatter_element(whatsit, "ltx:bibliography");
  // Try to compute a reasonable, but unique ID;
  // relative to the document's ID, if any.
  // But also, if there are multiple bibliographies,
  let bibnumber = 1 + lookup_int("n_bibliographies");
  assign_value("n_bibliographies", bibnumber, Some(Scope::Global));
  let mut docid: String = Expand!(T_CS!("\\thedocument@ID")).to_string();
  if !docid.is_empty() {
    docid += ".";
  }
  let bibid = s!("{}bib{}", docid, radix::radix_alpha(bibnumber - 1));
  DefMacro!(T_CS!("\\thebibliography@ID"), None, T_OTHER!(&bibid), scope => Some(Scope::Global));
  whatsit.set_property("id", bibid);
  let title_opt = if let Some(bt) = bibtitle {
    Some(Digest!(bt)?)
  } else {
    match DigestIf!("\\refname")? {
      Some(v) => Some(v),
      None => DigestIf!("\\bibname")?,
    }
  };
  if let Some(title) = title_opt {
    if let Some(titlefont) = title.get_font()? {
      whatsit.set_property("titlefont", titlefont);
    }
    whatsit.set_property("title", title);
  }
  if let Some(bs) = lookup_value("BIBSTYLE") {
    whatsit.set_property("bibstyle", bs);
  }
  if let Some(cs) = lookup_value("CITE_STYLE") {
    whatsit.set_property("citestyle", cs);
  }
  // And prepare for the likely nonsense that appears within bibliographies
  ResetCounter!("enumiv");
  Ok(())
}

// Perl: $BIBSTYLES hash — maps bib style names to (citestyle, sort) pairs
fn lookup_bibstyle_params(style: &str) -> Option<(&'static str, &'static str)> {
  match style {
    "plain" => Some(("numbers", "true")),
    "unsrt" => Some(("numbers", "false")),
    "alpha" => Some(("AY", "true")),
    "abbrv" => Some(("numbers", "true")),
    "plainnat" => Some(("numbers", "true")),
    "unsrtnat" => Some(("numbers", "false")),
    "alphanat" => Some(("AY", "true")),
    "abbrvnat" => Some(("numbers", "true")),
    _ => None,
  }
}

// Perl: setBibstyle($style) — set BIBSTYLE, CITE_STYLE, CITE_SORT
pub fn set_bibstyle(style: &str) {
  assign_value("BIBSTYLE", arena::pin(style), None);
  if let Some((cs, so)) = lookup_bibstyle_params(style) {
    assign_value("CITE_STYLE", arena::pin(cs), None);
    assign_value("CITE_SORT", arena::pin(so), None);
  }
}

/// Perl: addIndexPhraseKey — sets the `key` attribute on index/glossary phrase
/// nodes from their text content, applying CleanIndexKey normalization.
fn add_index_phrase_key(node: &mut Node) -> Result<()> {
  if node.get_attribute("key").is_none() {
    let text = node.get_content();
    let key = clean_index_key(&text);
    if !key.is_empty() {
      node.set_attribute("key", &key)?;
    }
  }
  Ok(())
}
/// Perl: doIndexItem — open/close index list levels.
fn do_index_item(document: &mut Document, level: i64) -> Result<()> {
  if document.is_closeable("ltx:indexrefs").is_some() {
    document.close_element("ltx:indexrefs")?;
  }
  if document.is_closeable("ltx:indexphrase").is_some() {
    document.close_element("ltx:indexphrase")?;
  }
  let current_level = state::lookup_int("INDEXLEVEL");
  let mut l = current_level;
  while l < level {
    document.open_element("ltx:indexlist", None, None)?;
    l += 1;
  }
  while l > level {
    document.close_element("ltx:indexlist")?;
    l -= 1;
  }
  state::assign_value("INDEXLEVEL", Stored::Int(l), Some(Scope::Local));
  if level > 0 {
    document.open_element("ltx:indexentry", None, None)?;
    document.open_element("ltx:indexphrase", None, None)?;
  }
  Ok(())
}

/// Perl: CleanIndexKey — trim whitespace, remove trailing punctuation.
fn clean_index_key(key: &str) -> String {
  let key = key.trim();
  key.trim_end_matches(['.', ',', ';']).to_string()
}
/// Perl: process_index_phrases — expand \index{a!b@c|see{d}} into
/// \@index{\@indexphrase{a}\@indexphrase[c]{b}} etc.
///
/// Port of latex_constructs.pool.ltxml L4528-4591
fn process_index_phrases(tokens: Tokens) -> Result<Tokens> {
  let token_list = tokens.unlist();
  if token_list.is_empty() {
    return Ok(Tokens::new(vec![]));
  }
  // Add terminal ! if not present
  let mut toks = token_list;
  if toks
    .last()
    .map(|t| t.with_str(|s| s != "!"))
    .unwrap_or(true)
  {
    toks.push(T_OTHER!("!"));
  }
  let mut expansion: Vec<Token> = Vec::new();
  let mut phrase: Vec<Token> = Vec::new();
  let mut sortas: Vec<Token> = Vec::new();
  let mut style: Option<String> = None;
  let mut i = 0;
  while i < toks.len() {
    let tok = toks[i];
    let s = tok.with_str(|s| s.to_string());
    i += 1;
    if s == "\"" && i < toks.len() {
      // Escaped character: take next token literally
      phrase.push(toks[i]);
      i += 1;
    } else if s == "@" {
      // Sort key: everything before @ is the sort key
      while phrase
        .last()
        .map(|t| t.with_str(|s| s.trim().is_empty()))
        .unwrap_or(false)
      {
        phrase.pop();
      }
      sortas = phrase;
      phrase = Vec::new();
    } else if s == "!" || s == "|" {
      // End of phrase
      while phrase
        .last()
        .map(|t| t.with_str(|s| s.trim().is_empty()))
        .unwrap_or(false)
      {
        phrase.pop();
      }
      if !phrase.is_empty() {
        expansion.push(T_CS!("\\@indexphrase"));
        if !sortas.is_empty() {
          expansion.push(T_OTHER!("["));
          expansion.append(&mut sortas);
          expansion.push(T_OTHER!("]"));
        }
        expansion.push(T_BEGIN!());
        expansion.append(&mut phrase);
        expansion.push(T_END!());
      }
      sortas.clear();
      if s == "|" {
        // Collect remaining tokens as style/see/seealso
        if i < toks.len()
          && toks
            .last()
            .map(|t| t.with_str(|s| s == "!"))
            .unwrap_or(false)
        {
          // Remove terminal ! stopbit
          toks.pop();
        }
        let extra: String = toks[i..]
          .iter()
          .map(|t| t.with_str(|s| s.to_string()))
          .collect();
        if extra.starts_with("see{") || extra.starts_with("see {") {
          // \@indexsee{content}
          // Skip "see{", collect until "}"
          expansion.push(T_CS!("\\@indexsee"));
          // Find the content between { and }
          let content = extra.trim_start_matches("see").trim();
          let content = content.strip_prefix('{').unwrap_or(content);
          let content = content.strip_suffix('}').unwrap_or(content);
          expansion.push(T_BEGIN!());
          expansion.extend(Explode!(content));
          expansion.push(T_END!());
        } else if extra.starts_with("seealso{") || extra.starts_with("seealso {") {
          expansion.push(T_CS!("\\@indexseealso"));
          let content = extra.trim_start_matches("seealso").trim();
          let content = content.strip_prefix('{').unwrap_or(content);
          let content = content.strip_suffix('}').unwrap_or(content);
          expansion.push(T_BEGIN!());
          expansion.extend(Explode!(content));
          expansion.push(T_END!());
        } else if extra == "(" {
          style = Some("rangestart".to_string());
        } else if extra == ")" {
          style = Some("rangeend".to_string());
        } else if !extra.is_empty() {
          // Style name (e.g., textbf → bold)
          style = Some(match extra.as_str() {
            "textbf" | "bf" => "bold".to_string(),
            "textit" | "it" | "emph" => "italic".to_string(),
            "textrm" | "rm" => String::new(),
            other => other.to_string(),
          });
        }
        break; // Consumed everything after |
      }
    } else if phrase.is_empty() && s.trim().is_empty() {
      // Skip leading whitespace
    } else {
      phrase.push(tok);
    }
  }
  // Wrap in \@index[style]{...}
  let mut result = vec![T_BEGIN!(), T_CS!("\\normalfont"), T_CS!("\\@index")];
  if let Some(ref sty) = style {
    if !sty.is_empty() {
      result.push(T_OTHER!("["));
      result.extend(Explode!(sty));
      result.push(T_OTHER!("]"));
    }
  }
  result.push(T_BEGIN!());
  result.extend(expansion);
  result.push(T_END!());
  result.push(T_END!());
  Ok(Tokens::new(result))
}

/// Convert TeX points to CSS pixels using DPI setting (default 100).
/// Perl: $$self[0] / 65536 * DPI / 72.27
fn px_value(pt: f64) -> f64 {
  // DPI default is 100 in LaTeXML (state::lookupValue('DPI') || 100)
  let dpi = state::lookup_value("DPI")
    .and_then(|v| {
      if let Stored::Number(n) = v {
        Some(n.0 as f64)
      } else {
        None
      }
    })
    .unwrap_or(100.0);
  // Round to 2 decimal places (Perl default precision)
  (pt * dpi / 72.27 * 100.0).round() / 100.0
}
/// Format a px value, dropping trailing ".0" for integers
fn fmt_px(v: f64) -> String {
  if v == v.round() && v.abs() < 1e10 {
    format!("{}", v as i64)
  } else {
    format!("{v}")
  }
}

/// Perl: %unicode_enclosed_alphanumerics table
/// Maps single chars (0-9, a-z, A-Z) and numbers 10-20 to their circled Unicode equivalents.
fn unicode_enclosed_alphanumeric(text: &str) -> Option<String> {
  let ch = match text {
    "0" => '\u{24EA}',
    "1" => '\u{2460}',
    "2" => '\u{2461}',
    "3" => '\u{2462}',
    "4" => '\u{2463}',
    "5" => '\u{2464}',
    "6" => '\u{2465}',
    "7" => '\u{2466}',
    "8" => '\u{2467}',
    "9" => '\u{2468}',
    "10" => '\u{2469}',
    "11" => '\u{246A}',
    "12" => '\u{246B}',
    "13" => '\u{246C}',
    "14" => '\u{246D}',
    "15" => '\u{246E}',
    "16" => '\u{246F}',
    "17" => '\u{2470}',
    "18" => '\u{2471}',
    "19" => '\u{2472}',
    "20" => '\u{2473}',
    "a" => '\u{24D0}',
    "b" => '\u{24D1}',
    "c" => '\u{24D2}',
    "d" => '\u{24D3}',
    "e" => '\u{24D4}',
    "f" => '\u{24D5}',
    "g" => '\u{24D6}',
    "h" => '\u{24D7}',
    "i" => '\u{24D8}',
    "j" => '\u{24D9}',
    "k" => '\u{24DA}',
    "l" => '\u{24DB}',
    "m" => '\u{24DC}',
    "n" => '\u{24DD}',
    "o" => '\u{24DE}',
    "p" => '\u{24DF}',
    "q" => '\u{24E0}',
    "r" => '\u{24E1}',
    "s" => '\u{24E2}',
    "t" => '\u{24E3}',
    "u" => '\u{24E4}',
    "v" => '\u{24E5}',
    "w" => '\u{24E6}',
    "x" => '\u{24E7}',
    "y" => '\u{24E8}',
    "z" => '\u{24E9}',
    "A" => '\u{24B6}',
    "B" => '\u{24B7}',
    "C" => '\u{24B8}',
    "D" => '\u{24B9}',
    "E" => '\u{24BA}',
    "F" => '\u{24BB}',
    "G" => '\u{24BC}',
    "H" => '\u{24BD}',
    "I" => '\u{24BE}',
    "J" => '\u{24BF}',
    "K" => '\u{24C0}',
    "L" => '\u{24C1}',
    "M" => '\u{24C2}',
    "N" => '\u{24C3}',
    "O" => '\u{24C4}',
    "P" => '\u{24C5}',
    "Q" => '\u{24C6}',
    "R" => '\u{24C7}',
    "S" => '\u{24C8}',
    "T" => '\u{24C9}',
    "U" => '\u{24CA}',
    "V" => '\u{24CB}',
    "W" => '\u{24CC}',
    "X" => '\u{24CD}',
    "Y" => '\u{24CE}',
    "Z" => '\u{24CF}',
    _ => return None,
  };
  Some(ch.to_string())
}

#[rustfmt::skip]
LoadDefinitions!({

  // Perl `latex_constructs.pool.ltxml` L19-38 — force-reload of
  // `plain_constructs` and `math_common`. By the time
  // `latex_constructs` runs, both pools were already loaded during the
  // plain-format chain (`tex.rs::LoadFormat('plain')`), and several of
  // their definitions have since been clobbered by `latex_base` and
  // earlier `latex_constructs` activity. Perl explicitly clears the
  // `_loaded` flags and re-runs `LoadPool('plain_constructs')` (L21)
  // followed by `LoadPool('math_common')` (L38) to re-establish those
  // pools' definitions on top of LaTeX-side changes.
  //
  // Rust note: since commit `8dfcb12f7`, `InnerPool!(...)` honors
  // `<name>.pool_loaded` (mirror of Perl `LoadPool`'s
  // `<name>.pool.ltxml_loaded` guard, with the Rust suffix
  // convention). The two `assign_value(... Stored::None)` resets
  // below are therefore load-bearing — without them, `InnerPool!`
  // would skip the re-run.
  //
  // Perl interleaves a handful of defs (font reset, `\hline`,
  // `\f@encoding`, `\par→\lx@normal@par`, etc.) between L21 and L38;
  // Rust collapses both reloads here at the top because the
  // intervening defs are positioned later in this file (or in
  // `plain_constructs.rs`) and are agnostic to whether `math_common`
  // is reloaded before or after them.
  state::assign_value(
    "plain_constructs.pool_loaded",
    latexml_core::common::store::Stored::None,
    Some(latexml_core::state::Scope::Global),
  );
  state::assign_value(
    "math_common.pool_loaded",
    latexml_core::common::store::Stored::None,
    Some(latexml_core::state::Scope::Global),
  );
  // The reloads MUST run with state unlocked. By the time we get here,
  // the first plain-format pass has already locked common math CSes
  // (e.g. `\prime`, `\active@math@prime`) via their `locked => true`
  // DefMath/DefMacro entries. Without an unlocked frame, the second
  // pass sees `\prime:locked` and silently drops the redefinition,
  // leaving the dump-loaded `\mathchardef\prime="0230` mathchar in
  // place — which renders as digit `0` (char 0x30 in fam 2) instead
  // of U+2032 ′. Mirror Perl's LoadPool flow which reloads via the
  // top-level binding scope where re-locks are allowed.
  latexml_core::state::local_state_unlocked(true);
  InnerPool!(plain_constructs);
  InnerPool!(math_common);
  latexml_core::state::expire_state_unlocked();

  // ======================================================================
  // C.1 Commands and Environments
  // ======================================================================


  // Apparently LaTeX does NOT define \magnification,
  // and babel uses that to determine whether we're runing LaTeX!!!
  Let!("\\magnification", "\\@undefined");
  Let!("\\@empty", "\\lx@empty");
  Let!("\\@ifundefined", "\\lx@ifundefined");
  //**********************************************************************
  // Basic \documentclass & \documentstyle

  DefConditional!("\\if@compatibility", { lookup_bool("2.09_COMPATIBILITY") });
  DefMacro!("\\@compatibilitytrue", "");
  DefMacro!("\\@compatibilityfalse", "");

  Let!("\\@currentlabel", "\\@empty");
  DefMacro!("\\@currdir", "./");

  // Let's try just starting with this set (since we've loaded LaTeX)
  AssignValue!("inPreamble", true); // \begin{document} will clear this.

  DefConstructor!("\\documentclass OptionalSemiverbatim SkipSpaces Semiverbatim []",
                  "<?latexml class='#2' ?#1(options='#1')?>",
    after_digest => sub[whatsit] {
      let options: Option<&Digested> = whatsit.get_arg(1);
      let class_opts = match options {
        Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(ToString::to_string).collect(),
        None => Vec::new(),
      };
      load_class(&(whatsit.get_arg(2).unwrap().to_string()),
                class_opts,
                Tokens!(T_CS!("\\AtBeginDocument"), T_CS!("\\warn@unusedclassoptions")))
  });

  AssignValue!("@unusedoptionlist", Stored::Strings(Rc::new([])));
  DefPrimitive!("\\warn@unusedclassoptions", {
    if let Some(Stored::Strings(unused)) = lookup_value("@unusedoptionlist") {
      if !unused.is_empty() {
        Info!(
          "unexpected",
          "options",
          "Unused global options: {}",
          arena::with_many(&unused, |u| u.join(","))
        );
        state::assign_value("@unusedoptionlist", Stored::Strings(Rc::new([])), None);
      }
    }
  });

  // Perl latex_constructs.pool.ltxml:137-154:
  //   DefPrimitiveI('\compat@loadpackages', undef, sub {
  //       my $hadmissing = 0;
  //       foreach my $option (@{ LookupValue('@unusedoptionlist') }) {
  //         if (FindFile($option, type => 'sty')) { RequirePackage($option); }
  //         else { $hadmissing = 1; Info('unexpected', $option, ...); } }
  //       if ($hadmissing && !LookupValue('OmniBus.cls_loaded')) {
  //         Info('note', 'OmniBus', ...); LoadClass('OmniBus'); }
  //       AssignValue('@unusedoptionlist', []); });
  //
  // Scheduled via `after => Tokens(T_CS('\compat@loadpackages'))` when the
  // LaTeX-2.09-compat \documentstyle finishes its class load. Consumes the
  // unused options that the class (e.g. article.cls) didn't recognise and
  // routes each to \RequirePackage via Rust's `require_package` sub — which
  // includes `find_file_fallback` (version-suffix stripping, e.g.
  // `aaspp4` → `aaspp.sty.ltxml`). This is what lets
  // `\documentstyle[aaspp4]{article}` load aas_support transitively and
  // define \affil / \altaffilmark / \acknowledgments etc. that ~49 astro-ph
  // papers in the 10k sandbox need (docs/SANDBOX_TRIAGE.md Class A).
  //
  // TODO(next cycle): once \documentstyle is converted to a DefConstructor
  // with afterDigest (mirroring Perl's dispatch — .sty/.cls/OmniBus
  // branching), this primitive becomes the sole route for "unused option →
  // package". Currently \documentstyle's tex_job.rs DefMacro still emits
  // `\RequirePackage` tokens inline; that will be removed when the
  // DefConstructor lands.
  DefPrimitive!("\\compat@loadpackages", {
    use latexml_core::binding::content::{find_file, find_file_fallback};
    let unused_list: Vec<String> = match state::lookup_value("@unusedoptionlist") {
      Some(Stored::Strings(rc)) => {
        rc.iter()
          .map(|s| latexml_core::common::arena::with(*s, |s| s.to_string()))
          .collect()
      },
      _ => Vec::new(),
    };
    let mut had_missing = false;
    for opt in &unused_list {
      let found = find_file(&format!("{opt}.sty"), None).is_some()
        || find_file_fallback(opt, "sty").is_some();
      if found {
        require_package(opt, RequireOptions::default())?;
      } else {
        had_missing = true;
        Info!("unexpected", opt, "Unexpected option '{}' passed via \\documentstyle", opt);
      }
    }
    if had_missing && !state::lookup_bool("OmniBus.cls_loaded") {
      Info!("note", "OmniBus", "Loading OmniBus class to attempt to cover missing options");
      load_class("OmniBus", Vec::new(), Tokens!())?;
    }
    state::assign_value(
      "@unusedoptionlist",
      Stored::Strings(std::rc::Rc::new([])),
      Some(Scope::Global),
    );
  });

  // onlyPreamble (Perl helper) — flag-only; the actual Error emission when
  // used outside preamble is a future polish (the mis-use cascade already
  // surfaces downstream Errors today).


  AssignValue!("current_environment", String::new(), Some(Scope::Global));
  DefMacro!("\\@currenvir", "");
  // Note: LaTeX kernel defines \def\f#1{\def\@currenvir{#1}} but this is just
  // a kernel internal that gets overridden by user \newcommand{\f}{...}.
  // We do NOT define \f here — use \lx@setcurrenvir instead (matching Perl).
  // The old DefPrimitive!("\\f{}", ...) was a bug: primitives can't be overridden
  // by \newcommand, so \newcommand{\f}{\mathcal{F}} would silently fail, and
  // $\f$ would eat the closing $ as an argument, corrupting the mode stack.

  DefPrimitive!(
  "\\lx@setcurrenvir{}", sub[(env)] {
    let env_string = env.to_string();
    DefMacro!(T_CS!("\\@currenvir"), None, env);
    AssignValue!("current_environment", env_string);
  });
  Let!("\\@currenvline", "\\@empty");

  // Perl: latex_constructs.pool.ltxml line 190
  DefMacro!("\\@checkend{}", r"\def\reserved@a{#1}\ifx\reserved@a\@currenvir \else\@badend{#1}\fi}");

  DefMacro!("\\begin{}", sub[(env)] {
    let name = Expand!(env.clone()).to_string();
    let begin_name = format!("\\begin{{{name}}}");
    let before_opt = state::lookup_tokens(&format!("@environment@{name}@beforebegin"));
    let after_opt  = state::lookup_tokens(&format!("@environment@{name}@atbegin"));

    if is_defined(&begin_name) {
      let mut tks = before_opt.map(Tokens::unlist).unwrap_or_default();
      tks.push(T_CS!(begin_name));
      Ok(Tokens::new(tks)) // Magic cs!
    } else {
      let token = T_CS!(format!("\\{name}"));
      if !is_defined_token(&token) {
        // this creates {name} , {{ and }} are escapes in Rust's `format` macro
        let undef = format!("{{{name}}}");
        let message = s!("The environment {} is not defined.", undef);
        Error!("undefined", undef, message);
        note_status(LogStatus::Undefined, Some(&undef));
        // TODO:
        // state::install_definition(LaTeXML::Core::Definition::Constructor->new($token, undef,
        //       sub { LaTeXML::Core::Stomach::makeError($_[0], "undefined", $undef); })); }
      }
      let mut out_tokens = before_opt.map(Tokens::unlist).unwrap_or_default();
      out_tokens.push(T_CS!("\\begingroup"));
      if let Some(after) = after_opt {
        out_tokens.extend(after.unlist());
      }
      out_tokens.extend(Invocation!(T_CS!("\\lx@setcurrenvir"), vec![env]).unlist());
      out_tokens.push(token);
      Ok(Tokens::new(out_tokens))
    }
  });

  DefMacro!("\\end {}", sub[(env)]{
    let name = Expand!(env).to_string();
    let before = state::lookup_tokens(&s!("@environment@{name}@atend"));
    let after = state::lookup_tokens(&s!("@environment@{name}@afterend"));
    let mut t = T_CS!(s!("\\end{{{name}}}"));
    let mut out_tokens = Vec::new();
    if is_defined_token(&t) {
      // Magic CS!
      out_tokens.push(t);
      if let Some(afterend_toks) = after {
        out_tokens.extend(afterend_toks.unlist())
      }
    } else {
      out_tokens = before.map(Tokens::unlist).unwrap_or_default();
      t = T_CS!(s!("\\end{name}"));
      if is_defined_token(&t) {
        out_tokens.push(t);
      }
      out_tokens.push(T_CS!("\\endgroup"));
      if let Some(afterend_toks) = after {
        out_tokens.extend(afterend_toks.unlist())
      }
    }
    Ok(Tokens::new(out_tokens))
  });


  TeX!(
    r"
\def\@ignorefalse{\global\let\if@ignore\iffalse}
\def\@ignoretrue {\global\let\if@ignore\iftrue}
\def\zap@space#1 #2{%
  #1%
  \ifx#2\@empty\else\expandafter\zap@space\fi
  #2}
\def\@unexpandable@protect{\noexpand\protect\noexpand}
\def\x@protect#1{%
   \ifx\protect\@typeset@protect\else
      \@x@protect#1%
   \fi
}
\def\@x@protect#1\fi#2#3{%
   \fi\protect#1%
}
\let\@typeset@protect\relax
\def\set@display@protect{\let\protect\string}
\def\set@typeset@protect{\let\protect\@typeset@protect}
\def\protected@edef{%
   \let\@@protect\protect
   \let\protect\@unexpandable@protect
   \afterassignment\restore@protect
   \edef
}
\def\protected@xdef{%
   \let\@@protect\protect
   \let\protect\@unexpandable@protect
   \afterassignment\restore@protect
   \xdef
}
\def\unrestored@protected@xdef{%
   \let\protect\@unexpandable@protect
   \xdef
}
\def\restore@protect{\let\protect\@@protect}
\set@typeset@protect
\def\@nobreakfalse{\global\let\if@nobreak\iffalse}
\def\@nobreaktrue {\global\let\if@nobreak\iftrue}
\@nobreakfalse

\newif\ifv@
\newif\ifh@
\newif\ifdt@p
\newif\if@pboxsw
\newif\if@rjfield
\newif\if@firstamp
\newif\if@negarg
\newif\if@ovt
\newif\if@ovb
\newif\if@ovl
\newif\if@ovr
\newdimen\@ovxx
\newdimen\@ovyy
\newdimen\@ovdx
\newdimen\@ovdy
\newdimen\@ovro
\newdimen\@ovri
\newif\if@noskipsec \@noskipsectrue
"
  );


  //======================================================================
  // C.1.4 Declarations
  //======================================================================
  // actual implementation later.
  //======================================================================
  // C.1.5 Invisible Commands
  //======================================================================
  // actual implementation later.

  //======================================================================
  // C.1.6 The \\ Command
  //======================================================================
  // In math, \\ is just a formatting hint, unless within an array, cases, .. environment.
  // Perl: DefConstructor('\lx@newline OptionalMatch:* [Glue]', sub { ... });
  // Complex constructor that checks document context:
  //   - in math: insert <ltx:XMHint name='newline'/>
  //   - no context or _CaptureBlock_: skip
  //   - ltx:p with parent _CaptureBlock_: maybeCloseElement('ltx:p')
  //   - can contain ltx:break: insert <ltx:break/>
  DefConstructor!("\\lx@newline OptionalMatch:* [Glue]", sub[document] {
    if state::lookup_bool_sym(pin!("IN_MATH")) {
      document.insert_element("ltx:XMHint", Vec::new(), Some(map!("name" => s!("newline"))))?;
    } else {
      if let Some(context) = document.get_element() {
        let tag = document::get_node_qname(&context);
        let capture_block = arena::pin_static("ltx:_CaptureBlock_");
        if tag == capture_block {
          // skip, if in insertBlock
        } else if tag == arena::pin_static("ltx:p") {
          // Close <p> if parent is _CaptureBlock_
          if let Some(parent) = context.get_parent() {
            if document::get_node_qname(&parent) == capture_block {
              document.maybe_close_element("ltx:p")?;
            } else if document::can_contain(&context, "ltx:break") {
              document.insert_element("ltx:break", Vec::new(), None)?;
            }
          }
        } else if document::can_contain(&context, "ltx:break") {
          document.insert_element("ltx:break", Vec::new(), None)?;
        }
      }
      // else: no context => skip
    }
  },
    reversion => Tokens!(T_CS!("\\\\"), T_CR!()),
    properties => { stored_map!("isBreak" => true) },
  );
  Let!("\\\\", "\\lx@newline");

  DefConstructor!("\\newline", "?#isMath(<ltx:XMHint name='newline'/>)(<ltx:break/>)",
    reversion  => Tokens!(T_CS!("\\newline"), T_CR!()),
    properties => { Ok(stored_map!("isBreak" => true)) },
  );

  Let!("\\@normalcr", "\\\\");
  Let!("\\@normalnewline", "\\newline");
  // NOTE: Activating this binding messes up an \afterassign test,
  //       so it may be best left disabled.
  // PushValue!("TEXT_MODE_BINDINGS" => Tokens!(T_CS!("\\\\"), T_CS!("\\@normalcr")));

  DefMacro!("\\@nolnerr", "");
  DefMacro!(
    "\\@centercr",
    r"\ifhmode\unskip\else\@nolnerr\fi\par\@ifstar{\nobreak\@xcentercr}\@xcentercr"
  );
  DefMacro!(
    "\\@xcentercr",
    r"\addvspace{-\parskip}\@ifnextchar[\@icentercr\ignorespaces"
  );
  DefMacro!("\\@icentercr[]", "\\vskip #1\\ignorespaces");


  // ======================================================================
  // C.2 The Structure of the Document
  // ======================================================================


  //**********************************************************************
  // C.2. The Structure of the Document
  //**********************************************************************
  //   prepended files (using filecontents environment)
  //   preamble (starting with \documentclass)
  //   \begin{document}
  //    text
  //   \end{document}

  // Perl: PushValue('@at@begin@document', $_[1]->unlist)
  // Note: in modern LaTeX with expl3, \AtBeginDocument is redefined to use
  // the L3 hook system (\AddToHook{begindocument}{...}). Our definition here
  // serves as a fallback when expl3 isn't loaded. When expl3 IS loaded, it
  // overrides this with its own version that routes through \hook_gput_code:nnn.
  // Perl 93f875a6: support optional [label] from modern LaTeX hooks system
  DefMacro!("\\AtBeginDocument[]{}", sub[(_label, rules)] {
    state::push_value("@at@begin@document", rules)
  });
  DefMacro!("\\AtEndDocument[]{}", sub[(_label, rules)] {
    state::push_value("@at@end@document", rules)
  });

  // Like  "<ltx:document xml:id='#id'>#body</ltx:document>",
  // But more complicated due to id, at begin/end document and so forth.
  // AND, lower-level so that we can cope with common errors at document end.
  DefConstructor!(T_CS!("\\begin{document}"), None, sub[document, _args, props] {
    let id = prop_str!(props,"id");
    // Already (auto) created?
    if let Some(mut docel) = document.findnode("/ltx:document", None) {
      if id != pin!("") {
        let id_s = arena::with(id, |s| s.to_string());
        document.set_attribute(&mut docel, "xml:id", &id_s)?;
      }
    } else {
      let props = arena::with(id, |id_str| string_map!("xml:id" => id_str));
      document.open_element("ltx:document", Some(props), None)?;
    }
  },
  after_digest => sub[whatsit] {
    // Perl: beginMode('internal_vertical', 1) — noframe=1
    // Begin internal_vertical mode WITHOUT pushing a stack frame, keeping level=0
    begin_mode_opt("internal_vertical", true)?;
    // we need to re-bind in order to nest calls to the binding macro machinery
    DefMacro!("\\@currenvir", "document");
    state::assign_value("current_environment", "document", None);
    let expanded_id = Expand!(T_CS!("\\thedocument@ID"));
    whatsit.set_property("id", expanded_id);
    Let!("\\@nodocument", "\\relax", Scope::Global);
    // Clear \everypar at document start (Perl parity)
    state::assign_value("\\everypar", Tokens!(), Some(Scope::Global));
    let mut boxes = Vec::new();
    if let Some(ops) = state::lookup_tokens("@document@preamble@atend") {
      boxes.push(stomach::digest(ops)?);
    }
    if let Some(ops) = state::lookup_tokens("@at@begin@document") {
      boxes.push(stomach::digest(ops)?);
    }
    // Fire the L3 hook system for begindocument.
    // Modern LaTeX (with expl3) uses \hook_use:n{begindocument} instead of
    // \@begindocumenthook. This fires hooks registered via \AtBeginDocument
    // when expl3 has redefined it to use \AddToHook{begindocument}{...}.
    // Includes babel's \lx@babel@activate@mainlang.
    //
    // NOTE: this is a Rust-only deviation from Perl (Perl does not fire a
    // begindocument hook dispatch), but it's load-bearing because our raw
    // expl3-code.tex load path *does* define `\hook_use:n` and enqueues
    // real hook code against it. Keep until the kernel-parity direction
    // either (a) stops loading raw expl3-code.tex, or (b) ports l3hooks
    // natively with storage. See SYNC_STATUS.md "l3hooks parity".
    if lookup_definition(&T_CS!("\\hook_use:n"))?.is_some() {
      // Build the Tokens explicitly: `Tokenize!` runs at the runtime
      // catcode regime where `:` is OTHER (not LETTER), which would
      // truncate the CS to `\hook_use` and emit `:n` as plain text.
      // That leaks `_use:n` + arg-text into the document body.
      boxes.push(stomach::digest(Tokens!(
        T_CS!("\\hook_use:n"),
        T_BEGIN!(),
        T_LETTER!("b"),
        T_LETTER!("e"),
        T_LETTER!("g"),
        T_LETTER!("i"),
        T_LETTER!("n"),
        T_LETTER!("d"),
        T_LETTER!("o"),
        T_LETTER!("c"),
        T_LETTER!("u"),
        T_LETTER!("m"),
        T_LETTER!("e"),
        T_LETTER!("n"),
        T_LETTER!("t"),
        T_END!()
      ))?);
    }
    // Preamble cleanup: force `\ExplSyntaxOff` if `_` is still LETTER at
    // document start. Mirrors LaTeX2e kernel's preamble cleanup (latex.ltx
    // L7122 `\bool_if:NTF \l__kernel_expl_bool { \ExplSyntaxOff } ...`) —
    // packages like mhchem.sty end with an unmatched final `\ExplSyntaxOn`
    // (see mhchem.sty tail, "legacy" block), and LaTeX's kernel relies on
    // this scheduled cleanup to restore catcodes before the document body.
    // Without this, `\sum_{...}` tokenizes as the CS `\sum_` (letter `_`)
    // rather than `\sum` + `_` + `{...}`.
    if state::lookup_catcode('_') == Some(Catcode::LETTER)
      && lookup_definition(&T_CS!("\\ExplSyntaxOff"))?.is_some()
    {
      boxes.push(stomach::digest(Tokens!(T_CS!("\\ExplSyntaxOff")))?);
    }
    // Fire babel language activation AFTER all hooks (including babel's own
    // \selectlanguage call). This runs even if babel's hook code has errors.
    // Use T_CS! directly since @ is OTHER catcode at \begin{document} time.
    if lookup_definition(&T_CS!("\\lx@babel@activate@mainlang"))?.is_some() {
      boxes.push(stomach::digest(Tokens!(T_CS!("\\lx@babel@activate@mainlang")))?);
    }
    state::assign_value("inPreamble", false, None); // atbegin is still (sorta) preamble
    if let Some(ops) = state::lookup_tokens("@document@preamble@afterend") {
      boxes.push(stomach::digest(ops)?);
    }
    whatsit.set_font(lookup_font().unwrap()); // Start w/ whatever font was last selected.
    leave_horizontal_internal();
    boxes
  });

  // \document is used directly in e.g. expl3.sty
  Let!("\\document", "\\begin{document}", Scope::Global);

  DefConstructor!(T_CS!("\\end{document}"), None, sub[document,_args,_props] {
      document.close_element("ltx:document")?;
    },
    before_digest => {
      let mut boxes : Vec<Digested> = Vec::new();
      if let Some(ops) = state::lookup_tokens("@at@end@document") {
        boxes.push(stomach::digest(ops)?);
      }
      // Should we try to indent the last paragraph? If so, it goes like this:
      boxes.push(stomach::digest(T_CS!("\\lx@normal@par"))?);
      // Pop unclosed groups and environments back to the document frame
      // so endMode's strict BOUND_MODE check sees the right frame at the
      // top. Mirrors Perl latex_constructs.pool.ltxml L350-374. Without
      // this loop, papers with a dangling `\begingroup` inside the body
      // (e.g. `\providecommand{\href}[2]{#2}\begingroup\raggedright
      // \begin{thebibliography}{99}`) trigger
      // "Attempt to end mode `internal_vertical` in `internal_vertical`"
      // because the top frame is the dangling group, not the document.
      // Note: Rust port omits Perl's if_stack handling — Rust's gullet
      // does not maintain an explicit if_stack value.
      let top_is_document = state::is_value_bound("current_environment", Some(0))
        && state::lookup_string("current_environment") == "document";
      if !top_is_document {
        let mut popped_lines: Vec<String> = Vec::new();
        while !(state::is_value_bound("current_environment", Some(0))
          && state::lookup_string("current_environment") == "document")
          && get_frame_depth() > 0
        {
          let initiator = state::lookup_string("groupInitiator");
          let initiator = if initiator.is_empty() {
            "<unknown>".to_string()
          } else {
            initiator
          };
          let env_bound = state::is_value_bound("current_environment", Some(0));
          let env_name = if env_bound {
            state::lookup_string("current_environment")
          } else {
            String::new()
          };
          if !env_name.is_empty() {
            popped_lines.push(s!("Environment {env_name} opened by {initiator}"));
          } else {
            popped_lines.push(s!("Group opened by {initiator}"));
          }
          state::pop_frame()?;
        }
        let detail = if popped_lines.is_empty() {
          String::new()
        } else {
          s!("\n{}", popped_lines.join("\n"))
        };
        Warn!(
          "unexpected",
          "\\end{document}",
          s!(
            "Attempt to end document with open groups, environments or conditionals{detail}"
          )
        );
      }
      // Perl: endMode('internal_vertical', 1) — noframe=1
      // End mode without popping stack frame (executes beforeAfterGroup)
      end_mode_opt("internal_vertical", true)?;
      gullet::flush();
      boxes
  });

  // \enddocument is used directly in e.g. standalone.cls
  Let!("\\enddocument", "\\end{document}", Scope::Global);


  // ======================================================================
  // C.3 Sentences and Paragraphs
  // ======================================================================


  //======================================================================
  // C.3.1 Making Sentences
  //======================================================================
  // quotes;  should these be handled in DOM/construction?
  // dashes:  We'll need some sort of Ligature analog, or something like
  // Omega's OTP, to combine sequences of "-" into endash, emdash,
  // Perhaps it also applies more semantically?
  // Such as interpreting certain sequences as section headings,
  // or math constructs.

  // Spacing; in TeX.pool.ltxml

  // Special Characters; in TeX.pool.ltxml

  // Logos
  // \TeX is in TeX.pool.ltxml
  DefMacro!("\\LaTeX", "LaTeX");
  DefMacro!("\\LaTeXe", "LaTeX2e");
  // Perl: enterHorizontal => 1
  DefConstructor!("\\LaTeX","<ltx:text class='ltx_LaTeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.1em'
  >L<ltx:text cssstyle='font-variant:small-caps;' yoffset='0.4ex'
  >a</ltx:text
  >T<ltx:text cssstyle='font-variant:small-caps;font-size:120%' yoffset='-0.2ex'
  >e</ltx:text
  >X</ltx:text>",
  enter_horizontal => true,
  sizer => { Ok((Dimension!("2.6em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  // Perl: enterHorizontal => 1
  DefConstructor!("\\LaTeXe","<ltx:text class='ltx_LaTeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.1em'
  >L<ltx:text cssstyle='font-variant:small-caps;' yoffset='0.4ex'
  >a</ltx:text
  >T<ltx:text cssstyle='font-variant:small-caps;font-size:120%' yoffset='-0.2ex'
  >e</ltx:text
  >X\u{2002}2<ltx:text cssstyle='font-style:italic' yoffset='-0.3ex'
  >\u{03B5}</ltx:text></ltx:text>",
  enter_horizontal => true,
  sizer => { Ok((Dimension!("3.7em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  DefMacro!("\\fmtname", "LaTeX2e");
  DefMacro!("\\fmtversion", "2018/12/01");

  DefMacro!("\\today", { ExplodeText!(Today!()) });

  // Use fonts (w/ special flag) to propogate emphasis as a font change,
  // but preserve it's "emph"-ness.
  // Perl latex_constructs.pool.ltxml L401-408: mode => 'restricted_horizontal',
  //   enterHorizontal => 1, font => { emph => 1 }, alias => '\emph', beforeDigest => {...}.
  DefConstructor!("\\emph{}", "<ltx:emph _force_font='1'>#1",
    mode => "text",
    bounded        => true,
    enter_horizontal => true,
    font=> { emph => true },
    alias => "\\emph",
    before_digest => {
      if Expand!(T_CS!("\\f@shape")).eq_text("it") {
        DefMacro!(T_CS!("\\f@shape"), None, Tokens!(T_LETTER!("n")));
      } else {
        DefMacro!(T_CS!("\\f@shape"), None, Tokens!(T_LETTER!("i"),T_LETTER!("t")));
      }
    },
    after_construct => sub[doc,_args] {
      doc.maybe_close_element("ltx:emph")?; }
  );

  //======================================================================
  // C.3.2 Making Paragraphs
  //======================================================================
  // \noindent, \indent, \par in TeX.pool.ltxml

  Let!("\\@@par", "\\par");
  DefMacro!("\\@par", r"\let\par\@@par\par");
  DefMacro!("\\@restorepar", r"\def\par{\@par}");

  // Style parameters
  // \parindent, \baselineskip, \parskip alreadin in TeX.pool.ltxml

  DefPrimitive!("\\linespread{}", None);

  // ?
  DefMacro!("\\@noligs", "");
  DefConditional!("\\if@endpe");
  DefMacro!("\\@doendpe", "");
  DefMacro!("\\@bsphack", "\\relax"); // what else?
  DefMacro!("\\@esphack", "\\relax");
  DefMacro!("\\@Esphack", "\\relax");

  //======================================================================
  // C.3.3 Footnotes
  //======================================================================

  NewCounter!("footnote");
  DefMacro!("\\thefootnote", "\\arabic{footnote}");
  NewCounter!("mpfootnote");
  DefMacro!("\\thempfn", "\\thefootnote");
  DefMacro!("\\thempfootnote", "\\arabic{mpfootnote}");
  DefMacro!("\\footnotetyperefname", "footnote");

  DefMacro!("\\ext@footnote", None);
  DefConstructor!("\\lx@note[]{}[]{}",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id' inlist='#list'>#tags#4</ltx:note>",
  mode         => "internal_vertical",
  before_digest => {
    neutralize_font(); },
  properties   => sub [args] {
    let arg1 = args[0].as_ref();
    let arg2 = args[1].as_ref();
    let arg3 = args[2].as_ref().map(Cow::Borrowed);
    let note_type = arg2.as_ref().map(ToString::to_string).unwrap_or_default();
    let mut props = make_note_tags(&note_type, arg1, arg3)?;
    props.insert("list", digest_text(Tokens!(T_CS!(s!("\\ext@{note_type}"))))?.into());
    props.insert("role", note_type.into());
    Ok(props)
  },
  reversion => "");

  DefConstructor!("\\lx@notemark[]{}[]",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id' inlist='#list'>#tags</ltx:note>",
  mode       => "text", enter_horizontal => true,
  properties => sub[args] {
    let arg1 = args[0].as_ref();
    let arg2 = args[1].as_ref();
    let arg3 = args[2].as_ref().map(Cow::Borrowed);
    let note_type = arg2.as_ref().map(ToString::to_string).unwrap_or_default();
    let mut props = make_note_tags(&note_type, arg1, arg3)?;
    props.insert("role", s!("{note_type}mark").into());
    props.insert("list", digest_text(Tokens!(T_CS!(s!("\\ext@{note_type}"))))?.into());
    Ok(props)
  },
  reversion => "");

  DefConstructor!("\\lx@notetext[]{}[]{}",
  "^<ltx:note role='#role' mark='#mark' xml:id='#id'>#4</ltx:note>",
  mode       => "internal_vertical",
  properties => sub [args] {
    let arg1 = args[0].as_ref();
    let arg2 = args[1].as_ref();
    let arg3 = args[2].as_ref();
    let note_type = arg2.as_ref().map(ToString::to_string).unwrap_or_default();
    let arg3_ready = if let Some(v) = arg3 { Cow::Borrowed(v) } else {
      Cow::Owned(
        stomach::digest(T_CS!(s!("\\the{note_type}")))?
      )
    };
    let mut props = make_note_tags(&note_type, arg1, Some(arg3_ready))?;
    props.insert("role", s!("{note_type}text").into());
    Ok(props)
  },
  reversion => "");

  DefMacro!("\\footnote",      "\\lx@note{footnote}",     locked => true);
  DefMacro!("\\footnotemark",  "\\lx@notemark{footnote}", locked => true);
  DefMacro!("\\footnotetext",  "\\lx@notetext{footnote}", locked => true);
  DefMacro!("\\@footnotetext", "\\lx@notetext{footnote}", locked => true);
  // we don't implement the internals directly, so lock them to the latexml variant
  Let!("\\@thefnmark", "\\lx@notemark{footnote}");

  Tag!("ltx:emph", auto_close => true);
  Tag!("ltx:note", after_close => sub[doc, node] { relocate_footnote(doc, node)?; });

  // Style parameters
  DefRegister!("\\footnotesep" => Dimension::new(0));
  DefPrimitive!("\\footnoterule", None);


  // ======================================================================
  // C.4 Sectioning and Table of Contents
  // ======================================================================


  //======================================================================
  // C.4.1 Sectioning Commands.
  //======================================================================
  // Note that LaTeX allows fairly arbitrary stuff in \the<ctr>, although
  // it can get you in trouble.  However, in almost all cases, the result
  // is plain text.  So, I'm putting refnum as an attribute, where I like it!
  // You want something else? Redefine!

  // Also, we're adding an id to each, that is parallel to the refnum, but
  // valid as an ID.  You can tune the representation by defining, eg. \thesection@ID

  // A little more messy than seems necessary:
  //  We don't know whether to step the counter and update \@currentlabel until we see the '*',
  // but we have to know it before we digest the title, since \label can be there!

  // These are defined in terms of \@startsection so that
  // casual user redefinitions work, too.
  DefMacro!("\\chapter", "\\@startsection{chapter}{0}{}{}{}{}", locked=>true);

  // not locked since sometimes redefined as partition?
  DefMacro!("\\part", "\\@startsection{part}{-1}{}{}{}{}");
  DefMacro!("\\section", "\\@startsection{section}{1}{}{}{}{}", locked=>true);
  DefMacro!("\\subsection", "\\@startsection{subsection}{2}{}{}{}{}", locked => true);
  DefMacro!(
    "\\subsubsection",
    "\\@startsection{subsubsection}{3}{}{}{}{}",
    locked => true);
  DefMacro!("\\paragraph", "\\@startsection{paragraph}{4}{}{}{}{}", locked => true);
  DefMacro!("\\subparagraph", "\\@startsection{subparagraph}{5}{}{}{}{}", locked => true);

  Tag!("ltx:part", auto_close=>true);
  Tag!("ltx:chapter", auto_close=>true);
  Tag!("ltx:section", auto_close=>true);
  Tag!("ltx:subsection", auto_close=>true);
  Tag!("ltx:subsubsection", auto_close=>true);
  Tag!("ltx:paragraph", auto_close=>true);
  Tag!("ltx:subparagraph", auto_close=>true);
  // Also auto-close structural/backmatter containers so papers that open
  // `\acknowledgments` / `\appendix` / `\index` without a matching `\end...`
  // (common in mn/jheppub/pos classes) don't leave the element open until
  // `\end{document}` and produce schema-violation errors when a following
  // bibliography or section is emitted.
  // Perl: ltx:bibliography already has autoClose=1 (latex_constructs L4078);
  // these siblings match its container-with-trailing-content semantics.
  Tag!("ltx:acknowledgements", auto_close => true);
  Tag!("ltx:appendix", auto_close => true);
  Tag!("ltx:index", auto_close => true);
  // NOTE: tried Tag!("ltx:itemize"/"ltx:enumerate"/"ltx:description",
  // auto_close=>true) to address schema errors like "ltx:bibitem in
  // <ltx:itemize>" from malformed user input (e.g. 0801.4271). That
  // BROKE the 10_expansion/partial test because itemize would close
  // immediately before items are added. Perl's L1337 only marks
  // `ltx:item` as autoClose/autoOpen — container remains
  // explicit-close-only. Leaving these alone for now.

  DefMacro!("\\secdef {}{} OptionalMatch:*", sub[(token1, token2, star)] {
    if star.is_some() {
      Ok(token2) // can't move out without clone, how to circumvent?
    } else {
      Ok(token1)
    }
  });

  DefMacro!("\\@startsection@hook", "");

  NewCounter!("secnumdepth");
  SetCounter!("secnumdepth", Number::new(3));
  DefMacro!(
    "\\@startsection{}{}{}{}{}{} OptionalMatch:*",
    sub[(type_tokens, level_arg, _ignore3, _ignore4, _ignore5, _ignore6, flag)] {
      // Aside: Guard mode
      // Never start sections in math mode -- this is a good recovery point for broken documents
      if state::lookup_bool_sym(pin!("IN_MATH")) {
        let mode = state::lookup_string_from_sym(pin!("MODE"));
        if mode.contains("math") { // double-check we're really in math
          end_mode(&mode)?;
        } else { // otherwise, just unset the flag?
          state::assign_value("IN_MATH", false, Some(Scope::Global));
        }
      }
      // Main logic — Perl's `$level > ...` coerces non-numeric to 0 via
      // implicit numeric context; match that with unwrap_or(0) rather than
      // panicking when a caller passes a surprising value.
      let level = level_arg.to_string();
      let level_int = level.trim().parse::<i64>().unwrap_or(0);
      let mut tokens: Vec<Token>;
      if flag.is_some() { // No number, not in TOC
        tokens = vec![
          T_CS!("\\par"), T_CS!("\\@startsection@hook"), T_CS!("\\@@unnumbered@section"),
        T_BEGIN!()];
        tokens.extend(type_tokens.unlist());
        tokens.extend(vec![T_END!(), T_BEGIN!(), T_END!()]);
      } else if level_int > CounterValue!("secnumdepth").value_of() ||
        lookup_bool("no_number_sections") {
        // No number, but in TOC
        tokens = vec![
          T_CS!("\\par"), T_CS!("\\@startsection@hook"), T_CS!("\\@@unnumbered@section"),
        T_BEGIN!()];
        tokens.extend(type_tokens.unlist());
        tokens.extend(vec![T_END!(), T_BEGIN!(), T_OTHER!("toc"), T_END!()]);
      } else { // Number and in TOC
        tokens = vec![T_CS!("\\par"), T_CS!("\\@startsection@hook"), T_CS!("\\@@numbered@section"),
        T_BEGIN!()];
        tokens.extend(type_tokens.unlist());
        tokens.extend(vec![T_END!(), T_BEGIN!(), T_OTHER!("toc"), T_END!()]);
      };
      Ok(Tokens::new(tokens))
    },
    locked => true
  );

  DefConstructor!(
    "\\@@numbered@section{} Undigested OptionalUndigested Undigested",
    sub[document, args, props] {
      // args:=(stype,inlist,toctitle,title)
      let stype = args[0].as_ref().unwrap().to_string();
      let inlist = args[1].as_ref().unwrap().to_string();
      // TODO: This bizarre argument API interaction needs to be simplified down to Perl's
      // intuitive level of:       let (x,y,z, ...) = @args;
      // If backmatter, find insertion point as if inserting the backmatter element type
      if let Some(asif) = props.get("backmatterelement") {
        let asif_str = asif.to_string();
        let point = document.find_insertion_point(&asif_str, None)?;
        document.set_node(&point);
      }
      let clean_id = prop_string!(props,"id"); // TODO: CleanID($id);
      let tagname = s!("ltx:{stype}");
      document.open_element(&tagname,
        Some(string_map!("xml:id" => clean_id, "inlist" => inlist)),
        None,
          )?;
      // TODO: Another instance where the immutability of props causes endless cloning
      //       which is slow and wasteful.
      // The big problem is that for props to be mutable, the entire parent whatsit needs to
      // be mutable, and Rust hits a mutability conflict between the parent, and the
      // "args" and "props" children ... will come back here after performance becomes
      // an issue again
      //
      // Part 2: I have now, with great attention and profiling, solidified the position that
      //       Whatsits are immutable during the absorption phase -- and hence
      // the args and props passed in here will remain immutable in latexml_oxide.
      // Hence, for this absorb call to run correctly, it must either:
      // 1) Accept a cloned value as currently, paying with performance
      // 2) Accept immutable references to digested objects,
      // which may lead to far-reaching borrowing constraints
      //   e.g. unlist()-ing a digested List will have to produce box references,
      //  rather than provide the owned boxes directly.
      //   would have to experiment with this - as it is of course much lighter on performance
      //

      // Update 2022: The notes are generally still accurate,
      // but cloning a Digested object is now cheap enough,
      // as each enum variant is guarded by an Rc reference counter. Rc<Tbox>, Rc<List>, etc.
      if let Some(Stored::Digested(tags)) = props.get("tags") {
        document.absorb(tags, None)?;
      }
      let title = prop_digested!(props, "title");
      document.insert_element("ltx:title", title, None)?;

      let toctitle = prop_digested!(props, "toctitle");
      if !toctitle.is_empty() {
        document.insert_element("ltx:toctitle", toctitle, None)?;
      }
    },
    properties => sub[args] {
      let stype = args[0].as_ref().unwrap();
      // let inlist = args[1].as_ref().unwrap();
      let toctitle_arg = args[2].as_ref();
      let title = args[3].as_ref().unwrap();

      maybe_peek_label()?;
      let stype_str = stype.to_string();
      let mut props = ref_step_counter(&stype_str, false)?;
      // For appendix, look up the backmatter element mapping
      if stype_str == "appendix" {
        if let Some(bme) = state::lookup_mapping("BACKMATTER_ELEMENT", &s!("ltx:{stype_str}")) {
          props.insert("backmatterelement", bme);
        }
      }
      let toctitle = match toctitle_arg {
        Some(v) => if !v.to_string().is_empty() {
          args[2].as_ref().unwrap()
        } else {
          title
        },
        None => title
      };
      let stype_tokens = stype.revert()?;
      let title_tokens = title.revert()?;
      let invoked_title =
        Invocation!(T_CS!("\\lx@format@title@@"), vec![stype_tokens, title_tokens]);
      let xtitle    = stomach::digest(invoked_title)?;
      let invoked_toctitle = Invocation!(T_CS!("\\lx@format@toctitle@@"),
          vec![stype.revert()?, toctitle.revert()?]);
      let xtoctitle = stomach::digest(invoked_toctitle)?;

      if xtoctitle.to_string() != xtitle.to_string() {
        props.insert("toctitle", xtoctitle.into());
      }
      props.insert("title", xtitle.into());

      Ok(props)
    }
  );

  // No tags, at all? Consider...
  DefConstructor!("\\@@unnumbered@section{} Undigested OptionalUndigested Undigested",
  sub[document, args, props] {
      let stype = args[0].as_ref().unwrap();
      let inlist = args[1].as_ref().unwrap();
      // If backmatter, find insertion point as if inserting the backmatter element type
      if let Some(asif) = props.get("backmatterelement") {
        let asif_str = asif.to_string();
        let point = document.find_insertion_point(&asif_str, None)?;
        document.set_node(&point);
      }
      let id = props.get("id").unwrap().to_string();
      document.open_element(&s!("ltx:{stype}"),
        Some(string_map!(
          "xml:id" => clean_id(&id),
          "inlist"  => inlist.to_string()
        )), None)?;
      let title = prop_digested!(props, "title");
      document.insert_element("ltx:title", title, None)?;

      let toctitle = prop_digested!(props, "toctitle");
      if !toctitle.is_empty() {
        document.insert_element("ltx:toctitle", toctitle, None)?;
      }
    },
    properties => sub[args] {
      use DigestedData::*;
      let stype = args[0].as_ref().unwrap();
      // let inlist = args[1].as_ref().unwrap();
      let toctitle_arg = args[2].as_ref();
      let title = args[3].as_ref().unwrap();
      maybe_peek_label()?;
      let stype_str = stype.to_string();
      let mut props = RefStepID!(&stype_str)?;
      // For appendix, look up the backmatter element mapping
      if stype_str == "appendix" {
        if let Some(bme) = state::lookup_mapping("BACKMATTER_ELEMENT", &s!("ltx:{stype_str}")) {
          props.insert("backmatterelement", bme);
        }
      }
      let title_digested = if let Postponed(tokens) = title.data() {
        // TODO: is .clone() on the tokens before they are unlisted a sign that
        // the DigestedData::Postponed variant isn't ideal?
        // should we be draining it? Or is there a better conceptual organization?
        stomach::digest(
          Tokens!(T_CS!("\\lx@hidden@bgroup"), tokens.clone().unlist(), T_CS!("\\lx@hidden@egroup")))?
      } else {
        title.clone()
      };
      props.insert("title", title_digested.into());

      if let Some(toctitle) = toctitle_arg {
        if let Postponed(toctokens) = toctitle.data() {
          if !toctokens.is_empty() {
            let toctitle_digested = stomach::digest(
              Tokens!(T_CS!("\\lx@hidden@bgroup"),
                toctokens.clone().unlist(), T_CS!("\\lx@hidden@egroup")))?;
            props.insert("toctitle", toctitle_digested.into());
          }
        }
      }
      Ok(props)
    }
  );

  //----------------------------------------------------------------------
  // The following macros provide a few layers of customization
  // in particular for supporting localization for different languages.
  //----------------------------------------------------------------------
  // \lx@format@title@@{type}{title} — implemented in base_utilities.rs
  // \lx@format@toctitle@@{type}{toctitle} — implemented in base_utilities.rs
  // \lx@@compose@title{}{} — implemented in base_utilities.rs
  // \lx@tag[][ ]{}{} — implemented in base_utilities.rs
  //
  // \@@section{type}{id}{refnum}{formattedrefnum}{toctitle}{title}

  // DefConstructor!(
  //   "\\@@section{}{}{}{}{}{}",
  //   replacement!(document, args, props, inner{
  //     unpack!(args => stype, id, refnum_arg, frefnum_arg, toctitle, title);
  //     let refnum = refnum_arg.to_string();
  //     let mut frefnum = frefnum_arg.to_string();
  //     if frefnum == refnum {
  //       frefnum = String::new();
  //     }

  //     let clean_id = id; // TODO: CleanID($id);
  //     let has_toctitle =
  //       !toctitle.to_string().is_empty() && (toctitle.to_string() != title.to_string());
  //     document.open_element(
  //       &s!("ltx:{}", stype.to_string()),
  //       Some(string_map!("xml:id" => clean_id, "refnum" => refnum, "frefnum" => frefnum)),
  //       None,
  //       inner_state::
  //     )?;
  //     document.insert_element("ltx:title", vec![title], None, inner_state::?;
  //     if has_toctitle {
  //       document.insert_element("ltx:toctitle", vec![toctitle], None, inner_state::?;
  //     }
  //   }),
  //   state
  // );

  // Not sure if this is best, but if no explicit \section'ing...
  //### Tag('ltx:section',autoOpen=>1);

  //======================================================================
  // C.4.2 The Appendix
  //======================================================================
  // Handled in article,report or book.
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\appendixesname", "Appendixes");
  // TODO: add the rest...
  DefMacro!("\\@@appendix", "\\@startsection{appendix}{0}{}{}{}{}");

  //======================================================================
  // C.4.3 Table of Contents
  //======================================================================
  // Insert stubs that will be filled in during post processing.
  DefMacro!("\\contentsname", "Contents");
  DefConstructor!("\\tableofcontents",
    "<ltx:TOC lists='toc' scope='global' select='#select'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => {
      let mut td = CounterValue!("tocdepth").value_of() as usize + 1;
      let s  = ["ltx:part", "ltx:chapter", "ltx:section", "ltx:subsection", "ltx:subsubsection",
          "ltx:paragraph", "ltx:subparagraph"];
      let max_level = s.len()-1;
      td = std::cmp::min(td,max_level);
      let mut s_depth : Vec<&'static str> = s.into_iter().take(td+1).collect();
      if !s_depth.is_empty() {
        s_depth.push("ltx:appendix");
        s_depth.push("ltx:index");
        s_depth.push("ltx:bibliography");
      }

      Ok(stored_map!("select" => s_depth.join(" | "),
        "name" => digest(T_CS!("\\contentsname"))?))
    }
  );

  DefMacro!("\\listfigurename", "List of Figures");
  DefConstructor!("\\listoffigures",
    "<ltx:TOC lists='lof' scope='global'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => { Ok(stored_map!("name" => stomach::digest(T_CS!("\\listfigurename"))?)) });

  DefMacro!("\\listtablename", "List of Tables");
  DefConstructor!("\\listoftables",
    "<ltx:TOC lists='lot' scope='global'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => { Ok(stored_map!("name" => stomach::digest(T_CS!("\\listtablename"))?)) });

  DefPrimitive!("\\numberline{}{}", None);
  DefPrimitive!("\\addtocontents{}{}", None);

  DefConstructor!("\\addcontentsline{}{}{}", sub[document,args] {
      if let [inlist,_vtype,_title @ ..] = args.as_slice() {
        // Note that the node can be inlist $inlist.
        // Could conceivably want to add $title as toctitle???
        if let Some(savenode) = document.float_to_label() {
          // DG: The Document+Node mutability API is strange
          //     w.r.t the original Perl ergonomics.
          // if we use `.get_node_mut()` we can no longer `doc.set_attribute(node)`,
          // as it induces TWO simultaneous mutable pointers into document.
          // cloning Node is now cheap enough (as the Node data lives in C's libxml)
          // but it's not yet an idiomatic Rust interface. Something to ponder...
          let mut node  = document.get_node().clone();
          let inlist_str = inlist.as_ref().map(|v|v.to_string()).unwrap_or_default();
          let inlist_v = if let Some(lists) = node.get_attribute("inlist") {
            if !lists.is_empty() {
              s!("{lists} {inlist_str}")
            } else { inlist_str }
          } else {
            inlist_str
          };
          document.set_attribute(&mut node, "inlist", &inlist_v)?;
          document.set_node(&savenode);
        }
      }
    }
  );

  //======================================================================
  // C.4.4 Style registers
  //======================================================================
  NewCounter!("tocdepth");


  // ======================================================================
  // C.5 Classes, Packages and Page Styles
  // ======================================================================


  // ======================================================================
  // C.5.2 Packages
  // ======================================================================
  // We'll prefer to load package.pm, but will try package.sty or
  // package.tex (the latter being unlikely to work, but....)
  // See Stomach.pm for details
  // Ignorable packages ??
  // pre-defined packages??

  DefMacro!("\\@clsextension", "cls");
  DefMacro!("\\@pkgextension", "sty");
  Let!("\\@currext", "\\@empty");
  Let!("\\@currname", "\\@empty");
  Let!("\\@classoptionslist", "\\relax");
  Let!("\\@raw@classoptionslist", "\\relax");
  DefMacro!("\\@declaredoptions", None);
  DefMacro!("\\@curroptions", None);
  DefMacro!("\\@unusedoptionlist", None);

  DefConstructor!("\\usepackage OptionalSemiverbatim Semiverbatim []",
                  "<?latexml package='#2' ?#1(options='#1')?>",
    before_digest => { only_preamble("\\usepackage") },
    after_digest => sub[whatsit] {
      let options: Option<&Digested> = whatsit.get_arg(1);
      let packages: Option<&Digested> = whatsit.get_arg(2);
      let package_list = match packages {
        Some(value) => OPTS_REGEX.split(&value.to_string())
          .map(ToString::to_string).filter(|s| !s.starts_with('%')).collect(),
        None => Vec::new(),
      };
      let options_list = match options {
        Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(ToString::to_string).collect(),
        None => Vec::new(),
      };
      for package in package_list {
        require_package(&package, RequireOptions {
          options: options_list.clone(),
          ..RequireOptions::default()
        })?
      }
      Ok(Vec::new())
    }
  );

  DefConstructor!("\\RequirePackage OptionalSemiverbatim Semiverbatim []",
  "<?latexml package='#2' ?#1(options='#1')?>",
  before_digest =>  { only_preamble("\\RequirePackage") },
  after_digest => sub[whatsit] {
    // let options  = whatsit.get_arg(1);
    let packages = whatsit.get_arg(2).unwrap();
  //   $options = [($options ? split(/\s*,\s*/, (ToString($options))) : ())];
    for pkg in packages.to_string().split(',') {
      let pkg_trimmed = pkg.trim();
      if pkg_trimmed.is_empty() || pkg.starts_with('%') { continue; }
      require_package(pkg, RequireOptions::default())?;
    }
  });

  DefConstructor!("\\LoadClass OptionalSemiverbatim Semiverbatim []",
    "<?latexml class='#2' ?#1(options='#1')?>",
    before_digest => { only_preamble("\\LoadClass") }
    after_digest => sub[whatsit] {
      let options_arg: Option<&Digested> = whatsit.get_arg(1);
      let class_arg: Option<&Digested> = whatsit.get_arg(2);
      let class = class_arg.map(|c| c.to_string().replace(' ', "")).unwrap_or_default();
      let options: Vec<String> = match options_arg {
        Some(opts) => OPTS_REGEX.split(&opts.to_string())
          .map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        None => Vec::new(),
      };
      load_class(&class, options, Tokens!())?;
    }
  );

  // Related internal macros for package definition
  // Internals used in Packages
  DefMacro!("\\NeedsTeXFormat{}[]", None);

  DefPrimitive!("\\ProvidesClass{}[]", sub[(class, version_opt)] {
    let ver_cs = T_CS!(s!("\\ver@{class}.cls"));
    let version = version_opt.unwrap_or_default();
    DefMacro!(ver_cs, None, version, scope => Some(Scope::Global));
  });

  // Note that these, like LaTeX, define macros like \var@mypkg.sty to give the version info.
  DefMacro!("\\ProvidesPackage{}[]", sub[(package, version_opt)] {
    let ver_cs = T_CS!(s!("\\ver@{package}.sty"));
    let version = version_opt.unwrap_or_default();
    DefMacro!(ver_cs, None, version, scope => Some(Scope::Global));
  });

  DefMacro!("\\ProvidesFile{}[]", sub[(file, version_opt)] {
    let ver_cs = T_CS!(s!("\\ver@{file}"));
    let version = version_opt.unwrap_or_default();
    DefMacro!(ver_cs, None, version, scope => Some(Scope::Global));
  });

  // anything useful?
  //\DeclareRelease{v4.46}{2020-03-19}{glossaries-2020-03-19.sty}
  DefMacro!("\\DeclareRelease{}{}{}", None);
  //\DeclareCurrentRelease{v4.49}{2021-11-01}
  DefMacro!("\\DeclareCurrentRelease{}{}", None);
  DefMacro!("\\IncludeInRelease{}{}{} Until:\\EndIncludeInRelease", None);
  DefMacro!("\\NewModuleRelease{}{}{} Until:\\EndModuleRelease", None);

  DefPrimitive!("\\DeclareOption{}{}", sub[(option, code)] {
    let option_str = option.to_string();
    if option_str == "*" {
      DeclareOption!(None, code);
    } else {
      DeclareOption!(option_str, code);
    }
    Ok(Vec::new())
  });

  // Perl: latex_constructs.pool.ltxml lines 868-878
  DefPrimitive!("\\PassOptionsToPackage{}{}", sub[(options, name)] {
    let name_str = Expand!(name).to_string().replace(' ', "");
    let opts_str = Expand!(options).to_string();
    let opts: Vec<String> = opts_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    state::push_value(&s!("opt@{}.sty", name_str), opts)?;
  });

  DefPrimitive!("\\PassOptionsToClass{}{}", sub[(options, name)] {
    let name_str = Expand!(name).to_string().replace(' ', "");
    let opts_str = Expand!(options).to_string();
    let opts: Vec<String> = opts_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    state::push_value(&s!("opt@{}.cls", name_str), opts)?;
  });

  DefConstructor!("\\RequirePackageWithOptions Semiverbatim []",
  "<?latexml package='#1'?>",
  before_digest => { only_preamble("\\RequirePackage") }
  // afterDigest  => sub { my ($stomach, $whatsit) = @_;
  //   my $package = ToString($whatsit->getArg(1));
  //   $package =~ s/\s+//g;
  //   RequirePackage($package, withoptions => 1);
  //   return; }
  );

  DefConstructor!("\\LoadClassWithOptions Semiverbatim []", "<?latexml class='#1'?>",
    before_digest => { only_preamble("\\LoadClassWithOptions") }
    // afterDigest  => sub { my ($stomach, $whatsit) = @_;
    //   my $class = ToString($whatsit->getArg(1));
    //   $class =~ s/\s+//g;
    //   LoadClass($class, withoptions => 1);
    //   return; });
  );
  // Perl: latex_constructs.pool.ltxml L900-903
  DefPrimitive!("\\@onefilewithoptions {} [][] {}", sub[(name, option1, _option2, ext)] {
    let name_str = Expand!(name).to_string();
    let ext_str = Expand!(ext).to_string();
    let opts_str = match option1 {
      Some(o) => Expand!(o).to_string(),
      None => String::new(),
    };
    let options: Vec<String> = opts_str.split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();
    let _ = input_definitions(&name_str, NewDefault!(InputDefinitionOptions,
      extension => Some(Cow::Owned(ext_str)),
      handleoptions => true,
      options => options
    ));
  });

  DefMacro!("\\CurrentOption", None);

  // Perl: latex_constructs.pool.ltxml lines 907-919
  DefPrimitive!("\\OptionNotUsed", {
    let option = Expand!(T_CS!("\\CurrentOption")).to_string();
    if !option.is_empty() {
      let ext = Expand!(T_CS!("\\@currext")).to_string();
      if ext == "cls" {
        state::push_value("@unusedoptionlist", option)?;
      }
    }
  });
  DefPrimitive!("\\@unknownoptionerror", {
    let option = Expand!(T_CS!("\\CurrentOption")).to_string();
    let name = Expand!(T_CS!("\\@currname")).to_string();
    Info!("unexpected", &option, &s!("Unknown option '{}' for {}", option, name));
  });

  DefPrimitive!("\\ExecuteOptions{}", sub[(options)] {
    let expanded = do_expand(options)?.to_string();
    let opts: Vec<&str> = expanded.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    execute_options(&opts)?;
    Ok(Vec::new())
  });

  DefPrimitive!("\\ProcessOptions OptionalMatch:*", sub[(star)] {
    // Perl: ProcessOptions(($star ? (inorder => 1) : ()));
    let inorder = star.is_some();
    process_options(inorder)?;
    Ok(Vec::new())
  });
  DefMacro!("\\@options", "\\ProcessOptions*");

  Let!("\\@enddocumenthook", "\\@empty");
  DefMacro!("\\AtEndOfPackage{}", sub [(code)] {
    let name = Expand!(T_CS!("\\@currname")).to_string();
    let ttype = Expand!(T_CS!("\\@currext")).to_string();
    let hookcs = T_CS!(s!("\\{name}.{ttype}-h@@k"));
    AddToMacro!(hookcs, code);
  });

  DefMacro!("\\@ifpackageloaded", r"\@ifl@aded\@pkgextension");
  Let!("\\ltx@ifpackageloaded", r"\@ifpackageloaded");
  DefMacro!("\\@ifclassloaded", r"\@ifl@aded\@clsextension");
  Let!("\\ltx@ifclassloaded", r"\@ifclassloaded");
  // Latex.ltx L15252-15256: LaTeX3-style aliases for the file-load
  // tracking commands. The dump captures these as `Lt(...)` self-let
  // entries that don't actually replay because we filter same-target
  // aliases in `dump_writer`. Re-establish here post-dump.
  Let!("\\IfPackageLoadedTF", r"\@ifpackageloaded");
  Let!("\\IfClassLoadedTF", r"\@ifclassloaded");
  Let!("\\IfPackageAtLeastTF", r"\@ifpackagelater");
  Let!("\\IfClassAtLeastTF", r"\@ifclasslater");
  Let!("\\IfFormatAtLeastTF", r"\@ifl@t@r@released");
  Let!("\\IfFileAtLeastTF", r"\@ifl@t@r");
  DefMacro!("\\@ifl@aded{}{}", sub[(ext, name)] {
  let path = s!("{}.{}", Expand!(name), Expand!(ext));
  // Per OXIDIZED_DESIGN #23: a package is "loaded" when EITHER the
  // binding (`_loaded`) OR the raw .sty/.cls (`_raw_loaded`) is in
  // place. User-level `\@ifpackageloaded{X}` doesn't care which path.
  // Mirrors Perl `\@ifpackageloaded` checking `<X.sty>_loaded` (which
  // is set by both Perl `loadLTXML` and `loadTeXDefinitions`).
  if lookup_bool(&s!("{path}_loaded")) || lookup_bool(&s!("{path}_raw_loaded")) {
    T_CS!("\\@firstoftwo")
  } else {
    T_CS!("\\@secondoftwo")
  }});

  DefMacro!("\\@ifpackagewith", r"\@if@ptions\@pkgextension");
  DefMacro!("\\@ifclasswith", r"\@if@ptions\@clsextension");
  // Perl: latex_constructs.pool.ltxml lines 952-958
  DefMacro!("\\@if@ptions{}{}{}", sub[(ext, name, option)] {
    let option_str = Expand!(option).to_string();
    let key = s!("opt@{}.{}", Expand!(name), Expand!(ext));
    let found = with_value(&key, |val_opt| {
      if let Some(Stored::VecDequeStored(values)) = val_opt {
        values.iter().any(|v| v.to_string() == option_str)
      } else {
        false
      }
    });
    if found {
      T_CS!("\\@firstoftwo")
    } else {
      T_CS!("\\@secondoftwo")
    }
  });
  DefMacro!(
    "\\@ptionlist {}",
    r"\@ifundefined{opt@#1}\@empty{\csname opt@#1\endcsname}"
  );

  // Perl L962: DefMacro('\g@addto@macro DefToken {}', sub { AddToMacro(...) });
  // The state mutation fires during gullet expansion, not stomach-level
  // digestion — so \g@addto@macro takes effect immediately when the
  // expansion chain reaches it (needed for \edef / \AtBeginDocument
  // callers that rely on the hook-macro state being visible during
  // subsequent expansion in the same batch).
  DefMacro!("\\g@addto@macro DefToken {}", sub[(target, content)] {
    AddToMacro!(target, content);
    Ok(Tokens!())
  });
  DefMacro!("\\addto@hook DefToken {}", "#1\\expandafter{\\the#1#2}");

  // Alas, we're not tracking versions, so we'll assume it's "later" & cross fingers....
  DefMacro!("\\@ifpackagelater{}{}{}{}", "#3");
  DefMacro!("\\@ifclasslater{}{}{}{}", "#3");
  Let!("\\AtEndOfClass", "\\AtEndOfPackage");

  DefMacro!("\\AtBeginDvi {}", None);

  TeX!(
    r###"
  \def\@ifl@t@r#1#2{%
    \ifnum\expandafter\@parse@version@#1//00\@nil<%
          \expandafter\@parse@version@#2//00\@nil
      \expandafter\@secondoftwo
    \else
      \expandafter\@firstoftwo
    \fi}
  \def\@parse@version@#1{\@parse@version0#1}
  \def\@parse@version#1/#2/#3#4#5\@nil{%
  \@parse@version@dash#1-#2-#3#4\@nil
  }
  \def\@parse@version@dash#1-#2-#3#4#5\@nil{%
    \if\relax#2\relax\else#1\fi#2#3#4 }"###
  );

  //======================================================================
  // Somewhat related I/O stuff
  DefMacro!("\\filename@parse{}", sub[(pathname)] {
    let (mut dir, name, ext) = pathname::split(&Expand!(pathname).to_string());
    if !dir.is_empty() {
      dir.push('/');
    }
    let dir_tokens = Tokens!(ExplodeText!(dir));
    DefMacro!("\\filename@area", None, dir_tokens);
    let name_tokens = Tokens!(ExplodeText!(name));
    DefMacro!("\\filename@base", None, name_tokens);
    let ext_tokens = if !ext.is_empty() {
      Tokens!(ExplodeText!(ext))
    } else { Tokens!(T_CS!("\\relax")) };
    DefMacro!("\\filename@ext", None, ext_tokens);
    Vec::new()
  });

  // latex.ltx initializes \@filelist to \@gobble, which eats the leading comma
  // from the first \@addtofilelist call. We replicate this by using \@gobble.
  DefMacro!("\\@filelist", "\\@gobble");
  DefMacro!("\\@addtofilelist{}", sub[(arg)] {
    let expansion = Expand!(Tokens!(T_CS!("\\@filelist"), T_OTHER!(","), arg.unlist()));
    DefMacro!("\\@filelist",None,expansion);
    Vec::new()
  });


  //======================================================================
  // C.5.3 Page Styles
  //======================================================================
  // Ignored
  // Perl 74181415 (#2442): page counter starts at 1, not 0.
  NewCounter!("page");
  SetCounter!("page" => Number::new(1));
  DefMacro!("\\@mkboth", "\\@gobbletwo");
  DefMacro!("\\ps@empty",
    "\\let\\@mkboth\\@gobbletwo\\let\\@oddhead\\@empty\\let\\@oddfoot\\@empty\
     \\let\\@evenhead\\@empty\\let\\@evenfoot\\@empty");
  DefMacro!("\\ps@plain",
    "\\let\\@mkboth\\@gobbletwo\
     \\let\\@oddhead\\@empty\\def\\@oddfoot{\\reset@font\\hfil\\thepage\
     \\hfil}\\let\\@evenhead\\@empty\\let\\@evenfoot\\@oddfoot");
  Let!("\\@leftmark", "\\@firstoftwo");
  Let!("\\@rightmark", "\\@secondoftwo");

  DefPrimitive!("\\pagestyle{}", None);
  DefPrimitive!("\\thispagestyle{}", None);
  DefPrimitive!("\\markright{}", None);
  DefPrimitive!("\\markboth{}{}", None);
  DefPrimitive!("\\leftmark", None);
  DefPrimitive!("\\rightmark", None);
  DefPrimitive!("\\pagenumbering{}", None);
  // Perl: DefMacro('\twocolumn[]', '\ifx.#1.\else\par\noindent#1\fi\par');
  DefMacro!("\\twocolumn[]", "\\ifx.#1.\\else\\par\\noindent#1\\fi\\par");
  // Perl: DefMacro('\onecolumn', '\par');
  DefMacro!("\\onecolumn", "\\par");
  DefMacro!("\\@onecolumna", "", locked => true);
  DefMacro!("\\@twocolumna", "", locked => true);

  // Style parameters from Fig. C.3, p.182
  DefRegister!("\\paperheight"     => Dimension!("11in"));
  DefRegister!("\\paperwidth"      => Dimension!("8.5in"));
  DefRegister!("\\textheight"      => Dimension!("550pt"));
  DefRegister!("\\textwidth"       => Dimension!("345pt"));
  DefRegister!("\\topmargin"       => Dimension::new(0));
  DefRegister!("\\headheight"      => Dimension::new(0));
  DefRegister!("\\headsep"         => Dimension::new(0));
  DefRegister!("\\footskip"        => Dimension::new(0));
  DefRegister!("\\footheight"      => Dimension::new(0));
  DefRegister!("\\evensidemargin"  => Dimension::new(0));
  DefRegister!("\\oddsidemargin"   => Dimension::new(0));
  DefRegister!("\\marginparwidth"  => Dimension::new(0));
  DefRegister!("\\marginparsep"    => Dimension::new(0));
  DefRegister!("\\columnwidth"     => Dimension!("6in"));
  DefRegister!("\\linewidth"       => Dimension!("6in"));
  DefRegister!("\\baselinestretch" => Dimension::new(0));
  // Perl: latex_base.pool.ltxml lines 309-311
  DefRegister!("\\columnsep"       => Dimension::new(0));
  DefRegister!("\\columnseprule"   => Dimension::new(0));
  DefRegister!("\\mathindent"      => Dimension::new(0));

  TeX!(
    r"\def\@ifl@t@r#1#2{%
  \ifnum\expandafter\@parse@version@#1//00\@nil<%
        \expandafter\@parse@version@#2//00\@nil
    \expandafter\@secondoftwo
  \else
    \expandafter\@firstoftwo
  \fi}
\def\@parse@version@#1{\@parse@version0#1}
\def\@parse@version#1/#2/#3#4#5\@nil{%
\@parse@version@dash#1-#2-#3#4\@nil
}
\def\@parse@version@dash#1-#2-#3#4#5\@nil{%
  \if\relax#2\relax\else#1\fi#2#3#4 }
"
  );


  //======================================================================
  // C.5.4 The Title Page and Abstract
  //======================================================================
  // See frontmatter support in TeX.ltxml

  Let!("\\@title", "\\@empty");
  DefMacro!("\\title{}", "\\def\\@title{#1}\\@add@frontmatter{ltx:title}{#1}", locked => true);
  DefMacro!("\\@date", "\\@empty");
  DefMacro!(
    "\\date{}",
    r"\def\@date{#1}\
\@add@frontmatter{ltx:date}[role=creation,name={\@ifundefined{datename}{}{\datename}}]{#1}"
  );
  // Perl latex_constructs.pool.ltxml L1062-1064: DefConstructor('\person@thanks{}', ...,
  //   alias => '\thanks', mode => 'restricted_horizontal', enterHorizontal => 1).
  DefConstructor!("\\person@thanks{}", "^ <ltx:contact role='thanks'>#1</ltx:contact>",
    alias => "\\thanks", mode => "text", enter_horizontal => true);
  // Perl L1065-1067: DefConstructor('\@personname{}', ...,
  //   beforeDigest => { Let('\thanks', '\person@thanks') },
  //   mode => 'restricted_horizontal', enterHorizontal => 1).
  DefConstructor!("\\@personname{}", "<ltx:personname>#1</ltx:personname>",
    before_digest => { Let!("\\thanks", "\\person@thanks"); },
    bounded => true,
    mode => "text",
    enter_horizontal => true
  );

  // Sanitize person names for (obvious) punctuation abuse at start+end
  Tag!("ltx:personname", after_close => sub[_document, node] {
    if let Some(mut first) = node.get_first_child() {
      if first.get_type() == Some(NodeType::TextNode) {
        let first_text = first.get_content();
        let mut first_text_iter = first_text.chars().peekable();
        while let Some(peeked) = first_text_iter.peek() {
          if peeked.is_whitespace() || matches!(peeked, ',' | '!' | ';' | '.' | ':' | '?') {
            first_text_iter.next();
          } else {
            break;
          }
        }
        let new_text = first_text_iter.collect::<String>();
        if first_text != new_text {
          first.set_content(&new_text)?;
        }
      }
      if let Some(mut last) = node.get_last_child() {
        if last.get_type() == Some(NodeType::TextNode) {
          let last_text = last.get_content();
          let mut last_text_iter  = last_text.chars().rev().peekable();
          while let Some(peeked) = last_text_iter.peek() {
            if peeked.is_whitespace() || matches!(peeked, ',' | '!' | ';' | '.' | ':' | '?') {
              last_text_iter.next();
            } else {
              break;
            }
          }
          let new_text = last_text_iter.rev().collect::<String>();

          if last_text != new_text {
            last.set_content(&new_text)?;
          }
        }
      }
    }
  });

  DefConstructor!("\\and", " and ");

  AssignValue!("NUMBER_OF_AUTHORS" => 0);
  DefPrimitive!("\\lx@count@author", {
    let current = lookup_int("NUMBER_OF_AUTHORS");
    AssignValue!("NUMBER_OF_AUTHORS" => current + 1, Some(Scope::Global));
  });
  DefMacro!(
    "\\lx@author{}",
    r"\lx@count@author\@add@frontmatter{ltx:creator}[role=author]{\lx@author@prefix\@personname{#1}}"
  );
  DefConstructor!("\\lx@@@contact{}{}", "^ <ltx:contact role='#1'>#2</ltx:contact>");
  DefMacro!("\\lx@contact{}{}",
  r"\@add@to@frontmatter{ltx:creator}{\lx@@@contact{#1}{#2}}");
  DefMacro!("\\lx@author@sep", "\\qquad");
  DefMacro!("\\lx@author@conj", "\\qquad");
  DefConstructor!("\\lx@author@prefix", sub[document, _args, _props] {
    let mut node   = document.get_element().unwrap();
    let nauthors   = lookup_int("NUMBER_OF_AUTHORS");
    let i          = document.findnodes("//ltx:creator[@role='author']", None).len() as i64;
    if i <= 1 { }
    else if i == nauthors {
      // Perl: setAttribute(before => DigestText(T_CS('\lx@author@conj'))).
      // `\lx@author@conj` is overridable: latex_constructs sets it to
      // `\qquad` (em-spaces); ams_support overrides to `\ and\ `. Use
      // `get_string()` rather than `to_string()` so Tbox text content
      // (chars from `\ and\ `) is concatenated, but we also need
      // `\hskip` Whatsits (from `\qquad → \hskip 2em\relax`) to fall
      // back to `dimension_to_spaces(width)` since `\hskip`'s Display
      // would revert to the macro name. Digest the macro and walk the
      // tree extracting text-or-spaces.
      let conj = DigestText!(Tokens!(T_CS!("\\lx@author@conj")))?;
      let s = digested_to_text(&conj)?;
      document.set_attribute(&mut node, "before", &s)?;
    } else {
      let sep = DigestText!(Tokens!(T_CS!("\\lx@author@sep")))?;
      let s = digested_to_text(&sep)?;
      document.set_attribute(&mut node, "before", &s)?;
    }
  });

  DefMacro!("\\@author", "\\@empty");
  // Perl latex_constructs.pool.ltxml L1116:
  //   DefMacro('\author[]{}', '\def\@author{#2}\lx@make@authors@anded{#2}', locked => 1);
  // The optional `[short]` arg is standard for many journal classes (mn,
  // elsart, revtex variants, etc.); without it, `\author[short]{long}`
  // leaves `[short]` in the token stream, dumping it into whatever context
  // was around — most visibly, if the author has `$...$` math, the leftover
  // `[short]` gets parsed inside math, which then drifts into `\thanks`
  // bodies and produces XMTok-in-note schema errors (arxiv 0709.4470,
  // 0802.3360). The short form is used for running heads/toc and is
  // otherwise discarded.
  DefMacro!("\\author[]{}", "\\def\\@author{#2}\\lx@make@authors@anded{#2}", locked => true);
  DefMacro!("\\lx@make@authors@anded{}", sub[(authors)] {
    and_split(T_CS!("\\lx@author"), authors)
  });
  DefPrimitive!("\\ltx@authors@oneline", {
    AssignMapping!("DOCUMENT_CLASSES", "ltx_authors_1line" => true);
  });
  DefPrimitive!("\\ltx@authors@multiline", {
    AssignMapping!("DOCUMENT_CLASSES", "ltx_authors_multiline" => true);
  });

  DefMacro!(
    "\\@add@conversion@date",
    "\\@add@frontmatter{ltx:date}[role=creation]{\\today}"
  );

  // Perl: latex_constructs.pool.ltxml L1128-1129
  // In case \@maketitle defines \And/\AND — we can't emulate that, so map them to \and
  // for and_split to properly separate authors.
  Let!("\\And", "\\and");
  Let!("\\AND", "\\and");

  // Doesn't produce anything (we're already inserting frontmatter),
  // But, it does make the various frontmatter macros into no-ops.
  // Locked: raw TeX packages (e.g., nips_2017.sty) may \renewcommand{\maketitle}, but
  // LaTeXML's frontmatter handling must take precedence. Perl achieves this by having
  // the compiled binding override raw TeX; we use `locked` to prevent raw overwrite.
  DefMacro!(
    "\\maketitle",
    r"\lx@frontmatterhere\let\lx@frontmatter@fallback\relax\@startsection@hook\global\let\thanks\relax\global\let\maketitle\relax\
\global\let\@maketitle\relax\global\let\@thanks\@empty\global\let\@author\@empty\
\global\let\@date\@empty\global\let\@title\@empty\global\let\title\relax\
\global\let\author\relax\global\let\date\relax\global\let\and\relax",
    locked => true
  );
  // In case \maketitle isn't used in the document, let's check for it.
  AddToMacro!("\\@startsection@hook", "\\lx@frontmatter@fallback");
  // in cases such as titlepage, the document end is the last fallback.
  let _ = state::push_value("@at@end@document",
    Tokens!(T_CS!("\\lx@frontmatter@fallback")));

  DefMacro!("\\@thanks", "\\@empty");
  // Perl latex_constructs.pool.ltxml L1154: `\thanks[]{}` — optional arg for
  // OmniBus use (thrown away). #2 is the required body.
  DefMacro!("\\thanks[]{}", r"\def\@thanks{#2}\lx@make@thanks{#2}");
  DefConstructor!(
    "\\lx@make@thanks{}",
    "<ltx:note role='thanks'>#1</ltx:note>"
  );

  // Abstract SHOULD have been so simple, but seems to be a magnet for abuse.
  // For one thing, we'd like to just write
  //   DefEnvironment('{abstract}','<ltx:abstract>//body</ltx:abstract>');
  // However, we don't want to place the <ltx:abstract> environment directly where
  // we found it, but we want to add it to frontmatter. This requires capturing the
  // recently digested list and storing it in the frontmatter structure.

  // The really messy stuff comes from the way authors -- and style designers -- misuse it.
  // Basic LaTeX wants it to be an environment WITHIN the document environment,
  // and AFTER the \maketitle.
  // However, since all it really does is typeset "Abstract" in bold, it allows:
  //   \abstract stuff...
  // without even an \endabstract!  We MUST know when the abstract ends, so we've got
  // to recognize when we've moved on to other stuff... \sections at the VERY LEAST.

  // Additional complications come from certain other classes and styles that
  // redefine abstract to take the text as an argument. And some treat it
  // like \title, \author, and such, that are expected to appear in the preamble!!
  // The treatment below allows an abstract environment in the preamble,
  // (even though straight latex doesn't) but does not cover the 1-arg case in preamble!
  //
  // Probably there are other places (eg in titlepage?) that should force the close??

  // Perl: latex_constructs.pool.ltxml lines 1180-1194
  DefEnvironment!("{abstract}", "",
    after_digest_begin => {
      AssignValue!("inPreamble" => false);
    },
    after_digest => {
      let abstract_title = stomach::digest(Tokens!(T_CS!("\\format@title@abstract"),
        T_BEGIN!(), T_CS!("\\abstractname"), T_END!()))?;
      let regurgitated = List::new(clone_box_list());

      with_value_mut("frontmatter",|frontmatter_opt| {
        let frontmatter = match frontmatter_opt {
          Some(&mut Stored::HashTagData(ref mut frnt)) => frnt,
          _ => Fatal!(TexPool, Expected,
              "Global TeX Frontmatter hash was not available, should never happen"),
        };
        let abstr = frontmatter.entry("ltx:abstract".to_string()).or_insert_with(Vec::new);
        abstr.push(("ltx:abstract".to_string(),
          Some(string_map!("name" => abstract_title)), regurgitated.into()));
        Ok(())
      })?;
      DefMacro!("\\maybe@end@abstract", "", scope => Some(Scope::Global));
    },
    after_construct => sub[doc, _whatsit] {
      insert_frontmatter(doc)?; // HERE if not already done.
    },
    locked => true,
    mode => "internal_vertical"
  );
  // If we get a plain \abstract, instead of an environment, look for \abstract{the abstract}
  AssignValue!("\\abstract:locked" => false); // REDEFINE the above locked definition!
  // Perl: latex_constructs.pool.ltxml lines 1197-1203
  DefMacro!("\\abstract", {
    if gullet::if_next(T_BEGIN!())? {
      Tokens!(T_CS!("\\abstract@onearg"))
    } else {
      // When \abstract is used without braces (e.g. \abstract ... \section{...}),
      // add \maybe@end@abstract to \@startsection@hook so the abstract closes
      // when the next sectioning command starts.
      Tokens!(
        T_CS!("\\g@addto@macro"), T_CS!("\\@startsection@hook"), T_CS!("\\maybe@end@abstract"),
        T_CS!("\\begin{abstract}"))
    }
  },
  locked => true);
  DefMacro!("\\abstract@onearg{}", "\\begin{abstract}#1\\end{abstract}\\let\\endabstract\\relax");
  DefMacro!("\\maybe@end@abstract", "\\endabstract");
  DefMacro!("\\abstractname", "Abstract");
  DefMacro!("\\format@title@abstract{}", "#1");

  // Hmm, titlepage is likely to be hairy, low-level markup,
  // without even title, author, etc, specified as such!
  // Hmm, should this even redefine author, title, etc so that they
  // are simply output?
  // This is horrible hackery; What we really need, I think, is the
  // ability to bind some sort of "Do <this> when we create a text box"...
  // ON Second Thought...
  // For the time being, ignore titlepage!
  // Maybe we could do some of this if there is no title/author
  // otherwise defined? Ugh!

  //DefEnvironment('{titlepage}','');
  // Or perhaps it's better just to ignore the markers?
  //DefMacro('\titlepage','');
  //DefMacro('\endtitlepage','');

  // Or perhaps not....
  // There's a title and other stuff in here, but how could we guess?
  // Well, there's likely to be a sequence of <p><text font="xx" fontsize="yy">...</text></p>
  // Presumably the earlier, larger one is title, rest are authors/affiliations...
  // Particularly, if they start with a pseudo superscript or other "marker", they're probably
  // affil! For now, we just give an info message
  DefEnvironment!("{titlepage}", "<ltx:titlepage>#body",
    before_digest => {
      Let!("\\centering", "\\relax");
      state::assign_value("frontmatter_deferred", true, Some(Scope::Global));
      AddToMacro!("\\maketitle", "\\unwind@titlepage");
      // In titlepage, abstract is simpler: direct body
      DefEnvironment!("{abstract}", "<ltx:abstract>#body</ltx:abstract>");
      Let!("\\abstract", "\\abstract@onearg");
    },
    before_digest_end => {
      stomach::digest(Tokens!(T_CS!("\\maybe@end@titlepage")))?
    },
    after_construct => sub[doc, _whatsit] {
      insert_frontmatter(doc)?;
    },
    locked => true,
    mode => "internal_vertical"
  );

  Tag!("ltx:titlepage", auto_close => true);

  DefConstructor!("\\maybe@end@title", sub[document,_args,_props] {
    if document.is_closeable("ltx:titlepage").is_some() {
      document.close_element("ltx:titlepage")?;
    }
  });

  DefConstructor!("\\maybe@end@titlepage", sub[document,_args,_props] {
    document.maybe_close_element("ltx:titlepage")?;
  });
  DefConstructor!("\\unwind@titlepage", sub[document,_args,_props] {
    if let Some(titlepage) = document.maybe_close_element("ltx:titlepage")? {
      document.unwrap_nodes(titlepage)?;
    }
  });

  DefMacro!("\\sectionmark{}", "");
  DefMacro!("\\subsectionmark{}", "");
  DefMacro!("\\subsubsectionmark{}", "");
  DefMacro!("\\paragraphmark{}", "");
  DefMacro!("\\subparagraphmark{}", "");
  DefMacro!("\\@oddfoot", "");
  DefMacro!("\\@oddhed", "");
  DefMacro!("\\@evenfoot", "");
  DefMacro!("\\@evenfoot", "");


  // ======================================================================
  // C.6 Displayed Paragraphs
  // ======================================================================


  DefEnvironment!("{center}", sub[document, _args, props] {
    document.maybe_close_element("ltx:p")?; // this starts a new vertical block
    // aligning will take care of \\\\ "rows"
    aligning_environment("center", "ltx_centering", document, props)?;
    Ok(())
  });
  // HOWEVER, define a plain \center to act like \centering (?)
  DefMacro!("\\center", "\\centering");
  DefMacro!("\\endcenter", None);
  DefEnvironment!("{flushleft}", sub[document, _args, props] {
    document.maybe_close_element("ltx:p")?; // this starts a new vertical block
    aligning_environment("center", "ltx_align_left", document, props)?;
    Ok(())
  });
  DefEnvironment!("{flushright}", sub[document, _args, props] {
    document.maybe_close_element("ltx:p")?; // this starts a new vertical block
    aligning_environment("center", "ltx_align_right", document, props)?;
    Ok(())
  });
  // Perl latex_constructs.pool.ltxml L1316-1318: "Redefine these so they work
  // both as environments, and as single commands". The bare `\flushleft` /
  // `\flushright` commands (without matching `\end...`) are used as
  // declarations — they should NOT push a group frame + enter
  // restricted_horizontal, since that would leak mode when the enclosing
  // group (e.g. `table*`) closes.
  //
  // `\begin{flushleft}` / `\end{flushleft}` go through a separate environment
  // constructor and are unaffected by these Let aliases.
  //
  // Fixes sandbox papers 0705.2808 and 0707.4170 (mode mismatch at
  // `\end{table*}` when document uses `\flushleft` as a command inside the
  // float body).

  // # These add an operation to be carried out on the current node & following siblings, when the
  // current group ends. # These operators will add alignment (class) attributes to each "line" in
  // the current block. #DefPrimitiveI('\centering',   undef, sub {
  // UnshiftValue(beforeAfterGroup=>T_CS('\@add@centering')); }); # NOTE: THere's a problem here.
  // The current method seems to work right for these operators # appearing within the typical
  // environments.  HOWEVER, it doesn't work for a simple \bgroup or \begingroup!!! # (they don't
  // create a node! or even a whatsit!)
  // Perl: setupAligningContext saves [node, node.lastChild] to ALIGNING_NODE.
  // applyAligningContext then only applies class to children AFTER the saved lastChild.
  DefConstructor!("\\centering", sub[doc,_args] {
    setup_aligning_context(doc);
  },
  before_digest => {
    unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@centering")]);
  });
  // Perl: latex_constructs.pool.ltxml lines 1299-1302
  DefConstructor!("\\raggedright", sub[doc,_args] {
    setup_aligning_context(doc);
  },
    before_digest => {
      unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@raggedright")]);
    });
  DefConstructor!("\\raggedleft", sub[doc,_args] {
    setup_aligning_context(doc);
  },
    before_digest => {
      unshift_value("beforeAfterGroup", vec![T_CS!("\\@add@raggedleft")]);
    });

  DefConstructor!("\\@add@centering", sub[document] {
    apply_aligning_context(document, "center", "ltx_centering")?;
  });
  // Note that \raggedright is essentially align left (undef align, just class)
  DefConstructor!("\\@add@raggedright", sub[document] {
    apply_aligning_context(document, "", "ltx_align_left")?;
  });
  DefConstructor!("\\@add@raggedleft", sub[document] {
    apply_aligning_context(document, "", "ltx_align_right")?;
  });
  DefConstructor!("\\@add@flushright", sub[document] {
    let node_opt = lookup_value("ALIGNING_NODE");
    if let Some(Stored::Node(node)) = node_opt {
      for mut child in node.get_child_elements() {
        set_align_or_class(document, &mut child, "right", "ltx_align_right")?;
      }
    }
  });
  DefConstructor!("\\@add@flushleft", sub[document] {
    let node_opt = lookup_value("ALIGNING_NODE");
    if let Some(Stored::Node(node)) = node_opt {
      for mut child in node.get_child_elements() {
        set_align_or_class(document, &mut child, "left", "ltx_align_left")?;
      }
    }
  });

  // Perl latex_constructs.pool.ltxml L1317-1318: Redefine so `\flushleft` /
  // `\flushright` work both as environments AND as single commands.
  // As a command (no matching `\end...`), the bare CS acts like
  // `\raggedright` / `\raggedleft` — a declaration that applies via
  // beforeAfterGroup rather than opening a restricted_horizontal group
  // frame. `\begin{flushleft}` / `\end{flushleft}` still go through the
  // environment constructors and are unaffected.
  Let!("\\flushright", "\\raggedleft");
  Let!("\\flushleft",  "\\raggedright");

  // Perl: Let('\@block@cr', '\lx@newline');  # Obsolete, but in case still used
  Let!("\\@block@cr", "\\lx@newline");
  DefEnvironment!("{quote}",
    "<ltx:quote>#body</ltx:quote>",
    mode => "internal_vertical");
  DefEnvironment!("{quotation}",
    "<ltx:quote>#body</ltx:quote>",
    mode => "internal_vertical");
  DefEnvironment!("{verse}",
    "<ltx:quote role='verse'>#body</ltx:quote>",
    mode => "internal_vertical");


  //======================================================================
  // C.6.2 List-Making environments
  //======================================================================
  Tag!("ltx:item",        auto_close => true, auto_open => true);
  Tag!("ltx:inline-item", auto_close => true, auto_open => true);

  // These are for the (not quite legit) case where \item appears outside
  // of an itemize, enumerate, etc, environment.
  // DefCon('\item[]',
  //   "<ltx:item>?&defined(#1)(<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>)");
  // DefCon('\subitem[]',
  //   "<ltx:item>?&defined(#1)(<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>)");
  // DefCon('\subsubitem[]',
  //   "<ltx:item>?&defined(#1)(<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>)");

  // Or maybe best just to do \par ?
  DefMacro!("\\item[]", "\\par");
  DefMacro!("\\subitem[]", "\\par");
  DefMacro!("\\subsubitem[]", "\\par");

  AssignValue!("@itemlevel" => 0, Some(Scope::Global));
  AssignValue!("enumlevel"  => 0, Some(Scope::Global));
  AssignValue!("@desclevel" => 0, Some(Scope::Global));
  // protection against lower-level code...
  DefConditional!("\\if@noitemarg");
  DefMacro!("\\@item", "\\item"); // Hopefully no circles...
  DefMacro!("\\@itemlabel", ""); // Maybe needs to be same as \item will be using?

  // These counters are ONLY used for id's of ALL the various itemize, enumerate, etc elements
  // Only create the 1st level (so that binding style can start numbering 'within' appropriately)
  // Additional ones created by need.
  NewCounter!("@itemizei",   "section",      idprefix => "I");

  // Perl: latex_constructs.pool.ltxml L1505-1510 — paragraph before list items
  DefConstructor!("\\preitem@par", sub[document] {
    let _ = document.maybe_close_element("ltx:p");
    let _ = document.maybe_close_element("ltx:para");
  }, alias => "\\par");

  // Perl: latex_constructs.pool.ltxml L1560
  DefMacro!("\\@mklab{}", "\\hfil #1");

  // id, but NO refnum (et.al) attributes on itemize \\item ...
  // unless the optional tag argument was given!
  // We"ll make the <ltx:tag> from either the optional arg, or from \\labelitemi..
  DefMacro!("\\itemize@item", "\\preitem@par\\itemize@item@");
  DefConstructor!("\\itemize@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });
  DefConstructor!("\\inline@itemize@item OptionalUndigested",
    "<ltx:inline-item xml:id='#id'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });

  DefMacro!("\\enumerate@item", "\\preitem@par\\enumerate@item@");
  DefConstructor!("\\enumerate@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });
  DefConstructor!("\\inline@enumerate@item OptionalUndigested",
    "<ltx:inline-item xml:id='#id'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });

  DefMacro!("\\description@item", "\\preitem@par\\description@item@");
  DefConstructor!("\\description@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });
  DefConstructor!("\\inline@description@item OptionalUndigested",
    "<ltx:inline-item xml:id='#id'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) });

  DefEnvironment!("{itemize}",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    properties => { BeginItemize!("itemize", "@item") },
    before_digest_end => { Digest!("\\par") },
    locked => true,
    mode => "internal_vertical"
  );
  DefEnvironment!("{enumerate}",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    properties => { BeginItemize!("enumerate", "enum") },
    before_digest_end => { Digest!("\\par") },
    locked => true,
    mode => "internal_vertical"
  );
  DefEnvironment!("{description}",
    "<ltx:description  xml:id='#id'>#body</ltx:description>",
    before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
    properties => { BeginItemize!("description", "@desc") },
    before_digest_end => { Digest!("\\par") },
    locked => true,
    mode => "internal_vertical"
  );

  DefMacro!("\\makelabel{}", "#1");
  //----------------------------------------------------------------------
  // Basic itemize bits
  // Fake counter for itemize to give id's to ltx:item.
  NewCounter!("@itemi",   "", idwithin => "@itemizei", idprefix => "i");
  NewCounter!("@itemii",  "", idwithin => "@itemi",    idprefix => "i");
  NewCounter!("@itemiii", "", idwithin => "@itemii",   idprefix => "i");
  NewCounter!("@itemiv",  "", idwithin => "@itemiii",  idprefix => "i");
  NewCounter!("@itemv",   "", idwithin => "@itemiv",   idprefix => "i");
  NewCounter!("@itemvi",  "", idwithin => "@itemv",    idprefix => "i");
  // These are empty to make the "refnum" go away.
  DefMacro!("\\the@itemi", "");
  DefMacro!("\\the@itemii", "");
  DefMacro!("\\the@itemiii", "");
  DefMacro!("\\the@itemiv", "");
  DefMacro!("\\the@itemv", "");
  DefMacro!("\\the@itemvi", "");

  // Formatted item tags.
  // Really should be in the class file, but already was here.
  DefMacro!("\\labelitemi", "\\textbullet");
  DefMacro!("\\labelitemii", "\\normalfont\\bfseries \\textendash");
  DefMacro!("\\labelitemiii", "\\textasteriskcentered");
  DefMacro!("\\labelitemiv", "\\textperiodcentered");

  // Make the fake counters point to the real labels
  DefMacro!("\\label@itemi", "\\labelitemi");
  DefMacro!("\\label@itemii", "\\labelitemii");
  DefMacro!("\\label@itemiii", "\\labelitemiii");
  DefMacro!("\\label@itemiv", "\\labelitemiv");

  // These hookup latexml"s tagging to normal latex"s \labelitemi...
  DefMacro!("\\fnum@@itemi", r"{\makelabel{\label@itemi}}");
  DefMacro!("\\fnum@@itemii", r"{\makelabel{\label@itemii}}");
  DefMacro!("\\fnum@@itemiii", r"{\makelabel{\label@itemiii}}");
  DefMacro!("\\fnum@@itemiv", r"{\makelabel{\label@itemiv}}");

  DefMacro!("\\lx@poormans@ordinal{}", sub[(ctr)] {
    let mut ctr_str      = CounterValue!(&ctr.to_string()).value_of().to_string();
    let last_char = ctr_str.chars().last().unwrap_or('.');
    if last_char.is_ascii_digit() {
      ctr_str.push_str(PM_ORDINAL_SUFFICES[last_char.to_digit(10).unwrap() as usize]);
    }
    T_OTHER!(ctr_str)
  });
  DefMacro!("\\itemtyperefname", "item");
  DefMacro!("\\itemcontext", "\\space in \\@listcontext");
  DefMacro!("\\itemcontext", "");
  // Probably would help to give a bit more context for the ii & higher?
  DefMacro!(
    "\\typerefnum@@itemi",
    "\\lx@poormans@ordinal{@itemi} \\itemtyperefname \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@@itemii",
    "\\lx@poormans@ordinal{@itemii} \\itemtyperefname \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@@itemiii",
    "\\lx@poormans@ordinal{@itemiii} \\itemtyperefname \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@@itemiv",
    "\\lx@poormans@ordinal{@itemiv} \\itemtyperefname \\itemcontext"
  );
  //----------------------------------------------------------------------
  // Basic enumeration bits

  // Class file should have
  //  NewCounter for enumi,...,
  //  define \labelenumi,... and probably \p@enumii...
  NewCounter!("enumi",   "", idwithin => "@itemizei", idprefix => "i");
  NewCounter!("enumii",  "", idwithin => "enumi",     idprefix => "i");
  NewCounter!("enumiii", "", idwithin => "enumii",    idprefix => "i");
  NewCounter!("enumiv",  "", idwithin => "enumiii",   idprefix => "i");
  NewCounter!("enumv",   "", idwithin => "enumiv",    idprefix => "i"); // A couple of extra
  NewCounter!("enumvi",  "", idwithin => "enumv",     idprefix => "i");

  // How the refnums look... (probably should be in class file, but already here)
  DefMacro!("\\p@enumii", "\\theenumi");
  DefMacro!("\\p@enumiii", "\\theenumi(\\theenumii)");
  DefMacro!("\\p@enumiv", "\\p@enumii\\theenumiii");

  // Formatting of item tags (probably should be in the class file, but already here)
  DefMacro!("\\labelenumi", "\\theenumi.");
  DefMacro!("\\labelenumii", "(\\theenumii)");
  DefMacro!("\\labelenumiii", "\\theenumiii.");
  DefMacro!("\\labelenumiv", "\\theenumiv.");

  // These hookup latexml"s tagging to normal latex"s \labelenummi...
  DefMacro!("\\fnum@enumi", "{\\makelabel{\\labelenumi}}");
  DefMacro!("\\fnum@enumii", "{\\makelabel{\\labelenumii}}");
  DefMacro!("\\fnum@enumiii", "{\\makelabel{\\labelenumiii}}");
  DefMacro!("\\fnum@enumiv", "{\\makelabel{\\labelenumiv}}");

  // These define the typerefnum form, for out-of-context \ref's
  DefMacro!("\\enumtyperefname", "item");
  DefMacro!(
    "\\typerefnum@enumi",
    "\\enumtyperefname~\\p@enumi\\theenumi \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@enumii",
    "\\enumtyperefname~\\p@enumii\\theenumii \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@enumiii",
    "\\enumtyperefname~\\p@enumiii\\theenumiii \\itemcontext"
  );
  DefMacro!(
    "\\typerefnum@enumiv",
    "\\enumtyperefname~\\p@enumiv\\theenumiv \\itemcontext"
  );

  //----------------------------------------------------------------------
  // Basic description list bits
  // Fake counter for itemize to give id"s to ltx:item.
  NewCounter!("@desci",   "", idwithin => "@itemizei", idprefix => "i");
  NewCounter!("@descii",  "", idwithin => "@desci",    idprefix => "i");
  NewCounter!("@desciii", "", idwithin => "@descii",   idprefix => "i");
  NewCounter!("@desciv",  "", idwithin => "@desciii",  idprefix => "i");
  NewCounter!("@descv",   "", idwithin => "@desciv",   idprefix => "i");
  NewCounter!("@descvi",  "", idwithin => "@descv",    idprefix => "i");
  // No refnum"s here, either
  DefMacro!("\\the@desci", "");
  DefMacro!("\\the@descii", "");
  DefMacro!("\\the@desciii", "");
  DefMacro!("\\the@desciv", "");
  DefMacro!("\\the@descv", "");
  DefMacro!("\\the@descvi", "");
  // These hookup latexml"s numbering to normal latex"s
  // Umm.... but they"re not normally used, since \item usually gets an argument!
  DefMacro!("\\descriptionlabel{}", "\\normalfont\\bfseries #1");
  DefMacro!("\\fnum@@desci", "{\\descriptionlabel{}}");
  DefMacro!("\\fnum@@descii", "{\\descriptionlabel{}}");
  DefMacro!("\\fnum@@desciii", "{\\descriptionlabel{}}");
  DefMacro!("\\fnum@@desciv", "{\\descriptionlabel{}}");

  DefMacro!("\\desctyperefname", "item");

  // Blech
  for lvl in &[
    "@itemi", "@itemii", "@itemiii", "@itemiv", "@itemv", "@itemvi",
  ] {
    DefMacro!(T_CS!(s!("\\{}name", lvl)), None, T_CS!("\\itemtyperefname"));
  }
  for lvl in &["enumi", "enumii", "enumiii", "enumiv"] {
    DefMacro!(T_CS!(s!("\\{}name", lvl)), None, T_CS!("\\enumtyperefname"));
  }
  for lvl in &[
    "@desci", "@descii", "@desciii", "@desciv", "@descv", "@descvi",
  ] {
    DefMacro!(T_CS!(s!("\\{}name", lvl)), None, T_CS!("\\desctyperefname"));
  }


  //======================================================================
  // C.6.3 The list and trivlist environments.
  //======================================================================
  // Generic lists are given a way to format the item label, and presumably
  // a counter.

  DefConditional!("\\if@nmbrlist");
  DefMacro!("\\@listctr", "");
  DefPrimitive!("\\usecounter{}", sub[(counter)] {
    let counter = Expand!(counter).to_string();
    let counter_opt = if counter.is_empty() { None } else { Some(counter.as_str()) };
    begin_itemize("list", counter_opt, BeginItemizeOptions {
      nolevel: !counter.is_empty(),
      ..BeginItemizeOptions::default() })?;
  });

  DefMacro!(
    r"\list{}{}",
    r"\let\@listctr\@empty#2\ifx\@listctr\@empty\usecounter{}\fi\expandafter\def\csname fnum@\@listctr\endcsname{#1}\lx@list"
  );
  DefMacro!("\\endlist", "\\endlx@list");

  // Start an anonymous list (often misused)
  DefConstructor!("\\lx@list",
    "<ltx:itemize>",
    before_digest => { bgroup(); });
  // Close the anonymous list if we're still within one.
  DefConstructor!("\\endlx@list", sub[document] {
    document.maybe_close_element("ltx:itemize")?; },
    before_digest => { egroup()?; });

  DefConstructor!("\\list@item OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[args] {
      let undigested = args[0].as_ref().map(|d| d.raw_tokens()).unwrap_or_default();
      ref_step_item_counter(undigested) }
  );

  // Perl latex_constructs.pool.ltxml L1720-1726:
  //   DefConstructor('\trivlist', "<ltx:itemize _autoclose='1'>", mode=>internal_vertical, …);
  //   DefConstructor('\endtrivlist', sub { maybeCloseElement('ltx:itemize') }, beforeDigest=>Digest('\par'));
  // The `\endtrivlist` is an *idempotent* closer — `maybeCloseElement` is a
  // no-op when the element is already closed. That matters when user code
  // calls `\endtrivlist` directly (e.g. arxiv 0908.0398's `\cqfd → …\endtrivlist`),
  // then `\end{proof}` closes the outer trivlist, then `\end{proof}`'s own
  // `\endproof → \endtrivlist` fires again. Perl swallows the double-close;
  // Rust's previous DefEnvironment emitted a strict env-frame closer that
  // errored on the second call.
  DefConstructor!("\\trivlist",
    "<ltx:itemize _autoclose='1'>",
    mode => "internal_vertical",
    properties => {
      begin_itemize("trivlist", None, BeginItemizeOptions::default())?
    }
  );
  DefConstructor!("\\endtrivlist",
    sub[document, _args, _props] {
      document.maybe_close_element("ltx:itemize")?;
    },
    before_digest => { Digest!("\\par")?; }
  );

  DefMacro!("\\trivlist@item", "\\preitem@par\\trivlist@item@");
  DefConstructor!("\\trivlist@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'><ltx:tags><ltx:tag>#tag</ltx:tag></ltx:tags>",
    // At least an empty tag! ?
    properties => sub[args] {
      if let Some(ref arg) = args[0] {
        if let DigestedData::Postponed(ref tag_tokens) = arg.data() {
          let tag_expanded = Expand!(tag_tokens.clone());
          let tag = stomach::digest(tag_expanded)?;
          Ok(stored_map!("tag" => tag))
        } else {
          Ok(SymHashMap::default())
        }
      } else {
          Ok(SymHashMap::default())
      }
    }
  );

  DefRegister!("\\topsep"             => Glue::new(0));
  DefRegister!("\\partopsep"          => Glue::new(0));
  DefRegister!("\\lx@default@itemsep" => Glue::new(0));
  DefRegister!("\\itemsep"            => Glue::new(0));
  DefRegister!("\\parsep"             => Glue::new(0));
  DefRegister!("\\@topsep"            => Glue::new(0));
  DefRegister!("\\@topsepadd"         => Glue::new(0));
  DefRegister!("\\@outerparskip"      => Glue::new(0));
  DefRegister!("\\leftmargin"         => Dimension::new(0));
  DefRegister!("\\rightmargin"        => Dimension::new(0));
  DefRegister!("\\listparindent"      => Dimension::new(0));
  DefRegister!("\\itemindent"         => Dimension::new(0));
  DefRegister!("\\labelwidth"         => Dimension::new(0));
  DefRegister!("\\labelsep"           => Dimension::new(0));
  DefRegister!("\\@totalleftmargin"   => Dimension::new(0));
  DefRegister!("\\leftmargini"        => Dimension::new(0));
  DefRegister!("\\leftmarginii"       => Dimension::new(0));
  DefRegister!("\\leftmarginiii"      => Dimension::new(0));
  DefRegister!("\\leftmarginiv"       => Dimension::new(0));
  DefRegister!("\\leftmarginv"        => Dimension::new(0));
  DefRegister!("\\leftmarginvi"       => Dimension::new(0));
  DefRegister!("\\@listdepth"         => Number::new(0));
  DefRegister!("\\@itempenalty"       => Number::new(0));
  DefRegister!("\\@beginparpenalty"   => Number::new(0));
  DefRegister!("\\@endparpenalty"     => Number::new(0));
  DefRegister!("\\labelwidthi"        => Dimension::new(0));
  DefRegister!("\\labelwidthii"       => Dimension::new(0));
  DefRegister!("\\labelwidthiii"      => Dimension::new(0));
  DefRegister!("\\labelwidthiv"       => Dimension::new(0));
  DefRegister!("\\labelwidthv"        => Dimension::new(0));
  DefRegister!("\\labelwidthvi"       => Dimension::new(0));

  DefRegister!("\\@itemdepth" => Number::new(0));
  DefRegister!("\\@maxlistdepth" => Number::new(6));

  // List formatting macros from article.cls / report.cls / book.cls
  // These set list parameters at various nesting levels.
  // In raw TeX classes, \@listi etc. are defined by the class file.
  // We stub them as no-ops since LaTeXML handles list formatting via CSS.
  DefMacro!("\\@listi", "");
  DefMacro!("\\@listii", "");
  DefMacro!("\\@listiii", "");
  DefMacro!("\\@listiv", "");
  DefMacro!("\\@listv", "");
  DefMacro!("\\@listvi", "");

  //======================================================================
  // C.6.4 Verbatim
  //======================================================================
  // NOTE: how's the best way to get verbatim material through?
  // DefEnvironment!("{verbatim}", "<ltx:verbatim>#body</ltx:verbatim>");
  // DefEnvironment!("{verbatim*}", "<ltx:verbatim>#body</ltx:verbatim>");

  DefMacro!(
    "\\@verbatim",
    r"\par\aftergroup\lx@end@verbatim\lx@@verbatim"
  ); // Close enough?
  // Perl latex_constructs.pool.ltxml L1774-1782: enterHorizontal => 1 + beforeDigest.
  DefConstructor!("\\lx@@verbatim", "<ltx:verbatim font='#font'>",
  enter_horizontal => true,
  before_digest => {
    begin_semiverbatim(Some(&SEMIVERBATIM_CHARS));
    merge_font(fontmap!(family => "typewriter", series => "medium", shape => "upright"));
    assign_catcode(' ', Catcode::ACTIVE, None);  // Do NOT (necessarily) skip spaces after \verb!!!
    Let!(&T_ACTIVE!(' '), T_SPACE!());
  });
  DefConstructor!(r"\lx@end@verbatim", "</ltx:verbatim>",
    before_digest => { end_semiverbatim()?; });

  // verbatim is a bit of special case;
  // It looks like an environment, but it only ends with an explicit "\end{verbatim}" on it's own line.
  // So, we'll end up doing things more manually.
  // We're going to sidestep the Gullet for inputting,
  // and also the usual environment capture.
  DefConstructor!(T_CS!("\\begin{verbatim}"), None, 
    "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    before_digest => { before_digest_verbatim() }
    after_digest => sub[whatsit] { after_digest_verbatim(false, whatsit)?; },
    before_construct => sub[document, _whatsit] {
      document.maybe_close_element("ltx:p")?; }
  );
  DefConstructor!(T_CS!("\\begin{verbatim*}"), None, 
    "<ltx:verbatim font='#font'>#body</ltx:verbatim>",
    before_digest => { before_digest_verbatim() }
    after_digest => sub[whatsit] { after_digest_verbatim(true, whatsit)?; },
    before_construct => sub[document, _whatsit] {
      document.maybe_close_element("ltx:p")?; }
  );

  // Perl latex_constructs.pool.ltxml L1847 — re-let `\nobreakspace`
  // to LaTeXML's `\lx@nobreakspace` (= NBSP `\u{00A0}`). Required HERE
  // (not just plain_base.rs) so the override survives `LoadFormat`'s
  // dump path: the dump captures latex.ltx's
  // `\nobreakspace → \protect\nobreakspace<sp>` chain which decays to a
  // regular space + `\leavevmode\nobreak\<sp>`. Without this Let, the
  // hyperref autoref wrapping `\sectionautorefname\nobreakspace\thesection`
  // produced `section 1` (regular space) instead of `section\u{00A0}1`.
  Let!("\\nobreakspace", "\\lx@nobreakspace");

  DefPrimitive!("\\@vobeyspaces", {
    AssignCatcode!(' ', Catcode::ACTIVE);
    Let!(&T_ACTIVE!(' '), T_CS!("\\nobreakspace"));
  });
  DefMacro!("\\@xobeysp", "\\nobreakspace");

  // WARNING: Need to be careful about what catcodes are active here
  // And clearly separate expansion from digestion
  DefMacro!("\\verb", {
   begin_semiverbatim(Some(&SEMIVERBATIM_CHARS));
    // Do NOT (necessarily) skip spaces after \verb!!!
    assign_catcode(' ', Catcode::ACTIVE, None);
    let mut init = None;
    let mut skipped_space = false;
    // As of texlive 2021, DO skip spaces before delimiter (even tho we've changed catcodes)
    // but if we do skip spaces, * can be the delimiter
    let space_sym = arena::pin_static(" ");
    while let Some(maybe_init) = gullet::read_token()? {
      if maybe_init.get_sym() == space_sym {
        skipped_space = true;
      } else {
        init = Some(maybe_init);
        break;
      }
    }
    let mut starred = false;
    if let Some(ref init_token) = init {
      if *init_token == T_OTHER!("*") && !skipped_space {
        starred = true;
        while let Some(maybe_init) =  gullet::read_token()? {
          if maybe_init.get_sym() != space_sym {
            init = Some(maybe_init);
            break;
          }
        }
      }
    }
    if let Some(init_token) = init {
      let init_ch = init_token.with_str(|is| is.chars().next().unwrap());
      assign_catcode(init_ch, Catcode::ACTIVE, None);
      let delim = Tokens!(T_ACTIVE!(init_ch));
      let body = gullet::read_until(&delim)?;
      end_semiverbatim()?;

      let mut result = vec![T_CS!("\\lx@hidden@bgroup")];
      if starred {
        result.push(T_CS!("\\lx@use@visiblespace"));
      }
      result.extend(Invocation!(T_CS!("\\@internal@verb"), vec![
        if starred { Tokens!(T_OTHER!("*")) } else { Tokens!() },
        Tokens!(init_token),
        body
      ]).unlist());
      result.push(T_CS!("\\lx@hidden@egroup"));
      Ok(Tokens::new(result))
    } else { // typically something read too far got \verb and the content is somewhere else..?
      Error!("expected", "delimiter",
        "Verbatim argument lost\n Bindings for preceding code is probably broken");
      end_semiverbatim()?;
      Ok(Tokens!())
    }
  });

  DefPrimitive!("\\lx@use@visiblespace", {
    // Do NOT (necessarily) skip spaces after \verb!!!
    assign_catcode(' ', Catcode::ACTIVE, None);
    // Visible space
    Let!(&T_ACTIVE!(' '), T_OTHER!("\u{2423}"));
  });

  // Arrange to digest the body in text mode, to keep (eg) "_" from turning to "\_"
  DefMacro!("\\@internal@verb{}{}{}",
      r"\ifmmode\@internal@math@verb{#1}{#2}{#3}\else\@internal@text@verb{#1}{#2}{#3}\fi");
  DefConstructor!("\\@internal@math@verb{} Undigested {}",
    "<ltx:XMTok font='#font'>#3</ltx:XMTok>",
    mode      => "text",
    enter_horizontal => true,
    font      => { family => "typewriter", series => "medium", shape => "upright" },
    reversion => "\\verb#1#2#3#2");
  DefConstructor!("\\@internal@text@verb{} Undigested {}",
    "<ltx:verbatim font='#font'>#3</ltx:verbatim>",
    font            => { family => "typewriter", series => "medium", shape => "upright" },
    enter_horizontal => true,
    before_construct => sub[doc,_whatsit] {
      if !document::can_contain(doc.get_element().as_ref().unwrap(), "#PCDATA") {
        doc.open_element("ltx:p", None, None)?;
      }
    },
    reversion => "\\verb#1#2#3#2");


  // Actually, latex sets catcode to 13 ... is this close enough?
  DefPrimitive!("\\obeycr", {
    AssignValue!("PRESERVE_NEWLINES", 1);
  });
  DefPrimitive!("\\restorecr", {
    AssignValue!("PRESERVE_NEWLINES", 0);
  });
  DefMacro!(T_CS!("\\normalsfcodes"), None, Tokens!());


  // ======================================================================
  // C.7 Mathematical Formulas
  // ======================================================================


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
    if state::lookup_bool_sym(pin!("IN_MATH")) {
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

  // Perl: latex_constructs.pool.ltxml lines 2323-2329
  // \lx@eqnarray@label wraps the label in \lx@hidden@noalign so it's processed
  // at the row level, not inside a cell. This is critical because in align-like
  // environments, a cell containing only \label is skippable (its content is not
  // absorbed during beAbsorbed), so the \label constructor would never run.
  // By routing through noalign, the \label constructor runs at the equation level
  // where float_to_label can find the ltx:equation parent.
  DefMacro!("\\lx@eqnarray@label Semiverbatim",
    "\\lx@hidden@noalign{\\lx@eqnarray@save@label{#1}}");

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

  // Perl: latex_constructs.pool.ltxml L1971-1973
  NewCounter!("subequation", "equation", idprefix => "E", idwithin => "equation");
  DefMacro!("\\thesubequation", "\\theequation\\alph{subequation}");
  DefMacro!("\\fnum@subequation", "(\\thesubequation)");

  // Perl: latex_constructs.pool.ltxml L2174-2191
  // \lx@equationgroup@subnumbering@begin/end — subequation numbering
  DefConstructor!("\\lx@equationgroup@subnumbering@begin",
    "<ltx:equationgroup xml:id='#id'>#tags",
    after_digest => sub[whatsit] {
      use latexml_core::binding::counter::dialect::reset_counter;
      use latexml_core::mouth;
      // Step the equation counter and get properties (id, refnum, tags)
      let eqn_props = ref_step_counter("equation", false)?;
      // Expand \theequation to get the parent equation number text
      let eqnum_toks = gullet::do_expand(T_CS!("\\theequation"))?;
      let eqnum_str = eqnum_toks.to_string();
      // Save current equation counter value
      let saved = state::lookup_register("\\c@equation", Vec::new())?.map_or(0, |rv| {
        match rv {
          RegisterValue::Number(n) => n.0,
          _ => 0,
        }
      });
      state::assign_value("SAVED_EQUATION_NUMBER", Stored::Number(Number::new(saved)), None);
      // Set properties on the whatsit
      for (k, v) in eqn_props {
        arena::with(k, |ks| whatsit.set_property(ks, v));
      }
      // Reset equation counter to 0
      reset_counter(&T_OTHER!("equation"))?;
      // Redefine \theequation to parent_number + \alph{equation}
      let new_theequation = format!("{}\\alph{{equation}}", eqnum_str);
      def_macro(T_CS!("\\theequation"), None, mouth::tokenize_internal(&new_theequation), None)?;
      // Redefine \theequation@ID for xml:id generation
      if let Some(id_val) = whatsit.get_property("id") {
        let id_str = match &*id_val {
          Stored::String(s) => arena::to_string(*s),
          other => other.to_string(),
        };
        let new_id_macro = format!("{}.\\@equation@ID", id_str);
        def_macro(T_CS!("\\theequation@ID"), None, mouth::tokenize_internal(&new_id_macro), None)?;
      }
    });
  Tag!("ltx:equationgroup", auto_close => true);
  DefConstructor!("\\lx@equationgroup@subnumbering@end",
    sub[document, _args, _props] {
      document.maybe_close_element("ltx:equationgroup")?;
    },
    after_digest => {
      // Restore the saved equation counter
      if let Some(saved) = state::lookup_value("SAVED_EQUATION_NUMBER") {
        let n = match saved {
          Stored::Number(n) => n.0,
          _ => 0,
        };
        state::assign_register(
          "\\c@equation",
          Number::new(n).into(),
          Some(state::Scope::Global),
          Vec::new(),
        )?;
      }
    });

  // Perl: latex_constructs.pool.ltxml L2142-2163 — automath wrapping
  // Simplified: \ensuremathfollows checks if next content is already math,
  // if not wraps with \( ... \). Used by equation labels / alt text.
  DefMacro!("\\ensuremathfollows", "");  // stub — automath needs gullet lookahead
  DefMacro!("\\ensuremathpreceeds", ""); // stub — pairs with ensuremathfollows

  // Perl: latex_constructs.pool.ltxml L2166
  // Since the arXMLiv folks keep wanting ids on all math, let's try this!
  Tag!("ltx:Math", after_open => sub[document, node] {
    document.generate_id(node, "m")?;
  });


  // \stackrel{over}{base}: places "over" as a superscript over "base" relation
  DefMacro!("\\stackrel{}{}", r"\lx@stackrel{{\scriptstyle #1}}{{#2}}");
  DefConstructor!("\\lx@stackrel{}{}",
    "<ltx:XMApp role='RELOP'>\
      <ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/>\
      <ltx:XMArg>#2</ltx:XMArg>\
      <ltx:XMArg>#1</ltx:XMArg>\
    </ltx:XMApp>",
    reversion => "\\stackrel{#1}{#2}",
    properties => { stored_map!("scriptpos" => s!("mid{}", stomach::get_script_level())) }
  );

  DefConstructor!(
    "\\frac InFractionStyle InFractionStyle",
    "<ltx:XMApp>\
      <ltx:XMTok meaning='divide' role='FRACOP' mathstyle='#mathstyle'/>\
      <ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg>\
      </ltx:XMApp>",
    properties => {
      let ms = lookup_font()
        .and_then(|f| f.get_mathstyle().map(|s| s.to_string()));
      match ms {
        Some(s) => Ok(stored_map!("mathstyle" => s)),
        None => Ok(stored_map!()),
      }
    }
  );


  DefConstructor!("\\mathrm{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "serif", series => "medium", shape => "upright"});
  DefConstructor!("\\mathit{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {shape => "italic", family => "serif", series => "medium"});
  DefConstructor!("\\mathbf{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {series => "bold", family => "serif", shape => "upright"});
  DefConstructor!("\\mathsf{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "sansserif", series => "medium", shape => "upright"});
  DefConstructor!("\\mathtt{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "typewriter", series => "medium", shape => "upright"});
  DefConstructor!("\\mathcal{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "caligraphic", series => "medium", shape => "upright"});
  DefConstructor!("\\mathscr{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "script", series => "medium", shape => "upright"});
  DefConstructor!("\\mathnormal{}", "#1", bounded => true, require_math => true,
    locked => true,
    font => {family => "math", shape => "italic", series => "medium"});

  DefMacro!("\\fontsubfuzz", ".4pt");
  DefMacro!("\\oldstylenums", "");

  DefPrimitive!("\\operator@font", None,
    font => {family => "serif", series => "medium", shape => "upright"});


  // ======================================================================
  // C.8 Definitions, Numbering and Programming
  // ======================================================================


  //**********************************************************************
  // C.8 Definitions, Numbering and Programming
  //**********************************************************************

  //======================================================================
  // C.8.1 Defining Commands
  //======================================================================

  DefMacro!("\\@tabacckludge {}", "\\csname\\string#1\\endcsname");
  // latex.ltx L10007 — `\let\a=\@tabacckludge`. The dump carries
  // `\a` as an E record (the serializer captured
  // `\@tabacckludge`'s body with a `\@changed@cmd` wrapper, not a
  // pure Let-alias), which the outer M-gate rejects as a
  // public-CS Expandable (expl3-cascade safety). Neither the
  // PA/MPA gate relaxation nor the deferred-alias retry pass in
  // `dump_reader.rs` applies, so we keep this alias hand-written
  // to match latex.ltx source. Inside a `tabbing` environment,
  // tabbing_bindings() overrides this local to `\@tabbing@accent`.
  // Found in arxiv 1611.05395.
  Let!("\\a", "\\@tabacckludge");

  DefPrimitive!("\\newcommand OptionalMatch:* DefToken [Number][]{}",
  sub[(_star,cs_token,nargs,opt,body)] {
    let nargs = nargs.value_of() as usize;
    if !IsDefinable!(&cs_token) {
      if !has_value(&s!("{}:locked", cs_token.to_string())) { // not locked, inform.
        let message = s!("Ignoring redefinition (\\newcommand) of {}", cs_token.stringify());
        Info!("ignore", cs_token, message);
      }
      return Ok(vec![]);
    }
    let macro_args = convert_latex_args(nargs, opt)?;
    DefMacro!(cs_token, macro_args, body);
  });

  DefPrimitive!("\\renewcommand OptionalMatch:* DefToken [Number][]{}",
  sub[(_star, cs, nargs_num, opt, body)] {
    let nargs = nargs_num.value_of() as usize;
    let macro_args = convert_latex_args(nargs, opt)?;
    DefMacro!(cs, macro_args, body);
  });

  // low-level implementation of both \newcommand and \renewcommand depends on \@argdef
  // and robustness upgrades are often realized via redefining \l@ngrel@x
  // Perl latex_constructs.pool.ltxml L2591-2604
  DefPrimitive!("\\@argdef DefToken [Number]{}", sub[(cs, nargs, body)] {
    let macro_args = convert_latex_args(nargs.value_of() as usize, None)?;
    DefMacro!(cs, macro_args, body);
  });
  DefPrimitive!("\\@xargdef DefToken [Number][]{}", sub[(cs, nargs, opt, body)] {
    let macro_args = convert_latex_args(nargs.value_of() as usize, opt)?;
    DefMacro!(cs, macro_args, body);
  });
  // Perl L2597-2602: \@yargdef checks if arg2 equals \tw@ (2) for optional arg type
  DefPrimitive!("\\@yargdef DefToken DefToken {}{}", sub[(cs, type_tok, nargs_toks, body)] {
    let nargs_str = nargs_toks.to_string();
    let nargs: usize = nargs_str.trim().parse().unwrap_or(0);
    let has_optional = type_tok.with_str(|s| s.contains("2"))
      || state::x_equals(&type_tok, &T_CS!("\\tw@"));
    let opt = if has_optional { Some(Tokens!()) } else { None };
    let macro_args = convert_latex_args(nargs, opt)?;
    DefMacro!(cs, macro_args, body);
  });
  DefPrimitive!("\\@reargdef DefToken [Number]{}", sub[(cs, nargs, body)] {
    let macro_args = convert_latex_args(nargs.value_of() as usize, None)?;
    DefMacro!(cs, macro_args, body);
  });

  DefPrimitive!("\\providecommand OptionalMatch:* DefToken [Number][]{}",
  sub[(_star, cs, nargs, opt, body)] {
    if IsDefinable!(&cs) {
      let nargs = nargs.value_of() as usize;
      let cs_args = convert_latex_args(nargs, opt)?;
      DefMacro!(cs, cs_args, body);
    }
  });

  // Crazy; define \cs in terms of \cs[space] !!!
  DefPrimitive!("\\DeclareRobustCommand OptionalMatch:* SkipSpaces DefToken [Number][]{}",
  sub[(_star,cs,nargs,opt,body)] {
    let nargs = nargs.value_of() as usize;
    let cs_args = convert_latex_args(nargs, opt)?;
    DefMacro!(cs, cs_args, body, robust => true);
  });

  DefPrimitive!("\\MakeRobust DefToken", sub[(cs)] {
    let mungedcs = T_CS!(cs.with_str(|cstr| s!("{cstr} ")));
    // only if defined but not yet robust
    if LookupDefinition!(&cs).is_some() &&
       LookupDefinition!(&mungedcs).is_none() {
      Let!(&mungedcs, &cs);
      DefMacro!(cs, None, Tokens!(T_CS!("\\protect"),mungedcs));
    }
  });

  // \CheckCommand validates but doesn't define — absorb and ignore args
  DefPrimitive!("\\CheckCommand OptionalMatch:* SkipSpaces DefToken [Number][]{}", None);
  // Font encoding subset declaration — ignored in our context
  DefPrimitive!("\\DeclareEncodingSubset{}{}{}", None);

  //------------------------------------------------------------
  // The following commands define encoding-specific expansions
  // or glyphs.  The control-sequence is defined to use the expansion for
  // the current encoding, if any, or the default expansion (for encoding "?").
  // We don't want to redefine control-sequence if it already has a definition:
  // It may be that we've already defined it to expand into the above conditional.
  // But more importantly, we don't want to override a hand-written definition (if any).
  //------------------------------------------------------------
  DefPrimitive!("\\DeclareTextCommand DefToken {}[Number][]{}",
  sub[(cs, encoding, nargs, opts, expansion)] {
    let cs_str = cs.to_string();
    let nargs = nargs.value_of() as usize;
    let encoding_str = Expand!(encoding).to_string();
    let ecs = T_CS!(s!("\\{encoding_str}{cs_str}"));
    let ecs_args = convert_latex_args(nargs, opts.clone())?;
    DefMacro!(ecs, ecs_args, expansion.clone());
    if !IsDefined!(&cs) {    // If not already defined...
      let cs_args = convert_latex_args(nargs, opts)?;
      DefMacro!(cs, cs_args, expansion);
    }
  });

  DefMacro!(
    "\\DeclareTextCommandDefault DefToken",
    "\\DeclareTextCommand{#1}{?}"
  );

  DefPrimitive!("\\ProvideTextCommand DefToken {}[Number][]{}",
  sub[(cs, encoding, nargs, opts, expansion)] {
    let cs_str = cs.to_string();
    let nargs = nargs.value_of() as usize;
    let encoding_str = Expand!(encoding).to_string();
    let ecs = T_CS!(s!("\\{encoding_str}{cs_str}"));
    if !IsDefined!(&ecs) { // If not already defined...
      let ecs_args = convert_latex_args(nargs, opts.clone())?;
      DefMacro!(ecs, ecs_args, expansion.clone());
    }
    if IsDefinable!(&cs) { // If not already defined...
      // Define base command: use encoding-specific expansion directly
      let cs_args = convert_latex_args(nargs, opts)?;
      DefMacro!(cs, cs_args, expansion);
    }
  });

  DefMacro!(
    "\\ProvideTextCommandDefault DefToken",
    "\\ProvideTextCommand{#1}{?}"
  );

  // #------------------------------------------------------------

  DefPrimitive!("\\DeclareTextSymbol DefToken {}{Number}", sub[(cs, encoding, code)] {
    let code_value = code.value_of() as u8;
    let cs_str = cs.to_string();
    let encoding_str = Expand!(encoding).to_string();
    let ecs = T_CS!(s!("\\{encoding_str}{cs_str}"));
    if let Some(replacement_value) = font::decode(code_value, Some(encoding_str), false) {
      // Define the encoding-specific command
      def_primitive(ecs, None, Some(PrimitiveBody::from(replacement_value)),
        PrimitiveOptions::default())?;
      // Also define the base command directly if not already defined
      if IsDefinable!(&cs) {
        def_primitive(cs, None, Some(PrimitiveBody::from(replacement_value)),
          PrimitiveOptions::default())?;
      }
    } else if IsDefinable!(&cs) {
      // Can't decode: define conditional fallback
      DefMacro!(cs, None, Some(s!(r"\expandafter\ifx\csname\cf@encoding\string{cs_str}\endcsname\relax
      \csname?\string{cs_str}\endcsname\else\csname\cf@encoding\string{cs_str}\endcsname\fi").into()));
    }
  });

  // hmmm... what needs doing here; basically it means use this encoding as the default for the
  // symbol
  // Perl L2683: DefPrimitive('\DeclareTextSymbolDefault DefToken {}', sub { DefMacroI(...) })
  // Kind-parity stub — Perl performs a DefMacroI side-effect registering a
  // `\?<cs>` → `\<encoding><cs>` alias. Current Rust engine doesn't rely on
  // that alias, so body stays None; matches Perl's stomach-level
  // invocation kind.
  DefPrimitive!("\\DeclareTextSymbolDefault DefToken {}", None);

  //------------------------------------------------------------
  DefPrimitive!("\\DeclareTextAccent DefToken {}{}", None);
  DefPrimitive!("\\DeclareTextAccentDefault{}{}", None);

  DefMacro!("\\fontencoding{}", "\\lx@fontencoding{#1}");
  DefMacro!("\\f@encoding", {
    ExplodeText!(LookupFont!().unwrap().get_encoding().unwrap())
  });
  DefMacro!("\\cf@encoding", {
    ExplodeText!(LookupFont!().unwrap().get_encoding().unwrap())
  });

  // #------------------------------------------------------------
  DefPrimitive!("\\DeclareTextComposite{}{}{}{}", None);
  // sub { ignoredDefinition("DeclareTextComposite", $_[1]); });
  DefPrimitive!("\\DeclareTextCompositeCommand{}{}{}{}", None);
  // sub { ignoredDefinition("DeclareTextCompositeCommand", $_[1]); });

  DefPrimitive!("\\UndeclareTextCommand{}{}", None);
  DefMacro!("\\UseTextSymbol{}{}", "{\\fontencoding{#1}#2}");
  DefMacro!("\\UseTextAccent{}{}", "{\\fontencoding{#1}#2{#3}}");

  // Perl: DefPrimitive('\DeclareMathAccent DefToken {}{} {Number}', ...)
  // latex_constructs.pool.ltxml:2702-2709. Perl always calls DefMathI even
  // when FontDecode returns undef (DefMathI normalizes `$presentation = ''`
  // when undef, Package.pm:1609). Earlier Rust skipped def_math when glyph
  // is None — that left the CS undefined for unknown encodings (e.g.
  // `\DeclareMathAccent{\widecheck}{\mathalpha}{mathx}{"71}` with no
  // mathx font map → \widecheck undefined → 1806.02506-style 1-error
  // cluster). Mirror Perl: always install, fall back to empty presentation.
  DefPrimitive!("\\DeclareMathAccent DefToken {}{} {Number}",
  sub[(cs, kind, class, code)] {
    let class_str = class.to_string();
    let encoding = lookup_value(&s!("fontdeclaration@{}", class_str))
      .and_then(|v| if let Stored::Font(ref f) = v { f.get_encoding().map(|e| e.to_string()) } else { None })
      .unwrap_or(class_str);
    let (glyph, _font) = font_decode(code.value_of() as i32, Some(&encoding), None);
    let presentation = glyph.map(|c| c.to_string()).unwrap_or_default();
    let paramlist = parse_parameters("Digested", &cs, true)?;
    let opts = MathPrimitiveOptions{
      operator_role: Some("OVERACCENT".to_string()),
      ..Default::default()};
    def_math(cs, paramlist, presentation, opts)?;
    // Perl: return AddToPreamble('\DeclareMathAccent', $cs, $kind, $class, $code);
    // AddToPreamble returns Digest(Invocation(\lx@add@Preamble@PI, Invocation(\DeclareMathAccent, ...)))
    // The primitive must RETURN this digested result so it gets absorbed by the document.
    let preamble_text = format!("\\DeclareMathAccent{}{{{}}}{{{}}}{{{}}}",
      cs.with_str(|s| s.to_string()), kind, class, code.value_of());
    let preamble_toks = build_invocation(
      T_CS!("\\lx@add@Preamble@PI"),
      vec![Some(Tokens::new(Explode!(preamble_text)))])?;
    let digested = stomach::digest(preamble_toks)?;
    Ok(vec![digested])
  });

  // Perl: DefPrimitive('\DeclareMathSymbol DefToken SkipSpaces DefToken {}{Number}', ...)
  // my $symboltype_roles = { '\mathord' => 'ID', '\mathop' => 'BIGOP', '\mathbin' => 'BINOP',
  //   '\mathrel' => 'RELOP', '\mathopen' => 'OPEN', '\mathclose' => 'CLOSE', '\mathpunct' => 'PUNCT' };
  // locked: prevents raw TeX dump from overriding with version that errors on redefinition
  DefPrimitive!("\\DeclareMathSymbol DefToken SkipSpaces DefToken {}{Number}",
  sub[(cs, sym_type, fontkind, code)] {
    let mut encoding = fontkind.to_string();
    if let Some(Stored::Font(ref decl)) = lookup_value(&s!("fontdeclaration@{}", encoding)) {
      if let Some(enc) = decl.get_encoding() {
        encoding = enc.to_string();
      }
    }
    let (glyph, _font) = font_decode(code.value_of() as i32, Some(&encoding), None);
    let role = match sym_type.to_string().as_str() {
      "\\mathord"  => Some("ID"),
      "\\mathop"   => Some("BIGOP"),
      "\\mathbin"  => Some("BINOP"),
      "\\mathrel"  => Some("RELOP"),
      "\\mathopen" => Some("OPEN"),
      "\\mathclose"=> Some("CLOSE"),
      "\\mathpunct"=> Some("PUNCT"),
      _ => None,
    };
    // Perl Package.pm L2761: `DefMathI($cs, undef, $glyph, role => $role)` —
    // called unconditionally, even when FontDecode returns `undef` (e.g. the
    // encoding's `.fontmap.ltxml` isn't shipped with LaTeXML, like "U").
    // Fall back to the raw codepoint so the CS is defined; better to render
    // an ASCII stand-in than to cascade into Error:undefined for the
    // command itself. arxiv 1011.1955 hits this with
    //   \DeclareSymbolFont{AMSb}{U}{msb}{m}{n}
    //   \DeclareMathSymbol{\Z}{\mathalpha}{AMSb}{"5A}
    // where no u.fontmap exists.
    let presentation = match glyph {
      Some(ch) => ch.to_string(),
      None => {
        let codepoint = code.value_of() as u32;
        char::from_u32(codepoint).map(|c| c.to_string()).unwrap_or_default()
      },
    };
    let mut opts = MathPrimitiveOptions::default();
    if let Some(r) = role {
      opts.role = Some(r.to_string());
    }
    def_math(cs, None, presentation, opts)?;
  });

  DefPrimitive!("\\DeclareMathDelimiter{}{}{}{}", None);
  DefPrimitive!("\\DeclareMathRadical{}{}{}{}{}", None);
  DefPrimitive!("\\DeclareMathVersion{}", None);
  DefPrimitive!("\\DeclarePreloadSizes{}{}{}{}{}", None);

  // The next font declaration commands are based on
  // http://tex.loria.fr/general/new/fntguide.html
  // we ignore font encoding
  DefPrimitive!("\\DeclareSymbolFont{}{}{}{}{}",
  sub[(name, enc, family, series, shape)] {
    AssignValue!(&s!("fontdeclaration@{}", name),
      fontmap!(family => family.to_string(),
        series   => series.to_string(),
        shape    => shape.to_string(),
        encoding => enc.to_string()
      )
    );
  });
  DefPrimitive!("\\DeclareSymbolFontAlphabet {Token} {}", sub[(cs, name)] {
    let fontkey = s!("fontdeclarations@{}", name.to_string());
    let font : Option<Font> = if let Some(Stored::Font(value)) = lookup_value(&fontkey) {
      Some((*value).clone())
    } else {
      None
    };
    DefPrimitive!(cs, None, None, font => font);
  });

  DefPrimitive!("\\DeclareFixedFont{}{}{}{}{}{}", None);
  DefPrimitive!("\\DeclareErrorFont{}{}{}{}{}", None);
  // Font declaration stubs (Perl latex_constructs.pool.ltxml)
  DefPrimitive!("\\DeclareFontShape{}{}{}{}{}{}", None);
  DefPrimitive!("\\DeclareFontFamily{}{}{}", None);
  DefPrimitive!("\\DeclareSizeFunction{}{}", None);
  DefPrimitive!("\\DeclareMathSizes{}{}{}{}", None);
  DefMacro!("\\newmathalphabet{}{}{}", None, None);
  // DeclareMathAlphabet: define math font command if not already defined
  DefPrimitive!("\\DeclareMathAlphabet{}{}{}{}{}", sub[(cs, _enc, family, series, shape)] {
    let cs_tok = T_CS!(cs.to_string());
    if !IsDefined!(&cs_tok) {
      let font : Option<Font> = Some(fontmap!(
        family => family.to_string(),
        series => series.to_string(),
        shape  => shape.to_string()
      ));
      DefPrimitive!(cs_tok, None, None, font => font);
    }
  });

  DefMacro!("\\cdp@list", "\\@empty");
  Let!("\\cdp@elt", "\\relax");
  DefPrimitive!("\\DeclareFontEncoding{}{}{}", sub[(encoding, x, y)] {
    // Perl: AddToMacro(\cdp@list, \cdp@elt{enc}{family}{series}{shape})
    let cdp_cs = T_CS!("\\cdp@list");
    let enc_toks = encoding.clone().unlist();
    let mut cdp_tokens_vec = vec![T_CS!("\\cdp@elt"), T_BEGIN!()];
    cdp_tokens_vec.extend(enc_toks);
    cdp_tokens_vec.extend(vec![
      T_END!(),
      T_BEGIN!(), T_CS!("\\default@family"), T_END!(),
      T_BEGIN!(), T_CS!("\\default@series"), T_END!(),
      T_BEGIN!(), T_CS!("\\default@shape"), T_END!(),
    ]);
    let cdp_tokens = Tokens::new(cdp_tokens_vec);
    AddToMacro!(cdp_cs, cdp_tokens);

    let e = Expand!(encoding);
    DefMacro!(T_CS!("\\LastDeclaredEncoding"), None, e.clone());
    DefMacro!(T_CS!(s!("\\T@{}", e)), None, x);
    DefMacro!(T_CS!(s!("\\M@{}", e)), None, Tokens!(T_CS!("\\default@M"), y.unlist()));
  });

  DefMacro!("\\LastDeclaredEncoding", None, None);

  // \DeclareUnicodeCharacter — from utf8.def / latex_constructs
  // Maps a hex codepoint to an expansion, making the character active.
  DefPrimitive!("\\DeclareUnicodeCharacter Expanded {}", sub[(hexcode, expansion)] {
    let hex_str = hexcode.to_string();
    let hex_str = hex_str.trim();
    if hex_str.chars().all(|c| c.is_ascii_hexdigit()) && !hex_str.is_empty() {
      if let Ok(cp) = u32::from_str_radix(hex_str, 16) {
        if cp <= 0x10FFFF {
          if let Some(ch) = char::from_u32(cp) {
            AssignCatcode!(ch, Catcode::ACTIVE);
            DefMacro!(T_ACTIVE!(ch), None, expansion);
          }
        } else {
          Error!("unexpected", hex_str,
            s!("{} too large for Unicode. Values between 0 and 10FFFF are permitted.", hex_str));
        }
      }
    } else {
      Error!("unexpected", hex_str,
        s!("Non-hex value {} in \\DeclareUnicodeCharacter", hex_str));
    }
  });

  DefPrimitive!("\\DeclareFontSubstitution{}{}{}{}", None);
  DefPrimitive!("\\DeclareFontEncodingDefaults{}{}", None);
  DefMacro!("\\LastDeclaredEncoding", None, None);

  DefPrimitive!("\\SetSymbolFont{}{}{}{}{}{}", None);
  DefPrimitive!("\\SetMathAlphabet{}{}{}{}{}{}", None);
  DefPrimitive!("\\addtoversion{}{}", None);
  DefPrimitive!("\\TextSymbolUnavailable{}", None);

  // LaTeX3 ltcmd: \NewCommandCopy and \DeclareCommandCopy
  // These are semantic \let equivalents from the 2023+ LaTeX kernel.
  // Not in Perl LaTeXML (too new), but needed for modern packages (tcolorbox, etc.).
  DefPrimitive!("\\NewCommandCopy Token Token", sub[(new_cs, old_cs)] {
    state::let_i(&new_cs, &old_cs, None);
  });
  DefPrimitive!("\\DeclareCommandCopy Token Token", sub[(new_cs, old_cs)] {
    state::let_i(&new_cs, &old_cs, None);
  });
  DefMacro!("\\ShowCommand Token", "");

  TeX!(
    r#"""
  \DeclareSymbolFont{operators}   {OT1}{cmr} {m}{n}
  \DeclareSymbolFont{letters}     {OML}{cmm} {m}{it}
  \DeclareSymbolFont{symbols}     {OMS}{cmsy}{m}{n}
  \DeclareSymbolFont{largesymbols}{OMX}{cmex}{m}{n}
  """#
  );
  // Perl: latex_constructs.pool.ltxml L5759-5764 — picture font stubs
  DefPrimitive!("\\OMX", None, font => { family => "cmex10" });
  DefPrimitive!("\\tenln", None, font => { family => "line10" });
  DefPrimitive!("\\tenlnw", None, font => { family => "linew10" });
  DefPrimitive!("\\tencirc", None, font => { family => "lcircle10" });
  DefPrimitive!("\\tencircw", None, font => { family => "lcirclew10" });

  // Perl latex_constructs.pool.ltxml L2814-2832: uclclist members are
  // DefPrimitiveI(..., robust=>1) — Expandable wrapper expanding to
  // `\protect <cs-munged>` (Rust `def_robust_cs`), with the munged CS
  // as the primitive emitting the Unicode char. `\MakeUppercase`'s
  // case-mapping pipeline reads `\protect <cs>` pairs; see
  // `lx_read_and_change_case` protect-branch + `\lx@prepare@case@mapping`.
  DefPrimitive!("\\OE", "\u{0152}", robust => true); // LATIN CAPITAL LIGATURE OE
  DefPrimitive!("\\oe", "\u{0153}", robust => true); // LATIN SMALL LIGATURE OE
  DefPrimitive!("\\AE", "\u{00C6}", robust => true); // LATIN CAPITAL LETTER AE
  DefPrimitive!("\\ae", "\u{00E6}", robust => true); // LATIN SMALL LETTER AE
  DefPrimitive!("\\AA", "\u{00C5}", robust => true); // LATIN CAPITAL LETTER A WITH RING ABOVE
  DefPrimitive!("\\aa", "\u{00E5}", robust => true); // LATIN SMALL LETTER A WITH RING ABOVE
  DefPrimitive!("\\O",  "\u{00D8}", robust => true); // LATIN CAPITAL LETTER O WITH STROKE
  DefPrimitive!("\\o",  "\u{00F8}", robust => true); // LATIN SMALL LETTER O WITH STROKE
  DefPrimitive!("\\L",  "\u{0141}", robust => true); // LATIN CAPITAL LETTER L WITH STROKE
  DefPrimitive!("\\l",  "\u{0142}", robust => true); // LATIN SMALL LETTER L WITH STROKE
  DefPrimitive!("\\ss", "\u{00DF}", robust => true); // LATIN SMALL LETTER SHARP S
  DefPrimitive!("\\dh", "\u{00F0}", robust => true); // eth
  DefPrimitive!("\\DH", "\u{00D0}", robust => true); // Eth (looks same as \DJ!)
  DefPrimitive!("\\dj", "\u{0111}", robust => true); // d with stroke
  DefPrimitive!("\\DJ", "\u{0110}", robust => true); // D with stroke (looks same as \DH!)
  DefPrimitive!("\\ng", "\u{014B}", robust => true);
  DefPrimitive!("\\NG", "\u{014A}", robust => true);
  DefPrimitive!("\\th", "\u{00FE}", robust => true);
  DefPrimitive!("\\TH", "\u{00DE}", robust => true);


  DefPrimitive!("\\newenvironment OptionalMatch:* {}[Number][]{}{}",
  sub[(_star_opt, name, nargs, opt, begin, end)] {
    let name = { Expand!(name).to_string() };
    let name_cs = T_CS!(format!("\\{name}"));
    if IsDefined!(&name_cs) {
      let is_locked = lookup_bool(&s!("\\{}:locked",name)) ||
       lookup_bool(&s!("\\begin{{{}}}:locked",name));
      if !is_locked {
        let message = s!("Ignoring redefinition (\\newenvironment) of Environment {:?}", name);
        Info!("ignore", name, message);
      }
    } else {
      // TODO: can we convince DefMacro! this is not a second mutable borrow of state::
      let converted_args = convert_latex_args(nargs.value_of() as usize, opt)?;
      let end_name_cs = T_CS!(s!("\\end{}",name));
      DefMacro!(name_cs, converted_args, begin);
      DefMacro!(end_name_cs, None, end);
    }
    Ok(Vec::new())
  });

  DefPrimitive!("\\renewenvironment OptionalMatch:* {}[Number][]{}{}",
  sub[(_star, name, nargs, opt, begin, end)] {
    let name = Expand!(name).to_string();
    let is_locked = lookup_bool(&s!("\\{}:locked",name)) ||
       lookup_bool(&s!("\\begin{{{}}}:locked",name));
    if !is_locked {
      let name_cs = T_CS!(s!("\\{}",name));
      let end_name_cs = T_CS!(s!("\\end{}",name));
      let converted_args = convert_latex_args(nargs.value_of() as usize, opt)?;

      DefMacro!(name_cs, converted_args, begin);
      DefMacro!(end_name_cs, None, end);
    }
    Ok(Vec::new())
  });


  //======================================================================
  // C.8.3 Theorem-like Environments
  //======================================================================
  AssignValue!("thm@swap" => 0i64);
  DefRegister!("\\thm@style"         => Tokens!(T_OTHER!("plain")));
  DefRegister!("\\thm@headfont"      => Tokens!(T_CS!("\\bfseries")));
  DefRegister!("\\thm@notefont"      => Tokens!(T_CS!("\\the"), T_CS!("\\thm@headfont")));
  DefRegister!("\\thm@bodyfont"      => Tokens!(T_CS!("\\itshape")));
  DefRegister!("\\thm@headformatter" => Tokens!());
  DefRegister!("\\thm@headpunct"     => Tokens!());
  DefRegister!("\\thm@styling"       => Tokens!());
  DefRegister!("\\thm@headstyling"   => Tokens!());
  DefRegister!("\\thm@prework"       => Tokens!());
  DefRegister!("\\thm@postwork"      => Tokens!());
  DefRegister!("\\thm@symbol"        => Tokens!());
  DefRegister!("\\thm@numbering"     => Tokens!(T_CS!("\\arabic")));

  DefPrimitive!("\\th@plain", {
    state::assign_register("\\thm@bodyfont",
      RegisterValue::Tokens(Tokens!(T_CS!("\\itshape"))), None, vec![])?;
    state::assign_register("\\thm@headstyling",
      RegisterValue::Tokens(Tokens!(T_CS!("\\lx@makerunin"))), None, vec![])?;
  });

  DefMacro!("\\lx@makerunin",   "\\@ADDCLASS{ltx_runin}");
  DefMacro!("\\lx@makeoutdent", "\\@ADDCLASS{ltx_outdent}");

  DefMacro!("\\@thmcountersep", ".");
  DefMacro!("\\thm@doendmark",  "");

  init_savable_theorem_parameters(vec![
    "\\thm@bodyfont", "\\thm@headpunct",
    "\\thm@styling", "\\thm@headstyling",
    "thm@swap",
  ]);

  // Activate the default style.
  RawTeX!("\\th@plain");

  Tag!("ltx:theorem", auto_close => true);
  Tag!("ltx:proof",   auto_close => true);

  DefPrimitive!("\\newtheorem OptionalMatch:* {}[]{}[]", sub[(flag, thmset, otherthmset, typ, reset)] {
    define_new_theorem(
      flag.filter(|f| !f.is_empty()),
      thmset,
      otherthmset.filter(|t| !t.is_empty()),
      if typ.is_empty() { None } else { Some(typ) },
      reset.filter(|t| !t.is_empty()),
    )?;
    // Reset these!
    state::assign_register("\\thm@prework",
      RegisterValue::Tokens(Tokens!()), None, vec![])?;
    state::assign_register("\\thm@postwork",
      RegisterValue::Tokens(Tokens!()), None, vec![])?;
  });


  //======================================================================
  // C.8.4 Numbering
  //======================================================================
  // For LaTeX documents, We want id's on para, as well as sectional units.
  // However, para get created implicitly on Document construction, rather than
  // explicitly during digestion (via a whatsit), we can't use the usual LaTeX counter mechanism.
  Tag!("ltx:para", after_open => sub[document, node] {
    document.generate_id(node, "p")?;
  });

  DefPrimitive!("\\newcounter{}[]", sub[(cs, default_opt)] {
    let default = if let Some(tks) = default_opt {
      if !tks.is_empty() {
        Expand!(tks)
      } else {
        Tokens!()
      }
    } else {
      Tokens!()
    };
    let cs_expanded = &Expand!(cs).to_string();
    NewCounter!(cs_expanded, &default.to_string());
  });
  DefPrimitive!("\\setcounter{}{Number}", sub[(cs, default)] {
    let cs_expanded = &Expand!(cs).to_string();
    SetCounter!(cs_expanded, default);
  });
  DefPrimitive!("\\addtocounter{}{Number}", sub[(cs,default)] {
    let cs_expanded = &Expand!(cs).to_string();
    AddToCounter!(cs_expanded, default);
  });
  DefPrimitive!("\\stepcounter{}",    sub[(cs)] {
    let cs_expanded = &Expand!(cs).to_string();
    StepCounter!(cs_expanded, false)?;
  });
  DefPrimitive!("\\refstepcounter{}", sub[(cs)] {
    let cs_expanded = &Expand!(cs).to_string();
    RefStepCounter!(cs_expanded, false)?;
  });

  // Perl latex_constructs.pool.ltxml: addtoCounterReset + defCounterID
  DefPrimitive!("\\@addtoreset{}{}", sub[(ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    let unctr = s!("UN{}", ctr_str);
    let reg = s!("\\cl@{}", within_str);
    // Prepend ctr and UNctr to the counter reset list for 'within'
    let prev = state::lookup_tokens(&reg).unwrap_or_default();
    let mut toks = vec![T_CS!(ctr_str), T_CS!(unctr)];
    toks.extend(prev.unlist());
    state::assign_value(&reg, Stored::Tokens(Tokens::new(toks)), None);
  });

  // Perl: latex_constructs.pool.ltxml \@removefromreset
  DefPrimitive!("\\@removefromreset{}{}", sub[(ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    let reg = s!("\\cl@{}", within_str);
    if let Some(prev) = state::lookup_tokens(&reg) {
      let unctr_cs = T_CS!(s!("UN{}", ctr_str));
      let ctr_cs = T_CS!(ctr_str);
      let filtered: Vec<Token> = prev.unlist().into_iter()
        .filter(|t| *t != ctr_cs && *t != unctr_cs)
        .collect();
      state::assign_value(&reg, Stored::Tokens(Tokens::new(filtered)), None);
    }
  });

  // Perl: latex_constructs.pool.ltxml \counterwithin
  DefPrimitive!("\\counterwithin OptionalMatch:* {}{}", sub[(star, ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    // Add ctr to reset list of within
    let unctr = s!("UN{}", ctr_str);
    let reg = s!("\\cl@{}", within_str);
    let prev = state::lookup_tokens(&reg).unwrap_or_default();
    let mut toks = vec![T_CS!(ctr_str.clone()), T_CS!(unctr)];
    toks.extend(prev.unlist());
    state::assign_value(&reg, Stored::Tokens(Tokens::new(toks)), None);
    if star.is_none() {
      // Redefine \thectr to include \thewithin prefix
      let the_ctr = T_CS!(s!("\\the{}", ctr_str));
      let expansion = s!("\\the{}.\\arabic{{{}}}", within_str, ctr_str);
      let _ = def_macro(the_ctr, None,
        Some(ExpansionBody::from(expansion)),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
      // defCounterID with within
      let prefix = state::lookup_string(&s!("@ID@prefix@{}", ctr_str));
      let clean_prefix = if prefix.is_empty() { ctr_str.clone() } else { prefix };
      let ctr_for_id = ctr_str.clone();
      let within_for_id = within_str.clone();
      let thectrid = s!("\\the{}@ID", ctr_str);
      let _ = def_macro(T_CS!(thectrid), None,
        Some(ExpansionBody::Closure(Rc::new(move |_args| {
          Ok(mouth::tokenize_internal(&s!(
            "\\expandafter\\ifx\\csname the{}@ID\\endcsname\\@empty\\else\\csname the{}@ID\\endcsname.\\fi {}\\csname @{}@ID\\endcsname",
            within_for_id, within_for_id, clean_prefix, ctr_for_id
          )))
        }))),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
    }
  });

  // Perl: latex_constructs.pool.ltxml \counterwithout
  DefPrimitive!("\\counterwithout OptionalMatch:* {}{}", sub[(star, ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    // Remove ctr from reset list of within
    let reg = s!("\\cl@{}", within_str);
    if let Some(prev) = state::lookup_tokens(&reg) {
      let ctr_cs = T_CS!(ctr_str.clone());
      let unctr_cs = T_CS!(s!("UN{}", ctr_str));
      let filtered: Vec<Token> = prev.unlist().into_iter()
        .filter(|t| *t != ctr_cs && *t != unctr_cs)
        .collect();
      state::assign_value(&reg, Stored::Tokens(Tokens::new(filtered)), None);
    }
    if star.is_none() {
      // Redefine \thectr without prefix
      let the_ctr = T_CS!(s!("\\the{}", ctr_str));
      let expansion = s!("\\arabic{{{}}}", ctr_str);
      let _ = def_macro(the_ctr, None,
        Some(ExpansionBody::from(expansion)),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
      // defCounterID without within — redefine \thectr@ID
      let prefix = state::lookup_string(&s!("@ID@prefix@{}", ctr_str));
      let clean_prefix = if prefix.is_empty() { ctr_str.clone() } else { prefix };
      let ctr_for_id = ctr_str.clone();
      let thectrid = s!("\\the{}@ID", ctr_str);
      let _ = def_macro(T_CS!(thectrid), None,
        Some(ExpansionBody::Closure(Rc::new(move |_args| {
          Ok(mouth::tokenize_internal(&s!(
            "{}\\csname @{}@ID\\endcsname", clean_prefix, ctr_for_id
          )))
        }))),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
    }
  });

  DefMacro!("\\cl@@ckpt", "\\@elt{page}");

  DefMacro!("\\value{}", sub[(value)] {
    T_CS!(s!("\\c@{}", Expand!(value)))
  });

  DefMacro!("\\@arabic{Number}", sub[(number)] {
    ExplodeText!(number.value_of().to_string())
  });
  DefMacro!("\\arabic{}", sub[(value)] {
    let ctr_expansion = Expand!(value).to_string();
    let ctr_value = CounterValue!(&ctr_expansion).value_of();
    ExplodeText!(ctr_value)
  });

  DefMacro!("\\@roman{Number}", sub[(number)] {
    ExplodeText!(radix::radix_roman(number.value_of()))
  });
  DefMacro!("\\roman{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_roman(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@Roman{Number}", sub[(number)] {
    ExplodeText!(radix::radix_up_roman(number.value_of()))
  });
  DefMacro!("\\Roman{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_up_roman(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@alph{Number}", sub[(number)] {
    ExplodeText!(radix::radix_alpha(number.value_of()))
  });
  DefMacro!("\\alph{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_alpha(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@Alph{Number}", sub[(number)] {
    ExplodeText!(radix::radix_up_alpha(number.value_of()))
  });
  DefMacro!("\\Alph{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_up_alpha(CounterValue!(&ctr).value_of()))
  });

  DefMacro!("\\@fnsymbol{Number}", sub[(number)] {
    ExplodeText!(radix::radix_format_str(number.value_of(), FNSYMBOLS))
  });
  DefMacro!("\\fnsymbol{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_format_str(CounterValue!(&ctr).value_of(), FNSYMBOLS))
  });


  // ======================================================================
  // C.9 Figures and Other Floating Bodies
  // ======================================================================


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

  // Perl: latex_constructs.pool.ltxml L3199-3212 — internal @float/@dblfloat
  // Used by raw TeX packages (e.g., nips_2017.sty) via \@float{type}[placement]
  // Since the float type arg isn't known at compile time, we use a properties
  // closure to call beforeFloat dynamically.
  DefEnvironment!("{@float}{}[]",
    "<ltx:float xml:id='#id' inlist='#inlist' ?#2(placement='#2') class='ltx_float_#1'>\
    #tags#body\
    </ltx:float>",
    properties => sub[args] {
      let float_type = args.first().and_then(|a| a.as_ref())
        .map(|d| d.to_string()).unwrap_or_default();
      before_float(&float_type, None);
      Ok(stored_map!("layout" => "vertical"))
    },
    after_digest  => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical");
  DefEnvironment!("{@dblfloat}{}[]",
    "<ltx:float xml:id='#id' inlist='#inlist' ?#2(placement='#2') class='ltx_float_#1'>\
    #tags#body\
    </ltx:float>",
    properties => sub[args] {
      let float_type = args.first().and_then(|a| a.as_ref())
        .map(|d| d.to_string()).unwrap_or_default();
      before_float_ex(&float_type, None, true);
      Ok(stored_map!("layout" => "vertical"))
    },
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
  // \abovecaptionskip, \belowcaptionskip — not in Perl engine
  // (Perl: article.cls.ltxml; Rust: article_cls.rs already defines them)
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



  DefConditional!("\\if@reversemargin");
  Let!("\\reversemarginpar", "\\@reversemargintrue");
  Let!("\\normalmarginpar", "\\@reversemarginfalse");
  // Perl: latex_constructs.pool.ltxml lines 3543-3546
  DefConstructor!("\\marginpar[]{}", r###"?#1(<ltx:note role='margin' class='ltx_marginpar_left'><ltx:inline-logical-block>#1</ltx:inline-logical-block></ltx:note>?#2(<ltx:note role='margin' class='ltx_marginpar_right'><ltx:inline-logical-block>#2</ltx:inline-logical-block></ltx:note>))(<ltx:note role='margin' class='ltx_marginpar'><ltx:inline-logical-block>#2</ltx:inline-logical-block></ltx:note>)"###);

  DefRegister!("\\marginparpush", Dimension::new(0));

  // ======================================================================
  // C.10 Lining It Up in Columns
  // ======================================================================


  //======================================================================
  // C.10.1 The tabbing Environment
  // Perl: latex_constructs.pool.ltxml lines 3554-3651
  //======================================================================

  DefRegister!("\\tabbingsep" => Dimension::new(0));

  // Main entry: \tabbing → \par\@tabbing@bindings\@@tabbing\lx@begin@alignment
  DefMacro!("\\tabbing", "\\par\\@tabbing@bindings\\@@tabbing\\lx@begin@alignment");
  DefMacro!("\\endtabbing", "\\lx@end@alignment\\@end@tabbing\\par");

  DefPrimitive!("\\@end@tabbing", sub [_args] {
    egroup()?;
  });

  DefConstructor!("\\@@tabbing SkipSpaces DigestedBody", "#1",
    reversion => "\\begin{tabbing}#1\\end{tabbing}",
    before_digest => sub {
      bgroup();
    },
    mode => "internal_vertical"
  );

  // Wrapper macros that expand to marker + & (column separator)
  DefMacro!("\\@tabbing@tabset", "\\@tabbing@tabset@marker&");
  DefMacro!("\\@tabbing@nexttab", "\\@tabbing@nexttab@marker&");
  DefMacro!("\\@tabbing@newline OptionalMatch:* [Dimension]", "\\@tabbing@newline@marker\\cr");
  DefMacro!("\\@tabbing@kill", "\\@tabbing@kill@marker\\cr\\@tabbing@start@tabs");

  // Marker constructors
  DefConstructor!("\\@tabbing@tabset@marker", "",
    reversion => "\\=",
    properties => { Ok(stored_map!("alignmentSkippable" => true)) }
  );
  DefConstructor!("\\@tabbing@nexttab@marker", "",
    reversion => "\\>",
    properties => { Ok(stored_map!("alignmentSkippable" => true)) }
  );
  DefConstructor!("\\@tabbing@newline@marker", "",
    reversion => "\\\\"
  );
  DefConstructor!("\\@tabbing@kill@marker", "",
    reversion => "\\kill",
    after_digest => sub [_whatsit] {
      // Perl: LookupValue('Alignment')->removeRow
      if let Some(alignment_stored) = lookup_alignment() {
        if let Some(alignment_cell) = alignment_stored.alignment_cell() {
          alignment_cell.borrow_mut().remove_row();
        }
      }
    },
    properties => { Ok(stored_map!("alignmentSkippable" => true)) }
  );

  // Tab tracking
  state::assign_value(
    "tabbing_start_tabs",
    Stored::Tokens(Tokens!()),
    Some(Scope::Global),
  );

  DefMacro!("\\@tabbing@start@tabs", sub [_args] {
    if let Some(Stored::Tokens(toks)) = state::lookup_value("tabbing_start_tabs") {
      toks
    } else {
      Tokens!()
    }
  });

  // \+ increments tab start by adding \> to tabbing_start_tabs
  DefPrimitive!("\\@tabbing@increment", sub [_args] {
    let mut tabs = if let Some(Stored::Tokens(toks)) = state::lookup_value("tabbing_start_tabs") {
      toks.unlist()
    } else {
      Vec::new()
    };
    tabs.push(T_CS!("\\>"));
    state::assign_value(
      "tabbing_start_tabs",
      Stored::Tokens(Tokens::new(tabs)),
      Some(Scope::Global),
    );
  });

  // \- decrements tab start by removing first element from tabbing_start_tabs
  DefPrimitive!("\\@tabbing@decrement", sub [_args] {
    let tabs = if let Some(Stored::Tokens(toks)) = state::lookup_value("tabbing_start_tabs") {
      let mut v = toks.unlist();
      if !v.is_empty() {
        v.remove(0);
      }
      v
    } else {
      Vec::new()
    };
    state::assign_value(
      "tabbing_start_tabs",
      Stored::Tokens(Tokens::new(tabs)),
      Some(Scope::Global),
    );
  });

  // Stubs for unimplemented features (matching Perl)
  DefPrimitive!("\\@tabbing@untab", sub [_args] { /* NOT HANDLED — see Perl note */ });
  DefPrimitive!("\\@tabbing@flushright", sub [_args] { /* NOT HANDLED */ });
  DefPrimitive!("\\@tabbing@hfil", sub [_args] { /* NOT HANDLED */ });
  DefPrimitive!("\\@tabbing@pushtabs", sub [_args] { /* NOT HANDLED */ });
  DefPrimitive!("\\@tabbing@poptabs", sub [_args] { /* NOT HANDLED */ });

  // Accent redirect: \a{x} → \@tabbing@x (looks up the accent by name)
  DefMacro!("\\@tabbing@accent{}", sub [args] {
    let accent = args[0].to_string();
    Tokens::new(vec![T_CS!(&format!("\\@tabbing@{accent}"))])
  });

  // Default definitions for \pushtabs/\poptabs/\kill (outside tabbing)
  DefMacro!("\\pushtabs", "");
  DefMacro!("\\poptabs", "");
  DefMacro!("\\kill", "");

  // The binding primitive that sets up the alignment
  DefPrimitive!("\\@tabbing@bindings", sub [_args] {
    tabbing_bindings()?;
  });

  // Internals of tabbing for program.sty compatibility
  DefMacro!("\\@startfield", "\\global\\setbox\\@curfield\\hbox\\bgroup\\color@begingroup");
  DefMacro!("\\@stopfield", "\\color@endgroup\\egroup");
  DefMacro!("\\@contfield", "\\global\\setbox\\@curfield\\hbox\\bgroup\\color@begingroup\\unhbox\\@curfield");
  DefMacro!("\\@addfield", "\\global\\setbox\\@curline\\hbox{\\unhbox\\@curline\\unhbox\\@curfield}");


  DefRegister!("\\lx@arstrut", Dimension!("0pt"));
  DefRegister!("\\lx@default@tabcolsep", Dimension!("6pt"));
  DefRegister!("\\tabcolsep", Dimension!("6pt"));
  DefMacro!("\\arraystretch", None, T_OTHER!("1"));
  Let!("\\@tabularcr", "\\lx@alignment@newline");
  if !has_value("GUESS_TABULAR_HEADERS") {
    AssignValue!("GUESS_TABULAR_HEADERS" => true); // Defaults to yes
  }

  // Keyvals are for attributes for the alignment.
  // Typical keys are width, vattach,...
  DefKeyVal!("tabular", "width", "Dimension");
  DefPrimitive!("\\@tabular@bindings AlignmentTemplate OptionalKeyVals:tabular",
    sub[(template, attributes_opt)] {
    let attrs_stored = attributes_opt.map(KeyVals::as_flat_hash).unwrap_or_default();
    let mut attrs = HashMap::default();
    for (k,v) in attrs_stored {
      attrs.insert(k, v.to_string());
    }
    if let Some(va) = attrs.get("vattach") {
      attrs.insert(String::from("vattach"), translate_attachment(va).to_string());
    }

    tabular_bindings(template, SymHashMap::default(), attrs)?;
  });

  DefMacro!("\\@tabular@before", None);
  DefMacro!("\\@tabular@after", None);
  DefMacro!("\\@tabular@row@before", None);
  DefMacro!("\\@tabular@row@after", None);
  DefMacro!("\\@tabular@column@before", None);
  DefMacro!("\\@tabular@column@after", None);

  // The Core alignment support is in LaTeXML::Core::Alignment and in TeX.ltxml
  DefMacro!("\\tabular[]{}",
    r"\@tabular@bindings{#2}[vattach=#1]\@@tabular[#1]{#2}\lx@begin@alignment\@tabular@before",
    locked => true);
  DefMacro!("\\endtabular", r"\@tabular@after\lx@end@alignment\@end@tabular",
    locked => true);
  DefPrimitive!("\\@end@tabular", {
    egroup()?;
  });
  // Perl latex_constructs.pool.ltxml L3735-3746: mode => 'restricted_horizontal',
  //   enterHorizontal => 1.
  DefConstructor!("\\@@tabular[] Undigested DigestedBody",
    "#3",
    reversion    => r"\begin{tabular}[#1]{#2}#3\end{tabular}",
    before_digest => { bgroup(); },
    sizer        => "#3",
    after_digest  => sub[whatsit] {
      if let Some(alignment) = lookup_alignment() {
        if let DigestedData::Alignment(data) = alignment.data() {
          let attachment = if let Some(arg) = whatsit.get_arg(1) { translate_attachment(arg) }
          else { translate_attachment(String::new()) };
          let mut data_lock = data.borrow_mut();
          let attributes = data_lock.get_xml_attributes_mut();
          attributes.insert(String::from("vattach"), attachment.to_string());
        }
      }
    },
    locked => true,
    mode   => "text",
    enter_horizontal => true);

  DefMacro!("\\csname tabular*\\endcsname{Dimension}[]{}",
    r"\@tabular@bindings{#3}[width=#1,vattach=#2]\@@tabular@{#1}[#2]{#3}\lx@begin@alignment");
  DefMacro!("\\csname endtabular*\\endcsname",
    r"\lx@end@alignment\@end@tabular@");
  // Perl latex_constructs.pool.ltxml L3753-3757: mode => 'restricted_horizontal',
  //   enterHorizontal => 1.
  DefConstructor!("\\@@tabular@{Dimension}[] Undigested DigestedBody",
    "#4",
    before_digest => { bgroup(); },
    reversion    => r"\begin{tabular*}{#1}[#2]{#3}#4\end{tabular*}",
    mode         => "text",
    enter_horizontal => true);
  DefPrimitive!("\\@end@tabular@", {
    egroup()?;
  });
  // Perl: Let('\multicolumn', '\lx@alignment@multicolumn');
  Let!("\\multicolumn", "\\lx@alignment@multicolumn");

  // A weird bit that sometimes gets invoked by Cargo Cult programmers...
  // to \noalign in the defn of \hline! Bizarre! (see latex.ltx)
  // However, the really weird thing is the way this provides the } to close the argument
  DefMacro!("\\@xhline", r"\ifnum0=`{\fi}");

  DefMacro!("\\cline{}", r"\noalign{\@cline{#1}}");
  DefConstructor!("\\@cline{}", "",
    after_digest => sub[whatsit] {
      let cols = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      let mut cols_vec = Vec::new();
      let cols_chars = cols.chars();
      let mut from : Option<usize> = None;
      let mut num = String::new();
      for c_next in cols_chars {
        match c_next {
          ',' => if !num.is_empty() {
            let this_num = num.parse::<usize>().unwrap();
            if let Some(from_num) = from {
              for num_in_range in from_num..=this_num {
                cols_vec.push(num_in_range);
              }
            } else {
              cols_vec.push(this_num);
            }
            from = None;
            num = String::new();
          },
          '-' => {
            from = Some(num.parse::<usize>().unwrap());
            num = String::new();
          }
          c if c.is_ascii_digit() => num.push(c_next),
          _ => break
        }
      }
      if !num.is_empty() {
        let this_num = num.parse::<usize>().unwrap();
        if let Some(from_num) = from {
          for num_in_range in from_num..=this_num {
            cols_vec.push(num_in_range);
          }
        } else {
          cols_vec.push(this_num);
        }
      }
      if let Some(alignment_stored) = lookup_alignment() {
        alignment_stored.alignment_cell().unwrap().borrow_mut()
          .add_line("t", cols_vec);
      }
    },
    sizer      => 0, alias => "\\cline",
    // properties => { "isHorizontalRule" => true }
  );

  DefConstructor!("\\vline", "",
    properties => sub[_args] {
      Ok(stored_map!("isVerticalRule" => true))
    },
    sizer      => 0,
  );
  DefRegister!("\\lx@default@arraycolsep", Dimension!("5pt"));
  DefRegister!("\\arraycolsep", Dimension!("5pt"));
  DefRegister!("\\arrayrulewidth", Dimension!("0.4pt"));
  DefRegister!("\\doublerulesep", Dimension!("2pt"));
  DefMacro!("\\extracolsep{}", None);

  // Array and similar environments
  // Perl: latex_constructs.pool.ltxml lines 3792-3809
  DefPrimitive!("\\@array@bindings [] AlignmentTemplate", sub[(pos, template)] {
    let mut attrs = HashMap::default();
    let attachment = pos.map(|a| translate_attachment(a.to_string()))
      .unwrap_or_else(|| translate_attachment(""));
    attrs.insert(String::from("vattach"), attachment.to_string());
    attrs.insert(String::from("role"), String::from("ARRAY"));
    // Determine column and row separations, if non default
    let colsep = lookup_dimension("\\arraycolsep");
    if let Some(sep) = colsep {
      if sep.value_of()
        != lookup_dimension("\\lx@default@arraycolsep")
          .unwrap_or_default()
          .value_of()
      {
        attrs.insert(String::from("colsep"), sep.to_attribute());
      }
    }
    let astr = gullet::do_expand(T_CS!("\\arraystretch"))?.to_string();
    if astr != "1" {
      if let Ok(astr_f) = astr.parse::<f64>() {
        if astr_f != 1.0 {
          let rowsep = Dimension::from_str(&s!("{}em", astr_f - 1.0))?;
          attrs.insert(String::from("rowsep"), rowsep.to_attribute());
        }
      }
    }
    alignment_bindings(template, String::from("math"), SymHashMap::default(), attrs);
    // Perl: if display math, switch to text mathstyle
    if state::lookup_string_from_sym(pin!("MODE")).ends_with("math") {
      MergeFont!(mathstyle => "text");
    }
    Let!("\\\\", "\\lx@alignment@newline");
    Let!("\\lx@intercol", "\\lx@math@intercol");
  });

  DefMacro!(
    "\\array[]{}",
    r"\@array@bindings[#1]{#2}\@@array[#1]{#2}\lx@begin@alignment"
  );
  DefMacro!("\\endarray", None, r"\lx@end@alignment\@end@array");
  DefPrimitive!("\\@end@array", {
    egroup()?;
  });
  DefConstructor!("\\@@array[] Undigested DigestedBody",
    "#3",
    before_digest => { bgroup(); },
    reversion    => r"\begin{array}[#1]{#2}#3\end{array}");

  DefMacro!("\\@tabarray", r"\m@th\@@array[c]");


  // ======================================================================
  // C.11 Moving Information Around
  // ======================================================================


  //======================================================================
  // C.11.1 Files
  //======================================================================
  DefPrimitive!("\\nofiles", None);

  // Perl: DefPrimitive('\listfiles', undef) — no-op. Required so the
  // autoload trigger for `\listfiles` (engine/tex.rs) gets overridden
  // after LaTeX.pool loads; otherwise the trigger re-expands itself
  // after each pool load, creating a unique mouth-source per iteration
  // that eventually trips the 50M arena::pin sentinel (arxiv 1311.6082).
  DefPrimitive!("\\listfiles", None);

  //======================================================================
  // C.11.2 Cross-References
  //======================================================================

  // \label attaches a label to the nearest parent that can accept a labels attribute
  // but only those that have an xml:id (but should this require a refnum and/or title ???)
  // Note that latex essentially allows redundant labels, but we can record only one!!!
  DefConstructor!("\\label Semiverbatim", sub[document, _olabel, props] {
    if let Some(savenode) = document.float_to_label() {
      let mut labels : HashMap<String,bool> = HashMap::default();
      if let Some(label) = props.get("label") {
        labels.insert(label.to_string(), true);
      }
      for label in document.node_get_attribute("labels").unwrap_or_default().split_whitespace() {
        labels.insert(label.to_string(), true);
      }
      let mut sorted_labels: Vec<String> = labels.into_keys().collect();
      sorted_labels.sort();
      document.node_set_attribute("labels", &sorted_labels.join(" "))?;
      document.set_node(&savenode);
    }
  },
  // Perl L3847-3848: disappear in tex=/content-tex unless outside DUAL_BRANCH.
  // Empty reversion: \label contributes no visible content to tex= attributes.
  reversion => "",
  properties => {stored_map!("alignmentSkippable" => true, "alignmentPreserve" => true)},
  after_digest => sub[whatsit] {
    if let Some(arg1) = whatsit.get_arg(1) {
      maybe_note_label(&arg1.to_string());
    }
    let label = match whatsit.get_arg(1) {
      Some(labeld) => clean_label(&labeld.to_string(), None).into_owned(),
      None => String::new()
    };
    let scope = label.replace("LABEL:","label:");
    let label_key = s!("LABEL@{}", label);
    whatsit.set_property("label", label);

    let ctr_key_opt = with_value("current_counter", |val_opt| val_opt
      .map(|ctr| s!("scopes_for_counter:{}", ctr)));
    if let Some(ctr_key) = ctr_key_opt {
      // TODO: we should probably improve the ergonomics here to avoid the vec![]
      state::unshift_value(&ctr_key, vec![scope.clone()]);
      state::activate_scope(arena::pin(scope));
      stomach::begin_mode("text")?;
      let current_label = stomach::digest(Tokens!(T_CS!("\\@currentlabel")))?;
      state::assign_value(&label_key, current_label, Some(Scope::Global));
      stomach::end_mode("text")?;
    }
  }
  );

  // If a node has been labeled, but still hasn't yet got an id by afterClose:late,
  // we'd better generate an id for it.
  Tag!("ltx:*", after_close_late => sub[document, node] {
    if node.has_attribute("labels") && !node.has_attribute("xml:id") {
      document.generate_id(node, "")?;
    }
  });

  // # These will get filled in during postprocessing.
  // # * is added to accommodate hyperref
  // Perl latex_constructs.pool.ltxml L3873-3878: sizer => '()',
  //   robust => 1, enterHorizontal => 1.
  DefConstructor!("\\ref OptionalMatch:* Semiverbatim",
    "<ltx:ref ?#1(class='ltx_nolink')() labelref='#label' _force_font='true'/>",
    sizer => "()",
    robust => true,
    enter_horizontal => true,
    properties => sub[args] {
      unpack_opt_ref!(args => _star, label_opt);
      let label = label_opt.as_ref().unwrap().to_string();
      Ok(stored_map!("label" => Stored::String(arena::pin(clean_label(&label, None)))))
  });

  // "page" does not make sense in xml.  If the user really wants, they will need:
  // \usepackage{latexml} ... \iflatexml alternate\else page \pageref{label}\fi
  Let!("\\pageref", "\\ref");

  // \@setref is from latex.ltx kernel. LaTeXML redefines \ref directly,
  // so \@setref is normally bypassed — but some packages call it directly.
  // The body below IS the latex.ltx kernel definition: if #1 is \relax
  // (undefined ref), show "??"; otherwise apply #2 to #1 with a \null guard.
  RawTeX!("\\def\\@setref#1#2#3{\\ifx#1\\relax ??\\else\\expandafter#2#1\\null\\fi}");

  // ======================================================================
  //  C.11.3 Bibliography and Citation
  // ======================================================================

  // Note that it's called \refname in LaTeX's article, but \bibname in report & book.
  // And likewise, mixed up in various other classes!

  DefMacro!("\\thebibliography@ID", "");
  // Perl: latex_constructs.pool.ltxml L3891 — initial empty value
  DefMacro!("\\the@lx@bibliography@ID", "");

  DefMacro!(
    "\\bibliography Semiverbatim",
    r#"\lx@ifusebbl{#1}{\input{\jobname.bbl}}{\lx@bibliography{#1}}"#
  );

  DefMacro!("\\lx@ifusebbl{}{}{}", sub[(bib_files_tks, bbl_clause, bib_clause)] {
    let bib_files = Expand!(bib_files_tks).to_string();
    if bib_files.is_empty() {
      return Ok(Tokens!());
    }
    let jobname = Expand!(T_CS!("\\jobname")).to_string();
    let bbl_path = FindFile!(&jobname, type => "bbl");
    // BIB_CONFIG is a list of phases; with bibconfig=bbl,bib: try bbl first, fall back to bib.
    // Default (bibtex option) is ['bib', 'bbl']; nobibtex sets ['bbl'].
    let default_bib_config: Rc<[SymStr]> = Rc::new([arena::pin("bib"), arena::pin("bbl")]);
    let bib_config = match lookup_value("BIB_CONFIG") {
      Some(Stored::Strings(v)) => v,
      _ => default_bib_config,
    };
    if bib_config.is_empty() {
      Info!("missing", "bib_config", "BIB_CONFIG was empty, ignoring bibliography phase.");
      return Ok(Tokens!());
    }
    // Iterate through config phases as a fallback chain
    for phase in bib_config.iter() {
      let is_bbl = arena::with(*phase, |s| s == "bbl");
      if is_bbl {
        if bbl_path.is_some() {
          return Ok(bbl_clause);
        }
        // bbl not found — fall through to next phase
        Info!("expected", "bbl", "Couldn't find bbl file, trying next bibliography phase.");
      } else {
        // 'bib' phase — check if .bib files exist
        let mut missing_bibs = String::new();
        for bf in bib_files.split(',') {
          let bib_path = FindFile!(bf, type => "bib");
          if bib_path.is_none() {
            if !missing_bibs.is_empty() {
              missing_bibs.push(',');
            }
            missing_bibs.push_str(bf);
          }
        }
        if missing_bibs.is_empty() || bbl_path.is_none() {
          return Ok(bib_clause);
        } else {
          Info!("expected", missing_bibs, s!("Couldn't find all bib files, using {jobname}.bbl instead"));
          return Ok(bbl_clause);
        }
      }
    }
    // All phases exhausted — no bibliography found
    Info!("expected", "bbl", "Couldn't find bbl file, bibliography may be empty.");
    Ok(Tokens!())
  });

  AssignMapping!("BACKMATTER_ELEMENT", "ltx:bibliography" => "ltx:section");
  AssignMapping!("BACKMATTER_ELEMENT", "ltx:index"        => "ltx:section");

  DefConstructor!("\\lx@bibliography [] Semiverbatim",
    "<ltx:bibliography files='#2' xml:id='#id' bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort' lists='#1'><ltx:title font='#titlefont' _force_font='true'>#title</ltx:title></ltx:bibliography>",
    after_digest => sub[whatsit] {
      stomach::bgroup();
      begin_bibliography(whatsit)?;
      let _ = stomach::egroup();
    },
    before_construct => sub[doc,whatsit] {
      adjust_backmatter_element(doc, whatsit)?;
    }
  );

  DefConstructor!("\\bibstyle{}", sub[document, _whatsit, props] {
    let style = prop_string!(props, "style");
    set_bibstyle(&style);
    if let Some(mut bib) = document.findnode("//ltx:bibliography", None) {
      if let Some(Stored::String(bs)) = lookup_value("BIBSTYLE") {
        arena::with(bs, |s| document.set_attribute(&mut bib, "bibstyle", s))?;
      }
      if let Some(Stored::String(cs)) = lookup_value("CITE_STYLE") {
        arena::with(cs, |s| document.set_attribute(&mut bib, "citestyle", s))?;
      }
      if let Some(Stored::String(so)) = lookup_value("CITE_SORT") {
        arena::with(so, |s| document.set_attribute(&mut bib, "sort", s))?;
      }
    }
  },
    after_digest => sub[whatsit] {
      let style = whatsit.get_arg(1).map(|a| a.to_string()).unwrap_or_default();
      assign_value("BIBSTYLE", arena::pin(&style), Some(Scope::Global));
      if let Some((cs, so)) = lookup_bibstyle_params(&style) {
        assign_value("CITE_STYLE", arena::pin(cs), None);
        assign_value("CITE_SORT", arena::pin(so), None);
      } else {
        Info!("unexpected", style, s!("Unknown bibstyle '{style}', it will be ignored"));
      }
    },
    properties => sub[args] {
      unpack_opt_ref!(args => style_opt);
      let style = style_opt.as_ref().map_or(String::new(), |s| s.to_string());
      Ok(stored_map!("style" => Stored::String(arena::pin(&style))))
    }
  );

  DefMacro!("\\bibliographystyle Semiverbatim", "\\bibstyle{#1}");

  DefConditional!("\\if@lx@inbibliography");
  // Should be an environment, but people seem to want to misuse it.
  DefConstructor!("\\thebibliography",
  "<ltx:bibliography xml:id='#id'><ltx:title font='#titlefont' _force_font='true'>#title</ltx:title><ltx:biblist>",
    before_digest => {
        before_digest_bibliography() },
    after_digest => sub[whatsit] {
      // NOTE that in some perverse situations (revtex?)
      // it seems to be allowable to omit the argument
      // It's ignorable for latexml anyway, so we'll just read it if its there.
      gullet::skip_spaces()?;
      if gullet::if_next(T_BEGIN!())? {
        gullet::read_arg(ExpansionLevel::Off)?;
      }
      begin_bibliography(whatsit)?;
    },
    before_construct => sub[doc,whatsit] {
      adjust_backmatter_element(doc, whatsit)?;
    },
    locked => true
  );

  // Close the bibliography
  DefConstructor!("\\endthebibliography", sub[document,_whatsit,_props] {
    document.maybe_close_element("ltx:biblist")?;
    document.maybe_close_element("ltx:bibliography")?;
  }, locked=>true);
  Let!("\\saved@endthebibliography", "\\endthebibliography");
  // auto close the bibliography and contained biblist.
  Tag!("ltx:biblist",      auto_close => true);
  Tag!("ltx:bibliography", auto_close => true);

  DefMacro!("\\par@in@bibliography", {
    gullet::skip_spaces()?;
    if let Some(tok) = gullet::read_token()? {
      // If next token is another \par, or a REAL \bibitem,
      // then this \par expands into what followed
      // Else, put it back, and start a bibitem.
      if tok == T_CS!("\\par") || tok == T_CS!("\\bibitem") {
        Ok(Tokens!(tok))
      } else {
        gullet::unread_one(tok);
        Ok(Tokens!(T_CS!("\\save@bibitem"), T_BEGIN!(), T_END!()))
      }
    } else {
      Ok(Tokens!(T_CS!("\\save@bibitem"), T_BEGIN!(), T_END!()))
    }
  });
  DefMacro!("\\vskip@in@bibliography Glue", None);
  DefMacro!("\\item@in@bibliography", "\\save@bibitem{}");

  // If we hit a real \bibitem, put \par & \bibitem back to correct defn, and then \bibitem.
  // A bibitem with now key or label...
  //
  // Porting note: careful with the escaping rules. In perl we had a '\let\\\\\save@...'
  // but if we use the r## 'raw string literal' in Rust, the extra \\ escape is not needed.
  DefMacro!(
    "\\restoring@bibitem",
    r#"\let\bibitem\save@bibitem\let\par\save@par\let\\\save@backbackslash\bibitem"#
  );

  NewCounter!("@bibitem", "bibliography", idprefix => "bib");
  DefMacro!("\\the@bibitem", "\\arabic{@bibitem}");
  DefMacro!("\\@biblabel{}", "[#1]");
  DefMacro!("\\fnum@@bibitem", "{\\@biblabel{\\the@bibitem}}");
  // Hack for abused bibliographies; see below
  DefMacro!(
    "\\bibitem",
    r#"\if@lx@inbibliography\else\expandafter\lx@mung@bibliography\expandafter{\@currenvir}\fi\lx@bibitem"#,
    locked=>true);
  // Perl latex_constructs.pool.ltxml L4134-4162: enterHorizontal => 1 + afterDigest.
  DefConstructor!("\\lx@bibitem[] Semiverbatim",
    "<ltx:bibitem key='#key' xml:id='#id'>#tags<ltx:bibblock>",
    enter_horizontal => true,
    after_digest => sub[whatsit] {
      // Perl #2409: prune previous \lx@bibitem whatsit if it was auto-opened
      // with no tag/key body, and reuse its ID (avoids empty bibitem elements).
      let pruned_prev = stomach::with_box_list(|list| {
        if let Some(prev) = list.last() {
          if let DigestedData::Whatsit(prev_ws_cell) = prev.data() {
            let prev_ws = prev_ws_cell.borrow();
            let defn = prev_ws.get_definition();
            let cs_str = defn.get_cs().to_string();
            if cs_str == "\\lx@bibitem"
              && prev_ws.get_arg(1).is_none()
              && prev_ws.get_arg(2).is_none_or(|a| a.is_empty().unwrap_or(true))
            {
              return true;
            }
          }
        }
        false
      });
      if pruned_prev {
        stomach::with_box_list_mut_vec(|list| { list.pop(); });
        Info!("empty", "bibitem",
          "Encountered an empty \\bibitem, likely auto-opened without need. Pruning and reusing its id.");
      }
      let tag_opt = whatsit.get_arg(1);
      let key = if let Some(key) = whatsit.get_arg(2) {
        clean_bib_key(&key.to_string())
      } else { String::default() };
      if let Some(tag) = tag_opt {
        let mut properties = if pruned_prev {
          RefCurrentID!("@bibitem")?
        } else {
          RefStepID!("@bibitem")?
        };
        properties.insert("key", key.into());
        let mut tag_tokens = vec![
            T_BEGIN!(), T_CS!("\\def"), T_CS!("\\the@bibitem"), T_BEGIN!()];
        tag_tokens.extend(Revert!(tag));
        tag_tokens.push(T_END!());
        tag_tokens.extend(
          Invocation!(T_CS!("\\lx@make@tags"), vec![T_OTHER!("@bibitem")]).unlist());
        tag_tokens.push(T_END!());
        properties.insert("tags",
          stomach::digest(tag_tokens)?.into());
        whatsit.set_properties(properties);
      } else {
        let mut properties = RefStepCounter!("@bibitem")?;
        properties.insert("key", key.into());
        whatsit.set_properties(properties);
      }
    }
  );

  // This attempts to handle the case where folks put \bibitem's within an enumerate or such.
  // We try to close the list and open the bibliography
  DefMacro!("\\lx@mung@bibliography{}", sub[(env)] {
    let tag = env.to_string();
    let mut tokens = Vec::new();
    // If we're in some sort of list environment, maybe we can recover
    if tag == "enumerate" || tag == "itemize" || tag == "description" {
      tokens.extend(Invocation!("\\end", vec![env]).unlist());
      tokens.extend(vec![
        T_CS!("\\let"),
        T_CS!(format!("\\end{tag}")),
        T_CS!("\\endthebibliography"),
        T_CS!("\\let"),
        T_CS!(format!("\\end{{{tag}}}")),
        T_CS!("\\end{thebibliography}")
      ]);
    }
    // else ? it probably isn't going to work??
    //Now, try to open {thebibliography}
    tokens.push(T_CS!("\\lx@mung@bibliography@pre"));
    tokens.push(T_CS!("\\thebibliography"));
    Ok(Tokens::new(tokens))
  });
  // Perl: maybeCloseElement($tag) if tag =~ /^ltx:(?:itemize|enumerate|description)$/
  DefConstructor!("\\lx@mung@bibliography@pre", sub[document] {
    let parent     = document.get_node();
    let tag_sym    = model::get_node_qname(parent);
    arena::with(tag_sym, |tag|
      if tag == "ltx:itemize" || tag == "ltx:enumerate" || tag == "ltx:description" {
        document.maybe_close_element(tag)
      } else { Ok(None) }
    )?;
  });

  // Perl latex_constructs.pool.ltxml L4187-4189: enterHorizontal => 1.
  DefConstructor!("\\lx@bibnewblock", sub[document] {
  if document.is_openable("ltx:bibblock") {
    document.open_element("ltx:bibblock",None,None)?;
  }}, enter_horizontal => true);
  Let!("\\newblock", "\\lx@bibnewblock");
  Tag!("ltx:bibitem",  auto_open => true, auto_close => true);
  Tag!("ltx:bibblock", auto_open => true, auto_close => true);

  //----------------------------------------------------------------------
  // We've got the same problem as LaTeX: Lather, Rinse, Repeat.
  // It would be nice to know the bib info at digestion time
  //  * whether author lists will collapse
  //  * whether there are "a","b".. extensions on the year.
  // We could process the bibliography first, (IF it is a separate *.bib!)
  // but won't know which entries are included (and so can't resolve the a/b/c..)
  // until we've finished looking at (all of) the source(s) that will refer to them!
  //
  // We can do this in 2 passes, however
  //  (1) convert (latexml) both the source document(s) and the bibliography
  //  (2) extract the required bibitems and integrate (latexmlpost) it into the documents.
  // [Note that for mult-document sites, step (2) becomes 2 stages: scan and integrate]
  //
  // Here's the general layout.
  //   <ltx:cite> contains everything that the citations produce,
  //     including parens, pre-note, punctunation that precede the <ltx:bibcite>
  //     and punctuation, post-note, parens, that follow it.
  //   <ltx:bibcite show="string" bibrefs="keys" sep="" yysep="">phrases</ltx:bibcite>
  //     encodes the actual citation.
  //
  //     bibrefs : lists the bibliographic keys that will be used
  //     show    : gives the pattern for formatting using data from the bibliography
  //       It can contain:
  //         authors or fullauthors
  //         year
  //         number
  //         phrase1,phrase2,... selects one of the phrases from the content of the <ltx:bibref>
  //     This format is used as follows:
  //       If author and year is present, and a subset of the citations share the same authors,
  //         then the format is used, but the year is repeated for each citation in the subset,
  //         as a link to the bib entry.
  //       Otherwise, the format is applied to each entry.
  //
  // The design is intended to support natbib, as well as plain LaTeX.

  AssignValue!("CITE_STYLE", "numbers");
  AssignValue!("CITE_OPEN", T_OTHER!("["));
  AssignValue!("CITE_CLOSE", T_OTHER!("]"));
  AssignValue!("CITE_SEPARATOR", T_OTHER!(","));
  AssignValue!("CITE_YY_SEPARATOR", T_OTHER!(","));
  AssignValue!("CITE_NOTE_SEPARATOR", T_OTHER!(","));

  // Perl latex_constructs.pool.ltxml L4239-4241: DefConstructor('\@@cite []{}', ...,
  //   alias => '\cite', mode => 'restricted_horizontal', enterHorizontal => 1)
  DefConstructor!("\\@@cite[]{}", "<ltx:cite ?#1(class='ltx_citemacro_#1')>#2</ltx:cite>",
    alias => "\\cite", mode => "text", enter_horizontal => true);

  // \@@bibref{what to show}{bibkeys}{phrase1}{phrase2}
  // Perl latex_constructs.pool.ltxml L4244-4251: enterHorizontal => 1.
  DefConstructor!("\\@@bibref Semiverbatim Semiverbatim {}{}",
    "<ltx:bibref show='#1' bibrefs='#bibrefs' inlist='#bibunit' separator='#separator'
      yyseparator='#yyseparator'>#3#4</ltx:bibref>",
    enter_horizontal => true,
    properties => sub[args] {
      unref!(args => _show, keys, _phrase1, _phrase2);
      Ok(stored_map!("bibrefs" => clean_bib_key(&keys.to_string()),
        "separator" => match state::lookup_tokens("CITE_SEPARATOR") {
          Some(sep) => stomach::digest(sep)?.to_string(),
          None => String::new() },
        "yyseparator" => match state::lookup_tokens("CITE_YY_SEPARATOR") {
          Some(yysep) => stomach::digest(yysep)?.to_string(),
          None => String::new() },
        "bibunit" => match state::lookup_value("CITE_UNIT") {
          Some(Stored::String(s)) => arena::to_string(s),
          _ => String::new() }
      ))
    }
  );

  // Simple container for any phrases used in the bibref
  // Perl latex_constructs.pool.ltxml L4254-4255: enterHorizontal => 1.
  DefConstructor!("\\@@citephrase{}", "<ltx:bibrefphrase>#1</ltx:bibrefphrase>",
    mode => "text", enter_horizontal => true);

  DefMacro!("\\cite[] Semiverbatim", sub[(post_opt, keys)] {
    // let style = state::lookup_tokens("CITE_STYLE").unwrap_or(NO_TOKENS);
    let open = state::lookup_tokens("CITE_OPEN");
    let open = open.unwrap_or(NO_TOKENS);
    let close = state::lookup_tokens("CITE_CLOSE").unwrap_or(NO_TOKENS);
    let mut post_tokens = match post_opt {
      Some(tks) => tks.unlist(),
      None => Vec::new()
    };
    if !post_tokens.is_empty() {
      let ns = state::lookup_tokens("CITE_NOTE_SEPARATOR").unwrap_or(NO_TOKENS);
      let mut post_wrapped = ns.unlist();
      post_wrapped.push(T_SPACE!());
      post_wrapped.extend(post_tokens);
      post_tokens = post_wrapped;
    }
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens!(), keys, Tokens!(), Tokens!()]);
    let mut arg_tokens = open.unlist();
    arg_tokens.extend(bibref.unlist());
    arg_tokens.extend(post_tokens);
    arg_tokens.extend(close.unlist());

    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("cite")), Tokens::new(arg_tokens)]))
  }, robust => true);

  // Perl L4271-4278: \nocite — defer to document end for MakeBibliography
  DefMacro!("\\nocite{}", sub[args] {
    let key = args.first().map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let mut toks = vec![T_CS!("\\lx@mark@nocite"), T_BEGIN!()];
    toks.extend(key.unlist());
    toks.push(T_END!());
    let _ = state::push_value("@at@end@document", Stored::Tokens(Tokens::new(toks)));
    Ok(Tokens!())
  });
  DefConstructor!(
    "\\lx@mark@nocite Semiverbatim",
    "<ltx:cite><ltx:bibref show='nothing' bibrefs='#bibrefs' inlist='#bibunit'/></ltx:cite>",
    properties => sub[args] {
      let key = args[0].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      // Perl CleanBibKey: trim + remove internal spaces
      let bibrefs: String = key.chars().filter(|c| !c.is_whitespace()).collect();
      let bibunit = state::lookup_value("CITE_UNIT")
        .map(|v| v.to_string()).unwrap_or_default();
      Ok(stored_map!("bibrefs" => bibrefs, "bibunit" => bibunit))
    }
  );


  // #======================================================================
  // # C.11.4 Splitting the input
  // #======================================================================
  // NOTE: do NOT `Let!(\@@input, \input)` here. The Let in
  // `latex_bootstrap.rs:48` already aliased `\@@input` to the raw
  // TeX `\input` (the engine-init version from `tex_file_io.rs`)
  // BEFORE the dump load installed latex.ltx's redefined `\input`
  // (`\@ifnextchar\bgroup\@iinput\@@input`). Doing the Let again
  // here would re-alias `\@@input` to THAT redefined `\input` —
  // a self-recursive macro that loops at the false branch:
  // `\@@input snippet` → `\@@input` (itself) → infinite recursion
  // → TokenLimit. Triggered by `\verbatimlisting{snippet}` in
  // tests/tokenize/verb.tex.
  // LaTeX's \input is a bit different...

  // Input, now
  DefPrimitive!("\\ltx@input {}", sub[(arg)] { Input!(&Expand!(arg).to_string()); });
  DefMacro!("\\input", "\\@ifnextchar\\bgroup\\@iinput\\@@input");
  Let!("\\@iinput", "\\ltx@input");
  DefMacro!(
    "\\@input{}",
    "\\IfFileExists{#1}{\\@@input\\@filef@und}{\\typeout{No file #1.}}"
  );
  DefMacro!(
    "\\@input@{}",
    "\\InputIfFileExists{#1}{}{\\typeout{No file #1.}}"
  );

  DefMacro!("\\quote@name{}", "\"\\quote@@name#1\\@gobble\"\"");
  DefMacro!("\\quote@@name{} Match:\"", "#1\\quote@@name");
  DefMacro!("\\unquote@name{}", "\\quote@@name#1\\@gobble\"");

  // Perl L4313-4315: \include — input a file, respecting \includeonly
  DefPrimitive!("\\include{}", sub[(path)] {
    let path_str = Expand!(path).to_string();
    // Check if \includeonly restricts inclusion
    let table = state::lookup_value("including@only");
    let should_include = match table {
      None => true, // no \includeonly — include everything
      Some(Stored::HashString(map)) => map.contains_key(&path_str),
      _ => true,
    };
    if should_include {
      Input!(&path_str);
    }
  });

  // Perl L4303-4311: \includeonly — restrict which files \include loads
  DefPrimitive!("\\includeonly{}", sub[(paths)] {
    let paths_str = Expand!(paths).to_string();
    let mut map = rustc_hash::FxHashMap::default();
    for part in paths_str.split(',') {
      let trimmed = part.trim().to_string();
      if !trimmed.is_empty() {
        map.insert(trimmed, "1".to_string());
      }
    }
    state::assign_value("including@only", Stored::HashString(map), Scope::Global);
  });

  // Perl latex_constructs L4316-4353: {filecontents} and {filecontents*} environments
  // Read raw lines until \end{filecontents}, cache content for later \input.
  fn cache_filecontents(end_marker: &str, header_star: bool) -> Result<()> {
    gullet::skip_spaces()?;
    let filename_toks = gullet::read_arg(ExpansionLevel::Off)?;
    let filename = filename_toks.to_string();
    // Perl latex_constructs L4316-4353: header comments match Perl's
    // three-line preamble. The \jobname line is synthesized as `\jobname`
    // (unexpanded literal) rather than the digested jobname — our tests
    // don't exercise a specific date and we don't want to leak
    // compile-time state into the dump-like content cache.
    let mut lines: Vec<String> = vec![
      format!("%% LaTeX2e file `{filename}'"),
      if header_star {
        "%% generated by the `filecontents*' environment".to_string()
      } else {
        "%% generated by the `filecontents' environment".to_string()
      },
      "%% from source `\\jobname' on YYYY/MM/DD.".to_string(),
    ];
    if !header_star { lines.push("%%".to_string()); }
    // Discard remainder of \begin{filecontents} line
    gullet::read_raw_line();
    // Read raw lines until end marker
    loop {
      match gullet::read_raw_line() {
        Some(line) if !line.contains(end_marker) => lines.push(line),
        _ => break,
      }
    }
    let n = lines.len();
    let content = lines.join("\n");
    Info!("note", "filecontents", s!("Cached filecontents for {filename} ({n} lines)"));
    state::assign_value(&s!("{filename}_contents"), Stored::from(content), Some(Scope::Global));
    Ok(())
  }
  // Perl: DefConstructorI(T_CS("\\begin{filecontents}"), "Semiverbatim", '', afterDigest => ...)
  // The \filecontents primitive reads filename + raw lines until \end{filecontents}.
  // When called via \begin{filecontents}, \begin opens a group first.
  // We manually close the group after caching, matching the \end that was consumed.
  DefPrimitive!("\\filecontents", {
    cache_filecontents("\\end{filecontents}", false)?;
    // \begin{filecontents} opens a \begingroup; since we consumed \end{filecontents}
    // as raw text, we must close the group that \begin opened.
    stomach::endgroup()?;
  });
  DefPrimitive!("\\lx@filecontents@star", {
    cache_filecontents("\\end{filecontents*}", true)?;
    // Same: close the \begingroup from \begin{filecontents*}
    stomach::endgroup()?;
  });
  state::assign_meaning(
    &T_CS!("\\filecontents*"),
    state::lookup_meaning(&T_CS!("\\lx@filecontents@star")).unwrap_or(Stored::None),
    Some(Scope::Global),
  );
  DefMacro!("\\endfilecontents", "");
  state::assign_meaning(
    &T_CS!("\\endfilecontents*"),
    state::lookup_meaning(&T_CS!("\\endfilecontents")).unwrap_or(Stored::None),
    Some(Scope::Global),
  );


  Tag!("ltx:indexphrase", after_close => sub[_document, node] {
    add_index_phrase_key(node)?;
  });
  Tag!("ltx:glossaryphrase", after_close => sub[_document, node] {
    add_index_phrase_key(node)?;
  });

  // \@index[style][inlist]{phrases} → <ltx:indexmark>
  DefConstructor!("\\@index[][]{}", "^<ltx:indexmark style='#1' inlist='#2'>#3</ltx:indexmark>",
    bounded => true,
    mode => "restricted_horizontal",
    sizer => 0
  );

  // \@indexphrase[sortkey]{phrase} → <ltx:indexphrase>
  DefConstructor!("\\@indexphrase[]{}", "<ltx:indexphrase key='#key'>#2</ltx:indexphrase>",
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      if key.is_empty() {
        Ok(stored_map!())
      } else {
        Ok(stored_map!("key" => key))
      }
    }
  );

  // \@indexsee{key} → <ltx:indexsee>
  DefConstructor!("\\@indexsee{}", "<ltx:indexsee key='#key'>#1</ltx:indexsee>",
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      Ok(stored_map!("key" => key))
    }
  );

  // \@indexseealso{key} → <ltx:indexsee>
  DefConstructor!("\\@indexseealso{}", "<ltx:indexsee key='#key'>#1</ltx:indexsee>",
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      Ok(stored_map!("key" => key))
    }
  );

  // \index{phrases} — expand to \@index via process_index_phrases.
  // Perl: latex_constructs.pool.ltxml L4454 uses the SanitizedVerbatim
  // parameter type so that `\index{a_b}`, `\index{with spaces}`, etc. don't
  // fail tokenization on chars that normally have non-OTHER catcodes.
  DefMacro!("\\index SanitizedVerbatim", sub[(phrases)] {
    process_index_phrases(Tokens::new(phrases.revert()))
  });

  DefMacro!("\\indexname", "Index");
  DefEnvironment!("{theindex}",
    "<ltx:index xml:id='#id'>#body</ltx:index>");

  DefPrimitive!("\\indexspace", None);
  DefPrimitive!("\\makeindex", None);
  DefPrimitive!("\\makeglossary", None);
  // \printindex removed — not in Perl engine (defined in makeidx.sty.ltxml)

  // Perl: \glossary{} — simplified glossary entry
  DefConstructor!("\\glossary{}", "<ltx:glossaryphrase role='glossary' key='#key'>#1</ltx:glossaryphrase>",
    properties => sub[args] {
      let key = args[0].as_ref()
        .map(|a| clean_index_key(&a.to_string()))
        .unwrap_or_default();
      Ok(stored_map!("key" => key))
    },
    sizer => 0
  );

  // \glossaryname, \printglossary removed — not in Perl engine
  // (Perl: glossaries.sty.ltxml defines \printglossary)
  // \seename, \alsoname removed — not in Perl engine
  // (Perl: makeidx.sty.ltxml defines these; also babel captions)

  //======================================================================
  // Perl: latex_constructs.pool.ltxml L4536-4564 — index constructors

  // Helper: close an open indexphrase element
  DefConstructor!("\\index@dotfill", sub[document] {
    if document.is_closeable("ltx:indexphrase").is_some() {
      document.close_element("ltx:indexphrase")?;
    }
    document.open_element("ltx:indexrefs", None, None)?;
  });

  DefConstructor!("\\index@item", sub[document] {
    do_index_item(document, 1)?;
  });
  DefConstructor!("\\index@subitem", sub[document] {
    do_index_item(document, 2)?;
  });
  DefConstructor!("\\index@subsubitem", sub[document] {
    do_index_item(document, 3)?;
  });
  DefConstructor!("\\index@done", sub[document] {
    do_index_item(document, 0)?;
  });

  //======================================================================
  // C.11.6 Terminal Input and Output
  //======================================================================
  DefPrimitive!("\\typeout{}", sub[(stuff)] {
    if state::current_verbosity() > -1 {
      let content = Expand!(stuff);
      Note!(s!("{content}"));
    }
  });
  DefPrimitive!("\\typein[]{}", None);


  // ======================================================================
  // C.12-C.13 Line/Page Breaking, Boxes
  // ======================================================================


  //======================================================================
  // C.12.1 Line Breaking
  //======================================================================
  DefPrimitive!("\\linebreak[]");
  DefPrimitive!("\\nolinebreak[]");
  DefPrimitive!("\\-"); // We don't do hyphenation.
  // \hyphenation in TeX.pool
  DefPrimitive!("\\sloppy");
  DefPrimitive!("\\fussy");
  // sloppypar can be used as an environment, or by itself.
  DefMacro!("\\sloppypar", "\\par\\sloppy");
  DefMacro!("\\endsloppypar", "\\par");
  DefMacro!("\\nobreakdashes", "-");
  DefMacro!("\\showhyphens{}", "#1");
  //======================================================================
  // C.12.2 Page Breaking
  //======================================================================
  DefMacro!("\\pagebreak[Default:4]", sub[(arg_opt)] {
      let arg : u32 = if let Some(arg_t) = arg_opt {
        arg_t.to_string().parse::<u32>().unwrap_or(0)
      } else { 0 };
      if arg <= 2 {
        Ok(Tokens!()) }
      else {
        Ok(Invocation!(T_CS!("\\vadjust"), vec![T_CS!("\\clearpage")]))
      }
  });
  DefPrimitive!("\\nopagebreak[]");
  DefPrimitive!("\\columnbreak"); // latex? or multicol?
  DefPrimitive!("\\enlargethispage OptionalMatch:* {}");

  DefMacro!("\\clearpage", "\\lx@newpage");
  DefMacro!("\\cleardoublepage", "\\lx@newpage");
  DefPrimitive!("\\samepage");


  //======================================================================
  // C.13.1 Length
  //======================================================================
  // \fill
  DefMacro!("\\stretch{}", "0pt plus #1fill\\relax");

  DefPrimitive!("\\@check@length DefToken", sub[(cs)] {
    match lookup_definition(&cs)? {
      None => {
        let message = s!("'{}' is not a length; defining it now", cs.stringify());
        Warn!("undefined", cs, message);
        DefRegister!(cs, None, Dimension::new(0));
      },
      Some(defn) => if !defn.is_register() {
        let message = s!("'{}' length was expected, got {:?} instead of register.",
          cs.to_string(), defn.register_type());
        Error!("misdefined", cs, message);
      }
    };
  });

  DefPrimitive!("\\newlength DefToken", sub[(cs)] {
    DefRegister!(cs, None, Glue::new(0), allocate => "\\skip");
    Ok(vec![])
  });

  // Perl parity: `return unless $defn && ($defn ne 'missing');` — silently
  // skip when the target variable has no register definition (e.g. undefined
  // length register). Matches calc_sty.rs's \setlength/\addtolength fallback.
  DefPrimitive!("\\setlength {Variable}{Dimension}", sub[(variable,length)] {
    if let ArgWrap::RegisterDefinition(dbox) = variable {
      let (rtoken, params) = *dbox;
      if let Some(defn) = rtoken.to_register() {
        defn.set_value(length.into(), None, params);
      }
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\addtolength {Variable}{Dimension}", sub[(variable,length)] {
    if let ArgWrap::RegisterDefinition(dbox) = variable {
      let (rtoken, params) = *dbox;
      if let Some(defn) = rtoken.to_register() {
        // TODO: can we avoid cloning the params?
        let oldlength = defn.value_of(params.clone()).unwrap_or_default();
        defn.set_value(oldlength.add(length), None, params);
      }
    }
    Ok(Vec::new())
  });

  DefMacro!(
    "\\@settodim{}{}{}",
    "\\setbox\\@tempboxa\\hbox{{#3}}#2#1\\@tempboxa\\setbox\\@tempboxa\\box\\voidb@x"
  );
  DefMacro!("\\settoheight", "\\@settodim\\ht");
  DefMacro!("\\settodepth", "\\@settodim\\dp");
  DefMacro!("\\settowidth", "\\@settodim\\wd");
  // \settototalheight sets its register to \ht+\dp of the box. Perl
  // calc.sty.ltxml L73-77 models it as a DefPrimitive that directly
  // sums getHeight+getDepth; we follow the same trampoline shape as
  // the sibling \setto* macros and use \advance to add the depth.
  DefMacro!(
    "\\settototalheight{}{}",
    "\\setbox\\@tempboxa\\hbox{{#2}}\
     #1\\ht\\@tempboxa\
     \\advance#1\\dp\\@tempboxa\
     \\setbox\\@tempboxa\\box\\voidb@x"
  );
  DefMacro!(r"\@settopoint{}", r"\divide#1\p@\multiply#1\p@");

  DefRegister!("\\fill", Glue!("0pt plus 1fill"));

  //======================================================================
  // C.13.2 Space
  //======================================================================

  DefPrimitive!("\\hspace OptionalMatch:* {Dimension}", sub[(_star,length)] {
    let s = dimension_to_spaces(length);
    if !s.is_empty() {
      let length_tokens = length.revert()?;

      let tokens = Invocation!(T_CS!("\\hskip"), vec![length_tokens]);
      Tbox::new(arena::pin(&s), None, None, tokens,
        stored_map!("width" => length, "isSpace" => true));
    }
  });

  // Perl: DefMacro('\vspace OptionalMatch:* {}', '\vskip #2\relax');
  //
  // Rust uses DefPrimitive with a None body — effectively a silent
  // no-op that swallows the optional-star + dimension arg. Intentional
  // DefMacro → DefPrimitive kind divergence (WISDOM #44): the literal
  // Perl port (expanding to `\vskip #2\relax`) triggered paragraph
  // breaks in moderncv that cascaded into multiple test failures. The
  // no-op stub matches Perl's _observable_ effect on most documents
  // (LaTeXML ignores vertical spacing) without the moderncv break.
  // See docs/SYNC_STATUS.md Work-Plan batch B5 for the broader
  // deferred vspace→\vskip port plan.
  DefPrimitive!("\\vspace OptionalMatch:* {}", None);
  DefPrimitive!("\\addvspace {}", None);
  DefPrimitive!("\\addpenalty {}", None);
  DefPrimitive!("\\@endparenv", None);

  //======================================================================
  // C.13.3 Boxes
  //======================================================================
  // Can't really get these?
  DefMacro!("\\height", "0pt");
  DefMacro!("\\totalheight", "0pt");
  DefMacro!("\\depth", "0pt");
  DefMacro!("\\width", "0pt");

  DefConstructor!("\\mbox {}", "<ltx:text _noautoclose='1'>#1</ltx:text>",
    mode => "text",
    bounded => true,
    sizer => "#1",
    before_digest => {
      reenter_text_mode(false); }
  );

  // our %makebox_alignment = (l => 'left', r => 'right', s => 'justified');
  // Perl latex_constructs.pool.ltxml L4717: `robust => 1` so \makebox
  // survives \write/\edef contexts (e.g. captions, moving arguments).
  // Rust was missing the flag.
  DefMacro!("\\makebox", "\\@ifnextchar(\\pic@makebox\\@makebox",
    robust => true);
  // Perl: enterHorizontal => 1 (now automatic via mode => "text")
  DefConstructor!("\\@makebox[Dimension][]{}",
    "<ltx:text ?#width(width='#width') ?#align(align='#align') _noautoclose='1'>#3</ltx:text>",
    mode         => "text", bounded => true, alias => "\\makebox", sizer => "#3",
    before_digest => {
      reenter_text_mode(false); },
    properties   => sub[args] {
      // Perl: (($_[2] ? (align => $makebox_alignment{...}) : ()), ($_[1] ? (width => $_[1]) : ()))
      let mut props = stored_map!();
      if let Some(ref dim_d) = args[0] {
        if let DigestedData::RegisterValue(v) = dim_d.data() {
          let dim: Dimension = v.into();
          props.insert("width", Stored::from(dim));
        }
      }
      if let Some(ref align_d) = args[1] {
        let align_str = align_d.to_string();
        let align = match align_str.as_str() {
          "l" => "left",
          "r" => "right",
          "s" => "justified",
          _ => "",
        };
        if !align.is_empty() {
          props.insert("align", Stored::from(align));
        }
      }
      Ok(props)
    }
  );

  DefRegister!("\\fboxrule", Dimension!(".4pt"));
  DefRegister!("\\fboxsep", Dimension!("3pt"));

  // Peculiar special case!
  //  These are nominally text mode macros. However, there is a somewhat common idiom:
  //     $ ... \framebox{$operator$} ... $
  // in which case the operator gets boxed and really should be treated as a math object.
  // (and ultimately converted to mml:menclose)
  // So, we need to switch to text mode, as usual, but FIRST note whether we started in math mode!
  // Afterwards, if we were in math mode, and the content is math, we'll convert the whole thing
  // to a framed math object.
  // Second special issue:
  //   Although framebox doesn't allow flowed content inside, it is also somewhat common
  // to put a vbox or some other block construct inside.
  // Seemingly, the ultimate html gets somewhat tangled (browser bugs?)
  // At any rate, since we're wrapping with an ltx:text, we'll try to unwrap it,
  // if the contents are a single child that can handle the framing.

  // Perl latex_constructs.pool.ltxml L4744-4745: both \fbox and
  // \framebox are defined with `robust => 1` so they survive
  // \write/\edef moving-argument contexts.
  DefMacro!("\\fbox{}", "\\@framebox{#1}", robust => true);
  DefMacro!("\\framebox", "\\@ifnextchar(\\pic@framebox\\@framebox",
    robust => true);
  // Perl: DefConstructor('\@framebox[Dimension][]{}', ...)
  // Perl uses restricted_horizontal mode, saves IN_MATH, unwraps single children
  // When in math mode, produces <ltx:XMArg enclose='box'> instead of <ltx:text framed='rectangle'>
  DefConstructor!("\\@framebox[Dimension][]{}",
    "?#mathframe(<ltx:XMArg enclose='box'>#inner</ltx:XMArg>)\
     (<ltx:text ?#width(width='#width') ?#align(align='#align') ?#cssstyle(cssstyle='#cssstyle') framed='rectangle' framecolor='#framecolor' _noautoclose='1'>#3</ltx:text>)",
    alias => "\\framebox",
    sizer => "#3",
    before_digest => {
      // Perl: $wasmath = LookupValue('IN_MATH') — uses boolean value, not key existence.
      // IN_MATH is initialized to false at startup, so is_some() would always be true.
      let wasmath = state::lookup_bool_sym(pin!("IN_MATH"));
      stomach::begin_mode("restricted_horizontal")?;
      state::assign_value("FRAME_IN_MATH", wasmath, None); },
    properties => sub[args] {
      // Perl: framecolor => LookupValue('font')->getColor
      let framecolor = lookup_font()
        .and_then(|f| f.get_color().cloned())
        .map(|c| c.to_attribute())
        .unwrap_or_else(|| s!("#000000"));
      let mut props = stored_map!("framecolor" => framecolor);
      // Perl: align from arg 2 (optional []) — only set when explicitly given
      // Perl: only emit align for l/r/s; 'c' (center) is default → not emitted
      if let Some(align_val) = args[1].as_ref() {
        let align_str = align_val.to_string();
        let mapped = match align_str.as_str() {
          "l" => Some("left"),
          "r" => Some("right"),
          "s" => Some("justified"),
          _ => None, // 'c' or empty → default center, not emitted
        };
        if let Some(m) = mapped {
          props.insert("align", Stored::String(arena::pin_static(m)));
        }
      }
      if let Some(width_val) = args[0].as_ref() {
        props.insert("width", Stored::String(arena::pin(width_val.to_attribute())));
      }
      // Perl: ($sep ne '3.0pt' ? (cssstyle => 'padding:' . $sep) : ())
      if let Some(sep) = lookup_dimension("\\fboxsep") {
        let sep_str = sep.to_attribute();
        if sep_str != "3.0pt" {
          props.insert("cssstyle", Stored::String(arena::pin(s!("padding:{sep_str}"))));
        }
      }
      Ok(props)
    },
    after_digest => sub[whatsit] {
      let wasmath = state::lookup_bool("FRAME_IN_MATH");
      let arg = whatsit.get_arg(3).cloned();
      stomach::end_mode("restricted_horizontal")?;
      if wasmath {
        if let Some(ref a) = arg {
          // Perl: $arg->isMath checks mode property =~ /math$/
          // For \fbox{$...$}, the body is a List in restricted_horizontal mode
          // containing a math whatsit. Check if any child has isMath.
          let is_math = a.get_property_bool("isMath")
            || a.unlist().iter().any(|child| child.get_property_bool("isMath"));
          if is_math {
            whatsit.set_property("mathframe", true);
            // Extract inner body for the XMArg template
            // For \fbox{$...$}, get the math body from the inner whatsit
            if let Ok(Some(body)) = a.get_body() {
              whatsit.set_property("inner", body);
            } else {
              // Fallback: use the entire arg
              whatsit.set_property("inner", a.clone());
            }
          }
        }
      }
    },
    after_construct => sub[document, _whatsit] {
      // Perl afterConstruct: if the <ltx:text> has a single non-text child
      // that can have 'framed', unwrap the text and copy attributes to the child.
      let current = document.get_node().clone();
      if let Some(node) = current.get_last_child() {
        if document::get_node_qname(&node) != arena::pin_static("ltx:text") {
          return Ok(());
        }
        // Filter to non-whitespace children
        let children: Vec<Node> = node.get_child_nodes().into_iter().filter(|n| {
          if n.get_type() == Some(NodeType::ElementNode) {
            true
          } else {
            // text node — keep only if non-whitespace
            n.get_content().chars().any(|c| !c.is_whitespace())
          }
        }).collect();
        if children.len() == 1
          && children[0].get_type() == Some(NodeType::ElementNode)
          && document::can_node_have_attribute(&children[0], "framed")
          && !children[0].has_attribute("framed")
        {
          // Copy attributes from ltx:text to child, then unwrap
          for attr in ["width", "align", "framed"] {
            if let Some(v) = node.get_attribute(attr) {
              document.set_attribute(&mut children[0].clone(), attr, &v)?;
            }
          }
          document.unwrap_nodes(node)?;
        }
      }
    }
  );

  AssignValue!("SAVEBOX", 100);
  TeX!(
    r#"""\def\newsavebox#1{\@ifdefinable{#1}{\newbox#1}}
  \DeclareRobustCommand\savebox[1]{%
    \@ifnextchar(%)
      {\@savepicbox#1}{\@ifnextchar[{\@savebox#1}{\sbox#1}}}%
  \DeclareRobustCommand\sbox[2]{\setbox#1\hbox{%
    \color@setgroup#2\color@endgroup}}
  \def\@savebox#1[#2]{%
    \@ifnextchar [{\@isavebox#1[#2]}{\@isavebox#1[#2][c]}}
  \long\def\@isavebox#1[#2][#3]#4{%
    \sbox#1{\@imakebox[#2][#3]{#4}}}
  \def\@savepicbox#1(#2,#3){%
    \@ifnextchar[%]
      {\@isavepicbox#1(#2,#3)}{\@isavepicbox#1(#2,#3)[]}}
  \long\def\@isavepicbox#1(#2,#3)[#4]#5{%
    \sbox#1{\@imakepicbox(#2,#3)[#4]{#5}}}
  \def\lrbox#1{%
    \edef\reserved@a{%
      \endgroup
      \setbox#1\hbox{%
        \begingroup\aftergroup}%
          \def\noexpand\@currenvir{\@currenvir}%
          \def\noexpand\@currenvline{\on@line}}%
    \reserved@a
      \@endpefalse
      \color@setgroup
        \ignorespaces}
  \def\endlrbox{\unskip\color@endgroup}
  \DeclareRobustCommand\usebox[1]{\leavevmode\copy #1\relax}
  """#
  );

  // DefMacro!(T_CS!("\\begin{lrbox}"), '{Token}', "\@begin@lrbox #1");
  // DefPrimitive!("\\end{lrbox}", primtiveproc!( args, {stomach.egroup()?; }));
  // DefPrimitive!("\\@begin@lrbox Token", sub {
  //     my ($stomach, $token) = @_;
  //     $stomach->bgroup;
  //     my $box = List($stomach->digestNextBody());
  //     AssignValue('box' . ToString($token), $box); });

  // DefPrimitive!("\\usebox {Register}", sub {
  //     my ($defn) = @{ $_[1] };
  //     return Box() unless $defn && ($defn ne 'missing');
  //     my $value = $defn->valueOf()->valueOf;
  //     LookupValue('box' . $value) || Box(); });

  // A soft sorta \par that only closes an ltx:p, but not ltx:para
  DefConstructor!("\\lx@parboxnewline[]", sub[document, _args, _props] {
    document.maybe_close_element("ltx:p")?;
  });

  // Perl: latex_constructs.pool.ltxml lines 4795-4818
  Let!("\\lx@parboxnewline", "\\lx@newline");
  // NOTE: There are 2 extra arguments (See LaTeX Companion, p.866)
  // for height and inner-pos. We're ignoring inner-pos, for now, though.
  DefMacro!("\\parbox[] [] [] {Dimension}{}",
    r"\lx@hidden@bgroup\hsize=#4\textwidth\hsize\columnwidth\hsize\ifx.#2.\lx@parbox[#1]{#4}{#5}\else\lx@parbox[#1][#2][#3]{#4}{#5}\fi\lx@hidden@egroup");
  DefConstructor!("\\lx@parbox[][Dimension] OptionalUndigested {Dimension} VBoxContents",
    sub[document, args, props] {
      let body = args[4].as_ref().unwrap();
      let mut attr = string_map!("class" => "ltx_parbox");
      if let Some(w) = props.get("width") { attr.insert("width".to_string(), w.to_string()); }
      if let Some(v) = props.get("vattach") { attr.insert("vattach".to_string(), v.to_string()); }
      insert_block(document, body, attr)?;
    },
    alias => "\\parbox",
    properties => sub[args] {
      let attachment = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let width = args[3].as_ref().map(|w| w.to_attribute()).unwrap_or_default();
      Ok(stored_map!("width" => width, "vattach" => translate_attachment(&attachment)))
    },
    // Sizer: width from arg #4 (Dimension), height/depth from body (arg #5)
    // Perl: sizer => '#5' — uses font.computeBoxesSize(body, vattach => ..., width => ...)
    // which does proper line breaking and vattach transformation.
    sizer => sub[whatsit] {
      // Width from the "width" property (arg #4 Dimension)
      let w = whatsit.get_property("width")
        .and_then(|s| Dimension::new_f64(Dimension::spec_to_f64(&s.to_string()).ok()?).into())
        .unwrap_or_default();
      // Height/depth from body (arg #5 VBoxContents)
      if let Some(body) = whatsit.get_arg(5) {
        let w_val = w.value_of();
        if w_val > 0 {
          // Approximate paragraph height: measure total unwrapped width,
          // estimate lines, use \baselineskip for line height.
          let (body_w, body_h, body_d) = body.compute_size(SymHashMap::default())?;
          let total_w = body_w.value_of();
          let (mut ht, mut dp) = if total_w > w_val {
            // Paragraph wrapping: estimate number of lines
            let num_lines = ((total_w as f64) / (w_val as f64)).ceil() as i64;
            // Use \baselineskip (typically 12pt = 786432 sp) for line height
            let baseline_skip = state::lookup_dimension("\\baselineskip")
              .unwrap_or(Dimension::new(786432)); // 12pt default
            let line_h = baseline_skip.value_of();
            let total_h = num_lines * line_h;
            // Default: top alignment (first line as height)
            let first_line_h = body_h.value_of().max(line_h * 2 / 3);
            (first_line_h, total_h - first_line_h)
          } else {
            (body_h.value_of(), body_d.value_of())
          };
          // Perl Font.pm L793-800: apply vattach transformation
          let vattach = whatsit.get_property("vattach")
            .map(|v| v.to_string())
            .unwrap_or_default();
          let total = ht + dp;
          if vattach == "middle" {
            let font_size = lookup_font()
              .and_then(|f| f.get_size().map(|s| s as i64))
              .unwrap_or(10);
            let hh = total / 2;
            let c = font_size * UNITY / 4; // math axis ≈ size/4
            ht = hh + c;
            dp = hh - c;
          } else if vattach == "bottom" {
            // Align to baseline of bottom row
            let last_line_d = body_d.value_of().min(total);
            dp = last_line_d;
            ht = total - dp;
          }
          // else: "top"/"baseline" — keep first line as height (default above)
          Ok((w, Dimension::new(ht), Dimension::new(dp)))
        } else {
          let (_, h, d) = body.compute_size(SymHashMap::default())?;
          Ok((w, h, d))
        }
      } else {
        Ok((w, Dimension::default(), Dimension::default()))
      }
    },
    mode => "internal_vertical",
    before_digest => {
      Let!("\\\\", "\\lx@newline");
    }
  );
  DefMacro!("\\@parboxrestore", "");

  DefConditional!("\\if@minipage");
  DefMacro!("\\@setminipage", "");
  // Perl: latex_constructs.pool.ltxml lines 4822-4846
  DefEnvironment!("{minipage}[] OptionalUndigested [] {Dimension}",
    sub[document, args, props] {
      let attachment = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let vattach = translate_attachment(&attachment);
      let width = match props.get("width") {
        Some(Stored::Dimension(d)) => d.to_attribute(),
        Some(w) => w.to_string(),
        None => args.get(3).and_then(|a| a.as_ref()).map(|a| a.to_attribute())
          .unwrap_or_default(),
      };
      let mut attr = string_map!("class" => "ltx_minipage");
      if !width.is_empty() { attr.insert("width".to_string(), width); }
      attr.insert("vattach".to_string(), vattach.to_string());
      if let Some(Stored::Digested(body)) = props.get("body") {
        insert_block(document, body, attr)?;
      }
      Ok(())
    },
    mode => "internal_vertical",
    before_digest => {
      stomach::digest(Tokens!(T_CS!("\\@minipagetrue")))?;
    },
    after_digest_begin => sub[whatsit] {
      // Perl: afterDigestBegin sets \hsize, \textwidth, \columnwidth from width arg
      let vattach = whatsit.get_arg(1)
        .map(|a| translate_attachment(a.to_string()))
        .unwrap_or("middle");
      if let Some(width_arg) = whatsit.get_arg(4) {
        let width_val = width_arg.value_of();
        let dim = Dimension::new(width_val);
        let rv: RegisterValue = dim.into();
        state::assign_register("\\hsize", rv.clone(), None, Vec::new())?;
        state::assign_register("\\textwidth", rv.clone(), None, Vec::new())?;
        state::assign_register("\\columnwidth", rv, None, Vec::new())?;
        whatsit.set_property("width", Stored::Dimension(dim));
      }
      whatsit.set_property("vattach", Stored::from(vattach.to_string()));
      Let!("\\\\", "\\lx@newline");
    },
    after_digest_body => sub[whatsit] {
      // Perl: afterDigestBody copies vattach from whatsit to body
      if let Some(vattach) = whatsit.get_property("vattach").map(|v| v.into_owned()) {
        if let Some(Stored::Digested(body)) = whatsit.properties.get("body").cloned() {
          let mut body = body;
          body.set_property("vattach", vattach);
        }
      }
    }
  );

  DefConstructor!("\\rule[Dimension]{Dimension}{Dimension}",
    "<ltx:rule ?#offset(yoffset='#offset') width='#width' height='#height'/>",
    enter_horizontal => true,
    properties => sub[args] {
      Ok(stored_map!(
        "offset" => args[0].as_ref().map(|a| a.to_attribute()).unwrap_or_default(),
        "width" => args[1].as_ref().map(|a| a.to_attribute()).unwrap_or_default(),
        "height" => args[2].as_ref().map(|a| a.to_attribute()).unwrap_or_default()
      ))
    }
  );
  DefConstructor!("\\raisebox{Dimension}[Dimension][Dimension]{}",
    "<ltx:text yoffset='#1' _noautoclose='1'>#4</ltx:text>",
    mode         => "text", bounded => true,
    before_digest => {
      reenter_text_mode(false); }
    // TODO
    // sizer        => sub { raisedSizer($_[0]->getArg(4), $_[0]->getArg(1)); }
  );


  // ======================================================================
  // C.14-C.15 Pictures, Fonts, Symbols
  // ======================================================================


  // Not sure that ltx:p is the best to use here, but ... (see also \vbox, \vtop)
  // This should be fairly compact vertically.
  DefConstructor!("\\@shortstack@cr",
    "</ltx:p><ltx:p>",
    properties   => { stored_map!("isBreak" => true) },
    reversion    => Tokens!(T_CS!("\\\\"), T_CR!()),
    before_digest => { egroup()?; },
    after_digest  => { bgroup(); });

  DefConstructor!("\\shortstack[]{}  OptionalMatch:* [Dimension]",
  "<ltx:inline-block align='#align'><ltx:p>#2</ltx:p></ltx:inline-block>",
  bounded      => true,
  sizer        => "#2",
  before_digest => {
    // Rebind \\ and \lx@newline to shortstack line break.
    // Matches Perl: only \\ is rebound (Perl does NOT rebind \lx@hidden@cr).
    // \lx@newline is also rebound because \\ is Let to \lx@newline at the
    // top level, so \lx@newline tokens in content must also become @shortstack@cr.
    // NOTE: \lx@hidden@cr must NOT be rebound — doing so causes is_column_end()
    // to match \\ as a column end inside alignments, because is_column_end
    // compares meanings and \lx@hidden@cr is a COLUMN_END sentinel.
    Let!("\\\\", "\\@shortstack@cr");
    Let!("\\lx@newline", "\\@shortstack@cr");
    AssignRegister!("\\baselineskip" , Glue::new_spec("-1pt", None, None, None, None).into());
    AssignRegister!("\\lineskip"     , Glue::new_spec("3pt", None, None, None, None).into());
    bgroup(); },
  after_digest => sub[_whatsit] {
    egroup()?; },
  // Note: does not get layout=vertical, since linebreaks are explicit
  properties => sub[args] {
    let align = args[0].as_ref().map(|a| {
      match a.to_string().as_str() {
        "l" => "left", "r" => "right", _ => ""
      }
    }).unwrap_or("");
    Ok(stored_map!("align" => align, "vattach" => "bottom"))
  },
  mode => "restricted_horizontal");

  //======================================================================
  // C.14.1 The picture Environment
  // Perl: latex_constructs.pool.ltxml lines 4927-5185
  //======================================================================

  // Registers
  DefRegister!("\\unitlength" => Dimension!("1pt"));
  DefRegister!("\\@wholewidth" => Dimension!("0.4pt"));
  DefRegister!("\\@halfwidth" => Dimension!("0.2pt"));

  // \thinlines / \thicklines — set \@wholewidth register
  // Perl L4928-4929: DefPrimitiveI — assigns \@wholewidth register directly at
  // stomach level (not via TeX-level expansion). Faithful port.
  DefPrimitive!("\\thinlines", {
    state::assign_register(
      "\\@wholewidth",
      latexml_core::definition::register::RegisterValue::Dimension(Dimension!("0.4pt")),
      None,
      vec![],
    )?;
  });
  DefPrimitive!("\\thicklines", {
    state::assign_register(
      "\\@wholewidth",
      latexml_core::definition::register::RegisterValue::Dimension(Dimension!("0.8pt")),
      None,
      vec![],
    )?;
  });
  DefMacro!("\\linethickness{}", "\\@wholewidth #1\\relax");
  // Perl L4933: DefPrimitive('\arrowlength{Dimension}', sub { AssignValue('arrowlength', $_[1]); });
  // Stores the dimension under state key `arrowlength` for later lookup
  // by the picture drawing routines (see Perl L4978-4979).
  DefPrimitive!("\\arrowlength {Dimension}", sub[(length)] {
    state::assign_value("arrowlength", Stored::Dimension(length), None);
  });
  DefMacro!("\\qbeziermax", "500");
  // Perl: \bezier — LaTeX 2.09 compat alias for \qbezier with different syntax
  DefMacro!("\\bezier Until:(", "\\ifx.#1.\\lx@pic@bezier{0}(\\else\\lx@pic@bezier{#1}(\\fi");
  DefMacro!("\\lx@pic@bezier{} Pair Pair Pair", "\\qbezier[#1]#2#3#4");
  DefMacro!("\\@killglue", "\\unskip\\@whiledim \\lastskip >\\z@\\do{\\unskip}");

  // Tag: ltx:picture — Perl latex_constructs.pool.ltxml L4995:
  //   Tag('ltx:picture', autoOpen => 0.5, autoClose => 1, afterOpen => &GenerateID)
  // The 0.5 fractional priority is honoured by `compute_indirect_model` in
  // state.rs: picture is the only tag with lower-than-full openability, so
  // other auto-openers (para, p, text, item, …) win whenever they can also
  // reach the target element. Picture is selected only for picture-specific
  // primitives (\line, \circle, \vector, \put) used bare inside a {figure}
  // or similar context where no fuller wrapper fits.
  Tag!("ltx:picture",
    auto_open  => true,
    auto_close => true,
    after_open => sub[document, node] {
      document.generate_id(node, "pic")?;
    }
  );

  // {picture} environment: (width,height) with optional (origin-x,origin-y)
  // Pair now survives digestion via RegisterValue::Pair, so properties can extract coordinates.
  DefEnvironment!("{picture} Pair OptionalPair",
    "<ltx:picture width='#width' height='#height' origin-x='#origin-x' origin-y='#origin-y'\
      fill='none' stroke='none' unitlength='#unitlength'>\
      ?#transform(<ltx:g transform='#transform'>#body</ltx:g>)(#body)\
    </ltx:picture>",
    mode => "internal_vertical",
    before_digest => {
      // Perl: before_picture — Let \raisebox to \pic@raisebox
      Let!("\\raisebox", "\\pic@raisebox");
    },
    properties => sub[args] {
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let (w, h) = match args[0].as_ref() {
        Some(d) => match d.data() {
          DigestedData::RegisterValue(RegisterValue::Pair(p)) => (p.x.0 * unit, p.y.0 * unit),
          _ => (0.0, 0.0),
        },
        None => (0.0, 0.0),
      };
      // Perl Float formats with at least one decimal place
      let fmt_pt = |v: f64| -> String {
        if v == v.round() { format!("{v:.1}pt") } else { format!("{v}pt") }
      };
      let mut map = stored_map!(
        "width"      => Stored::String(arena::pin(fmt_pt(w))),
        "height"     => Stored::String(arena::pin(fmt_pt(h))),
        "unitlength" => Stored::String(arena::pin(fmt_pt(unit)))
      );
      // Origin from OptionalPair — Perl: origin-x, origin-y, transform
      if let Some(d) = args[1].as_ref() {
        if let DigestedData::RegisterValue(RegisterValue::Pair(p)) = d.data() {
          let ox = p.x.0 * unit;
          let oy = p.y.0 * unit;
          map.insert("origin-x", Stored::String(arena::pin(fmt_pt(ox))));
          map.insert("origin-y", Stored::String(arena::pin(fmt_pt(oy))));
          // Perl: translate(negate(origin).pxValue)
          let tx = px_value(-ox);
          let ty = px_value(-oy);
          map.insert("transform", Stored::String(arena::pin(
            format!("translate({},{})", fmt_px(tx), fmt_px(ty)))));
        }
      }
      Ok(map)
    }
  );

  // \put(x,y){content} — Perl: Match:( reads "(", Until:, reads y, Until:) reads y
  // Now that Pair survives digestion (RegisterValue::Pair), use it directly.
  DefMacro!("\\put SkipSpaces Match:( Until:, Until:) {}", "\\lx@pic@put(#2,#3){#4\\relax}");
  DefConstructor!("\\lx@pic@put Pair {}",
    "<ltx:g transform='#transform' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth'>#2</ltx:g>",
    alias => "\\put",
    mode  => "restricted_horizontal",
    properties => sub[args] {
      let (x, y) = match args[0].as_ref() {
        Some(d) => match d.data() {
          DigestedData::RegisterValue(RegisterValue::Pair(p)) => (p.x.0, p.y.0),
          _ => { let s = d.to_string(); let mut p = s.splitn(2, ',');
            (p.next().unwrap_or("0").trim().parse().unwrap_or(0.0),
             p.next().unwrap_or("0").trim().parse().unwrap_or(0.0)) }
        },
        None => (0.0, 0.0),
      };
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let tx = px_value(x * unit);
      let ty = px_value(y * unit);
      let transform_str = format!("translate({},{})", fmt_px(tx), fmt_px(ty));
      // Perl: $box->getSize to extract inner dimensions
      let (iw, ih, id) = if let Some(body) = args[1].as_ref() {
        let (w, h, d, _, _, _) = body.clone().get_size(None)?;
        // Perl: $w = undef if $w && ($w->ptValue == 0)
        let w_opt = if w.value_of() == 0 { None } else { Some(w) };
        (w_opt, Some(h), Some(d))
      } else {
        (None, None, None)
      };
      let mut map = stored_map!(
        "transform" => Stored::String(arena::pin(&transform_str))
      );
      if let Some(w) = iw { map.insert("innerwidth", Stored::Dimension(w)); }
      if let Some(h) = ih { map.insert("innerheight", Stored::Dimension(h)); }
      if let Some(d) = id { map.insert("innerdepth", Stored::Dimension(d)); }
      Ok(map)
    }
  );

  //============================================================
  // Picture primitives (\line, \vector, \oval, \qbezier, \bezier)
  //============================================================
  //
  // Umbrella WISDOM #44 intentional divergence for the block below:
  //
  // Perl defines each picture primitive as
  //   DefConstructor('\line Pair:Number {Float}', …)
  //   DefConstructor('\vector Pair:Number {Float}', …)
  //   DefConstructor('\oval Pair:Float []', …)
  //   DefConstructor('\qbezier [] Pair:Number Pair:Number Pair:Number', …)
  //   DefConstructor('\bezier {Number} Pair:Float Pair:Float Pair:Float', …)
  // using the `Pair:Number`/`Pair:Float` parameter type, which parses
  // the LaTeX `(x,y)` slope/position syntax directly into a pair of
  // numbers for the constructor's args.
  //
  // Rust doesn't have the `Pair:*` parameter type, so each port is
  // split into a DefMacro trampoline with
  // `Match:( Until:, Until:) {…}` parsing the (a,b) syntax manually,
  // followed by a hidden `\lx@pic@<name>{}{}{…}` DefConstructor that
  // takes the 3 (or more) pre-parsed args.
  //
  // Audit reports 5 DefConstructor → DefMacro kind flips across
  // \line, \vector, \oval, \qbezier, \lx@pic@bezier. All 5 carry
  // the same rationale (missing Pair:Number parameter type), so
  // individual entries don't re-carry the tag.

  // \line(slope){length} — Perl: DefConstructor('\line Pair:Number {Float}', ...)
  DefMacro!("\\line Match:( Until:, Until:) {Float}", "\\lx@pic@line{#2}{#3}{#4}");
  DefConstructor!("\\lx@pic@line{}{}{}",
    "<ltx:line points='#points' stroke='#color' stroke-width='#thick'/>",
    alias => "\\line",
    properties => sub[args] {
      let mx: f64 = args[0].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let my: f64 = args[1].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let xlength: f64 = args[2].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      // slopeToPicCoord: compute endpoint from slope and length, then convert to px
      let s = if mx > 0.0 { 1.0 } else if mx < 0.0 { -1.0 } else { 0.0 };
      let ex = px_value(xlength * unit * s);
      let ey = if s == 0.0 {
        px_value(xlength * unit * (if my > 0.0 { 1.0 } else { -1.0 }))
      } else {
        px_value(xlength * unit * my / mx.abs())
      };
      Ok(stored_map!(
        "points" => Stored::String(arena::pin(format!("0,0 {},{}", fmt_px(ex), fmt_px(ey)))),
        "thick"  => Stored::String(arena::pin(format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // \vector(slope){length} — Perl: DefConstructor('\vector Pair:Number {Float}', ...)
  DefMacro!("\\vector Match:( Until:, Until:) {Float}", "\\lx@pic@vector{#2}{#3}{#4}");
  DefConstructor!("\\lx@pic@vector{}{}{}",
    "<ltx:line points='#points' stroke='#color' stroke-width='#thick' terminators='->'/>",
    alias => "\\vector",
    properties => sub[args] {
      let mx: f64 = args[0].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let my: f64 = args[1].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let xlength: f64 = args[2].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      let s = if mx > 0.0 { 1.0 } else if mx < 0.0 { -1.0 } else { 0.0 };
      let ex = px_value(xlength * unit * s);
      let ey = if s == 0.0 {
        px_value(xlength * unit * (if my > 0.0 { 1.0 } else { -1.0 }))
      } else {
        px_value(xlength * unit * my / mx.abs())
      };
      Ok(stored_map!(
        "points" => Stored::String(arena::pin(format!("0,0 {},{}", fmt_px(ex), fmt_px(ey)))),
        "thick"  => Stored::String(arena::pin(format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // \circle*{diameter} — filled or unfilled circle
  DefConstructor!("\\circle OptionalMatch:* {Float}",
    "<ltx:circle x='0' y='0' r='#radius' fill='#fill' stroke='#stroke' stroke-width='#thick'/>",
    alias => "\\circle",
    properties => sub[args] {
      let filled = args[0].is_some(); // OptionalMatch:* → Some if * present
      let dia: f64 = args[1].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      let radius = px_value(dia * unit * 0.5);
      let (fill, stroke) = if filled {
        ("#000000", "none")
      } else {
        ("none", "#000000")
      };
      Ok(stored_map!(
        "radius" => Stored::String(arena::pin(fmt_px(radius))),
        "fill"   => fill,
        "stroke" => stroke,
        "thick"  => Stored::String(arena::pin(format!("{thick}")))
      ))
    }
  );

  // \oval[radius](width,height)[part] — decompose pair
  DefMacro!("\\oval", "\\lx@pic@oval");
  DefConstructor!("\\lx@pic@oval [Float] Pair []",
    "<ltx:rect x='#ox' y='#oy' width='#owidth' height='#oheight' rx='#radius'\
      stroke='#color' fill='none' part='#3' stroke-width='#thick'/>",
    alias => "\\oval",
    properties => sub[args] {
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      // Perl: $r = ($r ? picScale($r) : Dimension('40pt'))
      let r_requested: f64 = args[0].as_ref()
        .map(|d| d.to_string().trim().parse().unwrap_or(40.0) * unit)
        .unwrap_or(40.0);
      // Extract size from Pair
      let (sx, sy) = match args[1].as_ref() {
        Some(d) => match d.data() {
          DigestedData::RegisterValue(RegisterValue::Pair(p)) => (p.x.0 * unit, p.y.0 * unit),
          _ => (0.0, 0.0),
        },
        None => (0.0, 0.0),
      };
      let (hx, hy) = (sx * 0.5, sy * 0.5);
      // Perl: $r = $r->smaller($halfsize->getX->absolute)->smaller($halfsize->getY->absolute)
      let r = r_requested.min(hx.abs()).min(hy.abs());
      Ok(stored_map!(
        "ox"      => Stored::String(arena::pin(fmt_px(px_value(-hx)))),
        "oy"      => Stored::String(arena::pin(fmt_px(px_value(-hy)))),
        "owidth"  => Stored::String(arena::pin(fmt_px(px_value(sx)))),
        "oheight" => Stored::String(arena::pin(fmt_px(px_value(sy)))),
        "radius"  => Stored::String(arena::pin(fmt_px(px_value(r)))),
        "thick"   => Stored::String(arena::pin(s!("{thick}"))),
        "color"   => "#000000"
      ))
    }
  );

  // \qbezier[N](p1)(p2)(p3) — decompose 3 pairs into coordinates
  DefMacro!("\\qbezier [Number] Match:( Until:, Until:) Match:( Until:, Until:) Match:( Until:, Until:)",
    "\\lx@pic@qbezier{#1}{#3}{#4}{#6}{#7}{#9}{#10}");
  DefConstructor!("\\lx@pic@qbezier{}{}{}{}{}{}{}",
    "<ltx:bezier points='#points' stroke='#color' stroke-width='#thick'/>",
    alias => "\\qbezier",
    properties => sub[args] {
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      // args: [0]=N, [1]=x1, [2]=y1, [3]=x2, [4]=y2, [5]=x3, [6]=y3
      let parse_f = |i: usize| -> f64 {
        args[i].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0)
      };
      let (x1, y1) = (px_value(parse_f(1) * unit), px_value(parse_f(2) * unit));
      let (x2, y2) = (px_value(parse_f(3) * unit), px_value(parse_f(4) * unit));
      let (x3, y3) = (px_value(parse_f(5) * unit), px_value(parse_f(6) * unit));
      Ok(stored_map!(
        "points" => Stored::String(arena::pin(format!("{},{} {},{} {},{}",
          fmt_px(x1), fmt_px(y1), fmt_px(x2), fmt_px(y2), fmt_px(x3), fmt_px(y3)))),
        "thick"  => Stored::String(arena::pin(format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // Perl L5166-5175: \multiput expands to n \put commands with coordinate stepping.
  DefMacro!("\\multiput Match:( Until:, Until:) Match:( Until:, Until:) {}{}", sub[args] {
    // args: 0=Match:(, 1=x, 2=y, 3=Match:(, 4=dx, 5=dy, 6=n, 7=body
    let x_str = args.get(1).map(|a| a.revert().unwrap_or_default().to_string()).unwrap_or_default();
    let y_str = args.get(2).map(|a| a.revert().unwrap_or_default().to_string()).unwrap_or_default();
    let dx_str = args.get(4).map(|a| a.revert().unwrap_or_default().to_string()).unwrap_or_default();
    let dy_str = args.get(5).map(|a| a.revert().unwrap_or_default().to_string()).unwrap_or_default();
    let n: i64 = args.get(6).map(|a| a.revert().unwrap_or_default().to_string()
      .trim().parse().unwrap_or(1)).unwrap_or(1);
    let body = args.get(7).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();

    let mut x: f64 = x_str.trim().parse().unwrap_or(0.0);
    let mut y: f64 = y_str.trim().parse().unwrap_or(0.0);
    let dx: f64 = dx_str.trim().parse().unwrap_or(0.0);
    let dy: f64 = dy_str.trim().parse().unwrap_or(0.0);

    // Each iteration emits roughly `8 + body.len()` tokens; pre-size
    // conservatively + use borrow-iter-copied for body to avoid the
    // per-iteration Vec<Token> clone.
    let body_len = body.len();
    let mut result = Vec::with_capacity(((n as usize) * (8 + body_len)).min(1 << 20));
    for _ in 0..n {
      result.push(T_CS!("\\put"));
      result.push(T_OTHER!("("));
      result.extend(Explode!(s!("{}", x)));
      result.push(T_OTHER!(","));
      result.extend(Explode!(s!("{}", y)));
      result.push(T_OTHER!(")"));
      result.push(T_BEGIN!());
      result.extend(body.unlist_ref().iter().copied());
      result.push(T_END!());
      x += dx;
      y += dy;
    }
    Ok(Tokens::new(result))
  });

  // Box commands for picture mode
  // Perl: \pic@makebox@ Undigested RequiredKeyVals Pair []{} — the master box constructor
  // Creates optional <rect> for frame + <g class="makebox"> for content with positioning.
  // Properties compute inner dimensions from $box->getSize and position from [pos] arg.
  //
  // The Perl macros are:
  //   \pic@makebox  → \pic@makebox@{\makebox}{}
  //   \pic@framebox → \pic@makebox@{\framebox}{framed=true}
  //   \frame{}      → \pic@makebox@{\framebox}{framed=true}(0,0)[bl]{#1}
  //   \dashbox      → \pic@makebox@{\dashbox(N)}{framed=true,dash={N}}
  //
  // For now: simplified port without getSize (uses zero defaults).
  // The constructor uses sub[] to build DOM directly matching Perl's output structure.
  DefConstructor!("\\pic@makebox@ Undigested {} Pair []{}",
    sub[document, args, props] {
      // args: [0]=cs(Undigested), [1]=kv_text({}), [2]=size(Pair), [3]=pos([]), [4]=box({})
      let framed = props.get("framed").is_some();
      // \@wholewidth captured at digest time in properties callback
      let thick = match props.get("thick") {
        Some(Stored::String(s)) => arena::with(*s, |v| v.parse::<f64>().unwrap_or(0.4)),
        _ => 0.4,
      };
      // Frame rect (only when framed=true)
      if framed {
        let mut rect_attrs = map!(
          "x" => "0".to_string(), "y" => "0".to_string(),
          "width" => props.get("fwidth").map(|s| s.to_string()).unwrap_or_else(|| "0".into()),
          "height" => props.get("fheight").map(|s| s.to_string()).unwrap_or_else(|| "0".into()),
          "stroke" => "#000000".to_string(),
          "stroke-width" => format!("{thick}"),
          "fill" => "none".to_string()
        );
        if let Some(dash) = props.get("dash") {
          rect_attrs.insert("stroke-dasharray".to_string(), dash.to_string());
        }
        document.insert_element("ltx:rect", Vec::new(), Some(rect_attrs))?;
      }
      // Content <g>
      let mut g_attrs = map!("class" => "makebox".to_string());
      for &key in &["innerwidth", "innerheight", "innerdepth"] {
        if let Some(v) = props.get(key) {
          let vs = v.to_string();
          if !vs.is_empty() {
            g_attrs.insert(key.to_string(), vs);
          }
        }
      }
      let xshift = props.get("xshift").map(|s| s.to_string()).unwrap_or_else(|| s!("0"));
      let yshift = props.get("yshift").map(|s| s.to_string()).unwrap_or_else(|| s!("0"));
      g_attrs.insert(s!("transform"), format!("translate({xshift},{yshift})"));
      document.open_element("ltx:g", Some(g_attrs), None)?;
      if let Some(body) = args.get(4).and_then(|a| a.as_ref()) {
        document.absorb(body, None)?;
      }
      document.close_element("ltx:g")?;
    },
    properties => sub[args] {
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      // Capture \@wholewidth at digest time for frame stroke-width
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      // args: [0]=cs, [1]=kv_text, [2]=size(Pair), [3]=pos, [4]=box
      let kv_str = args[1].as_ref().map(|d| d.to_string()).unwrap_or_default();

      // Perl: $box->getSize — extract (width, height, depth) from body
      let (w, h, d) = if let Some(body) = args[4].as_ref() {
        let (bw, bh, bd, _, _, _) = body.clone().get_size(None)?;
        (bw, bh, bd)
      } else {
        (Dimension::default(), Dimension::default(), Dimension::default())
      };
      let ht = Dimension::new(h.value_of() + d.value_of()); // total height = h + d

      // Extract frame size from Pair parameter (args[2])
      let (mut ww, mut hh) = match args[2].as_ref() {
        Some(d) => match d.data() {
          DigestedData::RegisterValue(RegisterValue::Pair(p)) => {
            (Dimension::new((p.x.0 * unit * 65536.0) as i64),
             Dimension::new((p.y.0 * unit * 65536.0) as i64))
          },
          _ => (Dimension::default(), Dimension::default()),
        },
        None => (Dimension::default(), Dimension::default()),
      };

      // Perl: position-based shift computation
      let (mut xshift, mut yshift) = (Dimension::default(), Dimension::default());
      if ww.value_of() != 0 || hh.value_of() != 0 {
        let pos = args[3].as_ref().map(|d| d.to_string().to_lowercase()).unwrap_or_default();
        // x positioning
        if pos.contains('l') {
          xshift = Dimension::default(); // left-aligned: x = 0
        } else if pos.contains('r') {
          xshift = Dimension::new(ww.value_of() - w.value_of()); // right-aligned
        } else {
          xshift = Dimension::new((ww.value_of() - w.value_of()) / 2); // centered
        }
        // y positioning
        if pos.contains('b') {
          yshift = Dimension::default(); // bottom-aligned: y = 0
        } else if pos.contains('t') {
          yshift = Dimension::new(hh.value_of() - ht.value_of()); // top-aligned
        } else {
          yshift = Dimension::new((hh.value_of() - ht.value_of()) / 2); // centered
        }
      } else {
        ww = w;
        hh = Dimension::new(h.value_of() + d.value_of());
      }

      // Frame dimensions: use ww/hh if nonzero, else content size
      let fw = if ww.value_of() != 0 { ww } else { w };
      let fh = if hh.value_of() != 0 { hh } else { Dimension::new(h.value_of() + d.value_of()) };

      let xs_px = px_value(xshift.pt_value(None));
      let ys_px = px_value(yshift.pt_value(None));

      let mut map = stored_map!(
        "innerwidth" => Stored::Dimension(w),
        "innerheight" => Stored::Dimension(h),
        "innerdepth" => Stored::Dimension(d),
        "fwidth" => Stored::Dimension(fw),
        "fheight" => Stored::Dimension(fh),
        "xshift" => Stored::String(arena::pin(fmt_px(xs_px))),
        "yshift" => Stored::String(arena::pin(fmt_px(ys_px)))
      );
      if kv_str.contains("framed") {
        map.insert("framed", Stored::Bool(true));
      }
      if let Some(dash_start) = kv_str.find("dash={") {
        let rest = &kv_str[dash_start + 6..];
        if let Some(end) = rest.find('}') {
          map.insert("dash", Stored::String(arena::pin(&rest[..end])));
        }
      }
      map.insert("thick", Stored::String(arena::pin(s!("{thick}"))));
      Ok(map)
    },
    mode => "text"
  );

  // Perl macro aliases
  DefMacro!("\\pic@makebox",            "\\pic@makebox@{\\makebox}{}");
  DefMacro!("\\pic@framebox",           "\\pic@makebox@{\\framebox}{framed=true}");
  DefMacro!("\\lx@pic@dashbox{Float}",  "\\pic@makebox@{\\dashbox(#1)}{framed=true,dash={#1}}");
  DefMacro!("\\dashbox Until:(",
    "\\ifx.#1.\\lx@pic@dashbox{0}(\\else\\lx@pic@dashbox{#1}(\\fi");
  DefMacro!("\\frame{}",
    "\\pic@makebox@{\\framebox}{framed=true}(0,0)[bl]{#1}");

  // \pic@raisebox — simplified raisebox for picture mode
  DefConstructor!("\\pic@raisebox{Dimension}[Dimension][Dimension]{}",
    "<ltx:g y='#1'>#4</ltx:g>",
    alias => "\\raisebox"
  );

  // Perl: latex_constructs.pool.ltxml line 4862
  // Stubs for color/xcolor packages (overridden when color.sty is loaded)
  Let!("\\set@color", "\\relax");
  Let!("\\color@begingroup", "\\relax");
  Let!("\\color@endgroup", "\\relax");
  Let!("\\color@setgroup", "\\relax");
  Let!("\\color@hbox", "\\relax");
  Let!("\\color@vbox", "\\relax");
  Let!("\\color@endbox", "\\relax");

  // Perl: latex_constructs.pool.ltxml line 5802
  // \stop — closes the current input mouth (Plain TeX command)
  Let!("\\stop", "\\endinput");
  DefMacro!("\\ignorespacesafterend", None);

  // Perl: latex_constructs.pool.ltxml line 5027
  // Pre-define \Gin@driver so graphics.sty doesn't error when loaded from disk
  DefMacro!("\\Gin@driver", "");


  //**********************************************************************
  // C.15 Font Selection
  //**********************************************************************
  //======================================================================
  // C.15.1 Changing the Type Style
  //======================================================================
  // Text styles.

  DefMacro!("\\rmdefault", "cmr");
  DefMacro!("\\sfdefault", "cmss");
  DefMacro!("\\ttdefault", "cmtt");
  DefMacro!("\\bfdefault", "bx");
  DefMacro!("\\mddefault", "m");
  DefMacro!("\\itdefault", "it");
  DefMacro!("\\sldefault", "sl");
  DefMacro!("\\scdefault", "sc");
  DefMacro!("\\updefault", "n");
  DefMacro!("\\encodingdefault", "OT1");
  DefMacro!("\\familydefault", "\\rmdefault");
  DefMacro!("\\seriesdefault", "\\mddefault");
  DefMacro!("\\shapedefault", "\\updefault");

  Let!("\\mediumseries", "\\mdseries");
  Let!("\\normalshape", "\\upshape");

  // ? DefMacro("\\f@encoding','cm');
  DefMacro!("\\f@family", "cmr");
  DefMacro!("\\f@series", "m");
  DefMacro!("\\f@shape", "n");
  DefMacro!("\\f@size", "10");

  // These do NOT immediately effect the font!
  DefMacro!("\\fontfamily{}", "\\edef\\f@family{#1}");
  DefMacro!("\\fontseries{}", "\\edef\\f@series{#1}");
  DefMacro!("\\fontshape{}", "\\edef\\f@shape{#1}");

  // For fonts not allowed in math!!!
  // Perl L5226: \not@math@alphabet@@ checks if we're in math mode
  // LaTeX kernel also defines \not@math@alphabet (2 args) — stub both
  // Perl L5349: DefMacro('\not@math@alphabet{}{}', ...) — conditional error
  // message in math mode, no-op otherwise. Rust keeps the no-op stub but
  // matches the Perl kind (DefMacro — expansion-time, same as the
  // invocation sites `\mdseries`/`\bfseries` which expand it inline).
  DefMacro!("\\not@math@alphabet{}{}", None);
  DefPrimitive!("\\not@math@alphabet@@ {}", sub[(c)] {
    if state::lookup_bool_sym(pin!("IN_MATH")) {
      let c = c.to_string();
      let message = s!("Command {:?} invalid in math mode", c);
      Warn!("unexpected", c, message);
    }
    Ok(vec![])
  });

  // These DO immediately effect the font!
  DefMacro!(
    "\\mdseries",
    "\\not@math@alphabet@@{\\mddefault}\\fontseries{\\mddefault}\\selectfont"
  );
  DefMacro!(
    "\\bfseries",
    "\\not@math@alphabet@@{\\bfdefault}\\fontseries{\\bfdefault}\\selectfont"
  );

  DefMacro!(
    "\\rmfamily",
    "\\not@math@alphabet@@{\\rmdefault}\\fontfamily{\\rmdefault}\\selectfont"
  );
  DefMacro!(
    "\\sffamily",
    "\\not@math@alphabet@@{\\sfdefault}\\fontfamily{\\sfdefault}\\selectfont"
  );
  DefMacro!(
    "\\ttfamily",
    "\\not@math@alphabet@@{\\ttdefault}\\fontfamily{\\ttdefault}\\selectfont"
  );

  DefMacro!(
    "\\upshape",
    "\\not@math@alphabet@@{\\updefault}\\fontshape{\\updefault}\\selectfont"
  );
  DefMacro!(
    "\\itshape",
    "\\not@math@alphabet@@{\\itdefault}\\fontshape{\\itdefault}\\selectfont"
  );
  DefMacro!(
    "\\slshape",
    "\\not@math@alphabet@@{\\sldefault}\\fontshape{\\sldefault}\\selectfont"
  );
  DefMacro!(
    "\\scshape",
    "\\not@math@alphabet@@{\\scdefault}\\fontshape{\\scdefault}\\selectfont"
  );

  DefMacro!(
    "\\normalfont",
    "\\fontfamily{\\rmdefault}\\fontseries{\\mddefault}\\fontshape{\\updefault}\\selectfont"
  );
  DefMacro!(
    "\\verbatim@font",
    "\\fontfamily{\\ttdefault}\\fontseries{\\mddefault}\\fontshape{\\updefault}\\selectfont"
  );

  Let!("\\reset@font", "\\normalfont");

  DefPrimitive!("\\selectfont", {
    let family = Expand!(T_CS!("\\f@family")).to_string();
    let series = Expand!(T_CS!("\\f@series")).to_string();
    let shape = Expand!(T_CS!("\\f@shape")).to_string();
    if let Some(sh) = font::lookup_font_family(&family) {
      MergeFont!(sh.clone());
    } else {
      let message = s!("Unrecognized font family {:?}.", family);
      Info!("unexpected", family, message);
    }
    if let Some(sh) = font::lookup_font_series(&series) {
      MergeFont!(sh.clone());
    } else {
      let message = s!("Unrecognized font series {:?}.", series);
      Info!("unexpected", series, message);
    }
    if let Some(sh) = font::lookup_font_shape(&shape) {
      MergeFont!(sh.clone());
    } else {
      let message = s!("Unrecognized font shape {:?}.", shape);
      Info!("unexpected", shape, message);
    }
    Ok(Vec::new())
  });

  DefMacro!(
    "\\usefont{}{}{}{}",
    "\\fontencoding{#1}\\fontfamily{#2}\\fontseries{#3}\\fontshape{#4}\\selectfont"
  );

  // If these series or shapes appear in math, they revert it to roman, medium, upright (?)
  DefConstructor!("\\textmd@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { series => "medium" }, alias => "\\textmd",
    before_digest => { DefMacro!("\\f@series", "m"); });
  DefConstructor!("\\textbf@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { series => "bold" }, alias => "\\textbf",
    before_digest => { DefMacro!("\\f@series", "b"); });
  DefConstructor!("\\textrm@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>",
    mode => "text", bounded => true, font => { family => "serif" }, alias => "\\textrm",
    before_digest => { DefMacro!("\\f@family", "cm"); });
  DefConstructor!("\\textsf@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { family => "sansserif" }, alias => "\\textsf",
    before_digest => { DefMacro!("\\f@family", "cmss"); });
  DefConstructor!("\\texttt@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { family => "typewriter" }, alias => "\\texttt",
    before_digest => { DefMacro!("\\f@family", "cmtt"); });
  DefConstructor!("\\textup@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "upright" }, alias => "\\textup",
    before_digest => { DefMacro!("\\f@shape", ""); });
  DefConstructor!("\\textit@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "italic" }, alias => "\\textit",
    before_digest => { DefMacro!("\\f@shape", "i"); });
  DefConstructor!("\\textsl@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "slanted" }, alias => "\\textsl",
    before_digest => { DefMacro!("\\f@shape", "sl"); });
  DefConstructor!("\\textsc@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "smallcaps" }, alias => "\\textsc",
    before_digest => { DefMacro!("\\f@shape", "sc"); });
  DefConstructor!("\\textnormal@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode =>
  "text",   bounded => true, font => { family => "serif", series => "medium", shape => "upright"
  }, alias => "\\textnormal",   before_digest => {
    DefMacro!("\\f@family", "cmtt");
    DefMacro!("\\f@series", "m");
    DefMacro!("\\f@shape",  "n"); });

  // These really should be robust! which is a source of expand timing issues!
  DefMacro!("\\textmd{}",     "\\ifmmode\\textmd@math{#1}\\else{\\mdseries #1}\\fi",       protected => true);
  DefMacro!("\\textbf{}",     "\\ifmmode\\textbf@math{#1}\\else{\\bfseries #1}\\fi",       protected => true);
  DefMacro!("\\textrm{}",     "\\ifmmode\\textrm@math{#1}\\else{\\rmfamily #1}\\fi",       protected => true);
  DefMacro!("\\textsf{}",     "\\ifmmode\\textsf@math{#1}\\else{\\sffamily #1}\\fi",       protected => true);
  DefMacro!("\\texttt{}",     "\\ifmmode\\texttt@math{#1}\\else{\\ttfamily #1}\\fi",       protected => true);
  DefMacro!("\\textup{}",     "\\ifmmode\\textup@math{#1}\\else{\\upshape #1}\\fi",        protected => true);
  DefMacro!("\\textit{}",     "\\ifmmode\\textit@math{#1}\\else{\\itshape #1}\\fi",        protected => true);
  DefMacro!("\\textsl{}",     "\\ifmmode\\textsl@math{#1}\\else{\\slshape #1}\\fi",        protected => true);
  DefMacro!("\\textsc{}",     "\\ifmmode\\textsc@math{#1}\\else{\\scshape #1}\\fi",        protected => true);
  DefMacro!("\\textnormal{}", "\\ifmmode\\textnormal@math{#1}\\else{\\normalfont #1}\\fi", protected => true);

  // Perl: latex_constructs.pool.ltxml line 5365
  // \DeclareOldFontCommand{\cmd}{text-font-switch}{math-font-cmd}
  // Defines \cmd to use text-font-switch in text mode, math-font-cmd in math mode.
  DefPrimitive!("\\DeclareOldFontCommand{}{}{}", sub[(cmd, font, mathcmd)] {
    // cmd contains a CS token like \bf; get the first token
    let cmd_cs = *cmd.unlist_ref().first()
      .ok_or("DeclareOldFontCommand: expected a CS token")?;
    // Move `font` and `mathcmd` directly into the closure capture —
    // they're not used outside. Saves two Tokens clones at setup time.
    DefMacro!(cmd_cs, None, ExpansionBody::Closure(Rc::new(move |_args| {
      if state::lookup_bool_sym(pin!("IN_MATH")) {
        Ok(mathcmd.clone())
      } else {
        Ok(font.clone())
      }
    })));
    Ok(Vec::new())
  });

  // Perl L5333-5339: \DeclareTextFontCommand — creates a text font command.
  // Simplified: \cmd{} → {\font #1} (group with font change).
  DefPrimitive!("\\DeclareTextFontCommand DefToken {}", sub[(cmd, font)] {
    let cs = cmd;
    let font_rev: Tokens = font;
    // Build expansion: {<font> #1}
    let mut expansion = vec![T_BEGIN!()];
    expansion.extend(font_rev.unlist());
    expansion.push(T_PARAM!());
    expansion.push(T_OTHER!("1"));
    expansion.push(T_END!());
    // init_flag=true: engine is up at \DeclareTextFontCommand expansion
    // time, so Parameter::init() can resolve readers via PARAMETER_TYPES.
    // With init=false the declared command's Plain arg uses the mock
    // reader and fails to consume input at invocation.
    let params = parse_parameters("{}", &cs, true)?;
    def_macro(cs, params,
      Some(ExpansionBody::Tokens(Tokens::new(expansion))), None)?;
  });

  // Perl L5341-5348: \mathversion — switches between bold/normal math fonts
  // Perl L5373: \newfont{cmd}{fontname} — legacy LaTeX font command
  DefMacro!("\\newfont{}{}", "\\font#1=#2\\relax");
  // Perl L5375: \normalcolor — default no-op (overridden by color.sty)
  Let!("\\normalcolor", "\\relax");

  // Perl L5364: \math@version default
  DefMacro!("\\math@version", "normal");

  // Perl L5341-5348: \mathversion — switches between bold/normal math fonts
  DefPrimitive!("\\mathversion{}", sub[(version)] {
    let v = version.to_string();
    match v.trim() {
      "bold" => { MergeFont!(forcebold => true); },
      "normal" => { MergeFont!(forcebold => false); },
      _ => {},
    }
  });

  //======================================================================
  // C.15.3 Special Symbol
  //======================================================================
  DefMacro!("\\symbol{}", "\\char#1\\relax");

  // These in LaTeX, but not in the book...
  DefPrimitive!("\\textdollar", "$");
  DefPrimitive!("\\textemdash", "\u{2014}"); // EM DASH
  DefPrimitive!("\\textendash", "\u{2013}"); // EN DASH
  DefPrimitive!("\\textexclamdown", "\u{00A1}"); // INVERTED EXCLAMATION MARK
  DefPrimitive!("\\textquestiondown", "\u{00BF}"); // INVERTED QUESTION MARK
  DefPrimitive!("\\textquotedblleft", "\u{201C}"); // LEFT DOUBLE QUOTATION MARK
  DefPrimitive!("\\textquotedblright", "\u{201D}"); // RIGHT DOUBLE QUOTATION MARK
  DefPrimitive!("\\textquotedbl", "\""); // plain ascii DOUBLE QUOTATION
  DefPrimitive!("\\textquoteleft", "\u{2018}"); // LEFT SINGLE QUOTATION MARK
  DefPrimitive!("\\textquoteright", "\u{2019}"); // RIGHT SINGLE QUOTATION MARK
  DefPrimitive!("\\textsterling", "\u{00A3}"); // POUND SIGN
  DefPrimitive!("\\textasteriskcentered", "*");
  DefPrimitive!("\\textbackslash", "\u{005C}"); // REVERSE SOLIDUS
  DefPrimitive!("\\textbar", "|");
  DefPrimitive!("\\textbraceleft", "{");
  DefPrimitive!("\\textbraceright", "}");
  DefPrimitive!("\\textbullet", "\u{2022}"); // BULLET
  DefPrimitive!("\\textdaggerdbl", "\u{2021}"); // DOUBLE DAGGER
  DefPrimitive!("\\textdagger", "\u{2020}"); // DAGGER
  DefPrimitive!("\\textparagraph", "\u{00B6}"); // PILCROW SIGN
  DefPrimitive!("\\textperiodcentered", "\u{00B7}"); // MIDDLE DOT
  DefPrimitive!("\\textsection", "\u{00A7}"); // SECTION SIGN
  // Perl: DefPrimitive('\textcircled {}', sub { ... })
  // Uses unicode_enclosed_alphanumerics table, falls back to combining circle U+20DD
  DefPrimitive!("\\textcircled {}", sub[(arg)] {
    let text = arg.to_string();
    let content = unicode_enclosed_alphanumeric(&text)
      .unwrap_or_else(|| format!("{}\u{20DD}", text));
    let in_math = state::lookup_bool_sym(pin!("IN_MATH"));
    let is_number = !text.is_empty() && text.chars().all(|c| c.is_ascii_digit());
    let mut props = stored_map!();
    if in_math {
      props.insert("role", Stored::from(if is_number { "NUMBER" } else { "UNKNOWN" }));
      props.insert("meaning", Stored::from(format!("circled-{}", text)));
    }
    Tbox::new(arena::pin(&content), None, None,
      Invocation!(T_CS!("\\textcircled"), vec![arg]),
      props)
  });
  // From latex_constructs.pool.ltxml
  DefAccent!("\\k", '\u{0328}', "\u{02DB}", below => true); // COMBINING OGONEK & OGONEK
  DefPrimitive!("\\textless", "<");
  DefPrimitive!("\\textgreater", ">");
  DefPrimitive!("\\textcopyright", "\u{00A9}"); // COPYRIGHT SIGN
  DefPrimitive!("\\textasciicircum", "^");
  DefPrimitive!("\\textasciitilde", "~");
  DefPrimitive!("\\textcompwordmark", ""); // ???
  DefPrimitive!("\\textcapitalcompwordmark", ""); // ???
  DefPrimitive!("\\textascendercompwordmark", ""); // ???
  DefPrimitive!("\\textunderscore", "_");
  // SYMBOL FOR SPACE;  Not really the right symbol!
  DefPrimitive!("\\textvisiblespace", "\u{2423}");
  DefPrimitive!("\\textellipsis", "\u{2026}"); // HORIZONTAL ELLIPSIS
  DefPrimitive!("\\textregistered", "\u{00AE}"); // REGISTERED SIGN
  DefPrimitive!("\\texttrademark", "\u{2122}"); // TRADE MARK SIGN
  DefConstructor!("\\textsuperscript{}", "<ltx:sup>#1</ltx:sup>",  mode => "text");
  // Perl L5424-5425: locked variant for \@makefnmark
  DefConstructor!("\\@textsuperscript{}", "<ltx:sup>#1</ltx:sup>",
    mode => "text", locked => true);
  DefConstructor!("\\textsubscript{}", "<ltx:sub>#1</ltx:sub>",  mode => "text");
  // This is something coming from xetex/xelatex ? Why define this way?
  //DefConstructor!("\\realsuperscript{}', "<ltx:text yoffset='0.5em'
  // _noautoclose='1'>#1</ltx:text>");
  DefConstructor!("\\realsuperscript{}", "<ltx:sup>#1</ltx:sup>",  mode => "text");
  DefPrimitive!("\\textordfeminine", "\u{00AA}"); // FEMININE ORDINAL INDICATOR
  DefPrimitive!("\\textordmasculine", "\u{00BA}"); // MASCULINE ORDINAL INDICATOR
  DefPrimitive!("\\SS", "SS"); // ?

  DefMacro!("\\dag", "\\ifmmode{\\dagger}\\else\\textdagger\\fi");
  DefMacro!("\\ddag", "\\ifmmode{\\ddagger}\\else\\textdaggerdbl\\fi");

  DefConstructor!(
    "\\sqrtsign Digested",
    "<ltx:XMApp><ltx:XMTok meaning='square-root'/><ltx:XMArg>#1</ltx:XMArg></ltx:XMApp>"
  );

  DefPrimitive!("\\mathparagraph", "\u{00B6}");
  DefPrimitive!("\\mathsection", "\u{00A7}");
  DefPrimitive!("\\mathdollar", "$");
  DefPrimitive!("\\mathsterling", "\u{00A3}");
  DefPrimitive!("\\mathunderscore", "_");
  DefPrimitive!("\\mathellipsis", "\u{2026}");

  // Perl: plain_constructs.pool.ltxml — glyph pieces that also work as delimiters
  DefMath!("\\arrowvert", None, "|", role => "VERTBAR");
  DefMath!("\\Arrowvert", None, "\u{2225}", role => "VERTBAR");

  // The following are glyph "pieces"...
  DefPrimitive!("\\braceld", "\u{239D}"); //   left brace down part
  DefPrimitive!("\\bracelu", "\u{239B}"); //   left brace up part
  DefPrimitive!("\\bracerd", "\u{23A0}"); //   right brace down part
  DefPrimitive!("\\braceru", "\u{239E}"); //   right brace up part

  // Perl: plain_constructs.pool.ltxml
  DefMath!("\\cdotp", None, "\u{22C5}", role => "MULOP");
  DefMath!("\\ldotp", None, ".", role => "MULOP");
  // Perl: latex_constructs.pool.ltxml — intop/ointop with dynamic scriptpos/mathstyle
  DefMath!("\\intop", None, "\u{222B}", role => "INTOP", meaning => "integral",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\ointop", None, "\u{222E}", role => "INTOP", meaning => "contour-integral",
    dynamic_scriptpos => true, dynamic_mathstyle => true);

  // WHat are these? They look like superscripted parentheses, or combining accents!
  // \lhook
  // \rhook
  Let!("\\gets", "\\leftarrow");

  DefPrimitive!("\\lmoustache", "\u{23B0}");
  DefPrimitive!("\\rmoustache", "\u{23B1}");
  // Perl: plain_constructs.pool.ltxml
  DefMath!("\\mapstochar", None, "\u{21A6}", role => "ARROW", meaning => "maps-to");
  DefMath!("\\owns", None, "\u{220B}", role => "RELOP", meaning => "contains");

  // \symbol lookup symbol in font by index?

  // Perl: latex_constructs.pool.ltxml L5805
  Let!("\\mathalpha", "\\relax");

  // Perl latex_constructs.pool.ltxml L5937-5938:
  // LaTeX now includes textcomp by default.
  RequirePackage!("textcomp");

  //======================================================================
  // Perl latex_constructs.pool.ltxml L5941-5993: Case-changing
  //======================================================================

  DefMacro!(
    "\\@uclclist",
    r"\oe\OE\o\O\ae\AE\dh\DH\dj\DJ\l\L\ng\NG\ss\SS\th\TH"
  );

  DefPrimitive!("\\lx@prepare@case@mapping", {
    assign_mapping("text_uppercase", "\\i ", Some(T_LETTER!("I")));
    assign_mapping("text_uppercase", "\\j ", Some(T_LETTER!("J")));
    // Perl (latex_constructs.pool L5546-5550):
    //   my @pairs = $STATE->lookupDefinition(T_CS('\@uclclist'))
    //                     ->getExpansion->unlist;
    // — reads the RAW expansion body, NOT further expanded. Critical when
    // the pair members (\ae, \oe, ...) are robust-wrapped: deep-expanding
    // would unfold each to `\protect <cs-munged>`, shifting pair indices
    // and mis-registering the case mapping.
    let pairs: Vec<Token> = match lookup_definition_stored(&T_CS!("\\@uclclist"))? {
      Some(Stored::Expandable(exp)) => match exp.get_expansion() {
        Some(latexml_core::definition::ExpansionBody::Tokens(tks)) => {
          tks.clone().unlist()
        },
        _ => Vec::new(),
      },
      _ => Vec::new(),
    };
    let mut i = 0;
    while i + 1 < pairs.len() {
      let lower = pairs[i];
      let upper = pairs[i + 1];
      let lower_key = lower.with_str(|s| format!("{} ", s));
      let upper_key = upper.with_str(|s| format!("{} ", s));
      assign_mapping("text_uppercase", &lower_key, Some(upper));
      assign_mapping("text_lowercase", &upper_key, Some(lower));
      i += 2;
    }
  });

  DefPrimitive!("\\AddToNoCaseChangeList DefToken", sub[(cs)] {
    let key = cs.with_str(|s| s.trim_end().to_string());
    assign_mapping("text_case_exclude", &key, Some(true));
  });

  DefMacro!("\\NoCaseChange {}", "#1", robust => true);

  DefMacro!("\\lx@latex@changecase {} GeneralText", sub[(case, tokens)] {
    let req_case = Expand!(case).to_string().to_lowercase();
    Ok(Tokens::new(lx_change_case_tokens(&req_case, &tokens)?))
  });

  TeX!(
    r"\AddToNoCaseChangeList{\NoCaseChange}%
\AddToNoCaseChangeList{\label}%
\AddToNoCaseChangeList{\ref}%
\AddToNoCaseChangeList{\cite}%
\AddToNoCaseChangeList{\ensuremath}%
\AddToNoCaseChangeList{\@ensuremath}%
\AddToNoCaseChangeList{\thanks}%"
  );

  // Perl L5966-5993: \MakeUppercase, \MakeLowercase, \MakeTitlecase
  TeX!(
    r"\DeclareRobustCommand{\MakeUppercase}[1]{{%
  \lx@prepare@case@mapping%
  \def\({$}\let\)\(%
  \def\i{I}\def\j{J}%
  \let\UTF@two@octets@noexpand\@empty
  \let\UTF@three@octets@noexpand\@empty
  \let\UTF@four@octets@noexpand\@empty
  \edef\reserved@a{\lx@latex@changecase{upper}{#1}}%
  \reserved@a
}}
\DeclareRobustCommand{\MakeLowercase}[1]{{%
  \lx@prepare@case@mapping%
  \def\({$}\let\)\(%
  \let\UTF@two@octets@noexpand\@empty
  \let\UTF@three@octets@noexpand\@empty
  \let\UTF@four@octets@noexpand\@empty
  \edef\reserved@a{\lx@latex@changecase{lower}{#1}}%
  \reserved@a
}}
\DeclareRobustCommand{\MakeTitlecase}[1]{{%
  \lx@prepare@case@mapping%
  \def\({$}\let\)\(%
  \let\UTF@two@octets@noexpand\@empty
  \let\UTF@three@octets@noexpand\@empty
  \let\UTF@four@octets@noexpand\@empty
  \edef\reserved@a{\lx@latex@changecase{sentence}{#1}}%
  \reserved@a
}}
\protected@edef\MakeUppercase#1{\MakeUppercase{#1}}
\protected@edef\MakeLowercase#1{\MakeLowercase{#1}}
\protected@edef\MakeTitlecase#1{\MakeTitlecase{#1}}"
  );

  // Perl L5913,5916: fixltx2e defaults
  DefMacro!("\\eminnershape", None, None);
  DefMacro!("\\TextOrMath{}{}", "\\ifmmode#2\\else#1\\fi");

  //======================================================================
  // Semi-undocumented commands
  // Perl: latex_constructs.pool.ltxml various locations
  //======================================================================

  // Hacky version matches multiple chars! but does NOT expand
  DefMacro!("\\@ifnext@n {}{}{}", sub[(tokens,if_toks,else_toks)] {
    let mut toks = VecDeque::from(tokens.unlist());
    let mut read = Vec::new();

    while let Some(t) = gullet::read_token()? {
      // Stop as soon as we've matched the full token sequence —
      // otherwise the `toks[0]` index panics on the next iteration
      // (arxiv 1608.08252 hit this with a matching prefix followed
      // by arbitrary tokens in the stream).
      if toks.is_empty() {
        read.push(t);
        break;
      }
      if t == toks[0] {
        toks.pop_front();
        read.push(t);
      } else {
        read.push(t);
        break;
      }
    }
    let mut result = if toks.is_empty() {
      if_toks.unlist()
    } else {
      else_toks.unlist()
    };
    result.extend(read);
    Ok(Tokens::new(result))
  });

  DefMacro!("\\@ifstar {}{}", sub[(if_toks,else_toks)] {
    let next_opt = gullet::read_non_space()?;
    if next_opt == Some(T_OTHER!("*")) {
      Ok(if_toks)
    } else {
      let mut result = else_toks.unlist();
      if let Some(next) = next_opt {
        result.push(next);
      }
      Ok(Tokens::new(result))
    }
  });

  DefMacro!("\\@dblarg {}", r"\kernel@ifnextchar[{#1}{\@xdblarg{#1}}");
  DefMacro!("\\@xdblarg {}{}", r"#1[{#2}]{#2}");

  DefMacro!("\\@testopt{}{}", sub[(cmd, option)] {
    if gullet::if_next(T_OTHER!("["))? {
      Ok(cmd)
    } else {
      Ok(Tokens!(cmd.unlist(), T_OTHER!("["), option.unlist(), T_OTHER!("]")))
    }
  });
  TeX!(
    r"
  \def\@protected@testopt#1{%%
    \ifx\protect\@typeset@protect
      \expandafter\@testopt
    \else
      \@x@protect#1%
    \fi}"
  );

  Let!("\\l@ngrel@x", "\\relax");
  DefMacro!(
    "\\@star@or@long{}",
    r"\@ifstar{\let\l@ngrel@x\relax#1}{\let\l@ngrel@x\long#1}"
  );

  TeX!(
    r"
  \def\in@#1#2{%
  \def\in@@##1#1##2##3\in@@{%
    \ifx\in@##2\in@false\else\in@true\fi}%
  \in@@#2#1\in@\in@@}
  \newif\ifin@"
  );

  DefMacro!("\\IfFileExists{}{}{}", sub[(file, if_tks, else_tks)] {
    let file_string = Expand!(file).to_string();
    if find_file(&file_string, None).is_some() {
      let found_str = s!("\"{file_string}\" ");
      def_macro(T_CS!("\\@filef@und"), None, Some(found_str.into()), None)?;
      if_tks
    } else {
      else_tks
    }
  });

  // LaTeX3 format-version guard (ltcmd.dtx 2020/10/01 kernel). Source that
  // checks for format features writes
  //   \IfFormatAtLeastTF{YYYY/MM/DD}{then}{else}
  // expecting the `then` branch on modern LaTeX. LaTeXML simulates a
  // current-enough format, so always take `#2`. This is how babel's
  // greek.ldf probes for LaTeX3 catcode primitives; without the stub it
  // emits Error:undefined and bails out of the language setup.
  DefMacro!("\\IfFormatAtLeastTF{}{}{}", "#2");

  DefMacro!("\\InputIfFileExists{}{}{}", sub[(file, if_tks, else_tks)] {
    let file_tks = Expand!(file);
    let file_string = file_tks.to_string();
    if find_file(&file_string, None).is_some() {
      let found_str = s!("\"{file_string}\" ");
      def_macro(T_CS!("\\@filef@und"), None, Some(found_str.into()), None)?;
      Tokens!(if_tks, T_CS!("\\@addtofilelist"), T_BEGIN!(), file_tks.clone(), T_END!(),
        T_CS!("\\ltx@input"), T_BEGIN!(), file_tks, T_END!())
    } else {
      else_tks
    }
  });

  DefMacro!("\\@ifdefinable DefToken {}", sub[(token, iftoken)] {
    if is_definable(&token) {
      iftoken.unlist()
    } else {
      let token_str = token.to_string();
      let mut s = ExplodeText!(token_str);
      if token_str.starts_with('\\') {
        s.remove(0);
      }
      DefMacro!(T_CS!("\\reserved@a"), None, Tokens::new(s));
      vec![T_CS!("\\@notdefinable")]
    }
  });

  Let!("\\@@ifdefinable", "\\@ifdefinable");

  DefMacro!("\\@rc@ifdefinable DefToken {}", sub[(_token, iftoken)] {
    Let!("\\@ifdefinable", "\\@@ifdefinable");
    iftoken.unlist()
  });

  DefMacro!(
    "\\@notdefinable",
    None,
    r###"\@latex@error{%
    Command \@backslashchar\reserved@a\space
    already defined.
    Or name \@backslashchar\@qend... illegal, see p.192 of the manual}
  "###
  );

  // Sundry
  // Perl latex_constructs.pool L5771: DefPrimitiveI('\textprime', undef, UTF(0xB4))
  DefPrimitive!("\\textprime", "\u{00B4}"); // ACUTE ACCENT
  Let!("\\endgraf", "\\par");
  Let!("\\endline", "\\cr");
  DefMacro!("\\fileversion", "");
  DefMacro!("\\filedate", "");
  DefMacro!("\\chaptername", "Chapter");
  DefMacro!("\\partname", "Part");
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\sectiontyperefname", "\\lx@sectionsign\\lx@ignorehardspaces");
  DefMacro!("\\subsectiontyperefname", "\\lx@sectionsign\\lx@ignorehardspaces");
  DefMacro!("\\subsubsectiontyperefname", "\\lx@sectionsign\\lx@ignorehardspaces");
  DefMacro!("\\paragraphtyperefname", "\\lx@paragraphsign\\lx@ignorehardspaces");
  DefMacro!("\\subparagraphtyperefname", "\\lx@paragraphsign\\lx@ignorehardspaces");

  //======================================================================
  // Perl latex_constructs.pool.ltxml L5796-5800: aux file stubs
  //======================================================================
  DefMacro!("\\bibdata{}", None);
  DefMacro!("\\bibcite{}{}", None);
  DefMacro!("\\citation{}", None);
  DefMacro!("\\contentsline{}{}{}", None);
  DefMacro!("\\newlabel{}{}", None);

  // Perl L5804-5806
  Let!("\\mathgroup", "\\fam");

  // Perl L5808-5821: nocorr, text@command, check@ic stubs
  DefMacro!("\\nocorrlist", None, ".,");
  Let!("\\nocorr", "\\relax");
  Let!("\\check@icl", "\\@empty");
  Let!("\\check@icr", "\\@empty");
  Let!("\\curr@math@size", "\\@empty");
  DefMacro!("\\text@command{}", "");
  DefMacro!("\\check@nocorr@ Until:\\nocorr Until:\\@nil", "");
  TeX!("\\newif\\ifmaybe@ic");
  DefMacro!("\\maybe@ic", None, None);
  DefMacro!("\\maybe@ic@", None, None);
  DefMacro!("\\sw@slant", None, None);
  DefMacro!("\\fix@penalty", None, None);

  // Perl L5814: \mathhexbox
  DefPrimitive!("\\mathhexbox {}{}{}", sub[(a, b, c)] {
    let n = a.to_string().trim().parse::<i32>().unwrap_or(0) * 256
      + b.to_string().trim().parse::<i32>().unwrap_or(0) * 16
      + c.to_string().trim().parse::<i32>().unwrap_or(0);
    let (glyph, _font) = font_decode(n, None, None);
    if let Some(ch) = glyph {
      vec![Tbox::new(arena::pin_char(ch), None, None, Tokens!(), SymHashMap::default()).into()]
    } else {
      Vec::new()
    }
  });

  // Perl L5825: \extrafloats — modern LaTeX (2015+) for extra float slots (no-op)
  DefPrimitive!("\\extrafloats{}", None);

  //======================================================================
  // Perl latex_constructs.pool.ltxml L5836-5886: language declarations
  // Pre-declare hyphenation languages for babel's \iflanguage checks
  //======================================================================
  TeX!(r"\newlanguage\l@english
\newlanguage\l@usenglishmax
\newlanguage\l@USenglish
\newlanguage\l@dumylang
\newlanguage\l@nohyphenation
\newlanguage\l@arabic
\newlanguage\l@basque
\newlanguage\l@bulgarian
\newlanguage\l@coptic
\newlanguage\l@welsh
\newlanguage\l@czech
\newlanguage\l@slovak
\newlanguage\l@german
\newlanguage\l@ngerman
\newlanguage\l@danish
\newlanguage\l@esperanto
\newlanguage\l@spanish
\newlanguage\l@catalan
\newlanguage\l@galician
\newlanguage\l@estonian
\newlanguage\l@farsi
\newlanguage\l@finnish
\newlanguage\l@french
\newlanguage\l@greek
\newlanguage\l@monogreek
\newlanguage\l@ancientgreek
\newlanguage\l@croatian
\newlanguage\l@hungarian
\newlanguage\l@interlingua
\newlanguage\l@ibycus
\newlanguage\l@indonesian
\newlanguage\l@icelandic
\newlanguage\l@italian
\newlanguage\l@latin
\newlanguage\l@mongolian
\newlanguage\l@dutch
\newlanguage\l@norsk
\newlanguage\l@polish
\newlanguage\l@portuguese
\newlanguage\l@pinyin
\newlanguage\l@romanian
\newlanguage\l@russian
\newlanguage\l@slovenian
\newlanguage\l@uppersorbian
\newlanguage\l@serbian
\newlanguage\l@swedish
\newlanguage\l@turkish
\newlanguage\l@ukenglish
\newlanguage\l@ukrainiane");

  // Perl latex_constructs: \protected@write
  DefPrimitive!("\\protected@write{Number}{}{}", sub[(_port, prelude, _tokens)] {
    bgroup();
    Let!("\\thepage", "\\relax");
    let _digested = digest(prelude)?;
    egroup()?;
  });

  // \@@end — saved TeX \end primitive
  DefPrimitive!("\\@@end", {
    if !state::lookup_bool_sym(pin!("INTERPRETING_DEFINITIONS")) {
      gullet::flush();
    }
  });

  //======================================================================
  // Closure-backed primitives — Perl: latex_constructs.pool.ltxml L5645-5766.
  // These MUST live in `_constructs` (always loaded), not `_base` (optional
  // under Perl's LoadFormat mutual-exclusivity). Their closures cannot be
  // serialized into the kernel dump; defining them here guarantees they
  // exist whether or not the dump short-circuits `_base`.
  //
  // Relocated from `latex_base.rs` 2026-04-18 for Perl-parity and to
  // unblock `LATEXML_DUMP_ONLY=1` paths (see SYNC_STATUS D0 v3.f).

  // Perl L5645
  DefPrimitive!("\\@onlypreamble{}", {
    only_preamble("\\@onlypreamble")?;
  });

  // Perl L5646-5648
  DefPrimitive!("\\GenericError{}{}{}{}", sub[(_arg1,arg2,arg3,arg4)] {
    make_generic_message("\\GenericError", vec![arg2, arg3, arg4], "error")?;
  });
  DefPrimitive!("\\GenericWarning{}{}", sub[(arg1,arg2)] {
    make_generic_message("\\GenericWarning", vec![arg1,arg2], "warn")?;
  });
  DefPrimitive!("\\GenericInfo{}{}", sub[(arg1,arg2)] {
    make_generic_message("\\GenericInfo", vec![arg1,arg2], "info")?;
  });

  // `\newif`-generated boolean toggles from latex.ltx post-line-11957.
  // Our latex.ltx dump-build OOMs at L11958 (expl3 `\cs_new_protected:Npn
  // \property_new:nnnn ...` block) so the dump is currently truncated to
  // the first 11957 lines. The 22+ `\newif` calls between L11958 and end
  // (line 18513) are missing — common LaTeX2e booleans that papers use.
  // Add them as a single `\newif` raw-TeX block here so:
  //  - `\if@twocolumn`/`\@twocolumntrue`/`\@twocolumnfalse`
  //  - `\if@twoside`, `\if@compatibility`, `\if@firstcolumn`,
  //  - `\if@mparswitch`, `\if@reversemargin`, `\if@specialpage`,
  //  - `\if@insert`, `\if@fcolmade`, `\if@noskipsec`, `\if@afterindent`
  //  - and 10+ more
  // are defined post-dump. Mirrors latex.ltx's actual `\newif` calls.
  RawTeX!(r"\newif\ifv@
\newif\ifh@
\newif\ifdt@p
\newif\if@eqnsw\@eqnswtrue
\newif\if@inlabel \@inlabelfalse
\newif\if@newlist   \@newlistfalse
\newif\if@noparitem \@noparitemfalse
\newif\if@noparlist \@noparlistfalse
\newif\if@noitemarg \@noitemargfalse
\newif\if@nmbrlist  \@nmbrlistfalse
\newif\if@endpe
\newif\if@in@minipage@env
\newif\if@pboxsw
\newif\if@rjfield
\newif\if@firstamp
\newif\if@negarg
\newif\if@ovt
\newif\if@ovb
\newif\if@ovl
\newif\if@ovr
\newif\if@ovvline \@ovvlinetrue
\newif\if@ovhline \@ovhlinetrue
\newif\if@noskipsec \@noskipsectrue
\newif\if@afterindent \@afterindenttrue
\newif\if@compatibility
\newif\if@fcolmade
\newif\if@firstcolumn \@firstcolumntrue
\newif\if@insert
\newif\if@mparswitch  \@mparswitchfalse
\newif\if@reversemargin \@reversemarginfalse
\newif\if@specialpage \@specialpagefalse
\newif\if@twocolumn   \@twocolumnfalse
\newif\if@twoside     \@twosidefalse
");

  // `\ltx@hard@MessageBreak` is the literal newline target used by
  // `make_generic_message` to convert `\MessageBreak`-separated lines
  // in `\GenericInfo`/`\GenericWarning`/`\GenericError` messages.
  // Originally defined in `latex_base.rs:287`, but `latex_base` is
  // replaced by `latex_dump` in dump path — so the DefMacro doesn't
  // run there and `\ltx@hard@MessageBreak` is undefined. When
  // `make_generic_message` then calls `let_i(\MessageBreak,
  // \ltx@hard@MessageBreak)`, the let-target is undefined → meaning
  // becomes Stored::None → `\MessageBreak` becomes undefined for the
  // remainder of the digestion. The next babel info message
  // ("Importing font data...") then errors with "MessageBreak
  // undefined". Re-define here in latex_constructs (post-dump) so
  // both paths converge.
  DefMacro!("\\ltx@hard@MessageBreak", None, "^^J");

  // Perl L5650 — re-let `\MessageBreak` to `\relax` here, post-dump.
  // Defensive parity with Perl's exact placement.
  Let!("\\MessageBreak", "\\relax");

  // Perl L5652 — `DefMacro` in Perl (not DefPrimitive), empty-body no-op.
  DefMacro!("\\@setsize{}{}{}{}", "");

  // Perl L5765-5766
  DefPrimitive!("\\makeatletter", {
    AssignCatcode!('@', Catcode::LETTER, Some(Scope::Local));
  });
  DefPrimitive!("\\makeatother", {
    AssignCatcode!('@', Catcode::OTHER, Some(Scope::Local));
  });

  // Perl L5670-5673 — font size stubs. Token-list bodies (Perl:
  // `Tokens()` = empty) that swallow their args. Relocated from
  // latex_base.rs 2026-04-18 for Perl-parity AND so they round-trip
  // through the dump under LATEXML_DUMP_ONLY=1 (the dump reader's
  // @-internal safety filter rejects public-CS macros, so public
  // kernel CSes like `\fontsize` must live in always-loaded
  // `_constructs.rs`).
  DefMacro!("\\check@mathfonts", None);
  DefMacro!("\\fontsize{}{}", None);
  DefMacro!("\\@setfontsize{}{}{}", "\\let\\@currsize#1");

  // Perl L5687-5695 — \@ifnextchar + siblings (closure-backed).
  // Relocated from latex_base.rs 2026-04-18 to survive dump-only mode.
  DefMacro!("\\@ifnextchar DefToken {}{}", sub[(token, t_if, t_else)] {
    let next = gullet::read_non_space()?;
    let next_test = match next {
      Some(ref n) => XEquals!(&token, n),
      None => XEquals!(&token, &*TOKEN_END)
    };
    let which = if next_test { t_if } else { t_else };
    let mut result = which.substitute_parameters(&[]).unlist();
    if let Some(t_next) = next {
      result.push(t_next);
    }
    result
  });
  Let!("\\kernel@ifnextchar", "\\@ifnextchar");
  Let!("\\@ifnext", "\\@ifnextchar");
});

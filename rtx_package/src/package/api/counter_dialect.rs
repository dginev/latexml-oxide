use libxml::tree::Node;
use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;

use rtx_core::common::error::*;
use rtx_core::common::number::Number;
use rtx_core::common::xml::XML_NS;
use rtx_core::definition::expandable::ExpandableOptions;
use rtx_core::definition::register::NumericOps;
use rtx_core::definition::ExpansionBody;
use rtx_core::document::Document;
use rtx_core::gullet::Gullet;
use rtx_core::mouth;
use rtx_core::state::{Scope, State, Stored};
use rtx_core::stomach::Stomach;
use rtx_core::token::*;
use rtx_core::tokens::Tokens;
use rtx_core::whatsit::Whatsit;
use rtx_core::BoxOps;

use super::cleaners::{clean_id, roman_aux};
use super::content::{build_invocation, digest_if, digest_literal, digest_text};
use super::def_dialect::{def_macro, def_register};
use super::*;

//**********************************************************************
/// This function computes an xml:id for a node, if it hasn't already got one.
/// It is suitable for use in Tag afterOpen as
///  `Tag('ltx:para',afterOpen=>sub { GenerateID(@_,'p'); });`
/// It generates an id of the form <parentid>.<prefix><number>
/// The parent node (the one with ID=<parentid>) also maintains a counter
/// stored in an attribute `_ID_counter_<prefix>` recording the last used
/// <number> for <prefix> amongst its descendents.
pub fn generate_id(document: &mut Document, mut node: &mut Node, mut prefix: &str, state: &mut State) -> Result<()> {
  // If node doesn't already have an id, and can
  let node_qname = document.get_node_qname(node, state);
  // but isn't a _Capture_ node (which ultimately should disappear)
  if node.get_attribute_ns("id", XML_NS).is_none() && document.can_have_attribute(&node_qname, "xml:id", state) && (node_qname != "ltx:_Capture_") {
    let mut ancestor = document
      .findnode("ancestor::*[@xml:id][1]", Some(node), state)
      .unwrap_or_else(|| document.get_document().get_root_element().unwrap());
    //// Old versions don't like ancestor.getAttribute('xml:id');
    let ancestor_id = ancestor.get_attribute_ns("id", XML_NS);
    // If we've got no ancestor_id, then we've got no ancestor (no document yet!),
    // or ancestor IS the root element (but without an id);
    // If we also have no prefix, we'll end up with an illegal id (just digits)!!!
    // We'll use "id" for an id prefix; this will work whether or not we have an ancestor.
    if prefix.is_empty() && ancestor_id.is_none() {
      prefix = "id";
    }

    let ctrkey = s!("_ID_counter_") + prefix + "_";
    let a_ctr = ancestor.get_attribute(&ctrkey).unwrap_or_else(|| s!("0"));

    let ctr_int = 1 + a_ctr.parse::<u32>().unwrap_or(0);
    let ctr = ctr_int.to_string();

    let id = match ancestor_id {
      Some(aid) => aid + ".",
      None => String::new(),
    } + prefix
      + &ctr;

    ancestor.set_attribute(&ctrkey, &ctr)?;
    node.set_attribute("xml:id", &id)?;
  }
  Ok(())
}

pub struct NewCounterOptions<'ct> {
  pub idprefix: &'ct str,
  pub idwithin: &'ct str,
  pub nested: Vec<&'ct str>,
}
impl<'ct> Default for NewCounterOptions<'ct> {
  fn default() -> Self {
    NewCounterOptions {
      idprefix: "",
      idwithin: "",
      nested: Vec::new(),
    }
  }
}

pub fn new_counter(ctr: &str, within: &str, options_opt: Option<NewCounterOptions>, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  let unctr = s!("UN{}", ctr); // UNctr is counter for generating ID's for UN-numbered items.
  let cctr = s!("\\c@{}", ctr);
  let clctr = s!("\\cl@{}", ctr);
  let cunctr = s!("\\c@{}", unctr);
  let clunctr = s!("\\cl@{}", unctr);

  def_register(T_CS!(cctr), None, Number::new(0.0), None, state);
  state.assign_value(&cctr, Number::new(0.0), Some(Scope::Global));
  state.after_assignment();
  if !state.lookup_bool(&clctr) {
    state.assign_value(&clctr, Tokens!(), Some(Scope::Global));
  }

  def_register(T_CS!(cunctr), None, Number::new(0.0), None, state);
  state.assign_value(&cunctr, Number::new(0.0), Some(Scope::Global));
  if !state.lookup_bool(&clunctr) {
    state.assign_value(&clunctr, Tokens!(), Some(Scope::Global));
  }

  if !within.is_empty() {
    let clwithin = s!("\\cl@{}", within);
    let clunwithin = s!("\\cl@UN{}", within);
    let mut x = if let Some(cl) = state.lookup_tokens(&clwithin) {
      cl.unlist()
    } else {
      Vec::new()
    };
    let mut clwithin_tokens = vec![T_CS!(ctr), T_CS!(unctr)];
    clwithin_tokens.append(&mut x);
    state.assign_value(&clwithin, Stored::Tokens(Tokens::new(clwithin_tokens)), Some(Scope::Global));

    let mut unx = if let Some(clun) = state.lookup_tokens(&clunwithin) {
      clun.unlist()
    } else {
      Vec::new()
    };
    let mut clunwithin_tokens = vec![T_CS!(unctr)];
    clunwithin_tokens.append(&mut unx);

    state.assign_value(&clunwithin, Stored::Tokens(Tokens::new(clunwithin_tokens)), Some(Scope::Global))
  }

  if let Some(ref options) = options_opt {
    if !options.nested.is_empty() {
      state.assign_value(&s!("nested_counters_{}", ctr), options.nested.clone(), Some(Scope::Global))
    }
  }

  // default is equivalent to \arabic{ctr}, but w/o using the LaTeX macro!
  let ctr_string = ctr.to_string();
  def_macro(
    T_CS!(s!("\\the{}", ctr)),
    None,
    Some(ExpansionBody::Closure(Rc::new(move |gullet, args, inner_state| {
      let counter_value = CounterValue!(&ctr_string, inner_state).value_of();
      Ok(Tokens::new(ExplodeText!(counter_value)))
    }))),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
    state,
  );

  let mut prefix = match options_opt {
    None => String::new(),
    Some(ref opt) => opt.idprefix.to_string(),
  };
  if !prefix.is_empty() {
    state.assign_value(&s!("@ID@prefix@{}", ctr), prefix.to_string(), Some(Scope::Global));
  } else {
    prefix = state.lookup_string(&s!("@ID@prefix@{}", ctr));
    if prefix.is_empty() {
      prefix = ctr.to_string();
    }
  }
  prefix = clean_id(&prefix);
  let opts_idwithin = match options_opt {
    None => "",
    Some(ref opt) => opt.idwithin,
  };
  let opts_idwithin = match options_opt {
    None => "",
    Some(ref opt) => opt.idwithin,
  };

  if !prefix.is_empty() {
    let mut idwithin = if !opts_idwithin.is_empty() { opts_idwithin } else { within }.to_string();
    if !idwithin.is_empty() {
      let ctr_string = ctr.to_string();
      let thectrid = s!("\\the{}@ID", ctr);
      def_macro(
        T_CS!(thectrid),
        None,
        Some(ExpansionBody::Closure(Rc::new(move |gullet, args, inner_state| {
          Ok(TokenizeInternal!(&s!(
            "\\expandafter\\ifx\\csname the{}@ID\\endcsname\\@empty\\else\
             \\csname the{}@ID\\endcsname.\\fi {}\\csname @{}@ID\\endcsname",
            idwithin,
            idwithin,
            prefix,
            ctr_string
          )))
        }))),
        Some(ExpandableOptions {
          scope: Some(Scope::Global),
          ..ExpandableOptions::default()
        }),
        state,
      )
    } else {
      let ctr_string = ctr.to_string();
      let thectrid = s!("\\the{}@ID", ctr);
      def_macro(
        T_CS!(thectrid),
        None,
        Some(ExpansionBody::Closure(Rc::new(move |gullet, args, inner_state| {
          Ok(TokenizeInternal!(&s!("{}\\csname @{}@ID\\endcsname", prefix, ctr_string)))
        }))),
        Some(ExpandableOptions {
          scope: Some(Scope::Global),
          ..ExpandableOptions::default()
        }),
        state,
      );
    }
    def_macro(
      T_CS!(s!("\\@{}@ID", ctr)),
      None,
      Some(ExpansionBody::Tokens(Tokens!(T_OTHER!("0")))),
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        ..ExpandableOptions::default()
      }),
      state,
    );
  }

  Ok(())
}

pub fn counter_value(ctr: &str, state: &mut State) -> Number {
  match state.lookup_number(&s!("\\c@{}", ctr)) {
    None => {
      let message = s!("Counter {} was not defined; assuming 0", ctr);
      Warn!("undefined", ctr, None, state, message);
      Number::new(0.0)
    },
    Some(value) => value,
  }
}

pub fn add_to_counter(ctr: &str, value: Number, gullet: &mut Gullet, state: &mut State) {
  let v = counter_value(ctr, state).add(value);
  state.assign_value(&s!("\\c@{}", ctr), v, Some(Scope::Global));
  state.after_assignment();
  let id_cs = T_CS!(s!("\\@{}@ID", ctr));
  def_macro(
    id_cs.clone(),
    None,
    Tokens::new(Explode!(v.value_of())),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
    state,
  );
}

pub fn step_counter(ctr: &str, noreset: bool, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  let value = counter_value(ctr, state);
  state.assign_value(&s!("\\c@{}", ctr), value.add(Number::new(1.0)), Some(Scope::Global));
  state.after_assignment();
  let token_value = Tokens::new(Explode!(counter_value(ctr, state).value_of()));
  def_macro(
    T_CS!(s!("\\@{}@ID", ctr)),
    None,
    token_value.clone(),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
    state,
  );

  // and reset any within counters!
  if !noreset {
    if let Some(nested) = state.lookup_tokens(&s!("\\cl@{}", ctr)) {
      for c in nested.unlist() {
        reset_counter(&c.to_string(), state);
      }
    }
  }
  digest_if(T_CS!(s!("\\the{}", ctr)), stomach, state)?;
  Ok(())
}

pub struct RefStepValue {
  pub id: Option<String>,
  pub tags: Option<Tokens>,
}

pub fn ref_step_counter(ctype: &str, noreset: bool, stomach: &mut Stomach, state: &mut State) -> Result<HashMap<String, Stored>> {
  let ctr = match state.lookup_mapping("counter_for_type", ctype) {
    Some(Stored::String(ctr)) => ctr.to_string(),
    _ => ctype.to_string(),
  };
  step_counter(&ctr, noreset, stomach, state)?;

  let has_id: bool = if let Some(iddef) = state.lookup_definition(&T_CS!(s!("\\the{}@ID", ctr))) {
    if let Some(params) = iddef.get_parameters() {
      params.get_num_args() == 0
    } else {
      true
    }
  } else {
    false
  };

  let the_cs = T_CS!(s!("\\the{}", ctr));
  let the_id_cs = T_CS!(s!("\\the{}@ID", ctr));
  def_macro(
    T_CS!("\\@currentlabel"),
    None,
    the_cs.clone(),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
    state,
  );
  if has_id {
    def_macro(
      T_CS!("\\@currentID"),
      None,
      the_id_cs.clone(),
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        ..ExpandableOptions::default()
      }),
      state,
    );
  }

  let id = if has_id {
    digest_literal(Tokens!(T_CS!(s!("\\the{}@ID", ctr))), stomach, state)?.to_string()
  } else {
    String::new()
  };

  let refnum = digest_text(Tokens!(T_CS!(s!("\\the{}", ctr))), stomach, state)?;
  let invocation;
  {
    let gullet = stomach.get_gullet_mut();
    invocation = build_invocation(T_CS!("\\lx@make@tags"), vec![Tokens!(T_OTHER!(ctype))], gullet, state)?;
  }

  let tags = stomach.digest(invocation, state)?;

  // Any scopes activated for previous value of this counter (& any nested counters) must be
  // removed. This may also include scopes activated for \label
  deactivate_counter_scope(&ctr, state);

  // And install the scope (if any) for this reference number.
  state.assign_value("current_counter", ctr.to_string(), Some(Scope::Local));

  let scope = s!("{}:{}", ctr, refnum.to_string());
  state.assign_value(&s!("scopes_for_counter:{}", ctr), vec![scope.clone()], Some(Scope::Local));
  state.activate_scope(&scope);

  Ok(map!(
    "tags" => Stored::Digested(Box::new(tags)),
    "id" => Stored::String(id)
  ))
}

fn deactivate_counter_scope(ctr: &str, state: &mut State) {
  //  print STDERR "Unusing scopes for $ctr\n";
  if let Some(Stored::VecString(stored_scopes)) = state.lookup_value(&s!("scopes_for_counter:{}", ctr)) {
    for scope in stored_scopes.clone() {
      state.deactivate_scope(&scope);
    }
  }

  if let Some(Stored::VecString(stored_counters)) = state.lookup_value(&s!("nested_counters_{}", ctr)) {
    for inner_ctr in stored_counters.clone() {
      deactivate_counter_scope(&inner_ctr, state);
    }
  }
}

// For UN-numbered units
pub fn ref_step_id(ctype: &str, stomach: &mut Stomach, state: &mut State) -> Result<HashMap<String, Stored>> {
  let ctr = match state.lookup_mapping("counter_for_type", ctype) {
    Some(map) => map.to_string(),
    None => ctype.to_string(),
  };
  let unctr = s!("UN{}", ctr);
  step_counter(&unctr, false, stomach, state)?;

  let cunctr_val = state.lookup_number(&s!("\\c@{}", unctr)).unwrap().value_of();
  def_macro(
    T_CS!(&s!("\\@{}@ID", ctr)),
    None,
    Tokens!(T_OTHER!("x"), Explode!(cunctr_val)),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
    state,
  );

  let thectr = s!("\\the{}@ID", ctr);
  def_macro(T_CS!("\\@currentID"), None, T_CS!(&thectr), None, state);
  Ok(map!("id".to_string() => digest_literal(T_CS!(&thectr), stomach, state)?.to_string().into()))
}

pub fn reset_counter(ctr: &str, state: &mut State) {
  state.assign_value(&s!("\\c@{}", ctr), Number::new(0.0), Some(Scope::Global));
  // and reset any within counters!
  let nested = state.lookup_tokens(&s!("\\cl@{}", ctr)).unwrap_or_else(|| Tokens!());

  for c in &(nested.unlist()) {
    reset_counter(&c.to_string(), state);
  }
}

#[allow(clippy::float_cmp)]
/// Create id, and tags for an itemize type \item
pub fn ref_step_item_counter(tag: &str, stomach: &mut Stomach, state: &mut State) -> Result<HashMap<String, Stored>> {
  let counter = state.lookup_string("itemcounter");
  let n = state.lookup_int("itemization_items");
  state.assign_value("itemization_items", n + 1, None);
  let mut attr: HashMap<String, Stored> = HashMap::new();
  if n > 0 {
    if let Some(sep) = state.lookup_dimension("\\itemsep") {
      if let Some(default) = state.lookup_dimension("\\lx@default@itemsep") {
        if sep.value_of() != default.value_of() {
          attr.insert("itemsep".to_string(), sep.into());
        }
      }
    }
  }
  let mut stepped = if !tag.is_empty() {
    let mut props = ref_step_id(&counter, stomach, state)?;
    if tag.is_empty() {
      //empty tag?
      props
    } else {
      let formatter = if counter.starts_with("\\@desc") {
        T_CS!("\\descriptionlabel")
      } else {
        T_CS!("\\makelabel")
      };
      let gullet = stomach.get_gullet_mut();
      let mut tag_tokens = vec![
        T_BEGIN!(),
        T_CS!("\\let"),
        T_CS!(&s!("\\the{}", counter)),
        T_CS!("\\@empty"),
        T_CS!("\\def"),
        T_CS!(&s!("\\fnum@{}", counter)),
        T_BEGIN!(),
        formatter,
        T_BEGIN!(),
        T_OTHER!(tag),
        T_END!(),
        T_END!(),
        T_CS!("\\def"),
        T_CS!(&s!("\\typerefnum@{}", counter)),
        T_BEGIN!(),
        T_CS!("\\itemtyperefname"),
        T_SPACE!(),
        T_OTHER!(tag),
        T_END!(),
      ];
      tag_tokens.extend(build_invocation(T_CS!("\\lx@make@tags"), vec![Tokens!(T_OTHER!(counter))], gullet, state)?.unlist());
      tag_tokens.push(T_END!());

      let tags = stomach.digest(tag_tokens, state)?;
      if let Digested::List(l) = tags {
        if !l.is_empty() {
          props.insert("tags".to_string(), l.into());
        }
      } else {
        props.insert("tags".to_string(), tags.into());
      }
      props
    }
  } else {
    ref_step_counter(&counter, false, stomach, state)?
  };

  for (k, v) in attr.into_iter() {
    stepped.insert(k, v);
  }
  Ok(stepped)
}

/// Prepare for an list (itemize/enumerate/description/etc)
/// by determining the right counter (level)
/// and binding the right \item ( \$type@item, if $type is defined)
pub fn begin_itemize(itype: &str, counter: Option<&str>, nolevel: bool, stomach: &mut Stomach, state: &mut State) -> Result<HashMap<String, Stored>> {
  let counter = counter.unwrap_or("@item");
  let level = state.lookup_int(&s!("{}level", counter)) + 1;
  AssignRegister!(
    "\\itemsep",
    state.lookup_dimension("\\lx@default@itemsep").unwrap_or_default().into(),
    Vec::new(),
    state
  );
  state.assign_value(&s!("{}level", counter), level, None);
  state.assign_value("itemization_items", 0, None);
  let postfix = roman!(level).to_string();
  let mut usecounter = counter.to_string();
  if !nolevel {
    usecounter.push_str(&postfix);
  }
  if !itype.is_empty() {
    let itype_cs = T_CS!(s!("\\{}@item", itype));
    state.let_i(&T_CS!("\\item"), itype_cs, None);
  }
  state.let_i(&T_CS!("\\par"), T_CS!("\\normal@par"), None); // In case within odd environment.
  def_macro(T_CS!("\\@listctr"), None, Tokens!(Explode!(usecounter)), None, state);
  state.assign_value("itemcounter", usecounter.clone(), None);
  reset_counter(&usecounter, state);
  ref_step_counter(&s!("@itemize{}", postfix), false, stomach, state)
}

pub fn rescue_caption_counters(captype: &str, whatsit: &mut Whatsit, stomach: &mut Stomach, state: &mut State) {
  let tagskey = &s!("{}_tags", captype);
  if let Some(tags) = state.remove_value(&tagskey) {
    state.assign_value(&tagskey, false, Some(Scope::Global));
    whatsit.set_property("tags", tags);
  }
  let idkey = s!("{}_id", captype);
  if let Some(id) = state.remove_value(&idkey) {
    state.assign_value(&idkey, false, Some(Scope::Global));
    whatsit.set_property("id", id);
  }
  let inlistkey = s!("{}_inlist", captype);
  if let Some(inlist) = state.remove_value(&inlistkey) {
    state.assign_value(&inlistkey, false, Some(Scope::Global));
    whatsit.set_property("inlist", inlist);
  }
}

use std::borrow::Cow;
use std::collections::{VecDeque};
use rustc_hash::{FxHashMap as HashMap};
use std::sync::Arc;

use crate::binding::content::{build_invocation, digest_if, digest_literal, digest_text};
use crate::binding::def::dialect::{def_macro, def_register, is_defined};
use crate::common::cleaners::{clean_id, clean_label, roman_aux};
use crate::common::error::*;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::definition::expandable::ExpandableOptions;
use crate::definition::ExpansionBody;
use crate::gullet::Gullet;
use crate::mouth;
use crate::state::{Scope, State, Stored};
use crate::stomach::Stomach;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::BoxOps;

///!
///! # Counters
///!
///! This is modelled on LaTeX's counter mechanisms, but since it also
///! provides support for ID's, even where there is no visible reference number,
///! it is defined in general.
///! These id's should be both unique, and parallel the visible reference numbers
///! (as much as possible).  Also, for consistency, we add id's to unnumbered
///! document elements (eg from \section*); this requires an additional counter
///! (eg. UNsection) and  mechanisms to track it.


/// configuration for new_counter
#[derive(Default)]
pub struct NewCounterOptions<'ct>
 {
  /// specifies a prefix to be used in formatting ID's for document structure elements
  /// counted by this counter.  Ie. subsection 3 in section 2 might get: id="S2.SS3"
  pub idprefix: &'ct str,
  /// specifies that the ID is composed from $idwithin's ID,, even though
  /// the counter isn't numbered within it.  (mainly to avoid duplicated ids)
  pub idwithin: &'ct str,
  /// a list of counters that correspond to scopes which are "inside" this one.
  /// Whenever any definitions scoped to this counter are deactivated,
  /// the inner counter's scopes are also deactivated.
  // NOTE: I'm not sure this is even a sensible implementation,
  // or why inner should be different than the counters reset by incrementing this counter.
  pub nested: Vec<&'ct str>,
}

/// Defines a new counter named $ctr.
/// If `within` is defined, `ctr` will be reset whenever `within` is incremented.
pub fn new_counter(ctr: &str, within: &str, options_opt: Option<NewCounterOptions>, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  let unctr = s!("UN{}", ctr); // UNctr is counter for generating ID's for UN-numbered items.
  let cctr = s!("\\c@{}", ctr);
  let clctr = s!("\\cl@{}", ctr);
  let cunctr = s!("\\c@{}", unctr);
  let clunctr = s!("\\cl@{}", unctr);

  def_register(T_CS!(cctr), None, Number::new(0), None, state);
  state.assign_value(&cctr, Number::new(0), Some(Scope::Global));
  state.after_assignment(stomach.get_gullet_mut());
  if state.lookup_value(&clctr).is_none() {
    state.assign_value(&clctr, Tokens!(), Some(Scope::Global));
  }

  def_register(T_CS!(cunctr), None, Number::new(0), None, state);
  state.assign_value(&cunctr, Number::new(0), Some(Scope::Global));
  if state.lookup_value(&clunctr).is_none() {
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
    Some(ExpansionBody::Closure(Arc::new(move |_gullet, _args, inner_state| {
      let counter_value = counter_value(&ctr_string, inner_state).value_of();
      Ok(Tokens::new(ExplodeText!(counter_value)))
    }))),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
    state,
  );
  let p_ctr_cs = T_CS!(&s!("\\p@{}", ctr));
  if state.lookup_definition(&p_ctr_cs).is_none() {
    def_macro(
      p_ctr_cs,
      None,
      Tokens::default(),
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        ..ExpandableOptions::default()
      }),
      state,
    );
  }

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

  if !prefix.is_empty() {
    let idwithin = match options_opt {
      Some(ref opts) => {
        if opts.idwithin.is_empty() {
          within
        } else {
          opts.idwithin
        }
      },
      None => within,
    }
    .to_string();

    let ctr_string = ctr.to_string();
    let thectrid = s!("\\the{}@ID", ctr);
    if !idwithin.is_empty() {
      def_macro(
        T_CS!(thectrid),
        None,
        Some(ExpansionBody::Closure(Arc::new(move |_gullet, _args, _inner_state| {
          Ok(mouth::tokenize_internal(&s!(
            "\\expandafter\\ifx\\csname the{}@ID\\endcsname\\@empty\\else\\csname the{}@ID\\endcsname.\\fi {}\\csname @{}@ID\\endcsname",
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
      def_macro(
        T_CS!(thectrid),
        None,
        Some(ExpansionBody::Closure(Arc::new(move |_gullet, _args, _inner_state| {
          Ok(mouth::tokenize_internal(&s!("{}\\csname @{}@ID\\endcsname", prefix, ctr_string)))
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
/// Fetches the value associated with the counter C<$ctr>.
pub fn counter_value(ctr: &str, state: &mut State) -> Number {
  match state.lookup_number(&s!("\\c@{}", ctr)) {
    None => {
      let message = s!("Counter {} was not defined; assuming 0", ctr);
      Warn!("undefined", ctr, None, state, message);
      Number::new(0)
    },
    Some(value) => value,
  }
}
/// increments a named counter by a `Number`
pub fn add_to_counter(ctr: &str, value: Number, gullet: &mut Gullet, state: &mut State) {
  let v = counter_value(ctr, state).add(value);
  state.assign_value(&s!("\\c@{}", ctr), v, Some(Scope::Global));
  state.after_assignment(gullet);
  let id_cs = T_CS!(s!("\\@{}@ID", ctr));
  def_macro(
    id_cs,
    None,
    Tokens::new(Explode!(v.value_of())),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
    state,
  );
}

/// Analog of `\stepcounter`, steps the counter and returns the expansion of
/// `\the$ctr`  Usually you should use `ref_step_counter(ctr)` instead.
pub fn step_counter(ctr: &str, noreset: bool, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  let value = counter_value(ctr, state);
  state.assign_value(&s!("\\c@{}", ctr), value.add(Number::new(1)), Some(Scope::Global));
  state.after_assignment(stomach.get_gullet_mut());
  let token_value = Tokens::new(Explode!(state.lookup_number(&s!("\\c@{}", ctr)).unwrap().value_of()));
  def_macro(
    T_CS!(s!("\\@{}@ID", ctr)),
    None,
    token_value,
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
        reset_counter(c.get_string(), state);
      }
    }
  }
  digest_if(T_CS!(s!("\\the{}", ctr)), stomach, state)?;
  Ok(())
}

/// DG: This is a dirty refactor that avoids the Stomach threading for DefMath --- is there a conceptually better organization?
///
pub fn step_counter_gullet(ctr: &str, noreset: bool, gullet: &mut Gullet, state: &mut State) -> Result<()> {
  let value = counter_value(ctr, state);
  state.assign_value(&s!("\\c@{}", ctr), value.add(Number::new(1)), Some(Scope::Global));
  state.after_assignment(gullet);
  let token_value = Tokens::new(Explode!(state.lookup_number(&s!("\\c@{}", ctr)).unwrap().value_of()));
  def_macro(
    T_CS!(s!("\\@{}@ID", ctr)),
    None,
    token_value,
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
        reset_counter(c.get_string(), state);
      }
    }
  }
  // digest_if(T_CS!(s!("\\the{}", ctr)), stomach, state)?;
  Ok(())
}

/// Analog of `\refstepcounter`, steps the counter and returns a hash
/// containing the keys `refnum` and `id`.  This makes it
/// suitable for use in a `properties` option to constructors.
/// The `id` is generated in parallel with the reference number
/// to assist debugging.
pub fn ref_step_counter(ctype: &str, noreset: bool, stomach: &mut Stomach, state: &mut State) -> Result<HashMap<String, Stored>> {
  let ctr = match state.lookup_mapping("counter_for_type", ctype) {
    Some(Stored::String(ctr)) => ctr.to_string(),
    _ => ctype.to_string(),
  };
  step_counter(&ctr, noreset, stomach, state)?;
  maybe_preempt_refnum(&ctr, false, stomach.get_gullet_mut(), state);

  let has_id: bool = if let Some(iddef) = state.lookup_definition(&T_CS!(s!("\\the{ctr}@ID"))) {
    if let Some(params) = iddef.get_parameters() {
      params.get_num_args() == 0
    } else {
      true
    }
  } else {
    false
  };

  let the_cs = T_CS!(s!("\\the{ctr}"));
  let the_id_cs = T_CS!(s!("\\the{ctr}@ID"));
  def_macro(
    T_CS!("\\@currentlabel"),
    None,
    the_cs,
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
      the_id_cs,
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        ..ExpandableOptions::default()
      }),
      state,
    );
  }

  let id = if has_id {
    digest_literal(Tokens!(T_CS!(s!("\\the{ctr}@ID"))), stomach, state)?.to_string()
  } else {
    String::new()
  };

  let refnum = digest_text(Tokens!(T_CS!(s!("\\the{ctr}"))), stomach, state)?;
  let invocation;
  {
    let gullet = stomach.get_gullet_mut();
    invocation = build_invocation(T_CS!("\\lx@make@tags"), vec![Some(Tokens!(T_OTHER!(ctype)))], gullet, state)?;
  }

  let tags = stomach.digest(invocation, state)?;

  // Any scopes activated for previous value of this counter (& any nested counters) must be
  // removed. This may also include scopes activated for \label
  deactivate_counter_scope(&ctr, state);

  // And install the scope (if any) for this reference number.
  state.assign_value("current_counter", ctr.to_string(), Some(Scope::Local));

  let scope = s!("{ctr}:{refnum}");
  let mut receiver = VecDeque::new();
  receiver.push_front(Stored::String(scope.clone()));
  state.assign_value(&s!("scopes_for_counter:{ctr}"), receiver, Some(Scope::Local));
  state.activate_scope(&scope);

  Ok(map!(
    "tags" => Stored::Digested(tags),
    "id" => Stored::String(id)
  ))
}

/// Internal: Use a label-derived reference number and/or ID
/// instead of the traditional counter based ones.
/// Since the \label{} determins the reference number and ID,
/// we MUST sniff out the label BEFORE we call RefStepCounter/RefStepID !!!!!
/// (see MaybePeekLabel below; and also MaybeNoteLabel for use within
/// captions & certain equation environments)
/// Assign a sub to LABEL_MAPPING_HOOK: &sub($label,$counter,$norefnum)
/// to return the desired refnum and id for a given object.
fn maybe_preempt_refnum(ctr: &str, norefnum: bool, gullet: &mut Gullet, state: &mut State) {
  if let Some(_mapper) = state.lookup_value("LABEL_MAPPING_HOOK") {
    let hj_refnum = T_CS!(s!("\\_PREEMPTED_REFNUM_{ctr}"));
    let hj_id = T_CS!(s!("\\_PREEMPTED_ID_{ctr}"));
    // First, restore the \the<ctr> and \the<ctr>@ID macros to defaults
    if !norefnum && state.lookup_meaning(&hj_refnum).is_some() {
      state.let_i(&T_CS!(s!("\\the{ctr}")), hj_refnum, Some(Scope::Global), gullet);
    }
    if state.lookup_meaning(&hj_id).is_some() {
      state.let_i(&T_CS!(s!("\\the{ctr}@ID")), hj_id, Some(Scope::Global), gullet);
    }
    let _label = state.lookup_value("PEEKED_LABEL");
    // TODO: Continue once we know the type of "mapper"
    unimplemented!();
    //   let (fixedrefnum, fixedid) = mapper(label, ctr, norefnum);
    //   if !norefnum && fixedrefnum {
    //     if !state.lookup_neaning(hj_refnum) {    // Save for later
    //       state.let_i(&hj_refnum, T_CS!(s!("\\the{}",ctr)), Some(Scope::Global));
    //     }
    //     def_macro(T_CS!(s!("\\the{}",ctr)), None, fixedrefnum, Some(ExpandableOptions { scope: Some(Scope::Global),
    // ..ExpandableOptions::default()}), state);   }
    //   if fixedid {
    //     if state.lookup_meaning(&hj_id).is_none() {        // Save for later
    //       state.let_i(&hj_id, T_CS!(s!("\\the{}@ID",ctr)), Some(Scope::Global));
    //     }
    //     def_macro(T_CS!(s!("\\the{}@ID",ctr)), None, fixedid, Some(ExpandableOptions { scope: Some(Scope::Global),
    // ..ExpandableOptions::default()}), state);   }
    //   state.remove_value("PEEKED_LABEL"); // CONSUME the label
    //   state.assign_value("PROCESSED_LABEL", label, Some(Scope::Global));    // Note that we've consumed the label
  }
}

/// Use to note a discovered label to support label-derived refererence numbers
/// Can by used by \label, among others. Note we only record the label
/// if it hasn't already been peeked, and consumed.
pub fn maybe_note_label(label: &str, state: &mut State) {
  if state.lookup_value("LABEL_MAPPING_HOOK").is_some() {
    let label = clean_label(label, Some(""));
    let processed = state.lookup_string("PROCESSED_LABEL");
    if processed.is_empty() || processed != label {
      // Only if not already processed
      state.remove_value("PROCESSED_LABEL");
      state.assign_value("PEEKED_LABEL", Stored::String(label.into_owned()), Some(Scope::Global));
    }
  }
}

fn deactivate_counter_scope(ctr: &str, state: &mut State) {
  //  print STDERR "Unusing scopes for $ctr\n";
  if let Some(Stored::VecDequeStored(stored_scopes)) = state.lookup_value(&s!("scopes_for_counter:{ctr}")) {
    for scope_stored in stored_scopes.clone() {
      if let Stored::String(scope) = scope_stored {
        state.deactivate_scope(&scope);
      } else {
        panic!("assignmenet scopes should be stored as strings, got: {scope_stored:?}");
      }
    }
  }

  // TODO: if we ever want to unshift from the nested_counters, we'll need to also use Stored::VecDequeStored for them.
  if let Some(Stored::VecString(stored_counters)) = state.lookup_value(&s!("nested_counters_{ctr}")) {
    for inner_ctr in stored_counters.clone() {
      deactivate_counter_scope(&inner_ctr, state);
    }
  }
}

/// For UN-numbered units.
/// Like `RefStepCounter`, but only steps the "uncounter",
/// and returns only the id;  This is useful for unnumbered cases
/// of objects that normally get both a refnum and id.
pub fn ref_step_id(ctype: &str, stomach: &mut Stomach, state: &mut State) -> Result<HashMap<String, Stored>> {
  let ctr = match state.lookup_mapping("counter_for_type", ctype) {
    Some(map) => map.to_string(),
    None => ctype.to_string(),
  };
  let unctr = s!("UN{ctr}");
  step_counter(&unctr, false, stomach, state)?;
  maybe_preempt_refnum(&ctr, true, stomach.get_gullet_mut(), state);
  let cunctr_val = state.lookup_number(&s!("\\c@{unctr}")).unwrap().value_of();
  def_macro(
    T_CS!(s!("\\@{ctr}@ID")),
    None,
    Tokens!(T_OTHER!("x"), Explode!(cunctr_val)),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
    state,
  );

  let thectr = s!("\\the{ctr}@ID");
  def_macro(T_CS!("\\@currentID"), None, T_CS!(thectr), None, state);
  Ok(stored_map!("id" =>
    clean_id(&digest_literal(T_CS!(thectr), stomach, state)?.to_string())))
}

/// Resets the counter `ctr` to zero.
pub fn reset_counter(ctr: &str, state: &mut State) {
  state.assign_value(&s!("\\c@{ctr}"), Number::new(0), Some(Scope::Global));
  state.assign_value(&s!("\\c@UN{ctr}"), Number::new(0), Some(Scope::Global));
  def_macro(
    T_CS!(s!("\\@{}@ID", ctr)),
    None,
    Tokens!(Explode!(state.lookup_number(&s!("\\c@{ctr}")).unwrap().value_of())),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
    state,
  );
  // and reset any within counters!
  let nested = state.lookup_tokens(&s!("\\cl@{ctr}")).unwrap_or_default();
  for c in nested.unlist() {
    reset_counter(c.get_string(), state);
  }
}

/// Create id, and tags for an itemize type \item
pub fn ref_step_item_counter(tag_opt: Option<&Tokens>, stomach: &mut Stomach, state: &mut State) -> Result<HashMap<String, Stored>> {
  let counter = state.lookup_string("itemcounter");
  let n = state.lookup_int("itemization_items");
  state.assign_value("itemization_items", n + 1, None);
  let mut attr: HashMap<String, Stored> = HashMap::default();
  if n > 0 {
    if let Some(sep) = state.lookup_dimension("\\itemsep") {
      let default_opt = state.lookup_dimension("\\lx@default@itemsep");
      if default_opt.is_none() || sep.value_of() != default_opt.unwrap().value_of() {
        attr.insert("itemsep".to_string(), sep.into());
      }
    }
  }

  let mut result = if let Some(tag) = tag_opt {
    let mut props = ref_step_id(&counter, stomach, state)?;
    if tag.is_empty() {
      return Ok(props);
    }
    let formatter = if counter.starts_with("@desc") {
      T_CS!("\\descriptionlabel")
    } else {
      T_CS!("\\makelabel")
    };
    let counter_name = s!("\\{counter}name");
    let typename = if is_defined(&counter_name, state) {
      T_CS!(counter_name)
    } else {
      T_CS!("\\itemtyperefname")
    };
    let gullet = stomach.get_gullet_mut();

    let mut tag_tokens = vec![
      T_BEGIN!(),
      T_CS!("\\let"),
      T_CS!(s!("\\the{counter}")),
      T_CS!("\\@empty"),
      T_CS!("\\def"),
      T_CS!(s!("\\fnum@{counter}")),
      T_BEGIN!(),
      formatter,
      T_BEGIN!(),
    ];
    // TODO: Another iffy clone...
    let reverted_tag = (*tag).clone().revert();
    tag_tokens.extend(reverted_tag.clone());
    tag_tokens.extend(vec![
      T_END!(),
      T_END!(),
      T_CS!("\\def"),
      T_CS!(s!("\\typerefnum@{counter}")),
      T_BEGIN!(),
      typename,
      T_SPACE!(),
    ]);
    tag_tokens.extend(reverted_tag);
    tag_tokens.push(T_END!());
    tag_tokens.extend(build_invocation(T_CS!("\\lx@make@tags"), vec![Some(Tokens!(T_OTHER!(counter)))], gullet, state)?.unlist());
    tag_tokens.push(T_END!());

    let tags = stomach.digest(tag_tokens, state)?;
    if !tags.is_empty() {
      props.insert("tags".to_string(), tags.into());
    }
    props
  } else {
    ref_step_counter(&counter, false, stomach, state)?
  };
  for (k, v) in attr.into_iter() {
    result.insert(k, v);
  }
  Ok(result)
}

/// configuration for begin_itemize
#[derive(Debug, Default, Clone)]
pub struct BeginItemizeOptions {
  /// disable nested id suffix based on stacking level
  pub nolevel: bool,
  /// enumitem series
  pub series: Option<Tokens>,
  /// start at a custom value
  pub start: Option<Number>,
  /// enumitem resume?
  pub resume: Option<String>,
  /// enumitem resume* ?
  pub resume_star: Option<String>,
}

/// Prepare for an list (itemize/enumerate/description/etc)
/// by determining the right counter (level)
/// and binding the right \item ( \$type@item, if $type is defined)
pub fn begin_itemize(
  itype: &str,
  counter: Option<&str>,
  options: BeginItemizeOptions,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<HashMap<String, Stored>> {
  let outercounter = state.lookup_string("itemcounter");
  let outerlevel = if !outercounter.is_empty() {
    state.lookup_int(&s!("{outercounter}level"))
  } else {
    0
  };
  let counter = counter.unwrap_or("@item");
  let listlevel = state.lookup_int("itemization_level") + 1; // level for this list overall
  let level = state.lookup_int(&s!("{counter}level")) + 1; // level for lists of specific type
  AssignRegister!(
    "\\itemsep",
    state.lookup_dimension("\\lx@default@itemsep").unwrap_or_default().into(),
    Vec::new(),
    state
  );
  state.assign_value("itemization_level", listlevel, None);
  state.assign_value(&s!("{counter}level"), level, None);
  state.assign_value("itemization_items", 0, None);
  let listpostfix = roman!(listlevel).to_string();
  let postfix = roman!(level).to_string();
  let mut usecounter = counter.to_string();
  if !options.nolevel && !postfix.is_empty() {
    usecounter.push_str(&postfix);
  }
  let gullet = stomach.get_gullet_mut();
  if !itype.is_empty() {
    let itype_cs = T_CS!(s!("\\{itype}@item"));
    state.let_i(&T_CS!("\\item"), itype_cs, None, gullet);
  }
  state.let_i(&T_CS!("\\par"), T_CS!("\\normal@par"), None, gullet); // In case within odd environment.
  def_macro(T_CS!("\\@listctr"), None, Tokens!(Explode!(usecounter)), None, state);
  // Now arrange that this list's id's are relative to the current (outer) item (if any)
  // And that the items within this list's id's are relative to this (new) list.
  state.assign_value("itemcounter", Stored::String(usecounter.clone()), None);
  let listcounter = s!("@itemize{listpostfix}");
  if state.lookup_value(&s!("\\c@{listcounter}")).is_none() {
    //Create new list counters as needed
    new_counter(&listcounter, "", None, stomach, state)?;
  }
  if !outercounter.is_empty() {
    // Make this list's ID relative to outer list's ID
    let outerusecounter = s!("{outercounter}{}", roman!(outerlevel).to_string());
    let thectr = s!("\\the{listcounter}@ID");
    let theexpansion = s!("\\the{outerusecounter}@ID.I\\arabic{{{listcounter}}}");
    def_macro(T_CS!(thectr), None, mouth::tokenize_internal(&theexpansion), None, state);

    // AND reset this list's counter when the outer item is stepped
    let mut cl_toks = vec![T_CS!(listcounter)];
    let cs_name = s!("\\cl@{outerusecounter}");
    if let Some(Stored::Tokens(tks)) = state.lookup_value(&cs_name) {
      cl_toks.extend(tks.clone().unlist());
    }
    state.assign_value(&cs_name, Stored::Tokens(Tokens::new(cl_toks)), Some(Scope::Global));
  }
  // format the id of \item's relative to the id of this list
  let useexp = mouth::tokenize_internal(&s!("\\the{listcounter}@ID.i\\@{usecounter}@ID"));
  def_macro(T_CS!(s!("\\the{usecounter}@ID")), None, useexp, None, state);

  let mut series = if let Some(s) = options.series { s.to_string() } else { String::new() };
  if let Some(start) = options.start {
    SetCounter!(usecounter, start, stomach, state);
    let gullet = stomach.get_gullet_mut();
    add_to_counter(&usecounter, Number(-1), gullet, state);
  } else if let Some(s) = match options.resume {
    Some(s) => Some(s),
    None => options.resume_star,
  } {
    if s != "noseries" {
      series = s;
      // TODO:
      // SetCounter!(usecounter,
      //   state.lookup_int(&s!("enumitem_series_{}_last",series)),
      //   state);
    }
  } else {
    reset_counter(&usecounter, state);
  }

  let mut rsc = ref_step_counter(&s!("@itemize{listpostfix}"), false, stomach, state)?;
  rsc.insert("counter".to_string(), Stored::String(usecounter));
  rsc.insert("series".to_string(), Stored::String(series));
  Ok(rsc)
}

/// Copies the current id, tags, and inlist counter values into whatsit properties
pub fn rescue_caption_counters(captype: &str, whatsit: &mut Whatsit, _stomach: &mut Stomach, state: &mut State) {
  let tagskey = &s!("{captype}_tags");
  if let Some(tags) = state.remove_value(tagskey) {
    state.assign_value(tagskey, false, Some(Scope::Global));
    whatsit.set_property("tags", tags);
  }
  let idkey = s!("{captype}_id");
  if let Some(id) = state.remove_value(&idkey) {
    state.assign_value(&idkey, false, Some(Scope::Global));
    whatsit.set_property("id", id);
  }
  let inlistkey = s!("{captype}_inlist");
  if let Some(inlist) = state.remove_value(&inlistkey) {
    state.assign_value(&inlistkey, false, Some(Scope::Global));
    whatsit.set_property("inlist", inlist);
  }
}

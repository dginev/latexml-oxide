//! # Counters
//!
//! This is modelled on LaTeX's counter mechanisms, but since it also
//! provides support for ID's, even where there is no visible reference number,
//! it is defined in general.
//! These id's should be both unique, and parallel the visible reference numbers
//! (as much as possible).  Also, for consistency, we add id's to unnumbered
//! document elements (eg from \section*); this requires an additional counter
//! (eg. UNsection) and  mechanisms to track it.

use std::collections::VecDeque;
use std::rc::Rc;

use crate::binding::content::{build_invocation, digest_literal, digest_text};
use crate::binding::def::dialect::{RegisterOptions, def_macro, def_register, is_defined};
use crate::common::arena::SymHashMap as HashMap;
use crate::common::arena::{self, SymStr};
use crate::common::cleaners::{clean_id, clean_label, roman_aux};
use crate::common::error::*;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;

use crate::BoxOps;
use crate::definition::expandable::ExpandableOptions;
use crate::definition::{Definition, ExpansionBody};
use crate::mouth;
use crate::state;
use crate::state::*;
use crate::stomach;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;

/// configuration for new_counter
#[derive(Default)]
pub struct NewCounterOptions<'ct> {
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
pub fn new_counter(ctr: &str, within: &str, options_opt: Option<NewCounterOptions>) -> Result<()> {
  let unctr = s!("UN{ctr}"); // UNctr is counter for generating ID's for UN-numbered items.
  if !within.is_empty()
    && within != "document"
    && lookup_definition(&T_CS!(s!("\\c@{within}")))?.is_none()
  {
    new_counter(within, "", None)?;
  }
  let cctr = s!("\\c@{ctr}");
  let clctr = s!("\\cl@{ctr}");
  let cunctr = s!("\\c@{unctr}");
  let clunctr = s!("\\cl@{unctr}");
  // Perl Package.pm L660-672: Check if counter already defined. Skip register if already a
  // Register. Warn if previously defined as something other than \relax.
  let cs_cctr = T_CS!(&cctr);
  let prev_defn = lookup_definition(&cs_cctr)?;
  if let Some(ref defn) = prev_defn {
    if defn.is_register() {
      // Counter already exists as a register — fine, just continue (may change within/nesting)
    } else {
      // Warn unless the previous definition was \relax
      let relax_meaning = state::lookup_meaning(&T_RELAX!());
      let prev_meaning = state::lookup_meaning(&cs_cctr);
      if prev_meaning != relax_meaning {
        Warn!(
          "unexpected",
          &cctr,
          s!("Counter {} was already defined; redefining", cctr)
        );
      }
      def_register(
        cs_cctr,
        None,
        Number::new(0),
        Some(RegisterOptions {
          allocate: Some(String::from("\\count")),
          ..RegisterOptions::default()
        }),
      )?;
    }
  } else {
    def_register(
      cs_cctr,
      None,
      Number::new(0),
      Some(RegisterOptions {
        allocate: Some(String::from("\\count")),
        ..RegisterOptions::default()
      }),
    )?;
  }
  after_assignment();
  if !has_value(&clctr) {
    state::assign_value(&clctr, Tokens!(), Some(Scope::Global));
  }
  def_register(T_CS!(&cunctr), None, Number::new(0), None)?;
  if !has_value(&clunctr) {
    state::assign_value(&clunctr, Tokens!(), Some(Scope::Global));
  }

  if !within.is_empty() {
    let clwithin = s!("\\cl@{within}");
    let clunwithin = s!("\\cl@UN{within}");
    let x = if let Some(cl) = state::lookup_tokens(&clwithin) {
      cl.unlist()
    } else {
      Vec::new()
    };
    let mut clwithin_tokens = vec![T_CS!(ctr), T_CS!(&unctr)];
    clwithin_tokens.extend(x);
    state::assign_value(
      &clwithin,
      Stored::Tokens(Tokens::new(clwithin_tokens)),
      Some(Scope::Global),
    );

    let mut unx = if let Some(clun) = state::lookup_tokens(&clunwithin) {
      clun.unlist()
    } else {
      Vec::new()
    };
    let mut clunwithin_tokens = vec![T_CS!(unctr)];
    clunwithin_tokens.append(&mut unx);

    state::assign_value(
      &clunwithin,
      Stored::Tokens(Tokens::new(clunwithin_tokens)),
      Some(Scope::Global),
    )
  }

  if let Some(ref options) = options_opt {
    if !options.nested.is_empty() {
      state::assign_value(
        &s!("nested_counters_{}", ctr),
        options.nested.clone(),
        Some(Scope::Global),
      )
    }
  }

  // default is equivalent to \arabic{ctr}, but w/o using the LaTeX macro!
  let ctr_string = ctr.to_string();
  def_macro(
    T_CS!(s!("\\the{}", ctr)),
    None,
    Some(ExpansionBody::Closure(Rc::new(move |_args| {
      let counter_value = counter_value(&ctr_string)?.value_of();
      Ok(Tokens::new(ExplodeText!(counter_value)))
    }))),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
  )?;
  let p_ctr_cs = T_CS!(&s!("\\p@{}", ctr));
  if lookup_definition(&p_ctr_cs)?.is_none() {
    def_macro(
      p_ctr_cs,
      None,
      Tokens::default(),
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        ..ExpandableOptions::default()
      }),
    )?;
  }

  let mut prefix = match options_opt {
    None => String::new(),
    Some(ref opt) => opt.idprefix.to_string(),
  };
  if !prefix.is_empty() {
    state::assign_value(
      &s!("@ID@prefix@{}", ctr),
      prefix.clone(),
      Some(Scope::Global),
    );
  } else {
    prefix = state::lookup_string(&s!("@ID@prefix@{}", ctr));
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
        Some(ExpansionBody::Closure(Rc::new(move |_args| {
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
      )?;
    } else {
      def_macro(
        T_CS!(thectrid),
        None,
        Some(ExpansionBody::Closure(Rc::new(move |_args| {
          Ok(mouth::tokenize_internal(&s!(
            "{prefix}\\csname @{ctr_string}@ID\\endcsname",
          )))
        }))),
        Some(ExpandableOptions {
          scope: Some(Scope::Global),
          ..ExpandableOptions::default()
        }),
      )?;
    }
    def_macro(
      T_CS!(s!("\\@{}@ID", ctr)),
      None,
      Some(ExpansionBody::Tokens(Tokens!(T_OTHER!("0")))),
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        ..ExpandableOptions::default()
      }),
    )?;
  }

  Ok(())
}
/// Fetches the value associated with the counter C<$ctr>.
pub fn counter_value(ctr: &str) -> Result<Number> {
  match state::lookup_register(&s!("\\c@{ctr}"), Vec::new())? {
    None => {
      let message = s!("Counter {} was not defined; assuming 0", ctr);
      Warn!("undefined", ctr, message);
      Ok(Number::new(0))
    },
    Some(value) => Ok(Number::new(value.value_of())),
  }
}
/// increments a named counter by a `Number`
pub fn add_to_counter(ctr: &str, value: Number) -> Result<()> {
  let v = counter_value(ctr)?.add(value);
  state::assign_register(&s!("\\c@{ctr}"), v.into(), Some(Scope::Global), Vec::new())?;
  after_assignment();
  let id_cs = T_CS!(s!("\\@{ctr}@ID"));
  def_macro(
    id_cs,
    None,
    Tokens::new(Explode!(v.value_of())),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
  )
}

/// Analog of `\stepcounter`, steps the counter and returns the expansion of
/// `\the$ctr`  Usually you should use `ref_step_counter(ctr)` instead.
pub fn step_counter(ctr: &str, noreset: bool) -> Result<()> {
  let value = counter_value(ctr)?;
  let newvalue = value.add(Number::new(1));
  let c_ctr = s!("\\c@{ctr}");
  state::assign_register(&c_ctr, newvalue.into(), Some(Scope::Global), Vec::new())?;
  state::after_assignment();
  let token_value = Tokens::new(Explode!(newvalue.value_of()));
  def_macro(
    T_CS!(s!("\\@{ctr}@ID")),
    None,
    token_value,
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
  )?;

  // and reset any within counters!
  if !noreset {
    if let Some(nested) = state::lookup_tokens(&s!("\\cl@{ctr}")) {
      for c in nested.unlist() {
        reset_counter(&c)?;
      }
    }
  }
  Ok(())
}

/// Analog of `\refstepcounter`, steps the counter and returns a hash
/// containing the keys `refnum` and `id`.
///
/// This makes it
/// suitable for use in a `properties` option to constructors.
/// The `id` is generated in parallel with the reference number
/// to assist debugging.
// TODO: Maybe these should be specialized types in Rust, rather than hashmaps?
pub fn ref_step_counter(ctype: &str, noreset: bool) -> Result<HashMap<Stored>> {
  // Defensive: under some upstream conditions the {} parameter reader pulls a
  // trailing `\par` (or similar trailing CS) into a counter-type identifier
  // before it reaches us (Cluster A: math0010095, hep-ph0204075). Strip it
  // here so the downstream `\csname @<ctype>...@ID\endcsname` and similar
  // constructions stay well-formed. We strip the same well-known sentinels
  // as latex_constructs::strip_trailing_cs.
  let ctype = {
    let mut s = ctype;
    for tail in ["\\par", "\\@startsection@hook", "\\relax"] {
      if let Some(stripped) = s.strip_suffix(tail) {
        s = stripped;
        break;
      }
    }
    s
  };
  let ctr = with_mapping("counter_for_type", ctype, |meaning| match meaning {
    Some(Stored::String(ctr)) => arena::to_string(*ctr),
    _ => ctype.to_string(),
  });
  step_counter(&ctr, noreset)?;
  maybe_preempt_refnum(&ctr, false);

  let the_ctr_id = s!("\\the{ctr}@ID");
  let the_ctr = s!("\\the{ctr}");

  let has_id: bool = if let Some(iddef) = state::lookup_definition(&T_CS!(&the_ctr_id))? {
    if let Some(params) = iddef.get_parameters() {
      params.get_num_args() == 0
    } else {
      true
    }
  } else {
    false
  };

  let the_ctr_cs = T_CS!(&the_ctr);
  let the_ctr_id_cs = T_CS!(&the_ctr_id);
  def_macro(
    T_CS!("\\@currentlabel"),
    None,
    the_ctr_cs,
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
  )?;
  if has_id {
    def_macro(
      T_CS!("\\@currentID"),
      None,
      the_ctr_id_cs,
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        ..ExpandableOptions::default()
      }),
    )?;
  }

  let id = if has_id {
    digest_literal(Tokens!(T_CS!(&the_ctr_id)))?.to_string()
  } else {
    String::new()
  };

  let refnum = digest_text(Tokens!(T_CS!(&the_ctr)))?;
  let invocation;
  {
    invocation = build_invocation(T_CS!("\\lx@make@tags"), vec![Some(Tokens!(T_OTHER!(
      ctype
    )))])?;
  }

  let tags = stomach::digest(invocation)?;

  // Any scopes activated for previous value of this counter (& any nested counters) must be
  // removed. This may also include scopes activated for \label
  deactivate_counter_scope(arena::pin(&ctr));

  // And install the scope (if any) for this reference number.
  state::assign_value("current_counter", ctr.clone(), Some(Scope::Local));

  let scope = arena::pin(format!("{ctr}:{refnum}"));
  let mut receiver = VecDeque::new();
  receiver.push_front(Stored::String(scope));
  state::assign_value(
    &s!("scopes_for_counter:{ctr}"),
    receiver,
    Some(Scope::Local),
  );
  state::activate_scope(scope);

  Ok(stored_map!(
    "tags" => Stored::Digested(tags),
    "id" => Stored::String(arena::pin(id))
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
fn maybe_preempt_refnum(ctr: &str, norefnum: bool) {
  if let Some(mapper) = state::get_label_mapping_hook() {
    let hj_refnum = T_CS!(s!("\\_PREEMPTED_REFNUM_{ctr}"));
    let hj_id = T_CS!(s!("\\_PREEMPTED_ID_{ctr}"));
    // First, restore the \the<ctr> and \the<ctr>@ID macros to defaults
    if !norefnum && state::has_meaning(&hj_refnum) {
      state::let_i(&T_CS!(s!("\\the{ctr}")), &hj_refnum, Some(Scope::Global));
    }
    if state::has_meaning(&hj_id) {
      state::let_i(&T_CS!(s!("\\the{ctr}@ID")), &hj_id, Some(Scope::Global));
    }
    let label = state::lookup_string("PEEKED_LABEL");
    let (fixedrefnum, fixedid) = mapper(&label, ctr, norefnum);
    if let Some(refnum) = fixedrefnum {
      if !norefnum {
        if !state::has_meaning(&hj_refnum) {
          // Save for later
          state::let_i(&hj_refnum, &T_CS!(s!("\\the{ctr}")), Some(Scope::Global));
        }
        let _ = def_macro(
          T_CS!(s!("\\the{ctr}")),
          None,
          ExpansionBody::Tokens(Tokens::new(Explode!(&refnum))),
          Some(ExpandableOptions {
            scope: Some(Scope::Global),
            ..Default::default()
          }),
        );
      }
    }
    if let Some(id) = fixedid {
      if !state::has_meaning(&hj_id) {
        // Save for later
        state::let_i(&hj_id, &T_CS!(s!("\\the{ctr}@ID")), Some(Scope::Global));
      }
      let _ = def_macro(
        T_CS!(s!("\\the{ctr}@ID")),
        None,
        ExpansionBody::Tokens(Tokens::new(Explode!(&id))),
        Some(ExpandableOptions {
          scope: Some(Scope::Global),
          ..Default::default()
        }),
      );
    }
    state::remove_value("PEEKED_LABEL"); // CONSUME the label
    state::assign_value(
      "PROCESSED_LABEL",
      Stored::String(arena::pin(label)),
      Some(Scope::Global),
    );
  }
}

/// Use to peek for FOLLOWING \label{...} to support label-derived reference numbers
/// (Perl: MaybePeekLabel)
pub fn maybe_peek_label() -> Result<()> {
  if state::get_label_mapping_hook().is_some() {
    let peek = crate::gullet::read_non_space()?;
    if let Some(ref token) = peek {
      if x_equals(token, &T_CS!("\\label")) {
        state::begin_semiverbatim(None);
        let arg = crate::gullet::read_arg(crate::gullet::ExpansionLevel::Off)?;
        state::end_semiverbatim()?;
        let arg_str = arg.to_string();
        let label = clean_label(&arg_str, Some("")).into_owned();
        state::assign_value(
          "PEEKED_LABEL",
          Stored::String(arena::pin(&label)),
          Some(Scope::Global),
        );
        // Put back the arg wrapped in braces so \label can re-read it
        crate::gullet::unread(Tokens!(T_BEGIN!(), arg, T_END!()));
      } else {
        state::remove_value("PROCESSED_LABEL");
        state::remove_value("PEEKED_LABEL");
      }
    }
    if let Some(token) = peek {
      crate::gullet::unread_one(token);
    }
  }
  Ok(())
}

/// Use to note a discovered label to support label-derived refererence numbers
/// Can by used by \label, among others. Note we only record the label
/// if it hasn't already been peeked, and consumed.
pub fn maybe_note_label(label: &str) {
  if state::get_label_mapping_hook().is_some() {
    let label = clean_label(label, Some(""));
    let processed = state::lookup_string("PROCESSED_LABEL");
    if processed.is_empty() || processed != label {
      // Only if not already processed
      state::remove_value("PROCESSED_LABEL");
      state::assign_value(
        "PEEKED_LABEL",
        Stored::String(arena::pin(label)),
        Some(Scope::Global),
      );
    }
  }
}

fn deactivate_counter_scope(ctr: SymStr) {
  let (scopes_for_counter, nested_counters) = arena::with(ctr, |cstr| {
    (
      s!("scopes_for_counter:{cstr}"),
      s!("nested_counters_{cstr}"),
    )
  });
  // with_value avoids the outer Stored::clone by reading through a
  // borrow; we still collect the scope SymStrs (Copy) or panic-pointers
  // into owned Vecs to outlive the borrow.
  let scope_syms: Vec<SymStr> = state::with_value(&scopes_for_counter, |v| match v {
    Some(Stored::VecDequeStored(stored_scopes)) => stored_scopes
      .iter()
      .map(|s| match s {
        Stored::String(scope) => *scope,
        _ => panic!("assignment scopes should be stored as strings, got: {s:?}"),
      })
      .collect(),
    _ => Vec::new(),
  });
  for scope in scope_syms {
    state::deactivate_scope(scope);
  }

  // TODO: if we ever want to unshift from the nested_counters, we'll need to also use
  // Stored::VecDequeStored for them.
  let inner_ctrs: Vec<SymStr> = state::with_value(&nested_counters, |v| match v {
    Some(Stored::Strings(stored_counters)) => stored_counters.iter().copied().collect(),
    _ => Vec::new(),
  });
  for inner_ctr in inner_ctrs {
    deactivate_counter_scope(inner_ctr);
  }
}

/// For UN-numbered units.
/// Like `RefStepCounter`, but only steps the "uncounter",
/// and returns only the id;  This is useful for unnumbered cases
/// of objects that normally get both a refnum and id.
pub fn ref_step_id(ctype: &str) -> Result<HashMap<Stored>> {
  let ctr = with_mapping("counter_for_type", ctype, |mapping| match mapping {
    Some(map) => map.to_string(),
    None => ctype.to_string(),
  });
  let unctr = s!("UN{ctr}");
  // Perl Package.pm L863-864 ("Avoid fatals..."): if `\c@UN<ctr>` isn't
  // defined as a register, `NewCounter(ctr)` creates both `\c@<ctr>` and
  // `\c@UN<ctr>` and the associated `\the<ctr>@ID` expansion. Without it,
  // unnumbered-section callers like `\specialsection*{}` (which amsart
  // Let's to `\chapter*{}` even though amsart has no chapter counter) hit
  // Error:undefined:\thechapter@ID because the counter was never created.
  let unctr_cmd = s!("\\c@{unctr}");
  let unctr_defined = state::lookup_register(&unctr_cmd, Vec::new())
    .ok()
    .flatten()
    .is_some();
  if !unctr_defined {
    let _ = new_counter(&ctr, "document", None);
  }
  step_counter(&unctr, false)?;
  maybe_preempt_refnum(&ctr, true);
  let cunctr_val = lookup_number(&s!("\\c@{unctr}"))
    .unwrap_or_default()
    .value_of();
  def_macro(
    T_CS!(s!("\\@{ctr}@ID")),
    None,
    Tokens!(T_OTHER!("x"), Explode!(cunctr_val)),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
  )?;

  let the_ctr_id = s!("\\the{ctr}@ID");
  def_macro(T_CS!("\\@currentID"), None, T_CS!(&the_ctr_id), None)?;
  Ok(stored_map!("id" =>
    clean_id(&digest_literal(T_CS!(the_ctr_id))?.to_string())))
}

/// Recycle the last ID without incrementing (Perl: RefCurrentID)
/// Useful if the last ID-ed box got pruned.
pub fn ref_current_id(ctype: &str) -> Result<HashMap<Stored>> {
  let ctr = with_mapping("counter_for_type", ctype, |mapping| match mapping {
    Some(map) => map.to_string(),
    None => ctype.to_string(),
  });
  let the_ctr_id = s!("\\the{ctr}@ID");
  let id = clean_id(&digest_literal(T_CS!(the_ctr_id))?.to_string());
  Ok(stored_map!("id" => id))
}

/// Resets the counter `ctr` to zero.
pub fn reset_counter(ctr: &Token) -> Result<()> {
  let (c_ctr, c_un_ctr, ctr_id) =
    ctr.with_str(|ctr| (s!("\\c@{ctr}"), s!("\\c@UN{ctr}"), s!("\\@{ctr}@ID")));
  state::assign_register(
    &c_ctr,
    Number::new(0).into(),
    Some(Scope::Global),
    Vec::new(),
  )?;
  if !ctr.with_str(|cstr| cstr.starts_with("UN")) {
    // but not UN
    state::assign_register(
      &c_un_ctr,
      Number::new(0).into(),
      Some(Scope::Global),
      Vec::new(),
    )?;
  }
  def_macro(
    T_CS!(ctr_id),
    None,
    Tokens!(T_OTHER!("0")),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
  )?;
  // and reset any within counters!
  if let Some(nested) = state::lookup_tokens(&s!("\\cl@{ctr}")) {
    for c in nested.unlist() {
      reset_counter(&c)?;
    }
  }
  Ok(())
}

/// Create id, and tags for an itemize type \item
pub fn ref_step_item_counter(tag_opt: Option<&Tokens>) -> Result<HashMap<Stored>> {
  let counter = state::lookup_string("itemcounter");
  let n = lookup_int("itemization_items");
  state::assign_value("itemization_items", n + 1, None);
  let mut attr: HashMap<Stored> = HashMap::default();
  if n > 0 {
    if let Some(sep) = lookup_dimension("\\itemsep") {
      let default_opt = lookup_dimension("\\lx@default@itemsep");
      if default_opt.is_none() || sep.value_of() != default_opt.unwrap().value_of() {
        attr.insert("itemsep", sep.into());
      }
    }
  }

  let mut result = if let Some(tag) = tag_opt {
    let mut props = ref_step_id(&counter)?;
    if tag.is_empty() {
      return Ok(props);
    }
    let formatter = if counter.starts_with("@desc") {
      T_CS!("\\descriptionlabel")
    } else {
      T_CS!("\\makelabel")
    };
    let counter_name = s!("\\{counter}name");
    let typename = if is_defined(&counter_name) {
      T_CS!(counter_name)
    } else {
      T_CS!("\\itemtyperefname")
    };

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
    tag_tokens.extend(
      build_invocation(T_CS!("\\lx@make@tags"), vec![Some(Tokens!(T_OTHER!(
        counter
      )))])?
      .unlist(),
    );
    tag_tokens.push(T_END!());

    let tags = stomach::digest(tag_tokens)?;
    if !tags.is_empty()? {
      props.insert("tags", tags.into());
    }
    props
  } else {
    ref_step_counter(&counter, false)?
  };
  for (k, v) in attr.into_iter() {
    result.insert_sym(k, v);
  }
  Ok(result)
}

/// configuration for begin_itemize
#[derive(Debug, Default, Clone)]
pub struct BeginItemizeOptions {
  /// disable nested id suffix based on stacking level
  pub nolevel:     bool,
  /// enumitem series
  pub series:      Option<Tokens>,
  /// start at a custom value
  pub start:       Option<Number>,
  /// enumitem resume?
  pub resume:      Option<String>,
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
) -> Result<HashMap<Stored>> {
  // The list-type and level of the *containing* list (if any!)
  let outercounter = state::lookup_string("itemcounter");
  let outerlevel = if !outercounter.is_empty() {
    lookup_int(&s!("{outercounter}level"))
  } else {
    0
  };
  let counter = counter.unwrap_or("@item");
  let listlevel = lookup_int("itemization_level") + 1; // level for this list overall
  let level = lookup_int(&s!("{counter}level")) + // level for lists of specific type
    (if options.nolevel { 0 } else { 1 });
  AssignRegister!(
    "\\itemsep",
    lookup_dimension("\\lx@default@itemsep")
      .unwrap_or_default()
      .into()
  );
  state::assign_value("itemization_level", listlevel, None);
  state::assign_value(&s!("{counter}level"), level, None);
  state::assign_value("itemization_items", 0, None);
  let listpostfix = roman!(listlevel).to_string();
  let postfix = roman!(level).to_string();
  let mut usecounter = counter.to_string();
  if !options.nolevel && !postfix.is_empty() {
    usecounter.push_str(&postfix);
  }
  if !itype.is_empty() {
    let itype_cs = T_CS!(s!("\\{itype}@item"));
    state::let_i(&T_CS!("\\item"), &itype_cs, None);
  }
  // In case within odd environment.
  state::let_i(&T_CS!("\\par"), &T_CS!("\\lx@normal@par"), None);
  def_macro(
    T_CS!("\\@listctr"),
    None,
    Tokens!(Explode!(usecounter)),
    None,
  )?;
  // Now arrange that this list's id's are relative to the current (outer) item (if any)
  // And that the items within this list's id's are relative to this (new) list.
  state::assign_value("itemcounter", Stored::String(arena::pin(&usecounter)), None);
  let listcounter = s!("@itemize{listpostfix}");
  if lookup_definition(&T_CS!(s!("\\c@{listcounter}")))?.is_none() {
    //Create new list counters as needed
    new_counter(&listcounter, "", None)?;
  }
  if !outercounter.is_empty() {
    // Make this list's ID relative to outer list's ID
    let outerusecounter = s!("{outercounter}{}", roman!(outerlevel).to_string());
    let thectr = s!("\\the{listcounter}@ID");
    let theexpansion = s!("\\the{outerusecounter}@ID.I\\arabic{{{listcounter}}}");
    def_macro(
      T_CS!(thectr),
      None,
      mouth::tokenize_internal(&theexpansion),
      None,
    )?;

    // AND reset this list's counter when the outer item is stepped
    let mut cl_toks = vec![T_CS!(&listcounter)];
    let cl_name = s!("\\cl@{outerusecounter}");
    let existing = state::with_value(&cl_name, |v| match v {
      Some(Stored::Tokens(tks)) => tks.clone().unlist(),
      _ => Vec::new(),
    });
    cl_toks.extend(existing);
    state::assign_value(
      &cl_name,
      Stored::Tokens(Tokens::new(cl_toks)),
      Some(Scope::Global),
    );
  }
  // format the id of \item's relative to the id of this list.
  // Perl: Tokens(T_CS('\the' . $listcounter . '@ID'), T_OTHER('.i'),
  //              T_CS('\@' . $usecounter . '@ID'))
  // — build the Tokens array directly from 3 explicit tokens rather than
  // round-tripping through a string tokenizer. This matters when `usecounter`
  // contains digits (e.g. "count1" from \usecounter{count1}): `tokenize_internal`
  // on a literal string "\@count1@ID" would split into \@count + "1" + "@" + "ID"
  // because TeX's CS reader stops at digits (digits are catcode OTHER even in
  // the style table). Perl uses T_CS($name) to build a single CS token directly
  // by name, bypassing tokenization.
  let useexp = Tokens::new(vec![
    T_CS!(s!("\\the{listcounter}@ID")),
    T_OTHER!(".i"),
    T_CS!(s!("\\@{usecounter}@ID")),
  ]);
  def_macro(T_CS!(s!("\\the{usecounter}@ID")), None, useexp, None)?;

  let mut series = if let Some(s) = options.series {
    s.to_string()
  } else {
    String::new()
  };
  if let Some(start) = options.start {
    SetCounter!(usecounter, start);
    add_to_counter(&usecounter, Number(-1))?;
  } else if let Some(s) = match options.resume {
    Some(s) => Some(s),
    None => options.resume_star,
  } {
    if s != "noseries" {
      series = s.clone();
      let last_val = lookup_int(&s!("enumitem_series_{s}_last"));
      if last_val != 0 {
        SetCounter!(usecounter, Number(last_val));
      }
    }
  } else {
    reset_counter(&T_OTHER!(&usecounter))?;
  }

  let mut rsc = ref_step_counter(&s!("@itemize{listpostfix}"), false)?;
  rsc.insert("counter", usecounter.into());
  rsc.insert("series", series.into());
  Ok(rsc)
}

/// Set the itemization style for a given level.
/// Perl: setItemizationStyle($stuff, $level)
/// If $level is not given, uses the current @itemlevel.
/// Defines \labelitem$level to $stuff.
pub fn set_itemization_style(stuff: Option<&Tokens>, level: Option<i32>) -> Result<()> {
  if let Some(stuff) = stuff {
    if stuff.is_empty() {
      return Ok(());
    }
    let level = level.unwrap_or_else(|| lookup_int("@itemlevel").max(0) as i32);
    let level_str = roman_aux(level);
    let cs_name = s!("\\labelitem{level_str}");
    def_macro(T_CS!(&cs_name), None, stuff.clone(), None)?;
  }
  Ok(())
}

/// Set the enumeration style for a given level.
/// Perl: setEnumerationStyle($stuff, $level)
/// Parses the style tokens to detect A/a/I/i/1 patterns
/// and defines \theenum$level and \labelenum$level accordingly.
pub fn set_enumeration_style(stuff: Option<&Tokens>, level: Option<i32>) -> Result<()> {
  if let Some(stuff) = stuff {
    if stuff.is_empty() {
      return Ok(());
    }
    let level = level.unwrap_or_else(|| lookup_int("enumlevel").max(0) as i32);
    let level_str = roman_aux(level);
    // Iterate the borrowed token slice — no clone needed, only reads.
    let tokens = stuff.unlist_ref();
    let mut out: Vec<Token> = Vec::with_capacity(tokens.len());
    let ctr = T_OTHER!(s!("enum{level_str}"));
    let mut i = 0;
    while i < tokens.len() {
      let t = tokens[i];
      if t.get_catcode() == Catcode::BEGIN {
        // Copy braced groups verbatim
        out.push(t);
        let mut brlevel = 1i32;
        i += 1;
        while brlevel > 0 && i < tokens.len() {
          let tt = tokens[i];
          if tt.get_catcode() == Catcode::BEGIN {
            brlevel += 1;
          } else if tt.get_catcode() == Catcode::END {
            brlevel -= 1;
          }
          out.push(tt);
          i += 1;
        }
      } else {
        let ch = char::from_u32(t.get_charcode()).unwrap_or('\0');
        let cat = t.get_catcode();
        match (ch, cat) {
          ('A', Catcode::LETTER) => {
            // \Alph{enum$level}
            def_macro(
              T_CS!(s!("\\theenum{level_str}")),
              None,
              Tokens::new(vec![T_CS!("\\Alph"), T_BEGIN!(), ctr, T_END!()]),
              None,
            )?;
            out.push(T_CS!(s!("\\theenum{level_str}")));
          },
          ('a', Catcode::LETTER) => {
            // \alph{enum$level}
            def_macro(
              T_CS!(s!("\\theenum{level_str}")),
              None,
              Tokens::new(vec![T_CS!("\\alph"), T_BEGIN!(), ctr, T_END!()]),
              None,
            )?;
            out.push(T_CS!(s!("\\theenum{level_str}")));
          },
          ('I', Catcode::LETTER) => {
            // \Roman{enum$level}
            def_macro(
              T_CS!(s!("\\theenum{level_str}")),
              None,
              Tokens::new(vec![T_CS!("\\Roman"), T_BEGIN!(), ctr, T_END!()]),
              None,
            )?;
            out.push(T_CS!(s!("\\theenum{level_str}")));
          },
          ('i', Catcode::LETTER) => {
            // \roman{enum$level}
            def_macro(
              T_CS!(s!("\\theenum{level_str}")),
              None,
              Tokens::new(vec![T_CS!("\\roman"), T_BEGIN!(), ctr, T_END!()]),
              None,
            )?;
            out.push(T_CS!(s!("\\theenum{level_str}")));
          },
          ('1', Catcode::OTHER) => {
            // \arabic{enum$level}
            def_macro(
              T_CS!(s!("\\theenum{level_str}")),
              None,
              Tokens::new(vec![T_CS!("\\arabic"), T_BEGIN!(), ctr, T_END!()]),
              None,
            )?;
            out.push(T_CS!(s!("\\theenum{level_str}")));
          },
          _ => {
            out.push(t);
          },
        }
        i += 1;
      }
    }
    // Define \labelenum$level = { out }
    let mut label_tokens = vec![T_BEGIN!()];
    label_tokens.extend(out);
    label_tokens.push(T_END!());
    def_macro(
      T_CS!(s!("\\labelenum{level_str}")),
      None,
      Tokens::new(label_tokens),
      None,
    )?;
  }
  Ok(())
}

/// Copies the current id, tags, and inlist counter values into whatsit properties
/// Perl: RescueCaptionCounters (latex_constructs.pool.ltxml L3260-3271)
pub fn rescue_caption_counters(captype: &str, whatsit: &mut Whatsit) {
  let tagskey = &s!("{captype}_tags");
  if let Some(tags) = state::remove_value(tagskey) {
    whatsit.set_property("tags", tags);
  }
  let idkey = s!("{captype}_id");
  if let Some(id) = state::remove_value(&idkey) {
    whatsit.set_property("id", id);
  }
  let inlistkey = s!("{captype}_inlist");
  if let Some(inlist) = state::remove_value(&inlistkey) {
    whatsit.set_property("inlist", inlist);
  }
}

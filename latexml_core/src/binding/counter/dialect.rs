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

use crate::{
  BoxOps, RegisterValue,
  binding::{
    content::{build_invocation, digest_literal, digest_text},
    def::dialect::{RegisterOptions, def_macro, def_register, is_defined},
  },
  common::{
    arena::{self, SymHashMap as HashMap, SymStr},
    cleaners::{clean_id, clean_label, roman_aux},
    error::*,
    number::Number,
    numeric_ops::NumericOps,
  },
  definition::{Definition, ExpansionBody, expandable::ExpandableOptions},
  mouth, state,
  state::*,
  stomach,
  token::*,
  tokens::{TeXString, Tokens},
  whatsit::Whatsit,
};

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

/// Perl `addtoCounterReset` (latex_constructs.pool.ltxml:3005-3012): put `\<ctr>` and `\UN<ctr>`
/// at the front of `\cl@<within>`, globally, as latex.ltx:10156's `\@cons` `\xdef`s — a reset
/// registered inside a group outlives it — and re-derive the list's macro.
/// Repro macro-state/counter_reset_lists_are_global.
pub fn add_to_counter_reset(ctr: &str, within: &str) -> Result<()> {
  if copy_provides_keeps_counter(ctr) {
    return Ok(());
  }
  let reg = s!("\\cl@{within}");
  let mut toks = vec![T_CS!(ctr), T_CS!(s!("UN{ctr}"))];
  if let Some(prev) = lookup_tokens(&reg) {
    toks.extend(prev.unlist());
  }
  assign_value(&reg, Stored::Tokens(Tokens::new(toks)), Some(Scope::Global));
  sync_reset_list_macro(within)?;
  define_hyper_counter_name(ctr, within)
}

/// The `\theH<ctr>` hyperref builds a counter's anchor from (`\hyper@makecurrent`), as the
/// kernel makes it since 2024-11-01: latex.ltx:10145 `\@definecounter` xdefs it as
/// `\the\value{<ctr>}`, and :10157 `\@addtoreset` as `\theH<within>.\the\value{<ctr>}`, so
/// hyperref.sty:6607 skips its own `\providecommand` list. Perl's counters have none. A package
/// building on them needs them: pgfplots' `\theHpgfplotslink{\theHsection.…}`
/// (pgfplots.code.tex:6138, through `\hyper@makecurrent{pgfplotslink}` at :6123). LaTeXML's
/// pseudo-counter `document` is no parent: no `\theHdocument` prefix.
/// Guard: `perfect_kernel_batch64::kernel_counters_have_hyperref_names`.
fn define_hyper_counter_name(ctr: &str, within: &str) -> Result<()> {
  let mut body: Vec<Token> = Vec::new();
  if !within.is_empty() && within != "document" {
    body.push(T_CS!(s!("\\theH{within}")));
    body.push(T_OTHER!("."));
  }
  body.push(T_CS!("\\the"));
  body.push(T_CS!("\\value"));
  body.push(T_BEGIN!());
  body.extend(ExplodeText!(ctr));
  body.push(T_END!());
  def_macro(
    T_CS!(s!("\\theH{ctr}")),
    None,
    ExpansionBody::from(Tokens::new(body)),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
  )
}

/// Perl `remfromCounterReset` (latex_constructs.pool.ltxml:3014-3022): drop `\<ctr>` and
/// `\UN<ctr>` from `\cl@<within>`, globally (latex.ltx:10170 `\xdef`), when there is a list.
pub fn remove_from_counter_reset(ctr: &str, within: &str) -> Result<()> {
  let reg = s!("\\cl@{within}");
  if let Some(prev) = lookup_tokens(&reg) {
    let (t, unt) = (T_CS!(ctr), T_CS!(s!("UN{ctr}")));
    let kept: Vec<Token> = prev
      .unlist()
      .into_iter()
      .filter(|x| *x != t && *x != unt)
      .collect();
    assign_value(&reg, Stored::Tokens(Tokens::new(kept)), Some(Scope::Global));
    sync_reset_list_macro(within)?;
  }
  Ok(())
}

/// Mirror the reset list into the `\cl@<ctr>` MACRO latex.ltx exposes.
///
/// latex.ltx:10140-10142 `\@definecounter` lets `\cl@<ctr>` to `\@empty` and
/// :10156 `\@addtoreset` `\@cons`es `\@elt{<child>}` onto it, so the reset list
/// is a macro raw code expands and rewrites: contract.sty:336
/// `\edef\cl@Clause{\cl@Clause\cl@contractClause}` (contract-example-de/-en,
/// 44 errors each), afthesis.cls:44-49's `\@removefromreset` re-`\edef` with a
/// filtering `\@elt` (usethesis: "`\cl@chapter` expands into itself"). LaTeXML
/// keeps the list as the State VALUE `\cl@<ctr>` (Perl Package.pm:674, entries
/// `\<child>`/`\UN<child>`), which stays authoritative for stepping; this
/// re-derives the macro (`\@elt{child}` per non-`UN` entry) after every
/// mutation so both faces agree. Perl shares the gap (sectioning topic).
/// Guard: `perfect_kernel_batch54::reset_list_is_an_expandable_cl_macro`.
pub fn sync_reset_list_macro(ctr: &str) -> Result<()> {
  let clctr = s!("\\cl@{ctr}");
  let mut body: Vec<Token> = Vec::new();
  if let Some(list) = lookup_tokens(&clctr) {
    for t in list.unlist() {
      let name = t.with_str(|s| s.to_string());
      if name.starts_with("UN") {
        continue;
      }
      body.push(T_CS!("\\@elt"));
      body.push(T_BEGIN!());
      body.extend(ExplodeText!(name));
      body.push(T_END!());
    }
  }
  def_macro(
    T_CS!(&clctr),
    None,
    ExpansionBody::from(Tokens::new(body)),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
  )
}

/// Defines a new counter named $ctr.
/// If `within` is defined, `ctr` will be reset whenever `within` is incremented.
pub fn new_counter(ctr: &str, within: &str, options_opt: Option<NewCounterOptions>) -> Result<()> {
  // (a stand-in's copy keeps a counter the binding owns as the binding numbers it: `state::copy_provides_keeps_counter`)
  if copy_provides_keeps_counter(ctr) {
    return Ok(());
  }
  // (any other counter a stand-in's copy declares is declared for real — its `\c@`, `\cl@`, `\the`, `\p@` names outlive
  // the copy's overlay, as the real class's do: `state::copy_provides_keeps_counter`)
  let _declared = (crate::definition::origin::copy_provides() != 0)
    .then(|| crate::definition::origin::CopyProvidesGuard::new(0));
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
      let relax_meaning = lookup_meaning(&T_RELAX!());
      let prev_meaning = lookup_meaning(&cs_cctr);
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
  // An alias counter (`register_counter_alias`) given a register of its own (`\let\c@case\relax\newcounter{case}`
  // after llncs's `\@spothm{case}[theorem]`: 2303.08726) is an alias no more: `\refstepcounter{case}` steps the new
  // register, as in LaTeX, where the alias was only the `\let` of `\c@case`. A counter whose `\c@` is still the alias's
  // register keeps it (LaTeX refuses that `\newcounter`, "Command \c@case already defined").
  if !prev_defn.as_ref().is_some_and(|defn| defn.is_register()) && is_counter_alias(ctr) {
    assign_mapping::<Stored>("counter_for_type", ctr, None);
    assign_mapping::<Stored>("counter_alias", ctr, None);
  }
  after_assignment();
  if !has_value(&clctr) {
    assign_value(&clctr, Tokens!(), Some(Scope::Global));
  }
  sync_reset_list_macro(ctr)?;
  def_register(T_CS!(&cunctr), None, Number::new(0), None)?;
  if !has_value(&clunctr) {
    assign_value(&clunctr, Tokens!(), Some(Scope::Global));
  }

  if !within.is_empty() {
    let clwithin = s!("\\cl@{within}");
    let clunwithin = s!("\\cl@UN{within}");
    let x = if let Some(cl) = lookup_tokens(&clwithin) {
      cl.unlist()
    } else {
      Vec::new()
    };
    let mut clwithin_tokens = vec![T_CS!(ctr), T_CS!(&unctr)];
    clwithin_tokens.extend(x);
    assign_value(
      &clwithin,
      Stored::Tokens(Tokens::new(clwithin_tokens)),
      Some(Scope::Global),
    );

    let mut unx = if let Some(clun) = lookup_tokens(&clunwithin) {
      clun.unlist()
    } else {
      Vec::new()
    };
    let mut clunwithin_tokens = vec![T_CS!(unctr)];
    clunwithin_tokens.append(&mut unx);

    assign_value(
      &clunwithin,
      Stored::Tokens(Tokens::new(clunwithin_tokens)),
      Some(Scope::Global),
    );
    sync_reset_list_macro(within)?;
  }
  define_hyper_counter_name(ctr, within)?;

  // default is equivalent to \arabic{ctr}, but w/o using the LaTeX macro:
  // Perl Package.pm:686 `\the<ctr>` = `\lx@counter@arabic{<ctr>}`, a Token
  // body the format dump can carry (a closure is opaque to it).
  let ctr_string = ctr.to_string();
  def_macro(
    T_CS!(s!("\\the{}", ctr)),
    None,
    Some(ExpansionBody::Tokens(mouth::tokenize_internal(
      TeXString::assembled(s!("\\lx@counter@arabic{{{ctr_string}}}")),
    ))),
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

  // (a standard-class derivative re-declaring a counter its standard class's binding declared — article.cls's
  // `\newcounter{figure}` in a full copy — keeps the binding's ids, LaTeXML's markup: `S1.F1`, not `F1`; the register,
  // its reset list and `\the<ctr>` are the copy's, as TeX makes them. OXIDIZED_DESIGN_DIVERGENCES #731 rule 13;
  // 2402.12725's article_saj.cls; guard `perfect_kernel_batch77w::standard_class_derivative_patches_stand`)
  let ids = T_CS!(s!("\\the{ctr}@ID"));
  if crate::binding::content::standard_class_binding_assigned(meaning_key(&ids)) {
    return Ok(());
  }
  define_counter_ids(ctr, within, options_opt.as_ref())
}

/// The id half of [`new_counter`] (Perl Package.pm:689-705, `NewCounter`'s `idprefix`/`idwithin`/`nested`): records
/// `@ID@prefix@<ctr>` and `nested_counters_<ctr>`, and defines `\the<ctr>@ID` and `\@<ctr>@ID`. A binding calls it
/// alone for a counter a raw class declared, to give it the ids its standard-class binding gives (the KOMA classes,
/// `koma_script.rs`: `S1.SS1`, not `section1.subsection1`); the register and its reset list stay the class's.
pub fn define_counter_ids(
  ctr: &str,
  within: &str,
  options_opt: Option<&NewCounterOptions>,
) -> Result<()> {
  if let Some(options) = options_opt
    && !options.nested.is_empty()
  {
    assign_value(
      &s!("nested_counters_{}", ctr),
      options.nested.clone(),
      Some(Scope::Global),
    )
  }

  let mut prefix = match options_opt {
    None => String::new(),
    Some(opt) => opt.idprefix.to_string(),
  };
  if !prefix.is_empty() {
    assign_value(
      &s!("@ID@prefix@{}", ctr),
      prefix.clone(),
      Some(Scope::Global),
    );
  } else {
    prefix = lookup_string(&s!("@ID@prefix@{}", ctr));
    if prefix.is_empty() {
      prefix = ctr.to_string();
    }
  }
  prefix = clean_id(&prefix);

  if !prefix.is_empty() {
    let idwithin = match options_opt {
      Some(opts) => {
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
    // Token bodies, as Perl's `DefMacroI` string bodies (Package.pm:695-703):
    // a closure here is opaque to the format dump (recorded as an unresolvable
    // self-alias), so every kernel counter's `\the<ctr>@ID` was lost on the
    // dump path and a class with no binding (ptptex) generated ids from an
    // undefined formatter — `subequations` children `.1`/`X.1`, colliding.
    if !idwithin.is_empty() {
      def_macro(
        T_CS!(thectrid),
        None,
        Some(ExpansionBody::Tokens({
          // Perl Package.pm L696 probes `\lx@empty`, NOT `\@empty` — and
          // deliberately so: `\@empty` is LaTeX-pool-only (latex_base.rs
          // aliases it to `\lx@empty`), while `\lx@empty` is engine-level
          // (base_schema.rs) and exists in PLAIN TeX documents too. With
          // `\@empty` here, a plain-TeX document's `\the<ctr>@ID` probe
          // compared `\thedocument@ID` (an alias of `\lx@empty`) against an
          // UNDEFINED `\@empty` → unequal → the \else branch re-expands the
          // parent's @ID formatter, spinning the gullet into a 4-token
          // expansion cycle (caught by the cycle guard as a phantom
          // `Fatal:Timeout:Recursion`, swallowed inside math-parser
          // semantics — `create_xmrefs`/`get_xmarg_id`). Witness
          // math0402448 (plain TeX + 3464 formulae): "Conversion failed:
          // 1 fatal error" with no Fatal: line in the log.
          // The names are spliced in as characters (`id_name_chars`, as `define_hyper_counter_name` does), never
          // re-read from a string: a counter named `claim_nb` or `c_n` (2606.16543, 1912.04210) made the `_` a
          // subscript, digested for the id ("Script _ can only appear in math mode"; Perl Package.pm:695-702 alike,
          // KNOWN_PERL_ERRORS #621). Guard `perfect_kernel_batch65m::counter_name_with_underscore_is_its_id`.
          let mut body = vec![T_CS!("\\expandafter"), T_CS!("\\ifx"), T_CS!("\\csname")];
          body.extend(id_name_chars(&s!("the{idwithin}@ID")));
          body.extend([
            T_CS!("\\endcsname"),
            T_CS!("\\lx@empty"),
            T_CS!("\\else"),
            T_CS!("\\csname"),
          ]);
          body.extend(id_name_chars(&s!("the{idwithin}@ID")));
          body.extend([T_CS!("\\endcsname"), T_OTHER!("."), T_CS!("\\fi")]);
          body.extend(id_name_chars(&prefix));
          body.extend(counter_id_number(&ctr_string));
          Tokens::new(body)
        })),
        Some(ExpandableOptions {
          scope: Some(Scope::Global),
          ..ExpandableOptions::default()
        }),
      )?;
    } else {
      def_macro(
        T_CS!(thectrid),
        None,
        Some(ExpansionBody::Tokens({
          let mut body = id_name_chars(&prefix);
          body.extend(counter_id_number(&ctr_string));
          Tokens::new(body)
        })),
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
/// A name in a `\the<ctr>@ID` body as characters: letters and `@` letters, the rest other, as `tokenize_internal`
/// reads a name made of them (the format dump keeps those bodies), but a `_`, `^` or `&` in the name is a character too.
fn id_name_chars(name: &str) -> Vec<Token> {
  name
    .chars()
    .map(|c| {
      let mut buf = [0u8; 4];
      let c_str: &str = c.encode_utf8(&mut buf);
      if c == ' ' {
        T_SPACE!()
      } else if c == '@' || c.is_alphabetic() {
        T_LETTER!(c_str)
      } else {
        T_OTHER!(c_str)
      }
    })
    .collect()
}
/// `\csname @<ctr>@ID\endcsname`, the counter's number in an id, the name spliced in as characters.
fn counter_id_number(ctr: &str) -> Vec<Token> {
  let mut toks = vec![T_CS!("\\csname")];
  toks.extend(id_name_chars(&s!("@{ctr}@ID")));
  toks.push(T_CS!("\\endcsname"));
  toks
}
/// Fetches the value associated with the counter C<$ctr>.
pub fn counter_value(ctr: &str) -> Result<Number> {
  match lookup_register(&s!("\\c@{ctr}"), Vec::new())? {
    None => {
      // Perl Package.pm L712 quotes the counter name: `Counter '$ctr' ...`.
      let message = s!("Counter '{}' was not defined; assuming 0", ctr);
      Warn!("undefined", ctr, message);
      Ok(Number::new(0))
    },
    Some(value) => Ok(Number::new(value.value_of())),
  }
}
/// increments a named counter by a `Number`
pub fn add_to_counter(ctr: &str, value: Number) -> Result<()> {
  let v = counter_value(ctr)?.add(value);
  assign_register(&s!("\\c@{ctr}"), v.into(), Some(Scope::Global), Vec::new())?;
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
  assign_register(&c_ctr, newvalue.into(), Some(Scope::Global), Vec::new())?;
  after_assignment();
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
  if !noreset && let Some(nested) = lookup_tokens(&s!("\\cl@{ctr}")) {
    for c in nested.unlist() {
      reset_counter(&c)?;
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
  let ctype = strip_counter_type_sentinel(ctype);
  let ctr = counter_for_type(ctype);
  step_counter(&ctr, noreset)?;
  label_stepped_counter(ctype, &ctr)
}

thread_local! {
  /// Per counter, the type whose tags the step in progress makes (`ref_step_counter_planned`),
  /// taken by the kernel's `\lx@refstepcounter`: theorem types share a counter (`counter_for_type`)
  /// and LaTeX's `\refstepcounter` is passed the counter.
  static PENDING_STEPS: std::cell::RefCell<rustc_hash::FxHashMap<String, String>> =
    std::cell::RefCell::new(rustc_hash::FxHashMap::default());
  /// Per counter, the props the kernel's label computed for a planned step.
  static REFSTEP_PROPS: std::cell::RefCell<rustc_hash::FxHashMap<String, HashMap<Stored>>> =
    std::cell::RefCell::new(rustc_hash::FxHashMap::default());
}

/// Forgets pending steps and recorded props (per document: they hold its symbols).
pub fn reset_refstep_state() {
  PENDING_STEPS.with(|steps| steps.borrow_mut().clear());
  REFSTEP_PROPS.with(|slot| slot.borrow_mut().clear());
}

/// The type whose tags the step in progress on `counter` makes, if a binding planned one (the
/// kernel's `\lx@refstepcounter`).
pub fn take_pending_step(counter: &str) -> Option<String> {
  PENDING_STEPS.with(|steps| steps.borrow_mut().remove(counter))
}

/// Records the props the kernel's `\refstepcounter` label computed for a planned step on `counter`.
pub fn record_refstep_props(counter: &str, props: &HashMap<Stored>) {
  REFSTEP_PROPS.with(|slot| slot.borrow_mut().insert(counter.to_string(), props.clone()));
}

/// The tags of the step a raw `\refstepcounter` made (`note_raw_refstep`): one no constructor took, so no element
/// carries its number.
const RAW_REFSTEP_TAGS: &str = "raw_refstep_tags";
/// The counter of that step.
const RAW_REFSTEP_COUNTER: &str = "raw_refstep_counter";
/// Which step it is (a serial, one per raw step), told apart from another step's on the object one numbered
/// (`Document::take_raw_refstep_number`).
const RAW_REFSTEP_SERIAL: &str = "raw_refstep_serial";
/// The step [`forget_raw_refstep`] forgot (a listing's start), for [`renote_forgotten_raw_refstep`].
const RAW_REFSTEP_FORGOTTEN: &str = "raw_refstep_forgotten";
/// The step an amsmath display's labels name ([`hold_raw_refstep_for_display`]): `Some` inside one.
const RAW_REFSTEP_DISPLAY: &str = "raw_refstep_display";

thread_local! {
  static RAW_REFSTEP_SERIALS: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
}

/// A raw counter step a `\label` names ([`current_raw_refstep`]): the `tags` LaTeXML makes for every step and the
/// step's serial.
#[derive(Clone, Debug)]
pub struct RawRefstep {
  /// The step's `tags` (`label_stepped_counter`), whose `refnum` is what `\ref` prints.
  pub tags:   crate::Digested,
  /// Which step it is: two steps that print the same number are still two.
  pub serial: i64,
}

/// Records the step a raw `\refstepcounter` made on `counter` (its `props` from [`label_stepped_counter`]), whose
/// number LaTeX's `\label` takes from `\@currentlabel` (latex.ltx:14963-14967), for the labels made after it
/// ([`current_raw_refstep`]). Local, as LaTeX's `\@currentlabel` is (`\protected@edef`): a box's step stays in
/// the box. Any other step forgets it ([`label_stepped_counter`]). Witnesses: 2506.01072 (`\newtcbtheorem`),
/// 2505.20088 (`\newtcolorbox[auto counter]`), 2402.06608 (`[blend into=figures]`), 2609.20563 (`\newtcblisting`).
pub fn note_raw_refstep(counter: &str, props: &HashMap<Stored>) {
  if let Some(tags @ Stored::Digested(_)) = props.get("tags") {
    let serial = RAW_REFSTEP_SERIALS.with(|c| {
      c.set(c.get() + 1);
      c.get()
    });
    assign_value(RAW_REFSTEP_TAGS, tags.clone(), Some(Scope::Local));
    assign_value(RAW_REFSTEP_COUNTER, counter.to_string(), Some(Scope::Local));
    assign_value(RAW_REFSTEP_SERIAL, Stored::Int(serial), Some(Scope::Local));
  }
}

/// The raw step recorded now ([`note_raw_refstep`]) stays recorded after the group it was made in: a package that
/// makes `\@currentlabel` global after its `\refstepcounter` (algorithm2e.sty:1663 `\gdef\@currentlabel{\theAlgoLine}`,
/// :1659 with hyperref), so that a `\label` after the group (`\While{c}{\label{x} …}`, the line's number made in the
/// header's group) still names the step (`\lx@refstep@global`, algorithm2e_sty.rs). Witness 1602.05866
/// (`algline:mainloop`, PDF 8).
pub fn globalize_raw_refstep() {
  if let Some((tags, counter, serial)) = recorded_raw_refstep() {
    assign_value(
      RAW_REFSTEP_TAGS,
      Stored::Digested(tags),
      Some(Scope::Global),
    );
    assign_value(RAW_REFSTEP_COUNTER, counter, Some(Scope::Global));
    assign_value(RAW_REFSTEP_SERIAL, Stored::Int(serial), Some(Scope::Global));
  }
}

/// The raw step current now, as recorded ([`note_raw_refstep`]).
fn recorded_raw_refstep() -> Option<(crate::Digested, String, i64)> {
  let Some(Stored::Digested(tags)) = lookup_value(RAW_REFSTEP_TAGS) else {
    return None;
  };
  let serial = match lookup_value(RAW_REFSTEP_SERIAL) {
    Some(Stored::Int(serial)) => serial,
    _ => 0,
  };
  Some((tags, lookup_string(RAW_REFSTEP_COUNTER), serial))
}

/// Forgets the raw step locally, keeping it for [`renote_forgotten_raw_refstep`]: a listing starts, where listings
/// makes `\@currentlabel` its line number's (lstmisc.sty:1228 `\lst@Init`, and `\refstepcounter{lstnumber}` per line,
/// :1225), so a `\label` in an unnumbered listing does not name a step made before it (a `\refstepcounter{problem}`
/// then an `lstlisting` with `(*\label{ln1}*)`: pdflatex the line's 2; repro
/// `cross-references/listing_label_names_no_step_before_it.tex`).
pub fn forget_raw_refstep() {
  let forgotten = recorded_raw_refstep();
  assign_value(
    RAW_REFSTEP_FORGOTTEN,
    forgotten.is_some(),
    Some(Scope::Local),
  );
  if let Some((tags, counter, serial)) = forgotten {
    assign_value(
      &s!("{RAW_REFSTEP_FORGOTTEN}_tags"),
      Stored::Digested(tags),
      Some(Scope::Local),
    );
    assign_value(
      &s!("{RAW_REFSTEP_FORGOTTEN}_counter"),
      counter,
      Some(Scope::Local),
    );
    assign_value(
      &s!("{RAW_REFSTEP_FORGOTTEN}_serial"),
      Stored::Int(serial),
      Some(Scope::Local),
    );
    assign_value(RAW_REFSTEP_TAGS, Stored::None, Some(Scope::Local));
  }
}

/// Notes again, locally, the step [`forget_raw_refstep`] forgot where the listing started: the box a tcolorbox
/// listing box is, whose phantom code (its `label`) runs before the listing in the box (tcolorbox.sty:1154-1161,
/// :1925) while this port runs it in the listing's first line (`listings_sty::with_tcb_phantom`): its label names
/// the box's step (`\newtcblisting` with `auto counter`, 2609.20563).
pub fn renote_forgotten_raw_refstep() {
  if !matches!(
    lookup_value(RAW_REFSTEP_FORGOTTEN),
    Some(Stored::Bool(true))
  ) {
    return;
  }
  if let Some(tags @ Stored::Digested(_)) = lookup_value(&s!("{RAW_REFSTEP_FORGOTTEN}_tags")) {
    assign_value(RAW_REFSTEP_TAGS, tags, Some(Scope::Local));
    let counter = lookup_string(&s!("{RAW_REFSTEP_FORGOTTEN}_counter"));
    assign_value(RAW_REFSTEP_COUNTER, counter, Some(Scope::Local));
    if let Some(serial) = lookup_value(&s!("{RAW_REFSTEP_FORGOTTEN}_serial")) {
      assign_value(RAW_REFSTEP_SERIAL, serial, Some(Scope::Local));
    }
  }
}

/// A label the display's own `\label` makes ([`mark_display_label`]) is pending until the kernel's `\label` takes it.
const RAW_REFSTEP_DISPLAY_LABEL: &str = "raw_refstep_display_label";

/// An amsmath display starts (`{gather}`, `{align}`, `{multline}`, `{aligned}` … and their starred forms): the labels
/// its own `\label` makes ([`mark_display_label`]) name the step held here, not one made in a row. amsmath's `\label`
/// there is `\label@in@display` (amsmath.sty:1230-1236), which only stores the key; `\make@display@tag` makes the label
/// in the row's tag cell (:1240-1252, :1621-1627 `{gather}`, every row), after the cell group a `\refstepcounter` was
/// made in restored `\@currentlabel` — `\begin{gather*} u=v \refstepcounter{foo}\label{e6} \end{gather*}` is the
/// section's 7 in pdflatex. `names_the_step`: the display's labels may name the step current here (`{gather}`, and an
/// `{aligned}`/`{split}` in an `{equation*}`: pdflatex 44, 45). Not for the align family (`{align}`, `{flalign}`,
/// `{alignat}`) and `{multline}`, starred or not: `\math@cr@@@align` makes the display tag only under `\iftag@`
/// (:1833-1843), so an untagged row's label is the next tagged row's (`\df@label` carried) or none (pdflatex `??`),
/// never a step held at the start (repro `cross-references/amsmath_row_label_names_the_display_step.tex`, review probe
/// rev78g2 al2.tex: `\begin{align} a&=b \label{n2}\notag \\ c&=d \end{align}` in a `\refstepcounter` environment is
/// equation 1). The kernel's `\label` a document saved before (`\let\originallabel\label` at `\begin{document}`, the
/// `\labelrel` idiom of 2310.19138) labels at once, after the cell's step ("a"). Local to the display; an inner one
/// (`{aligned}` in `{gather*}`) keeps the outer's. The outermost start forgets a display label announced and never made
/// ([`mark_display_label`]).
pub fn hold_raw_refstep_for_display(names_the_step: bool) {
  if matches!(lookup_value(RAW_REFSTEP_DISPLAY), Some(Stored::Bool(true))) {
    return;
  }
  end_display_label();
  if let Some(step) = current_raw_refstep_now().filter(|_| names_the_step) {
    assign_value(
      &s!("{RAW_REFSTEP_DISPLAY}_tags"),
      Stored::Digested(step.tags),
      Some(Scope::Local),
    );
    assign_value(
      &s!("{RAW_REFSTEP_DISPLAY}_serial"),
      Stored::Int(step.serial),
      Some(Scope::Local),
    );
  } else {
    assign_value(
      &s!("{RAW_REFSTEP_DISPLAY}_tags"),
      Stored::None,
      Some(Scope::Local),
    );
  }
  assign_value(RAW_REFSTEP_DISPLAY, Stored::Bool(true), Some(Scope::Local));
}

/// A paragraph box (`{minipage}`, `\parbox`, the kernel's `\@arrayparboxrestore`) starts: amsmath's
/// `\@arrayparboxrestore` restores `\label` to the kernel's (amsmath.sty:1237-1238 `\let\label\ltx@label`), so a label
/// in the box is made at once and names the step current in it, not the display's held step (review probe rev78g2
/// nest.tex z5, `\begin{gather*} a=b \refstepcounter{foo}\text{\begin{minipage}{2cm}\label{z5}x\end{minipage}}
/// \end{gather*}`: pdflatex 42; repro `cross-references/amsmath_row_label_names_the_display_step.tex`). Local to the
/// box. Outside an amsmath display nothing.
pub fn leave_display_for_parbox() {
  if !matches!(lookup_value(RAW_REFSTEP_DISPLAY), Some(Stored::Bool(true))) {
    return;
  }
  assign_value(RAW_REFSTEP_DISPLAY, Stored::Bool(false), Some(Scope::Local));
  let display_label = matches!(
    (lookup_meaning(&T_CS!("\\label")), lookup_meaning(&T_CS!("\\lx@ams@display@label"))),
    (Some(Stored::Expandable(a)), Some(Stored::Expandable(b))) if std::rc::Rc::ptr_eq(&a, &b)
  );
  if display_label && lookup_meaning(&T_CS!("\\ltx@label")).is_some() {
    let_i(&T_CS!("\\label"), &T_CS!("\\ltx@label"), None);
  }
}

/// The display's own `\label` is making a label (amsmath's `\label@in@display`, expanded): the kernel's `\label` it
/// reaches (directly, through cleveref's, or after a row's `\noalign` deferral) names the step current where the
/// display started ([`hold_raw_refstep_for_display`]), until [`end_display_label`].
pub fn mark_display_label() { assign_value(RAW_REFSTEP_DISPLAY_LABEL, true, Some(Scope::Global)); }

/// The kernel's `\label` made the label [`mark_display_label`] announced.
pub fn end_display_label() {
  if lookup_bool(RAW_REFSTEP_DISPLAY_LABEL) {
    assign_value(RAW_REFSTEP_DISPLAY_LABEL, false, Some(Scope::Global));
  }
}

/// The raw step ([`note_raw_refstep`]) a `\label` made now names: `\@currentlabel` is still the step's
/// `\the<counter>`, not text a package defined since (`\protected@edef\@currentlabel{…}`); for an amsmath display's
/// own `\label` ([`mark_display_label`]) the step current where the display started ([`hold_raw_refstep_for_display`]).
/// Witnesses: 2506.01072 (`\newtcbtheorem`), 2505.20088 (`\newtcolorbox[auto counter]`), 2402.06608 (`[blend into=
/// figures]`), 2609.20563 (`\newtcblisting`), 2310.19138 (`\labelrel` in `{align*}`).
pub fn current_raw_refstep() -> Option<RawRefstep> {
  if !(matches!(lookup_value(RAW_REFSTEP_DISPLAY), Some(Stored::Bool(true)))
    && lookup_bool(RAW_REFSTEP_DISPLAY_LABEL))
  {
    return current_raw_refstep_now();
  }
  let Some(Stored::Digested(tags)) = lookup_value(&s!("{RAW_REFSTEP_DISPLAY}_tags")) else {
    return None;
  };
  let serial = match lookup_value(&s!("{RAW_REFSTEP_DISPLAY}_serial")) {
    Some(Stored::Int(serial)) => serial,
    _ => 0,
  };
  Some(RawRefstep { tags, serial })
}

/// The raw step a `\label`'s whatsit recorded ([`current_raw_refstep`], its `raw_refstep_tags` and
/// `raw_refstep_serial` properties).
pub fn raw_refstep_of(props: &HashMap<Stored>) -> Option<RawRefstep> {
  let Some(Stored::Digested(tags)) = props.get("raw_refstep_tags") else {
    return None;
  };
  let serial = match props.get("raw_refstep_serial") {
    Some(Stored::Int(serial)) => *serial,
    _ => 0,
  };
  Some(RawRefstep { tags: tags.clone(), serial })
}

/// [`current_raw_refstep`] outside an amsmath display.
fn current_raw_refstep_now() -> Option<RawRefstep> {
  let (tags, counter, serial) = recorded_raw_refstep()?;
  let the_counter = Tokens!(T_CS!(s!("\\the{}", counter)));
  let names_the_step = match lookup_meaning(&T_CS!("\\@currentlabel")) {
    Some(Stored::Expandable(label)) => {
      matches!(label.get_expansion(), Some(ExpansionBody::Tokens(body)) if *body == the_counter)
    },
    _ => false,
  };
  names_the_step.then_some(RawRefstep { tags, serial })
}

/// `\refstepcounter{<ctype>}` through what `\refstepcounter` means now
/// (`ref_step_counter_planned`).
pub fn ref_step_counter_by_meaning(ctype: &str) -> Result<HashMap<Stored>> {
  ref_step_counter_planned(ctype, Vec::new(), false)
}

/// A counter step as LaTeX's caption, equation, item and section steps are: `\refstepcounter`
/// with its current meaning (latex.ltx:17391-17399 `\refstepcounter\@captype`, :15737-15738), so a
/// rebinding applies — floatrow measures each floatbox under `\FR@loc@`, whose `\refstepcounter`
/// is a local `\advance` undone with the box (floatrow.sty:543-556); hyperref's and cleveref's wrap
/// the kernel's; crossreference.sty and old varioref copy it. As caption does
/// (caption.sty:551-570): `before` (its prepare hook) is digested first, and a continued float's
/// step suppresses the counter's own `\stepcounter` for the length of the `\refstepcounter`
/// (`\caption@@refcounter`), so the number repeats without resets. With the kernel meaning this is
/// direct; otherwise `\refstepcounter{<counter>}` is digested — the counter, as LaTeX passes it
/// (cleveref's `\cref@constructprefix{#1}`, fncylab's `\stepcounter{#1}` read it) — with `ctype`
/// left for the kernel's label to make the tags of (a shared-counter theorem's own type); the props
/// are the ones it recorded, else (a rebinding that never reaches the kernel) the label of the
/// counter's value as it now stands. Perl steps the counter directly (`RefStepCounter`; KPE #412).
/// Witness kaytannollista-latexia (floatrow), 2605.17685 (continued floats).
/// Guards `perfect_kernel_batch58::{caption_steps_through_refstepcounter,
/// floatrow_floatbox_steps_its_counter_once, floatbox_steps_its_equation_once,
/// steps_keep_their_type_and_continuation_through_a_wrapper}`.
pub fn ref_step_counter_planned(
  ctype: &str,
  before: Vec<Token>,
  continued: bool,
) -> Result<HashMap<Stored>> {
  let ctype = strip_counter_type_sentinel(ctype);
  let ctr = counter_for_type(ctype);
  if !before.is_empty() {
    stomach::digest(Tokens::new(before))?;
  }
  let refstep = T_CS!("\\refstepcounter");
  if x_equals(&refstep, &T_CS!("\\lx@kernel@refstepcounter")) {
    if continued {
      // The suppressed `\stepcounter`: the value stays, no counter within is reset, and the
      // continuation is used up (caption's next step clears its flags, caption.sty:590-599).
      assign_value("lx@float@continued", String::new(), Some(Scope::Global));
      add_to_counter(&ctr, Number::new(-1))?;
      return ref_step_counter(ctype, true);
    }
    return ref_step_counter(ctype, false);
  }
  PENDING_STEPS.with(|steps| steps.borrow_mut().insert(ctr.clone(), ctype.to_string()));
  REFSTEP_PROPS.with(|slot| slot.borrow_mut().remove(&ctr));
  let mut tokens = Vec::new();
  if continued {
    // caption.sty:557-568 `\caption@@refcounter`.
    tokens.extend([
      T_CS!("\\let"),
      T_CS!("\\lx@caption@stepcounter@ORI"),
      T_CS!("\\stepcounter"),
      T_CS!("\\def"),
      T_CS!("\\stepcounter"),
      T_PARAM!(),
      T_OTHER!("1"),
      T_BEGIN!(),
      T_CS!("\\lx@caption@stepcounter@unless"),
      T_BEGIN!(),
    ]);
    tokens.extend(Explode!(&ctr));
    tokens.extend([
      T_END!(),
      T_BEGIN!(),
      T_PARAM!(),
      T_OTHER!("1"),
      T_END!(),
      T_END!(),
    ]);
  }
  tokens.extend([refstep, T_BEGIN!()]);
  tokens.extend(Explode!(&ctr));
  tokens.push(T_END!());
  if continued {
    tokens.extend([
      T_CS!("\\let"),
      T_CS!("\\stepcounter"),
      T_CS!("\\lx@caption@stepcounter@ORI"),
    ]);
  }
  let digested = stomach::digest(Tokens::new(tokens));
  take_pending_step(&ctr);
  digested?;
  match REFSTEP_PROPS.with(|slot| slot.borrow_mut().remove(&ctr)) {
    Some(props) => Ok(props),
    None => label_stepped_counter(ctype, &ctr),
  }
}

/// Does the `\the<start>@ID` formatter chain reach `\the<target>@ID`?
/// `begin_itemize` defines each list's `\the<usecounter>@ID` through its
/// own `\the<listcounter>@ID`, which is defined through the enclosing
/// list's `\the<outerusecounter>@ID` — so the open lists form a chain of
/// `\the…@ID` macros. Walking the bodies (never expanding) is bounded by
/// the nesting depth.
pub fn list_id_chain_reaches(start: &str, target: &str) -> Result<bool> {
  let target_cs = T_CS!(s!("\\the{target}@ID"));
  let mut frontier = vec![T_CS!(s!("\\the{start}@ID"))];
  let mut seen: Vec<Token> = Vec::new();
  while let Some(cs) = frontier.pop() {
    if cs == target_cs {
      return Ok(true);
    }
    if seen.contains(&cs) || seen.len() > 64 {
      continue;
    }
    seen.push(cs);
    if let Some(defn) = lookup_definition(&cs)?
      && defn.is_expandable()
      && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
    {
      for t in body.unlist_ref() {
        if t.get_catcode() == Catcode::CS
          && t.with_cs_name(|n| n.starts_with("\\the") && n.ends_with("@ID"))
        {
          frontier.push(*t);
        }
      }
    }
  }
  Ok(false)
}

/// The counter that `\refstepcounter{<ctype>}` steps: the `counter_for_type` mapping (theorem
/// types sharing a counter), else `<ctype>` itself.
pub fn counter_for_type(ctype: &str) -> String {
  with_mapping("counter_for_type", ctype, |meaning| match meaning {
    Some(Stored::String(ctr)) => arena::to_string(*ctr),
    _ => ctype.to_string(),
  })
}

/// Records `alias` as an alias counter of `base`, as aliascnt.sty's `\newaliascnt{alias}{base}` makes it
/// (aliascnt_sty.rs; llncs.cls v2.16+'s `\@spothm`, llncs_cls.rs): `\refstepcounter{alias}` steps `base`'s root
/// counter (`counter_for_type`, following a chain of aliases), so the alias shares its number and its ids, while the
/// type stays `alias` — the counter LaTeX's `\refstepcounter` is passed, whose name hyperref gives the target
/// (`\<alias>autorefname`, `\<alias>name`; [`is_counter_alias`]). Witnesses 2605.20695 (aliascnt lemmas), 2202.10090
/// (llncs, envcountsame).
pub fn register_counter_alias(alias: &str, base: &str) {
  let root = counter_for_type(base);
  assign_mapping("counter_for_type", alias, Some(root.clone()));
  assign_mapping("counter_alias", alias, Some(root));
}

/// Is `ctype` an alias counter ([`register_counter_alias`])?
pub fn is_counter_alias(ctype: &str) -> bool {
  with_mapping("counter_alias", ctype, |meaning| meaning.is_some())
}

/// `<ctype>` without a trailing sentinel control sequence the parameter reader may have pulled in.
pub fn strip_counter_type_sentinel(ctype: &str) -> &str {
  // Defensive: under some upstream conditions the {} parameter reader pulls a
  // trailing `\par` (or similar trailing CS) into a counter-type identifier
  // before it reaches us (Cluster A: math0010095, hep-ph0204075). Strip it
  // here so the downstream `\csname @<ctype>...@ID\endcsname` and similar
  // constructions stay well-formed. We strip the same well-known sentinels
  // as latex_constructs::strip_trailing_cs.
  for tail in ["\\par", "\\@startsection@hook", "\\relax"] {
    if let Some(stripped) = ctype.strip_suffix(tail) {
      return stripped;
    }
  }
  ctype
}

/// The serial of the counter step in scope ([`label_stepped_counter`]; local).
const CURRENT_REFSTEP_SERIAL: &str = "current_refstep_serial";
/// The serial of the latest counter step made ([`label_stepped_counter`]; global).
const LATEST_REFSTEP_SERIAL: &str = "latest_refstep_serial";

/// Whether a `\label` made now follows a counter step a group has since undone: LaTeX's `\label` names
/// `\@currentlabel` (latex.ltx:14963-14967), which reverts when the group of the step that set it closes, so a label
/// after `\end{equation}` inside a theorem names the theorem, not the equation (math0107195, "Theorem 5.3"). Such a
/// label never names the object that closed step numbered, however near it is (`Document::float_to_label_scoped`).
pub fn label_follows_an_undone_step() -> bool {
  match (
    lookup_value(CURRENT_REFSTEP_SERIAL),
    lookup_value(LATEST_REFSTEP_SERIAL),
  ) {
    (Some(Stored::Int(current)), Some(Stored::Int(latest))) => current != latest,
    (None, Some(Stored::Int(_))) => true,
    _ => false,
  }
}

/// The latest counter step stays the one in scope although the group it was made in has closed: a wrapper group of
/// LaTeXML's own (caption's `\captionof` set in a float, caption_sty.rs `\@captionof@`) around a step LaTeX makes in
/// the current group (caption.sty:390-391 `\setcaptiontype*` then `\caption`), so a `\label` after it names the step's
/// object (2402.05535 `\captionof{figure}{…}\fi\label{…}`).
pub fn keep_latest_refstep_in_scope() {
  if let Some(latest @ Stored::Int(_)) = lookup_value(LATEST_REFSTEP_SERIAL) {
    assign_value(CURRENT_REFSTEP_SERIAL, latest, Some(Scope::Local));
  }
}

/// The labelling half of `\refstepcounter` (latex.ltx:14956-14967, after its `\stepcounter`):
/// `\@currentlabel`, `\@currentID`, the tags and the counter's scope, for the counter `ctr` just
/// stepped for `ctype`. Returns the `tags` and `id` properties.
pub fn label_stepped_counter(ctype: &str, ctr: &str) -> Result<HashMap<Stored>> {
  let ctr = ctr.to_string();
  // A step is the current label's now (`note_raw_refstep` records it again for a raw one); global, as this port's
  // `\@currentlabel` is.
  if matches!(lookup_value(RAW_REFSTEP_TAGS), Some(Stored::Digested(_))) {
    assign_value(RAW_REFSTEP_TAGS, Stored::None, Some(Scope::Global));
  }
  maybe_preempt_refnum(&ctr, false);

  let the_ctr_id = s!("\\the{ctr}@ID");
  let the_ctr = s!("\\the{ctr}");

  let has_id: bool = match lookup_definition(&T_CS!(&the_ctr_id))? {
    Some(iddef) => {
      if let Some(params) = iddef.get_parameters() {
        params.get_num_args() == 0
      } else {
        true
      }
    },
    _ => false,
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

  let digest_label = || -> Result<(crate::Digested, crate::Digested)> {
    let refnum = digest_text(Tokens!(T_CS!(&the_ctr)))?;
    let invocation = build_invocation(T_CS!("\\lx@make@tags"), vec![Some(Tokens!(T_OTHER!(
      ctype
    )))])?;
    Ok((refnum, stomach::digest(invocation)?))
  };
  // TeX keeps the label unexpanded (`\@currentlabel`, latex.ltx:14963-14967) and typesets it only where `\ref`
  // prints it; Perl digests it as text here (Package.pm:770 `DigestText`). A step made in math whose label text
  // cannot set (`\renewcommand{\theconsta}{{\kappa_{\arabic{consta}}}}`, `$\refstepcounter{consta}…$`: three "Script _
  // can only appear in math mode" per step, 1609.03070, 1904.00290) is the author's math: its digest is discarded and
  // the label set as `\ensuremath{\the<ctr>}`, local to the label (OXIDIZED_DESIGN #748; guard
  // `perfect_kernel_batch73b::refstepcounter_in_math_sets_a_math_label`). A label text sets keeps Perl's digest.
  let (refnum, tags) = if lookup_bool("IN_MATH") {
    let hold = crate::util::logger::DiagnosticsHold::begin();
    match digest_label() {
      Ok(label) if hold.errors_raised() == 0 => {
        hold.commit();
        label
      },
      Err(err) => {
        hold.commit();
        return Err(err);
      },
      Ok(_) => {
        hold.discard();
        push_frame();
        let saved = T_CS!(s!("\\lx@the@in@math@{ctr}"));
        let math_label = Tokens::new(vec![T_CS!("\\ensuremath"), T_BEGIN!(), saved, T_END!()]);
        // (a label whose digest left a group open — a `\p@equation` taking nine arguments ate the closing braces,
        // 1902.09345 — keeps the frame this pushed, and with it the math label: saved again, the label would call
        // itself, `\@ensuremath` for ever)
        let label = if matches!(lookup_meaning(&the_ctr_cs), Some(Stored::Expandable(ref d))
          if matches!(d.get_expansion(), Some(ExpansionBody::Tokens(body)) if *body == math_label))
        {
          digest_label()
        } else {
          let_i(&saved, &the_ctr_cs, Some(Scope::Local));
          def_macro(
            the_ctr_cs,
            None,
            math_label,
            Some(ExpandableOptions {
              scope: Some(Scope::Local),
              ..ExpandableOptions::default()
            }),
          )
          .and_then(|()| digest_label())
        };
        pop_frame()?;
        label?
      },
    }
  } else {
    digest_label()?
  };

  // Any scopes activated for previous value of this counter (& any nested counters) must be
  // removed. This may also include scopes activated for \label
  deactivate_counter_scope(arena::pin(&ctr));

  // And install the scope (if any) for this reference number.
  assign_value("current_counter", ctr.clone(), Some(Scope::Local));
  // The step's serial: in scope (local, as LaTeX's `\@currentlabel` is, latex.ltx:14963) and the latest one made
  // (global). A `\label` whose scope's step is not the latest comes after a step a group has since undone
  // ([`label_follows_an_undone_step`]).
  let serial = match lookup_value(LATEST_REFSTEP_SERIAL) {
    Some(Stored::Int(serial)) => serial + 1,
    _ => 1,
  };
  assign_value(
    CURRENT_REFSTEP_SERIAL,
    Stored::Int(serial),
    Some(Scope::Local),
  );
  assign_value(
    LATEST_REFSTEP_SERIAL,
    Stored::Int(serial),
    Some(Scope::Global),
  );

  let scope = arena::pin(format!("{ctr}:{refnum}"));
  let mut receiver = VecDeque::new();
  receiver.push_front(Stored::String(scope));
  assign_value(
    &s!("scopes_for_counter:{ctr}"),
    receiver,
    Some(Scope::Local),
  );
  activate_scope(scope);

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
  if let Some(mapper) = get_label_mapping_hook() {
    let hj_refnum = T_CS!(s!("\\_PREEMPTED_REFNUM_{ctr}"));
    let hj_id = T_CS!(s!("\\_PREEMPTED_ID_{ctr}"));
    // First, restore the \the<ctr> and \the<ctr>@ID macros to defaults
    if !norefnum && has_meaning(&hj_refnum) {
      let_i(&T_CS!(s!("\\the{ctr}")), &hj_refnum, Some(Scope::Global));
    }
    if has_meaning(&hj_id) {
      let_i(&T_CS!(s!("\\the{ctr}@ID")), &hj_id, Some(Scope::Global));
    }
    let label = lookup_string("PEEKED_LABEL");
    let (fixedrefnum, fixedid) = mapper(&label, ctr, norefnum);
    if let Some(refnum) = fixedrefnum
      && !norefnum
    {
      if !has_meaning(&hj_refnum) {
        // Save for later
        let_i(&hj_refnum, &T_CS!(s!("\\the{ctr}")), Some(Scope::Global));
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
    if let Some(id) = fixedid {
      if !has_meaning(&hj_id) {
        // Save for later
        let_i(&hj_id, &T_CS!(s!("\\the{ctr}@ID")), Some(Scope::Global));
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
    remove_value("PEEKED_LABEL"); // CONSUME the label
    assign_value(
      "PROCESSED_LABEL",
      Stored::String(arena::pin(label)),
      Some(Scope::Global),
    );
  }
}

/// Use to peek for FOLLOWING \label{...} to support label-derived reference numbers
/// (Perl: MaybePeekLabel)
pub fn maybe_peek_label() -> Result<()> {
  if get_label_mapping_hook().is_some() {
    let peek = crate::gullet::read_non_space()?;
    if let Some(ref token) = peek {
      if x_equals(token, &T_CS!("\\label")) {
        begin_semiverbatim(None);
        let arg = crate::gullet::read_arg(crate::gullet::ExpansionLevel::Off)?;
        end_semiverbatim()?;
        let arg_str = arg.to_string();
        let label = clean_label(&arg_str, Some("")).into_owned();
        assign_value(
          "PEEKED_LABEL",
          Stored::String(arena::pin(&label)),
          Some(Scope::Global),
        );
        // Put back the arg wrapped in braces so \label can re-read it
        crate::gullet::unread(Tokens!(T_BEGIN!(), arg, T_END!()));
      } else {
        remove_value("PROCESSED_LABEL");
        remove_value("PEEKED_LABEL");
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
  if get_label_mapping_hook().is_some() {
    let label = clean_label(label, Some(""));
    let processed = lookup_string("PROCESSED_LABEL");
    if processed.is_empty() || processed != label {
      // Only if not already processed
      remove_value("PROCESSED_LABEL");
      assign_value(
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
  let scope_syms: Vec<SymStr> = with_value(&scopes_for_counter, |v| match v {
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
    deactivate_scope(scope);
  }

  // TODO: if we ever want to unshift from the nested_counters, we'll need to also use
  // Stored::VecDequeStored for them.
  let inner_ctrs: Vec<SymStr> = with_value(&nested_counters, |v| match v {
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
  let unctr_defined = lookup_register(&unctr_cmd, Vec::new())
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
  let exists = |cs: &str| lookup_register(cs, Vec::new()).ok().flatten().is_some();
  // A reset list names each child and its `UN` companion (Package.pm:674); a
  // companion never allocated (below) has nothing to reset.
  if ctr.with_str(|cstr| cstr.starts_with("UN")) && !exists(&c_ctr) {
    return Ok(());
  }
  assign_register(
    &c_ctr,
    Number::new(0).into(),
    Some(Scope::Global),
    Vec::new(),
  )?;
  // but not UN — and only a companion that exists: `\c@UN<ctr>` is LaTeXML's
  // (`new_counter`), so a raw `\newcount\c@CodelineNo` (doc.sty:870) in a reset
  // list (l3doc.cls:463 `\@addtoreset{CodelineNo}{part}`) has none, and
  // assigning it warned "not a register" at every `\part` (ltx-talk-code, 20).
  // Perl `ResetCounter` (Package.pm:886) assigns unconditionally and warns
  // too; the same existence test guards `ref_step_id` above (Package.pm:863).
  // OXIDIZED_DESIGN #268.
  if !ctr.with_str(|cstr| cstr.starts_with("UN")) && exists(&c_un_ctr) {
    assign_register(
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
  if let Some(nested) = lookup_tokens(&s!("\\cl@{ctr}")) {
    for c in nested.unlist() {
      reset_counter(&c)?;
    }
  }
  Ok(())
}

/// The counter of the last numbered `\item` in force, whose label an `\item[…]` references ([`ref_step_item_counter`]).
const ITEM_LABEL_COUNTER: &str = "lx@item@label@counter";
/// The [`ITEM_LABEL_COUNTER`] the list's first numbered `\item` found in force: an enclosing list's counter, empty in
/// a top-level list ([`ref_step_item_counter`]).
const ITEM_LABEL_ENCLOSING: &str = "lx@item@label@enclosing";

/// Whether the macro `cs` expands to something: defined, and not a parameterless empty macro. An empty body is
/// stored as no expansion at all (`Expandable::new` "treat empty tokens as None"), so both forms are empty: the
/// itemize's `\the@item<n>` (sect06.rs) and enumitem's emptied `\p@enum<n>` (enumitem_sty.rs).
fn prints_something(cs: &str) -> Result<bool> {
  Ok(match lookup_definition(&T_CS!(cs))? {
    Some(defn) => {
      !(defn.is_expandable()
        && defn.get_conditional_type().is_none()
        && defn
          .get_parameters()
          .is_none_or(|params| params.get_num_args() == 0)
        && match defn.get_expansion() {
          None => true,
          Some(ExpansionBody::Tokens(body)) => body.is_empty(),
          Some(ExpansionBody::Closure(_)) => false,
        })
    },
    None => false,
  })
}

/// Create id, and tags for an itemize type \item
pub fn ref_step_item_counter(tag_opt: Option<&Tokens>) -> Result<HashMap<Stored>> {
  let counter = lookup_string("itemcounter");
  let n = lookup_int("itemization_items");
  assign_value("itemization_items", n + 1, None);
  let mut attr: HashMap<Stored> = HashMap::default();
  if n > 0
    && let Some(sep) = lookup_dimension("\\itemsep")
  {
    let default_opt = lookup_dimension("\\lx@default@itemsep");
    if default_opt.is_none() || sep.value_of() != default_opt.unwrap().value_of() {
      attr.insert("itemsep", sep.into());
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

    let mut tag_tokens = vec![T_BEGIN!()];
    // An `\item[…]` does not `\refstepcounter` (latex.ltx `\@item`: only `\if@noitemarg`), so a `\label` on it takes
    // `\@currentlabel` as the last numbered item in force left it. Perl types `\p@<counter>` instead
    // (latex_constructs.pool.ltxml:1383 `\let\the<counter>\@empty`), the enclosing item's label only while `\p@enumii`
    // is the kernel's `\theenumi`: enumitem empties it below the first level (enumitem.sty:551-556), and the reference
    // was lost (2603.02347, 7 items; covington's `\item[b.]` in an `{example}`, 2405.01972). So when the last numbered
    // item in force is an enclosing list's (`ITEM_LABEL_COUNTER` names another counter), the refnum is that item's
    // `\p@<ctr>\the<ctr>`, read before `\the<counter>` is emptied. After a numbered item of the same list LaTeX's
    // label is that sibling's. Perl's form keeps the enclosing item's label (`\p@enumii` = `\theenumi`, "1" where
    // LaTeX has "1a"), and nothing at all once enumitem has emptied `\p@<ctr>`: there a nested list's refnum is the
    // sibling's `\p@<ctr>\the<ctr>`, LaTeX's `\@currentlabel` ("a"). A top-level list keeps Perl's form (no refnum,
    // the reference reads the item's own label): its `\p@enumi` is empty too, but its authors replace the sibling's
    // label with their own (`\def\@currentlabel{(P2')}\label{…}`, 2502.11630; 2409.02896), and `\@currentlabel` is
    // not read here; so is a top-level `\item[…]` with no numbered item in force, where LaTeX's label is the last
    // section's or equation's. A nested list's author can override it too, and there the sibling's label wins
    // (2609.14520: "ii)" for the author's "ii′)"); reading an author-set `\@currentlabel` is deferred (user ruling
    // 2026-10-20, SYNC_STATUS 77u residual). KNOWN_PERL_ERRORS #1003; guards `perfect_kernel_batch77u::{
    // item_with_its_own_label_takes_the_enclosing_label, enumitem_item_with_its_own_label_takes_the_enclosing_label}`.
    let ctr = counter_for_type(&counter);
    let labelled = lookup_string(ITEM_LABEL_COUNTER);
    let sibling_label = labelled == ctr
      && !lookup_string(ITEM_LABEL_ENCLOSING).is_empty()
      && !prints_something(&s!("\\p@{ctr}"))?;
    if !labelled.is_empty() && (labelled != ctr || sibling_label) {
      for (alias, cs) in [
        ("\\lx@item@label@p@", s!("p@{labelled}")),
        ("\\lx@item@label@the@", s!("the{labelled}")),
      ] {
        tag_tokens.extend([
          T_CS!("\\expandafter"),
          T_CS!("\\let"),
          T_CS!("\\expandafter"),
          T_CS!(alias),
          T_CS!("\\csname"),
        ]);
        tag_tokens.extend(Explode!(cs));
        tag_tokens.push(T_CS!("\\endcsname"));
      }
      tag_tokens.extend([
        T_CS!("\\def"),
        T_CS!(s!("\\p@{ctr}")),
        T_BEGIN!(),
        T_CS!("\\lx@item@label@p@"),
        T_CS!("\\lx@item@label@the@"),
        T_END!(),
      ]);
    }
    tag_tokens.extend([
      T_CS!("\\let"),
      T_CS!(s!("\\the{counter}")),
      T_CS!("\\@empty"),
      T_CS!("\\def"),
      T_CS!(s!("\\fnum@{counter}")),
      T_BEGIN!(),
      formatter,
      T_BEGIN!(),
    ]);
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
    let props = ref_step_counter_by_meaning(&counter)?;
    // The item set `\@currentlabel` (latex.ltx `\refstepcounter` in `\@item`) when its list is numbered — its
    // `\the<counter>` prints something; an itemize's `\the@item<n>` and a description's are empty (sect06.rs), as
    // LaTeX's `\@nmbrlistfalse` lists never step, so an `\item[…]` after an itemize sibling still takes the enclosing
    // enumerate item's label ("1"). Local to the list, as the `\protected@edef` is; the first numbered item records
    // the enclosing list's counter it replaces.
    let ctr = counter_for_type(&counter);
    if prints_something(&s!("\\the{ctr}"))? {
      let in_force = lookup_string(ITEM_LABEL_COUNTER);
      if in_force != ctr {
        assign_value(ITEM_LABEL_ENCLOSING, in_force, None);
      }
      assign_value(ITEM_LABEL_COUNTER, ctr, None);
    }
    props
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
  pub nolevel:       bool,
  /// enumitem series
  pub series:        Option<Tokens>,
  /// start at a custom value
  pub start:         Option<Number>,
  /// enumitem resume?
  pub resume:        Option<String>,
  /// enumitem resume* ?
  pub resume_star:   Option<String>,
  /// a list that opens no `\list` (paralist's `in…` lists), so `\@listdepth` stays;
  /// enumitem's inline lists do open one (enumitem.sty:1188-1191)
  pub opens_no_list: bool,
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
  let outercounter = lookup_string("itemcounter");
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
  assign_value("itemization_level", listlevel, None);
  assign_value(&s!("{counter}level"), level, None);
  // latex.ltx's `\itemize`/`\enumerate` advance `\@itemdepth`/`\@enumdepth` inside their group
  // (as do enumitem's and paralist's inline forms; a `\newlist` list has its own
  // `\enitdp@<name>`), and every list but an in-paragraph one opens a `\list` that advances
  // `\@listdepth` globally until `\endlist` (latex.ltx:15852, :15913; enumitem.sty:1188-1191,
  // :1254 for its inline lists). Packages read them (enumitem's `list<depth>` keys,
  // enumitem.sty:977-978). The kernel `\list` binding keeps its own `\@listdepth`, and
  // `\trivlist` opens no `\list`. Perl's beginItemize (latex_constructs.pool.ltxml:1296) leaves all
  // three at 0 (SHARED).
  let depth_register = match itype {
    "itemize" | "inline@itemize" => Some("\\@itemdepth"),
    "enumerate" | "inline@enumerate" => Some("\\@enumdepth"),
    _ => None,
  };
  let register_depth = |cs: &str| match lookup_register_quiet(cs) {
    Some(RegisterValue::Number(n)) => n.0,
    _ => 0,
  };
  let mut item_depth = None;
  if let Some(cs) = depth_register {
    let depth = register_depth(cs) + 1;
    assign_register(cs, RegisterValue::Number(Number(depth)), None, vec![])?;
    if cs == "\\@itemdepth" && counter == "@item" {
      item_depth = Some(depth);
    }
  }
  if !options.opens_no_list && !matches!(itype, "list" | "trivlist") {
    let depth = register_depth("\\@listdepth");
    assign_register(
      "\\@listdepth",
      RegisterValue::Number(Number(depth + 1)),
      Some(Scope::Global),
      vec![],
    )?;
    let mut restore = vec![T_CS!("\\lx@listdepth@restore")];
    restore.extend(Explode!(&depth.to_string()));
    restore.push(T_CS!("\\relax"));
    push_value("afterGroup", Stored::Tokens(Tokens::new(restore)))?;
  }
  assign_value("itemization_items", 0, None);
  let listpostfix = roman!(listlevel).to_string();
  let postfix = roman!(level).to_string();
  let mut usecounter = counter.to_string();
  if !options.nolevel && !postfix.is_empty() {
    usecounter.push_str(&postfix);
  }
  // The `@item` levels past the six the kernel declares (sect06.rs) are declared as they are reached, in the same
  // id chain: lists nest that deep where a source opens trivlists it ends at once (doc.sty's `\@doc@env` opens one
  // per documented name; frankenstein, source2e).
  if counter == "@item" && level > 6 && lookup_definition(&T_CS!(s!("\\c@{usecounter}")))?.is_none()
  {
    let within = s!("@item{}", roman!(level - 1));
    new_counter(
      &usecounter,
      "",
      Some(NewCounterOptions {
        idprefix: "i",
        idwithin: &within,
        ..Default::default()
      }),
    )?;
    // As the kernel's six: an unnumbered item, named as an item (Perl pool:1514-1519, :1622).
    let global = || {
      Some(ExpandableOptions {
        scope: Some(Scope::Global),
        ..Default::default()
      })
    };
    def_macro(
      T_CS!(s!("\\the{usecounter}")),
      None,
      Tokens::new(Vec::new()),
      global(),
    )?;
    def_macro(
      T_CS!(s!("\\{usecounter}name")),
      None,
      Tokens!(T_CS!("\\itemtyperefname")),
      global(),
    )?;
  }
  if !itype.is_empty() {
    let itype_cs = T_CS!(s!("\\{itype}@item"));
    let_i(&T_CS!("\\item"), &itype_cs, None);
  }
  // In case within odd environment.
  let_i(&T_CS!("\\par"), &T_CS!("\\lx@normal@par"), None);
  def_macro(
    T_CS!("\\@listctr"),
    None,
    Tokens!(Explode!(usecounter)),
    None,
  )?;
  // latex.ltx:16068-16075 `\itemize` labels by `\labelitem\romannumeral\the\@itemdepth`, which only
  // itemizes advance; the `@item` level (the counter the item ids hang on) also counts kernel
  // `\list`s and `\trivlist`s (Perl pool:1639 `beginItemize('list')`), so an itemize inside a
  // `\list` took `\labelitemii` where pdflatex prints `\labelitemi`. The label follows the depth; the
  // counter, and so every id, stays Perl's. PERL-ORIGIN, pdflatex the oracle. A caller's own label
  // (enumitem's `label=`, paralist's `[<label>]` through `set_itemization_style`) is defined after
  // this. Repros: list-structure/{itemize_in_a_list_takes_the_first_label,
  // itemize_label_follows_the_itemize_depth}.
  if let Some(depth) = item_depth
    && depth != level
    && (1..=4).contains(&depth)
  {
    def_macro(
      T_CS!(s!("\\label{usecounter}")),
      None,
      Tokens!(T_CS!(s!("\\labelitem{}", roman!(depth)))),
      None,
    )?;
  }
  // latex.ltx:16057 `\enumerate` also does
  // `\edef\@enumctr{enum\romannumeral\the\@enumdepth}`; packages that reuse
  // the kernel enumerate read it back (bullenum.sty:58/61
  // `\csname the\@enumctr\endcsname`). `usecounter` is already
  // `enum` + roman(level), so this is the same value; enumerate-only, as in
  // the kernel. Perl's beginItemize (pool:1314) omits it (SHARED).
  // Witness: bullcntr/bullcntr-man (100-error cascade).
  if itype == "enumerate" {
    def_macro(
      T_CS!("\\@enumctr"),
      None,
      Tokens!(Explode!(usecounter)),
      None,
    )?;
  }
  // Now arrange that this list's id's are relative to the current (outer) item (if any)
  // And that the items within this list's id's are relative to this (new) list.
  assign_value("itemcounter", Stored::String(arena::pin(&usecounter)), None);
  // …and the environment this itemization is (natbib's `\bibitem` keeps its place only among this list's own `\item`s)
  assign_value(
    "itemization_environment",
    Stored::String(arena::pin(lookup_string("current_environment"))),
    None,
  );
  let listcounter = s!("@itemize{listpostfix}");
  if lookup_definition(&T_CS!(s!("\\c@{listcounter}")))?.is_none() {
    //Create new list counters as needed
    new_counter(&listcounter, "", None)?;
  }
  let mut shared = false;
  if !outercounter.is_empty() {
    // Make this list's ID relative to outer list's ID
    let outerusecounter = s!("{outercounter}{}", roman!(outerlevel).to_string());
    let thectr = s!("\\the{listcounter}@ID");
    // Explicit tokens, as `useexp` below: a counter name with a digit (`steps2i`) must stay one
    // control sequence (KPE #414).
    let mut expansion = vec![
      T_CS!(s!("\\the{outerusecounter}@ID")),
      T_OTHER!(".I"),
      T_CS!("\\arabic"),
      T_BEGIN!(),
    ];
    expansion.extend(Explode!(&listcounter));
    expansion.push(T_END!());
    // A list numbered by a counter an enclosing list's id hangs on — `\list{}{\usecounter{enumi}}`
    // around an `enumerate`, directly or through lists in between — redefines that counter's id
    // below through this list's, so the formatters would expand one another
    // (Fatal:Timeout:PushbackLimit; Perl loops too, latex_constructs.pool.ltxml:1323-1335). Then
    // the list hangs on the enclosing item, its id expanded once (the item's `\@<counter>@ID` is put
    // back when a list on the shared counter closes, below), and the shared counter's steps (this
    // list's own items) no longer reset the open lists' counters. Only inputs that looped meet the
    // condition. Guards `perfect_kernel_batch58::{
    // numbered_list_with_a_nested_enumerate_converts, list_numbered_by_an_enclosing_counter}`.
    shared = list_id_chain_reaches(&outerusecounter, &usecounter)?;
    if shared {
      let outer_id = crate::gullet::do_expand(Tokens!(expansion[0]))?;
      let mut frozen = outer_id.unlist();
      frozen.extend(expansion.drain(1..));
      expansion = frozen;
    }
    def_macro(T_CS!(thectr), None, Tokens::new(expansion), None)?;
    if shared {
      let open: Vec<Token> = (1..=listlevel)
        .map(|k| T_CS!(s!("@itemize{}", roman!(k))))
        .collect();
      let cl_use = s!("\\cl@{usecounter}");
      let kept = with_value(&cl_use, |v| match v {
        Some(Stored::Tokens(tks)) => Some(
          tks
            .clone()
            .unlist()
            .into_iter()
            .filter(|t| !open.contains(t))
            .collect::<Vec<_>>(),
        ),
        _ => None,
      });
      if let Some(kept) = kept {
        assign_value(
          &cl_use,
          Stored::Tokens(Tokens::new(kept)),
          Some(Scope::Global),
        );
      }
    }

    // AND reset this list's counter when the outer item is stepped
    let mut cl_toks = if shared && outerusecounter == usecounter {
      Vec::new()
    } else {
      vec![T_CS!(&listcounter)]
    };
    let cl_name = s!("\\cl@{outerusecounter}");
    let existing = with_value(&cl_name, |v| match v {
      Some(Stored::Tokens(tks)) => tks.clone().unlist(),
      _ => Vec::new(),
    });
    cl_toks.extend(existing);
    assign_value(
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
  // The shared counter's `\@<counter>@ID` is the enclosing item's id while this list is open
  // (`shared`, above): this list resets and steps it, so its value as the enclosing item had it is
  // put back, globally as the step set it, when the list closes — a later sibling list, or the
  // enclosing list's next items, hang on that item again.
  if shared {
    let at_id = T_CS!(s!("\\@{usecounter}@ID"));
    if let Some(defn) = lookup_definition(&at_id)?
      && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
    {
      let mut restore = vec![T_CS!("\\gdef"), at_id, T_BEGIN!()];
      restore.extend(body.clone().unlist());
      restore.push(T_END!());
      push_value("afterGroup", Stored::Tokens(Tokens::new(restore)))?;
    }
  }
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
  // Perl latex_constructs.pool L1354-1356: the list carries its TeX vertical
  // padding (\topsep + \parskip + \partopsep) as padtop/padbottom sizing
  // properties; Box::computeSizeStore adds them to the computed height/depth
  // (compute_size_and_cache here). Omitting them under-measured every list,
  // clipping tcolorbox frames drawn from the estimate (2605.02240).
  let pad = lookup_dimension_cs("\\topsep", false)
    .unwrap_or_default()
    .add(lookup_dimension_cs("\\parskip", false).unwrap_or_default())
    .add(lookup_dimension_cs("\\partopsep", false).unwrap_or_default());
  rsc.insert("padtop", Stored::Dimension(pad));
  rsc.insert("padbottom", Stored::Dimension(pad));
  Ok(rsc)
}

/// Set the itemization style for a given level.
/// Perl: setItemizationStyle($stuff, $level)
/// If $level is not given, uses the current itemize depth: paralist.sty:281-282 names the label
/// `labelitem\romannumeral\the\@itemdepth`, the register `begin_itemize` labels an itemize by. Perl
/// reads `@itemlevel`, which also counts kernel `\list`s (PERL-ORIGIN; repro
/// list-structure/itemize_label_follows_the_itemize_depth).
/// Defines \labelitem$level to $stuff.
pub fn set_itemization_style(stuff: Option<&Tokens>, level: Option<i32>) -> Result<()> {
  if let Some(stuff) = stuff {
    if stuff.is_empty() {
      return Ok(());
    }
    let level = level.unwrap_or_else(|| match lookup_register_quiet("\\@itemdepth") {
      Some(RegisterValue::Number(n)) => n.0.max(0) as i32,
      _ => 0,
    });
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
/// Perl: RescueCaptionCounters (latex_constructs.pool.ltxml L3203-3214)
pub fn rescue_caption_counters(captype: &str, whatsit: &mut Whatsit) {
  let tagskey = &s!("{captype}_tags");
  if let Some(tags) = remove_value(tagskey) {
    whatsit.set_property("tags", tags);
  }
  let idkey = s!("{captype}_id");
  if let Some(id) = remove_value(&idkey) {
    whatsit.set_property("id", id);
  }
  let inlistkey = s!("{captype}_inlist");
  if let Some(inlist) = remove_value(&inlistkey) {
    whatsit.set_property("inlist", inlist);
  }
}

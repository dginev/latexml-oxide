use std::borrow::{Borrow, Cow};
use std::sync::Arc;

use rtx_core::common::error::*;
use rtx_core::common::font::Font;
use rtx_core::common::object::Object;
use rtx_core::common::stateful_cmp::StatefulEq;
use rtx_core::definition::argument::ArgWrap;
use rtx_core::definition::conditional::{Conditional, ConditionalOptions, ConditionalType};
use rtx_core::definition::constructor::{Constructor, ConstructorOptions};
use rtx_core::definition::expandable::{Expandable, ExpandableOptions};
use rtx_core::definition::math_primitive::{MathPrimitive, MathPrimitiveOptions};
use rtx_core::definition::primitive::{Primitive, PrimitiveOptions};
use rtx_core::definition::register::{Register, RegisterGetterClosure, RegisterSetterClosure, RegisterType, RegisterValue};
use rtx_core::definition::{
  BeforeDigestClosure, ConditionalClosure, ConstructionClosure, Definition, DigestionClosure, ExpansionBody, PrimitiveClosure, ReplacementClosure,
};
use rtx_core::document::Document;
use rtx_core::gullet::Gullet;
use rtx_core::parameter::Parameters;
use rtx_core::state::{Scope, State, Stored};
use rtx_core::stomach::Stomach;
use rtx_core::tbox::Tbox;
use rtx_core::token::*;
use rtx_core::tokens::Tokens;
use rtx_core::whatsit::Whatsit;
use rtx_core::Digested;

use super::content::merge_font;
use super::*;

/// Is defined in the `LaTeX`-y sense of also not being let to \relax.
pub fn is_defined(name: &str, state: &State) -> bool {
  let cs = T_CS!(name);
  is_defined_token(&cs, state)
}

pub fn is_defined_token(cs: &Token, state: &State) -> bool {
  let meaning = state.lookup_meaning(cs);
  match meaning {
    Some(store) => match store {
      Stored::Token(ref m) => true,
      Stored::Expandable(ref m) => m.get_cs_name() != "\\relax",
      Stored::Primitive(ref m) => m.get_cs_name() != "\\relax",
      Stored::Constructor(ref m) => m.get_cs_name() != "\\relax",
      _ => false,
    },
    _ => false,
  }
}

pub fn is_definable(token: &Token, state: &State) -> bool {
  let meaning = state.lookup_meaning(token);
  let mut name = token.get_string();
  (name != "\\relax" && !name.starts_with("\\end")) && (meaning.is_none() || meaning.eq(&state.lookup_meaning(&T_RELAX), &state))
}

pub fn coerce_cs(t: &str) -> Token { T_CS!(t) }

pub fn revert(_arg: &[Token]) -> Tokens { unimplemented!() }

//======================================================================
// Defining Conditional Control Sequences.
//======================================================================
// Define a conditional control sequence. Its processing takes place in
// the Gullet.  The test is applied to the arguments (if any),
// which determines which branch is executed.
// If the test is undefined, the conditional is a "user defined" one;
// Two additional primitives are defined \footrue and \foofalse;
// the test is then determined by the most recently called of those.
//
// If you supply a skipper instead of a test, it is also applied to the arguments
// and should skip to the right place in the following \or, \else, \fi.

pub fn def_conditional(
  cs: Token,
  paramlist: Option<Parameters>,
  test: Option<ConditionalClosure>,
  options: ConditionalOptions,
  gullet: &mut Gullet,
  state: &mut State,
) {
  let cs_name = cs.get_cs_name();
  let locked_key = if let Some(true) = options.locked {
    s!("{}:locked", cs_name)
  } else {
    String::new()
  };
  match cs_name {
    "\\fi" | "\\else" | "\\or" => state.install_definition(
      Conditional {
        cs: cs.clone(),
        paramlist: None,
        test: None,
        conditional_type: ConditionalType::from(cs_name),
        locked: options.locked,
        skipper: options.skipper,
      },
      options.scope,
    ),
    custom => {
      if let Some(captures) = CONDITIONAL_CS_RE.captures(custom) {
        let name = captures.get(1).map_or("", |m| m.as_str()).to_string();
        if !name.is_empty() && name != "case" && test.is_none() {
          // user-defined conditional, like with \newif
          // Note: setting up these macros is compile-time expensive, maybe there is some way to
          // avoid...
          // Note: the double clones are technically correct Rust if annoying to write and read.
          //       first, we want to capture a cloned value of cs, to be able to keep using cs here.
          // second, each invocation of the conditional macro needs to create new tokens to
          // return,       hence a clone is required on each call.
          def_macro(
            T_CS!(s!("\\{}true", name)),
            None,
            Tokens!(T_CS!("\\let"), cs.clone(), T_CS!("\\iftrue")),
            None,
            state,
          );
          def_macro(
            T_CS!(s!("\\{}false", name)),
            None,
            Tokens!(T_CS!("\\let"), cs.clone(), T_CS!("\\iffalse")),
            None,
            state,
          );
          state.let_i(&cs, T_CS!("\\iffalse"), None, gullet);
        } else {
          //  For \ifcase, the parameter list better be a single Number !!
          state.install_definition(
            Conditional {
              cs,
              paramlist,
              test,
              conditional_type: ConditionalType::If,
              locked: options.locked,
              skipper: options.skipper,
            },
            options.scope,
          );
        }
      } else {
        let message = s!("The conditional {} is being defined but doesn't start with \\if", cs);
        Error!("misdefined", cs, None, state, message);
      }
    },
  }

  if let Some(true) = options.locked {
    state.assign_value(&locked_key, true, None);
  }
}

pub fn def_macro<T: Into<Option<ExpansionBody>>>(
  cs: Token,
  paramlist: Option<Parameters>,
  expansion: T,
  options_opt: Option<ExpandableOptions>,
  state: &mut State,
) {
  let expansion_opt: Option<ExpansionBody> = expansion.into();
  // TODO: The None case could be refactored to feel much cleaner.
  // For now it's equivalent to Tokens!()
  let expansion = expansion_opt.unwrap_or_default();
  let mut options = options_opt.unwrap_or_default();
  let scope = options.scope.take();
  if options.mathactive && cs.get_string().len() == 1 {
    state.assign_mathcode(cs.get_string().chars().next().unwrap(), 0x8000u16, scope.clone());
  }
  let locked_key_opt = if options.locked { Some(format!("{cs}:locked")) } else { None };
  state.install_definition(Expandable::new(cs, paramlist, expansion, Some(options), state), scope);
  if let Some(locked_key) = locked_key_opt {
    state.assign_value(&locked_key, true, Some(Scope::Global));
  }
}

#[derive(Default)]
pub struct RegisterOptions {
  pub getter: Option<RegisterGetterClosure>,
  pub setter: Option<RegisterSetterClosure>,
  pub readonly: bool,
  pub name: Option<String>
}

pub fn def_register<T: Into<RegisterValue>>(cs: Token, parameters: Option<Parameters>, value: T, options: Option<RegisterOptions>, state: &mut State) {
  let options: RegisterOptions = options.unwrap_or_default();
  let value: RegisterValue = value.into();
  let name = options.name.unwrap_or_else(|| cs.to_string() );
  let register_type: RegisterType = value.borrow().into();
  // Prepare clones to move into closures
  let getter_value = value.clone();
  let setter_name = name.clone();

  let getter: RegisterGetterClosure = match options.getter {
    Some(getter) => getter.clone(),
    None => {
      let name_clone = name.clone();
      Arc::new(move |args: Vec<ArgWrap>, state: &mut State| -> Option<RegisterValue> {
        let args_string: String = args.iter().map(ToString::to_string).collect::<Vec<String>>().join("");
        match state.lookup_value(&format!("{name_clone}{args_string}")) {
          None => Some(getter_value.clone()),
          Some(v) => v.into(),
        }
      })
    }
  };
  let readonly = options.readonly;

  let setter: RegisterSetterClosure = match options.setter {
    Some(setter) => setter.clone(),
    None => {
      if readonly {
        Arc::new(move |value, args, state| {
          let message = s!("Can't assign to register {}", setter_name);
          Warn!("unexpected", setter_name, None, state, message);
        })
      } else {
        Arc::new(move |value, args, state| {
          let args_string: String = args
            .into_iter()
            .map(|a| a.as_tokens(state).expect("TODO: handle malformed values here.").unwrap().to_string())
            .collect::<Vec<String>>()
            .join("");

          state.assign_value(&(setter_name.clone() + &args_string), value, None);
        })
      }
    },
  };

  // Not really right to set the value!
  state.assign_value(&cs.to_string(), value, None);
  state.install_definition(
    Register {
      cs,
      name,
      parameters,
      register_type,
      readonly,
      getter,
      setter,
      value: None,
      internalcs: None,
    },
    Some(Scope::Global),
  );
}

pub fn def_primitive(
  cs: Token,
  paramlist: Option<Parameters>,
  compiled_replacement: Option<PrimitiveClosure>,
  options: PrimitiveOptions,
  state: &mut State,
) {
  let options_locked = options.locked;
  let scope = options.scope;
  let mut before_digest_env: Vec<BeforeDigestClosure> = Vec::new();
  let cs_name = cs.get_cs_name().to_owned();

  if options.require_math {
    let cs_name_cloned = cs_name.clone();
    let require_math_closure = before_digest_single!(stomach, state, { requireMath!(cs_name_cloned, state) });
    before_digest_env.push(require_math_closure);
  }

  if options.forbid_math {
    let cs_name_cloned = cs_name.clone();
    let forbid_math_closure = before_digest_single!(stomach, state, { forbidMath!(cs_name_cloned, state) });
    before_digest_env.push(forbid_math_closure);
  }
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let begin_mode_closure = before_digest_single!(stomach, state, {
      stomach.begin_mode(&mode_clone, state)?;
    });
    before_digest_env.push(begin_mode_closure);
  } else if options.bounded {
    let bgroup_closure = before_digest_single!(stomach, state, {
      stomach.bgroup(state);
    });
    before_digest_env.push(bgroup_closure);
  }
  if let Some(chosen_font) = options.font {
    let merge_font_closure = before_digest_single!(stomach, state, {
      MergeFont!(chosen_font.clone(), state);
    });
    before_digest_env.push(merge_font_closure);
  }
  before_digest_env.extend(options.before_digest);

  let mut after_digest_env: Vec<DigestionClosure> = options.after_digest;
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let end_mode_closure: DigestionClosure = after_digest_single!(stomach, whatsit, state, {
      stomach.end_mode(&mode_clone, state)?;
    });
    after_digest_env.push(end_mode_closure);
  } else if options.bounded {
    let egroup_closure: DigestionClosure = after_digest_single!(stomach, whatsit, state, {
      stomach.egroup(state)?;
    });
    after_digest_env.push(egroup_closure);
  }

  state.install_definition(
    Primitive {
      cs,
      paramlist,
      replacement: compiled_replacement,
      before_digest: before_digest_env,
      after_digest: after_digest_env,
      alias: options.alias,
      nargs: options.nargs,
      is_prefix: options.is_prefix,
      reversion: options.reversion,
    },
    scope,
  );
  if options_locked {
    state.assign_value(&s!("{}:locked", cs_name), true, None);
  }
}

pub fn def_math_primitive(cs: Token, paramlist: Option<Parameters>, presentation: String, mut options: MathPrimitiveOptions, state: &mut State) {
  let scope = options.scope.clone();
  let reqfont = match options.font {
    Some(ref fnt) => fnt.clone(),
    None => Font::default(),
  };
  let moved_options = options.clone();

  state.install_definition(
    MathPrimitive {
      cs: cs.clone(),
      paramlist: None, // never any parameters, this is intentional
      replacement: Some(Arc::new(move |stomach, args, state| {
        let locator = stomach.get_locator().unwrap().into_owned();
        let mut properties = moved_options.clone();
        properties.mode = Some(String::from("math"));
        // TODO: Improve font precision here, the defaults may not belong in this lookup
        let font = Arc::new(state.lookup_font().unwrap().merge(reqfont.clone()).specialize(&presentation));

        // foreach my $key (keys %properties) {
        //   my $value = $properties{$key};
        //   if (ref $value eq 'CODE') {
        //     $properties{$key} = &$value(); } }
        // info!("defmath_prim: {}, tokens: {:?}", &$presentation, $cs);
        Ok(vec![Digested::from(Tbox {
          text: presentation.clone(),
          tokens: Tokens!(cs.clone()),
          font,
          properties: properties.to_hash_stored(),
          locator,
        })])
      })),
      options,
      ..MathPrimitive::default()
    },
    scope,
  );
}

pub fn def_constructor(
  cs: Token,
  paramlist: Option<Parameters>,
  compiled_replacement: Option<ReplacementClosure>,
  options: ConstructorOptions,
  state: &mut State,
) {
  // TODO: This won't work, as we can only invoke method calls on paramlist in runtime
  //*rtx_codegen::constructable::NARGS = $paramlist.get_num_args();
  let scope = options.scope;
  let is_locked = options.locked;
  let cs_name = cs.get_cs_name().to_owned();
  let locked_key = if is_locked { s!("{}:locked", cs_name) } else { String::new() };

  let mut before_digest_closures: Vec<BeforeDigestClosure> = Vec::new();

  if options.require_math {
    let cs_name_cloned = cs_name.clone();
    let require_math_closure = before_digest_single!(stomach, state, { requireMath!(cs_name_cloned, state) });
    before_digest_closures.push(require_math_closure);
  }
  if options.forbid_math {
    let cs_name_cloned = cs_name;
    let forbid_math_closure = before_digest_single!(stomach, state, { forbidMath!(cs_name_cloned, state) });
    before_digest_closures.push(forbid_math_closure);
  }
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let begin_mode_closure = before_digest_single!(stomach, state, {
      stomach.begin_mode(&mode_clone, state)?;
    });
    before_digest_closures.push(begin_mode_closure);
  } else if options.bounded {
    let bgroup_closure = before_digest_single!(stomach, state, {
      stomach.bgroup(state);
    });
    before_digest_closures.push(bgroup_closure);
  }
  if let Some(chosen_font) = options.font {
    let merge_font_closure = before_digest_single!(stomach, state, {
      MergeFont!(chosen_font.clone(), state);
    });
    before_digest_closures.push(merge_font_closure);
  }
  before_digest_closures.extend(options.before_digest);

  let mut after_digest_closures: Vec<DigestionClosure> = options.after_digest;
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let end_mode_closure: DigestionClosure = after_digest_single!(stomach, whatsit, state, {
      stomach.end_mode(&mode_clone, state)?;
    });
    after_digest_closures.push(end_mode_closure);
  } else if options.bounded {
    let egroup_closure: DigestionClosure = after_digest_single!(stomach, whatsit, state, {
      stomach.egroup(state)?;
    });
    after_digest_closures.push(egroup_closure);
  }

  let constructor = Constructor {
    cs,
    paramlist,
    replacement: compiled_replacement,
    before_digest: before_digest_closures,
    after_digest: after_digest_closures,
    before_construct: options.before_construct,
    after_construct: options.after_construct,
    nargs: options.nargs,
    alias: options.alias,
    reversion: options.reversion,
    // sizer
    capture_body: options.capture_body,
    properties: options.properties,
    // outer
    // long
    ..Constructor::default()
  };
  state.install_definition(constructor, scope);

  if is_locked {
    state.assign_value(&locked_key, true, None);
  }
}

pub fn def_environment(
  name: String,
  paramlist: Option<Parameters>,
  compiled_replacement: Option<ReplacementClosure>,
  options: ConstructorOptions,
  state: &mut State,
) {
  let begin_name = s!("\\begin{{{}}}", &name);
  let end_name = s!("\\end{{{}}}", &name);
  // This is for the common case where the environment is opened by \begin{env}
  // let sizer = inferSizer($options.sizer, $options.reversion);
  let mut before_digest_env: Vec<BeforeDigestClosure> = Vec::new();
  match &options.mode {
    Some(ref mode) => {
      let bmode = mode.clone();
      let mode_closure = Arc::new(move |stomach: &mut Stomach, state: &mut State| {
        stomach.begin_mode(&bmode, state)?;
        Ok(Vec::new())
      });
      before_digest_env.push(mode_closure);
    },
    None => {
      let bgroup_closure = before_digest_single!(stomach, state, {
        stomach.bgroup(state);
      });
      before_digest_env.push(bgroup_closure);
    },
  };
  if options.require_math {
    let require_name = begin_name.clone();
    let require_math_closure = before_digest_single!(stomach, state, { requireMath!(require_name, state) });
    before_digest_env.push(require_math_closure);
  }
  if options.forbid_math {
    let forbid_name = begin_name.clone();
    let forbid_math_closure = before_digest_single!(stomach, state, { forbidMath!(forbid_name, state) });
    before_digest_env.push(forbid_math_closure);
  }

  let env_name = name.clone();
  let current_environment_closure = before_digest_single!(stomach, state, {
    AssignValue!("current_environment", env_name.clone(), None, state);
    let body = T_LETTER!(env_name.clone());
    DefMacro!(T_CS!("\\@currenvir"), None, body, state);
  });
  before_digest_env.push(current_environment_closure);

  if let Some(chosen_font) = options.font {
    let merge_font_closure = before_digest_single!(stomach, state, {
      MergeFont!(chosen_font.clone(), state);
    });
    before_digest_env.push(merge_font_closure);
  }
  before_digest_env.extend(options.before_digest);

  let push_frame_closure = Arc::new(|_document: &mut Document, _whatsit: &Whatsit, state: &mut State| {
    state.push_frame();
    Ok(())
  });
  let mut before_construct_with_frame: Vec<ConstructionClosure> = vec![push_frame_closure];
  before_construct_with_frame.extend(options.before_construct);

  let mut after_construct_with_frame: Vec<ConstructionClosure> = options.after_construct;

  let pop_frame_closure = Arc::new(|_document: &mut Document, _whatsit: &Whatsit, state: &mut State| {
    state.pop_frame()?;
    Ok(())
  });
  after_construct_with_frame.push(pop_frame_closure);

  let begin_name_constructor = Arc::new(Constructor {
    cs: T_CS!(begin_name),
    paramlist: paramlist.clone(),
    replacement: compiled_replacement.clone(),
    nargs: options.nargs,
    before_digest: before_digest_env,
    after_digest: options.after_digest_begin,
    after_digest_body: options.after_digest_body,
    before_construct: before_construct_with_frame,
    // Curiously, it's the \begin whose afterConstruct gets called.
    after_construct: after_construct_with_frame,
    capture_body: true,
    properties: options.properties.clone(),
    // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
    // (defined $sizer ? (sizer => $sizer) : ()),
    // ), $options{scope});
    reversion: options.reversion,
    alias: options.alias,
    sizer: options.sizer,
  });
  state.install_definition(begin_name_constructor, options.scope.clone());

  let mut after_digest_env = options.after_digest.clone();
  let name_clone = name.to_string();
  let end_name_clone = end_name.to_string();
  let unexpected_end_closure = after_digest_single!(stomach, whatsit, state, {
    let env = state.lookup_string("current_environment");
    if env.is_empty() || name_clone != env {
      let message1 = s!("Can't close environment {}", name_clone);
      let message2 = s!(
        "Current are {} ",
        state
          .lookup_stacked_values("current_environment")
          .iter()
          .map(|x| s!("{:?}", x))
          .collect::<Vec<String>>()
          .join(", ")
      );
      Error!("unexpected", end_name_clone, stomach, state, message1, message2);
    }
    Ok(Vec::new())
  });
  after_digest_env.push(unexpected_end_closure);

  match options.mode {
    Some(mode) => {
      let emode = mode;
      let emode_closure = Arc::new(move |stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
        stomach.end_mode(&emode, state)?;
        Ok(Vec::new())
      });
      after_digest_env.push(emode_closure);
    },
    None => {
      let egroup_closure = Arc::new(|stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
        stomach.egroup(state)?;
        Ok(Vec::new())
      });
      after_digest_env.push(egroup_closure);
    },
  };

  let end_envname_constructor = Arc::new(Constructor {
    cs: T_CS!(end_name),
    replacement: None,
    paramlist: None,
    before_digest: options.before_digest_end,
    after_digest: after_digest_env,
    ..Constructor::default() // TODO ? fill in missing ones
  });
  state.install_definition(end_envname_constructor, options.scope.clone());

  // For the uncommon case opened by \csname env\endcsname
  let name_constructor = Arc::new(Constructor {
    cs: T_CS!(s!("\\{}", &name)),
    paramlist,
    replacement: compiled_replacement,
    // beforeDigest => flatten(($options{requireMath} ? (sub { requireMath($name); }) : ()),
    //   ($options{forbidMath} ? (sub { forbidMath($name); })              : ()),
    //   ($mode                ? (sub { $_[0]->beginMode($mode); })        : ()),
    //   ($options{font}       ? (sub { MergeFont(%{ $options{font} }); }) : ()),
    //   $options{beforeDigest}),
    // afterDigest     => flatten($options{afterDigestBegin}),
    // afterDigestBody => flatten($options{afterDigestBody}),
    // beforeConstruct => flatten(sub { state->pushFrame; }, $options{beforeConstruct}),
    // Curiously, it's the \begin whose afterConstruct gets called.
    // afterConstruct => flatten($options{afterConstruct}, sub { state->popFrame; }),
    nargs: options.nargs,
    capture_body: true,
    properties: options.properties.clone(),
    // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
    // (defined $sizer ? (sizer => $sizer) : ()),
    // ), $options{scope});
    ..Constructor::default()
  });
  state.install_definition(name_constructor, options.scope.clone());
  let end_name = s!("\\end{}", &name);
  let name_clone = name.clone(); // for after_digest
  let mut after_digest_end = options.after_digest;
  after_digest_end.push(after_digest_single!(stomach, whatsit, state, {
    stomach.egroup(state)?;
  }));

  let end_name_constructor = Constructor {
    cs: T_CS!(end_name),
    paramlist: None,
    replacement: None,
    after_digest: after_digest_end,
    // beforeDigest => flatten($options{beforeDigestEnd}),
    //   ($mode ? (sub { $_[0]->endMode($mode); }) : ())),
    // ), $options{scope});
    ..Constructor::default()
  };
  state.install_definition(Arc::new(end_name_constructor), options.scope);

  if options.locked {
    state.assign_value(&s!("\\begin{{{}}}:locked", &name), true, None);
    state.assign_value(&s!("\\end{{{}}}:locked", &name), true, None);
    state.assign_value(&s!("\\{}:locked", &name), true, None);
    state.assign_value(&s!("\\end{}:locked", &name), true, None);
  }
}

use crate::prelude::*;
use std::collections::VecDeque;

fn clean_class_name(name: &str) -> String {
  name.trim().chars().filter(|c| c.is_alphanumeric()).collect()
}

#[rustfmt::skip]
LoadDefinitions!({
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
});

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
    Some(Stored::VecDequeStored(keys)) => {
      keys.iter().map(|k| k.to_string()).collect()
    },
    _ => vec![
      "\\thm@bodyfont".into(), "\\thm@headpunct".into(),
      "\\thm@styling".into(), "\\thm@headstyling".into(),
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
            _ => mouth::tokenize(&val.to_string()),
          };
          let _ = state::assign_register(
            &key,
            RegisterValue::Tokens(tokens),
            None,
            vec![],
          );
        } else {
          state::assign_value(&key, val, None);
        }
      }
      i += 2;
    }
  }
}

fn define_new_theorem(
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
  let has_type = typ.as_ref().map_or(false, |t| !t.is_empty());
  let is_starred = flag.is_some();

  let within_str = if let Some(ref w) = within {
    let ws = digest_literal(w.clone())?.to_string();
    if ws.is_empty() { None } else { Some(ws) }
  } else {
    None
  };

  let counter = otherthmset_str.clone().unwrap_or_else(|| thmset_str.clone());
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
    if let RegisterValue::Tokens(t) = reg { t.to_string() } else { "\\arabic".into() }
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
    let mut fmt_toks = vec![
      T_CS!("\\the"), T_CS!("\\thm@headfont"),
    ];
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
    move |document: &mut Document,
          _args: &Vec<Option<Digested>>,
          props: &SymHashMap<Stored>| {
      let mut av_props: HashMap<String, String> = HashMap::default();
      if let Some(stored) = props.get("id") {
        av_props.insert("xml:id".into(), stored.to_string());
      }
      av_props.insert("inlist".into(), inlist_val.clone());
      av_props.insert("class".into(), class_val.clone());
      let this_font_opt = match props.get("font") {
        Some(Stored::Font(f)) => Some(Cow::Borrowed(&**f)),
        Some(Stored::FontDirective(FontDirective::Asset(fa))) => Some(Cow::Borrowed(&**fa)),
        Some(Stored::FontDirective(FontDirective::Closure(code))) =>
          Some(Cow::Owned(code(None)?)),
        _ => None
      };
      if let Some(this_font) = this_font_opt {
        document.open_element("ltx:theorem", Some(av_props), Some(&this_font))?;
      } else {
        document.open_element("ltx:theorem", Some(av_props), None)?;
      }
      // #tags
      if let Some(ref stored_digested) = props.get("tags") {
        let digested_opt: Option<Digested> = (*stored_digested).into();
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
        Some(Stored::FontDirective(FontDirective::Closure(code))) =>
          Some(Cow::Owned(code(None)?)),
        _ => None
      };
      if let Some(title_font) = title_font_opt {
        document.open_element("ltx:title", Some(title_av), Some(&title_font))?;
      } else {
        document.open_element("ltx:title", Some(title_av), None)?;
      }
      if let Some(ref stored_digested) = props.get("title") {
        let digested_opt: Option<Digested> = (*stored_digested).into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      document.close_element("ltx:title")?;
      // #body
      if let Some(ref stored_digested) = props.get("body") {
        let digested_opt: Option<Digested> = (*stored_digested).into();
        if let Some(ref digested) = digested_opt {
          document.absorb(digested, None)?;
        }
      }
      Ok(())
    }));

  let thmset_for_before = thmset_str.clone();
  let counter_for_props = counter.clone();
  let thmset_for_props = thmset_str.clone();
  let is_starred_for_props = is_starred;
  let has_type_for_props = has_type;

  let mut options = ConstructorOptions::default();
  options.mode = Some("internal_vertical".into());
  options.scope = Some(Scope::Global);

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
    let digest_str = s!(
      "\\the\\thm@bodyfont\\the\\thm@styling\\def\\lx@thistheorem{{{name_str}}}"
    );
    let digested = stomach::digest(mouth::tokenize_internal(&digest_str))?;
    Ok(vec![digested])
  });
  options.after_digest_begin.push(after_digest_begin_closure);

  // before_digest_end
  let before_digest_end_closure: BeforeDigestClosure = Rc::new(move || {
    let digested = stomach::digest(mouth::tokenize_internal("\\thm@doendmark\\the\\thm@postwork"))?;
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

  // properties
  let thmset_for_tags = thmset_for_props.clone();
  let counter_for_tags = counter_for_props.clone();
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
            full_toks.extend(mouth::tokenize(&thmset_for_tags).unlist());
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

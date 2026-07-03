use latexml_core::{
  common::arena::SymHashMap,
  definition::{PropertiesClosure, argument::ArgWrap},
  document::Document,
};

use crate::prelude::*;

/// Perl: beginEnumItemize($type, $counter, $keys) — enumitem.sty.ltxml L80-112
fn begin_enum_itemize(
  itype: &str,
  counter: &str,
  keys: Option<&KeyVals>,
) -> Result<SymHashMap<Stored>> {
  let counter_str = if counter.is_empty() { "@item" } else { counter };
  let level = lookup_int(&s!("{counter_str}level")) + 1;
  let postfix = roman_aux(level);

  let usecounter = if postfix.is_empty() {
    counter_str.to_string()
  } else {
    s!("{counter_str}{postfix}")
  };

  // Merge defaults with argument keyvals
  let hash = merged_enumitem_keyvals(itype, level, keys);

  // Deal with shortlabels — Perl L88-93
  if let Some(kv) = keys {
    let pairs: Vec<_> = kv.get_pairs().collect();
    if let Some((first_key, first_val)) = pairs.first()
      && matches!(first_val, ArgWrap::None)
      && has_value("enumitem@shortlabels")
      && lookup_definition(&T_CS!(s!("\\KV@enumitem@{first_key}")))
        .ok()
        .flatten()
        .is_none()
    {
      let toks = mouth::tokenize_internal(first_key);
      set_enumeration_style(Some(&toks), Some(level as i32))?;
    }
  }

  // label / label* — Perl L94-101
  let label_toks = hash
    .get("label")
    .or_else(|| hash.get("label*"))
    .and_then(argwrap_to_tokens);
  if let Some(ref label) = label_toks {
    let llabel = replace_star(label, &T_OTHER!(&usecounter));
    let llabel = if hash.contains_key("label*") && level > 1 {
      let prev_postfix = roman_aux(level - 1);
      let prev_label_cs = T_CS!(s!("\\label{counter_str}{prev_postfix}"));
      let mut combined = vec![prev_label_cs];
      combined.extend(llabel.unlist());
      Tokens::new(combined)
    } else {
      llabel
    };
    def_macro(T_CS!(s!("\\the{usecounter}")), None, llabel.clone(), None)?;
    def_macro(T_CS!(s!("\\label{usecounter}")), None, llabel, None)?;
    def_macro(
      T_CS!(s!("\\fnum@{usecounter}")),
      None,
      Tokens::new(vec![
        T_BEGIN!(),
        T_CS!("\\makelabel"),
        T_BEGIN!(),
        T_CS!(s!("\\label{usecounter}")),
        T_END!(),
        T_END!(),
      ]),
      None,
    )?;
  }

  // ref — Perl L102-109
  if let Some(ref_toks) = hash.get("ref").and_then(argwrap_to_tokens) {
    let rref = replace_star(&ref_toks, &T_OTHER!(&usecounter));
    // Perl L104-108 hotfix: if the ref body contains \the<usecounter>,
    // expand it BEFORE redefining \the<usecounter> to itself — otherwise
    // the redefinition is recursive (driver: 1904.10839 with
    // ref=\theenumi{}).
    let the_cs = s!("\\the{usecounter}");
    let rref = if rref.to_string().contains(&the_cs) {
      do_expand(rref)?
    } else {
      rref
    };
    def_macro(T_CS!(the_cs), None, rref, None)?;
  }

  // font / format — Perl L110-111
  if let Some(font_toks) = hash
    .get("font")
    .or_else(|| hash.get("format"))
    .and_then(argwrap_to_tokens)
  {
    def_macro(T_CS!(s!("\\fnum@font@{usecounter}")), None, font_toks, None)?;
  }

  // Build BeginItemizeOptions from the merged hash
  let mut opts = BeginItemizeOptions::default();
  if let Some(aw) = hash.get("start") {
    match aw {
      ArgWrap::Number(n) => {
        opts.start = Some(*n);
      },
      ArgWrap::Tokens(toks) => {
        // start may arrive as a token string "12", parse it
        let s = toks.to_string().trim().to_string();
        if let Ok(n) = s.parse::<i64>() {
          opts.start = Some(Number(n));
        }
      },
      _ => {},
    }
  }
  if let Some(series_toks) = hash.get("series").and_then(argwrap_to_tokens) {
    opts.series = Some(series_toks);
  }
  if let Some(resume_toks) = hash.get("resume").and_then(argwrap_to_tokens) {
    opts.resume = Some(resume_toks.to_string());
  }
  if let Some(resume_star_toks) = hash.get("resume*").and_then(argwrap_to_tokens) {
    opts.resume_star = Some(resume_star_toks.to_string());
  }

  begin_itemize(itype, Some(counter_str), opts)
}

/// Perl: replace_star($tokens, $replacement) — enumitem.sty.ltxml L114-119
fn replace_star(tokens: &Tokens, replacement: &Token) -> Tokens {
  let src = tokens.unlist_ref();
  let mut out = Vec::with_capacity(src.len());
  for t in src {
    if t.with_str(|s| s == "*") && t.get_catcode() == Catcode::OTHER {
      out.push(*replacement);
    } else {
      out.push(*t);
    }
  }
  Tokens::new(out)
}

/// Perl: endEnumItemize($whatsit) — enumitem.sty.ltxml L121-126
fn end_enum_itemize(whatsit: &mut Whatsit) -> Result<Vec<Digested>> {
  if let Some(series) = whatsit.get_property("series") {
    let series_str = series.to_string();
    if !series_str.is_empty()
      && let Some(counter) = whatsit.get_property("counter")
    {
      let counter_str = counter.to_string();
      if let Ok(val) = counter_value(&counter_str) {
        assign_value(
          &s!("enumitem_series_{series_str}_last"),
          Stored::Number(val),
          Some(Scope::Global),
        );
      }
    }
  }
  Ok(Vec::new())
}

/// Perl: store_enumitem_defaults($name, $kv) — enumitem.sty.ltxml L228-237
fn store_enumitem_defaults(name: &str, kv: &KeyVals) {
  // Load existing keys directly inside the state/arena closure pair —
  // the intermediate keys_str String is avoided; we split the interned
  // &str and collect owned keys straight into the Vec.
  let mut keys: Vec<String> = with_value(&s!("{name}@keys"), |v| match v {
    Some(Stored::String(s)) => with(*s, |ks| {
      ks.split(',')
        .filter(|k| !k.is_empty())
        .map(String::from)
        .collect()
    }),
    _ => Vec::new(),
  });

  for (key, val) in kv.get_pairs() {
    let val_key = s!("{name}@{key}");
    match val {
      ArgWrap::Tokens(t) => {
        assign_value(&val_key, Stored::Tokens(t.clone()), Some(Scope::Global));
      },
      ArgWrap::None => {
        assign_value(&val_key, Stored::None, Some(Scope::Global));
      },
      _ => {
        assign_value(
          &val_key,
          Stored::String(pin(val.to_string())),
          Some(Scope::Global),
        );
      },
    }
    if !keys.contains(key) {
      keys.push(key.clone());
    }
  }
  assign_value(
    &s!("{name}@keys"),
    Stored::String(pin(keys.join(","))),
    Some(Scope::Global),
  );
}

/// Perl: merged_enumitem_keyvals($name, $level, $argkv) — enumitem.sty.ltxml L239-249
fn merged_enumitem_keyvals(
  name: &str,
  level: i64,
  argkv: Option<&KeyVals>,
) -> rustc_hash::FxHashMap<String, ArgWrap> {
  let mut hash = rustc_hash::FxHashMap::default();

  let default_names = [
    "enumitem_defaults".to_string(),
    s!("enumitem_{name}_defaults"),
    s!("enumitem_{name}{level}_defaults"),
  ];

  for def_name in &default_names {
    // with_value pulls the keys-string out of the Stored::String arm
    // without cloning the envelope; the per-key inner lookup still
    // needs to produce an owned ArgWrap, so we pay the clone there.
    // Collect keys as owned Vec<String> inside state+arena closures
    // so the split happens on the interned &str and the owned
    // intermediary is smaller (Vec<String> of just the keys, not
    // the whole comma-separated string plus allocs).
    let keys: Vec<String> = with_value(&s!("{def_name}@keys"), |v| match v {
      Some(Stored::String(s)) => with(*s, |ks| {
        ks.split(',')
          .filter(|k| !k.is_empty())
          .map(String::from)
          .collect()
      }),
      _ => Vec::new(),
    });
    if keys.is_empty() {
      continue;
    }
    for key in &keys {
      if !key.is_empty() {
        let val_key = s!("{def_name}@{key}");
        if let Some(val) = lookup_value(&val_key) {
          let aw = match val {
            Stored::Tokens(t) => ArgWrap::Tokens(t),
            Stored::Number(n) => ArgWrap::Number(n),
            Stored::None => ArgWrap::None,
            Stored::String(s) => ArgWrap::Tokens(with(s, mouth::tokenize_internal)),
            _ => ArgWrap::None,
          };
          hash.insert(key.clone(), aw);
        }
      }
    }
  }

  // Merge argument keyvals last (highest priority)
  if let Some(kv) = argkv {
    for (key, val) in kv.get_pairs() {
      hash.insert(key.clone(), val.clone());
    }
  }

  hash
}

/// Helper: convert ArgWrap to Option<Tokens>
fn argwrap_to_tokens(aw: &ArgWrap) -> Option<Tokens> {
  match aw {
    ArgWrap::Tokens(t) => Some(t.clone()),
    ArgWrap::None => None,
    _ => Some(mouth::tokenize_internal(&aw.to_string())),
  }
}

/// Perl: \newlist{name}{type}{maxdepth} — enumitem.sty.ltxml L184-206
fn newlist_impl(listname: &str, listtype: &str, maxdepth: i32) -> Result<()> {
  let (basetype, is_inline) = if let Some(base) = listtype.strip_suffix('*') {
    (base.to_string(), true)
  } else {
    (listtype.to_string(), false)
  };

  let elementname = if is_inline {
    s!("inline-{basetype}")
  } else {
    basetype.clone()
  };

  // Create counters for each depth level
  for d in 1..=(maxdepth as i64) {
    let ctr_name = s!("{listname}{}", roman_aux(d));
    new_counter(&ctr_name, "", None)?;
  }

  // Hook up to item command
  let item_source = if is_inline {
    s!("\\inline@{basetype}@item")
  } else {
    s!("\\{basetype}@item")
  };
  let_i(
    &T_CS!(s!("\\{listname}@item")),
    &T_CS!(item_source),
    Some(Scope::Global),
  );

  // Create the environment
  let env_cs = T_CS!(s!("\\begin{{{listname}}}"));
  let paramlist = parse_parameters("OptionalKeyVals:enumitem", &env_cs, true)?;

  let elem_open = s!("ltx:{elementname}");
  let elem_close = elem_open.clone();
  let replacement: ReplacementClosure = Rc::new(move |document, _args, props| {
    let mut av: HashMap<String, String> = HashMap::default();
    if let Some(id) = props.get("id") {
      av.insert("xml:id".into(), id.to_string());
    }
    document.open_element(&elem_open, Some(av), None)?;
    if let Some(body) = props.get("body") {
      let digested_opt: Option<Digested> = body.into();
      if let Some(ref digested) = digested_opt {
        document.absorb(digested, None)?;
      }
    }
    document.close_element(&elem_close)?;
    Ok(())
  });

  let ln = listname.to_string();
  let properties: PropertiesClosure = Rc::new(move |args| {
    let kv = extract_keyvals(args);
    begin_enum_itemize(&ln, &ln, kv.as_ref())
  });

  // Perl #2798: a block list ends with \par; an INLINE list must NOT \par
  // (it would break the surrounding paragraph) — mirrors the standard
  // itemize*/enumerate*/description* inline envs, which carry no before_digest_end.
  // No before_digest_end \par: Perl list environments have none — an
  // isolated Digest(\par) resets MODE to the bound vertical mode, which
  // DEFUSES the env-end leave_horizontal_internal repack; item text then
  // stays as bare char boxes and the vertical sizer stacks each as a line
  // (952pt for a 16-word item; witness 2605.02240's 12000pt tcolorbox
  // frames). endMode does the repacking, exactly like Perl.
  let before_digest_end: Vec<BeforeDigestClosure> = Vec::new();

  let after_digest_body: DigestionClosure =
    Rc::new(|whatsit: &mut Whatsit| end_enum_itemize(whatsit));

  let options = ConstructorOptions {
    // Perl #2798: inline lists are inline blocks (internal_vertical but NO
    // leaveHorizontal — they stay inside the surrounding paragraph).
    mode: Some(
      if is_inline {
        "inline_internal_vertical"
      } else {
        "internal_vertical"
      }
      .into(),
    ),
    locked: true,
    properties,
    before_digest_end,
    after_digest_body: vec![after_digest_body],
    ..Default::default()
  };
  def_environment(listname.to_string(), paramlist, Some(replacement), options);
  Ok(())
}

/// Extract KeyVals from a digested argument
fn extract_keyvals(args: &[Option<Digested>]) -> Option<KeyVals> {
  args.first().and_then(|a| {
    a.as_ref().and_then(|d| {
      if let DigestedData::KeyVals(kv) = d.data() {
        Some(kv.clone())
      } else {
        None
      }
    })
  })
}

#[rustfmt::skip]
LoadDefinitions!({
  // Package Options
  DeclareOption!("shortlabels", {
    AssignValue!("enumitem@shortlabels" => true);
  });
  DeclareOption!("inline", {
    AssignValue!("enumitem@inline" => true);
  });
  DeclareOption!("loadonly", {
    AssignValue!("enumitem@loadonly" => true);
  });
  ProcessOptions!();

  // KeyVals
  DefKeyVal!("enumitem", "label", "UndigestedKey");
  DefKeyVal!("enumitem", "label*", "UndigestedKey");
  DefKeyVal!("enumitem", "ref", "UndigestedKey");
  DefKeyVal!("enumitem", "font", "UndigestedKey");
  DefKeyVal!("enumitem", "format", "UndigestedKey");
  DefKeyVal!("enumitem", "start", "Number");
  DefKeyVal!("enumitem", "series", "UndigestedKey");
  DefKeyVal!("enumitem", "resume", "", "noseries");
  DefKeyVal!("enumitem", "resume*", "", "noseries");
  DefKeyVal!("enumitem", "style", "UndigestedKey");
  DefKeyVal!("enumitem", "itemjoin", "UndigestedKey");
  DefKeyVal!("enumitem", "itemjoin*", "UndigestedKey");
  DefKeyVal!("enumitem", "afterlabel", "UndigestedKey");
  DefKeyVal!("enumitem", "mode", "UndigestedKey");
  DefKeyVal!("enumitem", "align", "UndigestedKey");
  DefKeyVal!("enumitem", "labelindent", "Dimension");
  DefKeyVal!("enumitem", "left", "Dimension");
  DefKeyVal!("enumitem", "leftmargin", "UndigestedKey");
  DefKeyVal!("enumitem", "itemindent", "Dimension");
  DefKeyVal!("enumitem", "labelsep", "Dimension");
  DefKeyVal!("enumitem", "labelwidth", "Dimension");
  DefKeyVal!("enumitem", "widest", "UndigestedKey");
  DefKeyVal!("enumitem", "beginpenalty", "Number");
  DefKeyVal!("enumitem", "midpenalty", "Number");
  DefKeyVal!("enumitem", "endpenalty", "Number");
  DefKeyVal!("enumitem", "noitemsep", "", "true");
  DefKeyVal!("enumitem", "nolistsep", "", "true");
  DefKeyVal!("enumitem", "nosep", "", "true");
  DefKeyVal!("enumitem", "before", "UndigestedKey");
  DefKeyVal!("enumitem", "before*", "UndigestedKey");
  DefKeyVal!("enumitem", "after", "UndigestedKey");
  DefKeyVal!("enumitem", "after*", "UndigestedKey");
  // Spacing keyvals (Perl L160-175) — ignored for HTML but must be recognized
  DefKeyVal!("enumitem", "topsep", "Dimension");
  DefKeyVal!("enumitem", "partopsep", "Dimension");
  DefKeyVal!("enumitem", "parsep", "Dimension");
  DefKeyVal!("enumitem", "itemsep", "Dimension");
  DefKeyVal!("enumitem", "listparindent", "Dimension");
  DefKeyVal!("enumitem", "rightmargin", "Dimension");
  DefKeyVal!("enumitem", "wide", "", "true");
  DefKeyVal!("enumitem", "first", "UndigestedKey");
  DefKeyVal!("enumitem", "first*", "UndigestedKey");

  if !has_value("enumitem@loadonly") {
    DefEnvironment!("{itemize} OptionalKeyVals:enumitem",
      "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("itemize", "@item", kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      mode => "internal_vertical",
      locked => true
    );
    DefEnvironment!("{enumerate} OptionalKeyVals:enumitem",
      "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("enumerate", "enum", kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      mode => "internal_vertical",
      locked => true
    );
    DefEnvironment!("{description} OptionalKeyVals:enumitem",
      "<ltx:description xml:id='#id'>#body</ltx:description>",
      before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("description", "@desc", kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      mode => "internal_vertical",
      locked => true
    );
  }

  if has_value("enumitem@inline") {
    DefEnvironment!("{itemize*} OptionalKeyVals:enumitem",
      "<ltx:inline-itemize xml:id='#id'>#body</ltx:inline-itemize>",
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("inline@itemize", "@item", kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      // Perl #2798: inline lists are inline blocks — internal_vertical but NO
      // leaveHorizontal (they stay inside the surrounding paragraph).
      mode => "inline_internal_vertical"
    );
    DefEnvironment!("{enumerate*} OptionalKeyVals:enumitem",
      "<ltx:inline-enumerate xml:id='#id'>#body</ltx:inline-enumerate>",
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("inline@enumerate", "enum", kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      // Perl #2798: inline lists stay inside the surrounding paragraph.
      mode => "inline_internal_vertical"
    );
    DefEnvironment!("{description*} OptionalKeyVals:enumitem",
      "<ltx:inline-description xml:id='#id'>#body</ltx:inline-description>",
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("inline@description", "@desc", kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      // Perl #2798: inline lists stay inside the surrounding paragraph.
      mode => "inline_internal_vertical"
    );
  }

  // \newlist{name}{type}{maxdepth} — Perl: enumitem.sty.ltxml L184-206
  DefPrimitive!("\\newlist{}{}{}", sub[(listname, listtype, maxdepth)] {
    let listname = listname.to_string();
    let listtype = listtype.to_string();
    let maxdepth: i32 = maxdepth.to_string().parse().unwrap_or(4);
    newlist_impl(&listname, &listtype, maxdepth)?;
  });
  Let!("\\renewlist", "\\newlist");

  // \setlist[names]{keyvals} — Perl: enumitem.sty.ltxml L210-221
  DefPrimitive!("\\setlist Optional RequiredKeyVals:enumitem", sub[(names, kv)] {
    if let Some(ref names_toks) = names {
      let names_str = names_toks.to_string();
      let parts: Vec<&str> = names_str.split(',').map(|s| s.trim()).collect();
      if parts.len() == 1 && !parts[0].is_empty() {
        store_enumitem_defaults(&s!("enumitem_{}_defaults", parts[0]), &kv);
      } else if parts.len() > 1 {
        let name = parts[0];
        for level in &parts[1..] {
          store_enumitem_defaults(&s!("enumitem_{name}{level}_defaults"), &kv);
        }
      } else {
        store_enumitem_defaults("enumitem_defaults", &kv);
      }
    } else {
      store_enumitem_defaults("enumitem_defaults", &kv);
    }
  });

  // Obsolete shorthands
  DefMacro!("\\setitemize Optional {}", "\\setlist[itemize,#1]{#2}");
  DefMacro!("\\setenumerate Optional {}", "\\setlist[enumerate,#1]{#2}");
  DefMacro!("\\setdescription Optional {}", "\\setlist[description,#1]{#2}");

  // \restartlist — Perl enumitem.sty.ltxml L128-140 uses `DefMacro` with a
  // side-effect sub returning undef (empty expansion). Match that kind:
  // macro-level so the reset happens during gullet expansion, consistent
  // with how Perl dispatches `\restartlist` inside `\begin{enumerate}`.
  DefMacro!("\\restartlist{}", sub[(listname)] {
    let listname = listname.to_string();
    let counter = match listname.as_str() {
      "enumerate" => "enum",
      "itemize" => "@item",
      "description" => "@desc",
      _ => &listname,
    };
    for i in 1_i64..=6 {
      let r = roman_aux(i);
      let ctr_name = s!("{counter}{r}");
      if lookup_definition(&T_CS!(s!("\\c@{ctr_name}"))).ok().flatten().is_some() {
        SetCounter!(ctr_name, Number(0));
      }
    }
    Ok(Tokens!())
  });

  // Not-yet-handled bits
  def_macro_noop("\\SetLabelAlign{}{}")?;
  def_macro_noop("\\EnumitemId")?;
  def_macro_noop("\\SetEnumitemKey{}{}")?;
  def_macro_noop("\\SetEnumerateShortLabel{}{}")?;
  def_macro_noop("\\SetEnumitemValue{}{}{}")?;
  def_macro_noop("\\SetEnumitemSize{}{}")?;
  def_macro_noop("\\AddEnumerateCounter{}{}{}")?;
});

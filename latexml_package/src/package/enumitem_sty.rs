use latexml_core::{
  common::arena::SymHashMap,
  definition::{PropertiesClosure, argument::ArgWrap},
  document::Document,
  gullet::unread,
};

use crate::prelude::*;

/// Perl: beginEnumItemize($type, $counter, $keys) — enumitem.sty.ltxml L80-112. `enumerate_type`: the
/// list is an enumerate (or an enumerate-type `\newlist`), the only kind whose label and ref enumitem
/// reads for `*` counters (`\enit@enumerate@i`, enumitem.sty:1396-1397); an itemize or description
/// label is used as written (:1425-1436, :1459ff).
fn begin_enum_itemize(
  itype: &str,
  counter: &str,
  enumerate_type: bool,
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

  // Merge defaults with argument keyvals. `list<depth>` is the depth of this list, whose
  // `begin_itemize` advances `\@listdepth` below (enumitem's inline lists advance it too,
  // enumitem.sty:1188-1191).
  let listdepth = match lookup_register_quiet("\\@listdepth") {
    Some(RegisterValue::Number(n)) => n.0 + 1,
    _ => 1,
  };
  // enumitem keys a level's `\setlist` by the list's own depth register (`\enit@itemize@i` reads
  // `\@itemdepth`, which only itemizes advance); the `@item` level also counts kernel `\list`s and
  // `\trivlist`s, so an itemize inside a `\list` would take the second level's keys. Repro:
  // list-structure/itemize_label_follows_the_itemize_depth.
  let key_level = if matches!(itype, "itemize" | "inline@itemize") {
    match lookup_register_quiet("\\@itemdepth") {
      Some(RegisterValue::Number(n)) => n.0 + 1,
      _ => 1,
    }
  } else {
    level
  };
  let hash = merged_enumitem_keyvals(itype, key_level, listdepth, keys);

  // Deal with shortlabels — Perl L88-93: the label template the `EnumitemKeyVals`
  // parameter found (enumitem.sty's `\enit@first`).
  if let Some(template) = keys.and_then(|kv| kv.get_value(SHORTLABEL_KEY))
    && let Some(toks) = argwrap_to_tokens(template)
  {
    set_enumeration_style(Some(&toks), Some(level as i32))?;
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

  let mut props = begin_itemize(itype, Some(counter_str), opts)?;
  // label / label* — Perl L94-101. Defined AFTER `begin_itemize`, which aliases an itemize's
  // `\label<usecounter>` to `\labelitem<depth>`: the list's own label wins, as enumitem's
  // `\@itemlabel` does over `\labelitem<depth>` (enumitem.sty:519-521). Repro:
  // list-structure/itemize_label_follows_the_itemize_depth.
  let label_toks = hash
    .get("label")
    .or_else(|| hash.get("label*"))
    .and_then(argwrap_to_tokens);
  if let Some(ref label) = label_toks {
    let llabel = if enumerate_type {
      normalize_label(label, &usecounter)?
    } else {
      label.clone()
    };
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
    let rref = if enumerate_type {
      normalize_label(&ref_toks, &usecounter)?
    } else {
      ref_toks
    };
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

  // Surpass-Perl (OXIDIZED_DESIGN #105, issue #559): expose enumitem `leftmargin`
  // for CSS theming. Perl deliberately ignores every positioning key
  // (enumitem.sty.ltxml:54 "# IGNORED: Alignment, Positioning, penalties") to
  // keep the HTML structural; we instead surface `leftmargin` so a theme can act
  // on it. The flush mode `leftmargin=*` (a boolean-like toggle) becomes a
  // semantic `class` the stylesheet themes; an explicit `leftmargin=<dim>`
  // becomes a numeric `--ltx-enum-leftmargin` CSS custom property that the base
  // `.ltx_itemize, .ltx_enumerate { margin-left: var(--ltx-enum-leftmargin, 1em) }`
  // rule consumes (both live in ltx-article/book/report.css).
  if let Some(lm) = hash.get("leftmargin").and_then(argwrap_to_tokens) {
    let lm = lm.to_string();
    let lm = lm.trim();
    if lm == "*" {
      props.insert("class", s!("ltx_leftmargin_flush").into());
    } else if let Some(css) = css_length(lm) {
      props.insert("cssstyle", s!("--ltx-enum-leftmargin:{css}").into());
    }
    // A non-`*`, non-CSS-length value (a macro/`\dimexpr` like `\parindent`) is
    // dropped rather than leaked into CSS — the list falls back to the default
    // `--ltx-enum-leftmargin` (1em).
  }

  // Execute list-level `before` and `first` key code at list start
  // (order per enumitem.sty:1030-1060 \enit@before … \enit@first).
  let mut start_tokens = Vec::new();
  if let Some(b) = hash.get("before").and_then(argwrap_to_tokens) {
    start_tokens.extend(b.unlist());
  }
  if let Some(b) = hash.get("before*").and_then(argwrap_to_tokens) {
    start_tokens.extend(b.unlist());
  }
  if let Some(f) = hash.get("first").and_then(argwrap_to_tokens) {
    start_tokens.extend(f.unlist());
  }
  if let Some(f) = hash.get("first*").and_then(argwrap_to_tokens) {
    start_tokens.extend(f.unlist());
  }
  if !start_tokens.is_empty() {
    unread(Tokens::new(start_tokens));
  }

  // Store list-level `after` key code for execution at list end.
  let mut after_tokens = Vec::new();
  if let Some(a) = hash.get("after").and_then(argwrap_to_tokens) {
    after_tokens.extend(a.unlist());
  }
  if let Some(a) = hash.get("after*").and_then(argwrap_to_tokens) {
    after_tokens.extend(a.unlist());
  }
  if !after_tokens.is_empty() {
    props.insert("after", Stored::Tokens(Tokens::new(after_tokens)));
  }

  Ok(props)
}

/// A `leftmargin` value we can safely place verbatim in a CSS length: a number
/// followed by a unit CSS shares with TeX (`pt`/`em`/`ex`/`cm`/`mm`/`in`, plus
/// CSS `px`/`rem`/`%`), or a bare `0`. Returns `None` for anything else — a raw
/// TeX macro / `\dimexpr` is not valid CSS, so it must not reach the stylesheet.
fn css_length(raw: &str) -> Option<String> {
  let s = raw.trim();
  for unit in ["pt", "px", "em", "ex", "rem", "cm", "mm", "in", "%"] {
    if let Some(num) = s.strip_suffix(unit)
      && num.parse::<f64>().is_ok()
    {
      return Some(s.to_string());
    }
  }
  // A bare number (e.g. `0`) is a valid CSS length only when it is zero.
  s.parse::<f64>()
    .ok()
    .filter(|n| *n == 0.0)
    .map(|_| s.to_string())
}

/// enumitem.sty:910-936 `\enit@normlabel` (`\lx@enit@normlabel`, defined in TeX below): in a group,
/// every counter command of the label list (`\enit@labellist`: the kernel's five and each
/// `\AddEnumerateCounter`) and `\value` read their next argument and put `{<usecounter>}` for a `*`;
/// the label is then expanded by `\protected@xdef`, braced (enumitem.sty:934, "as \ref is in the
/// global scope"). So `\roman{*}`, `\csname greek\endcsname*` and a
/// macro expanding to a counter command all star, and `**` stays text. Perl replaced every `*`
/// (enumitem.sty.ltxml:114-119). Repros: list-structure/{enumitem_label_star_is_read_by_expansion,
/// addenumeratecounter_reads_the_star}; witnesses 2605.17001, 2605.06065, moreenum-doc. Enumerate-type
/// lists only (`enumerate_type` in `begin_enum_itemize`): an itemize label such as a `\tikz` bullet does
/// not survive `\xdef` (list-structure/itemize_label_is_used_as_written).
fn normalize_label(label: &Tokens, usecounter: &str) -> Result<Tokens> {
  // `\lx@enit@label` starts as `\relax`, so a `\protected@xdef` that never assigns (an error
  // recovery) keeps the label as written rather than the previous list's.
  let mut toks = vec![
    T_CS!("\\global"),
    T_CS!("\\let"),
    T_CS!("\\lx@enit@label"),
    T_CS!("\\relax"),
    T_CS!("\\lx@enit@normlabel"),
    T_BEGIN!(),
  ];
  toks.extend(Explode!(usecounter));
  toks.push(T_END!());
  toks.push(T_BEGIN!());
  toks.extend(label.unlist_ref().iter().copied());
  toks.push(T_END!());
  digest(Tokens::new(toks))?;
  let defn = lookup_definition(&T_CS!("\\lx@enit@label"))?;
  // An `\xdef` body is balanced in TeX; ours is not only after an error recovery inside the label
  // (an unprotected `\tikz`, where pdflatex errors too), and an unbalanced `\the<ctr>` would end
  // the document with `Fatal:Stomach:Misdefined`: keep the label as written then.
  Ok(match defn.as_ref().and_then(|d| d.get_expansion()) {
    Some(ExpansionBody::Tokens(body)) if body.is_balanced() => body.clone(),
    _ => label.clone(),
  })
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
  if let Some(Stored::Tokens(after)) = whatsit.properties.get("after") {
    unread(after.clone());
  }
  Ok(Vec::new())
}

/// The store of `\setlist[<name>,<level>]`'s keys, enumitem's `\enit@@<name><roman level>`
/// (enumitem.sty:1597-1612): level 0 is every level of the list, the name `list` every list.
fn enumitem_defaults_key(name: &str, level: i64) -> String {
  match (name, level) {
    ("list", 0) => "enumitem_defaults".to_string(),
    (_, 0) => s!("enumitem_{name}_defaults"),
    _ => s!("enumitem_{name}{level}_defaults"),
  }
}

/// The `\setlist` name of a list binding: enumitem's inline lists (the `inline` option,
/// enumitem.sty:1796-1805) are `\newenvironment`s that run the base list's machinery under its
/// name, so `enumerate*` reads `\setlist[enumerate]`'s keys; there is no `\enitdp@enumerate*`.
fn enumitem_setlist_name(itype: &str) -> &str { itype.strip_prefix("inline@").unwrap_or(itype) }

/// enumitem.sty:1680-1685 `\enit@setlist@i`: an entry of `\setlist`'s names is a list when
/// `\enitdp@<entry>` is defined (the standard lists, `trivlist` and every `\newlist`), and a level
/// otherwise; `\setlist[enumerate*]` is a level ("Missing number") in enumitem too.
fn enumitem_is_list_name(name: &str) -> bool {
  matches!(name, "itemize" | "enumerate" | "description" | "trivlist")
    || has_value(&s!("enumitem_list_{name}"))
}

/// Perl: store_enumitem_defaults($name, $kv) — enumitem.sty.ltxml L228-237
fn store_enumitem_defaults(name: &str, kv: &KeyVals, scope: Option<Scope>) {
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
        assign_value(&val_key, Stored::Tokens(t.clone()), scope);
      },
      ArgWrap::None => {
        assign_value(&val_key, Stored::None, scope);
      },
      _ => {
        assign_value(&val_key, Stored::String(pin(val.to_string())), scope);
      },
    }
    if !keys.contains(key) {
      keys.push(key.clone());
    }
  }
  assign_value(
    &s!("{name}@keys"),
    Stored::String(pin(keys.join(","))),
    scope,
  );
}

/// Perl: merged_enumitem_keyvals($name, $level, $argkv) — enumitem.sty.ltxml L239-249
///
/// enumitem.sty:977-980 applies the stored keys of `list`, `list<\@listdepth>`, the list's
/// name, then the name at its level; Perl's took the first and the last two.
fn merged_enumitem_keyvals(
  name: &str,
  level: i64,
  listdepth: i64,
  argkv: Option<&KeyVals>,
) -> rustc_hash::FxHashMap<String, ArgWrap> {
  let mut hash = rustc_hash::FxHashMap::default();

  let name = enumitem_setlist_name(name);
  let default_names = [
    enumitem_defaults_key("list", 0),
    enumitem_defaults_key("list", listdepth),
    enumitem_defaults_key(name, 0),
    enumitem_defaults_key(name, level),
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
            Stored::String(s) => ArgWrap::Tokens(with(s, |t| {
              mouth::tokenize_internal(TeXString::assembled(t.to_string()))
            })),
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
    _ => Some(mouth::tokenize_internal(TeXString::assembled(
      aw.to_string(),
    ))),
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

  // enumitem's `\enitdp@<name>`: `\setlist` now takes the name as a list (`enumitem_is_list_name`).
  assign_value(&s!("enumitem_list_{listname}"), true, Some(Scope::Global));

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
  let paramlist = parse_parameters("EnumitemKeyVals", &env_cs, true)?;

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
  let enumerate_type = basetype == "enumerate";
  let properties: PropertiesClosure = Rc::new(move |args| {
    let kv = extract_keyvals(args);
    begin_enum_itemize(&ln, &ln, enumerate_type, kv.as_ref())
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
/// The internal key under which `EnumitemKeyVals` passes a short label.
const SHORTLABEL_KEY: &str = "lx@shortlabel";

/// enumitem.sty:660-681 `\enit@first`, run by the `shortlabels` option
/// (:1787-1794): the first element of a list's key list, when it holds no `=`
/// and names no enumitem key, is a label template, which enumitem sets as
/// `label=`. Read as a key, `[(a)]` raised "unknown KeyVals key '(a)'".
fn mark_short_label(list: Tokens) -> Tokens {
  let toks = list.unlist();
  let mut depth = 0i32;
  let mut end = toks.len();
  for (i, t) in toks.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      Catcode::OTHER if depth == 0 && *t == T_OTHER!(",") => {
        end = i;
        break;
      },
      _ => {},
    }
  }
  let first = &toks[..end];
  let key = Tokens::new(first.to_vec()).to_string();
  let key = key.trim();
  if key.is_empty()
    || first.contains(&T_OTHER!("="))
    || has_meaning(&T_CS!(s!("\\KV@enumitem@{key}")))
  {
    return Tokens::new(toks);
  }
  let mut marked = mouth::tokenize_internal(TeXString::assembled(s!("{SHORTLABEL_KEY}="))).unlist();
  marked.push(T_BEGIN!());
  marked.extend_from_slice(first);
  marked.push(T_END!());
  marked.extend_from_slice(&toks[end..]);
  Tokens::new(marked)
}

fn extract_keyvals(args: &[Option<Digested>]) -> Option<KeyVals> {
  args.first().and_then(|a| {
    a.as_ref().and_then(|d| {
      if let DigestedData::KeyVals(kv) = d.data() {
        Some((**kv).clone())
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

  // enumitem.sty:108-117 `\enitkv@key{prefix}{key}[default]{code}` — the
  // package's private keyval clone, used by classes to add list keys
  // (verifica.cls:307 `\enitkv@key{}{fattorevf}{\fattorevf{#1}}`). The keys
  // here live in keyval's `enumitem` set, so route it to `\define@key`
  // (the `[default]{code}` tail is read by `\define@key` itself).
  RequirePackage!("keyval");
  DefMacro!("\\enitkv@key{}{}", "\\define@key{enumitem}{#2}");

  // A list's `[keys]`, with enumitem's short label marked when `shortlabels` is on.
  DefParameterType!(EnumitemKeyVals, sub[_inner, _extra] {
    if if_next(T_OTHER!("["))? {
      if has_value("enumitem@shortlabels") {
        let list = read_optional(None)?.unwrap_or(Tokens!());
        let mut again = vec![T_OTHER!("[")];
        again.extend(mark_short_label(list).unlist());
        again.push(T_OTHER!("]"));
        unread(Tokens::new(again));
      }
      Some(keyvals_aux(Some(T_OTHER!("]")), KVSpec {
        prefix: Some("enumitem".to_string()),
        ..KVSpec::default()
      })?)
    } else {
      None
    }
  }, optional => true,
  reversion => sub[arg, _inner, _extra] {
    Ok(Tokens!(T_OTHER!("["), Tokens::new(arg).revert(), T_OTHER!("]")))
  });

  // KeyVals
  DefKeyVal!("enumitem", SHORTLABEL_KEY, "UndigestedKey");
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

  // Each list locally resets `\makelabel` (latex.ltx:16061/16072) — see the
  // matching note on `{itemize}` in latex_constructs.rs.
  if !has_value("enumitem@loadonly") {
    DefEnvironment!("{itemize} EnumitemKeyVals",
      "<ltx:itemize xml:id='#id' class='#class' cssstyle='#cssstyle'>#body</ltx:itemize>",
      before_digest => { def_macro_identity("\\makelabel{}")?; },
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("itemize", "@item", false, kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      mode => "internal_vertical",
      locked => true
    );
    DefEnvironment!("{enumerate} EnumitemKeyVals",
      "<ltx:enumerate xml:id='#id' class='#class' cssstyle='#cssstyle'>#body</ltx:enumerate>",
      before_digest => { def_macro_identity("\\makelabel{}")?; },
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("enumerate", "enum", true, kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      mode => "internal_vertical",
      locked => true
    );
    DefEnvironment!("{description} EnumitemKeyVals",
      "<ltx:description xml:id='#id' class='#class' cssstyle='#cssstyle'>#body</ltx:description>",
      before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("description", "@desc", false, kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      mode => "internal_vertical",
      locked => true
    );
  }

  if has_value("enumitem@inline") {
    DefEnvironment!("{itemize*} EnumitemKeyVals",
      "<ltx:inline-itemize xml:id='#id' class='#class' cssstyle='#cssstyle'>#body</ltx:inline-itemize>",
      before_digest => { def_macro_identity("\\makelabel{}")?; },
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("inline@itemize", "@item", false, kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      // Perl #2798: inline lists are inline blocks — internal_vertical but NO
      // leaveHorizontal (they stay inside the surrounding paragraph).
      mode => "inline_internal_vertical"
    );
    DefEnvironment!("{enumerate*} EnumitemKeyVals",
      "<ltx:inline-enumerate xml:id='#id' class='#class' cssstyle='#cssstyle'>#body</ltx:inline-enumerate>",
      before_digest => { def_macro_identity("\\makelabel{}")?; },
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("inline@enumerate", "enum", true, kv.as_ref())
      },
      after_digest_body => sub[whatsit] { end_enum_itemize(whatsit) },
      // Perl #2798: inline lists stay inside the surrounding paragraph.
      mode => "inline_internal_vertical"
    );
    DefEnvironment!("{description*} EnumitemKeyVals",
      "<ltx:inline-description xml:id='#id' class='#class' cssstyle='#cssstyle'>#body</ltx:inline-description>",
      properties => sub[args] {
        let kv = extract_keyvals(args);
        begin_enum_itemize("inline@description", "@desc", false, kv.as_ref())
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

  // \setlist[names]{keyvals} — Perl: enumitem.sty.ltxml L210-221, which took the first name as
  // the list and the rest as its levels. enumitem.sty:1674-1696 `\enit@setlist@i` sorts each
  // entry into lists and levels (`enumitem_is_list_name`), defaults them to `list` and level 0,
  // and stores the keys for every list at every level; `\setlist[itemize,enumerate]` sets both
  // lists (2605.00593). `\setlist*` appends to the stored keys where `\setlist` replaces them
  // (enumitem.sty:1597-1612 `\enit@saveset`); Perl's binding had no star, so
  // `\setlist*[inlinelist,1]{…}` (hep-text.sty L74) errored and leaked the `*`.
  // The names are expanded before they are sorted (enumitem.sty:1678 `\protected@edef\enit@a{#2}`):
  // `\def\@listenv{enumerate}\setlist[\@listenv,\@listdep]{…}` (2605.31510's clevethm.sty).
  DefPrimitive!("\\setlist OptionalMatch:* Optional RequiredKeyVals:enumitem", sub[(star, names, kv)] {
    let names = match names {
      Some(t) => Expand!(t).to_string(),
      None => String::new(),
    };
    let mut lists: Vec<String> = Vec::new();
    let mut levels: Vec<i64> = Vec::new();
    for entry in names.split(',').map(str::trim).filter(|e| !e.is_empty()) {
      if enumitem_is_list_name(entry) {
        lists.push(entry.to_string());
      } else if let Ok(level) = entry.parse::<i64>() {
        levels.push(level);
      } else {
        // `\setcounter{enit@cnt}{<entry>}` (enumitem.sty:1598): not a number.
        Error!("expected", "\\setlist", &s!("'{entry}' is neither a list nor a level"));
      }
    }
    if lists.is_empty() {
      lists.push("list".to_string());
    }
    if levels.is_empty() {
      levels.push(0);
    }
    for list in &lists {
      for &level in &levels {
        let key = enumitem_defaults_key(list, level);
        // `\enit@saveset` (enumitem.sty:1597-1612) stores with a local `\def`: a `\setlist` in a
        // group ends with it.
        if star.is_none() {
          assign_value(&s!("{key}@keys"), Stored::String(pin("")), None);
        }
        store_enumitem_defaults(&key, &kv, None);
      }
    }
  });

  // Obsolete shorthands
  // enumitem.sty:1700-1705: `\newcommand\setenumerate[1][0]{\setlist[enumerate,#1]}` — the level
  // defaults to 0, all levels (2605.01646 `\setenumerate[0]{…}`).
  DefMacro!("\\setitemize[Default:0]", "\\setlist[itemize,#1]");
  DefMacro!("\\setenumerate[Default:0]", "\\setlist[enumerate,#1]");
  DefMacro!("\\setdescription[Default:0]", "\\setlist[description,#1]");
  DefMacro!("\\setdisplayed[Default:0]", "\\setlist[trivlist,#1]");

  // \restartlist — Perl enumitem.sty.ltxml L128-140 uses `DefMacro` with a
  // side-effect sub returning undef (empty expansion). Match that kind:
  // macro-level so the reset happens during gullet expansion, consistent
  // with how Perl dispatches `\restartlist` inside `\begin{enumerate}`.
  // enumitem.sty:421-426's body reaches a `\let` before it acts, where a number
  // scan ends; this closure reset the counters in the scan's look-ahead
  // (`peeks_by_futurelet`; Perl Fatals there). Guard:
  // `sweep125_roots::binding_assignments_wait_for_a_number_scan`.
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
  }, peeks_by_futurelet => true);

  // Not-yet-handled bits
  def_macro_noop("\\SetLabelAlign{}{}")?;
  def_macro_noop("\\EnumitemId")?;
  def_macro_noop("\\SetEnumitemKey{}{}")?;
  def_macro_noop("\\SetEnumerateShortLabel{}{}")?;
  def_macro_noop("\\SetEnumitemValue{}{}{}")?;
  def_macro_noop("\\SetEnumitemSize{}{}")?;
  // enumitem.sty:575-598 `\@ifstar\enit@addcounter@s\enit@addcounter`: the command and its
  // internal form join the label list (a local `\edef`, :582), starred or not. Perl's `{}{}{}`
  // (enumitem.sty.ltxml:255) read the star as the command and typeset the width sample ("9").
  // `\lx@enit@normlabel` is enumitem.sty:910-936 `\enit@normlabel`, reading the counter name from
  // its first argument where enumitem reads `\@enumctr`, and leaving its result in `\lx@enit@label`.
  RawTeX!(r"\def\lx@enit@labellist{\lx@enit@elt\arabic\@arabic\lx@enit@elt\alph\@alph
  \lx@enit@elt\Alph\@Alph\lx@enit@elt\roman\@roman\lx@enit@elt\Roman\@Roman}
\def\lx@enit@addcounter#1#2{\expandafter\def\expandafter\lx@enit@labellist\expandafter{%
  \lx@enit@labellist\lx@enit@elt#1#2}}
\def\lx@enit@refstar@i#1#2{\if*#2\@empty\noexpand#1{\lx@enit@ctr}\else\noexpand#1{#2}\fi}
\def\lx@enit@refstar@ii#1#2{\if*#2\@empty\noexpand\the\noexpand#1{\lx@enit@ctr}\else
  \noexpand\the\noexpand#1{#2}\fi}
\def\lx@enit@refstar#1#2{\def#1{\lx@enit@refstar@i#1}\def#2{\lx@enit@refstar@i#2}}
\def\lx@enit@normlabel#1#2{\begingroup\def\lx@enit@ctr{#1}%
  \def\value{\lx@enit@refstar@ii\value}\let\lx@enit@elt\lx@enit@refstar\lx@enit@labellist
  \protected@xdef\lx@enit@label{{#2}}\endgroup}");
  DefMacro!("\\AddEnumerateCounter OptionalMatch:* {}{}{}", "\\lx@enit@addcounter{#2}{#3}");

  // enumitem `\setlistdepth{n}`: the list-depth budget is presentation-only for XML.
  // `\renewlist` stays `\newlist` (above; enumitem.sty:1730-1731 run the same `\enit@newlist`), so
  // its list is registered for `\setlist[<name>]` — ctxdoc.cls `\renewlist{tablenotes}`, fdudoc.cls.
  def_macro_noop("\\setlistdepth{}")?;
});

#[cfg(test)]
mod tests {
  use super::css_length;

  /// The `leftmargin` → CSS-length sanitizer (issue #559): keep number+unit and
  /// bare `0`, drop everything else (bare non-zero, raw TeX macros/`\dimexpr`) so
  /// nothing invalid reaches the stylesheet's `--ltx-enum-leftmargin`.
  #[test]
  fn css_length_accepts_only_safe_lengths() {
    for ok in [
      "2em", "0pt", "3cm", "1.5em", "10px", "50%", "0.25in", "-1em",
    ] {
      assert_eq!(css_length(ok), Some(ok.to_string()), "{ok} should pass");
    }
    assert_eq!(css_length("0"), Some("0".to_string()));
    for bad in [
      "*",
      "\\parindent",
      "\\dimexpr 2em",
      "5",
      "auto",
      "",
      "em",
      "2 em",
    ] {
      assert_eq!(css_length(bad), None, "{bad:?} should be rejected");
    }
  }
}

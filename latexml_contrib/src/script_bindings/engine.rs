//! The sandboxed Rhai engine: every `register_fn` of the binding-language
//! surface (state, tokens, counters, definitions, package/class machinery,
//! document/whatsit proxies). Split from `mod.rs` for navigability; each
//! registration lowers to the same native function its compile-time macro
//! does (`setup_binding_language.rs`).

use super::*;

/// Mini-DSL to expose a `document.<rhai>(qname)` → `Document::<rust>(qname)`
/// method whose result is discarded — the common side-effect-on-element shape.
/// Adding a new such method is then one line; see the registrations in
/// `make_engine`.
macro_rules! doc_qname_method {
  ($engine:ident, $rhai:literal, $rust:ident) => {
    $engine.register_fn(
      $rhai,
      |_d: &mut DocProxy, qname: &str| -> std::result::Result<(), Box<EvalAltResult>> {
        with_doc(|doc, _props| {
          doc.$rust(qname).map_err(rhai_err)?;
          Ok(())
        })
      },
    );
  };
}

/// Build a sandboxed Rhai engine with the binding API registered.
pub(super) fn make_engine() -> Engine {
  let mut engine = Engine::new();
  engine.set_max_operations(MAX_OPERATIONS);
  engine.set_max_call_levels(MAX_CALL_LEVELS);
  engine.set_max_string_size(MAX_STRING_SIZE);

  // ── registration API (wired to native defs IMMEDIATELY, in script order) ──
  engine.register_fn(
    "DefMacro",
    |proto: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_macro(e, a, proto, body))
    },
  );
  // String-body form — Perl's most common `DefMacro('\foo', 'bar')`, where the
  // body is TeX source that becomes the expansion directly (no Rhai closure).
  // Wires the same native expandable as the compile-time `DefMacro!` string
  // form, so a `.rhai` binding reads like the `.ltxml` original. The wiring
  // ignores the script handles, hence the throwaway `|_, _|`.
  engine.register_fn(
    "DefMacro",
    |proto: &str, body: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|_, _| wire_macro_string(proto, body))
    },
  );
  // Option-bag form (Perl's trailing `key => value`s): scalars onto
  // `ExpandableOptions` via the shared mapper. Both the closure-body and the
  // string-body variants accept it.
  engine.register_fn(
    "DefMacro",
    |proto: &str, body: FnPtr, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_macro_opts(e, a, proto, body, opts))
    },
  );
  engine.register_fn(
    "DefMacro",
    |proto: &str, body: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|_, _| wire_macro_string_opts(proto, body, opts))
    },
  );
  engine.register_fn(
    "DefPrimitive",
    |proto: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_primitive(e, a, proto, body, PrimitiveOptions::default()))
    },
  );
  engine.register_fn(
    "DefPrimitive",
    |proto: &str, body: FnPtr, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_primitive(e, a, proto, body, primitive_options_from_map(opts)))
    },
  );
  // Class/package option. Mirrors the `DeclareOption!` macro's lowering
  // (setup_binding_language.rs): push the name onto `@declaredoptions` and
  // define a `\ds@<opt>` primitive carrying the body. Installed immediately,
  // so a following `ProcessOptions()` in the same script sees it.
  engine.register_fn(
    "DeclareOption",
    |opt: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_option(e, a, opt, body))
    },
  );
  // String-body form: `DeclareOption('opt', '\xdef...')` — the body is TeX,
  // installed as a token-expansion macro (Perl's `DeclareOption($o, $string)`).
  engine.register_fn(
    "DeclareOption",
    |opt: &str, body: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|_, _| wire_option_string(opt, body))
    },
  );
  // Default-option handler: `DeclareOption(sub {...})` with no name — Perl's
  // `DeclareOption(undef, sub {...})`, defining `\default@ds` for any option not
  // otherwise declared.
  engine.register_fn(
    "DeclareOption",
    |body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_option_default(e, a, body))
    },
  );

  // ── free-function helpers (prelude names) over core datatypes ──
  // `Tokens` is registered as an opaque pass-through handle (Tier-2): a script
  // produces it with Tokenize and hands it to Digest without inspecting it.
  engine.register_type_with_name::<Tokens>("Tokens");
  engine.register_fn("Tokenize", |s: &str| -> Tokens { mouth::tokenize(s) });
  engine.register_fn("TokenizeInternal", |s: &str| -> Tokens {
    mouth::tokenize_internal(s)
  });
  // `Digest` returns the digested handle (Perl returns the digested box), so a
  // script can `ToString(Digest(...))` — e.g. a default-option handler reading
  // `\CurrentOption`. Callers that ignore the return (the common side-effect
  // use) are unaffected.
  engine.register_fn(
    "Digest",
    |t: Tokens| -> std::result::Result<Digested, Box<EvalAltResult>> {
      latexml_core::stomach::digest(t).map_err(rhai_err)
    },
  );
  // `T_CS('\foo')` — a single control-sequence token (wrapped as `Tokens` so it
  // composes with `Digest`/`Expand`). The dominant token-constructor use.
  engine.register_fn("T_CS", |name: &str| -> Tokens {
    Tokens::new(vec![latexml_core::T_CS!(name)])
  });

  // ── state API (for primitive side-effects / value-reading macros) ──
  // `assign_value` is group-local (TeX default); `assign_global` persists past
  // the enclosing group. (Untrusted-script key-namespacing is a documented
  // follow-up — see the plan's critical re-eval §5.)
  engine.register_fn("assign_value", |key: &str, val: &str| {
    latexml_core::state::assign_value(key, val.to_string(), Some(Scope::Local));
  });
  engine.register_fn("assign_global", |key: &str, val: &str| {
    latexml_core::state::assign_value(key, val.to_string(), Some(Scope::Global));
  });
  // Curated pool helpers, registered 1:1 under their Perl/Rust names so binding
  // bodies read like the originals (e.g. `beforeDigest: || neutralize_font()`).
  engine.register_fn(
    "neutralize_font",
    latexml_engine::base_utilities::neutralize_font,
  );
  engine.register_fn("lookup_value", |key: &str| -> String {
    match latexml_core::state::lookup_value(key) {
      Some(Stored::String(s)) => arena::to_string(s),
      Some(other) => other.to_string(),
      None => String::new(),
    }
  });

  // ── the binding-language surface, 1:1 under its macro names ──
  // Each registration lowers to the SAME native function its compile-time
  // macro does (setup_binding_language.rs), so a Rhai binding reads like the
  // `.pool`/`_sty.rs` original. String values in, strings/handles out.
  engine.register_fn("AssignValue", |k: &str, v: &str| {
    latexml_core::state::assign_value(k, v.to_string(), None);
  });
  engine.register_fn("AssignValue", |k: &str, v: &str, scope: &str| {
    latexml_core::state::assign_value(k, v.to_string(), scope_of(scope));
  });
  engine.register_fn("LookupString", |k: &str| -> String {
    latexml_core::state::lookup_string(k)
  });
  // Perl-style `LookupValue`: returns the stored value in its natural shape. A
  // list value (e.g. `class_options`, a pushed queue) becomes a Rhai array —
  // mirroring Perl's `LookupValue` returning the arrayref (#315) — rather than
  // the leaked Debug repr `LookupString` used to emit. Scalars stringify as
  // before; a missing key is unit (Perl `undef`).
  engine.register_fn("LookupValue", |k: &str| -> Dynamic {
    match latexml_core::state::lookup_value(k) {
      None => Dynamic::UNIT,
      Some(v) if v.is_list() => {
        let arr: rhai::Array = v
          .list_items()
          .unwrap_or_default()
          .into_iter()
          .map(Dynamic::from)
          .collect();
        Dynamic::from_array(arr)
      },
      Some(Stored::String(s)) => arena::to_string(s).into(),
      Some(other) => other.to_string().into(),
    }
  });
  engine.register_fn("LookupNumber", |k: &str| -> i64 {
    latexml_core::state::lookup_number(k)
      .map(|n| n.0)
      .unwrap_or(0)
  });
  engine.register_fn("LookupBool", |k: &str| -> bool {
    latexml_core::state::lookup_bool(k)
  });
  engine.register_fn("LookupTokens", |k: &str| -> Tokens {
    latexml_core::state::lookup_tokens(k).unwrap_or_default()
  });
  // Catcode access: the char as a 1-char string, the catcode as its TeX int.
  engine.register_fn("LookupCatcode", |c: &str| -> i64 {
    c.chars()
      .next()
      .and_then(latexml_core::state::lookup_catcode)
      .map(|cc| cc as i64)
      .unwrap_or(12)
  });
  engine.register_fn("AssignCatcode", |c: &str, code: i64| {
    if let Some(ch) = c.chars().next() {
      latexml_core::state::assign_catcode(ch, Catcode::from(code as u8), None);
    }
  });
  // LookupMeaning: the meaning of a CS as its display string ("" if none).
  engine.register_fn("LookupMeaning", |cs: &str| -> String {
    match latexml_core::state::lookup_meaning(&latexml_core::T_CS!(cs)) {
      Some(m) => m.to_string(),
      None => String::new(),
    }
  });
  engine.register_fn("Let", |a: &str, b: &str| {
    latexml_core::state::let_i(&latexml_core::T_CS!(a), &latexml_core::T_CS!(b), None);
  });
  engine.register_fn("XEquals", |a: &str, b: &str| -> bool {
    latexml_core::state::x_equals(&latexml_core::T_CS!(a), &latexml_core::T_CS!(b))
  });
  engine.register_fn("IsDefined", |cs: &str| -> bool {
    latexml_core::binding::def::dialect::is_defined_token(&latexml_core::T_CS!(cs))
  });
  // RawTeX: process literal TeX as definitions input (the raw-`\def` escape
  // hatch every nontrivial binding uses). TeX: tokenize-internal + digest.
  engine.register_fn(
    "RawTeX",
    |text: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::stomach::raw_tex(text).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "TeX",
    |text: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::stomach::digest(mouth::tokenize_internal(text)).map_err(rhai_err)?;
      Ok(())
    },
  );
  engine.register_fn(
    "Expand",
    |t: Tokens| -> std::result::Result<Tokens, Box<EvalAltResult>> {
      latexml_core::gullet::do_expand(t).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "ExpandPartially",
    |t: Tokens| -> std::result::Result<Tokens, Box<EvalAltResult>> {
      latexml_core::gullet::do_expand_partially(t).map_err(rhai_err)
    },
  );
  engine.register_fn("UnTeX", |t: Tokens| -> String { t.untex() });
  // DigestText: digest a TeX string into a `Digested` handle — the workhorse
  // of `properties` closures that precompute content (e.g. IEEEproof's title).
  engine.register_fn(
    "DigestText",
    |s: &str| -> std::result::Result<Digested, Box<EvalAltResult>> {
      latexml_core::binding::content::digest_text(mouth::tokenize_internal(s)).map_err(rhai_err)
    },
  );
  // ToString/ToAttribute/Revert on a Digested handle (Perl ToString/Revert).
  engine.register_fn("ToString", |d: Digested| -> String { d.to_string() });
  engine.register_fn("ToAttribute", |d: Digested| -> String { d.to_attribute() });
  engine.register_fn(
    "Revert",
    |d: Digested| -> std::result::Result<Tokens, Box<EvalAltResult>> {
      d.revert().map_err(rhai_err)
    },
  );
  engine.register_fn(
    "Today",
    || -> std::result::Result<String, Box<EvalAltResult>> {
      latexml_engine::base_utilities::today().map_err(rhai_err)
    },
  );
  engine.register_fn("Warn", |cat: &str, obj: &str, msg: &str| {
    latexml_core::Warn!(cat, obj, msg.to_string());
  });
  engine.register_fn(
    "Error",
    |cat: &str, obj: &str, msg: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      // Error! escalates to Fatal past MAX_ERRORS (a latexml `Err`) — surface
      // that to the script as a Rhai error so conversion aborts cleanly.
      let res: Result<()> = (|| {
        latexml_core::Error!(cat, obj, msg.to_string());
        Ok(())
      })();
      res.map_err(rhai_err)
    },
  );
  // ── the rest of the diagnostics surface (Perl Common/Error.pm), completing
  // the Warn/Error pair above (#319). ──
  engine.register_fn("Info", |cat: &str, obj: &str, msg: &str| {
    latexml_core::Info!(cat, obj, msg.to_string());
  });
  engine.register_fn(
    "Fatal",
    |cat: &str, obj: &str, msg: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      // Perl `Fatal($cat,$obj,$msg)` logs a fatal and aborts. Surface it as a
      // Rhai error so the script/conversion aborts cleanly, carrying the
      // "cat:obj msg" identity, and note the fatal status for the run tally.
      let res: Result<()> = (|| {
        latexml_core::Fatal!(Internal, Unexpected, format!("{cat}:{obj} {msg}"));
        #[allow(unreachable_code)]
        Ok(())
      })();
      res.map_err(rhai_err)
    },
  );
  // Notes + progress reporting (Perl `NoteSTDERR`/`NoteLog`, `ProgressStep`,
  // `ProgressSpinup`/`ProgressSpindown`).
  engine.register_fn("NoteSTDERR", |msg: &str| {
    latexml_core::Note!(msg);
  });
  engine.register_fn("NoteLog", |msg: &str| {
    latexml_core::NoteLog!(msg);
  });
  engine.register_fn("ProgressStep", |note: &str| {
    latexml_core::common::error::progress_step(note);
  });
  engine.register_fn("ProgressSpinup", |stage: &str| {
    latexml_core::common::error::note_begin(stage);
  });
  engine.register_fn("ProgressSpindown", |stage: &str| {
    latexml_core::common::error::note_end(stage);
  });

  // ── counters (counter_dialect, the NewCounter!/StepCounter!/… family) ──
  engine.register_fn(
    "NewCounter",
    |c: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::new_counter(c, "", None).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "NewCounter",
    |c: &str, within: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::new_counter(c, within, None).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "StepCounter",
    |c: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::step_counter(c, false).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "ResetCounter",
    |c: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::reset_counter(&latexml_core::T_LETTER!(c))
        .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "AddToCounter",
    |c: &str, n: i64| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::add_to_counter(c, Number(n)).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "CounterValue",
    |c: &str| -> std::result::Result<i64, Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::counter_value(c)
        .map(|n| n.0)
        .map_err(rhai_err)
    },
  );
  // RefStepCounter: returns the refnum/id property map (Digested values come
  // back as handles a `properties` closure can return directly — the amsmath
  // `properties => ref_step_counter("equation")` idiom).
  engine.register_fn(
    "RefStepCounter",
    |c: &str| -> std::result::Result<Map, Box<EvalAltResult>> {
      let props =
        latexml_core::binding::counter::dialect::ref_step_counter(c, false).map_err(rhai_err)?;
      Ok(props_to_map(props))
    },
  );

  // AssignMeaning: bind a CS's meaning to another CS's current meaning by
  // name (the string form; Tokens-meaning binding goes via Let/RawTeX).
  engine.register_fn("AssignMeaning", |cs: &str, other: &str| {
    if let Some(m) = latexml_core::state::lookup_meaning(&latexml_core::T_CS!(other)) {
      latexml_core::state::assign_meaning(&latexml_core::T_CS!(cs), m, None);
    }
  });

  // AssignMapping / mapping lookup (Perl AssignMapping/LookupMapping).
  engine.register_fn("AssignMapping", |map: &str, key: &str, value: &str| {
    latexml_core::state::assign_mapping(map, key, Some(value.to_string()));
  });
  engine.register_fn("LookupMapping", |map: &str, key: &str| -> String {
    latexml_core::state::with_mapping(map, key, |meaning| match meaning {
      Some(Stored::String(s)) => arena::to_string(*s),
      Some(other) => other.to_string(),
      None => String::new(),
    })
  });

  // GetKeyVal / GetKeyVals over a keyval argument. A `KeyVals`/`OptionalKeyVals`
  // argument reaches a constructor *closure* body as a digested `KeyVals` handle
  // (or unit `()` when an optional set was omitted); it reaches other contexts
  // as its TeX-source string ("k=v,k2={v2}"). `GetKeyVal` accepts both — plus
  // unit — so a script body can `GetKeyVal(#1, "width")` uniformly. Missing key
  // (or unit) → "".
  engine.register_fn("GetKeyVal", |kv: Dynamic, key: &str| -> String {
    if let Some(d) = kv.clone().try_cast::<Digested>() {
      return match d.data() {
        latexml_core::digested::DigestedData::KeyVals(keyval) => keyval
          .get_value_digested(key)
          .map(|v| v.to_string())
          .unwrap_or_default(),
        _ => String::new(),
      };
    }
    if kv.is_string() {
      let s = kv.into_string().unwrap_or_default();
      return latexml_core::keyval::split_keyval_source(&s)
        .into_iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .unwrap_or_default();
    }
    String::new()
  });
  engine.register_fn("GetKeyVals", |kv: &str| -> Map {
    let mut m = Map::new();
    for (k, v) in latexml_core::keyval::split_keyval_source(kv) {
      m.insert(k.into(), Dynamic::from(v));
    }
    m
  });

  // RefStepID / RefCurrentID: the id-only siblings of RefStepCounter.
  engine.register_fn(
    "RefStepID",
    |c: &str| -> std::result::Result<Map, Box<EvalAltResult>> {
      let props = latexml_core::binding::counter::dialect::ref_step_id(c).map_err(rhai_err)?;
      Ok(props_to_map(props))
    },
  );
  engine.register_fn(
    "RefCurrentID",
    |c: &str| -> std::result::Result<Map, Box<EvalAltResult>> {
      let props = latexml_core::binding::counter::dialect::ref_current_id(c).map_err(rhai_err)?;
      Ok(props_to_map(props))
    },
  );

  // ── document model: namespaces + schema (class-binding essentials) ──
  engine.register_fn("RegisterNamespace", |prefix: &str, ns: &str| {
    latexml_core::common::model::register_namespace(prefix, Some(ns));
  });
  engine.register_fn("RegisterDocumentNamespace", |prefix: &str, ns: &str| {
    latexml_core::common::model::register_document_namespace(prefix, Some(ns));
  });
  engine.register_fn("RelaxNGSchema", |schema: &str| {
    latexml_core::binding::content::select_relaxng_schema(schema, None);
  });

  // ── further definition forms (DefRegister/DefKeyVal/DefMath/DefLigature) ──
  // These run when the script executes (per conversion, like a Perl .ltxml),
  // lowering to the same dialect functions their compile-time macros use.
  engine.register_fn(
    "DefRegister",
    |proto: &str, v: i64| -> std::result::Result<(), Box<EvalAltResult>> {
      let (cs, params) = parse_prototype(proto, true).map_err(rhai_err)?;
      latexml_core::binding::def::dialect::def_register(cs, params, Number(v), None)
        .map_err(rhai_err)
    },
  );
  // String value = a dimension spec ("5pt", "0.4em", …) → Dimension register.
  engine.register_fn(
    "DefRegister",
    |proto: &str, spec: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let (cs, params) = parse_prototype(proto, true).map_err(rhai_err)?;
      let dim = Dimension::new_f64(Dimension::spec_to_f64(spec).map_err(rhai_err)?);
      latexml_core::binding::def::dialect::def_register(cs, params, dim, None).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DefKeyVal",
    |keyset: &str, key: &str, vtype: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::keyval::define(latexml_core::keyval::KeyvalConfig {
        prefix: "KV",
        keyset,
        key,
        vtype,
        default: None,
        ..latexml_core::keyval::KeyvalConfig::default()
      })
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DefKeyVal",
    |keyset: &str,
     key: &str,
     vtype: &str,
     default: &str|
     -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::keyval::define(latexml_core::keyval::KeyvalConfig {
        prefix: "KV",
        keyset,
        key,
        vtype,
        default: Some(default),
        ..latexml_core::keyval::KeyvalConfig::default()
      })
      .map_err(rhai_err)
    },
  );
  // Option-bag form: `DefKeyVal(keyset, key, vtype, default, #{prefix, kind,
  // macroprefix, choices})` — the xkeyval-style declaration (a `prefix`, a
  // `kind` of ordinary/command/choice/boolean, and `choices` for choice keys).
  engine.register_fn(
    "DefKeyVal",
    |keyset: &str,
     key: &str,
     vtype: &str,
     default: &str,
     opts: Map|
     -> std::result::Result<(), Box<EvalAltResult>> {
      keyval_define_from_map(keyset, key, vtype, default, opts).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DefLigature",
    |pattern: &str, replacement: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let regex_compiled = regex::Regex::new(pattern)
        .map_err(|e| Box::<EvalAltResult>::from(format!("DefLigature bad regex: {e}")))?;
      let replacement = replacement.to_string();
      latexml_core::state::unshift_value("TEXT_LIGATURES", vec![
        latexml_core::ligature::Ligature {
          id:        latexml_core::state::generate_ligature_id(),
          regex:     Some(pattern.to_string()),
          code:      Some(Rc::new(move |text| {
            regex_compiled
              .replace_all(text, replacement.as_str())
              .to_string()
          })),
          font_test: None,
          matcher:   None,
        },
      ]);
      Ok(())
    },
  );
  // DefMath: presentation-string form (the dominant one) + option bag with the
  // macro's scalar option set.
  engine.register_fn(
    "DefMath",
    |proto: &str, presentation: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let (cs, params) = parse_prototype(proto, true).map_err(rhai_err)?;
      latexml_core::binding::def::dialect::def_math(
        cs,
        params,
        presentation.to_string(),
        MathPrimitiveOptions::default(),
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DefMath",
    |proto: &str, presentation: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      let (cs, params) = parse_prototype(proto, true).map_err(rhai_err)?;
      let options = math_options_from_map(opts).map_err(rhai_err)?;
      latexml_core::binding::def::dialect::def_math(cs, params, presentation.to_string(), options)
        .map_err(rhai_err)
    },
  );
  // DefConditional: the test is a Rhai closure receiving args as strings and
  // returning a bool.
  engine.register_fn(
    "DefConditional",
    |proto: &str, test: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_conditional(e, a, proto, test))
    },
  );
  // DefColumnType: "<char><params>" prototype; the body returns the column's
  // rewrite expansion (installs `\NC@rewrite@<char>`, as the macro does).
  engine.register_fn(
    "DefColumnType",
    |proto: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_columntype(e, a, proto, body))
    },
  );

  // DefAccent: combining/standalone accent chars + the protected applyaccent
  // macro (mirrors the DefAccent! lowering; below=true for under-accents).
  engine.register_fn(
    "DefAccent",
    |accent: &str,
     combining: &str,
     standalone: &str|
     -> std::result::Result<(), Box<EvalAltResult>> {
      def_accent_impl(accent, combining, standalone, false).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DefAccent",
    |accent: &str,
     combining: &str,
     standalone: &str,
     below: bool|
     -> std::result::Result<(), Box<EvalAltResult>> {
      def_accent_impl(accent, combining, standalone, below).map_err(rhai_err)
    },
  );

  // Read-only node proxy (closure-form matcher bodies).
  engine.register_type_with_name::<NodeProxy>("Node");
  engine.register_fn("qname", |n: &mut NodeProxy| -> String {
    latexml_core::common::model::with_node_qname(&n.0, |q| q.to_string())
  });
  engine.register_fn("content", |n: &mut NodeProxy| -> String {
    n.0.get_content()
  });
  engine.register_fn("getAttribute", |n: &mut NodeProxy, k: &str| -> String {
    n.0.get_attribute(k).unwrap_or_default()
  });
  engine.register_fn("prevSibling", |n: &mut NodeProxy| -> Dynamic {
    match n.0.get_prev_sibling() {
      Some(p) => Dynamic::from(NodeProxy(p)),
      None => Dynamic::UNIT,
    }
  });
  // Write methods (libxml handles alias the same C node, so mutation through
  // a cloned handle is the library's intended model — used by the rewrite
  // `replace` closure form, which owns its matched nodes).
  // A failed libxml write from a rewrite-replace body must NOT vanish — surface
  // it as a Rhai error the script (and the conversion log) can see (review m3).
  engine.register_fn(
    "setAttribute",
    |n: &mut NodeProxy, k: &str, v: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      n.0.set_attribute(k, v).map(|_| ()).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "setContent",
    |n: &mut NodeProxy, v: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      n.0.set_content(v).map(|_| ()).map_err(rhai_err)
    },
  );
  engine.register_fn("unlink", |n: &mut NodeProxy| {
    n.0.unlink();
  });
  engine.register_fn("parent", |n: &mut NodeProxy| -> Dynamic {
    match n.0.get_parent() {
      Some(p) => Dynamic::from(NodeProxy(p)),
      None => Dynamic::UNIT,
    }
  });

  // DefMathLigature, matcher-closure form (`matcher => sub[document,node]`):
  // the body inspects the node (and its prevSibling chain) and returns UNIT
  // for no-match, or #{ n, replacement, role?, name?, meaning? }.
  engine.register_fn(
    "DefMathLigature",
    |matcher: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_math_ligature_matcher(e, a, matcher))
    },
  );

  // DefRewrite/DefMathRewrite (data forms: xpath/select/attributes/regexp/
  // attributes-map/on_match; the `replace` closure form stays native-only).
  engine.register_fn(
    "DefRewrite",
    |opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      def_rewrite_impl("text", opts).map_err(rhai_err)
    },
  );
  // replace-closure form: xpath/select picks nodes, the Rhai body receives
  // them as an array of Node proxies and mutates in place.
  engine.register_fn(
    "DefRewrite",
    |opts: Map, replace: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_rewrite_replace("text", e, a, opts, replace))
    },
  );
  engine.register_fn(
    "DefMathRewrite",
    |opts: Map, replace: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_rewrite_replace("math", e, a, opts, replace))
    },
  );
  engine.register_fn(
    "DefMathRewrite",
    |opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      def_rewrite_impl("math", opts).map_err(rhai_err)
    },
  );

  // DefMathLigature: pattern/replacement/attrs are plain data — the XMTok
  // prev-sibling matcher is built natively (same lowering as the macro).
  engine.register_fn(
    "DefMathLigature",
    |pattern: &str, replacement: &str, opts: Map| {
      def_math_ligature_impl(pattern, replacement, opts);
    },
  );

  // ── gullet seams (Perl `$gullet->…` reads from inside macro bodies) ──
  engine.register_fn(
    "SkipSpaces",
    || -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::gullet::skip_spaces().map_err(rhai_err)
    },
  );
  // ReadArg: one balanced argument off the stream (unexpanded), as TeX source.
  engine.register_fn(
    "ReadArg",
    || -> std::result::Result<String, Box<EvalAltResult>> {
      latexml_core::gullet::read_arg(latexml_core::gullet::ExpansionLevel::Off)
        .map(|t| t.untex())
        .map_err(rhai_err)
    },
  );
  // ReadUntil(delim): tokens up to (and consuming) the delimiter TeX string.
  engine.register_fn(
    "ReadUntil",
    |delim: &str| -> std::result::Result<String, Box<EvalAltResult>> {
      latexml_core::gullet::read_until(&mouth::tokenize_internal(delim))
        .map(|t| t.untex())
        .map_err(rhai_err)
    },
  );
  // ReadOptional: a bracketed [..] optional ("" when absent).
  engine.register_fn(
    "ReadOptional",
    || -> std::result::Result<String, Box<EvalAltResult>> {
      latexml_core::gullet::read_optional(None)
        .map(|t| t.map(|tt| tt.untex()).unwrap_or_default())
        .map_err(rhai_err)
    },
  );

  // ── package/class machinery (content.rs, the RequirePackage!/… family) ──
  engine.register_fn(
    "RequirePackage",
    |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::require_package(
        name,
        latexml_core::binding::content::RequireOptions::default(),
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "RequirePackage",
    |name: &str, options: rhai::Array| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::require_package(
        name,
        latexml_core::binding::content::RequireOptions {
          options: options.into_iter().map(dynamic_to_string).collect(),
          ..latexml_core::binding::content::RequireOptions::default()
        },
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "LoadClass",
    |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::load_class(name, Vec::new(), Tokens::default())
        .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "LoadClass",
    |name: &str, options: rhai::Array| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::load_class(
        name,
        options.into_iter().map(dynamic_to_string).collect(),
        Tokens::default(),
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "ProcessOptions",
    || -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::process_options(false, &[]).map_err(rhai_err)
    },
  );
  // ProcessOptions(true) = the `\ProcessOptions*` in-order variant.
  engine.register_fn(
    "ProcessOptions",
    |inorder: bool| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::process_options(inorder, &[]).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "ExecuteOptions",
    |options: rhai::Array| -> std::result::Result<(), Box<EvalAltResult>> {
      let opts: Vec<String> = options.into_iter().map(dynamic_to_string).collect();
      let refs: Vec<&str> = opts.iter().map(String::as_str).collect();
      latexml_core::binding::content::execute_options(&refs).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "PassOptions",
    |name: &str, ext: &str, options: rhai::Array| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::pass_options(
        name,
        ext,
        options.into_iter().map(dynamic_to_string).collect(),
      )
      .map_err(rhai_err)
    },
  );
  // `InputDefinitions('name', #{type, noltxml, withoptions, handleoptions,
  // reloadable, ...})` — raw-load a `.sty`/`.cls` (the wrapper form many local
  // test fixtures use). The opts map mirrors Perl's `key => value` tail.
  engine.register_fn(
    "InputDefinitions",
    |name: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::input_definitions(name, input_def_options_from_map(opts))
        .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "InputDefinitions",
    |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::input_definitions(
        name,
        latexml_core::binding::content::InputDefinitionOptions::default(),
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn("RequireResource", |resource: &str| {
    latexml_core::binding::content::require_resource(latexml_core::document::resource::Resource {
      name: resource.to_string(),
      ..latexml_core::document::resource::Resource::default()
    });
  });
  // Tag: the scalar subset (autoOpen/autoClose) of TagOptions.
  engine.register_fn("Tag", |tag: &str, opts: Map| {
    let mut options = latexml_core::document::tag::TagOptions::default();
    for (key, val) in opts {
      match key.as_str() {
        "autoOpen" => options.auto_open = val.as_bool().ok(),
        "autoClose" => options.auto_close = val.as_bool().ok(),
        _ => {},
      }
    }
    latexml_core::binding::content::install_tag(tag, options);
  });
  // MergeFont: merge the given partial font into the current one (Perl
  // `MergeFont(family=>…)`); string keys family/series/shape/size.
  engine.register_fn("MergeFont", |opts: Map| {
    latexml_core::binding::content::merge_font(font_from_rhai_map(opts));
  });
  engine.register_fn(
    "DefConstructor",
    |proto: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_constructor(e, a, proto, body))
    },
  );
  // String-template form (the dominant constructor dialect): the second arg is
  // an XML template instead of a closure. Rhai dispatches by the arg's type.
  engine.register_fn(
    "DefConstructor",
    |proto: &str, template: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|_e, _a| wire_constructor_template(proto, template.to_string()))
    },
  );
  // Option-bag forms: a trailing Rhai object map `#{ mode: …, afterDigest: |…| … }`
  // — the analog of Perl's `%options` / the `DefConstructor!` macro's `key => value`
  // (named, any order, omittable; values may be strings or closures).
  engine.register_fn(
    "DefConstructor",
    |proto: &str, template: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| {
        wire_constructor_opts(
          e,
          a,
          proto,
          ConstructorRepl::Template(template.to_string()),
          opts,
        )
      })
    },
  );
  engine.register_fn(
    "DefConstructor",
    |proto: &str, body: FnPtr, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_constructor_opts(e, a, proto, ConstructorRepl::Closure(body), opts))
    },
  );

  // ── DefEnvironment: same four shapes as DefConstructor; the prototype is the
  // `DefEnvironment!` form (`"{name}"` / `"{name}{}…"`), the template will
  // typically reference `#body`. ──
  engine.register_fn(
    "DefEnvironment",
    |proto: &str, template: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| {
        wire_environment(
          e,
          a,
          proto,
          ConstructorRepl::Template(template.to_string()),
          Map::new(),
        )
      })
    },
  );
  engine.register_fn(
    "DefEnvironment",
    |proto: &str, body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_environment(e, a, proto, ConstructorRepl::Closure(body), Map::new()))
    },
  );
  engine.register_fn(
    "DefEnvironment",
    |proto: &str, template: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| {
        wire_environment(
          e,
          a,
          proto,
          ConstructorRepl::Template(template.to_string()),
          opts,
        )
      })
    },
  );
  engine.register_fn(
    "DefEnvironment",
    |proto: &str, body: FnPtr, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_environment(e, a, proto, ConstructorRepl::Closure(body), opts))
    },
  );

  // ── document proxy: methods mirror Perl's `$document->method` idiom ──
  // The body receives `document` (a DocProxy) as its first arg, and each digested
  // argument as an opaque `Digested` handle it can pass back to `document.absorb`.
  engine.register_type_with_name::<DocProxy>("Document");
  engine.register_type_with_name::<Digested>("Digested");

  doc_qname_method!(engine, "closeElement", close_element);
  doc_qname_method!(engine, "maybeCloseElement", maybe_close_element);

  engine.register_fn(
    "openElement",
    |_d: &mut DocProxy, tag: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        doc.open_element(tag, None, None).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  engine.register_fn(
    "setAttribute",
    |_d: &mut DocProxy, key: &str, val: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        let mut node = doc.get_node().clone();
        doc.set_attribute(&mut node, key, val).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  engine.register_fn(
    "absorbString",
    |_d: &mut DocProxy, s: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      with_doc(|doc, props| {
        doc.absorb_string(s, props).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  engine.register_fn(
    "absorb",
    |_d: &mut DocProxy, arg: Digested| -> std::result::Result<(), Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        doc.absorb(&arg, None).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  // Absorb a whatsit property at the current point — the imperative analog of a
  // template's `#name` hole at content position. The workhorse is
  // `document.absorbProperty("body")` inside an imperative `DefEnvironment`
  // (mirroring natives like `{center}`'s `sub[document, _args, props]` body).
  engine.register_fn(
    "absorbProperty",
    |_d: &mut DocProxy, name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      with_doc(|doc, props| {
        if let Some(stored) = props.get(name) {
          let dig: Option<Digested> = stored.into();
          if let Some(ref d) = dig {
            doc.absorb(d, None).map_err(rhai_err)?;
          }
        }
        Ok(())
      })
    },
  );

  // ── whatsit proxy: reached from a hook body via `whatsit()` ──
  engine.register_type_with_name::<WhatsitProxy>("Whatsit");
  engine.register_fn("whatsit", || WhatsitProxy);
  // The n-th (1-based) argument as its TeX-source string.
  engine.register_fn(
    "argString",
    |_w: &mut WhatsitProxy, n: i64| -> std::result::Result<String, Box<EvalAltResult>> {
      // SAFETY: `current_whatsit()` returns the in-flight whatsit pointer the
      // core published onto WHATSIT_CTX for the duration of THIS hook body (it
      // errors out, not UB, when called outside one). The pointer is live for
      // the call and read-only here (`&*`), so no aliasing `&mut` exists.
      let w = unsafe { &*current_whatsit()? };
      match w.get_arg(n as usize) {
        Some(d) => d.untex().map_err(rhai_err),
        None => Ok(String::new()),
      }
    },
  );
  // Set a whatsit property from a hook body (Perl `$whatsit->setProperty(k, v)`,
  // e.g. plain `\footnote`'s afterDigest routing its mark arg to `mark`/`prenote`).
  // The value lands as a string `Stored`, which the template interpreter renders
  // at attribute (`to_attribute`), content (absorb), and truth-test positions.
  engine.register_fn(
    "setProperty",
    |_w: &mut WhatsitProxy, key: &str, val: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let (ptr, mutable) = current_whatsit_entry()?;
      if !mutable {
        return Err(Box::<EvalAltResult>::from(
          "setProperty in a construction hook (whatsit is read-only there)",
        ));
      }
      // SAFETY: `ptr` is the in-flight whatsit the core published onto
      // WHATSIT_CTX for THIS digestion-hook body, and the `mutable` flag (just
      // checked) confirms it was published as writable. It is the sole live
      // `&mut` to this whatsit, so the re-mint is unique and non-aliasing.
      // AUDITED (2026-06-27, sibling of PR #248 B1): the core runs after-digest
      // hooks ONE-PASS and sequentially on a FRESH-LOCAL whatsit
      // (`definition.rs::execute_after_digest`: `for post in … { post(&mut
      // whatsit) }`), never re-entering a hook on the SAME whatsit — so unlike
      // the Document case there is no nested same-object re-mint at all. This is
      // the single-body parked-`&mut` + brief-re-mint pattern proven sound by
      // the Miri model `latexml_core::runtime_bindings_reentrancy_model`
      // (`reentrancy_model_single_body_sound`). (A hook that digests other
      // content builds DIFFERENT whatsits → different `WHATSIT_CTX` entries, not
      // an alias of this one.) The read-only ops above (`&*`) need no such
      // argument — shared refs may alias freely.
      let w = unsafe { &mut *ptr };
      w.set_property(key, val.to_string());
      Ok(())
    },
  );
  // Read a whatsit property as a string ("" when absent) — Perl `getProperty`.
  engine.register_fn(
    "propertyString",
    |_w: &mut WhatsitProxy, key: &str| -> std::result::Result<String, Box<EvalAltResult>> {
      // SAFETY: `current_whatsit()` returns the in-flight whatsit pointer the
      // core published onto WHATSIT_CTX for the duration of THIS hook body (it
      // errors out, not UB, when called outside one). The pointer is live for
      // the call and read-only here (`&*`), so no aliasing `&mut` exists.
      let w = unsafe { &*current_whatsit()? };
      Ok(w.get_property_string(key))
    },
  );

  engine
}

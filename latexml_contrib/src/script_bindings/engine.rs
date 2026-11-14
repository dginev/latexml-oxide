//! The sandboxed Rhai engine: every `register_fn` of the binding-language
//! surface (state, tokens, counters, definitions, package/class machinery,
//! document/whatsit proxies). Split from `mod.rs` for navigability; each
//! registration lowers to the same native function its compile-time macro
//! does (`setup_binding_language.rs`).

use super::*;

/// Build a sandboxed Rhai engine with the binding API registered.
pub(super) fn make_engine() -> Engine {
  let mut engine = Engine::new();
  engine.set_max_operations(MAX_OPERATIONS);
  engine.set_max_call_levels(MAX_CALL_LEVELS);
  engine.set_max_string_size(MAX_STRING_SIZE);
  // Registered operators must win over the built-ins for `()` vs a string (the deprecated string tests below).
  engine.set_fast_operators(false);

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
      wire_now(|e, a| {
        wire_primitive(
          e,
          a,
          proto,
          body,
          primitive_options_from_map(proto, opts, e, a)?,
        )
      })
    },
  );
  // Perl's I-forms (Package.pm:1157, :1291, :1453): a CS (a Token, or a string `coerceCS` reads) and a parameter-spec
  // string or `()`, no prototype parsing; each lowers to the core call its prototype form makes.
  engine.register_fn(
    "DefMacroI",
    |cs: Dynamic, params: Dynamic, body: Dynamic| -> RhaiResult<()> {
      let cs = coerce_cs("DefMacroI", cs)?;
      wire_now(|e, a| {
        wire_macro_i(
          e,
          a,
          cs,
          i_form_params("DefMacroI", &cs, params)?,
          body,
          None,
        )
      })
    },
  );
  engine.register_fn(
    "DefMacroI",
    |cs: Dynamic, params: Dynamic, body: Dynamic, opts: Map| -> RhaiResult<()> {
      let cs = coerce_cs("DefMacroI", cs)?;
      wire_now(|e, a| {
        wire_macro_i(
          e,
          a,
          cs,
          i_form_params("DefMacroI", &cs, params)?,
          body,
          Some(opts),
        )
      })
    },
  );
  let primitive_i = |cs: Dynamic, params: Dynamic, body: Dynamic, opts: Map| -> RhaiResult<()> {
    let cs = coerce_cs("DefPrimitiveI", cs)?;
    let body = match body.clone().try_cast::<FnPtr>() {
      Some(fp) => Some(fp),
      None if body.is_unit() => None,
      None => {
        return Err(
          format!(
            "DefPrimitiveI: the replacement is a closure or (), got {}",
            body.type_name()
          )
          .into(),
        );
      },
    };
    wire_now(|e, a| {
      let options = primitive_options_from_map(&cs.to_string(), opts, e, a)?;
      wire_primitive_i(
        e,
        a,
        cs,
        i_form_params("DefPrimitiveI", &cs, params)?,
        body,
        options,
      )
    })
  };
  engine.register_fn(
    "DefPrimitiveI",
    move |cs: Dynamic, params: Dynamic, body: Dynamic| primitive_i(cs, params, body, Map::new()),
  );
  engine.register_fn("DefPrimitiveI", primitive_i);
  let constructor_i = |cs: Dynamic, params: Dynamic, repl: Dynamic, opts: Map| -> RhaiResult<()> {
    let cs = coerce_cs("DefConstructorI", cs)?;
    wire_now(|e, a| {
      wire_constructor_i(
        e,
        a,
        cs,
        i_form_params("DefConstructorI", &cs, params)?,
        repl,
        opts,
      )
    })
  };
  engine.register_fn(
    "DefConstructorI",
    move |cs: Dynamic, params: Dynamic, repl: Dynamic| constructor_i(cs, params, repl, Map::new()),
  );
  engine.register_fn("DefConstructorI", constructor_i);
  // Perl's `IfCondition($if)` (Package.pm:1238): the conditional's truth, `()` (with its Error) when not one.
  engine.register_fn("IfCondition", |cs: Dynamic| -> RhaiResult<Dynamic> {
    let cs = coerce_cs("IfCondition", cs)?;
    Ok(
      latexml_core::binding::content::if_condition(&cs)
        .map_err(rhai_err)?
        .map_or(Dynamic::UNIT, Dynamic::from),
    )
  });
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
  // `assembled`: the argument is TeX the script author wrote — the runtime twin
  // of a `Tokenize!("…")` literal. A script cannot hand us a flattened `Tokens`
  // (Rhai `Tokens` is opaque and has no string conversion registered).
  engine.register_fn("Tokenize", |s: &str| -> Tokens {
    mouth::tokenize(TeXString::assembled(s.to_string()))
  });
  engine.register_fn("TokenizeInternal", |s: &str| -> Tokens {
    mouth::tokenize_internal(TeXString::assembled(s.to_string()))
  });
  // `Digest` returns the digested handle (Perl returns the digested box), so a
  // script can `ToString(Digest(...))` — e.g. a default-option handler reading
  // `\CurrentOption`. Callers that ignore the return (the common side-effect
  // use) are unaffected.
  engine.register_fn(
    "Digest",
    |t: Tokens| -> std::result::Result<Digested, Box<EvalAltResult>> {
      latexml_core::binding::content::binding_digest(t).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "Digest",
    |t: Token| -> std::result::Result<Digested, Box<EvalAltResult>> {
      latexml_core::binding::content::binding_digest(Tokens::new(vec![t])).map_err(rhai_err)
    },
  );
  // A string is TeX source, TokenizeInternal'd (Package.pm:393).
  engine.register_fn(
    "Digest",
    |s: &str| -> std::result::Result<Digested, Box<EvalAltResult>> {
      latexml_core::binding::content::binding_digest(mouth::tokenize_internal(
        TeXString::assembled(s.to_string()),
      ))
      .map_err(rhai_err)
    },
  );
  // Perl's `Tokens(@tokens)` (Tokens.pm:29): an array of `Token`s and `Tokens`, flattened; also
  // `Tokens()` and `Tokens(a, …)` with up to four items, Rhai having no variadic functions.
  engine.register_fn("Tokens", || Tokens::new(Vec::new()));
  engine.register_fn("Tokens", |a: Dynamic| tokens_of(vec![a]));
  engine.register_fn("Tokens", |a: Dynamic, b: Dynamic| tokens_of(vec![a, b]));
  engine.register_fn("Tokens", |a: Dynamic, b: Dynamic, c: Dynamic| {
    tokens_of(vec![a, b, c])
  });
  engine.register_fn(
    "Tokens",
    |a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic| tokens_of(vec![a, b, c, d]),
  );
  engine.register_fn("Tokens", tokens_of);
  // Perl's token constructors and catcode constants (Token.pm:26-37, :45-84); a Perl
  // constant (`T_MATH`, `CC_ACTIVE`) is a zero-argument function here.
  engine.register_type_with_name::<Token>("Token");
  values::register_values(&mut engine);
  // A token value in a string context (`"(" + ReadArg() + ")"`, `${…}`) is its TeX source, as `UnTeX` gives it.
  engine.register_fn("to_string", |t: &mut Token| Tokens::new(vec![*t]).untex());
  engine.register_fn("to_string", |t: &mut Tokens| t.clone().untex());
  // In an array (Rhai's display takes each element's `to_debug`), as `to_string`.
  engine.register_fn("to_debug", |t: &mut Token| Tokens::new(vec![*t]).untex());
  engine.register_fn("to_debug", |t: &mut Tokens| t.clone().untex());
  // Token.pm's and Tokens.pm's accessors: `unlist` the list's tokens (a Token's is itself), `getString` the text,
  // `getCSName` the name a definition is stored under (`{` of any spelling is `{`), `getCatcode` the catcode's int.
  engine.register_fn("unlist", |t: &mut Tokens| -> rhai::Array {
    t.clone().unlist().into_iter().map(Dynamic::from).collect()
  });
  engine.register_fn("unlist", |t: &mut Token| -> rhai::Array {
    vec![Dynamic::from(*t)]
  });
  engine.register_fn("getString", |t: &mut Token| t.with_str(str::to_string));
  engine.register_fn("getCSName", |t: &mut Token| t.with_cs_name(str::to_string));
  engine.register_fn("getCatcode", |t: &mut Token| t.get_catcode() as i64);
  engine.register_fn("T_CS", |name: &str| -> Token { latexml_core::T_CS!(name) });
  engine.register_fn("T_LETTER", |c: &str| -> Token {
    Token::new(c, Catcode::LETTER)
  });
  engine.register_fn("T_OTHER", |c: &str| -> Token {
    Token::new(c, Catcode::OTHER)
  });
  engine.register_fn("T_ACTIVE", |c: &str| -> Token {
    Token::new(c, Catcode::ACTIVE)
  });
  engine.register_fn(
    "Token",
    |s: &str, cc: i64| -> std::result::Result<Token, Box<EvalAltResult>> {
      match u8::try_from(cc) {
        Ok(cc) if cc <= Catcode::ARG as u8 => Ok(Token::new(s, Catcode::from(cc))),
        _ => Err(format!("Token: no catcode {cc}").into()),
      }
    },
  );
  engine.register_fn("Token", |s: &str| -> Token {
    Token::new(s, Catcode::OTHER)
  });
  // Token.pm:81-95: `T_COMMENT` prefixes the `%`; `T_ARG` takes 1-9 or a digit Token.
  engine.register_fn("T_COMMENT", |c: &str| -> Token {
    Token::new(format!("%{c}"), Catcode::COMMENT)
  });
  engine.register_fn("T_MARKER", |t: &str| -> Token {
    Token::new(t, Catcode::MARKER)
  });
  engine.register_fn(
    "T_ARG",
    |v: Dynamic| -> std::result::Result<Token, Box<EvalAltResult>> {
      let n = match v.clone().try_cast::<Token>() {
        Some(t) => t.to_string().parse::<i64>().unwrap_or(0),
        None => v
          .as_int()
          .map_err(|got| format!("T_ARG: expected an int or a Token, got {got}"))?,
      };
      if (1..=9).contains(&n) {
        Ok(Token::new(n.to_string(), Catcode::ARG))
      } else {
        Err(format!("T_ARG: value should be #1-#9, got {n}").into())
      }
    },
  );
  for (name, token) in [
    ("T_BEGIN", latexml_core::T_BEGIN!()),
    ("T_END", latexml_core::T_END!()),
    ("T_MATH", latexml_core::T_MATH!()),
    ("T_ALIGN", latexml_core::T_ALIGN!()),
    ("T_PARAM", latexml_core::T_PARAM!()),
    ("T_SUB", latexml_core::T_SUB!()),
    ("T_SUPER", latexml_core::T_SUPER!()),
    ("T_SPACE", latexml_core::T_SPACE!()),
    ("T_CR", latexml_core::T_CR!()),
  ] {
    engine.register_fn(name, move || -> Token { token });
  }
  for (name, cc) in [
    ("CC_ESCAPE", Catcode::ESCAPE),
    ("CC_BEGIN", Catcode::BEGIN),
    ("CC_END", Catcode::END),
    ("CC_MATH", Catcode::MATH),
    ("CC_ALIGN", Catcode::ALIGN),
    ("CC_EOL", Catcode::EOL),
    ("CC_PARAM", Catcode::PARAM),
    ("CC_SUPER", Catcode::SUPER),
    ("CC_SUB", Catcode::SUB),
    ("CC_IGNORE", Catcode::IGNORE),
    ("CC_SPACE", Catcode::SPACE),
    ("CC_LETTER", Catcode::LETTER),
    ("CC_OTHER", Catcode::OTHER),
    ("CC_ACTIVE", Catcode::ACTIVE),
    ("CC_COMMENT", Catcode::COMMENT),
    ("CC_INVALID", Catcode::INVALID),
    ("CC_CS", Catcode::CS),
    ("CC_MARKER", Catcode::MARKER),
    ("CC_ARG", Catcode::ARG),
  ] {
    engine.register_fn(name, move || -> i64 { cc as i64 });
  }

  // ── deprecated state shims (#716 §3; Perl writes `AssignValue(k, v, 'global')`): one release, then removed ──
  engine.register_fn("assign_value", |key: &str, val: &str| {
    warn_deprecated("assign_value");
    latexml_core::state::assign_value(key, val.to_string(), Some(Scope::Local));
  });
  engine.register_fn("assign_global", |key: &str, val: &str| {
    warn_deprecated("assign_global");
    latexml_core::state::assign_value(key, val.to_string(), Some(Scope::Global));
  });
  // Curated pool helpers, registered 1:1 under their Perl/Rust names so binding
  // bodies read like the originals (e.g. `beforeDigest: || neutralize_font()`).
  engine.register_fn(
    "neutralize_font",
    latexml_engine::base_utilities::neutralize_font,
  );
  engine.register_fn("lookup_value", |key: &str| -> String {
    warn_deprecated("lookup_value");
    match latexml_core::state::lookup_value(key) {
      Some(Stored::String(s)) => arena::to_string(s),
      Some(other) => other.to_string(),
      None => String::new(),
    }
  });

  // ── the binding-language surface, 1:1 under its macro names ──
  // Each registration lowers to the SAME native function its compile-time
  // macro does (setup_binding_language.rs), so a Rhai binding reads like the
  // `.pool`/`_sty.rs` original.
  // A value keeps its type both ways (Package.pm AssignValue :261 / LookupValue :257).
  engine.register_fn("AssignValue", |k: &str, v: Dynamic| {
    latexml_core::state::assign_value(k, dynamic_to_stored(v), None);
  });
  engine.register_fn("AssignValue", |k: &str, v: Dynamic, scope: &str| {
    latexml_core::state::assign_value(k, dynamic_to_stored(v), scope_of(scope));
  });
  // The float setter `LookupFloat` pairs with (#542/#543), kept beside `AssignValue` (audit §3).
  engine.register_fn("AssignFloat", |k: &str, v: f64| {
    latexml_core::state::assign_value(k, Float(v), None);
  });
  engine.register_fn("AssignFloat", |k: &str, v: f64, scope: &str| {
    latexml_core::state::assign_value(k, Float(v), scope_of(scope));
  });
  // Deprecated typed twins of `AssignValue` (#716 §3), which stores each with its type.
  for old in ["AssignString", "AssignNumber", "AssignBool"] {
    engine.register_fn(old, move |k: &str, v: Dynamic| {
      warn_deprecated(old);
      latexml_core::state::assign_value(k, dynamic_to_stored(v), None);
    });
    engine.register_fn(old, move |k: &str, v: Dynamic, scope: &str| {
      warn_deprecated(old);
      latexml_core::state::assign_value(k, dynamic_to_stored(v), scope_of(scope));
    });
  }
  // The Perl `@values` list-value family (Package.pm L265-279 -> State.pm L218):
  // a value-table key holds a queue that these push/pop/unshift/shift. Always
  // GLOBAL scope, auto-vivifying the list if absent — matching Perl, which takes
  // no scope argument. Perl's `PushValue` is untyped and the Rust State layer
  // holds any `Stored`, so `PushValue`/`UnshiftValue` take any Rhai-representable
  // value (string/int/float/bool/`Tokens`/digested handle) and `PopValue`/
  // `ShiftValue` return it with its type preserved (#540). `LookupValue` reads
  // the whole queue back as a Rhai array. As of #561 `SEARCHPATHS` is itself a
  // group-scoped value-table list (like `GRAPHICSPATHS`), so `PushValue`/
  // `UnshiftValue` reach it too; the dedicated `PrependSearchPath`/
  // `AppendSearchPath` below are the ergonomic front for input dirs.
  engine.register_fn("PushValue", |k: &str, v: Dynamic| {
    // Returns Ok(()) always; the wrong-type BUG path self-reports via Error!.
    let _ = latexml_core::state::push_value(k, dynamic_to_stored(v));
  });
  engine.register_fn("UnshiftValue", |k: &str, v: Dynamic| {
    latexml_core::state::unshift_value(k, vec![dynamic_to_stored(v)]);
  });
  // Perl's `PushValue($key, @values)` / `UnshiftValue($key, @values)` (Package.pm:265/275), up to four values.
  for arity in 2..=4 {
    register_variadic_values(&mut engine, arity);
  }
  engine.register_fn("PopValue", |k: &str| -> Dynamic {
    match latexml_core::state::pop_value(k) {
      Ok(Some(popped)) => stored_to_dynamic(popped),
      _ => Dynamic::UNIT,
    }
  });
  engine.register_fn("ShiftValue", |k: &str| -> Dynamic {
    match latexml_core::state::shift_value(k) {
      Ok(Some(shifted)) => stored_to_dynamic(shifted),
      _ => Dynamic::UNIT,
    }
  });
  // Input search paths. Perl keeps these in the `SEARCHPATHS` value and prepends
  // via `UnshiftValue`; as of #561 the Rust port does the same — `SEARCHPATHS` is
  // a group-scoped value-table list read by `find_file` — so these dedicated
  // bindings are a convenience over `PushValue`/`UnshiftValue`. Prepend = searched
  // first (Perl's `SEARCHPATHS` semantics).
  engine.register_fn("PrependSearchPath", |dir: &str| {
    latexml_core::state::search_paths_push_front(dir.to_string());
  });
  engine.register_fn("AppendSearchPath", |dir: &str| {
    latexml_core::state::add_search_path(dir.to_string());
  });
  engine.register_fn("LookupString", |k: &str| -> String {
    latexml_core::state::lookup_string(k)
  });
  // Perl's `LookupValue` (Package.pm:257): the stored value with its type — a list value (`class_options`, a
  // pushed queue) as a Rhai array (Perl's arrayref, #315), a missing key as `()` (undef).
  engine.register_fn("LookupValue", |k: &str| -> Dynamic {
    latexml_core::state::lookup_value(k).map_or(Dynamic::UNIT, stored_to_dynamic)
  });
  engine.register_fn("LookupNumber", |k: &str| -> i64 {
    latexml_core::state::lookup_number(k)
      .map(|n| n.0)
      .unwrap_or(0)
  });
  // The fractional reader `LookupNumber` cannot be: a value assigned with
  // `AssignFloat` (e.g. NOMINAL_FONT_SIZE = 10.95) would read back whole
  // through `LookupNumber`, so scripts that need the fraction use this (#542/#543).
  engine.register_fn("LookupFloat", |k: &str| -> f64 {
    latexml_core::state::lookup_float(k)
      .map(|f| f.0)
      .unwrap_or(0.0)
  });
  engine.register_fn("LookupBool", |k: &str| -> bool {
    latexml_core::state::lookup_bool(k)
  });
  // Perl's `$LaTeXML::VERSION` — the running latexml-oxide version as a bare
  // `X.Y.Z`, so a binding can gate on it (BookML does `$LaTeXML::VERSION <= …`).
  // It is OUR crate version, not the emulated Perl LaTeXML's. The top crate
  // publishes it to state (`LATEXML_VERSION`) at session init (#320); `""` if a
  // conversion hasn't been initialized (e.g. a bare unit test).
  engine.register_fn("LaTeXMLVersion", || -> String {
    latexml_core::state::lookup_string("LATEXML_VERSION")
  });
  engine.register_fn("LookupTokens", |k: &str| -> Tokens {
    latexml_core::state::lookup_tokens(k).unwrap_or_default()
  });
  // The char-code tables (Package.pm :295-381): the char as a 1-char string, the code as its TeX int, `()` for a
  // char the table does not hold (Perl undef); each Assign* takes the macro's optional scope.
  engine.register_fn("LookupCatcode", |c: &str| -> Dynamic {
    char_of(c)
      .and_then(latexml_core::state::lookup_catcode)
      .map_or(Dynamic::UNIT, |cc| Dynamic::from(cc as i64))
  });
  engine.register_fn("AssignCatcode", |c: &str, code: i64| {
    assign_code("AssignCatcode", assign_catcode, 15, c, code, "")
  });
  engine.register_fn("AssignCatcode", |c: &str, code: i64, scope: &str| {
    assign_code("AssignCatcode", assign_catcode, 15, c, code, scope)
  });
  engine.register_fn("LookupMathcode", |c: &str| {
    code_of(latexml_core::state::lookup_mathcode(c))
  });
  engine.register_fn("LookupSFcode", |c: &str| {
    code_of(char_of(c).and_then(latexml_core::state::lookup_sfcode))
  });
  engine.register_fn("LookupLCcode", |c: &str| {
    code_of(char_of(c).and_then(latexml_core::state::lookup_lccode))
  });
  engine.register_fn("LookupUCcode", |c: &str| {
    code_of(char_of(c).and_then(latexml_core::state::lookup_uccode))
  });
  engine.register_fn("LookupDelcode", |c: &str| {
    code_of(char_of(c).and_then(latexml_core::state::lookup_delcode))
  });
  for (name, assign) in [
    (
      "AssignMathcode",
      latexml_core::state::assign_mathcode::<u16> as fn(char, u16, Option<Scope>),
    ),
    ("AssignSFcode", latexml_core::state::assign_sfcode::<u16>),
    (
      "AssignLCcode",
      latexml_core::state::assign_lccode::<u16, char>,
    ),
    (
      "AssignUCcode",
      latexml_core::state::assign_uccode::<u16, char>,
    ),
    ("AssignDelcode", latexml_core::state::assign_delcode::<u16>),
  ] {
    let max = i64::from(u16::MAX);
    engine.register_fn(name, move |c: &str, code: i64| {
      assign_code(name, assign, max, c, code, "")
    });
    engine.register_fn(name, move |c: &str, code: i64, scope: &str| {
      assign_code(name, assign, max, c, code, scope)
    });
  }
  // StartSemiverbatim/EndSemiverbatim (Perl `Package.pm` exports, L1002-1008):
  // read the following content raw — the default neutralizes the math/special
  // catcodes (`^ _ ~ & $ # '`) to `OTHER` so they are literal, keeping `\ { }`
  // special so e.g. `\end{env}` still parses (#653). The canonical way to give a
  // Rhai `DefEnvironment` a verbatim body: `beforeDigest: || StartSemiverbatim()`
  // + `afterDigestBody: || EndSemiverbatim()`. Pass extra single-char strings to
  // neutralize them too (e.g. `StartSemiverbatim(["%"])`).
  engine.register_fn("StartSemiverbatim", || {
    latexml_core::state::begin_semiverbatim(None);
  });
  engine.register_fn("StartSemiverbatim", |extra: rhai::Array| {
    let chars: Vec<char> = extra
      .iter()
      .filter_map(|d| d.clone().into_string().ok())
      .filter_map(|s| s.chars().next())
      .collect();
    latexml_core::state::begin_semiverbatim(Some(&chars));
  });
  engine.register_fn(
    "EndSemiverbatim",
    || -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::state::end_semiverbatim().map_err(rhai_err)
    },
  );
  // A token argument is a string naming a control sequence, or a `Token` as is (Perl `Let`, Package.pm:383-390;
  // `IsDefined`, :330-336), through the core calls the `Let!`/`XEquals!`/`IsDefined!` macros make (#796).
  // Perl's `LookupMeaning` (State.pm:370): the definition (a `Definition` handle) or the Token, `()` when undefined.
  engine.register_fn("LookupMeaning", |cs: Dynamic| -> RhaiResult<Dynamic> {
    Ok(
      latexml_core::state::lookup_meaning(&token_arg("LookupMeaning", cs)?)
        .map_or(Dynamic::UNIT, stored_to_dynamic),
    )
  });
  engine.register_fn(
    "Let",
    |a: Dynamic, b: Dynamic| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::state::let_definition(&token_arg("Let", a)?, &token_arg("Let", b)?, None);
      Ok(())
    },
  );
  // Perl `Let($token1, $token2, $scope)` (Package.pm:383): "local"/"global", as `AssignValue` (#796).
  engine.register_fn(
    "Let",
    |a: Dynamic, b: Dynamic, scope: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::state::let_definition(
        &token_arg("Let", a)?,
        &token_arg("Let", b)?,
        scope_of(scope),
      );
      Ok(())
    },
  );
  engine.register_fn(
    "XEquals",
    |a: Dynamic, b: Dynamic| -> std::result::Result<bool, Box<EvalAltResult>> {
      Ok(latexml_core::state::x_equals(
        &token_arg("XEquals", a)?,
        &token_arg("XEquals", b)?,
      ))
    },
  );
  engine.register_fn(
    "IsDefined",
    |cs: Dynamic| -> std::result::Result<bool, Box<EvalAltResult>> {
      Ok(latexml_core::binding::def::dialect::is_defined_token(
        &token_arg("IsDefined", cs)?,
      ))
    },
  );
  // RawTeX: process literal TeX as definitions input (the raw-`\def` escape
  // hatch every nontrivial binding uses). TeX: tokenize-internal + digest.
  engine.register_fn(
    "RawTeX",
    |text: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::stomach::raw_tex(text).map_err(rhai_err)
    },
  );
  // Document-lifecycle hooks (Perl `Package.pm` `AtBeginDocument`/`AtEndDocument`,
  // L2815/L2830). Each pushes a Tokens body onto the value-table list the engine
  // digests at `\begin{document}`/`\end{document}` — mirroring the native macros
  // in `latex_constructs.rs` (which `push_value` onto `@at@begin@document`/
  // `@at@end@document`). Accept TeX source (via `TokenizeInternal`, so `@` is a
  // letter as Perl's string path does) or a `Tokens` value (#539). The Perl
  // CODE-ref form is not exposed; a script can `DefMacro` a CS and hook that.
  // `AtEndOfPackage`/`AtEndOfClass`/`AtBeginDvi` are TeX-macro-level (`AddToMacro`
  // / no-op), reachable via `RawTeX`.
  engine.register_fn(
    "AtBeginDocument",
    |text: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::state::push_value(
        "@at@begin@document",
        mouth::tokenize_internal(TeXString::assembled(text.to_string())),
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "AtBeginDocument",
    |tokens: Tokens| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::state::push_value("@at@begin@document", tokens).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "AtBeginDocument",
    |token: Token| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::state::push_value("@at@begin@document", Tokens::new(vec![token]))
        .map_err(rhai_err)
    },
  );
  // The closure form (Package.pm:2815): the code becomes a macro, whose token is pushed, as Perl's `DefMacroI`.
  engine.register_fn(
    "AtBeginDocument",
    |body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      let cs = latexml_core::T_CS!(code_macro_name().as_str());
      wire_now(|e, a| wire_macro_token(e, a, cs, None, body))?;
      latexml_core::state::push_value("@at@begin@document", Tokens::new(vec![cs])).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "AtEndDocument",
    |text: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::state::push_value(
        "@at@end@document",
        mouth::tokenize_internal(TeXString::assembled(text.to_string())),
      )
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "AtEndDocument",
    |tokens: Tokens| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::state::push_value("@at@end@document", tokens).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "AtEndDocument",
    |token: Token| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::state::push_value("@at@end@document", Tokens::new(vec![token]))
        .map_err(rhai_err)
    },
  );
  // The closure form (Package.pm:2815): the code becomes a macro, whose token is pushed, as Perl's `DefMacroI`.
  engine.register_fn(
    "AtEndDocument",
    |body: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      let cs = latexml_core::T_CS!(code_macro_name().as_str());
      wire_now(|e, a| wire_macro_token(e, a, cs, None, body))?;
      latexml_core::state::push_value("@at@end@document", Tokens::new(vec![cs])).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "TeX",
    |text: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::stomach::digest(mouth::tokenize_internal(TeXString::assembled(
        text.to_string(),
      )))
      .map_err(rhai_err)?;
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
  engine.register_fn(
    "Expand",
    |t: Token| -> std::result::Result<Tokens, Box<EvalAltResult>> {
      latexml_core::gullet::do_expand(Tokens::new(vec![t])).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "ExpandPartially",
    |t: Token| -> std::result::Result<Tokens, Box<EvalAltResult>> {
      latexml_core::gullet::do_expand_partially(Tokens::new(vec![t])).map_err(rhai_err)
    },
  );
  engine.register_fn("UnTeX", |t: Token| -> String {
    Tokens::new(vec![t]).untex()
  });
  // DigestText: digest a TeX string into a `Digested` handle — the workhorse
  // of `properties` closures that precompute content (e.g. IEEEproof's title).
  // A string is TeX source, TokenizeInternal'd; a `Token`/`Tokens` is digested as is (Package.pm:405).
  engine.register_fn(
    "DigestText",
    |s: &str| -> std::result::Result<Digested, Box<EvalAltResult>> {
      latexml_core::binding::content::digest_text(mouth::tokenize_internal(TeXString::assembled(
        s.to_string(),
      )))
      .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DigestText",
    |t: Tokens| -> std::result::Result<Digested, Box<EvalAltResult>> {
      latexml_core::binding::content::digest_text(t).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DigestText",
    |t: Token| -> std::result::Result<Digested, Box<EvalAltResult>> {
      latexml_core::binding::content::digest_text(Tokens::new(vec![t])).map_err(rhai_err)
    },
  );
  // ToString/ToAttribute/Revert on a Digested handle (Perl ToString/Revert).
  // Perl's `ToString` of anything (Object.pm:59): a box, Tokens, a Token, a string; `()` as "".
  engine.register_fn("ToString", |d: Dynamic| -> String {
    if d.is_unit() {
      String::new()
    } else {
      dynamic_to_string(d)
    }
  });
  // Perl's `if ($v)` / `unless ($v)` as `!!v` / `!v` (perl_truthy): every value type, Rhai's `!` taking only a bool.
  engine.register_fn("!", |v: Dynamic| !perl_truthy(&v));
  // A value tested against a string (`w != ""`), which Rhai would answer by type alone: their string forms (`()` as
  // ""), Warned once per load with the Perl-form test; Perl compares an object's address, so tests `ToString`.
  register_string_tests::<Tokens>(&mut engine, "value == string");
  register_string_tests::<Token>(&mut engine, "value == string");
  register_string_tests::<Digested>(&mut engine, "value == string");
  register_string_tests::<DefinitionProxy>(&mut engine, "value == string");
  // `()` against a string either way round reads as "", as Perl's `undef eq ''` (which Warns `uninitialized`).
  let unit_test = |s: &str| {
    warn_deprecated("() == string");
    s.is_empty()
  };
  engine.register_fn("==", move |_: (), s: &str| unit_test(s));
  engine.register_fn("!=", move |_: (), s: &str| !unit_test(s));
  engine.register_fn("==", move |s: &str, _: ()| unit_test(s));
  engine.register_fn("!=", move |s: &str, _: ()| !unit_test(s));
  // Perl's `defined($v)`: every value but `()`.
  engine.register_fn("defined", |v: Dynamic| !v.is_unit());
  engine.register_fn("ToString", |_w: WhatsitProxy| whatsit_string());
  engine.register_fn("ToAttribute", |d: Digested| -> String { d.to_attribute() });
  // Perl's `Box($string, $font, $locator, $tokens, %properties)` (Box.pm:29): a text box, in the current font when
  // `font` is `()`, at the gullet's locator (a script has no locator: `()`), its tokens `T_OTHER($string)` when `()`.
  engine.register_fn("Box", |s: &str| {
    text_box(s, Dynamic::UNIT, Dynamic::UNIT, Map::new())
  });
  engine.register_fn("Box", |s: &str, font: Dynamic| {
    text_box(s, font, Dynamic::UNIT, Map::new())
  });
  engine.register_fn(
    "Box",
    |s: &str, font: Dynamic, _loc: (), tokens: Dynamic| text_box(s, font, tokens, Map::new()),
  );
  engine.register_fn(
    "Box",
    |s: &str, font: Dynamic, _loc: (), tokens: Dynamic, props: Map| {
      text_box(s, font, tokens, props)
    },
  );
  // Perl's `List(@boxes, mode => $mode)` (List.pm:31): one box (of the asked mode) is itself, else a List in the mode.
  engine.register_fn("List", |boxes: rhai::Array| list_of(boxes, None));
  engine.register_fn(
    "List",
    |boxes: rhai::Array, opts: Map| -> RhaiResult<Digested> {
      let mode = match opts.get("mode") {
        Some(m) if opts.len() == 1 => m.to_string(),
        _ => return Err("List: the one option is mode".into()),
      };
      list_of(boxes, Some(mode))
    },
  );
  // Perl's `$box->getFont` (Box.pm, Whatsit.pm): a `Font` handle, `()` for a box without one.
  engine.register_fn("getFont", |d: &mut Digested| -> RhaiResult<Dynamic> {
    Ok(font_handle(d.get_font().map_err(rhai_err)?))
  });
  engine.register_fn("getFont", |_w: &mut WhatsitProxy| -> RhaiResult<Dynamic> {
    with_whatsit(|w| w.get_font().map(font_handle).map_err(rhai_err))?
  });
  engine.register_fn(
    "Revert",
    |d: Digested| -> std::result::Result<Tokens, Box<EvalAltResult>> {
      d.revert().map_err(rhai_err)
    },
  );
  // Perl's Revert of a string Explodes it; of undef, nothing (Object.pm:108).
  engine.register_fn("Revert", |s: &str| explode(s, false));
  engine.register_fn("Revert", |_: ()| Tokens::new(Vec::new()));
  engine.register_fn(
    "Today",
    || -> std::result::Result<String, Box<EvalAltResult>> {
      latexml_engine::base_utilities::today().map_err(rhai_err)
    },
  );
  // Diagnostics (Perl Common/Error.pm :271-394): `cat:obj msg`, plus trailing details as strings or an array of
  // strings, one line each. Perl's form `(cat, obj, $where, msg, @details)` is read when `$where` is `()`, a handle,
  // or any 5+-argument call; a string third argument in a 4-argument call stays the message (the older form).
  for kind in [Diag::Warn, Diag::Error, Diag::Info, Diag::Fatal] {
    let name = kind.name();
    engine.register_fn(name, move |cat: &str, obj: &str, msg: &str| {
      emit_diag(kind, cat, obj, msg, None)
    });
    engine.register_fn(
      name,
      move |cat: &str, obj: &str, a3: Dynamic, a4: Dynamic| {
        if a3.is_string() {
          emit_diag(kind, cat, obj, &dynamic_to_string(a3), Some(details_of(a4)))
        } else {
          emit_diag(kind, cat, obj, &dynamic_to_string(a4), None)
        }
      },
    );
    engine.register_fn(
      name,
      move |cat: &str, obj: &str, _where: Dynamic, msg: Dynamic, d1: Dynamic| {
        emit_diag(
          kind,
          cat,
          obj,
          &dynamic_to_string(msg),
          Some(details_of(d1)),
        )
      },
    );
    engine.register_fn(
      name,
      move |cat: &str, obj: &str, _where: Dynamic, msg: Dynamic, d1: Dynamic, d2: Dynamic| {
        let details = format!("{}\n\t{}", details_of(d1), details_of(d2));
        emit_diag(kind, cat, obj, &dynamic_to_string(msg), Some(details))
      },
    );
  }
  // Notes + progress reporting (Perl `Note`/`NoteSTDERR`/`NoteLog`, `ProgressStep`,
  // `ProgressSpinup`/`ProgressSpindown`). Matching Perl `Common/Error.pm`: `Note`
  // writes to BOTH the log and stderr, `NoteSTDERR` to stderr only, `NoteLog` to
  // the log only. (#593: NoteLog/NoteSTDERR previously reached neither the log —
  // both macros only wrote stderr — nor the right stream.)
  engine.register_fn("Note", |msg: &str| {
    latexml_core::Note!(msg);
  });
  engine.register_fn("NoteSTDERR", |msg: &str| {
    latexml_core::NoteSTDERR!(msg);
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

  // ── external commands: a thin mirror of `std::process::Command` (#318). Lets
  // a TRUSTED binding shell out (BookML's `latexmk`/`dvisvgm` during digestion),
  // as Perl `.ltxml` does with `system()`. Method names mirror std exactly, so a
  // user reads the `std::process::Command` docs. SAFETY.md: the runtime-bindings
  // feature is the trust boundary — untrusted deployments drop it. Runs to
  // completion like std/`system()` (no hidden timeout); the overall conversion
  // timeout is the outer bound. ──
  engine.register_type_with_name::<RhaiCommand>("Command");
  engine.register_fn("Command", |program: &str| RhaiCommand {
    program: program.to_string(),
    ..RhaiCommand::default()
  });
  engine.register_fn("arg", |c: &mut RhaiCommand, a: &str| {
    c.args.push(a.to_string())
  });
  engine.register_fn("args", |c: &mut RhaiCommand, a: rhai::Array| {
    c.args.extend(a.into_iter().map(dynamic_to_string));
  });
  engine.register_fn("env", |c: &mut RhaiCommand, k: &str, v: &str| {
    c.envs.push((k.to_string(), v.to_string()));
  });
  engine.register_fn("current_dir", |c: &mut RhaiCommand, dir: &str| {
    c.cwd = Some(dir.to_string());
  });
  // `output()` mirrors std's: runs to completion, capturing stdout/stderr.
  // Returns `std::process::Output` flattened for Rhai: `ExitStatus` → `status`
  // (exit code, -1 on signal) + `success`; `Vec<u8>` → lossy `String`. A spawn
  // failure (program not found) surfaces as a Rhai error, mirroring std's
  // `io::Result::Err` (catchable with `try`/`catch`).
  engine.register_fn(
    "output",
    |c: &mut RhaiCommand| -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      // Allowed by default (Perl `.ltxml` runs `system()` freely), but BLOCKABLE
      // via `LATEXML_DISABLE_SHELL_ESCAPE` — the opt-out an untrusted-input
      // deployment (e.g. `cortex_worker`) sets, since a `.rhai` beside the
      // source auto-loads (converter.rs `rhai_dispatch`) and could otherwise
      // shell out. See SAFETY.md.
      if std::env::var_os("LATEXML_DISABLE_SHELL_ESCAPE").is_some() {
        return Err(rhai_err(format!(
          "Command '{}': external commands from bindings are disabled \
           (LATEXML_DISABLE_SHELL_ESCAPE is set)",
          c.program
        )));
      }
      let mut cmd = std::process::Command::new(&c.program);
      cmd.args(&c.args);
      for (k, v) in &c.envs {
        cmd.env(k, v);
      }
      if let Some(dir) = &c.cwd {
        cmd.current_dir(dir);
      }
      let out = cmd
        .output()
        .map_err(|e| rhai_err(format!("Command '{}': {e}", c.program)))?;
      let mut m = Map::new();
      m.insert(
        "status".into(),
        (out.status.code().unwrap_or(-1) as i64).into(),
      );
      m.insert("success".into(), out.status.success().into());
      m.insert(
        "stdout".into(),
        String::from_utf8_lossy(&out.stdout).into_owned().into(),
      );
      m.insert(
        "stderr".into(),
        String::from_utf8_lossy(&out.stderr).into_owned().into(),
      );
      Ok(m.into())
    },
  );

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
  // Perl `NewCounter($ctr, $within, idprefix => …, idwithin => …, nested => [...])` (Package.pm:676), as `NewCounter!`.
  engine.register_fn(
    "NewCounter",
    |c: &str, within: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      let (mut idprefix, mut idwithin, mut nested) = (String::new(), String::new(), Vec::new());
      for (key, val) in opts {
        match key.as_str() {
          "idprefix" => idprefix = dynamic_to_string(val),
          "idwithin" => idwithin = dynamic_to_string(val),
          "nested" if val.is_array() => {
            nested = val
              .cast::<rhai::Array>()
              .into_iter()
              .map(dynamic_to_string)
              .collect()
          },
          "nested" => nested = vec![dynamic_to_string(val)],
          // (Perl's NewCounter, Package.pm:654, checks no option names)
          _ => {},
        }
      }
      let options = latexml_core::binding::counter::dialect::NewCounterOptions {
        idprefix: &idprefix,
        idwithin: &idwithin,
        nested:   nested.iter().map(String::as_str).collect(),
      };
      latexml_core::binding::counter::dialect::new_counter(c, within, Some(options))
        .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "StepCounter",
    |c: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::step_counter(c, false).map_err(rhai_err)
    },
  );
  // Perl `StepCounter($ctr, $noreset)` (Package.pm:739).
  engine.register_fn(
    "StepCounter",
    |c: &str, noreset: Dynamic| -> RhaiResult<()> {
      latexml_core::binding::counter::dialect::step_counter(c, perl_truthy(&noreset))
        .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "ResetCounter",
    |c: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::reset_counter(&latexml_core::T_OTHER!(c))
        .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "AddToCounter",
    |c: &str, n: i64| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::counter::dialect::add_to_counter(c, Number(n)).map_err(rhai_err)
    },
  );
  // Perl's form takes a Number (Package.pm:730).
  engine.register_fn("AddToCounter", |c: &str, n: Number| -> RhaiResult<()> {
    latexml_core::binding::counter::dialect::add_to_counter(c, n).map_err(rhai_err)
  });
  // A Number, as Perl's (Package.pm:706); its int-like uses (`+ 1`, `== 3`) Warn for one release (values.rs).
  engine.register_fn("CounterValue", |c: &str| -> RhaiResult<Number> {
    latexml_core::binding::counter::dialect::counter_value(c).map_err(rhai_err)
  });
  // RefStepCounter: returns the refnum/id property map (Digested values come
  // back as handles a `properties` closure can return directly — the amsmath
  // `properties => ref_step_counter("equation")` idiom). Through `\refstepcounter`'s
  // current meaning, as `RefStepCounter!`; `noreset` steps the counter directly.
  engine.register_fn(
    "RefStepCounter",
    |c: &str| -> std::result::Result<Map, Box<EvalAltResult>> {
      let props = latexml_core::binding::counter::dialect::ref_step_counter_by_meaning(c)
        .map_err(rhai_err)?;
      Ok(props_to_map(props))
    },
  );
  engine.register_fn(
    "RefStepCounter",
    |c: &str, noreset: bool| -> std::result::Result<Map, Box<EvalAltResult>> {
      let props = if noreset {
        latexml_core::binding::counter::dialect::ref_step_counter(c, true)
      } else {
        latexml_core::binding::counter::dialect::ref_step_counter_by_meaning(c)
      }
      .map_err(rhai_err)?;
      Ok(props_to_map(props))
    },
  );

  // Perl `$STATE->assignMeaning($token, $meaning, $scope)` (State.pm:380), the `AssignMeaning!` macro: the definition a
  // `LookupDefinition` handle found (as Perl's object, unchanged by a later redefinition), a Token (`\let\x=a`), or `()` (undefines). A string
  // meaning is the old `Let` form, kept one release with a Warning.
  fn assign_meaning(
    cs: Dynamic,
    meaning: Dynamic,
    scope: Option<Scope>,
  ) -> std::result::Result<(), Box<EvalAltResult>> {
    let cs = token_arg("AssignMeaning", cs)?;
    if meaning.is_string() {
      warn_deprecated("AssignMeaning(cs, string)");
      latexml_core::state::let_definition(&cs, &token_arg("AssignMeaning", meaning)?, scope);
    } else if meaning.is_unit() {
      latexml_core::state::assign_meaning(&cs, Stored::None, scope);
    } else if let Some(def) = meaning.clone().try_cast::<DefinitionProxy>() {
      latexml_core::state::assign_meaning(&cs, def.meaning, scope);
    } else {
      latexml_core::state::assign_meaning(&cs, token_arg("AssignMeaning", meaning)?, scope);
    }
    Ok(())
  }
  engine.register_fn("AssignMeaning", |cs: Dynamic, meaning: Dynamic| {
    assign_meaning(cs, meaning, None)
  });
  engine.register_fn(
    "AssignMeaning",
    |cs: Dynamic, meaning: Dynamic, scope: &str| assign_meaning(cs, meaning, scope_of(scope)),
  );

  // AssignMapping / LookupMapping / LookupMappingKeys (Package.pm :283-293): a value keeps its type, as
  // AssignValue's; a missing key is `()` (undef); the keys come sorted, as State.pm:261-265 sorts them.
  engine.register_fn("AssignMapping", |map: &str, key: &str, value: Dynamic| {
    let value = (!value.is_unit()).then(|| dynamic_to_stored(value));
    latexml_core::state::assign_mapping(map, key, value);
  });
  engine.register_fn("LookupMapping", |map: &str, key: &str| -> Dynamic {
    latexml_core::state::lookup_mapping(map, key).map_or(Dynamic::UNIT, stored_to_dynamic)
  });
  engine.register_fn("LookupMappingKeys", |map: &str| -> rhai::Array {
    let mut keys: Vec<String> = latexml_core::state::with_mapping_keys(map, |keys| {
      keys.iter().map(|k| arena::to_string(*k)).collect()
    });
    keys.sort();
    keys.into_iter().map(Dynamic::from).collect()
  });

  // GetKeyVal / GetKeyVals (KeyVals.pm:31-37): the value as stored — Tokens as read, a digested box once the set is
  // digested (a constructor's argument), `()` for a missing key; a repeated key's last value, its values as an array
  // in GetKeyVals. A `k=v,…` TeX string (an old-form macro's argument) is read as its values' Tokens.
  engine.register_type_with_name::<latexml_core::keyvals::KeyVals>("KeyVals");
  engine.register_fn("to_string", |kv: &mut latexml_core::keyvals::KeyVals| {
    kv.to_string()
  });
  engine.register_fn("GetKeyVal", |kv: Dynamic, key: &str| -> Dynamic {
    keyval_values(kv)
      .into_iter()
      .rev()
      .find(|(k, _)| k == key)
      .map_or(Dynamic::UNIT, |(_, v)| v)
  });
  engine.register_fn("GetKeyVals", |kv: Dynamic| -> Map {
    let mut m = Map::new();
    for (k, v) in keyval_values(kv) {
      match m.get_mut(k.as_str()) {
        None => {
          m.insert(k.into(), v);
        },
        Some(prev) if prev.is_array() => {
          let mut list = std::mem::take(prev).cast::<rhai::Array>();
          list.push(v);
          *prev = Dynamic::from_array(list);
        },
        Some(prev) => *prev = Dynamic::from_array(vec![std::mem::take(prev), v]),
      }
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

  // ── thin wrappers (W3RD1b): Perl's argument forms, each the core call its compile-time macro makes ──
  // Invocation (Package.pm:965): a CS name/`Token`, or TeX with `#n` (an anonymous macro); each argument a
  // `Tokens`/`Token`, TeX source, or `()` (an omitted optional), positionally or as one array.
  engine.register_fn("Invocation", |cs: Dynamic| invocation(cs, Vec::new()));
  engine.register_fn("Invocation", invocation);
  engine.register_fn("Invocation", |cs: Dynamic, a: Dynamic| {
    invocation(cs, vec![a])
  });
  engine.register_fn("Invocation", |cs: Dynamic, a: Dynamic, b: Dynamic| {
    invocation(cs, vec![a, b])
  });
  engine.register_fn(
    "Invocation",
    |cs: Dynamic, a: Dynamic, b: Dynamic, c: Dynamic| invocation(cs, vec![a, b, c]),
  );
  // AddToMacro (Package.pm:2534): TeX source is TokenizeInternal'd, as Perl's.
  engine.register_fn(
    "AddToMacro",
    |cs: Dynamic, tokens: Dynamic| -> RhaiResult<()> {
      let cs = token_arg("AddToMacro", cs)?;
      latexml_core::binding::content::add_to_macro(cs, dynamic_to_tokens(tokens)).map_err(rhai_err)
    },
  );
  // DeclareFontMap (Package.pm:2859): the map an array of 1-char strings or `()`; `family` and the
  // `*_mathstyle` keys as Perl's options.
  engine.register_fn(
    "DeclareFontMap",
    |name: &str, map: rhai::Array| -> RhaiResult<()> { declare_font_map(name, map, Map::new()) },
  );
  engine.register_fn("DeclareFontMap", declare_font_map);
  // DeclareFontMapMultichar (the `DeclareFontMapMultichar!` macro): position (decimal) → its string.
  engine.register_fn("DeclareFontMapMultichar", |name: &str, map: Map| {
    font_map_multichar(format!("{name}_fontmap_multichar"), map);
  });
  engine.register_fn(
    "DeclareFontMapMultichar",
    |name: &str, family: &str, map: Map| {
      font_map_multichar(format!("{name}_{family}_fontmap_multichar"), map);
    },
  );
  engine.register_fn("LoadPool", |name: &str| -> RhaiResult<()> {
    latexml_core::binding::content::input_definitions(
      name,
      latexml_core::binding::content::InputDefinitionOptions {
        extension: Some("pool".into()),
        ..Default::default()
      },
    )
    .map_err(rhai_err)
  });
  // IsEmpty (Package.pm:1034): spaces, braces and other non-printing tokens are empty; a list when each item is.
  engine.register_fn("IsEmpty", |thing: Dynamic| -> RhaiResult<bool> {
    is_empty_thing(&thing)
  });
  engine.register_fn("DigestIf", |cs: Dynamic| -> RhaiResult<Dynamic> {
    let token = token_arg("DigestIf", cs)?;
    Ok(
      latexml_core::binding::content::digest_if(token)
        .map_err(rhai_err)?
        .map_or(Dynamic::UNIT, Dynamic::from),
    )
  });
  // ProcessKeyOptions (the `ProcessKeyOptions!` macro): ProcessOptions' forms, `\@curroptions` kept.
  engine.register_fn("ProcessKeyOptions", || -> RhaiResult<()> {
    latexml_core::binding::content::process_key_options(false, &[]).map_err(rhai_err)
  });
  engine.register_fn("ProcessKeyOptions", |inorder: bool| -> RhaiResult<()> {
    latexml_core::binding::content::process_key_options(inorder, &[]).map_err(rhai_err)
  });
  engine.register_fn("ProcessKeyOptions", |opts: Map| -> RhaiResult<()> {
    check_options(
      "ProcessKeyOptions",
      &["inorder", "keysets"],
      opts.keys().map(|k| k.as_str()),
    )
    .map_err(rhai_err)?;
    let inorder = opts
      .get("inorder")
      .and_then(|v| v.as_bool().ok())
      .unwrap_or(false);
    let keysets = opts
      .get("keysets")
      .cloned()
      .map(string_list)
      .unwrap_or_default();
    let keysets: Vec<&str> = keysets.iter().map(String::as_str).collect();
    latexml_core::binding::content::process_key_options(inorder, &keysets).map_err(rhai_err)
  });
  // FindFile (Package.pm:2076): the path, or `()`; Perl's `type`/`notex`/`noltxml`/`searchpaths_only`.
  engine.register_fn("FindFile", |name: &str| -> RhaiResult<Dynamic> {
    find_file(name, Map::new())
  });
  engine.register_fn("FindFile", find_file);
  engine.register_fn("Input", |name: &str| -> RhaiResult<()> {
    latexml_core::binding::content::input(
      name,
      latexml_core::binding::content::InputOptions::default(),
    )
    .map_err(rhai_err)
  });
  // CleanLabel/CleanID (Package.pm:482-511): a `Token`/`Tokens` label is read as its ToString.
  engine.register_fn("CleanLabel", |label: Dynamic| {
    latexml_core::common::cleaners::clean_label(&dynamic_to_string(label), None).into_owned()
  });
  engine.register_fn("CleanLabel", |label: Dynamic, prefix: &str| {
    latexml_core::common::cleaners::clean_label(&dynamic_to_string(label), Some(prefix))
      .into_owned()
  });
  engine.register_fn("CleanID", |key: Dynamic| {
    latexml_core::common::cleaners::clean_id(&dynamic_to_string(key))
  });
  engine.register_fn("SetCounter", |ctr: &str, value: i64| -> RhaiResult<()> {
    latexml_core::binding::counter::dialect::set_counter(ctr, Number(value)).map_err(rhai_err)
  });
  // Perl's form takes a Number (Package.pm:722).
  engine.register_fn("SetCounter", |ctr: &str, value: Number| -> RhaiResult<()> {
    latexml_core::binding::counter::dialect::set_counter(ctr, value).map_err(rhai_err)
  });
  // roman/Roman (Package.pm:470-476) and Explode/ExplodeText (Token.pm:103-116) return Tokens, as Perl's lists.
  engine.register_fn("roman", |n: i64| {
    explode(&latexml_core::common::cleaners::roman_aux(n), true)
  });
  engine.register_fn("Roman", |n: i64| {
    explode(
      &latexml_core::common::cleaners::roman_aux(n).to_ascii_uppercase(),
      true,
    )
  });
  // CounterValue returned an int before it returned Perl's Number: `roman(CounterValue(c))` keeps working, Warned.
  engine.register_fn("roman", |n: Number| {
    warn_deprecated("Number as an int");
    explode(&latexml_core::common::cleaners::roman_aux(n.0), true)
  });
  engine.register_fn("Roman", |n: Number| {
    warn_deprecated("Number as an int");
    explode(
      &latexml_core::common::cleaners::roman_aux(n.0).to_ascii_uppercase(),
      true,
    )
  });
  engine.register_fn("Explode", |s: &str| explode(s, false));
  engine.register_fn("ExplodeText", |s: &str| explode(s, true));
  // UTF (Util/Unicode.pm:23): the character of a code point.
  engine.register_fn("UTF", |code: i64| -> RhaiResult<String> {
    u32::try_from(code)
      .ok()
      .and_then(char::from_u32)
      .map(String::from)
      .ok_or_else(|| format!("UTF: {code} is not a Unicode code point").into())
  });
  // BeginItemize (latex_constructs.pool.ltxml:1296): the list's property map, for a `properties` closure.
  engine.register_fn(
    "BeginItemize",
    |itype: &str, counter: &str| -> RhaiResult<Map> { begin_itemize(itype, counter, Map::new()) },
  );
  engine.register_fn("BeginItemize", begin_itemize);
  // Equals (Object.pm:73): same type and value; two tokens are equal when `\let` to the same meaning.
  engine.register_fn("Equals", |x: Dynamic, y: Dynamic| equals(x, y));

  // ── further definition forms (DefKeyVal/DefMath/DefLigature; DefRegister is in `values.rs`) ──
  // These run when the script executes (per conversion, like a Perl .ltxml),
  // lowering to the same dialect functions their compile-time macros use.
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
    |pattern: &str, replacement: &str| -> RhaiResult<()> {
      def_ligature(pattern, replacement, Map::new())
    },
  );
  // Perl `DefLigature($regexp, $replacement, fontTest => sub { my ($font) = @_; … })` (Package.pm:3088): the
  // ligature applies only where the test, given the text's `Font`, is true.
  engine.register_fn("DefLigature", def_ligature);
  // A `Font` handle (Perl's Common/Font.pm getters), as a `fontTest` closure receives it; an unset component is `()`.
  engine.register_type_with_name::<latexml_core::common::font::Font>("Font");
  macro_rules! font_getter {
    ($name:literal, $get:ident) => {
      engine.register_fn($name, |f: &mut latexml_core::common::font::Font| {
        f.$get()
          .map_or(Dynamic::UNIT, |v| Dynamic::from(v.to_string()))
      });
    };
  }
  font_getter!("getFamily", get_family);
  font_getter!("getSeries", get_series);
  font_getter!("getShape", get_shape);
  font_getter!("getEncoding", get_encoding);
  font_getter!("getLanguage", get_language);
  engine.register_fn("getSize", |f: &mut latexml_core::common::font::Font| {
    f.get_size().map_or(Dynamic::UNIT, Dynamic::from)
  });
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
      // Loading-script handles for the digest-hook trampolines (DefMath doesn't
      // route through `wire_now`, so fetch them directly like `wire_now` does).
      let (engine, ast) = current_script()?;
      let options = math_options_from_map(proto, opts, &engine, &ast).map_err(rhai_err)?;
      latexml_core::binding::def::dialect::def_math(cs, params, presentation.to_string(), options)
        .map_err(rhai_err)
    },
  );
  // DefConditional: the test is a Rhai closure receiving its args as Tokens and returning a
  // bool; without a test (or with `()`), the `\newif` form driven by `SetCondition`.
  engine.register_fn(
    "DefConditional",
    |proto: &str, test: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_conditional(e, a, proto, Some(test), Map::new()))
    },
  );
  engine.register_fn(
    "DefConditional",
    |proto: &str, test: FnPtr, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_conditional(e, a, proto, Some(test), opts))
    },
  );
  engine.register_fn(
    "DefConditional",
    |proto: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_conditional(e, a, proto, None, Map::new()))
    },
  );
  engine.register_fn(
    "DefConditional",
    |proto: &str, _test: (), opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      wire_now(|e, a| wire_conditional(e, a, proto, None, opts))
    },
  );
  // SetCondition (Package.pm:1255): `\iffoo` to `\iftrue`/`\iffalse`, for a test-less conditional.
  engine.register_fn("SetCondition", |cs: Dynamic, value: bool| {
    let cs = token_arg("SetCondition", cs)?;
    latexml_core::binding::content::set_condition(&cs, value, None);
    Ok::<(), Box<EvalAltResult>>(())
  });
  engine.register_fn("SetCondition", |cs: Dynamic, value: bool, scope: &str| {
    let cs = token_arg("SetCondition", cs)?;
    latexml_core::binding::content::set_condition(&cs, value, scope_of(scope));
    Ok::<(), Box<EvalAltResult>>(())
  });
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
  // Perl's option form `DefAccent($accent, $comb, $standalone, %options)` (TeX_Character.pool.ltxml:92), which reads
  // no option, so none is checked; the Rust definer reads `below`.
  engine.register_fn(
    "DefAccent",
    |accent: &str,
     combining: &str,
     standalone: &str,
     opts: Map|
     -> std::result::Result<(), Box<EvalAltResult>> {
      let below = opts.get("below").is_some_and(dynamic_to_bool);
      def_accent_impl(accent, combining, standalone, below).map_err(rhai_err)
    },
  );
  engine.register_fn(
    "DefAccent",
    |accent: &str,
     combining: &str,
     standalone: &str,
     below: bool|
     -> std::result::Result<(), Box<EvalAltResult>> {
      warn_deprecated("DefAccent");
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
  // Namespace-aware (`model::get_node_attribute`): libxml stores `xml:id` under
  // the built-in xml namespace, so a plain lookup by the qualified name misses
  // it — a script could not read back the id `generateID` had just written.
  // XML::LibXML's getAttribute: undef (`()`) when absent, so `defined(n.getAttribute(k))` tests presence.
  engine.register_fn("getAttribute", |n: &mut NodeProxy, k: &str| -> Dynamic {
    latexml_core::common::model::get_node_attribute(&n.0, k).map_or(Dynamic::UNIT, Dynamic::from)
  });
  engine.register_fn("prevSibling", |n: &mut NodeProxy| -> Dynamic {
    match n.0.get_prev_sibling() {
      Some(p) => Dynamic::from(n.derived(p)),
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
      // A `ParseXML` node is a TOP-LEVEL node of its chunk: everything above it
      // is an artifact of how the chunk was parsed — the throwaway
      // `_lxfragment` wrapper, or the parsed document node — never markup the
      // script wrote. Report NO parent rather than hand one back, which would
      // leak an internal name and let `insertXML(n.parent())` splice
      // `<_lxfragment>` into the page. In-tree nodes are unaffected.
      Some(p) if n.1.is_some() && latexml_core::common::xml::is_parse_artifact(&p) => Dynamic::UNIT,
      Some(p) => Dynamic::from(n.derived(p)),
      None => Dynamic::UNIT,
    }
  });

  // ── node traversal / editing ──
  // The extension point for XML manipulation: every method below is a single
  // `register_fn` over `NodeProxy`, and works identically on a node inside the
  // conversion's tree (rewrite/matcher callbacks) and on one returned by
  // `ParseXML`, because both are the same type. Adding a capability is one entry
  // here; nothing else has to learn about it. Nodes reached FROM a node use
  // `derived`, so a parsed tree keeps its document alive however deep a script
  // walks.
  engine.register_fn("name", |n: &mut NodeProxy| -> String { n.0.get_name() });
  engine.register_fn("children", |n: &mut NodeProxy| -> rhai::Array {
    n.0
      .get_child_nodes()
      .into_iter()
      .map(|c| Dynamic::from(n.derived(c)))
      .collect()
  });
  engine.register_fn("firstChild", |n: &mut NodeProxy| -> Dynamic {
    match n.0.get_first_child() {
      Some(c) => Dynamic::from(n.derived(c)),
      None => Dynamic::UNIT,
    }
  });
  engine.register_fn("nextSibling", |n: &mut NodeProxy| -> Dynamic {
    match n.0.get_next_sibling() {
      Some(s) => Dynamic::from(n.derived(s)),
      None => Dynamic::UNIT,
    }
  });
  engine.register_fn("hasAttribute", |n: &mut NodeProxy, k: &str| -> bool {
    latexml_core::common::model::get_node_attribute(&n.0, k).is_some()
  });
  // Removing an attribute the node does not have is a silent no-op in libxml, so
  // the only thing this can report is a genuine libxml2 failure — propagate it,
  // exactly as `setAttribute` above does, rather than dropping it on the floor.
  engine.register_fn(
    "removeAttribute",
    |n: &mut NodeProxy, k: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::common::model::remove_node_attribute(&mut n.0, k)
        .map_err(Box::<EvalAltResult>::from)
    },
  );
  // The node (and its subtree) serialized back to markup — the inverse of
  // `ParseXML`, so a script can inspect or log what it is about to insert.
  // libxml serializes a node THROUGH its document, so use the one this handle
  // owns when it has one (a `ParseXML` node), and otherwise the live conversion
  // document that an in-tree node belongs to.
  engine.register_fn(
    "toString",
    |n: &mut NodeProxy| -> std::result::Result<String, Box<EvalAltResult>> {
      match &n.1 {
        Some(owned) => Ok(owned.node_to_string(&n.0)),
        None => with_doc(|doc, _props| Ok(doc.document.node_to_string(&n.0))),
      }
    },
  );

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
  // The gullet's reads return Tokens (Perl `$gullet->readArg` …, Core/Gullet.pm), so a catcode the
  // source gave survives. They are methods of a `Gullet` handle, the object a `DefParameterType` reader will
  // receive (round D), and free functions for a macro body, which has no handle (the Perl body's `$_[0]`).
  engine.register_type_with_name::<GulletHandle>("Gullet");
  engine.register_fn("readArg", |_g: &mut GulletHandle| read_arg());
  engine.register_fn("ReadArg", read_arg);
  engine.register_fn("readUntil", |_g: &mut GulletHandle, delim: Dynamic| {
    read_until(delim)
  });
  engine.register_fn("ReadUntil", read_until);
  engine.register_fn("readOptional", |_g: &mut GulletHandle| read_optional());
  engine.register_fn("ReadOptional", read_optional);
  engine.register_fn("readToken", |_g: &mut GulletHandle| read_token());
  engine.register_fn("ReadToken", read_token);
  engine.register_fn("skipSpaces", |_g: &mut GulletHandle| {
    latexml_core::gullet::skip_spaces().map_err(rhai_err)
  });
  // DefParameterType (Package.pm:586): the reader gets the gullet handle and the type's extra arguments.
  engine.register_fn(
    "DefParameterType",
    |name: &str, reader: FnPtr| -> RhaiResult<()> {
      wire_now(|e, a| wire_parameter_type(e, a, name, reader, Map::new()))
    },
  );
  engine.register_fn(
    "DefParameterType",
    |name: &str, reader: FnPtr, opts: Map| -> RhaiResult<()> {
      wire_now(|e, a| wire_parameter_type(e, a, name, reader, opts))
    },
  );
  engine.register_type_with_name::<StomachHandle>("Stomach");
  engine.register_fn("getGullet", |_s: &mut StomachHandle| GulletHandle);

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
  // Perl `RequirePackage($package, %options)` (Package.pm:2672-2697); a key outside Perl's table is an Error.
  engine.register_fn(
    "RequirePackage",
    |name: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      const REQUIRE_OPTIONS: &[&str] = &[
        "options",
        "withoptions",
        "type",
        "as_class",
        "noltxml",
        "notex",
        "raw",
        "after",
        "searchpaths_only",
      ];
      let operation = format!("RequirePackage ({name})");
      check_options(&operation, REQUIRE_OPTIONS, opts.keys().map(|k| k.as_str()))
        .map_err(rhai_err)?;
      let mut o = latexml_core::binding::content::RequireOptions::default();
      for (key, val) in opts {
        match key.as_str() {
          "options" => o.options = string_list(val),
          "type" => o.extension = Some(dynamic_to_string(val).into()),
          "as_class" => o.as_class = val.as_bool().unwrap_or(false),
          "noltxml" => o.noltxml = val.as_bool().ok(),
          "notex" => o.notex = val.as_bool().ok(),
          "raw" if val.as_bool().unwrap_or(false) => {
            latexml_core::Warn!(
              "deprecated",
              "raw",
              "RequirePackage option raw is obsolete; it is not needed"
            );
            o.notex = Some(false);
          },
          "after" => o.after = dynamic_to_tokens(val),
          "searchpaths_only" => o.searchpaths_only = val.as_bool().unwrap_or(false),
          // `withoptions` is accepted and recomputed from `options`, as Perl's (:2694).
          _ => {},
        }
      }
      latexml_core::binding::content::require_package(name, o).map_err(rhai_err)
    },
  );
  // Perl `LoadClass($class, %options)` (Package.pm:2699-2730): `options`, `withoptions` (the document's options),
  // `after`; `notex`/`searchpaths_only` reach no class-load option yet (a Warning).
  engine.register_fn(
    "LoadClass",
    |name: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      const LOADCLASS_OPTIONS: &[&str] = &[
        "options",
        "withoptions",
        "after",
        "notex",
        "searchpaths_only",
      ];
      let operation = format!("LoadClass ({name})");
      check_options(
        &operation,
        LOADCLASS_OPTIONS,
        opts.keys().map(|k| k.as_str()),
      )
      .map_err(rhai_err)?;
      let (mut options, mut withoptions, mut after) = (Vec::new(), false, Tokens::default());
      for (key, val) in opts {
        match key.as_str() {
          "options" => options = string_list(val),
          "withoptions" => withoptions = val.as_bool().unwrap_or(false),
          "after" => after = dynamic_to_tokens(val),
          other => unsupported_option(&operation, other),
        }
      }
      if withoptions {
        latexml_core::binding::content::load_class_with_options(name, after)
      } else {
        latexml_core::binding::content::load_class(name, options, after)
      }
      .and_then(|()| latexml_engine::latex_constructs::settle_chapter_after_class_load())
      .map_err(rhai_err)
    },
  );
  // A script binding's `LoadClass` makes the "has chapters" step after the load, as a compiled binding's `LoadClass!`
  // (OXIDIZED_DESIGN #179, `settle_chapter_after_class_load`).
  engine.register_fn(
    "LoadClass",
    |name: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      latexml_core::binding::content::load_class(name, Vec::new(), Tokens::default())
        .and_then(|()| latexml_engine::latex_constructs::settle_chapter_after_class_load())
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
      .and_then(|()| latexml_engine::latex_constructs::settle_chapter_after_class_load())
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
  // Perl `ProcessOptions(inorder => 1, keysets => [...])` (Package.pm:2444-2447).
  engine.register_fn(
    "ProcessOptions",
    |opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      check_options(
        "ProcessOptions",
        &["inorder", "keysets"],
        opts.keys().map(|k| k.as_str()),
      )
      .map_err(rhai_err)?;
      let inorder = opts
        .get("inorder")
        .and_then(|v| v.as_bool().ok())
        .unwrap_or(false);
      let keysets = opts
        .get("keysets")
        .cloned()
        .map(string_list)
        .unwrap_or_default();
      let keysets: Vec<&str> = keysets.iter().map(String::as_str).collect();
      latexml_core::binding::content::process_options(inorder, &keysets).map_err(rhai_err)
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
  // Option-map form — Perl `RequireResource($resource, %options)` (#317). The
  // second hash carries `type` (mime), `media`, and `content`. Perl's option
  // name `type` maps to the `Resource`'s `mimetype`. Unknown keys are ignored
  // (Perl's CheckOptions warns; harmless to drop). The core `require_resource`
  // still infers a missing mime from the extension and warns if none.
  engine.register_fn("RequireResource", |resource: &str, opts: Map| {
    let mut r = latexml_core::document::resource::Resource {
      name: resource.to_string(),
      ..latexml_core::document::resource::Resource::default()
    };
    for (key, val) in opts {
      let v = dynamic_to_string(val);
      match key.as_str() {
        "type" => r.mimetype = v,
        "media" => r.media = v,
        "content" => r.content = v,
        _ => {},
      }
    }
    latexml_core::binding::content::require_resource(r);
  });
  // Tag (Package.pm:2005-2030): autoOpen/autoClose, and the afterOpen/afterClose hooks (each also `:early`, prepended,
  // and `:late`), a closure or an array of closures; any other key is an Error, as Perl's CheckOptions.
  engine.register_fn(
    "Tag",
    |tag: &str, opts: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      const TAG_OPTIONS: &[&str] = &[
        "autoOpen",
        "autoClose",
        "afterOpen",
        "afterClose",
        "afterOpen:early",
        "afterClose:early",
        "afterOpen:late",
        "afterClose:late",
      ];
      let operation = format!("Tag ({tag})");
      check_options(&operation, TAG_OPTIONS, opts.keys().map(|k| k.as_str())).map_err(rhai_err)?;
      let (engine, ast) = current_script()?;
      let mut options = latexml_core::document::tag::TagOptions::default();
      for (key, val) in opts {
        let hooks = || {
          Some(
            hooks_of(val.clone())
              .filter_map(|v| v.try_cast::<FnPtr>())
              .map(|fp| tag_trampoline(fp, engine.clone(), ast.clone()))
              .collect(),
          )
        };
        match key.as_str() {
          "autoOpen" => {
            options.auto_open = val
              .as_bool()
              .map(Into::into)
              .or_else(|_| val.as_float().map(Into::into))
              .ok()
          },
          "autoClose" => options.auto_close = val.as_bool().ok(),
          "afterOpen" => options.after_open = hooks(),
          "afterClose" => options.after_close = hooks(),
          "afterOpen:early" => options.after_open_early = hooks(),
          "afterClose:early" => options.after_close_early = hooks(),
          "afterOpen:late" => options.after_open_late = hooks(),
          "afterClose:late" => options.after_close_late = hooks(),
          _ => {},
        }
      }
      latexml_core::binding::content::install_tag(tag, options);
      Ok(())
    },
  );
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
  // A box in a string context is its `ToString` (Perl's), as `absorbString` reads it.
  engine.register_fn("to_string", |d: &mut Digested| d.to_string());
  engine.register_fn("to_debug", |d: &mut Digested| d.to_string());

  // Perl's `openElement($qname, %attributes)` returns the new node, `closeElement`/`maybeCloseElement` the node
  // closed or undef (Document.pm:787-866; #716 §2).
  engine.register_fn(
    "openElement",
    |_d: &mut DocProxy, tag: &str| -> std::result::Result<NodeProxy, Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        Ok(NodeProxy::borrowed(
          doc.open_element(tag, None, None).map_err(rhai_err)?,
        ))
      })
    },
  );
  engine.register_fn(
    "openElement",
    |_d: &mut DocProxy,
     tag: &str,
     attrib: Map|
     -> std::result::Result<NodeProxy, Box<EvalAltResult>> {
      let attrib = attribute_map(attrib);
      with_doc(|doc, _props| {
        Ok(NodeProxy::borrowed(
          doc
            .open_element(tag, Some(attrib), None)
            .map_err(rhai_err)?,
        ))
      })
    },
  );
  engine.register_fn(
    "closeElement",
    |_d: &mut DocProxy, tag: &str| -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      with_doc(|doc, _props| Ok(node_or_unit(doc.close_element(tag).map_err(rhai_err)?)))
    },
  );
  engine.register_fn(
    "maybeCloseElement",
    |_d: &mut DocProxy, tag: &str| -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        Ok(node_or_unit(
          doc.maybe_close_element(tag).map_err(rhai_err)?,
        ))
      })
    },
  );
  // Perl `$document->setAttribute($node, $key, $value)` (Document.pm:1361).
  engine.register_fn(
    "setAttribute",
    |_d: &mut DocProxy, n: NodeProxy, key: &str, val: &str| -> RhaiResult<()> {
      with_doc(|doc, _props| {
        let mut node = n.0.clone();
        doc.set_attribute(&mut node, key, val).map_err(rhai_err)
      })
    },
  );
  // The older form on the current node, deprecated.
  engine.register_fn(
    "setAttribute",
    |_d: &mut DocProxy, key: &str, val: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      warn_deprecated("document.setAttribute(key, value)");
      with_doc(|doc, _props| {
        let mut node = doc.get_node().clone();
        doc.set_attribute(&mut node, key, val).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  // Any value as its string (Perl's `absorb` of a string; a `GetKeyVal` value, Tokens, a box), `()` as nothing.
  engine.register_fn(
    "absorbString",
    |_d: &mut DocProxy, s: Dynamic| -> std::result::Result<(), Box<EvalAltResult>> {
      if s.is_unit() {
        return Ok(());
      }
      let s = if s.is::<WhatsitProxy>() {
        whatsit_string()?
      } else {
        dynamic_to_string(s)
      };
      with_doc(|doc, props| {
        doc.absorb_string(&s, props).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  // ── the Document XML surface the COMPILE-TIME layer uses (#350) ──
  // "expose it in rhai in full generality, so that the interfaces are comparable
  // in the runtime and compile-time layers." Each registration below is a thin
  // wrapper over the same `Document` method a `_sty.rs` binding calls, named
  // after its Perl `Core/Document.pm` original, ranked by how often the
  // compile-time bindings actually use it.
  //
  // TWO RULES hold for every one of them, because a `.rhai` script is untrusted:
  //  * it must not panic — a panic escapes `catch_unwind` policy here and, under
  //    the `maxperf` `panic = "abort"` profile, takes the process with it; and
  //  * a node from `ParseXML` must NOT be spliced in directly. Such a node is
  //    still owned by its throwaway parse document, so moving it into ours leaves
  //    our tree pointing at memory freed when the script drops the handle.
  //    `insertXML` is the supported route: it RE-CREATES the subtree through
  //    `append_tree`. `in_tree` below enforces that at every splice site.

  // findnodes/findnode (Perl `findnodes`/`findnode`, Document.pm:139-147) — the
  // query half. Without it a script could walk a tree by hand but never ask it a
  // question, which is what "full generality" mostly means in practice.
  engine.register_fn(
    "findnodes",
    |_d: &mut DocProxy, xpath: &str| -> rhai::Array {
      with_doc_infallible(|doc| nodes_to_array(doc.findnodes(xpath, None)))
    },
  );
  engine.register_fn(
    "findnodes",
    |_d: &mut DocProxy, xpath: &str, scope: NodeProxy| -> rhai::Array {
      with_doc_infallible(|doc| nodes_to_array(doc.findnodes(xpath, Some(&scope.0))))
    },
  );
  // No match is `()`, not an error — Perl returns undef, and a script asking
  // "is there one?" is the normal case, not an exceptional one.
  engine.register_fn("findnode", |_d: &mut DocProxy, xpath: &str| -> Dynamic {
    with_doc_infallible(|doc| node_or_unit(doc.findnode(xpath, None)))
  });
  engine.register_fn(
    "findnode",
    |_d: &mut DocProxy, xpath: &str, scope: NodeProxy| -> Dynamic {
      with_doc_infallible(|doc| node_or_unit(doc.findnode(xpath, Some(&scope.0))))
    },
  );

  // getNode/setNode/getElement (Perl Document.pm:72-98) — the insertion point.
  engine.register_fn("getNode", |_d: &mut DocProxy| -> Dynamic {
    with_doc_infallible(|doc| Dynamic::from(NodeProxy::borrowed(doc.get_node().clone())))
  });
  engine.register_fn(
    "setNode",
    |_d: &mut DocProxy, n: NodeProxy| -> std::result::Result<(), Box<EvalAltResult>> {
      let n = in_tree(n, "setNode")?;
      with_doc(|doc, _props| {
        doc.set_node(&n);
        Ok(())
      })
    },
  );
  engine.register_fn("getElement", |_d: &mut DocProxy| -> Dynamic {
    with_doc_infallible(|doc| node_or_unit(doc.get_element()))
  });

  // canContain (Perl `canContain`, Document.pm:160) — the schema query a binding
  // asks before opening an element: may `tag` (a node OR a qname) hold a `child`
  // qname? The motivating use (#594) is `canContain(node, "#PCDATA")` to decide
  // whether a text-bearing wrapper (`ltx:p`) must be opened first. Read-only, so
  // it is `with_doc_infallible` — meaningful only inside a body (where a live
  // node handle exists); `false` outside one.
  engine.register_fn(
    "canContain",
    |_d: &mut DocProxy, tag: NodeProxy, child: &str| -> bool {
      with_doc_infallible(|_doc| latexml_core::document::can_contain(&tag.0, child))
    },
  );
  engine.register_fn(
    "canContain",
    |_d: &mut DocProxy, tag: &str, child: &str| -> bool {
      with_doc_infallible(|_doc| latexml_core::document::can_contain_qname(tag, child))
    },
  );

  // floatToElement (Perl `floatToElement`, Document.pm:1052) — move the insertion
  // point up to the nearest ancestor that can contain `qname`, and RETURN the
  // previous insertion point (a `Node`, or `()` if none can) so the caller can
  // `setNode(saved)` to restore it after inserting. The 2-arg form passes Perl's
  // `$closeifpossible`: close the intervening auto-closable nodes instead of just
  // repositioning. Mutating, so `with_doc`.
  engine.register_fn(
    "floatToElement",
    |_d: &mut DocProxy, qname: &str| -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        Ok(node_or_unit(
          doc.float_to_element(qname, false).map_err(rhai_err)?,
        ))
      })
    },
  );
  engine.register_fn(
    "floatToElement",
    |_d: &mut DocProxy,
     qname: &str,
     closeifpossible: bool|
     -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        Ok(node_or_unit(
          doc
            .float_to_element(qname, closeifpossible)
            .map_err(rhai_err)?,
        ))
      })
    },
  );

  // insertElement (Perl `insertElement`, Document.pm:639) — open, absorb the
  // content, close, and hand back the node. The ELEMENT counterpart that
  // `insertXML` is named after; it was the one missing half of that pairing.
  engine.register_fn(
    "insertElement",
    |_d: &mut DocProxy, qname: &str| -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        let n = doc
          .insert_element(qname, Vec::new(), None)
          .map_err(rhai_err)?;
        Ok(Dynamic::from(NodeProxy::borrowed(n)))
      })
    },
  );
  engine.register_fn(
    "insertElement",
    |_d: &mut DocProxy,
     qname: &str,
     attrib: Map|
     -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      let attrib = attribute_map(attrib);
      with_doc(|doc, _props| {
        let n = doc
          .insert_element(qname, Vec::new(), Some(attrib))
          .map_err(rhai_err)?;
        Ok(Dynamic::from(NodeProxy::borrowed(n)))
      })
    },
  );
  engine.register_fn(
    "insertElement",
    |_d: &mut DocProxy,
     qname: &str,
     content: Digested|
     -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        let n = doc
          .insert_element(qname, vec![&content], None)
          .map_err(rhai_err)?;
        Ok(Dynamic::from(NodeProxy::borrowed(n)))
      })
    },
  );
  engine.register_fn(
    "insertElement",
    |_d: &mut DocProxy,
     qname: &str,
     content: Digested,
     attrib: Map|
     -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      let attrib = attribute_map(attrib);
      with_doc(|doc, _props| {
        let n = doc
          .insert_element(qname, vec![&content], Some(attrib))
          .map_err(rhai_err)?;
        Ok(Dynamic::from(NodeProxy::borrowed(n)))
      })
    },
  );
  // insertPI (Perl `insertPI`, Document.pm:703) — insert a `<?op attr=".."?>`
  // processing instruction: a bare target, or a target + attribute map (#537).
  engine.register_fn(
    "insertPI",
    |_d: &mut DocProxy, op: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        doc.insert_pi(op, None).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  engine.register_fn(
    "insertPI",
    |_d: &mut DocProxy, op: &str, attrib: Map| -> std::result::Result<(), Box<EvalAltResult>> {
      let attrib = attribute_map(attrib);
      with_doc(|doc, _props| {
        doc.insert_pi(op, Some(attrib)).map_err(rhai_err)?;
        Ok(())
      })
    },
  );

  // openElementAt/closeElementAt (Perl Document.pm:1830-1884) — build at an
  // explicit node instead of the current insertion point.
  engine.register_fn(
    "openElementAt",
    |_d: &mut DocProxy,
     at: NodeProxy,
     qname: &str|
     -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      let mut at = in_tree(at, "openElementAt")?;
      with_doc(|doc, _props| {
        let n = doc
          .open_element_at(&mut at, qname, None, None)
          .map_err(rhai_err)?;
        Ok(Dynamic::from(NodeProxy::borrowed(n)))
      })
    },
  );
  engine.register_fn(
    "openElementAt",
    |_d: &mut DocProxy,
     at: NodeProxy,
     qname: &str,
     attrib: Map|
     -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      let mut at = in_tree(at, "openElementAt")?;
      let attrib = attribute_map(attrib);
      with_doc(|doc, _props| {
        let n = doc
          .open_element_at(&mut at, qname, Some(attrib), None)
          .map_err(rhai_err)?;
        Ok(Dynamic::from(NodeProxy::borrowed(n)))
      })
    },
  );
  engine.register_fn(
    "closeElementAt",
    |_d: &mut DocProxy, at: NodeProxy| -> std::result::Result<(), Box<EvalAltResult>> {
      let mut at = in_tree(at, "closeElementAt")?;
      with_doc(|doc, _props| doc.close_element_at(&mut at).map_err(rhai_err))
    },
  );

  // Per-node document operations (Perl `addClass`, `generateID`).
  engine.register_fn(
    "addClass",
    |_d: &mut DocProxy, n: NodeProxy, class: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      let mut n = in_tree(n, "addClass")?;
      with_doc(|doc, _props| doc.add_class(&mut n, class).map_err(rhai_err))
    },
  );
  engine.register_fn(
    "generateID",
    |_d: &mut DocProxy,
     n: NodeProxy,
     prefix: &str|
     -> std::result::Result<(), Box<EvalAltResult>> {
      let mut n = in_tree(n, "generateID")?;
      with_doc(|doc, _props| doc.generate_id(&mut n, prefix).map_err(rhai_err))
    },
  );

  // Structural edits (Perl `removeNode`, `replaceNode`, `renameNode`,
  // `unwrapNodes`, `wrapNodes`, `appendClone`).
  engine.register_fn(
    "removeNode",
    |_d: &mut DocProxy, n: NodeProxy| -> std::result::Result<(), Box<EvalAltResult>> {
      let n = in_tree(n, "removeNode")?;
      with_doc(|doc, _props| {
        doc.remove_node(n);
        Ok(())
      })
    },
  );
  engine.register_fn(
    "replaceNode",
    |_d: &mut DocProxy,
     n: NodeProxy,
     with: rhai::Array|
     -> std::result::Result<(), Box<EvalAltResult>> {
      let n = in_tree(n, "replaceNode")?;
      let with = in_tree_array(with, "replaceNode")?;
      with_doc(|doc, _props| doc.replace_node(n, with).map_err(rhai_err))
    },
  );
  // `Document::rename_node` PANICS on a node with no parent (its own `expect`),
  // so the orphan case has to be refused here rather than reached — an untrusted
  // script must not be able to abort the conversion by renaming a node it just
  // removed.
  engine.register_fn(
    "renameNode",
    |_d: &mut DocProxy,
     n: NodeProxy,
     newname: &str|
     -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      rename_node_checked(n, newname, true)
    },
  );
  engine.register_fn(
    "renameNode",
    |_d: &mut DocProxy,
     n: NodeProxy,
     newname: &str,
     reinsert: bool|
     -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      rename_node_checked(n, newname, reinsert)
    },
  );
  engine.register_fn(
    "unwrapNodes",
    |_d: &mut DocProxy, n: NodeProxy| -> std::result::Result<(), Box<EvalAltResult>> {
      let n = in_tree(n, "unwrapNodes")?;
      with_doc(|doc, _props| doc.unwrap_nodes(n).map_err(rhai_err))
    },
  );
  // Returns `()` when there is nothing to wrap or the target has no parent —
  // `Document::wrap_nodes`' own "can't wrap here" answer, not an error.
  engine.register_fn(
    "wrapNodes",
    |_d: &mut DocProxy,
     qname: &str,
     nodes: rhai::Array|
     -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      let nodes = in_tree_array(nodes, "wrapNodes")?;
      with_doc(|doc, _props| {
        let wrapped = doc.wrap_nodes(qname, nodes).map_err(rhai_err)?;
        Ok(node_or_unit(wrapped))
      })
    },
  );
  engine.register_fn(
    "appendClone",
    |_d: &mut DocProxy,
     n: NodeProxy,
     children: rhai::Array|
     -> std::result::Result<(), Box<EvalAltResult>> {
      let mut n = in_tree(n, "appendClone")?;
      let children = in_tree_array(children, "appendClone")?;
      with_doc(|doc, _props| doc.append_clone(&mut n, children).map_err(rhai_err))
    },
  );

  // ── XML manipulation surface ──
  // Three shapes, one operation: parse-and-insert in a single call, or insert
  // nodes a script already holds (from `ParseXML`, possibly after editing them).
  //
  // insertXML(markup) — parse an XML/(X)HTML string and splice the parsed subtree
  // in at the current point (Perl BookML's `\bmlRawHTML`: parse_string +
  // appendTree). Distinct from `absorbString`, which inserts the argument as
  // ESCAPED text; this inserts it as structured element/attribute/text nodes.
  engine.register_fn(
    "insertXML",
    |_d: &mut DocProxy, s: &str| -> std::result::Result<(), Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        doc.insert_xml(s).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  // insertXML(node) — insert one node a script is holding.
  engine.register_fn(
    "insertXML",
    |_d: &mut DocProxy, n: NodeProxy| -> std::result::Result<(), Box<EvalAltResult>> {
      with_doc(|doc, _props| {
        doc.insert_nodes(vec![n.0.clone()]).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  // insertXML(nodes) — insert an array of nodes, i.e. what `ParseXML` returns.
  // A non-node entry is an ERROR naming its position and type, never a silent
  // skip: quietly dropping part of what a script asked to insert is exactly the
  // content-loss failure mode this feature has to avoid.
  engine.register_fn(
    "insertXML",
    |_d: &mut DocProxy, nodes: rhai::Array| -> std::result::Result<(), Box<EvalAltResult>> {
      let mut owned = Vec::with_capacity(nodes.len());
      for (i, entry) in nodes.into_iter().enumerate() {
        let kind = entry.type_name();
        owned.push(entry.try_cast::<NodeProxy>().ok_or_else(|| {
          Box::<EvalAltResult>::from(format!(
            "insertXML: entry {i} of the array is a {kind}, not a Node (from ParseXML)"
          ))
        })?);
      }
      with_doc(move |doc, _props| {
        let raw = owned.iter().map(|n| n.0.clone()).collect();
        doc.insert_nodes(raw).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  // ParseXML(markup) -> [Node] — the parser exposed as its own primitive (#350),
  // so a script can inspect or edit markup BEFORE it reaches the document. Each
  // returned node OWNS the throwaway document it was parsed into (see
  // `NodeProxy`), so it is safe to hold, walk and mutate for as long as the
  // script keeps it — unlike an in-tree node from a rewrite callback. Returns an
  // array because a chunk may legitimately be a fragment of several siblings.
  engine.register_fn(
    "ParseXML",
    |markup: &str| -> std::result::Result<rhai::Array, Box<EvalAltResult>> {
      // Malformed markup is reported as a latexml `Error:` and yields an EMPTY
      // array — deliberately NOT a Rhai runtime error. A script error raised here
      // would propagate out of the constructor and ABORT THE WHOLE CONVERSION
      // (`wire.rs` maps it to a hard `Error::from`), so a single bad snippet in one
      // contrib binding would cost the entire document. That contradicts the
      // failure-isolation contract (`script_bindings_plan.md`: a bad binding
      // "degrades only that package"), and it would also make these two siblings
      // behave oppositely on the same bad input — `insertXML(markup)` already
      // degrades this way. Loud, but contained: the author still gets the `Error:`.
      match latexml_core::common::xml::parse_fragment(markup) {
        Ok(parsed) => {
          let doc = parsed.document().clone();
          Ok(
            parsed
              .nodes()
              .into_iter()
              .map(|n| Dynamic::from(NodeProxy::owning(n, doc.clone())))
              .collect(),
          )
        },
        Err(e) => {
          // The one thing that MAY still abort is `Error!` escalating to Fatal
          // past MAX_ERRORS — same shape as the `Error` registration above.
          let res: Result<()> = (|| {
            latexml_core::Error!(
              "malformed",
              "ParseXML",
              format!("could not parse XML markup: {e}")
            );
            Ok(())
          })();
          res.map_err(rhai_err)?;
          Ok(rhai::Array::new())
        },
      }
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
  // absorb overload for a `Tokens` value — an `Undigested` constructor arg reaches
  // the body as `Tokens` (#634), and the dominant Perl port idiom is
  // `$document->absorb($tokens)`. Wrapping as `Digested::from` yields the
  // `Postponed(Tokens)` the core absorbs as text under the active font/mode —
  // identical to how the arg was absorbed before it was unwrapped at the boundary.
  engine.register_fn(
    "absorb",
    |_d: &mut DocProxy, tokens: Tokens| -> std::result::Result<(), Box<EvalAltResult>> {
      let arg = Digested::from(tokens);
      with_doc(|doc, _props| {
        doc.absorb(&arg, None).map_err(rhai_err)?;
        Ok(())
      })
    },
  );
  engine.register_fn(
    "absorb",
    |_d: &mut DocProxy, token: Token| -> std::result::Result<(), Box<EvalAltResult>> {
      let arg = Digested::from(Tokens::new(vec![token]));
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
  // Deprecated (#716 §1): a constructor body reads its trailing properties map (Perl's `%props`, Constructor.pm:137),
  // a construction hook its whatsit argument (#635 added these).
  engine.register_fn(
    "getProperty",
    |_d: &mut DocProxy, name: &str| -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      warn_deprecated("document.getProperty");
      with_doc(|_doc, props| {
        Ok(match props.get(name) {
          Some(stored) => stored_to_dynamic(stored.clone()),
          None => Dynamic::UNIT,
        })
      })
    },
  );
  engine.register_fn(
    "getProperties",
    |_d: &mut DocProxy| -> std::result::Result<Map, Box<EvalAltResult>> {
      warn_deprecated("document.getProperties");
      with_doc(|_doc, props| Ok(props_to_map(props.clone())))
    },
  );

  // ── whatsit proxy: a hook's `whatsit` argument (#716 §1), with Whatsit.pm's accessors ──
  engine.register_type_with_name::<WhatsitProxy>("Whatsit");
  // Perl `getArg($n)` (Whatsit.pm:59): the argument as a constructor body receives it, `()` when absent.
  engine.register_fn(
    "getArg",
    |_w: &mut WhatsitProxy, n: i64| -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      let n = usize::try_from(n).ok().filter(|n| *n > 0);
      with_whatsit(|w| ctor_arg_to_dynamic(&n.and_then(|n| w.get_arg(n)).cloned()))
    },
  );
  // In a string the whatsit is Perl's `ToString($whatsit)`, its reversion (Whatsit.pm:163).
  engine.register_fn(
    "to_string",
    |_w: &mut WhatsitProxy| -> std::result::Result<String, Box<EvalAltResult>> { whatsit_string() },
  );
  // Perl `getProperty($key)` (Whatsit.pm:55): the typed value, `()` when absent.
  engine.register_fn(
    "getProperty",
    |_w: &mut WhatsitProxy, key: &str| -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      with_whatsit(|w| {
        w.get_property(key)
          .map_or(Dynamic::UNIT, |v| stored_to_dynamic(v.into_owned()))
      })
    },
  );
  engine.register_fn(
    "getProperties",
    |_w: &mut WhatsitProxy| -> std::result::Result<Map, Box<EvalAltResult>> {
      with_whatsit(|w| props_to_map(w.get_properties().clone()))
    },
  );
  // Perl `setProperty($key, $value)`, any value (e.g. plain `\footnote`'s afterDigest routing its mark to `mark`).
  engine.register_fn(
    "setProperty",
    |_w: &mut WhatsitProxy,
     key: &str,
     val: Dynamic|
     -> std::result::Result<(), Box<EvalAltResult>> {
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
      // an alias of this one.) The read-only ops (`with_whatsit`) need no such
      // argument — shared refs may alias freely.
      let w = unsafe { &mut *ptr };
      w.set_property(key, dynamic_to_stored(val));
      Ok(())
    },
  );
  // Deprecated (#716): the nullary accessor and the string readers, one release.
  engine.register_fn("whatsit", || {
    warn_deprecated("whatsit");
    WhatsitProxy
  });
  engine.register_fn(
    "argString",
    |_w: &mut WhatsitProxy, n: i64| -> std::result::Result<String, Box<EvalAltResult>> {
      warn_deprecated("whatsit.argString");
      with_whatsit(|w| w.get_arg(n as usize).map(Digested::untex))?
        .transpose()
        .map(Option::unwrap_or_default)
        .map_err(rhai_err)
    },
  );
  engine.register_fn(
    "propertyString",
    |_w: &mut WhatsitProxy, key: &str| -> std::result::Result<String, Box<EvalAltResult>> {
      warn_deprecated("whatsit.propertyString");
      with_whatsit(|w| w.get_property_string(key))
    },
  );

  // ── definition proxy (#321): LookupDefinition(cs) → push/unshift hooks ──
  // Faithful to Perl `LookupDefinition($t)` + BookML's `push(@{ $$def{hook} },
  // sub{…})`: fetch an installed definition and splice a hook onto one of its
  // before/after lists. Returns `()` for an undefined CS (Perl undef). Each
  // `push*` appends (Perl `push`), each `unshift*` prepends (Perl `unshift`).
  engine.register_type_with_name::<DefinitionProxy>("Definition");
  // In a string, Perl's `ToString($definition)`: the CS and its parameters.
  engine.register_fn("to_string", |d: &mut DefinitionProxy| definition_string(d));
  engine.register_fn("to_debug", |d: &mut DefinitionProxy| definition_string(d));
  // `()` also for a token `\let` to a character, as `LookupDefinition!` and Perl `lookupDefinition` (State.pm:399-416).
  engine.register_fn(
    "LookupDefinition",
    |cs: Dynamic| -> std::result::Result<Dynamic, Box<EvalAltResult>> {
      let cs = token_arg("LookupDefinition", cs)?;
      Ok(match latexml_core::state::lookup_definition(&cs) {
        Ok(Some(_)) => match latexml_core::state::lookup_meaning(&cs) {
          Some(meaning) => Dynamic::from(DefinitionProxy { cs, meaning }),
          None => Dynamic::UNIT,
        },
        _ => Dynamic::UNIT,
      })
    },
  );
  macro_rules! def_hook_method {
    ($rhai:literal, $family:expr, $at_front:expr) => {
      engine.register_fn(
        $rhai,
        |d: &mut DefinitionProxy, fp: FnPtr| -> std::result::Result<(), Box<EvalAltResult>> {
          push_definition_hook(&d.cs, $family, $at_front, fp)
        },
      );
    };
  }
  def_hook_method!("pushBeforeDigest", HookFamily::BeforeDigest, false);
  def_hook_method!("unshiftBeforeDigest", HookFamily::BeforeDigest, true);
  def_hook_method!("pushAfterDigest", HookFamily::AfterDigest, false);
  def_hook_method!("unshiftAfterDigest", HookFamily::AfterDigest, true);
  def_hook_method!("pushBeforeConstruct", HookFamily::BeforeConstruct, false);
  def_hook_method!("unshiftBeforeConstruct", HookFamily::BeforeConstruct, true);
  def_hook_method!("pushAfterConstruct", HookFamily::AfterConstruct, false);
  def_hook_method!("unshiftAfterConstruct", HookFamily::AfterConstruct, true);
  def_hook_method!("pushAfterDigestBody", HookFamily::AfterDigestBody, false);
  def_hook_method!("unshiftAfterDigestBody", HookFamily::AfterDigestBody, true);

  engine
}

/// Perl's `Tokens(@tokens)` (Tokens.pm:29): `Token`s, `Tokens` and arrays of them, flattened.
fn tokens_of(items: rhai::Array) -> std::result::Result<Tokens, Box<EvalAltResult>> {
  let mut list = Vec::with_capacity(items.len());
  for item in items {
    if let Some(t) = item.clone().try_cast::<Token>() {
      list.push(t);
    } else if let Some(ts) = item.clone().try_cast::<Tokens>() {
      list.extend(ts.unlist());
    } else if let Some(nested) = item.clone().try_cast::<rhai::Array>() {
      list.extend(tokens_of(nested)?.unlist());
    } else {
      return Err(
        format!(
          "Tokens: expected a Token or Tokens, got {}",
          item.type_name()
        )
        .into(),
      );
    }
  }
  Ok(Tokens::new(list))
}

/// The char of a 1-char string argument (Perl passes the char itself).
fn font_handle(font: Option<Rc<latexml_core::common::font::Font>>) -> Dynamic {
  font.map_or(Dynamic::UNIT, |f| Dynamic::from((*f).clone()))
}

fn text_box(s: &str, font: Dynamic, tokens: Dynamic, props: Map) -> RhaiResult<Digested> {
  let font = match font.clone().try_cast::<latexml_core::common::font::Font>() {
    Some(f) => Some(Rc::new(f)),
    None if font.is_unit() => None,
    None => return Err(format!("Box: the font is a Font or (), got {}", font.type_name()).into()),
  };
  let tokens = if tokens.is_unit() {
    Tokens::default()
  } else {
    dynamic_to_tokens(tokens)
  };
  let mut properties = SymHashMap::default();
  for (k, v) in props {
    properties.insert(k.as_str(), dynamic_to_stored(v));
  }
  Ok(Digested::from(latexml_core::tbox::Tbox::new(
    arena::pin(s),
    font,
    None,
    tokens,
    properties,
  )))
}

fn list_of(boxes: rhai::Array, reqmode: Option<String>) -> RhaiResult<Digested> {
  let mut items = Vec::with_capacity(boxes.len());
  for b in boxes.into_iter().filter(|b| !b.is_unit()) {
    match b.clone().try_cast::<Digested>() {
      Some(d) => items.push(d),
      None => return Err(format!("List: expected boxes, got {}", b.type_name()).into()),
    }
  }
  if items.len() == 1
    && reqmode.as_ref().is_none_or(|m| {
      items[0]
        .get_property("mode")
        .is_some_and(|p| p.to_string() == *m)
    })
  {
    return Ok(items.remove(0));
  }
  let mode = reqmode.unwrap_or_else(|| latexml_core::state::lookup_string("MODE"));
  if mode == "horizontal" {
    items = items
      .into_iter()
      .flat_map(|d| match d.data() {
        latexml_core::digested::DigestedData::List(l)
          if d
            .get_property("mode")
            .is_some_and(|p| p.to_string() == "horizontal") =>
        {
          l.borrow().boxes.clone()
        },
        _ => vec![d],
      })
      .collect();
  }
  let mut list = latexml_core::list::List::new(items);
  list.mode = Some(if mode.ends_with("math") {
    latexml_core::TexMode::Math
  } else {
    latexml_core::TexMode::Text
  });
  list
    .properties
    .insert("mode", Stored::String(arena::pin(&mode)));
  if !list.boxes.is_empty()
    && mode.ends_with("vertical")
    && let Some(baseline) = latexml_core::state::lookup_dimension("\\baselineskip")
  {
    list
      .properties
      .insert("baseline", Stored::Dimension(baseline));
  }
  Ok(Digested::from(list))
}

fn char_of(c: &str) -> Option<char> { c.chars().next() }

/// A char-code table's entry as a Rhai int, or `()` for an entry the table does not hold (Perl undef).
fn code_of(code: Option<u16>) -> Dynamic {
  code.map_or(Dynamic::UNIT, |n| Dynamic::from(i64::from(n)))
}

fn assign_catcode(ch: char, code: u16, scope: Option<Scope>) {
  latexml_core::state::assign_catcode(ch, Catcode::from(code as u8), scope);
}

/// An `Assign*code(char, code[, scope])` call, as its macro (Package.pm :299-381). A code above `max` (a catcode
/// above 15, TeX's "Invalid code"; a code the u16 table cannot hold) or a missing char is an Error, never dropped.
fn assign_code(
  call: &str,
  assign: fn(char, u16, Option<Scope>),
  max: i64,
  c: &str,
  code: i64,
  scope: &str,
) -> RhaiResult<()> {
  match (char_of(c), u16::try_from(code)) {
    (Some(ch), Ok(n)) if code <= max => assign(ch, n, scope_of(scope)),
    _ => {
      let res: Result<()> = (|| {
        latexml_core::Error!(
          "unexpected",
          call,
          format!("{call}({c:?}, {code}): the code must be 0..={max} and the char one character")
        );
        Ok(())
      })();
      res.map_err(rhai_err)?;
    },
  }
  Ok(())
}

/// A handle on the gullet (thread-local state, so the handle carries nothing): the receiver of Perl's
/// `$gullet->read*` methods.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct GulletHandle;

/// A handle on the stomach, a primitive body's first argument (Primitive.pm:78); `getGullet` reaches the gullet.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct StomachHandle;

type RhaiResult<T> = std::result::Result<T, Box<EvalAltResult>>;

/// One balanced argument off the stream, unexpanded (Perl `readArg`).
fn read_arg() -> RhaiResult<Tokens> {
  latexml_core::gullet::read_arg(latexml_core::gullet::ExpansionLevel::Off).map_err(rhai_err)
}

/// The tokens up to, and consuming, the delimiter (Perl `readUntil`): a string delimiter is TeX source,
/// TokenizeInternal'd; a `Token`/`Tokens` is used as is.
fn read_until(delim: Dynamic) -> RhaiResult<Tokens> {
  let delim = if delim.is_string() {
    mouth::tokenize_internal(TeXString::assembled(
      delim.into_string().unwrap_or_default(),
    ))
  } else {
    tokens_of(vec![delim])?
  };
  latexml_core::gullet::read_until(&delim)
    .map(Option::unwrap_or_default)
    .map_err(rhai_err)
}

/// A bracketed optional argument, or `()` when there is none (Perl `readOptional` returns undef).
fn read_optional() -> RhaiResult<Dynamic> {
  latexml_core::gullet::read_optional(None)
    .map(|t| t.map_or(Dynamic::UNIT, Dynamic::from))
    .map_err(rhai_err)
}

/// The next token, unexpanded, or `()` at the end of input (Perl `readToken`).
fn read_token() -> RhaiResult<Dynamic> {
  latexml_core::gullet::read_token()
    .map(|t| t.map_or(Dynamic::UNIT, Dynamic::from))
    .map_err(rhai_err)
}

/// `PushValue`/`UnshiftValue` of `arity` values in one call, in order, as Perl's lists (Package.pm:265/275).
fn register_variadic_values(engine: &mut Engine, arity: usize) {
  let push = |k: &str, vs: Vec<Dynamic>| {
    for v in vs {
      let _ = latexml_core::state::push_value(k, dynamic_to_stored(v));
    }
  };
  let unshift = |k: &str, vs: Vec<Dynamic>| {
    latexml_core::state::unshift_value(k, vs.into_iter().map(dynamic_to_stored).collect());
  };
  match arity {
    2 => {
      engine.register_fn("PushValue", move |k: &str, a: Dynamic, b: Dynamic| {
        push(k, vec![a, b])
      });
      engine.register_fn("UnshiftValue", move |k: &str, a: Dynamic, b: Dynamic| {
        unshift(k, vec![a, b])
      });
    },
    3 => {
      engine.register_fn(
        "PushValue",
        move |k: &str, a: Dynamic, b: Dynamic, c: Dynamic| push(k, vec![a, b, c]),
      );
      engine.register_fn(
        "UnshiftValue",
        move |k: &str, a: Dynamic, b: Dynamic, c: Dynamic| unshift(k, vec![a, b, c]),
      );
    },
    _ => {
      engine.register_fn(
        "PushValue",
        move |k: &str, a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic| push(k, vec![a, b, c, d]),
      );
      engine.register_fn(
        "UnshiftValue",
        move |k: &str, a: Dynamic, b: Dynamic, c: Dynamic, d: Dynamic| unshift(k, vec![a, b, c, d]),
      );
    },
  }
}

/// The four diagnostics of Perl Common/Error.pm, as the Rhai functions of the same names.
#[derive(Clone, Copy)]
enum Diag {
  Warn,
  Error,
  Info,
  Fatal,
}
impl Diag {
  fn name(self) -> &'static str {
    match self {
      Diag::Warn => "Warn",
      Diag::Error => "Error",
      Diag::Info => "Info",
      Diag::Fatal => "Fatal",
    }
  }
}

fn emit_diag(
  kind: Diag,
  cat: &str,
  obj: &str,
  msg: &str,
  details: Option<String>,
) -> RhaiResult<()> {
  match (kind, details) {
    (Diag::Warn, Some(d)) => latexml_core::Warn!(cat, obj, msg.to_string(), d),
    (Diag::Warn, None) => latexml_core::Warn!(cat, obj, msg.to_string()),
    (Diag::Info, Some(d)) => latexml_core::Info!(cat, obj, msg.to_string(), d),
    (Diag::Info, None) => latexml_core::Info!(cat, obj, msg.to_string()),
    (Diag::Error, details) => return script_error(cat, obj, msg, details),
    (Diag::Fatal, details) => return script_fatal(cat, obj, msg, details),
  }
  Ok(())
}

/// Perl's trailing `@details` (Error.pm:271-378): a string, or an array of strings joined one per line.
fn details_of(details: Dynamic) -> String {
  if details.is_array() {
    let lines: Vec<String> = details
      .cast::<rhai::Array>()
      .into_iter()
      .map(dynamic_to_string)
      .collect();
    lines.join("\n\t")
  } else {
    dynamic_to_string(details)
  }
}

/// `Error(cat, obj, msg[, details])`. `Error!` escalates to a Fatal past MAX_ERRORS (an `Err`), which the script
/// sees as a Rhai error so the conversion aborts cleanly.
pub(super) fn script_error(
  cat: &str,
  obj: &str,
  msg: &str,
  details: Option<String>,
) -> RhaiResult<()> {
  let res: Result<()> = (|| {
    match details {
      Some(details) => latexml_core::Error!(cat, obj, msg.to_string(), details),
      None => latexml_core::Error!(cat, obj, msg.to_string()),
    }
    Ok(())
  })();
  res.map_err(rhai_err)
}

/// `Fatal(cat, obj, msg[, details])`: logged as `Fatal:<cat>:<obj> <msg>` (Error.pm:271), ending the run (#792).
fn script_fatal(cat: &str, obj: &str, msg: &str, details: Option<String>) -> RhaiResult<()> {
  use latexml_core::common::error::{ErrorCategory, ErrorTarget};
  let message = match details {
    Some(details) => format!("{msg}\n\t{details}"),
    None => msg.to_string(),
  };
  let res: Result<()> = (|| {
    latexml_core::Fatal!(
      ErrorTarget::Named(cat.to_string()),
      ErrorCategory::Named(obj.to_string()),
      message
    );
    #[allow(unreachable_code)]
    Ok(())
  })();
  res.map_err(rhai_err)
}

/// A list option (`options => [...]`, `keysets => [...]`): an array of strings, or one comma-separated string.
fn string_list(val: Dynamic) -> Vec<String> {
  if val.is_array() {
    val
      .cast::<rhai::Array>()
      .into_iter()
      .map(dynamic_to_string)
      .collect()
  } else {
    dynamic_to_string(val)
      .split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect()
  }
}

thread_local! {
  static CODE_MACROS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// A fresh name for a closure installed as a macro, as Perl's `T_CS(ToString($code))` (`\CODE(0x…)`).
fn code_macro_name() -> String {
  let n = CODE_MACROS.with(|c| {
    c.set(c.get() + 1);
    c.get()
  });
  format!("\\CODE({n})")
}

/// Perl `Invocation($token, @args)` (Package.pm:965) over the macro's core calls.
fn invocation(cs: Dynamic, args: rhai::Array) -> RhaiResult<Tokens> {
  let args: Vec<Option<Tokens>> = args
    .into_iter()
    .map(|a| (!a.is_unit()).then(|| dynamic_to_tokens(a)))
    .collect();
  use latexml_core::binding::content::{build_invocation, build_invocation_str};
  if cs.is_string() {
    build_invocation_str(TeXString::assembled(dynamic_to_string(cs)), args)
  } else if let Some(tokens) = cs.clone().try_cast::<Tokens>() {
    match tokens.clone().unlist().as_slice() {
      [token] => build_invocation(*token, args),
      _ => {
        let args: Vec<Option<std::borrow::Cow<Tokens>>> = args
          .into_iter()
          .map(|a| a.map(std::borrow::Cow::Owned))
          .collect();
        tokens
          .pack_parameters()
          .map(|t| t.substitute_parameters(&args))
      },
    }
  } else {
    build_invocation(token_arg("Invocation", cs)?, args)
  }
  .map_err(rhai_err)
}

/// Perl `DeclareFontMap($name, $map, %options)` (Package.pm:2859).
fn declare_font_map(name: &str, map: rhai::Array, opts: Map) -> RhaiResult<()> {
  const FONTMAP_OPTIONS: &[&str] = &[
    "family",
    "uppercase_mathstyle",
    "lowercase_mathstyle",
    "digit_mathstyle",
  ];
  check_options(
    "DeclareFontMap",
    FONTMAP_OPTIONS,
    opts.keys().map(|k| k.as_str()),
  )
  .map_err(rhai_err)?;
  let encname = match opts.get("family") {
    Some(family) => format!("{name}_{}", dynamic_to_string(family.clone())),
    None => name.to_string(),
  };
  let glyphs: Rc<[Option<char>]> = map
    .into_iter()
    .map(|g| {
      if g.is_unit() {
        None
      } else {
        dynamic_to_string(g).chars().next()
      }
    })
    .collect();
  latexml_core::state::assign_value(&format!("{encname}_fontmap"), glyphs, Some(Scope::Global));
  for style in [
    "uppercase_mathstyle",
    "lowercase_mathstyle",
    "digit_mathstyle",
  ] {
    // A font map, stored as the `Font` the glyphs take on in math (tex_fonts.rs's `OMS_uppercase_mathstyle`).
    match opts.get(style).map(|v| v.clone().try_cast::<Map>()) {
      Some(Some(font)) => latexml_core::state::assign_value(
        &format!("{encname}_{style}"),
        Stored::Font(Rc::new(font_from_rhai_map(font))),
        Some(Scope::Global),
      ),
      Some(None) => unsupported_option("DeclareFontMap", &format!("{style} other than a font map")),
      None => {},
    }
  }
  Ok(())
}

/// The `DeclareFontMapMultichar!` store: position (decimal) → the string it prints.
fn font_map_multichar(mapname: String, map: Map) {
  let map: rustc_hash::FxHashMap<String, String> = map
    .into_iter()
    .map(|(pos, s)| (pos.to_string(), dynamic_to_string(s)))
    .collect();
  latexml_core::state::assign_value(&mapname, map, Some(Scope::Global));
}

/// A keyval set's (key, value) pairs in the order given (`GetKeyVal`'s reading): a `KeyVals` argument's values, a
/// digested set's digested values (its read values before digestion), a `k=v,…` string's values TokenizeInternal'd.
fn keyval_values(kv: Dynamic) -> Vec<(String, Dynamic)> {
  let from_set = |set: &latexml_core::keyvals::KeyVals, digested: bool| {
    let mut out = Vec::new();
    let mut seen: rustc_hash::FxHashMap<&str, usize> = rustc_hash::FxHashMap::default();
    for (key, value) in set.get_pairs() {
      let n = seen.entry(key.as_str()).or_insert(0);
      let value = match set.get_values_digested(key).and_then(|vs| vs.get(*n)) {
        Some(d) if digested => Dynamic::from(d.clone()),
        _ => arg_value(value.clone()),
      };
      *n += 1;
      out.push((key.clone(), value));
    }
    out
  };
  if let Some(set) = kv.clone().try_cast::<latexml_core::keyvals::KeyVals>() {
    from_set(&set, false)
  } else if let Some(d) = kv.clone().try_cast::<Digested>() {
    match d.data() {
      latexml_core::digested::DigestedData::KeyVals(set) => from_set(set, true),
      _ => Vec::new(),
    }
  } else if kv.is_string() {
    latexml_core::keyval::split_keyval_source(&kv.into_string().unwrap_or_default())
      .into_iter()
      .map(|(k, v)| {
        let v = mouth::tokenize_internal(TeXString::assembled(v));
        (k, Dynamic::from(v))
      })
      .collect()
  } else {
    Vec::new()
  }
}

/// Perl `IsEmpty` (Package.pm:1034): a token is empty unless a letter, other, active or control sequence; TeX
/// source is read as its tokens; a digested thing as the core's `is_empty`; an array when each item is.
fn is_empty_thing(thing: &Dynamic) -> RhaiResult<bool> {
  use latexml_core::token::Catcode;
  let token_is_empty = |t: &Token| {
    !matches!(
      t.code,
      Catcode::LETTER | Catcode::OTHER | Catcode::ACTIVE | Catcode::CS
    )
  };
  if thing.is_unit() {
    Ok(true)
  } else if let Some(token) = thing.clone().try_cast::<Token>() {
    Ok(token_is_empty(&token))
  } else if let Some(digested) = thing.clone().try_cast::<Digested>() {
    digested.is_empty().map_err(rhai_err)
  } else if thing.is_array() {
    for item in thing.clone().cast::<rhai::Array>() {
      if !is_empty_thing(&item)? {
        return Ok(false);
      }
    }
    Ok(true)
  } else {
    Ok(
      dynamic_to_tokens(thing.clone())
        .unlist()
        .iter()
        .all(token_is_empty),
    )
  }
}

/// Perl `FindFile($file, %options)` (Package.pm:2076): the path, or `()`.
fn find_file(name: &str, opts: Map) -> RhaiResult<Dynamic> {
  const FINDFILE_OPTIONS: &[&str] = &[
    "type",
    "notex",
    "noltxml",
    "searchpaths_only",
    "installation_subdir",
  ];
  let operation = format!("FindFile ({name})");
  let mut opts = opts;
  // Package.pm:2079-2082: `raw` is deprecated and dropped.
  if opts.remove("raw").is_some() {
    latexml_core::Warn!(
      "deprecated",
      "raw",
      "FindFile option raw is deprecated; it is not needed"
    );
  }
  check_options(
    &operation,
    FINDFILE_OPTIONS,
    opts.keys().map(|k| k.as_str()),
  )
  .map_err(rhai_err)?;
  let mut o = latexml_core::binding::content::FindFileOptions::default();
  for (key, val) in opts {
    match key.as_str() {
      "type" => o.ext_type = Some(dynamic_to_string(val).into()),
      "notex" => o.notex = val.as_bool().unwrap_or(false),
      "noltxml" => o.forbid_ltxml = val.as_bool().unwrap_or(false),
      "searchpaths_only" => o.search_paths_only = val.as_bool().unwrap_or(false),
      other => unsupported_option(&operation, other),
    }
  }
  Ok(latexml_core::binding::content::find_file(name, Some(o)).map_or(Dynamic::UNIT, Dynamic::from))
}

/// Perl `beginItemize($type, $counter, %options)` (latex_constructs.pool.ltxml:1296): the property map.
fn begin_itemize(itype: &str, counter: &str, opts: Map) -> RhaiResult<Map> {
  check_options(
    "BeginItemize",
    &["nolevel", "series", "start", "resume"],
    opts.keys().map(|k| k.as_str()),
  )
  .map_err(rhai_err)?;
  let mut o = latexml_core::binding::counter::dialect::BeginItemizeOptions::default();
  for (key, val) in opts {
    match key.as_str() {
      "nolevel" => o.nolevel = val.as_bool().unwrap_or(false),
      "series" => o.series = Some(dynamic_to_tokens(val)),
      "start" => o.start = val.as_int().ok().map(Number),
      _ => o.resume = Some(dynamic_to_string(val)),
    }
  }
  let counter = (!counter.is_empty()).then_some(counter);
  let props =
    latexml_core::binding::counter::dialect::begin_itemize(itype, counter, o).map_err(rhai_err)?;
  Ok(props_to_map(props))
}

/// Perl `Equals($x, $y)` (Object.pm:73): undef equals only undef; values of different types differ; tokens
/// compare their meanings, or themselves when neither has one; `Tokens` token by token; others by value.
fn equals(x: Dynamic, y: Dynamic) -> bool {
  if x.is_unit() || y.is_unit() {
    return x.is_unit() && y.is_unit();
  }
  if let (Some(a), Some(b)) = (x.clone().try_cast::<Token>(), y.clone().try_cast::<Token>()) {
    use latexml_core::state::lookup_meaning;
    return match (lookup_meaning(&a), lookup_meaning(&b)) {
      (Some(m), Some(n)) => m == n,
      (None, None) => a == b,
      _ => false,
    };
  }
  if let (Some(a), Some(b)) = (
    x.clone().try_cast::<Tokens>(),
    y.clone().try_cast::<Tokens>(),
  ) {
    return a.equals(b);
  }
  // Perl compares builtins (strings, numbers) as strings; a blessed object by its string form.
  let scalar = |d: &Dynamic| d.is_string() || d.is_int() || d.is_float() || d.is_bool();
  (x.type_name() == y.type_name() || (scalar(&x) && scalar(&y)))
    && dynamic_to_string(x) == dynamic_to_string(y)
}

/// Perl `DefLigature($regexp, $replacement, %options)` (Package.pm:3088), as the `DefLigature!` macro: an
/// invalid regex is an Error naming it; `fontTest` a closure over the `Font`.
fn def_ligature(pattern: &str, replacement: &str, opts: Map) -> RhaiResult<()> {
  check_options(
    "DefLigature",
    &["fontTest"],
    opts.keys().map(|k| k.as_str()),
  )
  .map_err(rhai_err)?;
  let regex_compiled = regex::Regex::new(pattern)
    .map_err(|e| Box::<EvalAltResult>::from(format!("DefLigature bad regex: {e}")))?;
  let font_test = match opts
    .get("fontTest")
    .and_then(|f| f.clone().try_cast::<FnPtr>())
  {
    Some(test) => {
      let (engine, ast) = current_script()?;
      let test: latexml_core::ligature::FontTestClosure = Rc::new(move |font| {
        // (the hook returns a bool, so a Fatal raised in the test is logged, not propagated)
        match call_deferred_body(&engine, &ast, &test, (Dynamic::from(font.clone()),)) {
          Ok(ret) => ret.as_bool().unwrap_or(false),
          Err(e) => contain_bool("DefLigature fontTest", e),
        }
      });
      Some(test)
    },
    None => None,
  };
  let replacement = replacement.to_string();
  latexml_core::state::unshift_value("TEXT_LIGATURES", vec![latexml_core::ligature::Ligature {
    id: latexml_core::state::generate_ligature_id(),
    regex: Some(pattern.to_string()),
    code: Some(Rc::new(move |text| {
      regex_compiled
        .replace_all(text, replacement.as_str())
        .to_string()
    })),
    font_test,
    matcher: None,
  }]);
  Ok(())
}

/// The `Explode!`/`ExplodeText!` macros (Perl Token.pm:103-116) as Tokens.
fn explode(s: &str, text: bool) -> Tokens {
  use latexml_core::{CharToken, T_LETTER, T_OTHER, T_SPACE};
  Tokens::new(if text {
    latexml_core::ExplodeText!(s)
  } else {
    latexml_core::Explode!(s)
  })
}

/// `==`/`!=` between a `T` and a string either way round, on its string form ([`dynamic_to_string`]), Warning
/// `deprecated` (`old`) once per binding load.
pub(super) fn register_string_tests<T: Clone + 'static>(engine: &mut Engine, old: &'static str) {
  let same = move |v: T, s: &str| {
    warn_deprecated(old);
    dynamic_to_string(Dynamic::from(v)) == s
  };
  engine.register_fn("==", same);
  engine.register_fn("!=", move |v: T, s: &str| !same(v, s));
  engine.register_fn("==", move |s: &str, v: T| same(v, s));
  engine.register_fn("!=", move |s: &str, v: T| !same(v, s));
}

//! Wiring: turn a registration (prototype + Rhai body/template + option bag)
//! into a native definition via the shared core builders, plus the hook
//! trampolines and per-options-struct scalar mappers.

use super::*;

/// Install one `DefMacro` registration as a native expandable definition.
pub(super) fn wire_macro(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  body: FnPtr,
) -> Result<()> {
  let (cs, paramlist) = parse_prototype(proto, true)?;
  let cs_name = cs.to_string();
  let engine = engine.clone();
  let ast = ast.clone();

  let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| -> Result<Tokens> {
    let dyn_args: Vec<Dynamic> = args.into_iter().map(arg_to_dynamic).collect();
    let ret: Dynamic = body
      .call::<Dynamic>(&engine, &ast, dyn_args)
      .map_err(|e| Error::from(format!("script macro {cs_name}: {e}")))?;
    Ok(mouth::tokenize_internal(&dynamic_to_string(ret)))
  });

  def_macro(cs, paramlist, ExpansionBody::Closure(closure), None)?;
  Ok(())
}

/// The imperative-body replacement closure: publishes the active context, calls
/// the body as `|document, arg1, …|`, pops the context. Shared by every
/// constructor form that uses a closure body.
pub(super) fn closure_replacement(
  body: FnPtr,
  engine: Rc<Engine>,
  ast: Rc<AST>,
  cs_name: String,
) -> ReplacementClosure {
  Rc::new(
    move |document: &mut Document, args: &Vec<Option<Digested>>, props| -> Result<()> {
      // Each `with` is a short borrow; the body's document ops re-borrow freshly.
      CTOR_CTX.with(|c| {
        c.borrow_mut().push(CtorCtx { document, props });
      });
      // `document` first (Perl's `$_[0]`), then each digested arg as a handle.
      let mut call_args: Vec<Dynamic> = Vec::with_capacity(args.len() + 1);
      call_args.push(Dynamic::from(DocProxy));
      for a in args {
        call_args.push(match a {
          Some(d) => Dynamic::from(d.clone()),
          None => Dynamic::UNIT,
        });
      }
      let result = body.call::<Dynamic>(&engine, &ast, call_args);
      CTOR_CTX.with(|c| {
        c.borrow_mut().pop();
      });
      result
        .map(|_| ())
        .map_err(|e| Error::from(format!("script constructor {cs_name}: {e}")))
    },
  )
}

/// The string-template replacement closure. The template is parsed **once** here
/// (at wire time) into the shared `ReplacementOp` AST — the same parser the
/// compile-time codegen uses (#171) — and the cached AST is interpreted per
/// invocation. This eliminates the former per-invocation byte-scan and removes
/// the second, divergent template implementation.
pub(super) fn template_replacement(template: &str) -> Result<ReplacementClosure> {
  let ops = Rc::new(replacement::parse_replacement(template)?);
  Ok(Rc::new(move |document: &mut Document, args, props| {
    replacement::apply_ops(&ops, document, args, props)
  }))
}

/// Install one `DefConstructor` (imperative body, no options) via the shared
/// `ConstructorBuilder`.
pub(super) fn wire_constructor(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  body: FnPtr,
) -> Result<()> {
  let repl = closure_replacement(body, engine.clone(), ast.clone(), proto.to_string());
  ConstructorBuilder::new(proto)?.replacement(repl).install()
}

/// Install a `DefConstructor` with an option bag (`#{ mode, afterDigest, … }`)
/// through the shared `ConstructorBuilder` — the *same* builder the
/// `DefConstructor!` macro targets, so the two front-ends cannot drift. Scalar
/// options route through the builder's single-source `set_option`; closure
/// options through its typed setters (here, `after_digest`).
pub(super) fn wire_constructor_opts(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  repl: ConstructorRepl,
  opts: Map,
) -> Result<()> {
  let replacement = match repl {
    ConstructorRepl::Template(t) => template_replacement(&t)?,
    ConstructorRepl::Closure(b) => {
      closure_replacement(b, engine.clone(), ast.clone(), proto.to_string())
    },
  };
  let builder = ConstructorBuilder::new(proto)?.replacement(replacement);
  apply_opts(builder, opts, engine, ast)?.install()
}

/// Install one `DefEnvironment` registration (template or imperative body, with
/// an option bag — possibly empty) through the shared `EnvironmentBuilder`.
/// The imperative body reaches `#body` via `document.absorbProperty("body")`.
pub(super) fn wire_environment(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  repl: ConstructorRepl,
  opts: Map,
) -> Result<()> {
  let replacement = match repl {
    ConstructorRepl::Template(t) => template_replacement(&t)?,
    ConstructorRepl::Closure(b) => {
      closure_replacement(b, engine.clone(), ast.clone(), proto.to_string())
    },
  };
  let builder = EnvironmentBuilder::new(proto)?.replacement(replacement);
  apply_opts(builder, opts, engine, ast)?.install()
}

/// The builder surface the option-bag loop needs — implemented for both core
/// builders so [`apply_opts`] is written once. (A local trait over the foreign
/// builder types; new closure options are added in `apply_opts` + one impl line
/// per builder.)
pub(super) trait BindingBuilder: Sized {
  fn set_option(self, key: &str, value: OptionValue) -> Result<Self>;
  fn after_digest(self, hook: DigestionClosure) -> Self;
  fn after_digest_begin(self, hook: DigestionClosure) -> Self;
  fn before_digest(self, hook: BeforeDigestClosure) -> Self;
  fn before_digest_end(self, hook: BeforeDigestClosure) -> Self;
  fn before_construct(self, hook: ConstructionClosure) -> Self;
  fn after_construct(self, hook: ConstructionClosure) -> Self;
  fn properties(self, props: PropertiesClosure) -> Self;
  fn reversion(self, rev: Reversion) -> Self;
  fn font(self, font: FontDirective) -> Self;
  fn sizer(self, sizer: latexml_core::definition::SizingClosure) -> Self;
  // (`install` stays inherent on each builder: call sites get the concrete
  // type back from `apply_opts`, so a trait method would be dead code.)
}

macro_rules! impl_binding_builder {
  ($t:ty) => {
    impl BindingBuilder for $t {
      fn set_option(self, key: &str, value: OptionValue) -> Result<Self> {
        <$t>::set_option(self, key, value)
      }
      fn after_digest(self, hook: DigestionClosure) -> Self { <$t>::after_digest(self, hook) }
      fn after_digest_begin(self, hook: DigestionClosure) -> Self {
        <$t>::after_digest_begin(self, hook)
      }
      fn before_digest(self, hook: BeforeDigestClosure) -> Self { <$t>::before_digest(self, hook) }
      fn before_digest_end(self, hook: BeforeDigestClosure) -> Self {
        <$t>::before_digest_end(self, hook)
      }
      fn before_construct(self, hook: ConstructionClosure) -> Self {
        <$t>::before_construct(self, hook)
      }
      fn after_construct(self, hook: ConstructionClosure) -> Self {
        <$t>::after_construct(self, hook)
      }
      fn properties(self, props: PropertiesClosure) -> Self { <$t>::properties(self, props) }
      fn reversion(self, rev: Reversion) -> Self { <$t>::reversion(self, rev) }
      fn font(self, font: FontDirective) -> Self { <$t>::font(self, font) }
      fn sizer(self, sizer: latexml_core::definition::SizingClosure) -> Self {
        <$t>::sizer(self, sizer)
      }
    }
  };
}
impl_binding_builder!(ConstructorBuilder);
impl_binding_builder!(EnvironmentBuilder);

/// Apply a Rhai option bag onto a builder: closure options become trampolines
/// (typed setters), `properties` also accepts a static map, scalars route
/// through the builder's single-source `set_option`. Shared by the constructor
/// and environment front-ends.
pub(super) fn apply_opts<B: BindingBuilder>(
  mut builder: B,
  opts: Map,
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
) -> Result<B> {
  for (key, val) in opts {
    if let Some(fp) = val.clone().try_cast::<FnPtr>() {
      // Closure option → typed builder setter (front-end builds the closure).
      match key.as_str() {
        "afterDigest" => {
          builder = builder.after_digest(after_digest_trampoline(fp, engine.clone(), ast.clone()));
        },
        "afterDigestBegin" => {
          builder =
            builder.after_digest_begin(after_digest_trampoline(fp, engine.clone(), ast.clone()));
        },
        "properties" => {
          builder = builder.properties(properties_trampoline(fp, engine.clone(), ast.clone()));
        },
        "beforeDigest" => {
          builder =
            builder.before_digest(before_digest_trampoline(fp, engine.clone(), ast.clone()));
        },
        "beforeDigestEnd" => {
          builder =
            builder.before_digest_end(before_digest_trampoline(fp, engine.clone(), ast.clone()));
        },
        "beforeConstruct" => {
          builder =
            builder.before_construct(construction_trampoline(fp, engine.clone(), ast.clone()));
        },
        "reversion" => {
          builder = builder.reversion(Reversion::Closure(reversion_trampoline(
            fp,
            engine.clone(),
            ast.clone(),
          )));
        },
        "sizer" => {
          builder = builder.sizer(sizer_trampoline(fp, engine.clone(), ast.clone()));
        },
        "afterConstruct" => {
          builder =
            builder.after_construct(construction_trampoline(fp, engine.clone(), ast.clone()));
        },
        // Unknown closure options are silently ignored (forgiving, like Perl
        // %options; the builder's set_option does the same for scalars).
        _ => {},
      }
    } else if key.as_str() == "properties" && val.is_map() {
      // Static property map (Perl's `properties => { key => value, … }`).
      let map = val.cast::<Map>();
      builder = builder.properties(Rc::new(move |_args| Ok(rhai_map_to_props(map.clone()))));
    } else if key.as_str() == "reversion" {
      // String reversion (`reversion => "\\begin{x}#1\\end{x}"`, "" disables).
      builder = builder.reversion(Reversion::Tokens(mouth::tokenize_internal(
        &dynamic_to_string(val),
      )));
    } else if key.as_str() == "font" && val.is_map() {
      // Partial-font directive (`font => { family => 'typewriter', … }`).
      let font = font_from_rhai_map(val.cast::<Map>());
      builder = builder.font(FontDirective::Asset(Rc::new(font)));
    } else if let Some(ov) = dynamic_to_option_value(&val) {
      // Scalar option → the builder's generic, single-source `set_option`.
      builder = builder.set_option(key.as_str(), ov)?;
    }
  }
  Ok(builder)
}

/// Build a `before/afterConstruct` trampoline: the body runs with the live
/// document published as active context (so `document.*` proxy calls work);
/// the whatsit itself is not yet exposed there (read-only marshaling TBD).
pub(super) fn construction_trampoline(
  fp: FnPtr,
  engine: Rc<Engine>,
  ast: Rc<AST>,
) -> ConstructionClosure {
  Rc::new(
    move |document: &mut Document, whatsit: &Whatsit| -> Result<()> {
      CTOR_CTX.with(|c| {
        c.borrow_mut().push(CtorCtx {
          document,
          props: whatsit.get_properties(),
        });
      });
      let result = fp.call::<Dynamic>(&engine, &ast, (Dynamic::from(DocProxy),));
      CTOR_CTX.with(|c| {
        c.borrow_mut().pop();
      });
      let _: Dynamic = result.map_err(|e| Error::from(format!("script afterConstruct: {e}")))?;
      Ok(())
    },
  )
}

/// Build an `afterDigest` trampoline: publish the whatsit so a parameterless body
/// can reach it via `whatsit()`, call the Rhai closure, pop.
pub(super) fn after_digest_trampoline(
  fp: FnPtr,
  engine: Rc<Engine>,
  ast: Rc<AST>,
) -> DigestionClosure {
  Rc::new(move |w: &mut Whatsit| -> Result<Vec<Digested>> {
    WHATSIT_CTX.with(|c| c.borrow_mut().push((w as *mut Whatsit, true)));
    let r = fp.call::<Dynamic>(&engine, &ast, ());
    WHATSIT_CTX.with(|c| {
      c.borrow_mut().pop();
    });
    let _: Dynamic = r.map_err(|e| Error::from(format!("script afterDigest: {e}")))?;
    Ok(Vec::new())
  })
}

/// Build a `beforeDigest` trampoline (Perl `beforeDigest => sub {…}`): runs the
/// parameterless Rhai closure before the constructor's arguments are digested
/// (state/font side-effects, e.g. `neutralize_font()`); contributes no boxes.
pub(super) fn before_digest_trampoline(
  fp: FnPtr,
  engine: Rc<Engine>,
  ast: Rc<AST>,
) -> BeforeDigestClosure {
  Rc::new(move || -> Result<Vec<Digested>> {
    let _: Dynamic = fp
      .call::<Dynamic>(&engine, &ast, ())
      .map_err(|e| Error::from(format!("script beforeDigest: {e}")))?;
    Ok(Vec::new())
  })
}

/// Build a `properties` trampoline (Perl `properties => sub {…}`): the Rhai
/// closure receives each digested argument as its TeX-source string (`()` for an
/// omitted optional) and returns a map; its entries become the whatsit's
/// properties as string `Stored`s, ready for the template's `#name` holes.
pub(super) fn properties_trampoline(
  fp: FnPtr,
  engine: Rc<Engine>,
  ast: Rc<AST>,
) -> PropertiesClosure {
  Rc::new(
    move |args: &Vec<Option<Digested>>| -> Result<SymHashMap<Stored>> {
      let dyn_args: Vec<Dynamic> = args
        .iter()
        .map(|a| match a {
          Some(d) => Dynamic::from(d.untex().unwrap_or_default()),
          None => Dynamic::UNIT,
        })
        .collect();
      let ret: Dynamic = fp
        .call::<Dynamic>(&engine, &ast, dyn_args)
        .map_err(|e| Error::from(format!("script properties: {e}")))?;
      if ret.is_unit() {
        Ok(SymHashMap::default())
      } else if ret.is_map() {
        Ok(rhai_map_to_props(ret.cast::<Map>()))
      } else {
        Err(Error::from(
          "script properties: body must return a map (or unit)",
        ))
      }
    },
  )
}

/// Map a Rhai scalar `Dynamic` to a builder `OptionValue` (string/bool/int).
pub(super) fn dynamic_to_option_value(v: &Dynamic) -> Option<OptionValue> {
  if v.is_string() {
    Some(OptionValue::Str(
      v.clone().into_string().unwrap_or_default(),
    ))
  } else if v.is_bool() {
    v.as_bool().ok().map(OptionValue::Bool)
  } else {
    v.as_int().ok().map(OptionValue::Int)
  }
}

/// Install one `DefPrimitive` registration as a native primitive whose body
/// runs at digestion time for side-effects (state assignments, etc.). The body
/// receives args as strings and its return value is ignored (pure side-effect);
/// the primitive contributes no boxes.
pub(super) fn wire_primitive(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  body: FnPtr,
  options: PrimitiveOptions,
) -> Result<()> {
  let (cs, paramlist) = parse_prototype(proto, true)?;
  let cs_name = cs.to_string();
  let engine = engine.clone();
  let ast = ast.clone();

  let closure: PrimitiveClosure = Rc::new(move |args: Vec<ArgWrap>| -> Result<Vec<Digested>> {
    let dyn_args: Vec<Dynamic> = args.into_iter().map(arg_to_dynamic).collect();
    let _: Dynamic = body
      .call::<Dynamic>(&engine, &ast, dyn_args)
      .map_err(|e| Error::from(format!("script primitive {cs_name}: {e}")))?;
    Ok(Vec::new())
  });

  def_primitive(
    cs,
    paramlist,
    Some(PrimitiveBody::Closure(closure)),
    options,
  )?;
  Ok(())
}

/// `DefMacro` with an option bag: scalars onto `ExpandableOptions`.
pub(super) fn wire_macro_opts(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  body: FnPtr,
  opts: Map,
) -> Result<()> {
  let (cs, paramlist) = parse_prototype(proto, true)?;
  let cs_name = cs.to_string();
  let engine = engine.clone();
  let ast = ast.clone();
  let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| -> Result<Tokens> {
    let dyn_args: Vec<Dynamic> = args.into_iter().map(arg_to_dynamic).collect();
    let ret: Dynamic = body
      .call::<Dynamic>(&engine, &ast, dyn_args)
      .map_err(|e| Error::from(format!("script macro {cs_name}: {e}")))?;
    Ok(mouth::tokenize_internal(&dynamic_to_string(ret)))
  });
  def_macro(
    cs,
    paramlist,
    ExpansionBody::Closure(closure),
    Some(expandable_options_from_map(opts)),
  )?;
  Ok(())
}

/// Map a Rhai option bag onto `ExpandableOptions` (the `DefMacro!` scalar set).
pub(super) fn expandable_options_from_map(opts: Map) -> ExpandableOptions {
  let mut o = ExpandableOptions::default();
  for (key, val) in opts {
    let b = val.as_bool().unwrap_or(false);
    match key.as_str() {
      "locked" => o.locked = b,
      "protected" => o.protected = b,
      "outer" => o.outer = b,
      "long" => o.long = b,
      "mathactive" => o.mathactive = b,
      "robust" => o.robust = b,
      "scope" => o.scope = scope_of(&dynamic_to_string(val.clone())),
      "alias" => o.alias = Some(dynamic_to_string(val.clone())),
      _ => {},
    }
  }
  o
}

/// Map a Rhai option bag onto `PrimitiveOptions` (the `DefPrimitive!` scalar set).
pub(super) fn primitive_options_from_map(opts: Map) -> PrimitiveOptions {
  let mut o = PrimitiveOptions::default();
  for (key, val) in opts {
    let b = val.as_bool().unwrap_or(false);
    match key.as_str() {
      "bounded" => o.bounded = b,
      "isPrefix" => o.is_prefix = b,
      "requireMath" => o.require_math = b,
      "forbidMath" => o.forbid_math = b,
      "robust" => o.robust = b,
      "locked" => o.locked = b,
      "enterHorizontal" => o.enter_horizontal = b,
      "leaveHorizontal" => o.leave_horizontal = b,
      "scope" => o.scope = scope_of(&dynamic_to_string(val.clone())),
      "mode" => o.mode = Some(dynamic_to_string(val.clone())),
      "nargs" => o.nargs = val.as_int().ok().map(|i| i as usize),
      _ => {},
    }
  }
  o
}

/// Install one `DefConditional` registration: the Rhai test closure receives
/// each argument as its TeX-source string and must return a bool (mirrors the
/// `DefConditional!` macro's `sub` test → `def_conditional`).
pub(super) fn wire_conditional(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  test: FnPtr,
) -> Result<()> {
  let (cs, paramlist) = parse_prototype(proto, true)?;
  let cs_name = cs.to_string();
  let engine = engine.clone();
  let ast = ast.clone();
  let closure: ConditionalClosure = Rc::new(move |args: Vec<ArgWrap>| -> Result<bool> {
    let dyn_args: Vec<Dynamic> = args.into_iter().map(arg_to_dynamic).collect();
    let ret: Dynamic = test
      .call::<Dynamic>(&engine, &ast, dyn_args)
      .map_err(|e| Error::from(format!("script conditional {cs_name}: {e}")))?;
    Ok(ret.as_bool().unwrap_or(false))
  });
  def_conditional(cs, paramlist, Some(closure), ConditionalOptions::default())
}

/// Install one `DefColumnType` registration: `"<char><params>"` →
/// `\NC@rewrite@<char>` expandable whose body trampolines into Rhai (mirrors
/// the `DefColumnType!` macro's lowering).
pub(super) fn wire_columntype(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  proto: &str,
  body: FnPtr,
) -> Result<()> {
  let mut chars = proto.chars();
  let Some(first_c) = chars.next() else {
    return Err(Error::from(
      "DefColumnType: expected a column-specifier character",
    ));
  };
  let rest: String = chars.collect::<String>().trim_start().to_string();
  let cs = latexml_core::T_CS!(latexml_core::s!("\\NC@rewrite@{first_c}"));
  let paramlist = if rest.is_empty() {
    None
  } else {
    latexml_core::common::def_parser::parse_parameters(&rest, &cs, true)?
  };
  let cs_name = cs.to_string();
  let engine = engine.clone();
  let ast = ast.clone();
  let closure: ExpansionClosure = Rc::new(move |args: Vec<ArgWrap>| -> Result<Tokens> {
    let dyn_args: Vec<Dynamic> = args.into_iter().map(arg_to_dynamic).collect();
    let ret: Dynamic = body
      .call::<Dynamic>(&engine, &ast, dyn_args)
      .map_err(|e| Error::from(format!("script column type {cs_name}: {e}")))?;
    Ok(mouth::tokenize_internal(&dynamic_to_string(ret)))
  });
  def_macro(cs, paramlist, ExpansionBody::Closure(closure), None)?;
  Ok(())
}

/// Map a Rhai option bag onto `MathPrimitiveOptions` (the `DefMath!` scalar
/// option set; unknown keys are ignored, matching Perl %options forgiveness).
pub(super) fn math_options_from_map(opts: Map) -> Result<MathPrimitiveOptions> {
  let mut o = MathPrimitiveOptions::default();
  for (key, val) in opts {
    let s = || dynamic_to_string(val.clone());
    let b = || val.as_bool().unwrap_or(false);
    match key.as_str() {
      "name" => o.name = Some(s()),
      "meaning" => o.meaning = Some(s()),
      "omcd" => o.omcd = Some(s()),
      "role" => o.role = Some(s()),
      "operator_role" => o.operator_role = Some(s()),
      "mathstyle" => o.mathstyle = Some(s()),
      "scriptpos" => o.scriptpos = Some(s()),
      "mode" => o.mode = Some(s()),
      "alias" => o.alias = Some(s()),
      "revert_as" => o.revert_as = Some(std::borrow::Cow::Owned(s())),
      "bounded" => o.bounded = b(),
      "requireMath" => o.require_math = b(),
      "forbidMath" => o.forbid_math = b(),
      "isPrefix" => o.is_prefix = b(),
      "reorder" => o.reorder = b(),
      "dual" => o.dual = b(),
      "nogroup" => o.nogroup = b(),
      "stretchy" => o.stretchy = val.as_bool().ok(),
      "operator_stretchy" => o.operator_stretchy = val.as_bool().ok(),
      "protected" => o.protected = b(),
      "robust" => o.robust = b(),
      "locked" => o.locked = b(),
      "hide_content_reversion" => o.hide_content_reversion = b(),
      "lpadding" => o.lpadding = val.as_int().ok().map(|i| i as usize),
      "rpadding" => o.rpadding = val.as_int().ok().map(|i| i as usize),
      _ => {},
    }
  }
  Ok(o)
}

/// Install a `DeclareOption` registration. Mirrors the `DeclareOption!` macro
/// (`setup_binding_language.rs`): record the option name in `@declaredoptions`
/// and define a `\ds@<opt>` primitive whose body runs when the option fires.
pub(super) fn wire_option(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  opt: &str,
  body: FnPtr,
) -> Result<()> {
  latexml_core::state::push_value("@declaredoptions", opt.to_string())?;
  let cs_proto = format!("\\ds@{opt}");
  wire_primitive(engine, ast, &cs_proto, body, PrimitiveOptions::default())
}

/// Install a string-template `DefConstructor` as a native constructor. The
/// template is parsed into the shared `ReplacementOp` AST once and interpreted by
/// the core runtime — no Rhai involved per invocation, so this path is fast.
pub(super) fn wire_constructor_template(proto: &str, template: String) -> Result<()> {
  ConstructorBuilder::new(proto)?
    .replacement(template_replacement(&template)?)
    .install()
}

/// Build a closure-form `reversion` trampoline: the body runs with the whatsit
/// published READ-ONLY (argString/propertyString available) and each digested
/// arg as its TeX-source string; its returned string is the reversion TeX.
pub(super) fn reversion_trampoline(
  fp: FnPtr,
  engine: Rc<Engine>,
  ast: Rc<AST>,
) -> latexml_core::definition::DigestedReversionClosure {
  Rc::new(move |w: &Whatsit, args: &Vec<Option<Digested>>| -> Result<Tokens> {
    WHATSIT_CTX.with(|c| c.borrow_mut().push((w as *const Whatsit as *mut Whatsit, false)));
    let dyn_args: Vec<Dynamic> = args
      .iter()
      .map(|a| match a {
        Some(d) => Dynamic::from(d.untex().unwrap_or_default()),
        None => Dynamic::UNIT,
      })
      .collect();
    let result = fp.call::<Dynamic>(&engine, &ast, dyn_args);
    WHATSIT_CTX.with(|c| {
      c.borrow_mut().pop();
    });
    let ret = result.map_err(|e| Error::from(format!("script reversion: {e}")))?;
    Ok(mouth::tokenize_internal(&dynamic_to_string(ret)))
  })
}

/// Build a `sizer` trampoline: the body sees the read-only whatsit and returns
/// "w;h;d" dimension specs (e.g. "10pt;8pt;2pt").
pub(super) fn sizer_trampoline(
  fp: FnPtr,
  engine: Rc<Engine>,
  ast: Rc<AST>,
) -> latexml_core::definition::SizingClosure {
  Rc::new(move |w: &Whatsit| -> Result<(Dimension, Dimension, Dimension)> {
    WHATSIT_CTX.with(|c| c.borrow_mut().push((w as *const Whatsit as *mut Whatsit, false)));
    let result = fp.call::<Dynamic>(&engine, &ast, ());
    WHATSIT_CTX.with(|c| {
      c.borrow_mut().pop();
    });
    let ret = result.map_err(|e| Error::from(format!("script sizer: {e}")))?;
    let spec = dynamic_to_string(ret);
    let mut parts = spec.split(';');
    let mut next_dim = || -> Result<Dimension> {
      Ok(Dimension::new_f64(Dimension::spec_to_f64(
        parts.next().unwrap_or("0pt").trim(),
      )?))
    };
    Ok((next_dim()?, next_dim()?, next_dim()?))
  })
}

/// The `DefAccent!` lowering: register the combiner mapping and the protected
/// `\<accent>` macro expanding to `\lx@applyaccent`.
pub(super) fn def_accent_impl(
  accent: &str,
  combining: &str,
  standalone: &str,
  below: bool,
) -> Result<()> {
  use latexml_core::parameter::{Parameter, Parameters};
  let comb_char = combining
    .chars()
    .next()
    .ok_or_else(|| Error::from("DefAccent: empty combining char"))?;
  let map = if below { "accent_combiner_below" } else { "accent_combiner_above" };
  latexml_core::state::assign_mapping(map, standalone, Some(combining.to_string()));
  let plain_param = Some(Parameters::new(vec![
    Parameter {
      name: arena::pin_static("Plain"),
      spec: arena::pin_static("{}"),
      ..Parameter::default()
    }
    .init()?,
  ]));
  def_macro(
    latexml_core::T_CS!(accent),
    plain_param,
    ExpansionBody::Tokens(latexml_core::Tokens!(
      latexml_core::T_CS!("\\lx@applyaccent"),
      latexml_core::T_OTHER!(accent),
      latexml_core::T_OTHER!(comb_char.to_string()),
      latexml_core::T_OTHER!(standalone),
      latexml_core::T_BEGIN!(),
      latexml_core::T_ARG!(1),
      latexml_core::T_END!()
    )),
    Some(ExpandableOptions { protected: true, ..ExpandableOptions::default() }),
  )?;
  Ok(())
}

/// The `DefMathLigature!` data-form lowering: walk `ntomatch` preceding
/// `ltx:XMTok` siblings matching the pattern (reversed), then rewrite to
/// `replacement` with the given role/name/meaning attributes.
pub(super) fn def_math_ligature_impl(pattern: &str, replacement: &str, opts: Map) {
  use latexml_core::ligature::{Ligature, MathLigatureOptions};
  let mut attr = MathLigatureOptions::default();
  for (key, val) in opts {
    let v = dynamic_to_string(val);
    match key.as_str() {
      "role" => attr.role = Some(v),
      "name" => attr.name = Some(v),
      "meaning" => attr.meaning = Some(v),
      _ => {},
    }
  }
  let chars: Vec<char> = pattern.chars().rev().collect();
  let ntomatch = chars.len();
  let replacement = replacement.to_string();
  let matcher: Option<latexml_core::ligature::LigatureMatcher> =
    Some(Rc::new(move |_document: &mut Document, node_opt: &mut libxml::tree::Node| {
      let mut node: libxml::tree::Node;
      let mut node_mut = node_opt;
      for c in chars.iter() {
        if latexml_core::common::model::with_node_qname(node_mut, |qname| qname != "ltx:XMTok")
          || node_mut.get_content() != c.to_string()
        {
          return Ok(None);
        }
        if let Some(sibling) = node_mut.get_prev_sibling() {
          node = sibling;
          node_mut = &mut node;
        } else {
          return Ok(None);
        }
      }
      if ntomatch > 0 {
        Ok(Some((ntomatch, replacement.clone(), attr.clone())))
      } else {
        Ok(None)
      }
    }));
  latexml_core::state::unshift_value("MATH_LIGATURES", vec![Ligature {
    id: latexml_core::state::generate_ligature_id(),
    matcher,
    code: None,
    font_test: None,
    regex: None,
  }]);
}

/// The `DefRewrite!`/`DefMathRewrite!` data-form lowering: build
/// `RewriteOptions` from the option bag and push the rule.
pub(super) fn def_rewrite_impl(kind: &str, opts: Map) -> Result<()> {
  use latexml_core::rewrite::{Rewrite, RewriteOptions};
  let mut o = RewriteOptions { is_math: kind == "math", ..RewriteOptions::default() };
  for (key, val) in opts {
    match key.as_str() {
      "label" => o.label = Some(dynamic_to_string(val)),
      "xpath" => o.xpath = Some(dynamic_to_string(val)),
      "select" => o.select = Some(dynamic_to_string(val)),
      "attributes" => {
        if val.is_map() {
          let mut m = rustc_hash::FxHashMap::default();
          for (k, v) in val.cast::<Map>() {
            m.insert(k.to_string(), dynamic_to_string(v));
          }
          o.attributes_map = Some(m);
        } else {
          o.attributes = Some(dynamic_to_string(val));
        }
      },
      "regexp" => o.regexp = Some(dynamic_to_string(val)),
      "match" => o.on_match = Some(mouth::tokenize_internal(&dynamic_to_string(val))),
      "scope" => o.scope = scope_of(&dynamic_to_string(val)),
      _ => {},
    }
  }
  latexml_core::state::push_value("DOCUMENT_REWRITE_RULES", Rewrite::new(kind, o))?;
  Ok(())
}

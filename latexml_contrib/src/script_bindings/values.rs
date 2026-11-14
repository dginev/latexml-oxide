//! Perl's value types as Rhai handles (DG ruling 2026-11-13): `Number`, `Dimension`, `Glue` and `MuGlue` are
//! latexml_core's own types, with Perl's constructors (Common/Number.pm, Dimension.pm, Glue.pm, Core/MuGlue.pm) and
//! methods (`valueOf`, `add`, `multiply`, …), plus Rhai's operators and comparisons over the same core arithmetic.
//! The register calls (`DefRegister`, `LookupRegister`, `AssignRegister`, `LookupDimension`) take and return them.

use latexml_core::{
  binding::def::{dialect::RegisterOptions, options::REGISTER_OPTIONS},
  common::{
    glue::{FillCode, Glue, is_glue_spec},
    muglue::MuGlue,
  },
  definition::register::{
    RegisterGetterClosure, RegisterSetterClosure, RegisterType, RegisterValue,
  },
};

use super::*;

type RhaiResult<T> = std::result::Result<T, Box<EvalAltResult>>;

/// A register value as a script value: the typed handle, a token register's `Tokens`/`Token` (Perl's
/// `$defn->valueOf`, Register.pm).
pub(super) fn register_value_to_dynamic(v: RegisterValue) -> Dynamic {
  match v {
    RegisterValue::Number(n) => Dynamic::from(n),
    RegisterValue::Dimension(d) => Dynamic::from(d),
    RegisterValue::Glue(g) => Dynamic::from(g),
    RegisterValue::MuGlue(g) => Dynamic::from(g),
    RegisterValue::MuDimension(d) => Dynamic::from(MuGlue::new(d.0)),
    RegisterValue::Token(t) => Dynamic::from(t),
    RegisterValue::Tokens(t) => Dynamic::from(t),
    RegisterValue::Pair(p) => Dynamic::from(p.to_string()),
  }
}

/// A script value as a register value, when it is one of Perl's register types (`%register_types`,
/// Package.pm:1327): a typed handle, `Tokens` or a `Token`; a Rhai int is a Number.
pub(super) fn dynamic_to_register_value(v: &Dynamic) -> Option<RegisterValue> {
  if let Some(n) = v.clone().try_cast::<Number>() {
    Some(n.into())
  } else if let Some(d) = v.clone().try_cast::<Dimension>() {
    Some(d.into())
  } else if let Some(g) = v.clone().try_cast::<Glue>() {
    Some(g.into())
  } else if let Some(g) = v.clone().try_cast::<MuGlue>() {
    Some(g.into())
  } else if let Some(t) = v.clone().try_cast::<Tokens>() {
    Some(t.into())
  } else if let Some(t) = v.clone().try_cast::<Token>() {
    Some(t.into())
  } else {
    v.as_int().ok().map(|i| Number(i).into())
  }
}

/// The TeX string of a typed handle (Perl's `toString`), `None` for any other value.
pub(super) fn value_string(d: &Dynamic) -> Option<String> {
  match dynamic_to_register_value(d)? {
    v @ (RegisterValue::Number(_)
    | RegisterValue::Dimension(_)
    | RegisterValue::Glue(_)
    | RegisterValue::MuGlue(_)) => (!d.is_int()).then(|| v.to_string()),
    _ => None,
  }
}

/// A spec the constructor cannot read: an Error (Perl reads it as 0 silently, or Warns for Glue), and 0.
fn unreadable(ctor: &str, spec: &str) -> RhaiResult<()> {
  let msg = format!("{ctor}(\"{spec}\"): not a {ctor} specification; using 0");
  engine::script_error("unexpected", spec, &msg, None)
}

fn number_of(spec: &str) -> Number {
  // Perl `int($number)` (Number.pm `new`): truncated; a trailing non-number Warns as Perl's `isn't numeric` and
  // reads the numeric prefix (`12abc` is 12).
  let trimmed = spec.trim_start();
  let prefix = numeric_prefix(trimmed);
  if prefix.len() != trimmed.len() || prefix.is_empty() {
    latexml_core::Warn!(
      "perl",
      "warn",
      format!("Argument \"{spec}\" isn't numeric in int")
    );
  }
  Number(prefix.parse::<f64>().map_or(0, |f| f.trunc() as i64))
}

/// The longest prefix of `s` Perl's string-to-number conversion reads: `[+-]?\d*(\.\d*)?([eE][+-]?\d+)?`.
fn numeric_prefix(s: &str) -> &str {
  let b = s.as_bytes();
  let digits = |mut i: usize| {
    while i < b.len() && b[i].is_ascii_digit() {
      i += 1;
    }
    i
  };
  let mut i = usize::from(matches!(b.first(), Some(b'+' | b'-')));
  let start = i;
  i = digits(i);
  let mut mantissa = i > start;
  if b.get(i) == Some(&b'.') {
    let j = digits(i + 1);
    mantissa |= j > i + 1;
    i = j;
  }
  if !mantissa {
    return "";
  }
  if matches!(b.get(i), Some(b'e' | b'E')) {
    let k = i + 1 + usize::from(matches!(b.get(i + 1), Some(b'+' | b'-')));
    let e = digits(k);
    if e > k {
      i = e;
    }
  }
  &s[..i]
}

fn dimension_of(spec: &str) -> RhaiResult<Dimension> {
  if Dimension::is_spec(spec) {
    Dimension::spec_to_f64(spec)
      .map(Dimension::new_f64)
      .map_err(rhai_err)
  } else {
    unreadable("Dimension", spec).map(|()| Dimension(0))
  }
}

fn glue_of(spec: &str) -> RhaiResult<Glue> {
  if is_glue_spec(spec) {
    Ok(Glue::new_spec(spec, None, None, None, None))
  } else {
    unreadable("Glue", spec).map(|()| Glue::default())
  }
}

fn muglue_of(spec: &str) -> RhaiResult<MuGlue> {
  if is_glue_spec(spec) {
    Ok(MuGlue::new_spec(spec, None, None, None, None))
  } else {
    unreadable("MuGlue", spec).map(|()| MuGlue::default())
  }
}

/// A fill order as Perl's `$pfill`: a name (`"fil"`), or 0-3.
fn fill_of(fill: &Dynamic) -> Option<FillCode> {
  match fill.as_int() {
    Ok(i) => usize::try_from(i).ok().and_then(FillCode::new),
    Err(_) => FillCode::from(&dynamic_to_string(fill.clone())),
  }
}

fn fill_index(fill: Option<FillCode>) -> i64 { fill.map_or(0, |f| f as i64 + 1) }

/// A string value of `DefRegister`, read for the declared type: an existing register's, else the spec's own form
/// (a `mu` unit is MuGlue, `plus`/`minus` Glue, another unit a Dimension; no unit is an Error, as Perl's untyped
/// register errs at `\the`).
fn register_value_of_spec(operation: &str, cs: &Token, spec: &str) -> RhaiResult<RegisterValue> {
  let declared = latexml_core::state::lookup_register_definition(cs).map(|defn| defn.register_type);
  let declared = match declared {
    Some(declared) => declared,
    None => {
      let unit = spec
        .trim()
        .trim_start_matches(|c: char| c == '-' || c == '+' || c == '.' || c.is_ascii_digit())
        .trim_start();
      if unit.is_empty() {
        let msg = format!(
          "{operation} ({cs}): \"{spec}\" has no unit, so no register type; write Number({spec}) or a unit; using \
           Dimension 0"
        );
        engine::script_error("unexpected", spec, &msg, None)?;
        return Ok(Dimension(0).into());
      }
      if unit.starts_with("mu") {
        RegisterType::MuGlue
      } else if spec.contains("plus") || spec.contains("minus") {
        RegisterType::Glue
      } else {
        RegisterType::Dimension
      }
    },
  };
  Ok(match declared {
    RegisterType::Number => number_of(spec).into(),
    RegisterType::Glue => glue_of(spec)?.into(),
    RegisterType::MuGlue => muglue_of(spec)?.into(),
    RegisterType::Tokens => mouth::tokenize_internal(TeXString::assembled(spec.to_string())).into(),
    _ => dimension_of(spec)?.into(),
  })
}

/// Perl `DefRegister($proto, $value, %options)` (Package.pm:1335-1358), as the `DefRegister!` macro: the value's
/// type is the register's; `getter`/`setter` closures as Register.pm calls them, `readonly`, `address`, `allocate`.
fn def_register(proto: &str, value: Dynamic, opts: Map) -> RhaiResult<()> {
  let (engine, ast) = current_script()?;
  let operation = format!("DefRegister ({proto})");
  check_options(
    &operation,
    REGISTER_OPTIONS,
    opts.keys().map(|k| k.as_str()),
  )
  .map_err(rhai_err)?;
  let (cs, params) = parse_prototype(proto, true).map_err(rhai_err)?;
  let value = if value.is_string() {
    register_value_of_spec("DefRegister", &cs, &dynamic_to_string(value))?
  } else {
    dynamic_to_register_value(&value).ok_or_else(|| {
      Box::<EvalAltResult>::from(format!(
        "{operation}: the value is a {}, not a Number, Dimension, Glue, MuGlue, Tokens or Token",
        value.type_name().rsplit("::").next().unwrap_or_default()
      ))
    })?
  };
  let mut options = RegisterOptions::default();
  for (key, val) in opts {
    match key.as_str() {
      "readonly" => options.readonly = perl_truthy(&val),
      "address" => options.address = Some(dynamic_to_string(val)),
      "allocate" => options.allocate = Some(dynamic_to_string(val)),
      "getter" => {
        options.getter = Some(register_getter(
          &engine,
          &ast,
          &cs,
          closure_of(&operation, val)?,
        ))
      },
      "setter" => {
        options.setter = Some(register_setter(
          &engine,
          &ast,
          &cs,
          closure_of(&operation, val)?,
        ))
      },
      _ => {},
    }
  }
  latexml_core::binding::def::dialect::def_register(cs, params, value, Some(options))
    .map_err(rhai_err)?;
  note_wired();
  Ok(())
}

fn closure_of(operation: &str, val: Dynamic) -> RhaiResult<FnPtr> {
  val
    .try_cast::<FnPtr>()
    .ok_or_else(|| format!("{operation}: getter and setter take a closure").into())
}

/// Perl's `&$getter(@args)` (Register.pm `valueOf`): the closure's value, which must be a register value.
fn register_getter(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  cs: &Token,
  f: FnPtr,
) -> RegisterGetterClosure {
  let (engine, ast, what) = (engine.clone(), ast.clone(), format!("register getter {cs}"));
  Rc::new(move |args: Vec<ArgWrap>| {
    let args: Vec<Dynamic> = args.into_iter().map(arg_value).collect();
    let ret = match call_deferred_body(&engine, &ast, &f, args) {
      Ok(ret) => ret,
      Err(e) => contain(&what, e, Dynamic::UNIT).ok()?,
    };
    let value = dynamic_to_register_value(&ret);
    if value.is_none() && !ret.is_unit() {
      let msg = format!(
        "{what} returned a {}, not a register value",
        ret.type_name()
      );
      let _ = engine::script_error("unexpected", &what, &msg, None);
    }
    value
  })
}

/// Perl's `&$setter($value, $scope, @args)` (Register.pm `setValue`): the typed value and the scope name.
fn register_setter(
  engine: &Rc<Engine>,
  ast: &Rc<AST>,
  cs: &Token,
  f: FnPtr,
) -> RegisterSetterClosure {
  let (engine, ast, what) = (engine.clone(), ast.clone(), format!("register setter {cs}"));
  Rc::new(
    move |value: RegisterValue, scope: Option<Scope>, args: Vec<ArgWrap>| {
      let scope = match scope {
        Some(Scope::Global) => Dynamic::from("global"),
        Some(Scope::Local) => Dynamic::from("local"),
        Some(Scope::Named(name)) => Dynamic::from(arena::to_string(name)),
        _ => Dynamic::UNIT,
      };
      let mut dyn_args = vec![register_value_to_dynamic(value), scope];
      dyn_args.extend(args.into_iter().map(arg_value));
      if let Err(e) = call_deferred_body(&engine, &ast, &f, dyn_args) {
        let _ = contain(&what, e, ());
      }
    },
  )
}

/// A register's parameters (`LookupRegister("\\count", 5)`), as the arguments its prototype reads.
fn register_args(args: rhai::Array) -> Vec<ArgWrap> {
  args
    .into_iter()
    .map(|a| match dynamic_to_register_value(&a) {
      Some(RegisterValue::Number(n)) => ArgWrap::Number(n),
      Some(RegisterValue::Dimension(d)) => ArgWrap::Dimension(d),
      _ => dynamic_to_arg(a),
    })
    .collect()
}

/// Perl `LookupRegister($cs, @parameters)` (Package.pm:1360-1368), as the `LookupRegister!` macro: the typed value;
/// `()` with Perl's Warning when `cs` is no register, undefined included.
fn lookup_register(cs: Dynamic, args: rhai::Array) -> RhaiResult<Dynamic> {
  let cs = token_arg("LookupRegister", cs)?;
  let Some(defn) = latexml_core::state::lookup_register_definition(&cs) else {
    let message = format!("The control sequence '{cs}' is not a register");
    latexml_core::Warn!("expected", "register", message);
    return Ok(Dynamic::UNIT);
  };
  Ok(
    latexml_core::definition::Definition::value_of(&*defn, register_args(args))
      .map_or(Dynamic::UNIT, register_value_to_dynamic),
  )
}

/// Perl `AssignRegister($cs, $value, $scope, @parameters)` (Package.pm:1397); a string value is read for the
/// register's type, as the `DefRegister` string form.
fn assign_register(cs: Dynamic, value: Dynamic, scope: &str, args: rhai::Array) -> RhaiResult<()> {
  let cs = token_arg("AssignRegister", cs)?;
  let value = if value.is_string() {
    register_value_of_spec("AssignRegister", &cs, &dynamic_to_string(value))?
  } else {
    dynamic_to_register_value(&value).ok_or_else(|| {
      Box::<EvalAltResult>::from(format!(
        "AssignRegister ({cs}): the value is no register value"
      ))
    })?
  };
  latexml_core::state::assign_register_token(&cs, value, scope_of(scope), register_args(args))
    .map_err(rhai_err)
}

/// Register the value types, their constructors, methods and operators, and the register calls.
pub(super) fn register_values(engine: &mut Engine) {
  engine.register_type_with_name::<Number>("Number");
  engine.register_type_with_name::<Dimension>("Dimension");
  engine.register_type_with_name::<Glue>("Glue");
  engine.register_type_with_name::<MuGlue>("MuGlue");

  // Perl's constructors: a spec string, or an integer (Number's value, the others' sp).
  engine.register_fn("Number", |n: i64| Number(n));
  engine.register_fn("Number", |f: f64| Number(f.trunc() as i64));
  engine.register_fn("Number", number_of);
  engine.register_fn("Dimension", |sp: i64| Dimension(sp));
  engine.register_fn("Dimension", |sp: f64| Dimension::new_f64(sp));
  engine.register_fn("Dimension", |s: &str| dimension_of(s));
  engine.register_fn("Glue", |sp: i64| Glue::new(sp));
  engine.register_fn("Glue", |s: &str| glue_of(s));
  // Perl's five-argument form: sp values and fill orders (`Glue(sp, plus, "fil", minus, 0)`).
  engine.register_fn(
    "Glue",
    |skip: i64, plus: i64, pfill: Dynamic, minus: i64, mfill: Dynamic| {
      Glue::new_full(
        skip,
        Some(plus),
        fill_of(&pfill),
        Some(minus),
        fill_of(&mfill),
      )
    },
  );
  engine.register_fn("MuGlue", |mu: i64| MuGlue::new(mu));
  engine.register_fn("MuGlue", |s: &str| muglue_of(s));
  engine.register_fn(
    "MuGlue",
    |skip: i64, plus: i64, pfill: Dynamic, minus: i64, mfill: Dynamic| {
      MuGlue::new_full(
        skip,
        Some(plus),
        fill_of(&pfill),
        Some(minus),
        fill_of(&mfill),
      )
    },
  );

  register_numeric::<Number>(engine);
  register_numeric::<Dimension>(engine);
  register_numeric::<Glue>(engine);
  register_numeric::<MuGlue>(engine);
  register_glue_sums::<Number>(engine);
  register_glue_sums::<Dimension>(engine);
  // Glue ± Glue and MuGlue ± MuGlue as TeX's `\advance` (tex.web §1239): the higher fill order wins, not as
  // Glue.pm `add` (KNOWN_PERL_ERRORS #1244) or MuGlue's inherited Number.pm `add` (OD #1030).
  engine.register_fn("add", |a: Glue, b: Glue| a.add_glue(b));
  engine.register_fn("+", |a: Glue, b: Glue| a.add_glue(b));
  engine.register_fn("subtract", |a: Glue, b: Glue| a.add_glue(b.negate()));
  engine.register_fn("-", |a: Glue, b: Glue| a.add_glue(b.negate()));
  engine.register_fn("add", muglue_add);
  engine.register_fn("+", muglue_add);
  engine.register_fn("subtract", |a: MuGlue, b: MuGlue| muglue_add(a, b.negate()));
  engine.register_fn("-", |a: MuGlue, b: MuGlue| muglue_add(a, b.negate()));
  for ptvalue in ["ptValue", "pxValue"] {
    let px = ptvalue == "pxValue";
    let value = move |sp: i64, prec: Option<u8>| {
      let d = Dimension(sp);
      if px {
        d.px_value(prec)
      } else {
        d.pt_value(prec)
      }
    };
    engine.register_fn(ptvalue, move |d: Dimension| value(d.0, Some(2)));
    // Number.pm `roundto` clamps the precision to 0..5.
    engine.register_fn(ptvalue, move |d: Dimension, prec: i64| {
      value(d.0, Some(prec_of(prec)))
    });
    engine.register_fn(ptvalue, move |g: Glue| value(g.skip, Some(2)));
    engine.register_fn(ptvalue, move |g: Glue, prec: i64| {
      value(g.skip, Some(prec_of(prec)))
    });
  }
  engine.register_fn("spValue", |d: Dimension| d.0);
  engine.register_fn("spValue", |g: Glue| g.skip);
  // Glue.pm's component getters.
  engine.register_fn("getStretch", |g: Glue| Dimension(g.plus.unwrap_or(0)));
  engine.register_fn("getShrink", |g: Glue| Dimension(g.minus.unwrap_or(0)));
  engine.register_fn("getStretchOrder", |g: Glue| Number(fill_index(g.pfill)));
  engine.register_fn("getShrinkOrder", |g: Glue| Number(fill_index(g.mfill)));

  engine.register_fn("DefRegister", |proto: &str, value: Dynamic| {
    def_register(proto, value, Map::new())
  });
  engine.register_fn("DefRegister", def_register);
  engine.register_fn("LookupRegister", |cs: Dynamic| {
    lookup_register(cs, Vec::new())
  });
  engine.register_fn("LookupRegister", lookup_register);
  engine.register_fn("AssignRegister", |cs: Dynamic, value: Dynamic| {
    assign_register(cs, value, "", Vec::new())
  });
  engine.register_fn(
    "AssignRegister",
    |cs: Dynamic, value: Dynamic, scope: &str| assign_register(cs, value, scope, Vec::new()),
  );
  engine.register_fn("AssignRegister", assign_register);
  // Perl `LookupDimension($cs)` (Package.pm:1373): a spec or a register's name; Dimension(0) with Perl's Warning.
  engine.register_fn("LookupDimension", |cs: &str| {
    latexml_core::state::lookup_dimension_cs(cs, false).unwrap_or_default()
  });
  engine.register_fn("LookupDimension", |cs: &str, noerror: bool| {
    latexml_core::state::lookup_dimension_cs(cs, noerror).unwrap_or_default()
  });
  // Perl's other argument forms: a Dimension is itself; a Token, Tokens or box reads as its string.
  engine.register_fn("LookupDimension", |d: Dimension| d);
  engine.register_fn("LookupDimension", |cs: Token| {
    latexml_core::state::lookup_dimension_cs(&cs.to_string(), false).unwrap_or_default()
  });
  engine.register_fn("LookupDimension", |cs: Tokens| {
    latexml_core::state::lookup_dimension_cs(&cs.to_string(), false).unwrap_or_default()
  });
  engine.register_fn("LookupDimension", |cs: Digested| {
    latexml_core::state::lookup_dimension_cs(&cs.to_string(), false).unwrap_or_default()
  });
  // CounterValue returned an int before it returned Perl's Number: its int uses keep working one release, Warned.
  register_number_int_transition(engine);
  // The macro's `LookupRegisterOrDefault`: the typed value, the type's zero (a Number 0) when `cs` is no register.
  engine.register_fn(
    "LookupRegisterOrDefault",
    |cs: Dynamic| -> RhaiResult<Dynamic> {
      let cs = token_arg("LookupRegisterOrDefault", cs)?;
      Ok(register_value_to_dynamic(
        latexml_core::state::lookup_register_definition(&cs)
          .and_then(|defn| latexml_core::definition::Definition::value_of(&*defn, Vec::new()))
          .unwrap_or_default(),
      ))
    },
  );
}

fn prec_of(prec: i64) -> u8 { prec.clamp(0, 5) as u8 }

/// `Number ⊕ int` and `int ⊕ Number` as the int arithmetic and comparisons they were, each Warned once per load.
fn register_number_int_transition(engine: &mut Engine) {
  fn int(n: Number) -> i64 {
    warn_deprecated("Number as an int");
    n.0
  }
  engine.register_fn("+", |a: Number, b: i64| int(a) + b);
  engine.register_fn("+", |a: i64, b: Number| a + int(b));
  engine.register_fn("-", |a: Number, b: i64| int(a) - b);
  engine.register_fn("-", |a: i64, b: Number| a - int(b));
  engine.register_fn("==", |a: Number, b: i64| int(a) == b);
  engine.register_fn("==", |a: i64, b: Number| a == int(b));
  engine.register_fn("!=", |a: Number, b: i64| int(a) != b);
  engine.register_fn("!=", |a: i64, b: Number| a != int(b));
  engine.register_fn("<", |a: Number, b: i64| int(a) < b);
  engine.register_fn("<=", |a: Number, b: i64| int(a) <= b);
  engine.register_fn(">", |a: Number, b: i64| int(a) > b);
  engine.register_fn(">=", |a: Number, b: i64| int(a) >= b);
}

/// MuGlue + MuGlue as TeX adds glue (tex.web §1239): Glue's addition over the mu components.
fn muglue_add(a: MuGlue, b: MuGlue) -> MuGlue {
  let as_glue = |m: MuGlue| Glue::new_full(m.skip, m.plus, m.pfill, m.minus, m.mfill);
  let g = as_glue(a).add_glue(as_glue(b));
  MuGlue::new_full(g.skip, g.plus, g.pfill, g.minus, g.mfill)
}

/// Number.pm's methods every value type shares, its operators, and the comparisons by `valueOf`.
fn register_numeric<T>(engine: &mut Engine)
where T:
    NumericOps + Copy + PartialEq + std::fmt::Display + Into<RegisterValue> + Send + Sync + 'static
{
  engine.register_fn("valueOf", |v: T| v.value_of());
  engine.register_fn("toString", |v: &mut T| v.to_string());
  engine.register_fn("to_string", |v: &mut T| v.to_string());
  engine.register_fn("toAttribute", |v: T| v.to_attribute());
  engine.register_fn("negate", |v: T| v.negate());
  engine.register_fn("-", |v: T| v.negate());
  engine.register_fn("absolute", |v: T| v.absolute());
  engine.register_fn("sign", |v: T| i64::from(v.sign()));
  engine.register_fn(
    "smaller",
    |a: T, b: T| if a.value_of() < b.value_of() { a } else { b },
  );
  engine.register_fn(
    "larger",
    |a: T, b: T| if a.value_of() > b.value_of() { a } else { b },
  );
  // `multiply`/`divide` by a number (Number.pm: `int($self * $other)`, `int($self / $other)`), or by a Number.
  engine.register_fn("multiply", |v: T, n: i64| v.multiply(Number(n)));
  engine.register_fn("*", |v: T, n: i64| v.multiply(Number(n)));
  engine.register_fn("multiply", |v: T, f: f64| v.multiply(Float(f)));
  engine.register_fn("*", |v: T, f: f64| v.multiply(Float(f)));
  engine.register_fn("multiply", |v: T, n: Number| v.multiply(n));
  engine.register_fn("divide", |v: T, n: i64| v.divide(Number(n)));
  engine.register_fn("/", |v: T, n: i64| v.divide(Number(n)));
  engine.register_fn("divide", |v: T, n: Number| v.divide(n));
  engine.register_fn("divideround", |v: T, n: i64| v.divideround(Number(n)));
  engine.register_fn("==", |a: T, b: T| a == b);
  engine.register_fn("!=", |a: T, b: T| a != b);
  engine.register_fn("<", |a: T, b: T| a.value_of() < b.value_of());
  engine.register_fn("<=", |a: T, b: T| a.value_of() <= b.value_of());
  engine.register_fn(">", |a: T, b: T| a.value_of() > b.value_of());
  engine.register_fn(">=", |a: T, b: T| a.value_of() >= b.value_of());
  engine::register_string_tests::<T>(engine, "value == string");
}

/// `add`/`subtract` (and `+`/`-`) of `T` with itself, and of a Glue with `T`: the sum of `valueOf`s, a Glue's
/// stretch and shrink kept, as TeX's `\advance` (Perl's Number.pm `subtract` drops them; OD #1030).
fn register_glue_sums<T>(engine: &mut Engine)
where T: NumericOps + Copy + Send + Sync + 'static {
  engine.register_fn("add", |a: T, b: T| a.add(b));
  engine.register_fn("+", |a: T, b: T| a.add(b));
  engine.register_fn("subtract", |a: T, b: T| a.subtract(b));
  engine.register_fn("-", |a: T, b: T| a.subtract(b));
  engine.register_fn("add", |a: Glue, b: T| a.add(b));
  engine.register_fn("+", |a: Glue, b: T| a.add(b));
  engine.register_fn("subtract", |a: Glue, b: T| a.subtract(b));
  engine.register_fn("-", |a: Glue, b: T| a.subtract(b));
}

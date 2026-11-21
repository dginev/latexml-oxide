use latexml_engine::latex_constructs::define_optional_internal;

use crate::prelude::*;

LoadDefinitions!({
  DefKeyVal!("xargs", "usedefault", "", "");
  // xargs uses positional numeric keys (1, 2, ..., for argument defaults)
  // plus `addprefix` (\global et al). Perl `xargs.sty.ltxml:27` only
  // registers `usedefault`; the rest ride on Info-level pass-through
  // (silent under Perl). Rust-only divergence paired with `21e730e71e`.
  for key in ["1", "2", "3", "4", "5", "6", "7", "8", "9", "addprefix"] {
    DefKeyVal!("xargs", key, "");
  }

  DefParameterType!(XArgsOptional, sub[_inner, extra] {
    let no_tks = &NO_TOKENS;
    let default = extra.first().unwrap_or(no_tks);
    let usedefault = extra.get(1).unwrap_or(no_tks);
    let value = read_optional(None)?.unwrap_or(Tokens!());
    if (!usedefault.is_empty() && value.to_string() == usedefault.to_string()) ||
        usedefault.is_empty() && value.to_string().is_empty() {
      default.clone()
    } else {
      value
    }
  },
  optional => true);

  // xargs.sty:20-25, the head of a command whose optional arguments are not only its first (:209-212), and the copy
  // against which the command sees it redefined (`Expandable::testopt_call`; hyperref.sty:549)
  RawTeX!(
    r"\newcommand*\@protected@testopt@xargs[1]{\ifx\protect\@typeset@protect\expandafter\xargs@read\else\@x@protect#1\fi}
\let\lx@xargs@protected@testopt@xargs\@protected@testopt@xargs"
  );

  // Macros

  DefPrimitive!(
    "\\CheckCommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}",
    None
  );

  DefPrimitive!("\\newcommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}", sub[(star,cs,nargs_opt,defaults,body)] {
    if !is_definable(&cs) {
      Info!("ignore", cs, &s!("Ignoring redefinition (\\newcommandx) of '{}'", cs));
    } else {
      let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) }
                  else {None};
      let nargs = if let Some(nargs_tks) = nargs_opt {
        nargs_tks.to_string().parse::<usize>()?
      } else {0};
      define_xargs(&cs, nargs, defaults.as_ref(), body, scope)?;
    }
  });

  DefPrimitive!("\\renewcommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}",
    sub[(star,cs,nargs_opt,defaults,body)] {
    let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) } else {None};
    let nargs = if let Some(nargs_tks) = nargs_opt {
      nargs_tks.to_string().parse::<usize>()?
    } else {0};
    define_xargs(&cs, nargs, defaults.as_ref(), body, scope)?;
  });

  DefPrimitive!("\\providecommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}",
    sub[(star,cs,nargs_opt,defaults,body)] {
    if is_definable(&cs) {
      let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) }
                  else {None};
      let nargs = if let Some(nargs_tks) = nargs_opt {
        nargs_tks.to_string().parse::<usize>()?
      } else {0};
      define_xargs(&cs, nargs, defaults.as_ref(), body, scope)?;
    }
  });

  DefPrimitive!("\\DeclareRobustCommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}",
  sub[(star, cs, nargs_opt, defaults, body)] {
    let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) } else {None};
    let munged = cs.with_str(|cstr| s!("{cstr} "));
    let mungedcs = T_CS!(munged);
    let nargs = if let Some(nargs_tks) = nargs_opt {
      nargs_tks.to_string().parse::<usize>()?
    } else {0};
    // xargs.sty:282-300 `\xargs@DRC`: `\cs` is `\protect\cs␣`, and `\cs␣` is made as `\newcommandx` makes a command,
    // its internal `\\cs␣` with it (`define_xargs`); Perl's convertLaTeXArgs takes the key-value list itself for the
    // default, so `\DeclareRobustCommandx\rx[2][1=d]{R#1#2}` prints `\rx{b}` as "R1=db" (KNOWN_PERL_ERRORS #1304).
    define_xargs(&mungedcs, nargs, defaults.as_ref(), body, scope)?;
    DefMacro!(cs, None, Tokens!(T_CS!("\\protect"), mungedcs), scope=>scope);
  });

  DefPrimitive!("\\newenvironmentx OptionalMatch:* {} [] OptionalKeyVals:xargs {}{}",
  sub[(star,cs_tks,nargs_opt,defaults,preamble,postamble)] {
    let cs_str = cs_tks.to_string();
    let cs_full = s!("\\{cs_str}");
    if lookup_definition(&T_CS!(&cs_full))?.is_some() {
      Info!("ignore", cs_full, s!("Ignoring redefinition (\\newenvironmentx) of Environment '{cs_full}'"));
      return Ok(Vec::new()); }
    let end_cs_full = s!("\\end{cs_str}");
    let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) } else {None};
    let nargs = if let Some(nargs_tks) = nargs_opt {
      nargs_tks.to_string().parse::<usize>()?
    } else {0};
    define_xargs(&T_CS!(cs_full), nargs, defaults.as_ref(), preamble, scope)?;
    DefMacro!(T_CS!(end_cs_full), None, postamble, scope => scope);
  });

  DefPrimitive!("\\renewenvironmentx OptionalMatch:* {} [] OptionalKeyVals:xargs {}{}",
  sub[(star, cs_tks, nargs_opt, defaults, preamble, postamble)] {
    let cs_str = cs_tks.to_string();
    let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) } else {None};
    let nargs = if let Some(nargs_tks) = nargs_opt {
      nargs_tks.to_string().parse::<usize>()?
    } else {0};
    define_xargs(&T_CS!(s!("\\{cs_str}")), nargs, defaults.as_ref(), preamble, scope)?;
    DefMacro!(T_CS!(s!("\\end{cs_str}")), None, postamble, scope => scope);
  });
});

// Utils

/// xargs.sty:205-212: `\cs` with an optional argument is `\@protected@testopt\cs\\cs{<default>}`, or
/// `\@protected@testopt@xargs\cs\\cs{<its reading>}` where one after the first is optional; the internal `\\cs`
/// here reads the arguments itself (sect08.rs `define_optional_internal`), and the call carries the first default.
fn define_xargs(
  cs: &Token,
  nargs: usize,
  defaults: Option<&KeyVals>,
  body: Tokens,
  scope: Option<Scope>,
) -> Result<()> {
  let optional = |i: usize| defaults.is_some_and(|kv| kv.get_value(&i.to_string()).is_some());
  let params = convert_xargs_args(nargs, defaults)?;
  let internal = if (1..=nargs).any(optional) {
    define_optional_internal(cs, params.clone(), &body, false, scope)?
  } else {
    None
  };
  let params = match internal {
    Some(internal) if (2..=nargs).any(optional) => params.map(|params| {
      params.with_testopt_call(TestoptCall {
        head: T_CS!("\\@protected@testopt@xargs"),
        kernel: T_CS!("\\lx@xargs@protected@testopt@xargs"),
        internal,
      })
    }),
    Some(internal) => params.map(|params| params.with_testopt_call(TestoptCall::kernel(internal))),
    None => params,
  };
  DefMacro!(*cs, params, body, scope => scope);
  Ok(())
}

// generate paramlist
fn convert_xargs_args(nargs: usize, keyval: Option<&KeyVals>) -> Result<Option<Parameters>> {
  let mut paramlist = Vec::new();
  for i in 1..=nargs {
    if let Some(val) = keyval
      .map(|kv| kv.get_value(&i.to_string()))
      .unwrap_or(None)
    {
      let usedef_opt = keyval
        .map(|kv| kv.get_value("usedefault"))
        .unwrap_or_default();
      if let Some(usedef) = usedef_opt {
        paramlist.push(Parameter::new(
          "XArgsOptional",
          &s!("XArgsOptional:{val}|{usedef}"),
          Some(vec![val.revert()?, usedef.revert()?]),
        )?);
      } else {
        paramlist.push(Parameter::new(
          "Optional",
          &s!("Optional:{val}"),
          Some(vec![val.revert()?]),
        )?);
      }
    } else {
      paramlist.push(Parameter::new("Plain", "{}", None)?);
    }
  }
  // (a command with an optional argument is `\@protected@testopt\cs…` or `\@protected@testopt@xargs\cs…`, which stays
  // unexpanded whenever `\protect` is not `\@typeset@protect` — xargs.sty:205-212 `\xargs@xargdef`/`\xargs@xargsdef`,
  // :20-25: inside ifthen's `\xdef` of a test, an `\equal` argument holding `\sequence{1}` keeps it, 2002.09653)
  if nargs > 0
    && (1..=nargs).any(|i| keyval.is_some_and(|kv| kv.get_value(&i.to_string()).is_some()))
    && let Some(first) = paramlist.first_mut()
  {
    first.testopt = true;
  }
  Ok(if paramlist.is_empty() {
    None
  } else {
    Some(Parameters::new(paramlist))
  })
}

/// generate command prefix (\global, \long, ...; but not \outer)
fn get_xargs_is_global(star: Option<Tokens>, keyval_opt: Option<&KeyVals>) -> bool {
  let mut prefix = String::new();
  if star.is_none() {
    // defaults to \long for unstarred form
    prefix = String::from("\\long");
  }
  if let Some(keyval) = keyval_opt
    && let Some(p) = keyval.get_value("addprefix")
  {
    prefix.push_str(&p.to_string());
  }
  // true if global in prefix
  prefix.contains("global")
}

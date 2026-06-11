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

  // Macros

  DefPrimitive!(
    "\\CheckCommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}",
    None
  );

  DefPrimitive!("\\newcommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}", sub[(star,cs,nargs_opt,defaults,body)] {
    if !is_definable(&cs) {
      Info!("ignore", cs, "Ignoring redefinition (\\newcommandx) of '{}'",cs);
    } else {
      let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) }
                  else {None};
      let nargs = if let Some(nargs_tks) = nargs_opt {
        nargs_tks.to_string().parse::<usize>()?
      } else {0};
      let cargs = convert_xargs_args(nargs, defaults.as_ref())?;
      DefMacro!(cs, cargs, body, scope => scope);
    }
  });

  DefPrimitive!("\\renewcommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}",
    sub[(star,cs,nargs_opt,defaults,body)] {
    let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) } else {None};
    let nargs = if let Some(nargs_tks) = nargs_opt {
      nargs_tks.to_string().parse::<usize>()?
    } else {0};
    DefMacro!(cs, convert_xargs_args(nargs, defaults.as_ref())?, body, scope=>scope);
  });

  DefPrimitive!("\\providecommandx OptionalMatch:* DefToken [] OptionalKeyVals:xargs {}",
    sub[(star,cs,nargs_opt,defaults,body)] {
    if is_definable(&cs) {
      let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) }
                  else {None};
      let nargs = if let Some(nargs_tks) = nargs_opt {
        nargs_tks.to_string().parse::<usize>()?
      } else {0};
      DefMacro!(cs, convert_xargs_args(nargs, defaults.as_ref())?, body, scope => scope);
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
    let defaults_tks = defaults.map(|kvs| kvs.revert().ok().unwrap_or_default());
    DefMacro!(mungedcs, convert_latex_args(nargs, defaults_tks)?, body, scope => scope);
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
    DefMacro!(T_CS!(cs_full), convert_xargs_args(nargs, defaults.as_ref())?,preamble, scope=>scope);
    DefMacro!(T_CS!(end_cs_full), None, postamble, scope => scope);
  });

  DefPrimitive!("\\renewenvironmentx OptionalMatch:* {} [] OptionalKeyVals:xargs {}{}",
  sub[(star, cs_tks, nargs_opt, defaults, preamble, postamble)] {
    let cs_str = cs_tks.to_string();
    let scope = if get_xargs_is_global(star, defaults.as_ref()) { Some(Scope::Global) } else {None};
    let nargs = if let Some(nargs_tks) = nargs_opt {
      nargs_tks.to_string().parse::<usize>()?
    } else {0};
    DefMacro!(T_CS!(s!("\\{cs_str}")), convert_xargs_args(nargs, defaults.as_ref())?, preamble, scope => scope);
    DefMacro!(T_CS!(s!("\\end{cs_str}")), None, postamble, scope => scope);
  });
});

// Utils

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

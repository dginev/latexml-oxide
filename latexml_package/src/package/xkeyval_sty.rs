use latexml_core::keyvals::SkipMissing;

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Pretend keyval loaded too
  AssignValue!("keyval.sty_loaded" => 1, Some(Scope::Global));

  // `\XKV@ifundefined{<csname>}{<undefined>}{<defined>}` — xkeyval's group-safe
  // existence test (xkvutils.tex L59, e-TeX branch). Our binding REPLACES
  // xkeyval.sty and never \input's xkvutils.tex, so this low-level internal was
  // missing — yet packages built on xkeyval use it DIRECTLY (e.g. extract.sty
  // L84: `\XKV@ifundefined{XTR@file}{...deactivate...}{}`). Ported verbatim from
  // xkvutils.tex (e-TeX `\ifcsname` branch; we always have e-TeX). Witness
  // 1611.02736 (extract.sty). `\@firstoftwo`/`\@secondoftwo` are kernel macros.
  TeX!(r"\def\XKV@ifundefined#1{\ifcsname#1\endcsname\expandafter\@secondoftwo\else\expandafter\@firstoftwo\fi}");

  // xkeyval's comma-list for-loop machinery (xkvutils.tex L44, L84-107).
  // Same gap as \XKV@ifundefined: packages built on xkeyval call these
  // directly (e.g. extract.sty L62 `\XKV@for@n{#1}\XTR@tempa\XTR@tempb` to
  // iterate the extract-env list). Ported verbatim. Witness 1611.02736.
  TeX!(r"\newtoks\XKV@tempa@toks
\long\def\XKV@for@n#1#2#3{%
  \XKV@tempa@toks{#1}\edef#2{\the\XKV@tempa@toks}%
  \ifx#2\@empty\XKV@for@break\else\expandafter\XKV@f@r\fi#2{#3}#1,\@nil,%
}%
\long\def\XKV@f@r#1#2#3,{%
  \XKV@tempa@toks{#3}\edef#1{\the\XKV@tempa@toks}%
  \ifx#1\@nnil\expandafter\@gobbletwo\else#2\expandafter\XKV@f@r\fi#1{#2}%
}%
\long\def\XKV@for@break #1\@nil,{\fi}%
\long\def\XKV@for@o#1{\expandafter\XKV@for@n\expandafter{#1}}%
\long\def\XKV@for@en#1#2#3{\XKV@f@r#2{#3}#1,\@nil,}%
\long\def\XKV@for@eo#1#2#3{\def#2{\XKV@f@r#2{#3}}\expandafter#2#1,\@nil,}");

  //
  // Basic \setkeys
  //

  // \setkeys[*][+][prefix]{keyset}[na]{keyvals}
  DefMacro!("\\setkeys OptionalMatch:* OptionalMatch:+ []{}[]", 
    sub[(star, plus, prefix_opt, keysets_tks, skip_opt)] {
    let prefix = prefix_opt.map(|p| do_expand(p).map(|t| t.to_string()))
      .transpose()?;
    let keysets_str = do_expand(keysets_tks)?.to_string();

    let skip_str = skip_opt.map(|s| s.to_string());
    let skip: Vec<String> = skip_str.iter()
      .flat_map(|s| s.split(',').map(|x| x.trim().to_string()))
      .collect();

    let skip_missing = if star.is_some() {
      SkipMissing::Store(T_CS!("\\XKV@rm"))
    } else {
      SkipMissing::None
    };

    let keysets: Vec<String> = keysets_str.split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();

    let mut keyvals = KeyVals::new(KeyvalsConfig {
      prefix,
      keysets,
      set_all: plus.is_some(),
      set_internals: true,
      skip,
      skip_missing,
      hook_missing: None,
    });
    keyvals.read_from(T_END!(), false)?;
    Ok(keyvals.set_keys_expansion())
  });

  // \setrmkeys[*][prefix]{keyset}[na]
  DefMacro!("\\setrmkeys OptionalMatch:* []{}[]", sub[(star, prefix_opt, keysets_tks, na_opt)] {    
    // expand and delete the list of tokens we need to work on
    let rm_tokens = do_expand(Tokens!(T_CS!("\\XKV@rm")))?;
    DefMacro!(T_CS!("\\XKV@rm"), None, Some(ExpansionBody::Tokens(Tokens!())));

    let mut tokens = Vec::new();
    tokens.push(T_CS!("\\setkeys"));
    if star.is_some() {
      tokens.push(T_OTHER!("*"));
    }
    if let Some(prefix) = prefix_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(prefix.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(keysets_tks.unlist());
    tokens.push(T_END!());
    if let Some(na) = na_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(na.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(rm_tokens.unlist());
    tokens.push(T_END!());
    Ok(Tokens::new(tokens))
  });

  //
  // Regular keys
  //

  // \define@key[prefix]{keyset}{key}[default]{code}
  DefPrimitive!("\\define@key[]{}{}[]{}", sub[(prefix_opt, keyset_tks, key_tks, default_opt, code)] {
    let prefix = prefix_opt.map(|p: Tokens| do_expand(p).map(|t| t.to_string()))
      .transpose()?;
    let default = default_opt.map(|d: Tokens| d.to_string());
    let keyset = do_expand(keyset_tks)?.to_string();
    let key = do_expand(key_tks)?.to_string();

    keyval::define(KeyvalConfig {
      prefix: prefix.as_deref().unwrap_or("KV"),
      keyset: &keyset,
      key: &key,
      vtype: "",
      default: default.as_deref(),
      code: Some(ExpansionBody::Tokens(code)),
      ..KeyvalConfig::default()
    })?;
  });

  //
  // Command keys
  //

  // \define@cmdkey[prefix]{keyset}[macroprefix]{key}[default]{code}
  DefPrimitive!("\\define@cmdkey[]{}[]{}[]{}", sub[(
    prefix_opt, keyset_tks, macroprefix_opt, key_tks, default_opt, code
  )] {
    let prefix = prefix_opt.map(|p: Tokens| do_expand(p).map(|t| t.to_string()))
      .transpose()?;
    let macroprefix = macroprefix_opt
      .map(|mp: Tokens| do_expand(mp).map(|t| t.to_string()))
      .transpose()?;
    let keyset = do_expand(keyset_tks)?.to_string();
    let key = do_expand(key_tks)?.to_string();
    let default = default_opt.map(|d: Tokens| d.to_string());

    keyval::define(KeyvalConfig {
      prefix: prefix.as_deref().unwrap_or("KV"),
      keyset: &keyset,
      key: &key,
      vtype: "",
      default: default.as_deref(),
      kind: Some("command"),
      macroprefix: macroprefix.as_deref(),
      code: Some(ExpansionBody::Tokens(code)),
      ..KeyvalConfig::default()
    })?;
  });

  // \define@cmdkeys[prefix]{keyset}[macroprefix]{keys}[default]
  DefPrimitive!("\\define@cmdkeys[]{}[]{}[]", sub[(
    prefix_opt, keyset_tks, macroprefix_opt, keys_tks, default_opt
  )] {
    let prefix = prefix_opt.map(|p: Tokens| do_expand(p).map(|t| t.to_string()))
      .transpose()?;
    let keyset = do_expand(keyset_tks)?.to_string();
    let macroprefix = macroprefix_opt
      .map(|mp: Tokens| do_expand(mp).map(|t| t.to_string()))
      .transpose()?;
    let default = default_opt.map(|d: Tokens| d.to_string());

    let keys_str = keys_tks.to_string();
    for key in keys_str.split(',') {
      let key = key.trim();
      if key.is_empty() { continue; }
      keyval::define(KeyvalConfig {
        prefix: prefix.as_deref().unwrap_or("KV"),
        keyset: &keyset,
        key,
        vtype: "",
        default: default.as_deref(),
        kind: Some("command"),
        macroprefix: macroprefix.as_deref(),
        code: Some(ExpansionBody::Tokens(Tokens!())),
        ..KeyvalConfig::default()
      })?;
    }
  });

  //
  // Choice keys
  //

  // \define@choicekey*+[prefix]{keyset}{key}[bin]{choices}[default]{code}{mismatch}
  // Two-phase: macro collects args, then calls internal primitive
  DefMacro!("\\define@choicekey OptionalMatch:* OptionalMatch:+ []{}{}[]{}[]{}", 
  sub[(star, plus, prefix_opt, keyset_tks, key_tks, bin_opt, choices_tks, default_opt, code_tks)] {
    let mut tokens = Vec::new();
    tokens.push(T_CS!("\\ltx@define@choicekey@int"));
    if star.is_some() { tokens.push(T_OTHER!("*")); }
    if plus.is_some() { tokens.push(T_OTHER!("+")); }
    if let Some(prefix) = prefix_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(prefix.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(keyset_tks.unlist());
    tokens.push(T_END!());
    tokens.push(T_BEGIN!());
    tokens.extend(key_tks.unlist());
    tokens.push(T_END!());
    if let Some(bin) = bin_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(bin.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(choices_tks.unlist());
    tokens.push(T_END!());
    if let Some(default) = default_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(default.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(code_tks.unlist());
    tokens.push(T_END!());
    // handle the optional mismatch (for the not-plus case)
    if plus.is_none() {
      tokens.push(T_BEGIN!());
      tokens.push(T_END!());
    }
    Ok(Tokens::new(tokens))
  });

  DefPrimitive!("\\ltx@define@choicekey@int OptionalMatch:* OptionalMatch:+ []{}{}[]{}[]{}{}", sub[(
    star, plus, prefix_opt, keyset_tks, key_tks, bin_opt, choices_tks, default_opt, code, mismatch
  )] {
    let prefix = prefix_opt.map(|p: Tokens| do_expand(p).map(|t| t.to_string()))
      .transpose()?;
    let default = default_opt.map(|d: Tokens| d.to_string());
    let keyset = do_expand(keyset_tks)?.to_string();
    let key = do_expand(key_tks)?.to_string();
    let choices_str = choices_tks.to_string();
    // Note: Perl uses Vec<&'static str> for choices. We can't do that easily.
    // The keyval::define function takes Vec<&'static str>, so we need to leak
    // the strings to create 'static references. This is intentional -- key definitions
    // live for the entire program lifetime.
    let choices: Vec<&'static str> = choices_str.split(',')
      .map(|s| &*Box::leak(s.trim().to_string().into_boxed_str()))
      .collect();
    let normalize = star.is_some();
    let bin_tks = bin_opt.filter(|t: &Tokens| !t.is_empty());

    let mismatch_body = if !mismatch.is_empty() {
      Some(ExpansionBody::Tokens(mismatch))
    } else {
      None
    };

    keyval::define(KeyvalConfig {
      prefix: prefix.as_deref().unwrap_or("KV"),
      keyset: &keyset,
      key: &key,
      vtype: "",
      default: default.as_deref(),
      kind: Some("choice"),
      normalize: Some(normalize),
      choices,
      bin: bin_tks,
      code: Some(ExpansionBody::Tokens(code)),
      mismatch: mismatch_body,
      ..KeyvalConfig::default()
    })?;
  });

  //
  // Bool keys
  //

  // \define@boolkey[+][prefix]{keyset}[macroprefix]{key}[default]{code}{mismatch}
  // Two-phase: macro collects args, then calls internal primitive
  DefMacro!("\\define@boolkey OptionalMatch:+ []{}[]{}[]{}", 
    sub[(plus, prefix_opt, keyset_tks, macroprefix_opt, key_tks, default_opt, code_tks)] {
    
    let mut tokens = Vec::new();
    tokens.push(T_CS!("\\define@boolkey@int"));
    if plus.is_some() { tokens.push(T_OTHER!("+")); }
    if let Some(prefix) = prefix_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(prefix.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(keyset_tks.unlist());
    tokens.push(T_END!());
    if let Some(macroprefix) = macroprefix_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(macroprefix.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(key_tks.unlist());
    tokens.push(T_END!());
    if let Some(default) = default_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(default.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(code_tks.unlist());
    tokens.push(T_END!());
    // handle the optional mismatch (for the not-plus case)
    if plus.is_none() {
      tokens.push(T_BEGIN!());
      tokens.push(T_END!());
    }
    Ok(Tokens::new(tokens))
  });

  DefPrimitive!("\\define@boolkey@int OptionalMatch:+ []{}[]{}[]{}{}", sub[(
    plus, prefix_opt, keyset_tks, macroprefix_opt, key_tks, default_opt, code, mismatch
  )] {
    let _ = plus;
    let prefix = prefix_opt.map(|p: Tokens| do_expand(p).map(|t| t.to_string()))
      .transpose()?;
    let macroprefix = macroprefix_opt
      .map(|mp: Tokens| do_expand(mp).map(|t| t.to_string()))
      .transpose()?;
    let default = default_opt.map(|d: Tokens| d.to_string());
    let keyset = do_expand(keyset_tks)?.to_string();
    let key = do_expand(key_tks)?.to_string();

    let mismatch_body = if !mismatch.is_empty() {
      Some(ExpansionBody::Tokens(mismatch))
    } else {
      None
    };

    keyval::define(KeyvalConfig {
      prefix: prefix.as_deref().unwrap_or("KV"),
      keyset: &keyset,
      key: &key,
      vtype: "",
      default: default.as_deref(),
      kind: Some("boolean"),
      macroprefix: macroprefix.as_deref(),
      code: Some(ExpansionBody::Tokens(code)),
      mismatch: mismatch_body,
      ..KeyvalConfig::default()
    })?;
  });

  // \define@boolkeys[prefix]{keyset}[macroprefix]{keys}[default]
  DefPrimitive!("\\define@boolkeys[]{}[]{}[]", sub[(
    prefix_opt, keyset_tks, macroprefix_opt, keys_tks, default_opt
  )] {
    let prefix = prefix_opt.map(|p: Tokens| do_expand(p).map(|t| t.to_string()))
      .transpose()?;
    let keyset = do_expand(keyset_tks)?.to_string();
    let macroprefix = macroprefix_opt
      .map(|mp: Tokens| do_expand(mp).map(|t| t.to_string()))
      .transpose()?;
    let default = default_opt.map(|d: Tokens| d.to_string());

    let keys_str = do_expand(keys_tks)?.to_string();
    for key in keys_str.split(',') {
      let key = key.trim();
      if key.is_empty() { continue; }
      keyval::define(KeyvalConfig {
        prefix: prefix.as_deref().unwrap_or("KV"),
        keyset: &keyset,
        key,
        vtype: "",
        default: default.as_deref(),
        kind: Some("boolean"),
        macroprefix: macroprefix.as_deref(),
        code: Some(ExpansionBody::Tokens(Tokens!())),
        ..KeyvalConfig::default()
      })?;
    }
  });

  //
  // Check for a defined key
  //

  // \key@ifundefined[prefix]{keyset}{key}{undefined}{defined}
  DefMacro!("\\key@ifundefined[]{}{}{}{}",
    sub[(prefix_opt, keysets_tks, key_tks, undefined, defined)] {
    let sprefix = prefix_opt
      .map(|p| do_expand(p).map(|t| t.to_string()))
      .transpose()?
      .unwrap_or_else(|| "KV".to_string());
    let skeysets_str = do_expand(keysets_tks)?.to_string();
    let skey = do_expand(key_tks)?.to_string();

    for skeyset in skeysets_str.split(',') {
      let skeyset = skeyset.trim();
      // Perl #2777 (2026-03-27): skip empty keyset names from leading,
      // trailing, or doubled commas.
      if skeyset.is_empty() { continue; }
      if keyval::has_keyval(&sprefix, skeyset, &skey) {
        let keyset_owned = skeyset.to_string();
        DefMacro!(T_CS!("\\XKV@tfam"), None, {
          Ok(Tokens::new(Explode!(keyset_owned)))
        });
        return Ok(defined);
      }
    }
    Ok(undefined)
  });

  //
  // Disabling keys
  //

  // \disable@keys[prefix]{keyset}{keys}
  DefMacro!("\\disable@keys[]{}{}", sub[(prefix_opt, keyset_tks, keys_tks)] {
    let sprefix = prefix_opt
      .map(|p| do_expand(p).map(|t| t.to_string()))
      .transpose()?
      .unwrap_or_else(|| "KV".to_string());
    let skeyset = do_expand(keyset_tks)?.to_string();
    let skeys = do_expand(keys_tks)?.to_string();

    for skey in skeys.split(',') {
      let skey = skey.trim();
      if !skey.is_empty() {
        keyval::disable_keyval(&sprefix, &skeyset, skey)?;
      }
    }
    Ok(Tokens!())
  });

  //
  // Option processing
  //

  // OptionalAngle parameter type: reads <...> delimited content.
  // Perl xkeyval.sty.ltxml L231-237: DefParameterType with reversion closure
  // that wraps the read value in `<...>` on reversion (so `\DeclareOptionX`
  // and friends' `tex=` attribute reconstructs the angle delimiters).
  // The DefParameterType! macro's `reversion =>` key is locked to a
  // Tokens-into-Option form used by DefConstructor, so assemble Parameter
  // manually and register via DefParameterTypeWO!.
  DefParameterTypeWO!(OptionalAngle, Parameter {
    name: pin_static("OptionalAngle"),
    optional: true,
    reader: reader!(_inner, _extra, {
      if if_next(T_OTHER!("<"))? {
        read_token()?;
        read_until_token(T_OTHER!(">"))
      } else {
        Ok(Tokens!())
      }
    }),
    reversion: Some(Rc::new(|tks: Vec<Token>, _params: Option<&Parameters>, _extra: &[Tokens]| -> Result<Tokens> {
      if tks.is_empty() {
        Ok(Tokens!())
      } else {
        let mut out: Vec<Token> = vec![T_OTHER!("<")];
        out.extend(tks);
        out.push(T_OTHER!(">"));
        Ok(Tokens::new(out))
      }
    })),
    ..Parameter::default()
  });

  //
  // DeclareOptionX
  //

  // \DeclareOptionX[*]
  DefMacro!("\\DeclareOptionX OptionalMatch:*", sub[(star)] {
    if star.is_some() {
      Ok(Tokens!(T_CS!("\\DeclareOptionX@int@star")))
    } else {
      Ok(Tokens!(T_CS!("\\DeclareOptionX@int@normal")))
    }
  });

  // \DeclareOptionX*{code}
  DefMacro!("\\DeclareOptionX@int@star {}", sub[(code)] {
    DefMacro!(T_CS!("\\XKV@doxs@int"), None,
      Some(ExpansionBody::Tokens(code)));
    DefMacro!("\\XKV@doxs {}", "\\edef\\CurrentOption{#1}\\XKV@doxs@int");
    Ok(Tokens!())
  });

  // \DeclareOptionX@int@normal [prefix]<keyset>{key}[default]{function}
  DefMacro!("\\DeclareOptionX@int@normal [] OptionalAngle {}[]{}", sub[args] {
    let [prefix_arg, keyset_arg, key_arg, default_arg, code_arg] :
      [ArgWrap; 5] = args.try_into().unwrap();
    let prefix_opt: Option<Tokens> = prefix_arg.owned_tokens();
    let keyset_opt: Option<Tokens> = keyset_arg.owned_tokens();
    let key_tks: Tokens = key_arg.owned_tokens().unwrap_or_default();
    let default_opt: Option<Tokens> = default_arg.owned_tokens();
    let code: Tokens = code_arg.owned_tokens().unwrap_or_default();

    // defaults may be passed with an empty argument
    let mut tokens = Vec::new();
    tokens.push(T_CS!("\\define@key"));
    if let Some(prefix) = prefix_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(prefix.unlist());
      tokens.push(T_OTHER!("]"));
    }
    // keyset defaults to current file name
    if let Some(keyset) = keyset_opt.filter(|t| !t.is_empty()) {
      tokens.push(T_BEGIN!());
      tokens.extend(keyset.unlist());
      tokens.push(T_END!());
    } else {
      tokens.push(T_BEGIN!());
      tokens.extend(Explode!(xkeyval_get_file_name()));
      tokens.push(T_END!());
    }
    tokens.push(T_BEGIN!());
    tokens.extend(key_tks.unlist());
    tokens.push(T_END!());
    if let Some(default) = default_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(default.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(code.unlist());
    tokens.push(T_END!());
    Ok(Tokens::new(tokens))
  });

  //
  // ExecuteOptionsX
  //

  // \ExecuteOptionsX [prefix]<keyset>[na]
  DefMacro!("\\ExecuteOptionsX [] OptionalAngle []", sub[args] {
    let [prefix_arg, keyset_arg, na_arg] :
      [ArgWrap; 3] = args.try_into().unwrap();
    let prefix_opt: Option<Tokens> = prefix_arg.owned_tokens();
    let keyset_opt: Option<Tokens> = keyset_arg.owned_tokens();
    let na_opt: Option<Tokens> = na_arg.owned_tokens();

    let mut tokens = Vec::new();
    tokens.push(T_CS!("\\setkeys"));
    if let Some(prefix) = prefix_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(prefix.unlist());
      tokens.push(T_OTHER!("]"));
    }
    if let Some(keyset) = keyset_opt.filter(|t| !t.is_empty()) {
      tokens.push(T_BEGIN!());
      tokens.extend(keyset.unlist());
      tokens.push(T_END!());
    } else {
      tokens.push(T_BEGIN!());
      tokens.extend(Explode!(xkeyval_get_file_name()));
      tokens.push(T_END!());
    }
    if let Some(na) = na_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(na.unlist());
      tokens.push(T_OTHER!("]"));
    }
    Ok(Tokens::new(tokens))
  });

  //
  // ProcessOptionsX
  //

  // \ProcessOptionsX[*] [prefix]<keysets>[na]
  DefMacro!("\\ProcessOptionsX OptionalMatch:* [] OptionalAngle []", sub[args] {
    let [star_arg, prefix_arg, keysets_arg, skip_arg] :
      [ArgWrap; 4] = args.try_into().unwrap();
    let star: Option<Tokens> = star_arg.owned_tokens();
    let prefix_opt: Option<Tokens> = prefix_arg.owned_tokens();
    let keysets_opt: Option<Tokens> = keysets_arg.owned_tokens();
    let skip_opt: Option<Tokens> = skip_arg.owned_tokens();

    let file_name = xkeyval_get_file_name();
    let keysets = if let Some(ks) = keysets_opt.filter(|t| !t.is_empty()) {
      ks
    } else {
      Tokens::new(Explode!(file_name))
    };

    // expand options for this file
    let opt_cs = T_CS!(s!("\\opt@{file_name}"));
    let options = do_expand(Tokens!(opt_cs))?.unlist();
    // check if we are inside a class file and fall back (if applicable)
    let is_star = star.is_some() && !xkeyval_is_in_class_file();

    let mut tokens = Vec::new();
    tokens.push(T_CS!("\\ProcessOptionsX@int"));
    if is_star { tokens.push(T_OTHER!("*")); }
    if let Some(prefix) = prefix_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(prefix.unlist());
      tokens.push(T_OTHER!("]"));
    }
    tokens.push(T_BEGIN!());
    tokens.extend(keysets.unlist());
    tokens.push(T_END!());
    if let Some(skip) = skip_opt {
      tokens.push(T_OTHER!("["));
      tokens.extend(skip.unlist());
      tokens.push(T_OTHER!("]"));
    }
    if is_star {
      tokens.push(T_BEGIN!());
      tokens.extend(
        do_expand(Tokens!(T_CS!("\\XKV@classoptionslist")))?.unlist()
      );
      tokens.push(T_END!());
    }
    tokens.push(T_BEGIN!());
    tokens.extend(options);
    tokens.push(T_END!());
    Ok(Tokens::new(tokens))
  });

  // \ProcessOptionsX@int [*] [prefix]{keysets}[na]
  DefMacro!("\\ProcessOptionsX@int OptionalMatch:* [] {} []", sub[(star, prefix_opt, keysets_tks, skip_opt)] {
    // store the missing macros if defined
    let hook_missing = if star.is_some() && has_meaning(&T_CS!("\\XKV@doxs")) {
      Some(T_CS!("\\XKV@doxs"))
    } else {
      None
    };

    // skip processing class options if we are inside a class file
    let is_star = star.is_some() && !xkeyval_is_in_class_file();

    let prefix = prefix_opt
      .map(|p| do_expand(p).map(|t| t.to_string()))
      .transpose()?;
    let skip: Vec<String> = skip_opt.map(|s| s.to_string())
      .iter()
      .flat_map(|s| s.split(',').map(|x| x.trim().to_string()))
      .collect();
    let keysets_str = keysets_tks.to_string();
    let keysets: Vec<String> = keysets_str.split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();

    let skip_missing = if hook_missing.is_some() {
      SkipMissing::All
    } else {
      SkipMissing::None
    };

    let mut keyvals = KeyVals::new(KeyvalsConfig {
      prefix,
      keysets,
      set_all: false,
      set_internals: true,
      skip,
      skip_missing,
      hook_missing,
    });

    // read class options (silenced) if star
    if is_star {
      keyvals.read_from(T_END!(), true)?;
    }
    // read package options
    keyvals.read_from(T_END!(), false)?;

    Ok(keyvals.set_keys_expansion())
  });

  //
  // Internals (empty by default)
  //

  DefMacro!(T_CS!("\\XKV@rm"), None, "");
  DefMacro!(T_CS!("\\XKV@prefix"), None, "");
  DefMacro!(T_CS!("\\XKV@tfam"), None, "");
  DefMacro!(T_CS!("\\XKV@header"), None, "");
  DefMacro!(T_CS!("\\XKV@tkey"), None, "");
  DefMacro!(T_CS!("\\XKV@fams"), None, "");
  DefMacro!(T_CS!("\\XKV@na"), None, "");

  //
  // Setup document class info
  //
  xkeyval_setup_document_class();

  //
  // Pointer System (Unsupported)
  //

  DefMacro!("\\savevalue{}", sub[_args] {
    Warn!("unexpected", "\\savevalue",
      "The xkeyval pointer system is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\gsavevalue{}", sub[args] {
    Warn!("unexpected", "\\gsavevalue",
      "The xkeyval pointer system is currently not supported. ");
    let [key] : [ArgWrap; 1] = args.try_into().unwrap();
    let key_tks: Tokens = key.owned_tokens().unwrap_or_default();
    Ok(key_tks)
  });

  DefMacro!("\\savekeys[]{}{}", sub[_args] {
    Error!("unexpected", "\\savekeys",
      "The xkeyval pointer system is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\gsavekeys[]{}{}", sub[_args] {
    Error!("unexpected", "\\gsavekeys",
      "The xkeyval pointer system is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\delsavekeys[]{}{}", sub[_args] {
    Error!("unexpected", "\\delsavekeys",
      "The xkeyval pointer system is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\gdelsavekeys[]{}{}", sub[_args] {
    Error!("unexpected", "\\gdelsavekeys",
      "The xkeyval pointer system is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\unsavekeys[]{}", sub[_args] {
    Error!("unexpected", "\\unsavekeys",
      "The xkeyval pointer system is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\gunsavekeys[]{}", sub[_args] {
    Error!("unexpected", "\\gunsavekeys",
      "The xkeyval pointer system is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\usevalue{}", sub[_args] {
    Error!("unexpected", "\\usevalue",
      "The xkeyval pointer system is currently not supported. ");
    Ok(Tokens!())
  });

  //
  // Presetting keys (Unsupported)
  //

  DefMacro!("\\presetkeys[]{}{}{}", sub[_args] {
    Warn!("unexpected", "\\presetkeys",
      "Presetting keys is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\gpresetkeys[]{}{}{}", sub[_args] {
    Warn!("unexpected", "\\gpresetkeys",
      "Presetting keys is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\delpresetkeys[]{}{}{}", sub[_args] {
    Warn!("unexpected", "\\delpresetkeys",
      "Presetting keys is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\gdelpresetkeys[]{}{}{}", sub[_args] {
    Warn!("unexpected", "\\gdelpresetkeys",
      "Presetting keys is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\unpresetkeys[]{}", sub[_args] {
    Warn!("unexpected", "\\unpresetkeys",
      "Presetting keys is currently not supported. ");
    Ok(Tokens!())
  });

  DefMacro!("\\gunpresetkeys[]{}", sub[_args] {
    Warn!("unexpected", "\\gunpresetkeys",
      "Presetting keys is currently not supported. ");
    Ok(Tokens!())
  });

  //
  // RawTeX block: \XKV@for@n, \XKV@f@r, \XKV@for@break
  //
  RawTeX!(r"\newtoks\XKV@tempa@toks");
  RawTeX!(concat!(
    "\\long\\def\\XKV@for@n#1#2#3{%\n",
    "\\XKV@tempa@toks{#1}\\edef#2{\\the\\XKV@tempa@toks}%\n",
    "\\ifx#2\\@empty\n",
    "\\XKV@for@break\n",
    "\\else\n",
    "\\expandafter\\XKV@f@r\n",
    "\\fi\n",
    "#2{#3}#1,\\@nil,%\n",
    "}"
  ));
  RawTeX!(concat!(
    "\\long\\def\\XKV@f@r#1#2#3,{%\n",
    "\\XKV@tempa@toks{#3}\\edef#1{\\the\\XKV@tempa@toks}%\n",
    "\\ifx#1\\@nnil\n",
    "\\expandafter\\@gobbletwo\n",
    "\\else\n",
    "#2\\expandafter\\XKV@f@r\n",
    "\\fi\n",
    "#1{#2}%\n",
    "}"
  ));
  RawTeX!(r"\long\def\XKV@for@break #1\@nil,{\fi}");
});

// Helper: get the current filename from \@currname.\@currext
fn xkeyval_get_file_name() -> String {
  let name = do_expand(Tokens!(T_CS!("\\@currname")))
    .map(|t| t.to_string())
    .unwrap_or_default();
  let ext = do_expand(Tokens!(T_CS!("\\@currext")))
    .map(|t| t.to_string())
    .unwrap_or_default();
  s!("{name}.{ext}")
}

// Helper: check if we are inside a class file
fn xkeyval_is_in_class_file() -> bool {
  let document_class = do_expand(Tokens!(T_CS!("\\XKV@documentclass")))
    .map(|t| t.to_string())
    .unwrap_or_default();
  let file_name = xkeyval_get_file_name();
  document_class == file_name
}

// Helper: Setup the XKV@documentclass and XKV@classoptionslist macros
fn xkeyval_setup_document_class() {
  let filelist = do_expand(Tokens!(T_CS!("\\@filelist")))
    .map(|t| t.to_string())
    .unwrap_or_default();
  let clsext = do_expand(Tokens!(T_CS!("\\@clsextension")))
    .map(|t| t.to_string())
    .unwrap_or_default();

  // Try to find the document class in @filelist (Perl approach)
  for file in filelist.split(',') {
    let file = file.trim();
    if file.is_empty() {
      continue;
    }
    let (_area, _base, ext) = pathname::split(file);
    // Perl xkeyval.sty.ltxml L254: `if ($ext eq $clsext)` — case-sensitive.
    if ext == clsext {
      let opt_cs = T_CS!(s!("\\opt@{file}"));
      if lookup_meaning(&opt_cs).is_some() {
        let file_tks = Tokens::new(Explode!(file));
        let _ = def_macro(
          T_CS!("\\XKV@documentclass"),
          None,
          Some(ExpansionBody::Tokens(file_tks)),
          None,
        );
        let_i(
          &T_CS!("\\XKV@classoptionslist"),
          &T_CS!("\\@classoptionslist"),
          None,
        );
        return;
      }
    }
  }
  // Fallback: check if \@classoptionslist is defined (non-\relax) even without @filelist.
  // In Rust, compiled bindings don't call \@addtofilelist, so @filelist may be empty,
  // but \@classoptionslist is set by input_definitions when loading a .cls.
  let classoptlist = do_expand(Tokens!(T_CS!("\\@classoptionslist")))
    .map(|t| t.to_string())
    .unwrap_or_default();
  if !classoptlist.is_empty() {
    // We have class options but couldn't find the class in @filelist.
    // Still set up XKV@classoptionslist from \@classoptionslist.
    let_i(
      &T_CS!("\\XKV@classoptionslist"),
      &T_CS!("\\@classoptionslist"),
      None,
    );
    // Determine document class name from stored value
    let doc_class = match lookup_value("document_class_filename") {
      Some(Stored::String(sym)) => with(sym, |s| s.to_string()),
      _ => String::new(),
    };
    let _ = def_macro(
      T_CS!("\\XKV@documentclass"),
      None,
      Some(ExpansionBody::Tokens(Tokens::new(Explode!(doc_class)))),
      None,
    );
    return;
  }
  // oops, we did not have a documentclass
  // Perl xkeyval.sty.ltxml L260: `Error('undefined', 'xkeyval', ...)`.
  // Was Warn! pre-fix — severity downgrade vs Perl. Use Error! to match.
  // IIFE wraps because the enclosing fn returns `()` and the Error!
  // macro's Fatal-cap path uses `return Err(...)`.
  let _ = (|| -> Result<()> {
    Error!(
      "undefined",
      "xkeyval",
      "Package xkeyval loaded before \\documentclass"
    );
    Ok(())
  })();
  let _ = def_macro(
    T_CS!("\\XKV@documentclass"),
    None,
    Some(ExpansionBody::Tokens(Tokens!())),
    None,
  );
  let _ = def_macro(
    T_CS!("\\XKV@classoptionslist"),
    None,
    Some(ExpansionBody::Tokens(Tokens!())),
    None,
  );
}

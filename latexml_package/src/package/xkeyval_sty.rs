use latexml_core::keyvals::SkipMissing;

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Really load keyval, the way real `xkeyval.sty` does — OXIDIZED_DESIGN #95.
  //
  // Perl `xkeyval.sty.ltxml` L23 instead only PRETENDS
  // (`AssignValue('keyval.sty_loaded' => 1, 'global')`), so that keyval's plain
  // `\setkeys`/`\define@key` can never clobber the extended ones below. But that
  // flag is exactly what `Package.pm:loadLTXML` L2328-2330 and
  // `loadTeXDefinitions` L2363 gate on, so it also suppresses the RAW
  // `keyval.sty` that `keyval.sty.ltxml` reads — and keyval's internals live
  // ONLY there. `\KV@do` (keyval.sty L31) is the witness: raw `fancyvrb.sty`
  // L112-117 `\FV@UseKeyValues` calls it directly, so a document loading
  // xkeyval before fancyvrb lost it and `\DefineVerbatimEnvironment` reported
  // `Error:undefined:\KV@do` (issue #500; Perl 0.8.8 errors identically —
  // KNOWN_PERL_ERRORS #73). Real xkeyval has no such gap: `xkeyval.sty` L39
  // `\input xkeyval` pulls in the bundle's own `keyval.tex`, which defines
  // `\KV@do` at L52 — i.e. loading xkeyval genuinely provides keyval.
  //
  // Ordering is the real xkeyval's: keyval FIRST, then the extended
  // `\setkeys`/`\define@key`/… below override it. `RequirePackage` also sets
  // `keyval.sty_loaded`, so a later `\RequirePackage{keyval}` stays a no-op and
  // xkeyval keeps the last word, exactly as the pretense intended.
  RequirePackage!("keyval");

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
  // The implementation lives under the private `\lx@xkv@setkeys`; the public
  // `\setkeys` is an alias (below). Internal re-entry points (`\XKV@s@tkeys`,
  // `\setrmkeys`) target the private name so a package that overrides
  // `\setkeys` (xkeymask's mask dispatcher ends in `\XKV@s@tkeys`) cannot
  // loop back into its own front-end.
  DefMacro!("\\lx@xkv@setkeys OptionalMatch:* OptionalMatch:+ []{}[]",
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
  // Public `\setkeys` takes the real xkeyval front-end (xkeyval.tex L437) so
  // the preset hooks in \XKV@setkeys fire; the chain still lands in the Rust
  // reader via the \XKV@s@tkeys shim. Internal re-entry points (\setrmkeys,
  // \XKV@s@tkeys) keep targeting \lx@xkv@setkeys directly (anti-loop).
  RawTeX!(r"\def\setkeys{\XKV@testopta{\XKV@testoptc\XKV@setkeys}}");

  // \setrmkeys[*][prefix]{keyset}[na]
  DefMacro!("\\setrmkeys OptionalMatch:* []{}[]", sub[(star, prefix_opt, keysets_tks, na_opt)] {    
    // expand and delete the list of tokens we need to work on
    let rm_tokens = do_expand(Tokens!(T_CS!("\\XKV@rm")))?;
    DefMacro!(T_CS!("\\XKV@rm"), None, Some(ExpansionBody::Tokens(Tokens!())));

    let mut tokens = Vec::new();
    tokens.push(T_CS!("\\lx@xkv@setkeys"));
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
    let default_tks_cfg: Option<Tokens> = default_opt;
    let default = default_tks_cfg.clone().map(|d| d.to_string());
    let keyset = do_expand(keyset_tks)?.to_string();
    let key = do_expand(key_tks)?.to_string();

    keyval::define(KeyvalConfig {
      prefix: prefix.as_deref().unwrap_or("KV"),
      keyset: &keyset,
      key: &key,
      vtype: "",
      default: default.as_deref(),
      default_tks: default_tks_cfg.clone(),
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
    let default_tks_cfg: Option<Tokens> = default_opt;
    let default = default_tks_cfg.clone().map(|d| d.to_string());

    keyval::define(KeyvalConfig {
      prefix: prefix.as_deref().unwrap_or("KV"),
      keyset: &keyset,
      key: &key,
      vtype: "",
      default: default.as_deref(),
      default_tks: default_tks_cfg.clone(),
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
    let default_tks_cfg: Option<Tokens> = default_opt;
    let default = default_tks_cfg.clone().map(|d| d.to_string());

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
      default_tks: default_tks_cfg.clone(),
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
    let default_tks_cfg: Option<Tokens> = default_opt;
    let default = default_tks_cfg.clone().map(|d| d.to_string());
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
      default_tks: default_tks_cfg.clone(),
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
    let default_tks_cfg: Option<Tokens> = default_opt;
    let default = default_tks_cfg.clone().map(|d| d.to_string());
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
      default_tks: default_tks_cfg.clone(),
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
    let default_tks_cfg: Option<Tokens> = default_opt;
    let default = default_tks_cfg.clone().map(|d| d.to_string());

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
      default_tks: default_tks_cfg.clone(),
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
    name: pin!("OptionalAngle"),
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
    tokens.push(T_CS!("\\lx@xkv@setkeys"));
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
  // Raw front-end scaffolding — verbatim from xkeyval.tex / xkvutils.tex /
  // keyval.tex. Packages clone `\setkeys`' argument parser out of xkeyval.tex
  // and call the internals directly: chessboard.sty L98-107
  // (`\XKV@testopta{\XKV@testoptc\board@XKVsetsinglekeys}` ending in
  // `\XKV@s@tkeys`), xkeymask.sty (`\XKV@setkeys`), xskak. The scaffolding is
  // ported faithfully; only `\XKV@s@tkeys` — the point where every path has
  // finished parsing and holds prefix/families/star/plus in XKV state — is a
  // thin shim onto our Rust `\setkeys` (guard:
  // `cluster_package_guards::xkeyval_internals`).
  //

  // xkvutils.tex L44-46: token registers (\XKV@tempa@toks is allocated by the
  // \XKV@for@n block above).
  RawTeX!(r"\newtoks\XKV@toks \newtoks\XKV@tempb@toks");
  // xkeyval.tex L51-58: depth counter + state conditionals.
  RawTeX!(
    r"\newcount\XKV@depth
\newif\ifXKV@st \newif\ifXKV@sg \newif\ifXKV@pl \newif\ifXKV@knf
\newif\ifXKV@rkv \newif\ifXKV@inpox \newif\ifXKV@preset"
  );
  // xkeyval.tex L68-71: message channels.
  RawTeX!(
    r"\def\XKV@warn#1{\message{xkeyval warning: #1}}
\def\XKV@err#1{\errmessage{xkeyval error: #1}}
\def\KV@errx{\XKV@err}
\let\KV@err\KV@errx"
  );
  // keyval.tex L40-52: `\KV@@sp@def` — define #1 as #2 with surrounding space
  // tokens stripped (the classic space-as-delimiter device, run verbatim).
  RawTeX!(
    r"\def\XKV@tempa#1{%
\long\def\KV@@sp@def##1##2{%
  \futurelet\XKV@resa\KV@@sp@d##2\@nil\@nil#1\@nil\relax##1}%
\long\def\KV@@sp@d{%
  \ifx\XKV@resa\@sptoken
    \expandafter\KV@@sp@b
  \else
    \expandafter\KV@@sp@b\expandafter#1%
  \fi}%
\long\def\KV@@sp@b#1##1 \@nil{\KV@@sp@c##1}%
  }
\XKV@tempa{ }
\long\def\KV@@sp@c#1\@nil#2\relax#3{\XKV@toks{#1}\edef#3{\the\XKV@toks}}"
  );
  // xkvutils.tex L47-48, L73-84: \fi-jumpers and single-character tests.
  RawTeX!(
    r"\long\def\XKV@afterfi#1\fi{\fi#1}
\long\def\XKV@afterelsefi#1\else#2\fi{\fi#1}
\long\def\@ifnextcharacter#1#2#3{%
  \@ifnextchar\bgroup
  {\@ifnextchar{#1}{#2}{#3}}%
  {\@ifncharacter{#1}{#2}{#3}}%
}
\long\def\@ifncharacter#1#2#3#4{%
  \if\string#1\string#4%
    \expandafter\@firstoftwo
  \else
    \expandafter\@secondoftwo
  \fi
  {#2}{#3}#4%
}"
  );
  // xkvutils.tex L127-148: macro/list accumulators.
  RawTeX!(
    r"\long\def\XKV@addtomacro@n#1#2{%
  \XKV@tempa@toks\expandafter{#1#2}%
  \edef#1{\the\XKV@tempa@toks}%
}
\def\XKV@addtomacro@o#1#2{%
  \expandafter\XKV@addtomacro@n\expandafter#1\expandafter{#2}%
}
\def\XKV@addtolist@n#1#2{%
  \ifx#1\@empty
    \XKV@addtomacro@n#1{#2}%
  \else
    \XKV@addtomacro@n#1{,#2}%
  \fi
}
\def\XKV@addtolist@o#1#2{%
  \ifx#1\@empty
    \XKV@addtomacro@o#1#2%
  \else
    \XKV@addtomacro@o#1{\expandafter,#2}%
  \fi
}
\def\XKV@addtolist@x#1#2{\edef#1{#1\ifx#1\@empty\else,\fi#2}}"
  );
  // xkvutils.tex L149-206: the selective sanitizer (only entered when a `=`
  // or `,` exists in the \meaning string but not as a catcode-12 token in the
  // actual list — i.e. active characters; the common path never runs it).
  RawTeX!(
    r"\def\@selective@sanitize{\@testopt\@s@lective@sanitize\@M}
\def\@s@lective@sanitize[#1]#2#3{%
  \begingroup
    \count@#1\relax\advance\count@\@ne
    \XKV@toks\expandafter{#3}%
    \def#3{#2}\@onelevel@sanitize#3%
    \edef#3{{#3}{\the\XKV@toks}}%
    \expandafter\@s@l@ctive@sanitize\expandafter#3#3%
    \expandafter\XKV@tempa@toks\expandafter{#3}%
  \expandafter\endgroup\expandafter\XKV@tempb@toks\expandafter{\the\XKV@tempa@toks}%
  \edef#3{\the\XKV@tempb@toks}%
}
\def\@s@l@ctive@sanitize#1#2#3{%
  \def\@i{\futurelet\@@tok\@ii}%
  \def\@ii{%
    \expandafter\@iii\meaning\@@tok\relax
    \ifx\@@tok\@s@l@ctive@sanitize
      \let\@@cmd\@gobble
    \else
      \ifx\@@tok\@sptoken
        \XKV@toks\expandafter{#1}\edef#1{\the\XKV@toks\space}%
        \def\@@cmd{\afterassignment\@i\let\@@tok= }%
      \else
        \let\@@cmd\@iv
      \fi
    \fi
    \@@cmd
  }%
  \def\@iii##1##2\relax{\if##1\@backslashchar\let\@@tok\relax\fi}%
  \def\@iv##1{%
    \toks@\expandafter{#1}\XKV@toks{##1}%
    \ifx\@@tok\bgroup
      \advance\count@\m@ne
      \ifnum\count@>\z@
        \begingroup
          \def#1{\expandafter\@s@l@ctive@sanitize
            \csname\string#1\endcsname{#2}}%
          \expandafter#1\expandafter{\the\XKV@toks}%
          \XKV@toks\expandafter\expandafter\expandafter
            {\csname\string#1\endcsname}%
          \edef#1{\noexpand\XKV@toks{\the\XKV@toks}}%
        \expandafter\endgroup#1%
      \fi
      \edef#1{\the\toks@{\the\XKV@toks}}%
      \advance\count@\@ne
      \let\@@cmd\@i
    \else
      \edef#1{\expandafter\string\the\XKV@toks}%
      \expandafter\in@\expandafter{#1}{#2}%
      \edef#1{\the\toks@\ifin@#1\else
              \ifx\@@tok\@sptoken\space\else\the\XKV@toks\fi\fi}%
      \edef\@@cmd{\noexpand\@i\ifx\@@tok\@sptoken\the\XKV@toks\fi}%
    \fi
    \@@cmd
  }%
  \let#1\@empty\@i#3\@s@l@ctive@sanitize
}"
  );
  // xkvutils.tex L207-236: sanitize checks + space-stripped list definer.
  RawTeX!(
    r"\long\def\XKV@checksanitizea#1#2{%
  \XKV@ch@cksanitize{#1}#2=%
  \ifin@\else\XKV@ch@cksanitize{#1}#2,\fi
  \ifin@\@selective@sanitize[0]{,=}#2\fi
}
\def\XKV@checksanitizeb#1#2{%
  \XKV@ch@cksanitize{#1}#2,%
  \ifin@\@selective@sanitize[0],#2\fi
}
\long\def\XKV@ch@cksanitize#1#2#3{%
  \XKV@tempa@toks{#1}\edef#2{\the\XKV@tempa@toks}%
  \@onelevel@sanitize#2%
  \@expandtwoargs\in@#3{#2}%
  \ifin@
    \long\def#2##1#3##2\@nil{%
      \XKV@tempa@toks{##2}\edef#2{\the\XKV@tempa@toks}%
      \ifx#2\@empty\else\in@false\fi
    }%
    #2#1#3\@nil
  \fi
  \XKV@tempa@toks{#1}\edef#2{\the\XKV@tempa@toks}%
}
\def\XKV@sp@deflist#1#2{%
  \let#1\@empty
  \XKV@for@n{#2}\XKV@resa{%
    \expandafter\KV@@sp@def\expandafter\XKV@resa\expandafter{\XKV@resa}%
    \XKV@addtomacro@o#1{\expandafter,\XKV@resa}%
  }%
  \ifx#1\@empty\else
    \expandafter\XKV@sp@d@flist\expandafter#1#1\@nil
  \fi
}
\def\XKV@sp@d@flist#1,#2\@nil{\def#1{#2}}"
  );
  // xkeyval.tex L72-97: star/plus tests, prefix/header builders, save/restore.
  RawTeX!(
    r"\def\XKV@ifstar#1{\@ifnextcharacter*{\@firstoftwo{#1}}}
\def\XKV@ifplus#1{\@ifnextcharacter+{\@firstoftwo{#1}}}
\def\XKV@makepf#1{%
  \KV@@sp@def\XKV@prefix{#1}%
  \def\XKV@resa{XKV}%
  \ifx\XKV@prefix\XKV@resa
    \XKV@err{`XKV' prefix is not allowed}%
    \let\XKV@prefix\@empty
  \else
    \edef\XKV@prefix{\ifx\XKV@prefix\@empty\else\XKV@prefix @\fi}%
  \fi
}
\def\XKV@makehd#1{%
  \expandafter\KV@@sp@def\expandafter\XKV@header\expandafter{#1}%
  \edef\XKV@header{%
    \XKV@prefix\ifx\XKV@header\@empty\else\XKV@header @\fi
  }%
}
\def\XKV@srstate#1#2{%
  \ifx\@empty#2\@empty\advance\XKV@depth\@ne\fi
  \XKV@for@n{XKV@prefix,XKV@fams,XKV@tkey,XKV@na,%
    ifXKV@st,ifXKV@pl,ifXKV@knf,CurrentOption}\XKV@resa{%
    \expandafter\let\csname\XKV@resa#1\expandafter
      \endcsname\csname\XKV@resa#2\endcsname
  }%
  \ifx\@empty#1\@empty\advance\XKV@depth\m@ne\fi
}"
  );
  // xkeyval.tex L98-127: the four optional-argument parsing chains.
  RawTeX!(
    r"\def\XKV@testopta#1{%
  \XKV@ifstar{\XKV@sttrue\XKV@t@stopta{#1}}%
    {\XKV@stfalse\XKV@t@stopta{#1}}%
}
\def\XKV@t@stopta#1{\XKV@ifplus{\XKV@pltrue#1}{\XKV@plfalse#1}}
\def\XKV@testoptb#1{\@testopt{\XKV@t@stoptb#1}{KV}}
\def\XKV@t@stoptb#1[#2]#3{%
  \XKV@makepf{#2}%
  \XKV@makehd{#3}%
  \KV@@sp@def\XKV@tfam{#3}%
  #1%
}
\def\XKV@testoptc#1{\@testopt{\XKV@t@stoptc#1}{KV}}
\def\XKV@t@stoptc#1[#2]#3{%
  \XKV@makepf{#2}%
  \XKV@checksanitizeb{#3}\XKV@fams
  \expandafter\XKV@sp@deflist\expandafter
    \XKV@fams\expandafter{\XKV@fams}%
  \@testopt#1{}%
}
\def\XKV@testoptd#1#2{%
  \XKV@testoptb{%
    \edef\XKV@tempa{#2\XKV@header}%
    \def\XKV@tempb{\@testopt{\XKV@t@stoptd#1}}%
    \expandafter\XKV@tempb\expandafter{\XKV@tempa}%
  }%
}
\def\XKV@t@stoptd#1[#2]#3{%
  \@ifnextchar[{\XKV@sttrue#1{#2}{#3}}{\XKV@stfalse#1{#2}{#3}[]}%
}"
  );
  // xkeyval.tex L128-152: pointer detection + key-name splitter.
  RawTeX!(
    r"\def\XKV@ifcmd#1#2#3{%
  \def\XKV@@ifcmd##1#2##2##3\@nil##4{%
    \def##4{##2}\ifx##4\@nnil
      \def##4{##1}\expandafter\@secondoftwo
    \else
      \expandafter\@firstoftwo
    \fi
  }%
  \XKV@@ifcmd#1#2{\@nil}\@nil#3%
}
\def\XKV@getkeyname#1#2{\expandafter\XKV@g@tkeyname#1=\@nil#2}
\long\def\XKV@g@tkeyname#1=#2\@nil#3{%
  \XKV@ifcmd{#1}\savevalue#3{\XKV@rkvtrue\XKV@sgfalse}{%
    \XKV@ifcmd{#1}\gsavevalue#3%
      {\XKV@rkvtrue\XKV@sgtrue}{\XKV@rkvfalse\XKV@sgfalse}%
  }%
}
\def\XKV@getsg#1#2{%
  \expandafter\XKV@ifcmd\expandafter{#1}\global#2\XKV@sgtrue\XKV@sgfalse
}
\def\XKV@define@default#1#2{%
  \expandafter\def\csname\XKV@header#1@default\expandafter
    \endcsname\expandafter{\csname\XKV@header#1\endcsname{#2}}%
}"
  );
  // xkeyval.tex L438-463: `\XKV@setkeys` + preset hooks — LIVE now that the
  // preset store is implemented above (the `\XKV@ifundefined{XKV@…preseth}`
  // branch finds the stored list and applies it, excluding user-passed keys).
  RawTeX!(
    r"\long\def\XKV@setkeys[#1]#2{%
  \XKV@checksanitizea{#2}\XKV@resb
  \let\XKV@naa\@empty
  \XKV@for@o\XKV@resb\XKV@tempa{%
    \expandafter\XKV@g@tkeyname\XKV@tempa=\@nil\XKV@tempa
    \XKV@addtolist@x\XKV@naa\XKV@tempa
  }%
  \ifnum\XKV@depth=\z@\let\XKV@rm\@empty\fi
  \XKV@usepresetkeys{#1}{preseth}%
  \expandafter\XKV@s@tkeys\expandafter{\XKV@resb}{#1}%
  \XKV@usepresetkeys{#1}{presett}%
  \let\CurrentOption\@empty
}
\def\XKV@usepresetkeys#1#2{%
  \XKV@presettrue
  \XKV@for@eo\XKV@fams\XKV@tfam{%
    \XKV@makehd\XKV@tfam
    \XKV@ifundefined{XKV@\XKV@header#2}{}{%
      \XKV@toks\expandafter\expandafter\expandafter
        {\csname XKV@\XKV@header#2\endcsname}%
      \@expandtwoargs\XKV@s@tkeys{\the\XKV@toks}%
        {\XKV@naa\ifx\XKV@naa\@empty\else,\fi#1}%
    }%
  }%
  \XKV@presetfalse
}"
  );
  // SHIM (not verbatim): real `\XKV@s@tkeys#1#2` (xkeyval.tex L464-469) walks
  // the list through `\XKV@s@tk@ys`, but at this point every front-end path
  // has parsed star/plus/prefix/families into XKV state — exactly the inputs
  // of our Rust `\setkeys`. Reconstruct the surface call so key dispatch,
  // `\XKV@rm` collection and na-filtering run through the one Rust path.
  // `\XKV@pf@strip` drops the trailing `@` that `\XKV@makepf` appends; an
  // all-empty prefix maps to the Rust default "KV" (keyval_qname), matching
  // the `{KV}` \@testopt default — bare `[]` empty-prefix headers (`fam@`
  // without prefix) are not distinguished by the Rust path.
  RawTeX!(
    r"\def\XKV@pf@strip#1@#2\@nil{#1}
\long\def\XKV@s@tkeys#1#2{%
  \XKV@toks{#1}%
  \edef\XKV@tempb{\noexpand\lx@xkv@setkeys
    \ifXKV@st*\fi\ifXKV@pl+\fi
    [\expandafter\XKV@pf@strip\XKV@prefix @\@nil]{\XKV@fams}%
    [#2]{\the\XKV@toks}}%
  \XKV@tempb
}"
  );

  //
  // Setup document class info
  //
  xkeyval_setup_document_class();

  //
  // Pointer system — xkeyval.tex L140-146 (\savevalue/\gsavevalue detection),
  // L405-436 (\savekeys family), L518-533 (value store in \XKV@s@tk@ys@),
  // L560-583 (\XKV@replacepointers). Raw packages both drive it and read the
  // store directly: chessboard.sty L1059 `\savekeys[UFCB]{locset}
  // {\global{psset},…}` then L1221-1229 `\boolean{\XKV@UFCB@locset@psset@value}`;
  // xskak.sty L385/L415 same shape. The list bookkeeping is the verbatim TeX;
  // the per-key store/replace lives in the Rust reader (keyvals.rs read_from)
  // and the two closures below, which communicate through XKV@ptr@* state.
  //

  // xkvutils.tex L239-270: comma-list merge/delete used by \savekeys.
  RawTeX!(
    r"\def\XKV@merge#1#2#3{%
  \XKV@checksanitizea{#2}\XKV@tempa
  \XKV@for@o\XKV@tempa\XKV@tempa{%
    \XKV@pltrue
    #3\XKV@tempa\XKV@tempb
    \let\XKV@tempc#1%
    \let#1\@empty
    \XKV@for@o\XKV@tempc\XKV@tempc{%
      #3\XKV@tempc\XKV@tempd
      \ifx\XKV@tempb\XKV@tempd
        \XKV@plfalse
        \XKV@addtolist@o#1\XKV@tempa
      \else
        \XKV@addtolist@o#1\XKV@tempc
      \fi
    }%
    \ifXKV@pl\XKV@addtolist@o#1\XKV@tempa\fi
  }%
  \ifXKV@st\global\let#1#1\fi
}
\def\XKV@delete#1#2#3{%
  \XKV@checksanitizeb{#2}\XKV@tempa
  \let\XKV@tempb#1%
  \let#1\@empty
  \XKV@for@o\XKV@tempb\XKV@tempb{%
    #3\XKV@tempb\XKV@tempc
    \@expandtwoargs\in@{,\XKV@tempc,}{,\XKV@tempa,}%
    \ifin@\else\XKV@addtolist@o#1\XKV@tempb\fi
  }%
  \ifXKV@st\global\let#1#1\fi
}"
  );
  // xkeyval.tex L405-436, verbatim.
  RawTeX!(
    r"\def\savekeys{\XKV@stfalse\XKV@testoptb\XKV@savekeys}
\def\gsavekeys{\XKV@sttrue\XKV@testoptb\XKV@savekeys}
\def\XKV@savekeys#1{%
  \XKV@ifundefined{XKV@\XKV@header save}{%
    \XKV@checksanitizeb{#1}\XKV@tempa
    \ifXKV@st\expandafter\global\fi\expandafter\def\csname XKV@%
      \XKV@header save\expandafter\endcsname\expandafter{\XKV@tempa}%
  }{%
    \expandafter\XKV@merge\csname XKV@\XKV@header
      save\endcsname{#1}\XKV@getsg
  }%
}
\def\delsavekeys{\XKV@stfalse\XKV@testoptb\XKV@delsavekeys}
\def\gdelsavekeys{\XKV@sttrue\XKV@testoptb\XKV@delsavekeys}
\def\XKV@delsavekeys#1{%
  \XKV@ifundefined{XKV@\XKV@header save}{%
    \XKV@err{no save keys defined for `\XKV@header'}%
  }{%
    \expandafter\XKV@delete\csname XKV@\XKV@header
      save\endcsname{#1}\XKV@getsg
  }%
}
\def\unsavekeys{\XKV@stfalse\XKV@testoptb\XKV@unsavekeys}
\def\gunsavekeys{\XKV@sttrue\XKV@testoptb\XKV@unsavekeys}
\def\XKV@unsavekeys{%
  \XKV@ifundefined{XKV@\XKV@header save}{%
    \XKV@err{no save keys defined for `\XKV@header'}%
  }{%
    \ifXKV@st\expandafter\global\fi\expandafter\let
      \csname XKV@\XKV@header save\endcsname\@undefined
  }%
}"
  );

  // `\savevalue{key}` / `\gsavevalue{key}` wrap a KEY inside a \setkeys
  // list. The reader expands the key portion, so the closures fire mid-read:
  // latch the pending-save flags and expand to the bare key. read_from
  // (keyvals.rs) consumes the latch and stores `\XKV@<header><key>@value`.
  DefMacro!("\\savevalue{}", sub[(key_tks)] {
    assign_value("XKV@ptr@rkv", Stored::from(true), None);
    assign_value("XKV@ptr@sg", Stored::from(false), None);
    Ok(key_tks)
  });
  DefMacro!("\\gsavevalue{}", sub[(key_tks)] {
    assign_value("XKV@ptr@rkv", Stored::from(true), None);
    assign_value("XKV@ptr@sg", Stored::from(true), None);
    Ok(key_tks)
  });

  // `\usevalue{key}` inside a VALUE expands to the saved value. Real xkeyval
  // splices it in `\XKV@replacepointers` before the key code runs, with the
  // current header in scope; our reader records the active prefix/keysets as
  // XKV@ptr@prefix/XKV@ptr@keysets, and the closure resolves the store at
  // expansion time (same saved values — earlier keys of the same \setkeys
  // call have already been stored by read_from).
  DefMacro!("\\usevalue{}", sub[(key_tks)] {
    let key = do_expand(key_tks)?.to_string();
    let prefix = match lookup_value("XKV@ptr@prefix") {
      Some(Stored::String(sym)) => with(sym, |s| s.to_string()),
      _ => String::from("KV"),
    };
    let keysets = match lookup_value("XKV@ptr@keysets") {
      Some(Stored::String(sym)) => with(sym, |s| s.to_string()),
      _ => String::new(),
    };
    for ks in keysets.split(',') {
      let cs = T_CS!(s!("\\XKV@{prefix}@{ks}@{key}@value"));
      if lookup_meaning(&cs).is_some() {
        return Ok(Tokens!(cs));
      }
    }
    Error!("undefined", "\\usevalue",
      s!("no value recorded for key `{key}'; ignored"));
    Ok(Tokens!())
  });

  //
  // Presetting keys — verbatim xkeyval.tex L363-403. Every `\setkeys` first
  // applies the family's head presets and then its tail presets for keys the
  // user didn't pass (\XKV@setkeys → \XKV@usepresetkeys, already present
  // verbatim below); cntperchap's `\gdef\@cps@@keymacro@@tracklevel` only
  // ever runs from a preset (cntperchap.sty L75-79), so the former
  // warn-stubs left it undefined ("section level … is unknown"). Perl
  // LaTeXML stubs these too (xkeyval.sty.ltxml L452-475) — beyond-Perl,
  // oracle = xkeyval.tex + pdflatex. All dependencies (\XKV@testoptb,
  // \XKV@ifundefined, \XKV@checksanitizea, \XKV@merge, \XKV@delete,
  // \XKV@getkeyname, \ifXKV@st) are already defined in this binding.
  RawTeX!(
    r"\def\presetkeys{\XKV@stfalse\XKV@testoptb\XKV@presetkeys}
\def\gpresetkeys{\XKV@sttrue\XKV@testoptb\XKV@presetkeys}
\def\XKV@presetkeys#1#2{%
  \XKV@pr@setkeys{#1}{preseth}%
  \XKV@pr@setkeys{#2}{presett}%
}
\def\XKV@pr@setkeys#1#2{%
  \XKV@ifundefined{XKV@\XKV@header#2}{%
    \XKV@checksanitizea{#1}\XKV@tempa
    \ifXKV@st\expandafter\global\fi\expandafter\def\csname
      XKV@\XKV@header#2\expandafter\endcsname\expandafter{\XKV@tempa}%
  }{%
    \expandafter\XKV@merge\csname XKV@\XKV@header
      #2\endcsname{#1}\XKV@getkeyname
  }%
}
\def\delpresetkeys{\XKV@stfalse\XKV@testoptb\XKV@delpresetkeys}
\def\gdelpresetkeys{\XKV@sttrue\XKV@testoptb\XKV@delpresetkeys}
\def\XKV@delpresetkeys#1#2{%
  \XKV@d@lpresetkeys{#1}{preseth}%
  \XKV@d@lpresetkeys{#2}{presett}%
}
\def\XKV@d@lpresetkeys#1#2{%
  \XKV@ifundefined{XKV@\XKV@header#2}{%
    \XKV@err{no presets defined for `\XKV@header'}%
  }{%
    \expandafter\XKV@delete\csname XKV@\XKV@header
      #2\endcsname{#1}\XKV@getkeyname
  }%
}
\def\unpresetkeys{\XKV@stfalse\XKV@testoptb\XKV@unpresetkeys}
\def\gunpresetkeys{\XKV@sttrue\XKV@testoptb\XKV@unpresetkeys}
\def\XKV@unpresetkeys{%
  \XKV@ifundefined{XKV@\XKV@header preseth}{%
    \XKV@err{no presets defined for `\XKV@header'}%
  }{%
    \ifXKV@st\expandafter\global\fi\expandafter\let
      \csname XKV@\XKV@header preseth\endcsname\@undefined
    \ifXKV@st\expandafter\global\fi\expandafter\let
      \csname XKV@\XKV@header presett\endcsname\@undefined
  }%
}"
  );

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

  // \XKV@checkchoice — the choice-key checker style authors call directly
  // (regulatory.sty, glossaries-extra, keyreader). Verbatim from
  // xkeyval.tex L249-321; all its internals (\XKV@afterfi/\XKV@toks/
  // \ifXKV@st/\ifXKV@pl/\XKV@err/\XKV@addtomacro@n) exist above.
  RawTeX!(
    r"\def\XKV@checkchoice[#1]#2#3{%
  \def\XKV@tempa{#1}%
  \ifXKV@st\lowercase{\fi
  \ifx\XKV@tempa\@empty
    \def\XKV@tempa{\XKV@ch@ckch@ice\@nil{#2}{#3}}%
  \else
    \def\XKV@tempa{\XKV@ch@ckchoice#1\@nil{#2}{#3}}%
  \fi
  \ifXKV@st}\fi\XKV@tempa
}
\def\XKV@ch@ckchoice#1#2\@nil#3#4{%
  \def\XKV@tempa{#2}%
  \ifx\XKV@tempa\@empty\XKV@afterelsefi
    \XKV@ch@ckch@ice#1{#3}{#4}%
  \else\XKV@afterfi
    \XKV@@ch@ckchoice#1#2{#3}{#4}%
  \fi
}
\def\XKV@ch@ckch@ice#1#2#3{%
  \def\XKV@tempa{#1}%
  \ifx\XKV@tempa\@nnil\let\XKV@tempa\@empty\else
    \def\XKV@tempa{\def#1{#2}}%
  \fi
  \in@{,#2,}{,#3,}%
  \ifin@
    \ifXKV@pl
      \XKV@addtomacro@n\XKV@tempa\@firstoftwo
    \else
      \XKV@addtomacro@n\XKV@tempa\@firstofone
    \fi
  \else
    \ifXKV@pl
      \XKV@addtomacro@n\XKV@tempa\@secondoftwo
    \else
      \XKV@toks{#2}%
      \XKV@err{value `\the\XKV@toks' is not allowed}%
      \XKV@addtomacro@n\XKV@tempa\@gobble
    \fi
  \fi
  \XKV@tempa
}
\def\XKV@@ch@ckchoice#1#2#3#4{%
  \edef\XKV@tempa{\the\count@}\count@\z@
  \def\XKV@tempb{#3}%
  \def\XKV@tempc##1,{%
    \def#1{##1}%
    \ifx#1\@nnil
      \def#1{#3}\def#2{-1}\count@\XKV@tempa
      \ifXKV@pl
        \let\XKV@tempd\@secondoftwo
      \else
        \XKV@toks{#3}%
        \XKV@err{value `\the\XKV@toks' is not allowed}%
        \let\XKV@tempd\@gobble
      \fi
    \else
      \ifx#1\XKV@tempb
        \edef#2{\the\count@}\count@\XKV@tempa
        \ifXKV@pl
          \let\XKV@tempd\XKV@@ch@ckch@ice
        \else
          \let\XKV@tempd\XKV@@ch@ckch@ic@
        \fi
      \else
        \advance\count@\@ne
        \let\XKV@tempd\XKV@tempc
      \fi
    \fi
    \XKV@tempd
  }%
  \XKV@tempc#4,\@nil,%
}
\def\XKV@@ch@ckch@ice#1\@nil,{\@firstoftwo}
\def\XKV@@ch@ckch@ic@#1\@nil,{\@firstofone}"
  );
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

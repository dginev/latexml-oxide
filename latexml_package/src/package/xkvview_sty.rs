use crate::prelude::*;
use latexml_core::keyval::{self, KeyvalConfig};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl xkvview.sty.ltxml L21-25: load both xkeyval + xkvview raw .sty.
  InputDefinitions!("xkeyval", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  InputDefinitions!("xkvview", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Perl L33-47: shadow \define@key so each invocation also registers
  // the keyset/key metadata in LaTeXML's keyval store. The body then
  // re-invokes the saved \ltx@orig@define@key with all original args so
  // xkeyval's own bookkeeping still runs.
  Let!("\\ltx@orig@define@key", "\\define@key");
  DefMacro!("\\define@key []{}{}[]{}", sub[(prefix, keyset, key, default, code)] {
    let sprefix = prefix.as_ref().map(|p| p.to_string());
    let skeyset = keyset.to_string();
    let skey = key.to_string();
    let sdefault = default.as_ref().map(|d| d.to_string());
    let _ = keyval::define(KeyvalConfig {
      prefix: "KV",
      keyset: &skeyset,
      key: &skey,
      vtype: "",
      default: sdefault.as_deref(),
      macroprefix: sprefix.as_deref(),
      ..KeyvalConfig::default()
    });
    // Re-assemble the invocation of the saved original.
    let mut out: Vec<Token> = vec![T_CS!("\\ltx@orig@define@key")];
    if let Some(p) = prefix {
      out.push(T_OTHER!("["));
      out.extend(p.unlist());
      out.push(T_OTHER!("]"));
    }
    out.push(T_BEGIN!());
    out.extend(keyset.unlist());
    out.push(T_END!());
    out.push(T_BEGIN!());
    out.extend(key.unlist());
    out.push(T_END!());
    if let Some(d) = default {
      out.push(T_OTHER!("["));
      out.extend(d.unlist());
      out.push(T_OTHER!("]"));
    }
    out.push(T_BEGIN!());
    out.extend(code.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });

  // Perl L74-94: \define@cmdkeys [prefix]{keyset}[macroprefix]{keys}[default]
  // — same as cmdkey but #4 is a comma-separated key list, no code arg.
  Let!("\\ltx@orig@define@cmdkeys", "\\define@cmdkeys");
  DefMacro!("\\define@cmdkeys []{}[]{}[]",
    sub[(prefix, keyset, macroprefix, keys, default)] {
      let sprefix = prefix.as_ref().map(|p| p.to_string());
      let skeyset = keyset.to_string();
      let smacroprefix = macroprefix.as_ref().map(|m| m.to_string());
      let sdefault = default.as_ref().map(|d| d.to_string());
      let keys_str = keys.to_string();
      for key in keys_str.split(',') {
        let key = key.trim();
        if !key.is_empty() {
          let _ = keyval::define(KeyvalConfig {
            prefix: "KV",
            keyset: &skeyset,
            key,
            vtype: "",
            default: sdefault.as_deref(),
            kind: Some("command"),
            macroprefix: smacroprefix.as_deref().or(sprefix.as_deref()),
            ..KeyvalConfig::default()
          });
        }
      }
      let mut out: Vec<Token> = vec![T_CS!("\\ltx@orig@define@cmdkeys")];
      if let Some(p) = prefix {
        out.push(T_OTHER!("[")); out.extend(p.unlist()); out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!()); out.extend(keyset.unlist()); out.push(T_END!());
      if let Some(m) = macroprefix {
        out.push(T_OTHER!("[")); out.extend(m.unlist()); out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!()); out.extend(keys.unlist()); out.push(T_END!());
      if let Some(d) = default {
        out.push(T_OTHER!("[")); out.extend(d.unlist()); out.push(T_OTHER!("]"));
      }
      Ok(Tokens::new(out))
    }
  );

  // Perl L100-124: \define@choicekey ...
  Let!("\\ltx@orig@define@choicekey", "\\define@choicekey");
  DefMacro!(
    "\\define@choicekey OptionalMatch:* OptionalMatch:+ []{}{}[]{}[]{}",
    sub[(star, plus, prefix, keyset, key, bin, choices, default, code)] {
      let sprefix = prefix.as_ref().map(|p| p.to_string());
      let skeyset = keyset.to_string();
      let skey = key.to_string();
      let sdefault = default.as_ref().map(|d| d.to_string());
      // KeyvalConfig.choices requires Vec<&'static str>. Leak the
      // comma-split entries — key definitions persist for the program
      // lifetime (matches xkeyval_sty.rs:222-224 pattern).
      let choices_str = choices.to_string();
      let choices_vec: Vec<&'static str> = choices_str
        .split(',')
        .map(|s| &*Box::leak(s.trim().to_string().into_boxed_str()))
        .collect();
      let bin_tokens = bin.clone();
      let _ = keyval::define(KeyvalConfig {
        prefix: "KV",
        keyset: &skeyset,
        key: &skey,
        vtype: "",
        default: sdefault.as_deref(),
        kind: Some("choice"),
        normalize: Some(star.is_some()),
        choices: choices_vec,
        bin: bin_tokens,
        macroprefix: sprefix.as_deref(),
        ..KeyvalConfig::default()
      });
      let mut out: Vec<Token> = vec![T_CS!("\\ltx@orig@define@choicekey")];
      if let Some(s) = star { out.extend(s.unlist()); }
      if let Some(p) = plus { out.extend(p.unlist()); }
      if let Some(p) = prefix {
        out.push(T_OTHER!("[")); out.extend(p.unlist()); out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!()); out.extend(keyset.unlist()); out.push(T_END!());
      out.push(T_BEGIN!()); out.extend(key.unlist()); out.push(T_END!());
      if let Some(b) = bin {
        out.push(T_OTHER!("[")); out.extend(b.unlist()); out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!()); out.extend(choices.unlist()); out.push(T_END!());
      if let Some(d) = default {
        out.push(T_OTHER!("[")); out.extend(d.unlist()); out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!()); out.extend(code.unlist()); out.push(T_END!());
      Ok(Tokens::new(out))
    }
  );

  // Perl L130-149: \define@boolkey OptionalMatch:+ []{}[]{}[]{}
  Let!("\\ltx@orig@define@boolkey", "\\define@boolkey");
  DefMacro!("\\define@boolkey OptionalMatch:+ []{}[]{}[]{}",
    sub[(plus, prefix, keyset, macroprefix, key, default, code)] {
      let sprefix = prefix.as_ref().map(|p| p.to_string());
      let skeyset = keyset.to_string();
      let skey = key.to_string();
      let smacroprefix = macroprefix.as_ref().map(|m| m.to_string());
      let sdefault = default.as_ref().map(|d| d.to_string());
      let _ = keyval::define(KeyvalConfig {
        prefix: "KV",
        keyset: &skeyset,
        key: &skey,
        vtype: "",
        default: sdefault.as_deref(),
        kind: Some("boolean"),
        macroprefix: smacroprefix.as_deref().or(sprefix.as_deref()),
        ..KeyvalConfig::default()
      });
      let mut out: Vec<Token> = vec![T_CS!("\\ltx@orig@define@boolkey")];
      if let Some(p) = plus { out.extend(p.unlist()); }
      if let Some(p) = prefix {
        out.push(T_OTHER!("[")); out.extend(p.unlist()); out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!()); out.extend(keyset.unlist()); out.push(T_END!());
      if let Some(m) = macroprefix {
        out.push(T_OTHER!("[")); out.extend(m.unlist()); out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!()); out.extend(key.unlist()); out.push(T_END!());
      if let Some(d) = default {
        out.push(T_OTHER!("[")); out.extend(d.unlist()); out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!()); out.extend(code.unlist()); out.push(T_END!());
      Ok(Tokens::new(out))
    }
  );

  // Perl L151-171: \define@boolkeys []{}[]{}[]
  Let!("\\ltx@orig@define@boolkeys", "\\define@boolkeys");
  DefMacro!("\\define@boolkeys []{}[]{}[]",
    sub[(prefix, keyset, macroprefix, keys, default)] {
      let sprefix = prefix.as_ref().map(|p| p.to_string());
      let skeyset = keyset.to_string();
      let smacroprefix = macroprefix.as_ref().map(|m| m.to_string());
      let sdefault = default.as_ref().map(|d| d.to_string());
      let keys_str = keys.to_string();
      for key in keys_str.split(',') {
        let key = key.trim();
        if !key.is_empty() {
          let _ = keyval::define(KeyvalConfig {
            prefix: "KV",
            keyset: &skeyset,
            key,
            vtype: "",
            default: sdefault.as_deref(),
            kind: Some("boolean"),
            macroprefix: smacroprefix.as_deref().or(sprefix.as_deref()),
            ..KeyvalConfig::default()
          });
        }
      }
      let mut out: Vec<Token> = vec![T_CS!("\\ltx@orig@define@boolkeys")];
      if let Some(p) = prefix {
        out.push(T_OTHER!("[")); out.extend(p.unlist()); out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!()); out.extend(keyset.unlist()); out.push(T_END!());
      if let Some(m) = macroprefix {
        out.push(T_OTHER!("[")); out.extend(m.unlist()); out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!()); out.extend(keys.unlist()); out.push(T_END!());
      if let Some(d) = default {
        out.push(T_OTHER!("[")); out.extend(d.unlist()); out.push(T_OTHER!("]"));
      }
      Ok(Tokens::new(out))
    }
  );

  // Perl L53-72: command keys — `kind => "command"` + macroprefix.
  Let!("\\ltx@orig@define@cmdkey", "\\define@cmdkey");
  DefMacro!("\\define@cmdkey []{}[]{}[]{}",
    sub[(prefix, keyset, macroprefix, key, default, code)] {
      let sprefix = prefix.as_ref().map(|p| p.to_string());
      let skeyset = keyset.to_string();
      let skey = key.to_string();
      let smacroprefix = macroprefix.as_ref().map(|m| m.to_string());
      let sdefault = default.as_ref().map(|d| d.to_string());
      let _ = keyval::define(KeyvalConfig {
        prefix: "KV",
        keyset: &skeyset,
        key: &skey,
        vtype: "",
        default: sdefault.as_deref(),
        kind: Some("command"),
        macroprefix: smacroprefix.as_deref().or(sprefix.as_deref()),
        ..KeyvalConfig::default()
      });
      let mut out: Vec<Token> = vec![T_CS!("\\ltx@orig@define@cmdkey")];
      if let Some(p) = prefix {
        out.push(T_OTHER!("["));
        out.extend(p.unlist());
        out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!());
      out.extend(keyset.unlist());
      out.push(T_END!());
      if let Some(m) = macroprefix {
        out.push(T_OTHER!("["));
        out.extend(m.unlist());
        out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!());
      out.extend(key.unlist());
      out.push(T_END!());
      if let Some(d) = default {
        out.push(T_OTHER!("["));
        out.extend(d.unlist());
        out.push(T_OTHER!("]"));
      }
      out.push(T_BEGIN!());
      out.extend(code.unlist());
      out.push(T_END!());
      Ok(Tokens::new(out))
    }
  );
});

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

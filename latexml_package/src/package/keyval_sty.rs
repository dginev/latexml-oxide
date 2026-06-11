use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: keyval.sty.ltxml
  InputDefinitions!("keyval", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // HOOK into define@key to make the latexml definitions as well
  // \define@key{keyset}{key}[default]{code}
  DefPrimitive!("\\define@key{}{}[]{}", sub[(keyset_tks, key_tks, default_opt, code)] {
    let keyset = do_expand(keyset_tks)?.to_string();
    let key = do_expand(key_tks)?.to_string();
    let default = default_opt.map(|d: Tokens| d.to_string());

    keyval::define(KeyvalConfig {
      prefix: "KV",
      keyset: &keyset,
      key: &key,
      vtype: "",
      default: default.as_deref(),
      code: Some(ExpansionBody::Tokens(code)),
      ..KeyvalConfig::default()
    })?;
  });

  // \setkeys{keyset}{keyvals}
  DefMacro!("\\setkeys{}", sub[(keyset_tks)] {
    let keyset = do_expand(keyset_tks)?.to_string();
    let keysets: Vec<String> = keyset.split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();

    let mut keyvals = KeyVals::new(KeyvalsConfig {
      prefix: None,
      keysets,
      set_all: false,
      set_internals: true,
      skip: Vec::new(),
      skip_missing: keyvals::SkipMissing::None,
      hook_missing: None,
    });
    keyvals.read_from(T_END!(), false)?;
    Ok(keyvals.set_keys_expansion())
  });
});

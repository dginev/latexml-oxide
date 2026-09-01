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
    let default_tks_cfg: Option<Tokens> = default_opt;
    let default = default_tks_cfg.clone().map(|d| d.to_string());

    keyval::define(KeyvalConfig {
      prefix: "KV",
      keyset: &keyset,
      key: &key,
      vtype: "",
      default: default.as_deref(),
      default_tks: default_tks_cfg.clone(),
      code: Some(ExpansionBody::Tokens(code)),
      ..KeyvalConfig::default()
    })?;
  });

  // \setkeys{keyset}{keyvals} and its starred form \setkeys*{keyset}{keyvals}.
  //
  // keyval's `\setkeys*` (and xkeyval's) silently *ignores* keys not defined in
  // the keyset rather than raising "undefined key" — the two forms differ only
  // in how a missing key is handled (`SkipMissing::None` = warn/error vs
  // `SkipMissing::All` = silently drop). The keyvals themselves are read
  // imperatively from the stream (not a macro argument), and the required
  // keyset group is a `{}` argument grabbed *before* the sub body runs, so a
  // leading `*` cannot be peeked here — dispatch on it in TeX with `\@ifstar`,
  // routing to the warn/ignore variant, each of which then grabs `{keyset}` and
  // reads the following `{keyvals}` group.
  fn setkeys_body(keyset_tks: Tokens, skip_missing: keyvals::SkipMissing) -> Result<Tokens> {
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
      skip_missing,
      hook_missing: None,
    });
    keyvals.read_from(T_END!(), false)?;
    Ok(keyvals.set_keys_expansion())
  }

  DefMacro!("\\ltx@setkeys@warn{}", sub[(keyset_tks)] {
    setkeys_body(keyset_tks, keyvals::SkipMissing::None)
  });
  DefMacro!("\\ltx@setkeys@ignore{}", sub[(keyset_tks)] {
    setkeys_body(keyset_tks, keyvals::SkipMissing::All)
  });
  RawTeX!(r"\def\setkeys{\@ifstar\ltx@setkeys@ignore\ltx@setkeys@warn}");
});

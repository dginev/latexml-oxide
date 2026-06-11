use latexml_package::prelude::*;

// Focused regression for `ProcessOptions(keysets => ['...'])` plumbing.
// Mirrors the Perl `Package.pm::executeOption_internal` keyset branch and
// the Rust `keyval_option_qname` helper added alongside the ar5iv
// `tokenlimit` fix (2026-05-22). Counterpart test pair lives at
// `latexml_oxide/tests/keyval_options/keysetopt[ab].{tex,xml}`.
LoadDefinitions!({
  DefKeyVal!("KSO", "value", "");

  ProcessOptions!(keysets => ["KSO"]);

  let captured = lookup_value("KV@KSO@value")
    .map(|v| v.to_string())
    .unwrap_or_else(|| "unset".to_string());
  def_macro(
    T_CS!("\\keysetoptvalue"),
    None,
    Tokens!(ExplodeText!(&captured)),
    None,
  )?;
});

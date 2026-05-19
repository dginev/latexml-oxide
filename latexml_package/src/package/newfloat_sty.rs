use crate::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  // Perl: DefPrimitive('\SetupFloatingEnvironment OptionalKeyVals {}', sub { ... })
  DefPrimitive!("\\SetupFloatingEnvironment OptionalKeyVals {}", sub[(options, ftype)] {
    let ftype = ftype.to_string();
    let within = options.as_ref()
      .and_then(|o| o.get_value("within"))
      .map(|v| v.to_string())
      .unwrap_or_default();
    new_counter(&ftype, &within, None)?;
    let inlist = options.as_ref()
      .and_then(|o| o.get_value("fileext"))
      .map(|v| v.to_string())
      .unwrap_or_else(|| s!("lo{ftype}"));
    let ext_cs = s!("\\ext@{ftype}");
    def_macro(T_CS!(ext_cs), None, Tokens::new(ExplodeText!(inlist)), None)?;
    let name = options.as_ref()
      .and_then(|o| o.get_value("listname"))
      .map(|v| v.to_string())
      .unwrap_or_else(|| s!("List of {ftype}s"));
    let name_cs = s!("\\{ftype}name");
    def_macro(T_CS!(name_cs), None, Tokens::new(ExplodeText!(name)), None)?;
  });

  // Perl: DefPrimitive('\DeclareFloatingEnvironment OptionalKeyVals {}', sub { ... })
  //
  // Perl L64-80 creates both the `$type` and `$type*` envs with
  // `beforeDigest => sub { beforeFloat($type [, double => 1]) }`. Rust
  // delegates to `float_sty::define_float_environment` which calls
  // `create_float_env` twice (once for `$type`, once for `$type*`, the
  // latter with `is_double=true`). Both envs get a `before_float_ex`
  // before_digest closure inside the helper (float_sty.rs:174-180).
  // Audit breadcrumb: count-diff shows 2 Perl beforeDigest vs 0 here,
  // but the hooks live in the shared helper — not a gap.
  DefPrimitive!("\\DeclareFloatingEnvironment OptionalKeyVals {}", sub[(options, ftype)] {
    let ftype = ftype.to_string();
    let within = options.as_ref()
      .and_then(|o| o.get_value("within"))
      .map(|v| v.to_string())
      .unwrap_or_default();
    let inlist = options.as_ref()
      .and_then(|o| o.get_value("fileext"))
      .map(|v| v.to_string())
      .unwrap_or_else(|| s!("lo{ftype}"));
    let name = options.as_ref()
      .and_then(|o| o.get_value("listname"))
      .map(|v| v.to_string())
      .unwrap_or_else(|| s!("List of {ftype}s"));

    // Use shared float environment helper from float.sty
    crate::package::float_sty::define_float_environment(&ftype, &inlist, &within)?;

    // Perl also defines the name macro
    let name_cs = s!("\\{ftype}name");
    def_macro(T_CS!(name_cs), None, Tokens::new(ExplodeText!(name)), None)?;

    // Perl: fnum@font@ and format@title@font@ default to float versions
    let fnum_font_cs = s!("\\fnum@font@{ftype}");
    def_macro(
      T_CS!(fnum_font_cs), None,
      mouth::tokenize_internal("\\fnum@font@float"), None,
    )?;
    let ftf_cs = s!("\\format@title@font@{ftype}");
    def_macro(
      T_CS!(ftf_cs), None,
      mouth::tokenize_internal("\\format@title@font@float"), None,
    )?;
  });

  def_macro_noop("\\ForEachFloatingEnvironment{}")?;
  def_macro_noop("\\PrepareListOf{}{}")?;
});

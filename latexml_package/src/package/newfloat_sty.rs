use crate::prelude::*;

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
  // newfloat.sty:59-125: `\DeclareFloatingEnvironment[opts]{type}[singular]
  // [listname]` — the two TRAILING optionals (`\newfloat@DFE@setname`,
  // `\newfloat@DFE@setlistname`, :117-125) set `\<type>name` and
  // `\list<type>name`; without them `[Listagem][Lista de listagens]`
  // (pygmentex.sty:23 via `\DeclareCaptionType`) leaked into the text as a
  // paragraph in both engines. The primitive keeps the declaration; the
  // macro layer reads the tail.
  RawTeX!(
    r"\def\DeclareFloatingEnvironment{\@ifnextchar[\lx@newfloat@DFE{\lx@newfloat@DFE[]}}
\def\lx@newfloat@DFE[#1]#2{\lx@newfloat@declare[#1]{#2}\@ifnextchar[\newfloat@DFE@setname\relax}
\def\newfloat@DFE@setname[#1]{\expandafter\def\csname\lx@newfloat@current name\endcsname{#1}%
  \@ifnextchar[\newfloat@DFE@setlistname\relax}
\def\newfloat@DFE@setlistname[#1]{\expandafter\def\csname list\lx@newfloat@current name\endcsname{#1}}"
  );
  DefPrimitive!("\\lx@newfloat@declare OptionalKeyVals {}", sub[(options, ftype)] {
    let ftype = ftype.to_string();
    def_macro(T_CS!("\\lx@newfloat@current"), None, Tokens::new(ExplodeText!(ftype.clone())), None)?;
    let within = options.as_ref()
      .and_then(|o| o.get_value("within"))
      .map(|v| v.to_string())
      .unwrap_or_default();
    let inlist = options.as_ref()
      .and_then(|o| o.get_value("fileext"))
      .map(|v| v.to_string())
      .unwrap_or_else(|| s!("lo{ftype}"));
    // newfloat.sty:87-111: `name=` → `\<type>name` (default: the type,
    // capitalized), `listname=` → `\list<type>name` (default "List of <Type>s").
    let mut type_name: String = ftype.chars().take(1).flat_map(char::to_uppercase)
      .chain(ftype.chars().skip(1)).collect();
    if let Some(v) = options.as_ref().and_then(|o| o.get_value("name")) {
      type_name = v.to_string();
    }
    let list_name = options.as_ref()
      .and_then(|o| o.get_value("listname"))
      .map(|v| v.to_string())
      .unwrap_or_else(|| s!("List of {type_name}s"));

    // Use shared float environment helper from float.sty
    float_sty::define_float_environment(&ftype, &inlist, &within)?;

    // Perl also defines the name macro
    let name_cs = s!("\\{ftype}name");
    def_macro(T_CS!(name_cs), None, Tokens::new(ExplodeText!(type_name)), None)?;
    let list_cs = s!("\\list{ftype}name");
    def_macro(T_CS!(list_cs), None, Tokens::new(ExplodeText!(list_name)), None)?;

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

use crate::prelude::*;

LoadDefinitions!({
  // newfloat.sty:304-306 `\SetupFloatingEnvironment{type}{options}` sets an existing type's options
  // (`\newfloat@setoptions`, :149-167, keys :173-271): `name` (`\newfloat@setname`, which babel's `\captions<language>`
  // keeps, :179-188), `listname`, `fileext`, `autorefname`, `legendname`, `placement`, `within`/`without` (the counter
  // reset and `\the<type>`, :210-245) and `chapterlistsgaps` (list spacing, nothing here). Perl's binding has
  // `\SetupFloatingEnvironment OptionalKeyVals {}`, a second `\DeclareFloatingEnvironment` that read the type from the
  // options' place, so `\SetupFloatingEnvironment{listing}{name=Prog}` printed "name=Prog" and the name was unchanged
  // (KNOWN_PERL_ERRORS #1138; 2306.12224, 2405.15425, 2411.16612 "List.", 2507.09840, 2510.09447 "Listagem"; review
  // rev78s2 probe sf). newfloat.sty:48 loads keyval.
  RequirePackage!("keyval");
  RawTeX!(
    r"\def\SetupFloatingEnvironment#1#2{\def\newfloat@type{#1}%
\let\newfloat@within@value\@undefined\setkeys{@newfloat}{#2}%
\ifx\newfloat@within@value\@undefined\else
\edef\@tempa{\noexpand\lx@newfloat@setwithin{#1}{\newfloat@within@value}}\@tempa\fi}
\def\newfloat@setname#1#2{\newfloat@@setname{#1}{#2}\begingroup
\ifcsname languagename\endcsname\ifcsname captions\languagename\endcsname
\expandafter\g@addto@macro\csname captions\languagename\endcsname{\newfloat@@setname{#1}{#2}}\fi\fi
\endgroup}
\def\newfloat@@setname#1#2{\@namedef{#1name}{#2}}
\define@key{@newfloat}{fileext}{\@namedef{ext@\newfloat@type}{#1}}
\define@key{@newfloat}{listname}{\@namedef{list\newfloat@type name}{#1}}
\define@key{@newfloat}{name}{\expandafter\newfloat@setname\expandafter{\newfloat@type}{#1}}
\define@key{@newfloat}{autorefname}{\@namedef{\newfloat@type autorefname}{#1}}
\define@key{@newfloat}{legendname}{\@namedef{fleg\newfloat@type}{#1}}
\define@key{@newfloat}{placement}{\@namedef{fps@\newfloat@type}{#1}}
\define@key{@newfloat}{within}{\def\newfloat@within@value{#1}}
\define@key{@newfloat}{without}[]{\def\newfloat@within@value{none}}
\define@key{@newfloat}{chapterlistsgaps}{}
\def\lx@newfloat@setwithin#1#2{\ifcsname c@chapter\endcsname\@removefromreset{#1}{chapter}\fi
\@removefromreset{#1}{section}\edef\@tempa{#2}\ifx\@tempa\@empty\def\@tempa{none}\fi
\def\@tempb{none}\ifx\@tempa\@tempb
  \lx@newfloat@@setwithin{#1}{}%
\else\def\@tempb{chapter}\ifx\@tempa\@tempb
  \@addtoreset{#1}{chapter}\lx@newfloat@@setwithin{#1}{\ifnum\c@chapter>\z@ \thechapter.\fi}%
\else\def\@tempb{section}\ifx\@tempa\@tempb
  \@addtoreset{#1}{section}%
  \ifcsname c@chapter\endcsname\@addtoreset{#1}{chapter}\lx@newfloat@@setwithin{#1}{\thesection.}%
  \else\lx@newfloat@@setwithin{#1}{\ifnum\c@section>\z@ \thesection.\fi}\fi
\else\PackageError{newfloat}{Invalid value `#2' for option `within'}\@ehc
\fi\fi\fi}
\def\lx@newfloat@@setwithin#1#2{\expandafter\gdef\csname the#1\endcsname{#2\arabic{#1}}}"
  );
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
    // newfloat.sty:59-125 makes the type a kernel float (`\@float`), whose `\caption` is the class's `\@makecaption`
    // (`\lx@makecaption`, latex_constructs_rust_only.rs): the kernel's "Listing 1: text" (latex.ltx `\sbox\@tempboxa{#1:
    // #2}`), amsart's "Listing 1. text", IEEEtran's figure format "Listing 1. text" (the minted
    // `newfloat` route). Perl's newfloat.sty.ltxml defines none, so the tag closed with a space, "Listing 1 text",
    // beside "Table 1: text" in the same paper (2411.16612; KNOWN_PERL_ERRORS #1124). Guards
    // `perfect_kernel_batch78s::newfloat_caption_tag_closes_with_a_colon`,
    // `perfect_kernel_batch78y::{ieeetran_*, amsart_newfloat_caption_takes_the_class_format}`.
    let format_cs = T_CS!(s!("\\format@title@{ftype}"));
    let format_params = parse_parameters("{}", &format_cs, true)?;
    def_macro(
      format_cs, format_params,
      mouth::tokenize_internal(TeXString::assembled(s!("\\lx@makecaption{{{ftype}}}{{#1}}"))), None,
    )?;
  });

  def_macro_noop("\\ForEachFloatingEnvironment{}")?;
  def_macro_noop("\\PrepareListOf{}{}")?;
});

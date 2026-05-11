use crate::prelude::*;
use latexml_core::common::color::{self, Color, color_from_model_spec};

/// Parse a color from an optional model name and a spec string.
/// If model is given, constructs the Color directly.
/// If no model, looks up the named color from state.
/// Perl: ParseColor($model, $spec) in color.sty.ltxml
pub fn parse_color(model: Option<&str>, spec: &str) -> Color {
  let spec = spec.trim().trim_matches(|c| c == '{' || c == '}').trim();
  if let Some(model) = model {
    let model_lc = model.to_lowercase();
    if model_lc == "named" {
      return lookup_color_obj(&format!("named_{spec}"));
    }
    color_from_model_spec(&model_lc, spec)
  } else {
    lookup_color_obj(spec)
  }
}

/// Look up a named color from state, returning a Color object.
/// Perl: LookupColor($name) in Package.pm
pub fn lookup_color_obj(name: &str) -> Color {
  // Empty or whitespace-only name: silently return BLACK (expression-decode
  // paths invoke us with `""` or `" "` when parsing malformed input; no
  // error needed here because the decoder already surfaces its own).
  if name.trim().is_empty() {
    return color::BLACK;
  }
  let key = s!("color_{name}");
  match state::lookup_value(&key) {
    Some(Stored::String(sym)) => {
      let stored_str = arena::with(sym, |s| s.to_string());
      Color::from_stored(&stored_str).unwrap_or_else(|| {
        Info!("undefined", name, &s!("color '{}' is undefined...", name));
        color::BLACK
      })
    },
    _ => {
      // Perl color.sty.ltxml L50-53:
      //   AssignValue('color_'.$spec => Black);
      //   Error('unexpected', $spec, $STATE->getStomach,
      //     "Can't find color named '$spec'; assuming Black");
      // Persist Black under this name so subsequent lookups resolve it
      // without repeating the diagnostic, then surface an Error-status
      // diagnostic. We inline the bookkeeping from the `Error!` macro
      // because that macro is return-based (`Fatal!` at threshold) and
      // `lookup_color_obj` returns `Color`, not `Result<Color>`.
      assign_value(
        &s!("color_{name}"),
        Stored::String(arena::pin(color::BLACK.to_stored())),
        None,
      );
      latexml_core::common::error::note_status(latexml_core::common::error::LogStatus::Error, None);
      if !latexml_core::common::error::is_log_output_suppressed() {
        log::error!(
          target: &format!("unexpected:{}", name),
          "Can't find color named '{}'; assuming Black",
          name
        );
      }
      color::BLACK
    },
  }
}

/// Look up a named color from state. Returns hex string.
/// Perl: LookupColor($name) in Package.pm
pub fn lookup_color(name: &str) -> String { lookup_color_obj(name).to_attribute() }

LoadDefinitions!({
  //======================================================================
  // Ignorable options (mostly drivers)
  for option in &[
    "monochrome",
    "debugshow",
    "dvipdf",
    "dvipdfm",
    "dvipdfmx",
    "pdftex",
    "xetex",
    "dvipsone",
    "dviwindo",
    "emtex",
    "dviwin",
    "textures",
    "pctexps",
    "pctexwin",
    "pctexhp",
    "pctex32",
    "truetex",
    "tcidvi",
    "vtex",
    "nodvipsnames",
  ] {
    DeclareOption!(option, None);
  }
  // `usenames` makes `\DefineNamedColor` ALSO expose the color under
  // the plain name (no `named_` prefix), so `\color{Blue}` after
  // `\usepackage[usenames,dvipsnames]{color}` resolves to the
  // dvipsnam.def-loaded `Blue`. LaTeX color.sty L84-87 implements this
  // by redefining `\c@lor@usename` to actually register the lookup; we
  // emulate by setting a flag the `\DefineNamedColor` primitive
  // honors. Mirrors Perl `color.sty.ltxml` (which silently relied on
  // the listings.sty option-deferral path; this is the more direct
  // fix). Driver: 1205.2217 (`\lstset{keywordstyle=...\color{Blue}}`
  // 16 × `unexpected:Blue` → 0 errors).
  DeclareOption!("usenames", {
    state::assign_value("color_usenames_active", true, Some(Scope::Global));
  });
  // Options that want the dvipsnam definitions
  for option in &["dvips", "xdvi", "oztex", "dvipsnames"] {
    DeclareOption!(*option, {
      InputDefinitions!("dvipsnam", extension => Some(Cow::Borrowed("def")));
    });
  }

  //======================================================================
  // \definecolor{name}{model}{spec}
  // Perl L59-63:
  //   ($name, $model, $spec) = map { $_ && Expand($_) } $name, $model, $spec;
  //   DefColor(ToString($name), ParseColor($model, $spec));
  //   Box(undef, undef, undef,
  //       Invocation(T_CS('\definecolor'), $name, $model, $spec));
  DefPrimitive!("\\definecolor{}{}{}", sub[(name, model, spec)] {
    let name_expanded = do_expand(name)?;
    let model_expanded = do_expand(model)?;
    let spec_expanded = do_expand(spec)?;
    let name_str = name_expanded.to_string();
    let model_str = model_expanded.to_string();
    let spec_str = spec_expanded.to_string();
    // Use parse_color to handle all models including "named" lookups
    let color = parse_color(Some(&model_str), &spec_str);
    def_color(&name_str, &color, None)?;
    // Perl L63: Box with reversion Invocation so \definecolor round-trips
    // into tex= attributes.
    let reversion_tokens = Invocation!("\\definecolor",
      vec![Some(name_expanded), Some(model_expanded), Some(spec_expanded)]);
    Ok(vec![Digested::from(Tbox::new(pin!(""), None, None,
      reversion_tokens, arena::SymHashMap::default()))])
  });

  // \DefineNamedColor{dmodel}{name}{model}{spec}
  // Perl L69-73:
  //   ($dmodel, $name, $model, $spec) = map { $_ && Expand($_) } ...;
  //   DefColor('named_'.ToString($name), ParseColor($model, $spec));
  //   Box(undef, undef, undef,
  //       Invocation(T_CS('\DefineNamedColor'), $dmodel, $name, $model, $spec));
  DefPrimitive!("\\DefineNamedColor{}{}{}{}", sub[(dmodel, name, model, spec)] {
    let dmodel_expanded = do_expand(dmodel)?;
    let name_expanded = do_expand(name)?;
    let model_expanded = do_expand(model)?;
    let spec_expanded = do_expand(spec)?;
    let name_str = name_expanded.to_string();
    let model_str = model_expanded.to_string();
    let spec_str = spec_expanded.to_string();
    let color = parse_color(Some(&model_str), &spec_str);
    let named_key = format!("named_{}", name_str);
    def_color(&named_key, &color, None)?;
    // Perl color.sty L84-87 + L133 `\c@lor@usename`: with the
    // `usenames` package option, ALSO expose the plain name so
    // `\color{Blue}` works after `\DefineNamedColor{named}{Blue}...`.
    // Driver: 1205.2217 dvipsnam.def's 68 named colors used directly.
    if state::lookup_bool("color_usenames_active") {
      def_color(&name_str, &color, None)?;
    }
    // Perl L73: Box with reversion Invocation preserving all four args.
    let reversion_tokens = Invocation!("\\DefineNamedColor",
      vec![Some(dmodel_expanded), Some(name_expanded),
           Some(model_expanded), Some(spec_expanded)]);
    Ok(vec![Digested::from(Tbox::new(pin!(""), None, None,
      reversion_tokens, arena::SymHashMap::default()))])
  });

  // \color[model]{spec} or \color{name}
  // Perl: returns Box(undef,undef,undef, Invocation(\color, T_OTHER('rgb'), T_OTHER(components)))
  DefPrimitive!("\\color[]{}", sub[(model_opt, spec)] {
    let model_str = model_opt.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let spec_str = do_expand(spec)?.to_string();
    let color = parse_color(model_str.as_deref(), &spec_str);

    // If in preamble, store for \normalcolor
    if state::lookup_bool_sym(pin!("inPreamble")) {
      assign_value("preambleTextcolor", Stored::String(arena::pin(color.to_stored())), None);
    }
    merge_font(fontmap!(color => color));

    // Perl: Box(undef,undef,undef, Invocation(\color, T_OTHER('rgb'), T_OTHER(comps)))
    // Return an empty Tbox whose reversion produces \color[rgb]{r,g,b} for the tex attribute.
    let rgb = color.to_rgb();
    let comps = rgb.components().iter()
      .map(|c| {
        let v = (*c * 10000.0).round() / 10000.0;
        if v == v.floor() { format!("{}", v as i64) } else { format!("{v}") }
      })
      .collect::<Vec<_>>().join(",");
    let reversion_tokens = Invocation!("\\color",
      vec![Some(Tokens::from(T_OTHER!("rgb"))),
           Some(Tokens::from(T_OTHER!(&*comps)))]);
    Ok(vec![Digested::from(Tbox::new(pin!(""), None, None,
      reversion_tokens, arena::SymHashMap::default()))])
  });

  // \pagecolor[model]{spec}
  // Perl: returns Box(undef,undef,undef, Invocation(\pagecolor, $model, $spec))
  DefPrimitive!("\\pagecolor[]{}", sub[(model_opt, spec)] {
    let model_str = model_opt.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let spec_str = do_expand(spec)?.to_string();
    let color = parse_color(model_str.as_deref(), &spec_str);
    merge_font(fontmap!(bg => color));

    // Perl returns Box(undef,undef,undef, Invocation(\pagecolor, $model, $spec))
    let reversion_tokens = Invocation!("\\pagecolor",
      vec![model_str.as_deref().map(|s| Tokens::from(T_OTHER!(s))),
           Some(Tokens::from(T_OTHER!(&*spec_str)))]);
    Ok(vec![Digested::from(Tbox::new(pin!(""), None, None,
      reversion_tokens, arena::SymHashMap::default()))])
  });

  // \normalcolor — restores color from preamble
  DefPrimitive!("\\normalcolor", {
    let color = match state::lookup_value("preambleTextcolor") {
      Some(Stored::String(sym)) => {
        let stored_str = arena::with(sym, |s| s.to_string());
        Color::from_stored(&stored_str).unwrap_or(color::BLACK)
      },
      _ => color::BLACK,
    };
    merge_font(fontmap!(color => color));
  });

  // \textcolor[model]{spec}{text}
  DefMacro!(
    "\\textcolor[]{}{}",
    "{\\ifx.#1.\\color{#2}\\else\\color[#1]{#2}\\fi#3}"
  );

  // \colorbox[model]{spec}{text}
  DefMacro!(
    "\\colorbox[]{}{}",
    "\\hbox{\\ifx.#1.\\pagecolor{#2}\\else\\pagecolor[#1]{#2}\\fi#3}"
  );

  // \fcolorbox[model]{framespec}{bgspec}{text}
  DefConstructor!("\\fcolorbox[]{}{} Undigested",
    "<ltx:text framed='rectangle' framecolor='#framecolor' _noautoclose='1'>#text</ltx:text>",
    mode => "internal_vertical",
    after_digest => sub[whatsit] {
      let model_str = whatsit.get_arg(1).map(|m| m.to_string());
      let fspec_str = whatsit.get_arg(2).map(|f| f.to_string()).unwrap_or_default();
      let bspec_str = whatsit.get_arg(3).map(|b| b.to_string()).unwrap_or_default();
      let text_tokens = whatsit.get_arg(4).map(|t| t.revert()).transpose()?;

      let framecolor = parse_color(model_str.as_deref(), &fspec_str);
      whatsit.set_property("framecolor", Stored::String(arena::pin(framecolor.to_attribute())));

      let bgcolor = parse_color(model_str.as_deref(), &bspec_str);
      merge_font(fontmap!(bg => bgcolor));

      if let Some(tokens) = text_tokens {
        let digested = digest(tokens)?;
        whatsit.set_property("text", Stored::Digested(digested));
      }
    }
  );

  // NOTE: Perl color.sty.ltxml does NOT define \ifglobalcolors — only
  // xcolor.sty does (Perl xcolor.sty.ltxml L29). `def_color` in
  // latexml_core::binding::content guards its ifglobalcolors check with
  // `lookup_definition(\ifglobalcolors).is_some()`, so leaving the CS
  // undefined here is the faithful behavior; the global-scope branch
  // short-circuits naturally.

  //========================
  // Low-level stuff; redefined from LaTeX stubs (Perl color.sty.ltxml L122-132)
  // Note: Perl deliberately does not define \current@color / \default@color / \reset@color
  // (see Perl comment "Not sure what \current@color should return").
  DefMacro!("\\set@color", None);
  DefMacro!("\\color@begingroup", "\\begingroup");
  DefMacro!("\\color@endgroup", "\\endgroup");
  DefMacro!("\\color@setgroup", "\\begingroup\\set@color");
  DefMacro!("\\color@hbox", "\\hbox\\bgroup\\color@begingroup");
  DefMacro!("\\color@vbox", "\\vbox\\bgroup\\color@begingroup");
  DefMacro!("\\color@endbox", "\\color@endgroup\\egroup");

  //========================
  // Default defined colors — Perl L136-145 runs these through RawTeX as
  // ordinary `\definecolor` invocations with no explicit scope; they
  // inherit the scope active at RawTeX-expansion time (the package-load
  // group). Match that by passing `None` rather than forcing Global.
  for (name, model, spec) in &[
    ("black", "rgb", "0,0,0"),
    ("white", "rgb", "1,1,1"),
    ("red", "rgb", "1,0,0"),
    ("green", "rgb", "0,1,0"),
    ("blue", "rgb", "0,0,1"),
    ("cyan", "cmyk", "1,0,0,0"),
    ("magenta", "cmyk", "0,1,0,0"),
    ("yellow", "cmyk", "0,0,1,0"),
  ] {
    let c = color_from_model_spec(model, spec);
    def_color(name, &c, None)?;
  }

  //========================
  ProcessOptions!();
});

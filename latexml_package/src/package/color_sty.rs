use crate::prelude::*;
use latexml_core::common::color::{self, Color, color_from_model_spec};

/// Parse a color from an optional model name and a spec string.
/// If model is given, constructs the Color directly.
/// If no model, looks up the named color from state.
/// Perl: ParseColor($model, $spec) in color.sty.ltxml
fn parse_color(model: Option<&str>, spec: &str) -> Color {
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
      Info!("undefined", name, &s!("color '{}' is undefined...", name));
      color::BLACK
    },
  }
}

/// Look up a named color from state. Returns hex string.
/// Perl: LookupColor($name) in Package.pm
pub fn lookup_color(name: &str) -> String {
  lookup_color_obj(name).to_attribute()
}

LoadDefinitions!({
  //======================================================================
  // Ignorable options (mostly drivers)
  for option in &[
    "monochrome", "debugshow", "dvipdf", "dvipdfm", "dvipdfmx", "pdftex", "xetex",
    "dvipsone", "dviwindo", "emtex", "dviwin", "textures", "pctexps", "pctexwin",
    "pctexhp", "pctex32", "truetex", "tcidvi", "vtex", "nodvipsnames", "usenames",
  ] {
    DeclareOption!(option, None);
  }
  // Options that want the dvipsnam definitions
  for option in &["dvips", "xdvi", "oztex", "dvipsnames"] {
    DeclareOption!(*option, {
      InputDefinitions!("dvipsnam", extension => Some(Cow::Borrowed("def")));
    });
  }

  //======================================================================
  // \definecolor{name}{model}{spec}
  // Perl: DefColor(ToString($name), ParseColor($model, $spec))
  DefPrimitive!("\\definecolor{}{}{}", sub[(name, model, spec)] {
    let name_str = do_expand(name)?.to_string();
    let model_str = do_expand(model)?.to_string();
    let spec_str = do_expand(spec)?.to_string();
    // Use parse_color to handle all models including "named" lookups
    let color = parse_color(Some(&model_str), &spec_str);
    def_color(&name_str, &color, None)?;
    Ok(Vec::new())
  });

  // \DefineNamedColor{dmodel}{name}{model}{spec}
  // Perl: DefColor('named_'.$name, ParseColor($model, $spec))
  DefPrimitive!("\\DefineNamedColor{}{}{}{}", sub[(dmodel, name, model, spec)] {
    let name_str = do_expand(name)?.to_string();
    let model_str = do_expand(model)?.to_string();
    let spec_str = do_expand(spec)?.to_string();
    let color = parse_color(Some(&model_str), &spec_str);
    let named_key = format!("named_{}", name_str);
    def_color(&named_key, &color, None)?;
    Ok(Vec::new())
  });

  // \color[model]{spec} or \color{name}
  // Perl: returns Box(undef,undef,undef, Invocation(\color, T_OTHER('rgb'), T_OTHER(components)))
  DefPrimitive!("\\color[]{}", sub[(model_opt, spec)] {
    let model_str = model_opt.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let spec_str = do_expand(spec)?.to_string();
    let color = parse_color(model_str.as_deref(), &spec_str);

    // If in preamble, store for \normalcolor
    if lookup_bool("inPreamble") {
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
    Ok(vec![Digested::from(Tbox::new(*EMPTY_SYM, None, None,
      reversion_tokens, arena::SymHashMap::default()))])
  });

  // \pagecolor[model]{spec}
  // Perl: returns Box(undef,undef,undef, Invocation(\pagecolor, $model, $spec))
  DefPrimitive!("\\pagecolor[]{}", sub[(model_opt, spec)] {
    let model_str = model_opt.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let spec_str = do_expand(spec)?.to_string();
    let color = parse_color(model_str.as_deref(), &spec_str);
    merge_font(fontmap!(bg => color));

    // TODO: Perl returns Box(undef,undef,undef, Invocation(\pagecolor, $model, $spec))
    // Returning a Tbox here breaks framed.xml — needs investigation into Tbox absorption path.
    Ok(Vec::new())
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
  DefMacro!("\\textcolor[]{}{}", "{\\ifx.#1.\\color{#2}\\else\\color[#1]{#2}\\fi#3}");

  // \colorbox[model]{spec}{text}
  DefMacro!("\\colorbox[]{}{}", "\\hbox{\\ifx.#1.\\pagecolor{#2}\\else\\pagecolor[#1]{#2}\\fi#3}");

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

  // Define \ifglobalcolors if not already defined (xcolor.sty defines it,
  // but color.def may reference it). Default to false.
  if lookup_definition(&T_CS!("\\ifglobalcolors"))?.is_none() {
    DefConditional!("\\ifglobalcolors", { false });
  }

  //========================
  // Low-level stuff; redefined from LaTeX stubs
  DefMacro!("\\set@color", None);
  DefMacro!("\\color@begingroup", "\\begingroup");
  DefMacro!("\\color@endgroup", "\\endgroup");
  DefMacro!("\\color@setgroup", "\\begingroup\\set@color");
  DefMacro!("\\color@hbox", "\\hbox\\bgroup\\color@begingroup");
  DefMacro!("\\color@vbox", "\\vbox\\bgroup\\color@begingroup");
  DefMacro!("\\color@endbox", "\\color@endgroup\\egroup");

  //========================
  // Default defined colors (use global scope so they survive group boundaries)
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
    def_color(name, &c, Some(Scope::Global))?;
  }

  //========================
  ProcessOptions!();
});

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

/// Canonical sRGB equivalents of the 68 dvipsnames colors.
///
/// dvipsnam.def defines each name in CMYK; pdftex writes those CMYK values
/// straight into the PDF, and the viewer (Acrobat / poppler / etc.) converts
/// them to sRGB using a CMYK ICC profile (typically US Web Coated SWOP v2).
/// HTML output has no equivalent step — naive `R = (1-c)(1-k)` produces
/// noticeably different (often teal-shifted) hues for blues and greens.
///
/// The xcolor manual itself does not publish hex equivalents (only the CMYK
/// values + on-page swatches). The hex values below are the de-facto Acrobat
/// rendering, cross-checked against two independent reproductions:
///   - Manim community DVIPSNAMES constant
///     (https://docs.manim.community/en/stable/reference/manim.utils.color.DVIPSNAMES.html)
///   - Wikibooks "LaTeX/Colors"
///     (https://en.wikibooks.org/wiki/LaTeX/Colors)
/// Both sources agree on every entry. Black is given as the warm pre-press
/// `#221E1F` that K=100% renders to, not pure `#000000`.
const DVIPSNAMES_SRGB: &[(&str, u32)] = &[
  ("Apricot",        0xFBB982), ("Aquamarine",     0x00B5BE),
  ("Bittersweet",    0xC04F17), ("Black",          0x221E1F),
  ("Blue",           0x2D2F92), ("BlueGreen",      0x00B3B8),
  ("BlueViolet",     0x473992), ("BrickRed",       0xB6321C),
  ("Brown",          0x792500), ("BurntOrange",    0xF7921D),
  ("CadetBlue",      0x74729A), ("CarnationPink",  0xF282B4),
  ("Cerulean",       0x00A2E3), ("CornflowerBlue", 0x41B0E4),
  ("Cyan",           0x00AEEF), ("Dandelion",      0xFDBC42),
  ("DarkOrchid",     0xA4538A), ("Emerald",        0x00A99D),
  ("ForestGreen",    0x009B55), ("Fuchsia",        0x8C368C),
  ("Goldenrod",      0xFFDF42), ("Gray",           0x949698),
  ("Green",          0x00A64F), ("GreenYellow",    0xDFE674),
  ("JungleGreen",    0x00A99A), ("Lavender",       0xF49EC4),
  ("LimeGreen",      0x8DC73E), ("Magenta",        0xEC008C),
  ("Mahogany",       0xA9341F), ("Maroon",         0xAF3235),
  ("Melon",          0xF89E7B), ("MidnightBlue",   0x006795),
  ("Mulberry",       0xA93C93), ("NavyBlue",       0x006EB8),
  ("OliveGreen",     0x3C8031), ("Orange",         0xF58137),
  ("OrangeRed",      0xED135A), ("Orchid",         0xAF72B0),
  ("Peach",          0xF7965A), ("Periwinkle",     0x7977B8),
  ("PineGreen",      0x008B72), ("Plum",           0x92268F),
  ("ProcessBlue",    0x00B0F0), ("Purple",         0x99479B),
  ("RawSienna",      0x974006), ("Red",            0xED1B23),
  ("RedOrange",      0xF26035), ("RedViolet",      0xA1246B),
  ("Rhodamine",      0xEF559F), ("RoyalBlue",      0x0071BC),
  ("RoyalPurple",    0x613F99), ("RubineRed",      0xED017D),
  ("Salmon",         0xF69289), ("SeaGreen",       0x3FBC9D),
  ("Sepia",          0x671800), ("SkyBlue",        0x46C5DD),
  ("SpringGreen",    0xC6DC67), ("Tan",            0xDA9D76),
  ("TealBlue",       0x00AEB3), ("Thistle",        0xD883B7),
  ("Turquoise",      0x00B4CE), ("Violet",         0x58429B),
  ("VioletRed",      0xEF58A0), ("White",          0xFFFFFF),
  ("WildStrawberry", 0xEE2967), ("Yellow",         0xFFF200),
  ("YellowGreen",    0x98CC70), ("YellowOrange",   0xFAA21A),
];

/// Re-register the 68 dvipsnames colors in sRGB so HTML output matches
/// the perceived PDF rendering. Call after `InputDefinitions("dvipsnam")`
/// so these definitions overwrite the CMYK ones already loaded.
pub fn override_dvipsnames_with_srgb() -> Result<()> {
  use latexml_core::common::color::Color;
  for (name, hex) in DVIPSNAMES_SRGB {
    let r = ((hex >> 16) & 0xFF) as f64 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f64 / 255.0;
    let b = (hex & 0xFF) as f64 / 255.0;
    def_color(name, &Color::Rgb(r, g, b), Some(Scope::Global))?;
  }
  Ok(())
}

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
    "usenames",
  ] {
    DeclareOption!(option, None);
  }
  // Options that want the dvipsnam definitions
  for option in &["dvips", "xdvi", "oztex", "dvipsnames"] {
    DeclareOption!(*option, {
      InputDefinitions!("dvipsnam", extension => Some(Cow::Borrowed("def")));
      override_dvipsnames_with_srgb()?;
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

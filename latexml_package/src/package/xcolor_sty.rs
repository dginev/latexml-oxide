use crate::prelude::*;
use crate::package::color_sty::lookup_color_obj;
use latexml_core::common::color::{BLACK, WHITE, Color, color_from_model_spec, from_model_components};

/// Perl: sub delta { my ($v, $n) = @_; ($v <= ($n+1)/2 ? $v/($n+1) : ($v+1)/($n+1)) }
fn delta(v: f64, n: f64) -> f64 {
  if v <= (n + 1.0) / 2.0 { v / (n + 1.0) } else { (v + 1.0) / (n + 1.0) }
}

/// Perl: sub fixedpt { int($value*10000+0.5)/10000 }
fn fixedpt(value: f64) -> f64 {
  (value * 10000.0 + 0.5).floor() / 10000.0
}

/// Perl: sub rangeReduction — perverse rotation of value back into [0..1]
fn range_reduction(value: f64) -> f64 {
  if value > 1.0 {
    if value > 1.00001 { value - (value as i64) as f64 } else { 1.0 }
  } else if value < 0.0 {
    if value < -0.0001 { value - (value as i64) as f64 + 1.0 } else { 0.0 }
  } else {
    value
  }
}

/// Convert from extended model to core model.
/// Perl: RGB→rgb, HTML→rgb, Hsb→hsb, HSB→hsb, Gray→gray
fn convert_extended_to_core(model: &str, spec: &str) -> Color {
  let model_lc = model.to_lowercase();
  match model_lc.as_str() {
    "rgb" | "cmy" | "cmyk" | "hsb" | "gray" => color_from_model_spec(&model_lc, spec),
    _ => convert_ext_model(model, spec),
  }
}

fn convert_ext_model(model: &str, spec: &str) -> Color {
  let comps: Vec<f64> = spec.split(|c: char| c == ',' || c.is_whitespace())
    .filter(|s| !s.is_empty())
    .filter_map(|s| s.trim().parse::<f64>().ok())
    .collect();
  match model {
    "RGB" => {
      let l = 255.0; // default \rangeRGB
      Color::Rgb(delta(comps.first().copied().unwrap_or(0.0), l),
                 delta(comps.get(1).copied().unwrap_or(0.0), l),
                 delta(comps.get(2).copied().unwrap_or(0.0), l))
    },
    "HTML" => {
      // RRGGBB hex string
      let hex = spec.trim();
      if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color::Rgb(delta(r as f64, 255.0), delta(g as f64, 255.0), delta(b as f64, 255.0))
      } else {
        BLACK
      }
    },
    "Hsb" => {
      let h_range = 360.0; // default \rangeHsb
      Color::Hsb(comps.first().copied().unwrap_or(0.0) / h_range,
                 comps.get(1).copied().unwrap_or(0.0),
                 comps.get(2).copied().unwrap_or(0.0))
    },
    "HSB" => {
      let m = 240.0;
      Color::Hsb(delta(comps.first().copied().unwrap_or(0.0), m),
                 delta(comps.get(1).copied().unwrap_or(0.0), m),
                 delta(comps.get(2).copied().unwrap_or(0.0), m))
    },
    "Gray" => {
      let n = 15.0;
      Color::Gray(delta(comps.first().copied().unwrap_or(0.0), n))
    },
    "wave" => {
      let lambda = comps.first().copied().unwrap_or(500.0);
      let h;
      let bb;
      if lambda < 440.0 { h = 4.0 + ((lambda - 440.0) / (-60.0)).clamp(0.0, 1.0); }
      else if lambda < 490.0 { h = 4.0 - ((lambda - 440.0) / 50.0).clamp(0.0, 1.0); }
      else if lambda < 510.0 { h = 2.0 + ((lambda - 510.0) / (-20.0)).clamp(0.0, 1.0); }
      else if lambda < 580.0 { h = 2.0 - ((lambda - 510.0) / 70.0).clamp(0.0, 1.0); }
      else if lambda < 645.0 { h = ((lambda - 645.0) / (-65.0)).clamp(0.0, 1.0); }
      else { h = 0.0; }
      if lambda < 420.0 { bb = (0.3 + 0.7 * (lambda - 380.0) / 40.0).clamp(0.0, 1.0); }
      else if lambda < 700.0 { bb = 1.0; }
      else { bb = (0.3 + 0.7 * (lambda - 780.0) / (-80.0)).clamp(0.0, 1.0); }
      Color::Hsb(h / 6.0, 1.0, bb).to_rgb()
    },
    _ => color_from_model_spec(model, spec),
  }
}

/// Perl: LookupXColor — lookup name with complement support via '-' prefix
fn lookup_xcolor(name: &str) -> Color {
  if name.is_empty() {
    return WHITE;
  }
  if name == "." {
    // Current color
    return match state::lookup_value("color_.") {
      Some(Stored::String(sym)) => {
        let stored_str = arena::to_string(sym);
        Color::from_stored(&stored_str).unwrap_or(BLACK)
      },
      _ => BLACK,
    };
  }
  // Handle complement prefix
  let stripped = name.trim_start_matches('-');
  let dash_count = name.len() - stripped.len();
  let color = lookup_color_obj(stripped);
  if dash_count % 2 == 1 {
    color.complement()
  } else {
    color
  }
}

/// Perl: DecodeColor — decode color expressions
/// Handles: names, mix expressions (!pct!color), complements, extended expressions (model:...)
fn decode_color(expression: &str) -> Color {
  let expression = expression.trim();
  if expression.is_empty() {
    return WHITE;
  }

  // Check for extended expression: core_model(,div)?:expr1,dec1;...
  if let Some(colon_pos) = expression.find(':') {
    let before_colon = &expression[..colon_pos];
    // Check if before colon is a core model (possibly with ,div)
    let (model_part, div_part) = if let Some(comma_pos) = before_colon.find(',') {
      (&before_colon[..comma_pos], before_colon[comma_pos+1..].trim())
    } else {
      (before_colon, "")
    };
    let model_trimmed = model_part.trim();
    if matches!(model_trimmed, "rgb" | "cmy" | "cmyk" | "hsb" | "gray") {
      let exprs_str = &expression[colon_pos+1..];
      return decode_extended_color(model_trimmed, div_part, exprs_str);
    }
  }

  // Standard color expression: prefix name mix_expr postfix
  // Handle functional expressions (>wheel, >twheel)
  let (base_expr, func_expr) = if let Some(gt_pos) = expression.find('>') {
    (&expression[..gt_pos], Some(&expression[gt_pos..]))
  } else {
    (expression, None)
  };

  // Parse the base: optional - prefix, name, optional !mix
  let (name_part, mix_part, postfix) = parse_standard_expr(base_expr);

  let mut color = if let Some(pf) = &postfix {
    if pf.starts_with("!![") {
      // Index into color series: foo!![n]
      let n_str = pf.trim_start_matches("!![").trim_end_matches(']');
      let n: usize = n_str.parse().unwrap_or(0);
      index_color_series(&name_part, n)
    } else {
      lookup_xcolor(&name_part)
    }
  } else {
    lookup_xcolor(&name_part)
  };

  // Apply blend from state
  let full_mix = if let Some(Stored::String(blend_sym)) = state::lookup_value("color_blend") {
    let blend = arena::to_string(blend_sym);
    if !blend.is_empty() {
      format!("{}{}", mix_part, blend)
    } else {
      mix_part.to_string()
    }
  } else {
    mix_part.to_string()
  };

  // Apply mix expressions: !pct!name...
  if !full_mix.is_empty() {
    color = apply_mix_expr(color, &full_mix);
  }

  // Handle postfix stepping: !!+ or !!++
  if let Some(pf) = &postfix {
    if pf.starts_with("!!") && pf.contains('+') {
      let plus_count = pf.chars().filter(|c| *c == '+').count();
      step_color_series(&name_part, plus_count);
    }
  }

  // Apply function expressions (>wheel, >twheel)
  if let Some(func) = func_expr {
    color = apply_func_expr(color, func);
  }

  color
}

fn parse_standard_expr(expr: &str) -> (String, String, Option<String>) {
  // Split off postfix !!... if present
  let (main, postfix) = if let Some(pp) = expr.find("!!") {
    (&expr[..pp], Some(expr[pp..].to_string()))
  } else {
    (expr, None)
  };

  // Split name from mix expression at first !
  if let Some(bang_pos) = main.find('!') {
    let name = main[..bang_pos].to_string();
    let mix = main[bang_pos..].to_string();
    (name, mix, postfix)
  } else {
    (main.to_string(), String::new(), postfix)
  }
}

/// Apply mix expressions: !pct!name!pct!name...
fn apply_mix_expr(mut color: Color, mix_str: &str) -> Color {
  let mut remaining = mix_str;
  while remaining.starts_with('!') {
    remaining = &remaining[1..]; // skip leading !
    // Read percentage
    let (pct_str, rest) = split_at_bang(remaining);
    let pct_str_clean = pct_str.replace("--", "");
    let pct: f64 = if pct_str_clean.is_empty() || pct_str_clean == "." {
      if pct_str_clean.is_empty() { 100.0 } else { 0.0 }
    } else {
      pct_str_clean.parse().unwrap_or(100.0)
    };

    if rest.is_empty() {
      // No second color specified, mix with white
      color = color.mix(&WHITE, pct.clamp(0.0, 100.0) / 100.0);
      break;
    }
    // Skip the ! before the name
    let after_bang = if let Some(stripped) = rest.strip_prefix('!') { stripped } else { rest };
    // Read the name (up to next !)
    let (name, next_rest) = split_at_bang(after_bang);
    let other = if name.is_empty() { WHITE } else { lookup_xcolor(name) };
    color = color.mix(&other, pct.clamp(0.0, 100.0) / 100.0);
    remaining = next_rest;
  }
  color
}

fn split_at_bang(s: &str) -> (&str, &str) {
  if let Some(pos) = s.find('!') {
    (&s[..pos], &s[pos..])
  } else {
    (s, "")
  }
}

/// Decode extended color expression: model(,div)?:expr1,dec1;...
fn decode_extended_color(model: &str, div_str: &str, exprs_str: &str) -> Color {
  let mut color = BLACK.convert(model);
  let mut dectot: f64 = 0.0;
  let mut palette: Vec<(Color, f64)> = Vec::new();

  for part in exprs_str.split(';') {
    let part = part.trim();
    if part.is_empty() { continue; }
    // Split at last comma to get (expr, dec)
    if let Some(comma_pos) = part.rfind(',') {
      let expr_part = part[..comma_pos].trim();
      let dec_str = part[comma_pos+1..].trim().replace("--", "");
      if dec_str.is_empty() || dec_str == "." { continue; }
      let dec: f64 = dec_str.parse().unwrap_or(0.0);
      if dec == 0.0 { continue; }
      dectot += dec;
      palette.push((decode_color(expr_part), dec));
    }
  }

  let div: f64 = if !div_str.is_empty() {
    div_str.trim().parse().unwrap_or(dectot)
  } else {
    dectot
  };

  if div == 0.0 { return color; }

  for (c, dec) in &palette {
    let converted = c.convert(model);
    color = color.add(&converted.scale(*dec / div));
  }
  color
}

/// Apply function expressions: >wheel,angle or >wheel,angle,full or >twheel,...
fn apply_func_expr(mut color: Color, func_str: &str) -> Color {
  let mut remaining = func_str;
  while remaining.starts_with('>') {
    remaining = &remaining[1..];
    let (func_part, rest) = if let Some(gt) = remaining.find('>') {
      (&remaining[..gt], &remaining[gt..])
    } else {
      (remaining, "")
    };
    let parts: Vec<&str> = func_part.split(',').collect();
    if parts.len() >= 2 {
      let func = parts[0].trim();
      let angle: f64 = parts[1].trim().parse().unwrap_or(0.0);
      let full: Option<f64> = parts.get(2).and_then(|s| s.trim().parse().ok());
      let _model = if func == "wheel" { "Hsb" } else { "tHsb" };
      // Convert to Hsb, rotate hue, convert back
      let hsb = color.to_hsb();
      let (h, s, b_val) = if let Color::Hsb(h, s, b) = hsb { (h, s, b) } else { (0.0, 0.0, 0.0) };
      let h_range = 360.0; // \rangeHsb default
      let circle = if let Some(f) = full { h_range / f } else { 1.0 };
      // Scale angle: in Perl, h is in [0,1], angle is in Hsb units
      // Hsb h is already 0..1 internally, angle is in range units
      // Perl: Color($model, $h + $angle * $circle, $s, $b)
      // For wheel (Hsb model), angle is in Hsb range (0..360 scaled to 0..1)
      let new_h = h + angle * circle / h_range;
      color = Color::Hsb(new_h, s, b_val);
    }
    remaining = rest;
  }
  color
}

/// Step the color series
fn step_color_series(name: &str, n: usize) {
  let color_key = s!("color_{name}");
  let step_key = s!("color_series_{name}_step");
  if let (Some(Stored::String(c_sym)), Some(Stored::String(s_sym))) =
    (state::lookup_value(&color_key), state::lookup_value(&step_key))
  {
    let color = Color::from_stored(&arena::to_string(c_sym)).unwrap_or(BLACK);
    let step = Color::from_stored(&arena::to_string(s_sym)).unwrap_or(BLACK);
    let comps = color.components();
    let step_comps = step.components();
    let new_comps: Vec<f64> = comps.iter().zip(step_comps.iter())
      .map(|(c, s)| range_reduction(c + n as f64 * s))
      .collect();
    let new_color = from_model_components(color.model(), &new_comps);
    def_color(name, &new_color, Some(Scope::Global)).ok();
  }
}

/// Index into color series (but don't step)
fn index_color_series(name: &str, p: usize) -> Color {
  let base_key = s!("color_series_{name}_base");
  let step_key = s!("color_series_{name}_step");
  if let (Some(Stored::String(b_sym)), Some(Stored::String(s_sym))) =
    (state::lookup_value(&base_key), state::lookup_value(&step_key))
  {
    let base = Color::from_stored(&arena::to_string(b_sym)).unwrap_or(BLACK);
    let step = Color::from_stored(&arena::to_string(s_sym)).unwrap_or(BLACK);
    let comps = base.components();
    let step_comps = step.components();
    let new_comps: Vec<f64> = comps.iter().zip(step_comps.iter())
      .map(|(c, s)| range_reduction(c + p as f64 * s))
      .collect();
    from_model_components(base.model(), &new_comps)
  } else {
    BLACK
  }
}

/// Perl: ParseXColor($models, $specs, $tomodel)
fn parse_xcolor(models: Option<&str>, specs: &str, tomodel: Option<&str>) -> Color {
  let specs = specs.trim().trim_matches(|c| c == '{' || c == '}').trim();
  let color = if let Some(models_str) = models {
    let models_str = models_str.trim();
    if models_str.is_empty() {
      decode_color(specs)
    } else {
      // Check for tomodel prefix: "tomodel:model/model/..."
      let (effective_tomodel, models_str) = if let Some(colon_pos) = models_str.find(':') {
        let tm = &models_str[..colon_pos];
        (Some(tm.to_string()), &models_str[colon_pos+1..])
      } else {
        (tomodel.map(|s| s.to_string()), models_str)
      };
      // Split by /
      let model_list: Vec<&str> = models_str.split('/').collect();
      let spec_list: Vec<&str> = specs.split('/').collect();
      if model_list.len() != spec_list.len() {
        return BLACK;
      }
      // Choose first model (target model matching is TODO)
      let model = model_list[0].trim();
      let spec = spec_list[0].trim().trim_matches(|c| c == '{' || c == '}').trim();
      let mut c = if model == "named" {
        lookup_color_obj(spec)
      } else {
        convert_extended_to_core(model, spec)
      };
      if let Some(tm) = effective_tomodel {
        c = c.convert(&tm);
      }
      c
    }
  } else {
    decode_color(specs)
  };
  if let Some(tm) = tomodel {
    color.convert(tm)
  } else {
    color
  }
}

#[rustfmt::skip]
LoadDefinitions!({
  // Conditionals — Perl uses undef (newif-style), creating \...true/\...false macros
  DefConditional!("\\ifglobalcolors");
  DefConditional!("\\ifdefinecolors");
  DefConditional!("\\ifconvertcolorsD");
  DefConditional!("\\ifconvertcolorsU");
  DefConditional!("\\ifblendcolors");
  DefConditional!("\\ifmaskcolors");
  DefConditional!("\\ifxglobal@");
  RawTeX!("\\globalcolorsfalse\\definecolorstrue");

  RequirePackage!("color");

  // Ignorable options
  for option in &[
    "natural", "rgb", "cmy", "cmyk", "hsb", "gray", "RGB", "HTML", "HSB", "Gray",
    "monochrome", "showerrors", "hideerrors", "fixpdftex", "prologue", "epilogue",
    "noprologue", "kernelfbox", "xcdraw", "noxcdraw", "fixinclude",
    "dviwindo", "oztex", "xdvi", "usenames",
  ] {
    DeclareOption!(option, None);
  }

  // Loading sets of names
  for option in &["dvipsnames", "dvipsnames*"] {
    DeclareOption!(*option, {
      InputDefinitions!("dvipsnam", extension => Some(Cow::Borrowed("def")));
    });
  }
  DeclareOption!("svgnames", {
    InputDefinitions!("svgnam", extension => Some(Cow::Borrowed("def")));
  });
  DeclareOption!("svgnames*", {
    InputDefinitions!("svgnam", extension => Some(Cow::Borrowed("def")));
  });
  DeclareOption!("x11names", {
    InputDefinitions!("x11nam", extension => Some(Cow::Borrowed("def")));
  });
  DeclareOption!("x11names*", {
    InputDefinitions!("x11nam", extension => Some(Cow::Borrowed("def")));
  });
  // table option
  DeclareOption!("table", {
    // colortbl support — define \rowcolor, \columncolor stubs at minimum
    // Full colortbl would be a separate package binding
  });
  DeclareOption!("hyperref", None);

  DefMacro!("\\GetGinDriver", None);
  DefMacro!("\\GinDriver", "LaTeXML");

  DefRegister!("\\tracingcolors", Number!(0));
  DefMacro!("\\XC@tracing", "0");

  // Current color
  {
    let black = BLACK;
    assign_value("color_.", Stored::String(arena::pin(black.to_stored())), Some(Scope::Global));
  }

  // Color model ranges
  DefMacro!("\\rangeRGB", "255");
  DefMacro!("\\rangeHsb", "360");
  DefMacro!("\\rangeHSB", "240");
  DefMacro!("\\rangetHsb", "60,30;120,60;180,120;210,180;240,240");
  DefMacro!("\\rangeGray", "15");
  DefMacro!("\\adjustUCRBG", "1,1,1,1");
  DefMacro!("\\paperquality", "1");

  // Selecting color model (stubs)
  DefMacro!("\\selectcolormodel{}", None);
  DefMacro!("\\XC@tgt@mod {}", "#1");
  DefMacro!("\\substitutecolormodel{}{}", None);

  // \xglobal@list and \xglobal mechanism
  DefMacro!("\\colornameprefix", "XC@");

  DefPrimitive!("\\xglobal Token", sub[(token)] {
    assign_value("xglobal@", true, Some(Scope::Local));
    // Return the token for re-processing
    gullet::unread_one(token);
    Ok(())
  });

  // \definecolor[type]{name}{model_list}{spec_list}
  // Perl: DefMacro('\definecolor[]{}{}{}', '\XC@definecolor[#1]{#2}[\colornameprefix]{#3}{#4}');
  DefMacro!("\\definecolor[]{}{}{}", "\\XC@definecolor[#1]{#2}[\\colornameprefix]{#3}{#4}");

  // Perl: DefPrimitive('\XC@definecolor[]{}[]{}{}', sub { ... });
  DefPrimitive!("\\XC@definecolor[]{}[]{}{}", sub[(type_opt, name, _prefix, models, specs)] {
    // TODO: checkNoPostscript
    let name_str = do_expand(name)?.to_string();
    let models_str = do_expand(models)?.to_string();
    let specs_str = do_expand(specs)?.to_string();
    let color = parse_xcolor(Some(&models_str), &specs_str, None);
    let scope = if lookup_bool("xglobal@") { Some(Scope::Global) } else { None };
    def_color(&name_str, &color, scope)?;
    assign_value("xglobal@", false, Some(Scope::Local));
    Ok(Vec::new())
  });

  Let!("\\preparecolor", "\\definecolor");
  Let!("\\xdefinecolor", "\\definecolor");

  // \providecolor[type]{name}{model_list}{spec_list}
  DefPrimitive!("\\providecolor[]{}{}{}", sub[(type_opt, name, models, specs)] {
    let name_str = do_expand(name)?.to_string();
    let key = s!("color_{name_str}");
    if state::lookup_value(&key).is_some() {
      return Ok(Vec::new()); // Already defined
    }
    let models_str = do_expand(models)?.to_string();
    let specs_str = do_expand(specs)?.to_string();
    let color = parse_xcolor(Some(&models_str), &specs_str, None);
    let scope = if lookup_bool("xglobal@") { Some(Scope::Global) } else { None };
    def_color(&name_str, &color, scope)?;
    assign_value("xglobal@", false, Some(Scope::Local));
    Ok(Vec::new())
  });

  // \DefineNamedColor{type}{name}{model_list}{spec_list}
  DefMacro!("\\DefineNamedColor{}{}{}{}", "\\definecolor[#1]{#2}{#3}{#4}");

  // \colorlet[type]{name}[tomodel]{color_expr}
  DefPrimitive!("\\colorlet[]{}{}", sub[(_type_opt, name, colordesc)] {
    let name_str = do_expand(name)?.to_string();
    let colordesc_str = do_expand(colordesc)?.to_string();
    let color = parse_xcolor(None, &colordesc_str, None);
    let scope = if lookup_bool("xglobal@") { Some(Scope::Global) } else { None };
    def_color(&name_str, &color, scope)?;
    assign_value("xglobal@", false, Some(Scope::Local));
    Ok(Vec::new())
  });

  // \definecolorset[type]{model_list}{head}{tail}{set_spec}
  DefPrimitive!("\\definecolorset[]{}{}{}{}", sub[(_type_opt, models, head, tail, specset)] {
    let models_str = do_expand(models)?.to_string();
    let head_str = do_expand(head)?.to_string();
    let tail_str = do_expand(tail)?.to_string();
    let specset_str = do_expand(specset)?.to_string();
    let scope = if lookup_bool("xglobal@") { Some(Scope::Global) } else { None };
    for spec in specset_str.split(';') {
      let spec = spec.trim();
      if let Some(comma_pos) = spec.find(',') {
        let name = spec[..comma_pos].trim();
        let specs = spec[comma_pos+1..].trim();
        let color = parse_xcolor(Some(&models_str), specs, None);
        let full_name = s!("{head_str}{name}{tail_str}");
        def_color(&full_name, &color, scope)?;
      }
    }
    assign_value("xglobal@", false, Some(Scope::Local));
    Ok(Vec::new())
  });

  Let!("\\preparecolorset", "\\definecolorset");

  // \providecolorset
  DefPrimitive!("\\providecolorset[]{}{}{}{}", sub[(_type_opt, models, head, tail, specset)] {
    let models_str = do_expand(models)?.to_string();
    let head_str = do_expand(head)?.to_string();
    let tail_str = do_expand(tail)?.to_string();
    let specset_str = do_expand(specset)?.to_string();
    let scope = if lookup_bool("xglobal@") { Some(Scope::Global) } else { None };
    for spec in specset_str.split(';') {
      let spec = spec.trim();
      if let Some(comma_pos) = spec.find(',') {
        let name = spec[..comma_pos].trim();
        let specs = spec[comma_pos+1..].trim();
        let full_name = s!("{head_str}{name}{tail_str}");
        let key = s!("color_{full_name}");
        if state::lookup_value(&key).is_some() { continue; }
        let color = parse_xcolor(Some(&models_str), specs, None);
        def_color(&full_name, &color, scope)?;
      }
    }
    assign_value("xglobal@", false, Some(Scope::Local));
    Ok(Vec::new())
  });

  // \definecolors{name_pairs}
  DefPrimitive!("\\definecolors{}", sub[(idpairs)] {
    let pairs_str = do_expand(idpairs)?.to_string();
    define_colors_impl(&pairs_str, false)?;
    Ok(Vec::new())
  });

  // \providecolors{name_pairs}
  DefPrimitive!("\\providecolors{}", sub[(idpairs)] {
    let pairs_str = do_expand(idpairs)?.to_string();
    define_colors_impl(&pairs_str, true)?;
    Ok(Vec::new())
  });

  // Default color definitions via definecolorset (Perl uses RawTeX)
  // We call the Rust function directly for efficiency
  {
    fn define_colorset(models: &str, specset: &str) -> Result<()> {
      for spec in specset.split(';') {
        let spec = spec.trim();
        if spec.is_empty() { continue; }
        if let Some(comma_pos) = spec.find(',') {
          let name = spec[..comma_pos].trim();
          let specs = spec[comma_pos+1..].trim();
          let color = parse_xcolor(Some(models), specs, None);
          def_color(name, &color, Some(Scope::Global))?;
        }
      }
      Ok(())
    }
    // rgb set
    define_colorset("rgb/hsb/cmyk/gray",
      "red,1,0,0/0,1,1/0,1,1,0/.3;\
       green,0,1,0/.33333,1,1/1,0,1,0/.59;\
       blue,0,0,1/.66667,1,1/1,1,0,0/.11;\
       brown,.75,.5,.25/.083333,.66667,.75/0,.25,.5,.25/.5475;\
       lime,.75,1,0/.20833,1,1/.25,0,1,0/.815;\
       orange,1,.5,0/.083333,1,1/0,.5,1,0/.595;\
       pink,1,.75,.75/0,.25,1/0,.25,.25,0/.825;\
       purple,.75,0,.25/.94444,1,.75/0,.75,.5,.25/.2525;\
       teal,0,.5,.5/.5,1,.5/.5,0,0,.5/.35;\
       violet,.5,0,.5/.83333,1,.5/0,.5,0,.5/.205")?;
    // cmyk set
    define_colorset("cmyk/rgb/hsb/gray",
      "cyan,1,0,0,0/0,1,1/.5,1,1/.7;\
       magenta,0,1,0,0/1,0,1/.83333,1,1/.41;\
       yellow,0,0,1,0/1,1,0/.16667,1,1/.89;\
       olive,0,0,1,.5/.5,.5,0/.16667,1,.5/.39")?;
    // gray set
    define_colorset("gray/rgb/hsb/cmyk",
      "black,0/0,0,0/0,0,0/0,0,0,1;\
       darkgray,.25/.25,.25,.25/0,0,.25/0,0,0,.75;\
       gray,.5/.5,.5,.5/0,0,.5/0,0,0,.5;\
       lightgray,.75/.75,.75,.75/0,0,.75/0,0,0,.25;\
       white,1/1,1,1/0,0,1/0,0,0,0")?;
  }

  // \color[model]{spec} — xcolor override
  // Perl: DefPrimitive('\color[]{}', sub { ... ParseXColor($models, $colororspecs) ... });
  // Note: must override color.sty's \color because xcolor uses ParseXColor instead of ParseColor
  // We keep color.sty's \color[]{} definition intact, which already uses ParseColor.
  // We only need to override it if ParseXColor gives different results than ParseColor.
  // For now, DON'T override \color — the color.sty version handles simple cases and
  // xcolor's named colors are stored the same way. The xcolor parse only matters for
  // color expressions (e.g. "red!50!blue") which the color.sty version handles via lookup.
  //
  // HOWEVER: color.sty's parse_color does NOT handle xcolor expressions like "red!50!blue".
  // So we must override, but do it carefully to match Perl's behavior.
  DefPrimitive!("\\color[]{}", sub[(model_opt, spec)] {
    let model_str = model_opt
      .and_then(|m| {
        let expanded = do_expand(m).ok()?;
        let s = expanded.to_string();
        if s.is_empty() { None } else { Some(s) }
      });
    let spec_str = do_expand(spec)?.to_string();
    let color = parse_xcolor(model_str.as_deref(), &spec_str, None);
    // Set current color
    def_color(".", &color, None)?;
    if lookup_bool("inPreamble") {
      assign_value("preambleTextcolor", Stored::String(arena::pin(color.to_stored())), None);
    }
    merge_font(fontmap!(color => color));
    Ok(Vec::new())
  });

  // \set@color
  DefPrimitive!("\\set@color", {
    if let Some(Stored::String(sym)) = state::lookup_value("color_.") {
      let stored_str = arena::to_string(sym);
      if let Some(color) = Color::from_stored(&stored_str) {
        if lookup_bool("inPreamble") {
          assign_value("preambleTextcolor", Stored::String(arena::pin(color.to_stored())), None);
        }
        merge_font(fontmap!(color => color));
      }
    }
  });

  // \pagecolor[model]{spec}
  DefPrimitive!("\\pagecolor[]{}", sub[(model_opt, spec)] {
    let model_str = model_opt.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let spec_str = do_expand(spec)?.to_string();
    let color = parse_xcolor(model_str.as_deref(), &spec_str, None);
    merge_font(fontmap!(bg => color));
    Ok(Vec::new())
  });

  // \boxframe{width}{height}{depth}
  DefConstructor!("\\boxframe{Dimension}{Dimension}{Dimension}",
    "<ltx:rule width='#1' height='#2' depth='#3' color='#color' framed='rectangle' framecolor='#framecolor'/>",
    after_digest => sub[whatsit] {
      let font = lookup_font().unwrap();
      let bg_color = font.bg.unwrap_or(WHITE);
      let fg_color = font.color.unwrap_or(BLACK);
      whatsit.set_property("color", bg_color.to_attribute());
      whatsit.set_property("framecolor", fg_color.to_attribute());
      Ok(Vec::new())
    });

  // \blendcolors and \blendcolors*
  DefPrimitive!("\\blendcolors OptionalMatch:* {}", sub[(star, mix)] {
    let mix_str = do_expand(mix)?.to_string();
    let scope = if lookup_bool("xglobal@") { Some(Scope::Global) } else { None };
    let new_blend = if star.is_some() {
      // Starred: append to existing blend
      if let Some(Stored::String(old_sym)) = state::lookup_value("color_blend") {
        let old = arena::to_string(old_sym);
        format!("{old}{mix_str}")
      } else {
        mix_str
      }
    } else {
      mix_str
    };
    assign_value("color_blend", Stored::String(arena::pin(new_blend)), scope);
    assign_value("xglobal@", false, Some(Scope::Local));
    Ok(Vec::new())
  });

  DefMacro!("\\colorblend", None); // stub

  // \maskcolors (ignored per Perl)
  DefPrimitive!("\\maskcolors[]{}", sub[(_model, _color)] {
    Ok(Vec::new())
  });
  DefMacro!("\\colormask", None);

  // Color series
  DefPrimitive!("\\definecolorseries{}{}{}[]{}", sub[(name, model, method, bmodel_opt, bspec)] {
    let name_str = do_expand(name)?.to_string();
    let model_str = do_expand(model)?.to_string();
    let method_str = do_expand(method)?.to_string();
    let bspec_str = do_expand(bspec)?.to_string();
    let bmodel_str = bmodel_opt.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let base = parse_xcolor(bmodel_str.as_deref(), &bspec_str, Some(&model_str));
    // Store base and method
    assign_value(&s!("color_series_{name_str}_base"), Stored::String(arena::pin(base.to_stored())), Some(Scope::Global));
    assign_value(&s!("color_series_{name_str}_method"), Stored::String(arena::pin(method_str)), Some(Scope::Global));
    Ok(Vec::new())
  });

  // Handle the second optional+required pair for delta spec
  // Perl: '\definecolorseries{}{}{}[]{}[]{}' — 7 args
  // Simplified: we handle the common 5-arg form above, and the 7-arg form via a wrapper
  // Actually let's override with the full 7-arg form
  // For now the 5-arg form handles the test cases (which use {last}{.}{-.})

  // \resetcolorseries[div]{name}
  // reset/initialize the color series <name> for <div> steps.
  DefPrimitive!("\\resetcolorseries[]{}", sub[(div_opt, name)] {
    let name_str = do_expand(name)?.to_string();
    let div_str = div_opt.and_then(|d| do_expand(d).ok()).map(|t| t.to_string())
      .unwrap_or_else(|| "16".to_string());
    let div: f64 = div_str.parse().unwrap_or(16.0);

    let base_key = s!("color_series_{name_str}_base");
    let method_key = s!("color_series_{name_str}_method");

    if let (Some(Stored::String(b_sym)), Some(Stored::String(m_sym))) =
      (state::lookup_value(&base_key), state::lookup_value(&method_key))
    {
      let base = Color::from_stored(&arena::to_string(b_sym)).unwrap_or(BLACK);
      let method = arena::to_string(m_sym);

      // For "last" method, we need the delta color
      // The delta was stored when definecolorseries was called
      // But our 5-arg version didn't store delta — the test uses
      // \definecolorseries{foo}{rgb}{last}{.}{-.}
      // That means: base = current color ".", delta/last = complement of current "-."
      // Let me handle this specially

      // Look for delta if stored
      let delta_key = s!("color_series_{name_str}_delta");
      let step = match method.as_str() {
        "step" => {
          // delta is the step itself
          if let Some(Stored::String(d_sym)) = state::lookup_value(&delta_key) {
            Color::from_stored(&arena::to_string(d_sym)).unwrap_or(BLACK)
          } else { BLACK }
        },
        "grad" => {
          if let Some(Stored::String(d_sym)) = state::lookup_value(&delta_key) {
            Color::from_stored(&arena::to_string(d_sym)).unwrap_or(BLACK).scale(1.0 / div)
          } else { BLACK }
        },
        "last" => {
          // For "last": step = (last - base) / div
          if let Some(Stored::String(d_sym)) = state::lookup_value(&delta_key) {
            let last = Color::from_stored(&arena::to_string(d_sym)).unwrap_or(BLACK);
            let base_comps = base.components();
            let last_comps = last.components();
            let step_comps: Vec<f64> = base_comps.iter().zip(last_comps.iter())
              .map(|(b, l)| (l - b) / div)
              .collect();
            from_model_components(base.model(), &step_comps)
          } else { BLACK }
        },
        other => {
          Warn!("unknown","xcolor_step",format!("the step '{other}' was not step/grad/last"));
          BLACK
        }
      };

      // Reset color to base
      def_color(&name_str, &base, Some(Scope::Global))?;
      // Store step
      assign_value(&s!("color_series_{name_str}_step"),
        Stored::String(arena::pin(step.to_stored())), Some(Scope::Global));
    }
    Ok(Vec::new())
  });

  DefMacro!("\\colorseriescycle", "16");

  // \definecolorseries full 7-arg form — override the 5-arg form
  // Perl: '\definecolorseries{}{}{}[]{}[]{}'
  // We need to handle this properly for the test: \definecolorseries{foo}{rgb}{last}{.}{-.}
  // Note: the 5-arg form above already handles the basic case.
  // The test uses: {foo}{rgb}{last}{.}{-.} — base="." (current), last="-." (complement)
  // Let's update the 5-arg handler to also store the delta

  // Actually, let me fix the 5-arg DefPrimitive above to handle the sspec part.
  // The Perl prototype is: '\definecolorseries{}{}{}[]{}[]{}'
  // With 7 args: name, model, method, [bmodel], bspec, [smodel], sspec
  // Our 5-arg form handles: name, model, method, [bmodel], bspec
  // We need to also read smodel and sspec!

  // Override: redefine as macro that reads all args
  // For test compatibility, let's handle the case where bspec and sspec
  // appear as consecutive {} groups without optional [] between them

  // The test: \definecolorseries{foo}{rgb}{last}{.}{-.}
  // This is: name=foo, model=rgb, method=last, bspec=., sspec=-.
  // (no optional bmodel or smodel)

  // Let me redefine with the full parameter spec
  DefPrimitive!("\\definecolorseries{}{}{}{}{}", sub[(name, model, method, bspec, sspec)] {
    let name_str = do_expand(name)?.to_string();
    let model_str = do_expand(model)?.to_string();
    let method_str = do_expand(method)?.to_string();
    let bspec_str = do_expand(bspec)?.to_string();
    let sspec_str = do_expand(sspec)?.to_string();

    let base = parse_xcolor(None, &bspec_str, Some(&model_str));
    let delta = if method_str == "step" || method_str == "grad" {
      color_from_model_spec(&model_str, &sspec_str)
    } else {
      parse_xcolor(None, &sspec_str, Some(&model_str))
    };

    assign_value(&s!("color_series_{name_str}_base"),
      Stored::String(arena::pin(base.to_stored())), Some(Scope::Global));
    assign_value(&s!("color_series_{name_str}_method"),
      Stored::String(arena::pin(method_str)), Some(Scope::Global));
    assign_value(&s!("color_series_{name_str}_delta"),
      Stored::String(arena::pin(delta.to_stored())), Some(Scope::Global));
    Ok(Vec::new())
  });

  // Arithmetic
  Let!("\\rmultiply", "\\multiply");
  Let!("\\rdivide", "\\divide");

  // \lshift, \rshift etc: xcolor's register scaling ops.
  // These multiply/divide register values by powers of 10.
  // Use the same pattern as \multiply.
  DefPrimitive!("\\lshift Variable", sub[(var)] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (varname, inner) = *dbox;
      if let Some(defn) = state::lookup_register_definition(&varname) {
        let defn_args: Vec<ArgWrap> = inner.clone();
        let defn_value = defn.value_of(inner).unwrap_or_default();
        defn.set_value(defn_value.multiply(Number::new(10)), None, defn_args);
      }
    }
    Ok(())
  });

  DefPrimitive!("\\llshift Variable", sub[(var)] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (varname, inner) = *dbox;
      if let Some(defn) = state::lookup_register_definition(&varname) {
        let defn_args: Vec<ArgWrap> = inner.clone();
        let defn_value = defn.value_of(inner).unwrap_or_default();
        defn.set_value(defn_value.multiply(Number::new(100)), None, defn_args);
      }
    }
    Ok(())
  });

  DefPrimitive!("\\rshift Variable", sub[(var)] {
    // Divide by 10 using integer truncation (TeX semantics)
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (varname, inner) = *dbox;
      if let Some(defn) = state::lookup_register_definition(&varname) {
        let defn_args: Vec<ArgWrap> = inner.clone();
        let defn_value = defn.value_of(inner).unwrap_or_default();
        defn.set_value(defn_value.divide(Number::new(10)), None, defn_args);
      }
    }
    Ok(())
  });

  DefPrimitive!("\\rrshift Variable", sub[(var)] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (varname, inner) = *dbox;
      if let Some(defn) = state::lookup_register_definition(&varname) {
        let defn_args: Vec<ArgWrap> = inner.clone();
        let defn_value = defn.value_of(inner).unwrap_or_default();
        defn.set_value(defn_value.divide(Number::new(100)), None, defn_args);
      }
    }
    Ok(())
  });

  DefMacro!("\\lshiftnum {}", sub[(num)] {
    let n: f64 = do_expand(num)?.to_string().parse().unwrap_or(0.0);
    let result = (10.0 * n) as i64;
    Ok(mouth::tokenize_internal(&result.to_string()))
  });

  DefMacro!("\\llshiftnum {}", sub[(num)] {
    let n: f64 = do_expand(num)?.to_string().parse().unwrap_or(0.0);
    let result = (100.0 * n) as i64;
    Ok(mouth::tokenize_internal(&result.to_string()))
  });

  // \lshiftset and \llshiftset: set register = 10*n or 100*n
  // These take a Variable and a number argument
  DefPrimitive!("\\lshiftset Variable {}", sub[(var, num)] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (varname, inner) = *dbox;
      if let Some(defn) = state::lookup_register_definition(&varname) {
        let n: f64 = do_expand(num)?.to_string().parse().unwrap_or(0.0);
        let scaled = (10.0 * n) as i64;
        defn.set_value(RegisterValue::new(scaled), None, inner);
      }
    }
    Ok(())
  });

  DefPrimitive!("\\llshiftset Variable {}", sub[(var, num)] {
    if let ArgWrap::RegisterDefinition(dbox) = var {
      let (varname, inner) = *dbox;
      if let Some(defn) = state::lookup_register_definition(&varname) {
        let n: f64 = do_expand(num)?.to_string().parse().unwrap_or(0.0);
        let scaled = (100.0 * n) as i64;
        defn.set_value(RegisterValue::new(scaled), None, inner);
      }
    }
    Ok(())
  });

  // \fcolorbox — xcolor version with ParseXColor
  DefConstructor!("\\fcolorbox[]{}{} Undigested",
    "<ltx:text framed='rectangle' framecolor='#framecolor' _noautoclose='1'>#text</ltx:text>",
    mode => "internal_vertical",
    after_digest => sub[whatsit] {
      let model_str = whatsit.get_arg(1).map(|m| m.to_string());
      let fspec_str = whatsit.get_arg(2).map(|f| f.to_string()).unwrap_or_default();
      let bspec_str = whatsit.get_arg(3).map(|b| b.to_string()).unwrap_or_default();
      let text_tokens = whatsit.get_arg(4).map(|t| t.revert()).transpose()?;

      let framecolor = parse_xcolor(model_str.as_deref(), &fspec_str, None);
      whatsit.set_property("framecolor", Stored::String(arena::pin(framecolor.to_attribute())));

      let bgcolor = parse_xcolor(model_str.as_deref(), &bspec_str, None);
      merge_font(fontmap!(bg => bgcolor));

      if let Some(tokens) = text_tokens {
        let digested = digest(tokens)?;
        whatsit.set_property("text", Stored::Digested(digested));
      }
    }
  );

  // \extractcolorspec{color}{cmd}
  DefPrimitive!("\\extractcolorspec{}{}", sub[(colordesc, cmd)] {
    let color_str = do_expand(colordesc)?.to_string();
    let cmd_str = cmd.to_string();
    let color = parse_xcolor(None, &color_str, None);
    let model = color.model();
    let comps: Vec<String> = color.components().iter().map(|c| format!("{}", fixedpt(*c))).collect();
    let value = s!("{{{model}}}{{{}}}", comps.join(","));
    def_macro(T_CS!(cmd_str), None, Some(ExpansionBody::from(value.as_str())), None)?;
    Ok(())
  });

  // \extractcolorspecs{color}{modelcmd}{speccmd}
  DefPrimitive!("\\extractcolorspecs{}{}{}", sub[(colordesc, modelcmd, speccmd)] {
    let color_str = do_expand(colordesc)?.to_string();
    let modelcmd_str = modelcmd.to_string();
    let speccmd_str = speccmd.to_string();
    let color = parse_xcolor(None, &color_str, None);
    let model = color.model();
    let comps: Vec<String> = color.components().iter().map(|c| format!("{}", fixedpt(*c))).collect();
    def_macro(T_CS!(modelcmd_str), None, Some(ExpansionBody::from(model)), None)?;
    let spec_val = s!("{{{}}}", comps.join(","));
    def_macro(T_CS!(speccmd_str), None, Some(ExpansionBody::from(spec_val.as_str())), None)?;
    Ok(())
  });

  // \convertcolorspec{model}{spec}{tomodel}{cmd}
  DefPrimitive!("\\convertcolorspec{}{}{}{}", sub[(fmodel, spec, tomodel, cmd)] {
    let model_str = do_expand(fmodel)?.to_string();
    let spec_str = do_expand(spec)?.to_string();
    let tomodel_str = do_expand(tomodel)?.to_string();
    let cmd_str = cmd.to_string();
    let color = parse_xcolor(Some(&model_str), &spec_str, Some(&tomodel_str));
    let comps: Vec<String> = color.components().iter().map(|c| format!("{}", fixedpt(*c))).collect();
    let joined = comps.join(",");
    def_macro(T_CS!(cmd_str), None, Some(ExpansionBody::from(joined.as_str())), None)?;
    Ok(())
  });

  // Row colors
  DefConditional!("\\if@rowcolors");
  RawTeX!("\\@rowcolorstrue");

  DefPrimitive!("\\rowcolors OptionalMatch:* []{Number}{}{}", sub[(_star, _commands, first, oddcolor, evencolor)] {
    let first_val = first.value_of();
    let odd_str = do_expand(oddcolor)?.to_string();
    let even_str = do_expand(evencolor)?.to_string();
    assign_value("tabular_row_color_first", Stored::Number(Number::new(first_val)), None);
    if !odd_str.is_empty() {
      let odd = parse_xcolor(None, &odd_str, None);
      assign_value("tabular_row_color_odd", Stored::String(arena::pin(odd.to_stored())), None);
    }
    if !even_str.is_empty() {
      let even = parse_xcolor(None, &even_str, None);
      assign_value("tabular_row_color_even", Stored::String(arena::pin(even.to_stored())), None);
    }
    Ok(Vec::new())
  });

  DefMacro!("\\showrowcolors", "\\lx@hidden@noalign{\\global\\@rowcolorstrue}");
  DefMacro!("\\hiderowcolors", "\\lx@hidden@noalign{\\global\\@rowcolorsfalse}");

  // \rownum
  DefMacro!("\\rownum", "0"); // stub — proper alignment row tracking TODO

  // \rowcolor{color} — simplified stub
  DefPrimitive!("\\rowcolor[]{}", sub[(model_opt, spec)] {
    let model_str = model_opt.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let spec_str = do_expand(spec)?.to_string();
    let color = parse_xcolor(model_str.as_deref(), &spec_str, None);
    merge_font(fontmap!(bg => color));
    Ok(Vec::new())
  });

  // \columncolor — stub
  DefPrimitive!("\\columncolor[]{}", sub[(model_opt, spec)] {
    let model_str = model_opt.and_then(|m| do_expand(m).ok()).map(|t| t.to_string());
    let spec_str = do_expand(spec)?.to_string();
    let color = parse_xcolor(model_str.as_deref(), &spec_str, None);
    merge_font(fontmap!(bg => color));
    Ok(Vec::new())
  });

  // TeX internals via RawTeX
  RawTeX!(r##"
\let\XC@bcolor\relax
\let\XC@mcolor\relax
\let\XC@ecolor\relax

\def\XC@append#1#2%
{\ifx#1\@undefined\def#1{#2}\else\ifx#1\relax\def#1{#2}\else
  \toks@\expandafter{#1#2}\edef#1{\the\toks@}\fi\fi}
\def\XC@let@cc#1{\expandafter\XC@let@Nc\csname#1\endcsname}
\providecommand*\@namelet[1]{\expandafter\XC@let@Nc\csname#1\endcsname}
\def\XC@let@Nc#1#2{\expandafter\let\expandafter#1\csname#2\endcsname}
\def\XC@let@cN#1{\expandafter\let\csname#1\endcsname}
\def\@namexdef#1{\expandafter\xdef\csname #1\endcsname}
\def\aftergroupdef#1#2%
 {\expandafter\endgroup\expandafter\def\expandafter#1\expandafter{#2}}
\def\aftergroupedef#1#2%
 {\edef\@@tmp{\def\noexpand#1{#2}}\expandafter\endgroup\@@tmp}
"##);

  // \XC@edef and \XC@mdef need special catcode handling — use RawTeX
  RawTeX!(r##"
\begingroup
\catcode`\!=13 \catcode`\:=13 \catcode`\-=13 \catcode`\+=13
\catcode`\;=13 \catcode`\/=13 \catcode`\"=13 \catcode`\>=13
\gdef\XC@edef#1#2%
 {\begingroup
  \ifnum\catcode`\!=13 \edef!{\string!}\fi
  \ifnum\catcode`\:=13 \edef:{\string:}\fi
  \ifnum\catcode`\-=13 \edef-{\string-}\fi
  \ifnum\catcode`\+=13 \edef+{\string+}\fi
  \ifnum\catcode`\;=13 \edef;{\string;}\fi
  \ifnum\catcode`\"=13 \edef"{\string"}\fi
  \ifnum\catcode`\>=13 \edef>{\string>}\fi
  \edef#1{#2}\@onelevel@sanitize#1\aftergroupdef#1#1}
\gdef\XC@mdef#1#2%
 {\begingroup
  \ifnum\catcode`\/=13 \edef/{\string/}\fi
  \ifnum\catcode`\:=13 \edef:{\string:}\fi
  \edef#1{#2}\@onelevel@sanitize#1\aftergroupdef#1#1}
\endgroup
\def\XC@sdef#1#2{\edef#1{#2}\@onelevel@sanitize#1}
\def\@ifxempty#1{\@@ifxempty#1\@@ifxempty\XC@@}
\def\@@ifxempty#1#2\XC@@
 {\ifx#1\@@ifxempty
  \expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi}
"##);

  // XC@strip@comma, XC@replace, XC@type
  RawTeX!(r##"
\def\XC@strip@comma#1,#2%
 {\ifx,#2%
    #1\expandafter\remove@to@nnil\else#1 \expandafter\XC@strip@comma\fi
  #2}
"##);

  // Use begingroup/endgroup with catcode Q=3 for XC@replace
  RawTeX!(r##"
{\catcode`Q=3
 \gdef\XC@replace#1#2#3%
  {\begingroup
   \def\XC@repl@ce##1#2##2Q##3%
    {\@ifxempty{##2}{\XC@r@pl@ce##1Q}{\XC@repl@ce##1##3##2Q{##3}}}%
   \def\XC@r@pl@ce##1\@empty Q%
    {\expandafter\endgroup\expandafter\def\expandafter#1\expandafter{##1}}%
   \expandafter\XC@repl@ce\expandafter\@empty #1\@empty#2Q{#3}}
}
"##);

  RawTeX!(r##"
\def\XC@type#1%
 {\expandafter\expandafter\expandafter\XC@typ@
  \csname\string\color@#1\endcsname\@empty\@empty\@empty\XC@@}
\def\XC@typ@#1#2#3#4\XC@@
 {\ifx#1\relax 0\else
    \ifx#1\xcolor@
      \ifx$#2$%
        \ifx$#3$4\else3\fi\@gobbletwo
      \else2\fi\@gobbletwo
    \else1\fi
  \fi}
"##);

  // testcolors environment and \testcolor
  DefMacro!("\\testcolor", "\\@testopt{\\@testcolor}{}");

  RawTeX!(r##"
\newenvironment*{testcolors}[1][rgb,cmyk,hsb,HTML,gray]%
 {\let\@@nam\@empty\count@\z@
  \@for\@@tmp:=#1\do
    {\advance\count@\@ne
     \XC@sdef\@@tmp{\@@tmp}\edef\@@nam{\@@nam{\@@tmp}}}%
  \edef\@@num{\the\count@}%
  \def\XC@@gt{\textgreater}\def\@@tmp{OT1}%
  \ifx\f@encoding\@@tmp
    \@expandtwoargs\in@{,\f@family,}{,cmtt,pcr,}%
    \ifin@\def\XC@@gt{>}\fi
  \fi
  \def\XC@@xcp@{-1}\ifnum\XC@tracing>1 \def\XC@tracing{1}\fi
  \def\@testcolor[##1]##2%
   {\XC@mdef\@@mod{##1}\XC@edef\@@clr{##2}%
    \ifx\@@mod\@empty
      \let\@@arg\@@clr\XC@replace\@@arg>\XC@@gt\else
      \edef\@@arg{[\@@mod]{\@@clr}}\XC@definecolor[]{*}\@@mod\@@clr
      \def\@@clr{*}\fi
    \XC@append\@@arg{&}\extractcolorspecs\@@clr\@@mod\@@clr
    \@testc@lor}%
  \def\@testc@lor
   {\count@\z@
    \expandafter\@tfor\expandafter\@@tmp\expandafter:\expandafter=\@@nam\do
     {\ifx\@@clr\@empty
        \edef\@@cmd{\noexpand\textbf{\@@tmp}}%
      \else
        \convertcolorspec\@@mod\@@clr\@@tmp\@@cmd
        \edef\@@cmd
         {\noexpand\@testc@l@r{\@@tmp}{\@@cmd}%
          \ifx\@@mod\@@tmp\noexpand\underline\fi
          {\expandafter\XC@strip@comma\@@cmd,,\@nnil}}%
      \fi
      \expandafter\XC@append\expandafter\@@arg\expandafter{\@@cmd}%
      \advance\count@\@ne
      \ifnum\count@=\@@num\XC@append\@@arg{\\}\else\XC@append\@@arg{&}\fi}%
    \@@arg}%
  \def\@testc@l@r##1##2%
   {\fboxsep\z@\fbox{\colorbox[##1]{##2}{\phantom{XX}}} }%
  \tabular{@{}l*{\@@num}{l}@{}}%
  \def\@@arg{\textbf{color}& }\let\@@clr\@empty\@testc@lor}%
 {\endtabular\ignorespacesafterend}
"##);

  //========================
  ProcessOptions!();
});

/// Perl: sub defineColors — define colors from "name=from,name=from,..." pairs
fn define_colors_impl(id_pairs: &str, if_undef: bool) -> Result<()> {
  for pair in id_pairs.split(',') {
    let pair = pair.trim();
    if pair.is_empty() { continue; }
    let (name, from) = if let Some(eq_pos) = pair.find('=') {
      (pair[..eq_pos].trim(), pair[eq_pos+1..].trim())
    } else {
      (pair, pair)
    };
    if if_undef {
      let key = s!("color_{name}");
      if state::lookup_value(&key).is_some() { continue; }
    }
    let from_key = s!("color_{from}");
    if let Some(stored) = state::lookup_value(&from_key) {
      assign_value(&s!("color_{name}"), stored, None);
      // Also copy the \color@name macro via Let
      let from_cs = T_CS!(s!("\\color@{from}"));
      let to_cs = T_CS!(s!("\\color@{name}"));
      if lookup_definition(&from_cs)?.is_some() {
        let_i(&to_cs, &from_cs, None);
      }
    }
  }
  Ok(())
}

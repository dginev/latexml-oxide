use crate::prelude::*;

/// Convert a color component float (0.0-1.0) to u8 (0-255).
/// Matches Perl's `roundto($n * 255, 0)` which adds a small epsilon factor
/// to handle floating-point boundary cases (e.g., 1.0 - 0.90 = 0.0999... not 0.10).
fn color_component_to_u8(v: f64) -> u8 {
  // Perl: int($n * scale * (1 + 100*epsilon) + 0.5)
  // We use a small epsilon to nudge values that are very close to .5 boundaries
  let scaled = v.clamp(0.0, 1.0) * 255.0 * (1.0 + 100.0 * f64::EPSILON);
  scaled.round() as u8
}

/// Convert RGB float components (0.0-1.0) to hex color string like "#FF0000"
fn rgb_to_hex(r: f64, g: f64, b: f64) -> String {
  format!("#{:02X}{:02X}{:02X}", color_component_to_u8(r), color_component_to_u8(g), color_component_to_u8(b))
}

/// Convert CMYK float components (0.0-1.0) to hex via CMY→RGB
fn cmyk_to_hex(c: f64, m: f64, y: f64, k: f64) -> String {
  // cmyk → cmy: cmy_i = min(1, cmyk_i + k)
  let cc = (c + k).min(1.0);
  let cm = (m + k).min(1.0);
  let cy = (y + k).min(1.0);
  // cmy → rgb: rgb_i = 1 - cmy_i
  rgb_to_hex(1.0 - cc, 1.0 - cm, 1.0 - cy)
}

/// Convert gray float (0.0-1.0) to hex
fn gray_to_hex(g: f64) -> String {
  rgb_to_hex(g, g, g)
}

/// Parse a color specification, returning a hex color string.
/// Perl: ParseColor($model, $spec) in color.sty.ltxml
fn parse_color(model: Option<&str>, spec: &str) -> String {
  let spec = spec.trim().trim_matches(|c| c == '{' || c == '}').trim();

  if let Some(model) = model {
    let model_lc = model.to_lowercase();
    if model_lc == "named" {
      // Named color: look up "named_<spec>"
      return lookup_color(&format!("named_{spec}"));
    }
    // Parse components from spec
    let components: Vec<f64> = if spec.contains(',') {
      spec.split(',').filter_map(|s| s.trim().parse::<f64>().ok()).collect()
    } else {
      spec.split_whitespace().filter_map(|s| s.parse::<f64>().ok()).collect()
    };

    match model_lc.as_str() {
      "rgb" => {
        if components.len() >= 3 {
          rgb_to_hex(components[0], components[1], components[2])
        } else {
          "#000000".to_string()
        }
      },
      "cmyk" => {
        if components.len() >= 4 {
          cmyk_to_hex(components[0], components[1], components[2], components[3])
        } else {
          "#000000".to_string()
        }
      },
      "cmy" => {
        if components.len() >= 3 {
          rgb_to_hex(1.0 - components[0], 1.0 - components[1], 1.0 - components[2])
        } else {
          "#000000".to_string()
        }
      },
      "gray" => {
        if !components.is_empty() {
          gray_to_hex(components[0])
        } else {
          "#000000".to_string()
        }
      },
      "hsb" => {
        // HSB (HSV): h,s,b in [0,1]
        if components.len() >= 3 {
          let (h, s, b) = (components[0], components[1], components[2]);
          let (r, g, bl) = hsb_to_rgb(h, s, b);
          rgb_to_hex(r, g, bl)
        } else {
          "#000000".to_string()
        }
      },
      _ => {
        // Unknown model, try as named
        lookup_color(spec)
      },
    }
  } else {
    // No model — look up by name
    lookup_color(spec)
  }
}

/// HSB/HSV to RGB conversion. H,S,B all in [0,1]
fn hsb_to_rgb(h: f64, s: f64, b: f64) -> (f64, f64, f64) {
  if s == 0.0 {
    return (b, b, b);
  }
  let h6 = h * 6.0;
  let i = h6.floor() as i32;
  let f = h6 - i as f64;
  let p = b * (1.0 - s);
  let q = b * (1.0 - s * f);
  let t = b * (1.0 - s * (1.0 - f));
  match i % 6 {
    0 => (b, t, p),
    1 => (q, b, p),
    2 => (p, b, t),
    3 => (p, q, b),
    4 => (t, p, b),
    _ => (b, p, q),
  }
}

/// Look up a named color from state. Returns hex string.
/// Perl: LookupColor($name) in Package.pm
pub fn lookup_color(name: &str) -> String {
  let key = s!("color_{name}");
  match state::lookup_value(&key) {
    Some(Stored::String(sym)) => {
      // Copy the string out of the arena to avoid re-entrant borrow issues
      let stored_str = arena::with(sym, |s| s.to_string());
      let parts: Vec<&str> = stored_str.split_whitespace().collect();
      if parts.is_empty() {
        return "#000000".to_string();
      }
      let model = parts[0];
      let comps: Vec<f64> = parts[1..].iter()
        .filter_map(|p| p.parse::<f64>().ok()).collect();
      match model {
        "rgb" if comps.len() >= 3 => rgb_to_hex(comps[0], comps[1], comps[2]),
        "cmyk" if comps.len() >= 4 => cmyk_to_hex(comps[0], comps[1], comps[2], comps[3]),
        "cmy" if comps.len() >= 3 => rgb_to_hex(1.0 - comps[0], 1.0 - comps[1], 1.0 - comps[2]),
        "gray" if !comps.is_empty() => gray_to_hex(comps[0]),
        "named" if parts.len() >= 2 => lookup_color(&format!("named_{}", parts[1])),
        _ => "#000000".to_string(),
      }
    },
    _ => {
      // Color not found — default to black
      Info!("undefined", name, &s!("color '{}' is undefined...", name));
      "#000000".to_string()
    },
  }
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
  DefPrimitive!("\\definecolor{}{}{}", sub[(name, model, spec)] {
    let name_str = do_expand(name)?.to_string();
    let model_str = do_expand(model)?.to_string();
    let spec_str = do_expand(spec)?.to_string();
    // Store as color_<name> with "model components..." format for later lookup
    let spec_parts: Vec<&str> = if spec_str.contains(',') {
      spec_str.split(',').map(|s| s.trim()).collect()
    } else {
      spec_str.split_whitespace().collect()
    };
    let stored = format!("{} {}", model_str, spec_parts.join(" "));
    assign_value(&s!("color_{}", name_str), Stored::String(arena::pin(stored)), None);

    // Also define \color@<name> macro (Perl: DefColor)
    let macro_body = format!(
      "\\relax\\relax{{{} {}}}{{{}}}{{{}}}", model_str, spec_parts.join(" "),
      model_str, spec_parts.join(",")
    );
    def_macro(
      T_CS!(s!("\\\\color@{}", name_str)),
      None,
      Some(ExpansionBody::Tokens(Tokens::new(Explode!(macro_body)))),
      None,
    )?;

    // Return Box with revert
    Ok(Vec::new())
  });

  // \DefineNamedColor{dmodel}{name}{model}{spec}
  DefPrimitive!("\\DefineNamedColor{}{}{}{}", sub[(dmodel, name, model, spec)] {
    let name_str = do_expand(name)?.to_string();
    let model_str = do_expand(model)?.to_string();
    let spec_str = do_expand(spec)?.to_string();

    let spec_parts: Vec<&str> = if spec_str.contains(',') {
      spec_str.split(',').map(|s| s.trim()).collect()
    } else {
      spec_str.split_whitespace().collect()
    };
    let stored = format!("{} {}", model_str, spec_parts.join(" "));
    let named_key = format!("named_{}", name_str);
    assign_value(&s!("color_{}", named_key), Stored::String(arena::pin(stored)), None);

    let macro_body = format!(
      "\\relax\\relax{{{} {}}}{{{}}}{{{}}}", model_str, spec_parts.join(" "),
      model_str, spec_parts.join(",")
    );
    def_macro(
      T_CS!(s!("\\\\color@{}", named_key)),
      None,
      Some(ExpansionBody::Tokens(Tokens::new(Explode!(macro_body)))),
      None,
    )?;
    Ok(Vec::new())
  });

  // \color[model]{spec} or \color{name}
  DefPrimitive!("\\color[]{}", sub[(model_opt, spec)] {
    let model_str = model_opt.map(|m| do_expand(m).ok()).flatten().map(|t| t.to_string());
    let spec_str = do_expand(spec)?.to_string();
    let hex = parse_color(model_str.as_deref(), &spec_str);

    // If in preamble, store for \normalcolor
    if lookup_bool("inPreamble") {
      assign_value("preambleTextcolor", Stored::String(arena::pin(hex.clone())), None);
    }
    merge_font(fontmap!(color => hex));
    Ok(Vec::new())
  });

  // \pagecolor[model]{spec}
  DefPrimitive!("\\pagecolor[]{}", sub[(model_opt, spec)] {
    let model_str = model_opt.map(|m| do_expand(m).ok()).flatten().map(|t| t.to_string());
    let spec_str = do_expand(spec)?.to_string();
    let hex = parse_color(model_str.as_deref(), &spec_str);
    merge_font(fontmap!(bg => hex));
    Ok(Vec::new())
  });

  // \normalcolor — restores color from preamble
  DefPrimitive!("\\normalcolor", {
    let hex = match state::lookup_value("preambleTextcolor") {
      Some(Stored::String(sym)) => arena::with(sym, |s| s.to_string()),
      _ => "#000000".to_string(), // Black default
    };
    merge_font(fontmap!(color => hex));
  });

  // \textcolor[model]{spec}{text}
  DefMacro!("\\textcolor[]{}{}", "{\\ifx.#1.\\color{#2}\\else\\color[#1]{#2}\\fi#3}");

  // \colorbox[model]{spec}{text}
  DefMacro!("\\colorbox[]{}{}", "\\hbox{\\ifx.#1.\\pagecolor{#2}\\else\\pagecolor[#1]{#2}\\fi#3}");

  // \fcolorbox[model]{framespec}{bgspec}{text}
  DefConstructor!("\\fcolorbox[]{}{} Undigested",
    "<ltx:text framed='rectangle' framecolor='#framecolor' _noautoclose='1'>#text</ltx:text>",
    mode => "text",
    after_digest => sub[whatsit] {
      // Extract all values before mutating whatsit
      let model_str = whatsit.get_arg(1).map(|m| m.to_string());
      let fspec_str = whatsit.get_arg(2).map(|f| f.to_string()).unwrap_or_default();
      let bspec_str = whatsit.get_arg(3).map(|b| b.to_string()).unwrap_or_default();
      let text_tokens = whatsit.get_arg(4).map(|t| t.revert()).transpose()?;

      let framecolor = parse_color(model_str.as_deref(), &fspec_str);
      whatsit.set_property("framecolor", Stored::String(arena::pin(framecolor)));

      let bgcolor = parse_color(model_str.as_deref(), &bspec_str);
      merge_font(fontmap!(bg => bgcolor));

      if let Some(tokens) = text_tokens {
        let digested = digest(tokens)?;
        whatsit.set_property("text", Stored::Digested(digested.into()));
      }
    }
  );

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
  // Default defined colors
  TeX!(r#"\definecolor{black}{rgb}{0,0,0}
\definecolor{white}{rgb}{1,1,1}
\definecolor{red}{rgb}{1,0,0}
\definecolor{green}{rgb}{0,1,0}
\definecolor{blue}{rgb}{0,0,1}
\definecolor{cyan}{cmyk}{1,0,0,0}
\definecolor{magenta}{cmyk}{0,1,0,0}
\definecolor{yellow}{cmyk}{0,0,1,0}"#);

  //========================
  ProcessOptions!();
});

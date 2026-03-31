use crate::prelude::*;
use latexml_core::common::color::from_model_components;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L20-30: \DefineNamedColor — defines a named color + \text<name> + \<name>
  DefPrimitive!("\\DefineNamedColor{}{}{}{}", sub[(_dmodel, name, model, spec)] {
    let name_str = do_expand(name)?.to_string();
    let model_str = do_expand(model)?.to_string().trim().to_string();
    let spec_str = do_expand(spec)?.to_string();
    let spec_trimmed = spec_str.trim().trim_matches(|c| c == '{' || c == '}').trim().to_string();
    let model = if model_str.is_empty() { "cmyk".to_string() } else { model_str };
    let comps: Vec<f64> = if spec_trimmed.contains(',') {
      spec_trimmed.split(',').filter_map(|s| s.trim().parse().ok()).collect()
    } else {
      spec_trimmed.split_whitespace().filter_map(|s| s.parse().ok()).collect()
    };
    let color = from_model_components(&model, &comps);
    def_color(&name_str, &color, Some(Scope::Global))?;
    // Define \text<name> and \<name>{text} via RawTeX
    let text_def = s!(
      "\\expandafter\\def\\csname text{}\\endcsname{{\\color{{{}}}}}",
      name_str, name_str
    );
    let name_def = s!(
      "\\expandafter\\def\\csname {}\\endcsname#1{{{{\\csname text{}\\endcsname #1}}}}",
      name_str, name_str
    );
    for def_str in [&text_def, &name_def] {
      let tokens = mouth::tokenize_internal(def_str);
      gullet::do_expand(tokens)?;
    }
    Ok(Vec::new())
  });

  // Perl L34-37: \background — sets background color
  DefPrimitive!("\\background{}", sub[(color_arg)] {
    let color_str = do_expand(color_arg)?.to_string();
    let color = crate::package::color_sty::lookup_color_obj(&color_str);
    MergeFont!(bg => color);
    Ok(Vec::new())
  });

  DefMacro!("\\subdef{}", "");

  // Perl L42-53: \textColor — set color from CMYK spec
  DefPrimitive!("\\textColor{}", sub[(cmyk_arg)] {
    let cmyk_str = do_expand(cmyk_arg)?.to_string();
    let spec = cmyk_str.trim().trim_matches(|c| c == '{' || c == '}').trim().to_string();
    let comps: Vec<f64> = if spec.contains(',') {
      spec.split(',').filter_map(|s| s.trim().parse().ok()).collect()
    } else {
      spec.split_whitespace().filter_map(|s| s.parse().ok()).collect()
    };
    let color = from_model_components("cmyk", &comps);
    if lookup_bool("inPreamble") {
      assign_value("preambleTextcolor", Stored::String(arena::pin(color.to_stored())), None);
    }
    MergeFont!(color => color);
    Ok(Vec::new())
  });

  // Perl L56: \Color{CMYK}{text}
  DefMacro!("\\Color{}{}", "{\\textColor{#1} #2}");

  // Perl L61-63: \newColor — stub with warning
  DefPrimitive!("\\newColor{}", sub[(name)] {
    let name_str = name.to_string();
    Warn!("unexpected", "newColor", &s!("Ignoring definition of \\newColor {}", name_str));
    Ok(Vec::new())
  });

  // Perl L66: load DVI color definitions
  InputDefinitions!("dvipsnam", extension => Some(Cow::Borrowed("def")));
});

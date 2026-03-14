use crate::prelude::*;
use crate::package::color_sty::{lookup_color, lookup_color_obj};

LoadDefinitions!({
  // Perl: soul.sty.ltxml
  // Space-Out and UnderLine package

  // \sodef \cs {font}{letterspacing}{innerspace}{outerspace}
  // Perl: DefConstructorI($cs, '{}', "<ltx:text cssstyle='letter-spacing:#spacing;' ...>#1</ltx:text>",
  //   properties => { spacing => $letterspace->pxValue . 'px' }, bounded => 1,
  //   beforeDigest => sub { Digest($font); });
  // We store spacing + font per CS in state. The generic constructor reads them
  // via get_current_token() which preserves the original CS name through Let resolution.
  DefPrimitive!("\\sodef Token {} {Dimension}{Dimension}{Dimension}",
    sub[(cs, font, letterspace, _innerspace, _outerspace)] {
      let cs_name = cs.to_string();
      let px = letterspace.px_value(Some(2));
      let spacing_key = s!("soul_spacing_{cs_name}");
      assign_value(&spacing_key, s!("{px}px"), Some(Scope::Global));
      // Store the font tokens for beforeDigest
      let font_str = font.to_string();
      if !font_str.is_empty() {
        let font_key = s!("soul_font_{cs_name}");
        assign_value(&font_key, font_str, Some(Scope::Global));
      }
      // Define \cs as an alias for the generic letter-spacing constructor
      Let!(cs, T_CS!("\\lx@soul@letterspaced"));
    });

  // Generic letter-spacing constructor.
  // The after_digest reads the spacing from state keyed by the original CS name
  // (available via get_current_token(), which preserves the Let source token).
  DefConstructor!("\\lx@soul@letterspaced {}",
    "<ltx:text cssstyle='#cssstyle' _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true,
    bounded => true,
    before_digest => {
      // Look up and digest font tokens for the original CS
      if let Some(token) = get_current_token() {
        let cs_name = token.with_cs_name(|n| n.to_string());
        let font_key = s!("soul_font_{cs_name}");
        if let Some(Stored::String(font_sym)) = lookup_value(&font_key) {
          let font_str = arena::to_string(font_sym);
          let toks = latexml_core::mouth::tokenize_internal(&font_str);
          digest(toks)?;
        }
      }
    },
    after_digest => sub[whatsit] {
      // Look up the spacing for the original CS
      if let Some(token) = get_current_token() {
        let cs_name = token.with_cs_name(|n| n.to_string());
        let spacing_key = s!("soul_spacing_{cs_name}");
        if let Some(Stored::String(spacing_sym)) = lookup_value(&spacing_key) {
          let spacing = arena::to_string(spacing_sym);
          whatsit.set_property("cssstyle", s!("letter-spacing:{spacing};"));
        }
      }
      Ok(Vec::new())
    });

  RawTeX!("\\sodef\\textso{}{0.25em}{0.65em}{.55em}");
  RawTeX!("\\sodef\\sloppyword{}{0em}{.33em}{.33em}");

  DefMacro!("\\resetso", "\\sodef\\so{}{0.25em}{0.65em}{.55em}");

  // Small caps
  DefMacro!("\\capsfont", "\\scshape");
  RawTeX!("\\sodef\\textcaps{\\capsfont}{0.28em}{0.37em}{.37em}");

  // Ignorable caps customization
  DefMacro!("\\capsdef {} {Dimension}{Dimension}{Dimension}", None);
  DefMacro!("\\capssave{}", None);
  DefMacro!("\\capsselect{}", None);
  DefMacro!("\\capsreset", None);

  // Underline (with optional frame color from \setulcolor)
  DefConstructor!("\\textul{}",
    "<ltx:text framed='underline' framecolor='#framecolor' _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true,
    after_digest => sub[whatsit] {
      if let Some(Stored::String(color_sym)) = lookup_value("soul_ul_color") {
        let color_name = arena::to_string(color_sym);
        let hex = lookup_color(&color_name);
        whatsit.set_property("framecolor", hex);
      }
      Ok(Vec::new())
    });

  // Customizing underlines
  DefPrimitive!("\\setulcolor{}", sub[(color_arg)] {
    let color_str = color_arg.to_string();
    assign_value("soul_ul_color", color_str, Some(Scope::Global));
    Ok(())
  });
  DefMacro!("\\setul{Dimension}{Dimension}", None);
  DefMacro!("\\resetul", None);
  DefMacro!("\\setuldepth{}", None);
  DefMacro!("\\setuloverlap{Dimension}", None);

  // Strike-out (with optional strike color from \setstcolor)
  DefConstructor!("\\textst{}",
    "<ltx:text cssstyle='#cssstyle' _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true,
    after_digest => sub[whatsit] {
      let mut css = String::from("text-decoration:line-through;");
      if let Some(Stored::String(color_sym)) = lookup_value("soul_strike_color") {
        let color_name = arena::to_string(color_sym);
        let hex = lookup_color(&color_name);
        css.push_str(&s!("text-decoration-color:{hex};"));
      }
      whatsit.set_property("cssstyle", css);
      Ok(Vec::new())
    });

  // Customizing strikeout
  DefPrimitive!("\\setstcolor{}", sub[(color_arg)] {
    let color_str = color_arg.to_string();
    assign_value("soul_strike_color", color_str, Some(Scope::Global));
    Ok(())
  });

  // Highlighting — use background color (via MergeFont with bg)
  DefConstructor!("\\lx@texthl@color{}",
    "<ltx:text _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true,
    bounded => true,
    before_digest => {
      if let Some(Stored::String(color_sym)) = lookup_value("soul_hl_color") {
        let color_name = arena::to_string(color_sym);
        let color = lookup_color_obj(&color_name);
        merge_font(Font { bg: Some(color), ..Font::default() });
      }
    });
  DefMacro!("\\texthl", "\\@ifpackageloaded{color}{\\lx@texthl@color}{\\textul}");

  // Initialize default highlight color
  assign_value("soul_hl_color", "yellow", Some(Scope::Global));

  // Customizing highlight
  DefPrimitive!("\\sethlcolor{}", sub[(color_arg)] {
    let color_str = color_arg.to_string();
    assign_value("soul_hl_color", color_str, Some(Scope::Global));
    Ok(())
  });

  // Aliases — also copy sodef spacing/font state for the alias names
  // Helper to copy soul state from target to alias
  fn copy_soul_state(alias: &str, target: &str) {
    let spacing_key = s!("soul_spacing_{target}");
    if let Some(spacing) = lookup_value(&spacing_key) {
      assign_value(&s!("soul_spacing_{alias}"), spacing, Some(Scope::Global));
    }
    let font_key = s!("soul_font_{target}");
    if let Some(font_val) = lookup_value(&font_key) {
      assign_value(&s!("soul_font_{alias}"), font_val, Some(Scope::Global));
    }
  }
  Let!("\\so", "\\textso");
  copy_soul_state("\\so", "\\textso");
  Let!("\\caps", "\\textcaps");
  copy_soul_state("\\caps", "\\textcaps");
  Let!("\\ul", "\\textul");
  Let!("\\st", "\\textst");
  Let!("\\hl", "\\texthl");

  // Ignorable commands
  DefMacro!("\\soulomit{}", "#1");
  DefMacro!("\\soulaccent{}", None);
  DefMacro!("\\soulregister{}{}", None);
  Let!("\\soulfont", "\\soulregister");
});

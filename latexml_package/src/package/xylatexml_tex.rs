//! xylatexml.tex — LaTeXML SVG driver for xy-pic
//! Perl: xylatexml.tex.ltxml (1093 lines)
//!
//! Pragmatic port: registers the latexml driver, defines core SVG drawing
//! primitives. Many advanced features (curves, complex arrows) are stubs.
use crate::prelude::*;

/// Helper: convert dimension to px value
fn dim_to_px(d: Dimension) -> f64 {
  let dpi = state::lookup_int("DPI");
  let dpi = if dpi > 0 { dpi as f64 } else { 100.0 };
  (d.value_of() as f64 / 65536.0) * (dpi / 72.27)
}

/// Helper: read an xy register as dimension
fn xy_reg_dim(name: &str) -> Dimension {
  match state::lookup_register(name, Vec::new()) {
    Ok(Some(RegisterValue::Dimension(d))) => d,
    Ok(Some(RegisterValue::Number(n))) => Dimension::new(n.value_of()),
    _ => Dimension::new(0),
  }
}

/// Helper: read a macro's expansion as a string
fn macro_string(cs: &str) -> String {
  gullet::do_expand(T_CS!(cs)).map(|t| t.to_string()).unwrap_or_default()
}

/// Helper: insert an empty SVG element with attributes
fn svg_empty_element(document: &mut Document, tag: &str, attrs: HashMap<String, String>) -> Result<()> {
  document.open_element(tag, Some(attrs), None)?;
  document.close_element(tag)?;
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("color");

  // Line pattern management (Perl L82-90)
  DefPrimitive!("\\lx@xy@solidpat", { state::assign_value("xy_linepattern", Stored::None, None); });
  DefPrimitive!("\\lx@xy@dashpat", { state::assign_value("xy_linepattern", Stored::String(arena::pin("5")), None); });
  DefPrimitive!("\\lx@xy@dotpat", { state::assign_value("xy_linepattern", Stored::String(arena::pin("1 2")), None); });
  DefPrimitive!("\\lx@xy@cldashpat", { state::assign_value("xy_linepattern", Stored::String(arena::pin("5")), None); });
  DefPrimitive!("\\lx@xy@cldotpat", { state::assign_value("xy_linepattern", Stored::String(arena::pin("1 2")), None); });

  // Stroke/fill state (Perl L92-97)
  state::assign_value("xy_fill", false, None);
  state::assign_value("xy_stroke", true, None);
  DefPrimitive!("\\lx@xy@stroke@on", { state::assign_value("xy_stroke", true, None); });
  DefPrimitive!("\\lx@xy@stroke@off", { state::assign_value("xy_stroke", false, None); });
  DefPrimitive!("\\lx@xy@fill@on", { state::assign_value("xy_fill", true, None); });
  DefPrimitive!("\\lx@xy@fill@off", { state::assign_value("xy_fill", false, None); });

  // Color support (Perl L101-109)
  DefMacro!("\\xycolor@ {}", "");
  DefMacro!("\\xylocalColor@ {}{}", "");
  DefPrimitive!("\\lx@xy@usecolor {}{}", sub[(_spec, _model)] {});

  // Direction calculation (Perl L175-179)
  DefPrimitive!("\\lx@xy@calculate@direction", {
    let direction = match state::lookup_register("\\Direction", Vec::new()) {
      Ok(Some(RegisterValue::Number(n))) => n.value_of(),
      _ => 0,
    };
    let k = 1024i64;
    let kangle = ((direction + 8 * k) % (8 * k)) as f64;
    let q = kangle / 2.0 / k as f64;
    let quad = q as i32;
    let delta = 2.0 * (q - quad as f64) - 1.0;
    let norm = 1.0 / (1.0 + delta * delta).sqrt();
    let (dx, dy) = match quad {
      0 => (delta, -1.0),
      1 => (1.0, delta),
      2 => (-delta, 1.0),
      _ => (-1.0, -delta),
    };
    def_macro(T_CS!("\\cosDirection"), None, Tokenize!(&s!("{:.6}", dx * norm)), None)?;
    def_macro(T_CS!("\\sinDirection"), None, Tokenize!(&s!("{:.6}", dy * norm)), None)?;
  });

  // \lx@xy@move@to — position content at (x,y) in SVG (Perl L236-242)
  // Simplified: absorb content in an svg:g without precise positioning
  DefConstructor!("\\lx@xy@move@to {Dimension}{Dimension}{}",
    sub[document, args, _props] {
      document.open_element("svg:g", None, None)?;
      if let Some(Some(content)) = args.get(2) {
        document.absorb(content, None)?;
      }
      document.close_element("svg:g")?;
    }
  );

  // \zerodot — dot for dotted lines (Perl L262-269)
  DefConstructor!("\\zerodot",
    sub[document, _args, _props] {
      svg_empty_element(document, "svg:path", string_map!(
        "d" => "M -2 -1 l 1 1", "stroke" => "#000000", "fill" => "none"
      ))?;
    }
  );

  // \lx@xy@droprule — horizontal/vertical rule (Perl L280-293)
  DefConstructor!("\\lx@xy@droprule",
    sub[document, _args, _props] {
      let xp = dim_to_px(xy_reg_dim("\\X@p"));
      let yp = dim_to_px(xy_reg_dim("\\Y@p"));
      let xc = dim_to_px(xy_reg_dim("\\X@c"));
      let yc = dim_to_px(xy_reg_dim("\\Y@c"));
      let mut attrs = string_map!(
        "d" => s!("M {} {} L {} {}", xc, yc, xp, yp),
        "stroke" => "#000000",
        "fill" => "none"
      );
      let dashes = state::lookup_string("xy_linepattern");
      if !dashes.is_empty() { attrs.insert(String::from("stroke-dasharray"), dashes); }
      svg_empty_element(document, "svg:path", attrs)?;
    }
  );
  DefMacro!("\\Droprule@", "\\setboxz@h{\\lx@xy@droprule}\\advance\\X@p-\\X@c\\Drop@@");

  // \line@@ — line segment (Perl L296-316)
  DefConstructor!("\\line@@",
    sub[document, _args, _props] {
      let cos_v: f64 = macro_string("\\cosDirection").parse().unwrap_or(1.0);
      let sin_v: f64 = macro_string("\\sinDirection").parse().unwrap_or(0.0);
      let len = dim_to_px(xy_reg_dim("\\xydashl@"));
      let mut attrs = string_map!(
        "d" => s!("M 0 0 l {} {}", cos_v * len, sin_v * len),
        "stroke" => "#000000", "fill" => "none"
      );
      let dashes = state::lookup_string("xy_linepattern");
      if !dashes.is_empty() { attrs.insert(String::from("stroke-dasharray"), dashes); }
      svg_empty_element(document, "svg:path", attrs)?;
    }
  );

  // \lx@xy@drawline@ — connecting line (Perl L318-340)
  DefConstructor!("\\lx@xy@drawline@",
    sub[document, _args, _props] {
      let xp = dim_to_px(xy_reg_dim("\\X@p"));
      let yp = dim_to_px(xy_reg_dim("\\Y@p"));
      let xc = dim_to_px(xy_reg_dim("\\X@c"));
      let yc = dim_to_px(xy_reg_dim("\\Y@c"));
      let mut attrs = string_map!(
        "d" => s!("M {} {} L {} {}", xc, yc, xp, yp),
        "stroke" => "#000000", "fill" => "none"
      );
      let dashes = state::lookup_string("xy_linepattern");
      if !dashes.is_empty() { attrs.insert(String::from("stroke-dasharray"), dashes); }
      svg_empty_element(document, "svg:path", attrs)?;
    }
  );

  // Arrow tips (Perl L640-680)
  DefConstructor!("\\lx@xy@tip {}",
    sub[document, _args, _props] {
      let cos_v: f64 = macro_string("\\cosDirection").parse().unwrap_or(1.0);
      let sin_v: f64 = macro_string("\\sinDirection").parse().unwrap_or(0.0);
      let size = 4.0;
      let dx = cos_v * size; let dy = sin_v * size;
      let px = -sin_v * size * 0.4; let py = cos_v * size * 0.4;
      svg_empty_element(document, "svg:path", string_map!(
        "d" => s!("M 0 0 L {} {} L {} {} Z", -dx + px, -dy + py, -dx - px, -dy - py),
        "stroke" => "#000000", "fill" => "#000000"
      ))?;
    }
  );

  // Circle — simplified (Perl uses dimension from arg)
  DefConstructor!("\\lx@xy@circle {Dimension}",
    sub[document, _args, _props] {
      svg_empty_element(document, "svg:circle", string_map!(
        "cx" => "0", "cy" => "0", "r" => "5",
        "stroke" => "#000000", "fill" => "none"
      ))?;
    }
  );

  // Spline/curve stubs
  DefConstructor!("\\lx@xy@spline@",
    sub[document, _args, _props] {
      svg_empty_element(document, "svg:path", string_map!(
        "d" => "M 0 0", "stroke" => "#000000", "fill" => "none"
      ))?;
    }
  );
  DefConstructor!("\\lx@xy@drawsquiggles@",
    sub[document, _args, _props] {
      svg_empty_element(document, "svg:path", string_map!(
        "d" => "M 0 0", "stroke" => "#000000", "fill" => "none"
      ))?;
    }
  );

  // Straight typesetting override (Perl L187-229)
  DefConstructor!("\\lx@xy@straight@typeset",
    sub[document, _args, _props] {
      let xp = dim_to_px(xy_reg_dim("\\X@p"));
      let yp = dim_to_px(xy_reg_dim("\\Y@p"));
      let xc = dim_to_px(xy_reg_dim("\\X@c"));
      let yc = dim_to_px(xy_reg_dim("\\Y@c"));
      svg_empty_element(document, "svg:path", string_map!(
        "d" => s!("M {} {} L {} {}", xc, yc, xp, yp),
        "stroke" => "#000000", "fill" => "none"
      ))?;
    }
  );

  // Enable features — no-op messages (Perl L55-70)
  DefMacro!("\\lx@xy@latexmlon", "");
  DefMacro!("\\lx@xy@curveon", "");
  DefMacro!("\\lx@xy@frameon", "");
  DefMacro!("\\lx@xy@tipson", "");
  DefMacro!("\\lx@xy@lineon", "");
  DefMacro!("\\lx@xy@rotateon", "");
  DefMacro!("\\lx@xy@coloron", "");
  DefMacro!("\\lx@xy@crayonon", "");
  DefMacro!("\\lx@xy@matrixon", "");
  DefMacro!("\\lx@xy@arrowon", "");
  DefMacro!("\\lx@xy@graphon", "");
  DefMacro!("\\lx@xy@arcon", "");
  DefMacro!("\\lx@xy@polyon", "");
  DefMacro!("\\lx@xy@knoton", "");
  DefMacro!("\\lx@xy@tileon", "");
  DefMacro!("\\lx@xy@webon", "");
});

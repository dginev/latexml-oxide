use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // Perl: xy.sty.ltxml (57 lines) + xy.tex.ltxml (153 lines)
  //======================================================================

  // Register SVG document namespace for xy picture output
  model::register_document_namespace("svg", Some("http://www.w3.org/2000/svg"));

  // Step 1: Catcode management
  DefMacro!("\\xystycatcode", "12");

  // Step 2: Load raw xy.tex (Perl: at_letter => 0)
  assign_catcode('@', Catcode::OTHER, Some(Scope::Global));
  InputDefinitions!("xy", noltxml => true, extension => Some(Cow::Borrowed("tex")), at_letter => false);

  //======================================================================
  // Step 3: xy.tex.ltxml overlay (Perl L26-151)
  //======================================================================

  // Redefine \xyoption to filter incompatible drivers (Perl L27-50)
  Let!("\\lx@xy@xyoption@orig", "\\xyoption");
  DefMacro!("\\xyoption{}", sub[(option)] {
    let option_s = option.to_string();
    let other_drivers = [
      "16textures", "17oztex", "dvidrv", "dvips", "dvitops",
      "oztex", "pdf", "textures", "dvi",
    ];
    let unsupported = ["barr", "movie", "necula", "smart"];
    if other_drivers.contains(&option_s.as_str()) {
      Info!("xy", "ignored", s!("Ignoring xy driver {} (using latexml)", option_s));
      return Ok(Tokens!());
    }
    if unsupported.contains(&option_s.as_str()) {
      Warn!("xy", "unsupported",
        s!("The xy extension/feature {} may not be supported", option_s));
    }
    let cache_key = s!("loaded_xyoption_{}", option_s);
    if state::lookup_bool(&cache_key) {
      return Ok(Tokens!());
    }
    state::assign_value(&cache_key, true, Some(Scope::Global));
    Ok(Tokens!(T_CS!("\\lx@xy@xyoption@orig"), T_BEGIN!(), option, T_END!()))
  });

  // \xywarning@, \xyerror@ (Perl L53-54)
  DefPrimitive!("\\xywarning@ {}", sub[(msg)] {
    Info!("xy", "warning", msg.to_string());
  });
  DefPrimitive!("\\xyerror@ {}{}", sub[(msg, _detail)] {
    Info!("xy", "error", msg.to_string());
  });

  // Defer latexml driver to \AtBeginDocument (Perl L59)
  RawTeX!("\\AtBeginDocument{\\xyoption{latexml}}");

  // xy font primitives → no-op (Perl L66-72)
  DefMacro!("\\xydashfont", "");
  DefMacro!("\\xyatipfont", "");
  DefMacro!("\\xybtipfont", "");
  DefMacro!("\\xybsqlfont", "");
  DefMacro!("\\xycircfont", "");

  // \lx@xy@capturerange — capture coordinate range (Perl L143-146)
  DefPrimitive!("\\lx@xy@capturerange", {
    let mut dims = String::new();
    for reg in ["\\X@min", "\\Y@min", "\\X@max", "\\Y@max"] {
      if let Ok(Some(val)) = state::lookup_register(reg, Vec::new()) {
        let sp = match val {
          RegisterValue::Dimension(d) => d.value_of(),
          RegisterValue::Number(n) => n.value_of(),
          _ => 0,
        };
        if !dims.is_empty() { dims.push(','); }
        dims.push_str(&sp.to_string());
      } else {
        if !dims.is_empty() { dims.push(','); }
        dims.push('0');
      }
    }
    state::assign_value("saved_xy_range", Stored::String(arena::pin(&dims)), Some(Scope::Global));
  });

  // Helper: read saved_xy_range as pixel values
  // Returns (x0, y0, x1, y1) in px
  // fn xy_range_px() -> (f64, f64, f64, f64) — inline at use sites

  // \lx@xy@svg — top-level xy picture SVG (Perl L95-141)
  DefConstructor!("\\lx@xy@svg Digested",
    sub[document, args, _props] {
      let range_str = state::lookup_string("saved_xy_range");
      let dpi_val = state::lookup_int("DPI");
      let dpi = if dpi_val > 0 { dpi_val as f64 } else { 100.0 };
      let dims: Vec<f64> = range_str.split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .map(|v| (v as f64 / 65536.0) * (dpi / 72.27))
        .collect();
      let (x0, y0, x1, y1) = (
        dims.first().copied().unwrap_or(0.0),
        dims.get(1).copied().unwrap_or(0.0),
        dims.get(2).copied().unwrap_or(0.0),
        dims.get(3).copied().unwrap_or(0.0),
      );
      let mut w = x1 - x0;
      let mut h = y1 - y0;
      let (x0f, y0f) = if w < 0.0 { w = 0.0; (0.0f64, y0) } else { (x0, y0) };
      let y0f = if h < 0.0 { h = 0.0; 0.0 } else { y0f };
      let x = -x0f;
      let y = y1 - y0f;
      let minx = x;
      let miny = -y0f;
      let transform = s!("matrix(1 0 0 -1 {} {})", x, y);
      let pxwidth = if w > 1.0 { w } else { 1.0 };
      let pxheight = if h > 1.0 { h } else { 1.0 };

      let pic_attrs = string_map!(
        "width" => s!("{:.2}", pxwidth),
        "height" => s!("{:.2}", pxheight)
      );
      document.open_element("ltx:picture", Some(pic_attrs), None)?;

      let mut svg_attrs = string_map!(
        "version" => "1.1",
        "overflow" => "visible",
        "width" => s!("{:.2}", pxwidth),
        "height" => s!("{:.2}", pxheight),
        "viewBox" => s!("{:.2} {:.2} {:.2} {:.2}", minx, miny, pxwidth, pxheight)
      );
      if miny != 0.0 {
        svg_attrs.insert(String::from("style"), s!("vertical-align:{}px", -miny));
      }
      document.open_element("svg:svg", Some(svg_attrs), None)?;

      let g_attrs = string_map!("transform" => transform);
      document.open_element("svg:g", Some(g_attrs), None)?;

      if let Some(Some(content)) = args.first() {
        document.absorb(content, None)?;
      }

      document.close_element("svg:g")?;
      document.close_element("svg:svg")?;
      document.close_element("ltx:picture")?;
    }
  );

  // \lx@xy@svgnested — nested xy picture (Perl L79-93)
  DefConstructor!("\\lx@xy@svgnested Digested",
    sub[document, args, _props] {
      let range_str = state::lookup_string("saved_xy_range");
      let dpi_val = state::lookup_int("DPI");
      let dpi = if dpi_val > 0 { dpi_val as f64 } else { 100.0 };
      let dims: Vec<f64> = range_str.split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .map(|v| (v as f64 / 65536.0) * (dpi / 72.27))
        .collect();
      let (x0, y0, _x1, _y1) = (
        dims.first().copied().unwrap_or(0.0),
        dims.get(1).copied().unwrap_or(0.0),
        dims.get(2).copied().unwrap_or(0.0),
        dims.get(3).copied().unwrap_or(0.0),
      );
      let h = _y1 - y0;
      let x_px = if x0 > 0.0 { x0 } else { 0.0 };
      let y_px = if state::lookup_bool("IN_MATH") { -(h / 2.0) } else { 0.0 };
      let transform = s!("matrix(1 0 0 1 {} {})", x_px, y_px);

      let g_attrs = string_map!("transform" => transform);
      document.open_element("svg:g", Some(g_attrs), None)?;
      if let Some(Some(content)) = args.first() {
        document.absorb(content, None)?;
      }
      document.close_element("svg:g")?;
    }
  );

  // Save original \xy/\endxy, redefine to wrap with SVG (Perl L148-151)
  Let!("\\lx@xy@original", "\\xy");
  Let!("\\end@lx@xy@original", "\\endxy");
  DefMacro!("\\xy", "\\if\\inxy@\\lx@xy@svgnested\\else\\lx@xy@svg\\fi\\lx@xy@original");
  DefMacro!("\\endxy", "\\relax\\lx@xy@capturerange\\end@lx@xy@original");

  //======================================================================
  // Step 4
  RequirePackage!("ifpdf");

  //======================================================================
  // Step 5: DeclareOption (Perl xy.sty.ltxml L27-53)
  DeclareOption!("cmactex", "\\xyoption{dvips}");
  DeclareOption!("dvips", "\\xyoption{dvips}");
  DeclareOption!("dvitops", "\\xyoption{dvitops}");
  DeclareOption!("emtex", "\\xyoption{emtex}");
  DeclareOption!("ln", "\\xyoption{ln}");
  DeclareOption!("oztex", "\\xyoption{oztex}");
  DeclareOption!("textures", "\\xyoption{textures}");
  DeclareOption!("xdvi", "\\xyoption{xdvi}");
  DeclareOption!("pdftex", "\\xyoption{pdf}");
  DeclareOption!("dvipdfm", "\\xyoption{pdf}");
  DeclareOption!("dvipdfmx", "\\xyoption{pdf}");
  DeclareOption!("colour", "\\xyoption{color}");
  DeclareOption!("cmtip", "\\xyoption{cmtip}\\UseComputerModernTips");
  DeclareOption!("10pt", "\\xywithoption{tips}{\\def\\tipsize@@{10}}");
  DeclareOption!("11pt", "\\xywithoption{tips}{\\def\\tipsize@@{11}}");
  DeclareOption!("12pt", "\\xywithoption{tips}{\\def\\tipsize@@{12}}");

  // Catch-all: DeclareOption(undef, ...)
  DeclareOption!(None, {
    let current_option = gullet::do_expand(T_CS!("\\CurrentOption"))?.to_string();
    gullet::unread(Tokenize!(&s!("\\xyoption{{{}}}", current_option)));
  });

  //======================================================================
  // Step 6
  ProcessOptions!();
});

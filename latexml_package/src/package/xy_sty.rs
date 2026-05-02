use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // Perl: xy.sty.ltxml (57 lines) + xy.tex.ltxml (153 lines)
  //======================================================================

  // Register SVG document namespace for xy picture output
  model::register_document_namespace("svg", Some("http://www.w3.org/2000/svg"));

  // Step 1: Catcode management
  // Perl xy.sty.ltxml L19:
  //   DefMacro('\xystycatcode', sub { Explode(LookupCatcode('@')); });
  // Dynamically expands to the digit chars of `@`'s current catcode.
  DefMacro!("\\xystycatcode", sub[_args] {
    let cc = state::lookup_catcode('@')
      .map(|cc| cc as u8).unwrap_or(12);
    Tokens!(Explode!(s!("{}", cc)))
  });

  // Step 2: Load raw xy.tex (Perl: at_letter => 0).
  // Perl xy.sty.ltxml L21 explicitly sets `\catcode`\@=12` before
  // InputDefinitions('xy', type => 'tex'). xy.tex's L47 \xyreuncatcodes
  // captures `@`'s catcode so that subsequent invocations of \xyuncatcodes
  // (e.g. inside `\xyxy@@ix@`'s `\begingroup ... \xyuncatcodes ...
  // \afterassignment\endgroup\global\toks9=` sequence used during
  // `\CompileMatrices` re-input of `.xyc` files) restore the same value.
  // Without this pre-assign, our binding-load path enters xy.tex with
  // `@` = LETTER (from ambient binding-load context), so \xyuncatcodes
  // ends up restoring `@` to LETTER — but the .xyc compile body relies
  // on `\xycatcodes` having already set `@` = LETTER, and the local
  // group-pop expectation breaks.
  let saved_currname = gullet::do_expand(T_CS!("\\@currname")).ok().map(|t| t.to_string());
  let saved_currext = gullet::do_expand(T_CS!("\\@currext")).ok().map(|t| t.to_string());
  let saved_at_cc_before_xy = state::lookup_catcode('@');
  state::assign_catcode('@', Catcode::OTHER, Some(Scope::Global));
  InputDefinitions!("xy", noltxml => true, extension => Some(Cow::Borrowed("tex")), at_letter => false);
  if let Some(cc) = saved_at_cc_before_xy {
    state::assign_catcode('@', cc, Some(Scope::Global));
  }
  // Restore \@currname/\@currext so ProcessOptions uses the correct package name
  if let Some(ref name) = saved_currname {
    def_macro(T_CS!("\\@currname"), None, Tokenize!(name), None)?;
  }
  if let Some(ref ext) = saved_currext {
    def_macro(T_CS!("\\@currext"), None, Tokenize!(ext), None)?;
  }

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
    // Special case: "latexml" driver — load our Rust xylatexml overlay directly
    // since the file xylatexml.tex doesn't exist on disk (it's compiled into the binary).
    if option_s == "latexml" {
      crate::package::xylatexml_tex::load_definitions()?;
      return Ok(Tokens!());
    }
    Ok(Tokens!(T_CS!("\\lx@xy@xyoption@orig"), T_BEGIN!(), option, T_END!()))
  });

  // \xywarning@, \xyerror@ (Perl L53-54)
  DefPrimitive!("\\xywarning@ {}", sub[(msg)] {
    Info!("xy", "warning", msg.to_string());
  });
  DefPrimitive!("\\xyerror@ {}{}", sub[(msg, _detail)] {
    Info!("xy", "error", msg.to_string());
  });

  // Perl L59: defers \xyoption{latexml} to \AtBeginDocument.
  // The @ catcode fix (removing assign_catcode('@', OTHER) before xy.tex) means
  // xyline.tex's \xydef@ definitions now work correctly. The early
  // xylatexml_tex::load_definitions() call was removed because it triggers
  // \xyprovide{latexml} and \newdriver{...} from xylatexml_tex's raw_tex block,
  // which sets up xy's driver mechanism incorrectly (our Rust override for
  // \xyoption{latexml} never sets \csname xylatexml loaded\endcsname, so
  // \xywithoption{latexml}{...} defers \selectdriver@{latexml} indefinitely,
  // causing massive token expansion during ProcessOptions). Fix: keep only
  // \AtBeginDocument for the full initialization, matching the Perl flow.
  at_begin_document(TokenizeInternal!(r"\xyoption{latexml}"))?;

  // xy font primitives: do NOT pre-define these as empty macros!
  // xy.tex's \xyfont@ mechanism checks \ifx#1\undefined and only loads the font
  // if it's undefined. Pre-defining them as empty macros makes the check fail,
  // leaving them as expandable macros (not font tokens). This breaks \fontdimen
  // usage because the number scanner expands them to empty and reads past.

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
  // Perl uses a `properties` callback (digestion time) to capture saved_xy_range,
  // then the constructor body (construction time) reads from %props.
  // We replicate this with after_digest to snapshot the range at digest time.
  DefConstructor!("\\lx@xy@svg Digested",
    sub[document, args, props] {
      // All values pre-computed at digest time via properties callback
      let get_s = |k: &str| -> String {
        match props.get(k) { Some(Stored::String(s)) => arena::to_string(*s), _ => String::new() }
      };
      let pxwidth = get_s("pxwidth");
      let pxheight = get_s("pxheight");
      let pic_attrs = string_map!("width" => pxwidth.clone(), "height" => pxheight.clone());
      document.open_element("ltx:picture", Some(pic_attrs), None)?;

      let mut svg_attrs = string_map!(
        "version" => "1.1", "overflow" => "visible",
        "width" => pxwidth.clone(), "height" => pxheight.clone(),
        "viewBox" => get_s("viewBox")
      );
      let style = get_s("style");
      if !style.is_empty() { svg_attrs.insert(String::from("style"), style); }
      document.open_element("svg:svg", Some(svg_attrs), None)?;

      let g_attrs = string_map!("transform" => get_s("transform"));
      document.open_element("svg:g", Some(g_attrs), None)?;

      if let Some(Some(content)) = args.first() {
        document.absorb(content, None)?;
      }
      document.close_element("svg:g")?;
      document.close_element("svg:svg")?;
      document.close_element("ltx:picture")?;
    },
    // Perl: properties => sub { ... } — capture AND compute all values at digest time
    after_digest => sub[whatsit] {
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
      let pxwidth = if w > 1.0 { w } else { 1.0 };
      let pxheight = if h > 1.0 { h } else { 1.0 };
      whatsit.set_property("pxwidth", Stored::from(s!("{:.2}", pxwidth)));
      whatsit.set_property("pxheight", Stored::from(s!("{:.2}", pxheight)));
      whatsit.set_property("viewBox", Stored::from(
        s!("{:.2} {:.2} {:.2} {:.2}", minx, miny, pxwidth, pxheight)));
      whatsit.set_property("transform", Stored::from(
        s!("matrix(1 0 0 -1 {} {})", x, y)));
      if miny != 0.0 {
        whatsit.set_property("style", Stored::from(
          s!("vertical-align:{}px", -miny)));
      }
    }
  );

  // \lx@xy@svgnested — nested xy picture (Perl L79-93)
  DefConstructor!("\\lx@xy@svgnested Digested",
    sub[document, args, props] {
      let transform = match props.get("transform") {
        Some(Stored::String(s)) => arena::to_string(*s),
        _ => String::from("matrix(1 0 0 1 0 0)"),
      };
      let g_attrs = string_map!("transform" => transform);
      document.open_element("svg:g", Some(g_attrs), None)?;
      if let Some(Some(content)) = args.first() {
        document.absorb(content, None)?;
      }
      document.close_element("svg:g")?;
    },
    after_digest => sub[whatsit] {
      let range_str = state::lookup_string("saved_xy_range");
      let dpi_val = state::lookup_int("DPI");
      let dpi = if dpi_val > 0 { dpi_val as f64 } else { 100.0 };
      let dims: Vec<f64> = range_str.split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .map(|v| (v as f64 / 65536.0) * (dpi / 72.27))
        .collect();
      let (x0, y0, _x1, y1) = (
        dims.first().copied().unwrap_or(0.0),
        dims.get(1).copied().unwrap_or(0.0),
        dims.get(2).copied().unwrap_or(0.0),
        dims.get(3).copied().unwrap_or(0.0),
      );
      let h = y1 - y0;
      let x_px = if x0 > 0.0 { x0 } else { 0.0 };
      let in_math = state::lookup_bool_sym(pin!("IN_MATH"));
      let y_px = if in_math { -(h / 2.0) } else { 0.0 };
      whatsit.set_property("transform", Stored::from(
        s!("matrix(1 0 0 1 {} {})", x_px, y_px)));
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
  // Pre-load amstext to provide `\text{...}` for in-math text. Perl's
  // xy chain ends up with `\text` defined via xyv2.tex's
  // `\def\text{\relax\textC}` (raw-loaded by Perl's xypic.tex.ltxml
  // chain). Rust's xy.tex raw-load doesn't reach xyv2.tex with the
  // same definition, leaving `\text` undefined for papers that use
  // it inside math (witness: math0211451 `\text{deg}` in math).
  // amstext provides a clean DefConstructor path. This is a
  // pragmatic Rust-side bridge; not a Perl divergence since amstext
  // is part of the standard math chain Perl effectively reaches.
  RequirePackage!("amstext");

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
  // Perl: DeclareOption(undef, '\edef\next{\noexpand\xyoption{\CurrentOption}}\next');
  DeclareOption!(None, {
    let current_option = gullet::do_expand(T_CS!("\\CurrentOption"))?.to_string();
    gullet::unread(Tokenize!(&s!("\\xyoption{{{}}}", current_option)));
  });

  //======================================================================
  // Step 6
  ProcessOptions!();

  //======================================================================
  // Step 7: Defensive stub for `\xymatrix` when bare `\usepackage{xy}`
  // (no [matrix,arrow] options) leaves `xymatrix.tex` unloaded. Without
  // a stub, papers using `\xymatrix@1{a & b \\ c & d}` (e.g.
  // arXiv:1409.7007) cascade 100+ "Stray alignment '&'" errors as the
  // unguarded body is parsed in math mode. The stub uses TeX's `#{`
  // implicit-brace-stop pattern to read xy-pic's `@<modifier>` prefix
  // tokens (`@1`, `@C=10pt`, `@R=2cm`, etc.) up to the brace, then
  // swallows the brace-balanced body. The diagram is dropped — but
  // the cascade is gone, matching latexml's general philosophy of
  // "drop unsupported package content rather than fail conversion".
  //
  // Gated by `\@ifundefined{xymatrix}` so it only fires when xy.tex's
  // option pipeline didn't auto-load matrix/arrow features.
  RawTeX!(r"\@ifundefined{xymatrix}{%
    \def\xymatrix#1#{\lx@xy@stub@xymatrix@body}%
    \def\lx@xy@stub@xymatrix@body#1{}%
  }{}");
});

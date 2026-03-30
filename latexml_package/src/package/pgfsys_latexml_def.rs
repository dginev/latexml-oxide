//! pgfsys-latexml.def — LaTeXML SVG driver for pgf/tikz
//! Perl: pgfsys-latexml.def.ltxml (1022 lines)
//!
//! Port of the SVG drawing primitives that make pgf/tikz produce SVG output.
use crate::prelude::*;

/// Helper: convert dimension to px value, rounded to 2 decimal places
/// Perl: $dim->pxValue  =>  roundto(sp / UNITY * DPI / 72.27, 2)
fn dim_to_px(d: Dimension) -> f64 {
  let dpi = state::lookup_int("DPI");
  let dpi = if dpi > 0 { dpi as f64 } else { 100.0 };
  let raw = (d.value_of() as f64 / 65536.0) * (dpi / 72.27);
  // Perl roundto(n, 2): round to 2 decimal places
  (raw * 100.0).round() / 100.0
}

/// Perl: sub SVGNextObject — global counter for clip paths etc.
fn svg_next_object() -> i64 {
  let n = state::lookup_int("svg_objcount") + 1;
  state::assign_value("svg_objcount", Stored::Int(n), Scope::Global);
  n
}

/// Perl: sub addToSVGPath — accumulates path data in state
fn add_to_svg_path(operation: &str, points: &[Dimension]) {
  let new_path = if points.is_empty() {
    operation.to_string()
  } else {
    let pts: Vec<String> = points.iter().map(|p| format!("{}", dim_to_px(*p))).collect();
    format!("{} {}", operation, pts.join(" "))
  };
  let current = state::lookup_string("pgf_SVGpath");
  let combined = if current.is_empty() {
    new_path
  } else {
    format!("{} {}", current, new_path)
  };
  state::assign_value("pgf_SVGpath", Stored::String(arena::pin(&combined)), Scope::Global);
}

/// Look up a pgf register as a Dimension
fn pgf_reg_dim(name: &str) -> Dimension {
  match state::lookup_register(name, Vec::new()) {
    Ok(Some(RegisterValue::Dimension(d))) => d,
    Ok(Some(RegisterValue::Number(n))) => Dimension::new(n.value_of()),
    _ => Dimension::new(0),
  }
}

/// Helper: format color channel (0.0–1.0) to hex
fn color_to_hex(r: f64, g: f64, b: f64) -> String {
  format!("#{:02X}{:02X}{:02X}",
    (r * 255.0).round().min(255.0).max(0.0) as u8,
    (g * 255.0).round().min(255.0).max(0.0) as u8,
    (b * 255.0).round().min(255.0).max(0.0) as u8)
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl L29-31: image suffix list
  DefMacro!("\\pgfsys@imagesuffixlist",
    ".svg:.png:.gif:.jpg:.jpeg:.eps:.ps:.postscript:.ai:.pdf:");

  //===================================================================
  // 0. Environment specific stuff
  //===================================================================

  // Perl L59: \lxSVG@installcommands
  DefPrimitive!("\\lxSVG@installcommands", {
    state::let_i(&T_CS!("\\halign"), &T_CS!("\\lxSVG@halign"), None);
    state::let_i(&T_CS!("\\ignorespaces"), &T_CS!("\\lx@inpgf@ignorespaces"), None);
  });

  // Perl L63: redefine ignorespaces inside PGF context
  DefPrimitive!("\\lx@inpgf@ignorespaces SkipSpaces", None);

  // Perl L65-69: \lxSVG@picture — wraps pgfpicture with SVG setup
  DefMacro!("\\lxSVG@picture", sub[_gullet] {
    vec![
      T_CS!("\\lxSVG@clearpath"),
      T_CS!("\\begingroup"),
      T_CS!("\\lxSVG@installcommands"),
    ]
  });
  DefMacro!("\\endlxSVG@picture", "\\endgroup");

  // Perl L75-147: \lxSVG@insertpicture — creates <ltx:picture> + <svg:svg>
  DefConstructor!("\\lxSVG@insertpicture{}",
    sub[document, _args, props] {
      let current = document.get_node().clone();
      let current_qname = document::get_node_qname(&current);
      let current_qname_str = arena::to_string(current_qname);
      if current_qname_str.starts_with("svg:") {
        // Already in SVG — just open a nested svg:g
        let minx = match props.get("minx") {
          Some(Stored::Float(f)) => f.0, _ => 0.0
        };
        let miny = match props.get("miny") {
          Some(Stored::Float(f)) => f.0, _ => 0.0
        };
        let transform = format!("matrix(1 0 0 1 {} {})", -minx, -miny);
        document.open_element("svg:g", Some(string_map!(
          "transform" => transform,
          "_scopebegin" => "1".to_string(),
          "class" => "ltx_nestedsvg".to_string()
        )), None)?;
        if let Some(Stored::Digested(content)) = props.get("content_box") {
          document.absorb(content, None)?;
        }
        document.close_element("svg:g")?;
      } else {
        // Not in SVG — create full picture + svg:svg wrapper
        let pxwidth = match props.get("pxwidth") {
          Some(Stored::Float(f)) => f.0, _ => 1.0
        };
        let pxheight = match props.get("pxheight") {
          Some(Stored::Float(f)) => f.0, _ => 1.0
        };

        // Perl: $document->openElement('ltx:picture');
        // height/width attributes come from pxValue of the picture dimensions
        let pic_attrs = string_map!(
          "height" => format!("{}", pxheight),
          "width" => format!("{}", pxwidth)
        );
        document.open_element("ltx:picture", Some(pic_attrs), None)?;

        let minx = match props.get("minx") {
          Some(Stored::Float(f)) => f.0, _ => 0.0
        };
        let miny = match props.get("miny") {
          Some(Stored::Float(f)) => f.0, _ => 0.0
        };
        // Perl L88: viewBox => "$props{minx} $props{miny} $props{pxwidth} $props{pxheight}"
        let svg_attrs = string_map!(
          "version" => "1.1".to_string(),
          "width" => format!("{}", pxwidth),
          "height" => format!("{}", pxheight),
          "viewBox" => format!("{} {} {} {}", minx, miny, pxwidth, pxheight),
          "overflow" => "visible".to_string()
        );
        document.open_element("svg:svg", Some(svg_attrs), None)?;
        // Perl L89: style => $props{style} (baseline vertical-align)
        if let Some(Stored::String(style)) = props.get("style") {
          let s = arena::to_string(*style);
          if !s.is_empty() {
            let svg_node = document.get_node_mut();
            let _ = svg_node.set_attribute("style", &s);
          }
        }

        // Perl L91-92: x0=-(0+minx), y0=pxheight+miny
        let x0 = -(0.0 + minx);
        let y0 = pxheight + miny;
        // Avoid -0 in output
        let x0 = if x0 == 0.0 { 0.0 } else { x0 };
        let y0 = if y0 == 0.0 { 0.0 } else { y0 };
        let transform = format!("translate({},{}) matrix(1 0 0 -1 0 0)", x0, y0);
        document.open_element("svg:g", Some(string_map!(
          "transform" => transform,
          "_scopebegin" => "1".to_string()
        )), None)?;

        if let Some(Stored::Digested(content)) = props.get("content_box") {
          document.absorb(content, None)?;
        }

        // Close all svg:g's
        while let Ok(Some(_)) = document.maybe_close_element("svg:g") {}
        document.close_element("svg:svg")?;
        document.close_element("ltx:picture")?;
      }
    },
    reversion => sub[whatsit, _args] {
      // Perl L140-147: reversion produces \hbox to<W>pt{\vbox to<H>pt{...}}
      let w = whatsit.with_properties(|props| {
        match props.get("width_dim") {
          Some(Stored::Dimension(d)) => d.pt_value(None),
          _ => 0.0
        }
      });
      let h = whatsit.with_properties(|props| {
        match props.get("height_dim") {
          Some(Stored::Dimension(d)) => d.pt_value(None),
          _ => 0.0
        }
      });
      let mut toks: Vec<Token> = vec![T_CS!("\\hbox"), T_SPACE!()];
      toks.extend(Explode!(format!("to{}pt", w)));
      toks.push(T_BEGIN!());
      toks.extend(vec![T_CS!("\\vbox"), T_SPACE!()]);
      toks.extend(Explode!(format!("to{}pt", h)));
      toks.push(T_BEGIN!());
      toks.push(T_CS!("\\pgfpicture"));
      toks.push(T_CS!("\\makeatletter"));
      if let Some(arg) = whatsit.get_arg(1) {
        toks.extend(arg.revert()?.unlist());
      }
      toks.push(T_CS!("\\endpgfpicture"));
      toks.push(T_END!());
      toks.push(T_END!());
      Ok(Tokens::new(toks))
    },
    after_digest => sub[whatsit] {
      // Perl L106-138: read pgf registers to compute picture dimensions
      // \pgf@picmaxx,\pgf@picmaxy are now the SIZE of the picture (adjusted by typesetpicturebox)
      let miny = pgf_reg_dim("\\pgf@picminy");
      let width = pgf_reg_dim("\\pgf@picmaxx");
      let height = pgf_reg_dim("\\pgf@picmaxy");

      let w = dim_to_px(width).max(1.0);
      let h = dim_to_px(height).max(1.0);

      // Perl L120-121: baseline shift from \pgf@shift@baseline
      let shift_tokens = Expand!(T_CS!("\\pgf@shift@baseline"));
      let shift_str = shift_tokens.to_string();
      let shift_str = shift_str.trim();
      let base = if !shift_str.is_empty() && shift_str != "0pt" {
        if let Ok(shift_dim) = Dimension::from_str(shift_str) {
          // Perl: $base = ($shift ? $miny->subtract(Dimension($shift))->pxValue : 0)
          let base_dim = Dimension::new(miny.value_of() - shift_dim.value_of());
          dim_to_px(base_dim)
        } else { 0.0 }
      } else { 0.0 };

      whatsit.set_property("minx", Stored::Float(Float::new_f64(0.0)));
      whatsit.set_property("miny", Stored::Float(Float::new_f64(0.0)));
      whatsit.set_property("width", Stored::Dimension(width));
      whatsit.set_property("height", Stored::Dimension(height));
      whatsit.set_property("depth", Stored::Dimension(Dimension::new(0)));
      whatsit.set_property("pxwidth", Stored::Float(Float::new_f64(w)));
      whatsit.set_property("pxheight", Stored::Float(Float::new_f64(h)));
      // Perl L132: style for vertical alignment
      if base != 0.0 {
        let style = format!("vertical-align:{}px", base);
        whatsit.set_property("style", Stored::String(arena::pin(&style)));
      }
      // Store raw dimensions for reversion (tex= attribute)
      whatsit.set_property("width_dim", Stored::Dimension(width));
      whatsit.set_property("height_dim", Stored::Dimension(height));
      // Store the content box for construction time
      let content_opt = whatsit.get_arg(1).cloned();
      if let Some(content) = content_opt {
        whatsit.set_property("content_box", Stored::Digested(content));
      }
    }
  );

  // Perl L171-172: baseline shift macros
  DefMacro!("\\pgf@shift@baseline", "0pt");
  DefMacro!("\\pgf@shift@baseline@smuggle", "0pt");

  //===================================================================
  // 1. Beginning and ending a stream
  //===================================================================

  // Perl L178-192: \pgfsys@typesetpicturebox
  RawTeX!(r"\def\pgfsys@typesetpicturebox#1{\pgf@ya=\pgf@shift@baseline\relax\advance\pgf@ya by-\pgf@picminy\relax\advance\pgf@picmaxy by-\pgf@picminy\relax\advance\pgf@picmaxx by-\pgf@picminx\relax\setbox#1=\hbox{\hskip-\pgf@picminx\lower\pgf@picminy\box#1}\ht#1=\pgf@picmaxy\wd#1=\pgf@picmaxx\dp#1=0pt\leavevmode\lxSVG@insertpicture{\box#1\lxSVG@closescope}}");

  DefMacro!("\\pgfsys@beginpicture", "");
  DefMacro!("\\pgfsys@endpicture", "");

  // Perl L197-210: \pgfsys@hbox — inserts a box in SVG context
  DefConstructor!("\\pgfsys@hbox{Number}",
    sub[document, _args, props] {
      if let Some(Stored::Digested(thebox)) = props.get("thebox") {
        document.absorb(thebox, None)?;
      }
    },
    after_digest => sub[whatsit] {
      let boxnum = whatsit.get_arg(1)
        .map(|a| a.to_string().parse::<i64>().unwrap_or(0))
        .unwrap_or(0);
      let boxname = format!("box{}", boxnum);
      if let Some(bx) = state::lookup_value(&boxname) {
        whatsit.set_property("thebox", bx);
      }
    }
  );

  //===================================================================
  // 2. Path construction
  //===================================================================

  DefPrimitive!("\\lxSVG@clearpath", {
    state::assign_value("pgf_SVGpath", Stored::String(arena::pin("")), Scope::Global);
  });
  DefPrimitive!("\\lxSVG@clearclip", {
    state::assign_value("pgf_clipnext", Stored::Int(0), None);
  });

  // Perl L242-276: path operations
  DefPrimitive!("\\pgfsys@moveto{Dimension}{Dimension}", sub[(x, y)] {
    add_to_svg_path("M", &[x, y]);
  });
  DefPrimitive!("\\pgfsys@lineto{Dimension}{Dimension}", sub[(x, y)] {
    add_to_svg_path("L", &[x, y]);
  });
  DefPrimitive!("\\pgfsys@curveto{Dimension}{Dimension}{Dimension}{Dimension}{Dimension}{Dimension}",
    sub[(x1, y1, x2, y2, x3, y3)] {
    add_to_svg_path("C", &[x1, y1, x2, y2, x3, y3]);
  });
  DefPrimitive!("\\pgfsys@rect{Dimension}{Dimension}{Dimension}{Dimension}",
    sub[(x, y, w, h)] {
    add_to_svg_path("M", &[x, y]);
    add_to_svg_path("h", &[w]);
    add_to_svg_path("v", &[h]);
    add_to_svg_path("h", &[w.negate()]);
    add_to_svg_path("Z", &[]);
  });
  DefPrimitive!("\\pgfsys@closepath", {
    add_to_svg_path("Z", &[]);
  });

  //===================================================================
  // 3. Canvas transformation
  //===================================================================

  DefMacro!("\\pgfsys@transformcm{Float}{Float}{Float}{Float}{Dimension}{Dimension}",
    "\\lxSVG@transformcm{#1}{#2}{#3}{#4}{#5}{#6}\\lxSVG@@transformcm{#1}{#2}{#3}{#4}{#5}{#6}");

  DefConstructor!("\\lxSVG@transformcm{Float}{Float}{Float}{Float}{Dimension}{Dimension}", "");

  DefMacro!("\\lxSVG@@transformcm{Float}{Float}{Float}{Float}{Dimension}{Dimension}",
    sub[(a, b, c, d, e, f)] {
    let transform = format!("transform=matrix({} {} {} {} {} {})",
      floatformat(a.0),
      floatformat(b.0),
      floatformat(c.0),
      floatformat(d.0),
      dim_to_px(e),
      dim_to_px(f));
    let tok_str = format!("\\lxSVG@begingroup{{{}}}", transform);
    mouth::tokenize_internal(&tok_str).unlist()
  });

  //===================================================================
  // 4. Stroking, filling, and clipping
  //===================================================================

  DefPrimitive!("\\pgfsys@clipnext", {
    state::assign_value("pgf_clipnext", Stored::Int(1), None);
  });

  DefMacro!("\\pgfsys@stroke",     "\\lxSVG@stroke\\lxSVG@drawpath{fill:none}");
  DefMacro!("\\pgfsys@fill",       "\\lxSVG@fill\\lxSVG@drawpath{stroke:none}");
  DefMacro!("\\pgfsys@fillstroke", "\\lxSVG@fillstroke\\lxSVG@drawpath{}");

  DefConstructor!("\\lxSVG@stroke", "");
  DefConstructor!("\\lxSVG@fill", "");
  DefConstructor!("\\lxSVG@fillstroke", "");

  // Perl L321-334: \lxSVG@drawpath
  DefMacro!("\\lxSVG@drawpath{}", sub[(arg)] {
    let arg_str = arg.to_string();
    let path = state::lookup_string("pgf_SVGpath");
    let clip = state::lookup_int("pgf_clipnext") != 0;
    if clip {
      let clip_cmd = format!("\\lxSVG@clearpath\\lxSVG@clearclip\\pgfsysprotocol@literal{{\\lxSVG@drawpath@clipped{{{}}}{{{}}}}}", path, arg_str);
      mouth::tokenize_internal(&clip_cmd).unlist()
    } else {
      let draw_cmd = format!("\\lxSVG@clearpath\\pgfsysprotocol@literal{{\\lxSVG@drawpath@unclipped{{{}}}{{{}}}}}", path, arg_str);
      mouth::tokenize_internal(&draw_cmd).unlist()
    }
  });

  // Perl L337-339: unclipped path — emit svg:path
  DefConstructor!("\\lxSVG@drawpath@unclipped{}{}",
    sub[document, args, _props] {
      let d = args.get(0).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      if !d.is_empty() {
        let style = args.get(1).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
        let mut attrs = string_map!("d" => d);
        if !style.is_empty() {
          attrs.insert("style".to_string(), style);
        }
        document.insert_element("svg:path", Vec::new(), Some(attrs))?;
      }
    }
  );

  // Perl L341-348: clipped path
  DefConstructor!("\\lxSVG@drawpath@clipped{}{}",
    sub[document, args, _props] {
      let d = args.get(0).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let style = args.get(1).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let obj = svg_next_object();
      document.open_element("svg:clipPath", Some(string_map!(
        "id" => format!("pgfcp{}", obj)
      )), None)?;
      document.insert_element("svg:path", Vec::new(), Some(string_map!(
        "id" => format!("pgfpath{}", obj),
        "d" => d.clone()
      )))?;
      document.close_element("svg:clipPath")?;
      let mut use_attrs = string_map!(
        "xlink:href" => format!("#pgfpath{}", obj)
      );
      if !style.is_empty() {
        use_attrs.insert("style".to_string(), style);
      }
      document.insert_element("svg:use", Vec::new(), Some(use_attrs))?;
      document.open_element("svg:g", Some(string_map!(
        "clip-path" => format!("url(#pgfcp{})", obj),
        "_autoclose" => "1".to_string()
      )), None)?;
    }
  );

  // Perl L351-371: \pgfsys@discardpath
  DefMacro!("\\pgfsys@discardpath", "\\lxSVG@discardpath\\lxSVG@@discardpath");
  DefConstructor!("\\lxSVG@discardpath", "");

  DefMacro!("\\lxSVG@@discardpath", sub[_args] {
    let path = state::lookup_string("pgf_SVGpath");
    let clip = state::lookup_int("pgf_clipnext") != 0;
    if clip {
      let clip_cmd = format!("\\lxSVG@clearpath\\lxSVG@clearclip\\pgfsysprotocol@literal{{\\lxSVG@discardpath@clipped{{{}}}}}", path);
      mouth::tokenize_internal(&clip_cmd).unlist()
    } else {
      vec![]
    }
  });

  DefConstructor!("\\lxSVG@discardpath@clipped{}",
    sub[document, args, _props] {
      let d = args.get(0).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let obj = svg_next_object();
      document.open_element("svg:clipPath", Some(string_map!(
        "id" => format!("pgfcp{}", obj)
      )), None)?;
      document.insert_element("svg:path", Vec::new(), Some(string_map!("d" => d)))?;
      document.close_element("svg:clipPath")?;
      document.open_element("svg:g", Some(string_map!(
        "clip-path" => format!("url(#pgfcp{})", obj),
        "_autoclose" => "1".to_string()
      )), None)?;
    }
  );

  //===================================================================
  // 5. Graphic state options
  //===================================================================

  DefPrimitive!("\\lxSVG@stopexpansion", {});

  DefMacro!("\\lxSVG@begingroup{}",
    "\\ifpgfpicture\\pgfsysprotocol@literal{\\lxSVG@begingroup@{#1}}\\fi");

  // Perl L387-395: \lxSVG@begingroup@ — opens svg:g with RequiredKeyVals
  DefConstructor!("\\lxSVG@begingroup@ RequiredKeyVals",
    sub[document, args, _props] {
      let current = document.get_node().clone();
      let qname = document::get_node_qname(&current);
      let qname_str = arena::to_string(qname);
      if qname_str.starts_with("ltx:") {
        document.open_element("svg:svg", Some(string_map!(
          "_autoopened" => "1".to_string(),
          "_autoclose" => "1".to_string()
        )), None)?;
      }
      let mut attrs = string_map!("_autoclose" => "1".to_string());
      if let Some(Some(kv)) = args.get(0) {
        // The RequiredKeyVals argument contains key=value pairs
        // Parse them from the string representation
        let kv_str = kv.to_string();
        for pair in kv_str.split(',') {
          let pair = pair.trim();
          if let Some((k, v)) = pair.split_once('=') {
            attrs.insert(k.trim().to_string(), v.trim().to_string());
          } else if !pair.is_empty() {
            // bare key with no value — store as key="key"
            attrs.insert(pair.to_string(), pair.to_string());
          }
        }
      }
      document.open_element("svg:g", Some(attrs), None)?;
    }
  );

  //===================================================================
  // 5b. Line width, caps, joins, dashes
  //===================================================================

  DefMacro!("\\pgfsys@setlinewidth{}",
    "\\lxSVG@setlinewidth{#1}\\lxSVG@begingroup{stroke-width={#1}}");
  DefMacro!("\\pgfsys@buttcap",
    "\\lxSVG@buttcap\\lxSVG@begingroup{stroke-linecap=butt}");
  DefMacro!("\\pgfsys@roundcap",
    "\\lxSVG@roundcap\\lxSVG@begingroup{stroke-linecap=round}");
  DefMacro!("\\pgfsys@rectcap",
    "\\lxSVG@rectcap\\lxSVG@begingroup{stroke-linecap=rect}");
  DefMacro!("\\pgfsys@miterjoin",
    "\\lxSVG@miterjoin\\lxSVG@begingroup{stroke-linejoin=miter}");
  DefMacro!("\\pgfsys@setmiterlimit{}",
    "\\lxSVG@setmiterlimit{#1}\\lxSVG@begingroup{stroke-miterlimit={#1}}");
  DefMacro!("\\pgfsys@roundjoin",
    "\\lxSVG@roundjoin\\lxSVG@begingroup{stroke-linejoin=round}");
  DefMacro!("\\pgfsys@beveljoin",
    "\\lxSVG@beveljoin\\lxSVG@begingroup{stroke-linejoin=bevel}");
  DefMacro!("\\pgfsys@setdash{}{}",
    "\\lxSVG@setdash{#1}{#2}\\edef\\pgf@test@dashpattern{#1}\\lxSVG@begingroup{stroke-dasharray={\\ifx\\pgf@test@dashpattern\\pgfutil@empty none\\else#1\\fi}, stroke-dashoffset={#2}}");
  DefMacro!("\\pgfsys@eoruletrue",
    "\\lxSVG@eoruletrue\\lxSVG@begingroup{fill-rule=evenodd}");
  DefMacro!("\\pgfsys@eorulefalse",
    "\\lxSVG@eorulefalse\\lxSVG@begingroup{fill-rule=nonzero}");

  // Constructors for reversion
  DefConstructor!("\\lxSVG@setlinewidth Undigested", "");
  DefConstructor!("\\lxSVG@buttcap", "");
  DefConstructor!("\\lxSVG@roundcap", "");
  DefConstructor!("\\lxSVG@rectcap", "");
  DefConstructor!("\\lxSVG@miterjoin", "");
  DefConstructor!("\\lxSVG@setmiterlimit Undigested", "");
  DefConstructor!("\\lxSVG@roundjoin", "");
  DefConstructor!("\\lxSVG@beveljoin", "");
  DefConstructor!("\\lxSVG@setdash Undigested Undigested", "");
  DefConstructor!("\\lxSVG@eoruletrue", "");
  DefConstructor!("\\lxSVG@eorulefalse", "");

  //===================================================================
  // 6. Color
  //===================================================================

  DefMacro!("\\lxSVG@setcolor{}{}", "\\ifpgfpicture\\lxSVG@begingroup{#1={#2}}\\fi");

  DefMacro!("\\pgfsys@color@rgb{}{}{}",
    "\\ifpgfpicture\\pgfsys@color@rgb@stroke{#1}{#2}{#3}\\pgfsys@color@rgb@fill{#1}{#2}{#3}\\fi");

  DefMacro!("\\lxSVG@RGB{Float}{Float}{Float}", sub[(r, g, b)] {
    let r = r.0;
    let g = g.0;
    let b = b.0;
    let hex = color_to_hex(r, g, b);
    mouth::tokenize_internal(&hex).unlist()
  });

  DefMacro!("\\lxSVG@GRAY{Float}", sub[(g)] {
    let v = g.0;
    let hex = color_to_hex(v, v, v);
    mouth::tokenize_internal(&hex).unlist()
  });

  DefMacro!("\\lxSVG@CMYK{Float}{Float}{Float}{Float}", sub[(c, m, y, k)] {
    let c = c.0;
    let m = m.0;
    let y = y.0;
    let k = k.0;
    let hex = color_to_hex((1.0-c)*(1.0-k), (1.0-m)*(1.0-k), (1.0-y)*(1.0-k));
    mouth::tokenize_internal(&hex).unlist()
  });

  DefMacro!("\\lxSVG@CMY{Float}{Float}{Float}", sub[(c, m, y)] {
    let c = c.0;
    let m = m.0;
    let y = y.0;
    let hex = color_to_hex(1.0-c, 1.0-m, 1.0-y);
    mouth::tokenize_internal(&hex).unlist()
  });

  DefMacro!("\\pgfsys@color@rgb@stroke{}{}{}",
    "\\lxSVG@color@rgb@stroke{#1}{#2}{#3}\\lxSVG@begingroup{stroke=\\lxSVG@RGB{#1}{#2}{#3}}");
  DefMacro!("\\pgfsys@color@rgb@fill{}{}{}",
    "\\lxSVG@color@rgb@fill{#1}{#2}{#3}\\lxSVG@begingroup{fill=\\lxSVG@RGB{#1}{#2}{#3}}");
  DefMacro!("\\pgfsys@color@cmyk@stroke{}{}{}{}",
    "\\lxSVG@color@cmyk@stroke{#1}{#2}{#3}{#4}\\lxSVG@begingroup{stroke=\\lxSVG@CMYK{#1}{#2}{#3}{#4}}");
  DefMacro!("\\pgfsys@color@cmyk@fill{}{}{}{}",
    "\\lxSVG@color@cmyk@fill{#1}{#2}{#3}{#4}\\lxSVG@begingroup{fill=\\lxSVG@CMYK{#1}{#2}{#3}{#4}}");
  DefMacro!("\\pgfsys@color@cmy@stroke{}{}{}",
    "\\lxSVG@color@cmy@stroke{#1}{#2}{#3}\\lxSVG@begingroup{stroke=\\lxSVG@CMY{#1}{#2}{#3}}");
  DefMacro!("\\pgfsys@color@cmy@fill{}{}{}",
    "\\lxSVG@color@cmy@fill{#1}{#2}{#3}\\lxSVG@begingroup{fill=\\lxSVG@CMY{#1}{#2}{#3}}");
  DefMacro!("\\pgfsys@color@gray@stroke{}",
    "\\lxSVG@color@gray@stroke{#1}\\lxSVG@begingroup{stroke=\\lxSVG@GRAY{#1}}");
  DefMacro!("\\pgfsys@color@gray@fill{}",
    "\\lxSVG@color@gray@fill{#1}\\lxSVG@begingroup{fill=\\lxSVG@GRAY{#1}}");

  DefConstructor!("\\lxSVG@color@rgb@stroke Undigested Undigested Undigested", "");
  DefConstructor!("\\lxSVG@color@rgb@fill Undigested Undigested Undigested", "");
  DefConstructor!("\\lxSVG@color@cmyk@stroke Undigested Undigested Undigested Undigested", "");
  DefConstructor!("\\lxSVG@color@cmyk@fill Undigested Undigested Undigested Undigested", "");
  DefConstructor!("\\lxSVG@color@cmy@stroke Undigested Undigested Undigested", "");
  DefConstructor!("\\lxSVG@color@cmy@fill Undigested Undigested Undigested", "");
  DefConstructor!("\\lxSVG@color@gray@stroke Undigested", "");
  DefConstructor!("\\lxSVG@color@gray@fill Undigested", "");

  //===================================================================
  // 8. Scoping
  //===================================================================

  DefMacro!("\\pgfsys@beginscope",
    "\\lxSVG@beginscope\\lxSVG@begingroup{_scopebegin=1}");
  DefMacro!("\\pgfsys@endscope",
    "\\pgfsysprotocol@literal{\\lxSVG@closescope}\\lxSVG@endscope");

  DefConstructor!("\\lxSVG@closescope",
    sub[document, _args, _props] {
      loop {
        match document.maybe_close_element("svg:g") {
          Ok(Some(node)) => {
            if node.get_attribute("_scopebegin").is_some() {
              break;
            }
          }
          _ => break,
        }
      }
    }
  );

  DefPrimitive!("\\lxSVG@beginscope", {
    stomach::begingroup();
  });

  DefPrimitive!("\\lxSVG@endscope", {
    let _ = stomach::endgroup();
  });

  //===================================================================
  // 9. Image
  //===================================================================

  RawTeX!("\\newbox\\lxSVG@imgbox");

  DefMacro!("\\pgfsys@defineimage",
    "\\edef\\pgf@image{\\lxSVG@includegraphics{\\pgf@imagewidth}{\\pgf@imageheight}{\\pgf@filename}}");

  // Perl L597-623: \lxSVG@includegraphics — simplified
  // The full Perl version has complex beforeConstruct/afterConstruct for foreignObject
  // wrapping. For now, just emit the graphics element.
  DefConstructor!("\\lxSVG@includegraphics{}{} Semiverbatim",
    "<ltx:graphics graphic='#3' options='#options'/>",
    properties => sub[args] {
      let w = args.get(0).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let h = args.get(1).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let graphic = args.get(2).and_then(|a| a.as_ref()).map(|a| a.to_string().trim().to_string()).unwrap_or_default();
      let mut options_parts = Vec::new();
      if !w.is_empty() { options_parts.push(format!("width={}", w)); }
      if !h.is_empty() { options_parts.push(format!("height={}", h)); }
      let options = options_parts.join(",");
      let candidates = graphic.clone();
      Ok(stored_map!("graphic" => graphic, "candidates" => candidates, "options" => options))
    }
  );

  //===================================================================
  // 11. Transparency
  //===================================================================

  DefMacro!("\\pgfsys@stroke@opacity{}",
    "\\lxSVG@stroke@opacity{#1}\\lxSVG@begingroup{stroke-opacity={#1}}");
  DefMacro!("\\pgfsys@fill@opacity{}",
    "\\lxSVG@fill@opacity{#1}\\lxSVG@begingroup{fill-opacity={#1}}");
  DefMacro!("\\pgfsys@fadingfrombox{}{}", "\\lxSVG@fadingfrombox{#1}{#2}");
  DefMacro!("\\pgfsys@usefading{}{}{}{}{}{}{}", "\\lxSVG@usefading{#1}{#2}{#3}{#4}{#5}{#6}{#7}");
  DefMacro!("\\pgfsys@transparencygroupfrombox{}", "\\lxSVG@transparencygroupfrombox{#1}");
  DefMacro!("\\pgfsys@definemask", "\\lxSVG@definemask");

  DefConstructor!("\\lxSVG@stroke@opacity Undigested", "");
  DefConstructor!("\\lxSVG@fill@opacity Undigested", "");
  DefConstructor!("\\lxSVG@fadingfrombox Undigested{Number}", "");
  DefConstructor!("\\lxSVG@usefading Undigested Undigested Undigested Undigested Undigested Undigested Undigested", "");
  DefConstructor!("\\lxSVG@transparencygroupfrombox{Number}", "");
  DefConstructor!("\\lxSVG@definemask", "");

  //===================================================================
  // 12. Reusable objects
  //===================================================================

  DefConstructor!("\\pgfsys@invoke{}",
    sub[document, args, _props] {
      if let Some(Some(arg)) = args.get(0) {
        document.absorb(arg, None)?;
      }
    }
  );

  DefMacro!("\\pgfsys@markposition{}", "");

  //===================================================================
  // 13. Invisibility
  //===================================================================

  RawTeX!("\\def\\pgfsys@begininvisible#1\\pgfsys@endinvisible{}");

  //===================================================================
  // Shading stubs
  //===================================================================
  DefMacro!("\\pgfsys@horishading{}{}{}", "");
  DefMacro!("\\pgfsys@vertshading{}{}{}", "");
  DefMacro!("\\pgfsys@radialshading{}{}{}", "");
  DefMacro!("\\pgfsys@functionalshading{}{}{}", "");
});

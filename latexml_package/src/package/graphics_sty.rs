use crate::prelude::*;
use latexml_core::common::dimension::attribute_format;
use latexml_core::common::numeric_ops::kround;

/// Perl: graphics_scaledbox_props($box, $xscale, $yscale) in graphics.sty.ltxml L63-81
/// Computes scaled dimensions and translation offsets for \scalebox.
pub fn scaled_properties(
  mut body: Digested,
  xscale: f64,
  yscale: f64,
) -> Result<Vec<(&'static str, Stored)>> {
  let (w_dim, h_dim, d_dim, ..) = body.get_size(None)?;
  let w = w_dim.value_of() as f64;
  let h = h_dim.value_of() as f64;
  let d = d_dim.value_of() as f64;
  if w == 0.0 && h == 0.0 && d == 0.0 {
    return Ok(Vec::new());
  }
  let sw = w * xscale;
  let sh = h * yscale;
  let sd = d * yscale;
  let total_h = h + d;
  let s_total_h = sh + sd;
  let xtranslate = (sw - w) * 0.5;
  let ytranslate = (s_total_h - total_h) * (-0.5);

  let dim_attr = |v: f64| attribute_format(kround(v), None);

  Ok(vec![
    ("width", Stored::from(dim_attr(sw))),
    ("height", Stored::from(dim_attr(sh))),
    ("depth", Stored::from(dim_attr(sd))),
    ("xtranslate", Stored::from(dim_attr(xtranslate))),
    ("ytranslate", Stored::from(dim_attr(ytranslate))),
  ])
}

/// Perl: rotatedProperties($box, $angle, %options) in graphics.sty.ltxml L152-202
/// Computes bounding box and translation for rotated box content.
pub fn rotated_properties(
  mut body: Digested,
  angle: f64,
  smash: bool,
) -> Result<Vec<(&'static str, Stored)>> {
  let (w_dim, h_dim, d_dim, ..) = body.get_size(None)?;
  let w = w_dim.value_of() as f64;
  let h = h_dim.value_of() as f64;
  let d = d_dim.value_of() as f64;
  if w == 0.0 && h == 0.0 && d == 0.0 {
    return Ok(Vec::new());
  }
  let x0: f64 = 0.0;
  let y0: f64 = 0.0;
  // Origin parsing omitted for now (TODO: parse from keyvals)

  let total_h = h + d;
  #[allow(clippy::approx_constant)]
  let rad = angle * 3.1415926 / 180.0; // Perl uses this approximation
  let s = rad.sin();
  let c = rad.cos();
  let wp = (w * c).abs() + (total_h * s).abs();
  let corners = [
    (-d - y0) * c + (0.0 - x0) * s + y0, // bottom-left
    (-d - y0) * c + (w - x0) * s + y0,   // bottom-right
    (h - y0) * c + (w - x0) * s + y0,    // top-right
    (h - y0) * c + (0.0 - x0) * s + y0,  // top-left
  ];
  let hp = corners.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
  let dp = -corners.iter().cloned().fold(f64::INFINITY, f64::min);
  let xsh = (wp - w) / 2.0;
  let ysh = (h + d - hp - dp) / 2.0;

  let dim_attr = |v: f64| attribute_format(kround(v), None);
  let width_val = if smash {
    "0.0pt".to_string()
  } else {
    dim_attr(wp)
  };

  Ok(vec![
    ("angle", Stored::from(s!("{angle}"))),
    ("width", Stored::from(width_val)),
    ("height", Stored::from(dim_attr(hp))),
    ("depth", Stored::from(dim_attr(dp))),
    ("innerwidth", Stored::from(dim_attr(w))),
    ("innerheight", Stored::from(dim_attr(h))),
    ("innerdepth", Stored::from(dim_attr(d))),
    ("xtranslate", Stored::from(dim_attr(xsh))),
    ("ytranslate", Stored::from(dim_attr(ysh))),
  ])
}

LoadDefinitions!({
  // Perl: graphics.sty.ltxml — base graphics package
  // Package options: draft, final, hiderotate, hidescale, hiresbb
  // (most are no-ops for LaTeXML)

  // == Scaling boxes ==

  // \scalebox{xscale}[yscale]{content}
  // Perl: DefConstructor('\Gscale@box {Float} [Float] {}', ...)
  // Perl: graphics_scaledbox_props computes scaled dimensions and translation
  // Perl: \Gscale@box {Float} [Float] {} — Float parameters format as "3.0" not "3"
  DefConstructor!("\\scalebox{} []{}", "<ltx:inline-block xscale='#xscale' yscale='#yscale' width='#width' height='#height' depth='#depth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#3</ltx:inline-block>",
  mode => "restricted_horizontal", enter_horizontal => true,
  properties => sub[args] {
    // Format scales as float (3 → "3.0") to match Perl's Float parameter type
    let xs = args[0].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
    let xscale_f: f64 = xs.parse().unwrap_or(1.0);
    let xscale_str = format!("{:.1}", xscale_f);
    let yscale_f: f64 = args[1]
      .as_ref()
      .map(|a| a.to_attribute().parse().unwrap_or(xscale_f))
      .unwrap_or(xscale_f);
    let yscale_str = format!("{:.1}", yscale_f);
    Ok(stored_map!("xscale" => xscale_str, "yscale" => yscale_str))
  },
  after_digest => sub[whatsit] {
    let xscale = whatsit.get_arg(1)
      .map(|a| a.to_attribute().parse::<f64>().unwrap_or(1.0)).unwrap_or(1.0);
    let yscale = whatsit.get_arg(2)
      .map(|a| a.to_attribute().parse::<f64>().unwrap_or(xscale)).unwrap_or(xscale);
    if let Some(body) = whatsit.get_arg(3) {
      let scaled = crate::package::graphics_sty::scaled_properties(body.clone(), xscale, yscale);
      if let Ok(props) = scaled {
        for (k, v) in props {
          whatsit.set_property(k, v);
        }
      }
    }
  });
  Let!("\\Gscale@box", "\\scalebox");

  // \Gscale@box@dd {Dimension}{Dimension} {body}  — Perl L103-110.
  // Two Dimension args express scale as their RATIO (num/denom). LaTeX's
  // graphics emits this internally for `\scalebox{0.5}` when the .5 came
  // from a register lookup that resolved to a Dimension/Dimension form.
  // Without a Rust port, papers using these intermediate CSes (rare but
  // present in some templates) would error with undefined CS.
  DefConstructor!("\\Gscale@box@dd {Dimension}{Dimension}{}",
  "<ltx:inline-block xscale='#xscale' yscale='#yscale' width='#width' height='#height' depth='#depth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#3</ltx:inline-block>",
  mode => "restricted_horizontal", enter_horizontal => true,
  after_digest => sub[whatsit] {
    let parse_pt = |a: Option<&Digested>| -> f64 {
      a.and_then(|x| x.to_attribute().trim_end_matches("pt").parse::<f64>().ok()).unwrap_or(0.0)
    };
    let num = parse_pt(whatsit.get_arg(1));
    let denom = parse_pt(whatsit.get_arg(2));
    let scale = if denom != 0.0 { num / denom } else { 1.0 };
    whatsit.set_property("xscale", Stored::from(s!("{}", scale)));
    whatsit.set_property("yscale", Stored::from(s!("{}", scale)));
    if let Some(body) = whatsit.get_arg(3).cloned() {
      if let Ok(props) = scaled_properties(body, scale, scale) {
        for (k, v) in props { whatsit.set_property(k, v); }
      }
    }
  });

  // \Gscale@box@dddd {xnum}{xdenom}{ynum}{ydenom}{body} — Perl L112-118.
  // Same idea, but separate xscale/yscale ratios.
  DefConstructor!("\\Gscale@box@dddd {Dimension}{Dimension}{Dimension}{Dimension}{}",
  "<ltx:inline-block xscale='#xscale' yscale='#yscale' width='#width' height='#height' depth='#depth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#5</ltx:inline-block>",
  mode => "restricted_horizontal", enter_horizontal => true,
  after_digest => sub[whatsit] {
    let parse_pt = |a: Option<&Digested>| -> f64 {
      a.and_then(|x| x.to_attribute().trim_end_matches("pt").parse::<f64>().ok()).unwrap_or(0.0)
    };
    let xn = parse_pt(whatsit.get_arg(1));
    let xd = parse_pt(whatsit.get_arg(2));
    let yn = parse_pt(whatsit.get_arg(3));
    let yd = parse_pt(whatsit.get_arg(4));
    let xscale = if xd != 0.0 { xn / xd } else { 1.0 };
    let yscale = if yd != 0.0 { yn / yd } else { 1.0 };
    whatsit.set_property("xscale", Stored::from(s!("{}", xscale)));
    whatsit.set_property("yscale", Stored::from(s!("{}", yscale)));
    if let Some(body) = whatsit.get_arg(5).cloned() {
      if let Ok(props) = scaled_properties(body, xscale, yscale) {
        for (k, v) in props { whatsit.set_property(k, v); }
      }
    }
  });

  // Perl: DefParameterType('GraphixDimension', sub { skipSpaces, readXToken,
  //   if ! or undef → undef, else unread + readDimension }, optional => 1)
  DefParameterType!(GraphixDimension, sub[_inner, _extra] {
    gullet::skip_spaces()?;
    let next = gullet::read_x_token(Some(false), false, None)?;
    if next.is_none() || next.as_ref().is_some_and(|t| t.text == pin!("!")) {
      // ! or end-of-input: "let other dimensions determine size"
      Ok(Tokens!())
    } else {
      // Unread and read a Dimension
      if let Some(tok) = next {
        gullet::unread_one(tok);
      }
      let dim = gullet::read_dimension()?;
      // Return the raw sp value as tokens for lossless round-trip.
      // to_attribute() rounds to 1 decimal pt, losing precision in scale calculations.
      Ok(Tokenize!(&dim.value_of().to_string()))
    }
  }, optional => true);

  // Perl graphics.sty.ltxml L40-57: DefParameterType('GraphixDimensions', ...)
  //   A sequence of up to 4 dimensions (for `trim=` / `viewport=`). They
  //   MUST be space-separated but trailing commas are tolerated between
  //   entries. Each entry tries a register value first, else reads a
  //   factor + unit (defaulting to bp). Returns a space-separated token
  //   sequence of the raw sp values.
  DefParameterType!(GraphixDimensions, sub[_inner, _extra] {
    gullet::skip_spaces()?;
    let mut dims: Vec<i64> = Vec::new();
    while dims.len() < 4 {
      if !dims.is_empty() {
        // Optionally consume a single comma between entries (Perl: if
        // the next token isn't T_OTHER(','), unread it).
        if let Some(t) = gullet::read_token()? {
          if t.text != pin!(",") {
            gullet::unread_one(t);
          }
        }
      }
      let is_negative = gullet::read_optional_signs()?;
      // Try register value (Dimension) first, allowing coercion.
      let register_dim = gullet::read_register_value_coerce(
        latexml_core::definition::register::RegisterType::Dimension,
        true,
      )?;
      if let Some(latexml_core::definition::register::RegisterValue::Dimension(d)) = register_dim {
        let v = d.value_of();
        dims.push(if is_negative { -v } else { v });
        continue;
      }
      // Otherwise try factor + unit. If the unit is missing, fall back
      // to `bp` (big points) per Perl L52-54.
      if let Some(factor) = gullet::read_factor()? {
        let unit = match gullet::read_unit()? {
          Some(u) => u,
          None => state::convert_unit("bp"),
        };
        let signed = if is_negative { -factor } else { factor };
        let sp = latexml_core::common::numeric_ops::fixpoint(signed, Some(unit));
        dims.push(sp);
      } else {
        break;
      }
    }
    if dims.is_empty() {
      Ok(Tokens!())
    } else {
      // Space-separated token sequence of raw sp values — matches the
      // shape expected by `image_graphicx_parse` which splits on
      // whitespace and passes each value through `to_bp` (Util::Image
      // L155-159).
      let joined = dims
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join(" ");
      Ok(Tokenize!(&joined))
    }
  }, optional => true);

  // \resizebox{width}{height}{content}
  // Perl: \Gscale@@box computes xscale/yscale, wraps in inline-block.
  DefMacro!(
    "\\resizebox",
    "\\leavevmode\\@ifstar{\\Gscale@@box\\totalheight}{\\Gscale@@box\\height}"
  );
  DefConstructor!("\\Gscale@@box{}{GraphixDimension}{GraphixDimension}{}", "<ltx:inline-block xscale='#xscale' yscale='#yscale' width='#width' height='#height' depth='#depth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#4</ltx:inline-block>",
  mode => "restricted_horizontal", enter_horizontal => true,
  // Perl L124: reversion => '\resizebox{#2}{#3}{#4}' so the `tex=`
  // attribute serializes back to the author-facing \resizebox shape
  // rather than the internal \Gscale@@box dispatcher + heighttype arg.
  reversion => "\\resizebox{#2}{#3}{#4}",
  after_digest => sub[whatsit] {
    let heighttype = whatsit.get_arg(1);
    let use_totalheight = heighttype.as_ref()
      .map(|h| h.to_attribute().contains("totalheight"))
      .unwrap_or(false);
    let target_width = whatsit.get_arg(2);
    let target_height = whatsit.get_arg(3);
    if let Some(body) = whatsit.get_arg(4).cloned() {
      let (w_dim, h_dim, d_dim, _, _, _) = body.clone().get_size(None)?;
      let w = w_dim.value_of() as f64;
      let mut h = h_dim.value_of() as f64;
      let d = d_dim.value_of() as f64;
      if use_totalheight { h += d; }
      // GraphixDimension stores raw sp value as token string
      let tw: Option<f64> = target_width.and_then(|a| {
        let s = a.to_attribute();
        if s.is_empty() { None } else {
          s.parse::<f64>().ok()
        }
      });
      let th: Option<f64> = target_height.and_then(|a| {
        let s = a.to_attribute();
        if s.is_empty() { None } else {
          s.parse::<f64>().ok()
        }
      });
      let mut xscale = 1.0_f64;
      let mut yscale = 1.0_f64;
      if let Some(tw_val) = tw { xscale = tw_val / (if w != 0.0 { w } else { 1.0 }); }
      if let Some(th_val) = th { yscale = th_val / (if h != 0.0 { h } else { 1.0 }); }
      if tw.is_some() && th.is_none() { yscale = xscale; }
      if th.is_some() && tw.is_none() { xscale = yscale; }
      whatsit.set_property("xscale", Stored::from(s!("{}", xscale)));
      whatsit.set_property("yscale", Stored::from(s!("{}", yscale)));
      if let Ok(props) = crate::package::graphics_sty::scaled_properties(body, xscale, yscale) {
        for (k, v) in props {
          whatsit.set_property(k, v);
        }
      }
    }
  });

  // == Rotation ==

  // Rotation keyvals
  DefKeyVal!("Grot", "origin", "");
  DefKeyVal!("Grot", "x", "Dimension");
  DefKeyVal!("Grot", "y", "Dimension");
  DefKeyVal!("Grot", "units", "");

  // ORDER MATTERS: define `{rotatebox}` environment FIRST, then the
  // `\rotatebox` DefConstructor. DefEnvironment auto-registers a bare
  // `\rotatebox` CS (Perl Package.pm L1949-1969 hook-pipeline parity)
  // with the environment's signature `{Float}` and the env's mode setup.
  // If the env def runs AFTER the DefConstructor, the env's bare form
  // clobbers the constructor — users writing `\rotatebox{0}{…}` then get
  // the ENV semantics (single `{Float}` arg, `restricted_horizontal` body
  // that never unwinds on the outer `\end{figure}`). arxiv 1007.3314 hit
  // this: `graphicx.sty` happens to re-register `\rotatebox` AFTER the
  // env, so graphicx-loading papers worked, but bare `graphics`-only
  // papers (revtex4 with `\usepackage[dvips]{graphics}`) left the env
  // bare form active and tripped `\end{figure} in restricted_horizontal`.
  // DefEnvironment form — used as `\begin{rotatebox}{90}…\end{rotatebox}`.
  DefEnvironment!("{rotatebox}{Float}",
  "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#body</ltx:inline-block>",
  after_digest_body => sub[whatsit] {
    let angle = whatsit.get_arg(1)
      .map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0))
      .unwrap_or(0.0);
    if let Ok(Some(body)) = whatsit.get_body() {
      if let Ok(props) = crate::package::graphics_sty::rotated_properties(body, angle, false) {
        for (k, v) in props {
          whatsit.set_property(k, v);
        }
      }
    }
  });

  // Now re-register the bare `\rotatebox` as a DefConstructor, overriding
  // the env's auto-registered bare form. `\rotatebox[keys]{angle}{body}`
  // needs the OptionalKeyVals + Float + group signature that the env
  // cannot express.
  DefConstructor!("\\rotatebox OptionalKeyVals:Grot {Float} {}",
  "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#3</ltx:inline-block>",
  mode => "restricted_horizontal", enter_horizontal => true,
  after_digest => sub[whatsit] {
    let angle = whatsit.get_arg(2)
      .map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0))
      .unwrap_or(0.0);
    if let Some(body) = whatsit.get_arg(3) {
      let rotated = crate::package::graphics_sty::rotated_properties(body.clone(), angle, false);
      if let Ok(props) = rotated {
        for (k, v) in props {
          whatsit.set_property(k, v);
        }
      }
    }
  });

  DefMacro!("\\Grot@erotate", "\\rotatebox[]");

  // Perl: DefConstructor('\reflectbox {}', ...) with properties callback
  // Returns width/height/depth from box size, xscale=-1, yscale=1
  DefConstructor!("\\reflectbox{}", "<ltx:inline-block xscale='#xscale' yscale='#yscale' width='#width' height='#height' depth='#depth'>#1</ltx:inline-block>",
  mode => "restricted_horizontal", enter_horizontal => true,
  after_digest => sub[whatsit] {
    if let Some(mut body) = whatsit.get_arg(1).cloned() {
      if let Ok((w, h, d, _, _, _)) = body.get_size(None) {
        if w.value_of() != 0 || h.value_of() != 0 || d.value_of() != 0 {
          whatsit.set_property("width", Stored::from(w.to_attribute()));
          whatsit.set_property("height", Stored::from(h.to_attribute()));
          whatsit.set_property("depth", Stored::from(d.to_attribute()));
          whatsit.set_property("xscale", Stored::from("-1".to_string()));
          whatsit.set_property("yscale", Stored::from("1".to_string()));
        }
      }
    }
  });

  // == Graphics path and inclusion ==

  // Perl graphics.sty.ltxml L248-260: \graphicspath DirectoryList.
  //   properties: for each dir → PushValue(GRAPHICSPATHS => pathname_absolute(…))
  //   body: for each path in props{paths} → insertPI('latexml', graphicspath=>$path)
  //
  // DirectoryList reads the arg ToString-first so `_` in path names never
  // becomes a SUB-catcode during digestion.
  DefConstructor!("\\graphicspath DirectoryList",
  sub[document, _args, props] {
    if let Some(Stored::String(paths_sym)) = props.get("paths") {
      let paths = arena::with(*paths_sym, |s| s.to_string());
      for path in paths.split('\x1e').filter(|p| !p.is_empty()) {
        let mut attrs = HashMap::default();
        attrs.insert(String::from("graphicspath"), path.to_string());
        document.insert_pi("latexml", Some(attrs))?;
      }
    }
  },
  properties => sub[args] {
    let arg = args.first()
      .and_then(|a| a.as_ref())
      .map(|a| a.to_string())
      .unwrap_or_default();
    let root = state::with_value("SOURCEDIRECTORY",
      |v| v.map(|s| s.to_string()).unwrap_or_default());
    let mut collected: Vec<String> = Vec::new();
    for dir in arg.split('}') {
      let dir = dir.trim_start_matches('{').trim();
      if !dir.is_empty() {
        // Perl: pathname_absolute(pathname_canonical($dir), $root)
        let path = if root.is_empty() || dir.starts_with('/') {
          dir.to_string()
        } else {
          s!("{}/{}", root, dir)
        };
        // Perl: PushValue(GRAPHICSPATHS => $path)
        let _ = state::push_value("GRAPHICSPATHS",
          Stored::String(arena::pin(&path)));
        collected.push(path);
      }
    }
    Ok(stored_map!("paths" => collected.join("\x1e")))
  });

  // Perl: DefMacro('\includegraphics OptionalMatch:* [][] Semiverbatim',
  //   '\@includegraphics#1[#2][#3]{#4}');
  DefMacro!(
    "\\includegraphics OptionalMatch:* [][] Semiverbatim",
    "\\@includegraphics#1[#2][#3]{#4}"
  );

  DefConstructor!("\\@includegraphics OptionalMatch:* [][] Semiverbatim",
    "<ltx:graphics graphic='#graphic' candidates='#candidates' options='#options'/>",
    enter_horizontal => true,
    properties => sub[args] {
      let path = args[3].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      let path = path.trim().to_string();
      let candidates = latexml_core::util::image::image_candidates(&path);
      Ok(stored_map!("graphic" => path, "candidates" => candidates, "options" => ""))
    },
    alias => "\\includegraphics");

  DefConstructor!("\\DeclareGraphicsExtensions{}", "");
  DefConstructor!("\\DeclareGraphicsRule{}{}{} Undigested", "");

  // == Gin internal macros (Perl: RawTeX block, lines 311-324) ==

  Let!("\\Gin@decode", "\\@empty");
  DefMacro!("\\Gin@exclamation", "!");
  Let!("\\Gin@page", "\\@empty");
  DefMacro!("\\Gin@pagebox", "cropbox");
  DefConditional!("\\ifGin@interpolate");
  Let!("\\Gin@log", "\\wlog");
  Let!("\\Gin@req@sizes", "\\relax");
  DefMacro!("\\Gin@scalex", "1");
  Let!("\\Gin@scaley", "\\Gin@exclamation");
  // These reference macros that may not exist yet, so define them
  DefMacro!("\\Gin@nat@height", "");
  DefMacro!("\\Gin@nat@width", "");
  Let!("\\Gin@req@height", "\\Gin@nat@height");
  Let!("\\Gin@req@width", "\\Gin@nat@width");
  Let!("\\Gin@viewport@code", "\\relax");

  // Perl: DefConditional('\ifGin@clip');
  DefConditional!("\\ifGin@clip");
  // Perl: DefMacro('\Gin@i [][]{}', '');
  DefMacro!("\\Gin@i[][]{}", "");

  // Perl: DefPrimitive('\Gscale@div DefToken Dimension Dimension', sub {
  //   my $n = $num->valueOf; my $d = $denom->valueOf;
  //   DefMacro($cs, Tokens(Explode(($n == 0 ? 1 : $n / $d)))); });
  // \Gscale@div{\cs}{\dima}{\dimb} : \cs = \dima / \dimb.
  // Port matches the multido_sty \multido@step@d pattern (DefToken {Dimension}
  // arg + runtime DefMacro! install). Perl's `$n / $d` is a Perl scalar
  // division so we cast to f64; mirror the "0 divisor → 1" guard.
  DefPrimitive!("\\Gscale@div DefToken {Dimension} {Dimension}",
    sub[(cs, num, denom)] {
    let n = num.value_of() as f64;
    let d = denom.value_of() as f64;
    let ratio = if n == 0.0 { 1.0 } else { n / d };
    DefMacro!(cs, None, Tokens!(Explode!(format!("{ratio}"))));
  });

  // Perl: \set@color defined elsewhere but referenced by graphics
  // Provide a no-op fallback if not already defined
  DefMacro!("\\set@color", "");
});

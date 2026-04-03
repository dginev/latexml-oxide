use crate::prelude::*;

/// Convert TeX points to CSS pixels using DPI setting (default 100).
/// Perl: $$self[0] / 65536 * DPI / 72.27
fn px_value(pt: f64) -> f64 {
  // DPI default is 100 in LaTeXML (state::lookupValue('DPI') || 100)
  let dpi = state::lookup_value("DPI")
    .and_then(|v| if let Stored::Number(n) = v { Some(n.0 as f64) } else { None })
    .unwrap_or(100.0);
  // Round to 2 decimal places (Perl default precision)
  (pt * dpi / 72.27 * 100.0).round() / 100.0
}

/// Format a px value, dropping trailing ".0" for integers
fn fmt_px(v: f64) -> String {
  if v == v.round() && v.abs() < 1e10 {
    format!("{}", v as i64)
  } else {
    format!("{v}")
  }
}

LoadDefinitions!({
  // Not sure that ltx:p is the best to use here, but ... (see also \vbox, \vtop)
  // This should be fairly compact vertically.
  DefConstructor!("\\@shortstack@cr",
    "</ltx:p><ltx:p>",
    properties   => { stored_map!("isBreak" => true) },
    reversion    => Tokens!(T_CS!("\\\\"), T_CR!()),
    before_digest => { egroup()?; },
    after_digest  => { bgroup(); });

  DefConstructor!("\\shortstack[]{}  OptionalMatch:* [Dimension]",
  "<ltx:inline-block align='#align'><ltx:p>#2</ltx:p></ltx:inline-block>",
  bounded      => true,
  sizer        => "#2",
  before_digest => { reenter_text_mode(false);
    // Rebind \\ and its aliases to shortstack line break
    Let!("\\\\", "\\@shortstack@cr");
    Let!("\\lx@hidden@cr", "\\@shortstack@cr");
    Let!("\\lx@newline", "\\@shortstack@cr");
    AssignRegister!("\\baselineskip" , Glue::new_spec("-1pt", None, None, None, None).into());
    AssignRegister!("\\lineskip"     , Glue::new_spec("3pt", None, None, None, None).into());
    bgroup(); },
  after_digest => sub[_whatsit] {
    egroup()?; },
  // Note: does not get layout=vertical, since linebreaks are explicit
  properties => sub[args] {
    let align = args[0].as_ref().map(|a| {
      match a.to_string().as_str() {
        "l" => "left", "r" => "right", _ => ""
      }
    }).unwrap_or("");
    Ok(stored_map!("align" => align, "vattach" => "bottom"))
  },
  mode => "text");

  //======================================================================
  // C.14.1 The picture Environment
  // Perl: latex_constructs.pool.ltxml lines 4927-5185
  //======================================================================

  // Registers
  DefRegister!("\\unitlength" => Dimension!("1pt"));
  DefRegister!("\\@wholewidth" => Dimension!("0.4pt"));
  DefRegister!("\\@halfwidth" => Dimension!("0.2pt"));

  // \thinlines / \thicklines — set \@wholewidth register
  DefMacro!("\\thinlines", "\\@wholewidth 0.4pt\\relax");
  DefMacro!("\\thicklines", "\\@wholewidth 0.8pt\\relax");
  DefMacro!("\\linethickness{}", "\\@wholewidth #1\\relax");
  DefMacro!("\\arrowlength{}", None);
  DefMacro!("\\qbeziermax", "500");
  DefMacro!("\\@killglue", "\\unskip\\@whiledim \\lastskip >\\z@\\do{\\unskip}");

  // Tag: ltx:picture — auto-generate ID with "pic" prefix
  Tag!("ltx:picture", after_open => sub[document, node] {
    document.generate_id(node, "pic")?;
  });

  // {picture} environment: (width,height) with optional (origin-x,origin-y)
  // Pair now survives digestion via RegisterValue::Pair, so properties can extract coordinates.
  DefEnvironment!("{picture} Pair OptionalPair",
    "<ltx:picture width='#width' height='#height' origin-x='#origin-x' origin-y='#origin-y'\
      fill='none' stroke='none' unitlength='#unitlength'>\
      ?#transform(<ltx:g transform='#transform'>#body</ltx:g>)(#body)\
    </ltx:picture>",
    mode => "text",
    before_digest => {
      // Perl: before_picture — Let \raisebox to \pic@raisebox
      Let!("\\raisebox", "\\pic@raisebox");
    },
    properties => sub[args] {
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let (w, h) = match args[0].as_ref() {
        Some(d) => match d.data() {
          DigestedData::RegisterValue(RegisterValue::Pair(p)) => (p.x.0 * unit, p.y.0 * unit),
          _ => (0.0, 0.0),
        },
        None => (0.0, 0.0),
      };
      // Perl Float formats with at least one decimal place
      let fmt_pt = |v: f64| -> String {
        if v == v.round() { format!("{v:.1}pt") } else { format!("{v}pt") }
      };
      let mut map = stored_map!(
        "width"      => Stored::String(arena::pin(fmt_pt(w))),
        "height"     => Stored::String(arena::pin(fmt_pt(h))),
        "unitlength" => Stored::String(arena::pin(fmt_pt(unit)))
      );
      // Origin from OptionalPair — Perl: origin-x, origin-y, transform
      if let Some(d) = args[1].as_ref() {
        if let DigestedData::RegisterValue(RegisterValue::Pair(p)) = d.data() {
          let ox = p.x.0 * unit;
          let oy = p.y.0 * unit;
          map.insert("origin-x", Stored::String(arena::pin(fmt_pt(ox))));
          map.insert("origin-y", Stored::String(arena::pin(fmt_pt(oy))));
          // Perl: translate(negate(origin).pxValue)
          let tx = px_value(-ox);
          let ty = px_value(-oy);
          map.insert("transform", Stored::String(arena::pin(
            format!("translate({},{})", fmt_px(tx), fmt_px(ty)))));
        }
      }
      Ok(map)
    }
  );

  // \put(x,y){content} — Perl: Match:( reads "(", Until:, reads y, Until:) reads y
  // Now that Pair survives digestion (RegisterValue::Pair), use it directly.
  DefMacro!("\\put SkipSpaces Match:( Until:, Until:) {}", "\\lx@pic@put(#2,#3){#4\\relax}");
  DefConstructor!("\\lx@pic@put Pair {}",
    "<ltx:g transform='#transform' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth'>#2</ltx:g>",
    alias => "\\put",
    mode  => "text",
    properties => sub[args] {
      let (x, y) = match args[0].as_ref() {
        Some(d) => match d.data() {
          DigestedData::RegisterValue(RegisterValue::Pair(p)) => (p.x.0, p.y.0),
          _ => { let s = d.to_string(); let mut p = s.splitn(2, ',');
            (p.next().unwrap_or("0").trim().parse().unwrap_or(0.0),
             p.next().unwrap_or("0").trim().parse().unwrap_or(0.0)) }
        },
        None => (0.0, 0.0),
      };
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let tx = px_value(x * unit);
      let ty = px_value(y * unit);
      let transform_str = format!("translate({},{})", fmt_px(tx), fmt_px(ty));
      // Perl: $box->getSize to extract inner dimensions
      let (iw, ih, id) = if let Some(body) = args[1].as_ref() {
        let (w, h, d, _, _, _) = body.clone().get_size(None)?;
        // Perl: $w = undef if $w && ($w->ptValue == 0)
        let w_opt = if w.value_of() == 0 { None } else { Some(w) };
        (w_opt, Some(h), Some(d))
      } else {
        (None, None, None)
      };
      let mut map = stored_map!(
        "transform" => Stored::String(arena::pin(&transform_str))
      );
      if let Some(w) = iw { map.insert("innerwidth", Stored::Dimension(w)); }
      if let Some(h) = ih { map.insert("innerheight", Stored::Dimension(h)); }
      if let Some(d) = id { map.insert("innerdepth", Stored::Dimension(d)); }
      Ok(map)
    }
  );

  // \line(slope){length} — Perl: DefConstructor('\line Pair:Number {Float}', ...)
  DefMacro!("\\line Match:( Until:, Until:) {Float}", "\\lx@pic@line{#2}{#3}{#4}");
  DefConstructor!("\\lx@pic@line{}{}{}",
    "<ltx:line points='#points' stroke='#color' stroke-width='#thick'/>",
    alias => "\\line",
    properties => sub[args] {
      let mx: f64 = args[0].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let my: f64 = args[1].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let xlength: f64 = args[2].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      // slopeToPicCoord: compute endpoint from slope and length, then convert to px
      let s = if mx > 0.0 { 1.0 } else if mx < 0.0 { -1.0 } else { 0.0 };
      let ex = px_value(xlength * unit * s);
      let ey = if s == 0.0 {
        px_value(xlength * unit * (if my > 0.0 { 1.0 } else { -1.0 }))
      } else {
        px_value(xlength * unit * my / mx.abs())
      };
      Ok(stored_map!(
        "points" => Stored::String(arena::pin(format!("0,0 {},{}", fmt_px(ex), fmt_px(ey)))),
        "thick"  => Stored::String(arena::pin(format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // \vector(slope){length} — Perl: DefConstructor('\vector Pair:Number {Float}', ...)
  DefMacro!("\\vector Match:( Until:, Until:) {Float}", "\\lx@pic@vector{#2}{#3}{#4}");
  DefConstructor!("\\lx@pic@vector{}{}{}",
    "<ltx:line points='#points' stroke='#color' stroke-width='#thick' terminators='->'/>",
    alias => "\\vector",
    properties => sub[args] {
      let mx: f64 = args[0].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let my: f64 = args[1].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let xlength: f64 = args[2].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      let s = if mx > 0.0 { 1.0 } else if mx < 0.0 { -1.0 } else { 0.0 };
      let ex = px_value(xlength * unit * s);
      let ey = if s == 0.0 {
        px_value(xlength * unit * (if my > 0.0 { 1.0 } else { -1.0 }))
      } else {
        px_value(xlength * unit * my / mx.abs())
      };
      Ok(stored_map!(
        "points" => Stored::String(arena::pin(format!("0,0 {},{}", fmt_px(ex), fmt_px(ey)))),
        "thick"  => Stored::String(arena::pin(format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // \circle*{diameter} — filled or unfilled circle
  DefConstructor!("\\circle OptionalMatch:* {Float}",
    "<ltx:circle x='0' y='0' r='#radius' fill='#fill' stroke='#stroke' stroke-width='#thick'/>",
    alias => "\\circle",
    properties => sub[args] {
      let filled = args[0].is_some(); // OptionalMatch:* → Some if * present
      let dia: f64 = args[1].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0);
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      let radius = px_value(dia * unit * 0.5);
      let (fill, stroke) = if filled {
        ("#000000", "none")
      } else {
        ("none", "#000000")
      };
      Ok(stored_map!(
        "radius" => Stored::String(arena::pin(fmt_px(radius))),
        "fill"   => fill,
        "stroke" => stroke,
        "thick"  => Stored::String(arena::pin(format!("{thick}")))
      ))
    }
  );

  // \oval[radius](width,height)[part] — decompose pair
  DefMacro!("\\oval", "\\lx@pic@oval");
  DefConstructor!("\\lx@pic@oval [Float] Pair []",
    "<ltx:rect x='#ox' y='#oy' width='#owidth' height='#oheight' rx='#radius'\
      stroke='#color' fill='none' part='#3' stroke-width='#thick'/>",
    alias => "\\oval",
    properties => sub[args] {
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      // Perl: $r = ($r ? picScale($r) : Dimension('40pt'))
      let r_requested: f64 = args[0].as_ref()
        .map(|d| d.to_string().trim().parse().unwrap_or(40.0) * unit)
        .unwrap_or(40.0);
      // Extract size from Pair
      let (sx, sy) = match args[1].as_ref() {
        Some(d) => match d.data() {
          DigestedData::RegisterValue(RegisterValue::Pair(p)) => (p.x.0 * unit, p.y.0 * unit),
          _ => (0.0, 0.0),
        },
        None => (0.0, 0.0),
      };
      let (hx, hy) = (sx * 0.5, sy * 0.5);
      // Perl: $r = $r->smaller($halfsize->getX->absolute)->smaller($halfsize->getY->absolute)
      let r = r_requested.min(hx.abs()).min(hy.abs());
      Ok(stored_map!(
        "ox"      => Stored::String(arena::pin(fmt_px(px_value(-hx)))),
        "oy"      => Stored::String(arena::pin(fmt_px(px_value(-hy)))),
        "owidth"  => Stored::String(arena::pin(fmt_px(px_value(sx)))),
        "oheight" => Stored::String(arena::pin(fmt_px(px_value(sy)))),
        "radius"  => Stored::String(arena::pin(fmt_px(px_value(r)))),
        "thick"   => Stored::String(arena::pin(s!("{thick}"))),
        "color"   => "#000000"
      ))
    }
  );

  // \qbezier[N](p1)(p2)(p3) — decompose 3 pairs into coordinates
  DefMacro!("\\qbezier [Number] Match:( Until:, Until:) Match:( Until:, Until:) Match:( Until:, Until:)",
    "\\lx@pic@qbezier{#1}{#3}{#4}{#6}{#7}{#9}{#10}");
  DefConstructor!("\\lx@pic@qbezier{}{}{}{}{}{}{}",
    "<ltx:bezier points='#points' stroke='#color' stroke-width='#thick'/>",
    alias => "\\qbezier",
    properties => sub[args] {
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      // args: [0]=N, [1]=x1, [2]=y1, [3]=x2, [4]=y2, [5]=x3, [6]=y3
      let parse_f = |i: usize| -> f64 {
        args[i].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0)
      };
      let (x1, y1) = (px_value(parse_f(1) * unit), px_value(parse_f(2) * unit));
      let (x2, y2) = (px_value(parse_f(3) * unit), px_value(parse_f(4) * unit));
      let (x3, y3) = (px_value(parse_f(5) * unit), px_value(parse_f(6) * unit));
      Ok(stored_map!(
        "points" => Stored::String(arena::pin(format!("{},{} {},{} {},{}",
          fmt_px(x1), fmt_px(y1), fmt_px(x2), fmt_px(y2), fmt_px(x3), fmt_px(y3)))),
        "thick"  => Stored::String(arena::pin(format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // Perl L5166-5175: \multiput expands to n \put commands with coordinate stepping.
  DefMacro!("\\multiput Match:( Until:, Until:) Match:( Until:, Until:) {}{}", sub[args] {
    // args: 0=Match:(, 1=x, 2=y, 3=Match:(, 4=dx, 5=dy, 6=n, 7=body
    let x_str = args.get(1).map(|a| a.revert().unwrap_or_default().to_string()).unwrap_or_default();
    let y_str = args.get(2).map(|a| a.revert().unwrap_or_default().to_string()).unwrap_or_default();
    let dx_str = args.get(4).map(|a| a.revert().unwrap_or_default().to_string()).unwrap_or_default();
    let dy_str = args.get(5).map(|a| a.revert().unwrap_or_default().to_string()).unwrap_or_default();
    let n: i64 = args.get(6).map(|a| a.revert().unwrap_or_default().to_string()
      .trim().parse().unwrap_or(1)).unwrap_or(1);
    let body = args.get(7).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();

    let mut x: f64 = x_str.trim().parse().unwrap_or(0.0);
    let mut y: f64 = y_str.trim().parse().unwrap_or(0.0);
    let dx: f64 = dx_str.trim().parse().unwrap_or(0.0);
    let dy: f64 = dy_str.trim().parse().unwrap_or(0.0);

    let mut result = Vec::new();
    for _ in 0..n {
      result.push(T_CS!("\\put"));
      result.push(T_OTHER!("("));
      result.extend(Explode!(s!("{}", x)));
      result.push(T_OTHER!(","));
      result.extend(Explode!(s!("{}", y)));
      result.push(T_OTHER!(")"));
      result.push(T_BEGIN!());
      result.extend(body.clone().unlist());
      result.push(T_END!());
      x += dx;
      y += dy;
    }
    Ok(Tokens::new(result))
  });

  // Box commands for picture mode
  // Perl: \pic@makebox@ Undigested RequiredKeyVals Pair []{} — the master box constructor
  // Creates optional <rect> for frame + <g class="makebox"> for content with positioning.
  // Properties compute inner dimensions from $box->getSize and position from [pos] arg.
  //
  // The Perl macros are:
  //   \pic@makebox  → \pic@makebox@{\makebox}{}
  //   \pic@framebox → \pic@makebox@{\framebox}{framed=true}
  //   \frame{}      → \pic@makebox@{\framebox}{framed=true}(0,0)[bl]{#1}
  //   \dashbox      → \pic@makebox@{\dashbox(N)}{framed=true,dash={N}}
  //
  // For now: simplified port without getSize (uses zero defaults).
  // The constructor uses sub[] to build DOM directly matching Perl's output structure.
  DefConstructor!("\\pic@makebox@ Undigested {} Pair []{}",
    sub[document, args, props] {
      // args: [0]=cs(Undigested), [1]=kv_text({}), [2]=size(Pair), [3]=pos([]), [4]=box({})
      let framed = props.get("framed").is_some();
      // \@wholewidth captured at digest time in properties callback
      let thick = match props.get("thick") {
        Some(Stored::String(s)) => arena::to_string(*s).parse::<f64>().unwrap_or(0.4),
        _ => 0.4,
      };
      // Frame rect (only when framed=true)
      if framed {
        let mut rect_attrs = map!(
          "x" => "0".to_string(), "y" => "0".to_string(),
          "width" => props.get("fwidth").map(|s| s.to_string()).unwrap_or_else(|| "0".into()),
          "height" => props.get("fheight").map(|s| s.to_string()).unwrap_or_else(|| "0".into()),
          "stroke" => "#000000".to_string(),
          "stroke-width" => format!("{thick}"),
          "fill" => "none".to_string()
        );
        if let Some(dash) = props.get("dash") {
          rect_attrs.insert("stroke-dasharray".to_string(), dash.to_string());
        }
        document.insert_element("ltx:rect", Vec::new(), Some(rect_attrs))?;
      }
      // Content <g>
      let mut g_attrs = map!("class" => "makebox".to_string());
      for &key in &["innerwidth", "innerheight", "innerdepth"] {
        if let Some(v) = props.get(key) {
          let vs = v.to_string();
          if !vs.is_empty() {
            g_attrs.insert(key.to_string(), vs);
          }
        }
      }
      let xshift = props.get("xshift").map(|s| s.to_string()).unwrap_or_else(|| s!("0"));
      let yshift = props.get("yshift").map(|s| s.to_string()).unwrap_or_else(|| s!("0"));
      g_attrs.insert(s!("transform"), format!("translate({xshift},{yshift})"));
      document.open_element("ltx:g", Some(g_attrs), None)?;
      if let Some(body) = args.get(4).and_then(|a| a.as_ref()) {
        document.absorb(body, None)?;
      }
      document.close_element("ltx:g")?;
    },
    properties => sub[args] {
      let unit = match state::lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      // Capture \@wholewidth at digest time for frame stroke-width
      let thick = match state::lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      // args: [0]=cs, [1]=kv_text, [2]=size(Pair), [3]=pos, [4]=box
      let kv_str = args[1].as_ref().map(|d| d.to_string()).unwrap_or_default();

      // Perl: $box->getSize — extract (width, height, depth) from body
      let (w, h, d) = if let Some(body) = args[4].as_ref() {
        let (bw, bh, bd, _, _, _) = body.clone().get_size(None)?;
        (bw, bh, bd)
      } else {
        (Dimension::default(), Dimension::default(), Dimension::default())
      };
      let ht = Dimension::new(h.value_of() + d.value_of()); // total height = h + d

      // Extract frame size from Pair parameter (args[2])
      let (mut ww, mut hh) = match args[2].as_ref() {
        Some(d) => match d.data() {
          DigestedData::RegisterValue(RegisterValue::Pair(p)) => {
            (Dimension::new((p.x.0 * unit * 65536.0) as i64),
             Dimension::new((p.y.0 * unit * 65536.0) as i64))
          },
          _ => (Dimension::default(), Dimension::default()),
        },
        None => (Dimension::default(), Dimension::default()),
      };

      // Perl: position-based shift computation
      let (mut xshift, mut yshift) = (Dimension::default(), Dimension::default());
      if ww.value_of() != 0 || hh.value_of() != 0 {
        let pos = args[3].as_ref().map(|d| d.to_string().to_lowercase()).unwrap_or_default();
        // x positioning
        if pos.contains('l') {
          xshift = Dimension::default(); // left-aligned: x = 0
        } else if pos.contains('r') {
          xshift = Dimension::new(ww.value_of() - w.value_of()); // right-aligned
        } else {
          xshift = Dimension::new((ww.value_of() - w.value_of()) / 2); // centered
        }
        // y positioning
        if pos.contains('b') {
          yshift = Dimension::default(); // bottom-aligned: y = 0
        } else if pos.contains('t') {
          yshift = Dimension::new(hh.value_of() - ht.value_of()); // top-aligned
        } else {
          yshift = Dimension::new((hh.value_of() - ht.value_of()) / 2); // centered
        }
      } else {
        ww = w;
        hh = Dimension::new(h.value_of() + d.value_of());
      }

      // Frame dimensions: use ww/hh if nonzero, else content size
      let fw = if ww.value_of() != 0 { ww } else { w };
      let fh = if hh.value_of() != 0 { hh } else { Dimension::new(h.value_of() + d.value_of()) };

      let xs_px = px_value(xshift.pt_value(None));
      let ys_px = px_value(yshift.pt_value(None));

      let mut map = stored_map!(
        "innerwidth" => Stored::Dimension(w),
        "innerheight" => Stored::Dimension(h),
        "innerdepth" => Stored::Dimension(d),
        "fwidth" => Stored::Dimension(fw),
        "fheight" => Stored::Dimension(fh),
        "xshift" => Stored::String(arena::pin(fmt_px(xs_px))),
        "yshift" => Stored::String(arena::pin(fmt_px(ys_px)))
      );
      if kv_str.contains("framed") {
        map.insert("framed", Stored::Bool(true));
      }
      if let Some(dash_start) = kv_str.find("dash={") {
        let rest = &kv_str[dash_start + 6..];
        if let Some(end) = rest.find('}') {
          map.insert("dash", Stored::String(arena::pin(&rest[..end])));
        }
      }
      map.insert("thick", Stored::String(arena::pin(s!("{thick}"))));
      Ok(map)
    },
    mode => "text"
  );

  // Perl macro aliases
  DefMacro!("\\pic@makebox",            "\\pic@makebox@{\\makebox}{}");
  DefMacro!("\\pic@framebox",           "\\pic@makebox@{\\framebox}{framed=true}");
  DefMacro!("\\lx@pic@dashbox{Float}",  "\\pic@makebox@{\\dashbox(#1)}{framed=true,dash={#1}}");
  DefMacro!("\\dashbox Until:(",
    "\\ifx.#1.\\lx@pic@dashbox{0}(\\else\\lx@pic@dashbox{#1}(\\fi");
  DefMacro!("\\frame{}",
    "\\pic@makebox@{\\framebox}{framed=true}(0,0)[bl]{#1}");

  // \pic@raisebox — simplified raisebox for picture mode
  DefConstructor!("\\pic@raisebox{Dimension}[Dimension][Dimension]{}",
    "<ltx:g y='#1'>#4</ltx:g>",
    alias => "\\raisebox"
  );

  // Perl: latex_constructs.pool.ltxml line 4862
  // Stubs for color/xcolor packages (overridden when color.sty is loaded)
  Let!("\\set@color", "\\relax");
  Let!("\\color@begingroup", "\\relax");
  Let!("\\color@endgroup", "\\relax");
  Let!("\\color@setgroup", "\\relax");
  Let!("\\color@hbox", "\\relax");
  Let!("\\color@vbox", "\\relax");
  Let!("\\color@endbox", "\\relax");

  // Perl: latex_constructs.pool.ltxml line 5802
  DefMacro!("\\ignorespacesafterend", None);

  // Perl: latex_constructs.pool.ltxml line 5027
  // Pre-define \Gin@driver so graphics.sty doesn't error when loaded from disk
  DefMacro!("\\Gin@driver", "");
});

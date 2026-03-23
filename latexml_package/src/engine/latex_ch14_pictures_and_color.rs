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
    "<ltx:picture width='#width' height='#height' fill='none' stroke='none' unitlength='#unitlength'>\
      #body\
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
      Ok(stored_map!(
        "width"      => Stored::String(arena::pin(&format!("{w}pt"))),
        "height"     => Stored::String(arena::pin(&format!("{h}pt"))),
        "unitlength" => Stored::String(arena::pin(&format!("{unit}pt")))
      ))
    }
  );

  // \put(x,y){content} — Perl: Match:( reads "(", Until:, reads y, Until:) reads y
  // Now that Pair survives digestion (RegisterValue::Pair), use it directly.
  DefMacro!("\\put SkipSpaces Match:( Until:, Until:) {}", "\\lx@pic@put(#2,#3){#4\\relax}");
  DefConstructor!("\\lx@pic@put Pair {}",
    "<ltx:g transform='#transform'>#2</ltx:g>",
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
      Ok(stored_map!("transform" => Stored::String(arena::pin(&transform_str))))
    }
  );

  // \line(slope){length} — decompose pair into separate slope components
  DefMacro!("\\line Match:( Until:, Until:) {}", "\\lx@pic@line{#2}{#3}{#4}");
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
        "points" => Stored::String(arena::pin(&format!("0,0 {},{}", fmt_px(ex), fmt_px(ey)))),
        "thick"  => Stored::String(arena::pin(&format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // \vector(slope){length} — like \line but with arrow terminator
  DefMacro!("\\vector Match:( Until:, Until:) {}", "\\lx@pic@vector{#2}{#3}{#4}");
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
        "points" => Stored::String(arena::pin(&format!("0,0 {},{}", fmt_px(ex), fmt_px(ey)))),
        "thick"  => Stored::String(arena::pin(&format!("{thick}"))),
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
        "radius" => Stored::String(arena::pin(&format!("{}", fmt_px(radius)))),
        "fill"   => fill,
        "stroke" => stroke,
        "thick"  => Stored::String(arena::pin(&format!("{thick}")))
      ))
    }
  );

  // \oval[radius](width,height)[part] — decompose pair
  DefMacro!("\\oval", "\\lx@pic@oval");
  DefConstructor!("\\lx@pic@oval [Float] Pair []",
    "<ltx:rect x='#ox' y='#oy' width='#owidth' height='#oheight' rx='#radius'\
      stroke='#color' fill='none' part='#3' stroke-width='#thick'/>",
    alias => "\\oval"
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
        "points" => Stored::String(arena::pin(&format!("{},{} {},{} {},{}",
          fmt_px(x1), fmt_px(y1), fmt_px(x2), fmt_px(y2), fmt_px(x3), fmt_px(y3)))),
        "thick"  => Stored::String(arena::pin(&format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // \multiput(pos)(delta){n}{body} — Perl expands to n \put commands via macro.
  // Use Match:(/Until: to decompose coordinates, avoiding Pair digestion issues.
  // Simplified: just place the body at the initial position.
  // TODO: full multiput loop expansion with delta stepping
  DefMacro!("\\multiput Match:( Until:, Until:) Match:( Until:, Until:) {}{}", "\\put(#2,#3){#8}");

  // Box commands for picture mode (simplified)
  DefMacro!("\\pic@makebox Pair [] {}", "#3");
  DefMacro!("\\pic@framebox Pair [] {}", "#3");
  DefMacro!("\\frame{}", "#1");
  DefMacro!("\\dashbox{Float} Pair [] {}", "#4");

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

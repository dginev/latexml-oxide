//! `latex_constructs` section 13: C.14-C.15 Pictures, Fonts, Symbols
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.14-C.15 Pictures, Fonts, Symbols
  // ======================================================================

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
  before_digest => {
    // Rebind \\ and \lx@newline to shortstack line break.
    // Matches Perl: only \\ is rebound (Perl does NOT rebind \lx@hidden@cr).
    // \lx@newline is also rebound because \\ is Let to \lx@newline at the
    // top level, so \lx@newline tokens in content must also become @shortstack@cr.
    // NOTE: \lx@hidden@cr must NOT be rebound — doing so causes is_column_end()
    // to match \\ as a column end inside alignments, because is_column_end
    // compares meanings and \lx@hidden@cr is a COLUMN_END sentinel.
    Let!("\\\\", "\\@shortstack@cr");
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
  mode => "restricted_horizontal");

  //======================================================================
  // C.14.1 The picture Environment
  // Perl: latex_constructs.pool.ltxml lines 4927-5185
  //======================================================================

  // Registers
  DefRegister!("\\unitlength" => Dimension!("1pt"));
  DefRegister!("\\@wholewidth" => Dimension!("0.4pt"));
  DefRegister!("\\@halfwidth" => Dimension!("0.2pt"));

  // \thinlines / \thicklines — set \@wholewidth register
  // Perl L4928-4929: DefPrimitiveI — assigns \@wholewidth register directly at
  // stomach level (not via TeX-level expansion). Faithful port.
  DefPrimitive!("\\thinlines", {
    assign_register(
      "\\@wholewidth",
      RegisterValue::Dimension(Dimension!("0.4pt")),
      None,
      vec![],
    )?;
  });
  DefPrimitive!("\\thicklines", {
    assign_register(
      "\\@wholewidth",
      RegisterValue::Dimension(Dimension!("0.8pt")),
      None,
      vec![],
    )?;
  });
  DefMacro!("\\linethickness{}", "\\@wholewidth #1\\relax");
  // Perl L4933: DefPrimitive('\arrowlength{Dimension}', sub { AssignValue('arrowlength', $_[1]); });
  // Stores the dimension under state key `arrowlength` for later lookup
  // by the picture drawing routines (see Perl L4978-4979).
  DefPrimitive!("\\arrowlength {Dimension}", sub[(length)] {
    assign_value("arrowlength", Stored::Dimension(length), None);
  });
  DefMacro!("\\qbeziermax", "500");
  // Perl: \bezier — LaTeX 2.09 compat alias for \qbezier with different syntax
  DefMacro!(
    "\\bezier Until:(",
    "\\ifx.#1.\\lx@pic@bezier{0}(\\else\\lx@pic@bezier{#1}(\\fi"
  );
  DefMacro!("\\lx@pic@bezier{} Pair Pair Pair", "\\qbezier[#1]#2#3#4");
  DefMacro!(
    "\\@killglue",
    "\\unskip\\@whiledim \\lastskip >\\z@\\do{\\unskip}"
  );

  // Tag: ltx:picture — Perl latex_constructs.pool.ltxml L4995:
  //   Tag('ltx:picture', autoOpen => 0.5, autoClose => 1, afterOpen => &GenerateID)
  // The 0.5 fractional priority is honoured by `compute_indirect_model` in
  // state.rs: picture is the only tag with lower-than-full openability, so
  // other auto-openers (para, p, text, item, …) win whenever they can also
  // reach the target element. Picture is selected only for picture-specific
  // primitives (\line, \circle, \vector, \put) used bare inside a {figure}
  // or similar context where no fuller wrapper fits.
  Tag!("ltx:picture",
    auto_open  => true,
    auto_close => true,
    after_open => sub[document, node] {
      document.generate_id(node, "pic")?;
    }
  );

  // {picture} environment: (width,height) with optional (origin-x,origin-y)
  // Pair now survives digestion via RegisterValue::Pair, so properties can extract coordinates.
  DefEnvironment!("{picture} Pair OptionalPair",
    "<ltx:picture width='#width' height='#height' origin-x='#origin-x' origin-y='#origin-y'\
      fill='none' stroke='none' unitlength='#unitlength'>\
      ?#transform(<ltx:g transform='#transform'>#body</ltx:g>)(#body)\
    </ltx:picture>",
    // Perl #2798: picture is an inline block — internal_vertical, no leaveHorizontal.
    mode => "inline_internal_vertical",
    before_digest => {
      // Perl: before_picture — Let \raisebox to \pic@raisebox
      Let!("\\raisebox", "\\pic@raisebox");
    },
    properties => sub[args] {
      let unit = match lookup_register("\\unitlength", Vec::new())? {
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
      // curve2e.sty:273-280 `\@picture` records the picture's span and offset
      // as `\pict@dimen`/`\pict@offset` (in `\unitlength` multiples) for its
      // `\AutoGrid`/`\GraphGrid` defaults; the pict2e binding reads these.
      let raw_pair = |data: Option<&DigestedData>| -> String {
        match data {
          Some(DigestedData::RegisterValue(RegisterValue::Pair(p))) => format!("{},{}", p.x.0, p.y.0),
          _ => "0,0".to_string(),
        }
      };
      assign_value("PICTURE_DIMEN", raw_pair(args[0].as_ref().map(|d| d.data())), Some(Scope::Global));
      assign_value("PICTURE_OFFSET", raw_pair(args[1].as_ref().map(|d| d.data())), Some(Scope::Global));
      // Perl Float formats with at least one decimal place
      let fmt_pt = |v: f64| -> String {
        if v == v.round() { format!("{v:.1}pt") } else { format!("{v}pt") }
      };
      let mut map = stored_map!(
        "width"      => Stored::String(pin(fmt_pt(w))),
        "height"     => Stored::String(pin(fmt_pt(h))),
        "unitlength" => Stored::String(pin(fmt_pt(unit)))
      );
      // Origin from OptionalPair — Perl: origin-x, origin-y, transform
      if let Some(d) = args[1].as_ref()
        && let DigestedData::RegisterValue(RegisterValue::Pair(p)) = d.data() {
          let ox = p.x.0 * unit;
          let oy = p.y.0 * unit;
          map.insert("origin-x", Stored::String(pin(fmt_pt(ox))));
          map.insert("origin-y", Stored::String(pin(fmt_pt(oy))));
          // Perl: translate(negate(origin).pxValue)
          let tx = px_value(-ox);
          let ty = px_value(-oy);
          map.insert("transform", Stored::String(pin(
            format!("translate({},{})", fmt_px(tx), fmt_px(ty)))));
        }
      Ok(map)
    }
  );

  // \put(x,y){content} — Perl: Match:( reads "(", Until:, reads y, Until:) reads y
  // Now that Pair survives digestion (RegisterValue::Pair), use it directly.
  DefMacro!(
    "\\put SkipSpaces Match:( Until:, Until:) {}",
    "\\lx@pic@put(#2,#3){#4\\relax}"
  );
  DefConstructor!("\\lx@pic@put Pair {}",
    "<ltx:g transform='#transform' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth'>#2</ltx:g>",
    alias => "\\put",
    mode  => "restricted_horizontal",
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
      let unit = match lookup_register("\\unitlength", Vec::new())? {
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
        "transform" => Stored::String(pin(&transform_str))
      );
      if let Some(w) = iw { map.insert("innerwidth", Stored::Dimension(w)); }
      if let Some(h) = ih { map.insert("innerheight", Stored::Dimension(h)); }
      if let Some(d) = id { map.insert("innerdepth", Stored::Dimension(d)); }
      Ok(map)
    }
  );

  //============================================================
  // Picture primitives (\line, \vector, \oval, \qbezier, \bezier)
  //============================================================
  //
  // Umbrella WISDOM #44 intentional divergence for the block below:
  //
  // Perl defines each picture primitive as
  //   DefConstructor('\line Pair:Number {Float}', …)
  //   DefConstructor('\vector Pair:Number {Float}', …)
  //   DefConstructor('\oval Pair:Float []', …)
  //   DefConstructor('\qbezier [] Pair:Number Pair:Number Pair:Number', …)
  //   DefConstructor('\bezier {Number} Pair:Float Pair:Float Pair:Float', …)
  // using the `Pair:Number`/`Pair:Float` parameter type, which parses
  // the LaTeX `(x,y)` slope/position syntax directly into a pair of
  // numbers for the constructor's args.
  //
  // Rust doesn't have the `Pair:*` parameter type, so each port is
  // split into a DefMacro trampoline with
  // `Match:( Until:, Until:) {…}` parsing the (a,b) syntax manually,
  // followed by a hidden `\lx@pic@<name>{}{}{…}` DefConstructor that
  // takes the 3 (or more) pre-parsed args.
  //
  // Audit reports 5 DefConstructor → DefMacro kind flips across
  // \line, \vector, \oval, \qbezier, \lx@pic@bezier. All 5 carry
  // the same rationale (missing Pair:Number parameter type), so
  // individual entries don't re-carry the tag.

  // \line(slope){length} — Perl: DefConstructor('\line Pair:Number {Float}', ...)
  //
  // Some papers (witness 2306.13101) use `\line` in non-picture context as a
  // length unit, e.g. `\diagbox[height=2.5\line]{…}{…}`. There the
  // following token is NOT `(` and the strict `Match:(` reader emits
  // `Error:expected:Match Missing argument Match:(`. Dispatch by peeking
  // the next non-space token: if `(`, run the picture chain; otherwise
  // fall back to plain TeX's `\line` (an `\hbox to \hsize` length builder
  // from plain_base.rs) so a surrounding dimension reader can consume
  // `\line` as a length without errors.
  DefMacro!("\\line", sub[_args] {
    if if_next(T_OTHER!("("))? {
      Ok(Tokens!(T_CS!("\\lx@pic@line@dispatch")))
    } else {
      Ok(mouth::tokenize_internal("\\hbox to \\hsize"))
    }
  });
  // The actual picture-mode \line dispatched from the peek above.
  DefMacro!(
    "\\lx@pic@line@dispatch Match:( Until:, Until:) {Float}",
    "\\lx@pic@line{#2}{#3}{#4}"
  );
  DefConstructor!("\\lx@pic@line{}{}{}",
    "<ltx:line points='#points' stroke='#color' stroke-width='#thick'/>",
    alias => "\\line",
    properties => sub[args] {
      let mx: f64 = args[0]
        .as_ref()
        .map(|d| d.to_string().trim().parse().unwrap_or(0.0))
        .unwrap_or(0.0);
      let my: f64 = args[1]
        .as_ref()
        .map(|d| d.to_string().trim().parse().unwrap_or(0.0))
        .unwrap_or(0.0);
      let xlength: f64 = args[2]
        .as_ref()
        .map(|d| d.to_string().trim().parse().unwrap_or(0.0))
        .unwrap_or(0.0);
      let unit = match lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match lookup_register("\\@wholewidth", Vec::new())? {
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
        "points" => Stored::String(pin(format!("0,0 {},{}", fmt_px(ex), fmt_px(ey)))),
        "thick"  => Stored::String(pin(format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // pict2e.sty:686-740 `\polyline(x1,y1)(x2,y2)…`, `\polygon[*]`, `\Line`,
  // `\Vector`/`\polyvector`: a polyline in ABSOLUTE picture coordinates
  // (`\unitlength` multiples), rendered as one `ltx:line` (the same element
  // `\line` emits). `\lx@pic@polyline{terminators}{closed}` reads the `(x,y)`
  // pairs itself. The pict2e binding is otherwise a no-op stub (its driver
  // chain has no XML meaning); curve2e.sty:240 `\renewcommand*\polyline`
  // needs the base to exist. Witnesses sapthesis-doc, unifith-doc.
  fn read_pic_pairs() -> Result<Vec<(f64, f64)>> {
    let mut pairs = Vec::new();
    loop {
      skip_spaces()?;
      if !if_next(T_OTHER!("("))? {
        break;
      }
      read_token()?;
      let x = read_until(&Tokens!(T_OTHER!(",")))?
        .map(|t| t.to_string())
        .unwrap_or_default();
      let y = read_until(&Tokens!(T_OTHER!(")")))?
        .map(|t| t.to_string())
        .unwrap_or_default();
      let parse = |s: String| -> f64 { s.trim().parse().unwrap_or(0.0) };
      pairs.push((parse(x), parse(y)));
    }
    Ok(pairs)
  }
  DefConstructor!("\\lx@pic@polyline{}{}",
    "<ltx:line points='#points' stroke='#color' stroke-width='#thick' terminators='#terminators'/>",
    properties => sub[args] {
      let terminators = args[0].as_ref().map(|d| d.to_string()).unwrap_or_default();
      let closed = args[1].as_ref().map(|d| d.to_string() == "1").unwrap_or(false);
      let unit = match lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      let mut pairs = read_pic_pairs()?;
      if closed && let Some(first) = pairs.first().copied() {
        pairs.push(first);
      }
      let points = pairs.iter()
        .map(|(x, y)| format!("{},{}", fmt_px(px_value(x * unit)), fmt_px(px_value(y * unit))))
        .collect::<Vec<_>>().join(" ");
      Ok(stored_map!(
        "points" => Stored::String(pin(points)),
        "thick"  => Stored::String(pin(format!("{thick}"))),
        "color"  => "#000000",
        "terminators" => Stored::String(pin(terminators))
      ))
    }
  );

  // \vector(slope){length} — Perl: DefConstructor('\vector Pair:Number {Float}', ...)
  DefMacro!(
    "\\vector Match:( Until:, Until:) {Float}",
    "\\lx@pic@vector{#2}{#3}{#4}"
  );
  DefConstructor!("\\lx@pic@vector{}{}{}",
    "<ltx:line points='#points' stroke='#color' stroke-width='#thick' terminators='->'/>",
    alias => "\\vector",
    properties => sub[args] {
      let mx: f64 = args[0]
        .as_ref()
        .map(|d| d.to_string().trim().parse().unwrap_or(0.0))
        .unwrap_or(0.0);
      let my: f64 = args[1]
        .as_ref()
        .map(|d| d.to_string().trim().parse().unwrap_or(0.0))
        .unwrap_or(0.0);
      let xlength: f64 = args[2]
        .as_ref()
        .map(|d| d.to_string().trim().parse().unwrap_or(0.0))
        .unwrap_or(0.0);
      let unit = match lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match lookup_register("\\@wholewidth", Vec::new())? {
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
        "points" => Stored::String(pin(format!("0,0 {},{}", fmt_px(ex), fmt_px(ey)))),
        "thick"  => Stored::String(pin(format!("{thick}"))),
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
      let dia: f64 = args[1]
        .as_ref()
        .map(|d| d.to_string().trim().parse().unwrap_or(0.0))
        .unwrap_or(0.0);
      let unit = match lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match lookup_register("\\@wholewidth", Vec::new())? {
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
        "radius" => Stored::String(pin(fmt_px(radius))),
        "fill"   => fill,
        "stroke" => stroke,
        "thick"  => Stored::String(pin(format!("{thick}")))
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
      let unit = match lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match lookup_register("\\@wholewidth", Vec::new())? {
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
        "ox"      => Stored::String(pin(fmt_px(px_value(-hx)))),
        "oy"      => Stored::String(pin(fmt_px(px_value(-hy)))),
        "owidth"  => Stored::String(pin(fmt_px(px_value(sx)))),
        "oheight" => Stored::String(pin(fmt_px(px_value(sy)))),
        "radius"  => Stored::String(pin(fmt_px(px_value(r)))),
        "thick"   => Stored::String(pin(s!("{thick}"))),
        "color"   => "#000000"
      ))
    }
  );

  // \qbezier[N](p1)(p2)(p3) — quadratic Bezier. Perl LaTeX.pool.ltxml L5182:
  //   DefConstructor('\qbezier [Number] Pair Pair Pair', …).
  // Read each coordinate via the `Pair` parameter type (`(x,y)`, skipping
  // leading spaces), mirroring Perl, rather than a manual
  // `Match:( Until:, Until:)` decomposition. The manual form had two faults:
  // (1) the THIRD pair's y-coordinate was always dropped — the terminal
  // `Until:)` slot returned empty, so `\qbezier(1,2)(3,4)(5,6)` rendered
  // `points="…,0"` instead of `…,6` (with an optional `[N]` it grabbed
  // unrelated garbage); (2) a space after the optional `[N]`
  // (`\qbezier[10] (…)`) failed the first match. `Pair` reads each `(x,y)`
  // cleanly (and skips leading spaces), fixing both. Witness 1701.03735
  // (`\qbezier[10] (…)`) + the long-standing y3 drop (picture.xml baseline).
  // The DefMacro extracts the three Pair structs and forwards the six
  // coordinates as text to `\lx@pic@qbezier`, whose constructor scales them
  // by \unitlength (px) exactly as before — the px-scaling is a separate,
  // pre-existing divergence from Perl's raw storage, kept unchanged.
  DefMacro!("\\qbezier [Number] Pair Pair Pair", sub[args] {
    let get_pair = |i: usize| -> (f64, f64) {
      args.get(i).and_then(|a| match a {
        ArgWrap::Pair(p) => Some((p.x.0, p.y.0)),
        _ => None,
      }).unwrap_or((0.0_f64, 0.0_f64))
    };
    let n = args.first().map(|a| a.revert().unwrap_or_default()).unwrap_or_default();
    let (x1, y1) = get_pair(1);
    let (x2, y2) = get_pair(2);
    let (x3, y3) = get_pair(3);
    let mut result = Vec::with_capacity(40);
    result.push(T_CS!("\\lx@pic@qbezier"));
    result.push(T_BEGIN!());
    result.extend(n.unlist_ref().iter().copied());
    result.push(T_END!());
    for (x, y) in [(x1, y1), (x2, y2), (x3, y3)] {
      result.push(T_BEGIN!());
      result.extend(Explode!(s!("{}", x)));
      result.push(T_END!());
      result.push(T_BEGIN!());
      result.extend(Explode!(s!("{}", y)));
      result.push(T_END!());
    }
    Ok(Tokens::new(result))
  });
  DefConstructor!("\\lx@pic@qbezier{}{}{}{}{}{}{}",
    "<ltx:bezier points='#points' stroke='#color' stroke-width='#thick'/>",
    alias => "\\qbezier",
    properties => sub[args] {
      let unit = match lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match lookup_register("\\@wholewidth", Vec::new())? {
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
        "points" => Stored::String(pin(format!("{},{} {},{} {},{}",
          fmt_px(x1), fmt_px(y1), fmt_px(x2), fmt_px(y2), fmt_px(x3), fmt_px(y3)))),
        "thick"  => Stored::String(pin(format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // `\lx@pic@cbezier{N}{x0}{y0}…{x3}{y3}` — the four-point (cubic) sibling of
  // `\lx@pic@qbezier`, the target of pict2e's `\cbezier` (pict2e_sty.rs); a
  // four-point `<ltx:bezier>` renders as an SVG `C` segment (latexml_post
  // svg.rs `convert_bezier`, Perl SVG.pm `convertBezier`).
  DefConstructor!("\\lx@pic@cbezier{}{}{}{}{}{}{}{}{}",
    "<ltx:bezier points='#points' stroke='#color' stroke-width='#thick'/>",
    alias => "\\cbezier",
    properties => sub[args] {
      let unit = match lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      let thick = match lookup_register("\\@wholewidth", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 0.4,
      };
      let parse_f = |i: usize| -> f64 {
        args[i].as_ref().map(|d| d.to_string().trim().parse().unwrap_or(0.0)).unwrap_or(0.0)
      };
      let pts: Vec<String> = (0..4)
        .map(|k| {
          let (x, y) = (px_value(parse_f(2 * k + 1) * unit), px_value(parse_f(2 * k + 2) * unit));
          format!("{},{}", fmt_px(x), fmt_px(y))
        })
        .collect();
      Ok(stored_map!(
        "points" => Stored::String(pin(pts.join(" "))),
        "thick"  => Stored::String(pin(format!("{thick}"))),
        "color"  => "#000000"
      ))
    }
  );

  // Perl L5166-5175: \multiput expands to n \put commands with coordinate stepping.
  DefMacro!("\\multiput Pair Pair {}{}", sub[args] {
    let (x0, y0) = args.first().and_then(|a| match a {
      ArgWrap::Pair(p) => Some((p.x.0, p.y.0)),
      _ => None,
    }).unwrap_or((0.0_f64, 0.0_f64));
    let (dx, dy) = args.get(1).and_then(|a| match a {
      ArgWrap::Pair(p) => Some((p.x.0, p.y.0)),
      _ => None,
    }).unwrap_or((0.0_f64, 0.0_f64));
    let n: i64 = args.get(2).map(|a| a.revert().unwrap_or_default().to_string()
      .trim().parse().unwrap_or(1)).unwrap_or(1);
    let body = args.get(3).map(|a| a.revert().unwrap_or_default()).unwrap_or_default();

    let mut x: f64 = x0;
    let mut y: f64 = y0;

    // Each iteration emits roughly `8 + body.len()` tokens; pre-size
    // conservatively + use borrow-iter-copied for body to avoid the
    // per-iteration Vec<Token> clone.
    let body_len = body.len();
    let mut result = Vec::with_capacity(((n as usize) * (8 + body_len)).min(1 << 20));
    for _ in 0..n {
      result.push(T_CS!("\\put"));
      result.push(T_OTHER!("("));
      result.extend(Explode!(s!("{}", x)));
      result.push(T_OTHER!(","));
      result.extend(Explode!(s!("{}", y)));
      result.push(T_OTHER!(")"));
      result.push(T_BEGIN!());
      result.extend(body.unlist_ref().iter().copied());
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
        Some(Stored::String(s)) => with(*s, |v| v.parse::<f64>().unwrap_or(0.4)),
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
      let unit = match lookup_register("\\unitlength", Vec::new())? {
        Some(RegisterValue::Dimension(d)) => d.pt_value(None),
        _ => 1.0,
      };
      // Capture \@wholewidth at digest time for frame stroke-width
      let thick = match lookup_register("\\@wholewidth", Vec::new())? {
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
        "xshift" => Stored::String(pin(fmt_px(xs_px))),
        "yshift" => Stored::String(pin(fmt_px(ys_px)))
      );
      if kv_str.contains("framed") {
        map.insert("framed", Stored::Bool(true));
      }
      if let Some(dash_start) = kv_str.find("dash={") {
        let rest = &kv_str[dash_start + 6..];
        if let Some(end) = rest.find('}') {
          map.insert("dash", Stored::String(pin(&rest[..end])));
        }
      }
      map.insert("thick", Stored::String(pin(s!("{thick}"))));
      Ok(map)
    },
    mode => "text"
  );

  // Perl macro aliases
  DefMacro!("\\pic@makebox", "\\pic@makebox@{\\makebox}{}");
  DefMacro!("\\pic@framebox", "\\pic@makebox@{\\framebox}{framed=true}");
  DefMacro!(
    "\\lx@pic@dashbox{Float}",
    "\\pic@makebox@{\\dashbox(#1)}{framed=true,dash={#1}}"
  );
  DefMacro!(
    "\\dashbox Until:(",
    "\\ifx.#1.\\lx@pic@dashbox{0}(\\else\\lx@pic@dashbox{#1}(\\fi"
  );
  DefMacro!(
    "\\frame{}",
    "\\pic@makebox@{\\framebox}{framed=true}(0,0)[bl]{#1}"
  );

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
  // \stop — closes the current input mouth (Plain TeX command)
  Let!("\\stop", "\\endinput");
  def_macro_noop("\\ignorespacesafterend")?;

  // \Gin@driver lives in `latex_constructs_rust_only.rs` (pure Rust
  // hotfix, not in any Perl latex_*.pool.ltxml).

  //**********************************************************************
  // C.15 Font Selection
  //**********************************************************************
  //======================================================================
  // C.15.1 Changing the Type Style
  //======================================================================
  // Text styles.

  DefMacro!("\\rmdefault", "cmr");
  DefMacro!("\\sfdefault", "cmss");
  DefMacro!("\\ttdefault", "cmtt");
  DefMacro!("\\bfdefault", "bx");
  DefMacro!("\\mddefault", "m");
  DefMacro!("\\itdefault", "it");
  DefMacro!("\\sldefault", "sl");
  DefMacro!("\\scdefault", "sc");
  DefMacro!("\\updefault", "n");
  DefMacro!("\\encodingdefault", "OT1");
  DefMacro!("\\familydefault", "\\rmdefault");
  DefMacro!("\\seriesdefault", "\\mddefault");
  DefMacro!("\\shapedefault", "\\updefault");

  Let!("\\mediumseries", "\\mdseries");
  Let!("\\normalshape", "\\upshape");

  // ? DefMacro("\\f@encoding','cm');
  DefMacro!("\\f@family", "cmr");
  DefMacro!("\\f@series", "m");
  DefMacro!("\\f@shape", "n");
  DefMacro!("\\f@size", "10");

  // These do NOT immediately effect the font!
  DefMacro!("\\fontfamily{}", "\\edef\\f@family{#1}");
  DefMacro!("\\fontseries{}", "\\edef\\f@series{#1}");
  DefMacro!("\\fontshape{}", "\\edef\\f@shape{#1}");

  // For fonts not allowed in math!!!
  // Perl L5226: \not@math@alphabet@@ checks if we're in math mode
  // LaTeX kernel also defines \not@math@alphabet (2 args) — stub both
  // Perl L5349: DefMacro('\not@math@alphabet{}{}', ...) — conditional error
  // message in math mode, no-op otherwise. Rust keeps the no-op stub but
  // matches the Perl kind (DefMacro — expansion-time, same as the
  // invocation sites `\mdseries`/`\bfseries` which expand it inline).
  def_macro_noop("\\not@math@alphabet{}{}")?;
  DefPrimitive!("\\not@math@alphabet@@ {}", sub[(c)] {
    if lookup_bool_sym(pin!("IN_MATH")) {
      let c = c.to_string();
      let message = s!("Command {:?} invalid in math mode", c);
      Warn!("unexpected", c, message);
    }
    Ok(vec![])
  });

  // These DO immediately effect the font!
  DefMacro!(
    "\\mdseries",
    "\\not@math@alphabet@@{\\mddefault}\\fontseries{\\mddefault}\\selectfont"
  );
  DefMacro!(
    "\\bfseries",
    "\\not@math@alphabet@@{\\bfdefault}\\fontseries{\\bfdefault}\\selectfont"
  );

  DefMacro!(
    "\\rmfamily",
    "\\not@math@alphabet@@{\\rmdefault}\\fontfamily{\\rmdefault}\\selectfont"
  );
  DefMacro!(
    "\\sffamily",
    "\\not@math@alphabet@@{\\sfdefault}\\fontfamily{\\sfdefault}\\selectfont"
  );
  DefMacro!(
    "\\ttfamily",
    "\\not@math@alphabet@@{\\ttdefault}\\fontfamily{\\ttdefault}\\selectfont"
  );

  DefMacro!(
    "\\upshape",
    "\\not@math@alphabet@@{\\updefault}\\fontshape{\\updefault}\\selectfont"
  );
  DefMacro!(
    "\\itshape",
    "\\not@math@alphabet@@{\\itdefault}\\fontshape{\\itdefault}\\selectfont"
  );
  DefMacro!(
    "\\slshape",
    "\\not@math@alphabet@@{\\sldefault}\\fontshape{\\sldefault}\\selectfont"
  );
  DefMacro!(
    "\\scshape",
    "\\not@math@alphabet@@{\\scdefault}\\fontshape{\\scdefault}\\selectfont"
  );

  DefMacro!(
    "\\normalfont",
    "\\fontfamily{\\rmdefault}\\fontseries{\\mddefault}\\fontshape{\\updefault}\\selectfont"
  );
  // `\fontencoding{ASCII}` (OXIDIZED_DESIGN #144, issue #723): a verbatim `~`/`^`
  // is a literal catcode-12 char, so under T1 it decodes through the fontmap to
  // Bruce Miller's deliberate accent glyphs U+02DC/U+02C6 (LaTeXML #2435). In a
  // verbatim/URL those must stay ASCII. Selecting the identity `ASCII` fontmap for
  // the verbatim font (grouped, so it reverts after) keeps `~`/`^` ASCII while the
  // `\ttdefault` family still drives the typewriter styling — the same treatment
  // `Verbatim`/`HyperVerbatim` apply at digest time. Perl loses ASCII here too.
  DefMacro!(
    "\\verbatim@font",
    "\\fontencoding{ASCII}\\fontfamily{\\ttdefault}\\fontseries{\\mddefault}\\fontshape{\\updefault}\\selectfont"
  );

  Let!("\\reset@font", "\\normalfont");

  // Perl: latex_constructs.pool.ltxml:5251
  // \@fontswitch — LaTeX 2.09 compat helper used by article/book/letter
  // classes to define \cal and \mit (math-mode font switches). Perl's
  // override drops the kernel body's \math@bgroup/\math@egroup
  // machinery (irrelevant outside real TeX boxing); the simpler form
  // is functionally equivalent for LaTeXML's XML output.
  DefMacro!("\\@fontswitch{}{}", "\\ifmmode #2\\relax\\else #1 \\fi");

  // Perl: latex_constructs.pool.ltxml L5204-5222.
  //
  // `already_reported` mirrors Perl's `reported_unrecognized_font_*` guard:
  // an unrecognized family/series/shape is announced ONCE per document,
  // globally, not once per `\selectfont`. Before this, 2503.04421 emitted
  // 28 identical `Info:unexpected:ding` lines — one per table cell.
  //
  // The middle `LoadFontMap($family)` branch is load-bearing for the whole
  // family of dingbat/symbol packages that select a font by FAMILY rather
  // than by encoding: bbding's `\dingfamily` is
  // `\fontencoding{U}\fontfamily{ding}\selectfont`, and there is no
  // `u.fontmap` — so the glyph can only be decoded by treating the family
  // name as the encoding and consulting `ding.fontmap`. Without it,
  // `\XSolidBrush` (`\@chooseSymbol{'045}`) fell through to the OT1
  // fallback and silently emitted that slot's TEXT character, `%`
  // (witness 2503.04421: 28 result-table cells read `%`/`!` instead of
  // ✗/✓). Same shape as Perl's own comment: this is a hack, but it is the
  // hack Perl commits to.
  DefPrimitive!("\\selectfont", {
    let family = Expand!(T_CS!("\\f@family")).to_string();
    let series = Expand!(T_CS!("\\f@series")).to_string();
    let shape = Expand!(T_CS!("\\f@shape")).to_string();
    if let Some(sh) = font::lookup_font_family(&family) {
      MergeFont!(sh.clone());
    } else if load_font_map(&family).is_some() {
      // Special case hack: Tentatively treat family as the encoding!
      // (typically "U" encoding)
      MergeFont!(encoding => family);
    } else if !already_reported(&s!("reported_unrecognized_font_family_{family}")) {
      let message = s!("Unrecognized font family {:?}.", family);
      Info!("unexpected", family, message);
    }
    if let Some(sh) = font::lookup_font_series(&series) {
      MergeFont!(sh.clone());
    } else if !already_reported(&s!("reported_unrecognized_font_series_{series}")) {
      let message = s!("Unrecognized font series {:?}.", series);
      Info!("unexpected", series, message);
    }
    if let Some(sh) = font::lookup_font_shape(&shape) {
      MergeFont!(sh.clone());
    } else if !already_reported(&s!("reported_unrecognized_font_shape_{shape}")) {
      let message = s!("Unrecognized font shape {:?}.", shape);
      Info!("unexpected", shape, message);
    }
    Ok(Vec::new())
  });

  DefMacro!(
    "\\usefont{}{}{}{}",
    "\\fontencoding{#1}\\fontfamily{#2}\\fontseries{#3}\\fontshape{#4}\\selectfont"
  );

  // If these series or shapes appear in math, they revert it to roman, medium, upright (?)
  DefConstructor!("\\textmd@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { series => "medium" }, alias => "\\textmd",
    before_digest => { DefMacro!("\\f@series", "m"); });
  DefConstructor!("\\textbf@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { series => "bold" }, alias => "\\textbf",
    before_digest => { DefMacro!("\\f@series", "b"); });
  DefConstructor!("\\textrm@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>",
    mode => "text", bounded => true, font => { family => "serif" }, alias => "\\textrm",
    before_digest => { DefMacro!("\\f@family", "cm"); });
  DefConstructor!("\\textsf@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { family => "sansserif" }, alias => "\\textsf",
    before_digest => { DefMacro!("\\f@family", "cmss"); });
  DefConstructor!("\\texttt@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { family => "typewriter" }, alias => "\\texttt",
    before_digest => { DefMacro!("\\f@family", "cmtt"); });
  DefConstructor!("\\textup@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "upright" }, alias => "\\textup",
    before_digest => { DefMacro!("\\f@shape", ""); });
  DefConstructor!("\\textit@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "italic" }, alias => "\\textit",
    // Perl: `DefMacro('\f@shape', 'it')` (latex_constructs.pool.ltxml L5255) —
    // `it`, not `i`. Both keys map to `italic` in FONT_SHAPE, so this is only
    // visible to code that reads `\f@shape` textually; the sibling
    // `\textsl@math`/`\textsc@math` already use Perl's `sl`/`sc`.
    before_digest => { DefMacro!("\\f@shape", "it"); });
  DefConstructor!("\\textsl@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "slanted" }, alias => "\\textsl",
    before_digest => { DefMacro!("\\f@shape", "sl"); });
  DefConstructor!("\\textsc@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode => "text",
    bounded      => true, font => { shape => "smallcaps" }, alias => "\\textsc",
    before_digest => { DefMacro!("\\f@shape", "sc"); });
  DefConstructor!("\\textnormal@math{}", "<ltx:text _noautoclose='1'>#1</ltx:text>", mode =>
  "text",   bounded => true, font => { family => "serif", series => "medium", shape => "upright"
  }, alias => "\\textnormal",   before_digest => {
    DefMacro!("\\f@family", "cmtt");
    DefMacro!("\\f@series", "m");
    DefMacro!("\\f@shape",  "n"); });

  // These really should be robust! which is a source of expand timing issues!
  DefMacro!("\\textmd{}",     "\\ifmmode\\textmd@math{#1}\\else{\\mdseries #1}\\fi",       protected => true);
  DefMacro!("\\textbf{}",     "\\ifmmode\\textbf@math{#1}\\else{\\bfseries #1}\\fi",       protected => true);
  DefMacro!("\\textrm{}",     "\\ifmmode\\textrm@math{#1}\\else{\\rmfamily #1}\\fi",       protected => true);
  DefMacro!("\\textsf{}",     "\\ifmmode\\textsf@math{#1}\\else{\\sffamily #1}\\fi",       protected => true);
  DefMacro!("\\texttt{}",     "\\ifmmode\\texttt@math{#1}\\else{\\ttfamily #1}\\fi",       protected => true);
  DefMacro!("\\textup{}",     "\\ifmmode\\textup@math{#1}\\else{\\upshape #1}\\fi",        protected => true);
  DefMacro!("\\textit{}",     "\\ifmmode\\textit@math{#1}\\else{\\itshape #1}\\fi",        protected => true);
  DefMacro!("\\textsl{}",     "\\ifmmode\\textsl@math{#1}\\else{\\slshape #1}\\fi",        protected => true);
  DefMacro!("\\textsc{}",     "\\ifmmode\\textsc@math{#1}\\else{\\scshape #1}\\fi",        protected => true);
  DefMacro!("\\textnormal{}", "\\ifmmode\\textnormal@math{#1}\\else{\\normalfont #1}\\fi", protected => true);

  // Perl: latex_constructs.pool.ltxml line 5365
  // \DeclareOldFontCommand{\cmd}{text-font-switch}{math-font-cmd}
  // Defines \cmd to use text-font-switch in text mode, math-font-cmd in math mode.
  DefPrimitive!("\\DeclareOldFontCommand{}{}{}", sub[(cmd, font, mathcmd)] {
    // cmd contains a CS token like \bf; get the first token
    let cmd_cs = *cmd.unlist_ref().first()
      .ok_or("DeclareOldFontCommand: expected a CS token")?;
    // Move `font` and `mathcmd` directly into the closure capture —
    // they're not used outside. Saves two Tokens clones at setup time.
    DefMacro!(cmd_cs, None, ExpansionBody::Closure(Rc::new(move |_args| {
      if lookup_bool_sym(pin!("IN_MATH")) {
        Ok(mathcmd.clone())
      } else {
        Ok(font.clone())
      }
    })));
    Ok(Vec::new())
  });

  // Perl L5428-5434: \DeclareTextFontCommand defines the command as a
  // CONSTRUCTOR (non-expandable), digesting the font argument at
  // digestion time in beforeDigest:
  //   DefConstructorI($cmd, "{}",
  //     "?#isMath(<ltx:text _noautoclose='1'>#1</ltx:text>)(#1)",
  //     mode => 'text', bounded => 1, beforeDigest => sub { Digest($font); () });
  // It is CRITICAL that this be a constructor, not an expandable macro
  // expanding to `{<font> #1}`. natbib's `\lx@NAT@parselabel` `Expand!`s
  // bibitem labels for-execution (full expansion). An expandable
  // `\textcyr{…}` → `{\cyrfamily …}` would, during that expansion, run
  // `\cyrfamily`→`\cyracc`, whose body ends with
  // `\def\!{…\def\result{\@stressit}\fi\result}`; for-execution expansion
  // of a `\def`'s replacement-text wrongly expands that body, invoking
  // `\result`→`\@stressit`→`\futurelet\chartest\@stresschar`, which loops
  // until the 650000 PushbackLimit fires (FATAL). As a constructor (the
  // Perl form) `Expand!` leaves `\textcyr{…}` intact — the font is digested
  // only when the constructor is actually digested. Witness 1803.11541
  // (Cyrillic bibliography: amsfonts `cyracc.def` + `\DeclareTextFontCommand`
  // `\textcyr` + natbib `\bibitem[{\textcyr{…\u\i…}}(1906)]{key}`); Perl
  // converts it (282 KB), Rust looped to FATAL before this fix.
  DefPrimitive!("\\DeclareTextFontCommand DefToken {}", sub[(cmd, font)] {
    let cs = cmd;
    let font_toks: Tokens = font;
    let params = parse_parameters("{}", &cs, true)?;
    DefConstructor!(cs, params,
      "?#isMath(<ltx:text _noautoclose='1'>#1</ltx:text>)(#1)",
      mode => "text", bounded => true,
      before_digest => { Digest!(font_toks.clone())?; });
  });

  // Perl L5373: \newfont{cmd}{fontname} — legacy LaTeX font command
  DefMacro!("\\newfont{}{}", "\\font#1=#2\\relax");
  // Perl L5375: \normalcolor — default no-op (overridden by color.sty)
  Let!("\\normalcolor", "\\relax");

  // Perl L5364: \math@version default
  DefMacro!("\\math@version", "normal");

  // Perl latex_constructs.pool.ltxml L5290-5297: \mathversion switches the
  // MATHFONT — `AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold
  // => N), 'local')` — exactly as \boldmath/\unboldmath do (plain_base.rs
  // L748-762). It was using `MergeFont!`, which merges the current *text* `font`
  // value, so `\mathversion{bold}` never reached the math font; and an unknown
  // version fell to `_ => {}`, silently swallowed where Perl raises
  // `Error('unexpected', …)`. Both now match Perl. (\mathversion has no
  // `forbidMath`, unlike \boldmath/\unboldmath — Perl doesn't either.)
  DefPrimitive!("\\mathversion{}", sub[(version)] {
    let set_forcebold = |bold: bool| {
      let mf = lookup_mathfont().unwrap_or_else(|| Rc::new(Font::math_default()));
      let merged = mf.merge(Font { forcebold: Some(bold), ..Font::default() });
      assign_value("mathfont", Stored::Font(Rc::new(merged)), Some(Scope::Local));
    };
    match version.to_string().trim() {
      "bold" => set_forcebold(true),
      "normal" => set_forcebold(false),
      // a version registered by `\DeclareMathVersion` (sect08.rs) is
      // selectable; it changes no font here
      other if lookup_bool(&s!("MATH_VERSION_{other}")) => {},
      other => { Error!("unexpected", other, s!("Unknown math version '{other}'")); },
    }
  });

  //======================================================================
  // C.15.3 Special Symbol
  //======================================================================
  DefMacro!("\\symbol{}", "\\char#1\\relax");

  // These in LaTeX, but not in the book...
  DefPrimitive!("\\textdollar", "$");
  DefPrimitive!("\\textemdash", "\u{2014}"); // EM DASH
  DefPrimitive!("\\textendash", "\u{2013}"); // EN DASH
  DefPrimitive!("\\textexclamdown", "\u{00A1}"); // INVERTED EXCLAMATION MARK
  DefPrimitive!("\\textquestiondown", "\u{00BF}"); // INVERTED QUESTION MARK
  DefPrimitive!("\\textquotedblleft", "\u{201C}"); // LEFT DOUBLE QUOTATION MARK
  DefPrimitive!("\\textquotedblright", "\u{201D}"); // RIGHT DOUBLE QUOTATION MARK
  DefPrimitive!("\\textquotedbl", "\""); // plain ascii DOUBLE QUOTATION
  DefPrimitive!("\\textquoteleft", "\u{2018}"); // LEFT SINGLE QUOTATION MARK
  DefPrimitive!("\\textquoteright", "\u{2019}"); // RIGHT SINGLE QUOTATION MARK
  DefPrimitive!("\\textsterling", "\u{00A3}"); // POUND SIGN
  DefPrimitive!("\\textasteriskcentered", "*");
  DefPrimitive!("\\textbackslash", "\u{005C}"); // REVERSE SOLIDUS
  DefPrimitive!("\\textbar", "|");
  DefPrimitive!("\\textbraceleft", "{");
  DefPrimitive!("\\textbraceright", "}");
  DefPrimitive!("\\textbullet", "\u{2022}"); // BULLET
  DefPrimitive!("\\textdaggerdbl", "\u{2021}"); // DOUBLE DAGGER
  DefPrimitive!("\\textdagger", "\u{2020}"); // DAGGER
  DefPrimitive!("\\textparagraph", "\u{00B6}"); // PILCROW SIGN
  // \textperiodcentered defined in `latex_constructs_rust_only.rs` (Perl
  // uses it but never defines it — Perl-side gap noted in rust_only.rs).
  DefPrimitive!("\\textsection", "\u{00A7}"); // SECTION SIGN
  // Perl: DefPrimitive('\textcircled {}', sub { ... })
  // Uses unicode_enclosed_alphanumerics table, falls back to combining circle U+20DD
  DefPrimitive!("\\textcircled {}", sub[(arg)] {
    let text = arg.to_string();
    let content = unicode_enclosed_alphanumeric(&text)
      .unwrap_or_else(|| format!("{}\u{20DD}", text));
    let in_math = lookup_bool_sym(pin!("IN_MATH"));
    let is_number = !text.is_empty() && text.chars().all(|c| c.is_ascii_digit());
    let mut props = stored_map!();
    if in_math {
      props.insert("role", Stored::from(if is_number { "NUMBER" } else { "UNKNOWN" }));
      props.insert("meaning", Stored::from(format!("circled-{}", text)));
    }
    Tbox::new(pin(&content), None, None,
      Invocation!(T_CS!("\\textcircled"), vec![arg]),
      props)
  });
  // From latex_constructs.pool.ltxml
  DefAccent!("\\k", '\u{0328}', "\u{02DB}", below => true); // COMBINING OGONEK & OGONEK
  DefPrimitive!("\\textless", "<");
  DefPrimitive!("\\textgreater", ">");
  DefPrimitive!("\\textcopyright", "\u{00A9}"); // COPYRIGHT SIGN
  DefPrimitive!("\\textasciicircum", "^");
  DefPrimitive!("\\textasciitilde", "~");
  DefPrimitive!("\\textcompwordmark", ""); // ???
  DefPrimitive!("\\textcapitalcompwordmark", ""); // ???
  DefPrimitive!("\\textascendercompwordmark", ""); // ???
  DefPrimitive!("\\textunderscore", "_");
  // SYMBOL FOR SPACE;  Not really the right symbol!
  DefPrimitive!("\\textvisiblespace", "\u{2423}");
  DefPrimitive!("\\textellipsis", "\u{2026}"); // HORIZONTAL ELLIPSIS
  DefPrimitive!("\\textregistered", "\u{00AE}"); // REGISTERED SIGN
  DefPrimitive!("\\texttrademark", "\u{2122}"); // TRADE MARK SIGN
  DefConstructor!("\\textsuperscript{}", "<ltx:sup>#1</ltx:sup>",  mode => "text");
  // Perl L5424-5425: locked variant for \@makefnmark
  DefConstructor!("\\@textsuperscript{}", "<ltx:sup>#1</ltx:sup>",
    mode => "text", locked => true);
  DefConstructor!("\\textsubscript{}", "<ltx:sub>#1</ltx:sub>",  mode => "text");
  // This is something coming from xetex/xelatex ? Why define this way?
  //DefConstructor!("\\realsuperscript{}', "<ltx:text yoffset='0.5em'
  // _noautoclose='1'>#1</ltx:text>");
  DefConstructor!("\\realsuperscript{}", "<ltx:sup>#1</ltx:sup>",  mode => "text");
  DefPrimitive!("\\textordfeminine", "\u{00AA}"); // FEMININE ORDINAL INDICATOR
  DefPrimitive!("\\textordmasculine", "\u{00BA}"); // MASCULINE ORDINAL INDICATOR
  DefPrimitive!("\\SS", "SS"); // ?

  DefMacro!("\\dag", "\\ifmmode{\\dagger}\\else\\textdagger\\fi");
  DefMacro!("\\ddag", "\\ifmmode{\\ddagger}\\else\\textdaggerdbl\\fi");

  // Real LaTeX (latex.ltx) defines `\sqrtsign` as a MACRO: `\def\sqrtsign{\radical"270370\relax}`.
  // `\meaning\sqrtsign` is therefore `macro:->\radical "270370\relax ` — note the TWO catcode-12
  // backslashes (`\radical`, `\relax`). mdwmath's catcode-tricky `\sq@readrad #1"#2\#3` parses
  // exactly this: #1 up to `"` = `macro:->\radical `, #2 up to the `\` of `\relax` = `270370`,
  // #3 up to `\relax`. LaTeXML had `\sqrtsign` as a 1-arg square-root *constructor*, whose meaning
  // lacked that structure → `\sq@readrad` ran away consuming `\endgroup`, leaving `\` at catcode-12
  // and corrupting every later `\def` (43 `#`-to-Stomach leaks; ~1080 canvas papers; SHARED w/ Perl).
  // Match real LaTeX. `\sqrt` does its own construction; nothing in core calls `\sqrtsign{…}`.
  TeX!(r#"\def\sqrtsign{\radical"270370\relax}"#);

  DefPrimitive!("\\mathparagraph", "\u{00B6}");
  DefPrimitive!("\\mathsection", "\u{00A7}");
  DefPrimitive!("\\mathdollar", "$");
  DefPrimitive!("\\mathsterling", "\u{00A3}");
  DefPrimitive!("\\mathunderscore", "_");
  DefPrimitive!("\\mathellipsis", "\u{2026}");

  // Perl: plain_constructs.pool.ltxml — glyph pieces that also work as delimiters
  DefMath!("\\arrowvert", None, "|", role => "VERTBAR");
  DefMath!("\\Arrowvert", None, "\u{2225}", role => "VERTBAR");

  // The following are glyph "pieces"...
  DefPrimitive!("\\braceld", "\u{239D}"); //   left brace down part
  DefPrimitive!("\\bracelu", "\u{239B}"); //   left brace up part
  DefPrimitive!("\\bracerd", "\u{23A0}"); //   right brace down part
  DefPrimitive!("\\braceru", "\u{239E}"); //   right brace up part

  // Perl: plain_constructs.pool.ltxml
  DefMath!("\\cdotp", None, "\u{22C5}", role => "MULOP");
  DefMath!("\\ldotp", None, ".", role => "MULOP");
  // Perl: latex_constructs.pool.ltxml — intop/ointop with dynamic scriptpos/mathstyle
  DefMath!("\\intop", None, "\u{222B}", role => "INTOP", meaning => "integral",
    dynamic_scriptpos => true, dynamic_mathstyle => true);
  DefMath!("\\ointop", None, "\u{222E}", role => "INTOP", meaning => "contour-integral",
    dynamic_scriptpos => true, dynamic_mathstyle => true);

  // WHat are these? They look like superscripted parentheses, or combining accents!
  // \lhook
  // \rhook
  Let!("\\gets", "\\leftarrow");

  DefPrimitive!("\\lmoustache", "\u{23B0}");
  DefPrimitive!("\\rmoustache", "\u{23B1}");
  // Perl: plain_constructs.pool.ltxml
  DefMath!("\\mapstochar", None, "\u{21A6}", role => "ARROW", meaning => "maps-to");
  DefMath!("\\owns", None, "\u{220B}", role => "RELOP", meaning => "contains");

  // \symbol lookup symbol in font by index?

  // Perl: latex_constructs.pool.ltxml L5805
  Let!("\\mathalpha", "\\relax");

  // Perl latex_constructs.pool.ltxml L5937-5938:
  // LaTeX now includes textcomp by default.
  RequirePackage!("textcomp");

  //======================================================================
  // Perl latex_constructs.pool.ltxml L5941-5993: Case-changing
  //======================================================================

  DefMacro!(
    "\\@uclclist",
    r"\oe\OE\o\O\ae\AE\dh\DH\dj\DJ\l\L\ng\NG\ss\SS\th\TH"
  );

  DefPrimitive!("\\lx@prepare@case@mapping", {
    assign_mapping("text_uppercase", "\\i ", Some(T_LETTER!("I")));
    assign_mapping("text_uppercase", "\\j ", Some(T_LETTER!("J")));
    // Perl (latex_constructs.pool L5546-5550):
    //   my @pairs = $STATE->lookupDefinition(T_CS('\@uclclist'))
    //                     ->getExpansion->unlist;
    // — reads the RAW expansion body, NOT further expanded. Critical when
    // the pair members (\ae, \oe, ...) are robust-wrapped: deep-expanding
    // would unfold each to `\protect <cs-munged>`, shifting pair indices
    // and mis-registering the case mapping.
    let pairs: Vec<Token> = match lookup_definition_stored(&T_CS!("\\@uclclist"))? {
      Some(Stored::Expandable(exp)) => match exp.get_expansion() {
        Some(ExpansionBody::Tokens(tks)) => tks.clone().unlist(),
        _ => Vec::new(),
      },
      _ => Vec::new(),
    };
    let mut i = 0;
    while i + 1 < pairs.len() {
      let lower = pairs[i];
      let upper = pairs[i + 1];
      let lower_key = lower.with_str(|s| format!("{} ", s));
      let upper_key = upper.with_str(|s| format!("{} ", s));
      assign_mapping("text_uppercase", &lower_key, Some(upper));
      assign_mapping("text_lowercase", &upper_key, Some(lower));
      i += 2;
    }
  });

  DefPrimitive!("\\AddToNoCaseChangeList DefToken", sub[(cs)] {
    let key = cs.with_str(|s| s.trim_end().to_string());
    assign_mapping("text_case_exclude", &key, Some(true));
  });

  DefMacro!("\\NoCaseChange {}", "#1", robust => true);

  DefMacro!("\\lx@latex@changecase {} GeneralText", sub[(case, tokens)] {
    let req_case = Expand!(case).to_string().to_lowercase();
    Ok(Tokens::new(lx_change_case_tokens(&req_case, &tokens)?))
  });

  TeX!(
    r"\AddToNoCaseChangeList{\NoCaseChange}%
\AddToNoCaseChangeList{\label}%
\AddToNoCaseChangeList{\ref}%
\AddToNoCaseChangeList{\cite}%
\AddToNoCaseChangeList{\ensuremath}%
\AddToNoCaseChangeList{\@ensuremath}%
\AddToNoCaseChangeList{\thanks}%"
  );

  // Perl L5966-5993: \MakeUppercase, \MakeLowercase, \MakeTitlecase
  // Pre-define the UTF@*octets@noexpand CSes that the bodies below
  // unconditionally `\let` to `\@empty`. Without these the `\edef`
  // partial-expansion auto-defines them as `<ltx:ERROR/>` (unexpected
  // for a guard meant to prevent expansion within case-change). Real
  // TeX's `\let<undef>\@empty` is a no-op without error; mirror that
  // by stubbing them as `\@empty` ahead of `\edef`. inputenc.sty
  // overrides these when utf8 encoding is active.
  Let!("\\UTF@two@octets@noexpand", "\\@empty");
  Let!("\\UTF@three@octets@noexpand", "\\@empty");
  Let!("\\UTF@four@octets@noexpand", "\\@empty");
  TeX!(
    r"\DeclareRobustCommand{\MakeUppercase}[1]{{%
  \lx@prepare@case@mapping%
  \def\({$}\let\)\(%
  \def\i{I}\def\j{J}%
  \let\UTF@two@octets@noexpand\@empty
  \let\UTF@three@octets@noexpand\@empty
  \let\UTF@four@octets@noexpand\@empty
  \edef\reserved@a{\lx@latex@changecase{upper}{#1}}%
  \reserved@a
}}
\DeclareRobustCommand{\MakeLowercase}[1]{{%
  \lx@prepare@case@mapping%
  \def\({$}\let\)\(%
  \let\UTF@two@octets@noexpand\@empty
  \let\UTF@three@octets@noexpand\@empty
  \let\UTF@four@octets@noexpand\@empty
  \edef\reserved@a{\lx@latex@changecase{lower}{#1}}%
  \reserved@a
}}
\DeclareRobustCommand{\MakeTitlecase}[1]{{%
  \lx@prepare@case@mapping%
  \def\({$}\let\)\(%
  \let\UTF@two@octets@noexpand\@empty
  \let\UTF@three@octets@noexpand\@empty
  \let\UTF@four@octets@noexpand\@empty
  \edef\reserved@a{\lx@latex@changecase{sentence}{#1}}%
  \reserved@a
}}
\protected@edef\MakeUppercase#1{\MakeUppercase{#1}}
\protected@edef\MakeLowercase#1{\MakeLowercase{#1}}
\protected@edef\MakeTitlecase#1{\MakeTitlecase{#1}}"
  );

  // Perl L5913,5916: fixltx2e defaults
  DefMacro!("\\eminnershape", None, None);
  DefMacro!("\\TextOrMath{}{}", "\\ifmmode#2\\else#1\\fi");

  //======================================================================
  // Semi-undocumented commands
  // Perl: latex_constructs.pool.ltxml various locations
  //======================================================================

  // Hacky version matches multiple chars! but does NOT expand
  DefMacro!("\\@ifnext@n {}{}{}", sub[(tokens,if_toks,else_toks)] {
    let mut toks = VecDeque::from(tokens.unlist());
    let mut read = Vec::new();

    while let Some(t) = read_token()? {
      // Stop as soon as we've matched the full token sequence —
      // otherwise the `toks[0]` index panics on the next iteration
      // (arxiv 1608.08252 hit this with a matching prefix followed
      // by arbitrary tokens in the stream).
      if toks.is_empty() {
        read.push(t);
        break;
      }
      if t == toks[0] {
        toks.pop_front();
        read.push(t);
      } else {
        read.push(t);
        break;
      }
    }
    let mut result = if toks.is_empty() {
      if_toks.unlist()
    } else {
      else_toks.unlist()
    };
    // The matched/peeked tokens were read (brace-counted) and re-enter via
    // our expansion: retract (tex.web back_input flavor).
    retract_scanned_braces(&read);
    result.extend(read);
    Ok(Tokens::new(result))
  });

  DefMacro!("\\@ifstar {}{}", sub[(if_toks,else_toks)] {
    let next_opt = read_non_space()?;
    if next_opt == Some(T_OTHER!("*")) {
      Ok(if_toks)
    } else {
      let mut result = else_toks.unlist();
      if let Some(next) = next_opt {
        // `next` was read (brace-counted) and re-enters via our expansion:
        // retract, tex.web back_input flavor (see gullet::retract_scanned_braces).
        retract_scanned_brace(&next);
        result.push(next);
      }
      Ok(Tokens::new(result))
    }
  });

  DefMacro!("\\@dblarg {}", r"\kernel@ifnextchar[{#1}{\@xdblarg{#1}}");
  DefMacro!("\\@xdblarg {}{}", r"#1[{#2}]{#2}");

  DefMacro!("\\@testopt{}{}", sub[(cmd, option)] {
    if if_next(T_OTHER!("["))? {
      Ok(cmd)
    } else {
      Ok(Tokens!(cmd.unlist(), T_OTHER!("["), option.unlist(), T_OTHER!("]")))
    }
  });
  TeX!(
    r"
  \def\@protected@testopt#1{%%
    \ifx\protect\@typeset@protect
      \expandafter\@testopt
    \else
      \@x@protect#1%
    \fi}"
  );

  Let!("\\l@ngrel@x", "\\relax");
  DefMacro!(
    "\\@star@or@long{}",
    r"\@ifstar{\let\l@ngrel@x\relax#1}{\let\l@ngrel@x\long#1}"
  );

  TeX!(
    r"
  \def\in@#1#2{%
  \def\in@@##1#1##2##3\in@@{%
    \ifx\in@##2\in@false\else\in@true\fi}%
  \in@@#2#1\in@\in@@}
  \newif\ifin@"
  );

  DefMacro!("\\IfFileExists{}{}{}", sub[(file, if_tks, else_tks)] {
    let file_string = Expand!(file).to_string();
    // latex.ltx:19794 file substitution applies before the existence test.
    let file_string = substitute_file_request(&file_string).unwrap_or(file_string);
    // Disk-search variant first (matches Perl FindFile default).
    let found = find_file(&file_string, None).is_some()
      // Then binding-only fallback (notex=true): pgf's
      // `\IfFileExists{pgfsys-latexml.def}` and similar driver-file probes
      // need to discover compiled-in bindings (.def/.sty/.cls registered
      // via `latexml_package::package`). Without this fallback the
      // notex=false disk search returns None for binding-only files,
      // pgf bails with "Driver file ... not found", and the entire
      // tikz/pgf rendering pipeline fails (11 PGF/tikz tests + 10k_sandbox
      // pgf-using papers).
      || find_file(&file_string, Some(FindFileOptions {
          notex: true, ..Default::default()
        })).is_some();
    // latex.ltx:9670 `\IfFileExists@` re-`\def`s the selected branch
    // (`\expandafter\def\expandafter\reserved@a\expandafter{\reserved@a{#2}{#3}}`),
    // so a `\def\x##1{…}` written inside it is scanned as a replacement text
    // and its `##` halves once (tex.web §473-476). Returning the branch
    // verbatim (Perl latex_constructs.pool.ltxml:5692-5697 too) left the
    // doubled `#` in `\react@`'s body — chemexec.sty:274-289, 20 PARAM
    // errors per manual. `pack_parameters` is that halving.
    // Guard: `perfect_kernel_batch56::iffileexists_branch_halves_doubled_parameters`.
    if found {
      let found_str = s!("\"{file_string}\" ");
      def_macro(T_CS!("\\@filef@und"), None, Some(found_str.into()), None)?;
      if_tks.pack_parameters()?
    } else {
      else_tks.pack_parameters()?
    }
  });

  // \IfFormatAtLeastTF stub lives in `latex_constructs_rust_only.rs` (loads
  // last). Removed duplicate identical-body DefMacro here.

  DefMacro!("\\InputIfFileExists{}{}{}", sub[(file, if_tks, else_tks)] {
    let file_tks = Expand!(file);
    let file_string = file_tks.to_string();
    // latex.ltx:19794 file substitution applies before the existence test;
    // `input()` re-applies it to the name we hand `\ltx@input`.
    let file_string = substitute_file_request(&file_string).unwrap_or(file_string);
    // Disk-search first (matches Perl FindFile default), then
    // binding-only fallback (notex=true) so registered .ldf / .def
    // bindings shipped via latexml_package are discoverable. Without
    // the fallback, babel.sty L4175 `\InputIfFileExists{italian.ldf}`
    // returns "not found" even though we ship `italian.ldf` as a
    // compile-time binding — and babel errors with "Unknown option
    // 'italian'". Witness: 38 papers across recent stages with
    // missing-on-disk babel-language packages (italian/spanish/
    // portuges/brazil/...).
    let found = find_file(&file_string, None).is_some()
      || find_file(&file_string, Some(FindFileOptions {
          notex: true, ..Default::default()
        })).is_some();
    if found {
      let found_str = s!("\"{file_string}\" ");
      def_macro(T_CS!("\\@filef@und"), None, Some(found_str.into()), None)?;
      Tokens!(if_tks.pack_parameters()?, T_CS!("\\@addtofilelist"), T_BEGIN!(), file_tks.clone(), T_END!(),
        T_CS!("\\ltx@input"), T_BEGIN!(), file_tks, T_END!())
    } else {
      else_tks.pack_parameters()?
    }
  });

  DefMacro!("\\@ifdefinable DefToken {}", sub[(token, iftoken)] {
    if is_definable(&token) {
      iftoken.unlist()
    } else {
      let token_str = token.to_string();
      let mut s = ExplodeText!(token_str);
      if token_str.starts_with('\\') {
        s.remove(0);
      }
      DefMacro!(T_CS!("\\reserved@a"), None, Tokens::new(s));
      vec![T_CS!("\\@notdefinable")]
    }
  });

  Let!("\\@@ifdefinable", "\\@ifdefinable");

  DefMacro!("\\@rc@ifdefinable DefToken {}", sub[(_token, iftoken)] {
    Let!("\\@ifdefinable", "\\@@ifdefinable");
    iftoken.unlist()
  });

  DefMacro!(
    "\\@notdefinable",
    None,
    r###"\@latex@error{%
    Command \@backslashchar\reserved@a\space
    already defined.
    Or name \@backslashchar\@qend... illegal, see p.192 of the manual}
  "###
  );

  // Sundry
  // Perl latex_constructs.pool L5771: DefPrimitiveI('\textprime', undef, UTF(0xB4))
  DefPrimitive!("\\textprime", "\u{00B4}"); // ACUTE ACCENT
  Let!("\\endgraf", "\\par");
  Let!("\\endline", "\\cr");
  def_macro_noop("\\fileversion")?;
  def_macro_noop("\\filedate")?;
  DefMacro!("\\chaptername", "Chapter");
  DefMacro!("\\partname", "Part");
  // \appendixname already defined earlier in this file (DefMacro `Appendix` at the
  // C.4.4 appendix block); avoid duplicate identical re-definition.
  DefMacro!(
    "\\sectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subsectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subsubsectiontyperefname",
    "\\lx@sectionsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\paragraphtyperefname",
    "\\lx@paragraphsign\\lx@ignorehardspaces"
  );
  DefMacro!(
    "\\subparagraphtyperefname",
    "\\lx@paragraphsign\\lx@ignorehardspaces"
  );

  //======================================================================
  // Perl latex_constructs.pool.ltxml L5796-5800: aux file stubs
  //======================================================================
  def_macro_noop("\\bibdata{}")?;
  def_macro_noop("\\bibcite{}{}")?;
  def_macro_noop("\\citation{}")?;
  def_macro_noop("\\contentsline{}{}{}")?;
  def_macro_noop("\\newlabel{}{}")?;

  // Perl L5804-5806
  Let!("\\mathgroup", "\\fam");

  // Perl L5808-5821: nocorr, text@command, check@ic stubs
  DefMacro!("\\nocorrlist", None, ".,");
  Let!("\\nocorr", "\\relax");
  Let!("\\check@icl", "\\@empty");
  Let!("\\check@icr", "\\@empty");
  Let!("\\curr@math@size", "\\@empty");
  def_macro_noop("\\text@command{}")?;
  def_macro_noop("\\check@nocorr@ Until:\\nocorr Until:\\@nil")?;
  TeX!("\\newif\\ifmaybe@ic");
  DefMacro!("\\maybe@ic", None, None);
  DefMacro!("\\maybe@ic@", None, None);
  DefMacro!("\\sw@slant", None, None);
  DefMacro!("\\fix@penalty", None, None);

  // Perl L5814: \mathhexbox
  DefPrimitive!("\\mathhexbox {}{}{}", sub[(a, b, c)] {
    let n = a.to_string().trim().parse::<i32>().unwrap_or(0) * 256
      + b.to_string().trim().parse::<i32>().unwrap_or(0) * 16
      + c.to_string().trim().parse::<i32>().unwrap_or(0);
    let (glyph, _font) = font_decode(n, None, None);
    if let Some(ch) = glyph {
      vec![Tbox::new(pin_char(ch), None, None, Tokens!(), SymHashMap::default()).into()]
    } else {
      Vec::new()
    }
  });

  // \extrafloats (modern LaTeX 2015+; no Perl pool source) lives in
  // `latex_constructs_rust_only.rs`. Identical-body duplicate removed here.

  //======================================================================
  // Perl latex_constructs.pool.ltxml L5836-5886: language declarations
  // Pre-declare hyphenation languages for babel's \iflanguage checks
  //======================================================================
  TeX!(
    r"\newlanguage\l@english
\newlanguage\l@usenglishmax
\newlanguage\l@USenglish
\newlanguage\l@dumylang
\newlanguage\l@nohyphenation
\newlanguage\l@arabic
\newlanguage\l@basque
\newlanguage\l@bulgarian
\newlanguage\l@coptic
\newlanguage\l@welsh
\newlanguage\l@czech
\newlanguage\l@slovak
\newlanguage\l@german
\newlanguage\l@ngerman
\newlanguage\l@danish
\newlanguage\l@esperanto
\newlanguage\l@spanish
\newlanguage\l@catalan
\newlanguage\l@galician
\newlanguage\l@estonian
\newlanguage\l@farsi
\newlanguage\l@finnish
\newlanguage\l@french
\newlanguage\l@greek
\newlanguage\l@monogreek
\newlanguage\l@ancientgreek
\newlanguage\l@croatian
\newlanguage\l@hungarian
\newlanguage\l@interlingua
\newlanguage\l@ibycus
\newlanguage\l@indonesian
\newlanguage\l@icelandic
\newlanguage\l@italian
\newlanguage\l@latin
\newlanguage\l@mongolian
\newlanguage\l@dutch
\newlanguage\l@norsk
\newlanguage\l@polish
\newlanguage\l@portuguese
\newlanguage\l@pinyin
\newlanguage\l@romanian
\newlanguage\l@russian
\newlanguage\l@slovenian
\newlanguage\l@uppersorbian
\newlanguage\l@serbian
\newlanguage\l@swedish
\newlanguage\l@turkish
\newlanguage\l@ukenglish
\newlanguage\l@ukrainiane"
  );

  // Perl latex_constructs: \protected@write
  DefPrimitive!("\\protected@write{Number}{}{}", sub[(_port, prelude, _tokens)] {
    bgroup();
    Let!("\\thepage", "\\relax");
    let _digested = digest(prelude)?;
    egroup()?;
  });

  // \@@end — saved TeX \end primitive
  DefPrimitive!("\\@@end", {
    if !lookup_bool_sym(pin!("INTERPRETING_DEFINITIONS")) {
      flush();
    }
  });

  //======================================================================
  // Closure-backed primitives — Perl: latex_constructs.pool.ltxml L5645-5766.
  // These MUST live in `_constructs` (always loaded), not `_base` (optional
  // under Perl's LoadFormat mutual-exclusivity). Their closures cannot be
  // serialized into the kernel dump; defining them here guarantees they
  // exist whether or not the dump short-circuits `_base`.
  //
  // Relocated from `latex_base.rs` 2026-04-18 for Perl-parity and to
  // unblock `LATEXML_DUMP_ONLY=1` paths (see SYNC_STATUS D0 v3.f).

  // Perl L5645
  DefPrimitive!("\\@onlypreamble{}", {
    only_preamble("\\@onlypreamble")?;
  });

  // Perl L5646-5648
  DefPrimitive!("\\GenericError{}{}{}{}", sub[(arg1,arg2,arg3,arg4)] {
    // Perl passes ALL four args; make_generic_message peels the first as
    // the message $type (the `(pkgname)` prefix) and joins the rest.
    make_generic_message("\\GenericError", vec![arg1, arg2, arg3, arg4], "error")?;
  });
  DefPrimitive!("\\GenericWarning{}{}", sub[(arg1,arg2)] {
    make_generic_message("\\GenericWarning", vec![arg1,arg2], "warn")?;
  });
  DefPrimitive!("\\GenericInfo{}{}", sub[(arg1,arg2)] {
    make_generic_message("\\GenericInfo", vec![arg1,arg2], "info")?;
  });

  // `\ltx@hard@MessageBreak` is the literal newline target used by
  // `make_generic_message` to convert `\MessageBreak`-separated lines
  // in `\GenericInfo`/`\GenericWarning`/`\GenericError` messages.
  // Originally defined in `latex_base.rs:287`, but `latex_base` is
  // replaced by `latex_dump` in dump path — so the DefMacro doesn't
  // run there and `\ltx@hard@MessageBreak` is undefined. When
  // `make_generic_message` then calls `let_i(\MessageBreak,
  // \ltx@hard@MessageBreak)`, the let-target is undefined → meaning
  // becomes Stored::None → `\MessageBreak` becomes undefined for the
  // remainder of the digestion. The next babel info message
  // \ltx@hard@MessageBreak lives in `latex_constructs_rust_only.rs`
  // (not in any Perl latex_*.pool.ltxml; covers both dump and NODUMP
  // paths since rust_only.rs runs last).

  // Perl L5650 — re-let `\MessageBreak` to `\relax` here, post-dump.
  // Defensive parity with Perl's exact placement.
  Let!("\\MessageBreak", "\\relax");

  // Perl L5652 — `DefMacro` in Perl (not DefPrimitive), empty-body no-op.
  def_macro_noop("\\@setsize{}{}{}{}")?;

  // Perl L5654-5666 — kernel CSes the comment in latex_base.rs:572-575
  // promised would live here. Without these, `\on@line` (used by
  // `\@latex@error`/`\@warning` and our own `\@currenvline` block at
  // L8219) and friends are auto-defined as `<ltx:ERROR/>`, leaking
  // raw `#` parameters into the Stomach (witness 1610.05489 + ~17 more).
  DefMacro!(
    "\\hexnumber@ {}",
    r"\ifcase\number#1 0\or 1\or 2\or 3\or 4\or 5\or 6\or 7\or 8\or 9\or A\or B\or C\or D\or E\or F\fi"
  );
  DefMacro!("\\on@line", r" on input line \the\inputlineno");
  Let!("\\@warning", "\\@latex@warning");
  Let!("\\@@warning", "\\@latex@warning@no@line");
  def_macro_noop("\\G@refundefinedtrue")?;
  DefMacro!(
    "\\@nomath{}",
    r"\relax\ifmmode\@font@warning{Command \noexpand#1invalid in math mode}\fi"
  );
  DefMacro!(
    "\\@font@warning{}",
    r"\GenericWarning{(Font)\@spaces\@spaces\@spaces\space\space}{LaTeX Font Warning: #1}"
  );

  // Perl L5765-5766
  DefPrimitive!("\\makeatletter", {
    AssignCatcode!('@', Catcode::LETTER, Some(Scope::Local));
  });
  DefPrimitive!("\\makeatother", {
    AssignCatcode!('@', Catcode::OTHER, Some(Scope::Local));
  });

  // Perl L5670-5673 — font size stubs. Token-list bodies (Perl:
  // `Tokens()` = empty) that swallow their args. Relocated from
  // latex_base.rs 2026-04-18 for Perl-parity AND so they round-trip
  // through the dump under LATEXML_DUMP_ONLY=1 (the dump reader's
  // @-internal safety filter rejects public-CS macros, so public
  // kernel CSes like `\fontsize` must live in always-loaded
  // `_constructs.rs`).
  def_macro_noop("\\check@mathfonts")?;
  def_macro_noop("\\fontsize{}{}")?;
  // latex.ltx:14103-14107 guards the `\let` with `\ifx\protect\@typeset@protect`
  // so `\@setfontsize` is a no-op inside `\protected@edef`; unguarded (Perl
  // latex_constructs.pool:5622 identical, OOMs same-host) a raw class whose
  // size commands route through it (tufte-common.def:368-405) re-expanded
  // `\@currsize`→`\normalsize`→`\@setfontsize\normalsize…` without bound
  // when pgf edef'd a `font=\normalsize` label — tikz-network manual
  // PushbackLimit Fatal. `\@nomath`/`\fontsize…\selectfont` stay dropped.
  // …and `\set@fontsize` (latex.ltx:12580-12599) sets `\baselineskip` from
  // `#3` and rebuilds `\strutbox` as `.7\baselineskip`/`.3\baselineskip` —
  // the part that keeps `\strut`-based measurements honest (fillwith's line
  // coffins, see plain_base.rs `\strutbox`).
  DefMacro!(
    "\\@setfontsize{}{}{}",
    "\\ifx\\protect\\@typeset@protect\\let\\@currsize#1\\baselineskip#3\\relax\\setbox\\strutbox\\hbox{\\vrule\\@height.7\\baselineskip\\@depth.3\\baselineskip\\@width\\z@}\\fi"
  );
  // OXIDIZED_DESIGN #165: real LaTeX guarantees `\@currsize` is defined once
  // `\begin{document}` has run `\normalsize` (whose class definition routes
  // through `\@setfontsize`). Our class bindings define the size commands as
  // font primitives that never call `\@setfontsize`, so `\@currsize` stayed
  // permanently undefined (Perl 0.8.8 identical, same-host verified) and raw
  // packages that restore the surrounding size via `\@currsize` errored —
  // linguistics doc family (linguex, covington, philex, drs,
  // movement-arrows; 5 TL doc bundles). Provide the invariant as an
  // expansion-indirection default; a class that DOES route through
  // `\@setfontsize` overwrites it with the exact size command.
  DefMacro!("\\@currsize", "\\normalsize");
  // latex.ltx:18349-18350 `\ifx\@normalsize\@undefined\let\@normalsize\normalsize\fi`
  // — the size `.clo`s set it through `\@setfontsize\@normalsize…`, which the
  // font-primitive size commands here bypass (UNAMThesis under report).
  DefMacro!("\\@normalsize", "\\normalsize");

  // Perl L5687-5695 — \@ifnextchar + siblings (closure-backed).
  // Relocated from latex_base.rs 2026-04-18 to survive dump-only mode.
  DefMacro!("\\@ifnextchar DefToken {}{}", sub[(token, t_if, t_else)] {
    let next = read_non_space()?;
    let next_test = match next {
      Some(ref n) => XEquals!(&token, n),
      None => XEquals!(&token, &*TOKEN_END)
    };
    let which = if next_test { t_if } else { t_else };
    // Real \@ifnextchar re-scans BOTH branches as macro bodies
    // (`\def\reserved@a{#2}`, latex.ltx L1756-1760), which collapses `##`
    // to `#`. `substitute_parameters` passes PARAM tokens through
    // untouched (Perl Tokens.pm substituteParameters likewise — its
    // L5639 pool comment "collapsing ## pairs" is stale), so a branch
    // like `{\def\ATleftbranch##1##2{…}}` replayed the doubled params
    // and the later \def leaked `#` to the stomach (adtreesdoc ×2,
    // misdefined:# family; pdflatex clean, Perl shares the bug).
    let mut result = which
      .substitute_parameters(&[])
      .pack_parameters()?
      .unlist();
    if let Some(t_next) = next {
      // Read token re-enters via our expansion: retract its brace count
      // (tex.web back_input flavor; see gullet::retract_scanned_braces).
      retract_scanned_brace(&t_next);
      result.push(t_next);
    }
    result
  });
  Let!("\\kernel@ifnextchar", "\\@ifnextchar");

  // Re-establish the engine `\hline` override after dump load.
  // Mirrors `TeX_Tables.pool.ltxml`'s `DefMacro('\hline', '\noalign{\@@alignment@hline}')`
  // which Rust port has at `tex_tables.rs:418` (TeX.pool baseline). However:
  // CLAUDE.md "Unconditional dump apply" → the dump's M-line for `\hline`
  // (latex.ltx's `\def\hline{\noalign{...\hrule...\@xhline}}`) overwrites
  // the engine version on dump-load. The macro expansion emits a literal
  // `\hrule` inside `<td>` (Constructor at `tex_box.rs:1100`) instead of
  // setting `border="t"` on the next row, breaking ~30 tabular tests
  // (lettercase, ot1/t1/t2*/ts1/ly1, latin*, cp*, applemac, longtable,
  // array, colortbls, tabular, supertabular, morse, cells, ntheorem, …).
  //
  // This is NOT a Rust-only divergence — same definition as
  // `tex_tables.rs:418` and Perl's `TeX_Tables.pool.ltxml`. The only
  // divergence is location: pragmatic late re-install after dump-load,
  // since under unconditional-dump-apply the only way for the engine
  // override to survive is to apply it AFTER the dump has been read.
  DefMacro!("\\hline", r"\noalign{\@@alignment@hline}");

  // Re-establish the engine `\documentstyle` impl after dump load. Same
  // pattern as `\hline` above: under unconditional dump-apply the dump's
  // `\documentstyle` body — `\input{latex209.def}\documentclass` — wins
  // over the engine `Let \documentstyle = \lx@documentstyle@impl` from
  // `tex_job.rs:279`. That dump body routes everything through
  // `\documentclass`, which calls `load_class(name, opts,
  // after=\AtBeginDocument\warn@unusedclassoptions)` — DROPPING the
  // `\compat@loadpackages` after-hook our `\documentstyle` impl
  // installs. Symptom: `\documentstyle[multicol,...]{revtex}` doesn't
  // route the `multicol` option to `RequirePackage{multicol}`, leaving
  // `\multicols`/`\multicolsep` undefined. Witness: cond-mat0109091.
  //
  // The Let-restore in `tex.rs` `\@load@latex@pool` only fires when an
  // autoload trigger CS (\documentclass, \usepackage, …) is invoked
  // AFTER engine init — but `\documentstyle` is not in the trigger
  // list, so a paper whose first command is `\documentstyle` never
  // gets the restore. Doing it here, at the end of the
  // bootstrap → dump → constructs flow, guarantees our impl wins.
  Let!("\\documentstyle", "\\lx@documentstyle@impl");
  Ok(())
}

use latexml_core::common::{dimension::attribute_format, numeric_ops::kround};

use crate::prelude::*;

/// Format a float the way Perl stringifies a non-integer NV — its default
/// `%.15g`: up to 15 significant digits, trailing zeros stripped, no trailing
/// '.'. Used for the COMPUTED `\resizebox` scale factors (`num/denom`), which
/// Perl serializes via this default stringification. Rust's `{}` instead emits
/// the shortest round-trip form (often one digit more, e.g. `4.267906543111354`
/// vs Perl's `4.26790654311135`), so we match Perl explicitly here. (`\scalebox`
/// takes a Float arg formatted separately and is unaffected.)
pub(crate) fn perl_g15(v: f64) -> String {
  if v == 0.0 {
    return "0".to_string();
  }
  if !v.is_finite() {
    return s!("{v}");
  }
  const PREC: i32 = 15;
  let exp = v.abs().log10().floor() as i32;
  // %g uses %e when exp < -4 or exp >= PREC, else %f. For box scale factors the
  // value is always in the %f range; the %e branch is a faithful fallback.
  if !(-4..PREC).contains(&exp) {
    let s = s!("{:.*e}", (PREC - 1) as usize, v);
    match s.split_once('e') {
      Some((mant, ex)) => {
        let mant = if mant.contains('.') {
          mant.trim_end_matches('0').trim_end_matches('.')
        } else {
          mant
        };
        s!("{mant}e{ex}")
      },
      None => s,
    }
  } else {
    let decimals = (PREC - 1 - exp).max(0) as usize;
    let s = s!("{:.*}", decimals, v);
    if s.contains('.') {
      s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
      s
    }
  }
}

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

  // The box's size is typed (Perl graphics.sty.ltxml:63-81 stores Dimensions,
  // truncated to sp by `multiply`, Number.pm:107-109): the size code reads only a Dimension, and a
  // rendered string made `\wd` sum the arguments as text (`\resizebox{1em}`
  // measured 49.7pt). The template renders it to the same attribute.
  Ok(vec![
    ("width", Stored::Dimension(Dimension::new(sw as i64))),
    ("height", Stored::Dimension(Dimension::new(sh as i64))),
    ("depth", Stored::Dimension(Dimension::new(sd as i64))),
    ("xtranslate", Stored::from(dim_attr(xtranslate))),
    ("ytranslate", Stored::from(dim_attr(ytranslate))),
  ])
}

/// One `Grot` key of `\\rotatebox[…]`, in the order given (`\\setkeys{Grot}`).
#[derive(Clone, Debug)]
pub enum RotationKey {
  /// `origin=<letters>`: `l`/`r` set x, `t`/`b`/`B` set y, the last per axis
  /// winning; `c` (the default value) changes nothing.
  Origin(String),
  /// `x=<length>`, in sp.
  X(f64),
  /// `y=<length>`, in sp.
  Y(f64),
}

/// The point a box turns about, and `smash` (a zero width, rotating.sty).
/// Without `[…]` it is the reference point (graphicx `\\Grot@box@std`,
/// rotating.sty's environments: `\\Grot@x\\z@ \\Grot@y\\z@`). With `[…]`,
/// graphicx's `\\Grot@box@kv` (graphicx.sty:226-242) starts at the box's
/// centre, `\\width/2` and `(\\height-\\depth)/2` (truncating `\\divide`),
/// then applies the keys in order. Perl's `rotatedProperties`
/// (graphics.sty.ltxml:159-169) starts at the reference point and reads `c` as
/// the centre, so a single-axis origin (`origin=r`) turned about the wrong
/// point (OXIDIZED_DESIGN_DIVERGENCES #329). Witnesses 2605.02583 (`valign=m`
/// label), 2605.30813, 2605.25220 (`origin=l`/`r`).
#[derive(Clone, Debug, Default)]
pub struct RotationOptions {
  pub keys:  Option<Vec<RotationKey>>,
  pub smash: bool,
}

impl RotationOptions {
  /// `\\rotatebox`'s optional `Grot` keyvals, `None` when there are none (Perl
  /// passes `$kv->getHash`, graphics.sty.ltxml:219-225).
  pub fn from_keyvals(keyvals: Option<&Digested>) -> Self {
    let Some(arg) = keyvals else {
      return Self::default();
    };
    let DigestedData::KeyVals(ref keyvals) = *arg.data() else {
      return Self::default();
    };
    let mut taken: HashMap<&str, usize> = HashMap::default();
    let mut keys = Vec::new();
    for (key, _) in keyvals.get_pairs() {
      let index = taken.entry(key.as_str()).or_default();
      let value = keyvals
        .get_values_digested(key)
        .and_then(|values| values.get(*index));
      *index += 1;
      let length = || {
        value
          .and_then(|value| value.get_dimension())
          .map(|value| value.value_of() as f64)
      };
      match key.as_str() {
        "origin" => keys.push(RotationKey::Origin(
          value.map_or_else(String::new, |value| value.to_string()),
        )),
        "x" => keys.extend(length().map(RotationKey::X)),
        "y" => keys.extend(length().map(RotationKey::Y)),
        _ => {},
      }
    }
    RotationOptions {
      keys:  Some(keys),
      smash: false,
    }
  }
}

/// Perl: rotatedProperties($box, $angle, %options) in graphics.sty.ltxml L152-202,
/// turning about the reference point.
pub fn rotated_properties(
  body: Digested,
  angle: f64,
  smash: bool,
) -> Result<Vec<(&'static str, Stored)>> {
  rotated_properties_with(body, angle, &RotationOptions {
    smash,
    ..RotationOptions::default()
  })
}

/// Perl: rotatedProperties($box, $angle, %options) in graphics.sty.ltxml L152-202:
/// the bounding box and translation of `body` turned by `angle` degrees about
/// the point `options` names.
pub fn rotated_properties_with(
  mut body: Digested,
  angle: f64,
  options: &RotationOptions,
) -> Result<Vec<(&'static str, Stored)>> {
  let (w_dim, h_dim, d_dim, ..) = body.get_size(None)?;
  let w = w_dim.value_of() as f64;
  let h = h_dim.value_of() as f64;
  let d = d_dim.value_of() as f64;
  if w == 0.0 && h == 0.0 && d == 0.0 {
    return Ok(Vec::new());
  }
  let (mut x0, mut y0) = (0.0, 0.0);
  if let Some(keys) = &options.keys {
    x0 = (w / 2.0).trunc();
    y0 = ((h - d) / 2.0).trunc();
    for key in keys {
      match key {
        RotationKey::Origin(letters) => {
          for letter in letters.chars() {
            match letter {
              'l' => x0 = 0.0,
              'r' => x0 = w,
              't' => y0 = h,
              'b' => y0 = -d,
              'B' => y0 = 0.0,
              _ => {},
            }
          }
        },
        RotationKey::X(x) => x0 = *x,
        RotationKey::Y(y) => y0 = *y,
      }
    }
  }

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
  // Typed, as `scaled_properties`', but rounded: Perl builds these with
  // `Dimension($float)`, which rounds (Dimension.pm:40-47; graphics.sty.ltxml:183-188).
  let width_val = if options.smash { 0 } else { kround(wp) };

  Ok(vec![
    ("angle", Stored::from(s!("{angle}"))),
    ("width", Stored::Dimension(Dimension::new(width_val))),
    ("height", Stored::Dimension(Dimension::new(kround(hp)))),
    ("depth", Stored::Dimension(Dimension::new(kround(dp)))),
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
      let scaled = scaled_properties(body.clone(), xscale, yscale);
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
  // Its lengths are `\setlength` operands (graphics.sty:555-568): read through a
  // redefined `\setlength` (OXIDIZED_DESIGN #325), with the box set, so that
  // `\width` etc. measure it (`\Gscale@box@dd` sets `\@tempboxa` first).
  DefConstructor!("\\Gscale@box@dd {TempboxaDimension}{TempboxaDimension}{}",
  "<ltx:inline-block xscale='#xscale' yscale='#yscale' width='#width' height='#height' depth='#depth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#3</ltx:inline-block>",
  mode => "restricted_horizontal", enter_horizontal => true,
  after_digest => sub[whatsit] {
    let parse_pt = |a: Option<&Digested>| -> f64 {
      a.and_then(|x| x.to_attribute().trim_end_matches("pt").parse::<f64>().ok()).unwrap_or(0.0)
    };
    let num = parse_pt(whatsit.get_arg(1));
    let denom = parse_pt(whatsit.get_arg(2));
    let scale = if denom != 0.0 { num / denom } else { 1.0 };
    whatsit.set_property("xscale", Stored::from(perl_g15(scale)));
    whatsit.set_property("yscale", Stored::from(perl_g15(scale)));
    if let Some(body) = whatsit.get_arg(3).cloned()
      && let Ok(props) = scaled_properties(body, scale, scale) {
        for (k, v) in props { whatsit.set_property(k, v); }
      }
  });

  // \Gscale@box@dddd {xnum}{xdenom}{ynum}{ydenom}{body} — Perl L112-118.
  // Same idea, but separate xscale/yscale ratios.
  DefConstructor!("\\Gscale@box@dddd {TempboxaDimension}{TempboxaDimension}{TempboxaDimension}{TempboxaDimension}{}",
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
    whatsit.set_property("xscale", Stored::from(perl_g15(xscale)));
    whatsit.set_property("yscale", Stored::from(perl_g15(yscale)));
    if let Some(body) = whatsit.get_arg(5).cloned()
      && let Ok(props) = scaled_properties(body, xscale, yscale) {
        for (k, v) in props { whatsit.set_property(k, v); }
      }
  });

  // Perl: DefParameterType('GraphixDimension', sub { skipSpaces, readXToken,
  //   if ! or undef → undef, else unread + readDimension }, optional => 1)
  DefParameterType!(GraphixDimension, sub[_inner, _extra] {
    skip_spaces()?;
    let next = read_x_token(Some(false), false, None)?;
    if next.is_none() || next.as_ref().is_some_and(|t| t.text == pin!("!")) {
      // ! or end-of-input: "let other dimensions determine size"
      Ok(Tokens!())
    } else {
      // Unread and read a Dimension
      if let Some(tok) = next {
        unread_one(tok);
      }
      // graphics.sty:555-568 `\Gscale@box@dd`/`\Gscale@box@dddd` read the
      // lengths with `\setlength`, which calc redefines (calc.sty:86): with
      // calc loaded, `\resizebox{\width}{\ht\cut@boxi+\dp\cut@boxi}` is
      // one expression (perfectcut.sty:119), whose `+\dp\cut@boxi` came
      // back as text since 56jr (OXIDIZED_DESIGN #317). A package's own
      // `\setlength` (bxcalc's units) reads it first (OXIDIZED_DESIGN #325).
      let dim: Dimension = match read_through_redefined_setlength(RegisterType::Dimension)? {
        Some(value) => value.into(),
        None => match braced_length_evaluator() {
          Some(evaluate) if in_braced_read() => evaluate(RegisterType::Dimension)?.into(),
          _ => read_dimension()?,
        },
      };
      // Return the raw sp value as tokens for lossless round-trip.
      // to_attribute() rounds to 1 decimal pt, losing precision in scale calculations.
      Ok(Tokenize!(TeXString::assembled(dim.value_of().to_string())))
    }
  }, optional => true);

  // Perl graphics.sty.ltxml L40-57: DefParameterType('GraphixDimensions', ...)
  //   A sequence of up to 4 dimensions (for `trim=` / `viewport=`). They
  //   MUST be space-separated but trailing commas are tolerated between
  //   entries. Each entry tries a register value first, else reads a
  //   factor + unit (defaulting to bp). Returns a space-separated token
  //   sequence of the raw sp values.
  DefParameterType!(GraphixDimensions, sub[_inner, _extra] {
    skip_spaces()?;
    let mut dims: Vec<i64> = Vec::new();
    while dims.len() < 4 {
      if !dims.is_empty() {
        // Optionally consume a single comma between entries (Perl: if
        // the next token isn't T_OTHER(','), unread it).
        if let Some(t) = read_token()?
          && t.text != pin!(",") {
            unread_one(t);
          }
      }
      let is_negative = read_optional_signs()?;
      // Try register value (Dimension) first, allowing coercion.
      let register_dim = read_register_value_coerce(
        RegisterType::Dimension,
        true,
      )?;
      if let Some(RegisterValue::Dimension(d)) = register_dim {
        let v = d.value_of();
        dims.push(if is_negative { -v } else { v });
        continue;
      }
      // Otherwise try factor + unit. If the unit is missing, fall back
      // to `bp` (big points) per Perl L52-54.
      if let Some(factor) = read_factor()? {
        let (num, den) = match read_unit()? {
          Some(ratio) => ratio,
          None => convert_unit_ratio("bp"),
        };
        let signed = if is_negative { -factor } else { factor };
        let sp = common::numeric_ops::fixpoint_unit(signed, num, den);
        dims.push(sp);
      } else {
        break;
      }
    }
    if dims.is_empty() {
      Ok(Tokens!())
    } else {
      // Perl returns `LaTeXML::Core::Array->new(values => \@dims, ...)` where
      // each `@dims` entry is a `Dimension` object; its ToString (and so the
      // recorded `options="…trim=10.0pt 20.0pt…"` attribute) is the standard
      // pt form. Emit the same `Dimension` formatting (`10.0pt`), NOT the raw
      // sp integer — the post-processor's image-trim parser, like Perl's
      // `Util::Image` to_bp, reads the pt-suffixed value.
      let joined = dims
        .iter()
        .map(|d| Dimension(*d).to_string())
        .collect::<Vec<_>>()
        .join(" ");
      Ok(Tokenize!(TeXString::assembled(joined)))
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
      whatsit.set_property("xscale", Stored::from(perl_g15(xscale)));
      whatsit.set_property("yscale", Stored::from(perl_g15(yscale)));
      if let Ok(props) = scaled_properties(body, xscale, yscale) {
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
    if let Ok(Some(body)) = whatsit.get_body()
      && let Ok(props) = rotated_properties(body, angle, false) {
        for (k, v) in props {
          whatsit.set_property(k, v);
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
      let options = RotationOptions::from_keyvals(whatsit.get_arg(1));
      let rotated = rotated_properties_with(body.clone(), angle, &options);
      if let Ok(props) = rotated {
        for (k, v) in props {
          whatsit.set_property(k, v);
        }
      }
    }
  });

  DefMacro!("\\Grot@erotate", "\\rotatebox[]");
  // graphics.sty:445-460 rotation internals reached by isorot.sty's own
  // `\sideways`/`\turn`: `\Grot@setangle` records the angle, `\Grot@x`/
  // `\Grot@y` are the origin shifts (no XML meaning), and `\Grot@box`
  // rotates `\box\z@` through the constructor above (isorot/rotman,
  // newpax/doc-use-newpax; Perl errs the same way).
  RawTeX!(
    r"\def\Grot@setangle#1{\edef\Grot@angle{#1}}
\newdimen\Grot@x \newdimen\Grot@y
\def\Grot@box{\expandafter\rotatebox\expandafter{\Grot@angle}{\box\z@}}"
  );

  // Perl: DefConstructor('\reflectbox {}', ...) with properties callback
  // Returns width/height/depth from box size, xscale=-1, yscale=1
  DefConstructor!("\\reflectbox{}", "<ltx:inline-block xscale='#xscale' yscale='#yscale' width='#width' height='#height' depth='#depth'>#1</ltx:inline-block>",
  mode => "restricted_horizontal", enter_horizontal => true,
  after_digest => sub[whatsit] {
    if let Some(mut body) = whatsit.get_arg(1).cloned()
      && let Ok((w, h, d, _, _, _)) = body.get_size(None)
        && (w.value_of() != 0 || h.value_of() != 0 || d.value_of() != 0) {
          whatsit.set_property("width", Stored::Dimension(w));
          whatsit.set_property("height", Stored::Dimension(h));
          whatsit.set_property("depth", Stored::Dimension(d));
          whatsit.set_property("xscale", Stored::from("-1".to_string()));
          whatsit.set_property("yscale", Stored::from("1".to_string()));
        }
  });

  // == Graphics path and inclusion ==

  // Perl graphics.sty.ltxml L248-260: \graphicspath DirectoryList.
  //   properties: for each dir → PushValue(GRAPHICSPATHS => pathname_absolute(…))
  //   body: for each path in props{paths} → insertPI('latexml', graphicspath=>$path)
  //
  // DirectoryList reads the arg ToString-first so `_` in path names never
  // becomes a SUB-catcode during digestion.
  // graphics.sty:157-159's test (`\let\Ginput@path\input@path` there; empty here). Not
  // `\providecommand`: a plain document (`\input psfig.sty`, 1409.5819) would autoload
  // LaTeX mid-document, which Perl never loads.
  RawTeX!(r"\ifx\Ginput@path\@undefined \def\Ginput@path{}\fi");
  DefConstructor!("\\graphicspath DirectoryList",
  sub[document, _args, props] {
    if let Some(Stored::String(paths_sym)) = props.get("paths") {
      let paths = with(*paths_sym, |s| s.to_string());
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
    // graphics.sty:156 `\def\graphicspath#1{\def\Ginput@path{#1}}` — the
    // TeX-visible search-path macro (upmethodology reads it back).
    def_macro(T_CS!("\\Ginput@path"), None, Tokens::new(ExplodeText!(&arg)), None)?;
    let root = with_value("SOURCEDIRECTORY",
      |v| v.map(|s| s.to_string()).unwrap_or_default());
    let mut collected: Vec<String> = Vec::new();
    for dir in arg.split('}') {
      // Beyond-Perl (OXIDIZED_DESIGN #55): strip surrounding double-quotes
      // from each directory entry. pdflatex/kpathsea tolerate quoted paths
      // such as `\graphicspath{{"./figures"}}` — the quotes guard embedded
      // spaces and are removed before any filesystem lookup. Perl LaTeXML
      // keeps the literal quotes here (it strips them only in
      // `\lx@special@graphics`), so the quoted directory joins to
      // `<root>/"./figures"` and no `\includegraphics` ever resolves —
      // every one emits `expected:source`. This mirrors the existing quote
      // strip on the includegraphics FILENAME side (`image_candidates`,
      // `image.rs:53` `path.trim_matches('"')`). Witness: arXiv 2606.22880
      // (acmart, 9 figures, all lost under both Perl and Rust-before).
      let dir = dir.trim_start_matches('{').trim().trim_matches('"').trim();
      if !dir.is_empty() {
        // Perl: pathname_absolute(pathname_canonical($dir), $root)
        let path = if root.is_empty() || dir.starts_with('/') {
          dir.to_string()
        } else {
          s!("{}/{}", root, dir)
        };
        // Perl: PushValue(GRAPHICSPATHS => $path)
        let _ = push_value("GRAPHICSPATHS",
          Stored::String(pin(&path)));
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
      let candidates = util::image::image_candidates(&path);
      Ok(stored_map!("graphic" => path, "candidates" => candidates, "options" => ""))
    },
    alias => "\\includegraphics");

  // graphics.sty:189 `\Ginclude@graphics#1` is the driver-level include that
  // `\includegraphics` reaches after its keyval pass (graphicx.sty:87 only
  // re-points it for `type=`). Classes call it directly — pagelayout.cls:1494
  // `\Ginclude@graphics#6` inside `\pal@putgraphic`'s crop probe. Perl's
  // graphics.sty.ltxml has no internal either (SHARED, pdflatex clean); route it
  // to the constructor `\includegraphics` itself expands to. Witness:
  // pagelayout/example-grid (1→0), example-graphic, quickstart.
  // Guard: `perfect_kernel_batch56::ginclude_graphics_internal_routes_to_the_constructor`.
  DefMacro!("\\Ginclude@graphics{}", "\\@includegraphics{#1}");

  DefConstructor!("\\DeclareGraphicsExtensions{}", "");
  DefConstructor!("\\DeclareGraphicsRule{}{}{} Undigested", "");

  // == Gin internal macros (Perl: RawTeX block, lines 311-324) ==

  // \Gin@extensions — comma-separated list of recognized image extensions.
  // Real graphics.sty assigns this from the chosen graphics driver
  // (.eps for dvips, .pdf,.png,.jpg for pdftex). User code rarely reads
  // it directly, but `\graphicspath` / `\@for` loops in third-party
  // packages (e.g. xkeyval drivers) iterate over it. Pre-define as
  // empty to suppress "undefined" cascades — the actual driver-specific
  // list is filled in elsewhere if a graphics driver binding loads.
  // Driver: 2103.04594.
  Let!("\\Gin@extensions", "\\@empty");
  Let!("\\Gin@decode", "\\@empty");
  DefMacro!("\\Gin@exclamation", "!");
  Let!("\\Gin@page", "\\@empty");
  DefMacro!("\\Gin@pagebox", "cropbox");
  // Real graphics.sty boolean state, ported as the actual \newif lines so
  // the \Gin@<name>true/false setters exist — raw packages drive them
  // directly and through graphicx's \Gin@boolkey dispatcher (sweep-11
  // clusters: `\Gin@boolkey` 34 docs incl. hvfloat.sty L411
  // `\Gin@boolkey{true}{iso}`; `\Gin@draftfalse` 9 docs, e.g. bohr,
  // pagelayout). Sources: graphics.sty L55 (draft), L63 (setpagesize,
  // default true), L253 (Gread@, default true), L307 (interpolate),
  // L319 (bbox), L579 (iso). Guard:
  // cluster_package_guards::graphicx_internals.
  RawTeX!(r"\newif\ifGin@draft");
  RawTeX!(r"\newif\ifGin@setpagesize\Gin@setpagesizetrue");
  RawTeX!(r"\newif\ifGread@\Gread@true");
  RawTeX!(r"\newif\ifGin@interpolate");
  RawTeX!(r"\newif\ifGin@bbox");
  RawTeX!(r"\newif\ifGin@iso");
  Let!("\\Gin@log", "\\wlog");
  Let!("\\Gin@req@sizes", "\\relax");
  DefMacro!("\\Gin@scalex", "1");
  Let!("\\Gin@scaley", "\\Gin@exclamation");
  // These reference macros that may not exist yet, so define them
  def_macro_noop("\\Gin@nat@height")?;
  def_macro_noop("\\Gin@nat@width")?;
  Let!("\\Gin@req@height", "\\Gin@nat@height");
  Let!("\\Gin@req@width", "\\Gin@nat@width");
  Let!("\\Gin@viewport@code", "\\relax");

  // graphics.sty L115 (\newif form, so \Gin@cliptrue/false exist too;
  // was DefConditional per Perl, which lacks the setters raw code calls).
  RawTeX!(r"\newif\ifGin@clip");
  // Perl: DefMacro('\Gin@i [][]{}', '');
  def_macro_noop("\\Gin@i[][]{}")?;

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
  def_macro_noop("\\set@color")?;

  // The pdftex driver's begin-document load (pdftex.def:681-701; luatex.def:681-700 alike): unless
  // `\DoNotLoadEpstopdf` is defined, with `\includegraphics` defined and neither pst-pdf nor pdftricks loaded, it
  // requires epstopdf-base. Its conversion rules mean nothing here, but one branch does: when `\@curroptions` (what
  // the last package's options processing left) is not empty, it requires pdftexcmds (epstopdf-base.sty:151-182),
  // and that iftex, which defines `\ifpdf` afresh when it loads first. A paper's earlier `\let\ifpdf\relax` is then
  // undone in TeX; kept, JINST's `\label` (JINST.cls:328-334, `\ifpdf…\fi` inside its own conditional) left a stray
  // `\fi` that ended the caption's argument scan in a Fatal (1310.6454).
  DefMacro!("\\lx@graphics@if@epstopdf@driver", sub[_args] {
    Ok(Tokens!(if graphics_driver_loads_epstopdf() {
      T_CS!("\\@firstoftwo")
    } else {
      T_CS!("\\@secondoftwo")
    }))
  });
  // Loaded as a package loads it: the driver's load is no `\RequirePackage` of the document's, and leaves no
  // `<?latexml package?>` for one.
  DefPrimitive!("\\lx@graphics@require@pdftexcmds", {
    RequirePackage!("pdftexcmds");
  });
  RawTeX!(
    r"\expandafter\ifx\csname DoNotLoadEpstopdf\endcsname\relax
    \AtBeginDocument{\ifx\includegraphics\@undefined\else\lx@graphics@if@epstopdf@driver{%
      \@ifpackageloaded{pst-pdf}{}{\@ifpackageloaded{pdftricks}{}{%
        \ifx\@curroptions\@empty\else\lx@graphics@require@pdftexcmds\fi}}}{}\fi}\fi"
  );
});

/// graphics.sty's driver options in declaration order (graphics.sty:67-88), with whether that driver file loads
/// epstopdf-base at `\begin{document}`: only pdftex.def and luatex.def do.
const GRAPHICS_DRIVER_OPTIONS: [(&str, bool); 22] = [
  ("dvips", false),
  ("xdvi", false),
  ("dvipdf", false),
  ("dvipdfm", false),
  ("dvipdfmx", false),
  ("xetex", false),
  ("pdftex", true),
  ("luatex", true),
  ("dvisvgm", false),
  ("dvipsone", false),
  ("dviwindo", false),
  ("emtex", false),
  ("dviwin", false),
  ("oztex", false),
  ("textures", false),
  ("pctexps", false),
  ("pctexwin", false),
  ("pctexhp", false),
  ("pctex32", false),
  ("truetex", false),
  ("tcidvi", false),
  ("vtex", false),
];

/// Does graphics.sty's driver (`\Gin@driver`) load epstopdf-base? `\ProcessOptions` runs the declared driver options
/// that the package or the class was given in declaration order, so the last one listed wins; graphicx passes its
/// own options on (graphicx.sty:29). With none, graphics.cfg picks the PDF driver for PDF output (`\pdfoutput` > 0,
/// as K6 sets it) and xetex.def under XeTeX.
fn graphics_driver_loads_epstopdf() -> bool {
  let mut given: Vec<String> = Vec::new();
  for key in ["opt@graphics.sty", "opt@graphicx.sty", "class_options"] {
    for item in lookup_vecdeque(key).unwrap_or_default() {
      match item {
        Stored::String(s) => given.push(with(s, |s| s.trim().to_string())),
        Stored::Strings(ss) => given.extend(ss.iter().map(|s| with(*s, |s| s.trim().to_string()))),
        _ => {},
      }
    }
  }
  match GRAPHICS_DRIVER_OPTIONS
    .iter()
    .rev()
    .find(|(name, _)| given.iter().any(|g| g == name))
  {
    Some((_, loads)) => *loads,
    None => {
      !lookup_bool("XETEX_PROFILE")
        && lookup_register("\\pdfoutput", Vec::new())
          .ok()
          .flatten()
          .is_some_and(|value| Number::from(&value).value_of() > 0)
    },
  }
}

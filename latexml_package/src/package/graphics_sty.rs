use crate::prelude::*;
use latexml_core::common::dimension::attribute_format;

/// Perl: graphics_scaledbox_props($box, $xscale, $yscale) in graphics.sty.ltxml L63-81
/// Computes scaled dimensions and translation offsets for \scalebox.
pub fn scaled_properties(
  mut body: Digested,
  xscale: f64,
  yscale: f64,
) -> Result<Vec<(&'static str, Stored)>> {
  let (w_dim, h_dim, d_dim, _, _, _) = body.get_size(None)?;
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

  let dim_attr = |v: f64| attribute_format(v as i64, None);

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
  let (w_dim, h_dim, d_dim, _, _, _) = body.get_size(None)?;
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
  let rad = angle * std::f64::consts::PI / 180.0;
  let s = rad.sin();
  let c = rad.cos();
  let wp = (w * c).abs() + (total_h * s).abs();
  let corners = [
    (-d - y0) * c + (0.0 - x0) * s + y0,    // bottom-left
    (-d - y0) * c + (w - x0) * s + y0,       // bottom-right
    (h - y0) * c + (w - x0) * s + y0,        // top-right
    (h - y0) * c + (0.0 - x0) * s + y0,      // top-left
  ];
  let hp = corners.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
  let dp = -corners.iter().cloned().fold(f64::INFINITY, f64::min);
  let xsh = (wp - w) / 2.0;
  let ysh = (h + d - hp - dp) / 2.0;

  let dim_attr = |v: f64| attribute_format(v as i64, None);
  let width_val = if smash { "0.0pt".to_string() } else { dim_attr(wp) };

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
  DefConstructor!("\\scalebox{} []{}", "<ltx:inline-block xscale='#1' yscale='#yscale' width='#width' height='#height' depth='#depth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#3</ltx:inline-block>",
    mode => "restricted_horizontal", enter_horizontal => true,
    properties => sub[args] {
      let yscale = args[1].as_ref().map(|a| a.to_attribute())
        .unwrap_or_else(|| args[0].as_ref().map(|a| a.to_attribute()).unwrap_or_default());
      Ok(stored_map!("yscale" => yscale))
    },
    after_digest => sub[whatsit] {
      let xscale = whatsit.get_arg(1)
        .map(|a| a.to_attribute().parse::<f64>().unwrap_or(1.0)).unwrap_or(1.0);
      let yscale = whatsit.get_arg(2)
        .map(|a| a.to_attribute().parse::<f64>().unwrap_or(xscale)).unwrap_or(xscale);
      if let Some(body) = whatsit.get_arg(3) {
        if let Ok(props) = crate::package::graphics_sty::scaled_properties(body.clone(), xscale, yscale) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
      }
    });
  Let!("\\Gscale@box", "\\scalebox");

  // \resizebox{width}{height}{content}
  DefMacro!("\\resizebox", "\\leavevmode\\@ifstar{\\Gscale@@box\\totalheight}{\\Gscale@@box\\height}");

  // Simplified \Gscale@@box — just passes through content for now
  DefConstructor!("\\Gscale@@box{}{}{}{}", "#4",
    mode => "restricted_horizontal", enter_horizontal => true);

  // == Rotation ==

  // Rotation keyvals
  DefKeyVal!("Grot", "origin", "");
  DefKeyVal!("Grot", "x", "Dimension");
  DefKeyVal!("Grot", "y", "Dimension");
  DefKeyVal!("Grot", "units", "");

  DefConstructor!("\\rotatebox OptionalKeyVals:Grot {Float}{}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#3</ltx:inline-block>",
    mode => "restricted_horizontal", enter_horizontal => true,
    after_digest => sub[whatsit] {
      let angle = whatsit.get_arg(1).map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
      if let Some(body) = whatsit.get_arg(2) {
        if let Ok(props) = crate::package::graphics_sty::rotated_properties(body.clone(), angle, false) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
      }
    });

  // {rotatebox} environment form — used as \begin{rotatebox}{90}...\end{rotatebox}
  DefEnvironment!("{rotatebox}{Float}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#body</ltx:inline-block>",
    after_digest_body => sub[whatsit] {
      let angle = whatsit.get_arg(1).map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
      if let Ok(Some(body)) = whatsit.get_body() {
        if let Ok(props) = crate::package::graphics_sty::rotated_properties(body, angle, false) {
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

  DefConstructor!("\\graphicspath{}", "",
    after_digest => sub[_whatsit] {
      // TODO: push paths to GRAPHICSPATHS
    });

  // Perl: DefMacro('\includegraphics OptionalMatch:* [][] Semiverbatim',
  //   '\@includegraphics#1[#2][#3]{#4}');
  DefMacro!("\\includegraphics OptionalMatch:* [][] Semiverbatim",
    "\\@includegraphics#1[#2][#3]{#4}");

  DefConstructor!("\\@includegraphics OptionalMatch:* [][] Semiverbatim",
    "<ltx:graphics graphic='#graphic' candidates='#candidates' options='#options'/>",
    enter_horizontal => true,
    properties => sub[args] {
      let path = args[3].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      let path = path.trim().to_string();
      Ok(stored_map!("graphic" => path.clone(), "candidates" => path, "options" => ""))
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

  // Perl: DefPrimitive('\Gscale@div DefToken Dimension Dimension', sub { ... })
  // \Gscale@div{\cs}{\dima}{\dimb} : \cs = \dima / \dimb
  DefMacro!("\\Gscale@div{}{}{}", "");

  // Perl: \set@color defined elsewhere but referenced by graphics
  // Provide a no-op fallback if not already defined
  DefMacro!("\\set@color", "");
});

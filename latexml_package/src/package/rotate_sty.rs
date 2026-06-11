use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: rotate.sty.ltxml
  RequirePackage!("graphicx");

  // \rotate[l]{stuff}
  // l=left, r=right, u=upsidedown, f=flip
  DefConstructor!("\\rotate Optional:l Digested",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth'\
     innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth'\
     xscale='#xscale' yscale='#yscale'\
     xtranslate='#xtranslate' ytranslate='#ytranslate'>\
     #2\
     </ltx:inline-block>",
    after_digest => sub[whatsit] {
      let rot_str = whatsit.get_arg(1)
        .map(|a| a.to_string())
        .unwrap_or_default();
      let rot = if rot_str.is_empty() { "l".to_string() } else { rot_str };
      if rot == "l" || rot == "r" {
        let angle = if rot == "l" { -90.0 } else { 90.0 };
        if let Some(body) = whatsit.get_arg(2)
          && let Ok(props) = graphics_sty::rotated_properties(body.clone(), angle, false) {
            for (k, v) in props {
              whatsit.set_property(k, v);
            }
          }
      } else {
        // u=upsidedown, f=flip
        let xscale: f64 = if rot == "u" { 1.0 } else { -1.0 };
        let yscale: f64 = if rot == "u" { -1.0 } else { 1.0 };
        if let Some(body) = whatsit.get_arg(2) {
          let mut body = body.clone();
          if let Ok((w_dim, h_dim, d_dim, _, _, _)) = body.get_size(None) {
            let w = w_dim.value_of() as f64;
            let h = h_dim.value_of() as f64;
            let d = d_dim.value_of() as f64;
            if w != 0.0 || h != 0.0 || d != 0.0 {
              let dim_attr = |v: f64| Stored::from(common::dimension::attribute_format(v as i64, None));
              let ytranslate = (h + d) * (yscale - 1.0) / 2.0;
              whatsit.set_property("width", dim_attr(w));
              whatsit.set_property("height", dim_attr(h * yscale));
              whatsit.set_property("depth", dim_attr(d * yscale));
              whatsit.set_property("xscale", Stored::from(s!("{xscale}")));
              whatsit.set_property("yscale", Stored::from(s!("{yscale}")));
              whatsit.set_property("ytranslate", dim_attr(ytranslate));
            }
          }
        }
      }
    },
    mode => "internal_vertical"
  );
});

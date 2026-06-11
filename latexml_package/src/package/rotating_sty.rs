use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: rotating.sty.ltxml — rotation environments with rotatedProperties

  DeclareOption!("twoside", None);
  DeclareOption!("figuresright", None);
  DeclareOption!("figuresleft", None);
  DeclareOption!("quiet", None);
  DeclareOption!("log", None);
  DeclareOption!("chatter", None);
  ProcessOptions!();

  RequirePackage!("graphicx");
  RequirePackage!("ifthen");

  TeX!(r"\newdimen\rotFPtop \rotFPtop=0pt
\newdimen\rotFPbot \rotFPbot=0pt
");

  DefEnvironment!("{sideways}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#body</ltx:inline-block>",
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, 90.0, false) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
    });

  DefEnvironment!("{turn}{Float}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#body</ltx:inline-block>",
    after_digest_body => sub[whatsit] {
      let angle = whatsit.get_arg(1).map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
      // Always set angle even if body dimensions fail (e.g. alignment inside {turn})
      whatsit.set_property("angle", Stored::from(s!("{angle}")));
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, angle, false) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
    });

  DefEnvironment!("{rotate}{Float}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#body</ltx:inline-block>",
    after_digest_body => sub[whatsit] {
      let angle = whatsit.get_arg(1).map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, angle, true) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
    });

  DefConstructor!("\\turnbox{Float} {}",
    "<ltx:inline-block angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xtranslate='#xtranslate' ytranslate='#ytranslate'>#2</ltx:inline-block>",
    mode => "internal_vertical",
    after_digest => sub[whatsit] {
      let angle = whatsit.get_arg(1).map(|a| a.to_attribute().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
      if let Some(body) = whatsit.get_arg(2)
        && let Ok(props) = graphics_sty::rotated_properties(body.clone(), angle, false) {
          for (k, v) in props {
            whatsit.set_property(k, v);
          }
        }
    });

  // sidewaysfigure/sidewaystable — full port matching Perl rotating.sty.ltxml
  // Each has: beforeFloat/afterFloat hooks, rotatedPage hsize, rotatedProperties for dimensions
  DefEnvironment!("{sidewaysfigure}[]",
    "<ltx:figure xml:id='#id' ?#1(placement='#1') angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xscale='#xscale' yscale='#yscale' xtranslate='#xtranslate' ytranslate='#ytranslate'>#tags#body</ltx:figure>",
    mode => "internal_vertical",
    before_digest => {
      engine::latex_constructs::before_float("figure", None);
      // rotatedPage: \hsize = \textheight
      // rotatedPage: \hsize = \textheight
      let texheight = lookup_dimension("\\textheight").unwrap_or_default();
      AssignRegister!("\\hsize", texheight.into());
    },
    after_digest => sub[whatsit] {
      engine::latex_constructs::after_float(whatsit);
    },
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, 90.0, false) {
          for (k, v) in props { whatsit.set_property(k, v); }
        }
    });

  DefEnvironment!("{sidewaysfigure*}[]",
    "<ltx:figure xml:id='#id' ?#1(placement='#1') angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xscale='#xscale' yscale='#yscale' xtranslate='#xtranslate' ytranslate='#ytranslate'>#tags#body</ltx:figure>",
    mode => "internal_vertical",
    before_digest => {
      engine::latex_constructs::before_float("figure", None);
      // rotatedPage: \hsize = \textheight
      let texheight = lookup_dimension("\\textheight").unwrap_or_default();
      AssignRegister!("\\hsize", texheight.into());
    },
    after_digest => sub[whatsit] {
      engine::latex_constructs::after_float(whatsit);
    },
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, 90.0, false) {
          for (k, v) in props { whatsit.set_property(k, v); }
        }
    });

  DefEnvironment!("{sidewaystable}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1') angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xscale='#xscale' yscale='#yscale' xtranslate='#xtranslate' ytranslate='#ytranslate'>#tags#body</ltx:table>",
    mode => "internal_vertical",
    before_digest => {
      engine::latex_constructs::before_float("table", None);
      // rotatedPage: \hsize = \textheight
      let texheight = lookup_dimension("\\textheight").unwrap_or_default();
      AssignRegister!("\\hsize", texheight.into());
    },
    after_digest => sub[whatsit] {
      engine::latex_constructs::after_float(whatsit);
    },
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, 90.0, false) {
          for (k, v) in props { whatsit.set_property(k, v); }
        }
    });

  DefEnvironment!("{sidewaystable*}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1') angle='#angle' width='#width' height='#height' depth='#depth' innerwidth='#innerwidth' innerheight='#innerheight' innerdepth='#innerdepth' xscale='#xscale' yscale='#yscale' xtranslate='#xtranslate' ytranslate='#ytranslate'>#tags#body</ltx:table>",
    mode => "internal_vertical",
    before_digest => {
      engine::latex_constructs::before_float("table", None);
      // rotatedPage: \hsize = \textheight
      let texheight = lookup_dimension("\\textheight").unwrap_or_default();
      AssignRegister!("\\hsize", texheight.into());
    },
    after_digest => sub[whatsit] {
      engine::latex_constructs::after_float(whatsit);
    },
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body()
        && let Ok(props) = graphics_sty::rotated_properties(body, 90.0, false) {
          for (k, v) in props { whatsit.set_property(k, v); }
        }
    });

  DefMacro!("\\rotcaption{}", r"\caption{\turnbox{90}{#1}}");
});

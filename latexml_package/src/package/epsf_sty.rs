use crate::prelude::*;
use latexml_core::definition::register::RegisterValue;

#[rustfmt::skip]
LoadDefinitions!({
  RawTeX!("\\newif\\ifepsfatend");
  RawTeX!("\\newif\\ifepsfdraft");
  RawTeX!("\\newif\\ifepsfframe");
  RawTeX!("\\newif\\ifepsfshow");
  RawTeX!("\\epsfshowtrue");
  RawTeX!("\\newif\\ifepsfshowfilename");
  RawTeX!("\\newif\\ifepsfverbose");
  RawTeX!("\\newdimen\\epsfframemargin");
  RawTeX!("\\newdimen\\epsfframethickness");
  RawTeX!("\\newdimen\\epsfxsize");
  RawTeX!("\\newdimen\\epsfysize");
  RawTeX!("\\newdimen\\pspoints");
  RawTeX!("\\pspoints = 1bp");
  RawTeX!("\\epsfxsize = 0pt");
  RawTeX!("\\epsfysize = 0pt");
  RawTeX!("\\epsfframemargin = 0pt");
  RawTeX!("\\epsfframethickness = 0.4pt");
  DefPrimitive!("\\epsfclipon", {
    state::assign_value("epsf_clip", Stored::from(1), None);
  });
  DefPrimitive!("\\epsfclipoff", {
    state::assign_value("epsf_clip", Stored::from(0), None);
  });
  // Perl: DefConstructor('\epsfbox [] Semiverbatim', "<ltx:graphics graphic='#graphic' candidates='#candidates' options='#options'/>", ...)
  // Creates ltx:graphics directly — does NOT require graphicx/\includegraphics to be loaded.
  // Perl epsf.sty.ltxml L45-62: options string assembled from epsf_clip,
  // optional bounding-box `[bb]`, \epsfxsize and \epsfysize. Prior Rust
  // always emitted options="", dropping clip/viewport/width/height.
  DefConstructor!("\\epsfbox [] Semiverbatim",
    "<ltx:graphics graphic='#graphic' candidates='#candidates' options='#options'/>",
    enter_horizontal => true,
    properties => sub[args] {
      let bb   = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let bb   = bb.trim().to_string();
      let path = args[1].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      let path = path.trim().to_string();
      let candidates = latexml_core::util::image::image_candidates(&path);

      let clip = state::lookup_value("epsf_clip")
        .map(|v| v.to_string() != "0" && !v.to_string().is_empty())
        .unwrap_or(false);
      let mut options = String::new();
      if clip {
        options = if bb.is_empty() {
          "clip".to_string()
        } else {
          format!("viewport={}, clip", bb)
        };
      }
      let reg_dim = |name: &str| -> Option<Dimension> {
        match state::lookup_register(name, Vec::new()) {
          Ok(Some(RegisterValue::Dimension(d))) => Some(d),
          _ => None,
        }
      };
      if let Some(w) = reg_dim("\\epsfxsize") {
        if w.value_of() > 0 {
          if !options.is_empty() { options.push(','); }
          options.push_str(&format!("width={}", w.to_attribute()));
        }
      }
      if let Some(h) = reg_dim("\\epsfysize") {
        if h.value_of() > 0 {
          if !options.is_empty() { options.push(','); }
          options.push_str(&format!("height={}", h.to_attribute()));
        }
      }
      Ok(stored_map!("graphic" => path, "candidates" => candidates, "options" => options))
    });
  Let!("\\epsfgetlitbb", "\\epsfbox");
  Let!("\\epsfnormal",   "\\epsfbox");
  Let!("\\epsffile",     "\\epsfbox");
  def_primitive_noop("\\epsfgetbb Semiverbatim")?;
  def_primitive_noop("\\epsfframe")?;
});

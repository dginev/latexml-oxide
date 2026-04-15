use crate::prelude::*;

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
  DefConstructor!("\\epsfbox [] Semiverbatim",
    "<ltx:graphics graphic='#graphic' candidates='#candidates' options='#options'/>",
    enter_horizontal => true,
    properties => sub[args] {
      let path = args[1].as_ref().map(|a| a.to_attribute()).unwrap_or_default();
      let path = path.trim().to_string();
      let candidates = crate::package::graphicx_sty::image_candidates(&path);
      Ok(stored_map!("graphic" => path, "candidates" => candidates, "options" => ""))
    });
  Let!("\\epsfgetlitbb", "\\epsfbox");
  Let!("\\epsfnormal",   "\\epsfbox");
  Let!("\\epsffile",     "\\epsfbox");
  DefPrimitive!("\\epsfgetbb Semiverbatim", None);
  DefPrimitive!("\\epsfframe",              None);
});

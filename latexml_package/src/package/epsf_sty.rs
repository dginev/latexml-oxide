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
  // Perl: DefConstructor('\epsfbox [] Semiverbatim', "<ltx:graphics graphic='#2' options=''/>")
  // TODO: The CompileReplacement proc macro can't resolve #2 for [] {} parameter patterns.
  // Using DefMacro delegation to \includegraphics as a faithful approximation.
  DefMacro!("\\epsfbox[]{}", "\\includegraphics{#2}");
  Let!("\\epsfgetlitbb", "\\epsfbox");
  Let!("\\epsfnormal",   "\\epsfbox");
  Let!("\\epsffile",     "\\epsfbox");
  DefPrimitive!("\\epsfgetbb Semiverbatim", None);
  DefPrimitive!("\\epsfframe",              None);
});

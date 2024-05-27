use crate::prelude::*;
// use super::tex_boxes::adjust_box_color;

//**********************************************************************
// Primitives
// See The TeXBook, Chapter 24, Summary of Vertical Mode
//  and Chapter 25, Summary of Horizontal Mode.
// Parsing of basic types (pp.268--271) is (mostly) handled in Gullet.pm
//**********************************************************************

LoadDefinitions!({
  //======================================================================
  // Remaining Mode independent primitives in Ch.24, pp.279-280
  DefPrimitive!("\\ignorespaces SkipSpaces", None);

  //======================================================================
  // Remaining semi- Vertical Mode primitives in Ch.24, pp.280--281
  DefPrimitive!("\\penalty Number", None);

  DefMacro!(
    "\\mkern MuGlue",
    "\\ifmmode\\@math@mskip #1\\relax\\else\\@text@mskip #1\\relax\\fi"
  );
  DefPrimitive!("\\unpenalty", None);
  
  // Worrisome, but...
  DefPrimitive!("\\unskip", {
    // pop until a non-empty box is found
    while let Some(last_box) = pop_box_list() {
      if !last_box.is_empty()? {
        push_box_list(last_box);
        break;
      }
    }
  });
  
  // \vadjust<filler>{<vertical mode material>}
  // Note: \vadjust ignores in vertical mode...
  DefPrimitive!("\\vadjust {}", sub[(arg)] { push_tokens("vAdjust", arg); });
});

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\windowsname",    "Windows");
  DefMacro!("\\notwindowsname", "*NIX");
  DefMacro!("\\linuxname",      "Linux");
  DefMacro!("\\macosxname",     "Mac\\,OS\\,X");
  DefMacro!("\\cygwinname",     "Cygwin");
  DefConditional!("\\ifshellescape", {
    true
  });
  DefMacro!("\\unknownplatform", "Linux");
  Let!("\\platformname", "\\linuxname");
  DefConditional!("\\ifwindows", {
    false
  });
  DefConditional!("\\iflinux", {
    true
  });
  DefConditional!("\\ifmacosx", {
    false
  });
  DefConditional!("\\ifcygwin", {
    false
  });
});

use crate::prelude::*;
LoadDefinitions!({
  RequirePackage!("amsgen");

  DefConstructor!("\\text{}", "<ltx:text _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true, locked => true);
});

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DeclareOption!(None, {
    Digest!("\\PassOptionsToPackage{\\CurrentOption}{xy}")?;
  });
  ProcessOptions!();
  RequirePackage!("xy", options => vec!["v2".to_string()]);
});

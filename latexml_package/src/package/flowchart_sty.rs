use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: flowchart.sty.ltxml — a tikz extension
  InputDefinitions!("flowchart", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});

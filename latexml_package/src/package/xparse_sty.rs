//! xparse.sty — document command parser interface
//! Perl: xparse.sty.ltxml — loads the raw sty with noltxml
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl xparse.sty.ltxml L20-21: mark versioned xparse variants as
  // acceptable INTERPRETABLE_SOURCES so they bypass the version-suffix
  // strip fallback and can be fed directly to the raw-TeX loader.
  AssignMapping!("INTERPRETABLE_SOURCES", "xparse-2018-04-12.sty" => 1);
  AssignMapping!("INTERPRETABLE_SOURCES", "xparse-2020-10-01.sty" => 1);

  // Load raw xparse.sty — may hit errors from partial expl3 kernel. Its `\ExplSyntaxOn` changes catcodes that our
  // `\ExplSyntaxOff` does not fully restore (spaces left IGNORE broke paragraphs), so the load restores the values
  // they had before it.
  let _ = expl3_sty::with_expl_syntax_restored(|| Ok(input_definitions("xparse",
    NewDefault!(InputDefinitionOptions, noltxml => true, extension => Some(Cow::Borrowed("sty"))))));
});

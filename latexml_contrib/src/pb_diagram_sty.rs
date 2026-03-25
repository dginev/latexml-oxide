use latexml_package::prelude::*;

LoadDefinitions!({
  Warn!("missing_file", "pb-diagram.sty",
    "pb-diagram.sty is not implemented and will not be interpreted raw.");
  // TODO: Perl has a discard_env_body closure that reads and discards
  // the {diagram} environment body, emitting an <ltx:ERROR> element.
  // For now, stub the environment as empty.
  DefMacro!("\\enddiagram", "\\relax");
});

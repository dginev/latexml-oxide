use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("harvmac", noltxml => true, extension => Some(Cow::Borrowed("tex")));
  // TODO: Perl has a complex \eqn macro with closure that checks IN_MATH
  // and either returns content directly or wraps in display math.
  // Also \listtoc and \writetoc for Table-of-Contents are stubbed.
  // For now, just load raw definitions.
});

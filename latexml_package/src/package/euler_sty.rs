use crate::prelude::*;
#[rustfmt::skip]
LoadDefinitions!({
  // euler.sty:47-57 `\DeclareMathSymbol\Gamma\mathord{letters}{"00}`…, its `letters` the upright Euler font: the capital
  // Greek letters upright, over a class's italic ones (llncs.cls:379-389; KNOWN_PERL_ERRORS #1147).
  declare_capital_greek("", None)?;
});

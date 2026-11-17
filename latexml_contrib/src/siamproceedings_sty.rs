//! siamproceedings.sty: the paper's copy read raw, the marks its `\maketitle` sets declared — none under `titlepage`,
//! whose `\maketitle` sets no `\thefootnote` (2509.02371, 2509.13490; OD #1053).
use latexml_package::{prelude::*, shipped_class::input_shipped_package_title_marks};

LoadDefinitions!({
  // (siamproceedings.sty:187-189)
  input_shipped_package_title_marks(&[(r"\@fnsymbol\c@footnote", r"\@fnsymbol\c@footnote")])?;
});

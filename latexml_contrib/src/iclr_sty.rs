//! iclr.sty and its kin — the NIPS style's descendants over `\documentclass{article}`: ICLR's (iclr2015-iclr2027,
//! iclr_conference, iclr_preprint, their `_arXiv` copies; "Adapted by Hugo Larochelle from the NIPS stylefile",
//! iclr2020_conference.sty:2), TMLR's (tmlr.sty:1-3, "from the ICLR stylefile"), CoRL's (corl_2017-corl_2026,
//! corl_2024.sty:5) and NIPS's own 2016-2018 (nips_2016-nips_2018.sty); the versioned fallback strips each to its
//! family name. The paper's copy is read raw; the binding only says which marks the copy's `\maketitle` prints, which
//! the lock refused: `\def\thefootnote{\fnsymbol{footnote}}` (iclr2026_conference.sty:61-63, tmlr.sty:71-73;
//! nips_2017.sty:246-249 and corl_2025.sty:343-346 `\renewcommand`), the document's `\@fnsymbol` of the counter (all 329
//! ICLR, 35 TMLR, 35 CoRL and 58 NIPS copies of the census with a `\maketitle`). 2303.07937, 2502.12413, 2509.00328,
//! 1706.03762.
use latexml_package::{
  prelude::*,
  shipped_class::{declare_shipped_copy_title_marks, input_shipped_package},
};

LoadDefinitions!({
  if input_shipped_package()? {
    declare_shipped_copy_title_marks("sty")?;
  }
});

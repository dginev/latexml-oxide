//! usenix.sty — USENIX's conference style over `\documentclass{article}` (usenix.sty, usenix2019_v3.sty,
//! usenix-2020-09.sty; the versioned fallback strips each to `usenix`). The paper's copy is read raw; the binding only
//! says which marks the copy's `\maketitle` prints, which the lock refused: usenix.sty:48-50 (usenix-2020-09.sty:83-85)
//! `\renewcommand\thefootnote{\fnsymbol{footnote}}`, the document's `\@fnsymbol` of the counter; a copy with no
//! `\maketitle` of its own (cs0405079's) leaves the class's. 2509.03117, 1110.1687.
use latexml_package::{
  prelude::*,
  shipped_class::{declare_shipped_copy_title_marks, input_shipped_package},
};

LoadDefinitions!({
  if input_shipped_package()? {
    declare_shipped_copy_title_marks("sty")?;
  }
});

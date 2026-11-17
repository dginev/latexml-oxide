//! acl.sty and its kin — the ACL conference styles over `\documentclass{article}` (acl.sty, acl2010-acl2024.sty,
//! naacl2021.sty, naaclhlt2016-2019.sty, emnlp2016-2023.sty, emnlp-ijcnlp-2019.sty, eacl2017-2023.sty, coling.sty and
//! coling2018/2020.sty, tacl2018v2/2021v1.sty, conll-2019.sty; the versioned fallback strips each to its family name).
//! The paper's copy is read raw; the binding only says which marks the copy's `\maketitle` prints, which the lock
//! refused: every copy's is `\def\thefootnote{\fnsymbol{footnote}}` (acl.sty:148-150, acl2019.sty:392-394), the
//! document's `\@fnsymbol` of the counter (latex.ltx `\fnsymbol`; acl2019.sty:37's own, 1905.10033 "Zhaojiang Lin†");
//! 2502.19103.
use latexml_package::{
  engine::latex_constructs::declare_package_title_marks, prelude::*,
  shipped_class::input_shipped_package,
};

LoadDefinitions!({
  if input_shipped_package()? {
    declare_package_title_marks(r"\@fnsymbol\c@footnote", true)?;
  }
});

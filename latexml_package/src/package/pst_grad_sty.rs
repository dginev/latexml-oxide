//! `pst-grad.sty` — gradient fill styles for PSTricks.
//!
//! pst-grad.sty is a two-line wrapper: `\RequirePackage{pstricks}` then
//! `\input{pst-grad.tex}`. Perl's binding (`pst-grad.sty.ltxml`) stops at the
//! `RequirePackage`, which leaves `\psfs@gradient` and the `grad*` keys
//! undefined; once `\pst@object` really dispatches (batch 56af) every
//! `[fillstyle=gradient,…]` bracket on a raw object (`\pscharpath`,
//! `\pscustom`, …) raised "Undefined fill style", leaving `\psk@fillstyle`
//! at its `\relax` default, so a following `addfillstyle`'s
//! `\expandafter\noexpand\psk@fillstyle` froze the name into itself — a
//! `\psk@fillstyle` self-recursion (witness arabi/big2). We load
//! the raw `pst-grad.tex` the wrapper loads: its fill styles are PostScript
//! strings our renderer ignores, but the key surface is complete.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("pstricks");
  // pst-grad.sty:3 `\input{pst-grad.tex}` (pst-grad.tex:39-47 self-guards
  // with `\GradientLoaded` and re-inputs pstricks/pst-xkey only if missing).
  InputDefinitions!("pst-grad", extension => Some(Cow::Borrowed("tex")), noltxml => true);
});

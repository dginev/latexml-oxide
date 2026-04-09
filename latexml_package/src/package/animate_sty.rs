//! animate.sty — PDF/SVG animation from graphics files
//! No Perl binding exists — loaded as raw TeX in both Perl and Rust.
//! animate.sty uses \ExplSyntaxOn from the LaTeX 2022+ kernel.
//! With our expl3 autoload (matching Perl's DefAutoload), this loads naturally.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Load animate.sty as raw TeX. The \ExplSyntaxOn autoload trigger
  // will auto-load expl3 if not already available.
  InputDefinitions!("animate", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});

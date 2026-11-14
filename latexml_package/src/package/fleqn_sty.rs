use crate::prelude::*;

/// fleqn.clo, which `fleqn.sty` inputs and the kernel classes input for their `fleqn` option (article.cls:100,
/// report.cls:106, book.cls:108, slides.cls:101; elsarticle.cls:1280 through `fleqn.sty` here): Perl's flush-left
/// class (fleqn.sty.ltxml), and its `{equation}` ([`math_displays_in_hbox`]). Witnesses cond-mat0202518,
/// hep-ex9806021 (rc_struct6 row 2; KNOWN_PERL_ERRORS #907). Guard
/// `perfect_kernel_batch73b::dollar_in_a_fleqn_clo_display_is_text`.
pub fn fleqn_clo() -> Result<()> {
  AssignMapping!("DOCUMENT_CLASSES", "ltx_fleqn" => true);
  math_displays_in_hbox()
}

/// A class's `{equation}` set as fleqn.clo:92-103 sets it, `\hb@xt@\linewidth\bgroup $\m@th\displaystyle
/// \hskip\mathindent` (iopart.cls:771-775, pasj00.cls:4025-4035, pasj01.cls:4098-4110): a math in an `\hbox`, so an
/// author's `$` there ends that math and starts text in the box, and the next begins math again (tex.web §1193,
/// §1138): `{equation}`'s hook (`\lx@equation@hook`, latex_constructs_rust_only.rs) runs `\lx@math@in@hbox`
/// (tex_math.rs), as amsmath's displays do under `\if@fleqn` (amsmath_sty.rs, which sets the hook back when it loads,
/// as amsmath redefines `{equation}`). Their `\[` (fleqn.clo:50-73, iopart.cls:763-769) opens a `\bgroup` after the
/// `$`, which an author's `$` meets as TeX's "Missing } inserted": it keeps the kernel's display, whose `$` errs too.
pub fn math_displays_in_hbox() -> Result<()> {
  DefMacro!("\\lx@equation@hook", "\\lx@math@in@hbox");
  Ok(())
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: fleqn.sty.ltxml — left aligned equations in compatibility mode; fleqn.sty inputs fleqn.clo.
  fleqn_clo()?;
});

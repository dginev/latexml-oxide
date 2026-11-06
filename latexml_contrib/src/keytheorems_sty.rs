//! keytheorems.sty, read raw (no Perl binding), and the head of a `tcolorbox=` theorem.
//!
//! keytheorems stores a tcolorbox theorem's head with `\deferred@thm@head` (keytheorems.sty:981, :1075-1079) for the box's
//! `title=\__keythms_thm_tcbtemphead:` (:994). The amsthm binding's theorem never calls `\deferred@thm@head`: it makes
//! the head its `ltx:title`, before the box. So the macro has nothing to store and its title slot is empty, the head
//! kept (tutodoc-en, tutodoc-fr: "\__keythms_thm_tcbtemphead: undefined"). Guard
//! `perfect_kernel_batchm3::a_tcolorbox_keytheorem_keeps_its_head_and_body`.
use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  InputDefinitions!("keytheorems", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  def_macro(T_CS!(r"\__keythms_thm_tcbtemphead:"), None, Tokens!(), None)?;
});

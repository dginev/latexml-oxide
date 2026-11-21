use crate::prelude::*;
LoadDefinitions!({
  RequirePackage!("amsgen");

  define_amstext_text()?;
});

/// amstext.sty:28-39 `\text`, which setstack.sty:10-19 copies (the `setstack` stand-in, latexml_contrib; 2206.02651).
pub fn define_amstext_text() -> Result<()> {
  // amstext.sty:28-39: `\text` is `\mbox` in text and `\text@` in math, whose `\textdef@` sets `#1` in
  // `\hbox{{…}}`: the argument is digested inside box braces, so an alignment's `&` or `\cr` in it (tex.web §342
  // counts `align_state` at each brace) never ends the cell around it — `\text{\bgroup\let\\=\crcr\egroup}` in an
  // `{align}` cell (1703.08664's `\mWord`, an `\halign` matrix built in a `\text`) is no row end. The argument is
  // read before `\text` digests it, so its own braces are no longer counted: the count is raised for the
  // digestion, as glossaries_sty.rs does for an entry's fields. Perl the same (KNOWN_PERL_ERRORS #931). Guard
  // `perfect_kernel_batchint19::text_argument_is_braced_in_an_alignment_cell`.
  // `\text` reads `#1` as a macro does, at the cell's count: a brace-less `$x$\text & 0.6` ends the cell at the `&`
  // (tex.web §342; 2212.00883). Robust as amstext.sty:28 `\DeclareRobustCommand`; its inner `\text␣` is
  // protected, as Perl's constructor is unexpandable, so an `\edef` keeps `\text` (not `\lx@text`). Guards
  // `perfect_kernel_batchregfixk::{text_without_braces_before_a_cell_end, text_survives_an_edef}`.
  DefMacro!("\\text{}", "\\lx@text{#1}", robust => true, protected => true, locked => true);
  define_text_constructor()
}

/// The `\lx@text` box `\text{#1}` sets its argument in (amstext_sty.rs, revtex4_support_sty.rs), reverting to `\text`.
pub fn define_text_constructor() -> Result<()> {
  DefConstructor!("\\lx@text{}", "<ltx:text _noautoclose='1'>#1</ltx:text>",
    alias => "\\text", mode => "restricted_horizontal", enter_horizontal => true,
    before_digest => { increment_align_group_count(); },
    after_digest => { decrement_align_group_count(); });
  Ok(())
}

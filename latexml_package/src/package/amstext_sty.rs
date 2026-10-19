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
  DefConstructor!("\\text{}", "<ltx:text _noautoclose='1'>#1</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true, locked => true,
    before_digest => { increment_align_group_count(); },
    after_digest => { decrement_align_group_count(); });
  Ok(())
}

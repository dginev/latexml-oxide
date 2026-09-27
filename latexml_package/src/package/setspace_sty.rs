//! setspace.sty — line spacing (no-op in LaTeXML)
//! Perl: setspace.sty.ltxml
use crate::prelude::*;

LoadDefinitions!({
  def_macro_noop("\\singlespacing")?;
  def_macro_noop("\\onehalfspacing")?;
  def_macro_noop("\\doublespacing")?;
  def_macro_noop("\\setstretch{}")?;
  def_macro_noop("\\SetSinglespace{}")?;
  def_macro_noop("\\setdisplayskipstretch{}")?;
  def_macro_noop("\\restore@spacing")?;

  // {spacing}{}: the body-wrapping variant. Keep BOUND_MODE vertical so `$$`
  // inside it still enters display math. Witness 2305.08368:
  // `\begin{spacing}{1.25}` wrapping the whole body made `$$x_1=...$$` fall
  // through the `$` handler's vertical-only check (tex_math.rs:447) → 199
  // `Error:unexpected:_` cascades. Perl-faithful binding is `#body` only, but
  // the default Package.pm mode is `restricted_horizontal`; in Rust we have
  // to make `internal_vertical` explicit so paragraphs and display math
  // survive the wrap.
  //
  // Every one ends with `\par` (setspace.sty:489-548, `\restore@spacing` :516-523), so its last
  // paragraph closes before the end and the text after it starts a new one (a transparent `#body`
  // left it open, 57l); `{spacing}` begins with `\par` (:525-526) and `{singlespace}` with
  // `\vskip` (:489-495), which end the paragraph before them, while `{onehalfspace}` and
  // `{doublespace}` begin with `\begingroup` alone (:534-548) and go on in it (KPE #320).
  DefEnvironment!("{spacing}{}", "#body", mode => "internal_vertical",
    before_digest_end => { leave_horizontal()?; });
  DefEnvironment!("{singlespace}", "#body", mode => "internal_vertical",
    before_digest_end => { leave_horizontal()?; });
  // The paragraph these two close was opened outside their group, which a mode frame cannot end
  // (ARCHITECTURE_THEMES 1), so the `\par` is the document's: the body goes on in the current
  // paragraph and its `<p>` closes after it, where the text that follows starts a new one.
  DefEnvironment!("{onehalfspace}", sub[document, _args, props] {
    body_closing_its_paragraph(document, props)
  });
  DefEnvironment!("{doublespace}", sub[document, _args, props] {
    body_closing_its_paragraph(document, props)
  });
  // Standalone-switch overrides: some papers (witness 2310.08233 IEEEtran)
  // use `\singlespace` as a SWITCH inside an arg-grabbing context such as
  // `\title{\singlespace ...}`. DefEnvironment binds `\singlespace` to the
  // env-begin CS, which pushes a (restricted_)horizontal mode-switch group.
  // That group cannot be cleanly popped at arg-close, leaking into the
  // following `\@add@frontmatter@now`. Override the env-begin CS (and its
  // sibling `\endsinglespace`) with plain noops so standalone use is inert.
  // For `\begin{singlespace}...\end{singlespace}` blocks: \begin/\end track
  // env scope themselves; the begin/end CSes are just hooks. Without the
  // hooks the body still renders (it is just literal #body), so the env
  // form remains correct.
  def_macro_noop("\\singlespace")?;
  def_macro_noop("\\endsinglespace")?;
  def_macro_noop("\\onehalfspace")?;
  def_macro_noop("\\endonehalfspace")?;
  def_macro_noop("\\doublespace")?;
  def_macro_noop("\\enddoublespace")?;
});

/// A setspace environment's body, then the `\par` its end runs (`\restore@spacing`, setspace.sty:516)
/// as the closing of the current `<p>`.
fn body_closing_its_paragraph(document: &mut Document, props: &SymHashMap<Stored>) -> Result<()> {
  if let Some(Stored::Digested(body)) = props.get("body") {
    document.absorb(body, None)?;
  }
  document.maybe_close_element("ltx:p")?;
  Ok(())
}

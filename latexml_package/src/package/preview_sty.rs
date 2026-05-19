use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!({
  // Perl: preview.sty.ltxml — stub to avoid errors
  // preview is used for extracting previews of specific environments;
  // not practically useful with LaTeXML

  DefConditional!("\\ifPreview");

  for option in [
    "noconfig", "delayed", "psfixbb", "dvips", "pdftex", "xetex", "auctex", "lyx",
    "showlabels", "tightpage", "counters", "tracingall", "showbox",
  ] {
    DeclareOption!(option, None);
  }
  // Perl preview.sty.ltxml L40-41: per-environment options push their
  // name into the `preview_environments` stacked value so downstream
  // tooling can check which envs are marked for preview extraction.
  // Rust had empty no-op stubs — port the push so any consumer of the
  // stored value (e.g. a future preview post-processor) sees the set.
  for option in ["displaymath", "textmath", "graphics", "floats", "sections", "footnotes"] {
    let opt = option;
    DeclareOption!(opt, {
      let _ = state::push_value("preview_environments", Stored::from(opt));
    });
  }
  // Perl L43: `active` option digests `\Previewtrue` to flip the
  // \ifPreview conditional on. Without this, \ifPreview stays false and
  // the `active` option has no observable effect.
  DeclareOption!("active", {
    Digest!("\\Previewtrue")?;
  });

  def_macro_noop("\\PreviewMacro OptionalMatch:* []{}")?;
  def_macro_noop("\\PreviewEnvironment OptionalMatch:* []{}")?;
  def_macro_noop("\\PreviewSnarfEnvironment OptionalMatch:* []{}")?;
  def_macro_noop("\\PreviewOpen OptionalMatch:* []{}")?;
  def_macro_noop("\\PreviewClose OptionalMatch:* []{}")?;

  DefEnvironment!("{preview}", "#body");
  DefEnvironment!("{nopreview}", "#body");

  DefRegister!("\\PreviewBorder", Dimension(0));
});

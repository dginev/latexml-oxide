use latexml_package::prelude::*;


LoadDefinitions!({
  Warn!(
    "missing_file",
    "mdframed.sty",
    "mdframed.sty is only minimally stubbed and will not be interpreted raw."
  );
  RequirePackage!("kvoptions");
  RequirePackage!("xparse");
  RequirePackage!("etoolbox");
  RequirePackage!("xcolor");
  def_macro_noop("\\newmdtheoremenv[]{}{}[]")?;
  def_macro_noop("\\newmdenv[]{}")?;
  def_macro_noop("\\renewmdenv[]{}")?;
  def_macro_noop("\\surroundwithmdframed[]{}")?;
  def_macro_noop("\\mdfsubtitle[]{}")?;
  def_macro_noop("\\mdfapptodefinestyle{}{}")?;
  def_macro_noop("\\mdfsetup{}")?;
  def_macro_noop("\\mdfdefinestyle{}{}")?;
  DefRegister!("\\mdflength" => Dimension::new(0));
  // Perl ar5iv-bindings/mdframed.sty.ltxml L31-34: wrap body in an
  // inline-block with framed="rectangle" and framecolor from the current
  // font color (`LookupValue('font')->getColor`). The template emits
  // `framecolor=` only when the #framecolor property is set (via the
  // `?#framecolor(...)` guard), so an unset color correctly omits the
  // attribute rather than emitting `framecolor=''`.
  DefEnvironment!(
    "{mdframed}[]",
    "<ltx:inline-block framed='rectangle' ?#framecolor(framecolor='#framecolor') _noautoclose='1'>#body</ltx:inline-block>",
    properties => sub[_args] {
      let mut props = arena::SymHashMap::default();
      if let Some(font) = latexml_core::state::lookup_font() {
        if let Some(color) = font.get_color() {
          props.insert("framecolor", Stored::from(color.to_attribute()));
        }
      }
      Ok(props)
    }
  );
});

use latexml_package::prelude::*;


LoadDefinitions!({
  Warn!(
    "missing_file",
    "scrbook.cls",
    "scrbook.cls is only minimally stubbed and will not be interpreted raw."
  );
  LoadClass!("OmniBus");
  def_macro_noop("\\setkomafont{}{}")?;
  def_macro_noop("\\setcapindent{}")?;
  def_macro_noop("\\deffootnote[]{}{}{}")?;
  def_macro_noop("\\deffootnotemark{}")?;
  // KOMA page-style marks — see scrartcl_cls.rs for rationale.
  def_macro_noop("\\headmark")?;
  def_macro_noop("\\pagemark")?;
});

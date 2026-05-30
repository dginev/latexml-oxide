use latexml_package::prelude::*;


LoadDefinitions!({
  Warn!(
    "missing_file",
    "scrbook.cls",
    "scrbook.cls is only minimally stubbed and will not be interpreted raw."
  );
  LoadClass!("OmniBus");
  // Real scrbook.cls loads the KOMA dependency chain (scrkbase, tocbasic, …),
  // which transitively loads `iftex`, defining `\ifpdf`/`\ifpdftex`/… for
  // author engine/driver detection. Perl raw-loads the .cls and gets iftex
  // that way; our OmniBus stub intercepts it. Mirror the dependency (same
  // rationale as scrartcl_cls.rs, witness 1802.07175).
  RequirePackage!("iftex");
  def_macro_noop("\\setkomafont{}{}")?;
  def_macro_noop("\\setcapindent{}")?;
  def_macro_noop("\\deffootnote[]{}{}{}")?;
  def_macro_noop("\\deffootnotemark{}")?;
  // KOMA page-style marks — see scrartcl_cls.rs for rationale.
  def_macro_noop("\\headmark")?;
  def_macro_noop("\\pagemark")?;
});

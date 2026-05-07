use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pgfrcs.sty.ltxml
  InputDefinitions!("pgfutil-common", extension => Some(Cow::Borrowed("tex")));
  InputDefinitions!("pgfutil-latex",  extension => Some(Cow::Borrowed("def")));
  InputDefinitions!("pgfrcs.code",    extension => Some(Cow::Borrowed("tex")));

  // Pre-set \pgfsysdriver and re-route pgfutil's IfFileExists so
  // pgfsys.code.tex's `\pgfutil@InputIfFileExists{\pgfsysdriver}{}{...}`
  // resolves to our pgfsys-latexml.def binding. The pgf.sty binding does
  // this too, but `\usepackage{nicematrix}` (driver: 2402.09676) and
  // similar packages load pgfcore.sty DIRECTLY — bypassing pgf.sty —
  // so the assignment had never fired. pgfrcs is the earliest-loaded
  // pgf-stack binding, so pre-set here.
  DefMacro!("\\pgfsysdriver", "pgfsys-latexml.def");
  state::assign_value("pgfsys-latexml.def_binding_available", true, Some(Scope::Global));
  Let!("\\pgfutil@IfFileExists", "\\IfFileExists");
});

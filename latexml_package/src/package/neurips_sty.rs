use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("geometry");
  RequirePackage!("lineno");
  DeclareOption!("final", {
    state::assign_value("neurips_final", Stored::from(1), Some(Scope::Global));
  });
  DeclareOption!("preprint", {
    state::assign_value("neurips_preprint", Stored::from(1), Some(Scope::Global));
  });
  DeclareOption!("nonatbib", {
    state::assign_value("neurips_nonatbib", Stored::from(1), Some(Scope::Global));
  });
  ProcessOptions!();
  if state::lookup_value("neurips_nonatbib").is_none() {
    RequirePackage!("natbib");
  }
  DefMacro!("\\AND",                                   "");
  DefMacro!("\\And",                                   "");
  DefMacro!("\\bottomfraction",                        "");
  DefMacro!("\\patchAmsMathEnvironmentForLineno",      "");
  DefMacro!("\\patchBothAmsMathEnvironmentsForLineno", "");
  DefMacro!("\\subsubsubsection", "\\@startsection{subsubsubsection}{4}{}{}{}{}");
  DefMacro!("\\textfraction", "");
  DefMacro!("\\topfraction",  "");
  DefMacro!("\\@neuripsordinal",  "36th");
  DefMacro!("\\@neuripsyear",     "2022");
  DefMacro!("\\@neuripslocation", "New Orleans");
  DefMacro!("\\acksection", "\\section*{Acknowledgments and Disclosure of Funding}");
  DefMacro!("\\answerYes[]",  "\\textcolor{blue}{[Yes] #1}");
  DefMacro!("\\answerNo[]",   "\\textcolor{orange}{[No] #1}");
  DefMacro!("\\answerNA[]",   "\\textcolor{gray}{[N/A] #1}");
  DefMacro!("\\answerTODO[]", "\\textcolor{red}{\\bf [TODO]}");

  // {ack} environment — Perl L51-52
  DefEnvironment!("{ack}", "#body");

  // {hide} environment — Perl L59
  DefEnvironment!("{hide}", "");
});

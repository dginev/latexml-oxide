use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\subfloat",
    "\\ifx\\@captype\\@undefined\\expandafter\\@gobble\\else\\expandafter\\@firstofone\\fi{\\sf@subfloat}");
  DefMacro!("\\sf@subfloat",
    "\\csname lx@subfloat@\\@captype\\endcsname");
  DefMacro!("\\subref",       "\\@ifstar\\sf@@subref\\sf@subref");
  DefMacro!("\\sf@subref{}",  "\\ref{sub@#1}");
  DefMacro!("\\sf@@subref{}", "\\pageref{sub@#1}");
  DefMacro!("\\DeclareCaptionListOfFormat{}{}", "");
  DefMacro!("\\DeclareSubrefFormat{}{}", "");
  DefMacro!("\\listsubcaptions", "");
  DefMacro!("\\captionsetup[]{}", "");
  DefMacro!("\\clearcaptionsetup{}", "");
  DefConditional!("\\ifmaincaptiontop");
  RawTeX!("\\@ifundefined{c@subfigure}{\\newcounter{subfigure}[figure]}{}");
  RawTeX!("\\@ifundefined{c@subtable}{\\newcounter{subtable}[table]}{}");
});

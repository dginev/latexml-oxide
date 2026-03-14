use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: rotating.sty.ltxml
  // Stub: rotatedProperties() not yet ported, so rotation attributes are omitted.

  DeclareOption!("twoside", None);
  DeclareOption!("figuresright", None);
  DeclareOption!("figuresleft", None);
  DeclareOption!("quiet", None);
  DeclareOption!("log", None);
  DeclareOption!("chatter", None);
  ProcessOptions!();

  RequirePackage!("graphicx");
  RequirePackage!("ifthen");

  TeX!(r"\newdimen\rotFPtop \rotFPtop=0pt
\newdimen\rotFPbot \rotFPbot=0pt
");

  DefEnvironment!("{sideways}",
    "<ltx:inline-block>#body</ltx:inline-block>");

  DefEnvironment!("{turn}{Float}",
    "<ltx:inline-block>#body</ltx:inline-block>");

  DefEnvironment!("{rotate}{Float}",
    "<ltx:inline-block>#body</ltx:inline-block>");

  DefConstructor!("\\turnbox{Float} {}",
    "<ltx:inline-block>#2</ltx:inline-block>",
    mode => "internal_vertical");

  // sidewaysfigure/sidewaystable — simplified stubs (no beforeFloat/afterFloat yet)
  DefEnvironment!("{sidewaysfigure}[]",
    "<ltx:figure xml:id='#id' ?#1(placement='#1')>#tags#body</ltx:figure>",
    mode => "internal_vertical");

  DefEnvironment!("{sidewaysfigure*}[]",
    "<ltx:figure xml:id='#id' ?#1(placement='#1')>#tags#body</ltx:figure>",
    mode => "internal_vertical");

  DefEnvironment!("{sidewaystable}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>#tags#body</ltx:table>",
    mode => "internal_vertical");

  DefEnvironment!("{sidewaystable*}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' ?#1(placement='#1')>#tags#body</ltx:table>",
    mode => "internal_vertical");

  DefMacro!("\\rotcaption{}", r"\caption{\turnbox{90}{#1}}");
});

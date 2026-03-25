use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: a0poster.cls.ltxml
  RequirePackage!("a0size");

  DefConditional!("\\ifportrait");
  DefConditional!("\\ifanullb");
  DefConditional!("\\ifanull");
  DefConditional!("\\ifaeins");
  DefConditional!("\\ifazwei");
  DefConditional!("\\ifadrei");
  DefConditional!("\\ifposterdraft");

  DeclareOption!("a0b",         "\\anullbtrue");
  DeclareOption!("a0",          "\\anulltrue \\anullbfalse");
  DeclareOption!("a1",          "\\aeinstrue \\anullbfalse");
  DeclareOption!("a2",          "\\azweitrue \\anullbfalse");
  DeclareOption!("a3",          "\\adreitrue \\anullbfalse");
  DeclareOption!("landscape",   "\\portraitfalse");
  DeclareOption!("portrait",    "\\portraittrue");
  DeclareOption!("draft",       "\\posterdrafttrue");
  DeclareOption!("posterdraft", "\\posterdrafttrue");
  DeclareOption!("final",       "\\posterdraftfalse");

  Digest!("\\ExecuteOptions{landscape,a0b,final}")?;
  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;

  RawTeX!(r"\ifanullb
  \setlength { \paperwidth }{ 119 cm }
  \setlength { \paperheight }{ 87 cm }
  \setlength { \textwidth }{ 114 cm }
  \setlength { \textheight }{ 87 cm }
\else \ifanull
  \setlength { \paperwidth }{ 118.82 cm }
  \setlength { \paperheight }{ 83.96 cm }
  \setlength { \textwidth }{ 114.82 cm }
  \setlength { \textheight }{ 79.96 cm }
\else \ifaeins
  \setlength { \paperwidth }{ 83.96 cm }
  \setlength { \paperheight }{ 59.4 cm }
  \setlength { \textwidth }{ 79.96 cm }
  \setlength { \textheight }{ 55.4 cm }
\else \ifazwei
  \setlength { \paperwidth }{ 59.4 cm }
  \setlength { \paperheight }{ 41.98 cm }
  \setlength { \textwidth }{ 55.4 cm }
  \setlength { \textheight }{ 37.98 cm }
\else \ifadrei
  \setlength { \paperwidth }{ 41.98 cm }
  \setlength { \paperheight }{ 29.7 cm }
  \setlength { \textwidth }{ 37.98 cm }
  \setlength { \textheight }{ 25.7 cm }
\else \relax
\fi\fi\fi\fi\fi

\ifportrait
  \newdimen \tausch
  \setlength { \tausch }{ \paperwidth }
  \setlength { \paperwidth }{ \paperheight }
  \setlength { \paperheight }{ \tausch }
  \setlength { \tausch }{ \textwidth }
  \setlength { \textwidth }{ \textheight }
  \setlength { \textheight }{ \tausch }
\else \relax
\fi");

  AssignValue!("NOMINAL_FONT_SIZE", 25);
  DefPrimitive!("\\tiny",         None, font => {size => 12 });
  DefPrimitive!("\\scriptsize",   None, font => {size => 14.4 });
  DefPrimitive!("\\footnotesize", None, font => {size => 17.28 });
  DefPrimitive!("\\small",        None, font => {size => 20.74 });
  DefPrimitive!("\\normalsize",   None, font => {size => 25 });
  DefPrimitive!("\\large",        None, font => {size => 29.86 });
  DefPrimitive!("\\Large",        None, font => {size => 35.83 });
  DefPrimitive!("\\LARGE",        None, font => {size => 43 });
  DefPrimitive!("\\huge",         None, font => {size => 51.6 });
  DefPrimitive!("\\Huge",         None, font => {size => 61.92 });
  DefPrimitive!("\\veryHuge",     None, font => {size => 74.3 });
  DefPrimitive!("\\VeryHuge",     None, font => {size => 89.16 });
  DefPrimitive!("\\VERYHuge",     None, font => {size => 107 });
});

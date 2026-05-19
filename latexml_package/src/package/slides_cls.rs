use crate::prelude::*;
/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!( {
  LoadPool!("LaTeX");
  //**********************************************************************
  // Option handling
  for option in [
    "letterpaper",
    "legalpaper",
    "executivepaper",
    "a4paper",
    "a5paper",
    "b5paper",
    "landscape",
    "clock",
    "final",
    "draft",
    "titlepage",
    "notitlepage",
    "onecolumn",
    "twocolumn",
  ]
  .iter()
  {
    DeclareOption!(*option, None);
  }

  DeclareOption!("leqno", sub { AssignMapping!("DOCUMENT_CLASSES", "ltx_leqno" => true); });
  DeclareOption!("fleqn", sub { AssignMapping!("DOCUMENT_CLASSES", "ltx_fleqn" => true); });

  ProcessOptions!();

  //**********************************************************************
  // Document structure.
  RelaxNGSchema!("LaTeXML");

  DefConditional!("\\if@bw");
  DefConditional!("\\if@clock");
  DefConditional!("\\if@makingslides");
  DefConditional!("\\if@onlynotesw");
  DefConditional!("\\if@onlyslidesw");
  DefConditional!("\\if@titlepage");
  DefConditional!("\\if@visible");

  DefMacro!("\\ifourteenpt",   "13.82");
  DefMacro!("\\iseventeenpt",  "16.59");
  DefMacro!("\\itwentypt",     "19.907");
  DefMacro!("\\itwentyfourpt", "23.89");
  DefMacro!("\\itwentyninept", "28.66");
  DefMacro!("\\ithirtyfourpt", "34.4");
  DefMacro!("\\ifortyonept",   "41.28");

  // \newifg Token — just delegates to \newif
  DefMacro!("\\newifg Token", "\\newif#1");
  DefConditional!("\\ifG@slidesw");

  Let!("\\@topfil", "\\vfil");
  Let!("\\@botfil", "\\vfil");

  DefConstructor!("\\addtime Number", "<ltx:note>add time #1</ltx:note>");
  DefConstructor!("\\settime Number", "<ltx:note>set time #1</ltx:note>");

  // Hmm... should be saving the color and restoring upon visible.
  // CSS3 has an opacity property (0--1)
  DefPrimitive!("\\invisible", None, font => {opacity => "0" });
  DefPrimitive!("\\visible",   None, font => {opacity => "1" });

  // \showfont — debug helper, no-op in Rust
  DefPrimitive!("\\showfont", None);

  RequirePackage!("color");
  def_macro_noop("\\blackandwhite")?;
  def_macro_noop("\\colors{}")?;
  def_macro_noop("\\colorslides{}")?;

  def_macro_noop("\\setupcounters")?;

  def_macro_noop("\\ps@headings")?;
  def_macro_noop("\\ps@note")?;
  def_macro_noop("\\ps@overlay")?;
  def_macro_noop("\\ps@slide")?;

  //**********************************************************************
  // The core sectioning commands are defined in LaTeX.pm
  // but the counter setup, etc, depends on slides
  NewCounter!("slide",   "document", idprefix => "s",  nested => vec!["overlay"]);
  NewCounter!("overlay", "slide",    idprefix => "o");
  NewCounter!("note",    "document", idprefix => "n");

  DefMacro!("\\theslide",   "\\arabic{slide}");
  DefMacro!("\\thenote",    "\\arabic{note}");
  DefMacro!("\\theoverlay", "\\theslide.\\arabic{overlay}");

  NewCounter!("equation", "document", idprefix => "E");
  DefMacro!("\\theequation", "\\arabic{equation}");

  NewCounter!("@itemizei", "document", idprefix => "I");

  DefMacro!("\\theenumi",   "\\arabic{enumi}");
  DefMacro!("\\theenumii",  "\\alph{enumii}");
  DefMacro!("\\theenumiii", "\\roman{enumiii}");
  DefMacro!("\\theenumiv",  "\\Alph{enumiv}");

  //**********************************************************************

  AssignValue!("DOSLIDES" => true, Some(Scope::Global));
  AssignValue!("DONOTES"  => true, Some(Scope::Global));

  DefPrimitive!("\\onlynotes", {
    AssignValue!("DONOTES"  => true,  Some(Scope::Global));
    AssignValue!("DOSLIDES" => false, Some(Scope::Global));
  });

  DefPrimitive!("\\onlyslides", {
    AssignValue!("DONOTES"  => false, Some(Scope::Global));
    AssignValue!("DOSLIDES" => true,  Some(Scope::Global));
  });

  DefEnvironment!("{slide}",
    "<ltx:slide xml:id='#id'>#tags#body</ltx:slide>",
    properties => { RefStepCounter!("slide") }
  );

  DefEnvironment!("{overlay}",
    "<ltx:slide xml:id='#id'>#tags#body</ltx:slide>",
    properties => { RefStepCounter!("overlay") }
  );

  DefEnvironment!("{note}",
    "<ltx:note xml:id='#id'>#tags#body</ltx:note>",
    properties => { RefStepCounter!("note") }
  );

  //======================================================================
  DefPrimitive!("\\tiny",         None, font => {size => 5 });
  DefPrimitive!("\\scriptsize",   None, font => {size => 7 });
  DefPrimitive!("\\footnotesize", None, font => {size => 8 });
  DefPrimitive!("\\small",        None, font => {size => 9 });
  DefPrimitive!("\\normalsize",   None, font => {size => 10 });
  DefPrimitive!("\\large",        None, font => {size => 12 });
  DefPrimitive!("\\Large",        None, font => {size => 14.4 });
  DefPrimitive!("\\LARGE",        None, font => {size => 17.28 });
  DefPrimitive!("\\huge",         None, font => {size => 20.74 });
  DefPrimitive!("\\Huge",         None, font => {size => 29.8 });
});

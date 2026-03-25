use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  DefMacro!("\\Affilfont", "\\normalfont");
  DefMacro!("\\Authfont",  "\\normalfont");
  DefMacro!("\\Authsep",   ",");
  DefMacro!("\\Authand",   " and ");
  DefMacro!("\\Authands",  ", and ");
  DefMacro!("\\authorcr",  "\\\\");
  DefConditional!("\\ifnewaffil");
  DefRegister!("\\affilsep" =>  Dimension::from_str("1em")?);
  DefRegister!("\\@affilsep" => Dimension::from_str("1em")?);
  NewCounter!("Maxaffil");
  NewCounter!("authors");
  NewCounter!("affil");
  NewCounter!("@affil");
  DefMacro!("\\the@affil", "affil\\arabic{@affil}");
  DefMacro!("\\author[]{}",
    "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#2}}");
  DefConstructor!("\\affil[]{}",
    "^ <ltx:note role='affiliationtext'>#2</ltx:note>");
  DefMacro!("\\AB@authnote{}",  "\\textsuperscript{\\normalfont#1}");
  DefMacro!("\\AB@affilnote{}", "\\textsuperscript{\\normalfont#1}");
});

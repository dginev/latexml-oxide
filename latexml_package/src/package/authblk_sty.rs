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

  // Perl L40-42: \author splits on \and and ,
  // Simplified: just add each author as a creator
  DefMacro!("\\lx@ab@author[]{}",
    "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#2}\\lx@split@authormark{#1}}");
  DefMacro!("\\author[]{}", "\\lx@ab@author[#1]{#2}");

  // Perl L50-54: split marks on commas, create affiliationmark elements
  // Simplified: single mark per author
  DefMacro!("\\lx@split@authormark{}", "\\lx@authormark{#1}");
  DefConstructor!("\\lx@authormark{}",
    "^ <ltx:contact role='affiliationmark'>#1</ltx:contact>");

  // Perl L60-69: \affil with mark counter
  DefConstructor!("\\affil[]{}",
    "^ <ltx:note role='affiliationtext'>#2</ltx:note>");

  // Perl L95-96: note formatting
  DefMacro!("\\AB@authnote{}",  "\\textsuperscript{\\normalfont#1}");
  DefMacro!("\\AB@affilnote{}", "\\textsuperscript{\\normalfont#1}");
});

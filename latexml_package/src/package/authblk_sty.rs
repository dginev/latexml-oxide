use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: authblk.sty.ltxml — 100 lines
  // Author/affiliation blocks with mark-based association

  // Font/separator macros — Perl L22-27
  DefMacro!("\\Affilfont", "\\normalfont");
  DefMacro!("\\Authfont",  "\\normalfont");
  DefMacro!("\\Authsep",   ",");
  DefMacro!("\\Authand",   " and ");
  DefMacro!("\\Authands",  ", and ");
  DefMacro!("\\authorcr",  "\\\\");

  // Bookkeeping — Perl L30-38
  DefConditional!("\\ifnewaffil");
  DefRegister!("\\affilsep" =>  Dimension::from_str("1em")?);
  DefRegister!("\\@affilsep" => Dimension::from_str("1em")?);
  NewCounter!("Maxaffil");
  RawTeX!("\\setcounter{Maxaffil}{2}");
  NewCounter!("authors");
  NewCounter!("affil");
  NewCounter!("@affil");
  DefMacro!("\\the@affil", "affil\\arabic{@affil}");

  // \author — Perl L40-46
  // Perl splits on \and and comma; simplified to single author per call
  DefMacro!("\\lx@ab@author[]{}",
    "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#2}\\lx@split@authormark{#1}}");
  DefMacro!("\\author[]{}", "\\lx@ab@author[#1]{#2}");

  // Mark splitting — Perl L50-54
  // Simplified: passes mark through as single affiliationmark
  DefMacro!("\\lx@split@authormark{}", "\\lx@authormark{#1}");
  DefConstructor!("\\lx@authormark{}",
    "^ <ltx:contact role='affiliationmark' _mark='#1'>#1</ltx:contact>");

  // \affil — Perl L60-69
  DefConstructor!("\\affil[]{}",
    "^ <ltx:note role='affiliationtext' mark='#1'>#2</ltx:note>");

  // Note: Perl has Tag('ltx:document', afterClose => \&authblkRelocateAffil)
  // which does DOM surgery to move affiliationtext into matching creator.
  // This requires deep DOM manipulation not yet available in Rust.

  // Note formatting — Perl L95-96
  DefMacro!("\\AB@authnote{}",  "\\textsuperscript{\\normalfont#1}");
  DefMacro!("\\AB@affilnote{}", "\\textsuperscript{\\normalfont#1}");
});

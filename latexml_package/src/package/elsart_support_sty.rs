//! elsart_support.sty — Elsevier article support (non-core additions)
//! Perl: elsart_support.sty.ltxml — 175 lines
//! Loads elsart_support_core and adds theorem/proof/section formatting
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("elsart_support_core");

  // Theorem stubs (if amsthm not loaded)
  DefMacro!("\\theoremstyle{}", "");
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true);

  // Proof environment — Perl L38-60
  DefEnvironment!("{proof}[]",
    "<ltx:proof><ltx:title font='italic' _force_font='true' class='ltx_runin'>#title</ltx:title>#body</ltx:proof>",
    properties => { stored_map!("title" => Stored::from("Proof")) }
  );

  // Section formatting — Perl L63-120
  // These customize section numbering and font for Elsevier style
  DefMacro!("\\elsartstyle", "");
  DefMacro!("\\semark{}",    "");
  DefMacro!("\\ssmark{}",    "");
  DefMacro!("\\sssmark{}",   "");
  DefMacro!("\\elsmarks",    "");

  // Abstract keywords with continuation
  DefMacro!("\\KWD{}", "\\@add@frontmatter{ltx:keywords}{#1}");
  DefMacro!("\\AMS{}",  "\\@add@frontmatter{ltx:classification}[scheme=MSC]{#1}");
  DefMacro!("\\PAC{}",  "\\@add@frontmatter{ltx:classification}[scheme=PACS]{#1}");

  // Bibliography — Perl L130-175
  DefMacro!("\\biboptions{}", "");
  DefMacro!("\\bibliographystyle{}", "");
  DefMacro!("\\harvarditem[]{}{}{}",
    "\\bibitem[#2(#3)]{#4}");
  DefMacro!("\\harvardand", "\\&");
  DefMacro!("\\harvardurl{}", "\\url{#1}");
  DefMacro!("\\harvestremark{}", "");
  DefMacro!("\\harvardyearleft", "(");
  DefMacro!("\\harvardyearright", ")");
  DefMacro!("\\citestyle{}", "");
});

//! apa6.cls (APA 6th edition manuscripts, TeX Live apa6 2023/08/07 v2.35): the class is loaded raw, and its numbered author
//! forms — `\twoauthors` … `\sixauthors`, `\onetwoauthors`/`\twooneauthors` (apa6.cls:745-759) and `\twoaffiliations` …
//! `\sixaffiliations`, `\onetwoaffiliations`/`\twooneaffiliations` (:761-777), which store each name in `\@authorOne` …
//! `\@authorSix` and each affiliation in `\@affilOne` … `\@affilSix` — become the frontmatter as its title page lays
//! them out: every layout (man :1143-1214; jou and doc `\put@one@authaffil`/`\put@two@authaffil` :1321-1365 at :1378-1414)
//! sets the n-th name over the n-th affiliation, so each name is a creator carrying the affiliation of its number.
//! Under the raw class (the rawclasses lever) only the class's refused `\maketitle` read those stores, so the names and
//! affiliations were lost (2102.04789: "Najko Jahn, Lisa Matthias, Mikael Laakso" and their Göttingen address;
//! 1710.04173 `\sixauthors`/`\sixaffiliations`). The one-author `\author`/`\affiliation` stay the kernel's and K11's.
use latexml_package::prelude::*;

/// The numbered author and affiliation forms of apa.cls and apa6.cls (apa.cls:427-459 is apa6.cls:745-777 line for
/// line), handed to the frontmatter at `\maketitle`, where the class reads them: `mask` names the class's
/// author-masking flag (apa6.cls:126 the `mask` option, read at :1143 and :1378; apa.cls has none, `None`).
///
/// The title page picks its layout by which numbered names are defined (:1145-1214, :1381-1414: `\@authorTwo` …
/// `\@authorSix`, each checked only when the one before it is). A name slot may hold a group of names over its one
/// affiliation (1401.2054 `\twoauthors{Zhiyong Zhang, Kaifeng Jiang, Haiyan Liu}{In-Sue Oh}`, 1402.6720
/// `{Edgar C.\ Merkle and Dongjun You}`): the slot is split as the kernel's `\author` splits its names,
/// appended to the slots before it (`\lx@add@authors@list`), and its affiliation given to each of them (`annotate=new`: the creators of this slot, the
/// earlier ones having theirs). A numbered form also sets `\@author` and `\@affil` to
/// its first name and affiliation (:747, :762), so once `\@authorTwo` is defined the numbered names and affiliations
/// are the title page's whatever `\author` and `\affiliation` said: the creators the kernel's `\author` made are
/// cleared (2011.06172 `\author{Author}` "hack around some bugs in apa.cls" beside `\threeauthors`), and so is the
/// one-author affiliation store, which K11 would otherwise give every author at `\maketitle` (`harvest_stores`;
/// 1811.01821 `\affiliation{University of Cambridge}` beside `\twoaffiliations`). Without numbered names the title
/// page sets `\@author` over `\@affil`, a numbered affiliation form's first one included (1605.09288, 1609.04156
/// `\author{…}` with `\twoaffiliations`; the class warns "More affiliations than authors defined", :785): K11's own.
pub fn define_numbered_author_forms(mask: Option<&str>) -> Result<()> {
  let (open, close) = match mask {
    Some(flag) => (s!(r"\@ifundefined{{{flag}}}{{"), "}{}"),
    None => (String::new(), ""),
  };
  RawTeX!(&s!(
    r"\def\lx@apa@pair#1{{\expandafter\expandafter\expandafter\lx@add@authors@list\expandafter\expandafter\expandafter{{\csname @author#1\endcsname}}\@ifundefined{{@affil#1}}{{}}{{\expandafter\expandafter\expandafter\lx@apa@affiliation\expandafter\expandafter\expandafter{{\csname @affil#1\endcsname}}}}}}
\def\lx@apa@affiliation#1{{\lx@add@affiliation[annotate=new]{{#1}}}}
\def\lx@apa@numbered@authors{{\@ifundefined{{@authorTwo}}{{}}{{\global\let\@affil\@empty{open}\lx@clear@creators\lx@apa@pair{{One}}\lx@apa@pair{{Two}}\@ifundefined{{@authorThree}}{{}}{{\lx@apa@pair{{Three}}\@ifundefined{{@authorFour}}{{}}{{\lx@apa@pair{{Four}}\@ifundefined{{@authorFive}}{{}}{{\lx@apa@pair{{Five}}\@ifundefined{{@authorSix}}{{}}{{\lx@apa@pair{{Six}}}}}}}}}}{close}}}\global\let\lx@apa@numbered@authors\relax}}"
  ));
  AddToMacro!("\\lx@maketitle@begin", "\\lx@apa@numbered@authors");
  Ok(())
}

/// Interpret the class `name` raw (as the document asked for it: [`crate::shipped_class::requested_shipped_class`], so
/// a class only named like the family — TeX Live's apa6e.cls, a paper's `apa6mod.cls`, which the prefix fallback
/// answers with this binding — is read as itself), then give its numbered author forms to the frontmatter
/// ([`define_numbered_author_forms`]) when it has them. A paper whose source lacks the class falls to OmniBus.
pub fn apa_family_class(name: &str, mask: Option<&str>) -> Result<()> {
  let Some(class) = crate::shipped_class::requested_shipped_class(name)? else {
    LoadClass!("OmniBus");
    return Ok(());
  };
  InputDefinitions!(&class, noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // The class read raw by a binding is still a raw class to K11, as when the rawclasses lever reads it: its title-page
  // stores (`\shorttitle`, `\affiliation`, `\authornote`, `\keywords`, `\abstract`, `\note`, …) reach the
  // frontmatter (`reroute_raw_class_stores`, fired for a raw load by `input_definitions`, which a binding's own load of
  // the file does not do).
  RawTeX!(&s!(r"\lx@class@loaded@raw{{{class}}}"));
  // (a class only named like the family, without the numbered stores, is left as it is)
  if !IsDefined!(&T_CS!("\\twoauthors")) || !IsDefined!(&T_CS!("\\twoaffiliations")) {
    return Ok(());
  }
  define_numbered_author_forms(mask)
}

LoadDefinitions!({
  apa_family_class("apa6", Some("apaSix@maskauthoridentity"))?;
});

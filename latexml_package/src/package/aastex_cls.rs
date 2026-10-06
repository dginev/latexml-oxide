use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aastex.cls.ltxml — AAS TeX document class

  // Ignorable options
  //
  // Note on `revtex4`: Perl #2698 (2026) makes it an explicit no-op
  // because the class now loads revtex4 unconditionally anyway. Adding
  // it here prevents the option from falling through to the article
  // fallback below and getting spuriously flagged.
  for option in [
    "10pt", "11pt", "12pt",
    "manuscript", "preprint", "preprint2", "longabstract",
    "tighten", "landscape",
    "aasms4", "aaspp4", "aas2pp4", "aj_pt4", "apjpt4", "astro",
    "flushrt", "anonymous",
    "revtex4",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // Number equations within sections
  DeclareOption!("eqsecnum", "\\AtEndOfClass{\\eqsecnum}");

  // Anything else is for article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  ProcessOptions!();

  load_class("revtex4", Vec::new(), Tokens!())?;
  // aastex7.cls:11484 / aastex701.cls:11416 `\usepackage[figuresright]{rotating}`, ahead of their own `\rotate`
  // (:12264 / :12196), which a deluxetable's `\rotate` (deluxetable_sty.rs) must stay; aastex631 and earlier do not load
  // it. Read from the class file after this binding (the versioned fallback's dependency scan), rotating's
  // `{rotate}` environment replaced it: `\rotate` in a `deluxetable*` opened a box that never closed (2609.09266).
  let request = lookup_string("fallback_request");
  if request.rsplit('/').next().is_some_and(|name| name.starts_with("aastex7")) {
    RequirePackage!("rotating", options => vec![s!("figuresright")]);
  }
  RequirePackage!("aas_support");
  // aastex701.cls:13637-13638 — `\digitalasset` flags a digital-asset paper
  // (aastex701-sample; the Perl reimplementation lacks it too).
  RawTeX!(r"\newif\ifdigitasset\def\digitalasset{\digitassettrue}");
  // aastex701.cls:13494-13497 — `\centerwidetable` centers a wide deluxetable instead of setting it sideways, a page
  // layout flag (2609.06985).
  RawTeX!(r"\newif\ifcenterwidetable\def\centerwidetable{\global\centerwidetabletrue}");
  // aastex701.cls:11756, 13690 — the offsets a table is moved by on its page (`\movetabledown=2cm`, 2609.00308).
  RawTeX!(r"\newdimen\movetabledown\newdimen\movetableright");
});

//! Stub for IMS (Institute of Mathematical Statistics) `imsart` class.
//!
//! imsart.cls loads `article` + requires `imsart.sty` (support file with
//! \startlocaldefs, \endlocaldefs, etc.). We fall back to OmniBus and raw-
//! load imsart.sty so most user macros become available.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // imsart.cls L149: \RequirePackage{imsart}.
  InputDefinitions!("imsart", noltxml => true, extension => Some(Cow::Borrowed("sty")));

  // Frontmatter primitives commonly used in imsart papers but not
  // always defined by imsart.sty (some are journal-driver dependent).
  // \startlocaldefs / \endlocaldefs are defined in imsart.sty L657-660;
  // these are belt-and-suspenders in case the raw load is short-circuited.
  DefMacro!("\\startlocaldefs", "\\makeatletter");
  DefMacro!("\\endlocaldefs", "\\makeatother");
  // imsart.sty L2268, L2360: \let\kwd@sep\relax inside conditionals
  // we may not fully replay. Define defensively. Witness 2406.17390.
  Let!("\\kwd@sep", "\\relax");

  // {funding} env — IMS journal funding-statement frontmatter.
  // Preserve as ltx:note (content-preservation directive). Witness
  // 2406.15844 (+5 imsart papers). Use internal_vertical mode so the
  // body can contain paragraphs / lists without tripping
  // mode-mismatch errors.
  DefEnvironment!("{funding}",
    "<ltx:note role='funding'>#body</ltx:note>",
    mode => "internal_vertical");
  // {acknowledgement} / {acknowledgements} aliases for spelling variants.
  DefEnvironment!("{acknowledgement}",
    "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  // {acks} env — IMS-specific acknowledgements ("acks" shorthand).
  // Witness 2406.15844, 2406.04191, 2406.02840 (3 imsart papers).
  DefEnvironment!("{acks}",
    "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  // IMS authors use \orcid for ORCID identifier. Preserve as ltx:note.
  DefMacro!("\\orcid{}",
    "\\@add@frontmatter{ltx:note}[role=orcid]{#1}");
  // IMS journal bibliography entry types — imsart.sty defines these as
  // \def commands but they're used as environments in some .bbl files.
  // Provide as no-op envs (the actual bibliography rendering is handled
  // by biblatex/natbib elsewhere). Witness 2406.15844 (+4 imsart papers).
  DefEnvironment!("{barticle}", "#body");
  DefEnvironment!("{bbook}",    "#body");
  DefEnvironment!("{bbooklet}", "#body");
  DefEnvironment!("{binbook}",  "#body");
  DefEnvironment!("{bincollection}", "#body");
  DefEnvironment!("{bunpublished}",  "#body");
  DefEnvironment!("{bmisc}",         "#body");
  DefEnvironment!("{bproceedings}",  "#body");
  DefEnvironment!("{bphdthesis}",    "#body");
  DefEnvironment!("{bmastersthesis}", "#body");
  DefEnvironment!("{btechreport}",   "#body");

  // IMS bibliography field-tagging macros — imsart.sty defines these
  // as NLM/JATS-style 1-arg setters inside `{barticle}` etc. envs:
  // \bauthor{name}, \binits{initials}, \bfnm{first}, \bsnm{surname},
  // \byear{2024}, \bvolume{42}, \bissue{3}, \bpages{1-20},
  // \bjournal{Annals}, \bpublisher{Springer}, \bseries{Lecture Notes},
  // \btitle{...}, \bmrnumber{...}, etc. Raw imsart.sty defines them,
  // but its preamble has complex catcode/group state that sometimes
  // fails mid-load, leaving these undefined. Provide content-
  // preserving stubs that emit args inline so the substantive
  // bibliography text survives. Witness 2305.13037, 2306.02821.
  DefMacro!("\\bauthor{}", "#1");
  DefMacro!("\\binits{}", "#1");
  DefMacro!("\\bfnm{}", "#1");
  DefMacro!("\\bsnm{}", "#1");
  DefMacro!("\\byear{}", "#1");
  DefMacro!("\\bvolume{}", "#1");
  DefMacro!("\\bissue{}", "#1");
  DefMacro!("\\bpages{}", "#1");
  DefMacro!("\\bjournal{}", "#1");
  DefMacro!("\\bpublisher{}", "#1");
  DefMacro!("\\bseries{}", "#1");
  DefMacro!("\\btitle{}", "#1");
  DefMacro!("\\bmrnumber{}", "#1");
  DefMacro!("\\bedition{}", "#1");
  DefMacro!("\\beditor{}", "#1");
  DefMacro!("\\beditortype{}", "#1");
  DefMacro!("\\baddress{}", "#1");
  DefMacro!("\\borganization{}", "#1");
  DefMacro!("\\bcollaboration{}", "#1");
  DefMacro!("\\bdoi{}", "#1");
  DefMacro!("\\burl{}", "#1");
  DefMacro!("\\bothertype{}", "#1");
});

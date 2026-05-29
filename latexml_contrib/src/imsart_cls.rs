//! Stub for IMS (Institute of Mathematical Statistics) `imsart` class.
//!
//! imsart.cls loads `article` + requires `imsart.sty` (support file with
//! \startlocaldefs, \endlocaldefs, etc.). We fall back to OmniBus and raw-
//! load imsart.sty so most user macros become available.
use latexml_package::prelude::*;


LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  // NOTE: do NOT eagerly `RequirePackage!("amsthm")` here. OmniBus already
  // provides lazy amsthm autoload (theorem-env stubs), and pre-loading it
  // broke the common `\let\proof\relax` + `\usepackage{amsthm}` idiom: the
  // paper's explicit \usepackage{amsthm} would no-op (already loaded), so
  // amsthm's `\let\proof\@proof` never re-ran after the paper cleared
  // `\proof` → `Error:undefined:{proof}`. Letting the paper's
  // \usepackage{amsthm} be the first real load matches Perl (clean).
  // Witness 1612.03054 (`\let\proof\relax` L5 + amsthm L22).
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
  def_macro_identity("\\bauthor{}")?;
  def_macro_identity("\\binits{}")?;
  def_macro_identity("\\bfnm{}")?;
  def_macro_identity("\\bsnm{}")?;
  def_macro_identity("\\byear{}")?;
  def_macro_identity("\\bvolume{}")?;
  def_macro_identity("\\bissue{}")?;
  def_macro_identity("\\bpages{}")?;
  def_macro_identity("\\bjournal{}")?;
  def_macro_identity("\\bpublisher{}")?;
  def_macro_identity("\\bseries{}")?;
  def_macro_identity("\\btitle{}")?;
  def_macro_identity("\\bmrnumber{}")?;
  def_macro_identity("\\bedition{}")?;
  def_macro_identity("\\beditor{}")?;
  def_macro_identity("\\beditortype{}")?;
  def_macro_identity("\\baddress{}")?;
  def_macro_identity("\\borganization{}")?;
  def_macro_identity("\\bcollaboration{}")?;
  def_macro_identity("\\bdoi{}")?;
  def_macro_identity("\\burl{}")?;
  def_macro_identity("\\bothertype{}")?;
  def_macro_identity("\\bparticle{}")?;
  def_macro_identity("\\bnote{}")?;
  def_macro_identity("\\btype{}")?;
  // imsart.sty `\common@pub@types` also `\let`s these to `\@firstofone`
  // (identity), but they were missing from the list above — so an imsart
  // `.bbl` using `\begin{barticle}…\betal{…}` (bold-"et al." separator) or
  // `\banumber{…}` saw them undefined. Witness 1912.11583 (`\betal`, 1 error
  // → 0). Mirror `\common@pub@types`.
  def_macro_identity("\\betal{}")?;
  def_macro_identity("\\banumber{}")?;
  // Additional imsart bibliography field macros. The bundled imsart.cls/sty
  // `\let`s each of these to `\@firstofone` (identity) inside its bib setup
  // (or applies a style via `\set@bibl@cmd`, e.g. `\bbooktitle` → \itshape —
  // we keep content-preserving identity, matching the sibling stubs above).
  // They were missing, so an imsart `.bbl` using `\bbooktitle{…}` (book title
  // in an `In …` reference), `\bchapter`, `\bschool` (theses), etc. saw them
  // undefined. Witness 2006.02044 (`\bbooktitle`, 1 error → 0; Perl errors on
  // ALL 28 imsart `\b*`/`{b*}` constructs, so this also surpasses Perl). NB:
  // `\bmisc` is intentionally NOT added as a macro — it would clobber the
  // `{bmisc}` environment defined above.
  def_macro_identity("\\bbooktitle{}")?;
  def_macro_identity("\\bchapter{}")?;
  def_macro_identity("\\bhowpublished{}")?;
  def_macro_identity("\\binstitution{}")?;
  def_macro_identity("\\bisbn{}")?;
  def_macro_identity("\\blocation{}")?;
  def_macro_identity("\\bnumber{}")?;
  def_macro_identity("\\bschool{}")?;
  def_macro_identity("\\bsuffix{}")?;
});

//! Stub for IMS (Institute of Mathematical Statistics) `imsart` class.
//!
//! imsart.cls loads `article` + requires `imsart.sty` (support file with
//! \startlocaldefs, \endlocaldefs, etc.). We fall back to OmniBus and raw-
//! load imsart.sty so most user macros become available.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // imsart.cls:84-118: the aap/aop/aos/aoas/sts/aihp journal options pass
  // `leqno` to amsmath — those journals number equations on the left
  // (DIVERGENCES #336; 2605.16086 `[preprint,aap]`).
  let journal_leqno = lookup_vecdeque("opt@imsart.cls")
    .map(|v| {
      v.iter()
        .any(|o| ["aap", "aop", "aos", "aoas", "sts", "aihp"].contains(&o.to_string().as_str()))
    })
    .unwrap_or(false);
  if journal_leqno {
    RequirePackage!("amsmath", options => vec!["leqno".to_string()]);
  } else {
    RequirePackage!("amsmath");
  }
  // NOTE: do NOT eagerly `RequirePackage!("amsthm")` here. OmniBus already
  // provides lazy amsthm autoload (theorem-env stubs), and pre-loading it
  // broke the common `\let\proof\relax` + `\usepackage{amsthm}` idiom: the
  // paper's explicit \usepackage{amsthm} would no-op (already loaded), so
  // amsthm's `\let\proof\@proof` never re-ran after the paper cleared
  // `\proof` → `Error:undefined:{proof}`. Letting the paper's
  // \usepackage{amsthm} be the first real load matches Perl (clean).
  // Witness 1612.03054 (`\let\proof\relax` L5 + amsthm L22).
  // imsart.sty's frontmatter, defined here so its raw definitions are refused. `\ead[label=e1]{addr}`
  // (imsart.sty:944 `\DeclareRobustCommand\ead[2][label= ,email]`) is the author's email, or URL with the `url` key
  // or a `://` address (`\checkead@prefix`, :955); OmniBus's `\ead{}[]` (elsarticle's order) read `[` as the address
  // and left `label=e1]{…}` in the author's name (2201.11773, 2507.12447). `\printead[presep=…]{e1,u1}` (:965)
  // reprints, in an address, the addresses `\ead` gave the authors, each already their contact. `\author[A,B]{…}`
  // and `\address[A]{…}` (:2003, :2088) link an author to the addresses its marks label, as revtex's and elsarticle's
  // do (revtex4_support_sty.rs, elsart_support_core_sty.rs).
  DefMacro!("\\ead[] Semiverbatim", sub[(keys, address)] {
    let url = keys.as_ref().is_some_and(|keys| {
      keys.to_string().split(',').any(|key| key.split('=').next().is_some_and(|k| k.trim() == "url"))
    }) || address.to_string().contains("://");
    Ok(if url {
      Invocation!(T_CS!("\\lx@add@url"), vec![None, Some(address)])
    } else {
      Invocation!(T_CS!("\\lx@add@email"), vec![None, Some(address)])
    })
  }, locked => true);
  DefMacro!("\\printead OptionalMatch:* []{}", "", locked => true);
  // (a name list in one `\author{A and B}` is an author each, every one with the marks: 2401.00578, 2505.07640)
  DefMacro!(
    "\\author OptionalSemiverbatim {}",
    "\\lx@add@authors@append[annotations={#1}]{#2}",
    locked => true
  );
  DefMacro!(
    "\\address OptionalSemiverbatim {}",
    "\\lx@add@contact[label={#1},role=address]{#2}",
    locked => true
  );
  // `\thanksref[mark]{addr1,t1}` (imsart.sty:746) marks the author with the addresses and notes those labels name, and
  // `\thankstext{t1}{text}` (:789) is such a note: as elsarticle's `\thanksref`/`\thanks` (elsart_support_core_sty.rs).
  // Read raw, the note went to `\@thanks`, which nothing typesets, and its text was lost (2201.08502).
  // The `thanks:` prefix on both sides, as elsarticle's, matches a title's `\thanksref{T1}` to its note exactly (the
  // prefix-stripped match serves creators only, OXIDIZED_DESIGN_DIVERGENCES #461), and an author's to an `\address`.
  DefMacro!(
    "\\thanksref[]{}",
    "\\lx@request@frontmatter@annotation[thanks]{#2}",
    locked => true
  );
  DefMacro!(
    "\\thankstext[]{}{}",
    "\\lx@add@contact[label={thanks:#2},role=thanks]{#3}",
    locked => true
  );
  // imsart.cls L149: \RequirePackage{imsart}.
  InputDefinitions!("imsart", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // `\arxiv{id}` (imsart.sty:1186-1189) prints the paper's arXiv identifier as an unmarked title note: the paper's
  // arXiv pubnote, as jheppub's `\arxivnumber` (jheppub_sty.rs), not an author's note (2201.08502). Defined over the
  // raw one and not locked: a paper citing with its own `\renewcommand\arxiv` keeps it.
  DefMacro!(
    "\\arxiv{}",
    "\\gdef\\@arxiv{#1}\\lx@add@pubnote[role=arxiv]{#1}"
  );

  // imsart.sty (L3015) redefines `\bibliography#1` to only
  // `\@input@{\jobname.bbl}` — it inputs a pre-built `.bbl` and NEVER reads the
  // `.bib`. arXiv imsart submissions routinely ship the `.bib` alone (no
  // compiled `<jobname>.bbl`), so that raw definition silently drops the whole
  // bibliography (0 bibitems, no diagnostic). Restore the core `\bibliography`
  // (`latex_constructs.rs`): `\lx@ifusebbl` inputs `<jobname>.bbl` when it
  // exists (unchanged for `.bbl` papers) and otherwise reads the `.bib` directly
  // via `\lx@bibliography`, which is our beyond-Perl `.bib` path. Witnesses
  // 2606.00231 (`reference.bib`, 0→N) and 2606.17491 (two `.bib` files, 0→N).
  DefMacro!(
    "\\bibliography Semiverbatim",
    r#"\lx@ifusebbl{#1}{\input{\jobname.bbl}}{\lx@bibliography{#1}}"#
  );

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
  // IMS authors use \orcid for the ORCID identifier. Route it through the kernel
  // `\lx@add@orcid`, producing an `ltx:contact[role=orcid]` the XSLT renders as a
  // clickable https://orcid.org/<id> link (vs a bare dagger note). html_feedback#6571.
  DefMacro!("\\orcid{}", "\\lx@add@orcid{#1}");
  // IMS journal bibliography entry types — imsart.sty defines these as
  // \def commands but they're used as environments in some .bbl files.
  // Provide as no-op envs (the actual bibliography rendering is handled
  // by biblatex/natbib elsewhere). Witness 2406.15844 (+4 imsart papers).
  DefEnvironment!("{barticle}", "#body");
  DefEnvironment!("{bbook}", "#body");
  DefEnvironment!("{bbooklet}", "#body");
  DefEnvironment!("{binbook}", "#body");
  DefEnvironment!("{bincollection}", "#body");
  DefEnvironment!("{bunpublished}", "#body");
  DefEnvironment!("{bmisc}", "#body");
  DefEnvironment!("{bproceedings}", "#body");
  DefEnvironment!("{bphdthesis}", "#body");
  DefEnvironment!("{bmastersthesis}", "#body");
  DefEnvironment!("{btechreport}", "#body");

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

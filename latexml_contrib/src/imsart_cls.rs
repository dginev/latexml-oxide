//! Stub for IMS (Institute of Mathematical Statistics) `imsart` class.
//!
//! imsart.cls loads `article` + requires `imsart.sty` (support file with
//! \startlocaldefs, \endlocaldefs, etc.). We fall back to OmniBus and raw-
//! load imsart.sty so most user macros become available.
use latexml_package::{package::ims_support_sty::vtex_keyword_markup, prelude::*};

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
  // imsart.cls loads no amsmath itself (244 copies in a 1/20 arXiv sample): amsmath's commands for papers that use them
  // unloaded, its "Old form" error once the paper loads it; the paper's copy decides
  // (`amsmath_sty::require_amsmath_beyond_original`, KNOWN_PERL_ERRORS #993).
  amsmath_sty::require_amsmath_beyond_original(
    latexml_package::shipped_class::shipped_copy_loads("cls", "amsmath")?,
    if journal_leqno { &["leqno"] } else { &[] },
  )?;
  // NOTE: do NOT eagerly `RequirePackage!("amsthm")` here. OmniBus already
  // provides lazy amsthm autoload (theorem-env stubs), and pre-loading it
  // broke the common `\let\proof\relax` + `\usepackage{amsthm}` idiom: the
  // paper's explicit \usepackage{amsthm} would no-op (already loaded), so
  // amsthm's `\let\proof\@proof` never re-ran after the paper cleared
  // `\proof` → `Error:undefined:{proof}`. Letting the paper's
  // \usepackage{amsthm} be the first real load matches Perl (clean).
  // Witness 1612.03054 (`\let\proof\relax` L5 + amsthm L22).
  imsart_sty_markup()?;
  for cs in KEYWORD_MARKUP {
    AssignValue!(&s!("{cs}:locked") => true, Some(Scope::Global));
  }
  // imsart.cls L149: \RequirePackage{imsart}.
  InputDefinitions!("imsart", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  DefPrimitive!("\\lx@ims@keyword@unlock", {
    for cs in KEYWORD_MARKUP {
      AssignValue!(&s!("{cs}:locked") => false, Some(Scope::Global));
    }
  });
  RawTeX!(r"\AtEndOfClass{\lx@ims@keyword@unlock}");
  imsart_arxiv()?;

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
  // Each reads an optional argument and drops it (imsart.sty 2012/08/16 :2585-2769, `\def\barticle{\@ifnextchar[
  // {\@barticle}{\@barticle[]}}` `\def\@barticle[#1]{…}`), which the `.bbl`s fill with the entry's key list,
  // `\begin{barticle}[author]`: read as text it printed "[author]" before every structured entry (0906.4329,
  // 1010.4313, 1202.6182). `{binproceedings}` and `{bmanual}` (:2633, :2706) were missing.
  DefEnvironment!("{barticle}[]", "#body");
  DefEnvironment!("{bbook}[]", "#body");
  DefEnvironment!("{bbooklet}[]", "#body");
  DefEnvironment!("{binbook}[]", "#body");
  DefEnvironment!("{bincollection}[]", "#body");
  DefEnvironment!("{binproceedings}[]", "#body");
  DefEnvironment!("{bmanual}[]", "#body");
  DefEnvironment!("{bunpublished}[]", "#body");
  DefEnvironment!("{bmisc}[]", "#body");
  DefEnvironment!("{bproceedings}[]", "#body");
  DefEnvironment!("{bphdthesis}[]", "#body");
  DefEnvironment!("{bmastersthesis}[]", "#body");
  DefEnvironment!("{btechreport}[]", "#body");
  // imsart.sty :2514 `\let\endbibitem\relax`: the `.bbl`s close every `\bibitem` with `\endbibitem`
  // (0906.4329, 1010.4313, 1202.6182).
  Let!("\\endbibitem", "\\relax");

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
  // Except `\bdoi`, which imsart.sty gobbles "for compatibility with old bibtex style" (:2583 `\def\bdoi#1{\ignorespaces}`,
  // :4151 in later copies): printed, its DOI's `_` raised "Script _ can only appear in math mode" (1405.0671,
  // 1511.07613, 2110.07639; OXIDIZED_DESIGN #551). Guard `perfect_kernel_batch65m::imsart_bdoi_is_gobbled`.
  DefMacro!("\\bdoi{}", "\\ignorespaces");
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

/// imsart.sty's commands the binding owns: its frontmatter and its keyword blocks, defined before the class reads
/// imsart.sty raw (so its raw definitions are refused) and by the imsart.sty binding (`imsart_sty.rs`), which a shipped
/// copy declaring itself imsart.sty reaches (OXIDIZED_DESIGN_DIVERGENCES #986).
#[rustfmt::skip]
pub fn imsart_sty_markup() -> Result<()> {
  // imsart.sty's frontmatter, defined before its raw definitions so they are refused. `\ead[label=e1]{addr}`
  // (imsart.sty:944 `\DeclareRobustCommand\ead[2][label= ,email]`) is the author's email, or URL with the `url` key
  // or a `://` address (`\checkead@prefix`, :955); OmniBus's `\ead{}[]` (elsarticle's order) read `[` as the address
  // and left `label=e1]{…}` in the author's name (2201.11773, 2507.12447). `\printead[presep=…]{e1,u1}` (:965)
  // reprints, in an address, the addresses `\ead` gave the authors, each already their contact. `\author[A,B]{…}`
  // and `\address[A]{…}` (:2003, :2088) link an author to the addresses its marks label, as revtex's and elsarticle's
  // do (revtex4_support_sty.rs, elsart_support_core_sty.rs). The address prints in typewriter type (imsart.sty:1162
  // `\setattribute{ead}{size}{\upshape\ttfamily}`; 1601.00202's `\ead[url]`).
  DefMacro!("\\ead[] Semiverbatim", sub[(keys, address)] {
    // (stored under its label for a `\printead{e1}` in an address printed in place, below)
    let url = record_ead(keys.as_ref(), &address);
    let address = Invocation!(T_CS!("\\lx@address@urlfont"), vec![Some(address)]);
    Ok(if url {
      Invocation!(T_CS!("\\lx@add@url"), vec![None, Some(address)])
    } else {
      Invocation!(T_CS!("\\lx@add@email"), vec![None, Some(address)])
    })
  }, locked => true);
  // In an address printed in place (below), no author's contact, `\printead{e1,u1}` prints what imsart.sty's
  // `\@@printead` (:506-519 of 1301.1466's copy) prints: each labelled address `\ead` stored, "e-mail: " or "url: " before
  // it (`\email@text`, `\url@text`; none under `*`), `;~` between, in typewriter type; `\printead[text]{…}` the text alone
  // (review revCR3b finding 1: 1301.1466's end blocks, `\printead{e1}\\ \printead{u1}`). A label no `\ead` gave prints
  // "??", as `\saferef`. Inside the frontmatter it prints nothing: the address it stands in is the author's contact.
  DefMacro!("\\printead OptionalMatch:* []{}", sub[(star, text, labels)] {
    if !lookup_bool("lx_imsart_address_inplace") {
      return Ok(Tokens!());
    }
    if let Some(text) = text {
      return Ok(Invocation!(T_CS!("\\lx@address@urlfont"), vec![Some(text)]));
    }
    let mut printed: Vec<Token> = Vec::new();
    for (at, label) in labels.to_string().split(',').map(str::trim).filter(|l| !l.is_empty()).enumerate() {
      if at > 0 {
        printed.extend(mouth::tokenize_internal(";~").unlist());
      }
      let kind = match lookup_value(&format!("lx_imsart_ead_{label}")) {
        Some(Stored::Tokens(kind)) => kind.to_string(),
        _ => String::from("email"),
      };
      if star.is_none() {
        printed.extend(mouth::tokenize_internal(if kind == "url" { "url:" } else { "e-mail:" }).unlist());
        printed.push(T_SPACE!());
      }
      let address = match lookup_value(&format!("lx_imsart_ead_{kind}_{label}")) {
        Some(Stored::Tokens(address)) => Invocation!(T_CS!("\\lx@address@printed"), vec![Some(address)]),
        _ => mouth::tokenize_internal("??"),
      };
      printed.extend(Invocation!(T_CS!("\\lx@address@urlfont"), vec![Some(address)]).unlist());
    }
    Ok(Tokens::new(printed))
  }, locked => true);
  // (a name list in one `\author{A and B}` is an author each, every one with the marks: 2401.00578, 2505.07640)
  // (each `\ead` in it recorded at the call, as imsart.sty's `\safelabel` writes the aux file that a `\printead` reads on
  // the next run, wherever it stands: an address printed in place before the frontmatter's flush digests the author)
  DefMacro!("\\author OptionalSemiverbatim {}", sub[(annotations, name)] {
    record_eads(&name);
    Ok(Invocation!(T_CS!("\\lx@add@authors@append"), vec![
      Some(Tokens::new(
        [mouth::tokenize_internal("annotations=").unlist(), vec![T_BEGIN!()],
         annotations.map(Tokens::unlist).unwrap_or_default(), vec![T_END!()]].concat()
      )),
      Some(name)
    ]))
  }, locked => true);
  // An unlabelled `\address` after `\end{frontmatter}` is no author's: under the journal options that set
  // `\@addressatendtrue` (`aap`, imsart.sty:150 of 1301.1466's copy) `\address` stores each block (:1242-1252) and the
  // end of the document prints them all (`\printaddresses`, :1269-1290), each tied to no author; the API's default
  // owner, the last author, would take every one (1301.1466's three after its `\bibliography`). It is set as
  // `\printaddressnum` sets a block (`\address@size`, `{tabular}[t]{@{}l@{}}`, :1255-1264), at the end of the document under
  // `\@addressatendtrue` (`\AtEndDocument`, as imsart.sty's `\enddocument` runs `\printaddresses`; review revCR3b NIT 4),
  // else where it is called. Inside `{frontmatter}`, or labelled (`\address[A]`, the authors marked A), it is a contact;
  // unlabelled, of every author since the last `\address` (`annotate=run`, as the sty binding's `\affiliation`):
  // imsart.sty:1021 resets `\author@fmt@init` there, so `\author{A}\and\author{B}\address{Univ AB}` prints the address
  // under both (review revCR3b finding 3).
  RawTeX!(
    r"\AddToHook{env/frontmatter/after}{\lx@imsart@frontmatter@closed}
\def\lx@imsart@address@contact#1#2{\lx@add@contact[label={#1},role=address]{#2}}
\def\lx@imsart@address@run#1#2{\lx@add@contact[label={#1},role=address,annotate=run]{#2}}
\def\lx@imsart@address@inplace#1#2{\@ifundefined{if@addressatend}{\lx@imsart@address@print{#2}}{\csname if@addressatend\endcsname\expandafter\g@addto@macro\expandafter\lx@imsart@addresses@end\else\expandafter\@firstofone\fi{\lx@imsart@address@print{#2}}}}
\def\lx@imsart@address@print#1{\par\begingroup\lx@imsart@address@inplace@on\csname address@size\endcsname\noindent\begin{tabular}[t]{@{}l@{}}#1\end{tabular}\par\endgroup}
\def\lx@imsart@addresses@end{}
\AtEndDocument{\lx@imsart@addresses@end}"
  );
  DefPrimitive!("\\lx@imsart@address@inplace@on", sub[_args] {
    assign_value("lx_imsart_address_inplace", true, Some(Scope::Local));
    Ok(())
  });
  DefPrimitive!("\\lx@imsart@frontmatter@closed", sub[_args] {
    assign_value("lx_imsart_frontmatter_closed", true, Some(Scope::Global));
    Ok(())
  });
  DefMacro!("\\address OptionalSemiverbatim {}", sub[(label, address)] {
    let label = label.unwrap_or_default();
    Ok(if label.is_empty() && lookup_bool("lx_imsart_frontmatter_closed") {
      Invocation!(T_CS!("\\lx@imsart@address@inplace"), vec![Some(label), Some(address)])
    } else if label.is_empty() {
      Invocation!(T_CS!("\\lx@imsart@address@run"), vec![Some(label), Some(address)])
    } else {
      Invocation!(T_CS!("\\lx@imsart@address@contact"), vec![Some(label), Some(address)])
    })
  }, locked => true);
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
  // `\thanksmark[mark]{m1}` (imsart.sty:640-645) prints, in an `\affiliation`, the mark the authors' `\thanksref{m1}`
  // point at; the authors' marks are not printed here (the `\thanksref` annotation links them), so the address's
  // mark is dropped too (1202.6182, 1306.1438). Locked as `\thanksref`/`\thankstext` are, so a paper that ships
  // imsart.sty does not get its `\thanksmark`, which prints `<sup>??</sup>` here.
  DefMacro!("\\thanksmark[]{}", "", locked => true);
  // imsart.sty's author markup, defined only inside its `\author@fmt` (:2010-2028 of 2201.11773's copy: `\degs`,
  // `\fnms`, `\inits`, `\snm`, `\roles`, each printing its argument), which the `\author` above replaces: in force for
  // the author content (`\lx@author@markup@class`, frontmatter_fields.rs), as a refused class `\author` gives them —
  // and where no lock records that refusal, a stand-in copy's (`imsart_sty.rs`): "\snm undefined" there (1905.11425,
  // 2101.10506, 1301.1466, math0608476).
  RawTeX!(
    r"\def\lx@author@markup@class{\lx@author@markup@provide\degs\@firstofone\lx@author@markup@provide\fnms\@firstofone\lx@author@markup@provide\inits\@firstofone\lx@author@markup@provide\snm\@firstofone\lx@author@markup@provide\roles\@firstofone}"
  );
  // imsart.sty:464-466: `\setattribute[…]{journal}{name}{…}` stores `\journal@name` (`\getattribute` reads it back);
  // papers blank the journal name with it (1306.0113, 1706.08519).
  RawTeX!(
    r"\def\setattribute{\@ifnextchar[\@setattribute{\@setattribute[]}}
\def\@setattribute[#1]#2#3#4{\expandafter\gdef\csname #2@#3\endcsname{#4}}
\def\getattribute#1#2{\csname #1@#2\endcsname}"
  );
  // imsart.sty's `{keyword}[class=…]` and `\kwd` (:1518-1567 of 1410.4462's copy) are the VTeX classes' shared markup
  // (`vtex_keyword_markup`, ims_support_sty.rs): every `\kwd` of a block in one list, and a `class=MSC` (`AMS`,
  // `MSC2010`, `MSC2020`) block the paper's subject classification. Locked, so the raw definitions are refused: imsart.sty
  // sets its blocks in `\keyword@box` (or, under the journal options' `keywordsasfootnote`, collects them in
  // `\keyword@toks` through its `\pseudo@keyword`, :1609, which `\begin` reads first), and OmniBus's `\kwd` kept only
  // the last keyword (KNOWN_PERL_ERRORS #1196): 1410.4462, 2407.13970, 2605.27137 lost every keyword but one and their
  // MSC codes. The headings are imsart.sty's (:1521-1525), which its own `\setattribute`/`\setpkgattr` lines set again.
  // Locked only while the class loads — imsart.sty read raw, and a renamed copy of the class read after this binding
  // with the imsart.sty copy it loads (1401.3408's myimsart.cls, myimsart.sty) — and unlocked at the class's end, as
  // `\arxiv` below is defined over the raw one: a document's own `\renewcommand{\kwd}` is the document's.
  RequirePackage!("keyval");
  vtex_keyword_markup()?;
  RawTeX!(
    r"\@namedef{keyword@KWD}{Keywords and phrases:}
\@namedef{keyword@AMS}{AMS 2000 subject classifications:}
\@namedef{keyword@MSC}{MSC 2010 subject classifications:}
\@namedef{keyword@MSC2010}{MSC 2010 subject classifications:}
\@namedef{keyword@MSC2020}{MSC2020 subject classifications:}"
  );
  Ok(())
}

/// Records imsart.sty's `\ead[keys]{address}` under its label, as its `\safelabel` writes the aux file (`\ead` :493-498 of
/// 1301.1466's copy), for `\printead{label}` in an address printed in place ([`imsart_sty_markup`]): the address and
/// whether it is a URL (the `url` key, or a `://` address, `\checkead@prefix`), which it returns. Past any speculative
/// run's rollback (`assign_value_past_rollback`), as TeX never takes back what it wrote to a file.
fn record_ead(keys: Option<&Tokens>, address: &Tokens) -> bool {
  let keys = keys.map(ToString::to_string).unwrap_or_default();
  let url = keys
    .split(',')
    .any(|key| key.split('=').next().is_some_and(|k| k.trim() == "url"))
    || address.to_string().contains("://");
  let label = keys.split(',').find_map(|key| {
    let (k, v) = key.split_once('=')?;
    (k.trim() == "label" && !v.trim().is_empty()).then(|| v.trim().to_string())
  });
  if let Some(label) = label {
    let kind = if url { "url" } else { "email" };
    assign_value_past_rollback(
      &format!("lx_imsart_ead_{kind}_{label}"),
      Stored::Tokens(address.clone()),
    );
    assign_value_past_rollback(
      &format!("lx_imsart_ead_{label}"),
      Stored::Tokens(mouth::tokenize_internal(kind)),
    );
  }
  url
}

/// Records each `\ead[keys]{address}` written in an `\author`'s argument ([`record_ead`]), at the call.
fn record_eads(name: &Tokens) {
  let v = name.unlist_ref();
  // (the group opened at `v[open]` by `open_t` and closed by `close_t` outside braces: its inner tokens and the index
  // after it)
  let group = |open: usize, open_t: Token, close_t: Token| -> Option<(Tokens, usize)> {
    if v.get(open) != Some(&open_t) {
      return None;
    }
    let mut braces = usize::from(open_t == T_BEGIN!());
    for (at, t) in v.iter().enumerate().skip(open + 1) {
      if *t == T_BEGIN!() {
        braces += 1;
      } else if *t == T_END!() {
        braces = braces.checked_sub(1)?;
      }
      if *t == close_t && braces == 0 {
        return Some((Tokens::new(v[open + 1..at].to_vec()), at + 1));
      }
    }
    None
  };
  let mut at = 0;
  while at < v.len() {
    if v[at] == T_CS!("\\ead") {
      let mut next = at + 1;
      while v.get(next) == Some(&T_SPACE!()) {
        next += 1;
      }
      let keys = group(next, T_OTHER!("["), T_OTHER!("]"));
      if let Some((_, after)) = &keys {
        next = *after;
      }
      while v.get(next) == Some(&T_SPACE!()) {
        next += 1;
      }
      if let Some((address, after)) = group(next, T_BEGIN!(), T_END!()) {
        record_ead(keys.as_ref().map(|(keys, _)| keys), &address);
        at = after;
        continue;
      }
    }
    at += 1;
  }
}

/// imsart.sty's keyword markup ([`imsart_sty_markup`]), locked while the class loads.
pub const KEYWORD_MARKUP: [&str; 7] = [
  "\\keyword",
  "\\endkeyword",
  "\\begin{keyword}",
  "\\end{keyword}",
  "\\kwd",
  "\\@kwd",
  "\\pseudo@keyword",
];

/// imsart.sty's `\arxiv`, defined over the raw one ([`imsart_sty_markup`]'s note).
pub fn imsart_arxiv() -> Result<()> {
  // `\arxiv{id}` (imsart.sty:1186-1189) prints the paper's arXiv identifier as an unmarked title note: the paper's
  // arXiv pubnote, as jheppub's `\arxivnumber` (jheppub_sty.rs), not an author's note (2201.08502). Defined over the
  // raw one and not locked: a paper citing with its own `\renewcommand\arxiv` keeps it.
  DefMacro!(
    "\\arxiv{}",
    "\\gdef\\@arxiv{#1}\\lx@add@pubnote[role=arxiv]{#1}"
  );

  Ok(())
}

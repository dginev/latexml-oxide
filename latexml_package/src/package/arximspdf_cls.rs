//! arximspdf.cls / arxstspdf.cls — the arXiv IMS journal classes (Annals of
//! Probability/Statistics/Applied Probability, `aop`/`aos`/`aap`/`aoas`).
//!
//! Neither Perl LaTeXML nor (previously) Rust bound these self-contained ~3000-
//! line classes, so `\documentclass{arximspdf}` papers cascaded into dozens of
//! undefined-macro errors (the structured `\b*` bibliography, `{barticle}`/…,
//! plus `\eqntext`/`\dvtx`/`{longlist}`). This binding loads `article` as the
//! base and defines the IMS-specific macros semantically so the papers convert —
//! a strict improvement over Perl, which fails outright (both engines lack the
//! class; same-host confirmed). Witnesses: 0910.0069 (aop632) + the 16-paper
//! aop/aos sandbox cluster.
//!
//! Bibliography note: arximspdf uses `natbib` + `\bibitem`/`thebibliography`, so
//! a bibitem produces `ltx:bibitem > ltx:bibblock` (Flow.model). The structured
//! `ltx:bib-*` vocabulary is schema-valid only inside `ltx:bibentry` (the BibTeX
//! path), NOT inside `bibblock` — so the `\b*` field macros are PASSTHROUGH text
//! (readable + schema-valid), not mapped to `ltx:bib-*`.
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};

use crate::{
  prelude::*,
  shipped_class::{class_code_contains, shipped_class_text},
};

#[rustfmt::skip]
LoadDefinitions!({
  // ---- options -------------------------------------------------------------
  // Journal selectors + numbering/layout options: all ignorable for a semantic
  // conversion; pass anything unrecognized through to article.
  for opt in [
    "aop","aos","aap","aoas",
    "10pt","11pt","12pt","draft","final","openright","openany",
    "onecolumn","twocolumn","leqno","fleqn","twoside","oneside",
    "number","nameyear","noMRlinks","MSNbibl","citesort","dvips","pdftex",
    "autosecdot","noautosecdot","xxtheorem","nohyperref",
    "normalfloat","secfloat","chapfloat",
    "normaleqn","seceqn","chapeqn","normalthm","secthm","chapthm",
  ] { DeclareOption!(opt, None); }
  // arximspdf.cls:164 `\DeclareOption{rotating}{\AtEndOfClass{\RequirePackage[…]{rotating}}}`, the floats
  // `\sidewaystable`/`\sidewaysfigure` the class lets at `\begin{document}` (:1838-1848; 1304.4448, 1104.3398); the
  // rotation is print-only.
  DeclareOption!("rotating", "\\AtEndOfClass{\\RequirePackage{rotating}}");
  DeclareOption!(None, { Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?; });
  ProcessOptions!();

  LoadClass!("article", withoptions => true);

  RequirePackage!("amssymb");
  // NB: arximspdf does NOT load amsmath — it defines \tfrac/\dfrac/\operatorname/
  // {aligned} itself. We mirror that: load only amsopn (for \operatorname /
  // \DeclareMathOperator) and define the rest directly. Loading full amsmath would
  // override plain-TeX \matrix with its env-form, breaking the command-form
  // \matrix{...\cr...} these papers use (Perl is parity-broken the same way and
  // also omits amsmath).
  RequirePackage!("amsopn");
  RequirePackage!("bm");        // \boldsymbol
  RequirePackage!("natbib");    // \citep/\citet/\citeyear, thebibliography
  RequirePackage!("graphicx");
  RequirePackage!("hyperref");  // \href, \texorpdfstring (class loads it too)

  // ---- math the class defines itself (arximspdf L1099-1100) ----------------
  DefMacro!("\\tfrac{}{}", "{\\textstyle\\frac{#1}{#2}}");
  DefMacro!("\\dfrac{}{}", "{\\displaystyle\\frac{#1}{#2}}");
  DefMacro!("\\dvt",  "\\colon\\ ");
  DefMacro!("\\dvtx", "\\colon\\;");
  DefMacro!("\\divid{}{}", "\\frac{#1}{#2}");
  DefMacro!("\\zs{}",  "#1");
  DefMacro!("\\zss{}", "#1");
  DefMacro!("\\eqntext{}", "#1");
  // The structured `\b*` bibliography, `{pf}` proofs with `\qed`, `\tablewidth` table notes and `\bolds`: the
  // markup the VTeX IMS classes share (ims_support_sty.rs).
  RequirePackage!("ims_support");

  // ---- misc list / sectioning / theorems the class adds --------------------
  DefEnvironment!("{longlist}[]", "<ltx:enumerate>#body</ltx:enumerate>",
    mode => "internal_vertical");
  // arximspdf predefines a Theorem env (L1405), with its starred twin as every `\newtheorem` (vtexthm's, in
  // ims_support with `\newproclaim`; `{hy*}` 1107.3284); its proof envs are in ims_support.
  RawTeX!(r"\newtheorem{thm}{Theorem}");
  // ---- frontmatter (standard LaTeXML frontmatter API; metadata preserved) --
  // Standardize the IMS scaffolding onto the article frontmatter flow: the
  // {frontmatter}/{aug}/{keyword} blocks are TRANSPARENT wrappers (no special
  // mode), and the IMS metadata macros route to the same \lx@add@* sinks
  // article/acmart use, so creators/title/keywords/pubnotes are emitted into the
  // document frontmatter.
  // `{frontmatter}` a plain group, as OmniBus's (omnibus_cls.rs): a DefEnvironment boxes its body in restricted
  // horizontal mode, where a `$$` in the abstract is no display (TeX_Math.pool.ltxml:65).
  Let!("\\frontmatter", "\\@empty");
  Let!("\\endfrontmatter", "\\@empty");
  DefEnvironment!("{aug}", "#body");
  // `{keyword}[class=…]` and `\kwd` are the VTeX classes' shared markup (ims_support_sty.rs): arximspdf's entries
  // are joined by ", " (arximspdf.cls:2438 of 1107.3284's copy), under its headings "Key words and phrases." and "AMS
  // 2000 subject classifications." (:2408-2409; arxstspdf.cls:2356-2357 of 0903.0664's copy end them with ":"). The
  // collecting `\lx@begin@keywords` started afresh at each block, so a paper's AMS block was lost to the keywords that
  // follow it (1107.3284, 1309.4864).
  let shipped = match shipped_class_text("arximspdf")? {
    Some(text) => Some(text),
    None => shipped_class_text("arxstspdf")?,
  };
  let colon = shipped
    .as_deref()
    .is_some_and(|text| class_code_contains(text, "{KWD}{Key words and phrases:}"));
  RawTeX!(r"\def\sep{\unskip, }");
  if colon {
    RawTeX!(r"\setattribute{keyword}{KWD}{Key words and phrases:}\setattribute{keyword}{AMS}{AMS 2000 subject classifications:}");
  } else {
    RawTeX!(r"\setattribute{keyword}{KWD}{Key words and phrases.}\setattribute{keyword}{AMS}{AMS 2000 subject classifications.}");
  }
  // Author-name parts used inside \author{...}: pass their content through.
  for m in ["fnms","snm","inits","degs","roles","suffix"] {
    def_macro_identity(&format!("\\{m}{{}}"))?;
  }
  // \author[mark]{name} (IMS adds an optional thanks-mark) -> standard creator.
  DefMacro!("\\author[]{}", "\\lx@add@creator[role=author]{#2}");
  DefMacro!("\\affiliation{}", "\\lx@add@contact[role=affiliation]{#1}");
  DefMacro!("\\address[]{}", "\\lx@add@contact[role=address]{#2}");
  DefMacro!("\\email{}", "\\lx@add@contact[role=email,name={e-mail: }]{#1}");
  DefMacro!("\\ead[]{}", "\\lx@add@contact[role=email,name={e-mail: }]{#2}");
  DefMacro!("\\journal{}", "\\lx@add@pubnote[role=journal]{#1}");
  DefMacro!("\\volume{}", "\\lx@add@pubnote[role=volume]{#1}");
  DefMacro!("\\issue{}", "\\lx@add@pubnote[role=number]{#1}");
  DefMacro!("\\pubyear{}", "\\lx@add@date[role=publication]{#1}");
  DefMacro!("\\doi{}", "\\lx@add@pubnote[role=doi]{#1}");
  DefMacro!("\\received{}", "\\lx@add@date[role=received]{#1}");
  DefMacro!("\\revised{}", "\\lx@add@date[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\lx@add@date[role=accepted]{#1}");
  // History date parts + misc frontmatter: passthrough / drop.
  for m in ["sday","smonth","syear","sname","stitle","sdescription"] {
    def_macro_identity(&format!("\\{m}{{}}"))?;
  }
  DefMacro!("\\dochead{}", "");
  DefMacro!("\\dedicated{}", "");
  DefMacro!("\\runtitle{}", "");
  DefMacro!("\\runauthor{}", "");
  for m in ["corref{}","docsubty{}","thankstext[]{}{}","thanksref[]{}",
            "thanksmark[]{}","thankslabel[]{}","firstpage{}","lastpage{}",
            "referstodoi{}","relateddoi[]{}{}",
            "printaddress{}","printaddressnum{}","printaddresses",
            "pdftitle{}","pdfauthor{}","pdfsubject{}","pdfkeywords{}",
            "sdatatype{}","sfilename{}","slink[]{}","thesuppdoi{}"] {
    def_macro_noop(&format!("\\{m}"))?;
  }
  // `\printead*[text]{e1,e2}` prints the addresses `\ead[label=e1]` recorded (arximspdf.cls:599-603); `\ead`
  // already adds each as a contact, so the reprint goes. A `"printead*"` entry in the list above was the
  // prototype `\printead` + a literal `*`, which replaced the plain one: every `\printead{e1}` raised
  // "Missing argument Match" (30 aoas/aos/sts papers of run 329: 1107.4843, 1011.3351, 1205.6055).
  def_macro_noop("\\printead OptionalMatch:* []{}")?;
  DefEnvironment!("{supplement}[]", "#body");

  // ---- misc stubs ----------------------------------------------------------
  DefMacro!("\\startlocaldefs", "\\makeatletter");
  DefMacro!("\\endlocaldefs", "\\makeatother");
  for cs in ["HPROOF","PROOF","CRC","psdraft","psfull"] { def_macro_noop(&format!("\\{cs}"))?; }
  def_macro_noop("\\vtexed{}")?;

  // The URL, attribute and script helpers of the arximspdf.cls versions (`\doiurl`, `\arxivurl`, `\surl`,
  // `\setattribute`, `\tsup`, …) are the VTeX classes' shared markup (ims_support_sty.rs).
  // What both classes define and papers call (run 347: 42 arxstspdf and 13 arximspdf papers; line numbers from
  // 0808.2902's arxstspdf.cls and 0704.1778's arximspdf.cls), a copy without one leaving it to the paper
  // (`DefinitionOrigin::Fallback`):
  // - `\ccline{2-3,4-5}` (arxstspdf :1167-1169, arximspdf :1202-1204), a `\cline` for each range (33 papers: 1010.0304,
  //   1010.5040, 1011.1079);
  // - `\bhline` and its siblings `\thline`, `\fhline`, `\lhline` (arxstspdf :1828-1832), a table's rules spaced
  //   around, `\bhline`'s thicker: `\hline`s;
  // - `\freefootnotetext[mark]{text}` (arxstspdf :1917, arximspdf :1908), a footnote under the given mark, an empty one
  //   for an unmarked note (1011.1160 "This conversation was conducted in spring 2008.", 1009.1507's disclaimer);
  // - `\relateddois[label]{key}{text}` (1008.4373's arxstspdf.cls :2106-2115; 43 of the 68 copies, 99 of 371), a
  //   title note naming the articles related to this one, its `\relateddoi{label}{doi}` each a link to the DOI
  //   (1410.0163, 1301.7161, 1210.0385 "Discussed in …");
  // - `\shref{url}{text}` (arxstspdf :3021-3024), a link whose text breaks at its `\\`s;
  // - arximspdf's :2530-2531 `\write@toc@ignorecontentsline`/`\write@toc@restorecontentsline` (1205.5627) and
  //   :2585-2601 `\maxcsec{…}`, `\maxcsubsec`, `\maxcsubsubsec`, the contents' number widths (1103.4825, 1309.3728); its
  //   :533-540 `\setvaluelist`/`\getitemvalue` (1203.0565) are ims_support's, shared with arxbj.
  {
    let _class = OriginGuard::new(DefinitionOrigin::Fallback);
    DefMacro!("\\ccline{}", sub[(ranges)] {
      let ranges: Tokens = ranges;
      let mut out: Vec<Token> = Vec::new();
      for range in ranges.to_string().split(',') {
        let range = range.trim();
        if !range.is_empty() {
          out.extend(mouth::tokenize_internal(TeXString::assembled(s!("\\cline{{{range}}}"))).unlist());
        }
      }
      Ok(Tokens::new(out))
    });
    RawTeX!(r"\def\thline{\hline}\def\fhline{\hline}\def\bhline{\hline}\def\lhline{\hline}
\def\freefootnotetext[#1]{\footnotetext[#1]}
\@ifundefined{doi@base}{\def\doi@base{http://dx.doi.org/}}{}
\def\relateddois{\@ifnextchar[{\lx@ims@relateddois}{\lx@ims@relateddois[]}}
\def\lx@ims@relateddois[#1]#2#3{\lx@add@frontmatter{ltx:note}[role=relateddois]{\def\relateddoi##1##2{\href{\doi@base##2}{##2}}#3}}
\DeclareRobustCommand\shref[2]{\href{#1}{\let\\\relax#2}}
\def\write@toc@ignorecontentsline{\addtocontents{toc}{\protect\ignorecontentsline}}
\def\write@toc@restorecontentsline{\addtocontents{toc}{\protect\restorecontentsline}}
\def\maxcsec#1{}\def\maxcsubsec#1{}\def\maxcsubsubsec#1{}");
  }
});

/// arxstspdf.cls is arximspdf.cls (its `\doiurl`/`\arxivurl`, arxstspdf.cls:2902-2975, are the arximspdf copies' too;
/// 4 and 6 papers of run 329 had them undefined: 0903.0664).
pub fn load_arxstspdf_definitions() -> Result<()> { load_definitions() }

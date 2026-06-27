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
use crate::prelude::*;

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
    "normalfloat","secfloat","chapfloat","rotating",
    "normaleqn","seceqn","chapeqn","normalthm","secthm","chapthm",
  ] { DeclareOption!(opt, None); }
  DeclareOption!(None, { Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?; });
  ProcessOptions!();

  load_class_with_options("article", Tokens!())?;

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
  Let!("\\bolds", "\\boldsymbol");

  // ---- structured bibliography (passthrough) -------------------------------
  // Entry environments: transparent (swallow the optional [type] arg, pass body).
  for env in [
    "barticle","bbook","bincollection","binproceedings","binbook","bproceedings",
    "btechreport","bmanual","bmastersthesis","bphdthesis","bbooklet",
    "bunpublished","bmisc","bchapter",
  ] {
    def_macro_noop(&format!("\\{env}[]"))?;
    def_macro_noop(&format!("\\end{env}"))?;
  }
  // Field markup macros: emit their content as text.
  for m in [
    "bauthor","beditor","bsnm","bfnm","binits","bparticle","bsuffix",
    "btitle","bjournal","bbooktitle","bseries","bvolume","byear","bpages",
    "bedition","bpublisher","baddress","blocation","borganization","binstitution",
    "bschool","btype","bnumber","bchapter","bhowpublished","bnote","banumber",
    "bisbn","betal",
  ] {
    def_macro_identity(&format!("\\{m}{{}}"))?;
  }
  DefMacro!("\\AND", "and ");
  def_macro_noop("\\bptok{}")?;
  def_macro_noop("\\bptnote{}")?;
  def_macro_noop("\\bid{}")?;          // keyval mr=/doi=… — drop (matches class display)
  DefMacro!("\\bmrnumber{}", "\\MR{#1}");
  DefMacro!("\\MR{}", "MR#1");         // MathReviews id, as text
  def_macro_noop("\\endbibitem")?;

  // ---- misc list / sectioning / theorems the class adds --------------------
  DefEnvironment!("{longlist}[]", "<ltx:enumerate>#body</ltx:enumerate>",
    mode => "internal_vertical");
  DefMacro!("\\newproclaim", "\\newtheorem");
  // arximspdf predefines a Theorem env and proof env(s) (L1405/1430-1435).
  RawTeX!(r"\newtheorem{thm}{Theorem}");
  RawTeX!(r"\newenvironment{pf}{\begin{proof}}{\end{proof}}");
  RawTeX!(r"\newenvironment{pf*}[1]{\begin{proof}[#1]}{\end{proof}}");

  // ---- frontmatter (standard LaTeXML frontmatter API; metadata preserved) --
  // Standardize the IMS scaffolding onto the article frontmatter flow: the
  // {frontmatter}/{aug}/{keyword} blocks are TRANSPARENT wrappers (no special
  // mode), and the IMS metadata macros route to the same \lx@add@* sinks
  // article/acmart use, so creators/title/keywords/pubnotes are emitted into the
  // document frontmatter.
  DefEnvironment!("{frontmatter}", "#body");
  DefEnvironment!("{aug}", "#body");
  // {keyword} collects MANY \kwd into one keywords list: \lx@add@keywords alone
  // CLEARS on each call (keeps only the last), so use the collecting
  // \lx@begin@keywords/\lx@end@keywords form and let each \kwd emit its text.
  DefMacro!("\\keyword[]", "\\lx@begin@keywords");
  DefMacro!("\\endkeyword", "\\lx@end@keywords");
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
  DefMacro!("\\kwd[]{}", "#2, ");   // emit into the collecting {keyword} block
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
            "referstodoi{}","relateddoi[]{}{}","printead","printead*",
            "printaddress{}","printaddressnum{}","printaddresses",
            "pdftitle{}","pdfauthor{}","pdfsubject{}","pdfkeywords{}",
            "sdatatype{}","sfilename{}","slink[]{}","thesuppdoi{}"] {
    def_macro_noop(&format!("\\{m}"))?;
  }
  DefEnvironment!("{supplement}[]", "#body");

  // ---- misc stubs ----------------------------------------------------------
  DefMacro!("\\startlocaldefs", "\\makeatletter");
  DefMacro!("\\endlocaldefs", "\\makeatother");
  for cs in ["HPROOF","PROOF","CRC","psdraft","psfull"] { def_macro_noop(&format!("\\{cs}"))?; }
  def_macro_noop("\\vtexed{}")?;
});

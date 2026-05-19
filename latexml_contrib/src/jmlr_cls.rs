//! Stub for jmlr.cls and clear2025.cls family.
use latexml_package::prelude::*;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  RequirePackage!("amssymb");
  RequirePackage!("xcolor");
  RequirePackage!("hyperref");
  RequirePackage!("natbib");

  // Author-block primitives (jmlr.cls L335-342, L374-445).
  def_macro_noop("\\addr")?;
  DefMacro!("\\Name[]{}", "#2");
  // \Email{addr} — author email; preserve as ltx:creator/ltx:contact.
  DefMacro!("\\Email{}",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@email{email}{#1}}");
  DefConstructor!("\\@@@email{}{}", "^ <ltx:contact role='#1'>#2</ltx:contact>");
  // \IncludeName{firstname}{lastname} — author name parts in JMLR
  // bibliography. Preserve as ltx:note (rare in main paper body).
  DefMacro!("\\IncludeName{}{}",
    "\\@add@frontmatter{ltx:note}[role=name]{#1 #2}");
  DefMacro!("\\And", " ");
  // \acks{text} — JMLR Acknowledgments-and-Disclosure-of-Funding
  // section. Author body; emit as structural ltx:acknowledgements
  // (matches jmlr2e \acks treatment from commit 78bd49f1e2).
  DefConstructor!("\\acks{}",
    "<ltx:acknowledgements name='acknowledgments-disclosure-of-funding'>#1</ltx:acknowledgements>");
  DefMacro!("\\clearauthor{}", "\\author{#1}");

  // Frontmatter / pagination ceremony — preserve as ltx:note.
  DefMacro!("\\jmlrheading{}{}{}{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=heading]{#1 #2 #3 #4 #5 #6}");
  DefMacro!("\\jmlrvolume{}",
    "\\@add@frontmatter{ltx:note}[role=volume]{#1}");
  // JMLR frontmatter — preserve author-typed metadata as ltx:note so
  // it reaches the XML (content-preserving). Year/page/workshop/dates
  // are short scalars but the editor list is real prose authors care
  // about; gobbling drops attribution.
  DefMacro!("\\jmlryear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}");
  DefMacro!("\\jmlrworkshop{}",
    "\\@add@frontmatter{ltx:note}[role=workshop]{#1}");
  DefMacro!("\\jmlrsubmitted{}",
    "\\@add@frontmatter{ltx:note}[role=submitted]{#1}");
  DefMacro!("\\jmlrpublished{}",
    "\\@add@frontmatter{ltx:note}[role=published]{#1}");
  DefMacro!("\\jmlrproceedings{}{}",
    "\\@add@frontmatter{ltx:note}[role=proceedings]{#1: #2}");
  DefMacro!("\\editor{}",
    "\\@add@frontmatter{ltx:note}[role=editor]{#1}");
  DefMacro!("\\editors{}",
    "\\@add@frontmatter{ltx:note}[role=editors]{#1}");
  def_macro_noop("\\firstpageno{}")?;

  // {keywords} env.
  DefEnvironment!(
    "{keywords}",
    "<ltx:classification scheme='keywords'>#body</ltx:classification>"
  );

  // jmlrcombine helpers used in tables / floats.
  DefMacro!("\\floatconts{}{}{}", "#3");
  DefMacro!("\\tableref{}", "#1");
  DefMacro!("\\figureref{}", "#1");
  DefMacro!("\\algorithmref{}", "#1");
  // jmlrutils.sty L86-140: reference helpers. Stub as the LaTeX
  // \ref expansion so cross-refs still resolve. Witness 2409.07012.
  DefMacro!("\\sectionref{}", "\\ref{#1}");
  DefMacro!("\\appendixref{}", "\\ref{#1}");
  DefMacro!("\\equationref{}", "(\\ref{#1})");
  DefMacro!("\\theoremref{}", "\\ref{#1}");
  DefMacro!("\\lemmaref{}", "\\ref{#1}");
  DefMacro!("\\corollaryref{}", "\\ref{#1}");
  DefMacro!("\\propositionref{}", "\\ref{#1}");
  DefMacro!("\\definitionref{}", "\\ref{#1}");
  DefMacro!("\\exampleref{}", "\\ref{#1}");
  DefMacro!("\\remarkref{}", "\\ref{#1}");

  // jmlrutils theorem-style configuration helpers (gobble silently —
  // we don't replicate the punctuation/spacing). Witness: 2502.19625
  // (\theorempostheader{:}).
  def_macro_noop("\\theorempostheader{}")?;
  def_macro_noop("\\theoremheader{}")?;
  def_macro_noop("\\theoremsep{}")?;
  def_macro_noop("\\theoremprework{}")?;
  def_macro_noop("\\theorempostwork{}")?;
  def_macro_noop("\\theorembodyfont{}")?;
  def_macro_noop("\\theoremheaderfont{}")?;
  def_macro_noop("\\definetheoremstyle{}{}")?;
  def_macro_noop("\\settheoremtag{}")?;

  // Theorem-likes.
  RawTeX!(
    r"\newtheorem{theorem}{Theorem}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{proposition}[theorem]{Proposition}
\newtheorem{definition}[theorem]{Definition}
\newtheorem{example}[theorem]{Example}
\newtheorem{remark}[theorem]{Remark}"
  );
});

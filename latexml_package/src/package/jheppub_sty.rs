use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!({
  // Perl: jheppub.sty.ltxml — 112 lines
  RequirePackage!("hyperref");
  RequirePackage!("color");
  RequirePackage!("natbib");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("epsfig");
  RequirePackage!("graphicx");
  RequirePackage!("inst_support");

  // Author — Perl L32-34 carries `locked => 1`. The JHEP style overloads
  // \author to always record institute marks and feed through
  // \lx@author. Without the lock, latex.ltx \author (from article.cls)
  // or a user-side \newcommand\author can replace our institute-tagging
  // path and lose the [mark]-parsing branch.
  DefMacro!("\\author[]{}",
    "\\ifx.#1.\\else\\@institutemark{#1}\\fi\\def\\@author{#2}\\lx@author{#2}",
    locked => true);

  // Affiliation — Perl L36-38 has `beforeDigest => sub { AssignValue(inPreamble => 0); }`
  // so the body digests as if we're past \begin{document} — important since
  // \affiliation is typically used inside the preamble-style frontmatter block.
  // Without it, Rust left the inPreamble state on, which suppressed emitting the
  // note in some code paths.
  DefConstructor!("\\affiliation[]{}",
    "<ltx:note role='institutetext' mark='#1'>#2</ltx:note>", bounded => true,
    before_digest => {
      state::assign_value("inPreamble", false, None);
    });

  // Footnote alias — Perl L41
  Let!("\\note", "\\footnote");

  // Email — Perl L43-44
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\emailAdd Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");

  // Keywords — Perl L46-47
  DefMacro!("\\keywordname", "\\textbf{Keywords}");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}[name={\\keywordname}]{#1}");

  // Frontmatter metadata — Perl L49-54
  DefMacro!("\\arxivnumber{}", "\\@add@frontmatter{ltx:note}[role=arxiv]{#1}");
  DefMacro!("\\preprint{}", "\\@add@frontmatter{ltx:note}[role=preprint]{#1}");
  DefMacro!("\\proceeding{}", "\\@add@frontmatter{ltx:note}[role=proceeding]{#1}");
  DefMacro!("\\dedicated{}", "\\@add@frontmatter{ltx:note}[role=dedication]{#1}");
  DefMacro!("\\collaboration{}{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@collaborator{#2}}");
  def_macro_noop("\\collaborationImg[]{}")?;
  // \@@@collaborator internal — mirror aas_support's definition so the
  // expansion above resolves to actual XML markup instead of being
  // reported as undefined. Witness 2305.10497.
  DefConstructor!("\\@@@collaborator{}", "<ltx:note role='collaborator'>#1</ltx:note>");

  // Acknowledgements — Perl L56-60 emits `name='#name'` on
  // <ltx:acknowledgements> with the name digested from
  // \acknowledgmentsname. Rust was omitting the attribute, so
  // downstream templates that need the title (e.g. HTML post-proc)
  // had no name to pick up.
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='#name'>",
    properties => sub[_args] {
      Ok(stored_map!("name" =>
        stomach::digest(T_CS!("\\acknowledgmentsname"))?))
    });
  DefConstructor!("\\endacknowledgments", "</ltx:acknowledgements>");
  Tag!("ltx:acknowledgements", auto_close => true);

  // Conditionals — Perl L62-65
  DefConditional!("\\ifaffil");
  DefConditional!("\\ifnotoc");
  DefConditional!("\\ifemailadd");
  DefConditional!("\\iftoccontinuous");

  // Empty defaults — Perl L68-77
  DefMacro!("\\@subheader", "\\@empty");
  DefMacro!("\\@keywords", "\\@empty");
  DefMacro!("\\@abstract", "\\@empty");
  DefMacro!("\\@xtum", "\\@empty");
  DefMacro!("\\@dedicated", "\\@empty");
  DefMacro!("\\@arxivnumber", "\\@empty");
  DefMacro!("\\@collaboration", "\\@empty");
  DefMacro!("\\@collaborationImg", "\\@empty");
  DefMacro!("\\@proceeding", "\\@empty");
  DefMacro!("\\@preprint", "\\@empty");

  // Spacing macros — Perl L80-96
  DefMacro!("\\afterLogoSpace", "\\smallskip");
  DefMacro!("\\afterSubheaderSpace", "\\vskip3pt plus 2pt minus 1pt");
  DefMacro!("\\afterProceedingsSpace", "\\vskip21pt plus0.4fil minus15pt");
  DefMacro!("\\afterTitleSpace", "\\vskip23pt plus0.06fil minus13pt");
  DefMacro!("\\afterRuleSpace", "\\vskip23pt plus0.06fil minus13pt");
  DefMacro!("\\afterCollaborationSpace", "\\vskip3pt plus 2pt minus 1pt");
  DefMacro!("\\afterCollaborationImgSpace", "\\vskip3pt plus 2pt minus 1pt");
  DefMacro!("\\afterAuthorSpace", "\\vskip5pt plus4pt minus4pt");
  DefMacro!("\\afterAffiliationSpace", "\\vskip3pt plus3pt");
  DefMacro!("\\afterEmailSpace", "\\vskip16pt plus9pt minus10pt\\filbreak");
  DefMacro!("\\afterXtumSpace", "\\par\\bigskip");
  DefMacro!("\\afterAbstractSpace", "\\vskip16pt plus9pt minus13pt");
  DefMacro!("\\afterKeywordsSpace", "\\vskip16pt plus9pt minus13pt");
  DefMacro!("\\afterArxivSpace", "\\vskip3pt plus0.01fil minus10pt");
  DefMacro!("\\afterDedicatedSpace", "\\vskip0pt plus0.01fil");
  DefMacro!("\\afterTocSpace", "\\bigskip\\medskip");
  DefMacro!("\\afterTocRuleSpace", "\\bigskip\\bigskip");

  // Misc — Perl L99-109
  def_macro_noop("\\beforetochook")?;
  def_macro_noop("\\notoc")?;
  def_macro_noop("\\compress")?;
  // \correctionref{label}{url}{text} — link to a corrigendum.
  // Perl gobbles (raw \gdef binding hard to translate); we surpass
  // by emitting the text as a link via hyperref's \href.
  DefMacro!("\\correctionref{}{}{}", "\\href{#2}{#3}");
  DefMacro!("\\jname", "JHEP");
  // \subheader{text} — author-typed subheader prose; preserve as
  // ltx:note. Perl L? gobbles.
  DefMacro!("\\subheader{}",
    "\\@add@frontmatter{ltx:note}[role=subheader]{#1}");
  DefMacro!("\\xtumfont{}", "\\textsc{#1}");
  Let!("\\oldthebibliography", "\\thebibliography");
  Let!("\\endoldthebibliography", "\\endthebibliography");
});

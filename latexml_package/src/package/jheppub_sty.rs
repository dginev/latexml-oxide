use crate::prelude::*;

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

  // \author[]{} (Perl PR #2767)
  // One \author per author followed by \affiliation
  // OR both are supplied an optional label by which the affiliation is attached to author
  // optional arg is a label identifying which affiliation belongs
  DefMacro!("\\author[]{}", "\\lx@add@creator[role=author,annotations={#1}]{#2}");
  DefMacro!("\\affiliation OptionalSemiverbatim {}",
    "\\lx@add@contact[role=affiliation,label={#1}]{#2}");
  // \note{} appears inside author?
  DefMacro!("\\note{}", "\\lx@add@contact[role=note]{#1}");
  // The n-th \emailAdd is attached to the n-th author!
  DefMacro!("\\emailAdd Semiverbatim", "\\lx@add@contact[role=email,labelseq=author]{#1}");

  // Keywords
  DefMacro!("\\keywordname", "\\textbf{Keywords}");
  DefMacro!("\\keywords{}", "\\lx@add@keywords[name={\\keywordname}]{#1}");

  // Frontmatter metadata
  DefMacro!("\\arxivnumber{}", "\\lx@add@pubnote[role=arxiv]{#1}");
  DefMacro!("\\preprint{}", "\\lx@add@pubnote[role=preprint]{#1}");
  DefMacro!("\\proceeding{}", "\\lx@add@pubnote[role=conference]{#1}");
  DefMacro!("\\dedicated{}", "\\lx@add@pubnote[role=dedication]{#1}");
  DefMacro!("\\collaboration{}", "\\lx@add@pubnote[role=collaboration]{#1}");
  def_macro_noop("\\collaborationImg[]{}")?;

  // Acknowledgements — Perl L56-60 emits `name='#name'` on
  // <ltx:acknowledgements> with the name digested from
  // \acknowledgmentsname. Rust was omitting the attribute, so
  // downstream templates that need the title (e.g. HTML post-proc)
  // had no name to pick up.
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='#name'>",
    properties => sub[_args] {
      Ok(stored_map!("name" =>
        digest(T_CS!("\\acknowledgmentsname"))?))
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
    "\\lx@add@frontmatter{ltx:note}[role=subheader]{#1}");
  DefMacro!("\\xtumfont{}", "\\textsc{#1}");
  Let!("\\oldthebibliography", "\\thebibliography");
  Let!("\\endoldthebibliography", "\\endthebibliography");
});

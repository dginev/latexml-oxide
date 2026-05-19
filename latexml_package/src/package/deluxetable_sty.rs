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
  // Perl: deluxetable.sty.ltxml — deluxetable environment for AAS styles

  //======================================================================
  // 2.15.1 The deluxetable Environment
  DefMacro!("\\dummytable", "\\refstepcounter{table}");

  // Perl: DefMacroI('\deluxetable', '{}', '\set@deluxetable@template{#1}\def\@deluxetable@header{}\begin{table}');
  DefMacro!("\\deluxetable{}",
    "\\set@deluxetable@template{#1}\\def\\@deluxetable@header{}\\begin{table}");
  DefMacro!("\\enddeluxetable", "\\spew@tblnotes\\end{table}");

  // star variant — use runtime def_macro for star-containing names
  {
    let params = parse_parameters("{}", &T_CS!("\\deluxetable*"), true)?;
    let expansion = TokenizeInternal!(
      "\\set@deluxetable@template{#1}\\def\\@deluxetable@header{}\\begin{table}");
    def_macro(T_CS!("\\deluxetable*"), params, ExpansionBody::Tokens(expansion), None)?;
  }
  {
    let expansion = TokenizeInternal!("\\spew@tblnotes\\end{table}");
    def_macro(T_CS!("\\enddeluxetable*"), None, ExpansionBody::Tokens(expansion), None)?;
  }

  // Perl: DefMacro('\set@deluxetable@template AlignmentTemplate', sub { AssignValue(...); }).
  // Rust uses DefPrimitive — same AssignValue side effect at stomach time.
  // WISDOM #44: DefMacro↔DefPrimitive is NOT universally equivalent; safe
  // here because `\set@deluxetable@template` is only emitted by the
  // `\deluxetable{...}` expansion and consumed immediately in document
  // body, never captured by `\edef`.
  // WISDOM #44 verified 2026-04-23: zero `\edef`/`\ifx`/`\expandafter`
  // uses of `\set@deluxetable@template` across LaTeXML/lib + ar5iv-bindings.
  DefPrimitive!("\\set@deluxetable@template AlignmentTemplate", sub[(template)] {
    AssignValue!("@deluxetable@template", template);
  });

  DefMacro!("\\startdata",
    "\\bgroup\\@deluxetable@bindings\\@@deluxetabular\\lx@begin@alignment\\hline\\hline\\@deluxetable@header");
  DefMacro!("\\enddata",
    "\\\\\\hline\\lx@end@alignment\\@end@deluxetabular\\egroup");

  // Perl: DefPrimitive('\@deluxetable@bindings', sub { tabularBindings(LookupValue('@deluxetable@template')); });
  DefPrimitive!("\\@deluxetable@bindings", {
    let template_stored = lookup_value("@deluxetable@template");
    if let Some(Stored::Template(template_rc)) = template_stored {
      let template = (*template_rc).clone();
      tabular_bindings(template, SymHashMap::default(), HashMap::default())?;
    }
  });

  DefConstructor!("\\@@deluxetabular DigestedBody",
    "#1",
    reversion => "\\begin{tabular}[#1]{#2}#3\\end{tabular}",
    before_digest => { bgroup(); },
    mode => "restricted_horizontal"
  );
  DefPrimitive!("\\@end@deluxetabular", {
    egroup()?;
  });

  //======================================================================
  // 2.15.2 Preamble to the deluxetable

  DefRegister!("\\pt@width", Dimension!("6pt"));
  DefRegister!("\\pt@line", Dimension!("0pt"));
  DefRegister!("\\pt@column", Dimension!("0pt"));
  DefRegister!("\\pt@nlines", Dimension!("0pt"));
  DefRegister!("\\pt@ncol", Dimension!("0pt"));
  DefRegister!("\\pt@page", Dimension!("0pt"));

  def_macro_noop("\\tabletypesize{}")?;
  def_macro_noop("\\rotate")?;
  // \tabletail{text} — text shown at the bottom of every table page
  // (e.g. "Continued on next page"). HTML output is single-page so
  // we preserve the text as ltx:note role='tabletail' rather than
  // gobbling — author body, not config.
  DefMacro!("\\tabletail{}",
    "\\@add@frontmatter{ltx:note}[role=tabletail]{#1}");
  DefMacro!("\\tablewidth{Dimension}", "\\pt@width=#1\\relax");
  def_macro_noop("\\tableheadfrac{}")?;
  DefMacro!("\\tablenum{}", "\\def\\thetable{#1}");

  def_macro_noop("\\tablecolumns{Number}")?;

  Let!("\\tablecaption", "\\caption");

  DefMacro!("\\tablehead{}",
    "\\def\\@deluxetable@header{\\lx@alignment@begin@heading#1\\\\\\hline\\lx@alignment@end@heading}");
  DefMacro!("\\colhead{}", "\\multicolumn{1}{c}{#1}");
  DefMacro!("\\twocolhead{}", "\\multicolumn{2}{c}{\\hss #1 \\hss}");
  DefMacro!("\\nocolhead{}", "\\multicolumn{1}{h}{#1}");
  DefMacro!("\\dcolhead{}", "\\multicolumn{1}{c}{$\\relax#1$}");

  DefMacro!("\\nl", "\\\\[0pt]");
  DefMacro!("\\nextline", "\\\\[0pt]");
  DefMacro!("\\tablevspace{}", "\\noalign{\\vskip#1}");

  //======================================================================
  // 2.15.3 Content of deluxetable

  def_macro_noop("\\tablebreak")?;
  def_macro_noop("\\nodata")?;

  DefMacro!("\\cutinhead{}", "\\hline\\multicolumn{\\lx@alignment@ncolumns}{c}{#1}\\\\\\hline");
  DefMacro!("\\sidehead{}", "\\hline\\multicolumn{\\lx@alignment@ncolumns}{l}{#1}\\\\\\hline");

  DefMacro!("\\tableline", "\\hline");

  DefConstructor!("\\tablenotemark{}",
    "<ltx:note role='footnotemark' mark='#1'></ltx:note>",
    mode => "restricted_horizontal");
  DefConstructor!("\\tablenotetext{}{}",
    "<ltx:note role='footnotetext' mark='#1'>#2</ltx:note>",
    mode => "internal_vertical");

  // Perl uses AddToMacro to accumulate into \tblnote@list
  // We use \g@addto@macro which does the same at the TeX level
  DefMacro!("\\tablerefs{}", "\\g@addto@macro\\tblnote@list{\\@tableref{#1}\\let\\email\\@@email}");
  DefMacro!("\\@tableref{}", "\\par\n \\vspace*{3ex}%\n {\\parbox{\\textwidth}{\\hskip1em\\rmfamily References. --- #1}\\par}");

  DefMacro!("\\tablecomments{}", "\\g@addto@macro\\tblnote@list{\\@tablecom{#1}}");
  DefMacro!("\\@tablecom{}", "\\par \n \\vspace*{3ex}% \n {\\parbox{\\textwidth}{\\hskip1em\\rmfamily Note. --- #1}\\par}");

  DefMacro!("\\spew@tblnotes",
    "\\@tablenotes{\\tblnote@list}\\global\\let\\tblnote@list\\@empty");
  DefMacro!("\\@tablenotes{}", "\\par \n \\vspace{4.5ex}\\footnoterule\\vspace{.5ex}%\n {\\footnotesize #1}");

  //======================================================================
  // Other esoterica

  DefMacro!("\\ulap{}", "#1");
  DefMacro!("\\dlap{}", "#1");

  // Perl deluxetable.sty.ltxml L144-151 `AtBeginDocument` block. The
  // deferred timing matters for `\pt@width\textwidth`: at binding-load
  // \textwidth hasn't been sized by article.cls yet, so copying it here
  // yields the default 0pt. The \@empty lets and \pt@headfrac def are
  // order-insensitive but keep them together with the width init so the
  // block matches Perl one-for-one.
  at_begin_document(TokenizeInternal!(
    r"\let\tblnote@list\@empty\let\pt@caption\@empty\let\pt@head\@empty\let\pt@tail\@empty\pt@width\textwidth\def\pt@headfrac{.1}"
  ))?;
});

use crate::prelude::*;

/// What a table-note mark shows, the key that pairs a `\tablenotetext` with its `\tablenotemark`s (latex_constructs
/// `relocate_footnote` matches their `mark`s, ignoring spaces) and the mark the note prints: the characters and
/// symbols as set, in text or math, a command's shown arguments and a formula's body read for theirs (`$\dagger$`,
/// `^\dagger` and `\dag` are "†", `\rm{b}` and `\textrm{b}` are "b", `\hyperlink{tn5}{e}` is "e" and
/// `\raisebox{1pt}{g}` "g": a value, a key-value list or a semiverbatim name sets no character), its spaces collapsed
/// (`c, d` prints "c, d", as TeX sets it, and pairs with `c,d`). The one function serves both commands, so a mark
/// pairs by what it shows whatever its source. Math's `*` and `\ast` are TeX's one asterisk (plain.tex
/// `\mathcode`\*="2203`, `\mathchardef\ast="2203`), set here as U+2217: an asterisk, as a text `*` is.
fn shown_mark(digested: &Digested) -> String {
  /// Parameter types whose argument is a value, a name or a key-value list, which sets no character.
  const UNSHOWN: &[&str] = &[
    "Number",
    "NumExpr",
    "Dimension",
    "DimExpr",
    "Glue",
    "GlueExpr",
    "MuDimension",
    "MuGlue",
    "MuExpr",
    "Length",
    "Float",
    "TempboxaDimension",
    "GraphixDimension",
    "GraphixDimensions",
    "SetlengthDimension",
    "SetlengthGlue",
    "Keyword",
    "CSName",
    "CSNameQuiet",
    "Token",
    "XToken",
    "DefToken",
    "Undigested",
    "OptionalUndigested",
    "TeXFileName",
    "DirectoryList",
    "PackageOptions",
  ];
  fn shows(parameter: &Parameter) -> bool {
    let unshown = parameter.semiverbatim.is_some()
      || with(parameter.name, |name| {
        UNSHOWN.contains(&name) || name.contains("KeyVals")
      });
    !unshown
      && parameter
        .inner
        .as_ref()
        .is_none_or(|inner| inner.get_parameters().into_iter().any(shows))
  }
  fn shown(digested: &Digested, out: &mut String) {
    match digested.data() {
      DigestedData::List(list) => list.borrow().boxes.iter().for_each(|item| shown(item, out)),
      DigestedData::RegisterValue(_) | DigestedData::KeyVals(_) => {},
      DigestedData::Whatsit(whatsit) => {
        let whatsit = whatsit.borrow();
        let body = whatsit.get_body().ok().flatten();
        let args = whatsit.get_args();
        if args.iter().all(Option::is_none) && body.is_none() {
          out.push_str(&digested.to_attribute());
        } else {
          // One argument per valued parameter (`Parameters::read_arguments_and_digest`); without the parameters
          // every argument is read.
          let definition = whatsit.get_definition();
          let parameters: Vec<&Parameter> = definition
            .get_parameters()
            .map(|parameters| {
              parameters
                .get_parameters()
                .into_iter()
                .filter(|p| !p.novalue)
                .collect()
            })
            .unwrap_or_default();
          let aligned = parameters.len() == args.len();
          for (index, arg) in args.iter().enumerate() {
            if let Some(arg) = arg
              && (!aligned || shows(parameters[index]))
            {
              shown(arg, out);
            }
          }
          if let Some(body) = body {
            shown(&body, out);
          }
        }
      },
      _ => out.push_str(&digested.to_attribute()),
    }
  }
  let mut text = String::new();
  shown(digested, &mut text);
  text
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
    .chars()
    .map(|c| if c == '\u{2217}' { '*' } else { c })
    .collect()
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

  // (`\iflx@deluxetable@data`: the data is open, for an end that closes it itself — AASTeX 3's `{planotable}`,
  // aas_support_sty.rs)
  RawTeX!("\\newif\\iflx@deluxetable@data");
  DefMacro!("\\startdata",
    "\\global\\lx@deluxetable@datatrue\
     \\bgroup\\@deluxetable@bindings\\@@deluxetabular\\lx@begin@alignment\\hline\\hline\\@deluxetable@header");
  DefMacro!("\\enddata",
    "\\global\\lx@deluxetable@datafalse\\\\\\hline\\lx@end@alignment\\@end@deluxetabular\\egroup");

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
  // aastex62.cls:1849-1858 `\def\tablewidth#1{\ifdim#1=\z@ \else …\fi}`: for `0pt`, the common use, it
  // expands to nothing, so `\tablewidth{0pt}` at the start of a `tabular` row leaves the row's `\hline` a
  // `\noalign`. Perl's `\pt@width=#1\relax` (deluxetable.sty.ltxml:70) is an assignment, which opened a cell and
  // made the `\hline` misplaced (1805.09353). Guard: `perfect_kernel_batch65::tablewidth_zero_is_expandable`.
  DefMacro!("\\tablewidth{}", r"\ifdim#1=\z@\else\pt@width=#1\relax\fi");
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

  // aastex701.cls:12652 `\newcommand\tablebreak{\\[-11pt]\noalign{\break}}`: the row ends and the
  // page may break after it (the negative space hides the break's empty row). Its explicit `[…]`
  // keeps a next row led by `[` (`[Fe/H]`) from being read as the row spacing, as `\nl` above.
  // Perl's deluxetable.sty.ltxml:94 overrides its own `\\[0pt]` (L89) with nothing, so the next
  // row ran into this one: an extra `&` per cell (astro-ph0604363, 1811.07447).
  DefMacro!("\\tablebreak", "\\\\[0pt]");
  def_macro_noop("\\nodata")?;

  DefMacro!("\\cutinhead{}", "\\hline\\multicolumn{\\lx@alignment@ncolumns}{c}{#1}\\\\\\hline");
  DefMacro!("\\sidehead{}", "\\hline\\multicolumn{\\lx@alignment@ncolumns}{l}{#1}\\\\\\hline");

  DefMacro!("\\tableline", "\\hline");

  DefConstructor!("\\tablenotemark{}",
    "<ltx:note role='footnotemark' mark='#mark'></ltx:note>",
    mode => "restricted_horizontal",
    properties => sub[args] {
      let mark = args[0].as_ref().map(shown_mark).unwrap_or_default();
      Ok(stored_map!("mark" => Stored::String(pin(&mark))))
    });
  // The note's mark is its `mark`, what the mark shows (`shown_mark`), which pairs it with a `\tablenotemark` showing
  // the same. Perl writes both as the argument's text (deluxetable.sty.ltxml:103-108), so `\tablenotemark{$\dagger$}`
  // showed its TeX source and paired with no note (KNOWN_PERL_ERRORS #873). A class that sets the mark in math sets it
  // in a box (`\lx@deluxe@tablenote@mark`, AASTeX's `\hbox to5pt{$#1$}`, aas_support_sty.rs): `$a$` there is an empty
  // formula, the text "a" and another empty formula, as TeX sets it (1807.05231).
  DefMacro!("\\lx@deluxe@tablenote@mark{}", "\\hbox{$#1$}");
  DefConstructor!("\\lx@deluxe@tablenotetext{}{}",
    "<ltx:note role='footnotetext' mark='#mark'>#2</ltx:note>",
    mode => "internal_vertical", sizer => sub[_whatsit] { Ok(out_of_line_size()) },
    properties => sub[args] {
      let mark = args[0].as_ref().map(shown_mark).unwrap_or_default();
      Ok(stored_map!("mark" => Stored::String(pin(&mark))))
    },
    reversion => "\\tablenotetext{#1}{#2}");
  DefMacro!("\\tablenotetext{}{}", "\\lx@deluxe@tablenotetext{#1}{#2}");

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

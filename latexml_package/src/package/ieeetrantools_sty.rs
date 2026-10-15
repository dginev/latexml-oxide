//! IEEEtrantools.sty — the standalone IEEE alignment package (IEEEeqnarray,
//! IEEEeqnarraybox, …) for use with ANY class.
//!
//! Perl LaTeXML ships NO `IEEEtrantools.sty.ltxml` — it only binds the same
//! machinery inside `IEEEtran.cls.ltxml`. So a plain `article` +
//! `\usepackage{IEEEtrantools}` falls through to raw-loading IEEEtrantools.sty,
//! whose raw `\halign`-based IEEEeqnarray breaks LaTeXML's alignment model in
//! BOTH engines: an IEEEeqnarray row that begins with an empty cell (a leading
//! `&`, e.g. `\nonumber\\ & & +\beta\ldots`) raises
//! `\halign Attempt to end mode restricted_horizontal` and cascades
//! `_`/`^ can only appear in math mode`, mangling the equation. Perl fails the
//! same way. Witness: /home/deyan/Downloads/ieee_eqn_bug (main_arXiv.tex L554);
//! minimal reproducer docs/reproducers/ieeeeqnarray_leading_empty_cell.tex.
//!
//! This binding maps the IEEEeqnarray family onto LaTeXML's NATIVE alignment
//! (`\eqnarray`'s, over the columns of the specification), which handles
//! leading-empty cells correctly — so we surpass Perl here. The IEEEtran class
//! binding (`ieeetran_cls.rs`, Perl IEEEtran.cls.ltxml L242-332) shares it:
//! [`define_ieeeeqnarray`].
use latexml_core::alignment::template::TemplateConfig;

use crate::{
  engine::latex_constructs::{
    FontUnit, eqnarray_bindings_with_columns, math_array_bindings, symbol_box_size,
  },
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // IEEEtrantools.sty:86-106 `[redeflists]`: the standard lists become the IEEE ones; `retainorgcmds` (legacy) keeps
  // them.
  RawTeX!(r"\newif\if@IEEETOOLSredeflists");
  DeclareOption!("redeflists", r"\@IEEETOOLSredefliststrue");
  DeclareOption!("retainorgcmds", r"\@IEEETOOLSredeflistsfalse");
  ProcessOptions!();
  define_ieeeeqnarray()?;
  // 67x: the package's commands outside the IEEEeqnarray family that the IEEEtran class binding has and the
  // IEEEtrantools net's papers call (run 340: `{IEEEproof}` 36 papers, `\IEEEauthorrefmark` 4, `\IEEEPARstart` 3,
  // `{IEEEdescription}` 3, `\IEEEsetlabelwidth` 3, `\IEEEusemathlabelsep` 3, `\IEEEQEDhere`; witnesses 1710.09177,
  // 1802.00899, 2002.01226, 2003.00712, 1304.3949, 1609.02977, 1905.00616): the IEEEproof family and its QED switches
  // (:2726-2747), the IED lists and their parameters (:1000-1140; the standard lists are the IEEE ones under
  // `[redeflists]`, :1122-1129), `\IEEEauthorrefmark{<n>}` (:2721-2723, the footnote symbol sequence * † ‡ § ¶ ‖ ** ††
  // ‡‡, roman numerals from 10), `\IEEEPARstart{<letter>}{<rest>}` (:2616, the drop cap: its text, as the class
  // binding's).
  // Under the IEEEtran class (its binding defines the IEEEproof family), IEEEtrantools.sty only raises its `\PackageError`
  // (:73-78, "not for use with classes that already provide it"): the class binding's IEEEproof, IED lists and
  // `\IEEEauthorrefmark` stay, which its author-block matching reads (67x review probes d, k).
  if lookup_definition(&T_CS!("\\IEEEproof"))?.is_none() {
    define_ieeeproof()?;
    DefMacro!("\\IEEEproofname", "Proof");
    define_ied_lists()?;
    RawTeX!(r"\if@IEEETOOLSredeflists
\let\itemize\IEEEitemize \let\enditemize\endIEEEitemize
\let\enumerate\IEEEenumerate \let\endenumerate\endIEEEenumerate
\let\description\IEEEdescription \let\enddescription\endIEEEdescription
\expandafter\let\csname begin{itemize}\endcsname\IEEEitemize
\expandafter\let\csname end{itemize}\endcsname\endIEEEitemize
\expandafter\let\csname begin{enumerate}\endcsname\IEEEenumerate
\expandafter\let\csname end{enumerate}\endcsname\endIEEEenumerate
\expandafter\let\csname begin{description}\endcsname\IEEEdescription
\expandafter\let\csname end{description}\endcsname\endIEEEdescription
\fi
\DeclareRobustCommand*{\IEEEauthorrefmark}[1]{\raisebox{0pt}[0pt][0pt]{\textsuperscript{\footnotesize\ensuremath{\ifcase#1\or *\or \dagger\or \ddagger\or
    \mathsection\or \mathparagraph\or \|\or **\or \dagger\dagger
    \or \ddagger\ddagger \else\textsuperscript{\expandafter\romannumeral#1}\fi}}}}");
    DefMacro!("\\IEEEPARstart{}{}", "#1#2");
  }
  // IEEEtrantools.sty's commands outside the IEEEeqnarray family, as the IEEEtran class binding has
  // them (ieeetran_cls.rs; the class carries the same code): :1387 `\yesnumber`, the opposite of
  // `\nonumber` (`\global\@eqnswtrue` there; `\IEEEyesnumber` on our equation rows); :1904-1920
  // `\IEEEstrut[<height>][<depth>][<font>]`, a strut; :2756-2760 `\bstctlcite[<aux>]{<key>}`, which
  // writes the IEEEtran.bst control entry to the .aux; :445 `\IEEEsettextheight[<sample>]{<top>}{<bottom>}`
  // and the page geometry family papers call together (:365-860): `\IEEEsettextwidth{<inner>}{<outer>}`,
  // `\IEEEsetsidemargin{<mode>}{<margin>}`, `\IEEEquantizetextheight[<unit>]{<mode>}`,
  // `\IEEEsettopmargin[<sample>]{<mode>}{<margin>}`, `\IEEEsetheadermargin[<header sample>][<text sample>]{<mode>}{<margin>}`,
  // `\IEEEsetfootermargin[…][…]{<mode>}{<margin>}`; and the strut settings of an IEEEeqnarray's declarations
  // (:1769-1935 `\IEEEeqnarraystrutsize{<height>}{<depth>}[<font>]`, `\IEEEeqnarraystrutsizeadd`,
  // `\IEEEeqnarraystrutmode`, as the class binding defines them) — layout and BibTeX
  // control, which set nothing here. Run 340 first errors: `\bstctlcite` 88, `\IEEEstrut` 37,
  // `\yesnumber` 26, `\IEEEsettextheight` 7 (witnesses 2103.12231, 1310.8585, 2008.05910, 1310.1135,
  // 2008.05838, 2004.10847, 2211.06691, 2310.15756, 2306.14499).
  RawTeX!(r"\providecommand{\yesnumber}{\IEEEyesnumber\relax}");
  DefMacro!("\\IEEEstrut[][][]", "");
  DefMacro!("\\bstctlcite[]{}", "");
  DefMacro!("\\IEEEsettextheight[]{}{}", "");
  DefMacro!("\\IEEEsettopmargin[]{}{}", "");
  DefMacro!("\\IEEEquantizetextheight[]{}", "");
  DefMacro!("\\IEEEsettextwidth{}{}", "");
  DefMacro!("\\IEEEsetsidemargin{}{}", "");
  DefMacro!("\\IEEEsetheadermargin[][]{}{}", "");
  DefMacro!("\\IEEEsetfootermargin[][]{}{}", "");
  if lookup_definition(&T_CS!("\\IEEEeqnarraystrutsize"))?.is_none() {
    DefMacro!("\\IEEEeqnarraystrutsize{}{}[]", "");
  }
  DefMacro!("\\IEEEeqnarraystrutsizeadd{}{}[]", "");
  def_macro_noop("\\IEEEeqnarraystrutmode")?;
});

/// The IEEEeqnarray family (IEEEtrantools.sty; Perl IEEEtran.cls.ltxml L242-332), shared with the IEEEtran class
/// binding (`ieeetran_cls.rs`), whose class carries the same code.
#[rustfmt::skip]
pub fn define_ieeeeqnarray() -> Result<()> {
  // IEEEtrantools.sty:1357-1360, 1485-1539: the column types, each a `\@IEEEeqnarraycolPRE<name>` and
  // `\@IEEEeqnarraycolPOST<name>` put around the cell and a `\@IEEEeqnarraycolDEF<name>` saying it is defined —
  // the document's own `\IEEEeqnarraydefcol` adds to them. `L C R` drop the `{}` the package puts inside the math
  // (`$\IEEEeqnarraymathstyle{}`, an empty atom spacing a binary operator); LaTeXML's math parser spaces operators
  // itself, as Perl's L/C/R (IEEEtran.cls.ltxml:300-307) have none. The rule columns `v V vv VV h H` keep their
  // rules.
  RawTeX!(r"\def\IEEEeqnarraymathstyle{\displaystyle}
\def\IEEEeqnarraytextstyle{\relax}
\def\IEEEeqnarraydefcol#1#2#3{\expandafter\def\csname @IEEEeqnarraycolPRE#1\endcsname{#2}%
\expandafter\def\csname @IEEEeqnarraycolPOST#1\endcsname{#3}%
\expandafter\def\csname @IEEEeqnarraycolDEF#1\endcsname{1}}
\IEEEeqnarraydefcol{l}{$\IEEEeqnarraymathstyle}{$\hfil}
\IEEEeqnarraydefcol{c}{\hfil$\IEEEeqnarraymathstyle}{$\hfil}
\IEEEeqnarraydefcol{r}{\hfil$\IEEEeqnarraymathstyle}{$}
\IEEEeqnarraydefcol{L}{$\IEEEeqnarraymathstyle}{$\hfil}
\IEEEeqnarraydefcol{C}{\hfil$\IEEEeqnarraymathstyle}{$\hfil}
\IEEEeqnarraydefcol{R}{\hfil$\IEEEeqnarraymathstyle}{$}
\IEEEeqnarraydefcol{s}{\IEEEeqnarraytextstyle}{\hfil}
\IEEEeqnarraydefcol{t}{\hfil\IEEEeqnarraytextstyle}{\hfil}
\IEEEeqnarraydefcol{u}{\hfil\IEEEeqnarraytextstyle}{}
\IEEEeqnarraydefcol{v}{}{\vrule width\arrayrulewidth}
\IEEEeqnarraydefcol{vv}{\vrule width\arrayrulewidth\hfil}{\hfil\vrule width\arrayrulewidth}
\IEEEeqnarraydefcol{V}{}{\vrule width\arrayrulewidth\hskip\doublerulesep\vrule width\arrayrulewidth}
\IEEEeqnarraydefcol{VV}{\vrule width\arrayrulewidth\hskip\doublerulesep\vrule width\arrayrulewidth\hfil}%
{\hfil\vrule width\arrayrulewidth\hskip\doublerulesep\vrule width\arrayrulewidth}
\IEEEeqnarraydefcol{h}{}{\leaders\hrule height\arrayrulewidth\hfil}
\IEEEeqnarraydefcol{H}{}{\leaders\vbox{\hrule width\arrayrulewidth\vskip\doublerulesep\hrule width\arrayrulewidth}\hfil}
\IEEEeqnarraydefcol{x}{}{}
\IEEEeqnarraydefcol{X}{$}{$}
\IEEEeqnarraydefcol{@IEEEdefault}{\hfil$\IEEEeqnarraymathstyle}{$\hfil}");
  // The inter-column glues (IEEEtrantools.sty:1492) are not modelled.
  def_macro_noop("\\IEEEeqnarraydefcolsep{}{}")?;
  // IEEEtrantools.sty:1347-1353 (IEEEtran.cls:3028-3034): `\@IEEEeqnnumcols`, the number of columns the
  // specification defines (set by `\@IEEEeqnarray@bindings`/`\@IEEEeqnarraybox@bindings` below, as :2498/:2535
  // count them), and the column counter IEEEeqnarray advances (`\if@advanceIEEEeqncolcnt`, not modelled: false).
  // :1566-1690 (IEEEtran.cls:3247-3360) the separator and rule rows, spanning every column: `\IEEEeqnarrayseprow`
  // `[<height>][<font>]` (a blank row: the strut that sets its height, in the last column
  // `\@IEEEeqnarrayhoptolastcolumn` moves to, is not modelled), `\IEEEeqnarrayseprowcut`,
  // `\IEEEeqnarrayrulerow[<thickness>]` (a leader rule across the row), `\IEEEeqnarraydblrulerow[<thickness>]
  // [<spacing>]` and `\IEEEeqnarraydblrulerowcut` (two rule rows around a separator row), as written. 67x (run 340:
  // 46 papers; witnesses cs0608041, 0911.1713, 1004.2757, 1112.1125, 0908.3162, 1312.4895, 2310.13447, 1806.01342;
  // KNOWN_PERL_ERRORS #710). In an `{IEEEeqnarraybox}` (an array) a rule row is one cell spanning the columns, holding
  // its leader rule. In an `{IEEEeqnarray}` the rows are rearranged into equations, which keep only their math, as
  // `{eqnarray}`'s do (`\multispan{3}\leaders\hrule…`): a rule row is an equation with its number and no rule, and a
  // separator row, empty, is removed with its number (residual, #710; repro
  // alignment/ieeeeqnarray_rule_rows_keep_rules_and_numbers).
  RawTeX!(r"\newif\if@advanceIEEEeqncolcnt
\newcount\@IEEEeqnnumcols
\newcount\@IEEEeqncolcnt
\def\IEEEeqnarrayseprow{\relax\@ifnextchar[{\@IEEEeqnarrayseprow}{\@IEEEeqnarrayseprow[0.25\normalbaselineskip]}}
\def\@IEEEeqnarrayseprow[#1]{\relax\@ifnextchar[{\@@IEEEeqnarrayseprow[#1]}{\@@IEEEeqnarrayseprow[#1][\relax]}}
\def\@@IEEEeqnarrayseprow[#1][#2]{\IEEEeqnarraystrutsize{#1}{0pt}[\relax]\relax}
\def\IEEEeqnarrayseprowcut{\multispan{\@IEEEeqnnumcols}\relax
\if@advanceIEEEeqncolcnt\global\advance\@IEEEeqncolcnt by \@IEEEeqnnumcols\relax\fi
\@ifnextchar[{\@IEEEeqnarrayseprowcut}{\@IEEEeqnarrayseprowcut[0.25\normalbaselineskip]}}
\def\@IEEEeqnarrayseprowcut[#1]{\relax\@ifnextchar[{\@@IEEEeqnarrayseprowcut[#1]}{\@@IEEEeqnarrayseprowcut[#1][\relax]}}
\def\@@IEEEeqnarrayseprowcut[#1][#2]{\IEEEeqnarraystrutsize{#1}{0pt}[\relax]\relax}
\def\IEEEeqnarrayrulerow{\multispan{\@IEEEeqnnumcols}\relax
\if@advanceIEEEeqncolcnt\global\advance\@IEEEeqncolcnt by \@IEEEeqnnumcols\relax\fi
\@ifnextchar[{\@IEEEeqnarrayrulerow}{\@IEEEeqnarrayrulerow[\arrayrulewidth]}}
\def\@IEEEeqnarrayrulerow[#1]{\leaders\hrule height#1\hfil\relax
\IEEEeqnarraystrutsize{0pt}{0pt}[\relax]\relax}
\def\IEEEeqnarraydblrulerow{\multispan{\@IEEEeqnnumcols}\relax
\if@advanceIEEEeqncolcnt\global\advance\@IEEEeqncolcnt by \@IEEEeqnnumcols\relax\fi
\@ifnextchar[{\@IEEEeqnarraydblrulerow}{\@IEEEeqnarraydblrulerow[\arrayrulewidth]}}
\def\@IEEEeqnarraydblrulerow[#1]{\relax\@ifnextchar[{\@@IEEEeqnarraydblrulerow[#1]}%
{\@@IEEEeqnarraydblrulerow[#1][\doublerulesep]}}
\def\@@IEEEeqnarraydblrulerow[#1][#2]{\def\@IEEEeqnarraydblrulerowARG{#1}%
\ifx\@IEEEeqnarraydblrulerowARG\@empty\@IEEEeqnarrayrulerow[\arrayrulewidth]\else\@IEEEeqnarrayrulerow[#1]\relax\fi
\def\@IEEEeqnarraydblrulerowARG{#2}%
\ifx\@IEEEeqnarraydblrulerowARG\@empty\\\IEEEeqnarrayseprow[\doublerulesep][\relax]%
\else\\\IEEEeqnarrayseprow[#2][\relax]\fi
\\\multispan{\@IEEEeqnnumcols}%
\if@advanceIEEEeqncolcnt\global\advance\@IEEEeqncolcnt by \@IEEEeqnnumcols\relax\fi
\def\@IEEEeqnarraydblrulerowARG{#1}%
\ifx\@IEEEeqnarraydblrulerowARG\@empty\@IEEEeqnarrayrulerow[\arrayrulewidth]\else\@IEEEeqnarrayrulerow[#1]\fi}
\def\IEEEeqnarraydblrulerowcut{\multispan{\@IEEEeqnnumcols}\relax
\if@advanceIEEEeqncolcnt\global\advance\@IEEEeqncolcnt by \@IEEEeqnnumcols\relax\fi
\@ifnextchar[{\@IEEEeqnarraydblrulerowcut}{\@IEEEeqnarraydblrulerowcut[\arrayrulewidth]}}
\def\@IEEEeqnarraydblrulerowcut[#1]{\relax\@ifnextchar[{\@@IEEEeqnarraydblrulerowcut[#1]}%
{\@@IEEEeqnarraydblrulerowcut[#1][\doublerulesep]}}
\def\@@IEEEeqnarraydblrulerowcut[#1][#2]{\def\@IEEEeqnarraydblrulerowARG{#1}%
\ifx\@IEEEeqnarraydblrulerowARG\@empty\@IEEEeqnarrayrulerow[\arrayrulewidth]\else\@IEEEeqnarrayrulerow[#1]\fi
\def\@IEEEeqnarraydblrulerowARG{#2}%
\ifx\@IEEEeqnarraydblrulerowARG\@empty\\\IEEEeqnarrayseprowcut[\doublerulesep][\relax]%
\else\\\IEEEeqnarrayseprowcut[#2][\relax]\fi
\\\multispan{\@IEEEeqnnumcols}%
\if@advanceIEEEeqncolcnt\global\advance\@IEEEeqncolcnt by \@IEEEeqnnumcols\relax\fi
\def\@IEEEeqnarraydblrulerowARG{#1}%
\ifx\@IEEEeqnarraydblrulerowARG\@empty\@IEEEeqnarrayrulerow[\arrayrulewidth]\else\@IEEEeqnarrayrulerow[#1]\fi}");

  // IEEEtrantools.sty:1940-1996: `\IEEEeqnarray[<decl>]{<cols>}` is an eqnarray whose `\halign` preamble is built
  // from the column specification (`\@IEEEbuildpreamble`), every row numbered unless starred. Perl maps it to the
  // three-column `\eqnarray` and drops the specification (IEEEtran.cls.ltxml:289-295, a TODO), so a row of more
  // columns overran it: `{rCCCl}`'s `a & = & b & = & c` was "Extra alignment tab" and its scripts "can only appear in
  // math mode" (2508.03314, 2011.14178, 1309.2819). The rows are numbered and rearranged as the eqnarray's
  // (`\IEEEyesnumber` and the rest below). The `<decl>` (struts, `\IEEEeqnarraydecl` overrides) is not modelled.
  DefMacro!("\\IEEEeqnarray[]{}",
    "\\@IEEEeqnarray@bindings{#2}\\@@eqnarray\
     \\@equationgroup@numbering{numbered=1,preset=1,deferretract=1,grouped=1,aligned=1}\
     \\lx@begin@alignment");
  Let!("\\endIEEEeqnarray", "\\endeqnarray");
  DefMacro!("\\csname IEEEeqnarray*\\endcsname []{}",
    "\\@IEEEeqnarray@bindings{#2}\\@@eqnarray\
     \\@equationgroup@numbering{numbered=1,preset=1,retract=1,grouped=1,aligned=1}\
     \\lx@begin@alignment");
  Let!("\\endIEEEeqnarray*", "\\endeqnarray*");
  DefPrimitive!("\\@IEEEeqnarray@bindings{}", sub[(cols)] {
    let columns: Vec<Cell> = ieee_columns(&Expand!(cols))?
      .into_iter()
      .map(|(before, after)| Cell { before: Some(before), after: Some(after), empty: true, ..Cell::default() })
      .collect();
    set_column_count(columns.len());
    eqnarray_bindings_with_columns(columns)?;
    AssignValue!("IEEEeqnarray@mathcells" => false);
  });
  def_macro_noop("\\IEEEeqnarraynumspace")?;

  define_eqnarraybox()?;
  // IEEEtrantools.sty:1448-1475: `\IEEEeqnarraymulticol{<n>}{<type>}{<text>}` spans `<n>` columns with the column
  // type's material around `<text>`.
  DefMacro!("\\IEEEeqnarraymulticol{}{}{}", sub[(n, coltype, text)] {
    let (before, after) = ieee_column_type(&coltype.to_string())?;
    let (before, after) =
      if lookup_bool("IEEEeqnarray@mathcells") { math_cell_column(before, after) } else { (before, after) };
    let (lead, before) = split_hfil(before, true);
    let (after, trail) = split_hfil(after, false);
    // The span's alignment is the column type's `\hfil`s.
    let align = match (lead.is_empty(), trail.is_empty()) {
      (false, false) => "c",
      (false, true) => "r",
      _ => "l",
    };
    let mut out = vec![T_CS!("\\multicolumn"), T_BEGIN!()];
    out.extend(n.unlist());
    out.extend([T_END!(), T_BEGIN!(), T_LETTER!(align), T_END!(), T_BEGIN!()]);
    out.extend(before);
    out.extend(text.unlist());
    out.extend(after);
    out.push(T_END!());
    Ok(Tokens::new(out))
  });

  // IEEEnonumber/yesnumber/(no)subnumber (Perl IEEEtran.cls.ltxml L245-289):
  // flip the EQUATION_NUMBERING (starred) or EQUATIONROW_TAGS (unstarred)
  // retract/counter keys in place.
  DefPrimitive!("\\IEEEnonumber OptionalMatch:*", sub[(star)] {
    let key = if star.is_some() { "EQUATION_NUMBERING" } else { "EQUATIONROW_TAGS" };
    with_value_mut(key, |v| {
      if let Some(Stored::HashStored(m)) = v {
        m.insert("retract", Stored::Bool(true));
        m.remove("counter");
      }
    });
    Ok(())
  });
  DefPrimitive!("\\IEEEyesnumber OptionalMatch:*", sub[(star)] {
    let subeq = with_value("EQUATION_NUMBERING", |v| {
      if let Some(Stored::HashStored(m)) = v {
        matches!(m.get("counter"),
          Some(Stored::String(s)) if to_string(*s) == "subequation")
      } else { false }
    });
    if subeq {
      RefStepCounter!("equation", false)?;
    }
    if star.is_some() {
      with_value_mut("EQUATION_NUMBERING", |v| {
        if let Some(Stored::HashStored(m)) = v {
          m.insert("retract", Stored::Bool(false));
          m.remove("counter");
        }
      });
    } else {
      with_value_mut("EQUATIONROW_TAGS", |v| {
        if let Some(Stored::HashStored(m)) = v {
          m.insert("noretract", Stored::Bool(true));
          m.remove("counter");
        }
      });
    }
    Ok(())
  });
  DefPrimitive!("\\IEEEyessubnumber OptionalMatch:*", sub[(star)] {
    let key = if star.is_some() { "EQUATION_NUMBERING" } else { "EQUATIONROW_TAGS" };
    with_value_mut(key, |v| {
      if let Some(Stored::HashStored(m)) = v {
        m.insert("counter", Stored::String(pin!("subequation")));
      }
    });
    let preset = with_value("EQUATION_NUMBERING", |v| {
      matches!(v, Some(Stored::HashStored(m)) if m.contains_key("preset"))
    }) || with_value("EQUATIONROW_TAGS", |v| {
      matches!(v, Some(Stored::HashStored(m)) if m.contains_key("preset"))
    });
    if preset {
      RefStepCounter!("subequation", false)?;
    }
    Ok(())
  });
  DefPrimitive!("\\IEEEnosubnumber OptionalMatch:*", sub[(star)] {
    let key = if star.is_some() { "EQUATION_NUMBERING" } else { "EQUATIONROW_TAGS" };
    with_value_mut(key, |v| {
      if let Some(Stored::HashStored(m)) = v {
        m.insert("counter", Stored::String(pin!("equation")));
      }
    });
    Ok(())
  });

  // Column types L/C/R (Perl IEEEtran.cls.ltxml L305-311): flush-left,
  // centered, flush-right via \hfil before/after hooks.
  DefColumnType!("L", {
    with_building_template(|template| {
      template.add_column(Cell {
        after: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("C", {
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        after:  Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("R", {
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  Ok(())
}

/// The IEEEeqnarraybox family — a variant of `\array` (Perl IEEEtran.cls.ltxml L315-332) over the IEEEtrantools
/// column types (IEEEtrantools.sty:2194 `\@IEEEbuildpreamble`). `\ifmmode` dispatches to the math (m) or text (t)
/// form.
pub fn define_eqnarraybox() -> Result<()> {
  RawTeX!(
    r"\def\IEEEeqnarraybox{\ifmmode\def\@tempa{\let\endIEEEeqnarraybox\endIEEEeqnarrayboxm\IEEEeqnarrayboxm}\else\def\@tempa{\let\endIEEEeqnarraybox\endIEEEeqnarrayboxt\IEEEeqnarrayboxt}\fi\@tempa}"
  );
  // IEEEtrantools.sty:2166-2173: `[<decl>][<pos>][<width>]{<cols>}` (Perl takes `{<cols>}` alone, so a box written
  // with its options read `[` as the column specification).
  DefMacro!("\\IEEEeqnarrayboxm OptionalMatch:* [][][] {}", sub[(star, decl, pos, width, cols)] {
    Ok(eqnarraybox_begin(star.is_some(), [decl, pos, width], &cols, false))
  });
  DefMacro!("\\endIEEEeqnarrayboxm", "\\lx@end@alignment\\@end@array");
  DefMacro!("\\IEEEeqnarrayboxt OptionalMatch:* [][][] {}", sub[(star, decl, pos, width, cols)] {
    Ok(eqnarraybox_begin(star.is_some(), [decl, pos, width], &cols, true))
  });
  DefMacro!(
    "\\endIEEEeqnarrayboxt",
    "\\lx@end@alignment\\@end@array\\lx@end@inline@math"
  );
  // The array over the specification's column types, its cells math already: a math column's `$` comes out, a text
  // column's goes around it (`math_cell_column`). `[<pos>]` (`t`, `c`, `b`) attaches the box as an array's.
  DefPrimitive!("\\@IEEEeqnarraybox@bindings [] {}", sub[(pos, cols)] {
    let columns: Vec<Cell> = ieee_columns(&Expand!(cols))?
      .into_iter()
      .map(|(before, after)| {
        let (before, after) = math_cell_column(before, after);
        Cell { before: Some(before), after: Some(after), ..Cell::default() }
      })
      .collect();
    set_column_count(columns.len());
    let template = Template::new(TemplateConfig {
      columns: Some(columns),
      ..TemplateConfig::default()
    });
    math_array_bindings(pos.as_ref(), template)?;
    AssignValue!("IEEEeqnarray@mathcells" => true);
  });
  // `{<star>}{<options>{<cols>}}`, as written, for the reversion.
  DefConstructor!("\\@@IEEE@array Undigested Undigested DigestedBody", "#3",
    before_digest => { bgroup(); },
    reversion => "\\begin{IEEEeqnarraybox#1}#2#3\\end{IEEEeqnarraybox#1}");
  // IEEEtrantools.sty:2145-2159: the starred environments are the boxes without the `\jot` row padding (2609.32652
  // `{IEEEeqnarraybox*}`): the same alignment here, the star kept for the reversion.
  RawTeX!(
    r"\expandafter\def\csname IEEEeqnarraybox*\endcsname{\IEEEeqnarraybox*}
\expandafter\def\csname endIEEEeqnarraybox*\endcsname{\endIEEEeqnarraybox}
\expandafter\def\csname IEEEeqnarrayboxm*\endcsname{\IEEEeqnarrayboxm*}
\expandafter\let\csname endIEEEeqnarrayboxm*\endcsname\endIEEEeqnarrayboxm
\expandafter\def\csname IEEEeqnarrayboxt*\endcsname{\IEEEeqnarrayboxt*}
\expandafter\let\csname endIEEEeqnarrayboxt*\endcsname\endIEEEeqnarrayboxt"
  );
  Ok(())
}

/// `\@IEEEeqnnumcols`, the number of columns of the specification (IEEEtrantools.sty:2498, :2535), for the rows that
/// span them all (`\IEEEeqnarrayrulerow` and the rest).
fn set_column_count(count: usize) {
  AssignRegister!("\\@IEEEeqnnumcols", Number::new(count as i64).into());
}

/// The alignment an IEEEeqnarraybox opens over the column specification `cols`; `\@@IEEE@array` keeps `cols` whole
/// for the reversion.
fn eqnarraybox_begin(
  star: bool,
  options: [Option<Tokens>; 3],
  cols: &Tokens,
  text: bool,
) -> Tokens {
  let mut out = Vec::new();
  if text {
    out.push(T_CS!("\\lx@begin@inline@math"));
  }
  out.push(T_CS!("\\@IEEEeqnarraybox@bindings"));
  if let [_, Some(pos), _] = &options {
    out.push(T_OTHER!("["));
    out.extend_from_slice(pos.unlist_ref());
    out.push(T_OTHER!("]"));
  }
  out.push(T_BEGIN!());
  out.extend_from_slice(cols.unlist_ref());
  out.push(T_END!());
  out.push(T_CS!("\\@@IEEE@array"));
  out.push(T_BEGIN!());
  if star {
    out.push(T_OTHER!("*"));
  }
  out.push(T_END!());
  out.push(T_BEGIN!());
  // The options up to the last one given, an earlier one omitted written `[]` as IEEEtrantools reads it.
  let given = options
    .iter()
    .rposition(Option::is_some)
    .map_or(0, |last| last + 1);
  for option in &options[..given] {
    out.push(T_OTHER!("["));
    if let Some(option) = option {
      out.extend_from_slice(option.unlist_ref());
    }
    out.push(T_OTHER!("]"));
  }
  out.push(T_BEGIN!());
  out.extend_from_slice(cols.unlist_ref());
  out.push(T_END!());
  out.push(T_END!());
  out.push(T_CS!("\\lx@begin@alignment"));
  Tokens::new(out)
}

/// The material around each column's cells — `(\@IEEEeqnarraycolPRE<type>, \@IEEEeqnarraycolPOST<type>)` — of a
/// column specification, fully expanded as IEEEtrantools.sty:2280 `\edef`s it (macro containers) and read as
/// `\@@IEEEbuildpreamble` (:2289-2310) does: a letter, or a braced name starting with one (`{vv}`), is a column of
/// that type (`\@IEEEgetcoltype`, :2333-2355); a digit or one of the punctuation glues `! , : ; ' " . / ? * + -`
/// (:2373-2422) spaces the columns and makes none, so `{,c/c/c,}` is three centred columns (the glues are not
/// modelled). Anything else is the package's error: punctuation no glue names (:2418), a control sequence (:2352).
fn ieee_columns(expanded: &Tokens) -> Result<Vec<(Tokens, Tokens)>> {
  let mut columns = Vec::new();
  let mut tokens = expanded.unlist_ref().iter();
  while let Some(token) = tokens.next() {
    match token.get_catcode() {
      Catcode::LETTER => columns.push(ieee_column_type(&token.to_string())?),
      Catcode::BEGIN => {
        let mut depth = 1usize;
        let mut name = String::new();
        let mut starts_with_letter = None;
        for inner in tokens.by_ref() {
          match inner.get_catcode() {
            Catcode::BEGIN => depth += 1,
            Catcode::END => {
              depth -= 1;
              if depth == 0 {
                break;
              }
            },
            _ => {},
          }
          starts_with_letter.get_or_insert(inner.get_catcode() == Catcode::LETTER);
          name.push_str(&inner.to_string());
        }
        if starts_with_letter == Some(true) {
          columns.push(ieee_column_type(&name)?);
        }
      },
      Catcode::SPACE | Catcode::END => {},
      Catcode::OTHER => {
        if !with(token.text, |c| {
          c.len() == 1 && "!,:;'\"./?*+-0123456789".contains(c)
        }) {
          let what = token.to_string();
          Error!(
            "unexpected",
            what,
            s!(
              "Invalid predefined inter-column glue type \"{what}\" in IEEEeqnarray column specifications. Using a default value of 0pt instead"
            ),
            "Only !,:;'\"./?*+ and - are valid predefined glue types in the IEEEeqnarray column specifications."
          );
        }
      },
      _ => {
        let what = token.to_string();
        Error!(
          "unexpected",
          what,
          s!("Invalid character \"{what}\" in IEEEeqnarray column specifications"),
          "Only letters, numerals and certain other symbols are allowed as IEEEeqnarray column specifiers."
        );
      },
    }
  }
  Ok(columns)
}

/// The material around a cell of the column type `name` (IEEEtrantools.sty:1485 `\IEEEeqnarraydefcol`); an undefined
/// type is the package's error and its default centred math column (:2362-2367 `\@IEEEgetcurcol`).
fn ieee_column_type(name: &str) -> Result<(Tokens, Tokens)> {
  let defined = |name: &str| -> Result<bool> {
    Ok(lookup_definition(&T_CS!(s!("\\@IEEEeqnarraycolDEF{name}")))?.is_some())
  };
  let name = if defined(name)? {
    name
  } else {
    Error!(
      "unexpected",
      name,
      s!(
        "Invalid column type \"{name}\" in IEEEeqnarray column specifications. Using a default centering column instead"
      ),
      "You must define IEEEeqnarray column types before use."
    );
    "@IEEEdefault"
  };
  let body = |part: &str| -> Result<Tokens> {
    Ok(
      match lookup_definition(&T_CS!(s!("\\@IEEEeqnarraycol{part}{name}")))? {
        Some(defn) => match defn.get_expansion() {
          Some(ExpansionBody::Tokens(body)) => body.clone(),
          _ => Tokens!(),
        },
        None => Tokens!(),
      },
    )
  };
  Ok((body("PRE")?, body("POST")?))
}

/// A column's material for cells that are math already (the IEEEeqnarraybox's array, whose `$` leaves math,
/// `\lx@dollar@in@mathmode`): the `$`s a math column enters math with come out, and a text column (`s t u`, the
/// rules, `x`) is put in `$…$`, outside its `\hfil`s.
fn math_cell_column(before: Tokens, after: Tokens) -> (Tokens, Tokens) {
  let is_shift = |t: &Token| t.get_catcode() == Catcode::MATH;
  if before
    .unlist_ref()
    .iter()
    .chain(after.unlist_ref())
    .any(is_shift)
  {
    let strip = |tokens: Tokens| {
      Tokens::new(
        tokens
          .unlist()
          .into_iter()
          .filter(|t| !is_shift(t))
          .collect(),
      )
    };
    return (strip(before), strip(after));
  }
  let (mut lead, before) = split_hfil(before, true);
  let (mut after, trail) = split_hfil(after, false);
  lead.push(T_MATH!());
  lead.extend(before);
  after.push(T_MATH!());
  after.extend(trail);
  (Tokens::new(lead), Tokens::new(after))
}

/// `tokens` split at its leading `\hfil`s (`(hfils, rest)`) or at its trailing ones (`(rest, hfils)`).
fn split_hfil(tokens: Tokens, leading: bool) -> (Vec<Token>, Vec<Token>) {
  let hfil = T_CS!("\\hfil");
  let mut tokens = tokens.unlist();
  if leading {
    let n = tokens.iter().take_while(|t| **t == hfil).count();
    let rest = tokens.split_off(n);
    (tokens, rest)
  } else {
    let n = tokens.iter().rev().take_while(|t| **t == hfil).count();
    let hfils = tokens.split_off(tokens.len() - n);
    (tokens, hfils)
  }
}

/// The QED symbols and `{IEEEproof}` (IEEEtran.cls:5533-5556, IEEEtrantools.sty:2726-2747; Perl IEEEtran.cls.ltxml
/// L194-222), shared with the IEEEtran class binding (`ieeetran_cls.rs`), whose class carries the same code.
#[rustfmt::skip]
pub fn define_ieeeproof() -> Result<()> {
  // QED symbols (Perl L194-198)
  // Sized as IEEEtran.cls:5533's `\mbox{\rule[0pt]{1.3ex}{1.3ex}}`: a size-less whatsit alone in an alignment cell
  // made the cell "empty" (Alignment::normalize, Perl Alignment.pm:458-470) and the QED was dropped — an
  // `{IEEEeqnarray}` text cell `& \hfill\IEEEQEDhere` (2201.11150, 1903.06134), an eqnarray `\mbox{\IEEEQED}`
  // (synthetic). The other classes' QEDs alike (`symbol_box_size`; llncs, svjour3/svmult, A&A, mn2e, elsart, amsthm).
  DefConstructor!("\\IEEEQEDclosed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true,
    sizer => sub[whatsit] { Ok(symbol_box_size(whatsit, 1.3, 1.3, FontUnit::Ex)) });
  Let!("\\IEEEQEDopen", "\\IEEEQEDclosed");
  Let!("\\IEEEQED", "\\IEEEQEDclosed");
  // IEEEtran.cls:5543 `\newif\if@IEEEQEDshow \@IEEEQEDshowtrue`, the class's own switch (a document may set it), true
  // at class load: a bare `\endIEEEproof` prints its QED too.
  RawTeX!(r"\newif\if@IEEEQEDshow \@IEEEQEDshowtrue");


  // IEEEproof (Perl IEEEtran.cls.ltxml:199-222), a constructor pair as amsthm's `\@proof`/`\end@proof`
  // (amsthm.sty.ltxml:147-175): IEEEtran.cls:5547-5549 makes it two macros, `\IEEEproof` (`\par\noindent…{\itshape
  // #1: }`, #1 defaulting to `\IEEEproofname`) and `\endIEEEproof` (the QED, `\par`), so its body stays in the mode
  // around it and a `$$` there is display math (tex.web:21715 `init_math`). Perl's DefEnvironment boxes the body in
  // restricted horizontal mode, where `$$` is two empty inline formulas (TeX_Math.pool.ltxml:65) and the formula ran
  // as text: "Script _ can only appear in math mode" (1203.1892, 1402.4543, 0802.1555, 1612.01904). A bare
  // `\endIEEEproof` (author error) closes a `ltx:proof` only when one is open (`maybe_close_element`), never a mode
  // frame (2009.01572: a mode-bound IEEEproof's `\endIEEEproof` popped the locked document frame, Fatal), and an
  // `\end{IEEEproof}` after the element closed reports no "ltx:proof isn't open" (1001.3714, 0801.0061).
  // The title is the optional argument (IEEEtran.cls:5548), which Perl's fixed "Proof:" dropped; the font is Perl's
  // bold italic. Perl (L205-221) pushes `\qed` on the QED@stack amsthm shares and digests it at the end unless Perl's
  // `\IEEEQEDhere` took it; here, as IEEEtran.cls:5543-5556, the end typesets `\IEEEQED` while IEEEtran's own switch
  // `\if@IEEEQEDshow` is true — set locally by `\@IEEEproof`, cleared globally by `\IEEEQEDoff`/`\IEEEQEDhere`/
  // `\IEEEQEDhereeqn` — so no switch reaches an amsthm proof's QED nor amsthm's `\qedhere` this one's, and a global
  // false outlives a nested IEEEproof's local true (repros block-model/ieeeproof_dollardollar_is_display,
  // block-model/ieeeproof_qed_switches).
  DefConstructor!("\\IEEEproof OptionalUndigested",
    "<ltx:proof class='#class'><ltx:title font='#titlefont' _force_font='true' class='#titleclass'>#title</ltx:title>",
    after_digest => sub[whatsit] {
      // IEEEtran.cls:5548 `\@IEEEQEDshowtrue`, local to the environment's group.
      digest(Tokens!(T_CS!("\\@IEEEQEDshowtrue")))?;
    },
    properties => sub[args] {
      let mut title_tokens = vec![T_BEGIN!(), T_CS!("\\bfseries"), T_CS!("\\itshape")];
      match args.first() {
        Some(Some(arg)) => title_tokens.extend(arg.revert()?.unlist()),
        _ => title_tokens.push(T_CS!("\\IEEEproofname")),
      }
      title_tokens.push(T_OTHER!(":"));
      title_tokens.push(T_END!());
      let title = digest(Tokens::new(title_tokens))?;
      // The template engine binds the `"font"` prop to the element's font attribute (amsthm_sty.rs `\@proof`).
      let titlefont = title.get_font().ok().flatten().map(|f| (*f).clone());
      let mut map = SymHashMap::default();
      map.insert("title", title.into());
      if let Some(f) = titlefont {
        map.insert("font", Stored::Font(Rc::new(f)));
      }
      map.insert("titleclass", "ltx_runin".into());
      Ok(map)
    });
  DefConstructor!("\\endIEEEproof", sub[document, _args] {
    document.maybe_close_element("ltx:proof")?;
  },
  // IEEEtran.cls:5549 `\if@IEEEQEDshow … \IEEEQED\fi`: the document's `\IEEEQED`, and nothing an amsthm `\qedhere`
  // could take (2001.04812).
  before_digest => {
    if x_equals(&T_CS!("\\if@IEEEQEDshow"), &T_CS!("\\iftrue")) {
      return Ok(vec![digest(Tokens!(T_CS!("\\IEEEQED")))?]);
    }
    Ok(vec![])
  });
  // IEEEtran.cls:5556 `\IEEEQEDoff` = `\global\@IEEEQEDshowfalse`: no QED at the end of this IEEEproof (synthetic, no
  // live witness found). :5554 `\IEEEQEDhere` the same and the QED where it stands (2201.01339, at the end of an
  // itemize item, `\hfill \IEEEQEDhere`; 2201.11150, IEEEeqnarray cells; 1410.7694); :5551-5552 `\IEEEQEDhereeqn` the
  // same with the QED as the display's equation number (1903.06134, 2201.03502). Perl's `\IEEEQEDhere`
  // (IEEEtran.cls.ltxml:193-196) took the top of the QED@stack an amsthm proof shares; Perl has no `\IEEEQEDhereeqn`.
  RawTeX!(r"\def\IEEEQEDoff{\global\@IEEEQEDshowfalse}");
  DefMacro!("\\IEEEQEDhere", "\\IEEEQEDoff\\IEEEQED");
  // The `\eqno` material is the QED and, in a numbered `equation`, LaTeX's own `\eqno\hbox{\@eqnnum}` after it — the
  // `\let\eqno\relax` makes it ONE tag, "■(1)", and a second `\IEEEQEDhereeqn` adds to it ("■■(1)"). The equation's
  // number is already its `ltx:tags`, so the QED's material moves into that tags' displayed (role-less) tag, before the
  // number: one `ltx:tags`, the scan's refnum still the number's (its `refnum` tag comes last), the page "∎(1)"; a
  // numbered `{IEEEeqnarray}` row's alike. A QED's material is moved as elements only (its element children, or an
  // `ltx:text` holding them when text nodes are among them — libxml merges, and frees, adjacent text nodes on
  // insertion), marked `_qed` so a later QED goes after the earlier ones, before the number. A second
  // `\IEEEQEDhereeqn` inside the first's material is relaxed as `\eqno` is (IEEEtran.cls:5551), adding to its tag.
  RawTeX!(r"\def\IEEEQEDhereeqn{\IEEEQEDoff\lx@IEEEQED@eqno\let\lx@IEEEQED@eqno\relax\let\eqno\relax\let\leqno\relax\let\veqno\relax\hbox{\IEEEQED}}");
  // `\eqno` is a display-math command (tex.web §1140): in a `{IEEEeqnarray}` row,
  // an alignment cell, pdflatex reports "You can't use `\eqno' in math mode" and
  // typesets the QED beside the number anyway — what we build, so a Warning.
  DefConstructor!("\\lx@IEEEQED@eqno EqnoTag",
    "^ <ltx:tags><ltx:tag><ltx:Math><ltx:XMath>#1</ltx:XMath></ltx:Math></ltx:tag></ltx:tags>",
    before_digest => {
      if lookup_string("MODE") != "display_math" {
        Warn!(
          "unexpected",
          "\\eqno",
          "You can't use `\\eqno' in math mode: \\IEEEQEDhereeqn outside a display (IEEEtran.cls:5551-5552); \
           the QED is set with the row's number"
        );
      }
    },
    reversion => "",
    after_construct => sub[document, _whatsit] {
      let node = document.get_node().clone();
      let Some(equation) = document
        .findnodes("ancestor-or-self::ltx:equation[1]", Some(&node))
        .into_iter()
        .next()
      else {
        return Ok(());
      };
      let tags = document.findnodes("ltx:tags", Some(&equation));
      let Some(mut qed_tags) = tags.last().cloned() else {
        return Ok(());
      };
      let Some(mut qed_tag) = document.findnodes("ltx:tag", Some(&qed_tags)).into_iter().next() else {
        return Ok(());
      };
      // The QED's material as marked elements: its element children, or one `ltx:text` holding them when text nodes are
      // among them.
      let children = qed_tag.get_child_nodes();
      let mut material: Vec<Node> = if children.iter().all(|child| child.get_type() == Some(NodeType::ElementNode)) {
        children
      } else {
        let mut wrapper = document.open_element_at(&mut qed_tag, "ltx:text", None, None)?;
        for mut child in children {
          child.unlink_node();
          if child.get_type() == Some(NodeType::TextNode) {
            wrapper.append_text(&child.get_content()).ok();
          } else {
            wrapper.add_child(&mut child).ok();
          }
        }
        document.close_element_at(&mut wrapper)?;
        let _ = wrapper.remove_attribute("font");
        vec![wrapper]
      };
      for element in material.iter_mut() {
        element.set_attribute("_qed", "1").ok();
      }
      // The equation's own displayed tag, when it has one before ours.
      let Some(number_tags) = tags.first().filter(|first| **first != qed_tags) else {
        return Ok(());
      };
      let Some(mut number_tag) = document
        .findnodes("ltx:tag[not(@role)]", Some(number_tags))
        .into_iter()
        .next()
      else {
        return Ok(());
      };
      let before = number_tag
        .get_child_nodes()
        .into_iter()
        .find(|child| child.get_attribute("_qed").is_none());
      for element in material.iter_mut() {
        element.unlink_node();
        match &before {
          Some(before) => {
            before.clone().add_prev_sibling(element).ok();
          },
          None => {
            number_tag.add_child(element).ok();
          },
        }
      }
      qed_tags.unlink_node();
    });
  Ok(())
}

/// The IED lists and their parameters (IEEEtran.cls:2028-2153, IEEEtrantools.sty:1000-1140; Perl IEEEtran.cls.ltxml
/// L340-366), shared with the IEEEtran class binding (`ieeetran_cls.rs`).
#[rustfmt::skip]
pub fn define_ied_lists() -> Result<()> {
  // IED list stubs (Perl L340-347)
  DefMacro!("\\IEEEsetlabelwidth{}", "\\settowidth{\\labelwidth}{#1}");
  def_macro_noop("\\IEEEusemathlabelsep")?;
  def_macro_noop("\\IEEEtriggercmd{}")?;
  // IEEEtran.cls:2031-2059: the IED lists' label indents are dimens a document sets (`\IEEEilabelindent
  // \IEEEilabelindentB`, 2609.36487); Perl's empty `\IEEElabelindent` made `\IEEElabelindent\parindent` an
  // assignment to `\parindent` (KNOWN_PERL_ERRORS #514).
  RawTeX!(r"\newdimen\IEEEilabelindentA \IEEEilabelindentA\parindent
\newdimen\IEEEilabelindentB \IEEEilabelindentB 1.3\parindent
\newdimen\IEEEilabelindent \IEEEilabelindent\IEEEilabelindentA
\newdimen\IEEEelabelindent \IEEEelabelindent\parindent
\newdimen\IEEEdlabelindent \IEEEdlabelindent\parindent
\newdimen\IEEElabelindent \IEEElabelindent\parindent");
  def_macro_noop("\\IEEEcalcleftmargin{}")?;
  def_macro_noop("\\IEEEiedlabeljustifyc")?;
  def_macro_noop("\\IEEEiedlabeljustifyl")?;
  def_macro_noop("\\IEEEiedlabeljustifyr")?;

  // IEEEitemize/enumerate/description (Perl IEEEtran.cls.ltxml L351-366).
  // Each env:
  //   - runs `beginItemize('<kind>', '<counter>')` via properties
  //     closure — registers the nested list level, resets counters,
  //     and wires the itemize/enumerate counter-label machinery.
  //   - digests `\par` on close so trailing text in the last item
  //     closes its <ltx:item> cleanly instead of leaking whitespace.
  //   - locks against further redefinition (matches Perl `locked=>1`).
  // {IEEEdescription} additionally re-`\let`s `\makelabel` to
  // `\descriptionlabel` at env start (Perl L363), mirroring LaTeX's
  // core description-env plumbing.
  DefEnvironment!("{IEEEitemize}[]",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    properties => sub[_args] { BeginItemize!("itemize", "@item") },
    locked => true,
    mode => "internal_vertical");
  DefEnvironment!("{IEEEenumerate}[]",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    properties => sub[_args] { BeginItemize!("enumerate", "enum") },
    locked => true,
    mode => "internal_vertical");
  DefEnvironment!("{IEEEdescription}[]",
    "<ltx:description xml:id='#id'>#body</ltx:description>",
    before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
    properties => sub[_args] { BeginItemize!("description", "@desc") },
    locked => true,
    mode => "internal_vertical");
  // IEEEtran.cls:2085-2153 (IEEEtrantools.sty:1007-1040): the label separations, the list top space, the user hook
  // `\IEEEiedlistdecl` and the margin switches, which documents set in a list's options or redefine (1705.09999,
  // 1209.0532, 2009.04715 `\nocalcleftmargintrue`, V1.6's name).
  RawTeX!(r"\newdimen\IEEEiednormlabelsep \IEEEiednormlabelsep 0.6em
\newdimen\IEEEiedmathlabelsep \IEEEiedmathlabelsep 1.2em
\newskip\IEEEiedtopsep \IEEEiedtopsep 2pt plus 1pt minus 1pt
\def\IEEEiedlistdecl{\relax}
\newif\ifIEEEnocalcleftmargin \IEEEnocalcleftmarginfalse
\newif\ifIEEEnolabelindentfactor \IEEEnolabelindentfactorfalse");
  Ok(())
}

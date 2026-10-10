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
  engine::latex_constructs::{eqnarray_bindings_with_columns, math_array_bindings},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  define_ieeeeqnarray()?;
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
    let columns = ieee_columns(&Expand!(cols))?
      .into_iter()
      .map(|(before, after)| Cell { before: Some(before), after: Some(after), empty: true, ..Cell::default() })
      .collect();
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
    let columns = ieee_columns(&Expand!(cols))?
      .into_iter()
      .map(|(before, after)| {
        let (before, after) = math_cell_column(before, after);
        Cell { before: Some(before), after: Some(after), ..Cell::default() }
      })
      .collect();
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

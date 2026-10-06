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
//! (`\eqnarray`), which handles leading-empty cells correctly — so we surpass
//! Perl here. It mirrors the IEEEeqnarray defs in
//! `latexml_package/src/package/ieeetran_cls.rs` (Perl IEEEtran.cls.ltxml
//! L242-332); keep the two in sync.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // \IEEEeqnarray{cols} → \eqnarray (the `{rCl}` column spec is consumed and
  // discarded, exactly as IEEEtran.cls.ltxml L291-294:
  //   DefMacroI('\IEEEeqnarray', '{}', '\eqnarray')).
  // The compile-time DefMacro! form drops row-1 cell-1 (see ieeetran_cls.rs
  // note); the at-begin-document \def is the proven-working override.
  DefMacro!("\\IEEEeqnarray{}", "\\eqnarray");
  DefMacro!("\\endIEEEeqnarray", "\\endeqnarray");
  // Starred ENV handler: the CS name must literally be `\IEEEeqnarray*`
  // (env-begin csname lookup); the string prototype form would parse the
  // `*` as a literal PARAMETER and clobber the unstarred definition
  // ([[feedback_defmacro_starred]]).
  DefMacro!(T_CS!("\\IEEEeqnarray*"), Some(parse_parameters("{}", &T_CS!("\\IEEEeqnarray*"), true)?.unwrap()), Some(ExpansionBody::Tokens(Tokenize!(TeXString::assembled(r"\eqnarray*".to_string())))));
  Let!("\\endIEEEeqnarray*", "\\endeqnarray*");
  at_begin_document(TokenizeInternal!(
    r"\def\IEEEeqnarray#1{\eqnarray}\def\endIEEEeqnarray{\endeqnarray}\expandafter\def\csname IEEEeqnarray*\endcsname#1{\csname eqnarray*\endcsname}\expandafter\def\csname endIEEEeqnarray*\endcsname{\csname endeqnarray*\endcsname}"
  ))?;
  def_macro_noop("\\IEEEeqnarraynumspace")?;

  define_eqnarraybox()?;
  DefMacro!("\\IEEEeqnarraymulticol{}{}{}", "\\multicolumn{#1}{#2}{#3}");
  def_macro_noop("\\IEEEeqnarraydefcol{}{}{}")?;
  def_macro_noop("\\IEEEeqnarraydefcolsep{}{}")?;

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
});

/// The IEEEeqnarraybox family — a variant of `\array` (Perl IEEEtran.cls.ltxml L315-332), shared with the IEEEtran
/// class binding (`ieeetran_cls.rs`). `\ifmmode` dispatches to the math (m) or text (t) form.
pub fn define_eqnarraybox() -> Result<()> {
  RawTeX!(
    r"\def\IEEEeqnarraybox{\ifmmode\def\@tempa{\let\endIEEEeqnarraybox\endIEEEeqnarrayboxm\IEEEeqnarrayboxm}\else\def\@tempa{\let\endIEEEeqnarraybox\endIEEEeqnarrayboxt\IEEEeqnarrayboxt}\fi\@tempa}"
  );
  // IEEEtrantools.sty:2166-2173: `[<decl>][<pos>][<width>]{<cols>}` (Perl takes `{<cols>}` alone, so a box written
  // with its options read `[` as the column specification).
  DefMacro!("\\IEEEeqnarrayboxm OptionalMatch:* [][][] {}", sub[(star, decl, pos, width, cols)] {
    Ok(eqnarraybox_begin(star.is_some(), [decl, pos, width], &cols, Expand!(cols.clone()), false))
  });
  DefMacro!("\\endIEEEeqnarrayboxm", "\\lx@end@alignment\\@end@array");
  DefMacro!("\\IEEEeqnarrayboxt OptionalMatch:* [][][] {}", sub[(star, decl, pos, width, cols)] {
    Ok(eqnarraybox_begin(star.is_some(), [decl, pos, width], &cols, Expand!(cols.clone()), true))
  });
  DefMacro!(
    "\\endIEEEeqnarrayboxt",
    "\\lx@end@alignment\\@end@array\\lx@end@inline@math"
  );
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

/// The alignment an IEEEeqnarraybox opens over the column letters of `expanded`, the specification `cols` fully
/// expanded as IEEEtrantools.sty:2280 `\edef`s it (macro containers); `\@@IEEE@array` keeps `cols` whole for the
/// reversion. A specification's inter-column glue (IEEEtrantools.sty:2370-2420: `! , : ; ' " . / ? * + -`, and the
/// digits of the glues `\IEEEeqnarraydefcolsep` numbers) spaces the columns and makes none, so `{,c/c/c,}` is three
/// centred columns; the template here sets no column glue.
fn eqnarraybox_begin(
  star: bool,
  options: [Option<Tokens>; 3],
  cols: &Tokens,
  expanded: Tokens,
  text: bool,
) -> Tokens {
  let mut depth = 0usize;
  let letters = expanded
    .unlist_ref()
    .iter()
    .filter(|t| match t.get_catcode() {
      Catcode::BEGIN => {
        depth += 1;
        true
      },
      Catcode::END => {
        depth = depth.saturating_sub(1);
        true
      },
      code => {
        depth > 0
          || code == Catcode::CS
          || !with(t.text, |c| {
            c.len() == 1 && "!,:;'\"./?*+-0123456789".contains(c)
          })
      },
    });
  let letters: Vec<Token> = letters.copied().collect();
  let mut out = Vec::new();
  if text {
    out.push(T_CS!("\\lx@begin@inline@math"));
  }
  out.push(T_CS!("\\@array@bindings"));
  out.push(T_BEGIN!());
  out.extend(letters);
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

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
  DefMacro!("\\IEEEeqnarray*{}", "\\eqnarray*");
  Let!("\\endIEEEeqnarray*", "\\endeqnarray*");
  at_begin_document(TokenizeInternal!(
    r"\def\IEEEeqnarray#1{\eqnarray}\def\endIEEEeqnarray{\endeqnarray}\expandafter\def\csname IEEEeqnarray*\endcsname#1{\csname eqnarray*\endcsname}\expandafter\def\csname endIEEEeqnarray*\endcsname{\csname endeqnarray*\endcsname}"
  ))?;
  def_macro_noop("\\IEEEeqnarraynumspace")?;

  // IEEEeqnarraybox — treated as a variant of \array (Perl IEEEtran.cls.ltxml
  // L315-332). \ifmmode dispatches to the math (m) or text (t) form.
  RawTeX!(
    r"\def\IEEEeqnarraybox{\ifmmode\def\@tempa{\let\endIEEEeqnarraybox\endIEEEeqnarrayboxm\IEEEeqnarrayboxm}\else\def\@tempa{\let\endIEEEeqnarraybox\endIEEEeqnarrayboxt\IEEEeqnarrayboxt}\fi\@tempa}"
  );
  DefMacro!("\\IEEEeqnarrayboxm OptionalMatch:* {}",
    "\\@array@bindings{#2}\\@@IEEE@array{#2}\\lx@begin@alignment");
  DefMacro!("\\endIEEEeqnarrayboxm", "\\lx@end@alignment\\@end@array");
  DefMacro!("\\IEEEeqnarrayboxt OptionalMatch:* {}",
    "\\lx@begin@inline@math\\@array@bindings{#2}\\@@IEEE@array{#2}\\lx@begin@alignment");
  DefMacro!("\\endIEEEeqnarrayboxt",
    "\\lx@end@alignment\\@end@array\\lx@end@inline@math");
  DefConstructor!("\\@@IEEE@array[] Undigested DigestedBody", "#3",
    before_digest => { bgroup(); },
    reversion => "\\begin{IEEEeqnarraybox}[#1]{#2}#3\\end{IEEEeqnarraybox}");
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
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        after: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("C", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        after:  Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("R", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
});

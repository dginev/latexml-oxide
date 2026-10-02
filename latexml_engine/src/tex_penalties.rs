//! TeX Penalties
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Penalties Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // Adding/removing penalties
  //----------------------------------------------------------------------
  // \penalty          c  adds a penalty to the current list.
  // \unpenalty        c  removes a penalty from the current list.
  // \lastpenalty      iq is 0 or the last penalty on the current list.
  // tex.web §1103: in vertical mode a penalty is a node of the list — `\vsplit` breaks at it (§974, a
  // `\penalty-10000` forces the break; short-math-guide's `\null\penalty-\@M` before each symbol list),
  // `\lastpenalty` reads it (§424) and `\unpenalty` removes it (the `\unkern` shape, tex_kern.rs). It is an
  // empty box, as Perl's undef primitive makes (TeX_Penalties.pool.ltxml:29, Primitive.pm `invoke`): it absorbs to
  // nothing. `\lastbox` is void at it (§1080): Perl's box was taken by `\lastbox`, so
  // `\loop \unskip\unpenalty\unskip\unpenalty \setbox0\lastbox \ifvoid0…` (caesar_book.cls:106-115 counting
  // title lines; sidenotes caesar_example) gained a box per iteration and never ended, in Perl too. Guard:
  // `perfect_kernel_batch56::unpenalty_does_not_grow_the_box_list`, `perfect_kernel_batch59::vsplit_breaks_at_penalties`.
  // In horizontal and math mode a penalty stays
  // nothing, so `\lastpenalty` there is 0 where TeX gives the value (OXIDIZED_DESIGN_DIVERGENCES #419).
  DefPrimitive!("\\penalty Number", sub[(n)] {
    let value = n.value_of();
    if !lookup_string_from_sym(pin!("MODE")).ends_with("vertical") {
      return Ok(Vec::new());
    }
    let mut reversion = vec![T_CS!("\\penalty")];
    reversion.extend(ExplodeText!(&value.to_string()));
    let mut props = SymHashMap::default();
    props.insert("isEmpty", Stored::Bool(true));
    props.insert("isPenalty", Stored::Bool(true));
    props.insert("penalty", Stored::Int(value));
    Ok(vec![Digested::from(Tbox::new(pin!(""), None, None, Tokens::new(reversion), props))])
  });
  DefPrimitive!("\\unpenalty", {
    let mut comments = Vec::new();
    while let Some(last_box) = pop_own_box() {
      if matches!(last_box.data(), DigestedData::Comment(_)) {
        comments.push(last_box);
      } else {
        if !last_box.get_property_bool("isPenalty") {
          push_box_list(last_box);
        }
        break;
      }
    }
    for comment in comments.into_iter().rev() {
      push_box_list(comment);
    }
  });
  // tex.web §424: the last item's penalty when it is one, else 0 (the `\lastkern` shape, tex_kern.rs).
  DefRegister!("\\lastpenalty" => Number::new(0), readonly => true,
  getter => {
    with_own_box_list(|list| {
      list
        .iter()
        .rev()
        .find(|item| !matches!(item.data(), DigestedData::Comment(_)))
        .filter(|item| item.get_property_bool("isPenalty"))
        .and_then(|item| match item.get_property("penalty").as_deref() {
          Some(Stored::Int(value)) => Some(Number::new(*value)),
          _ => None,
        })
        .unwrap_or_else(|| Number::new(0))
    })
  });

  //======================================================================
  // values for various penalties
  //----------------------------------------------------------------------
  // \brokenpenalty    pi is the penalty added after a line ending with an hyphenated word.
  // \clubpenalty      pi is the penalty added after the first line in a paragraph.
  // \exhyphenpenalty  pi is the penalty for a line break after an explicit hyphen.
  // \floatingpenalty  pi is the penalty for insertions that are split between pages.
  // \hyphenpenalty    pi is the penalty for a line break after a discretionary hyphen.
  // \interlinepenalty pi is the penalty added between lines in a paragraph.
  // \linepenalty      pi is an amount added to the \badness calculated for every line in a
  // paragraph. \outputpenalty    pi holds the penalty from the current page break.
  // \widowpenalty     pi is the penalty added after the penultimate line in a paragraph.
  DefRegister!("\\brokenpenalty", Number!(100));
  DefRegister!("\\clubpenalty", Number!(150));
  DefRegister!("\\exhyphenpenalty", Number!(50));
  DefRegister!("\\floatingpenalty", Number!(0));
  DefRegister!("\\hyphenpenalty", Number!(50));
  DefRegister!("\\interlinepenalty", Number!(0));
  DefRegister!("\\linepenalty", Number!(10));
  DefRegister!("\\outputpenalty", Number!(0));
  DefRegister!("\\widowpenalty", Number!(150));
});

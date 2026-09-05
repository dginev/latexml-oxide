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
  // Perl TeX_Penalties.pool.ltxml:29-30 makes both `undef` primitives, but an
  // undef primitive digests to a Whatsit that is PUSHED on the box list, so
  // `\loop \unskip\unpenalty\unskip\unpenalty \setbox0\lastbox \ifvoid0…`
  // (caesar_book.cls:106-115 counting title lines; sidenotes caesar_example)
  // gains a box per iteration and never sees the void box — an unbounded
  // runaway in Perl too, while pdflatex terminates. Penalties are not boxes
  // here: `\penalty` produces nothing and `\unpenalty` removes a last
  // `isPenalty` box if one ever exists (the `\unkern` shape, tex_kern.rs).
  // Guard: `perfect_kernel_batch56::unpenalty_does_not_grow_the_box_list`.
  DefPrimitive!("\\penalty Number", sub[(_n)] { Ok(Vec::new()) });
  DefPrimitive!("\\unpenalty", {
    let mut comments = Vec::new();
    while let Some(last_box) = pop_box_list() {
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
  DefRegister!("\\lastpenalty", Number::new(0), readonly => true);

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

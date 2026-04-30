use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl pgfcircutils.tex.ltxml L22-37 defines `\pgf@circ@stripdecimals`
  // as a DefMacro (gullet-level, returns raw tokens) that reads tokens
  // until `\pgf@nil` and drops everything from the first `.` onward.
  // The previous Rust port wired this through `DefPrimitive`, which runs
  // at the stomach level and emits Digested boxes — the wrong category
  // for callers like pgf/tikz that pass the result through \expandafter
  // / \edef chains. Switch to `DefMacro` with a token-returning closure
  // so the semantics line up with Perl (and pgfcircutils's own callers).
  DefMacro!("\\pgf@circ@stripdecimals Until:\\pgf@nil", sub[(arg)] {
    let dot = T_OTHER!(".");
    let mut leading = Vec::new();
    for t in arg.unlist() {
      if t == dot { break; }
      leading.push(t);
    }
    Ok(Tokens::new(leading))
  }, locked => true);

  InputDefinitions!("pgfcircutils", noltxml => true, extension => Some(Cow::Borrowed("tex")));
});

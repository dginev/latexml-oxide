use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: pgfcircutils.tex.ltxml (44 lines)
  // Custom \pgf@circ@stripdecimals that handles missing decimal points.
  // The Perl version uses "Until:\pgf@nil" parameter to read tokens until delimiter.
  DefPrimitive!("\\pgf@circ@stripdecimals Until:\\pgf@nil", sub[(arg)] {
    // Drop everything from the decimal point onward; return integer part
    let mut leading = Vec::new();
    for t in arg.unlist() {
      if t == T_OTHER!(".") {
        break;
      }
      leading.push(Digested::from(Tbox::new(pin!(""), None, None,
        Tokens::new(vec![t]), arena::SymHashMap::default())));
    }
    Ok(leading)
  }, locked => true);

  InputDefinitions!("pgfcircutils", noltxml => true, extension => Some(Cow::Borrowed("tex")));
});

use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("epsfig");
  // psfig.sty 1.10:122-123 (shipped by 0712.1995): `\def\figurepath{./:}` and
  // `\def\psfigurepath#1{\edef\figurepath{#1:}}`, the `:`-separated directories `\psfig` searches for its file (:162
  // `\DoPaths\figurepath`); each goes to the graphics search path as `\graphicspath` puts it. Witnesses
  // cond-mat0010435, cond-mat0012297 (`\psfigurepath{.:plot:figure}`); OXIDIZED_DESIGN_DIVERGENCES #1088.
  RawTeX!(r"\def\figurepath{./:}
\def\psfigurepath#1{\edef\figurepath{#1:}\expandafter\lx@psfig@graphicspath\expandafter{\figurepath}}");
  DefMacro!("\\lx@psfig@graphicspath{}", sub[(paths)] {
    let paths = paths.to_string();
    let mut out = vec![T_CS!("\\graphicspath"), T_BEGIN!()];
    for dir in paths.split(':').map(str::trim).filter(|dir| !dir.is_empty()) {
      out.push(T_BEGIN!());
      out.extend(ExplodeText!(dir));
      out.push(T_END!());
    }
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
});

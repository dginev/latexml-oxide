//! spotcolor.sty (Jens Elstner, 2006; no Perl binding): read raw, as before. Its `\AddSpotColor{<space>}{<name>}
//! {<PDF name>}{<C M Y K>}` (spotcolor.sty:31-41) writes the spot colour's CMYK alternate only into a PDF object, so
//! the binding keeps it for color's driver models (`color_sty::driver_alternate_color`): a `\definecolor{<n>}
//! {spotcolor}{<name>,<tint>}` that defines `<n>` first takes the alternate at that tint — the colour a viewer without
//! the spot ink shows (the Separation's tint transform, C0 0 0 0 0 to C1 the CMYK, N 1). ieeeaccess.cls:126-131's
//! `PANTONE3015C` (CMYK 1 0.3 0 0.2).
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("spotcolor", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true);
  // (the package's own `\AddSpotColor` runs as it was; its CMYK is kept first)
  RawTeX!(
    r"\let\lx@spot@AddSpotColor\AddSpotColor
\def\AddSpotColor#1#2#3#4{\lx@spot@alternate{#2}{#4}\lx@spot@AddSpotColor{#1}{#2}{#3}{#4}}"
  );
  DefPrimitive!("\\lx@spot@alternate{}{}", sub[(name, cmyk)] {
    let cmyk = do_expand(cmyk)?.to_string();
    let values: Vec<&str> = cmyk.split_whitespace().collect();
    if values.len() == 4 && values.iter().all(|v| v.parse::<f64>().is_ok()) {
      AssignValue!(&s!("spotcolor_alternate:{}", name.to_string().trim()) => values.join(","), Some(Scope::Global));
    }
  });
  // color.sty's driver model `spotcolor` (spotcolor.sty:71 `\color@spotcolor`): `<name>,<tint>` (`\c@lor@@spotcolor`,
  // :72-75) → the CMYK alternate scaled by the tint, or nothing for a spot colour no `\AddSpotColor` gave.
  DefMacro!("\\lx@color@alternate@spotcolor{}", sub[(spec)] {
    let spec = spec.to_string();
    let (name, tint) = spec.split_once(',').unwrap_or((spec.as_str(), "1"));
    let tint: f64 = tint.trim().parse().unwrap_or(1.0);
    let alternate = lookup_string(&s!("spotcolor_alternate:{}", name.trim()));
    if alternate.is_empty() {
      return Ok(Tokens::default());
    }
    let scaled: Vec<String> = alternate
      .split(',')
      .filter_map(|v| v.parse::<f64>().ok())
      .map(|v| s!("{}", (v * tint).clamp(0.0, 1.0)))
      .collect();
    Ok(mouth::tokenize_internal(TeXString::assembled(s!("{{cmyk}}{{{}}}", scaled.join(",")))))
  });
});
